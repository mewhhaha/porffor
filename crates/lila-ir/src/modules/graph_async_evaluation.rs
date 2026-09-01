//! Async-module propagation and pending-dependency queries.

use std::collections::BTreeSet;

use super::evaluation_mode::ModuleEvaluationModeIr;
use super::graph::ModuleGraphIr;
use super::record::ModuleUnitId;

impl ModuleGraphIr {
    /// `[[AsyncEvaluation]]` (16.2.1.5.2 steps 11-14) for every unit, indexed
    /// by [`ModuleUnitId`].
    ///
    /// A module evaluates asynchronously when it has its own top-level `await`
    /// *or* when `InnerModuleEvaluation` gave it a non-zero
    /// [`pending_async_dependencies`]: step 11.b.i copies a dependency's
    /// `[[AsyncEvaluation]]` upwards, so asynchrony is transitive along
    /// dependency edges and never stops at the module that wrote the `await`.
    ///
    /// A cycle is one unit for this purpose. Every member of a strongly-
    /// connected component shares one `[[TopLevelCapability]]` in the spec and
    /// they resume as a group, so one member's `await` makes the whole
    /// component asynchronous.
    ///
    /// `evaluation_order` is a reverse-topological order over components, so a
    /// single forward pass reaches every dependency before its dependent and
    /// the propagation needs no fixed point.
    ///
    /// [`pending_async_dependencies`]: Self::pending_async_dependencies
    #[must_use]
    pub fn async_evaluation(&self) -> Vec<bool> {
        let components = self.component_of_unit();
        let component_count = components.iter().copied().max().map_or(0, |max| max + 1);
        let mut component_async = vec![false; component_count];

        for member in &self.evaluation_order {
            if self.evaluation_mode(*member) != ModuleEvaluationModeIr::Eager {
                continue;
            }
            let component = components[*member as usize];
            if self.has_tla(*member) {
                component_async[component] = true;
            }
            for dependency in self.evaluation_dependencies_of(*member) {
                let dependency = dependency.target();
                let dependency_component = components[dependency as usize];
                // Same component: the cycle is settled by the `has_tla` test
                // above over all of its members, and reading its own flag mid-
                // pass would depend on member order.
                if dependency_component != component && component_async[dependency_component] {
                    component_async[component] = true;
                }
            }
        }

        components
            .iter()
            .enumerate()
            .map(|(module, component)| {
                let module = ModuleUnitId::try_from(module).expect(
                    "unit index is capped by build_graph, which rejects a graph with more units than MAX_LINKABLE_MODULE_UNIT_ID",
                );
                self.evaluation_mode(module) == ModuleEvaluationModeIr::Eager
                    && component_async[*component]
            })
            .collect()
    }

    /// `[[PendingAsyncDependencies]]` (16.2.1.5.2 step 11.b.ii) for one unit:
    /// how many of its dependencies it must wait for before its own body may
    /// resume.
    ///
    /// Counted over distinct dependency *components*, and excluding the unit's
    /// own component: a cycle member is evaluated by the same
    /// `InnerModuleEvaluation` call and is never something to wait on, and two
    /// requests that resolve to one module are one dependency.
    #[must_use]
    pub fn pending_async_dependencies(&self, module: ModuleUnitId) -> usize {
        if self.evaluation_mode(module) != ModuleEvaluationModeIr::Eager {
            return 0;
        }
        let components = self.component_of_unit();
        let asynchronous = self.async_evaluation();
        let Some(&own) = components.get(module as usize) else {
            return 0;
        };
        let mut counted: BTreeSet<usize> = BTreeSet::new();
        for dependency in self.evaluation_dependencies_of(module) {
            let dependency = dependency.target();
            let component = components[dependency as usize];
            if component != own && asynchronous[dependency as usize] {
                counted.insert(component);
            }
        }
        counted.len()
    }
}
