//! Preserve static rejection while assigning dynamic-only load failures to import jobs.

use super::loaded_sources::ModuleParse;
use super::*;
use crate::{EarlyErrorCode, IrDiagnostic, NativeErrorKind, ParseGoal};
use std::collections::{BTreeMap, BTreeSet};

mod closure;
use closure::StaticClosure;

#[cfg(test)]
mod tests;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum LoadedSourceIdentity<'a> {
    Script(&'a ModuleKey),
    Module(&'a ModuleKey),
}

fn source_identity(source: &ModuleSourceIr) -> LoadedSourceIdentity<'_> {
    match source.goal() {
        ParseGoal::Script => LoadedSourceIdentity::Script(source.key()),
        ParseGoal::Module => LoadedSourceIdentity::Module(source.key()),
    }
}

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

/// The caller supplies either a loaded closure or the complete declared host
/// catalog. Required at admission and component discovery so one cannot use
/// static discovery while the other admits computed request variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum GraphAdmission {
    #[default]
    LoadedClosure,
    CompleteCatalog,
}

impl GraphAdmission {
    pub(super) fn occurrences<'a>(
        self,
        site: &DynamicImportSiteIr,
        record: &ModuleRecordIr,
        keys: impl Iterator<Item = &'a ModuleRequestKeyIr>,
    ) -> Vec<ModuleRequestIr> {
        match self {
            Self::LoadedClosure if site.static_specifier.is_some() => {
                site.discovery_request().into_iter().collect()
            }
            Self::LoadedClosure => keys
                .filter(|key| record.module_resolution_requests.contains(key))
                .filter_map(|key| site.catalog_occurrence(key))
                .collect(),
            Self::CompleteCatalog => keys
                .filter_map(|key| site.catalog_occurrence(key))
                .collect(),
        }
    }
}

pub(crate) struct ModuleGraphRejection {
    pub(crate) graph: Option<ModuleGraphIr>,
    pub(crate) diagnostics: Vec<IrDiagnostic>,
}

