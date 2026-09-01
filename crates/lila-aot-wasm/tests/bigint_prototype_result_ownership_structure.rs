use std::fs;
use std::path::Path;

const PARENT_SOURCE: &str = include_str!("../src/builtins/bigint.rs");
const RADIX_FORMATTING_SOURCE: &str = include_str!("../src/builtins/bigint/radix_formatting.rs");
const SOURCE: &str = concat!(
    include_str!("../src/builtins/bigint.rs"),
    include_str!("../src/builtins/bigint/radix_formatting.rs")
);
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/bigint-prototype-result-policy.md");
const TASK: &str = include_str!("../../../tasks/20-number-bigint-math-json.md");

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
fn prototype_result_authorities_are_exact_move_only_domains() {
    let lexical_probe = rust_code(
        r###"
        // BigIntRadixStringResult
        BigIntRadixStringResult /* nested /* ignored */ comment */;
        "BigIntRadixStringResult"; b"BigIntRadixStringResult";
        c"BigIntRadixStringResult"; r"BigIntRadixStringResult";
        br##"BigIntRadixStringResult"##; cr#"BigIntRadixStringResult"#;
        'B'; b'B'; 'lifetime;
        "###,
        false,
    );
    assert_eq!(
        exact_identifier_count(&lexical_probe, "BigIntRadixStringResult"),
        1
    );

    let declarations = normalized_rust(bounded(
        PARENT_SOURCE,
        "mod radix_formatting;",
        "enum BigIntPrototypeResultPolicy {",
    ));
    assert_eq!(
        declarations,
        concat!(
            "structBigIntValueResult(());",
            "structBigIntRadixStringResult(());",
            "structBigIntLocaleStringFallbackResult(());"
        )
    );
    let policy = normalized_rust(bounded(
        PARENT_SOURCE,
        "enum BigIntPrototypeResultPolicy {",
        "enum BigIntFixedWidthOperation {",
    ));
    assert_eq!(
        policy,
        concat!(
            "ExactValue(BigIntValueResult),",
            "RadixString(BigIntRadixStringResult),",
            "LocaleStringFallback(BigIntLocaleStringFallbackResult),}"
        )
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for (identifier, count) in [
        ("BigIntValueResult", 4),
        ("BigIntRadixStringResult", 5),
        ("BigIntLocaleStringFallbackResult", 4),
        ("BigIntPrototypeResultPolicy", 11),
    ] {
        assert_eq!(
            count_identifier_in_rust_sources(&source_root, identifier),
            count,
            "unexpected `{identifier}` authority route"
        );
    }
    for forbidden in [
        "impl Clone for BigIntValueResult",
        "impl Copy for BigIntValueResult",
        "impl Clone for BigIntRadixStringResult",
        "impl Copy for BigIntRadixStringResult",
        "impl Clone for BigIntLocaleStringFallbackResult",
        "impl Copy for BigIntLocaleStringFallbackResult",
        "impl Clone for BigIntPrototypeResultPolicy",
        "impl Copy for BigIntPrototypeResultPolicy",
    ] {
        assert!(!SOURCE.contains(forbidden), "found `{forbidden}`");
    }
}

#[test]
fn prototype_names_construct_their_exact_result_authorities() {
    let producers = normalized_rust(bounded(
        SOURCE,
        "#[allow(non_upper_case_globals)]",
        "impl<'a> FunctionBuilder<'a> {",
    ));
    assert_eq!(
        producers,
        concat!(
            "implBigIntBuiltin{",
            "constPrototypeToString:Self=Self::Prototype(",
            "BigIntPrototypeResultPolicy::RadixString(BigIntRadixStringResult(()),));",
            "constPrototypeToLocaleString:Self=Self::Prototype(",
            "BigIntPrototypeResultPolicy::LocaleStringFallback(",
            "BigIntLocaleStringFallbackResult(())),);",
            "constPrototypeValueOf:Self=Self::Prototype(",
            "BigIntPrototypeResultPolicy::ExactValue(BigIntValueResult(()),));}"
        )
    );
}

#[test]
fn emitter_consumes_each_result_authority_once() {
    let emitter = bounded(
        SOURCE,
        "    fn emit_bigint_builtin(",
        "    fn emit_bigint_exact_value_result(",
    );
    assert_eq!(emitter.matches("match result_policy {").count(), 1);
    assert!(!emitter.contains("match &result_policy"));
    let result_match = bounded(
        emitter,
        "                match result_policy {",
        "                self.release_temp_local(bigint_tag_local);",
    );
    assert_eq!(
        result_match
            .matches("BigIntPrototypeResultPolicy::")
            .count(),
        3
    );
    assert_eq!(result_match.matches("=>").count(), 3);
    assert!(!result_match.contains("_ =>"));

    let normalized_emitter = normalized_rust(emitter);
    for handoff in [
        concat!(
            "BigIntPrototypeResultPolicy::ExactValue(result)=>{",
            "self.emit_bigint_exact_value_result(result,bigint_payload_local,",
            "bigint_tag_local,function,);}",
        ),
        concat!(
            "BigIntPrototypeResultPolicy::RadixString(result)=>{",
            "self.emit_bigint_radix_string_result(result,bigint_payload_local,",
            "bigint_tag_local,function,)?;}",
        ),
        concat!(
            "BigIntPrototypeResultPolicy::LocaleStringFallback(result)=>{",
            "self.emit_bigint_locale_string_fallback_result(result,bigint_payload_local,",
            "bigint_tag_local,function,)?;}",
        ),
    ] {
        assert_eq!(normalized_emitter.matches(handoff).count(), 1, "{handoff}");
    }

    let radix_result = normalized_rust(bounded(
        RADIX_FORMATTING_SOURCE,
        "    pub(super) fn emit_bigint_radix_string_result(",
        "    fn emit_prepare_bigint_radix(",
    ));
    assert_eq!(
        radix_result
            .matches("letradix=self.emit_prepare_bigint_radix(result,function)?;")
            .count(),
        1
    );
    assert_eq!(exact_identifier_count(&radix_result, "result"), 2);

    assert!(!PARENT_SOURCE.contains("PreparedBigIntRadixLocal"));
    assert!(!PARENT_SOURCE.contains("emit_prepare_bigint_radix("));
    assert!(!PARENT_SOURCE.contains("radix_formatting::"));
    assert_eq!(
        exact_identifier_count(SOURCE, "PreparedBigIntRadixLocal"),
        4
    );
    assert_eq!(
        SOURCE.matches("emit_bigint_radix_string_result(").count(),
        2
    );
    assert_eq!(SOURCE.matches("emit_prepare_bigint_radix(").count(), 2);
    assert_eq!(RADIX_FORMATTING_SOURCE.matches("self.0").count(), 2);
    assert_eq!(RADIX_FORMATTING_SOURCE.matches("radix.local()").count(), 2);
    assert_eq!(
        RADIX_FORMATTING_SOURCE
            .matches("radix.into_local()")
            .count(),
        1
    );
    assert!(RADIX_FORMATTING_SOURCE.contains("struct PreparedBigIntRadixLocal(u32);"));
    assert!(!RADIX_FORMATTING_SOURCE.contains("pub struct PreparedBigIntRadixLocal"));
    assert!(!RADIX_FORMATTING_SOURCE.contains("pub(super) struct PreparedBigIntRadixLocal"));
    assert!(!RADIX_FORMATTING_SOURCE.contains("pub(super) fn emit_prepare_bigint_radix("));
    for capability in ["Clone", "Copy", "PartialEq", "Eq"] {
        assert_eq!(
            exact_identifier_count(RADIX_FORMATTING_SOURCE, capability),
            0,
            "prepared radix must not gain `{capability}`"
        );
    }
}

#[test]
fn contract_and_task_record_the_move_only_boundary() {
    for marker in [
        "move-only result authority",
        "cannot be duplicated by",
        "bigint_prototype_result_ownership_structure",
        "Batch AX",
        "six fixed BigInt entries",
        "source-equivalent",
    ] {
        assert!(CONTRACT.contains(marker), "contract marker `{marker}`");
        assert!(TASK.contains(marker), "task marker `{marker}`");
    }
}
