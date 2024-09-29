use anyhow::Result;
use camino::Utf8Path;
use tokio::fs;

pub trait Flatten {
    async fn flatten(&self, name: &str, path: &Utf8Path) -> Result<()>;
}

#[derive(Debug)]
pub struct None;

impl Flatten for None {
    async fn flatten(&self, _name: &str, _path: &Utf8Path) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct Dir(pub &'static str);

impl Flatten for Dir {
    async fn flatten(&self, name: &str, path: &Utf8Path) -> Result<()> {
        let unflattened_path = path.join(self.0);
        let temp_path = path.with_file_name(format!("{name}-temp"));

        // 1. Move esbuild/package/bin to esbuild-temp.
        // 2. Delete esbuild (directory).
        // 3. Move esbuild-temp to esbuild.
        fs::rename(unflattened_path, &temp_path).await?;
        fs::remove_dir_all(path).await?;
        fs::rename(temp_path, path).await?;

        Ok(())
    }
}
