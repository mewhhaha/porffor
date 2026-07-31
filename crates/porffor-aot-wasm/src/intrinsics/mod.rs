//! Per-family realm bootstrap and property-descriptor installation.
//!
//! This is the `intrinsics/` boundary named by T02. It exists to break up
//! `builtins/bootstrap.rs::init_builtin_constructor_object`, a single function
//! spanning ~4,760 lines whose final arm is a 485-variant no-op or-pattern that
//! every new builtin has to be appended to. That function is the worst merge
//! point in the backend: two lanes adding builtins to unrelated families still
//! collide inside it.
//!
//! Each family owns one file here. An arm moves across **verbatim** — the
//! shared preamble values are re-bound by destructuring [`IntrinsicInstall`]
//! under their original names, so a moved body needs no textual edits and the
//! move is provable byte-for-byte with `tests/emit_golden.rs`.
//!
//! Property installation order is observable through `Object.keys`, so arms
//! must keep their internal ordering and the dispatch in `bootstrap.rs` must
//! keep calling them in the same order it always did.

use super::*;

pub(crate) mod array;
pub(crate) mod binary_data;
pub(crate) mod collections;
pub(crate) mod date;
pub(crate) mod errors;
pub(crate) mod function;
pub(crate) mod iterator;
pub(crate) mod numeric;
pub(crate) mod object;
pub(crate) mod promise;
pub(crate) mod proxy;
pub(crate) mod regexp;
pub(crate) mod string;
pub(crate) mod symbol;
pub(crate) mod temporal;

/// The values `init_builtin_constructor_object` computes once, before
/// dispatching to a family installer.
///
/// `meta` borrows through `FunctionBuilder::functions`, which is a `&'a`
/// reference rather than owned state, so it is independent of the `&mut self`
/// that the installers need. The original function already relies on this when
/// it passes `meta` to `&mut self` methods.
#[derive(Clone, Copy)]
pub(crate) struct IntrinsicInstall<'m> {
    /// Carried because multi-variant arms such as
    /// `ArrayBufferConstructor | SharedArrayBufferConstructor` branch on it with
    /// `matches!(builtin, ...)`; keeping it here left those bodies verbatim.
    pub(crate) builtin: StandardBuiltinId,
    pub(crate) meta: &'m WasmFunctionMeta,
    pub(crate) prototype_global_index: u32,
    pub(crate) constructor_global_index: u32,
    pub(crate) object_local: u32,
    pub(crate) key_local: u32,
    pub(crate) payload_local: u32,
    pub(crate) tag_local: u32,
    pub(crate) prototype_object_local: u32,
}
