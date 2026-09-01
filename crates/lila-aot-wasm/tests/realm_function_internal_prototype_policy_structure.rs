use std::fs;
use std::path::Path;

const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");

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

struct NormalizedRust {
    identifiers: String,
    routes: String,
}

fn normalize_rust(source: &str) -> NormalizedRust {
    let bytes = source.as_bytes();
    let mut identifiers = String::new();
    let mut routes = String::new();
    let mut offset = 0;
    while offset < bytes.len() {
        if let Some(end) = literal_end(source, offset) {
            identifiers.push(' ');
            routes.push_str(&source[offset..end]);
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
            identifiers.push(character);
            routes.push(character);
        }
        offset += character.len_utf8();
    }
    NormalizedRust {
        identifiers,
        routes,
    }
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

fn normalized_rust_sources(dir: &Path) -> NormalizedRust {
    let mut paths = fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .collect::<Vec<_>>();
    paths.sort();
    let mut identifiers = String::new();
    let mut routes = String::new();
    for path in paths {
        if path.is_dir() {
            let nested = normalized_rust_sources(&path);
            identifiers.push_str(&nested.identifiers);
            routes.push_str(&nested.routes);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            let normalized = normalize_rust(&source);
            identifiers.push_str(&normalized.identifiers);
            identifiers.push(' ');
            routes.push_str(&normalized.routes);
            routes.push('\n');
        }
    }
    NormalizedRust {
        identifiers,
        routes,
    }
}

#[test]
fn realm_function_internal_prototype_policy_is_the_complete_non_capability_domain() {
    let lexical_probe = r###"
        RealmFunctionInternalPrototypePolicy /* nested /* ignored */ comment */ ::
            r#RealmFunctionPrototype;
        // RealmFunctionInternalPrototypePolicy::UnsupportedSpecializedPrototype
        let normal = "RealmFunctionInternalPrototypePolicy";
        let byte = b"RealmFunctionInternalPrototypePolicy";
        let c_string = c"RealmFunctionInternalPrototypePolicy";
        let raw = r#"RealmFunctionInternalPrototypePolicy"#;
        let raw_byte = br#"RealmFunctionInternalPrototypePolicy"#;
        let raw_c = cr#"RealmFunctionInternalPrototypePolicy"#;
        let character = ':';
        let byte_character = b':';
        let borrowed: &'a str = value;
    "###;
    let normalized_probe = normalize_rust(lexical_probe);
    assert_eq!(
        exact_identifier_count(
            &normalized_probe.identifiers,
            "RealmFunctionInternalPrototypePolicy"
        ),
        1
    );
    assert!(normalized_probe
        .routes
        .starts_with("RealmFunctionInternalPrototypePolicy::RealmFunctionPrototype;letnormal="));

    let declaration = bounded(
        FUNCTIONS_SOURCE,
        "pub(crate) enum FunctionPrototypeMaterialization {",
        "const fn realm_function_internal_prototype_policy(",
    );
    let expected_declaration = r#"
    Automatic,
    BootstrapSupplied,
}

enum RealmFunctionInternalPrototypePolicy {
    RealmFunctionPrototype,
    UnsupportedSpecializedPrototype,
}

"#;
    assert_eq!(
        normalize_rust(declaration).routes,
        normalize_rust(expected_declaration).routes,
        "the exact adjacent declaration must remain private and attribute-free"
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let source = normalized_rust_sources(&source_root);
    for (identifier, expected) in [
        ("RealmFunctionInternalPrototypePolicy", 10),
        ("RealmFunctionPrototype", 5),
        ("UnsupportedSpecializedPrototype", 5),
        ("realm_function_internal_prototype_policy", 4),
    ] {
        assert_eq!(
            exact_identifier_count(&source.identifiers, identifier),
            expected,
            "source ownership census drifted for {identifier}"
        );
    }
    for capability in ["Clone", "Copy", "Debug", "Default", "PartialEq", "Eq"] {
        assert!(!source.routes.contains(&format!(
            "impl{capability}forRealmFunctionInternalPrototypePolicy"
        )));
    }
}

#[test]
fn realm_function_internal_prototype_policy_preserves_all_four_rows_and_unit_observations() {
    let mapping_start = FUNCTIONS_SOURCE
        .find("const fn realm_function_internal_prototype_policy(")
        .expect("policy mapping");
    let mapping_end = FUNCTIONS_SOURCE[mapping_start..]
        .find("#[cfg(test)]")
        .map(|offset| mapping_start + offset)
        .expect("policy test module after mapping");
    let mapping = &FUNCTIONS_SOURCE[mapping_start..mapping_end];
    let expected_mapping = r#"const fn realm_function_internal_prototype_policy(
    execution_kind: FunctionExecutionKind,
) -> RealmFunctionInternalPrototypePolicy {
    match execution_kind {
        FunctionExecutionKind::Ordinary => {
            RealmFunctionInternalPrototypePolicy::RealmFunctionPrototype
        }
        FunctionExecutionKind::Generator
        | FunctionExecutionKind::Async
        | FunctionExecutionKind::AsyncGenerator => {
            RealmFunctionInternalPrototypePolicy::UnsupportedSpecializedPrototype
        }
    }
}

"#;
    assert_eq!(
        normalize_rust(mapping).routes,
        normalize_rust(expected_mapping).routes,
        "the four execution kinds must keep their exact policy rows"
    );

    let unit = bounded(
        FUNCTIONS_SOURCE,
        "    fn specialized_created_realm_function_prototypes_are_explicitly_unsupported() {",
        "    #[test]\n    fn created_realm_function_sites_require_the_coupled_context() {",
    );
    let expected_unit = r#"
        match realm_function_internal_prototype_policy(FunctionExecutionKind::Ordinary) {
            RealmFunctionInternalPrototypePolicy::RealmFunctionPrototype => {}
            RealmFunctionInternalPrototypePolicy::UnsupportedSpecializedPrototype => {
                panic!("ordinary created-realm functions require the realm Function prototype")
            }
        }
        for execution_kind in [
            FunctionExecutionKind::Generator,
            FunctionExecutionKind::Async,
            FunctionExecutionKind::AsyncGenerator,
        ] {
            match realm_function_internal_prototype_policy(execution_kind) {
                RealmFunctionInternalPrototypePolicy::RealmFunctionPrototype => {
                    panic!("specialized created-realm functions require specialized prototypes")
                }
                RealmFunctionInternalPrototypePolicy::UnsupportedSpecializedPrototype => {}
            }
        }
    }

