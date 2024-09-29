use super::Server;
use axum::{extract::FromRequestParts, http::request::Parts};
use std::convert::Infallible;

pub trait Extra: Clone + Send + Sync + 'static {
    fn base_page_title(&self) -> Option<&str> {
        None
    }
}

impl Extra for () {}

#[derive(Debug, Clone)]
pub struct ServerExtra<E>(pub E);

#[axum::async_trait]
impl<E: Extra> FromRequestParts<Server<E>> for ServerExtra<E> {
    type Rejection = Infallible;

    async fn from_request_parts(_: &mut Parts, server: &Server<E>) -> Result<Self, Infallible> {
        Ok(ServerExtra(server.extra.clone()))
    }
}
