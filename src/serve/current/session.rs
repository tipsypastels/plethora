use crate::{
    db::Id,
    serve::{App, Hooks, SessionHooks},
};
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, Extensions},
};
use serde::Serialize;
use std::{convert::Infallible, ops::Deref, sync::Arc};
use tower_cookies::Cookies;

const COOKIE: &str = "plethora-session";

#[derive(Debug, Clone)]
pub struct CurrentSessionState<H: Hooks>(Option<Arc<H::Session>>);

impl<H: Hooks> CurrentSessionState<H> {
    pub(super) async fn new(app: &App<H>, cookies: &Cookies) -> Self {
        let Some(cookie) = cookies.get(COOKIE) else {
            return Self(None);
        };

        let Ok(session_id) = Id::parse_str(cookie.value()) else {
            tracing::debug!(value = %cookie.value(), "malformed session cookie");
            return Self(None);
        };

        match app.hooks.get_current_session(&app.db, session_id).await {
            Ok(session) => Self(session.map(Arc::new)),
            Err(error) => {
                tracing::error!("error resolving current session: {error}");
                Self(None)
            }
        }
    }

    pub fn extension(extensions: &Extensions) -> Self {
        super::CurrentState::extension(extensions).session
    }

    pub fn empty() -> Self {
        Self(None)
    }

    pub fn get(&self) -> Option<CurrentSession<H>> {
        self.0.as_ref().cloned().map(CurrentSession)
    }

    pub fn user_id(&self) -> Option<Id> {
        self.get().map(|s| s.session_user_id())
    }
}

#[axum::async_trait]
impl<H: Hooks> FromRequestParts<App<H>> for CurrentSessionState<H> {
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _: &App<H>) -> Result<Self, Infallible> {
        Ok(Self::extension(&parts.extensions))
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CurrentSession<H: Hooks>(Arc<H::Session>);

impl<H: Hooks> Deref for CurrentSession<H> {
    type Target = H::Session;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
