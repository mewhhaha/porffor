const COLLECTIONS_SOURCE: &str = include_str!("../src/builtins/collections.rs");

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

fn group_by_body() -> &'static str {
    bounded(
        COLLECTIONS_SOURCE,
        "    fn emit_group_by(",
        "    pub(crate) fn emit_map_prototype_clear(",
    )
}

#[test]
fn group_by_result_has_no_equality_projection() {
    let declaration = bounded(
        COLLECTIONS_SOURCE,
        "#[derive(Clone, Copy)]\nenum GroupByResult {",
        "#[derive(Clone, Copy)]\nenum MapCollectionKind",
    );
    let variants = declaration
        .split_once('}')
        .expect("GroupBy result end")
        .0
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(variants, ["Map,", "Object,"]);

    let body = group_by_body();
    assert_eq!(body.matches("match result_kind").count(), 11);
    for forbidden in ["result_kind ==", "result_kind !=", "is_map", "is_object"] {
        assert!(
            !body.contains(forbidden),
            "grouping semantics must not collapse to {forbidden}"
        );
    }
}

#[test]
fn group_by_result_producers_are_exact() {
    let wrappers = bounded(
        COLLECTIONS_SOURCE,
        "    pub(crate) fn emit_map_group_by(",
        "    fn emit_group_by(",
    );
    let normalized = without_whitespace(wrappers);
    for (method, variant) in [("map", "Map"), ("object", "Object")] {
        let call = format!("emit_group_by(GroupByResult::{variant},function)");
        assert_eq!(
            normalized.matches(&call).count(),
            1,
            "{method} groupBy must produce its matching result kind"
        );
    }
    assert_eq!(normalized.matches("emit_group_by(").count(), 2);
}

#[test]
fn group_by_result_semantic_pairings_are_exhaustive() {
    let body = group_by_body();
    assert_eq!(body.matches("GroupByResult::Map").count(), 11);
    assert_eq!(body.matches("GroupByResult::Object").count(), 11);
    assert_eq!(body.matches("Map.groupBy").count(), 7);
    assert_eq!(body.matches("Object.groupBy").count(), 7);
    assert_eq!(body.matches("emit_tagged_to_primitive_locals(").count(), 1);
    assert_eq!(body.matches("emit_find_map_entry(").count(), 1);
    assert_eq!(
        body.matches("emit_alloc_plain_object_with_prototype(")
            .count(),
        2
    );
    assert_eq!(
        body.matches("CollectionDataReceiverKind::Map.brand()")
            .count(),
        1
    );
}
