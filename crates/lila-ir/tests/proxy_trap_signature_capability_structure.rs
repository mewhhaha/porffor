use std::fs;
use std::path::Path;

const PROXY_TRAPS_SOURCE: &str = include_str!("../src/lowering/proxy_traps.rs");
const LOWERING_SOURCE: &str = include_str!("../src/lowering.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

fn trimmed_nonempty_lines(source: &str) -> Vec<&str> {
    source
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect()
}

fn count_in_rust_sources(root: &Path, needle: &str) -> usize {
    fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()))
        .map(|entry| {
            entry
                .expect("source directory entry should be readable")
                .path()
        })
        .map(|path| {
            if path.is_dir() {
                return count_in_rust_sources(&path, needle);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return 0;
            }
            fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
                .matches(needle)
                .count()
        })
        .sum()
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

fn lexically_normalized_code(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut normalized = String::new();
    let mut offset = 0;
    while offset < bytes.len() {
        if let Some(end) = literal_end(source, offset) {
            normalized.push('L');
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
            normalized.push(character);
        }
        offset += character.len_utf8();
    }
    normalized
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

#[test]
fn proxy_trap_signature_is_the_exact_private_no_capability_domain() {
    let declaration_region = PROXY_TRAPS_SOURCE
        .split_once("/// Declare every member of the fixed Proxy trap domain once.")
        .expect("signature declaration must precede the Proxy-trap macro")
        .0;
    assert!(declaration_region.ends_with("}\n\n"));
    assert!(
        !declaration_region.contains("#["),
        "attributes before the signature declaration can add capabilities"
    );

    let domain = bounded(
        declaration_region,
        "pub(super) enum ProxyTrapSignature {",
        "}\n\n",
    );
    assert_eq!(
        trimmed_nonempty_lines(domain),
        [
            "Target,",
            "TargetAndPropertyKey,",
            "TargetPropertyKeyReceiver,",
            "TargetPropertyKeyValueReceiver,",
            "TargetPropertyKeyDescriptor,",
            "TargetAndPrototype,",
            "TargetThisArguments,",
            "TargetArgumentsNewTarget,",
        ]
    );

    assert_eq!(PROXY_TRAPS_SOURCE.matches("ProxyTrapSignature").count(), 3);
    assert_eq!(LOWERING_SOURCE.matches("ProxyTrapSignature").count(), 9);
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "ProxyTrapSignature"),
        12,
        "the declaration, return type, macro projection, import and eight consumer arms own every mention"
    );
}

#[test]
fn all_thirteen_proxy_traps_map_to_their_exact_semantic_signature() {
    let signature_projection = bounded(
        PROXY_TRAPS_SOURCE,
        "            pub(super) const fn signature(self) -> ProxyTrapSignature {\n",
        "\n            /// The generic object-literal heuristic",
    );
    assert_eq!(
        trimmed_nonempty_lines(signature_projection),
        [
            "match self {",
            "$(Self::$variant => ProxyTrapSignature::$signature),+",
            "}",
            "}",
        ]
    );

    let rows = PROXY_TRAPS_SOURCE
        .split_once("proxy_traps! {\n")
        .expect("Proxy-trap table must exist")
        .1;
    assert_eq!(
        trimmed_nonempty_lines(rows),
        [
            "13;",
            "GetPrototypeOf => (\"getPrototypeOf\", Target, false),",
            "SetPrototypeOf => (\"setPrototypeOf\", TargetAndPrototype, false),",
            "IsExtensible => (\"isExtensible\", Target, false),",
            "PreventExtensions => (\"preventExtensions\", Target, false),",
            "GetOwnPropertyDescriptor => (\"getOwnPropertyDescriptor\", TargetAndPropertyKey, true),",
            "DefineProperty => (\"defineProperty\", TargetPropertyKeyDescriptor, true),",
            "Has => (\"has\", TargetAndPropertyKey, true),",
            "Get => (\"get\", TargetPropertyKeyReceiver, true),",
            "Set => (\"set\", TargetPropertyKeyValueReceiver, false),",
            "DeleteProperty => (\"deleteProperty\", TargetAndPropertyKey, true),",
            "OwnKeys => (\"ownKeys\", Target, false),",
            "Apply => (\"apply\", TargetThisArguments, false),",
            "Construct => (\"construct\", TargetArgumentsNewTarget, false),",
            "}",
        ]
    );
}

#[test]
fn proxy_trap_signature_projects_to_all_eight_ordered_argument_records() {
    let lexical_probe = r###"
        trap.signature /* nested /* route */ comment */ ()
        ProxyTrap:: /* stored route */ signature
        trap.r#signature()
        ProxyTrap::r#signature
        trap.signature::<>();
        let text = "trap.signature() ProxyTrap::signature";
        let raw = r#"trap.signature() ProxyTrap::signature"#;
        let byte = b"trap.signature()";
        let c_string = c"ProxyTrap::signature";
        let raw_c_string = cr#"trap.signature()"#;
        let raw_byte_string = br#"ProxyTrap::signature"#;
        let character = ':';
        let escaped_character = '\x3a';
        let byte_character = b'\x3a';
        let borrowed: &'a str = value;
    "###;
    assert_eq!(
        lexically_normalized_code(lexical_probe),
        concat!(
            "trap.signature()ProxyTrap::signature",
            "trap.signature()ProxyTrap::signature",
            "trap.signature::<>();",
            "lettext=L;letraw=L;letbyte=L;letc_string=L;",
            "letraw_c_string=L;letraw_byte_string=L;",
            "letcharacter=L;letescaped_character=L;letbyte_character=L;",
            "letborrowed:&'astr=value;"
        )
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_normalized_rust_sources(&source_root, ".signature"),
        1,
        "trap.signature() must be the sole method-style signature route"
    );
    assert_eq!(
        count_in_normalized_rust_sources(&source_root, "::signature"),
        0,
        "ProxyTrap::signature must not gain a UFCS call or stored method-item route"
    );

    let consumer = bounded(
        LOWERING_SOURCE,
        "    fn proxy_trap_argument_infos(trap: ProxyTrap, target: ValueInfo) -> Vec<ValueInfo> {",
        "\n    fn merge_proxy_trap_signature_hint(",
    );
    assert_eq!(consumer.matches("match trap.signature() {").count(), 1);
    let projection = consumer
        .split_once("        match trap.signature() {\n")
        .expect("signature must be consumed by one exhaustive match")
        .1;
    assert_eq!(
        trimmed_nonempty_lines(projection),
        [
            "ProxyTrapSignature::Target => vec![target],",
            "ProxyTrapSignature::TargetAndPropertyKey => vec![target, property_key],",
            "ProxyTrapSignature::TargetPropertyKeyReceiver => {",
            "vec![target, property_key, any]",
            "}",
            "ProxyTrapSignature::TargetPropertyKeyValueReceiver => {",
            "vec![target, property_key, any.clone(), any]",
            "}",
            "ProxyTrapSignature::TargetPropertyKeyDescriptor => {",
            "vec![target, property_key, descriptor]",
            "}",
            "ProxyTrapSignature::TargetAndPrototype => vec![target, prototype],",
            "ProxyTrapSignature::TargetThisArguments => vec![target, any, arguments_list],",
            "ProxyTrapSignature::TargetArgumentsNewTarget => {",
            "vec![target, arguments_list, new_target]",
            "}",
            "}",
            "}",
        ]
    );
    assert!(!projection.contains("_ =>"));
}
