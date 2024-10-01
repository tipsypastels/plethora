use liquid_core as l;

mod args;
mod render;
mod traits;

pub use args::{Args, Body, EvaluatedKwargs, Kwargs};
pub use render::{Render, RenderFn};
pub use traits::{Block, Tag};

#[derive(Clone)]
pub struct Ex<T>(pub T);

impl<T> std::fmt::Debug for Ex<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ex({})", std::any::type_name::<T>())
    }
}

#[allow(unused)]
pub fn a2e(e: anyhow::Error) -> l::Error {
    l::Error::with_msg(format!("Template error: {e}."))
}
