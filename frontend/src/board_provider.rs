use chb_chess::{Board, Move};
use leptos::{
    create_signal, log, ReadSignal, Scope, SignalSetter, SignalUpdate, WriteSignal,
};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{MessageEvent, WebSocket};

pub type Provider = (ReadSignal<Board>, SignalSetter<Move>);

pub fn create_local_board(cx: Scope) -> Provider {
    let (board, set_board) = create_signal(cx, Board::default());
    let make_move = SignalSetter::map(cx, move |mv: Move| {
        set_board.update(|b| {
            let _ = b.make(mv);
        })
    });
    (board, make_move)
}
pub fn create_server_board(cx: Scope) -> Provider {
    // Would be request to set up board, just want to test starting a new thread and modifying
    // board from there
    let (board, set_board) = create_signal(cx, Board::default());
    let make_move = SignalSetter::map(cx, move |mv: Move| {
        set_board.update(|b| {
            let _ = b.make(mv);
        })
    });
    // Start a Websocket to sync board with server
    sync_board(board, set_board);
    (board, make_move)
}

pub fn sync_board(board: ReadSignal<Board>, set_board: WriteSignal<Board>) {
    let ws = WebSocket::new("wss://echo.websocket.events").unwrap();

    let on_message = Closure::<dyn Fn(_)>::new(move |e: MessageEvent| {
        log!("message received: {:?}", e.data());

        
    });
    ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
    on_message.forget();

    let on_opened = Closure::<dyn Fn()>::new(move || {
        log!("Websocket connected!");
    });

    ws.set_onopen(Some(on_opened.as_ref().unchecked_ref()));
    on_opened.forget();
}
