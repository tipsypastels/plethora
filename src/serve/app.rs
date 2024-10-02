use crate::{db::Db, reload::Reloader, scripts::Scripts, styles::Styles, themes::Themes};
use axum::{extract::FromRequestParts, http::request::Parts};
use std::convert::Infallible;

pub trait Application: Clone + Send + Sync + 'static {
    fn db(&self) -> &Db;
    fn styles(&self) -> &Styles;
    fn themes(&self) -> &Themes;
    fn scripts(&self) -> &Scripts;
    fn reloader(&self) -> &Reloader;

    fn default_theme_slug(&self) -> &str;

    fn base_page_title(&self) -> Option<&str> {
        None
    }
}

macro_rules! app_accessors {
    ($($field:ident: $ty:ty),*$(,)?) => {
        $(
            #[axum::async_trait]
            impl<A: Application> FromRequestParts<A> for $ty {
                type Rejection = Infallible;

                async fn from_request_parts(_: &mut Parts, app: &A) -> Result<$ty, Infallible> {
                    Ok(app.$field().clone())
                }
            }
        )*
    };
}

app_accessors! {
    db: Db,
    styles: Styles,
    themes: Themes,
    reloader: Reloader,
}
