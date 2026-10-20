mod compiler_fingerprint;

use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let manifest = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let workspace = manifest
        .join("../..")
        .canonicalize()
        .expect("Scriptw workspace root should resolve");
    for input in compiler_fingerprint::compiler_rerun_inputs(&workspace)
        .expect("production compiler rerun inputs should resolve")
    {
        println!("cargo:rerun-if-changed={}", input.display());
    }
    let fingerprint = compiler_fingerprint::fingerprint(&workspace)
        .unwrap_or_else(|error| panic!("failed to fingerprint Scriptw compiler inputs: {error}"));
    println!("cargo:rustc-env=SCRIPTW_COMPILER_FINGERPRINT={fingerprint}");
    let native_fingerprint = compiler_fingerprint::native_fingerprint(&workspace)
        .expect("native compiler dependency inputs are fully fingerprinted");
    println!("cargo:rustc-env=SCRIPTW_NATIVE_COMPILER_FINGERPRINT={native_fingerprint}");
    let target = std::env::var("TARGET").expect("Cargo target triple");
    println!("cargo:rustc-env=SCRIPTW_RUNTIME_BUILD_TARGET={target}");
    println!(
        "cargo:rustc-env=SCRIPTW_COMPILER_FINGERPRINT_SCHEME={}",
        compiler_fingerprint::FINGERPRINT_SCHEME
    );
    println!(
        "cargo:rustc-env=SCRIPTW_COMPILER_SOURCE_REVISION={}",
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
