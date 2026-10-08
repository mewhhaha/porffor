use super::*;

#[derive(Debug)]
struct SilentHooks;
impl HostHooks for SilentHooks {}

fn observe(source: &str, limits: SnapshotLimits) -> ObservedGraphExecutionOutcome {
    observe_script_graph_with_module_loading_policy(
        source,
        Some("rooted-graph.js"),
        ModuleLoadingPolicy::RejectAll,
        &[],
        false,
        Arc::new(SilentHooks),
        limits,
    )
    .expect("script must reach an execution completion")
}

fn graph(outcome: &ObservedGraphExecutionOutcome) -> &RootedSnapshotGraph {
    match &outcome.completion.outcome {
        SnapshotOutcome::Captured { graph } => graph,
        SnapshotOutcome::Rejected { reason } => panic!("unexpected graph rejection: {reason}"),
    }
}

fn limits(realms: u32, properties: u32, utf16: u32, bigint: u32) -> SnapshotLimits {
    SnapshotLimits::new(512, 256, realms, properties, utf16, bigint, 262144, 64)
        .expect("bounded test limits")
}

fn string_key(property: &SnapshotProperty) -> Option<String> {
    match &property.key {
        SnapshotKey::String { units } => {
            Some(String::from_utf16(units.units()).expect("ASCII key"))
        }
        SnapshotKey::Symbol { .. } => None,
    }
}

fn data<'a>(node: &'a SnapshotNode, key: &str) -> &'a SnapshotValue {
    let property = node
        .properties
        .iter()
        .find(|property| string_key(property).as_deref() == Some(key))
        .expect("retained property");
    match &property.descriptor {
        SnapshotDescriptor::Data { value, .. } => value,
        SnapshotDescriptor::Accessor { .. } => panic!("expected a data descriptor"),
    }
}

