use std::collections::BTreeSet;

use lila_front::{parse, ParseOptions};

#[path = "common/statement_awaits.rs"]
mod statement_awaits;
use lila_ir::{
    lower, ExprIr, FunctionIr, KindSet, OptionalCallReferenceCaptureIr, OptionalChainOperationIr,
    PropertyKeyIr, SpecOperationIr, StatementIr, TypedExpr,
};

fn choose(source: &str) -> FunctionIr {
    let parsed = parse(source, ParseOptions::script()).expect("optional Call source parses");
    let program = lower(&parsed);
    assert!(
        program.is_wasm_supported(),
        "{source}: {:?}",
        program.diagnostics
    );
    program
        .script
        .expect("script")
        .functions
        .into_iter()
        .find(|function| matches!(function.name.as_str(), "choose" | "C.choose"))
        .expect("plain async function")
}

fn flatten<'a>(source: &'a [StatementIr], result: &mut Vec<&'a StatementIr>) {
    for statement in source {
        match statement {
            StatementIr::EmptyStatementCompletion(item) => {
                flatten(std::slice::from_ref(item.statement()), result)
            }
            StatementIr::LexicalBlock(source) => flatten(source, result),
            StatementIr::Block(block) => flatten(&block.statements, result),
            statement => result.push(statement),
        }
    }
}

fn arm(statement: &StatementIr) -> Vec<&StatementIr> {
    let StatementIr::LexicalBlock(source) = statement else {
        panic!("private arm scope")
    };
    let mut result = Vec::new();
    flatten(source, &mut result);
    result
}

fn all<'a>(source: &'a [StatementIr], result: &mut Vec<&'a StatementIr>) {
    for statement in source {
        match statement {
            StatementIr::EmptyStatementCompletion(item) => {
                all(std::slice::from_ref(item.statement()), result)
            }
            StatementIr::LexicalBlock(source) => all(source, result),
            StatementIr::Block(block) => all(&block.statements, result),
            StatementIr::AsyncFunctionIf {
                then_branch,
                else_branch,
                ..
            }
            | StatementIr::If {
                then_branch,
                else_branch,
                ..
            } => {
                result.push(statement);
                all(std::slice::from_ref(then_branch.as_ref()), result);
                if let Some(branch) = else_branch {
                    all(std::slice::from_ref(branch.as_ref()), result);
                }
            }
            StatementIr::While { body, .. }
            | StatementIr::DoWhile { body, .. }
            | StatementIr::For { body, .. } => {
                result.push(statement);
                all(std::slice::from_ref(body.as_ref()), result);
            }
            StatementIr::AsyncFunctionWhile(plan) => {
                result.push(statement);
                all(plan.condition_prefix(), result);
                all(std::slice::from_ref(plan.body()), result);
            }
            statement => result.push(statement),
        }
    }
}

fn value(statement: &StatementIr) -> Option<&TypedExpr> {
    match statement {
        StatementIr::Lexical { init, .. } => Some(init),
        StatementIr::Expression(value)
        | StatementIr::DeclarationEvaluation(value)
        | StatementIr::Return(value) => Some(value),
        _ => None,
    }
}

fn values<'a>(value: &'a TypedExpr, result: &mut Vec<&'a TypedExpr>) {
    result.push(value);
    match &value.expr {
        ExprIr::AssignIdentifier { value, .. } => values(value, result),
        ExprIr::Comma { lhs, rhs } => {
            values(lhs, result);
            values(rhs, result);
        }
        ExprIr::MaterializeBinding { value, body, .. } => {
            values(value, result);
            values(body, result);
        }
        _ => {}
    }
}

fn expressions<'a>(statements: &[&'a StatementIr]) -> Vec<&'a TypedExpr> {
    let mut result = Vec::new();
    for statement in statements {
        if let Some(value) = value(statement) {
            values(value, &mut result);
        }
    }
    result
}

fn captures<'a>(statements: &[&'a StatementIr]) -> Vec<&'a OptionalCallReferenceCaptureIr> {
    expressions(statements)
        .into_iter()
        .filter_map(|value| match &value.expr {
            ExprIr::CaptureOptionalCallReference(capture) => Some(capture),
            _ => None,
        })
        .collect()
}

fn calls<'a>(statements: &[&'a StatementIr]) -> Vec<&'a TypedExpr> {
    expressions(statements)
        .into_iter()
        .filter(|value| matches!(value.expr, ExprIr::CallIndirect { .. }))
        .collect()
}

