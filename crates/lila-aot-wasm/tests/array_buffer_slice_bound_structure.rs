use std::fs;
use std::path::{Path, PathBuf};

const BINARY_DATA_SOURCE: &str = include_str!("../src/builtins/binary_data.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const BOUND_HELPER: &str = "emit_array_buffer_slice_index_to_local";

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

fn assert_normalized_once(source: &str, expected: &str, message: &str) {
    let source = without_whitespace(source);
    let expected = without_whitespace(expected);
    assert_eq!(source.matches(expected.as_str()).count(), 1, "{message}");
}

fn exact_identifier_mentions(source: &str, identifier: &str) -> usize {
    let is_boundary = |ch: Option<char>| match ch {
        Some(ch) => !ch.is_alphanumeric() && ch != '_',
        None => true,
    };

    source
        .match_indices(identifier)
        .filter(|(start, _)| {
            let end = *start + identifier.len();
            is_boundary(source[..*start].chars().next_back())
                && is_boundary(source[end..].chars().next())
        })
        .count()
}

fn fnv1a(source: &str) -> u64 {
    source.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

fn collect_rust_sources(dir: &Path, sources: &mut Vec<(PathBuf, String)>) {
    let mut paths = fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .map(|entry| entry.expect("failed to read source entry").path())
        .collect::<Vec<_>>();
    paths.sort();

    for path in paths {
        if path.is_dir() {
            collect_rust_sources(&path, sources);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            sources.push((path, source));
        }
    }
}

#[test]
fn array_buffer_slice_bound_is_closed_and_derives_its_argument_index() {
    let declaration_offset = BINARY_DATA_SOURCE
        .find("pub(super) enum ArrayBufferSliceBound {")
        .expect("slice-bound declaration");
    assert_eq!(
        BINARY_DATA_SOURCE[..declaration_offset]
            .lines()
            .rev()
            .find(|line| !line.trim().is_empty())
            .map(str::trim),
        Some("/// positions unrepresentable at the caller boundary.")
    );
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
            !BINARY_DATA_SOURCE.contains(&format!("impl {capability} for ArrayBufferSliceBound"))
        );
    }

    let variants = bounded(
        BINARY_DATA_SOURCE,
        "pub(super) enum ArrayBufferSliceBound {",
        "\n}",
    );
    assert_eq!(
        without_whitespace(variants),
        "Start,End,",
        "the slice-bound role must remain exactly Start and End"
    );
    assert_eq!(
        BINARY_DATA_SOURCE
            .matches("pub(super) enum ArrayBufferSliceBound {")
            .count(),
        1,
        "the bound domain must have one builtins-private definition"
    );
    assert!(!BINARY_DATA_SOURCE.contains("pub(crate) enum ArrayBufferSliceBound"));
    assert!(!BINARY_DATA_SOURCE.contains("pub enum ArrayBufferSliceBound"));

    let argument_index = bounded(
        BINARY_DATA_SOURCE,
        "const fn argument_index(&self) -> usize {",
        "\n    }\n}\n\n/// The closed source",
    );
    assert_eq!(
        without_whitespace(argument_index),
        "matchself{Self::Start=>0,Self::End=>1,}",
        "Start and End must exhaustively own argument positions zero and one"
    );
    assert_eq!(argument_index.matches("=>").count(), 2);
    assert!(!argument_index.contains("_ =>"));
    assert!(!argument_index.contains("if self"));
    assert!(!argument_index.contains("matches!(self"));

    let implementation = bounded(
        BINARY_DATA_SOURCE,
        "impl ArrayBufferSliceBound {",
        "\n}\n\n/// The closed source",
    );
    assert_eq!(
        (implementation.len(), fnv1a(implementation)),
        (141, 0xe1e9_402d_e2d5_17d1)
    );
    let normalized_implementation =
        implementation.replace("argument_index(&self)", "argument_index(self)");
    assert_eq!(
        (
            normalized_implementation.len(),
            fnv1a(&normalized_implementation)
        ),
        (140, 0x83b9_060d_06f3_fd6b)
    );
}

