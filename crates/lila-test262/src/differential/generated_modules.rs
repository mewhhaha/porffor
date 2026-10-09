//! Complete generated source graphs use the existing v4 owner and exact loader.
use super::*;
use lila_engine::{
    EmbeddedModuleEntryInput, EmbeddedModuleResolutionInput, EmbeddedModuleSourceInput,
};
use serde::Serialize;
use std::fmt::Write as _;

pub const MODULE_GRAPH_GRAMMAR: &str = "module-graph-v1";
pub const ASYNC_MODULE_GRAPH_GRAMMAR: &str = "module-graph-v2";
pub const MAX_GENERATED_MODULES: usize = 16;
pub const MAX_GENERATED_MODULE_EDGES: usize = 64;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleGrammar {
    StaticV1,
    AsyncV2,
}
impl ModuleGrammar {
    pub const fn name(self) -> &'static str {
        match self {
            Self::StaticV1 => MODULE_GRAPH_GRAMMAR,
            Self::AsyncV2 => ASYNC_MODULE_GRAPH_GRAMMAR,
        }
    }
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            MODULE_GRAPH_GRAMMAR => Some(Self::StaticV1),
            ASYNC_MODULE_GRAPH_GRAMMAR => Some(Self::AsyncV2),
            _ => None,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModuleGenerationPlan {
    grammar: ModuleGrammar,
    seed: u64,
    modules: u8,
    edges: u8,
}
impl ModuleGenerationPlan {
    pub fn new(seed: u64, modules: usize, edges: usize) -> Result<Self, DifferentialError> {
        Self::for_grammar(ModuleGrammar::StaticV1, seed, modules, edges)
    }
    pub fn for_grammar(
        grammar: ModuleGrammar,
        seed: u64,
        modules: usize,
        edges: usize,
    ) -> Result<Self, DifferentialError> {
        if !(1..=MAX_GENERATED_MODULES).contains(&modules) {
            return Err(DifferentialError::InvalidGeneration(
                "module graph needs 1..=16 modules".into(),
            ));
        }
        let maximum = MAX_GENERATED_MODULE_EDGES.min(match grammar {
            ModuleGrammar::StaticV1 => modules * modules,
            ModuleGrammar::AsyncV2 => modules * (modules - 1) / 2,
        });
        if edges < modules - 1 || edges > maximum {
            return Err(DifferentialError::InvalidGeneration(format!(
                "{} needs {}..={maximum} edges for {modules} modules",
                grammar.name(),
                modules - 1
            )));
        }
        Ok(Self {
            grammar,
            seed,
            modules: modules as u8,
            edges: edges as u8,
        })
    }
    pub const fn grammar(self) -> ModuleGrammar {
        self.grammar
    }
    pub const fn seed(self) -> u64 {
        self.seed
    }
    pub const fn modules(self) -> u8 {
        self.modules
    }
    pub const fn edges(self) -> u8 {
        self.edges
    }
    pub(super) fn with_seed(self, seed: u64) -> Self {
        Self { seed, ..self }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ModuleId(u8);
#[derive(Debug, Clone, PartialEq, Eq)]
enum ImportAttributes {
    None,
    Blue,
    Red,
}
impl ImportAttributes {
    fn wire(&self) -> Vec<(String, String)> {
        match self {
            Self::None => vec![],
            Self::Blue | Self::Red => vec![
                ("flavor".into(), self.flavor().into()),
                ("kind".into(), "probe".into()),
            ],
        }
    }
    fn flavor(&self) -> &'static str {
        match self {
            Self::None => "",
            Self::Blue => "blue",
            Self::Red => "red",
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
enum EdgeLoad {
    Static {
        namespace: bool,
        reexport: bool,
    },
    Dynamic {
        computed_specifier: bool,
        attributes: ImportAttributes,
    },
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct Edge {
    target: ModuleId,
    load: EdgeLoad,
    increment: bool,
}
impl Edge {
    fn namespace(&self) -> bool {
        match self.load {
            EdgeLoad::Static { namespace, .. } => namespace,
            EdgeLoad::Dynamic { .. } => true,
        }
    }
    fn weight(&self) -> usize {
        usize::from(self.increment)
            + match &self.load {
                EdgeLoad::Static {
                    namespace,
                    reexport,
                } => usize::from(*namespace) + usize::from(*reexport),
                EdgeLoad::Dynamic {
                    computed_specifier,
                    attributes,
                } => {
                    4 + usize::from(*computed_specifier)
                        + usize::from(*attributes != ImportAttributes::None)
                }
            }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct Module {
    label: u8,
    initial: i16,
    edges: Vec<Edge>,
    metadata: bool,
    await_before: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct Program {
    grammar: ModuleGrammar,
    modules: Vec<Module>,
}
impl Program {
    fn new(grammar: ModuleGrammar, modules: Vec<Module>) -> Option<Self> {
        if modules.is_empty()
            || modules.len() > MAX_GENERATED_MODULES
            || modules
                .iter()
                .map(|module| module.edges.len())
                .sum::<usize>()
                > MAX_GENERATED_MODULE_EDGES
        {
            return None;
        }
        for (index, module) in modules.iter().enumerate() {
            if !(-32..=32).contains(&module.initial)
                || modules[..index]
                    .iter()
                    .any(|prior| prior.label == module.label)
            {
                return None;
            }
            if grammar == ModuleGrammar::StaticV1 && module.await_before {
                return None;
            }
            for (edge_index, edge) in module.edges.iter().enumerate() {
                if usize::from(edge.target.0) >= modules.len()
                    || module.edges[..edge_index]
                        .iter()
                        .any(|prior| prior.target == edge.target)
                {
                    return None;
                }
                match grammar {
                    ModuleGrammar::StaticV1 if !matches!(edge.load, EdgeLoad::Static { .. }) => {
                        return None
                    }
                    // A strict topological order covers static and dynamic
                    // requests together. Awaiting import can never wait on an
                    // ancestor's unfinished evaluation or on its own module.
                    ModuleGrammar::AsyncV2 if usize::from(edge.target.0) <= index => return None,
                    ModuleGrammar::StaticV1 | ModuleGrammar::AsyncV2 => {}
                }
            }
        }
        let result = Self { grammar, modules };
        if result.reachable().iter().any(|keep| !keep) {
            return None;
        }
        Some(result)
    }
    fn reachable(&self) -> Vec<bool> {
        let mut result = vec![false; self.modules.len()];
        let mut queue = vec![ModuleId(0)];
        while let Some(id) = queue.pop() {
            if result[usize::from(id.0)] {
                continue;
            }
            result[usize::from(id.0)] = true;
            queue.extend(
                self.modules[usize::from(id.0)]
                    .edges
                    .iter()
                    .map(|edge| edge.target),
            );
        }
        result
    }
    fn prune(self) -> Option<Self> {
        let keep = self.reachable();
        let mut map = vec![None; self.modules.len()];
        let mut modules = Vec::new();
        for (index, module) in self.modules.into_iter().enumerate() {
            if keep[index] {
                map[index] = Some(ModuleId(modules.len() as u8));
                modules.push(module);
            }
        }
        for module in &mut modules {
            for edge in &mut module.edges {
                edge.target = map[usize::from(edge.target.0)].unwrap();
            }
        }
        Self::new(self.grammar, modules)
    }
    fn complexity(&self) -> (usize, usize, usize) {
        (
            self.modules.len(),
            self.modules.iter().map(|module| module.edges.len()).sum(),
            self.modules
                .iter()
                .map(|module| {
                    usize::from(module.initial.unsigned_abs())
                        + usize::from(module.metadata)
                        + usize::from(module.await_before)
                        + module.edges.iter().map(Edge::weight).sum::<usize>()
                })
                .sum(),
        )
    }
    fn identity(&self, id: ModuleId) -> String {
        format!("module-{}.mjs", self.modules[usize::from(id.0)].label)
    }
    fn source(&self, id: ModuleId) -> String {
        let module = &self.modules[usize::from(id.0)];
        let mut source = String::new();
        for (index, edge) in module.edges.iter().enumerate() {
            let specifier = format!("./{}", self.identity(edge.target));
            let EdgeLoad::Static {
                namespace,
                reexport,
            } = edge.load
            else {
                continue;
            };
            if namespace {
                writeln!(source, "import * as d{index} from '{specifier}';").unwrap();
            } else {
                writeln!(
                    source,
                    "import {{value as v{index},bump as b{index}}} from '{specifier}';"
                )
                .unwrap();
            }
            if reexport {
                writeln!(
                    source,
                    "export {{value as linked_{index}}} from '{specifier}';"
                )
                .unwrap();
            }
        }
        writeln!(
            source,
            "export let value={};\nexport function bump(){{value=value+1;}}",
            module.initial
        )
        .unwrap();
        if module.await_before {
            writeln!(
                source,
                "await Promise.resolve();\nprint('await-{}:'+value);",
                module.label
            )
            .unwrap();
        }
        for (index, edge) in module.edges.iter().enumerate() {
            let EdgeLoad::Dynamic {
                computed_specifier,
                attributes,
            } = &edge.load
            else {
                continue;
            };
            let identity = self.identity(edge.target);
            let specifier = if *computed_specifier {
                format!("('./'+'{identity}')")
            } else {
                format!("'./{identity}'")
            };
            let options = match attributes {
                ImportAttributes::None => String::new(),
                ImportAttributes::Blue | ImportAttributes::Red => format!(
                    ",{{with:{{kind:'probe',flavor:'{}'}}}}",
                    attributes.flavor()
                ),
            };
            writeln!(source, "const d{index}=await import({specifier}{options});\nprint('import-{}-{index}:'+d{index}.value);", module.label).unwrap();
        }
        source.push_str("export function read(){return value");
        for (index, edge) in module.edges.iter().enumerate() {
            if edge.namespace() {
                write!(source, "+d{index}.value").unwrap();
            } else {
                write!(source, "+v{index}").unwrap();
            }
        }
        source.push_str(";}\n");
        if module.metadata {
            writeln!(
                source,
                "print('module-{}:'+import.meta.url+':'+value);",
                module.label
            )
            .unwrap();
        }
        // Only the entry reads imported live cells after dependency evaluation. No
        // dependency reads a cyclic lexical export during its initialization.
        if id == ModuleId(0) {
            for (index, edge) in module.edges.iter().enumerate() {
                if edge.increment {
                    if edge.namespace() {
                        writeln!(source, "d{index}.bump();").unwrap();
                    } else {
                        writeln!(source, "b{index}();").unwrap();
                    }
                }
                if edge.namespace() {
                    writeln!(source, "print('live-{index}:'+d{index}.value);").unwrap();
                } else {
                    writeln!(source, "print('live-{index}:'+v{index});").unwrap();
                }
            }
            source.push_str("print('sum:'+read());\n");
        }
        source
    }
    fn case(
        &self,
        plan: ModuleGenerationPlan,
    ) -> Result<DifferentialReplayInput, DifferentialError> {
        if plan.grammar != self.grammar {
            return Err(DifferentialError::GeneratorInvariant(
                "module graph changed its generation grammar".into(),
            ));
        }
        let identity = self.identity(ModuleId(0));
        let entry = EmbeddedModuleEntryInput {
            goal: EmbeddedModuleGoal::Module,
            source: self.source(ModuleId(0)),
            meta_url: format!("lila://generated/{identity}"),
            identity,
        };
        let modules = (1..self.modules.len())
            .map(|index| {
                let id = ModuleId(index as u8);
                let identity = self.identity(id);
                EmbeddedModuleSourceInput {
                    source: self.source(id),
                    meta_url: format!("lila://generated/{identity}"),
                    identity,
                }
            })
            .collect();
        let mut resolutions = Vec::new();
        for (index, module) in self.modules.iter().enumerate() {
            for edge in &module.edges {
                let target = self.identity(edge.target);
                resolutions.push(EmbeddedModuleResolutionInput {
                    referrer: EmbeddedModuleReferrer::Module(self.identity(ModuleId(index as u8))),
                    specifier: format!("./{target}"),
                    attributes: match &edge.load {
                        EdgeLoad::Static { .. } => vec![],
                        EdgeLoad::Dynamic { attributes, .. } => attributes.wire(),
                    },
                    target,
                });
            }
        }
        let graph = EmbeddedModuleGraph::try_new(entry, modules, resolutions)
            .map_err(|error| DifferentialError::GeneratorInvariant(error.to_string()))?;
        let stem = format!(
            "seed-{:016x}-modules-{:02}-edges-{:02}",
            plan.seed, plan.modules, plan.edges
        );
        DifferentialReplayInput::new_embedded(
            format!("t25/generated/{}/{stem}", self.grammar.name()),
            5000,
            graph,
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
        for (index, module) in self.modules.iter().enumerate() {
            if module.initial != 0 {
                let mut candidate = self.clone();
                candidate.modules[index].initial = 0;
                push(candidate);
            }
            if module.metadata {
                let mut candidate = self.clone();
                candidate.modules[index].metadata = false;
                push(candidate);
            }
            if module.await_before {
                let mut candidate = self.clone();
                candidate.modules[index].await_before = false;
                push(candidate);
            }
            for (edge_index, edge) in module.edges.iter().enumerate() {
                let mut candidate = self.clone();
                candidate.modules[index].edges.remove(edge_index);
                push(candidate);
                match &edge.load {
                    EdgeLoad::Static {
                        namespace,
                        reexport,
                    } => {
                        if *namespace {
                            let mut candidate = self.clone();
                            candidate.modules[index].edges[edge_index].load = EdgeLoad::Static {
                                namespace: false,
                                reexport: *reexport,
                            };
                            push(candidate);
                        }
                        if *reexport {
                            let mut candidate = self.clone();
                            candidate.modules[index].edges[edge_index].load = EdgeLoad::Static {
                                namespace: *namespace,
                                reexport: false,
                            };
                            push(candidate);
                        }
                    }
                    EdgeLoad::Dynamic {
                        computed_specifier,
                        attributes,
                    } => {
                        let mut candidate = self.clone();
                        candidate.modules[index].edges[edge_index].load = EdgeLoad::Static {
                            namespace: true,
                            reexport: false,
                        };
                        push(candidate);
                        if *computed_specifier {
                            let mut candidate = self.clone();
                            candidate.modules[index].edges[edge_index].load = EdgeLoad::Dynamic {
                                computed_specifier: false,
                                attributes: attributes.clone(),
                            };
                            push(candidate);
                        }
                        if *attributes != ImportAttributes::None {
                            let mut candidate = self.clone();
                            candidate.modules[index].edges[edge_index].load = EdgeLoad::Dynamic {
                                computed_specifier: *computed_specifier,
                                attributes: ImportAttributes::None,
                            };
                            push(candidate);
                        }
                    }
                }
                if edge.increment {
                    let mut candidate = self.clone();
                    candidate.modules[index].edges[edge_index].increment = false;
                    push(candidate);
                }
            }
        }
        result
    }
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
    fn edge(&mut self, target: ModuleId, grammar: ModuleGrammar) -> Edge {
        let load = match grammar {
            ModuleGrammar::StaticV1 => EdgeLoad::Static {
                namespace: self.boolean(),
                reexport: self.boolean(),
            },
            ModuleGrammar::AsyncV2 => {
                if self.boolean() {
                    EdgeLoad::Static {
                        namespace: self.boolean(),
                        reexport: self.boolean(),
                    }
                } else {
                    EdgeLoad::Dynamic {
                        computed_specifier: self.boolean(),
                        attributes: match self.next() % 3 {
                            0 => ImportAttributes::None,
                            1 => ImportAttributes::Blue,
                            _ => ImportAttributes::Red,
                        },
                    }
                }
            }
        };
        Edge {
            target,
            load,
            increment: self.boolean(),
        }
    }
}
fn generate(plan: ModuleGenerationPlan) -> Result<Program, DifferentialError> {
    let mut random = Random(plan.seed);
    let mut modules: Vec<_> = (0..plan.modules)
        .map(|label| Module {
            label,
            initial: (random.next() % 65) as i16 - 32,
            metadata: true,
            edges: vec![],
            await_before: plan.grammar == ModuleGrammar::AsyncV2,
        })
        .collect();
    // A spanning star gives every declared module a real request. Remaining
    // exact edges may introduce self imports and arbitrary static cycles.
    for target in 1..plan.modules {
        let mut edge = random.edge(ModuleId(target), plan.grammar);
        if plan.grammar == ModuleGrammar::AsyncV2 && target == 1 {
            edge.load = EdgeLoad::Dynamic {
                computed_specifier: true,
                attributes: ImportAttributes::Blue,
            };
        }
        modules[0].edges.push(edge);
    }
    let mut available: Vec<_> = (0..plan.modules)
        .flat_map(|source| {
            (0..plan.modules)
                .filter(move |target| match plan.grammar {
                    ModuleGrammar::StaticV1 => source != 0 || *target == 0,
                    ModuleGrammar::AsyncV2 => source != 0 && *target > source,
                })
                .map(move |target| (source, target))
        })
        .collect();
    for _ in usize::from(plan.modules - 1)..usize::from(plan.edges) {
        let index = (random.next() % available.len() as u64) as usize;
        let (source, target) = available.remove(index);
        modules[usize::from(source)]
            .edges
            .push(random.edge(ModuleId(target), plan.grammar));
    }
    Program::new(plan.grammar, modules).ok_or_else(|| {
        DifferentialError::GeneratorInvariant(
            "module generator escaped its checked source graph".into(),
        )
    })
}
pub fn generate_module_graph_case(
    plan: ModuleGenerationPlan,
) -> Result<DifferentialReplayInput, DifferentialError> {
    generate(plan)?.case(plan)
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Witness {
    Compared {
        completion: bool,
        print: bool,
        wasm_kind: CompletionKindObservation,
        spec_kind: CompletionKindObservation,
    },
    WasmFailed(FailurePhase),
    SpecFailed(FailurePhase),
}
impl Witness {
    fn from_report(report: &DifferentialReport) -> Option<Self> {
        if report.protocol() != DifferentialProtocol::V4EmbeddedGraphPrimitivePrintTranscript
            || report.verdict() != DifferentialVerdict::Mismatch
        {
            return None;
        }
        match (&report.wasm_aot().execution, &report.spec_exec().execution) {
            (
                ExecutionObservation::PrimitiveCompletion {
                    completion: left, ..
                },
                ExecutionObservation::PrimitiveCompletion {
                    completion: right, ..
                },
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
                    completion: left != right,
                    print: a != b,
                    wasm_kind: left.kind(),
                    spec_kind: right.kind(),
                })
            }
            (
                ExecutionObservation::EngineFailure { phase, .. },
                ExecutionObservation::PrimitiveCompletion { .. },
            ) => Some(Self::WasmFailed(*phase)),
            (
                ExecutionObservation::PrimitiveCompletion { .. },
                ExecutionObservation::EngineFailure { phase, .. },
            ) => Some(Self::SpecFailed(*phase)),
            (
                ExecutionObservation::EngineFailure { .. },
                ExecutionObservation::EngineFailure { .. },
            ) => None,
            (
                ExecutionObservation::Normal { .. }
                | ExecutionObservation::Error { .. }
                | ExecutionObservation::SelectedObjectProbe { .. }
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
                | ExecutionObservation::SelectedObjectProbe { .. }
                | ExecutionObservation::UnsupportedCompletion { .. }
                | ExecutionObservation::RootedCompletionGraph { .. }
                | ExecutionObservation::ObservationRejected { .. }
                | ExecutionObservation::WorkerFailure { .. },
            ) => None,
        }
    }
}
#[derive(Debug, Clone, Copy, Serialize)]
pub struct ModuleReductionSummary {
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
        reduction: ModuleReductionSummary,
    },
    Rejected {
        report: DifferentialReport,
    },
}
pub(super) fn run_with_replay(
    plan: ModuleGenerationPlan,
    limit: ArithmeticReductionLimit,
    mut replay: impl FnMut(&DifferentialReplayInput) -> Result<DifferentialReport, DifferentialError>,
) -> Result<Outcome, DifferentialError> {
    let mut program = generate(plan)?;
    let case = program.case(plan)?;
    let mut report = replay(&case)?;
    if report.verdict() == DifferentialVerdict::PrimitiveCompletionAndPrintTranscriptMatch {
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
        reduction: ModuleReductionSummary {
            attempted_replays,
            accepted_reductions,
            stop,
        },
    })
}
#[cfg(test)]
mod tests;