fn call(value: &TypedExpr) -> (&str, Option<&str>, &[TypedExpr]) {
    let ExprIr::CallIndirect {
        callee,
        this_arg,
        args,
        direct_eval,
        ..
    } = &value.expr
    else {
        panic!("selected actual Call")
    };
    assert!(
        direct_eval.is_none(),
        "optional Call keeps indirect evaluation"
    );
    (
        identifier(callee),
        this_arg.as_deref().map(identifier),
        args,
    )
}

fn identifier(value: &TypedExpr) -> &str {
    let ExprIr::Identifier(name) = &value.expr else {
        panic!("retained activation binding")
    };
    name
}

fn owned(function: &FunctionIr, names: &[&str]) {
    let mut slots = BTreeSet::new();
    for name in names {
        let found = function
            .owned_env_bindings
            .iter()
            .filter(|binding| binding.name == *name)
            .collect::<Vec<_>>();
        assert_eq!(found.len(), 1, "{name}: {:?}", function.owned_env_bindings);
        assert!(
            slots.insert(found[0].slot),
            "distinct retained operands share a slot"
        );
    }
}

fn publication(statement: &StatementIr) -> (&str, &TypedExpr) {
    let StatementIr::Expression(TypedExpr {
        expr: ExprIr::AssignIdentifier { name, value },
        ..
    }) = statement
    else {
        panic!("Normal-only joined result publication")
    };
    (name, value)
}

fn awaits(statements: &[&StatementIr]) -> Vec<(u32, u32)> {
    statements
        .iter()
        .filter_map(|statement| match statement {
            StatementIr::AsyncAwait {
                suspend_state,
                resume_state,
                ..
            } => Some((*suspend_state, *resume_state)),
            _ => None,
        })
        .collect()
}

fn skipped(statement: &StatementIr) -> &str {
    let statements = arm(statement);
    assert_eq!(
        statements.len(),
        1,
        "the whole skipped suffix has no operand work"
    );
    let (name, value) = publication(statements[0]);
    assert!(matches!(value.expr, ExprIr::Undefined));
    name
}

#[test]
fn selected_property_call_captures_one_get_before_spread_and_awaited_arguments() {
    let function = choose(
        "async function choose(object, key, spread, argument, last) {
        const result = object?.[await key](...spread, await argument, last);
        await 0;
        return result;
    }",
    );
    let mut top = Vec::new();
    flatten(&function.body.statements, &mut top);
    let (branch_index, skipped_arm, selected_arm, plan) = top
        .iter()
        .enumerate()
        .find_map(|(index, statement)| match statement {
            StatementIr::AsyncFunctionIf {
                then_branch,
                else_branch: Some(branch),
                plan,
                ..
            } => Some((index, then_branch.as_ref(), branch.as_ref(), plan)),
            _ => None,
        })
        .expect("whole-suffix optional guard");
    let result = skipped(skipped_arm);
    let selected = arm(selected_arm);
    assert_eq!(
        awaits(&selected),
        [
            (plan.else_entry_state(), plan.else_entry_state() + 1),
            (plan.else_entry_state() + 1, plan.else_entry_state() + 2)
        ]
    );
    assert_eq!(plan.exit_state(), plan.else_entry_state() + 3);
    let capture = captures(&selected);
    assert_eq!(capture.len(), 1);
    let capture = capture[0];
    let ExprIr::OptionalPropertyChain { target, chain } = &capture.chain_expression().expr else {
        panic!("actual selected property Reference")
    };
    let [OptionalChainOperationIr::Property {
        key: PropertyKeyIr::StringExpr(key),
        shorted: false,
    }] = chain.as_slice()
    else {
        panic!("one terminal Get")
    };
    assert_eq!(key.possible_kinds, KindSet::all_runtime_tags());
    let base = identifier(target);
    assert_eq!(
        top.iter()
            .filter(|statement| matches!(statement,
        StatementIr::Lexical { name, init: TypedExpr { expr: ExprIr::Identifier(source), .. }, .. }
            if name == base && source == "object"))
            .count(),
        1
    );
    assert!(
        !expressions(&selected).iter().any(|value| matches!(
            value.expr,
            ExprIr::PropertyRead { .. } | ExprIr::OptionalPropertyChain { .. }
        )),
        "Get is inside the consumed Reference capture, never replayed"
    );
    let call_values = calls(&selected);
    assert_eq!(call_values.len(), 1);
    let (callee, receiver, args) = call(call_values[0]);
    assert_eq!(receiver, Some(capture.receiver().storage_name()));
    assert_eq!(args.len(), 3);
    let ExprIr::CapturedArgumentList(list) = &args[0].expr else {
        panic!("private spread snapshot")
    };
    let list_name = identifier(list.binding());
    let capture_index = selected
        .iter()
        .position(|statement| !captures(&[*statement]).is_empty())
        .unwrap();
    let spread_index = selected.iter().position(|statement| matches!(statement,
        StatementIr::Lexical { name, init: TypedExpr { expr: ExprIr::CaptureArgumentList(_), .. }, .. }
            if name == list_name)).expect("stored argument-list owner");
    let StatementIr::Lexical {
        init:
            TypedExpr {
                expr: ExprIr::CaptureArgumentList(spread),
                ..
            },
        ..
    } = selected[spread_index]
    else {
        unreachable!()
    };
    let [TypedExpr {
        expr: ExprIr::SpreadArgument(spread),
        ..
    }] = spread.arguments()
    else {
        panic!("selected spread iteration")
    };
    assert_eq!(identifier(&spread.value), "spread");
    let await_indexes = selected
        .iter()
        .enumerate()
        .filter_map(|(index, statement)| {
            matches!(statement, StatementIr::AsyncAwait { .. }).then_some(index)
        })
        .collect::<Vec<_>>();
    assert!(
        await_indexes[0] < capture_index
            && capture_index < spread_index
            && spread_index < await_indexes[1]
    );
    let (published, selected_value) = publication(selected.last().unwrap());
    assert_eq!(published, result);
    assert_eq!(selected_value, call_values[0]);
    assert_eq!(awaits(&top), [(plan.exit_state(), plan.exit_state() + 1)]);
    assert!(
        branch_index
            < top
                .iter()
                .position(|statement| matches!(statement, StatementIr::AsyncAwait { .. }))
                .unwrap()
    );
    owned(
        &function,
        &[
            base,
            result,
            callee,
            receiver.unwrap(),
            list_name,
            identifier(&args[1]),
            identifier(&args[2]),
        ],
    );
}

