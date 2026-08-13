//! Deterministic differential-corpus replay.
//!
//! This is intentionally a narrow first observation protocol. The engine now
//! exposes structured normal and thrown values plus per-run host output for
//! both backends, but report schema v1 consumes only projected disposition and
//! output emptiness. A v1 corpus program therefore remains a self-checking,
//! no-output program: normal completion means its assertions held, and abrupt
//! completion means they did not (or that the backend could not execute the
//! probe). The report records that narrow comparison and the remaining
//! capability gaps; it never calls equal disposition full semantic
//! equivalence.

use std::fmt;
use std::fs;
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};
#[cfg(feature = "spec-exec-oracle")]
use std::sync::{Arc, Mutex};

use lila_engine::CompileOptions;
#[cfg(feature = "spec-exec-oracle")]
use lila_engine::{
    Engine, EngineError, ExecutionBackend, HostOutputEvent, ObservedCompletion, RealmBuilder,
    RunOptions,
};
#[cfg(feature = "spec-exec-oracle")]
use lila_ir::IrDiagnosticPhase;
use serde::{Deserialize, Serialize};

mod generated_arithmetic;

pub use generated_arithmetic::{
    run_generated_arithmetic_campaign, ArithmeticCheckCount, ArithmeticExpressionDepth,
    ArithmeticGenerationPlan, ArithmeticGenerationSeed, ArithmeticReductionLimit,
    ArithmeticReductionStop, ArithmeticReductionSummary, GeneratedArithmeticCampaignOutcome,
    MAX_ARITHMETIC_CHECKS, MAX_ARITHMETIC_REDUCTION_REPLAYS,
};

pub const DIFFERENTIAL_CORPUS_SCHEMA_VERSION: u32 = 1;
pub const DIFFERENTIAL_REPORT_SCHEMA_VERSION: u32 = 1;

/// A corpus key with a stable, path-like machine spelling.
///
/// Colons and equals signs are excluded because the key is embedded verbatim
/// in the v1 mismatch signature.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct DifferentialCaseId(String);

impl DifferentialCaseId {
    pub fn new(value: impl Into<String>) -> Result<Self, DifferentialError> {
        let value = value.into();
        let valid = !value.is_empty()
            && !value.starts_with('/')
            && !value.ends_with('/')
            && value.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"-_/".contains(&byte)
            })
            && value.split('/').all(|part| !part.is_empty());
        if !valid {
            return Err(DifferentialError::InvalidCorpus(
                "case id must be a non-empty relative path of lowercase ASCII letters, digits, '-', and '_'"
                    .to_string(),
            ));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DifferentialGoal {
    Script,
    Module,
}

impl DifferentialGoal {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Script => "script",
            Self::Module => "module",
        }
    }
}

/// The only observation protocol admitted by corpus schema v1.
///
/// Programs in this protocol must not call a host output hook. They encode
/// their assertions in the source and complete normally only when all of them
/// hold. Both backends expose per-run output through the engine observation
/// API, so replay rejects the case contract when either transcript is nonempty.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationContract {
    SelfCheckingNoOutput,
}

impl ObservationContract {
    const fn as_str(self) -> &'static str {
        match self {
            Self::SelfCheckingNoOutput => "self_checking_no_output",
        }
    }
}

/// One deterministic input to differential replay.
///
/// Fields are private so callers cannot manufacture an unsupported schema
/// version, zero timeout, unstable filename, or malformed case key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DifferentialCase {
    schema_version: u32,
    id: DifferentialCaseId,
    goal: DifferentialGoal,
    observation_contract: ObservationContract,
    filename: String,
    timeout_ms: NonZeroU64,
    source: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DifferentialCaseWire {
    schema_version: u32,
    id: String,
    goal: DifferentialGoal,
    observation_contract: ObservationContract,
    filename: String,
    timeout_ms: u64,
    source: String,
}

