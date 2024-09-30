use crate::{
    db::Id,
    server::{Hooks, Server},
};
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, Extensions},
};
use serde::Serialize;
use std::{convert::Infallible, ops::Deref, sync::Arc};

#[derive(Debug, Clone)]
pub struct CurrentUserState<H: Hooks>(Option<Arc<H::User>>);

impl<H: Hooks> CurrentUserState<H> {
    pub(super) async fn new(server: &Server<H>, id: Option<Id>) -> Self {
        let Some(id) = id else {
            return Self(None);
        };

        match server.hooks.get_current_user(&server.db, id).await {
            Ok(user) => Self(user.map(Arc::new)),
            Err(error) => {
                tracing::error!("error resolving current user: {error}");
                Self(None)
            }
        }
    }

    pub fn extension(extensions: &Extensions) -> Self {
        super::CurrentState::extension(extensions).user
    }

    pub fn empty() -> Self {
        Self(None)
    }

    pub fn get(&self) -> Option<CurrentUser<H>> {
        self.0.as_ref().cloned().map(CurrentUser)
    }
}

#[axum::async_trait]
impl<H: Hooks> FromRequestParts<Server<H>> for CurrentUserState<H> {
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _: &Server<H>) -> Result<Self, Infallible> {
        Ok(Self::extension(&parts.extensions))
    }
}

#[derive(Debug, Clone)]
pub struct CurrentUser<H: Hooks>(Arc<H::User>);

impl<H: Hooks> Serialize for CurrentUser<H>
where
    H::User: Serialize,
{
    fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(ser)
    }
}

impl<H: Hooks> Deref for CurrentUser<H> {
    type Target = H::User;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
