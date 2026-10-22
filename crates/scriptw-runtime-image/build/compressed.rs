//! Compress native storage for both profiles. Raw Wasm packages, native producer
//! output and v4 manifests retain their exact existing bytes and formats.
#[path = "../src/native_storage.rs"]
mod native_storage;

use sha2::{Digest, Sha256};
use std::path::Path;

pub(super) fn write(out: &Path) {
    for stem in ["scriptw-runtime-native", "scriptw-runtime-portable-native"] {
        write_image(out, stem);
    }
}

fn write_image(out: &Path, stem: &str) {
    let raw = std::fs::read(out.join(format!("{stem}.bin")))
        .expect("the checked native producer published its image");
    let encoded = if raw.is_empty() {
        Vec::new()
    } else {
        assert!(raw.len() <= native_storage::MAX_RAW_BYTES);
        let compressed = zstd::stream::encode_all(raw.as_slice(), 1)
            .expect("Cargo's native image compression succeeds");
        let mut encoded = native_storage::MAGIC.to_vec();
        encoded.extend_from_slice(&(raw.len() as u64).to_le_bytes());
        encoded.extend_from_slice(&Sha256::digest(&raw));
        encoded.extend_from_slice(&compressed);
        let decoded = native_storage::decode(&encoded)
            .expect("Cargo's exact encoded image decodes within its framing bounds");
        assert_eq!(
            decoded.bytes(),
            raw.as_slice(),
            "encoded native storage exactly reproduces checked producer bytes"
        );
        println!("{stem} storage: {} -> {} bytes", raw.len(), encoded.len());
        encoded
    };
    std::fs::write(out.join(format!("{stem}.packed")), encoded)
        .expect("encoded native storage writes completely");
}
