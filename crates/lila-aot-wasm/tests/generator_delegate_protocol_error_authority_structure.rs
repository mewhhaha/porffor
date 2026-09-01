const DELEGATION_SOURCE: &str = include_str!("../src/generator_delegation.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

const PROTOCOL_ERRORS: [(&str, &str, usize); 8] = [
    ("TargetNotIterable", "yield* target is not iterable", 2),
    (
        "IteratorMethodNotCallable",
        "yield* iterator method must be callable",
        2,
    ),
    (
        "IteratorMethodResultNotObject",
        "yield* iterator method must return object",
        2,
    ),
    (
        "IteratorResultNotObject",
        "yield* iterator result must be object",
        2,
    ),
    (
        "MissingThrowMethod",
        "yield* iterator has no throw method",
        3,
    ),
    (
        "ReturnMethodNotCallable",
        "yield* return method must be callable",
        3,
    ),
    (
        "ThrowMethodNotCallable",
        "yield* throw method must be callable",
        2,
    ),
    (
        "NextMethodNotCallable",
        "yield* next method must be callable",
        2,
    ),
];

#[test]
fn generator_delegation_protocol_errors_have_one_private_closed_domain() {
    assert_eq!(
        DELEGATION_SOURCE
            .matches("enum GeneratorDelegateProtocolError {")
            .count(),
        1
    );
    assert!(!DELEGATION_SOURCE.contains("pub enum GeneratorDelegateProtocolError"));
    assert!(!DELEGATION_SOURCE.contains("pub(crate) enum GeneratorDelegateProtocolError"));

    let domain = bounded(
        DELEGATION_SOURCE,
        "enum GeneratorDelegateProtocolError {",
        "impl GeneratorDelegateProperty {",
    );
    assert_eq!(
        domain
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && *line != "}")
            .collect::<Vec<_>>(),
        PROTOCOL_ERRORS
            .map(|(variant, _, _)| format!("{variant},"))
            .as_slice()
    );
    assert!(!domain.contains("#[derive("));
}

#[test]
fn every_generator_delegation_error_producer_names_its_protocol_failure() {
    let producers = bounded(
        DELEGATION_SOURCE,
        "impl<'a> FunctionBuilder<'a> {",
        "    #[allow(clippy::too_many_arguments)]\n    fn emit_generator_delegate_call(",
    );
    assert_eq!(
        producers
            .matches("GeneratorDelegateProtocolError::")
            .count(),
        18
    );
    for (variant, message, producer_count) in PROTOCOL_ERRORS {
        assert_eq!(
            producers
                .matches(&format!("GeneratorDelegateProtocolError::{variant}"))
                .count(),
            producer_count,
            "unexpected producer count for `{variant}`"
        );
        assert!(
            !producers.contains(message),
            "producer bypasses the protocol-error authority with `{message}`"
        );
    }
}

#[test]
fn the_protocol_error_projection_is_exhaustive_and_owns_the_diagnostics() {
    let projection = bounded(
        DELEGATION_SOURCE,
        "    fn emit_generator_delegate_protocol_error(",
        "        self.emit_throw_runtime_error(",
    );
    assert!(projection.contains("protocol_error: GeneratorDelegateProtocolError,"));
    assert!(projection.contains("let message = match protocol_error {"));
    assert!(!projection.contains("_ =>"));

    for (variant, message, _) in PROTOCOL_ERRORS {
        assert_eq!(
            projection
                .matches(&format!("GeneratorDelegateProtocolError::{variant} =>"))
                .count(),
            1,
            "projection must handle `{variant}` exactly once"
        );
        assert_eq!(
            DELEGATION_SOURCE.matches(message).count(),
            1,
            "diagnostic `{message}` must exist only in the projection"
        );
    }
    assert_eq!(
        DELEGATION_SOURCE
            .matches("self.emit_throw_runtime_error(")
            .count(),
        1
    );
}

#[test]
fn shared_delegation_checks_require_the_closed_error_authority() {
    let call = bounded(
        DELEGATION_SOURCE,
        "    fn emit_generator_delegate_call(",
        "    #[allow(clippy::too_many_arguments)]\n    fn emit_generator_delegate_property_read(",
    );
    assert!(call.contains("protocol_error: GeneratorDelegateProtocolError,"));
    assert!(
        call.contains("self.emit_generator_delegate_protocol_error(protocol_error, function)?;")
    );
    assert!(!call.contains("message: &str"));

    let object_check = bounded(
        DELEGATION_SOURCE,
        "    fn emit_require_generator_delegate_object(",
        "    fn emit_generator_delegate_protocol_error(",
    );
    assert!(object_check.contains("protocol_error: GeneratorDelegateProtocolError,"));
    assert!(object_check
        .contains("self.emit_generator_delegate_protocol_error(protocol_error, function)?;"));
    assert!(!object_check.contains("message: &str"));
}
