use super::{l, Args, Body, Ex, Render};

pub trait Tag: Clone + Send + Sync + 'static {
    const NAME: &'static str;
    fn tag(&self, args: Args, language: &l::Language) -> l::Result<impl Render>;
}

impl<T: Tag> l::ParseTag for Ex<T> {
    fn parse(
        &self,
        iter: l::TagTokenIter,
        language: &l::Language,
    ) -> l::Result<Box<dyn l::Renderable>> {
        let args = Args::new(iter);
        let render = self.0.tag(args, language)?;
        Ok(Box::new(Ex(render)))
    }

    fn reflection(&self) -> &dyn l::TagReflection {
        self
    }
}

impl<T: Tag> l::TagReflection for Ex<T> {
    fn tag(&self) -> &str {
        T::NAME
    }

    fn description(&self) -> &str {
        ""
    }
}

pub trait Block: Clone + Send + Sync + 'static {
    const START: &'static str;
    const END: &'static str;
    fn block(&self, args: Args, body: Body, language: &l::Language) -> l::Result<impl Render>;
}

impl<B: Block> l::ParseBlock for Ex<B> {
    fn parse(
        &self,
        iter: l::TagTokenIter,
        block: l::TagBlock,
        language: &l::Language,
    ) -> l::Result<Box<dyn l::Renderable>> {
        let args = Args::new(iter);
        let body = Body::new(block);
        let render = self.0.block(args, body, language)?;
        Ok(Box::new(Ex(render)))
    }

    fn reflection(&self) -> &dyn l::BlockReflection {
        self
    }
}

impl<B: Block> l::BlockReflection for Ex<B> {
    fn start_tag(&self) -> &str {
        B::START
    }

    fn end_tag(&self) -> &str {
        B::END
    }

    fn description(&self) -> &str {
        ""
    }
}
