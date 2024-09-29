use crate::styles::Styles;
use anyhow::Result;
use dashmap::DashMap;
use kstring::KString;
use std::sync::Arc;

mod templates;
mod theme;

pub use theme::{Theme, ThemeManifest, ThemeManifestTailwind};

#[derive(Debug, Clone)]
pub struct Themes {
    map: Arc<DashMap<KString, Theme>>,
    styles: Styles,
}

impl Themes {
    pub async fn new(styles: Styles) -> Result<Self> {
        let map = Arc::new(DashMap::new());
        let this = Self { map, styles };

        Ok(this)
    }
}
