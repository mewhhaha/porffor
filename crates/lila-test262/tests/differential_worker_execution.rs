//! Paired execution uses Cargo's selected worker, never this test image.
#![cfg(feature = "spec-exec-oracle")]

use lila_test262::differential::*;
use lila_test262::CompilerProvenance;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

static EXECUTION: Mutex<()> = Mutex::new(());

fn serial_execution() -> std::sync::MutexGuard<'static, ()> {
    // This lock owns no mutable state. A previous assertion failure must not
    // prevent the remaining independent controls from producing their verdicts.
    EXECUTION
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn runner() -> &'static DifferentialWorkerRunner {
    static RUNNER: OnceLock<DifferentialWorkerRunner> = OnceLock::new();
    RUNNER.get_or_init(|| {
        DifferentialWorkerRunner::new(env!("CARGO_BIN_EXE_lila-differential-worker"))
            .expect("Cargo's named oracle worker is readable")
    })
}

fn replay(input: &DifferentialReplayInput) -> DifferentialReport {
    let _serial = serial_execution();
    replay_case(input, SpecExecOracle::explicitly_enabled(), runner())
        .expect("the selected worker admits and completes the authored case")
}

fn case_v1() -> DifferentialReplayInput {
    DifferentialReplayInput::from_json(include_str!(
        "differential/v1/t25-foundation-arithmetic-self-check.json"
    ))
    .unwrap()
}
fn case_v2() -> DifferentialReplayInput {
    DifferentialReplayInput::from_json(include_str!(
        "differential/v2/t25-foundation-primitive-number.json"
    ))
    .unwrap()
}
fn case_v3() -> DifferentialReplayInput {
    DifferentialReplayInput::from_json(include_str!(
        "differential/v3/t25-foundation-primitive-number-and-print.json"
    ))
    .unwrap()
}

const GENERATED_CASE: &str =
    include_str!("differential/v1/t25-generated-integer-arithmetic-v1-seed-1.json");
const BITWISE_PROBE: &str = include_str!("differential/v1/t25-integer-bitwise-v2-conversions.js");
const PRODUCT_PROBE: &str = include_str!("differential/v1/t25-integer-product-v3-signed-zero.js");
const GENERATED_CASE_TIMEOUT_MS: u64 = 5_000;

#[cfg(unix)]
#[test]
fn committed_v1_foundation_case_replays_through_both_backends() {
    let report = replay(&case_v1());

    assert_eq!(report.verdict(), DifferentialVerdict::BothCompleted);
    assert!(report.is_green());
}

#[cfg(unix)]
#[test]
fn committed_v2_primitive_case_replays_through_both_backends() {
    let report = replay(&case_v2());

    assert_eq!(
        report.verdict(),
        DifferentialVerdict::PrimitiveCompletionsMatch
    );
    assert!(report.is_green());
    for observation in [report.wasm_aot(), report.spec_exec()] {
        assert!(matches!(
            &observation.execution,
            ExecutionObservation::PrimitiveCompletion {
                completion: PrimitiveCompletionObservation::Normal {
                    value: PrimitiveValueObservation::Number { bits }
                },
                ..
            } if bits == "4008000000000000"
        ));
    }
}

#[cfg(unix)]
#[test]
fn committed_v3_primitive_and_print_case_replays_through_both_backends() {
    let report = replay(&case_v3());

    assert_eq!(
        report.verdict(),
        DifferentialVerdict::PrimitiveCompletionAndPrintTranscriptMatch
    );
    assert!(report.is_green());
    for observation in [report.wasm_aot(), report.spec_exec()] {
        assert!(matches!(
            &observation.execution,
            ExecutionObservation::PrimitiveCompletion {
                completion: PrimitiveCompletionObservation::Normal {
                    value: PrimitiveValueObservation::Number { bits }
                },
                ..
            } if bits == "4008000000000000"
        ));
        assert_eq!(
            &observation.output_events,
            &OutputEventsObservation::Captured {
                events: vec!["first".to_string(), "second".to_string()]
            }
        );
    }
}

