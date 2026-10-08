use std::collections::BTreeSet;

#[path = "common/statement_awaits.rs"]
mod statement_awaits;

use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, AsyncResumeModeIr, EnvironmentIdentifierOperationIr, ExprIr, FunctionIr,
    FunctionProtocolIr, IdentifierReferenceCaptureAccess, KindSet, OrdinaryPropertyAssignmentIr,
    PropertyKeyIr, StatementIr, Strictness, TypedExpr,
};

fn lower_assignment(source: &str) -> FunctionIr {
    let unit = parse(source, ParseOptions::script()).expect("assignment fixture parses");
    let program = lower(&unit);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    program
        .script
        .expect("script IR")
        .functions
        .into_iter()
        .find(|function| function.name == "assign")
        .expect("async assignment function")
}

fn flatten<'a>(statements: &'a [StatementIr], result: &mut Vec<&'a StatementIr>) {
    for statement in statements {
        match statement {
            StatementIr::LexicalBlock(statements) => flatten(statements, result),
            StatementIr::Block(block) => flatten(&block.statements, result),
            StatementIr::EmptyStatementCompletion(item) => {
                flatten(std::slice::from_ref(item.statement()), result)
            }
            _ => result.push(statement),
        }
    }
}

fn identifier(value: &TypedExpr) -> &str {
    let ExprIr::Identifier(name) = &value.expr else {
        panic!("retained operand must read its activation binding: {value:?}");
    };
    name
}

// Retained operands can cross more than one private cell. Trace only earlier
// definitions, stopping at the original read or the actual await result.
fn operand_origin<'a>(statements: &[&'a StatementIr], mut name: &'a str) -> (usize, &'a TypedExpr) {
    let mut before = statements.len();
    let mut origin = None;
    loop {
        let definition =
            statements[..before]
                .iter()
                .enumerate()
                .rev()
                .find_map(|(index, statement)| match statement {
                    StatementIr::Lexical {
                        name: binding,
                        init,
                        ..
                    } if binding == name => Some((index, init, false)),
                    StatementIr::AsyncAwait {
                        value,
                        resume_mode: AsyncResumeModeIr::AssignIdentifier(binding),
                        ..
                    } if binding == name => Some((index, value, true)),
                    _ => None,
                });
        let Some((index, value, resumed)) = definition else {
            return origin.expect("operand must have an earlier retained definition");
        };
        origin = Some((index, value));
        if resumed {
            return (index, value);
        }
        let ExprIr::Identifier(previous) = &value.expr else {
            return (index, value);
        };
        name = previous;
        before = index;
    }
}

fn assignment<'a>(statements: &[&'a StatementIr]) -> &'a OrdinaryPropertyAssignmentIr {
    statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Expression(TypedExpr {
                expr: ExprIr::OrdinaryPropertyAssignment(assignment),
                ..
            })
            | StatementIr::Return(TypedExpr {
                expr: ExprIr::OrdinaryPropertyAssignment(assignment),
                ..
            }) => Some(assignment),
            _ => None,
        })
        .expect("one fused ordinary property Reference must remain after the prefix")
}

#[test]
fn awaited_rhs_keeps_raw_reference_operands_in_distinct_activation_slots() {
    for declaration in ["async function", "async function*"] {
        let function = lower_assignment(&format!(
            "{declaration} assign(target, key, rhs) {{ 'use strict'; target[key] = await rhs; }}"
        ));
        let mut statements = Vec::new();
        flatten(&function.body.statements, &mut statements);
        let assignment = assignment(&statements);
        assert_eq!(assignment.strictness(), Strictness::Strict);
        let PropertyKeyIr::StringExpr(key) = assignment.referenced_name() else {
            panic!("the computed key must remain an uncoerced expression");
        };
        let names = [
            identifier(assignment.base_and_receiver()),
            identifier(key),
            identifier(assignment.rhs()),
        ];
        let slots = names.map(|name| {
            let bindings = function
                .owned_env_bindings
                .iter()
                .filter(|binding| binding.name == name)
                .collect::<Vec<_>>();
            assert_eq!(bindings.len(), 1, "{name}");
            bindings[0].slot
        });
        assert_eq!(slots.into_iter().collect::<BTreeSet<_>>().len(), 3);
        assert_eq!(assignment.rhs().possible_kinds, KindSet::all_runtime_tags());
        let await_position = statements
            .iter()
            .position(|statement| matches!(statement, StatementIr::AsyncAwait { .. }))
            .expect("the RHS must suspend");
        for (name, original) in names[..2].iter().zip(["target", "key"]) {
            let (position, value) = operand_origin(&statements, name);
            assert_eq!(identifier(value), original);
            assert!(position < await_position);
        }
        let (rhs_position, rhs) = operand_origin(&statements, names[2]);
        assert_eq!(rhs_position, await_position);
        assert_eq!(identifier(rhs), "rhs");
    }
}

