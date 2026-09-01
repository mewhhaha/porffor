const SOURCE: &str = include_str!("../src/planning.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn assert_single_selection(source: &str, expected: &str) {
    assert_eq!(
        source
            .matches("shape_accessor_references_function(")
            .count(),
        1
    );
    for variant in ["Getter", "Setter", "GetterOrSetter"] {
        let selection = format!("ShapeAccessorReferenceSelection::{variant},");
        assert_eq!(
            source.matches(&selection).count(),
            usize::from(variant == expected),
            "expected only `{expected}` in producer"
        );
    }
}

#[test]
fn shape_accessor_reference_selection_is_a_private_three_variant_domain() {
    let type_declaration = bounded(
        SOURCE,
        "#[derive(Clone, Copy, Debug, PartialEq, Eq)]\nenum ShapeAccessorReferenceSelection {",
        "\n\nfn shape_accessor_references_function(",
    );
    let variants = type_declaration
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && *line != "}")
        .collect::<Vec<_>>();

    assert_eq!(variants, ["Getter,", "Setter,", "GetterOrSetter,"]);
    assert!(!type_declaration.contains("Default"));
    assert!(!SOURCE.contains("pub enum ShapeAccessorReferenceSelection"));
    assert!(!SOURCE.contains("pub(crate) enum ShapeAccessorReferenceSelection"));
    assert!(!SOURCE.contains("pub(crate) fn shape_accessor_references_function"));
}

#[test]
fn static_and_dynamic_shape_accessors_project_the_selection_directly() {
    let consumer = bounded(
        SOURCE,
        "fn shape_accessor_references_function(",
        "pub(crate) fn shape_data_references_function(",
    );

    assert_eq!(
        consumer
            .matches("selection: ShapeAccessorReferenceSelection,")
            .count(),
        2
    );
    assert_eq!(consumer.matches("match selection {").count(), 2);
    for variant in ["Getter", "Setter", "GetterOrSetter"] {
        let selection = format!("ShapeAccessorReferenceSelection::{variant} =>");
        assert_eq!(
            consumer.matches(&selection).count(),
            2,
            "variant `{variant}`"
        );
    }
    assert_eq!(
        consumer
            .matches("any_accessor(prototype, target, selection)")
            .count(),
        1
    );
    assert_eq!(
        consumer
            .matches("any_accessor(shape, target, selection)")
            .count(),
        1
    );
    assert!(!consumer.contains("include_getter"));
    assert!(!consumer.contains("include_setter"));
    assert!(!consumer.contains("selection: bool"));
    assert!(!consumer.contains("matches!(selection"));
    assert!(!consumer.contains("if selection"));
    assert!(!consumer.contains("=> true"));
    assert!(!consumer.contains("=> false"));
    assert!(!consumer.contains("_ =>"));
    assert!(!consumer.contains("unreachable!"));
}

#[test]
fn exactly_seven_product_producers_choose_their_accessor_selection() {
    let producers = bounded(
        SOURCE,
        "        ExprIr::OptionalPropertyChain {\n            target: object,\n            chain,",
        "        ExprIr::StringCharCodeAt {\n            target: object,",
    );

    assert_eq!(
        producers
            .matches("shape_accessor_references_function(")
            .count(),
        7
    );
    assert_eq!(
        producers
            .matches("ShapeAccessorReferenceSelection::Getter,")
            .count(),
        2
    );
    assert_eq!(
        producers
            .matches("ShapeAccessorReferenceSelection::Setter,")
            .count(),
        2
    );
    assert_eq!(
        producers
            .matches("ShapeAccessorReferenceSelection::GetterOrSetter,")
            .count(),
        3
    );
    assert!(!producers.contains("include_getter"));
    assert!(!producers.contains("include_setter"));
    assert!(!producers.contains("_ =>"));
    assert!(!producers.contains("unreachable!"));

    assert_single_selection(
        producers
            .split_once("        ExprIr::PropertyRead {")
            .expect("missing property-read producer after optional-chain producer")
            .0,
        "Getter",
    );
    assert_single_selection(
        bounded(
            producers,
            "        ExprIr::PropertyRead {",
            "        ExprIr::DeleteProperty {",
        ),
        "Getter",
    );
    assert_single_selection(
        bounded(
            producers,
            "        ExprIr::OrdinaryPropertyAssignment(assignment) => {",
            "        ExprIr::OrdinaryPropertyLogicalAssignment(assignment) => {",
        ),
        "Setter",
    );
    assert_single_selection(
        bounded(
            producers,
            "        ExprIr::OrdinaryPropertyLogicalAssignment(assignment) => {",
            "        ExprIr::OrdinaryPropertyNumericUpdate(update) => {",
        ),
        "GetterOrSetter",
    );
    assert_single_selection(
        bounded(
            producers,
            "        ExprIr::OrdinaryPropertyNumericUpdate(update) => {",
            "        ExprIr::OrdinaryPropertyEagerCompoundAssignment(mutation) => {",
        ),
        "GetterOrSetter",
    );
    assert_single_selection(
        bounded(
            producers,
            "        ExprIr::OrdinaryPropertyEagerCompoundAssignment(mutation) => {",
            "        ExprIr::PropertyWrite {",
        ),
        "GetterOrSetter",
    );
    assert_single_selection(
        producers
            .split_once("        ExprIr::PropertyWrite {")
            .expect("missing property-write producer")
            .1,
        "Setter",
    );
}
