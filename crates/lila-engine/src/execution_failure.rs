use super::{DynamicSourceRuntimeOperation, EngineError, ObservedCompletion, WasmExecutionOutcome};

/// The execution outcome relevant to conformance classification. Only a direct
/// JavaScript exception from the root can satisfy a runtime-negative test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmExecutionFailureKind {
    JavaScriptException,
    DynamicSource,
    ConcurrentFailure,
    Trap,
    Timeout,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum EngineExecutionFailure {
    JavaScriptException { constructor_name: Option<String> },
    DynamicSource(DynamicSourceRuntimeOperation),
    Trap,
    Timeout,
    Concurrent(ExecutionFailures),
}

/// An aggregate always retains at least one failed execution. Even a single
/// worker's JavaScript exception is distinct from a root JavaScript exception.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ExecutionFailures {
    first: Box<EngineError>,
    additional: Vec<EngineError>,
}

impl ExecutionFailures {
    fn iter(&self) -> impl Iterator<Item = &EngineError> {
        std::iter::once(self.first.as_ref()).chain(&self.additional)
    }

    fn kind(&self) -> WasmExecutionFailureKind {
        let mut kind = WasmExecutionFailureKind::DynamicSource;
        for failure in self.iter() {
            match failure.wasm_execution_failure_kind() {
                Some(WasmExecutionFailureKind::Trap) => return WasmExecutionFailureKind::Trap,
                Some(WasmExecutionFailureKind::Timeout) => {
                    kind = WasmExecutionFailureKind::Timeout;
                }
                Some(WasmExecutionFailureKind::DynamicSource) => {}
                None if matches!(
                    failure
                        .ir_diagnostic()
                        .and_then(lila_ir::IrDiagnostic::unsupported_feature),
                    Some(lila_ir::UnsupportedFeature::DynamicSource(_))
                ) => {}
                Some(
                    WasmExecutionFailureKind::JavaScriptException
                    | WasmExecutionFailureKind::ConcurrentFailure,
                )
                | None => {
                    if kind != WasmExecutionFailureKind::Timeout {
                        kind = WasmExecutionFailureKind::ConcurrentFailure;
                    }
                }
            }
        }
        kind
    }
}

impl EngineError {
    pub(super) fn from_execution_failure(
        failure: EngineExecutionFailure,
        message: impl Into<String>,
    ) -> Self {
        let mut error = Self::new(message);
        error.execution_failure = Some(failure);
        error
    }

    pub(super) fn from_runtime_dynamic_source_operation(
        operation: DynamicSourceRuntimeOperation,
    ) -> Self {
        Self::from_execution_failure(
            EngineExecutionFailure::DynamicSource(operation),
            operation.to_string(),
        )
    }

    pub(super) fn from_execution_failures(
        first: EngineError,
        additional: Vec<EngineError>,
    ) -> Self {
        let failures = ExecutionFailures {
            first: Box::new(first),
            additional,
        };
        let message = format!(
            "Wasm execution failures: {}",
            failures
                .iter()
                .map(EngineError::message)
                .collect::<Vec<_>>()
                .join("; ")
        );
        Self::from_execution_failure(EngineExecutionFailure::Concurrent(failures), message)
    }

    pub fn wasm_execution_failure_kind(&self) -> Option<WasmExecutionFailureKind> {
        self.execution_failure
            .as_ref()
            .map(|failure| match failure {
                EngineExecutionFailure::JavaScriptException { .. } => {
                    WasmExecutionFailureKind::JavaScriptException
                }
                EngineExecutionFailure::DynamicSource(_) => WasmExecutionFailureKind::DynamicSource,
                EngineExecutionFailure::Trap => WasmExecutionFailureKind::Trap,
                EngineExecutionFailure::Timeout => WasmExecutionFailureKind::Timeout,
                EngineExecutionFailure::Concurrent(failures) => failures.kind(),
            })
    }

