use api::join::JoinBoard;
use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chb_chess::{Board, BoardBuilder, Color};
use futures::{SinkExt, StreamExt};
use leptos::log;
use tokio::sync::Mutex;

use crate::{
    code_gen::get_code,
    game::{ExecExt, Game},
    participant::web_player::WebPlayer,
    BoardList,
};

pub async fn get_board(
    State(locked_board_list): State<BoardList>,
    Path(id): Path<String>,
) -> Result<Json<Board>, StatusCode> {
    Ok(Json(
        locked_board_list
            .read()
            .await
            .get(&id)
            .ok_or(StatusCode::NOT_FOUND)?
            .lock()
            .await
            .board()
            .clone(),
    ))
}

pub async fn create_board(
    State(locked_board_list): State<BoardList>,
    Json(builder): Json<Option<BoardBuilder>>,
) -> Result<String, StatusCode> {
    let board = if let Some(bb) = builder {
        bb.build().map_err(|_| StatusCode::BAD_REQUEST)?
    } else {
        Board::default()
    };
    let mut board_list = locked_board_list.write().await;
    let mut id = get_code();
    // Probably not necessary, but might as well
    while board_list.contains_key(&id) {
        id = get_code();
    }
    board_list.insert(id.clone(), Arc::new(Mutex::new(Game::new(board))));
    Ok(id)
}

pub async fn subscribe_to_board(
    wsu: WebSocketUpgrade,
    State(locked_board_list): State<BoardList>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let board_list = locked_board_list.read().await;
    let board_state = match board_list.get(&id) {
        Some(bs) => bs.clone(),
        None => return Err(StatusCode::NOT_FOUND),
    };

    Ok(wsu.on_upgrade(|ws: WebSocket| async move {
        sync_board(ws, board_state).await;
    }))
}

pub async fn join_board(
    wsu: WebSocketUpgrade,
    State(locked_board_list): State<BoardList>,
    Path(id): Path<String>,
    Path(play_as): Path<Color>,
) -> impl IntoResponse {
    // Should really check if the player of that color is already set.
    log!("Joining board {id} as {play_as}");
    let game = match locked_board_list.read().await.get(&id) {
        Some(g) => g.clone(),
        None => return Err(StatusCode::NOT_FOUND),
    };

    Ok(wsu.on_upgrade(move |mut ws: WebSocket| async move {
        let mut g = game.lock().await;
        _ = ws.send(Message::Text(format!("fen: {}", g.board()))).await;
        g.set_player(play_as, Some(Arc::new(Mutex::new(WebPlayer::connect(ws)))));

        if g.is_active() {
            drop(g);
            game.start();
        }
    }))
}

async fn sync_board(stream: WebSocket, locked_game: Arc<Mutex<Game>>) {
    let game = locked_game.lock().await;
    // Send fen to update local board
    let (mut writer, _) = stream.split();
    let _ = writer
        .send(Message::Text(format!("fen:{}", game.fen())))
        .await;
    let mut rx = game.watch();

    while let Ok(m) = rx.recv().await {
        match writer.send(Message::Text(format!("move: {m}"))).await {
            Ok(_) => (),
            Err(e) => log!("Failed to send message to websocket: {e}"),
        };
    }
}
