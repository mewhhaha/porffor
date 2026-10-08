use std::fs;
use std::path::Path;

const SOURCE: &str = include_str!("../src/builtins/string.rs");
const REJECTION_SOURCE: &str = include_str!("../src/builtins/runtime_semantics.rs");

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

fn lexically_normalized(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut normalized = String::new();
    let mut offset = 0;
    while offset < bytes.len() {
        if let Some(end) = literal_end(source, offset) {
            normalized.push_str(&source[offset..end]);
            offset = end;
            continue;
        }
        if bytes.get(offset..offset + 2) == Some(b"//") {
            offset += 2;
            while bytes.get(offset).is_some_and(|byte| *byte != b'\n') {
                offset += 1;
            }
            continue;
        }
        if bytes.get(offset..offset + 2) == Some(b"/*") {
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
            assert_eq!(depth, 0, "unterminated block comment in RegExp emitter");
            continue;
        }
        let character = source[offset..].chars().next().unwrap();
        if !character.is_whitespace() {
            normalized.push(character);
        }
        offset += character.len_utf8();
    }
    normalized
}

fn find_code_marker(source: &str, marker: &str, mut offset: usize) -> Option<usize> {
    while offset < source.len() {
        if let Some(end) = literal_end(source, offset) {
            offset = end;
            continue;
        }
        if source[offset..].starts_with(marker) {
            return Some(offset);
        }
        offset += source[offset..].chars().next().unwrap().len_utf8();
    }
    None
}

fn normalized_match_bodies(source: &str) -> Vec<String> {
    let source = lexically_normalized(source);
    let marker = "matchresult_mode{";
    let mut bodies = Vec::new();
    let mut search_offset = 0;

    while let Some(start) = find_code_marker(&source, marker, search_offset) {
        let brace_start = start + marker.len() - 1;
        let mut depth = 0;
        let mut end = None;
        let mut offset = brace_start;
        while offset < source.len() {
            if let Some(literal_end) = literal_end(&source, offset) {
                offset = literal_end;
                continue;
            }
            let character = source[offset..].chars().next().unwrap();
            match character {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(offset + character.len_utf8());
                        break;
                    }
                }
                _ => {}
            }
            offset += character.len_utf8();
        }
        let end = end.expect("unterminated RegExp result-mode match");
        bodies.push(source[start..end].to_string());
        search_offset = end;
    }

    bodies
}

fn fnv1a(source: &str) -> u64 {
    source.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100_0000_01b3)
    })
}

#[test]
fn regexp_exec_result_mode_is_the_exact_private_no_capability_domain() {
    let type_declaration = bounded(
        SOURCE,
        concat!(
            "pub(crate) enum UriCodecKind {\n",
            "    Uri,\n",
            "    Component,\n",
            "}\n\n",
        ),
        "\n\nenum StringSymbolHookOperation {",
    );
    assert_eq!(
        normalized(type_declaration),
        "enumRegExpExecResultMode{MatchArrayOrNull,Boolean,}"
    );
    assert!(!type_declaration.contains("#["));

    let normalized_source = normalized(SOURCE);
    for forbidden in [
        "implRegExpExecResultMode",
        "forRegExpExecResultMode",
        "RegExpExecResultMode==",
        "RegExpExecResultMode!=",
        "matches!(result_mode",
    ] {
        assert!(
            !normalized_source.contains(forbidden),
            "found forbidden RegExp exec result-mode capability `{forbidden}`"
        );
    }

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "RegExpExecResultMode"),
        16,
        "the declaration, two typed parameters, ten projection arms and three intrinsic producers own every mention"
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "RegExpExecResultMode::MatchArrayOrNull"),
        7
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "RegExpExecResultMode::Boolean"),
        6
    );
}

