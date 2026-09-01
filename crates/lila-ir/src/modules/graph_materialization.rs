//! Queries that bound which linked modules may materialize at runtime.

use super::evaluation_mode::ModuleMaterializationModeIr;
use super::graph::ModuleGraphIr;
use super::module_unit::ModuleUnitIr;
use super::record::ModuleUnitId;

impl ModuleGraphIr {
    /// Runtime source-generation mode for one unit.
    ///
    /// `None` means either that `module` is not a unit of this graph or that it
    /// is source-phase-only. An unlinked graph follows [`Self::evaluation_mode`]
    /// and therefore retains its documented eager default.
    #[must_use]
    pub(super) fn materialization_mode(
        &self,
        module: ModuleUnitId,
    ) -> Option<ModuleMaterializationModeIr> {
        let index = usize::try_from(module).ok()?;
        self.units.get(index)?;
        self.evaluation_mode(module).materialization()
    }

    /// Units allowed to contribute bodies, aliases, objects or dispatchers to
    /// the emitted artifact.
    pub(super) fn materialized_units(
        &self,
    ) -> impl Iterator<Item = (ModuleUnitId, ModuleMaterializationModeIr, &ModuleUnitIr)> {
        self.units
            .iter()
            .enumerate()
            .filter_map(move |(index, unit)| {
                let id = ModuleUnitId::try_from(index).expect(
                    "unit index is capped by build_graph, which rejects a graph with more units than MAX_LINKABLE_MODULE_UNIT_ID",
                );
                self.materialization_mode(id)
                    .map(|mode| (id, mode, unit))
            })
    }
}
