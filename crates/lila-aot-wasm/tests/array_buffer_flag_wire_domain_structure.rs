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

fn projection_sequence(source: &str, spelling: &str) -> Vec<&'static str> {
    source
        .lines()
        .filter(|line| line.contains(spelling))
        .map(|line| {
            if line.contains("Resizable") || line.contains("RESIZABLE") {
                "Resizable"
            } else if line.contains("Shared") || line.contains("SHARED") {
                "Shared"
            } else if line.contains("Immutable") || line.contains("IMMUTABLE") {
                "Immutable"
            } else if line.contains("Detached") || line.contains("DETACHED") {
                "Detached"
            } else {
                panic!("unknown ArrayBuffer flag projection `{line}`")
            }
        })
        .collect()
}

#[test]
fn array_buffer_flag_is_one_capability_free_four_row_wire_authority() {
    let declaration_offset = HEAP
        .find("pub(crate) enum ArrayBufferFlag {")
        .expect("ArrayBufferFlag declaration");
    assert_eq!(
        HEAP[..declaration_offset]
            .lines()
            .rev()
            .find(|line| !line.trim().is_empty())
            .map(str::trim),
        Some("pub(crate) const HEAP_ARRAY_BUFFER_FLAGS_OFFSET: u64 = 120;")
    );
    let authority = bounded(
        HEAP,
        "pub(crate) enum ArrayBufferFlag {",
        "// TypedArray instances are ordinary heap objects",
    );
    assert_eq!(
        without_whitespace(authority),
        concat!(
            "Resizable,Shared,Immutable,Detached,}",
            "implArrayBufferFlag{pub(crate)constfnword(&self)->u64{matchself{",
            "Self::Resizable=>1,Self::Shared=>2,Self::Immutable=>4,",
            "Self::Detached=>8,}}}"
        )
    );
    assert_eq!(
        authority.matches("pub(crate) const fn word(&self)").count(),
        1
    );
    assert_eq!(authority.matches("=>").count(), 4);
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
        assert!(!HEAP.contains(&format!("impl {capability} for ArrayBufferFlag")));
    }
}

#[test]
fn all_array_buffer_flag_projections_use_the_closed_vocabulary() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(count_in_rust_sources(&source_root, "ArrayBufferFlag"), 31);
    assert_eq!(count_in_rust_sources(&source_root, "ArrayBufferFlag::"), 29);
    for old_name in [
        "ARRAY_BUFFER_FLAG_RESIZABLE",
        "ARRAY_BUFFER_FLAG_SHARED",
        "ARRAY_BUFFER_FLAG_IMMUTABLE",
        "ARRAY_BUFFER_FLAG_DETACHED",
    ] {
        assert_eq!(
            count_in_rust_sources(&source_root, old_name),
            0,
            "{old_name}"
        );
    }

    assert_eq!(OBJECTS.matches("ArrayBufferFlag::").count(), 2);
    assert_eq!(BINARY_DATA.matches("ArrayBufferFlag::").count(), 6);
    assert_eq!(STANDARD.matches("ArrayBufferFlag::").count(), 17);
    assert_eq!(HEAP.matches("ArrayBufferFlag::").count(), 4);
    assert_eq!(
        [OBJECTS, BINARY_DATA, STANDARD]
            .into_iter()
            .map(|source| {
                source
                    .lines()
                    .filter(|line| line.contains("ArrayBufferFlag::") && line.contains(".word()"))
                    .count()
            })
            .sum::<usize>(),
        25
    );
}

#[test]
fn every_product_projection_stays_with_its_single_algorithm_owner() {
    let object_owner = bounded(
        OBJECTS,
        "    pub(crate) fn emit_ordinary_prevent_extensions_i32(",
        "    pub(crate) fn emit_ordinary_is_extensible_i32(",
    );
    assert_eq!(object_owner.matches("ArrayBufferFlag::").count(), 2);

    for (start, end, expected) in [
        (
            "    pub(super) fn emit_array_buffer_slice_copy(",
            "    pub(crate) fn emit_initialize_typed_array_from_array_buffer(",
            2,
        ),
        (
            "    pub(crate) fn emit_initialize_typed_array_from_array_buffer(",
            "    pub(crate) fn emit_detach_array_buffer(",
            2,
        ),
        (
            "    pub(crate) fn emit_detach_array_buffer(",
            "    pub(crate) fn emit_throw_if_shared_array_buffer(",
            1,
        ),
        (
            "    pub(crate) fn emit_throw_if_array_buffer_immutable(",
            "    pub(crate) fn emit_initialize_data_view_private_state(",
            1,
        ),
    ] {
        assert_eq!(
            bounded(BINARY_DATA, start, end)
                .matches("ArrayBufferFlag::")
                .count(),
            expected,
            "flag projections in `{start}`"
        );
    }

    let slice_kind = bounded(
        STANDARD,
        "enum ArrayBufferSliceKind {",
        "fn emit_active_standard_builtin_function_payload(",
    );
    assert_eq!(slice_kind.matches("ArrayBufferFlag::").count(), 2);
    let stable_sort = bounded(
        STANDARD,
        "    fn emit_typed_array_stable_sort(",
        "    fn compile_typed_array_prototype_to_sorted_builtin(",
    );
    assert_eq!(stable_sort.matches("ArrayBufferFlag::").count(), 1);
    let standard_compiler = STANDARD
        .split_once("    pub(crate) fn compile_standard_builtin(")
        .expect("standard builtin compiler")
        .1;
    assert_eq!(standard_compiler.matches("ArrayBufferFlag::").count(), 14);
}

