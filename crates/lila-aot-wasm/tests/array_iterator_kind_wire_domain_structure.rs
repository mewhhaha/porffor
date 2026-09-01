use std::fs;
use std::path::Path;

const ITERATORS: &str = include_str!("../src/builtins/iterators.rs");
const BUILTINS: &str = include_str!("../src/builtins/mod.rs");
const STANDARD: &str = include_str!("../src/builtins/standard.rs");
const FUNCTIONS: &str = include_str!("../src/functions.rs");
const STRING: &str = include_str!("../src/builtins/string.rs");

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
fn array_iterator_kind_is_one_closed_three_row_wire_authority() {
    let authority = bounded(
        ITERATORS,
        "macro_rules! array_iterator_kind_domain {",
        "impl<'a> FunctionBuilder<'a> {",
    );
    let rows = bounded(
        ITERATORS,
        "array_iterator_kind_domain!(ArrayIteratorKind {",
        "\n});",
    )
    .lines()
    .map(str::trim)
    .filter(|line| !line.is_empty())
    .collect::<Vec<_>>();

    assert_eq!(rows, ["Key = 1,", "Value = 0,", "KeyAndValue = 2,"]);
    assert_eq!(authority.matches("pub(crate) enum $name").count(), 1);
    assert_eq!(authority.matches("const ALL:").count(), 1);
    assert_eq!(authority.matches("const fn word(&self) -> u64").count(), 1);
    assert!(authority.contains("assert!(all[left].word() != all[right].word());"));
    for capability in [
        "#[derive(",
        "Clone",
        "Copy",
        "Debug",
        "PartialEq",
        "Eq",
        "Default",
    ] {
        assert!(!authority.contains(capability), "capability `{capability}`");
    }
    assert_eq!(
        BUILTINS
            .matches("pub(crate) use iterators::ArrayIteratorKind;")
            .count(),
        1
    );
    assert!(!BUILTINS.contains("pub(crate) const ARRAY_ITERATOR_KIND_"));
}

#[test]
fn iterator_constructors_accept_kinds_and_serialize_only_at_storage() {
    let ordinary = bounded(
        ITERATORS,
        "    pub(crate) fn emit_array_iterator_create_from_locals(",
        "    pub(crate) fn emit_typed_array_iterator_create_from_locals(",
    );
    let typed = bounded(
        ITERATORS,
        "    pub(crate) fn emit_typed_array_iterator_create_from_locals(",
        "    pub(crate) fn emit_typed_array_iterator_next_from_locals(",
    );

    for constructor in [ordinary, typed] {
        assert!(constructor.contains("kind: &ArrayIteratorKind,"));
        assert_eq!(constructor.matches("kind.word(),").count(), 1);
        assert!(!constructor.contains("kind: u64"));
        for raw_constant in [
            "ARRAY_ITERATOR_KIND_KEYS",
            "ARRAY_ITERATOR_KIND_VALUES",
            "ARRAY_ITERATOR_KIND_ENTRIES",
        ] {
            assert!(!constructor.contains(raw_constant));
        }
    }
}

#[test]
fn every_array_iterator_producer_selects_a_named_kind() {
    let standard_producers = bounded(
        STANDARD,
        "            StandardBuiltinId::ArrayPrototypeKeys => {",
        "            StandardBuiltinId::ArrayIteratorIdentity => {",
    );
    assert_eq!(
        standard_producers
            .matches("self.compile_array_iterator_method_builtin(")
            .count(),
        6
    );
    assert_eq!(
        standard_producers
            .matches("&ArrayIteratorKind::Key,")
            .count(),
        2
    );
    assert_eq!(
        standard_producers
            .matches("&ArrayIteratorKind::Value,")
            .count(),
        2
    );
    assert_eq!(
        standard_producers
            .matches("&ArrayIteratorKind::KeyAndValue,")
            .count(),
        2
    );

    assert_eq!(STRING.matches("&ArrayIteratorKind::Value,").count(), 5);
    assert_eq!(FUNCTIONS.matches("method.iterator_kind(),").count(), 1);
    assert_eq!(
        FUNCTIONS
            .matches("Self::Keys => &ArrayIteratorKind::Key,")
            .count(),
        1
    );
    assert_eq!(
        FUNCTIONS
            .matches("Self::Values => &ArrayIteratorKind::Value,")
            .count(),
        1
    );
    assert_eq!(
        FUNCTIONS
            .matches("Self::Entries => &ArrayIteratorKind::KeyAndValue,")
            .count(),
        1
    );

    for source in [STANDARD, FUNCTIONS, STRING] {
        assert!(!source.contains("ARRAY_ITERATOR_KIND_"));
        assert!(!source.contains("kind: u64"));
    }

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "emit_array_iterator_create_from_locals("),
        8,
        "the ordinary constructor definition and all seven call sites must stay inventoried"
    );
    assert_eq!(
        count_in_rust_sources(
            &source_root,
            "emit_typed_array_iterator_create_from_locals(",
        ),
        3,
        "the private constructor definition and both receiver-policy call sites must stay inventoried"
    );
}

