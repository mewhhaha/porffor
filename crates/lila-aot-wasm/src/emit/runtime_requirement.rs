use crate::planning::expr_has_static_number_payload;
use lila_ir::{ArithmeticBinaryOp, BlockIr, ExprIr, ScriptIr, StatementIr, TypedExpr, ValueKind};

/// Elide Realm bootstrap only when the lowered script cannot observe it.
/// This is a conservative IR proof: every unrecognized operation retains the
/// ordinary runtime, including declarations, coercions, calls and abrupt exits.
/// Admitted expressions still go through the same Wasm expression emitter.
pub(super) fn requires_runtime(script: &ScriptIr) -> bool {
    script.eval_environment.is_some()
        || script.module_prelude.is_some()
        || !script.prepared_scripts.is_empty()
        || script.runtime_declarations != Default::default()
        || !script.prepared_dynamic_functions.is_empty()
        || !script.functions.is_empty()
        || !script.owned_env_bindings.is_empty()
        || !script.global_bindings.lexical_bindings().is_empty()
        || !script.host_builtins.is_empty()
        || script.top_level_this_uses != 0
        || !block_is_runtime_free(&script.body)
}

fn block_is_runtime_free(block: &BlockIr) -> bool {
    block.lexical_environment.is_none() && block.statements.iter().all(statement_is_runtime_free)
}

fn statement_is_runtime_free(statement: &StatementIr) -> bool {
    match statement {
        StatementIr::Empty => true,
        StatementIr::Expression(expression) => expression_is_runtime_free(expression),
        StatementIr::Block(block) => block_is_runtime_free(block),
        StatementIr::LexicalBlock(statements) => statements.iter().all(statement_is_runtime_free),
        _ => false,
    }
}

fn expression_is_runtime_free(expression: &TypedExpr) -> bool {
    match &expression.expr {
        ExprIr::Undefined
        | ExprIr::Null
        | ExprIr::Boolean(_)
        | ExprIr::Number(_)
        | ExprIr::String(_) => true,
        ExprIr::Void { expr } | ExprIr::DeleteValue { expr } | ExprIr::LogicalNot { expr } => {
            expression_is_runtime_free(expr)
        }
        ExprIr::UnaryPlus { expr }
        | ExprIr::UnaryMinusNumeric { expr }
        | ExprIr::UnaryBitwiseNumeric { expr, .. } => number_is_runtime_free(expr),
        ExprIr::BinaryNumber { op, lhs, rhs } => {
            // Exponentiation calls a separately planned host function. All
            // remaining Number operations emit scalar Wasm instructions.
            let scalar_operation = match op {
                ArithmeticBinaryOp::Add
                | ArithmeticBinaryOp::Sub
                | ArithmeticBinaryOp::Mul
                | ArithmeticBinaryOp::Div
                | ArithmeticBinaryOp::Mod => true,
                ArithmeticBinaryOp::Exp => false,
            };
            scalar_operation
                && expression.kind == ValueKind::Number
                && number_is_runtime_free(lhs)
                && number_is_runtime_free(rhs)
        }
        ExprIr::CompareNumber { lhs, rhs, .. } => {
            number_is_runtime_free(lhs) && number_is_runtime_free(rhs)
        }
        ExprIr::Comma { lhs, rhs } | ExprIr::LogicalShortCircuit { lhs, rhs, .. } => {
            expression_is_runtime_free(lhs) && expression_is_runtime_free(rhs)
        }
        ExprIr::Conditional {
            condition,
            then_expr,
            else_expr,
        } => {
            expression_is_runtime_free(condition)
                && expression_is_runtime_free(then_expr)
                && expression_is_runtime_free(else_expr)
        }
        _ => false,
    }
}

fn number_is_runtime_free(expression: &TypedExpr) -> bool {
    expr_has_static_number_payload(expression) && expression_is_runtime_free(expression)
}