#[test]
fn regexp_exec_result_mode_is_forwarded_to_the_sole_compiled_matcher() {
    let lexical_probe = r###"
        // match result_mode { }
        const IGNORED: &str = r#"matchresult_mode{"#;
        match result_mode {
            RegExpExecResultMode::Boolean => ("literal { } space", b"}", b'}', '}'),
            /* { nested /* } */ } */
            RegExpExecResultMode::MatchArrayOrNull => r#"{ raw literal }"#,
        }
    "###;
    assert_eq!(
        normalized_match_bodies(lexical_probe),
        [concat!(
            "matchresult_mode{RegExpExecResultMode::Boolean=>",
            "(\"literal { } space\",b\"}\",b'}','}'),",
            "RegExpExecResultMode::MatchArrayOrNull=>r#\"{ raw literal }\"#,}"
        )]
    );

    let consumers = bounded(
        SOURCE,
        "    fn emit_regexp_prototype_exec_from_locals(",
        "    pub(crate) fn emit_concat_string_payloads_local(",
    );
    let wrapper = bounded(
        SOURCE,
        "    fn emit_regexp_prototype_exec_from_locals(",
        "    /// Turn one typed matcher failure",
    );
    let program = bounded(
        SOURCE,
        "    fn emit_regexp_exec_program_from_locals(",
        "    pub(crate) fn emit_concat_string_payloads_local(",
    );

    for (function, expected_signature) in [
        (
            wrapper,
            concat!(
                "&mutself,receiver_payload_local:u32,receiver_tag_local:u32,",
                "input_payload_local:u32,input_tag_local:u32,",
                "result_mode:RegExpExecResultMode,payload_local:u32,tag_local:u32,",
                "function:&mutFunction,"
            ),
        ),
        (
            program,
            concat!(
                "&mutself,receiver_payload_local:u32,receiver_tag_local:u32,",
                "input_payload_local:u32,last_index_local:u32,result_mode:&RegExpExecResultMode,",
                "payload_local:u32,tag_local:u32,function:&mutFunction,"
            ),
        ),
    ] {
        let signature = function
            .split_once(") -> Result<(), EmitError> {")
            .expect("missing RegExp result-mode consumer signature")
            .0;
        assert_eq!(normalized(signature), expected_signature);
    }

    assert_eq!(
        consumers
            .matches("result_mode: RegExpExecResultMode,")
            .count(),
        1
    );
    assert_eq!(
        consumers
            .matches("result_mode: &RegExpExecResultMode,")
            .count(),
        1
    );
    assert_eq!(consumers.matches("match result_mode {").count(), 5);
    assert_eq!(
        consumers
            .matches("RegExpExecResultMode::MatchArrayOrNull")
            .count(),
        5
    );
    assert_eq!(
        consumers.matches("RegExpExecResultMode::Boolean").count(),
        5
    );
    assert!(!consumers.contains("return_boolean"));
    assert!(!consumers.contains(": bool"));
    assert!(!consumers.contains("matches!(result_mode"));
    assert!(!consumers.contains("if result_mode"));
    assert!(!consumers.contains("_ =>"));
    assert!(!consumers.contains("unreachable!"));

    // The five retained projections span roughly fifteen thousand normalized bytes.
    // Length plus FNV-1a pins their complete text and order without duplicating
    // the emitter implementation inside this guard.
    let match_bodies = normalized_match_bodies(consumers);
    let body_fingerprints = match_bodies
        .iter()
        .map(|body| (body.len(), fnv1a(body)))
        .collect::<Vec<_>>();
    assert_eq!(
        body_fingerprints,
        [
            (1896, 0x7565_612a_66e6_8697),
            (3810, 0x3825_d41d_a8c9_0680),
            (1651, 0x50c5_fe96_fd50_bd24),
            (829, 0xc103_e8b7_a1ac_03ac),
            (7022, 0x8b27_49b3_f019_9de2),
        ],
        "the five compiled-matcher projections retain their complete normalized bodies and order after fallback retirement"
    );

    let wrapper = normalized(wrapper);
    assert_eq!(
        wrapper
            .matches("self.emit_regexp_exec_program_from_locals(")
            .count(),
        1
    );
    let program_call = concat!(
        "self.emit_regexp_exec_program_from_locals(receiver_payload_local,",
        "receiver_tag_local,input_payload_local,last_index_local,&result_mode,",
        "payload_local,tag_local,function,)?;"
    );
    assert_eq!(wrapper.matches(program_call).count(), 1);
    assert!(!wrapper.contains("matchresult_mode{"));
    for forbidden in [
        "emit_regexp_exec_simple_from_locals",
        "program_handled_local",
        "sticky_handled_local",
    ] {
        assert!(
            !SOURCE.contains(forbidden),
            "retired exec route: {forbidden}"
        );
    }
    let to_length = wrapper
        .find("self.emit_to_length_i64_from_value_locals_with_abrupt_route(")
        .expect("lastIndex must be converted before the compiled matcher");
    assert!(
        to_length < wrapper.find(program_call).unwrap(),
        "the matcher borrows the result authority only after observable ToLength"
    );

    let rejection = lexically_normalized(REJECTION_SOURCE);
    assert!(rejection.contains(concat!(
        "function.instruction(&Instruction::I64Const(rejection.abi_code()));",
        "function.instruction(&Instruction::Call(self.functions.",
        "reject_runtime_semantics_import_function_index(),));",
        "function.instruction(&Instruction::Unreachable);"
    )));
}

