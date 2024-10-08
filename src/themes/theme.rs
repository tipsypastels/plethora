use super::templates::*;
use crate::{
    scratch,
    serve::{CurrentHooks, CurrentState},
    stuff::STUFF,
};
use anyhow::{Error, Result};
use camino::Utf8PathBuf;
use kstring::KString;
use liquid::Object;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct Theme {
    pub(super) slug: KString,
    #[serde(flatten)]
    pub(super) manifest: ThemeManifest,
    #[serde(skip)]
    pub(super) templates: Templates,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ThemeManifest {
    pub name: KString,
    pub layout: KString,
    pub error: KString,
    pub not_found: KString,
    pub tailwind: ThemeManifestTailwind,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ThemeManifestTailwind {
    pub input: KString,
    pub config: KString,
}

impl Theme {
    pub fn slug(&self) -> &KString {
        &self.slug
    }

    pub fn name(&self) -> &KString {
        &self.manifest.name
    }

    pub fn dir(&self) -> Utf8PathBuf {
        STUFF.themes.dir.join(self.slug.as_str())
    }

    pub fn tailwind_input_path(&self) -> Utf8PathBuf {
        self.dir().join(self.manifest.tailwind.input.as_str())
    }

    pub fn tailwind_output_path(&self) -> Utf8PathBuf {
        scratch::tailwind_output_path(self.slug.as_str())
    }

    pub fn tailwind_config_path(&self) -> Utf8PathBuf {
        self.dir().join(self.manifest.tailwind.config.as_str())
    }

    pub fn render<C: CurrentHooks>(
        &self,
        template: &str,
        base_title: Option<&str>,
        props: Object,
        current: &CurrentState<C>,
    ) -> Result<String> {
        let shared = self.shared_globals(template, current);
        let globals = TemplateGlobals { shared, props };
        self.render_inner(base_title, globals, current)
    }

    pub fn render_error<C: CurrentHooks>(
        &self,
        error: &Error,
        base_title: Option<&str>,
        current: &CurrentState<C>,
    ) -> Result<String> {
        let template = &self.manifest.error;
        let shared = self.shared_globals(template, current);
        let globals = ErrorGlobals { shared, error };
        self.render_inner(base_title, globals, current)
    }

    pub fn render_not_found<C: CurrentHooks>(
        &self,
        base_title: Option<&str>,
        current: &CurrentState<C>,
    ) -> Result<String> {
        let template = &self.manifest.not_found;
        let shared = self.shared_globals(template, current);
        let globals = NotFoundGlobals { shared };
        self.render_inner(base_title, globals, current)
    }

    fn render_inner<C: CurrentHooks>(
        &self,
        base_title: Option<&str>,
        globals: impl Into<Globals>,
        current: &CurrentState<C>,
    ) -> Result<String> {
        let globals = globals.into();
        let (content, snapshot) = self.templates.render_with_snapshot(&globals)?;
        self.render_layout(&content, base_title, snapshot, current)
    }

    fn render_layout<C: CurrentHooks>(
        &self,
        content: &str,
        base_title: Option<&str>,
        snapshot: Snapshot,
        current: &CurrentState<C>,
    ) -> Result<String> {
        let template = &self.manifest.layout;
        let mut scripts = STUFF.scripts.autoload.to_vec();
        let title = snapshot.title(base_title);

        scripts.extend(snapshot.included_scripts());

        let shared = self.shared_globals(template, current);
        let globals = LayoutGlobals {
            shared,
            title: title.as_deref(),
            content,
            scripts: &scripts,
        }
        .into();

        self.templates.render(&globals)
    }

    fn shared_globals<'a, C: CurrentHooks>(
        &'a self,
        template: &'a str,
        current: &'a CurrentState<C>,
    ) -> SharedGlobals<'a, C> {
        SharedGlobals {
            current,
            theme: self,
            template,
        }
    }
}
