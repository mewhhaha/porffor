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
    let selector = bounded(
        helper,
        "let reaction_list_offset = match reaction_type {",
        "        };",
    );
    for arm in [
        "PromiseReactionType::Fulfill => HEAP_PROMISE_FULFILL_REACTIONS_OFFSET",
        "PromiseReactionType::Reject => HEAP_PROMISE_REJECT_REACTIONS_OFFSET",
    ] {
        assert_eq!(selector.matches(arm).count(), 1, "selector arm `{arm}`");
    }
    assert_eq!(selector.matches("=>").count(), 2);
    assert!(!selector.contains("_ =>"));
    assert!(!selector.contains("unreachable!"));

    for retired in [
        "reaction_list_offset: u64",
        "emit_append_promise_reaction(\n                        promise_record_local,\n                        HEAP_PROMISE_FULFILL_REACTIONS_OFFSET,",
        "emit_append_promise_reaction(\n                        promise_record_local,\n                        HEAP_PROMISE_REJECT_REACTIONS_OFFSET,",
    ] {
        assert!(!PROMISE_SOURCE.contains(retired), "retired `{retired}`");
    }
}

#[test]
fn pending_state_pairs_each_reaction_with_its_direction() {
    let route = bounded(
        PROMISE_SOURCE,
        "fn emit_route_promise_reaction_pair(",
        "fn emit_intrinsic_promise_resolve_to_locals(",
    );
    let pending = bounded(
        route,
        "PromiseState::Pending => {",
        "PromiseState::Fulfilled =>",
    );
    for mapping in [
        ("PromiseReactionType::Fulfill,", "fulfill_reaction_local,"),
        ("PromiseReactionType::Reject,", "reject_reaction_local,"),
    ] {
        let direction = pending
            .split_once(mapping.0)
            .unwrap_or_else(|| panic!("missing direction `{}`", mapping.0))
            .1
            .split_once("function,")
            .unwrap_or_else(|| panic!("missing function argument after `{}`", mapping.0))
            .0;
        assert!(direction.contains(mapping.1), "mapping `{:?}`", mapping);
    }
    assert_eq!(pending.matches("emit_append_promise_reaction(").count(), 2);
    assert_eq!(pending.matches("PromiseReactionType::Fulfill,").count(), 1);
    assert_eq!(pending.matches("PromiseReactionType::Reject,").count(), 1);
    assert!(!pending.contains("HEAP_PROMISE_FULFILL_REACTIONS_OFFSET"));
    assert!(!pending.contains("HEAP_PROMISE_REJECT_REACTIONS_OFFSET"));
}
