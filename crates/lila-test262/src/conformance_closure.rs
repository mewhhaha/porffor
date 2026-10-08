//! A release verdict constructed only by two fresh full product runs.
//!
//! Reporting and publication may retain complete red evidence. This owner
//! instead executes both families itself and requires exact, all-pass results.

use super::*;

const FIRST_FAMILY: &str = "closure-first";
const SECOND_FAMILY: &str = "closure-second";
static WORKSPACE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum ClosureRejection {
    EmptyDenominator,
    FirstRunNonpass { count: usize },
    SecondRunNonpass { count: usize },
    DifferentResultSets,
}

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum ClosureVerdict {
    Passed,
    Rejected { reasons: Vec<ClosureRejection> },
}

#[derive(Debug, Serialize)]
struct FamilyReport {
    family: String,
    aggregate_json: PathBuf,
    total: usize,
    passed: usize,
    nonpass: usize,
    counts_per_kind: BTreeMap<FailureKind, usize>,
    counts_per_outcome: BTreeMap<OutcomeKind, usize>,
    counts_per_origin: BTreeMap<FailureOrigin, usize>,
    elapsed_ms: u128,
}

/// Only `run_closure` constructs this evidence. There is no decoder or
/// constructor that turns two names, existing snapshots or caller counts green.
#[derive(Debug, Serialize)]
pub struct ClosureReport {
    schema_version: u32,
    execution_backend: String,
    compiler_identity: CompilerProvenance,
    pinned_revisions: PinnedRevisions,
    manifest_hash: u64,
    exact_result_sets_match: bool,
    first: FamilyReport,
    second: FamilyReport,
    verdict: ClosureVerdict,
    report_path: PathBuf,
}

impl ClosureReport {
    pub fn is_passed(&self) -> bool {
        match &self.verdict {
            ClosureVerdict::Passed => true,
            ClosureVerdict::Rejected { .. } => false,
        }
    }

    pub fn report_path(&self) -> &Path {
        &self.report_path
    }
}

struct FreshWorkspace {
    root: PathBuf,
    first: PathBuf,
    second: PathBuf,
}

impl FreshWorkspace {
    fn create(base: &Path) -> Result<Self, String> {
        fs::create_dir_all(base)
            .map_err(|error| format!("cannot create closure evidence parent: {error}"))?;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|error| format!("cannot name fresh closure evidence: {error}"))?
            .as_nanos();
        for _ in 0..64 {
            let sequence = WORKSPACE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let root = base.join(format!(
                "closure-{}-{timestamp}-{sequence}",
                std::process::id()
            ));
            match fs::create_dir(&root) {
                Ok(()) => {
                    let first = root.join("first");
                    let second = root.join("second");
                    fs::create_dir(&first)
                        .and_then(|()| fs::create_dir(&second))
                        .map_err(|error| {
                            format!("cannot create fresh closure families: {error}")
                        })?;
                    return Ok(Self {
                        root,
                        first,
                        second,
                    });
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(format!("cannot reserve closure evidence: {error}")),
            }
        }
        Err("cannot reserve an unused closure evidence directory".into())
    }
}

/// Admit the actual product suite and its clean committed content. An alternate
/// fixture root, working-tree pin, custom harness or worker role cannot close T26.
fn admit_product_config(config: &SuiteConfig) -> Result<PinnedRevisions, String> {
    let directory = std::env::current_dir()
        .map_err(|error| format!("cannot locate the current product checkout: {error}"))?;
    let checkout = git_capture(&directory, &["rev-parse", "--show-toplevel"])
        .ok_or("closure requires the current product checkout and its pinned suite")?;
    let expected_root = PathBuf::from(checkout)
        .join(SuiteConfig::default().suite_root)
        .canonicalize()
        .map_err(|error| format!("cannot locate the current pinned Test262 suite: {error}"))?;
    let actual_root = config
        .suite_root
        .canonicalize()
        .map_err(|error| format!("cannot locate closure Test262 suite: {error}"))?;
    if actual_root != expected_root {
        return Err("closure requires the current pinned product suite, not a fixture or alternate suite root".into());
    }
    if config.local_harness != LocalHarnessSource::EmbeddedWasmAot
        || config.execution_role != CaseExecutionRole::Supervisor
        || config.worker_count != 1
        || config.timeout_ms != SuiteConfig::default().timeout_ms
    {
        return Err("closure requires the complete product harness, supervised serial cases and the standard release timeout".into());
    }
    let selected = config
        .case_runner_bin
        .as_ref()
        .ok_or("closure requires the actual running CLI as its supervised case worker")?;
    let selected = selected
        .canonicalize()
        .map_err(|error| format!("cannot locate closure worker: {error}"))?;
    let running = std::env::current_exe()
        .and_then(|path| path.canonicalize())
        .map_err(|error| format!("cannot locate the running closure executable: {error}"))?;
    if selected != running {
        return Err("closure worker must be the actual running executable".into());
    }
    let pin = current_clean_pin(&config.suite_root)?;
    let pinned = pinned_revisions(config);
    if pinned.test262 != pin {
        return Err("closure refuses a stale cached suite pin".into());
    }
    Ok(pinned)
}

