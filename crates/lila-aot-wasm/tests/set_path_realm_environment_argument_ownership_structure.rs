use std::fs;
use std::path::Path;

const REALM_SOURCE: &str = include_str!("../src/objects/set_path_realm.rs");
const OBJECTS_SOURCE: &str = include_str!("../src/objects.rs");
const CONTRACT: &str = include_str!(
    "../../../docs/rust-rewrite/contracts/set-path-realm-environment-argument-ownership.md"
);
const TASK: &str = include_str!("../../../tasks/11-proxy-reflect-metaobject.md");

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
fn realm_argument_is_the_exact_private_capability_free_two_row_authority() {
    let probe = rust_code(
        r###"
        // SetPathRealmEnvironmentArgument::MainRealmFallback
        SetPathRealmEnvironmentArgument /* nested /* ignored */ comment */ :: r#TrustedCurrentEnvironment;
        "SetPathRealmEnvironmentArgument"; b"SetPathRealmEnvironmentArgument";
        c"SetPathRealmEnvironmentArgument"; r#"SetPathRealmEnvironmentArgument"#;
        br#"SetPathRealmEnvironmentArgument"#; cr#"SetPathRealmEnvironmentArgument"#;
        '\x7b'; b'}'; 'lifetime;
        "###,
        false,
    );
    assert_eq!(
        exact_identifier_count(&probe, "SetPathRealmEnvironmentArgument"),
        1
    );

    let declaration = normalized_rust(bounded(
        REALM_SOURCE,
        "use crate::emit::ObjectMutationErrorRealmSource;\n\n",
        "#[derive(Clone, Copy, Debug, PartialEq, Eq)]\npub(super) enum ObjectMutationErrorRealm",
    ));
    assert_eq!(
        declaration,
        "pub(super)enumSetPathRealmEnvironmentArgument{TrustedCurrentEnvironment,MainRealmFallback,}"
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_identifier_in_rust_sources(&source_root, "SetPathRealmEnvironmentArgument"),
        11,
        "one declaration, return type, import, two producer rows, four unit rows and two consumer rows own the authority"
    );
    let code = rust_code(REALM_SOURCE, false);
    for forbidden in [
        "impl Clone for SetPathRealmEnvironmentArgument",
        "impl Copy for SetPathRealmEnvironmentArgument",
        "impl Debug for SetPathRealmEnvironmentArgument",
        "impl Default for SetPathRealmEnvironmentArgument",
        "impl PartialEq for SetPathRealmEnvironmentArgument",
        "impl Eq for SetPathRealmEnvironmentArgument",
        "type SetPathRealmEnvironmentArgument =",
        "as SetPathRealmEnvironmentArgument",
    ] {
        assert!(!code.contains(forbidden), "found `{forbidden}`");
    }
}

#[test]
fn source_projection_and_unit_observations_are_exhaustive_and_exact() {
    let projection = normalized_rust(bounded(
        REALM_SOURCE,
        "pub(super) const fn set_path_realm_environment_argument(",
        "pub(super) const fn object_mutation_error_realm(",
    ));
    assert_eq!(
        projection,
        concat!(
            "source:ObjectMutationErrorRealmSource,)->SetPathRealmEnvironmentArgument{matchsource{",
            "ObjectMutationErrorRealmSource::GlobalFallback=>{SetPathRealmEnvironmentArgument::MainRealmFallback}",
            "ObjectMutationErrorRealmSource::StandardBuiltinEnvironment|",
            "ObjectMutationErrorRealmSource::SetPathHelperArgument=>{",
            "SetPathRealmEnvironmentArgument::TrustedCurrentEnvironment}}}"
        )
    );

    let unit = normalized_rust(bounded(
        REALM_SOURCE,
        "    fn object_mutation_realm_projection_excludes_ordinary_lexical_environments() {",
        "\n    }\n}",
    ));
    for row in [
        concat!(
            "matchset_path_realm_environment_argument(source){",
            "SetPathRealmEnvironmentArgument::TrustedCurrentEnvironment=>{}",
            "SetPathRealmEnvironmentArgument::MainRealmFallback=>{",
            "panic!(\"trusted mutation source lost its set-path Realm argument\")}}"
        ),
        concat!(
            "matchset_path_realm_environment_argument(ObjectMutationErrorRealmSource::GlobalFallback){",
            "SetPathRealmEnvironmentArgument::MainRealmFallback=>{}",
            "SetPathRealmEnvironmentArgument::TrustedCurrentEnvironment=>{",
            "panic!(\"global mutation fallback exposed a set-path Realm argument\")}}"
        ),
    ] {
        assert_eq!(unit.matches(row).count(), 1, "missing exact unit row `{row}`");
    }
    assert!(!unit.contains("assert_eq!(set_path_realm_environment_argument"));
}

#[test]
fn sole_product_consumer_emits_exactly_one_abi_argument() {
    let consumer = normalized_rust(bounded(
        OBJECTS_SOURCE,
        "    pub(crate) fn emit_set_path_realm_environment_argument(&self, function: &mut Function) {",
        "    /// The strictness flag word for an object write that is **not** a",
    ));
    assert_eq!(
        consumer,
        concat!(
            "matchset_path_realm_environment_argument(self.object_mutation_error_realm_source()){",
            "SetPathRealmEnvironmentArgument::TrustedCurrentEnvironment=>{",
            "function.instruction(&Instruction::LocalGet(self.current_env_local));}",
            "SetPathRealmEnvironmentArgument::MainRealmFallback=>{",
            "function.instruction(&Instruction::I64Const(0));}}}"
        )
    );
    for forbidden in [
        "_=>",
        "matches!(",
        "==",
        "!=",
        "&SetPathRealmEnvironmentArgument",
    ] {
        assert!(!consumer.contains(forbidden), "found `{forbidden}`");
    }

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_identifier_in_rust_sources(&source_root, "set_path_realm_environment_argument"),
        5,
        "one definition, one import, two unit calls and one product call own the projection route"
    );
}

#[test]
fn contract_and_task_record_source_equivalence_and_deferred_conformance() {
    for phrase in [
        "SetPathRealmEnvironmentArgument",
        "11 identifier mentions",
        "exactly one helper ABI argument",
        "Test262 remains deferred",
    ] {
        assert!(CONTRACT.contains(phrase), "contract missing `{phrase}`");
    }
    assert!(TASK.contains("set-path Realm environment argument"));
    assert!(TASK.contains("11-mention"));
    assert!(TASK.contains("Test262 remains deferred"));
}
