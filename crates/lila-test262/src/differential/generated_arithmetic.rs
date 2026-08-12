use std::fmt::Write as _;
use std::num::{NonZeroU16, NonZeroU8};

use serde::Serialize;

use super::{
    replay_case, DifferentialCase, DifferentialError, DifferentialGoal, DifferentialReport,
    DifferentialVerdict, ExecutionObservation, FailurePhase, ObservationContract, SpecExecOracle,
};

const GENERATOR_VERSION: &str = "integer-arithmetic-v1";
const GENERATED_CASE_TIMEOUT_MS: u64 = 5_000;
const SMALL_INTEGER_MIN: i16 = -32;
const MAX_SAFE_INTEGER: i64 = 9_007_199_254_740_991;

pub const MAX_ARITHMETIC_CHECKS: u8 = 32;
pub const MAX_ARITHMETIC_REDUCTION_REPLAYS: u16 = 512;

/// The deterministic input to SplitMix64-v1.
///
/// Every `u64` is a valid seed. Keeping it distinct from the replay and check
/// budgets makes swapping those three CLI values a type error after parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ArithmeticGenerationSeed(u64);

impl ArithmeticGenerationSeed {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

/// A non-zero, resource-bounded number of independent self checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArithmeticCheckCount(NonZeroU8);

impl ArithmeticCheckCount {
    pub fn new(value: usize) -> Result<Self, DifferentialError> {
        let value = u8::try_from(value)
            .ok()
            .and_then(NonZeroU8::new)
            .filter(|value| value.get() <= MAX_ARITHMETIC_CHECKS)
            .ok_or_else(|| {
                DifferentialError::InvalidGeneration(format!(
                    "arithmetic check count must be in 1..={MAX_ARITHMETIC_CHECKS}"
                ))
            })?;
        Ok(Self(value))
    }

    pub const fn get(self) -> u8 {
        self.0.get()
    }
}

/// The closed expression-depth domain supported by generator version 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArithmeticExpressionDepth {
    One,
    Two,
    Three,
    Four,
}

impl ArithmeticExpressionDepth {
    pub fn new(value: usize) -> Result<Self, DifferentialError> {
        match value {
            1 => Ok(Self::One),
            2 => Ok(Self::Two),
            3 => Ok(Self::Three),
            4 => Ok(Self::Four),
            _ => Err(DifferentialError::InvalidGeneration(
                "arithmetic expression depth must be one of 1, 2, 3, or 4".to_string(),
            )),
        }
    }

    pub const fn get(self) -> u8 {
        match self {
            Self::One => 1,
            Self::Two => 2,
            Self::Three => 3,
            Self::Four => 4,
        }
    }
}

/// A non-zero upper bound on candidate replays during reduction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArithmeticReductionLimit(NonZeroU16);

impl ArithmeticReductionLimit {
    pub fn new(value: usize) -> Result<Self, DifferentialError> {
        let value = u16::try_from(value)
            .ok()
            .and_then(NonZeroU16::new)
            .filter(|value| value.get() <= MAX_ARITHMETIC_REDUCTION_REPLAYS)
            .ok_or_else(|| {
                DifferentialError::InvalidGeneration(format!(
                    "arithmetic reduction replay limit must be in 1..={MAX_ARITHMETIC_REDUCTION_REPLAYS}"
                ))
            })?;
        Ok(Self(value))
    }

    pub const fn get(self) -> u16 {
        self.0.get()
    }
}

/// One complete deterministic generation request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArithmeticGenerationPlan {
    seed: ArithmeticGenerationSeed,
    checks: ArithmeticCheckCount,
    depth: ArithmeticExpressionDepth,
}

impl ArithmeticGenerationPlan {
    pub const fn new(
        seed: ArithmeticGenerationSeed,
        checks: ArithmeticCheckCount,
        depth: ArithmeticExpressionDepth,
    ) -> Self {
        Self {
            seed,
            checks,
            depth,
        }
    }

