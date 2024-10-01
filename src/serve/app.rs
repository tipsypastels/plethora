use crate::{db::Db, styles::Styles, themes::Themes};

pub trait AsApp: Clone + Send + Sync + 'static {
    fn as_db(&self) -> &Db;
    fn as_styles(&self) -> &Styles;
    fn as_themes(&self) -> &Themes;

    fn base_page_title(&self) -> Option<&str> {
        None
    }

    fn default_theme_slug(&self) -> Option<&str> {
        None
    }
}