fn link_sources(
    sources: &ModuleGraphSources,
    entry_is_script: bool,
    admission: GraphAdmission,
) -> Result<ModuleGraphIr, ModuleGraphRejection> {
    let built = match admission {
        GraphAdmission::LoadedClosure => build_graph(sources),
        GraphAdmission::CompleteCatalog => super::graph_build::build_complete_catalog(sources),
    };
    let mut graph = built.map_err(|diagnostics| ModuleGraphRejection {
        graph: None,
        diagnostics,
    })?;
    graph.entry_is_script = entry_is_script;
    super::graph::link_with_admission(&mut graph, admission);
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
/// is partitioned only when its entry has the canonical execution driver;
/// Script text retains its parse goal while dynamic targets stay Module code.
/// Parser aborts, host contradictions and implementation limits remain
/// compiler failures even if their source is reachable only dynamically.
pub(crate) fn link_loaded_graph(
    sources: &ModuleGraphSources,
    entry_is_script: bool,
) -> Result<ModuleGraphIr, ModuleGraphRejection> {
    link_admitted_graph(sources, entry_is_script, GraphAdmission::LoadedClosure)
}

pub(crate) fn link_complete_catalog(
    sources: &ModuleGraphSources,
    entry_is_script: bool,
) -> Result<ModuleGraphIr, ModuleGraphRejection> {
    link_admitted_graph(sources, entry_is_script, GraphAdmission::CompleteCatalog)
}

fn link_admitted_graph(
    sources: &ModuleGraphSources,
    entry_is_script: bool,
    admission: GraphAdmission,
) -> Result<ModuleGraphIr, ModuleGraphRejection> {
    let original = match admission {
        GraphAdmission::LoadedClosure => {
            let original = link_sources(sources, entry_is_script, admission);
            match &original {
                Ok(graph)
                    if !graph.units.iter().any(|unit| {
                        unit.record
                            .dynamic_import_sites
                            .iter()
                            .any(|site| site.phase == ImportPhaseIr::Source)
                    }) =>
                {
                    return original;
                }
                Err(rejection) if !rejection.diagnostics.iter().all(can_reject_import) => {
                    return original;
                }
                Ok(_) | Err(_) => {}
            }
            if sources.modules.get(sources.entry as usize).is_none()
                || !host_identity_is_consistent(sources)
            {
                return original;
            }
            Some(original)
        }
        GraphAdmission::CompleteCatalog => {
            if sources.modules.get(sources.entry as usize).is_none() {
                return Err(ModuleGraphRejection {
                    graph: None,
                    diagnostics: vec![IrDiagnostic::lowering(
                        "complete module catalog has no entry source",
                    )],
                });
            }
            if !host_identity_is_consistent(sources)
                || sources.resolutions.iter().any(|(referrer, _, target)| {
                    sources.modules.get(*referrer as usize).is_none()
                        || sources
                            .modules
                            .get(*target as usize)
                            .is_none_or(|source| source.goal() != ParseGoal::Module)
                })
            {
                return Err(ModuleGraphRejection {
                    graph: None,
                    diagnostics: vec![IrDiagnostic::lowering(
                        "complete module catalog has inconsistent host identities or resolution rows",
                    )],
                });
            }
            None
        }
    };
    // Retain each actual parse outcome. Unused catalog records do not become
    // compilation failures; a matched closure still retains parser aborts and
    // implementation limits as compiler failures, never import rejections.
    let mut records = Vec::with_capacity(sources.modules.len());
    for (index, source) in sources.modules.iter().enumerate() {
        let Ok(id) = ModuleUnitId::try_from(index) else {
            return Err(ModuleGraphRejection {
                graph: None,
                diagnostics: vec![IrDiagnostic::lowering(
                    "complete module source indices exceed the module id domain",
                )],
            });
        };
        let record = match &source.parse {
            ModuleParse::Json(parsed) => Ok(super::record::ModuleRecordIr::json(
                parsed,
                id,
                source.key().clone(),
            )),
            ModuleParse::JsonRejected { error, .. } => {
                Err(vec![super::early::json_parse_failure_diagnostic(error)])
            }
            ModuleParse::Module(parsed) => match admission {
                GraphAdmission::LoadedClosure => {
                    super::record::parse_module_record(parsed, id, source.key().clone())
                }
                GraphAdmission::CompleteCatalog => {
                    super::record::parse_module_record_for_catalog(parsed, id, source.key().clone())
                }
            },
            ModuleParse::Rejected { error, .. } => {
                Err(vec![super::early::module_parse_failure_diagnostic(error)])
            }
            ModuleParse::ScriptEntry(parsed) => match admission {
                GraphAdmission::LoadedClosure => {
                    super::record::script_entry_record(parsed, id, source.key().clone())
                }
                GraphAdmission::CompleteCatalog => {
                    super::record::script_entry_record_for_catalog(parsed, id, source.key().clone())
                }
            },
        };
        if admission == GraphAdmission::LoadedClosure
            && record
                .as_ref()
                .is_err_and(|diagnostics| !diagnostics.iter().all(can_reject_import))
        {
            return original.expect("loaded admission retained its original result");
        }
        // A rejected non-entry module fails while the graph loads/links.
        records.push(if index == sources.entry as usize {
            record
        } else {
            record.map_err(|diagnostics| {
                diagnostics
                    .into_iter()
                    .map(IrDiagnostic::in_dependency_module)
                    .collect()
            })
        });
    }
    let closure = StaticClosure::new(sources, &records, admission);
    let (entry_members, entry_execution) = match admission {
        GraphAdmission::LoadedClosure => (closure.members(sources.entry), BTreeSet::new()),
        GraphAdmission::CompleteCatalog => closure.members_and_execution(sources.entry),
    };
    // Static imports, including defer, link before any entry body. A source
    // request parses its record without opening outgoing requests.
    link_sources(
        &closure.project(&entry_members, sources.entry),
        entry_is_script,
        admission,
    )?;
    let mut admitted = entry_members.clone();
    let mut pending = match admission {
        GraphAdmission::LoadedClosure => entry_members.into_iter().collect::<Vec<_>>(),
        GraphAdmission::CompleteCatalog => entry_execution.into_iter().collect::<Vec<_>>(),
    };
    let mut visited = BTreeSet::new();
    let mut validations = BTreeMap::new();
    let mut rejected_requests = Vec::new();
    let mut rejected_realm_requests = BTreeMap::new();
    for (request, resolution) in &sources.realm_requests {
        let RealmModuleResolutionIr::Loaded(target) = resolution else {
            continue;
        };
        if sources
            .modules
            .get(*target as usize)
            .is_none_or(|source| source.goal() != ParseGoal::Module)
        {
            return Err(ModuleGraphRejection {
                graph: None,
                diagnostics: vec![IrDiagnostic::lowering(
                    "Realm import target must be a declared Module Record",
                )],
            });
        }
        let (members, execution) = match admission {
            GraphAdmission::LoadedClosure => (closure.members(*target), BTreeSet::new()),
            GraphAdmission::CompleteCatalog => closure.members_and_execution(*target),
        };
        match link_sources(&closure.project(&members, *target), false, admission) {
            Ok(_) => {
                admitted.extend(members.iter().copied());
                pending.extend(match admission {
                    GraphAdmission::LoadedClosure => members.into_iter().collect::<Vec<_>>(),
                    GraphAdmission::CompleteCatalog => execution.into_iter().collect::<Vec<_>>(),
                });
            }
            Err(rejection) if rejection.diagnostics.iter().all(can_reject_import) => {
                rejected_realm_requests.insert(
                    request.clone(),
                    RealmModuleResolutionIr::Rejected(rejection.diagnostics[0].message.clone()),
                );
            }
            Err(rejection) => return Err(rejection),
        }
    }
    while let Some(referrer) = pending.pop() {
        if !visited.insert(referrer) {
            continue;
        }
        let record = records[referrer as usize]
            .as_ref()
            .expect("admitted source has a parsed record");
        for site in &record.dynamic_import_sites {
            let keys = sources
                .resolutions
                .iter()
                .filter_map(|(owner, key, _)| (*owner == referrer).then_some(key));
            for request in admission.occurrences(site, record, keys) {
                let Some(target) = closure.target(referrer, request.key()) else {
                    continue;
                };
                if request.phase() == ImportPhaseIr::Source {
                    let message = match &records[target as usize] {
                        Ok(_) => ModuleLinkErrorIr::SourceUnavailable {
                            referrer,
                            request: request.clone(),
                        }
                        .message(),
                        Err(diagnostics) => {
                            if !diagnostics.iter().all(can_reject_import) {
                                return match admission {
                                    GraphAdmission::LoadedClosure => original
                                        .expect("loaded admission retained its original result"),
                                    GraphAdmission::CompleteCatalog => Err(ModuleGraphRejection {
                                        graph: None,
                                        diagnostics: diagnostics.clone(),
                                    }),
                                };
                            }
                            diagnostics[0].message.clone()
                        }
                    };
                    rejected_requests.push((
                        referrer,
                        request,
                        DynamicModuleRejectionStage::ModuleLoad,
                        message,
                    ));
                    continue;
                }
                let outcome = validations.entry(target).or_insert_with(|| {
                    let (members, execution) = match admission {
                        GraphAdmission::LoadedClosure => (closure.members(target), BTreeSet::new()),
                        GraphAdmission::CompleteCatalog => closure.members_and_execution(target),
                    };
                    link_sources(&closure.project(&members, target), false, admission)
                        .map(|_| (members, execution))
                });
                match outcome {
                    Ok((members, execution)) => {
                        match admission {
                            GraphAdmission::LoadedClosure => {
                                for module in members.iter().copied() {
                                    if admitted.insert(module) {
                                        pending.push(module);
                                    }
                                }
                            }
                            GraphAdmission::CompleteCatalog => {
                                admitted.extend(members.iter().copied());
                                // A record already retained by source phase may
                                // now be promoted to an executable occurrence.
                                pending.extend(execution.iter().copied());
                            }
                        }
                    }
                    Err(rejection) => {
                        if !rejection.diagnostics.iter().all(can_reject_import) {
                            return match admission {
                                GraphAdmission::LoadedClosure => {
                                    original.expect("loaded admission retained its original result")
                                }
                                GraphAdmission::CompleteCatalog => Err(ModuleGraphRejection {
                                    graph: rejection.graph.clone(),
                                    diagnostics: rejection.diagnostics.clone(),
                                }),
                            };
                        }
                        let stage = if records[target as usize].is_err() {
                            DynamicModuleRejectionStage::ModuleLoad
                        } else {
                            DynamicModuleRejectionStage::Dependencies
                        };
                        rejected_requests.push((
                            referrer,
                            request,
                            stage,
                            rejection.diagnostics[0].message.clone(),
                        ));
                    }
                }
            }
        }
    }
    let mut graph = link_sources(
        &closure.project(&admitted, sources.entry),
        entry_is_script,
        admission,
    )?;
    graph.realm_requests.extend(rejected_realm_requests);
    for (referrer, request, stage, message) in rejected_requests {
        let referrer = match source_identity(&sources.modules[referrer as usize]) {
            LoadedSourceIdentity::Script(_) => graph.entry,
            LoadedSourceIdentity::Module(key) => graph.keys[key],
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
                        | EarlyErrorCode::ModuleSourceUnavailable
                        | EarlyErrorCode::ModuleAmbiguousExport
                )
        })
}

fn host_identity_is_consistent(sources: &ModuleGraphSources) -> bool {
    let mut loaded = BTreeMap::new();
    for source in &sources.modules {
        if matches!(source_identity(source), LoadedSourceIdentity::Script(_)) {
            continue;
        }
        if loaded
            .insert(source.key(), (source.kind(), source.source_text()))
            .is_some_and(|previous| previous != (source.kind(), source.source_text()))
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
                (source_identity(referrer), request),
                source_identity(target),
            )
            .is_some_and(|previous| previous != source_identity(target))
        {
            return false;
        }
    }
    true
}
