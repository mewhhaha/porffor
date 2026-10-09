const CONTROL_FLOW_SOURCE: &str = include_str!("../../src/control_flow.rs");

fn normalized(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

const DELEGATION_SOURCE: &str = include_str!("../../src/generator_delegation.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

#[test]
fn delegation_kind_is_the_exact_non_capability_two_row_domain() {
    let attributes = DELEGATION_SOURCE
        .split_once("pub(crate) enum AsyncGeneratorDelegationKind")
        .expect("closed enum declaration")
        .0
        .rsplit("\n\n")
        .next()
        .expect("enum declaration prefix");
    assert!(!attributes.contains("#[derive"));
    let declaration = bounded(
        DELEGATION_SOURCE,
        "pub(crate) enum AsyncGeneratorDelegationKind {",
        "enum GeneratorDelegateProperty {",
    );
    assert_eq!(normalized(declaration), "YieldStar,ForAwaitYield,}");
    for forbidden in [
        "impl Clone for AsyncGeneratorDelegationKind",
        "impl Copy for AsyncGeneratorDelegationKind",
        "impl Default for AsyncGeneratorDelegationKind",
        "impl PartialEq for AsyncGeneratorDelegationKind",
        "impl Eq for AsyncGeneratorDelegationKind",
        "for AsyncGeneratorDelegationKind",
    ] {
        assert!(
            !DELEGATION_SOURCE.contains(forbidden),
            "found `{forbidden}`"
        );
    }
}

#[test]
fn control_flow_has_exactly_one_producer_for_each_delegation_kind() {
    let control_flow = normalized(CONTROL_FLOW_SOURCE);
    let yield_body = normalized(bounded(
        CONTROL_FLOW_SOURCE,
        "fn compile_async_generator_yield(",
        "let schema = self.runtime_schema();",
    ));
    let delegate = yield_body
        .split_once("YieldForm::Delegate(_)=>{")
        .expect("yield-star retains its exact form arm")
        .1;
    assert!(delegate.contains(
        "returnself.compile_async_generator_delegation(value,suspend_state,resume_state,resume_mode,AsyncGeneratorDelegationKind::YieldStar,function,);"));
    assert!(
        delegate.contains("emit_checkpoint_generator_statement_list_value(function)"),
        "delegation retains the enclosing statement-list completion across suspension"
    );
    assert!(control_flow.contains(
        "async_generator_for_await_is_transparent_yield(&binding.name,body){self.compile_async_generator_delegation(iterable,async_plan.entry_state,async_plan.exit_state,&GeneratorResumeModeIr::Ignore,AsyncGeneratorDelegationKind::ForAwaitYield,function,)?;returnOk(());}"));
    assert_eq!(
        CONTROL_FLOW_SOURCE
            .matches("AsyncGeneratorDelegationKind::YieldStar")
            .count(),
        1
    );
    assert_eq!(
        CONTROL_FLOW_SOURCE
            .matches("AsyncGeneratorDelegationKind::ForAwaitYield")
            .count(),
        1
    );
}
