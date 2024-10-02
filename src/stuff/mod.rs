use camino::Utf8Path;
use kstring::KString;
use std::{ops::Deref, sync::OnceLock};

mod builder;
mod trace;

pub use builder::{builder, StuffBuilder, StuffGuard};
pub use trace::StuffTraceFilter;

pub static STUFF: StuffLock = StuffLock {
    lock: OnceLock::new(),
};

#[derive(Debug)]
pub struct StuffLock {
    lock: OnceLock<Stuff>,
}

impl Deref for StuffLock {
    type Target = Stuff;

    fn deref(&self) -> &Self::Target {
        self.lock.get().expect("stuff accessed before init")
    }
}

#[derive(Debug)]
pub struct Stuff {
    pub db: StuffDb,
    pub lang: StuffLang,
    pub log: StuffLog,
    pub public: StuffPublic,
    pub reload: bool,
    pub root: Box<Utf8Path>,
    pub scratch: StuffScratch,
    pub scripts: StuffScripts,
    pub setup: StuffSetup,
    pub templates: StuffTemplates,
    pub themes: StuffThemes,
    pub web: StuffWeb,
}

#[derive(Debug)]
pub struct StuffDb {
    pub url: Box<str>,
}

#[derive(Debug)]
pub struct StuffLang {
    #[cfg(feature = "langdir")]
    pub dir: Box<Utf8Path>,
}

#[derive(Debug)]
pub struct StuffLog {
    #[cfg(feature = "packaged")]
    pub dir: Box<Utf8Path>,
    pub filter: StuffTraceFilter,
}

#[derive(Debug)]
pub struct StuffPublic {
    pub dir: Box<Utf8Path>,
}

#[derive(Debug)]
pub struct StuffScratch {
    pub dir: Box<Utf8Path>,
}

#[derive(Debug)]
pub struct StuffScripts {
    pub dir: Box<Utf8Path>,
    pub glob: Box<str>,
    pub autoload: Box<[KString]>,
}

#[derive(Debug)]
pub struct StuffSetup {
    pub theme: Box<str>,
}

#[derive(Debug)]
pub struct StuffTemplates {
    pub boundary_comments: bool,
}

#[derive(Debug)]
pub struct StuffThemes {
    pub dir: Box<Utf8Path>,
    pub manifest_path: Box<Utf8Path>,
}

#[derive(Debug)]
pub struct StuffWeb {
    pub addr: Box<str>,
}
