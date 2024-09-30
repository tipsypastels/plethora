use super::{Hooks, Renderer};
use anyhow::Error;
use axum::response::{IntoResponse, Response};

mod fallback;
mod traits;

pub type ServeResult<T = Response> = Result<T, ServeError>;
pub use traits::*;

pub struct ServeError {
    response: Response,
}

impl ServeError {
    pub fn new<H: Hooks>(re: Renderer<H>, error: Error) -> Self {
        let response = match re.try_render_error(&error) {
            Ok(response) => response,
            Err(new_error) => fallback::render(error, new_error),
        };

        Self { response }
    }

    pub fn not_found<H: Hooks>(re: Renderer<H>) -> Self {
        match re.try_render_not_found() {
            Ok(response) => Self { response },
            Err(error) => Self::new(re, error),
        }
    }
}

impl IntoResponse for ServeError {
    fn into_response(self) -> Response {
        self.response
    }
}
