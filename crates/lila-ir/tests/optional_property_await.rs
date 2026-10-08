use lila_front::{parse, ParseOptions};

#[path = "common/statement_awaits.rs"]
mod statement_awaits;
use lila_ir::{
    lower, ExprIr, FunctionIr, KindSet, OptionalChainOperationIr, PropertyKeyIr, StatementIr,
    TypedExpr,
};

fn function(source: &str) -> FunctionIr {
    let parsed = parse(source, ParseOptions::script()).expect("optional await source parses");
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

fn flatten<'a>(statements: &'a [StatementIr], result: &mut Vec<&'a StatementIr>) {
    for statement in statements {
        match statement {
            StatementIr::EmptyStatementCompletion(item) => {
                flatten(std::slice::from_ref(item.statement()), result)
            }
            StatementIr::LexicalBlock(statements) => flatten(statements, result),
            StatementIr::Block(block) => flatten(&block.statements, result),
            statement => result.push(statement),
        }
    }
}

fn branch_statements(branch: &StatementIr) -> Vec<&StatementIr> {
    let StatementIr::LexicalBlock(statements) = branch else {
        panic!("checked arm scope")
    };
    let mut result = Vec::new();
    flatten(statements, &mut result);
    result
}

#[test]
fn nullish_arm_skips_the_whole_key_prefix_and_both_arms_commit_one_owned_result() {
    let function = function("async function choose(target, key) { const value = target?.[await key]; await 0; return value; }");
    let mut statements = Vec::new();
    flatten(&function.body.statements, &mut statements);
    let bases = statements
        .iter()
        .filter_map(|statement| match statement {
            StatementIr::Lexical {
                name,
                init:
                    TypedExpr {
                        expr: ExprIr::Identifier(source),
                        ..
                    },
                ..
            } if name.starts_with("$async.optional.base.") && source == "target" => Some(name),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(bases.len(), 1, "actual base GetValue retained once");
    let (condition, skipped, selected, plan) = statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::AsyncFunctionIf {
                condition,
                then_branch,
                else_branch: Some(else_branch),
                plan,
            } => Some((condition, then_branch.as_ref(), else_branch.as_ref(), plan)),
            _ => None,
        })
        .expect("optional guard owns key states");
    assert_eq!(
        [
            plan.entry_state(),
            plan.then_entry_state(),
            plan.else_entry_state(),
            plan.exit_state()
        ],
        [0, 1, 2, 4]
    );
    let ExprIr::LogicalShortCircuit { lhs, rhs, .. } = &condition.expr else {
        panic!("strict nullish predicate")
    };
    for comparison in [lhs, rhs] {
        let ExprIr::StrictEquality { lhs, .. } = &comparison.expr else {
            panic!("pure equality")
        };
        assert!(matches!(&lhs.expr, ExprIr::Identifier(name) if name == bases[0]));
    }
    let skipped = branch_statements(skipped);
    assert_eq!(skipped.len(), 1, "no skipped Await or property read");
    let StatementIr::Expression(TypedExpr {
        expr: ExprIr::AssignIdentifier {
            name: result,
            value,
        },
        ..
    }) = skipped[0]
    else {
        panic!("skipped result write")
    };
    assert!(matches!(&value.expr, ExprIr::Undefined));
    let selected = branch_statements(selected);
    assert!(selected.iter().any(|statement| matches!(
        statement,
        StatementIr::AsyncAwait {
            suspend_state: 2,
            resume_state: 3,
            ..
        }
    )));
    let StatementIr::Expression(TypedExpr {
        expr: ExprIr::AssignIdentifier { name, .. },
        ..
    }) = selected.last().expect("selected result write")
    else {
        panic!("selected result write")
    };
    assert_eq!(name, result);
    assert!(function
        .owned_env_bindings
        .iter()
        .any(|binding| binding.name == *result));
    assert!(function
        .owned_env_bindings
        .iter()
        .any(|binding| &binding.name == bases[0]));
    let raw_key = selected
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Lexical {
                init:
                    TypedExpr {
                        expr: ExprIr::OptionalPropertyChain { chain, .. },
                        ..
                    },
                ..
            } => match chain.as_slice() {
                [OptionalChainOperationIr::Property {
                    key: PropertyKeyIr::StringExpr(key),
                    shorted: false,
                }] => Some(key),
                _ => None,
            },
            _ => None,
        })
        .expect("selected read consumes a raw key");
    assert_eq!(
        raw_key.possible_kinds,
        KindSet::all_runtime_tags(),
        "ToPropertyKey remains the emitter's transition"
    );
    assert!(statements.iter().any(|statement| matches!(
        statement,
        StatementIr::AsyncAwait {
            suspend_state: 4,
            resume_state: 5,
            ..
        }
    )));
}

