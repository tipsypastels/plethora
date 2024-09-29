use super::{Stuff, STUFF};
use anyhow::{ensure, Result};
use camino::Utf8PathBuf;
use config::{builder::DefaultState, Config, ConfigBuilder, File, FileFormat};
use std::env;

const DEFAULT: &str = include_str!("./default.toml");

pub fn builder() -> StuffBuilder {
    StuffBuilder::new()
}

#[derive(Debug)]
pub struct StuffBuilder {
    config: ConfigBuilder<DefaultState>,
}

impl StuffBuilder {
    pub(super) fn new() -> Self {
        let config = ConfigBuilder::<DefaultState>::default()
            .add_source(File::from_str(DEFAULT, FileFormat::Toml));

        Self { config }
    }

    pub fn default(self, s: &str) -> Self {
        let file = File::from_str(s, FileFormat::Toml);
        let config = self.config.add_source(file);
        Self { config }
    }

    pub fn file(self, path: impl AsRef<str>, required: bool) -> Self {
        let file = File::new(path.as_ref(), FileFormat::Toml).required(required);
        let config = self.config.add_source(file);
        Self { config }
    }

    pub fn reload(self, reload: bool) -> Self {
        let config = self
            .config
            .set_override("reload", reload)
            .expect("reload is a valid key");
        Self { config }
    }

    pub fn init(self) -> Result<StuffGuard> {
        let config = self.config.build()?;
        let stuff = make(config)?;

        let guard = super::trace::init(&stuff)?;
        let guard = StuffGuard { _trace: guard };

        tracing::debug!("{stuff:#?}");
        ensure!(STUFF.lock.set(stuff).is_ok(), "stuff initialized twice");

        Ok(guard)
    }
}

#[derive(Debug)]
pub struct StuffGuard {
    _trace: super::trace::StuffTraceGuard,
}

fn make(config: Config) -> Result<Stuff> {
    use super::*;
    Ok(Stuff {
        db: StuffDb {
            url: config.get("db.url")?,
        },
        lang: StuffLang {
            #[cfg(feature = "langdir")]
            dir: config.get("lang.dir")?,
        },
        log: StuffLog {
            #[cfg(feature = "packaged")]
            dir: config.get("log.dir")?,
            filter: config.get("log.filter")?,
        },
        public: StuffPublic {
            dir: config.get("public.dir")?,
        },
        reload: config.get("reload")?,
        root: env::var("CARGO_MANIFEST_DIR")
            .map(Utf8PathBuf::from)
            .map(Into::into)?,
        scratch: StuffScratch {
            dir: config.get("scratch.dir")?,
        },
        scripts: StuffScripts {
            dir: config.get("scripts.dir")?,
            glob: config.get("scripts.glob")?,
            autoload: config.get("scripts.autoload")?,
        },
        setup: StuffSetup {
            theme: config.get("setup.theme")?,
        },
        templates: StuffTemplates {
            boundary_comments: config.get("templates.boundary_comments")?,
        },
        themes: StuffThemes {
            dir: config.get("themes.dir")?,
            manifest_path: config.get("themes.manifest_path")?,
        },
        web: StuffWeb {
            addr: config.get("web.addr")?,
        },
    })
}
