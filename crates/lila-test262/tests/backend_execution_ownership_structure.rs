const OWNER_SOURCE: &str = include_str!("../src/differential.rs");
const WORKER_SOURCE: &str = include_str!("../src/differential/worker.rs");
const PROCESS_SOURCE: &str = include_str!("../src/differential/worker_process.rs");
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

fn production_source() -> &'static str {
    OWNER_SOURCE
        .split_once("#[cfg(test)]\nfn case_fingerprint(")
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

    let declaration = normalize_rust(bounded_inclusive(
        production_source(),
        "#[cfg(any(test, feature = \"spec-exec-oracle\"))]\n#[derive(Debug)]\nstruct BackendExecution {",
        "#[cfg(any(test, feature = \"spec-exec-oracle\"))]\nconst fn execution_failure_phase(",
    ));
    assert_eq!(
        declaration.code,
        concat!(
            "#[cfg(any(test,feature=\"spec-exec-oracle\"))]",
            "#[derive(Debug)]structBackendExecution{backend:DifferentialBackend,",
            "output_events:OutputEventsObservation,result:BackendExecutionResult,}",
            "#[cfg(any(test,feature=\"spec-exec-oracle\"))]",
            "#[derive(Debug)]enumBackendExecutionResult{Completion{",
            "completion:ObservedCompletion,backend_note:String,},RootedCompletionGraph{",
            "completion:SnapshotCompletion,backend_note:String,},EngineFailure{",
            "phase:FailurePhase,message:String,},}",
        )
    );
    let source = normalize_rust(&format!(
        "{}\n{}\n{}",
        production_source(),
        WORKER_SOURCE.split("#[cfg(test)]").next().unwrap(),
        PROCESS_SOURCE.split("#[cfg(test)]").next().unwrap(),
    ));
    for authority in [
        "BackendExecution",
        "BackendExecutionResult",
        "CompletedWorkerAttempt",
    ] {
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
    let completed = normalize_rust(bounded_inclusive(
        PROCESS_SOURCE,
        "#[cfg(feature = \"spec-exec-oracle\")]\npub(super) struct CompletedWorkerAttempt",
        "#[cfg(feature = \"spec-exec-oracle\")]\nstruct Stage",
    ));
    assert_eq!(
        completed.code,
        concat!(
            "#[cfg(feature=\"spec-exec-oracle\")]",
            "pub(super)structCompletedWorkerAttempt{observation:BackendObservation,}",
            "#[cfg(feature=\"spec-exec-oracle\")]implCompletedWorkerAttempt{",
            "pub(super)fninto_observation(self)->BackendObservation{self.observation}}",
        )
    );
    assert!(!OWNER_SOURCE.contains("fn execute_case("));
}

#[test]
fn replay_constructs_wasm_then_spec_exec_and_moves_both_to_comparison() {
    let replay = normalize_rust(bounded_inclusive(
        production_source(),
        "#[cfg(feature = \"spec-exec-oracle\")]\npub fn replay_case(",
        "#[cfg(not(feature = \"spec-exec-oracle\"))]",
    ));
    assert_eq!(replay.code.matches("runner.run(").count(), 2);
    assert_eq!(replay.code.matches(".into_observation()").count(), 2);
    assert_eq!(replay.code.matches("compare_observations(").count(), 1);
    positions_in_order(
        &replay.code,
        &[
            "letwasm=runner.run(input,DifferentialBackend::WasmAot,oracle)?;",
            "letspec=runner.run(input,DifferentialBackend::SpecExec,oracle)?;",
            "compare_observations(input,wasm.into_observation(),spec.into_observation(),)",
        ],
    );
    assert!(!replay.code.contains("execute_case("));
    assert!(!replay.code.contains("Engine::"));
}