#[test]
fn first_optional_call_tests_the_callee_saved_with_its_ordinary_property_receiver() {
    let function = choose(
        "async function choose(object, argument) {
        const result = object.method?.(await argument); await 0; return result;
    }",
    );
    let mut top = Vec::new();
    flatten(&function.body.statements, &mut top);
    let (branch_index, condition, skipped_arm, selected_arm, plan) = top
        .iter()
        .enumerate()
        .find_map(|(index, statement)| match statement {
            StatementIr::AsyncFunctionIf {
                condition,
                then_branch,
                else_branch: Some(branch),
                plan,
            } => Some((
                index,
                condition,
                then_branch.as_ref(),
                branch.as_ref(),
                plan,
            )),
            _ => None,
        })
        .unwrap();
    let result = skipped(skipped_arm);
    let selected = arm(selected_arm);
    let call_values = calls(&selected);
    assert_eq!(call_values.len(), 1);
    let (callee, receiver, args) = call(call_values[0]);
    let receiver = receiver.expect("ordinary member Reference receiver");
    let callee_index = top.iter().position(|statement| matches!(statement,
        StatementIr::Lexical { name, init: TypedExpr { expr: ExprIr::SpecOperation { operation: SpecOperationIr::GetV, operands }, .. }, .. }
            if name == callee && operands.len() == 2 && identifier(&operands[0]) == receiver
                && matches!(&operands[1].expr, ExprIr::String(key) if key == "method")))
        .expect("one eager property Get supplies saved callee");
    let receiver_index = top
        .iter()
        .position(|statement| {
            matches!(statement,
        StatementIr::Lexical { name, init: TypedExpr { expr: ExprIr::Identifier(source), .. }, .. }
            if name == receiver && source == "object")
        })
        .unwrap();
    assert!(receiver_index < callee_index && callee_index < branch_index);
    assert!(captures(&top).is_empty());
    let ExprIr::LogicalShortCircuit { lhs, rhs, .. } = &condition.expr else {
        panic!("strict nullish predicate")
    };
    for comparison in [lhs, rhs] {
        let ExprIr::StrictEquality { lhs, .. } = &comparison.expr else {
            panic!("pure equality")
        };
        assert_eq!(
            identifier(lhs),
            callee,
            "optional Call tests GetValue, not receiver"
        );
    }
    assert_eq!(
        awaits(&selected),
        [(plan.else_entry_state(), plan.else_entry_state() + 1)]
    );
    assert_eq!(publication(selected.last().unwrap()).0, result);
    assert_eq!(args.len(), 1);
    owned(&function, &[callee, receiver, identifier(&args[0]), result]);
}

