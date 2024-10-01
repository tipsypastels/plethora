use super::{ServeError, ServeResult};
use anyhow::{Error, Result};
use axum::response::Response;
use liquid::Object;

pub trait Render: Clone + Send + Sync + 'static {
    fn render(&self, template: &str, props: Object) -> ServeResult {
        match self.try_render(template, props) {
            Ok(response) => Ok(response),
            Err(error) => Err(ServeError::new(self.clone(), error)),
        }
    }

    fn try_render(&self, template: &str, props: Object) -> Result<Response>;
    fn try_render_error(&self, error: &Error) -> Result<Response>;
    fn try_render_not_found(&self) -> Result<Response>;
}
