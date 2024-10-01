use super::App;
use axum::{extract::FromRequestParts, http::request::Parts};
use std::{
    any::{self, Any},
    convert::Infallible,
    mem,
    sync::Arc,
};

pub trait Extra: Any + Send + Sync {
    fn base_page_title(&self) -> Option<&str>;
}

#[derive(Debug)]
pub struct AppExtra<E>(pub Arc<E>);

#[axum::async_trait]
impl<E: Extra> FromRequestParts<App> for AppExtra<E> {
    type Rejection = Infallible;

    async fn from_request_parts(_: &mut Parts, app: &App) -> Result<Self, Infallible> {
        Ok(AppExtra(app.extra.downcast()))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct DynExtra {
    value: Arc<dyn Any + Send + Sync>,
    vtable: &'static DynExtraVtable,
}

#[derive(Debug, Clone)]
struct DynExtraVtable {
    base_page_title: fn(Arc<dyn Any + Send + Sync>) -> Option<&'static str>,
}

impl DynExtra {
    pub(super) fn new<E: Extra>(value: Arc<E>) -> Self {
        Self {
            value,
            vtable: &DynExtraVtable {
                base_page_title: |v| {
                    let x = v.downcast_ref::<E>().unwrap();
                    // Safety: `Extra::base_page_title` takes `self`
                    // and clones it, so we know we're alive, and the
                    // return type of that method has the same lifetime
                    // as self, so the static lifetime doesn't escape.
                    unsafe { mem::transmute(x.base_page_title()) }
                },
            },
        }
    }

    pub fn downcast<E: Extra>(&self) -> Arc<E> {
        self.value
            .clone()
            .downcast()
            .unwrap_or_else(|_| panic!("extra is not a {}", any::type_name::<E>()))
    }

    pub fn base_page_title(&self) -> Option<&str> {
        (self.vtable.base_page_title)(self.value.clone())
    }
}
