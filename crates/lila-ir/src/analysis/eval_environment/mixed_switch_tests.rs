use super::*;
use boa_ast::statement::{Switch as AstSwitch, With};

fn switch_analysis(source: &str, check: impl FnOnce(&Analysis<'_>)) {
    let mut interner = Interner::default();
    let scope = Scope::new_global();
    let script = Parser::new(Source::from_bytes(source.as_bytes()))
        .parse_script(&scope, &mut interner)
        .expect("Switch analysis fixture must parse");
    let analysis = AnalysisBuilder::default().finish(&script, &interner, source);
    check(&analysis);
}

#[derive(Default)]
struct ActualSources<'ast> {
    switches: Vec<&'ast AstSwitch>,
    withs: Vec<&'ast With>,
}

impl<'ast> Visitor<'ast> for ActualSources<'ast> {
    type BreakTy = core::convert::Infallible;

    fn visit_switch(&mut self, source: &'ast AstSwitch) -> ControlFlow<Self::BreakTy> {
        self.switches.push(source);
        source.visit_with(self)
    }

    fn visit_with(&mut self, source: &'ast With) -> ControlFlow<Self::BreakTy> {
        self.withs.push(source);
        source.visit_with(self)
    }

    fn visit_function_body(&mut self, _: &'ast FunctionBody) -> ControlFlow<Self::BreakTy> {
        ControlFlow::Continue(())
    }

    fn visit_class_declaration(&mut self, _: &'ast ClassDeclaration) -> ControlFlow<Self::BreakTy> {
        ControlFlow::Continue(())
    }

    fn visit_class_expression(&mut self, _: &'ast ClassExpression) -> ControlFlow<Self::BreakTy> {
        ControlFlow::Continue(())
    }
}

fn actual_sources(body: &FunctionBody) -> ActualSources<'_> {
    let mut sources = ActualSources::default();
    let _ = body.visit_with(&mut sources);
    sources
}

fn with_environment<'analysis, 'ast>(
    analysis: &'analysis Analysis<'ast>,
    source: &With,
) -> (
    &'analysis WithObjectEnvironmentPlan,
    &'analysis EnvironmentPlan,
) {
    let id = analysis.with_environment_ids[&(source as *const With as usize)];
    (
        &analysis.with_object_environment_plans[&id],
        &analysis.environment_plans[&id],
    )
}

fn assert_original_whole_with(analysis: &Analysis<'_>, source: &With) {
    let (object, environment) = with_environment(analysis, source);
    let WithContinuationOwner::MixedAsyncGeneratorWhole(proof) = object.continuation_owner else {
        panic!("checked FunctionBody Switch must retain its actual nested With owner");
    };
    assert!(std::ptr::eq(
        proof.checked_source(source).unwrap().source(),
        source
    ));
    assert_eq!(environment.kind, EnvironmentKind::WithObject);
    let slot = environment.owned_env_slots[object.binding_name.as_str()];
    assert_eq!(
        environment.eval_environment,
        Some(EvalEnvironmentRoleIr::WithObject { object_slot: slot })
    );
    assert!(
        analysis.physical_binding_environments[object.binding_name.as_str()]
            .contains(&environment.id)
    );
}

