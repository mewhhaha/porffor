//! Shared native code with fresh per-P data and unchanged calling Store/Realm.
use super::*;
use crate::wasm_native_source_compiler::{CallbackTrace, NativeSourceCompilerContext, Phase};
use scriptw_aot_wasm::{NativeProgramTemplate, NATIVE_PROGRAM_DATA_MODULE};
use wasmtime::{
    ArrayRef, ArrayRefPre, AsContextMut, Func, HeapType, Instance, Mutability, StorageType,
    StructType, Val, ValType,
};
mod bindings;
mod cache;
pub(super) use bindings::NativeArrayConstructorCache;

mod initializer;
mod instance;
pub(super) use initializer::arguments as initializer_arguments;
pub(super) use instance::instantiate;
