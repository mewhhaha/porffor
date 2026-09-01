use std::fs;
use std::path::Path;

const OBJECTS_SOURCE: &str = include_str!("../src/objects.rs");
const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const OBJECT_SOURCE: &str = include_str!("../src/builtins/object.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/stored-descriptor-role-relation.md");
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

struct RustCode {
    normalized: String,
    identifiers: String,
}

fn rust_code(source: &str) -> RustCode {
    let bytes = source.as_bytes();
    let mut normalized = String::new();
    let mut identifiers = String::new();
    let mut offset = 0;
    while offset < bytes.len() {
        if let Some(end) = literal_end(source, offset) {
            normalized.push_str(&source[offset..end]);
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
        if character.is_whitespace() {
            identifiers.push(' ');
        } else {
            normalized.push(character);
            identifiers.push(character);
        }
        offset += character.len_utf8();
    }
    RustCode {
        normalized,
        identifiers,
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
            exact_identifier_count(&rust_code(&source).identifiers, identifier)
        })
        .sum()
}

#[test]
fn stored_descriptor_roles_are_exact_private_wrappers() {
    let lexical_probe = rust_code(
        r###"
        // StoredDescriptorDataLocals
        StoredDescriptorDataLocals /* nested /* ignored */ comment */;
        "StoredDescriptorDataLocals"; b"StoredDescriptorDataLocals";
        c"StoredDescriptorDataLocals"; r"StoredDescriptorDataLocals";
        br##"StoredDescriptorDataLocals"##; cr#"StoredDescriptorDataLocals"#;
        'S'; b'S'; 'lifetime; r#StoredDescriptorDataLocals;
        "###,
    );
    assert_eq!(
        exact_identifier_count(&lexical_probe.identifiers, "StoredDescriptorDataLocals"),
        2
    );

    let role_declarations = rust_code(bounded(
        OBJECTS_SOURCE,
        "/// Allocation-free stored fields consumed by descriptor compatibility.",
        "/// A canonical property-key payload and its retained ECMAScript tag.",
    ));
    for role in ["Data", "Getter", "Setter"] {
        assert!(role_declarations.normalized.contains(&format!(
            "pub(crate)structStoredDescriptor{role}Locals(TaggedLocals);"
        )));
        assert!(!role_declarations.normalized.contains(&format!(
            "structStoredDescriptor{role}Locals(pub(crate)TaggedLocals)"
        )));
    }
    assert!(role_declarations.normalized.contains(concat!(
        "pub(crate)constfnnew(",
        "data:StoredDescriptorDataLocals,",
        "getter:StoredDescriptorGetterLocals,",
        "setter:StoredDescriptorSetterLocals,",
        ")->Self"
    )));
}

#[test]
fn role_types_have_one_closed_source_census() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for role in ["Data", "Getter", "Setter"] {
        assert_eq!(
            count_identifier_in_rust_sources(
                &source_root,
                &format!("StoredDescriptor{role}Locals")
            ),
            9,
            "StoredDescriptor{role}Locals census"
        );
    }
}

#[test]
fn all_three_producers_label_every_stored_descriptor_role() {
    let named_producer = bounded(
        OBJECTS_SOURCE,
        "    pub(crate) fn emit_validate_array_named_descriptor(",
        "    pub(crate) fn emit_validate_stored_descriptor(",
    );
    for (role, expected_value) in [
        ("Data", "TaggedLocals::new("),
        ("Getter", "TaggedLocals::new("),
        ("Setter", "TaggedLocals::new("),
    ] {
        assert_eq!(
            named_producer
                .matches(&format!("StoredDescriptor{role}Locals::new("))
                .count(),
            1
        );
        assert!(named_producer.contains(expected_value));
    }

    for source in [ARRAY_SOURCE, OBJECT_SOURCE] {
        assert_eq!(
            source
                .matches("StoredDescriptorDataLocals::new(existing_value)")
                .count(),
            1
        );
        assert_eq!(
            source
                .matches("StoredDescriptorGetterLocals::new(existing_value)")
                .count(),
            1
        );
        assert_eq!(
            source
                .matches("StoredDescriptorSetterLocals::new(existing_setter)")
                .count(),
            1
        );
    }
}

#[test]
fn contract_and_task_record_the_role_relation() {
    for marker in [
        "stored descriptor role relation",
        "cannot transpose data, getter, and setter locals",
        "stored_descriptor_role_relation_structure",
    ] {
        assert!(CONTRACT.contains(marker), "contract marker `{marker}`");
        assert!(TASK.contains(marker), "task marker `{marker}`");
    }
}
