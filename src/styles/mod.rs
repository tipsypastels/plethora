use self::process::Process;
use crate::{binary, themes::Theme};
use ahash::AHashMap;
use anyhow::Result;
use camino::Utf8Path;
use kstring::KString;
use std::sync::Arc;
use tokio::sync::Mutex;

mod process;

#[derive(Debug, Clone)]
pub struct Styles {
    binary: Arc<Utf8Path>,
    processes: Arc<Mutex<AHashMap<KString, Process>>>,
}

impl Styles {
    pub async fn new() -> Result<Self> {
        let binary = binary::TAILWIND.path().await?;

        Ok(Self {
            binary: Arc::from(binary),
            processes: Arc::new(Mutex::new(AHashMap::new())),
        })
    }

    pub async fn compile(&self, theme: &Theme) -> Result<()> {
        let mut processes = self.processes.lock().await;

        if processes.contains_key(theme.slug()) {
            return Ok(());
        }

        let process = Process::new(&self.binary, theme).await?;
        processes.insert(theme.slug().clone(), process);

        Ok(())
    }
}
