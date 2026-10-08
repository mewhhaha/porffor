use std::collections::BTreeSet;

#[path = "common/statement_awaits.rs"]
mod statement_awaits;

use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, AsyncFunctionIfPlanIr, BindingMode, EnvironmentIdentifierOperationIr, ExprIr,
    FunctionIr, IdentifierReferenceCaptureAccess, IdentifierReferenceCaptureDisposition,
    IdentifierReferenceCaptureIr, IdentifierReferenceFallbackDisposition,
    OrdinaryPropertyGetCaptureIr, PropertyKeyIr, StatementIr, Strictness, TypedExpr,
};

fn function(source: &str) -> FunctionIr {
    let parsed = parse(source, ParseOptions::script()).expect("logical assignment source parses");
    let program = lower(&parsed);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    program
        .script
        .expect("script")
        .functions
        .into_iter()
        .find(|function| function.name == "choose" || function.name.ends_with(".choose"))
        .expect("async function")
}

fn flatten<'a>(statements: &'a [StatementIr], output: &mut Vec<&'a StatementIr>) {
    for statement in statements {
        match statement {
            StatementIr::LexicalBlock(statements) => flatten(statements, output),
            StatementIr::Block(block) => flatten(&block.statements, output),
            StatementIr::EmptyStatementCompletion(item) => {
                flatten(std::slice::from_ref(item.statement()), output)
            }
            statement => output.push(statement),
        }
    }
}

fn branch<'a>(
    statements: &[&'a StatementIr],
) -> (&'a StatementIr, &'a StatementIr, AsyncFunctionIfPlanIr) {
    statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::AsyncFunctionIf {
                then_branch,
                else_branch: Some(else_branch),
                plan,
                ..
            } => Some((then_branch.as_ref(), else_branch.as_ref(), *plan)),
            _ => None,
        })
        .expect("selected RHS owns an If continuation")
}

fn arm(branch: &StatementIr) -> Vec<&StatementIr> {
    let StatementIr::LexicalBlock(statements) = branch else {
        panic!("arm scope")
    };
    let mut output = Vec::new();
    flatten(statements, &mut output);
    output
}

fn committed_value<'a>(statements: &[&'a StatementIr]) -> &'a TypedExpr {
    let Some(StatementIr::Expression(TypedExpr {
        expr: ExprIr::AssignIdentifier { value, .. },
        ..
    })) = statements.last()
    else {
        panic!("one result is committed after PutValue")
    };
    value
}

fn captured_get<'a>(
    statements: &[&'a StatementIr],
    source_name: &str,
) -> (usize, &'a str, &'a IdentifierReferenceCaptureIr) {
    statements
        .iter()
        .enumerate()
        .find_map(|(index, statement)| {
            let StatementIr::Lexical {
                name,
                init:
                    TypedExpr {
                        expr: ExprIr::EnvironmentIdentifier(identifier),
                        ..
                    },
                ..
            } = statement
            else {
                return None;
            };
            let EnvironmentIdentifierOperationIr::CaptureAssignmentReference { capture } =
                &identifier.operation
            else {
                return None;
            };
            if identifier.name != source_name {
                return None;
            }
            assert_eq!(
                capture.access(),
                IdentifierReferenceCaptureAccess::ReadBeforeRhs
            );
            let IdentifierReferenceCaptureDisposition::Located(
                IdentifierReferenceFallbackDisposition::Declarative { binding },
            ) = capture.disposition()
            else {
                panic!("original declarative cell");
            };
            assert!(matches!(&binding.expr, ExprIr::Identifier(name) if name == source_name));
            Some((index, name.as_str(), capture))
        })
        .expect("original Reference and GetValue precede selection")
}

fn assert_captured_put(value: &TypedExpr, capture: &IdentifierReferenceCaptureIr) {
    let ExprIr::EnvironmentIdentifier(identifier) = &value.expr else {
        panic!("PutValue must consume the retained Reference");
    };
    let EnvironmentIdentifierOperationIr::PutCapturedReference { reference, .. } =
        &identifier.operation
    else {
        panic!("one captured PutValue");
    };
    assert_eq!(reference, capture.reference());
}

