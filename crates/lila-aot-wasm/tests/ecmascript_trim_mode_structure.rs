use std::fs;
use std::path::{Path, PathBuf};

const OPERATIONS_SOURCE: &str = include_str!("../src/operations.rs");
const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const STRING_INTRINSICS_SOURCE: &str = include_str!("../src/intrinsics/string.rs");
const HOST_BUILTINS_SOURCE: &str = include_str!("../src/builtins/host.rs");

const RAW_TRIM_HELPER: &str = "emit_ecmascript_trim_payload_from_locals";
const START_TRIM_WRAPPER: &str = "emit_ecmascript_trim_start_payload_from_locals";
const END_TRIM_WRAPPER: &str = "emit_ecmascript_trim_end_payload_from_locals";
const BOTH_TRIM_WRAPPER: &str = "emit_ecmascript_trim_both_payload_from_locals";

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

fn identifier_occurrences(source: &str, identifier: &str) -> usize {
    source
        .match_indices(identifier)
        .filter(|(index, _)| {
            let before = source[..*index].chars().next_back();
            let after = source[*index + identifier.len()..].chars().next();
            let identifier_char = |ch: char| ch == '_' || ch.is_ascii_alphanumeric();
            before.is_none_or(|ch| !identifier_char(ch))
                && after.is_none_or(|ch| !identifier_char(ch))
        })
        .count()
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

fn assert_wrapper_forwards_only(wrapper: &str, variant: &str, label: &str) {
    assert_eq!(
        wrapper.matches(&format!("{RAW_TRIM_HELPER}(")).count(),
        1,
        "{label} wrapper must call the private raw core exactly once"
    );
    assert_eq!(
        wrapper.matches("EcmaTrimMode::").count(),
        1,
        "{label} wrapper must select exactly one trim mode"
    );
    assert_eq!(
        wrapper.matches(&format!("EcmaTrimMode::{variant}")).count(),
        1,
        "{label} wrapper must select {variant}"
    );
    assert!(!wrapper.contains(": bool"));
    assert!(!wrapper.contains("trim_start"));
    assert!(!wrapper.contains("trim_end"));
}

#[test]
fn ecmascript_trim_mode_is_private_closed_and_exhaustive() {
    let mode = bounded(OPERATIONS_SOURCE, "\nenum EcmaTrimMode {", "\n}");
    assert_eq!(
        without_whitespace(mode),
        "Start,End,Both,",
        "TrimString's where-domain must remain exactly start, end and start+end"
    );
    assert!(OPERATIONS_SOURCE.contains("\nenum EcmaTrimMode {"));
    assert!(!OPERATIONS_SOURCE.contains("\npub(crate) enum EcmaTrimMode {"));
    assert!(!OPERATIONS_SOURCE.contains("\npub(super) enum EcmaTrimMode {"));
    assert!(!OPERATIONS_SOURCE.contains("\npub enum EcmaTrimMode {"));

    assert_eq!(
        OPERATIONS_SOURCE
            .matches(&format!("\n    fn {RAW_TRIM_HELPER}("))
            .count(),
        1,
        "the raw trim core must have one private definition"
    );
    for public_spelling in [
        format!("\n    pub(crate) fn {RAW_TRIM_HELPER}("),
        format!("\n    pub(super) fn {RAW_TRIM_HELPER}("),
        format!("\n    pub fn {RAW_TRIM_HELPER}("),
    ] {
        assert!(
            !OPERATIONS_SOURCE.contains(&public_spelling),
            "the raw trim core must not escape operations.rs"
        );
    }

    let start_wrapper = bounded(
        OPERATIONS_SOURCE,
        &format!("\n    pub(crate) fn {START_TRIM_WRAPPER}("),
        &format!("\n\n    pub(crate) fn {END_TRIM_WRAPPER}("),
    );
    let end_wrapper = bounded(
        OPERATIONS_SOURCE,
        &format!("\n    pub(crate) fn {END_TRIM_WRAPPER}("),
        &format!("\n\n    pub(crate) fn {BOTH_TRIM_WRAPPER}("),
    );
    let both_wrapper = bounded(
        OPERATIONS_SOURCE,
        &format!("\n    pub(crate) fn {BOTH_TRIM_WRAPPER}("),
        &format!("\n\n    fn {RAW_TRIM_HELPER}("),
    );
    assert_wrapper_forwards_only(start_wrapper, "Start", "start-only");
    assert_wrapper_forwards_only(end_wrapper, "End", "end-only");
    assert_wrapper_forwards_only(both_wrapper, "Both", "both-ends");

    let raw_signature = bounded(
        OPERATIONS_SOURCE,
        &format!("\n    fn {RAW_TRIM_HELPER}"),
        ") -> Result<(), EmitError> {",
    );
    assert_eq!(
        without_whitespace(raw_signature),
        without_whitespace(
            r#"(
                &mut self,
                string_payload_local: u32,
                mode: EcmaTrimMode,
                function: &mut Function,
            "#,
        ),
        "the private raw core must take one closed mode in the fixed position"
    );

    let raw_core = bounded(
        OPERATIONS_SOURCE,
        &format!("\n    fn {RAW_TRIM_HELPER}("),
        "\n    pub(crate) fn emit_copy_bytes(",
    );
    assert!(!raw_core.contains(": bool"));
    assert!(!raw_core.contains("trim_start"));
    assert!(!raw_core.contains("trim_end"));
    assert!(!raw_core.contains("if mode"));
    assert!(!raw_core.contains("matches!(mode"));
    assert_eq!(
        raw_core.matches("match mode {").count(),
        2,
        "the mode must be projected once for each boundary scan"
    );

    let projections = raw_core.split("match mode {").collect::<Vec<_>>();
    assert_eq!(
        projections.len(),
        3,
        "exactly two mode matches are required"
    );
    let start_projection = projections[1];
    let final_slice = r#"
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I64Sub);
    "#;
    let end_projection = projections[2]
        .split_once(final_slice)
        .expect("the second projection must precede the unchanged final slice")
        .0;

    let normalized_start = without_whitespace(start_projection);
    assert!(normalized_start.starts_with("EcmaTrimMode::Start|EcmaTrimMode::Both=>{"));
    assert!(normalized_start.ends_with("}EcmaTrimMode::End=>{}}"));
    assert_eq!(start_projection.matches("=>").count(), 2);
    assert_eq!(start_projection.matches("EcmaTrimMode::Start").count(), 1);
    assert_eq!(start_projection.matches("EcmaTrimMode::End").count(), 1);
    assert_eq!(start_projection.matches("EcmaTrimMode::Both").count(), 1);
    assert_eq!(
        start_projection
            .matches("emit_skip_utf8_whitespace_forward(")
            .count(),
        1
    );
    assert!(!start_projection.contains("emit_skip_utf8_whitespace_backward("));
    assert!(!start_projection.contains("_ =>"));
    assert_normalized_once(
        start_projection,
        r#"
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(byte_local));
        "#,
        "the start scan must test start against end and load from start",
    );
    assert_normalized_once(
        start_projection,
        r#"
        Self::emit_skip_utf8_whitespace_forward(
            function,
            end_local,
            start_local,
            byte_local,
            bytes,
        );
        "#,
        "the forward UTF-8 scan must receive end as its bound and start as its index",
    );
    assert_normalized_once(
        start_projection,
        r#"
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(start_local));
        "#,
        "the start scan must advance start by one ASCII byte",
    );

    let normalized_end = without_whitespace(end_projection);
    assert!(normalized_end.starts_with("EcmaTrimMode::End|EcmaTrimMode::Both=>{"));
    assert!(normalized_end.ends_with("}EcmaTrimMode::Start=>{}}"));
    assert_eq!(end_projection.matches("=>").count(), 2);
    assert_eq!(end_projection.matches("EcmaTrimMode::Start").count(), 1);
    assert_eq!(end_projection.matches("EcmaTrimMode::End").count(), 1);
    assert_eq!(end_projection.matches("EcmaTrimMode::Both").count(), 1);
    assert_eq!(
        end_projection
            .matches("emit_skip_utf8_whitespace_backward(")
            .count(),
        1
    );
    assert!(!end_projection.contains("emit_skip_utf8_whitespace_forward("));
    assert!(!end_projection.contains("_ =>"));
    assert_normalized_once(
        end_projection,
        r#"
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(byte_local));
        "#,
        "the end scan must test end against start and load from end minus one",
    );
    assert_normalized_once(
        end_projection,
        r#"
        Self::emit_skip_utf8_whitespace_backward(
            function,
            start_local,
            end_local,
            byte_local,
            bytes,
        );
        "#,
        "the backward UTF-8 scan must receive start as its bound and end as its index",
    );
    assert_normalized_once(
        end_projection,
        r#"
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalSet(end_local));
        "#,
        "the end scan must publish its decremented index as the new end",
    );

    let forward_helper = bounded(
        OPERATIONS_SOURCE,
        "\n    pub(crate) fn emit_skip_utf8_whitespace_forward(",
        "\n    pub(crate) fn emit_skip_utf8_whitespace_backward(",
    );
    let backward_helper = bounded(
        OPERATIONS_SOURCE,
        "\n    pub(crate) fn emit_skip_utf8_whitespace_backward(",
        "\n    pub(crate) fn compile_loose_equality_i32(",
    );
    assert!(without_whitespace(forward_helper).starts_with(
        "function:&mutFunction,end_local:u32,index_local:u32,byte_local:u32,bytes:&[u8],){"
    ));
    assert!(without_whitespace(backward_helper).starts_with(
        "function:&mutFunction,start_local:u32,end_local:u32,byte_local:u32,bytes:&[u8],){"
    ));
    assert_eq!(forward_helper.matches("Instruction::I64Add").count(), 3);
    assert_eq!(forward_helper.matches("Instruction::I64Sub").count(), 0);
    assert_eq!(backward_helper.matches("Instruction::I64Sub").count(), 3);
    assert_eq!(backward_helper.matches("Instruction::I64Add").count(), 1);
    assert_normalized_once(
        forward_helper,
        r#"
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const((bytes.len() - 1) as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::I64LtU);
        "#,
        "a forward UTF-8 candidate must remain wholly below end",
    );
    assert_normalized_once(
        forward_helper,
        r#"
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(bytes.len() as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        "#,
        "a forward UTF-8 match must advance its index by the matched byte length",
    );
    assert_normalized_once(
        backward_helper,
        r#"
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(bytes.len() as i64));
        function.instruction(&Instruction::I64GeU);
        "#,
        "a backward UTF-8 candidate must fit between start and end",
    );
    assert_normalized_once(
        backward_helper,
        r#"
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::I64Const(bytes.len() as i64));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(end_local));
        "#,
        "a backward UTF-8 match must retreat its end by the matched byte length",
    );
}

