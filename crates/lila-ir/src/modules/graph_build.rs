use std::collections::BTreeSet;

use crate::{IrDiagnostic, MAX_LINKABLE_MODULE_UNIT_ID};

use super::graph::ModuleGraphIr;
use super::link_error::ModuleLinkErrorIr;
use super::loaded_sources::{ModuleGraphSources, ModuleParse};
use super::module_key::ModuleKey;
use super::module_unit::ModuleUnitIr;
use super::record::{parse_module_record, ModuleUnitId};
use super::resolved_binding::ResolvedBindingIr;

/// Builds the graph from an already-loaded closure: turns every retained parse
/// product into a module record and records the host's resolutions.
///
/// A key the graph already holds is reused and never rebuilt. That is module
/// map identity (16.2.1.7 `HostLoadImportedModule`): however many importers
/// name a module, and however many times the host hands its source over, the
/// graph holds exactly one unit for it — which is what makes a module evaluate
/// once and hand out one namespace object.
pub(crate) fn build_graph(
    sources: &ModuleGraphSources,
) -> Result<ModuleGraphIr, Vec<IrDiagnostic>> {
    let mut graph = ModuleGraphIr::default();
    let mut diagnostics = Vec::new();
    // Source position -> unit id. The two differ as soon as one key arrives
    // twice, and `resolutions` is stated in source positions.
    let mut remap: Vec<ModuleUnitId> = Vec::with_capacity(sources.modules.len());
    let mut inconsistent: Vec<ModuleKey> = Vec::new();

    for source in &sources.modules {
        if let Some(&existing) = graph.keys.get(source.key()) {
            // Same key, different bytes: the host contradicted itself, and
            // there is no honest way to pick a winner.
            if graph.units[existing as usize].source_text != source.source_text()
                && !inconsistent.iter().any(|key| key == source.key())
            {
                inconsistent.push(source.key().clone());
            }
            remap.push(existing);
            continue;
        }
        // The one place a unit id is minted, and therefore the one place the
        // byte budgets B1/B2 can be enforced at run time (contract ledger R3).
        // The previous `unwrap_or(ModuleUnitId::MAX)` was worse than unchecked:
        // it saturated to a ten-digit id, which violates both budgets silently
        // and then fails downstream as a confusing `StripError`.
        let Some(id) = ModuleUnitId::try_from(graph.units.len())
            .ok()
            .filter(|id| *id <= MAX_LINKABLE_MODULE_UNIT_ID)
        else {
            diagnostics.push(
                ModuleLinkErrorIr::TooManyUnits {
                    count: sources.modules.len(),
                }
                .to_diagnostic(),
            );
            return Err(diagnostics);
        };
        remap.push(id);
        let record = match &source.parse {
            ModuleParse::Module(unit_source) => {
                parse_module_record(unit_source, id, source.key().clone())
            }
            ModuleParse::ScriptEntry(unit_source) => Ok(super::record::script_entry_record(
                unit_source,
                id,
                source.key().clone(),
            )),
            ModuleParse::Rejected { error, .. } => {
                Err(vec![super::early::module_parse_failure_diagnostic(error)])
            }
        };
        match record {
            Ok(record) => {
                let resolved_imports =
                    vec![ResolvedBindingIr::NotFound; record.import_entries.len()];
                let resolved_indirect_exports =
                    vec![ResolvedBindingIr::NotFound; record.indirect_export_entries.len()];
                graph.keys.insert(source.key().clone(), id);
                graph.units.push(ModuleUnitIr {
                    record,
                    source_text: source.source_text().to_string(),
                    meta_url: source.meta_url().to_string(),
                    hoist: None,
                    body: None,
                    functions: Vec::new(),
                    owned_env_bindings: Vec::new(),
                    namespace: None,
                    resolved_imports,
                    resolved_indirect_exports,
                });
            }
            Err(mut parse_diagnostics) => diagnostics.append(&mut parse_diagnostics),
        }
    }
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    graph.entry = remap
        .get(sources.entry as usize)
        .copied()
        .unwrap_or_default();
    let mut inconsistent_resolutions = BTreeSet::new();
    for (referrer, request, target) in &sources.resolutions {
        let (Some(&referrer), Some(&target)) =
            (remap.get(*referrer as usize), remap.get(*target as usize))
        else {
            continue;
        };
        let identity = (referrer, request.clone());
        if inconsistent_resolutions.contains(&identity) {
            continue;
        }
        match graph.resolutions.get(&identity).copied() {
            None => {
                graph.resolutions.insert(identity, target);
            }
            Some(existing) if existing == target => {}
            Some(_) => {
                // A phase-free request has exactly one host resolution. Keep
                // no winner: last-write-wins would make public row order part
                // of module identity.
                graph.resolutions.remove(&identity);
                inconsistent_resolutions.insert(identity);
            }
        }
    }
    for key in inconsistent {
        graph
            .link_errors
            .push(ModuleLinkErrorIr::InconsistentLoad { key });
    }
    for (referrer, request) in inconsistent_resolutions {
        graph
            .link_errors
            .push(ModuleLinkErrorIr::InconsistentResolution { referrer, request });
    }
    Ok(graph)
}
