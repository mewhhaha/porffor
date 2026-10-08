use super::*;
use std::sync::atomic::{AtomicU64, Ordering};

struct Fixture {
    root: PathBuf,
    config: SuiteConfig,
}
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "lila-test262-seed-control-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let suite = root.join("suite");
        let snapshots = root.join("snapshots");
        fs::create_dir_all(suite.join("test")).unwrap();
        fs::create_dir_all(suite.join("harness")).unwrap();
        fs::create_dir_all(&snapshots).unwrap();
        fs::write(
            suite.join("harness/bridgeHelper.js"),
            "var bridgeHelper = 17;\n",
        )
        .unwrap();
        fs::write(
            suite.join("harness/assert.js"),
            "function assert(value) { if (value !== true) throw new Test262Error('assert'); }\n",
        )
        .unwrap();
        fs::write(
            suite.join("harness/doneprintHandle.js"),
            "function $DONE(error) { if (error) throw error; }\n",
        )
        .unwrap();
        Self {
            config: SuiteConfig {
                suite_root: suite,
                snapshot_dir: snapshots,
                local_harness: crate::LocalHarnessSource::EmbeddedWasmAot,
                ..SuiteConfig::default()
            },
            root,
        }
    }
    fn write(&self, source: &str) {
        fs::write(self.config.suite_root.join("test/case.js"), source).unwrap();
    }
    // This source-only fixture stands in for verified membership so the
    // independently checked materialization closure can be damaged directly.
    // Production never exposes this context or a raw Evidence constructor.
    fn seed(&self, mode: crate::TestExecutionMode) -> Test262ReplaySeed {
        let id = TestExecutionId::parse_wire_key(&format!("{}:case.js", mode.as_str())).unwrap();
        let identity = CompilerProvenance::current().unwrap();
        let failure = crate::classify_failure(
            id.clone(),
            FailureKind::HostHarness,
            "original harness witness",
        );
        let selected = SelectedEvidence {
            evidence: Evidence {
                base: "fixture-before".into(),
                candidate: "fixture-after".into(),
                selection: Test262SeedSelection::CandidateFailures,
                base_identity: identity.clone(),
                candidate_identity: identity,
                pins: crate::pinned_revisions(&self.config),
                manifest_hash: 1,
            },
            ids: [id.clone()].into_iter().collect(),
            failures: [(id.clone(), failure)].into_iter().collect(),
        };
        case_proof(
            &self.config,
            &selected,
            &id,
            suite_digest(&self.config.suite_root).unwrap(),
        )
        .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
fn ready(proof: &Materialization) -> &MaterializedSource {
    match proof {
        Materialization::Ready { actual } => actual,
        Materialization::Unavailable { message } => {
            panic!("expected actual materialization: {message}")
        }
    }
}

#[test]
fn strict_sloppy_and_raw_modes_retain_original_source_and_declared_harness() {
    let f = Fixture::new();
    let original =
        "/*---\nincludes: [bridgeHelper.js]\n---*/\nassert.sameValue(bridgeHelper, 17);\n";
    f.write(original);
    let strict = f.seed(crate::TestExecutionMode::StrictScript);
    let sloppy = f.seed(crate::TestExecutionMode::SloppyScript);
    assert!(ready(&strict.wire.wasm_aot)
        .source
        .starts_with("\"use strict\";\n"));
    assert!(!ready(&sloppy.wire.wasm_aot)
        .source
        .starts_with("\"use strict\";\n"));
    for seed in [&strict, &sloppy] {
        let source = &ready(&seed.wire.wasm_aot).source;
        assert!(source.ends_with(original));
        assert!(source.contains("var bridgeHelper = 17;"));
        assert_eq!(seed.wire.includes, ["bridgeHelper.js"]);
        assert_ne!(seed.wire.original_failure.ownership.owner().as_str(), "");
    }
    assert_ne!(strict.fingerprint(), sloppy.fingerprint());
    let raw_source =
        "/*---\nflags: [raw]\nincludes: [not-materialized.js]\n---*/\nvar original = 1;\n";
    f.write(raw_source);
    let raw = f.seed(crate::TestExecutionMode::RawScript);
    assert_eq!(
        ready(&raw.wire.wasm_aot).source.as_bytes(),
        raw_source.as_bytes()
    );
    assert!(ready(&raw.wire.wasm_aot).used_preludes.is_empty());
    assert_eq!(raw.wire.host, HostRequirement::RawUnmaterialized);
}

#[test]
fn module_negative_and_dependency_inventory_keep_the_original_parse_domains() {
    let f = Fixture::new();
    let original = "/*---\nflags: [module]\nincludes: [bridgeHelper.js]\nnegative:\n  phase: resolution\n  type: SyntaxError\n---*/\nimport './dependency.js';\n";
    f.write(original);
    let dependency = f.config.suite_root.join("test/dependency.js");
    fs::write(&dependency, "export var value = 1;\n").unwrap();
    let seed = f.seed(crate::TestExecutionMode::Module);
    let actual = ready(&seed.wire.wasm_aot);
    assert_eq!(actual.source, original);
    assert!(actual
        .module_prelude
        .as_ref()
        .unwrap()
        .contains("bridgeHelper = 17"));
    assert_eq!(
        seed.wire.negative.as_ref().unwrap().phase,
        NegativePhase::Resolution
    );
    let before = suite_digest(&f.config.suite_root).unwrap();
    fs::write(&dependency, "export var value = 2;\n").unwrap();
    assert_ne!(suite_digest(&f.config.suite_root).unwrap(), before);
    let mut damaged = seed.wire.clone();
    damaged.execution_id = TestExecutionId::parse_wire_key("sloppy-script:case.js").unwrap();
    assert!(admit_rederived(seed.clone(), damaged).is_err());
    let mut damaged = seed.wire.clone();
    damaged.negative.as_mut().unwrap().phase = NegativePhase::Runtime;
    assert!(admit_rederived(seed.clone(), damaged).is_err());
    let mut damaged = seed.wire.clone();
    damaged.wasm_aot = Materialization::Ready {
        actual: MaterializedSource {
            source: original.into(),
            module_prelude: None,
            agent_prelude: None,
            used_preludes: Vec::new(),
        },
    };
    assert!(admit_rederived(seed, damaged).is_err());
}

#[test]
fn missing_declared_helpers_are_retained_as_owned_unavailable_setup() {
    let f = Fixture::new();
    f.write("/*---\nflags: [noStrict]\nincludes: [missing.js]\n---*/\ntrue;\n");
    let seed = f.seed(crate::TestExecutionMode::SloppyScript);
    assert!(matches!(
        seed.wire.wasm_aot,
        Materialization::Unavailable { .. }
    ));
    assert!(matches!(
        seed.wire.spec_exec,
        Materialization::Unavailable { .. }
    ));
    assert!(matches!(seed.wire.host, HostRequirement::Unresolved { .. }));
    assert_eq!(seed.wire.includes, ["missing.js"]);
}

#[test]
fn feature_annotations_and_async_agent_hosts_keep_the_original_runner_owners() {
    let f = Fixture::new();
    f.write("/*---\nflags: [noStrict]\nfeatures: [immutable-arraybuffer]\n---*/\ntrue;\n");
    let seed = f.seed(crate::TestExecutionMode::SloppyScript);
    assert!(seed.wire.features.contains("immutable-arraybuffer"));
    let manifest = crate::discover_suite(&f.config, Some(&seed.execution_id().wire_key())).unwrap();
    let case = &manifest.cases[0];
    let preludes = crate::load_preludes(&f.config).unwrap();
    let result = crate::run_one_case(
        case,
        &preludes,
        f.config.timeout_ms,
        ExecutionBackend::WasmAot,
    );
    assert_eq!(result.status, crate::TestStatus::Passed, "{result:?}");

    let original = "/*---\nflags: [async, noStrict]\n---*/\n$262.agent.start('');\n$DONE();\n";
    f.write(original);
    let async_seed = f.seed(crate::TestExecutionMode::SloppyScript);
    assert!(async_seed.wire.flags.contains("async"));
    assert_eq!(
        async_seed.wire.host,
        HostRequirement::Complete {
            realm: false,
            agent_worker: true
        }
    );
    for actual in [&async_seed.wire.wasm_aot, &async_seed.wire.spec_exec] {
        let actual = ready(actual);
        assert!(actual.source.ends_with(original));
        assert!(actual.agent_prelude.is_some());
        assert!(actual
            .used_preludes
            .iter()
            .any(|(name, _)| name == "doneprintHandle.js"));
    }
}

#[test]
fn durable_bridge_output_keeps_each_mode_and_counts_unselected_members() {
    let f = Fixture::new();
    f.write("/*---\n---*/\nassert.sameValue(1, 1);\n");
    let seeds = vec![
        f.seed(crate::TestExecutionMode::SloppyScript),
        f.seed(crate::TestExecutionMode::StrictScript),
    ];
    let plan = Test262SeedPlan { seeds, eligible: 7 };
    let output = f.root.join("retained");
    plan.write(&output).unwrap();
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(output.join("bridge.json")).unwrap()).unwrap();
    assert_eq!(manifest["state"], "complete");
    assert_eq!(manifest["eligible"], 7);
    assert_eq!(manifest["selected"], 2);
    assert_eq!(manifest["retained"], 2);
    let first = fs::read(output.join("0000.test262-seed.json")).unwrap();
    assert!(plan.write(&output).is_err());
    assert_eq!(
        fs::read(output.join("0000.test262-seed.json")).unwrap(),
        first
    );
}

