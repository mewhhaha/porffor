use std::fs;
use std::path::Path;

const ENGINE_SOURCE: &str = include_str!("../src/lib.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/wasm-top-level-completion-kind.md");
const TASK: &str = include_str!("../../../tasks/04-spec-operations-and-completion-abi.md");

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

fn rust_code(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut code = String::new();
    let mut offset = 0;
    while offset < bytes.len() {
        if let Some(end) = literal_end(source, offset) {
            code.push(' ');
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
            code.push(' ');
        } else {
            code.push(character);
        }
        offset += character.len_utf8();
    }
    code
}

fn compact(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
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
            exact_identifier_count(&rust_code(&source), identifier)
        })
        .sum()
}

#[test]
fn top_level_completion_kind_is_the_exact_private_no_capability_domain() {
    let lexical_probe = rust_code(
        r###"
        WasmTopLevelCompletionKind::Normal;
        // WasmTopLevelCompletionKind::Throw
        /* WasmTopLevelCompletionKind /* nested */ :: Throw */
        "WasmTopLevelCompletionKind"; b"WasmTopLevelCompletionKind";
        c"WasmTopLevelCompletionKind"; r"WasmTopLevelCompletionKind";
        br#"WasmTopLevelCompletionKind"#; cr#"WasmTopLevelCompletionKind"#;
        'W'; b'T'; 'lifetime;
        "###,
    );
    assert_eq!(
        exact_identifier_count(&lexical_probe, "WasmTopLevelCompletionKind"),
        1
    );

    let declaration = bounded(
        ENGINE_SOURCE,
        "enum WasmTopLevelCompletionKind {",
        "enum WasmtimeExportedMemory {",
    );
    assert_eq!(compact(&rust_code(declaration)), "Normal,Throw,}");
    assert_eq!(
        ENGINE_SOURCE
            .matches("\n\nenum WasmTopLevelCompletionKind {")
            .count(),
        1
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_identifier_in_rust_sources(&source_root, "WasmTopLevelCompletionKind"),
        9
    );
    for capability in ["Clone", "Copy", "Debug", "Default", "PartialEq", "Eq"] {
        assert!(
            !ENGINE_SOURCE.contains(&format!("impl {capability} for WasmTopLevelCompletionKind"))
        );
    }
}

#[test]
fn raw_completion_kind_is_parsed_once_before_three_exhaustive_consumers() {
    let execution = compact(&rust_code(bounded(
        ENGINE_SOURCE,
        "fn execute_with_wasm_bytes_inner_with_agents(",
        "enum WasmTopLevelCompletionKind {",
    )));
    assert_eq!(
        execution
            .matches("matchi64::from(completion_kind){")
            .count(),
        1
    );
    assert_eq!(
        execution
            .matches("kindifkind==CompletionKindIr::Normal.abi_code()=>{")
            .count(),
        1
    );
    assert_eq!(
        execution
            .matches(
                "kindifkind==CompletionKindIr::Throw.abi_code()=>WasmTopLevelCompletionKind::Throw"
            )
            .count(),
        1
    );
    assert_eq!(execution.matches("match&completion_kind{").count(), 2);
    assert_eq!(execution.matches("matchcompletion_kind{").count(), 1);
    assert_eq!(
        execution
            .matches("WasmTopLevelCompletionKind::Normal")
            .count(),
        4
    );
    assert_eq!(
        execution
            .matches("WasmTopLevelCompletionKind::Throw")
            .count(),
        4
    );
    for forbidden in ["is_throw", "=>true", "=>false", "matches!(completion_kind"] {
        assert!(!execution.contains(forbidden), "found `{forbidden}`");
    }
}

#[test]
fn each_completion_consumer_owns_its_normal_and_throw_consequence() {
    let execution = compact(&rust_code(bounded(
        ENGINE_SOURCE,
        "fn execute_with_wasm_bytes_inner_with_agents(",
        "enum WasmTopLevelCompletionKind {",
    )));
    let legacy = bounded(
        &execution,
        "WasmExecutionMode::Legacy=>{",
        "WasmExecutionMode::Structured=>{",
    );
    assert_eq!(legacy.matches("match&completion_kind{").count(), 2);
    assert!(!legacy.contains("_=>"));
    for consequence in [
        "WasmTopLevelCompletionKind::Normal=>ThrownErrorText::NONE",
        "WasmTopLevelCompletionKind::Throw=>{ThrownErrorText::read(",
        "WasmTopLevelCompletionKind::Normal=>{Ok(WasmExecutionOutcome::Legacy(",
        "WasmTopLevelCompletionKind::Throw=>{letprefix=thrown_error.name_prefix();Err(",
    ] {
        assert_eq!(
            legacy.matches(consequence).count(),
            1,
            "missing `{consequence}`"
        );
    }

    let structured = execution
        .split_once("WasmExecutionMode::Structured=>{")
        .expect("structured execution arm")
        .1;
    assert_eq!(structured.matches("matchcompletion_kind{").count(), 1);
    assert!(!structured.contains("_=>"));
    assert_eq!(
        structured
            .matches("WasmTopLevelCompletionKind::Normal=>ObservedCompletion::Normal(value)")
            .count(),
        1
    );
    assert_eq!(
        structured
            .matches("WasmTopLevelCompletionKind::Throw=>ObservedCompletion::Throw(value)")
            .count(),
        1
    );

    for evidence in [CONTRACT, TASK] {
        let evidence = compact(evidence);
        assert!(evidence.contains("WasmTopLevelCompletionKind"));
        assert!(evidence.contains("threeexhaustiveconsumers"));
        assert!(evidence.contains("changesnocompletionABI"));
        assert!(evidence.contains("runtimebehavior"));
    }
}
