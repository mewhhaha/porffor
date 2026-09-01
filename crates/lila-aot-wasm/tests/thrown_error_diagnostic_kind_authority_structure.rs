const ERRORS_SOURCE: &str = include_str!("../src/builtins/errors.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/thrown-error-diagnostic-kind-authority.md");
const TASK: &str = include_str!("../../../tasks/24-globals-errors-annexb-host.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn normalized(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn assert_before(source: &str, earlier: &str, later: &str) {
    let earlier_offset = source
        .find(earlier)
        .unwrap_or_else(|| panic!("missing earlier operation `{earlier}`"));
    let later_offset = source
        .find(later)
        .unwrap_or_else(|| panic!("missing later operation `{later}`"));
    assert!(
        earlier_offset < later_offset,
        "`{earlier}` must precede `{later}`"
    );
}

#[test]
fn diagnostic_publisher_accepts_only_native_error_kind() {
    let publisher = bounded(
        ERRORS_SOURCE,
        "    fn emit_set_thrown_error_text(",
        "    pub(crate) fn emit_throw_runtime_error(",
    );
    assert!(publisher.contains("kind: NativeErrorKind,"));
    assert!(publisher.contains("let name = kind.as_str();"));
    assert!(publisher.contains("message: Option<&str>,"));
    assert!(!publisher.contains("name: &str"));
    assert!(!publisher.contains("native_error_kind("));
    assert_eq!(publisher.matches("GlobalSet(").count(), 2);
    let normalized_publisher = normalized(publisher);
    assert_eq!(
        normalized_publisher
            .matches("throw_error_name_global_index(self.uses_heap")
            .count(),
        1
    );
    assert_eq!(
        normalized_publisher
            .matches("throw_error_message_global_index(self.uses_heap")
            .count(),
        1
    );
    assert_before(
        publisher,
        "let name = kind.as_str();",
        "self.strings.payload(name)",
    );
}

#[test]
fn all_three_producers_forward_the_error_kind_they_already_own() {
    assert_eq!(
        ERRORS_SOURCE.matches("emit_set_thrown_error_text(").count(),
        4,
        "one publisher and three producers own every mention"
    );
    assert_eq!(
        ERRORS_SOURCE
            .matches("self.emit_set_thrown_error_text(kind, Some(message), function);")
            .count(),
        2,
        "global-prototype and resolved-prototype paths forward their existing kind"
    );
    assert_eq!(
        ERRORS_SOURCE
            .matches("self.emit_set_thrown_error_text(NativeErrorKind::TypeError, None, function);")
            .count(),
        1,
        "message-less TypeError names its closed kind"
    );
    for forbidden in [
        "emit_set_thrown_error_text(name",
        "emit_set_thrown_error_text(TYPE_ERROR_NAME",
        "emit_set_thrown_error_text(RANGE_ERROR_NAME",
        "emit_set_thrown_error_text(URI_ERROR_NAME",
    ] {
        assert!(!ERRORS_SOURCE.contains(forbidden), "found `{forbidden}`");
    }
}

#[test]
fn diagnostic_publication_follows_object_creation_and_precedes_throw_completion() {
    let global_prototype_path = bounded(
        ERRORS_SOURCE,
        "    fn emit_throw_runtime_error_kind(",
        "    pub(crate) fn emit_throw_current_function_realm_error(",
    );
    assert_before(
        global_prototype_path,
        "self.emit_runtime_error_object(kind, message, payload_local, tag_local, function)?;",
        "self.emit_set_thrown_error_text(kind, Some(message), function);",
    );
    assert_before(
        global_prototype_path,
        "self.emit_set_thrown_error_text(kind, Some(message), function);",
        "self.set_completion_kind_with_aux(",
    );

    let resolved_prototype_path = bounded(
        ERRORS_SOURCE,
        "    fn emit_throw_runtime_error_with_prototype_local_kind(",
        "    pub(crate) fn emit_capture_throw_error_name(",
    );
    assert_before(
        resolved_prototype_path,
        "self.emit_alloc_plain_object_with_prototype(Some(prototype_local), None, function)?;",
        "self.emit_set_thrown_error_text(kind, Some(message), function);",
    );
    assert_before(
        resolved_prototype_path,
        "self.emit_set_thrown_error_text(kind, Some(message), function);",
        "self.set_completion_kind_with_aux(",
    );
}

#[test]
fn contract_and_task_record_the_authority_and_non_claim() {
    let normalized_contract = normalized(CONTRACT);
    let normalized_task = normalized(TASK);
    for evidence in [
        "NativeErrorKind",
        "published diagnostic name",
        "interchangeable raw string",
        "does not change emitted Wasm",
        "user-thrown values",
    ] {
        let normalized_evidence = normalized(evidence);
        assert!(
            normalized_contract.contains(&normalized_evidence),
            "contract evidence `{evidence}`"
        );
        assert!(
            normalized_task.contains(&normalized_evidence),
            "task evidence `{evidence}`"
        );
    }
}
