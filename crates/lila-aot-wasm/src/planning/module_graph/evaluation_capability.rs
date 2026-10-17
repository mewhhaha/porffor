//! Static evaluation edges consume only declared executable record authority.

use super::ModuleGraphPlan;
use lila_ir::{ModuleRequestPhaseIr, RuntimeUnavailableCapability};
use std::collections::BTreeSet;

impl ModuleGraphPlan {
    pub(crate) fn unavailable_evaluation_capability(
        &self,
        module: u32,
    ) -> Option<RuntimeUnavailableCapability> {
        let mut pending = vec![module];
        let mut visited = BTreeSet::new();
        while let Some(module) = pending.pop() {
            if !visited.insert(module) {
                continue;
            }
            if self.source_record(module).is_some() {
                return Some(RuntimeUnavailableCapability::HostModuleEvaluation);
            }
            if let Some(activation) = self
                .graph
                .activations()
                .iter()
                .find(|activation| activation.module() == module)
            {
                pending.extend(activation.requests().iter().filter_map(|request| {
                    match request.phase() {
                        ModuleRequestPhaseIr::Evaluation => Some(request.target()),
                        ModuleRequestPhaseIr::Defer => None,
                    }
                }));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests;
