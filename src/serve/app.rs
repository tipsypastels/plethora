use super::Hooks;
use crate::{db::Db, styles::Styles, themes::Themes};
use anyhow::Result;
use axum::{extract::FromRequestParts, http::request::Parts};
use std::convert::Infallible;

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct App<H> {
    pub db: Db,
    pub styles: Styles,
    pub themes: Themes,
    pub hooks: H,
}

#[derive(Debug)]
pub struct AppInit<H> {
    pub db: Db,
    pub hooks: H,
}

impl<H> App<H>
where
    H: Hooks,
{
    pub async fn new(init: AppInit<H>) -> Result<Self> {
        let AppInit { db, hooks } = init;

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
impl<H: Hooks> FromRequestParts<App<H>> for App<H> {
    type Rejection = Infallible;

    async fn from_request_parts(_parts: &mut Parts, app: &Self) -> Result<Self, Infallible> {
        Ok(app.clone())
    }
}

app_accessors! {
    db: Db,
    styles: Styles,
    themes: Themes,
}

macro_rules! app_accessors {
    ($($field:ident: $ty:ty),*$(,)?) => {
        $(
            #[axum::async_trait]
            impl<H: Hooks> FromRequestParts<App<H>> for $ty {
                type Rejection = Infallible;

                async fn from_request_parts(_: &mut Parts, app: &App<H>) -> Result<$ty, Infallible> {
                    Ok(app.$field.clone())
                }
            }
        )*
    };
}

use app_accessors;
