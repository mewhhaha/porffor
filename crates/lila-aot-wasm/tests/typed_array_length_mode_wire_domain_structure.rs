use std::fs;
use std::path::Path;

const HEAP: &str = include_str!("../src/heap.rs");
const OBJECTS: &str = include_str!("../src/objects.rs");
const BINARY_DATA: &str = include_str!("../src/builtins/binary_data.rs");
const STANDARD: &str = include_str!("../src/builtins/standard.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn without_whitespace(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
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

fn fnv1a(source: &str) -> u64 {
    source.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100_0000_01b3)
    })
}

fn projection_sequence(source: &str) -> Vec<&'static str> {
    source
        .lines()
        .filter(|line| line.contains("TypedArrayLengthMode::"))
        .map(|line| {
            if line.contains("Fixed") {
                "Fixed"
            } else if line.contains("Tracking") {
                "Tracking"
            } else {
                panic!("unknown TypedArray length-mode projection `{line}`")
            }
        })
        .collect()
}

#[test]
fn typed_array_length_mode_is_one_capability_free_two_row_wire_authority() {
    let declaration_offset = HEAP
        .find("pub(crate) enum TypedArrayLengthMode {")
        .expect("TypedArrayLengthMode declaration");
    assert_eq!(
        HEAP[..declaration_offset]
            .lines()
            .rev()
            .find(|line| !line.trim().is_empty())
            .map(str::trim),
        Some("pub(crate) const HEAP_TYPED_ARRAY_LENGTH_TRACKING_OFFSET: u64 = 120;")
    );
    let authority = bounded(
        HEAP,
        "pub(crate) enum TypedArrayLengthMode {",
        "pub(crate) const HEAP_DATA_VIEW_VIEWED_BUFFER_OFFSET",
    );
    assert_eq!(
        without_whitespace(authority),
        concat!(
            "Fixed,Tracking,}",
            "implTypedArrayLengthMode{pub(crate)constfnword(&self)->u64{",
            "matchself{Self::Fixed=>0,Self::Tracking=>1,}}}"
        )
    );
    assert_eq!(authority.matches("=>").count(), 2);
    assert!(!authority.contains("_ =>"));
    assert!(!authority.contains("#[derive("));
    for capability in [
        "Clone",
        "Copy",
        "Debug",
        "Default",
        "PartialEq",
        "Eq",
        "Hash",
        "PartialOrd",
        "Ord",
    ] {
        assert!(!HEAP.contains(&format!("impl {capability} for TypedArrayLengthMode")));
    }
}

#[test]
fn six_named_projections_are_the_complete_typed_array_length_mode_census() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "TypedArrayLengthMode"),
        8
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "TypedArrayLengthMode::"),
        6
    );
    assert_eq!(HEAP.matches("TypedArrayLengthMode").count(), 2);
    assert_eq!(OBJECTS.matches("TypedArrayLengthMode::").count(), 1);
    assert_eq!(BINARY_DATA.matches("TypedArrayLengthMode::").count(), 3);
    assert_eq!(STANDARD.matches("TypedArrayLengthMode::").count(), 2);
    assert_eq!(
        [OBJECTS, BINARY_DATA, STANDARD]
            .into_iter()
            .map(|source| source.matches("TypedArrayLengthMode::Fixed.word()").count())
            .sum::<usize>(),
        5
    );
    assert_eq!(
        [OBJECTS, BINARY_DATA, STANDARD]
            .into_iter()
            .map(|source| source
                .matches("TypedArrayLengthMode::Tracking.word()")
                .count())
            .sum::<usize>(),
        1
    );
}

#[test]
fn three_readers_and_three_writers_own_every_length_mode_projection() {
    let object_reader = bounded(
        OBJECTS,
        "    pub(crate) fn emit_ordinary_prevent_extensions_i32(",
        "    pub(crate) fn emit_ordinary_is_extensible_i32(",
    );
    assert_eq!(object_reader.matches("TypedArrayLengthMode::").count(), 1);

    let witness_reader = bounded(
        BINARY_DATA,
        "    pub(crate) fn emit_typed_array_witness(",
        "    pub(crate) fn emit_initialize_array_buffer_private_state(",
    );
    assert_eq!(witness_reader.matches("TypedArrayLengthMode::").count(), 1);
    let buffer_initializer = bounded(
        BINARY_DATA,
        "    pub(crate) fn emit_initialize_typed_array_from_array_buffer(",
        "    pub(crate) fn emit_detach_array_buffer(",
    );
    assert_eq!(
        buffer_initializer.matches("TypedArrayLengthMode::").count(),
        2
    );
    assert_eq!(
        buffer_initializer
            .matches("Instruction::LocalSet(length_tracking_local)")
            .count(),
        2
    );

    let subarray_reader = bounded(
        STANDARD,
        "            StandardBuiltinId::TypedArrayPrototypeSubarray => {",
        "            StandardBuiltinId::DateNow => {",
    );
    assert_eq!(subarray_reader.matches("TypedArrayLengthMode::").count(), 1);
    let constructor_owner = bounded(
        STANDARD,
        "            StandardBuiltinId::Float64ArrayConstructor",
        "            StandardBuiltinId::DataViewPrototypeGetUint8",
    );
    assert_eq!(
        constructor_owner.matches("TypedArrayLengthMode::").count(),
        1
    );
    assert_eq!(
        constructor_owner
            .matches("HEAP_TYPED_ARRAY_LENGTH_TRACKING_OFFSET")
            .count(),
        1,
        "the grouped constructor remains the sole length-mode publisher"
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "HEAP_TYPED_ARRAY_LENGTH_TRACKING_OFFSET"),
        7
    );
}

#[test]
fn named_length_modes_preserve_the_frozen_zero_one_projection_sequence() {
    let legacy_rows = concat!(
        "        function.instruction(&Instruction::I64Const(0));\n",
        "        function.instruction(&Instruction::I64Const(0));\n",
        "        function.instruction(&Instruction::I64Const(0));\n",
        "        function.instruction(&Instruction::I64Const(1));\n",
        "                function.instruction(&Instruction::I64Const(0));\n",
        "                function.instruction(&Instruction::I64Const(0));\n",
    );
    let normalized = without_whitespace(legacy_rows);
    assert_eq!(
        (legacy_rows.len(), fnv1a(legacy_rows)),
        (358, 0xc988_3610_80d6_b5cc)
    );
    assert_eq!(
        (normalized.len(), fnv1a(&normalized)),
        (288, 0x7e69_1158_ebde_da94)
    );

    let current_sequence = [OBJECTS, BINARY_DATA, STANDARD]
        .into_iter()
        .flat_map(projection_sequence)
        .collect::<Vec<_>>();
    assert_eq!(
        current_sequence,
        ["Fixed", "Fixed", "Fixed", "Tracking", "Fixed", "Fixed"]
    );
}
