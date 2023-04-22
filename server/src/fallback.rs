use axum::response::Response as AxumResponse;
use axum::{
    body::{boxed, Body, BoxBody},
    extract::Extension,
    http::{Request, Response, StatusCode, Uri},
    response::IntoResponse,
};
use leptos::LeptosOptions;
use std::sync::Arc;
use tower::ServiceExt;
use tower_http::services::ServeDir;

pub async fn file_handler(
    uri: Uri,
    Extension(options): Extension<Arc<LeptosOptions>>,
) -> AxumResponse {
    let options = &*options;
    let root = options.site_root.clone();
    let res = get_static_file(uri.clone(), &root).await.unwrap();

    res.into_response()
}

async fn get_static_file(uri: Uri, root: &str) -> Result<Response<BoxBody>, (StatusCode, String)> {
    let req = Request::builder()
        .uri(uri.clone())
        .body(Body::empty())
        .unwrap();
    // `ServeDir` implements `tower::Service` so we can call it with `tower::ServiceExt::oneshot`
    // This path is relative to the cargo root
    match ServeDir::new(root).oneshot(req).await {
        Ok(res) => Ok(res.map(boxed)),
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", err),
        )),
    }
}
