use self::{
    frame::MostlySandboxedStackFrame,
    output::{Output, OutputHelper},
};
use super::prelude::{Render as RenderTrait, *};
use liquid_core::runtime::{GlobalFrame, StackFrame};

mod frame;
mod output;

#[derive(Clone)]
pub struct Contain;

impl Block for Contain {
    const START: &'static str = "contain";
    const END: &'static str = "endcontain";

    fn block(&self, args: Args, body: Body, language: &Language) -> Result<impl RenderTrait> {
        let (name, kwargs) = name_and_kwargs(args)?;
        let template = body.template(language)?;
        Ok(Output::<Self>::new(name, kwargs, Some(template)))
    }
}

impl OutputHelper for Contain {
    const TAG: &'static str = Self::START;
    const MACROS: bool = true;

    fn frame<'a>(runtime: &'a dyn Runtime, kwargs: EvaluatedKwargs<'a>) -> impl Runtime + 'a {
        GlobalFrame::new(MostlySandboxedStackFrame::new(runtime, kwargs))
    }
}

#[derive(Clone)]
pub struct Include;

impl Tag for Include {
    const NAME: &'static str = "include";

    fn tag(&self, args: Args, _language: &Language) -> Result<impl RenderTrait> {
        let (name, kwargs) = name_and_kwargs(args)?;
        Ok(Output::<Self>::new(name, kwargs, None))
    }
}

impl OutputHelper for Include {
    const TAG: &'static str = Include::NAME;
    const MACROS: bool = false;

    fn frame<'a>(runtime: &'a dyn Runtime, kwargs: EvaluatedKwargs<'a>) -> impl Runtime + 'a {
        StackFrame::new(runtime, kwargs)
    }
}

#[derive(Clone)]
pub struct Render;

impl Tag for Render {
    const NAME: &'static str = "render";

    fn tag(&self, args: Args, _language: &Language) -> Result<impl RenderTrait> {
        let (name, kwargs) = name_and_kwargs(args)?;
        Ok(Output::<Self>::new(name, kwargs, None))
    }
}

impl OutputHelper for Render {
    const TAG: &'static str = Render::NAME;
    const MACROS: bool = true;

    fn frame<'a>(runtime: &'a dyn Runtime, kwargs: EvaluatedKwargs<'a>) -> impl Runtime + 'a {
        GlobalFrame::new(MostlySandboxedStackFrame::new(runtime, kwargs))
    }
}

fn name_and_kwargs(mut args: Args) -> Result<(Expression, Kwargs)> {
    let name = args.expression()?;
    if args.comma().is_err() {
        return Ok((name, Kwargs::default()));
    }

    let kwargs = args.kwargs()?;
    args.empty()?;

    Ok((name, kwargs))
}
