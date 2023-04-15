use chb_chess::{Board, Color, Move, Square};
use leptos::*;

use piece::{PieceDisplay, PieceDisplayProps};

mod piece;

#[component]
pub fn ChessBoard(
    cx: Scope,
    #[prop(into)] board: Signal<Board>,
    #[prop(into)] make_move: SignalSetter<Move>,
    #[prop(into)] view_as: MaybeSignal<Color>,
    #[prop(into)] play_as: MaybeSignal<Option<Color>>,
) -> impl IntoView {
    let reverse_board = Signal::derive(cx, move || view_as() == Color::Black);
    let squares = move || {
        log!("Running memo");
        let mut list = (0u32..64u32)
            .map(|i| {
                let sqr = Square::try_from(i).expect("0-63 are valid squares");
                let piece = move || board.with(|b| b[sqr]);
                let may_move = move || {
                    board.with(|b| play_as() == Some(b.color_to_move()))
                        && piece().color() == play_as()
                };
                (sqr, piece.derive_signal(cx), may_move.derive_signal(cx))
            })
            .collect::<Vec<_>>();
        if view_as() == Color::Black {
            list.reverse();
        }
        list
    };

    view! {
        cx,
        <>
            <link rel="stylesheet" href="chess_board.css"/>
            <div class="chess-board">
                <For
                    each=squares
                    key=|sqr| sqr.0.to_string()
                    view=move |cx, (sqr, piece, may_move)| {
                        let even = (sqr.rank() + sqr.file()) % 2 == 0;
                        view! {
                            cx,
                            <div class="square" class:dark=even>
                                <PieceDisplay
                                    piece=piece
                                    square=sqr
                                    board_reversed=reverse_board
                                    may_move=may_move
                                    make_move=make_move
                                />
                            </div>
                        }
                    }
                />
            </div>
        </>
    }
}
