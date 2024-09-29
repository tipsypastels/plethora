use ahash::AHashMap;
use anyhow::Result;
use serde::Deserialize;
use tracing_subscriber::{fmt, layer::SubscriberExt, registry, util::SubscriberInitExt, EnvFilter};

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum StuffTraceFilter {
    Unit(Box<str>),
    List(AHashMap<Box<str>, Box<str>>),
}

impl StuffTraceFilter {
    fn to_env_filter(&self) -> Result<EnvFilter> {
        match self {
            Self::Unit(level) => {
                // TODO: Include consumer? Does CARGO_CRATE_NAME work?
                let filter = format!("plethora={level}");
                Ok(filter.parse()?)
            }
            Self::List(list) => {
                let mut filter = EnvFilter::default();
                for (field, level) in list {
                    filter = filter.add_directive(format!("{field}={level}").parse()?);
                }
                Ok(filter)
            }
        }
    }
}

#[derive(Debug)]
pub struct StuffTraceGuard {
    #[cfg(feature = "packaged")]
    _appender_guard: tracing_appender::non_blocking::WorkerGuard,
}

pub fn init(stuff: &super::Stuff) -> Result<StuffTraceGuard> {
    let (subscriber, guard) = {
        #[cfg(not(feature = "packaged"))]
        {
            let subscriber = registry().with(fmt::layer().without_time());
            let guard = StuffTraceGuard {};
            (subscriber, guard)
        }

        #[cfg(feature = "packaged")]
        {
            use tracing_appender::{non_blocking, rolling::hourly};
            let dir = stuff.log.dir.as_std_path();
            let (appender, _appender_guard) = non_blocking(hourly(dir, ""));
            let guard = StuffTraceGuard { _appender_guard };
            let subscriber = registry()
                .with(fmt::layer().without_time().with_target(false))
                .with(fmt::layer().json().with_writer(appender));

            (subscriber, guard)
        }
    };

    let filter = stuff.log.filter.to_env_filter()?;
    let subscriber = subscriber.with(filter);

    subscriber.try_init()?;
    Ok(guard)
}
