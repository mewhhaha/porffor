#[path = "../lila-engine/src/cache.rs"]
#[allow(dead_code)]
mod cache;
#[path = "../lila-engine/compiler_fingerprint.rs"]
mod compiler_fingerprint;
#[path = "../lila-engine/src/embedded_runtime/build.rs"]
mod native_runtime_build;
#[path = "build/objects.rs"]
mod objects;
#[path = "../lila-engine/src/wasmtime_config.rs"]
#[allow(dead_code)]
mod wasmtime_config;
#[path = "../lila-engine/src/wasmtime_policy.rs"]
#[allow(dead_code)]
mod wasmtime_policy;

use std::path::PathBuf;

fn main() {
    for variable in [
        "LILA_CACHE_DIR",
        "LILA_CACHE_LIMIT_BYTES",
        "LILA_FUNCTION_CACHE_LIMIT_BYTES",
        "LILA_MODULE_CACHE_LIMIT_BYTES",
        "LILA_BUILD_MODULE_CACHE_DIR",
        "XDG_CACHE_HOME",
        "HOME",
    ] {
        println!("cargo:rerun-if-env-changed={variable}");
    }
    let manifest = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let workspace = manifest
        .join("../..")
        .canonicalize()
        .expect("Lila workspace root should resolve");
    for input in compiler_fingerprint::compiler_rerun_inputs(&workspace)
        .expect("production compiler rerun inputs should resolve")
    {
        println!("cargo:rerun-if-changed={}", input.display());
    }
    let started = std::time::Instant::now();
    let fingerprint = compiler_fingerprint::fingerprint(&workspace)
        .unwrap_or_else(|error| panic!("failed to fingerprint Lila compiler inputs: {error}"));
    let native_fingerprint = compiler_fingerprint::native_fingerprint(&workspace)
        .expect("native compiler dependency inputs are fully fingerprinted");
    println!(
        "build runtime compiler fingerprints: {:?}",
        started.elapsed()
    );
    let out = PathBuf::from(std::env::var_os("OUT_DIR").expect("Cargo build output directory"));
    let target = std::env::var("TARGET").expect("Cargo target triple");
    native_runtime_build::write_bundle(
        &out,
        &fingerprint,
        &native_fingerprint,
        &target,
        &std::env::var("HOST").expect("Cargo build host triple"),
        &std::env::var("CARGO_CFG_TARGET_ARCH").expect("Cargo target architecture"),
        &std::env::var("CARGO_CFG_TARGET_ENDIAN").expect("Cargo target endian"),
    );
    let started = std::time::Instant::now();
    objects::embed(&out, &target);
    println!("build runtime object packaging: {:?}", started.elapsed());
}
