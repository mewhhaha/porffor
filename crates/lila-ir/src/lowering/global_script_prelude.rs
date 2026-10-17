//! Independently instantiated global Script entry after an earlier Script.
use super::*;

impl LoweringSession {
    /// Independently parsed global Script prelude and Script entry in one realm.
    pub fn lower_source_with_prelude(
        &self,
        source: &ParsedSource,
        prelude: &ParsedScript,
        host_surface_policy: HostSurfacePolicy,
    ) -> ProgramIr {
        match source {
            ParsedSource::Module(source) => self.lower_loaded_graph(
                &ModuleGraphSources::single(source),
                host_surface_policy,
                Some(prelude),
            ),
            ParsedSource::Script(source) => {
                let mut allocations = AnalysisAllocationState::default();
                allocations.prepared_sources = self.prepared_sources.clone();
                allocations.script_import_origin =
                    Some(crate::direct_eval_context::ScriptImportOrigin::Discovery);
                let mut program = lower_script_program_with_allocations(
                    source,
                    ParseGoal::Script,
                    source.source_text.len(),
                    vec![LoweringStage::ParsedSource],
                    None,
                    &modules::LinkedScriptDefinitions::default(),
                    host_surface_policy,
                    &mut allocations,
                    ScriptInstantiation::GlobalScriptAfterPrelude,
                );
                retain_global_script_entry(&mut program, &mut allocations);
                prepared_script::compile_module_prelude(
                    &mut program,
                    prelude,
                    host_surface_policy,
                    &mut allocations,
                );
                program
            }
        }
    }
}

/// Move the test entry's declaration/body owner out of bootstrap main. Its
/// GlobalDeclarationInstantiation must happen only after the prelude finishes.
pub(super) fn retain_global_script_entry(
    program: &mut ProgramIr,
    allocations: &mut AnalysisAllocationState,
) {
    if !program.diagnostics.is_empty() {
        return;
    }
    let script = program
        .script
        .as_mut()
        .expect("global Script entry lowers as Script");
    let id = allocations.allocate_static_script_id();
    script.script_entry_after_prelude = Some(crate::GlobalScriptEntryIr::new(PreparedScriptUnit {
        template_source: script.template_source.take(),
        eval_environment: script.eval_environment.take(),
        id,
        kind: PreparedScriptKind::RealmScript,
        strict: script.strict,
        body: std::mem::replace(
            &mut script.body,
            BlockIr {
                statements: Vec::new(),
                result_kind: ValueKind::Undefined,
                lexical_environment: None,
            },
        ),
        owned_env_bindings: std::mem::take(&mut script.owned_env_bindings),
        global_bindings: std::mem::take(&mut script.global_bindings),
        declarations: std::mem::take(&mut script.runtime_declarations),
        function_ids: script
            .functions
            .iter()
            .map(|function| function.id.clone())
            .collect(),
    }));
    // Realm creation precedes both Scripts. Their declarations remain owned by
    // the delayed units, while canonical standard globals exist before either.
    super::builtin_catalog::retain_canonical_standard_globals(&mut script.global_bindings);
    // Bootstrap still installs the host surface used by either independent Script.
    for builtin in &script.host_builtins {
        script.global_bindings.require_host_global(*builtin);
    }
    script.strict = false;
}
