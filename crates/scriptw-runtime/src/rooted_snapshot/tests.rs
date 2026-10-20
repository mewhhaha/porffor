use super::*;

#[derive(Clone)]
enum Value {
    Undefined,
    Null,
    Number(f64),
    Text(Vec<u16>),
    Object(usize),
    Symbol(usize),
}

struct Node {
    array: bool,
    realm: Option<usize>,
    properties: Vec<(String, Value)>,
}
struct Heap {
    nodes: Vec<Node>,
    entry: usize,
    reject: bool,
}

impl SnapshotBackend for Heap {
    type Value = Value;
    type Object = usize;
    type Symbol = usize;
    type Realm = usize;
    fn entry_realm(&self) -> usize {
        self.entry
    }
    fn same_object(&mut self, a: &usize, b: &usize) -> Result<bool, SnapshotRejection> {
        Ok(a == b)
    }
    fn same_symbol(&mut self, a: &usize, b: &usize) -> Result<bool, SnapshotRejection> {
        Ok(a == b)
    }
    fn same_realm(&mut self, a: &usize, b: &usize) -> Result<bool, SnapshotRejection> {
        Ok(a == b)
    }
    fn value(
        &mut self,
        value: &Value,
        budget: &mut SnapshotBudget,
    ) -> Result<SnapshotRawValue<usize, usize>, SnapshotRejection> {
        Ok(match value {
            Value::Undefined => SnapshotRawValue::Undefined,
            Value::Null => SnapshotRawValue::Null,
            Value::Number(number) => SnapshotRawValue::Number(number.to_bits()),
            Value::Text(text) => SnapshotRawValue::String(budget.copy_utf16(text)?),
            Value::Object(id) => SnapshotRawValue::Object(*id),
            Value::Symbol(id) => SnapshotRawValue::Symbol(*id),
        })
    }
    fn symbol(
        &mut self,
        _symbol: &usize,
        budget: &mut SnapshotBudget,
    ) -> Result<SnapshotSymbolData, SnapshotRejection> {
        Ok(SnapshotSymbolData {
            description: Some(budget.copy_utf16(&[0xd800])?),
            origin: SnapshotSymbolOrigin::Local {},
        })
    }
    fn object(
        &mut self,
        object: &usize,
        budget: &mut SnapshotBudget,
    ) -> Result<SnapshotObjectData<Value, usize, usize>, SnapshotRejection> {
        if self.reject {
            return Err(SnapshotRejection::UnsupportedExotic {
                exotic: SnapshotExotic::Proxy,
            });
        }
        let node = &self.nodes[*object];
        let mut properties = SnapshotProperties::default();
        for (key, value) in &node.properties {
            let length = node.array && key == "length";
            let key = budget.copy_utf16(&key.encode_utf16().collect::<Vec<_>>())?;
            properties.push(
                budget,
                SnapshotRawProperty {
                    key: SnapshotRawKey::String(key),
                    descriptor: SnapshotRawDescriptor::Data {
                        value: value.clone(),
                        writable: true,
                        enumerable: !length,
                        configurable: !length,
                    },
                },
            )?;
        }
        Ok(SnapshotObjectData {
            kind: match node.realm {
                Some(realm) => SnapshotRawObjectKind::Function {
                    constructable: false,
                    realm,
                },
                None if node.array => SnapshotRawObjectKind::Array,
                None => SnapshotRawObjectKind::Ordinary,
            },
            extensible: true,
            prototype: Value::Null,
            anchors: Vec::new(),
            properties,
        })
    }
}

fn node(properties: &[(&str, Value)]) -> Node {
    Node {
        array: false,
        realm: None,
        properties: properties
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect(),
    }
}
fn capture(heap: &mut Heap, value: Value) -> RootedSnapshotGraph {
    match capture_snapshot(heap, &value, SnapshotLimits::default()) {
        SnapshotOutcome::Captured { graph } => graph,
        rejected => panic!("{rejected:?}"),
    }
}
fn limits(nodes: u32, units: u32) -> SnapshotLimits {
    SnapshotLimits::new(nodes, 8, 8, 32, units, 32, 4096, 16).unwrap()
}

#[test]
fn cyclic_aliases_and_same_description_symbols_keep_distinct_canonical_identities() {
    let mut heap = Heap {
        entry: 88,
        reject: false,
        nodes: vec![
            node(&[
                ("self", Value::Object(0)),
                ("a", Value::Object(1)),
                ("b", Value::Object(1)),
                ("first", Value::Symbol(50)),
                ("again", Value::Symbol(50)),
                ("other", Value::Symbol(60)),
            ]),
            node(&[("parent", Value::Object(0))]),
        ],
    };
    let graph = capture(&mut heap, Value::Object(0));
    assert_eq!(graph.nodes().len(), 2);
    assert_eq!(graph.symbols().len(), 2);
    assert_eq!(graph.symbols()[0].data, graph.symbols()[1].data);
    let value = |index: usize| match &graph.nodes()[0].properties[index].descriptor {
        SnapshotDescriptor::Data { value, .. } => value,
        SnapshotDescriptor::Accessor { .. } => unreachable!(),
    };
    assert_eq!(value(1), value(2));
    assert_eq!(value(3), value(4));
    assert_ne!(value(3), value(5));
    let wire = serde_json::to_vec(&graph).unwrap();
    assert_eq!(
        serde_json::from_slice::<RootedSnapshotGraph>(&wire).unwrap(),
        graph
    );
}

