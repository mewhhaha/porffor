const PROMISE_SOURCE: &str = include_str!("../../src/builtins/promise.rs");

const BUILTINS_SOURCE: &str = include_str!("../../src/builtins/mod.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker after: {start}"))
        .0
}

#[test]
fn complete_step_kind_is_one_closed_domain_with_one_boolean_projection() {
    let attributes = PROMISE_SOURCE
        .split_once("pub(crate) enum AsyncGeneratorCompleteStepKind")
        .expect("closed enum declaration")
        .0
        .rsplit("\n\n")
        .next()
        .expect("enum declaration prefix");
    assert!(!attributes.contains("Default"));
    let declaration = bounded(
        PROMISE_SOURCE,
        "pub(crate) enum AsyncGeneratorCompleteStepKind {",
        "}\n",
    );
    let variants = declaration
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(variants, ["Yielded,", "Completed,"]);
    assert!(!declaration.contains("bool"));
    assert!(!declaration.contains("Default"));
    assert!(!PROMISE_SOURCE.contains("impl Default for AsyncGeneratorCompleteStepKind"));
    assert!(!PROMISE_SOURCE.contains("impl AsyncGeneratorCompleteStepKind"));
    let promise_reexport = bounded(BUILTINS_SOURCE, "pub(crate) use promise::{", "};");
    assert_eq!(
        promise_reexport
            .matches("AsyncGeneratorCompleteStepKind,")
            .count(),
        1
    );
    assert_eq!(
        BUILTINS_SOURCE
            .matches("AsyncGeneratorCompleteStepKind")
            .count(),
        1
    );
}
