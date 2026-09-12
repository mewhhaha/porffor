use std::collections::BTreeSet;

use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, AsyncResumeModeIr, ExprIr, FunctionIr, FunctionProtocolIr, KindSet,
    OrdinaryPropertyAssignmentIr, PropertyKeyIr, StatementIr, Strictness, TypedExpr,
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
            let position = statements
                .iter()
                .position(|statement| {
                    matches!(statement,
                    StatementIr::Lexical { name: binding, init, .. }
                        if binding.as_str() == *name && identifier(init) == original)
                })
                .expect("base and raw key must be captured before the RHS");
            assert!(position < await_position);
        }
        assert!(matches!(statements[await_position],
            StatementIr::AsyncAwait { resume_mode: AsyncResumeModeIr::AssignIdentifier(name), .. }
                if name == names[2]));
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
    let retained_position = |operand: &TypedExpr| {
        statements
            .iter()
            .position(|statement| {
                matches!(statement,
                StatementIr::Lexical { name, .. } if name == identifier(operand))
            })
            .expect("retained operand initialization")
    };
    assert!(retained_position(assignment.base_and_receiver()) < positions[0].0);
    assert!(retained_position(key) > positions[0].0);
    assert!(retained_position(key) < positions[1].0);
}

#[test]
fn conditional_property_assignment_awaits_are_refused_before_prefix_hoisting() {
    for expression in [
        "target.value = flag && await rhs",
        "flag && (target.value = await rhs)",
        "consume(flag ? (target.value = await rhs) : 0)",
    ] {
        let source =
            format!("async function assign(target, flag, rhs, consume) {{ {expression}; }}");
        let unit = parse(&source, ParseOptions::script()).expect("conditional fixture parses");
        let program = lower(&unit);
        assert!(!program.is_wasm_supported(), "{source}");
        assert!(format!("{:?}", program.diagnostics)
            .contains("conditionally reached or mixed suspension in async property assignment"));
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
fn direct_identifier_await_assignment_keeps_its_existing_resume_mode() {
    let function = lower_assignment("async function assign(target, rhs) { target = await rhs; }");
    let mut statements = Vec::new();
    flatten(&function.body.statements, &mut statements);
    assert!(matches!(statements.as_slice(),
        [StatementIr::AsyncAwait { resume_mode: AsyncResumeModeIr::AssignIdentifier(name), .. }]
            if name == "target"));
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
