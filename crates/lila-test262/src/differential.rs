//! Deterministic differential-corpus replay.
//!
//! These initial observation protocols are intentionally narrow. The engine
//! exposes structured normal and thrown values plus per-run host output for
//! both backends. Schema v1 still consumes only projected disposition and
//! output emptiness. Schema v2 additively compares primitive completion kind
//! and value while rejecting Symbol, Object and output as outside its bounded
//! contract. Neither protocol promotes its declared match to whole-program
//! semantic equivalence.

use std::fmt;
use std::fs;
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};
#[cfg(feature = "spec-exec-oracle")]
use std::sync::{Arc, Mutex};

use lila_engine::{CompileOptions, ObservedCompletion, ObservedJsValue};
#[cfg(feature = "spec-exec-oracle")]
use lila_engine::{
    Engine, EngineError, ExecutionBackend, HostOutputEvent, RealmBuilder, RunOptions,
};
#[cfg(feature = "spec-exec-oracle")]
use lila_ir::IrDiagnosticPhase;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};

mod generated_arithmetic;

pub use generated_arithmetic::{
    run_generated_arithmetic_campaign, ArithmeticCheckCount, ArithmeticExpressionDepth,
    ArithmeticGenerationPlan, ArithmeticGenerationSeed, ArithmeticReductionLimit,
    ArithmeticReductionStop, ArithmeticReductionSummary, GeneratedArithmeticCampaignOutcome,
    MAX_ARITHMETIC_CHECKS, MAX_ARITHMETIC_REDUCTION_REPLAYS,
};

pub const DIFFERENTIAL_CORPUS_SCHEMA_VERSION: u32 = 1;
pub const DIFFERENTIAL_REPORT_SCHEMA_VERSION: u32 = 1;
pub const DIFFERENTIAL_CORPUS_SCHEMA_VERSION_V2: u32 = 2;
pub const DIFFERENTIAL_REPORT_SCHEMA_VERSION_V2: u32 = 2;

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

/// The serialized observation-contract vocabulary used by corpus and report
/// schemas. The version is not stored independently in memory; see
/// [`DifferentialProtocol`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationContract {
    SelfCheckingNoOutput,
    PrimitiveCompletionNoOutput,
}

impl ObservationContract {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SelfCheckingNoOutput => "self_checking_no_output",
            Self::PrimitiveCompletionNoOutput => "primitive_completion_no_output",
        }
    }
}

/// One admitted corpus/report protocol.
///
/// The schema version and contract spelling are projections of this closed
/// enum, so an in-memory case cannot pair schema v1 with the v2 contract (or
/// vice versa). Wire decoding validates the pair once at the boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DifferentialProtocol {
    V1SelfCheckingNoOutput,
    V2PrimitiveCompletionNoOutput,
}

impl DifferentialProtocol {
    pub const fn schema_version(self) -> u32 {
        match self {
            Self::V1SelfCheckingNoOutput => DIFFERENTIAL_CORPUS_SCHEMA_VERSION,
            Self::V2PrimitiveCompletionNoOutput => DIFFERENTIAL_CORPUS_SCHEMA_VERSION_V2,
        }
    }

    const fn report_schema_version(self) -> u32 {
        match self {
            Self::V1SelfCheckingNoOutput => DIFFERENTIAL_REPORT_SCHEMA_VERSION,
            Self::V2PrimitiveCompletionNoOutput => DIFFERENTIAL_REPORT_SCHEMA_VERSION_V2,
        }
    }

    pub const fn observation_contract(self) -> ObservationContract {
        match self {
            Self::V1SelfCheckingNoOutput => ObservationContract::SelfCheckingNoOutput,
            Self::V2PrimitiveCompletionNoOutput => ObservationContract::PrimitiveCompletionNoOutput,
        }
    }

    fn from_wire(
        schema_version: u32,
        observation_contract: ObservationContract,
    ) -> Result<Self, DifferentialError> {
        match (schema_version, observation_contract) {
            (DIFFERENTIAL_CORPUS_SCHEMA_VERSION, ObservationContract::SelfCheckingNoOutput) => {
                Ok(Self::V1SelfCheckingNoOutput)
            }
            (
                DIFFERENTIAL_CORPUS_SCHEMA_VERSION_V2,
                ObservationContract::PrimitiveCompletionNoOutput,
            ) => Ok(Self::V2PrimitiveCompletionNoOutput),
            _ => Err(DifferentialError::InvalidCorpus(format!(
                "unsupported differential protocol pair: schema_version {schema_version} with observation_contract {}",
                observation_contract.as_str()
            ))),
        }
    }
}

impl From<ObservationContract> for DifferentialProtocol {
    fn from(contract: ObservationContract) -> Self {
        match contract {
            ObservationContract::SelfCheckingNoOutput => Self::V1SelfCheckingNoOutput,
            ObservationContract::PrimitiveCompletionNoOutput => Self::V2PrimitiveCompletionNoOutput,
        }
    }
}