#[test]
fn awaited_key_and_rhs_have_ordered_distinct_resume_boundaries() {
    let function = lower_assignment(
        "async function assign(target, key, rhs) { target[await key] = await rhs; }",
    );
    let mut statements = Vec::new();
    flatten(&function.body.statements, &mut statements);
    let assignment = assignment(&statements);
    let PropertyKeyIr::StringExpr(key) = assignment.referenced_name() else {
        panic!("computed key must remain raw");
    };
    let positions = statements
        .iter()
        .enumerate()
        .filter_map(|(position, statement)| match statement {
            StatementIr::AsyncAwait {
                suspend_state,
                resume_state,
                ..
            } => Some((position, *suspend_state, *resume_state)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(positions.len(), 2);
    assert_eq!((positions[0].1, positions[0].2), (0, 1));
    assert_eq!((positions[1].1, positions[1].2), (1, 2));
    let (base_position, base) =
        operand_origin(&statements, identifier(assignment.base_and_receiver()));
    assert_eq!(identifier(base), "target");
    assert!(base_position < positions[0].0);
    let (key_position, key_source) = operand_origin(&statements, identifier(key));
    assert_eq!(key_position, positions[0].0);
    assert_eq!(identifier(key_source), "key");
    let (rhs_position, rhs_source) = operand_origin(&statements, identifier(assignment.rhs()));
    assert_eq!(rhs_position, positions[1].0);
    assert_eq!(identifier(rhs_source), "rhs");
}

#[test]
fn compound_property_assignment_awaits_retain_complete_selected_continuations() {
    for (expression, awaits) in [
        ("target.value = (missing &&= await rhs)", 1),
        ("target[await 0] &&= (target.value = await rhs)", 2),
        (
            "consume(flag ? (target.value = (missing ||= await rhs)) : 0)",
            1,
        ),
    ] {
        let source =
            format!("async function assign(target, flag, rhs, consume) {{ {expression}; }}");
        let function = lower_assignment(&source);
        statement_awaits::assert_count(&function, awaits);
    }
    let function = lower_assignment(
        "async function assign(target, rhs) { null?.method(target.value = await rhs); }",
    );
    let mut statements = Vec::new();
    flatten(&function.body.statements, &mut statements);
    assert!(!statements
        .iter()
        .any(|statement| matches!(statement, StatementIr::AsyncAwait { .. })));
}

#[test]
fn direct_identifier_await_assignment_puts_the_original_write_only_reference_after_resume() {
    let function = lower_assignment("async function assign(target, rhs) { target = await rhs; }");
    let mut statements = Vec::new();
    flatten(&function.body.statements, &mut statements);
    let (capture_position, capture) = statements
        .iter()
        .enumerate()
        .find_map(|(position, statement)| {
            let StatementIr::Expression(TypedExpr {
                expr: ExprIr::EnvironmentIdentifier(identifier),
                ..
            }) = statement
            else {
                return None;
            };
            let EnvironmentIdentifierOperationIr::CaptureAssignmentReference { capture } =
                &identifier.operation
            else {
                return None;
            };
            assert_eq!(identifier.name, "target");
            Some((position, capture))
        })
        .expect("locate the original Reference before evaluating the RHS");
    assert_eq!(
        capture.access(),
        IdentifierReferenceCaptureAccess::WriteOnly
    );
    let (await_position, resumed) = statements
        .iter()
        .enumerate()
        .find_map(|(position, statement)| match statement {
            StatementIr::AsyncAwait {
                resume_mode: AsyncResumeModeIr::AssignIdentifier(name),
                ..
            } => Some((position, name)),
            _ => None,
        })
        .expect("RHS suspension");
    let put_position = statements.iter().position(|statement| {
        matches!(statement, StatementIr::Expression(TypedExpr { expr: ExprIr::EnvironmentIdentifier(identifier), .. })
            if matches!(&identifier.operation, EnvironmentIdentifierOperationIr::PutCapturedReference { reference, value }
                if reference == capture.reference() && matches!(&value.expr, ExprIr::Identifier(name) if name == resumed)))
    }).expect("PutValue consumes the same captured Reference and resumed value");
    assert!(capture_position < await_position && await_position < put_position);
}

#[test]
fn async_arrow_capture_hops_include_activation_frames_before_operand_slots_exist() {
    let unit = parse(
        "function outer(target, key, rhs) { let assign = async () => target[key] = await rhs; let make = async () => () => target; return [assign, make]; }",
        ParseOptions::script(),
    )
    .expect("nested async arrows parse");
    let program = lower(&unit);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.expect("script IR");
    let assign = script
        .functions
        .iter()
        .find(|function| function.name == "assign")
        .expect("assignment arrow");
    assert_eq!(assign.protocol, FunctionProtocolIr::AsyncArrow);
    let mut statements = Vec::new();
    flatten(&assign.body.statements, &mut statements);
    let reference = assignment(&statements);
    let PropertyKeyIr::StringExpr(key) = reference.referenced_name() else {
        panic!("computed raw key");
    };
    for operand in [reference.base_and_receiver(), key, reference.rhs()] {
        assert!(assign
            .owned_env_bindings
            .iter()
            .any(|binding| binding.name == identifier(operand)));
    }
    for source_name in ["target", "key", "rhs"] {
        let capture = assign
            .captured_bindings
            .iter()
            .find(|capture| capture.source_name == source_name)
            .expect("source operand capture");
        assert_eq!(capture.hops, 1, "{source_name}");
    }
    let make = script
        .functions
        .iter()
        .find(|function| function.name == "make")
        .expect("empty async parent");
    assert_eq!(make.protocol, FunctionProtocolIr::AsyncArrow);
    assert!(make.owned_env_bindings.is_empty());
    let reader = script
        .functions
        .iter()
        .find(|function| function.protocol == FunctionProtocolIr::Arrow)
        .expect("nested ordinary arrow");
    assert!(reader.owned_env_bindings.is_empty());
    let capture = reader
        .captured_bindings
        .iter()
        .find(|capture| capture.source_name == "target")
        .expect("reader crosses its empty async parent");
    assert_eq!(capture.hops, 1);
}
