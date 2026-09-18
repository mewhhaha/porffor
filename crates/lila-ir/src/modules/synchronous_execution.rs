//! AOT module instantiation records for synchronous module graphs.

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

/// One private execution owner and its dependency evaluator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SynchronousModuleActivationIr {
    pub module: ModuleUnitId,
    pub function: FunctionId,
    pub evaluator: FunctionId,
}

/// Allocation precedes all instantiation; all instantiation precedes evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SynchronousModuleGraphIr {
    pub record_count: u32,
    pub activations: Vec<SynchronousModuleActivationIr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleImportBindingIr {
    pub name: String,
    pub target: ModuleCellIr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SynchronousModuleEvaluationIr {
    module: ModuleUnitId,
    dependencies: Vec<ModuleUnitId>,
    component_members: Vec<ModuleUnitId>,
}

impl SynchronousModuleEvaluationIr {
    pub(super) fn new(
        module: ModuleUnitId,
        dependencies: Vec<ModuleUnitId>,
        component_members: Vec<ModuleUnitId>,
    ) -> Self {
        assert!(
            component_members.contains(&module),
            "an evaluator belongs to its component"
        );
        assert_eq!(
            component_members
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            component_members.len(),
            "a component contains each module once"
        );
        Self {
            module,
            dependencies,
            component_members,
        }
    }

    pub const fn module(&self) -> ModuleUnitId {
        self.module
    }

    /// Evaluation-phase dependencies in original request order.
    pub fn dependencies(&self) -> &[ModuleUnitId] {
        &self.dependencies
    }

    /// The complete static evaluation SCC, including this module.
    pub fn component_members(&self) -> &[ModuleUnitId] {
        &self.component_members
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeferredModuleEvaluationIr {
    pub module: ModuleUnitId,
    /// Transitive requested modules, including deferred edges, deduplicated.
    pub readiness: Vec<ModuleReadinessNodeIr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleReadinessNodeIr {
    pub module: ModuleUnitId,
    pub dependencies: Vec<ModuleUnitId>,
}
