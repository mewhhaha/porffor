use std::fs;
use std::path::Path;

const REFLECT_SOURCE: &str = include_str!("../src/builtins/reflect.rs");
const DESCRIPTOR_OBJECT_PROTOTYPE_SOURCE: &str =
    include_str!("../src/builtins/reflect/descriptor_object_prototype.rs");
const HOST_SOURCE: &str = include_str!("../src/builtins/host.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/reflect-descriptor-object-realm.md");
const TASK: &str = include_str!("../../../tasks/06-realms-intrinsics-cross-realm.md");

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
        if !character.is_whitespace() {
            code.push(character);
        } else if !retain_literals {
            code.push(' ');
        }
        offset += character.len_utf8();
    }
    code
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

fn count_fragment_in_rust_sources(dir: &Path, fragment: &str) -> usize {
    fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .map(|path| {
            if path.is_dir() {
                return count_fragment_in_rust_sources(&path, fragment);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return 0;
            }
            fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
                .matches(fragment)
                .count()
        })
        .sum()
}

fn positions_in_order(source: &str, markers: &[&str]) {
    let mut cursor = 0;
    for marker in markers {
        let offset = source[cursor..]
            .find(marker)
            .unwrap_or_else(|| panic!("missing marker after byte {cursor}: `{marker}`"));
        cursor += offset + marker.len();
    }
}

#[test]
fn descriptor_prototype_is_one_private_non_cloneable_authority() {
    assert_eq!(
        REFLECT_SOURCE
            .matches("\nmod descriptor_object_prototype;\n")
            .count(),
        1,
    );
    assert!(!REFLECT_SOURCE.contains("pub mod descriptor_object_prototype;"));
    assert!(!REFLECT_SOURCE.contains("descriptor_object_prototype::"));
    assert!(!REFLECT_SOURCE.contains("ReflectDescriptorObjectPrototypeLocal"));
    let lexical_probe = rust_code(
        r###"
        // ReflectDescriptorObjectPrototypeLocal
        ReflectDescriptorObjectPrototypeLocal /* nested /* ignored */ comment */ :: r#consume;
        "ReflectDescriptorObjectPrototypeLocal"; br##"ReflectDescriptorObjectPrototypeLocal"##;
        'R'; b'R'; 'lifetime;
        "###,
        false,
    );
    assert_eq!(
        exact_identifier_count(&lexical_probe, "ReflectDescriptorObjectPrototypeLocal"),
        1
    );

    let declaration = rust_code(
        bounded(
            DESCRIPTOR_OBJECT_PROTOTYPE_SOURCE,
            "#[must_use = \"the Reflect descriptor Object prototype must be consumed\"]",
            "impl<'a> FunctionBuilder<'a>",
        ),
        true,
    );
    assert_eq!(
        declaration,
        "pub(super)structReflectDescriptorObjectPrototypeLocal(u32);"
    );
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_identifier_in_rust_sources(&source_root, "ReflectDescriptorObjectPrototypeLocal"),
        5
    );
    assert_eq!(
        count_fragment_in_rust_sources(&source_root, "descriptor_object_prototype::"),
        0,
    );
    assert!(!DESCRIPTOR_OBJECT_PROTOTYPE_SOURCE.contains("#[derive"));
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq", "Default"] {
        assert!(!DESCRIPTOR_OBJECT_PROTOTYPE_SOURCE.contains(&format!(
            "impl {capability} for ReflectDescriptorObjectPrototypeLocal"
        )));
    }
    assert_eq!(
        DESCRIPTOR_OBJECT_PROTOTYPE_SOURCE
            .matches("ReflectDescriptorObjectPrototypeLocal(prototype_local)")
            .count(),
        2,
        "the child must own the sole construction and consuming destructure",
    );
}

