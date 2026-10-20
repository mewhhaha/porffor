use serde::Deserialize;
use sha2::{Digest as _, Sha256};

use super::{LocaleTextDirection, LocaleTextProfileError};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawProfile {
    schema: u32,
    algorithm: String,
    cldr_release: String,
    cldr_commit: String,
    primary_manifest_sha256: String,
    primary_source_sha256: String,
    license_sha256: String,
    scripts: Vec<RawScript>,
}

// A mandatory null field differs from an absent field. Untagged unit admits
// explicit null without serde's optional-field default for missing directions.
#[derive(Deserialize)]
#[serde(untagged)]
enum RawDirection {
    Known(LocaleTextDirection),
    Unknown(()),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawScript {
    script: String,
    direction: RawDirection,
}

struct ScriptDirection {
    script: Box<str>,
    direction: Option<LocaleTextDirection>,
}

pub(crate) struct LocaleTextProfile {
    rows: Box<[ScriptDirection]>,
}

fn sha_spelling(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

impl LocaleTextProfile {
    pub(crate) fn from_bytes(
        bytes: &[u8],
        digest: [u8; 32],
    ) -> Result<Self, LocaleTextProfileError> {
        let actual: [u8; 32] = Sha256::digest(bytes).into();
        if actual != digest {
            return Err(LocaleTextProfileError::Digest);
        }
        let raw: RawProfile =
            serde_json::from_slice(bytes).map_err(|_| LocaleTextProfileError::Encoding)?;
        if raw.schema != 1 {
            return Err(LocaleTextProfileError::Schema);
        }
        if raw.algorithm != "locale-text-cldr47-v1"
            || raw.cldr_release != "47.0.0"
            || raw.cldr_commit != "2ef784e3a4168bc2a43cd1b5b9839b6636f5899c"
            || !sha_spelling(&raw.primary_manifest_sha256)
            || raw.primary_source_sha256
                != "f765a79559429de5cf1c3346df9eebb73be5e46cef4489df7052b878e230bb0b"
            || raw.license_sha256
                != "b4c0ae8ef04f7059f96ce5bbe0467f9fe6f6d81bbe13517701dfeb961fb4d0b6"
        {
            return Err(LocaleTextProfileError::Revision);
        }
        let mut rows: Vec<ScriptDirection> = Vec::with_capacity(raw.scripts.len());
        for item in raw.scripts {
            let bytes = item.script.as_bytes();
            if bytes.len() != 4
                || !bytes[0].is_ascii_uppercase()
                || !bytes[1..].iter().all(u8::is_ascii_lowercase)
            {
                return Err(LocaleTextProfileError::Script);
            }
            if rows
                .last()
                .is_some_and(|previous| previous.script.as_ref() >= item.script.as_str())
            {
                return Err(LocaleTextProfileError::ScriptOrder);
            }
            let direction = match item.direction {
                RawDirection::Known(direction) => Some(direction),
                RawDirection::Unknown(()) => None,
            };
            rows.push(ScriptDirection {
                script: item.script.into_boxed_str(),
                direction,
            });
        }
        if rows.len() != 177 {
            return Err(LocaleTextProfileError::Coverage);
        }
        Ok(Self {
            rows: rows.into_boxed_slice(),
        })
    }

    pub(crate) fn direction(&self, script: &str) -> Option<LocaleTextDirection> {
        self.rows
            .binary_search_by(|row| row.script.as_ref().cmp(script))
            .ok()
            .and_then(|index| self.rows[index].direction)
    }
}
