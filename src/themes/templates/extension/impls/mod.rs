mod default;
mod js;
mod r#macro;
mod render;
mod title;

pub use default::Default;
pub use js::Js;
pub use r#macro::Macro;
pub use render::{Contain, Include, Render};
pub use title::Title;

#[allow(unused)]
mod prelude {
    pub use crate::themes::templates::{extension::core::*, Snapshot};
    pub use kstring::{KString, KStringCow};
    pub use liquid_core::{
        error::ResultLiquidExt, Error, Expression, Language, Result, Runtime, Template, Value,
        ValueCow, ValueView,
    };
    pub use std::io::Write;
}
