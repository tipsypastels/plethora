use super::AsApp;
use crate::db::{Db, Id};
use anyhow::Result;
use axum::{
    extract::{FromRequestParts, Request},
    http::{request::Parts, Extensions},
    middleware::Next,
    response::Response,
};
use serde::Serialize;
use std::{convert::Infallible, fmt};
use tower_cookies::Cookies;

mod language;
mod session;
mod theme;
mod user;

pub use language::{CurrentLanguage, CurrentLanguageState};
pub use session::{CurrentSession, CurrentSessionState};
pub use user::{CurrentUser, CurrentUserState};

pub trait Current: fmt::Debug + Clone + Send + Sync + 'static {
    type Session: fmt::Debug + Serialize + Send + Sync + 'static;
    type User: fmt::Debug + Serialize + Send + Sync + 'static;

    async fn session(db: &Db, id: Id) -> Result<Option<Self::Session>>;
    async fn user(db: &Db, id: Id) -> Result<Option<Self::User>>;

    fn user_id(session: &Self::Session) -> Id;
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct CurrentState<C: Current> {
    pub language: CurrentLanguageState<C>,
    pub session: CurrentSessionState<C>,
    pub user: CurrentUserState<C>,
}

impl<C: Current> CurrentState<C> {
    pub fn get(request: &Request) -> Self {
        Self::extension(request.extensions())
    }

    pub fn extension(extensions: &Extensions) -> Self {
        extensions
            .get::<Self>()
            .cloned()
            .expect("no current extension")
    }
}

#[axum::async_trait]
impl<S, C: Current> FromRequestParts<S> for CurrentState<C> {
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Infallible> {
        Ok(Self::extension(&parts.extensions))
    }
}

macro_rules! current_accessors {
    (<$type_var:ident> $($field:ident: $ty:ty),*$(,)?) => {
        $(
            impl<$type_var: Current> $ty {
                pub fn extension(extensions: &Extensions) -> Self {
                    CurrentState::<$type_var>::extension(extensions).$field
                }
            }

            #[axum::async_trait]
            impl<S, $type_var: Current> FromRequestParts<S> for $ty {
                type Rejection = Infallible;

                async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<$ty, Infallible> {
                    Ok(CurrentState::<$type_var>::extension(&parts.extensions).$field)
                }
            }
        )*
    };
}

current_accessors! {
    <C>
    language: CurrentLanguageState<C>,
    session: CurrentSessionState<C>,
    user: CurrentUserState<C>,
}

pub async fn layer<C: Current>(
    app: impl AsApp,
    cookies: Cookies,
    mut request: Request,
    next: Next,
) -> Result<Response, Infallible> {
    // let theme = CurrentThemeState::new(&app, &request, &cookies);
    let language = CurrentLanguageState::new();
    let session = CurrentSessionState::new(&app, &cookies).await;
    let user = CurrentUserState::new(&app, session.user_id()).await;
    let current = CurrentState::<C> {
        // theme,
        language,
        session,
        user,
    };

    request.extensions_mut().insert(current);
    Ok(next.run(request).await)
}
