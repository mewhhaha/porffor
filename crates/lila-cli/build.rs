//! Capture the toolchain which actually builds this CLI image.
use std::process::Command;
fn main() {
    for variable in ["RUSTC", "CARGO_ENCODED_RUSTFLAGS"] {
        println!("cargo:rerun-if-env-changed={variable}");
    }
    let version = Command::new(std::env::var_os("RUSTC").expect("Cargo supplies rustc"))
        .args(["--version", "--verbose"])
        .output()
        .expect("compiling rustc is readable");
    assert!(version.status.success(), "rustc version query failed");
    let version = String::from_utf8(version.stdout).expect("rustc version is UTF-8");
    println!(
        "cargo:rustc-env=LILA_PERF_RUSTC={}",
        version
            .chars()
            .flat_map(char::escape_default)
            .collect::<String>()
    );
    for variable in ["TARGET", "HOST", "PROFILE", "OPT_LEVEL", "DEBUG"] {
        println!(
            "cargo:rustc-env=LILA_PERF_{variable}={}",
            std::env::var(variable).expect("Cargo build metadata is present")
        );
    }
    println!(
        "cargo:rustc-env=LILA_PERF_RUSTFLAGS={}",
        std::env::var("CARGO_ENCODED_RUSTFLAGS")
            .unwrap_or_default()
            .chars()
            .flat_map(char::escape_default)
            .collect::<String>()
    );
}
