//use futures::channel::mpsc::{self, UnboundedReceiver, UnboundedSender};
//use std::{future::poll_fn, task::Poll};
use async_std::channel::{bounded, unbounded, Receiver, Sender};

use chb_chess::{Board, Move};
use leptos::{
    create_signal, log, spawn_local, ReadSignal, Scope, SignalSetter, SignalUpdate, WriteSignal,
};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{MessageEvent, WebSocket};

pub type Provider = (ReadSignal<Board>, SignalSetter<Move>);

pub fn create_server_board(cx: Scope) -> Provider {
    let (board, set_board) = create_signal(cx, Board::default());
    let (local_tx, local_rx) = unbounded::<Move>();
    let (server_tx, server_rx) = unbounded::<Move>();

    let make_move = SignalSetter::map(cx, move |mv: Move| {
        match local_tx.send_blocking(mv) {
            Ok(_) => (),
            Err(e) => log!("Failed to send move to thread {:?}", e),
        };
    });

    spawn_local(make_server_moves(server_rx, set_board));
    spawn_local(start_board_sync(server_tx, local_rx));
    (board, make_move)
}

async fn make_server_moves(server_rx: Receiver<Move>, set_board: WriteSignal<Board>) {
    let make_move = move |m| {
        set_board.update(|b| {
            let _ = b.make(m);
        });
    };
    poll_rx(server_rx, make_move).await;
}

async fn start_board_sync(tx: Sender<Move>, rx: Receiver<Move>) {
    let ws = WebSocket::new("wss://echo.websocket.events").unwrap();

    let on_message = Closure::<dyn Fn(_)>::new(move |e: MessageEvent| {
        log!("message received: {:?}", e.data());
        log!("need to parse message!!!");
    });
    ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
    on_message.forget();
    let make_move = |mv: Move| {
        match ws.send_with_str(&format!("move:{}", mv)) {
            Ok(_) => (),
            Err(e) => log!("Failed to send move to websocket: {:?}", e),
        };
        let _ = tx.send_blocking(mv);
    };
    poll_rx(rx, make_move).await;
}

async fn poll_rx<T, F: Fn(T)>(rx: Receiver<T>, f: F) {
    loop {
        match rx.recv().await {
            Ok(x) => f(x),
            Err(e) => log!("Failed to receive: {}", e),
        }
    }

}
