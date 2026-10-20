//! Bounded source trees exercise real completion and suspension algorithms.
//! Expected traces are produced only by the selected backends, never by a model.
use super::*;
use serde::Serialize;
use std::fmt::Write as _;

pub const CONTROL_FLOW_GRAMMAR: &str = "control-flow-v1";
pub const MAX_CONTROL_FLOW_STEPS: usize = 32;
const MAX_DEPTH: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ControlFlowGenerationPlan {
    seed: u64,
    steps: u8,
    depth: u8,
}
impl ControlFlowGenerationPlan {
    pub fn new(seed: u64, steps: usize, depth: usize) -> Result<Self, DifferentialError> {
        if !(1..=MAX_CONTROL_FLOW_STEPS).contains(&steps) || !(1..=MAX_DEPTH).contains(&depth) {
            return Err(DifferentialError::InvalidGeneration(
                "control-flow-v1 needs 1..=32 steps and 1..=4 depth".into(),
            ));
        }
        Ok(Self {
            seed,
            steps: steps as u8,
            depth: depth as u8,
        })
    }
    pub const fn seed(self) -> u64 {
        self.seed
    }
    pub const fn steps(self) -> u8 {
        self.steps
    }
    pub const fn depth(self) -> u8 {
        self.depth
    }
    pub(super) fn with_seed(self, seed: u64) -> Self {
        Self { seed, ..self }
    }
    fn execution(self) -> Execution {
        match self.seed % 4 {
            0 => Execution::Ordinary,
            1 => Execution::Generator,
            2 => Execution::Async,
            _ => Execution::AsyncGenerator,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Execution {
    Ordinary,
    Generator,
    Async,
    AsyncGenerator,
}
impl Execution {
    fn name(self) -> &'static str {
        match self {
            Self::Ordinary => "ordinary",
            Self::Generator => "generator",
            Self::Async => "async",
            Self::AsyncGenerator => "async-generator",
        }
    }
    fn accepts(self, suspension: Suspension) -> bool {
        match (self, suspension) {
            (Self::Ordinary, _)
            | (Self::Generator, Suspension::Await)
            | (Self::Async, Suspension::Yield) => false,
            (Self::Generator | Self::AsyncGenerator, Suspension::Yield)
            | (Self::Async | Self::AsyncGenerator, Suspension::Await) => true,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Request {
    Next,
    Return,
    Throw,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Suspension {
    Await,
    Yield,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LoopTarget {
    Current,
    Outer,
}
impl LoopTarget {
    fn distance(self) -> usize {
        match self {
            Self::Current => 1,
            Self::Outer => 2,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
enum Leaf {
    Observe,
    Update(i8),
    Capture,
    Suspend(Suspension, i8),
    Return(i8),
    Throw(i8),
    Break(LoopTarget),
    Continue(LoopTarget),
}
impl Leaf {
    fn valid(&self, execution: Execution, loops: usize) -> bool {
        match self {
            Self::Observe | Self::Capture => true,
            Self::Update(value) | Self::Return(value) | Self::Throw(value) => {
                (-16..=16).contains(value)
            }
            Self::Suspend(kind, value) => execution.accepts(*kind) && (-16..=16).contains(value),
            Self::Break(target) | Self::Continue(target) => loops >= target.distance(),
        }
    }
    fn weight(&self) -> usize {
        match self {
            Self::Observe => 0,
            Self::Update(value)
            | Self::Return(value)
            | Self::Throw(value)
            | Self::Suspend(_, value) => 1 + usize::from(value.unsigned_abs()),
            Self::Capture => 1,
            Self::Break(target) | Self::Continue(target) => target.distance(),
        }
    }
    fn reductions(&self) -> Vec<Self> {
        let mut result = vec![];
        if *self != Self::Observe {
            result.push(Self::Observe);
        }
        let zero = match self {
            Self::Update(value) if *value != 0 => Some(Self::Update(0)),
            Self::Return(value) if *value != 0 => Some(Self::Return(0)),
            Self::Throw(value) if *value != 0 => Some(Self::Throw(0)),
            Self::Suspend(kind, value) if *value != 0 => Some(Self::Suspend(*kind, 0)),
            Self::Break(LoopTarget::Outer) => Some(Self::Break(LoopTarget::Current)),
            Self::Continue(LoopTarget::Outer) => Some(Self::Continue(LoopTarget::Current)),
            Self::Observe
            | Self::Capture
            | Self::Update(_)
            | Self::Return(_)
            | Self::Throw(_)
            | Self::Suspend(_, _)
            | Self::Break(_)
            | Self::Continue(_) => None,
        };
        result.extend(zero);
        result
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
enum Node {
    Leaf(Leaf),
    Block(Box<Node>),
    If {
        selected: bool,
        body: Box<Node>,
        otherwise: i8,
    },
    Loop {
        iterations: u8,
        body: Box<Node>,
    },
    TryFinally {
        body: Box<Node>,
        finalizer: Leaf,
    },
    TryCatch {
        body: Box<Node>,
        handler: Leaf,
    },
}
impl Node {
    fn depth(&self) -> usize {
        match self {
            Self::Leaf(_) => 1,
            Self::Block(body)
            | Self::If { body, .. }
            | Self::Loop { body, .. }
            | Self::TryFinally { body, .. }
            | Self::TryCatch { body, .. } => 1 + body.depth(),
        }
    }
    fn valid(&self, execution: Execution, loops: usize, depth: usize) -> bool {
        if depth > MAX_DEPTH {
            return false;
        }
        match self {
            Self::Leaf(leaf) => leaf.valid(execution, loops),
            Self::Block(body) => body.valid(execution, loops, depth + 1),
            Self::If {
                body, otherwise, ..
            } => (-16..=16).contains(otherwise) && body.valid(execution, loops, depth + 1),
            Self::Loop { iterations, body } => {
                *iterations <= 3 && body.valid(execution, loops + 1, depth + 1)
            }
            Self::TryFinally { body, finalizer } => {
                body.valid(execution, loops, depth + 1) && finalizer.valid(execution, loops)
            }
            Self::TryCatch { body, handler } => {
                body.valid(execution, loops, depth + 1) && handler.valid(execution, loops)
            }
        }
    }
    fn complexity(&self) -> (usize, usize) {
        match self {
            Self::Leaf(leaf) => (1, leaf.weight()),
            Self::Block(body) => {
                let (nodes, weight) = body.complexity();
                (nodes + 1, weight)
            }
            Self::If {
                selected,
                body,
                otherwise,
            } => {
                let (nodes, weight) = body.complexity();
                (
                    nodes + 1,
                    weight + usize::from(*selected) + usize::from(otherwise.unsigned_abs()),
                )
            }
            Self::Loop { iterations, body } => {
                let (nodes, weight) = body.complexity();
                (nodes + 1, weight + usize::from(*iterations))
            }
            Self::TryFinally { body, finalizer } => {
                let (nodes, weight) = body.complexity();
                (nodes + 2, weight + finalizer.weight())
            }
            Self::TryCatch { body, handler } => {
                let (nodes, weight) = body.complexity();
                (nodes + 2, weight + handler.weight())
            }
        }
    }
    fn reductions(&self) -> Vec<Self> {
        match self {
            Self::Leaf(leaf) => leaf.reductions().into_iter().map(Self::Leaf).collect(),
            Self::Block(body) => std::iter::once((**body).clone())
                .chain(
                    body.reductions()
                        .into_iter()
                        .map(|body| Self::Block(Box::new(body))),
                )
                .collect(),
            Self::If {
                selected,
                body,
                otherwise,
            } => {
                let mut result = vec![(**body).clone()];
                result.extend(body.reductions().into_iter().map(|body| Self::If {
                    selected: *selected,
                    body: Box::new(body),
                    otherwise: *otherwise,
                }));
                if *otherwise != 0 {
                    result.push(Self::If {
                        selected: *selected,
                        body: body.clone(),
                        otherwise: 0,
                    });
                }
                if *selected {
                    result.push(Self::If {
                        selected: false,
                        body: body.clone(),
                        otherwise: *otherwise,
                    });
                }
                result
            }
            Self::Loop { iterations, body } => {
                let mut result = vec![(**body).clone()];
                result.extend(body.reductions().into_iter().map(|body| Self::Loop {
                    iterations: *iterations,
                    body: Box::new(body),
                }));
                if *iterations > 0 {
                    result.push(Self::Loop {
                        iterations: 0,
                        body: body.clone(),
                    });
                }
                if *iterations > 1 {
                    result.push(Self::Loop {
                        iterations: 1,
                        body: body.clone(),
                    });
                }
                result
            }
            Self::TryFinally { body, finalizer } => {
                let mut result = vec![(**body).clone()];
                result.extend(body.reductions().into_iter().map(|body| Self::TryFinally {
                    body: Box::new(body),
                    finalizer: finalizer.clone(),
                }));
                result.extend(finalizer.reductions().into_iter().map(|finalizer| {
                    Self::TryFinally {
                        body: body.clone(),
                        finalizer,
                    }
                }));
                result
            }
            Self::TryCatch { body, handler } => {
                let mut result = vec![(**body).clone()];
                result.extend(body.reductions().into_iter().map(|body| Self::TryCatch {
                    body: Box::new(body),
                    handler: handler.clone(),
                }));
                result.extend(
                    handler
                        .reductions()
                        .into_iter()
                        .map(|handler| Self::TryCatch {
                            body: body.clone(),
                            handler,
                        }),
                );
                result
            }
        }
    }
}

/// The only renderer input. Reductions remint this proof, so deleting a loop
/// cannot leave an orphan Break/Continue or a suspension in another protocol.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Program {
    execution: Execution,
    strict: bool,
    request: Request,
    steps: Vec<Node>,
}
impl Program {
    fn new(execution: Execution, strict: bool, request: Request, steps: Vec<Node>) -> Option<Self> {
        if steps.is_empty()
            || steps.len() > MAX_CONTROL_FLOW_STEPS
            || steps.iter().any(|node| !node.valid(execution, 0, 1))
        {
            return None;
        }
        Some(Self {
            execution,
            strict,
            request,
            steps,
        })
    }
    fn complexity(&self) -> (usize, usize, usize) {
        let (nodes, weight) = self
            .steps
            .iter()
            .map(Node::complexity)
            .fold((0, 0), |(an, aw), (bn, bw)| (an + bn, aw + bw));
        (self.steps.len(), nodes, weight)
    }
    fn source(&self) -> String {
        let mut source = String::new();
        if self.strict {
            source.push_str("'use strict';\n");
        }
        source.push_str("let state=0; const probes=[];\n");
        source.push_str(match self.execution {
            Execution::Ordinary => "function subject(){\n",
            Execution::Generator => "function* subject(){\nyield 0;\n",
            Execution::Async => "async function subject(){\nawait 0;\n",
            Execution::AsyncGenerator => "async function* subject(){\nawait 0; yield 0;\n",
        });
        if matches!(
            self.execution,
            Execution::Generator | Execution::AsyncGenerator
        ) {
            // The first driver Next passes the entry checkpoint and stops
            // inside this real Try. Injection cannot be skipped by a generated
            // Return/Throw before a randomly selected Yield.
            source.push_str("try{yield state;\n");
        }
        let mut renderer = Renderer {
            source: &mut source,
            next_id: 0,
            loops: vec![],
        };
        for step in &self.steps {
            renderer.node(step);
        }
        source.push_str("return state;\n");
        if matches!(
            self.execution,
            Execution::Generator | Execution::AsyncGenerator
        ) {
            source.push_str("}finally{print('request-finally');const received=yield state;print('request-finally-resume:'+received);}\n");
        }
        source.push_str("}\nfunction finish(value){print('result:'+value); for(let p=0;p<probes.length;++p){print('capture:'+probes[p]());} print('state:'+state);}\nfunction failed(error){print('throw:'+error);finish(undefined);}\n");
        match self.execution {
            Execution::Ordinary => {
                source.push_str("try{finish(subject());}catch(error){failed(error);}\n")
            }
            Execution::Async => source.push_str("subject().then(finish,failed);\n"),
            Execution::Generator | Execution::AsyncGenerator => {
                let awaited = self.execution == Execution::AsyncGenerator;
                source.push_str(if awaited {
                    "async function drive(){\n"
                } else {
                    "function drive(){\n"
                });
                let prefix = if awaited { "await " } else { "" };
                writeln!(
                    source,
                    "const iterator=subject(); let ordinal=0; let step={prefix}iterator.next();"
                )
                .unwrap();
                source.push_str("print('step:'+step.done+':'+step.value); while(!step.done){\n");
                match self.request {
                    Request::Next => {
                        writeln!(source, "step={prefix}iterator.next(++ordinal);").unwrap()
                    }
                    Request::Return | Request::Throw => {
                        let method = match self.request {
                            Request::Return => "return",
                            Request::Throw => "throw",
                            Request::Next => unreachable!(),
                        };
                        writeln!(source, "if(ordinal===1){{++ordinal;step={prefix}iterator.{method}(7);}}else{{step={prefix}iterator.next(++ordinal);}}").unwrap();
                    }
                }
                source
                    .push_str("print('step:'+step.done+':'+step.value);}\nfinish(step.value);}\n");
                source.push_str(if awaited {
                    "drive().catch(failed);\n"
                } else {
                    "try{drive();}catch(error){failed(error);}\n"
                });
            }
        }
        source.push_str("void 0;\n");
        source
    }
    fn case(
        &self,
        plan: ControlFlowGenerationPlan,
    ) -> Result<DifferentialReplayInput, DifferentialError> {
        if self.execution != plan.execution()
            || self.strict != ((plan.seed / 4) % 2 == 1)
            || self.request != request_for_seed(plan.seed)
            || self.steps.len() > usize::from(plan.steps)
            || self
                .steps
                .iter()
                .any(|step| step.depth() > usize::from(plan.depth))
        {
            return Err(DifferentialError::GeneratorInvariant(
                "control-flow reduction changed its execution contract or source budget".into(),
            ));
        }
        let stem = format!(
            "seed-{:016x}-{}-steps-{:02}-depth-{}",
            plan.seed,
            self.execution.name(),
            plan.steps,
            plan.depth
        );
        DifferentialReplayInput::new_script(
            format!("t25/generated/{CONTROL_FLOW_GRAMMAR}/{stem}"),
            DifferentialProtocol::V3PrimitiveCompletionPrintTranscript,
            format!("differential/v3/generated/{CONTROL_FLOW_GRAMMAR}/{stem}.js"),
            5_000,
            self.source(),
        )
    }
    fn reductions(&self) -> Vec<Self> {
        let mut result = Vec::new();
        let mut push = |steps| {
            if let Some(candidate) = Self::new(self.execution, self.strict, self.request, steps) {
                if candidate.complexity() < self.complexity() && !result.contains(&candidate) {
                    result.push(candidate);
                }
            }
        };
        for (index, step) in self.steps.iter().enumerate() {
            let mut steps = self.steps.clone();
            steps.remove(index);
            push(steps);
            for reduced in step.reductions() {
                let mut steps = self.steps.clone();
                steps[index] = reduced;
                push(steps);
            }
        }
        result
    }
}
struct Renderer<'a> {
    source: &'a mut String,
    next_id: usize,
    loops: Vec<usize>,
}
impl Renderer<'_> {
    fn id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
    fn leaf(&mut self, leaf: &Leaf) {
        let id = self.id();
        match leaf {
            Leaf::Observe => writeln!(self.source, "print('observe-{id}:'+state);").unwrap(),
            Leaf::Update(value) => writeln!(
                self.source,
                "state=(state+({value}))|0; print('update-{id}:'+state);"
            )
            .unwrap(),
            Leaf::Capture => writeln!(
                self.source,
                "{{const captured=state;probes.push(()=>captured);print('save-{id}');}}"
            )
            .unwrap(),
            Leaf::Suspend(Suspension::Await, value) => writeln!(
                self.source,
                "state=(state+(await Promise.resolve({value})))|0; print('await-{id}:'+state);"
            )
            .unwrap(),
            Leaf::Suspend(Suspension::Yield, value) => writeln!(
                self.source,
                "{{const received=yield state+({value});print('resume-{id}:'+received);}}"
            )
            .unwrap(),
            Leaf::Return(value) => writeln!(self.source, "return state+({value});").unwrap(),
            Leaf::Throw(value) => writeln!(self.source, "throw {value};").unwrap(),
            Leaf::Break(target) | Leaf::Continue(target) => {
                let label = self.loops[self.loops.len() - target.distance()];
                let keyword = match leaf {
                    Leaf::Break(_) => "break",
                    Leaf::Continue(_) => "continue",
                    _ => unreachable!(),
                };
                writeln!(self.source, "{keyword} loop_{label};").unwrap();
            }
        }
    }
    fn node(&mut self, node: &Node) {
        match node {
            Node::Leaf(leaf) => self.leaf(leaf),
            Node::Block(body) => {
                self.source.push_str("{\n");
                self.node(body);
                self.source.push_str("}\n");
            }
            Node::If {
                selected,
                body,
                otherwise,
            } => {
                writeln!(self.source, "if({selected}){{").unwrap();
                self.node(body);
                writeln!(
                    self.source,
                    "}}else{{state=(state+({otherwise}))|0;print('else:'+state);}}"
                )
                .unwrap();
            }
            Node::Loop { iterations, body } => {
                let id = self.id();
                writeln!(self.source, "loop_{id}: for(let i_{id}=0;i_{id}<{iterations};++i_{id}){{probes.push(()=>i_{id});").unwrap();
                self.loops.push(id);
                self.node(body);
                self.loops.pop();
                self.source.push_str("}\n");
            }
            Node::TryFinally { body, finalizer } => {
                self.source.push_str("try{\n");
                self.node(body);
                self.source.push_str("}finally{print('finally');\n");
                self.leaf(finalizer);
                self.source.push_str("}\n");
            }
            Node::TryCatch { body, handler } => {
                self.source.push_str("try{\n");
                self.node(body);
                self.source
                    .push_str("}catch(error){print('caught:'+error);\n");
                self.leaf(handler);
                self.source.push_str("}\n");
            }
        }
    }
}
struct Random(u64);
impl Random {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        let mut value = self.0;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
        value ^ (value >> 31)
    }
    fn small(&mut self) -> i8 {
        (self.next() % 33) as i8 - 16
    }
    fn leaf(&mut self, execution: Execution, loops: usize) -> Leaf {
        let value = self.small();
        let mut choices = vec![
            Leaf::Observe,
            Leaf::Update(value),
            Leaf::Capture,
            Leaf::Return(value),
            Leaf::Throw(value),
        ];
        for kind in [Suspension::Await, Suspension::Yield] {
            if execution.accepts(kind) {
                choices.push(Leaf::Suspend(kind, value));
            }
        }
        for target in [LoopTarget::Current, LoopTarget::Outer] {
            if loops >= target.distance() {
                choices.push(Leaf::Break(target));
                choices.push(Leaf::Continue(target));
            }
        }
        choices.swap_remove((self.next() % choices.len() as u64) as usize)
    }
    fn node(&mut self, execution: Execution, loops: usize, depth: u8) -> Node {
        if depth == 1 {
            return Node::Leaf(self.leaf(execution, loops));
        }
        match self.next() % 6 {
            0 => Node::Leaf(self.leaf(execution, loops)),
            1 => Node::Block(Box::new(self.node(execution, loops, depth - 1))),
            2 => Node::If {
                selected: self.next() & 1 == 1,
                body: Box::new(self.node(execution, loops, depth - 1)),
                otherwise: self.small(),
            },
            3 => Node::Loop {
                iterations: (self.next() % 4) as u8,
                body: Box::new(self.node(execution, loops + 1, depth - 1)),
            },
            4 => Node::TryFinally {
                body: Box::new(self.node(execution, loops, depth - 1)),
                finalizer: self.leaf(execution, loops),
            },
            _ => Node::TryCatch {
                body: Box::new(self.node(execution, loops, depth - 1)),
                handler: self.leaf(execution, loops),
            },
        }
    }
}
fn request_for_seed(seed: u64) -> Request {
    match (seed / 8) % 3 {
        0 => Request::Next,
        1 => Request::Return,
        _ => Request::Throw,
    }
}
fn generate(plan: ControlFlowGenerationPlan) -> Result<Program, DifferentialError> {
    let mut random = Random(plan.seed);
    let execution = plan.execution();
    let steps = (0..plan.steps)
        .map(|_| random.node(execution, 0, plan.depth))
        .collect();
    Program::new(
        execution,
        (plan.seed / 4) % 2 == 1,
        request_for_seed(plan.seed),
        steps,
    )
    .ok_or_else(|| {
        DifferentialError::GeneratorInvariant(
            "control-flow generator escaped its checked source tree".into(),
        )
    })
}
pub fn generate_control_flow_case(
    plan: ControlFlowGenerationPlan,
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
    WasmFailed {
        phase: FailurePhase,
        spec_kind: CompletionKindObservation,
    },
    SpecFailed {
        phase: FailurePhase,
        wasm_kind: CompletionKindObservation,
    },
}
impl Witness {
    fn from_report(report: &DifferentialReport) -> Option<Self> {
        if report.protocol() != DifferentialProtocol::V3PrimitiveCompletionPrintTranscript
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
                ExecutionObservation::PrimitiveCompletion { completion, .. },
            ) => Some(Self::WasmFailed {
                phase: *phase,
                spec_kind: completion.kind(),
            }),
            (
                ExecutionObservation::PrimitiveCompletion { completion, .. },
                ExecutionObservation::EngineFailure { phase, .. },
            ) => Some(Self::SpecFailed {
                phase: *phase,
                wasm_kind: completion.kind(),
            }),
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
pub struct ControlFlowReductionSummary {
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
        reduction: ControlFlowReductionSummary,
    },
    Rejected {
        report: DifferentialReport,
    },
}
fn has_finished_trace(program: &Program, report: &DifferentialReport) -> bool {
    let finished = |observation: &BackendObservation| {
        let OutputEventsObservation::Captured { events } = &observation.output_events else {
            return false;
        };
        events
            .last()
            .is_some_and(|event| event.starts_with("state:"))
            && events.iter().any(|event| event.starts_with("result:"))
            && (!matches!(
                program.execution,
                Execution::Generator | Execution::AsyncGenerator
            ) || (events.iter().any(|event| event == "request-finally")
                && events
                    .iter()
                    .any(|event| event.starts_with("request-finally-resume:"))))
    };
    finished(report.wasm_aot()) && finished(report.spec_exec())
}
pub(super) fn run_with_replay(
    plan: ControlFlowGenerationPlan,
    limit: ArithmeticReductionLimit,
    mut replay: impl FnMut(&DifferentialReplayInput) -> Result<DifferentialReport, DifferentialError>,
) -> Result<Outcome, DifferentialError> {
    let mut program = generate(plan)?;
    let case = program.case(plan)?;
    let mut report = replay(&case)?;
    if report.verdict() == DifferentialVerdict::PrimitiveCompletionAndPrintTranscriptMatch {
        if !has_finished_trace(&program, &report) {
            // The journal already retained the actual pair. A pair that
            // silently omitted both drivers must remain red, not be furnished
            // with an expected or fabricated transcript.
            return Err(DifferentialError::GeneratorInvariant(
                "matching control-flow observations omitted the terminal driver/finalizer transcript".into()));
        }
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
        reduction: ControlFlowReductionSummary {
            attempted_replays,
            accepted_reductions,
            stop,
        },
    })
}

#[cfg(test)]
mod tests;
