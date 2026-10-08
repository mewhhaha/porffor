use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, EnvironmentIdentifierOperationIr, ExprIr, FunctionIr, IdentifierReferenceCaptureAccess,
    IdentifierReferenceCaptureDisposition, StatementIr,
};

fn values(source: &str) -> FunctionIr {
    let parsed = parse(source, ParseOptions::script()).unwrap();
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

fn operations<'a>(
    value: &'a lila_ir::TypedExpr,
    output: &mut Vec<&'a EnvironmentIdentifierOperationIr>,
) {
    match &value.expr {
        ExprIr::EnvironmentIdentifier(identifier) => output.push(&identifier.operation),
        ExprIr::AssignIdentifier { value, .. } => operations(value, output),
        ExprIr::Comma { lhs, rhs } => {
            operations(lhs, output);
            operations(rhs, output);
        }
        _ => {}
    }
}

fn rows<'a>(statements: &'a [StatementIr], output: &mut Vec<&'a StatementIr>) {
    for statement in statements {
        output.push(statement);
        match statement {
            StatementIr::AsyncFunctionWith(plan) => {
                rows(&plan.head().statements, output);
                rows(&plan.body().statements, output);
            }
            StatementIr::AsyncFunctionArrayDestructuring(plan) => {
                rows(&plan.body().statements, output)
            }
            StatementIr::AsyncGeneratorLoop(plan) => {
                for region in plan.regions() {
                    rows(&region.block().statements, output);
                }
            }
            StatementIr::AsyncGeneratorSwitch(plan) => {
                for region in plan.regions() {
                    rows(&region.block().statements, output);
                }
            }
            StatementIr::AsyncGeneratorForIn(plan) => {
                rows(&plan.head().region().block().statements, output);
                rows(&plan.initialization_region().block().statements, output);
                rows(&plan.body().block().statements, output);
            }
            StatementIr::Block(block) => rows(&block.statements, output),
            StatementIr::LexicalBlock(body) => rows(body, output),
            StatementIr::If {
                then_branch,
                else_branch,
                ..
            }
            | StatementIr::AsyncFunctionIf {
                then_branch,
                else_branch,
                ..
            } => {
                rows(std::slice::from_ref(then_branch), output);
                if let Some(branch) = else_branch {
                    rows(std::slice::from_ref(branch), output);
                }
            }
            StatementIr::AsyncFunctionSwitch(plan) => {
                for case in plan.cases() {
                    rows(case.condition_prefix(), output);
                }
                rows(plan.lexical_declarations(), output);
                for case in plan.cases() {
                    rows(&case.body().statements, output);
                }
            }
            StatementIr::TryCatch {
                try_block,
                catch_block,
                ..
            } => {
                rows(&try_block.statements, output);
                rows(&catch_block.statements, output);
            }
            StatementIr::TryFinally {
                try_block,
                finally_block,
                ..
            } => {
                rows(&try_block.statements, output);
                rows(&finally_block.statements, output);
            }
            StatementIr::TryCatchFinally {
                try_block,
                catch_block,
                finally_block,
                ..
            } => {
                rows(&try_block.statements, output);
                rows(&catch_block.statements, output);
                rows(&finally_block.statements, output);
            }
            StatementIr::Labelled { statement, .. } => {
                rows(std::slice::from_ref(statement), output)
            }
            StatementIr::EmptyStatementCompletion(item) => {
                rows(std::slice::from_ref(item.statement()), output)
            }
            _ => {}
        }
    }
}

