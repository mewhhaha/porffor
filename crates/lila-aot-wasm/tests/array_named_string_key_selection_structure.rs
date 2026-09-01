const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const OBJECT_SOURCE: &str = include_str!("../src/builtins/object.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/array-named-string-key-selection.md");
const TASK: &str = include_str!("../../../tasks/16-arrays-and-array-builtins.md");

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
fn array_named_string_key_selection_is_a_closed_two_variant_domain() {
    let authority_header = bounded(
        ARRAY_SOURCE,
        "fn array_descriptor_field<T>",
        "\n\npub(crate) enum ArraySortOutput",
    );
    let declaration = bounded(
        ARRAY_SOURCE,
        "enum ArrayNamedStringKeySelection {",
        "\n}\n\npub(crate) enum ArraySortOutput",
    );
    let variants = declaration
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();

    assert_eq!(variants, ["All,", "EnumerableOnly,"]);
    assert!(!authority_header.contains("#[derive"));
    for capability in [
        "Clone",
        "Copy",
        "Debug",
        "Default",
        "PartialEq",
        "Eq",
        "PartialOrd",
        "Ord",
        "Hash",
    ] {
        assert!(
            !ARRAY_SOURCE.contains(&format!(
                "impl {capability} for ArrayNamedStringKeySelection"
            )),
            "named-string key selection must not implement {capability}"
        );
    }
}

#[test]
fn both_array_named_string_key_consumers_project_the_mode_exhaustively() {
    let consumers = bounded(
        ARRAY_SOURCE,
        "    fn emit_array_named_string_props_count(",
        "    pub(super) fn emit_array_all_named_string_props_count(",
    );

    assert_eq!(
        consumers
            .matches("selection: ArrayNamedStringKeySelection,")
            .count(),
        2
    );
    assert_eq!(consumers.matches("match &selection {").count(), 4);
    assert_eq!(
        consumers
            .matches("ArrayNamedStringKeySelection::All")
            .count(),
        4
    );
    assert_eq!(
        consumers
            .matches("ArrayNamedStringKeySelection::EnumerableOnly")
            .count(),
        4
    );
    assert!(!consumers.contains("enumerable_only"));
    assert!(!consumers.contains("selection: bool"));
    assert!(!consumers.contains("matches!(selection"));
    assert!(!consumers.contains("if selection"));
    assert!(!consumers.contains("match selection"));
    assert!(!consumers.contains("selection.clone()"));
    assert!(!consumers.contains("selection =="));
    assert!(!consumers.contains("selection !="));
    assert!(!consumers.contains("=> true"));
    assert!(!consumers.contains("=> false"));
    assert!(!consumers.contains("_ =>"));
    assert!(!consumers.contains("unreachable!"));

    let count = bounded(
        consumers,
        "        selection: ArrayNamedStringKeySelection,",
        "    fn emit_array_named_string_props_write_keys(",
    );
    assert_eq!(count.matches("match &selection {").count(), 2);

    let write = bounded(
        ARRAY_SOURCE,
        "    fn emit_array_named_string_props_write_keys(",
        "    pub(super) fn emit_array_all_named_string_props_count(",
    );
    assert_eq!(write.matches("match &selection {").count(), 2);
}

#[test]
fn exactly_four_object_producers_choose_their_named_selection() {
    assert!(!OBJECT_SOURCE.contains("ArrayNamedStringKeySelection"));
    assert!(!OBJECT_SOURCE.contains("self.emit_array_named_string_props_count("));
    assert!(!OBJECT_SOURCE.contains("self.emit_array_named_string_props_write_keys("));

    let own_property_names = bounded(
        OBJECT_SOURCE,
        "    pub(super) fn compile_object_get_own_property_names_builtin(",
        "    pub(super) fn compile_object_get_own_property_symbols_builtin(",
    );
    assert_eq!(
        own_property_names
            .matches("self.emit_array_all_named_string_props_")
            .count(),
        2,
    );
    assert!(!own_property_names.contains("emit_array_enumerable_named_string_props_"));

    let keys = bounded(
        OBJECT_SOURCE,
        "    pub(super) fn compile_object_keys_builtin(",
        "    pub(super) fn compile_object_is_builtin(",
    );
    assert_eq!(
        keys.matches("self.emit_array_enumerable_named_string_props_")
            .count(),
        2,
    );
    assert!(!keys.contains("emit_array_all_named_string_props_"));

    for evidence in [CONTRACT, TASK] {
        assert!(evidence.contains("capability-free `ArrayNamedStringKeySelection`"));
    }
}

#[test]
fn raw_selection_is_private_to_four_fixed_array_operations() {
    assert!(!ARRAY_SOURCE.contains("pub(super) enum ArrayNamedStringKeySelection"));
    assert!(!ARRAY_SOURCE.contains("pub(crate) enum ArrayNamedStringKeySelection"));
    assert_eq!(
        ARRAY_SOURCE
            .matches("fn emit_array_named_string_props_count(")
            .count(),
        1
    );
    assert_eq!(
        ARRAY_SOURCE
            .matches("fn emit_array_named_string_props_write_keys(")
            .count(),
        1
    );
    for (wrapper, variant) in [
        ("emit_array_all_named_string_props_count", "All"),
        (
            "emit_array_enumerable_named_string_props_count",
            "EnumerableOnly",
        ),
        ("emit_array_all_named_string_props_write_keys", "All"),
        (
            "emit_array_enumerable_named_string_props_write_keys",
            "EnumerableOnly",
        ),
    ] {
        let wrapper = bounded(
            ARRAY_SOURCE,
            &format!("    pub(super) fn {wrapper}("),
            "\n    }",
        );
        assert_eq!(
            wrapper
                .matches("self.emit_array_named_string_props_")
                .count(),
            1
        );
        assert_eq!(
            wrapper
                .matches(&format!("ArrayNamedStringKeySelection::{variant}"))
                .count(),
            1
        );
    }
}
