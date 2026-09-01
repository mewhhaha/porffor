use std::fs;
use std::path::Path;

const SOURCE: &str = include_str!("../src/early_error_code.rs");
const CONTRACT: &str = include_str!("../../../docs/rust-rewrite/contracts/early-error-taxonomy.md");
const TASK: &str = include_str!("../../../tasks/07-parser-grammar-early-errors.md");

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

fn exact_route_count(source: &str, route: &str) -> usize {
    source
        .match_indices(route)
        .filter(|(offset, _)| {
            let before = source[..*offset].chars().next_back();
            let after = source[*offset + route.len()..].chars().next();
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

fn normalized_consumer(start: &str, end: &str) -> NormalizedRust {
    normalize_rust(bounded_inclusive(SOURCE, start, end))
}

#[test]
fn parse_failure_pattern_is_the_exact_intentional_copy_table_domain() {
    let lexical_probe = r###"
        ParseFailurePattern /* nested /* ignored */ comment */ :: r#StartsWith;
        // ParseFailurePattern::Exact
        "ParseFailurePattern::ContainsAll"; b"ParseFailurePattern";
        c"ParseFailurePattern"; r"ParseFailurePattern";
        br#"ParseFailurePattern"#; cr#"ParseFailurePattern"#;
        'P'; b'F'; 'lifetime;
    "###;
    let lexical_probe = normalize_rust(lexical_probe);
    assert_eq!(
        exact_identifier_count(&lexical_probe.identifiers, "ParseFailurePattern"),
        1
    );
    assert_eq!(
        exact_route_count(&lexical_probe.routes, "ParseFailurePattern::StartsWith"),
        1
    );

    let declaration = normalize_rust(bounded_inclusive(
        SOURCE,
        "const fn code_eq(a: EarlyErrorCode, b: EarlyErrorCode) -> bool {",
        "/// One `boa` static-semantics message shape",
    ));
    assert_eq!(
        declaration.code,
        concat!(
            "constfncode_eq(a:EarlyErrorCode,b:EarlyErrorCode)->bool{aasu8==basu8}",
            "#[derive(Clone,Copy)]enumParseFailurePattern{",
            "ContainsAll(&'static[&'staticstr]),StartsWith(&'staticstr),Exact(&'staticstr),}"
        )
    );
    let rule_declaration = normalize_rust(bounded_inclusive(
        SOURCE,
        "struct ParseFailureRule {",
        "/// The row count, in the type.",
    ));
    assert_eq!(
        rule_declaration.code,
        concat!(
            "structParseFailureRule{pattern:ParseFailurePattern,",
            "code:EarlyErrorCode,witnesses:&'static[&'staticstr],}"
        )
    );
    assert_eq!(
        exact_identifier_count(&rule_declaration.identifiers, "pattern"),
        1,
        "the rule product must declare exactly one pattern field"
    );
    assert_eq!(
        exact_identifier_count(&normalize_rust(SOURCE).identifiers, "pattern"),
        80,
        "one field, 73 initializers and six reads own every pattern identifier"
    );
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_identifier_in_rust_sources(&source_root, "ParseFailurePattern"),
        91
    );
    for forbidden in ["Debug", "PartialEq", "Eq", "Default", "Hash"] {
        assert!(!declaration.code.contains(forbidden));
    }
}

#[test]
fn parse_failure_pattern_table_has_the_exact_three_producer_populations() {
    let table = normalize_rust(bounded_inclusive(
        SOURCE,
        "const PARSE_FAILURE_RULE_TABLE: [ParseFailureRule; PARSE_FAILURE_RULE_COUNT] = [",
        "/// Slice view of [`PARSE_FAILURE_RULE_TABLE`]",
    ));
    assert_eq!(
        exact_route_count(&table.routes, "ParseFailurePattern::ContainsAll"),
        54
    );
    assert_eq!(
        exact_route_count(&table.routes, "ParseFailurePattern::StartsWith"),
        18
    );
    assert_eq!(
        exact_route_count(&table.routes, "ParseFailurePattern::Exact"),
        1
    );
    assert_eq!(
        exact_identifier_count(&table.identifiers, "ParseFailurePattern"),
        73
    );
    assert_eq!(
        exact_identifier_count(&table.identifiers, "pattern"),
        73,
        "every table row must initialize the pattern field exactly once"
    );
}

#[test]
fn all_six_parse_failure_pattern_consumers_are_exhaustive() {
    let consumers = [
        (
            "const fn rule_matches(",
            "/// The failure-detail token",
            r#"
                const fn rule_matches(rule: &ParseFailureRule, message: &str) -> bool {
                    match rule.pattern {
                        ParseFailurePattern::ContainsAll(fragments) => {
                            let mut i = 0;
                            while i < fragments.len() {
                                if !contains_sub(message, fragments[i]) {
                                    return false;
                                }
                                i += 1;
                            }
                            true
                        }
                        ParseFailurePattern::StartsWith(prefix) => starts_with_sub(message, prefix),
                        ParseFailurePattern::Exact(expected) => str_eq(message, expected),
                    }
                }
            "#,
        ),
        (
            "const fn code_is_owned_only_by_starts_with(",
            "/// True only when exactly one row owns `code`",
            r#"
                const fn code_is_owned_only_by_starts_with(code: EarlyErrorCode) -> bool {
                    use ParseFailurePattern::{ContainsAll, Exact, StartsWith};

                    let mut found = false;
                    let mut i = 0;
                    while i < PARSE_FAILURE_RULES.len() {
                        let rule = &PARSE_FAILURE_RULES[i];
                        if code_eq(rule.code, code) {
                            found = true;
                            match rule.pattern {
                                ContainsAll(_) => return false,
                                StartsWith(_) => {}
                                Exact(_) => return false,
                            }
                        }
                        i += 1;
                    }
                    found
                }
            "#,
        ),
        (
            "const fn code_is_owned_once_by_exact_starts_with(",
            "/// True only when exactly one row owns `code` and that row requires byte-for-",
            r#"
                const fn code_is_owned_once_by_exact_starts_with(
                    code: EarlyErrorCode,
                    expected_prefix: &str,
                ) -> bool {
                    let mut owners = 0;
                    let mut i = 0;
                    while i < PARSE_FAILURE_RULES.len() {
                        let rule = &PARSE_FAILURE_RULES[i];
                        if code_eq(rule.code, code) {
                            match rule.pattern {
                                ParseFailurePattern::StartsWith(prefix) => {
                                    if !str_eq(prefix, expected_prefix) {
                                        return false;
                                    }
                                    owners += 1;
                                }
                                ParseFailurePattern::ContainsAll(_) | ParseFailurePattern::Exact(_) => {
                                    return false;
                                }
                            }
                        }
                        i += 1;
                    }
                    owners == 1
                }
            "#,
        ),
        (
            "const fn code_is_owned_once_by_exact_message(",
            "/// True only when exactly two independently spelled anchored rows own `code`",
            r#"
                const fn code_is_owned_once_by_exact_message(
                    code: EarlyErrorCode,
                    expected_message: &str
                ) -> bool {
                    let mut owners = 0;
                    let mut i = 0;
                    while i < PARSE_FAILURE_RULES.len() {
                        let rule = &PARSE_FAILURE_RULES[i];
                        if code_eq(rule.code, code) {
                            match rule.pattern {
                                ParseFailurePattern::Exact(message) => {
                                    if !str_eq(message, expected_message) {
                                        return false;
                                    }
                                    owners += 1;
                                }
                                ParseFailurePattern::ContainsAll(_) | ParseFailurePattern::StartsWith(_) => {
                                    return false;
                                }
                            }
                        }
                        i += 1;
                    }
                    owners == 1
                }
            "#,
        ),
        (
            "const fn code_is_owned_twice_by_exact_starts_with(",
            "// These conditions are intentionally parse-owned.",
            r#"
                const fn code_is_owned_twice_by_exact_starts_with(
                    code: EarlyErrorCode,
                    first_prefix: &str,
                    second_prefix: &str,
                ) -> bool {
                    if str_eq(first_prefix, second_prefix) {
                        return false;
                    }

                    let mut owners = 0;
                    let mut first_owners = 0;
                    let mut second_owners = 0;
                    let mut i = 0;
                    while i < PARSE_FAILURE_RULES.len() {
                        let rule = &PARSE_FAILURE_RULES[i];
                        if code_eq(rule.code, code) {
                            owners += 1;
                            match rule.pattern {
                                ParseFailurePattern::StartsWith(prefix) => {
                                    if str_eq(prefix, first_prefix) {
                                        first_owners += 1;
                                    } else if str_eq(prefix, second_prefix) {
                                        second_owners += 1;
                                    } else {
                                        return false;
                                    }
                                }
                                ParseFailurePattern::ContainsAll(_) | ParseFailurePattern::Exact(_) => {
                                    return false;
                                }
                            }
                        }
                        i += 1;
                    }
                    owners == 2 && first_owners == 1 && second_owners == 1
                }
            "#,
        ),
        (
            "const fn every_row_is_populated(",
            "/// P2: every witness selects exactly one row",
            r#"
                const fn every_row_is_populated() -> bool {
                    let mut i = 0;
                    while i < PARSE_FAILURE_RULES.len() {
                        let rule = &PARSE_FAILURE_RULES[i];
                        if rule.witnesses.is_empty() {
                            return false;
                        }
                        match rule.pattern {
                            ParseFailurePattern::ContainsAll(fragments) => {
                                if fragments.is_empty() {
                                    return false;
                                }
                                let mut f = 0;
                                while f < fragments.len() {
                                    if fragments[f].is_empty() {
                                        return false;
                                    }
                                    f += 1;
                                }
                            }
                            ParseFailurePattern::StartsWith(prefix) => {
                                if prefix.is_empty() {
                                    return false;
                                }
                            }
                            ParseFailurePattern::Exact(message) => {
                                if message.is_empty() {
                                    return false;
                                }
                            }
                        }
                        let mut w = 0;
                        while w < rule.witnesses.len() {
                            if rule.witnesses[w].is_empty() {
                                return false;
                            }
                            w += 1;
                        }
                        i += 1;
                    }
                    true
                }
            "#,
        ),
    ];

    let mut observation_routes = String::new();
    let mut observation_identifiers = String::new();
    for (start, end, expected) in consumers {
        let actual = normalized_consumer(start, end);
        assert_eq!(actual.code, normalize_rust(expected).code, "{start}");
        assert!(!actual.code.contains("_=>"), "wildcard arm in {start}");
        assert!(
            !actual.code.contains("matches!("),
            "matches! observer in {start}"
        );
        observation_routes.push_str(&actual.routes);
        observation_identifiers.push_str(&actual.identifiers);
        observation_identifiers.push(' ');
    }
    assert_eq!(
        exact_route_count(&observation_routes, "ParseFailurePattern::ContainsAll"),
        5
    );
    assert_eq!(
        exact_route_count(&observation_routes, "ParseFailurePattern::StartsWith"),
        5
    );
    assert_eq!(
        exact_route_count(&observation_routes, "ParseFailurePattern::Exact"),
        5
    );
    assert_eq!(
        exact_identifier_count(&observation_identifiers, "ParseFailurePattern"),
        16
    );
    assert_eq!(
        exact_identifier_count(&observation_identifiers, "pattern"),
        6,
        "each exact consumer must read the pattern field exactly once"
    );
}

#[test]
fn contract_and_t07_record_intentional_copy_and_exhaustive_observation() {
    let contract_words = CONTRACT.split_whitespace().collect::<Vec<_>>().join(" ");
    let task_words = TASK.split_whitespace().collect::<Vec<_>>().join(" ");
    for marker in [
        "ParseFailurePattern",
        "intentional static-table value semantics",
        "54/18/1",
        "six exhaustive consumers",
        "91 lexical mentions",
        "no wildcard or `matches!` observer",
    ] {
        assert!(
            contract_words.contains(marker),
            "missing contract marker: {marker}"
        );
        assert!(task_words.contains(marker), "missing T07 marker: {marker}");
    }
}