#[test]
fn earlier_get_values_complete_before_later_ordinary_key_suspension() {
    let function = function("async function choose(target, first, second) { return target?.[await first].ordinary[await second]; }");
    let mut statements = Vec::new();
    flatten(&function.body.statements, &mut statements);
    let selected = statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::AsyncFunctionIf {
                else_branch: Some(branch),
                ..
            } => Some(branch.as_ref()),
            _ => None,
        })
        .expect("selected whole suffix");
    let selected = branch_statements(selected);
    let awaits = selected
        .iter()
        .enumerate()
        .filter_map(|(index, statement)| {
            matches!(statement, StatementIr::AsyncAwait { .. }).then_some(index)
        })
        .collect::<Vec<_>>();
    assert_eq!(awaits.len(), 2);
    let reads = selected
        .iter()
        .enumerate()
        .filter_map(|(index, statement)| match statement {
            StatementIr::Lexical {
                init:
                    TypedExpr {
                        expr: ExprIr::OptionalPropertyChain { .. },
                        ..
                    },
                ..
            } => Some(index),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(reads.len(), 3);
    assert!(
        awaits[0] < reads[0] && reads[0] < reads[1] && reads[1] < awaits[1] && awaits[1] < reads[2]
    );
}

#[test]
fn nested_shorting_synchronous_siblings_target_only_await_and_outer_references_compose() {
    for source in [
        "async function choose(target, key) { return target?.[await key]?.[await 'value'].tail; }",
        "async function choose(target) { return (await 1) + target?.value; }",
        "async function choose(target, call) { return call(await 1, target?.value); }",
        "async function choose(target, flag) { return flag ? await 1 : target?.(); }",
        "async function choose(target) { return (await target)?.value; }",
        "async function choose(target) { return (await target)?.(); }",
        "async function choose(target) { return (target?.[await 'holder']).method(1); }",
        "async function choose(target) { return new (target?.[await 'C'])(1); }",
        "async function choose(target) { return typeof target?.[await 'value']; }",
        "async function choose(target) { return await target?.[await 'value']; }",
        "async function choose(target) { return delete (target?.[await 'holder']).value; }",
    ] {
        function(source);
    }
}

#[test]
fn erased_nested_key_await_uses_synchronous_if_without_claiming_resume_states() {
    let function = function("async function choose(target, reject) { const value = target?.[(undefined)?.[await reject()]]; await 0; return value; }");
    let mut statements = Vec::new();
    flatten(&function.body.statements, &mut statements);
    assert!(statements
        .iter()
        .any(|statement| matches!(statement, StatementIr::If { .. })));
    assert!(!statements
        .iter()
        .any(|statement| matches!(statement, StatementIr::AsyncFunctionIf { .. })));
    assert!(statements.iter().any(|statement| matches!(
        statement,
        StatementIr::AsyncAwait {
            suspend_state: 0,
            resume_state: 1,
            ..
        }
    )));
}

#[test]
fn unsupported_reference_links_and_other_protocols_are_refused_before_await_prefix_consumption() {
    for source in [
        "async function* choose(target, key) { target?.[await key]; }",
        "async function* choose(target, key) { yield target?.[await key]; }",
    ] {
        let program = lower(&parse(source, ParseOptions::script()).unwrap());
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
        let function = program
            .script
            .unwrap()
            .functions
            .into_iter()
            .find(|function| function.name == "choose")
            .unwrap();
        assert!(function.resumable_plan.is_some());
    }
    for source in [
        "async function choose(target, key) { return delete target?.[await key]; }",
        "class C { #value = 1; async choose(target, key) { return target?.[await key].#value; } }",
        "class C extends Object { async choose(key) { return super.value?.[await key]; } }",
        "async function choose(target, key) { target[await key] &&= target?.[await key]; }",
    ] {
        function(source);
    }
    for (source, awaits) in [
        (
            "async function choose(target, key) { while (true) { target?.[await key]; break; } }",
            1,
        ),
        (
            "async function choose(target, key) { while (target?.[await key]) { await 0; } }",
            2,
        ),
    ] {
        statement_awaits::assert_count(&function(source), awaits);
    }
}

#[test]
fn guarded_delete_commits_true_on_skip_and_emits_delete_without_terminal_get() {
    let function =
        function("async function choose(target,key) { return delete target?.[await key]; }");
    let mut top = Vec::new();
    flatten(&function.body.statements, &mut top);
    let (skipped, selected) = top
        .iter()
        .find_map(|statement| match statement {
            StatementIr::AsyncFunctionIf {
                then_branch,
                else_branch: Some(other),
                ..
            } => Some((then_branch.as_ref(), other.as_ref())),
            _ => None,
        })
        .expect("Delete guard owns selected Await");
    let skipped = branch_statements(skipped);
    assert!(
        matches!(skipped.as_slice(), [StatementIr::Expression(TypedExpr { expr: ExprIr::AssignIdentifier { value, .. }, .. })] if matches!(value.expr, ExprIr::Boolean(true)))
    );
    let selected = branch_statements(selected);
    assert_eq!(
        selected
            .iter()
            .filter(|statement| matches!(statement, StatementIr::AsyncAwait { .. }))
            .count(),
        1
    );
    let StatementIr::Expression(TypedExpr {
        expr: ExprIr::AssignIdentifier { value, .. },
        ..
    }) = selected.last().unwrap()
    else {
        panic!("normal Delete result publication");
    };
    assert!(
        matches!(value.expr, ExprIr::DeleteProperty { .. }),
        "terminal original Delete, no PropertyRead: {value:?}"
    );
    assert!(
        !selected.iter().any(|statement| matches!(
            statement,
            StatementIr::Lexical {
                init: TypedExpr {
                    expr: ExprIr::OptionalPropertyChain { .. },
                    ..
                },
                ..
            }
        )),
        "no terminal Get before Delete"
    );
}