    pub const fn seed(self) -> ArithmeticGenerationSeed {
        self.seed
    }

    pub const fn checks(self) -> ArithmeticCheckCount {
        self.checks
    }

    pub const fn depth(self) -> ArithmeticExpressionDepth {
        self.depth
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ArithmeticReductionStop {
    FixedPoint,
    ReplayLimitReached,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ArithmeticReductionSummary {
    attempted_replays: u16,
    accepted_reductions: u16,
    stop: ArithmeticReductionStop,
}

impl ArithmeticReductionSummary {
    pub const fn attempted_replays(self) -> u16 {
        self.attempted_replays
    }

    pub const fn accepted_reductions(self) -> u16 {
        self.accepted_reductions
    }

    pub const fn stop(self) -> ArithmeticReductionStop {
        self.stop
    }
}

/// The closed result of one generated campaign.
///
/// `Verified` and `ReducedMismatch` contain a useful replayable corpus case.
/// `Rejected` retains the red observation but is deliberately not a candidate
/// for corpus persistence.
#[derive(Debug)]
pub enum GeneratedArithmeticCampaignOutcome {
    Verified {
        case: DifferentialCase,
        report: DifferentialReport,
    },
    ReducedMismatch {
        case: DifferentialCase,
        report: DifferentialReport,
        reduction: ArithmeticReductionSummary,
    },
    Rejected {
        case: DifferentialCase,
        report: DifferentialReport,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArithmeticOp {
    Add,
    Subtract,
}

impl ArithmeticOp {
    const fn symbol(self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Subtract => "-",
        }
    }

    fn apply(self, left: i64, right: i64) -> Option<i64> {
        match self {
            Self::Add => left.checked_add(right),
            Self::Subtract => left.checked_sub(right),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SmallInteger(i16);

impl SmallInteger {
    fn from_random_word(word: u64) -> Self {
        let offset = (word % 65) as i32;
        Self((i32::from(SMALL_INTEGER_MIN) + offset) as i16)
    }

    fn reductions(self) -> Vec<Self> {
        if self.0 == 0 {
            return Vec::new();
        }

        let mut values = Vec::with_capacity(3);
        for value in [0, self.0.signum(), self.0 / 2] {
            let candidate = Self(value);
            if candidate != self && !values.contains(&candidate) {
                values.push(candidate);
            }
        }
        values
    }

    const fn get(self) -> i16 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ExactInteger(i64);

impl ExactInteger {
    fn new(value: i64) -> Option<Self> {
        (-MAX_SAFE_INTEGER..=MAX_SAFE_INTEGER)
            .contains(&value)
            .then_some(Self(value))
    }

    const fn get(self) -> i64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ArithmeticExpr {
    Literal(SmallInteger),
    Binary {
        op: ArithmeticOp,
        left: Box<Self>,
        right: Box<Self>,
    },
}

impl ArithmeticExpr {
    fn evaluate(&self) -> Option<ExactInteger> {
        let value = match self {
            Self::Literal(value) => i64::from(value.get()),
            Self::Binary { op, left, right } => {
                op.apply(left.evaluate()?.get(), right.evaluate()?.get())?
            }
        };
        ExactInteger::new(value)
    }

    fn write_javascript(&self, output: &mut String) -> std::fmt::Result {
        match self {
            Self::Literal(value) => write!(output, "{}", value.get()),
            Self::Binary { op, left, right } => {
                output.push('(');
                left.write_javascript(output)?;
                write!(output, " {} ", op.symbol())?;
                right.write_javascript(output)?;
                output.push(')');
                Ok(())
            }
        }
    }

    fn node_count(&self) -> usize {
        match self {
            Self::Literal(_) => 1,
            Self::Binary { left, right, .. } => 1 + left.node_count() + right.node_count(),
        }
    }

    fn literal_magnitude(&self) -> u64 {
        match self {
            Self::Literal(value) => i64::from(value.get()).unsigned_abs(),
            Self::Binary { left, right, .. } => {
                left.literal_magnitude() + right.literal_magnitude()
            }
        }
    }

    fn reductions(&self) -> Vec<Self> {
        let mut reductions = Vec::new();
        match self {
            Self::Literal(value) => {
                reductions.extend(value.reductions().into_iter().map(Self::Literal));
            }
            Self::Binary { op, left, right } => {
                reductions.push((**left).clone());
                reductions.push((**right).clone());
                for reduced_left in left.reductions() {
                    reductions.push(Self::Binary {
                        op: *op,
                        left: Box::new(reduced_left),
                        right: right.clone(),
                    });
                }
                for reduced_right in right.reductions() {
                    reductions.push(Self::Binary {
                        op: *op,
                        left: left.clone(),
                        right: Box::new(reduced_right),
                    });
                }
            }
        }
        deduplicate_preserving_order(&mut reductions);
        reductions
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ArithmeticCheck {
    expression: ArithmeticExpr,
    expected: ExactInteger,
}

impl ArithmeticCheck {
    fn new(expression: ArithmeticExpr) -> Option<Self> {
        let expected = expression.evaluate()?;
        Some(Self {
            expression,
            expected,
        })
    }

    fn reductions(&self) -> Vec<Self> {
        self.expression
            .reductions()
            .into_iter()
            .filter_map(Self::new)
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NonEmptyChecks {
    first: ArithmeticCheck,
    rest: Vec<ArithmeticCheck>,
}

impl NonEmptyChecks {
    fn from_vec(checks: Vec<ArithmeticCheck>) -> Option<Self> {
        let mut checks = checks.into_iter();
        Some(Self {
            first: checks.next()?,
            rest: checks.collect(),
        })
    }

    fn to_vec(&self) -> Vec<ArithmeticCheck> {
        let mut checks = Vec::with_capacity(self.len());
        checks.push(self.first.clone());
        checks.extend(self.rest.iter().cloned());
        checks
    }

    fn len(&self) -> usize {
        1 + self.rest.len()
    }

    fn iter(&self) -> impl Iterator<Item = &ArithmeticCheck> {
        std::iter::once(&self.first).chain(self.rest.iter())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct ProgramComplexity {
    check_count: usize,
    node_count: usize,
    literal_magnitude: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GeneratedArithmeticProgram {
    checks: NonEmptyChecks,
}

impl GeneratedArithmeticProgram {
    fn new(checks: Vec<ArithmeticCheck>) -> Option<Self> {
        Some(Self {
            checks: NonEmptyChecks::from_vec(checks)?,
        })
    }

    fn complexity(&self) -> ProgramComplexity {
        ProgramComplexity {
            check_count: self.checks.len(),
            node_count: self
                .checks
                .iter()
                .map(|check| check.expression.node_count())
                .sum(),
            literal_magnitude: self
                .checks
                .iter()
                .map(|check| check.expression.literal_magnitude())
                .sum(),
        }
    }

    fn source(&self) -> Result<String, DifferentialError> {
        let mut source = String::new();
        for (index, check) in self.checks.iter().enumerate() {
            let mut expression = String::new();
            check
                .expression
                .write_javascript(&mut expression)
                .map_err(|_| {
                    DifferentialError::GeneratorInvariant(
                        "failed to render generated arithmetic expression".to_string(),
                    )
                })?;
            writeln!(source, "if ({expression} !== {}) {{", check.expected.get()).map_err(
                |_| {
                    DifferentialError::GeneratorInvariant(
                        "failed to render generated arithmetic check".to_string(),
                    )
                },
            )?;
            writeln!(
                source,
                "  throw \"lila differential arithmetic check {index:02}\";"
            )
            .map_err(|_| {
                DifferentialError::GeneratorInvariant(
                    "failed to render generated arithmetic check".to_string(),
                )
            })?;
            source.push_str("}\n");
        }
        Ok(source)
    }

    fn to_case(
        &self,
        plan: ArithmeticGenerationPlan,
    ) -> Result<DifferentialCase, DifferentialError> {
        let stem = format!(
            "seed-{:016x}-checks-{:02}-depth-{}",
            plan.seed().get(),
            plan.checks().get(),
            plan.depth().get(),
        );
        DifferentialCase::new(
            format!("t25/generated/{GENERATOR_VERSION}/{stem}"),
            DifferentialGoal::Script,
            ObservationContract::SelfCheckingNoOutput,
            format!("differential/v1/generated/{GENERATOR_VERSION}/{stem}.js"),
            GENERATED_CASE_TIMEOUT_MS,
            self.source()?,
        )
    }

    fn reduction_candidates(&self) -> Vec<ReductionCandidate> {
        let current_complexity = self.complexity();
        let checks = self.checks.to_vec();
        let mut candidates = Vec::new();

        if checks.len() > 1 {
            for width in (1..checks.len()).rev() {
                for start in 0..=checks.len() - width {
                    let mut reduced = checks.clone();
                    reduced.drain(start..start + width);
                    if let Some(program) = Self::new(reduced) {
                        push_reduction_candidate(&mut candidates, current_complexity, program);
                    }
                }
            }
        }

        for (check_index, check) in checks.iter().enumerate() {
            for reduced_check in check.reductions() {
                let mut reduced = checks.clone();
                reduced[check_index] = reduced_check;
                if let Some(program) = Self::new(reduced) {
                    push_reduction_candidate(&mut candidates, current_complexity, program);
                }
            }
        }

        candidates
    }
}

#[derive(Debug)]
struct ReductionCandidate {
    program: GeneratedArithmeticProgram,
}

fn push_reduction_candidate(
    candidates: &mut Vec<ReductionCandidate>,
    current_complexity: ProgramComplexity,
    program: GeneratedArithmeticProgram,
) {
    if program.complexity() >= current_complexity
        || candidates
            .iter()
            .any(|candidate| candidate.program == program)
    {
        return;
    }
    candidates.push(ReductionCandidate { program });
}

fn deduplicate_preserving_order<T: PartialEq>(values: &mut Vec<T>) {
    let mut index = 0;
    while index < values.len() {
        if values[..index].contains(&values[index]) {
            values.remove(index);
        } else {
            index += 1;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ReductionWitness(ReductionWitnessKind);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReductionWitnessKind {
    WasmAotErrored(FailurePhase),
    SpecExecErrored(FailurePhase),
}

impl ReductionWitness {
    fn from_report(report: &DifferentialReport) -> Option<Self> {
        if report.verdict() != DifferentialVerdict::Mismatch {
            return None;
        }
        match (&report.wasm_aot().execution, &report.spec_exec().execution) {
            (ExecutionObservation::Error { phase, .. }, ExecutionObservation::Normal { .. }) => {
                Some(Self(ReductionWitnessKind::WasmAotErrored(*phase)))
            }
            (ExecutionObservation::Normal { .. }, ExecutionObservation::Error { phase, .. }) => {
                Some(Self(ReductionWitnessKind::SpecExecErrored(*phase)))
            }
            (ExecutionObservation::Normal { .. }, ExecutionObservation::Normal { .. })
            | (ExecutionObservation::Error { .. }, ExecutionObservation::Error { .. }) => None,
        }
    }
}

struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    const fn new(seed: ArithmeticGenerationSeed) -> Self {
        Self { state: seed.get() }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }
}

fn generate_program(
    plan: ArithmeticGenerationPlan,
) -> Result<GeneratedArithmeticProgram, DifferentialError> {
    let mut random = SplitMix64::new(plan.seed());
    let mut checks = Vec::with_capacity(usize::from(plan.checks().get()));
    for _ in 0..plan.checks().get() {
        let expression = generate_expression(&mut random, plan.depth().get());
        checks.push(ArithmeticCheck::new(expression).ok_or_else(|| {
            DifferentialError::GeneratorInvariant(
                "generated arithmetic expression escaped the exact safe-integer domain".to_string(),
            )
        })?);
    }
    GeneratedArithmeticProgram::new(checks).ok_or_else(|| {
        DifferentialError::GeneratorInvariant(
            "non-zero arithmetic check count produced an empty program".to_string(),
        )
    })
}

fn generate_expression(random: &mut SplitMix64, depth: u8) -> ArithmeticExpr {
    if depth == 0 {
        return ArithmeticExpr::Literal(SmallInteger::from_random_word(random.next_u64()));
    }
    let op = if random.next_u64() & 1 == 0 {
        ArithmeticOp::Add
    } else {
        ArithmeticOp::Subtract
    };
    ArithmeticExpr::Binary {
        op,
        left: Box::new(generate_expression(random, depth - 1)),
        right: Box::new(generate_expression(random, depth - 1)),
    }
}

fn reduce_with<Observation, Error>(
    mut program: GeneratedArithmeticProgram,
    mut observation: Observation,
    target: ReductionWitness,
    limit: ArithmeticReductionLimit,
    mut observe: impl FnMut(
        &GeneratedArithmeticProgram,
    ) -> Result<(Option<ReductionWitness>, Observation), Error>,
) -> Result<
    (
        GeneratedArithmeticProgram,
        Observation,
        ArithmeticReductionSummary,
    ),
    Error,
> {
    let mut attempted_replays = 0;
    let mut accepted_reductions = 0;

    loop {
        let candidates = program.reduction_candidates();
        let mut accepted = false;
        for candidate in candidates {
            if attempted_replays == limit.get() {
                return Ok((
                    program,
                    observation,
                    ArithmeticReductionSummary {
                        attempted_replays,
                        accepted_reductions,
                        stop: ArithmeticReductionStop::ReplayLimitReached,
                    },
                ));
            }
            attempted_replays += 1;
            let (witness, candidate_observation) = observe(&candidate.program)?;
            if witness == Some(target) {
                program = candidate.program;
                observation = candidate_observation;
                accepted_reductions += 1;
                accepted = true;
                break;
            }
        }

        if !accepted {
            return Ok((
                program,
                observation,
                ArithmeticReductionSummary {
                    attempted_replays,
                    accepted_reductions,
                    stop: ArithmeticReductionStop::FixedPoint,
                },
            ));
        }
    }
}

/// Generate one deterministic schema-v1 case, replay it through Wasm-AOT and
/// the explicitly selected spec-exec oracle, and reduce a disposition mismatch
/// within the supplied replay budget.
///
/// The capability token is required even in builds where the oracle cargo
/// feature is absent. In those builds the first replay returns
/// `DifferentialError::OracleNotLinked` and no backend executes.
pub fn run_generated_arithmetic_campaign(
    plan: ArithmeticGenerationPlan,
    reduction_limit: ArithmeticReductionLimit,
    oracle: SpecExecOracle,
) -> Result<GeneratedArithmeticCampaignOutcome, DifferentialError> {
    let program = generate_program(plan)?;
    let case = program.to_case(plan)?;
    let report = replay_case(&case, oracle)?;

    match report.verdict() {
        DifferentialVerdict::BothCompleted => {
            Ok(GeneratedArithmeticCampaignOutcome::Verified { case, report })
        }
        DifferentialVerdict::Mismatch => {
            let witness = ReductionWitness::from_report(&report).ok_or_else(|| {
                DifferentialError::GeneratorInvariant(
                    "mismatch verdict did not contain one normal and one error observation"
                        .to_string(),
                )
            })?;
            let (program, report, reduction) =
                reduce_with(program, report, witness, reduction_limit, |candidate| {
                    let candidate_case = candidate.to_case(plan)?;
                    let candidate_report = replay_case(&candidate_case, oracle)?;
                    Ok::<_, DifferentialError>((
                        ReductionWitness::from_report(&candidate_report),
                        candidate_report,
                    ))
                })?;
            Ok(GeneratedArithmeticCampaignOutcome::ReducedMismatch {
                case: program.to_case(plan)?,
                report,
                reduction,
            })
        }
        DifferentialVerdict::BothFailed | DifferentialVerdict::ObservationContractViolated => {
            Ok(GeneratedArithmeticCampaignOutcome::Rejected { case, report })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GENERATED_CASE: &str =
        include_str!("../../tests/differential/v1/t25-generated-integer-arithmetic-v1-seed-1.json");

    fn fixture_plan() -> ArithmeticGenerationPlan {
        ArithmeticGenerationPlan::new(
            ArithmeticGenerationSeed::new(1),
            ArithmeticCheckCount::new(4).expect("fixture check count should be valid"),
            ArithmeticExpressionDepth::new(2).expect("fixture depth should be valid"),
        )
    }

    fn literal(value: i16) -> ArithmeticExpr {
        ArithmeticExpr::Literal(SmallInteger(value))
    }

    fn add(left: ArithmeticExpr, right: ArithmeticExpr) -> ArithmeticExpr {
        ArithmeticExpr::Binary {
            op: ArithmeticOp::Add,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    #[test]
    fn generated_case_matches_committed_schema_v1_corpus_entry() {
        let plan = fixture_plan();
        let generated = generate_program(plan)
            .and_then(|program| program.to_case(plan))
            .expect("fixture plan should generate a case");

        assert_eq!(
            generated
                .to_pretty_json()
                .expect("generated case should encode"),
            GENERATED_CASE
        );
        assert_eq!(
            super::super::case_fingerprint(&generated).as_str(),
            "fnv1a64:b5a5446001a77052"
        );
        lila_front::parse(generated.source(), lila_front::ParseOptions::script())
            .expect("the closed arithmetic grammar should emit a valid Script");
    }

    #[test]
    fn every_reduction_candidate_is_strictly_smaller_and_nonempty() {
        let program = generate_program(fixture_plan()).expect("fixture plan should generate");
        let complexity = program.complexity();
        let candidates = program.reduction_candidates();

        assert!(!candidates.is_empty());
        assert!(candidates.iter().all(|candidate| {
            candidate.program.checks.len() > 0 && candidate.program.complexity() < complexity
        }));
    }

    #[test]
    fn reducer_preserves_the_typed_witness_and_reaches_a_fixed_point() {
        let first = ArithmeticCheck::new(add(literal(1), literal(2)))
            .expect("small arithmetic should be exact");
        let second = ArithmeticCheck::new(add(literal(7), literal(3)))
            .expect("small arithmetic should be exact");
        let program =
            GeneratedArithmeticProgram::new(vec![first, second]).expect("two checks are nonempty");
        let target = ReductionWitness(ReductionWitnessKind::WasmAotErrored(
            FailurePhase::WasmRuntimeOrBackend,
        ));
        let limit = ArithmeticReductionLimit::new(64).expect("test limit should be valid");

        let (reduced, (), summary) = reduce_with(program, (), target, limit, |candidate| {
            let source = candidate.source()?;
            Ok::<_, DifferentialError>((source.contains('7').then_some(target), ()))
        })
        .expect("pure reducer predicate should not fail");

        assert_eq!(reduced.checks.len(), 1);
        assert_eq!(reduced.checks.first.expression, literal(7));
        assert_eq!(summary.accepted_reductions(), 2);
        assert_eq!(summary.stop(), ArithmeticReductionStop::FixedPoint);
    }

    #[cfg(feature = "spec-exec-oracle")]
    #[test]
    fn committed_generated_arithmetic_case_replays_through_both_backends() {
        let case = DifferentialCase::from_json(GENERATED_CASE)
            .expect("committed generated case should decode");
        let report = replay_case(&case, SpecExecOracle::explicitly_enabled())
            .expect("both explicitly enabled backends should run");

        assert_eq!(report.verdict(), DifferentialVerdict::BothCompleted);
    }
}
