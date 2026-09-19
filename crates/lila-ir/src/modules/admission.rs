//! Preserve static rejection while assigning dynamic-only load failures to import jobs.

use super::loaded_sources::ModuleParse;
use super::*;
use crate::{EarlyErrorCode, IrDiagnostic, NativeErrorKind};
use std::collections::{BTreeMap, BTreeSet};

mod closure;
use closure::StaticClosure;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DynamicModuleRejectionStage {
    ModuleLoad,
    Dependencies,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RejectedDynamicModule {
    pub(super) referrer: ModuleUnitId,
    pub(super) request: ModuleRequestIr,
    pub(super) stage: DynamicModuleRejectionStage,
    pub(super) message: String,
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
/// is partitioned only when its Module entry has the canonical synchronous
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
    if entry_is_script
        || sources.modules.get(sources.entry as usize).is_none()
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
            ModuleParse::ScriptEntry(_) => return Err(original),
        };
        if record.as_ref().is_some_and(|record| {
            record.has_top_level_await
                || record
                    .requested_modules
                    .iter()
                    .any(|request| request.phase() == ImportPhaseIr::Source)
                || record
                    .dynamic_import_sites
                    .iter()
                    .any(|site| site.phase == ImportPhaseIr::Source)
        }) {
            return Err(original);
        }
        records.push(record);
    }
    let closure = StaticClosure::new(sources, &records);
    let entry_members = closure.members(sources.entry);
    // Static imports, including defer, must load and link before any entry body.
    link_sources(&closure.project(&entry_members, sources.entry), false)?;
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
        for site in &record.dynamic_import_sites {
            let Some(request) = site.discovery_request() else {
                continue;
            };
            let Some(target) = closure.target(referrer, request.key()) else {
                continue;
            };
            let outcome = validations.entry(target).or_insert_with(|| {
                let members = closure.members(target);
                link_sources(&closure.project(&members, target), false).map(|_| members)
            });
            match outcome {
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
                        sources.modules[referrer as usize].key().clone(),
                        request,
                        stage,
                        rejection.diagnostics[0].message.clone(),
                    ));
                }
            }
        }
    }
    let mut graph = link_sources(&closure.project(&admitted, sources.entry), false)?;
    for (referrer, request, stage, message) in rejected_requests {
        let referrer = graph.keys[&referrer];
        if !graph
            .dynamic_rejections
            .iter()
            .any(|rejection| rejection.referrer == referrer && rejection.request == request)
        {
            graph.dynamic_rejections.push(RejectedDynamicModule {
                referrer,
                request,
                stage,
                message,
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

fn host_identity_is_consistent(sources: &ModuleGraphSources) -> bool {
    let mut loaded = BTreeMap::new();
    for source in &sources.modules {
        if loaded
            .insert(source.key(), source.source_text())
            .is_some_and(|previous| previous != source.source_text())
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
            .insert((referrer.key(), request), target.key())
            .is_some_and(|previous| previous != target.key())
        {
            return false;
        }
    }
    true
}