#[test]
fn matching_red_outcomes_and_unsupported_cases_never_promote_to_green() {
    let f = Fixture::new();
    f.write("/*---\nflags: [noStrict]\n---*/\ntrue;\n");
    let seed = f.seed(crate::TestExecutionMode::SloppyScript);
    let result = Test262ReplayResult::Failed {
        kind: FailureKind::Unsupported,
        outcome: OutcomeKind::NotImplemented,
        origin: FailureOrigin::LocalHarness,
        detail: "unsupported owned operation".into(),
        detail_hash: 1,
        duration_ms: 0,
    };
    assert!(equivalent(&result, &result));
    let backend = Test262BackendReplay {
        compiler_identity: None,
        result,
        journal_bytes_hex: String::new(),
        stderr_bytes_hex: String::new(),
    };
    let report = Test262ReplayReport {
        schema_version: SCHEMA,
        state: BridgeState::Complete,
        seed,
        wasm_aot: backend.clone(),
        spec_exec: backend,
        observations_match: true,
    };
    assert!(!report.is_green());
    assert!(!equivalent(
        &Test262ReplayResult::AdmissionRejected {
            message: "missing foundation".into()
        },
        &Test262ReplayResult::AdmissionRejected {
            message: "missing foundation".into()
        }
    ));
    assert!(Test262ReplaySeed::from_json("{\"schema_version\":999}").is_err());
}