#[test]
fn ecmascript_trim_mode_callers_aliases_and_order_are_exact() {
    let string_to_bigint = bounded(
        OPERATIONS_SOURCE,
        "\n    pub(crate) fn emit_string_to_bigint_locals(",
        "\n    pub(crate) fn emit_nonstring_value_to_number_payload(",
    );
    assert_eq!(
        string_to_bigint
            .matches(&format!("{BOTH_TRIM_WRAPPER}("))
            .count(),
        1,
        "StringToBigInt must trim both ends exactly once"
    );
    assert!(!string_to_bigint.contains(&format!("{START_TRIM_WRAPPER}(")));
    assert!(!string_to_bigint.contains(&format!("{END_TRIM_WRAPPER}(")));
    assert!(!string_to_bigint.contains(&format!("{RAW_TRIM_HELPER}(")));
    let normalized_bigint = without_whitespace(string_to_bigint);
    let bigint_order = without_whitespace(
        r#"
        self.emit_ecmascript_trim_both_payload_from_locals(string_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(trimmed_string_payload_local));
        self.emit_unpack_string_payload(
            trimmed_string_payload_local,
            offset_local,
            len_local,
            function,
        );
        "#,
    );
    assert_eq!(
        normalized_bigint.matches(&bigint_order).count(),
        1,
        "StringToBigInt must capture the Both result before unpacking its parse source"
    );
    assert_eq!(
        string_to_bigint
            .matches("self.emit_unpack_string_payload(")
            .count(),
        1,
        "StringToBigInt must unpack exactly one source, so the original cannot replace the trimmed payload"
    );

    let method_call = bounded(
        FUNCTIONS_SOURCE,
        "\n    pub(crate) fn emit_method_call(",
        "\n    pub(crate) fn emit_call(",
    );
    let trim_fast_path = bounded(
        method_call,
        r#"if matches!(key, PropertyKeyIr::StaticString(name) if matches!(name.as_str(), "trim" | "trimStart" | "trimLeft" | "trimEnd" | "trimRight"))"#,
        "\n        let string_html_builtin = match key {",
    );
    for wrapper in [START_TRIM_WRAPPER, END_TRIM_WRAPPER, BOTH_TRIM_WRAPPER] {
        assert_eq!(
            trim_fast_path.matches(&format!("{wrapper}(")).count(),
            1,
            "static trim fast path must call {wrapper} exactly once"
        );
    }
    assert!(!trim_fast_path.contains(&format!("{RAW_TRIM_HELPER}(")));
    for operation in [
        "self.compile_expr_to_locals(",
        "self.compile_nullish_tagged_i32(",
        "self.emit_throw_runtime_error(",
        "self.emit_value_to_string_payload(",
    ] {
        assert_eq!(
            trim_fast_path.matches(operation).count(),
            1,
            "static trim path must emit {operation} exactly once"
        );
    }
    assert_normalized_once(
        trim_fast_path,
        r#"
        self.compile_expr_to_locals(
            receiver,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        self.compile_nullish_tagged_i32(receiver_tag_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "String.prototype method receiver is null or undefined",
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_value_to_string_payload(
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(string_local));
        match key {
            PropertyKeyIr::StaticString(name) => match name.as_str() {
                "trim" => {
                    self.emit_ecmascript_trim_both_payload_from_locals(string_local, function)?
                }
                "trimStart" | "trimLeft" => {
                    self.emit_ecmascript_trim_start_payload_from_locals(string_local, function)?
                }
                "trimEnd" | "trimRight" => {
                    self.emit_ecmascript_trim_end_payload_from_locals(string_local, function)?
                }
                _ => unreachable!("trim fast path requires a recognized static method name"),
            },
            _ => unreachable!("trim fast path requires a static method name"),
        }
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::End);
        "#,
        "the static trim path must retain receiver evaluation, nullish branching, exact ToString inputs, mode selection and result publication",
    );

    let static_builtin_mapping = without_whitespace(
        r#"
        "trim" => Some(StandardBuiltinId::StringPrototypeTrim),
        "trimStart" | "trimLeft" => Some(StandardBuiltinId::StringPrototypeTrimStart),
        "trimEnd" | "trimRight" => Some(StandardBuiltinId::StringPrototypeTrimEnd),
        "#,
    );
    assert_eq!(
        without_whitespace(method_call)
            .matches(static_builtin_mapping.as_str())
            .count(),
        1,
        "static trim aliases must also retain their exact builtin forwarding map"
    );

    let standard_trim = bounded(
        STANDARD_SOURCE,
        "\n            StandardBuiltinId::StringPrototypeTrim\n",
        "\n            StandardBuiltinId::ErrorPrototypeToString =>",
    );
    assert_normalized_once(
        STANDARD_SOURCE,
        r#"
        StandardBuiltinId::StringPrototypeTrim
        | StandardBuiltinId::StringPrototypeTrimStart
        | StandardBuiltinId::StringPrototypeTrimEnd => {
        "#,
        "the standard dispatcher trim arm must contain exactly the three trim builtin identities",
    );
    for wrapper in [START_TRIM_WRAPPER, END_TRIM_WRAPPER, BOTH_TRIM_WRAPPER] {
        assert_eq!(
            standard_trim.matches(&format!("{wrapper}(")).count(),
            1,
            "standard trim builtin family must call {wrapper} exactly once"
        );
    }
    assert!(!standard_trim.contains(&format!("{RAW_TRIM_HELPER}(")));
    for operation in [
        "self.compile_nullish_tagged_i32(",
        "self.emit_throw_current_function_realm_type_error(",
        "self.emit_value_to_string_payload(",
    ] {
        assert_eq!(
            standard_trim.matches(operation).count(),
            1,
            "standard trim path must emit {operation} exactly once"
        );
    }
    assert_normalized_once(
        standard_trim,
        r#"
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing String.prototype.trim receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing String.prototype.trim receiver",
            )
        })?;
        let string_local = self.reserve_temp_local();

        self.compile_nullish_tagged_i32(receiver_tag_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "String.prototype method receiver is null or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_value_to_string_payload(
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(string_local));
        match builtin {
            StandardBuiltinId::StringPrototypeTrim => {
                self.emit_ecmascript_trim_both_payload_from_locals(string_local, function)?
            }
            StandardBuiltinId::StringPrototypeTrimStart => {
                self.emit_ecmascript_trim_start_payload_from_locals(string_local, function)?
            }
            StandardBuiltinId::StringPrototypeTrimEnd => {
                self.emit_ecmascript_trim_end_payload_from_locals(string_local, function)?
            }
            _ => unreachable!("trim builtin arm requires a trim builtin identity"),
        }
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::End);
        "#,
        "the standard trim path must retain nullish branching, active-Realm error routing, exact ToString inputs, mode selection and result publication",
    );

    let intrinsic_aliases = bounded(
        STRING_INTRINSICS_SOURCE,
        "\n            match builtin {",
        "\n        let iterator_meta =",
    );
    let start_alias_install = without_whitespace(
        r#"
        StandardBuiltinId::StringPrototypeTrimStart => {
            self.emit_object_define_function_data_with_aliases(
                object_local,
                "trimStart",
                &["trimLeft"],
                meta,
                function,
            )?;
        }
        "#,
    );
    let end_alias_install = without_whitespace(
        r#"
        StandardBuiltinId::StringPrototypeTrimEnd => {
            self.emit_object_define_function_data_with_aliases(
                object_local,
                "trimEnd",
                &["trimRight"],
                meta,
                function,
            )?;
        }
        "#,
    );
    let normalized_intrinsic_aliases = without_whitespace(intrinsic_aliases);
    assert_eq!(
        normalized_intrinsic_aliases
            .matches(start_alias_install.as_str())
            .count(),
        1,
        "the intrinsic trimLeft alias must share the trimStart builtin identity"
    );
    assert_eq!(
        normalized_intrinsic_aliases
            .matches(end_alias_install.as_str())
            .count(),
        1,
        "the intrinsic trimRight alias must share the trimEnd builtin identity"
    );
    assert_eq!(
        intrinsic_aliases
            .matches("emit_object_define_function_data_with_aliases(")
            .count(),
        2,
        "only the two one-ended trim builtins may install aliases in this catalog"
    );

    let created_realm_alias_map = without_whitespace(
        r#"
        fn created_realm_string_prototype_method_aliases(name: &str) -> &'static [&'static str] {
            match name {
                "trimStart" => &["trimLeft"],
                "trimEnd" => &["trimRight"],
                _ => &[],
            }
        }
        "#,
    );
    assert_eq!(
        without_whitespace(HOST_BUILTINS_SOURCE)
            .matches(created_realm_alias_map.as_str())
            .count(),
        1,
        "created realms must retain the exact trimStart/trimEnd alias map"
    );
    assert_eq!(
        HOST_BUILTINS_SOURCE
            .matches("created_realm_string_prototype_method_aliases(")
            .count(),
        2,
        "the created-realm alias map must have one definition and one use"
    );

    let created_realm_string_metas = bounded(
        HOST_BUILTINS_SOURCE,
        "\n        let string_prototype_method_metas = [",
        "\n        let boolean_prototype_method_metas = [",
    );
    for (name, builtin) in [
        ("trim", "StringPrototypeTrim"),
        ("trimStart", "StringPrototypeTrimStart"),
        ("trimEnd", "StringPrototypeTrimEnd"),
    ] {
        let entry_start = without_whitespace(&format!(
            r#"(
                "{name}",
                self.functions
                    .get(&StandardBuiltinId::{builtin}.function_id())"#,
        ));
        assert_eq!(
            without_whitespace(created_realm_string_metas)
                .matches(entry_start.as_str())
                .count(),
            1,
            "created-realm {name} must retain the {builtin} identity"
        );
        assert_eq!(
            created_realm_string_metas
                .matches(&format!("\"{name}\""))
                .count(),
            1,
            "created-realm metadata must publish {name} exactly once"
        );
        assert_eq!(
            created_realm_string_metas
                .matches(&format!("StandardBuiltinId::{builtin}.function_id()"))
                .count(),
            1,
            "created-realm metadata must consume {builtin} exactly once"
        );
    }

    let created_realm_string_installer = bounded(
        HOST_BUILTINS_SOURCE,
        "\n        for (name, meta) in &string_prototype_method_metas {",
        "\n        for (name, meta) in &array_prototype_method_metas {",
    );
    assert_normalized_once(
        created_realm_string_installer,
        r#"
        self.emit_object_define_local_data(
            string_prototype_local,
            name,
            method_payload_local,
            tag_local,
            function,
        )?;
        for alias in created_realm_string_prototype_method_aliases(name) {
            self.emit_object_define_local_data(
                string_prototype_local,
                alias,
                method_payload_local,
                tag_local,
                function,
            )?;
        }
        "#,
        "created realms must look aliases up by canonical name and publish each alias with the same function payload and tag",
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut sources = Vec::new();
    collect_rust_sources(&source_root, &mut sources);
    for (path, source) in &sources {
        let compact = without_whitespace(source);
        assert!(
            !compact.contains("trim_start:bool")
                && !compact.contains("trim_end:bool")
                && !compact.contains("lettrim_start")
                && !compact.contains("lettrim_end"),
            "{} must not reconstruct the removed raw Boolean trim policy",
            path.display()
        );
    }
    let expected = [
        (RAW_TRIM_HELPER, 4_usize),
        (START_TRIM_WRAPPER, 3_usize),
        (END_TRIM_WRAPPER, 3_usize),
        (BOTH_TRIM_WRAPPER, 4_usize),
    ];
    for (identifier, expected_count) in expected {
        let direct_calls = sources
            .iter()
            .map(|(_, source)| source.matches(&format!("{identifier}(")).count())
            .sum::<usize>();
        let bare_identifiers = sources
            .iter()
            .map(|(_, source)| identifier_occurrences(source, identifier))
            .sum::<usize>();
        assert_eq!(
            direct_calls, expected_count,
            "{identifier} direct-call inventory changed"
        );
        assert_eq!(
            bare_identifiers, expected_count,
            "{identifier} must have no method-item alias, forwarding escape or hidden caller"
        );
        assert!(sources
            .iter()
            .all(|(_, source)| { !source.contains(&format!("FunctionBuilder::{identifier}")) }));
    }

    let raw_files = sources
        .iter()
        .filter(|(_, source)| identifier_occurrences(source, RAW_TRIM_HELPER) != 0)
        .map(|(path, _)| {
            path.strip_prefix(&source_root)
                .unwrap_or(path)
                .to_path_buf()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        raw_files,
        vec![PathBuf::from("operations.rs")],
        "only operations.rs may name the private raw trim core"
    );
}
