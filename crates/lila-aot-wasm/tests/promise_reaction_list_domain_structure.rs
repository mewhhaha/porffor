const PROMISE_SOURCE: &str = include_str!("../src/builtins/promise.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}`"))
        .0
}

#[test]
fn reaction_list_selector_uses_the_closed_reaction_domain() {
    let signature = bounded(PROMISE_SOURCE, "fn emit_append_promise_reaction(", ") {");
    assert!(signature.contains("reaction_type: PromiseReactionType,"));
    assert!(!signature.contains("reaction_list_offset"));
    assert!(!signature.contains(": u64"));
    let helper = bounded(
        PROMISE_SOURCE,
        "fn emit_append_promise_reaction(",
        "fn emit_route_promise_reaction_pair(",
    );
    let selector = bounded(helper, "let field = match reaction_type {", "        };");
    for direction in [
        "PromiseReactionType::Fulfill =>",
        "PromiseReactionType::Reject =>",
    ] {
        assert_eq!(
            selector.matches(direction).count(),
            1,
            "selector direction `{direction}`"
        );
    }
    assert_eq!(selector.matches("=>").count(), 2);
    assert!(!selector.contains("_ =>"));
    assert!(!selector.contains("unreachable!"));
}
