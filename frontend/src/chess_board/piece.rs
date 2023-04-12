use chb_chess::{Dir, Move, Piece, Square};
use leptos::{
    component, create_signal, event_target, log, use_context, view, IntoSignal, IntoView,
    ReadSignal, Scope, Signal, SignalWith,
};
use web_sys::{HtmlDivElement, MouseEvent};

#[component]
pub fn PieceDisplay<F>(
    cx: Scope,
    square: Square,
    piece: Signal<Piece>,
    #[prop(into)] board_reversed: Signal<bool>,
    may_move: Signal<bool>,
    make_move: F,
) -> impl IntoView
where
    F: Fn(Move) + 'static,
{
    let (held, set_held) = create_signal(cx, false);
    let mouse = use_context::<ReadSignal<(i32, i32)>>(cx).expect("Mouse signal to exist");
    let (origin, set_origin) = create_signal(cx, (0, 0));
    let translate = (move || {
        let (m_x, m_y) = mouse();
        let (o_x, o_y) = origin();
        (m_x - o_x, m_y - o_y)
    })
    .derive_signal(cx);
    let class = move || match piece() {
        Piece::Filled(kind, color) => format!("{}{}", color, kind),
        Piece::Empty => "empty".to_owned(),
    };
    let style = move || {
        if held() {
            translate.with(|(x, y)| format!("transform: translate({x}px, {y}px)"))
        } else {
            "".to_string()
        }
    };

    let mouse_up = move |e: MouseEvent| {
        if !held() {
            return;
        }
        let target: HtmlDivElement = event_target(&e);
        let (x, y) = translate();
        let dx = (x.abs() + target.offset_width() / 2) / target.offset_width();
        let dy = (y.abs() + target.offset_height() / 2) / target.offset_height();
        let mut dest = Some(square);
        for _ in 0..dx {
            dest = match (dest, x.is_negative() ^ board_reversed()) {
                (Some(s), true) => s.checked_add(Dir::West),
                (Some(s), false) => s.checked_add(Dir::East),
                (None, _) => break,
            }
        }
        for _ in 0..dy {
            dest = match (dest, y.is_negative() ^ board_reversed()) {
                (Some(s), true) => s.checked_add(Dir::North),
                (Some(s), false) => s.checked_add(Dir::South),
                (None, _) => break,
            }
        }
        log!("Moving to {:?}", dest);
        match dest {
            Some(d) => make_move(Move {
                origin: square,
                dest: d,
                promotion: Piece::Empty,
            }),
            _ => log!("Invalid move targets"),
        }
        set_held(false);
    };

    let mouse_down = move |e: MouseEvent| {
        if may_move() {
            let target: HtmlDivElement = event_target(&e);
            set_held(true);
            set_origin((
                target.offset_left() + target.offset_width() / 2,
                target.offset_top() + target.offset_height() / 2,
            ));
        }
    };

    view! {
        cx,
        <div
            class=move || format!("piece kind-{}", class())
            class:held=held
            prop:style=style
            on:mousedown=mouse_down
            on:mouseup=mouse_up
        />
    }
}
