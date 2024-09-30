use super::{App, Hooks};
use crate::{scratch, stuff::STUFF};
use axum::Router;
use tower_http::services::ServeDir;

pub fn public_router<H: Hooks>(app: App<H>) -> Router<()> {
    let public_dir = ServeDir::new(STUFF.public.dir.as_ref());
    let scratch_dir = ServeDir::new(scratch::public_dir());
    let service = public_dir.fallback(scratch_dir);

    Router::new()
        .fallback_service(service)
        // TODO
        // .route("/__reload__", get(reload::sse))
        .with_state(app)
}

// pub fn common_router<H, F>(app: App<H>, f: F) -> Router<()>
// where
//     H: Hooks,
//     F: FnOnce(App<H>, Router<H>) -> Router<H>,
// {
// }
