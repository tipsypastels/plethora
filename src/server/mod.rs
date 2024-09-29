use crate::{db::Db, styles::Styles, themes::Themes};
use anyhow::Result;
use axum::{extract::FromRequestParts, http::request::Parts};
use std::convert::Infallible;

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Server<Extra> {
    pub db: Db,
    pub styles: Styles,
    pub themes: Themes,
    pub extra: Extra,
}

#[derive(Debug)]
pub struct ServerInit<Extra> {
    pub db: Db,
    pub extra: Extra,
}

impl<Extra> Server<Extra> {
    pub async fn new(init: ServerInit<Extra>) -> Result<Self> {
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
impl<Extra> FromRequestParts<Server<Extra>> for Server<Extra>
where
    Extra: Clone + Send + Sync + 'static,
{
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
            impl<Extra> FromRequestParts<Server<Extra>> for $ty
            where
                Extra: Clone + Send + Sync + 'static
             {
                type Rejection = Infallible;

                async fn from_request_parts(_: &mut Parts, app: &Server<Extra>) -> Result<$ty, Infallible> {
                    Ok(app.$field.clone())
                }
            }
        )*
    };
}

use server_accessors;

#[derive(Debug, Clone)]
pub struct ServerExtra<Extra>(pub Extra);

#[axum::async_trait]
impl<Extra> FromRequestParts<Server<Extra>> for ServerExtra<Extra>
where
    Extra: Clone + Send + Sync + 'static,
{
    type Rejection = Infallible;

    async fn from_request_parts(
        _parts: &mut Parts,
        server: &Server<Extra>,
    ) -> Result<Self, Infallible> {
        Ok(ServerExtra(server.extra.clone()))
    }
}
