use crate::stuff::STUFF;
use anyhow::Result;
use camino::{Utf8Path, Utf8PathBuf};
use tokio::fs;

const BIN_DIR: &str = "bin";
const PUBLIC_DIR: &str = "public";

const PUBLIC_JS_DIR: &str = "scripts";
const PUBLIC_CSS_DIR: &str = "styles";

pub async fn init() -> Result<()> {
    fs::create_dir_all(bin_dir()).await?;
    fs::create_dir_all(public_dir()).await?;

    Ok(())
}

pub(crate) fn bin_dir() -> Utf8PathBuf {
    dir().join(BIN_DIR)
}

pub(crate) fn public_dir() -> Utf8PathBuf {
    dir().join(PUBLIC_DIR)
}

pub(crate) fn esbuild_output_dir() -> Utf8PathBuf {
    public_dir().join(PUBLIC_JS_DIR)
}

pub(crate) fn tailwind_output_path(slug: &str) -> Utf8PathBuf {
    public_dir()
        .join(PUBLIC_CSS_DIR)
        .join(slug)
        .with_extension("css")
}

fn dir() -> &'static Utf8Path {
    &STUFF.scratch.dir
}
