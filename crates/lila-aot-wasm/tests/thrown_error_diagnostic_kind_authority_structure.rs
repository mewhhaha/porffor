const PARENT_ERRORS_SOURCE: &str = include_str!("../src/builtins/errors.rs");
const ERRORS_SOURCE: &str = include_str!("../src/builtins/errors/runtime_error.rs");
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
    assert!(publisher.contains("self.emit_interned_string_reference(kind.as_str(), function)?"));
    assert!(publisher.contains("message: Option<RuntimeErrorMessage>,"));
    assert!(!publisher.contains("name: &str"));
    assert!(!publisher.contains("native_error_kind("));
    for role in ["Name", "Message"] {
        assert_eq!(
            publisher
                .matches(&format!(
                    "self.emit_clear_throw_diagnostic(ThrowDiagnosticRole::{role}, function);"
                ))
                .count(),
            1
        );
        assert_eq!(
            publisher
                .matches(&format!(
                    "self.emit_store_throw_diagnostic(ThrowDiagnosticRole::{role},"
                ))
                .count(),
            1
        );
    }
    assert_before(
        publisher,
        "self.emit_interned_string_reference(kind.as_str(), function)?",
        "self.emit_store_throw_diagnostic(ThrowDiagnosticRole::Name, &name, function);",
    );
    assert_before(
        publisher,
        "self.emit_runtime_error_message_reference(message, function)?",
        "self.emit_store_throw_diagnostic(ThrowDiagnosticRole::Message, &text, function);",
    );
}

#[test]
fn all_three_producers_forward_the_error_kind_they_already_own() {
    assert!(
        !PARENT_ERRORS_SOURCE.contains("emit_set_thrown_error_text("),
        "the private runtime-error child owns the publisher and all producers"
    );
    assert_eq!(
        ERRORS_SOURCE.matches("emit_set_thrown_error_text(").count(),
        4,
        "one publisher and three producers own every mention"
    );
    assert_eq!(
        ERRORS_SOURCE
            .matches("self.emit_set_thrown_error_text(kind, Some(message), function)?;")
            .count(),
        2,
        "global-prototype and resolved-prototype paths forward their existing kind"
    );
    assert_eq!(
        ERRORS_SOURCE
            .matches(
                "self.emit_set_thrown_error_text(NativeErrorKind::TypeError, None, function)?;"
            )
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
        "    pub(crate) fn emit_throw_runtime_error(",
        "    pub(crate) fn emit_throw_current_function_realm_error(",
    );
    assert_before(
        global_prototype_path,
        "self.emit_runtime_error_object(kind, message, &value, function)?;",
        "self.emit_set_thrown_error_text(kind, Some(message), function)?;",
    );
    assert_before(
        global_prototype_path,
        "self.emit_set_thrown_error_text(kind, Some(message), function)?;",
        "result.set_throw(&value, function);",
    );

    let resolved_prototype_path = bounded(
        ERRORS_SOURCE,
        "    pub(crate) fn emit_throw_runtime_error_with_prototype(",
        "    pub(crate) fn emit_throw_current_function_realm_type_error(",
    );
    assert_before(
        resolved_prototype_path,
        "self.emit_fresh_native_error_object(",
        "self.emit_set_thrown_error_text(kind, Some(message), function)?;",
    );
    assert_before(
        resolved_prototype_path,
        "self.emit_set_thrown_error_text(kind, Some(message), function)?;",
        "result.set_throw(&value, function);",
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