/// One deterministic input to differential replay.
///
/// Fields are private so callers cannot manufacture an unsupported schema
/// version, zero timeout, unstable filename, or malformed case key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DifferentialCase {
    protocol: DifferentialProtocol,
    id: DifferentialCaseId,
    goal: DifferentialGoal,
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
        protocol: impl Into<DifferentialProtocol>,
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
            protocol: protocol.into(),
            id: DifferentialCaseId::new(id)?,
            goal,
            filename,
            timeout_ms,
            source,
        })
    }

    pub fn from_json(json: &str) -> Result<Self, DifferentialError> {
        let wire: DifferentialCaseWire = serde_json::from_str(json)
            .map_err(|error| DifferentialError::DecodeCorpus(error.to_string()))?;
        let protocol =
            DifferentialProtocol::from_wire(wire.schema_version, wire.observation_contract)?;
        Self::new(
            wire.id,
            wire.goal,
            protocol,
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

    pub const fn protocol(&self) -> DifferentialProtocol {
        self.protocol
    }

    pub const fn observation_contract(&self) -> ObservationContract {
        self.protocol.observation_contract()
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

impl Serialize for DifferentialCase {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut case = serializer.serialize_struct("DifferentialCase", 7)?;
        case.serialize_field("schema_version", &self.protocol.schema_version())?;
        case.serialize_field("id", &self.id)?;
        case.serialize_field("goal", &self.goal)?;
        case.serialize_field(
            "observation_contract",
            &self.protocol.observation_contract(),
        )?;
        case.serialize_field("filename", &self.filename)?;
        case.serialize_field("timeout_ms", &self.timeout_ms)?;
        case.serialize_field("source", &self.source)?;
        case.end()
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
pub enum CompletionKindObservation {
    Normal,
    Throw,
}

impl CompletionKindObservation {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Throw => "throw",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PrimitiveValueObservation {
    Undefined,
    Null,
    Boolean {
        value: bool,
    },
    Number {
        bits: String,
    },
    String {
        utf16_units: Vec<u16>,
    },
    #[serde(rename = "bigint")]
    BigInt {
        decimal: String,
    },
}

impl PrimitiveValueObservation {
    fn from_observed(value: &ObservedJsValue) -> Result<Self, UnsupportedObservedValueType> {
        match value {
            ObservedJsValue::Undefined => Ok(Self::Undefined),
            ObservedJsValue::Null => Ok(Self::Null),
            ObservedJsValue::Boolean(value) => Ok(Self::Boolean { value: *value }),
            ObservedJsValue::Number(value) => Ok(Self::Number {
                bits: format!("{:016x}", value.bits()),
            }),
            ObservedJsValue::String(units) => Ok(Self::String {
                utf16_units: units.to_vec(),
            }),
            ObservedJsValue::BigInt(decimal) => Ok(Self::BigInt {
                decimal: decimal.to_string(),
            }),
            ObservedJsValue::Symbol => Err(UnsupportedObservedValueType::Symbol),
            ObservedJsValue::Object => Err(UnsupportedObservedValueType::Object),
        }
    }

    const fn type_name(&self) -> &'static str {
        match self {
            Self::Undefined => "undefined",
            Self::Null => "null",
            Self::Boolean { .. } => "boolean",
            Self::Number { .. } => "number",
            Self::String { .. } => "string",
            Self::BigInt { .. } => "bigint",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PrimitiveCompletionObservation {
    Normal { value: PrimitiveValueObservation },
    Throw { value: PrimitiveValueObservation },
}

impl PrimitiveCompletionObservation {
    const fn kind(&self) -> CompletionKindObservation {
        match self {
            Self::Normal { .. } => CompletionKindObservation::Normal,
            Self::Throw { .. } => CompletionKindObservation::Throw,
        }
    }

    fn value(&self) -> &PrimitiveValueObservation {
        match self {
            Self::Normal { value } | Self::Throw { value } => value,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UnsupportedObservedValueType {
    Symbol,
    Object,
}

impl UnsupportedObservedValueType {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Symbol => "symbol",
            Self::Object => "object",
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

impl FailurePhase {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Parse => "parse",
            Self::EarlyError => "early_error",
            Self::ModuleResolution => "module_resolution",
            Self::Lowering => "lowering",
            Self::WasmRuntimeCapability => "wasm_runtime_capability",
            Self::WasmRuntimeOrBackend => "wasm_runtime_or_backend",
            Self::SpecExecExecution => "spec_exec_execution",
            Self::RunnerInvariant => "runner_invariant",
        }
    }
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
    PrimitiveCompletion {
        completion: PrimitiveCompletionObservation,
        backend_note: String,
    },
    UnsupportedCompletion {
        completion_kind: CompletionKindObservation,
        value_type: UnsupportedObservedValueType,
        backend_note: String,
    },
    #[serde(rename = "engine_error")]
    EngineFailure {
        phase: FailurePhase,
        message: String,
    },
}

impl ExecutionObservation {
    pub const fn disposition(&self) -> ExecutionDisposition {
        match self {
            Self::Normal { .. } => ExecutionDisposition::Normal,
            Self::Error { .. } | Self::EngineFailure { .. } => ExecutionDisposition::Error,
            Self::PrimitiveCompletion { completion, .. } => match completion.kind() {
                CompletionKindObservation::Normal => ExecutionDisposition::Normal,
                CompletionKindObservation::Throw => ExecutionDisposition::Error,
            },
            Self::UnsupportedCompletion {
                completion_kind, ..
            } => match completion_kind {
                CompletionKindObservation::Normal => ExecutionDisposition::Normal,
                CompletionKindObservation::Throw => ExecutionDisposition::Error,
            },
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
    CompletionKind,
    PrimitiveValue,
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
    UncapturedSymbolIdentity,
    UncapturedObjectIdentity,
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

pub const COMPARED_DIMENSIONS_V2: [ComparedDimension; 2] = [
    ComparedDimension::CompletionKind,
    ComparedDimension::PrimitiveValue,
];
pub const OBSERVATION_GAPS_V2: [ObservationGap; 9] = [
    ObservationGap::UncapturedSymbolIdentity,
    ObservationGap::UncapturedObjectIdentity,
    ObservationGap::UncapturedErrorRealm,
    ObservationGap::UncapturedPropertyDescriptors,
    ObservationGap::UncapturedOwnKeyOrder,
    ObservationGap::UncapturedPrototypeIdentity,
    ObservationGap::UncapturedSideEffectLog,
    ObservationGap::UnisolatedPanicAndHostCrash,
    ObservationGap::SpecExecTimeoutNotEnforced,
];

impl DifferentialProtocol {
    const fn compared_dimensions(self) -> &'static [ComparedDimension] {
        match self {
            Self::V1SelfCheckingNoOutput => &COMPARED_DIMENSIONS,
            Self::V2PrimitiveCompletionNoOutput => &COMPARED_DIMENSIONS_V2,
        }
    }

    const fn observation_gaps(self) -> &'static [ObservationGap] {
        match self {
            Self::V1SelfCheckingNoOutput => &OBSERVATION_GAPS,
            Self::V2PrimitiveCompletionNoOutput => &OBSERVATION_GAPS_V2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DifferentialVerdict {
    BothCompleted,
    PrimitiveCompletionsMatch,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DifferentialReport {
    protocol: DifferentialProtocol,
    case_id: DifferentialCaseId,
    case_fingerprint: CaseFingerprint,
    verdict: DifferentialVerdict,
    /// Always `not_established`. Matching one protocol's bounded dimensions is
    /// deliberately not promoted to whole-program semantic equivalence.
    semantic_equivalence: SemanticEquivalence,
    wasm_aot: BackendObservation,
    spec_exec: BackendObservation,
    mismatch_signature: Option<MismatchSignature>,
}

impl DifferentialReport {
    pub const fn protocol(&self) -> DifferentialProtocol {
        self.protocol
    }

    pub const fn verdict(&self) -> DifferentialVerdict {
        self.verdict
    }

    pub const fn is_green(&self) -> bool {
        matches!(
            self.verdict,
            DifferentialVerdict::BothCompleted | DifferentialVerdict::PrimitiveCompletionsMatch
        )
    }

    pub const fn semantic_equivalence(&self) -> SemanticEquivalence {
        self.semantic_equivalence
    }

    pub const fn compared_dimensions(&self) -> &[ComparedDimension] {
        self.protocol.compared_dimensions()
    }

    pub const fn observation_gaps(&self) -> &[ObservationGap] {
        self.protocol.observation_gaps()
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

impl Serialize for DifferentialReport {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Keep this explicit projection in the original v1 field order. The
        // protocol chooses the versioned vocabulary; callers cannot combine a
        // v1 version with v2 dimensions or gaps.
        let mut report = serializer.serialize_struct("DifferentialReport", 11)?;
        report.serialize_field("schema_version", &self.protocol.report_schema_version())?;
        report.serialize_field("case_id", &self.case_id)?;
        report.serialize_field("case_fingerprint", &self.case_fingerprint)?;
        report.serialize_field(
            "observation_contract",
            &self.protocol.observation_contract(),
        )?;
        report.serialize_field("verdict", &self.verdict)?;
        report.serialize_field("semantic_equivalence", &self.semantic_equivalence)?;
        report.serialize_field("compared_dimensions", self.protocol.compared_dimensions())?;
        report.serialize_field("observation_gaps", self.protocol.observation_gaps())?;
        report.serialize_field("wasm_aot", &self.wasm_aot)?;
        report.serialize_field("spec_exec", &self.spec_exec)?;
        report.serialize_field("mismatch_signature", &self.mismatch_signature)?;
        report.end()
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
    Ok(compare_executions(case, wasm_aot, spec_exec))
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct BackendExecution {
    backend: DifferentialBackend,
    output_events: OutputEventsObservation,
    result: BackendExecutionResult,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum BackendExecutionResult {
    Completion {
        completion: ObservedCompletion,
        backend_note: String,
    },
    EngineFailure {
        phase: FailurePhase,
        message: String,
    },
}

impl BackendExecutionResult {
    const fn disposition(&self) -> ExecutionDisposition {
        match self {
            Self::Completion {
                completion: ObservedCompletion::Normal(_),
                ..
            } => ExecutionDisposition::Normal,
            Self::Completion {
                completion: ObservedCompletion::Throw(_),
                ..
            }
            | Self::EngineFailure { .. } => ExecutionDisposition::Error,
        }
    }
}

#[cfg(feature = "spec-exec-oracle")]
fn execute_case(case: &DifferentialCase, backend: DifferentialBackend) -> BackendExecution {
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
    let (result, output_events) = match outcome {
        Ok(outcome) if outcome.backend_used == backend.execution_backend() => {
            let result = BackendExecutionResult::Completion {
                completion: outcome.completion,
                backend_note: outcome.note,
            };
            (result, captured_output_events(outcome.output_events))
        }
        Ok(outcome) => {
            let output_events = captured_output_events(outcome.output_events);
            (
                BackendExecutionResult::EngineFailure {
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
    BackendExecution {
        backend,
        output_events,
        result,
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

const fn execution_failure_phase(backend: DifferentialBackend) -> FailurePhase {
    match backend {
        DifferentialBackend::WasmAot => FailurePhase::WasmRuntimeOrBackend,
        DifferentialBackend::SpecExec => FailurePhase::SpecExecExecution,
    }
}

/// Differential corpus programs are product probes, not Test262 harness
/// programs. Keep the authority choice at this boundary so replay cannot gain
/// conformance-only globals as an incidental engine-test convenience.
fn compile_options_for_case(case: &DifferentialCase) -> CompileOptions {
    CompileOptions {
        filename: Some(case.filename.clone()),
        ..CompileOptions::default()
    }
}

#[cfg(feature = "spec-exec-oracle")]
fn observe_engine_error(
    backend: DifferentialBackend,
    error: &EngineError,
) -> BackendExecutionResult {
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
    BackendExecutionResult::EngineFailure {
        phase,
        message: error.message().to_string(),
    }
}

fn compare_executions(
    case: &DifferentialCase,
    wasm_execution: BackendExecution,
    spec_execution: BackendExecution,
) -> DifferentialReport {
    let obeys_no_output_contract = |execution: &BackendExecution| match &execution.output_events {
        OutputEventsObservation::Captured { events } => events.is_empty(),
        OutputEventsObservation::Unavailable { .. } => false,
    };
    let no_output =
        obeys_no_output_contract(&wasm_execution) && obeys_no_output_contract(&spec_execution);
    let protocol = case.protocol();
    let wasm_disposition = wasm_execution.result.disposition();
    let spec_disposition = spec_execution.result.disposition();
    let wasm_aot = project_backend_execution(protocol, wasm_execution);
    let spec_exec = project_backend_execution(protocol, spec_execution);

    let verdict = if !no_output {
        DifferentialVerdict::ObservationContractViolated
    } else {
        match protocol {
            DifferentialProtocol::V1SelfCheckingNoOutput => {
                compare_v1_dispositions(wasm_disposition, spec_disposition)
            }
            DifferentialProtocol::V2PrimitiveCompletionNoOutput => {
                compare_v2_observations(&wasm_aot.execution, &spec_exec.execution)
            }
        }
    };
    let case_fingerprint = case_fingerprint(case);
    let mismatch_signature =
        matches!(verdict, DifferentialVerdict::Mismatch).then(|| match protocol {
            DifferentialProtocol::V1SelfCheckingNoOutput => MismatchSignature(format!(
                "lila-diff-v1:self-check-disposition:{}:{}:{}:wasm-aot={}:spec-exec={}",
                case.id.as_str(),
                case_fingerprint.as_str(),
                case.goal.as_str(),
                wasm_disposition.as_str(),
                spec_disposition.as_str(),
            )),
            DifferentialProtocol::V2PrimitiveCompletionNoOutput => MismatchSignature(format!(
                "lila-diff-v2:primitive-completion:{}:{}:{}:wasm-aot={}:spec-exec={}",
                case.id.as_str(),
                case_fingerprint.as_str(),
                case.goal.as_str(),
                v2_execution_signature(&wasm_aot.execution),
                v2_execution_signature(&spec_exec.execution),
            )),
        });
    DifferentialReport {
        protocol,
        case_id: case.id.clone(),
        case_fingerprint,
        verdict,
        semantic_equivalence: SemanticEquivalence::NotEstablished,
        wasm_aot,
        spec_exec,
        mismatch_signature,
    }
}

fn project_backend_execution(
    protocol: DifferentialProtocol,
    execution: BackendExecution,
) -> BackendObservation {
    let BackendExecution {
        backend,
        output_events,
        result,
    } = execution;
    let execution = match (protocol, result) {
        (
            DifferentialProtocol::V1SelfCheckingNoOutput,
            BackendExecutionResult::Completion {
                completion: ObservedCompletion::Normal(_),
                backend_note,
            },
        ) => ExecutionObservation::Normal { backend_note },
        (
            DifferentialProtocol::V1SelfCheckingNoOutput,
            BackendExecutionResult::Completion {
                completion: ObservedCompletion::Throw(_),
                backend_note,
            },
        ) => ExecutionObservation::Error {
            phase: execution_failure_phase(backend),
            message: backend_note,
        },
        (
            DifferentialProtocol::V1SelfCheckingNoOutput,
            BackendExecutionResult::EngineFailure { phase, message },
        ) => ExecutionObservation::Error { phase, message },
        (
            DifferentialProtocol::V2PrimitiveCompletionNoOutput,
            BackendExecutionResult::Completion {
                completion,
                backend_note,
            },
        ) => project_primitive_completion(completion, backend_note),
        (
            DifferentialProtocol::V2PrimitiveCompletionNoOutput,
            BackendExecutionResult::EngineFailure { phase, message },
        ) => ExecutionObservation::EngineFailure { phase, message },
    };
    BackendObservation {
        backend,
        output_events,
        execution,
    }
}

fn project_primitive_completion(
    completion: ObservedCompletion,
    backend_note: String,
) -> ExecutionObservation {
    let (kind, value) = match completion {
        ObservedCompletion::Normal(value) => (CompletionKindObservation::Normal, value),
        ObservedCompletion::Throw(value) => (CompletionKindObservation::Throw, value),
    };
    match PrimitiveValueObservation::from_observed(&value) {
        Ok(value) => {
            let completion = match kind {
                CompletionKindObservation::Normal => {
                    PrimitiveCompletionObservation::Normal { value }
                }
                CompletionKindObservation::Throw => PrimitiveCompletionObservation::Throw { value },
            };
            ExecutionObservation::PrimitiveCompletion {
                completion,
                backend_note,
            }
        }
        Err(value_type) => ExecutionObservation::UnsupportedCompletion {
            completion_kind: kind,
            value_type,
            backend_note,
        },
    }
}

const fn compare_v1_dispositions(
    wasm: ExecutionDisposition,
    spec_exec: ExecutionDisposition,
) -> DifferentialVerdict {
    match (wasm, spec_exec) {
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
}

fn compare_v2_observations(
    wasm: &ExecutionObservation,
    spec_exec: &ExecutionObservation,
) -> DifferentialVerdict {
    if matches!(wasm, ExecutionObservation::UnsupportedCompletion { .. })
        || matches!(
            spec_exec,
            ExecutionObservation::UnsupportedCompletion { .. }
        )
    {
        return DifferentialVerdict::ObservationContractViolated;
    }

    match (wasm, spec_exec) {
        (
            ExecutionObservation::PrimitiveCompletion {
                completion: wasm, ..
            },
            ExecutionObservation::PrimitiveCompletion {
                completion: spec_exec,
                ..
            },
        ) if wasm == spec_exec => DifferentialVerdict::PrimitiveCompletionsMatch,
        (
            ExecutionObservation::EngineFailure { .. },
            ExecutionObservation::EngineFailure { .. },
        ) => DifferentialVerdict::BothFailed,
        (
            ExecutionObservation::PrimitiveCompletion { .. },
            ExecutionObservation::PrimitiveCompletion { .. }
            | ExecutionObservation::EngineFailure { .. },
        )
        | (
            ExecutionObservation::EngineFailure { .. },
            ExecutionObservation::PrimitiveCompletion { .. },
        ) => DifferentialVerdict::Mismatch,
        (
            ExecutionObservation::Normal { .. }
            | ExecutionObservation::Error { .. }
            | ExecutionObservation::UnsupportedCompletion { .. },
            _,
        )
        | (
            _,
            ExecutionObservation::Normal { .. }
            | ExecutionObservation::Error { .. }
            | ExecutionObservation::UnsupportedCompletion { .. },
        ) => DifferentialVerdict::ObservationContractViolated,
    }
}

fn v2_execution_signature(execution: &ExecutionObservation) -> String {
    match execution {
        ExecutionObservation::PrimitiveCompletion { completion, .. } => format!(
            "{}-{}-{}",
            completion.kind().as_str(),
            completion.value().type_name(),
            primitive_value_signature(completion.value()),
        ),
        ExecutionObservation::UnsupportedCompletion {
            completion_kind,
            value_type,
            ..
        } => format!(
            "{}-{}-unsupported",
            completion_kind.as_str(),
            value_type.as_str()
        ),
        ExecutionObservation::EngineFailure { phase, .. } => {
            format!("engine-error-{}", phase.as_str())
        }
        ExecutionObservation::Normal { .. } => "invalid-v1-normal".to_string(),
        ExecutionObservation::Error { phase, .. } => {
            format!("invalid-v1-error-{}", phase.as_str())
        }
    }
}

fn primitive_value_signature(value: &PrimitiveValueObservation) -> String {
    match value {
        PrimitiveValueObservation::Undefined => "undefined".to_string(),
        PrimitiveValueObservation::Null => "null".to_string(),
        PrimitiveValueObservation::Boolean { value } => value.to_string(),
        PrimitiveValueObservation::Number { bits } => bits.clone(),
        PrimitiveValueObservation::String { utf16_units } => {
            let mut hash = fnv_update(FNV_OFFSET_BASIS, b"lila-diff-v2-string");
            hash = fnv_update(hash, &(utf16_units.len() as u64).to_le_bytes());
            for unit in utf16_units {
                hash = fnv_update(hash, &unit.to_le_bytes());
            }
            format!("fnv1a64-{hash:016x}")
        }
        PrimitiveValueObservation::BigInt { decimal } => {
            let hash = fnv_field(
                fnv_update(FNV_OFFSET_BASIS, b"lila-diff-v2-bigint"),
                decimal.as_bytes(),
            );
            format!("fnv1a64-{hash:016x}")
        }
    }
}

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;

fn fnv_update(mut hash: u64, bytes: &[u8]) -> u64 {
    const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn fnv_field(hash: u64, value: &[u8]) -> u64 {
    let hash = fnv_update(hash, &(value.len() as u64).to_le_bytes());
    fnv_update(hash, value)
}

fn case_fingerprint(case: &DifferentialCase) -> CaseFingerprint {
    let domain: &[u8] = match case.protocol {
        DifferentialProtocol::V1SelfCheckingNoOutput => b"lila-differential-case-v1",
        DifferentialProtocol::V2PrimitiveCompletionNoOutput => b"lila-differential-case-v2",
    };
    let mut hash = fnv_update(FNV_OFFSET_BASIS, domain);
    hash = fnv_field(hash, case.goal.as_str().as_bytes());
    hash = fnv_field(hash, case.observation_contract().as_str().as_bytes());
    hash = fnv_field(hash, case.filename.as_bytes());
    hash = fnv_field(hash, &case.timeout_ms.get().to_le_bytes());
    hash = fnv_field(hash, case.source.as_bytes());
    CaseFingerprint(format!("fnv1a64:{hash:016x}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use lila_engine::ObservedNumber;
    use lila_ir::HostSurfacePolicy;

    const FOUNDATION_CASE_V1: &str =
        include_str!("../tests/differential/v1/t25-foundation-arithmetic-self-check.json");
    const FOUNDATION_CASE_V2: &str =
        include_str!("../tests/differential/v2/t25-foundation-primitive-number.json");

    fn case_v1() -> DifferentialCase {
        DifferentialCase::from_json(FOUNDATION_CASE_V1).expect("v1 foundation case should decode")
    }

    fn case_v2() -> DifferentialCase {
        DifferentialCase::from_json(FOUNDATION_CASE_V2).expect("v2 foundation case should decode")
    }

    fn execution(backend: DifferentialBackend, result: BackendExecutionResult) -> BackendExecution {
        BackendExecution {
            backend,
            output_events: OutputEventsObservation::Captured { events: Vec::new() },
            result,
        }
    }

    fn completed(
        backend: DifferentialBackend,
        completion: ObservedCompletion,
        backend_note: &str,
    ) -> BackendExecution {
        execution(
            backend,
            BackendExecutionResult::Completion {
                completion,
                backend_note: backend_note.to_string(),
            },
        )
    }

    fn failed(
        backend: DifferentialBackend,
        phase: FailurePhase,
        message: &str,
    ) -> BackendExecution {
        execution(
            backend,
            BackendExecutionResult::EngineFailure {
                phase,
                message: message.to_string(),
            },
        )
    }

    #[test]
    fn committed_v1_case_and_fingerprint_remain_byte_for_byte_stable() {
        let case = case_v1();

        assert_eq!(case.id().as_str(), "t25/foundation/arithmetic-self-check");
        assert_eq!(case.goal(), DifferentialGoal::Script);
        assert_eq!(case.timeout_ms().get(), 5_000);
        assert_eq!(case_fingerprint(&case).as_str(), "fnv1a64:73f75d9ae75e0f47");
        assert_eq!(
            case.to_pretty_json().expect("v1 case should encode"),
            FOUNDATION_CASE_V1
        );
    }

    #[test]
    fn committed_v2_case_has_a_closed_protocol_and_stable_fingerprint() {
        let case = case_v2();

        assert_eq!(
            case.protocol(),
            DifferentialProtocol::V2PrimitiveCompletionNoOutput
        );
        assert_eq!(case.id().as_str(), "t25/foundation/primitive-number");
        assert_eq!(case_fingerprint(&case).as_str(), "fnv1a64:71825b2dcabbbc2c");
        assert_eq!(
            case.to_pretty_json().expect("v2 case should encode"),
            FOUNDATION_CASE_V2
        );
    }

    #[test]
    fn both_protocols_replay_with_the_product_host_surface() {
        assert_eq!(
            compile_options_for_case(&case_v1()).host_surface_policy,
            HostSurfacePolicy::Product
        );
        assert_eq!(
            compile_options_for_case(&case_v2()).host_surface_policy,
            HostSurfacePolicy::Product
        );
    }

    #[test]
    fn corpus_decoder_rejects_protocol_cross_pairs_and_zero_timeout() {
        let v2_version_with_v1_contract =
            FOUNDATION_CASE_V1.replacen("\"schema_version\": 1", "\"schema_version\": 2", 1);
        assert_eq!(
            DifferentialCase::from_json(&v2_version_with_v1_contract)
                .expect_err("v2 version with v1 contract should fail")
                .to_string(),
            "unsupported differential protocol pair: schema_version 2 with observation_contract self_checking_no_output"
        );

        let v1_version_with_v2_contract =
            FOUNDATION_CASE_V2.replacen("\"schema_version\": 2", "\"schema_version\": 1", 1);
        assert_eq!(
            DifferentialCase::from_json(&v1_version_with_v2_contract)
                .expect_err("v1 version with v2 contract should fail")
                .to_string(),
            "unsupported differential protocol pair: schema_version 1 with observation_contract primitive_completion_no_output"
        );

        let zero_timeout =
            FOUNDATION_CASE_V1.replacen("\"timeout_ms\": 5000", "\"timeout_ms\": 0", 1);
        assert_eq!(
            DifferentialCase::from_json(&zero_timeout)
                .expect_err("zero timeout should fail")
                .to_string(),
            "timeout_ms must be non-zero"
        );
    }

    #[test]
    fn v1_disposition_mismatch_keeps_its_pinned_machine_signature() {
        let case = case_v1();
        let report = compare_executions(
            &case,
            completed(
                DifferentialBackend::WasmAot,
                ObservedCompletion::Normal(ObservedJsValue::Undefined),
                "wasm diagnostic",
            ),
            failed(
                DifferentialBackend::SpecExec,
                FailurePhase::SpecExecExecution,
                "oracle diagnostic",
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
    fn v1_report_serialization_remains_byte_for_byte_stable() {
        let case = case_v1();
        let report = compare_executions(
            &case,
            completed(
                DifferentialBackend::WasmAot,
                ObservedCompletion::Normal(ObservedJsValue::Undefined),
                "wasm value rendering",
            ),
            completed(
                DifferentialBackend::SpecExec,
                ObservedCompletion::Normal(ObservedJsValue::Null),
                "unrelated oracle note",
            ),
        );

        assert_eq!(report.verdict(), DifferentialVerdict::BothCompleted);
        assert_eq!(
            report.semantic_equivalence(),
            SemanticEquivalence::NotEstablished
        );
        assert_eq!(report.observation_gaps(), &OBSERVATION_GAPS);
        assert!(report.mismatch_signature().is_none());

        assert_eq!(
            report.to_pretty_json().expect("v1 report should encode"),
            r#"{
  "schema_version": 1,
  "case_id": "t25/foundation/arithmetic-self-check",
  "case_fingerprint": "fnv1a64:73f75d9ae75e0f47",
  "observation_contract": "self_checking_no_output",
  "verdict": "both_completed",
  "semantic_equivalence": "not_established",
  "compared_dimensions": [
    "self_check_disposition"
  ],
  "observation_gaps": [
    "unstructured_normal_value",
    "unstructured_completion_kind",
    "unstructured_thrown_value",
    "uncaptured_error_realm",
    "uncaptured_property_descriptors",
    "uncaptured_own_key_order",
    "uncaptured_prototype_identity",
    "uncaptured_side_effect_log",
    "unisolated_panic_and_host_crash",
    "spec_exec_timeout_not_enforced"
  ],
  "wasm_aot": {
    "backend": "wasm-aot",
    "output_events": {
      "availability": "captured",
      "events": []
    },
    "execution": {
      "disposition": "normal",
      "backend_note": "wasm value rendering"
    }
  },
  "spec_exec": {
    "backend": "spec-exec",
    "output_events": {
      "availability": "captured",
      "events": []
    },
    "execution": {
      "disposition": "normal",
      "backend_note": "unrelated oracle note"
    }
  },
  "mismatch_signature": null
}"#
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
        for case in [case_v1(), case_v2()] {
            for output_backend in [DifferentialBackend::WasmAot, DifferentialBackend::SpecExec] {
                let mut wasm = completed(
                    DifferentialBackend::WasmAot,
                    ObservedCompletion::Normal(ObservedJsValue::Undefined),
                    "wasm diagnostic",
                );
                let mut spec_exec = completed(
                    DifferentialBackend::SpecExec,
                    ObservedCompletion::Normal(ObservedJsValue::Undefined),
                    "oracle diagnostic",
                );
                let output_events = OutputEventsObservation::Captured {
                    events: vec!["unexpected output".to_string()],
                };
                match output_backend {
                    DifferentialBackend::WasmAot => wasm.output_events = output_events,
                    DifferentialBackend::SpecExec => spec_exec.output_events = output_events,
                }
                let report = compare_executions(&case, wasm, spec_exec);

                assert_eq!(
                    report.verdict(),
                    DifferentialVerdict::ObservationContractViolated
                );
                assert!(!report.is_green());
            }
        }
    }

    #[test]
    fn v2_matches_each_supported_primitive_without_claiming_semantic_equivalence() {
        let values = [
            ObservedJsValue::Undefined,
            ObservedJsValue::Null,
            ObservedJsValue::Boolean(true),
            ObservedJsValue::Number(ObservedNumber::from_bits(0x7ff0_0000_0000_0001)),
            ObservedJsValue::String(vec![0xd800].into_boxed_slice()),
            ObservedJsValue::BigInt("-9007199254740993".to_string().into_boxed_str()),
        ];
        for value in values {
            let report = compare_executions(
                &case_v2(),
                completed(
                    DifferentialBackend::WasmAot,
                    ObservedCompletion::Normal(value.clone()),
                    "wasm",
                ),
                completed(
                    DifferentialBackend::SpecExec,
                    ObservedCompletion::Normal(value),
                    "spec",
                ),
            );

            assert_eq!(
                report.verdict(),
                DifferentialVerdict::PrimitiveCompletionsMatch
            );
            assert!(report.is_green());
            assert_eq!(
                report.semantic_equivalence(),
                SemanticEquivalence::NotEstablished
            );
            assert_eq!(report.compared_dimensions(), &COMPARED_DIMENSIONS_V2);
            assert_eq!(report.observation_gaps(), &OBSERVATION_GAPS_V2);
        }

        let canonical_nan = compare_executions(
            &case_v2(),
            completed(
                DifferentialBackend::WasmAot,
                ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_bits(
                    0x7ff0_0000_0000_0001,
                ))),
                "wasm",
            ),
            completed(
                DifferentialBackend::SpecExec,
                ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_bits(
                    0xfff8_1234_5678_9abc,
                ))),
                "spec",
            ),
        );
        assert_eq!(
            canonical_nan.verdict(),
            DifferentialVerdict::PrimitiveCompletionsMatch
        );
    }

    #[test]
    fn v2_report_serializes_only_its_versioned_primitive_contract() {
        let report = compare_executions(
            &case_v2(),
            completed(
                DifferentialBackend::WasmAot,
                ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(3.0))),
                "wasm number",
            ),
            completed(
                DifferentialBackend::SpecExec,
                ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(3.0))),
                "spec number",
            ),
        );
        let json: serde_json::Value =
            serde_json::from_str(&report.to_pretty_json().expect("v2 report should encode"))
                .expect("v2 report should be JSON");

        assert_eq!(json["schema_version"], 2);
        assert_eq!(
            json["observation_contract"],
            "primitive_completion_no_output"
        );
        assert_eq!(json["verdict"], "primitive_completions_match");
        assert_eq!(json["semantic_equivalence"], "not_established");
        assert_eq!(
            json["compared_dimensions"],
            serde_json::json!(["completion_kind", "primitive_value"])
        );
        assert_eq!(
            json["wasm_aot"]["execution"],
            serde_json::json!({
                "disposition": "primitive_completion",
                "completion": {
                    "kind": "normal",
                    "value": {
                        "type": "number",
                        "bits": "4008000000000000"
                    }
                },
                "backend_note": "wasm number"
            })
        );

        let bigint_report = compare_executions(
            &case_v2(),
            completed(
                DifferentialBackend::WasmAot,
                ObservedCompletion::Normal(ObservedJsValue::BigInt(
                    "-9007199254740993".to_string().into_boxed_str(),
                )),
                "wasm bigint",
            ),
            completed(
                DifferentialBackend::SpecExec,
                ObservedCompletion::Normal(ObservedJsValue::BigInt(
                    "-9007199254740993".to_string().into_boxed_str(),
                )),
                "spec bigint",
            ),
        );
        let bigint_json: serde_json::Value = serde_json::from_str(
            &bigint_report
                .to_pretty_json()
                .expect("v2 BigInt report should encode"),
        )
        .expect("v2 BigInt report should be JSON");
        assert_eq!(
            bigint_json["wasm_aot"]["execution"]["completion"]["value"],
            serde_json::json!({
                "type": "bigint",
                "decimal": "-9007199254740993"
            })
        );
    }

    #[test]
    fn v2_matching_primitive_throws_are_green_but_kind_is_compared() {
        let matching_throw = compare_executions(
            &case_v2(),
            completed(
                DifferentialBackend::WasmAot,
                ObservedCompletion::Throw(ObservedJsValue::Boolean(false)),
                "wasm throw",
            ),
            completed(
                DifferentialBackend::SpecExec,
                ObservedCompletion::Throw(ObservedJsValue::Boolean(false)),
                "spec throw",
            ),
        );
        assert_eq!(
            matching_throw.verdict(),
            DifferentialVerdict::PrimitiveCompletionsMatch
        );

        let different_kind = compare_executions(
            &case_v2(),
            completed(
                DifferentialBackend::WasmAot,
                ObservedCompletion::Normal(ObservedJsValue::Boolean(false)),
                "wasm normal",
            ),
            completed(
                DifferentialBackend::SpecExec,
                ObservedCompletion::Throw(ObservedJsValue::Boolean(false)),
                "spec throw",
            ),
        );
        assert_eq!(different_kind.verdict(), DifferentialVerdict::Mismatch);
    }

    #[test]
    fn v2_preserves_signed_zero_in_a_pinned_mismatch_signature() {
        let report = compare_executions(
            &case_v2(),
            completed(
                DifferentialBackend::WasmAot,
                ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(0.0))),
                "wasm",
            ),
            completed(
                DifferentialBackend::SpecExec,
                ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(-0.0))),
                "spec",
            ),
        );

        assert_eq!(report.verdict(), DifferentialVerdict::Mismatch);
        assert_eq!(
            report
                .mismatch_signature()
                .expect("v2 mismatch should have a signature")
                .as_str(),
            "lila-diff-v2:primitive-completion:t25/foundation/primitive-number:fnv1a64:71825b2dcabbbc2c:script:wasm-aot=normal-number-0000000000000000:spec-exec=normal-number-8000000000000000"
        );
    }

    #[test]
    fn v2_rejects_symbol_and_object_observations() {
        for value in [ObservedJsValue::Symbol, ObservedJsValue::Object] {
            let report = compare_executions(
                &case_v2(),
                completed(
                    DifferentialBackend::WasmAot,
                    ObservedCompletion::Normal(value.clone()),
                    "wasm type only",
                ),
                completed(
                    DifferentialBackend::SpecExec,
                    ObservedCompletion::Normal(value),
                    "spec type only",
                ),
            );

            assert_eq!(
                report.verdict(),
                DifferentialVerdict::ObservationContractViolated
            );
            assert!(!report.is_green());
            assert!(report.mismatch_signature().is_none());
        }
    }

    #[test]
    fn v2_shared_engine_failures_are_red() {
        let report = compare_executions(
            &case_v2(),
            failed(
                DifferentialBackend::WasmAot,
                FailurePhase::WasmRuntimeOrBackend,
                "wasm failed",
            ),
            failed(
                DifferentialBackend::SpecExec,
                FailurePhase::SpecExecExecution,
                "spec failed",
            ),
        );

        assert_eq!(report.verdict(), DifferentialVerdict::BothFailed);
        assert!(!report.is_green());
    }

    #[cfg(not(feature = "spec-exec-oracle"))]
    #[test]
    fn replay_requires_the_compile_time_oracle_gate() {
        let error = replay_case(&case_v1(), SpecExecOracle::explicitly_enabled())
            .expect_err("default build must not link spec-exec");
        assert!(matches!(error, DifferentialError::OracleNotLinked));
    }

    #[cfg(feature = "spec-exec-oracle")]
    #[test]
    fn committed_v1_foundation_case_replays_through_both_backends() {
        let report = replay_case(&case_v1(), SpecExecOracle::explicitly_enabled())
            .expect("both explicitly enabled backends should run");

        assert_eq!(report.verdict(), DifferentialVerdict::BothCompleted);
        assert!(report.is_green());
    }

    #[cfg(feature = "spec-exec-oracle")]
    #[test]
    fn committed_v2_primitive_case_replays_through_both_backends() {
        let report = replay_case(&case_v2(), SpecExecOracle::explicitly_enabled())
            .expect("both explicitly enabled backends should run");

        assert_eq!(
            report.verdict(),
            DifferentialVerdict::PrimitiveCompletionsMatch
        );
        assert!(report.is_green());
        for observation in [report.wasm_aot(), report.spec_exec()] {
            assert!(matches!(
                &observation.execution,
                ExecutionObservation::PrimitiveCompletion {
                    completion: PrimitiveCompletionObservation::Normal {
                        value: PrimitiveValueObservation::Number { bits }
                    },
                    ..
                } if bits == "4008000000000000"
            ));
        }
    }
}
