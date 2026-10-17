use super::*;
use crate::{WasmNativeCompilationMode, WasmtimeEngine};
#[test]
fn native_template_cache_requires_actual_engine_and_keeps_original_aggregate_limits() {
    let first = WasmtimeEngine::default();
    let second = WasmtimeEngine::default();
    let a = WasmtimeModule::new(&first, b"\0asm\x01\0\0\0").unwrap();
    let b = WasmtimeModule::new(&second, b"\0asm\x01\0\0\0").unwrap();
    let bytes = module_image_bytes(&a)
        .unwrap()
        .max(module_image_bytes(&b).unwrap());
    let limits = MemoryModuleCacheLimits {
        entries: NonZeroUsize::new(2).unwrap(),
        image_bytes: NonZeroUsize::new(bytes * 2).unwrap(),
    };
    let mut cache = MemoryWasmModuleCache::new(limits);
    let key = WasmModuleMemoryCacheKey {
        native_compilation_mode: WasmNativeCompilationMode::Fast,
        wasm_sha256: [7; 32],
    };
    cache.retain_compiled_for_engine(key, a.clone());
    assert!(cache.get_for_engine(&key, &first).is_some());
    assert!(cache.get_for_engine(&key, &second).is_none());
    let result = cache.retain_compiled_for_engine(key, b.clone());
    assert!(WasmtimeModule::same(&result, &b));
    assert!(!WasmtimeModule::same(&result, &a));
    assert_eq!(cache.entries.len(), 1);
    assert_eq!(cache.retained_image_bytes, module_image_bytes(&b).unwrap());
    assert!(cache.get_for_engine(&key, &first).is_none());
    assert!(cache.get_for_engine(&key, &second).is_some());
}
