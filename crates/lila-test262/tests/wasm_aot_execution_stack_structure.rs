use std::fs;
use std::path::Path;

const OWNER_SOURCE: &str = include_str!("../src/lib.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/test262-wasm-aot-execution-stack.md");
const TASK: &str = include_str!("../../../tasks/03-conformance-harness-integrity.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
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
        .map(|entry| entry.expect("failed to read Rust source entry").path())
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
        .map(|entry| entry.expect("failed to read Rust source entry").path())
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
fn wasm_aot_execution_stack_is_the_exact_private_no_capability_domain() {
    let declaration = bounded(
        OWNER_SOURCE,
        r#"fn run_one_case_on_persistent_worker(
    case: &TestCase,
    preludes: &PreludeStore,
    timeout_ms: u64,
    execution_backend: ExecutionBackend,
) -> TestResult {
    run_one_case_with_wasm_aot_execution(
        case,
        preludes,
        timeout_ms,
        execution_backend,
        WasmAotExecutionStack::PersistentTest262Worker,
    )
}

"#,
        "fn run_one_case_with_wasm_aot_execution(",
    );
    assert_eq!(
        declaration,
        concat!(
            "enum WasmAotExecutionStack {\n",
            "    #[cfg(test)]\n",
            "    DedicatedWorker,\n",
            "    PersistentTest262Worker,\n",
            "}\n\n",
        ),
        "the exact adjacent declaration must remain private, capability-free and cfg-correct"
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "WasmAotExecutionStack"),
        8,
        "one declaration, one parameter, two producers and four exhaustive arms own the authority"
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "WasmAotExecutionStack::DedicatedWorker"),
        3
    );
    assert_eq!(
        count_in_rust_sources(
            &source_root,
            "WasmAotExecutionStack::PersistentTest262Worker"
        ),
        3
    );
    for forbidden in [
        "pub enum WasmAotExecutionStack",
        "pub(crate) enum WasmAotExecutionStack",
        "impl WasmAotExecutionStack",
        "for WasmAotExecutionStack",
        "matches!(wasm_aot_execution_stack",
        "wasm_aot_execution_stack ==",
        "wasm_aot_execution_stack !=",
    ] {
        assert!(!OWNER_SOURCE.contains(forbidden), "found `{forbidden}`");
    }
}

#[test]
fn both_execution_entry_points_produce_their_exact_stack_authority() {
    let lexical_probe = r###"
        run_one_case_on_persistent_worker /* nested /* route */ comment */ (
            case,
        );
        r#run_one_case_on_persistent_worker();
        run_one_case_with_wasm_aot_execution
            /* shared route */
            (case);
        r#run_one_case_with_wasm_aot_execution();
        let ordinary = "run_one_case_on_persistent_worker(";
        let raw = r#"run_one_case_with_wasm_aot_execution("#;
        let byte = b"run_one_case_on_persistent_worker(";
        let raw_byte = br#"run_one_case_with_wasm_aot_execution("#;
        let c_string = c"run_one_case_on_persistent_worker(";
        let raw_c_string = cr#"run_one_case_with_wasm_aot_execution("#;
        let character = '(';
        let byte_character = b'(';
        let borrowed: &'a str = value;
    "###;
    let normalized_probe = lexically_normalized_code(lexical_probe);
    assert_eq!(
        normalized_probe
            .matches("run_one_case_on_persistent_worker(")
            .count(),
        2
    );
    assert_eq!(
        normalized_probe
            .matches("run_one_case_with_wasm_aot_execution(")
            .count(),
        2
    );

    let run_case_entry = bounded(
        OWNER_SOURCE,
        "fn run_case_entry(",
        "#[cfg(test)]\nfn run_one_case(",
    );
    let product_call = bounded(
        run_case_entry,
        "    panic::catch_unwind(AssertUnwindSafe(|| {\n",
        "    }))\n    .unwrap_or_else",
    );
    assert_eq!(
        product_call,
        concat!(
            "        run_one_case_on_persistent_worker(\n",
            "            case,\n",
            "            preludes,\n",
            "            config.timeout_ms,\n",
            "            run_config.execution_backend,\n",
            "        )\n",
        ),
        "the product catch boundary must forward the admitted case through the persistent worker"
    );

    let dedicated = bounded(
        OWNER_SOURCE,
        "#[cfg(test)]\nfn run_one_case(",
        "/// Runs one case on an `execute_cases` worker",
    );
    assert_eq!(
        trimmed_nonempty_lines(dedicated),
        trimmed_nonempty_lines(
            r#"
    case: &TestCase,
    preludes: &PreludeStore,
    timeout_ms: u64,
    execution_backend: ExecutionBackend,
) -> TestResult {
    run_one_case_with_wasm_aot_execution(
        case,
        preludes,
        timeout_ms,
        execution_backend,
        WasmAotExecutionStack::DedicatedWorker,
    )
}

"#,
        )
    );

    let persistent = bounded(
        OWNER_SOURCE,
        "fn run_one_case_on_persistent_worker(",
        "enum WasmAotExecutionStack {",
    );
    assert_eq!(
        trimmed_nonempty_lines(persistent),
        trimmed_nonempty_lines(
            r#"
    case: &TestCase,
    preludes: &PreludeStore,
    timeout_ms: u64,
    execution_backend: ExecutionBackend,
) -> TestResult {
    run_one_case_with_wasm_aot_execution(
        case,
        preludes,
        timeout_ms,
        execution_backend,
        WasmAotExecutionStack::PersistentTest262Worker,
    )
}

"#,
        )
    );

    let consumer_header = bounded(
        OWNER_SOURCE,
        "fn run_one_case_with_wasm_aot_execution(",
        ") -> TestResult {",
    );
    assert_eq!(
        trimmed_nonempty_lines(consumer_header),
        trimmed_nonempty_lines(
            r#"
    case: &TestCase,
    preludes: &PreludeStore,
    timeout_ms: u64,
    execution_backend: ExecutionBackend,
    wasm_aot_execution_stack: WasmAotExecutionStack,
"#,
        )
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_normalized_rust_sources(&source_root, "run_one_case_on_persistent_worker("),
        2,
        "one product call and one definition own the persistent-worker wrapper"
    );
    assert_eq!(
        count_in_normalized_rust_sources(&source_root, "run_one_case_with_wasm_aot_execution("),
        3,
        "the two typed producers and one definition own the shared runner"
    );
}