#[test]
fn call_results_survive_later_key_await_and_consecutive_call_has_no_property_receiver() {
    let function = choose(
        "async function choose(object, first, second, third) {
        const result = object?.make(await first)?.[await second](await third)();
        await 0; return result;
    }",
    );
    let mut statements = Vec::new();
    all(&function.body.statements, &mut statements);
    let call_values = calls(&statements);
    assert_eq!(call_values.len(), 3);
    let captures = captures(&statements);
    assert_eq!(captures.len(), 2);
    let first_call = call(call_values[0]);
    let second_call = call(call_values[1]);
    let final_call = call(call_values[2]);
    assert_eq!(first_call.1, Some(captures[0].receiver().storage_name()));
    assert_eq!(second_call.1, Some(captures[1].receiver().storage_name()));
    assert_ne!(first_call.1, second_call.1);
    assert!(final_call.1.is_none(), "a completed Call is a Value");
    assert!(final_call.2.is_empty());
    let saved_child = statements
        .iter()
        .enumerate()
        .find_map(|(index, statement)| match statement {
            StatementIr::Lexical { name, init, .. } if init == call_values[0] => {
                Some((index, name))
            }
            _ => None,
        })
        .expect("first Call completed into an activation-owned child value");
    let ExprIr::OptionalPropertyChain { target, .. } = &captures[1].chain_expression().expr else {
        panic!("later property capture")
    };
    assert_eq!(identifier(target), saved_child.1);
    let await_indexes = statements
        .iter()
        .enumerate()
        .filter_map(|(index, statement)| {
            matches!(statement, StatementIr::AsyncAwait { .. }).then_some(index)
        })
        .collect::<Vec<_>>();
    assert_eq!(await_indexes.len(), 4);
    assert!(await_indexes[0] < saved_child.0 && saved_child.0 < await_indexes[1]);
    assert!(
        statements.iter().any(|statement| matches!(statement,
        StatementIr::Lexical { name, init, .. } if name == final_call.0 && init == call_values[1])),
        "consecutive Call consumes the retained preceding Call result"
    );
    for statement in &statements {
        if let StatementIr::AsyncFunctionIf { then_branch, .. } = statement {
            skipped(then_branch);
        }
    }
    owned(
        &function,
        &[
            saved_child.1,
            first_call.0,
            first_call.1.unwrap(),
            second_call.0,
            second_call.1.unwrap(),
            final_call.0,
        ],
    );
}

#[test]
fn grouped_terminal_property_after_a_call_retains_outer_call_and_tag_reference() {
    for source in [
        "async function choose(object, first, key, argument) { const result = (object?.make(await first)[await key])(await argument); await 0; return result; }",
        "async function choose(object, first, key, argument) { const result = (object?.make(await first)[await key])`head${await argument}tail`; await 0; return result; }",
    ] {
        let function = choose(source);
        let mut top = Vec::new(); flatten(&function.body.statements, &mut top);
        let mut statements = Vec::new(); all(&function.body.statements, &mut statements);
        let capture_values = captures(&statements); assert_eq!(capture_values.len(), 2);
        let call_values = calls(&statements); assert_eq!(call_values.len(), 2);
        let terminal = capture_values[1];
        let outer = call(call_values[1]);
        assert_eq!(outer.1, Some(terminal.receiver().storage_name()));
        assert_ne!(outer.1, call(call_values[0]).1);
        let terminal_index = statements.iter().position(|statement|
            captures(&[*statement]).iter().any(|capture| *capture == terminal)).unwrap();
        let await_indexes = statements.iter().enumerate().filter_map(|(index, statement)|
            matches!(statement, StatementIr::AsyncAwait { .. }).then_some(index)).collect::<Vec<_>>();
        assert_eq!(await_indexes.len(), 4);
        assert!(await_indexes[1] < terminal_index && terminal_index < await_indexes[2]);
        assert_eq!(awaits(&top).len(), 2, "outer operand and following Await are unconditional");
        let callee_index = statements.iter().position(|statement| matches!(statement,
            StatementIr::Lexical { name, .. } if name == outer.0)).unwrap();
        assert!(terminal_index < callee_index && callee_index < await_indexes[2]);
        owned(&function, &[outer.0, outer.1.unwrap(), call(call_values[0]).0,
            call(call_values[0]).1.unwrap()]);
    }
}