    /// The final root throw's data-property `constructor.name`, when available
    /// without invoking user code. This is neither diagnostic `.name` nor an
    /// intrinsic constructor identity. Primitive throws and worker aggregates
    /// do not provide an exception constructor name.
    pub fn wasm_javascript_exception_constructor_name(&self) -> Option<&str> {
        match &self.execution_failure {
            Some(EngineExecutionFailure::JavaScriptException { constructor_name }) => {
                constructor_name.as_deref()
            }
            None
            | Some(
                EngineExecutionFailure::DynamicSource(_)
                | EngineExecutionFailure::Trap
                | EngineExecutionFailure::Timeout
                | EngineExecutionFailure::Concurrent(_),
            ) => None,
        }
    }

    /// Every distinct dynamic-source rejection retained from the root and its
    /// workers, including rejections accompanied by unrelated real failures.
    pub fn runtime_dynamic_source_operations(&self) -> Vec<DynamicSourceRuntimeOperation> {
        match &self.execution_failure {
            None => Vec::new(),
            Some(EngineExecutionFailure::DynamicSource(operation)) => vec![*operation],
            Some(EngineExecutionFailure::Concurrent(failures)) => {
                let mut operations = Vec::new();
                for operation in failures
                    .iter()
                    .flat_map(EngineError::runtime_dynamic_source_operations)
                {
                    if !operations.contains(&operation) {
                        operations.push(operation);
                    }
                }
                operations
            }
            Some(
                EngineExecutionFailure::JavaScriptException { .. }
                | EngineExecutionFailure::Trap
                | EngineExecutionFailure::Timeout,
            ) => Vec::new(),
        }
    }
}

