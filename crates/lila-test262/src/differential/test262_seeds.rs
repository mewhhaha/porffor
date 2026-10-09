//! Snapshot-selected Test262 replay retains the original harness domain. It
//! cannot be converted to the primitive-completion corpus by stripping source.
use super::{DifferentialError, DifferentialWorkerRunner, SpecExecOracle};
use crate::{
    CompilerProvenance, FailureKind, FailureOrigin, OutcomeKind, PinnedRevisions, SuiteConfig,
    TestExecutionId,
};
use lila_engine::ExecutionBackend;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

#[cfg(test)]
mod tests;
#[cfg(feature = "spec-exec-oracle")]
pub(super) mod worker;

const SCHEMA: u32 = 1;
pub const MAX_TEST262_REPLAY_SEEDS: usize = 128;
const MAX_SEED_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Test262SeedSelection {
    CandidateFailures,
    NewlyGreen,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Evidence {
    base: String,
    candidate: String,
    selection: Test262SeedSelection,
    base_identity: CompilerProvenance,
    candidate_identity: CompilerProvenance,
    pins: PinnedRevisions,
    manifest_hash: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum NegativePhase {
    Parse,
    Early,
    Resolution,
    Runtime,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Negative {
    phase: NegativePhase,
    error_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum HostRequirement {
    None,
    RawUnmaterialized,
    Complete { realm: bool, agent_worker: bool },
    Unresolved { message: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum HarnessOrigin {
    LocalMerged,
    VendoredHarness,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MaterializedSource {
    source: String,
    module_prelude: Option<String>,
    agent_prelude: Option<String>,
    used_preludes: Vec<(String, HarnessOrigin)>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
enum Materialization {
    Ready { actual: MaterializedSource },
    Unavailable { message: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SeedWire {
    schema_version: u32,
    suite_root: PathBuf,
    snapshot_dir: PathBuf,
    timeout_ms: u64,
    evidence: Evidence,
    execution_id: TestExecutionId,
    original_source: String,
    flags: BTreeSet<String>,
    features: BTreeSet<String>,
    includes: Vec<String>,
    negative: Option<Negative>,
    host: HostRequirement,
    original_failure: HistoricalFailure,
    suite_sha256: String,
    wasm_aot: Materialization,
    spec_exec: Materialization,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct HistoricalFailure {
    kind: FailureKind,
    outcome: OutcomeKind,
    origin: FailureOrigin,
    normalized_detail: String,
    detail_hash: u64,
    ownership: crate::BacklogOwnership,
}

/// The only public replay input is minted from verified snapshot membership,
/// original frontmatter and the two actual embedded harness materializations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct Test262ReplaySeed {
    wire: SeedWire,
}

struct SelectedEvidence {
    evidence: Evidence,
    ids: BTreeSet<TestExecutionId>,
    failures: BTreeMap<TestExecutionId, crate::FailureRecord>,
}
fn invalid(message: impl Into<String>) -> DifferentialError {
    DifferentialError::InvalidCorpus(message.into())
}

fn evidence(
    config: &SuiteConfig,
    base: &str,
    candidate: &str,
    selection: Test262SeedSelection,
) -> Result<SelectedEvidence, DifferentialError> {
    let comparison = crate::compare_snapshots(config, base, candidate, ExecutionBackend::WasmAot)
        .map_err(invalid)?;
    let verified =
        crate::load_comparison_aggregate_summary(config, candidate, ExecutionBackend::WasmAot)
            .map_err(invalid)?;
    let (ids, failures) = match selection {
        Test262SeedSelection::NewlyGreen => {
            let base =
                crate::load_comparison_aggregate_summary(config, base, ExecutionBackend::WasmAot)
                    .map_err(invalid)?;
            let failures = crate::load_failure_map(config, &base, ExecutionBackend::WasmAot)
                .map_err(invalid)?;
            (comparison.added_passes.into_iter().collect(), failures)
        }
        Test262SeedSelection::CandidateFailures => {
            let failures = crate::load_failure_map(config, &verified, ExecutionBackend::WasmAot)
                .map_err(invalid)?;
            if failures.len() != verified.summary.failed {
                return Err(invalid(
                    "candidate failure inventory disagrees with the verified snapshot",
                ));
            }
            (failures.keys().cloned().collect(), failures)
        }
    };
    Ok(SelectedEvidence {
        evidence: Evidence {
            base: base.into(),
            candidate: candidate.into(),
            selection,
            base_identity: comparison.base_compiler_identity,
            candidate_identity: comparison.candidate_compiler_identity,
            pins: comparison.pinned_revisions,
            manifest_hash: verified.manifest_hash,
        },
        ids,
        failures,
    })
}

fn backend_config(config: &SuiteConfig, backend: ExecutionBackend) -> SuiteConfig {
    SuiteConfig {
        local_harness: match backend {
            ExecutionBackend::WasmAot => crate::LocalHarnessSource::EmbeddedWasmAot,
            ExecutionBackend::SpecExec => crate::LocalHarnessSource::EmbeddedSpecExec,
        },
        worker_count: 1,
        case_runner_bin: None,
        ..config.clone()
    }
}

fn materialize(case: &crate::TestCase, config: &SuiteConfig) -> Materialization {
    let actual = match crate::load_preludes(config).and_then(|p| crate::materialize_test(case, &p))
    {
        Ok(actual) => actual,
        Err(message) => return Materialization::Unavailable { message },
    };
    Materialization::Ready {
        actual: MaterializedSource {
            source: actual.source,
            module_prelude: actual.module_prelude,
            agent_prelude: actual.agent_prelude,
            used_preludes: actual
                .used_preludes
                .into_iter()
                .map(|(name, origin)| {
                    (
                        name,
                        match origin {
                            crate::PreludeOrigin::LocalMerged => HarnessOrigin::LocalMerged,
                            crate::PreludeOrigin::VendoredHarness => HarnessOrigin::VendoredHarness,
                        },
                    )
                })
                .collect(),
        },
    }
}

fn case_proof(
    config: &SuiteConfig,
    selected: &SelectedEvidence,
    id: &TestExecutionId,
    suite_sha256: String,
) -> Result<Test262ReplaySeed, DifferentialError> {
    if config.timeout_ms == 0 || config.timeout_ms > 600_000 {
        return Err(invalid(
            "Test262 replay timeout must be 1..=600000 milliseconds",
        ));
    }
    if !selected.ids.contains(id) {
        return Err(invalid("execution is not in the selected snapshot domain"));
    }
    let manifest = crate::discover_suite(config, Some(&id.wire_key())).map_err(invalid)?;
    let case = manifest
        .cases
        .into_iter()
        .next()
        .ok_or_else(|| invalid("missing selected execution"))?;
    let historical = selected
        .failures
        .get(id)
        .ok_or_else(|| invalid("selected execution lacks its original failure record"))?;
    let owner_rules = crate::load_backlog_owner_rules(config).map_err(invalid)?;
    let normalized_detail = crate::normalize_backlog_detail(&historical.detail);
    let original_failure = HistoricalFailure {
        kind: historical.kind,
        outcome: historical.outcome,
        origin: historical.origin,
        detail_hash: crate::hash_detail(&normalized_detail),
        normalized_detail,
        ownership: crate::classify_backlog_owner(historical, Some(&case), &owner_rules),
    };
    let wasm = backend_config(config, ExecutionBackend::WasmAot);
    let spec = backend_config(config, ExecutionBackend::SpecExec);
    let preludes = crate::load_preludes(&wasm).map_err(invalid)?;
    let host = if case.execution_mode().is_raw() {
        HostRequirement::RawUnmaterialized
    } else {
        match crate::resolve_declared_preludes(&case, &preludes) {
            Err(message) => HostRequirement::Unresolved { message },
            Ok(declared) => match crate::test262_host_requirement(&case, &declared) {
                crate::Test262HostRequirement::None => HostRequirement::None,
                crate::Test262HostRequirement::Complete {
                    realm,
                    agent_worker,
                } => HostRequirement::Complete {
                    realm: realm == crate::WasmAotRealmActivation::Active,
                    agent_worker: agent_worker == crate::AgentWorkerRequirement::Required,
                },
            },
        }
    };
    let negative = case.negative.as_ref().map(|expected| Negative {
        phase: match expected.phase {
            crate::NegativePhase::Parse => NegativePhase::Parse,
            crate::NegativePhase::Early => NegativePhase::Early,
            crate::NegativePhase::Resolution => NegativePhase::Resolution,
            crate::NegativePhase::Runtime => NegativePhase::Runtime,
        },
        error_type: expected.error_type.clone(),
    });
    Ok(Test262ReplaySeed {
        wire: SeedWire {
            schema_version: SCHEMA,
            suite_root: fs::canonicalize(&config.suite_root).map_err(|e| invalid(e.to_string()))?,
            snapshot_dir: fs::canonicalize(&config.snapshot_dir)
                .map_err(|e| invalid(e.to_string()))?,
            timeout_ms: config.timeout_ms,
            evidence: selected.evidence.clone(),
            execution_id: id.clone(),
            original_source: case.original_source.to_string(),
            flags: case.flags.clone(),
            features: case.features.clone(),
            includes: case.includes.clone(),
            negative,
            host,
            original_failure,
            suite_sha256,
            wasm_aot: materialize(&case, &wasm),
            spec_exec: materialize(&case, &spec),
        },
    })
}

impl Test262ReplaySeed {
    pub fn execution_id(&self) -> &TestExecutionId {
        &self.wire.execution_id
    }
    pub fn selection(&self) -> Test262SeedSelection {
        self.wire.evidence.selection
    }
    pub fn to_pretty_json(&self) -> Result<String, DifferentialError> {
        serde_json::to_string_pretty(self)
            .map_err(|e| DifferentialError::DecodeCorpus(e.to_string()))
    }
    pub fn from_json(json: &str) -> Result<Self, DifferentialError> {
        if json.len() > MAX_SEED_BYTES {
            return Err(invalid("Test262 replay seed exceeds 16 MiB"));
        }
        let wire: SeedWire = serde_json::from_str(json)
            .map_err(|e| DifferentialError::DecodeCorpus(e.to_string()))?;
        let raw: serde_json::Value =
            serde_json::from_str(json).map_err(|e| invalid(e.to_string()))?;
        let canonical = serde_json::to_value(&wire).map_err(|e| invalid(e.to_string()))?;
        if raw != canonical {
            return Err(invalid(
                "Test262 seed contains noncanonical or unknown nested fields",
            ));
        }
        if wire.schema_version != SCHEMA {
            return Err(invalid("unknown Test262 replay seed schema"));
        }
        let config = SuiteConfig {
            suite_root: wire.suite_root.clone(),
            snapshot_dir: wire.snapshot_dir.clone(),
            timeout_ms: wire.timeout_ms,
            ..SuiteConfig::default()
        };
        let selected = evidence(
            &config,
            &wire.evidence.base,
            &wire.evidence.candidate,
            wire.evidence.selection,
        )?;
        let actual = case_proof(
            &config,
            &selected,
            &wire.execution_id,
            suite_digest(&config.suite_root)?,
        )?;
        admit_rederived(actual, wire)
    }
    pub fn load(path: &Path) -> Result<Self, DifferentialError> {
        let file = fs::File::open(path).map_err(|e| invalid(e.to_string()))?;
        let mut bytes = Vec::new();
        file.take(MAX_SEED_BYTES as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| invalid(e.to_string()))?;
        let json = std::str::from_utf8(&bytes).map_err(|e| invalid(e.to_string()))?;
        Self::from_json(json)
    }
    pub(super) fn fingerprint(&self) -> String {
        let bytes = serde_json::to_vec(&self.wire).expect("closed replay wire serializes");
        format!("{:x}", Sha256::digest(bytes))
    }
}

fn admit_rederived(
    actual: Test262ReplaySeed,
    retained: SeedWire,
) -> Result<Test262ReplaySeed, DifferentialError> {
    if actual.wire != retained {
        return Err(invalid(
            "Test262 seed differs from original suite, harness or snapshot admission",
        ));
    }
    Ok(actual)
}

/// Bounded deterministic prefix of a complete snapshot-selected inventory.
/// Unselected members are counted explicitly; this is not full-suite status.
pub struct Test262SeedPlan {
    seeds: Vec<Test262ReplaySeed>,
    eligible: usize,
}
impl Test262SeedPlan {
    pub fn new(
        config: &SuiteConfig,
        base: &str,
        candidate: &str,
        selection: Test262SeedSelection,
        maximum: usize,
    ) -> Result<Self, DifferentialError> {
        if !(1..=MAX_TEST262_REPLAY_SEEDS).contains(&maximum) {
            return Err(invalid("seed limit must be 1..=128"));
        }
        if !matches!(
            &config.local_harness,
            crate::LocalHarnessSource::EmbeddedSpecExec
                | crate::LocalHarnessSource::EmbeddedWasmAot
        ) {
            return Err(invalid(
                "seed bridge requires full canonical embedded backend harness profiles",
            ));
        }
        let selected = evidence(config, base, candidate, selection)?;
        if selected.ids.is_empty() {
            return Err(invalid("selected snapshot domain contains no executions"));
        }
        let digest = suite_digest(&config.suite_root)?;
        let seeds = selected
            .ids
            .iter()
            .take(maximum)
            .map(|id| case_proof(config, &selected, id, digest.clone()))
            .collect::<Result<Vec<_>, _>>()?;
        if suite_digest(&config.suite_root)? != digest {
            return Err(invalid("suite changed during seed materialization"));
        }
        Ok(Self {
            seeds,
            eligible: selected.ids.len(),
        })
    }
    pub fn seeds(&self) -> &[Test262ReplaySeed] {
        &self.seeds
    }
    pub fn eligible(&self) -> usize {
        self.eligible
    }
    pub fn write(&self, output: &Path) -> Result<(), DifferentialError> {
        fs::create_dir(output).map_err(|e| invalid(e.to_string()))?;
        let mut manifest = BridgeManifest {
            schema_version: SCHEMA,
            state: BridgeState::Incomplete,
            eligible: self.eligible,
            selected: self.seeds.len(),
            retained: 0,
            seeds: Vec::new(),
        };
        write_json(&output.join("bridge.json"), &manifest)?;
        for (index, seed) in self.seeds.iter().enumerate() {
            let name = format!("{index:04}.test262-seed.json");
            write_json(&output.join(&name), seed)?;
            manifest
                .seeds
                .push((name, seed.execution_id().clone(), seed.fingerprint()));
            manifest.retained += 1;
            write_json(&output.join("bridge.json"), &manifest)?;
        }
        manifest.state = BridgeState::Complete;
        write_json(&output.join("bridge.json"), &manifest)
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum BridgeState {
    Incomplete,
    Complete,
}
#[derive(Serialize)]
struct BridgeManifest {
    schema_version: u32,
    state: BridgeState,
    eligible: usize,
    selected: usize,
    retained: usize,
    seeds: Vec<(String, TestExecutionId, String)>,
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<(), DifferentialError> {
    use std::io::Write;
    let bytes = serde_json::to_vec_pretty(value).map_err(|e| invalid(e.to_string()))?;
    let temporary = path.with_extension("pending");
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|e| invalid(e.to_string()))?;
    file.write_all(&bytes)
        .and_then(|()| file.sync_all())
        .map_err(|e| invalid(e.to_string()))?;
    fs::rename(temporary, path).map_err(|e| invalid(e.to_string()))
}

/// Hash every retained suite file, including Module dependencies and host
/// fixtures, with path/length framing. Missing/unreadable/symlink entries fail.
fn suite_digest(root: &Path) -> Result<String, DifferentialError> {
    fn visit(
        root: &Path,
        directory: &Path,
        depth: usize,
        files: &mut Vec<PathBuf>,
    ) -> Result<(), DifferentialError> {
        if depth > 64 {
            return Err(invalid("suite dependency depth exceeds 64"));
        }
        for entry in fs::read_dir(directory).map_err(|e| invalid(e.to_string()))? {
            let path = entry.map_err(|e| invalid(e.to_string()))?.path();
            let meta = fs::symlink_metadata(&path).map_err(|e| invalid(e.to_string()))?;
            if meta.file_type().is_symlink() {
                return Err(invalid("suite seed dependencies cannot be symlinks"));
            }
            if meta.is_dir() {
                visit(root, &path, depth + 1, files)?;
            } else if meta.is_file() {
                if files.len() == 200_000 {
                    return Err(invalid("suite dependency inventory exceeds 200000 files"));
                }
                files.push(
                    path.strip_prefix(root)
                        .map_err(|e| invalid(e.to_string()))?
                        .to_owned(),
                );
            } else {
                return Err(invalid("unsupported suite dependency entry"));
            }
        }
        Ok(())
    }
    let mut files = Vec::new();
    visit(root, root, 0, &mut files)?;
    files.sort();
    let mut hash = Sha256::new();
    let mut total = 0u64;
    let mut block = [0u8; 65536];
    for relative in files {
        let name = relative
            .to_str()
            .ok_or_else(|| invalid("suite path is not UTF-8"))?;
        hash.update((name.len() as u64).to_le_bytes());
        hash.update(name.as_bytes());
        let mut file = fs::File::open(root.join(relative)).map_err(|e| invalid(e.to_string()))?;
        let before = file.metadata().map_err(|e| invalid(e.to_string()))?;
        total = total
            .checked_add(before.len())
            .ok_or_else(|| invalid("suite size overflow"))?;
        if total > 512 * 1024 * 1024 {
            return Err(invalid("suite dependency bytes exceed 512 MiB"));
        }
        hash.update(before.len().to_le_bytes());
        let mut read = 0u64;
        loop {
            let n = file.read(&mut block).map_err(|e| invalid(e.to_string()))?;
            if n == 0 {
                break;
            }
            read += n as u64;
            if read > before.len() {
                return Err(invalid("suite file changed while reading"));
            }
            hash.update(&block[..n]);
        }
        if read != before.len()
            || file
                .metadata()
                .map_err(|e| invalid(e.to_string()))?
                .modified()
                .ok()
                != before.modified().ok()
        {
            return Err(invalid("suite file changed while reading"));
        }
    }
    Ok(format!("{:x}", hash.finalize()))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
pub enum Test262ReplayResult {
    Passed {
        #[serde(with = "worker_duration_ms")]
        duration_ms: u128,
    },
    Failed {
        kind: FailureKind,
        outcome: OutcomeKind,
        origin: FailureOrigin,
        detail: String,
        detail_hash: u64,
        #[serde(with = "worker_duration_ms")]
        duration_ms: u128,
    },
    AdmissionRejected {
        message: String,
    },
    WorkerFailure {
        failure: super::DifferentialWorkerFailure,
        cleanup_error: Option<String>,
    },
}

// Internally tagged Serde frames buffer integer tokens as u64. Keep the
// in-process duration type while admitting an exact, bounded wire integer;
// neither JSON floating-point fallback nor serialization truncation is allowed.
mod worker_duration_ms {
    use serde::{Deserialize, Deserializer, Serializer};

    pub(super) fn serialize<S: Serializer>(value: &u128, serializer: S) -> Result<S::Ok, S::Error> {
        let value = u64::try_from(*value).map_err(serde::ser::Error::custom)?;
        serializer.serialize_u64(value)
    }

    pub(super) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<u128, D::Error> {
        u64::deserialize(deserializer).map(u128::from)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Test262BackendReplay {
    pub compiler_identity: Option<CompilerProvenance>,
    pub result: Test262ReplayResult,
    pub journal_bytes_hex: String,
    pub stderr_bytes_hex: String,
}
#[derive(Debug, Clone, Serialize)]
pub struct Test262ReplayReport {
    schema_version: u32,
    state: BridgeState,
    seed: Test262ReplaySeed,
    wasm_aot: Test262BackendReplay,
    spec_exec: Test262BackendReplay,
    observations_match: bool,
}
impl Test262ReplayReport {
    pub fn wasm_aot(&self) -> &Test262BackendReplay {
        &self.wasm_aot
    }
    pub fn spec_exec(&self) -> &Test262BackendReplay {
        &self.spec_exec
    }
    pub fn observations_match(&self) -> bool {
        self.observations_match
    }
    pub fn is_green(&self) -> bool {
        matches!(self.wasm_aot.result, Test262ReplayResult::Passed { .. })
            && matches!(self.spec_exec.result, Test262ReplayResult::Passed { .. })
    }
}
fn equivalent(a: &Test262ReplayResult, b: &Test262ReplayResult) -> bool {
    match (a, b) {
        (Test262ReplayResult::Passed { .. }, Test262ReplayResult::Passed { .. }) => true,
        (
            Test262ReplayResult::Failed {
                kind: a,
                outcome: ao,
                origin: ar,
                detail: ad,
                ..
            },
            Test262ReplayResult::Failed {
                kind: b,
                outcome: bo,
                origin: br,
                detail: bd,
                ..
            },
        ) => {
            a == b
                && ao == bo
                && ar == br
                && crate::normalize_backlog_detail(ad) == crate::normalize_backlog_detail(bd)
        }
        _ => false,
    }
}

pub fn replay_test262_seed(
    seed: &Test262ReplaySeed,
    oracle: SpecExecOracle,
    runner: &DifferentialWorkerRunner,
) -> Result<Test262ReplayReport, DifferentialError> {
    #[cfg(not(feature = "spec-exec-oracle"))]
    {
        let _ = (seed, oracle, runner);
        Err(DifferentialError::OracleNotLinked)
    }
    #[cfg(feature = "spec-exec-oracle")]
    {
        let wasm_aot = runner.run_test262(seed, super::DifferentialBackend::WasmAot, oracle)?;
        let spec_exec = runner.run_test262(seed, super::DifferentialBackend::SpecExec, oracle)?;
        let observations_match = equivalent(&wasm_aot.result, &spec_exec.result);
        Ok(Test262ReplayReport {
            schema_version: SCHEMA,
            state: BridgeState::Complete,
            seed: seed.clone(),
            wasm_aot,
            spec_exec,
            observations_match,
        })
    }
}