#[test]
fn first_call_over_grouped_awaited_reference_has_its_own_selected_argument_states() {
    let function = choose(
        "async function choose(object, key, argument) {
        const result = (object?.[await key])?.(await argument); await 0; return result;
    }",
    );
    let mut statements = Vec::new();
    all(&function.body.statements, &mut statements);
    let guards = statements
        .iter()
        .filter_map(|statement| match statement {
            StatementIr::AsyncFunctionIf {
                then_branch, plan, ..
            } => {
                skipped(then_branch);
                Some((plan.entry_state(), plan.exit_state()))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        guards,
        [(0, 4), (4, 8)],
        "inner Reference completion precedes outer callee guard"
    );
    assert_eq!(awaits(&statements), [(2, 3), (6, 7), (8, 9)]);
    let capture_values = captures(&statements);
    assert_eq!(capture_values.len(), 1);
    let call_values = calls(&statements);
    assert_eq!(call_values.len(), 1);
    let (callee, receiver, args) = call(call_values[0]);
    assert_eq!(receiver, Some(capture_values[0].receiver().storage_name()));
    assert_eq!(args.len(), 1);
    owned(
        &function,
        &[callee, receiver.unwrap(), identifier(&args[0])],
    );
}

#[test]
fn erased_argument_await_keeps_selected_call_without_inventing_resume_states() {
    let function = choose(
        "async function choose(object, forbidden) {
        const result = object?.method((undefined)?.[await forbidden]); await 0; return result;
    }",
    );
    let mut statements = Vec::new();
    all(&function.body.statements, &mut statements);
    assert!(statements
        .iter()
        .any(|statement| matches!(statement, StatementIr::If { .. })));
    assert!(!statements
        .iter()
        .any(|statement| matches!(statement, StatementIr::AsyncFunctionIf { .. })));
    assert_eq!(awaits(&statements), [(0, 1)]);
    let capture_values = captures(&statements);
    assert_eq!(capture_values.len(), 1);
    let call_values = calls(&statements);
    assert_eq!(call_values.len(), 1);
    let (_, receiver, args) = call(call_values[0]);
    assert_eq!(receiver, Some(capture_values[0].receiver().storage_name()));
    assert_eq!(args.len(), 1);
    let argument = identifier(&args[0]);
    let init = statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Lexical { name, init, .. } if name == argument => Some(init),
            _ => None,
        })
        .expect("erased chain result is still evaluated and retained as an argument");
    let ExprIr::MaterializeBinding { value, body, .. } = &init.expr else {
        panic!("static nullish target evaluation is preserved")
    };
    assert!(
        matches!(value.expr, ExprIr::Undefined)
            || matches!(&value.expr, ExprIr::GlobalPropertyRead { name } if name == "undefined")
    );
    assert!(matches!(body.expr, ExprIr::Undefined));
    owned(&function, &[argument, receiver.unwrap()]);
}

#[test]
fn optional_eval_retains_indirect_call_and_known_nonstring_first_argument() {
    let function = choose("async function choose(argument) { return eval?.(42, await argument); }");
    let mut statements = Vec::new();
    all(&function.body.statements, &mut statements);
    let call_values = calls(&statements);
    assert_eq!(call_values.len(), 1);
    let (callee, receiver, args) = call(call_values[0]);
    assert!(receiver.is_none());
    assert_eq!(args.len(), 2);
    assert!(matches!(args[0].expr, ExprIr::Number(bits) if bits == 42_f64.to_bits()));
    assert_eq!(awaits(&statements), [(2, 3)]);
    owned(&function, &[callee, identifier(&args[1])]);
}

