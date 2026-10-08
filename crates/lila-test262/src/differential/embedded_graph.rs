//! The v4 wire boundary carries one complete validated host-source graph.

use super::*;
use lila_engine::{
    EmbeddedModuleEntryInput, EmbeddedModuleInput, EmbeddedModuleKind,
    EmbeddedModuleResolutionInput, EmbeddedModuleSourceInput,
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct EmbeddedCaseWire {
    schema_version: u32,
    id: String,
    observation_contract: ObservationContract,
    timeout_ms: u64,
    module_graph: ModuleGraphWire,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ModuleGraphWire {
    entry: EntryWire,
    modules: Vec<ModuleWire>,
    resolutions: Vec<ResolutionWire>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EntryWire {
    goal: DifferentialGoal,
    identity: String,
    source: String,
    meta_url: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ModuleWire {
    #[serde(default, skip_serializing_if = "ModuleKindWire::is_source_text")]
    kind: ModuleKindWire,
    identity: String,
    source: String,
    meta_url: String,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ModuleKindWire {
    #[default]
    SourceText,
    Json,
}
impl ModuleKindWire {
    fn is_source_text(&self) -> bool {
        matches!(self, Self::SourceText)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ResolutionWire {
    referrer: ReferrerWire,
    specifier: String,
    attributes: Vec<(String, String)>,
    target: String,
}

#[derive(Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "identity",
    rename_all = "snake_case",
    deny_unknown_fields
)]
enum ReferrerWire {
    Script(String),
    Module(String),
    Unlocated,
    Realm,
}

impl EmbeddedCaseWire {
    pub(super) fn into_input(self) -> Result<DifferentialReplayInput, DifferentialError> {
        self.into_case().map(DifferentialReplayInput::from)
    }

    pub(super) fn into_case(self) -> Result<DifferentialCase, DifferentialError> {
        let protocol =
            DifferentialProtocol::from_wire(self.schema_version, self.observation_contract)?;
        if protocol != DifferentialProtocol::V4EmbeddedGraphPrimitivePrintTranscript {
            return Err(DifferentialError::InvalidCorpus(
                "embedded module_graph requires differential schema v4".into(),
            ));
        }
        let graph = self.module_graph.into_graph()?;
        DifferentialCase::new_embedded(self.id, self.timeout_ms, graph)
    }
}

impl ModuleGraphWire {
    pub(super) fn into_graph(self) -> Result<Arc<EmbeddedModuleGraph>, DifferentialError> {
        let entry = self.entry;
        EmbeddedModuleGraph::try_new_typed(
            EmbeddedModuleEntryInput {
                goal: match entry.goal {
                    DifferentialGoal::Script => EmbeddedModuleGoal::Script,
                    DifferentialGoal::Module => EmbeddedModuleGoal::Module,
                },
                identity: entry.identity,
                source: entry.source,
                meta_url: entry.meta_url,
            },
            self.modules
                .into_iter()
                .map(|module| {
                    let source = EmbeddedModuleSourceInput {
                        identity: module.identity,
                        source: module.source,
                        meta_url: module.meta_url,
                    };
                    match module.kind {
                        ModuleKindWire::SourceText => EmbeddedModuleInput::SourceText(source),
                        ModuleKindWire::Json => EmbeddedModuleInput::Json(source),
                    }
                })
                .collect(),
            self.resolutions
                .into_iter()
                .map(|resolution| EmbeddedModuleResolutionInput {
                    referrer: match resolution.referrer {
                        ReferrerWire::Script(identity) => EmbeddedModuleReferrer::Script(identity),
                        ReferrerWire::Module(identity) => EmbeddedModuleReferrer::Module(identity),
                        ReferrerWire::Unlocated => EmbeddedModuleReferrer::Unlocated,
                        ReferrerWire::Realm => EmbeddedModuleReferrer::Realm,
                    },
                    specifier: resolution.specifier,
                    attributes: resolution.attributes,
                    target: resolution.target,
                })
                .collect(),
        )
        .map_err(|error| DifferentialError::InvalidCorpus(error.to_string()))
    }
}

/// Serialization projects only the validated owner, never a second source or
/// goal. Dependency rows exclude the implicit Module entry exactly once.
pub(super) struct GraphProjection<'a>(pub(super) &'a EmbeddedModuleGraph);

impl Serialize for GraphProjection<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let entry = self.0.entry();
        ModuleGraphWire {
            entry: EntryWire {
                goal: match entry.goal() {
                    EmbeddedModuleGoal::Script => DifferentialGoal::Script,
                    EmbeddedModuleGoal::Module => DifferentialGoal::Module,
                },
                identity: entry.identity().into(),
                source: entry.source().into(),
                meta_url: entry.meta_url().into(),
            },
            modules: self
                .0
                .dependency_modules()
                .map(|module| ModuleWire {
                    kind: match module.kind() {
                        EmbeddedModuleKind::SourceText => ModuleKindWire::SourceText,
                        EmbeddedModuleKind::Json => ModuleKindWire::Json,
                    },
                    identity: module.identity().into(),
                    source: module.source().into(),
                    meta_url: module.meta_url().into(),
                })
                .collect(),
            resolutions: self
                .0
                .resolutions()
                .iter()
                .map(|resolution| ResolutionWire {
                    referrer: match resolution.referrer() {
                        EmbeddedModuleReferrer::Script(identity) => {
                            ReferrerWire::Script(identity.clone())
                        }
                        EmbeddedModuleReferrer::Module(identity) => {
                            ReferrerWire::Module(identity.clone())
                        }
                        EmbeddedModuleReferrer::Unlocated => ReferrerWire::Unlocated,
                        EmbeddedModuleReferrer::Realm => ReferrerWire::Realm,
                    },
                    specifier: resolution.request().specifier().into(),
                    attributes: resolution.request().attributes().to_vec(),
                    target: resolution.target().into(),
                })
                .collect(),
        }
        .serialize(serializer)
    }
}
