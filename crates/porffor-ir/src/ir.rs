use std::collections::{BTreeMap, BTreeSet};

use num_bigint::{BigInt, BigUint, Sign};
use num_traits::{One, ToPrimitive, Zero};
use porffor_front::ParseGoal;

use crate::{
    ArithmeticBinaryOp, BindingMode, BitwiseBinaryOp, CallableToStringRepresentation,
    CompletionRecordIr, EcmaLanguageType, EqualityBinaryOp, HostBuiltinId, IrDiagnostic,
    IrDiagnosticKind, LogicalBinaryOp, LoweringStage, NumericUpdateOp, RegExpProgram,
    RelationalBinaryOp, SpecOperationIr, StandardBuiltinId, ToPrimitiveHint, UnaryNumericOp,
    UpdateReturnMode, GLOBAL_THIS_NAME,
};

pub type FunctionId = String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StaticRegExpCompilation {
    Program(RegExpProgram),
    InvalidSyntax { message: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PrivateNameId {
    class_scope: u32,
    name_ordinal: u32,
}

impl PrivateNameId {
    pub(crate) const fn new(class_scope: u32, name_ordinal: u32) -> Self {
        Self {
            class_scope,
            name_ordinal,
        }
    }

    pub const fn name_ordinal(self) -> u32 {
        self.name_ordinal
    }

    pub const fn class_scope(self) -> u32 {
        self.class_scope
    }
}

impl std::fmt::Display for PrivateNameId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}.{}", self.class_scope, self.name_ordinal)
    }
}

pub fn private_data_key(private_name_id: PrivateNameId) -> String {
    format!("$class.private.data.{private_name_id}")
}

pub fn private_brand_key(private_name_id: PrivateNameId) -> String {
    format!("$class.private.brand.{private_name_id}")
}

pub fn private_getter_key(private_name_id: PrivateNameId) -> String {
    format!("$class.private.getter.{private_name_id}")
}

