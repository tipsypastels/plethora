use super::templates::Templates;
use kstring::KString;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct Theme {
    name: KString,
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
