use super::*;

fn analyze(source: &'static str) -> Analysis<'static> {
    let interner = Box::leak(Box::new(Interner::default()));
    let scope = Scope::new_global();
    let script = Box::leak(Box::new(
        Parser::new(Source::from_bytes(source.as_bytes()))
            .parse_script(&scope, interner)
            .expect("private-environment fixture should parse"),
    ));
    AnalysisBuilder::default().finish(script, interner, source)
}

fn environment_with_binding(analysis: &Analysis<'_>, name: &str) -> PrivateEnvironmentId {
    analysis
        .private_environment_plans
        .values()
        .find(|environment| environment.bindings.contains_key(name))
        .map(|environment| environment.id)
        .unwrap_or_else(|| panic!("private environment should bind `{name}`"))
}

fn function_private_environment(
    analysis: &Analysis<'_>,
    name: &str,
) -> Option<PrivateEnvironmentId> {
    let function = analysis
        .function_plans
        .values()
        .find(|function| function.name == name)
        .unwrap_or_else(|| panic!("function `{name}` should be planned"));
    analysis.owner_plans[&function.id].private_environment_id
}

#[test]
fn private_environments_resolve_nearest_binding_and_share_accessor_names() {
    let analysis = analyze(
        "class Outer {
                get #value() { return 1; }
                set #value(next) {}
                make() {
                    function read(receiver) { return receiver.#value; }
                    return class Inner {
                        #value;
                        readOuter(receiver) { return receiver.#value; }
                    };
                }
            }",
    );
    assert_eq!(analysis.private_environment_plans.len(), 2);

    let outer_id = analysis
        .private_environment_plans
        .values()
        .find(|environment| environment.parent.is_none())
        .expect("outer private environment should be planned")
        .id;
    let inner_id = analysis
        .private_environment_plans
        .values()
        .find(|environment| environment.parent == Some(outer_id))
        .expect("inner private environment should link to outer")
        .id;
    let outer = &analysis.private_environment_plans[&outer_id];
    assert_eq!(outer.bindings.len(), 1);
    assert_eq!(
        function_private_environment(&analysis, "read"),
        Some(outer_id)
    );

    let (_, outer_value) = analysis
        .resolve_private_name(Some(outer_id), "value")
        .expect("outer private name should resolve");
    let (resolved_inner_id, inner_value) = analysis
        .resolve_private_name(Some(inner_id), "value")
        .expect("inner private name should shadow outer");
    assert_eq!(resolved_inner_id, inner_id);
    assert_ne!(inner_value, outer_value);
}

#[test]
fn class_heritage_uses_outer_private_environment_and_body_uses_inner() {
    let analysis = analyze(
        "class Outer {
                #outer;
                make(Base) {
                    return class Inner extends ((function heritage(receiver) {
                        return receiver.#outer;
                    }), Base) {
                        #inner;
                        [(function body(receiver) { return receiver.#inner; })()]() {}
                    };
                }
            }",
    );
    let outer_id = environment_with_binding(&analysis, "outer");
    let inner_id = environment_with_binding(&analysis, "inner");
    assert_eq!(
        analysis.private_environment_plans[&inner_id].parent,
        Some(outer_id)
    );
    assert_eq!(
        function_private_environment(&analysis, "heritage"),
        Some(outer_id)
    );
    assert_eq!(
        function_private_environment(&analysis, "body"),
        Some(inner_id)
    );
}

#[test]
fn analysis_progress_keeps_the_callers_template_registry_and_shared_cache() {
    let parsed = scriptw_front::parse_script_utf16(
        &"function tag() {} tag`one`; class C { #x; }"
            .encode_utf16()
            .collect::<Vec<_>>(),
    )
    .unwrap();
    let mut state = AnalysisAllocationState::default().with_private_environment_floor(17);
    state.allocate_static_script_id();
    let template = state.template_sources.for_parsed(&parsed);
    parsed.with_compiler_session(|script, interner| {
        let analysis =
            AnalysisBuilder::with_allocations(&state, ScriptInstantiation::FreshEntry, template)
                .finish(
                    script,
                    interner,
                    CompilerSourceText::Utf16(parsed.compiler_source_utf16()),
                );
        assert!(analysis.allocations.next_function_id > state.next_function_id);
        assert!(analysis.allocations.next_private_environment_id > 17);
        state.commit_analysis(&analysis.allocations);
        assert_eq!(
            state.next_function_id,
            analysis.allocations.next_function_id
        );
        assert_eq!(
            state.next_environment_id,
            analysis.allocations.next_environment_id
        );
        assert_eq!(
            state.next_private_environment_id,
            analysis.allocations.next_private_environment_id
        );
        assert_eq!(state.next_static_script_id, 1);
    });
    assert_eq!(state.template_sources.for_parsed(&parsed), template);
    let separate =
        scriptw_front::parse_script_utf16(&"tag`two`;".encode_utf16().collect::<Vec<_>>()).unwrap();
    assert_ne!(state.template_sources.for_parsed(&separate), template);
}
