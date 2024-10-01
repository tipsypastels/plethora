use std::{
    pin::Pin,
    task::{Context, Poll},
};

use camino::{Utf8Path, Utf8PathBuf};
use futures::{future::Either, stream, Stream, StreamExt};

macro_rules! t {
    () => { ReadDir<impl Iterator<Item = Utf8PathBuf>> };
    (async) => { ReadDirAsync<impl Stream<Item = Utf8PathBuf> + Send + 'static> }
}

pub fn read_dir(path: &Utf8Path) -> t!() {
    ReadDir {
        iter: std::fs::read_dir(path)
            .map(|i| OrEmptyIter(Some(i)))
            .unwrap_or(OrEmptyIter(None))
            .filter_map(|e| e.ok())
            .filter_map(|e| Utf8PathBuf::from_path_buf(e.path()).ok()),
    }
}

pub fn walk_dir(path: &Utf8Path) -> t!() {
    ReadDir {
        iter: walkdir::WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter_map(|e| Utf8PathBuf::from_path_buf(e.into_path()).ok()),
    }
}

pub async fn read_dir_async(path: &Utf8Path) -> t!(async) {
    ReadDirAsync {
        stream: tokio::fs::read_dir(path)
            .await
            .map(|s| Either::Left(tokio_stream::wrappers::ReadDirStream::new(s)))
            .unwrap_or(Either::Right(stream::empty()))
            .filter_map(|e| async move { e.ok() })
            .filter_map(|e| async move { Utf8PathBuf::from_path_buf(e.path()).ok() }),
    }
}

pub fn walk_dir_async(path: &Utf8Path) -> t!(async) {
    async fn read(path: Utf8PathBuf, stack: &mut Vec<Utf8PathBuf>) -> Vec<Utf8PathBuf> {
        let Ok(mut dir) = tokio::fs::read_dir(&path).await else {
            return Vec::new();
        };

        let mut files = Vec::new();

        while let Some(entry) = dir.next_entry().await.transpose() {
            let Ok(entry) = entry else {
                continue;
            };
            let Ok(metadata) = entry.metadata().await else {
                continue;
            };
            let Ok(path) = Utf8PathBuf::from_path_buf(entry.path()) else {
                continue;
            };
            if metadata.is_dir() {
                stack.push(path);
            } else {
                files.push(path);
            }
        }
        files
    }

    ReadDirAsync {
        stream: stream::unfold(vec![path.to_owned()], |mut stack| async {
            let path = stack.pop()?;
            let files = stream::iter(read(path, &mut stack).await);
            Some((files, stack))
        })
        .flatten(),
    }
}

pub struct ReadDir<I> {
    iter: I,
}

impl<I> ReadDir<I>
where
    I: Iterator<Item = Utf8PathBuf>,
{
    pub fn files(self) -> t!() {
        ReadDir {
            iter: self.iter.filter(|p| p.is_file()),
        }
    }

    pub fn dirs(self) -> t!() {
        ReadDir {
            iter: self.iter.filter(|p| p.is_dir()),
        }
    }
}

impl<I> Iterator for ReadDir<I>
where
    I: Iterator<Item = Utf8PathBuf>,
{
    type Item = Utf8PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

pin_project_lite::pin_project! {
    pub struct ReadDirAsync<S> {
        #[pin]
        stream: S,
    }
}

impl<S> ReadDirAsync<S>
where
    S: Stream<Item = Utf8PathBuf> + Send + 'static,
{
    pub fn files(self) -> t!(async) {
        ReadDirAsync {
            stream: self.stream.filter(|p| {
                let is_file = p.is_file();
                async move { is_file }
            }),
        }
    }

    pub fn dirs(self) -> t!(async) {
        ReadDirAsync {
            stream: self.stream.filter(|p| {
                let is_dir = p.is_dir();
                async move { is_dir }
            }),
        }
    }
}

impl<S> Stream for ReadDirAsync<S>
where
    S: Stream<Item = Utf8PathBuf> + Send + 'static,
{
    type Item = Utf8PathBuf;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().stream.poll_next(cx)
    }
}

struct OrEmptyIter<I>(Option<I>);

impl<I: Iterator> Iterator for OrEmptyIter<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.as_mut()?.next()
    }
}
