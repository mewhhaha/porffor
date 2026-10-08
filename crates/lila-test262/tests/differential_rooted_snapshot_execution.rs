//! Actual v7 executions use the selected bounded worker and untouched source.
#![cfg(all(feature = "spec-exec-oracle", unix))]

use lila_runtime::rooted_snapshot::{
    SnapshotDescriptor, SnapshotKey, SnapshotObjectKind, SnapshotSymbolOrigin, SnapshotValue,
};
use lila_test262::differential::*;
use std::sync::{Mutex, OnceLock};

static EXECUTION: Mutex<()> = Mutex::new(());

fn replay(input: &DifferentialReplayInput) -> DifferentialReport {
    static RUNNER: OnceLock<DifferentialWorkerRunner> = OnceLock::new();
    let _serial = EXECUTION.lock().unwrap();
    let runner = RUNNER.get_or_init(|| {
        DifferentialWorkerRunner::new(env!("CARGO_BIN_EXE_lila-differential-worker")).unwrap()
    });
    replay_case(input, SpecExecOracle::explicitly_enabled(), runner).unwrap()
}

fn property<'a>(graph: &'a RootedSnapshotGraph, object: u32, key: &str) -> &'a SnapshotDescriptor {
    let units: Vec<_> = key.encode_utf16().collect();
    &graph.nodes()[object as usize].properties.iter().find(|property| {
        matches!(&property.key, SnapshotKey::String { units: actual } if actual.units() == units)
    }).unwrap().descriptor
}

#[test]
fn rooted_script_replay_preserves_post_job_cycles_symbols_holes_and_noninvoked_accessors() {
    let source = r#"
let root = Object.create(null);
root.self = root;
const values = [, undefined, -0];
Object.setPrototypeOf(values, null);
root.values = values;
const local = Symbol('\ud800');
root.local = local;
root.alias = local;
root.registry = Symbol.for('\ud801');
root.known = Symbol.iterator;
root[local] = root;
const accessor = () => { print('accessor-ran'); throw 9; };
Object.setPrototypeOf(accessor, null);
Object.defineProperty(root, 'accessor', { get: accessor, enumerable: false, configurable: true });
root.native = Object.keys;
Object.setPrototypeOf(root.native, null);
print('before-job');
Promise.resolve().then(() => { root.after = 7; root = null; gc(); print('after-job'); });
root;
"#;
    let input = DifferentialReplayInput::new_snapshot_script(
        "t25/rooted/post-job",
        "differential/rooted-post-job.js",
        120_000,
        source,
        SnapshotLimits::default(),
    )
    .unwrap();
    let native = input.to_pretty_json().unwrap();
    assert_eq!(
        DifferentialReplayInput::from_json(&native)
            .unwrap()
            .source(),
        source
    );
    let report = replay(&input);
    assert_eq!(
        report.verdict(),
        DifferentialVerdict::RootedCompletionGraphAndPrintTranscriptMatch,
        "{}",
        report.to_pretty_json().unwrap()
    );
    for backend in [report.wasm_aot(), report.spec_exec()] {
        assert_eq!(
            backend.output_events,
            OutputEventsObservation::Captured {
                events: vec!["before-job".into(), "after-job".into()]
            }
        );
        let ExecutionObservation::RootedCompletionGraph { completion, .. } = &backend.execution
        else {
            panic!("missing native snapshot")
        };
        assert_eq!(completion.kind, SnapshotCompletionKind::Normal);
        let SnapshotOutcome::Captured { graph } = &completion.outcome else {
            panic!("rooted capture rejected")
        };
        assert_eq!(graph.root(), &SnapshotValue::Object { id: 0 });
        assert!(matches!(
            property(graph, 0, "self"),
            SnapshotDescriptor::Data {
                value: SnapshotValue::Object { id: 0 },
                ..
            }
        ));
        assert!(
            matches!(property(graph, 0, "after"), SnapshotDescriptor::Data { value: SnapshotValue::Number { bits }, .. } if *bits == 7.0_f64.to_bits())
        );
        let SnapshotDescriptor::Data {
            value: SnapshotValue::Object { id: array },
            ..
        } = property(graph, 0, "values")
        else {
            panic!("missing Array identity")
        };
        assert_eq!(
            graph.nodes()[*array as usize].kind,
            SnapshotObjectKind::Array {}
        );
        assert!(matches!(
            property(graph, *array, "1"),
            SnapshotDescriptor::Data {
                value: SnapshotValue::Undefined {},
                ..
            }
        ));
        assert!(
            matches!(property(graph, *array, "2"), SnapshotDescriptor::Data { value: SnapshotValue::Number { bits }, .. } if *bits == (-0.0_f64).to_bits())
        );
        assert!(!graph.nodes()[*array as usize].properties.iter().any(|property| matches!(&property.key, SnapshotKey::String { units } if units.units() == [48])));
        assert_eq!(property(graph, 0, "local"), property(graph, 0, "alias"));
        assert!(graph.symbols().iter().any(|symbol| symbol
            .data
            .description
            .as_ref()
            .is_some_and(|description| description.units() == [0xd800])
            && symbol.data.origin == (SnapshotSymbolOrigin::Local {})));
        assert!(graph.symbols().iter().any(|symbol| matches!(&symbol.data.origin, SnapshotSymbolOrigin::Registry { key } if key.units() == [0xd801])));
        assert!(matches!(
            property(graph, 0, "accessor"),
            SnapshotDescriptor::Accessor {
                get: SnapshotValue::Object { .. },
                set: SnapshotValue::Undefined {},
                enumerable: false,
                configurable: true
            }
        ));
    }
}

#[test]
fn rooted_embedded_module_replay_keeps_original_goal_and_post_job_output() {
    let input = DifferentialReplayInput::from_json(&serde_json::json!({
        "schema_version":7,"id":"t25/rooted/module","observation_contract":"rooted_completion_graph_print_transcript",
        "host_profile":"test262","timeout_ms":120_000,"snapshot_limits":SnapshotLimits::default(),
        "program":{"kind":"embedded_graph","module_graph":{
            "entry":{"goal":"module","identity":"entry.mjs","source":"await 0; print('module-after-await'); export const value = 3;","meta_url":"lila://rooted/entry.mjs"},
            "modules":[],"resolutions":[]
        }}
    }).to_string()).unwrap();
    let report = replay(&input);
    assert_eq!(
        report.verdict(),
        DifferentialVerdict::RootedCompletionGraphAndPrintTranscriptMatch,
        "{}",
        report.to_pretty_json().unwrap()
    );
    for backend in [report.wasm_aot(), report.spec_exec()] {
        assert_eq!(
            backend.output_events,
            OutputEventsObservation::Captured {
                events: vec!["module-after-await".into()]
            }
        );
        assert!(
            matches!(&backend.execution, ExecutionObservation::RootedCompletionGraph { completion: SnapshotCompletion { kind: SnapshotCompletionKind::Normal, outcome: SnapshotOutcome::Captured { graph } }, .. } if graph.root() == &(SnapshotValue::Undefined {}))
        );
    }
}
