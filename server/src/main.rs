use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::IntoResponse,
    routing::get,
    Extension, Router,
};
use board_state::BoardState;
use chb_chess::Move;
use frontend::{App, AppProps};
use futures::{SinkExt, StreamExt};
use leptos::{get_configuration, view};
use leptos_axum::{generate_route_list, LeptosRoutes};

use crate::fallback::file_handler;

mod board_state;
mod fallback;

#[tokio::main]
async fn main() {
    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(|cx| view! {cx, <App/> }).await;

    let bs = BoardState::init();
    let state = Arc::new(bs);

    let app = Router::new()
        .route("/board/subscribe", get(ws_board))
        .with_state(state.clone())
        .leptos_routes(leptos_options.clone(), routes, |cx| view! {cx, <App/>})
        .fallback(file_handler)
        .layer(Extension(Arc::new(leptos_options)));

    println!("Server Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn ws_board(
    wsu: WebSocketUpgrade,
    State(app_state): State<Arc<BoardState>>,
) -> impl IntoResponse {
    wsu.on_upgrade(|ws: WebSocket| async move {
        sync_board(ws, app_state).await;
    })
}

async fn sync_board(stream: WebSocket, board_state: Arc<BoardState>) {
    // Send fen to update local board
    let (mut writer, mut reader) = stream.split();
    let _ = writer
        .send(Message::Text(format!(
            "fen:{}",
            board_state.board().await.to_fen()
        )))
        .await;
    let mut rx = board_state.subscribe();
    let mut outbound = tokio::spawn(async move {
        while let Ok(m) = rx.recv().await {
            match writer.send(Message::Text(format!("move: {m}"))).await {
                Ok(_) => (),
                Err(e) => println!("Failed to send message to websocket: {e}"),
            };
        }
    });
    let mut inbound = tokio::spawn(async move {
        while let Some(Ok(msg)) = reader.next().await {
            let text = match msg {
                Message::Text(t) => t,
                Message::Close(_) => break,
                a => {
                    println!("Received unknown message: {:?}", a);
                    continue;
                }
            };
            match text.split_once(':') {
                Some(("move", p)) if p.trim().parse::<Move>().is_ok() => {
                    board_state.make(p.trim().parse().expect("validated")).await;
                }
                b => println!("invalid argument: {:?}", b),
            }
        }
    });
    tokio::select! {
        _ = (&mut outbound) => inbound.abort(),
        _ = (&mut inbound) => outbound.abort(),
    }
}
