use super::l;
use ahash::AHashMap;
use kstring::{KString, KStringCow};
use l::ValueView as _;

pub struct Args<'a> {
    iter: l::TagTokenIter<'a>,
}

impl<'a> Args<'a> {
    pub(super) fn new(iter: l::TagTokenIter<'a>) -> Self {
        Self { iter }
    }

    pub fn identifier(&mut self) -> l::Result<KString> {
        Ok(self
            .iter
            .expect_next("Identifier expected.")?
            .expect_identifier()
            .into_result()?
            .to_kstr()
            .into_owned())
    }

    pub fn expression(&mut self) -> l::Result<l::Expression> {
        self.iter
            .expect_next("Expression expected.")?
            .expect_value()
            .into_result()
    }

    pub fn literal(&mut self) -> l::Result<l::Value> {
        self.iter
            .expect_next("Literal expected.")?
            .expect_literal()
            .into_result()
    }

    pub fn string_literal(&mut self) -> l::Result<KString> {
        Ok(self
            .literal()?
            .into_scalar()
            .ok_or_else(|| l::Error::with_msg("String expected."))?
            .into_string())
    }

    pub fn filter_chain(&mut self, language: &l::Language) -> l::Result<l::parser::FilterChain> {
        self.iter
            .expect_next("FilterChain expected")?
            .expect_filter_chain(language)
            .into_result()
    }

    pub fn kwargs(&mut self) -> l::Result<Kwargs> {
        let mut map = AHashMap::new();

        loop {
            let key = self.identifier()?;
            self.exact("Colon expected.", ":")?;

            let value = self.expression()?;
            map.insert(key, value);

            if self.comma().is_err() {
                break;
            }
        }

        Ok(Kwargs { map })
    }

    pub fn comma(&mut self) -> l::Result<()> {
        self.exact("Comma expected.", ",")
    }

    pub fn exact(&mut self, msg: &str, token: &str) -> l::Result<()> {
        self.iter
            .expect_next(msg)?
            .expect_str(token)
            .into_result()?;
        Ok(())
    }

    pub fn empty(mut self) -> l::Result<()> {
        self.iter.expect_nothing()
    }
}

pub struct Body<'a: 'b, 'b> {
    block: l::TagBlock<'a, 'b>,
}

impl<'a: 'b, 'b> Body<'a, 'b> {
    pub(super) fn new(block: l::TagBlock<'a, 'b>) -> Self {
        Self { block }
    }

    pub fn template(mut self, language: &l::Language) -> l::Result<l::Template> {
        Ok(l::Template::new(self.block.parse_all(language)?))
    }
}

pub type EvaluatedKwargs<'a> = ahash::HashMap<KStringCow<'a>, l::ValueCow<'a>>;

#[derive(Default)]
pub struct Kwargs {
    map: AHashMap<KString, l::Expression>,
}

impl Kwargs {
    pub fn evaluate<'a>(&'a self, runtime: &'a dyn l::Runtime) -> l::Result<EvaluatedKwargs<'a>> {
        self.map
            .iter()
            .map(|(k, v)| Ok((k.as_ref().into(), v.evaluate(runtime)?)))
            .collect()
    }
}
