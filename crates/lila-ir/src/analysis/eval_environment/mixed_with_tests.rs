use super::*;

fn with_analysis(source: &str, check: impl FnOnce(&Analysis<'_>)) {
    let mut interner = Interner::default();
    let scope = Scope::new_global();
    let script = Parser::new(Source::from_bytes(source.as_bytes()))
        .parse_script(&scope, &mut interner)
        .expect("analysis fixture must parse");
    let analysis = AnalysisBuilder::default().finish(&script, &interner, source);
    check(&analysis);
}

fn actual_with_sources<'ast>(body: &'ast FunctionBody) -> Vec<&'ast boa_ast::statement::With> {
    struct Sources<'ast>(Vec<&'ast boa_ast::statement::With>);
    impl<'ast> Visitor<'ast> for Sources<'ast> {
        type BreakTy = core::convert::Infallible;
        fn visit_with(
            &mut self,
            source: &'ast boa_ast::statement::With,
        ) -> ControlFlow<Self::BreakTy> {
            self.0.push(source);
            source.visit_with(self)
        }
        fn visit_function_body(&mut self, _: &'ast FunctionBody) -> ControlFlow<Self::BreakTy> {
            ControlFlow::Continue(())
        }
        fn visit_class_declaration(
            &mut self,
            _: &'ast ClassDeclaration,
        ) -> ControlFlow<Self::BreakTy> {
            ControlFlow::Continue(())
        }
        fn visit_class_expression(
            &mut self,
            _: &'ast ClassExpression,
        ) -> ControlFlow<Self::BreakTy> {
            ControlFlow::Continue(())
        }
    }
    let mut sources = Sources(Vec::new());
    let _ = body.visit_with(&mut sources);
    sources.0
}

fn analyzed_with<'analysis, 'ast>(
    analysis: &'analysis Analysis<'ast>,
    source: &boa_ast::statement::With,
) -> (
    &'analysis WithObjectEnvironmentPlan,
    &'analysis EnvironmentPlan,
) {
    let id = analysis.with_environment_ids[&(source as *const _ as usize)];
    (
        &analysis.with_object_environment_plans[&id],
        &analysis.environment_plans[&id],
    )
}

fn assert_original_mixed_row(analysis: &Analysis<'_>, source: &boa_ast::statement::With) {
    let (object, environment) = analyzed_with(analysis, source);
    let WithContinuationOwner::MixedAsyncGeneratorWhole(proof) = object.continuation_owner else {
        panic!("actual function-body With must carry its checked source proof");
    };
    let checked = proof
        .checked_source(source)
        .expect("same actual AST identity");
    assert!(std::ptr::eq(checked.source(), source));
    let states = checked
        .states(7)
        .expect("same checked source at real entry");
    assert_eq!(states.head().entry(), 7);
    assert_eq!(states.body().entry(), states.head().end() + 1);
    assert_eq!(states.exit(), states.body().end() + 1);
    assert_eq!(environment.kind, EnvironmentKind::WithObject);
    let slot = environment.owned_env_slots[object.binding_name.as_str()];
    assert_eq!(
        environment.eval_environment,
        Some(EvalEnvironmentRoleIr::WithObject { object_slot: slot }),
    );
    assert!(environment
        .binding_storage_names
        .contains(object.binding_name.as_str()));
    assert!(
        analysis.physical_binding_environments[object.binding_name.as_str()]
            .contains(&environment.id)
    );
}

#[test]
fn mixed_function_with_eager_and_suspended_phases_materialize_the_original_row() {
    with_analysis(
        "async function* work(scope,flag){with(scope){value;}for(let i=0;i<1;i++){with(scope){value=await Promise.resolve(yield value);}}if(flag){with(scope){value;}}}",
        |analysis| {
            let function = analysis.function_plans.values().find(|function| function.name == "work").unwrap();
            let sources = actual_with_sources(function.body);
            assert_eq!(sources.len(), 3);
            for source in sources {
                assert_original_mixed_row(analysis, source);
            }
        },
    );
}