pub fn private_setter_key(private_name_id: PrivateNameId) -> String {
    format!("$class.private.setter.{private_name_id}")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    Undefined,
    Null,
    Boolean,
    Number,
    String,
    Symbol,
    Object,
    Array,
    Function,
    Arguments,
    BigInt,
    Dynamic,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BigIntLiteralIr {
    pub decimal: String,
    pub low_bits: u64,
    pub requires_arbitrary_precision_storage: bool,
}

impl BigIntLiteralIr {
    pub fn from_bigint(value: BigInt) -> Self {
        let decimal = value.to_string();
        let low_bits = Self::low_bits(&value);
        let requires_arbitrary_precision_storage = value.bits() > 63;
        Self {
            decimal,
            low_bits,
            requires_arbitrary_precision_storage,
        }
    }

    pub fn from_i64(value: i64) -> Self {
        Self::from_bigint(BigInt::from(value))
    }

    pub fn from_u64_payload(bits: u64) -> Self {
        Self {
            decimal: bits.to_string(),
            low_bits: bits,
            requires_arbitrary_precision_storage: bits > i64::MAX as u64,
        }
    }

    pub fn to_bigint(&self) -> BigInt {
        self.decimal
            .parse::<BigInt>()
            .expect("BigIntLiteralIr decimal should parse")
    }

    pub fn wrapping_payload(&self) -> u64 {
        self.low_bits
    }

    pub fn negated(&self) -> Self {
        Self::from_bigint(-self.to_bigint())
    }

    pub fn added(&self, rhs: &Self) -> Self {
        Self::from_bigint(self.to_bigint() + rhs.to_bigint())
    }

    pub fn pow_u32(&self, exponent: u32) -> Self {
        Self::from_bigint(self.to_bigint().pow(exponent))
    }

    fn low_bits(value: &BigInt) -> u64 {
        let (_, magnitude) = value.to_bytes_le();
        let magnitude = BigUint::from_bytes_le(&magnitude);
        let low_bits_mask = (BigUint::one() << 64_u32) - BigUint::one();
        let low_bits = (magnitude & low_bits_mask).to_u64().unwrap_or(0);
        if value.sign() == Sign::Minus && !value.is_zero() {
            low_bits.wrapping_neg()
        } else {
            low_bits
        }
    }
}

impl ValueKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Undefined => "undefined",
            Self::Null => "null",
            Self::Boolean => "boolean",
            Self::Number => "number",
            Self::String => "string",
            Self::Symbol => "symbol",
            Self::Object => "object",
            Self::Array => "array",
            Self::Function => "function",
            Self::Arguments => "arguments",
            Self::BigInt => "bigint",
            Self::Dynamic => "dynamic",
        }
    }

    pub const fn known_ecmascript_type(self) -> Option<EcmaLanguageType> {
        match self {
            Self::Undefined => Some(EcmaLanguageType::Undefined),
            Self::Null => Some(EcmaLanguageType::Null),
            Self::Boolean => Some(EcmaLanguageType::Boolean),
            Self::Number => Some(EcmaLanguageType::Number),
            Self::String => Some(EcmaLanguageType::String),
            Self::Symbol => Some(EcmaLanguageType::Symbol),
            Self::BigInt => Some(EcmaLanguageType::BigInt),
            Self::Object | Self::Array | Self::Function | Self::Arguments => {
                Some(EcmaLanguageType::Object)
            }
            Self::Dynamic => None,
        }
    }

    pub const fn tag(self) -> i32 {
        match self {
            Self::Undefined => 0,
            Self::Null => 1,
            Self::Boolean => 2,
            Self::Number => 3,
            Self::String => 4,
            Self::Symbol => 5,
            Self::Object => 6,
            Self::Array => 7,
            Self::Function => 8,
            Self::Arguments => 9,
            Self::BigInt => 10,
            Self::Dynamic => 11,
        }
    }

    pub const fn from_tag(tag: i32) -> Option<Self> {
        match tag {
            0 => Some(Self::Undefined),
            1 => Some(Self::Null),
            2 => Some(Self::Boolean),
            3 => Some(Self::Number),
            4 => Some(Self::String),
            5 => Some(Self::Symbol),
            6 => Some(Self::Object),
            7 => Some(Self::Array),
            8 => Some(Self::Function),
            9 => Some(Self::Arguments),
            10 => Some(Self::BigInt),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KindSet(pub(crate) u16);

impl KindSet {
    pub const EMPTY: Self = Self(0);

    const UNDEFINED_BIT: u16 = 1 << 0;
    const NULL_BIT: u16 = 1 << 1;
    const BOOLEAN_BIT: u16 = 1 << 2;
    const NUMBER_BIT: u16 = 1 << 3;
    const STRING_BIT: u16 = 1 << 4;
    const SYMBOL_BIT: u16 = 1 << 5;
    const OBJECT_BIT: u16 = 1 << 6;
    const ARRAY_BIT: u16 = 1 << 7;
    const FUNCTION_BIT: u16 = 1 << 8;
    const ARGUMENTS_BIT: u16 = 1 << 9;
    const BIGINT_BIT: u16 = 1 << 10;

    pub const PRIMITIVE_ONLY: Self = Self(
        Self::UNDEFINED_BIT
            | Self::NULL_BIT
            | Self::BOOLEAN_BIT
            | Self::NUMBER_BIT
            | Self::STRING_BIT
            | Self::SYMBOL_BIT
            | Self::BIGINT_BIT,
    );

    pub const HEAP_COERCIBLE_ONLY: Self =
        Self(Self::OBJECT_BIT | Self::ARRAY_BIT | Self::ARGUMENTS_BIT);

    pub const PRIMITIVE_OR_HEAP_COERCIBLE: Self =
        Self(Self::PRIMITIVE_ONLY.0 | Self::HEAP_COERCIBLE_ONLY.0);
    pub const PROPERTY_KEY_COERCIBLE: Self =
        Self(Self::PRIMITIVE_OR_HEAP_COERCIBLE.0 | Self::FUNCTION_BIT);

    pub const NULLISH: Self = Self(Self::UNDEFINED_BIT | Self::NULL_BIT);

    pub const fn from_kind(kind: ValueKind) -> Self {
        match kind {
            ValueKind::Undefined => Self(Self::UNDEFINED_BIT),
            ValueKind::Null => Self(Self::NULL_BIT),
            ValueKind::Boolean => Self(Self::BOOLEAN_BIT),
            ValueKind::Number => Self(Self::NUMBER_BIT),
            ValueKind::String => Self(Self::STRING_BIT),
            ValueKind::Symbol => Self(Self::SYMBOL_BIT),
            ValueKind::Object => Self(Self::OBJECT_BIT),
            ValueKind::Array => Self(Self::ARRAY_BIT),
            ValueKind::Function => Self(Self::FUNCTION_BIT),
            ValueKind::Arguments => Self(Self::ARGUMENTS_BIT),
            ValueKind::BigInt => Self(Self::BIGINT_BIT),
            ValueKind::Dynamic => Self::all_runtime_tags(),
        }
    }

    pub const fn all_runtime_tags() -> Self {
        Self(
            Self::UNDEFINED_BIT
                | Self::NULL_BIT
                | Self::BOOLEAN_BIT
                | Self::NUMBER_BIT
                | Self::STRING_BIT
                | Self::SYMBOL_BIT
                | Self::OBJECT_BIT
                | Self::ARRAY_BIT
                | Self::FUNCTION_BIT
                | Self::ARGUMENTS_BIT
                | Self::BIGINT_BIT,
        )
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn contains(self, kind: ValueKind) -> bool {
        self.0 & Self::from_kind(kind).0 != 0
    }

    pub const fn is_singleton(self) -> bool {
        self.0 != 0 && (self.0 & (self.0 - 1)) == 0
    }

    pub const fn is_subset_of(self, other: Self) -> bool {
        self.0 & !other.0 == 0
    }

    pub const fn without(self, kind: ValueKind) -> Self {
        Self(self.0 & !Self::from_kind(kind).0)
    }

    pub const fn as_value_kind(self) -> ValueKind {
        if self.is_singleton() {
            if self.contains(ValueKind::Undefined) {
                ValueKind::Undefined
            } else if self.contains(ValueKind::Null) {
                ValueKind::Null
            } else if self.contains(ValueKind::Boolean) {
                ValueKind::Boolean
            } else if self.contains(ValueKind::Number) {
                ValueKind::Number
            } else if self.contains(ValueKind::String) {
                ValueKind::String
            } else if self.contains(ValueKind::Symbol) {
                ValueKind::Symbol
            } else if self.contains(ValueKind::Object) {
                ValueKind::Object
            } else if self.contains(ValueKind::Array) {
                ValueKind::Array
            } else if self.contains(ValueKind::Function) {
                ValueKind::Function
            } else if self.contains(ValueKind::Arguments) {
                ValueKind::Arguments
            } else if self.contains(ValueKind::BigInt) {
                ValueKind::BigInt
            } else {
                ValueKind::Dynamic
            }
        } else {
            ValueKind::Dynamic
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueInfo {
    pub kind: ValueKind,
    pub possible_kinds: KindSet,
    pub heap_shape: Option<Box<HeapShape>>,
    pub function_targets: BTreeSet<FunctionId>,
}

impl ValueInfo {
    pub const fn undefined() -> Self {
        Self {
            kind: ValueKind::Undefined,
            possible_kinds: KindSet::from_kind(ValueKind::Undefined),
            heap_shape: None,
            function_targets: BTreeSet::new(),
        }
    }

    pub const fn new(kind: ValueKind) -> Self {
        Self {
            kind,
            possible_kinds: KindSet::from_kind(kind),
            heap_shape: None,
            function_targets: BTreeSet::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeapShape {
    Object(ObjectShape),
    Array(ArrayShape),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoxedPrimitiveKind {
    Number,
    String,
    Boolean,
    Symbol,
    BigInt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectAccessorShape {
    pub function_id: FunctionId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectShapeProperty {
    Data(ValueInfo),
    Accessor {
        getter: Option<ObjectAccessorShape>,
        setter: Option<ObjectAccessorShape>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ObjectShape {
    pub prototype: Option<Box<HeapShape>>,
    pub properties: BTreeMap<String, ObjectShapeProperty>,
    pub private_brands: BTreeSet<PrivateNameId>,
    pub boxed_primitive: Option<Box<ValueInfo>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ArrayShape {
    pub prototype: Option<Box<HeapShape>>,
    pub properties: BTreeMap<String, ObjectShapeProperty>,
    pub elements: Vec<ValueInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionFlavor {
    Ordinary,
    Arrow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassFunctionKind {
    None,
    Constructor,
    Method,
    Getter,
    Setter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassElementExecutionKind {
    None,
    InstanceFieldInitializer,
    StaticFieldInitializer,
    StaticBlock,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectPropertyIr {
    PrototypeSetter {
        value: TypedExpr,
    },
    Data {
        key: String,
        value: TypedExpr,
        is_shorthand: bool,
    },
    NonEnumerableData {
        key: String,
        value: TypedExpr,
    },
    ComputedData {
        key: TypedExpr,
        value: TypedExpr,
    },
    ComputedMethod {
        key: TypedExpr,
        function: TypedExpr,
    },
    ComputedGetter {
        key: TypedExpr,
        function: TypedExpr,
    },
    ComputedSetter {
        key: TypedExpr,
        function: TypedExpr,
    },
    Method {
        key: String,
        function: TypedExpr,
    },
    Getter {
        key: String,
        function: TypedExpr,
    },
    Setter {
        key: String,
        function: TypedExpr,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrivateElementKindIr {
    Field,
    Method,
    Accessor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassMethodPlacementIr {
    Instance,
    Static,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassHeritageKind {
    None,
    Constructable,
    Null,
}

impl Default for ClassHeritageKind {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassPublicMethodIr {
    pub key: PropertyKeyIr,
    pub function_id: FunctionId,
    pub placement: ClassMethodPlacementIr,
    pub kind: ClassFunctionKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassPrivateMethodIr {
    pub private_name_id: PrivateNameId,
    pub function_id: FunctionId,
    pub placement: ClassMethodPlacementIr,
    pub kind: ClassFunctionKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClassFieldKeyIr {
    Public(String),
    ComputedPublic(u32),
    Private(PrivateNameId),
}

impl ClassFieldKeyIr {
    pub fn static_name(&self) -> Option<&str> {
        match self {
            Self::Public(name) => Some(name),
            Self::ComputedPublic(_) | Self::Private(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassFieldInitIr {
    pub key: ClassFieldKeyIr,
    pub init_function_id: Option<FunctionId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassInstanceElementPlanIr {
    pub private_method_brands: Vec<PrivateNameId>,
    pub fields: Vec<ClassFieldInitIr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassStaticBlockIr {
    pub function_id: FunctionId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClassElementDefinitionIr {
    PublicMethod(ClassPublicMethodIr),
    PrivateMethod(ClassPrivateMethodIr),
    ComputedFieldKey { slot: u32, key: PropertyKeyIr },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClassStaticElementIr {
    Field(ClassFieldInitIr),
    Block(ClassStaticBlockIr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassElementPlanIr {
    pub definitions: Vec<ClassElementDefinitionIr>,
    pub static_elements: Vec<ClassStaticElementIr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassNameBindingIr {
    pub storage_name: String,
    pub environment: LexicalEnvironmentIr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassDefinitionIr {
    pub name: Option<String>,
    pub name_binding: Option<ClassNameBindingIr>,
    pub constructor_function_id: FunctionId,
    pub explicit_constructor: bool,
    pub heritage_kind: ClassHeritageKind,
    pub heritage: Option<Box<TypedExpr>>,
    pub element_plan: ClassElementPlanIr,
    pub private_name_ids: BTreeMap<String, PrivateNameId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropertyKeyIr {
    StaticString(String),
    StringExpr(Box<TypedExpr>),
    ArrayIndex(Box<TypedExpr>),
    ArrayLength,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DestructuringPropertyKeyIr {
    Static(String),
    Computed(TypedExpr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DestructuringTargetIr {
    Binding {
        mode: BindingMode,
        name: String,
    },
    AssignmentIdentifier {
        name: String,
        global: bool,
        implicit: bool,
        immutable: bool,
    },
    AssignmentProperty {
        target: TypedExpr,
        key: DestructuringPropertyKeyIr,
    },
    AssignmentPrivate {
        target: TypedExpr,
        private_name_id: PrivateNameId,
    },
    NestedArray(Box<ArrayDestructuringPatternIr>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArrayDestructuringElementIr {
    Elision,
    Target {
        target: DestructuringTargetIr,
        default: Option<TypedExpr>,
    },
    Rest {
        target: DestructuringTargetIr,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArrayDestructuringPatternIr {
    pub elements: Vec<ArrayDestructuringElementIr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectDestructuringPropertyIr {
    pub key: DestructuringPropertyKeyIr,
    pub target: DestructuringTargetIr,
    pub default: Option<TypedExpr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectDestructuringPatternIr {
    pub properties: Vec<ObjectDestructuringPropertyIr>,
    pub rest: Option<DestructuringTargetIr>,
}

impl ObjectDestructuringPatternIr {
    pub fn visit_expressions(&self, visit: &mut impl FnMut(&TypedExpr)) {
        for property in &self.properties {
            if let DestructuringPropertyKeyIr::Computed(key) = &property.key {
                visit(key);
            }
            visit_destructuring_target_expressions(&property.target, visit);
            if let Some(default) = &property.default {
                visit(default);
            }
        }
        if let Some(rest) = &self.rest {
            visit_destructuring_target_expressions(rest, visit);
        }
    }
}

fn visit_destructuring_target_expressions(
    target: &DestructuringTargetIr,
    visit: &mut impl FnMut(&TypedExpr),
) {
    match target {
        DestructuringTargetIr::AssignmentProperty { target, key } => {
            visit(target);
            if let DestructuringPropertyKeyIr::Computed(key) = key {
                visit(key);
            }
        }
        DestructuringTargetIr::AssignmentPrivate { target, .. } => visit(target),
        DestructuringTargetIr::NestedArray(pattern) => pattern.visit_expressions(visit),
        DestructuringTargetIr::Binding { .. }
        | DestructuringTargetIr::AssignmentIdentifier { .. } => {}
    }
}

impl ArrayDestructuringPatternIr {
    pub fn visit_expressions(&self, visit: &mut impl FnMut(&TypedExpr)) {
        for element in &self.elements {
            let (target, default) = match element {
                ArrayDestructuringElementIr::Elision => continue,
                ArrayDestructuringElementIr::Target { target, default } => {
                    (target, default.as_ref())
                }
                ArrayDestructuringElementIr::Rest { target } => (target, None),
            };
            visit_destructuring_target_expressions(target, visit);
            if let Some(default) = default {
                visit(default);
            }
        }
    }

    pub fn visit_bindings(&self, visit: &mut impl FnMut(BindingMode, &str)) {
        for element in &self.elements {
            let target = match element {
                ArrayDestructuringElementIr::Elision => continue,
                ArrayDestructuringElementIr::Target { target, .. }
                | ArrayDestructuringElementIr::Rest { target } => target,
            };
            match target {
                DestructuringTargetIr::Binding { mode, name } => visit(*mode, name),
                DestructuringTargetIr::NestedArray(pattern) => {
                    pattern.visit_bindings(visit);
                }
                DestructuringTargetIr::AssignmentIdentifier { .. }
                | DestructuringTargetIr::AssignmentProperty { .. }
                | DestructuringTargetIr::AssignmentPrivate { .. } => {}
            }
        }
    }
}

impl PropertyKeyIr {
    pub fn static_name(&self) -> Option<&str> {
        match self {
            Self::StaticString(name) => Some(name),
            Self::ArrayLength => Some("length"),
            Self::StringExpr(_) | Self::ArrayIndex(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedExpr {
    pub kind: ValueKind,
    pub possible_kinds: KindSet,
    pub heap_shape: Option<Box<HeapShape>>,
    pub function_targets: BTreeSet<FunctionId>,
    pub expr: ExprIr,
}

impl TypedExpr {
    pub const fn undefined() -> Self {
        Self {
            kind: ValueKind::Undefined,
            possible_kinds: KindSet::from_kind(ValueKind::Undefined),
            heap_shape: None,
            function_targets: BTreeSet::new(),
            expr: ExprIr::Undefined,
        }
    }

    pub fn from_info(info: ValueInfo, expr: ExprIr) -> Self {
        Self {
            kind: info.kind,
            possible_kinds: info.possible_kinds,
            heap_shape: info.heap_shape,
            function_targets: info.function_targets,
            expr,
        }
    }

    pub fn value_info(&self) -> ValueInfo {
        ValueInfo {
            kind: self.kind,
            possible_kinds: self.possible_kinds,
            heap_shape: self.heap_shape.clone(),
            function_targets: self.function_targets.clone(),
        }
    }

    pub fn spec_is_callable(argument: TypedExpr) -> Self {
        Self::from_info(
            ValueInfo::new(ValueKind::Boolean),
            ExprIr::SpecOperation {
                operation: SpecOperationIr::IsCallable,
                operands: vec![argument],
            },
        )
    }

    pub fn spec_is_constructor(argument: TypedExpr) -> Self {
        Self::from_info(
            ValueInfo::new(ValueKind::Boolean),
            ExprIr::SpecOperation {
                operation: SpecOperationIr::IsConstructor,
                operands: vec![argument],
            },
        )
    }

    pub fn spec_is_property_key(argument: TypedExpr) -> Self {
        Self::from_info(
            ValueInfo::new(ValueKind::Boolean),
            ExprIr::SpecOperation {
                operation: SpecOperationIr::IsPropertyKey,
                operands: vec![argument],
            },
        )
    }

    pub fn spec_to_boolean(argument: TypedExpr) -> Self {
        Self::from_info(
            ValueInfo::new(ValueKind::Boolean),
            ExprIr::SpecOperation {
                operation: SpecOperationIr::ToBoolean,
                operands: vec![argument],
            },
        )
    }

    pub fn spec_to_primitive(argument: TypedExpr, hint: ToPrimitiveHint) -> Self {
        Self::from_info(
            ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::PRIMITIVE_ONLY,
                heap_shape: None,
                function_targets: BTreeSet::new(),
            },
            ExprIr::SpecOperation {
                operation: SpecOperationIr::ToPrimitive(hint),
                operands: vec![argument],
            },
        )
    }

    pub fn spec_to_numeric(argument: TypedExpr) -> Self {
        Self::from_info(
            ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::from_kind(ValueKind::Number)
                    .union(KindSet::from_kind(ValueKind::BigInt)),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            },
            ExprIr::SpecOperation {
                operation: SpecOperationIr::ToNumeric,
                operands: vec![argument],
            },
        )
    }

    pub fn spec_to_number(argument: TypedExpr) -> Self {
        Self::from_info(
            ValueInfo::new(ValueKind::Number),
            ExprIr::SpecOperation {
                operation: SpecOperationIr::ToNumber,
                operands: vec![argument],
            },
        )
    }

    pub fn spec_to_bigint(argument: TypedExpr) -> Self {
        Self::from_info(
            ValueInfo::new(ValueKind::BigInt),
            ExprIr::SpecOperation {
                operation: SpecOperationIr::ToBigInt,
                operands: vec![argument],
            },
        )
    }

    pub fn spec_to_string(argument: TypedExpr) -> Self {
        Self::from_info(
            ValueInfo::new(ValueKind::String),
            ExprIr::SpecOperation {
                operation: SpecOperationIr::ToString,
                operands: vec![argument],
            },
        )
    }

    pub fn spec_to_object(argument: TypedExpr) -> Self {
        let possible_kinds = KindSet::from_kind(ValueKind::Object)
            .union(KindSet::from_kind(ValueKind::Array))
            .union(KindSet::from_kind(ValueKind::Function))
            .union(KindSet::from_kind(ValueKind::Arguments));
        Self::from_info(
            ValueInfo {
                kind: possible_kinds.as_value_kind(),
                possible_kinds,
                heap_shape: None,
                function_targets: BTreeSet::new(),
            },
            ExprIr::SpecOperation {
                operation: SpecOperationIr::ToObject,
                operands: vec![argument],
            },
        )
    }

    pub fn spec_to_property_key(argument: TypedExpr) -> Self {
        Self::from_info(
            ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::from_kind(ValueKind::String)
                    .union(KindSet::from_kind(ValueKind::Symbol)),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            },
            ExprIr::SpecOperation {
                operation: SpecOperationIr::ToPropertyKey,
                operands: vec![argument],
            },
        )
    }

    pub fn spec_to_integer_or_infinity(argument: TypedExpr) -> Self {
        Self::from_info(
            ValueInfo::new(ValueKind::Number),
            ExprIr::SpecOperation {
                operation: SpecOperationIr::ToIntegerOrInfinity,
                operands: vec![argument],
            },
        )
    }

    pub fn spec_to_length(argument: TypedExpr) -> Self {
        Self::from_info(
            ValueInfo::new(ValueKind::Number),
            ExprIr::SpecOperation {
                operation: SpecOperationIr::ToLength,
                operands: vec![argument],
            },
        )
    }

    pub fn spec_to_index(argument: TypedExpr) -> Self {
        Self::from_info(
            ValueInfo::new(ValueKind::Number),
            ExprIr::SpecOperation {
                operation: SpecOperationIr::ToIndex,
                operands: vec![argument],
            },
        )
    }

    pub fn spec_same_value(lhs: TypedExpr, rhs: TypedExpr) -> Self {
        Self::from_info(
            ValueInfo::new(ValueKind::Boolean),
            ExprIr::SpecOperation {
                operation: SpecOperationIr::SameValue,
                operands: vec![lhs, rhs],
            },
        )
    }

    pub fn spec_same_value_zero(lhs: TypedExpr, rhs: TypedExpr) -> Self {
        Self::from_info(
            ValueInfo::new(ValueKind::Boolean),
            ExprIr::SpecOperation {
                operation: SpecOperationIr::SameValueZero,
                operands: vec![lhs, rhs],
            },
        )
    }

    pub fn spec_strict_equality_comparison(lhs: TypedExpr, rhs: TypedExpr) -> Self {
        Self::from_info(
            ValueInfo::new(ValueKind::Boolean),
            ExprIr::SpecOperation {
                operation: SpecOperationIr::StrictEqualityComparison,
                operands: vec![lhs, rhs],
            },
        )
    }

    pub fn spec_is_loosely_equal(lhs: TypedExpr, rhs: TypedExpr) -> Self {
        Self::from_info(
            ValueInfo::new(ValueKind::Boolean),
            ExprIr::SpecOperation {
                operation: SpecOperationIr::IsLooselyEqual,
                operands: vec![lhs, rhs],
            },
        )
    }

    pub fn spec_get_v(target: TypedExpr, property_key: TypedExpr) -> Self {
        Self::spec_get_v_with_info(
            ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::all_runtime_tags(),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            },
            target,
            property_key,
        )
    }

    pub fn spec_get(target: TypedExpr, property_key: TypedExpr) -> Self {
        Self::spec_get_with_info(
            ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::all_runtime_tags(),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            },
            target,
            property_key,
        )
    }

    pub fn spec_get_with_info(info: ValueInfo, target: TypedExpr, property_key: TypedExpr) -> Self {
        Self::from_info(
            info,
            ExprIr::SpecOperation {
                operation: SpecOperationIr::Get,
                operands: vec![target, property_key],
            },
        )
    }

    pub fn spec_get_v_with_info(
        info: ValueInfo,
        target: TypedExpr,
        property_key: TypedExpr,
    ) -> Self {
        Self::from_info(
            info,
            ExprIr::SpecOperation {
                operation: SpecOperationIr::GetV,
                operands: vec![target, property_key],
            },
        )
    }

    pub fn spec_has_property(target: TypedExpr, property_key: TypedExpr) -> Self {
        Self::from_info(
            ValueInfo::new(ValueKind::Boolean),
            ExprIr::SpecOperation {
                operation: SpecOperationIr::HasProperty,
                operands: vec![target, property_key],
            },
        )
    }

    pub fn spec_has_own_property(target: TypedExpr, property_key: TypedExpr) -> Self {
        Self::from_info(
            ValueInfo::new(ValueKind::Boolean),
            ExprIr::SpecOperation {
                operation: SpecOperationIr::HasOwnProperty,
                operands: vec![target, property_key],
            },
        )
    }

    pub fn spec_set(target: TypedExpr, property_key: TypedExpr, value: TypedExpr) -> Self {
        Self::from_info(
            ValueInfo::new(ValueKind::Boolean),
            ExprIr::SpecOperation {
                operation: SpecOperationIr::Set,
                operands: vec![target, property_key, value],
            },
        )
    }

    pub fn spec_delete_property_or_throw(target: TypedExpr, property_key: TypedExpr) -> Self {
        Self::from_info(
            ValueInfo::new(ValueKind::Boolean),
            ExprIr::SpecOperation {
                operation: SpecOperationIr::DeletePropertyOrThrow,
                operands: vec![target, property_key],
            },
        )
    }

    pub fn spec_get_method(target: TypedExpr, property_key: TypedExpr) -> Self {
        Self::from_info(
            ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::from_kind(ValueKind::Undefined)
                    .union(KindSet::from_kind(ValueKind::Function)),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            },
            ExprIr::SpecOperation {
                operation: SpecOperationIr::GetMethod,
                operands: vec![target, property_key],
            },
        )
    }

    pub fn spec_call(callee: TypedExpr, this_arg: TypedExpr, args: Vec<TypedExpr>) -> Self {
        let mut operands = Vec::with_capacity(args.len() + 2);
        operands.push(callee);
        operands.push(this_arg);
        operands.extend(args);
        Self::from_info(
            ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::all_runtime_tags(),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            },
            ExprIr::SpecOperation {
                operation: SpecOperationIr::Call,
                operands,
            },
        )
    }

    pub fn spec_construct(callee: TypedExpr, args: Vec<TypedExpr>) -> Self {
        let mut operands = Vec::with_capacity(args.len() + 1);
        operands.push(callee);
        operands.extend(args);
        Self::from_info(
            ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::from_kind(ValueKind::Object)
                    .union(KindSet::from_kind(ValueKind::Array))
                    .union(KindSet::from_kind(ValueKind::Function))
                    .union(KindSet::from_kind(ValueKind::Arguments)),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            },
            ExprIr::SpecOperation {
                operation: SpecOperationIr::Construct,
                operands,
            },
        )
    }

    pub fn spec_create_data_property_or_throw(
        target: TypedExpr,
        property_key: TypedExpr,
        value: TypedExpr,
    ) -> Self {
        Self::from_info(
            ValueInfo::new(ValueKind::Undefined),
            ExprIr::SpecOperation {
                operation: SpecOperationIr::CreateDataPropertyOrThrow,
                operands: vec![target, property_key, value],
            },
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExprIr {
    Undefined,
    ArrayHole,
    Null,
    Boolean(bool),
    Number(u64),
    BigInt(BigIntLiteralIr),
    Symbol {
        /// Description operand for `Symbol(desc)`. `None` for `Symbol()` /
        /// `Symbol(undefined)` (spec `[[Description]]` = undefined). When
        /// present, the operand has already been coerced via ToString during
        /// lowering.
        description: Option<Box<TypedExpr>>,
    },
    String(String),
    /// An intrinsic RegExp literal creation, independent of the mutable global
    /// `RegExp` constructor.
    RegExpLiteral {
        source: String,
        flags: String,
        program: Option<RegExpProgram>,
    },
    FunctionValue(FunctionId),
    This,
    Arguments,
    ObjectLiteral(Vec<ObjectPropertyIr>),
    ArrayLiteral(Vec<TypedExpr>),
    Identifier(String),
    GlobalPropertyRead {
        name: String,
    },
    AssignIdentifier {
        name: String,
        value: Box<TypedExpr>,
    },
    GlobalPropertyWrite {
        name: String,
        value: Box<TypedExpr>,
        implicit: bool,
    },
    PropertyRead {
        target: Box<TypedExpr>,
        key: PropertyKeyIr,
    },
    /// An ordered optional chain of property accesses and calls.
    ///
    /// Keys and call arguments remain expressions in the chain so a backend
    /// can defer them until all preceding optional operations have succeeded.
    OptionalPropertyChain {
        target: Box<TypedExpr>,
        chain: Vec<OptionalChainOperationIr>,
    },
    PropertyWrite {
        target: Box<TypedExpr>,
        key: PropertyKeyIr,
        value: Box<TypedExpr>,
    },
    PropertyUpdate {
        target: Box<TypedExpr>,
        key: PropertyKeyIr,
        op: NumericUpdateOp,
        return_mode: UpdateReturnMode,
        value_kind: ValueKind,
    },
    PropertyCompoundAssign {
        target: Box<TypedExpr>,
        key: PropertyKeyIr,
        op: ArithmeticBinaryOp,
        value: Box<TypedExpr>,
    },
    UpdateIdentifier {
        name: String,
        op: NumericUpdateOp,
        return_mode: UpdateReturnMode,
        value_kind: ValueKind,
    },
    GlobalPropertyUpdate {
        name: String,
        op: NumericUpdateOp,
        return_mode: UpdateReturnMode,
        value_kind: ValueKind,
    },
    CompoundAssignIdentifier {
        name: String,
        op: ArithmeticBinaryOp,
        value: Box<TypedExpr>,
    },
    GlobalPropertyCompoundAssign {
        name: String,
        op: ArithmeticBinaryOp,
        value: Box<TypedExpr>,
    },
    UnaryNumber {
        op: UnaryNumericOp,
        expr: Box<TypedExpr>,
    },
    Void {
        expr: Box<TypedExpr>,
    },
    DeleteValue {
        expr: Box<TypedExpr>,
    },
    DeleteIdentifier {
        name: String,
        kind: DeleteIdentifierKindIr,
    },
    DeleteGlobalProperty {
        name: String,
    },
    DeleteProperty {
        target: Box<TypedExpr>,
        key: PropertyKeyIr,
        strict: bool,
    },
    TypeOf {
        expr: Box<TypedExpr>,
    },
    NewTarget,
    TypeOfUnresolvedIdentifier {
        name: String,
    },
    LogicalNot {
        expr: Box<TypedExpr>,
    },
    SpecOperation {
        operation: SpecOperationIr,
        operands: Vec<TypedExpr>,
    },
    BinaryNumber {
        op: ArithmeticBinaryOp,
        lhs: Box<TypedExpr>,
        rhs: Box<TypedExpr>,
    },
    CoerciveAdd {
        lhs: Box<TypedExpr>,
        rhs: Box<TypedExpr>,
    },
    CoerciveBinaryNumber {
        op: ArithmeticBinaryOp,
        lhs: Box<TypedExpr>,
        rhs: Box<TypedExpr>,
    },
    BitwiseNumber {
        op: BitwiseBinaryOp,
        lhs: Box<TypedExpr>,
        rhs: Box<TypedExpr>,
    },
    StringFromCharCode {
        code: Box<TypedExpr>,
    },
    StringCharCodeAt {
        target: Box<TypedExpr>,
        index: Box<TypedExpr>,
    },
    StringConcat {
        lhs: Box<TypedExpr>,
        rhs: Box<TypedExpr>,
    },
    TemplateObject(TemplateObjectIr),
    CompareNumber {
        op: RelationalBinaryOp,
        lhs: Box<TypedExpr>,
        rhs: Box<TypedExpr>,
    },
    CompareValue {
        op: RelationalBinaryOp,
        lhs: Box<TypedExpr>,
        rhs: Box<TypedExpr>,
    },
    StrictEquality {
        op: EqualityBinaryOp,
        lhs: Box<TypedExpr>,
        rhs: Box<TypedExpr>,
    },
    LooseEquality {
        op: EqualityBinaryOp,
        lhs: Box<TypedExpr>,
        rhs: Box<TypedExpr>,
    },
    LogicalShortCircuit {
        op: LogicalBinaryOp,
        lhs: Box<TypedExpr>,
        rhs: Box<TypedExpr>,
    },
    Conditional {
        condition: Box<TypedExpr>,
        then_expr: Box<TypedExpr>,
        else_expr: Box<TypedExpr>,
    },
    Comma {
        lhs: Box<TypedExpr>,
        rhs: Box<TypedExpr>,
    },
    MaterializeBinding {
        name: String,
        value: Box<TypedExpr>,
        body: Box<TypedExpr>,
    },
    ArrayDestructure {
        value: Box<TypedExpr>,
        pattern: ArrayDestructuringPatternIr,
        assignment: bool,
    },
    ObjectDestructure {
        value: Box<TypedExpr>,
        pattern: Box<ObjectDestructuringPatternIr>,
    },
    CallNamed {
        name: String,
        args: Vec<TypedExpr>,
    },
    SpreadArgument(Box<TypedExpr>),
    AssertSameValue {
        actual: Box<TypedExpr>,
        expected: Box<TypedExpr>,
        message: String,
    },
    RuntimeThrow {
        name: &'static str,
        message: &'static str,
    },
    CallIndirect {
        callee: Box<TypedExpr>,
        this_arg: Option<Box<TypedExpr>>,
        args: Vec<TypedExpr>,
        /// The static compilation outcome for a direct, constant `RegExp` call.
        /// This is metadata only: the ordinary call path still observes callee,
        /// receiver, and argument evaluation before applying the outcome.
        static_regexp_compilation: Option<StaticRegExpCompilation>,
    },
    JsonParseStaticReviver {
        value: JsonStaticValueIr,
        reviver: Box<TypedExpr>,
    },
    Construct {
        callee: Box<TypedExpr>,
        args: Vec<TypedExpr>,
        /// The static compilation outcome for a direct, constant `new RegExp` call.
        /// This is metadata only: construction still observes callee and argument
        /// evaluation before applying the outcome.
        static_regexp_compilation: Option<StaticRegExpCompilation>,
    },
    ClassDefinition(Box<ClassDefinitionIr>),
    CallMethod {
        receiver: Box<TypedExpr>,
        key: PropertyKeyIr,
        args: Vec<TypedExpr>,
    },
    SuperConstruct {
        args: Vec<TypedExpr>,
    },
    SuperPropertyRead {
        key: PropertyKeyIr,
    },
    SuperPropertyWrite {
        key: PropertyKeyIr,
        value: Box<TypedExpr>,
    },
    PrivateRead {
        target: Box<TypedExpr>,
        private_name_id: PrivateNameId,
    },
    PrivateWrite {
        target: Box<TypedExpr>,
        private_name_id: PrivateNameId,
        value: Box<TypedExpr>,
    },
    PrivateIn {
        private_name_id: PrivateNameId,
        rhs: Box<TypedExpr>,
    },
    InstanceOf {
        lhs: Box<TypedExpr>,
        rhs: Box<TypedExpr>,
    },
    In {
        lhs: Box<TypedExpr>,
        rhs: Box<TypedExpr>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateObjectIr {
    pub site_id: u64,
    pub cooked: Vec<Option<String>>,
    pub raw: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OptionalChainOperationIr {
    Property {
        key: PropertyKeyIr,
        /// Whether this operation was introduced by `?.` and therefore
        /// short-circuits the whole chain for a nullish receiver.
        shorted: bool,
    },
    PrivateProperty {
        private_name_id: PrivateNameId,
        /// Whether this operation was introduced by `?.` and therefore
        /// short-circuits the whole chain for a nullish receiver.
        shorted: bool,
    },
    Call {
        args: Vec<TypedExpr>,
        /// How the call's `this` value is recovered from the source Reference.
        receiver: OptionalChainCallReceiverIr,
        /// Whether this operation was introduced by `?.` and therefore
        /// short-circuits the whole chain for a nullish callee.
        shorted: bool,
        /// Whether a parenthesized/grouped expression ended the preceding
        /// short-circuit segment before this call, without discarding a
        /// preceding property Reference used as the call receiver.
        boundary_before: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionalChainCallReceiverIr {
    /// Use the base retained by the immediately preceding property operation,
    /// or `undefined` when the callee was not obtained from a property Reference.
    ReferenceOrUndefined,
    /// Use the surrounding function's current `this`, as required when calling
    /// a function obtained from a `super` property Reference.
    CurrentThis,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonStaticValueIr {
    Null { source: String },
    Boolean { value: bool, source: String },
    Number { bits: u64, source: String },
    String { value: String, source: String },
    Array(Vec<JsonStaticValueIr>),
    Object(Vec<(String, JsonStaticValueIr)>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeleteIdentifierKindIr {
    NonDeletable,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForLexicalInitIr {
    pub mode: BindingMode,
    pub name: String,
    pub init: TypedExpr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForInitIr {
    Lexical {
        mode: BindingMode,
        name: String,
        init: TypedExpr,
    },
    LexicalBlock(Vec<ForLexicalInitIr>),
    Var(Vec<VarDeclaratorIr>),
    Expression(TypedExpr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VarDeclaratorIr {
    pub name: String,
    pub init: Option<TypedExpr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwitchCaseIr {
    pub condition: Option<TypedExpr>,
    pub body: BlockIr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionParamIr {
    pub name: String,
    pub kind: ValueKind,
    pub default_init: Option<TypedExpr>,
    pub is_rest: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnedEnvBindingIr {
    pub name: String,
    pub slot: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexicalEnvironmentIr {
    pub bindings: Vec<OwnedEnvBindingIr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForLexicalEnvironmentIr {
    pub bindings: Vec<OwnedEnvBindingIr>,
    pub per_iteration_slots: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForInOfEnvironmentIr {
    pub tdz_environment: Option<LexicalEnvironmentIr>,
    pub iteration_environment: Option<LexicalEnvironmentIr>,
    pub tdz_binding_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturedBindingIr {
    pub name: String,
    pub source_name: String,
    pub mode: BindingMode,
    pub slot: u32,
    pub hops: u32,
}

/// Compiler-private per-invocation state for a derived constructor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivedConstructorActivationIr {
    pub owner_function_id: FunctionId,
    pub this_binding: String,
    pub this_status_binding: String,
    pub new_target_binding: String,
    pub active_function_binding: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionIr {
    pub id: FunctionId,
    pub name: String,
    pub to_string_representation: CallableToStringRepresentation,
    pub flavor: FunctionFlavor,
    pub strict: bool,
    pub callable: bool,
    pub constructable: bool,
    pub class_kind: ClassFunctionKind,
    pub class_element_execution_kind: ClassElementExecutionKind,
    pub class_heritage_kind: ClassHeritageKind,
    pub is_static_class_member: bool,
    pub is_derived_constructor: bool,
    pub is_synthetic_default_derived_constructor: bool,
    pub class_instance_element_plan: Option<ClassInstanceElementPlanIr>,
    pub super_constructor_target: Option<FunctionId>,
    pub uses_super: bool,
    pub this_before_super: bool,
    pub lexical_derived_activation: Option<DerivedConstructorActivationIr>,
    pub private_name_ids: BTreeMap<String, PrivateNameId>,
    /// Keeps the lexical private environment available to this function and
    /// any nested function values it creates at runtime.
    pub captures_private_environment: bool,
    pub is_nested: bool,
    pub is_expression: bool,
    pub is_named_expression: bool,
    pub captures_lexical_this: bool,
    pub captures_lexical_arguments: bool,
    pub params: Vec<FunctionParamIr>,
    pub body: BlockIr,
    pub return_kind: ValueKind,
    pub return_shape: Option<Box<HeapShape>>,
    pub return_targets: BTreeSet<FunctionId>,
    pub constructor_instance: ValueInfo,
    pub owned_env_bindings: Vec<OwnedEnvBindingIr>,
    pub captured_bindings: Vec<CapturedBindingIr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatementIr {
    Empty,
    Lexical {
        mode: BindingMode,
        name: String,
        init: TypedExpr,
    },
    AnnexBFunctionCopy {
        source_name: String,
        block_storage_name: String,
        variable_storage_name: String,
    },
    LexicalBlock(Vec<StatementIr>),
    Var(Vec<VarDeclaratorIr>),
    Expression(TypedExpr),
    Block(BlockIr),
    If {
        condition: TypedExpr,
        then_branch: Box<StatementIr>,
        else_branch: Option<Box<StatementIr>>,
    },
    While {
        condition: TypedExpr,
        body: Box<StatementIr>,
    },
    DoWhile {
        body: Box<StatementIr>,
        condition: TypedExpr,
    },
    For {
        init: Option<ForInitIr>,
        test: Option<TypedExpr>,
        update: Option<TypedExpr>,
        body: Box<StatementIr>,
        lexical_environment: Option<ForLexicalEnvironmentIr>,
    },
    ForOfArray {
        mode: BindingMode,
        name: String,
        iterable: TypedExpr,
        body: Box<StatementIr>,
        lexical_environment: Option<ForInOfEnvironmentIr>,
    },
    ForOfString {
        mode: BindingMode,
        name: String,
        iterable: TypedExpr,
        body: Box<StatementIr>,
        lexical_environment: Option<ForInOfEnvironmentIr>,
    },
    ForOfIterator {
        mode: BindingMode,
        name: String,
        iterable: TypedExpr,
        body: Box<StatementIr>,
        lexical_environment: Option<ForInOfEnvironmentIr>,
    },
    ForInArray {
        mode: BindingMode,
        name: String,
        target: TypedExpr,
        body: Box<StatementIr>,
        lexical_environment: Option<ForInOfEnvironmentIr>,
    },
    ForInString {
        mode: BindingMode,
        name: String,
        target: TypedExpr,
        body: Box<StatementIr>,
        lexical_environment: Option<ForInOfEnvironmentIr>,
    },
    ForInObject {
        mode: BindingMode,
        name: String,
        target: TypedExpr,
        body: Box<StatementIr>,
        lexical_environment: Option<ForInOfEnvironmentIr>,
    },
    Switch {
        discriminant: TypedExpr,
        lexical_environment: Option<LexicalEnvironmentIr>,
        lexical_declarations: Vec<StatementIr>,
        cases: Vec<SwitchCaseIr>,
    },
    Labelled {
        labels: Vec<String>,
        statement: Box<StatementIr>,
    },
    Debugger,
    Throw(TypedExpr),
    TryCatch {
        try_block: BlockIr,
        catch_name: String,
        catch_source_name: String,
        catch_parameter_environment: Option<LexicalEnvironmentIr>,
        catch_block: BlockIr,
    },
    TryFinally {
        try_block: BlockIr,
        finally_block: BlockIr,
    },
    TryCatchFinally {
        try_block: BlockIr,
        catch_name: String,
        catch_source_name: String,
        catch_parameter_environment: Option<LexicalEnvironmentIr>,
        catch_block: BlockIr,
        finally_block: BlockIr,
    },
    Return(TypedExpr),
    Break {
        label: Option<String>,
    },
    Continue {
        label: Option<String>,
    },
}

impl StatementIr {
    pub fn abrupt_completion_record(&self) -> Option<CompletionRecordIr<TypedExpr>> {
        match self {
            Self::Throw(value) => Some(CompletionRecordIr::throw(value.clone())),
            Self::Return(value) => Some(CompletionRecordIr::return_(value.clone())),
            Self::Break { label } => Some(CompletionRecordIr::break_(None, label.clone())),
            Self::Continue { label } => Some(CompletionRecordIr::continue_(None, label.clone())),
            _ => None,
        }
    }

    pub fn is_abrupt_completion_statement(&self) -> bool {
        self.abrupt_completion_record().is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockIr {
    pub statements: Vec<StatementIr>,
    pub result_kind: ValueKind,
    pub lexical_environment: Option<LexicalEnvironmentIr>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptGlobalBindingKind {
    Intrinsic,
    Infinity,
    NaN,
    Undefined,
    Var,
    Function,
    ReflectObject,
    MathObject,
    JsonObject,
    AtomicsObject,
    BuiltinFunction(StandardBuiltinId),
    HostFunction(HostBuiltinId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptGlobalBindingIr {
    pub name: String,
    pub kind: ScriptGlobalBindingKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptIr {
    pub strict: bool,
    pub functions: Vec<FunctionIr>,
    pub body: BlockIr,
    pub owned_env_bindings: Vec<OwnedEnvBindingIr>,
    pub global_bindings: Vec<ScriptGlobalBindingIr>,
    pub host_builtins: Vec<HostBuiltinId>,
    pub builtin_ctor_calls: usize,
    pub builtin_static_calls: usize,
    pub error_builtin_calls: usize,
    pub aggregate_errors: usize,
    pub function_proto_calls: usize,
    pub function_proto_applies: usize,
    pub function_proto_binds: usize,
    pub function_proto_to_strings: usize,
    pub bound_functions: usize,
    pub bound_function_constructs: usize,
    pub boxed_builtin_calls: usize,
    pub boxed_builtin_constructs: usize,
    pub boxed_receiver_adaptations: usize,
    pub top_level_this_uses: usize,
    pub host_builtin_calls: usize,
    pub error_proto_to_strings: usize,
}

impl ScriptIr {
    pub const fn result_kind(&self) -> ValueKind {
        self.body.result_kind
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramIr {
    pub goal: ParseGoal,
    pub stages: Vec<LoweringStage>,
    pub source_len: usize,
    pub invariants: Vec<&'static str>,
    pub diagnostics: Vec<IrDiagnostic>,
    pub script: Option<ScriptIr>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IrBlockSummary {
    pub super_uses: usize,
    pub this_reads: usize,
}

pub(crate) fn summarize_block(block: &BlockIr) -> IrBlockSummary {
    let mut counts = IrSummaryCounts::default();
    counts.visit_block(block);
    IrBlockSummary {
        super_uses: counts.super_uses,
        this_reads: counts.this_reads,
    }
}

impl ProgramIr {
    pub fn is_wasm_supported(&self) -> bool {
        self.script.is_some()
            && self.diagnostics.iter().all(|diagnostic| {
                !matches!(
                    diagnostic.kind,
                    IrDiagnosticKind::Unsupported | IrDiagnosticKind::EarlyError
                )
            })
    }

    pub fn ir_summary(&self) -> String {
        match &self.script {
            Some(script) => {
                let mut counts = IrSummaryCounts::default();
                counts.functions += script.functions.len();
                for function in &script.functions {
                    counts.visit_function(function);
                }
                counts.visit_block(&script.body);
                format!(
                    "script statements={} result={} strict={} functions={} nested_functions={} function_exprs={} arrow_functions={} named_function_exprs={} closures={} captures={} lexical_this_captures={} lexical_arguments_captures={} default_params={} rest_params={} arguments_uses={} calls={} indirect_calls={} method_calls={} constructs={} classes={} class_exprs={} class_extends={} null_heritage_classes={} class_fields={} private_elements={} static_blocks={} super_uses={} null_heritage_super_uses={} private_in_checks={} throws={} try_catches={} try_finallys={} returns={} lets={} consts={} vars={} global_bindings={} builtin_globals={} boxed_builtin_globals={} global_this_uses={} top_level_this_uses={} global_default_this_calls={} global_property_reads={} global_property_writes={} implicit_globals={} host_globals={} host_builtin_calls={} builtin_ctor_calls={} builtin_static_calls={} error_builtin_calls={} aggregate_errors={} function_proto_calls={} function_proto_applies={} function_proto_binds={} function_proto_to_strings={} bound_functions={} bound_function_constructs={} boxed_builtin_calls={} boxed_builtin_constructs={} boxed_receiver_adaptations={} error_proto_to_strings={} blocks={} ifs={} whiles={} do_whiles={} fors={} switches={} labels={} debuggers={} breaks={} continues={} objects={} object_shorthands={} object_methods={} object_getters={} object_setters={} arrays={} property_reads={} property_writes={} array_lengths={} heap_shapes={} function_values={} this_reads={} new_target_uses={} assigns={} prefix_updates={} postfix_updates={} compound_assigns={} string_concats={} loose_equalities={} coercive_numeric_ops={} coercive_relational_ops={} typeof_uses={} void_uses={} deletes={} identifier_deletes={} global_deletes={} comma_ops={} nullish_ops={} spec_operations={} kind_unions={} heap_to_primitives={} heap_loose_equalities={} heap_coercions={} instanceofs={} in_ops={} prototype_reads={} prototype_writes={}",
                    counts.statements,
                    script.result_kind().as_str(),
                    script.strict,
                    counts.functions,
                    counts.nested_functions,
                    counts.function_exprs,
                    counts.arrow_functions,
                    counts.named_function_exprs,
                    counts.closures,
                    counts.captures,
                    counts.lexical_this_captures,
                    counts.lexical_arguments_captures,
                    counts.default_params,
                    counts.rest_params,
                    counts.arguments_uses,
                    counts.calls,
                    counts.indirect_calls,
                    counts.method_calls,
                    counts.constructs,
                    counts.classes,
                    counts.class_exprs,
                    counts.class_extends,
                    counts.null_heritage_classes,
                    counts.class_fields,
                    counts.private_elements,
                    counts.static_blocks,
                    counts.super_uses,
                    counts.null_heritage_super_uses,
                    counts.private_in_checks,
                    counts.throws,
                    counts.try_catches,
                    counts.try_finallys,
                    counts.returns,
                    counts.lets,
                    counts.consts,
                    counts.vars,
                    script.global_bindings.len(),
                    script
                        .global_bindings
                        .iter()
                        .filter(|binding| matches!(
                            binding.kind,
                            ScriptGlobalBindingKind::BuiltinFunction(_)
                        ))
                        .count(),
                    script
                        .global_bindings
                        .iter()
                        .filter(|binding| {
                            matches!(
                                binding.kind,
                                ScriptGlobalBindingKind::BuiltinFunction(builtin)
                                    if builtin.is_boxed_primitive_constructor()
                            )
                        })
                        .count(),
                    counts.global_this_uses,
                    script.top_level_this_uses,
                    counts.global_default_this_calls,
                    counts.global_property_reads,
                    counts.global_property_writes,
                    counts.implicit_globals,
                    script.host_builtins.len(),
                    script.host_builtin_calls,
                    script.builtin_ctor_calls,
                    script.builtin_static_calls,
                    script.error_builtin_calls,
                    script.aggregate_errors,
                    script.function_proto_calls,
                    script.function_proto_applies,
                    script.function_proto_binds,
                    script.function_proto_to_strings,
                    script.bound_functions,
                    script.bound_function_constructs,
                    script.boxed_builtin_calls,
                    script.boxed_builtin_constructs,
                    script.boxed_receiver_adaptations,
                    script.error_proto_to_strings,
                    counts.blocks,
                    counts.ifs,
                    counts.whiles,
                    counts.do_whiles,
                    counts.fors,
                    counts.switches,
                    counts.labels,
                    counts.debuggers,
                    counts.breaks,
                    counts.continues,
                    counts.objects,
                    counts.object_shorthands,
                    counts.object_methods,
                    counts.object_getters,
                    counts.object_setters,
                    counts.arrays,
                    counts.property_reads,
                    counts.property_writes,
                    counts.array_lengths,
                    counts.heap_shapes,
                    counts.function_values,
                    counts.this_reads,
                    counts.new_target_uses,
                    counts.assignments,
                    counts.prefix_updates,
                    counts.postfix_updates,
                    counts.compound_assignments,
                    counts.string_concats,
                    counts.loose_equalities,
                    counts.coercive_numeric_ops,
                    counts.coercive_relational_ops,
                    counts.typeof_uses,
                    counts.void_uses,
                    counts.deletes,
                    counts.identifier_deletes,
                    counts.global_deletes,
                    counts.comma_ops,
                    counts.nullish_ops,
                    counts.spec_operations,
                    counts.kind_unions,
                    counts.heap_to_primitives,
                    counts.heap_loose_equalities,
                    counts.heap_coercions,
                    counts.instanceofs,
                    counts.in_ops,
                    counts.prototype_reads,
                    counts.prototype_writes
                )
            }
            None => "no script ir".to_string(),
        }
    }
}

#[derive(Default)]
struct IrSummaryCounts {
    statements: usize,
    functions: usize,
    nested_functions: usize,
    function_exprs: usize,
    arrow_functions: usize,
    named_function_exprs: usize,
    closures: usize,
    captures: usize,
    lexical_this_captures: usize,
    lexical_arguments_captures: usize,
    default_params: usize,
    rest_params: usize,
    arguments_uses: usize,
    calls: usize,
    indirect_calls: usize,
    method_calls: usize,
    constructs: usize,
    classes: usize,
    class_exprs: usize,
    class_extends: usize,
    null_heritage_classes: usize,
    class_fields: usize,
    private_elements: usize,
    static_blocks: usize,
    super_uses: usize,
    null_heritage_super_uses: usize,
    private_in_checks: usize,
    throws: usize,
    try_catches: usize,
    try_finallys: usize,
    returns: usize,
    lets: usize,
    consts: usize,
    vars: usize,
    global_this_uses: usize,
    global_default_this_calls: usize,
    global_property_reads: usize,
    global_property_writes: usize,
    implicit_globals: usize,
    blocks: usize,
    ifs: usize,
    whiles: usize,
    do_whiles: usize,
    fors: usize,
    switches: usize,
    labels: usize,
    debuggers: usize,
    breaks: usize,
    continues: usize,
    objects: usize,
    object_shorthands: usize,
    object_methods: usize,
    object_getters: usize,
    object_setters: usize,
    arrays: usize,
    property_reads: usize,
    property_writes: usize,
    array_lengths: usize,
    heap_shapes: usize,
    function_values: usize,
    this_reads: usize,
    new_target_uses: usize,
    assignments: usize,
    prefix_updates: usize,
    postfix_updates: usize,
    compound_assignments: usize,
    string_concats: usize,
    loose_equalities: usize,
    coercive_numeric_ops: usize,
    coercive_relational_ops: usize,
    typeof_uses: usize,
    void_uses: usize,
    deletes: usize,
    identifier_deletes: usize,
    global_deletes: usize,
    comma_ops: usize,
    nullish_ops: usize,
    spec_operations: usize,
    kind_unions: usize,
    heap_to_primitives: usize,
    heap_loose_equalities: usize,
    heap_coercions: usize,
    instanceofs: usize,
    in_ops: usize,
    prototype_reads: usize,
    prototype_writes: usize,
}

impl IrSummaryCounts {
    fn visit_function(&mut self, function: &FunctionIr) {
        if let Some(plan) = &function.class_instance_element_plan {
            self.class_fields += plan.fields.len();
            self.private_elements += plan
                .fields
                .iter()
                .filter(|field| matches!(&field.key, ClassFieldKeyIr::Private(_)))
                .count();
        }
        if function.is_nested {
            self.nested_functions += 1;
        }
        if function.is_expression {
            self.function_exprs += 1;
        }
        if function.flavor == FunctionFlavor::Arrow {
            self.arrow_functions += 1;
        }
        if function.is_named_expression {
            self.named_function_exprs += 1;
        }
        if !function.captured_bindings.is_empty() || !function.owned_env_bindings.is_empty() {
            self.closures += 1;
        }
        self.captures += function.captured_bindings.len();
        if function.captures_lexical_this {
            self.lexical_this_captures += 1;
        }
        if function.captures_lexical_arguments {
            self.lexical_arguments_captures += 1;
        }
        for param in &function.params {
            if param.default_init.is_some() {
                self.default_params += 1;
            }
            if param.is_rest {
                self.rest_params += 1;
            }
        }
        if function.class_heritage_kind == ClassHeritageKind::Null && function.uses_super {
            self.null_heritage_super_uses += 1;
        }
        self.visit_block(&function.body);
    }

    fn visit_block(&mut self, block: &BlockIr) {
        for statement in &block.statements {
            self.visit_statement(statement);
        }
    }

    fn visit_statement(&mut self, statement: &StatementIr) {
        self.statements += 1;
        match statement {
            StatementIr::Empty | StatementIr::AnnexBFunctionCopy { .. } => {}
            StatementIr::Lexical { mode, init, .. } => {
                match mode {
                    BindingMode::Let => self.lets += 1,
                    BindingMode::Const => self.consts += 1,
                    BindingMode::Var => self.vars += 1,
                }
                self.visit_expr(init);
            }
            StatementIr::LexicalBlock(statements) => {
                for statement in statements {
                    self.visit_statement(statement);
                }
            }
            StatementIr::Var(declarators) => {
                self.vars += declarators.len();
                for declarator in declarators {
                    if let Some(init) = &declarator.init {
                        self.visit_expr(init);
                    }
                }
            }
            StatementIr::Expression(expr) => self.visit_expr(expr),
            StatementIr::Block(block) => {
                self.blocks += 1;
                self.visit_block(block);
            }
            StatementIr::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.ifs += 1;
                self.visit_expr(condition);
                self.visit_statement(then_branch);
                if let Some(else_branch) = else_branch {
                    self.visit_statement(else_branch);
                }
            }
            StatementIr::While { condition, body } => {
                self.whiles += 1;
                self.visit_expr(condition);
                self.visit_statement(body);
            }
            StatementIr::DoWhile { body, condition } => {
                self.do_whiles += 1;
                self.visit_statement(body);
                self.visit_expr(condition);
            }
            StatementIr::For {
                init,
                test,
                update,
                body,
                ..
            } => {
                self.fors += 1;
                if let Some(init) = init {
                    self.visit_for_init(init);
                }
                if let Some(test) = test {
                    self.visit_expr(test);
                }
                if let Some(update) = update {
                    self.visit_expr(update);
                }
                self.visit_statement(body);
            }
            StatementIr::ForOfArray { iterable, body, .. }
            | StatementIr::ForOfString { iterable, body, .. }
            | StatementIr::ForOfIterator { iterable, body, .. }
            | StatementIr::ForInArray {
                target: iterable,
                body,
                ..
            }
            | StatementIr::ForInString {
                target: iterable,
                body,
                ..
            }
            | StatementIr::ForInObject {
                target: iterable,
                body,
                ..
            } => {
                self.fors += 1;
                self.visit_expr(iterable);
                self.visit_statement(body);
            }
            StatementIr::Switch {
                discriminant,
                lexical_declarations,
                cases,
                ..
            } => {
                self.switches += 1;
                self.visit_expr(discriminant);
                for declaration in lexical_declarations {
                    self.visit_statement(declaration);
                }
                for case in cases {
                    if let Some(condition) = &case.condition {
                        self.visit_expr(condition);
                    }
                    self.visit_block(&case.body);
                }
            }
            StatementIr::Labelled { labels, statement } => {
                self.labels += labels.len();
                self.visit_statement(statement);
            }
            StatementIr::Debugger => self.debuggers += 1,
            StatementIr::Throw(expr) => {
                self.throws += 1;
                self.visit_expr(expr);
            }
            StatementIr::TryCatch {
                try_block,
                catch_block,
                ..
            } => {
                self.try_catches += 1;
                self.visit_block(try_block);
                self.visit_block(catch_block);
            }
            StatementIr::TryFinally {
                try_block,
                finally_block,
            } => {
                self.try_finallys += 1;
                self.visit_block(try_block);
                self.visit_block(finally_block);
            }
            StatementIr::TryCatchFinally {
                try_block,
                catch_block,
                finally_block,
                ..
            } => {
                self.try_finallys += 1;
                self.visit_block(try_block);
                self.visit_block(catch_block);
                self.visit_block(finally_block);
            }
            StatementIr::Return(expr) => {
                self.returns += 1;
                self.visit_expr(expr);
            }
            StatementIr::Break { .. } => self.breaks += 1,
            StatementIr::Continue { .. } => self.continues += 1,
        }
    }

    fn visit_for_init(&mut self, init: &ForInitIr) {
        match init {
            ForInitIr::Lexical { mode, init, .. } => {
                match mode {
                    BindingMode::Let => self.lets += 1,
                    BindingMode::Const => self.consts += 1,
                    BindingMode::Var => self.vars += 1,
                }
                self.visit_expr(init);
            }
            ForInitIr::LexicalBlock(bindings) => {
                for binding in bindings {
                    match binding.mode {
                        BindingMode::Let => self.lets += 1,
                        BindingMode::Const => self.consts += 1,
                        BindingMode::Var => self.vars += 1,
                    }
                    self.visit_expr(&binding.init);
                }
            }
            ForInitIr::Var(declarators) => {
                self.vars += declarators.len();
                for declarator in declarators {
                    if let Some(init) = &declarator.init {
                        self.visit_expr(init);
                    }
                }
            }
            ForInitIr::Expression(expr) => self.visit_expr(expr),
        }
    }

    fn visit_expr(&mut self, expr: &TypedExpr) {
        if expr.heap_shape.is_some() {
            self.heap_shapes += 1;
        }
        if !expr.possible_kinds.is_singleton() {
            self.kind_unions += 1;
        }
        if expr.kind == ValueKind::Function {
            self.function_values += 1;
        }
        match &expr.expr {
            ExprIr::AssignIdentifier { value, .. } => {
                self.assignments += 1;
                self.visit_expr(value);
            }
            ExprIr::GlobalPropertyRead { .. } => {
                self.global_property_reads += 1;
            }
            ExprIr::GlobalPropertyWrite {
                value, implicit, ..
            } => {
                self.assignments += 1;
                self.global_property_writes += 1;
                self.implicit_globals += usize::from(*implicit);
                self.visit_expr(value);
            }
            ExprIr::ObjectLiteral(properties) => {
                self.objects += 1;
                for property in properties {
                    match property {
                        ObjectPropertyIr::PrototypeSetter { value } => {
                            self.visit_expr(value);
                        }
                        ObjectPropertyIr::Data {
                            value,
                            is_shorthand,
                            ..
                        } => {
                            if *is_shorthand {
                                self.object_shorthands += 1;
                            }
                            self.visit_expr(value);
                        }
                        ObjectPropertyIr::NonEnumerableData { value, .. } => {
                            self.visit_expr(value);
                        }
                        ObjectPropertyIr::ComputedData { key, value } => {
                            self.visit_expr(key);
                            self.visit_expr(value);
                        }
                        ObjectPropertyIr::ComputedMethod { key, function } => {
                            self.object_methods += 1;
                            self.visit_expr(key);
                            self.visit_expr(function);
                        }
                        ObjectPropertyIr::Method { function, .. } => {
                            self.object_methods += 1;
                            self.visit_expr(function);
                        }
                        ObjectPropertyIr::ComputedGetter { key, function } => {
                            self.object_getters += 1;
                            self.visit_expr(key);
                            self.visit_expr(function);
                        }
                        ObjectPropertyIr::Getter { function, .. } => {
                            self.object_getters += 1;
                            self.visit_expr(function);
                        }
                        ObjectPropertyIr::ComputedSetter { key, function } => {
                            self.object_setters += 1;
                            self.visit_expr(key);
                            self.visit_expr(function);
                        }
                        ObjectPropertyIr::Setter { function, .. } => {
                            self.object_setters += 1;
                            self.visit_expr(function);
                        }
                    }
                }
            }
            ExprIr::RegExpLiteral { .. } => {
                self.objects += 1;
            }
            ExprIr::ArrayLiteral(elements) => {
                self.arrays += 1;
                for element in elements {
                    self.visit_expr(element);
                }
            }
            ExprIr::TemplateObject(_) => {
                self.arrays += 2;
            }
            ExprIr::SpreadArgument(value) => {
                self.visit_expr(value);
            }
            ExprIr::PropertyRead { target, key } => {
                self.property_reads += 1;
                if matches!(key, PropertyKeyIr::ArrayLength) {
                    self.array_lengths += 1;
                }
                if matches!(key, PropertyKeyIr::StaticString(name) if name == "prototype") {
                    self.prototype_reads += 1;
                }
                self.visit_expr(target);
                self.visit_property_key(key);
            }
            ExprIr::OptionalPropertyChain { target, chain } => {
                self.visit_expr(target);
                let mut previous_was_property = false;
                for operation in chain {
                    match operation {
                        OptionalChainOperationIr::Property { key, .. } => {
                            self.property_reads += 1;
                            if matches!(key, PropertyKeyIr::ArrayLength) {
                                self.array_lengths += 1;
                            }
                            if matches!(key, PropertyKeyIr::StaticString(name) if name == "prototype")
                            {
                                self.prototype_reads += 1;
                            }
                            self.visit_property_key(key);
                            previous_was_property = true;
                        }
                        OptionalChainOperationIr::PrivateProperty { .. } => {
                            self.private_elements += 1;
                            previous_was_property = true;
                        }
                        OptionalChainOperationIr::Call { args, receiver, .. } => {
                            self.calls += 1;
                            self.indirect_calls += 1;
                            self.method_calls += usize::from(
                                previous_was_property
                                    || *receiver == OptionalChainCallReceiverIr::CurrentThis,
                            );
                            for arg in args {
                                self.visit_expr(arg);
                            }
                            previous_was_property = false;
                        }
                    }
                }
            }
            ExprIr::PropertyWrite { target, key, value } => {
                self.property_writes += 1;
                if matches!(key, PropertyKeyIr::StaticString(name) if name == "prototype") {
                    self.prototype_writes += 1;
                }
                self.visit_expr(target);
                self.visit_property_key(key);
                self.visit_expr(value);
            }
            ExprIr::PropertyUpdate {
                target,
                key,
                return_mode,
                ..
            } => {
                self.property_reads += 1;
                self.property_writes += 1;
                match return_mode {
                    UpdateReturnMode::Prefix => self.prefix_updates += 1,
                    UpdateReturnMode::Postfix => self.postfix_updates += 1,
                }
                self.visit_expr(target);
                self.visit_property_key(key);
            }
            ExprIr::PropertyCompoundAssign {
                target, key, value, ..
            } => {
                self.property_reads += 1;
                self.property_writes += 1;
                self.compound_assignments += 1;
                self.visit_expr(target);
                self.visit_property_key(key);
                self.visit_expr(value);
            }
            ExprIr::UpdateIdentifier { return_mode, .. } => match return_mode {
                UpdateReturnMode::Prefix => self.prefix_updates += 1,
                UpdateReturnMode::Postfix => self.postfix_updates += 1,
            },
            ExprIr::GlobalPropertyUpdate { return_mode, .. } => {
                self.global_property_writes += 1;
                match return_mode {
                    UpdateReturnMode::Prefix => self.prefix_updates += 1,
                    UpdateReturnMode::Postfix => self.postfix_updates += 1,
                }
            }
            ExprIr::CompoundAssignIdentifier { value, .. } => {
                self.compound_assignments += 1;
                self.visit_expr(value);
            }
            ExprIr::GlobalPropertyCompoundAssign { value, .. } => {
                self.compound_assignments += 1;
                self.global_property_writes += 1;
                self.visit_expr(value);
            }
            ExprIr::UnaryNumber { expr, .. } | ExprIr::StringFromCharCode { code: expr } => {
                self.visit_expr(expr);
            }
            ExprIr::StringCharCodeAt { target, index } => {
                self.visit_expr(target);
                self.visit_expr(index);
            }
            ExprIr::LogicalNot { expr } => {
                self.visit_expr(expr);
            }
            ExprIr::SpecOperation { operands, .. } => {
                self.spec_operations += 1;
                if let ExprIr::SpecOperation { operation, .. } = &expr.expr {
                    match operation {
                        SpecOperationIr::Get | SpecOperationIr::GetV => {
                            self.property_reads += 1;
                            if matches!(operands.get(1).map(|operand| &operand.expr), Some(ExprIr::String(name)) if name == "prototype")
                            {
                                self.prototype_reads += 1;
                            }
                        }
                        SpecOperationIr::GetMethod => {
                            self.property_reads += 1;
                        }
                        SpecOperationIr::CreateDataPropertyOrThrow => {
                            self.property_writes += 1;
                        }
                        SpecOperationIr::Set => {
                            self.property_writes += 1;
                        }
                        SpecOperationIr::DeletePropertyOrThrow => {
                            self.deletes += 1;
                        }
                        SpecOperationIr::Call => {
                            self.calls += 1;
                            self.indirect_calls += 1;
                        }
                        SpecOperationIr::Construct => {
                            self.constructs += 1;
                            self.indirect_calls += 1;
                        }
                        SpecOperationIr::IsLooselyEqual => {
                            self.loose_equalities += 1;
                            if let [lhs, rhs] = operands.as_slice() {
                                if !lhs.possible_kinds.is_subset_of(KindSet::PRIMITIVE_ONLY)
                                    || !rhs.possible_kinds.is_subset_of(KindSet::PRIMITIVE_ONLY)
                                {
                                    self.heap_loose_equalities += 1;
                                    self.heap_coercions += 1;
                                }
                            }
                        }
                        _ => {}
                    }
                }
                for operand in operands {
                    self.visit_expr(operand);
                }
            }
            ExprIr::Void { expr } => {
                self.void_uses += 1;
                self.visit_expr(expr);
            }
            ExprIr::DeleteValue { expr } => {
                self.deletes += 1;
                self.visit_expr(expr);
            }
            ExprIr::DeleteIdentifier { kind, .. } => {
                self.deletes += 1;
                self.identifier_deletes += 1;
                if matches!(kind, DeleteIdentifierKindIr::Missing) {
                    self.global_deletes += 1;
                }
            }
            ExprIr::DeleteGlobalProperty { .. } => {
                self.deletes += 1;
                self.global_deletes += 1;
            }
            ExprIr::DeleteProperty { target, key, .. } => {
                self.deletes += 1;
                self.visit_expr(target);
                self.visit_property_key(key);
            }
            ExprIr::TypeOf { expr } => {
                self.typeof_uses += 1;
                self.visit_expr(expr);
            }
            ExprIr::TypeOfUnresolvedIdentifier { .. } => {
                self.typeof_uses += 1;
            }
            ExprIr::BinaryNumber { lhs, rhs, .. }
            | ExprIr::CoerciveAdd { lhs, rhs }
            | ExprIr::CoerciveBinaryNumber { lhs, rhs, .. }
            | ExprIr::BitwiseNumber { lhs, rhs, .. }
            | ExprIr::CompareNumber { lhs, rhs, .. }
            | ExprIr::CompareValue { lhs, rhs, .. }
            | ExprIr::StrictEquality { lhs, rhs, .. }
            | ExprIr::LooseEquality { lhs, rhs, .. }
            | ExprIr::AssertSameValue {
                actual: lhs,
                expected: rhs,
                ..
            }
            | ExprIr::StringConcat { lhs, rhs }
            | ExprIr::LogicalShortCircuit { lhs, rhs, .. }
            | ExprIr::Comma { lhs, rhs } => {
                if matches!(&expr.expr, ExprIr::StringConcat { .. }) {
                    self.string_concats += 1;
                }
                if matches!(&expr.expr, ExprIr::CoerciveAdd { .. }) {
                    self.heap_to_primitives += 1;
                    self.heap_coercions += 1;
                }
                if matches!(&expr.expr, ExprIr::LooseEquality { .. }) {
                    self.loose_equalities += 1;
                    if !lhs.possible_kinds.is_subset_of(KindSet::PRIMITIVE_ONLY)
                        || !rhs.possible_kinds.is_subset_of(KindSet::PRIMITIVE_ONLY)
                    {
                        self.heap_loose_equalities += 1;
                        self.heap_coercions += 1;
                    }
                }
                if matches!(&expr.expr, ExprIr::CoerciveBinaryNumber { .. }) {
                    self.coercive_numeric_ops += 1;
                    if !lhs.possible_kinds.is_subset_of(KindSet::PRIMITIVE_ONLY)
                        || !rhs.possible_kinds.is_subset_of(KindSet::PRIMITIVE_ONLY)
                    {
                        self.heap_to_primitives += 1;
                        self.heap_coercions += 1;
                    }
                }
                if matches!(&expr.expr, ExprIr::CompareValue { .. }) {
                    self.coercive_relational_ops += 1;
                    if !lhs.possible_kinds.is_subset_of(KindSet::PRIMITIVE_ONLY)
                        || !rhs.possible_kinds.is_subset_of(KindSet::PRIMITIVE_ONLY)
                    {
                        self.heap_to_primitives += 1;
                        self.heap_coercions += 1;
                    }
                }
                if matches!(
                    &expr.expr,
                    ExprIr::LogicalShortCircuit {
                        op: LogicalBinaryOp::Coalesce,
                        ..
                    }
                ) {
                    self.nullish_ops += 1;
                }
                if matches!(&expr.expr, ExprIr::Comma { .. }) {
                    self.comma_ops += 1;
                }
                self.visit_expr(lhs);
                self.visit_expr(rhs);
            }
            ExprIr::MaterializeBinding { value, body, .. } => {
                self.visit_expr(value);
                self.visit_expr(body);
            }
            ExprIr::ArrayDestructure { value, pattern, .. } => {
                self.assignments += 1;
                self.visit_expr(value);
                pattern.visit_expressions(&mut |expr| self.visit_expr(expr));
            }
            ExprIr::ObjectDestructure { value, pattern } => {
                self.assignments += 1;
                self.visit_expr(value);
                pattern.visit_expressions(&mut |expr| self.visit_expr(expr));
            }
            ExprIr::Conditional {
                condition,
                then_expr,
                else_expr,
            } => {
                self.visit_expr(condition);
                self.visit_expr(then_expr);
                self.visit_expr(else_expr);
            }
            ExprIr::CallNamed { args, .. } => {
                self.calls += 1;
                self.global_default_this_calls += 1;
                for arg in args {
                    self.visit_expr(arg);
                }
            }
            ExprIr::RuntimeThrow { .. } => {}
            ExprIr::Arguments => {
                self.arguments_uses += 1;
            }
            ExprIr::CallIndirect { callee, args, .. } => {
                self.calls += 1;
                self.indirect_calls += 1;
                self.global_default_this_calls += 1;
                self.visit_expr(callee);
                for arg in args {
                    self.visit_expr(arg);
                }
            }
            ExprIr::JsonParseStaticReviver { reviver, .. } => {
                self.calls += 1;
                self.indirect_calls += 1;
                self.visit_expr(reviver);
            }
            ExprIr::Construct { callee, args, .. } => {
                self.calls += 1;
                self.indirect_calls += 1;
                self.constructs += 1;
                self.visit_expr(callee);
                for arg in args {
                    self.visit_expr(arg);
                }
            }
            ExprIr::ClassDefinition(class) => {
                self.constructs += 1;
                self.classes += 1;
                self.class_exprs += 1;
                self.class_extends += usize::from(class.heritage_kind != ClassHeritageKind::None);
                self.null_heritage_classes +=
                    usize::from(class.heritage_kind == ClassHeritageKind::Null);
                for static_element in &class.element_plan.static_elements {
                    match static_element {
                        ClassStaticElementIr::Field(field) => {
                            self.class_fields += 1;
                            self.private_elements +=
                                usize::from(matches!(&field.key, ClassFieldKeyIr::Private(_)));
                        }
                        ClassStaticElementIr::Block(_) => self.static_blocks += 1,
                    }
                }
                self.private_elements += class
                    .element_plan
                    .definitions
                    .iter()
                    .filter(|definition| {
                        matches!(definition, ClassElementDefinitionIr::PrivateMethod(_))
                    })
                    .count();
                if let Some(heritage) = &class.heritage {
                    self.visit_expr(heritage);
                }
                for definition in &class.element_plan.definitions {
                    let ClassElementDefinitionIr::PublicMethod(method) = definition else {
                        continue;
                    };
                    self.visit_property_key(&method.key);
                }
            }
            ExprIr::CallMethod {
                receiver,
                key,
                args,
            } => {
                self.calls += 1;
                self.indirect_calls += 1;
                self.method_calls += 1;
                self.visit_expr(receiver);
                self.visit_property_key(key);
                for arg in args {
                    self.visit_expr(arg);
                }
            }
            ExprIr::This => {
                self.this_reads += 1;
            }
            ExprIr::NewTarget => {
                self.new_target_uses += 1;
            }
            ExprIr::SuperConstruct { args } => {
                self.super_uses += 1;
                self.calls += 1;
                self.indirect_calls += 1;
                for arg in args {
                    self.visit_expr(arg);
                }
            }
            ExprIr::SuperPropertyRead { key } => {
                self.super_uses += 1;
                self.visit_property_key(key);
            }
            ExprIr::SuperPropertyWrite { key, value } => {
                self.super_uses += 1;
                self.visit_property_key(key);
                self.visit_expr(value);
            }
            ExprIr::PrivateRead {
                target,
                private_name_id: _,
            } => {
                self.private_elements += 1;
                self.visit_expr(target);
            }
            ExprIr::PrivateWrite {
                target,
                private_name_id: _,
                value,
            } => {
                self.private_elements += 1;
                self.visit_expr(target);
                self.visit_expr(value);
            }
            ExprIr::PrivateIn {
                private_name_id: _,
                rhs,
            } => {
                self.private_in_checks += 1;
                self.visit_expr(rhs);
            }
            ExprIr::InstanceOf { lhs, rhs } => {
                self.instanceofs += 1;
                self.visit_expr(lhs);
                self.visit_expr(rhs);
            }
            ExprIr::In { lhs, rhs } => {
                self.in_ops += 1;
                self.visit_expr(lhs);
                self.visit_expr(rhs);
            }
            ExprIr::Symbol { description } => {
                if let Some(description) = description {
                    self.visit_expr(description);
                }
            }
            ExprIr::Undefined
            | ExprIr::ArrayHole
            | ExprIr::Null
            | ExprIr::Boolean(_)
            | ExprIr::Number(_)
            | ExprIr::BigInt(_)
            | ExprIr::String(_)
            | ExprIr::FunctionValue(_)
            | ExprIr::Identifier(_) => {
                if matches!(&expr.expr, ExprIr::Identifier(name) if name == GLOBAL_THIS_NAME) {
                    self.global_this_uses += 1;
                }
            }
        }
    }

    fn visit_property_key(&mut self, key: &PropertyKeyIr) {
        match key {
            PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => {}
            PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                self.visit_expr(expr)
            }
        }
    }
}

pub(crate) fn read_heap_shape_property(
    shape: &HeapShape,
    key: &str,
) -> Option<ObjectShapeProperty> {
    match shape {
        HeapShape::Object(object) => object.properties.get(key).cloned().or_else(|| {
            object
                .prototype
                .as_deref()
                .and_then(|proto| read_heap_shape_property(proto, key))
        }),
        HeapShape::Array(array) => array.properties.get(key).cloned().or_else(|| {
            array
                .prototype
                .as_deref()
                .and_then(|proto| read_heap_shape_property(proto, key))
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CompletionKindIr;

    #[test]
    fn value_tags_round_trip_for_runtime_tags() {
        for kind in [
            ValueKind::Undefined,
            ValueKind::Null,
            ValueKind::Boolean,
            ValueKind::Number,
            ValueKind::String,
            ValueKind::Symbol,
            ValueKind::Object,
            ValueKind::Array,
            ValueKind::Function,
            ValueKind::Arguments,
            ValueKind::BigInt,
        ] {
            assert_eq!(ValueKind::from_tag(kind.tag()), Some(kind));
            assert!(!kind.as_str().is_empty());
        }
        assert_eq!(ValueKind::from_tag(ValueKind::Dynamic.tag()), None);
    }

    #[test]
    fn operations_value_kind_classifies_known_ecmascript_type() {
        assert_eq!(
            ValueKind::Undefined.known_ecmascript_type(),
            Some(EcmaLanguageType::Undefined)
        );
        assert_eq!(
            ValueKind::Null.known_ecmascript_type(),
            Some(EcmaLanguageType::Null)
        );
        assert_eq!(
            ValueKind::Boolean.known_ecmascript_type(),
            Some(EcmaLanguageType::Boolean)
        );
        assert_eq!(
            ValueKind::String.known_ecmascript_type(),
            Some(EcmaLanguageType::String)
        );
        assert_eq!(
            ValueKind::Symbol.known_ecmascript_type(),
            Some(EcmaLanguageType::Symbol)
        );
        assert_eq!(
            ValueKind::Number.known_ecmascript_type(),
            Some(EcmaLanguageType::Number)
        );
        assert_eq!(
            ValueKind::BigInt.known_ecmascript_type(),
            Some(EcmaLanguageType::BigInt)
        );

        for kind in [
            ValueKind::Object,
            ValueKind::Array,
            ValueKind::Function,
            ValueKind::Arguments,
        ] {
            assert_eq!(kind.known_ecmascript_type(), Some(EcmaLanguageType::Object));
        }
        assert_eq!(ValueKind::Dynamic.known_ecmascript_type(), None);
    }

    #[test]
    fn operations_statement_throw_completion_record_preserves_value() {
        let value = TypedExpr::from_info(
            ValueInfo::new(ValueKind::String),
            ExprIr::String("boom".into()),
        );
        let statement = StatementIr::Throw(value.clone());
        let completion = statement
            .abrupt_completion_record()
            .expect("throw should produce an abrupt completion record");

        assert!(statement.is_abrupt_completion_statement());
        assert_eq!(completion.kind(), CompletionKindIr::Throw);
        assert!(completion.is_abrupt());
        assert_eq!(completion.value(), Some(&value));
        assert_eq!(completion.target(), None);
    }

    #[test]
    fn operations_statement_return_completion_record_preserves_value() {
        let value = TypedExpr::from_info(
            ValueInfo::new(ValueKind::Number),
            ExprIr::Number(7.0f64.to_bits()),
        );
        let statement = StatementIr::Return(value.clone());
        let completion = statement
            .abrupt_completion_record()
            .expect("return should produce an abrupt completion record");

        assert!(statement.is_abrupt_completion_statement());
        assert_eq!(completion.kind(), CompletionKindIr::Return);
        assert!(completion.is_abrupt());
        assert_eq!(completion.value(), Some(&value));
        assert_eq!(completion.target(), None);
    }

    #[test]
    fn operations_statement_break_continue_completion_record_preserves_target() {
        let break_statement = StatementIr::Break {
            label: Some("outer".into()),
        };
        let continue_statement = StatementIr::Continue {
            label: Some("loop".into()),
        };
        let empty_statement = StatementIr::Empty;

        let break_completion = break_statement
            .abrupt_completion_record()
            .expect("break should produce an abrupt completion record");
        let continue_completion = continue_statement
            .abrupt_completion_record()
            .expect("continue should produce an abrupt completion record");

        assert_eq!(break_completion.kind(), CompletionKindIr::Break);
        assert!(break_completion.is_abrupt());
        assert_eq!(break_completion.value(), None);
        assert_eq!(break_completion.target(), Some("outer"));
        assert_eq!(continue_completion.kind(), CompletionKindIr::Continue);
        assert!(continue_completion.is_abrupt());
        assert_eq!(continue_completion.value(), None);
        assert_eq!(continue_completion.target(), Some("loop"));
        assert!(!empty_statement.is_abrupt_completion_statement());
        assert_eq!(empty_statement.abrupt_completion_record(), None);
    }

    #[test]
    fn operations_spec_is_callable_expr_records_operation_and_operand() {
        let operand = TypedExpr::from_info(
            ValueInfo::new(ValueKind::Function),
            ExprIr::FunctionValue("callable".to_string()),
        );
        let expr = TypedExpr::spec_is_callable(operand.clone());

        assert_eq!(expr.kind, ValueKind::Boolean);
        assert_eq!(expr.possible_kinds, KindSet::from_kind(ValueKind::Boolean));
        let ExprIr::SpecOperation {
            operation,
            operands,
        } = expr.expr
        else {
            panic!("expected spec operation expression");
        };
        assert_eq!(operation, SpecOperationIr::IsCallable);
        assert_eq!(operation.name(), "IsCallable");
        assert_eq!(operands, vec![operand]);
    }

    #[test]
    fn operations_spec_is_constructor_expr_records_operation_and_operand() {
        let operand = TypedExpr::from_info(
            ValueInfo::new(ValueKind::Function),
            ExprIr::FunctionValue("ctor".to_string()),
        );
        let expr = TypedExpr::spec_is_constructor(operand.clone());

        assert_eq!(expr.kind, ValueKind::Boolean);
        assert_eq!(expr.possible_kinds, KindSet::from_kind(ValueKind::Boolean));
        let ExprIr::SpecOperation {
            operation,
            operands,
        } = expr.expr
        else {
            panic!("expected spec operation expression");
        };
        assert_eq!(operation, SpecOperationIr::IsConstructor);
        assert_eq!(operation.name(), "IsConstructor");
        assert_eq!(operands, vec![operand]);
    }

    #[test]
    fn operations_spec_is_property_key_expr_records_operation_and_operand() {
        let operand = TypedExpr::from_info(
            ValueInfo::new(ValueKind::Symbol),
            ExprIr::Symbol { description: None },
        );
        let expr = TypedExpr::spec_is_property_key(operand.clone());

        assert_eq!(expr.kind, ValueKind::Boolean);
        assert_eq!(expr.possible_kinds, KindSet::from_kind(ValueKind::Boolean));
        let ExprIr::SpecOperation {
            operation,
            operands,
        } = expr.expr
        else {
            panic!("expected spec operation expression");
        };
        assert_eq!(operation, SpecOperationIr::IsPropertyKey);
        assert_eq!(operation.name(), "IsPropertyKey");
        assert_eq!(operands, vec![operand]);
    }

    #[test]
    fn operations_spec_to_boolean_expr_records_operation_and_operand() {
        let operand = TypedExpr::from_info(
            ValueInfo::new(ValueKind::Number),
            ExprIr::Number(1.0f64.to_bits()),
        );
        let expr = TypedExpr::spec_to_boolean(operand.clone());

        assert_eq!(expr.kind, ValueKind::Boolean);
        assert_eq!(expr.possible_kinds, KindSet::from_kind(ValueKind::Boolean));
        let ExprIr::SpecOperation {
            operation,
            operands,
        } = expr.expr
        else {
            panic!("expected spec operation expression");
        };
        assert_eq!(operation, SpecOperationIr::ToBoolean);
        assert_eq!(operation.name(), "ToBoolean");
        assert_eq!(operands, vec![operand]);
    }

    #[test]
    fn operations_spec_to_primitive_expr_records_hint_and_operand() {
        let operand = TypedExpr::from_info(
            ValueInfo::new(ValueKind::Object),
            ExprIr::Identifier(GLOBAL_THIS_NAME.to_string()),
        );
        let expr = TypedExpr::spec_to_primitive(operand.clone(), ToPrimitiveHint::String);

        assert_eq!(expr.kind, ValueKind::Dynamic);
        assert_eq!(expr.possible_kinds, KindSet::PRIMITIVE_ONLY);
        let ExprIr::SpecOperation {
            operation,
            operands,
        } = expr.expr
        else {
            panic!("expected spec operation expression");
        };
        assert_eq!(
            operation,
            SpecOperationIr::ToPrimitive(ToPrimitiveHint::String)
        );
        assert_eq!(operation.name(), "ToPrimitive");
        assert_eq!(operands, vec![operand]);
    }

    #[test]
    fn operations_spec_to_number_expr_records_operation_and_operand() {
        let operand = TypedExpr::from_info(
            ValueInfo::new(ValueKind::String),
            ExprIr::String("1".to_string()),
        );
        let expr = TypedExpr::spec_to_number(operand.clone());

        assert_eq!(expr.kind, ValueKind::Number);
        assert_eq!(expr.possible_kinds, KindSet::from_kind(ValueKind::Number));
        let ExprIr::SpecOperation {
            operation,
            operands,
        } = expr.expr
        else {
            panic!("expected spec operation expression");
        };
        assert_eq!(operation, SpecOperationIr::ToNumber);
        assert_eq!(operation.name(), "ToNumber");
        assert_eq!(operands, vec![operand]);
    }

    #[test]
    fn operations_spec_to_numeric_expr_records_operation_and_operand() {
        let operand = TypedExpr::from_info(
            ValueInfo::new(ValueKind::BigInt),
            ExprIr::BigInt(BigIntLiteralIr::from_i64(1)),
        );
        let expr = TypedExpr::spec_to_numeric(operand.clone());

        assert_eq!(expr.kind, ValueKind::Dynamic);
        assert_eq!(
            expr.possible_kinds,
            KindSet::from_kind(ValueKind::Number).union(KindSet::from_kind(ValueKind::BigInt))
        );
        let ExprIr::SpecOperation {
            operation,
            operands,
        } = expr.expr
        else {
            panic!("expected spec operation expression");
        };
        assert_eq!(operation, SpecOperationIr::ToNumeric);
        assert_eq!(operation.name(), "ToNumeric");
        assert_eq!(operands, vec![operand]);
    }

    #[test]
    fn operations_spec_to_bigint_expr_records_operation_and_operand() {
        let operand =
            TypedExpr::from_info(ValueInfo::new(ValueKind::Boolean), ExprIr::Boolean(true));
        let expr = TypedExpr::spec_to_bigint(operand.clone());

        assert_eq!(expr.kind, ValueKind::BigInt);
        assert_eq!(expr.possible_kinds, KindSet::from_kind(ValueKind::BigInt));
        let ExprIr::SpecOperation {
            operation,
            operands,
        } = expr.expr
        else {
            panic!("expected spec operation expression");
        };
        assert_eq!(operation, SpecOperationIr::ToBigInt);
        assert_eq!(operation.name(), "ToBigInt");
        assert_eq!(operands, vec![operand]);
    }

    #[test]
    fn operations_spec_to_string_expr_records_operation_and_operand() {
        let operand = TypedExpr::from_info(
            ValueInfo::new(ValueKind::Number),
            ExprIr::Number(1.0f64.to_bits()),
        );
        let expr = TypedExpr::spec_to_string(operand.clone());

        assert_eq!(expr.kind, ValueKind::String);
        assert_eq!(expr.possible_kinds, KindSet::from_kind(ValueKind::String));
        let ExprIr::SpecOperation {
            operation,
            operands,
        } = expr.expr
        else {
            panic!("expected spec operation expression");
        };
        assert_eq!(operation, SpecOperationIr::ToString);
        assert_eq!(operation.name(), "ToString");
        assert_eq!(operands, vec![operand]);
    }

    #[test]
    fn operations_spec_to_object_expr_records_operation_and_operand() {
        let operand = TypedExpr::from_info(
            ValueInfo::new(ValueKind::String),
            ExprIr::String("boxed".to_string()),
        );
        let expr = TypedExpr::spec_to_object(operand.clone());
        let object_like = KindSet::from_kind(ValueKind::Object)
            .union(KindSet::from_kind(ValueKind::Array))
            .union(KindSet::from_kind(ValueKind::Function))
            .union(KindSet::from_kind(ValueKind::Arguments));

        assert_eq!(expr.kind, ValueKind::Dynamic);
        assert_eq!(expr.possible_kinds, object_like);
        let ExprIr::SpecOperation {
            operation,
            operands,
        } = expr.expr
        else {
            panic!("expected spec operation expression");
        };
        assert_eq!(operation, SpecOperationIr::ToObject);
        assert_eq!(operation.name(), "ToObject");
        assert_eq!(operands, vec![operand]);
    }

    #[test]
    fn operations_spec_to_property_key_expr_records_operation_and_operand() {
        let operand = TypedExpr::from_info(
            ValueInfo::new(ValueKind::Number),
            ExprIr::Number(1.0f64.to_bits()),
        );
        let expr = TypedExpr::spec_to_property_key(operand.clone());

        assert_eq!(expr.kind, ValueKind::Dynamic);
        assert_eq!(
            expr.possible_kinds,
            KindSet::from_kind(ValueKind::String).union(KindSet::from_kind(ValueKind::Symbol))
        );
        let ExprIr::SpecOperation {
            operation,
            operands,
        } = expr.expr
        else {
            panic!("expected spec operation expression");
        };
        assert_eq!(operation, SpecOperationIr::ToPropertyKey);
        assert_eq!(operation.name(), "ToPropertyKey");
        assert_eq!(operands, vec![operand]);
    }

    #[test]
    fn operations_spec_to_integer_or_infinity_expr_records_operation_and_operand() {
        let operand = TypedExpr::from_info(
            ValueInfo::new(ValueKind::String),
            ExprIr::String("-3.7".to_string()),
        );
        let expr = TypedExpr::spec_to_integer_or_infinity(operand.clone());

        assert_eq!(expr.kind, ValueKind::Number);
        assert_eq!(expr.possible_kinds, KindSet::from_kind(ValueKind::Number));
        let ExprIr::SpecOperation {
            operation,
            operands,
        } = expr.expr
        else {
            panic!("expected spec operation expression");
        };
        assert_eq!(operation, SpecOperationIr::ToIntegerOrInfinity);
        assert_eq!(operation.name(), "ToIntegerOrInfinity");
        assert_eq!(operands, vec![operand]);
    }

    #[test]
    fn operations_spec_to_length_expr_records_operation_and_operand() {
        let operand = TypedExpr::from_info(
            ValueInfo::new(ValueKind::String),
            ExprIr::String("3".to_string()),
        );
        let expr = TypedExpr::spec_to_length(operand.clone());

        assert_eq!(expr.kind, ValueKind::Number);
        assert_eq!(expr.possible_kinds, KindSet::from_kind(ValueKind::Number));
        let ExprIr::SpecOperation {
            operation,
            operands,
        } = expr.expr
        else {
            panic!("expected spec operation expression");
        };
        assert_eq!(operation, SpecOperationIr::ToLength);
        assert_eq!(operation.name(), "ToLength");
        assert_eq!(operands, vec![operand]);
    }

    #[test]
    fn operations_spec_to_index_expr_records_operation_and_operand() {
        let operand = TypedExpr::from_info(
            ValueInfo::new(ValueKind::String),
            ExprIr::String("3".to_string()),
        );
        let expr = TypedExpr::spec_to_index(operand.clone());

        assert_eq!(expr.kind, ValueKind::Number);
        assert_eq!(expr.possible_kinds, KindSet::from_kind(ValueKind::Number));
        let ExprIr::SpecOperation {
            operation,
            operands,
        } = expr.expr
        else {
            panic!("expected spec operation expression");
        };
        assert_eq!(operation, SpecOperationIr::ToIndex);
        assert_eq!(operation.name(), "ToIndex");
        assert_eq!(operands, vec![operand]);
    }

    #[test]
    fn operations_spec_strict_equality_expr_records_operands() {
        let lhs = TypedExpr::from_info(ValueInfo::new(ValueKind::Number), ExprIr::Number(1));
        let rhs = TypedExpr::from_info(ValueInfo::new(ValueKind::Number), ExprIr::Number(1));
        let expr = TypedExpr::spec_strict_equality_comparison(lhs.clone(), rhs.clone());

        assert_eq!(expr.kind, ValueKind::Boolean);
        assert_eq!(expr.possible_kinds, KindSet::from_kind(ValueKind::Boolean));
        let ExprIr::SpecOperation {
            operation,
            operands,
        } = expr.expr
        else {
            panic!("expected spec operation expression");
        };
        assert_eq!(operation, SpecOperationIr::StrictEqualityComparison);
        assert_eq!(operation.name(), "StrictEqualityComparison");
        assert_eq!(operands, vec![lhs, rhs]);
    }

    #[test]
    fn operations_spec_is_loosely_equal_expr_records_operands() {
        let lhs = TypedExpr::from_info(
            ValueInfo::new(ValueKind::String),
            ExprIr::String("1".to_string()),
        );
        let rhs = TypedExpr::from_info(
            ValueInfo::new(ValueKind::Number),
            ExprIr::Number(1.0f64.to_bits()),
        );
        let expr = TypedExpr::spec_is_loosely_equal(lhs.clone(), rhs.clone());

        assert_eq!(expr.kind, ValueKind::Boolean);
        assert_eq!(expr.possible_kinds, KindSet::from_kind(ValueKind::Boolean));
        let ExprIr::SpecOperation {
            operation,
            operands,
        } = expr.expr
        else {
            panic!("expected spec operation expression");
        };
        assert_eq!(operation, SpecOperationIr::IsLooselyEqual);
        assert_eq!(operation.name(), "IsLooselyEqual");
        assert_eq!(operands, vec![lhs, rhs]);
    }

    #[test]
    fn operations_spec_same_value_expr_records_operands() {
        let lhs = TypedExpr::from_info(ValueInfo::new(ValueKind::Number), ExprIr::Number(1));
        let rhs = TypedExpr::from_info(ValueInfo::new(ValueKind::Number), ExprIr::Number(1));
        let expr = TypedExpr::spec_same_value(lhs.clone(), rhs.clone());

        assert_eq!(expr.kind, ValueKind::Boolean);
        assert_eq!(expr.possible_kinds, KindSet::from_kind(ValueKind::Boolean));
        let ExprIr::SpecOperation {
            operation,
            operands,
        } = expr.expr
        else {
            panic!("expected spec operation expression");
        };
        assert_eq!(operation, SpecOperationIr::SameValue);
        assert_eq!(operation.name(), "SameValue");
        assert_eq!(operands, vec![lhs, rhs]);
    }

    #[test]
    fn operations_spec_same_value_zero_expr_records_operands() {
        let lhs = TypedExpr::from_info(ValueInfo::new(ValueKind::Number), ExprIr::Number(0));
        let rhs = TypedExpr::from_info(ValueInfo::new(ValueKind::Number), ExprIr::Number(0));
        let expr = TypedExpr::spec_same_value_zero(lhs.clone(), rhs.clone());

        assert_eq!(expr.kind, ValueKind::Boolean);
        assert_eq!(expr.possible_kinds, KindSet::from_kind(ValueKind::Boolean));
        let ExprIr::SpecOperation {
            operation,
            operands,
        } = expr.expr
        else {
            panic!("expected spec operation expression");
        };
        assert_eq!(operation, SpecOperationIr::SameValueZero);
        assert_eq!(operation.name(), "SameValueZero");
        assert_eq!(operands, vec![lhs, rhs]);
    }

    #[test]
    fn operations_spec_get_v_expr_records_target_and_key_operands() {
        let target = TypedExpr::from_info(
            ValueInfo::new(ValueKind::Object),
            ExprIr::Identifier(GLOBAL_THIS_NAME.to_string()),
        );
        let key = TypedExpr::from_info(
            ValueInfo::new(ValueKind::String),
            ExprIr::String("answer".to_string()),
        );
        let expr = TypedExpr::spec_get_v(target.clone(), key.clone());

        assert_eq!(expr.kind, ValueKind::Dynamic);
        let ExprIr::SpecOperation {
            operation,
            operands,
        } = expr.expr
        else {
            panic!("expected spec operation expression");
        };
        assert_eq!(operation, SpecOperationIr::GetV);
        assert_eq!(operation.name(), "GetV");
        assert_eq!(operands, vec![target, key]);
    }

    #[test]
    fn operations_spec_get_expr_records_target_and_key_operands() {
        let target = TypedExpr::from_info(
            ValueInfo::new(ValueKind::Object),
            ExprIr::Identifier(GLOBAL_THIS_NAME.to_string()),
        );
        let key = TypedExpr::from_info(
            ValueInfo::new(ValueKind::String),
            ExprIr::String("answer".to_string()),
        );
        let expr = TypedExpr::spec_get(target.clone(), key.clone());

        assert_eq!(expr.kind, ValueKind::Dynamic);
        let ExprIr::SpecOperation {
            operation,
            operands,
        } = expr.expr
        else {
            panic!("expected spec operation expression");
        };
        assert_eq!(operation, SpecOperationIr::Get);
        assert_eq!(operation.name(), "Get");
        assert_eq!(operands, vec![target, key]);
    }

    #[test]
    fn operations_spec_has_property_expr_records_target_and_key_operands() {
        let target = TypedExpr::from_info(
            ValueInfo::new(ValueKind::Object),
            ExprIr::Identifier(GLOBAL_THIS_NAME.to_string()),
        );
        let key = TypedExpr::from_info(
            ValueInfo::new(ValueKind::String),
            ExprIr::String("answer".to_string()),
        );
        let expr = TypedExpr::spec_has_property(target.clone(), key.clone());

        assert_eq!(expr.kind, ValueKind::Boolean);
        assert_eq!(expr.possible_kinds, KindSet::from_kind(ValueKind::Boolean));
        let ExprIr::SpecOperation {
            operation,
            operands,
        } = expr.expr
        else {
            panic!("expected spec operation expression");
        };
        assert_eq!(operation, SpecOperationIr::HasProperty);
        assert_eq!(operation.name(), "HasProperty");
        assert_eq!(operands, vec![target, key]);
    }

    #[test]
    fn operations_spec_create_data_property_or_throw_expr_records_operands() {
        let target = TypedExpr::from_info(
            ValueInfo::new(ValueKind::Object),
            ExprIr::Identifier(GLOBAL_THIS_NAME.to_string()),
        );
        let key = TypedExpr::from_info(
            ValueInfo::new(ValueKind::String),
            ExprIr::String("answer".to_string()),
        );
        let value = TypedExpr::from_info(
            ValueInfo::new(ValueKind::Number),
            ExprIr::Number(42.0f64.to_bits()),
        );
        let expr = TypedExpr::spec_create_data_property_or_throw(
            target.clone(),
            key.clone(),
            value.clone(),
        );

        assert_eq!(expr.kind, ValueKind::Undefined);
        let ExprIr::SpecOperation {
            operation,
            operands,
        } = expr.expr
        else {
            panic!("expected spec operation expression");
        };
        assert_eq!(operation, SpecOperationIr::CreateDataPropertyOrThrow);
        assert_eq!(operation.name(), "CreateDataPropertyOrThrow");
        assert_eq!(operands, vec![target, key, value]);
    }

    #[test]
    fn operations_spec_set_expr_records_operands() {
        let target = TypedExpr::from_info(
            ValueInfo::new(ValueKind::Object),
            ExprIr::Identifier(GLOBAL_THIS_NAME.to_string()),
        );
        let key = TypedExpr::from_info(
            ValueInfo::new(ValueKind::String),
            ExprIr::String("answer".to_string()),
        );
        let value = TypedExpr::from_info(
            ValueInfo::new(ValueKind::Number),
            ExprIr::Number(42.0f64.to_bits()),
        );
        let expr = TypedExpr::spec_set(target.clone(), key.clone(), value.clone());

        assert_eq!(expr.kind, ValueKind::Boolean);
        let ExprIr::SpecOperation {
            operation,
            operands,
        } = expr.expr
        else {
            panic!("expected spec operation expression");
        };
        assert_eq!(operation, SpecOperationIr::Set);
        assert_eq!(operation.name(), "Set");
        assert_eq!(operands, vec![target, key, value]);
    }

    #[test]
    fn operations_spec_delete_property_or_throw_expr_records_operands() {
        let target = TypedExpr::from_info(
            ValueInfo::new(ValueKind::Object),
            ExprIr::Identifier(GLOBAL_THIS_NAME.to_string()),
        );
        let key = TypedExpr::from_info(
            ValueInfo::new(ValueKind::String),
            ExprIr::String("answer".to_string()),
        );
        let expr = TypedExpr::spec_delete_property_or_throw(target.clone(), key.clone());

        assert_eq!(expr.kind, ValueKind::Boolean);
        let ExprIr::SpecOperation {
            operation,
            operands,
        } = expr.expr
        else {
            panic!("expected spec operation expression");
        };
        assert_eq!(operation, SpecOperationIr::DeletePropertyOrThrow);
        assert_eq!(operation.name(), "DeletePropertyOrThrow");
        assert_eq!(operands, vec![target, key]);
    }

    #[test]
    fn operations_spec_has_own_property_expr_records_target_and_key_operands() {
        let target = TypedExpr::from_info(
            ValueInfo::new(ValueKind::Object),
            ExprIr::Identifier(GLOBAL_THIS_NAME.to_string()),
        );
        let key = TypedExpr::from_info(
            ValueInfo::new(ValueKind::String),
            ExprIr::String("answer".to_string()),
        );
        let expr = TypedExpr::spec_has_own_property(target.clone(), key.clone());

        assert_eq!(expr.kind, ValueKind::Boolean);
        let ExprIr::SpecOperation {
            operation,
            operands,
        } = expr.expr
        else {
            panic!("expected spec operation expression");
        };
        assert_eq!(operation, SpecOperationIr::HasOwnProperty);
        assert_eq!(operation.name(), "HasOwnProperty");
        assert_eq!(operands, vec![target, key]);
    }

    #[test]
    fn operations_spec_get_method_expr_records_target_and_key_operands() {
        let target = TypedExpr::from_info(
            ValueInfo::new(ValueKind::Object),
            ExprIr::Identifier(GLOBAL_THIS_NAME.to_string()),
        );
        let key = TypedExpr::from_info(
            ValueInfo::new(ValueKind::String),
            ExprIr::String("answer".to_string()),
        );
        let expr = TypedExpr::spec_get_method(target.clone(), key.clone());

        assert_eq!(expr.kind, ValueKind::Dynamic);
        assert_eq!(
            expr.possible_kinds,
            KindSet::from_kind(ValueKind::Undefined).union(KindSet::from_kind(ValueKind::Function))
        );
        let ExprIr::SpecOperation {
            operation,
            operands,
        } = expr.expr
        else {
            panic!("expected spec operation expression");
        };
        assert_eq!(operation, SpecOperationIr::GetMethod);
        assert_eq!(operation.name(), "GetMethod");
        assert_eq!(operands, vec![target, key]);
    }

    #[test]
    fn operations_spec_call_expr_records_callee_this_and_args() {
        let callee = TypedExpr::from_info(
            ValueInfo::new(ValueKind::Function),
            ExprIr::FunctionValue(StandardBuiltinId::MathMax.function_id()),
        );
        let this_arg =
            TypedExpr::from_info(ValueInfo::new(ValueKind::Undefined), ExprIr::Undefined);
        let arg = TypedExpr::from_info(
            ValueInfo::new(ValueKind::Number),
            ExprIr::Number(1.0f64.to_bits()),
        );
        let expr = TypedExpr::spec_call(callee.clone(), this_arg.clone(), vec![arg.clone()]);

        assert_eq!(expr.kind, ValueKind::Dynamic);
        let ExprIr::SpecOperation {
            operation,
            operands,
        } = expr.expr
        else {
            panic!("expected spec operation expression");
        };
        assert_eq!(operation, SpecOperationIr::Call);
        assert_eq!(operation.name(), "Call");
        assert_eq!(operands, vec![callee, this_arg, arg]);
    }

    #[test]
    fn operations_spec_construct_expr_records_callee_and_args() {
        let callee = TypedExpr::from_info(
            ValueInfo::new(ValueKind::Function),
            ExprIr::FunctionValue(StandardBuiltinId::ArrayConstructor.function_id()),
        );
        let arg = TypedExpr::from_info(
            ValueInfo::new(ValueKind::Number),
            ExprIr::Number(1.0f64.to_bits()),
        );
        let expr = TypedExpr::spec_construct(callee.clone(), vec![arg.clone()]);

        assert_eq!(expr.kind, ValueKind::Dynamic);
        assert!(expr.possible_kinds.contains(ValueKind::Array));
        let ExprIr::SpecOperation {
            operation,
            operands,
        } = expr.expr
        else {
            panic!("expected spec operation expression");
        };
        assert_eq!(operation, SpecOperationIr::Construct);
        assert_eq!(operation.name(), "Construct");
        assert_eq!(operands, vec![callee, arg]);
    }

    #[test]
    fn heap_shape_property_read_follows_prototype_chain() {
        let proto = HeapShape::Object(ObjectShape {
            properties: BTreeMap::from([(
                "x".to_string(),
                ObjectShapeProperty::Data(ValueInfo::new(ValueKind::Number)),
            )]),
            ..ObjectShape::default()
        });
        let shape = HeapShape::Object(ObjectShape {
            prototype: Some(Box::new(proto)),
            ..ObjectShape::default()
        });
        assert_eq!(
            read_heap_shape_property(&shape, "x"),
            Some(ObjectShapeProperty::Data(ValueInfo::new(ValueKind::Number)))
        );
    }
}
