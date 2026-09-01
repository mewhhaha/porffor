use std::fs;
use std::path::Path;

const OBJECTS_SOURCE: &str = include_str!("../src/objects.rs");
const OBJECT_BUILTINS_SOURCE: &str = include_str!("../src/builtins/object.rs");
const REFLECT_BUILTINS_SOURCE: &str = include_str!("../src/builtins/reflect.rs");
const CLI_OBJECT_SOURCE: &str = include_str!("../../lila-cli/tests/cli/object.rs");
const OWN_KEYS_FIXTURE: &str = include_str!("../../lila-cli/tests/fixtures/wasm_proxy_own_keys.js");
const HANDLER_PROTOCOL_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_proxy_own_keys_handler_protocol.js");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/proxy-own-keys-result-ownership.md");
const TASK: &str = include_str!("../../../tasks/11-proxy-reflect-metaobject.md");

fn quoted_literal_end(source: &str, quote_start: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut offset = quote_start + 1;
    let mut escaped = false;
    while offset < bytes.len() {
        let byte = bytes[offset];
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == b'"' {
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
        b'"' => quoted_literal_end(source, start),
        b'\'' => character_literal_end(source, start),
        b'b' if bytes.get(start + 1) == Some(&b'\'') => character_literal_end(source, start + 1),
        b'b' | b'c' if bytes.get(start + 1) == Some(&b'"') => quoted_literal_end(source, start + 1),
        b'r' => raw_literal_end(source, start, 1),
        b'b' | b'c' if bytes.get(start + 1) == Some(&b'r') => raw_literal_end(source, start, 2),
        _ => None,
    }
}

fn rust_identifiers(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut identifiers = String::new();
    let mut offset = 0;
    while offset < bytes.len() {
        if let Some(end) = literal_end(source, offset) {
            identifiers.push(' ');
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
            assert_eq!(depth, 0, "unterminated block comment");
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
        identifiers.push(if character.is_whitespace() {
            ' '
        } else {
            character
        });
        offset += character.len_utf8();
    }
    identifiers
}

fn exact_identifier_count(source: &str, identifier: &str) -> usize {
    source
        .match_indices(identifier)
        .filter(|(offset, _)| {
            let before = source[..*offset].chars().next_back();
            let after = source[*offset + identifier.len()..].chars().next();
            [before, after].into_iter().all(|edge| {
                edge.map(|character| !character.is_alphanumeric() && character != '_')
                    .unwrap_or(true)
            })
        })
        .count()
}

fn count_identifier_in_rust_sources(dir: &Path, identifier: &str) -> usize {
    fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .map(|path| {
            if path.is_dir() {
                return count_identifier_in_rust_sources(&path, identifier);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return 0;
            }
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            exact_identifier_count(&rust_identifiers(&source), identifier)
        })
        .sum()
}

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}`"))
        .0
}

fn after<'a>(source: &'a str, start: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
}

fn assert_before(source: &str, earlier: &str, later: &str) {
    let earlier_offset = source.find(earlier).expect("earlier operation");
    let later_offset = source.find(later).expect("later operation");
    assert!(
        earlier_offset < later_offset,
        "`{earlier}` must precede `{later}`"
    );
}

fn mask_line_and_block_comments(source: &str) -> String {
    let mut characters = source.chars().peekable();
    let mut masked = String::with_capacity(source.len());
    let mut quote = None;
    let mut escaped = false;
    let mut line_comment = false;
    let mut block_comment_depth = 0usize;

    while let Some(character) = characters.next() {
        if line_comment {
            if character == '\n' {
                masked.push(character);
                line_comment = false;
            } else {
                masked.push(' ');
            }
            continue;
        }

        if block_comment_depth > 0 {
            if character == '/' && characters.peek() == Some(&'*') {
                characters.next();
                masked.push_str("  ");
                block_comment_depth += 1;
                continue;
            }
            if character == '*' && characters.peek() == Some(&'/') {
                characters.next();
                masked.push_str("  ");
                block_comment_depth -= 1;
                continue;
            }
            masked.push(if character == '\n' { '\n' } else { ' ' });
            continue;
        }

        if let Some(delimiter) = quote {
            masked.push(character);
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == delimiter {
                quote = None;
            }
            continue;
        }

        if matches!(character, '\'' | '"' | '`') {
            quote = Some(character);
            masked.push(character);
            continue;
        }
        if character == '/' && characters.peek() == Some(&'/') {
            characters.next();
            masked.push_str("  ");
            line_comment = true;
            continue;
        }
        if character == '/' && characters.peek() == Some(&'*') {
            characters.next();
            masked.push_str("  ");
            block_comment_depth = 1;
            continue;
        }
        masked.push(character);
    }

    assert_eq!(block_comment_depth, 0, "unterminated block comment");
    masked
}