#[test]
fn execution_producer_populates_one_complete_envelope_in_the_child() {
    let producer = normalize_rust(
        WORKER_SOURCE
            .split_once("fn execute_case(")
            .expect("sole child execution producer")
            .1
            .split("#[cfg(test)]")
            .next()
            .unwrap(),
    );
    assert_eq!(
        producer
            .code
            .matches("BackendExecutionResult::Completion{")
            .count(),
        1
    );
    assert_eq!(
        producer.code.matches("BackendExecution{backend,").count(),
        1
    );
    assert_eq!(
        producer
            .code
            .matches("BackendExecutionResult::RootedCompletionGraph{")
            .count(),
        1
    );
    assert!(producer.code.contains("events!=journal.events"));
    assert!(producer
        .code
        .contains("backend_used!=backend.execution_backend()"));
    positions_in_order(
        &producer.code,
        &[
            "letordinary_outcome=|outcome:lila_engine::ObservedRunOutcome|",
            "BackendExecutionResult::Completion{",
            "DifferentialProgram::RootedSnapshot(program)",
            "case.snapshot_limits().expect(",
            "engine.observe_script_graph(program.source(),compile,run,limits)",
            "engine.observe_module_graph(program.source(),compile,run,limits)",
            "BackendExecutionResult::RootedCompletionGraph{",
            "letresult=matchoutcome{",
            "backend_used!=backend.execution_backend()||events!=journal.events",
            "Err(error)=>observe_engine_error(backend,&error)",
            "BackendExecution{backend,",
        ],
    );
    let worker = normalize_rust(bounded_inclusive(
        WORKER_SOURCE,
        "pub fn run_differential_worker(",
        "fn execute_case(",
    ));
    positions_in_order(
        &worker.code,
        &[
            "WorkerFrame::Header{",
            "native.admit()",
            "WorkerFrame::Admitted{",
            "execute_case(&case,request.binding.backend",
            "project_backend_execution(case.protocol(),execution)",
            "WorkerFrame::Terminal{",
        ],
    );
    assert_eq!(worker.code.matches("execute_case(").count(), 1);
    let engine_error = normalize_rust(bounded_inclusive(
        production_source(),
        "fn observe_engine_error(",
        "#[cfg(any(test, feature = \"spec-exec-oracle\"))]\nfn compare_observations(",
    ));
    assert_eq!(
        engine_error
            .code
            .matches("BackendExecutionResult::EngineFailure{")
            .count(),
        1
    );
    assert!(engine_error.code.ends_with(
        "BackendExecutionResult::EngineFailure{phase,message:error.message().to_string(),}}"
    ));
}

#[test]
fn comparison_borrows_completed_observations_and_worker_failure_wins() {
    let comparison = normalize_rust(bounded_inclusive(
        production_source(),
        "fn compare_observations(",
        "#[cfg(any(test, feature = \"spec-exec-oracle\"))]\nfn obeys_output_policy(",
    ));
    positions_in_order(
        &comparison.code,
        &[
            "&wasm_aot.output_events",
            "&spec_exec.output_events",
            "letwasm_disposition=wasm_aot.execution.disposition();",
            "letspec_disposition=spec_exec.execution.disposition();",
            "ExecutionObservation::WorkerFailure{..}",
            "DifferentialVerdict::WorkerFailure",
            "elseif!output_policy_satisfied",
            "DifferentialVerdict::ObservationContractViolated",
            "matches!(verdict,DifferentialVerdict::Mismatch).then",
            "DifferentialReport{",
            "wasm_aot,spec_exec,",
        ],
    );
    assert!(!comparison.code.contains("project_backend_execution("));
    // The original single transport now retires and cleans up before returning
    // its evidence to the protocol-specific supervisor/decoder.
    let retire = normalize_rust(bounded_inclusive(
        PROCESS_SOURCE,
        "let mut cleanup = live.retire();",
        "pub(super) fn run_robustness(",
    ));
    positions_in_order(
        &retire.code,
        &[
            "live.retire()",
            "drop(live)",
            "fs::File::open(&journal_path)",
            "stage.finish()",
            "Ok(TransportAttempt{",
        ],
    );
    assert!(!retire.code.contains("self.decode_journal("));
    let supervisor = normalize_rust(bounded_inclusive(
        PROCESS_SOURCE,
        "let transport = self.run_transport(input.timeout_ms().get()",
        "pub(super) fn run_transport(",
    ));
    positions_in_order(
        &supervisor.code,
        &[
            "self.run_transport(",
            "self.decode_journal(&transport.bytes,&binding,input)",
            "ifletSome(failure)=transport.failure",
            "ifletSome(message)=transport.cleanup",
            "JournalTail::Completed(execution)",
        ],
    );
    let decoder = normalize_rust(bounded_inclusive(
        PROCESS_SOURCE,
        "fn decode_journal(",
        "#[cfg(not(unix))]",
    ));
    assert!(decoder.code.contains(concat!(
        "ifprint_count==decoded.events.len()asu64",
        "&&valid_terminal(input.protocol(),&execution)",
        "&&rooted_snapshot::terminal_limits_match(input,&execution)=>{",
        "decoded.tail=JournalTail::Completed(execution);phase=JournalPhase::Terminal;}",
    )));
}