#[test]
fn exactly_three_intrinsic_producers_choose_their_named_result_modes() {
    let regexp_exec = bounded(
        SOURCE,
        "    fn emit_regexp_exec_from_locals(",
        "    pub(crate) fn emit_regexp_prototype_symbol_match_builtin(",
    );
    assert_eq!(
        regexp_exec
            .matches("RegExpExecResultMode::MatchArrayOrNull")
            .count(),
        1
    );
    assert!(!regexp_exec.contains("RegExpExecResultMode::Boolean"));
    let regexp_exec = normalized(regexp_exec);
    let callable = regexp_exec
        .find("self.emit_is_callable_i32(exec_tag_local,exec_payload_local,function)?;")
        .unwrap();
    let call = regexp_exec
        .find(concat!(
            "self.emit_function_or_proxy_call_leave_throw_completion(exec_payload_local,",
            "exec_tag_local,receiver_payload_local,receiver_tag_local,",
            "&[(input_payload_local,input_tag_local)],payload_local,tag_local,function,)?;"
        ))
        .unwrap();
    let propagation = regexp_exec
        .find("self.emit_propagate_throw_from_locals_if_needed(payload_local,tag_local,function)?;")
        .unwrap();
    let null_check = regexp_exec
        .find("Instruction::I64Const(ValueKind::Null.tag()asi64)")
        .unwrap();
    let object_check = regexp_exec
        .find("self.emit_is_heap_object_like_tag_i32(tag_local,function);")
        .unwrap();
    let type_error = regexp_exec
        .find("self.emit_throw_current_function_realm_type_error(")
        .unwrap();
    let fallback = regexp_exec
        .find(concat!(
            "Instruction::Else);self.emit_regexp_prototype_exec_from_locals(",
            "receiver_payload_local,receiver_tag_local,input_payload_local,input_tag_local,",
            "RegExpExecResultMode::MatchArrayOrNull,payload_local,tag_local,function,)?;"
        ))
        .unwrap();
    assert!(callable < call && call < propagation && propagation < null_check);
    assert!(null_check < object_check && object_check < type_error && type_error < fallback);
    assert_eq!(
        regexp_exec
            .matches("self.emit_function_or_proxy_call_leave_throw_completion(")
            .count(),
        1
    );
    assert!(
        !regexp_exec.contains("self.emit_object_read("),
        "the abstract operation consumes the already-read exec value"
    );

    let symbol_match = bounded(
        SOURCE,
        "    pub(crate) fn emit_regexp_prototype_symbol_match_builtin(",
        "    pub(crate) fn emit_regexp_prototype_symbol_match_all_builtin(",
    );
    assert!(!symbol_match.contains("RegExpExecResultMode"));
    assert_eq!(
        symbol_match
            .matches("self.emit_regexp_exec_from_locals(")
            .count(),
        2,
        "global and non-global @@match both use the same cached-exec abstract operation"
    );

    let exec = bounded(
        SOURCE,
        "    pub(crate) fn emit_regexp_prototype_exec_builtin(",
        "    pub(crate) fn emit_regexp_prototype_test_builtin(",
    );
    assert_eq!(
        exec.matches("RegExpExecResultMode::MatchArrayOrNull")
            .count(),
        1
    );
    assert!(!exec.contains("RegExpExecResultMode::Boolean"));

    let test = bounded(
        SOURCE,
        "    pub(crate) fn emit_regexp_prototype_test_builtin(",
        "    fn emit_regexp_prototype_exec_from_locals(",
    );
    assert_eq!(test.matches("RegExpExecResultMode::Boolean").count(), 1);
    assert!(!test.contains("RegExpExecResultMode::MatchArrayOrNull"));

    assert_eq!(
        SOURCE
            .matches("self.emit_regexp_prototype_exec_from_locals(")
            .count(),
        3
    );
}
