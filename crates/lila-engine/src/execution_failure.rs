use super::{DynamicSourceRuntimeOperation, EngineError};

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

pub(super) fn finish_wasm_execution<T>(
    root: Result<T, EngineError>,
    agents: Result<(), EngineError>,
) -> Result<T, EngineError> {
    match (root, agents) {
        (Ok(result), Ok(())) => Ok(result),
        (Err(failure), Ok(())) | (Ok(_), Err(failure)) => Err(failure),
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
                let combined = finish_wasm_execution::<()>(Err(root), Err(workers))
                    .expect_err("both errors must survive");
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
        let root_failure =
            finish_wasm_execution::<()>(Err(root.clone()), Ok(())).expect_err("root exception");
        assert_eq!(
            root_failure.wasm_execution_failure_kind(),
            Some(WasmExecutionFailureKind::JavaScriptException)
        );
        assert_eq!(
            root_failure.wasm_javascript_exception_constructor_name(),
            Some("TypeError")
        );
        let worker_failure = finish_wasm_execution(
            Ok(()),
            Err(EngineError::from_execution_failures(root, Vec::new())),
        )
        .expect_err("worker exception");
        assert_eq!(
            worker_failure.wasm_execution_failure_kind(),
            Some(WasmExecutionFailureKind::ConcurrentFailure)
        );
        assert_eq!(
            worker_failure.wasm_javascript_exception_constructor_name(),
            None
        );
    }
}
