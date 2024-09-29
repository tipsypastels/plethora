use crate::{
    server::{Hooks, Server},
    themes::{ThemeGuard, Themes},
};
use anyhow::{Context, Result};
use axum::extract::{Query, Request};
use serde::Deserialize;
use std::sync::Arc;
use tower_cookies::Cookies;

const COOKIE: &str = "plethora-theme";

#[derive(Debug, Clone)]
pub struct CurrentThemeState(Arc<str>);

impl CurrentThemeState {
    pub(super) fn new<H: Hooks>(server: &Server<H>, request: &Request, cookies: &Cookies) -> Self {
        let themes = &server.themes;
        let current_slug = get_slug(request, cookies);
        let slug = current_slug
            .and_then(|slug| themes.get(&slug))
            .or_else(|| themes.get(server.hooks.default_theme_slug()?))
            .map(|theme| theme.slug().into())
            .unwrap_or_else(|| {
                tracing::warn!("no set or default theme");
                themes.iter().next().expect("no themes").slug().into()
            });

        Self(slug)
    }

    pub fn with_fixed_theme(slug: &str) -> Self {
        Self(Arc::from(slug))
    }

    pub fn slug(&self) -> &str {
        &self.0
    }

    pub fn resolve<'a>(&self, themes: &'a Themes) -> Result<ThemeGuard<'a>> {
        themes
            .get(&self.0)
            .with_context(|| format!("unknown theme {}", self.0))
    }
}

fn get_slug(request: &Request, cookies: &Cookies) -> Option<String> {
    #[derive(Deserialize)]
    struct QueryTheme {
        theme: String,
    }

    if let Ok(Query(q)) = Query::<QueryTheme>::try_from_uri(request.uri()) {
        return Some(q.theme);
    }

    if let Some(cookie) = cookies.get(COOKIE) {
        return Some(cookie.value().to_string());
    }

    None
}
