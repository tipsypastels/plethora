use super::prelude::*;
use std::mem;

#[derive(Clone)]
pub struct Title;

impl Tag for Title {
    const NAME: &'static str = "title";

    fn tag(&self, mut args: Args, language: &Language) -> Result<impl Render> {
        let expr = args.filter_chain(language)?;
        args.empty()?;

        Ok(RenderFn(expr, |expr, _, runtime| {
            let trace = || format!("{{% title {expr} %}}").into();
            let title = expr
                .evaluate(runtime)
                .trace_with(trace)?
                .into_owned()
                .to_kstr()
                .into_owned();

            *runtime.registers().get_mut::<Register>() = Register(Some(title));
            Ok(())
        }))
    }
}

impl Snapshot<'_> {
    pub fn title(&self, base: Option<&str>) -> Option<KString> {
        let mut reg = self.runtime().registers().get_mut::<Register>();
        let title = mem::take(&mut reg.0);

        match (title, base) {
            (Some(title), Some(base)) => Some(format!("{title} • {base}").into()),
            (Some(title), None) => Some(title),
            (None, Some(base)) => Some(base.to_string().into()),
            (None, None) => None,
        }
    }
}

#[derive(Default)]
struct Register(Option<KString>);
