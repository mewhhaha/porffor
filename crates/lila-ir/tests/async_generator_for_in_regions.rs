use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, AsyncGeneratorForInIr, BindingMode, ExprIr, FunctionIr, ResumableResumeEnvironmentIr,
    ResumableSuspensionKindIr, StatementIr, ValueKind,
};

fn values(source: &str) -> FunctionIr {
    let program = lower(&parse(source, ParseOptions::script()).unwrap());
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
        .find(|function| function.name == "values" || function.name.ends_with(".values"))
        .unwrap()
}

fn walk<'a>(items: &'a [StatementIr], output: &mut Vec<&'a StatementIr>) {
    for item in items {
        output.push(item);
        match item {
            StatementIr::AsyncGeneratorForIn(plan) => {
                walk(&plan.head().region().block().statements, output);
                walk(&plan.initialization().statements, output);
                walk(&plan.body().block().statements, output);
            }
            StatementIr::AsyncGeneratorForOf(plan) => {
                walk(&plan.head().region().block().statements, output);
                walk(&plan.initialization().block().statements, output);
                walk(&plan.body().block().statements, output);
            }
            StatementIr::AsyncGeneratorResourceScope(plan) => {
                walk(&plan.body().block().statements, output)
            }
            StatementIr::AsyncGeneratorLoop(plan) => {
                for region in plan.regions() {
                    walk(&region.block().statements, output);
                }
            }
            StatementIr::AsyncGeneratorIf(plan) => {
                for region in plan.regions() {
                    walk(&region.block().statements, output);
                }
            }
            StatementIr::AsyncGeneratorSwitch(plan) => {
                for region in plan.regions() {
                    walk(&region.block().statements, output);
                }
            }
            StatementIr::AsyncGeneratorWith(plan) => {
                walk(&plan.head().region().block().statements, output);
                walk(&plan.body().block().statements, output);
            }
            StatementIr::EmptyStatementCompletion(item) => {
                walk(std::slice::from_ref(item.statement()), output)
            }
            StatementIr::Block(block) => walk(&block.statements, output),
            StatementIr::LexicalBlock(items) => walk(items, output),
            StatementIr::Labelled { statement, .. } => {
                walk(std::slice::from_ref(statement.as_ref()), output)
            }
            StatementIr::TryCatch {
                try_block,
                catch_block,
                ..
            } => {
                walk(&try_block.statements, output);
                walk(&catch_block.statements, output);
            }
            StatementIr::TryFinally {
                try_block,
                finally_block,
                ..
            } => {
                walk(&try_block.statements, output);
                walk(&finally_block.statements, output);
            }
            StatementIr::TryCatchFinally {
                try_block,
                catch_block,
                finally_block,
                ..
            } => {
                walk(&try_block.statements, output);
                walk(&catch_block.statements, output);
                walk(&finally_block.statements, output);
            }
            _ => {}
        }
    }
}

#[test]
fn mixed_eager_super_enumeration_head_consumes_the_original_selected_string_key() {
    let function = values("class P{}class C extends P{async* values(input){for(super.chosen in await(yield input)){yield this;}}}");
    let mut items = Vec::new();
    walk(&function.body.statements, &mut items);
    let plan = items
        .iter()
        .find_map(|item| match item {
            StatementIr::AsyncGeneratorForIn(plan) => Some(plan),
            _ => None,
        })
        .unwrap();
    let [StatementIr::DeclarationEvaluation(value)] = plan.initialization().statements.as_slice()
    else {
        panic!("one original eager Super Put");
    };
    let ExprIr::SuperPropertyWrite { value, .. } = &value.expr else {
        panic!("original Super write");
    };
    assert!(matches!(&value.expr, ExprIr::Identifier(name) if name == &plan.key_binding().name));
}

fn first(function: &FunctionIr) -> &AsyncGeneratorForInIr {
    let mut items = Vec::new();
    walk(&function.body.statements, &mut items);
    items
        .into_iter()
        .find_map(|item| match item {
            StatementIr::AsyncGeneratorForIn(plan) => Some(plan.as_ref()),
            _ => None,
        })
        .expect("actual checked mixed enumeration")
}

