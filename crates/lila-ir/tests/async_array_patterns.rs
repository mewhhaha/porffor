use lila_front::{parse, ParseOptions};
use lila_ir::{lower, ArrayDestructuringOperationKindIr, ExprIr, FunctionIr, StatementIr};

fn values(source: &str) -> FunctionIr {
    let parsed = parse(source, ParseOptions::script()).expect("async pattern parses");
    let program = lower(&parsed);
    assert!(
        program.is_wasm_supported(),
        "{source}: {:?}",
        program.diagnostics
    );
    program
        .script
        .unwrap()
        .functions
        .into_iter()
        .find(|function| function.name == "values")
        .unwrap()
}

fn rows<'a>(statements: &'a [StatementIr], output: &mut Vec<&'a StatementIr>) {
    for statement in statements {
        output.push(statement);
        match statement {
            StatementIr::AsyncFunctionArrayDestructuring(plan) => {
                rows(&plan.body().statements, output)
            }
            StatementIr::LexicalBlock(body) => rows(body, output),
            StatementIr::Block(body) => rows(&body.statements, output),
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
                rows(std::slice::from_ref(then_branch), output);
                if let Some(branch) = else_branch {
                    rows(std::slice::from_ref(branch), output);
                }
            }
            StatementIr::EmptyStatementCompletion(item) => {
                rows(std::slice::from_ref(item.statement()), output)
            }
            _ => {}
        }
    }
}

