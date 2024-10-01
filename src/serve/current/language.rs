use std::marker::PhantomData;

// TODO
#[derive(Debug)]
pub struct CurrentLanguageState<C> {
    _ident: PhantomData<C>,
}

impl<C> CurrentLanguageState<C> {
    pub(super) fn new() -> Self {
        Self {
            _ident: PhantomData,
        }
    }
}

impl<C> Clone for CurrentLanguageState<C> {
    fn clone(&self) -> Self {
        Self {
            _ident: self._ident,
        }
    }
}

#[derive(Debug)]
pub struct CurrentLanguage<C> {
    _ident: PhantomData<C>,
}

impl<C> Clone for CurrentLanguage<C> {
    fn clone(&self) -> Self {
        Self {
            _ident: self._ident,
        }
    }
}
