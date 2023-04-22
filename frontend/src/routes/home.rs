use leptos::*;
use leptos_meta::{Title, TitleProps};
use leptos_router::{AProps, A};

#[component]
pub fn Home(cx: Scope) -> impl IntoView {
    let (id, set_id) = create_signal(cx, "".to_string());
    view! {
        cx,
        <>
            <Title text="Home"/>
            <div class="content">
                <input type="text" prop:value=id on:input=move |e| set_id(event_target_value(&e))/>
                <A href=move || format!("/play/{}", id())>
                    <button >"Go"</button>
                </A>
                <h1>"Hello"</h1>
            </div>
        </>
    }
}
