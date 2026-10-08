//! Negative source is generated from a closed grammar and goes through the
//! original entry frontends in selected workers. Expected rejection is a
//! campaign assertion; the actual differential report is never rewritten.
use super::*;
use lila_engine::EmbeddedModuleEntryInput;
use serde::Serialize;
use std::fmt::Write as _;

pub const NEGATIVE_SOURCE_GRAMMAR: &str = "negative-source-v1";
const MAX_STEPS: usize = 32;
const MAX_DEPTH: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NegativeGenerationPlan {
    seed: u64,
    steps: u8,
    depth: u8,
}
impl NegativeGenerationPlan {
    pub fn new(seed: u64, steps: usize, depth: usize) -> Result<Self, DifferentialError> {
        if !(1..=MAX_STEPS).contains(&steps) || !(1..=MAX_DEPTH).contains(&depth) {
            return Err(DifferentialError::InvalidGeneration(
                "negative-source-v1 needs 1..=32 statements and 1..=4 depth".into(),
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
    fn family(self) -> NegativeSourceFamily {
        NegativeSourceFamily::ALL[(self.seed % 8) as usize]
    }
    fn goal(self) -> DifferentialGoal {
        if self.seed / 8 % 2 == 0 {
            DifferentialGoal::Script
        } else {
            DifferentialGoal::Module
        }
    }
    fn strict(self) -> bool {
        self.goal() == DifferentialGoal::Module
            || self.family() == NegativeSourceFamily::StrictDelete
            || self.seed / 16 % 2 == 1
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NegativeSourcePhase {
    Parse,
    EarlyError,
}
impl NegativeSourcePhase {
    fn failure(self) -> FailurePhase {
        match self {
            Self::Parse => FailurePhase::Parse,
            Self::EarlyError => FailurePhase::EarlyError,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NegativeSourceFamily {
    MissingInitializer,
    UnterminatedString,
    DuplicateLexical,
    DuplicateProto,
    CoverInitializedName,
    SuperWithoutMethod,
    DuplicatePrivate,
    StrictDelete,
}
impl NegativeSourceFamily {
    const ALL: [Self; 8] = [
        Self::MissingInitializer,
        Self::UnterminatedString,
        Self::DuplicateLexical,
        Self::DuplicateProto,
        Self::CoverInitializedName,
        Self::SuperWithoutMethod,
        Self::DuplicatePrivate,
        Self::StrictDelete,
    ];
    fn phase(self) -> NegativeSourcePhase {
        match self {
            Self::MissingInitializer | Self::UnterminatedString => NegativeSourcePhase::Parse,
            Self::DuplicateLexical
            | Self::DuplicateProto
            | Self::CoverInitializedName
            | Self::SuperWithoutMethod
            | Self::DuplicatePrivate
            | Self::StrictDelete => NegativeSourcePhase::EarlyError,
        }
    }
}

/// A reduction must preserve the actual offending construct, goal and
/// strictness. Only irrelevant statement owners, nesting and bounded literal
/// or identifier payloads may change.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Program {
    family: NegativeSourceFamily,
    goal: DifferentialGoal,
    strict: bool,
    depth: u8,
    name: u8,
    value: i8,
    padding: Vec<i8>,
}
impl Program {
    fn new(
        family: NegativeSourceFamily,
        goal: DifferentialGoal,
        strict: bool,
        depth: u8,
        name: u8,
        value: i8,
        padding: Vec<i8>,
    ) -> Option<Self> {
        if !(1..=MAX_DEPTH as u8).contains(&depth)
            || name > 31
            || !(-16..=16).contains(&value)
            || padding.len() >= MAX_STEPS
            || padding.iter().any(|value| !(-16..=16).contains(value))
            || ((goal == DifferentialGoal::Module || family == NegativeSourceFamily::StrictDelete)
                && !strict)
        {
            return None;
        }
        Some(Self {
            family,
            goal,
            strict,
            depth,
            name,
            value,
            padding,
        })
    }
    fn source(&self) -> String {
        let mut source = String::new();
        if self.strict {
            source.push_str("'use strict';\n");
        }
        for (index, value) in self.padding.iter().enumerate() {
            writeln!(source, "let padding_{index} = {value};").unwrap();
        }
        for _ in 1..self.depth {
            source.push_str("{\n");
        }
        let name = format!("binding_{}", self.name);
        match self.family {
            NegativeSourceFamily::MissingInitializer => writeln!(source, "let {name} = ;").unwrap(),
            NegativeSourceFamily::UnterminatedString => {
                writeln!(source, "let {name} = 'unterminated;").unwrap()
            }
            NegativeSourceFamily::DuplicateLexical => {
                writeln!(source, "{{let {name}; let {name};}}").unwrap()
            }
            NegativeSourceFamily::DuplicateProto => {
                source.push_str("({__proto__:null,__proto__:null});\n")
            }
            NegativeSourceFamily::CoverInitializedName => {
                writeln!(source, "({{{name} = {}}});", self.value).unwrap()
            }
            NegativeSourceFamily::SuperWithoutMethod => writeln!(source, "super.{name};").unwrap(),
            NegativeSourceFamily::DuplicatePrivate => {
                writeln!(source, "class Broken {{ #{name}; #{name}; }}").unwrap()
            }
            NegativeSourceFamily::StrictDelete => {
                writeln!(source, "let {name}; delete {name};").unwrap()
            }
        }
        for _ in 1..self.depth {
            source.push_str("}\n");
        }
        source
    }
    fn complexity(&self) -> (usize, u8, usize) {
        (
            self.padding.len(),
            self.depth,
            usize::from(self.name)
                + usize::from(self.value.unsigned_abs())
                + self
                    .padding
                    .iter()
                    .map(|value| usize::from(value.unsigned_abs()))
                    .sum::<usize>(),
        )
    }
    fn reductions(&self) -> Vec<Self> {
        let mut result = Vec::new();
        let mut push = |candidate: Self| {
            if let Some(candidate) = Self::new(
                candidate.family,
                candidate.goal,
                candidate.strict,
                candidate.depth,
                candidate.name,
                candidate.value,
                candidate.padding,
            ) {
                if candidate.complexity() < self.complexity() && !result.contains(&candidate) {
                    result.push(candidate);
                }
            }
        };
        for index in 0..self.padding.len() {
            let mut candidate = self.clone();
            candidate.padding.remove(index);
            push(candidate);
        }
        if self.depth > 1 {
            let mut candidate = self.clone();
            candidate.depth = 1;
            push(candidate);
        }
        if self.name > 0 {
            let mut candidate = self.clone();
            candidate.name = 0;
            push(candidate);
        }
        if self.value != 0 {
            let mut candidate = self.clone();
            candidate.value = 0;
            push(candidate);
        }
        for index in 0..self.padding.len() {
            if self.padding[index] != 0 {
                let mut candidate = self.clone();
                candidate.padding[index] = 0;
                push(candidate);
            }
        }
        result
    }
    fn case(
        &self,
        plan: NegativeGenerationPlan,
    ) -> Result<GeneratedNegativeCase, DifferentialError> {
        if self.padding.len() + 1 > usize::from(plan.steps)
            || self.depth > plan.depth
            || self.family != plan.family()
            || self.goal != plan.goal()
            || self.strict != plan.strict()
        {
            return Err(DifferentialError::GeneratorInvariant(
                "negative source exceeded its original budget".into(),
            ));
        }
        let identity = match self.goal {
            DifferentialGoal::Script => "negative.js",
            DifferentialGoal::Module => "negative.mjs",
        };
        // V4's source catalog is admitted without parsing its entry. The worker
        // then uses the original selected backend with a complete, empty
        // dependency catalog; no ambient filesystem loader is introduced.
        let graph = EmbeddedModuleGraph::try_new(
            EmbeddedModuleEntryInput {
                goal: match self.goal {
                    DifferentialGoal::Script => EmbeddedModuleGoal::Script,
                    DifferentialGoal::Module => EmbeddedModuleGoal::Module,
                },
                identity: identity.into(),
                source: self.source(),
                meta_url: format!("lila-generated:{identity}"),
            },
            vec![],
            vec![],
        )
        .map_err(|error| DifferentialError::GeneratorInvariant(error.to_string()))?;
        let input = DifferentialReplayInput::new_embedded(
            format!(
                "t25/generated/{NEGATIVE_SOURCE_GRAMMAR}/seed-{:016x}-steps-{:02}-depth-{}",
                plan.seed, plan.steps, plan.depth
            ),
            5000,
            graph,
        )?;
        Ok(GeneratedNegativeCase {
            input,
            family: self.family,
            expected_phase: self.family.phase(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct GeneratedNegativeCase {
    input: DifferentialReplayInput,
    family: NegativeSourceFamily,
    expected_phase: NegativeSourcePhase,
}
impl GeneratedNegativeCase {
    pub fn input(&self) -> &DifferentialReplayInput {
        &self.input
    }
    pub const fn family(&self) -> NegativeSourceFamily {
        self.family
    }
    pub const fn expected_phase(&self) -> NegativeSourcePhase {
        self.expected_phase
    }
}

fn generate(plan: NegativeGenerationPlan) -> Result<Program, DifferentialError> {
    let mut state = plan.seed ^ 0x8ad7_962f_13c5_e04b;
    let mut draw = || {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        state
    };
    let name = (draw() % 32) as u8;
    let value = (draw() % 33) as i8 - 16;
    let padding = (1..plan.steps).map(|_| (draw() % 33) as i8 - 16).collect();
    Program::new(
        plan.family(),
        plan.goal(),
        plan.strict(),
        plan.depth,
        name,
        value,
        padding,
    )
    .ok_or_else(|| {
        DifferentialError::GeneratorInvariant(
            "negative generator produced an invalid source owner".into(),
        )
    })
}
pub fn generate_negative_case(
    plan: NegativeGenerationPlan,
) -> Result<GeneratedNegativeCase, DifferentialError> {
    generate(plan)?.case(plan)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum FrontendResult {
    Rejected {
        phase: FailurePhase,
    },
    Completed {
        completion: CompletionKindObservation,
    },
}
impl FrontendResult {
    fn from_observation(observation: &BackendObservation) -> Option<Self> {
        match &observation.output_events {
            OutputEventsObservation::Captured { events } if events.is_empty() => {}
            OutputEventsObservation::Captured { .. }
            | OutputEventsObservation::Incomplete { .. }
            | OutputEventsObservation::Unavailable { .. } => return None,
        }
        match &observation.execution {
            ExecutionObservation::EngineFailure { phase, .. } => {
                Some(Self::Rejected { phase: *phase })
            }
            ExecutionObservation::PrimitiveCompletion { completion, .. } => Some(Self::Completed {
                completion: completion.kind(),
            }),
            ExecutionObservation::Normal { .. }
            | ExecutionObservation::Error { .. }
            | ExecutionObservation::SelectedObjectProbe { .. }
            | ExecutionObservation::RootedCompletionGraph { .. }
            | ExecutionObservation::ObservationRejected { .. }
            | ExecutionObservation::UnsupportedCompletion { .. }
            | ExecutionObservation::WorkerFailure { .. } => None,
        }
    }
}

/// Stable mismatch dimensions survive minimization; diagnostic wording and
/// changing source fingerprints stay in the original reports beside them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct NegativeMismatchSignature {
    version: u8,
    expected_product_phase: NegativeSourcePhase,
    wasm_aot: FrontendResult,
    spec_exec: FrontendResult,
}
impl NegativeMismatchSignature {
    fn from_report(expected: NegativeSourcePhase, report: &DifferentialReport) -> Option<Self> {
        Some(Self {
            version: 1,
            expected_product_phase: expected,
            wasm_aot: FrontendResult::from_observation(report.wasm_aot())?,
            spec_exec: FrontendResult::from_observation(report.spec_exec())?,
        })
    }
    fn is_expected(self) -> bool {
        self.wasm_aot
            == (FrontendResult::Rejected {
                phase: self.expected_product_phase.failure(),
            })
            && self.spec_exec
                == (FrontendResult::Rejected {
                    phase: FailurePhase::SpecExecEntrySyntax,
                })
    }
}

#[derive(Debug, Serialize)]
pub struct NegativeSourceObservation {
    case: DifferentialReplayInput,
    family: NegativeSourceFamily,
    expected_product_phase: NegativeSourcePhase,
    expected_oracle_phase: FailurePhase,
    report: DifferentialReport,
    mismatch: Option<NegativeMismatchSignature>,
    expected_rejection: bool,
}
impl NegativeSourceObservation {
    pub const fn is_green(&self) -> bool {
        self.expected_rejection
    }
    pub fn input(&self) -> &DifferentialReplayInput {
        &self.case
    }
    pub fn report(&self) -> &DifferentialReport {
        &self.report
    }
    pub const fn family(&self) -> NegativeSourceFamily {
        self.family
    }
    pub const fn expected_phase(&self) -> NegativeSourcePhase {
        self.expected_product_phase
    }
    pub const fn mismatch_signature(&self) -> Option<NegativeMismatchSignature> {
        self.mismatch
    }
    pub(super) fn worker_failed(&self) -> bool {
        self.report.verdict() == DifferentialVerdict::WorkerFailure
    }
}
#[derive(Debug, Clone, Copy, Serialize)]
pub struct NegativeReductionSummary {
    attempted_replays: u16,
    accepted_reductions: u16,
    stop: ArithmeticReductionStop,
}
pub(super) enum Outcome {
    Verified {
        observation: NegativeSourceObservation,
    },
    ReducedMismatch {
        observation: NegativeSourceObservation,
        reduction: NegativeReductionSummary,
    },
    Rejected {
        observation: NegativeSourceObservation,
    },
}
fn observation(
    case: GeneratedNegativeCase,
    report: DifferentialReport,
) -> Result<NegativeSourceObservation, DifferentialError> {
    if report.case_id() != case.input.id()
        || report.case_fingerprint != input_fingerprint(&case.input)
        || report.protocol() != case.input.protocol()
    {
        return Err(DifferentialError::GeneratorInvariant(
            "negative replay returned a foreign source/protocol observation".into(),
        ));
    }
    let witness = NegativeMismatchSignature::from_report(case.expected_phase, &report);
    let expected_rejection = witness.is_some_and(NegativeMismatchSignature::is_expected);
    Ok(NegativeSourceObservation {
        case: case.input,
        family: case.family,
        expected_product_phase: case.expected_phase,
        expected_oracle_phase: FailurePhase::SpecExecEntrySyntax,
        report,
        mismatch: witness.filter(|witness| !witness.is_expected()),
        expected_rejection,
    })
}
pub(super) fn run_with_replay(
    plan: NegativeGenerationPlan,
    limit: ArithmeticReductionLimit,
    mut replay: impl FnMut(&DifferentialReplayInput) -> Result<DifferentialReport, DifferentialError>,
) -> Result<Outcome, DifferentialError> {
    let mut program = generate(plan)?;
    let case = program.case(plan)?;
    let report = replay(case.input())?;
    let mut current = observation(case, report)?;
    if current.is_green() {
        return Ok(Outcome::Verified {
            observation: current,
        });
    }
    let Some(witness) = current.mismatch else {
        return Ok(Outcome::Rejected {
            observation: current,
        });
    };
    let mut attempted_replays = 0;
    let mut accepted_reductions = 0;
    let stop = 'reduction: loop {
        let mut accepted = false;
        for candidate in program.reductions() {
            if attempted_replays == limit.get() {
                break 'reduction ArithmeticReductionStop::ReplayLimitReached;
            }
            attempted_replays += 1;
            let case = candidate.case(plan)?;
            let report = replay(case.input())?;
            let next = observation(case, report)?;
            if next.worker_failed() {
                return Ok(Outcome::Rejected { observation: next });
            }
            if next.mismatch == Some(witness) {
                program = candidate;
                current = next;
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
        observation: current,
        reduction: NegativeReductionSummary {
            attempted_replays,
            accepted_reductions,
            stop,
        },
    })
}

#[cfg(test)]
mod tests;
