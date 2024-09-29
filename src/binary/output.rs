use anyhow::{Context, Result};
use bytes::Bytes;
use camino::Utf8Path;
use flate2::read::GzDecoder;
use std::io;
use tar::Archive;
use tokio::{fs, io::AsyncWriteExt, task};

pub trait MakeOutput {
    type Output: Output;
    async fn make_output(&self, path: &Utf8Path) -> Result<Self::Output>;
}

pub trait Output {
    async fn output(&mut self, chunk: &Bytes) -> Result<()>;
    async fn finish(self) -> Result<()>;
}

#[derive(Debug)]
pub struct File {
    pub mode: u32,
}

impl MakeOutput for File {
    type Output = FileOutput;

    async fn make_output(&self, path: &Utf8Path) -> Result<FileOutput> {
        let mut options = fs::OpenOptions::new();
        options.write(true).create(true);
        #[cfg(unix)]
        options.mode(self.mode);

        let file = options.open(path).await.context("failed to create file")?;
        Ok(FileOutput { file })
    }
}

#[derive(Debug)]
pub struct FileOutput {
    file: fs::File,
}

impl Output for FileOutput {
    async fn output(&mut self, chunk: &Bytes) -> Result<()> {
        Ok(self.file.write_all(chunk).await?)
    }

    async fn finish(self) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct TarGz;

impl MakeOutput for TarGz {
    type Output = TarGzOutput;

    async fn make_output(&self, path: &Utf8Path) -> Result<TarGzOutput> {
        let path = path.to_owned();
        let (tx, rx) = flume::bounded(0);
        let task = task::spawn_blocking(move || {
            let reader = TarGzReader::new(rx);
            let gz = GzDecoder::new(reader);
            let mut ar = Archive::new(gz);
            ar.unpack(path)
        });

        Ok(TarGzOutput { tx, task })
    }
}

#[derive(Debug)]
pub struct TarGzOutput {
    tx: flume::Sender<Vec<u8>>,
    task: task::JoinHandle<io::Result<()>>,
}

impl Output for TarGzOutput {
    async fn output(&mut self, chunk: &Bytes) -> Result<()> {
        Ok(self.tx.send_async(chunk.to_vec()).await?)
    }

    async fn finish(self) -> Result<()> {
        let Self { tx, task } = self;
        drop(tx); // Close channel
        task.await
            .context("decompression thread error")?
            .context("decompression error")
    }
}

struct TarGzReader {
    rx: flume::Receiver<Vec<u8>>,
    cursor: io::Cursor<Vec<u8>>,
}

impl TarGzReader {
    fn new(rx: flume::Receiver<Vec<u8>>) -> Self {
        let cursor = io::Cursor::new(Vec::new());
        Self { rx, cursor }
    }

    fn current_chunk_is_exhausted(&self) -> bool {
        self.cursor.position() == self.cursor.get_ref().len() as u64
    }
}

impl io::Read for TarGzReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.current_chunk_is_exhausted() {
            if let Ok(vec) = self.rx.recv() {
                self.cursor = io::Cursor::new(vec);
            }
        }
        self.cursor.read(buf)
    }
}
