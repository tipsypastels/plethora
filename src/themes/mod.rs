use self::ingest::Ingest;
use crate::styles::Styles;
use anyhow::Result;
use dashmap::DashMap;
use ingest::IngestMany;
use kstring::KString;
use std::{fmt, ops::Deref, sync::Arc};

mod ingest;
mod templates;
mod theme;

#[cfg(feature = "baked")]
pub use include_dir::include_dir as baked;
pub use liquid::object as props;
pub use theme::{Theme, ThemeManifest, ThemeManifestTailwind};

#[derive(Debug, Clone)]
pub struct Themes {
    map: Arc<DashMap<KString, Theme>>,
    styles: Styles,
}

#[derive(Debug)]
pub struct ThemesInit {
    pub styles: Styles,
    #[cfg(feature = "baked")]
    pub baked: include_dir::Dir<'static>,
}

impl Themes {
    pub async fn new(init: ThemesInit) -> Result<Self> {
        let ThemesInit {
            styles,
            #[cfg(feature = "baked")]
            baked,
        } = init;

        let map = Arc::new(DashMap::new());
        let this = Self { map, styles };

        #[cfg(feature = "baked")]
        this.ingest_many::<ingest::Baked>(baked).await?;
        this.ingest_many::<ingest::Files>(()).await?;

        Ok(this)
    }

    pub fn get(&self, slug: &str) -> Option<ThemeGuard> {
        self.map.get(slug).map(ThemeGuard)
    }

    pub fn iter(&self) -> ThemeIter<'_> {
        ThemeIter(self.map.iter())
    }

    async fn ingest<I: Ingest>(&self, data: I::Data) -> Result<()> {
        let theme = I::ingest(data).await?;
        self.insert(theme).await
    }

    async fn ingest_many<I: IngestMany>(&self, dataset: I::Dataset) -> Result<()> {
        I::ingest_many(dataset, |theme| self.insert(theme)).await
    }

    async fn insert(&self, theme: Theme) -> Result<()> {
        self.styles.compile(&theme).await?;
        self.map.insert(theme.slug.clone(), theme);
        Ok(())
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
