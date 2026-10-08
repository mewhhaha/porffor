//! Deterministic native keyword data exported from the pinned typed CLDR rows.

use icu_locale::extensions::{transform, unicode};
use serde_json::json;

struct UnicodeKeywordAliases {
    key: unicode::Key,
    replacements: &'static [(&'static str, &'static str)],
}
struct TransformKeywordAliases {
    key: transform::Key,
    replacements: &'static [(&'static str, &'static str)],
}

#[path = "../provider/keyword_aliases/generated.rs"]
mod pinned;

pub fn export() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let unicode: Vec<_> = pinned::UNICODE_ALIASES
        .iter()
        .map(|row| json!({"key": row.key.as_str(), "replacements": row.replacements}))
        .collect();
    let transform: Vec<_> = pinned::TRANSFORM_ALIASES
        .iter()
        .map(|row| json!({"key": row.key.as_str(), "replacements": row.replacements}))
        .collect();
    // Arrays preserve the producer's strict key/source order. serde_json's
    // canonical map order and compact encoding have no host-dependent fields.
    Ok(serde_json::to_vec(&json!({
        "schema": 1,
        "algorithm": "keyword-aliases-cldr47-v1",
        "cldr_release": "47.0.0",
        "cldr_commit": "2ef784e3a4168bc2a43cd1b5b9839b6636f5899c",
        "provider_data_sha256": pinned::PROVIDER_DATA_SHA256,
        "unicode": unicode,
        "transform": transform,
    }))?)
}
