//! Preserve static rejection while assigning dynamic-only load failures to import jobs.

use super::loaded_sources::ModuleParse;
use super::*;
use crate::{EarlyErrorCode, IrDiagnostic, NativeErrorKind, ParseGoal};
use std::collections::{BTreeMap, BTreeSet};

mod closure;
use closure::StaticClosure;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DynamicModuleRejectionStage {
    ModuleLoad,
    Dependencies,
}

/// The abrupt completion a rejected `import()` settles with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum DynamicModuleFailure {
    /// `HostLoadImportedModule` produced no module for some request in the
    /// target's graph. Loading the whole graph finishes before linking starts,
    /// so this wins over every link error in the same graph, and the host
    /// reports it as the `TypeError` it uses for an `import()` string that
    /// names no module.
    Load { message: String },
    /// `ParseModule` or linking threw its `SyntaxError`.
    Syntax { message: String },
}

impl DynamicModuleFailure {
    fn of(diagnostics: &[IrDiagnostic]) -> Self {
        match diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code() == Some(EarlyErrorCode::ModuleUnresolved))
        {
            Some(unresolved) => Self::Load {
                message: unresolved.message.clone(),
            },
            None => Self::Syntax {
                message: diagnostics[0].message.clone(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RejectedDynamicModule {
    pub(super) referrer: ModuleUnitId,
    pub(super) request: ModuleRequestIr,
    pub(super) stage: DynamicModuleRejectionStage,
    pub(super) failure: DynamicModuleFailure,
}

pub(crate) struct ModuleGraphRejection {
    pub(crate) graph: Option<ModuleGraphIr>,
    pub(crate) diagnostics: Vec<IrDiagnostic>,
}

fn link_sources(
    sources: &ModuleGraphSources,
    entry_is_script: bool,
) -> Result<ModuleGraphIr, ModuleGraphRejection> {
    let mut graph = build_graph(sources).map_err(|diagnostics| ModuleGraphRejection {
        graph: None,
        diagnostics,
    })?;
    graph.entry_is_script = entry_is_script;
    link(&mut graph);
    if graph.link_errors.is_empty() {
        Ok(graph)
    } else {
        let diagnostics = graph
            .link_errors
            .iter()
            .map(ModuleLinkErrorIr::to_diagnostic)
            .collect();
        Err(ModuleGraphRejection {
            graph: Some(graph),
            diagnostics,
        })
    }
}

/// Successful closures keep the ordinary single linking pass. A rejected graph
/// is partitioned only when its Module entry has the canonical execution
/// driver; parser aborts, host contradictions and implementation limits remain
/// compiler failures even if their source is reachable only dynamically.
pub(crate) fn link_loaded_graph(
    sources: &ModuleGraphSources,
    entry_is_script: bool,
) -> Result<ModuleGraphIr, ModuleGraphRejection> {
    let original = match link_sources(sources, entry_is_script) {
        Ok(graph) => return Ok(graph),
        Err(rejection) => rejection,
    };
    if sources.modules.get(sources.entry as usize).is_none()
        || !original.diagnostics.iter().all(can_reject_import)
        || !host_identity_is_consistent(sources)
    {
        return Err(original);
    }
    let mut records = Vec::with_capacity(sources.modules.len());
    for (index, source) in sources.modules.iter().enumerate() {
        let record = match &source.parse {
            ModuleParse::Module(parsed) => {
                match super::record::parse_module_record(parsed, index as u32, source.key().clone())
                {
                    Ok(record) => Some(record),
                    Err(diagnostics) if diagnostics.iter().all(can_reject_import) => None,
                    Err(_) => return Err(original),
                }
            }
            ModuleParse::Rejected { .. } => None,
            // The lowerer admits Script syntax only in a Script entry's slot.
            // Its record carries `import()` sites and no static requests, so
            // its static closure is itself.
            ModuleParse::ScriptEntry(parsed)
                if entry_is_script && index == sources.entry as usize =>
            {
                Some(super::record::script_entry_record(
                    parsed,
                    index as u32,
                    source.key().clone(),
                ))
            }
            ModuleParse::ScriptEntry(_) => return Err(original),
        };
        records.push(record);
    }
    // A static source-phase request keeps the retained driver; a dynamic one
    // rejects in the canonical dispatcher (see
    // `synchronous_source::ModuleInstantiationGraph`). Only admitted modules
    // are emitted, so a module that is reachable solely through a rejected
    // `import()` may still carry one.
    let has_static_source_request = |module: ModuleUnitId| {
        records[module as usize].as_ref().is_some_and(|record| {
            record
                .requested_modules
                .iter()
                .any(|request| request.phase() == ImportPhaseIr::Source)
        })
    };
    let closure = StaticClosure::new(sources, &records);
    let entry_members = closure.members(sources.entry);
    if entry_members
        .iter()
        .any(|module| has_static_source_request(*module))
    {
        return Err(original);
    }
    // Static imports, including defer, must load and link before any entry body.
    link_sources(
        &closure.project(&entry_members, sources.entry),
        entry_is_script,
    )?;
    let mut admitted = entry_members.clone();
    let mut pending = entry_members.into_iter().collect::<Vec<_>>();
    let mut visited = BTreeSet::new();
    let mut validations = BTreeMap::new();
    let mut rejected_requests = Vec::new();
    while let Some(referrer) = pending.pop() {
        if !visited.insert(referrer) {
            continue;
        }
        let record = records[referrer as usize]
            .as_ref()
            .expect("admitted source has a parsed record");
        // The same occurrences `dynamic::discover_components` registers: each
        // literal request, and for a computed call every key the host resolved
        // for this referrer, in that call's phase.
        let mut requests: Vec<ModuleRequestIr> = record
            .dynamic_import_sites
            .iter()
            .filter_map(DynamicImportSiteIr::discovery_request)
            .collect();
        for phase in super::record::computed_import_phases(&record.dynamic_import_sites) {
            for key in closure.request_keys(referrer) {
                let request = ModuleRequestIr::from_key(key.clone(), phase);
                if !requests.contains(&request) {
                    requests.push(request);
                }
            }
        }
        for request in requests {
            let Some(target) = closure.target(referrer, request.key()) else {
                continue;
            };
            let outcome = validations.entry(target).or_insert_with(|| {
                let members = closure.members(target);
                link_sources(&closure.project(&members, target), false).map(|_| members)
            });
            match outcome {
                Ok(members)
                    if members
                        .iter()
                        .any(|module| has_static_source_request(*module)) =>
                {
                    return Err(original);
                }
                Ok(members) => {
                    for module in members.iter().copied() {
                        if admitted.insert(module) {
                            pending.push(module);
                        }
                    }
                }
                Err(rejection) => {
                    if !rejection.diagnostics.iter().all(can_reject_import) {
                        return Err(original);
                    }
                    let stage = if records[target as usize].is_none() {
                        DynamicModuleRejectionStage::ModuleLoad
                    } else {
                        DynamicModuleRejectionStage::Dependencies
                    };
                    rejected_requests.push((
                        referrer,
                        request,
                        stage,
                        DynamicModuleFailure::of(&rejection.diagnostics),
                    ));
                }
            }
        }
    }
    let mut graph = link_sources(&closure.project(&admitted, sources.entry), entry_is_script)?;
    for (referrer, request, stage, failure) in rejected_requests {
        // A Script entry is outside the module map, so it is found by position.
        let referrer = if referrer == sources.entry {
            graph.entry
        } else {
            graph.keys[sources.modules[referrer as usize].key()]
        };
        if !graph
            .dynamic_rejections
            .iter()
            .any(|rejection| rejection.referrer == referrer && rejection.request == request)
        {
            graph.dynamic_rejections.push(RejectedDynamicModule {
                referrer,
                request,
                stage,
                failure,
            });
        }
    }
    Ok(graph)
}

fn can_reject_import(diagnostic: &IrDiagnostic) -> bool {
    diagnostic.error_type() == Some(NativeErrorKind::SyntaxError)
        && diagnostic.code().is_some_and(|code| {
            code.is_parse_classified()
                || matches!(
                    code,
                    EarlyErrorCode::ModuleSyntax
                        | EarlyErrorCode::ModuleUnresolved
                        | EarlyErrorCode::ModuleMissingExport
                        | EarlyErrorCode::ModuleAmbiguousExport
                )
        })
}

/// Module-map identity is keyed per goal: a Script entry shares its file's
/// key with a Module Record the host loads from that file without being the
/// same record.
fn host_identity_is_consistent(sources: &ModuleGraphSources) -> bool {
    let mut loaded = BTreeMap::new();
    for source in &sources.modules {
        if loaded
            .insert((source.goal() == ParseGoal::Script, source.key()), source)
            .is_some_and(|previous| !previous.same_loaded_source(source))
        {
            return false;
        }
    }
    let mut requests = BTreeMap::new();
    for (referrer, request, target) in &sources.resolutions {
        let (Some(referrer), Some(target)) = (
            sources.modules.get(*referrer as usize),
            sources.modules.get(*target as usize),
        ) else {
            continue;
        };
        if requests
            .insert(
                (
                    referrer.goal() == ParseGoal::Script,
                    referrer.key(),
                    request,
                ),
                target.key(),
            )
            .is_some_and(|previous| previous != target.key())
        {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_host_key_cannot_change_type_behind_equal_synthesized_source() {
        let key = ModuleKey::from_host("/root/data.bin");
        let bytes = ModuleSourceIr::bytes(
            key.clone(),
            vec![0, 0x80, 0xff],
            "file:///root/data.bin".to_string(),
        );
        let javascript = ModuleSourceIr::new(
            key,
            bytes.source_text().to_string(),
            bytes.meta_url().to_string(),
        );
        let sources = ModuleGraphSources {
            modules: vec![bytes.clone(), javascript],
            entry: 0,
            resolutions: vec![],
        };
        assert!(!host_identity_is_consistent(&sources));

        let consistent = ModuleGraphSources {
            modules: vec![bytes.clone(), bytes],
            entry: 0,
            resolutions: vec![],
        };
        assert!(host_identity_is_consistent(&consistent));
    }
}
