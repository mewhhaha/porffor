use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy, RealmBuilder,
    RunOptions, SnapshotLimits, SnapshotOutcome,
};
use lila_runtime::rooted_snapshot::{
    SnapshotBudgetDimension, SnapshotCompletionKind, SnapshotDescriptor, SnapshotIntrinsic,
    SnapshotKey, SnapshotObjectKind, SnapshotRejection, SnapshotSymbolOrigin, SnapshotValue,
    SnapshotWellKnownSymbol,
};

fn observe(source: &str, limits: SnapshotLimits) -> lila_engine::GraphRunOutcome {
    lila_engine::configure_compilation_jobs(1).unwrap();
    Engine::new(RealmBuilder::new().build())
        .observe_script_graph(
            source,
            CompileOptions {
                host_surface_policy: HostSurfacePolicy::Test262,
                ..CompileOptions::default()
            },
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(120_000),
                ..RunOptions::default()
            },
            limits,
        )
        .unwrap()
}

fn property<'a>(
    node: &'a lila_runtime::rooted_snapshot::SnapshotNode,
    name: &str,
) -> &'a SnapshotDescriptor {
    let units: Vec<_> = name.encode_utf16().collect();
    &node
        .properties
        .iter()
        .find(|property| {
            matches!(&property.key,
        SnapshotKey::String { units: key } if key.units() == units)
        })
        .unwrap()
        .descriptor
}

#[test]
fn property_growth_snapshot_preserves_live_insertion_extent_without_spare_slots() {
    let outcome = observe(
        r#"
var object = Object.create(null);
for (var i = 0; i < 65; i++) object['key' + i] = i;
delete object.key0; delete object.key32; delete object.key64;
object.key0 = 1000;
object['2'] = 2; object['1'] = 1;
object[Symbol('last')] = object;
gc();
object;
"#,
        SnapshotLimits::default(),
    );
    let SnapshotOutcome::Captured { graph } = outcome.completion.outcome else {
        panic!("graph was rejected: {:?}", outcome.completion.outcome);
    };
    assert_eq!(graph.root(), &SnapshotValue::Object { id: 0 });
    let root = &graph.nodes()[0];
    assert_eq!(root.properties.len(), 66);
    let names = root
        .properties
        .iter()
        .filter_map(|property| match &property.key {
            SnapshotKey::String { units } => Some(String::from_utf16(units.units()).unwrap()),
            SnapshotKey::Symbol { .. } => None,
        })
        .collect::<Vec<_>>();
    let expected = ["1".into(), "2".into()]
        .into_iter()
        .chain(
            (1..64)
                .filter(|index| *index != 32)
                .map(|index| format!("key{index}")),
        )
        .chain(["key0".into()])
        .collect::<Vec<String>>();
    assert_eq!(names, expected);
    assert!(matches!(
        root.properties.last().unwrap().key,
        SnapshotKey::Symbol { .. }
    ));
    assert!(matches!(
        root.properties.last().unwrap().descriptor,
        SnapshotDescriptor::Data {
            value: SnapshotValue::Object { id: 0 },
            ..
        }
    ));
}

#[test]
fn original_throw_is_rooted_through_jobs_without_running_getters_or_symbol_hooks() {
    let outcome = observe(
        r#"
let root = Object.create(null);
const local = Symbol('same'), second = Symbol('same'), registered = Symbol.for('registry');
Object.defineProperty(root, 'accessor', { get() { throw 'snapshot called getter'; }, enumerable: true });
root.self = root;
root.array = [, undefined, -0];
Object.defineProperty(root.array, '2', { writable: false, configurable: false });
root.local = local; root.again = local; root.second = second;
root.registered = registered; root.wellKnown = Symbol.iterator;
root[local] = 'symbol property';
root.fn = function(value) { return value; }; root.native = Object;
Promise.resolve().then(() => { root.afterJobs = 7; root = null; gc(); print('jobs settled'); });
throw root;
"#,
        SnapshotLimits::default(),
    );
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert_eq!(outcome.completion.kind, SnapshotCompletionKind::Throw);
    assert_eq!(
        outcome.output_events,
        vec![HostOutputEvent::PrintLine("jobs settled".into())]
    );
    let SnapshotOutcome::Captured { graph } = outcome.completion.outcome else {
        panic!("graph was rejected: {:?}", outcome.completion.outcome);
    };
    assert_eq!(graph.root(), &SnapshotValue::Object { id: 0 });
    let root = &graph.nodes()[0];
    assert_eq!(root.prototype, SnapshotValue::Null {});
    assert!(matches!(
        property(root, "self"),
        SnapshotDescriptor::Data {
            value: SnapshotValue::Object { id: 0 },
            ..
        }
    ));
    assert!(
        matches!(property(root, "afterJobs"), SnapshotDescriptor::Data { value: SnapshotValue::Number { bits }, .. } if *bits == 7f64.to_bits())
    );
    assert!(matches!(
        property(root, "accessor"),
        SnapshotDescriptor::Accessor {
            get: SnapshotValue::Object { .. },
            set: SnapshotValue::Undefined {},
            ..
        }
    ));
    let symbol = |name| match property(root, name) {
        SnapshotDescriptor::Data {
            value: SnapshotValue::Symbol { id },
            ..
        } => *id,
        _ => panic!("missing Symbol"),
    };
    assert_eq!(symbol("local"), symbol("again"));
    assert_ne!(symbol("local"), symbol("second"));
    assert!(
        matches!(&graph.symbols()[symbol("registered") as usize].data.origin, SnapshotSymbolOrigin::Registry { key } if key.units() == "registry".encode_utf16().collect::<Vec<_>>())
    );
    assert_eq!(
        graph.symbols()[symbol("wellKnown") as usize].data.origin,
        SnapshotSymbolOrigin::WellKnown {
            name: SnapshotWellKnownSymbol::Iterator
        }
    );
    let array_id = match property(root, "array") {
        SnapshotDescriptor::Data {
            value: SnapshotValue::Object { id },
            ..
        } => *id,
        _ => panic!("missing Array"),
    };
    let array = &graph.nodes()[array_id as usize];
    assert_eq!(array.kind, SnapshotObjectKind::Array {});
    assert!(!array.properties.iter().any(
        |property| matches!(&property.key, SnapshotKey::String { units } if units.units() == [48])
    ));
    assert!(matches!(
        property(array, "1"),
        SnapshotDescriptor::Data {
            value: SnapshotValue::Undefined {},
            ..
        }
    ));
    assert!(
        matches!(property(array, "2"), SnapshotDescriptor::Data { value: SnapshotValue::Number { bits }, writable: false, configurable: false, .. } if *bits == (-0f64).to_bits())
    );
    assert!(matches!(
        property(array, "length"),
        SnapshotDescriptor::Data {
            enumerable: false,
            configurable: false,
            ..
        }
    ));
    assert!(graph.nodes().iter().any(|node| matches!(
        node.kind,
        SnapshotObjectKind::Function {
            constructable: true,
            realm: 0
        }
    )));
}