fn anchored_offsets(source: &str, declaration: &str) -> Vec<usize> {
    source
        .match_indices(declaration)
        .filter_map(|(offset, _)| {
            let line_start = source[..offset]
                .rfind('\n')
                .map_or(0, |newline| newline + 1);
            source[line_start..offset]
                .chars()
                .all(char::is_whitespace)
                .then_some(offset)
        })
        .collect()
}

fn braced_rust_function<'a>(source: &'a str, declaration: &str) -> &'a str {
    let offsets = anchored_offsets(source, declaration);
    assert_eq!(offsets.len(), 1, "exact Rust owner `{declaration}`");
    let start = offsets[0];
    let mut depth = 0;
    let mut body_started = false;
    for (relative_offset, character) in source[start..].char_indices() {
        match character {
            '{' => {
                depth += 1;
                body_started = true;
            }
            '}' => {
                depth -= 1;
                if body_started && depth == 0 {
                    return &source[start..start + relative_offset + character.len_utf8()];
                }
            }
            _ => {}
        }
    }
    panic!("unterminated Rust owner `{declaration}`");
}

fn assert_live_wasm_cli_test(source: &str, name: &str, fixture: &str) {
    let declaration = format!("fn {name}() {{");
    let offsets = anchored_offsets(source, &declaration);
    assert_eq!(offsets.len(), 1, "exact CLI test owner `{name}`");

    let attached_attributes = source[..offsets[0]]
        .rsplit_once("\n}\n")
        .unwrap_or_else(|| panic!("preceding top-level CLI owner for `{name}`"))
        .1;
    assert_eq!(
        attached_attributes.matches("#[test]").count(),
        1,
        "`{name}` must remain a live Rust test"
    );
    for disabling_attribute in ["#[cfg", "#[cfg_attr", "#[ignore"] {
        assert!(
            !attached_attributes.contains(disabling_attribute),
            "`{name}` must not carry `{disabling_attribute}`"
        );
    }

    let body = braced_rust_function(source, &declaration);
    for marker in [
        ".arg(\"run\")",
        ".arg(\"--execution-backend\")",
        ".arg(\"wasm\")",
        "assert!(output.status.success());",
        "assert!(stdout.contains(\"backend_used: WasmAot\"));",
        "assert!(stdout.contains(\"boolean(true)\"));",
    ] {
        assert_eq!(
            body.lines().filter(|line| line.trim() == marker).count(),
            1,
            "`{name}` must retain CLI marker `{marker}`"
        );
    }
    assert_eq!(
        body.matches(&format!("fixture_path(\"{fixture}\")"))
            .count(),
        1,
        "`{name}` must run its exact fixture"
    );
}

fn own_keys_acquisition() -> &'static str {
    bounded(
        OBJECTS_SOURCE,
        "pub(crate) fn emit_proxy_own_keys_trap_result(",
        "fn emit_proxy_own_keys_validated_snapshot(",
    )
}