impl DifferentialCase {
    pub fn new(
        id: impl Into<String>,
        goal: DifferentialGoal,
        observation_contract: ObservationContract,
        filename: impl Into<String>,
        timeout_ms: u64,
        source: impl Into<String>,
    ) -> Result<Self, DifferentialError> {
        let filename = filename.into();
        let stable_filename = !filename.is_empty()
            && !filename.starts_with('/')
            && !filename.contains('\\')
            && filename.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"-_/.".contains(&byte)
            })
            && filename
                .split('/')
                .all(|component| !matches!(component, "" | "." | ".."));
        if !stable_filename {
            return Err(DifferentialError::InvalidCorpus(
                "filename must be a normalized relative '/' path of lowercase ASCII letters, digits, '-', '_', and '.'"
                    .to_string(),
            ));
        }
        let timeout_ms = NonZeroU64::new(timeout_ms).ok_or_else(|| {
            DifferentialError::InvalidCorpus("timeout_ms must be non-zero".to_string())
        })?;
        let source = source.into();
        if source.is_empty() {
            return Err(DifferentialError::InvalidCorpus(
                "source must not be empty".to_string(),
            ));
        }
        Ok(Self {
            schema_version: DIFFERENTIAL_CORPUS_SCHEMA_VERSION,
            id: DifferentialCaseId::new(id)?,
            goal,
            observation_contract,
            filename,
            timeout_ms,
            source,
        })
    }

    pub fn from_json(json: &str) -> Result<Self, DifferentialError> {
        let wire: DifferentialCaseWire = serde_json::from_str(json)
            .map_err(|error| DifferentialError::DecodeCorpus(error.to_string()))?;
        if wire.schema_version != DIFFERENTIAL_CORPUS_SCHEMA_VERSION {
            return Err(DifferentialError::InvalidCorpus(format!(
                "unsupported differential corpus schema_version {}; expected {}",
                wire.schema_version, DIFFERENTIAL_CORPUS_SCHEMA_VERSION
            )));
        }
        Self::new(
            wire.id,
            wire.goal,
            wire.observation_contract,
            wire.filename,
            wire.timeout_ms,
            wire.source,
        )
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, DifferentialError> {
        let path = path.as_ref();
        let json = fs::read_to_string(path).map_err(|error| DifferentialError::ReadCorpus {
            path: path.to_path_buf(),
            message: error.to_string(),
        })?;
        Self::from_json(&json).map_err(|error| DifferentialError::CorpusAtPath {
            path: path.to_path_buf(),
            source: Box::new(error),
        })
    }

    pub fn id(&self) -> &DifferentialCaseId {
        &self.id
    }

    pub const fn goal(&self) -> DifferentialGoal {
        self.goal
    }

    pub const fn observation_contract(&self) -> ObservationContract {
        self.observation_contract
    }

    pub fn filename(&self) -> &str {
        &self.filename
    }

    pub const fn timeout_ms(&self) -> NonZeroU64 {
        self.timeout_ms
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn to_pretty_json(&self) -> Result<String, DifferentialError> {
        let mut json = serde_json::to_string_pretty(self)
            .map_err(|error| DifferentialError::EncodeCorpus(error.to_string()))?;
        json.push('\n');
        Ok(json)
    }
}

/// Capability token required by the replay API.
///
/// It has no `Default`: both the API caller and the CLI parser must explicitly
/// request the developer-only spec-exec oracle. The cargo feature remains a
/// second, independent gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpecExecOracle(());

impl SpecExecOracle {
    pub const fn explicitly_enabled() -> Self {
        Self(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DifferentialBackend {
    WasmAot,
    SpecExec,
}

impl DifferentialBackend {
    #[cfg(feature = "spec-exec-oracle")]
    const fn execution_backend(self) -> ExecutionBackend {
        match self {
            Self::WasmAot => ExecutionBackend::WasmAot,
            Self::SpecExec => ExecutionBackend::SpecExec,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionDisposition {
    Normal,
    Error,
}

impl ExecutionDisposition {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FailurePhase {
    Parse,
    EarlyError,
    ModuleResolution,
    Lowering,
    WasmRuntimeCapability,
    WasmRuntimeOrBackend,
    SpecExecExecution,
    RunnerInvariant,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "disposition", rename_all = "snake_case")]
pub enum ExecutionObservation {
    Normal {
        /// Backend diagnostic text, not a structured ECMAScript result.
        backend_note: String,
    },
    Error {
        phase: FailurePhase,
        /// Raw backend text retained for triage but excluded from the stable
        /// mismatch signature because it may contain paths or heap handles.
        message: String,
    },
}

impl ExecutionObservation {
    pub const fn disposition(&self) -> ExecutionDisposition {
        match self {
            Self::Normal { .. } => ExecutionDisposition::Normal,
            Self::Error { .. } => ExecutionDisposition::Error,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BackendObservation {
    pub backend: DifferentialBackend,
    pub output_events: OutputEventsObservation,
    pub execution: ExecutionObservation,
}

/// Report-v1 reason vocabulary. The existing reason remains part of the public
/// schema even though current replay can capture spec-exec output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputUnavailableReason {
    SpecExecBypassesEngineHostHooks,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "availability", rename_all = "snake_case")]
pub enum OutputEventsObservation {
    Captured { events: Vec<String> },
    Unavailable { reason: OutputUnavailableReason },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComparedDimension {
    SelfCheckDisposition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationGap {
    SpecExecOutputEventsUnavailable,
    UnstructuredNormalValue,
    UnstructuredCompletionKind,
    UnstructuredThrownValue,
    UncapturedErrorRealm,
    UncapturedPropertyDescriptors,
    UncapturedOwnKeyOrder,
    UncapturedPrototypeIdentity,
    UncapturedSideEffectLog,
    UnisolatedPanicAndHostCrash,
    SpecExecTimeoutNotEnforced,
}

pub const COMPARED_DIMENSIONS: [ComparedDimension; 1] = [ComparedDimension::SelfCheckDisposition];
pub const OBSERVATION_GAPS: [ObservationGap; 10] = [
    ObservationGap::UnstructuredNormalValue,
    ObservationGap::UnstructuredCompletionKind,
    ObservationGap::UnstructuredThrownValue,
    ObservationGap::UncapturedErrorRealm,
    ObservationGap::UncapturedPropertyDescriptors,
    ObservationGap::UncapturedOwnKeyOrder,
    ObservationGap::UncapturedPrototypeIdentity,
    ObservationGap::UncapturedSideEffectLog,
    ObservationGap::UnisolatedPanicAndHostCrash,
    ObservationGap::SpecExecTimeoutNotEnforced,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DifferentialVerdict {
    BothCompleted,
    BothFailed,
    Mismatch,
    ObservationContractViolated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticEquivalence {
    NotEstablished,
}

/// Non-cryptographic, versioned drift fingerprint for one exact corpus case.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct CaseFingerprint(String);

impl CaseFingerprint {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct MismatchSignature(String);

impl MismatchSignature {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DifferentialReport {
    schema_version: u32,
    case_id: DifferentialCaseId,
    case_fingerprint: CaseFingerprint,
    observation_contract: ObservationContract,
    verdict: DifferentialVerdict,
    /// Always `not_established` in schema v1. Matching the one observed
    /// disposition is deliberately not promoted to semantic equivalence.
    semantic_equivalence: SemanticEquivalence,
    compared_dimensions: [ComparedDimension; COMPARED_DIMENSIONS.len()],
    observation_gaps: [ObservationGap; OBSERVATION_GAPS.len()],
    wasm_aot: BackendObservation,
    spec_exec: BackendObservation,
    mismatch_signature: Option<MismatchSignature>,
}

impl DifferentialReport {
    pub const fn verdict(&self) -> DifferentialVerdict {
        self.verdict
    }

    pub const fn is_green(&self) -> bool {
        matches!(self.verdict, DifferentialVerdict::BothCompleted)
    }

    pub const fn semantic_equivalence(&self) -> SemanticEquivalence {
        self.semantic_equivalence
    }

    pub const fn compared_dimensions(&self) -> &[ComparedDimension; COMPARED_DIMENSIONS.len()] {
        &self.compared_dimensions
    }

    pub const fn observation_gaps(&self) -> &[ObservationGap; OBSERVATION_GAPS.len()] {
        &self.observation_gaps
    }

    pub fn wasm_aot(&self) -> &BackendObservation {
        &self.wasm_aot
    }

    pub fn spec_exec(&self) -> &BackendObservation {
        &self.spec_exec
    }

    pub fn mismatch_signature(&self) -> Option<&MismatchSignature> {
        self.mismatch_signature.as_ref()
    }

    pub fn to_pretty_json(&self) -> Result<String, DifferentialError> {
        serde_json::to_string_pretty(self)
            .map_err(|error| DifferentialError::EncodeReport(error.to_string()))
    }
}

#[derive(Debug)]
pub enum DifferentialError {
    ReadCorpus {
        path: PathBuf,
        message: String,
    },
    DecodeCorpus(String),
    InvalidCorpus(String),
    CorpusAtPath {
        path: PathBuf,
        source: Box<DifferentialError>,
    },
    InvalidGeneration(String),
    GeneratorInvariant(String),
    OracleNotLinked,
    EncodeCorpus(String),
    EncodeReport(String),
}

impl fmt::Display for DifferentialError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadCorpus { path, message } => {
                write!(formatter, "failed to read {}: {message}", path.display())
            }
            Self::DecodeCorpus(message) => {
                write!(formatter, "invalid differential corpus JSON: {message}")
            }
            Self::InvalidCorpus(message) => formatter.write_str(message),
            Self::CorpusAtPath { path, source } => {
                write!(
                    formatter,
                    "invalid differential case {}: {source}",
                    path.display()
                )
            }
            Self::InvalidGeneration(message) => formatter.write_str(message),
            Self::GeneratorInvariant(message) => {
                write!(
                    formatter,
                    "differential generator invariant failed: {message}"
                )
            }
            Self::OracleNotLinked => formatter.write_str(
                "spec-exec differential oracle is not linked; rebuild lila-cli with \
                 `--features spec-exec-oracle` and request it explicitly with \
                 `--oracle spec-exec`",
            ),
            Self::EncodeCorpus(message) => {
                write!(
                    formatter,
                    "failed to encode differential corpus case: {message}"
                )
            }
            Self::EncodeReport(message) => {
                write!(formatter, "failed to encode differential report: {message}")
            }
        }
    }
}

impl std::error::Error for DifferentialError {}

/// Replays one validated case through Wasm-AOT and the explicitly requested
/// spec-exec oracle, in that order and in fresh realms.
///
/// In builds without `spec-exec-oracle`, no backend is executed and a typed
/// `OracleNotLinked` error is returned.
#[cfg(feature = "spec-exec-oracle")]
pub fn replay_case(
    case: &DifferentialCase,
    _oracle: SpecExecOracle,
) -> Result<DifferentialReport, DifferentialError> {
    let wasm_aot = execute_case(case, DifferentialBackend::WasmAot);
    let spec_exec = execute_case(case, DifferentialBackend::SpecExec);
    Ok(compare_observations(case, wasm_aot, spec_exec))
}

#[cfg(not(feature = "spec-exec-oracle"))]
pub fn replay_case(
    _case: &DifferentialCase,
    _oracle: SpecExecOracle,
) -> Result<DifferentialReport, DifferentialError> {
    Err(DifferentialError::OracleNotLinked)
}

#[cfg(feature = "spec-exec-oracle")]
#[derive(Debug)]
struct CapturingOutput {
    events: Arc<Mutex<Vec<String>>>,
}

#[cfg(feature = "spec-exec-oracle")]
impl lila_engine::HostHooks for CapturingOutput {
    fn print_line(&self, text: &str) {
        self.events
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(text.to_string());
    }
}

#[cfg(feature = "spec-exec-oracle")]
fn execute_case(case: &DifferentialCase, backend: DifferentialBackend) -> BackendObservation {
    let captured_output = Arc::new(Mutex::new(Vec::new()));
    let engine = Engine::new(
        RealmBuilder::new()
            .with_host_hooks(Box::new(CapturingOutput {
                events: Arc::clone(&captured_output),
            }))
            .build(),
    );
    let compile = compile_options_for_case(case);
    let run = RunOptions {
        backend: backend.execution_backend(),
        test_path: Some(case.filename.clone()),
        can_block: false,
        timeout_ms: match backend {
            DifferentialBackend::WasmAot => Some(case.timeout_ms.get()),
            DifferentialBackend::SpecExec => None,
        },
        ..RunOptions::default()
    };
    let outcome = match case.goal {
        DifferentialGoal::Script => engine.observe_script(&case.source, compile, run),
        DifferentialGoal::Module => engine.observe_module(&case.source, compile, run),
    };
    let (execution, output_events) = match outcome {
        Ok(outcome) if outcome.backend_used == backend.execution_backend() => {
            let execution = match outcome.completion {
                ObservedCompletion::Normal(_) => ExecutionObservation::Normal {
                    backend_note: outcome.note,
                },
                ObservedCompletion::Throw(_) => ExecutionObservation::Error {
                    phase: execution_failure_phase(backend),
                    message: outcome.note,
                },
            };
            (execution, captured_output_events(outcome.output_events))
        }
        Ok(outcome) => {
            let output_events = captured_output_events(outcome.output_events);
            (
                ExecutionObservation::Error {
                    phase: FailurePhase::RunnerInvariant,
                    message: format!(
                        "requested backend {} reported backend {}",
                        backend.execution_backend().as_str(),
                        outcome.backend_used.as_str()
                    ),
                },
                output_events,
            )
        }
        Err(error) => (
            observe_engine_error(backend, &error),
            // The observation envelope deliberately keeps EngineError separate
            // from ECMAScript completion and therefore cannot own partial
            // events. The realm hook shadows the same print channel so this
            // branch can still report every event emitted before the failure.
            OutputEventsObservation::Captured {
                events: take_captured_output(&captured_output),
            },
        ),
    };
    BackendObservation {
        backend,
        output_events,
        execution,
    }
}

#[cfg(feature = "spec-exec-oracle")]
fn captured_output_events(events: Vec<HostOutputEvent>) -> OutputEventsObservation {
    OutputEventsObservation::Captured {
        events: events
            .into_iter()
            .map(|event| match event {
                HostOutputEvent::PrintLine(text) => text,
            })
            .collect(),
    }
}

#[cfg(feature = "spec-exec-oracle")]
fn take_captured_output(output: &Arc<Mutex<Vec<String>>>) -> Vec<String> {
    let mut events = output
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    std::mem::take(&mut *events)
}

#[cfg(feature = "spec-exec-oracle")]
const fn execution_failure_phase(backend: DifferentialBackend) -> FailurePhase {
    match backend {
        DifferentialBackend::WasmAot => FailurePhase::WasmRuntimeOrBackend,
        DifferentialBackend::SpecExec => FailurePhase::SpecExecExecution,
    }
}

/// Schema-v1 differential programs are product probes, not Test262 harness
/// programs. Keep the authority choice at this boundary so replay cannot gain
/// conformance-only globals as an incidental engine-test convenience.
fn compile_options_for_case(case: &DifferentialCase) -> CompileOptions {
    CompileOptions {
        filename: Some(case.filename.clone()),
        ..CompileOptions::default()
    }
}

#[cfg(feature = "spec-exec-oracle")]
fn observe_engine_error(backend: DifferentialBackend, error: &EngineError) -> ExecutionObservation {
    let phase = if error.parse_diagnostic().is_some() {
        FailurePhase::Parse
    } else if let Some(diagnostic) = error.ir_diagnostic() {
        match diagnostic.phase() {
            IrDiagnosticPhase::Early => FailurePhase::EarlyError,
            IrDiagnosticPhase::Resolution => FailurePhase::ModuleResolution,
            IrDiagnosticPhase::Lowering => FailurePhase::Lowering,
        }
    } else if error.wasm_gc_capability().is_some() {
        FailurePhase::WasmRuntimeCapability
    } else {
        execution_failure_phase(backend)
    };
    ExecutionObservation::Error {
        phase,
        message: error.message().to_string(),
    }
}

fn compare_observations(
    case: &DifferentialCase,
    wasm_aot: BackendObservation,
    spec_exec: BackendObservation,
) -> DifferentialReport {
    let wasm_disposition = wasm_aot.execution.disposition();
    let spec_disposition = spec_exec.execution.disposition();
    let obeys_no_output_contract =
        |observation: &BackendObservation| match &observation.output_events {
            OutputEventsObservation::Captured { events } => events.is_empty(),
            OutputEventsObservation::Unavailable { .. } => false,
        };
    let verdict = if obeys_no_output_contract(&wasm_aot) && obeys_no_output_contract(&spec_exec) {
        match (wasm_disposition, spec_disposition) {
            (ExecutionDisposition::Normal, ExecutionDisposition::Normal) => {
                DifferentialVerdict::BothCompleted
            }
            (ExecutionDisposition::Error, ExecutionDisposition::Error) => {
                DifferentialVerdict::BothFailed
            }
            (ExecutionDisposition::Normal, ExecutionDisposition::Error)
            | (ExecutionDisposition::Error, ExecutionDisposition::Normal) => {
                DifferentialVerdict::Mismatch
            }
        }
    } else {
        DifferentialVerdict::ObservationContractViolated
    };
    let case_fingerprint = case_fingerprint(case);
    let mismatch_signature = matches!(verdict, DifferentialVerdict::Mismatch).then(|| {
        MismatchSignature(format!(
            "lila-diff-v1:self-check-disposition:{}:{}:{}:wasm-aot={}:spec-exec={}",
            case.id.as_str(),
            case_fingerprint.as_str(),
            case.goal.as_str(),
            wasm_disposition.as_str(),
            spec_disposition.as_str(),
        ))
    });
    DifferentialReport {
        schema_version: DIFFERENTIAL_REPORT_SCHEMA_VERSION,
        case_id: case.id.clone(),
        case_fingerprint,
        observation_contract: case.observation_contract,
        verdict,
        semantic_equivalence: SemanticEquivalence::NotEstablished,
        compared_dimensions: COMPARED_DIMENSIONS,
        observation_gaps: OBSERVATION_GAPS,
        wasm_aot,
        spec_exec,
        mismatch_signature,
    }
}

fn case_fingerprint(case: &DifferentialCase) -> CaseFingerprint {
    const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;

    fn update(mut hash: u64, bytes: &[u8]) -> u64 {
        const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
        for byte in bytes {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(FNV_PRIME);
        }
        hash
    }

    fn field(hash: u64, value: &[u8]) -> u64 {
        let hash = update(hash, &(value.len() as u64).to_le_bytes());
        update(hash, value)
    }

    let mut hash = update(FNV_OFFSET_BASIS, b"lila-differential-case-v1");
    hash = field(hash, case.goal.as_str().as_bytes());
    hash = field(hash, case.observation_contract.as_str().as_bytes());
    hash = field(hash, case.filename.as_bytes());
    hash = field(hash, &case.timeout_ms.get().to_le_bytes());
    hash = field(hash, case.source.as_bytes());
    CaseFingerprint(format!("fnv1a64:{hash:016x}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use lila_ir::HostSurfacePolicy;

    const FOUNDATION_CASE: &str =
        include_str!("../tests/differential/v1/t25-foundation-arithmetic-self-check.json");

    fn case() -> DifferentialCase {
        DifferentialCase::from_json(FOUNDATION_CASE).expect("foundation case should decode")
    }

    fn observation(
        backend: DifferentialBackend,
        execution: ExecutionObservation,
    ) -> BackendObservation {
        BackendObservation {
            backend,
            output_events: OutputEventsObservation::Captured { events: Vec::new() },
            execution,
        }
    }

    #[test]
    fn committed_case_has_stable_schema_and_fingerprint() {
        let case = case();

        assert_eq!(case.id().as_str(), "t25/foundation/arithmetic-self-check");
        assert_eq!(case.goal(), DifferentialGoal::Script);
        assert_eq!(case.timeout_ms().get(), 5_000);
        assert_eq!(case_fingerprint(&case).as_str(), "fnv1a64:73f75d9ae75e0f47");
    }

    #[test]
    fn schema_v1_replay_uses_the_product_host_surface() {
        assert_eq!(
            compile_options_for_case(&case()).host_surface_policy,
            HostSurfacePolicy::Product
        );
    }

    #[test]
    fn corpus_decoder_rejects_version_drift_and_zero_timeout() {
        let wrong_version =
            FOUNDATION_CASE.replacen("\"schema_version\": 1", "\"schema_version\": 2", 1);
        assert!(DifferentialCase::from_json(&wrong_version)
            .expect_err("unknown corpus version should fail")
            .to_string()
            .contains("unsupported differential corpus schema_version 2"));

        let zero_timeout = FOUNDATION_CASE.replacen("\"timeout_ms\": 5000", "\"timeout_ms\": 0", 1);
        assert_eq!(
            DifferentialCase::from_json(&zero_timeout)
                .expect_err("zero timeout should fail")
                .to_string(),
            "timeout_ms must be non-zero"
        );
    }

    #[test]
    fn disposition_mismatch_has_a_pinned_machine_signature() {
        let case = case();
        let report = compare_observations(
            &case,
            observation(
                DifferentialBackend::WasmAot,
                ExecutionObservation::Normal {
                    backend_note: "wasm diagnostic".to_string(),
                },
            ),
            observation(
                DifferentialBackend::SpecExec,
                ExecutionObservation::Error {
                    phase: FailurePhase::SpecExecExecution,
                    message: "oracle diagnostic".to_string(),
                },
            ),
        );

        assert_eq!(report.verdict(), DifferentialVerdict::Mismatch);
        assert_eq!(
            report.semantic_equivalence(),
            SemanticEquivalence::NotEstablished
        );
        assert_eq!(
            report
                .mismatch_signature()
                .expect("mismatch should have signature")
                .as_str(),
            "lila-diff-v1:self-check-disposition:t25/foundation/arithmetic-self-check:\
             fnv1a64:73f75d9ae75e0f47:script:wasm-aot=normal:spec-exec=error"
        );
    }

    #[test]
    fn equal_disposition_does_not_claim_semantic_equivalence() {
        let case = case();
        let report = compare_observations(
            &case,
            observation(
                DifferentialBackend::WasmAot,
                ExecutionObservation::Normal {
                    backend_note: "wasm value rendering".to_string(),
                },
            ),
            observation(
                DifferentialBackend::SpecExec,
                ExecutionObservation::Normal {
                    backend_note: "unrelated oracle note".to_string(),
                },
            ),
        );

        assert_eq!(report.verdict(), DifferentialVerdict::BothCompleted);
        assert_eq!(
            report.semantic_equivalence(),
            SemanticEquivalence::NotEstablished
        );
        assert_eq!(report.observation_gaps(), &OBSERVATION_GAPS);
        assert!(report.mismatch_signature().is_none());

        let json: serde_json::Value = serde_json::from_str(
            &report
                .to_pretty_json()
                .expect("observation report should encode"),
        )
        .expect("observation report should be JSON");
        assert_eq!(json["schema_version"], 1);
        assert_eq!(json["semantic_equivalence"], "not_established");
        assert_eq!(json["compared_dimensions"][0], "self_check_disposition");
        assert_eq!(json["observation_gaps"].as_array().unwrap().len(), 10);
        assert_eq!(json["wasm_aot"]["execution"]["disposition"], "normal");
        assert_eq!(json["spec_exec"]["execution"]["disposition"], "normal");
        assert_eq!(
            json["wasm_aot"]["output_events"]["availability"],
            "captured"
        );
        assert_eq!(
            json["wasm_aot"]["output_events"]["events"],
            serde_json::json!([])
        );
        assert_eq!(
            json["spec_exec"]["output_events"]["availability"],
            "captured"
        );
        assert_eq!(
            json["spec_exec"]["output_events"]["events"],
            serde_json::json!([])
        );
    }

    #[test]
    fn report_v1_keeps_the_original_unavailable_and_gap_vocabulary() {
        let unavailable = serde_json::to_value(OutputEventsObservation::Unavailable {
            reason: OutputUnavailableReason::SpecExecBypassesEngineHostHooks,
        })
        .expect("legacy output availability should remain serializable");
        assert_eq!(
            unavailable,
            serde_json::json!({
                "availability": "unavailable",
                "reason": "spec_exec_bypasses_engine_host_hooks"
            })
        );
        assert_eq!(
            serde_json::to_value(ObservationGap::SpecExecOutputEventsUnavailable)
                .expect("legacy observation gap should remain serializable"),
            "spec_exec_output_events_unavailable"
        );
    }

    #[test]
    fn either_backend_output_makes_a_no_output_case_red() {
        let case = case();
        for output_backend in [DifferentialBackend::WasmAot, DifferentialBackend::SpecExec] {
            let mut wasm = observation(
                DifferentialBackend::WasmAot,
                ExecutionObservation::Normal {
                    backend_note: "wasm diagnostic".to_string(),
                },
            );
            let mut spec_exec = observation(
                DifferentialBackend::SpecExec,
                ExecutionObservation::Normal {
                    backend_note: "oracle diagnostic".to_string(),
                },
            );
            let output_events = OutputEventsObservation::Captured {
                events: vec!["unexpected output".to_string()],
            };
            match output_backend {
                DifferentialBackend::WasmAot => wasm.output_events = output_events,
                DifferentialBackend::SpecExec => spec_exec.output_events = output_events,
            }
            let report = compare_observations(&case, wasm, spec_exec);

            assert_eq!(
                report.verdict(),
                DifferentialVerdict::ObservationContractViolated
            );
            assert!(!report.is_green());
        }
    }

    #[cfg(not(feature = "spec-exec-oracle"))]
    #[test]
    fn replay_requires_the_compile_time_oracle_gate() {
        let error = replay_case(&case(), SpecExecOracle::explicitly_enabled())
            .expect_err("default build must not link spec-exec");
        assert!(matches!(error, DifferentialError::OracleNotLinked));
    }

    #[cfg(feature = "spec-exec-oracle")]
    #[test]
    fn committed_foundation_case_replays_through_both_backends() {
        let report = replay_case(&case(), SpecExecOracle::explicitly_enabled())
            .expect("both explicitly enabled backends should run");

        assert_eq!(report.verdict(), DifferentialVerdict::BothCompleted);
        assert!(report.is_green());
    }
}
