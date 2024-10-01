use self::{core::Ex, impls::*};
use super::parser::LanguageExt;
use liquid_core::Language;

mod core;
mod impls;

pub fn extension(language: &mut Language) {
    language
        // Blocks
        .block(Ex(Contain))
        .block(Ex(Macro))
        // Tags
        .tag(Ex(Default))
        .tag(Ex(Js))
        .tag(Ex(Include))
        .tag(Ex(Render))
        .tag(Ex(Title));
}
