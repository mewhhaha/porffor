//! Immutable normalization rows constructed once while building the compiler.

mod format;
pub(super) use format::{NormalizationMapping, NormalizationTables};
use std::sync::OnceLock;

// The same pinned ICU producer runs in the build script. No host pointer,
// native-endian word or pool address is serialized into the immutable image.
const EMBEDDED_IMAGE: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/unicode-normalization.bin"));

pub(super) fn tables() -> &'static NormalizationTables {
    static TABLES: OnceLock<NormalizationTables> = OnceLock::new();
    TABLES.get_or_init(|| {
        NormalizationTables::from_image(EMBEDDED_IMAGE)
            .expect("build-time normalization image must retain complete checked rows")
    })
}

#[cfg(test)]
mod construction;
#[cfg(test)]
mod tests;
