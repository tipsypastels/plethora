use anyhow::Result;
use liquid::model::ScalarCow;
use liquid_core::{runtime, Renderable, Value};
use std::{fmt, sync::Arc};

mod globals;
mod parser;

pub use globals::{
    ErrorGlobals, Globals, LayoutGlobals, NotFoundGlobals, SharedGlobals, TemplateGlobals,
};
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

    pub fn render(&self, globals: &Globals) -> Result<String> {
        self.render_with_snapshot(globals).map(|v| v.0)
    }

    pub fn render_with_snapshot<'a>(
        &'a self,
        globals: &'a Globals,
    ) -> Result<(String, Snapshot<'a>)> {
        let runtime = runtime::RuntimeBuilder::new()
            .set_globals(globals.as_object_view())
            .set_partials(self.partials.as_ref())
            .build();

        let html = self.template.render(&runtime)?;
        let snapshot = Snapshot {
            runtime: Box::new(runtime),
        };

        Ok((html, snapshot))
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
