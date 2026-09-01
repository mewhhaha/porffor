const OWNER_SOURCE: &str = include_str!("../src/differential.rs");
const CONTRACT: &str = include_str!(
    "../../../docs/rust-rewrite/contracts/differential-backend-execution-ownership.md"
);
const TASK: &str = include_str!("../../../tasks/25-differential-fuzzing-performance.md");

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
        if !character.is_whitespace() {
            code.push(character);
            identifiers.push(character);
            routes.push(character);
        } else {
            identifiers.push(' ');
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

fn production_source() -> &'static str {
    OWNER_SOURCE
        .split_once("#[cfg(test)]\nmod tests {")
        .expect("differential test module boundary")
        .0
}

#[test]
fn backend_execution_is_one_debug_only_owned_authority() {
    let lexical_probe = r###"
        // BackendExecution::clone
        BackendExecution /* nested /* ignored */ comment */ :: r#clone;
        BackendExecutionResult::Completion;
        "BackendExecution"; b"BackendExecutionResult";
        c"BackendExecution"; r"BackendExecutionResult";
        br##"BackendExecution"##; cr#"BackendExecutionResult"#;
        'B'; b'B'; 'lifetime;
    "###;
    let lexical_probe = normalize_rust(lexical_probe);
    assert_eq!(
        exact_identifier_count(&lexical_probe.identifiers, "BackendExecution"),
        1
    );
    assert_eq!(
        exact_identifier_count(&lexical_probe.identifiers, "BackendExecutionResult"),
        1
    );
    assert_eq!(
        exact_identifier_count(&lexical_probe.routes, "BackendExecution::clone"),
        1
    );

    let source = production_source();
    let declaration_marker = concat!(
        "#[cfg(any(test, feature = \"spec-exec-oracle\"))]\n",
        "#[derive(Debug)]\nstruct BackendExecution {"
    );
    let declaration_offset = source
        .find(declaration_marker)
        .expect("BackendExecution declaration");
    let preceding_item_end = source[..declaration_offset]
        .rfind('}')
        .expect("preceding CapturingOutput implementation");
    let following_producer = source[declaration_offset..]
        .find("#[cfg(feature = \"spec-exec-oracle\")]\nfn execute_case(")
        .map(|offset| declaration_offset + offset)
        .expect("following execute_case producer");
    assert_eq!(
        normalize_rust(&source[preceding_item_end + 1..following_producer]).code,
        concat!(
            "#[cfg(any(test,feature=\"spec-exec-oracle\"))]",
            "#[derive(Debug)]structBackendExecution{backend:DifferentialBackend,",
            "output_events:OutputEventsObservation,result:BackendExecutionResult,}",
            "#[cfg(any(test,feature=\"spec-exec-oracle\"))]",
            "#[derive(Debug)]enumBackendExecutionResult{Completion{",
            "completion:ObservedCompletion,backend_note:String,},EngineFailure{",
            "phase:FailurePhase,message:String,},}",
            "#[cfg(any(test,feature=\"spec-exec-oracle\"))]",
            "implBackendExecutionResult{constfndisposition(&self)->ExecutionDisposition{",
            "matchself{Self::Completion{completion:ObservedCompletion::Normal(_),..}",
            "=>ExecutionDisposition::Normal,Self::Completion{",
            "completion:ObservedCompletion::Throw(_),..}|Self::EngineFailure{..}",
            "=>ExecutionDisposition::Error,}}}"
        ),
        "both private authorities must retain only their diagnostic Debug capability"
    );

    let source = normalize_rust(source);
    assert_eq!(
        exact_identifier_count(&source.identifiers, "BackendExecution"),
        7
    );
    assert_eq!(
        exact_identifier_count(&source.identifiers, "BackendExecutionResult"),
        12
    );
    assert_eq!(
        exact_identifier_count(&source.routes, "BackendExecutionResult::Completion"),
        4
    );
    assert_eq!(
        exact_identifier_count(&source.routes, "BackendExecutionResult::EngineFailure"),
        4
    );
    for authority in ["BackendExecution", "BackendExecutionResult"] {
        for capability in ["Clone", "Copy", "Default", "PartialEq", "Eq"] {
            assert!(!source
                .routes
                .contains(&format!("impl{capability}for{authority}")));
            assert!(!source
                .routes
                .contains(&format!("<{authority}as{capability}>")));
        }
        for forbidden in [
            format!("{authority}::clone"),
            format!("{authority}::eq"),
            format!("{authority}::ne"),
            format!("type{authority}"),
        ] {
            assert!(!source.routes.contains(&forbidden), "found `{forbidden}`");
        }
    }
}

#[test]
fn replay_constructs_wasm_then_spec_exec_and_moves_both_to_comparison() {
    let replay = normalize_rust(bounded_inclusive(
        production_source(),
        "#[cfg(feature = \"spec-exec-oracle\")]\npub fn replay_case(",
        "#[cfg(not(feature = \"spec-exec-oracle\"))]",
    ));
    assert_eq!(replay.code.matches("execute_case(").count(), 2);
    assert_eq!(replay.code.matches("compare_executions(").count(), 1);
    positions_in_order(
        &replay.code,
        &[
            "letwasm_aot=execute_case(case,DifferentialBackend::WasmAot);",
            "letspec_exec=execute_case(case,DifferentialBackend::SpecExec);",
            "Ok(compare_executions(case,wasm_aot,spec_exec))",
        ],
    );
    assert_eq!(
        (replay.code.len(), fnv1a(&replay.code)),
        (315, 0x4efc_7e19_c664_3719)
    );
}

