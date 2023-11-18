use chb_chess::{Board, Color, Move};
use futures::{stream::SplitStream, StreamExt};
use gloo_net::websocket::{futures::WebSocket, Message};
use leptos::*;

// How to stop these from running when hydrating?
pub type Provider = (ReadSignal<Board>, SignalSetter<Move>);

pub fn spectate_board(cx: Scope, id: String) -> ReadSignal<Board> {
    let ws = WebSocket::open(&format!("ws://localhost:3000/api/board/{id}/subscribe")).unwrap();
    board_signal_from_stream(cx, ws.split().1)
}

pub fn play_board(cx: Scope, id: String, play_as: Color) -> Provider {
    log!("Playing board {id} as {play_as}");
    let (board, set_board) = create_signal(cx, Board::default());
    spawn_local(async move {
        let url = format!("ws://localhost:3000/api/board/join/{id}/{play_as}");
        log!("Url: {url}");
        let ws = WebSocket::open(&url).unwrap();
        let (mut _write, read) = ws.split();
        let board = board_signal_from_stream(cx, read);
    });

    let make_move = SignalSetter::map(cx, |_mv: Move| {
        todo!();
    });

    (board, make_move)
}

fn board_signal_from_stream(cx: Scope, stream: SplitStream<WebSocket>) -> ReadSignal<Board> {
    let (board, set_board) = create_signal(cx, Board::default());
    spawn_local(async move {
        stream
            .for_each(|m| async move {
                let Ok(Message::Text(m)) = m else {
                    return;
                };
                match m.split_once(':') {
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
                    Some(_) => (),
                    None => (),
                }
            })
            .await;
    });
    board
}
