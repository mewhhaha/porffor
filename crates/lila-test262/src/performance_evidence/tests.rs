//! Synthetic evidence exercises the real metadata discovery, matrix admission
//! and snapshot writers. No fixture invokes a case worker or compiler.
use super::*;
use std::time::{SystemTime, UNIX_EPOCH};

const BACKEND: ExecutionBackend = ExecutionBackend::WasmAot;
const NAME: &str = "performance-fixture";
static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    root: PathBuf,
    config: SuiteConfig,
    matrix: Vec<RunMatrixNode>,
}

impl Fixture {
    fn new() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "lila-performance-evidence-{}-{stamp}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let config = SuiteConfig {
            suite_root: root.join("vendor/test262"),
            snapshot_dir: root.join("snapshots"),
            local_harness: LocalHarnessSource::None,
            timeout_ms: 1_000,
            worker_count: 3,
            case_runner_bin: None,
            execution_role: CaseExecutionRole::Supervisor,
        };
        for (category, count) in [("Array", 12), ("Object", 2)] {
            let directory = config.suite_root.join("test/built-ins").join(category);
            fs::create_dir_all(&directory).unwrap();
            for index in 0..count {
                fs::write(
                    directory.join(format!("case-{index:02}.js")),
                    "/*---\nflags: [noStrict]\n---*/\n0;\n",
                )
                .unwrap();
            }
        }
        let matrix =
            load_or_build_run_matrix(&config, BACKEND).expect("metadata-only matrix should build");
        assert_eq!(matrix.len(), TOP_LEVEL_FILTERS.len() + 1);
        assert_eq!(
            matrix
                .iter()
                .find(|node| node.node_id == "built-ins/Array")
                .unwrap()
                .total_cases,
            12
        );
        assert_eq!(
            matrix
                .iter()
                .find(|node| node.node_id == "built-ins/Object")
                .unwrap()
                .total_cases,
            2
        );
        assert_eq!(
            matrix.iter().filter(|node| node.total_cases == 0).count(),
            TOP_LEVEL_FILTERS.len() - 1
        );
        Self {
            root,
            config,
            matrix,
        }
    }

    fn index(&self, node_id: &str) -> usize {
        self.matrix
            .iter()
            .position(|node| node.node_id == node_id)
            .expect("original matrix must contain the selected node")
    }

    fn settings(&self) -> TimingSettings {
        TimingSettings {
            resume_requested: true,
            configured_workers: self.config.worker_count,
            timeout_ms: self.config.timeout_ms,
            os: "fixture-os".into(),
            architecture: "fixture-arch".into(),
        }
    }

    fn node(
        &self,
        index: usize,
        elapsed: Option<u128>,
    ) -> (TopLevelRunSummary, ProgressSnapshot, PathBuf) {
        let node = &self.matrix[index];
        let discovered = discover_suite(&self.config, Some(&node.filter)).unwrap();
        let duration_group = usize::from(node.node_id == "built-ins/Object");
        let cases = discovered
            .cases
            .into_iter()
            .filter(|case| node.case_ids.contains(&case.execution_id))
            .collect();
        let manifest = SuiteManifest {
            pinned_revisions: pinned_revisions(&self.config),
            manifest_hash: matrix_node_manifest_hash(&pinned_revisions(&self.config), node),
            filter: Some(node.node_id.clone()),
            cases,
        };
        let results = node
            .case_ids
            .iter()
            .enumerate()
            .map(|(ordinal, id)| {
                // Passed slow cases exceed the configured timeout as recorded
                // durations. Only genuinely classified failures count as timeouts.
                let duration_ms = 10_000 + duration_group as u128 * 100 + ordinal as u128;
                let status = if node.node_id == "built-ins/Array" && ordinal < 2 {
                    let detail = "fixture worker timeout exceeded".to_string();
                    TestStatus::Failed(FailureRecord {
                        test_id: id.clone(),
                        test_path: id.path().into(),
                        kind: FailureKind::WasmBackend,
                        outcome: OutcomeKind::Crash,
                        origin: FailureOrigin::Unknown,
                        detail_hash: hash_detail(&detail),
                        detail,
                        duration_ms: None,
                    })
                } else {
                    TestStatus::Passed
                };
                TestResult {
                    test_id: id.clone(),
                    status,
                    duration_ms,
                }
            })
            .collect::<Vec<_>>();
        let summary = summarize_results(&results);
        let identity = CheckpointRunIdentity::matrix(node.node_kind, &node.matrix_path).unwrap();
        let mut snapshot = snapshot_from_summary(
            &manifest,
            identity.terminal_run_kind().into(),
            &summary,
            BACKEND,
        )
        .unwrap();
        snapshot.matrix_path = node.matrix_path.clone();
        let entry = TopLevelRunSummary {
            node_id: node.node_id.clone(),
            node_kind: node.node_kind,
            filter: node.filter.clone(),
            matrix_path: node.matrix_path.clone(),
            total: summary.total,
            passed: summary.passed,
            failed: summary.failures.len(),
            counts_per_kind: summary.counts_per_kind,
            counts_per_outcome: summary.counts_per_outcome,
            counts_per_origin: counts_per_origin(&summary.failures),
            manifest_hash: manifest.manifest_hash,
        };
        let name = format!("{NAME}-{}", sanitize_filter_for_snapshot(&node.node_id));
        let paths = write_snapshot(&self.config, &snapshot, &name).unwrap();
        validate_complete_node_contract(&snapshot, node, &entry, &paths.json_path).unwrap();
        if let Some(elapsed_ns) = elapsed {
            // Explicit synthetic timing; the production timer is not invoked.
            write_timing(
                &paths.json_path,
                &snapshot,
                node,
                FinishedNodeInvocation {
                    elapsed_ns,
                    settings: self.settings(),
                },
            )
            .unwrap();
        }
        (entry, snapshot, paths.json_path)
    }

    fn complete_entries(&self, nonempty: &[TopLevelRunSummary]) -> Vec<TopLevelRunSummary> {
        self.matrix
            .iter()
            .enumerate()
            .map(|(index, node)| {
                if let Some(entry) = nonempty.iter().find(|entry| entry.node_id == node.node_id) {
                    return entry.clone();
                }
                assert_eq!(
                    node.total_cases, 0,
                    "complete evidence must retain every nonempty original node"
                );
                // Empty roots are genuine complete terminal nodes from the full
                // original matrix, with unavailable wall timing and no case work.
                self.node(index, None).0
            })
            .collect()
    }

    fn aggregate(&self, entries: &[TopLevelRunSummary]) -> PathBuf {
        let summary = aggregate_from_entries(entries);
        let snapshot = aggregate_snapshot(
            &pinned_revisions(&self.config),
            hash_matrix_nodes(&self.matrix, BACKEND),
            &summary,
            BACKEND,
            "aggregate-matrix",
            vec!["top-level".into()],
            summary
                .entries
                .iter()
                .map(|entry| entry.node_id.clone())
                .collect(),
        )
        .unwrap();
        write_snapshot(&self.config, &snapshot, &format!("{NAME}-aggregate"))
            .unwrap()
            .json_path
    }

    fn replace_node(&self, index: usize, snapshot: &ProgressSnapshot, path: &Path) {
        fs::write(path, render_snapshot_json(snapshot)).unwrap();
        if timing_path(path).exists() {
            write_timing(
                path,
                snapshot,
                &self.matrix[index],
                FinishedNodeInvocation {
                    elapsed_ns: 7,
                    settings: self.settings(),
                },
            )
            .unwrap();
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn timing_admission_binds_exact_bytes_compiler_pins_backend_and_node() {
    let fixture = Fixture::new();
    let index = fixture.index("built-ins/Array");
    let (_, snapshot, path) = fixture.node(index, Some(7));
    let node = &fixture.matrix[index];
    let bytes = fs::read(&path).unwrap();
    let timing_bytes = fs::read(timing_path(&path)).unwrap();
    let original: NodeTiming = serde_json::from_slice(&timing_bytes).unwrap();
    original.admit(&snapshot, node, &bytes).unwrap();
    assert_eq!(original.settings, fixture.settings());
    assert_eq!(original.elapsed_ns, 7);
    let mutations: &[fn(&mut NodeTiming)] = &[
        |record| record.version += 1,
        |record| record.scope.push('!'),
        |record| record.execution_backend = "spec-exec".into(),
        |record| record.ecma262_revision.push('!'),
        |record| record.test262_revision.push('!'),
        |record| record.manifest_hash ^= 1,
        |record| record.node_id.push('!'),
        |record| record.snapshot_sha256 = "0".repeat(64),
        |record| record.settings.os.clear(),
        |record| record.settings.architecture.clear(),
    ];
    for mutate in mutations {
        let mut damaged = original.clone();
        mutate(&mut damaged);
        assert!(damaged.admit(&snapshot, node, &bytes).is_err());
    }
    let mut compiler = serde_json::to_value(&original).unwrap();
    compiler["compiler"]["executable_sha256"] = "0".repeat(64).into();
    let foreign: NodeTiming = serde_json::from_value(compiler).unwrap();
    assert!(foreign.admit(&snapshot, node, &bytes).is_err());
    let mut reserialized = bytes.clone();
    reserialized.push(b' ');
    assert_eq!(
        snapshot_from_file(decode_snapshot_bytes(&reserialized, &path).unwrap()).unwrap(),
        snapshot
    );
    assert!(
        original.admit(&snapshot, node, &reserialized).is_err(),
        "same JSON value must not acquire the old exact-byte timing"
    );
    let mut unknown = serde_json::to_value(&original).unwrap();
    unknown["invented_duration_ms"] = 1.into();
    assert!(serde_json::from_value::<NodeTiming>(unknown).is_err());
}

#[test]
fn partial_reports_include_only_completed_nodes_and_measured_invocations() {
    let fixture = Fixture::new();
    let (first, _, path) = fixture.node(fixture.index("built-ins/Array"), Some(7));
    let (second, _, _) = fixture.node(fixture.index("built-ins/Object"), Some(19));
    // A complete sibling file is not part of this admitted partial aggregate.
    fixture.aggregate(std::slice::from_ref(&first));
    let report = load(&fixture.config, NAME, BACKEND).unwrap();
    assert!(!report.matrix_complete());
    assert_eq!(report.expected_nodes, fixture.matrix.len());
    assert_eq!(report.completed_nodes, 1);
    assert_eq!(report.nodes.len(), 1);
    assert_eq!(report.total, 12);
    assert_eq!(report.passed, 10);
    assert_eq!(report.timeout_count(), 2);
    assert_eq!(report.nodes_with_wall_timing(), 1);
    assert_eq!(report.measured_invocation_sum_ns, 7);
    assert_eq!(
        report.nodes[0].snapshot_sha256,
        digest(&fs::read(path).unwrap())
    );
    assert_eq!(report.nodes[0].retained_slowest.len(), 10);
    assert_eq!(report.retained_slowest.len(), 10);
    assert!(report.nodes[0]
        .timeouts
        .iter()
        .all(|id| first.node_id.starts_with("built-ins/Array")
            && id.path().starts_with("built-ins/Array/")));
    fixture.aggregate(&[first.clone(), second.clone()]);
    let incomplete = load(&fixture.config, NAME, BACKEND).unwrap();
    assert!(!incomplete.matrix_complete());
    assert_eq!(incomplete.completed_nodes, 2);
    let all = fixture.complete_entries(&[first, second]);
    fixture.aggregate(&all);
    let complete = load(&fixture.config, NAME, BACKEND).unwrap();
    assert_eq!(complete.expected_nodes, fixture.matrix.len());
    assert_eq!(complete.completed_nodes, fixture.matrix.len());
    assert_eq!(
        complete.nodes.iter().filter(|node| node.total == 0).count(),
        TOP_LEVEL_FILTERS.len() - 1
    );
    assert!(complete.matrix_complete());
    assert_eq!(complete.total, 14);
    assert_eq!(complete.passed, 12);
    assert_eq!(complete.timeout_count(), 2);
    assert_eq!(complete.nodes_with_wall_timing(), 2);
    assert_eq!(complete.measured_invocation_sum_ns, 26);
    assert_eq!(complete.retained_slowest.len(), 10);
    assert!(complete
        .retained_slowest
        .windows(2)
        .all(|pair| pair[0].duration_ms > pair[1].duration_ms
            || (pair[0].duration_ms == pair[1].duration_ms
                && pair[0].execution <= pair[1].execution)));
    assert_eq!(complete.retained_slowest[0].duration_ms, 10_101);
}

#[test]
fn earlier_complete_snapshots_without_timing_remain_explicitly_unavailable() {
    let fixture = Fixture::new();
    let (first, _, _) = fixture.node(fixture.index("built-ins/Array"), None);
    let (second, _, _) = fixture.node(fixture.index("built-ins/Object"), None);
    fixture.aggregate(&fixture.complete_entries(&[first, second]));
    let report = load(&fixture.config, NAME, BACKEND).unwrap();
    assert!(report.matrix_complete());
    assert_eq!(report.nodes_with_wall_timing(), 0);
    assert_eq!(report.measured_invocation_sum_ns, 0);
    assert_eq!(report.completed_nodes, fixture.matrix.len());
    assert!(report
        .nodes
        .iter()
        .all(|node| matches!(&node.wall_timing, WallTiming::Unavailable { .. })));
    let json: serde_json::Value = serde_json::from_str(&report.to_pretty_json().unwrap()).unwrap();
    assert_eq!(json["nodes"][0]["wall_timing"]["status"], "unavailable");
    assert!(json["nodes"][0]["wall_timing"].get("elapsed_ns").is_none());
}

#[test]
fn performance_reports_refuse_incomplete_modes_timeout_damage_and_stale_timing() {
    let fixture = Fixture::new();
    let index = fixture.index("built-ins/Array");
    let (entry, original, path) = fixture.node(index, Some(7));
    fixture.aggregate(std::slice::from_ref(&entry));
    let mut damaged = original.clone();
    damaged.completed_test_ids.pop();
    fixture.replace_node(index, &damaged, &path);
    assert!(
        load(&fixture.config, NAME, BACKEND).is_err(),
        "matching timing cannot admit an unfinished matrix node"
    );
    let mut damaged = original.clone();
    let old = damaged.completed_test_ids[0].clone();
    let changed = TestExecutionId::new(old.path(), TestExecutionMode::StrictScript);
    for id in &mut damaged.completed_test_ids {
        if *id == old {
            *id = changed.clone();
        }
    }
    for (id, _) in &mut damaged.slowest_tests {
        if *id == old {
            *id = changed.clone();
        }
    }
    for id in &mut damaged.timeout_list {
        if *id == old {
            *id = changed.clone();
        }
    }
    for failure in &mut damaged.failures {
        if failure.test_id == old {
            failure.test_id = changed.clone();
        }
    }
    fixture.replace_node(index, &damaged, &path);
    assert!(
        load(&fixture.config, NAME, BACKEND).is_err(),
        "mode identity remains owned by the original matrix"
    );
    let mut damaged = original.clone();
    damaged.timeout_list.clear();
    fixture.replace_node(index, &damaged, &path);
    assert!(
        load(&fixture.config, NAME, BACKEND).is_err(),
        "timing cannot replace the actual timeout classification"
    );
    fixture.replace_node(index, &original, &path);
    let old_timing = fs::read(timing_path(&path)).unwrap();
    let mut damaged = original.clone();
    damaged.slowest_tests[0].1 += 1;
    fs::write(&path, render_snapshot_json(&damaged)).unwrap();
    assert!(
        load(&fixture.config, NAME, BACKEND).is_err(),
        "valid changed node data cannot inherit stale byte-bound timing"
    );
    fs::write(&path, render_snapshot_json(&original)).unwrap();
    fs::write(timing_path(&path), old_timing).unwrap();
    load(&fixture.config, NAME, BACKEND).unwrap();
    fs::remove_file(&path).unwrap();
    assert!(
        load(&fixture.config, NAME, BACKEND).is_err(),
        "declared completed nodes require their original complete evidence"
    );
}
