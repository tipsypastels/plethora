use super::{IngestImpl, IngestManyImpl};
use crate::{
    helper::fs::{read_dir_async, walk_dir_async},
    stuff::STUFF,
};
use anyhow::Result;
use camino::{Utf8Path, Utf8PathBuf};
use futures::{Stream, StreamExt};
use kstring::KString;
use std::{future::Future, sync::Arc};
use tokio::fs;

const CONCURRENCY: usize = 10;

pub struct Files {
    dir: Arc<Utf8Path>,
    slug: KString,
}

#[allow(private_interfaces)]
impl IngestImpl for Files {
    type Data = Utf8PathBuf;

    const KIND: &'static str = "files";

    fn new(dir: Utf8PathBuf) -> Self {
        let dir = Arc::<Utf8Path>::from(dir);
        let slug = dir.file_name().expect("empty file name");
        let slug = KString::from_string(slug.to_string());

        Self { dir, slug }
    }

    fn slug(&self) -> Result<KString> {
        Ok(self.slug.clone())
    }

    fn manifest(&self) -> impl Future<Output = Result<KString>> {
        let path = self.dir.join(&STUFF.themes.manifest_path);
        async move {
            let text = fs::read_to_string(&path).await?;
            Ok(KString::from_string(text))
        }
    }

    fn entries(self) -> impl Stream<Item = super::File> {
        // Can't use a regular .filter because that takes a reference
        // and thus has lifetime issues in async streams.
        async fn filter_ext(path: Utf8PathBuf) -> Option<Utf8PathBuf> {
            (path.extension() == Some(super::LIQUID_EXT)).then_some(path)
        }

        async fn read(path: Utf8PathBuf, theme_dir: Arc<Utf8Path>) -> Result<super::File> {
            let text = fs::read_to_string(&path).await?;
            let path = path.strip_prefix(theme_dir.as_ref())?.to_string();

            Ok(super::File { path, text })
        }

        walk_dir_async(&self.dir)
            .filter_map(filter_ext)
            .map(move |p| read(p, self.dir.clone()))
            .buffer_unordered(CONCURRENCY)
            .filter_map(|r| async move { r.ok() })
    }
}

impl IngestManyImpl for Files {
    type Dataset = ();

    fn buffered<F: Future>(stream: impl Stream<Item = F>) -> impl Stream<Item = F::Output> {
        stream.buffer_unordered(CONCURRENCY)
    }

    async fn dirs((): ()) -> impl Stream<Item = Utf8PathBuf> {
        read_dir_async(&STUFF.themes.dir).await
    }

    async fn data_stream((): ()) -> impl Stream<Item = Self::Data> {
        Self::dirs(()).await
    }
}
