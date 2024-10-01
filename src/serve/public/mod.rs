use super::Application;
use crate::{scratch, stuff::STUFF};
use axum::{routing::get, Router};
use tower_http::services::ServeDir;

mod reload;

pub fn router<A: Application>(app: A) -> Router {
    let public_dir = ServeDir::new(STUFF.public.dir.as_ref());
    let scratch_dir = ServeDir::new(scratch::public_dir());
    let service = public_dir.fallback(scratch_dir);

    Router::new()
        .fallback_service(service)
        .route("/__reload__", get(reload::js))
        .route("/__reload_sse__", get(reload::sse))
        .with_state(app)
}
