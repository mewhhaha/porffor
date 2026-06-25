use std::collections::{BTreeMap, BTreeSet};

use porffor_front::ParseGoal;

use crate::{
    ArithmeticBinaryOp, BindingMode, BitwiseBinaryOp, CallableToStringRepresentation,
    EqualityBinaryOp, HostBuiltinId, IrDiagnostic, IrDiagnosticKind, LogicalBinaryOp,
    LoweringStage, NumericUpdateOp, RelationalBinaryOp, SpecOperationIr, StandardBuiltinId,
    UnaryNumericOp, UpdateReturnMode, GLOBAL_THIS_NAME,
};

pub type FunctionId = String;
pub type PrivateNameId = u32;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectPropertyIr {
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
pub struct ClassFieldInitIr {
    pub key: Option<String>,
    pub private_name_id: Option<PrivateNameId>,
    pub init_function_id: Option<FunctionId>,
    pub placement: ClassMethodPlacementIr,
    pub is_private: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassStaticBlockIr {
    pub function_id: FunctionId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassDefinitionIr {
    pub name: Option<String>,
    pub constructor_function_id: FunctionId,
    pub explicit_constructor: bool,
    pub heritage_kind: ClassHeritageKind,
    pub heritage: Option<Box<TypedExpr>>,
    pub public_methods: Vec<ClassPublicMethodIr>,
    pub private_methods: Vec<ClassPrivateMethodIr>,
    pub fields: Vec<ClassFieldInitIr>,
    pub static_blocks: Vec<ClassStaticBlockIr>,
    pub private_name_ids: BTreeMap<String, PrivateNameId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropertyKeyIr {
    StaticString(String),
    StringExpr(Box<TypedExpr>),
    ArrayIndex(Box<TypedExpr>),
    ArrayLength,
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

    pub fn spec_to_boolean(argument: TypedExpr) -> Self {
        Self::from_info(
            ValueInfo::new(ValueKind::Boolean),
            ExprIr::SpecOperation {
                operation: SpecOperationIr::ToBoolean,
                operands: vec![argument],
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
    BigInt(u64),
    Symbol,
    String(String),
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
    },
    JsonParseStaticReviver {
        value: JsonStaticValueIr,
        reviver: Box<TypedExpr>,
    },
    Construct {
        callee: Box<TypedExpr>,
        args: Vec<TypedExpr>,
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
pub struct CapturedBindingIr {
    pub name: String,
    pub slot: u32,
    pub hops: u32,
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
    pub class_heritage_kind: ClassHeritageKind,
    pub is_static_class_member: bool,
    pub is_derived_constructor: bool,
    pub is_synthetic_default_derived_constructor: bool,
    pub super_constructor_target: Option<FunctionId>,
    pub uses_super: bool,
    pub this_before_super: bool,
    pub private_name_ids: BTreeMap<String, PrivateNameId>,
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
    },
    ForOfArray {
        mode: BindingMode,
        name: String,
        iterable: TypedExpr,
        body: Box<StatementIr>,
    },
    ForOfString {
        mode: BindingMode,
        name: String,
        iterable: TypedExpr,
        body: Box<StatementIr>,
    },
    ForOfIterator {
        mode: BindingMode,
        name: String,
        iterable: TypedExpr,
        body: Box<StatementIr>,
    },
    ForInArray {
        mode: BindingMode,
        name: String,
        target: TypedExpr,
        body: Box<StatementIr>,
    },
    ForInString {
        mode: BindingMode,
        name: String,
        target: TypedExpr,
        body: Box<StatementIr>,
    },
    ForInObject {
        mode: BindingMode,
        name: String,
        target: TypedExpr,
        body: Box<StatementIr>,
    },
    Switch {
        discriminant: TypedExpr,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockIr {
    pub statements: Vec<StatementIr>,
    pub result_kind: ValueKind,
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
            && self
                .diagnostics
                .iter()
                .all(|diagnostic| diagnostic.kind != IrDiagnosticKind::Unsupported)
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
                        .filter(|binding| matches!(binding.kind, ScriptGlobalBindingKind::BuiltinFunction(_)))
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
            StatementIr::Empty => {}
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
                cases,
            } => {
                self.switches += 1;
                self.visit_expr(discriminant);
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
            ExprIr::ArrayLiteral(elements) => {
                self.arrays += 1;
                for element in elements {
                    self.visit_expr(element);
                }
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
            ExprIr::Construct { callee, args } => {
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
                self.class_fields += class.fields.len();
                self.private_elements +=
                    class.fields.iter().filter(|field| field.is_private).count();
                self.private_elements += class.private_methods.len();
                self.static_blocks += class.static_blocks.len();
                if let Some(heritage) = &class.heritage {
                    self.visit_expr(heritage);
                }
                for method in &class.public_methods {
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
            ExprIr::Undefined
            | ExprIr::ArrayHole
            | ExprIr::Null
            | ExprIr::Boolean(_)
            | ExprIr::Number(_)
            | ExprIr::BigInt(_)
            | ExprIr::Symbol
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
