const OPERATIONS_SOURCE: &str = include_str!("../src/operations.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

#[test]
fn all_three_equality_algorithms_share_the_mixed_bigint_representation_path() {
    for (start, end) in [
        (
            "    pub(crate) fn emit_tagged_payload_same_value_i32(",
            "    pub(crate) fn emit_tagged_payload_same_value_zero_i32(",
        ),
        (
            "    pub(crate) fn emit_tagged_payload_same_value_zero_i32(",
            "    pub(crate) fn emit_tagged_payload_equality_i32(",
        ),
        (
            "    pub(crate) fn emit_tagged_payload_equality_i32(",
            "    fn emit_differently_tagged_bigint_equality_i32(",
        ),
    ] {
        let equality_algorithm = bounded(OPERATIONS_SOURCE, start, end);
        assert_eq!(
            equality_algorithm
                .matches("emit_differently_tagged_bigint_equality_i32(")
                .count(),
            1,
            "{start} must compare differently represented BigInts by value"
        );
    }
}

#[test]
fn mixed_bigint_representation_equality_requires_two_bigint_tags() {
    let equality = bounded(
        OPERATIONS_SOURCE,
        "    fn emit_differently_tagged_bigint_equality_i32(",
        "    fn emit_heap_bigint_equality_i32(",
    );

    assert_eq!(equality.matches("emit_is_bigint_tag_i32(").count(), 2);
    assert_eq!(
        equality.matches("emit_mixed_bigint_equality_i32(").count(),
        1
    );
    assert!(equality.contains("Instruction::I32And"));
    assert!(equality.contains("Instruction::I32Const(0)"));
}
