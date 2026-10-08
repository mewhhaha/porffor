//! Selected probes execute ordinary reflection through both real backends.
//! Only a bounded, validated canonical graph can reach comparison or a report.

use std::collections::HashSet;

use serde::{Deserialize, Deserializer, Serialize};

use super::*;

const CAPTURE_SOURCE: &str = include_str!("object_probe_capture.js");
const MAX_WIRE_BYTES: usize = 131_072;
const MAX_NODES: usize = 512;
const MAX_SYMBOLS: usize = 256;
const MAX_KEYS: usize = 4096;
const MAX_UNITS: usize = 32_768;

pub(super) fn execution_source(body: &str) -> String {
    format!("{CAPTURE_SOURCE}(function () {{\n{body}\n}});\n")
}

pub(super) fn validate_body(body: &str) -> Result<(), DifferentialError> {
    // The existing frontend parses a FunctionBody in isolation, so corpus
    // text cannot close the callback and replace the capture invocation.
    // This is host compilation admission; emitted Wasm contains no parser.
    lila_front::prepare_dynamic_function(
        lila_front::FunctionParseKind::Ordinary,
        &[body.to_string()],
    )
    .map(|_| ())
    .map_err(|error| {
        DifferentialError::InvalidCorpus(format!("invalid selected probe FunctionBody: {error}"))
    })
}

pub(super) fn capture_source_bytes() -> &'static [u8] {
    CAPTURE_SOURCE.as_bytes()
}

/// IDs name graph-local first encounters, never backend addresses. Its only
/// constructor checks bounds, closed value domains and canonical reachability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct SelectedObjectProbeGraph(GraphWire);

