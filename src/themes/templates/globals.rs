use crate::{
    serve::{Current, CurrentState},
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

    fn insert_shared<C: Current>(&mut self, shared: SharedGlobals<C>) {
        self.insert("current_user", shared.current.user.get());
        self.insert("current_session", shared.current.session.get());
        self.insert("current_theme", shared.theme);
        self.insert("template", shared.template);
    }
}

pub struct SharedGlobals<'a, C: Current> {
    pub current: &'a CurrentState<C>,
    pub theme: &'a Theme,
    pub template: &'a str,
}

pub struct TemplateGlobals<'a, C: Current> {
    pub props: Object,
    pub shared: SharedGlobals<'a, C>,
}

impl<C: Current> From<TemplateGlobals<'_, C>> for Globals {
    fn from(globals: TemplateGlobals<'_, C>) -> Self {
        let mut this = Self(globals.props);

        this.insert_shared(globals.shared);
        this
    }
}

pub struct LayoutGlobals<'a, C: Current> {
    pub shared: SharedGlobals<'a, C>,
    pub title: Option<&'a str>,
    pub content: &'a str,
    pub scripts: &'a [Box<str>],
}

impl<C: Current> From<LayoutGlobals<'_, C>> for Globals {
    fn from(globals: LayoutGlobals<'_, C>) -> Self {
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

pub struct ErrorGlobals<'a, C: Current> {
    pub shared: SharedGlobals<'a, C>,
    pub error: &'a Error,
}

impl<C: Current> From<ErrorGlobals<'_, C>> for Globals {
    fn from(globals: ErrorGlobals<'_, C>) -> Self {
        let mut this = Self(Object::new());

        this.insert_shared(globals.shared);
        this.insert("error", format!("{:?}", globals.error));
        this
    }
}

pub struct NotFoundGlobals<'a, C: Current> {
    pub shared: SharedGlobals<'a, C>,
}

impl<C: Current> From<NotFoundGlobals<'_, C>> for Globals {
    fn from(globals: NotFoundGlobals<'_, C>) -> Self {
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
