const DISPOSABLE_STACK_SOURCE: &str = include_str!("../src/builtins/disposable_stack.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

const VARIANTS: [&str; 7] = [
    "UseValueNotObject",
    "UseValueNotDisposable",
    "AdoptCallbackNotCallable",
    "DeferCallbackNotCallable",
    "DisposeMethodNotCallable",
    "ReceiverNotObject",
    "ReceiverMissingDisposableState",
];

#[test]
fn disposable_stack_type_errors_have_one_closed_domain() {
    assert_eq!(
        DISPOSABLE_STACK_SOURCE
            .matches("enum DisposableStackTypeError {")
            .count(),
        1
    );
    let domain = bounded(
        DISPOSABLE_STACK_SOURCE,
        "enum DisposableStackTypeError {",
        "impl<'a> FunctionBuilder<'a> {",
    );
    assert_eq!(
        domain
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && *line != "}")
            .collect::<Vec<_>>(),
        VARIANTS.map(|variant| format!("{variant},"))
    );
    assert!(!domain.contains("_ =>"));
}

#[test]
fn every_disposable_stack_type_error_producer_names_one_variant() {
    let producers = bounded(
        DISPOSABLE_STACK_SOURCE,
        "impl<'a> FunctionBuilder<'a> {",
        "    fn emit_disposable_stack_type_error(",
    );
    assert_eq!(
        producers
            .matches("self.emit_disposable_stack_type_error(")
            .count(),
        VARIANTS.len()
    );
    for variant in VARIANTS {
        assert_eq!(
            producers
                .matches(&format!("DisposableStackTypeError::{variant}"))
                .count(),
            1,
            "producer must select `{variant}` exactly once"
        );
    }
    for message in [
        "DisposableStack.prototype.use value is not an object",
        "DisposableStack.prototype.use value is not disposable",
        "DisposableStack.prototype.adopt onDispose is not callable",
        "DisposableStack.prototype.defer onDispose is not callable",
        "DisposableStack.prototype.use dispose method is not callable",
        "DisposableStack method receiver is not an object",
        "DisposableStack method receiver does not have [[DisposableState]]",
    ] {
        assert!(
            !producers.contains(message),
            "producer must not bypass the closed domain with `{message}`"
        );
    }
}

#[test]
fn the_type_error_emitter_projects_every_variant_exhaustively() {
    let emitter = bounded(
        DISPOSABLE_STACK_SOURCE,
        "    fn emit_disposable_stack_type_error(",
        "        self.emit_throw_current_function_realm_type_error(",
    );
    assert!(emitter.contains("error: DisposableStackTypeError,"));
    assert!(!emitter.contains("message: &'static str"));
    assert!(emitter.contains("let message = match error {"));
    for variant in VARIANTS {
        assert_eq!(
            emitter
                .matches(&format!("DisposableStackTypeError::{variant} =>"))
                .count(),
            1,
            "projection must handle `{variant}` exactly once"
        );
    }
    for message in [
        "DisposableStack.prototype.use value is not an object",
        "DisposableStack.prototype.use value is not disposable",
        "DisposableStack.prototype.adopt onDispose is not callable",
        "DisposableStack.prototype.defer onDispose is not callable",
        "DisposableStack.prototype.use dispose method is not callable",
        "DisposableStack method receiver is not an object",
        "DisposableStack method receiver does not have [[DisposableState]]",
    ] {
        assert_eq!(
            emitter.matches(message).count(),
            1,
            "projection must preserve `{message}` exactly once"
        );
    }
    assert!(!emitter.contains("_ =>"));
}
