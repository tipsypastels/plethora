use crate::{db::Db, styles::Styles};

pub trait AsApp: Clone + Send + Sync + 'static {
    fn as_db(&self) -> &Db;
    fn as_styles(&self) -> &Styles;
    // fn as_themes(&self) -> &Themes

    fn base_page_title(&self) -> Option<&str> {
        None
    }
}
