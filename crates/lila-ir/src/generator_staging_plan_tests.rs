//! The staged generator plan and the lowering it admits, checked against each
//! other: every admitted shape must lower without a diagnostic and use exactly
//! the resume states the plan allocated.

use super::*;
use lila_front::{parse, ParseOptions};

fn lower_script(source: &str) -> ProgramIr {
    let source = parse(source, ParseOptions::script()).expect("script should parse");
    crate::lower(&source)
}

fn generator<'a>(program: &'a ProgramIr, name: &str) -> &'a FunctionIr {
    program
        .script
        .as_ref()
        .expect("script ir should exist")
        .functions
        .iter()
        .find(|function| function.name == name)
        .unwrap_or_else(|| panic!("generator `{name}` should be registered"))
}

/// The highest resume state the lowered body uses, collected from every
/// suspension and structured resume point.
fn lowered_state_ceiling(statements: &[StatementIr]) -> u32 {
    fn visit(statement: &StatementIr, ceiling: &mut u32) {
        match statement {
            StatementIr::GeneratorYield {
                suspend_state,
                resume_state,
                ..
            } => *ceiling = (*ceiling).max(*suspend_state).max(*resume_state),
            StatementIr::GeneratorIf {
                then_before_yield,
                then_yield_statement,
                then_after_yield,
                else_before_yield,
                else_yield_statement,
                else_after_yield,
                exit_state,
                ..
            } => {
                *ceiling = (*ceiling).max(*exit_state);
                for statement in then_before_yield
                    .iter()
                    .chain(then_yield_statement.as_deref())
                    .chain(then_after_yield)
                    .chain(else_before_yield)
                    .chain(else_yield_statement.as_deref())
                    .chain(else_after_yield)
                {
                    visit(statement, ceiling);
                }
            }
            StatementIr::TryCatchFinally {
                try_block,
                catch_block,
                finally_block,
                generator_plan,
                ..
            } => {
                if let Some(plan) = generator_plan {
                    *ceiling = (*ceiling).max(plan.exit_state);
                }
                for statement in try_block
                    .statements
                    .iter()
                    .chain(&catch_block.statements)
                    .chain(&finally_block.statements)
                {
                    visit(statement, ceiling);
                }
            }
            StatementIr::LexicalBlock(statements) => {
                for statement in statements {
                    visit(statement, ceiling);
                }
            }
            StatementIr::Block(block) => {
                for statement in &block.statements {
                    visit(statement, ceiling);
                }
            }
            _ => {}
        }
    }
    let mut ceiling = 0;
    for statement in statements {
        visit(statement, &mut ceiling);
    }
    ceiling
}

fn assert_staged_generator(source: &str, name: &str) -> FunctionIr {
    let program = lower_script(source);
    assert!(
        program.is_wasm_supported(),
        "{source}: {:?}",
        program.diagnostics
    );
    let function = generator(&program, name).clone();
    let plan = function
        .generator_plan
        .clone()
        .unwrap_or_else(|| panic!("`{name}` should have a generator plan"));
    assert_eq!(
        plan.state_count,
        lowered_state_ceiling(&function.body.statements) + 1,
        "{source}: the plan and the lowering must allocate the same resume states"
    );
    function
}

fn resumable_steps(statements: &[StatementIr]) -> Vec<&ResumableArrayDestructuringStepIr> {
    fn visit_expr<'a>(expr: &'a TypedExpr, steps: &mut Vec<&'a ResumableArrayDestructuringStepIr>) {
        if let ExprIr::ResumableArrayDestructuring(destructuring) = &expr.expr {
            steps.push(destructuring.step());
        }
    }
    fn visit<'a>(
        statement: &'a StatementIr,
        steps: &mut Vec<&'a ResumableArrayDestructuringStepIr>,
    ) {
        match statement {
            StatementIr::Expression(expr) | StatementIr::Lexical { init: expr, .. } => {
                visit_expr(expr, steps)
            }
            StatementIr::LexicalBlock(statements) => statements
                .iter()
                .for_each(|statement| visit(statement, steps)),
            StatementIr::TryCatchFinally {
                try_block,
                catch_block,
                finally_block,
                ..
            } => try_block
                .statements
                .iter()
                .chain(&catch_block.statements)
                .chain(&finally_block.statements)
                .for_each(|statement| visit(statement, steps)),
            StatementIr::GeneratorIf {
                then_before_yield,
                then_after_yield,
                ..
            } => then_before_yield
                .iter()
                .chain(then_after_yield)
                .for_each(|statement| visit(statement, steps)),
            _ => {}
        }
    }
    let mut steps = Vec::new();
    for statement in statements {
        visit(statement, &mut steps);
    }
    steps
}

#[test]
fn staged_generator_array_destructuring_closes_through_a_synthesized_try() {
    let function = assert_staged_generator(
        "var x = {};
         function* g(vals) { var result; result = [ x[yield] ] = vals; }",
        "g",
    );
    let steps = resumable_steps(&function.body.statements);
    assert!(
        matches!(
            steps.as_slice(),
            [
                ResumableArrayDestructuringStepIr::Open { .. },
                ResumableArrayDestructuringStepIr::Elements(elements),
                ResumableArrayDestructuringStepIr::Close(ResumableIteratorCloseIr::Throw),
                ResumableArrayDestructuringStepIr::Close(ResumableIteratorCloseIr::NormalOrReturn),
            ] if elements.len() == 1
        ),
        "{steps:?}"
    );
    let plan = function.generator_plan.expect("generator plan");
    // One element yield, then one reserved state after each of the try,
    // catch and finally blocks.
    assert_eq!(plan.state_count, 5);
    assert_eq!(
        plan.suspension_points,
        vec![GeneratorSuspensionPointIr {
            suspend_state: 0,
            resume_state: 1,
        }]
    );
}