#[cfg(unix)]
#[test]
fn selected_object_probe_uses_actual_backends_and_observable_realm_anchors() {
    let input = DifferentialReplayInput::from_json(include_str!(
        "differential/v5/t25-object-graph-descriptors-and-realm.json"
    ))
    .unwrap();
    let report = replay(&input);
    assert_eq!(
        report.verdict(),
        DifferentialVerdict::SelectedObjectProbeAndPrintTranscriptMatch
    );
    assert!(report.is_green());
    for observation in [report.wasm_aot(), report.spec_exec()] {
        assert_eq!(
            observation.output_events,
            OutputEventsObservation::Captured {
                events: vec!["object-probe:selected".into()]
            }
        );
        let ExecutionObservation::SelectedObjectProbe { graph, .. } = &observation.execution else {
            panic!("worker did not publish the validated object graph")
        };
        let graph = serde_json::to_value(graph).unwrap();
        assert_eq!(
            graph["roots"][0]["value"], graph["roots"][1]["value"],
            "both roots retain the same observed identity"
        );
        assert_eq!(
            graph["roots"][3]["value"]["value"], true,
            "actual caught TypeError has the selected primary Realm prototype"
        );
        assert_eq!(
            graph["roots"][2]["value"], graph["anchors"][0]["value"],
            "caught error prototype and selected TypeError.prototype anchor retain actual identity"
        );
        let keys = &graph["nodes"][0]["properties"];
        assert_eq!(keys[0]["key"]["units"], serde_json::json!([50]));
        assert_eq!(keys[1]["key"]["units"], serde_json::json!([49, 48]));
        assert_eq!(keys[2]["key"]["units"], serde_json::json!([122]));
        assert_eq!(keys[3]["key"]["units"], serde_json::json!([97]));
        assert_eq!(keys[4]["descriptor"]["kind"], "accessor");
        assert_eq!(keys[5]["key"]["type"], "symbol");
    }
}

