use std::collections::BTreeMap;

use boa_ast::{
    declaration::{Binding, LexicalDeclaration},
    expression::literal::LiteralKind,
    expression::operator::binary::{ArithmeticOp, BinaryOp},
    expression::operator::unary::UnaryOp,
    expression::Expression,
    scope::Scope,
    statement::Statement,
    Declaration, Script, StatementListItem,
};
use boa_interner::Interner;
use boa_parser::{Parser, Source};
use porffor_front::{ParseGoal, SourceUnit};

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
}

impl ValueKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Undefined => "undefined",
            Self::Null => "null",
            Self::Boolean => "boolean",
            Self::Number => "number",
            Self::String => "string",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingMode {
    Let,
    Const,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedExpr {
    pub kind: ValueKind,
    pub expr: ExprIr,
}

impl TypedExpr {
    pub const fn undefined() -> Self {
        Self {
            kind: ValueKind::Undefined,
            expr: ExprIr::Undefined,
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
    Identifier(String),
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatementIr {
    Lexical {
        mode: BindingMode,
        name: String,
        init: TypedExpr,
    },
    Expression(TypedExpr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptIr {
    pub statements: Vec<StatementIr>,
    pub result_kind: ValueKind,
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
            Some(script) => format!(
                "script statements={} result={}",
                script.statements.len(),
                script.result_kind.as_str()
            ),
            None => "no script ir".to_string(),
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
            let lowered = ScriptLowerer::new(&interner).lower(&script);
            if !lowered.statements.is_empty() || lowered.diagnostics.is_empty() {
                program.script = Some(ScriptIr {
                    statements: lowered.statements,
                    result_kind: lowered.result_kind,
                });
                program.stages.push(LoweringStage::ScriptIrBuilt);
            }
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

struct LoweredScript {
    statements: Vec<StatementIr>,
    diagnostics: Vec<IrDiagnostic>,
    result_kind: ValueKind,
}

struct ScriptLowerer<'a> {
    interner: &'a Interner,
    bindings: BTreeMap<String, ValueKind>,
    statements: Vec<StatementIr>,
    diagnostics: Vec<IrDiagnostic>,
    result_kind: ValueKind,
}

impl<'a> ScriptLowerer<'a> {
    fn new(interner: &'a Interner) -> Self {
        Self {
            interner,
            bindings: BTreeMap::new(),
            statements: Vec::new(),
            diagnostics: Vec::new(),
            result_kind: ValueKind::Undefined,
        }
    }

    fn lower(mut self, script: &Script) -> LoweredScript {
        for item in script.statements().statements() {
            self.lower_statement_list_item(item);
        }
        LoweredScript {
            statements: self.statements,
            diagnostics: self.diagnostics,
            result_kind: self.result_kind,
        }
    }

    fn lower_statement_list_item(&mut self, item: &StatementListItem) {
        match item {
            StatementListItem::Statement(statement) => self.lower_statement(statement),
            StatementListItem::Declaration(declaration) => self.lower_declaration(declaration),
        }
    }

    fn lower_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Expression(expression) => {
                let lowered = self.lower_expression(expression);
                self.result_kind = lowered.kind;
                self.statements.push(StatementIr::Expression(lowered));
            }
            Statement::Empty => {
                self.result_kind = ValueKind::Undefined;
            }
            Statement::Var(_) => {
                self.unsupported("var declaration");
            }
            Statement::Block(_)
            | Statement::Debugger
            | Statement::If(_)
            | Statement::DoWhileLoop(_)
            | Statement::WhileLoop(_)
            | Statement::ForLoop(_)
            | Statement::ForInLoop(_)
            | Statement::ForOfLoop(_)
            | Statement::Switch(_)
            | Statement::Continue(_)
            | Statement::Break(_)
            | Statement::Return(_)
            | Statement::Labelled(_)
            | Statement::Throw(_)
            | Statement::Try(_)
            | Statement::With(_) => {
                self.unsupported("control-flow or non-expression statement");
            }
        }
    }

    fn lower_declaration(&mut self, declaration: &Declaration) {
        match declaration {
            Declaration::Lexical(lexical) => self.lower_lexical_declaration(lexical),
            Declaration::FunctionDeclaration(_)
            | Declaration::GeneratorDeclaration(_)
            | Declaration::AsyncFunctionDeclaration(_)
            | Declaration::AsyncGeneratorDeclaration(_)
            | Declaration::ClassDeclaration(_) => {
                self.unsupported("function or class declaration");
            }
        }
    }

    fn lower_lexical_declaration(&mut self, declaration: &LexicalDeclaration) {
        let (mode, list) = match declaration {
            LexicalDeclaration::Let(list) => (BindingMode::Let, list),
            LexicalDeclaration::Const(list) => (BindingMode::Const, list),
            LexicalDeclaration::Using(_) | LexicalDeclaration::AwaitUsing(_) => {
                self.unsupported("using declaration");
                return;
            }
        };

        for variable in list.as_ref() {
            let Binding::Identifier(identifier) = variable.binding() else {
                self.unsupported("destructuring binding");
                continue;
            };

            let name = self.interner.resolve_expect(identifier.sym()).to_string();
            let init = variable
                .init()
                .map(|expression| self.lower_expression(expression))
                .unwrap_or_else(TypedExpr::undefined);

            self.bindings.insert(name.clone(), init.kind);
            self.statements
                .push(StatementIr::Lexical { mode, name, init });
            self.result_kind = ValueKind::Undefined;
        }
    }

    fn lower_expression(&mut self, expression: &Expression) -> TypedExpr {
        match expression {
            Expression::Identifier(identifier) => {
                let name = self.interner.resolve_expect(identifier.sym()).to_string();
                let kind = self.bindings.get(&name).copied().unwrap_or_else(|| {
                    self.unsupported_with_message(format!(
                        "unsupported in porffor wasm-aot first slice: unbound identifier `{name}`"
                    ));
                    ValueKind::Undefined
                });
                TypedExpr {
                    kind,
                    expr: ExprIr::Identifier(name),
                }
            }
            Expression::Literal(literal) => match literal.kind() {
                LiteralKind::String(sym) => TypedExpr {
                    kind: ValueKind::String,
                    expr: ExprIr::String(self.interner.resolve_expect(*sym).to_string()),
                },
                LiteralKind::Num(value) => TypedExpr {
                    kind: ValueKind::Number,
                    expr: ExprIr::Number(value.to_bits()),
                },
                LiteralKind::Int(value) => TypedExpr {
                    kind: ValueKind::Number,
                    expr: ExprIr::Number((*value as f64).to_bits()),
                },
                LiteralKind::Bool(value) => TypedExpr {
                    kind: ValueKind::Boolean,
                    expr: ExprIr::Boolean(*value),
                },
                LiteralKind::Null => TypedExpr {
                    kind: ValueKind::Null,
                    expr: ExprIr::Null,
                },
                LiteralKind::Undefined => TypedExpr::undefined(),
                LiteralKind::BigInt(_) => self.unsupported_expr("bigint literal"),
            },
            Expression::Parenthesized(expression) => self.lower_expression(expression.expression()),
            Expression::Unary(unary) => self.lower_unary(unary.op(), unary.target()),
            Expression::Binary(binary) => {
                self.lower_binary(binary.op(), binary.lhs(), binary.rhs())
            }
            Expression::This(_)
            | Expression::RegExpLiteral(_)
            | Expression::ArrayLiteral(_)
            | Expression::ObjectLiteral(_)
            | Expression::Spread(_)
            | Expression::FunctionExpression(_)
            | Expression::ArrowFunction(_)
            | Expression::AsyncArrowFunction(_)
            | Expression::GeneratorExpression(_)
            | Expression::AsyncFunctionExpression(_)
            | Expression::AsyncGeneratorExpression(_)
            | Expression::ClassExpression(_)
            | Expression::TemplateLiteral(_)
            | Expression::PropertyAccess(_)
            | Expression::New(_)
            | Expression::Call(_)
            | Expression::SuperCall(_)
            | Expression::ImportCall(_)
            | Expression::Optional(_)
            | Expression::TaggedTemplate(_)
            | Expression::NewTarget(_)
            | Expression::ImportMeta(_)
            | Expression::Assign(_)
            | Expression::Update(_)
            | Expression::BinaryInPrivate(_)
            | Expression::Conditional(_)
            | Expression::Await(_)
            | Expression::Yield(_)
            | Expression::FormalParameterList(_)
            | Expression::Debugger => self.unsupported_expr("unsupported expression form"),
        }
    }

    fn lower_unary(&mut self, op: UnaryOp, target: &Expression) -> TypedExpr {
        let target = self.lower_expression(target);
        match op {
            UnaryOp::Plus => {
                if target.kind != ValueKind::Number {
                    return self.unsupported_expr("coercive unary plus");
                }
                TypedExpr {
                    kind: ValueKind::Number,
                    expr: ExprIr::UnaryNumber {
                        op: UnaryNumericOp::Plus,
                        expr: Box::new(target),
                    },
                }
            }
            UnaryOp::Minus => {
                if target.kind != ValueKind::Number {
                    return self.unsupported_expr("coercive unary minus");
                }
                TypedExpr {
                    kind: ValueKind::Number,
                    expr: ExprIr::UnaryNumber {
                        op: UnaryNumericOp::Minus,
                        expr: Box::new(target),
                    },
                }
            }
            UnaryOp::Not => TypedExpr {
                kind: ValueKind::Boolean,
                expr: ExprIr::LogicalNot {
                    expr: Box::new(target),
                },
            },
            UnaryOp::Tilde | UnaryOp::TypeOf | UnaryOp::Delete | UnaryOp::Void => {
                self.unsupported_expr("unsupported unary operator")
            }
        }
    }

    fn lower_binary(&mut self, op: BinaryOp, lhs: &Expression, rhs: &Expression) -> TypedExpr {
        let lhs = self.lower_expression(lhs);
        let rhs = self.lower_expression(rhs);

        let arithmetic = match op {
            BinaryOp::Arithmetic(arithmetic) => arithmetic,
            _ => return self.unsupported_expr("unsupported binary operator"),
        };

        match arithmetic {
            ArithmeticOp::Add => {
                if lhs.kind != ValueKind::Number || rhs.kind != ValueKind::Number {
                    return self.unsupported_expr("string or coercive `+`");
                }
                TypedExpr {
                    kind: ValueKind::Number,
                    expr: ExprIr::BinaryNumber {
                        op: ArithmeticBinaryOp::Add,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    },
                }
            }
            ArithmeticOp::Sub | ArithmeticOp::Mul | ArithmeticOp::Div | ArithmeticOp::Mod => {
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
                TypedExpr {
                    kind: ValueKind::Number,
                    expr: ExprIr::BinaryNumber {
                        op,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    },
                }
            }
            ArithmeticOp::Exp => self.unsupported_expr("exponentiation operator"),
        }
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
        let script = program.script.expect("script ir should exist");
        assert_eq!(script.statements.len(), 3);
        assert_eq!(script.result_kind, ValueKind::Number);
    }

    #[test]
    fn records_unsupported_features_without_failing_lower() {
        let program = lower_script("var x = 1;");
        assert!(!program.is_wasm_supported());
        assert!(program
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("var declaration")));
    }
}