fn current_clean_pin(suite_root: &Path) -> Result<String, String> {
    let context = suite_git_context(suite_root)
        .ok_or("closure requires a committed pinned Test262 checkout")?;
    let pathspec = if context.prefix.is_empty() {
        "."
    } else {
        context.prefix.as_str()
    };
    let status = git_capture(
        &context.toplevel,
        &[
            "status",
            "--porcelain",
            "--untracked-files=all",
            "--ignored",
            "--",
            pathspec,
        ],
    )
    .ok_or("cannot verify clean committed Test262 content")?;
    // A suite can be its own repository. The ordinary pin producer returns
    // HEAD in that case; HEAD alone cannot prove its working tree stayed clean.
    if !status.is_empty() {
        return Err("closure requires clean committed Test262 content; modified, untracked or ignored suite files cannot pass".into());
    }
    let pin = compute_suite_test262_pin(suite_root);
    if !is_full_hex_oid(&pin) || resolve_pin_to_suite_tree(&context, &pin).is_none() {
        return Err("closure requires clean committed Test262 content; worktree or missing pins cannot pass".into());
    }
    Ok(pin)
}

fn require_unchanged_suite(config: &SuiteConfig, pinned: &PinnedRevisions) -> Result<(), String> {
    // Bypass the process pin cache at every boundary of the two actual runs.
    if current_clean_pin(&config.suite_root)? != pinned.test262 {
        return Err(
            "pinned Test262 content changed during closure; no release verdict is valid".into(),
        );
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CaseResult {
    Passed,
    Nonpass {
        kind: FailureKind,
        outcome: OutcomeKind,
        origin: FailureOrigin,
        detail_hash: u64,
    },
}

struct FreshFamilyEvidence {
    verified: VerifiedAggregateSummary,
    results: BTreeMap<TestExecutionId, CaseResult>,
    elapsed_ms: u128,
}

fn collect_complete_results(
    config: &SuiteConfig,
    verified: &VerifiedAggregateSummary,
) -> Result<BTreeMap<TestExecutionId, CaseResult>, String> {
    let nodes = load_or_build_run_matrix(config, ExecutionBackend::WasmAot)?;
    let entries = verified
        .summary
        .entries
        .iter()
        .map(|entry| (entry.node_id.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    let mut results = BTreeMap::new();
    for node in &nodes {
        let entry = entries
            .get(node.node_id.as_str())
            .copied()
            .ok_or_else(|| format!("closure is missing matrix entry {}", node.node_id))?;
        let snapshot = load_completed_node_snapshot(
            config,
            &verified.resolved_snapshot_name,
            node,
            ExecutionBackend::WasmAot,
            entry,
            &verified.compiler_identity,
        )?
        .ok_or_else(|| format!("closure is missing node evidence {}", node.node_id))?;
        snapshot
            .provenance
            .require_running("closure case result set")?;
        for id in snapshot.completed_test_ids {
            if results.insert(id, CaseResult::Passed).is_some() {
                return Err("closure result set contains a duplicate execution".into());
            }
        }
        for failure in snapshot.failures {
            let result = results
                .get_mut(&failure.test_id)
                .ok_or("closure failure is outside the completed execution set")?;
            *result = CaseResult::Nonpass {
                kind: failure.kind,
                outcome: failure.outcome,
                origin: failure.origin,
                detail_hash: failure.detail_hash,
            };
        }
    }
    let nonpass = results
        .values()
        .filter(|result| matches!(result, CaseResult::Nonpass { .. }))
        .count();
    if results.len() != verified.summary.total || nonpass != verified.summary.failed {
        return Err("closure exact result set disagrees with validated aggregate counts".into());
    }
    Ok(results)
}

fn run_fresh_family(
    config: &SuiteConfig,
    family: &str,
    compiler: &CompilerProvenance,
    pinned: &PinnedRevisions,
) -> Result<FreshFamilyEvidence, String> {
    if fs::read_dir(&config.snapshot_dir)
        .map_err(|error| error.to_string())?
        .next()
        .is_some()
    {
        return Err(
            "closure refuses a nonempty family directory; loaded evidence is not a fresh run"
                .into(),
        );
    }
    require_unchanged_suite(config, pinned)?;
    let started = Instant::now();
    let produced = run_top_level_matrix(
        config,
        RunConfig {
            snapshot_name: family.into(),
            execution_backend: ExecutionBackend::WasmAot,
            // No filter, shard, node limit or resume enters this owner.
            ..RunConfig::default()
        },
    )?;
    require_unchanged_suite(config, pinned)?;
    let verified = load_verified_aggregate_summary(config, family, ExecutionBackend::WasmAot)?;
    if verified.resolved_snapshot_name != family
        || verified.compiler_identity != *compiler
        || verified.recorded_pinned_revisions != *pinned
        || verified.summary != produced
    {
        return Err("closure evidence does not belong to this exact fresh full run".into());
    }
    verified
        .compiler_identity
        .require_running("fresh closure family")?;
    let results = collect_complete_results(config, &verified)?;
    Ok(FreshFamilyEvidence {
        verified,
        results,
        elapsed_ms: started.elapsed().as_millis(),
    })
}

fn verdict_for_results(
    first: &BTreeMap<TestExecutionId, CaseResult>,
    second: &BTreeMap<TestExecutionId, CaseResult>,
) -> ClosureVerdict {
    let mut reasons = Vec::new();
    if first.is_empty() || second.is_empty() {
        reasons.push(ClosureRejection::EmptyDenominator);
    }
    let first_nonpass = first
        .values()
        .filter(|result| matches!(result, CaseResult::Nonpass { .. }))
        .count();
    let second_nonpass = second
        .values()
        .filter(|result| matches!(result, CaseResult::Nonpass { .. }))
        .count();
    if first_nonpass != 0 {
        reasons.push(ClosureRejection::FirstRunNonpass {
            count: first_nonpass,
        });
    }
    if second_nonpass != 0 {
        reasons.push(ClosureRejection::SecondRunNonpass {
            count: second_nonpass,
        });
    }
    if first != second {
        reasons.push(ClosureRejection::DifferentResultSets);
    }
    if reasons.is_empty() {
        ClosureVerdict::Passed
    } else {
        ClosureVerdict::Rejected { reasons }
    }
}

fn family_report(family: &str, evidence: &FreshFamilyEvidence) -> FamilyReport {
    let verified = &evidence.verified;
    FamilyReport {
        family: family.into(),
        aggregate_json: verified.snapshot_paths.json_path.clone(),
        total: verified.summary.total,
        passed: verified.summary.passed,
        nonpass: verified.summary.failed,
        counts_per_kind: verified.summary.counts_per_kind.clone(),
        counts_per_outcome: verified.summary.counts_per_outcome.clone(),
        counts_per_origin: verified.summary.counts_per_origin.clone(),
        elapsed_ms: evidence.elapsed_ms,
    }
}

/// Execute two distinct fresh full Wasm-AOT matrices and retain a release verdict.
///
/// Every case uses the ordinary compiler and mandatory supervised worker. A red
/// first family still runs the second family. Interrupted or invalid evidence
/// returns an error, retains its workspace, and never creates a passing receipt.
pub fn run_closure(config: SuiteConfig) -> Result<ClosureReport, String> {
    let pinned = admit_product_config(&config)?;
    let compiler = CompilerProvenance::current()?;
    let workspace = FreshWorkspace::create(&config.snapshot_dir)?;
    let result = (|| {
        let first_config = SuiteConfig {
            snapshot_dir: workspace.first.clone(),
            ..config.clone()
        };
        let second_config = SuiteConfig {
            snapshot_dir: workspace.second.clone(),
            ..config.clone()
        };
        let first = run_fresh_family(&first_config, FIRST_FAMILY, &compiler, &pinned)?;
        let second = run_fresh_family(&second_config, SECOND_FAMILY, &compiler, &pinned)?;
        require_unchanged_suite(&config, &pinned)?;
        if first.verified.manifest_hash != second.verified.manifest_hash {
            return Err("closure full-suite manifests differ between the fresh runs".into());
        }
        let report = ClosureReport {
            schema_version: 1,
            execution_backend: ExecutionBackend::WasmAot.as_str().into(),
            compiler_identity: compiler,
            pinned_revisions: pinned,
            manifest_hash: first.verified.manifest_hash,
            exact_result_sets_match: first.results == second.results,
            first: family_report(FIRST_FAMILY, &first),
            second: family_report(SECOND_FAMILY, &second),
            verdict: verdict_for_results(&first.results, &second.results),
            report_path: workspace.root.join("closure.json"),
        };
        let file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&report.report_path)
            .map_err(|error| format!("cannot create closure verdict: {error}"))?;
        serde_json::to_writer_pretty(file, &report)
            .map_err(|error| format!("cannot write closure verdict: {error}"))?;
        Ok(report)
    })();
    result.map_err(|error: String| {
        format!(
            "{error}; closure evidence retained in {}",
            workspace.root.display()
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(path: &str, mode: TestExecutionMode) -> TestExecutionId {
        TestExecutionId::new(path, mode)
    }

    fn pass_results() -> BTreeMap<TestExecutionId, CaseResult> {
        [
            (
                id("language/a.js", TestExecutionMode::SloppyScript),
                CaseResult::Passed,
            ),
            (
                id("language/a.js", TestExecutionMode::StrictScript),
                CaseResult::Passed,
            ),
        ]
        .into_iter()
        .collect()
    }

    #[test]
    fn closure_requires_nonzero_and_exact_execution_modes_in_both_runs() {
        let full = pass_results();
        assert!(matches!(
            verdict_for_results(&full, &full),
            ClosureVerdict::Passed
        ));
        let empty = BTreeMap::new();
        assert!(matches!(
            verdict_for_results(&empty, &empty),
            ClosureVerdict::Rejected { .. }
        ));
        let missing_strict = full
            .iter()
            .take(1)
            .map(|(id, result)| (id.clone(), result.clone()))
            .collect();
        assert!(matches!(
            verdict_for_results(&full, &missing_strict),
            ClosureVerdict::Rejected { .. }
        ));
        let renamed = [
            (
                id("language/b.js", TestExecutionMode::SloppyScript),
                CaseResult::Passed,
            ),
            (
                id("language/b.js", TestExecutionMode::StrictScript),
                CaseResult::Passed,
            ),
        ]
        .into_iter()
        .collect();
        assert!(matches!(
            verdict_for_results(&full, &renamed),
            ClosureVerdict::Rejected { .. }
        ));
    }

    #[test]
    fn every_nonpass_kind_outcome_and_origin_is_red_even_when_repeatable() {
        for kind in FailureKind::ALL {
            for outcome in OutcomeKind::ALL {
                if outcome == OutcomeKind::Success {
                    continue;
                }
                for origin in FailureOrigin::ALL {
                    let mut red = pass_results();
                    *red.values_mut().next().unwrap() = CaseResult::Nonpass {
                        kind,
                        outcome,
                        origin,
                        detail_hash: 17,
                    };
                    assert!(matches!(
                        verdict_for_results(&red, &red),
                        ClosureVerdict::Rejected { .. }
                    ));
                    assert!(matches!(
                        verdict_for_results(&pass_results(), &red),
                        ClosureVerdict::Rejected { .. }
                    ));
                }
            }
        }
    }

    #[test]
    fn a_result_change_cannot_be_hidden_by_equal_failure_counts() {
        let mut first = pass_results();
        let failure = CaseResult::Nonpass {
            kind: FailureKind::Runtime,
            outcome: OutcomeKind::Bug,
            origin: FailureOrigin::Unknown,
            detail_hash: 7,
        };
        *first.values_mut().next().unwrap() = failure.clone();
        let mut second = pass_results();
        *second.values_mut().last().unwrap() = failure;
        let ClosureVerdict::Rejected { reasons } = verdict_for_results(&first, &second) else {
            panic!("changed results passed")
        };
        assert!(reasons
            .iter()
            .any(|reason| matches!(reason, ClosureRejection::DifferentResultSets)));
    }

    #[test]
    fn fresh_workspaces_do_not_reuse_existing_family_evidence() {
        let base = std::env::temp_dir().join(format!(
            "lila-closure-control-{}-{}",
            std::process::id(),
            WORKSPACE_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        let first = FreshWorkspace::create(&base).unwrap();
        fs::write(first.first.join("existing.json"), b"old evidence").unwrap();
        let config = SuiteConfig {
            snapshot_dir: first.first.clone(),
            ..SuiteConfig::default()
        };
        let compiler = CompilerProvenance::current().unwrap();
        let pinned = pinned_revisions(&config);
        let error = match run_fresh_family(&config, FIRST_FAMILY, &compiler, &pinned) {
            Ok(_) => panic!("old evidence entered a fresh family"),
            Err(error) => error,
        };
        assert!(error.contains("nonempty family directory"), "{error}");
        let second = FreshWorkspace::create(&base).unwrap();
        assert_ne!(first.root, second.root);
        assert_ne!(first.first, first.second);
        assert_eq!(
            fs::read(first.first.join("existing.json")).unwrap(),
            b"old evidence"
        );
        assert!(fs::read_dir(second.first).unwrap().next().is_none());
        assert!(fs::read_dir(second.second).unwrap().next().is_none());
        fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn fixture_suite_cannot_enter_the_product_release_factory() {
        let config = SuiteConfig {
            suite_root: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/fake_test262/vendor/test262"),
            ..SuiteConfig::default()
        };
        let error = run_closure(config).unwrap_err();
        assert!(error.contains("fixture or alternate suite root"), "{error}");
    }

    #[test]
    fn clean_pin_rejects_repository_root_and_nested_suite_mutation() {
        for prefix in ["", "vendor/test262"] {
            let root = std::env::temp_dir().join(format!(
                "lila-closure-pin-control-{}-{}",
                std::process::id(),
                WORKSPACE_SEQUENCE.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&root).unwrap();
            let suite = root.join(prefix);
            fs::create_dir_all(suite.join("test")).unwrap();
            let source = suite.join("test/a.js");
            fs::write(&source, "0;").unwrap();
            fs::write(suite.join(".gitignore"), "test/ignored.js\n").unwrap();
            let git = |args: &[&str]| {
                let output = Command::new("git")
                    .arg("-C")
                    .arg(&root)
                    .args(args)
                    .output()
                    .unwrap();
                assert!(
                    output.status.success(),
                    "{}",
                    String::from_utf8_lossy(&output.stderr)
                );
            };
            git(&["init", "--quiet"]);
            git(&["add", "--force", "."]);
            git(&[
                "-c",
                "core.hooksPath=/dev/null",
                "-c",
                "commit.gpgsign=false",
                "-c",
                "user.name=Lila closure control",
                "-c",
                "user.email=closure@example.invalid",
                "commit",
                "--quiet",
                "-m",
                "finite pinned suite control",
            ]);
            let clean = current_clean_pin(&suite).unwrap();
            fs::write(&source, "1;").unwrap();
            assert!(current_clean_pin(&suite)
                .unwrap_err()
                .contains("modified, untracked or ignored"));
            fs::write(&source, "0;").unwrap();
            let extra = suite.join("test/untracked.js");
            fs::write(&extra, "2;").unwrap();
            assert!(current_clean_pin(&suite)
                .unwrap_err()
                .contains("modified, untracked or ignored"));
            fs::remove_file(extra).unwrap();
            let ignored = suite.join("test/ignored.js");
            fs::write(&ignored, "3;").unwrap();
            assert!(current_clean_pin(&suite)
                .unwrap_err()
                .contains("modified, untracked or ignored"));
            fs::remove_file(ignored).unwrap();
            assert_eq!(current_clean_pin(&suite).unwrap(), clean);
            fs::remove_dir_all(root.join(".git")).unwrap();
            assert!(current_clean_pin(&suite).is_err());
            fs::remove_dir_all(root).unwrap();
        }
    }
}
