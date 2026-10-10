const FOR_LOOP_LOWERING_SOURCE: &str = include_str!("../../../lila-ir/src/lowering/for_loop.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

fn assert_before(source: &str, earlier: &str, later: &str) {
    let earlier = source.find(earlier).expect("earlier operation");
    let later = source.find(later).expect("later operation");
    assert!(earlier < later, "`{earlier}` must precede `{later}`");
}

#[test]
fn async_generator_classic_for_registers_activation_owned_lexicals_before_loop_plan() {
    let loop_split = bounded(
        FOR_LOOP_LOWERING_SOURCE,
        "Self::split_resumable_loop_body(",
        "StatementIr::GeneratorLoop {",
    );
    let activation_ownership = bounded(
        loop_split,
        "                    if self.current_resumable_plan.is_some() {",
        "                    let exit_state = if self.current_resumable_plan.is_some() {",
    );

    for initializer in [
        "Some(ForInitIr::Lexical { mode, name, .. })",
        "Some(ForInitIr::LexicalBlock(bindings))",
    ] {
        assert!(activation_ownership.contains(initializer), "{initializer}");
    }
    assert!(activation_ownership
        .contains("Some(ForInitIr::Var(_)) | Some(ForInitIr::Expression(_)) | None => {}"));
    assert!(!activation_ownership.contains("_ =>"));
    assert_eq!(
        activation_ownership
            .matches("self.add_suspension_owned_binding(")
            .count(),
        3,
        "the lexical initializer, initializer block and direct body lexical paths must register"
    );
    assert_eq!(
        activation_ownership
            .matches("self.add_suspension_owned_binding(name.clone(), *mode);")
            .count(),
        2,
        "initializer and body bindings retain their actual write policy"
    );
    assert!(activation_ownership.contains("binding.mode,"));

    assert_before(
        activation_ownership,
        "match &init {",
        "for statement in before_suspension",
    );
    assert_before(
        activation_ownership,
        ".chain(after_suspension.iter())",
        "if let StatementIr::Lexical { mode, name, .. } = statement",
    );
    assert_before(
        loop_split,
        "self.add_suspension_owned_binding(name.clone(), *mode);",
        "let exit_state = if self.current_resumable_plan.is_some()",
    );
}
