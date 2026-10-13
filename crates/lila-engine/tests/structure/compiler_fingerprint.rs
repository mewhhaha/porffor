#[path = "../../compiler_fingerprint.rs"]
mod compiler_fingerprint;

use compiler_fingerprint::{
    fingerprint, native_fingerprint, COMPILER_INPUTS, NATIVE_COMPILER_INPUTS,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

struct Workspace(PathBuf);

impl Workspace {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "lila-fingerprint-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        let workspace = Self(root);
        for input in COMPILER_INPUTS {
            let path = Path::new(input);
            if path.extension().is_some() {
                workspace.write(input, "fixture");
            } else {
                fs::create_dir_all(workspace.0.join(input)).unwrap();
            }
        }
        for input in NATIVE_COMPILER_INPUTS {
            if Path::new(input).extension().is_some() && !workspace.0.join(input).exists() {
                workspace.write(input, "fixture");
            }
        }
        workspace
    }

    fn write(&self, path: &str, contents: &str) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }

    fn digest(&self) -> String {
        fingerprint(&self.0).unwrap()
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn every_previously_omitted_compiler_input_invalidates_the_cache() {
    for input in [
        "crates/lila-runtime/src/lib.rs",
        "crates/lila-runtime-image/Cargo.toml",
        "crates/lila-runtime-image/build.rs",
        "crates/lila-runtime-image/build/objects.rs",
        "crates/lila-runtime-image/src/lib.rs",
        "crates/lila-intl/src/lib.rs",
        "crates/lila-engine/src/wasmtime_policy.rs",
        "crates/lila-engine/src/wasmtime_config.rs",
        "crates/lila-engine/src/embedded_runtime/build.rs",
        "crates/lila-engine/src/embedded_runtime/compile.rs",
        "crates/lila-engine/src/embedded_runtime/compile_tests.rs",
        "crates/lila-engine/src/cache/native.rs",
        "crates/lila-engine/src/embedded_runtime/manifest.rs",
        "crates/lila-aot-wasm/src/runtime_artifact/build_package.rs",
        "vendor/boa_parser-0.21.1/src/lib.rs",
        "vendor/temporal_rs-0.1.0/src/lib.rs",
        "crates/lila-front/build.rs",
        "crates/lila-aot-wasm/src/embedded.json",
    ] {
        let workspace = Workspace::new();
        workspace.write(input, "before");
        let before = workspace.digest();
        workspace.write(input, "after");
        assert_ne!(before, workspace.digest(), "untracked input: {input}");
    }
}

#[test]
fn native_dependency_identity_invalidates_for_lockfile_features_profiles_and_patches() {
    for input in [
        "Cargo.lock",
        "Cargo.toml",
        "crates/lila-engine/Cargo.toml",
        "crates/lila-runtime-image/Cargo.toml",
        "vendor/wasmtime-47.0.0/src/compile/code_builder.rs",
        "vendor/wasmtime-internal-cache-47.0.0/src/lib.rs",
    ] {
        let workspace = Workspace::new();
        workspace.write(input, "before");
        let before = native_fingerprint(&workspace.0).unwrap();
        workspace.write(input, "after");
        assert_ne!(before, native_fingerprint(&workspace.0).unwrap(), "{input}");
    }
}

#[test]
fn native_dependency_identity_preserves_lila_source_invalidation_without_native_cache_churn() {
    let workspace = Workspace::new();
    workspace.write("crates/lila-ir/src/source.rs", "before");
    let native_before = native_fingerprint(&workspace.0).unwrap();
    let compiler_before = workspace.digest();
    workspace.write("crates/lila-ir/src/source.rs", "after");
    assert_eq!(native_before, native_fingerprint(&workspace.0).unwrap());
    assert_ne!(compiler_before, workspace.digest());
    fs::remove_file(workspace.0.join("Cargo.lock")).unwrap();
    assert!(native_fingerprint(&workspace.0).is_err());
}

#[test]
fn fingerprints_are_checkout_location_and_creation_order_independent() {
    let first = Workspace::new();
    let second = Workspace::new();
    first.write("vendor/fixture/z.rs", "z");
    first.write("vendor/fixture/a.rs", "a");
    second.write("vendor/fixture/a.rs", "a");
    second.write("vendor/fixture/z.rs", "z");
    assert_eq!(first.digest(), second.digest());
}

#[test]
fn adding_removing_or_renaming_an_input_changes_the_fingerprint() {
    let workspace = Workspace::new();
    let empty = workspace.digest();
    workspace.write("vendor/fixture/a.rs", "same bytes");
    let added = workspace.digest();
    assert_ne!(empty, added);
    fs::rename(
        workspace.0.join("vendor/fixture/a.rs"),
        workspace.0.join("vendor/fixture/b.rs"),
    )
    .unwrap();
    assert_ne!(added, workspace.digest());
    fs::remove_file(workspace.0.join("vendor/fixture/b.rs")).unwrap();
    assert_eq!(empty, workspace.digest());
}

#[test]
fn generated_build_outputs_do_not_invalidate_source_fingerprints() {
    let workspace = Workspace::new();
    let before = workspace.digest();
    workspace.write("vendor/fixture/target/debug/generated.rs", "generated");
    workspace.write("vendor/fixture/.git/index", "administrative state");
    assert_eq!(before, workspace.digest());
}

#[test]
fn missing_required_inputs_fail_closed() {
    let workspace = Workspace::new();
    fs::remove_file(workspace.0.join("Cargo.lock")).unwrap();
    assert_eq!(
        fingerprint(&workspace.0).unwrap_err().kind(),
        std::io::ErrorKind::NotFound
    );
}
