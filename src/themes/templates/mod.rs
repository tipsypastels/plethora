use liquid::model::ScalarCow;
use liquid_core::{runtime, Value};
use std::{fmt, sync::Arc};

mod parser;

pub use parser::Parser;

const BASE: &str = r#"{% include template %}"#;

pub struct Templates {
    template: runtime::Template,
    partials: Arc<dyn runtime::PartialStore + Send + Sync>,
}

impl Templates {
    pub fn new(parser: &Parser) -> Self {
        let render = liquid_core::parser::parse(BASE, &parser.language).expect("invalid BASE");
        let template = runtime::Template::new(render);
        let partials = parser.partials.clone();

        Self { template, partials }
    }
}

impl fmt::Debug for Templates {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.partials.names()).finish()
    }
}

pub struct Snapshot<'a> {
    runtime: Box<dyn runtime::Runtime + 'a>,
}

impl Snapshot<'_> {
    #[allow(unused)]
    pub fn get<'k>(&self, key: impl Into<ScalarCow<'k>>) -> Option<Value> {
        self.runtime.get(&[key.into()]).map(|v| v.into_owned()).ok()
    }

    pub fn runtime(&self) -> &dyn runtime::Runtime {
        self.runtime.as_ref()
    }
}
