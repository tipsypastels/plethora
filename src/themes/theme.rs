use super::templates::Templates;
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
}
