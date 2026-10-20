use super::*;
use boa_ast::statement::{iteration::ForInLoop, With};

fn for_in_analysis(source: &str, check: impl FnOnce(&Analysis<'_>, &Interner)) {
    let mut interner = Interner::default();
    let scope = Scope::new_global();
    let script = Parser::new(Source::from_bytes(source.as_bytes()))
        .parse_script(&scope, &mut interner)
        .expect("mixed ForIn analysis source must parse");
    let analysis = AnalysisBuilder::default().finish(&script, &interner, source);
    check(&analysis, &interner);
}

#[derive(Default)]
struct ForInSources<'ast> {
    loops: Vec<&'ast ForInLoop>,
    withs: Vec<&'ast With>,
}

impl<'ast> Visitor<'ast> for ForInSources<'ast> {
    type BreakTy = core::convert::Infallible;

    fn visit_statement(&mut self, source: &'ast Statement) -> ControlFlow<Self::BreakTy> {
        if let Statement::ForInLoop(source) = source {
            self.loops.push(source);
        }
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

fn for_in_sources(body: &FunctionBody) -> ForInSources<'_> {
    let mut sources = ForInSources::default();
    let _ = body.visit_with(&mut sources);
    sources
}

fn actual_whole_for_in(
    analysis: &Analysis<'_>,
    source: &ForInLoop,
    interner: &Interner,
) -> CompleteResumableForInOwner {
    let owner = analysis.for_in_continuation_owners[&(source as *const ForInLoop as usize)];
    let ForInContinuationOwner::CompleteWhole(owner) = owner else {
        panic!("only actual checked FunctionBody ForIn may own mixed whole phases");
    };
    let checked = owner.checked_source(source).unwrap();
    assert!(std::ptr::eq(checked.source(), source));
    let states = checked.states(9).unwrap();
    assert_eq!(states.entry(), 9);
    assert_eq!(states.advance_state(), states.head().end() + 1);
    assert_eq!(states.initialization().entry(), states.advance_state() + 1);
    assert_eq!(states.body().entry(), states.initialization().end() + 1);
    assert_eq!(states.exit(), states.body().end() + 1);
    assert!(states.matches_head(&checked.checked_head(interner).unwrap()));
    owner
}

fn actual_whole_with(analysis: &Analysis<'_>, source: &With) -> EnvironmentId {
    let id = analysis.with_environment_ids[&(source as *const With as usize)];
    let object = &analysis.with_object_environment_plans[&id];
    let WithContinuationOwner::MixedAsyncGeneratorWhole(owner) = object.continuation_owner else {
        panic!("checked complete ForIn must keep the original nested With owner");
    };
    assert!(std::ptr::eq(
        owner.checked_source(source).unwrap().source(),
        source
    ));
    let environment = &analysis.environment_plans[&id];
    let slot = environment.owned_env_slots[object.binding_name.as_str()];
    assert_eq!(environment.kind, EnvironmentKind::WithObject);
    assert_eq!(
        environment.eval_environment,
        Some(EvalEnvironmentRoleIr::WithObject { object_slot: slot })
    );
    assert!(analysis.physical_binding_environments[object.binding_name.as_str()].contains(&id));
    id
}

#[test]
fn mixed_for_in_retains_original_head_tdz_iteration_and_nested_with_capture_cells() {
    for_in_analysis(
        "async function* work(view,input){for(let key in await (yield ()=>key)){with(view){let marker=await (yield key);yield function read(){return [key,marker,value];};}}}",
        |analysis, interner| {
            let work = analysis.function_plans.values().find(|function| function.name == "work").unwrap();
            let sources = for_in_sources(work.body);
            assert_eq!(sources.loops.len(), 1);
            assert_eq!(sources.withs.len(), 1);
            actual_whole_for_in(analysis, sources.loops[0], interner);
            let identity = sources.loops[0] as *const ForInLoop as usize;
            let tdz = analysis.for_in_of_tdz_environment_ids[&identity];
            let iteration = analysis.for_in_of_iteration_environment_ids[&identity];
            assert_ne!(tdz, iteration);
            assert_eq!(analysis.environment_plans[&tdz].kind, EnvironmentKind::ForInOfTdzHead);
            assert_eq!(analysis.environment_plans[&iteration].kind, EnvironmentKind::ForInOfIteration);
            let head_capture = analysis.function_plans.values().flat_map(|function| function.captures.iter())
                .find(|(_, capture)| capture.environment_id == tdz).expect("actual arrow closes over head TDZ");
            assert_eq!(analysis.environment_plans[&tdz].owned_env_slots[head_capture.0], head_capture.1.slot);
            let read = analysis.function_plans.values().find(|function| function.name == "read").unwrap();
            let (key, capture) = read.captures.iter().find(|(_, capture)| capture.environment_id == iteration).unwrap();
            assert_eq!(analysis.environment_plans[&iteration].owned_env_slots[key], capture.slot);
            let (marker, capture) = read.captures.iter().find(|(_, capture)| capture.source_name == "marker").unwrap();
            let block = &analysis.environment_plans[&capture.environment_id];
            assert_eq!(block.kind, EnvironmentKind::Block);
            assert_eq!(block.owned_env_slots[marker], capture.slot);
            let object = actual_whole_with(analysis, sources.withs[0]);
            assert!(read.captures.values().any(|capture| capture.environment_id == object));
        },
    );
}

