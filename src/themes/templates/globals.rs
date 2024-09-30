use crate::{
    server::{CurrentState, Hooks},
    stuff::STUFF,
    themes::Theme,
};
use anyhow::Error;
use kstring::KString;
use liquid::{Object, ObjectView};
use serde::Serialize;
use std::time::SystemTime;

#[derive(Debug)]
pub struct Globals(Object);

impl Globals {
    pub fn as_object_view(&self) -> &dyn ObjectView {
        &self.0
    }

    fn insert(&mut self, key: impl Into<KString>, value: impl Serialize) {
        let value = liquid::model::to_value(&value).expect("invalid global");
        self.0.insert(key.into(), value);
    }

    fn insert_shared<H: Hooks>(&mut self, shared: SharedGlobals<H>) {
        self.insert("current_user", shared.current.user.get());
        self.insert("current_session", shared.current.session.get());
        self.insert("current_theme", shared.theme);
        self.insert("template", shared.template);
    }
}

pub struct SharedGlobals<'a, H: Hooks> {
    pub current: &'a CurrentState<H>,
    pub theme: &'a Theme,
    pub template: &'a str,
}

pub struct TemplateGlobals<'a, H: Hooks> {
    pub props: Object,
    pub shared: SharedGlobals<'a, H>,
}

impl<H: Hooks> From<TemplateGlobals<'_, H>> for Globals {
    fn from(globals: TemplateGlobals<'_, H>) -> Self {
        let mut this = Self(globals.props);

        this.insert_shared(globals.shared);
        this
    }
}

pub struct LayoutGlobals<'a, H: Hooks> {
    pub shared: SharedGlobals<'a, H>,
    pub title: Option<&'a str>,
    pub content: &'a str,
    pub scripts: &'a [Box<str>],
}

impl<H: Hooks> From<LayoutGlobals<'_, H>> for Globals {
    fn from(globals: LayoutGlobals<'_, H>) -> Self {
        let mut this = Self(Object::new());

        this.insert_shared(globals.shared);
        this.insert("title", globals.title);
        this.insert("content", globals.content);
        this.insert("scripts", globals.scripts);
        this.insert("cache_buster", cache_buster());
        this.insert("is_layout", true);
        this
    }
}

pub struct ErrorGlobals<'a, H: Hooks> {
    pub shared: SharedGlobals<'a, H>,
    pub error: &'a Error,
}

impl<H: Hooks> From<ErrorGlobals<'_, H>> for Globals {
    fn from(globals: ErrorGlobals<'_, H>) -> Self {
        let mut this = Self(Object::new());

        this.insert_shared(globals.shared);
        this.insert("error", format!("{:?}", globals.error));
        this
    }
}

pub struct NotFoundGlobals<'a, H: Hooks> {
    pub shared: SharedGlobals<'a, H>,
}

impl<H: Hooks> From<NotFoundGlobals<'_, H>> for Globals {
    fn from(globals: NotFoundGlobals<'_, H>) -> Self {
        let mut this = Self(Object::new());

        this.insert_shared(globals.shared);
        this
    }
}

fn cache_buster() -> u64 {
    if STUFF.reload {
        return SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|p| p.as_secs())
            .unwrap_or(0);
    }
    0
}