#[test]
fn staged_generator_destructuring_initializer_is_a_one_branch_resume_point() {
    let function = assert_staged_generator(
        "var x;
         function* g(vals) { [ x = yield ] = vals; }",
        "g",
    );
    fn contains_one_branch_if(statements: &[StatementIr]) -> bool {
        statements.iter().any(|statement| match statement {
            StatementIr::GeneratorIf {
                then_yield_statement: Some(_),
                else_yield_statement: None,
                then_resume_state: Some(then_resume_state),
                entry_state,
                exit_state,
                ..
            } => *then_resume_state == entry_state + 1 && *exit_state == entry_state + 2,
            StatementIr::LexicalBlock(statements) => contains_one_branch_if(statements),
            StatementIr::TryCatchFinally { try_block, .. } => {
                contains_one_branch_if(&try_block.statements)
            }
            _ => false,
        })
    }
    assert!(
        contains_one_branch_if(&function.body.statements),
        "{:?}",
        function.body.statements
    );
}

#[test]
fn staged_generator_nested_and_rest_patterns_share_one_plan() {
    for (source, name) in [
        (
            "var x = {}; function* g(vals) { [[x[yield]]] = vals; }",
            "g",
        ),
        (
            "var x; function* g(vals) { [...{ x = yield }] = vals; }",
            "g",
        ),
        (
            "var x = {}; function* g(vals) { [ {} , ...x[yield] ] = vals; }",
            "g",
        ),
        (
            "var x; function* g(vals) { return { a: [x = yield] } = vals; }",
            "g",
        ),
        (
            "var x = {}; function* g(vals) { ({ [yield]: x[yield], b = yield } = vals); }",
            "g",
        ),
    ] {
        assert_staged_generator(source, name);
    }
}

#[test]
fn staged_generator_object_pattern_reads_properties_without_an_iterator() {
    let function = assert_staged_generator(
        "var y = {}; function* g(vals) { ({ x: y[yield] } = vals); }",
        "g",
    );
    assert!(resumable_steps(&function.body.statements).is_empty());
}

#[test]
fn staged_generator_operands_cover_the_formerly_rejected_shapes() {
    for (source, name) in [
        ("function* g() { return 1 + (yield 2); }", "g"),
        ("function* g() { let o = { [yield 9]: 9 }; return o; }", "g"),
        (
            "function* g() { let o = { get [yield]() { return 1; }, set [yield](v) {} }; }",
            "g",
        ),
        ("var x = 1; function* g() { x += yield; }", "g"),
        ("function* g() { return typeof (yield) + -(yield); }", "g"),
        ("function* g(a) { return a ? 1 : 2, (yield) ? 3 : 4; }", "g"),
        ("function* g() { let s = `a${yield 1}b${yield 2}c`; }", "g"),
        ("function* g(C) { return new C(yield, yield); }", "g"),
        ("function* g() { yield [yield yield 1]; }", "g"),
    ] {
        assert_staged_generator(source, name);
    }
}

#[test]
fn staged_generator_class_method_admits_a_private_in_operand() {
    let source = "class C { #f; static *g() { return #f in (yield); } }";
    let program = lower_script(source);
    assert!(
        program.is_wasm_supported(),
        "{source}: {:?}",
        program.diagnostics
    );
}

#[test]
fn staged_generator_rejections_name_the_unstaged_form() {
    for (source, message) in [
        (
            "function* g(a) { return a && (yield); }",
            StagedYieldRejection::ConditionalOperand.message(),
        ),
        (
            "var x; function* g(vals) { [x = yield (yield)] = vals; }",
            StagedYieldRejection::DestructuringInitializer.message(),
        ),
        (
            "var a, r; function* g(vals) { ({ a = yield, ...r } = vals); }",
            StagedYieldRejection::DestructuringObjectRest.message(),
        ),
        (
            "function* g(a) { return { ...a, [yield]: 1 }; }",
            StagedYieldRejection::ObjectLiteralOperand.message(),
        ),
        (
            "var o = {}; function* g() { o.x += yield; }",
            StagedYieldRejection::UnstagedReference.message(),
        ),
        (
            "function* g(f, a) { f(...a, yield); }",
            StagedYieldRejection::UnstagedArgumentList.message(),
        ),
    ] {
        let program = lower_script(source);
        assert!(!program.is_wasm_supported(), "{source} should be refused");
        assert!(
            program
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains(message)),
            "{source}: {:?}",
            program.diagnostics
        );
    }
}

#[test]
fn generator_expression_refusals_report_the_plan_reason() {
    let program = lower_script("var g = function* () { return a && (yield); };");
    assert!(!program.is_wasm_supported());
    assert!(
        program.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains(StagedYieldRejection::ConditionalOperand.message())),
        "{:?}",
        program.diagnostics
    );
    assert!(!program
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.ends_with("generator suspension")));
}

#[test]
fn async_generator_suspending_patterns_are_refused_by_name() {
    let program = lower_script("var x; async function* g(v) { [x = yield] = v; }");
    assert!(!program.is_wasm_supported());
    assert!(
        program.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("async generator destructuring assignment whose pattern suspends")),
        "{:?}",
        program.diagnostics
    );
}
