use std::fs;
use std::path::Path;

const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/array.rs");
const CORE_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_array_to_locale_string_core.js");
const INVOCATION_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_array_to_locale_string_invocation.js");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/array-to-locale-string-receiver-kind.md");
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
fn receiver_kind_is_an_exact_private_non_derived_domain() {
    let declaration_region = bounded(
        ARRAY_SOURCE,
        concat!(
            "pub(crate) enum ArraySortOutput {\n",
            "    Receiver,\n",
            "    Copy,\n",
            "}\n\n"
        ),
        "\n\n#[derive(Clone, Copy, Debug, PartialEq, Eq)]\n#[repr(i64)]",
    );
    assert_eq!(
        normalized(declaration_region),
        "enumToLocaleStringReceiverKind{ArrayLike,TypedArray,}"
    );
    assert!(!declaration_region.contains("#["));
    assert!(!declaration_region.contains("pub"));
    assert!(!ARRAY_SOURCE.contains("impl ToLocaleStringReceiverKind"));

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "ToLocaleStringReceiverKind"),
        11,
        "the declaration, two producers, two typed parameters and six exhaustive arms own every mention"
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "ToLocaleStringReceiverKind::ArrayLike"),
        4
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "ToLocaleStringReceiverKind::TypedArray"),
        4
    );
    for capability in ["Clone", "Copy", "Debug", "Default", "PartialEq", "Eq"] {
        assert!(!ARRAY_SOURCE.contains(&format!("{capability} for ToLocaleStringReceiverKind")));
    }
}

#[test]
fn exactly_two_entry_producers_choose_their_receiver_kind() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "compile_to_locale_string_builtin("),
        3,
        "the definition and two builtin entries are the complete call census"
    );

    let entries = bounded(
        ARRAY_SOURCE,
        "pub(crate) fn compile_array_prototype_to_locale_string_builtin(",
        "fn emit_validate_to_locale_string_invocation(",
    );
    for producer in [
        "compile_to_locale_string_builtin(ToLocaleStringReceiverKind::ArrayLike, function)",
        "compile_to_locale_string_builtin(ToLocaleStringReceiverKind::TypedArray, function)",
    ] {
        assert_eq!(
            entries.matches(producer).count(),
            1,
            "producer `{producer}`"
        );
    }
    assert_before(
        entries,
        "ToLocaleStringReceiverKind::ArrayLike",
        "ToLocaleStringReceiverKind::TypedArray",
    );
}

#[test]
fn all_three_receiver_decisions_are_exhaustive_and_ordered() {
    let validator = bounded(
        ARRAY_SOURCE,
        "fn emit_validate_to_locale_string_invocation(",
        "fn emit_call_validated_to_locale_string_invocation(",
    );
    assert!(validator.contains("receiver_kind: &ToLocaleStringReceiverKind,"));
    let error_projection = normalized(bounded(
        validator,
        "let error_message = match receiver_kind {",
        "self.emit_throw_current_function_realm_type_error(",
    ));
    assert_eq!(
        error_projection,
        concat!(
            "ToLocaleStringReceiverKind::ArrayLike=>{",
            "\"Array.prototype.toLocaleStringelementmethodisnotcallable\"}",
            "ToLocaleStringReceiverKind::TypedArray=>{",
            "\"TypedArray.prototype.toLocaleStringelementmethodisnotcallable\"}};"
        )
    );

    let shared = bounded(
        ARRAY_SOURCE,
        "fn compile_to_locale_string_builtin(",
        "pub(crate) fn emit_object_has_array_index_key_in_range_i32(",
    );
    let method_projection = normalized(bounded(
        shared,
        "let method_name = match &receiver_kind {",
        "let receiver_payload_local = self.this_payload_local.ok_or_else(|| {",
    ));
    assert_eq!(
        method_projection,
        concat!(
            "ToLocaleStringReceiverKind::ArrayLike=>\"Array.prototype.toLocaleString\",",
            "ToLocaleStringReceiverKind::TypedArray=>\"TypedArray.prototype.toLocaleString\",};"
        )
    );
    let entry_projection = normalized(bounded(
        shared,
        "let typed_array_entry = match &receiver_kind {",
        "if typed_array_entry {",
    ));
    assert_eq!(
        entry_projection,
        concat!(
            "ToLocaleStringReceiverKind::ArrayLike=>false,",
            "ToLocaleStringReceiverKind::TypedArray=>true,};"
        )
    );
    assert_eq!(shared.matches("match &receiver_kind {").count(), 2);
    assert_eq!(
        shared
            .matches("emit_validate_to_locale_string_invocation(\n            &receiver_kind,")
            .count(),
        1
    );
    for forbidden in [
        "matches!(receiver_kind",
        "receiver_kind ==",
        "receiver_kind !=",
        "_ =>",
        "unreachable!",
    ] {
        assert!(
            !validator.contains(forbidden),
            "validator found `{forbidden}`"
        );
        assert!(
            !shared.contains(forbidden),
            "shared emitter found `{forbidden}`"
        );
    }
    assert_before(
        shared,
        "let method_name = match",
        "let receiver_payload_local",
    );
    assert_before(
        shared,
        "let typed_array_entry = match",
        "if typed_array_entry {",
    );
    assert_before(
        shared,
        "if typed_array_entry {",
        "emit_validate_to_locale_string_invocation(",
    );
}

#[test]
fn contract_and_existing_product_witnesses_pin_both_entries() {
    assert!(CONTRACT.contains("ToLocaleStringReceiverKind"));
    assert!(CONTRACT
        .contains("cargo test -p lila-aot-wasm --test to_locale_string_receiver_kind_structure"));
    assert!(TASK.contains("ToLocaleStringReceiverKind"));
    for registration in [
        "fn run_wasm_backend_succeeds_for_supported_array_to_locale_string_fixture()",
        "fn run_wasm_backend_succeeds_for_array_to_locale_string_invocation_fixture()",
    ] {
        assert!(CLI_TESTS.contains(registration), "missing `{registration}`");
    }
    for marker in [
        "Array.prototype.toLocaleString.call(tracking)",
        "tracking.toLocaleString()",
        "fixed.toLocaleString()",
    ] {
        assert!(
            CORE_FIXTURE.contains(marker),
            "missing core marker `{marker}`"
        );
    }
    for marker in [
        "otherArrayToLocaleString.call([{ toLocaleString: 0 }])",
        "otherTypedArrayToLocaleString.call(new other.Uint8Array([1]))",
        "Array.prototype.toLocaleString.call([proxyElement])",
    ] {
        assert!(
            INVOCATION_FIXTURE.contains(marker),
            "missing invocation marker `{marker}`"
        );
    }
}
