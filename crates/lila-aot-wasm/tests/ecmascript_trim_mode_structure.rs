use std::fs;
use std::path::{Path, PathBuf};

const OPERATIONS_SOURCE: &str = include_str!("../src/operations.rs");
const STRING_TRIM_SOURCE: &str = include_str!("../src/operations/string_trim.rs");
const BUILTINS_SOURCE: &str = include_str!("../src/builtins/mod.rs");
const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const STRING_INTRINSICS_SOURCE: &str = include_str!("../src/intrinsics/string.rs");
const HOST_BUILTINS_SOURCE: &str = include_str!("../src/builtins/host.rs");
const CONTRACT: &str = include_str!("../../../docs/rust-rewrite/contracts/ecmascript-trim-mode.md");
const TASK: &str = include_str!("../../../tasks/02-modularize-ir-and-wasm-backend.md");

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

fn bounded_inclusive<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_offset = source
        .find(start)
        .unwrap_or_else(|| panic!("missing start: {start}"));
    let end_offset = source[start_offset..]
        .find(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"));
    &source[start_offset..start_offset + end_offset]
}

fn quoted_literal_end(source: &str, quote_start: usize, quote: u8) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut offset = quote_start + 1;
    let mut escaped = false;
    while offset < bytes.len() {
        let byte = bytes[offset];
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == quote {
            return Some(offset + 1);
        }
        offset += 1;
    }
    None
}

fn character_literal_end(source: &str, start: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let value_start = start + 1;
    if value_start >= bytes.len() {
        return None;
    }
    let value_end = if bytes[value_start] == b'\\' {
        let mut offset = value_start + 1;
        if offset >= bytes.len() {
            return None;
        }
        if bytes[offset] == b'u' && bytes.get(offset + 1) == Some(&b'{') {
            offset += 2;
            while bytes.get(offset).is_some_and(|byte| *byte != b'}') {
                offset += 1;
            }
            if bytes.get(offset) != Some(&b'}') {
                return None;
            }
            offset + 1
        } else if bytes[offset] == b'x'
            && bytes
                .get(offset + 1..offset + 3)
                .is_some_and(|digits| digits.iter().all(u8::is_ascii_hexdigit))
        {
            offset + 3
        } else {
            offset + 1
        }
    } else {
        value_start + source[value_start..].chars().next()?.len_utf8()
    };
    (bytes.get(value_end) == Some(&b'\'')).then_some(value_end + 1)
}

fn raw_literal_end(source: &str, start: usize, prefix_len: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut quote_start = start + prefix_len;
    while bytes.get(quote_start) == Some(&b'#') {
        quote_start += 1;
    }
    if bytes.get(quote_start) != Some(&b'"') {
        return None;
    }
    let hashes = quote_start - start - prefix_len;
    let mut offset = quote_start + 1;
    while offset < bytes.len() {
        if bytes[offset] == b'"'
            && bytes
                .get(offset + 1..offset + 1 + hashes)
                .is_some_and(|suffix| suffix.iter().all(|byte| *byte == b'#'))
        {
            return Some(offset + 1 + hashes);
        }
        offset += 1;
    }
    None
}

fn literal_end(source: &str, start: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    match bytes.get(start).copied()? {
        b'"' => quoted_literal_end(source, start, b'"'),
        b'\'' => character_literal_end(source, start),
        b'b' if bytes.get(start + 1) == Some(&b'\'') => character_literal_end(source, start + 1),
        b'b' | b'c' if bytes.get(start + 1) == Some(&b'"') => {
            quoted_literal_end(source, start + 1, b'"')
        }
        b'r' => raw_literal_end(source, start, 1),
        b'b' | b'c' if bytes.get(start + 1) == Some(&b'r') => raw_literal_end(source, start, 2),
        _ => None,
    }
}

struct NormalizedRust {
    code: String,
    identifiers: String,
    routes: String,
}

