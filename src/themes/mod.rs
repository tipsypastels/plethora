use crate::styles::Styles;
use anyhow::Result;
use dashmap::DashMap;
use kstring::KString;
use std::{fmt, ops::Deref, sync::Arc};

mod templates;
mod theme;

pub use liquid::object as props;
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

    pub fn get(&self, slug: &str) -> Option<ThemeGuard> {
        self.map.get(slug).map(ThemeGuard)
    }

    pub fn iter(&self) -> ThemeIter<'_> {
        ThemeIter(self.map.iter())
    }
}

pub struct ThemeGuard<'a>(dashmap::mapref::one::Ref<'a, KString, Theme>);

impl fmt::Debug for ThemeGuard<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.deref().fmt(f)
    }
}

impl Deref for ThemeGuard<'_> {
    type Target = Theme;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

pub struct ThemeIter<'a>(dashmap::iter::Iter<'a, KString, Theme>);

impl fmt::Debug for ThemeIter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ThemeIter")
    }
}

impl<'a> Iterator for ThemeIter<'a> {
    type Item = ThemeIterItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(ThemeIterItem)
    }
}

pub struct ThemeIterItem<'a>(dashmap::mapref::multiple::RefMulti<'a, KString, Theme>);

impl fmt::Debug for ThemeIterItem<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.deref().fmt(f)
    }
}

impl Deref for ThemeIterItem<'_> {
    type Target = Theme;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
