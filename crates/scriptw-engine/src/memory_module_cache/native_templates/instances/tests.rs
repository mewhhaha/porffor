use super::*;
use crate::{WasmNativeCompilationMode, WasmtimeEngine};
#[test]
fn store_instance_reuse_checks_exact_code_and_execution_authority_within_original_limits() {
    let engine = WasmtimeEngine::default();
    let other_engine = WasmtimeEngine::default();
    let code = b"\0asm\x01\0\0\0";
    let module = WasmtimeModule::new(&engine, code).unwrap();
    let mut store = wasmtime::Store::new(&engine, ());
    let instance = wasmtime::Instance::new(&mut store, &module, &[]).unwrap();
    let authority = Arc::new(());
    let other_authority = Arc::new(());
    let key = WasmModuleMemoryCacheKey {
        native_compilation_mode: WasmNativeCompilationMode::Fast,
        wasm_sha256: [7; 32],
    };
    let bytes = module_image_bytes(&module).unwrap() + code.len();
    let mut cache = MemoryNativeInstanceCache {
        entries: VecDeque::new(),
        retained_bytes: 0,
        limits: MemoryModuleCacheLimits {
            entries: NonZeroUsize::new(1).unwrap(),
            image_bytes: NonZeroUsize::new(bytes).unwrap(),
        },
    };
    cache.retain(key, code, module.clone(), instance, Arc::clone(&authority));
    assert!(cache.get(&key, code, &engine, &authority).is_some());
    assert!(cache
        .get(
            &key,
            b"different code with the same supplied digest",
            &engine,
            &authority
        )
        .is_none());
    assert!(cache.get(&key, code, &other_engine, &authority).is_none());
    assert!(cache.get(&key, code, &engine, &other_authority).is_none());
    assert_eq!(cache.entries.len(), 1);
    assert_eq!(cache.retained_bytes, bytes);
    cache.retain(key, code, module, instance, other_authority.clone());
    assert!(cache.get(&key, code, &engine, &authority).is_none());
    assert!(cache.get(&key, code, &engine, &other_authority).is_some());
    assert_eq!(cache.entries.len(), 1);
    assert_eq!(cache.retained_bytes, bytes);
}
