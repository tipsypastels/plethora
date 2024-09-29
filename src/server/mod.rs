use crate::{db::Db, styles::Styles, themes::Themes};
use anyhow::Result;
use axum::{extract::FromRequestParts, http::request::Parts};
use std::convert::Infallible;

mod extra;

pub use extra::{Extra, ServerExtra};

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Server<E> {
    pub db: Db,
    pub styles: Styles,
    pub themes: Themes,
    pub extra: E,
}

#[derive(Debug)]
pub struct ServerInit<E> {
    pub db: Db,
    pub extra: E,
}

impl<E> Server<E> {
    pub async fn new(init: ServerInit<E>) -> Result<Self> {
        let ServerInit { db, extra } = init;

        let styles = Styles::new().await?;
        let themes = Themes::new(styles.clone()).await?;

        Ok(Self {
            db,
            styles,
            themes,
            extra,
        })
    }
}

#[axum::async_trait]
impl<E: Extra> FromRequestParts<Server<E>> for Server<E> {
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
            impl<E: Extra> FromRequestParts<Server<E>> for $ty {
                type Rejection = Infallible;

                async fn from_request_parts(_: &mut Parts, app: &Server<E>) -> Result<$ty, Infallible> {
                    Ok(app.$field.clone())
                }
            }
        )*
    };
}

use server_accessors;
