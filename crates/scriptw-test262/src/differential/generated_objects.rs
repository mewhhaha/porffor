//! A bounded source algebra for the existing selected-object probe protocol.
use super::*;
use serde::Serialize;
use std::fmt::Write as _;

pub const OBJECT_PROBE_GRAMMAR: &str = "object-probe-v1";
pub const OBJECT_MUTATION_GRAMMAR: &str = "object-mutations-v2";
pub const MAX_GENERATED_OBJECT_NODES: usize = 16;
pub const MAX_GENERATED_OBJECT_PROPERTIES: usize = 16;
pub const MAX_GENERATED_OBJECT_STEPS: usize = 64;
const MAX_ACCESSOR_FUNCTIONS: usize = 64;
mod mutations;
use mutations::Operation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectGrammar {
    StaticV1,
    MutationsV2,
}
impl ObjectGrammar {
    pub const fn name(self) -> &'static str {
        match self {
            Self::StaticV1 => OBJECT_PROBE_GRAMMAR,
            Self::MutationsV2 => OBJECT_MUTATION_GRAMMAR,
        }
    }
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            OBJECT_PROBE_GRAMMAR => Some(Self::StaticV1),
            OBJECT_MUTATION_GRAMMAR => Some(Self::MutationsV2),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjectGenerationPlan {
    grammar: ObjectGrammar,
    seed: u64,
    nodes: u8,
    properties: u8,
    steps: u8,
}
impl ObjectGenerationPlan {
    pub fn new(seed: u64, nodes: usize, properties: usize) -> Result<Self, DifferentialError> {
        Self::for_grammar(ObjectGrammar::StaticV1, seed, nodes, properties, 0)
    }
    pub fn for_grammar(
        grammar: ObjectGrammar,
        seed: u64,
        nodes: usize,
        properties: usize,
        steps: usize,
    ) -> Result<Self, DifferentialError> {
        if !(1..=MAX_GENERATED_OBJECT_NODES).contains(&nodes)
            || !(1..=MAX_GENERATED_OBJECT_PROPERTIES).contains(&properties)
        {
            return Err(DifferentialError::InvalidGeneration(
                "object generation needs 1..=16 nodes and 1..=16 properties per node".into(),
            ));
        }
        match grammar {
            ObjectGrammar::StaticV1 if steps != 0 => {
                return Err(DifferentialError::InvalidGeneration(
                    "object-probe-v1 has no mutation steps".into(),
                ))
            }
            ObjectGrammar::MutationsV2
                if nodes == MAX_GENERATED_OBJECT_NODES
                    || !(1..=MAX_GENERATED_OBJECT_STEPS).contains(&steps) =>
            {
                return Err(DifferentialError::InvalidGeneration(
                    "object-mutations-v2 needs 1..=15 nodes and 1..=64 mutation steps".into(),
                ))
            }
            ObjectGrammar::StaticV1 | ObjectGrammar::MutationsV2 => {}
        }
        Ok(Self {
            grammar,
            seed,
            nodes: nodes as u8,
            properties: properties as u8,
            steps: steps as u8,
        })
    }
    pub const fn grammar(self) -> ObjectGrammar {
        self.grammar
    }
    pub const fn seed(self) -> u64 {
        self.seed
    }
    pub const fn nodes(self) -> u8 {
        self.nodes
    }
    pub const fn properties(self) -> u8 {
        self.properties
    }
    pub const fn steps(self) -> u8 {
        self.steps
    }
    pub(super) fn with_seed(self, seed: u64) -> Self {
        Self { seed, ..self }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NodeId(u8);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SymbolId(u8);
#[derive(Debug, Clone, PartialEq, Eq)]
enum Value {
    Undefined,
    Null,
    Boolean(bool),
    Integer(i16),
    NegativeZero,
    NaN,
    String(&'static str),
    BigInt(i16),
    Node(NodeId),
    Symbol(SymbolId),
}
impl Value {
    fn source(&self) -> String {
        match self {
            Self::Undefined => "undefined".into(),
            Self::Null => "null".into(),
            Self::Boolean(value) => value.to_string(),
            Self::Integer(value) => value.to_string(),
            Self::NegativeZero => "-0".into(),
            Self::NaN => "(0/0)".into(),
            Self::String(value) => quoted(value),
            Self::BigInt(value) => format!("{value}n"),
            Self::Node(id) => format!("n{}", id.0),
            Self::Symbol(id) => format!("s{}", id.0),
        }
    }
    fn weight(&self) -> usize {
        match self {
            Self::Undefined | Self::Null | Self::Boolean(false) | Self::Integer(0) => 0,
            Self::Boolean(true) | Self::NegativeZero | Self::Node(_) | Self::Symbol(_) => 1,
            Self::Integer(value) => usize::from(value.unsigned_abs()),
            Self::NaN => 2,
            Self::String(value) => value.len(),
            Self::BigInt(value) => 1 + usize::from(value.unsigned_abs()),
        }
    }
    fn valid(&self, nodes: usize, symbols: usize) -> bool {
        match self {
            Self::Node(id) => usize::from(id.0) < nodes,
            Self::Symbol(id) => usize::from(id.0) < symbols,
            Self::Integer(value) | Self::BigInt(value) => (-32..=32).contains(value),
            Self::Undefined
            | Self::Null
            | Self::Boolean(_)
            | Self::NegativeZero
            | Self::NaN
            | Self::String(_) => true,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Key {
    String(&'static str),
    Symbol(SymbolId),
}
impl Key {
    fn source(&self) -> String {
        match self {
            Self::String(value) => quoted(value),
            Self::Symbol(id) => format!("s{}", id.0),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
enum Descriptor {
    Data { value: Value, writable: bool },
    Accessor { value: Value, setter: bool },
}
impl Descriptor {
    fn value(&self) -> &Value {
        match self {
            Self::Data { value, .. } | Self::Accessor { value, .. } => value,
        }
    }
    fn value_mut(&mut self) -> &mut Value {
        match self {
            Self::Data { value, .. } | Self::Accessor { value, .. } => value,
        }
    }
    fn weight(&self) -> usize {
        self.value().weight()
            + match self {
                Self::Data { writable, .. } => usize::from(*writable),
                Self::Accessor { setter, .. } => 2 + usize::from(*setter),
            }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct Property {
    key: Key,
    descriptor: Descriptor,
    enumerable: bool,
    configurable: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Object,
    Array,
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct Node {
    kind: Kind,
    prototype: Option<NodeId>,
    properties: Vec<Property>,
    extensible: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct Root {
    name: u8,
    node: NodeId,
}

/// Only checked construction can reach rendering or reduction. Prototype edges
/// point backwards; data/accessor payload edges may share or cycle freely.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Program {
    grammar: ObjectGrammar,
    nodes: Vec<Node>,
    symbols: Vec<&'static str>,
    roots: Vec<Root>,
    anchors: bool,
    operations: Vec<Operation>,
}
impl Program {
    fn new(
        nodes: Vec<Node>,
        symbols: Vec<&'static str>,
        roots: Vec<Root>,
        anchors: bool,
    ) -> Option<Self> {
        Self::checked(
            ObjectGrammar::StaticV1,
            nodes,
            symbols,
            roots,
            anchors,
            vec![],
        )
    }
    fn checked(
        grammar: ObjectGrammar,
        nodes: Vec<Node>,
        symbols: Vec<&'static str>,
        roots: Vec<Root>,
        anchors: bool,
        operations: Vec<Operation>,
    ) -> Option<Self> {
        if nodes.is_empty()
            || nodes.len() > MAX_GENERATED_OBJECT_NODES
            || symbols.len() > 8
            || roots.is_empty()
            || roots.len() > MAX_GENERATED_OBJECT_NODES
        {
            return None;
        }
        match grammar {
            ObjectGrammar::StaticV1 if !operations.is_empty() => return None,
            ObjectGrammar::MutationsV2
                if nodes.len() == MAX_GENERATED_OBJECT_NODES
                    || roots.len() == MAX_GENERATED_OBJECT_NODES
                    || operations.len() > MAX_GENERATED_OBJECT_STEPS =>
            {
                return None
            }
            ObjectGrammar::StaticV1 | ObjectGrammar::MutationsV2 => {}
        }
        if operations
            .iter()
            .any(|operation| !operation.valid(&nodes, symbols.len()))
        {
            return None;
        }
        for (index, node) in nodes.iter().enumerate() {
            if node.prototype.is_some_and(|id| usize::from(id.0) >= index)
                || node.properties.len() > MAX_GENERATED_OBJECT_PROPERTIES
            {
                return None;
            }
            for (position, property) in node.properties.iter().enumerate() {
                if node.properties[..position]
                    .iter()
                    .any(|prior| prior.key == property.key)
                    || !property
                        .descriptor
                        .value()
                        .valid(nodes.len(), symbols.len())
                {
                    return None;
                }
                match property.key {
                    Key::Symbol(id) if usize::from(id.0) >= symbols.len() => return None,
                    Key::String("length") if node.kind == Kind::Array => return None,
                    Key::String(_) | Key::Symbol(_) => {}
                }
            }
        }
        let functions: usize = nodes
            .iter()
            .flat_map(|node| &node.properties)
            .map(|property| match &property.descriptor {
                Descriptor::Data { .. } => 0,
                Descriptor::Accessor { setter, .. } => 1 + usize::from(*setter),
            })
            .sum();
        if functions > MAX_ACCESSOR_FUNCTIONS {
            return None;
        }
        for (index, root) in roots.iter().enumerate() {
            if usize::from(root.node.0) >= nodes.len()
                || roots[..index].iter().any(|prior| prior.name == root.name)
            {
                return None;
            }
        }
        let result = Self {
            grammar,
            nodes,
            symbols,
            roots,
            anchors,
            operations,
        };
        if result.reachable().iter().any(|used| !used) {
            return None;
        }
        Some(result)
    }
    fn reachable(&self) -> Vec<bool> {
        let mut used = vec![false; self.nodes.len()];
        let mut queue: Vec<_> = self.roots.iter().map(|root| root.node).collect();
        for operation in &self.operations {
            operation.references(|id| queue.push(id), |_| {});
        }
        while let Some(id) = queue.pop() {
            if used[usize::from(id.0)] {
                continue;
            }
            used[usize::from(id.0)] = true;
            let node = &self.nodes[usize::from(id.0)];
            queue.extend(node.prototype);
            for property in &node.properties {
                if let Value::Node(id) = property.descriptor.value() {
                    queue.push(*id);
                }
            }
        }
        used
    }
    fn prune(mut self) -> Option<Self> {
        let used = self.reachable();
        let mut node_map = vec![None; self.nodes.len()];
        let mut next = 0;
        for (index, keep) in used.iter().enumerate() {
            if *keep {
                node_map[index] = Some(NodeId(next));
                next += 1;
            }
        }
        let mut nodes: Vec<_> = self
            .nodes
            .into_iter()
            .enumerate()
            .filter_map(|(index, node)| used[index].then_some(node))
            .collect();
        let mut symbol_used = vec![false; self.symbols.len()];
        for operation in &self.operations {
            operation.references(|_| {}, |id| symbol_used[usize::from(id.0)] = true);
        }
        for node in &mut nodes {
            node.prototype = node
                .prototype
                .map(|id| node_map[usize::from(id.0)].unwrap());
            for property in &mut node.properties {
                if let Key::Symbol(id) = property.key {
                    symbol_used[usize::from(id.0)] = true;
                }
                match property.descriptor.value_mut() {
                    Value::Node(id) => *id = node_map[usize::from(id.0)].unwrap(),
                    Value::Symbol(id) => symbol_used[usize::from(id.0)] = true,
                    Value::Undefined
                    | Value::Null
                    | Value::Boolean(_)
                    | Value::Integer(_)
                    | Value::NegativeZero
                    | Value::NaN
                    | Value::String(_)
                    | Value::BigInt(_) => {}
                }
            }
        }
        let mut symbol_map = vec![None; self.symbols.len()];
        let mut symbols = Vec::new();
        for (index, description) in self.symbols.into_iter().enumerate() {
            if symbol_used[index] {
                symbol_map[index] = Some(SymbolId(symbols.len() as u8));
                symbols.push(description);
            }
        }
        for node in &mut nodes {
            for property in &mut node.properties {
                if let Key::Symbol(id) = &mut property.key {
                    *id = symbol_map[usize::from(id.0)].unwrap();
                }
                if let Value::Symbol(id) = property.descriptor.value_mut() {
                    *id = symbol_map[usize::from(id.0)].unwrap();
                }
            }
        }
        let roots = self
            .roots
            .into_iter()
            .map(|root| Root {
                name: root.name,
                node: node_map[usize::from(root.node.0)].unwrap(),
            })
            .collect();
        for operation in &mut self.operations {
            operation.remap(&node_map, &symbol_map);
        }
        Self::checked(
            self.grammar,
            nodes,
            symbols,
            roots,
            self.anchors,
            self.operations,
        )
    }
    fn complexity(&self) -> (usize, usize, usize, usize) {
        (
            self.nodes.len(),
            self.nodes.iter().map(|node| node.properties.len()).sum(),
            self.operations.len(),
            self.roots.len()
                + self.symbols.len()
                + usize::from(self.anchors)
                + self.operations.iter().map(Operation::weight).sum::<usize>()
                + self
                    .nodes
                    .iter()
                    .map(|node| {
                        usize::from(node.prototype.is_some())
                            + usize::from(!node.extensible)
                            + node
                                .properties
                                .iter()
                                .map(|property| {
                                    property.descriptor.weight()
                                        + usize::from(property.enumerable)
                                        + usize::from(property.configurable)
                                })
                                .sum::<usize>()
                    })
                    .sum::<usize>(),
        )
    }
    fn source(&self) -> String {
        let mut source = String::from("'use strict';\n");
        for (index, description) in self.symbols.iter().enumerate() {
            writeln!(source, "const s{index}=Symbol({});", quoted(description)).unwrap();
        }
        for (index, node) in self.nodes.iter().enumerate() {
            let allocation = match node.kind {
                Kind::Object => "Object.create(null)",
                Kind::Array => "[]",
            };
            writeln!(source, "const n{index}={allocation};").unwrap();
        }
        for (index, node) in self.nodes.iter().enumerate() {
            let prototype = node
                .prototype
                .map_or_else(|| "null".into(), |id| format!("n{}", id.0));
            writeln!(source, "Object.setPrototypeOf(n{index},{prototype});").unwrap();
            for property in &node.properties {
                let body = match &property.descriptor {
                    Descriptor::Data { value, writable } => {
                        format!("value:{},writable:{writable}", value.source())
                    }
                    Descriptor::Accessor { value, setter } => format!(
                        "get:function(){{print('generated-getter-invoked');return {};}},set:{}",
                        value.source(),
                        if *setter {
                            "function(value){print('generated-setter-invoked');}"
                        } else {
                            "undefined"
                        }
                    ),
                };
                writeln!(
                    source,
                    "Object.defineProperty(n{index},{},{{{body},enumerable:{},configurable:{}}});",
                    property.key.source(),
                    property.enumerable,
                    property.configurable
                )
                .unwrap();
            }
            if !node.extensible {
                writeln!(source, "Object.preventExtensions(n{index});").unwrap();
            }
        }
        match self.grammar {
            ObjectGrammar::StaticV1 => {}
            ObjectGrammar::MutationsV2 => {
                source.push_str("const operationResults=Object.create(null);\n");
                for (index, operation) in self.operations.iter().enumerate() {
                    operation.emit(&mut source, index);
                }
            }
        }
        source.push_str("return {roots:[");
        for (index, root) in self.roots.iter().enumerate() {
            if index > 0 {
                source.push(',');
            }
            write!(
                source,
                "{{name:'root_{}',value:n{}}}",
                root.name, root.node.0
            )
            .unwrap();
        }
        if self.grammar == ObjectGrammar::MutationsV2 {
            source.push_str(",{name:'operation_results',value:operationResults}");
        }
        source.push_str("],anchors:[");
        if self.anchors {
            source.push_str("{name:'object_prototype',value:Object.prototype},{name:'type_error_prototype',value:TypeError.prototype}");
        }
        source.push_str("]};\n");
        source
    }
    fn case(
        &self,
        plan: ObjectGenerationPlan,
    ) -> Result<DifferentialReplayInput, DifferentialError> {
        if self.grammar != plan.grammar {
            return Err(DifferentialError::GeneratorInvariant(
                "object source belongs to a different grammar plan".into(),
            ));
        }
        let mut stem = format!(
            "seed-{:016x}-nodes-{:02}-properties-{:02}",
            plan.seed, plan.nodes, plan.properties
        );
        if plan.grammar == ObjectGrammar::MutationsV2 {
            write!(stem, "-steps-{:02}", plan.steps).unwrap();
        }
        let grammar = plan.grammar.name();
        DifferentialReplayInput::new_script(
            format!("t25/generated/{grammar}/{stem}"),
            DifferentialProtocol::V5SelectedObjectProbePrintTranscript,
            format!("differential/v5/generated/{grammar}/{stem}.js"),
            5000,
            self.source(),
        )
    }
    fn reductions(&self) -> Vec<Self> {
        let mut result = Vec::new();
        let mut push = |candidate: Self| {
            if let Some(candidate) = candidate.prune() {
                if candidate.complexity() < self.complexity() && !result.contains(&candidate) {
                    result.push(candidate);
                }
            }
        };
        if self.roots.len() > 1 {
            for index in (0..self.roots.len()).rev() {
                let mut candidate = self.clone();
                candidate.roots.remove(index);
                push(candidate);
            }
        }
        if self.anchors {
            let mut candidate = self.clone();
            candidate.anchors = false;
            push(candidate);
        }
        for (index, operation) in self.operations.iter().enumerate() {
            let mut candidate = self.clone();
            candidate.operations.remove(index);
            push(candidate);
            for replacement in operation.reductions() {
                let mut candidate = self.clone();
                candidate.operations[index] = replacement;
                push(candidate);
            }
        }
        for (node_index, node) in self.nodes.iter().enumerate() {
            if node.prototype.is_some() {
                let mut candidate = self.clone();
                candidate.nodes[node_index].prototype = None;
                push(candidate);
            }
            if !node.extensible {
                let mut candidate = self.clone();
                candidate.nodes[node_index].extensible = true;
                push(candidate);
            }
            for (property_index, property) in node.properties.iter().enumerate() {
                let mut candidate = self.clone();
                candidate.nodes[node_index]
                    .properties
                    .remove(property_index);
                push(candidate);
                if property.descriptor.value().weight() > 0 {
                    let mut candidate = self.clone();
                    *candidate.nodes[node_index].properties[property_index]
                        .descriptor
                        .value_mut() = Value::Undefined;
                    push(candidate);
                }
                match &property.descriptor {
                    Descriptor::Accessor { .. } => {
                        let mut candidate = self.clone();
                        candidate.nodes[node_index].properties[property_index].descriptor =
                            Descriptor::Data {
                                value: property.descriptor.value().clone(),
                                writable: false,
                            };
                        push(candidate);
                    }
                    Descriptor::Data { writable: true, .. } => {
                        let mut candidate = self.clone();
                        if let Descriptor::Data { writable, .. } =
                            &mut candidate.nodes[node_index].properties[property_index].descriptor
                        {
                            *writable = false;
                        }
                        push(candidate);
                    }
                    Descriptor::Data {
                        writable: false, ..
                    } => {}
                }
                for attribute in [0, 1] {
                    if (attribute == 0 && property.enumerable)
                        || (attribute == 1 && property.configurable)
                    {
                        let mut candidate = self.clone();
                        let property = &mut candidate.nodes[node_index].properties[property_index];
                        if attribute == 0 {
                            property.enumerable = false;
                        } else {
                            property.configurable = false;
                        }
                        push(candidate);
                    }
                }
            }
        }
        result
    }
}
fn quoted(value: &str) -> String {
    serde_json::to_string(value).expect("string serialization cannot fail")
}
struct Random(u64);
impl Random {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut value = self.0;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }
    fn boolean(&mut self) -> bool {
        self.next() & 1 != 0
    }
    fn value(&mut self, nodes: u8, symbols: u8) -> Value {
        match self.next() % 10 {
            0 => Value::Undefined,
            1 => Value::Null,
            2 => Value::Boolean(self.boolean()),
            3 => Value::Integer((self.next() % 65) as i16 - 32),
            4 => Value::NegativeZero,
            5 => Value::NaN,
            6 => Value::String(["", "x", "a\0b", "λ💠"][(self.next() % 4) as usize]),
            7 => Value::BigInt((self.next() % 65) as i16 - 32),
            8 => Value::Node(NodeId((self.next() % u64::from(nodes)) as u8)),
            9 => Value::Symbol(SymbolId((self.next() % u64::from(symbols)) as u8)),
            _ => unreachable!(),
        }
    }
}
fn generate(plan: ObjectGenerationPlan) -> Result<Program, DifferentialError> {
    let mut random = Random(plan.seed);
    let symbols = vec!["shared", "shared", "λ", ""];
    const STRINGS: [&str; 12] = [
        "self",
        "2",
        "10",
        "x",
        "y",
        "",
        "__proto__",
        "constructor",
        "01",
        "a\0b",
        "λ",
        "4294967294",
    ];
    let mut nodes = Vec::new();
    let mut accessor_functions = 0;
    for index in 0..plan.nodes {
        let kind = if index == 0 || random.boolean() {
            Kind::Object
        } else {
            Kind::Array
        };
        let prototype = if index > 0 && random.boolean() {
            Some(NodeId((random.next() % u64::from(index)) as u8))
        } else {
            None
        };
        let mut keys: Vec<_> = STRINGS
            .into_iter()
            .map(Key::String)
            .chain((0..4).map(|id| Key::Symbol(SymbolId(id))))
            .collect();
        let mut properties = Vec::new();
        for position in 0..plan.properties {
            let key = keys.remove(if index == 0 && position == 0 {
                0
            } else {
                (random.next() % keys.len() as u64) as usize
            });
            let value = if index == 0 && position == 0 {
                Value::Node(NodeId(0))
            } else {
                random.value(plan.nodes, symbols.len() as u8)
            };
            let data = index == 0 && position == 0 || random.boolean();
            let descriptor = if data || accessor_functions == MAX_ACCESSOR_FUNCTIONS {
                Descriptor::Data {
                    value,
                    writable: random.boolean(),
                }
            } else {
                let setter = random.boolean() && accessor_functions + 1 < MAX_ACCESSOR_FUNCTIONS;
                accessor_functions += 1 + usize::from(setter);
                Descriptor::Accessor { value, setter }
            };
            properties.push(Property {
                key,
                descriptor,
                enumerable: random.boolean(),
                configurable: random.boolean(),
            });
        }
        nodes.push(Node {
            kind,
            prototype,
            properties,
            extensible: random.boolean(),
        });
    }
    let roots = (0..plan.nodes)
        .map(|id| Root {
            name: id,
            node: NodeId(id),
        })
        .collect();
    let program = match plan.grammar {
        ObjectGrammar::StaticV1 => Program::new(nodes, symbols, roots, true),
        ObjectGrammar::MutationsV2 => {
            let operations =
                mutations::generate(&nodes, symbols.len() as u8, plan.steps, &mut random);
            Program::checked(plan.grammar, nodes, symbols, roots, true, operations)
        }
    };
    program.ok_or_else(|| {
        DifferentialError::GeneratorInvariant(
            "object generation escaped its checked graph domain".into(),
        )
    })
}
pub fn generate_object_probe_case(
    plan: ObjectGenerationPlan,
) -> Result<DifferentialReplayInput, DifferentialError> {
    generate(plan)?.case(plan)
}

/// A reducer target is the observed v5 dimension, never the source-dependent
/// mismatch signature. Both graph and print differences must stay when present.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Witness {
    Compared { graph: bool, print: bool },
    WasmFailed(FailurePhase),
    SpecFailed(FailurePhase),
}
impl Witness {
    fn from_report(report: &DifferentialReport) -> Option<Self> {
        if report.protocol() != DifferentialProtocol::V5SelectedObjectProbePrintTranscript
            || report.verdict() != DifferentialVerdict::Mismatch
        {
            return None;
        }
        match (&report.wasm_aot().execution, &report.spec_exec().execution) {
            (
                ExecutionObservation::SelectedObjectProbe { graph: left, .. },
                ExecutionObservation::SelectedObjectProbe { graph: right, .. },
            ) => {
                let (
                    OutputEventsObservation::Captured { events: a },
                    OutputEventsObservation::Captured { events: b },
                ) = (
                    &report.wasm_aot().output_events,
                    &report.spec_exec().output_events,
                )
                else {
                    return None;
                };
                Some(Self::Compared {
                    graph: left != right,
                    print: a != b,
                })
            }
            (
                ExecutionObservation::EngineFailure { phase, .. },
                ExecutionObservation::SelectedObjectProbe { .. },
            ) => Some(Self::WasmFailed(*phase)),
            (
                ExecutionObservation::SelectedObjectProbe { .. },
                ExecutionObservation::EngineFailure { phase, .. },
            ) => Some(Self::SpecFailed(*phase)),
            (
                ExecutionObservation::EngineFailure { .. },
                ExecutionObservation::EngineFailure { .. },
            ) => None,
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
            ) => None,
        }
    }
}
#[derive(Debug, Clone, Copy, Serialize)]
pub struct ObjectReductionSummary {
    attempted_replays: u16,
    accepted_reductions: u16,
    stop: ArithmeticReductionStop,
}
pub(super) enum Outcome {
    Verified {
        case: DifferentialReplayInput,
        report: DifferentialReport,
    },
    ReducedMismatch {
        case: DifferentialReplayInput,
        report: DifferentialReport,
        reduction: ObjectReductionSummary,
    },
    Rejected {
        report: DifferentialReport,
    },
}
pub(super) fn run_with_replay(
    plan: ObjectGenerationPlan,
    limit: ArithmeticReductionLimit,
    mut replay: impl FnMut(&DifferentialReplayInput) -> Result<DifferentialReport, DifferentialError>,
) -> Result<Outcome, DifferentialError> {
    let mut program = generate(plan)?;
    let case = program.case(plan)?;
    let mut report = replay(&case)?;
    if report.verdict() == DifferentialVerdict::SelectedObjectProbeAndPrintTranscriptMatch {
        return Ok(Outcome::Verified { case, report });
    }
    let Some(witness) = Witness::from_report(&report) else {
        return Ok(Outcome::Rejected { report });
    };
    let mut attempted_replays = 0;
    let mut accepted_reductions = 0;
    let stop = 'reductions: loop {
        let mut accepted = false;
        for candidate in program.reductions() {
            if attempted_replays == limit.get() {
                break 'reductions ArithmeticReductionStop::ReplayLimitReached;
            }
            attempted_replays += 1;
            let candidate_report = replay(&candidate.case(plan)?)?;
            if candidate_report.verdict() == DifferentialVerdict::WorkerFailure {
                return Ok(Outcome::Rejected {
                    report: candidate_report,
                });
            }
            if Witness::from_report(&candidate_report) == Some(witness) {
                program = candidate;
                report = candidate_report;
                accepted_reductions += 1;
                accepted = true;
                break;
            }
        }
        if !accepted {
            break ArithmeticReductionStop::FixedPoint;
        }
    };
    Ok(Outcome::ReducedMismatch {
        case: program.case(plan)?,
        report,
        reduction: ObjectReductionSummary {
            attempted_replays,
            accepted_reductions,
            stop,
        },
    })
}

#[cfg(test)]
mod tests;
