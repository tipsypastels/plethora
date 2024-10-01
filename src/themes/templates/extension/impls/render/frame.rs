use kstring::{KString, KStringCow, KStringRef};
use liquid_core::{
    model::{find, try_find, ScalarCow},
    runtime::{PartialStore, Registers},
    Error, Object, ObjectView, Result, Runtime, Value, ValueCow, ValueView,
};
use std::{cell::RefCell, collections::BTreeSet, mem};

const NOT_SANDBOXED_VARS: &[&str] = &["current_user", "current_session", "current_theme"];

/// A `SandboxedStackFrame`, except it doesn't sandbox registers and
/// certain global context values.
pub struct MostlySandboxedStackFrame<P, O> {
    parent: P,
    data: O,
}

impl<P: Runtime, O: ObjectView> MostlySandboxedStackFrame<P, O> {
    pub fn new(parent: P, data: O) -> Self {
        Self { parent, data }
    }

    fn try_get_not_sandboxed(&self, s: &str, path: &[ScalarCow<'_>]) -> Option<ValueCow<'_>> {
        if NOT_SANDBOXED_VARS.iter().any(|&v| v == s) {
            self.parent.try_get(path)
        } else {
            None
        }
    }
}

impl<P: Runtime, O: ObjectView> Runtime for MostlySandboxedStackFrame<P, O> {
    fn partials(&self) -> &dyn PartialStore {
        self.parent.partials()
    }

    fn name(&self) -> Option<KStringRef<'_>> {
        self.parent.name()
    }

    fn roots(&self) -> BTreeSet<KStringCow<'_>> {
        unimplemented!("MostlySandboxedStackFrame::roots() is unused")
    }

    fn try_get(&self, path: &[ScalarCow<'_>]) -> Option<ValueCow<'_>> {
        let key = path.first()?.to_kstr();
        self.data
            .get(key.as_str())
            .and_then(|_| try_find(self.data.as_value(), path))
            .or_else(|| self.try_get_not_sandboxed(key.as_str(), path))
    }

    fn get(&self, path: &[ScalarCow<'_>]) -> Result<ValueCow<'_>> {
        let key = path.first().unwrap().to_kstr();
        self.data
            .get(key.as_str())
            .and_then(|_| try_find(self.data.as_value(), path))
            .map(|v| v.into_owned().into())
            .or_else(|| self.try_get_not_sandboxed(key.as_str(), path))
            .ok_or_else(|| Error::with_msg("Unknown variable").context("requested variable", key))
    }

    fn set_global(&self, name: KString, val: Value) -> Option<Value> {
        self.parent.set_global(name, val)
    }

    fn set_index(&self, name: KString, val: Value) -> Option<Value> {
        self.parent.set_index(name, val)
    }

    fn get_index<'a>(&'a self, name: &str) -> Option<ValueCow<'a>> {
        self.parent.get_index(name)
    }

    fn registers(&self) -> &Registers {
        self.parent.registers()
    }
}

type Witnessed<'a> = (KStringCow<'a>, ValueCow<'a>);

/// A stack frame that does not propagate set globals up to the parent,
/// but instead keeps them locally and allows a separate API for accessing
/// them. This is because we don't want the captures inside a
/// `contain` to become variables outside.
///
/// A lot of this is copied from `GlobalFrame`, which does similar things,
/// but has no public way to access its local variables.
pub struct WitnessFrame<P> {
    parent: P,
    data: RefCell<Object>,
}

impl<P: Runtime> WitnessFrame<P> {
    pub fn new(parent: P) -> Self {
        Self {
            parent,
            data: Default::default(),
        }
    }

    pub fn witnessed(&self) -> impl Iterator<Item = Witnessed<'_>> + '_ {
        let data = mem::take(&mut *self.data.borrow_mut());
        data.into_iter().map(|(k, v)| (k.into(), v.into()))
    }
}

impl<P: Runtime> Runtime for WitnessFrame<P> {
    fn partials(&self) -> &dyn PartialStore {
        self.parent.partials()
    }

    fn name(&self) -> Option<KStringRef<'_>> {
        self.parent.name()
    }

    fn roots(&self) -> BTreeSet<KStringCow<'_>> {
        let mut roots = self.parent.roots();
        roots.extend(self.data.borrow().keys().map(|k| k.clone().into()));
        roots
    }

    fn try_get(&self, path: &[ScalarCow<'_>]) -> Option<ValueCow<'_>> {
        let key = path.first()?;
        let key = key.to_kstr();
        let data = self.data.borrow();
        if data.contains_key(key.as_str()) {
            try_find(data.as_value(), path).map(|v| v.into_owned().into())
        } else {
            self.parent.try_get(path)
        }
    }

    fn get(&self, path: &[ScalarCow<'_>]) -> Result<ValueCow<'_>> {
        let key = path.first().unwrap().to_kstr();
        let data = self.data.borrow();
        if data.contains_key(key.as_str()) {
            find(data.as_value(), path).map(|v| v.into_owned().into())
        } else {
            self.parent.get(path)
        }
    }

    fn set_global(&self, name: KString, val: Value) -> Option<Value> {
        let mut data = self.data.borrow_mut();
        data.insert(name, val)
    }

    fn set_index(&self, name: KString, val: Value) -> Option<Value> {
        self.parent.set_index(name, val)
    }

    fn get_index<'a>(&'a self, name: &str) -> Option<ValueCow<'a>> {
        self.parent.get_index(name)
    }

    fn registers(&self) -> &Registers {
        self.parent.registers()
    }
}