"#;
    assert_eq!(
        normalize_rust(unit).routes,
        normalize_rust(expected_unit).routes,
        "the unit witness must observe both policies through exhaustive matches"
    );
}

#[test]
fn realm_function_internal_prototype_policy_rejects_before_created_realm_allocation() {
    let materializer = bounded(
        FUNCTIONS_SOURCE,
        "    fn emit_function_value_payload_in_realm_with_prototype_materialization(",
        "    pub(crate) fn reserve_realm_function_prototype_local(",
    );
    let expected_materializer = r#"
        &mut self,
        meta: &WasmFunctionMeta,
        prototype_materialization: FunctionPrototypeMaterialization,
        context: &RealmFunctionMaterializationContext,
        function_object_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match realm_function_internal_prototype_policy(meta.protocol.execution_kind()) {
            RealmFunctionInternalPrototypePolicy::RealmFunctionPrototype => {}
            RealmFunctionInternalPrototypePolicy::UnsupportedSpecializedPrototype => {
                return Err(EmitError::unsupported(format!(
                    "unsupported in lila wasm-aot: created-realm {:?} function `{}` requires its realm-local intrinsic function prototype",
                    meta.protocol.execution_kind(),
                    meta.name,
                )));
            }
        }
        self.emit_function_value_payload_with_prototype_materialization(
            meta,
            prototype_materialization,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(function_object_local));
        self.emit_store_function_defining_realm(
            function_object_local,
            context.realm.index(),
            function,
        );
        self.store_i64_local_at_offset(
            function_object_local,
            HEAP_PROTOTYPE_OFFSET,
            context.function_prototype_local,
            function,
        );
        self.store_i64_const_at_offset(
            function_object_local,
            HEAP_FUNCTION_INTERNAL_PROTOTYPE_TAG_OFFSET,
            ValueKind::Function.tag() as u64,
            function,
        );
        Ok(())
    }
"#;
    assert_eq!(
        normalize_rust(materializer).routes,
        normalize_rust(expected_materializer).routes,
        "the policy gate must be the first action before the sole allocation and publication body"
    );
}