#[cfg(unix)]
#[test]
fn selected_probe_reflection_traps_remain_observable() {
    let input = DifferentialReplayInput::new_script("t25/object-probe/proxy-traps", DifferentialProtocol::V5SelectedObjectProbePrintTranscript, "differential/v5/proxy-traps.js", 30_000, r#"
var target = Object.create(null);
target.x = 1;
var proxy = new Proxy(target, {
    getPrototypeOf: function (value) { print("prototype"); return Object.getPrototypeOf(value); },
    ownKeys: function (value) { print("keys"); return Reflect.ownKeys(value); },
    getOwnPropertyDescriptor: function (value, key) { print("descriptor"); return Object.getOwnPropertyDescriptor(value, key); },
    isExtensible: function (value) { print("extensible"); return Object.isExtensible(value); }
});
return {roots: [{name: "proxy", value: proxy}], anchors: []};
"#).unwrap();
    let report = replay(&input);
    assert_eq!(
        report.verdict(),
        DifferentialVerdict::SelectedObjectProbeAndPrintTranscriptMatch
    );
    for observation in [report.wasm_aot(), report.spec_exec()] {
        assert_eq!(
            observation.output_events,
            OutputEventsObservation::Captured {
                events: vec![
                    "prototype".into(),
                    "keys".into(),
                    "descriptor".into(),
                    "extensible".into()
                ]
            }
        );
    }
}

#[cfg(unix)]
#[test]
fn selected_probe_captures_primordials_before_intrinsic_mutation() {
    let input = DifferentialReplayInput::new_script(
        "t25/object-probe/captured-primordials",
        DifferentialProtocol::V5SelectedObjectProbePrintTranscript,
        "differential/v5/captured-primordials.js",
        30_000,
        r#"
var root = Object.create(null);
root.x = "captured";
function changed() { throw new Error("mutated primordial was called"); }
Object.prototype.toJSON = changed;
Object.defineProperty(Array.prototype, "0", {set: changed, configurable: true});
JSON.stringify = changed;
Reflect.ownKeys = changed;
Object.getOwnPropertyDescriptor = changed;
Object.getPrototypeOf = changed;
Object.isExtensible = changed;
Object.setPrototypeOf = changed;
return {roots: [{name: "root", value: root}], anchors: []};
"#,
    )
    .unwrap();
    let report = replay(&input);
    assert_eq!(
        report.verdict(),
        DifferentialVerdict::SelectedObjectProbeAndPrintTranscriptMatch
    );
    for observation in [report.wasm_aot(), report.spec_exec()] {
        assert_eq!(
            observation.output_events,
            OutputEventsObservation::Captured { events: vec![] }
        );
        let ExecutionObservation::SelectedObjectProbe { graph, .. } = &observation.execution else {
            panic!("capture used a mutated intrinsic")
        };
        let graph = serde_json::to_value(graph).unwrap();
        assert_eq!(graph["nodes"].as_array().unwrap().len(), 1);
        assert_eq!(
            graph["nodes"][0]["properties"][0]["descriptor"]["value"]["units"],
            serde_json::json!([99, 97, 112, 116, 117, 114, 101, 100])
        );
    }
}

const CASES: [(&str, &str, &[&str]); 7] = [
    (
        "cycles",
        include_str!("differential/v4/t25-module-cycles-and-metadata.json"),
        &["embedded-cycle:3"],
    ),
    (
        "self",
        include_str!("differential/v4/t25-script-self-import.json"),
        &["embedded-script-self:2"],
    ),
    (
        "attributes",
        include_str!("differential/v4/t25-computed-exact-attributes.json"),
        &["embedded-attributes:1,2"],
    ),
    (
        "undeclared",
        include_str!("differential/v4/t25-undeclared-and-unused.json"),
        &["embedded-undeclared:denied"],
    ),
    (
        "parse",
        include_str!("differential/v4/t25-dynamic-parse-failure.json"),
        &["embedded-parse:rejected"],
    ),
    (
        "defer",
        include_str!("differential/v4/t25-defer-order.json"),
        &[
            "embedded-defer:before",
            "embedded-defer:body",
            "embedded-defer:value=7",
        ],
    ),
    (
        "source",
        include_str!("differential/v4/t25-source-phase.json"),
        &["embedded-source:rejected"],
    ),
];

#[cfg(unix)]
#[test]
fn embedded_corpus_uses_real_paired_backends_and_exact_expected_observations() {
    for (name, json, output) in CASES {
        let case = DifferentialReplayInput::from_json(json).unwrap();
        let report = replay(&case);
        assert_eq!(
            report.verdict(),
            DifferentialVerdict::PrimitiveCompletionAndPrintTranscriptMatch,
            "{name}: {}",
            report.to_pretty_json().unwrap(),
        );
        let expected = PrimitiveCompletionObservation::Normal {
            value: match case.goal() {
                DifferentialGoal::Script => PrimitiveValueObservation::Number {
                    bits: format!("{:016x}", 262.0f64.to_bits()),
                },
                DifferentialGoal::Module => PrimitiveValueObservation::Undefined,
            },
        };
        for observed in [report.wasm_aot(), report.spec_exec()] {
            assert_eq!(
                observed.output_events,
                OutputEventsObservation::Captured {
                    events: output.iter().map(|line| (*line).into()).collect(),
                },
                "{name}",
            );
            assert!(
                matches!(&observed.execution,
                    ExecutionObservation::PrimitiveCompletion { completion, .. } if completion == &expected),
                "{name}: {:?}",
                observed.execution,
            );
        }
    }
}

#[cfg(unix)]
#[test]
fn committed_generated_arithmetic_case_replays_through_both_backends() {
    let case = DifferentialReplayInput::from_json(GENERATED_CASE)
        .expect("committed generated case should decode");
    let report = replay(&case);

    assert_eq!(report.verdict(), DifferentialVerdict::BothCompleted);
}

#[cfg(unix)]
#[test]
fn bitwise_conversion_probe_replays_through_both_backends() {
    let case = DifferentialReplayInput::new_script(
        "t25/regressions/integer-bitwise-v2/conversions-and-shifts",
        DifferentialProtocol::V1SelfCheckingNoOutput,
        "differential/v1/integer-bitwise-v2-conversions.js",
        GENERATED_CASE_TIMEOUT_MS,
        BITWISE_PROBE,
    )
    .expect("the hand-authored bitwise probe is a source-closed Script");
    let report = replay(&case);
    assert_eq!(report.verdict(), DifferentialVerdict::BothCompleted);
}

#[cfg(unix)]
#[test]
fn finite_product_signed_zero_probe_replays_through_both_backends() {
    let case = DifferentialReplayInput::new_script(
        "t25/regressions/integer-product-v3/signed-zero",
        DifferentialProtocol::V1SelfCheckingNoOutput,
        "differential/v1/integer-product-v3-signed-zero.js",
        GENERATED_CASE_TIMEOUT_MS,
        PRODUCT_PROBE,
    )
    .expect("the hand-authored product probe is a source-closed Script");
    let report = replay(&case);
    assert_eq!(report.verdict(), DifferentialVerdict::BothCompleted);
}

// This Engine policy witness is deliberately not a worker loading mode.
mod loader_policy {
    use super::*;
    use lila_engine::{
        CompileOptions, Engine, ExecutionBackend, HostOutputEvent, ModuleLoadingPolicy,
        ObservedCompletion, RealmBuilder, RunOptions,
    };
    fn runtime_created_import_sources(specifier: &str) -> Vec<(&'static str, String)> {
        let specifier =
            serde_json::to_string(specifier).expect("module specifier should encode as JSON");
        let import = format!(
            "import({specifier}).then(() => print('ambient-loaded'), () => print('module-rejected'))"
        );
        let import_source =
            serde_json::to_string(&import).expect("dynamic import source should encode as JSON");
        let function_body = serde_json::to_string(&format!("return {import}"))
            .expect("Function body should encode as JSON");
        let agent_import = format!(
            "import({specifier}).then(() => {{ print('ambient-loaded'); $262.agent.leaving(); }}, () => {{ print('module-rejected'); $262.agent.leaving(); }});"
        );
        let agent_import = serde_json::to_string(&agent_import)
            .expect("agent import source should encode as JSON");
        vec![
            ("direct-eval", format!("eval({import_source});")),
            ("indirect-eval", format!("(0, eval)({import_source});")),
            (
                "function-constructor",
                format!("Function({function_body})();"),
            ),
            (
                "created-realm",
                format!("$262.createRealm().evalScript({import_source});"),
            ),
            ("agent", format!("$262.agent.start({agent_import});")),
        ]
    }

    #[cfg(feature = "spec-exec-oracle")]
    fn module_loader_context_sources(specifier: &str) -> Vec<(&'static str, String)> {
        let mut runtime_sources = runtime_created_import_sources(specifier);
        let encoded_specifier =
            serde_json::to_string(specifier).expect("module specifier should encode as JSON");
        let mut sources = vec![(
            "root",
            format!(
                "import({encoded_specifier}).then(() => print('ambient-loaded'), () => print('module-rejected'));"
            ),
        )];
        sources.append(&mut runtime_sources);
        sources
    }

    fn observe_spec_exec_script_with_module_policy(
        source: &str,
        filename: &str,
        module_loading_policy: ModuleLoadingPolicy,
    ) -> (ObservedCompletion, Vec<String>) {
        let engine = Engine::new(RealmBuilder::new().build());
        let outcome = engine
            .observe_script(
                source,
                CompileOptions {
                    filename: Some(filename.into()),
                    module_loading_policy,
                    ..CompileOptions::default()
                },
                RunOptions {
                    backend: ExecutionBackend::SpecExec,
                    test_path: Some(filename.into()),
                    can_block: false,
                    ..RunOptions::default()
                },
            )
            .expect("the separately scoped Engine loader witness completes");
        assert_eq!(outcome.backend_used, ExecutionBackend::SpecExec);
        let events = outcome
            .output_events
            .into_iter()
            .map(|event| match event {
                HostOutputEvent::PrintLine(text) => text,
            })
            .collect();
        (outcome.completion, events)
    }
    #[cfg(unix)]
    #[test]
    fn filesystem_control_and_reject_all_cover_every_spec_exec_host_context() {
        let _serial = serial_execution();
        // Boa's ordinary filesystem loader is rooted at the process working
        // directory. Keep this positive witness inside that admitted root even
        // when the verification launcher sets TMPDIR elsewhere.
        let staging = TestDirectory::in_directory(
            "loader-policy",
            &std::env::current_dir().unwrap().join("target"),
        );
        let directory = staging.path.clone();
        let ambient_path = directory.join("ambient.mjs");
        let entry_path = directory.join("entry.js");
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).expect("ambient witness directory should exist");
        std::fs::write(
            &ambient_path,
            "print('ambient-module-body'); export const value = 1;\n",
        )
        .expect("ambient witness module should exist");
        let ambient_path = ambient_path.to_string_lossy().into_owned();
        let entry_path = entry_path.to_string_lossy().into_owned();
        let contexts = module_loader_context_sources(&ambient_path);
        assert_eq!(
            CompileOptions::default().module_loading_policy,
            ModuleLoadingPolicy::Filesystem,
            "ordinary engine callers must retain filesystem loading by default"
        );

        for (kind, source) in &contexts {
            let execution = observe_spec_exec_script_with_module_policy(
                source,
                &entry_path,
                ModuleLoadingPolicy::Filesystem,
            );
            assert!(
                matches!(&execution.0, ObservedCompletion::Normal(_)),
                "Filesystem {kind} control should complete: {execution:?}"
            );
            assert_eq!(
                &execution.1,
                &vec![
                    "ambient-module-body".to_string(),
                    "ambient-loaded".to_string()
                ],
                "Filesystem {kind} control did not execute the exact on-disk module: {execution:?}"
            );
        }

        let (root, runtime_contexts) = contexts
            .split_first()
            .expect("the root module-loader context should exist");
        let (root_kind, root_source) = root;
        let root_rejection = observe_spec_exec_script_with_module_policy(
            root_source,
            &entry_path,
            ModuleLoadingPolicy::RejectAll,
        );
        assert!(
            matches!(&root_rejection.0, ObservedCompletion::Normal(_)),
            "RejectAll {root_kind} import should settle through rejection: {root_rejection:?}"
        );
        assert_eq!(
            &root_rejection.1,
            &vec!["module-rejected".to_string()],
            "RejectAll {root_kind} import consulted the ambient module: {root_rejection:?}"
        );

        for protocol in [
            DifferentialProtocol::V1SelfCheckingNoOutput,
            DifferentialProtocol::V2PrimitiveCompletionNoOutput,
            DifferentialProtocol::V3PrimitiveCompletionPrintTranscript,
        ] {
            for (kind, source) in runtime_contexts {
                let case = DifferentialReplayInput::new_script(
                    "t25/source-closure/runtime-created-import",
                    protocol,
                    "differential/loader-context.js",
                    5_000,
                    source.clone(),
                )
                .unwrap_or_else(|error| panic!("{kind} native input: {error}"));
                let report =
                    replay_case(&case, SpecExecOracle::explicitly_enabled(), runner()).unwrap();
                let execution = report.spec_exec();
                assert!(
                    matches!(
                        &execution.execution,
                        ExecutionObservation::Normal { .. }
                            | ExecutionObservation::PrimitiveCompletion {
                                completion: PrimitiveCompletionObservation::Normal { .. },
                                ..
                            }
                    ),
                    "{protocol:?} {kind}: {execution:?}"
                );
                assert_eq!(
                    execution.output_events,
                    OutputEventsObservation::Captured {
                        events: vec!["module-rejected".to_string()]
                    },
                    "{protocol:?} {kind} consulted the ambient module"
                );
            }
        }

        std::fs::remove_dir_all(&directory).expect("ambient witness directory should be removed");
    }
}

