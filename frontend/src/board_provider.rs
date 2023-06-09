use async_std::channel::{unbounded, Receiver};

use chb_chess::{Board, Move};
use leptos::*;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{MessageEvent, WebSocket};

pub type Provider = (ReadSignal<Board>, SignalSetter<Move>);

pub fn get_board(cx: Scope, id: String) -> Provider {
    log!("id={id}");
    let (board, set_board) = create_signal(cx, Board::default());
    let (local_tx, local_rx) = unbounded::<Move>();

    let make_move = SignalSetter::map(cx, move |mv: Move| {
        match local_tx.send_blocking(mv) {
            Ok(_) => (),
            Err(e) => log!("Failed to send move to thread {:?}", e),
        };
    });

    spawn_local(start_board_sync(set_board, id, local_rx));
    (board, make_move)
}

async fn start_board_sync(set_board: WriteSignal<Board>, id: String, rx: Receiver<Move>) {
    let ws = match WebSocket::new(&format!("ws://localhost:3000/api/board/{id}/subscribe")) {
        Ok(w) => w,
        Err(e) => {
            log!("Error connecting to websocket: {:?}", e);
            return;
        }
    };

    let on_message = Closure::<dyn Fn(_)>::new(move |e: MessageEvent| {
        if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
            let txt: String = txt.into();
            match txt.split_once(':') {
                Some(("fen", f)) => {
                    if let Ok(b) = f.trim().parse::<Board>() {
                        set_board(b);
                    }
                }
                Some(("move", m)) if m.trim().parse::<Move>().is_ok() => {
                    set_board.update(|b| {
                        if b.make(m.trim().parse().expect("Validated")).is_err() {
                            log!("BOARD OUT OF SYNC!");
                        }
                    });
                }
                _ => (),
            }
        }
    });
    ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
    on_message.forget();

    let make_move = |mv: Move| {
        match ws.send_with_str(&format!("move:{}", mv)) {
            Ok(_) => (),
            Err(e) => log!("Failed to send move to websocket: {:?}", e),
        };
    };
    poll_rx(rx, make_move).await;
}

async fn poll_rx<T, F: Fn(T)>(rx: Receiver<T>, f: F) {
    while let Ok(x) = rx.recv().await {
        f(x);
    }
}