#[test]
fn mixed_for_in_has_disjoint_head_advance_body_and_exact_protocol_tape() {
    let function =
        values("async function* values(p){for(let key in await (yield p)){yield key;await p;}}");
    let plan = first(&function);
    assert_eq!(
        (
            plan.entry_state(),
            plan.head().region().end_state(),
            plan.advance_state(),
            plan.initialization_region().entry_state(),
            plan.initialization_region().end_state(),
            plan.body().entry_state(),
            plan.body().end_state(),
            plan.exit_state()
        ),
        (0, 2, 3, 4, 4, 5, 7, 8)
    );
    assert_eq!(plan.head_mode(), BindingMode::Let);
    let cells = [
        plan.head_binding(),
        plan.enumerator_binding(),
        plan.key_binding(),
        plan.value_binding(),
    ];
    for (index, cell) in cells.iter().enumerate() {
        assert!(function.owned_env_bindings.contains(cell));
        for other in &cells[..index] {
            assert_ne!(cell.slot, other.slot);
            assert_ne!(cell.name, other.name);
        }
    }
    let [StatementIr::Lexical { init: key, .. }] = plan.initialization().statements.as_slice()
    else {
        panic!("actual original Let key initialization");
    };
    assert_eq!(key.kind, ValueKind::String);
    assert_eq!(
        key.possible_kinds,
        lila_ir::KindSet::from_kind(ValueKind::String)
    );
    let resumable = function.resumable_plan.as_ref().unwrap();
    assert_eq!(resumable.state_count, 9);
    assert_eq!(
        resumable
            .suspension_points
            .iter()
            .map(|point| (point.kind, point.suspend_state, point.resume_state))
            .collect::<Vec<_>>(),
        [
            (ResumableSuspensionKindIr::Yield, 0, 1),
            (ResumableSuspensionKindIr::Await, 1, 2),
            (ResumableSuspensionKindIr::Yield, 5, 6),
            (ResumableSuspensionKindIr::Await, 6, 7),
        ]
    );
    assert!(resumable
        .suspension_points
        .iter()
        .all(|point| point.resume_environment == ResumableResumeEnvironmentIr::InvocationOuter));
    assert!(plan.head().region().block().lexical_environment.is_none());
    assert!(plan.initialization().lexical_environment.is_none());
    assert!(plan.body().block().lexical_environment.is_none());
    assert!(
        matches!(&plan.head().value().expr, ExprIr::Identifier(name) if name == &plan.head_binding().name)
    );
    assert!(
        matches!(plan.head().region().block().statements.last(), Some(StatementIr::Lexical { name, .. }) if name == &plan.head_binding().name)
    );
}

#[test]
fn mixed_for_in_retains_original_scoped_key_storage_and_whole_empty_prefix() {
    let function = values("async function* values(items,p){for(const key in await (yield items)){let read=()=>key;17;var saved=await (yield p);yield read();}return p;}");
    let plan = first(&function);
    let [StatementIr::Lexical {
        mode: BindingMode::Const,
        name,
        init,
    }] = plan.initialization().statements.as_slice()
    else {
        panic!("actual original key initialization");
    };
    assert!(matches!(&init.expr, ExprIr::Identifier(key) if key == &plan.key_binding().name));
    assert_ne!(name, &plan.key_binding().name);
    let mut items = Vec::new();
    walk(&plan.body().block().statements, &mut items);
    assert!(items.iter().any(|item| match item {
        StatementIr::EmptyStatementCompletion(item) => match item.statement() {
            StatementIr::LexicalBlock(prefix) =>
                prefix
                    .iter()
                    .any(|item| matches!(item, StatementIr::GeneratorYield { .. }))
                    && prefix
                        .iter()
                        .any(|item| matches!(item, StatementIr::AsyncAwait { .. })),
            _ => false,
        },
        _ => false,
    }));
    let tail = function
        .resumable_plan
        .as_ref()
        .unwrap()
        .suspension_points
        .last()
        .unwrap();
    assert_eq!(tail.kind, ResumableSuspensionKindIr::Await);
    assert_eq!(tail.suspend_state, plan.exit_state());
    assert_eq!(
        tail.resume_environment,
        ResumableResumeEnvironmentIr::SavedLexicalChain
    );
}

