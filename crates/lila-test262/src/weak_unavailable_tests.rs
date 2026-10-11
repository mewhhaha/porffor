use super::*;

#[test]
fn supported_weak_construction_passes_positive_cases_and_rejects_runtime_negatives() {
    let preludes = PreludeStore::default();
    for call in [
        "new WeakMap();",
        "new WeakSet();",
        "new WeakRef({});",
        "new FinalizationRegistry(function() {});",
    ] {
        for expected in [None, Some(""), Some("TypeError")] {
            let path = "built-ins/weak-reachability/capability.js";
            let body = if expected.is_none() {
                format!("try {{ {call} }} catch (error) {{}}")
            } else {
                call.to_string()
            };
            let source = format!("/*---\nflags: [raw]\n---*/\n{body}");
            let mut case = parse_test_case(path.into(), PathBuf::from(path), source);
            case.negative = expected.map(|error_type| {
                Arc::new(NegativeExpectation {
                    phase: NegativePhase::Runtime,
                    error_type: error_type.into(),
                })
            });
            let result = run_one_case(&case, &preludes, 60_000, ExecutionBackend::WasmAot);
            match expected {
                None => assert_eq!(result.status, TestStatus::Passed, "{call}: {result:?}"),
                Some(_) => {
                    let TestStatus::Failed(failure) = result.status else {
                        panic!("successful weak construction must not satisfy a runtime negative");
                    };
                    assert_ne!(failure.kind, FailureKind::Unsupported, "{failure:?}");
                }
            }
        }
    }
}

#[test]
fn weak_early_javascript_errors_still_satisfy_runtime_negatives() {
    let preludes = PreludeStore::default();
    for call in [
        "WeakMap();",
        "WeakSet();",
        "WeakRef({});",
        "FinalizationRegistry(function() {});",
        "new WeakRef(1);",
        "new FinalizationRegistry(null);",
        "WeakRef.prototype.deref.call({});",
        "WeakMap.prototype.getOrInsertComputed.call({}, {}, null);",
    ] {
        let path = "built-ins/weak-reachability/early-error.js";
        let source = format!(
            "/*---\nflags: [raw]\nnegative:\n  phase: runtime\n  type: TypeError\n---*/\n{call}"
        );
        let case = parse_test_case(path.into(), PathBuf::from(path), source);
        let result = run_one_case(&case, &preludes, 60_000, ExecutionBackend::WasmAot);
        assert_eq!(result.status, TestStatus::Passed, "{call}: {result:?}");
    }
}
