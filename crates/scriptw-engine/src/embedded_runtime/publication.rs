//! Publish each complete Cargo-owned runtime bundle with its own v4 manifest.

#[path = "output.rs"]
mod output;

use scriptw_aot_wasm::RuntimeSourceCompilationProfile;
use std::path::Path;

pub(super) fn publish(
    out: &Path,
    profile: RuntimeSourceCompilationProfile,
    package: &[u8],
    native: &[u8],
    manifest: &[u8],
) {
    let prefix = match profile {
        RuntimeSourceCompilationProfile::NativeHost => "scriptw-runtime",
        RuntimeSourceCompilationProfile::Portable => "scriptw-runtime-portable",
    };
    // Cargo admits the producer only after both complete triplets are written.
    for (suffix, bytes) in [
        ("package", package),
        ("native", native),
        ("manifest", manifest),
    ] {
        output::write(&out.join(format!("{prefix}-{suffix}.bin")), bytes)
            .expect("build runtime output writes completely");
    }
}
