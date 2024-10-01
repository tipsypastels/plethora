use self::proc::Proc;
use crate::stuff::STUFF;
use anyhow::Result;
use axum::response::sse::KeepAlive;
use camino::{Utf8Path, Utf8PathBuf};
use std::{cell::OnceCell, future::Future, sync::Arc, time::Duration};
use tokio::sync::broadcast::{self, error::RecvError};

mod proc;

const DEBOUNCE: Duration = Duration::from_secs(1);

pub trait Reload: Send + 'static {
    fn dir(&self) -> Option<&'static Utf8Path>;
    fn reload(&self, path: Utf8PathBuf) -> impl Future<Output = Result<()>> + Send + 'static;
}

#[derive(Debug, Clone)]
pub struct Reloader {
    reloaded_tx: Option<broadcast::Sender<()>>,
    _procs: Option<Arc<[Proc]>>,
}

impl Reloader {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> ReloadBuilder {
        ReloadBuilder {
            reloaded: OnceCell::new(),
            procs: Vec::new(),
        }
    }

    pub fn keep_alive(&self) -> KeepAlive {
        KeepAlive::new().interval(DEBOUNCE)
    }

    pub fn subscribe(&self) -> ReloadRx {
        ReloadRx {
            rx: self.reloaded_tx.as_ref().map(|tx| tx.subscribe()),
        }
    }
}

#[derive(Debug)]
pub struct ReloadBuilder {
    reloaded: OnceCell<(broadcast::Sender<()>, broadcast::Receiver<()>)>,
    procs: Vec<Proc>,
}

impl ReloadBuilder {
    pub fn reload<R: Reload>(mut self, reload: R) -> Self {
        if STUFF.reload {
            let Some(dir) = reload.dir() else {
                return self;
            };

            let reloaded = self.reloaded.get_or_init(|| broadcast::channel(1));
            let reloaded_tx = reloaded.0.clone();

            let proc = Proc::new(dir, reload, reloaded_tx)
                .unwrap_or_else(|_| panic!("failed to watch {dir} for reload"));

            tracing::debug!(%dir, "watching for reload");
            self.procs.push(proc);
        }

        self
    }

    pub fn build(self) -> Reloader {
        Reloader {
            reloaded_tx: self.reloaded.get().map(|(tx, _)| tx.clone()),
            _procs: if self.procs.is_empty() {
                None
            } else {
                Some(Arc::from(self.procs))
            },
        }
    }
}

#[derive(Debug)]
pub struct ReloadRx {
    rx: Option<broadcast::Receiver<()>>,
}

impl ReloadRx {
    pub async fn recv(&mut self) -> Result<(), RecvError> {
        self.rx.as_mut().ok_or(RecvError::Closed)?.recv().await
    }
}
