//! Snapshot-selection contracts, not real Test262 conformance evidence.
//! Compile-negative fixtures keep these tests on the product front-end path
//! without enabling the spec-exec oracle or executing expensive Wasm programs.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use super::{
    compare_snapshots, load_aggregate_progress_summary, load_verified_aggregate_summary,
    run_top_level_matrix, CaseExecutionRole, CompilerProvenance, LocalHarnessSource, RunConfig,
    SuiteConfig, TestExecutionId, VerifiedAggregateSummary,
};
use lila_engine::ExecutionBackend;

const BACKEND: ExecutionBackend = ExecutionBackend::WasmAot;
static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    root: PathBuf,
    config: SuiteConfig,
}

impl Fixture {
    fn new() -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should follow the epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "lila-snapshot-comparison-{}-{timestamp}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        let config = SuiteConfig {
            suite_root: root.join("vendor/test262"),
            local_harness: LocalHarnessSource::None,
            snapshot_dir: root.join("snapshots"),
            timeout_ms: 5_000,
            worker_count: 1,
            case_runner_bin: None,
            execution_role: CaseExecutionRole::FixtureWorker,
        };
        fs::create_dir_all(config.suite_root.join("test/language/comparison"))
            .expect("fixture directory should be created");
        let fixture = Self { root, config };
        fixture.case("case.js", "SyntaxError");
        fixture
    }

    fn case(&self, name: &str, expected_error: &str) {
        // The parser always produces SyntaxError. A different declared error
        // deliberately makes the harness record a non-passing outcome.
        fs::write(
            self.config
                .suite_root
                .join("test/language/comparison")
                .join(name),
            format!(
                "/*---\nflags: [raw]\nnegative:\n  phase: parse\n  type: {expected_error}\n---*/\nconst = ;\n"
            ),
        )
        .expect("fixture case should be written");
    }

    fn run(&self, name: &str, expected_total: usize, expected_passed: usize) {
        let summary = run_top_level_matrix(
            &self.config,
            RunConfig {
                snapshot_name: name.to_string(),
                execution_backend: BACKEND,
                ..RunConfig::default()
            },
        )
        .expect("fixture matrix should run");
        assert_eq!(summary.total, expected_total);
        assert_eq!(summary.passed, expected_passed);
        assert_eq!(summary.failed, expected_total - expected_passed);
        let verified = self.verified(name);
        assert_eq!(verified.resolved_snapshot_name, name);
        assert_eq!(verified.summary.total, expected_total);
        assert_eq!(verified.summary.passed, expected_passed);
    }

    fn verified(&self, name: &str) -> VerifiedAggregateSummary {
        load_verified_aggregate_summary(&self.config, name, BACKEND)
            .expect("complete fixture evidence should verify")
    }

    fn assert_missing_name(&self, base: &str, candidate: &str, missing: &str, actual: &str) {
        let error = compare_snapshots(&self.config, base, candidate, BACKEND)
            .expect_err("an absent comparison input must not alias another run");
        assert!(error.contains("requires exact snapshot name"), "{error}");
        assert!(error.contains(&format!("`{missing}`")), "{error}");
        assert!(error.contains(&format!("`{actual}`")), "{error}");
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn compare_requires_existing_base_name() {
    let fixture = Fixture::new();
    fixture.run("candidate", 1, 1);
    fixture.assert_missing_name("missing-base", "candidate", "missing-base", "candidate");
}

#[test]
fn compare_requires_existing_candidate_name() {
    let fixture = Fixture::new();
    fixture.run("baseline", 1, 1);
    fixture.assert_missing_name(
        "baseline",
        "missing-candidate",
        "missing-candidate",
        "baseline",
    );
}

#[test]
fn compare_rejects_two_missing_names() {
    let fixture = Fixture::new();
    fixture.run("unrelated", 1, 1);
    fixture.assert_missing_name(
        "missing-base",
        "missing-candidate",
        "missing-base",
        "unrelated",
    );
}

#[test]
fn compare_accepts_same_explicit_name() {
    let fixture = Fixture::new();
    fixture.case("case.js", "TypeError");
    fixture.run("baseline", 1, 0);
    let comparison = compare_snapshots(&fixture.config, "baseline", "baseline", BACKEND)
        .expect("an explicit self-comparison is valid");
    assert_eq!(comparison.base_snapshot_name, "baseline");
    assert_eq!(comparison.candidate_snapshot_name, "baseline");
    assert_eq!((comparison.base_total, comparison.candidate_total), (1, 1));
    assert!(comparison.added_passes.is_empty());
    assert!(comparison.regressions.is_empty());
    assert!(comparison.changed_failure_hashes.is_empty());
}

#[test]
fn compare_reports_real_pass_and_regression() {
    let fixture = Fixture::new();
    fixture.case("case.js", "TypeError");
    fixture.case("regression.js", "SyntaxError");
    fixture.run("baseline", 2, 1);
    fixture.case("case.js", "SyntaxError");
    fixture.case("regression.js", "TypeError");
    fixture.run("candidate", 2, 1);
    let comparison = compare_snapshots(&fixture.config, "baseline", "candidate", BACKEND)
        .expect("two explicitly named complete snapshots should compare");
    assert_eq!(comparison.base_snapshot_name, "baseline");
    assert_eq!(comparison.candidate_snapshot_name, "candidate");
    assert_eq!((comparison.base_total, comparison.candidate_total), (2, 2));
    assert_eq!(
        comparison.added_passes,
        vec![
            TestExecutionId::parse_wire_key("raw-script:language/comparison/case.js")
                .expect("fixture execution id should parse")
        ]
    );
    assert_eq!(
        comparison.regressions,
        vec![
            TestExecutionId::parse_wire_key("raw-script:language/comparison/regression.js")
                .expect("fixture execution id should parse")
        ]
    );
    assert!(comparison.changed_failure_hashes.is_empty());
}

#[test]
fn verified_status_retains_unique_name_fallback() {
    let fixture = Fixture::new();
    fixture.run("published-elsewhere", 1, 1);
    let verified = fixture.verified("latest");
    assert_eq!(verified.resolved_snapshot_name, "published-elsewhere");
    assert_eq!(verified.summary.total, 1);
    assert_eq!(verified.summary.passed, 1);
}

#[test]
fn compare_rejects_incomplete_exact_candidate() {
    let fixture = Fixture::new();
    fixture.run("baseline", 1, 1);
    fixture.run("candidate", 1, 1);
    let path = fixture.verified("candidate").snapshot_paths.json_path;
    let mut value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).expect("snapshot should read"))
            .expect("snapshot should parse");
    value["completed_nodes"] = serde_json::json!([]);
    fs::write(
        &path,
        serde_json::to_vec(&value).expect("snapshot should encode"),
    )
    .expect("incomplete snapshot should write");
    let error = compare_snapshots(&fixture.config, "baseline", "candidate", BACKEND)
        .expect_err("an incomplete named candidate must not fall back to the baseline");
    assert!(error.contains("aggregate snapshot incomplete"), "{error}");
}

