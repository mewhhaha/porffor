mod compiler_fingerprint;

use std::path::PathBuf;

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
}
