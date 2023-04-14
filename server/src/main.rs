use std::sync::Arc;

use axum::{Extension, Router};
use frontend::{App, AppProps};
use leptos::{get_configuration, view};
use leptos_axum::{generate_route_list, LeptosRoutes};

use crate::fallback::file_handler;

mod fallback;

#[tokio::main]
async fn main() {
    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(|cx| view! {cx, <App/> }).await;

    let app = Router::new()
        .leptos_routes(leptos_options.clone(), routes, |cx| view! {cx, <App/>})
        .fallback(file_handler)
        .layer(Extension(Arc::new(leptos_options)));

    println!("Server Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
