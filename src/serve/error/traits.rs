use super::{Renderer, ServeError, ServeResult};
use anyhow::Error;
use std::future::Future;

pub trait Re<T> {
    fn re(self, re: &impl Renderer) -> ServeResult<T>;
}

impl<T, E> Re<T> for Result<T, E>
where
    E: Into<Error>,
{
    fn re(self, re: &impl Renderer) -> ServeResult<T> {
        match self {
            Ok(value) => Ok(value),
            Err(error) => Err(ServeError::new(re.clone(), error.into())),
        }
    }
}

pub trait ReFuture<T> {
    async fn re(self, re: &impl Renderer) -> ServeResult<T>;
}

impl<F, T, E> ReFuture<T> for F
where
    F: Future<Output = Result<T, E>>,
    E: Into<Error>,
{
    async fn re(self, re: &impl Renderer) -> ServeResult<T> {
        self.await.re(re)
    }
}

pub trait OrNotFound<T> {
    fn or_not_found(self, re: &impl Renderer) -> ServeResult<T>;
}

impl<T> OrNotFound<T> for Option<T> {
    fn or_not_found(self, re: &impl Renderer) -> ServeResult<T> {
        match self {
            Some(value) => Ok(value),
            None => Err(ServeError::not_found(re.clone())),
        }
    }
}
