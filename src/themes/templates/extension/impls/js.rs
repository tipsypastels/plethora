use super::prelude::*;
use std::mem;

#[derive(Clone)]
pub struct Js;

impl Tag for Js {
    const NAME: &'static str = "js";

    fn tag(&self, mut args: Args, _language: &Language) -> Result<impl Render> {
        let js = args.string_literal()?;
        args.empty()?;

        Ok(RenderFn(js, |js, _, runtime| {
            runtime.registers().get_mut::<Register>().0.push(js.clone());
            Ok(())
        }))
    }
}

impl Snapshot<'_> {
    pub fn included_scripts(&self) -> Vec<KString> {
        mem::take(&mut self.runtime().registers().get_mut::<Register>().0)
    }
}

#[derive(Default)]
struct Register(Vec<KString>);