#[test]
fn created_realm_anchors_follow_retained_intrinsic_records_and_error_graphs_are_ordinary() {
    let outcome = observe(
        r#"
const foreign = __lilaCreateRealm().global;
const root = Object.create(null);
root.object = new foreign.Object();
root.function = foreign.Object;
root.error = new foreign.TypeError('foreign');
root.entry = Object.prototype;
foreign.Object = 17;
throw root;
"#,
        SnapshotLimits::default(),
    );
    assert_eq!(outcome.completion.kind, SnapshotCompletionKind::Throw);
    let SnapshotOutcome::Captured { graph } = outcome.completion.outcome else {
        panic!("graph was rejected: {:?}", outcome.completion.outcome);
    };
    assert_eq!(graph.realm_count(), 2);
    let anchor = |intrinsic, realm| {
        graph.nodes().iter().any(|node| {
            node.anchors
                .iter()
                .any(|anchor| anchor.intrinsic == intrinsic && anchor.realm == realm)
        })
    };
    assert!(anchor(SnapshotIntrinsic::ObjectPrototype, 0));
    assert!(anchor(SnapshotIntrinsic::ObjectConstructor, 1));
    assert!(anchor(SnapshotIntrinsic::ObjectPrototype, 1));
    assert!(anchor(SnapshotIntrinsic::TypeErrorPrototype, 1));
    let root = &graph.nodes()[0];
    let error = match property(root, "error") {
        SnapshotDescriptor::Data {
            value: SnapshotValue::Object { id },
            ..
        } => &graph.nodes()[*id as usize],
        _ => panic!("missing Error"),
    };
    assert_eq!(error.kind, SnapshotObjectKind::Ordinary {});
    assert!(
        matches!(property(error, "message"), SnapshotDescriptor::Data { value: SnapshotValue::String { units }, .. } if units.units() == "foreign".encode_utf16().collect::<Vec<_>>())
    );
}

#[test]
fn unsupported_exotic_and_copy_budget_are_execution_observations() {
    let outcome = observe(
        "throw new Proxy({}, { ownKeys() { throw 'hook'; } });",
        SnapshotLimits::default(),
    );
    assert_eq!(outcome.completion.kind, SnapshotCompletionKind::Throw);
    assert!(matches!(
        outcome.completion.outcome,
        SnapshotOutcome::Rejected {
            reason: SnapshotRejection::UnsupportedExotic { .. }
        }
    ));
    let limits = SnapshotLimits::new(8, 8, 2, 16, 4, 16, 4096, 8).unwrap();
    let outcome = observe("'abcde'", limits);
    assert_eq!(outcome.completion.kind, SnapshotCompletionKind::Normal);
    assert_eq!(
        outcome.completion.outcome,
        SnapshotOutcome::Rejected {
            reason: SnapshotRejection::BudgetExceeded {
                dimension: SnapshotBudgetDimension::Utf16Units
            }
        }
    );
    let outcome = observe("'\\uD800abc'", limits);
    let SnapshotOutcome::Captured { graph } = outcome.completion.outcome else {
        panic!("exact cap rejected");
    };
    assert!(
        matches!(graph.root(), SnapshotValue::String { units } if units.units() == [0xd800, 97, 98, 99])
    );
}