impl<'de> Deserialize<'de> for SelectedObjectProbeGraph {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::validate(GraphWire::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct GraphWire {
    version: u32,
    roots: Vec<NamedValue>,
    anchors: Vec<NamedValue>,
    nodes: Vec<Node>,
    symbols: Vec<Symbol>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct NamedValue {
    name: String,
    value: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum Value {
    Undefined,
    Null,
    Boolean { value: bool },
    Number { bits: String },
    NumberDecimal { decimal: String },
    String { units: Vec<u16> },
    BigInt { decimal: String },
    Object { id: usize },
    Symbol { id: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ObjectKind {
    Object,
    Array,
    Function,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Node {
    id: usize,
    kind: ObjectKind,
    extensible: bool,
    prototype: Value,
    properties: Vec<Property>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum Key {
    String { units: Vec<u16> },
    Symbol { id: usize },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Property {
    key: Key,
    descriptor: Descriptor,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Descriptor {
    Data {
        value: Value,
        writable: bool,
        enumerable: bool,
        configurable: bool,
    },
    Accessor {
        get: Value,
        set: Value,
        enumerable: bool,
        configurable: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Symbol {
    id: usize,
    description: Option<Vec<u16>>,
    origin: SymbolOrigin,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum SymbolOrigin {
    Local,
    Registry { key: Vec<u16> },
    WellKnown { name: WellKnownSymbol },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum WellKnownSymbol {
    AsyncIterator,
    HasInstance,
    IsConcatSpreadable,
    Iterator,
    Match,
    MatchAll,
    Replace,
    Search,
    Species,
    Split,
    ToPrimitive,
    ToStringTag,
    Unscopables,
    Dispose,
    AsyncDispose,
}

struct Admission<'a> {
    kinds: &'a [ObjectKind],
    seen_nodes: Vec<bool>,
    seen_symbols: Vec<bool>,
    next_node: usize,
    next_symbol: usize,
    units: usize,
}

impl Admission<'_> {
    fn units(&mut self, units: &[u16]) -> Result<(), &'static str> {
        self.units = self
            .units
            .checked_add(units.len())
            .ok_or("UTF-16 budget overflow")?;
        if self.units > MAX_UNITS {
            return Err("UTF-16 budget exceeded");
        }
        Ok(())
    }

    fn reference(id: usize, seen: &mut [bool], next: &mut usize) -> Result<(), &'static str> {
        let entry = seen.get_mut(id).ok_or("dangling graph reference")?;
        if !*entry {
            if id != *next {
                return Err("noncanonical first-encounter identity");
            }
            *entry = true;
            *next += 1;
        }
        Ok(())
    }

    fn value(&mut self, value: &mut Value) -> Result<(), &'static str> {
        match value {
            Value::Object { id } => Self::reference(*id, &mut self.seen_nodes, &mut self.next_node),
            Value::Symbol { id } => {
                Self::reference(*id, &mut self.seen_symbols, &mut self.next_symbol)
            }
            Value::String { units } => self.units(units),
            Value::NumberDecimal { decimal } => {
                let number = match decimal.as_str() {
                    "NaN" => f64::NAN,
                    "Infinity" => f64::INFINITY,
                    "-Infinity" => f64::NEG_INFINITY,
                    text if text.len() <= 32
                        && text
                            .bytes()
                            .all(|c| c.is_ascii_digit() || b"-.e+".contains(&c)) =>
                    {
                        let number = text.parse::<f64>().map_err(|_| "invalid Number decimal")?;
                        if !number.is_finite() {
                            return Err("nonfinite Number decimal");
                        }
                        number
                    }
                    _ => return Err("invalid Number decimal"),
                };
                *value = Value::Number {
                    bits: format!(
                        "{:016x}",
                        lila_engine::ObservedNumber::from_f64(number).bits()
                    ),
                };
                Ok(())
            }
            Value::Number { bits } => {
                if bits.len() != 16
                    || !bits
                        .bytes()
                        .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
                {
                    return Err("invalid Number bits");
                }
                let raw = u64::from_str_radix(bits, 16).map_err(|_| "invalid Number bits")?;
                if lila_engine::ObservedNumber::from_bits(raw).bits() != raw {
                    return Err("noncanonical NaN");
                }
                Ok(())
            }
            Value::BigInt { decimal } => {
                if decimal.len() > MAX_WIRE_BYTES
                    || lila_engine::ObservedBigInt::parse_canonical_decimal(
                        decimal.clone().into_boxed_str(),
                    )
                    .is_err()
                {
                    return Err("noncanonical BigInt");
                }
                Ok(())
            }
            Value::Undefined | Value::Null | Value::Boolean { .. } => Ok(()),
        }
    }

    fn accessor(&mut self, value: &mut Value) -> Result<(), &'static str> {
        match value {
            Value::Undefined => {}
            Value::Object { id } if self.kinds.get(*id) == Some(&ObjectKind::Function) => {}
            _ => return Err("accessor must be undefined or callable"),
        }
        self.value(value)
    }

    fn named(&mut self, values: &mut [NamedValue], anchors: bool) -> Result<(), &'static str> {
        let mut names = HashSet::new();
        for named in values {
            if named.name.is_empty()
                || named.name.len() > 64
                || !named
                    .name
                    .bytes()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || b"-_".contains(&c))
                || !names.insert(named.name.clone())
            {
                return Err("invalid or repeated root/anchor name");
            }
            if anchors && !matches!(named.value, Value::Object { .. } | Value::Symbol { .. }) {
                return Err("anchor must name an observed identity");
            }
            self.value(&mut named.value)?;
        }
        Ok(())
    }
}

impl SelectedObjectProbeGraph {
    fn validate(mut graph: GraphWire) -> Result<Self, &'static str> {
        if graph.version != 1
            || graph.roots.is_empty()
            || graph.roots.len() > 16
            || graph.anchors.len() > 16
            || graph.nodes.len() > MAX_NODES
            || graph.symbols.len() > MAX_SYMBOLS
        {
            return Err("unsupported graph version or graph bound exceeded");
        }
        let kinds: Vec<_> = graph.nodes.iter().map(|node| node.kind).collect();
        let mut admission = Admission {
            kinds: &kinds,
            seen_nodes: vec![false; graph.nodes.len()],
            seen_symbols: vec![false; graph.symbols.len()],
            next_node: 0,
            next_symbol: 0,
            units: 0,
        };
        admission.named(&mut graph.roots, false)?;
        admission.named(&mut graph.anchors, true)?;
        let mut keys = 0usize;
        for (id, node) in graph.nodes.iter_mut().enumerate() {
            if node.id != id || !admission.seen_nodes[id] {
                return Err("unreachable or misnumbered node");
            }
            if !matches!(node.prototype, Value::Null | Value::Object { .. }) {
                return Err("prototype is not object or null");
            }
            admission.value(&mut node.prototype)?;
            keys = keys
                .checked_add(node.properties.len())
                .ok_or("own-key budget overflow")?;
            if keys > MAX_KEYS {
                return Err("own-key budget exceeded");
            }
            let mut own_keys = HashSet::new();
            for property in &mut node.properties {
                if !own_keys.insert(property.key.clone()) {
                    return Err("duplicate own key");
                }
                match &property.key {
                    Key::String { units } => admission.units(units)?,
                    Key::Symbol { id } => Admission::reference(
                        *id,
                        &mut admission.seen_symbols,
                        &mut admission.next_symbol,
                    )?,
                }
                match &mut property.descriptor {
                    Descriptor::Data { value, .. } => admission.value(value)?,
                    Descriptor::Accessor { get, set, .. } => {
                        admission.accessor(get)?;
                        admission.accessor(set)?;
                    }
                }
            }
        }
        let mut registered = HashSet::new();
        let mut well_known = HashSet::new();
        for (id, symbol) in graph.symbols.iter().enumerate() {
            if symbol.id != id || !admission.seen_symbols[id] {
                return Err("unreachable or misnumbered Symbol");
            }
            if let Some(description) = &symbol.description {
                admission.units(description)?;
            }
            match &symbol.origin {
                SymbolOrigin::Local => {}
                SymbolOrigin::Registry { key } => {
                    admission.units(key)?;
                    if !registered.insert(key.clone()) || symbol.description.as_ref() != Some(key) {
                        return Err("inconsistent registry Symbol");
                    }
                }
                SymbolOrigin::WellKnown { name } => {
                    if !well_known.insert(*name) {
                        return Err("duplicate well-known Symbol");
                    }
                }
            }
        }
        let normalized = serde_json::to_vec(&graph).map_err(|_| "graph encoding failed")?;
        if normalized.len() > MAX_WIRE_BYTES {
            return Err("graph wire budget exceeded");
        }
        Ok(Self(graph))
    }

    #[cfg(any(test, feature = "spec-exec-oracle"))]
    pub(super) fn from_completion_units(units: &[u16]) -> Result<Self, &'static str> {
        if units.len() > MAX_WIRE_BYTES {
            return Err("graph wire budget exceeded");
        }
        let text = String::from_utf16(units).map_err(|_| "graph wire is not scalar JSON")?;
        if text.len() > MAX_WIRE_BYTES {
            return Err("graph wire budget exceeded");
        }
        serde_json::from_str(&text).map_err(|_| "invalid selected object probe graph")
    }

    #[cfg(any(test, feature = "spec-exec-oracle"))]
    pub(super) fn signature(&self) -> String {
        let bytes = serde_json::to_vec(self).expect("validated graph serialization is infallible");
        format!(
            "fnv1a64-{:016x}",
            fnv_field(
                fnv_update(FNV_OFFSET_BASIS, b"lila-selected-object-graph-v1"),
                &bytes
            )
        )
    }
}

#[cfg(any(test, feature = "spec-exec-oracle"))]
pub(super) fn project(
    completion: ObservedCompletion,
    backend_note: String,
) -> ExecutionObservation {
    match completion {
        ObservedCompletion::Normal(ObservedJsValue::String(units)) => {
            match SelectedObjectProbeGraph::from_completion_units(&units) {
                Ok(graph) => ExecutionObservation::SelectedObjectProbe {
                    graph,
                    backend_note: "selected object graph captured".into(),
                },
                Err(reason) => ExecutionObservation::ObservationRejected {
                    reason: reason.into(),
                },
            }
        }
        ObservedCompletion::Normal(
            ObservedJsValue::Undefined
            | ObservedJsValue::Null
            | ObservedJsValue::Boolean(_)
            | ObservedJsValue::Number(_)
            | ObservedJsValue::BigInt(_)
            | ObservedJsValue::Symbol
            | ObservedJsValue::Object,
        ) => ExecutionObservation::ObservationRejected {
            reason: "selected probe must return its graph String".into(),
        },
        ObservedCompletion::Throw(_) => ExecutionObservation::ObservationRejected {
            reason: format!("selected probe or reflection capture threw: {backend_note}"),
        },
    }
}

#[cfg(any(test, feature = "spec-exec-oracle"))]
pub(super) fn compare(wasm: &BackendObservation, spec: &BackendObservation) -> DifferentialVerdict {
    let (
        OutputEventsObservation::Captured {
            events: left_events,
        },
        OutputEventsObservation::Captured {
            events: right_events,
        },
    ) = (&wasm.output_events, &spec.output_events)
    else {
        return DifferentialVerdict::ObservationContractViolated;
    };
    match (&wasm.execution, &spec.execution) {
        (
            ExecutionObservation::SelectedObjectProbe { graph: left, .. },
            ExecutionObservation::SelectedObjectProbe { graph: right, .. },
        ) => {
            if left == right && left_events == right_events {
                DifferentialVerdict::SelectedObjectProbeAndPrintTranscriptMatch
            } else {
                DifferentialVerdict::Mismatch
            }
        }
        (
            ExecutionObservation::EngineFailure { .. },
            ExecutionObservation::EngineFailure { .. },
        ) => DifferentialVerdict::BothFailed,
        (
            ExecutionObservation::SelectedObjectProbe { .. },
            ExecutionObservation::EngineFailure { .. },
        )
        | (
            ExecutionObservation::EngineFailure { .. },
            ExecutionObservation::SelectedObjectProbe { .. },
        ) => DifferentialVerdict::Mismatch,
        (
            ExecutionObservation::Normal { .. }
            | ExecutionObservation::Error { .. }
            | ExecutionObservation::PrimitiveCompletion { .. }
            | ExecutionObservation::UnsupportedCompletion { .. }
            | ExecutionObservation::RootedCompletionGraph { .. }
            | ExecutionObservation::ObservationRejected { .. }
            | ExecutionObservation::WorkerFailure { .. },
            _,
        )
        | (
            _,
            ExecutionObservation::Normal { .. }
            | ExecutionObservation::Error { .. }
            | ExecutionObservation::PrimitiveCompletion { .. }
            | ExecutionObservation::UnsupportedCompletion { .. }
            | ExecutionObservation::RootedCompletionGraph { .. }
            | ExecutionObservation::ObservationRejected { .. }
            | ExecutionObservation::WorkerFailure { .. },
        ) => DifferentialVerdict::ObservationContractViolated,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wire() -> GraphWire {
        GraphWire {
            version: 1,
            roots: vec![NamedValue {
                name: "root".into(),
                value: Value::Object { id: 0 },
            }],
            anchors: vec![],
            nodes: vec![Node {
                id: 0,
                kind: ObjectKind::Object,
                extensible: true,
                prototype: Value::Null,
                properties: vec![Property {
                    key: Key::String { units: vec![120] },
                    descriptor: Descriptor::Data {
                        value: Value::Object { id: 0 },
                        writable: true,
                        enumerable: true,
                        configurable: true,
                    },
                }],
            }],
            symbols: vec![],
        }
    }

    fn observation(
        backend: DifferentialBackend,
        graph: SelectedObjectProbeGraph,
    ) -> BackendObservation {
        BackendObservation {
            worker_identity: None,
            backend,
            output_events: OutputEventsObservation::Captured {
                events: vec!["probe".into()],
            },
            execution: ExecutionObservation::SelectedObjectProbe {
                graph,
                backend_note: "captured".into(),
            },
        }
    }

    #[test]
    fn function_body_admission_prevents_capture_escape() {
        assert!(validate_body("return {roots: [{name: 'root', value: 1}], anchors: []};").is_ok());
        assert!(validate_body("}); 'forged graph'; (function () {").is_err());
        assert!(validate_body("return {}; } // close callback").is_err());
        let input = DifferentialReplayInput::new_script(
            "probe/escape",
            DifferentialProtocol::V5SelectedObjectProbePrintTranscript,
            "probe/escape.js",
            1000,
            "}); 'forged graph'; (function () {",
        )
        .unwrap();
        assert!(
            DifferentialCase::new(
                input.id().as_str(),
                input.goal(),
                input.protocol(),
                input.filename(),
                input.timeout_ms().get(),
                input.source()
            )
            .is_err(),
            "native input waits for worker FunctionBody admission"
        );
        let fixture =
            include_str!("../../tests/differential/v5/t25-object-graph-descriptors-and-realm.json");
        let admitted = DifferentialCase::from_json(fixture).unwrap();
        assert_eq!(
            admitted.protocol(),
            DifferentialProtocol::V5SelectedObjectProbePrintTranscript
        );
        assert_eq!(
            DifferentialReplayInput::from_json(&admitted.to_pretty_json().unwrap()).unwrap(),
            DifferentialReplayInput::from(admitted)
        );
        let mut wrong_goal: serde_json::Value = serde_json::from_str(fixture).unwrap();
        wrong_goal["goal"] = serde_json::json!("module");
        assert!(DifferentialReplayInput::from_json(&wrong_goal.to_string()).is_err());
        assert!(DifferentialCase::from_json(&wrong_goal.to_string()).is_err());
        wrong_goal["goal"] = serde_json::json!("script");
        wrong_goal["observation_contract"] =
            serde_json::json!("primitive_completion_print_transcript");
        assert!(DifferentialReplayInput::from_json(&wrong_goal.to_string()).is_err());
    }

    #[test]
    fn cyclic_graph_roundtrips_only_through_validated_admission() {
        let graph = SelectedObjectProbeGraph::validate(wire()).unwrap();
        let bytes = serde_json::to_vec(&graph).unwrap();
        let decoded: SelectedObjectProbeGraph = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(graph, decoded);
        let mut invalid = wire();
        invalid.nodes[0].prototype = Value::Object { id: 1 };
        assert!(SelectedObjectProbeGraph::validate(invalid).is_err());
        let mut detached = wire();
        detached.nodes.push(Node {
            id: 1,
            kind: ObjectKind::Object,
            extensible: true,
            prototype: Value::Null,
            properties: vec![],
        });
        assert!(SelectedObjectProbeGraph::validate(detached).is_err());
        let mut repeated = wire();
        let duplicate = repeated.nodes[0].properties[0].clone();
        repeated.nodes[0].properties.push(duplicate);
        assert!(SelectedObjectProbeGraph::validate(repeated).is_err());
    }

    #[test]
    fn comparison_preserves_aliasing_descriptors_keys_and_prototypes() {
        let baseline = SelectedObjectProbeGraph::validate(wire()).unwrap();
        let left = observation(DifferentialBackend::WasmAot, baseline.clone());
        assert_eq!(
            compare(
                &left,
                &observation(DifferentialBackend::SpecExec, baseline.clone())
            ),
            DifferentialVerdict::SelectedObjectProbeAndPrintTranscriptMatch
        );
        let mut changed = wire();
        let Descriptor::Data { writable, .. } = &mut changed.nodes[0].properties[0].descriptor
        else {
            unreachable!()
        };
        *writable = false;
        assert_eq!(
            compare(
                &left,
                &observation(
                    DifferentialBackend::SpecExec,
                    SelectedObjectProbeGraph::validate(changed).unwrap()
                )
            ),
            DifferentialVerdict::Mismatch
        );
        let mut changed = wire();
        changed.nodes[0].properties[0].key = Key::String { units: vec![121] };
        assert_eq!(
            compare(
                &left,
                &observation(
                    DifferentialBackend::SpecExec,
                    SelectedObjectProbeGraph::validate(changed).unwrap()
                )
            ),
            DifferentialVerdict::Mismatch
        );
        let mut changed = wire();
        changed.nodes[0].prototype = Value::Object { id: 0 };
        assert_eq!(
            compare(
                &left,
                &observation(
                    DifferentialBackend::SpecExec,
                    SelectedObjectProbeGraph::validate(changed).unwrap()
                )
            ),
            DifferentialVerdict::Mismatch
        );
        let mut changed = wire();
        let Descriptor::Data { value, .. } = &mut changed.nodes[0].properties[0].descriptor else {
            unreachable!()
        };
        *value = Value::Object { id: 1 };
        changed.nodes.push(Node {
            id: 1,
            kind: ObjectKind::Object,
            extensible: true,
            prototype: Value::Null,
            properties: vec![],
        });
        assert_eq!(
            compare(
                &left,
                &observation(
                    DifferentialBackend::SpecExec,
                    SelectedObjectProbeGraph::validate(changed).unwrap()
                )
            ),
            DifferentialVerdict::Mismatch
        );
    }

    #[test]
    fn realm_anchor_observes_actual_prototype_identity() {
        let mut primary = wire();
        primary.anchors.push(NamedValue {
            name: "type_error_prototype".into(),
            value: Value::Object { id: 1 },
        });
        primary.nodes[0].prototype = Value::Object { id: 1 };
        primary.nodes.push(Node {
            id: 1,
            kind: ObjectKind::Object,
            extensible: true,
            prototype: Value::Null,
            properties: vec![],
        });
        let mut foreign = primary.clone();
        foreign.nodes[0].prototype = Value::Object { id: 2 };
        foreign.nodes.push(Node {
            id: 2,
            kind: ObjectKind::Object,
            extensible: true,
            prototype: Value::Null,
            properties: vec![],
        });
        assert_eq!(
            compare(
                &observation(
                    DifferentialBackend::WasmAot,
                    SelectedObjectProbeGraph::validate(primary).unwrap()
                ),
                &observation(
                    DifferentialBackend::SpecExec,
                    SelectedObjectProbeGraph::validate(foreign).unwrap()
                )
            ),
            DifferentialVerdict::Mismatch
        );
    }

    #[test]
    fn symbol_identity_is_distinct_from_description_and_registry_key() {
        let mut shared = wire();
        shared.nodes.clear();
        shared.roots = vec![
            NamedValue {
                name: "left".into(),
                value: Value::Symbol { id: 0 },
            },
            NamedValue {
                name: "right".into(),
                value: Value::Symbol { id: 0 },
            },
        ];
        shared.symbols.push(Symbol {
            id: 0,
            description: Some(vec![120]),
            origin: SymbolOrigin::Local,
        });
        let mut distinct = shared.clone();
        distinct.roots[1].value = Value::Symbol { id: 1 };
        distinct.symbols.push(Symbol {
            id: 1,
            description: Some(vec![120]),
            origin: SymbolOrigin::Local,
        });
        assert_eq!(
            compare(
                &observation(
                    DifferentialBackend::WasmAot,
                    SelectedObjectProbeGraph::validate(shared.clone()).unwrap()
                ),
                &observation(
                    DifferentialBackend::SpecExec,
                    SelectedObjectProbeGraph::validate(distinct.clone()).unwrap()
                )
            ),
            DifferentialVerdict::Mismatch
        );
        shared.symbols.clear();
        assert!(SelectedObjectProbeGraph::validate(shared).is_err());
        for symbol in &mut distinct.symbols {
            symbol.origin = SymbolOrigin::Registry { key: vec![120] };
        }
        assert!(
            SelectedObjectProbeGraph::validate(distinct.clone()).is_err(),
            "one registry key cannot denote two identities"
        );
        for symbol in &mut distinct.symbols {
            symbol.origin = SymbolOrigin::WellKnown {
                name: WellKnownSymbol::Iterator,
            };
        }
        assert!(
            SelectedObjectProbeGraph::validate(distinct).is_err(),
            "a well-known Symbol has one identity"
        );
    }

    #[test]
    fn numeric_normalization_preserves_zero_sign_and_canonical_nan() {
        let mut negative = wire();
        negative.roots[0].value = Value::NumberDecimal {
            decimal: "-0".into(),
        };
        negative.nodes.clear();
        let mut positive = negative.clone();
        positive.roots[0].value = Value::NumberDecimal {
            decimal: "0".into(),
        };
        assert_ne!(
            SelectedObjectProbeGraph::validate(negative.clone()).unwrap(),
            SelectedObjectProbeGraph::validate(positive).unwrap()
        );
        negative.roots[0].value = Value::NumberDecimal {
            decimal: "NaN".into(),
        };
        let decimal_nan = SelectedObjectProbeGraph::validate(negative.clone()).unwrap();
        negative.roots[0].value = Value::Number {
            bits: "7ff8000000000000".into(),
        };
        assert_eq!(
            decimal_nan,
            SelectedObjectProbeGraph::validate(negative.clone()).unwrap()
        );
        negative.roots[0].value = Value::Number {
            bits: "7ff8000000000001".into(),
        };
        assert!(SelectedObjectProbeGraph::validate(negative).is_err());
    }

    #[test]
    fn graph_bounds_accessors_and_capture_failures_remain_red() {
        let mut invalid = wire();
        invalid.nodes[0].properties[0].descriptor = Descriptor::Accessor {
            get: Value::Object { id: 0 },
            set: Value::Undefined,
            enumerable: true,
            configurable: true,
        };
        assert!(SelectedObjectProbeGraph::validate(invalid).is_err());
        let mut invalid = wire();
        invalid.roots[0].value = Value::String {
            units: vec![0; MAX_UNITS + 1],
        };
        invalid.nodes.clear();
        assert!(SelectedObjectProbeGraph::validate(invalid).is_err());
        let mut invalid = wire();
        invalid.roots = (0..17)
            .map(|index| NamedValue {
                name: format!("root{index}"),
                value: Value::Object { id: 0 },
            })
            .collect();
        assert!(SelectedObjectProbeGraph::validate(invalid).is_err());
        let mut invalid = wire();
        let repeated_node = invalid.nodes[0].clone();
        invalid.nodes.resize(MAX_NODES + 1, repeated_node);
        assert!(SelectedObjectProbeGraph::validate(invalid).is_err());
        let mut invalid = wire();
        invalid.symbols = (0..=MAX_SYMBOLS)
            .map(|id| Symbol {
                id,
                description: None,
                origin: SymbolOrigin::Local,
            })
            .collect();
        assert!(SelectedObjectProbeGraph::validate(invalid).is_err());
        let mut invalid = wire();
        invalid.nodes[0].properties = (0..=MAX_KEYS)
            .map(|index| Property {
                key: Key::String {
                    units: vec![index as u16],
                },
                descriptor: Descriptor::Data {
                    value: Value::Null,
                    writable: true,
                    enumerable: true,
                    configurable: true,
                },
            })
            .collect();
        assert!(SelectedObjectProbeGraph::validate(invalid).is_err());
        let mut unknown = serde_json::to_value(wire()).unwrap();
        unknown["host_pointer"] = serde_json::json!(1);
        assert!(serde_json::from_value::<SelectedObjectProbeGraph>(unknown).is_err());
        let left = observation(
            DifferentialBackend::WasmAot,
            SelectedObjectProbeGraph::validate(wire()).unwrap(),
        );
        let mut right = left.clone();
        right.backend = DifferentialBackend::SpecExec;
        right.execution = ExecutionObservation::ObservationRejected {
            reason: "reflection trap threw".into(),
        };
        assert_eq!(
            compare(&left, &right),
            DifferentialVerdict::ObservationContractViolated
        );
        right.execution = left.execution.clone();
        right.output_events = OutputEventsObservation::Captured {
            events: vec!["different".into()],
        };
        assert_eq!(compare(&left, &right), DifferentialVerdict::Mismatch);
    }

    #[test]
    fn versioned_report_signature_binds_graph_and_excludes_backend_notes() {
        let input = DifferentialReplayInput::new_script(
            "probe/signature",
            DifferentialProtocol::V5SelectedObjectProbePrintTranscript,
            "probe/signature.js",
            1000,
            "return {roots: [{name: 'root', value: 1}], anchors: []};",
        )
        .unwrap();
        let left = observation(
            DifferentialBackend::WasmAot,
            SelectedObjectProbeGraph::validate(wire()).unwrap(),
        );
        let mut changed = wire();
        changed.nodes[0].extensible = false;
        let right = observation(
            DifferentialBackend::SpecExec,
            SelectedObjectProbeGraph::validate(changed).unwrap(),
        );
        let report = compare_observations(&input, left.clone(), right.clone());
        assert_eq!(
            report.protocol(),
            DifferentialProtocol::V5SelectedObjectProbePrintTranscript
        );
        assert_eq!(report.verdict(), DifferentialVerdict::Mismatch);
        assert!(!report.is_green());
        assert_eq!(
            report.semantic_equivalence(),
            SemanticEquivalence::NotEstablished
        );
        let mut diagnostic_only = right.clone();
        let ExecutionObservation::SelectedObjectProbe { backend_note, .. } =
            &mut diagnostic_only.execution
        else {
            unreachable!()
        };
        *backend_note = "a different diagnostic path or host handle".into();
        assert_eq!(
            report.mismatch_signature(),
            compare_observations(&input, left.clone(), diagnostic_only).mismatch_signature()
        );
        let mut different_print = right;
        different_print.output_events = OutputEventsObservation::Captured {
            events: vec!["other".into()],
        };
        assert_ne!(
            report.mismatch_signature(),
            compare_observations(&input, left, different_print).mismatch_signature()
        );
    }
}
