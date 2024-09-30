use super::{Hooks, Server};
use axum::{
    extract::{FromRequestParts, Request},
    http::{request::Parts, Extensions},
    middleware::Next,
    response::Response,
};
use std::convert::Infallible;
use tower_cookies::Cookies;

mod language;
mod session;
mod theme;
mod user;

pub use session::{CurrentSession, CurrentSessionState};
pub use theme::CurrentThemeState;
pub use user::{CurrentUser, CurrentUserState};

#[derive(Debug, Clone)]
pub struct CurrentState<H: Hooks> {
    pub theme: CurrentThemeState,
    pub session: CurrentSessionState<H>,
    pub user: CurrentUserState<H>,
}

impl<H: Hooks> CurrentState<H> {
    pub fn empty_with_fixed_theme(slug: &str) -> Self {
        Self {
            theme: CurrentThemeState::with_fixed_theme(slug),
            session: CurrentSessionState::empty(),
            user: CurrentUserState::empty(),
        }
    }

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
impl<H: Hooks> FromRequestParts<Server<H>> for CurrentState<H> {
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _: &Server<H>) -> Result<Self, Infallible> {
        Ok(Self::extension(&parts.extensions))
    }
}

pub async fn layer<H: Hooks>(
    server: Server<H>,
    cookies: Cookies,
    mut request: Request,
    next: Next,
) -> Result<Response, Infallible> {
    let theme = CurrentThemeState::new(&server, &request, &cookies);
    let session = CurrentSessionState::new(&server, &cookies).await;
    let user = CurrentUserState::new(&server, session.user_id()).await;
    let current = CurrentState {
        theme,
        session,
        user,
    };

    request.extensions_mut().insert(current);
    Ok(next.run(request).await)
}
