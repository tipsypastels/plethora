use self::ingest::Ingest;
use crate::{reload::Reload, stuff::STUFF, styles::Styles};
use anyhow::Result;
use camino::{Utf8Path, Utf8PathBuf};
use dashmap::DashMap;
use ingest::IngestMany;
use kstring::KString;
use std::{fmt, future::Future, ops::Deref, sync::Arc};

mod builder;
mod ingest;
mod templates;
mod theme;

pub use builder::ThemesBuilder;
pub use liquid::object as props;
pub use theme::{Theme, ThemeManifest, ThemeManifestTailwind};

#[derive(Debug, Clone)]
pub struct Themes {
    map: Arc<DashMap<KString, Theme>>,
    styles: Styles,
}

impl Themes {
    pub async fn new(styles: Styles) -> Result<Self> {
        Self::builder(styles).build().await
    }

    pub fn builder(styles: Styles) -> ThemesBuilder {
        ThemesBuilder::new(styles)
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

#[allow(clippy::manual_async_fn)]
impl Reload for Themes {
    fn dir(&self) -> Option<&'static Utf8Path> {
        Some(&STUFF.themes.dir)
    }

    fn reload(&self, mut path: Utf8PathBuf) -> impl Future<Output = Result<()>> + Send + 'static {
        let this = self.clone();

        async move {
            while path.parent() != Some(&STUFF.themes.dir) {
                path.pop();
            }
            this.ingest::<ingest::Files>(path).await
        }
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
