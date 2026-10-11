//! Actual copying-collector reachability, independent of JavaScript wrappers.
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use wasmtime::{
    AnyRef, AsContextMut, Collector, Config, Engine, ExternRef, FieldType, GcEphemeron, Mutability,
    RootScope, Rooted, StorageType, Store, StructRef, StructRefPre, StructType, Val, ValType,
    WeakAnyRef, I31,
};

fn heap() -> (Store<()>, StructRefPre) {
    let mut config = Config::new();
    config
        .wasm_gc(true)
        .wasm_function_references(true)
        .collector(Collector::Copying);
    let engine = Engine::new(&config).unwrap();
    let ty = StructType::new(
        &engine,
        [
            FieldType::new(Mutability::Var, StorageType::ValType(ValType::EQREF)),
            FieldType::new(Mutability::Const, StorageType::ValType(ValType::I32)),
        ],
    )
    .unwrap();
    let mut store = Store::new(&engine, ());
    let pre = StructRefPre::new(&mut store, ty);
    (store, pre)
}

fn object(mut store: impl AsContextMut<Data = ()>, pre: &StructRefPre, n: i32) -> Rooted<AnyRef> {
    StructRef::new(&mut store, pre, &[Val::AnyRef(None), Val::I32(n)])
        .unwrap()
        .to_anyref()
}

fn marker(mut store: impl AsContextMut<Data = ()>, reference: Rooted<AnyRef>) -> i32 {
    reference
        .as_struct(&store)
        .unwrap()
        .unwrap()
        .field(&mut store, 1)
        .unwrap()
        .unwrap_i32()
}

mod lifecycle;
mod reachability;
