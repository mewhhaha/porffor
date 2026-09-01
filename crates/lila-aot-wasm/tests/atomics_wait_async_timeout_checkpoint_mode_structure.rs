const ATOMICS_SOURCE: &str = include_str!("../src/builtins/atomics.rs");

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
fn timeout_checkpoint_mode_is_one_private_closed_domain() {
    let declaration = bounded(
        ATOMICS_SOURCE,
        "#[derive(Clone, Copy, Debug, PartialEq, Eq)]\nenum AtomicsWaitAsyncTimeoutCheckpointMode {",
        "\n}\n\nimpl<'a> FunctionBuilder<'a>",
    );
    let variants = declaration
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(variants, ["Drain,", "Poll,"]);
    assert!(!declaration.contains("pub"));
    assert!(!declaration.contains("bool"));
    assert!(!ATOMICS_SOURCE.contains("impl Default for AtomicsWaitAsyncTimeoutCheckpointMode"));
}

#[test]
fn named_wrappers_own_the_checkpoint_mode() {
    let drain = bounded(
        ATOMICS_SOURCE,
        "pub(crate) fn emit_drain_atomics_wait_async_timeouts(",
        "pub(crate) fn emit_poll_atomics_wait_async_timeouts(",
    );
    assert_eq!(
        drain
            .matches("AtomicsWaitAsyncTimeoutCheckpointMode::Drain")
            .count(),
        1
    );
    assert!(!drain.contains("true"));
    assert!(!drain.contains("false"));

    let poll = bounded(
        ATOMICS_SOURCE,
        "pub(crate) fn emit_poll_atomics_wait_async_timeouts(",
        "fn emit_atomics_wait_async_timeout_checkpoint(",
    );
    assert_eq!(
        poll.matches("AtomicsWaitAsyncTimeoutCheckpointMode::Poll")
            .count(),
        1
    );
    assert!(!poll.contains("true"));
    assert!(!poll.contains("false"));
}

#[test]
fn checkpoint_mode_exhaustively_selects_blocking_behavior() {
    let signature = bounded(
        ATOMICS_SOURCE,
        "fn emit_atomics_wait_async_timeout_checkpoint(",
        ") -> Result<(), EmitError> {",
    );
    assert!(signature.contains("mode: AtomicsWaitAsyncTimeoutCheckpointMode,"));
    assert!(!signature.contains("wait_for_deadline"));
    assert!(!signature.contains("bool"));

    let helper = bounded(
        ATOMICS_SOURCE,
        "fn emit_atomics_wait_async_timeout_checkpoint(",
        "fn emit_atomics_wait(",
    );
    let behavior = bounded(
        helper,
        "match mode {",
        "function.instruction(&Instruction::LocalGet(active_count_local));",
    );
    assert_eq!(
        behavior
            .matches("AtomicsWaitAsyncTimeoutCheckpointMode::Drain => {}")
            .count(),
        1
    );
    assert_eq!(
        behavior
            .matches("AtomicsWaitAsyncTimeoutCheckpointMode::Poll => {")
            .count(),
        1
    );
    assert_eq!(behavior.matches("Instruction::Br(1)").count(), 1);
    assert_eq!(behavior.matches("=>").count(), 2);
    assert!(!behavior.contains("_ =>"));
    assert!(!behavior.contains("unreachable!"));

    for retired in [
        "wait_for_deadline: bool",
        "emit_atomics_wait_async_timeout_checkpoint(function, true)",
        "emit_atomics_wait_async_timeout_checkpoint(function, false)",
    ] {
        assert!(!ATOMICS_SOURCE.contains(retired), "retired `{retired}`");
    }
}
