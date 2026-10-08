//! V7 native input and comparison. No JavaScript wrapper participates in capture.

use super::*;

const PROTOCOL: DifferentialProtocol =
    DifferentialProtocol::V7Test262HostRootedCompletionGraphPrintTranscript;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Program {
    source: Source,
    limits: SnapshotLimits,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Source {
    Script(String),
    EmbeddedGraph(Arc<EmbeddedModuleGraph>),
}

impl Program {
    pub(super) fn goal(&self) -> DifferentialGoal {
        match &self.source {
            Source::Script(_) => DifferentialGoal::Script,
            Source::EmbeddedGraph(graph) => match graph.entry().goal() {
                EmbeddedModuleGoal::Script => DifferentialGoal::Script,
                EmbeddedModuleGoal::Module => DifferentialGoal::Module,
            },
        }
    }
    pub(super) fn source(&self) -> &str {
        match &self.source {
            Source::Script(source) => source,
            Source::EmbeddedGraph(graph) => graph.entry().source(),
        }
    }
    pub(super) fn module_graph(&self) -> Option<&Arc<EmbeddedModuleGraph>> {
        match &self.source {
            Source::Script(_) => None,
            Source::EmbeddedGraph(graph) => Some(graph),
        }
    }
}

/// Authority is carried in the native wire, never inferred from source or ID.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum HostProfile {
    Test262,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Wire {
    schema_version: u32,
    id: String,
    observation_contract: ObservationContract,
    host_profile: HostProfile,
    timeout_ms: u64,
    snapshot_limits: SnapshotLimits,
    program: ProgramWire,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum ProgramWire {
    Script {
        filename: String,
        source: String,
    },
    EmbeddedGraph {
        module_graph: embedded_graph::ModuleGraphWire,
    },
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum ProgramProjection<'a> {
    Script {
        filename: &'a str,
        source: &'a str,
    },
    EmbeddedGraph {
        module_graph: embedded_graph::GraphProjection<'a>,
    },
}

pub(super) fn missing_limits() -> DifferentialError {
    DifferentialError::InvalidCorpus(
        "differential schema v7 requires an explicit rooted snapshot program and checked snapshot_limits".into(),
    )
}

pub(super) fn from_json(json: &str) -> Result<DifferentialReplayInput, DifferentialError> {
    if json.len() > worker_process::MAX_REQUEST_BYTES {
        return Err(DifferentialError::WorkerConfiguration(
            "differential corpus exceeds the worker request budget".into(),
        ));
    }
    let wire: Wire = serde_json::from_str(json)
        .map_err(|error| DifferentialError::DecodeCorpus(error.to_string()))?;
    if DifferentialProtocol::from_wire(wire.schema_version, wire.observation_contract)? != PROTOCOL
    {
        return Err(DifferentialError::InvalidCorpus(
            "rooted snapshots require schema v7".into(),
        ));
    }
    let HostProfile::Test262 = wire.host_profile;
    match wire.program {
        ProgramWire::Script { filename, source } => DifferentialReplayInput::new_snapshot_script(
            wire.id,
            filename,
            wire.timeout_ms,
            source,
            wire.snapshot_limits,
        ),
        ProgramWire::EmbeddedGraph { module_graph } => {
            DifferentialReplayInput::new_snapshot_embedded(
                wire.id,
                wire.timeout_ms,
                module_graph.into_graph()?,
                wire.snapshot_limits,
            )
        }
    }
}

pub(super) fn serialize<S: Serializer>(
    id: &DifferentialCaseId,
    timeout_ms: NonZeroU64,
    filename: &str,
    program: &Program,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let mut wire = serializer.serialize_struct("DifferentialRootedSnapshotCase", 7)?;
    wire.serialize_field("schema_version", &PROTOCOL.schema_version())?;
    wire.serialize_field("id", id)?;
    wire.serialize_field("observation_contract", &PROTOCOL.observation_contract())?;
    wire.serialize_field("host_profile", &HostProfile::Test262)?;
    wire.serialize_field("timeout_ms", &timeout_ms)?;
    wire.serialize_field("snapshot_limits", &program.limits)?;
    let source = match &program.source {
        Source::Script(source) => ProgramProjection::Script { filename, source },
        Source::EmbeddedGraph(graph) => ProgramProjection::EmbeddedGraph {
            module_graph: embedded_graph::GraphProjection(graph),
        },
    };
    wire.serialize_field("program", &source)?;
    wire.end()
}

impl DifferentialReplayInput {
    pub fn new_snapshot_script(
        id: impl Into<String>,
        filename: impl Into<String>,
        timeout_ms: u64,
        source: impl Into<String>,
        limits: SnapshotLimits,
    ) -> Result<Self, DifferentialError> {
        let mut input = Self::new_script(
            id,
            DifferentialProtocol::V6Test262HostPrimitivePrintTranscript,
            filename,
            timeout_ms,
            source,
        )?;
        let ReplayProgramInput::Script(source) = input.program else {
            unreachable!("native Script constructor owns a Script")
        };
        input.program = ReplayProgramInput::RootedSnapshot(Program {
            source: Source::Script(source),
            limits,
        });
        input.protocol = PROTOCOL;
        Ok(input)
    }

    pub fn new_snapshot_embedded(
        id: impl Into<String>,
        timeout_ms: u64,
        graph: Arc<EmbeddedModuleGraph>,
        limits: SnapshotLimits,
    ) -> Result<Self, DifferentialError> {
        let mut input = Self::new_embedded(id, timeout_ms, Arc::clone(&graph))?;
        input.program = ReplayProgramInput::RootedSnapshot(Program {
            source: Source::EmbeddedGraph(graph),
            limits,
        });
        input.protocol = PROTOCOL;
        Ok(input)
    }

    pub fn snapshot_limits(&self) -> Option<SnapshotLimits> {
        match &self.program {
            ReplayProgramInput::RootedSnapshot(program) => Some(program.limits),
            ReplayProgramInput::Script(_) | ReplayProgramInput::EmbeddedGraph(_) => None,
        }
    }

    pub(super) fn admit_snapshot(self) -> Result<DifferentialCase, DifferentialError> {
        let ReplayProgramInput::RootedSnapshot(program) = self.program else {
            return Err(missing_limits());
        };
        let mut case = match program.source {
            Source::Script(source) => DifferentialCase::new(
                self.id.0,
                DifferentialGoal::Script,
                DifferentialProtocol::V6Test262HostPrimitivePrintTranscript,
                self.filename,
                self.timeout_ms.get(),
                source,
            )?,
            Source::EmbeddedGraph(graph) => {
                DifferentialCase::new_embedded(self.id.0, self.timeout_ms.get(), graph)?
            }
        };
        let source = match case.program {
            DifferentialProgram::DependencySealedScript(source) => Source::Script(source),
            DifferentialProgram::EmbeddedGraph(graph) => Source::EmbeddedGraph(graph),
            DifferentialProgram::RootedSnapshot(_) => {
                unreachable!("source admission does not create snapshots")
            }
        };
        case.program = DifferentialProgram::RootedSnapshot(Program {
            source,
            limits: program.limits,
        });
        case.protocol = PROTOCOL;
        Ok(case)
    }
}

impl DifferentialCase {
    /// Admit the original Script once, without wrapping its completion.
    pub fn new_snapshot_script(
        id: impl Into<String>,
        filename: impl Into<String>,
        timeout_ms: u64,
        source: impl Into<String>,
        limits: SnapshotLimits,
    ) -> Result<Self, DifferentialError> {
        DifferentialReplayInput::new_snapshot_script(id, filename, timeout_ms, source, limits)?
            .admit_snapshot()
    }

    pub fn new_snapshot_embedded(
        id: impl Into<String>,
        timeout_ms: u64,
        graph: Arc<EmbeddedModuleGraph>,
        limits: SnapshotLimits,
    ) -> Result<Self, DifferentialError> {
        DifferentialReplayInput::new_snapshot_embedded(id, timeout_ms, graph, limits)?
            .admit_snapshot()
    }

    pub fn snapshot_limits(&self) -> Option<SnapshotLimits> {
        match &self.program {
            DifferentialProgram::RootedSnapshot(program) => Some(program.limits),
            DifferentialProgram::DependencySealedScript(_)
            | DifferentialProgram::EmbeddedGraph(_) => None,
        }
    }
}

#[cfg(any(test, feature = "spec-exec-oracle"))]
pub(super) fn terminal_limits_match(
    input: &DifferentialReplayInput,
    execution: &ExecutionObservation,
) -> bool {
    match execution {
        ExecutionObservation::RootedCompletionGraph { completion, .. } => match &completion.outcome
        {
            SnapshotOutcome::Captured { graph } => Some(graph.limits()) == input.snapshot_limits(),
            SnapshotOutcome::Rejected { .. } => input.snapshot_limits().is_some(),
        },
        ExecutionObservation::Normal { .. }
        | ExecutionObservation::Error { .. }
        | ExecutionObservation::PrimitiveCompletion { .. }
        | ExecutionObservation::SelectedObjectProbe { .. }
        | ExecutionObservation::ObservationRejected { .. }
        | ExecutionObservation::UnsupportedCompletion { .. }
        | ExecutionObservation::EngineFailure { .. }
        | ExecutionObservation::WorkerFailure { .. } => true,
    }
}

#[cfg(any(test, feature = "spec-exec-oracle"))]
pub(super) fn compare(
    case: &DifferentialReplayInput,
    wasm: &BackendObservation,
    spec: &BackendObservation,
) -> DifferentialVerdict {
    for execution in [&wasm.execution, &spec.execution] {
        if let ExecutionObservation::RootedCompletionGraph { completion, .. } = execution {
            match &completion.outcome {
                SnapshotOutcome::Rejected { .. } => {
                    return DifferentialVerdict::ObservationContractViolated
                }
                SnapshotOutcome::Captured { graph }
                    if Some(graph.limits()) != case.snapshot_limits() =>
                {
                    return DifferentialVerdict::ObservationContractViolated;
                }
                SnapshotOutcome::Captured { .. } => {}
            }
        }
    }
    match (&wasm.execution, &spec.execution) {
        (
            ExecutionObservation::RootedCompletionGraph {
                completion: left, ..
            },
            ExecutionObservation::RootedCompletionGraph {
                completion: right, ..
            },
        ) => {
            if left == right && wasm.output_events == spec.output_events {
                DifferentialVerdict::RootedCompletionGraphAndPrintTranscriptMatch
            } else {
                DifferentialVerdict::Mismatch
            }
        }
        (
            ExecutionObservation::EngineFailure { .. },
            ExecutionObservation::EngineFailure { .. },
        ) => DifferentialVerdict::BothFailed,
        (
            ExecutionObservation::RootedCompletionGraph { .. },
            ExecutionObservation::EngineFailure { .. },
        )
        | (
            ExecutionObservation::EngineFailure { .. },
            ExecutionObservation::RootedCompletionGraph { .. },
        ) => DifferentialVerdict::Mismatch,
        (
            ExecutionObservation::Normal { .. }
            | ExecutionObservation::Error { .. }
            | ExecutionObservation::PrimitiveCompletion { .. }
            | ExecutionObservation::SelectedObjectProbe { .. }
            | ExecutionObservation::ObservationRejected { .. }
            | ExecutionObservation::UnsupportedCompletion { .. }
            | ExecutionObservation::WorkerFailure { .. },
            _,
        )
        | (
            _,
            ExecutionObservation::Normal { .. }
            | ExecutionObservation::Error { .. }
            | ExecutionObservation::PrimitiveCompletion { .. }
            | ExecutionObservation::SelectedObjectProbe { .. }
            | ExecutionObservation::ObservationRejected { .. }
            | ExecutionObservation::UnsupportedCompletion { .. }
            | ExecutionObservation::WorkerFailure { .. },
        ) => DifferentialVerdict::ObservationContractViolated,
    }
}

#[cfg(any(test, feature = "spec-exec-oracle"))]
pub(super) fn signature(observation: &BackendObservation) -> String {
    use sha2::{Digest, Sha256};
    let mut digest = Sha256::new();
    digest.update(b"lila-diff-v7-rooted-completion-and-print\0");
    match &observation.execution {
        ExecutionObservation::RootedCompletionGraph { completion, .. } => {
            match &completion.outcome {
                SnapshotOutcome::Captured { .. } => digest.update(
                    serde_json::to_vec(completion).expect("snapshot completion serializes"),
                ),
                SnapshotOutcome::Rejected { .. } => {
                    digest.update(b"rejected-outside-mismatch-domain")
                }
            }
        }
        ExecutionObservation::EngineFailure { phase, .. } => {
            digest.update(phase.as_str().as_bytes())
        }
        ExecutionObservation::Normal { .. }
        | ExecutionObservation::Error { .. }
        | ExecutionObservation::PrimitiveCompletion { .. }
        | ExecutionObservation::SelectedObjectProbe { .. }
        | ExecutionObservation::ObservationRejected { .. }
        | ExecutionObservation::UnsupportedCompletion { .. }
        | ExecutionObservation::WorkerFailure { .. } => digest.update(b"outside-v7-contract"),
    }
    digest.update(
        serde_json::to_vec(&observation.output_events).expect("output observation serializes"),
    );
    format!("sha256:{:x}", digest.finalize())
}

#[cfg(test)]
mod tests;
