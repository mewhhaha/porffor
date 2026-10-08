//! Measurements accompany, and never replace, the original matrix evidence.
use super::*;
use sha2::{Digest, Sha256};
use std::io::{Read, Write};

const VERSION: u32 = 1;
const SNAPSHOT_LIMIT: u64 = 64 * 1024 * 1024;
const TIMING_LIMIT: u64 = 64 * 1024;
const SCOPE: &str = "last-completed-node-invocation;execute-cases-and-summarize;includes-resume-and-checkpoint-io;excludes-discovery-and-terminal-publication";

/// Only the executing node can mint a measured interval. Reconstructed or
/// resumed case durations are never added together to invent wall-clock time.
pub(super) struct NodeInvocationTimer {
    start: Instant,
    settings: TimingSettings,
}
impl NodeInvocationTimer {
    pub(super) fn start(config: &SuiteConfig, run: &RunConfig) -> Self {
        Self {
            start: Instant::now(),
            settings: TimingSettings {
                resume_requested: run.resume,
                configured_workers: config.worker_count,
                timeout_ms: config.timeout_ms,
                os: std::env::consts::OS.into(),
                architecture: std::env::consts::ARCH.into(),
            },
        }
    }
    pub(super) fn finish(self) -> FinishedNodeInvocation {
        FinishedNodeInvocation {
            elapsed_ns: self.start.elapsed().as_nanos(),
            settings: self.settings,
        }
    }
}
pub(super) struct FinishedNodeInvocation {
    elapsed_ns: u128,
    settings: TimingSettings,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TimingSettings {
    resume_requested: bool,
    configured_workers: usize,
    timeout_ms: u64,
    os: String,
    architecture: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct NodeTiming {
    version: u32,
    scope: String,
    compiler: CompilerProvenance,
    execution_backend: String,
    ecma262_revision: String,
    test262_revision: String,
    manifest_hash: u64,
    node_id: String,
    snapshot_sha256: String,
    elapsed_ns: u128,
    settings: TimingSettings,
}
impl NodeTiming {
    fn admit(
        &self,
        snapshot: &ProgressSnapshot,
        node: &RunMatrixNode,
        bytes: &[u8],
    ) -> Result<(), String> {
        if self.version != VERSION
            || self.scope != SCOPE
            || &self.compiler != snapshot.provenance.require_current("node timing")?
            || self.execution_backend != snapshot.execution_backend.as_str()
            || self.ecma262_revision != snapshot.pinned_revisions.ecma262
            || self.test262_revision != snapshot.pinned_revisions.test262
            || self.manifest_hash != snapshot.manifest_hash
            || self.node_id != node.node_id
            || self.snapshot_sha256 != digest(bytes)
            || self.settings.os.is_empty()
            || self.settings.architecture.is_empty()
        {
            return Err("node timing identity or exact snapshot binding differs from admitted matrix evidence".into());
        }
        Ok(())
    }
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
// Snapshot inventory/resume owns the .json extension. Timing JSON has a
// separate extension so existing discovery cannot mistake it for a snapshot.
fn timing_path(snapshot: &Path) -> PathBuf {
    snapshot.with_extension("timing")
}

pub(super) fn write_timing(
    path: &Path,
    snapshot: &ProgressSnapshot,
    node: &RunMatrixNode,
    measured: FinishedNodeInvocation,
) -> Result<(), String> {
    let timing = NodeTiming {
        version: VERSION,
        scope: SCOPE.into(),
        compiler: snapshot
            .provenance
            .require_running("node timing writer")?
            .clone(),
        execution_backend: snapshot.execution_backend.as_str().into(),
        ecma262_revision: snapshot.pinned_revisions.ecma262.clone(),
        test262_revision: snapshot.pinned_revisions.test262.clone(),
        manifest_hash: snapshot.manifest_hash,
        node_id: node.node_id.clone(),
        snapshot_sha256: digest(render_snapshot_json(snapshot).as_bytes()),
        elapsed_ns: measured.elapsed_ns,
        settings: measured.settings,
    };
    let bytes = serde_json::to_vec_pretty(&timing).map_err(|error| error.to_string())?;
    let path = timing_path(path);
    static SEQUENCE: AtomicU64 = AtomicU64::new(0);
    let temporary = path.with_extension(format!(
        "tmp-{}-{}",
        std::process::id(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));
    let result = (|| -> Result<(), String> {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|error| {
                format!(
                    "cannot create timing transaction {}: {error}",
                    temporary.display()
                )
            })?;
        file.write_all(&bytes)
            .and_then(|()| file.sync_all())
            .map_err(|error| error.to_string())?;
        drop(file);
        fs::rename(&temporary, &path)
            .map_err(|error| format!("cannot publish node timing {}: {error}", path.display()))
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn read_bounded(path: &Path, limit: u64) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(|error| format!("cannot open {}: {error}", path.display()))?
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() as u64 > limit {
        return Err(format!(
            "performance evidence {} exceeds its byte bound",
            path.display()
        ));
    }
    Ok(bytes)
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "kebab-case")]
enum WallTiming {
    Available {
        record: NodeTiming,
        sidecar_path: PathBuf,
        sidecar_sha256: String,
    },
    Unavailable {
        reason: &'static str,
    },
}
#[derive(Debug, Serialize)]
struct SlowExecution {
    execution: TestExecutionId,
    duration_ms: u128,
}
#[derive(Debug, Serialize)]
struct NodeEvidence {
    node_id: String,
    snapshot_path: PathBuf,
    snapshot_sha256: String,
    total: usize,
    passed: usize,
    timeouts: Vec<TestExecutionId>,
    retained_slowest: Vec<SlowExecution>,
    wall_timing: WallTiming,
}
/// Read-only report: callers cannot construct a report asserting unverified
/// completion or mutate timeout/identity evidence before publication.
#[derive(Debug, Serialize)]
pub struct ConformancePerformanceEvidence {
    version: u32,
    scope: &'static str,
    compiler: CompilerProvenance,
    execution_backend: String,
    ecma262_revision: String,
    test262_revision: String,
    snapshot_name: String,
    aggregate_snapshot: PathBuf,
    aggregate_sha256: String,
    matrix_complete: bool,
    expected_nodes: usize,
    completed_nodes: usize,
    total: usize,
    passed: usize,
    timeout_count: usize,
    nodes_with_wall_timing: usize,
    /// Sum of the retained completed invocation intervals only. Parallel case
    /// durations, earlier resume invocations and unfinished nodes are excluded.
    measured_invocation_sum_ns: u128,
    retained_slowest: Vec<SlowExecution>,
    nodes: Vec<NodeEvidence>,
}
impl ConformancePerformanceEvidence {
    pub fn matrix_complete(&self) -> bool {
        self.matrix_complete
    }
    pub fn timeout_count(&self) -> usize {
        self.timeout_count
    }
    pub fn nodes_with_wall_timing(&self) -> usize {
        self.nodes_with_wall_timing
    }
    pub fn to_pretty_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|error| error.to_string())
    }
}

pub(super) fn load(
    config: &SuiteConfig,
    name: &str,
    backend: ExecutionBackend,
) -> Result<ConformancePerformanceEvidence, String> {
    // This original reader verifies pins, compiler, matrix membership, exact
    // execution modes, timeout classifications and every completed node.
    let progress = load_aggregate_progress_summary(config, name, backend)?;
    let matrix = load_or_build_run_matrix(config, backend)?;
    let aggregate_bytes = read_bounded(&progress.snapshot_paths.json_path, SNAPSHOT_LIMIT)?;
    let aggregate = snapshot_from_file(decode_snapshot_bytes(
        &aggregate_bytes,
        &progress.snapshot_paths.json_path,
    )?)?;
    if aggregate
        .provenance
        .require_current("performance aggregate")?
        != &progress.compiler_identity
        || aggregate_summary_from_snapshot(&aggregate) != progress.summary
        || aggregate.manifest_hash != progress.manifest_hash
        || aggregate.execution_backend != backend
        || aggregate.pinned_revisions != progress.recorded_pinned_revisions
    {
        return Err("aggregate changed while reading performance evidence".into());
    }
    let recorded_nodes = matrix
        .iter()
        .filter(|node| aggregate.completed_nodes.contains(&node.node_id))
        .cloned()
        .collect::<Vec<_>>();
    validate_complete_aggregate_contract(
        &aggregate,
        &recorded_nodes,
        &progress.snapshot_paths.json_path,
    )?;
    let mut report = ConformancePerformanceEvidence {
        version: VERSION,
        scope: "verified-completed-matrix-nodes;retained-top-ten-per-node;missing-timing-is-unavailable;not-a-throughput-benchmark",
        compiler: progress.compiler_identity.clone(), execution_backend: backend.as_str().into(),
        ecma262_revision: progress.recorded_pinned_revisions.ecma262.clone(),
        test262_revision: progress.recorded_pinned_revisions.test262.clone(),
        snapshot_name: progress.resolved_snapshot_name.clone(),
        aggregate_snapshot: progress.snapshot_paths.json_path.clone(),
        aggregate_sha256: digest(&aggregate_bytes), matrix_complete: progress.is_complete(),
        expected_nodes: progress.matrix_nodes_total(), completed_nodes: progress.matrix_nodes_completed(),
        total: progress.summary.total, passed: progress.summary.passed, timeout_count: 0,
        nodes_with_wall_timing: 0, measured_invocation_sum_ns: 0,
        retained_slowest: Vec::new(), nodes: Vec::new(),
    };
    for entry in &progress.summary.entries {
        let node = matrix
            .iter()
            .find(|node| node.node_id == entry.node_id)
            .ok_or("unknown admitted matrix node")?;
        let (path, admitted_file) = locate_node_snapshot(
            config,
            &progress.resolved_snapshot_name,
            node,
            backend,
            entry,
            &progress.compiler_identity,
        )?
        .ok_or_else(|| format!("missing performance node {}", node.node_id))?;
        let admitted_snapshot = snapshot_from_file(admitted_file)?;
        let bytes = read_bounded(&path, SNAPSHOT_LIMIT)?;
        let file = decode_snapshot_bytes(&bytes, &path)?;
        file.require_compiler(&progress.compiler_identity, &path)?;
        let snapshot = snapshot_from_file(file)?;
        validate_complete_node_contract(&snapshot, node, entry, &path)?;
        if snapshot != admitted_snapshot {
            return Err("node identity changed while reading performance evidence".into());
        }
        let sidecar = timing_path(&path);
        let wall_timing = if sidecar.try_exists().map_err(|error| error.to_string())? {
            let timing_bytes = read_bounded(&sidecar, TIMING_LIMIT)?;
            let record: NodeTiming = serde_json::from_slice(&timing_bytes)
                .map_err(|error| format!("invalid node timing {}: {error}", sidecar.display()))?;
            record.admit(&snapshot, node, &bytes)?;
            report.nodes_with_wall_timing += 1;
            report.measured_invocation_sum_ns = report
                .measured_invocation_sum_ns
                .checked_add(record.elapsed_ns)
                .ok_or("node wall duration sum overflow")?;
            WallTiming::Available {
                record,
                sidecar_path: sidecar,
                sidecar_sha256: digest(&timing_bytes),
            }
        } else {
            WallTiming::Unavailable {
                reason: "no measured node invocation sidecar was retained",
            }
        };
        report.timeout_count += snapshot.timeout_list.len();
        report
            .retained_slowest
            .extend(
                snapshot
                    .slowest_tests
                    .iter()
                    .map(|(execution, duration_ms)| SlowExecution {
                        execution: execution.clone(),
                        duration_ms: *duration_ms,
                    }),
            );
        report.nodes.push(NodeEvidence {
            node_id: node.node_id.clone(),
            snapshot_path: path,
            snapshot_sha256: digest(&bytes),
            total: snapshot.total,
            passed: snapshot.passed,
            timeouts: snapshot.timeout_list,
            retained_slowest: snapshot
                .slowest_tests
                .into_iter()
                .map(|(execution, duration_ms)| SlowExecution {
                    execution,
                    duration_ms,
                })
                .collect(),
            wall_timing,
        });
    }
    report.retained_slowest.sort_by(|left, right| {
        right
            .duration_ms
            .cmp(&left.duration_ms)
            .then_with(|| left.execution.cmp(&right.execution))
    });
    report.retained_slowest.truncate(10);
    Ok(report)
}

#[cfg(test)]
mod tests;
