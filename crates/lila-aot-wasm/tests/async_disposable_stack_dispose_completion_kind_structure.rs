const SOURCE: &str = include_str!("../src/builtins/async_disposable_stack.rs");

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
fn disposal_completion_kind_is_a_private_capability_free_domain() {
    assert_eq!(
        SOURCE
            .matches("enum AsyncDisposableStackDisposeCompletionKind {")
            .count(),
        1
    );
    assert!(!SOURCE.contains("pub enum AsyncDisposableStackDisposeCompletionKind"));
    assert!(!SOURCE.contains("pub(crate) enum AsyncDisposableStackDisposeCompletionKind"));

    let declaration = bounded(
        SOURCE,
        "enum AsyncDisposableStackDisposeCompletionKind {",
        "impl AsyncDisposableStackDisposeCompletionKind {",
    );
    assert_eq!(
        declaration
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && *line != "}")
            .collect::<Vec<_>>(),
        ["Normal,", "Throw,"]
    );
    assert!(!declaration.contains("#[derive("));
    assert!(SOURCE.contains(
        "#[must_use = \"a loaded disposal completion kind must be consumed by an exhaustive route\"]\nstruct AsyncDisposableStackDisposeCompletionKindLocal(u32);"
    ));
}

#[test]
fn completion_kind_serialization_is_exhaustive_and_exact() {
    let projection = bounded(SOURCE, "    fn word(&self) -> u64 {", "}\n\n#[must_use");
    assert_eq!(
        projection
            .lines()
            .map(str::trim)
            .filter(|line| line.starts_with("Self::"))
            .collect::<Vec<_>>(),
        ["Self::Normal => 0,", "Self::Throw => 1,"]
    );
    assert!(!projection.contains("_ =>"));
}

#[test]
fn completion_kind_heap_word_has_one_typed_store_and_strict_load_authority() {
    assert_eq!(
        SOURCE
            .matches("ASYNC_DISPOSABLE_STACK_DISPOSE_COMPLETION_KIND_OFFSET")
            .count(),
        3
    );
    assert!(!SOURCE.contains("ASYNC_DISPOSABLE_STACK_DISPOSE_HAS_ERROR_OFFSET"));
    assert!(!SOURCE.contains("has_error_local"));

    let store = bounded(
        SOURCE,
        "    fn emit_store_async_disposable_stack_dispose_completion_kind(",
        "    fn emit_load_async_disposable_stack_dispose_completion_kind(",
    );
    assert!(store.contains("completion_kind: AsyncDisposableStackDisposeCompletionKind,"));
    assert!(store.contains("completion_kind.word(),"));

    let load = bounded(
        SOURCE,
        "    fn emit_load_async_disposable_stack_dispose_completion_kind(",
        "    fn emit_async_disposable_stack_dispose_completion_kind_is(",
    );
    assert!(load.contains(") -> AsyncDisposableStackDisposeCompletionKindLocal {"));
    assert_eq!(load.matches("Instruction::Unreachable").count(), 1);
    assert!(load.contains("AsyncDisposableStackDisposeCompletionKind::Normal,"));
    assert!(load.contains("AsyncDisposableStackDisposeCompletionKind::Throw,"));
    assert!(!load.contains("_ =>"));
}

#[test]
fn initialization_suppression_and_settlement_name_their_completion_routes() {
    assert_eq!(
        SOURCE
            .matches("self.emit_store_async_disposable_stack_dispose_completion_kind(")
            .count(),
        2
    );
    assert_eq!(
        SOURCE
            .matches(".emit_load_async_disposable_stack_dispose_completion_kind(")
            .count(),
        2
    );
    assert_eq!(
        SOURCE
            .matches("self.emit_async_disposable_stack_dispose_completion_kind_is(")
            .count(),
        2
    );

    let initialization = bounded(
        SOURCE,
        "        self.emit_heap_alloc_const(ASYNC_DISPOSABLE_STACK_DISPOSE_STATE_SIZE, function)?;",
        "        self.emit_async_disposable_stack_dispose_step(dispose_state_local, function)?;",
    );
    assert!(initialization.contains("AsyncDisposableStackDisposeCompletionKind::Normal,"));

    let settlement = bounded(
        SOURCE,
        "        function.instruction(&Instruction::LocalGet(suspended_local));",
        "        for local in [",
    );
    assert!(settlement.contains("AsyncDisposableStackDisposeCompletionKind::Normal,"));
    assert!(settlement.contains("PromiseSettlement::Fulfill,"));
    assert!(settlement.contains("PromiseSettlement::Reject,"));

    let suppression = bounded(
        SOURCE,
        "    fn emit_async_disposable_stack_record_error(",
        "    fn emit_store_async_disposable_stack_dispose_completion_kind(",
    );
    assert_eq!(
        suppression
            .matches("AsyncDisposableStackDisposeCompletionKind::Throw,")
            .count(),
        2
    );
    assert!(suppression.contains("self.emit_alloc_suppressed_error_instance_from_locals("));
}
