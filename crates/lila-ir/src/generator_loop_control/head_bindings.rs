//! The actual declaration bindings owned by a classic For initialization head.
use crate::{ArrayDestructuringEvaluationIr, BindingMode, ExprIr, StatementIr};

pub(crate) fn visit_lexical_head_statement_bindings(
    statements: &[StatementIr],
    visit: &mut impl FnMut(BindingMode, &str),
) {
    for statement in statements {
        visit_head_statement_binding(statement, visit);
    }
}

fn visit_head_statement_binding(
    statement: &StatementIr,
    visit: &mut impl FnMut(BindingMode, &str),
) {
    match statement {
        StatementIr::EmptyStatementCompletion(item) => {
            visit_head_statement_binding(item.statement(), visit);
        }
        StatementIr::Lexical { mode, name, .. } => visit(*mode, name),
        StatementIr::Var(declarations) => {
            for declaration in declarations {
                visit(BindingMode::Var, &declaration.name);
            }
        }
        StatementIr::LexicalBlock(statements) => {
            for statement in statements {
                visit_head_statement_binding(statement, visit);
            }
        }
        StatementIr::OrdinaryGeneratorArrayDestructuring(plan) => {
            for statement in &plan.body().block().statements {
                visit_head_statement_binding(statement, visit);
            }
        }
        StatementIr::AsyncFunctionArrayDestructuring(plan) => {
            visit_lexical_head_statement_bindings(&plan.body().statements, visit);
        }
        StatementIr::AsyncGeneratorResourceScope(plan) => {
            visit_lexical_head_statement_bindings(&plan.body().block().statements, visit);
        }
        StatementIr::AsyncGeneratorResourceRegistration(operation) => {
            visit(BindingMode::Const, operation.binding_name())
        }
        StatementIr::AsyncGeneratorArrayDestructuring(plan) => {
            for statement in &plan.body().block().statements {
                visit_head_statement_binding(statement, visit);
            }
        }
        StatementIr::Block(block) => {
            if block.lexical_environment.is_none() {
                for statement in &block.statements {
                    visit_head_statement_binding(statement, visit);
                }
            }
        }
        StatementIr::DeclarationEvaluation(value) => match &value.expr {
            ExprIr::ArrayDestructure {
                pattern,
                evaluation,
                ..
            } => match evaluation {
                ArrayDestructuringEvaluationIr::BindingInitialization => {
                    pattern.visit_bindings(visit)
                }
                ArrayDestructuringEvaluationIr::AssignmentEvaluation => {}
            },
            ExprIr::ObjectDestructure { pattern, .. } => pattern.visit_bindings(visit),
            ExprIr::ObjectDestructuringOperation(operation) => operation.visit_bindings(visit),
            _ => {}
        },
        // Switch declarations belong to its own CaseBlock record, including
        // when a complete Switch occurs inside another continuation region.
        StatementIr::Switch { .. }
        | StatementIr::OrdinaryGeneratorSwitch(_)
        | StatementIr::OrdinaryGeneratorWith(_) => {}
        StatementIr::Empty
        | StatementIr::ModuleImportBinding(_)
        | StatementIr::ResumableClassDefinition(_)
        | StatementIr::ModuleUnitOnce { .. }
        | StatementIr::AnnexBFunctionCopy { .. }
        | StatementIr::SyncDisposableScope { .. }
        | StatementIr::AsyncDisposableScope { .. }
        | StatementIr::ParameterInitialization { .. }
        | StatementIr::Expression(_)
        | StatementIr::ArrayDestructuringOperation(_)
        | StatementIr::AsyncFunctionWith(_)
        | StatementIr::GeneratorYield { .. }
        | StatementIr::AsyncModuleInstantiation
        | StatementIr::AsyncAwait { .. }
        | StatementIr::AsyncGeneratorLoop(_)
        | StatementIr::AsyncGeneratorIf(_)
        | StatementIr::AsyncGeneratorWith(_)
        | StatementIr::AsyncGeneratorSwitch(_)
        | StatementIr::AsyncGeneratorForOf(_)
        | StatementIr::AsyncGeneratorForIn(_)
        | StatementIr::OrdinaryGeneratorLoop(_)
        | StatementIr::OrdinaryGeneratorIf(_)
        | StatementIr::GeneratorLoop { .. }
        | StatementIr::GeneratorIf { .. }
        | StatementIr::If { .. }
        | StatementIr::AsyncFunctionIf { .. }
        | StatementIr::AsyncFunctionWhile(_)
        | StatementIr::AsyncFunctionSwitch(_)
        | StatementIr::While { .. }
        | StatementIr::DoWhile { .. }
        | StatementIr::For { .. }
        | StatementIr::ForOfIterator { .. }
        | StatementIr::AsyncFunctionForOfIterator { .. }
        | StatementIr::GeneratorForOfIterator { .. }
        | StatementIr::ForInArray { .. }
        | StatementIr::ForInString { .. }
        | StatementIr::ForInObject { .. }
        | StatementIr::Labelled { .. }
        | StatementIr::Debugger
        | StatementIr::Throw(_)
        | StatementIr::TryCatch { .. }
        | StatementIr::TryFinally { .. }
        | StatementIr::TryCatchFinally { .. }
        | StatementIr::Return(_)
        | StatementIr::Break { .. }
        | StatementIr::Continue { .. } => {}
    }
}