#[test]
fn compare_rejects_corrupt_exact_candidate() {
    let fixture = Fixture::new();
    fixture.run("baseline", 1, 1);
    fixture.run("candidate", 1, 1);
    let path = fixture.verified("candidate").snapshot_paths.json_path;
    fs::write(&path, "not JSON\n").expect("corrupt snapshot should write");
    let error = compare_snapshots(&fixture.config, "baseline", "candidate", BACKEND)
        .expect_err("a corrupt named candidate must not fall back to the baseline");
    assert!(error.contains("failed to parse snapshot"), "{error}");
}

fn replace_family_compiler(fixture: &Fixture, name: &str, digit: char) -> CompilerProvenance {
    let mut producer = None;
    for entry in fs::read_dir(&fixture.config.snapshot_dir).expect("snapshot family should read") {
        let path = entry.expect("snapshot entry should read").path();
        let filename = path.file_name().unwrap().to_str().unwrap();
        if !filename.starts_with(&format!("{name}-"))
            || path.extension().and_then(|x| x.to_str()) != Some("json")
        {
            continue;
        }
        let mut value: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        value["compiler_identity"]["executable_sha256"] =
            serde_json::json!(digit.to_string().repeat(64));
        let identity: CompilerProvenance =
            serde_json::from_value(value["compiler_identity"].clone()).unwrap();
        if let Some(expected) = &producer {
            assert_eq!(expected, &identity);
        }
        producer = Some(identity);
        fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
    }
    producer.expect("family must contain aggregate and node evidence")
}

#[test]
fn comparison_retains_two_complete_historical_compilers_without_rebinding_them() {
    let fixture = Fixture::new();
    fixture.run("baseline", 1, 1);
    fixture.run("candidate", 1, 1);
    let base = replace_family_compiler(&fixture, "baseline", '1');
    let candidate = replace_family_compiler(&fixture, "candidate", '2');
    assert_ne!(base, candidate);
    let comparison = compare_snapshots(&fixture.config, "baseline", "candidate", BACKEND)
        .expect("read-only comparison retains each complete family's compiler");
    assert_eq!(comparison.base_compiler_identity, base);
    assert_eq!(comparison.candidate_compiler_identity, candidate);
    assert!(comparison.added_passes.is_empty() && comparison.regressions.is_empty());
    let error = load_verified_aggregate_summary(&fixture.config, "baseline", BACKEND)
        .expect_err("current status may not bind historical evidence to the running compiler");
    assert!(
        error.contains("different compiler build or executable"),
        "{error}"
    );
}

#[test]
fn complete_and_progress_aggregates_reject_a_node_from_a_different_compiler() {
    let fixture = Fixture::new();
    fixture.run("mixed", 1, 1);
    let aggregate = fixture.verified("mixed").snapshot_paths.json_path;
    let node = fs::read_dir(&fixture.config.snapshot_dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| {
            path != &aggregate
                && path
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .starts_with("mixed-")
                && path.extension().and_then(|x| x.to_str()) == Some("json")
        })
        .expect("complete fixture has node evidence");
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&node).unwrap()).unwrap();
    value["compiler_identity"]["executable_sha256"] = serde_json::json!("3".repeat(64));
    fs::write(&node, serde_json::to_vec(&value).unwrap()).unwrap();
    for result in [
        load_verified_aggregate_summary(&fixture.config, "mixed", BACKEND).map(|_| ()),
        load_aggregate_progress_summary(&fixture.config, "mixed", BACKEND).map(|_| ()),
    ] {
        let error = result.expect_err("every recorded node must belong to the aggregate compiler");
        assert!(error.contains("different compiler_identity"), "{error}");
    }
    let error = run_top_level_matrix(
        &fixture.config,
        RunConfig {
            snapshot_name: "mixed".into(),
            execution_backend: BACKEND,
            resume: true,
            ..RunConfig::default()
        },
    )
    .expect_err("resume must reject foreign compiler node evidence rather than dropping it");
    assert!(error.contains("different compiler_identity"), "{error}");
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&fs::read(node).unwrap()).unwrap(),
        value
    );
}