struct TestDirectory {
    path: PathBuf,
}

impl TestDirectory {
    fn new(label: &str) -> Self {
        Self::in_directory(label, &std::env::temp_dir())
    }

    fn in_directory(label: &str, parent: &Path) -> Self {
        std::fs::create_dir_all(parent).expect("control staging parent should exist");
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let sequence = NEXT.fetch_add(1, Ordering::Relaxed);
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = parent.join(format!(
            "lila-differential-control-{label}-{}-{stamp}-{sequence}",
            std::process::id()
        ));
        std::fs::create_dir(&path).expect("unique control staging directory");
        Self { path }
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn selected_worker_digest() -> lila_engine::CompilerDigest {
    use sha2::{Digest, Sha256};
    use std::io::Read;
    let mut image = std::fs::File::open(env!("CARGO_BIN_EXE_lila-differential-worker")).unwrap();
    let mut hash = Sha256::new();
    let mut block = [0u8; 64 * 1024];
    loop {
        let length = image.read(&mut block).unwrap();
        if length == 0 {
            break;
        }
        hash.update(&block[..length]);
    }
    let spelling: String = hash
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    lila_engine::CompilerDigest::parse(&spelling).unwrap()
}

#[cfg(unix)]
#[test]
fn fresh_worker_pairs_keep_state_isolated_and_bind_the_selected_executable() {
    let input = DifferentialReplayInput::new_script(
        "t25/worker/fresh-state", DifferentialProtocol::V3PrimitiveCompletionPrintTranscript,
        "differential/worker-fresh-state.js", 5_000,
        "var previous = globalThis.workerState || 0; globalThis.workerState = previous + 1; print(String(previous)); previous;",
    ).unwrap();
    let selected = selected_worker_digest();
    let parent = CompilerProvenance::current().unwrap();
    assert_ne!(
        selected,
        parent.identity().executable_sha256(),
        "the Cargo worker is not the integration-test image"
    );
    for _ in 0..2 {
        let report = replay(&input);
        assert_eq!(report.case_id(), input.id());
        assert!(report.is_green(), "{}", report.to_pretty_json().unwrap());
        for observed in [report.wasm_aot(), report.spec_exec()] {
            assert_eq!(
                observed.output_events,
                OutputEventsObservation::Captured {
                    events: vec!["0".into()]
                }
            );
            assert!(matches!(&observed.execution,
                ExecutionObservation::PrimitiveCompletion {
                    completion: PrimitiveCompletionObservation::Normal { value: PrimitiveValueObservation::Number { bits } }, ..
                } if bits == "0000000000000000"
            ));
            let identity = observed
                .worker_identity
                .as_ref()
                .expect("validated worker header")
                .identity();
            assert_eq!(identity.executable_sha256(), selected);
            assert_eq!(
                identity.source_fingerprint(),
                parent.identity().source_fingerprint()
            );
            assert_eq!(
                identity.source_revision(),
                parent.identity().source_revision()
            );
        }
    }
}

#[cfg(unix)]
#[test]
fn a_nonterminating_spec_exec_worker_has_a_deadline_and_retains_only_committed_output() {
    let input = DifferentialReplayInput::new_script(
        "t25/worker/deadline-prefix",
        DifferentialProtocol::V3PrimitiveCompletionPrintTranscript,
        "differential/worker-deadline-prefix.js",
        5_000,
        "print('committed-prefix'); while (true) {}",
    )
    .unwrap();
    let _serial = serial_execution();
    let runner = runner();
    let start = std::time::Instant::now();
    let report = replay_case(&input, SpecExecOracle::explicitly_enabled(), runner).unwrap();
    assert!(
        start.elapsed() < std::time::Duration::from_secs(30),
        "both supervised attempts must terminate"
    );
    assert_eq!(report.verdict(), DifferentialVerdict::WorkerFailure);
    assert!(!report.is_green());
    assert!(report.mismatch_signature().is_none());
    assert!(matches!(
        &report.spec_exec().execution,
        ExecutionObservation::WorkerFailure {
            failure: DifferentialWorkerFailure::Timeout { timeout_ms: 5_000 },
            cleanup_error: None,
        }
    ));
    assert_eq!(
        report.spec_exec().output_events,
        OutputEventsObservation::Incomplete {
            events: vec!["committed-prefix".into()]
        }
    );
    assert!(report.spec_exec().worker_identity.is_some());
}

#[cfg(unix)]
#[test]
fn source_admission_occurs_in_the_worker_and_rejects_malformed_source() {
    let input = DifferentialReplayInput::new_script(
        "t25/worker/invalid-source",
        DifferentialProtocol::V1SelfCheckingNoOutput,
        "differential/worker-invalid-source.js",
        5_000,
        "function {",
    )
    .expect("native wire admission does not parse source");
    let _serial = serial_execution();
    let error = replay_case(&input, SpecExecOracle::explicitly_enabled(), runner()).unwrap_err();
    assert!(
        matches!(error, DifferentialError::InvalidCorpus(_)),
        "{error}"
    );
}

#[cfg(unix)]
#[test]
fn a_selected_nonworker_image_cannot_supply_a_semantic_completion() {
    let _serial = serial_execution();
    let wrong = DifferentialWorkerRunner::new(std::env::current_exe().unwrap()).unwrap();
    let report = replay_case(&case_v1(), SpecExecOracle::explicitly_enabled(), &wrong).unwrap();
    assert_eq!(report.verdict(), DifferentialVerdict::WorkerFailure);
    assert!(!report.is_green());
    assert!(report.mismatch_signature().is_none());
    for observed in [report.wasm_aot(), report.spec_exec()] {
        assert!(observed.worker_identity.is_none());
        assert_eq!(
            observed.output_events,
            OutputEventsObservation::Incomplete { events: Vec::new() }
        );
        assert!(matches!(
            &observed.execution,
            ExecutionObservation::WorkerFailure {
                cleanup_error: None,
                ..
            }
        ));
    }
}

#[cfg(unix)]
#[test]
fn failed_worker_campaigns_are_rejected_and_do_not_persist_a_mismatch() {
    use std::os::unix::fs::PermissionsExt;
    let _serial = serial_execution();
    let staging = TestDirectory::new("failed-image");
    let image = staging.path.join("nonexecutable-image");
    std::fs::write(&image, b"a readable file is not an executable worker\n").unwrap();
    std::fs::set_permissions(&image, std::fs::Permissions::from_mode(0o600)).unwrap();
    let failed_runner = DifferentialWorkerRunner::new(&image).unwrap();
    let plan = ArithmeticGenerationPlan::new(
        ArithmeticGrammar::IntegerArithmeticV1,
        ArithmeticGenerationSeed::new(1),
        ArithmeticCheckCount::new(1).unwrap(),
        ArithmeticExpressionDepth::new(1).unwrap(),
    );
    let outcome = run_generated_arithmetic_campaign(
        plan,
        ArithmeticReductionLimit::new(1).unwrap(),
        SpecExecOracle::explicitly_enabled(),
        &failed_runner,
    )
    .unwrap();
    let GeneratedArithmeticCampaignOutcome::Rejected { case, report } = outcome else {
        panic!("a failed worker cannot verify or reduce a mismatch");
    };
    assert_eq!(report.case_id(), case.id());
    assert_eq!(report.verdict(), DifferentialVerdict::WorkerFailure);
    assert!(!report.is_green());
    assert!(report.mismatch_signature().is_none());
    for observed in [report.wasm_aot(), report.spec_exec()] {
        assert!(matches!(
            &observed.execution,
            ExecutionObservation::WorkerFailure {
                failure: DifferentialWorkerFailure::Process { .. },
                cleanup_error: None
            }
        ));
        assert_eq!(
            observed.output_events,
            OutputEventsObservation::Incomplete { events: Vec::new() }
        );
    }
    let files = std::fs::read_dir(&staging.path)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    assert_eq!(
        files,
        vec![image],
        "no replayable mismatch corpus was published"
    );
}

#[cfg(not(unix))]
#[test]
fn unsupported_worker_platform_is_explicit_and_never_green() {
    let report = replay(&case_v1());
    assert_eq!(report.verdict(), DifferentialVerdict::WorkerFailure);
    assert!(!report.is_green());
    assert!(report.mismatch_signature().is_none());
    for observed in [report.wasm_aot(), report.spec_exec()] {
        assert!(matches!(
            &observed.execution,
            ExecutionObservation::WorkerFailure {
                failure: DifferentialWorkerFailure::UnsupportedPlatform,
                ..
            }
        ));
    }
}
