//! Paths and feature annotations identify executions; the compiler owns support.
use super::*;

fn case(path: &str, body: &str) -> TestCase {
    parse_test_case(
        path.into(),
        PathBuf::from(path),
        format!("/*---\nflags: [raw]\nfeatures: [SharedArrayBuffer, immutable-arraybuffer]\n---*/\n{body}"),
    )
}

#[test]
fn formerly_rejected_metadata_cannot_bypass_the_selected_case_worker() {
    for path in [
        "built-ins/Map/prototype/feature.js",
        "built-ins/Object/freeze/freeze-sharedarraybuffer.js",
        "built-ins/Proxy/apply/arguments-realm.js",
        "built-ins/Proxy/construct/arguments-realm.js",
        "built-ins/Proxy/construct/trap-is-undefined-proto-from-cross-realm-newtarget.js",
        "built-ins/Proxy/construct/trap-is-undefined-proto-from-newtarget-realm.js",
    ] {
        let cases = [case(path, "0;")];
        let mut config = SuiteConfig::default();
        let Err(error) = CaseExecutionDispatch::admit(&config, &cases) else {
            panic!("case metadata must not authorize a supervisor to bypass execution: {path}");
        };
        assert!(error.contains("requires a selected case worker executable"));
        let binary = PathBuf::from("selected-case-worker");
        config.case_runner_bin = Some(binary.clone());
        assert!(matches!(
            CaseExecutionDispatch::admit(&config, &cases).unwrap(),
            CaseExecutionDispatch::Supervised(selected) if selected == binary
        ));
    }
}

#[test]
fn original_parser_phase_wins_over_feature_annotations_and_former_path_rejections() {
    for path in [
        "built-ins/Map/prototype/feature.js",
        "built-ins/Proxy/apply/arguments-realm.js",
    ] {
        let mut input = case(path, "let value = ;");
        input.negative = Some(Arc::new(NegativeExpectation {
            phase: NegativePhase::Parse,
            error_type: "SyntaxError".into(),
        }));
        let parsed = run_one_case(
            &input,
            &PreludeStore::default(),
            60_000,
            ExecutionBackend::WasmAot,
        );
        assert_eq!(parsed.status, TestStatus::Passed, "{path}: {parsed:?}");

        input.negative = Some(Arc::new(NegativeExpectation {
            phase: NegativePhase::Early,
            error_type: "SyntaxError".into(),
        }));
        let wrong_phase = run_one_case(
            &input,
            &PreludeStore::default(),
            60_000,
            ExecutionBackend::WasmAot,
        );
        let TestStatus::Failed(failure) = wrong_phase.status else {
            panic!("a parser rejection cannot satisfy the wrong early-error phase: {path}");
        };
        assert_eq!(failure.kind, FailureKind::EarlyError);
        assert!(
            failure.detail.contains("negative test error mismatch"),
            "{failure:?}"
        );
    }
}

#[test]
fn annotated_formerly_rejected_source_reaches_actual_runtime_completion() {
    let mut input = case(
        "built-ins/Proxy/apply/arguments-realm.js",
        "throw new TypeError('original runtime completion');",
    );
    input.negative = Some(Arc::new(NegativeExpectation {
        phase: NegativePhase::Runtime,
        error_type: "TypeError".into(),
    }));
    let observed = run_one_case(
        &input,
        &PreludeStore::default(),
        60_000,
        ExecutionBackend::WasmAot,
    );
    assert_eq!(observed.status, TestStatus::Passed, "{observed:?}");
}

#[test]
fn actual_unavailable_capability_cannot_satisfy_an_annotated_runtime_negative() {
    let mut input = case(
        "built-ins/Object/freeze/freeze-sharedarraybuffer.js",
        "new WeakMap();",
    );
    input.negative = Some(Arc::new(NegativeExpectation {
        phase: NegativePhase::Runtime,
        error_type: "TypeError".into(),
    }));
    let observed = run_one_case(
        &input,
        &PreludeStore::default(),
        60_000,
        ExecutionBackend::WasmAot,
    );
    let TestStatus::Failed(failure) = observed.status else {
        panic!("unavailable capability must not satisfy a JavaScript runtime negative");
    };
    assert_eq!(failure.kind, FailureKind::Unsupported, "{failure:?}");
    assert_eq!(failure.outcome, OutcomeKind::NotImplemented, "{failure:?}");
    assert_eq!(failure.origin, FailureOrigin::Unknown, "{failure:?}");
    assert!(
        failure
            .detail
            .contains(&lila_ir::RuntimeUnavailableCapability::WeakReachability.to_string()),
        "classification must come from the actual runtime capability: {failure:?}",
    );
}
