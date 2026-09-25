use std::fs;
use std::path::Path;

const HEAP: &str = include_str!("../src/heap.rs");
const OBJECTS: &str = include_str!("../src/objects.rs");
const BINARY_DATA: &str = include_str!("../src/builtins/binary_data.rs");
const STANDARD: &str = include_str!("../src/builtins/standard.rs");
const UINT8_ARRAY_CODECS: &str = include_str!("../src/builtins/uint8array_codecs.rs");
const TYPED_ARRAY_FILL: &str = include_str!("../src/builtins/typed_array_fill.rs");

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
    // The shared IsImmutableBuffer predicate replaced six per-site immutable
    // projections. Stable TypedArray sort subsequently dropped its direct
    // Detached probe and now writes through TypedArraySetElement.
    assert_eq!(count_in_rust_sources(&source_root, "ArrayBufferFlag"), 27);
    assert_eq!(count_in_rust_sources(&source_root, "ArrayBufferFlag::"), 25);
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
    // Unchanged count: the immutable throw lost its projection and the
    // IsImmutableBuffer predicate gained one.
    assert_eq!(BINARY_DATA.matches("ArrayBufferFlag::").count(), 6);
    // Resize, slice-species and transfer ask the immutable predicate; stable
    // sort uses the per-element witness instead of its old Detached probe.
    assert_eq!(STANDARD.matches("ArrayBufferFlag::").count(), 13);
    // 1 -> 0: the codec's write validation calls the shared immutable throw.
    assert_eq!(UINT8_ARRAY_CODECS.matches("ArrayBufferFlag::").count(), 0);
    // 1 -> 0: fill validates write access through its method-entry witness.
    assert_eq!(TYPED_ARRAY_FILL.matches("ArrayBufferFlag::").count(), 0);
    assert_eq!(HEAP.matches("ArrayBufferFlag::").count(), 4);
    assert_eq!(
        [
            OBJECTS,
            BINARY_DATA,
            STANDARD,
            UINT8_ARRAY_CODECS,
            TYPED_ARRAY_FILL,
        ]
        .into_iter()
        .map(|source| {
            source
                .lines()
                .filter(|line| line.contains("ArrayBufferFlag::") && line.contains(".word()"))
                .count()
        })
        .sum::<usize>(),
        // The sort Detached projection is no longer a product `word()` row.
        21
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
            "    pub(crate) fn emit_array_buffer_is_immutable_i32(",
            1,
        ),
        (
            "    pub(crate) fn emit_array_buffer_is_immutable_i32(",
            "    pub(crate) fn emit_typed_array_buffer_is_immutable_i32(",
            1,
        ),
        // The TypedArray predicate, element descriptor and immutable throw all
        // consult the predicate instead of projecting the flag again.
        (
            "    pub(crate) fn emit_typed_array_buffer_is_immutable_i32(",
            "    pub(crate) fn emit_initialize_data_view_private_state(",
            0,
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

    // IsImmutableBuffer is the sole reader of the `Immutable` word; the other
    // two binary-data `Immutable` rows create an immutable buffer.
    let predicate = bounded(
        BINARY_DATA,
        "    pub(crate) fn emit_array_buffer_is_immutable_i32(",
        "    pub(crate) fn emit_typed_array_buffer_is_immutable_i32(",
    );
    assert_eq!(
        projection_sequence(predicate, "ArrayBufferFlag::"),
        ["Immutable"]
    );
    let immutable_throw = bounded(
        BINARY_DATA,
        "    pub(crate) fn emit_throw_if_array_buffer_immutable(",
        "    pub(crate) fn emit_initialize_data_view_private_state(",
    );
    assert_eq!(
        immutable_throw
            .matches("self.emit_array_buffer_is_immutable_i32(")
            .count(),
        1
    );
    assert!(immutable_throw.contains("writer.type_error_message()"));

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
    assert_eq!(stable_sort.matches("ArrayBufferFlag::").count(), 0);
    assert_eq!(
        stable_sort
            .matches("self.emit_typed_array_element_write_from_locals(")
            .count(),
        1,
        "stable sort writes its snapshot through the current-view element path"
    );
    let standard_compiler = STANDARD
        .split_once("    pub(crate) fn compile_standard_builtin(")
        .expect("standard builtin compiler")
        .1;
    // 14 -> 11: resize, slice-species and transfer immutable reads moved to
    // the shared predicate.
    assert_eq!(standard_compiler.matches("ArrayBufferFlag::").count(), 11);
    assert_eq!(
        standard_compiler
            .matches("self.emit_array_buffer_is_immutable_i32(")
            .count(),
        3,
        "resize, the grouped slice species check and the immutable getter"
    );
    assert_eq!(
        standard_compiler
            .matches("ImmutableBufferWriter::CopyAndDetach")
            .count(),
        1,
        "the transfer family rejects an immutable receiver through the shared throw"
    );

    let codec_receiver = bounded(
        UINT8_ARRAY_CODECS,
        "    pub(super) fn emit_uint8_array_codec_receiver(",
        "    pub(super) fn emit_uint8_array_codec_string(",
    );
    let write_validation = codec_receiver
        .split_once("Uint8ArrayCodecAccess::Write => {")
        .expect("only writing codecs reject immutable buffers")
        .1;
    assert_eq!(codec_receiver.matches("ArrayBufferFlag::").count(), 0);
    assert_eq!(
        codec_receiver
            .matches("self.emit_throw_if_array_buffer_immutable(")
            .count(),
        1
    );
    assert!(write_validation.contains("self.emit_throw_if_array_buffer_immutable("));
    assert!(write_validation.contains("ImmutableBufferWriter::Uint8ArrayCodec"));

    let typed_array_fill = TYPED_ARRAY_FILL
        .split_once("    pub(super) fn compile_typed_array_prototype_fill_builtin(")
        .expect("TypedArray.prototype.fill owner")
        .1;
    assert_eq!(typed_array_fill.matches("ArrayBufferFlag::").count(), 0);
    // Fill's entry witness is ValidateTypedArray(O, seq-cst, write): its
    // immutable-buffer rejection precedes the value conversion.
    let entry_witness = typed_array_fill
        .split_once("self.emit_typed_array_witness(")
        .expect("fill validates its receiver")
        .1;
    let (entry_arguments, after_entry) = entry_witness
        .split_once("function,\n        )?;")
        .expect("fill entry witness call");
    assert!(entry_arguments.contains("access: TypedArrayAccessMode::Write,"));
    assert!(after_entry.contains("self.emit_value_to_typed_array_element_payload("));
    assert_eq!(
        typed_array_fill
            .matches("access: TypedArrayAccessMode::Write,")
            .count(),
        1,
        "only the entry witness validates write access; the later observation re-reads bounds"
    );
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

    // Stable sort dropped its direct Detached probe (row 10) when writes
    // moved to TypedArraySetElement. The immutable-ArrayBuffer work removed
    // three per-site Immutable reads: resize (row 17), grouped slice species
    // (row 21) and transfer receiver (row 23). Those ask IsImmutableBuffer,
    // whose projection replaces the old immutable throw's at row 7.
    const REMOVED_LEGACY_ROWS: [usize; 4] = [10, 17, 21, 23];
    let legacy_sequence = projection_sequence(legacy_rows, "ARRAY_BUFFER_FLAG_");
    assert_eq!(legacy_sequence.len(), 25);
    for row in REMOVED_LEGACY_ROWS {
        let removed_flag = if row == 10 { "Detached" } else { "Immutable" };
        assert_eq!(legacy_sequence[row], removed_flag, "legacy row {row}");
    }
    let expected_sequence = legacy_sequence
        .iter()
        .enumerate()
        .filter(|(row, _)| !REMOVED_LEGACY_ROWS.contains(row))
        .map(|(_, flag)| *flag)
        .collect::<Vec<_>>();
    let current_sequence = [OBJECTS, BINARY_DATA, STANDARD]
        .into_iter()
        .flat_map(|source| projection_sequence(source, "ArrayBufferFlag::"))
        .collect::<Vec<_>>();
    assert_eq!(current_sequence, expected_sequence);
}
