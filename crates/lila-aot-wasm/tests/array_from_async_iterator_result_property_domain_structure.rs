use std::fs;
use std::path::Path;

const SOURCE: &str = include_str!("../src/builtins/array_from_async.rs");
const CONTRACT: &str = include_str!(
    "../../../docs/rust-rewrite/contracts/array-from-async-iterator-result-property-domain.md"
);
const TASK: &str = include_str!("../../../tasks/16-arrays-and-array-builtins.md");

fn bounded_inclusive<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_offset = source
        .find(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"));
    source[start_offset..]
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
            exact_identifier_count(&normalize_rust(&source).identifiers, identifier)
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

fn fnv1a(source: &str) -> u64 {
    source.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100_0000_01b3)
    })
}

#[test]
fn iterator_result_property_is_one_private_closed_domain() {
    let lexical_probe = r###"
        // ArrayFromAsyncIteratorResultProperty::Done
        ArrayFromAsyncIteratorResultProperty /* ignored */ :: r#Value;
        "ArrayFromAsyncIteratorResultProperty::Done";
        r#"ArrayFromAsyncIteratorResultProperty"#;
    "###;
    let lexical_probe = normalize_rust(lexical_probe);
    assert_eq!(
        exact_identifier_count(
            &lexical_probe.identifiers,
            "ArrayFromAsyncIteratorResultProperty"
        ),
        1
    );
    assert_eq!(
        exact_identifier_count(
            &lexical_probe.routes,
            "ArrayFromAsyncIteratorResultProperty::Value"
        ),
        1
    );

    let declaration_marker = "enum ArrayFromAsyncIteratorResultProperty {";
    let declaration_offset = SOURCE
        .find(declaration_marker)
        .expect("iterator-result property declaration");
    let declaration_start = SOURCE[..declaration_offset]
        .rfind("\n\n")
        .map_or(0, |offset| offset + 2);
    let following_impl = SOURCE[declaration_offset..]
        .find("impl ArrayFromAsyncIteratorResultProperty {")
        .map(|offset| declaration_offset + offset)
        .expect("iterator-result property impl");
    assert_eq!(
        normalize_rust(&SOURCE[declaration_start..following_impl]).code,
        "enumArrayFromAsyncIteratorResultProperty{Done,Value,}",
        "the private domain must remain exact and attribute-free"
    );

    let projection = normalize_rust(bounded_inclusive(
        SOURCE,
        "impl ArrayFromAsyncIteratorResultProperty {",
        "#[must_use = \"Array.fromAsync execution Realm context must be explicitly released\"]",
    ));
    assert_eq!(
        projection.code,
        concat!(
            "implArrayFromAsyncIteratorResultProperty{",
            "constfnkey(self)->&'staticstr{matchself{",
            "Self::Done=>\"done\",Self::Value=>\"value\",}}}"
        )
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_identifier_in_rust_sources(&source_root, "ArrayFromAsyncIteratorResultProperty"),
        11,
        "the declaration, impl, reader and eight producers own every type mention"
    );
    let routes = normalize_rust(SOURCE).routes;
    assert_eq!(
        exact_identifier_count(&routes, "ArrayFromAsyncIteratorResultProperty::Done"),
        4
    );
    assert_eq!(
        exact_identifier_count(&routes, "ArrayFromAsyncIteratorResultProperty::Value"),
        4
    );
    for capability in ["Clone", "Copy", "Debug", "Default", "PartialEq", "Eq"] {
        assert!(!routes.contains(&format!(
            "impl{capability}forArrayFromAsyncIteratorResultProperty"
        )));
    }
}