fn assert_typed_caller(
    caller: &str,
    object: &str,
    target: &str,
    handler: &str,
    result_binding: &str,
    validator: &str,
    validator_handler_count: usize,
) {
    assert_eq!(
        caller
            .matches("self.emit_proxy_own_keys_trap_result(")
            .count(),
        1
    );
    assert_eq!(caller.matches("ProxySlotLocals::new(").count(), 1);
    assert_eq!(caller.matches("ProxyTargetLocals::new(").count(), 2);
    assert_eq!(
        caller.matches("ProxyHandlerLocals::new(").count(),
        validator_handler_count + 1
    );
    assert_eq!(caller.matches("ProxyOwnKeysTrapLocals::new(").count(), 1);
    assert_eq!(
        caller.matches("ProxyOwnKeysTrapResultLocals::new(").count(),
        1
    );
    assert_eq!(caller.matches("TaggedLocals::new(").count(), 1);
    assert!(caller.contains(object));
    assert!(caller.contains(target));
    assert!(caller.contains(handler));
    assert_eq!(caller.matches(validator).count(), 1);
    assert_eq!(
        caller
            .matches(&format!(
                "let {result_binding} = self.emit_proxy_own_keys_trap_result("
            ))
            .count(),
        1
    );
    assert_eq!(
        caller
            .lines()
            .filter(|line| line.trim() == format!("{result_binding},"))
            .count(),
        1
    );
    assert_before(
        caller,
        "ProxyTargetLocals::new(",
        "ProxyHandlerLocals::new(",
    );
    assert_before(caller, "self.emit_proxy_own_keys_trap_result(", validator);
    for retired_inline_acquisition in [
        "self.strings.payload(\"ownKeys\")",
        "Proxy ownKeys trap is not callable",
    ] {
        assert!(
            !caller.contains(retired_inline_acquisition),
            "caller must not retain the raw ownKeys acquisition `{retired_inline_acquisition}`",
        );
    }
}

#[test]
fn own_keys_trap_roles_are_distinct_non_copy_and_closed_over_product_sources() {
    let lexical_probe = rust_identifiers(
        r###"
        // ProxyOwnKeysTrapLocals
        ProxyOwnKeysTrapLocals /* nested /* ignored */ comment */;
        "ProxyOwnKeysTrapLocals"; b"ProxyOwnKeysTrapLocals";
        c"ProxyOwnKeysTrapLocals"; r"ProxyOwnKeysTrapLocals";
        br##"ProxyOwnKeysTrapLocals"##; cr#"ProxyOwnKeysTrapLocals"#;
        'P'; b'P'; 'lifetime; r#ProxyOwnKeysTrapLocals;
        "###,
    );
    assert_eq!(
        exact_identifier_count(&lexical_probe, "ProxyOwnKeysTrapLocals"),
        2
    );

    let roles = bounded(
        OBJECTS_SOURCE,
        "/// The prospective Proxy `[[OwnPropertyKeys]]` trap method.",
        "/// A Proxy `[[Get]]` trap result whose completion has not yet been consumed.",
    );
    for (role, must_use) in [
        (
            "ProxyOwnKeysTrapLocals",
            "#[must_use = \"Proxy OwnPropertyKeys trap locals must be consumed by trap acquisition\"]",
        ),
        (
            "ProxyOwnKeysTrapResultLocals",
            "#[must_use = \"a Proxy OwnPropertyKeys trap result must be consumed by one validator\"]",
        ),
    ] {
        assert!(roles.contains(must_use));
        assert!(roles.contains(&format!("pub(crate) struct {role}(TaggedLocals);")));
    }
    for forbidden in ["#[derive", "impl Clone for", "impl Copy for"] {
        assert!(!roles.contains(forbidden), "found `{forbidden}`");
    }

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_identifier_in_rust_sources(&source_root, "ProxyOwnKeysTrapLocals"),
        9
    );
    assert_eq!(
        count_identifier_in_rust_sources(&source_root, "ProxyOwnKeysTrapResultLocals"),
        14
    );
}

#[test]
fn contract_and_t11_own_the_result_authority() {
    for marker in [
        "ProxyOwnKeysTrapLocals",
        "ProxyOwnKeysTrapResultLocals",
        "proxy_own_keys_handler_protocol_structure",
    ] {
        assert!(CONTRACT.contains(marker), "contract marker `{marker}`");
        assert!(TASK.contains(marker), "task marker `{marker}`");
    }
    assert!(CONTRACT.contains("transposing them compiled"));
}

