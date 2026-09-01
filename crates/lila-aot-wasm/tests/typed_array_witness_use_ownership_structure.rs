use std::fs;
use std::path::Path;

const SOURCE: &str = include_str!("../src/builtins/binary_data.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/typed-array-witness-use-ownership.md");
const TASK: &str = include_str!("../../../tasks/17-typedarrays-binary-data-atomics.md");

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

fn count_normalized_in_rust_sources(dir: &Path, needle: &str) -> usize {
    fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .map(|path| {
            if path.is_dir() {
                return count_normalized_in_rust_sources(&path, needle);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return 0;
            }
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            rust_code(&source).normalized.matches(needle).count()
        })
        .sum()
}

fn fnv1a(source: &str) -> u64 {
    source.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

#[test]
fn view_locals_is_the_exact_non_copyable_borrowed_carrier() {
    let declaration_marker = "pub(crate) struct TypedArrayViewLocals {";
    let declaration_offset = SOURCE
        .find(declaration_marker)
        .expect("TypedArray view-locals declaration");
    assert_eq!(
        SOURCE[..declaration_offset]
            .lines()
            .rev()
            .find(|line| !line.trim().is_empty())
            .map(str::trim),
        Some("/// again.")
    );

    let declaration = bounded(
        SOURCE,
        declaration_marker,
        "\n}\n\nimpl TypedArrayViewLocals",
    );
    assert_eq!(
        rust_code(declaration).normalized,
        concat!(
            "typed_array_payload_local:u32,",
            "buffer_payload_local:u32,",
            "byte_offset_local:u32,",
            "stored_byte_length_local:u32,",
            "bytes_per_element_local:u32,"
        )
    );
    assert_eq!(
        (declaration.len(), fnv1a(declaration)),
        (164, 0x3bce_01ef_c99c_d41e)
    );

    let constructor = bounded(
        SOURCE,
        "impl TypedArrayViewLocals {",
        "\n}\n\n/// The complete result domain",
    );
    assert_eq!(
        (constructor.len(), fnv1a(constructor)),
        (439, 0xe287_0c8f_9551_d511)
    );

    let witness = bounded(
        SOURCE,
        "    pub(crate) fn emit_typed_array_witness(",
        "    /// Compiles one of the three TypedArray view accessors",
    );
    assert_eq!(
        (witness.len(), fnv1a(witness)),
        (8433, 0xdba0_79dd_67aa_acdf)
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_identifier_in_rust_sources(&source_root, "TypedArrayViewLocals"),
        56
    );
    assert_eq!(
        count_normalized_in_rust_sources(&source_root, "TypedArrayViewLocals::new("),
        46
    );
    assert_eq!(
        count_normalized_in_rust_sources(&source_root, "&TypedArrayViewLocals"),
        2
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
        assert!(!SOURCE.contains(&format!("impl {capability} for TypedArrayViewLocals")));
    }
    assert!(!SOURCE.contains("TypedArrayViewLocals::clone"));
}

#[test]
fn witness_use_is_the_exact_crate_private_move_only_authority() {
    let lexical_probe = rust_code(
        r###"
        // TypedArrayWitnessUse::Accessor
        TypedArrayWitnessUse /* nested /* ignored */ comment */ :: r#Accessor;
        "TypedArrayWitnessUse"; b"TypedArrayWitnessUse"; c"TypedArrayWitnessUse";
        r"TypedArrayWitnessUse"; br##"TypedArrayWitnessUse"##;
        cr#"TypedArrayWitnessUse"#; 'T'; b'T'; 'lifetime;
        "###,
    );
    assert_eq!(
        exact_identifier_count(&lexical_probe.identifiers, "TypedArrayWitnessUse"),
        1
    );

    let declaration_marker = "pub(crate) enum TypedArrayWitnessUse {";
    let declaration_offset = SOURCE
        .find(declaration_marker)
        .expect("witness-use declaration");
    let preceding_item_end = SOURCE[..declaration_offset]
        .rfind('}')
        .expect("accessor-kind declaration end");
    let following_item_offset = SOURCE[declaration_offset..]
        .find("/// One live observation")
        .map(|offset| declaration_offset + offset)
        .expect("witness locals declaration");
    assert_eq!(
        rust_code(&SOURCE[preceding_item_end + 1..following_item_offset]).normalized,
        concat!(
            "pub(crate)enumTypedArrayWitnessUse{",
            "ValidatedMethodEntry{length_local:u32,},",
            "ArrayLikeLengthSnapshot{length_local:u32,},",
            "IntegerIndexedProperty{index_local:u32,result_local:u32,},",
            "Accessor{kind:TypedArrayAccessorKind,result_local:u32,},}"
        )
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_identifier_in_rust_sources(&source_root, "TypedArrayWitnessUse"),
        78
    );
    for forbidden in [
        "impl Clone for TypedArrayWitnessUse",
        "impl Copy for TypedArrayWitnessUse",
        "impl PartialEq for TypedArrayWitnessUse",
        "impl Eq for TypedArrayWitnessUse",
        "TypedArrayWitnessUse::clone",
    ] {
        assert!(!SOURCE.contains(forbidden), "found `{forbidden}`");
    }
}

#[test]
fn every_witness_use_route_has_an_exact_closed_projection() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for (variant, count) in [
        ("ValidatedMethodEntry", 33),
        ("ArrayLikeLengthSnapshot", 15),
        ("IntegerIndexedProperty", 18),
        ("Accessor", 4),
    ] {
        assert_eq!(
            count_identifier_in_rust_sources(
                &source_root,
                &format!("TypedArrayWitnessUse::{variant}"),
            ),
            count,
            "unexpected `{variant}` witness route"
        );
    }
    assert_eq!(
        count_identifier_in_rust_sources(&source_root, "emit_typed_array_witness"),
        62,
        "one definition and 61 calls must remain the complete witness boundary"
    );
}

