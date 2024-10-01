use super::Renderer;
use anyhow::Error;
use axum::response::{IntoResponse, Response};

mod fallback;
mod traits;

pub type ServeResult<T = Response> = Result<T, ServeError>;
pub use traits::*;

#[derive(Debug)]
pub struct ServeError {
    response: Response,
}

impl ServeError {
    pub fn new(re: impl Renderer, error: Error) -> Self {
        dbg!(&error);
        let response = match dbg!(re.try_render_error(&error)) {
            Ok(response) => response,
            Err(new_error) => fallback::render(error, new_error),
        };

        Self { response }
    }

    pub fn not_found(re: impl Renderer) -> Self {
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
