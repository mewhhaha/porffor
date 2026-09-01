use std::fs;
use std::path::{Path, PathBuf};

const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/test262-execution-identity.md");
const TASK: &str = include_str!("../../../tasks/01-baseline-and-generated-backlog.md");

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("crate should live under crates/")
        .to_path_buf()
}

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn normalized(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn count_in_rust_sources(root: &Path, needle: &str) -> usize {
    fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .map(|path| {
            if path.is_dir() {
                return count_in_rust_sources(&path, needle);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return 0;
            }
            fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
                .matches(needle)
                .count()
        })
        .sum()
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
    assert!(journal.contains("strikes: BTreeMap<TestExecutionId, CaseStrikes>"));
    assert!(journal.contains("struct StrikeEntries(Vec<(String, u32)>)"));
    assert!(journal.contains("#[serde(transparent)]\npub(crate) struct CaseStrikes(NonZeroU32)"));
    assert!(journal.contains("serde_json::json!(1)"));
}

#[test]
fn execution_plan_is_the_exact_private_no_capability_domain() {
    let root = repo_root();
    let source = fs::read_to_string(root.join("crates/lila-test262/src/lib.rs"))
        .expect("lila-test262 source should read");
    let enum_offset = source
        .find("enum TestExecutionPlan {")
        .expect("execution plan should exist");
    let documentation_offset = source[..enum_offset]
        .rfind("/// The validated expansion of one physical file.")
        .expect("execution plan documentation should exist");
    let preceding_item_end = source[..documentation_offset]
        .rfind('}')
        .expect("item before execution plan should close");
    assert!(
        !source[preceding_item_end + 1..enum_offset].contains("#["),
        "the execution plan must not acquire capabilities before or after its documentation"
    );
    let declaration = bounded(
        &source,
        "/// The validated expansion of one physical file.",
        "impl TestExecutionPlan",
    );

    assert_eq!(
        normalized(bounded(declaration, "enum TestExecutionPlan {", "}")),
        "One(TestExecutionMode),SloppyAndStrict,"
    );
    assert!(!source.contains("pub enum TestExecutionPlan"));
    assert!(!source.contains("pub(crate) enum TestExecutionPlan"));
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq", "Default"] {
        assert!(
            !source.contains(&format!("impl {capability} for TestExecutionPlan")),
            "found manual `{capability}` capability"
        );
    }
    assert_eq!(
        count_in_rust_sources(
            &root.join("crates/lila-test262/src"),
            "TestExecutionPlan"
        ),
        5,
        "the declaration, implementation, discovery owner and two unit witnesses are the complete ownership census"
    );
}

#[test]
fn execution_plan_exhaustively_binds_flags_to_ordered_modes() {
    let source = fs::read_to_string(repo_root().join("crates/lila-test262/src/lib.rs"))
        .expect("lila-test262 source should read");
    let flag_plan = normalized(bounded(
        &source,
        "    fn from_flags(path: &str, flags: &BTreeSet<String>) -> Result<Self, String> {",
        "    fn modes(self) -> impl Iterator<Item = TestExecutionMode> {",
    ));
    assert!(flag_plan.contains(concat!(
        "letonly_strict=flags.contains(\"onlyStrict\");",
        "letno_strict=flags.contains(\"noStrict\");",
        "letraw=flags.contains(\"raw\");",
        "letmodule=flags.contains(\"module\");"
    )));
    assert_eq!(
        flag_plan
            .matches("match(raw,module,only_strict,no_strict){")
            .count(),
        1
    );
    for row in [
        "(true,true,false,false)=>Ok(Self::One(TestExecutionMode::RawModule)),",
        "(true,false,false,false)=>Ok(Self::One(TestExecutionMode::RawScript)),",
        "(false,true,false,false)=>Ok(Self::One(TestExecutionMode::Module)),",
        "(false,false,true,false)=>Ok(Self::One(TestExecutionMode::StrictScript)),",
        "(false,false,false,true)=>Ok(Self::One(TestExecutionMode::SloppyScript)),",
        "(false,false,false,false)=>Ok(Self::SloppyAndStrict),",
    ] {
        assert_eq!(
            flag_plan.matches(row).count(),
            1,
            "missing exact row `{row}`"
        );
    }
    assert_eq!(flag_plan.matches("Ok(Self::").count(), 6);

    let modes = normalized(bounded(
        &source,
        "    fn modes(self) -> impl Iterator<Item = TestExecutionMode> {",
        "\n}\n\n#[derive(Debug, Clone, PartialEq, Eq)]\npub struct TestCase",
    ));
    assert_eq!(
        modes,
        "letmodes=matchself{Self::One(mode)=>[Some(mode),None],Self::SloppyAndStrict=>[Some(TestExecutionMode::SloppyScript),Some(TestExecutionMode::StrictScript),],};modes.into_iter().flatten()}"
    );
    assert!(!modes.contains("_=>"));
    assert!(CONTRACT.contains("TestExecutionPlan"));
    assert!(TASK.contains("TestExecutionPlan"));
}
