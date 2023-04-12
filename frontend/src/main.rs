use chb_chess::{Board, Color};
use leptos::{
    component, create_signal, ev::MouseEvent, event_target_value, mount_to_body, provide_context,
    view, IntoView, Scope, Signal,
};

use chess_board::{ChessBoard, ChessBoardProps};
use web_sys::Event;

mod chess_board;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    let (board, set_board) = create_signal(cx, Board::default());
    let (mouse, set_mouse) = create_signal(cx, (0, 0));
    provide_context(cx, mouse);
    let mouse_move = move |e: MouseEvent| set_mouse((e.client_x(), e.client_y()));

    let (play_as, set_play_as) = create_signal(cx, Some(Color::White));
    let view_as = Signal::derive(cx, move || play_as().unwrap_or(Color::White));
    let change_player = move |e: Event| {
        let player = event_target_value(&e).parse::<Color>().ok();
        set_play_as(player);
    };

    view! {
        cx,
        <div class="main" on:mousemove=mouse_move>
            <ChessBoard
                board=board
                set_board=set_board
                play_as=play_as
                view_as=view_as
            />
            <div class="board-controls">
                <fieldset>
                    <legend>"Play as"</legend>
                    <div>
                        <input
                            type="radio"
                            id="play-as-white"
                            name="play_as"
                            value=Color::White.to_string()
                            on:change=change_player
                            prop:checked=move || play_as() == Some(Color::White)
                        />
                        <label for="play-as-white">"White"</label>
                    </div>
                    <div>
                        <input
                            type="radio"
                            id="play-as-black"
                            name="play_as"
                            value=Color::Black.to_string()
                            on:change=change_player
                            prop:checked=move || play_as() == Some(Color::Black)
                        />
                        <label for="play-as-black">"Black"</label>
                    </div>
                    <div>
                        <input
                            type="radio"
                            id="play-as-none"
                            name="play_as"
                            value="-"
                            on:change=change_player
                            prop:checked=move || play_as() == None
                        />
                        <label for="play-as-none">"Spectate"</label>
                    </div>
                </fieldset>
            </div>
        </div>
    }
}

pub fn main() {
    mount_to_body(|cx| view! { cx, <App/> })
}