#[test]
fn acquisition_has_one_typed_live_slot_read() {
    let acquisition = own_keys_acquisition();

    for role in [
        "object: TaggedLocals,",
        "slots: ProxySlotLocals,",
        "trap: ProxyOwnKeysTrapLocals,",
        "trap_result: ProxyOwnKeysTrapResultLocals,",
    ] {
        assert_eq!(acquisition.matches(role).count(), 1, "typed role `{role}`");
    }
    for mapping in [
        "let object_payload_local = object.payload;",
        "let object_tag_local = object.tag;",
        "let target_payload_local = slots.target.0.payload;",
        "let target_tag_local = slots.target.0.tag;",
        "let handler_payload_local = slots.handler.0.payload;",
        "let handler_tag_local = slots.handler.0.tag;",
        "let trap_payload_local = trap.0.payload;",
        "let trap_tag_local = trap.0.tag;",
        "let trap_result_payload_local = trap_result.0.payload;",
        "let trap_result_tag_local = trap_result.0.tag;",
        "Ok(trap_result)",
    ] {
        assert_eq!(
            acquisition.matches(mapping).count(),
            1,
            "mapping `{mapping}`"
        );
    }

    assert_eq!(
        acquisition
            .matches("self.emit_load_live_proxy_slots(")
            .count(),
        1
    );
    assert_eq!(
        acquisition
            .matches("ProxyRevocationRoute::CurrentFunctionRealm,")
            .count(),
        1
    );
    assert_eq!(
        acquisition
            .matches("HEAP_OBJECT_BOXED_KIND_OFFSET,")
            .count(),
        1,
        "the one direct heap read is classification only"
    );
    for forbidden in [
        "HEAP_PROXY_HANDLER_TAG_OFFSET",
        "HEAP_OBJECT_BOXED_PAYLOAD_OFFSET",
        "HEAP_OBJECT_BOXED_TAG_OFFSET",
        "Instruction::LocalSet(handler_tag_local)",
        "ValueKind::Object.tag() as i64));\n        function.instruction(&Instruction::LocalSet(handler_tag_local)",
    ] {
        assert!(
            !acquisition.contains(forbidden),
            "live Proxy slot `{forbidden}` must not be reconstructed here"
        );
    }
}

#[test]
fn get_method_completion_and_proxy_aware_call_keep_exact_handler_tags() {
    let acquisition = own_keys_acquisition();

    assert_eq!(
        acquisition
            .matches("self.emit_object_read_without_throw_propagation(")
            .count(),
        1
    );
    assert_eq!(
        acquisition
            .matches("self.emit_return_current_completion_if_throw(function);")
            .count(),
        1
    );
    assert_eq!(acquisition.matches("self.emit_is_callable_i32(").count(), 1);
    assert_eq!(
        acquisition
            .matches("self.emit_function_or_proxy_call_with_throw_propagation(")
            .count(),
        1
    );
    assert_eq!(
        acquisition
            .matches("self.emit_throw_current_function_realm_type_error(")
            .count(),
        1
    );

    assert!(acquisition.contains(
        "self.emit_object_read_without_throw_propagation(\n            handler_payload_local,\n            handler_tag_local,\n            handler_payload_local,\n            handler_tag_local,\n            key_payload_local,\n            trap_payload_local,\n            trap_tag_local,"
    ));
    assert!(acquisition.contains(
        "self.emit_function_or_proxy_call_with_throw_propagation(\n            trap_payload_local,\n            trap_tag_local,\n            handler_payload_local,\n            handler_tag_local,\n            &[(target_payload_local, target_tag_local)],"
    ));

    assert_before(
        acquisition,
        "self.emit_load_live_proxy_slots(",
        "self.emit_object_read_without_throw_propagation(",
    );
    assert_before(
        acquisition,
        "self.emit_object_read_without_throw_propagation(",
        "self.emit_return_current_completion_if_throw(function);",
    );
    assert_before(
        acquisition,
        "self.emit_return_current_completion_if_throw(function);",
        "self.emit_is_callable_i32(",
    );
    assert_before(
        acquisition,
        "self.emit_is_callable_i32(",
        "self.emit_function_or_proxy_call_with_throw_propagation(",
    );

    for forbidden in [
        "self.emit_object_read(",
        "self.emit_function_handle_call",
        "self.emit_throw_runtime_error(",
        "ValueKind::Function.tag()",
    ] {
        assert!(
            !acquisition.contains(forbidden),
            "raw operation `{forbidden}` bypasses the handler protocol"
        );
    }
}

