//! Embedded upstream provenance. Execution manifests independently bind the
//! actual suite content; the upstream commit and its content tree are distinct.

use scriptw_engine::CompilerCommitId;
use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourcePins {
    ecma262: String,
    test262_upstream: String,
    test262_tree: String,
}

fn exact_git_revision(field: &str, value: &str) -> Result<CompilerCommitId, String> {
    if value.len() != 40 {
        return Err(format!(
            "{field} must identify an exact Git SHA-1 source revision"
        ));
    }
    CompilerCommitId::parse(value).map_err(|error| format!("invalid {field}: {error}"))
}

fn admit_specification_revision(bytes: &str) -> Result<CompilerCommitId, String> {
    let pins: SourcePins = serde_json::from_str(bytes).map_err(|error| error.to_string())?;
    let specification = exact_git_revision("ECMA-262 source revision", &pins.ecma262)?;
    exact_git_revision("Test262 upstream revision", &pins.test262_upstream)?;
    exact_git_revision("Test262 content tree", &pins.test262_tree)?;
    Ok(specification)
}

pub(crate) fn ecma262_revision() -> &'static str {
    static REVISION: OnceLock<CompilerCommitId> = OnceLock::new();
    REVISION
        .get_or_init(|| {
            admit_specification_revision(include_str!("source-pins.json"))
                .expect("embedded conformance source pins must identify exact revisions")
        })
        .as_str()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_pins_reject_unversioned_or_malformed_specification_and_suite_identities() {
        let pins: serde_json::Value =
            serde_json::from_str(include_str!("source-pins.json")).unwrap();
        for field in ["ecma262", "test262_upstream", "test262_tree"] {
            for invalid in [
                "ecma262-current-draft",
                "",
                &"a".repeat(39),
                &"a".repeat(64),
                &"A".repeat(40),
            ] {
                let mut changed = pins.clone();
                changed[field] = invalid.into();
                assert!(
                    admit_specification_revision(&changed.to_string()).is_err(),
                    "{field}: {invalid}"
                );
            }
            let mut missing = pins.clone();
            missing.as_object_mut().unwrap().remove(field);
            assert!(
                admit_specification_revision(&missing.to_string()).is_err(),
                "missing {field}"
            );
        }
        let mut ambiguous = pins;
        ambiguous["test262_revision"] = "b".repeat(40).into();
        assert!(admit_specification_revision(&ambiguous.to_string()).is_err());
    }

    #[test]
    fn embedded_test262_upstream_metadata_matches_the_actual_vendored_tree() {
        let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let checked = std::process::Command::new(if cfg!(windows) { "python" } else { "python3" })
            .arg("-B")
            .arg(repo.join("scripts/check_conformance_source_pins.py"))
            .output()
            .expect("Python checks actual conformance source identity");
        assert!(
            checked.status.success(),
            "source pin validation failed:\n{}\n{}",
            String::from_utf8_lossy(&checked.stdout),
            String::from_utf8_lossy(&checked.stderr),
        );
    }
}
