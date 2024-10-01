use super::frame::WitnessFrame;
use crate::{
    stuff::STUFF,
    themes::templates::extension::impls::{
        prelude::*,
        r#macro::{SavedMacroId, SavedMacroMap},
    },
};
use liquid_core::Renderable as _;
use std::{marker::PhantomData, str::FromStr};

pub struct Output<O> {
    name: Expression,
    kwargs: Kwargs,
    contain: Option<Template>,
    _ty: PhantomData<O>,
}

impl<O> Output<O> {
    pub fn new(name: Expression, kwargs: Kwargs, contain: Option<Template>) -> Self {
        Self {
            name,
            kwargs,
            contain,
            _ty: PhantomData,
        }
    }
}

impl<O: OutputHelper> Render for Output<O> {
    fn render(&self, writer: &mut dyn Write, runtime: &dyn Runtime) -> Result<()> {
        let name = self.name.evaluate(runtime)?.to_kstr().into_owned();
        let mut kwargs = self.kwargs.evaluate(runtime)?;
        let witness = WitnessFrame::new(runtime);

        if let Some(contain) = self.contain.as_ref() {
            contain.render_to(&mut std::io::sink(), &witness)?;
            kwargs.extend(witness.witnessed());
        }

        let frame = O::frame(runtime, kwargs);

        match SavedMacroId::from_str(&name) {
            Ok(macro_id) => {
                if !O::MACROS {
                    return Error::with_msg(format!("Can not `{}` macros", O::TAG)).into_err();
                }

                let r#macro = {
                    let map = runtime.registers().get_mut::<SavedMacroMap>();
                    map.get(macro_id)
                        .ok_or_else(|| Error::with_msg(format!("Unknown macro {}", self.name)))?
                };

                r#macro.template.render_to(writer, &frame)
            }
            Err(_) => {
                let partial = frame.partials().get(&format!("{name}.liquid"))?;
                let comment = should_write_boundary_comments(runtime);

                if comment {
                    write!(writer, "<!-- start-template:{name} -->").ok();
                }

                partial.render_to(writer, &frame)?;

                if comment {
                    write!(writer, "<!-- end-template:{name} -->").ok();
                }

                Ok(())
            }
        }
    }
}

pub trait OutputHelper: Send + Sync + 'static {
    const TAG: &'static str;
    const MACROS: bool;

    fn frame<'a>(runtime: &'a dyn Runtime, kwargs: EvaluatedKwargs<'a>) -> impl Runtime + 'a;
}

// We can't write comment boundaries for the layout because they would be written before
// the <!DOCTYPE>, which is invalid.
fn should_write_boundary_comments(runtime: &dyn Runtime) -> bool {
    STUFF.templates.boundary_comments && runtime.try_get(&["is_layout".into()]).is_none()
}
