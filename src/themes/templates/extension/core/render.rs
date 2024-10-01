use super::{l, Ex};
use std::io::Write;

pub trait Render: Send + Sync + 'static {
    fn render(&self, writer: &mut dyn Write, runtime: &dyn l::Runtime) -> l::Result<()>;
}

pub struct RenderFn<C>(
    pub C,
    pub fn(&C, &mut dyn Write, &dyn l::Runtime) -> l::Result<()>,
);

impl<C> Render for RenderFn<C>
where
    C: Send + Sync + 'static,
{
    fn render(&self, writer: &mut dyn Write, runtime: &dyn l::Runtime) -> l::Result<()> {
        self.1(&self.0, writer, runtime)
    }
}

impl<R: Render> l::Renderable for Ex<R> {
    fn render_to(&self, writer: &mut dyn Write, runtime: &dyn l::Runtime) -> l::Result<()> {
        self.0.render(writer, runtime)
    }
}
