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
use chb_chess::{Board, BoardBuilder, Move};
use futures::{SinkExt, StreamExt};
use leptos::log;

use crate::{board_state::BoardState, code_gen::get_code, BoardList};

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
            .board()
            .await
            .clone(),
    ))
}

pub async fn create_board(
    State(locked_board_list): State<BoardList>,
    Json(builder): Json<Option<BoardBuilder>>,
) -> Result<String, StatusCode> {
    let mut board_list = locked_board_list.write().await;
    let mut id = get_code();
    // Probably not necessary, but might as well
    while board_list.contains_key(&id) {
        id = get_code();
    }
    let board = if let Some(bb) = builder {
        bb.build().map_err(|_| StatusCode::BAD_REQUEST)?
    } else {
        Board::default()
    };
    board_list.insert(id.clone(), Arc::new(BoardState::new(board)));
    Ok(id)
}

pub async fn subscribe_to_board(
    wsu: WebSocketUpgrade,
    State(locked_board_list): State<BoardList>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let mut board_list = locked_board_list.write().await;
    let board_state = match board_list.get(&id) {
        Some(bs) => bs.clone(),
        None => {
            log!("Creating new board");
            let bs = Arc::new(BoardState::init());
            board_list.insert(id, bs.clone());
            log!("saved new board");
            bs
        }
    };

    wsu.on_upgrade(|ws: WebSocket| async move {
        sync_board(ws, board_state).await;
    })
}

async fn sync_board(stream: WebSocket, board_state: Arc<BoardState>) {
    // Send fen to update local board
    let (mut writer, mut reader) = stream.split();
    let _ = writer
        .send(Message::Text(format!("fen:{}", board_state.fen().await)))
        .await;
    let mut rx = board_state.subscribe();
    let mut outbound = tokio::spawn(async move {
        while let Ok(m) = rx.recv().await {
            match writer.send(Message::Text(format!("move: {m}"))).await {
                Ok(_) => (),
                Err(e) => log!("Failed to send message to websocket: {e}"),
            };
        }
    });
    let mut inbound = tokio::spawn(async move {
        while let Some(Ok(msg)) = reader.next().await {
            let text = match msg {
                Message::Text(t) => t,
                Message::Close(_) => break,
                a => {
                    log!("Received unknown message: {:?}", a);
                    continue;
                }
            };
            match text.split_once(':') {
                Some(("move", p)) if p.trim().parse::<Move>().is_ok() => {
                    board_state.make(p.trim().parse().expect("validated")).await;
                }
                b => log!("invalid argument: {:?}", b),
            }
        }
    });
    tokio::select! {
        _ = (&mut outbound) => inbound.abort(),
        _ = (&mut inbound) => outbound.abort(),
    }
}
