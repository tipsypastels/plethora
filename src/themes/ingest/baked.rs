use super::{IngestImpl, IngestManyImpl};
use crate::stuff::STUFF;
use anyhow::{Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use futures::{stream, Stream, StreamExt};
use kstring::KString;
use std::{future::Future, path::Path};

type Dir = include_dir::Dir<'static>;
type DirEntry = include_dir::DirEntry<'static>;
type ThemeDir = &'static Path;

pub struct Baked {
    dir: Dir,
}

#[allow(private_interfaces)]
impl IngestImpl for Baked {
    type Data = Dir;

    const KIND: &'static str = "baked";

    fn new(dir: Dir) -> Self {
        Self { dir }
    }

    fn slug(&self) -> Result<KString> {
        let path = self.dir.path().to_str().context("baked slug not utf-8")?;
        Ok(KString::from_static(path))
    }

    fn manifest(&self) -> impl Future<Output = Result<KString>> {
        let dir = self.dir.clone();
        async move {
            let path = dir.path().join(STUFF.themes.manifest_path.as_ref());
            let manifest = dir
                .get_file(path)
                .context("no manifest")?
                .contents_utf8()
                .context("manifest not utf-8")?;

            Ok(KString::from_static(manifest))
        }
    }

    fn entries(self) -> impl Stream<Item = super::File> {
        fn read(dir: Dir, theme_dir: ThemeDir, stack: &mut Vec<Dir>) -> Vec<super::File> {
            dir.entries()
                .iter()
                .filter_map(|e| match e {
                    DirEntry::File(f) => {
                        let path = Utf8Path::from_path(f.path())?;

                        if path.extension() != Some(super::LIQUID_EXT) {
                            return None;
                        }

                        let path = path.strip_prefix(theme_dir).ok()?.to_string();
                        let text = f.contents_utf8()?.to_string();

                        Some(super::File { path, text })
                    }
                    DirEntry::Dir(d) => {
                        stack.push(d.clone());
                        None
                    }
                })
                .collect()
        }

        let theme_dir = self.dir.path();
        let stack = vec![self.dir];

        stream::unfold(stack, |mut stack| async {
            let dir = stack.pop()?;
            let files = stream::iter(read(dir, theme_dir, &mut stack));
            Some((files, stack))
        })
        .flatten()
    }
}

impl IngestManyImpl for Baked {
    type Dataset = Dir;

    fn buffered<F: Future>(stream: impl Stream<Item = F>) -> impl Stream<Item = F::Output> {
        stream.then(|future| future)
    }

    async fn dirs(dir: Dir) -> impl Stream<Item = camino::Utf8PathBuf> {
        let f = |d: &Dir| Utf8PathBuf::from_path_buf(d.path().into()).ok();
        stream::iter(dir.dirs().filter_map(f))
    }

    async fn data_stream(dir: Dir) -> impl Stream<Item = Self::Data> {
        stream::iter(dir.dirs().cloned())
    }
}
