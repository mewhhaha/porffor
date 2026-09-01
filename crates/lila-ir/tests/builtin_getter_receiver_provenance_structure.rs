use std::fs;
use std::path::Path;

const OWNER_SOURCE: &str = include_str!("../src/lowering/ordinary_property_compound.rs");
const LOWERING_SOURCE: &str = include_str!("../src/lowering.rs");

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
    code: String,
    identifiers: String,
}

fn normalize_rust(source: &str) -> NormalizedRust {
    let bytes = source.as_bytes();
    let mut code = String::new();
    let mut identifiers = String::new();
    let mut offset = 0;
    while offset < bytes.len() {
        if let Some(end) = literal_end(source, offset) {
            code.push('L');
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
            assert_eq!(depth, 0, "unterminated block comment in Rust source");
            continue;
        }
        if bytes.get(offset..offset + 2) == Some(b"r#") {
            let identifier_start = source[offset + 2..].chars().next();
            if identifier_start
                .is_some_and(|character| character == '_' || character.is_alphabetic())
            {
                offset += 2;
                continue;
            }
        }
        let character = source[offset..].chars().next().unwrap();
        if !character.is_whitespace() {
            code.push(character);
        }
        identifiers.push(character);
        offset += character.len_utf8();
    }
    NormalizedRust { code, identifiers }
}

fn lexically_normalized_code(source: &str) -> String {
    normalize_rust(source).code
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

fn count_in_normalized_rust_sources(root: &Path, needle: &str) -> usize {
    fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()))
        .map(|entry| {
            entry
                .expect("source directory entry should be readable")
                .path()
        })
        .map(|path| {
            if path.is_dir() {
                return count_in_normalized_rust_sources(&path, needle);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return 0;
            }
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            lexically_normalized_code(&source).matches(needle).count()
        })
        .sum()
}

fn count_identifier_in_rust_sources(root: &Path, identifier: &str) -> usize {
    fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()))
        .map(|entry| {
            entry
                .expect("source directory entry should be readable")
                .path()
        })
        .map(|path| {
            if path.is_dir() {
                return count_identifier_in_rust_sources(&path, identifier);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return 0;
            }
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            exact_identifier_count(&normalize_rust(&source).identifiers, identifier)
        })
        .sum()
}

#[test]
fn builtin_getter_receiver_provenance_is_the_exact_private_no_capability_domain() {
    let lexical_probe = r###"
        let r#receiver_provenance = BuiltinGetterReceiverProvenance::r#MayBeProxy;
        // receiver_provenance BuiltinGetterReceiverProvenance
        let normal = "receiver_provenance BuiltinGetterReceiverProvenance";
        let byte = b"receiver_provenance";
        let c_string = c"BuiltinGetterReceiverProvenance";
        let raw = r#"receiver_provenance"#;
        let raw_byte = br#"BuiltinGetterReceiverProvenance"#;
        let raw_c = cr#"receiver_provenance"#;
        let character = ':';
        let byte_character = b':';
        let borrowed: &'a str = value;
    "###;
    let normalized_probe = normalize_rust(lexical_probe);
    assert_eq!(
        normalized_probe.code,
        concat!(
            "letreceiver_provenance=BuiltinGetterReceiverProvenance::MayBeProxy;",
            "letnormal=L;letbyte=L;letc_string=L;letraw=L;letraw_byte=L;",
            "letraw_c=L;letcharacter=L;letbyte_character=L;letborrowed:&'astr=value;"
        )
    );
    assert_eq!(
        exact_identifier_count(&normalized_probe.identifiers, "receiver_provenance"),
        1
    );
    assert_eq!(
        exact_identifier_count(
            &normalized_probe.identifiers,
            "BuiltinGetterReceiverProvenance"
        ),
        1
    );

    let owner = lexically_normalized_code(OWNER_SOURCE);
    assert!(owner.starts_with(concat!(
        "usesuper::*;",
        "pub(super)enumBuiltinGetterReceiverProvenance{",
        "ProvenNonProxy,MayBeProxy,}",
        "pub(super)structOrdinaryPropertyReferenceMetadata{"
    )));
    for forbidden in [
        "implCloneforBuiltinGetterReceiverProvenance",
        "implCopyforBuiltinGetterReceiverProvenance",
        "implDebugforBuiltinGetterReceiverProvenance",
        "implPartialEqforBuiltinGetterReceiverProvenance",
        "implEqforBuiltinGetterReceiverProvenance",
    ] {
        assert!(
            !owner.contains(forbidden),
            "forbidden capability `{forbidden}`"
        );
    }

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_identifier_in_rust_sources(&source_root, "BuiltinGetterReceiverProvenance"),
        9,
        "the declaration, import, borrowed parameter, four producers and two exhaustive arms own every mention"
    );
    for variant in ["ProvenNonProxy", "MayBeProxy"] {
        assert_eq!(
            count_in_normalized_rust_sources(
                &source_root,
                &format!("BuiltinGetterReceiverProvenance::{variant}"),
            ),
            3,
            "variant `{variant}` must have two producers and one exhaustive arm"
        );
    }
    assert_eq!(
        count_identifier_in_rust_sources(&source_root, "receiver_provenance"),
        6,
        "two declarations, two borrowed calls, the parameter and its exhaustive match own every inferred use"
    );
    for forbidden_observer in [
        "receiver_provenance.clone(",
        "receiver_provenance==",
        "receiver_provenance!=",
        "matches!(receiver_provenance",
        "discriminant(&receiver_provenance",
        "receiver_provenanceas",
    ] {
        assert_eq!(
            count_in_normalized_rust_sources(&source_root, forbidden_observer),
            0,
            "forbidden inferred provenance observer `{forbidden_observer}`"
        );
    }
}

