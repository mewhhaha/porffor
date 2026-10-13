//! Publish the complete Cargo-owned runtime bundle and its binding manifest.

use std::path::Path;

pub(super) fn publish(out: &Path, package: &[u8], native: &[u8], manifest: &[u8]) {
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
