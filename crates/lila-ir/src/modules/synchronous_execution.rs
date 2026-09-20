//! Validated execution records for compiled Module-entry graphs.

use crate::{FunctionId, ModuleNamespaceModeIr, ModuleUnitId};

/// An ultimate module environment cell, resolved before Wasm emission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleCellIr {
    Binding {
        module: ModuleUnitId,
        slot: u32,
    },
    Namespace {
        module: ModuleUnitId,
        mode: ModuleNamespaceModeIr,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleActivationKindIr {
    Synchronous,
    Async,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleRequestPhaseIr {
    Evaluation,
    Defer,
}

/// One original request. Phase and occurrence order survive linking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleExecutionRequestIr {
    phase: ModuleRequestPhaseIr,
    target: ModuleUnitId,
}

impl ModuleExecutionRequestIr {
    pub(super) const fn new(phase: ModuleRequestPhaseIr, target: ModuleUnitId) -> Self {
        Self { phase, target }
    }
    pub const fn phase(&self) -> ModuleRequestPhaseIr {
        self.phase
    }
    pub const fn target(&self) -> ModuleUnitId {
        self.target
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleActivationIr {
    module: ModuleUnitId,
    function: FunctionId,
    kind: ModuleActivationKindIr,
    requests: Vec<ModuleExecutionRequestIr>,
}

impl ModuleActivationIr {
    pub(super) fn new(
        module: ModuleUnitId,
        function: FunctionId,
        kind: ModuleActivationKindIr,
        requests: Vec<ModuleExecutionRequestIr>,
    ) -> Self {
        Self {
            module,
            function,
            kind,
            requests,
        }
    }
    pub const fn module(&self) -> ModuleUnitId {
        self.module
    }
    pub fn function(&self) -> &FunctionId {
        &self.function
    }
    pub const fn kind(&self) -> ModuleActivationKindIr {
        self.kind
    }
    pub fn requests(&self) -> &[ModuleExecutionRequestIr] {
        &self.requests
    }
}

/// Allocation precedes all instantiation; every request targets an owned record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleExecutionGraphIr {
    record_count: u32,
    activations: Vec<ModuleActivationIr>,
}

impl ModuleExecutionGraphIr {
    pub(super) fn new(record_count: u32, activations: Vec<ModuleActivationIr>) -> Self {
        assert!(
            !activations.is_empty(),
            "an execution graph owns at least its entry"
        );
        let functions: std::collections::BTreeSet<_> = activations
            .iter()
            .map(ModuleActivationIr::function)
            .collect();
        assert_eq!(
            functions.len(),
            activations.len(),
            "each module has its own compiled lexical owner"
        );
        let modules: std::collections::BTreeSet<_> =
            activations.iter().map(ModuleActivationIr::module).collect();
        assert_eq!(
            modules.len(),
            activations.len(),
            "a module has one activation"
        );
        assert!(
            modules.iter().all(|&module| module < record_count),
            "module IDs fit the record vector"
        );
        assert!(
            activations
                .iter()
                .flat_map(ModuleActivationIr::requests)
                .all(|request| modules.contains(&request.target())),
            "every requested module has an activation"
        );
        Self {
            record_count,
            activations,
        }
    }
    pub const fn record_count(&self) -> u32 {
        self.record_count
    }
    pub fn activations(&self) -> &[ModuleActivationIr] {
        &self.activations
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleImportBindingIr {
    pub name: String,
    pub target: ModuleCellIr,
}

/// Evaluation topology is a runtime property, never a static SCC schedule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleEvaluationIr {
    module: ModuleUnitId,
}

impl ModuleEvaluationIr {
    pub(super) const fn new(module: ModuleUnitId) -> Self {
        Self { module }
    }
    pub const fn module(&self) -> ModuleUnitId {
        self.module
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeferredModuleEvaluationIr {
    module: ModuleUnitId,
}

impl DeferredModuleEvaluationIr {
    pub(super) const fn new(module: ModuleUnitId) -> Self {
        Self { module }
    }
    pub const fn module(&self) -> ModuleUnitId {
        self.module
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requests_preserve_phase_and_occurrence_order() {
        let requests = vec![
            ModuleExecutionRequestIr::new(ModuleRequestPhaseIr::Defer, 0),
            ModuleExecutionRequestIr::new(ModuleRequestPhaseIr::Evaluation, 0),
        ];
        let graph = ModuleExecutionGraphIr::new(
            1,
            vec![ModuleActivationIr::new(
                0,
                "module".into(),
                ModuleActivationKindIr::Async,
                requests.clone(),
            )],
        );
        assert_eq!(graph.activations()[0].requests(), requests);
    }

    #[test]
    #[should_panic(expected = "every requested module has an activation")]
    fn missing_request_activation_is_rejected() {
        ModuleExecutionGraphIr::new(
            2,
            vec![ModuleActivationIr::new(
                0,
                "module".into(),
                ModuleActivationKindIr::Synchronous,
                vec![ModuleExecutionRequestIr::new(
                    ModuleRequestPhaseIr::Evaluation,
                    1,
                )],
            )],
        );
    }
    #[test]
    #[should_panic(expected = "at least its entry")]
    fn an_execution_graph_cannot_be_empty() {
        ModuleExecutionGraphIr::new(0, Vec::new());
    }

    #[test]
    #[should_panic(expected = "its own compiled lexical owner")]
    fn distinct_modules_cannot_share_one_source_activation_function() {
        ModuleExecutionGraphIr::new(
            2,
            vec![
                ModuleActivationIr::new(
                    0,
                    "owner".into(),
                    ModuleActivationKindIr::Synchronous,
                    Vec::new(),
                ),
                ModuleActivationIr::new(
                    1,
                    "owner".into(),
                    ModuleActivationKindIr::Async,
                    Vec::new(),
                ),
            ],
        );
    }
}
