use super::{Reload, DEBOUNCE};
use crate::stuff::STUFF;
use anyhow::Result;
use camino::{Utf8Path, Utf8PathBuf};
use notify_debouncer_full::{
    new_debouncer,
    notify::{RecommendedWatcher, RecursiveMode::Recursive, Watcher as _},
    DebounceEventHandler, DebounceEventResult, Debouncer, FileIdMap,
};
use tokio::sync::broadcast;
use tracing::Instrument;

#[derive(Debug)]
pub struct Proc {
    _debouncer: Debouncer<RecommendedWatcher, FileIdMap>,
}

impl Proc {
    pub fn new(
        dir: &'static Utf8Path,
        reload: impl Reload,
        reloaded_tx: broadcast::Sender<()>,
    ) -> Result<Self> {
        let (tx, mut rx) = broadcast::channel(1);

        let handler = Handler { tx };
        let mut _debouncer = new_debouncer(DEBOUNCE, None, handler)?;

        let path = dir.as_std_path();

        _debouncer.watcher().watch(path, Recursive)?;
        _debouncer.cache().add_root(path, Recursive);

        tokio::spawn(async move {
            while let Ok(path) = rx.recv().await {
                let span = tracing::debug_span!("reload", dir = %dir);
                match reload.reload(path).instrument(span).await {
                    Ok(()) => {
                        reloaded_tx.send(()).ok();
                    }
                    Err(error) => {
                        tracing::warn!(target: "plethora::reload", %dir, "reload error: {error}");
                    }
                }
            }
        });

        Ok(Self { _debouncer })
    }
}

struct Handler {
    tx: broadcast::Sender<Utf8PathBuf>,
}

impl Handler {
    fn handle(&self, result: DebounceEventResult) -> Option<()> {
        let mut events = result.ok()?;
        let mut event = events.pop()?;
        let abs_path = Utf8PathBuf::from_path_buf(event.paths.pop()?).ok()?;
        let path = abs_path.strip_prefix(STUFF.root.as_ref()).ok()?;

        self.tx.send(path.to_owned()).ok();
        Some(())
    }
}

impl DebounceEventHandler for Handler {
    fn handle_event(&mut self, result: DebounceEventResult) {
        self.handle(result);
    }
}
