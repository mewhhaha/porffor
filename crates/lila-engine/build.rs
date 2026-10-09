mod compiler_fingerprint;
#[path = "src/embedded_runtime/build.rs"]
mod native_runtime_build;
#[path = "src/wasmtime_config.rs"]
#[allow(dead_code)] // Both runtime modes are defined in the shared configuration.
mod wasmtime_config;
#[path = "src/wasmtime_policy.rs"]
#[allow(dead_code)] // Runtime reporting methods share the same policy owner.
mod wasmtime_policy;

use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let manifest = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let workspace = manifest
        .join("../..")
        .canonicalize()
        .expect("Lila workspace root should resolve");
    for input in compiler_fingerprint::COMPILER_INPUTS {
        println!("cargo:rerun-if-changed={}", workspace.join(input).display());
    }
    let fingerprint = compiler_fingerprint::fingerprint(&workspace)
        .unwrap_or_else(|error| panic!("failed to fingerprint Lila compiler inputs: {error}"));
    println!("cargo:rustc-env=LILA_COMPILER_FINGERPRINT={fingerprint}");
    let target = std::env::var("TARGET").expect("Cargo target triple");
    println!("cargo:rustc-env=LILA_RUNTIME_BUILD_TARGET={target}");
    native_runtime_build::write_bundle(
        &PathBuf::from(std::env::var_os("OUT_DIR").expect("Cargo build output directory")),
        &fingerprint,
        &target,
        &std::env::var("CARGO_CFG_TARGET_ARCH").expect("Cargo target architecture"),
        &std::env::var("CARGO_CFG_TARGET_ENDIAN").expect("Cargo target endian"),
    );
    println!(
        "cargo:rustc-env=LILA_COMPILER_FINGERPRINT_SCHEME={}",
        compiler_fingerprint::FINGERPRINT_SCHEME
    );
    println!(
        "cargo:rustc-env=LILA_COMPILER_SOURCE_REVISION={}",
        source_revision(&workspace)
    );
}

fn git_output(workspace: &Path, arguments: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(workspace)
        .args(arguments)
        .output()
        .expect("a Git checkout needs readable Git build metadata");
    assert!(
        output.status.success(),
        "cannot read Git compiler build metadata: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("Git compiler build metadata must be UTF-8")
        .trim()
        .to_owned()
}

fn git_path(workspace: &Path, name: &str) -> PathBuf {
    workspace.join(git_output(workspace, &["rev-parse", "--git-path", name]))
}

fn source_revision(workspace: &Path) -> String {
    let administrative_entry = workspace.join(".git");
    if !administrative_entry.exists() {
        return "unversioned-archive".into();
    }
    if administrative_entry.is_file() {
        // A worktree's gitdir indirection can change without touching its HEAD.
        println!("cargo:rerun-if-changed={}", administrative_entry.display());
    }
    let commit = git_output(workspace, &["rev-parse", "--verify", "HEAD"]);
    assert!(
        matches!(commit.len(), 40 | 64)
            && commit
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)),
        "Git compiler commit has an invalid object identity"
    );
    let head = git_path(workspace, "HEAD");
    println!("cargo:rerun-if-changed={}", head.display());
    // Linked worktrees resolve shared refs through this file. Watching HEAD
    // and the currently resolved ref alone misses changes to that routing.
    let common_directory = head
        .parent()
        .expect("Git HEAD must have an administrative directory")
        .join("commondir");
    if common_directory.exists() {
        println!("cargo:rerun-if-changed={}", common_directory.display());
    }
    let contents = std::fs::read_to_string(&head).expect("Git HEAD must be readable");
    if let Some(reference) = contents.trim().strip_prefix("ref: ") {
        println!(
            "cargo:rerun-if-changed={}",
            git_path(workspace, reference).display()
        );
    }
    let packed = git_path(workspace, "packed-refs");
    if packed.exists() {
        println!("cargo:rerun-if-changed={}", packed.display());
    }
    format!("git:{commit}")
}
