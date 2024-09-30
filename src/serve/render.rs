use super::{App, CurrentState, Hooks, ServeError, ServeResult};
use crate::themes::ThemeGuard;
use anyhow::{Error, Result};
use axum::{
    extract::FromRequestParts,
    http::request::Parts,
    response::{Html, IntoResponse, Response},
};
use liquid::Object;
use reqwest::StatusCode;
use std::convert::Infallible;

#[derive(Debug, Clone)]
pub struct Renderer<H: Hooks> {
    app: App<H>,
    current: CurrentState<H>,
}

impl<H: Hooks> Renderer<H> {
    pub fn render(&self, template: &str, props: Object) -> ServeResult {
        match self.try_render(template, props) {
            Ok(response) => Ok(response),
            Err(error) => Err(ServeError::new(self.clone(), error)),
        }
    }

    pub fn try_render(&self, template: &str, props: Object) -> Result<Response> {
        let theme = self.theme()?;
        let base_title = self.app.hooks.base_page_title();
        let html = theme.render(template, base_title, props, &self.current)?;

        Ok(Html(html).into_response())
    }

    pub fn try_render_error(&self, error: &Error) -> Result<Response> {
        let theme = self.theme()?;
        let base_title = self.app.hooks.base_page_title();
        let html = theme.render_error(error, base_title, &self.current)?;

        Ok((StatusCode::INTERNAL_SERVER_ERROR, Html(html)).into_response())
    }

    pub fn try_render_not_found(&self) -> Result<Response> {
        let theme = self.theme()?;
        let base_title = self.app.hooks.base_page_title();
        let html = theme.render_not_found(base_title, &self.current)?;

        Ok((StatusCode::NOT_FOUND, Html(html)).into_response())
    }

    fn theme(&self) -> Result<ThemeGuard> {
        self.current.theme.resolve(&self.app.themes)
    }
}

#[axum::async_trait]
impl<H: Hooks> FromRequestParts<App<H>> for Renderer<H> {
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, app: &App<H>) -> Result<Self, Infallible> {
        let current = CurrentState::from_request_parts(parts, app).await?;
        let app = app.clone();

        Ok(Self { app, current })
    }
}
