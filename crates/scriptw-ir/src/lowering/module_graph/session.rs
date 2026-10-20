use super::*;

impl LoweringSession {
    /// Lowers a retained host-loaded closure, selecting the original entry
    /// grammar and sharing finite-source parse products across discovery.
    pub fn lower_loaded_graph(
        &self,
        sources: &ModuleGraphSources,
        host_surface_policy: HostSurfacePolicy,
        prelude: Option<&ParsedScript>,
    ) -> ProgramIr {
        self.lower_graph_with_admission(
            sources,
            host_surface_policy,
            prelude,
            modules::GraphAdmission::LoadedClosure,
        )
    }

    /// Complete-catalog counterpart; undeclared requests remain outside the
    /// supplied host authority even when a prepared source spells their name.
    pub fn lower_complete_catalog(
        &self,
        sources: &ModuleGraphSources,
        host_surface_policy: HostSurfacePolicy,
        prelude: Option<&ParsedScript>,
    ) -> ProgramIr {
        self.lower_graph_with_admission(
            sources,
            host_surface_policy,
            prelude,
            modules::GraphAdmission::CompleteCatalog,
        )
    }

    fn lower_graph_with_admission(
        &self,
        sources: &ModuleGraphSources,
        host_surface_policy: HostSurfacePolicy,
        prelude: Option<&ParsedScript>,
        admission: modules::GraphAdmission,
    ) -> ProgramIr {
        let entry_is_script = sources
            .modules
            .get(sources.entry as usize)
            .is_some_and(|source| source.goal() == ParseGoal::Script);
        lower_graph(
            sources,
            entry_is_script,
            host_surface_policy,
            prelude,
            admission,
            self,
        )
    }
}
