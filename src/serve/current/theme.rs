use super::{Application, CurrentHooks};
use crate::themes::{ThemeGuard, Themes};
use anyhow::{Context, Result};
use axum::extract::{Query, Request};
use serde::Deserialize;
use std::{marker::PhantomData, sync::Arc};
use tower_cookies::Cookies;

const COOKIE: &str = "plethora-theme";

#[derive(Debug)]
pub struct CurrentThemeState<C> {
    slug: Arc<str>,
    _cur: PhantomData<C>,
}

impl<C: CurrentHooks> CurrentThemeState<C> {
    pub(super) fn new(app: &impl Application, request: &Request, cookies: &Cookies) -> Self {
        let themes = &app.themes();
        let current_slug = get_slug(request, cookies);
        let slug = current_slug
            .and_then(|slug| themes.get(&slug))
            .or_else(|| themes.get(app.default_theme_slug()))
            .expect("no themes")
            .slug()
            .as_str()
            .into();

        Self {
            slug,
            _cur: PhantomData,
        }
    }

    pub fn with_fixed_theme(slug: &str) -> Self {
        Self {
            slug: Arc::from(slug),
            _cur: PhantomData,
        }
    }

    pub fn slug(&self) -> &str {
        &self.slug
    }

    pub fn resolve<'a>(&self, themes: &'a Themes) -> Result<ThemeGuard<'a>> {
        themes
            .get(&self.slug)
            .with_context(|| format!("unknown theme {}", self.slug))
    }
}

impl<C> Clone for CurrentThemeState<C> {
    fn clone(&self) -> Self {
        Self {
            slug: self.slug.clone(),
            _cur: self._cur,
        }
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