#[test]
fn witness_validation_borrows_before_result_projection_consumes() {
    let witness = bounded(
        SOURCE,
        "    pub(crate) fn emit_typed_array_witness(",
        "    /// Compiles one of the three TypedArray view accessors",
    );
    assert_eq!(
        exact_identifier_count(&rust_code(witness).identifiers, "use_"),
        3
    );
    assert_eq!(witness.matches("match &use_ {").count(), 1);
    assert_eq!(witness.matches("match use_ {").count(), 1);
    let borrowed = witness
        .find("match &use_ {")
        .expect("borrowed validation match");
    let consumed = witness
        .rfind("match use_ {")
        .expect("consuming result match");
    assert!(borrowed < consumed);

    let validation = bounded(
        witness,
        "        match &use_ {",
        "\n\n        function.instruction(&Instruction::LocalGet(out_of_bounds_local));",
    );
    for variant in [
        "TypedArrayWitnessUse::ValidatedMethodEntry { .. }",
        "TypedArrayWitnessUse::ArrayLikeLengthSnapshot { .. }",
        "TypedArrayWitnessUse::IntegerIndexedProperty { .. }",
        "TypedArrayWitnessUse::Accessor { .. }",
    ] {
        assert_eq!(validation.matches(variant).count(), 1, "{variant}");
    }
    assert!(!validation.contains("{ length_local }"));
    assert!(!validation.contains("{ index_local,"));
    assert!(!validation.contains("{ kind, result_local }"));
    assert!(!validation.contains("_ =>"));

    let result = bounded(
        witness,
        "        match use_ {",
        "        self.release_temp_local(data_ptr_local);",
    );
    for variant in [
        "TypedArrayWitnessUse::ValidatedMethodEntry { length_local }",
        "TypedArrayWitnessUse::ArrayLikeLengthSnapshot { length_local }",
        "TypedArrayWitnessUse::IntegerIndexedProperty {",
        "TypedArrayWitnessUse::Accessor { kind, result_local }",
    ] {
        assert_eq!(result.matches(variant).count(), 1, "{variant}");
    }
    assert!(!result.contains("_ =>"));
    assert!(!result.contains("match &use_"));
}

#[test]
fn contract_and_task_record_the_owned_witness_use() {
    for marker in [
        "move-only witness-use",
        "final consuming projection",
        "typed_array_witness_use_ownership_structure",
    ] {
        assert!(CONTRACT.contains(marker), "contract marker `{marker}`");
        assert!(TASK.contains(marker), "task marker `{marker}`");
    }
}
