use super::{ingest, Themes};
use crate::styles::Styles;
use anyhow::Result;
use dashmap::DashMap;
use std::sync::Arc;

#[derive(Debug)]
pub struct ThemesBuilder {
    styles: Styles,
    #[cfg(feature = "baked-themes")]
    baked: Option<include_dir::Dir<'static>>,
}

impl ThemesBuilder {
    pub(super) fn new(styles: Styles) -> Self {
        Self {
            styles,
            #[cfg(feature = "baked-themes")]
            baked: None,
        }
    }

    #[cfg(feature = "baked-themes")]
    pub fn baked(self, baked: include_dir::Dir<'static>) -> Self {
        Self {
            styles: self.styles,
            baked: Some(baked),
        }
    }

    pub async fn build(self) -> Result<Themes> {
        let map = Arc::new(DashMap::new());
        let styles = self.styles;
        let themes = Themes { map, styles };

        #[cfg(feature = "baked-themes")]
        if let Some(baked) = self.baked {
            themes.ingest_many::<ingest::Baked>(baked).await?;
        }

        themes.ingest_many::<ingest::Files>(()).await?;

        Ok(themes)
    }
}