#[test]
fn execution_producer_populates_one_complete_envelope() {
    let producer = normalize_rust(bounded_inclusive(
        production_source(),
        "#[cfg(feature = \"spec-exec-oracle\")]\nfn execute_case(",
        "#[cfg(feature = \"spec-exec-oracle\")]\nfn captured_output_events(",
    ));
    assert_eq!(
        producer
            .code
            .matches("BackendExecutionResult::Completion{")
            .count(),
        1
    );
    assert_eq!(
        producer
            .code
            .matches("BackendExecutionResult::EngineFailure{")
            .count(),
        1
    );
    assert_eq!(
        producer
            .code
            .matches("BackendExecution{backend,output_events,result,}")
            .count(),
        1
    );
    assert!(producer
        .code
        .ends_with("BackendExecution{backend,output_events,result,}}"));
    assert_eq!(
        (producer.code.len(), fnv1a(&producer.code)),
        (1470, 0xc6b4_d0f8_fd89_d1ac)
    );

    let engine_error = normalize_rust(bounded_inclusive(
        production_source(),
        "#[cfg(feature = \"spec-exec-oracle\")]\nfn observe_engine_error(",
        "#[cfg(any(test, feature = \"spec-exec-oracle\"))]\nfn compare_executions(",
    ));
    assert_eq!(
        engine_error
            .code
            .matches("BackendExecutionResult::EngineFailure{")
            .count(),
        1
    );
    assert!(engine_error.code.ends_with(concat!(
        "BackendExecutionResult::EngineFailure{phase,",
        "message:error.message().to_string(),}}"
    )));
    assert_eq!(
        (engine_error.code.len(), fnv1a(&engine_error.code)),
        (635, 0x7c25_999d_0eea_68f8)
    );
}

#[test]
fn comparison_borrows_both_envelopes_before_consuming_each_once() {
    let comparison = normalize_rust(bounded_inclusive(
        production_source(),
        "fn compare_executions(",
        "#[cfg(any(test, feature = \"spec-exec-oracle\"))]\nfn obeys_output_policy(",
    ));
    assert_eq!(
        exact_identifier_count(&comparison.identifiers, "wasm_execution"),
        4
    );
    assert_eq!(
        exact_identifier_count(&comparison.identifiers, "spec_execution"),
        4
    );
    positions_in_order(
        &comparison.code,
        &[
            "&wasm_execution.output_events",
            "&spec_execution.output_events",
            "letwasm_disposition=wasm_execution.result.disposition();",
            "letspec_disposition=spec_execution.result.disposition();",
            "letwasm_aot=project_backend_execution(protocol,wasm_execution);",
            "letspec_exec=project_backend_execution(protocol,spec_execution);",
        ],
    );
    let wasm_move_route = "project_backend_execution(protocol,wasm_execution)";
    let wasm_move = comparison.code.find(wasm_move_route).unwrap();
    let spec_move_route = "project_backend_execution(protocol,spec_execution)";
    let spec_move = comparison.code.find(spec_move_route).unwrap();
    assert!(!comparison.code[wasm_move + wasm_move_route.len()..].contains("wasm_execution"));
    assert!(!comparison.code[spec_move + spec_move_route.len()..].contains("spec_execution"));
    assert_eq!(
        (comparison.code.len(), fnv1a(&comparison.code)),
        (1980, 0x7ea7_39af_e205_62c2)
    );
}

#[test]
fn projection_consumes_the_envelope_and_all_five_result_routes() {
    let projection = normalize_rust(bounded_inclusive(
        production_source(),
        "fn project_backend_execution(",
        "#[cfg(any(test, feature = \"spec-exec-oracle\"))]\nfn project_primitive_completion(",
    ));
    assert!(projection.code.starts_with(concat!(
        "fnproject_backend_execution(protocol:DifferentialProtocol,",
        "execution:BackendExecution,)->BackendObservation{letBackendExecution{",
        "backend,output_events,result,}=execution;"
    )));
    assert_eq!(
        projection
            .code
            .matches("BackendExecutionResult::Completion{")
            .count(),
        3
    );
    assert_eq!(
        projection
            .code
            .matches("BackendExecutionResult::EngineFailure{")
            .count(),
        2
    );
    assert_eq!(projection.code.matches("_=>").count(), 0);
    assert!(projection
        .code
        .ends_with("BackendObservation{backend,output_events,execution,}}"));
    assert_eq!(
        (projection.code.len(), fnv1a(&projection.code)),
        (1265, 0xa6c8_e8d8_71be_286b)
    );

    let source = normalize_rust(production_source());
    assert_eq!(
        exact_identifier_count(&source.identifiers, "project_backend_execution"),
        3
    );
    assert_eq!(
        source.routes.matches(".project_backend_execution").count(),
        0
    );
    assert_eq!(
        source.routes.matches("::project_backend_execution").count(),
        0
    );
}

#[test]
fn contract_and_t25_record_the_owned_execution_lifecycle() {
    let contract_words = CONTRACT.split_whitespace().collect::<Vec<_>>().join(" ");
    let task_words = TASK.split_whitespace().collect::<Vec<_>>().join(" ");
    for marker in [
        "seven production mentions",
        "12 production result mentions",
        "Debug-only",
        "borrow-before-consume order",
        "five-arm consuming projection",
    ] {
        assert!(
            contract_words.contains(marker),
            "missing contract marker: {marker}"
        );
        assert!(task_words.contains(marker), "missing T25 marker: {marker}");
    }
}