#[test]
fn async_with_completes_head_before_original_object_environment_and_retains_exact_cells() {
    let function = values(
        "async function values(view,p){with(await view){var read=()=>value;await p;return read;}}",
    );
    let mut ordered = Vec::new();
    rows(&function.body.statements, &mut ordered);
    let plan = ordered
        .iter()
        .find_map(|row| match row {
            StatementIr::AsyncFunctionWith(plan) => Some(plan),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        (
            plan.entry_state(),
            plan.head_ready_state(),
            plan.body_entry_state(),
            plan.body_end_state(),
            plan.exit_state()
        ),
        (0, 1, 2, 3, 4)
    );
    let ExprIr::Identifier(head) = &plan.head_value().expr else {
        panic!("retained boxed head");
    };
    assert_eq!(head, &plan.head_binding().name);
    assert!(function
        .owned_env_bindings
        .iter()
        .any(|row| row == plan.head_binding()));
    assert_eq!(
        plan.lexical_environment().bindings.as_slice(),
        std::slice::from_ref(plan.object_binding())
    );
    assert_ne!(head, &plan.object_binding().name);
    let awaits = ordered
        .iter()
        .filter_map(|row| match row {
            StatementIr::AsyncAwait {
                suspend_state,
                resume_state,
                ..
            } => Some((*suspend_state, *resume_state)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(awaits, [(0, 1), (2, 3)]);
}

#[test]
fn async_with_source_tape_covers_actual_branches_array_close_and_finalizer_ranges() {
    let function = values("async function values(view,input,p){with(view){try{if(await true){const [received=await p]=input;}else{await p;}switch(await 1){case await 1:await p;break;default:await p;}}finally{await p;}}}");
    let mut ordered = Vec::new();
    rows(&function.body.statements, &mut ordered);
    assert!(ordered
        .iter()
        .any(|row| matches!(row, StatementIr::AsyncFunctionArrayDestructuring(_))));
    assert!(ordered
        .iter()
        .any(|row| matches!(row, StatementIr::AsyncGeneratorSwitch(plan)
            if plan.execution() == lila_ir::ResumableRegionProtocolIr::Async)));
    assert!(ordered
        .iter()
        .any(|row| matches!(row, StatementIr::AsyncFunctionIf { .. })));
    let awaits = ordered
        .iter()
        .filter_map(|row| match row {
            StatementIr::AsyncAwait {
                suspend_state,
                resume_state,
                ..
            } => Some((*suspend_state, *resume_state)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(awaits.len(), 8);
    assert!(awaits.windows(2).all(|pair| pair[0].1 <= pair[1].0));
}

#[test]
fn awaited_with_identifier_reference_is_captured_before_rhs_and_put_uses_same_record() {
    let function = values(
        "async function values(view,p){with(view){value=await p;value+=await p;value??=await p;}}",
    );
    let mut ordered = Vec::new();
    rows(&function.body.statements, &mut ordered);
    let mut captures = Vec::new();
    for (index, row) in ordered.iter().enumerate() {
        let value = match row {
            StatementIr::Expression(value) | StatementIr::Lexical { init: value, .. } => value,
            _ => continue,
        };
        let ExprIr::EnvironmentIdentifier(identifier) = &value.expr else {
            continue;
        };
        let EnvironmentIdentifierOperationIr::CaptureAssignmentReference { capture } =
            &identifier.operation
        else {
            continue;
        };
        assert!(matches!(
            capture.disposition(),
            IdentifierReferenceCaptureDisposition::WithObject { .. }
        ));
        assert!(ordered
            .iter()
            .skip(index + 1)
            .any(|row| matches!(row, StatementIr::AsyncAwait { .. })));
        assert!(function
            .owned_env_bindings
            .iter()
            .any(|row| row.name == capture.reference().storage_name()));
        captures.push(capture);
    }
    assert_eq!(
        captures
            .iter()
            .map(|capture| capture.access())
            .collect::<Vec<_>>(),
        [
            IdentifierReferenceCaptureAccess::WriteOnly,
            IdentifierReferenceCaptureAccess::ReadBeforeRhs,
            IdentifierReferenceCaptureAccess::ReadBeforeRhs
        ]
    );
    let names = captures
        .iter()
        .map(|capture| capture.reference().storage_name())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(names.len(), 3);
    let mut writes = Vec::new();
    for row in &ordered {
        if let StatementIr::Expression(value) | StatementIr::DeclarationEvaluation(value) = row {
            operations(value, &mut writes);
        }
    }
    let put_names = writes
        .iter()
        .filter_map(|operation| match operation {
            EnvironmentIdentifierOperationIr::PutCapturedReference { reference, .. } => {
                Some(reference.storage_name())
            }
            _ => None,
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        put_names, names,
        "every original selected Reference supplies its actual Put"
    );
    assert!(writes.iter().any(|operation| matches!(operation,EnvironmentIdentifierOperationIr::ReleaseCapturedReference { reference } if reference.storage_name()==captures[2].reference().storage_name())), "logical skipped arm retires the exact original Reference");
}

#[test]
fn eager_with_phases_remain_owned_through_enclosing_if_switch_try_and_label() {
    let function = values("async function values(view,flag){if(true){with(view){value;}}switch(flag){case 0:with(view){value;}break;default:break;}label:{try{with(view){break label;}}finally{value;}}}");
    let mut ordered = Vec::new();
    rows(&function.body.statements, &mut ordered);
    let plans = ordered
        .iter()
        .filter_map(|row| match row {
            StatementIr::AsyncFunctionWith(plan) => Some(plan),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(plans.len(), 3);
    assert!(plans
        .iter()
        .all(
            |plan| plan.head_ready_state().checked_add(1) == Some(plan.body_entry_state())
                && plan.body_end_state().checked_add(1) == Some(plan.exit_state())
        ));
    assert!(!ordered
        .iter()
        .any(|row| matches!(row, StatementIr::AsyncAwait { .. })));
    assert!(ordered
        .iter()
        .any(|row| matches!(row, StatementIr::AsyncFunctionIf { .. })));
    assert!(ordered
        .iter()
        .any(|row| matches!(row, StatementIr::AsyncFunctionSwitch(_))));
}

#[test]
fn async_with_skips_nested_callable_awaits_and_retains_known_nullish_tail_owner() {
    let function = values("async function values(view,p){with((null?.[await p],view)){var read=async()=>await p;var skipped=null?.[await p]();return read;}}");
    let mut ordered = Vec::new();
    rows(&function.body.statements, &mut ordered);
    assert_eq!(
        ordered
            .iter()
            .filter(|row| matches!(row, StatementIr::AsyncAwait { .. }))
            .count(),
        2
    );
    assert!(ordered
        .iter()
        .any(|row| matches!(row, StatementIr::AsyncFunctionIf { .. })));
}

#[test]
fn async_with_preserves_eager_foreign_loop_route_and_explicit_mixed_scope_refusals() {
    let function = values("async function values(view,p){while(await p){with(view){value;}}}");
    let mut ordered = Vec::new();
    rows(&function.body.statements, &mut ordered);
    assert!(ordered
        .iter()
        .any(|row| matches!(row, StatementIr::AsyncFunctionWith(_))));
    assert!(ordered
        .iter()
        .any(|row| matches!(row, StatementIr::AsyncGeneratorLoop(plan)
        if plan.execution() == lila_ir::ResumableRegionProtocolIr::Async)));
    let for_in =
        values("async function values(view){for(var key in view){with(view){key;}}return 1;}");
    assert!(for_in
        .body
        .statements
        .iter()
        .any(|row| matches!(row, StatementIr::AsyncGeneratorForIn(plan)
            if plan.execution() == lila_ir::ResumableRegionProtocolIr::Async)));
    for source in [
        "async function* values(view){with(view){yield ()=>value;}}",
        "async function values(view,p){for(const item of [1]){with(view){await p;var read=()=>value;}}}",
        "async function values(view,p){with(view){do{await p;}while(false);}}",
    ] {
        let program = lower(&parse(source, ParseOptions::script()).unwrap());
        assert!(program.is_wasm_supported(), "{source}: {:?}", program.diagnostics);
    }
    assert!(parse(
        "'use strict';async function values(view){with(view){await 0;}}",
        ParseOptions::script()
    )
    .is_err());
}
