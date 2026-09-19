use super::prepared_script::compile_module_prelude;
use super::*;

/// Lowers an already-loaded module graph into one `ProgramIr`.
///
/// `lila-ir` performs no IO: the host resolves and reads every source and
/// passes the closure in. The graph is linked at compile time and merged into
/// the single `ScriptIr` the backend emits, while the spec records stay
/// addressable on `ProgramIr::modules`.
pub fn lower_module_graph(sources: &ModuleGraphSources) -> ProgramIr {
    lower_module_graph_with_host_surface_policy(sources, HostSurfacePolicy::default())
}

pub fn lower_module_graph_with_host_surface_policy(
    sources: &ModuleGraphSources,
    host_surface_policy: HostSurfacePolicy,
) -> ProgramIr {
    lower_graph(sources, false, host_surface_policy, None)
}

/// Lowers a Script entry together with the modules its `import()` calls reach.
///
/// `import()` is legal in Script goal and names a module, so serving it needs
/// the same compiled targets a module's `import()` needs — but the entry itself
/// stays Script code: it is not made strict, its top-level `this` is
/// `globalThis`, and its declarations stay in the Script's own scope. See
/// [`ModuleGraphIr::entry_is_script`].
///
/// A Script that writes no `import()` has nothing to gain here and should be
/// lowered with [`lower`].
pub fn lower_script_graph(sources: &ModuleGraphSources) -> ProgramIr {
    lower_script_graph_with_host_surface_policy(sources, HostSurfacePolicy::default())
}

pub fn lower_script_graph_with_host_surface_policy(
    sources: &ModuleGraphSources,
    host_surface_policy: HostSurfacePolicy,
) -> ProgramIr {
    lower_graph(sources, true, host_surface_policy, None)
}

/// Lowers a Module graph with an independently parsed global Script prelude.
/// Both source goals remain authoritative and share only the runtime realm.
pub fn lower_module_graph_with_prelude(
    sources: &ModuleGraphSources,
    prelude: &ParsedScript,
    host_surface_policy: HostSurfacePolicy,
) -> ProgramIr {
    lower_graph(sources, false, host_surface_policy, Some(prelude))
}

fn lower_graph(
    sources: &ModuleGraphSources,
    entry_is_script: bool,
    host_surface_policy: HostSurfacePolicy,
    prelude: Option<&ParsedScript>,
) -> ProgramIr {
    let goal = if entry_is_script {
        ParseGoal::Script
    } else {
        ParseGoal::Module
    };
    let source_len = sources
        .modules
        .get(sources.entry as usize)
        .map_or(0, |module| module.source_text().len());
    let mut stages = vec![LoweringStage::ParsedSource];

    let invalid_goal = sources.modules.iter().enumerate().find(|(index, source)| {
        let expected = if *index == sources.entry as usize && entry_is_script {
            ParseGoal::Script
        } else {
            ParseGoal::Module
        };
        source.goal() != expected
    });
    if let Some((index, source)) = invalid_goal {
        let expected = if index == sources.entry as usize && entry_is_script {
            ParseGoal::Script
        } else {
            ParseGoal::Module
        };
        stages.push(LoweringStage::UnsupportedFeaturesRecorded);
        let mut program = new_program(goal, source_len, stages);
        program.diagnostics.push(IrDiagnostic::lowering(format!(
            "module graph source {} has {:?} syntax in a {:?} slot",
            source.key().as_str(),
            source.goal(),
            expected,
        )));
        return program;
    }

    let mut graph = match modules::link_loaded_graph(sources, entry_is_script) {
        Ok(graph) => graph,
        Err(rejection) => {
            if rejection.graph.is_some() {
                stages.push(LoweringStage::ModuleGraphLoaded);
                stages.push(LoweringStage::ModuleGraphLinked);
            }
            stages.push(LoweringStage::UnsupportedFeaturesRecorded);
            let mut program = new_program(goal, source_len, stages);
            program.diagnostics = rejection.diagnostics;
            program.modules = rejection.graph;
            return program;
        }
    };
    stages.push(LoweringStage::ModuleGraphLoaded);
    stages.push(LoweringStage::ModuleGraphLinked);

    // The whole graph is merged into one Script-goal source, in evaluation
    // order, and lowered once. See `modules::link` for why the merge happens on
    // source text and what it still declines to link.
    let linked = match modules::linked_script_source(sources, &mut graph) {
        Ok(linked) => linked,
        Err(diagnostics) => {
            stages.push(LoweringStage::UnsupportedFeaturesRecorded);
            let mut program = new_program(goal, source_len, stages);
            program.diagnostics = diagnostics;
            program.modules = Some(graph);
            return program;
        }
    };

    let definitions = linked.definitions;
    let linked = match lila_front::parse(
        linked.source.source_text,
        lila_front::ParseOptions {
            goal: ParseGoal::Script,
            filename: linked.source.filename,
        },
    ) {
        Ok(ParsedSource::Script(linked)) => linked,
        Ok(ParsedSource::Module(_)) => {
            unreachable!("Script parse options cannot produce a Module parsed product")
        }
        Err(error) => {
            stages.push(LoweringStage::UnsupportedFeaturesRecorded);
            let mut program = new_program(goal, source_len, stages);
            program.diagnostics.push(IrDiagnostic::lowering(format!(
                "linked module source did not parse as Script: {error}"
            )));
            program.modules = Some(graph);
            return program;
        }
    };

    let mut allocations = AnalysisAllocationState::default();
    let instantiation = if prelude.is_some() {
        ScriptInstantiation::ModuleAfterGlobalScript
    } else {
        ScriptInstantiation::FreshEntry
    };
    let mut program = lower_script_program_with_allocations(
        &linked,
        goal,
        source_len,
        stages,
        Some(graph),
        &definitions,
        host_surface_policy,
        &mut allocations,
        instantiation,
    );
    if let Some(prelude) = prelude {
        compile_module_prelude(&mut program, prelude, host_surface_policy, &mut allocations);
    }
    program
}