#[test]
fn optional_outer_call_tests_the_completed_inner_call_value_and_guards_its_arguments() {
    let function = choose(
        "async function choose(object, value, argument) {
        const result = (object?.method(await value))?.(await argument); await 0; return result;
    }",
    );
    let mut top = Vec::new();
    flatten(&function.body.statements, &mut top);
    let guards = top
        .iter()
        .enumerate()
        .filter_map(|(index, statement)| match statement {
            StatementIr::AsyncFunctionIf {
                condition,
                then_branch,
                else_branch: Some(selected),
                plan,
            } => Some((
                index,
                condition,
                then_branch.as_ref(),
                selected.as_ref(),
                plan,
            )),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(guards.len(), 2);
    assert_eq!(
        guards
            .iter()
            .map(|(_, _, _, _, plan)| (plan.entry_state(), plan.exit_state()))
            .collect::<Vec<_>>(),
        [(0, 4), (4, 8)]
    );
    let inner_result = skipped(guards[0].2);
    let outer_result = skipped(guards[1].2);
    assert_ne!(inner_result, outer_result);
    let inner = arm(guards[0].3);
    let selected = arm(guards[1].3);
    let inner_calls = calls(&inner);
    let outer_calls = calls(&selected);
    assert_eq!(inner_calls.len(), 1);
    assert_eq!(outer_calls.len(), 1);
    let inner_call = call(inner_calls[0]);
    let outer_call = call(outer_calls[0]);
    assert!(inner_call.1.is_some() && outer_call.1.is_none());
    assert_eq!(
        publication(inner.last().unwrap()),
        (inner_result, inner_calls[0])
    );
    assert_eq!(
        publication(selected.last().unwrap()),
        (outer_result, outer_calls[0])
    );
    let pin = top
        .iter()
        .position(|statement| {
            matches!(statement,
        StatementIr::Lexical { name, init: TypedExpr { expr: ExprIr::Identifier(saved), .. }, .. }
            if name == outer_call.0 && saved == inner_result)
        })
        .unwrap();
    assert!(guards[0].0 < pin && pin < guards[1].0);
    let ExprIr::LogicalShortCircuit { lhs, rhs, .. } = &guards[1].1.expr else {
        panic!("strict outer callee nullish guard")
    };
    for comparison in [lhs, rhs] {
        let ExprIr::StrictEquality { lhs, .. } = &comparison.expr else {
            panic!("pure callee equality")
        };
        assert_eq!(identifier(lhs), outer_call.0);
    }
    assert_eq!(awaits(&inner), [(2, 3)]);
    assert_eq!(awaits(&selected), [(6, 7)]);
    assert_eq!(
        awaits(&top),
        [(8, 9)],
        "outer optional argument remains inside its own selected arm"
    );
    let mut statements = Vec::new();
    all(&function.body.statements, &mut statements);
    let references = captures(&statements);
    assert_eq!(references.len(), 1);
    assert_eq!(inner_call.1, Some(references[0].receiver().storage_name()));
    owned(
        &function,
        &[
            inner_result,
            outer_result,
            inner_call.0,
            inner_call.1.unwrap(),
            outer_call.0,
            identifier(&inner_call.2[0]),
            identifier(&outer_call.2[0]),
        ],
    );
}

#[test]
fn first_optional_call_consumes_target_only_grouped_property_reference_before_selected_argument_states(
) {
    let function=choose("async function choose(object, argument) { const result = ((await object)?.method)?.(await argument); await 0; return result; }");
    let mut top = Vec::new();
    flatten(&function.body.statements, &mut top);
    let mut statements = Vec::new();
    all(&function.body.statements, &mut statements);
    let references = captures(&statements);
    assert_eq!(references.len(), 1);
    let call_values = calls(&statements);
    assert_eq!(call_values.len(), 1);
    let (callee, receiver, args) = call(call_values[0]);
    assert_eq!(receiver, Some(references[0].receiver().storage_name()));
    assert_eq!(args.len(), 1);
    let guards = statements
        .iter()
        .filter_map(|statement| match statement {
            StatementIr::AsyncFunctionIf {
                then_branch, plan, ..
            } => {
                skipped(then_branch);
                Some((plan.entry_state(), plan.exit_state()))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        guards,
        [(1, 5)],
        "only the optional Call arguments own conditional Await states"
    );
    assert_eq!(awaits(&statements), [(0, 1), (3, 4), (5, 6)]);
    let get_index = top
        .iter()
        .position(|statement| !captures(&[*statement]).is_empty())
        .unwrap();
    let target_await = top
        .iter()
        .position(|statement| {
            matches!(
                statement,
                StatementIr::AsyncAwait {
                    suspend_state: 0,
                    resume_state: 1,
                    ..
                }
            )
        })
        .unwrap();
    let guard_index = top
        .iter()
        .position(|statement| matches!(statement, StatementIr::AsyncFunctionIf { .. }))
        .unwrap();
    assert!(
        target_await < get_index && get_index < guard_index,
        "the original Reference is completed before optional callee testing"
    );
    owned(
        &function,
        &[callee, receiver.unwrap(), identifier(&args[0])],
    );
}

#[test]
fn target_only_terminal_call_keeps_the_inner_property_receiver_and_outer_value_receiver_free() {
    let function = choose(
        "async function choose(object, key, argument) {
        const result = ((object?.[await key])?.())(await argument); await 0; return result;
    }",
    );
    let mut top = Vec::new();
    flatten(&function.body.statements, &mut top);
    let mut statements = Vec::new();
    all(&function.body.statements, &mut statements);
    let async_guards = statements
        .iter()
        .filter_map(|statement| match statement {
            StatementIr::AsyncFunctionIf { plan, .. } => {
                Some((plan.entry_state(), plan.exit_state()))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(async_guards, [(0, 4)]);
    assert_eq!(awaits(&statements), [(2, 3), (4, 5), (5, 6)]);
    let references = captures(&statements);
    assert_eq!(references.len(), 1);
    let call_values = calls(&statements);
    assert_eq!(call_values.len(), 2);
    let inner = call(call_values[0]);
    let outer = call(call_values[1]);
    assert_eq!(inner.1, Some(references[0].receiver().storage_name()));
    assert!(inner.2.is_empty() && outer.1.is_none());
    let (joined, selected) = top
        .iter()
        .find_map(|statement| match statement {
            StatementIr::If {
                then_branch,
                else_branch: Some(selected),
                ..
            } => Some((skipped(then_branch), arm(selected))),
            _ => None,
        })
        .expect("the target-only synchronous Call has an eager nullish guard");
    assert_eq!(
        publication(selected.last().unwrap()),
        (joined, call_values[0])
    );
    assert!(awaits(&selected).is_empty());
    let pin = top
        .iter()
        .position(|statement| {
            matches!(statement,
        StatementIr::Lexical { name, init: TypedExpr { expr: ExprIr::Identifier(saved), .. }, .. }
            if name == outer.0 && saved == joined)
        })
        .unwrap();
    let outer_await = top
        .iter()
        .position(|statement| {
            matches!(
                statement,
                StatementIr::AsyncAwait {
                    suspend_state: 4,
                    resume_state: 5,
                    ..
                }
            )
        })
        .unwrap();
    assert!(pin < outer_await);
    owned(
        &function,
        &[
            joined,
            inner.0,
            inner.1.unwrap(),
            outer.0,
            identifier(&outer.2[0]),
        ],
    );
}

#[test]
fn erased_link_await_publishes_terminal_call_value_without_branch_resume_states() {
    let function = choose("async function choose(object, forbidden, argument) {
        const result = (object?.method((undefined)?.[await forbidden]))(await argument); await 0; return result;
    }");
    let mut top = Vec::new();
    flatten(&function.body.statements, &mut top);
    let mut statements = Vec::new();
    all(&function.body.statements, &mut statements);
    assert!(!statements
        .iter()
        .any(|statement| matches!(statement, StatementIr::AsyncFunctionIf { .. })));
    assert_eq!(awaits(&statements), [(0, 1), (1, 2)]);
    let call_values = calls(&statements);
    assert_eq!(call_values.len(), 2);
    let inner = call(call_values[0]);
    let outer = call(call_values[1]);
    assert!(inner.1.is_some() && outer.1.is_none());
    let (joined, selected) = top
        .iter()
        .find_map(|statement| match statement {
            StatementIr::If {
                then_branch,
                else_branch: Some(selected),
                ..
            } => Some((skipped(then_branch), arm(selected))),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        publication(selected.last().unwrap()),
        (joined, call_values[0])
    );
    let pin = top
        .iter()
        .position(|statement| {
            matches!(statement,
        StatementIr::Lexical { name, init: TypedExpr { expr: ExprIr::Identifier(saved), .. }, .. }
            if name == outer.0 && saved == joined)
        })
        .unwrap();
    let outer_await = top
        .iter()
        .position(|statement| {
            matches!(
                statement,
                StatementIr::AsyncAwait {
                    suspend_state: 0,
                    resume_state: 1,
                    ..
                }
            )
        })
        .unwrap();
    assert!(pin < outer_await);
    let erased_argument = identifier(&inner.2[0]);
    assert!(selected.iter().any(|statement| matches!(statement,
        StatementIr::Lexical { name, init: TypedExpr { expr: ExprIr::MaterializeBinding { value, body, .. }, .. }, .. }
            if name == erased_argument
                && (matches!(value.expr, ExprIr::Undefined)
                    || matches!(&value.expr, ExprIr::GlobalPropertyRead { name } if name == "undefined"))
                && matches!(body.expr, ExprIr::Undefined))));
    owned(
        &function,
        &[
            joined,
            inner.0,
            inner.1.unwrap(),
            erased_argument,
            outer.0,
            identifier(&outer.2[0]),
        ],
    );
}

#[test]
fn grouped_call_values_compose_through_actual_sources_without_widening_property_references() {
    for source in [
        "async function choose(object, value, argument) { return (object?.method(await value))(await argument); }",
        "async function choose(object, value, argument) { return await (object?.method(await value))(await argument); }",
        "async function choose(object, value, argument) { var result = (object?.method(await value))(await argument); return result; }",
        "async function choose(object, value, argument) { (object?.method(await value))(await argument); }",
        "async function choose(object, key) { return (object?.[await key]())(); }",
        "async function choose(object, key) { return (object?.[await key]?.())(); }",
        "async function choose(object, key) { return (object?.[await key]())`tag`; }",
        "async function choose(object, key, argument) { return ((object?.[await key])?.())`head${await argument}tail`; }",
        "async function choose(object, value, argument) { return (object?.method(await value))?.(await argument); }",
        "async function choose(object, value, argument) { return ((object?.method(await value))?.())(await argument); }",
        "async function choose(object, value, flag, argument) { return (object?.method(flag ? await value : 0))(await argument); }",
        "async function choose(object, argument) { return ((await object)?.())(await argument); }",
        "async function choose(object, argument) { return ((await object)?.method())`head${await argument}tail`; }",
        "async function choose(object, argument) { return (((await object)?.method)?.())(await argument); }",
        "async function choose(object, argument) { return ((await object)?.method)?.(await argument); }",
    ] { choose(source); }
}

#[test]
fn admitted_declaration_return_discard_constructor_and_target_only_routes_compose() {
    for source in [
        "async function choose(object, key) { return object?.(await key); }",
        "async function choose(object, key) { return object?.[await key](); }",
        "async function choose(object, key) { return object?.[await key]?.(); }",
        "async function choose(object, key) { var result = object?.method(await key); return result; }",
        "async function choose(object, key) { object.method?.(await key); }",
        "async function choose(object, key, flag) { return object?.method(flag ? await key : 0); }",
        "async function choose(object, key) { return new (object?.make(await key).C)(1); }",
        "async function choose(object) { return (await object)?.(); }",
        "async function choose(object, argument) { return (object?.method)(await argument); }",
        "class C { #method; async choose(object, key) { return object.#method?.(await key); } }",
        "class C { #method; async choose(object, key) { return ((object.#method))?.(await key); } }",
        "class C { #method; async choose(object, key, argument) { return (object.#method?.(await key))(await argument); } }",
        "class C { #method; async choose(object, key, argument) { return (object.#method?.(await key).method)(await argument); } }",
        "class C { #value; async choose(object, key) { return object.#value?.method(await key); } }",
        "class C { #value; async choose(object, key) { return ((object.#value))?.method(await key); } }",
        "class C { #value; async choose(object, key) { return object.#value?.[await key]; } }",
        "class C { #value; async choose(object, key, argument) { return (object.#value?.[await key])(await argument); } }",
    ] { choose(source); }
}

#[test]
fn unowned_references_and_nonordinary_owners_are_refused_before_states() {
    for source in [
        "async function* choose(object, value) { return (object?.method(await value))(); }",
        "async function* choose(object, value) { return (object?.method(await value))`tag${yield 1}`; }",
        "async function* choose(object, key) { return object?.method(await key); }",
        "async function* choose(object, key) { return object?.method(await key, yield 1); }",
    ] { assert!(choose(source).resumable_plan.is_some()); }
    for source in [
        "async function choose(object, key) { return delete object?.method(await key); }",
        "class C { #method; async choose(object, key) { return object?.#method(await key); } }",
        "class C extends Object { async choose(key) { return super.method?.(await key); } }",
    ] {
        choose(source);
    }
    for source in [
        "async function choose(object, value) { while ((object?.method(await value))()) { break; } }",
        "async function choose(object, value) { while (true) { (object?.method(await value))(); break; } }",
        "async function choose(object, value) { for (; true; ) { (object?.method(await value))`tag`; break; } }",
        "async function choose(object, value) { do { (object?.method(await value))(); } while (false); }",
        "async function choose(object, key) { while (object?.method(await key)) { break; } }",
        "async function choose(object, key) { while (true) { object?.method(await key); break; } }",
        "async function choose(object, key) { for (; object?.method(await key); ) { break; } }",
        "async function choose(object, key) { do { break; } while (object?.method(await key)); }",
    ] {
        statement_awaits::assert_count(&choose(source), 1);
    }
}
