use super::templates::Templates;
use crate::{scratch, stuff::STUFF};
use camino::Utf8PathBuf;
use kstring::KString;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct Theme {
    slug: KString,
    #[serde(flatten)]
    manifest: ThemeManifest,
    #[serde(skip)]
    templates: Templates,
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
    pub fn slug(&self) -> &str {
        &self.slug
    }

    pub fn name(&self) -> &str {
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
}
