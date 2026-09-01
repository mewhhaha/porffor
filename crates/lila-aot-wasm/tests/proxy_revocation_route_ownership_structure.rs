use std::fs;
use std::path::Path;

const SOURCE: &str = include_str!("../src/objects.rs");
const OBJECT_DESCRIPTOR_BUILTIN_SOURCE: &str =
    include_str!("../src/builtins/object/get_own_property_descriptor.rs");
const REFLECT_BUILTIN_SOURCE: &str = include_str!("../src/builtins/reflect.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/proxy-revocation-route-ownership.md");
const TASK: &str = include_str!("../../../tasks/10-object-model-descriptors-exotics.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
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

fn rust_code(source: &str, retain_literals: bool) -> String {
    let bytes = source.as_bytes();
    let mut code = String::new();
    let mut offset = 0;
    while offset < bytes.len() {
        if let Some(end) = literal_end(source, offset) {
            if retain_literals {
                code.push_str(&source[offset..end]);
            } else {
                code.push(' ');
            }
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
        if character.is_whitespace() {
            if !retain_literals {
                code.push(' ');
            }
        } else {
            code.push(character);
        }
        offset += character.len_utf8();
    }
    code
}

fn normalized_rust(source: &str) -> String {
    rust_code(source, true)
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
            exact_identifier_count(&rust_code(&source, false), identifier)
        })
        .sum()
}

#[test]
fn revocation_route_is_the_exact_crate_private_no_capability_authority() {
    let lexical_probe = rust_code(
        r###"
        // ProxyRevocationRoute::CurrentFunctionRealm
        ProxyRevocationRoute /* nested /* ignored */ comment */ :: r#ActiveHandler;
        "ProxyRevocationRoute"; b"ProxyRevocationRoute"; c"ProxyRevocationRoute";
        r"ProxyRevocationRoute"; br##"ProxyRevocationRoute"##;
        cr#"ProxyRevocationRoute"#; 'P'; b'P'; 'lifetime;
        "###,
        false,
    );
    assert_eq!(
        exact_identifier_count(&lexical_probe, "ProxyRevocationRoute"),
        1
    );

    let declaration_region = normalized_rust(bounded(
        SOURCE,
        "impl ProxyHandlerLocals {",
        "/// Declares the complete runtime order for object internal-method dispatch.",
    ));
    assert_eq!(
        declaration_region,
        concat!(
            "pub(crate)constfnnew(payload:u32,tag:u32)->Self{",
            "Self(TaggedLocals::new(payload,tag))}}",
            "pub(crate)enumProxyRevocationRoute{CurrentFunctionRealm,ActiveHandler,",
            "ObjectMutationRealmToActiveHandler,CurrentCompletion,}"
        )
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_identifier_in_rust_sources(&source_root, "ProxyRevocationRoute"),
        18
    );
    for forbidden in [
        "impl Clone for ProxyRevocationRoute",
        "impl Copy for ProxyRevocationRoute",
        "impl Debug for ProxyRevocationRoute",
        "impl PartialEq for ProxyRevocationRoute",
        "impl Eq for ProxyRevocationRoute",
        "ProxyRevocationRoute::clone",
    ] {
        assert!(!SOURCE.contains(forbidden), "found `{forbidden}`");
    }
}

#[test]
fn live_proxy_slot_reader_consumes_one_exact_route() {
    let consumer = normalized_rust(bounded(
        SOURCE,
        "pub(crate) fn emit_load_live_proxy_slots(",
        "/// Acquire and call one Proxy `[[DefineOwnProperty]]` trap.",
    ));
    assert_eq!(
        consumer,
        concat!(
            "&mutself,proxy_local:u32,slots:ProxySlotLocals,",
            "revocation_route:ProxyRevocationRoute,function:&mutFunction,",
            ")->Result<(),EmitError>{",
            "self.load_i64_to_local_from_offset(proxy_local,",
            "HEAP_OBJECT_BOXED_KIND_OFFSET,slots.handler.0.payload,function,);",
            "function.instruction(&Instruction::LocalGet(slots.handler.0.payload));",
            "function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MINasi64));",
            "function.instruction(&Instruction::I64Eq);",
            "function.instruction(&Instruction::If(BlockType::Empty));",
            "matchrevocation_route{",
            "ProxyRevocationRoute::CurrentFunctionRealm=>{",
            "self.emit_throw_current_function_realm_type_error(",
            "\"Proxy handler is null\",self.result_local,self.result_tag_local,function,)?;",
            "self.emit_return_current_completion(function);}",
            "ProxyRevocationRoute::ActiveHandler=>{",
            "self.emit_throw_runtime_error_to_active_handler(TYPE_ERROR_NAME,",
            "\"Proxy handler is null\",self.result_local,self.result_tag_local,function,)?;}",
            "ProxyRevocationRoute::ObjectMutationRealmToActiveHandler=>{",
            "self.emit_object_mutation_type_error_to_active_handler(",
            "\"Proxy handler is null\",function,)?;}",
            "ProxyRevocationRoute::CurrentCompletion=>{",
            "self.emit_throw_runtime_error(TYPE_ERROR_NAME,\"Proxy handler is null\",",
            "self.result_local,self.result_tag_local,function,)?;",
            "self.emit_return_current_completion(function);}}",
            "function.instruction(&Instruction::End);",
            "self.load_i64_to_local_from_offset(proxy_local,HEAP_PROXY_HANDLER_TAG_OFFSET,",
            "slots.handler.0.tag,function,);",
            "self.load_i64_to_local_from_offset(proxy_local,HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,",
            "slots.target.0.payload,function,);",
            "self.load_i64_to_local_from_offset(proxy_local,HEAP_OBJECT_BOXED_TAG_OFFSET,",
            "slots.target.0.tag,function,);Ok(())}"
        )
    );
    assert!(!consumer.contains("_=>"));
}

