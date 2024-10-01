use super::{DynExtra, Extra};
use crate::{db::Db, styles::Styles};
use anyhow::Result;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct App {
    pub db: Db,
    pub(crate) extra: DynExtra,
    pub styles: Styles,
    // pub themes: Themes,
}

#[derive(Debug)]
pub struct AppInit<E> {
    pub db: Db,
    pub extra: Arc<E>,
}

impl App {
    pub async fn new<E: Extra>(init: AppInit<E>) -> Result<Self> {
        let AppInit { db, extra } = init;
        let extra = DynExtra::new(extra);

        let styles = Styles::new().await?;
        // let themes = Themes::new(styles.clone());

        Ok(Self {
            db,
            extra,
            styles,
            // themes,
        })
    }
}
