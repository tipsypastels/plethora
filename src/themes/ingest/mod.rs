use super::Theme;
use crate::themes::templates::{Parser, Templates};
use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use futures::{Stream, StreamExt};
use kstring::KString;
use liquid::partials::{EagerCompiler, InMemorySource};
use std::{future::Future, pin::pin};

#[cfg(feature = "baked-themes")]
mod baked;
mod files;

#[cfg(feature = "baked-themes")]
pub use baked::Baked;
pub use files::Files;

const LIQUID_EXT: &str = "liquid";

pub trait Ingest {
    type Data;
    async fn ingest(data: Self::Data) -> Result<Theme>;
}

pub trait IngestImpl {
    type Data;

    const KIND: &'static str;

    fn new(data: Self::Data) -> Self;
    fn slug(&self) -> Result<KString>;
    fn manifest(&self) -> impl Future<Output = Result<KString>>;
    fn entries(self) -> impl Stream<Item = File>;
}

impl<I, D> Ingest for I
where
    I: IngestImpl<Data = D>,
{
    type Data = D;

    async fn ingest(data: D) -> Result<Theme> {
        async fn inner<I>(this: I, slug: KString) -> Result<Theme>
        where
            I: IngestImpl,
        {
            let manifest = this.manifest().await.context("manifest read error")?;
            let manifest = toml::from_str(&manifest).context("manifest parse error")?;

            let mut entries = pin!(this.entries());
            let mut partials = EagerCompiler::<InMemorySource>::empty();

            while let Some(entry) = entries.next().await {
                partials.add(entry.path, entry.text);
            }

            let parser = Parser::new(partials)?;
            let templates = Templates::new(&parser);

            let theme = Theme {
                slug,
                manifest,
                templates,
            };

            tracing::debug!(target: "plethora::themes", theme = %theme.slug, from = %I::KIND, "theme ingested");
            Ok(theme)
        }

        let this = I::new(data);
        let kind = Self::KIND;
        let slug = this.slug()?;
        let res = inner::<I>(this, slug.clone()).await;

        res.with_context(|| format!("failed to resolve {slug} theme with {kind} ingest"))
    }
}

pub trait IngestMany {
    type Dataset;

    async fn ingest_many<Fut>(dataset: Self::Dataset, f: impl Fn(Theme) -> Fut) -> Result<()>
    where
        Fut: Future<Output = Result<()>>;
}

pub trait IngestManyImpl: IngestImpl {
    type Dataset;

    fn buffered<F: Future>(stream: impl Stream<Item = F>) -> impl Stream<Item = F::Output>;

    async fn dirs(dataset: Self::Dataset) -> impl Stream<Item = Utf8PathBuf>;
    async fn data_stream(dataset: Self::Dataset) -> impl Stream<Item = Self::Data>;
}

impl<I, D> IngestMany for I
where
    I: IngestManyImpl<Dataset = D>,
{
    type Dataset = D;

    async fn ingest_many<Fut>(dataset: D, f: impl Fn(Theme) -> Fut) -> Result<()>
    where
        Fut: Future<Output = Result<()>>,
    {
        let data_stream = I::data_stream(dataset).await;
        let stream = data_stream.map(|data| Self::ingest(data));
        let mut stream = pin!(I::buffered(stream));

        while let Some(result) = stream.next().await {
            let theme = result?;
            f(theme).await?;
        }

        Ok(())
    }
}

struct File {
    path: String,
    text: String,
}
