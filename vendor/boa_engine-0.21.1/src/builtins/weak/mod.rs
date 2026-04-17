//! Boa's implementation of ECMAScript's `WeakRef` object.

use boa_gc::{Finalize, Trace, WeakGc};
use boa_macros::JsData;

use crate::{
    JsValue,
    builtins::symbol::Symbol,
    object::{ErasedVTableObject, JsObject},
    symbol::JsSymbol,
};

mod finalization_registry;
mod weak_ref;

pub(crate) use finalization_registry::FinalizationRegistry;
pub(crate) use weak_ref::WeakRef;

#[derive(Clone, Trace, Finalize, JsData)]
pub(crate) enum WeakHeldValue {
    Object(WeakGc<ErasedVTableObject>),
    Symbol(JsSymbol),
}

impl WeakHeldValue {
    pub(crate) fn from_value(value: &JsValue) -> Option<Self> {
        if let Some(object) = value.as_object() {
            return Some(Self::Object(WeakGc::new(object.inner())));
        }
        value.as_symbol().and_then(|symbol| {
            is_symbol_weakly_holdable(&symbol).then_some(Self::Symbol(symbol))
        })
    }

    pub(crate) fn matches(&self, value: &JsValue) -> bool {
        match self {
            Self::Object(weak) => value.as_object().is_some_and(|object| {
                weak.upgrade()
                    .map(|target| JsObject::equals(&JsObject::from(target), &object))
                    .unwrap_or(false)
            }),
            Self::Symbol(symbol) => value.as_symbol().is_some_and(|candidate| candidate == *symbol),
        }
    }

    pub(crate) fn value(&self) -> JsValue {
        match self {
            Self::Object(weak) => weak
                .upgrade()
                .map(|object| JsValue::from(JsObject::from(object)))
                .unwrap_or_else(JsValue::undefined),
            Self::Symbol(symbol) => symbol.clone().into(),
        }
    }
}

pub(crate) fn is_symbol_weakly_holdable(symbol: &JsSymbol) -> bool {
    Symbol::registered_key(symbol).is_none()
}

pub(crate) fn can_be_held_weakly(value: &JsValue) -> bool {
    value.as_object().is_some() || value.as_symbol().is_some_and(|s| is_symbol_weakly_holdable(&s))
}