#[test]
fn descriptor_prototype_projection_has_only_entry_and_required_created_realm_routes() {
    let producer = rust_code(
        bounded(
            DESCRIPTOR_OBJECT_PROTOTYPE_SOURCE,
            "fn emit_reflect_descriptor_object_prototype(",
            "fn emit_alloc_reflect_descriptor_object(",
        ),
        true,
    );
    positions_in_order(
        &producer,
        &[
            "LocalGet(self.current_env_local)",
            "I64Eqz",
            "If(BlockType::Empty)",
            "GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX)",
            "Else",
            "HEAP_FUNCTION_DEFINING_REALM_OFFSET",
            "HEAP_REALM_INTRINSICS_OFFSET",
            "HEAP_REALM_INTRINSICS_OBJECT_PROTOTYPE_OFFSET",
            "ReflectDescriptorObjectPrototypeLocal(prototype_local)",
        ],
    );
    assert_eq!(producer.matches("Instruction::Unreachable").count(), 3);
    assert_eq!(
        producer
            .matches("Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX)")
            .count(),
        1
    );
    assert!(!producer.contains("emit_load_function_defining_realm_object_prototype"));
}

#[test]
fn reflect_define_property_consumes_the_proof_at_its_only_descriptor_allocation() {
    let consumer = rust_code(
        bounded(
            DESCRIPTOR_OBJECT_PROTOTYPE_SOURCE,
            "fn emit_alloc_reflect_descriptor_object(",
            "\n    }\n}",
        ),
        true,
    );
    assert!(consumer.starts_with(
        "&mutself,prototype:ReflectDescriptorObjectPrototypeLocal,descriptor_payload_local:u32,function:&mutFunction,)->Result<(),EmitError>{letReflectDescriptorObjectPrototypeLocal(prototype_local)=prototype;"
    ));
    positions_in_order(
        &consumer,
        &[
            "emit_alloc_plain_object_with_prototype(Some(prototype_local),None,function)",
            "LocalSet(descriptor_payload_local)",
            "release_temp_local(prototype_local)",
        ],
    );

    let define_property = rust_code(
        bounded(
            REFLECT_SOURCE,
            "pub(crate) fn compile_reflect_define_property_builtin(",
            "pub(crate) fn compile_reflect_delete_property_builtin(",
        ),
        true,
    );
    assert_eq!(
        define_property
            .matches("emit_reflect_descriptor_object_prototype(function)")
            .count(),
        1
    );
    assert_eq!(
        define_property
            .matches("emit_alloc_reflect_descriptor_object(")
            .count(),
        1
    );
    assert!(!define_property.contains("Some(OBJECT_PROTOTYPE_GLOBAL_INDEX)"));
}

#[test]
fn created_realm_reflect_methods_are_self_backed_and_the_contract_is_recorded() {
    let reflect_publication = rust_code(
        bounded(
            HOST_SOURCE,
            "let reflect_static_method_metas = [",
            "let math_static_method_metas = [",
        ),
        true,
    );
    assert_eq!(
        reflect_publication
            .matches("StandardBuiltinId::ReflectDefineProperty.function_id()")
            .count(),
        1
    );
    let reflect_installation = rust_code(
        bounded(
            HOST_SOURCE,
            "for (name, meta) in &reflect_static_method_metas {",
            "self.emit_object_define_local_data_with_flags(",
        ),
        true,
    );
    positions_in_order(
        &reflect_installation,
        &[
            "emit_function_value_payload_in_realm(meta,&realm_functions,method_payload_local,function,)",
            "HEAP_FUNCTION_ENV_HANDLE_OFFSET,method_payload_local",
            "emit_object_define_local_data(reflect_object_local,name,method_payload_local,tag_local,function,)",
        ],
    );

    for marker in [
        "Reflect descriptor Object prototype",
        "consumes the prototype proof",
        "does not claim general Realm isolation",
    ] {
        assert!(
            CONTRACT.contains(marker),
            "missing contract marker `{marker}`"
        );
    }
    assert!(TASK.contains("reflect-descriptor-object-realm.md"));
}
