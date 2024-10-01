use super::extension::extension;
use anyhow::Result;
use liquid::partials::PartialCompiler;
use liquid_core::{runtime, Language, ParseBlock, ParseFilter, ParseTag};
use liquid_lib::stdlib;
use std::sync::Arc;

pub struct Parser {
    pub language: Arc<Language>,
    pub partials: Arc<dyn runtime::PartialStore + Send + Sync>,
}

impl Parser {
    pub fn new(compiler: impl PartialCompiler) -> Result<Self> {
        let mut language = Language::empty();

        stdlib(&mut language);
        extension(&mut language);

        let language = Arc::new(language);
        let partials = compiler.compile(language.clone())?.into();

        Ok(Self { language, partials })
    }
}

fn stdlib(language: &mut Language) {
    language
        .tag(stdlib::AssignTag)
        .tag(stdlib::BreakTag)
        .tag(stdlib::ContinueTag)
        .tag(stdlib::CycleTag)
        // .tag(stdlib::IncludeTag) replaced with custom impl.
        .tag(stdlib::IncrementTag)
        .tag(stdlib::DecrementTag)
        // .tag(stdlib::RenderTag) replaced with custom impl.
        .block(stdlib::RawBlock)
        .block(stdlib::IfBlock)
        .block(stdlib::UnlessBlock)
        .block(stdlib::IfChangedBlock)
        .block(stdlib::ForBlock)
        .block(stdlib::TableRowBlock)
        .block(stdlib::CommentBlock)
        .block(stdlib::CaptureBlock)
        .block(stdlib::CaseBlock)
        .filter(stdlib::Abs)
        .filter(stdlib::Append)
        .filter(stdlib::AtLeast)
        .filter(stdlib::AtMost)
        .filter(stdlib::Capitalize)
        .filter(stdlib::Ceil)
        .filter(stdlib::Compact)
        .filter(stdlib::Concat)
        .filter(stdlib::Date)
        .filter(stdlib::Default)
        .filter(stdlib::DividedBy)
        .filter(stdlib::Downcase)
        .filter(stdlib::Escape)
        .filter(stdlib::EscapeOnce)
        .filter(stdlib::First)
        .filter(stdlib::Floor)
        .filter(stdlib::Join)
        .filter(stdlib::Last)
        .filter(stdlib::Lstrip)
        .filter(stdlib::Map)
        .filter(stdlib::Minus)
        .filter(stdlib::Modulo)
        .filter(stdlib::NewlineToBr)
        .filter(stdlib::Plus)
        .filter(stdlib::Prepend)
        .filter(stdlib::Remove)
        .filter(stdlib::RemoveFirst)
        .filter(stdlib::Replace)
        .filter(stdlib::ReplaceFirst)
        .filter(stdlib::Reverse)
        .filter(stdlib::Round)
        .filter(stdlib::Rstrip)
        .filter(stdlib::Size)
        .filter(stdlib::Slice)
        .filter(stdlib::Sort)
        .filter(stdlib::SortNatural)
        .filter(stdlib::Split)
        .filter(stdlib::Strip)
        .filter(stdlib::StripHtml)
        .filter(stdlib::StripNewlines)
        .filter(stdlib::Times)
        .filter(stdlib::Truncate)
        .filter(stdlib::TruncateWords)
        .filter(stdlib::Uniq)
        .filter(stdlib::Upcase)
        .filter(stdlib::UrlDecode)
        .filter(stdlib::UrlEncode)
        .filter(stdlib::Where);
}

pub trait LanguageExt {
    fn tag(&mut self, tag: impl ParseTag + 'static) -> &mut Self;
    fn block(&mut self, block: impl ParseBlock + 'static) -> &mut Self;
    fn filter(&mut self, filter: impl ParseFilter + 'static) -> &mut Self;
}

impl LanguageExt for Language {
    fn tag(&mut self, tag: impl ParseTag + 'static) -> &mut Self {
        let name = tag.reflection().tag().into();
        self.tags.register(name, Box::new(tag));
        self
    }

    fn block(&mut self, block: impl ParseBlock + 'static) -> &mut Self {
        let name = block.reflection().start_tag().into();
        self.blocks.register(name, Box::new(block));
        self
    }

    fn filter(&mut self, filter: impl ParseFilter + 'static) -> &mut Self {
        let name = filter.reflection().name().into();
        self.filters.register(name, Box::new(filter));
        self
    }
}
