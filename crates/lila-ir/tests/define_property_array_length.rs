use lila_front::{parse, ParseOptions};
use lila_ir::{lower, ArithmeticBinaryOp, ExprIr, ProgramIr, StatementIr, TypedExpr};

fn lower_script(source: &str) -> ProgramIr {
    let parsed = parse(source, ParseOptions::script()).expect("valid source");
    let program = lower(&parsed);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    program
}

fn final_expression(program: &ProgramIr) -> &TypedExpr {
    let Some(StatementIr::Expression(expression)) = program
        .script
        .as_ref()
        .expect("Script IR")
        .body
        .statements
        .last()
    else {
        panic!("source must end in its observed addition");
    };
    expression
}

#[test]
fn array_length_definitions_invalidate_facts_after_descriptor_value_coercion() {
    for builtin in ["Object.defineProperty", "Reflect.defineProperty"] {
        let source = format!(
            "const holder = {{ value: 1 }}; \
             {builtin}([], 'length', {{ __proto__: null, value: {{ \
                 valueOf() {{ holder.value = 'changed'; return 0; }} \
             }} }}); holder.value + 1;"
        );
        let program = lower_script(&source);
        assert!(
            matches!(final_expression(&program).expr, ExprIr::CoerciveAdd { .. }),
            "{builtin} must account for ArraySetLength after an effect-free descriptor: {:?}",
            final_expression(&program)
        );
    }
}

#[test]
fn an_ordinary_function_with_a_null_prototype_descriptor_keeps_unrelated_facts() {
    for builtin in ["Object.defineProperty", "Reflect.defineProperty"] {
        let source = format!(
            "const holder = {{ value: 1 }}; function target() {{}} \
             {builtin}(target, 'x', {{ __proto__: null, value: 0 }}); holder.value + 1;"
        );
        let program = lower_script(&source);
        assert!(
            matches!(
                final_expression(&program).expr,
                ExprIr::BinaryNumber {
                    op: ArithmeticBinaryOp::Add,
                    ..
                } | ExprIr::CoerciveBinaryNumber {
                    op: ArithmeticBinaryOp::Add,
                    ..
                }
            ),
            "ordinary DefineOwnProperty must keep its independent proof: {:?}",
            final_expression(&program)
        );
    }
}
