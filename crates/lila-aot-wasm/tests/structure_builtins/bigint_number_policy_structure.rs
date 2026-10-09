const OPERATIONS_SOURCE: &str = include_str!("../../src/operations.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

fn without_whitespace(source: &str) -> String {
    source.chars().filter(|ch| !ch.is_whitespace()).collect()
}

#[test]
fn bigint_number_policy_is_closed_and_projects_only_at_the_number_branch() {
    let policy = bounded(
        OPERATIONS_SOURCE,
        "pub(crate) enum BigIntNumberPolicy {",
        "\n}",
    );
    assert_eq!(
        without_whitespace(policy),
        "RejectNumber,NumberToBigInt,",
        "the Number-admission domain must remain exactly two-state"
    );
    assert!(!OPERATIONS_SOURCE.contains(")]\npub(crate) enum BigIntNumberPolicy"));
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq"] {
        assert!(!OPERATIONS_SOURCE.contains(&format!("impl {capability} for BigIntNumberPolicy")));
    }
}