#[test]
fn mixed_for_in_nests_whole_enumeration_with_switch_try_and_classic_owners() {
    let function = values("async function* values(items,scope){outer:for(const key in await (yield items)){with(await scope){switch(yield key){case 'one':for(var nested in await items){try{while(yield nested){await 0;continue outer;}}finally{await 0;yield nested;}}break;default:yield key;}}}}");
    let mut items = Vec::new();
    walk(&function.body.statements, &mut items);
    let plans: Vec<_> = items
        .iter()
        .filter_map(|item| match item {
            StatementIr::AsyncGeneratorForIn(plan) => Some(plan),
            _ => None,
        })
        .collect();
    assert_eq!(plans.len(), 2);
    assert_ne!(
        plans[0].enumerator_binding().slot,
        plans[1].enumerator_binding().slot
    );
    assert!(items
        .iter()
        .any(|item| matches!(item, StatementIr::AsyncGeneratorWith(_))));
    assert!(items
        .iter()
        .any(|item| matches!(item, StatementIr::AsyncGeneratorSwitch(_))));
    assert!(items
        .iter()
        .any(|item| matches!(item, StatementIr::AsyncGeneratorLoop(_))));
    assert!(items.iter().any(|item| matches!(
        item,
        StatementIr::TryFinally {
            generator_plan: Some(_),
            async_plan: Some(_),
            ..
        }
    )));
}

#[test]
fn mixed_for_in_consumes_original_eager_binding_pattern_property_and_identifier_heads() {
    for source in [
        "async function* values(items){for(var key in await (yield items)){yield key;}}",
        "async function* values(items){for(var key=0 in await (yield items)){yield key;}}",
        "async function* values(items){for(const {length:size} in await (yield items)){yield size;}}",
        "async function* values(items){let key;for(key in await (yield items)){yield key;}}",
        "async function* values(items,holder){for(holder.key in await (yield items)){yield holder.key;}}",
        "async function* values(items,holder){for({length:holder.size} in await (yield items)){yield holder.size;}}",
    ] {
        let function = values(source); let plan = first(&function);
        assert!(!plan.initialization().statements.is_empty());
        assert!(function.owned_env_bindings.contains(plan.key_binding()));
        assert_eq!(plan.head().region().end_state(), 2);
    }
}

#[test]
fn mixed_for_in_keeps_suspended_heads_foreign_protocols_and_resource_suffixes_bounded() {
    for source in [
        "async function* values(items){for(const key in yield items){for await(const value of items){yield value;}}}",
        "async function* values(items){for(const key in yield items){for(const value of items){yield value;}}}",
        "async function* values(items){for(const value of items){for(const key in yield items){yield key;}}}",
        "async function* values(items){for(const key in yield items){using resource=null;yield key;}}",
    ] { let function=values(source); let mut items=Vec::new(); walk(&function.body.statements,&mut items); assert!(items.iter().any(|item| matches!(item,StatementIr::AsyncGeneratorForIn(_)))); assert!(items.iter().any(|item| matches!(item,StatementIr::AsyncGeneratorForOf(_)|StatementIr::AsyncGeneratorResourceScope(_)))); }
    for source in [
        "async function* values(items){for(holder[yield 'key'] in items){yield 1;}}",
        "async function* values(items,p){for({length:holder.size=await p} in items){yield 1;}}",
    ] {
        let program = lower(&parse(source, ParseOptions::script()).unwrap());
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
    }
}

