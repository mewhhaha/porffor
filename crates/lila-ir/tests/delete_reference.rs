use lila_front::{parse, ParseOptions};
use lila_ir::{lower, ExprIr, FunctionIr, PropertyKeyIr, StatementIr, Strictness, TypedExpr};

fn lower_remove(source: &str) -> FunctionIr {
    let unit = parse(source, ParseOptions::script()).expect("delete fixture parses");
    let program = lower(&unit);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    program
        .script
        .expect("script IR")
        .functions
        .into_iter()
        .find(|function| function.name == "remove")
        .expect("remove function")
}

fn returned(statements: &[StatementIr]) -> Option<&TypedExpr> {
    for statement in statements {
        let found = match statement {
            StatementIr::Return(value) => Some(value),
            StatementIr::LexicalBlock(statements) => returned(statements),
            StatementIr::Block(block) => returned(&block.statements),
            _ => None,
        };
        if found.is_some() {
            return found;
        }
    }
    None
}

#[test]
fn primitive_bases_remain_property_references_through_parentheses() {
    for base in ["null", "undefined", "'abcdef'", "1", "true", "1n"] {
        let function = lower_remove(&format!(
            "function remove() {{ 'use strict'; return delete ((({base}[0]))); }}"
        ));
        let value = returned(&function.body.statements).expect("return");
        let ExprIr::DeleteProperty {
            key, strictness, ..
        } = &value.expr
        else {
            panic!("property Reference must survive: {value:?}");
        };
        assert_eq!(*strictness, Strictness::Strict);
        assert!(matches!(
            key,
            PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayIndex(_)
        ));
    }
}

#[test]
fn with_delete_selects_property_deletion_and_keeps_a_live_global_fallback() {
    let function =
        lower_remove("function remove(scope) { with(scope) { return delete (((x))); } }");
    let value = returned(&function.body.statements).expect("return");
    let ExprIr::Conditional {
        then_expr,
        else_expr,
        ..
    } = &value.expr
    else {
        panic!("with HasBinding must select the reference: {value:?}");
    };
    assert!(matches!(&then_expr.expr, ExprIr::DeleteProperty {
        key: PropertyKeyIr::StaticString(name), strictness: Strictness::Sloppy, ..
    } if name == "x"));
    assert!(
        matches!(&else_expr.expr, ExprIr::DeleteGlobalProperty { name, .. } if name == "x"),
        "HasBinding can create the global before fallback: {else_expr:?}"
    );
}

#[test]
fn an_inner_lexical_binding_stops_with_resolution_for_delete() {
    let function =
        lower_remove("function remove(scope) { with(scope) { let x = 1; return delete (x); } }");
    let value = returned(&function.body.statements).expect("return");
    assert!(
        matches!(&value.expr, ExprIr::DeleteIdentifier { .. }),
        "lexical binding remains nondeletable: {value:?}"
    );
}