fn normalize_rust(source: &str) -> NormalizedRust {
    let bytes = source.as_bytes();
    let mut code = String::new();
    let mut identifiers = String::new();
    let mut routes = String::new();
    let mut offset = 0;
    while offset < bytes.len() {
        if let Some(end) = literal_end(source, offset) {
            code.push_str(&source[offset..end]);
            identifiers.push(' ');
            routes.push('L');
            offset = end;
            continue;
        }
        if bytes.get(offset..offset + 2) == Some(b"//") {
            identifiers.push(' ');
            offset += 2;
            while bytes.get(offset).is_some_and(|byte| *byte != b'\n') {
                offset += 1;
            }
            continue;
        }
        if bytes.get(offset..offset + 2) == Some(b"/*") {
            identifiers.push(' ');
            offset += 2;
            let mut depth = 1;
            while offset < bytes.len() && depth != 0 {
                if bytes.get(offset..offset + 2) == Some(b"/*") {
                    depth += 1;
                    offset += 2;
                } else if bytes.get(offset..offset + 2) == Some(b"*/") {
                    depth -= 1;
                    offset += 2;
                } else {
                    offset += 1;
                }
            }
            assert_eq!(depth, 0, "unterminated block comment in Rust source");
            continue;
        }
        if bytes.get(offset..offset + 2) == Some(b"r#")
            && source[offset + 2..]
                .chars()
                .next()
                .is_some_and(|character| character == '_' || character.is_alphabetic())
        {
            offset += 2;
            continue;
        }
        let character = source[offset..].chars().next().unwrap();
        if character.is_whitespace() {
            identifiers.push(' ');
        } else {
            code.push(character);
            identifiers.push(character);
            routes.push(character);
        }
        offset += character.len_utf8();
    }
    NormalizedRust {
        code,
        identifiers,
        routes,
    }
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
    let normalized = normalize_rust(source);
    let identifiers = &normalized.identifiers;
    normalized
        .identifiers
        .match_indices(identifier)
        .filter(|(index, _)| {
            let before = identifiers[..*index].chars().next_back();
            let after = identifiers[*index + identifier.len()..].chars().next();
            let identifier_char = |ch: char| ch == '_' || ch.is_ascii_alphanumeric();
            before.is_none_or(|ch| !identifier_char(ch))
                && after.is_none_or(|ch| !identifier_char(ch))
        })
        .count()
}

