use boa_gc::{Finalize, Trace};

use crate::JsValue;

use super::PoisonableEnvironment;

#[derive(Debug, Trace, Finalize)]
pub(crate) struct LexicalEnvironment {
    inner: PoisonableEnvironment,
}

impl LexicalEnvironment {
    /// Creates a new `LexicalEnvironment`.
    pub(crate) fn new(bindings: u32, poisoned: bool, with: bool) -> Self {
        Self {
            inner: PoisonableEnvironment::new(bindings, poisoned, with),
        }
    }

    /// Gets the `poisonable_environment` of this lexical environment.
    pub(crate) const fn poisonable_environment(&self) -> &PoisonableEnvironment {
        &self.inner
    }

    /// Gets the binding value from the environment by it's index.
    ///
    /// # Panics
    ///
    /// Panics if the binding value is out of range or not initialized.
    #[track_caller]
    pub(crate) fn get(&self, index: u32) -> Option<JsValue> {
        self.inner.get(index)
    }

    /// Sets the binding value from the environment by index.
    ///
    /// # Panics
    ///
    /// Panics if the binding value is out of range.
    #[track_caller]
    pub(crate) fn set(&self, index: u32, value: JsValue) {
        self.inner.set(index, value);
    }

    /// Returns whether the given binding index exists.
    #[track_caller]
    pub(crate) fn has_binding_index(&self, index: u32) -> bool {
        self.inner.has_binding_index(index)
    }
}
