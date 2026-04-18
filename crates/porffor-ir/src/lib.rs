use std::collections::{BTreeMap, BTreeSet};

use boa_ast::property::{MethodDefinitionKind, PropertyName};
use boa_ast::{
    declaration::{Binding, LexicalDeclaration, VarDeclaration},
    expression::access::{PropertyAccess, PropertyAccessField},
    expression::literal::{
        ArrayLiteral, LiteralKind, ObjectLiteral, ObjectMethodDefinition, PropertyDefinition,
    },
    expression::operator::{
        assign::{AssignOp, AssignTarget},
        binary::{ArithmeticOp, BinaryOp, LogicalOp, RelationalOp},
        unary::UnaryOp,
        update::{UpdateOp, UpdateTarget},
    },
    expression::Expression,
    function::{
        ArrowFunction, FormalParameter, FormalParameterList, FunctionBody, FunctionDeclaration,
        FunctionExpression,
    },
    scope::Scope,
    statement::{
        iteration::{
            Break as AstBreak, Continue as AstContinue, DoWhileLoop, ForLoop, ForLoopInitializer,
            WhileLoop,
        },
        Block, If, Labelled as AstLabelled, LabelledItem, Return as AstReturn, Statement,
        Switch as AstSwitch,
    },
    Declaration, Script, Spanned, StatementListItem,
};
use boa_interner::Interner;
use boa_parser::{Parser, Source};
use porffor_front::{ParseGoal, SourceUnit};

const SCRIPT_OWNER_ID: &str = "$script";
pub const LEXICAL_THIS_NAME: &str = "$this";
pub const LEXICAL_ARGUMENTS_NAME: &str = "$arguments";

