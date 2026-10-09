const ENGINE_SOURCE: &str = include_str!("../../src/lib.rs");
const GC_SOURCE: &str = include_str!("../../src/wasm_gc_completion.rs");
const GC_SCHEMA_SOURCE: &str = include_str!("../../../lila-aot-wasm/src/gc_types/host.rs");
const CONTRACT: &str =
    include_str!("../../../../docs/rust-rewrite/contracts/wasm-top-level-completion-kind.md");
const TASK: &str = include_str!("../../../../tasks/04-spec-operations-and-completion-abi.md");

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
        "fn wasm_memory_span(",
    );
    assert_eq!(compact(&rust_code(declaration)), "Normal,Throw,}");
    assert_eq!(
        ENGINE_SOURCE
            .matches("\n\nenum WasmTopLevelCompletionKind {")
            .count(),
        1
    );
    for capability in ["Clone", "Copy", "Debug", "Default", "PartialEq", "Eq"] {
        assert!(
            !ENGINE_SOURCE.contains(&format!("impl {capability} for WasmTopLevelCompletionKind"))
        );
    }
    let production = compact(&rust_code(GC_SOURCE.split("#[cfg(test)]").next().unwrap()));
    assert!(production.contains(concat!(
        "pub(super)structGcObservedCompletion{tag:WasmRuntimeValueTag,",
        "kind:WasmTopLevelCompletionKind,value:ObservedJsValue,}"
    )));
    assert_eq!(
        production.matches("Ok(GcObservedCompletion{tag:").count(),
        1,
        "only checked observation constructs the completed owner"
    );
    assert!(!production.contains("implCloneforGcObservedCompletion"));
    assert!(!production.contains("implCopyforGcObservedCompletion"));
}

#[test]
fn rooted_main_tuple_is_checked_once_before_three_exhaustive_consumers() {
    let schema = compact(&rust_code(bounded(
        GC_SCHEMA_SOURCE,
        "pub fn check_gc_main_completion(",
        "/// Native imports",
    )));
    assert!(schema.contains("iftarget!=0"));
    for branch in [
        "crate::COMPLETION_KIND_NORMAL=>Ok(lila_ir::CompletionKindIr::Normal)",
        "crate::COMPLETION_KIND_THROW=>Ok(lila_ir::CompletionKindIr::Throw)",
    ] {
        assert!(schema.contains(branch));
    }
    let observation = compact(&rust_code(bounded(
        GC_SOURCE,
        "pub(super) fn observe(",
        "fn field(",
    )));
    assert_eq!(
        observation
            .matches("check_gc_main_completion(kind,target)")
            .count(),
        1
    );
    assert_eq!(
        observation
            .matches("GcHostValue::check(tag,scalar,reference.is_some())")
            .count(),
        1
    );
    assert!(!observation.contains("_=>"));
    assert!(observation.contains("CompletionKindIr::Normal=>WasmTopLevelCompletionKind::Normal"));
    assert!(observation.contains("CompletionKindIr::Throw=>WasmTopLevelCompletionKind::Throw"));
    let gc = compact(&rust_code(GC_SOURCE));
    assert!(gc.contains("pub(super)typeGcMainCompletion=(i32,i64,Option<Rooted<EqRef>>,i32,i32);"));

    let execution = compact(&rust_code(bounded(
        ENGINE_SOURCE,
        "fn execute_with_wasm_bytes_inner_with_agents(",
        "enum WasmTopLevelCompletionKind {",
    )));
    let lookup = execution
        .find("get_typed_func::<(),GcMainCompletion>")
        .unwrap();
    let roots = execution
        .find("wasmtime::RootScope::new(&mutstore)")
        .unwrap();
    let call = execution.find("main.call(&mutroots,())").unwrap();
    let check = execution
        .find("wasm_gc_completion::observe(&mutroots,completion)?.into_parts()")
        .unwrap();
    assert!(lookup < roots && roots < call && call < check);
    assert_eq!(execution.matches("wasm_gc_completion::observe(").count(), 1);
    assert_eq!(execution.matches("match&completion_kind{").count(), 2);
    assert_eq!(execution.matches("matchcompletion_kind{").count(), 1);
    for forbidden in [
        "WASM_EXPORT_RESULT_TAG",
        "WASM_EXPORT_COMPLETION_KIND",
        "is_throw",
        "matches!(completion_kind",
    ] {
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
        .unwrap()
        .1;
    assert_eq!(structured.matches("matchcompletion_kind{").count(), 1);
    assert!(!structured.contains("_=>"));
    for consequence in [
        "WasmTopLevelCompletionKind::Normal=>ObservedCompletion::Normal(value)",
        "WasmTopLevelCompletionKind::Throw=>ObservedCompletion::Throw(value)",
    ] {
        assert_eq!(structured.matches(consequence).count(), 1);
    }
    for evidence in [CONTRACT, TASK] {
        assert!(evidence.contains("WasmTopLevelCompletionKind"));
        assert!(evidence.contains("three exhaustive consumers"));
        assert!(evidence.contains("RootScope"));
        assert!(evidence.contains("five-result Main"));
        assert!(evidence.contains("unverified"));
    }
}