#[test]
fn nullish_fallback_retains_the_tagged_target() {
    let acquisition = own_keys_acquisition();

    assert!(acquisition.contains(
        "Instruction::LocalGet(target_payload_local));\n        function.instruction(&Instruction::LocalSet(object_payload_local));\n        function.instruction(&Instruction::LocalGet(target_tag_local));\n        function.instruction(&Instruction::LocalSet(object_tag_local));\n        function.instruction(&Instruction::Br(2));"
    ));
    assert_before(
        acquisition,
        "ValueKind::Undefined.tag()",
        "Instruction::LocalGet(target_payload_local)",
    );
    assert_before(
        acquisition,
        "Instruction::LocalGet(target_payload_local)",
        "self.emit_throw_current_function_realm_type_error(",
    );
}

#[test]
fn all_four_consumers_use_the_typed_acquisition_and_keep_validation() {
    assert_eq!(
        OBJECT_BUILTINS_SOURCE
            .matches("self.emit_proxy_own_keys_trap_result(")
            .count(),
        3
    );
    assert_eq!(
        REFLECT_BUILTINS_SOURCE
            .matches("self.emit_proxy_own_keys_trap_result(")
            .count(),
        1
    );

    let names = bounded(
        OBJECT_BUILTINS_SOURCE,
        "pub(super) fn compile_object_get_own_property_names_builtin(",
        "pub(super) fn compile_object_get_own_property_symbols_builtin(",
    );
    assert_typed_caller(
        names,
        "TaggedLocals::new(arg_payload_local, arg_tag_local)",
        "ProxyTargetLocals::new(proxy_target_payload_local, proxy_target_tag_local)",
        "ProxyHandlerLocals::new(proxy_handler_payload_local, proxy_handler_tag_local)",
        "proxy_trap_result",
        "self.emit_proxy_own_keys_filtered_result(",
        0,
    );

    let symbols = bounded(
        OBJECT_BUILTINS_SOURCE,
        "pub(super) fn compile_object_get_own_property_symbols_builtin(",
        "pub(super) fn compile_object_keys_builtin(",
    );
    assert_typed_caller(
        symbols,
        "TaggedLocals::new(arg_payload_local, arg_tag_local)",
        "ProxyTargetLocals::new(proxy_target_payload_local, proxy_target_tag_local)",
        "ProxyHandlerLocals::new(proxy_handler_payload_local, proxy_handler_tag_local)",
        "proxy_trap_result",
        "self.emit_proxy_own_keys_filtered_result(",
        0,
    );

    let keys = bounded(
        OBJECT_BUILTINS_SOURCE,
        "pub(super) fn compile_object_keys_builtin(",
        "fn compile_object_own_descriptor_predicate_builtin(",
    );
    assert_typed_caller(
        keys,
        "TaggedLocals::new(arg_payload_local, arg_tag_local)",
        "ProxyTargetLocals::new(proxy_target_payload_local, proxy_target_tag_local)",
        "ProxyHandlerLocals::new(proxy_handler_payload_local, proxy_handler_tag_local)",
        "proxy_trap_result",
        "self.emit_proxy_object_keys_from_own_keys_result(",
        1,
    );

    let reflect = after(
        REFLECT_BUILTINS_SOURCE,
        "pub(crate) fn compile_reflect_own_keys_builtin(",
    );
    assert_typed_caller(
        reflect,
        "TaggedLocals::new(target_payload_local, target_tag_local)",
        "ProxyTargetLocals::new(proxy_target_payload_local, proxy_target_tag_local)",
        "ProxyHandlerLocals::new(handler_payload_local, handler_tag_local)",
        "trap_result",
        "self.emit_proxy_own_keys_array_result(",
        0,
    );
}