#[test]
fn mixed_with_foreign_iterator_switch_and_resource_suffix_keep_their_actual_owner() {
    with_analysis(
        "async function* work(scope,items,flag){with(scope){value;}{using r=resource;with(scope){value;}}with(scope){value;}{await using r=resource;with(scope){value;}}with(scope){value;}for await(const item of items){with(scope){value;}}for(const item of items){with(scope){value;}}for(const key in items){with(scope){value;}}switch(flag){case 0:with(scope){value;}break;}}",
        |analysis| {
            let function = analysis.function_plans.values().find(|function| function.name == "work").unwrap();
            let sources = actual_with_sources(function.body);
            assert_eq!(sources.len(), 9);
            for (index, source) in sources.into_iter().enumerate() {
                if matches!(index, 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8) {
                    assert_original_mixed_row(analysis, source);
                } else {
                    let (object, environment) = analyzed_with(analysis, source);
                    assert_eq!(object.continuation_owner, WithContinuationOwner::LinearResumable);
                    assert!(environment.owned_env_slots.is_empty());
                }
            }
        },
    );
}

#[test]
fn mixed_with_proof_refuses_an_equal_shaped_foreign_ast_identity() {
    with_analysis(
        "async function* work(scope){with(scope){value;}with(scope){value;}}",
        |analysis| {
            let function = analysis
                .function_plans
                .values()
                .find(|function| function.name == "work")
                .unwrap();
            let sources = actual_with_sources(function.body);
            assert_eq!(sources.len(), 2);
            let (first, _) = analyzed_with(analysis, sources[0]);
            let WithContinuationOwner::MixedAsyncGeneratorWhole(proof) = first.continuation_owner
            else {
                panic!("first actual With must have the checked owner");
            };
            assert!(proof.checked_source(sources[0]).is_some());
            assert!(proof.checked_source(sources[1]).is_none());
        },
    );
}

#[test]
fn mixed_with_rows_are_reserved_before_real_nested_capture_hops() {
    with_analysis(
        "async function* work(outer,inner){with(outer){let marker=1;with(inner){function read(){return marker+value;}yield read;}}}",
        |analysis| {
            let work = analysis.function_plans.values().find(|function| function.name == "work").unwrap();
            let sources = actual_with_sources(work.body);
            assert_eq!(sources.len(), 2);
            for source in &sources {
                assert_original_mixed_row(analysis, source);
            }
            let read = analysis.function_plans.values().find(|function| function.name == "read").unwrap();
            let captures: Vec<_> = read.captures.values().filter(|capture| analysis.environment_plans[&capture.environment_id].kind == EnvironmentKind::WithObject).collect();
            assert_eq!(captures.len(), 2);
            for capture in captures {
                let environment = &analysis.environment_plans[&capture.environment_id];
                let object = &analysis.with_object_environment_plans[&environment.id];
                assert_eq!(capture.source_name, object.binding_name.as_str());
                assert_eq!(environment.owned_env_slots[object.binding_name.as_str()], capture.slot);
                let (_, outer) = analyzed_with(analysis, sources[0]);
                let (_, inner) = analyzed_with(analysis, sources[1]);
                // The actual marker block lies between the two object records;
                // the ordinary reader has no materialized local activation.
                let expected_hops = if environment.id == inner.id {
                    0
                } else {
                    assert_eq!(environment.id, outer.id);
                    2
                };
                assert_eq!(capture.hops, expected_hops);
            }
        },
    );
}

#[test]
fn mixed_with_foreign_body_refusal_and_nested_callable_scope_are_distinct() {
    with_analysis(
        "async function* outer(scope,items){with(scope){for await(const item of items){yield item;}}for await(const item of items){with(scope){value;}const nested=async function* nested(){with(scope){yield value;}};yield nested;}}",
        |analysis| {
            let outer = analysis.function_plans.values().find(|function| function.name == "outer").unwrap();
            let sources = actual_with_sources(outer.body);
            assert_eq!(sources.len(), 2);
            assert!(crate::async_generator_source::AsyncGeneratorWithSource::new(sources[0]).is_some());
            for source in sources {
                assert_original_mixed_row(analysis, source);
            }
            let nested = analysis.function_plans.values().find(|function| function.name == "nested").unwrap();
            let sources = actual_with_sources(nested.body);
            assert_eq!(sources.len(), 1);
            assert_original_mixed_row(analysis, sources[0]);
        },
    );
}