#[test]
fn declarative_get_precedes_selection_and_only_selected_arm_writes_the_original_cell() {
    for (operator, selected_then, states) in [
        ("&&=", true, [0, 1, 3, 4]),
        ("||=", false, [0, 1, 2, 4]),
        ("??=", true, [0, 1, 3, 4]),
    ] {
        let function = function(&format!("async function choose(value, rhs) {{ const result = (value {operator} await rhs); await 0; return result; }}"));
        let mut statements = Vec::new();
        flatten(&function.body.statements, &mut statements);
        let (capture_position, old_value, capture) = captured_get(&statements, "value");
        let (left_position, captured) = statements
            .iter()
            .enumerate()
            .find_map(|(position, statement)| match statement {
                StatementIr::Lexical {
                    name,
                    init:
                        TypedExpr {
                            expr: ExprIr::Identifier(original),
                            ..
                        },
                    ..
                } if original == old_value => Some((position, name)),
                _ => None,
            })
            .expect("original cell is read once before selection");
        let (then_branch, else_branch, plan) = branch(&statements);
        let branch_position = statements
            .iter()
            .position(|statement| matches!(statement, StatementIr::AsyncFunctionIf { .. }))
            .unwrap();
        assert!(capture_position < left_position && left_position < branch_position);
        assert!(function
            .owned_env_bindings
            .iter()
            .any(|binding| binding.name == *captured));
        assert_eq!(
            [
                plan.entry_state(),
                plan.then_entry_state(),
                plan.else_entry_state(),
                plan.exit_state()
            ],
            states
        );
        let selected = arm(if selected_then {
            then_branch
        } else {
            else_branch
        });
        let skipped = arm(if selected_then {
            else_branch
        } else {
            then_branch
        });
        assert_eq!(skipped.len(), 1, "skipped arm has no RHS or PutValue");
        let ExprIr::Comma { lhs, rhs } = &committed_value(&skipped).expr else {
            panic!("skipped arm releases its Reference and returns the original GetValue");
        };
        assert!(
            matches!(&lhs.expr, ExprIr::EnvironmentIdentifier(identifier)
            if matches!(&identifier.operation, EnvironmentIdentifierOperationIr::ReleaseCapturedReference { reference } if reference == capture.reference()))
        );
        assert!(matches!(&rhs.expr, ExprIr::Identifier(name) if name == captured));
        assert_captured_put(committed_value(&selected), capture);
        assert!(selected
            .iter()
            .any(|statement| matches!(statement, StatementIr::AsyncAwait { .. })));
        assert!(statements.iter().any(|statement| matches!(
            statement,
            StatementIr::AsyncAwait {
                suspend_state: 4,
                resume_state: 5,
                ..
            }
        )));
    }
}

#[test]
fn property_get_retains_three_distinct_activation_slots_and_selected_put_consumes_them() {
    let function = function("async function choose(target, key, rhs) { 'use strict'; return target[key] ||= await rhs; }");
    let mut statements = Vec::new();
    flatten(&function.body.statements, &mut statements);
    let capture: &OrdinaryPropertyGetCaptureIr = statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Lexical {
                init:
                    TypedExpr {
                        expr: ExprIr::OrdinaryPropertyGetCapture(capture),
                        ..
                    },
                ..
            } => Some(capture),
            _ => None,
        })
        .expect("one actual Get capture before the RHS branch");
    let ExprIr::Identifier(base) = &capture.base_and_receiver().expr else {
        panic!("retained original base");
    };
    assert!(statements.iter().any(|statement| matches!(statement,
        StatementIr::Lexical { name, init: TypedExpr { expr: ExprIr::Identifier(original), .. }, .. }
            if name == base && original == "target")));
    assert!(
        matches!(capture.referenced_name(), PropertyKeyIr::StringExpr(key) if matches!(&key.expr, ExprIr::Identifier(name) if name == "key"))
    );
    let names = [
        capture.receiver_storage_name(),
        capture.target_storage_name(),
        capture.key_storage_name(),
    ];
    let slots = names.map(|name| {
        let bindings = function.owned_env_bindings.iter().filter(|binding| binding.name == name).collect::<Vec<_>>();
        assert_eq!(bindings.len(), 1);
        assert!(statements.iter().any(|statement| matches!(statement, StatementIr::Lexical { name: declared, .. } if declared == name)));
        bindings[0].slot
    });
    assert_eq!(slots.into_iter().collect::<BTreeSet<_>>().len(), 3);
    let (skipped, selected, _) = branch(&statements);
    assert_eq!(arm(skipped).len(), 1);
    let selected = arm(selected);
    let ExprIr::CapturedOrdinaryPropertyWrite(write) = &committed_value(&selected).expr else {
        panic!("selected Put consumes the normalized Reference")
    };
    assert_eq!(
        [
            write.receiver_storage_name(),
            write.target_storage_name(),
            write.key_storage_name()
        ],
        names
    );
    assert_eq!(write.strictness(), Strictness::Strict);
    assert!(matches!(&write.rhs().expr, ExprIr::Identifier(_)));
    assert!(selected
        .iter()
        .any(|statement| matches!(statement, StatementIr::AsyncAwait { .. })));
}

