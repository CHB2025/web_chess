use chb_chess::Board;
use leptos::{component, create_signal, ev::MouseEvent, mount_to_body, view, IntoView, Scope, provide_context};

use chess_board::{ChessBoard, ChessBoardProps};

mod chess_board;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    let (board, set_board) = create_signal(cx, Board::default());
    let (mouse, set_mouse) = create_signal(cx, (0, 0));
    provide_context(cx, mouse);
    let mouse_move = move |e: MouseEvent| set_mouse((e.client_x(), e.client_y()));

    view! {
        cx,
        <div class="main" on:mousemove=mouse_move>
            <ChessBoard board=board set_board=set_board/>
        </div>
    }
}

pub fn main() {
    mount_to_body(|cx| view! { cx, <App/> })
}
