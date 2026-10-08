//! Build identity must describe the compiler source, not an enclosing checkout.

#[path = "../build.rs"]
#[allow(
    dead_code,
    reason = "the integration test calls the build-script identity function"
)]
mod build_script;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const VERSION: &str = "47.0.0";
const PACKAGING_METADATA: &str = include_str!("../.cargo_vcs_info.json");

struct Directory(PathBuf);

impl Directory {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "wasmtime-cache-identity-{}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed),
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn git_output(directory: &Path, arguments: &[&str]) -> Output {
    Command::new("git")
        .current_dir(directory)
        .args([
            "-c",
            "user.name=Lila cache test",
            "-c",
            "user.email=cache-test@example.invalid",
        ])
        .args(["-c", "commit.gpgsign=false"])
        .arg("-c")
        .arg(format!(
            "core.hooksPath={}",
            directory.join("disabled-hooks").display()
        ))
        .args(arguments)
        .output()
        .expect("Git is available for the source identity regression")
}

fn git(directory: &Path, arguments: &[&str]) -> String {
    let output = git_output(directory, arguments);
    assert!(
        output.status.success(),
        "git {arguments:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

fn checkout(directory: &Path) -> String {
    git(directory, &["init", "--quiet", "--template="]);
    fs::write(directory.join("build.rs"), "fn main() {}\n").unwrap();
    git(directory, &["add", "build.rs"]);
    git(directory, &["commit", "--quiet", "-m", "fixture source"]);
    git(directory, &["rev-parse", "HEAD"])
}

fn packaged_source(directory: &Path) {
    fs::create_dir_all(directory).unwrap();
    fs::write(directory.join(".cargo_vcs_info.json"), PACKAGING_METADATA).unwrap();
    fs::write(
        directory.join("Cargo.toml"),
        "[package]\nname = \"cache-fixture\"\n",
    )
    .unwrap();
    fs::write(directory.join("build.rs"), "fn main() {}\n").unwrap();
}

#[test]
fn packaged_source_ignores_an_unrelated_ancestor_git_checkout() {
    let fixture = Directory::new();
    let unrelated_head = checkout(&fixture.0);
    let source = fixture.0.join("registry/wasmtime-internal-cache");
    packaged_source(&source);
    assert_eq!(git(&source, &["rev-parse", "HEAD"]), unrelated_head);
    assert!(
        !git_output(&source, &["ls-files", "--error-unmatch", "--", "build.rs"])
            .status
            .success()
    );
    assert_eq!(
        build_script::compiler_identity(&source, VERSION),
        (VERSION.to_owned(), false)
    );
}

#[test]
fn tracked_vendored_package_keeps_the_published_identity() {
    let fixture = Directory::new();
    checkout(&fixture.0);
    let source = fixture.0.join("vendor/wasmtime-internal-cache");
    packaged_source(&source);
    git(&fixture.0, &["add", "vendor"]);
    git(
        &fixture.0,
        &["commit", "--quiet", "-m", "vendored published package"],
    );
    assert_eq!(
        git(&source, &["ls-files", "--error-unmatch", "--", "build.rs"]),
        "build.rs"
    );
    assert_eq!(
        build_script::compiler_identity(&source, VERSION),
        (VERSION.to_owned(), false)
    );
}

#[test]
fn development_checkout_keeps_git_and_executable_mtime_invalidation() {
    let fixture = Directory::new();
    let first_head = checkout(&fixture.0);
    assert!(!fixture.0.join(".cargo_vcs_info.json").exists());
    assert_eq!(
        build_script::compiler_identity(&fixture.0, VERSION),
        (first_head.clone(), true)
    );
    fs::write(
        fixture.0.join("build.rs"),
        "fn main() { println!(\"changed\"); }\n",
    )
    .unwrap();
    assert_eq!(
        build_script::compiler_identity(&fixture.0, VERSION),
        (first_head.clone(), true),
        "dirty development builds still require executable mtime invalidation"
    );
    git(
        &fixture.0,
        &["commit", "--quiet", "-a", "-m", "changed fixture source"],
    );
    let second_head = git(&fixture.0, &["rev-parse", "HEAD"]);
    assert_ne!(first_head, second_head);
    assert_eq!(
        build_script::compiler_identity(&fixture.0, VERSION),
        (second_head, true)
    );
}

#[test]
fn unavailable_git_metadata_uses_the_published_version() {
    let fixture = Directory::new();
    // Stop Git discovery here even if the system temporary directory has a
    // Git ancestor. The missing target makes rev-parse fail deterministically.
    fs::write(fixture.0.join(".git"), "gitdir: missing-checkout\n").unwrap();
    assert!(
        !git_output(&fixture.0, &["rev-parse", "HEAD"])
            .status
            .success()
    );
    assert_eq!(
        build_script::compiler_identity(&fixture.0, VERSION),
        (VERSION.to_owned(), false)
    );
}
