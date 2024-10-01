use super::{AsApp, Current, CurrentState, ServeError, ServeResult};
use crate::themes::ThemeGuard;
use anyhow::{Error, Result};
use axum::response::{Html, IntoResponse, Response};
use liquid::Object;
use reqwest::StatusCode;

pub trait Render: Clone + Send + Sync + 'static {
    type App: AsApp;
    type Current: Current;

    fn render(&self, template: &str, props: Object) -> ServeResult {
        match self.try_render(template, props) {
            Ok(response) => Ok(response),
            Err(error) => Err(ServeError::new(self.clone(), error)),
        }
    }

    fn try_render(&self, template: &str, props: Object) -> Result<Response> {
        let theme = self.theme()?;
        let base_title = self.app().base_page_title();
        let html = theme.render(template, base_title, props, self.current())?;

        Ok(Html(html).into_response())
    }

    fn try_render_error(&self, error: &Error) -> Result<Response> {
        let theme = self.theme()?;
        let base_title = self.app().base_page_title();
        let html = theme.render_error(error, base_title, self.current())?;

        Ok((StatusCode::INTERNAL_SERVER_ERROR, Html(html)).into_response())
    }

    fn try_render_not_found(&self) -> Result<Response> {
        let theme = self.theme()?;
        let base_title = self.app().base_page_title();
        let html = theme.render_not_found(base_title, self.current())?;

        Ok((StatusCode::NOT_FOUND, Html(html)).into_response())
    }

    fn theme(&self) -> Result<ThemeGuard> {
        self.current().theme.resolve(self.app().as_themes())
    }

    fn app(&self) -> &Self::App;
    fn current(&self) -> &CurrentState<Self::Current>;
}
