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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AbstractRelationalComparisonResult {
    True,
    False,
    Undefined,
}

impl AbstractRelationalComparisonResult {
    pub const fn from_bool(value: bool) -> Self {
        if value {
            Self::True
        } else {
            Self::False
        }
    }

    pub const fn as_bool(self) -> Option<bool> {
        match self {
            Self::True => Some(true),
            Self::False => Some(false),
            Self::Undefined => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IntegerIndexedElementType {
    Int8,
    Uint8,
    Uint8Clamped,
    Int16,
    Uint16,
    Int32,
    Uint32,
    BigInt64,
    BigUint64,
    Float16,
    Float32,
    Float64,
}

impl IntegerIndexedElementType {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Int8 => "Int8",
            Self::Uint8 => "Uint8",
            Self::Uint8Clamped => "Uint8Clamped",
            Self::Int16 => "Int16",
            Self::Uint16 => "Uint16",
            Self::Int32 => "Int32",
            Self::Uint32 => "Uint32",
            Self::BigInt64 => "BigInt64",
            Self::BigUint64 => "BigUint64",
            Self::Float16 => "Float16",
            Self::Float32 => "Float32",
            Self::Float64 => "Float64",
        }
    }

    pub const fn uses_bigint_conversion(self) -> bool {
        matches!(self, Self::BigInt64 | Self::BigUint64)
    }

    pub const fn uses_number_conversion(self) -> bool {
        !self.uses_bigint_conversion()
    }

    pub const fn is_clamped(self) -> bool {
        matches!(self, Self::Uint8Clamped)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntegerIndexedConversionIr {
    pub element_type: IntegerIndexedElementType,
}

impl IntegerIndexedConversionIr {
    pub const fn new(element_type: IntegerIndexedElementType) -> Self {
        Self { element_type }
    }

    pub const fn requires_bigint_input(self) -> bool {
        self.element_type.uses_bigint_conversion()
    }

    pub const fn requires_number_input(self) -> bool {
        self.element_type.uses_number_conversion()
    }

    pub const fn clamps_result(self) -> bool {
        self.element_type.is_clamped()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IteratorRecordKind {
    Sync,
    Async,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IteratorRecordIr<T> {
    pub iterator: T,
    pub next_method: T,
    pub done: bool,
    pub kind: IteratorRecordKind,
}

impl<T> IteratorRecordIr<T> {
    pub const fn sync(iterator: T, next_method: T) -> Self {
        Self {
            iterator,
            next_method,
            done: false,
            kind: IteratorRecordKind::Sync,
        }
    }

    pub const fn async_(iterator: T, next_method: T) -> Self {
        Self {
            iterator,
            next_method,
            done: false,
            kind: IteratorRecordKind::Async,
        }
    }

    pub const fn is_done(&self) -> bool {
        self.done
    }

    pub fn mark_done(&mut self) {
        self.done = true;
    }

    pub fn map_value<U>(self, mut map: impl FnMut(T) -> U) -> IteratorRecordIr<U> {
        IteratorRecordIr {
            iterator: map(self.iterator),
            next_method: map(self.next_method),
            done: self.done,
            kind: self.kind,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PropertyDescriptorKind {
    Data,
    Accessor,
    Generic,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropertyDescriptorIr<T> {
    Data {
        value: Option<T>,
        writable: Option<bool>,
        enumerable: Option<bool>,
        configurable: Option<bool>,
    },
    Accessor {
        get: Option<T>,
        set: Option<T>,
        enumerable: Option<bool>,
        configurable: Option<bool>,
    },
    Generic {
        enumerable: Option<bool>,
        configurable: Option<bool>,
    },
}

impl<T> PropertyDescriptorIr<T> {
    pub const fn data(
        value: Option<T>,
        writable: Option<bool>,
        enumerable: Option<bool>,
        configurable: Option<bool>,
    ) -> Self {
        Self::Data {
            value,
            writable,
            enumerable,
            configurable,
        }
    }

    pub const fn accessor(
        get: Option<T>,
        set: Option<T>,
        enumerable: Option<bool>,
        configurable: Option<bool>,
    ) -> Self {
        Self::Accessor {
            get,
            set,
            enumerable,
            configurable,
        }
    }

    pub const fn generic(enumerable: Option<bool>, configurable: Option<bool>) -> Self {
        Self::Generic {
            enumerable,
            configurable,
        }
    }

    pub const fn kind(&self) -> PropertyDescriptorKind {
        match self {
            Self::Data { .. } => PropertyDescriptorKind::Data,
            Self::Accessor { .. } => PropertyDescriptorKind::Accessor,
            Self::Generic { .. } => PropertyDescriptorKind::Generic,
        }
    }

    pub const fn is_data_descriptor(&self) -> bool {
        matches!(self, Self::Data { .. })
    }

    pub const fn is_accessor_descriptor(&self) -> bool {
        matches!(self, Self::Accessor { .. })
    }

    pub const fn is_generic_descriptor(&self) -> bool {
        matches!(self, Self::Generic { .. })
    }

    pub const fn enumerable(&self) -> Option<bool> {
        match self {
            Self::Data { enumerable, .. }
            | Self::Accessor { enumerable, .. }
            | Self::Generic { enumerable, .. } => *enumerable,
        }
    }

    pub const fn configurable(&self) -> Option<bool> {
        match self {
            Self::Data { configurable, .. }
            | Self::Accessor { configurable, .. }
            | Self::Generic { configurable, .. } => *configurable,
        }
    }

    pub fn complete(self, default_value: T) -> Self
    where
        T: Clone,
    {
        match self {
            Self::Data {
                value,
                writable,
                enumerable,
                configurable,
            } => Self::Data {
                value: value.or(Some(default_value)),
                writable: Some(writable.unwrap_or(false)),
                enumerable: Some(enumerable.unwrap_or(false)),
                configurable: Some(configurable.unwrap_or(false)),
            },
            Self::Generic {
                enumerable,
                configurable,
            } => Self::Data {
                value: Some(default_value),
                writable: Some(false),
                enumerable: Some(enumerable.unwrap_or(false)),
                configurable: Some(configurable.unwrap_or(false)),
            },
            Self::Accessor {
                get,
                set,
                enumerable,
                configurable,
            } => Self::Accessor {
                get: get.or(Some(default_value.clone())),
                set: set.or(Some(default_value)),
                enumerable: Some(enumerable.unwrap_or(false)),
                configurable: Some(configurable.unwrap_or(false)),
            },
        }
    }

    pub fn map_value<U>(self, mut map: impl FnMut(T) -> U) -> PropertyDescriptorIr<U> {
        match self {
            Self::Data {
                value,
                writable,
                enumerable,
                configurable,
            } => PropertyDescriptorIr::Data {
                value: value.map(&mut map),
                writable,
                enumerable,
                configurable,
            },
            Self::Accessor {
                get,
                set,
                enumerable,
                configurable,
            } => PropertyDescriptorIr::Accessor {
                get: get.map(&mut map),
                set: set.map(&mut map),
                enumerable,
                configurable,
            },
            Self::Generic {
                enumerable,
                configurable,
            } => PropertyDescriptorIr::Generic {
                enumerable,
                configurable,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateDataPropertyIr<T> {
    pub target: T,
    pub key: T,
    pub value: T,
}

impl<T> CreateDataPropertyIr<T> {
    pub const fn new(target: T, key: T, value: T) -> Self {
        Self { target, key, value }
    }

    pub fn map_value<U>(self, mut map: impl FnMut(T) -> U) -> CreateDataPropertyIr<U> {
        CreateDataPropertyIr {
            target: map(self.target),
            key: map(self.key),
            value: map(self.value),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefinePropertyIr<T> {
    pub target: T,
    pub key: T,
    pub descriptor: PropertyDescriptorIr<T>,
}

impl<T> DefinePropertyIr<T> {
    pub const fn new(target: T, key: T, descriptor: PropertyDescriptorIr<T>) -> Self {
        Self {
            target,
            key,
            descriptor,
        }
    }

    pub fn map_value<U>(self, mut map: impl FnMut(T) -> U) -> DefinePropertyIr<U> {
        DefinePropertyIr {
            target: map(self.target),
            key: map(self.key),
            descriptor: self.descriptor.map_value(map),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrdinaryCreateFromConstructorIr<T> {
    pub constructor: T,
    pub intrinsic_default_proto: &'static str,
}

impl<T> OrdinaryCreateFromConstructorIr<T> {
    pub const fn new(constructor: T, intrinsic_default_proto: &'static str) -> Self {
        Self {
            constructor,
            intrinsic_default_proto,
        }
    }

    pub fn map_value<U>(self, mut map: impl FnMut(T) -> U) -> OrdinaryCreateFromConstructorIr<U> {
        OrdinaryCreateFromConstructorIr {
            constructor: map(self.constructor),
            intrinsic_default_proto: self.intrinsic_default_proto,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpeciesConstructorIr<T> {
    pub object: T,
    pub default_constructor: T,
}

impl<T> SpeciesConstructorIr<T> {
    pub const fn new(object: T, default_constructor: T) -> Self {
        Self {
            object,
            default_constructor,
        }
    }

    pub fn map_value<U>(self, mut map: impl FnMut(T) -> U) -> SpeciesConstructorIr<U> {
        SpeciesConstructorIr {
            object: map(self.object),
            default_constructor: map(self.default_constructor),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArraySpeciesCreateIr<T> {
    pub original_array: T,
    pub length: T,
}

impl<T> ArraySpeciesCreateIr<T> {
    pub const fn new(original_array: T, length: T) -> Self {
        Self {
            original_array,
            length,
        }
    }

    pub fn map_value<U>(self, mut map: impl FnMut(T) -> U) -> ArraySpeciesCreateIr<U> {
        ArraySpeciesCreateIr {
            original_array: map(self.original_array),
            length: map(self.length),
        }
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationLoweringStatus {
    CatalogOnly,
    SharedRustModel,
    SharedWasmEmitter,
    TrackedGap(&'static str),
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

impl SpecOperationIr {
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpecOperationCatalogEntry {
    pub name: &'static str,
    pub family: SpecOperationFamily,
    pub normal_result: &'static str,
    pub abrupt: &'static [CompletionAbruptKind],
    pub lowering_status: OperationLoweringStatus,
}

const NO_ABRUPT: &[CompletionAbruptKind] = &[];
const MAY_THROW: &[CompletionAbruptKind] = &[CompletionAbruptKind::Throw];
const CONTROL_COMPLETIONS: &[CompletionAbruptKind] = &[
    CompletionAbruptKind::Throw,
    CompletionAbruptKind::Return,
    CompletionAbruptKind::Break,
    CompletionAbruptKind::Continue,
];

pub const SPEC_OPERATION_CATALOG: &[SpecOperationCatalogEntry] = &[
    lowered_op(
        "Type",
        SpecOperationFamily::TypeQuery,
        "Type",
        NO_ABRUPT,
        OperationLoweringStatus::SharedRustModel,
    ),
    lowered_op(
        "IsCallable",
        SpecOperationFamily::TypeQuery,
        "Boolean",
        NO_ABRUPT,
        OperationLoweringStatus::SharedWasmEmitter,
    ),
    lowered_op(
        "IsConstructor",
        SpecOperationFamily::TypeQuery,
        "Boolean",
        NO_ABRUPT,
        OperationLoweringStatus::SharedWasmEmitter,
    ),
    lowered_op(
        "IsPropertyKey",
        SpecOperationFamily::TypeQuery,
        "Boolean",
        NO_ABRUPT,
        OperationLoweringStatus::SharedWasmEmitter,
    ),
    lowered_op(
        "ToPrimitive",
        SpecOperationFamily::Conversion,
        "ECMAScript language value",
        MAY_THROW,
        OperationLoweringStatus::SharedWasmEmitter,
    ),
    lowered_op(
        "ToBoolean",
        SpecOperationFamily::Conversion,
        "Boolean",
        NO_ABRUPT,
        OperationLoweringStatus::SharedWasmEmitter,
    ),
    lowered_op(
        "ToNumeric",
        SpecOperationFamily::Conversion,
        "Number or BigInt",
        MAY_THROW,
        OperationLoweringStatus::SharedWasmEmitter,
    ),
    lowered_op(
        "ToNumber",
        SpecOperationFamily::Conversion,
        "Number",
        MAY_THROW,
        OperationLoweringStatus::SharedWasmEmitter,
    ),
    lowered_op(
        "ToBigInt",
        SpecOperationFamily::Conversion,
        "BigInt",
        MAY_THROW,
        OperationLoweringStatus::SharedWasmEmitter,
    ),
    lowered_op(
        "ToString",
        SpecOperationFamily::Conversion,
        "String",
        MAY_THROW,
        OperationLoweringStatus::SharedWasmEmitter,
    ),
    lowered_op(
        "ToObject",
        SpecOperationFamily::Conversion,
        "Object",
        MAY_THROW,
        OperationLoweringStatus::SharedWasmEmitter,
    ),
    lowered_op(
        "ToPropertyKey",
        SpecOperationFamily::Conversion,
        "PropertyKey",
        MAY_THROW,
        OperationLoweringStatus::SharedWasmEmitter,
    ),
    lowered_op(
        "ToIntegerOrInfinity",
        SpecOperationFamily::Conversion,
        "Number",
        MAY_THROW,
        OperationLoweringStatus::SharedWasmEmitter,
    ),
    lowered_op(
        "ToLength",
        SpecOperationFamily::Conversion,
        "Integer",
        MAY_THROW,
        OperationLoweringStatus::SharedWasmEmitter,
    ),
    lowered_op(
        "ToIndex",
        SpecOperationFamily::Conversion,
        "Integer",
        MAY_THROW,
        OperationLoweringStatus::SharedWasmEmitter,
    ),
    lowered_op(
        "IntegerIndexedConversion",
        SpecOperationFamily::Conversion,
        "Integer",
        MAY_THROW,
        OperationLoweringStatus::SharedRustModel,
    ),
    lowered_op(
        "SameValue",
        SpecOperationFamily::Comparison,
        "Boolean",
        NO_ABRUPT,
        OperationLoweringStatus::SharedWasmEmitter,
    ),
    lowered_op(
        "SameValueZero",
        SpecOperationFamily::Comparison,
        "Boolean",
        NO_ABRUPT,
        OperationLoweringStatus::SharedWasmEmitter,
    ),
    lowered_op(
        "StrictEqualityComparison",
        SpecOperationFamily::Comparison,
        "Boolean",
        NO_ABRUPT,
        OperationLoweringStatus::SharedWasmEmitter,
    ),
    lowered_op(
        "IsLooselyEqual",
        SpecOperationFamily::Comparison,
        "Boolean",
        MAY_THROW,
        OperationLoweringStatus::SharedWasmEmitter,
    ),
    lowered_op(
        "IsLessThan",
        SpecOperationFamily::Comparison,
        "Boolean or Undefined",
        MAY_THROW,
        OperationLoweringStatus::SharedRustModel,
    ),
    lowered_op(
        "Get",
        SpecOperationFamily::Object,
        "ECMAScript language value",
        MAY_THROW,
        OperationLoweringStatus::SharedWasmEmitter,
    ),
    lowered_op(
        "GetV",
        SpecOperationFamily::Object,
        "ECMAScript language value",
        MAY_THROW,
        OperationLoweringStatus::SharedWasmEmitter,
    ),
    lowered_op(
        "Set",
        SpecOperationFamily::Object,
        "Boolean",
        MAY_THROW,
        OperationLoweringStatus::SharedWasmEmitter,
    ),
    lowered_op(
        "HasProperty",
        SpecOperationFamily::Object,
        "Boolean",
        MAY_THROW,
        OperationLoweringStatus::SharedWasmEmitter,
    ),
    lowered_op(
        "HasOwnProperty",
        SpecOperationFamily::Object,
        "Boolean",
        MAY_THROW,
        OperationLoweringStatus::SharedWasmEmitter,
    ),
    lowered_op(
        "DeletePropertyOrThrow",
        SpecOperationFamily::Object,
        "Boolean",
        MAY_THROW,
        OperationLoweringStatus::SharedWasmEmitter,
    ),
    lowered_op(
        "CreateDataProperty",
        SpecOperationFamily::Object,
        "Boolean",
        MAY_THROW,
        OperationLoweringStatus::SharedRustModel,
    ),
    lowered_op(
        "CreateDataPropertyOrThrow",
        SpecOperationFamily::Object,
        "Unused",
        MAY_THROW,
        OperationLoweringStatus::SharedWasmEmitter,
    ),
    lowered_op(
        "CopyDataProperties",
        SpecOperationFamily::Object,
        "Unused",
        MAY_THROW,
        OperationLoweringStatus::SharedWasmEmitter,
    ),
    lowered_op(
        "DefinePropertyOrThrow",
        SpecOperationFamily::Object,
        "Unused",
        MAY_THROW,
        OperationLoweringStatus::SharedRustModel,
    ),
    lowered_op(
        "ToPropertyDescriptor",
        SpecOperationFamily::Object,
        "PropertyDescriptor",
        MAY_THROW,
        OperationLoweringStatus::SharedRustModel,
    ),
    lowered_op(
        "FromPropertyDescriptor",
        SpecOperationFamily::Object,
        "Object or Undefined",
        MAY_THROW,
        OperationLoweringStatus::SharedRustModel,
    ),
    lowered_op(
        "GetMethod",
        SpecOperationFamily::Invocation,
        "Callable or Undefined",
        MAY_THROW,
        OperationLoweringStatus::SharedWasmEmitter,
    ),
    lowered_op(
        "Call",
        SpecOperationFamily::Invocation,
        "ECMAScript language value",
        MAY_THROW,
        OperationLoweringStatus::SharedWasmEmitter,
    ),
    lowered_op(
        "Construct",
        SpecOperationFamily::Invocation,
        "Object",
        MAY_THROW,
        OperationLoweringStatus::SharedWasmEmitter,
    ),
    lowered_op(
        "OrdinaryCreateFromConstructor",
        SpecOperationFamily::Invocation,
        "Object",
        MAY_THROW,
        OperationLoweringStatus::SharedRustModel,
    ),
    lowered_op(
        "SpeciesConstructor",
        SpecOperationFamily::Invocation,
        "Constructor",
        MAY_THROW,
        OperationLoweringStatus::SharedRustModel,
    ),
    lowered_op(
        "ArraySpeciesCreate",
        SpecOperationFamily::Invocation,
        "Array",
        MAY_THROW,
        OperationLoweringStatus::SharedRustModel,
    ),
    lowered_op(
        "GetIterator",
        SpecOperationFamily::Iterator,
        "IteratorRecord",
        MAY_THROW,
        OperationLoweringStatus::SharedRustModel,
    ),
    lowered_op(
        "IteratorStep",
        SpecOperationFamily::Iterator,
        "Object or false",
        MAY_THROW,
        OperationLoweringStatus::SharedRustModel,
    ),
    lowered_op(
        "IteratorValue",
        SpecOperationFamily::Iterator,
        "ECMAScript language value",
        MAY_THROW,
        OperationLoweringStatus::SharedRustModel,
    ),
    lowered_op(
        "IteratorClose",
        SpecOperationFamily::Iterator,
        "Completion",
        CONTROL_COMPLETIONS,
        OperationLoweringStatus::SharedRustModel,
    ),
    lowered_op(
        "AsyncIteratorClose",
        SpecOperationFamily::Iterator,
        "Completion",
        CONTROL_COMPLETIONS,
        OperationLoweringStatus::SharedRustModel,
    ),
    lowered_op(
        "Completion",
        SpecOperationFamily::Completion,
        "Completion Record",
        CONTROL_COMPLETIONS,
        OperationLoweringStatus::SharedRustModel,
    ),
    lowered_op(
        "UpdateEmpty",
        SpecOperationFamily::Completion,
        "Completion Record",
        CONTROL_COMPLETIONS,
        OperationLoweringStatus::SharedRustModel,
    ),
];

const fn lowered_op(
    name: &'static str,
    family: SpecOperationFamily,
    normal_result: &'static str,
    abrupt: &'static [CompletionAbruptKind],
    lowering_status: OperationLoweringStatus,
) -> SpecOperationCatalogEntry {
    SpecOperationCatalogEntry {
        name,
        family,
        normal_result,
        abrupt,
        lowering_status,
    }
}

pub fn spec_operation_catalog() -> &'static [SpecOperationCatalogEntry] {
    SPEC_OPERATION_CATALOG
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
    use std::collections::BTreeSet;

    const REQUIRED_T04_OPERATIONS: &[&str] = &[
        "Type",
        "IsCallable",
        "IsConstructor",
        "IsPropertyKey",
        "ToPrimitive",
        "ToBoolean",
        "ToNumeric",
        "ToNumber",
        "ToBigInt",
        "ToString",
        "ToObject",
        "ToPropertyKey",
        "ToIntegerOrInfinity",
        "ToLength",
        "ToIndex",
        "IntegerIndexedConversion",
        "SameValue",
        "SameValueZero",
        "StrictEqualityComparison",
        "IsLooselyEqual",
        "IsLessThan",
        "Get",
        "GetV",
        "Set",
        "HasProperty",
        "HasOwnProperty",
        "DeletePropertyOrThrow",
        "CreateDataProperty",
        "CreateDataPropertyOrThrow",
        "CopyDataProperties",
        "DefinePropertyOrThrow",
        "ToPropertyDescriptor",
        "FromPropertyDescriptor",
        "GetMethod",
        "Call",
        "Construct",
        "OrdinaryCreateFromConstructor",
        "SpeciesConstructor",
        "ArraySpeciesCreate",
        "GetIterator",
        "IteratorStep",
        "IteratorValue",
        "IteratorClose",
        "AsyncIteratorClose",
        "Completion",
        "UpdateEmpty",
    ];

    #[test]
    fn operations_catalog_covers_t04_required_operations() {
        for required in REQUIRED_T04_OPERATIONS {
            assert!(
                find_spec_operation(required).is_some(),
                "missing T04 operation catalog entry: {required}"
            );
        }
    }

    #[test]
    fn operations_catalog_names_are_unique() {
        let mut names = BTreeSet::new();
        for entry in SPEC_OPERATION_CATALOG {
            assert!(
                names.insert(entry.name),
                "duplicate operation {}",
                entry.name
            );
        }
    }

    #[test]
    fn operations_catalog_tracks_every_gap_or_shared_lowering() {
        let mut lowered = BTreeSet::new();
        let mut rust_modeled = BTreeSet::new();
        for entry in SPEC_OPERATION_CATALOG {
            assert!(!entry.normal_result.is_empty(), "{} result", entry.name);
            match entry.lowering_status {
                OperationLoweringStatus::SharedWasmEmitter => {
                    lowered.insert(entry.name);
                }
                OperationLoweringStatus::SharedRustModel => {
                    rust_modeled.insert(entry.name);
                }
                OperationLoweringStatus::TrackedGap(task) => {
                    assert_eq!(task, "T04", "{} owner", entry.name);
                }
                OperationLoweringStatus::CatalogOnly => {
                    panic!(
                        "{} must be explicitly tracked before implementation",
                        entry.name
                    );
                }
            }
        }
        assert!(lowered.contains("IsCallable"));
        assert!(lowered.contains("IsPropertyKey"));
        assert!(lowered.contains("ToPrimitive"));
        assert!(lowered.contains("ToBoolean"));
        assert!(lowered.contains("ToNumeric"));
        assert!(lowered.contains("IsConstructor"));
        assert!(lowered.contains("ToNumber"));
        assert!(lowered.contains("ToBigInt"));
        assert!(lowered.contains("ToString"));
        assert!(lowered.contains("ToObject"));
        assert!(lowered.contains("ToPropertyKey"));
        assert!(lowered.contains("ToIntegerOrInfinity"));
        assert!(lowered.contains("ToLength"));
        assert!(lowered.contains("ToIndex"));
        assert!(lowered.contains("SameValue"));
        assert!(lowered.contains("SameValueZero"));
        assert!(lowered.contains("StrictEqualityComparison"));
        assert!(lowered.contains("IsLooselyEqual"));
        assert!(lowered.contains("Get"));
        assert!(lowered.contains("GetV"));
        assert!(lowered.contains("Set"));
        assert!(lowered.contains("HasProperty"));
        assert!(lowered.contains("HasOwnProperty"));
        assert!(lowered.contains("DeletePropertyOrThrow"));
        assert!(lowered.contains("CreateDataPropertyOrThrow"));
        assert!(lowered.contains("CopyDataProperties"));
        assert!(lowered.contains("GetMethod"));
        assert!(lowered.contains("Call"));
        assert!(lowered.contains("Construct"));
        assert!(rust_modeled.contains("Type"));
        assert!(rust_modeled.contains("IntegerIndexedConversion"));
        assert!(rust_modeled.contains("IsLessThan"));
        assert!(rust_modeled.contains("CreateDataProperty"));
        assert!(rust_modeled.contains("DefinePropertyOrThrow"));
        assert!(rust_modeled.contains("ToPropertyDescriptor"));
        assert!(rust_modeled.contains("FromPropertyDescriptor"));
        assert!(rust_modeled.contains("OrdinaryCreateFromConstructor"));
        assert!(rust_modeled.contains("SpeciesConstructor"));
        assert!(rust_modeled.contains("ArraySpeciesCreate"));
        assert!(rust_modeled.contains("GetIterator"));
        assert!(rust_modeled.contains("IteratorStep"));
        assert!(rust_modeled.contains("IteratorValue"));
        assert!(rust_modeled.contains("IteratorClose"));
        assert!(rust_modeled.contains("AsyncIteratorClose"));
        assert!(rust_modeled.contains("Completion"));
        assert!(rust_modeled.contains("UpdateEmpty"));
    }

    #[test]
    fn operations_iterator_record_tracks_kind_and_done_state() {
        let mut sync = IteratorRecordIr::sync("iterator", "next");
        assert_eq!(sync.kind, IteratorRecordKind::Sync);
        assert_eq!(sync.iterator, "iterator");
        assert_eq!(sync.next_method, "next");
        assert!(!sync.is_done());

        sync.mark_done();
        assert!(sync.is_done());

        let async_record = IteratorRecordIr::async_("async_iterator", "async_next");
        assert_eq!(async_record.kind, IteratorRecordKind::Async);
        assert!(!async_record.is_done());
    }

    #[test]
    fn operations_abstract_relational_comparison_result_preserves_undefined() {
        assert_eq!(
            AbstractRelationalComparisonResult::from_bool(true),
            AbstractRelationalComparisonResult::True
        );
        assert_eq!(
            AbstractRelationalComparisonResult::from_bool(false),
            AbstractRelationalComparisonResult::False
        );
        assert_eq!(
            AbstractRelationalComparisonResult::True.as_bool(),
            Some(true)
        );
        assert_eq!(
            AbstractRelationalComparisonResult::False.as_bool(),
            Some(false)
        );
        assert_eq!(
            AbstractRelationalComparisonResult::Undefined.as_bool(),
            None
        );
    }

    #[test]
    fn operations_integer_indexed_conversion_tracks_element_conversion_contract() {
        let number = IntegerIndexedConversionIr::new(IntegerIndexedElementType::Int16);
        let clamped = IntegerIndexedConversionIr::new(IntegerIndexedElementType::Uint8Clamped);
        let bigint = IntegerIndexedConversionIr::new(IntegerIndexedElementType::BigUint64);

        assert_eq!(number.element_type.name(), "Int16");
        assert!(number.requires_number_input());
        assert!(!number.requires_bigint_input());
        assert!(!number.clamps_result());

        assert!(clamped.requires_number_input());
        assert!(clamped.clamps_result());

        assert!(bigint.requires_bigint_input());
        assert!(!bigint.requires_number_input());
        assert!(!bigint.clamps_result());
    }

    #[test]
    fn operations_iterator_record_map_value_preserves_metadata() {
        let mut record = IteratorRecordIr::sync("iter".to_string(), "next".to_string());
        record.mark_done();

        let lengths = record.map_value(|value| value.len());

        assert_eq!(lengths.iterator, 4);
        assert_eq!(lengths.next_method, 4);
        assert_eq!(lengths.kind, IteratorRecordKind::Sync);
        assert!(lengths.is_done());
    }

    #[test]
    fn operations_property_descriptor_reports_descriptor_kind_and_flags() {
        let data = PropertyDescriptorIr::data(Some("value"), Some(true), Some(false), Some(true));
        assert_eq!(data.kind(), PropertyDescriptorKind::Data);
        assert!(data.is_data_descriptor());
        assert!(!data.is_accessor_descriptor());
        assert!(!data.is_generic_descriptor());
        assert_eq!(data.enumerable(), Some(false));
        assert_eq!(data.configurable(), Some(true));

        let accessor = PropertyDescriptorIr::accessor(Some("get"), None::<&str>, None, None);
        assert_eq!(accessor.kind(), PropertyDescriptorKind::Accessor);
        assert!(accessor.is_accessor_descriptor());

        let generic = PropertyDescriptorIr::<&str>::generic(Some(true), None);
        assert_eq!(generic.kind(), PropertyDescriptorKind::Generic);
        assert!(generic.is_generic_descriptor());
        assert_eq!(generic.enumerable(), Some(true));
        assert_eq!(generic.configurable(), None);
    }

    #[test]
    fn operations_property_descriptor_complete_applies_spec_defaults() {
        let data = PropertyDescriptorIr::data(None, None, Some(true), None).complete("undefined");
        assert_eq!(
            data,
            PropertyDescriptorIr::Data {
                value: Some("undefined"),
                writable: Some(false),
                enumerable: Some(true),
                configurable: Some(false),
            }
        );

        let generic = PropertyDescriptorIr::generic(None, Some(true)).complete("undefined");
        assert_eq!(
            generic,
            PropertyDescriptorIr::Data {
                value: Some("undefined"),
                writable: Some(false),
                enumerable: Some(false),
                configurable: Some(true),
            }
        );

        let accessor =
            PropertyDescriptorIr::accessor(Some("getter"), None, None, None).complete("undefined");
        assert_eq!(
            accessor,
            PropertyDescriptorIr::Accessor {
                get: Some("getter"),
                set: Some("undefined"),
                enumerable: Some(false),
                configurable: Some(false),
            }
        );
    }

    #[test]
    fn operations_property_descriptor_map_value_preserves_attributes() {
        let descriptor = PropertyDescriptorIr::accessor(
            Some("getter".to_string()),
            Some("setter".to_string()),
            Some(true),
            None,
        );
        let lengths = descriptor.map_value(|value| value.len());

        assert_eq!(
            lengths,
            PropertyDescriptorIr::Accessor {
                get: Some(6),
                set: Some(6),
                enumerable: Some(true),
                configurable: None,
            }
        );
    }

    #[test]
    fn operations_create_data_property_record_preserves_target_key_and_value() {
        let record =
            CreateDataPropertyIr::new("target".to_string(), "key".to_string(), "value".to_string());
        let lengths = record.map_value(|value| value.len());

        assert_eq!(
            lengths,
            CreateDataPropertyIr {
                target: 6,
                key: 3,
                value: 5,
            }
        );
    }

    #[test]
    fn operations_define_property_record_preserves_descriptor_attributes() {
        let descriptor =
            PropertyDescriptorIr::data(Some("value".to_string()), Some(true), None, Some(false));
        let record = DefinePropertyIr::new("target".to_string(), "key".to_string(), descriptor);
        let lengths = record.map_value(|value| value.len());

        assert_eq!(lengths.target, 6);
        assert_eq!(lengths.key, 3);
        assert_eq!(
            lengths.descriptor,
            PropertyDescriptorIr::Data {
                value: Some(5),
                writable: Some(true),
                enumerable: None,
                configurable: Some(false),
            }
        );
    }

    #[test]
    fn operations_constructor_species_records_preserve_defaults() {
        let ordinary =
            OrdinaryCreateFromConstructorIr::new("ctor".to_string(), "%Array.prototype%");
        let ordinary_lengths = ordinary.map_value(|value| value.len());
        assert_eq!(ordinary_lengths.constructor, 4);
        assert_eq!(
            ordinary_lengths.intrinsic_default_proto,
            "%Array.prototype%"
        );

        let species = SpeciesConstructorIr::new("object".to_string(), "default".to_string())
            .map_value(|value| value.len());
        assert_eq!(
            species,
            SpeciesConstructorIr {
                object: 6,
                default_constructor: 7,
            }
        );

        let array_species = ArraySpeciesCreateIr::new("array".to_string(), "length".to_string())
            .map_value(|value| value.len());
        assert_eq!(
            array_species,
            ArraySpeciesCreateIr {
                original_array: 5,
                length: 6,
            }
        );
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
    fn operations_catalog_marks_abrupt_capable_operations() {
        for name in ["ToPrimitive", "Get", "Call", "IteratorClose", "Completion"] {
            let entry = find_spec_operation(name).expect("operation should exist");
            assert!(
                !entry.abrupt.is_empty(),
                "{name} must document possible abrupt completions"
            );
        }
        let same_value = find_spec_operation("SameValue").expect("SameValue should exist");
        assert!(same_value.abrupt.is_empty());
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