#[test]
fn ten_proxy_operations_select_their_exact_revocation_routes() {
    let producers = [
        (
            "pub(crate) fn emit_proxy_define_property_trap_result(",
            "pub(crate) fn emit_object_boxed_kind_for_tag(",
            "self.emit_load_live_proxy_slots(object.payload,slots,ProxyRevocationRoute::CurrentFunctionRealm,function,)?;",
        ),
        (
            "pub(crate) fn emit_proxy_own_keys_trap_result(",
            "fn emit_proxy_own_keys_validated_snapshot(",
            "self.emit_load_live_proxy_slots(object_payload_local,slots,ProxyRevocationRoute::CurrentFunctionRealm,function,)?;",
        ),
        (
            "pub(crate) fn emit_object_delete(",
            "pub(crate) fn emit_delete_ordinary_by_tag(",
            concat!(
                "self.emit_load_live_proxy_slots(current_payload_local,ProxySlotLocals::new(",
                "ProxyTargetLocals::new(target_payload_local,target_tag_local),",
                "ProxyHandlerLocals::new(handler_payload_local,handler_tag_local),),",
                "ProxyRevocationRoute::CurrentCompletion,function,)?;"
            ),
        ),
        (
            "pub(crate) fn emit_object_get_prototype_of(",
            "pub(crate) fn emit_ordinary_get_prototype_of(",
            concat!(
                "self.emit_load_live_proxy_slots(object_payload_local,ProxySlotLocals::new(",
                "ProxyTargetLocals::new(target_payload_local,target_tag_local),",
                "ProxyHandlerLocals::new(handler_payload_local,handler_tag_local),),",
                "ProxyRevocationRoute::ActiveHandler,function,)?;"
            ),
        ),
        (
            "pub(crate) fn emit_object_set_prototype_of_i32(",
            "pub(crate) fn emit_ordinary_set_prototype_of_i32(",
            concat!(
                "self.emit_load_live_proxy_slots(object_payload_local,ProxySlotLocals::new(",
                "ProxyTargetLocals::new(target_payload_local,target_tag_local),",
                "ProxyHandlerLocals::new(handler_payload_local,handler_tag_local),),",
                "ProxyRevocationRoute::ObjectMutationRealmToActiveHandler,function,)?;"
            ),
        ),
        (
            "pub(crate) fn emit_object_prevent_extensions(",
            "pub(crate) fn emit_object_is_extensible_i32(",
            concat!(
                "self.emit_load_live_proxy_slots(object.payload,ProxySlotLocals::new(",
                "ProxyTargetLocals::new(target_payload_local,target_tag_local),",
                "ProxyHandlerLocals::new(handler_payload_local,handler_tag_local),),",
                "ProxyRevocationRoute::ActiveHandler,function,)?;"
            ),
        ),
        (
            "pub(crate) fn emit_object_is_extensible_i32(",
            "fn emit_typed_array_canonical_numeric_index_i32(",
            concat!(
                "self.emit_load_live_proxy_slots(object_payload_local,ProxySlotLocals::new(",
                "ProxyTargetLocals::new(target_payload_local,target_tag_local),",
                "ProxyHandlerLocals::new(handler_payload_local,handler_tag_local),),",
                "ProxyRevocationRoute::ActiveHandler,function,)?;"
            ),
        ),
        (
            "fn emit_has_property_dispatch_with_key_tag_i32(",
            "pub(crate) fn emit_data_property_read_no_call(",
            concat!(
                "self.emit_load_live_proxy_slots(current_local,ProxySlotLocals::new(",
                "ProxyTargetLocals::new(target_payload_local,target_tag_local),",
                "ProxyHandlerLocals::new(boxed_kind_local,handler_tag_local),),",
                "ProxyRevocationRoute::CurrentCompletion,function,)?;"
            ),
        ),
    ];
    for (start, end, expected_call) in producers {
        let owner = normalized_rust(bounded(SOURCE, start, end));
        assert_eq!(owner.matches("self.emit_load_live_proxy_slots(").count(), 1);
        assert_eq!(owner.matches(expected_call).count(), 1, "owner `{start}`");
    }

    let object_builtin = normalized_rust(bounded(
        OBJECT_DESCRIPTOR_BUILTIN_SOURCE,
        "pub(in crate::builtins) fn compile_object_get_own_property_descriptor_builtin(",
        "\n}",
    ));
    assert_eq!(
        object_builtin
            .matches("self.emit_load_live_proxy_slots(")
            .count(),
        1
    );
    assert_eq!(
        object_builtin
            .matches(concat!(
                "self.emit_load_live_proxy_slots(target_payload_local,ProxySlotLocals::new(",
                "ProxyTargetLocals::new(value_payload_local,value_tag_local),",
                "ProxyHandlerLocals::new(proxy_handler_payload_local,proxy_handler_tag_local),),",
                "ProxyRevocationRoute::CurrentFunctionRealm,function,)?;"
            ))
            .count(),
        1
    );

    let reflect_set_builtin = normalized_rust(bounded(
        REFLECT_BUILTIN_SOURCE,
        "pub(crate) fn compile_reflect_set_builtin(",
        "pub(crate) fn compile_reflect_has_builtin(",
    ));
    assert_eq!(
        reflect_set_builtin
            .matches("self.emit_load_live_proxy_slots(")
            .count(),
        1
    );
    assert_eq!(
        reflect_set_builtin
            .matches(concat!(
                "self.emit_load_live_proxy_slots(target_payload_local,ProxySlotLocals::new(",
                "ProxyTargetLocals::new(proxy_target_payload_local,proxy_target_tag_local),",
                "ProxyHandlerLocals::new(handler_payload_local,handler_tag_local),),",
                "ProxyRevocationRoute::CurrentFunctionRealm,function,)?;"
            ))
            .count(),
        1
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let objects_code = rust_code(SOURCE, false);
    let builtin_code = rust_code(OBJECT_DESCRIPTOR_BUILTIN_SOURCE, false);
    let reflect_code = rust_code(REFLECT_BUILTIN_SOURCE, false);
    assert_eq!(
        objects_code.matches(".emit_load_live_proxy_slots").count(),
        8
    );
    assert_eq!(
        builtin_code.matches(".emit_load_live_proxy_slots").count(),
        1
    );
    assert_eq!(
        reflect_code.matches(".emit_load_live_proxy_slots").count(),
        1
    );
    assert_eq!(
        count_identifier_in_rust_sources(&source_root, "emit_load_live_proxy_slots"),
        11
    );
    assert_eq!(
        objects_code.matches("::emit_load_live_proxy_slots").count(),
        0
    );
    assert_eq!(
        builtin_code.matches("::emit_load_live_proxy_slots").count(),
        0
    );
    assert_eq!(
        reflect_code.matches("::emit_load_live_proxy_slots").count(),
        0
    );
    assert_eq!(
        SOURCE
            .matches("ProxyRevocationRoute::CurrentFunctionRealm")
            .count(),
        3
    );
    assert_eq!(
        OBJECT_DESCRIPTOR_BUILTIN_SOURCE
            .matches("ProxyRevocationRoute::CurrentFunctionRealm")
            .count(),
        1
    );
    assert_eq!(
        REFLECT_BUILTIN_SOURCE
            .matches("ProxyRevocationRoute::CurrentFunctionRealm")
            .count(),
        1
    );
    assert_eq!(
        SOURCE
            .matches("ProxyRevocationRoute::ActiveHandler")
            .count(),
        4
    );
    assert_eq!(
        SOURCE
            .matches("ProxyRevocationRoute::ObjectMutationRealmToActiveHandler")
            .count(),
        2
    );
    assert_eq!(
        SOURCE
            .matches("ProxyRevocationRoute::CurrentCompletion")
            .count(),
        3
    );
}

#[test]
fn contract_and_task_record_the_bounded_ownership_law() {
    for phrase in [
        "ten exact producers",
        "one consuming exhaustive router",
        "CurrentFunctionRealm",
        "ActiveHandler",
        "ObjectMutationRealmToActiveHandler",
        "CurrentCompletion",
        "Realm-correcting",
    ] {
        assert!(CONTRACT.contains(phrase), "contract missing `{phrase}`");
    }
    assert!(CONTRACT
        .contains("cargo test -p lila-aot-wasm --test proxy_revocation_route_ownership_structure"));
    assert!(TASK.contains("`ProxyRevocationRoute` is now a crate-private, capability-free"));
}