fn route_occurrences(source: &str, route: &str) -> usize {
    let normalized = normalize_rust(source);
    normalized
        .routes
        .match_indices(route)
        .filter(|(index, _)| {
            let routes = &normalized.routes;
            let before = routes[..*index].chars().next_back();
            let after = routes[*index + route.len()..].chars().next();
            [before, after].into_iter().all(|edge| {
                edge.map(|character| !character.is_alphanumeric() && character != '_')
                    .unwrap_or(true)
            })
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
    let lexical_probe = r###"
        EcmaTrimMode /* nested /* ignored */ comment */ :: r#Start;
        // EcmaTrimMode::End
        "EcmaTrimMode::Both";
        r#"EcmaTrimMode::Both"#;
        struct r#EcmaTrimMode;
    "###;
    let lexical_probe = normalize_rust(lexical_probe);
    assert_eq!(
        lexical_probe.routes,
        "EcmaTrimMode::Start;L;L;structEcmaTrimMode;"
    );
    assert!(lexical_probe.code.contains("r#\"EcmaTrimMode::Both\"#"));
    assert_eq!(
        identifier_occurrences(&lexical_probe.identifiers, "EcmaTrimMode"),
        2
    );

    assert_eq!(OPERATIONS_SOURCE.matches("mod string_trim;").count(), 1);
    assert!(!OPERATIONS_SOURCE.contains("pub mod string_trim;"));
    assert!(!OPERATIONS_SOURCE.contains("pub(crate) mod string_trim;"));
    assert!(!OPERATIONS_SOURCE.contains("enum EcmaTrimMode"));
    assert!(!OPERATIONS_SOURCE.contains(&format!("fn {RAW_TRIM_HELPER}(")));

    let declaration_prefix = STRING_TRIM_SOURCE
        .split_once("enum EcmaTrimMode {")
        .expect("EcmaTrimMode declaration")
        .0;
    let whitespace_table_prefix = declaration_prefix
        .split_once("const ECMASCRIPT_NON_ASCII_WHITESPACE_UTF8")
        .expect("whitespace table before EcmaTrimMode")
        .0;
    assert_eq!(
        normalize_rust(whitespace_table_prefix).routes,
        "usesuper::*;"
    );
    let mode = bounded(STRING_TRIM_SOURCE, "\nenum EcmaTrimMode {", "\n}");
    assert_eq!(
        normalize_rust(mode).routes,
        "Start,End,Both,",
        "TrimString's where-domain must remain exactly start, end and start+end"
    );
    assert!(STRING_TRIM_SOURCE.contains("\nenum EcmaTrimMode {"));
    assert!(!STRING_TRIM_SOURCE.contains("\npub(crate) enum EcmaTrimMode {"));
    assert!(!STRING_TRIM_SOURCE.contains("\npub(super) enum EcmaTrimMode {"));
    assert!(!STRING_TRIM_SOURCE.contains("\npub enum EcmaTrimMode {"));

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut sources = Vec::new();
    collect_rust_sources(&source_root, &mut sources);
    assert_eq!(
        sources
            .iter()
            .map(|(_, source)| identifier_occurrences(source, "EcmaTrimMode"))
            .sum::<usize>(),
        11,
        "the declaration, typed parameter, three wrappers and six exhaustive arms must be the complete source census"
    );
    for variant in ["Start", "End", "Both"] {
        let route = format!("EcmaTrimMode::{variant}");
        assert_eq!(
            1 + sources
                .iter()
                .map(|(_, source)| route_occurrences(source, &route))
                .sum::<usize>(),
            4,
            "{variant} must have one declaration row, one wrapper and one arm in each scan"
        );
    }
    let all_routes = sources
        .iter()
        .map(|(_, source)| normalize_rust(source).routes)
        .collect::<String>();
    for capability in ["Clone", "Copy", "Debug", "Default", "PartialEq", "Eq"] {
        assert!(!all_routes.contains(&format!("impl{capability}forEcmaTrimMode")));
    }

    assert_eq!(
        STRING_TRIM_SOURCE
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
            !STRING_TRIM_SOURCE.contains(&public_spelling),
            "the raw trim core must not escape its private owner"
        );
    }

    let start_wrapper = bounded(
        STRING_TRIM_SOURCE,
        &format!("\n    pub(crate) fn {START_TRIM_WRAPPER}("),
        &format!("\n\n    pub(crate) fn {END_TRIM_WRAPPER}("),
    );
    let end_wrapper = bounded(
        STRING_TRIM_SOURCE,
        &format!("\n    pub(crate) fn {END_TRIM_WRAPPER}("),
        &format!("\n\n    pub(crate) fn {BOTH_TRIM_WRAPPER}("),
    );
    let both_wrapper = bounded(
        STRING_TRIM_SOURCE,
        &format!("\n    pub(crate) fn {BOTH_TRIM_WRAPPER}("),
        &format!("\n\n    fn {RAW_TRIM_HELPER}("),
    );
    assert_wrapper_forwards_only(start_wrapper, "Start", "start-only");
    assert_wrapper_forwards_only(end_wrapper, "End", "end-only");
    assert_wrapper_forwards_only(both_wrapper, "Both", "both-ends");
    for (start, end, variant) in [
        (
            "    pub(crate) fn emit_ecmascript_trim_start_payload_from_locals(",
            "    pub(crate) fn emit_ecmascript_trim_end_payload_from_locals(",
            "Start",
        ),
        (
            "    pub(crate) fn emit_ecmascript_trim_end_payload_from_locals(",
            "    pub(crate) fn emit_ecmascript_trim_both_payload_from_locals(",
            "End",
        ),
        (
            "    pub(crate) fn emit_ecmascript_trim_both_payload_from_locals(",
            "    fn emit_ecmascript_trim_payload_from_locals(",
            "Both",
        ),
    ] {
        let wrapper = bounded_inclusive(STRING_TRIM_SOURCE, start, end);
        let method = match variant {
            "Start" => START_TRIM_WRAPPER,
            "End" => END_TRIM_WRAPPER,
            "Both" => BOTH_TRIM_WRAPPER,
            _ => unreachable!(),
        };
        assert_eq!(
            normalize_rust(wrapper).routes,
            format!(
                "pub(crate)fn{method}(&mutself,string_payload_local:u32,function:&mutFunction,)->Result<(),EmitError>{{self.{RAW_TRIM_HELPER}(string_payload_local,EcmaTrimMode::{variant},function,)}}"
            )
        );
    }

    let raw_signature = bounded(
        STRING_TRIM_SOURCE,
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
        STRING_TRIM_SOURCE,
        &format!("\n    fn {RAW_TRIM_HELPER}("),
        "\n    }\n}\n",
    );
    assert!(!raw_core.contains(": bool"));
    assert!(!raw_core.contains("trim_start"));
    assert!(!raw_core.contains("trim_end"));
    assert!(!raw_core.contains("if mode"));
    assert!(!raw_core.contains("matches!(mode"));
    let normalized_raw_core = normalize_rust(raw_core).routes;
    assert_eq!(normalized_raw_core.matches("match&mode{").count(), 1);
    assert_eq!(normalized_raw_core.matches("matchmode{").count(), 1);
    for forbidden in [
        "matches!(mode",
        "mode==",
        "mode!=",
        "_=>",
        "Default::default",
    ] {
        assert!(!normalized_raw_core.contains(forbidden), "{forbidden}");
    }

    let start_projection =
        bounded_inclusive(raw_core, "        match &mode {", "        match mode {");
    let expected_start_projection = r#"
        match &mode {
            EcmaTrimMode::Start | EcmaTrimMode::Both => {
                function.instruction(&Instruction::Block(BlockType::Empty));
                function.instruction(&Instruction::Loop(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(start_local));
                function.instruction(&Instruction::LocalGet(end_local));
                function.instruction(&Instruction::I64GeU);
                function.instruction(&Instruction::BrIf(1));
                function.instruction(&Instruction::LocalGet(start_local));
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::LocalSet(byte_local));
                for bytes in ECMASCRIPT_NON_ASCII_WHITESPACE_UTF8 {
                    Self::emit_skip_utf8_whitespace_forward(
                        function,
                        end_local,
                        start_local,
                        byte_local,
                        bytes,
                    );
                }
                self.emit_is_ascii_whitespace_i32(byte_local, function);
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::BrIf(1));
                function.instruction(&Instruction::LocalGet(start_local));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::LocalSet(start_local));
                function.instruction(&Instruction::Br(0));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
            }
            EcmaTrimMode::End => {}
        }
    "#;
    assert_eq!(
        normalize_rust(start_projection).routes,
        normalize_rust(expected_start_projection).routes,
        "the borrowed start scan must retain its complete body and order"
    );

    let final_slice_start =
        "        function.instruction(&Instruction::LocalGet(end_local));\n        function.instruction(&Instruction::LocalGet(start_local));\n        function.instruction(&Instruction::I64Sub);";
    let end_projection = bounded_inclusive(raw_core, "        match mode {", final_slice_start);
    let expected_end_projection = r#"
        match mode {
            EcmaTrimMode::End | EcmaTrimMode::Both => {
                function.instruction(&Instruction::Block(BlockType::Empty));
                function.instruction(&Instruction::Loop(BlockType::Empty));
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
                for bytes in ECMASCRIPT_NON_ASCII_WHITESPACE_UTF8 {
                    Self::emit_skip_utf8_whitespace_backward(
                        function,
                        start_local,
                        end_local,
                        byte_local,
                        bytes,
                    );
                }
                self.emit_is_ascii_whitespace_i32(byte_local, function);
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::BrIf(1));
                function.instruction(&Instruction::LocalGet(index_local));
                function.instruction(&Instruction::LocalSet(end_local));
                function.instruction(&Instruction::Br(0));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
            }
            EcmaTrimMode::Start => {}
        }
    "#;
    assert_eq!(
        normalize_rust(end_projection).routes,
        normalize_rust(expected_end_projection).routes,
        "the consuming end scan must retain its complete body and order"
    );
    assert!(raw_core.find("match &mode {").unwrap() < raw_core.find("match mode {").unwrap());
    assert!(raw_core.find("match mode {").unwrap() < raw_core.find(final_slice_start).unwrap());

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
fn ecmascript_trim_whitespace_table_is_private_complete_and_single_owned() {
    const TABLE_NAME: &str = "ECMASCRIPT_NON_ASCII_WHITESPACE_UTF8";

    assert!(!BUILTINS_SOURCE.contains(TABLE_NAME));
    assert_eq!(
        STRING_TRIM_SOURCE
            .matches("const ECMASCRIPT_NON_ASCII_WHITESPACE_UTF8: [&[u8]; 19] = [")
            .count(),
        1
    );
    for visibility in ["pub const", "pub(crate) const", "pub(super) const"] {
        assert!(!STRING_TRIM_SOURCE.contains(&format!("{visibility} {TABLE_NAME}")));
    }

    let table = bounded(
        STRING_TRIM_SOURCE,
        "const ECMASCRIPT_NON_ASCII_WHITESPACE_UTF8: [&[u8]; 19] = [",
        "];",
    );
    assert_eq!(
        normalize_rust(table).routes,
        "&[0xC2,0xA0],&[0xE1,0x9A,0x80],&[0xE2,0x80,0x80],&[0xE2,0x80,0x81],&[0xE2,0x80,0x82],&[0xE2,0x80,0x83],&[0xE2,0x80,0x84],&[0xE2,0x80,0x85],&[0xE2,0x80,0x86],&[0xE2,0x80,0x87],&[0xE2,0x80,0x88],&[0xE2,0x80,0x89],&[0xE2,0x80,0x8A],&[0xE2,0x80,0xA8],&[0xE2,0x80,0xA9],&[0xE2,0x80,0xAF],&[0xE2,0x81,0x9F],&[0xE3,0x80,0x80],&[0xEF,0xBB,0xBF],"
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut sources = Vec::new();
    collect_rust_sources(&source_root, &mut sources);
    assert_eq!(
        sources
            .iter()
            .map(|(_, source)| identifier_occurrences(source, TABLE_NAME))
            .sum::<usize>(),
        3
    );
    let owners = sources
        .iter()
        .filter(|(_, source)| identifier_occurrences(source, TABLE_NAME) != 0)
        .map(|(path, _)| {
            path.strip_prefix(&source_root)
                .unwrap_or(path)
                .to_path_buf()
        })
        .collect::<Vec<_>>();
    assert_eq!(owners, vec![PathBuf::from("operations/string_trim.rs")]);

    for evidence in [CONTRACT, TASK] {
        assert!(evidence.contains("owner-private `ECMASCRIPT_NON_ASCII_WHITESPACE_UTF8`"));
        assert!(
            evidence.contains("3b3f4cb67213c7881b83d193a979ff4ae654805c1e7c783c473d781eb5395bd8")
        );
        assert!(evidence.contains("no new String behavior"));
    }
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
        let compact = normalize_rust(source).routes;
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
            .map(|(_, source)| {
                normalize_rust(source)
                    .routes
                    .matches(&format!("{identifier}("))
                    .count()
            })
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
        assert!(sources.iter().all(|(_, source)| {
            !normalize_rust(source)
                .routes
                .contains(&format!("FunctionBuilder::{identifier}"))
        }));
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
        vec![PathBuf::from("operations/string_trim.rs")],
        "only the private string-trim owner may name the raw trim core"
    );
}
