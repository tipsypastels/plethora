use super::{Application, CurrentHooks};
use crate::db::Id;
use serde::Serialize;
use std::{ops::Deref, sync::Arc};

#[derive(Debug)]
pub struct CurrentUserState<C: CurrentHooks>(Option<Arc<C::User>>);

impl<C: CurrentHooks> CurrentUserState<C> {
    pub(super) async fn new(app: &impl Application, id: Option<Id>) -> Self {
        let Some(id) = id else {
            return Self(None);
        };

        match C::user(app.db(), id).await {
            Ok(user) => Self(user.map(Arc::new)),
            Err(error) => {
                tracing::error!("error resolving current user: {error}");
                Self(None)
            }
        }
    }

    pub fn empty() -> Self {
        Self(None)
    }

    pub fn get(&self) -> Option<CurrentUser<C>> {
        self.0.as_ref().cloned().map(CurrentUser)
    }
}

impl<C: CurrentHooks> Clone for CurrentUserState<C> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[derive(Debug, Serialize)]
pub struct CurrentUser<C: CurrentHooks>(Arc<C::User>);

impl<C: CurrentHooks> Clone for CurrentUser<C> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<C: CurrentHooks> Deref for CurrentUser<C> {
    type Target = C::User;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
