use crate::{
    db::Id,
    server::{hooks::HooksSession, Hooks, Server},
};
use std::{ops::Deref, sync::Arc};
use tower_cookies::Cookies;

const COOKIE: &str = "plethora-session";

pub struct CurrentSessionState<H: Hooks>(Option<Arc<H::Session>>);

impl<H: Hooks> CurrentSessionState<H> {
    pub(super) async fn new(server: &Server<H>, cookies: &Cookies) -> Self {
        let Some(cookie) = cookies.get(COOKIE) else {
            return Self(None);
        };

        let Ok(session_id) = Id::parse_str(cookie.value()) else {
            tracing::debug!(value = %cookie.value(), "malformed session cookie");
            return Self(None);
        };

        match server
            .hooks
            .get_current_session(&server.db, session_id)
            .await
        {
            Ok(session) => Self(session.map(Arc::new)),
            Err(error) => {
                tracing::error!("error resolving current session: {error}");
                Self(None)
            }
        }
    }

    pub fn empty() -> Self {
        Self(None)
    }

    pub fn get(&self) -> Option<CurrentSession<H>> {
        self.0.as_ref().cloned().map(CurrentSession)
    }

    pub fn user_id(&self) -> Option<Id> {
        self.get().map(|s| s.hooks_session_user_id())
    }
}

pub struct CurrentSession<H: Hooks>(Arc<H::Session>);

impl<H: Hooks> Deref for CurrentSession<H> {
    type Target = H::Session;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
