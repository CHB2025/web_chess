use chb_chess::{Board, Dir, Move, Piece, Square};
use leptos::{ev::MouseEvent, *};
use web_sys::HtmlDivElement;

#[component]
pub fn ChessBoard(
    cx: Scope,
    #[prop(into)] board: Signal<Board>,
    #[prop(into)] set_board: WriteSignal<Board>,
) -> impl IntoView {
    let (held_piece, set_held_piece) = create_signal::<Option<Square>>(cx, None);
    let (held_origin, set_held_origin) = create_signal(cx, (0, 0));
    let mouse = use_context::<ReadSignal<(i32, i32)>>(cx).expect("Mouse signal to exist");
    let translate = (move || {
        let (m_x, m_y) = mouse();
        let (o_x, o_y) = held_origin();
        (m_x - o_x, m_y - o_y)
    })
    .derive_signal(cx);

    let squares = (0u32..64u32).map(|i| {
        let sqr = move || Square::try_from(i).expect("0-63 are valid squares");
        let piece = move || board.with(|b| b[sqr()]);
        let style = move || match held_piece() {
            Some(s) if s == sqr() => {
                translate.with(|(x, y)| format!("transform:translate({x}px, {y}px)"))
            }
            _ => "".to_owned(),
        };
        (
            sqr.derive_signal(cx),
            piece.derive_signal(cx),
            style.derive_signal(cx),
        )
    });
    let mouse_up = move |e: MouseEvent| {
        let target: HtmlDivElement = event_target(&e);
        let (x, y) = translate();
        let dx = (x.abs() + target.offset_width() / 2) / target.offset_width();
        let dy = (y.abs() + target.offset_height() / 2) / target.offset_height();
        let mut dest = held_piece();
        for _ in 0..dx {
            dest = match (dest, x.is_negative()) {
                (Some(s), true) => s.checked_add(Dir::West),
                (Some(s), false) => s.checked_add(Dir::East),
                (None, _) => break,
            }
        }
        for _ in 0..dy {
            dest = match (dest, y.is_negative()) {
                (Some(s), true) => s.checked_add(Dir::North),
                (Some(s), false) => s.checked_add(Dir::South),
                (None, _) => break,
            }
        }
        match (held_piece(), dest) {
            (Some(o), Some(d)) => set_board.update(|b| {
                let r = b.make(Move {
                    origin: o,
                    dest: d,
                    promotion: Piece::Empty,
                });
                if let Err(e) = r {
                    log!("{}", e);
                }
            }),
            _ => log!("could not make move."),
        }
        set_held_piece(None);
        set_held_origin((0, 0));
    };
    let view = squares
        .map(|(sqr, p, style)| {
            view! {
                cx,
                <div class="square">
                    <PieceDisplay
                        held=(move || held_piece() == Some(sqr())).derive_signal(cx)
                        piece=p
                        style=style
                        on:mousedown=move |e| {
                            let color_to_move = board.with(|b| b.color_to_move());
                            if p().color() == Some(color_to_move){
                                let target: HtmlDivElement = event_target(&e);
                                set_held_piece(Some(sqr()));
                                set_held_origin((target.offset_left() + target.offset_width() / 2 , target.offset_top() + target.offset_height() / 2))
                            } 
                        }
                        on:mouseup=mouse_up
                    />
                </div>
            }
        })
        .collect::<Vec<_>>();
    view! {
        cx,
        <div class="chess-board">
            {view}
        </div>
    }
}

#[component]
pub fn PieceDisplay(
    cx: Scope,
    piece: Signal<Piece>,
    style: Signal<String>,
    held: Signal<bool>,
) -> impl IntoView {
    let class = move || {
        match piece() {
            Piece::Filled(kind, color) => format!("{}{}", color, kind),
            Piece::Empty => "".to_owned(),
        }
    };

    view! {
        cx,
        <div
            class=move || format!("piece kind-{}", class())
            class:held=held
            prop:style=style
        />
    }
}