pub(super) fn finish_wasm_execution(
    root: Result<WasmExecutionOutcome, EngineError>,
    agents: Result<(), EngineError>,
) -> Result<WasmExecutionOutcome, EngineError> {
    match (root, agents) {
        (Ok(result), Ok(())) => Ok(result),
        (Err(failure), Ok(())) => Err(failure),
        (Ok(result), Err(agents)) => match result {
            WasmExecutionOutcome::Legacy(_) => Err(agents),
            WasmExecutionOutcome::Structured(observation) => match observation.completion {
                ObservedCompletion::Normal(_) => Err(agents),
                ObservedCompletion::Throw(_) => Err(EngineError::from_execution_failures(
                    EngineError::from_execution_failure(
                        EngineExecutionFailure::JavaScriptException {
                            constructor_name: None,
                        },
                        observation.note,
                    ),
                    vec![agents],
                )),
            },
        },
        (Err(root), Err(agents)) => Err(EngineError::from_execution_failures(root, vec![agents])),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lila_ir::DynamicFunctionKind;

    fn rejection(operation: DynamicSourceRuntimeOperation) -> EngineError {
        EngineError::from_runtime_dynamic_source_operation(operation)
    }

    #[test]
    fn distinct_worker_capabilities_are_retained_and_duplicates_are_coalesced() {
        let eval = DynamicSourceRuntimeOperation::Eval;
        let function = DynamicSourceRuntimeOperation::Function(DynamicFunctionKind::Ordinary);
        let failures = EngineError::from_execution_failures(
            rejection(eval),
            vec![rejection(function), rejection(eval)],
        );
        assert_eq!(
            failures.runtime_dynamic_source_operations(),
            vec![eval, function]
        );
        assert_eq!(
            failures.wasm_execution_failure_kind(),
            Some(WasmExecutionFailureKind::DynamicSource)
        );
    }

    #[test]
    fn compile_and_runtime_capabilities_remain_unsupported_in_both_orders() {
        let compile =
            EngineError::from_ir_diagnostic(lila_ir::IrDiagnostic::unsupported_dynamic_source(
                lila_ir::DynamicSourceGap::aot_known_source(lila_ir::DynamicSourceKind::Function(
                    DynamicFunctionKind::Generator,
                )),
            ));
        assert_eq!(compile.wasm_execution_failure_kind(), None);
        assert!(compile.runtime_dynamic_source_operations().is_empty());
        let compile_only = EngineError::from_execution_failures(compile.clone(), Vec::new());
        assert_eq!(
            compile_only.wasm_execution_failure_kind(),
            Some(WasmExecutionFailureKind::DynamicSource)
        );
        assert!(compile_only.runtime_dynamic_source_operations().is_empty());

        let runtime = rejection(DynamicSourceRuntimeOperation::Eval);
        for (root, worker) in [
            (compile.clone(), runtime.clone()),
            (runtime, compile.clone()),
        ] {
            let workers = EngineError::from_execution_failures(worker, Vec::new());
            let Err(combined) = finish_wasm_execution(Err(root), Err(workers)) else {
                panic!("both unsupported failures must survive");
            };
            assert_eq!(
                combined.wasm_execution_failure_kind(),
                Some(WasmExecutionFailureKind::DynamicSource)
            );
            assert_eq!(
                combined.runtime_dynamic_source_operations(),
                vec![DynamicSourceRuntimeOperation::Eval]
            );
            assert_eq!(combined.wasm_javascript_exception_constructor_name(), None);
            assert!(combined.message().contains(compile.message()));
            assert!(combined
                .message()
                .contains(&DynamicSourceRuntimeOperation::Eval.to_string()));
        }
    }

    #[test]
    fn compiled_capability_gaps_never_hide_other_failures() {
        let compile =
            EngineError::from_ir_diagnostic(lila_ir::IrDiagnostic::unsupported_dynamic_source(
                lila_ir::DynamicSourceGap::aot_known_source(lila_ir::DynamicSourceKind::Function(
                    DynamicFunctionKind::Generator,
                )),
            ));
        let capabilities = EngineError::from_execution_failures(
            compile,
            vec![rejection(DynamicSourceRuntimeOperation::Eval)],
        );
        for (other, expected) in [
            (
                EngineError::from_parse_error(lila_front::ParseError::malformed(
                    "parse marker: unsupported dynamic-source",
                    None,
                )),
                WasmExecutionFailureKind::ConcurrentFailure,
            ),
            (
                EngineError::new("untyped marker: unsupported dynamic-source"),
                WasmExecutionFailureKind::ConcurrentFailure,
            ),
            (
                EngineError::from_ir_diagnostic(lila_ir::IrDiagnostic::unsupported(
                    "untyped IR marker: unsupported dynamic-source",
                )),
                WasmExecutionFailureKind::ConcurrentFailure,
            ),
            (
                EngineError::from_execution_failure(
                    EngineExecutionFailure::JavaScriptException {
                        constructor_name: Some("TypeError".to_string()),
                    },
                    "uncaught throw: TypeError: unsupported dynamic-source",
                ),
                WasmExecutionFailureKind::ConcurrentFailure,
            ),
            (
                EngineError::from_execution_failure(
                    EngineExecutionFailure::Trap,
                    "trap marker: unsupported dynamic-source",
                ),
                WasmExecutionFailureKind::Trap,
            ),
            (
                EngineError::from_execution_failure(
                    EngineExecutionFailure::Timeout,
                    "timeout marker: unsupported dynamic-source",
                ),
                WasmExecutionFailureKind::Timeout,
            ),
        ] {
            for (root, workers) in [
                (other.clone(), capabilities.clone()),
                (capabilities.clone(), other.clone()),
            ] {
                let Err(combined) = finish_wasm_execution(Err(root), Err(workers)) else {
                    panic!("a capability gap must not conceal another failure");
                };
                assert_eq!(combined.wasm_execution_failure_kind(), Some(expected));
                assert_eq!(
                    combined.runtime_dynamic_source_operations(),
                    vec![DynamicSourceRuntimeOperation::Eval]
                );
                assert_eq!(combined.wasm_javascript_exception_constructor_name(), None);
                assert!(combined.message().contains(other.message()));
                assert!(combined.message().contains(capabilities.message()));
            }
        }
    }

    #[test]
    fn root_and_worker_failures_survive_in_both_orders_without_becoming_js_exceptions() {
        for reason in [
            EngineExecutionFailure::JavaScriptException {
                constructor_name: None,
            },
            EngineExecutionFailure::Trap,
            EngineExecutionFailure::Timeout,
        ] {
            let expected = match reason {
                EngineExecutionFailure::JavaScriptException { .. } => {
                    WasmExecutionFailureKind::ConcurrentFailure
                }
                EngineExecutionFailure::Trap => WasmExecutionFailureKind::Trap,
                EngineExecutionFailure::Timeout => WasmExecutionFailureKind::Timeout,
                EngineExecutionFailure::DynamicSource(_)
                | EngineExecutionFailure::Concurrent(_) => {
                    unreachable!("fixture has only real execution failures")
                }
            };
            let real = EngineError::from_execution_failure(reason, "root failure evidence");
            let capability = EngineError::from_execution_failures(
                rejection(DynamicSourceRuntimeOperation::Eval),
                Vec::new(),
            );
            for (root, workers) in [(real.clone(), capability.clone()), (capability, real)] {
                let Err(combined) = finish_wasm_execution(Err(root), Err(workers)) else {
                    panic!("both errors must survive");
                };
                assert_eq!(combined.wasm_execution_failure_kind(), Some(expected));
                assert_eq!(
                    combined.runtime_dynamic_source_operations(),
                    vec![DynamicSourceRuntimeOperation::Eval]
                );
                assert!(combined.message().contains("root failure evidence"));
                assert!(combined.message().contains("dynamic-source"));
            }
        }
    }

    #[test]
    fn only_an_unaccompanied_root_js_throw_is_a_js_exception() {
        let root = EngineError::from_execution_failure(
            EngineExecutionFailure::JavaScriptException {
                constructor_name: Some("TypeError".to_string()),
            },
            "uncaught throw: TypeError: root",
        );
        let Err(root_failure) = finish_wasm_execution(Err(root.clone()), Ok(())) else {
            panic!("root exception must survive");
        };
        assert_eq!(
            root_failure.wasm_execution_failure_kind(),
            Some(WasmExecutionFailureKind::JavaScriptException)
        );
        assert_eq!(
            root_failure.wasm_javascript_exception_constructor_name(),
            Some("TypeError")
        );
        let Err(worker_failure) = finish_wasm_execution(
            Ok(WasmExecutionOutcome::Legacy(crate::RunOutcome {
                backend_used: crate::ExecutionBackend::WasmAot,
                note: "normal root completion".to_string(),
            })),
            Err(EngineError::from_execution_failures(root, Vec::new())),
        ) else {
            panic!("worker exception must survive");
        };
        assert_eq!(
            worker_failure.wasm_execution_failure_kind(),
            Some(WasmExecutionFailureKind::ConcurrentFailure)
        );
        assert_eq!(
            worker_failure.wasm_javascript_exception_constructor_name(),
            None
        );
    }

    #[test]
    fn structured_throws_remain_observations_until_workers_also_fail() {
        let observed = |completion| {
            WasmExecutionOutcome::Structured(crate::ObservedRunOutcome {
                backend_used: crate::ExecutionBackend::WasmAot,
                completion,
                output_events: Vec::new(),
                note: "structured root evidence".to_string(),
            })
        };
        let root_throw = ObservedCompletion::Throw(crate::ObservedJsValue::Object);
        let outcome = finish_wasm_execution(Ok(observed(root_throw.clone())), Ok(()))
            .and_then(WasmExecutionOutcome::into_structured)
            .expect("a root throw alone remains a successful structured observation");
        assert_eq!(outcome.completion, root_throw);

        for (completion, expected_kind) in [
            (root_throw, WasmExecutionFailureKind::ConcurrentFailure),
            (
                ObservedCompletion::Normal(crate::ObservedJsValue::Undefined),
                WasmExecutionFailureKind::DynamicSource,
            ),
        ] {
            let Err(failure) = finish_wasm_execution(
                Ok(observed(completion)),
                Err(EngineError::from_execution_failures(
                    rejection(DynamicSourceRuntimeOperation::Eval),
                    Vec::new(),
                )),
            ) else {
                panic!("worker failure must remain observable");
            };
            assert_eq!(failure.wasm_execution_failure_kind(), Some(expected_kind));
            assert_eq!(
                failure.runtime_dynamic_source_operations(),
                vec![DynamicSourceRuntimeOperation::Eval]
            );
            assert_eq!(failure.wasm_javascript_exception_constructor_name(), None);
            if expected_kind == WasmExecutionFailureKind::ConcurrentFailure {
                assert!(failure.message().contains("structured root evidence"));
            }
        }
    }
}
