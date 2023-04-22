use std::{collections::HashMap, sync::Arc};

use axum::routing::post;
use axum::{routing::get, Extension, Router};
use board_state::BoardState;
use frontend::{App, AppProps};
use leptos::{get_configuration, log, view};
use leptos_axum::{generate_route_list, LeptosRoutes};
use tokio::sync::RwLock;

use crate::routes::board::{create_board, get_board};
use crate::{fallback::file_handler, routes::board::subscribe_to_board};

mod board_state;
mod code_gen;
mod fallback;
mod routes;

type BoardList = Arc<RwLock<HashMap<String, Arc<BoardState>>>>;

#[tokio::main]
async fn main() {
    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(|cx| view! {cx, <App/> }).await;

    simple_logger::init_with_level(log::Level::Debug).expect("couldn't initialize logging");

    let bs_map: BoardList = Arc::new(RwLock::new(HashMap::new()));
    let state = bs_map;

    let api = Router::new()
        .route("/board/:id", get(get_board))
        .route("/board/create", post(create_board))
        .route("/board/:id/subscribe", get(subscribe_to_board))
        .with_state(state);

    let app = Router::new()
        .nest("/api", api)
        .leptos_routes(leptos_options.clone(), routes, |cx| view! {cx, <App/>})
        .fallback(file_handler)
        .layer(Extension(Arc::new(leptos_options)));

    log!("Server Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