#[test]
fn projection_consumes_the_envelope_and_all_five_result_routes() {
    let projection = normalize_rust(bounded_inclusive(
        production_source(),
        "fn project_backend_execution(",
        "#[cfg(any(test, feature = \"spec-exec-oracle\"))]\nfn project_primitive_completion(",
    ));
    assert!(projection.code.starts_with(concat!(
        "fnproject_backend_execution(protocol:DifferentialProtocol,execution:BackendExecution,)",
        "->BackendObservation{letBackendExecution{backend,output_events,result,}=execution;"
    )));
    assert_eq!(
        projection
            .code
            .matches("BackendExecutionResult::Completion{")
            .count(),
        5
    );
    assert_eq!(
        projection
            .code
            .matches("BackendExecutionResult::EngineFailure{")
            .count(),
        2
    );
    assert_eq!(projection.code.matches("_=>").count(), 0);
    // The historical five-arm name is retained; V5 and V7 add four closed
    // routes, including explicit refusal in both wrong-result directions.
    assert_eq!(
        projection
            .code
            .matches("BackendExecutionResult::RootedCompletionGraph{")
            .count(),
        2
    );
    assert_eq!(projection.code.matches(")=>").count(), 9);
    assert!(projection.code.contains(concat!(
        "DifferentialProtocol::V7Test262HostRootedCompletionGraphPrintTranscript,",
        "BackendExecutionResult::RootedCompletionGraph{completion,backend_note,},",
        ")=>ExecutionObservation::RootedCompletionGraph{completion,backend_note,}",
    )));
    assert!(projection.code.contains(concat!(
        "DifferentialProtocol::V7Test262HostRootedCompletionGraphPrintTranscript,",
        "BackendExecutionResult::Completion{..},)=>ExecutionObservation::EngineFailure{",
        "phase:FailurePhase::RunnerInvariant,",
    )));
    assert!(projection.code.contains(concat!(
        "BackendExecutionResult::RootedCompletionGraph{..},",
        ")=>ExecutionObservation::ObservationRejected{",
    )));
    assert!(projection
        .code
        .ends_with("BackendObservation{worker_identity:None,backend,output_events,execution,}}"));
    let worker = normalize_rust(WORKER_SOURCE.split("#[cfg(test)]").next().unwrap());
    assert_eq!(worker.code.matches("project_backend_execution(").count(), 1);
    assert!(!PROCESS_SOURCE.contains("project_backend_execution("));
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
            "missing historical contract marker: {marker}"
        );
        assert!(
            task_words.contains(marker),
            "missing historical T25 marker: {marker}"
        );
    }
}