#[test]
fn array_buffer_slice_bound_owns_the_missing_or_undefined_default() {
    let signature = bounded(
        BINARY_DATA_SOURCE,
        &format!("\n    pub(super) fn {BOUND_HELPER}"),
        ") -> Result<(), EmitError> {",
    );
    assert_eq!(
        without_whitespace(signature),
        without_whitespace(
            r#"(
                &mut self,
                bound: ArrayBufferSliceBound,
                length_local: u32,
                dest_local: u32,
                function: &mut Function,
            "#,
        ),
        "the bound helper must expose only the closed role and its data locals"
    );
    assert!(!signature.contains(": bool"));
    assert!(!signature.contains("arg_index: usize"));
    assert!(!signature.contains("default_to_length"));

    let helper = bounded(
        BINARY_DATA_SOURCE,
        &format!("\n    pub(super) fn {BOUND_HELPER}("),
        "\n    pub(crate) fn emit_array_buffer_transfer_length_to_local(",
    );
    assert_eq!(helper.matches("match &bound {").count(), 1);
    assert_eq!(
        helper
            .matches("let arg_index = bound.argument_index();")
            .count(),
        1,
        "the caller-independent role must derive its argument position once"
    );
    assert!(!helper.contains("default_to_length"));
    assert!(!helper.contains("if bound"));
    assert!(!helper.contains("matches!(bound"));

    assert_normalized_once(
        helper,
        r#"
        let arg_index = bound.argument_index();

        self.emit_builtin_arg_to_locals(arg_index, payload_local, tag_local, function);
        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Const(arg_index as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        match &bound {
        "#,
        "argument lookup and absent-or-undefined detection must precede the default projection",
    );

    let defaults = bounded(
        helper,
        "\n        match &bound {",
        "\n        function.instruction(&Instruction::LocalSet(dest_local));",
    );
    assert_eq!(
        without_whitespace(defaults),
        without_whitespace(
            r#"
            ArrayBufferSliceBound::Start => {
                function.instruction(&Instruction::I64Const(0));
            }
            ArrayBufferSliceBound::End => {
                function.instruction(&Instruction::LocalGet(length_local));
            }
            }
            "#,
        ),
        "Start must default to zero and End must default to the entry length"
    );
    assert_eq!(defaults.matches("=>").count(), 2);
    assert!(!defaults.contains("_ =>"));

    let default_position = helper
        .find("match &bound {")
        .expect("missing bound default projection");
    let conversion_position = helper
        .find("self.emit_value_to_number_payload(tag_local, payload_local, function)?;")
        .expect("missing explicit-bound numeric conversion");
    assert!(
        default_position < conversion_position,
        "the default arm must remain before the explicit-bound conversion path"
    );

    let normalized_helper = helper.replace("match &bound {", "match bound {");
    assert_eq!((helper.len(), fnv1a(helper)), (5366, 0x4424_aea8_6175_c9ce));
    assert_eq!(
        (normalized_helper.len(), fnv1a(&normalized_helper)),
        (5365, 0x8d95_4656_6782_1a54)
    );
}

#[test]
fn grouped_slice_builtins_have_exactly_one_start_then_one_end_call() {
    assert_normalized_once(
        STANDARD_SOURCE,
        r#"
        StandardBuiltinId::ArrayBufferPrototypeSlice
        | StandardBuiltinId::SharedArrayBufferPrototypeSlice
        | StandardBuiltinId::ArrayBufferPrototypeSliceToImmutable => {
        "#,
        "ordinary, shared and immutable slice must retain one grouped owner body",
    );

    let slice_body = bounded(
        STANDARD_SOURCE,
        "\n            StandardBuiltinId::ArrayBufferPrototypeSlice\n",
        "\n            StandardBuiltinId::ArrayBufferPrototypeTransfer\n",
    );
    let normalized_body = without_whitespace(slice_body);
    assert_eq!(
        slice_body.matches(&format!("{BOUND_HELPER}(")).count(),
        2,
        "the grouped body must normalize exactly Start and End"
    );
    assert_normalized_once(
        slice_body,
        r#"
        self.emit_array_buffer_slice_index_to_local(
            ArrayBufferSliceBound::Start,
            byte_length_local,
            start_local,
            function,
        )?;
        "#,
        "Start must be the sole writer selected for start_local",
    );
    assert_normalized_once(
        slice_body,
        r#"
        self.emit_array_buffer_slice_index_to_local(
            ArrayBufferSliceBound::End,
            byte_length_local,
            end_local,
            function,
        )?;
        "#,
        "End must be the sole writer selected for end_local",
    );
    assert_normalized_once(
        slice_body,
        r#"
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I64GtU);
        "#,
        "the requested-length comparison must consume End then Start",
    );
    assert_normalized_once(
        slice_body,
        r#"
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I64Sub);
        "#,
        "the positive requested length must subtract Start from End",
    );
    assert_normalized_once(
        slice_body,
        r#"
        ArrayBufferSliceCopyLocals::new(
            receiver_payload_local,
            start_local,
            end_local,
            new_len_local,
        )
        "#,
        "the copy carrier must receive the immutable normalized Start and End locals",
    );

    for local in ["start_local", "end_local"] {
        assert_eq!(
            exact_identifier_mentions(slice_body, local),
            6,
            "{local} must occur only in its reservation, bound projection, two requested-length reads, copy carrier and release"
        );
        assert_eq!(
            normalized_body
                .matches(&format!("let{local}=self.reserve_temp_local();"))
                .count(),
            1,
            "{local} must have one reservation"
        );
        assert_eq!(
            normalized_body
                .matches(&format!("Instruction::LocalGet({local})"))
                .count(),
            2,
            "{local} must be read only by the requested-length calculation"
        );
        assert!(
            !normalized_body.contains(&format!("Instruction::LocalSet({local})")),
            "{local} must not have a direct or later writer"
        );
        assert_eq!(
            normalized_body
                .matches(&format!("self.release_temp_local({local});"))
                .count(),
            1,
            "{local} must have one release"
        );
    }
    assert!(!slice_body.contains("default_to_length"));
    assert!(!slice_body.contains("emit_builtin_arg_to_locals("));
    assert!(!slice_body.contains("emit_value_to_number_payload("));

    for mapping in [
        "StandardBuiltinId::ArrayBufferPrototypeSlice => ArrayBufferSliceKind::Ordinary",
        r#"StandardBuiltinId::SharedArrayBufferPrototypeSlice => {
            ArrayBufferSliceKind::Shared
        }"#,
        r#"StandardBuiltinId::ArrayBufferPrototypeSliceToImmutable => {
            ArrayBufferSliceKind::ToImmutable
        }"#,
    ] {
        assert_normalized_once(
            slice_body,
            mapping,
            "every grouped builtin owner must retain its exact slice-kind projection",
        );
    }
    for builtin in [
        "ArrayBufferPrototypeSlice",
        "SharedArrayBufferPrototypeSlice",
        "ArrayBufferPrototypeSliceToImmutable",
    ] {
        assert_eq!(
            exact_identifier_mentions(
                STANDARD_SOURCE,
                &format!("StandardBuiltinId::{builtin}"),
            ),
            2,
            "StandardBuiltinId::{builtin} must occur only in the grouped owner and its inner kind projection"
        );
    }

    let entry_length_marker = without_whitespace(
        r#"
        self.emit_load_array_buffer_byte_length(
            receiver_payload_local,
            byte_length_local,
            function,
        );
        "#,
    );
    let entry_length = normalized_body
        .find(entry_length_marker.as_str())
        .expect("missing entry byte-length observation");
    let start = normalized_body
        .find("self.emit_array_buffer_slice_index_to_local(ArrayBufferSliceBound::Start")
        .expect("missing Start normalization");
    let end = normalized_body
        .find("self.emit_array_buffer_slice_index_to_local(ArrayBufferSliceBound::End")
        .expect("missing End normalization");
    let requested_length = normalized_body
        .find("Instruction::LocalSet(new_len_local)")
        .expect("missing requested-length calculation");
    let target_work = normalized_body
        .find("ifslice_kind.uses_species(){")
        .expect("missing species/target work");
    let copy = normalized_body
        .find("self.emit_array_buffer_slice_copy(")
        .expect("missing policy-specific slice copy");
    assert!(
        entry_length < start
            && start < end
            && end < requested_length
            && requested_length < target_work
            && target_work < copy,
        "entry length, Start, End, requested length, target work and copy must remain ordered"
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut sources = Vec::new();
    collect_rust_sources(&source_root, &mut sources);

    let mut total_mentions = 0;
    let mut total_bound_mentions = 0;
    for (path, source) in sources {
        let relative = path
            .strip_prefix(&source_root)
            .expect("collected source must remain below src")
            .to_string_lossy();
        let expected_mentions = match relative.as_ref() {
            "builtins/binary_data.rs" => 1,
            "builtins/standard.rs" => 2,
            _ => 0,
        };
        let mentions = source.matches(&format!("{BOUND_HELPER}(")).count();
        assert_eq!(
            mentions, expected_mentions,
            "unexpected slice-bound helper definition or caller inventory in {relative}"
        );
        total_mentions += mentions;

        let expected_bound_mentions = match relative.as_ref() {
            "builtins/binary_data.rs" => 5,
            "builtins/standard.rs" => 3,
            _ => 0,
        };
        let bound_mentions = exact_identifier_mentions(&source, "ArrayBufferSliceBound");
        assert_eq!(
            bound_mentions, expected_bound_mentions,
            "unexpected ArrayBufferSliceBound producer or consumer in {relative}"
        );
        total_bound_mentions += bound_mentions;
    }
    assert_eq!(
        total_mentions, 3,
        "the slice-bound helper must have one definition and exactly two calls"
    );
    assert_eq!(
        total_bound_mentions, 8,
        "the declaration, impl, owned helper boundary, two projections, import and two producers must own every mention"
    );
}
