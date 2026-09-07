#[path = "../compiler_fingerprint.rs"]
mod compiler_fingerprint;

use compiler_fingerprint::{fingerprint, COMPILER_INPUTS};
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
        "crates/lila-intl/src/lib.rs",
        "crates/lila-engine/src/wasmtime_policy.rs",
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
