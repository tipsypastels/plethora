use std::{future::Future, process::Stdio};

use crate::{
    binary::{ESBUILD, PNPM},
    reload::Reload,
    scratch,
    stuff::STUFF,
};
use anyhow::{ensure, Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use tokio::{fs, process::Command, try_join};

mod capture;

#[derive(Debug, Clone)]
pub struct Scripts {
    esbuild: &'static Utf8Path,
}

impl Scripts {
    pub async fn new() -> Result<Self> {
        let (pnpm, esbuild) = try_join!(PNPM.path(), ESBUILD.path())?;

        deps(pnpm).await.context("pnpm error")?;
        build(esbuild).await.context("esbuild error")?;

        Ok(Self { esbuild })
    }
}

impl Reload for Scripts {
    fn dir(&self) -> Option<&'static Utf8Path> {
        Some(&STUFF.scripts.dir)
    }

    fn reload(&self, _path: Utf8PathBuf) -> impl Future<Output = Result<()>> + Send + 'static {
        build(self.esbuild)
    }
}

async fn deps(pnpm: &'static Utf8Path) -> Result<()> {
    if fs::try_exists(STUFF.scripts.dir.join("node_modules")).await? {
        return Ok(());
    }

    // We're using `current_dir`, so we have to account for that;
    let pnpm = format!("../{pnpm}");
    let mut command = Command::new(pnpm);

    let status = command
        .current_dir(STUFF.scripts.dir.as_ref())
        .kill_on_drop(true)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .arg("install")
        .status()
        .await?;

    ensure!(status.success(), "failed to install");
    tracing::debug!("javascript dependencies installed");

    Ok(())
}

async fn build(esbuild: &'static Utf8Path) -> Result<()> {
    let mut command = Command::new(esbuild);

    let input = STUFF.scripts.dir.join(STUFF.scripts.glob.as_ref());
    let output = scratch::esbuild_output_dir();
    let tsconfig = STUFF.scripts.dir.join("tsconfig.json");

    command
        .kill_on_drop(true)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .arg(input)
        .arg("--bundle")
        .arg("--minify")
        .arg("--log-level=warning")
        .arg(format!("--outdir={output}"))
        .arg(format!("--tsconfig={tsconfig}"));

    let mut child = command.spawn().context("failed to spawn esbuild")?;
    let stderr = child.stderr.take().expect("stderr should be piped");
    let status = child.wait().await.context("failed to run esbuild")?;

    capture::capture(stderr)
        .await
        .context("failed to log esbuild output")?;

    ensure!(status.success(), "esbuild failed, status {status}");
    tracing::debug!("javascript built");

    Ok(())
}
