use crate::{db::Db, styles::Styles, themes::Themes};
use anyhow::Result;
use axum::{extract::FromRequestParts, http::request::Parts};
use std::convert::Infallible;

mod current;
mod error;
mod hooks;
mod render;

pub use current::{
    layer, CurrentSession, CurrentSessionState, CurrentState, CurrentThemeState, CurrentUser,
    CurrentUserState,
};
pub use error::{OrNotFound, Re, ServeError, ServeResult};
pub use hooks::{Hooks, ServerHooks, SessionHooks, UserHooks};
pub use render::Renderer;

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Server<H> {
    pub db: Db,
    pub styles: Styles,
    pub themes: Themes,
    pub hooks: H,
}

#[derive(Debug)]
pub struct ServerInit<H> {
    pub db: Db,
    pub hooks: H,
}

impl<H> Server<H> {
    pub async fn new(init: ServerInit<H>) -> Result<Self> {
        let ServerInit { db, hooks } = init;

        let styles = Styles::new().await?;
        let themes = Themes::new(styles.clone()).await?;

        Ok(Self {
            db,
            styles,
            themes,
            hooks,
        })
    }
}

#[axum::async_trait]
impl<H: Hooks> FromRequestParts<Server<H>> for Server<H> {
    type Rejection = Infallible;

    async fn from_request_parts(_parts: &mut Parts, server: &Self) -> Result<Self, Infallible> {
        Ok(server.clone())
    }
}

server_accessors! {
    db: Db,
    styles: Styles,
    themes: Themes,
}

macro_rules! server_accessors {
    ($($field:ident: $ty:ty),*$(,)?) => {
        $(
            #[axum::async_trait]
            impl<H: Hooks> FromRequestParts<Server<H>> for $ty {
                type Rejection = Infallible;

                async fn from_request_parts(_: &mut Parts, app: &Server<H>) -> Result<$ty, Infallible> {
                    Ok(app.$field.clone())
                }
            }
        )*
    };
}

use server_accessors;
