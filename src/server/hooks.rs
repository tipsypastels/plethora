use super::Server;
use crate::db::{Db, Id};
use anyhow::Result;
use axum::{
    extract::{FromRequestParts, Request},
    http::request::Parts,
    response::Redirect,
};
use std::convert::Infallible;

pub trait Hooks: Clone + Send + Sync + 'static {
    type Session: HooksSession;
    type User: HooksUser;

    fn base_page_title(&self) -> Option<&str> {
        None
    }

    fn force_redirect(&self, request: Request) -> Option<Redirect> {
        let _ = request;
        None
    }

    fn default_theme_slug(&self) -> Option<&str> {
        None
    }

    async fn get_current_session(&self, db: &Db, id: Id) -> Result<Option<Self::Session>> {
        let _ = (db, id);
        Ok(None)
    }

    async fn get_current_user(&self, db: &Db, id: Id) -> Result<Option<Self::User>> {
        let _ = (db, id);
        Ok(None)
    }
}

impl Hooks for () {
    type Session = Infallible;
    type User = Infallible;
}

pub trait HooksSession: Clone + Send + Sync + 'static {
    fn hooks_session_user_id(&self) -> Id;
}

impl HooksSession for Infallible {
    fn hooks_session_user_id(&self) -> Id {
        match *self {}
    }
}

pub trait HooksUser: Clone + Send + Sync + 'static {}

impl HooksUser for Infallible {}

#[derive(Debug, Clone)]
pub struct ServerHooks<H>(pub H);

#[axum::async_trait]
impl<H: Hooks> FromRequestParts<Server<H>> for ServerHooks<H> {
    type Rejection = Infallible;

    async fn from_request_parts(_: &mut Parts, server: &Server<H>) -> Result<Self, Infallible> {
        Ok(ServerHooks(server.hooks.clone()))
    }
}
