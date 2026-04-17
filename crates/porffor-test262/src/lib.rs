use std::collections::{hash_map::DefaultHasher, BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::hash::{Hash, Hasher};
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use porffor_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};
use serde::{Deserialize, Serialize};

const TOP_LEVEL_FILTERS: [&str; 6] = [
    "annexB",
    "built-ins",
    "harness",
    "intl402",
    "language",
    "staging",
];
const MATRIX_SPLIT_FILTERS: [&str; 4] = ["built-ins", "intl402", "language", "staging"];
const SNAPSHOT_VERSION: u32 = 4;
const MATRIX_STRATEGY_VERSION: u32 = 2;
// Keep matrix nodes small enough that slow semantic buckets like RegExp and
// Temporal checkpoint incrementally instead of monopolizing a whole run.
const MATRIX_RECURSION_THRESHOLD: usize = 500;
const MATRIX_CHUNK_SIZE: usize = 250;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FailureKind {
    Parser,
    EarlyError,
    Lowering,
    Runtime,
    WasmBackend,
    HostHarness,
    Unsupported,
}

impl FailureKind {
    pub const ALL: [FailureKind; 7] = [
        FailureKind::Parser,
        FailureKind::EarlyError,
        FailureKind::Lowering,
        FailureKind::Runtime,
        FailureKind::WasmBackend,
        FailureKind::HostHarness,
        FailureKind::Unsupported,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            FailureKind::Parser => "Parser",
            FailureKind::EarlyError => "EarlyError",
            FailureKind::Lowering => "Lowering",
            FailureKind::Runtime => "Runtime",
            FailureKind::WasmBackend => "WasmBackend",
            FailureKind::HostHarness => "HostHarness",
            FailureKind::Unsupported => "Unsupported",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FailureOrigin {
    Unknown,
    LocalHarness,
    BoaRuntime,
    BoaParser,
    IcuIntl,
    SpecExecHost,
}

impl FailureOrigin {
    pub const ALL: [FailureOrigin; 6] = [
        FailureOrigin::Unknown,
        FailureOrigin::LocalHarness,
        FailureOrigin::BoaRuntime,
        FailureOrigin::BoaParser,
        FailureOrigin::IcuIntl,
        FailureOrigin::SpecExecHost,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            FailureOrigin::Unknown => "unknown",
            FailureOrigin::LocalHarness => "local-harness",
            FailureOrigin::BoaRuntime => "boa-runtime",
            FailureOrigin::BoaParser => "boa-parser",
            FailureOrigin::IcuIntl => "icu-intl",
            FailureOrigin::SpecExecHost => "spec-exec-host",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailureRecord {
    pub test_path: String,
    pub kind: FailureKind,
    pub origin: FailureOrigin,
    pub detail: String,
    pub detail_hash: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinnedRevisions {
    pub ecma262: String,
    pub test262: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuiteConfig {
    pub suite_root: PathBuf,
    pub local_harness_path: PathBuf,
    pub snapshot_dir: PathBuf,
    pub timeout_ms: u64,
    pub worker_count: usize,
}

impl Default for SuiteConfig {
    fn default() -> Self {
        let root = PathBuf::from("test262");
        Self {
            suite_root: root.join("vendor").join("test262"),
            local_harness_path: root.join("harness.js"),
            snapshot_dir: root.join("snapshots"),
            // Conformance buckets now include a few correctness-green but still slow
            // intrinsic graph traversals, so the default timeout must not classify them
            // as harness failures after they complete successfully.
            timeout_ms: 180_000,
            worker_count: thread::available_parallelism()
                .map(|count| count.get().min(4))
                .unwrap_or(4),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NegativeExpectation {
    pub phase: String,
    pub error_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestCase {
    pub path: String,
    pub source_path: PathBuf,
    pub original_source: String,
    pub flags: BTreeSet<String>,
    pub includes: Vec<String>,
    pub negative: Option<NegativeExpectation>,
    pub is_module: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuiteManifest {
    pub pinned_revisions: PinnedRevisions,
    pub manifest_hash: u64,
    pub filter: Option<String>,
    pub cases: Vec<TestCase>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreludeOrigin {
    LocalMerged,
    VendoredHarness,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreludeEntry {
    pub name: String,
    pub contents: String,
    pub origin: PreludeOrigin,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PreludeStore {
    entries: BTreeMap<String, PreludeEntry>,
}

impl PreludeStore {
    pub fn insert(&mut self, name: String, contents: String, origin: PreludeOrigin) {
        self.entries.insert(
            name.clone(),
            PreludeEntry {
                name,
                contents,
                origin,
            },
        );
    }

    pub fn get(&self, name: &str) -> Option<&PreludeEntry> {
        self.entries.get(name)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.entries.contains_key(name)
    }

    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.entries.keys().map(String::as_str)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterializedTest {
    pub path: String,
    pub source: String,
    pub used_preludes: Vec<(String, PreludeOrigin)>,
    pub negative: Option<NegativeExpectation>,
    pub is_module: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunConfig {
    pub filter: Option<String>,
    pub shard_index: usize,
    pub shard_count: usize,
    pub resume: bool,
    pub snapshot_name: String,
    pub execution_backend: ExecutionBackend,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            filter: None,
            shard_index: 0,
            shard_count: 1,
            resume: false,
            snapshot_name: "latest".to_string(),
            execution_backend: ExecutionBackend::SpecExec,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestStatus {
    Passed,
    Failed(FailureRecord),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestResult {
    pub test_path: String,
    pub status: TestStatus,
    pub duration_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShardSummary {
    pub shard_index: usize,
    pub shard_count: usize,
    pub total: usize,
    pub passed: usize,
    pub failures: Vec<FailureRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunSummary {
    pub total: usize,
    pub passed: usize,
    pub counts_per_kind: BTreeMap<FailureKind, usize>,
    pub failures: Vec<FailureRecord>,
    pub timeouts: Vec<String>,
    pub slowest_tests: Vec<(String, u128)>,
    pub completed_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgressSnapshot {
    pub snapshot_version: u32,
    pub matrix_strategy_version: u32,
    pub execution_backend: ExecutionBackend,
    pub pinned_revisions: PinnedRevisions,
    pub manifest_hash: u64,
    pub run_kind: String,
    pub total: usize,
    pub passed: usize,
    pub counts_per_kind: BTreeMap<FailureKind, usize>,
    pub slowest_tests: Vec<(String, u128)>,
    pub timeout_list: Vec<String>,
    pub failures: Vec<FailureRecord>,
    pub completed_paths: Vec<String>,
    pub matrix_path: Vec<String>,
    pub completed_nodes: Vec<String>,
    pub aggregate_counts_so_far: BTreeMap<FailureKind, usize>,
    pub aggregate_entries: Vec<MatrixEntrySummary>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotPaths {
    pub json_path: PathBuf,
    pub txt_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OracleComparison {
    pub rust_count: usize,
    pub js_count: Option<usize>,
    pub matches: Option<bool>,
    pub unavailable_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaselineBucket {
    pub kind: FailureKind,
    pub total: usize,
    pub top_subtrees: Vec<(String, usize)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaselineReport {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub buckets: Vec<BaselineBucket>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopLevelRunSummary {
    pub node_id: String,
    pub node_kind: MatrixNodeKind,
    pub filter: String,
    pub matrix_path: Vec<String>,
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub counts_per_kind: BTreeMap<FailureKind, usize>,
    pub counts_per_origin: BTreeMap<FailureOrigin, usize>,
    pub manifest_hash: u64,
}

pub type MatrixEntrySummary = TopLevelRunSummary;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MatrixNodeKind {
    FilterLeaf,
    ChunkLeaf,
}

impl MatrixNodeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            MatrixNodeKind::FilterLeaf => "filter-leaf",
            MatrixNodeKind::ChunkLeaf => "chunk-leaf",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AggregateRunSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub counts_per_kind: BTreeMap<FailureKind, usize>,
    pub counts_per_origin: BTreeMap<FailureOrigin, usize>,
    pub entries: Vec<TopLevelRunSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunMatrixNode {
    pub node_id: String,
    pub node_kind: MatrixNodeKind,
    pub filter: String,
    pub matrix_path: Vec<String>,
    pub total_cases: usize,
    pub case_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatrixRunSummary {
    pub nodes: Vec<RunMatrixNode>,
    pub aggregate: AggregateRunSummary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SnapshotFile {
    snapshot_version: u32,
    matrix_strategy_version: u32,
    execution_backend: String,
    pinned_revisions: SnapshotPinnedRevisions,
    manifest_hash: u64,
    run_kind: String,
    total: usize,
    passed: usize,
    counts_per_kind: BTreeMap<String, usize>,
    slowest_tests: Vec<SnapshotSlowTest>,
    timeout_list: Vec<String>,
    failures: Vec<SnapshotFailureRecord>,
    completed_paths: Vec<String>,
    matrix_path: Vec<String>,
    completed_nodes: Vec<String>,
    aggregate_counts_so_far: BTreeMap<String, usize>,
    aggregate_entries: Vec<SnapshotAggregateEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SnapshotPinnedRevisions {
    ecma262: String,
    test262: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SnapshotSlowTest {
    path: String,
    duration_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SnapshotFailureRecord {
    test_path: String,
    kind: String,
    origin: String,
    detail: String,
    detail_hash: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SnapshotAggregateEntry {
    node_id: String,
    node_kind: String,
    filter: String,
    matrix_path: Vec<String>,
    total: usize,
    passed: usize,
    failed: usize,
    counts_per_kind: BTreeMap<String, usize>,
    counts_per_origin: BTreeMap<String, usize>,
    manifest_hash: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConformanceRunner {
    config: SuiteConfig,
}

impl Default for ConformanceRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl ConformanceRunner {
    pub fn new() -> Self {
        Self {
            config: SuiteConfig::default(),
        }
    }

    pub fn with_config(config: SuiteConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &SuiteConfig {
        &self.config
    }

    pub fn pinned_revisions(&self) -> PinnedRevisions {
        pinned_revisions(&self.config)
    }

    pub fn architecture_invariants(&self) -> &'static [&'static str] {
        &[
            "build wasm must compile user program semantics directly",
            "no interpreter or VM compiled to Wasm as shipped execution strategy",
            "no permanent expected-fail list at completion",
        ]
    }

    pub fn classify(
        &self,
        test_path: impl Into<String>,
        kind: FailureKind,
        detail: impl Into<String>,
    ) -> FailureRecord {
        classify_failure(test_path, kind, detail)
    }

    pub fn discover_suite(&self, filter: Option<&str>) -> Result<SuiteManifest, String> {
        discover_suite(&self.config, filter)
    }

    pub fn load_preludes(&self) -> Result<PreludeStore, String> {
        load_preludes(&self.config)
    }

    pub fn materialize_test(
        &self,
        case: &TestCase,
        preludes: &PreludeStore,
    ) -> Result<MaterializedTest, String> {
        materialize_test(case, preludes)
    }

    pub fn run_shard(&self, run_config: RunConfig) -> Result<ShardSummary, String> {
        run_shard(&self.config, run_config)
    }

    pub fn run_full(&self, run_config: RunConfig) -> Result<RunSummary, String> {
        run_full(&self.config, run_config)
    }

    pub fn run_top_level_matrix(
        &self,
        run_config: RunConfig,
    ) -> Result<AggregateRunSummary, String> {
        run_top_level_matrix(&self.config, run_config)
    }

    pub fn build_run_matrix(&self) -> Result<Vec<RunMatrixNode>, String> {
        build_run_matrix(&self.config)
    }

    pub fn write_snapshot(
        &self,
        snapshot: &ProgressSnapshot,
        snapshot_name: &str,
    ) -> Result<SnapshotPaths, String> {
        write_snapshot(&self.config, snapshot, snapshot_name)
    }

    pub fn baseline_report(&self, summary: &RunSummary) -> BaselineReport {
        baseline_report(summary)
    }

    pub fn aggregate_baseline_report(&self, summary: &AggregateRunSummary) -> AggregateRunSummary {
        aggregate_baseline_report(summary)
    }
}

pub fn pinned_revisions(config: &SuiteConfig) -> PinnedRevisions {
    let test262 =
        read_git_head(&config.suite_root).unwrap_or_else(|| "missing-vendored-suite".to_string());
    PinnedRevisions {
        ecma262: "ecma262-current-draft".to_string(),
        test262,
    }
}

pub fn discover_suite(config: &SuiteConfig, filter: Option<&str>) -> Result<SuiteManifest, String> {
    let test_root = config.suite_root.join("test");
    if !test_root.exists() {
        return Err(format!(
            "vendored Test262 suite not found at {}",
            test_root.display()
        ));
    }

    let filter = filter.map(|value| {
        value
            .trim_start_matches("test/")
            .trim_end_matches('/')
            .to_string()
    });
    let mut cases = Vec::new();
    scan_tests(&test_root, &test_root, filter.as_deref(), &mut cases)?;
    cases.sort_by(|left, right| left.path.cmp(&right.path));

    let pinned_revisions = pinned_revisions(config);
    let manifest_hash = hash_manifest(&pinned_revisions, &cases, filter.as_deref());

    Ok(SuiteManifest {
        pinned_revisions,
        manifest_hash,
        filter,
        cases,
    })
}

pub fn load_preludes(config: &SuiteConfig) -> Result<PreludeStore, String> {
    let mut store = PreludeStore::default();

    if config.local_harness_path.exists() {
        let merged = fs::read_to_string(&config.local_harness_path).map_err(|err| {
            format!(
                "failed to read local harness {}: {err}",
                config.local_harness_path.display()
            )
        })?;
        for section in merged.split("///").skip(1) {
            let mut lines = section.lines();
            let Some(name) = lines.next() else {
                continue;
            };
            let key = name.trim().to_string();
            let mut body = lines.collect::<Vec<_>>().join("\n");
            body.push('\n');
            store.insert(key, body, PreludeOrigin::LocalMerged);
        }
    }

    let vendored_harness_root = config.suite_root.join("harness");
    if vendored_harness_root.exists() {
        let mut files = Vec::new();
        scan_harness_files(&vendored_harness_root, &vendored_harness_root, &mut files)?;
        files.sort();
        for (name, path) in files {
            if store.contains(&name) {
                continue;
            }
            let contents = fs::read_to_string(&path).map_err(|err| {
                format!("failed to read vendored harness {}: {err}", path.display())
            })?;
            store.insert(
                name,
                format!("{contents}\n"),
                PreludeOrigin::VendoredHarness,
            );
        }
    }

    Ok(store)
}

pub fn materialize_test(
    case: &TestCase,
    preludes: &PreludeStore,
) -> Result<MaterializedTest, String> {
    let mut source = String::new();
    let mut used_preludes = Vec::new();

    if !case.flags.contains("raw") {
        if case.flags.contains("onlyStrict") {
            source.push_str("\"use strict\";\n");
        }

        for always in ["sta.js", "assert.js"] {
            if let Some(prelude) = preludes.get(always) {
                source.push_str(&prelude.contents);
                used_preludes.push((prelude.name.clone(), prelude.origin));
            }
        }

        if case.flags.contains("async") {
            if let Some(prelude) = preludes.get("doneprintHandle.js") {
                source.push_str(&prelude.contents);
                source.push_str("globalThis.$DONE = $DONE;\n");
                used_preludes.push((prelude.name.clone(), prelude.origin));
            }
        }

        for include in &case.includes {
            if include.is_empty() {
                continue;
            }
            if let Some(prelude) = preludes.get(include) {
                source.push_str(&prelude.contents);
                used_preludes.push((prelude.name.clone(), prelude.origin));
            }
        }
    }

    source.push_str(&case.original_source);

    Ok(MaterializedTest {
        path: case.path.clone(),
        source,
        used_preludes,
        negative: case.negative.clone(),
        is_module: case.is_module,
    })
}

pub fn run_shard(config: &SuiteConfig, run_config: RunConfig) -> Result<ShardSummary, String> {
    let manifest = discover_suite(config, run_config.filter.as_deref())?;
    let preludes = load_preludes(config)?;
    let cases = shard_cases(
        &manifest.cases,
        run_config.shard_index,
        run_config.shard_count,
    )?;
    let results = execute_cases(config, &manifest, &preludes, &cases, &run_config)?;
    let summary = summarize_results(&results);

    let snapshot = snapshot_from_summary(
        &manifest,
        format!(
            "shard-{}/{}",
            run_config.shard_index + 1,
            run_config.shard_count
        ),
        &summary,
        run_config.execution_backend,
    );
    write_snapshot(config, &snapshot, &run_config.snapshot_name)?;

    Ok(ShardSummary {
        shard_index: run_config.shard_index,
        shard_count: run_config.shard_count,
        total: summary.total,
        passed: summary.passed,
        failures: summary.failures,
    })
}

pub fn run_full(config: &SuiteConfig, run_config: RunConfig) -> Result<RunSummary, String> {
    let manifest = discover_suite(config, run_config.filter.as_deref())?;
    let preludes = load_preludes(config)?;
    let results = execute_cases(config, &manifest, &preludes, &manifest.cases, &run_config)?;
    let summary = summarize_results(&results);

    let snapshot = snapshot_from_summary(
        &manifest,
        "full".to_string(),
        &summary,
        run_config.execution_backend,
    );
    write_snapshot(config, &snapshot, &run_config.snapshot_name)?;

    Ok(summary)
}

pub fn run_top_level_matrix(
    config: &SuiteConfig,
    run_config: RunConfig,
) -> Result<AggregateRunSummary, String> {
    let nodes = build_run_matrix(config)?;
    let aggregate_manifest_hash = hash_matrix_nodes(&nodes, run_config.execution_backend);
    let aggregate_snapshot_name = format!("{}-aggregate", run_config.snapshot_name);
    let pinned_revisions = pinned_revisions(config);

    let mut entries = Vec::new();
    let mut completed_nodes = BTreeSet::new();
    if run_config.resume {
        if let Some(snapshot) = load_resume_aggregate_snapshot(
            config,
            &aggregate_snapshot_name,
            aggregate_manifest_hash,
            run_config.execution_backend,
        )? {
            if snapshot.run_kind == "aggregate-matrix" {
                completed_nodes.extend(snapshot.completed_nodes.iter().cloned());
                entries.extend(snapshot.aggregate_entries);
            }
        }
    }

    for node in &nodes {
        if completed_nodes.contains(&node.node_id) {
            continue;
        }

        entries.push(run_matrix_node(config, node, &run_config)?);
        completed_nodes.insert(node.node_id.clone());

        let aggregate = aggregate_from_entries(&entries);
        let aggregate_snapshot = aggregate_snapshot(
            &pinned_revisions,
            aggregate_manifest_hash,
            &aggregate,
            run_config.execution_backend,
            "aggregate-matrix",
            vec!["top-level".to_string()],
            completed_nodes.iter().cloned().collect(),
        );
        write_snapshot(config, &aggregate_snapshot, &aggregate_snapshot_name)?;
    }

    Ok(aggregate_from_entries(&entries))
}

pub fn classify_failure(
    test_path: impl Into<String>,
    kind: FailureKind,
    detail: impl Into<String>,
) -> FailureRecord {
    let detail = detail.into();
    let origin = classify_failure_origin(&detail);
    let detail = format!("[origin:{}] {detail}", origin.as_str());
    FailureRecord {
        test_path: test_path.into(),
        kind,
        origin,
        detail_hash: hash_detail(&detail),
        detail,
    }
}

pub fn write_snapshot(
    config: &SuiteConfig,
    snapshot: &ProgressSnapshot,
    snapshot_name: &str,
) -> Result<SnapshotPaths, String> {
    fs::create_dir_all(&config.snapshot_dir).map_err(|err| {
        format!(
            "failed to create snapshot dir {}: {err}",
            config.snapshot_dir.display()
        )
    })?;

    let json_path = config
        .snapshot_dir
        .join(format!("{snapshot_name}-{}.json", snapshot.manifest_hash));
    let txt_path = config
        .snapshot_dir
        .join(format!("{snapshot_name}-{}.txt", snapshot.manifest_hash));

    fs::write(&json_path, render_snapshot_json(snapshot))
        .map_err(|err| format!("failed to write snapshot {}: {err}", json_path.display()))?;
    fs::write(&txt_path, render_human_summary(snapshot))
        .map_err(|err| format!("failed to write summary {}: {err}", txt_path.display()))?;

    Ok(SnapshotPaths {
        json_path,
        txt_path,
    })
}

pub fn compare_with_js_oracle(
    config: &SuiteConfig,
    filter: Option<&str>,
) -> Result<OracleComparison, String> {
    let rust = discover_suite(config, filter)?;
    let script = format!(
        r#"
import fs from 'node:fs';
import path from 'node:path';
import readTest262 from './test262/read.js';
const repoRoot = process.cwd();
const test262Path = path.join(repoRoot, 'test262', 'vendor', 'test262');
const localHarnessPath = path.join(repoRoot, 'test262', 'harness.js');
const merged = fs.readFileSync(localHarnessPath, 'utf8');
const preludes = merged.split('///').slice(1).reduce((acc, section) => {{
  const [name, ...content] = section.split('\n');
  acc[name.trim()] = content.join('\n').trim() + '\n';
  return acc;
}}, {{}});
const harnessPath = path.join(test262Path, 'harness');
const walk = dir => {{
  for (const entry of fs.readdirSync(dir, {{ withFileTypes: true }})) {{
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {{
      walk(full);
      continue;
    }}
    if (!entry.isFile() || !entry.name.endsWith('.js')) continue;
    const rel = full.slice(harnessPath.length + 1).replaceAll('\\', '/');
    if (preludes[rel] === undefined) preludes[rel] = fs.readFileSync(full, 'utf8') + '\n';
  }}
}};
walk(harnessPath);
const tests = await readTest262(test262Path, {filter_literal}, preludes, []);
console.log(String(tests.length));
"#,
        filter_literal = js_string_literal(filter.unwrap_or(""))
    );

    let output = std::process::Command::new("node")
        .arg("--input-type=module")
        .arg("-e")
        .arg(script)
        .current_dir(repo_root_from_suite(&config.suite_root))
        .output()
        .map_err(|err| format!("failed to invoke node for JS oracle comparison: {err}"))?;

    if !output.status.success() {
        return Err(format!(
            "js oracle comparison failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let js_count = stdout.trim().parse::<usize>().map_err(|err| {
        format!(
            "failed to parse JS oracle output {:?}: {err}",
            stdout.trim()
        )
    })?;

    Ok(OracleComparison {
        rust_count: rust.cases.len(),
        js_count: Some(js_count),
        matches: Some(rust.cases.len() == js_count),
        unavailable_reason: None,
    })
}

pub fn try_compare_with_js_oracle(
    config: &SuiteConfig,
    filter: Option<&str>,
) -> Result<OracleComparison, String> {
    match compare_with_js_oracle(config, filter) {
        Ok(comparison) => Ok(comparison),
        Err(err) if is_missing_node_error(&err) => Ok(OracleComparison {
            rust_count: discover_suite(config, filter)?.cases.len(),
            js_count: None,
            matches: None,
            unavailable_reason: Some(err),
        }),
        Err(err) => Err(err),
    }
}

pub fn build_run_matrix(config: &SuiteConfig) -> Result<Vec<RunMatrixNode>, String> {
    let mut nodes = Vec::new();
    for root in TOP_LEVEL_FILTERS {
        let manifest = discover_suite(config, Some(root))?;
        nodes.extend(build_matrix_nodes_for_root(
            root,
            &manifest.cases,
            MATRIX_RECURSION_THRESHOLD,
            MATRIX_CHUNK_SIZE,
        ));
    }
    nodes.sort_by(|left, right| left.node_id.cmp(&right.node_id));
    Ok(nodes)
}

fn group_cases_by_segment(
    cases: &[TestCase],
    segment_index: usize,
) -> BTreeMap<String, Vec<TestCase>> {
    let mut groups = BTreeMap::<String, Vec<TestCase>>::new();
    for case in cases {
        let segments = case.path.split('/').collect::<Vec<_>>();
        let Some(segment) = segments.get(segment_index) else {
            continue;
        };
        groups
            .entry((*segment).to_string())
            .or_default()
            .push(case.clone());
    }
    groups
}

fn group_cases_by_directory_segment(
    cases: &[TestCase],
    segment_index: usize,
) -> BTreeMap<String, Vec<TestCase>> {
    let mut groups = BTreeMap::<String, Vec<TestCase>>::new();
    for case in cases {
        let segments = case.path.split('/').collect::<Vec<_>>();
        if segments.len() <= segment_index + 1 {
            continue;
        }
        let Some(segment) = segments.get(segment_index) else {
            continue;
        };
        groups
            .entry((*segment).to_string())
            .or_default()
            .push(case.clone());
    }
    groups
}

fn build_matrix_nodes_for_root(
    root: &str,
    cases: &[TestCase],
    recursion_threshold: usize,
    chunk_size: usize,
) -> Vec<RunMatrixNode> {
    let filter = root.to_string();
    let matrix_path = vec![filter.clone()];
    if !MATRIX_SPLIT_FILTERS.contains(&root) || cases.is_empty() {
        let mut ordered = cases.to_vec();
        ordered.sort_by(|left, right| left.path.cmp(&right.path));
        return vec![RunMatrixNode {
            node_id: filter.clone(),
            node_kind: MatrixNodeKind::FilterLeaf,
            filter,
            matrix_path,
            total_cases: ordered.len(),
            case_paths: ordered.into_iter().map(|case| case.path).collect(),
        }];
    }

    let child_groups = group_cases_by_segment(cases, 1);
    if child_groups.is_empty() {
        return finalize_matrix_nodes(
            filter,
            matrix_path,
            cases.to_vec(),
            recursion_threshold,
            chunk_size,
        );
    }

    let mut nodes = Vec::new();
    for (child, child_cases) in child_groups {
        let child_filter = format!("{root}/{child}");
        let child_path = vec![root.to_string(), child.clone()];
        if child_cases.len() > recursion_threshold {
            let grandchild_groups = group_cases_by_directory_segment(&child_cases, 2);
            if !grandchild_groups.is_empty() {
                let grandchild_case_paths = grandchild_groups
                    .values()
                    .flat_map(|group| group.iter().map(|case| case.path.clone()))
                    .collect::<BTreeSet<_>>();
                let residual_cases = child_cases
                    .iter()
                    .filter(|case| !grandchild_case_paths.contains(&case.path))
                    .cloned()
                    .collect::<Vec<_>>();
                if !residual_cases.is_empty() {
                    nodes.extend(finalize_matrix_nodes(
                        child_filter.clone(),
                        child_path.clone(),
                        residual_cases,
                        recursion_threshold,
                        chunk_size,
                    ));
                }
                for (grandchild, grandchild_cases) in grandchild_groups {
                    nodes.extend(finalize_matrix_nodes(
                        format!("{child_filter}/{grandchild}"),
                        vec![root.to_string(), child.clone(), grandchild],
                        grandchild_cases,
                        recursion_threshold,
                        chunk_size,
                    ));
                }
                continue;
            }
        }
        nodes.extend(finalize_matrix_nodes(
            child_filter,
            child_path,
            child_cases,
            recursion_threshold,
            chunk_size,
        ));
    }
    nodes
}

fn finalize_matrix_nodes(
    filter: String,
    matrix_path: Vec<String>,
    mut cases: Vec<TestCase>,
    recursion_threshold: usize,
    chunk_size: usize,
) -> Vec<RunMatrixNode> {
    cases.sort_by(|left, right| left.path.cmp(&right.path));
    if cases.len() > recursion_threshold {
        let total_chunks = cases.len().div_ceil(chunk_size);
        return cases
            .chunks(chunk_size)
            .enumerate()
            .map(|(index, chunk)| {
                let node_id = format!(
                    "{filter}@chunk-{chunk_index:04}-of-{total_chunks:04}",
                    chunk_index = index + 1
                );
                RunMatrixNode {
                    node_id,
                    node_kind: MatrixNodeKind::ChunkLeaf,
                    filter: filter.clone(),
                    matrix_path: matrix_path.clone(),
                    total_cases: chunk.len(),
                    case_paths: chunk.iter().map(|case| case.path.clone()).collect(),
                }
            })
            .collect();
    }

    vec![RunMatrixNode {
        node_id: filter.clone(),
        node_kind: MatrixNodeKind::FilterLeaf,
        filter,
        matrix_path,
        total_cases: cases.len(),
        case_paths: cases.into_iter().map(|case| case.path).collect(),
    }]
}

fn aggregate_from_entries(entries: &[TopLevelRunSummary]) -> AggregateRunSummary {
    let mut counts_per_kind = BTreeMap::new();
    let mut counts_per_origin = BTreeMap::new();
    for kind in FailureKind::ALL {
        counts_per_kind.insert(kind, 0);
    }
    for origin in FailureOrigin::ALL {
        counts_per_origin.insert(origin, 0);
    }

    let mut total = 0;
    let mut passed = 0;
    let mut ordered_entries = entries.to_vec();
    ordered_entries.sort_by(|left, right| left.node_id.cmp(&right.node_id));
    for entry in &ordered_entries {
        total += entry.total;
        passed += entry.passed;
        for kind in FailureKind::ALL {
            *counts_per_kind.entry(kind).or_insert(0) +=
                entry.counts_per_kind.get(&kind).copied().unwrap_or(0);
        }
        for origin in FailureOrigin::ALL {
            *counts_per_origin.entry(origin).or_insert(0) +=
                entry.counts_per_origin.get(&origin).copied().unwrap_or(0);
        }
    }

    AggregateRunSummary {
        total,
        passed,
        failed: total.saturating_sub(passed),
        counts_per_kind,
        counts_per_origin,
        entries: ordered_entries,
    }
}

fn hash_matrix_nodes(nodes: &[RunMatrixNode], execution_backend: ExecutionBackend) -> u64 {
    let mut hasher = DefaultHasher::new();
    MATRIX_STRATEGY_VERSION.hash(&mut hasher);
    execution_backend.as_str().hash(&mut hasher);
    for node in nodes {
        node.node_id.hash(&mut hasher);
        node.node_kind.hash(&mut hasher);
        node.filter.hash(&mut hasher);
        node.matrix_path.hash(&mut hasher);
        node.total_cases.hash(&mut hasher);
    }
    hasher.finish()
}

fn sanitize_filter_for_snapshot(filter: &str) -> String {
    filter.replace('/', "_")
}

fn execute_cases(
    config: &SuiteConfig,
    manifest: &SuiteManifest,
    preludes: &PreludeStore,
    cases: &[TestCase],
    run_config: &RunConfig,
) -> Result<Vec<TestResult>, String> {
    let previous = if run_config.resume {
        load_previous_snapshot(config, &run_config.snapshot_name, manifest.manifest_hash)?
    } else {
        None
    };

    let mut completed = BTreeMap::new();
    if let Some(snapshot) = previous {
        for failure in snapshot.failures {
            completed.insert(
                failure.test_path.clone(),
                TestResult {
                    test_path: failure.test_path.clone(),
                    status: TestStatus::Failed(failure),
                    duration_ms: 0,
                },
            );
        }
        for path in snapshot.completed_paths {
            completed.entry(path.clone()).or_insert(TestResult {
                test_path: path,
                status: TestStatus::Passed,
                duration_ms: 0,
            });
        }
    }

    let remaining: Vec<TestCase> = cases
        .iter()
        .filter(|case| !completed.contains_key(&case.path))
        .cloned()
        .collect();

    if remaining.is_empty() {
        let mut existing = completed.into_values().collect::<Vec<_>>();
        existing.sort_by(|left, right| left.test_path.cmp(&right.test_path));
        return Ok(existing);
    }

    let queue = Arc::new(Mutex::new(remaining));
    let results = Arc::new(Mutex::new(Vec::new()));
    let worker_count = config.worker_count.max(1).min(cases.len().max(1));

    thread::scope(|scope| {
        for _ in 0..worker_count {
            let queue = Arc::clone(&queue);
            let results = Arc::clone(&results);
            let preludes = preludes.clone();
            let timeout = config.timeout_ms;
            let execution_backend = run_config.execution_backend;
            thread::Builder::new()
                .stack_size(32 * 1024 * 1024)
                .spawn_scoped(scope, move || loop {
                    let maybe_case = {
                        let mut guard = queue.lock().expect("queue mutex poisoned");
                        guard.pop()
                    };
                    let Some(case) = maybe_case else {
                        break;
                    };
                    let result = panic::catch_unwind(AssertUnwindSafe(|| {
                        run_one_case(&case, &preludes, timeout, execution_backend)
                    }))
                    .unwrap_or_else(|panic_payload| TestResult {
                        test_path: case.path.clone(),
                        status: TestStatus::Failed(classify_failure(
                            &case.path,
                            FailureKind::Runtime,
                            format!("worker panic: {}", panic_message(&panic_payload)),
                        )),
                        duration_ms: timeout.into(),
                    });
                    results.lock().expect("results mutex poisoned").push(result);
                })
                .expect("worker thread should spawn");
        }
    });

    let mut all_results = completed.into_values().collect::<Vec<_>>();
    all_results.extend(results.lock().expect("results mutex poisoned").clone());
    all_results.sort_by(|left, right| left.test_path.cmp(&right.test_path));
    Ok(all_results)
}

fn run_one_case(
    case: &TestCase,
    preludes: &PreludeStore,
    timeout_ms: u64,
    execution_backend: ExecutionBackend,
) -> TestResult {
    let start = Instant::now();
    let outcome = (|| -> Result<(), FailureRecord> {
        let materialized = materialize_test(case, preludes)
            .map_err(|detail| classify_failure(&case.path, FailureKind::HostHarness, detail))?;

        let engine = Engine::new(RealmBuilder::new().build());
        let compile_options = CompileOptions {
            filename: Some(case.source_path.display().to_string()),
            ..CompileOptions::default()
        };

        let compile_result = if materialized.is_module {
            engine.compile_module(&materialized.source, compile_options.clone())
        } else {
            engine.compile_script(&materialized.source, compile_options.clone())
        };

        if let Some(negative) = &case.negative {
            let negative_kind = classify_negative_phase(&negative.phase);
            match compile_result.as_ref() {
                Err(err)
                    if negative.phase.eq_ignore_ascii_case("parse")
                        || negative.phase.eq_ignore_ascii_case("early") =>
                {
                    let detail = err.message().to_string();
                    if negative.error_type.is_empty()
                        || detail.contains(&negative.error_type)
                        || negative.phase.eq_ignore_ascii_case("parse")
                    {
                        return Ok(());
                    }
                    return Err(classify_failure(
                        &case.path,
                        negative_kind,
                        format!(
                            "negative test error mismatch: expected {}, got {}",
                            negative.error_type, detail
                        ),
                    ));
                }
                Err(err) => {
                    return Err(classify_failure(
                        &case.path,
                        classify_engine_error(err.message()),
                        err.to_string(),
                    ));
                }
                Ok(_)
                    if (negative.phase.eq_ignore_ascii_case("parse")
                        || negative.phase.eq_ignore_ascii_case("early"))
                        && execution_backend != ExecutionBackend::SpecExec =>
                {
                    return Err(classify_failure(
                        &case.path,
                        negative_kind,
                        format!(
                            "negative test expected {} error but compile succeeded",
                            negative.phase
                        ),
                    ));
                }
                Ok(_) => {}
            }
        } else if let Err(err) = compile_result.as_ref() {
            if execution_backend != ExecutionBackend::SpecExec {
                let kind = classify_engine_error(err.message());
                return Err(classify_failure(&case.path, kind, err.to_string()));
            }
        }

        let run_result = if materialized.is_module {
            engine.run_module(
                &materialized.source,
                compile_options,
                RunOptions {
                    backend: execution_backend,
                    argv: Vec::new(),
                    module_root: case
                        .source_path
                        .parent()
                        .map(|path| path.display().to_string()),
                    test_path: Some(case.source_path.display().to_string()),
                    can_block: case.flags.contains("CanBlockIsTrue"),
                },
            )
        } else {
            engine.run_script(
                &materialized.source,
                compile_options,
                RunOptions {
                    backend: execution_backend,
                    argv: Vec::new(),
                    module_root: None,
                    test_path: Some(case.source_path.display().to_string()),
                    can_block: case.flags.contains("CanBlockIsTrue"),
                },
            )
        };

        if let Some(negative) = &case.negative {
            let negative_kind = classify_negative_phase(&negative.phase);
            return match run_result {
                Ok(_) => Err(classify_failure(
                    &case.path,
                    negative_kind,
                    format!(
                        "negative test expected {} error but execution succeeded",
                        negative.phase
                    ),
                )),
                Err(err) => {
                    let detail = err.message().to_string();
                    if negative.error_type.is_empty() || detail.contains(&negative.error_type) {
                        Ok(())
                    } else {
                        Err(classify_failure(
                            &case.path,
                            negative_kind,
                            format!(
                                "negative test error mismatch: expected {}, got {}",
                                negative.error_type, detail
                            ),
                        ))
                    }
                }
            };
        }

        match run_result {
            Ok(_) => Ok(()),
            Err(err) => Err(classify_failure(
                &case.path,
                classify_engine_error(err.message()),
                err.to_string(),
            )),
        }
    })();

    let duration_ms = start.elapsed().as_millis();
    if duration_ms > u128::from(timeout_ms) {
        return TestResult {
            test_path: case.path.clone(),
            status: TestStatus::Failed(classify_failure(
                &case.path,
                FailureKind::Runtime,
                format!("timeout exceeded after {}ms", duration_ms),
            )),
            duration_ms,
        };
    }

    match outcome {
        Ok(()) => TestResult {
            test_path: case.path.clone(),
            status: TestStatus::Passed,
            duration_ms,
        },
        Err(failure) => TestResult {
            test_path: case.path.clone(),
            status: TestStatus::Failed(failure),
            duration_ms,
        },
    }
}

fn classify_negative_phase(phase: &str) -> FailureKind {
    if phase.eq_ignore_ascii_case("parse") {
        FailureKind::Parser
    } else if phase.eq_ignore_ascii_case("early") {
        FailureKind::EarlyError
    } else {
        FailureKind::Runtime
    }
}

fn summarize_results(results: &[TestResult]) -> RunSummary {
    let mut counts = BTreeMap::new();
    for kind in FailureKind::ALL {
        counts.insert(kind, 0);
    }

    let mut failures = Vec::new();
    let mut completed_paths = Vec::new();
    let mut slowest = results
        .iter()
        .map(|result| (result.test_path.clone(), result.duration_ms))
        .collect::<Vec<_>>();
    slowest.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    slowest.truncate(10);

    let mut timeouts = Vec::new();
    let mut passed = 0;
    for result in results {
        completed_paths.push(result.test_path.clone());
        match &result.status {
            TestStatus::Passed => {
                passed += 1;
            }
            TestStatus::Failed(failure) => {
                *counts.entry(failure.kind).or_insert(0) += 1;
                if failure.detail.contains("timeout exceeded") {
                    timeouts.push(failure.test_path.clone());
                }
                failures.push(failure.clone());
            }
        }
    }

    RunSummary {
        total: results.len(),
        passed,
        counts_per_kind: counts,
        failures,
        timeouts,
        slowest_tests: slowest,
        completed_paths,
    }
}

fn run_matrix_node(
    config: &SuiteConfig,
    node: &RunMatrixNode,
    run_config: &RunConfig,
) -> Result<TopLevelRunSummary, String> {
    let manifest = discover_suite(config, Some(&node.filter))?;
    let preludes = load_preludes(config)?;
    let case_lookup = manifest
        .cases
        .iter()
        .cloned()
        .map(|case| (case.path.clone(), case))
        .collect::<BTreeMap<_, _>>();
    let mut cases = Vec::with_capacity(node.case_paths.len());
    for path in &node.case_paths {
        let Some(case) = case_lookup.get(path) else {
            return Err(format!(
                "matrix node {} references missing case {}",
                node.node_id, path
            ));
        };
        cases.push(case.clone());
    }

    let node_manifest = SuiteManifest {
        pinned_revisions: manifest.pinned_revisions.clone(),
        manifest_hash: hash_manifest(
            &manifest.pinned_revisions,
            &cases,
            Some(node.node_id.as_str()),
        ),
        filter: Some(node.filter.clone()),
        cases: cases.clone(),
    };
    let summary = summarize_results(&execute_cases(
        config,
        &node_manifest,
        &preludes,
        &cases,
        &RunConfig {
            filter: Some(node.filter.clone()),
            shard_index: 0,
            shard_count: 1,
            resume: run_config.resume,
            snapshot_name: format!(
                "{}-{}",
                run_config.snapshot_name,
                sanitize_filter_for_snapshot(&node.node_id)
            ),
            execution_backend: run_config.execution_backend,
        },
    )?);

    let mut snapshot = snapshot_from_summary(
        &node_manifest,
        format!("matrix-{}", node.node_kind.as_str()),
        &summary,
        run_config.execution_backend,
    );
    snapshot.matrix_path = node.matrix_path.clone();
    write_snapshot(
        config,
        &snapshot,
        &format!(
            "{}-{}",
            run_config.snapshot_name,
            sanitize_filter_for_snapshot(&node.node_id)
        ),
    )?;

    Ok(TopLevelRunSummary {
        node_id: node.node_id.clone(),
        node_kind: node.node_kind,
        filter: node.filter.clone(),
        matrix_path: node.matrix_path.clone(),
        total: summary.total,
        passed: summary.passed,
        failed: summary.total.saturating_sub(summary.passed),
        counts_per_kind: summary.counts_per_kind.clone(),
        counts_per_origin: counts_per_origin(&summary.failures),
        manifest_hash: node_manifest.manifest_hash,
    })
}

fn counts_per_origin(failures: &[FailureRecord]) -> BTreeMap<FailureOrigin, usize> {
    let mut counts = BTreeMap::new();
    for origin in FailureOrigin::ALL {
        counts.insert(origin, 0);
    }
    for failure in failures {
        *counts.entry(failure.origin).or_insert(0) += 1;
    }
    counts
}

fn panic_message(payload: &Box<dyn core::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&'static str>() {
        (*message).to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "non-string panic payload".to_string()
    }
}

fn snapshot_from_summary(
    manifest: &SuiteManifest,
    run_kind: String,
    summary: &RunSummary,
    execution_backend: ExecutionBackend,
) -> ProgressSnapshot {
    ProgressSnapshot {
        snapshot_version: SNAPSHOT_VERSION,
        matrix_strategy_version: MATRIX_STRATEGY_VERSION,
        execution_backend,
        pinned_revisions: manifest.pinned_revisions.clone(),
        manifest_hash: manifest.manifest_hash,
        run_kind,
        total: summary.total,
        passed: summary.passed,
        counts_per_kind: summary.counts_per_kind.clone(),
        slowest_tests: summary.slowest_tests.clone(),
        timeout_list: summary.timeouts.clone(),
        failures: summary.failures.clone(),
        completed_paths: summary.completed_paths.clone(),
        matrix_path: Vec::new(),
        completed_nodes: Vec::new(),
        aggregate_counts_so_far: BTreeMap::new(),
        aggregate_entries: Vec::new(),
    }
}

fn aggregate_snapshot(
    pinned_revisions: &PinnedRevisions,
    manifest_hash: u64,
    summary: &AggregateRunSummary,
    execution_backend: ExecutionBackend,
    run_kind: &str,
    matrix_path: Vec<String>,
    completed_nodes: Vec<String>,
) -> ProgressSnapshot {
    ProgressSnapshot {
        snapshot_version: SNAPSHOT_VERSION,
        matrix_strategy_version: MATRIX_STRATEGY_VERSION,
        execution_backend,
        pinned_revisions: pinned_revisions.clone(),
        manifest_hash,
        run_kind: run_kind.to_string(),
        total: summary.total,
        passed: summary.passed,
        counts_per_kind: summary.counts_per_kind.clone(),
        slowest_tests: Vec::new(),
        timeout_list: Vec::new(),
        failures: Vec::new(),
        completed_paths: Vec::new(),
        matrix_path,
        completed_nodes,
        aggregate_counts_so_far: summary.counts_per_kind.clone(),
        aggregate_entries: summary.entries.clone(),
    }
}

fn classify_engine_error(message: &str) -> FailureKind {
    let lower = message.to_ascii_lowercase();
    if lower.contains("runtime execution for wasm is not implemented yet")
        || lower.contains("not supported in porffor-spec-exec")
        || lower.contains("not supported in porffor")
        || lower.contains("detacharraybuffer")
    {
        FailureKind::Unsupported
    } else if lower.contains("nul byte")
        || lower.contains("parse")
        || lower.contains("syntaxerror")
        || lower.contains("syntax error")
    {
        FailureKind::Parser
    } else if lower.contains("early error") {
        FailureKind::EarlyError
    } else if lower.contains("wasm") && lower.contains("not implemented") {
        FailureKind::WasmBackend
    } else if lower.contains("interpreter-in-wasm")
        || lower.contains("not implemented yet")
        || lower.contains("stub")
    {
        FailureKind::Unsupported
    } else {
        FailureKind::Runtime
    }
}

fn classify_failure_origin(detail: &str) -> FailureOrigin {
    let lower = detail.to_ascii_lowercase();
    if lower.contains("failed to read local harness")
        || lower.contains("local harness")
        || lower.contains("worker panic:")
    {
        FailureOrigin::LocalHarness
    } else if lower.contains("icu_")
        || lower.contains("hijri")
        || lower.contains("intl")
        || lower.contains("datetimeformat")
        || lower.contains("numberformat")
        || lower.contains("durationformat")
        || lower.contains("relativetimeformat")
    {
        FailureOrigin::IcuIntl
    } else if lower.contains("agent threads are not supported in porffor-spec-exec")
        || lower.contains("__porfdetacharraybuffer")
        || lower.contains("spec-exec")
        || lower.contains("detacharraybuffer")
    {
        FailureOrigin::SpecExecHost
    } else if lower.contains("syntaxerror")
        || lower.contains("syntax error")
        || lower.contains("parse")
    {
        FailureOrigin::BoaParser
    } else if lower.contains("referenceerror")
        || lower.contains("typeerror")
        || lower.contains("rangeerror")
        || lower.contains("urierror")
        || lower.contains("runtime")
        || lower.contains("index out of bounds")
        || lower.contains("must be declarative environment")
    {
        FailureOrigin::BoaRuntime
    } else {
        FailureOrigin::Unknown
    }
}

fn snapshot_to_file(snapshot: &ProgressSnapshot) -> SnapshotFile {
    SnapshotFile {
        snapshot_version: snapshot.snapshot_version,
        matrix_strategy_version: snapshot.matrix_strategy_version,
        execution_backend: snapshot.execution_backend.as_str().to_string(),
        pinned_revisions: SnapshotPinnedRevisions {
            ecma262: snapshot.pinned_revisions.ecma262.clone(),
            test262: snapshot.pinned_revisions.test262.clone(),
        },
        manifest_hash: snapshot.manifest_hash,
        run_kind: snapshot.run_kind.clone(),
        total: snapshot.total,
        passed: snapshot.passed,
        counts_per_kind: encode_kind_counts(&snapshot.counts_per_kind),
        slowest_tests: snapshot
            .slowest_tests
            .iter()
            .map(|(path, duration_ms)| SnapshotSlowTest {
                path: path.clone(),
                duration_ms: *duration_ms,
            })
            .collect(),
        timeout_list: snapshot.timeout_list.clone(),
        failures: snapshot
            .failures
            .iter()
            .map(|failure| SnapshotFailureRecord {
                test_path: failure.test_path.clone(),
                kind: failure.kind.as_str().to_string(),
                origin: failure.origin.as_str().to_string(),
                detail: failure.detail.clone(),
                detail_hash: failure.detail_hash,
            })
            .collect(),
        completed_paths: snapshot.completed_paths.clone(),
        matrix_path: snapshot.matrix_path.clone(),
        completed_nodes: snapshot.completed_nodes.clone(),
        aggregate_counts_so_far: encode_kind_counts(&snapshot.aggregate_counts_so_far),
        aggregate_entries: snapshot
            .aggregate_entries
            .iter()
            .map(|entry| SnapshotAggregateEntry {
                node_id: entry.node_id.clone(),
                node_kind: entry.node_kind.as_str().to_string(),
                filter: entry.filter.clone(),
                matrix_path: entry.matrix_path.clone(),
                total: entry.total,
                passed: entry.passed,
                failed: entry.failed,
                counts_per_kind: encode_kind_counts(&entry.counts_per_kind),
                counts_per_origin: encode_origin_counts(&entry.counts_per_origin),
                manifest_hash: entry.manifest_hash,
            })
            .collect(),
    }
}

fn snapshot_from_file(file: SnapshotFile) -> Option<ProgressSnapshot> {
    if file.snapshot_version != SNAPSHOT_VERSION {
        return None;
    }
    Some(ProgressSnapshot {
        snapshot_version: file.snapshot_version,
        matrix_strategy_version: file.matrix_strategy_version,
        execution_backend: match file.execution_backend.as_str() {
            "wasm-aot" => ExecutionBackend::WasmAot,
            _ => ExecutionBackend::SpecExec,
        },
        pinned_revisions: PinnedRevisions {
            ecma262: file.pinned_revisions.ecma262,
            test262: file.pinned_revisions.test262,
        },
        manifest_hash: file.manifest_hash,
        run_kind: file.run_kind,
        total: file.total,
        passed: file.passed,
        counts_per_kind: decode_kind_counts(&file.counts_per_kind),
        slowest_tests: file
            .slowest_tests
            .into_iter()
            .map(|entry| (entry.path, entry.duration_ms))
            .collect(),
        timeout_list: file.timeout_list,
        failures: file
            .failures
            .into_iter()
            .map(|failure| FailureRecord {
                test_path: failure.test_path,
                kind: decode_kind(&failure.kind),
                origin: decode_origin(&failure.origin),
                detail: failure.detail,
                detail_hash: failure.detail_hash,
            })
            .collect(),
        completed_paths: file.completed_paths,
        matrix_path: file.matrix_path,
        completed_nodes: file.completed_nodes,
        aggregate_counts_so_far: decode_kind_counts(&file.aggregate_counts_so_far),
        aggregate_entries: file
            .aggregate_entries
            .into_iter()
            .map(|entry| TopLevelRunSummary {
                node_id: entry.node_id,
                node_kind: match entry.node_kind.as_str() {
                    "chunk-leaf" => MatrixNodeKind::ChunkLeaf,
                    _ => MatrixNodeKind::FilterLeaf,
                },
                filter: entry.filter,
                matrix_path: entry.matrix_path,
                total: entry.total,
                passed: entry.passed,
                failed: entry.failed,
                counts_per_kind: decode_kind_counts(&entry.counts_per_kind),
                counts_per_origin: decode_origin_counts(&entry.counts_per_origin),
                manifest_hash: entry.manifest_hash,
            })
            .collect(),
    })
}

fn encode_kind_counts(counts: &BTreeMap<FailureKind, usize>) -> BTreeMap<String, usize> {
    let mut out = BTreeMap::new();
    for kind in FailureKind::ALL {
        out.insert(
            kind.as_str().to_string(),
            counts.get(&kind).copied().unwrap_or(0),
        );
    }
    out
}

fn decode_kind_counts(counts: &BTreeMap<String, usize>) -> BTreeMap<FailureKind, usize> {
    let mut out = BTreeMap::new();
    for kind in FailureKind::ALL {
        out.insert(kind, counts.get(kind.as_str()).copied().unwrap_or(0));
    }
    out
}

fn encode_origin_counts(counts: &BTreeMap<FailureOrigin, usize>) -> BTreeMap<String, usize> {
    let mut out = BTreeMap::new();
    for origin in FailureOrigin::ALL {
        out.insert(
            origin.as_str().to_string(),
            counts.get(&origin).copied().unwrap_or(0),
        );
    }
    out
}

fn decode_origin_counts(counts: &BTreeMap<String, usize>) -> BTreeMap<FailureOrigin, usize> {
    let mut out = BTreeMap::new();
    for origin in FailureOrigin::ALL {
        out.insert(origin, counts.get(origin.as_str()).copied().unwrap_or(0));
    }
    out
}

fn decode_kind(kind: &str) -> FailureKind {
    match kind {
        "Parser" => FailureKind::Parser,
        "EarlyError" => FailureKind::EarlyError,
        "Lowering" => FailureKind::Lowering,
        "Runtime" => FailureKind::Runtime,
        "WasmBackend" => FailureKind::WasmBackend,
        "HostHarness" => FailureKind::HostHarness,
        "Unsupported" => FailureKind::Unsupported,
        _ => FailureKind::Runtime,
    }
}

fn decode_origin(origin: &str) -> FailureOrigin {
    match origin {
        "local-harness" => FailureOrigin::LocalHarness,
        "boa-runtime" => FailureOrigin::BoaRuntime,
        "boa-parser" => FailureOrigin::BoaParser,
        "icu-intl" => FailureOrigin::IcuIntl,
        "spec-exec-host" => FailureOrigin::SpecExecHost,
        _ => FailureOrigin::Unknown,
    }
}

fn render_snapshot_json(snapshot: &ProgressSnapshot) -> String {
    serde_json::to_string_pretty(&snapshot_to_file(snapshot))
        .map(|json| format!("{json}\n"))
        .unwrap_or_else(|err| {
            format!(
                "{{\"snapshot_error\":\"{}\"}}\n",
                json_escape(&err.to_string())
            )
        })
}

fn render_human_summary(snapshot: &ProgressSnapshot) -> String {
    let failed = snapshot.total.saturating_sub(snapshot.passed);
    let mut out = String::new();
    writeln!(
        &mut out,
        "test262 {} summary (snapshot v{})",
        snapshot.run_kind, snapshot.snapshot_version
    )
    .unwrap();
    writeln!(
        &mut out,
        "execution_backend={}",
        snapshot.execution_backend.as_str()
    )
    .unwrap();
    writeln!(
        &mut out,
        "matrix_strategy_version={}",
        snapshot.matrix_strategy_version
    )
    .unwrap();
    writeln!(
        &mut out,
        "pinned: ecma262={} test262={}",
        snapshot.pinned_revisions.ecma262, snapshot.pinned_revisions.test262
    )
    .unwrap();
    writeln!(
        &mut out,
        "total={} pass={} fail={}",
        snapshot.total, snapshot.passed, failed
    )
    .unwrap();
    for kind in FailureKind::ALL {
        writeln!(
            &mut out,
            "{}={}",
            kind.as_str(),
            snapshot.counts_per_kind.get(&kind).copied().unwrap_or(0)
        )
        .unwrap();
    }
    if !snapshot.timeout_list.is_empty() {
        writeln!(&mut out, "timeouts={}", snapshot.timeout_list.join(", ")).unwrap();
    }
    if !snapshot.failures.is_empty() {
        writeln!(&mut out, "top_failures:").unwrap();
        for failure in snapshot.failures.iter().take(10) {
            writeln!(
                &mut out,
                "- {} [{}] {}",
                failure.test_path,
                format!("{}/{}", failure.kind.as_str(), failure.origin.as_str()),
                failure.detail
            )
            .unwrap();
        }
    }
    out
}

fn read_snapshot_file(path: &Path) -> Result<SnapshotFile, String> {
    let raw = fs::read_to_string(path)
        .map_err(|err| format!("failed to read snapshot {}: {err}", path.display()))?;
    serde_json::from_str(&raw)
        .map_err(|err| format!("failed to parse snapshot {}: {err}", path.display()))
}

fn load_resume_aggregate_snapshot(
    config: &SuiteConfig,
    snapshot_name: &str,
    expected_manifest_hash: u64,
    expected_backend: ExecutionBackend,
) -> Result<Option<ProgressSnapshot>, String> {
    let exact_path = config
        .snapshot_dir
        .join(format!("{snapshot_name}-{expected_manifest_hash}.json"));
    if exact_path.exists() {
        let file = read_snapshot_file(&exact_path)?;
        validate_resume_aggregate_snapshot(
            &file,
            &exact_path,
            expected_manifest_hash,
            expected_backend,
        )?;
        return Ok(snapshot_from_file(file));
    }

    let prefix = format!("{snapshot_name}-");
    let mut candidates = fs::read_dir(&config.snapshot_dir)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(Result::ok))
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension().and_then(|value| value.to_str()) == Some("json")
                && path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .is_some_and(|name| name.starts_with(&prefix))
        })
        .collect::<Vec<_>>();
    candidates.sort();
    let Some(path) = candidates.pop() else {
        return Ok(None);
    };
    let file = read_snapshot_file(&path)?;
    validate_resume_aggregate_snapshot(&file, &path, expected_manifest_hash, expected_backend)?;
    Ok(snapshot_from_file(file))
}

fn validate_resume_aggregate_snapshot(
    file: &SnapshotFile,
    path: &Path,
    expected_manifest_hash: u64,
    expected_backend: ExecutionBackend,
) -> Result<(), String> {
    if file.snapshot_version != SNAPSHOT_VERSION {
        return Err(format!(
            "resume snapshot mismatch for snapshot_version in {}: expected {}, found {}",
            path.display(),
            SNAPSHOT_VERSION,
            file.snapshot_version
        ));
    }
    if file.matrix_strategy_version != MATRIX_STRATEGY_VERSION {
        return Err(format!(
            "resume snapshot mismatch for matrix_strategy_version in {}: expected {}, found {}",
            path.display(),
            MATRIX_STRATEGY_VERSION,
            file.matrix_strategy_version
        ));
    }
    if file.execution_backend != expected_backend.as_str() {
        return Err(format!(
            "resume snapshot mismatch for execution_backend in {}: expected {}, found {}",
            path.display(),
            expected_backend.as_str(),
            file.execution_backend
        ));
    }
    if file.manifest_hash != expected_manifest_hash {
        return Err(format!(
            "resume snapshot mismatch for manifest_hash in {}: expected {}, found {}",
            path.display(),
            expected_manifest_hash,
            file.manifest_hash
        ));
    }
    if file.run_kind != "aggregate-matrix" {
        return Err(format!(
            "resume snapshot mismatch for run_kind in {}: expected aggregate-matrix, found {}",
            path.display(),
            file.run_kind
        ));
    }
    Ok(())
}

fn load_previous_snapshot(
    config: &SuiteConfig,
    snapshot_name: &str,
    manifest_hash: u64,
) -> Result<Option<ProgressSnapshot>, String> {
    let path = config
        .snapshot_dir
        .join(format!("{snapshot_name}-{manifest_hash}.json"));
    if !path.exists() {
        return Ok(None);
    }
    let file = read_snapshot_file(&path)?;
    let Some(mut snapshot) = snapshot_from_file(file) else {
        return Ok(None);
    };
    snapshot.manifest_hash = manifest_hash;
    Ok(Some(snapshot))
}

pub fn baseline_report(summary: &RunSummary) -> BaselineReport {
    let mut subtree_by_kind: BTreeMap<FailureKind, BTreeMap<String, usize>> = BTreeMap::new();
    for failure in &summary.failures {
        let subtree = top_level_subtree(&failure.test_path);
        *subtree_by_kind
            .entry(failure.kind)
            .or_default()
            .entry(subtree)
            .or_insert(0) += 1;
    }

    let mut buckets = Vec::new();
    for kind in FailureKind::ALL {
        let total = summary.counts_per_kind.get(&kind).copied().unwrap_or(0);
        let mut top_subtrees = subtree_by_kind
            .remove(&kind)
            .unwrap_or_default()
            .into_iter()
            .collect::<Vec<_>>();
        top_subtrees.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
        top_subtrees.truncate(10);
        buckets.push(BaselineBucket {
            kind,
            total,
            top_subtrees,
        });
    }
    buckets.sort_by(|left, right| {
        right
            .total
            .cmp(&left.total)
            .then_with(|| left.kind.as_str().cmp(right.kind.as_str()))
    });

    BaselineReport {
        total: summary.total,
        passed: summary.passed,
        failed: summary.total.saturating_sub(summary.passed),
        buckets,
    }
}

pub fn aggregate_baseline_report(summary: &AggregateRunSummary) -> AggregateRunSummary {
    let mut grouped = BTreeMap::<String, TopLevelRunSummary>::new();
    for entry in &summary.entries {
        let aggregate = grouped.entry(entry.filter.clone()).or_insert_with(|| {
            let mut counts_per_kind = BTreeMap::new();
            for kind in FailureKind::ALL {
                counts_per_kind.insert(kind, 0);
            }
            let mut counts_per_origin = BTreeMap::new();
            for origin in FailureOrigin::ALL {
                counts_per_origin.insert(origin, 0);
            }
            TopLevelRunSummary {
                node_id: entry.filter.clone(),
                node_kind: MatrixNodeKind::FilterLeaf,
                filter: entry.filter.clone(),
                matrix_path: entry.matrix_path.clone(),
                total: 0,
                passed: 0,
                failed: 0,
                counts_per_kind,
                counts_per_origin,
                manifest_hash: entry.manifest_hash,
            }
        });
        aggregate.total += entry.total;
        aggregate.passed += entry.passed;
        aggregate.failed += entry.failed;
        for kind in FailureKind::ALL {
            *aggregate.counts_per_kind.entry(kind).or_insert(0) +=
                entry.counts_per_kind.get(&kind).copied().unwrap_or(0);
        }
        for origin in FailureOrigin::ALL {
            *aggregate.counts_per_origin.entry(origin).or_insert(0) +=
                entry.counts_per_origin.get(&origin).copied().unwrap_or(0);
        }
    }

    let mut entries = grouped.into_values().collect::<Vec<_>>();
    entries.sort_by(|left, right| left.filter.cmp(&right.filter));

    AggregateRunSummary {
        total: summary.total,
        passed: summary.passed,
        failed: summary.failed,
        counts_per_kind: summary.counts_per_kind.clone(),
        counts_per_origin: summary.counts_per_origin.clone(),
        entries,
    }
}

fn top_level_subtree(test_path: &str) -> String {
    let mut parts = test_path.split('/');
    match (parts.next(), parts.next()) {
        (Some(first), Some(second)) => format!("{first}/{second}"),
        (Some(first), None) => first.to_string(),
        _ => "unknown".to_string(),
    }
}

fn is_missing_node_error(err: &str) -> bool {
    err.contains("No such file or directory") || err.contains("node") && err.contains("os error 2")
}

fn shard_cases(
    cases: &[TestCase],
    shard_index: usize,
    shard_count: usize,
) -> Result<Vec<TestCase>, String> {
    if shard_count == 0 {
        return Err("shard count must be at least 1".to_string());
    }
    if shard_index >= shard_count {
        return Err(format!(
            "shard index {} out of range for shard count {}",
            shard_index, shard_count
        ));
    }

    Ok(cases
        .iter()
        .enumerate()
        .filter(|(index, _)| index % shard_count == shard_index)
        .map(|(_, case)| case.clone())
        .collect())
}

fn scan_tests(
    dir: &Path,
    test_root: &Path,
    filter: Option<&str>,
    cases: &mut Vec<TestCase>,
) -> Result<(), String> {
    let mut entries = fs::read_dir(dir)
        .map_err(|err| format!("failed to read test dir {}: {err}", dir.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| format!("failed to iterate test dir {}: {err}", dir.display()))?;
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        if entry
            .file_type()
            .map_err(|err| format!("failed to read file type {}: {err}", path.display()))?
            .is_dir()
        {
            scan_tests(&path, test_root, filter, cases)?;
            continue;
        }

        if path.extension().and_then(|value| value.to_str()) != Some("js") {
            continue;
        }

        let rel = path
            .strip_prefix(test_root)
            .map_err(|err| {
                format!(
                    "failed to make relative test path {}: {err}",
                    path.display()
                )
            })?
            .to_string_lossy()
            .replace('\\', "/");
        if rel.contains("_FIXTURE") {
            continue;
        }
        if let Some(filter) = filter {
            if !rel.starts_with(filter) {
                continue;
            }
        }

        let original_source = fs::read_to_string(&path)
            .map_err(|err| format!("failed to read test {}: {err}", path.display()))?;
        cases.push(parse_test_case(rel, path, original_source));
    }

    Ok(())
}

fn parse_test_case(path: String, source_path: PathBuf, original_source: String) -> TestCase {
    let frontmatter = parse_frontmatter_block(&original_source);
    let flags = parse_frontmatter_list(frontmatter.get("flags").map(String::as_str));
    let includes = parse_frontmatter_vec(frontmatter.get("includes").map(String::as_str));
    let negative = parse_negative(frontmatter.get("negative").map(String::as_str));
    let is_module = flags.iter().any(|flag| flag == "module");

    TestCase {
        path,
        source_path,
        original_source,
        flags,
        includes,
        negative,
        is_module,
    }
}

fn parse_frontmatter_block(source: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let Some(start) = source.find("/*---") else {
        return out;
    };
    let Some(end) = source[start + 5..].find("---*/") else {
        return out;
    };
    let body = &source[start + 5..start + 5 + end];
    let normalized_body = body.replace("\r\n", "\n").replace('\r', "\n");
    let mut current_key = None::<String>;
    let mut current_value = String::new();

    for line in normalized_body.lines() {
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            continue;
        }
        if let Some((key, value)) = trimmed.split_once(':') {
            if !line.starts_with(' ') && !line.starts_with('\t') {
                if let Some(key) = current_key.take() {
                    out.insert(key, current_value.trim().to_string());
                    current_value.clear();
                }
                current_key = Some(key.trim().to_string());
                current_value.push_str(value.trim());
                current_value.push('\n');
                continue;
            }
        }

        current_value.push_str(trimmed.trim());
        current_value.push('\n');
    }

    if let Some(key) = current_key {
        out.insert(key, current_value.trim().to_string());
    }

    out
}

fn parse_frontmatter_list(value: Option<&str>) -> BTreeSet<String> {
    let Some(value) = value else {
        return BTreeSet::new();
    };
    let value = value.trim();
    if value.starts_with('[') && value.ends_with(']') {
        return value[1..value.len() - 1]
            .split(',')
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(str::to_string)
            .collect();
    }

    let mut out = BTreeSet::new();
    for line in value.lines() {
        let item = line.trim().trim_start_matches('-').trim();
        if !item.is_empty() {
            out.insert(item.to_string());
        }
    }
    out
}

fn parse_frontmatter_vec(value: Option<&str>) -> Vec<String> {
    let Some(value) = value else {
        return Vec::new();
    };
    let value = value.trim();
    if value.starts_with('[') && value.ends_with(']') {
        return value[1..value.len() - 1]
            .split(',')
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(str::to_string)
            .collect();
    }

    value
        .lines()
        .map(|line| line.trim().trim_start_matches('-').trim())
        .filter(|item| !item.is_empty())
        .map(str::to_string)
        .collect()
}

fn parse_negative(value: Option<&str>) -> Option<NegativeExpectation> {
    let value = value?;
    let mut phase = String::new();
    let mut error_type = String::new();
    for line in value.lines() {
        if let Some(rest) = line.trim().strip_prefix("phase:") {
            phase = rest.trim().to_string();
        } else if let Some(rest) = line.trim().strip_prefix("type:") {
            error_type = rest.trim().to_string();
        }
    }

    if phase.is_empty() && error_type.is_empty() {
        None
    } else {
        Some(NegativeExpectation { phase, error_type })
    }
}

fn scan_harness_files(
    root: &Path,
    dir: &Path,
    files: &mut Vec<(String, PathBuf)>,
) -> Result<(), String> {
    let mut entries = fs::read_dir(dir)
        .map_err(|err| format!("failed to read harness dir {}: {err}", dir.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| format!("failed to iterate harness dir {}: {err}", dir.display()))?;
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|err| format!("failed to read file type {}: {err}", path.display()))?;
        if file_type.is_dir() {
            scan_harness_files(root, &path, files)?;
            continue;
        }
        if file_type.is_file() && path.extension().and_then(|value| value.to_str()) == Some("js") {
            let name = path
                .strip_prefix(root)
                .map_err(|err| {
                    format!(
                        "failed to make relative harness path {}: {err}",
                        path.display()
                    )
                })?
                .to_string_lossy()
                .replace('\\', "/");
            files.push((name, path));
        }
    }

    Ok(())
}

fn read_git_head(path: &Path) -> Option<String> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(path)
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn hash_manifest(
    pinned_revisions: &PinnedRevisions,
    cases: &[TestCase],
    filter: Option<&str>,
) -> u64 {
    let mut hasher = DefaultHasher::new();
    pinned_revisions.ecma262.hash(&mut hasher);
    pinned_revisions.test262.hash(&mut hasher);
    filter.hash(&mut hasher);
    for case in cases {
        case.path.hash(&mut hasher);
    }
    hasher.finish()
}

fn hash_detail(detail: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    detail.hash(&mut hasher);
    hasher.finish()
}

fn json_escape(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn js_string_literal(input: &str) -> String {
    format!(
        "'{}'",
        input
            .replace('\\', "\\\\")
            .replace('\'', "\\'")
            .replace('\n', "\\n")
    )
}

fn repo_root_from_suite(suite_root: &Path) -> PathBuf {
    suite_root
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fake_test262")
    }

    fn fixture_config() -> SuiteConfig {
        let root = fixture_root();
        SuiteConfig {
            suite_root: root.join("vendor").join("test262"),
            local_harness_path: root.join("harness.js"),
            snapshot_dir: std::env::temp_dir()
                .join(format!("porffor-test262-fixture-{}", std::process::id())),
            timeout_ms: 1_000,
            worker_count: 2,
        }
    }

    fn synthetic_case(path: &str) -> TestCase {
        TestCase {
            path: path.to_string(),
            source_path: PathBuf::from(path),
            original_source: "0;".to_string(),
            flags: BTreeSet::new(),
            includes: Vec::new(),
            negative: None,
            is_module: false,
        }
    }

    #[test]
    fn discover_suite_parses_frontmatter() {
        let manifest =
            discover_suite(&fixture_config(), None).expect("fixture suite should discover");
        assert_eq!(manifest.cases.len(), 3);
        let module_case = manifest
            .cases
            .iter()
            .find(|case| case.path.ends_with("module-pass.js"))
            .expect("module case should exist");
        assert!(module_case.is_module);
        assert!(module_case.flags.contains("module"));
    }

    #[test]
    fn load_preludes_prefers_local_override() {
        let store = load_preludes(&fixture_config()).expect("preludes should load");
        let assert = store.get("assert.js").expect("assert prelude should exist");
        assert_eq!(assert.origin, PreludeOrigin::LocalMerged);
        assert!(assert.contents.contains("local assert"));
        let helper = store
            .get("helper.js")
            .expect("vendored helper should exist");
        assert_eq!(helper.origin, PreludeOrigin::VendoredHarness);
    }

    #[test]
    fn materialize_test_applies_preludes_and_strict() {
        let config = fixture_config();
        let manifest =
            discover_suite(&config, Some("language/pass")).expect("fixture suite should discover");
        let store = load_preludes(&config).expect("preludes should load");
        let case = manifest
            .cases
            .iter()
            .find(|case| case.path.ends_with("strict-pass.js"))
            .expect("strict case should exist");
        let materialized = materialize_test(case, &store).expect("materialization should work");
        assert!(materialized.source.starts_with("\"use strict\";"));
        assert!(materialized.source.contains("local assert"));
        assert!(materialized.source.contains("vendored helper"));
    }

    #[test]
    fn parse_frontmatter_block_handles_cr_line_endings() {
        let source =
            "/*---\resid: test\rincludes: [nativeFunctionMatcher.js]\rflags: [raw]\r---*/\r0;";
        let frontmatter = parse_frontmatter_block(source);

        assert_eq!(frontmatter.get("esid"), Some(&"test".to_string()));
        assert_eq!(
            frontmatter.get("includes"),
            Some(&"[nativeFunctionMatcher.js]".to_string())
        );
        assert_eq!(frontmatter.get("flags"), Some(&"[raw]".to_string()));
    }

    #[test]
    fn run_full_writes_snapshot_and_marks_completed() {
        let config = fixture_config();
        let run_config = RunConfig {
            snapshot_name: "fixture".to_string(),
            ..RunConfig::default()
        };
        let summary = run_full(&config, run_config).expect("run should complete");
        assert_eq!(summary.total, 3);
        assert_eq!(summary.completed_paths.len(), 3);
        let files = fs::read_dir(config.snapshot_dir)
            .expect("snapshot dir should exist")
            .count();
        assert!(files >= 2);
    }

    #[test]
    fn resume_uses_existing_snapshot() {
        let config = fixture_config();
        let run_config = RunConfig {
            snapshot_name: "resume".to_string(),
            ..RunConfig::default()
        };
        let first = run_full(&config, run_config.clone()).expect("first run should complete");
        let second = run_full(
            &config,
            RunConfig {
                resume: true,
                ..run_config
            },
        )
        .expect("resume run should complete");
        assert_eq!(first.total, second.total);
        assert_eq!(second.completed_paths.len(), first.completed_paths.len());
    }

    #[test]
    fn baseline_report_groups_by_kind_and_subtree() {
        let config = fixture_config();
        let run_config = RunConfig {
            snapshot_name: "baseline-report".to_string(),
            ..RunConfig::default()
        };
        let summary = run_full(&config, run_config).expect("run should complete");
        let report = baseline_report(&summary);
        assert_eq!(report.total, 3);
        assert_eq!(report.passed, 3);
        assert!(report.buckets.iter().all(|bucket| bucket.total == 0));
    }

    #[test]
    fn report_all_aggregates_fixture_suite() {
        let config = fixture_config();
        let summary = run_top_level_matrix(
            &config,
            RunConfig {
                snapshot_name: "aggregate".to_string(),
                ..RunConfig::default()
            },
        )
        .expect("aggregate run should complete");
        assert_eq!(summary.total, 3);
        assert_eq!(summary.passed, 3);
        assert_eq!(summary.failed, 0);
        assert!(summary.entries.len() >= TOP_LEVEL_FILTERS.len());
        assert!(summary
            .entries
            .iter()
            .any(|entry| entry.filter == "language/pass"));
    }

    #[test]
    fn report_all_resume_uses_aggregate_snapshot() {
        let config = fixture_config();
        let run_config = RunConfig {
            snapshot_name: "aggregate-resume".to_string(),
            ..RunConfig::default()
        };
        let first = run_top_level_matrix(&config, run_config.clone()).expect("first matrix run");
        let second = run_top_level_matrix(
            &config,
            RunConfig {
                resume: true,
                ..run_config
            },
        )
        .expect("resume matrix run");
        assert_eq!(first.total, second.total);
        assert_eq!(first.passed, second.passed);

        let nodes = build_run_matrix(&config).expect("matrix should build");
        let aggregate_hash = hash_matrix_nodes(&nodes, ExecutionBackend::SpecExec);
        let json_path = config
            .snapshot_dir
            .join(format!("aggregate-resume-aggregate-{aggregate_hash}.json"));
        let raw = fs::read_to_string(json_path).expect("aggregate snapshot should exist");
        assert!(raw.contains("\"completed_nodes\""));
        assert!(raw.contains("\"aggregate_entries\""));
    }

    #[test]
    fn matrix_builder_chunks_oversized_leaves_without_file_nodes() {
        let cases = (0..7)
            .map(|index| {
                synthetic_case(&format!(
                    "built-ins/RegExp/property-escapes/generated/test-{index}.js"
                ))
            })
            .collect::<Vec<_>>();
        let nodes = build_matrix_nodes_for_root("built-ins", &cases, 3, 2);
        assert_eq!(nodes.len(), 4);
        assert!(nodes
            .iter()
            .all(|node| node.node_kind == MatrixNodeKind::ChunkLeaf));
        assert!(nodes
            .iter()
            .all(|node| node.filter == "built-ins/RegExp/property-escapes"));
        assert!(nodes.iter().all(|node| !node.node_id.ends_with(".js")));
        assert_eq!(
            nodes[0].node_id,
            "built-ins/RegExp/property-escapes@chunk-0001-of-0004"
        );
    }

    #[test]
    fn non_split_root_stays_single_leaf_even_when_large() {
        let cases = (0..7)
            .map(|index| synthetic_case(&format!("annexB/example-{index}.js")))
            .collect::<Vec<_>>();
        let nodes = build_matrix_nodes_for_root("annexB", &cases, 3, 2);
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].node_kind, MatrixNodeKind::FilterLeaf);
        assert_eq!(nodes[0].node_id, "annexB");
        assert_eq!(nodes[0].total_cases, 7);
    }

    #[test]
    fn aggregate_resume_rejects_stale_matrix_strategy_snapshot() {
        let config = fixture_config();
        let nodes = build_run_matrix(&config).expect("matrix should build");
        let aggregate_hash = hash_matrix_nodes(&nodes, ExecutionBackend::SpecExec);
        let pinned = pinned_revisions(&config);
        let mut counts_per_kind = BTreeMap::new();
        let mut counts_per_origin = BTreeMap::new();
        for kind in FailureKind::ALL {
            counts_per_kind.insert(kind, 0);
        }
        for origin in FailureOrigin::ALL {
            counts_per_origin.insert(origin, 0);
        }
        let mut snapshot = aggregate_snapshot(
            &pinned,
            aggregate_hash,
            &AggregateRunSummary {
                total: 0,
                passed: 0,
                failed: 0,
                counts_per_kind,
                counts_per_origin,
                entries: Vec::new(),
            },
            ExecutionBackend::SpecExec,
            "aggregate-matrix",
            vec!["top-level".to_string()],
            Vec::new(),
        );
        snapshot.matrix_strategy_version = MATRIX_STRATEGY_VERSION - 1;
        write_snapshot(&config, &snapshot, "stale-aggregate-aggregate")
            .expect("stale aggregate snapshot should write");

        let err = run_top_level_matrix(
            &config,
            RunConfig {
                resume: true,
                snapshot_name: "stale-aggregate".to_string(),
                ..RunConfig::default()
            },
        )
        .expect_err("stale aggregate snapshot should be rejected");
        assert!(err.contains("matrix_strategy_version"));
    }

    #[test]
    fn resolution_negative_runs_execution_before_failing() {
        let root = std::env::temp_dir().join(format!(
            "porffor-test262-resolution-negative-{}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("temp root should exist");
        let source_path = root.join("resolution-negative.mjs");
        fs::write(
            &source_path,
            "import './does-not-exist.mjs';\nexport const value = 1;\n",
        )
        .expect("source file should write");

        let case = TestCase {
            path: "language/module-code/resolution-negative.js".to_string(),
            source_path: source_path.clone(),
            original_source: "import './does-not-exist.mjs';\nexport const value = 1;\n"
                .to_string(),
            flags: BTreeSet::from(["module".to_string()]),
            includes: Vec::new(),
            negative: Some(NegativeExpectation {
                phase: "resolution".to_string(),
                error_type: String::new(),
            }),
            is_module: true,
        };

        let result = run_one_case(
            &case,
            &PreludeStore::default(),
            5_000,
            ExecutionBackend::SpecExec,
        );
        assert!(matches!(result.status, TestStatus::Passed));
    }

    #[test]
    fn classify_failure_tags_origin() {
        let failure = classify_failure(
            "language/example.js",
            FailureKind::Runtime,
            "ReferenceError: boom",
        );
        assert_eq!(failure.origin, FailureOrigin::BoaRuntime);
        assert!(failure.detail.starts_with("[origin:boa-runtime] "));
    }

    #[test]
    fn snapshot_json_includes_version_field() {
        let config = fixture_config();
        let run_config = RunConfig {
            snapshot_name: "versioned".to_string(),
            ..RunConfig::default()
        };
        run_full(&config, run_config).expect("run should complete");
        let manifest = discover_suite(&config, None).expect("fixture suite should discover");
        let json_path = config
            .snapshot_dir
            .join(format!("versioned-{}.json", manifest.manifest_hash));
        let raw = fs::read_to_string(json_path).expect("snapshot should be readable");
        assert!(raw.contains(&format!("\"snapshot_version\": {SNAPSHOT_VERSION}")));
        assert!(raw.contains(&format!(
            "\"matrix_strategy_version\": {MATRIX_STRATEGY_VERSION}"
        )));
        assert!(raw.contains("\"execution_backend\": \"spec-exec\""));
        assert!(raw.contains("\"matrix_path\": []"));
    }
}
