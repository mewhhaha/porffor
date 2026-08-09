//! Spec-operation vocabulary and the catalog of ECMAScript abstract operations.
//!
//! The catalog is **evidence**, not documentation. A row that claims an
//! operation is implemented can only be produced from something that implements
//! it: either a [`SpecOperationIr`] variant (which the shared emitter arm in
//! `porffor-aot-wasm/src/operations.rs` must handle, or the match there fails to
//! build), or an [`EmissionSite`] naming a statement-shaped emitter arm (which
//! `porffor-aot-wasm/src/emission_sites.rs` joins to a real function path).
//! Everything else is a [`TrackedGapRow`], whose type has no field capable of
//! holding an implementation status.
//!
//! See `docs/rust-rewrite/contracts/Spec-operation catalog evidence and the
//! iterator-protocol obligation witness.md`.

use crate::iterator_obligations::EmissionSite;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingMode {
    Let,
    Const,
    Var,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryNumericOp {
    Plus,
    Minus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArithmeticBinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Exp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitwiseBinaryOp {
    And,
    Or,
    Xor,
    Shl,
    Shr,
    UShr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalBinaryOp {
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EqualityBinaryOp {
    StrictEqual,
    StrictNotEqual,
    LooseEqual,
    LooseNotEqual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ToPrimitiveHint {
    Default,
    Number,
    String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicalBinaryOp {
    And,
    Or,
    Coalesce,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericUpdateOp {
    Increment,
    Decrement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateReturnMode {
    Prefix,
    Postfix,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SpecOperationFamily {
    TypeQuery,
    Conversion,
    Comparison,
    Object,
    Invocation,
    Iterator,
    Completion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EcmaLanguageType {
    Undefined,
    Null,
    Boolean,
    String,
    Symbol,
    Number,
    BigInt,
    Object,
}

impl EcmaLanguageType {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Undefined => "Undefined",
            Self::Null => "Null",
            Self::Boolean => "Boolean",
            Self::String => "String",
            Self::Symbol => "Symbol",
            Self::Number => "Number",
            Self::BigInt => "BigInt",
            Self::Object => "Object",
        }
    }

    pub const fn is_primitive(self) -> bool {
        match self {
            Self::Undefined
            | Self::Null
            | Self::Boolean
            | Self::String
            | Self::Symbol
            | Self::Number
            | Self::BigInt => true,
            Self::Object => false,
        }
    }
}

// ---------------------------------------------------------------------------
// Iterator Records (7.4)
// ---------------------------------------------------------------------------

/// `[[Iterator]]` — the name of the binding holding the iterator object.
///
/// The three slots of an Iterator Record are all binding names, i.e. all
/// `String`. Spelling them as three distinct newtypes is what makes transposing
/// two of them `E0308` instead of a `for await` that compiles and miscompiles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IteratorSlot(String);

/// `[[NextMethod]]` — the name of the binding holding the once-read `next`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NextMethodSlot(String);

/// `[[Done]]` — the name of the suspension slot holding the done flag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoneSlot(String);

impl IteratorSlot {
    pub fn new(binding: String) -> Self {
        Self(binding)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl NextMethodSlot {
    pub fn new(binding: String) -> Self {
        Self(binding)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl DoneSlot {
    pub fn new(binding: String) -> Self {
        Self(binding)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// An Iterator Record (7.4): `{ [[Iterator]], [[NextMethod]], [[Done]] }`.
///
/// `[[Done]]` at IR level is the *name of a suspension slot*, not a
/// compile-time boolean: the compiler never knows whether an iterator is
/// exhausted, only where the emitted code keeps that flag. There is therefore
/// no `is_done`/`mark_done` here, and there is no type parameter — the only
/// thing an IR-level record can hold is binding names.
///
/// There is also no `kind: Sync | Async`. The contract proposed one on the
/// grounds that "`kind` and the plan's existence cannot disagree", but a field
/// set by the single reachable constructor is a constant, and a constant cannot
/// disagree with anything — it is decoration by the same argument the contract
/// uses to delete `OperationLoweringStatus::SharedRustModel`. The sync/async
/// distinction is carried by *which plan owns the record*; reintroduce the
/// enum in the patch that gives the sync for-of path a record of its own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IteratorRecordIr {
    iterator: IteratorSlot,
    next_method: NextMethodSlot,
    done: DoneSlot,
}

impl IteratorRecordIr {
    pub fn new(iterator: IteratorSlot, next_method: NextMethodSlot, done: DoneSlot) -> Self {
        Self {
            iterator,
            next_method,
            done,
        }
    }

    pub fn iterator(&self) -> &IteratorSlot {
        &self.iterator
    }

    pub fn next_method(&self) -> &NextMethodSlot {
        &self.next_method
    }

    pub fn done(&self) -> &DoneSlot {
        &self.done
    }
}

// ---------------------------------------------------------------------------
// Completion Records (6.2.4)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CompletionAbruptKind {
    Throw,
    Return,
    Break,
    Continue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CompletionKindIr {
    Normal,
    Throw,
    Return,
    Break,
    Continue,
    Empty,
}

impl CompletionKindIr {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Throw => "throw",
            Self::Return => "return",
            Self::Break => "break",
            Self::Continue => "continue",
            Self::Empty => "empty",
        }
    }

    pub const fn abi_code(self) -> i64 {
        match self {
            Self::Normal => 0,
            Self::Throw => 1,
            Self::Return => 2,
            Self::Break => 3,
            Self::Continue => 4,
            Self::Empty => 5,
        }
    }

    pub const fn carries_value(self) -> bool {
        match self {
            Self::Normal | Self::Throw | Self::Return | Self::Break | Self::Continue => true,
            Self::Empty => false,
        }
    }

    pub const fn carries_target(self) -> bool {
        match self {
            Self::Break | Self::Continue => true,
            Self::Normal | Self::Throw | Self::Return | Self::Empty => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompletionAbiSlot {
    pub kind: CompletionKindIr,
    pub name: &'static str,
    pub code: i64,
    pub carries_value: bool,
    pub carries_target: bool,
}

pub const COMPLETION_ABI_SLOTS: &[CompletionAbiSlot] = &[
    completion_slot(CompletionKindIr::Normal),
    completion_slot(CompletionKindIr::Throw),
    completion_slot(CompletionKindIr::Return),
    completion_slot(CompletionKindIr::Break),
    completion_slot(CompletionKindIr::Continue),
    completion_slot(CompletionKindIr::Empty),
];

pub const fn completion_slot(kind: CompletionKindIr) -> CompletionAbiSlot {
    CompletionAbiSlot {
        kind,
        name: kind.name(),
        code: kind.abi_code(),
        carries_value: kind.carries_value(),
        carries_target: kind.carries_target(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompletionRecordIr<T> {
    Normal {
        value: Option<T>,
    },
    Throw {
        value: T,
    },
    Return {
        value: T,
    },
    Break {
        value: Option<T>,
        target: Option<String>,
    },
    Continue {
        value: Option<T>,
        target: Option<String>,
    },
    Empty,
}

impl<T> CompletionRecordIr<T> {
    pub const fn normal(value: T) -> Self {
        Self::Normal { value: Some(value) }
    }

    pub const fn empty_normal() -> Self {
        Self::Normal { value: None }
    }

    pub const fn throw(value: T) -> Self {
        Self::Throw { value }
    }

    pub const fn return_(value: T) -> Self {
        Self::Return { value }
    }

    pub fn break_(value: Option<T>, target: Option<String>) -> Self {
        Self::Break { value, target }
    }

    pub fn continue_(value: Option<T>, target: Option<String>) -> Self {
        Self::Continue { value, target }
    }

    pub const fn empty() -> Self {
        Self::Empty
    }

    pub const fn kind(&self) -> CompletionKindIr {
        match self {
            Self::Normal { .. } => CompletionKindIr::Normal,
            Self::Throw { .. } => CompletionKindIr::Throw,
            Self::Return { .. } => CompletionKindIr::Return,
            Self::Break { .. } => CompletionKindIr::Break,
            Self::Continue { .. } => CompletionKindIr::Continue,
            Self::Empty => CompletionKindIr::Empty,
        }
    }

    pub const fn is_abrupt(&self) -> bool {
        match self {
            Self::Normal { .. } | Self::Empty => false,
            Self::Throw { .. }
            | Self::Return { .. }
            | Self::Break { .. }
            | Self::Continue { .. } => true,
        }
    }

    pub const fn value(&self) -> Option<&T> {
        match self {
            Self::Normal { value } | Self::Break { value, .. } | Self::Continue { value, .. } => {
                value.as_ref()
            }
            Self::Throw { value } | Self::Return { value } => Some(value),
            Self::Empty => None,
        }
    }

    pub fn target(&self) -> Option<&str> {
        match self {
            Self::Break { target, .. } | Self::Continue { target, .. } => target.as_deref(),
            Self::Normal { .. } | Self::Throw { .. } | Self::Return { .. } | Self::Empty => None,
        }
    }

    pub fn update_empty(self, replacement: T) -> Self {
        match self {
            Self::Normal { value: None } => Self::Normal {
                value: Some(replacement),
            },
            Self::Break {
                value: None,
                target,
            } => Self::Break {
                value: Some(replacement),
                target,
            },
            Self::Continue {
                value: None,
                target,
            } => Self::Continue {
                value: Some(replacement),
                target,
            },
            Self::Empty => Self::Normal {
                value: Some(replacement),
            },
            completion => completion,
        }
    }

    pub fn map_value<U>(self, mut map: impl FnMut(T) -> U) -> CompletionRecordIr<U> {
        match self {
            Self::Normal { value } => CompletionRecordIr::Normal {
                value: value.map(map),
            },
            Self::Throw { value } => CompletionRecordIr::Throw { value: map(value) },
            Self::Return { value } => CompletionRecordIr::Return { value: map(value) },
            Self::Break { value, target } => CompletionRecordIr::Break {
                value: value.map(map),
                target,
            },
            Self::Continue { value, target } => CompletionRecordIr::Continue {
                value: value.map(map),
                target,
            },
            Self::Empty => CompletionRecordIr::Empty,
        }
    }
}

// ---------------------------------------------------------------------------
// Part A — the catalog as evidence
// ---------------------------------------------------------------------------

/// Proof that a [`SpecOperationIr`] variant stands behind a catalog row.
///
/// The field is private and there is no public constructor: outside this module
/// the only way to obtain one is [`SpecOperationIr::emitter_evidence`], so
/// `OperationLoweringStatus::SharedWasmEmitter(..)` cannot be forged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmitterEvidence {
    operation: SpecOperationIr,
}

impl EmitterEvidence {
    pub const fn operation(self) -> SpecOperationIr {
        self.operation
    }
}

/// Why an operation has no implementation. Closed; no free text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TrackedGapReason {
    /// No `SpecOperationIr` variant and no emitter arm implements it.
    NoImplementation,
    /// A Rust model type exists in `porffor-ir` but nothing on the product path
    /// constructs it.
    ModelWithoutCallSite,
}

impl TrackedGapReason {
    pub const fn name(self) -> &'static str {
        match self {
            Self::NoImplementation => "no implementation",
            Self::ModelWithoutCallSite => "model without call site",
        }
    }
}

/// A backlog task id. The only constructor validates, in `const`, so a
/// malformed owner is a compile error rather than an assertion in a test.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct OwnerTaskId(&'static str);

impl OwnerTaskId {
    pub const fn new(id: &'static str) -> Self {
        let bytes = id.as_bytes();
        assert!(bytes.len() == 3, "owner task id must be T + two digits");
        assert!(bytes[0] == b'T', "owner task id must start with T");
        assert!(
            matches!(bytes[1], b'0'..=b'9'),
            "owner task id must be T + two digits"
        );
        assert!(
            matches!(bytes[2], b'0'..=b'9'),
            "owner task id must be T + two digits"
        );
        Self(id)
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

/// How an operation reaches emitted Wasm — or the honest statement that it does
/// not.
///
/// Three variants, and both implementation-claiming variants carry evidence
/// that cannot be produced by hand:
///
/// - `SharedWasmEmitter` needs an [`EmitterEvidence`], whose only constructor is
///   [`SpecOperationIr::emitter_evidence`].
/// - `StatementEmission` needs an [`EmissionSite`], every variant of which is
///   joined to a real function path by
///   `porffor-aot-wasm/src/emission_sites.rs::emission_sites_are_backed`.
///
/// There is deliberately no `CatalogOnly` (a variant whose only semantics was
/// "must never occur" — that is a missing variant, not a runtime check) and no
/// `SharedRustModel` (measured: zero rows for which it is true; reintroduce it
/// in the same patch that gives some model type its first product call site).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationLoweringStatus {
    SharedWasmEmitter(EmitterEvidence),
    StatementEmission(EmissionSite),
    TrackedGap {
        reason: TrackedGapReason,
        owner: OwnerTaskId,
    },
}

/// The normal codomain of an abstract operation. Closed: exactly the shapes the
/// catalog's rows use.
///
/// This exists so that the old `assert!(!entry.normal_result.is_empty())` has
/// nothing to check — emptiness is unrepresentable — and so that a typo in a
/// codomain is `E0599` rather than a string nobody reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NormalResult {
    Unused,
    Boolean,
    BooleanOrUndefined,
    String,
    Number,
    NumberOrBigInt,
    BigInt,
    Integer,
    Object,
    ObjectOrUndefined,
    ObjectOrFalse,
    Array,
    Constructor,
    CallableOrUndefined,
    PropertyKey,
    PropertyDescriptor,
    LanguageValue,
    LanguageType,
    IteratorRecord,
    CompletionRecord,
}

impl NormalResult {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Unused => "Unused",
            Self::Boolean => "Boolean",
            Self::BooleanOrUndefined => "Boolean or Undefined",
            Self::String => "String",
            Self::Number => "Number",
            Self::NumberOrBigInt => "Number or BigInt",
            Self::BigInt => "BigInt",
            Self::Integer => "Integer",
            Self::Object => "Object",
            Self::ObjectOrUndefined => "Object or Undefined",
            Self::ObjectOrFalse => "Object or false",
            Self::Array => "Array",
            Self::Constructor => "Constructor",
            Self::CallableOrUndefined => "Callable or Undefined",
            Self::PropertyKey => "PropertyKey",
            Self::PropertyDescriptor => "PropertyDescriptor",
            Self::LanguageValue => "ECMAScript language value",
            Self::LanguageType => "Type",
            Self::IteratorRecord => "IteratorRecord",
            Self::CompletionRecord => "Completion Record",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SpecOperationIr {
    IsCallable,
    IsConstructor,
    IsPropertyKey,
    ToPrimitive(ToPrimitiveHint),
    ToBoolean,
    ToNumeric,
    ToNumber,
    ToBigInt,
    ToString,
    ToObject,
    ToPropertyKey,
    ToIntegerOrInfinity,
    ToLength,
    ToIndex,
    SameValue,
    SameValueZero,
    StrictEqualityComparison,
    IsLooselyEqual,
    Get,
    GetV,
    Set,
    HasProperty,
    HasOwnProperty,
    DeletePropertyOrThrow,
    CreateDataPropertyOrThrow,
    CopyDataProperties,
    GetMethod,
    Call,
    Construct,
}

const NO_ABRUPT: &[CompletionAbruptKind] = &[];
const MAY_THROW: &[CompletionAbruptKind] = &[CompletionAbruptKind::Throw];
const CONTROL_COMPLETIONS: &[CompletionAbruptKind] = &[
    CompletionAbruptKind::Throw,
    CompletionAbruptKind::Return,
    CompletionAbruptKind::Break,
    CompletionAbruptKind::Continue,
];

impl SpecOperationIr {
    /// Every variant, in catalog order. See ledger **L1**: stable Rust has no
    /// `variant_count`, so "you added a variant and forgot to list it here" is
    /// the one drift this area cannot make a compile error. It is bounded —
    /// rows are *derived*, so a missing entry yields an incomplete enumeration,
    /// never a false claim — and is covered by
    /// `spec_operation_all_is_complete_and_dense`.
    pub const ALL: &'static [SpecOperationIr] = &[
        Self::IsCallable,
        Self::IsConstructor,
        Self::IsPropertyKey,
        Self::ToPrimitive(ToPrimitiveHint::Default),
        Self::ToBoolean,
        Self::ToNumeric,
        Self::ToNumber,
        Self::ToBigInt,
        Self::ToString,
        Self::ToObject,
        Self::ToPropertyKey,
        Self::ToIntegerOrInfinity,
        Self::ToLength,
        Self::ToIndex,
        Self::SameValue,
        Self::SameValueZero,
        Self::StrictEqualityComparison,
        Self::IsLooselyEqual,
        Self::Get,
        Self::GetV,
        Self::Set,
        Self::HasProperty,
        Self::HasOwnProperty,
        Self::DeletePropertyOrThrow,
        Self::CreateDataPropertyOrThrow,
        Self::CopyDataProperties,
        Self::GetMethod,
        Self::Call,
        Self::Construct,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::IsCallable => "IsCallable",
            Self::IsConstructor => "IsConstructor",
            Self::IsPropertyKey => "IsPropertyKey",
            Self::ToPrimitive(_) => "ToPrimitive",
            Self::ToBoolean => "ToBoolean",
            Self::ToNumeric => "ToNumeric",
            Self::ToNumber => "ToNumber",
            Self::ToBigInt => "ToBigInt",
            Self::ToString => "ToString",
            Self::ToObject => "ToObject",
            Self::ToPropertyKey => "ToPropertyKey",
            Self::ToIntegerOrInfinity => "ToIntegerOrInfinity",
            Self::ToLength => "ToLength",
            Self::ToIndex => "ToIndex",
            Self::SameValue => "SameValue",
            Self::SameValueZero => "SameValueZero",
            Self::StrictEqualityComparison => "StrictEqualityComparison",
            Self::IsLooselyEqual => "IsLooselyEqual",
            Self::Get => "Get",
            Self::GetV => "GetV",
            Self::Set => "Set",
            Self::HasProperty => "HasProperty",
            Self::HasOwnProperty => "HasOwnProperty",
            Self::DeletePropertyOrThrow => "DeletePropertyOrThrow",
            Self::CreateDataPropertyOrThrow => "CreateDataPropertyOrThrow",
            Self::CopyDataProperties => "CopyDataProperties",
            Self::GetMethod => "GetMethod",
            Self::Call => "Call",
            Self::Construct => "Construct",
        }
    }

    pub const fn family(self) -> SpecOperationFamily {
        match self {
            Self::IsCallable | Self::IsConstructor | Self::IsPropertyKey => {
                SpecOperationFamily::TypeQuery
            }
            Self::ToPrimitive(_)
            | Self::ToBoolean
            | Self::ToNumeric
            | Self::ToNumber
            | Self::ToBigInt
            | Self::ToString
            | Self::ToObject
            | Self::ToPropertyKey
            | Self::ToIntegerOrInfinity
            | Self::ToLength
            | Self::ToIndex => SpecOperationFamily::Conversion,
            Self::SameValue
            | Self::SameValueZero
            | Self::StrictEqualityComparison
            | Self::IsLooselyEqual => SpecOperationFamily::Comparison,
            Self::Get
            | Self::GetV
            | Self::Set
            | Self::HasProperty
            | Self::HasOwnProperty
            | Self::DeletePropertyOrThrow
            | Self::CreateDataPropertyOrThrow
            | Self::CopyDataProperties => SpecOperationFamily::Object,
            Self::GetMethod | Self::Call | Self::Construct => SpecOperationFamily::Invocation,
        }
    }

    pub const fn normal_result(self) -> NormalResult {
        match self {
            Self::IsCallable
            | Self::IsConstructor
            | Self::IsPropertyKey
            | Self::ToBoolean
            | Self::SameValue
            | Self::SameValueZero
            | Self::StrictEqualityComparison
            | Self::IsLooselyEqual
            | Self::Set
            | Self::HasProperty
            | Self::HasOwnProperty
            | Self::DeletePropertyOrThrow => NormalResult::Boolean,
            Self::ToPrimitive(_) | Self::Get | Self::GetV | Self::Call => {
                NormalResult::LanguageValue
            }
            Self::ToNumeric => NormalResult::NumberOrBigInt,
            Self::ToNumber | Self::ToIntegerOrInfinity => NormalResult::Number,
            Self::ToBigInt => NormalResult::BigInt,
            Self::ToString => NormalResult::String,
            Self::ToObject | Self::Construct => NormalResult::Object,
            Self::ToPropertyKey => NormalResult::PropertyKey,
            Self::ToLength | Self::ToIndex => NormalResult::Integer,
            Self::CreateDataPropertyOrThrow | Self::CopyDataProperties => NormalResult::Unused,
            Self::GetMethod => NormalResult::CallableOrUndefined,
        }
    }

    /// The abrupt completions this operation may return.
    ///
    /// This is a **total function of the variant**, not a per-row argument.
    /// Commit `ca09433c1` (ToPrimitive abrupt completions compiled as if they
    /// could not escape) is the shape of the defect a free `abrupt` field
    /// allows; here there is no parameter to get wrong.
    ///
    /// Whether these sets agree with what the emitter arms actually emit is
    /// **ledger L2** — `porffor-ir` cannot see `porffor-aot-wasm`.
    pub const fn abrupt(self) -> &'static [CompletionAbruptKind] {
        match self {
            Self::IsCallable
            | Self::IsConstructor
            | Self::IsPropertyKey
            | Self::ToBoolean
            | Self::SameValue
            | Self::SameValueZero
            | Self::StrictEqualityComparison => NO_ABRUPT,
            Self::ToPrimitive(_)
            | Self::ToNumeric
            | Self::ToNumber
            | Self::ToBigInt
            | Self::ToString
            | Self::ToObject
            | Self::ToPropertyKey
            | Self::ToIntegerOrInfinity
            | Self::ToLength
            | Self::ToIndex
            | Self::IsLooselyEqual
            | Self::Get
            | Self::GetV
            | Self::Set
            | Self::HasProperty
            | Self::HasOwnProperty
            | Self::DeletePropertyOrThrow
            | Self::CreateDataPropertyOrThrow
            | Self::CopyDataProperties
            | Self::GetMethod
            | Self::Call
            | Self::Construct => MAY_THROW,
        }
    }

    /// Position of this operation's row in [`SPEC_OPERATION_CATALOG`]. Exists
    /// for the density const assert (J3), not for lookup.
    pub const fn catalog_index(self) -> usize {
        match self {
            Self::IsCallable => 0,
            Self::IsConstructor => 1,
            Self::IsPropertyKey => 2,
            Self::ToPrimitive(_) => 3,
            Self::ToBoolean => 4,
            Self::ToNumeric => 5,
            Self::ToNumber => 6,
            Self::ToBigInt => 7,
            Self::ToString => 8,
            Self::ToObject => 9,
            Self::ToPropertyKey => 10,
            Self::ToIntegerOrInfinity => 11,
            Self::ToLength => 12,
            Self::ToIndex => 13,
            Self::SameValue => 14,
            Self::SameValueZero => 15,
            Self::StrictEqualityComparison => 16,
            Self::IsLooselyEqual => 17,
            Self::Get => 18,
            Self::GetV => 19,
            Self::Set => 20,
            Self::HasProperty => 21,
            Self::HasOwnProperty => 22,
            Self::DeletePropertyOrThrow => 23,
            Self::CreateDataPropertyOrThrow => 24,
            Self::CopyDataProperties => 25,
            Self::GetMethod => 26,
            Self::Call => 27,
            Self::Construct => 28,
        }
    }

    /// The only constructor of [`EmitterEvidence`].
    pub const fn emitter_evidence(self) -> EmitterEvidence {
        EmitterEvidence { operation: self }
    }

    /// The row. Not a table entry that happens to match the variant — the row
    /// *is* the variant, so a variant without a row is not expressible, and a
    /// row whose signature disagrees with the variant is not expressible
    /// either.
    pub const fn catalog_entry(self) -> SpecOperationCatalogEntry {
        SpecOperationCatalogEntry {
            name: self.name(),
            family: self.family(),
            normal_result: self.normal_result(),
            abrupt: self.abrupt(),
            lowering_status: OperationLoweringStatus::SharedWasmEmitter(self.emitter_evidence()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpecOperationCatalogEntry {
    pub name: &'static str,
    pub family: SpecOperationFamily,
    pub normal_result: NormalResult,
    pub abrupt: &'static [CompletionAbruptKind],
    pub lowering_status: OperationLoweringStatus,
}

/// A row for an operation emitted by a statement-shaped emitter arm rather than
/// by a `SpecOperationIr` arm.
///
/// It cannot claim the shared emitter: there is no field that can hold an
/// [`EmitterEvidence`]. What it *can* claim is that a named function emits it,
/// and that name is checked by
/// `porffor-aot-wasm/src/emission_sites.rs::emission_sites_are_backed`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatementEmissionRow {
    pub name: &'static str,
    pub family: SpecOperationFamily,
    pub normal_result: NormalResult,
    pub abrupt: &'static [CompletionAbruptKind],
    pub site: EmissionSite,
}

impl StatementEmissionRow {
    pub const fn into_entry(self) -> SpecOperationCatalogEntry {
        SpecOperationCatalogEntry {
            name: self.name,
            family: self.family,
            normal_result: self.normal_result,
            abrupt: self.abrupt,
            lowering_status: OperationLoweringStatus::StatementEmission(self.site),
        }
    }
}

/// A row for an operation we have *not* implemented. By construction it cannot
/// carry `SharedWasmEmitter` or `StatementEmission`: there is no field for one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrackedGapRow {
    pub name: &'static str,
    pub family: SpecOperationFamily,
    pub normal_result: NormalResult,
    pub abrupt: &'static [CompletionAbruptKind],
    pub reason: TrackedGapReason,
    pub owner: OwnerTaskId,
}

impl TrackedGapRow {
    pub const fn into_entry(self) -> SpecOperationCatalogEntry {
        SpecOperationCatalogEntry {
            name: self.name,
            family: self.family,
            normal_result: self.normal_result,
            abrupt: self.abrupt,
            lowering_status: OperationLoweringStatus::TrackedGap {
                reason: self.reason,
                owner: self.owner,
            },
        }
    }
}

const T04: OwnerTaskId = OwnerTaskId::new("T04");

/// The four 7.4 obligations plus `AsyncIteratorClose`, emitted by the for-of
/// statement arms rather than by a `SpecOperationIr` arm.
///
/// Evidence, read rather than assumed: `compile_for_of_iterator`
/// (`control_flow.rs:7422`) emits the `@@iterator` `Get`, the `Call`, the
/// once-only `Get` of `"next"`, the per-step `next()` call and the
/// `"done"`/`"value"` reads, and routes exits through
/// `emit_iterator_close_condition_i32` into `emit_iterator_close` or
/// `emit_iterator_close_preserving_current_throw`.
/// `compile_async_for_of_iterator` (`control_flow.rs:6078`) open-codes the async
/// close.
pub const STATEMENT_EMISSION_ROWS: &[StatementEmissionRow] = &[
    StatementEmissionRow {
        name: "GetIterator",
        family: SpecOperationFamily::Iterator,
        normal_result: NormalResult::IteratorRecord,
        abrupt: MAY_THROW,
        site: EmissionSite::SyncForOfIterator,
    },
    StatementEmissionRow {
        name: "IteratorStep",
        family: SpecOperationFamily::Iterator,
        normal_result: NormalResult::ObjectOrFalse,
        abrupt: MAY_THROW,
        site: EmissionSite::SyncForOfIterator,
    },
    StatementEmissionRow {
        name: "IteratorValue",
        family: SpecOperationFamily::Iterator,
        normal_result: NormalResult::LanguageValue,
        abrupt: MAY_THROW,
        site: EmissionSite::SyncForOfIterator,
    },
    StatementEmissionRow {
        name: "IteratorClose",
        family: SpecOperationFamily::Iterator,
        normal_result: NormalResult::CompletionRecord,
        abrupt: CONTROL_COMPLETIONS,
        site: EmissionSite::SyncForOfIterator,
    },
    StatementEmissionRow {
        name: "AsyncIteratorClose",
        family: SpecOperationFamily::Iterator,
        normal_result: NormalResult::CompletionRecord,
        abrupt: CONTROL_COMPLETIONS,
        site: EmissionSite::AsyncForOfIterator,
    },
];

/// Operations with no implementation on the product path.
///
/// `ModelWithoutCallSite` marks the three whose Rust model exists and is
/// correct but has zero product call sites (measured); `NoImplementation` marks
/// the nine whose model type was deleted along with the false row, because a
/// type with no call site is a claim and this area exists because claims were
/// being read as implementations.
pub const TRACKED_GAP_ROWS: &[TrackedGapRow] = &[
    TrackedGapRow {
        name: "Type",
        family: SpecOperationFamily::TypeQuery,
        normal_result: NormalResult::LanguageType,
        abrupt: NO_ABRUPT,
        reason: TrackedGapReason::ModelWithoutCallSite,
        owner: T04,
    },
    TrackedGapRow {
        name: "IntegerIndexedConversion",
        family: SpecOperationFamily::Conversion,
        normal_result: NormalResult::Integer,
        abrupt: MAY_THROW,
        reason: TrackedGapReason::NoImplementation,
        owner: T04,
    },
    TrackedGapRow {
        name: "IsLessThan",
        family: SpecOperationFamily::Comparison,
        normal_result: NormalResult::BooleanOrUndefined,
        abrupt: MAY_THROW,
        reason: TrackedGapReason::NoImplementation,
        owner: T04,
    },
    TrackedGapRow {
        name: "CreateDataProperty",
        family: SpecOperationFamily::Object,
        normal_result: NormalResult::Boolean,
        abrupt: MAY_THROW,
        reason: TrackedGapReason::NoImplementation,
        owner: T04,
    },
    TrackedGapRow {
        name: "DefinePropertyOrThrow",
        family: SpecOperationFamily::Object,
        normal_result: NormalResult::Unused,
        abrupt: MAY_THROW,
        reason: TrackedGapReason::NoImplementation,
        owner: T04,
    },
    TrackedGapRow {
        name: "ToPropertyDescriptor",
        family: SpecOperationFamily::Object,
        normal_result: NormalResult::PropertyDescriptor,
        abrupt: MAY_THROW,
        reason: TrackedGapReason::NoImplementation,
        owner: T04,
    },
    TrackedGapRow {
        name: "FromPropertyDescriptor",
        family: SpecOperationFamily::Object,
        normal_result: NormalResult::ObjectOrUndefined,
        abrupt: MAY_THROW,
        reason: TrackedGapReason::NoImplementation,
        owner: T04,
    },
    TrackedGapRow {
        name: "OrdinaryCreateFromConstructor",
        family: SpecOperationFamily::Invocation,
        normal_result: NormalResult::Object,
        abrupt: MAY_THROW,
        reason: TrackedGapReason::NoImplementation,
        owner: T04,
    },
    TrackedGapRow {
        name: "SpeciesConstructor",
        family: SpecOperationFamily::Invocation,
        normal_result: NormalResult::Constructor,
        abrupt: MAY_THROW,
        reason: TrackedGapReason::NoImplementation,
        owner: T04,
    },
    TrackedGapRow {
        name: "ArraySpeciesCreate",
        family: SpecOperationFamily::Invocation,
        normal_result: NormalResult::Array,
        abrupt: MAY_THROW,
        reason: TrackedGapReason::NoImplementation,
        owner: T04,
    },
    TrackedGapRow {
        name: "Completion",
        family: SpecOperationFamily::Completion,
        normal_result: NormalResult::CompletionRecord,
        abrupt: CONTROL_COMPLETIONS,
        reason: TrackedGapReason::ModelWithoutCallSite,
        owner: T04,
    },
    TrackedGapRow {
        name: "UpdateEmpty",
        family: SpecOperationFamily::Completion,
        normal_result: NormalResult::CompletionRecord,
        abrupt: CONTROL_COMPLETIONS,
        reason: TrackedGapReason::ModelWithoutCallSite,
        owner: T04,
    },
];

pub const SPEC_OPERATION_ROW_COUNT: usize =
    SpecOperationIr::ALL.len() + STATEMENT_EMISSION_ROWS.len() + TRACKED_GAP_ROWS.len();

const CATALOG_PLACEHOLDER: SpecOperationCatalogEntry = SpecOperationIr::IsCallable.catalog_entry();

const fn build_catalog() -> [SpecOperationCatalogEntry; SPEC_OPERATION_ROW_COUNT] {
    let mut rows = [CATALOG_PLACEHOLDER; SPEC_OPERATION_ROW_COUNT];
    let mut next = 0;

    let mut i = 0;
    while i < SpecOperationIr::ALL.len() {
        rows[next] = SpecOperationIr::ALL[i].catalog_entry();
        next += 1;
        i += 1;
    }

    let mut j = 0;
    while j < STATEMENT_EMISSION_ROWS.len() {
        rows[next] = STATEMENT_EMISSION_ROWS[j].into_entry();
        next += 1;
        j += 1;
    }

    let mut k = 0;
    while k < TRACKED_GAP_ROWS.len() {
        rows[next] = TRACKED_GAP_ROWS[k].into_entry();
        next += 1;
        k += 1;
    }

    rows
}

const CATALOG: [SpecOperationCatalogEntry; SPEC_OPERATION_ROW_COUNT] = build_catalog();

/// The catalog. A `static` rather than a `const` so that `.iter()` yields
/// `&'static` entries; `CATALOG` is the `const` the assertions below evaluate.
pub static SPEC_OPERATION_CATALOG: [SpecOperationCatalogEntry; SPEC_OPERATION_ROW_COUNT] = CATALOG;

/// Byte-wise `str` equality, usable in `const`.
const fn str_eq(a: &str, b: &str) -> bool {
    let a = a.as_bytes();
    let b = b.as_bytes();
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

// (J1) Every catalog name is distinct. Replaces the runtime test
//      `operations_catalog_names_are_unique`. Because emitter rows are derived
//      from `SpecOperationIr` and every hand-written row is a
//      `StatementEmissionRow` or `TrackedGapRow`, this also subsumes (J2): a
//      gap row can never shadow an implemented operation.
const _: () = {
    let mut i = 0;
    while i < SPEC_OPERATION_ROW_COUNT {
        let mut j = i + 1;
        while j < SPEC_OPERATION_ROW_COUNT {
            assert!(
                !str_eq(CATALOG[i].name, CATALOG[j].name),
                "duplicate spec operation name"
            );
            j += 1;
        }
        i += 1;
    }
};

// (J3) `SpecOperationIr::ALL` is dense and duplicate-free under
//      `catalog_index`, so the derived rows occupy exactly slots 0..ALL.len().
const _: () = {
    let mut seen = [false; SPEC_OPERATION_ROW_COUNT];
    let mut i = 0;
    while i < SpecOperationIr::ALL.len() {
        let idx = SpecOperationIr::ALL[i].catalog_index();
        assert!(!seen[idx], "duplicate catalog_index");
        seen[idx] = true;
        i += 1;
    }
    let mut j = 0;
    while j < SpecOperationIr::ALL.len() {
        assert!(seen[j], "catalog_index is not dense over SpecOperationIr::ALL");
        j += 1;
    }
};

// (J4) The census the contract states, tied to the tables that produce it:
//      29 shared-emitter rows + 5 statement-emission rows + 12 tracked gaps.
//      Changing any of the three tables without restating the census here is a
//      compile error, which is the point — the row counts are the claim this
//      area exists to keep honest.
const _: () = {
    assert!(SpecOperationIr::ALL.len() == 29);
    assert!(STATEMENT_EMISSION_ROWS.len() == 5);
    assert!(TRACKED_GAP_ROWS.len() == 12);
    assert!(SPEC_OPERATION_ROW_COUNT == 46);
};

// (J5) Row order: the derived rows come first, so `catalog_index` addresses the
//      catalog directly.
const _: () = {
    let mut i = 0;
    while i < SpecOperationIr::ALL.len() {
        let operation = SpecOperationIr::ALL[i];
        assert!(
            str_eq(CATALOG[operation.catalog_index()].name, operation.name()),
            "catalog_index does not address the operation's own row"
        );
        i += 1;
    }
};

pub fn spec_operation_catalog() -> &'static [SpecOperationCatalogEntry] {
    &SPEC_OPERATION_CATALOG
}

pub fn find_spec_operation(name: &str) -> Option<&'static SpecOperationCatalogEntry> {
    SPEC_OPERATION_CATALOG
        .iter()
        .find(|entry| entry.name == name)
}

pub fn completion_abi_slots() -> &'static [CompletionAbiSlot] {
    COMPLETION_ABI_SLOTS
}

pub fn find_completion_abi_slot(kind: CompletionKindIr) -> Option<&'static CompletionAbiSlot> {
    COMPLETION_ABI_SLOTS.iter().find(|slot| slot.kind == kind)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ledger **L1**. This is the only surviving catalog test, and it is not
    /// vacuous: it checks the *assembly* (that `ALL` enumerates every variant
    /// and that the assembled catalog has the shape the const asserts assume),
    /// not the table's own contents. Stable Rust has no `variant_count`, so a
    /// variant absent from `ALL` is invisible to every const expression.
    #[test]
    fn spec_operation_all_is_complete_and_dense() {
        assert_eq!(
            SPEC_OPERATION_CATALOG.len(),
            SPEC_OPERATION_ROW_COUNT,
            "assembled catalog length disagrees with SPEC_OPERATION_ROW_COUNT"
        );

        for (index, operation) in SpecOperationIr::ALL.iter().enumerate() {
            assert_eq!(
                operation.catalog_index(),
                index,
                "SpecOperationIr::ALL order disagrees with catalog_index for {}; \
                 if you added a variant, add it to SpecOperationIr::ALL",
                operation.name()
            );
            assert_eq!(
                SPEC_OPERATION_CATALOG[index].name,
                operation.name(),
                "SpecOperationIr::ALL does not address its own catalog row"
            );
        }

        // A variant missing from `ALL` shows up here as a hole in the index
        // space: `catalog_index` hands out 0..=28, so `ALL` must have 29 rows.
        assert_eq!(
            SpecOperationIr::ALL.len(),
            29,
            "SpecOperationIr::ALL is missing a variant"
        );
    }

    /// Not a test of the type's own contents: it pins that each slot reads back
    /// through its own accessor, which is the property a transposition at the
    /// construction site would break. The transposition itself is `E0308` and
    /// therefore has no test.
    #[test]
    fn operations_iterator_record_slots_read_back_through_their_own_accessor() {
        let record = IteratorRecordIr::new(
            IteratorSlot::new("async.forof.iterator.0".to_string()),
            NextMethodSlot::new("async.forof.next.0".to_string()),
            DoneSlot::new("async.forof.done.0".to_string()),
        );

        assert_eq!(record.iterator().as_str(), "async.forof.iterator.0");
        assert_eq!(record.next_method().as_str(), "async.forof.next.0");
        assert_eq!(record.done().as_str(), "async.forof.done.0");
    }

    #[test]
    fn operations_ecmascript_language_type_reports_names_and_primitive_status() {
        for primitive in [
            EcmaLanguageType::Undefined,
            EcmaLanguageType::Null,
            EcmaLanguageType::Boolean,
            EcmaLanguageType::String,
            EcmaLanguageType::Symbol,
            EcmaLanguageType::Number,
            EcmaLanguageType::BigInt,
        ] {
            assert!(!primitive.name().is_empty());
            assert!(primitive.is_primitive());
        }

        assert_eq!(EcmaLanguageType::Object.name(), "Object");
        assert!(!EcmaLanguageType::Object.is_primitive());
    }

    #[test]
    fn operations_completion_abi_slots_are_stable_and_dense() {
        for (expected, slot) in completion_abi_slots().iter().enumerate() {
            assert_eq!(
                slot.code, expected as i64,
                "completion kind {} should stay at {}",
                slot.name, expected
            );
            assert_eq!(slot.name, slot.kind.name());
            assert_eq!(slot.carries_value, slot.kind.carries_value());
            assert_eq!(slot.carries_target, slot.kind.carries_target());
        }
    }

    #[test]
    fn operations_completion_abi_documents_values_and_targets() {
        let value_slots = completion_abi_slots()
            .iter()
            .filter(|slot| slot.carries_value)
            .map(|slot| slot.name)
            .collect::<Vec<_>>();
        assert_eq!(
            value_slots,
            vec!["normal", "throw", "return", "break", "continue"]
        );

        let target_slots = completion_abi_slots()
            .iter()
            .filter(|slot| slot.carries_target)
            .map(|slot| slot.name)
            .collect::<Vec<_>>();
        assert_eq!(target_slots, vec!["break", "continue"]);
    }

    #[test]
    fn operations_completion_abi_slot_lookup_matches_kind() {
        for slot in COMPLETION_ABI_SLOTS {
            assert_eq!(find_completion_abi_slot(slot.kind), Some(slot));
        }
    }

    #[test]
    fn operations_completion_record_reports_kind_value_target_and_abruptness() {
        let normal = CompletionRecordIr::normal("ok");
        assert_eq!(normal.kind(), CompletionKindIr::Normal);
        assert_eq!(normal.value(), Some(&"ok"));
        assert_eq!(normal.target(), None);
        assert!(!normal.is_abrupt());

        let throw = CompletionRecordIr::throw("boom");
        assert_eq!(throw.kind(), CompletionKindIr::Throw);
        assert_eq!(throw.value(), Some(&"boom"));
        assert!(throw.is_abrupt());

        let break_ = CompletionRecordIr::break_(None::<&str>, Some("outer".to_string()));
        assert_eq!(break_.kind(), CompletionKindIr::Break);
        assert_eq!(break_.value(), None);
        assert_eq!(break_.target(), Some("outer"));
        assert!(break_.is_abrupt());
    }

    #[test]
    fn operations_completion_record_update_empty_preserves_abrupt_kind() {
        assert_eq!(
            CompletionRecordIr::empty_normal().update_empty("fallback"),
            CompletionRecordIr::normal("fallback")
        );
        assert_eq!(
            CompletionRecordIr::empty().update_empty("fallback"),
            CompletionRecordIr::normal("fallback")
        );
        assert_eq!(
            CompletionRecordIr::break_(None, Some("outer".to_string())).update_empty("fallback"),
            CompletionRecordIr::break_(Some("fallback"), Some("outer".to_string()))
        );
        assert_eq!(
            CompletionRecordIr::throw("boom").update_empty("fallback"),
            CompletionRecordIr::throw("boom")
        );
    }

    #[test]
    fn operations_completion_record_map_value_keeps_empty_slots_empty() {
        let mapped = CompletionRecordIr::continue_(Some(2), None).map_value(|value| value * 3);
        assert_eq!(mapped, CompletionRecordIr::continue_(Some(6), None));

        let empty: CompletionRecordIr<i32> = CompletionRecordIr::empty();
        assert_eq!(
            empty.map_value(|value| value * 3),
            CompletionRecordIr::Empty
        );
    }
}