#[test]
fn realm_ids_follow_encounters_and_keep_entry_separate() {
    let make = |entry, first, second| {
        let mut first_function = node(&[]);
        first_function.realm = Some(first);
        let mut second_function = node(&[]);
        second_function.realm = Some(second);
        Heap {
            entry,
            reject: false,
            nodes: vec![
                node(&[("a", Value::Object(1)), ("b", Value::Object(2))]),
                first_function,
                second_function,
            ],
        }
    };
    let graph = capture(&mut make(90, 44, 33), Value::Object(0));
    assert_eq!(graph, capture(&mut make(8, 2, 7), Value::Object(0)));
    assert_eq!(graph.realm_count(), 3);
    assert_ne!(graph, capture(&mut make(8, 2, 2), Value::Object(0)));
    assert_ne!(graph, capture(&mut make(8, 8, 7), Value::Object(0)));
}

#[test]
fn adjacent_copy_and_node_budgets_preserve_exact_boundary_and_cycles() {
    let mut heap = Heap {
        entry: 0,
        reject: false,
        nodes: vec![node(&[("x", Value::Object(0))])],
    };
    let text = Value::Text(vec![97, 0xd800, 0xdc00, 98]);
    assert!(matches!(
        capture_snapshot(&mut heap, &text, limits(1, 4)),
        SnapshotOutcome::Captured { .. }
    ));
    assert_eq!(
        capture_snapshot(&mut heap, &text, limits(1, 3)),
        SnapshotOutcome::Rejected {
            reason: SnapshotRejection::BudgetExceeded {
                dimension: SnapshotBudgetDimension::Utf16Units
            },
        }
    );
    assert!(matches!(
        capture_snapshot(&mut heap, &Value::Object(0), limits(1, 32)),
        SnapshotOutcome::Captured { .. }
    ));
    heap.nodes[0]
        .properties
        .push(("y".into(), Value::Object(1)));
    heap.nodes.push(node(&[]));
    assert_eq!(
        capture_snapshot(&mut heap, &Value::Object(0), limits(1, 32)),
        SnapshotOutcome::Rejected {
            reason: SnapshotRejection::BudgetExceeded {
                dimension: SnapshotBudgetDimension::Nodes
            },
        }
    );
}

#[test]
fn arrays_keep_holes_and_nondefault_descriptors() {
    let mut array = node(&[("1", Value::Undefined), ("length", Value::Number(3.0))]);
    array.array = true;
    let mut heap = Heap {
        entry: 0,
        reject: false,
        nodes: vec![array],
    };
    let holes = capture(&mut heap, Value::Object(0));
    heap.nodes[0]
        .properties
        .insert(0, ("0".into(), Value::Undefined));
    let filled = capture(&mut heap, Value::Object(0));
    assert_ne!(holes, filled);
    let mut forged = serde_json::to_value(&filled).unwrap();
    forged["nodes"][0]["properties"][2]["descriptor"]["enumerable"] = true.into();
    assert!(serde_json::from_value::<RootedSnapshotGraph>(forged).is_err());
}

#[test]
fn wire_rejects_dangling_noncanonical_unused_and_unbounded_state() {
    let mut heap = Heap {
        entry: 0,
        reject: false,
        nodes: vec![node(&[("self", Value::Object(0))])],
    };
    let graph = capture(&mut heap, Value::Object(0));
    let wire = serde_json::to_value(&graph).unwrap();
    for (path, value) in [
        ("root", serde_json::json!({"type":"object","id":1})),
        ("realm_count", 2.into()),
        ("version", 2.into()),
        ("extra", true.into()),
    ] {
        let mut damaged = wire.clone();
        damaged[path] = value;
        assert!(
            serde_json::from_value::<RootedSnapshotGraph>(damaged).is_err(),
            "{path}"
        );
    }
    let mut damaged = wire.clone();
    damaged["nodes"][0]["id"] = 7.into();
    assert!(serde_json::from_value::<RootedSnapshotGraph>(damaged).is_err());
    let mut damaged = wire;
    damaged["limits"]["nodes"] = 4097.into();
    assert!(serde_json::from_value::<RootedSnapshotGraph>(damaged).is_err());
    assert!(SnapshotLimits::new(0, 1, 1, 1, 1, 1, 1, 1).is_err());
}

#[test]
fn unsupported_exotic_is_rejection_and_not_an_empty_successful_graph() {
    let mut heap = Heap {
        entry: 0,
        reject: true,
        nodes: Vec::new(),
    };
    assert_eq!(
        capture_snapshot(&mut heap, &Value::Object(0), SnapshotLimits::default()),
        SnapshotOutcome::Rejected {
            reason: SnapshotRejection::UnsupportedExotic {
                exotic: SnapshotExotic::Proxy
            },
        }
    );
}

#[test]
fn number_bits_keep_negative_zero_and_normalize_nan_payloads() {
    let mut heap = Heap {
        entry: 0,
        reject: false,
        nodes: Vec::new(),
    };
    assert_ne!(
        capture(&mut heap, Value::Number(0.0)),
        capture(&mut heap, Value::Number(-0.0))
    );
    assert_eq!(
        capture(&mut heap, Value::Number(f64::NAN)),
        capture(
            &mut heap,
            Value::Number(f64::from_bits(0x7ff0_0000_0000_0001))
        )
    );
}

#[test]
fn tagged_empty_rows_reject_foreign_payload_fields() {
    for value_type in ["undefined", "null"] {
        assert!(serde_json::from_value::<SnapshotValue>(
            serde_json::json!({"type": value_type, "foreign": 1})
        )
        .is_err());
    }
    for object_kind in ["ordinary", "array"] {
        assert!(serde_json::from_value::<SnapshotObjectKind>(
            serde_json::json!({"kind": object_kind, "realm": 0})
        )
        .is_err());
    }
    assert!(serde_json::from_value::<SnapshotSymbolOrigin>(
        serde_json::json!({"kind": "local", "key": []})
    )
    .is_err());
}
