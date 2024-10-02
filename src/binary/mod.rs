use crate::scratch;
use anyhow::{Context, Result};
use camino::Utf8Path;
use futures::StreamExt;
use output::Output;
use std::env::consts::{ARCH, OS};
use tokio::{fs, sync::OnceCell};
use url::Url;

mod flatten;
mod output;

pub async fn install() -> Result<()> {
    let tailwind = TAILWIND.state().await?;
    let esbuild = ESBUILD.state().await?;
    let pnpm = PNPM.state().await?;

    if tailwind || esbuild || pnpm {
        tracing::info!("installation complete")
    } else {
        tracing::info!("nothing to do");
    }

    Ok(())
}

pub static TAILWIND: Source<output::File, flatten::None> = Source {
    name: "tailwind",
    url: "https://github.com/tailwindlabs/tailwindcss/releases/download/v3.4.4/{TARGET}",
    cell: OnceCell::const_new(),
    target: |arch, os| match (arch, os) {
        ("x86_64", "macos") => Some("tailwindcss-macos-x64"),
        _ => None,
    },
    output: output::File { mode: 0o755 },
    flatten: flatten::None,
};

pub static ESBUILD: Source<output::TarGz, flatten::Dir> = Source {
    name: "esbuild",
    url: "https://registry.npmjs.org/@esbuild/{TARGET}/-/{TARGET}-0.23.0.tgz",
    cell: OnceCell::const_new(),
    target: |arch, os| match (arch, os) {
        ("x86_64", "macos") => Some("darwin-x64"),
        _ => None,
    },
    output: output::TarGz,
    flatten: flatten::Dir("package/bin/esbuild"),
};

pub static PNPM: Source<output::File, flatten::None> = Source {
    name: "pnpm",
    url: "https://github.com/pnpm/pnpm/releases/download/v9.11.0/pnpm-{TARGET}",
    cell: OnceCell::const_new(),
    target: |arch, os| match (arch, os) {
        ("x86_64", "macos") => Some("macos-x64"),
        _ => None,
    },
    output: output::File { mode: 0o755 },
    flatten: flatten::None,
};

#[derive(Debug)]
pub struct Source<O, F> {
    name: &'static str,
    url: &'static str,
    cell: OnceCell<Box<Utf8Path>>,
    target: fn(&'static str, &'static str) -> Option<&'static str>,
    output: O,
    flatten: F,
}

impl<O, F> Source<O, F>
where
    O: output::MakeOutput,
    F: flatten::Flatten,
{
    pub async fn path(&self) -> Result<&Utf8Path> {
        self.init().await.map(|(p, _)| p)
    }

    async fn state(&self) -> Result<bool> {
        self.init().await.map(|(_, v)| v)
    }

    async fn init(&self) -> Result<(&Utf8Path, bool)> {
        let mut did = false;
        let path = self
            .cell
            .get_or_try_init(|| self.try_init(&mut did))
            .await?;

        Ok((path, did))
    }

    async fn try_init(&self, did_init: &mut bool) -> Result<Box<Utf8Path>> {
        let path = scratch::bin_dir().join(self.name);
        if !fs::try_exists(&path).await? {
            *did_init = true;
            self.download(&path)
                .await
                .with_context(|| format!("failed to download {} binary", self.name))?;
        }
        Ok(Box::from(path))
    }

    async fn download(&self, path: &Utf8Path) -> Result<()> {
        let client = reqwest::Client::new();
        let url = self.url()?;

        tracing::info!(binary = self.name, "downloading");

        let res = client.get(url).send().await?.error_for_status()?;

        let mut output = self.output(path).await?;
        let mut stream = res.bytes_stream();

        while let Some(result) = stream.next().await {
            let chunk = result.context("invalid chunk")?;
            output.output(&chunk).await.context("invalid write")?;
        }

        output.finish().await?;
        self.flatten(path).await?;

        Ok(())
    }

    fn url(&self) -> Result<Url> {
        let target = (self.target)(ARCH, OS).with_context(|| format!("no binary {ARCH}-{OS}"))?;
        let url = self.url.replace("{TARGET}", target);
        Url::parse(&url).context("invalid URL")
    }

    async fn output(&self, path: &Utf8Path) -> Result<O::Output> {
        self.output
            .make_output(path)
            .await
            .context("invalid output")
    }

    async fn flatten(&self, path: &Utf8Path) -> Result<()> {
        self.flatten
            .flatten(self.name, path)
            .await
            .context("failed to flatten output")
    }
}
