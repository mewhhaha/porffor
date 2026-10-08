use super::*;

#[test]
fn weak_capability_rejection_cannot_pass_by_catch_or_runtime_negative() {
    let capability = lila_ir::RuntimeUnavailableCapability::WeakReachability;
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
            let TestStatus::Failed(failure) = result.status else {
                panic!("weak reachability unavailable cannot become PASS: {expected:?}: {call}");
            };
            assert_eq!(failure.kind, FailureKind::Unsupported, "{failure:?}");
            assert_eq!(failure.outcome, OutcomeKind::NotImplemented, "{failure:?}");
            assert_eq!(failure.origin, FailureOrigin::Unknown, "{failure:?}");
            assert_eq!(failure.detail, format!("[origin:unknown] {capability}"));
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
