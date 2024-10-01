use super::prelude::*;

#[derive(Clone)]
pub struct Default;

impl Tag for Default {
    const NAME: &'static str = "default";

    fn tag(&self, mut args: Args, language: &Language) -> Result<impl Render> {
        let dst = args.identifier()?;
        args.exact("Equals expected.", "=")?;

        let src = args.filter_chain(language)?;
        args.empty()?;

        Ok(RenderFn((dst, src), |(dst, src), _writer, runtime| {
            if runtime.try_get(&[dst.clone().into()]).is_some() {
                return Ok(());
            }

            let trace = || format!("{{% default {dst} = {src} %}}").into();
            let value = src.evaluate(runtime).trace_with(trace)?.into_owned();

            runtime.set_global(dst.clone(), value);
            Ok(())
        }))
    }
}