#[test]
fn cli_regressions_are_live_and_cover_handler_protocol_boundaries() {
    const PROTOCOL_TEST_NAME: &str =
        "run_wasm_backend_succeeds_for_proxy_own_keys_handler_protocol";
    let protocol_declaration = format!("fn {PROTOCOL_TEST_NAME}() {{");
    for commented_owner in [
        format!("// #[test]\n// {protocol_declaration}\n// }}"),
        format!("/*\n#[test]\n{protocol_declaration}\n}}\n*/"),
    ] {
        let active_source = mask_line_and_block_comments(&commented_owner);
        assert!(
            anchored_offsets(&active_source, &protocol_declaration).is_empty(),
            "commented CLI owner must not count as active"
        );
    }
    for disabling_attribute in [
        "#[cfg(\n    any()\n)]",
        "#[cfg_attr(\n    all(),\n    ignore\n)]",
        "#[ignore\n]",
    ] {
        let disabled_registration = format!(
            "fn preceding_owner() {{\n}}\n{disabling_attribute}\n#[test]\n{protocol_declaration}\n}}"
        );
        let active_source = mask_line_and_block_comments(&disabled_registration);
        assert!(
            std::panic::catch_unwind(|| {
                assert_live_wasm_cli_test(
                    &active_source,
                    PROTOCOL_TEST_NAME,
                    "wasm_proxy_own_keys_handler_protocol.js",
                );
            })
            .is_err(),
            "multi-line `{disabling_attribute}` must disable the CLI owner"
        );
    }

    const EXECUTABLE_MARKER: &str = "check(functionTrapThis === functionHandler);";
    for commented_marker in [
        format!("// {EXECUTABLE_MARKER}"),
        format!("/* {EXECUTABLE_MARKER} */"),
    ] {
        assert!(
            !mask_line_and_block_comments(&commented_marker).contains(EXECUTABLE_MARKER),
            "commented fixture assertion must not count as executable"
        );
    }

    let cli_object_source = mask_line_and_block_comments(CLI_OBJECT_SOURCE);
    let own_keys_fixture = mask_line_and_block_comments(OWN_KEYS_FIXTURE);
    let handler_protocol_fixture = mask_line_and_block_comments(HANDLER_PROTOCOL_FIXTURE);

    assert_live_wasm_cli_test(
        &cli_object_source,
        "run_wasm_backend_succeeds_for_supported_proxy_own_keys_fixture",
        "wasm_proxy_own_keys.js",
    );
    assert_live_wasm_cli_test(
        &cli_object_source,
        PROTOCOL_TEST_NAME,
        "wasm_proxy_own_keys_handler_protocol.js",
    );

    assert!(own_keys_fixture.contains(
        "function throwsTypeError(fn) {\n  try {\n    fn();\n  } catch (error) {\n    return error instanceof TypeError;\n  }\n  return false;\n}"
    ));
    assert!(own_keys_fixture.matches("failures |=").count() >= 20);
    for load_bearing_scenario in [
        "if (trapTarget !== target) failures |= 16;",
        "if (!throwsTypeError(function() { Object.keys(duplicateProxy); })) failures |= 32;",
        "if (!throwsTypeError(function() { Object.keys(invalidEntryProxy); })) failures |= 64;",
        "if (!throwsTypeError(function() { Object.keys(invalidResultProxy); })) failures |= 128;",
        "var nestedProxy = new Proxy(nestedTarget, {\n  ownKeys: null\n});",
        "var reflectProxy = new Proxy(reflectTarget, {\n  ownKeys: undefined\n});",
        "failures === 0;",
    ] {
        assert!(
            own_keys_fixture.contains(load_bearing_scenario),
            "base ownKeys fixture must retain `{load_bearing_scenario}`"
        );
    }
    let base_nested_fallback = bounded(
        &own_keys_fixture,
        "var nestedTarget = new Proxy(",
        "var symbolKey = Symbol();",
    );
    assert_before(
        base_nested_fallback,
        "ownKeys: null",
        "Object.keys(nestedProxy)",
    );
    assert_before(
        base_nested_fallback,
        "Object.keys(nestedProxy)",
        "nestedKeys.length !== 2",
    );

    assert!(handler_protocol_fixture
        .contains("function check(condition) {\n  if (!condition) failures += 1;\n}"));
    assert!(handler_protocol_fixture.contains(
        "function capture(fn) {\n  try {\n    fn();\n  } catch (error) {\n    return error;\n  }\n  return undefined;\n}"
    ));
    assert!(handler_protocol_fixture.matches("check(").count() >= 30);
    assert!(handler_protocol_fixture
        .trim_end()
        .ends_with("failures === 0;"));

    let function_handler = bounded(
        &handler_protocol_fixture,
        "var functionTarget = {};",
        "var arrayTarget = { arrayKey: 2 };",
    );
    for marker in [
        "function functionHandler() {}",
        "var callableProxyTrap = new Proxy(functionTrapTargetFunction, {});",
        "return callableProxyTrap;",
        "Reflect.ownKeys(new Proxy(functionTarget, functionHandler))",
        "check(functionGetterThis === functionHandler);",
        "check(functionTrapThis === functionHandler);",
        "check(functionTrapTarget === functionTarget);",
        "check(functionKeys.length === 1 && functionKeys[0] === \"functionKey\");",
    ] {
        assert!(
            function_handler.contains(marker),
            "Function handler `{marker}`"
        );
    }
    assert_before(
        function_handler,
        "return callableProxyTrap;",
        "Reflect.ownKeys(new Proxy(functionTarget, functionHandler))",
    );
    assert_before(
        function_handler,
        "Reflect.ownKeys(new Proxy(functionTarget, functionHandler))",
        "check(functionTrapThis === functionHandler);",
    );

    let array_handler = bounded(
        &handler_protocol_fixture,
        "var arrayTarget = { arrayKey: 2 };",
        "function makeArgumentsHandler() {",
    );
    for marker in [
        "var arrayHandler = [];",
        "Object.getOwnPropertyNames(new Proxy(arrayTarget, arrayHandler))",
        "check(arrayGetterThis === arrayHandler);",
        "check(arrayTrapThis === arrayHandler);",
        "check(arrayTrapTarget === arrayTarget);",
        "check(arrayKeys.length === 1 && arrayKeys[0] === \"arrayKey\");",
    ] {
        assert!(array_handler.contains(marker), "Array handler `{marker}`");
    }
    assert_before(
        array_handler,
        "Object.defineProperty(arrayHandler, \"ownKeys\"",
        "Object.getOwnPropertyNames(new Proxy(arrayTarget, arrayHandler))",
    );
    assert_before(
        array_handler,
        "Object.getOwnPropertyNames(new Proxy(arrayTarget, arrayHandler))",
        "check(arrayGetterThis === arrayHandler);",
    );

    let arguments_handler = bounded(
        &handler_protocol_fixture,
        "function makeArgumentsHandler() {",
        "var symbolKey = Symbol(\"proxy-handler-key\");",
    );
    for marker in [
        "return arguments;",
        "var argumentsHandler = makeArgumentsHandler(1);",
        "Object.keys(new Proxy(argumentsTarget, argumentsHandler))",
        "check(argumentsGetterThis === argumentsHandler);",
        "check(argumentsTrapThis === argumentsHandler);",
        "check(argumentsTrapTarget === argumentsTarget);",
        "check(argumentsKeys.length === 1 && argumentsKeys[0] === \"argumentsKey\");",
    ] {
        assert!(
            arguments_handler.contains(marker),
            "arguments handler `{marker}`"
        );
    }
    assert_before(
        arguments_handler,
        "Object.defineProperty(argumentsHandler, \"ownKeys\"",
        "Object.keys(new Proxy(argumentsTarget, argumentsHandler))",
    );
    assert_before(
        arguments_handler,
        "Object.keys(new Proxy(argumentsTarget, argumentsHandler))",
        "check(argumentsGetterThis === argumentsHandler);",
    );

    let proxy_handler = bounded(
        &handler_protocol_fixture,
        "var symbolKey = Symbol(\"proxy-handler-key\");",
        "var lookupSentinel = {};",
    );
    for marker in [
        "proxyHandler = new Proxy(proxyHandlerTarget, proxyLookupHandler);",
        "Object.getOwnPropertySymbols(new Proxy(proxyTarget, proxyHandler))",
        "check(proxyLookupThis === proxyLookupHandler);",
        "check(proxyLookupTarget === proxyHandlerTarget);",
        "check(proxyLookupKey === \"ownKeys\");",
        "check(proxyLookupReceiver === proxyHandler);",
        "check(proxyTrapThis === proxyHandler);",
        "check(proxyTrapTarget === proxyTarget);",
        "check(symbolKeys.length === 1 && symbolKeys[0] === symbolKey);",
    ] {
        assert!(proxy_handler.contains(marker), "Proxy handler `{marker}`");
    }
    assert_before(
        proxy_handler,
        "proxyHandler = new Proxy(proxyHandlerTarget, proxyLookupHandler);",
        "Object.getOwnPropertySymbols(new Proxy(proxyTarget, proxyHandler))",
    );
    assert_before(
        proxy_handler,
        "Object.getOwnPropertySymbols(new Proxy(proxyTarget, proxyHandler))",
        "check(proxyLookupReceiver === proxyHandler);",
    );

    let abrupt_lookup = bounded(
        &handler_protocol_fixture,
        "var lookupSentinel = {};",
        "var nestedCalls = 0;",
    );
    assert!(abrupt_lookup.contains("throw lookupSentinel;"));
    assert!(abrupt_lookup.contains("Reflect.ownKeys(new Proxy({}, abruptHandler))"));
    assert!(abrupt_lookup.contains("check(lookupError === lookupSentinel);"));
    assert_before(
        abrupt_lookup,
        "throw lookupSentinel;",
        "var lookupError = capture(",
    );
    assert_before(
        abrupt_lookup,
        "var lookupError = capture(",
        "check(lookupError === lookupSentinel);",
    );

    let nested_fallback = bounded(
        &handler_protocol_fixture,
        "var nestedCalls = 0;",
        "var other = __lilaCreateRealm().global;",
    );
    assert!(nested_fallback.contains("ownKeys: function() {"));
    assert!(nested_fallback.contains("new Proxy(nestedTarget, { ownKeys: null })"));
    assert!(nested_fallback.contains("check(nestedCalls === 1);"));
    assert!(nested_fallback
        .contains("check(nestedKeys.length === 1 && nestedKeys[0] === \"nestedKey\");"));
    assert_before(
        nested_fallback,
        "new Proxy(nestedTarget, { ownKeys: null })",
        "check(nestedCalls === 1);",
    );
    assert_before(
        nested_fallback,
        "new Proxy(nestedTarget, { ownKeys: null })",
        "check(nestedKeys.length === 1 && nestedKeys[0] === \"nestedKey\");",
    );

    let foreign_realm_errors = after(
        &handler_protocol_fixture,
        "var other = __lilaCreateRealm().global;",
    );
    for marker in [
        "other.Reflect.ownKeys(new Proxy({}, { ownKeys: {} }))",
        "Object.getPrototypeOf(nonCallableError) === other.TypeError.prototype",
        "nonCallableError instanceof other.TypeError",
        "!(nonCallableError instanceof TypeError)",
        "revocable.revoke();",
        "other.Object.keys(revocable.proxy)",
        "Object.getPrototypeOf(revokedError) === other.TypeError.prototype",
        "revokedError instanceof other.TypeError",
        "!(revokedError instanceof TypeError)",
    ] {
        assert!(
            foreign_realm_errors.contains(marker),
            "foreign Realm `{marker}`"
        );
    }
    assert_before(
        foreign_realm_errors,
        "other.Reflect.ownKeys(new Proxy({}, { ownKeys: {} }))",
        "Object.getPrototypeOf(nonCallableError) === other.TypeError.prototype",
    );
    assert_before(
        foreign_realm_errors,
        "revocable.revoke();",
        "other.Object.keys(revocable.proxy)",
    );
    assert_before(
        foreign_realm_errors,
        "other.Object.keys(revocable.proxy)",
        "Object.getPrototypeOf(revokedError) === other.TypeError.prototype",
    );
}
