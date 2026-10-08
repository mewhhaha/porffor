use std::fmt::Write as _;
use std::num::{NonZeroI64, NonZeroU16, NonZeroU8};

use serde::Serialize;

use super::{
    replay_case, DifferentialError, DifferentialProtocol, DifferentialReplayInput,
    DifferentialReport, DifferentialVerdict, DifferentialWorkerRunner, ExecutionObservation,
    FailurePhase, SpecExecOracle,
};

const GENERATED_CASE_TIMEOUT_MS: u64 = 5_000;
const SMALL_INTEGER_MIN: i16 = -32;
const PRODUCT_INTEGER_BOUND: i16 = 8;
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

/// The closed expression-depth domain supported by the integer grammars.
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

/// A versioned grammar is part of both deterministic generation and corpus
/// identity. V1 preserves its original SplitMix64 draw order and source bytes;
/// V2 adds integer bitwise operations and distinguishes positive zero. V3
/// admits bounded products and arithmetic negation with signed-zero results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArithmeticGrammar {
    IntegerArithmeticV1,
    IntegerBitwiseV2,
    IntegerProductV3,
}

impl ArithmeticGrammar {
    pub fn from_name(name: &str) -> Result<Self, DifferentialError> {
        match name {
            "integer-arithmetic-v1" => Ok(Self::IntegerArithmeticV1),
            "integer-bitwise-v2" => Ok(Self::IntegerBitwiseV2),
            "integer-product-v3" => Ok(Self::IntegerProductV3),
            _ => Err(DifferentialError::InvalidGeneration(format!(
                "unknown arithmetic grammar: {name} (expected integer-arithmetic-v1, integer-bitwise-v2, or integer-product-v3)"
            ))),
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::IntegerArithmeticV1 => "integer-arithmetic-v1",
            Self::IntegerBitwiseV2 => "integer-bitwise-v2",
            Self::IntegerProductV3 => "integer-product-v3",
        }
    }

    fn allows(self, expression: &ArithmeticExpr) -> bool {
        match expression {
            ArithmeticExpr::Literal(value) => match self {
                Self::IntegerArithmeticV1 | Self::IntegerBitwiseV2 => true,
                Self::IntegerProductV3 => {
                    (-PRODUCT_INTEGER_BOUND..=PRODUCT_INTEGER_BOUND).contains(&value.get())
                }
            },
            ArithmeticExpr::Unary { op, operand } => {
                let allows_op = match (self, op) {
                    (
                        Self::IntegerArithmeticV1 | Self::IntegerProductV3,
                        ArithmeticUnaryOp::BitwiseNot,
                    ) => false,
                    (Self::IntegerBitwiseV2, ArithmeticUnaryOp::BitwiseNot) => true,
                    (Self::IntegerProductV3, ArithmeticUnaryOp::Negate) => true,
                    (
                        Self::IntegerArithmeticV1 | Self::IntegerBitwiseV2,
                        ArithmeticUnaryOp::Negate,
                    ) => false,
                };
                allows_op && self.allows(operand)
            }
            ArithmeticExpr::Binary { op, left, right } => {
                let allows_op = match (self, op) {
                    (
                        Self::IntegerArithmeticV1 | Self::IntegerBitwiseV2 | Self::IntegerProductV3,
                        ArithmeticOp::Add | ArithmeticOp::Subtract,
                    )
                    | (
                        Self::IntegerBitwiseV2,
                        ArithmeticOp::BitwiseAnd
                        | ArithmeticOp::BitwiseOr
                        | ArithmeticOp::BitwiseXor
                        | ArithmeticOp::LeftShift
                        | ArithmeticOp::SignedRightShift
                        | ArithmeticOp::UnsignedRightShift,
                    ) => true,
                    (Self::IntegerProductV3, ArithmeticOp::Multiply) => true,
                    (
                        Self::IntegerArithmeticV1 | Self::IntegerBitwiseV2,
                        ArithmeticOp::Multiply,
                    ) => false,
                    (
                        Self::IntegerArithmeticV1 | Self::IntegerProductV3,
                        ArithmeticOp::BitwiseAnd
                        | ArithmeticOp::BitwiseOr
                        | ArithmeticOp::BitwiseXor
                        | ArithmeticOp::LeftShift
                        | ArithmeticOp::SignedRightShift
                        | ArithmeticOp::UnsignedRightShift,
                    ) => false,
                };
                allows_op && self.allows(left) && self.allows(right)
            }
        }
    }
}

/// One complete deterministic generation request. Grammar selection is required
/// at construction rather than being inferred from a seed or source program.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArithmeticGenerationPlan {
    grammar: ArithmeticGrammar,
    seed: ArithmeticGenerationSeed,
    checks: ArithmeticCheckCount,
    depth: ArithmeticExpressionDepth,
}

impl ArithmeticGenerationPlan {
    pub const fn new(
        grammar: ArithmeticGrammar,
        seed: ArithmeticGenerationSeed,
        checks: ArithmeticCheckCount,
        depth: ArithmeticExpressionDepth,
    ) -> Self {
        Self {
            grammar,
            seed,
            checks,
            depth,
        }
    }