#[test]
fn async_array_pattern_retains_the_original_rhs_and_actual_iterator_body_ranges() {
    let function=values("async function values(input,p,q) { var received; return ([,received=(await p,await q)]=input); }");
    let mut ordered = Vec::new();
    rows(&function.body.statements, &mut ordered);
    let plan = ordered
        .iter()
        .find_map(|statement| match statement {
            StatementIr::AsyncFunctionArrayDestructuring(plan) => Some(plan),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        (
            plan.entry_state(),
            plan.body_entry_state(),
            plan.body_end_state(),
            plan.exit_state()
        ),
        (0, 1, 6, 7)
    );
    let ExprIr::Identifier(raw) = &plan.raw_source().expr else {
        panic!("original retained RHS");
    };
    assert_ne!(raw, &plan.storage().binding().name);
    for name in [raw, &plan.storage().binding().name] {
        assert_eq!(
            function
                .owned_env_bindings
                .iter()
                .filter(|binding| &binding.name == name)
                .count(),
            1
        );
    }
    let operations = ordered
        .iter()
        .filter_map(|statement| match statement {
            StatementIr::ArrayDestructuringOperation(operation) => Some(operation.kind()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        operations,
        [
            ArrayDestructuringOperationKindIr::Elision,
            ArrayDestructuringOperationKindIr::StepValue
        ]
    );
    let awaits = ordered
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
    assert_eq!(awaits, [(2, 3), (3, 4)]);
    let body = &plan.body().statements;
    let default = body
        .iter()
        .find_map(|statement| match statement {
            StatementIr::AsyncFunctionIf { plan, .. } => Some(plan),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        (
            default.entry_state(),
            default.then_entry_state(),
            default.else_entry_state(),
            default.exit_state()
        ),
        (1, 2, 5, 6)
    );
}

#[test]
fn recursive_async_object_and_array_patterns_share_original_binding_inventory() {
    let function=values("async function values(input,p) { let [{[await 'inner']:[received=await p]}]=input; return received; }");
    let mut ordered = Vec::new();
    rows(&function.body.statements, &mut ordered);
    let plans = ordered
        .iter()
        .filter_map(|statement| match statement {
            StatementIr::AsyncFunctionArrayDestructuring(plan) => Some(plan),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(plans.len(), 2);
    assert_ne!(
        plans[0].storage().binding().slot,
        plans[1].storage().binding().slot
    );
    let mut names = std::collections::BTreeSet::new();
    for plan in plans {
        assert!(names.insert(plan.storage().binding().name.as_str()));
        assert!(function
            .owned_env_bindings
            .iter()
            .any(|row| row == plan.storage().binding()));
    }
    assert!(ordered.iter().any(
        |statement| matches!(statement,StatementIr::DeclarationEvaluation(value)
        if matches!(&value.expr,ExprIr::ObjectDestructuringOperation(_)))
    ));
}

#[test]
fn async_pattern_defaults_keep_complete_branch_prefixes_inside_the_undefined_guard() {
    let function=values("async function values(input,p,q) { const [received=(await true) ? await p : await q]=input; return received; }");
    let mut ordered = Vec::new();
    rows(&function.body.statements, &mut ordered);
    assert_eq!(
        ordered
            .iter()
            .filter(|statement| matches!(statement, StatementIr::AsyncFunctionIf { .. }))
            .count(),
        2
    );
    let awaits = ordered
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
    assert_eq!(awaits, [(2, 3), (4, 5), (6, 7)]);
    assert!(ordered
        .iter()
        .all(|statement| !matches!(statement, StatementIr::GeneratorYield { .. })));
}

#[test]
fn async_pattern_optional_tail_keeps_only_real_await_branch_ranges() {
    let function = values("async function values(input,base,key) { const [received=base?.[await key]?.()]=input; return received; }");
    let mut ordered = Vec::new();
    rows(&function.body.statements, &mut ordered);
    let plan = ordered
        .iter()
        .find_map(|statement| match statement {
            StatementIr::AsyncFunctionArrayDestructuring(plan) => Some(plan),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        (
            plan.entry_state(),
            plan.body_entry_state(),
            plan.body_end_state(),
            plan.exit_state()
        ),
        (0, 1, 8, 9)
    );
    let awaits = ordered
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
    assert_eq!(awaits, [(4, 5)]);
    assert_eq!(
        ordered
            .iter()
            .filter(|statement| matches!(statement, StatementIr::AsyncFunctionIf { .. }))
            .count(),
        2
    );
    assert!(ordered
        .iter()
        .any(|statement| matches!(statement, StatementIr::If { .. })));
}

#[test]
fn async_pattern_class_default_counts_only_its_actual_evaluation_operands() {
    let function = values("async function values(input,Base,key,nested) { const [received=class extends (await Base) { [await key]() {} async method() { return await nested; } }]=input; return received; }");
    let mut ordered = Vec::new();
    rows(&function.body.statements, &mut ordered);
    let plan = ordered
        .iter()
        .find_map(|statement| match statement {
            StatementIr::AsyncFunctionArrayDestructuring(plan) => Some(plan),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        (
            plan.entry_state(),
            plan.body_entry_state(),
            plan.body_end_state(),
            plan.exit_state()
        ),
        (0, 1, 6, 7)
    );
    let class = ordered
        .iter()
        .find_map(|statement| match statement {
            StatementIr::ResumableClassDefinition(plan) => Some(plan),
            _ => None,
        })
        .unwrap();
    assert_eq!((class.entry_state(), class.exit_state()), (2, 4));
    let awaits = class
        .prefixes()
        .flat_map(|prefix| prefix.statements())
        .filter_map(|statement| match statement {
            StatementIr::AsyncAwait {
                suspend_state,
                resume_state,
                ..
            } => Some((*suspend_state, *resume_state)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(awaits, [(2, 3), (3, 4)]);
}

#[test]
fn existing_awaited_initializer_stays_eager_and_mixed_async_generator_patterns_stay_refused() {
    let function =
        values("async function values(input) { const [received]=await input; return received; }");
    let mut ordered = Vec::new();
    rows(&function.body.statements, &mut ordered);
    assert!(ordered
        .iter()
        .any(|statement| matches!(statement, StatementIr::AsyncAwait { .. })));
    assert!(!ordered
        .iter()
        .any(|statement| matches!(statement, StatementIr::AsyncFunctionArrayDestructuring(_))));
    let parsed = parse(
        "async function* values(input) { const [received=await 1]=input; yield received; }",
        ParseOptions::script(),
    )
    .unwrap();
    let program = lower(&parsed);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let function = program
        .script
        .unwrap()
        .functions
        .into_iter()
        .find(|function| function.name == "values")
        .unwrap();
    let mut ordered = Vec::new();
    for statement in &function.body.statements {
        if let StatementIr::LexicalBlock(items) = statement {
            ordered.extend(items);
        } else {
            ordered.push(statement);
        }
    }
    assert!(ordered
        .iter()
        .any(|statement| matches!(statement, StatementIr::AsyncGeneratorArrayDestructuring(_))));
}
