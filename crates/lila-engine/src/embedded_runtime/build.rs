//! Cargo producer: checked R/package/manifest and bounded exact-Wasm native compilation.

#[path = "compile.rs"]
mod compile;
#[path = "manifest.rs"]
mod manifest;
#[path = "publication.rs"]
mod publication;
use crate::wasmtime_config::{base_config, CONFIGURATION_SCHEMA, ENGINE_WORKER_STACK_SIZE};
use compile::RuntimeImageCompiler;
use sha2::{Digest, Sha256};
use std::path::Path;
use std::sync::Arc;
use wasmtime::WasmBacktraceDetails;

pub(crate) fn write_bundle(
    out: &Path,
    compiler_source: &str,
    native_identity: &str,
    target: &str,
    host: &str,
    target_arch: &str,
    endian: &str,
) {
    // The current raw Intl admission requires little-endian data. Other targets
    // keep the ordinary runtime path and its capability checks, without a bundle.
    if endian != "little" || !matches!(target_arch, "x86_64" | "aarch64" | "riscv64") {
        println!(
            "cargo:warning=embedded runtime unavailable for {target}; preserving normal runtime compilation"
        );
        publication::publish(out, &[], &[], &[]);
        return;
    }
    let source =
        manifest::source_identity(compiler_source).expect("compiler source fingerprint is SHA-256");
    std::thread::scope(|scope| {
        std::thread::Builder::new()
            .name("lila-build-native-runtime".to_owned())
            .stack_size(ENGINE_WORKER_STACK_SIZE)
            .spawn_scoped(scope, || {
                let selection =
                    lila_intl::IntlDataSelection::new(lila_intl::IntlCompilationProfile::Minimal);
                println!("build runtime native dependency identity: {native_identity}");
                let started = std::time::Instant::now();
                let runtime =
                    lila_aot_wasm::build_runtime_artifact_with_source_compilation_profile(
                        &selection,
                        &source,
                        lila_aot_wasm::RuntimeSourceCompilationProfile::NativeHost,
                    )
                    .expect("the compiler emits and admits its exact build runtime");
                println!(
                    "build runtime fresh emission/admission: {:?}",
                    started.elapsed()
                );
                let started = std::time::Instant::now();
                // Optimize immutable R once; ordinary P compilation retains Fast.
                let mut config = base_config(
                    manifest::COMPILATION_MODES.runtime,
                    WasmBacktraceDetails::Disable,
                );
                // Ordinary P retains base_config's SinglePass allocator and Fast mode.
                config.cranelift_regalloc_algorithm(
                    manifest::COMPILATION_MODES.register_allocation.algorithm(),
                );
                // Explicit target also disables build-host CPU feature inference.
                config
                    .target(target)
                    .expect("supported target has a native compiler");
                config.parallel_compilation(false);
                // Cranelift's bounded function store also warms module misses.
                // Native caches remain independent of current source identity in the package.
                let function_cache = crate::cache::FunctionCache::new(
                    crate::cache::function_cache_directory(),
                    crate::cache::function_cache_limit_bytes(),
                )
                .and_then(|cache| {
                    crate::cache::NativeFunctionCache::new(Arc::new(cache), native_identity)
                })
                .map(Arc::new)
                .map_err(|error| {
                    println!("cargo:warning=build runtime function cache unavailable: {error}");
                })
                .ok();
                if let Some(cache) = &function_cache {
                    config
                        .enable_incremental_compilation(cache.clone())
                        .expect("Cranelift supports the shared function cache");
                }
                // Module::new requires a native-host mapping. Cross-target
                // builds preserve precompile_module and never open this cache.
                let module_cache = if host == target {
                    crate::cache::module_cache()
                        .map_err(|error| {
                            println!(
                                "cargo:warning=build runtime module cache unavailable: {error}"
                            );
                        })
                        .ok()
                } else {
                    None
                };
                let compiler = RuntimeImageCompiler::new(config, module_cache, native_identity)
                    .expect("build engine has the required product configuration and capabilities");
                println!("build runtime native setup: {:?}", started.elapsed());
                let started = std::time::Instant::now();
                let native = compiler
                    .compile(&runtime)
                    .expect("exact checked runtime precompiles for the Cargo target");
                println!("build runtime native compilation: {:?}", started.elapsed());
                if let Some((hits, misses)) = compiler.module_cache_counters() {
                    println!("build runtime module cache: {hits} hits, {misses} misses");
                }
                if let Some(cache) = function_cache {
                    let (hits, misses) = cache.counters();
                    println!("build runtime function cache: {hits} hits, {misses} misses");
                }
                let started = std::time::Instant::now();
                let manifest = manifest::Manifest {
                    source,
                    target: target.to_owned(),
                    configuration: CONFIGURATION_SCHEMA,
                    modes: manifest::COMPILATION_MODES,
                    runtime_key: *runtime.key().as_bytes(),
                    package_digest: Sha256::digest(runtime.package_bytes()).into(),
                    native_digest: Sha256::digest(&native).into(),
                };
                let encoded = manifest.encode();
                assert_eq!(manifest::Manifest::decode(&encoded), Some(manifest));
                publication::publish(out, runtime.package_bytes(), &native, &encoded);
                println!(
                    "build runtime manifest publication: {:?}",
                    started.elapsed()
                );
            })
            .expect("bounded build-runtime compiler thread starts")
            .join()
            .expect("build runtime compiler thread completes");
    });
}
