use chb_chess::Color;
use leptos::{component, create_signal, event_target_value, view, IntoView, Scope, Signal};
use web_sys::Event;

use crate::board_provider::create_server_board;
use crate::chess_board::{ChessBoard, ChessBoardProps};

#[component]
pub fn Play(cx: Scope) -> impl IntoView {
    let (board, make_move) = create_server_board(cx);
    let (play_as, set_play_as) = create_signal(cx, Some(Color::White));
    let view_as = Signal::derive(cx, move || play_as().unwrap_or(Color::White));
    let change_player = move |e: Event| {
        let player = event_target_value(&e).parse::<Color>().ok();
        set_play_as(player);
    };

    view! {
        cx,
        <>
            <ChessBoard
                board=board
                make_move=make_move
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
                            prop:checked=move || play_as().is_none()
                        />
                        <label for="play-as-none">"Spectate"</label>
                    </div>
                </fieldset>
            </div>
        </>
    }
}
