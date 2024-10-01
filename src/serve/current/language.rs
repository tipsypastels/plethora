use std::marker::PhantomData;

// TODO
#[derive(Debug)]
pub struct CurrentLanguageState<C> {
    _cur: PhantomData<C>,
}

impl<C> CurrentLanguageState<C> {
    pub(super) fn new() -> Self {
        Self { _cur: PhantomData }
    }
}

impl<C> Clone for CurrentLanguageState<C> {
    fn clone(&self) -> Self {
        Self { _cur: self._cur }
    }
}

#[derive(Debug)]
pub struct CurrentLanguage<C> {
    _cur: PhantomData<C>,
}

impl<C> Clone for CurrentLanguage<C> {
    fn clone(&self) -> Self {
        Self { _cur: self._cur }
    }
}
