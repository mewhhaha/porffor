use std::fs;
use std::path::Path;

const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const OBJECT_SOURCE: &str = include_str!("../src/builtins/object.rs");
const ENUMERABLE_OWN_PROPERTIES_SOURCE: &str =
    include_str!("../src/builtins/object/enumerable_own_properties.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/array-named-string-key-selection.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn count_in_rust_sources(dir: &Path, needle: &str) -> usize {
    fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .map(|path| {
            if path.is_dir() {
                return count_in_rust_sources(&path, needle);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return 0;
            }
            fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
                .matches(needle)
                .count()
        })
        .sum()
}

#[test]
fn retired_named_string_key_selection_cannot_return() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for retired in [
        "ArrayNamedStringKeySelection",
        "emit_array_named_string_props_count",
        "emit_array_named_string_props_write_keys",
        "emit_array_enumerable_named_string_props_",
    ] {
        assert_eq!(
            count_in_rust_sources(&source_root, retired),
            0,
            "retired named-string selection `{retired}` must stay deleted"
        );
    }
    for owner in [
        "fn emit_array_all_named_string_props_count(",
        "fn emit_array_all_named_string_props_write_keys(",
    ] {
        assert_eq!(count_in_rust_sources(&source_root, owner), 1, "{owner}");
    }
}

#[test]
fn count_and_write_phases_share_one_unconditional_string_key_filter() {
    let count = bounded(
        ARRAY_SOURCE,
        "    pub(super) fn emit_array_all_named_string_props_count(",
        "    pub(super) fn emit_array_all_named_string_props_write_keys(",
    );
    let write = bounded(
        ARRAY_SOURCE,
        "    pub(super) fn emit_array_all_named_string_props_write_keys(",
        "    pub(crate) fn emit_array_delete_property_key(",
    );

    assert!(count.starts_with(concat!(
        "\n        &mut self,\n",
        "        array_local: u32,\n",
        "        count_local: u32,\n",
        "        function: &mut Function,\n",
        "    ) {\n",
    )));
    assert!(write.starts_with(concat!(
        "\n        &mut self,\n",
        "        array_local: u32,\n",
        "        result_payload_local: u32,\n",
        "        write_index_local: u32,\n",
        "        function: &mut Function,\n",
        "    ) -> Result<(), EmitError> {\n",
    )));

    for phase in [count, write] {
        assert_eq!(
            phase
                .matches("self.emit_property_key_payload_is_symbol_i32(")
                .count(),
            1
        );
        assert_eq!(
            phase.matches("HEAP_ARRAY_NAMED_PROPS_PTR_OFFSET").count(),
            1
        );
        assert_eq!(
            phase.matches("HEAP_ARRAY_NAMED_PROPS_LEN_OFFSET").count(),
            1
        );
        for forbidden in [
            "selection",
            "enumerable",
            "HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET",
            "OBJECT_DESCRIPTOR_ENUMERABLE",
            "match ",
            "matches!(",
            "_ =>",
            "unreachable!",
        ] {
            assert!(
                !phase.contains(forbidden),
                "named-string phase must not reintroduce `{forbidden}`"
            );
        }
    }
    assert_eq!(write.matches("self.emit_array_write(").count(), 1);
    assert!(!count.contains("self.emit_array_write("));
}

#[test]
fn only_get_own_property_names_consumes_array_named_string_storage() {
    assert_eq!(
        OBJECT_SOURCE
            .matches("self.emit_array_all_named_string_props_")
            .count(),
        2
    );
    let own_property_names = bounded(
        OBJECT_SOURCE,
        "    pub(super) fn compile_object_get_own_property_names_builtin(",
        "    pub(super) fn compile_object_get_own_property_symbols_builtin(",
    );
    assert_eq!(
        own_property_names
            .matches("self.emit_array_all_named_string_props_count(")
            .count(),
        1
    );
    assert_eq!(
        own_property_names
            .matches("self.emit_array_all_named_string_props_write_keys(")
            .count(),
        1
    );

    assert!(!OBJECT_SOURCE.contains("fn compile_object_keys_builtin("));
    let keys = bounded(
        ENUMERABLE_OWN_PROPERTIES_SOURCE,
        "    pub(in crate::builtins) fn compile_object_keys_builtin(",
        "    pub(in crate::builtins) fn compile_object_entries_builtin(",
    );
    assert_eq!(
        keys.matches("self.compile_object_enumerable_own_properties_builtin(")
            .count(),
        1
    );
    assert_eq!(keys.matches("EnumerableOwnProperties::Keys,").count(), 1);
    assert!(!ENUMERABLE_OWN_PROPERTIES_SOURCE.contains("named_string_props"));
}

#[test]
fn contract_records_the_selection_retirement() {
    let contract_words = CONTRACT.split_whitespace().collect::<Vec<_>>().join(" ");
    for marker in [
        "## Retirement",
        "`Object.keys` now runs the shared `EnumerableOwnProperties::Keys` algorithm",
        "`ArrayNamedStringKeySelection` was deleted",
    ] {
        assert!(
            contract_words.contains(marker),
            "missing contract retirement marker: {marker}"
        );
    }
}