#[test]
fn closed_flag_selection_preserves_the_frozen_wire_projection_sequence() {
    let legacy_rows = concat!(
        "        function.instruction(&Instruction::I64Const(ARRAY_BUFFER_FLAG_RESIZABLE as i64));\n",
        "        function.instruction(&Instruction::I64Const(ARRAY_BUFFER_FLAG_SHARED as i64));\n",
        "                function.instruction(&Instruction::I64Const(ARRAY_BUFFER_FLAG_DETACHED as i64));\n",
        "                function.instruction(&Instruction::I64Const(ARRAY_BUFFER_FLAG_IMMUTABLE as i64));\n",
        "        function.instruction(&Instruction::I64Const(ARRAY_BUFFER_FLAG_DETACHED as i64));\n",
        "        function.instruction(&Instruction::I64Const(ARRAY_BUFFER_FLAG_RESIZABLE as i64));\n",
        "        function.instruction(&Instruction::I64Const(ARRAY_BUFFER_FLAG_DETACHED as i64));\n",
        "        function.instruction(&Instruction::I64Const(ARRAY_BUFFER_FLAG_IMMUTABLE as i64));\n",
        "            Self::Shared => ARRAY_BUFFER_FLAG_SHARED,\n",
        "            Self::ToImmutable => ARRAY_BUFFER_FLAG_IMMUTABLE,\n",
        "        function.instruction(&Instruction::I64Const(ARRAY_BUFFER_FLAG_DETACHED as i64));\n",
        "                    function.instruction(&Instruction::I64Const(ARRAY_BUFFER_FLAG_SHARED as i64));\n",
        "                        .instruction(&Instruction::I64Const(ARRAY_BUFFER_FLAG_RESIZABLE as i64));\n",
        "                function.instruction(&Instruction::I64Const(ARRAY_BUFFER_FLAG_RESIZABLE as i64));\n",
        "                function.instruction(&Instruction::I64Const(ARRAY_BUFFER_FLAG_DETACHED as i64));\n",
        "                function.instruction(&Instruction::I64Const(ARRAY_BUFFER_FLAG_DETACHED as i64));\n",
        "                function.instruction(&Instruction::I64Const(ARRAY_BUFFER_FLAG_RESIZABLE as i64));\n",
        "                function.instruction(&Instruction::I64Const(ARRAY_BUFFER_FLAG_IMMUTABLE as i64));\n",
        "                function.instruction(&Instruction::I64Const(ARRAY_BUFFER_FLAG_RESIZABLE as i64));\n",
        "                            .instruction(&Instruction::I64Const(ARRAY_BUFFER_FLAG_DETACHED as i64));\n",
        "                                    ARRAY_BUFFER_FLAG_DETACHED as i64,\n",
        "                                        ARRAY_BUFFER_FLAG_IMMUTABLE as i64,\n",
        "                function.instruction(&Instruction::I64Const(ARRAY_BUFFER_FLAG_RESIZABLE as i64));\n",
        "                function.instruction(&Instruction::I64Const(ARRAY_BUFFER_FLAG_IMMUTABLE as i64));\n",
        "                        .instruction(&Instruction::I64Const(ARRAY_BUFFER_FLAG_IMMUTABLE as i64));\n",
    );
    let normalized = without_whitespace(legacy_rows);
    assert_eq!(
        (normalized.len(), fnv1a(&normalized)),
        (1773, 0xa28c_7750_59da_a571)
    );

    let legacy_sequence = projection_sequence(legacy_rows, "ARRAY_BUFFER_FLAG_");
    let current_sequence = [OBJECTS, BINARY_DATA, STANDARD]
        .into_iter()
        .flat_map(|source| projection_sequence(source, "ArrayBufferFlag::"))
        .collect::<Vec<_>>();
    assert_eq!(current_sequence, legacy_sequence);
}
