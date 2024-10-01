use super::Current;
use crate::{db::Id, serve::AsApp};
use serde::Serialize;
use std::{ops::Deref, sync::Arc};
use tower_cookies::Cookies;

const COOKIE: &str = "plethora-session";

#[derive(Debug)]
pub struct CurrentSessionState<C: Current>(Option<Arc<C::Session>>);

impl<C: Current> CurrentSessionState<C> {
    pub(super) async fn new(app: &impl AsApp, cookies: &Cookies) -> Self {
        let Some(cookie) = cookies.get(COOKIE) else {
            return Self(None);
        };

        let Ok(session_id) = Id::parse_str(cookie.value()) else {
            tracing::debug!(value = %cookie.value(), "malformed session cookie");
            return Self(None);
        };

        match C::session(app.as_db(), session_id).await {
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

    pub fn get(&self) -> Option<CurrentSession<C>> {
        self.0.as_ref().cloned().map(CurrentSession)
    }

    pub fn user_id(&self) -> Option<Id> {
        self.get().map(|s| C::user_id(&s))
    }
}

impl<C: Current> Clone for CurrentSessionState<C> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[derive(Debug, Serialize)]
pub struct CurrentSession<C: Current>(Arc<C::Session>);

impl<C: Current> Clone for CurrentSession<C> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<C: Current> Deref for CurrentSession<C> {
    type Target = C::Session;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
