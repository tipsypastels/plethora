use super::prelude::*;
use ahash::AHashMap;
use std::{str::FromStr, sync::Arc};
use uuid::Uuid;

#[derive(Clone)]
pub struct Macro;

impl Block for Macro {
    const START: &'static str = "macro";
    const END: &'static str = "endmacro";

    fn block(&self, mut args: Args, body: Body, language: &Language) -> Result<impl Render> {
        let name = args.identifier()?;
        let template = Arc::new(body.template(language)?);
        args.empty()?;

        Ok(RenderFn(
            (name, template),
            |(name, template), _, runtime| {
                let key = name.clone();
                let id = SavedMacroId::new();
                let r#macro = SavedMacro {
                    template: template.clone(),
                };

                runtime.set_global(key, id.to_value());
                runtime
                    .registers()
                    .get_mut::<SavedMacroMap>()
                    .save(id, r#macro);

                Ok(())
            },
        ))
    }
}

#[derive(Debug, Clone)]
pub struct SavedMacro {
    pub template: Arc<Template>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SavedMacroId(Uuid);

impl SavedMacroId {
    const PREFIX: &'static str = "macro-";

    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn to_value(self) -> Value {
        let string = format!("{}{}", Self::PREFIX, self.0);
        string.to_value()
    }
}

impl FromStr for SavedMacroId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let err = || Error::with_msg("Invalid macro id");
        if !s.starts_with(Self::PREFIX) {
            return Err(err());
        }
        let uuid = Uuid::parse_str(&s[Self::PREFIX.len()..]).map_err(|_| err())?;
        Ok(Self(uuid))
    }
}

#[derive(Debug, Default)]
pub struct SavedMacroMap {
    saved: AHashMap<SavedMacroId, SavedMacro>,
}

impl SavedMacroMap {
    fn save(&mut self, id: SavedMacroId, r#macro: SavedMacro) {
        self.saved.insert(id, r#macro);
    }

    pub fn get(&self, id: SavedMacroId) -> Option<SavedMacro> {
        self.saved.get(&id).cloned()
    }
}