#[test]
fn execution_stack_is_borrowed_exhaustively_before_exact_ordered_engine_routes() {
    let routing = bounded(
        OWNER_SOURCE,
        "        let run_result = ",
        "\n\n        if let Some(negative) = &case.negative {",
    );
    let expected = r#"if execution_backend == ExecutionBackend::WasmAot
            && !materialized.execution_mode().is_module()
            && agent_prelude.is_some()
            && (match &wasm_aot_execution_stack {
                #[cfg(test)]
                WasmAotExecutionStack::DedicatedWorker => false,
                WasmAotExecutionStack::PersistentTest262Worker => true,
            }) {
            engine.run_wasm_aot_script_with_agents_on_current_thread(
                &materialized.source,
                compile_options,
                run_options.timeout_ms,
                run_options.can_block,
                agent_prelude
                    .clone()
                    .expect("agent prelude was checked above"),
            )
        } else if execution_backend == ExecutionBackend::WasmAot
            && !materialized.execution_mode().is_module()
            && agent_prelude.is_some()
        {
            engine.run_wasm_aot_script_with_agents(
                &materialized.source,
                compile_options,
                run_options.timeout_ms,
                run_options.can_block,
                agent_prelude.expect("agent prelude was checked above"),
            )
        } else if execution_backend == ExecutionBackend::WasmAot
            && (match &wasm_aot_execution_stack {
                #[cfg(test)]
                WasmAotExecutionStack::DedicatedWorker => false,
                WasmAotExecutionStack::PersistentTest262Worker => true,
            })
        {
            if materialized.execution_mode().is_module() {
                engine.run_wasm_aot_module_on_current_thread(
                    &materialized.source,
                    compile_options,
                    run_options.timeout_ms,
                    run_options.can_block,
                )
            } else {
                engine.run_wasm_aot_script_on_current_thread(
                    &materialized.source,
                    compile_options,
                    run_options.timeout_ms,
                    run_options.can_block,
                )
            }
        } else if materialized.execution_mode().is_module() {
            engine.run_module(&materialized.source, compile_options, run_options)
        } else {
            engine.run_script(&materialized.source, compile_options, run_options)
        };"#;
    assert_eq!(
        trimmed_nonempty_lines(routing),
        trimmed_nonempty_lines(expected)
    );
    assert_eq!(
        routing.matches("match &wasm_aot_execution_stack {").count(),
        2
    );
    assert!(!routing.contains("_ =>"));
}

#[test]
fn contract_and_t03_record_the_execution_stack_boundary() {
    for evidence in [
        "WasmAotExecutionStack::{DedicatedWorker, PersistentTest262Worker}",
        "execute_cases_runs_wasm_aot_cases_on_persistent_workers",
        "wasm_aot_enforces_async_done_output_after_jobs_drain",
    ] {
        assert!(CONTRACT.contains(evidence), "contract missing `{evidence}`");
        assert!(TASK.contains(evidence), "T03 missing `{evidence}`");
    }
}