    pub const fn grammar(self) -> ArithmeticGrammar {
        self.grammar
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
        case: DifferentialReplayInput,
        report: DifferentialReport,
    },
    ReducedMismatch {
        case: DifferentialReplayInput,
        report: DifferentialReport,
        reduction: ArithmeticReductionSummary,
    },
    Rejected {
        case: DifferentialReplayInput,
        report: DifferentialReport,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArithmeticOp {
    Add,
    Subtract,
    Multiply,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    LeftShift,
    SignedRightShift,
    UnsignedRightShift,
}

impl ArithmeticOp {
    const fn symbol(self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Multiply => "*",
            Self::BitwiseAnd => "&",
            Self::BitwiseOr => "|",
            Self::BitwiseXor => "^",
            Self::LeftShift => "<<",
            Self::SignedRightShift => ">>",
            Self::UnsignedRightShift => ">>>",
        }
    }

    fn apply(self, left: ExactInteger, right: ExactInteger) -> Option<ExactInteger> {
        let value = match self {
            Self::Add => left.integer_value().checked_add(right.integer_value())?,
            Self::Subtract => left.integer_value().checked_sub(right.integer_value())?,
            Self::Multiply => left.integer_value().checked_mul(right.integer_value())?,
            Self::BitwiseAnd => i64::from(left.to_int32() & right.to_int32()),
            Self::BitwiseOr => i64::from(left.to_int32() | right.to_int32()),
            Self::BitwiseXor => i64::from(left.to_int32() ^ right.to_int32()),
            Self::LeftShift => i64::from(left.to_int32().wrapping_shl(right.shift_count())),
            Self::SignedRightShift => i64::from(left.to_int32() >> right.shift_count()),
            Self::UnsignedRightShift => i64::from(left.to_uint32() >> right.shift_count()),
        };
        let admitted = ExactInteger::new(value)?;
        match admitted {
            ExactInteger::NonZero(_) => Some(admitted),
            ExactInteger::PositiveZero | ExactInteger::NegativeZero => {
                let zero = match self {
                    Self::Add => match (left.sign(), right.sign()) {
                        (NumberSign::Negative, NumberSign::Negative) => ExactInteger::NegativeZero,
                        (NumberSign::Positive, NumberSign::Positive)
                        | (NumberSign::Positive, NumberSign::Negative)
                        | (NumberSign::Negative, NumberSign::Positive) => {
                            ExactInteger::PositiveZero
                        }
                    },
                    Self::Subtract => match (left.sign(), right.sign()) {
                        (NumberSign::Negative, NumberSign::Positive) => ExactInteger::NegativeZero,
                        (NumberSign::Positive, NumberSign::Positive)
                        | (NumberSign::Positive, NumberSign::Negative)
                        | (NumberSign::Negative, NumberSign::Negative) => {
                            ExactInteger::PositiveZero
                        }
                    },
                    Self::Multiply => match (left.sign(), right.sign()) {
                        (NumberSign::Positive, NumberSign::Negative)
                        | (NumberSign::Negative, NumberSign::Positive) => {
                            ExactInteger::NegativeZero
                        }
                        (NumberSign::Positive, NumberSign::Positive)
                        | (NumberSign::Negative, NumberSign::Negative) => {
                            ExactInteger::PositiveZero
                        }
                    },
                    Self::BitwiseAnd
                    | Self::BitwiseOr
                    | Self::BitwiseXor
                    | Self::LeftShift
                    | Self::SignedRightShift
                    | Self::UnsignedRightShift => ExactInteger::PositiveZero,
                };
                Some(zero)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArithmeticUnaryOp {
    BitwiseNot,
    Negate,
}

impl ArithmeticUnaryOp {
    const fn symbol(self) -> &'static str {
        match self {
            Self::BitwiseNot => "~",
            Self::Negate => "-",
        }
    }

    fn apply(self, operand: ExactInteger) -> Option<ExactInteger> {
        match self {
            Self::BitwiseNot => ExactInteger::new(i64::from(!operand.to_int32())),
            Self::Negate => match operand {
                ExactInteger::PositiveZero => Some(ExactInteger::NegativeZero),
                ExactInteger::NegativeZero => Some(ExactInteger::PositiveZero),
                ExactInteger::NonZero(value) => ExactInteger::new(value.get().checked_neg()?),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SmallInteger(i16);

impl SmallInteger {
    fn from_random_word(word: u64, grammar: ArithmeticGrammar) -> Self {
        match grammar {
            ArithmeticGrammar::IntegerArithmeticV1 | ArithmeticGrammar::IntegerBitwiseV2 => {
                let offset = (word % 65) as i32;
                Self((i32::from(SMALL_INTEGER_MIN) + offset) as i16)
            }
            ArithmeticGrammar::IntegerProductV3 => {
                let width = (2 * PRODUCT_INTEGER_BOUND + 1) as u64;
                Self((word % width) as i16 - PRODUCT_INTEGER_BOUND)
            }
        }
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
struct ExactNonZeroInteger(NonZeroI64);

impl ExactNonZeroInteger {
    fn new(value: i64) -> Option<Self> {
        if !(-MAX_SAFE_INTEGER..=MAX_SAFE_INTEGER).contains(&value) {
            return None;
        }
        NonZeroI64::new(value).map(Self)
    }

    const fn get(self) -> i64 {
        self.0.get()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NumberSign {
    Positive,
    Negative,
}

/// Exact finite integral Number results, including both ECMAScript zero signs.
/// Nonzero values have one range-checked constructor; zero signs are closed
/// variants rather than an independently mutable flag beside an integer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExactInteger {
    PositiveZero,
    NegativeZero,
    NonZero(ExactNonZeroInteger),
}

impl ExactInteger {
    fn new(value: i64) -> Option<Self> {
        if value == 0 {
            Some(Self::PositiveZero)
        } else {
            ExactNonZeroInteger::new(value).map(Self::NonZero)
        }
    }

    // Mathematical integer extraction is valid for safe-integer operations
    // and ToInt32/ToUint32, but deliberately is not the source renderer.
    const fn integer_value(self) -> i64 {
        match self {
            Self::PositiveZero | Self::NegativeZero => 0,
            Self::NonZero(value) => value.get(),
        }
    }

    fn sign(self) -> NumberSign {
        match self {
            Self::PositiveZero => NumberSign::Positive,
            Self::NegativeZero => NumberSign::Negative,
            Self::NonZero(value) => {
                if value.get() < 0 {
                    NumberSign::Negative
                } else {
                    NumberSign::Positive
                }
            }
        }
    }

    fn write_javascript(self, output: &mut String) -> std::fmt::Result {
        match self {
            Self::PositiveZero => output.write_str("0"),
            Self::NegativeZero => output.write_str("-0"),
            Self::NonZero(value) => write!(output, "{}", value.get()),
        }
    }

    // This domain contains finite integral binary64 values and signed zeros,
    // so ToUint32 needs only mathematical modulo, not float saturation or a
    // NaN/fraction special case. A negative value must wrap upward modulo 2^32.
    fn to_uint32(self) -> u32 {
        self.integer_value().rem_euclid(1_i64 << 32) as u32
    }

    fn to_int32(self) -> i32 {
        // The Rust narrowing cast interprets the retained low 32 bits as a
        // signed two's-complement value, matching ECMAScript ToInt32.
        self.to_uint32() as i32
    }

    fn shift_count(self) -> u32 {
        self.to_uint32() & 31
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ArithmeticExpr {
    Literal(SmallInteger),
    Unary {
        op: ArithmeticUnaryOp,
        operand: Box<Self>,
    },
    Binary {
        op: ArithmeticOp,
        left: Box<Self>,
        right: Box<Self>,
    },
}

impl ArithmeticExpr {
    fn evaluate(&self) -> Option<ExactInteger> {
        match self {
            Self::Literal(value) => ExactInteger::new(i64::from(value.get())),
            Self::Unary { op, operand } => op.apply(operand.evaluate()?),
            Self::Binary { op, left, right } => op.apply(left.evaluate()?, right.evaluate()?),
        }
    }

    fn write_javascript(&self, output: &mut String) -> std::fmt::Result {
        match self {
            Self::Literal(value) => write!(output, "{}", value.get()),
            Self::Unary { op, operand } => {
                output.push('(');
                output.push_str(op.symbol());
                match op {
                    ArithmeticUnaryOp::BitwiseNot => operand.write_javascript(output)?,
                    ArithmeticUnaryOp::Negate => {
                        // A negative literal must not render as a decrement
                        // token, and nested negation must retain its arity.
                        output.push('(');
                        operand.write_javascript(output)?;
                        output.push(')');
                    }
                }
                output.push(')');
                Ok(())
            }
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
            Self::Unary { operand, .. } => 1 + operand.node_count(),
            Self::Binary { left, right, .. } => 1 + left.node_count() + right.node_count(),
        }
    }

    fn literal_magnitude(&self) -> u64 {
        match self {
            Self::Literal(value) => i64::from(value.get()).unsigned_abs(),
            Self::Unary { operand, .. } => operand.literal_magnitude(),
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
            Self::Unary { op, operand } => {
                reductions.push((**operand).clone());
                for reduced_operand in operand.reductions() {
                    reductions.push(Self::Unary {
                        op: *op,
                        operand: Box::new(reduced_operand),
                    });
                }
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
    grammar: ArithmeticGrammar,
    checks: NonEmptyChecks,
}

impl GeneratedArithmeticProgram {
    fn new(grammar: ArithmeticGrammar, checks: Vec<ArithmeticCheck>) -> Option<Self> {
        if checks
            .iter()
            .any(|check| !grammar.allows(&check.expression))
        {
            return None;
        }
        Some(Self {
            grammar,
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
            let mut expected = String::new();
            check
                .expected
                .write_javascript(&mut expected)
                .map_err(|_| {
                    DifferentialError::GeneratorInvariant(
                        "failed to render generated arithmetic expected value".to_string(),
                    )
                })?;
            match self.grammar {
                ArithmeticGrammar::IntegerArithmeticV1 => {
                    writeln!(source, "if ({expression} !== {expected}) {{")
                }
                ArithmeticGrammar::IntegerBitwiseV2 | ArithmeticGrammar::IntegerProductV3 => {
                    writeln!(source, "if (!Object.is({expression}, {expected})) {{")
                }
            }
            .map_err(|_| {
                DifferentialError::GeneratorInvariant(
                    "failed to render generated arithmetic check".to_string(),
                )
            })?;
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
    ) -> Result<DifferentialReplayInput, DifferentialError> {
        if plan.grammar() != self.grammar {
            return Err(DifferentialError::GeneratorInvariant(
                "generated program and corpus plan have different grammars".to_string(),
            ));
        }
        let version = self.grammar.name();
        let stem = format!(
            "seed-{:016x}-checks-{:02}-depth-{}",
            plan.seed().get(),
            plan.checks().get(),
            plan.depth().get(),
        );
        DifferentialReplayInput::new_script(
            format!("t25/generated/{version}/{stem}"),
            DifferentialProtocol::V1SelfCheckingNoOutput,
            format!("differential/v1/generated/{version}/{stem}.js"),
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
                    if let Some(program) = Self::new(self.grammar, reduced) {
                        push_reduction_candidate(&mut candidates, current_complexity, program);
                    }
                }
            }
        }

        for (check_index, check) in checks.iter().enumerate() {
            for reduced_check in check.reductions() {
                let mut reduced = checks.clone();
                reduced[check_index] = reduced_check;
                if let Some(program) = Self::new(self.grammar, reduced) {
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
            (
                ExecutionObservation::PrimitiveCompletion { .. }
                | ExecutionObservation::UnsupportedCompletion { .. }
                | ExecutionObservation::EngineFailure { .. }
                | ExecutionObservation::WorkerFailure { .. }
                | ExecutionObservation::SelectedObjectProbe { .. }
                | ExecutionObservation::ObservationRejected { .. }
                | ExecutionObservation::RootedCompletionGraph { .. },
                _,
            )
            | (
                _,
                ExecutionObservation::PrimitiveCompletion { .. }
                | ExecutionObservation::UnsupportedCompletion { .. }
                | ExecutionObservation::EngineFailure { .. }
                | ExecutionObservation::WorkerFailure { .. }
                | ExecutionObservation::SelectedObjectProbe { .. }
                | ExecutionObservation::ObservationRejected { .. }
                | ExecutionObservation::RootedCompletionGraph { .. },
            ) => None,
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
        let expression = generate_expression(&mut random, plan.depth().get(), plan.grammar());
        checks.push(ArithmeticCheck::new(expression).ok_or_else(|| {
            DifferentialError::GeneratorInvariant(
                "generated arithmetic expression escaped the exact safe-integer domain".to_string(),
            )
        })?);
    }
    GeneratedArithmeticProgram::new(plan.grammar(), checks).ok_or_else(|| {
        DifferentialError::GeneratorInvariant(
            "non-zero arithmetic check count produced an empty program".to_string(),
        )
    })
}

#[derive(Clone, Copy)]
enum GeneratedOperation {
    Binary(ArithmeticOp),
    Unary(ArithmeticUnaryOp),
}

fn generate_expression(
    random: &mut SplitMix64,
    depth: u8,
    grammar: ArithmeticGrammar,
) -> ArithmeticExpr {
    if depth == 0 {
        return ArithmeticExpr::Literal(SmallInteger::from_random_word(random.next_u64(), grammar));
    }
    let operation = match grammar {
        ArithmeticGrammar::IntegerArithmeticV1 => {
            // Keep V1's low-bit choice and one draw per binary node exactly.
            let op = if random.next_u64() & 1 == 0 {
                ArithmeticOp::Add
            } else {
                ArithmeticOp::Subtract
            };
            GeneratedOperation::Binary(op)
        }
        ArithmeticGrammar::IntegerBitwiseV2 => {
            const OPERATIONS: [GeneratedOperation; 9] = [
                GeneratedOperation::Binary(ArithmeticOp::Add),
                GeneratedOperation::Binary(ArithmeticOp::Subtract),
                GeneratedOperation::Binary(ArithmeticOp::BitwiseAnd),
                GeneratedOperation::Binary(ArithmeticOp::BitwiseOr),
                GeneratedOperation::Binary(ArithmeticOp::BitwiseXor),
                GeneratedOperation::Binary(ArithmeticOp::LeftShift),
                GeneratedOperation::Binary(ArithmeticOp::SignedRightShift),
                GeneratedOperation::Binary(ArithmeticOp::UnsignedRightShift),
                GeneratedOperation::Unary(ArithmeticUnaryOp::BitwiseNot),
            ];
            OPERATIONS[(random.next_u64() % OPERATIONS.len() as u64) as usize]
        }
        ArithmeticGrammar::IntegerProductV3 => {
            // At depth four, the largest magnitude is 8^(2^4) = 2^48.
            // Add/Sub/Negate cannot exceed the all-product bound, so every
            // admitted plan stays exact without retries or fallback nodes.
            const OPERATIONS: [GeneratedOperation; 4] = [
                GeneratedOperation::Binary(ArithmeticOp::Add),
                GeneratedOperation::Binary(ArithmeticOp::Subtract),
                GeneratedOperation::Binary(ArithmeticOp::Multiply),
                GeneratedOperation::Unary(ArithmeticUnaryOp::Negate),
            ];
            OPERATIONS[(random.next_u64() % OPERATIONS.len() as u64) as usize]
        }
    };
    match operation {
        GeneratedOperation::Binary(op) => ArithmeticExpr::Binary {
            op,
            left: Box::new(generate_expression(random, depth - 1, grammar)),
            right: Box::new(generate_expression(random, depth - 1, grammar)),
        },
        GeneratedOperation::Unary(op) => ArithmeticExpr::Unary {
            op,
            operand: Box::new(generate_expression(random, depth - 1, grammar)),
        },
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

enum ArithmeticCampaignReductionError {
    Replay(DifferentialError),
    WorkerFailure {
        case: DifferentialReplayInput,
        report: DifferentialReport,
    },
}

/// Generate one deterministic schema-v1 case, replay it through Wasm-AOT and
/// the explicitly selected spec-exec oracle, and reduce a disposition mismatch
/// within the supplied replay budget.
///
/// The capability token is required even in builds where the oracle cargo
/// feature is absent. In those builds the first replay returns
/// `DifferentialError::OracleNotLinked` and no backend executes.
/// The worker runner is mandatory for initial and reduction replays. Native
/// candidate construction does not parse JavaScript in the campaign process.
/// Worker failures never provide a reduction witness or persistable case.
pub fn run_generated_arithmetic_campaign(
    plan: ArithmeticGenerationPlan,
    reduction_limit: ArithmeticReductionLimit,
    oracle: SpecExecOracle,
    runner: &DifferentialWorkerRunner,
) -> Result<GeneratedArithmeticCampaignOutcome, DifferentialError> {
    run_with_replay(plan, reduction_limit, |case| {
        replay_case(case, oracle, runner)
    })
}

// Both product callers supply the same selected-worker replay. The campaign
// caller additionally journals each exact candidate before and after replay.
pub(super) fn run_with_replay(
    plan: ArithmeticGenerationPlan,
    reduction_limit: ArithmeticReductionLimit,
    mut replay: impl FnMut(&DifferentialReplayInput) -> Result<DifferentialReport, DifferentialError>,
) -> Result<GeneratedArithmeticCampaignOutcome, DifferentialError> {
    let program = generate_program(plan)?;
    let case = program.to_case(plan)?;
    let report = replay(&case)?;

    match report.verdict() {
        DifferentialVerdict::BothCompleted => {
            Ok(GeneratedArithmeticCampaignOutcome::Verified { case, report })
        }
        DifferentialVerdict::PrimitiveCompletionsMatch => {
            Err(DifferentialError::GeneratorInvariant(
                "schema-v1 arithmetic replay returned a schema-v2 primitive verdict".to_string(),
            ))
        }
        DifferentialVerdict::PrimitiveCompletionAndPrintTranscriptMatch => {
            Err(DifferentialError::GeneratorInvariant(
                "schema-v1 arithmetic replay returned a schema-v3 primitive-and-print verdict"
                    .to_string(),
            ))
        }
        DifferentialVerdict::SelectedObjectProbeAndPrintTranscriptMatch => {
            Err(DifferentialError::GeneratorInvariant(
                "schema-v1 arithmetic replay returned a schema-v5 selected-object-probe verdict"
                    .into(),
            ))
        }
        DifferentialVerdict::RootedCompletionGraphAndPrintTranscriptMatch => {
            Err(DifferentialError::GeneratorInvariant(
                "schema-v1 arithmetic replay returned a schema-v7 rooted-completion-graph verdict"
                    .into(),
            ))
        }
        DifferentialVerdict::Mismatch => {
            let witness = ReductionWitness::from_report(&report).ok_or_else(|| {
                DifferentialError::GeneratorInvariant(
                    "mismatch verdict did not contain one normal and one error observation"
                        .to_string(),
                )
            })?;
            let reduced = reduce_with(program, report, witness, reduction_limit, |candidate| {
                let candidate_case = candidate
                    .to_case(plan)
                    .map_err(ArithmeticCampaignReductionError::Replay)?;
                let candidate_report =
                    replay(&candidate_case).map_err(ArithmeticCampaignReductionError::Replay)?;
                if candidate_report.verdict() == DifferentialVerdict::WorkerFailure {
                    return Err(ArithmeticCampaignReductionError::WorkerFailure {
                        case: candidate_case,
                        report: candidate_report,
                    });
                }
                Ok((
                    ReductionWitness::from_report(&candidate_report),
                    candidate_report,
                ))
            });
            match reduced {
                Ok((program, report, reduction)) => {
                    Ok(GeneratedArithmeticCampaignOutcome::ReducedMismatch {
                        case: program.to_case(plan)?,
                        report,
                        reduction,
                    })
                }
                Err(ArithmeticCampaignReductionError::Replay(error)) => Err(error),
                Err(ArithmeticCampaignReductionError::WorkerFailure { case, report }) => {
                    Ok(GeneratedArithmeticCampaignOutcome::Rejected { case, report })
                }
            }
        }
        DifferentialVerdict::BothFailed
        | DifferentialVerdict::ObservationContractViolated
        | DifferentialVerdict::WorkerFailure => {
            Ok(GeneratedArithmeticCampaignOutcome::Rejected { case, report })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GENERATED_CASE: &str =
        include_str!("../../tests/differential/v1/t25-generated-integer-arithmetic-v1-seed-1.json");
    const BITWISE_PROBE: &str =
        include_str!("../../tests/differential/v1/t25-integer-bitwise-v2-conversions.js");
    const PRODUCT_PROBE: &str =
        include_str!("../../tests/differential/v1/t25-integer-product-v3-signed-zero.js");

    fn fixture_plan() -> ArithmeticGenerationPlan {
        ArithmeticGenerationPlan::new(
            ArithmeticGrammar::IntegerArithmeticV1,
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

    fn binary(op: ArithmeticOp, left: ArithmeticExpr, right: ArithmeticExpr) -> ArithmeticExpr {
        ArithmeticExpr::Binary {
            op,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    fn bitwise_not(operand: ArithmeticExpr) -> ArithmeticExpr {
        ArithmeticExpr::Unary {
            op: ArithmeticUnaryOp::BitwiseNot,
            operand: Box::new(operand),
        }
    }

    fn negate(operand: ArithmeticExpr) -> ArithmeticExpr {
        ArithmeticExpr::Unary {
            op: ArithmeticUnaryOp::Negate,
            operand: Box::new(operand),
        }
    }

    fn exact(value: i64) -> ExactInteger {
        ExactInteger::new(value).expect("regression operand is an exact safe integer")
    }

    #[test]
    fn exact_results_own_both_zero_signs_and_reject_unbounded_nonzero_values() {
        assert_eq!(exact(0), ExactInteger::PositiveZero);
        assert_ne!(ExactInteger::PositiveZero, ExactInteger::NegativeZero);
        assert!(ExactNonZeroInteger::new(0).is_none());
        assert!(ExactInteger::new(MAX_SAFE_INTEGER + 1).is_none());
        assert!(ExactInteger::new(-MAX_SAFE_INTEGER - 1).is_none());
        for zero in [ExactInteger::PositiveZero, ExactInteger::NegativeZero] {
            assert_eq!(zero.to_uint32(), 0);
            assert_eq!(zero.to_int32(), 0);
            assert_eq!(
                ArithmeticOp::BitwiseAnd.apply(zero, exact(-1)),
                Some(ExactInteger::PositiveZero),
            );
        }
        assert!(ArithmeticOp::Multiply
            .apply(exact(MAX_SAFE_INTEGER), exact(2))
            .is_none());
        assert!(ArithmeticOp::Multiply
            .apply(exact(MAX_SAFE_INTEGER), exact(MAX_SAFE_INTEGER))
            .is_none());
    }

    #[test]
    fn arithmetic_zero_signs_survive_negation_addition_subtraction_and_products() {
        use ExactInteger::{NegativeZero as negative, PositiveZero as positive};
        for (op, left, right, expected) in [
            (ArithmeticOp::Add, positive, positive, positive),
            (ArithmeticOp::Add, positive, negative, positive),
            (ArithmeticOp::Add, negative, positive, positive),
            (ArithmeticOp::Add, negative, negative, negative),
            (ArithmeticOp::Subtract, positive, positive, positive),
            (ArithmeticOp::Subtract, positive, negative, positive),
            (ArithmeticOp::Subtract, negative, positive, negative),
            (ArithmeticOp::Subtract, negative, negative, positive),
            (ArithmeticOp::Multiply, positive, positive, positive),
            (ArithmeticOp::Multiply, positive, negative, negative),
            (ArithmeticOp::Multiply, negative, positive, negative),
            (ArithmeticOp::Multiply, negative, negative, positive),
            (ArithmeticOp::Multiply, positive, exact(-8), negative),
            (ArithmeticOp::Multiply, exact(-8), positive, negative),
            (ArithmeticOp::Multiply, negative, exact(-8), positive),
            (ArithmeticOp::Multiply, exact(-8), negative, positive),
            (ArithmeticOp::Add, exact(-8), exact(8), positive),
            (ArithmeticOp::Subtract, exact(-8), exact(-8), positive),
        ] {
            assert_eq!(
                op.apply(left, right),
                Some(expected),
                "{left:?} {} {right:?}",
                op.symbol()
            );
        }
        for (operand, expected) in [
            (positive, negative),
            (negative, positive),
            (exact(-8), exact(8)),
            (exact(MAX_SAFE_INTEGER), exact(-MAX_SAFE_INTEGER)),
        ] {
            assert_eq!(ArithmeticUnaryOp::Negate.apply(operand), Some(expected));
        }
    }

    #[test]
    fn product_grammar_bounds_and_signed_zero_checks_survive_source_materialization() {
        let product = binary(ArithmeticOp::Multiply, literal(-8), literal(0));
        let check = ArithmeticCheck::new(product).unwrap();
        assert_eq!(check.expected, ExactInteger::NegativeZero);
        for grammar in [
            ArithmeticGrammar::IntegerArithmeticV1,
            ArithmeticGrammar::IntegerBitwiseV2,
        ] {
            assert!(GeneratedArithmeticProgram::new(grammar, vec![check.clone()]).is_none());
            assert!(GeneratedArithmeticProgram::new(
                grammar,
                vec![ArithmeticCheck::new(negate(literal(0))).unwrap()]
            )
            .is_none());
        }
        for expression in [literal(-9), literal(9), bitwise_not(literal(0))] {
            assert!(GeneratedArithmeticProgram::new(
                ArithmeticGrammar::IntegerProductV3,
                vec![ArithmeticCheck::new(expression).unwrap()]
            )
            .is_none());
        }
        let mut maximum = literal(PRODUCT_INTEGER_BOUND);
        for _ in 0..ArithmeticExpressionDepth::Four.get() {
            maximum = binary(ArithmeticOp::Multiply, maximum.clone(), maximum);
        }
        assert_eq!(
            ArithmeticCheck::new(maximum).unwrap().expected,
            exact(1_i64 << 48)
        );
        let checks = vec![
            check,
            ArithmeticCheck::new(negate(literal(-5))).unwrap(),
            ArithmeticCheck::new(negate(literal(0))).unwrap(),
            ArithmeticCheck::new(binary(
                ArithmeticOp::Subtract,
                negate(literal(0)),
                literal(0),
            ))
            .unwrap(),
        ];
        let program =
            GeneratedArithmeticProgram::new(ArithmeticGrammar::IntegerProductV3, checks).unwrap();
        let source = program.source().unwrap();
        assert!(source.contains("Object.is((-8 * 0), -0)"));
        assert!(source.contains("Object.is((-(-5)), 5)"));
        assert!(source.contains("Object.is((-(0)), -0)"));
        assert!(source.contains("Object.is(((-(0)) - 0), -0)"));
        assert!(program.to_case(fixture_plan()).is_err());
        let plan = ArithmeticGenerationPlan::new(
            ArithmeticGrammar::IntegerProductV3,
            fixture_plan().seed(),
            fixture_plan().checks(),
            fixture_plan().depth(),
        );
        let case = program.to_case(plan).unwrap();
        assert!(case
            .id()
            .as_str()
            .starts_with("t25/generated/integer-product-v3/"));
        assert!(case
            .filename()
            .starts_with("differential/v1/generated/integer-product-v3/"));
        assert_eq!(
            DifferentialReplayInput::from_json(&case.to_pretty_json().unwrap()).unwrap(),
            case
        );
        lila_front::parse(case.source(), lila_front::ParseOptions::script())
            .expect("negated negative literals retain unary syntax");
        lila_front::parse(PRODUCT_PROBE, lila_front::ParseOptions::script())
            .expect("the finite hand-authored signed-zero probe is valid Script syntax");
    }

    #[test]
    fn product_reducer_recomputes_zero_sign_and_keeps_the_product_grammar() {
        let program = GeneratedArithmeticProgram::new(
            ArithmeticGrammar::IntegerProductV3,
            vec![
                ArithmeticCheck::new(binary(ArithmeticOp::Multiply, literal(-8), literal(0)))
                    .unwrap(),
                ArithmeticCheck::new(negate(literal(7))).unwrap(),
            ],
        )
        .unwrap();
        let target = ReductionWitness(ReductionWitnessKind::WasmAotErrored(
            FailurePhase::WasmRuntimeOrBackend,
        ));
        let (reduced, (), summary) = reduce_with(
            program,
            (),
            target,
            ArithmeticReductionLimit::new(64).unwrap(),
            |candidate| {
                let source = candidate.source()?;
                Ok::<_, DifferentialError>((
                    (source.contains(" * ") && source.contains(", -0)")).then_some(target),
                    (),
                ))
            },
        )
        .unwrap();
        assert_eq!(reduced.grammar, ArithmeticGrammar::IntegerProductV3);
        assert_eq!(reduced.checks.len(), 1);
        assert_eq!(reduced.checks.first.expected, ExactInteger::NegativeZero);
        assert!(reduced
            .source()
            .unwrap()
            .contains("Object.is((-1 * 0), -0)"));
        assert!(summary.accepted_reductions() >= 2);
        assert_eq!(summary.stop(), ArithmeticReductionStop::FixedPoint);
    }

    #[test]
    fn exact_integer_conversions_wrap_modulo_32_bits_without_saturation() {
        for (value, unsigned, signed) in [
            (0, 0, 0),
            (-1, u32::MAX, -1),
            (2_147_483_648, 2_147_483_648, i32::MIN),
            (4_294_967_295, u32::MAX, -1),
            (4_294_967_296, 0, 0),
            (4_294_967_299, 3, 3),
            (-4_294_967_299, u32::MAX - 2, -3),
            (MAX_SAFE_INTEGER, u32::MAX, -1),
            (-MAX_SAFE_INTEGER, 1, 1),
        ] {
            assert_eq!(exact(value).to_uint32(), unsigned, "ToUint32({value})");
            assert_eq!(exact(value).to_int32(), signed, "ToInt32({value})");
        }
        assert!(ExactInteger::new(MAX_SAFE_INTEGER + 1).is_none());
        assert!(ExactInteger::new(-MAX_SAFE_INTEGER - 1).is_none());
    }

    #[test]
    fn bitwise_operations_convert_each_validated_operand_before_operation() {
        for (op, left, right, expected) in [
            (ArithmeticOp::BitwiseAnd, -4_294_967_295, 5, 1),
            (ArithmeticOp::BitwiseOr, 4_294_967_296, 5, 5),
            (ArithmeticOp::BitwiseXor, MAX_SAFE_INTEGER, -1, 0),
            (ArithmeticOp::LeftShift, 1, 31, -2_147_483_648),
            (ArithmeticOp::LeftShift, 1, 32, 1),
            (ArithmeticOp::LeftShift, 1, -1, -2_147_483_648),
            (ArithmeticOp::LeftShift, 1, 4_294_967_328, 1),
            (ArithmeticOp::SignedRightShift, 2_147_483_648, 31, -1),
            (ArithmeticOp::SignedRightShift, -1, 32, -1),
            (ArithmeticOp::UnsignedRightShift, -1, 1, 2_147_483_647),
            (ArithmeticOp::UnsignedRightShift, -1, 32, 4_294_967_295),
            (
                ArithmeticOp::UnsignedRightShift,
                MAX_SAFE_INTEGER,
                0,
                4_294_967_295,
            ),
        ] {
            assert_eq!(
                op.apply(exact(left), exact(right))
                    .map(ExactInteger::integer_value),
                Some(expected),
                "{left} {} {right}",
                op.symbol(),
            );
        }
        assert_eq!(
            ArithmeticUnaryOp::BitwiseNot.apply(exact(4_294_967_303)),
            Some(exact(-8)),
        );
    }

    #[test]
    fn bitwise_grammar_cannot_be_labelled_as_a_v1_arithmetic_corpus() {
        let check = ArithmeticCheck::new(bitwise_not(literal(-1))).unwrap();
        assert!(GeneratedArithmeticProgram::new(
            ArithmeticGrammar::IntegerArithmeticV1,
            vec![check.clone()],
        )
        .is_none());
        let program =
            GeneratedArithmeticProgram::new(ArithmeticGrammar::IntegerBitwiseV2, vec![check])
                .unwrap();
        assert!(program.to_case(fixture_plan()).is_err());
        let plan = ArithmeticGenerationPlan::new(
            ArithmeticGrammar::IntegerBitwiseV2,
            fixture_plan().seed(),
            fixture_plan().checks(),
            fixture_plan().depth(),
        );
        let case = program.to_case(plan).unwrap();
        assert!(case
            .id()
            .as_str()
            .starts_with("t25/generated/integer-bitwise-v2/"));
        assert!(case
            .filename()
            .starts_with("differential/v1/generated/integer-bitwise-v2/"));
        assert!(case.source().contains("if (!Object.is((~-1), 0))"));
        lila_front::parse(case.source(), lila_front::ParseOptions::script())
            .expect("unary bitwise rendering is valid Script syntax");
        lila_front::parse(BITWISE_PROBE, lila_front::ParseOptions::script())
            .expect("the consumed conversion regression is valid Script syntax");
    }

    #[test]
    fn unary_and_shift_reductions_keep_grammar_and_strictly_smaller_complexity() {
        let expression = binary(
            ArithmeticOp::UnsignedRightShift,
            binary(ArithmeticOp::LeftShift, literal(1), literal(31)),
            literal(32),
        );
        let check = ArithmeticCheck::new(expression).unwrap();
        assert_eq!(check.expected, exact(2_147_483_648));
        let other = ArithmeticCheck::new(bitwise_not(binary(
            ArithmeticOp::BitwiseOr,
            literal(7),
            literal(1),
        )))
        .unwrap();
        let program = GeneratedArithmeticProgram::new(
            ArithmeticGrammar::IntegerBitwiseV2,
            vec![check, other],
        )
        .unwrap();
        assert!(program
            .source()
            .unwrap()
            .contains("Object.is(((1 << 31) >>> 32), 2147483648)"));
        let candidates = program.reduction_candidates();
        assert!(!candidates.is_empty());
        assert!(candidates.iter().any(|candidate| candidate
            .program
            .source()
            .unwrap()
            .contains("(~7)")));
        assert!(candidates.iter().all(|candidate| {
            candidate.program.grammar == ArithmeticGrammar::IntegerBitwiseV2
                && candidate.program.checks.len() > 0
                && candidate.program.complexity() < program.complexity()
        }));
        let target = ReductionWitness(ReductionWitnessKind::WasmAotErrored(
            FailurePhase::WasmRuntimeOrBackend,
        ));
        let (reduced, (), summary) = reduce_with(
            program,
            (),
            target,
            ArithmeticReductionLimit::new(128).unwrap(),
            |candidate| {
                Ok::<_, DifferentialError>((
                    candidate.source()?.contains(">>>").then_some(target),
                    (),
                ))
            },
        )
        .unwrap();
        assert_eq!(reduced.grammar, ArithmeticGrammar::IntegerBitwiseV2);
        assert!(reduced.source().unwrap().contains(">>>"));
        assert!(summary.accepted_reductions() > 0);
        assert_eq!(summary.stop(), ArithmeticReductionStop::FixedPoint);
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
            super::super::input_fingerprint(&generated).as_str(),
            "fnv1a64:b5a5446001a77052"
        );
        lila_front::parse(generated.source(), lila_front::ParseOptions::script())
            .expect("the closed arithmetic grammar should emit a valid Script");
    }

    #[test]
    fn generated_v2_programs_are_source_closed_at_every_supported_depth() {
        for depth in [
            ArithmeticExpressionDepth::One,
            ArithmeticExpressionDepth::Two,
            ArithmeticExpressionDepth::Three,
            ArithmeticExpressionDepth::Four,
        ] {
            let plan = ArithmeticGenerationPlan::new(
                ArithmeticGrammar::IntegerBitwiseV2,
                ArithmeticGenerationSeed::new(u64::MAX),
                ArithmeticCheckCount::new(usize::from(MAX_ARITHMETIC_CHECKS)).unwrap(),
                depth,
            );
            let program = generate_program(plan)
                .expect("bounded V2 generation stays inside the exact integer domain");
            let case = program.to_case(plan).unwrap();
            assert_eq!(program.checks.len(), usize::from(MAX_ARITHMETIC_CHECKS));
            assert!(case
                .id()
                .as_str()
                .starts_with("t25/generated/integer-bitwise-v2/"));
            lila_front::parse(case.source(), lila_front::ParseOptions::script())
                .expect("generated unary and binary nodes form valid Script syntax");
            let encoded = case.to_pretty_json().unwrap();
            assert_eq!(DifferentialReplayInput::from_json(&encoded).unwrap(), case);
        }
    }

    #[test]
    fn generated_product_programs_are_admitted_deterministic_and_source_closed() {
        for depth in [
            ArithmeticExpressionDepth::One,
            ArithmeticExpressionDepth::Two,
            ArithmeticExpressionDepth::Three,
            ArithmeticExpressionDepth::Four,
        ] {
            for seed in [0, 1, u64::MAX] {
                let plan = ArithmeticGenerationPlan::new(
                    ArithmeticGrammar::IntegerProductV3,
                    ArithmeticGenerationSeed::new(seed),
                    ArithmeticCheckCount::new(usize::from(MAX_ARITHMETIC_CHECKS)).unwrap(),
                    depth,
                );
                let program = generate_program(plan)
                    .expect("every bounded V3 plan remains in the exact signed-integer domain");
                let repeated = generate_program(plan)
                    .expect("the same bounded V3 plan remains admissible on regeneration");
                assert_eq!(program, repeated);
                assert_eq!(program.grammar, ArithmeticGrammar::IntegerProductV3);
                assert_eq!(program.checks.len(), usize::from(MAX_ARITHMETIC_CHECKS));
                for check in program.checks.iter() {
                    assert!(plan.grammar().allows(&check.expression));
                    assert_eq!(
                        ArithmeticCheck::new(check.expression.clone()),
                        Some(check.clone()),
                    );
                    assert!(check.expected.integer_value().unsigned_abs() <= (1_u64 << 48));
                }
                let case = program.to_case(plan).unwrap();
                let repeated_case = repeated.to_case(plan).unwrap();
                assert_eq!(case, repeated_case);
                let stem = format!(
                    "seed-{seed:016x}-checks-{:02}-depth-{}",
                    plan.checks().get(),
                    depth.get(),
                );
                assert_eq!(
                    case.id().as_str(),
                    format!("t25/generated/integer-product-v3/{stem}"),
                );
                assert_eq!(
                    case.filename(),
                    format!("differential/v1/generated/integer-product-v3/{stem}.js"),
                );
                assert_eq!(
                    case.source().matches("if (!Object.is(").count(),
                    usize::from(MAX_ARITHMETIC_CHECKS),
                );
                lila_front::parse(case.source(), lila_front::ParseOptions::script())
                    .expect("V3 product and negation rendering forms valid Script syntax");
                let encoded = case.to_pretty_json().unwrap();
                assert_eq!(encoded, repeated_case.to_pretty_json().unwrap());
                assert_eq!(DifferentialReplayInput::from_json(&encoded).unwrap(), case);
            }
        }
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
        let program = GeneratedArithmeticProgram::new(
            ArithmeticGrammar::IntegerArithmeticV1,
            vec![first, second],
        )
        .expect("two checks are nonempty");
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
}
