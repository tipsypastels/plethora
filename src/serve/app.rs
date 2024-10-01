use crate::{db::Db, styles::Styles, themes::Themes};

pub trait Application: Clone + Send + Sync + 'static {
    fn db(&self) -> &Db;
    fn styles(&self) -> &Styles;
    fn themes(&self) -> &Themes;

    fn default_theme_slug(&self) -> &str;

    fn base_page_title(&self) -> Option<&str> {
        None
    }
}