#[test]
fn both_receiver_shape_decisions_construct_the_exact_provenance() {
    let ordinary_producer = bounded(
        OWNER_SOURCE,
        "        let receiver_shapes_are_known = possible_receiver_values",
        "        let mut possible_getters = PropertyHookTargets::from_known(known_getters);",
    );
    assert_eq!(
        lexically_normalized_code(ordinary_producer),
        concat!(
            ".iter().all(|receiver|receiver.heap_shape.is_some());",
            "letreceiver_provenance=ifreceiver_shapes_are_known{",
            "BuiltinGetterReceiverProvenance::ProvenNonProxy",
            "}else{BuiltinGetterReceiverProvenance::MayBeProxy};"
        )
    );

    let direct_producer = bounded(
        LOWERING_SOURCE,
        "        let receiver_provenance = if target.heap_shape.is_some() {",
        "        let known_getter_may_call_user_code = match known_builtin_getter {",
    );
    assert_eq!(
        lexically_normalized_code(direct_producer),
        concat!(
            "BuiltinGetterReceiverProvenance::ProvenNonProxy",
            "}else{BuiltinGetterReceiverProvenance::MayBeProxy};"
        )
    );
}

#[test]
fn both_routes_borrow_the_provenance_without_alternate_consumers() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_normalized_rust_sources(
            &source_root,
            "standard_builtin_getter_may_call_user_code",
        ),
        3,
        "one definition and two calls must own the complete route census"
    );

    let ordinary_route = lexically_normalized_code(bounded(
        OWNER_SOURCE,
        "        let getter_may_dispatch_transitive_property_hooks = possible_getters",
        "        if getter_may_dispatch_transitive_property_hooks {",
    ));
    assert_eq!(
        ordinary_route,
        concat!(
            ".iter().filter_map(|getter|StandardBuiltinId::from_function_id(getter))",
            ".any(|getter|{Self::standard_builtin_getter_may_call_user_code(",
            "getter,&receiver_provenance)});"
        )
    );

    let direct_route = lexically_normalized_code(bounded(
        LOWERING_SOURCE,
        "        let known_getter_may_call_user_code = match known_builtin_getter {",
        "        let builtin_getter_may_dispatch =",
    ));
    assert_eq!(
        direct_route,
        concat!(
            "Some(builtin)=>{Self::standard_builtin_getter_may_call_user_code(",
            "builtin,&receiver_provenance)}None=>known_getter.is_some(),};"
        )
    );
}

#[test]
fn proto_getter_dispatch_projects_the_borrowed_provenance_exhaustively() {
    let consumer = bounded(
        OWNER_SOURCE,
        "    pub(super) fn standard_builtin_getter_may_call_user_code(",
        "    pub(super) fn pre_write_global_property_value(",
    );
    assert_eq!(
        lexically_normalized_code(consumer),
        concat!(
            "builtin:StandardBuiltinId,",
            "receiver_provenance:&BuiltinGetterReceiverProvenance,)->bool{matchbuiltin{",
            "StandardBuiltinId::MapPrototypeSizeGetter|",
            "StandardBuiltinId::SetPrototypeSizeGetter|",
            "StandardBuiltinId::TemporalZonedDateTimePrototypeOffsetGetter|",
            "StandardBuiltinId::TemporalZonedDateTimePrototypeOffsetNanosecondsGetter=>false,",
            "StandardBuiltinId::ObjectPrototypeProtoGetter=>matchreceiver_provenance{",
            "BuiltinGetterReceiverProvenance::ProvenNonProxy=>false,",
            "BuiltinGetterReceiverProvenance::MayBeProxy=>true,},_=>true,}}"
        )
    );
    assert!(!lexically_normalized_code(consumer).contains("matches!("));
}
