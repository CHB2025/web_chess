use leptos::{component, create_signal, provide_context, view, IntoView, Scope};
use leptos_meta::*;
use leptos_router::*;
use routes::play::*;
use web_sys::MouseEvent;
use cfg_if::cfg_if;

mod board_provider;
mod chess_board;
mod routes;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    provide_meta_context(cx);
    let (mouse, set_mouse) = create_signal(cx, (0, 0));
    provide_context(cx, mouse);
    let mouse_move = move |e: MouseEvent| set_mouse((e.client_x(), e.client_y()));
    let formatter = |text| format!("{text} - Chess!");

    view! {
        cx,
        <>
            <Stylesheet href="/pkg/web_chess.css"/>
            <Meta name="description" content="Leptos chess website"/>
            <Title text="Chess!" formatter />

            <Router>
                <main on:mousemove=mouse_move>
                    <Routes>
                        <Route path="/" view=move |cx| view! {cx, <div>"Hello world"</div>}/>
                        <Route path="play" view=move |cx| view! {cx, <Play/>}/>
                        <Route path="play/:id" view=move |cx| view! {cx,  <Play/>}/>
                    </Routes>
                </main>
            </Router>
        </>
    }
}

cfg_if! {
    if #[cfg(feature = "hydrate")] {
        use wasm_bindgen::prelude::wasm_bindgen;

        #[wasm_bindgen]
        pub fn hydrate() {
            leptos::mount_to_body(move |cx| {
                view! {cx, <App/>}
            });
        }
    }
}
