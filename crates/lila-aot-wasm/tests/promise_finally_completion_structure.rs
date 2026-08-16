const PROMISE_SOURCE: &str = include_str!("../src/builtins/promise.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");

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
fn finally_completion_is_one_private_closed_domain() {
    let declaration = bounded(
        PROMISE_SOURCE,
        "enum PromiseFinallyCompletion {",
        "impl PromiseFinallyCompletion {",
    );
    assert_eq!(declaration.matches("Fulfill,").count(), 1);
    assert_eq!(declaration.matches("Reject,").count(), 1);
    assert_eq!(
        declaration
            .lines()
            .map(str::trim)
            .filter(|line| line.ends_with(','))
            .count(),
        2,
        "the completion domain must contain exactly two variants"
    );
    assert!(!declaration.contains("pub"));
    assert!(!declaration.contains("bool"));

    let policy = bounded(
        PROMISE_SOURCE,
        "impl PromiseFinallyCompletion {",
        "#[derive(Clone, Copy, PartialEq, Eq)]",
    );
    for arm in [
        "Self::Fulfill => StandardBuiltinId::PromiseValueThunk",
        "Self::Reject => StandardBuiltinId::PromiseThrower",
        "Self::Fulfill => CompletionKind::Normal",
        "Self::Reject => CompletionKind::Throw",
    ] {
        assert_eq!(policy.matches(arm).count(), 1, "policy arm `{arm}`");
    }
    assert_eq!(policy.matches("match self {").count(), 2);
    assert_eq!(policy.matches("=>").count(), 4);
    assert!(!policy.contains("_ =>"));
    assert!(!policy.contains("unreachable!"));
}

#[test]
fn named_wrappers_own_the_four_spec_mappings() {
    for (start, end, expected) in [
        (
            "pub(crate) fn emit_promise_then_finally(",
            "pub(crate) fn emit_promise_catch_finally(",
            "PromiseFinallyCompletion::Fulfill",
        ),
        (
            "pub(crate) fn emit_promise_catch_finally(",
            "fn emit_promise_finally_continuation(",
            "PromiseFinallyCompletion::Reject",
        ),
        (
            "pub(crate) fn emit_promise_value_thunk(",
            "pub(crate) fn emit_promise_thrower(",
            "PromiseFinallyCompletion::Fulfill",
        ),
        (
            "pub(crate) fn emit_promise_thrower(",
            "fn emit_promise_finally_value_thunk(",
            "PromiseFinallyCompletion::Reject",
        ),
    ] {
        let wrapper = bounded(PROMISE_SOURCE, start, end);
        assert_eq!(wrapper.matches(expected).count(), 1, "wrapper `{start}`");
        assert!(!wrapper.contains("true"), "wrapper `{start}`");
        assert!(!wrapper.contains("false"), "wrapper `{start}`");
    }

    let continuation = bounded(
        PROMISE_SOURCE,
        "fn emit_promise_finally_continuation(",
        "pub(crate) fn emit_promise_value_thunk(",
    );
    assert!(continuation.contains("completion: PromiseFinallyCompletion,"));
    assert_eq!(
        continuation
            .matches("completion.continuation_builtin()")
            .count(),
        1
    );

    let restoration = bounded(
        PROMISE_SOURCE,
        "fn emit_promise_finally_value_thunk(",
        "fn emit_run_async_continuation_job(",
    );
    assert!(restoration.contains("completion: PromiseFinallyCompletion,"));
    assert_eq!(
        restoration.matches("completion.completion_kind()").count(),
        1
    );

    for retired in [
        "emit_promise_finally_continuation(\n        &mut self,\n        rejected: bool,",
        "emit_promise_finally_value_thunk(\n        &mut self,\n        throws: bool,",
        "emit_promise_finally_continuation(false, function)",
        "emit_promise_finally_continuation(true, function)",
        "emit_promise_finally_value_thunk(false, function)",
        "emit_promise_finally_value_thunk(true, function)",
    ] {
        assert!(!PROMISE_SOURCE.contains(retired), "retired `{retired}`");
        assert!(!STANDARD_SOURCE.contains(retired), "retired `{retired}`");
    }
}

#[test]
fn standard_dispatch_has_no_finally_direction_choice() {
    let dispatch = bounded(
        STANDARD_SOURCE,
        "StandardBuiltinId::PromiseThenFinally => {",
        "StandardBuiltinId::PromiseResolve => {",
    );
    for wrapper in [
        "self.emit_promise_then_finally(function)?;",
        "self.emit_promise_catch_finally(function)?;",
        "self.emit_promise_value_thunk(function)?;",
        "self.emit_promise_thrower(function)?;",
    ] {
        assert_eq!(dispatch.matches(wrapper).count(), 1, "dispatch `{wrapper}`");
    }
    assert!(!dispatch.contains("PromiseFinallyCompletion"));
    assert!(!dispatch.contains("true"));
    assert!(!dispatch.contains("false"));
}