#[test]
fn mixed_for_in_initializer_retains_selected_key_and_original_pattern_record() {
    let function = values("async function* values(items){for(const {absent:received=await(yield ()=>received)} in items){yield ()=>received;}}");
    let plan = first(&function);
    assert_eq!(
        plan.advance_state() + 1,
        plan.initialization_region().entry_state()
    );
    assert_eq!(
        plan.initialization_region().end_state() + 1,
        plan.body().entry_state()
    );
    assert!(plan.initialization_region().end_state() > plan.initialization_region().entry_state());
    let environment = plan
        .lexical_environment()
        .unwrap()
        .iteration_environment
        .as_ref()
        .unwrap();
    let mut items = Vec::new();
    walk(&plan.initialization().statements, &mut items);
    let writes = items
        .iter()
        .filter_map(|item| match item {
            StatementIr::DeclarationEvaluation(value) => match &value.expr {
                ExprIr::ObjectDestructuringOperation(operation) => Some(operation),
                _ => None,
            },
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(writes.iter().any(|operation| {
        let mut found = false;
        operation.visit_bindings(&mut |_, name| {
            found |= environment
                .bindings
                .iter()
                .any(|binding| binding.name == name)
        });
        found
    }));
    let resumable = function.resumable_plan.as_ref().unwrap();
    assert!(resumable
        .suspension_points
        .iter()
        .filter(
            |point| point.suspend_state >= plan.initialization_region().entry_state()
                && point.resume_state <= plan.initialization_region().end_state()
        )
        .all(|point| {
            point.resume_environment == ResumableResumeEnvironmentIr::InvocationOuter
                && resumable
                    .resume_environment_plan()
                    .enclosing_scope_resume_states()
                    .contains(&point.resume_state)
        }));
}

#[test]
fn mixed_for_in_array_default_and_super_key_own_real_initializer_continuations() {
    let array = values("async function* values(items){for(let [received=await(yield 'default')] in items){yield ()=>received;}}");
    let plan = first(&array);
    let mut items = Vec::new();
    walk(&plan.initialization().statements, &mut items);
    assert!(items
        .iter()
        .any(|item| matches!(item, StatementIr::AsyncGeneratorArrayDestructuring(_))));
    let function = values("class P{}class C extends P{async* values(items){for(super[await(yield 'key')] in items){yield this;}}}");
    let plan = first(&function);
    assert!(plan.initialization_region().end_state() > plan.initialization_region().entry_state());
    assert!(plan.initialization().statements.iter().any(|item| matches!(item,
        StatementIr::Expression(value) if matches!(&value.expr, ExprIr::SuperPropertyMutation(mutation)
            if matches!(mutation.operation(), lila_ir::SuperPropertyMutationOperationIr::PutCaptured { value, .. }
                if matches!(&value.expr, ExprIr::Identifier(name) if name == &plan.key_binding().name))))));
}

#[test]
fn eager_mixed_for_in_still_owns_complete_phases_with_no_source_suspension() {
    let function = values("async function* values(items){if(true){for(var key in items){key;}}}");
    let plan = first(&function);
    assert_eq!(
        plan.initialization_region().entry_state(),
        plan.advance_state() + 1
    );
    assert_eq!(
        plan.body().entry_state(),
        plan.initialization_region().end_state() + 1
    );
    assert_eq!(plan.exit_state(), plan.body().end_state() + 1);
    assert!(function
        .resumable_plan
        .as_ref()
        .unwrap()
        .suspension_points
        .is_empty());
    assert!(!plan.initialization().statements.is_empty());
}

#[test]
fn mixed_annex_b_var_initializer_precedes_the_complete_enumeration_target() {
    let function = values("async function* values(input){for(var key=await(yield 'initializer') in await(yield input)){yield key;}}");
    let plan = first(&function);
    let tape = &function.resumable_plan.as_ref().unwrap().suspension_points;
    assert_eq!(
        tape.iter()
            .take(4)
            .map(|point| point.kind)
            .collect::<Vec<_>>(),
        [
            ResumableSuspensionKindIr::Yield,
            ResumableSuspensionKindIr::Await,
            ResumableSuspensionKindIr::Yield,
            ResumableSuspensionKindIr::Await,
        ]
    );
    assert!(tape[3].resume_state <= plan.head().region().end_state());
    assert!(tape
        .iter()
        .take(4)
        .all(|point| point.resume_environment == ResumableResumeEnvironmentIr::InvocationOuter));
    assert!(plan.head().region().block().statements.iter().any(|statement|
        matches!(statement, StatementIr::Var(declarations) if declarations.iter().any(|declaration| declaration.init.is_none()))));
    assert!(plan
        .head()
        .region()
        .block()
        .statements
        .iter()
        .any(|statement| matches!(statement, StatementIr::DeclarationEvaluation(_))));
    assert!(
        matches!(plan.head().region().block().statements.last(), Some(StatementIr::Lexical { name, .. }) if name == &plan.head_binding().name)
    );
    assert!(parse("'use strict';async function* values(input){for(var key=await(yield 'initializer') in input){yield key;}}", ParseOptions::script()).is_err());
}
