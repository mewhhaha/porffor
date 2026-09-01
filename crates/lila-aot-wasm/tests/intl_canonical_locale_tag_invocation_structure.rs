use std::fs;
use std::path::Path;

const INTL_SOURCE: &str = include_str!("../src/builtins/intl.rs");
const DTF_SOURCE: &str = include_str!("../src/builtins/intl_datetimeformat.rs");
const CLI_SOURCE: &str = include_str!("../../lila-cli/tests/cli/intl.rs");
const FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_intl_canonical_locale_tag_roles.js");
const CONTRACT: &str = include_str!(
    "../../../docs/rust-rewrite/contracts/intl-canonical-locale-tag-invocation-authority.md"
);
const TASK: &str = include_str!("../../../tasks/23-intl402.md");

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

const ROLES: [&str; 7] = [
    "TagInputPayload",
    "TagPayload",
    "LanguagePayload",
    "ScriptPayload",
    "RegionPayload",
    "BaseNamePayload",
    "Validity",
];

#[test]
fn invocation_authority_is_the_exact_private_move_only_domain() {
    let lexical_probe = rust_code(
        r###"
        // CanonicalLocaleTagInvocationLocals
        CanonicalLocaleTagInvocationLocals /* nested /* ignored */ comment */;
        "CanonicalLocaleTagInvocationLocals"; b"CanonicalLocaleTagInvocationLocals";
        c"CanonicalLocaleTagInvocationLocals"; r"CanonicalLocaleTagInvocationLocals";
        br##"CanonicalLocaleTagInvocationLocals"##;
        cr#"CanonicalLocaleTagInvocationLocals"#;
        'C'; b'C'; 'lifetime; r#CanonicalLocaleTagInvocationLocals;
        "###,
    );
    assert_eq!(
        exact_identifier_count(
            &lexical_probe.identifiers,
            "CanonicalLocaleTagInvocationLocals"
        ),
        2
    );

    let module = rust_code(bounded(
        INTL_SOURCE,
        "mod canonical_locale_tag_invocation {",
        "\n}\n\npub(super) use canonical_locale_tag_invocation::{",
    ));
    for role in ROLES {
        assert!(module.normalized.contains(&format!(
            "pub(incrate::builtins)structCanonicalLocale{role}Local(u32);"
        )));
    }
    assert!(module.normalized.contains(concat!(
        "pub(incrate::builtins)constfnnew(",
        "input:CanonicalLocaleTagInputPayloadLocal,",
        "tag:CanonicalLocaleTagPayloadLocal,",
        "language:CanonicalLocaleLanguagePayloadLocal,",
        "script:CanonicalLocaleScriptPayloadLocal,",
        "region:CanonicalLocaleRegionPayloadLocal,",
        "base_name:CanonicalLocaleBaseNamePayloadLocal,",
        "validity:CanonicalLocaleValidityLocal,",
        ")->Self"
    )));
    for forbidden in ["derive(", "implClonefor", "implCopyfor"] {
        assert!(
            !module.normalized.contains(forbidden),
            "found `{forbidden}`"
        );
    }
}

#[test]
fn role_and_authority_census_is_closed_over_product_sources() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_identifier_in_rust_sources(&source_root, "CanonicalLocaleTagInvocationLocals"),
        10
    );
    for role in ROLES {
        assert_eq!(
            count_identifier_in_rust_sources(&source_root, &format!("CanonicalLocale{role}Local")),
            11,
            "CanonicalLocale{role}Local census"
        );
    }
}

#[test]
fn five_producers_construct_all_roles_and_the_consumer_projects_once() {
    let product_source = format!("{INTL_SOURCE}\n{DTF_SOURCE}");
    assert_eq!(
        product_source
            .matches("CanonicalLocaleTagInvocationLocals::new(")
            .count(),
        5
    );
    for role in ROLES {
        assert_eq!(
            product_source
                .matches(&format!("CanonicalLocale{role}Local::new("))
                .count(),
            5,
            "CanonicalLocale{role}Local producer census"
        );
    }

    let consumer = rust_code(bounded(
        INTL_SOURCE,
        "    pub(super) fn emit_intl_canonicalize_locale_tag(",
        "        let src_offset_local = self.reserve_temp_local();",
    ));
    assert!(consumer
        .normalized
        .contains("invocation:CanonicalLocaleTagInvocationLocals,function:&mutFunction,"));
    assert!(consumer.normalized.contains(concat!(
        "let(input_payload_local,tag_payload_local,language_payload_local,",
        "script_payload_local,region_payload_local,base_name_payload_local,",
        "ok_local,)=invocation.into_parts();"
    )));
    for forbidden in [
        "input_payload_local:u32",
        "tag_payload_local:u32",
        "language_payload_local:u32",
        "base_name_payload_local:u32",
        "ok_local:u32",
    ] {
        assert!(
            !consumer.normalized.contains(forbidden),
            "found `{forbidden}`"
        );
    }
    assert_eq!(INTL_SOURCE.matches("invocation.into_parts()").count(), 1);
}

#[test]
fn contract_task_and_public_fixture_own_the_authority() {
    for marker in [
        "canonical locale tag invocation authority",
        "transpose tag, language, script, region, base-name, and validity roles",
        "intl_canonical_locale_tag_invocation_structure",
    ] {
        assert!(CONTRACT.contains(marker), "contract marker `{marker}`");
        assert!(TASK.contains(marker), "task marker `{marker}`");
    }
    assert_eq!(
        CLI_SOURCE
            .matches("run_wasm_intl_canonical_locale_tag_roles_fixture_succeeds")
            .count(),
        1
    );
    assert_eq!(
        CLI_SOURCE
            .matches("wasm_intl_canonical_locale_tag_roles.js")
            .count(),
        1
    );
    for witness in [
        "locale.language !== \"en\"",
        "locale.script !== \"Latn\"",
        "locale.region !== \"US\"",
        "locale.baseName !== \"en-Latn-US\"",
        "resolved.locale !== \"en-US-u-ca-iso8601\"",
    ] {
        assert!(FIXTURE.contains(witness), "fixture witness `{witness}`");
    }
}