#[test]
fn both_next_paths_decode_all_rows_and_exhaustively_emit_their_semantics() {
    let typed = bounded(
        ITERATORS,
        "    pub(crate) fn emit_typed_array_iterator_next_from_locals(",
        "    pub(crate) fn emit_iterator_result_object_from_locals(",
    );
    assert_eq!(
        typed.matches("for kind in ArrayIteratorKind::ALL").count(),
        1
    );
    assert_eq!(typed.matches("match kind {").count(), 1);
    for arm in [
        "ArrayIteratorKind::Key => {",
        "ArrayIteratorKind::Value => {",
        "ArrayIteratorKind::KeyAndValue => {",
    ] {
        assert_eq!(typed.matches(arm).count(), 1, "typed arm `{arm}`");
    }
    assert_eq!(typed.matches("Instruction::Unreachable").count(), 1);
    assert_eq!(typed.matches("Instruction::Br(1)").count(), 1);
    assert!(!typed.contains("_ =>"));

    let ordinary = bounded(
        STANDARD,
        "            StandardBuiltinId::ArrayIteratorNext => {",
        "            StandardBuiltinId::ArrayBufferIsView => {",
    );
    assert_eq!(
        ordinary
            .matches("for kind in ArrayIteratorKind::ALL")
            .count(),
        2
    );
    assert_eq!(ordinary.matches("match kind {").count(), 1);
    for arm in [
        "ArrayIteratorKind::Key => {",
        "ArrayIteratorKind::Value => {",
        "ArrayIteratorKind::KeyAndValue => {",
    ] {
        assert_eq!(ordinary.matches(arm).count(), 1, "ordinary arm `{arm}`");
    }
    assert!(!ordinary.contains("Instruction::Unreachable"));
    assert_eq!(ordinary.matches("Instruction::BrIf(0)").count(), 1);
    assert_eq!(ordinary.matches("Instruction::Br(1)").count(), 1);
    assert_eq!(ordinary.matches("Instruction::F64Eq").count(), 1);
    assert_eq!(
        ordinary.matches("kind.word() as f64").count(),
        1,
        "ordinary validation must compare the exact Number against every stable row"
    );
    assert!(!ordinary.contains("_ =>"));
    let kind_validation = bounded(
        ordinary,
        "                function.instruction(&Instruction::Block(BlockType::Empty));\n                for kind in ArrayIteratorKind::ALL {",
        "\n\n                self.emit_alloc_plain_object_with_prototype(",
    );
    assert_eq!(kind_validation.matches("Instruction::BrIf(0)").count(), 1);
    assert_eq!(
        kind_validation
            .matches("self.emit_throw_current_function_realm_type_error(")
            .count(),
        1
    );
}

#[test]
fn static_method_spelling_has_no_value_fallback() {
    let projection = bounded(
        FUNCTIONS,
        "enum StaticArrayIteratorMethod {",
        "impl ProxyCallThrowRouting {",
    );
    for spelling in ["\"keys\"", "\"values\"", "\"entries\""] {
        assert_eq!(
            projection.matches(spelling).count(),
            1,
            "spelling `{spelling}`"
        );
    }
    assert_eq!(projection.matches("return Some(Self::").count(), 3);
    assert!(!projection.contains("StaticGenerator"));
    assert!(projection.contains("match self {"));
    assert!(!projection.contains("#[derive("));
    assert!(!projection.contains("_ =>"));
    assert!(!projection.contains("unwrap_or"));
    assert!(!projection.contains("unwrap_or_else"));
}