pub type FunctionId = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionFlavor {
    Ordinary,
    Arrow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoweringStage {
    ParsedSource,
    AstReparsed,
    ScriptIrBuilt,
    UnsupportedFeaturesRecorded,
    WasmReady,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrDiagnosticKind {
    Unsupported,
    Lowering,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrDiagnostic {
    pub kind: IrDiagnosticKind,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    Undefined,
    Null,
    Boolean,
    Number,
    String,
    Object,
    Array,
    Function,
    Arguments,
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
            Self::Object => "object",
            Self::Array => "array",
            Self::Function => "function",
            Self::Arguments => "arguments",
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
            Self::Object => 5,
            Self::Array => 6,
            Self::Function => 7,
            Self::Arguments => 8,
            Self::Dynamic => 9,
        }
    }

    pub const fn from_tag(tag: i32) -> Option<Self> {
        match tag {
            0 => Some(Self::Undefined),
            1 => Some(Self::Null),
            2 => Some(Self::Boolean),
            3 => Some(Self::Number),
            4 => Some(Self::String),
            5 => Some(Self::Object),
            6 => Some(Self::Array),
            7 => Some(Self::Function),
            8 => Some(Self::Arguments),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectPropertyIr {
    Data {
        key: String,
        value: TypedExpr,
        is_shorthand: bool,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueInfo {
    pub kind: ValueKind,
    pub heap_shape: Option<Box<HeapShape>>,
    pub function_targets: BTreeSet<FunctionId>,
}

impl ValueInfo {
    pub const fn undefined() -> Self {
        Self {
            kind: ValueKind::Undefined,
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
    pub properties: BTreeMap<String, ObjectShapeProperty>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ArrayShape {
    pub elements: Vec<ValueInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropertyKeyIr {
    StaticString(String),
    StringExpr(Box<TypedExpr>),
    ArrayIndex(Box<TypedExpr>),
    ArrayLength,
}

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicalBinaryOp {
    And,
    Or,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedExpr {
    pub kind: ValueKind,
    pub heap_shape: Option<Box<HeapShape>>,
    pub function_targets: BTreeSet<FunctionId>,
    pub expr: ExprIr,
}

impl TypedExpr {
    pub const fn undefined() -> Self {
        Self {
            kind: ValueKind::Undefined,
            heap_shape: None,
            function_targets: BTreeSet::new(),
            expr: ExprIr::Undefined,
        }
    }

    pub fn from_info(info: ValueInfo, expr: ExprIr) -> Self {
        Self {
            kind: info.kind,
            heap_shape: info.heap_shape,
            function_targets: info.function_targets,
            expr,
        }
    }

    pub fn value_info(&self) -> ValueInfo {
        ValueInfo {
            kind: self.kind,
            heap_shape: self.heap_shape.clone(),
            function_targets: self.function_targets.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExprIr {
    Undefined,
    Null,
    Boolean(bool),
    Number(u64),
    String(String),
    FunctionValue(FunctionId),
    This,
    Arguments,
    ObjectLiteral(Vec<ObjectPropertyIr>),
    ArrayLiteral(Vec<TypedExpr>),
    Identifier(String),
    AssignIdentifier {
        name: String,
        value: Box<TypedExpr>,
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
    UpdateIdentifier {
        name: String,
        op: NumericUpdateOp,
        return_mode: UpdateReturnMode,
    },
    CompoundAssignIdentifier {
        name: String,
        op: ArithmeticBinaryOp,
        value: Box<TypedExpr>,
    },
    UnaryNumber {
        op: UnaryNumericOp,
        expr: Box<TypedExpr>,
    },
    LogicalNot {
        expr: Box<TypedExpr>,
    },
    BinaryNumber {
        op: ArithmeticBinaryOp,
        lhs: Box<TypedExpr>,
        rhs: Box<TypedExpr>,
    },
    CompareNumber {
        op: RelationalBinaryOp,
        lhs: Box<TypedExpr>,
        rhs: Box<TypedExpr>,
    },
    StrictEquality {
        op: EqualityBinaryOp,
        lhs: Box<TypedExpr>,
        rhs: Box<TypedExpr>,
    },
    LogicalShortCircuit {
        op: LogicalBinaryOp,
        lhs: Box<TypedExpr>,
        rhs: Box<TypedExpr>,
    },
    CallNamed {
        name: String,
        args: Vec<TypedExpr>,
    },
    CallIndirect {
        callee: Box<TypedExpr>,
        args: Vec<TypedExpr>,
    },
    CallMethod {
        receiver: Box<TypedExpr>,
        key: PropertyKeyIr,
        args: Vec<TypedExpr>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForInitIr {
    Lexical {
        mode: BindingMode,
        name: String,
        init: TypedExpr,
    },
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
    pub flavor: FunctionFlavor,
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
    Switch {
        discriminant: TypedExpr,
        cases: Vec<SwitchCaseIr>,
    },
    Labelled {
        labels: Vec<String>,
        statement: Box<StatementIr>,
    },
    Debugger,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptIr {
    pub functions: Vec<FunctionIr>,
    pub body: BlockIr,
    pub owned_env_bindings: Vec<OwnedEnvBindingIr>,
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
                    "script statements={} result={} functions={} nested_functions={} function_exprs={} arrow_functions={} named_function_exprs={} closures={} captures={} lexical_this_captures={} lexical_arguments_captures={} default_params={} rest_params={} arguments_uses={} calls={} indirect_calls={} method_calls={} returns={} lets={} consts={} vars={} blocks={} ifs={} whiles={} do_whiles={} fors={} switches={} labels={} debuggers={} breaks={} continues={} objects={} object_shorthands={} object_methods={} object_getters={} object_setters={} arrays={} property_reads={} property_writes={} array_lengths={} heap_shapes={} function_values={} this_reads={} assigns={} prefix_updates={} postfix_updates={} compound_assigns={}",
                    counts.statements,
                    script.result_kind().as_str(),
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
                    counts.returns,
                    counts.lets,
                    counts.consts,
                    counts.vars,
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
                    counts.assignments,
                    counts.prefix_updates,
                    counts.postfix_updates,
                    counts.compound_assignments
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
    returns: usize,
    lets: usize,
    consts: usize,
    vars: usize,
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
    assignments: usize,
    prefix_updates: usize,
    postfix_updates: usize,
    compound_assignments: usize,
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
        if expr.kind == ValueKind::Function {
            self.function_values += 1;
        }
        match &expr.expr {
            ExprIr::AssignIdentifier { value, .. } => {
                self.assignments += 1;
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
                        ObjectPropertyIr::Method { function, .. } => {
                            self.object_methods += 1;
                            self.visit_expr(function);
                        }
                        ObjectPropertyIr::Getter { function, .. } => {
                            self.object_getters += 1;
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
            ExprIr::PropertyRead { target, key } => {
                self.property_reads += 1;
                if matches!(key, PropertyKeyIr::ArrayLength) {
                    self.array_lengths += 1;
                }
                self.visit_expr(target);
                self.visit_property_key(key);
            }
            ExprIr::PropertyWrite { target, key, value } => {
                self.property_writes += 1;
                self.visit_expr(target);
                self.visit_property_key(key);
                self.visit_expr(value);
            }
            ExprIr::UpdateIdentifier { return_mode, .. } => match return_mode {
                UpdateReturnMode::Prefix => self.prefix_updates += 1,
                UpdateReturnMode::Postfix => self.postfix_updates += 1,
            },
            ExprIr::CompoundAssignIdentifier { value, .. } => {
                self.compound_assignments += 1;
                self.visit_expr(value);
            }
            ExprIr::UnaryNumber { expr, .. } | ExprIr::LogicalNot { expr } => self.visit_expr(expr),
            ExprIr::BinaryNumber { lhs, rhs, .. }
            | ExprIr::CompareNumber { lhs, rhs, .. }
            | ExprIr::StrictEquality { lhs, rhs, .. }
            | ExprIr::LogicalShortCircuit { lhs, rhs, .. } => {
                self.visit_expr(lhs);
                self.visit_expr(rhs);
            }
            ExprIr::CallNamed { args, .. } => {
                self.calls += 1;
                for arg in args {
                    self.visit_expr(arg);
                }
            }
            ExprIr::Arguments => {
                self.arguments_uses += 1;
            }
            ExprIr::CallIndirect { callee, args } => {
                self.calls += 1;
                self.indirect_calls += 1;
                self.visit_expr(callee);
                for arg in args {
                    self.visit_expr(arg);
                }
            }
            ExprIr::CallMethod { receiver, key, args } => {
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
            ExprIr::Undefined
            | ExprIr::Null
            | ExprIr::Boolean(_)
            | ExprIr::Number(_)
            | ExprIr::String(_)
            | ExprIr::FunctionValue(_)
            | ExprIr::Identifier(_) => {}
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

pub fn lower(source: &SourceUnit) -> ProgramIr {
    let mut program = ProgramIr {
        goal: source.goal,
        stages: vec![LoweringStage::ParsedSource],
        source_len: source.source_text.len(),
        invariants: vec![
            "direct-js-to-wasm-only",
            "no-shipped-interpreter-in-wasm",
            "spec-ir-is-semantic-source-of-truth",
        ],
        diagnostics: Vec::new(),
        script: None,
    };

    if source.goal != ParseGoal::Script {
        program.diagnostics.push(IrDiagnostic {
            kind: IrDiagnosticKind::Unsupported,
            message: "unsupported in porffor wasm-aot first slice: modules".to_string(),
        });
        program
            .stages
            .push(LoweringStage::UnsupportedFeaturesRecorded);
        return program;
    }

    match reparse_script(source) {
        Ok((script, interner)) => {
            program.stages.push(LoweringStage::AstReparsed);
            let analysis = AnalysisBuilder::default().finish(&script, &interner);
            let lowered =
                ScriptLowerer::new(&interner, &analysis, SCRIPT_OWNER_ID.to_string()).lower(&script);
            program.script = Some(ScriptIr {
                functions: lowered.functions,
                body: lowered.body,
                owned_env_bindings: lowered.owned_env_bindings,
            });
            program.stages.push(LoweringStage::ScriptIrBuilt);
            if !lowered.diagnostics.is_empty() {
                program
                    .stages
                    .push(LoweringStage::UnsupportedFeaturesRecorded);
            } else {
                program.stages.push(LoweringStage::WasmReady);
            }
            program.diagnostics = lowered.diagnostics;
        }
        Err(message) => {
            program.diagnostics.push(IrDiagnostic {
                kind: IrDiagnosticKind::Lowering,
                message,
            });
        }
    }

    program
}

fn reparse_script(source: &SourceUnit) -> Result<(Script, Interner), String> {
    let mut interner = Interner::default();
    let scope = Scope::new_global();
    let parser_source = if let Some(filename) = &source.filename {
        Source::from_bytes(source.source_text.as_bytes()).with_path(std::path::Path::new(filename))
    } else {
        Source::from_bytes(source.source_text.as_bytes())
    };
    let script = Parser::new(parser_source)
        .parse_script(&scope, &mut interner)
        .map_err(|err| format!("lowering reparse failed: {err}"))?;
    Ok((script, interner))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BindingInfo {
    mode: BindingMode,
    kind: ValueKind,
    heap_shape: Option<Box<HeapShape>>,
    function_targets: BTreeSet<FunctionId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VarBindingInfo {
    kind: ValueKind,
    heap_shape: Option<Box<HeapShape>>,
    function_targets: BTreeSet<FunctionId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LabelTargetKind {
    Breakable,
    Loop,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ActiveLabel {
    name: String,
    kind: LabelTargetKind,
}

struct LoweredScript {
    functions: Vec<FunctionIr>,
    body: BlockIr,
    owned_env_bindings: Vec<OwnedEnvBindingIr>,
    diagnostics: Vec<IrDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FunctionSignature {
    id: FunctionId,
    flavor: FunctionFlavor,
    params: Vec<FunctionParamSignature>,
    return_kind: ValueKind,
    return_shape: Option<Box<HeapShape>>,
    return_targets: BTreeSet<FunctionId>,
    this_info: ValueInfo,
    this_observed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FunctionParamSignature {
    kind: ValueKind,
    has_default: bool,
    is_rest: bool,
}

#[derive(Debug, Clone)]
struct PendingFunction<'a> {
    id: FunctionId,
    name: String,
    flavor: FunctionFlavor,
    self_binding_name: Option<String>,
    parameters: &'a FormalParameterList,
    body: &'a FunctionBody,
    is_expression: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CaptureBindingPlan {
    owner_id: String,
    slot: u32,
    hops: u32,
}

#[derive(Debug, Clone)]
struct OwnerPlan {
    flavor: FunctionFlavor,
    parent_owner_id: Option<String>,
    root_bindings: BTreeSet<String>,
    function_bindings: BTreeMap<String, FunctionId>,
    owned_env_slots: BTreeMap<String, u32>,
}

#[derive(Debug, Clone)]
struct FunctionPlan<'a> {
    id: FunctionId,
    name: String,
    flavor: FunctionFlavor,
    self_binding_name: Option<String>,
    parent_owner_id: String,
    parameters: &'a FormalParameterList,
    body: &'a FunctionBody,
    is_expression: bool,
    root_functions: Vec<PendingFunction<'a>>,
    captures: BTreeMap<String, CaptureBindingPlan>,
}

#[derive(Debug, Clone)]
struct Analysis<'a> {
    owner_plans: BTreeMap<String, OwnerPlan>,
    function_plans: BTreeMap<FunctionId, FunctionPlan<'a>>,
    function_expr_ids: BTreeMap<String, FunctionId>,
    function_order: Vec<FunctionId>,
    script_root_functions: Vec<PendingFunction<'a>>,
}

#[derive(Default)]
struct AnalysisBuilder<'a> {
    owner_plans: BTreeMap<String, OwnerPlan>,
    function_plans: BTreeMap<FunctionId, FunctionPlan<'a>>,
    function_expr_ids: BTreeMap<String, FunctionId>,
    function_free_refs: BTreeMap<FunctionId, BTreeSet<String>>,
    function_order: Vec<FunctionId>,
    next_function_id: usize,
}

impl<'a> AnalysisBuilder<'a> {
    fn finish(mut self, script: &'a Script, interner: &'a Interner) -> Analysis<'a> {
        let script_root_functions =
            self.collect_root_functions(interner, script.statements().statements());
        self.owner_plans.insert(
            SCRIPT_OWNER_ID.to_string(),
            OwnerPlan {
                flavor: FunctionFlavor::Ordinary,
                parent_owner_id: None,
                root_bindings: self.collect_owner_bindings(
                    interner,
                    &[],
                    None,
                    false,
                    false,
                    script.statements().statements(),
                    &script_root_functions,
                ),
                function_bindings: script_root_functions
                    .iter()
                    .map(|function| (function.name.clone(), function.id.clone()))
                    .collect(),
                    owned_env_slots: BTreeMap::new(),
            },
        );
        self.scan_owner_items(
            SCRIPT_OWNER_ID,
            script.statements().statements(),
            interner,
            None,
        );
        for function in script_root_functions.iter().cloned() {
            self.collect_function_plan(function, SCRIPT_OWNER_ID.to_string(), interner);
        }
        self.finalize_capture_plans();
        Analysis {
            owner_plans: self.owner_plans,
            function_plans: self.function_plans,
            function_expr_ids: self.function_expr_ids,
            function_order: self.function_order,
            script_root_functions,
        }
    }

    fn alloc_function_id(&mut self) -> FunctionId {
        let id = format!("f{}", self.next_function_id);
        self.next_function_id += 1;
        id
    }

    fn collect_root_functions(
        &mut self,
        interner: &Interner,
        items: &'a [StatementListItem],
    ) -> Vec<PendingFunction<'a>> {
        let mut functions = Vec::new();
        for item in items {
            let StatementListItem::Declaration(declaration) = item else {
                continue;
            };
            let Declaration::FunctionDeclaration(function) = declaration.as_ref() else {
                continue;
            };
            let name = function_name(interner, function, None);
            functions.push(PendingFunction {
                id: self.alloc_function_id(),
                name,
                flavor: FunctionFlavor::Ordinary,
                self_binding_name: Some(
                    interner.resolve_expect(function.name().sym()).to_string(),
                ),
                parameters: function.parameters(),
                body: function.body(),
                is_expression: false,
            });
        }
        functions
    }

    fn collect_function_plan(
        &mut self,
        function: PendingFunction<'a>,
        parent_owner_id: String,
        interner: &'a Interner,
    ) {
        let owner_id = function.id.clone();
        let root_functions = self.collect_root_functions(interner, function.body.statements());
        let simple_parameter_names = if function.flavor == FunctionFlavor::Ordinary {
            collect_simple_parameter_names(interner, function.parameters)
        } else {
            Vec::new()
        };
        let mut owned_env_slots = BTreeMap::new();
        for (slot, name) in simple_parameter_names.iter().enumerate() {
            owned_env_slots.insert(name.clone(), slot as u32);
        }
        self.owner_plans.insert(
            owner_id.clone(),
            OwnerPlan {
                flavor: function.flavor,
                parent_owner_id: Some(parent_owner_id.clone()),
                root_bindings: self.collect_owner_bindings(
                    interner,
                    function.parameters.as_ref(),
                    function.self_binding_name.as_deref(),
                    function.flavor == FunctionFlavor::Ordinary,
                    function.flavor == FunctionFlavor::Ordinary,
                    function.body.statements(),
                    &root_functions,
                ),
                function_bindings: root_functions
                    .iter()
                    .map(|nested| (nested.name.clone(), nested.id.clone()))
                    .collect(),
                owned_env_slots,
            },
        );
        self.scan_owner_items(
            &owner_id,
            function.body.statements(),
            interner,
            Some(function.name.as_str()),
        );
        self.function_plans.insert(
            owner_id.clone(),
            FunctionPlan {
                id: owner_id.clone(),
                name: function.name.clone(),
                flavor: function.flavor,
                self_binding_name: function.self_binding_name.clone(),
                parent_owner_id,
                parameters: function.parameters,
                body: function.body,
                is_expression: function.is_expression,
                root_functions: root_functions.clone(),
                captures: BTreeMap::new(),
            },
        );
        for nested in root_functions {
            self.collect_function_plan(nested, owner_id.clone(), interner);
        }
        self.function_order.push(owner_id);
    }

    fn collect_owner_bindings(
        &self,
        interner: &Interner,
        params: &[FormalParameter],
        self_name: Option<&str>,
        has_own_this: bool,
        has_own_arguments: bool,
        items: &'a [StatementListItem],
        root_functions: &[PendingFunction<'a>],
    ) -> BTreeSet<String> {
        let mut bindings = BTreeSet::new();
        if let Some(self_name) = self_name {
            bindings.insert(self_name.to_string());
        }
        if has_own_this {
            bindings.insert(LEXICAL_THIS_NAME.to_string());
        }
        if has_own_arguments {
            bindings.insert(LEXICAL_ARGUMENTS_NAME.to_string());
        }
        for parameter in params {
            if let Binding::Identifier(identifier) = parameter.variable().binding() {
                bindings.insert(interner.resolve_expect(identifier.sym()).to_string());
            }
        }
        for function in root_functions {
            bindings.insert(function.name.clone());
        }
        self.collect_declared_bindings_from_items(interner, items, &mut bindings);
        bindings
    }

    fn collect_declared_bindings_from_items(
        &self,
        interner: &Interner,
        items: &'a [StatementListItem],
        bindings: &mut BTreeSet<String>,
    ) {
        for item in items {
            match item {
                StatementListItem::Statement(statement) => {
                    self.collect_declared_bindings_from_statement(interner, statement, bindings);
                }
                StatementListItem::Declaration(declaration) => match declaration.as_ref() {
                    Declaration::Lexical(lexical) => {
                        self.collect_declared_bindings_from_lexical(interner, lexical, bindings);
                    }
                    Declaration::FunctionDeclaration(_) => {}
                    _ => {}
                },
            }
        }
    }

    fn collect_declared_bindings_from_statement(
        &self,
        interner: &Interner,
        statement: &'a Statement,
        bindings: &mut BTreeSet<String>,
    ) {
        match statement {
            Statement::Block(block) => {
                self.collect_declared_bindings_from_items(
                    interner,
                    block.statement_list().statements(),
                    bindings,
                )
            }
            Statement::If(if_statement) => {
                self.collect_declared_bindings_from_statement(interner, if_statement.body(), bindings);
                if let Some(else_node) = if_statement.else_node() {
                    self.collect_declared_bindings_from_statement(interner, else_node, bindings);
                }
            }
            Statement::WhileLoop(while_loop) => {
                self.collect_declared_bindings_from_statement(interner, while_loop.body(), bindings);
            }
            Statement::DoWhileLoop(do_while) => {
                self.collect_declared_bindings_from_statement(interner, do_while.body(), bindings);
            }
            Statement::ForLoop(for_loop) => {
                if let Some(init) = for_loop.init() {
                    match init {
                        ForLoopInitializer::Var(var) => {
                            self.collect_declared_bindings_from_var(interner, var, bindings);
                        }
                        ForLoopInitializer::Lexical(lexical) => {
                            self.collect_declared_bindings_from_lexical(
                                interner,
                                lexical.declaration(),
                                bindings,
                            );
                        }
                        ForLoopInitializer::Expression(_) => {}
                    }
                }
                self.collect_declared_bindings_from_statement(interner, for_loop.body(), bindings);
            }
            Statement::Switch(switch) => {
                for case in switch.cases() {
                    self.collect_declared_bindings_from_items(
                        interner,
                        case.body().statements(),
                        bindings,
                    );
                }
            }
            Statement::Labelled(labelled) => {
                if let Some(statement) = ScriptLowerer::labelled_base_statement(labelled) {
                    self.collect_declared_bindings_from_statement(interner, statement, bindings);
                }
            }
            Statement::Var(var) => self.collect_declared_bindings_from_var(interner, var, bindings),
            Statement::Expression(_)
            | Statement::Empty
            | Statement::Break(_)
            | Statement::Continue(_)
            | Statement::Debugger
            | Statement::ForInLoop(_)
            | Statement::ForOfLoop(_)
            | Statement::Return(_)
            | Statement::Throw(_)
            | Statement::Try(_)
            | Statement::With(_) => {}
        }
    }

    fn collect_declared_bindings_from_var(
        &self,
        interner: &Interner,
        declaration: &'a VarDeclaration,
        bindings: &mut BTreeSet<String>,
    ) {
        for declarator in declaration.0.as_ref() {
            if let Binding::Identifier(identifier) = declarator.binding() {
                bindings.insert(interner.resolve_expect(identifier.sym()).to_string());
            }
        }
    }

    fn collect_declared_bindings_from_lexical(
        &self,
        interner: &Interner,
        declaration: &'a LexicalDeclaration,
        bindings: &mut BTreeSet<String>,
    ) {
        let list = match declaration {
            LexicalDeclaration::Let(list) | LexicalDeclaration::Const(list) => list,
            LexicalDeclaration::Using(_) | LexicalDeclaration::AwaitUsing(_) => return,
        };
        for declarator in list.as_ref() {
            if let Binding::Identifier(identifier) = declarator.binding() {
                bindings.insert(interner.resolve_expect(identifier.sym()).to_string());
            }
        }
    }

    fn scan_owner_items(
        &mut self,
        owner_id: &str,
        items: &'a [StatementListItem],
        interner: &'a Interner,
        self_name: Option<&str>,
    ) {
        let mut refs = BTreeSet::new();
        for item in items {
            self.scan_item(owner_id, item, interner, self_name, &mut refs);
        }
        if owner_id != SCRIPT_OWNER_ID {
            self.function_free_refs
                .insert(owner_id.to_string(), refs);
        }
    }

    fn scan_item(
        &mut self,
        owner_id: &str,
        item: &'a StatementListItem,
        interner: &'a Interner,
        self_name: Option<&str>,
        refs: &mut BTreeSet<String>,
    ) {
        match item {
            StatementListItem::Statement(statement) => {
                self.scan_statement(owner_id, statement, interner, self_name, refs);
            }
            StatementListItem::Declaration(declaration) => match declaration.as_ref() {
                Declaration::Lexical(lexical) => {
                    let list = match lexical {
                        LexicalDeclaration::Let(list) | LexicalDeclaration::Const(list) => list,
                        LexicalDeclaration::Using(_) | LexicalDeclaration::AwaitUsing(_) => return,
                    };
                    for declarator in list.as_ref() {
                        if let Some(init) = declarator.init() {
                            self.scan_expression(owner_id, init, interner, self_name, refs);
                        }
                    }
                }
                Declaration::FunctionDeclaration(_) => {}
                _ => {}
            },
        }
    }

    fn scan_statement(
        &mut self,
        owner_id: &str,
        statement: &'a Statement,
        interner: &'a Interner,
        self_name: Option<&str>,
        refs: &mut BTreeSet<String>,
    ) {
        match statement {
            Statement::Expression(expression) => {
                self.scan_expression(owner_id, expression, interner, self_name, refs);
            }
            Statement::Block(block) => {
                for item in block.statement_list().statements() {
                    self.scan_item(owner_id, item, interner, self_name, refs);
                }
            }
            Statement::If(if_statement) => {
                self.scan_expression(owner_id, if_statement.cond(), interner, self_name, refs);
                self.scan_statement(owner_id, if_statement.body(), interner, self_name, refs);
                if let Some(else_node) = if_statement.else_node() {
                    self.scan_statement(owner_id, else_node, interner, self_name, refs);
                }
            }
            Statement::WhileLoop(while_loop) => {
                self.scan_expression(owner_id, while_loop.condition(), interner, self_name, refs);
                self.scan_statement(owner_id, while_loop.body(), interner, self_name, refs);
            }
            Statement::DoWhileLoop(do_while) => {
                self.scan_statement(owner_id, do_while.body(), interner, self_name, refs);
                self.scan_expression(owner_id, do_while.cond(), interner, self_name, refs);
            }
            Statement::ForLoop(for_loop) => {
                if let Some(init) = for_loop.init() {
                    match init {
                        ForLoopInitializer::Expression(expr) => {
                            self.scan_expression(owner_id, expr, interner, self_name, refs);
                        }
                        ForLoopInitializer::Var(var) => {
                            for declarator in var.0.as_ref() {
                                if let Some(init) = declarator.init() {
                                    self.scan_expression(owner_id, init, interner, self_name, refs);
                                }
                            }
                        }
                        ForLoopInitializer::Lexical(lexical) => {
                            let declaration = lexical.declaration();
                            let list = match declaration {
                                LexicalDeclaration::Let(list) | LexicalDeclaration::Const(list) => {
                                    list
                                }
                                LexicalDeclaration::Using(_)
                                | LexicalDeclaration::AwaitUsing(_) => return,
                            };
                            for declarator in list.as_ref() {
                                if let Some(init) = declarator.init() {
                                    self.scan_expression(owner_id, init, interner, self_name, refs);
                                }
                            }
                        }
                    }
                }
                if let Some(condition) = for_loop.condition() {
                    self.scan_expression(owner_id, condition, interner, self_name, refs);
                }
                if let Some(update) = for_loop.final_expr() {
                    self.scan_expression(owner_id, update, interner, self_name, refs);
                }
                self.scan_statement(owner_id, for_loop.body(), interner, self_name, refs);
            }
            Statement::Switch(switch) => {
                self.scan_expression(owner_id, switch.val(), interner, self_name, refs);
                for case in switch.cases() {
                    if let Some(condition) = case.condition() {
                        self.scan_expression(owner_id, condition, interner, self_name, refs);
                    }
                    for item in case.body().statements() {
                        self.scan_item(owner_id, item, interner, self_name, refs);
                    }
                }
            }
            Statement::Labelled(labelled) => {
                if let Some(statement) = ScriptLowerer::labelled_base_statement(labelled) {
                    self.scan_statement(owner_id, statement, interner, self_name, refs);
                }
            }
            Statement::Var(var) => {
                for declarator in var.0.as_ref() {
                    if let Some(init) = declarator.init() {
                        self.scan_expression(owner_id, init, interner, self_name, refs);
                    }
                }
            }
            Statement::Return(ret) => {
                if let Some(target) = ret.target() {
                    self.scan_expression(owner_id, target, interner, self_name, refs);
                }
            }
            Statement::Break(_)
            | Statement::Continue(_)
            | Statement::Debugger
            | Statement::Empty
            | Statement::ForInLoop(_)
            | Statement::ForOfLoop(_)
            | Statement::Throw(_)
            | Statement::Try(_)
            | Statement::With(_) => {}
        }
    }

    fn scan_expression(
        &mut self,
        owner_id: &str,
        expression: &'a Expression,
        interner: &'a Interner,
        self_name: Option<&str>,
        refs: &mut BTreeSet<String>,
    ) {
        match expression {
            Expression::Identifier(identifier) => {
                let name = interner.resolve_expect(identifier.sym()).to_string();
                if name == "arguments" {
                    let owner = self.owner_plans.get(owner_id);
                    let source_binds_arguments = owner
                        .is_some_and(|owner| owner.root_bindings.contains("arguments"));
                    if !source_binds_arguments {
                        if owner.is_some_and(|owner| owner.flavor == FunctionFlavor::Arrow) {
                            refs.insert(LEXICAL_ARGUMENTS_NAME.to_string());
                        } else if owner_id != SCRIPT_OWNER_ID {
                            refs.insert(LEXICAL_ARGUMENTS_NAME.to_string());
                        } else {
                            refs.insert(name);
                        }
                        return;
                    }
                }
                refs.insert(name);
            }
            Expression::Parenthesized(expression) => {
                self.scan_expression(owner_id, expression.expression(), interner, self_name, refs);
            }
            Expression::ArrayLiteral(array) => {
                for element in array.as_ref().iter().flatten() {
                    self.scan_expression(owner_id, element, interner, self_name, refs);
                }
            }
            Expression::ObjectLiteral(object) => {
                for property in object.properties() {
                    match property {
                        PropertyDefinition::Property(_, value) => {
                            self.scan_expression(owner_id, value, interner, self_name, refs);
                        }
                        PropertyDefinition::SpreadObject(value) => {
                            self.scan_expression(owner_id, value, interner, self_name, refs);
                        }
                        PropertyDefinition::MethodDefinition(method) => {
                            let key = object_method_key(method);
                            if !self.function_expr_ids.contains_key(&key) {
                                let id = self.alloc_function_id();
                                self.function_expr_ids.insert(key, id.clone());
                                let name = method
                                    .name()
                                    .prop_name()
                                    .map(|identifier| {
                                        interner.resolve_expect(identifier.sym()).to_string()
                                    })
                                    .unwrap_or_else(|| "<method>".to_string());
                                let pending = PendingFunction {
                                    id,
                                    name,
                                    flavor: FunctionFlavor::Ordinary,
                                    self_binding_name: None,
                                    parameters: method.parameters(),
                                    body: method.body(),
                                    is_expression: true,
                                };
                                self.collect_function_plan(pending, owner_id.to_string(), interner);
                            }
                        }
                        PropertyDefinition::IdentifierReference(identifier) => {
                            refs.insert(interner.resolve_expect(identifier.sym()).to_string());
                        }
                        PropertyDefinition::CoverInitializedName(_, value) => {
                            self.scan_expression(owner_id, value, interner, self_name, refs);
                        }
                    }
                }
            }
            Expression::Unary(unary) => {
                self.scan_expression(owner_id, unary.target(), interner, self_name, refs);
            }
            Expression::Binary(binary) => {
                self.scan_expression(owner_id, binary.lhs(), interner, self_name, refs);
                self.scan_expression(owner_id, binary.rhs(), interner, self_name, refs);
            }
            Expression::Assign(assign) => {
                match assign.lhs() {
                    AssignTarget::Identifier(identifier) => {
                        refs.insert(interner.resolve_expect(identifier.sym()).to_string());
                    }
                    AssignTarget::Access(access) => {
                        self.scan_property_access(owner_id, access, interner, self_name, refs);
                    }
                    _ => {}
                }
                self.scan_expression(owner_id, assign.rhs(), interner, self_name, refs);
            }
            Expression::Update(update) => {
                if let UpdateTarget::Identifier(identifier) = update.target() {
                    refs.insert(interner.resolve_expect(identifier.sym()).to_string());
                }
            }
            Expression::Call(call) => {
                self.scan_expression(owner_id, call.function(), interner, self_name, refs);
                for arg in call.args() {
                    self.scan_expression(owner_id, arg, interner, self_name, refs);
                }
            }
            Expression::PropertyAccess(access) => {
                self.scan_property_access(owner_id, access, interner, self_name, refs);
            }
            Expression::FunctionExpression(function) => {
                let key = function_expression_key(function);
                if !self.function_expr_ids.contains_key(&key) {
                    let id = self.alloc_function_id();
                    self.function_expr_ids.insert(key, id.clone());
                    let self_binding_name = function
                        .name()
                        .map(|identifier| interner.resolve_expect(identifier.sym()).to_string());
                    let pending = PendingFunction {
                        id,
                        name: self_binding_name
                            .clone()
                            .unwrap_or_else(|| "<anonymous>".to_string()),
                        flavor: FunctionFlavor::Ordinary,
                        self_binding_name,
                        parameters: function.parameters(),
                        body: function.body(),
                        is_expression: true,
                    };
                    self.collect_function_plan(pending, owner_id.to_string(), interner);
                }
            }
            Expression::ArrowFunction(function) => {
                let key = arrow_function_key(function);
                if !self.function_expr_ids.contains_key(&key) {
                    let id = self.alloc_function_id();
                    self.function_expr_ids.insert(key, id.clone());
                    let pending = PendingFunction {
                        id,
                        name: function
                            .name()
                            .map(|identifier| interner.resolve_expect(identifier.sym()).to_string())
                            .unwrap_or_else(|| "<arrow>".to_string()),
                        flavor: FunctionFlavor::Arrow,
                        self_binding_name: None,
                        parameters: function.parameters(),
                        body: function.body(),
                        is_expression: true,
                    };
                    self.collect_function_plan(pending, owner_id.to_string(), interner);
                }
            }
            Expression::This(_) => {
                if self
                    .owner_plans
                    .get(owner_id)
                    .is_some_and(|owner| owner.flavor == FunctionFlavor::Arrow)
                {
                    refs.insert(LEXICAL_THIS_NAME.to_string());
                }
            }
            Expression::AsyncArrowFunction(_)
            | Expression::Literal(_)
            | Expression::RegExpLiteral(_)
            | Expression::Spread(_)
            | Expression::GeneratorExpression(_)
            | Expression::AsyncFunctionExpression(_)
            | Expression::AsyncGeneratorExpression(_)
            | Expression::ClassExpression(_)
            | Expression::TemplateLiteral(_)
            | Expression::New(_)
            | Expression::SuperCall(_)
            | Expression::ImportCall(_)
            | Expression::Optional(_)
            | Expression::TaggedTemplate(_)
            | Expression::NewTarget(_)
            | Expression::ImportMeta(_)
            | Expression::BinaryInPrivate(_)
            | Expression::Conditional(_)
            | Expression::Await(_)
            | Expression::Yield(_)
            | Expression::FormalParameterList(_)
            | Expression::Debugger => {}
        }
        let _ = self_name;
    }

    fn scan_property_access(
        &mut self,
        owner_id: &str,
        access: &'a PropertyAccess,
        interner: &'a Interner,
        self_name: Option<&str>,
        refs: &mut BTreeSet<String>,
    ) {
        let PropertyAccess::Simple(access) = access else {
            return;
        };
        self.scan_expression(owner_id, access.target(), interner, self_name, refs);
        if let PropertyAccessField::Expr(expr) = access.field() {
            self.scan_expression(owner_id, expr, interner, self_name, refs);
        }
    }

    fn finalize_capture_plans(&mut self) {
        let mut owned_names = BTreeMap::<String, BTreeSet<String>>::new();
        let function_ids = self.function_order.clone();
        for function_id in function_ids {
            let Some(function) = self.function_plans.get(&function_id).cloned() else {
                continue;
            };
            let local_bindings = self
                .owner_plans
                .get(&function.id)
                .map(|owner| owner.root_bindings.clone())
                .unwrap_or_default();
            let free_refs = self
                .function_free_refs
                .get(&function.id)
                .cloned()
                .unwrap_or_default();
            let mut captures = BTreeMap::new();
            for name in free_refs {
                if local_bindings.contains(&name) {
                    continue;
                }
                let Some(owner_id) = self.resolve_capture_owner(&function.parent_owner_id, &name) else {
                    continue;
                };
                owned_names
                    .entry(owner_id.clone())
                    .or_default()
                    .insert(name.clone());
                captures.insert(
                    name,
                    CaptureBindingPlan {
                        owner_id,
                        slot: 0,
                        hops: 0,
                    },
                );
            }
            if let Some(plan) = self.function_plans.get_mut(&function.id) {
                plan.captures = captures;
            }
        }

        for (owner_id, names) in owned_names {
            let Some(owner) = self.owner_plans.get_mut(&owner_id) else {
                continue;
            };
            let mut next_slot = owner
                .owned_env_slots
                .values()
                .copied()
                .max()
                .map(|slot| slot + 1)
                .unwrap_or(0);
            for name in names {
                owner.owned_env_slots.entry(name).or_insert_with(|| {
                    let slot = next_slot;
                    next_slot += 1;
                    slot
                });
            }
        }

        let function_ids = self.function_order.clone();
        for function_id in function_ids {
            let Some(function) = self.function_plans.get(&function_id).cloned() else {
                continue;
            };
            let mut captures = function.captures;
            for capture in captures.values_mut() {
                if let Some(owner) = self.owner_plans.get(&capture.owner_id) {
                    capture.slot = *owner
                        .owned_env_slots
                        .get(
                            self.function_plans[&function_id]
                                .captures
                                .iter()
                                .find(|(_, plan)| plan.owner_id == capture.owner_id && plan.slot == 0)
                                .map(|(name, _)| name)
                                .unwrap_or_else(|| panic!("capture binding should exist")),
                        )
                        .unwrap_or(&0);
                }
            }
            let names: Vec<String> = captures.keys().cloned().collect();
            for name in names {
                if let Some(capture) = captures.get_mut(&name) {
                    capture.hops = self.capture_hops(&function.id, &capture.owner_id);
                    capture.slot = self.owner_plans[&capture.owner_id].owned_env_slots[&name];
                }
            }
            if let Some(plan) = self.function_plans.get_mut(&function_id) {
                plan.captures = captures;
            }
        }
    }

    fn resolve_capture_owner(&self, start_owner_id: &str, name: &str) -> Option<String> {
        let mut owner_id = Some(start_owner_id.to_string());
        while let Some(current) = owner_id {
            let owner = self.owner_plans.get(&current)?;
            if owner.root_bindings.contains(name) {
                return Some(current);
            }
            owner_id = owner.parent_owner_id.clone();
        }
        None
    }

    fn capture_hops(&self, current_owner_id: &str, target_owner_id: &str) -> u32 {
        let mut hops = 0;
        let mut env_owner = self.effective_env_owner(current_owner_id);
        while let Some(current) = env_owner {
            if current == target_owner_id {
                return hops;
            }
            env_owner = self.next_env_owner(&current);
            hops += 1;
        }
        0
    }

    fn effective_env_owner(&self, owner_id: &str) -> Option<String> {
        let mut current = Some(owner_id.to_string());
        while let Some(owner_id) = current {
            let owner = self.owner_plans.get(&owner_id)?;
            if !owner.owned_env_slots.is_empty() {
                return Some(owner_id);
            }
            current = owner.parent_owner_id.clone();
        }
        None
    }

    fn next_env_owner(&self, owner_id: &str) -> Option<String> {
        let mut current = self.owner_plans.get(owner_id)?.parent_owner_id.clone();
        while let Some(parent_id) = current {
            let owner = self.owner_plans.get(&parent_id)?;
            if !owner.owned_env_slots.is_empty() {
                return Some(parent_id);
            }
            current = owner.parent_owner_id.clone();
        }
        None
    }
}

fn function_name(
    interner: &Interner,
    function: &FunctionDeclaration,
    fallback: Option<&str>,
) -> String {
    fallback
        .map(ToString::to_string)
        .unwrap_or_else(|| interner.resolve_expect(function.name().sym()).to_string())
}

fn collect_simple_parameter_names(
    interner: &Interner,
    parameters: &FormalParameterList,
) -> Vec<String> {
    let mut names = Vec::with_capacity(parameters.as_ref().len());
    let mut seen = BTreeSet::new();
    for parameter in parameters.as_ref() {
        let Binding::Identifier(identifier) = parameter.variable().binding() else {
            return Vec::new();
        };
        if parameter.init().is_some() || parameter.is_rest_param() {
            return Vec::new();
        }
        let name = interner.resolve_expect(identifier.sym()).to_string();
        if !seen.insert(name.clone()) {
            return Vec::new();
        }
        names.push(name);
    }
    names
}

fn default_param_uses_current_or_later_name(
    expression: &Expression,
    blocked: &[String],
    interner: &Interner,
) -> bool {
    match expression {
        Expression::Identifier(identifier) => {
            let ident = interner.resolve_expect(identifier.sym()).to_string();
            blocked.iter().any(|name| name == &ident)
        }
        Expression::Parenthesized(expression) => {
            default_param_uses_current_or_later_name(expression.expression(), blocked, interner)
        }
        Expression::ArrayLiteral(array) => array
            .as_ref()
            .iter()
            .flatten()
            .any(|expr| default_param_uses_current_or_later_name(expr, blocked, interner)),
        Expression::ObjectLiteral(object) => object.properties().iter().any(|property| match property {
            PropertyDefinition::Property(_, value)
            | PropertyDefinition::SpreadObject(value)
            | PropertyDefinition::CoverInitializedName(_, value) => {
                default_param_uses_current_or_later_name(value, blocked, interner)
            }
            PropertyDefinition::MethodDefinition(_) | PropertyDefinition::IdentifierReference(_) => false,
        }),
        Expression::Unary(unary) => {
            default_param_uses_current_or_later_name(unary.target(), blocked, interner)
        }
        Expression::Binary(binary) => {
            default_param_uses_current_or_later_name(binary.lhs(), blocked, interner)
                || default_param_uses_current_or_later_name(binary.rhs(), blocked, interner)
        }
        Expression::Assign(assign) => {
            default_param_uses_current_or_later_name(assign.rhs(), blocked, interner)
                || match assign.lhs() {
                    AssignTarget::Identifier(identifier) => blocked
                        .iter()
                        .any(|name| name == &interner.resolve_expect(identifier.sym()).to_string()),
                    AssignTarget::Access(access) => {
                        default_param_property_access_uses_blocked(access, blocked, interner)
                    }
                    _ => false,
                }
        }
        Expression::Update(update) => match update.target() {
            UpdateTarget::Identifier(identifier) => blocked
                .iter()
                .any(|name| name == &interner.resolve_expect(identifier.sym()).to_string()),
            _ => false,
        },
        Expression::Call(call) => {
            default_param_uses_current_or_later_name(call.function(), blocked, interner)
                || call
                    .args()
                    .iter()
                    .any(|arg| default_param_uses_current_or_later_name(arg, blocked, interner))
        }
        Expression::PropertyAccess(access) => {
            default_param_property_access_uses_blocked(access, blocked, interner)
        }
        Expression::FunctionExpression(_)
        | Expression::ArrowFunction(_)
        | Expression::AsyncArrowFunction(_)
        | Expression::Literal(_)
        | Expression::RegExpLiteral(_)
        | Expression::Spread(_)
        | Expression::GeneratorExpression(_)
        | Expression::AsyncFunctionExpression(_)
        | Expression::AsyncGeneratorExpression(_)
        | Expression::ClassExpression(_)
        | Expression::TemplateLiteral(_)
        | Expression::New(_)
        | Expression::SuperCall(_)
        | Expression::ImportCall(_)
        | Expression::Optional(_)
        | Expression::TaggedTemplate(_)
        | Expression::NewTarget(_)
        | Expression::ImportMeta(_)
        | Expression::BinaryInPrivate(_)
        | Expression::Conditional(_)
        | Expression::Await(_)
        | Expression::Yield(_)
        | Expression::FormalParameterList(_)
        | Expression::This(_)
        | Expression::Debugger => false,
    }
}

fn default_param_property_access_uses_blocked(
    access: &PropertyAccess,
    blocked: &[String],
    interner: &Interner,
) -> bool {
    let PropertyAccess::Simple(access) = access else {
        return false;
    };
    default_param_uses_current_or_later_name(access.target(), blocked, interner)
        || matches!(
            access.field(),
            PropertyAccessField::Expr(expr)
                if default_param_uses_current_or_later_name(expr, blocked, interner)
        )
}

fn function_expression_key(function: &FunctionExpression) -> String {
    if let Some(span) = function.linear_span() {
        return format!("linear:{}:{}", span.start().pos(), span.end().pos());
    }
    let span = function.span();
    format!(
        "span:{}:{}:{}:{}",
        span.start().line_number(),
        span.start().column_number(),
        span.end().line_number(),
        span.end().column_number()
    )
}

fn arrow_function_key(function: &ArrowFunction) -> String {
    let span = function.linear_span();
    format!("linear:{}:{}", span.start().pos(), span.end().pos())
}

fn object_method_key(method: &ObjectMethodDefinition) -> String {
    let span = method.linear_span();
    format!("object-method:{}:{}", span.start().pos(), span.end().pos())
}

struct ScriptLowerer<'a> {
    interner: &'a Interner,
    analysis: &'a Analysis<'a>,
    current_owner_id: String,
    scopes: Vec<BTreeMap<String, BindingInfo>>,
    var_bindings: BTreeMap<String, VarBindingInfo>,
    function_signatures: BTreeMap<FunctionId, FunctionSignature>,
    visible_function_names: BTreeMap<String, FunctionId>,
    diagnostics: Vec<IrDiagnostic>,
    breakable_depth: usize,
    loop_depth: usize,
    labels: Vec<ActiveLabel>,
    is_function_body: bool,
    current_function_id: Option<FunctionId>,
    current_param_names: Vec<String>,
    current_return_info: Option<ValueInfo>,
    current_this_info: ValueInfo,
}

impl<'a> ScriptLowerer<'a> {
    fn new(interner: &'a Interner, analysis: &'a Analysis<'a>, current_owner_id: String) -> Self {
        Self {
            interner,
            analysis,
            current_owner_id,
            scopes: vec![BTreeMap::new()],
            var_bindings: BTreeMap::new(),
            function_signatures: BTreeMap::new(),
            visible_function_names: BTreeMap::new(),
            diagnostics: Vec::new(),
            breakable_depth: 0,
            loop_depth: 0,
            labels: Vec::new(),
            is_function_body: false,
            current_function_id: None,
            current_param_names: Vec::new(),
            current_return_info: None,
            current_this_info: ValueInfo::undefined(),
        }
    }

    fn lower(mut self, script: &Script) -> LoweredScript {
        for function_id in &self.analysis.function_order {
            let plan = self
                .analysis
                .function_plans
                .get(function_id)
                .expect("function plan must exist");
            self.function_signatures.insert(
                function_id.clone(),
                FunctionSignature {
                    id: function_id.clone(),
                    flavor: plan.flavor,
                    params: plan
                        .parameters
                        .as_ref()
                        .iter()
                        .map(|parameter| FunctionParamSignature {
                            kind: if parameter.is_rest_param() {
                                ValueKind::Array
                            } else {
                                ValueKind::Dynamic
                            },
                            has_default: parameter.init().is_some(),
                            is_rest: parameter.is_rest_param(),
                        })
                        .collect(),
                    return_kind: ValueKind::Undefined,
                    return_shape: None,
                    return_targets: BTreeSet::new(),
                    this_info: ValueInfo::undefined(),
                    this_observed: false,
                },
            );
        }
        let mut prepass =
            ScriptLowerer::new(self.interner, self.analysis, SCRIPT_OWNER_ID.to_string());
        prepass.function_signatures = self.function_signatures.clone();
        prepass.hoist_statement_items(script.statements().statements());
        let _ = prepass.lower_root_statement_items(
            script.statements().statements(),
            self.analysis.script_root_functions.as_slice(),
        );
        self.function_signatures = prepass.function_signatures;
        let mut functions = Vec::with_capacity(self.analysis.function_order.len());
        for function_id in &self.analysis.function_order {
            let plan = self
                .analysis
                .function_plans
                .get(function_id)
                .expect("function plan must exist");
            functions.push(self.lower_function(plan));
        }
        self.hoist_statement_items(script.statements().statements());
        let body = self.lower_root_statement_items(
            script.statements().statements(),
            self.analysis.script_root_functions.as_slice(),
        );
        LoweredScript {
            functions,
            body,
            owned_env_bindings: self
                .analysis
                .owner_plans
                .get(SCRIPT_OWNER_ID)
                .map(|owner| {
                    owner
                        .owned_env_slots
                        .iter()
                        .map(|(name, slot)| OwnedEnvBindingIr {
                            name: name.clone(),
                            slot: *slot,
                        })
                        .collect()
                })
                .unwrap_or_default(),
            diagnostics: self.diagnostics,
        }
    }

    fn lower_root_statement_items(
        &mut self,
        items: &[StatementListItem],
        root_functions: &[PendingFunction<'a>],
    ) -> BlockIr {
        let mut statements = Vec::new();
        let mut result_kind = ValueKind::Undefined;

        self.prepare_root_function_bindings(root_functions);
        statements.extend(self.root_function_init_statements(root_functions));

        for item in items {
            match item {
                StatementListItem::Declaration(declaration)
                    if matches!(declaration.as_ref(), Declaration::FunctionDeclaration(_)) => {}
                _ => {
                    let (statement, kind) = self.lower_statement_list_item(item);
                    statements.push(statement);
                    result_kind = kind;
                }
            }
        }

        BlockIr {
            statements,
            result_kind,
        }
    }

    fn lower_statement_items(&mut self, items: &[StatementListItem]) -> BlockIr {
        let mut statements = Vec::new();
        let mut result_kind = ValueKind::Undefined;

        for item in items {
            let (statement, kind) = self.lower_statement_list_item(item);
            statements.push(statement);
            result_kind = kind;
        }

        BlockIr {
            statements,
            result_kind,
        }
    }

    fn prepare_root_function_bindings(&mut self, root_functions: &[PendingFunction<'a>]) {
        self.visible_function_names.clear();
        for function in root_functions {
            self.visible_function_names
                .insert(function.name.clone(), function.id.clone());
            self.declare_binding(
                function.name.clone(),
                BindingInfo {
                    mode: BindingMode::Const,
                    kind: ValueKind::Function,
                    heap_shape: None,
                    function_targets: BTreeSet::from([function.id.clone()]),
                },
            );
        }
    }

    fn root_function_init_statements(
        &mut self,
        root_functions: &[PendingFunction<'a>],
    ) -> Vec<StatementIr> {
        root_functions
            .iter()
            .map(|function| StatementIr::Lexical {
                mode: BindingMode::Const,
                name: function.name.clone(),
                init: TypedExpr::from_info(
                    ValueInfo {
                        kind: ValueKind::Function,
                        heap_shape: None,
                        function_targets: BTreeSet::from([function.id.clone()]),
                    },
                    ExprIr::FunctionValue(function.id.clone()),
                ),
            })
            .collect()
    }

    fn lower_statement_list_item(&mut self, item: &StatementListItem) -> (StatementIr, ValueKind) {
        match item {
            StatementListItem::Statement(statement) => self.lower_statement(statement),
            StatementListItem::Declaration(declaration) => {
                if matches!(declaration.as_ref(), Declaration::FunctionDeclaration(_)) {
                    self.unsupported("function or class declaration");
                    (StatementIr::Empty, ValueKind::Undefined)
                } else {
                    self.lower_declaration(declaration)
                }
            }
        }
    }

    fn lower_statement(&mut self, statement: &Statement) -> (StatementIr, ValueKind) {
        match statement {
            Statement::Expression(expression) => {
                let lowered = self.lower_expression(expression);
                let kind = lowered.kind;
                (StatementIr::Expression(lowered), kind)
            }
            Statement::Empty => (StatementIr::Empty, ValueKind::Undefined),
            Statement::Block(block) => {
                self.push_scope();
                let block_ir = self.lower_block(block);
                self.pop_scope();
                let kind = block_ir.result_kind;
                (StatementIr::Block(block_ir), kind)
            }
            Statement::If(if_statement) => self.lower_if_statement(if_statement),
            Statement::WhileLoop(while_loop) => self.lower_while_loop(while_loop),
            Statement::DoWhileLoop(do_while) => self.lower_do_while_loop(do_while),
            Statement::ForLoop(for_loop) => self.lower_for_loop(for_loop),
            Statement::Switch(switch) => self.lower_switch(switch),
            Statement::Labelled(labelled) => self.lower_labelled(labelled),
            Statement::Break(brk) => self.lower_break(brk),
            Statement::Continue(cont) => self.lower_continue(cont),
            Statement::Debugger => (StatementIr::Debugger, ValueKind::Undefined),
            Statement::Var(var) => self.lower_var_statement(var),
            Statement::Return(ret) => self.lower_return(ret),
            Statement::ForInLoop(_)
            | Statement::ForOfLoop(_)
            | Statement::Throw(_)
            | Statement::Try(_)
            | Statement::With(_) => {
                self.unsupported("control-flow or non-expression statement");
                (StatementIr::Empty, ValueKind::Undefined)
            }
        }
    }

    fn lower_block(&mut self, block: &Block) -> BlockIr {
        self.lower_statement_items(block.statement_list().statements())
    }

    fn lower_if_statement(&mut self, if_statement: &If) -> (StatementIr, ValueKind) {
        let condition = self.lower_expression(if_statement.cond());
        let before_vars = self.var_bindings.clone();
        let (then_branch, then_kind) = self.lower_statement(if_statement.body());
        let then_vars = self.var_bindings.clone();
        let (else_branch, result_kind) = match if_statement.else_node() {
            Some(else_node) => {
                self.var_bindings = before_vars.clone();
                let (else_branch, else_kind) = self.lower_statement(else_node);
                let else_vars = self.var_bindings.clone();
                self.var_bindings = self.merge_var_bindings(&then_vars, &else_vars);
                let kind = if then_kind == else_kind {
                    then_kind
                } else {
                    self.unsupported("if branches with different completion kinds");
                    ValueKind::Undefined
                };
                (Some(Box::new(else_branch)), kind)
            }
            None => {
                self.var_bindings = self.merge_var_bindings(&then_vars, &before_vars);
                (None, ValueKind::Undefined)
            }
        };

        (
            StatementIr::If {
                condition,
                then_branch: Box::new(then_branch),
                else_branch,
            },
            result_kind,
        )
    }

    fn lower_while_loop(&mut self, while_loop: &WhileLoop) -> (StatementIr, ValueKind) {
        let condition = self.lower_expression(while_loop.condition());
        let before_vars = self.var_bindings.clone();
        let (body, body_kind) = self.lower_loop_body(while_loop.body());
        let after_vars = self.var_bindings.clone();
        self.var_bindings = self.merge_var_bindings(&before_vars, &after_vars);
        (
            StatementIr::While {
                condition,
                body: Box::new(body),
            },
            body_kind,
        )
    }

    fn lower_do_while_loop(&mut self, do_while: &DoWhileLoop) -> (StatementIr, ValueKind) {
        let (body, body_kind) = self.lower_loop_body(do_while.body());
        let condition = self.lower_expression(do_while.cond());
        (
            StatementIr::DoWhile {
                body: Box::new(body),
                condition,
            },
            body_kind,
        )
    }

    fn lower_for_loop(&mut self, for_loop: &ForLoop) -> (StatementIr, ValueKind) {
        let before_vars = self.var_bindings.clone();
        self.push_scope();
        let init = for_loop.init().and_then(|init| self.lower_for_init(init));
        let after_init_vars = self.var_bindings.clone();
        let test = for_loop.condition().map(|expr| self.lower_expression(expr));
        let update = for_loop
            .final_expr()
            .map(|expr| self.lower_expression(expr));
        let (body, body_kind) = self.lower_loop_body(for_loop.body());
        self.pop_scope();
        let after_body_vars = self.var_bindings.clone();
        self.var_bindings = self.merge_var_bindings(&after_init_vars, &after_body_vars);
        if for_loop.init().is_none() {
            self.var_bindings = self.merge_var_bindings(&before_vars, &self.var_bindings.clone());
        }

        (
            StatementIr::For {
                init,
                test,
                update,
                body: Box::new(body),
            },
            body_kind,
        )
    }

    fn lower_switch(&mut self, switch: &AstSwitch) -> (StatementIr, ValueKind) {
        let discriminant = self.lower_expression(switch.val());
        let before_vars = self.var_bindings.clone();
        self.push_scope();
        self.breakable_depth += 1;

        let mut cases = Vec::with_capacity(switch.cases().len());
        let mut result_kind: Option<ValueKind> = None;
        let mut merged_vars = before_vars.clone();

        for case in switch.cases() {
            self.var_bindings = before_vars.clone();
            let condition = case.condition().map(|expr| self.lower_expression(expr));
            let body = self.lower_statement_items(case.body().statements());
            merged_vars = self.merge_var_bindings(&merged_vars, &self.var_bindings);
            if let Some(kind) = result_kind {
                if kind != body.result_kind {
                    result_kind = Some(ValueKind::Undefined);
                }
            } else {
                result_kind = Some(body.result_kind);
            }
            cases.push(SwitchCaseIr { condition, body });
        }

        self.breakable_depth -= 1;
        self.pop_scope();
        self.var_bindings = merged_vars;

        (
            StatementIr::Switch {
                discriminant,
                cases,
            },
            result_kind.unwrap_or(ValueKind::Undefined),
        )
    }

    fn lower_labelled(&mut self, labelled: &AstLabelled) -> (StatementIr, ValueKind) {
        let Some((labels, label_kind, base_statement)) = self.collect_labels(labelled) else {
            self.unsupported("label on unsupported statement kind");
            return (StatementIr::Empty, ValueKind::Undefined);
        };

        for label in &labels {
            self.labels.push(ActiveLabel {
                name: label.clone(),
                kind: label_kind,
            });
        }

        let lowered = self.lower_statement(base_statement);

        for _ in 0..labels.len() {
            self.labels.pop();
        }

        (
            StatementIr::Labelled {
                labels,
                statement: Box::new(lowered.0),
            },
            lowered.1,
        )
    }

    fn collect_labels<'b>(
        &self,
        labelled: &'b AstLabelled,
    ) -> Option<(Vec<String>, LabelTargetKind, &'b Statement)> {
        let mut labels = vec![self.interner.resolve_expect(labelled.label()).to_string()];
        let mut item = labelled.item();

        loop {
            match item {
                LabelledItem::Statement(Statement::Labelled(next)) => {
                    labels.push(self.interner.resolve_expect(next.label()).to_string());
                    item = next.item();
                }
                LabelledItem::Statement(statement) => {
                    let Some(kind) = Self::label_target_kind(statement) else {
                        return None;
                    };
                    return Some((labels, kind, statement));
                }
                LabelledItem::FunctionDeclaration(_) => {
                    return None;
                }
            }
        }
    }

    fn label_target_kind(statement: &Statement) -> Option<LabelTargetKind> {
        match statement {
            Statement::Block(_) | Statement::Switch(_) => Some(LabelTargetKind::Breakable),
            Statement::WhileLoop(_) | Statement::DoWhileLoop(_) | Statement::ForLoop(_) => {
                Some(LabelTargetKind::Loop)
            }
            _ => None,
        }
    }

    fn lower_for_init(&mut self, init: &ForLoopInitializer) -> Option<ForInitIr> {
        match init {
            ForLoopInitializer::Expression(expr) => {
                Some(ForInitIr::Expression(self.lower_expression(expr)))
            }
            ForLoopInitializer::Var(var) => self.lower_var_init(var),
            ForLoopInitializer::Lexical(lexical) => {
                self.lower_for_lexical_init(lexical.declaration())
            }
        }
    }

    fn lower_for_lexical_init(&mut self, declaration: &LexicalDeclaration) -> Option<ForInitIr> {
        let (mode, list) = match declaration {
            LexicalDeclaration::Let(list) => (BindingMode::Let, list),
            LexicalDeclaration::Const(list) => (BindingMode::Const, list),
            LexicalDeclaration::Using(_) | LexicalDeclaration::AwaitUsing(_) => {
                self.unsupported("using declaration");
                return None;
            }
        };

        if list.as_ref().len() != 1 {
            self.unsupported("multi-binding lexical declaration");
            return None;
        }

        let variable = &list.as_ref()[0];
        let Binding::Identifier(identifier) = variable.binding() else {
            self.unsupported("destructuring binding");
            return None;
        };

        let name = self.interner.resolve_expect(identifier.sym()).to_string();
        let init = variable
            .init()
            .map(|expression| self.lower_expression(expression))
            .unwrap_or_else(TypedExpr::undefined);

        self.declare_binding(
            name.clone(),
            BindingInfo {
                mode,
                kind: init.kind,
                heap_shape: init.heap_shape.clone(),
                function_targets: init.function_targets.clone(),
            },
        );
        Some(ForInitIr::Lexical { mode, name, init })
    }

    fn lower_var_statement(&mut self, declaration: &VarDeclaration) -> (StatementIr, ValueKind) {
        (
            StatementIr::Var(self.lower_var_declarators(declaration)),
            ValueKind::Undefined,
        )
    }

    fn lower_var_init(&mut self, declaration: &VarDeclaration) -> Option<ForInitIr> {
        Some(ForInitIr::Var(self.lower_var_declarators(declaration)))
    }

    fn lower_var_declarators(&mut self, declaration: &VarDeclaration) -> Vec<VarDeclaratorIr> {
        let mut declarators = Vec::with_capacity(declaration.0.as_ref().len());
        for variable in declaration.0.as_ref() {
            let Binding::Identifier(identifier) = variable.binding() else {
                self.unsupported("destructuring var declaration");
                continue;
            };

            let name = self.interner.resolve_expect(identifier.sym()).to_string();
            let init = variable
                .init()
                .map(|expression| self.lower_expression(expression));
            if let Some(init) = &init {
                self.set_var_kind(&name, init.kind);
            }
            declarators.push(VarDeclaratorIr { name, init });
        }
        declarators
    }

    fn lower_break(&mut self, brk: &AstBreak) -> (StatementIr, ValueKind) {
        if let Some(label) = brk.label() {
            let label = self.interner.resolve_expect(label).to_string();
            if self.labels.iter().rev().any(|active| active.name == label) {
                return (
                    StatementIr::Break { label: Some(label) },
                    ValueKind::Undefined,
                );
            }
            self.unsupported("break to unknown label");
            return (StatementIr::Empty, ValueKind::Undefined);
        }
        if self.breakable_depth == 0 {
            self.unsupported("break outside loop or switch");
            return (StatementIr::Empty, ValueKind::Undefined);
        }
        (StatementIr::Break { label: None }, ValueKind::Undefined)
    }

    fn lower_continue(&mut self, cont: &AstContinue) -> (StatementIr, ValueKind) {
        if let Some(label) = cont.label() {
            let label = self.interner.resolve_expect(label).to_string();
            let Some(active) = self.labels.iter().rev().find(|active| active.name == label) else {
                self.unsupported("continue to unknown label");
                return (StatementIr::Empty, ValueKind::Undefined);
            };
            if active.kind != LabelTargetKind::Loop {
                self.unsupported("continue to non-loop label");
                return (StatementIr::Empty, ValueKind::Undefined);
            }
            return (
                StatementIr::Continue { label: Some(label) },
                ValueKind::Undefined,
            );
        }
        if self.loop_depth == 0 {
            self.unsupported("continue outside loop");
            return (StatementIr::Empty, ValueKind::Undefined);
        }
        (StatementIr::Continue { label: None }, ValueKind::Undefined)
    }

    fn lower_function_parameters<'b>(
        &mut self,
        parameters: &'b FormalParameterList,
        function_name: &str,
    ) -> Option<&'b FormalParameterList> {
        let mut names = BTreeSet::new();
        for parameter in parameters.as_ref() {
            let Binding::Identifier(_) = parameter.variable().binding() else {
                self.unsupported_with_message(format!(
                    "unsupported in porffor wasm-aot first slice: unsupported parameter form in `{function_name}`"
                ));
                return None;
            };
            let Binding::Identifier(identifier) = parameter.variable().binding() else {
                unreachable!();
            };
            let name = self.interner.resolve_expect(identifier.sym()).to_string();
            if !names.insert(name) {
                self.unsupported_with_message(format!(
                    "unsupported in porffor wasm-aot first slice: duplicate parameter name in `{function_name}`"
                ));
                return None;
            }
        }
        Some(parameters)
    }

    fn lower_function(&mut self, function: &FunctionPlan<'a>) -> FunctionIr {
        let mut lowerer =
            ScriptLowerer::new(self.interner, self.analysis, function.id.clone());
        lowerer.function_signatures = self.function_signatures.clone();
        lowerer.visible_function_names = self.visible_function_names.clone();
        lowerer.is_function_body = true;
        lowerer.current_function_id = Some(function.id.clone());
        lowerer.current_this_info = if function.flavor == FunctionFlavor::Arrow {
            function
                .captures
                .get(LEXICAL_THIS_NAME)
                .map(|capture| lowerer.capture_value_info(capture.owner_id.as_str(), LEXICAL_THIS_NAME))
                .unwrap_or_else(ValueInfo::undefined)
        } else {
            lowerer
                .function_signatures
                .get(&function.id)
                .map(|signature| signature.this_info.clone())
                .unwrap_or_else(ValueInfo::undefined)
        };
        lowerer.current_owner_id = function.id.clone();
        let Some(parameters) =
            lowerer.lower_function_parameters(function.parameters, function.name.as_str())
        else {
            self.diagnostics.extend(lowerer.diagnostics.clone());
            return FunctionIr {
                id: function.id.clone(),
                name: function.name.clone(),
                flavor: function.flavor,
                is_nested: function.parent_owner_id != SCRIPT_OWNER_ID,
                is_expression: function.is_expression,
                is_named_expression: function.is_expression && function.self_binding_name.is_some(),
                captures_lexical_this: function.captures.contains_key(LEXICAL_THIS_NAME),
                captures_lexical_arguments: function.captures.contains_key(LEXICAL_ARGUMENTS_NAME),
                params: Vec::new(),
                body: BlockIr {
                    statements: Vec::new(),
                    result_kind: ValueKind::Undefined,
                },
                return_kind: ValueKind::Undefined,
                return_shape: None,
                return_targets: BTreeSet::new(),
                owned_env_bindings: Vec::new(),
                captured_bindings: Vec::new(),
            };
        };
        if let Some(self_binding_name) = function.self_binding_name.as_ref() {
            lowerer.declare_binding(
                self_binding_name.clone(),
                BindingInfo {
                    mode: BindingMode::Const,
                    kind: ValueKind::Function,
                    heap_shape: None,
                    function_targets: BTreeSet::from([function.id.clone()]),
                },
            );
        }
        for (name, capture) in &function.captures {
            let info = lowerer.capture_value_info(capture.owner_id.as_str(), name);
            lowerer.declare_binding(
                name.clone(),
                BindingInfo {
                    mode: BindingMode::Let,
                    kind: info.kind,
                    heap_shape: info.heap_shape,
                    function_targets: info.function_targets,
                },
            );
        }

        if function.flavor == FunctionFlavor::Ordinary {
            lowerer.declare_binding(
                LEXICAL_ARGUMENTS_NAME.to_string(),
                BindingInfo {
                    mode: BindingMode::Let,
                    kind: ValueKind::Arguments,
                    heap_shape: None,
                    function_targets: BTreeSet::new(),
                },
            );
        }

        let mut params = Vec::with_capacity(parameters.as_ref().len());
        let parameter_names = parameters
            .as_ref()
            .iter()
            .map(|parameter| {
                let Binding::Identifier(identifier) = parameter.variable().binding() else {
                    unreachable!();
                };
                self.interner.resolve_expect(identifier.sym()).to_string()
            })
            .collect::<Vec<_>>();
        for (index, parameter) in parameters.as_ref().iter().enumerate() {
            let Binding::Identifier(identifier) = parameter.variable().binding() else {
                continue;
            };
            let name = self.interner.resolve_expect(identifier.sym()).to_string();
            if let Some(init) = parameter.init() {
                if default_param_uses_current_or_later_name(
                    init,
                    &parameter_names[index..],
                    self.interner,
                ) {
                    lowerer.unsupported_with_message(format!(
                        "unsupported in porffor wasm-aot first slice: self- or later-param read in default initializer for `{}`",
                        function.name
                    ));
                    self.diagnostics.extend(lowerer.diagnostics.clone());
                    return FunctionIr {
                        id: function.id.clone(),
                        name: function.name.clone(),
                        flavor: function.flavor,
                        is_nested: function.parent_owner_id != SCRIPT_OWNER_ID,
                        is_expression: function.is_expression,
                        is_named_expression: function.is_expression && function.self_binding_name.is_some(),
                        captures_lexical_this: function.captures.contains_key(LEXICAL_THIS_NAME),
                        captures_lexical_arguments: function.captures.contains_key(LEXICAL_ARGUMENTS_NAME),
                        params: Vec::new(),
                        body: BlockIr {
                            statements: Vec::new(),
                            result_kind: ValueKind::Undefined,
                        },
                        return_kind: ValueKind::Undefined,
                        return_shape: None,
                        return_targets: BTreeSet::new(),
                        owned_env_bindings: Vec::new(),
                        captured_bindings: Vec::new(),
                    };
                }
            }
            let kind = lowerer
                .function_signatures
                .get(&function.id)
                .and_then(|signature| signature.params.get(params.len()).cloned())
                .map(|signature| signature.kind)
                .unwrap_or_else(|| {
                    if parameter.is_rest_param() {
                        ValueKind::Array
                    } else {
                        ValueKind::Dynamic
                    }
                });
            let default_init = parameter.init().map(|expression| lowerer.lower_expression(expression));
            lowerer.declare_binding(
                name.clone(),
                BindingInfo {
                    mode: BindingMode::Let,
                    kind,
                    heap_shape: None,
                    function_targets: BTreeSet::new(),
                },
            );
            lowerer.current_param_names.push(name.clone());
            params.push(FunctionParamIr {
                name,
                kind,
                default_init,
                is_rest: parameter.is_rest_param(),
            });
        }

        lowerer.hoist_statement_items(function.body.statements());

        let body = lowerer.lower_root_statement_items(
            function.body.statements(),
            function.root_functions.as_slice(),
        );
        let mut return_info = lowerer
            .current_return_info
            .clone()
            .unwrap_or_else(ValueInfo::undefined);
        let final_statement_is_return =
            matches!(body.statements.last(), Some(StatementIr::Return(_)));
        if !final_statement_is_return {
            return_info = lowerer.merge_return_infos(return_info, ValueInfo::undefined());
        }

        if let Some(signature) = lowerer.function_signatures.get(&function.id) {
            for (param, signature_param) in params.iter_mut().zip(signature.params.iter()) {
                param.kind = signature_param.kind;
            }
            return_info = ValueInfo {
                kind: signature.return_kind,
                heap_shape: signature.return_shape.clone(),
                function_targets: signature.return_targets.clone(),
            };
            if !final_statement_is_return {
                return_info = lowerer.merge_return_infos(return_info, ValueInfo::undefined());
            }
        }

        self.diagnostics.extend(lowerer.diagnostics.clone());
        self.function_signatures = lowerer.function_signatures;

        FunctionIr {
            id: function.id.clone(),
            name: function.name.clone(),
            flavor: function.flavor,
            is_nested: function.parent_owner_id != SCRIPT_OWNER_ID,
            is_expression: function.is_expression,
            is_named_expression: function.is_expression && function.self_binding_name.is_some(),
            captures_lexical_this: function.captures.contains_key(LEXICAL_THIS_NAME),
            captures_lexical_arguments: function.captures.contains_key(LEXICAL_ARGUMENTS_NAME),
            params,
            body,
            return_kind: return_info.kind,
            return_shape: return_info.heap_shape,
            return_targets: return_info.function_targets,
            owned_env_bindings: self
                .analysis
                .owner_plans
                .get(&function.id)
                .map(|owner| {
                    owner
                        .owned_env_slots
                        .iter()
                        .map(|(name, slot)| OwnedEnvBindingIr {
                            name: name.clone(),
                            slot: *slot,
                        })
                        .collect()
                })
                .unwrap_or_default(),
            captured_bindings: function
                .captures
                .iter()
                .map(|(name, capture)| CapturedBindingIr {
                    name: name.clone(),
                    slot: capture.slot,
                    hops: capture.hops,
                })
                .collect(),
        }
    }

    fn lower_loop_body(&mut self, statement: &Statement) -> (StatementIr, ValueKind) {
        self.breakable_depth += 1;
        self.loop_depth += 1;
        let lowered = self.lower_statement(statement);
        self.loop_depth -= 1;
        self.breakable_depth -= 1;
        lowered
    }

    fn lower_return(&mut self, ret: &AstReturn) -> (StatementIr, ValueKind) {
        if !self.is_function_body {
            self.unsupported("top-level return");
            return (StatementIr::Empty, ValueKind::Undefined);
        }
        let value = ret
            .target()
            .map(|expression| self.lower_expression(expression))
            .unwrap_or_else(TypedExpr::undefined);
        self.record_return_info(value.value_info());
        (StatementIr::Return(value.clone()), value.kind)
    }

    fn lower_declaration(&mut self, declaration: &Declaration) -> (StatementIr, ValueKind) {
        match declaration {
            Declaration::Lexical(lexical) => self.lower_lexical_declaration(lexical),
            Declaration::FunctionDeclaration(_)
            | Declaration::GeneratorDeclaration(_)
            | Declaration::AsyncFunctionDeclaration(_)
            | Declaration::AsyncGeneratorDeclaration(_)
            | Declaration::ClassDeclaration(_) => {
                self.unsupported("function or class declaration");
                (StatementIr::Empty, ValueKind::Undefined)
            }
        }
    }

    fn lower_lexical_declaration(
        &mut self,
        declaration: &LexicalDeclaration,
    ) -> (StatementIr, ValueKind) {
        let (mode, list) = match declaration {
            LexicalDeclaration::Let(list) => (BindingMode::Let, list),
            LexicalDeclaration::Const(list) => (BindingMode::Const, list),
            LexicalDeclaration::Using(_) | LexicalDeclaration::AwaitUsing(_) => {
                self.unsupported("using declaration");
                return (StatementIr::Empty, ValueKind::Undefined);
            }
        };

        if list.as_ref().len() != 1 {
            self.unsupported("multi-binding lexical declaration");
            return (StatementIr::Empty, ValueKind::Undefined);
        }

        let variable = &list.as_ref()[0];
        let Binding::Identifier(identifier) = variable.binding() else {
            self.unsupported("destructuring binding");
            return (StatementIr::Empty, ValueKind::Undefined);
        };

        let name = self.interner.resolve_expect(identifier.sym()).to_string();
        let init = variable
            .init()
            .map(|expression| self.lower_expression(expression))
            .unwrap_or_else(TypedExpr::undefined);

        self.declare_binding(
            name.clone(),
            BindingInfo {
                mode,
                kind: init.kind,
                heap_shape: init.heap_shape.clone(),
                function_targets: init.function_targets.clone(),
            },
        );

        (
            StatementIr::Lexical { mode, name, init },
            ValueKind::Undefined,
        )
    }

    fn hoist_statement_items(&mut self, items: &[StatementListItem]) {
        for item in items {
            match item {
                StatementListItem::Statement(statement) => self.hoist_statement(statement),
                StatementListItem::Declaration(_) => {}
            }
        }
    }

    fn hoist_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Block(block) => {
                self.hoist_statement_items(block.statement_list().statements())
            }
            Statement::If(if_statement) => {
                self.hoist_statement(if_statement.body());
                if let Some(else_node) = if_statement.else_node() {
                    self.hoist_statement(else_node);
                }
            }
            Statement::WhileLoop(while_loop) => self.hoist_statement(while_loop.body()),
            Statement::DoWhileLoop(do_while) => self.hoist_statement(do_while.body()),
            Statement::ForLoop(for_loop) => {
                if let Some(ForLoopInitializer::Var(var)) = for_loop.init() {
                    self.hoist_var_declaration(var);
                }
                self.hoist_statement(for_loop.body());
            }
            Statement::Switch(switch) => {
                for case in switch.cases() {
                    self.hoist_statement_items(case.body().statements());
                }
            }
            Statement::Labelled(labelled) => {
                if let Some(statement) = Self::labelled_base_statement(labelled) {
                    self.hoist_statement(statement);
                }
            }
            Statement::Var(var) => self.hoist_var_declaration(var),
            Statement::Expression(_)
            | Statement::Empty
            | Statement::Break(_)
            | Statement::Continue(_)
            | Statement::Debugger
            | Statement::ForInLoop(_)
            | Statement::ForOfLoop(_)
            | Statement::Return(_)
            | Statement::Throw(_)
            | Statement::Try(_)
            | Statement::With(_) => {}
        }
    }

    fn hoist_var_declaration(&mut self, declaration: &VarDeclaration) {
        for variable in declaration.0.as_ref() {
            let Binding::Identifier(identifier) = variable.binding() else {
                self.unsupported("destructuring var declaration");
                continue;
            };
            let name = self.interner.resolve_expect(identifier.sym()).to_string();
            self.var_bindings.entry(name).or_insert(VarBindingInfo {
                kind: ValueKind::Undefined,
                heap_shape: None,
                function_targets: BTreeSet::new(),
            });
        }
    }

    fn labelled_base_statement<'b>(labelled: &'b AstLabelled) -> Option<&'b Statement> {
        let mut item = labelled.item();
        loop {
            match item {
                LabelledItem::Statement(Statement::Labelled(next)) => {
                    item = next.item();
                }
                LabelledItem::Statement(statement) => return Some(statement),
                LabelledItem::FunctionDeclaration(_) => return None,
            }
        }
    }

    fn lower_expression(&mut self, expression: &Expression) -> TypedExpr {
        match expression {
            Expression::Identifier(identifier) => {
                let name = self.interner.resolve_expect(identifier.sym()).to_string();
                if let Some(binding) = self.lookup_binding(&name) {
                    let info = ValueInfo {
                        kind: binding.kind,
                        heap_shape: binding.heap_shape.clone(),
                        function_targets: binding.function_targets.clone(),
                    };
                    TypedExpr::from_info(info, ExprIr::Identifier(name))
                } else if let Some(function_id) = self.visible_function_names.get(&name).cloned() {
                    let mut function_targets = BTreeSet::new();
                    function_targets.insert(function_id.clone());
                    TypedExpr::from_info(
                        ValueInfo {
                            kind: ValueKind::Function,
                            heap_shape: None,
                            function_targets,
                        },
                        ExprIr::FunctionValue(function_id),
                    )
                } else if name == "arguments"
                    && self.lookup_binding(LEXICAL_ARGUMENTS_NAME).is_some()
                {
                    TypedExpr::from_info(
                        ValueInfo {
                            kind: ValueKind::Arguments,
                            heap_shape: None,
                            function_targets: BTreeSet::new(),
                        },
                        ExprIr::Arguments,
                    )
                } else {
                    self.unsupported_with_message(format!(
                        "unsupported in porffor wasm-aot first slice: unbound identifier `{name}`"
                    ));
                    TypedExpr::undefined()
                }
            }
            Expression::Literal(literal) => match literal.kind() {
                LiteralKind::String(sym) => TypedExpr::from_info(
                    ValueInfo {
                        kind: ValueKind::String,
                        heap_shape: None,
                        function_targets: BTreeSet::new(),
                    },
                    ExprIr::String(self.interner.resolve_expect(*sym).to_string()),
                ),
                LiteralKind::Num(value) => TypedExpr::from_info(
                    ValueInfo {
                        kind: ValueKind::Number,
                        heap_shape: None,
                        function_targets: BTreeSet::new(),
                    },
                    ExprIr::Number(value.to_bits()),
                ),
                LiteralKind::Int(value) => TypedExpr::from_info(
                    ValueInfo {
                        kind: ValueKind::Number,
                        heap_shape: None,
                        function_targets: BTreeSet::new(),
                    },
                    ExprIr::Number((*value as f64).to_bits()),
                ),
                LiteralKind::Bool(value) => TypedExpr::from_info(
                    ValueInfo {
                        kind: ValueKind::Boolean,
                        heap_shape: None,
                        function_targets: BTreeSet::new(),
                    },
                    ExprIr::Boolean(*value),
                ),
                LiteralKind::Null => TypedExpr::from_info(
                    ValueInfo {
                        kind: ValueKind::Null,
                        heap_shape: None,
                        function_targets: BTreeSet::new(),
                    },
                    ExprIr::Null,
                ),
                LiteralKind::Undefined => TypedExpr::undefined(),
                LiteralKind::BigInt(_) => self.unsupported_expr("bigint literal"),
            },
            Expression::Parenthesized(expression) => self.lower_expression(expression.expression()),
            Expression::ArrayLiteral(array) => self.lower_array_literal(array),
            Expression::ObjectLiteral(object) => self.lower_object_literal(object),
            Expression::Unary(unary) => self.lower_unary(unary.op(), unary.target()),
            Expression::Binary(binary) => {
                self.lower_binary(binary.op(), binary.lhs(), binary.rhs())
            }
            Expression::Assign(assign) => {
                self.lower_assign(assign.op(), assign.lhs(), assign.rhs())
            }
            Expression::Call(call) => self.lower_call(call.function(), call.args()),
            Expression::FunctionExpression(function) => self.lower_function_expression(function),
            Expression::ArrowFunction(function) => self.lower_arrow_function(function),
            Expression::PropertyAccess(access) => self.lower_property_access(access),
            Expression::This(_) => {
                if !self.is_function_body {
                    return self.unsupported_expr("top-level `this`");
                }
                TypedExpr::from_info(self.current_this_info.clone(), ExprIr::This)
            }
            Expression::RegExpLiteral(_)
            | Expression::Spread(_)
            | Expression::AsyncArrowFunction(_)
            | Expression::GeneratorExpression(_)
            | Expression::AsyncFunctionExpression(_)
            | Expression::AsyncGeneratorExpression(_)
            | Expression::ClassExpression(_)
            | Expression::TemplateLiteral(_)
            | Expression::New(_)
            | Expression::SuperCall(_)
            | Expression::ImportCall(_)
            | Expression::Optional(_)
            | Expression::TaggedTemplate(_)
            | Expression::NewTarget(_)
            | Expression::ImportMeta(_)
            | Expression::BinaryInPrivate(_)
            | Expression::Conditional(_)
            | Expression::Await(_)
            | Expression::Yield(_)
            | Expression::FormalParameterList(_)
            | Expression::Debugger => self.unsupported_expr("unsupported expression form"),
            Expression::Update(update) => self.lower_update(update.op(), update.target()),
        }
    }

    fn lower_function_expression(&mut self, function: &FunctionExpression) -> TypedExpr {
        let key = function_expression_key(function);
        let Some(function_id) = self.analysis.function_expr_ids.get(&key).cloned() else {
            return self.unsupported_expr("function expression");
        };
        TypedExpr::from_info(
            ValueInfo {
                kind: ValueKind::Function,
                heap_shape: None,
                function_targets: BTreeSet::from([function_id.clone()]),
            },
            ExprIr::FunctionValue(function_id),
        )
    }

    fn lower_arrow_function(&mut self, function: &ArrowFunction) -> TypedExpr {
        let key = arrow_function_key(function);
        let Some(function_id) = self.analysis.function_expr_ids.get(&key).cloned() else {
            return self.unsupported_expr("arrow function");
        };
        if !self.is_function_body {
            let captures_lexical_this = self
                .analysis
                .function_plans
                .get(&function_id)
                .is_some_and(|plan| plan.captures.contains_key(LEXICAL_THIS_NAME));
            if captures_lexical_this {
                return self.unsupported_expr("top-level `this`");
            }
            let captures_lexical_arguments = self
                .analysis
                .function_plans
                .get(&function_id)
                .is_some_and(|plan| plan.captures.contains_key(LEXICAL_ARGUMENTS_NAME));
            if captures_lexical_arguments {
                return self.unsupported_expr("top-level `arguments`");
            }
        }
        TypedExpr::from_info(
            ValueInfo {
                kind: ValueKind::Function,
                heap_shape: None,
                function_targets: BTreeSet::from([function_id.clone()]),
            },
            ExprIr::FunctionValue(function_id),
        )
    }

    fn lower_call(&mut self, callee: &Expression, args: &[Expression]) -> TypedExpr {
        if let Expression::PropertyAccess(PropertyAccess::Simple(access)) = callee {
            let receiver = self.lower_property_target(access.target());
            let callee = match receiver.kind {
                ValueKind::Object => self.lower_object_property_key(receiver.clone(), access.field()),
                ValueKind::Array => self.lower_array_index_key(receiver.clone(), access.field()),
                ValueKind::Dynamic => return self.unsupported_expr("indirect call"),
                _ => return self.unsupported_expr("indirect call"),
            };
            if callee.kind != ValueKind::Function {
                return self.unsupported_expr("indirect call");
            }
            let Some(function_id) = self.resolve_single_function_target(&callee) else {
                return self.unsupported_expr("indirect call");
            };
            self.merge_function_this_info(&function_id, receiver.value_info());
            let (args, info) = self.lower_call_args(&function_id, args);
            let ExprIr::PropertyRead { key, .. } = callee.expr else {
                return self.unsupported_expr("indirect call");
            };
            return TypedExpr::from_info(
                info,
                ExprIr::CallMethod {
                    receiver: Box::new(receiver),
                    key,
                    args,
                },
            );
        }

        let callee = self.lower_expression(callee);
        if callee.kind != ValueKind::Function {
            return self.unsupported_expr("indirect call");
        }
        let Some(function_id) = self.resolve_single_function_target(&callee) else {
            return self.unsupported_expr("indirect call");
        };
        self.merge_function_this_info(&function_id, ValueInfo::undefined());
        let (args, info) = self.lower_call_args(&function_id, args);
        TypedExpr::from_info(
            info,
            ExprIr::CallIndirect {
                callee: Box::new(callee),
                args,
            },
        )
    }

    fn lower_call_args(
        &mut self,
        function_id: &FunctionId,
        args: &[Expression],
    ) -> (Vec<TypedExpr>, ValueInfo) {
        let Some(signature) = self.function_signatures.get(function_id).cloned() else {
            self.unsupported_expr("indirect call");
            return (Vec::new(), ValueInfo::undefined());
        };
        let mut lowered_args = Vec::with_capacity(args.len());
        for arg in args {
            lowered_args.push(self.lower_expression(arg));
        }

        for (_index, param) in signature.params.iter().enumerate().skip(lowered_args.len()) {
            if param.is_rest {
                break;
            }
            if param.kind == ValueKind::Number && !param.has_default {
                self.unsupported_with_message(format!(
                    "unsupported in porffor wasm-aot first slice: missing numeric argument"
                ));
            }
        }

        for (index, param) in signature.params.iter().enumerate() {
            if param.is_rest || param.kind != ValueKind::Number {
                continue;
            }
            let Some(arg) = lowered_args.get(index).cloned() else {
                continue;
            };
            let Some(number_arg) = self.coerce_expr_to_number(arg) else {
                self.unsupported_with_message(format!(
                    "unsupported in porffor wasm-aot first slice: numeric argument required"
                ));
                return (Vec::new(), ValueInfo::undefined());
            };
            lowered_args[index] = number_arg;
        }

        (
            lowered_args,
            ValueInfo {
                kind: signature.return_kind,
                heap_shape: signature.return_shape,
                function_targets: signature.return_targets,
            },
        )
    }

    fn resolve_single_function_target(&self, expr: &TypedExpr) -> Option<FunctionId> {
        if expr.kind != ValueKind::Function || expr.function_targets.len() != 1 {
            return None;
        }
        expr.function_targets.iter().next().cloned()
    }

    fn merge_function_this_info(&mut self, function_id: &FunctionId, info: ValueInfo) {
        let Some(signature) = self.function_signatures.get(function_id).cloned() else {
            return;
        };
        if signature.flavor == FunctionFlavor::Arrow {
            return;
        }
        let (next, observed) = if signature.this_observed {
            (self.merge_value_infos(signature.this_info, info), true)
        } else {
            (info, true)
        };
        if let Some(signature) = self.function_signatures.get_mut(function_id) {
            signature.this_info = next;
            signature.this_observed = observed;
        }
    }

    fn lower_array_literal(&mut self, array: &ArrayLiteral) -> TypedExpr {
        let mut elements = Vec::with_capacity(array.as_ref().len());
        let mut shape = ArrayShape::default();
        for element in array.as_ref() {
            let Some(element) = element else {
                return self.unsupported_expr("array literal holes");
            };
            if matches!(element, Expression::Spread(_)) {
                return self.unsupported_expr("array literal spread");
            }
            let lowered = self.lower_expression(element);
            shape.elements.push(lowered.value_info());
            elements.push(lowered);
        }
        TypedExpr::from_info(
            ValueInfo {
                kind: ValueKind::Array,
                heap_shape: Some(Box::new(HeapShape::Array(shape))),
                function_targets: BTreeSet::new(),
            },
            ExprIr::ArrayLiteral(elements),
        )
    }

    fn function_value_expr(&self, function_id: FunctionId) -> TypedExpr {
        TypedExpr::from_info(
            ValueInfo {
                kind: ValueKind::Function,
                heap_shape: None,
                function_targets: BTreeSet::from([function_id.clone()]),
            },
            ExprIr::FunctionValue(function_id),
        )
    }

    fn accessor_return_info(&self, function_id: &FunctionId) -> ValueInfo {
        self.function_signatures
            .get(function_id)
            .map(|signature| ValueInfo {
                kind: signature.return_kind,
                heap_shape: signature.return_shape.clone(),
                function_targets: signature.return_targets.clone(),
            })
            .unwrap_or_else(ValueInfo::undefined)
    }

    fn lower_object_method_function(
        &mut self,
        method: &ObjectMethodDefinition,
        function_name: &str,
    ) -> Option<TypedExpr> {
        let Some(parameters) = self.lower_function_parameters(method.parameters(), function_name) else {
            return None;
        };
        match method.kind() {
            MethodDefinitionKind::Ordinary => {}
            MethodDefinitionKind::Get => {
                if !parameters.as_ref().is_empty() {
                    self.unsupported_with_message(format!(
                        "unsupported in porffor wasm-aot first slice: getter `{function_name}` must not declare parameters"
                    ));
                    return None;
                }
            }
            MethodDefinitionKind::Set => {
                if parameters.as_ref().len() != 1 {
                    self.unsupported_with_message(format!(
                        "unsupported in porffor wasm-aot first slice: setter `{function_name}` must declare exactly one parameter"
                    ));
                    return None;
                }
                let parameter = &parameters.as_ref()[0];
                if parameter.is_rest_param() || parameter.init().is_some() {
                    self.unsupported_with_message(format!(
                        "unsupported in porffor wasm-aot first slice: setter `{function_name}` parameter must be plain identifier"
                    ));
                    return None;
                }
            }
            MethodDefinitionKind::Generator
            | MethodDefinitionKind::AsyncGenerator
            | MethodDefinitionKind::Async => {
                self.unsupported_expr("object literal method");
                return None;
            }
        }
        let key = object_method_key(method);
        let Some(function_id) = self.analysis.function_expr_ids.get(&key).cloned() else {
            self.unsupported_expr("object literal method");
            return None;
        };
        Some(self.function_value_expr(function_id))
    }

    fn lower_object_literal(&mut self, object: &ObjectLiteral) -> TypedExpr {
        let mut properties = Vec::with_capacity(object.properties().len());
        let mut shape = ObjectShape::default();
        for property in object.properties() {
            match property {
                PropertyDefinition::Property(PropertyName::Literal(name), value) => {
                    let key = self.interner.resolve_expect(name.sym()).to_string();
                    let lowered = self.lower_expression(value);
                    shape
                        .properties
                        .insert(key.clone(), ObjectShapeProperty::Data(lowered.value_info()));
                    properties.push(ObjectPropertyIr::Data {
                        key,
                        value: lowered,
                        is_shorthand: false,
                    });
                }
                PropertyDefinition::IdentifierReference(identifier) => {
                    let key = self.interner.resolve_expect(identifier.sym()).to_string();
                    let lowered = self.lower_expression(&Expression::Identifier(*identifier));
                    shape
                        .properties
                        .insert(key.clone(), ObjectShapeProperty::Data(lowered.value_info()));
                    properties.push(ObjectPropertyIr::Data {
                        key,
                        value: lowered,
                        is_shorthand: true,
                    });
                }
                PropertyDefinition::MethodDefinition(method) => {
                    let PropertyName::Literal(name) = method.name() else {
                        return self.unsupported_expr("computed object key");
                    };
                    let key = self.interner.resolve_expect(name.sym()).to_string();
                    let Some(function) = self.lower_object_method_function(method, &key) else {
                        return TypedExpr::undefined();
                    };
                    match method.kind() {
                        MethodDefinitionKind::Ordinary => {
                            shape.properties.insert(
                                key.clone(),
                                ObjectShapeProperty::Data(function.value_info()),
                            );
                            properties.push(ObjectPropertyIr::Method { key, function });
                        }
                        MethodDefinitionKind::Get => {
                            let function_id = self
                                .resolve_single_function_target(&function)
                                .expect("getter should resolve single target");
                            let entry = shape.properties.remove(&key);
                            let setter = match entry {
                                Some(ObjectShapeProperty::Accessor { setter, .. }) => setter,
                                _ => None,
                            };
                            shape.properties.insert(
                                key.clone(),
                                ObjectShapeProperty::Accessor {
                                    getter: Some(ObjectAccessorShape { function_id }),
                                    setter,
                                },
                            );
                            properties.push(ObjectPropertyIr::Getter { key, function });
                        }
                        MethodDefinitionKind::Set => {
                            let function_id = self
                                .resolve_single_function_target(&function)
                                .expect("setter should resolve single target");
                            let entry = shape.properties.remove(&key);
                            let getter = match entry {
                                Some(ObjectShapeProperty::Accessor { getter, .. }) => getter,
                                _ => None,
                            };
                            shape.properties.insert(
                                key.clone(),
                                ObjectShapeProperty::Accessor {
                                    getter,
                                    setter: Some(ObjectAccessorShape { function_id }),
                                },
                            );
                            properties.push(ObjectPropertyIr::Setter { key, function });
                        }
                        MethodDefinitionKind::Generator
                        | MethodDefinitionKind::AsyncGenerator
                        | MethodDefinitionKind::Async => {
                            return self.unsupported_expr("object literal method");
                        }
                    }
                }
                PropertyDefinition::CoverInitializedName(_identifier, _) => {
                    return self.unsupported_expr("object literal shorthand");
                }
                PropertyDefinition::Property(PropertyName::Computed(_), _) => {
                    return self.unsupported_expr("computed object key");
                }
                PropertyDefinition::SpreadObject(_) => {
                    return self.unsupported_expr("object literal spread");
                }
            }
        }
        TypedExpr::from_info(
            ValueInfo {
                kind: ValueKind::Object,
                heap_shape: Some(Box::new(HeapShape::Object(shape))),
                function_targets: BTreeSet::new(),
            },
            ExprIr::ObjectLiteral(properties),
        )
    }

    fn lower_property_access(&mut self, access: &PropertyAccess) -> TypedExpr {
        let PropertyAccess::Simple(access) = access else {
            return self.unsupported_expr("unsupported property access");
        };

        let target = self.lower_property_target(access.target());
        match target.kind {
            ValueKind::Object => self.lower_object_property_key(target, access.field()),
            ValueKind::Array => self.lower_array_index_key(target, access.field()),
            ValueKind::Arguments => self.lower_arguments_index_key(target, access.field()),
            ValueKind::Dynamic => self.unsupported_expr("property access on dynamic target"),
            _ => self.unsupported_expr("property access on non-object target"),
        }
    }

    fn lower_property_target(&mut self, target: &Expression) -> TypedExpr {
        self.lower_expression(target)
    }

    fn lower_object_property_key(
        &mut self,
        target: TypedExpr,
        field: &PropertyAccessField,
    ) -> TypedExpr {
        let key = match field {
            PropertyAccessField::Const(name) => {
                PropertyKeyIr::StaticString(self.interner.resolve_expect(name.sym()).to_string())
            }
            PropertyAccessField::Expr(expr) => {
                if let Some(key) = self.try_static_string_key(expr) {
                    PropertyKeyIr::StaticString(key)
                } else {
                    let lowered = self.lower_expression(expr);
                    if lowered.kind != ValueKind::String {
                        return self.unsupported_expr("object property key must be string");
                    }
                    PropertyKeyIr::StringExpr(Box::new(lowered))
                }
            }
        };
        let info = match &key {
            PropertyKeyIr::StaticString(key) => {
                if let Some(ObjectShapeProperty::Accessor {
                    getter: Some(getter),
                    ..
                }) = self.read_object_shape_property(&target, key)
                {
                    self.merge_function_this_info(&getter.function_id, target.value_info());
                }
                self.read_object_shape(&target, key).unwrap_or(ValueInfo {
                    kind: ValueKind::Dynamic,
                    heap_shape: None,
                    function_targets: BTreeSet::new(),
                })
            }
            PropertyKeyIr::StringExpr(_) => ValueInfo {
                kind: ValueKind::Dynamic,
                heap_shape: None,
                function_targets: BTreeSet::new(),
            },
            PropertyKeyIr::ArrayIndex(_) | PropertyKeyIr::ArrayLength => unreachable!(),
        };
        TypedExpr::from_info(
            info,
            ExprIr::PropertyRead {
                target: Box::new(target),
                key,
            },
        )
    }

    fn lower_array_index_key(
        &mut self,
        target: TypedExpr,
        field: &PropertyAccessField,
    ) -> TypedExpr {
        if let PropertyAccessField::Const(name) = field {
            let name = self.interner.resolve_expect(name.sym()).to_string();
            if name == "length" {
                return TypedExpr::from_info(
                    ValueInfo {
                        kind: ValueKind::Number,
                        heap_shape: None,
                        function_targets: BTreeSet::new(),
                    },
                    ExprIr::PropertyRead {
                        target: Box::new(target),
                        key: PropertyKeyIr::ArrayLength,
                    },
                );
            }
            return self.unsupported_expr("unsupported array dot access");
        }
        let PropertyAccessField::Expr(expr) = field else {
            return self.unsupported_expr("unsupported array dot access");
        };
        let index = self.lower_expression(expr);
        if index.kind != ValueKind::Number {
            return self.unsupported_expr("array index must be number");
        }
        let info = self.read_array_shape(&target, &index).unwrap_or(ValueInfo {
            kind: ValueKind::Dynamic,
            heap_shape: None,
            function_targets: BTreeSet::new(),
        });
        TypedExpr::from_info(
            info,
            ExprIr::PropertyRead {
                target: Box::new(target),
                key: PropertyKeyIr::ArrayIndex(Box::new(index)),
            },
        )
    }

    fn lower_arguments_index_key(
        &mut self,
        target: TypedExpr,
        field: &PropertyAccessField,
    ) -> TypedExpr {
        if let PropertyAccessField::Const(name) = field {
            let name = self.interner.resolve_expect(name.sym()).to_string();
            if name == "length" {
                return TypedExpr::from_info(
                    ValueInfo {
                        kind: ValueKind::Number,
                        heap_shape: None,
                        function_targets: BTreeSet::new(),
                    },
                    ExprIr::PropertyRead {
                        target: Box::new(target),
                        key: PropertyKeyIr::ArrayLength,
                    },
                );
            }
            return self.unsupported_expr("unsupported arguments dot access");
        }
        let PropertyAccessField::Expr(expr) = field else {
            return self.unsupported_expr("unsupported arguments access");
        };
        let index = self.lower_expression(expr);
        if index.kind != ValueKind::Number {
            return self.unsupported_expr("arguments index must be number");
        }
        TypedExpr::from_info(
            ValueInfo {
                kind: ValueKind::Dynamic,
                heap_shape: None,
                function_targets: BTreeSet::new(),
            },
            ExprIr::PropertyRead {
                target: Box::new(target),
                key: PropertyKeyIr::ArrayIndex(Box::new(index)),
            },
        )
    }

    fn lower_assign(&mut self, op: AssignOp, lhs: &AssignTarget, rhs: &Expression) -> TypedExpr {
        match op {
            AssignOp::Assign => match lhs {
                AssignTarget::Identifier(identifier) => {
                    let name = self.interner.resolve_expect(identifier.sym()).to_string();
                    let Some(binding) = self.lookup_binding(&name) else {
                        self.unsupported_with_message(format!(
                            "unsupported in porffor wasm-aot first slice: unbound identifier `{name}`"
                        ));
                        return TypedExpr::undefined();
                    };
                    if binding.mode == BindingMode::Const {
                        return self.unsupported_expr("assignment to const binding");
                    }

                    let value = self.lower_expression(rhs);
                    if binding.mode != BindingMode::Var
                        && binding.kind != ValueKind::Dynamic
                        && value.kind != binding.kind
                    {
                        return self.unsupported_expr("assignment changes binding kind");
                    }
                    self.set_binding_value_info(&name, value.value_info());

                    TypedExpr::from_info(
                        value.value_info(),
                        ExprIr::AssignIdentifier {
                            name,
                            value: Box::new(value),
                        },
                    )
                }
                AssignTarget::Access(access) => self.lower_property_assign(access, rhs),
                _ => self.unsupported_expr("non-identifier assignment target"),
            },
            AssignOp::Add | AssignOp::Sub | AssignOp::Mul | AssignOp::Div | AssignOp::Mod => {
                let AssignTarget::Identifier(identifier) = lhs else {
                    return self.unsupported_expr("unsupported property assignment operator");
                };

                let name = self.interner.resolve_expect(identifier.sym()).to_string();
                let Some(binding) = self.lookup_binding(&name) else {
                    self.unsupported_with_message(format!(
                        "unsupported in porffor wasm-aot first slice: unbound identifier `{name}`"
                    ));
                    return TypedExpr::undefined();
                };
                if binding.mode == BindingMode::Const {
                    return self.unsupported_expr("assignment to const binding");
                }

                if binding.kind != ValueKind::Number {
                    return self.unsupported_expr("compound assignment on non-number binding");
                }
                let value = self.lower_expression(rhs);
                if value.kind != ValueKind::Number {
                    return self.unsupported_expr("coercive compound assignment");
                }
                self.set_binding_value_info(
                    &name,
                    ValueInfo {
                        kind: ValueKind::Number,
                        heap_shape: None,
                        function_targets: BTreeSet::new(),
                    },
                );
                let op = match op {
                    AssignOp::Add => ArithmeticBinaryOp::Add,
                    AssignOp::Sub => ArithmeticBinaryOp::Sub,
                    AssignOp::Mul => ArithmeticBinaryOp::Mul,
                    AssignOp::Div => ArithmeticBinaryOp::Div,
                    AssignOp::Mod => ArithmeticBinaryOp::Mod,
                    _ => unreachable!(),
                };
                TypedExpr::from_info(
                    ValueInfo {
                        kind: ValueKind::Number,
                        heap_shape: None,
                        function_targets: BTreeSet::new(),
                    },
                    ExprIr::CompoundAssignIdentifier {
                        name,
                        op,
                        value: Box::new(value),
                    },
                )
            }
            AssignOp::BoolAnd | AssignOp::BoolOr | AssignOp::Coalesce => {
                self.unsupported_expr("logical assignment")
            }
            AssignOp::Exp => self.unsupported_expr("exponentiation assignment"),
            AssignOp::And
            | AssignOp::Or
            | AssignOp::Xor
            | AssignOp::Shl
            | AssignOp::Shr
            | AssignOp::Ushr => self.unsupported_expr("unsupported compound assignment operator"),
        }
    }

    fn lower_property_assign(&mut self, access: &PropertyAccess, rhs: &Expression) -> TypedExpr {
        let PropertyAccess::Simple(access) = access else {
            return self.unsupported_expr("unsupported property access");
        };

        let target = self.lower_property_target(access.target());

        match target.kind {
            ValueKind::Object => {
                let key = match access.field() {
                    PropertyAccessField::Const(name) => PropertyKeyIr::StaticString(
                        self.interner.resolve_expect(name.sym()).to_string(),
                    ),
                    PropertyAccessField::Expr(expr) => {
                        if let Some(key) = self.try_static_string_key(expr) {
                            PropertyKeyIr::StaticString(key)
                        } else {
                            let lowered = self.lower_expression(expr);
                            if lowered.kind != ValueKind::String {
                                return self.unsupported_expr("object property key must be string");
                            }
                            PropertyKeyIr::StringExpr(Box::new(lowered))
                        }
                    }
                };
                let value = self.lower_expression(rhs);
                if let PropertyKeyIr::StaticString(key_name) = &key {
                    if let Some(ObjectShapeProperty::Accessor {
                        setter: Some(setter),
                        ..
                    }) = self.read_object_shape_property(&target, key_name)
                    {
                        self.merge_function_this_info(&setter.function_id, target.value_info());
                    }
                }
                self.update_written_shape(access.target(), &key, &value.value_info());
                TypedExpr::from_info(
                    value.value_info(),
                    ExprIr::PropertyWrite {
                        target: Box::new(target),
                        key,
                        value: Box::new(value),
                    },
                )
            }
            ValueKind::Array => {
                let PropertyAccessField::Expr(expr) = access.field() else {
                    return self.unsupported_expr("unsupported array dot access");
                };
                let index = self.lower_expression(expr);
                if index.kind != ValueKind::Number {
                    return self.unsupported_expr("array index must be number");
                }
                let value = self.lower_expression(rhs);
                let key = PropertyKeyIr::ArrayIndex(Box::new(index));
                self.update_written_shape(access.target(), &key, &value.value_info());
                TypedExpr::from_info(
                    value.value_info(),
                    ExprIr::PropertyWrite {
                        target: Box::new(target),
                        key,
                        value: Box::new(value),
                    },
                )
            }
            ValueKind::Arguments => {
                let PropertyAccessField::Expr(expr) = access.field() else {
                    return self.unsupported_expr("unsupported arguments dot access");
                };
                let index = self.lower_expression(expr);
                if index.kind != ValueKind::Number {
                    return self.unsupported_expr("arguments index must be number");
                }
                let value = self.lower_expression(rhs);
                let key = PropertyKeyIr::ArrayIndex(Box::new(index));
                TypedExpr::from_info(
                    value.value_info(),
                    ExprIr::PropertyWrite {
                        target: Box::new(target),
                        key,
                        value: Box::new(value),
                    },
                )
            }
            ValueKind::Dynamic => self.unsupported_expr("property access on dynamic target"),
            _ => self.unsupported_expr("property access on non-object target"),
        }
    }

    fn lower_update(&mut self, op: UpdateOp, target: &UpdateTarget) -> TypedExpr {
        let UpdateTarget::Identifier(identifier) = target else {
            return self.unsupported_expr("non-identifier update target");
        };

        let name = self.interner.resolve_expect(identifier.sym()).to_string();
        let Some(binding) = self.lookup_binding(&name) else {
            self.unsupported_with_message(format!(
                "unsupported in porffor wasm-aot first slice: unbound identifier `{name}`"
            ));
            return TypedExpr::undefined();
        };
        if binding.mode == BindingMode::Const {
            return self.unsupported_expr("update of const binding");
        }
        if binding.kind != ValueKind::Number {
            return self.unsupported_expr("numeric update on non-number binding");
        }
        self.set_binding_value_info(
            &name,
            ValueInfo {
                kind: ValueKind::Number,
                heap_shape: None,
                function_targets: BTreeSet::new(),
            },
        );

        let (op, return_mode) = match op {
            UpdateOp::IncrementPost => (NumericUpdateOp::Increment, UpdateReturnMode::Postfix),
            UpdateOp::IncrementPre => (NumericUpdateOp::Increment, UpdateReturnMode::Prefix),
            UpdateOp::DecrementPost => (NumericUpdateOp::Decrement, UpdateReturnMode::Postfix),
            UpdateOp::DecrementPre => (NumericUpdateOp::Decrement, UpdateReturnMode::Prefix),
        };

        TypedExpr::from_info(
            ValueInfo {
                kind: ValueKind::Number,
                heap_shape: None,
                function_targets: BTreeSet::new(),
            },
            ExprIr::UpdateIdentifier {
                name,
                op,
                return_mode,
            },
        )
    }

    fn lower_unary(&mut self, op: UnaryOp, target: &Expression) -> TypedExpr {
        let target = self.lower_expression(target);
        match op {
            UnaryOp::Plus => {
                if target.kind != ValueKind::Number {
                    return self.unsupported_expr("coercive unary plus");
                }
                TypedExpr::from_info(
                    ValueInfo {
                        kind: ValueKind::Number,
                        heap_shape: None,
                        function_targets: BTreeSet::new(),
                    },
                    ExprIr::UnaryNumber {
                        op: UnaryNumericOp::Plus,
                        expr: Box::new(target),
                    },
                )
            }
            UnaryOp::Minus => {
                if target.kind != ValueKind::Number {
                    return self.unsupported_expr("coercive unary minus");
                }
                TypedExpr::from_info(
                    ValueInfo {
                        kind: ValueKind::Number,
                        heap_shape: None,
                        function_targets: BTreeSet::new(),
                    },
                    ExprIr::UnaryNumber {
                        op: UnaryNumericOp::Minus,
                        expr: Box::new(target),
                    },
                )
            }
            UnaryOp::Not => TypedExpr::from_info(
                ValueInfo {
                    kind: ValueKind::Boolean,
                    heap_shape: None,
                    function_targets: BTreeSet::new(),
                },
                ExprIr::LogicalNot {
                    expr: Box::new(target),
                },
            ),
            UnaryOp::Tilde | UnaryOp::TypeOf | UnaryOp::Delete | UnaryOp::Void => {
                self.unsupported_expr("unsupported unary operator")
            }
        }
    }

    fn lower_binary(&mut self, op: BinaryOp, lhs: &Expression, rhs: &Expression) -> TypedExpr {
        match op {
            BinaryOp::Arithmetic(arithmetic) => self.lower_arithmetic(arithmetic, lhs, rhs),
            BinaryOp::Relational(relational) => self.lower_relational(relational, lhs, rhs),
            BinaryOp::Logical(logical) => self.lower_logical(logical, lhs, rhs),
            BinaryOp::Bitwise(_) => self.unsupported_expr("unsupported binary operator"),
            BinaryOp::Comma => self.unsupported_expr("comma operator"),
        }
    }

    fn lower_arithmetic(
        &mut self,
        arithmetic: ArithmeticOp,
        lhs: &Expression,
        rhs: &Expression,
    ) -> TypedExpr {
        let lhs = self.lower_expression(lhs);
        let rhs = self.lower_expression(rhs);

        match arithmetic {
            ArithmeticOp::Add => {
                let lhs = match self.coerce_expr_to_number(lhs) {
                    Some(lhs) => lhs,
                    None => return self.unsupported_expr("string or coercive `+`"),
                };
                let rhs = match self.coerce_expr_to_number(rhs) {
                    Some(rhs) => rhs,
                    None => return self.unsupported_expr("string or coercive `+`"),
                };
                if lhs.kind != ValueKind::Number || rhs.kind != ValueKind::Number {
                    return self.unsupported_expr("string or coercive `+`");
                }
                TypedExpr::from_info(
                    ValueInfo {
                        kind: ValueKind::Number,
                        heap_shape: None,
                        function_targets: BTreeSet::new(),
                    },
                    ExprIr::BinaryNumber {
                        op: ArithmeticBinaryOp::Add,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    },
                )
            }
            ArithmeticOp::Sub | ArithmeticOp::Mul | ArithmeticOp::Div | ArithmeticOp::Mod => {
                let lhs = match self.coerce_expr_to_number(lhs) {
                    Some(lhs) => lhs,
                    None => return self.unsupported_expr("coercive numeric operator"),
                };
                let rhs = match self.coerce_expr_to_number(rhs) {
                    Some(rhs) => rhs,
                    None => return self.unsupported_expr("coercive numeric operator"),
                };
                if lhs.kind != ValueKind::Number || rhs.kind != ValueKind::Number {
                    return self.unsupported_expr("coercive numeric operator");
                }
                let op = match arithmetic {
                    ArithmeticOp::Sub => ArithmeticBinaryOp::Sub,
                    ArithmeticOp::Mul => ArithmeticBinaryOp::Mul,
                    ArithmeticOp::Div => ArithmeticBinaryOp::Div,
                    ArithmeticOp::Mod => ArithmeticBinaryOp::Mod,
                    ArithmeticOp::Add | ArithmeticOp::Exp => unreachable!(),
                };
                TypedExpr::from_info(
                    ValueInfo {
                        kind: ValueKind::Number,
                        heap_shape: None,
                        function_targets: BTreeSet::new(),
                    },
                    ExprIr::BinaryNumber {
                        op,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    },
                )
            }
            ArithmeticOp::Exp => self.unsupported_expr("exponentiation operator"),
        }
    }

    fn lower_relational(
        &mut self,
        relational: RelationalOp,
        lhs: &Expression,
        rhs: &Expression,
    ) -> TypedExpr {
        let lhs = self.lower_expression(lhs);
        let rhs = self.lower_expression(rhs);

        match relational {
            RelationalOp::LessThan
            | RelationalOp::LessThanOrEqual
            | RelationalOp::GreaterThan
            | RelationalOp::GreaterThanOrEqual => {
                let lhs = match self.coerce_expr_to_number(lhs) {
                    Some(lhs) => lhs,
                    None => return self.unsupported_expr("coercive comparison operator"),
                };
                let rhs = match self.coerce_expr_to_number(rhs) {
                    Some(rhs) => rhs,
                    None => return self.unsupported_expr("coercive comparison operator"),
                };
                if lhs.kind != ValueKind::Number || rhs.kind != ValueKind::Number {
                    return self.unsupported_expr("coercive comparison operator");
                }
                let op = match relational {
                    RelationalOp::LessThan => RelationalBinaryOp::LessThan,
                    RelationalOp::LessThanOrEqual => RelationalBinaryOp::LessThanOrEqual,
                    RelationalOp::GreaterThan => RelationalBinaryOp::GreaterThan,
                    RelationalOp::GreaterThanOrEqual => RelationalBinaryOp::GreaterThanOrEqual,
                    _ => unreachable!(),
                };
                TypedExpr::from_info(
                    ValueInfo {
                        kind: ValueKind::Boolean,
                        heap_shape: None,
                        function_targets: BTreeSet::new(),
                    },
                    ExprIr::CompareNumber {
                        op,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    },
                )
            }
            RelationalOp::StrictEqual | RelationalOp::StrictNotEqual => {
                let op = match relational {
                    RelationalOp::StrictEqual => EqualityBinaryOp::StrictEqual,
                    RelationalOp::StrictNotEqual => EqualityBinaryOp::StrictNotEqual,
                    _ => unreachable!(),
                };
                TypedExpr::from_info(
                    ValueInfo {
                        kind: ValueKind::Boolean,
                        heap_shape: None,
                        function_targets: BTreeSet::new(),
                    },
                    ExprIr::StrictEquality {
                        op,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    },
                )
            }
            RelationalOp::Equal | RelationalOp::NotEqual => {
                self.unsupported_expr("loose equality operator")
            }
            RelationalOp::In | RelationalOp::InstanceOf => {
                self.unsupported_expr("unsupported comparison operator")
            }
        }
    }

    fn lower_logical(
        &mut self,
        logical: LogicalOp,
        lhs: &Expression,
        rhs: &Expression,
    ) -> TypedExpr {
        let lhs = self.lower_expression(lhs);
        let rhs = self.lower_expression(rhs);

        match logical {
            LogicalOp::And | LogicalOp::Or => {
                if lhs.kind != rhs.kind
                    && lhs.kind != ValueKind::Dynamic
                    && rhs.kind != ValueKind::Dynamic
                {
                    return self.unsupported_expr("mixed-kind logical operator");
                }
                let op = match logical {
                    LogicalOp::And => LogicalBinaryOp::And,
                    LogicalOp::Or => LogicalBinaryOp::Or,
                    LogicalOp::Coalesce => unreachable!(),
                };
                TypedExpr::from_info(
                    if lhs.kind == rhs.kind {
                        ValueInfo {
                            kind: lhs.kind,
                            heap_shape: self.merge_operand_shapes(&lhs, &rhs),
                            function_targets: if lhs.kind == ValueKind::Function {
                                let mut targets = lhs.function_targets.clone();
                                targets.extend(rhs.function_targets.clone());
                                targets
                            } else {
                                BTreeSet::new()
                            },
                        }
                    } else {
                        ValueInfo {
                            kind: ValueKind::Dynamic,
                            heap_shape: None,
                            function_targets: BTreeSet::new(),
                        }
                    },
                    ExprIr::LogicalShortCircuit {
                        op,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    },
                )
            }
            LogicalOp::Coalesce => self.unsupported_expr("nullish coalescing operator"),
        }
    }

    fn try_static_string_key(&self, expr: &Expression) -> Option<String> {
        let Expression::Literal(literal) = expr else {
            return None;
        };
        let LiteralKind::String(sym) = literal.kind() else {
            return None;
        };
        Some(self.interner.resolve_expect(*sym).to_string())
    }

    fn read_object_shape(&self, target: &TypedExpr, key: &str) -> Option<ValueInfo> {
        let property = self.read_object_shape_property(target, key)?;
        Some(match property {
            ObjectShapeProperty::Data(info) => info,
            ObjectShapeProperty::Accessor {
                getter: Some(getter),
                ..
            } => self.accessor_return_info(&getter.function_id),
            ObjectShapeProperty::Accessor { getter: None, .. } => ValueInfo::undefined(),
        })
    }

    fn read_object_shape_property(
        &self,
        target: &TypedExpr,
        key: &str,
    ) -> Option<ObjectShapeProperty> {
        let HeapShape::Object(shape) = target.heap_shape.as_deref()? else {
            return None;
        };
        shape.properties.get(key).cloned()
    }

    fn read_array_shape(&self, target: &TypedExpr, index: &TypedExpr) -> Option<ValueInfo> {
        let HeapShape::Array(shape) = target.heap_shape.as_deref()? else {
            return None;
        };
        let index = self.constant_array_index(index)?;
        Some(
            shape
                .elements
                .get(index)
                .cloned()
                .unwrap_or_else(ValueInfo::undefined),
        )
    }

    fn constant_array_index(&self, index: &TypedExpr) -> Option<usize> {
        let ExprIr::Number(bits) = &index.expr else {
            return None;
        };
        let value = f64::from_bits(*bits);
        if !value.is_finite() || value < 0.0 || value.fract() != 0.0 {
            return None;
        }
        Some(value as usize)
    }

    fn update_written_shape(&mut self, target: &Expression, key: &PropertyKeyIr, value: &ValueInfo) {
        let Some((root, mut path)) = self.binding_shape_path(target) else {
            return;
        };
        path.push(key.clone());
        self.update_binding_shape_path(&root, &path, value.clone());
    }

    fn binding_shape_path(&mut self, expr: &Expression) -> Option<(String, Vec<PropertyKeyIr>)> {
        match expr {
            Expression::Identifier(identifier) => Some((
                self.interner.resolve_expect(identifier.sym()).to_string(),
                Vec::new(),
            )),
            Expression::PropertyAccess(PropertyAccess::Simple(access)) => {
                let (root, mut path) = self.binding_shape_path(access.target())?;
                let key = match access.field() {
                    PropertyAccessField::Const(name) => {
                        let name = self.interner.resolve_expect(name.sym()).to_string();
                        if name == "length" {
                            PropertyKeyIr::ArrayLength
                        } else {
                            PropertyKeyIr::StaticString(name.to_string())
                        }
                    }
                    PropertyAccessField::Expr(expr) => {
                        if let Some(key) = self.try_static_string_key(expr) {
                            PropertyKeyIr::StaticString(key)
                        } else if let Some(index) = self.try_constant_array_index_expr(expr) {
                            PropertyKeyIr::ArrayIndex(Box::new(TypedExpr::from_info(
                                ValueInfo {
                                    kind: ValueKind::Number,
                                    heap_shape: None,
                                    function_targets: BTreeSet::new(),
                                },
                                ExprIr::Number(index.to_bits()),
                            )))
                        } else {
                            return Some((root, path));
                        }
                    }
                };
                path.push(key);
                Some((root, path))
            }
            _ => None,
        }
    }

    fn update_binding_shape_path(&mut self, name: &str, path: &[PropertyKeyIr], value: ValueInfo) {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(binding) = scope.get_mut(name) {
                let binding_value = ValueInfo {
                    kind: binding.kind,
                    heap_shape: binding.heap_shape.clone(),
                    function_targets: binding.function_targets.clone(),
                };
                let next = Self::apply_shape_write(binding_value, path, value);
                binding.kind = next.kind;
                binding.heap_shape = next.heap_shape;
                binding.function_targets = next.function_targets;
                return;
            }
        }
        if let Some(binding) = self.var_bindings.get_mut(name) {
            let binding_value = ValueInfo {
                kind: binding.kind,
                heap_shape: binding.heap_shape.clone(),
                function_targets: binding.function_targets.clone(),
            };
            let next = Self::apply_shape_write(binding_value, path, value);
            binding.kind = next.kind;
            binding.heap_shape = next.heap_shape;
            binding.function_targets = next.function_targets;
        }
    }

    fn apply_shape_write(mut target: ValueInfo, path: &[PropertyKeyIr], value: ValueInfo) -> ValueInfo {
        if path.is_empty() {
            return value;
        }
        let Some(shape) = target.heap_shape.as_mut() else {
            return target;
        };
        match (shape.as_mut(), &path[0]) {
            (HeapShape::Object(object), PropertyKeyIr::StaticString(key)) => {
                if path.len() == 1 {
                    match object.properties.get(key).cloned() {
                        Some(ObjectShapeProperty::Accessor { getter, setter }) => {
                            object.properties.insert(
                                key.clone(),
                                ObjectShapeProperty::Accessor { getter, setter },
                            );
                        }
                        _ => {
                            object
                                .properties
                                .insert(key.clone(), ObjectShapeProperty::Data(value));
                        }
                    }
                } else if let Some(ObjectShapeProperty::Data(existing)) =
                    object.properties.get(key).cloned()
                {
                    object.properties.insert(
                        key.clone(),
                        ObjectShapeProperty::Data(Self::apply_shape_write(
                            existing,
                            &path[1..],
                            value,
                        )),
                    );
                }
            }
            (HeapShape::Object(_), PropertyKeyIr::StringExpr(_)) => {
                target.heap_shape = None;
            }
            (HeapShape::Array(array), PropertyKeyIr::ArrayIndex(index)) => {
                let Some(index) = Self::constant_array_index_static(index) else {
                    target.heap_shape = None;
                    return target;
                };
                if array.elements.len() <= index {
                    array.elements.resize(index + 1, ValueInfo::undefined());
                }
                if path.len() == 1 {
                    array.elements[index] = value;
                } else {
                    let existing = array.elements[index].clone();
                    array.elements[index] = Self::apply_shape_write(existing, &path[1..], value);
                }
            }
            (HeapShape::Array(_), PropertyKeyIr::ArrayLength) => {}
            (HeapShape::Array(_), _) => {
                target.heap_shape = None;
            }
            (HeapShape::Object(_), PropertyKeyIr::ArrayIndex(_) | PropertyKeyIr::ArrayLength) => {
                target.heap_shape = None;
            }
        }
        target
    }

    fn constant_array_index_static(index: &TypedExpr) -> Option<usize> {
        let ExprIr::Number(bits) = &index.expr else {
            return None;
        };
        let value = f64::from_bits(*bits);
        if !value.is_finite() || value < 0.0 || value.fract() != 0.0 {
            return None;
        }
        Some(value as usize)
    }

    fn try_constant_array_index_expr(&self, expr: &Expression) -> Option<f64> {
        let Expression::Literal(literal) = expr else {
            return None;
        };
        match literal.kind() {
            LiteralKind::Num(value) => {
                if value.is_finite() && *value >= 0.0 && value.fract() == 0.0 {
                    Some(*value)
                } else {
                    None
                }
            }
            LiteralKind::Int(value) if *value >= 0 => Some(*value as f64),
            _ => None,
        }
    }

    fn declare_binding(&mut self, name: String, info: BindingInfo) {
        self.scopes
            .last_mut()
            .expect("scope stack must exist")
            .insert(name, info);
    }

    fn set_binding_value_info(&mut self, name: &str, info: ValueInfo) -> Option<()> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(binding) = scope.get_mut(name) {
                binding.kind = info.kind;
                binding.heap_shape = info.heap_shape.clone();
                binding.function_targets = info.function_targets.clone();
                self.record_param_kind(name, info.kind);
                return Some(());
            }
        }
        if let Some(binding) = self.var_bindings.get_mut(name) {
            binding.kind = info.kind;
            binding.heap_shape = info.heap_shape;
            binding.function_targets = info.function_targets;
            return Some(());
        }
        None
    }

    fn coerce_expr_to_number(&mut self, mut expr: TypedExpr) -> Option<TypedExpr> {
        if expr.kind == ValueKind::Number {
            return Some(expr);
        }
        if expr.kind != ValueKind::Dynamic {
            return None;
        }

        match &expr.expr {
            ExprIr::Identifier(name) => {
                self.set_binding_kind(name, ValueKind::Number)?;
                expr.kind = ValueKind::Number;
                expr.heap_shape = None;
                expr.function_targets.clear();
                Some(expr)
            }
            ExprIr::CallNamed { name, .. } => {
                let next_info = {
                    let function_id = self.visible_function_names.get(name)?;
                    let signature = self.function_signatures.get(function_id)?;
                    self.merge_return_infos(
                        ValueInfo {
                            kind: signature.return_kind,
                            heap_shape: signature.return_shape.clone(),
                            function_targets: signature.return_targets.clone(),
                        },
                        ValueInfo {
                            kind: ValueKind::Number,
                            heap_shape: None,
                            function_targets: BTreeSet::new(),
                        },
                    )
                };
                if let Some(function_id) = self.visible_function_names.get(name).cloned() {
                    if let Some(signature) = self.function_signatures.get_mut(&function_id) {
                        signature.return_kind = next_info.kind;
                        signature.return_shape = next_info.heap_shape.clone();
                        signature.return_targets = next_info.function_targets.clone();
                    }
                }
                expr.kind = ValueKind::Number;
                expr.heap_shape = None;
                expr.function_targets.clear();
                Some(expr)
            }
            ExprIr::CallIndirect { callee, .. } => {
                let function_id = self.resolve_single_function_target(callee)?;
                let next_info = {
                    let signature = self.function_signatures.get(&function_id)?;
                    self.merge_return_infos(
                        ValueInfo {
                            kind: signature.return_kind,
                            heap_shape: signature.return_shape.clone(),
                            function_targets: signature.return_targets.clone(),
                        },
                        ValueInfo {
                            kind: ValueKind::Number,
                            heap_shape: None,
                            function_targets: BTreeSet::new(),
                        },
                    )
                };
                if let Some(signature) = self.function_signatures.get_mut(&function_id) {
                    signature.return_kind = next_info.kind;
                    signature.return_shape = next_info.heap_shape.clone();
                    signature.return_targets = next_info.function_targets.clone();
                }
                expr.kind = ValueKind::Number;
                expr.heap_shape = None;
                expr.function_targets.clear();
                Some(expr)
            }
            _ => None,
        }
    }

    fn capture_value_info(&self, owner_id: &str, name: &str) -> ValueInfo {
        if name == LEXICAL_THIS_NAME {
            return self
                .function_signatures
                .get(owner_id)
                .map(|signature| signature.this_info.clone())
                .unwrap_or_else(ValueInfo::undefined);
        }
        if name == LEXICAL_ARGUMENTS_NAME {
            return ValueInfo {
                kind: ValueKind::Arguments,
                heap_shape: None,
                function_targets: BTreeSet::new(),
            };
        }
        let Some(owner) = self.analysis.owner_plans.get(owner_id) else {
            return ValueInfo {
                kind: ValueKind::Dynamic,
                heap_shape: None,
                function_targets: BTreeSet::new(),
            };
        };
        if let Some(function_id) = owner.function_bindings.get(name) {
            return ValueInfo {
                kind: ValueKind::Function,
                heap_shape: None,
                function_targets: BTreeSet::from([function_id.clone()]),
            };
        }
        ValueInfo {
            kind: ValueKind::Dynamic,
            heap_shape: None,
            function_targets: BTreeSet::new(),
        }
    }

    fn lookup_binding(&self, name: &str) -> Option<BindingInfo> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).cloned())
            .or_else(|| {
                self.var_bindings.get(name).map(|binding| {
                    let mode = BindingMode::Var;
                    let kind = binding.kind;
                    let heap_shape = binding.heap_shape.clone();
                    let function_targets = binding.function_targets.clone();
                    BindingInfo {
                        mode,
                        kind,
                        heap_shape,
                        function_targets,
                    }
                })
            })
    }

    fn set_binding_kind(&mut self, name: &str, kind: ValueKind) -> Option<()> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(binding) = scope.get_mut(name) {
                binding.kind = kind;
                binding.heap_shape = None;
                binding.function_targets.clear();
                self.record_param_kind(name, kind);
                return Some(());
            }
        }
        if let Some(binding) = self.var_bindings.get_mut(name) {
            binding.kind = kind;
            binding.heap_shape = None;
            binding.function_targets.clear();
            return Some(());
        }
        None
    }

    fn record_param_kind(&mut self, name: &str, kind: ValueKind) {
        let Some(function_id) = &self.current_function_id else {
            return;
        };
        let Some(index) = self
            .current_param_names
            .iter()
            .position(|param| param == name)
        else {
            return;
        };
        if let Some(signature) = self.function_signatures.get_mut(function_id) {
            if let Some(param) = signature.params.get_mut(index) {
                if param.is_rest {
                    return;
                }
                param.kind = match param.kind {
                    ValueKind::Dynamic | ValueKind::Undefined => kind,
                    existing if existing == kind => existing,
                    _ => ValueKind::Dynamic,
                };
            }
        }
    }

    fn set_var_kind(&mut self, name: &str, kind: ValueKind) {
        if let Some(binding) = self.var_bindings.get_mut(name) {
            binding.kind = kind;
            binding.heap_shape = None;
            binding.function_targets.clear();
        }
    }

    fn merge_var_bindings(
        &self,
        left: &BTreeMap<String, VarBindingInfo>,
        right: &BTreeMap<String, VarBindingInfo>,
    ) -> BTreeMap<String, VarBindingInfo> {
        let mut merged = BTreeMap::new();
        for name in left.keys().chain(right.keys()) {
            if merged.contains_key(name) {
                continue;
            }
            let left_info = left.get(name).map(|binding| ValueInfo {
                kind: binding.kind,
                heap_shape: binding.heap_shape.clone(),
                function_targets: binding.function_targets.clone(),
            });
            let right_info = right.get(name).map(|binding| ValueInfo {
                kind: binding.kind,
                heap_shape: binding.heap_shape.clone(),
                function_targets: binding.function_targets.clone(),
            });
            let info = match (left_info, right_info) {
                (Some(lhs), Some(rhs)) => self.merge_value_infos(lhs, rhs),
                (Some(lhs), None) => lhs,
                (None, Some(rhs)) => rhs,
                (None, None) => continue,
            };
            merged.insert(
                name.clone(),
                VarBindingInfo {
                    kind: info.kind,
                    heap_shape: info.heap_shape,
                    function_targets: info.function_targets,
                },
            );
        }
        merged
    }

    fn merge_value_kinds(&self, left: ValueKind, right: ValueKind) -> ValueKind {
        if left == right {
            left
        } else {
            ValueKind::Dynamic
        }
    }

    fn merge_operand_shapes(&self, left: &TypedExpr, right: &TypedExpr) -> Option<Box<HeapShape>> {
        if left.kind != right.kind {
            return None;
        }
        self.merge_heap_shapes(left.kind, &left.heap_shape, &right.heap_shape)
    }

    fn merge_heap_shapes(
        &self,
        kind: ValueKind,
        left: &Option<Box<HeapShape>>,
        right: &Option<Box<HeapShape>>,
    ) -> Option<Box<HeapShape>> {
        match kind {
            ValueKind::Object | ValueKind::Array if left == right => left.clone(),
            ValueKind::Object | ValueKind::Array => None,
            _ => None,
        }
    }

    fn merge_value_infos(&self, left: ValueInfo, right: ValueInfo) -> ValueInfo {
        if left.kind == ValueKind::Function && right.kind == ValueKind::Function {
            let mut function_targets = left.function_targets;
            function_targets.extend(right.function_targets);
            return ValueInfo {
                kind: ValueKind::Function,
                heap_shape: None,
                function_targets,
            };
        }
        if left.kind == right.kind {
            return ValueInfo {
                kind: left.kind,
                heap_shape: self.merge_heap_shapes(left.kind, &left.heap_shape, &right.heap_shape),
                function_targets: if left.kind == ValueKind::Function {
                    let mut function_targets = left.function_targets;
                    function_targets.extend(right.function_targets);
                    function_targets
                } else {
                    left.function_targets
                },
            };
        }
        ValueInfo {
            kind: self.merge_value_kinds(left.kind, right.kind),
            heap_shape: None,
            function_targets: BTreeSet::new(),
        }
    }

    fn merge_return_infos(&self, left: ValueInfo, right: ValueInfo) -> ValueInfo {
        if left.kind == ValueKind::Undefined {
            return right;
        }
        if right.kind == ValueKind::Undefined {
            return left;
        }
        self.merge_value_infos(left, right)
    }

    fn record_return_info(&mut self, info: ValueInfo) {
        self.current_return_info = Some(match self.current_return_info.take() {
            Some(existing) => self.merge_return_infos(existing, info.clone()),
            None => info.clone(),
        });
        if let Some(function_id) = &self.current_function_id {
            let next_info = self.function_signatures.get(function_id).map(|signature| {
                match signature.return_kind {
                    ValueKind::Undefined => info.clone(),
                    _ => self.merge_return_infos(
                        ValueInfo {
                            kind: signature.return_kind,
                            heap_shape: signature.return_shape.clone(),
                            function_targets: signature.return_targets.clone(),
                        },
                        info.clone(),
                    ),
                }
            });
            if let Some(next_info) = next_info {
                if let Some(signature) = self.function_signatures.get_mut(function_id) {
                    signature.return_kind = next_info.kind;
                    signature.return_shape = next_info.heap_shape;
                    signature.return_targets = next_info.function_targets;
                }
            }
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(BTreeMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn unsupported_expr(&mut self, feature: &str) -> TypedExpr {
        self.unsupported_with_message(format!(
            "unsupported in porffor wasm-aot first slice: {feature}"
        ));
        TypedExpr::undefined()
    }

    fn unsupported(&mut self, feature: &str) {
        self.unsupported_with_message(format!(
            "unsupported in porffor wasm-aot first slice: {feature}"
        ));
    }

    fn unsupported_with_message(&mut self, message: String) {
        self.diagnostics.push(IrDiagnostic {
            kind: IrDiagnosticKind::Unsupported,
            message,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use porffor_front::{parse, ParseOptions};

    fn lower_script(source: &str) -> ProgramIr {
        let source = parse(source, ParseOptions::script()).expect("script should parse");
        lower(&source)
    }

    #[test]
    fn lowers_simple_script_ir() {
        let program = lower_script("let x = 40; const y = 2; x + y;");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert_eq!(script.body.statements.len(), 3);
        assert_eq!(script.result_kind(), ValueKind::Number);
    }

    #[test]
    fn lowers_assignment_and_if_ir() {
        let program = lower_script("let x = 0; if (!x) { x = 5; } x;");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert_eq!(script.result_kind(), ValueKind::Number);
        assert!(matches!(script.body.statements[1], StatementIr::If { .. }));
        assert!(program.ir_summary().contains("ifs=1"));
        assert!(program.ir_summary().contains("assigns=1"));
    }

    #[test]
    fn lowers_loop_ir() {
        let program = lower_script("let i = 0; while (i < 3) { i = i + 1; continue; } i;");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert!(matches!(
            script.body.statements[1],
            StatementIr::While { .. }
        ));
        assert!(program.ir_summary().contains("whiles=1"));
        assert!(program.ir_summary().contains("continues=1"));
    }

    #[test]
    fn lowers_update_and_compound_ir() {
        let program = lower_script("let i = 2; let x = i++; x += ++i; x;");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert_eq!(script.result_kind(), ValueKind::Number);
        let summary = program.ir_summary();
        assert!(summary.contains("postfix_updates=1"));
        assert!(summary.contains("prefix_updates=1"));
        assert!(summary.contains("compound_assigns=1"));
    }

    #[test]
    fn lowers_switch_labels_and_debugger_ir() {
        let program = lower_script(
            "outer: while (true) { switch (2) { case 1: break; case 2: debugger; break outer; default: break; } }",
        );
        assert!(program.is_wasm_supported());
        let summary = program.ir_summary();
        assert!(summary.contains("switches=1"));
        assert!(summary.contains("labels=1"));
        assert!(summary.contains("debuggers=1"));
    }

    #[test]
    fn lowers_hoisted_var_ir() {
        let program = lower_script("x; var x = 1; if (true) { var y = 2; } y;");
        assert!(program.is_wasm_supported());
        let summary = program.ir_summary();
        assert!(summary.contains("vars=2"));
        let script = program.script.as_ref().expect("script ir should exist");
        assert!(matches!(script.body.statements[1], StatementIr::Var(_)));
    }

    #[test]
    fn lowers_top_level_functions_and_calls() {
        let program = lower_script("add(1, 2); function add(x, y) { return x + y; }");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert_eq!(script.functions.len(), 1);
        assert_eq!(script.functions[0].params.len(), 2);
        let summary = program.ir_summary();
        assert!(summary.contains("functions=1"));
        assert!(summary.contains("calls=1"));
        assert!(summary.contains("returns=1"));
    }

    #[test]
    fn lowers_objects_arrays_and_properties() {
        let program = lower_script("let o = { x: 1 }; let a = [1]; a[2] = 4; o.x;");
        assert!(program.is_wasm_supported());
        let summary = program.ir_summary();
        assert!(summary.contains("objects=1"));
        assert!(summary.contains("arrays=1"));
        assert!(summary.contains("property_reads=1"));
        assert!(summary.contains("property_writes=1"));
    }

    #[test]
    fn lowers_heap_shapes_and_array_length() {
        let program = lower_script(
            "function box() { let o = { inner: { x: 2 } }; return o; } let a = [1, 2, 3]; box().inner.x + a.length;",
        );
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert_eq!(script.functions[0].return_kind, ValueKind::Object);
        assert!(script.functions[0].return_shape.is_some());
        let summary = program.ir_summary();
        assert!(summary.contains("array_lengths=1"));
        assert!(summary.contains("heap_shapes="));
    }

    #[test]
    fn rejects_property_access_on_dynamic_after_kind_merge() {
        let program = lower_script("let v; if (true) { v = 1; } else { v = { x: 1 }; } v.x;");
        assert!(!program.is_wasm_supported());
        assert!(program.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("unsupported in porffor wasm-aot first slice")));
    }

    #[test]
    fn lowers_nested_function_declaration() {
        let program =
            lower_script("function outer() { function inner() { return 1; } return inner(); }");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert_eq!(script.functions.len(), 2);
        assert!(script.functions.iter().any(|function| function.is_nested));
        let summary = program.ir_summary();
        assert!(summary.contains("nested_functions=1"));
    }

    #[test]
    fn lowers_closure_capture_and_function_expression() {
        let program = lower_script(
            "function outer() { let x = 2; return function (y) { return x + y; }; } let f = outer(); f(3);",
        );
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert_eq!(script.functions.len(), 2);
        assert!(script.functions.iter().any(|function| function.is_expression));
        assert!(script
            .functions
            .iter()
            .any(|function| !function.captured_bindings.is_empty()));
        let summary = program.ir_summary();
        assert!(summary.contains("function_exprs=1"));
        assert!(summary.contains("closures=2"));
        assert!(summary.contains("captures=1"));
    }

    #[test]
    fn lowers_script_closure_capture() {
        let program = lower_script("let x = 1; function f() { return x; } f();");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert_eq!(script.owned_env_bindings.len(), 1);
        assert_eq!(script.owned_env_bindings[0].name, "x");
        assert_eq!(script.functions[0].captured_bindings[0].name, "x");
    }

    #[test]
    fn records_unsupported_var_destructuring_without_failing_lower() {
        let program = lower_script("var { x } = foo;");
        assert!(!program.is_wasm_supported());
        assert!(program
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("destructuring var declaration")));
    }

    #[test]
    fn rejects_assignment_to_const() {
        let program = lower_script("const x = 1; x = 2;");
        assert!(!program.is_wasm_supported());
        assert!(program
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("assignment to const binding")));
    }

    #[test]
    fn rejects_coercive_compound_assignment() {
        let program = lower_script("let s = \"a\"; s += \"b\";");
        assert!(!program.is_wasm_supported());
        assert!(program.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("compound assignment on non-number binding")));
    }

    #[test]
    fn rejects_label_on_unsupported_statement_kind() {
        let program = lower_script("label: 1;");
        assert!(!program.is_wasm_supported());
        assert!(program.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("label on unsupported statement kind")));
    }

    #[test]
    fn rejects_unknown_kind_numeric_use_after_var_merge() {
        let program = lower_script("var x; if (true) { x = 1; } else { x = \"a\"; } x + 1;");
        assert!(!program.is_wasm_supported());
        assert!(program.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("unsupported in porffor wasm-aot first slice")));
    }
}
