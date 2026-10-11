//! Cargo-only producer: native bytes come directly from Wasmtime precompilation
//! of the checked typed owner's exact raw R, never from a native cache file.

#[path = "manifest.rs"]
mod manifest;
use crate::wasmtime_config::{base_config, CONFIGURATION_SCHEMA, ENGINE_WORKER_STACK_SIZE};
use crate::wasmtime_policy::PRODUCT_WASMTIME_POLICY;
use sha2::{Digest, Sha256};
use std::path::Path;
use std::sync::Arc;
use wasmtime::{Engine, WasmBacktraceDetails};

pub(crate) fn write_bundle(
    out: &Path,
    compiler_source: &str,
    target: &str,
    target_arch: &str,
    endian: &str,
) {
    // The current raw Intl admission requires little-endian data. Other targets
    // keep the ordinary runtime path and its capability checks, without a bundle.
    if endian != "little" || !matches!(target_arch, "x86_64" | "aarch64" | "riscv64") {
        println!("cargo:warning=embedded runtime unavailable for {target}; preserving normal runtime compilation");
        publish(out, &[], &[], &[]);
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
                let runtime = lila_aot_wasm::build_runtime_artifact(&selection, &source)
                    .expect("the compiler emits and admits its exact build runtime");
                // R is immutable build output. Optimize it once without moving
                // ordinary P compilation off its existing Fast setting.
                let mut config = base_config(
                    manifest::COMPILATION_MODES.runtime,
                    WasmBacktraceDetails::Disable,
                );
                // Explicit target also disables build-host CPU feature inference.
                config
                    .target(target)
                    .expect("supported target has a native compiler");
                config.parallel_compilation(false);
                // Cranelift keys stencils by function, ISA and compiler flags.
                // Reuse that existing bounded store across source fingerprints;
                // the current raw R still goes through Wasmtime precompilation,
                // and the package below always binds the current full source.
                let function_cache = crate::cache::FunctionCache::new(
                    crate::cache::function_cache_directory(),
                    crate::cache::function_cache_limit_bytes(),
                )
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
                let engine = Engine::new(&config)
                    .expect("build runtime has the product Wasmtime configuration");
                PRODUCT_WASMTIME_POLICY
                    .verify_engine(&engine)
                    .expect("build engine has every required product capability");
                let started = std::time::Instant::now();
                let native = engine
                    .precompile_module(runtime.wasm())
                    .expect("exact checked runtime precompiles for the Cargo target");
                println!("build runtime precompile: {:?}", started.elapsed());
                if let Some(cache) = function_cache {
                    let (hits, misses) = cache.counters();
                    println!("build runtime function cache: {hits} hits, {misses} misses");
                }
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
                publish(out, runtime.package_bytes(), &native, &encoded);
            })
            .expect("bounded build-runtime compiler thread starts")
            .join()
            .expect("build runtime compiler thread completes");
    });
}

fn publish(out: &Path, package: &[u8], native: &[u8], manifest: &[u8]) {
    // Cargo admits this build script only after all three writes succeed; the
    // final manifest binds both complete immutable include_bytes inputs.
    for (name, bytes) in [
        ("lila-runtime-package.bin", package),
        ("lila-runtime-native.bin", native),
        ("lila-runtime-manifest.bin", manifest),
    ] {
        std::fs::write(out.join(name), bytes).expect("build runtime output writes completely");
    }
}
