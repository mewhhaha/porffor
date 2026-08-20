use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("crate should live under crates/")
        .to_path_buf()
}

fn struct_body<'a>(source: &'a str, name: &str) -> &'a str {
    let marker = format!("pub struct {name} {{");
    let start = source.find(&marker).expect("public struct should exist") + marker.len();
    let end = source[start..]
        .find("\n}")
        .map(|offset| start + offset)
        .expect("struct should close");
    &source[start..end]
}

#[test]
fn test_case_and_materialized_test_have_one_path_mode_authority() {
    let source = fs::read_to_string(repo_root().join("crates/lila-test262/src/lib.rs"))
        .expect("lila-test262 source should read");

    for name in ["TestCase", "MaterializedTest"] {
        let body = struct_body(&source, name);
        assert!(body.contains("execution_id: TestExecutionId"), "{name}");
        assert!(!body.contains("pub execution_id"), "{name}");
        assert!(!body.contains("path: String"), "{name}");
        assert!(!body.contains("execution_mode:"), "{name}");
    }

    let case = struct_body(&source, "TestCase");
    assert!(case.contains("original_source: Arc<str>"));
    assert!(case.contains("negative: Option<Arc<NegativeExpectation>>"));

    let result = struct_body(&source, "TestResult");
    assert!(result.contains("test_id: TestExecutionId"));
    assert!(!result.contains("test_path:"));
}

#[test]
fn durable_schemas_and_child_selection_are_execution_identity_aware() {
    let root = repo_root();
    let source = fs::read_to_string(root.join("crates/lila-test262/src/lib.rs"))
        .expect("lila-test262 source should read");
    let journal = fs::read_to_string(root.join("crates/lila-test262/src/attempt_journal.rs"))
        .expect("attempt journal source should read");

    assert!(source.contains("const SNAPSHOT_VERSION: u32 = 7;"));
    assert!(source.contains("const MATRIX_STRATEGY_VERSION: u32 = 3;"));
    assert!(journal.contains("const ATTEMPT_JOURNAL_VERSION: u32 = 3;"));
    assert!(source.contains(".arg(case.execution_id.wire_key())"));
    assert!(source.contains("pub case_ids: Vec<TestExecutionId>"));
    assert!(source.contains("pub completed_test_ids: Vec<TestExecutionId>"));
    assert!(source.contains("pub checkpoint_identity: Option<CheckpointRunIdentity>"));
    assert!(source.contains("enum WireCheckpointIdentity"));
    assert!(source.contains("enum ResumeCheckpointLoadPolicy"));
    assert!(source.contains("RequireExact"));
    assert!(source.contains("TreatStaleEnvelopeAsAbsent"));
    assert!(source.contains("ResumeCheckpointLoadError::StaleEnvelope"));
    assert!(source.contains("checkpoint_load_policy.resolve("));
    assert!(!source.contains("err.starts_with(\"resume node snapshot mismatch\")"));
    let production = source
        .split_once("\nmod tests {")
        .expect("lila-test262 source should contain its test module")
        .0;
    assert_eq!(
        production
            .matches("ResumeCheckpointLoadPolicy::TreatStaleEnvelopeAsAbsent")
            .count(),
        1,
        "only the top-level low-RAM product flow may select permissive checkpoint loading"
    );
    assert!(journal.contains("strikes: BTreeMap<TestExecutionId, u32>"));
}