#[test]
fn mixed_for_in_identity_proof_and_original_identifier_reference_capture_are_distinct() {
    for_in_analysis(
        "async function* work(view,input){with(view){let target=0;function read(){return target;}for(target in await (yield input)){yield read;}for(target in await (yield input)){yield read;}}}",
        |analysis, interner| {
            let work = analysis.function_plans.values().find(|function| function.name == "work").unwrap();
            let sources = for_in_sources(work.body);
            assert_eq!(sources.loops.len(), 2);
            let first = actual_whole_for_in(analysis, sources.loops[0], interner);
            actual_whole_for_in(analysis, sources.loops[1], interner);
            assert!(first.checked_source(sources.loops[1]).is_none());
            let head = first.checked_source(sources.loops[0]).unwrap().checked_head(interner).unwrap();
            assert_eq!(head.mode(), BindingMode::Var);
            assert_eq!(head.kind(), crate::lowering_helpers::GeneratorForInHeadKind::AssignmentIdentifier);
            assert!(head.bound_names().is_empty());
            let read = analysis.function_plans.values().find(|function| function.name == "read").unwrap();
            let (name, capture) = read.captures.iter().find(|(_, capture)| capture.source_name == "target").unwrap();
            let environment = &analysis.environment_plans[&capture.environment_id];
            assert_eq!(environment.kind, EnvironmentKind::Block);
            assert_eq!(environment.owned_env_slots[name], capture.slot);
            actual_whole_with(analysis, sources.withs[0]);
        },
    );
}

#[test]
fn mixed_for_in_foreign_resource_and_unsupported_head_domains_do_not_mint_whole_owners() {
    for_in_analysis(
        "async function* outer(view,input){for await(const item of input){for(let key in input){with(view){value;}}}for(const item of input){for(let key in input){with(view){value;}}}{using r=resource;for(let key in input){with(view){value;}}}{await using r=resource;for(let key in input){with(view){value;}}}for(let key in input){for await(const item of input){with(view){value;}}}for(factory() in input){with(view){value;}}for(let key in input){with(view){yield value;}}for await(const item of input){const nested=async function* nested(){for(let key in input){with(view){yield value;}}};yield nested;}}",
        |analysis, interner| {
            let outer = analysis.function_plans.values().find(|function| function.name == "outer").unwrap();
            let sources = for_in_sources(outer.body);
            assert_eq!(sources.loops.len(), 7);
            assert_eq!(sources.withs.len(), 7);
            for index in [5] {
                assert_eq!(analysis.for_in_continuation_owners[&(sources.loops[index] as *const ForInLoop as usize)], ForInContinuationOwner::ImmediateOrLinear);
                let id = analysis.with_environment_ids[&(sources.withs[index] as *const With as usize)];
                assert_eq!(analysis.with_object_environment_plans[&id].continuation_owner, WithContinuationOwner::LinearResumable);
            }
            for index in [0,1,2,3,4,6] { actual_whole_for_in(analysis,sources.loops[index],interner); actual_whole_with(analysis,sources.withs[index]); }
            for source in &sources.loops[..4] {
                assert!(crate::async_generator_source::AsyncGeneratorForInSource::new(source).is_some());
            }
            assert!(crate::async_generator_source::AsyncGeneratorForInSource::new(sources.loops[4]).is_some());
            assert!(crate::async_generator_source::AsyncGeneratorForInSource::new(sources.loops[5]).is_none());
            actual_whole_for_in(analysis, sources.loops[6], interner);
            actual_whole_with(analysis, sources.withs[6]);
            let nested = analysis.function_plans.values().find(|function| function.name == "nested").unwrap();
            let nested_sources = for_in_sources(nested.body);
            assert_eq!(nested_sources.loops.len(), 1);
            actual_whole_for_in(analysis, nested_sources.loops[0], interner);
            actual_whole_with(analysis, nested_sources.withs[0]);
        },
    );
}