#[test]
fn primitives_preserve_bits_utf16_and_exact_bigint_budget_boundary() {
    let negative_zero = observe("-0", SnapshotLimits::default());
    assert_eq!(
        graph(&negative_zero).root(),
        &SnapshotValue::Number {
            bits: (-0.0f64).to_bits()
        }
    );
    let nan = observe("NaN", SnapshotLimits::default());
    assert_eq!(
        graph(&nan).root(),
        &SnapshotValue::Number {
            bits: ObservedNumber::from_f64(f64::NAN).bits()
        }
    );
    let text = observe(r#"'a\ud800\u0000b'"#, SnapshotLimits::default());
    let SnapshotValue::String { units } = graph(&text).root() else {
        panic!("String root")
    };
    assert_eq!(units.units(), &[0x61, 0xd800, 0, 0x62]);

    for (source, bound, expected) in [
        ("9n", 1, "9"),
        ("-9n", 2, "-9"),
        ("99n", 2, "99"),
        ("0n", 1, "0"),
    ] {
        let outcome = observe(source, limits(16, 4096, 32768, bound));
        let SnapshotValue::BigInt { decimal } = graph(&outcome).root() else {
            panic!("BigInt root")
        };
        assert_eq!(decimal.decimal(), expected);
    }
    let refused = observe("10n", limits(16, 4096, 32768, 1));
    assert_eq!(
        refused.completion,
        SnapshotCompletion {
            kind: SnapshotCompletionKind::Normal,
            outcome: SnapshotOutcome::Rejected {
                reason: SnapshotRejection::BudgetExceeded {
                    dimension: SnapshotBudgetDimension::BigIntDigits
                }
            },
        }
    );
}

#[test]
fn cyclic_descriptors_symbols_and_accessors_use_raw_identity_without_calls() {
    let outcome = observe(
        r#"
        const o = Object.create(null);
        const local = Symbol('same'), distinct = Symbol('same');
        const get = () => { print('getter-called'); return 17; };
        const set = value => { print('setter-called'); };
        Object.setPrototypeOf(get, null); Object.setPrototypeOf(set, null);
        o.z = 1; o[10] = 10; o[2] = 2; o.self = o;
        o.local = local; o.distinct = distinct; o.registry = Symbol.for('registry');
        o.wellKnown = Symbol.iterator;
        Object.defineProperty(o, 'accessor', { get, set, enumerable: false, configurable: true });
        Object.defineProperty(o, local, { value: local, writable: false, enumerable: true, configurable: false });
        Object.preventExtensions(o); o;
    "#,
        SnapshotLimits::default(),
    );
    assert!(outcome.output_events.is_empty());
    let graph = graph(&outcome);
    assert_eq!(graph.root(), &SnapshotValue::Object { id: 0 });
    let node = &graph.nodes()[0];
    assert_eq!(node.kind, SnapshotObjectKind::Ordinary {});
    assert_eq!(node.prototype, SnapshotValue::Null {});
    assert!(!node.extensible);
    assert_eq!(
        node.properties
            .iter()
            .filter_map(string_key)
            .collect::<Vec<_>>(),
        [
            "2",
            "10",
            "z",
            "self",
            "local",
            "distinct",
            "registry",
            "wellKnown",
            "accessor"
        ]
    );
    assert_eq!(data(node, "self"), &SnapshotValue::Object { id: 0 });
    assert_eq!(data(node, "local"), &SnapshotValue::Symbol { id: 0 });
    assert_eq!(data(node, "distinct"), &SnapshotValue::Symbol { id: 1 });
    assert!(matches!(
        graph.symbols()[0].data.origin,
        SnapshotSymbolOrigin::Local {}
    ));
    assert!(matches!(
        graph.symbols()[1].data.origin,
        SnapshotSymbolOrigin::Local {}
    ));
    let SnapshotSymbolOrigin::Registry { key } = &graph.symbols()[2].data.origin else {
        panic!("registry identity")
    };
    assert_eq!(key.units(), "registry".encode_utf16().collect::<Vec<_>>());
    assert_eq!(
        graph.symbols()[3].data.origin,
        SnapshotSymbolOrigin::WellKnown {
            name: SnapshotWellKnownSymbol::Iterator
        }
    );
    let symbol_property = node
        .properties
        .last()
        .expect("Symbol property follows strings");
    assert_eq!(symbol_property.key, SnapshotKey::Symbol { id: 0 });
    assert_eq!(
        symbol_property.descriptor,
        SnapshotDescriptor::Data {
            value: SnapshotValue::Symbol { id: 0 },
            writable: false,
            enumerable: true,
            configurable: false,
        }
    );
    let accessor = &node.properties[8].descriptor;
    assert!(matches!(
        accessor,
        SnapshotDescriptor::Accessor {
            get: SnapshotValue::Object { .. },
            set: SnapshotValue::Object { .. },
            enumerable: false,
            configurable: true,
        }
    ));
    for function in &graph.nodes()[1..] {
        assert_eq!(
            function.kind,
            SnapshotObjectKind::Function {
                constructable: false,
                realm: 0
            }
        );
        assert_eq!(function.prototype, SnapshotValue::Null {});
    }
}

#[test]
fn sparse_array_retains_length_holes_and_self_alias() {
    let outcome = observe(
        r#"
        const array = []; Object.setPrototypeOf(array, null);
        array[2] = array; array.length = 5;
        Object.defineProperty(array, 'length', { writable: false });
        Object.preventExtensions(array); array;
    "#,
        SnapshotLimits::default(),
    );
    let graph = graph(&outcome);
    assert_eq!(graph.nodes().len(), 1);
    let array = &graph.nodes()[0];
    assert_eq!(array.kind, SnapshotObjectKind::Array {});
    assert!(!array.extensible);
    assert_eq!(
        array
            .properties
            .iter()
            .filter_map(string_key)
            .collect::<Vec<_>>(),
        ["2", "length"]
    );
    assert_eq!(data(array, "2"), &SnapshotValue::Object { id: 0 });
    assert_eq!(
        array.properties[1].descriptor,
        SnapshotDescriptor::Data {
            value: SnapshotValue::Number {
                bits: 5.0f64.to_bits()
            },
            writable: false,
            enumerable: false,
            configurable: false,
        }
    );
}

#[test]
fn actual_function_realm_and_intrinsic_anchors_survive_constructor_mutation() {
    let outcome = observe(
        r#"
        $262.createRealm();
        const other = $262.createRealm().global;
        const constructor = other.Object, prototype = constructor.prototype;
        for (const key of Reflect.ownKeys(constructor)) {
          if (key !== 'prototype' && key !== 'length' && key !== 'name') delete constructor[key];
        }
        for (const key of Reflect.ownKeys(prototype)) delete prototype[key];
        prototype.constructor = 17;
        Object.setPrototypeOf(constructor, null);
        Object.defineProperty(constructor, 'name', { value: 'misleading-current-Realm' });
        const root = Object.create(null); root.constructor = constructor; root.prototype = prototype; root;
    "#,
        SnapshotLimits::default(),
    );
    let graph = graph(&outcome);
    assert_eq!(
        graph.realm_count(),
        2,
        "encounter order excludes the unreferenced first child Realm"
    );
    let SnapshotValue::Object { id: function } = data(&graph.nodes()[0], "constructor") else {
        panic!("function identity")
    };
    let function = &graph.nodes()[*function as usize];
    assert_eq!(
        function.kind,
        SnapshotObjectKind::Function {
            constructable: true,
            realm: 1
        }
    );
    assert_eq!(
        function.anchors,
        [SnapshotAnchor {
            realm: 1,
            intrinsic: SnapshotIntrinsic::ObjectConstructor
        }]
    );
    let SnapshotValue::Object { id: prototype } = data(&graph.nodes()[0], "prototype") else {
        panic!("prototype identity")
    };
    let prototype = &graph.nodes()[*prototype as usize];
    assert_eq!(prototype.kind, SnapshotObjectKind::Ordinary {});
    assert_eq!(
        prototype.anchors,
        [SnapshotAnchor {
            realm: 1,
            intrinsic: SnapshotIntrinsic::ObjectPrototype
        }]
    );
    assert_eq!(
        data(prototype, "constructor"),
        &SnapshotValue::Number {
            bits: 17.0f64.to_bits()
        }
    );
}

#[test]
fn real_roots_remain_live_through_jobs_and_primary_throw_precedence() {
    for (tail, kind) in [
        ("root;", SnapshotCompletionKind::Normal),
        ("throw root;", SnapshotCompletionKind::Throw),
    ] {
        let source = format!(
            r#"
            const root = Object.create(null);
            Promise.resolve().then(() => {{ root.afterJobs = root; print('job'); throw 99; }});
            {tail}
        "#
        );
        let outcome = observe(&source, SnapshotLimits::default());
        assert_eq!(outcome.completion.kind, kind);
        assert_eq!(
            outcome.output_events,
            [HostOutputEvent::PrintLine("job".into())]
        );
        assert_eq!(
            data(&graph(&outcome).nodes()[0], "afterJobs"),
            &SnapshotValue::Object { id: 0 }
        );
        assert!(!outcome.note.contains("99"));
    }
    let module = observe_module_graph(
        r#"
        const root = Object.create(null);
        Promise.resolve().then(() => { root.afterJobs = root; print('module-job'); });
        throw root;
    "#,
        Some("rooted-graph.mjs"),
        ModuleHostConfig::default(),
        &[],
        false,
        Arc::new(SilentHooks),
        SnapshotLimits::default(),
    )
    .expect("module rejection is a completion");
    assert_eq!(module.completion.kind, SnapshotCompletionKind::Throw);
    assert_eq!(
        module.output_events,
        [HostOutputEvent::PrintLine("module-job".into())]
    );
    assert_eq!(
        data(&graph(&module).nodes()[0], "afterJobs"),
        &SnapshotValue::Object { id: 0 }
    );
}

#[test]
fn unsupported_exotics_and_private_brands_never_run_user_inspection_hooks() {
    for (source, kind, exotic) in [
        ("new Proxy({}, { ownKeys() { print('trap'); throw 1; }, getPrototypeOf() { print('trap'); throw 1; } })", SnapshotCompletionKind::Normal, SnapshotExotic::Proxy),
        ("throw new Proxy({}, { get() { print('trap'); throw 1; } })", SnapshotCompletionKind::Throw, SnapshotExotic::Proxy),
        ("new (class { #hidden = 1; })()", SnapshotCompletionKind::Normal, SnapshotExotic::Other),
        ("Promise.resolve(1)", SnapshotCompletionKind::Normal, SnapshotExotic::Promise),
        ("$262.IsHTMLDDA", SnapshotCompletionKind::Normal, SnapshotExotic::Other),
    ] {
        let outcome = observe(source, SnapshotLimits::default());
        assert_eq!(outcome.completion, SnapshotCompletion {
            kind, outcome: SnapshotOutcome::Rejected { reason: SnapshotRejection::UnsupportedExotic { exotic } },
        });
        assert!(outcome.output_events.is_empty(), "inspection must not invoke a user hook");
    }
}

#[test]
fn limits_reject_whole_observation_without_changing_completion_kind() {
    for (source, limits, dimension, kind) in [
        (
            "throw 'ab'",
            limits(16, 4096, 1, 4096),
            SnapshotBudgetDimension::Utf16Units,
            SnapshotCompletionKind::Throw,
        ),
        (
            "({ __proto__: null, a: 1, b: 2 })",
            limits(16, 1, 32768, 4096),
            SnapshotBudgetDimension::Properties,
            SnapshotCompletionKind::Normal,
        ),
        (
            "$262.createRealm(); 1",
            limits(1, 4096, 32768, 4096),
            SnapshotBudgetDimension::Realms,
            SnapshotCompletionKind::Normal,
        ),
    ] {
        let outcome = observe(source, limits);
        assert_eq!(
            outcome.completion,
            SnapshotCompletion {
                kind,
                outcome: SnapshotOutcome::Rejected {
                    reason: SnapshotRejection::BudgetExceeded { dimension }
                },
            }
        );
    }
    assert!(
        observe_script_graph(
            "let = ;",
            Some("invalid.js"),
            &[],
            false,
            Arc::new(SilentHooks),
            SnapshotLimits::default()
        )
        .is_err(),
        "parse failure is not a snapshot rejection"
    );
}
