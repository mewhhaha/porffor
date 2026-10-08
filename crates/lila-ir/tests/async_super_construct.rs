use std::collections::BTreeSet;

use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, ExprIr, FunctionIr, FunctionProtocolIr, PreparedSuperConstructIr, StatementIr, TypedExpr,
};

fn arrow(source: &str) -> FunctionIr {
    let parsed = parse(source, ParseOptions::script()).expect("derived async arrow parses");
    let lowered = lower(&parsed);
    assert!(lowered.is_wasm_supported(), "{:?}", lowered.diagnostics);
    lowered
        .script
        .unwrap()
        .functions
        .into_iter()
        .find(|function| function.protocol == FunctionProtocolIr::AsyncArrow)
        .unwrap()
}

fn flatten<'a>(statements: &'a [StatementIr], output: &mut Vec<&'a StatementIr>) {
    for statement in statements {
        match statement {
            StatementIr::LexicalBlock(items) => flatten(items, output),
            StatementIr::Block(block) => flatten(&block.statements, output),
            StatementIr::EmptyStatementCompletion(item) => {
                flatten(std::slice::from_ref(item.statement()), output)
            }
            _ => output.push(statement),
        }
    }
}

fn expression(statement: &StatementIr) -> Option<&TypedExpr> {
    match statement {
        StatementIr::Lexical { init, .. }
        | StatementIr::Expression(init)
        | StatementIr::DeclarationEvaluation(init)
        | StatementIr::Return(init) => Some(init),
        _ => None,
    }
}

fn prepared<'a>(statements: &[&'a StatementIr]) -> &'a PreparedSuperConstructIr {
    statements
        .iter()
        .find_map(
            |statement| match expression(statement).map(|value| &value.expr) {
                Some(ExprIr::PreparedSuperConstruct(prepared)) => Some(prepared.as_ref()),
                _ => None,
            },
        )
        .expect("the terminal operation consumes actual captured preparation")
}

fn identifier(value: &TypedExpr) -> &str {
    let ExprIr::Identifier(name) = &value.expr else {
        panic!("retained implicit operand: {value:?}");
    };
    name
}

#[test]
fn awaited_super_captures_original_constructor_and_new_target_before_argument_evaluation() {
    let function = arrow("class Base {} class Derived extends Base { constructor(gate, first, last) { const invoke = async () => super(first(), await gate, last()); invoke(); return {}; } }");
    let owner = function.lexical_derived_activation.as_ref().unwrap();
    assert_ne!(owner.owner_function_id, function.id);
    for name in [
        &owner.this_binding,
        &owner.this_status_binding,
        &owner.new_target_binding,
        &owner.active_function_binding,
    ] {
        assert!(
            function
                .captured_bindings
                .iter()
                .any(|binding| &binding.name == name),
            "{name}"
        );
    }
    let mut statements = Vec::new();
    flatten(&function.body.statements, &mut statements);
    let operation = prepared(&statements);
    let target_position = statements
        .iter()
        .position(|statement| {
            matches!(
                expression(statement).map(|value| &value.expr),
                Some(ExprIr::SuperNewTarget)
            )
        })
        .unwrap();
    let constructor_position = statements
        .iter()
        .position(|statement| {
            matches!(
                expression(statement).map(|value| &value.expr),
                Some(ExprIr::SuperConstructor)
            )
        })
        .unwrap();
    let suspension = statements
        .iter()
        .position(|statement| matches!(statement, StatementIr::AsyncAwait { .. }))
        .unwrap();
    assert!(target_position < constructor_position && constructor_position < suspension);
    let first_call = statements
        .iter()
        .position(|statement| {
            matches!(
                expression(statement).map(|value| &value.expr),
                Some(ExprIr::CallIndirect { .. } | ExprIr::CallNamed { .. })
            )
        })
        .unwrap();
    assert!(constructor_position < first_call && first_call < suspension);
    let names = [
        identifier(operation.new_target()),
        identifier(operation.constructor()),
    ];
    let slots = names.map(|name| {
        let rows = function
            .owned_env_bindings
            .iter()
            .filter(|binding| binding.name == name)
            .collect::<Vec<_>>();
        assert_eq!(rows.len(), 1);
        rows[0].slot
    });
    assert_eq!(slots.into_iter().collect::<BTreeSet<_>>().len(), 2);
    assert_eq!(operation.arguments().len(), 3);
    assert!(operation
        .arguments()
        .iter()
        .all(|argument| matches!(argument.expr, ExprIr::Identifier(_))));
    assert!(!statements.iter().any(|statement| matches!(
        expression(statement).map(|value| &value.expr),
        Some(ExprIr::SuperConstruct { .. })
    )));
}

#[test]
fn super_spread_is_captured_before_later_await_in_the_original_argument_list_owner() {
    let function = arrow("class Base {} class Derived extends Base { constructor(values, gate) { const invoke = async () => super(...values, await gate); invoke(); return {}; } }");
    let mut statements = Vec::new();
    flatten(&function.body.statements, &mut statements);
    let operation = prepared(&statements);
    let capture = statements
        .iter()
        .position(|statement| {
            matches!(
                expression(statement).map(|value| &value.expr),
                Some(ExprIr::CaptureArgumentList(_))
            )
        })
        .unwrap();
    let suspension = statements
        .iter()
        .position(|statement| matches!(statement, StatementIr::AsyncAwait { .. }))
        .unwrap();
    assert!(capture < suspension);
    assert!(matches!(
        operation.arguments()[0].expr,
        ExprIr::CapturedArgumentList(_)
    ));
    assert!(matches!(
        operation.arguments()[1].expr,
        ExprIr::Identifier(_)
    ));
}

#[test]
fn terminal_super_result_remains_dynamic_before_lexical_this_initialization() {
    let function = arrow("class Base {} class Derived extends Base { constructor(gate) { const invoke = async () => { const result = super(await gate); return result; }; invoke(); return {}; } }");
    let mut statements = Vec::new();
    flatten(&function.body.statements, &mut statements);
    let value = statements
        .iter()
        .find_map(|statement| {
            expression(statement)
                .filter(|value| matches!(value.expr, ExprIr::PreparedSuperConstruct(_)))
        })
        .unwrap();
    assert_eq!(value.kind, lila_ir::ValueKind::Dynamic);
    assert_eq!(value.possible_kinds, lila_ir::KindSet::all_runtime_tags());
}

#[test]
fn ordinary_async_functions_and_methods_do_not_gain_a_derived_constructor_owner() {
    for source in [
        "async function call(gate) { super(await gate); }",
        "class Base {} class Derived extends Base { async call(gate) { super(await gate); } }",
    ] {
        assert!(parse(source, ParseOptions::script()).is_err(), "{source}");
    }
}