#[test]
fn checked_mixed_caseblock_with_and_nested_block_keep_original_captured_cells() {
    switch_analysis(
        "async function* work(view,other,flag){switch(await (yield flag)){case await (yield 'selector'):let caseLocal=1;with(view){let innerLocal=2;yield function read(){return [caseLocal,innerLocal,value];};}break;default:with(other){yield value;}}}",
        |analysis| {
            let work = analysis.function_plans.values().find(|function| function.name == "work").unwrap();
            let sources = actual_sources(work.body);
            assert_eq!(sources.switches.len(), 1);
            assert_eq!(sources.withs.len(), 2);
            let checked = crate::async_generator_source::AsyncGeneratorSwitchSource::new(sources.switches[0]).unwrap();
            assert!(std::ptr::eq(checked.source(), sources.switches[0]));
            for source in &sources.withs {
                assert_original_whole_with(analysis, source);
            }
            let case_id = analysis.switch_environment_ids[&(sources.switches[0] as *const AstSwitch as usize)];
            let case_environment = &analysis.environment_plans[&case_id];
            assert_eq!(case_environment.kind, EnvironmentKind::SwitchCaseBlock);
            let read = analysis.function_plans.values().find(|function| function.name == "read").unwrap();
            let (case_name, case_capture) = read.captures.iter().find(|(_, capture)| capture.source_name == "caseLocal").unwrap();
            assert_eq!(case_capture.environment_id, case_id);
            assert_eq!(case_environment.owned_env_slots[case_name], case_capture.slot);
            let (block_name, block_capture) = read.captures.iter().find(|(_, capture)| capture.source_name == "innerLocal").unwrap();
            let block = &analysis.environment_plans[&block_capture.environment_id];
            assert_eq!(block.kind, EnvironmentKind::Block);
            assert_ne!(block.id, case_id);
            assert_eq!(block.owned_env_slots[block_name], block_capture.slot);
            let (object, environment) = with_environment(analysis, sources.withs[0]);
            let object_capture = read.captures.values().find(|capture| capture.environment_id == environment.id).unwrap();
            assert_eq!(object_capture.source_name, object.binding_name.as_str());
            assert_eq!(object_capture.slot, environment.owned_env_slots[object.binding_name.as_str()]);
        },
    );
}

#[test]
fn checked_switch_shape_does_not_widen_foreign_iterator_or_resource_ancestry() {
    switch_analysis(
        "async function* work(view,input,flag){for await(const item of input){switch(flag){case 0:with(view){value;}break;}}for(const item of input){switch(flag){case 0:with(view){value;}break;}}for(const key in input){switch(flag){case 0:with(view){value;}break;}}{using r=resource;switch(flag){case 0:with(view){value;}break;}}{await using r=resource;switch(flag){case 0:with(view){value;}break;}}switch(flag){case 0:for await(const item of input){with(view){value;}}}switch(flag){case 0:with(view){value;}break;}}",
        |analysis| {
            let work = analysis.function_plans.values().find(|function| function.name == "work").unwrap();
            let sources = actual_sources(work.body);
            assert_eq!(sources.switches.len(), 7);
            assert_eq!(sources.withs.len(), 7);
            for index in 0..5 {
                assert!(crate::async_generator_source::AsyncGeneratorSwitchSource::new(sources.switches[index]).is_some());
                if matches!(index,0|1|2|3|4) {
                    assert_original_whole_with(analysis, sources.withs[index]);
                } else {
                    let (object, environment) = with_environment(analysis, sources.withs[index]);
                    assert_eq!(object.continuation_owner, WithContinuationOwner::LinearResumable);
                    assert!(environment.owned_env_slots.is_empty());
                }
            }
            assert!(crate::async_generator_source::AsyncGeneratorSwitchSource::new(sources.switches[5]).is_some());
            assert_original_whole_with(analysis,sources.withs[5]);
            assert!(crate::async_generator_source::AsyncGeneratorSwitchSource::new(sources.switches[6]).is_some());
            assert_original_whole_with(analysis, sources.withs[6]);
        },
    );
}

#[test]
fn nested_callable_checked_switch_owns_its_independent_function_body_domain() {
    switch_analysis(
        "async function* outer(view,input){for await(const item of input){switch(0){case 0:with(view){value;}const nested=async function* nested(flag){switch(await flag){case 0:with(view){yield value;}break;default:break;}};yield nested;}}}",
        |analysis| {
            let outer = analysis.function_plans.values().find(|function| function.name == "outer").unwrap();
            let sources = actual_sources(outer.body);
            assert_eq!(sources.withs.len(), 1);
            assert_original_whole_with(analysis,sources.withs[0]);
            let nested = analysis.function_plans.values().find(|function| function.name == "nested").unwrap();
            let sources = actual_sources(nested.body);
            assert_eq!(sources.switches.len(), 1);
            assert_eq!(sources.withs.len(), 1);
            assert!(crate::async_generator_source::AsyncGeneratorSwitchSource::new(sources.switches[0]).is_some());
            assert_original_whole_with(analysis, sources.withs[0]);
        },
    );
}