#[test]
fn immutable_put_follows_selected_rhs_and_tdz_get_precedes_branch_entry() {
    let constant =
        function("async function choose(rhs) { const fixed = 0; return fixed ||= await rhs; }");
    let mut statements = Vec::new();
    flatten(&constant.body.statements, &mut statements);
    let (_, _, capture) = captured_get(&statements, "fixed");
    assert!(statements.iter().any(|statement| matches!(statement, StatementIr::Lexical { mode: BindingMode::Const, name, .. } if name == "fixed")));
    let (_, selected, _) = branch(&statements);
    let selected = arm(selected);
    assert!(selected
        .iter()
        .any(|statement| matches!(statement, StatementIr::AsyncAwait { .. })));
    assert_captured_put(committed_value(&selected), capture);
    assert!(
        selected
            .iter()
            .position(|statement| matches!(statement, StatementIr::AsyncAwait { .. }))
            .unwrap()
            < selected.len() - 1
    );
    let function = function("async function choose(rhs) { later ??= await rhs; let later = 0; }");
    let mut statements = Vec::new();
    flatten(&function.body.statements, &mut statements);
    let (capture_position, _, capture) = captured_get(&statements, "later");
    let branch_position = statements
        .iter()
        .position(|statement| matches!(statement, StatementIr::AsyncFunctionIf { .. }))
        .unwrap();
    let initialization = statements.iter().position(|statement| matches!(statement, StatementIr::Lexical { mode: BindingMode::Let, name, .. } if name == "later")).expect("original lexical initialization");
    assert!(capture_position < branch_position && branch_position < initialization);
    assert!(!statements[..branch_position]
        .iter()
        .any(|statement| matches!(statement, StatementIr::AsyncAwait { .. })));
    let (selected, _, _) = branch(&statements);
    assert_captured_put(committed_value(&arm(selected)), capture);
}

#[test]
fn checked_reference_values_compose_with_invocations_branches_and_outer_await() {
    for source in [
        "async function choose(value, rhs, f) { return f(await 0, value &&= await rhs); }",
        "async function choose(value, rhs, C) { return new C(value ||= await rhs); }",
        "async function choose(value, rhs, tag) { return tag`${value ??= await rhs}`; }",
        "async function choose(value, rhs, flag) { return flag ? (value ||= await rhs) : 0; }",
        "async function choose(value, rhs) { var result = await (value ||= await rhs); return result; }",
        "async function choose(table, rhs) { return (table?.holder).value ||= await rhs; }",
        "async function choose(value, rhs) { return (await 0, async function nested(local) { local ||= await rhs; }); }",
    ] { function(source); }
}

#[test]
fn logical_assignment_suspensions_are_owned_inside_complete_loop_regions() {
    for source in [
        "async function choose(value, rhs) { while (value ||= await rhs) { break; } }",
        "async function choose(value, rhs) { while (value) { value ||= await rhs; } }",
    ] {
        let function = function(source);
        statement_awaits::assert_count(&function, 1);
    }
}

#[test]
fn complete_reference_owners_admit_awaited_left_private_super_global_and_with_shapes() {
    for source in [
        "async function choose(rhs, consume) { consume(await 0, missing ||= await rhs); }",
        "var globalCell = 0; async function choose(rhs) { globalCell ||= await rhs; }",
        "async function choose(target, rhs) { target[await 0] ||= await rhs; }",
        "async function choose(target, rhs) { (await target).value ||= await rhs; }",
        "async function* choose(value, rhs) { value ||= await rhs; }",
        "class C { #value = 0; async choose(rhs) { this.#value ||= await rhs; } }",
        "class C extends Object { async choose(rhs) { super.value ||= await rhs; } }",
        "async function choose(object, value, rhs) { with (object) { value ||= await rhs; } }",
        "async function choose(target,key){return (await target)[await key] ??= 17;}",
        "class C{#value=0;async choose(target,rhs){return (await target).#value ||= await rhs;}}",
        "class P{}class C extends P{async choose(key,rhs){return super[await key] ||= await rhs;}}",
    ] {
        function(source);
    }
}

#[test]
fn awaited_left_finishes_before_get_selection_and_does_not_reserve_eager_rhs_phases() {
    let function =
        function("async function choose(target,key){return (await target)[await key] ??= 17;}");
    let mut statements = Vec::new();
    flatten(&function.body.statements, &mut statements);
    let points = statements
        .iter()
        .filter_map(|statement| match statement {
            StatementIr::AsyncAwait {
                suspend_state,
                resume_state,
                ..
            } => Some((*suspend_state, *resume_state)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(points, [(0, 1), (1, 2)]);
    let capture = statements
        .iter()
        .position(|statement| {
            matches!(
                statement,
                StatementIr::Lexical {
                    init: TypedExpr {
                        expr: ExprIr::OrdinaryPropertyGetCapture(_),
                        ..
                    },
                    ..
                }
            )
        })
        .unwrap();
    assert!(
        statements[..capture]
            .iter()
            .filter(|statement| matches!(statement, StatementIr::AsyncAwait { .. }))
            .count()
            == 2
    );
    assert!(statements
        .iter()
        .any(|statement| matches!(statement, StatementIr::If { .. })));
    assert!(!statements
        .iter()
        .any(|statement| matches!(statement, StatementIr::AsyncFunctionIf { .. })));
}
