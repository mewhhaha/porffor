use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, AsyncGeneratorResourceCapabilityIr, AsyncGeneratorSwitchIr, FunctionIr,
    ResumableResumeEnvironmentIr, ResumableSuspensionKindIr, StatementIr,
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
        .find(|function| function.name == "values")
        .unwrap()
}

fn walk<'a>(items: &'a [StatementIr], output: &mut Vec<&'a StatementIr>) {
    for item in items {
        output.push(item);
        match item {
            StatementIr::AsyncGeneratorSwitch(plan) => {
                for region in plan.regions() {
                    walk(&region.block().statements, output);
                }
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
            StatementIr::AsyncGeneratorWith(plan) => {
                walk(&plan.head().region().block().statements, output);
                walk(&plan.body().block().statements, output);
            }
            StatementIr::AsyncGeneratorForIn(plan) => {
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

fn first_switch(function: &FunctionIr) -> &AsyncGeneratorSwitchIr {
    let mut items = Vec::new();
    walk(&function.body.statements, &mut items);
    items
        .into_iter()
        .find_map(|item| match item {
            StatementIr::AsyncGeneratorSwitch(plan) => Some(plan.as_ref()),
            _ => None,
        })
        .expect("actual complete mixed Switch")
}

#[test]
fn mixed_switch_has_disjoint_discriminant_selector_fallback_and_fallthrough_ranges() {
    let function = values("async function* values(p){switch(await (yield p)){case await p:yield 'one';break;default:await p;yield 'default';case yield 'last':yield 'last-body';}}");
    let plan = first_switch(&function);
    assert_eq!(
        (
            plan.entry_state(),
            plan.discriminant().region().end_state(),
            plan.case_block_entry_state(),
            plan.fallback_state(),
            plan.exit_state()
        ),
        (0, 2, 3, 7, 15)
    );
    assert_eq!(
        plan.cases()
            .iter()
            .map(|case| (
                case.selector().map(|selector| (
                    selector.region().entry_state(),
                    selector.region().end_state()
                )),
                (case.body().entry_state(), case.body().end_state())
            ))
            .collect::<Vec<_>>(),
        [
            (Some((3, 4)), (8, 9)),
            (None, (10, 12)),
            (Some((5, 6)), (13, 14))
        ]
    );
    assert!(function
        .owned_env_bindings
        .contains(plan.discriminant_binding()));
    assert!(function.owned_env_bindings.contains(plan.value_binding()));
    assert_ne!(plan.discriminant_binding().slot, plan.value_binding().slot);
    assert!(plan
        .regions()
        .all(|region| region.block().lexical_environment.is_none()));
    let source = function.resumable_plan.as_ref().unwrap();
    assert_eq!(source.state_count, 16);
    assert_eq!(
        source
            .suspension_points
            .iter()
            .map(|point| (point.kind, point.suspend_state, point.resume_state))
            .collect::<Vec<_>>(),
        [
            (ResumableSuspensionKindIr::Yield, 0, 1),
            (ResumableSuspensionKindIr::Await, 1, 2),
            (ResumableSuspensionKindIr::Await, 3, 4),
            (ResumableSuspensionKindIr::Yield, 5, 6),
            (ResumableSuspensionKindIr::Yield, 8, 9),
            (ResumableSuspensionKindIr::Await, 10, 11),
            (ResumableSuspensionKindIr::Yield, 11, 12),
            (ResumableSuspensionKindIr::Yield, 13, 14),
        ]
    );
    assert!(source
        .suspension_points
        .iter()
        .all(|point| point.resume_environment == ResumableResumeEnvironmentIr::InvocationOuter));
    assert_eq!(
        source
            .resume_environment_plan()
            .enclosing_scope_resume_states(),
        [4, 6, 9, 11, 12, 14]
    );
}

#[test]
fn mixed_case_block_uses_original_captured_cells_and_whole_empty_declaration_prefixes() {
    let function = values("async function* values(p){switch(await p){case 0:let shared=await (yield p);const read=()=>shared;{19;var saved=await (yield read);if(p)var branch=await p;try{let local=await p;yield read;}finally{await p;}}default:yield hoisted();function hoisted(){return shared;}}}");
    let plan = first_switch(&function);
    assert!(plan.lexical_environment().is_some());
    assert_eq!(
        plan.lexical_declarations().len(),
        1,
        "actual CaseBlock function initialized once before selectors"
    );
    assert!(plan
        .cases()
        .iter()
        .all(|case| case.body().block().lexical_environment.is_none()));
    let mut body = Vec::new();
    for case in plan.cases() {
        walk(&case.body().block().statements, &mut body);
    }
    let empty = body
        .iter()
        .filter_map(|item| match item {
            StatementIr::EmptyStatementCompletion(item) => Some(item.statement()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        empty.len(),
        5,
        "shared let/const and nested var/branch var/let are the five actual Empty source items"
    );
    assert!(
        empty
            .iter()
            .any(|item| matches!(item, StatementIr::LexicalBlock(_))),
        "whole suspended initializer prefix remains inside its original Empty source item"
    );
    assert!(body.iter().any(|item| matches!(
        item,
        StatementIr::TryFinally {
            generator_plan: Some(_),
            async_plan: Some(_),
            ..
        }
    )));
    assert!(
        function
            .resumable_plan
            .as_ref()
            .unwrap()
            .resume_environment_plan()
            .enclosing_scope_resume_states()
            .len()
            > 4
    );
}

#[test]
fn mixed_switch_recurses_through_real_if_loop_with_and_nested_case_owners() {
    let function = values("async function* values(scope,p){with(scope){if(p){for(let i=0;i<1;i++){switch(await (yield p)){case 1:switch(yield p){case 2:await p;yield value;break;}break;default:yield value;}}}}}");
    let mut body = Vec::new();
    walk(&function.body.statements, &mut body);
    assert_eq!(
        body.iter()
            .filter(|item| matches!(item, StatementIr::AsyncGeneratorSwitch(_)))
            .count(),
        2
    );
    assert!(body
        .iter()
        .any(|item| matches!(item, StatementIr::AsyncGeneratorWith(_))));
    assert!(body
        .iter()
        .any(|item| matches!(item, StatementIr::AsyncGeneratorLoop(_))));
    assert!(body
        .iter()
        .any(|item| matches!(item, StatementIr::AsyncGeneratorIf(_))));
    let source = function.resumable_plan.as_ref().unwrap();
    assert!(source
        .suspension_points
        .windows(2)
        .all(|pair| pair[0].resume_state <= pair[1].suspend_state));
}

#[test]
fn mixed_switch_retains_iterator_resource_and_pattern_owners() {
    for source in [
        "async function* values(items){switch(yield 0){case 0:for await(const item of items){yield item;}}}",
        "async function* values(items){switch(yield 0){case 0:for(const item of items){yield item;}}}",
        "async function* values(p){switch(yield 0){case 0:const [item=await p]=[];yield item;}}",
        "async function* values(items){for(const item of items){switch(yield item){case 0:yield 1;}}}",
    ] { let function=values(source);assert!(function.resumable_plan.as_ref().unwrap().suspension_points.iter().any(|point| point.kind==ResumableSuspensionKindIr::Yield)); }
    for source in [
        "async function* values(){switch(yield 0){case 0:using resource=null;yield 1;}}",
        "async function* values(){switch(yield 0){case 0:await using resource=null;yield 1;}}",
    ] {
        let error = parse(source, ParseOptions::script())
            .expect_err("a resource declaration requires an owning lexical block inside a case");
        let lila_front::ParseCode::Early(code) = error.diagnostic().code else {
            panic!("direct CaseBlock resource declarations remain early errors");
        };
        assert_eq!(
            code.code(),
            lila_front::EarlyErrorCode::SwitchClauseUsingDeclaration
        );
    }
    for (source, asynchronous) in [
        ("async function* values(){switch(yield 0){case 0:{using resource=null;yield 1;}}}", false),
        ("async function* values(){switch(yield 0){case 0:{await using resource=null;yield 1;}}}", true),
    ] {
        let function = values(source);
        let plan = first_switch(&function);
        assert!(plan.resource().is_none(), "the case's lexical block owns disposal");
        let case = &plan.cases()[0];
        let [StatementIr::Block(block)] = case.body().block().statements.as_slice() else {
            panic!("original owning lexical block");
        };
        let [StatementIr::AsyncGeneratorResourceScope(resource)] = block.statements.as_slice() else {
            panic!("one complete resource owner inside the case");
        };
        assert_eq!(resource.capacity(), 1);
        assert_eq!(resource.entry_state(), case.body().entry_state());
        assert_eq!(resource.exit_state(), case.body().end_state());
        assert_eq!(matches!(resource.capability(), AsyncGeneratorResourceCapabilityIr::Async(_)), asynchronous);
        assert_eq!(function.owned_env_bindings.iter()
            .filter(|binding| *binding == resource.capability_binding()).count(), 1);
    }
    let eager = values("async function* values(items){switch(0){case 0:for(const item of items){item;}for(var key in items){key;}break;}yield 1;}");
    assert!(
        first_switch(&eager).exit_state() > 0,
        "phase-free original eager iterator children stay admitted"
    );
}

#[test]
fn mixed_switch_and_for_in_keep_nested_selection_and_enumeration_regions() {
    let function = values("async function* values(items){switch(yield 0){case 0:for(const key in await items){switch(yield key){case 'one':yield key;break;default:await 0;}}break;}}");
    let mut items = Vec::new();
    walk(&function.body.statements, &mut items);
    assert_eq!(
        items
            .iter()
            .filter(|item| matches!(item, StatementIr::AsyncGeneratorSwitch(_)))
            .count(),
        2
    );
    let enumeration = items
        .iter()
        .find_map(|item| match item {
            StatementIr::AsyncGeneratorForIn(plan) => Some(plan),
            _ => None,
        })
        .unwrap();
    let inner = items
        .iter()
        .filter_map(|item| match item {
            StatementIr::AsyncGeneratorSwitch(plan) => Some(plan),
            _ => None,
        })
        .last()
        .unwrap();
    assert!(inner.entry_state() >= enumeration.body().entry_state());
    assert!(inner.exit_state() <= enumeration.body().end_state());
}
