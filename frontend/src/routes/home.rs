use chb_chess::BoardBuilder;
use gloo_net::http::Request;
use leptos::*;
use leptos_meta::{Title, TitleProps};
use leptos_router::{AProps, A};

#[component]
pub fn Home(cx: Scope) -> impl IntoView {
    //let (id, set_id) = create_signal(cx, "".to_string());
    let id = create_local_resource(
        cx,
        || "",
        move |_| async {
            let req = Request::post("/api/board/create")
                .json(&BoardBuilder::default())
                .unwrap();
            req.send().await.unwrap().text().await.unwrap()
        },
    );
    view! {
        cx,
        <>
            <Title text="Home"/>
            <div class="content">
                <Suspense fallback=move || view! { cx, <p>"Loading..."</p>}>
                    <A href=move || format!("/play/{}", id.read(cx).unwrap_or("".to_owned()))>
                        <button >"Play"</button>
                    </A>
                    <br/>
                    <h1>"Hello"</h1>
                </Suspense>
            </div>
        </>
    }
}