#[test]
fn iterator_result_property_reader_consumes_the_only_selection() {
    let reader = normalize_rust(bounded_inclusive(
        SOURCE,
        "fn emit_array_from_async_read_iterator_result_property(",
        "fn emit_array_from_async_read_array_like_value(",
    ));
    assert!(reader.code.starts_with(concat!(
        "fnemit_array_from_async_read_iterator_result_property(&mutself,",
        "iterator_result_payload_local:u32,iterator_result_tag_local:u32,",
        "property:ArrayFromAsyncIteratorResultProperty,"
    )));
    assert_eq!(exact_identifier_count(&reader.identifiers, "property"), 2);
    assert_eq!(
        exact_identifier_count(&reader.routes, "property.key"),
        1,
        "the typed selection must be projected exactly once"
    );
    for forbidden in [
        "&property",
        "property.clone",
        "discriminant(&property)",
        "matchproperty",
        "matches!(property",
        "property==",
        "property!=",
        "propertyas",
    ] {
        assert!(!reader.code.contains(forbidden), "found `{forbidden}`");
    }
    assert_eq!(
        (reader.code.len(), fnv1a(&reader.code)),
        (701, 7_163_376_605_815_621_529),
        "the complete consuming reader must retain its structure"
    );
}

#[test]
fn four_continuations_own_one_done_then_value_pair_each() {
    let continuations = [
        (
            "fn emit_array_from_async_iterable_start(",
            "pub(crate) fn emit_array_from_async_fulfilled(",
            (9610, 16_735_338_314_939_235_615),
        ),
        (
            "pub(crate) fn emit_array_from_async_fulfilled(",
            "pub(crate) fn emit_array_from_async_rejected(",
            (9971, 6_392_141_318_506_301_516),
        ),
        (
            "fn emit_array_from_async_schedule_iterator_step_callback(",
            "fn emit_array_from_async_close_or_reject_callback_current_throw(",
            (3819, 585_541_180_704_700_588),
        ),
        (
            "fn emit_array_from_async_begin_close_current_throw(",
            "fn emit_array_from_async_reject_saved_error_on_current_throw(",
            (4963, 15_084_420_027_097_662_195),
        ),
    ];

    for (start, end, fingerprint) in continuations {
        let continuation = normalize_rust(bounded_inclusive(SOURCE, start, end));
        assert_eq!(
            exact_identifier_count(
                &continuation.routes,
                "ArrayFromAsyncIteratorResultProperty::Done"
            ),
            1,
            "Done producer in `{start}`"
        );
        assert_eq!(
            exact_identifier_count(
                &continuation.routes,
                "ArrayFromAsyncIteratorResultProperty::Value"
            ),
            1,
            "Value producer in `{start}`"
        );
        positions_in_order(
            &continuation.code,
            &[
                "ArrayFromAsyncIteratorResultProperty::Done",
                "ArrayFromAsyncIteratorResultProperty::Value",
            ],
        );
        assert_eq!(
            (continuation.code.len(), fnv1a(&continuation.code)),
            fingerprint,
            "complete continuation body `{start}`"
        );
    }

    let normalized_source = normalize_rust(SOURCE);
    assert_eq!(
        exact_identifier_count(
            &normalized_source.routes,
            "self.emit_array_from_async_read_iterator_result_property"
        ),
        8
    );
}

#[test]
fn contract_and_t16_record_the_one_shot_property_authority() {
    let contract_words = CONTRACT.split_whitespace().collect::<Vec<_>>().join(" ");
    let task_words = TASK.split_whitespace().collect::<Vec<_>>().join(" ");
    for marker in [
        "non-derived `ArrayFromAsyncIteratorResultProperty::{Done, Value}` selection",
        "exact 11 type mentions",
        "four producers per variant",
        "complete reader and all four continuation bodies",
    ] {
        assert!(
            contract_words.contains(marker) || task_words.contains(marker),
            "missing contract/task marker: {marker}"
        );
    }
    for text in [&contract_words, &task_words] {
        assert!(
            text.contains("full-body fingerprints")
                || text.contains("fingerprints the complete reader")
        );
        assert!(text.contains("passes `4/4`"));
    }
}
