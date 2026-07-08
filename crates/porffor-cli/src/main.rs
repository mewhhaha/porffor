use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use porffor_engine::{
    CompileOptions, Engine, ExecutionBackend, HostHooks, RealmBuilder, RunOptions,
};
use porffor_test262::{
    try_compare_with_js_oracle, ConformanceRunner, FailureKind, FailureOrigin, OutcomeKind,
    RunConfig, SuiteConfig, VerifiedAggregateSummary,
};
use serde::Serialize;
use serde_json::{Map, Value};

#[derive(Debug)]
struct StdoutHostHooks;

impl HostHooks for StdoutHostHooks {
    fn print_line(&self, text: &str) {
        println!("{text}");
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedTest262Args {
    config: SuiteConfig,
    filter: Option<String>,
    run_config: RunConfig,
    readme_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FakeSuiteCounts {
    wasm_safe_total: usize,
    full_total: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct PublishedStatusCount {
    passed: usize,
    total: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct PublishedCountEntry {
    label: String,
    count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct PublishedTargetEntry {
    filter: String,
    passed: usize,
    total: usize,
    failed: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct PublishedSnapshotPaths {
    json: String,
    txt: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct PublishedPinnedRevisions {
    ecma262: String,
    test262: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct PublishedRealSuiteGoal {
    name: String,
    denominator: String,
    target_total: usize,
    current_success: usize,
    remaining_to_green: usize,
    pass_rate: String,
    outcome_targets: Vec<PublishedCountEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct PublishedRealSuiteStatus {
    backend: String,
    refresh_date: String,
    manifest_hash: u64,
    passed: usize,
    total: usize,
    failed: usize,
    pinned_revisions: PublishedPinnedRevisions,
    counts_per_outcome: Vec<PublishedCountEntry>,
    counts_per_kind: Vec<PublishedCountEntry>,
    counts_per_origin: Vec<PublishedCountEntry>,
    top_targets: Vec<PublishedTargetEntry>,
    snapshot_paths: PublishedSnapshotPaths,
    goal: PublishedRealSuiteGoal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct PublishedStatusArtifact {
    fake_wasm_safe: PublishedStatusCount,
    fake_full: PublishedStatusCount,
    real_suite: PublishedRealSuiteStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PublishedStatusPaths {
    json_path: PathBuf,
    txt_path: PathBuf,
}

fn usage() -> &'static str {
    "porf <command> [args]

Commands:
  run [--execution-backend wasm|spec] <file>
                                        compile and run a script through Rust engine path
                                        (default backend: wasm; wasm-aot is the only
                                        supported product execution path)
  repl                                  reserved for the Rust REPL shell
  build wasm <file>                     compile JavaScript directly to Wasm
  build c <file>                        emit C from shared IR
  build native <file>                   emit native artifact from shared IR
  types [entrypoint] [output] [options] generate Worker-style TypeScript types
  typegen [entrypoint] [output] [options]
                                        alias for types
  test262 sync [--suite-root PATH]
  test262 list [filter] [--suite-root PATH]
  test262 run [filter] [options]
  test262 shard <index>/<total> [filter] [options]
  test262 report [filter] [options]
  test262 report-all [options]
  test262 publish-status [options]
  test262 progress-status [options]
  test262 triage-status [options]
  test262 failure-details <matrix-node> [options]
  test262 generate-backlog [options]
  test262 compare-snapshots <base-snapshot-name> [options]
  test262 compare-js-oracle [filter] [--suite-root PATH]
  inspect <file>                        show compile pipeline summary

test262 options:
  --suite-root PATH
  --snapshot-dir PATH
  --threads N
  --timeout-ms N
  --execution-backend wasm|spec         default: wasm (wasm-aot). `spec` selects
                                         spec-exec, an INTERNAL/DEBUG-ONLY interpreter
                                         (Boa) differential oracle for triage/T25 use
                                         only; it is never a product conformance
                                         backend and requires a `spec-exec-oracle`
                                         feature build of porffor-engine/porffor-cli
  --resume
  --snapshot-name NAME
  --max-matrix-nodes N
  --readme-path PATH

types options:
  --config PATH, -c PATH
  --entrypoint PATH
  --env NAME
  --env-interface NAME
  --include-runtime=false
  --include-env=false
  --strict-vars=false
  --check
  --print
  --cwd PATH
"
}

fn main() -> ExitCode {
    match real_main() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::from(1)
        }
    }
}

fn real_main() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let Some(command) = args.next() else {
        print!("{}", usage());
        return Ok(());
    };

    if matches!(command.as_str(), "--help" | "-h" | "help") {
        print!("{}", usage());
        return Ok(());
    }

    let engine = Engine::new(
        RealmBuilder::new()
            .with_host_hooks(Box::new(StdoutHostHooks))
            .build(),
    );
    match command.as_str() {
        "run" => {
            let ParsedRunArgs { backend, path } = parse_run_args(&args.collect::<Vec<_>>())?;
            let path = path.ok_or_else(|| "run needs a source file".to_string())?;
            let source = read_source(&path)?;
            engine
                .run_script(
                    &source,
                    CompileOptions {
                        filename: Some(path),
                        ..CompileOptions::default()
                    },
                    RunOptions {
                        backend,
                        ..RunOptions::default()
                    },
                )
                .map(|outcome| println!("run outcome: {:?}", outcome))
                .map_err(|err| err.to_string())
        }
        "repl" => Err("Rust REPL shell not implemented yet".to_string()),
        "build" => {
            let format = args
                .next()
                .ok_or_else(|| "build needs target: wasm, c, or native".to_string())?;
            let path = args
                .next()
                .ok_or_else(|| "build needs a source file".to_string())?;
            let source = read_source(&path)?;
            let unit = engine
                .compile_script(
                    &source,
                    CompileOptions {
                        filename: Some(path.clone()),
                        ..CompileOptions::default()
                    },
                )
                .map_err(|err| err.to_string())?;
            match format.as_str() {
                "wasm" => engine
                    .emit_wasm(&unit)
                    .map(|artifact| {
                        if let Some(path) = std::env::var_os("PORFFOR_WASM_DUMP") {
                            fs::write(&path, &artifact.bytes).unwrap_or_else(|err| {
                                panic!("failed to write PORFFOR_WASM_DUMP artifact: {err}");
                            });
                        }
                        if std::env::var_os("PORFFOR_WASM_TRACE").is_some() {
                            eprintln!(
                                "porffor wasm trace: artifact bytes: {}",
                                artifact.bytes.len()
                            );
                        }
                        println!(
                            "built {:?} artifact: {}",
                            artifact.kind, artifact.description
                        )
                    })
                    .map_err(|err| err.to_string()),
                "c" => engine
                    .emit_c(&unit)
                    .map(|artifact| {
                        println!(
                            "built {:?} artifact: {}",
                            artifact.kind, artifact.description
                        )
                    })
                    .map_err(|err| err.to_string()),
                "native" => engine
                    .emit_native(&unit, None)
                    .map(|artifact| {
                        println!(
                            "built {:?} artifact: {}",
                            artifact.kind, artifact.description
                        )
                    })
                    .map_err(|err| err.to_string()),
                _ => Err(format!("unknown build target: {format}")),
            }
        }
        "types" | "typegen" => handle_types_command(args.collect()),
        "test262" => handle_test262_command(args.collect()),
        "inspect" => {
            let path = args
                .next()
                .ok_or_else(|| "inspect needs a source file".to_string())?;
            let source = read_source(&path)?;
            let goal = if is_module_path(&path) {
                "module"
            } else {
                "script"
            };
            let unit = if goal == "module" {
                engine.compile_module(
                    &source,
                    CompileOptions {
                        filename: Some(path),
                        ..CompileOptions::default()
                    },
                )
            } else {
                engine.compile_script(
                    &source,
                    CompileOptions {
                        filename: Some(path),
                        ..CompileOptions::default()
                    },
                )
            }
            .map_err(|err| err.to_string())?;
            let report = engine.inspect(&unit);
            println!("goal: {:?}", report.goal);
            println!("source_len: {}", report.source_len);
            println!("stages: {}", report.stages.join(", "));
            println!("invariants: {}", report.invariants.join(", "));
            println!("ir: {}", report.ir_summary);
            if !report.diagnostics.is_empty() {
                println!("diagnostics:");
                for diagnostic in report.diagnostics {
                    println!("  {diagnostic}");
                }
            }
            Ok(())
        }
        _ => Err(format!("unknown command: {command}\n\n{}", usage())),
    }
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn default_readme_path() -> PathBuf {
    repo_root().join("README.md")
}

fn fake_suite_config() -> SuiteConfig {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("porffor-test262")
        .join("tests")
        .join("fixtures")
        .join("fake_test262");
    SuiteConfig {
        suite_root: root.join("vendor").join("test262"),
        local_harness_path: root.join("harness.js"),
        snapshot_dir: root.join("snapshots"),
        case_runner_bin: None,
        ..SuiteConfig::default()
    }
}

fn fake_suite_counts() -> Result<FakeSuiteCounts, String> {
    let runner = ConformanceRunner::with_config(fake_suite_config());
    let full_total = runner.discover_suite(None)?.cases.len();
    let wasm_safe_total = runner
        .discover_suite(Some("language/wasm/pass"))?
        .cases
        .len();
    Ok(FakeSuiteCounts {
        wasm_safe_total,
        full_total,
    })
}

fn current_utc_date_string() -> Result<String, String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| format!("system clock before unix epoch: {err}"))?;
    let days = (now.as_secs() / 86_400) as i64;
    let (year, month, day) = civil_from_days(days);
    Ok(format!("{year:04}-{month:02}-{day:02}"))
}

fn civil_from_days(days_since_unix_epoch: i64) -> (i32, u32, u32) {
    let z = days_since_unix_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if m <= 2 { 1 } else { 0 };
    (year as i32, m as u32, d as u32)
}

fn sorted_kind_counts(counts: &BTreeMap<FailureKind, usize>) -> Vec<PublishedCountEntry> {
    let mut entries = FailureKind::ALL
        .into_iter()
        .map(|kind| PublishedCountEntry {
            label: kind.as_str().to_string(),
            count: counts.get(&kind).copied().unwrap_or(0),
        })
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| left.label.cmp(&right.label))
    });
    entries
}

fn sorted_outcome_counts(counts: &BTreeMap<OutcomeKind, usize>) -> Vec<PublishedCountEntry> {
    OutcomeKind::ALL
        .into_iter()
        .map(|outcome| PublishedCountEntry {
            label: outcome.as_str().to_string(),
            count: counts.get(&outcome).copied().unwrap_or(0),
        })
        .collect()
}

fn sorted_origin_counts(counts: &BTreeMap<FailureOrigin, usize>) -> Vec<PublishedCountEntry> {
    let mut entries = FailureOrigin::ALL
        .into_iter()
        .map(|origin| PublishedCountEntry {
            label: origin.as_str().to_string(),
            count: counts.get(&origin).copied().unwrap_or(0),
        })
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| left.label.cmp(&right.label))
    });
    entries
}

fn top_target_entries(summary: &VerifiedAggregateSummary) -> Vec<PublishedTargetEntry> {
    let mut entries = summary
        .summary
        .entries
        .iter()
        .filter(|entry| entry.failed > 0)
        .map(|entry| PublishedTargetEntry {
            filter: entry.filter.clone(),
            passed: entry.passed,
            total: entry.total,
            failed: entry.failed,
        })
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| {
        right
            .failed
            .cmp(&left.failed)
            .then_with(|| left.filter.cmp(&right.filter))
    });
    entries.truncate(5);
    entries
}

fn build_published_status_artifact(
    fake_counts: &FakeSuiteCounts,
    summary: &VerifiedAggregateSummary,
    execution_backend: ExecutionBackend,
    refresh_date: &str,
) -> PublishedStatusArtifact {
    let success = summary
        .summary
        .counts_per_outcome
        .get(&OutcomeKind::Success)
        .copied()
        .unwrap_or(0);
    let not_implemented = summary
        .summary
        .counts_per_outcome
        .get(&OutcomeKind::NotImplemented)
        .copied()
        .unwrap_or(0);
    let crash = summary
        .summary
        .counts_per_outcome
        .get(&OutcomeKind::Crash)
        .copied()
        .unwrap_or(0);
    let bug = summary
        .summary
        .counts_per_outcome
        .get(&OutcomeKind::Bug)
        .copied()
        .unwrap_or(0);
    PublishedStatusArtifact {
        fake_wasm_safe: PublishedStatusCount {
            passed: fake_counts.wasm_safe_total,
            total: fake_counts.wasm_safe_total,
        },
        fake_full: PublishedStatusCount {
            passed: fake_counts.full_total,
            total: fake_counts.full_total,
        },
        real_suite: PublishedRealSuiteStatus {
            backend: execution_backend.as_str().to_string(),
            refresh_date: refresh_date.to_string(),
            manifest_hash: summary.manifest_hash,
            passed: summary.summary.passed,
            total: summary.summary.total,
            failed: summary.summary.failed,
            pinned_revisions: PublishedPinnedRevisions {
                ecma262: summary.pinned_revisions.ecma262.clone(),
                test262: summary.pinned_revisions.test262.clone(),
            },
            counts_per_outcome: sorted_outcome_counts(&summary.summary.counts_per_outcome),
            counts_per_kind: sorted_kind_counts(&summary.summary.counts_per_kind),
            counts_per_origin: sorted_origin_counts(&summary.summary.counts_per_origin),
            top_targets: top_target_entries(summary),
            snapshot_paths: PublishedSnapshotPaths {
                json: summary.snapshot_paths.json_path.display().to_string(),
                txt: summary.snapshot_paths.txt_path.display().to_string(),
            },
            goal: PublishedRealSuiteGoal {
                name: "Full pinned Test262 green".to_string(),
                denominator: "pinned-suite-total".to_string(),
                target_total: summary.summary.total,
                current_success: success,
                remaining_to_green: not_implemented + crash + bug,
                pass_rate: percent_string(success, summary.summary.total),
                outcome_targets: vec![
                    PublishedCountEntry {
                        label: OutcomeKind::NotImplemented.as_str().to_string(),
                        count: 0,
                    },
                    PublishedCountEntry {
                        label: OutcomeKind::Crash.as_str().to_string(),
                        count: 0,
                    },
                    PublishedCountEntry {
                        label: OutcomeKind::Bug.as_str().to_string(),
                        count: 0,
                    },
                ],
            },
        },
    }
}

fn published_status_paths(
    snapshot_dir: &Path,
    execution_backend: ExecutionBackend,
) -> PublishedStatusPaths {
    let stem = format!("published-status-{}", execution_backend.as_str());
    PublishedStatusPaths {
        json_path: snapshot_dir.join(format!("{stem}.json")),
        txt_path: snapshot_dir.join(format!("{stem}.txt")),
    }
}

fn render_published_status_text(artifact: &PublishedStatusArtifact) -> String {
    let mut out = String::new();
    let real = &artifact.real_suite;
    out.push_str("published real Test262 status\n");
    out.push_str(&format!("refresh_date={}\n", real.refresh_date));
    out.push_str(&format!("execution_backend={}\n", real.backend));
    out.push_str(&format!("manifest_hash={}\n", real.manifest_hash));
    out.push_str(&format!(
        "pinned: ecma262={} test262={}\n",
        real.pinned_revisions.ecma262, real.pinned_revisions.test262
    ));
    out.push_str(&format!(
        "fake_wasm_safe={}/{}\n",
        artifact.fake_wasm_safe.passed, artifact.fake_wasm_safe.total
    ));
    out.push_str(&format!(
        "fake_full={}/{}\n",
        artifact.fake_full.passed, artifact.fake_full.total
    ));
    out.push_str(&format!("real_total={}\n", real.total));
    out.push_str(&format!("real_passed={}\n", real.passed));
    out.push_str(&format!("real_failed={}\n", real.failed));
    out.push_str(&format!("goal={}\n", real.goal.name));
    out.push_str(&format!(
        "progress={}/{}\n",
        real.goal.current_success, real.goal.target_total
    ));
    out.push_str(&format!(
        "remaining_to_green={}\n",
        real.goal.remaining_to_green
    ));
    out.push_str(&format!(
        "burn_down: NotImplemented={} Crash={} Bug={}\n",
        outcome_count(&real.counts_per_outcome, OutcomeKind::NotImplemented),
        outcome_count(&real.counts_per_outcome, OutcomeKind::Crash),
        outcome_count(&real.counts_per_outcome, OutcomeKind::Bug)
    ));
    out.push_str(&format!("snapshot_json={}\n", real.snapshot_paths.json));
    out.push_str(&format!("snapshot_txt={}\n", real.snapshot_paths.txt));
    out.push_str("outcomes:\n");
    for entry in &real.counts_per_outcome {
        out.push_str(&format!("  {}={}\n", entry.label, entry.count));
    }
    out.push_str("failure_kinds:\n");
    for entry in &real.counts_per_kind {
        out.push_str(&format!("  {}={}\n", entry.label, entry.count));
    }
    out.push_str("failure_origins:\n");
    for entry in &real.counts_per_origin {
        out.push_str(&format!("  {}={}\n", entry.label, entry.count));
    }
    out.push_str("top_targets:\n");
    if real.top_targets.is_empty() {
        out.push_str("  none\n");
    } else {
        for entry in &real.top_targets {
            out.push_str(&format!(
                "  {}: {}/{} passed (failed {})\n",
                entry.filter, entry.passed, entry.total, entry.failed
            ));
        }
    }
    out
}

fn write_published_status_artifact(
    snapshot_dir: &Path,
    execution_backend: ExecutionBackend,
    artifact: &PublishedStatusArtifact,
) -> Result<PublishedStatusPaths, String> {
    fs::create_dir_all(snapshot_dir).map_err(|err| {
        format!(
            "failed to create snapshot dir {}: {err}",
            snapshot_dir.display()
        )
    })?;
    let paths = published_status_paths(snapshot_dir, execution_backend);
    fs::write(
        &paths.json_path,
        serde_json::to_string_pretty(artifact)
            .map_err(|err| format!("failed to encode published status json: {err}"))?,
    )
    .map_err(|err| format!("failed to write {}: {err}", paths.json_path.display()))?;
    fs::write(&paths.txt_path, render_published_status_text(artifact))
        .map_err(|err| format!("failed to write {}: {err}", paths.txt_path.display()))?;
    Ok(paths)
}

fn percent_string(passed: usize, total: usize) -> String {
    if total == 0 {
        return "0.0%".to_string();
    }
    format!("{:.1}%", (passed as f64 * 100.0) / total as f64)
}

fn outcome_count(entries: &[PublishedCountEntry], outcome: OutcomeKind) -> usize {
    entries
        .iter()
        .find(|entry| entry.label == outcome.as_str())
        .map(|entry| entry.count)
        .unwrap_or(0)
}

fn top_nonzero_labels(entries: &[PublishedCountEntry], limit: usize) -> String {
    let labels = entries
        .iter()
        .filter(|entry| entry.count > 0)
        .take(limit)
        .map(|entry| format!("`{}={}`", entry.label, entry.count))
        .collect::<Vec<_>>();
    if labels.is_empty() {
        "none".to_string()
    } else {
        labels.join(", ")
    }
}

fn all_count_labels(entries: &[PublishedCountEntry]) -> String {
    entries
        .iter()
        .map(|entry| format!("`{}={}`", entry.label, entry.count))
        .collect::<Vec<_>>()
        .join(", ")
}

fn top_target_labels(entries: &[PublishedTargetEntry], limit: usize) -> String {
    let labels = entries
        .iter()
        .take(limit)
        .map(|entry| {
            format!(
                "`{}: {}/{} passed`",
                entry.filter, entry.passed, entry.total
            )
        })
        .collect::<Vec<_>>();
    if labels.is_empty() {
        "none".to_string()
    } else {
        labels.join(", ")
    }
}

fn render_current_status_block(artifact: &PublishedStatusArtifact) -> String {
    let real = &artifact.real_suite;
    let real_status = if real.passed == real.total {
        "green"
    } else {
        "not green"
    };
    format!(
        "## Current Status\n<!-- porffor-status:start -->\nRust rewrite status must be read in layers, not one vanity number:\n- Fake wasm-safe Test262 subset: `{}/{}` green\n- Fake full Rust rewrite suite: `{}/{}` green\n- Pinned real Test262 baseline (`{}`, refreshed `{}`): `{}/{}` {} (`{}`)\n- Real Test262 goal: Success={}/{} ({}); burn down NotImplemented={}, Crash={}, Bug={} to zero\n- Pinned revisions: `ecma262={}` `test262={}`\n- Current real outcomes: {}\n- Biggest current real failing kinds: {}\n- Biggest current real failing origins: {}\n- Worst current real matrix targets: {}\n- Published status artifacts: `{}` and `{}`\n\nAs of `{}`, Rust Wasm-AOT path is at 100% of repo fake coverage, not 100% ECMAScript. Project is still off literal 100% until full pinned real Test262 run is green for Rust path.\n\nStatus refresh commands:\n- `cargo test -p porffor-engine --quiet`\n- `cargo test -p porffor-cli --quiet`\n- `./target/debug/porf test262 run language/wasm/pass --suite-root crates/porffor-test262/tests/fixtures/fake_test262/vendor/test262 --execution-backend wasm`\n- `./target/debug/porf test262 run --suite-root crates/porffor-test262/tests/fixtures/fake_test262/vendor/test262`\n- `./scripts/publish-real-status-low-ram.sh {} codex-published-real`\n\nWhen counts move, update this block in same change. Do not claim full Test262 `100%` from fake-suite numbers.\n<!-- porffor-status:end -->",
        artifact.fake_wasm_safe.passed,
        artifact.fake_wasm_safe.total,
        artifact.fake_full.passed,
        artifact.fake_full.total,
        real.backend,
        real.refresh_date,
        real.passed,
        real.total,
        real_status,
        percent_string(real.passed, real.total),
        real.goal.current_success,
        real.goal.target_total,
        real.goal.pass_rate,
        outcome_count(&real.counts_per_outcome, OutcomeKind::NotImplemented),
        outcome_count(&real.counts_per_outcome, OutcomeKind::Crash),
        outcome_count(&real.counts_per_outcome, OutcomeKind::Bug),
        real.pinned_revisions.ecma262,
        real.pinned_revisions.test262,
        all_count_labels(&real.counts_per_outcome),
        top_nonzero_labels(&real.counts_per_kind, 3),
        top_nonzero_labels(&real.counts_per_origin, 3),
        top_target_labels(&real.top_targets, 3),
        real.snapshot_paths.json,
        real.snapshot_paths.txt,
        real.refresh_date,
        real.backend,
    )
}

fn rewrite_current_status_block(
    readme_path: &Path,
    artifact: &PublishedStatusArtifact,
) -> Result<(), String> {
    let raw = fs::read_to_string(readme_path)
        .map_err(|err| format!("failed to read {}: {err}", readme_path.display()))?;
    let replacement = render_current_status_block(artifact);
    let updated = if let (Some(start), Some(end)) = (
        raw.find("<!-- porffor-status:start -->"),
        raw.find("<!-- porffor-status:end -->"),
    ) {
        let section_start = raw[..start].rfind("## Current Status").ok_or_else(|| {
            format!(
                "missing `## Current Status` before status marker in {}",
                readme_path.display()
            )
        })?;
        let after_end = end + "<!-- porffor-status:end -->".len();
        format!(
            "{}{}{}",
            &raw[..section_start],
            replacement,
            &raw[after_end..]
        )
    } else {
        let section_start = raw
            .find("## Current Status")
            .ok_or_else(|| format!("missing `## Current Status` in {}", readme_path.display()))?;
        let section_end = raw[section_start + "## Current Status".len()..]
            .find("\n## ")
            .map(|offset| section_start + "## Current Status".len() + offset)
            .unwrap_or(raw.len());
        format!(
            "{}{}{}",
            &raw[..section_start],
            replacement,
            &raw[section_end..]
        )
    };
    fs::write(readme_path, updated)
        .map_err(|err| format!("failed to write {}: {err}", readme_path.display()))
}

const DEFAULT_TYPE_OUTPUT: &str = "worker-configuration.d.ts";
const DEFAULT_CONFIG_NAMES: &[&str] = &[
    "wrangler.jsonc",
    "wrangler.json",
    "wrangler.toml",
    "porffor.jsonc",
    "porffor.json",
    "porffor.toml",
];
const TYPEGEN_BINDING_SPECS: &[(&[&str], &str)] = &[
    (&["kv_namespaces"], "KVNamespace"),
    (&["r2_buckets"], "R2Bucket"),
    (&["d1_databases"], "D1Database"),
    (&["durable_objects", "bindings"], "DurableObjectNamespace"),
    (&["services"], "Fetcher"),
    (&["queues", "producers"], "Queue"),
    (&["analytics_engine_datasets"], "AnalyticsEngineDataset"),
    (&["vectorize"], "VectorizeIndex"),
    (&["ai_search"], "AiSearch"),
    (&["ai_search_namespaces"], "AiSearchNamespace"),
    (&["mtls_certificates"], "Fetcher"),
    (&["browser"], "BrowserRendering"),
    (&["images"], "ImagesBinding"),
    (&["hyperdrive"], "Hyperdrive"),
    (&["workflows"], "Workflow"),
    (&["pipelines"], "Pipeline"),
    (&["dispatch_namespaces"], "DispatchNamespace"),
    (&["send_email"], "SendEmail"),
];
const TYPEGEN_SINGLETON_BINDING_SPECS: &[(&[&str], &str)] = &[
    (&["ai"], "Ai"),
    (&["version_metadata"], "WorkerVersionMetadata"),
    (&["assets"], "Fetcher"),
];
const TYPEGEN_KNOWN_HANDLERS: &[&str] = &[
    "fetch",
    "scheduled",
    "queue",
    "email",
    "tail",
    "trace",
    "alarm",
];

#[derive(Debug, Clone, PartialEq, Eq)]
struct TypegenOptions {
    config_paths: Vec<String>,
    cwd: PathBuf,
    env: Option<String>,
    env_interface: String,
    include_runtime: bool,
    include_env: bool,
    strict_vars: bool,
    check: bool,
    print: bool,
    entrypoint: Option<String>,
    entrypoint_explicit: bool,
    output: String,
    help: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TypegenEntrypointInfo {
    path: String,
    syntax: String,
    handlers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TypegenBinding {
    name: String,
    types: BTreeSet<String>,
    optional: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TypegenInputs {
    configs: Vec<Value>,
    config_paths: Vec<PathBuf>,
    entrypoint_info: Option<TypegenEntrypointInfo>,
    bindings: Vec<TypegenBinding>,
}

fn typegen_usage() -> &'static str {
    "Usage: porf types [entrypoint] [worker-configuration.d.ts] [options]

Generate Wrangler-style TypeScript runtime and Env declarations from a worker config.

Options:
  --config, -c <path>          Path to wrangler/porffor config; can be repeated
  --entrypoint <path>          Worker entrypoint when not set by config main
  --env, -e <name>             Generate only one named environment
  --env-interface <name>       Global env interface name (default: Env)
  --include-runtime=<bool>     Include minimal runtime declarations (default: true)
  --include-env=<bool>         Include env declarations (default: true)
  --strict-vars=<bool>         Preserve literal var types (default: true)
  --check                      Exit non-zero if the output file is stale
  --print                      Print generated declarations instead of writing
  --cwd <path>                 Resolve config and output paths from this directory
"
}

fn handle_types_command(args: Vec<String>) -> Result<(), String> {
    let options = parse_typegen_args(&args)?;
    if options.help {
        print!("{}", typegen_usage());
        return Ok(());
    }

    let inputs = load_typegen_inputs(&options)?;
    let output = render_typegen_declarations(&inputs, &options);
    let output_path = resolve_from(&options.cwd, &options.output);

    if options.print {
        print!("{output}");
        return Ok(());
    }

    if options.check {
        let current = fs::read_to_string(&output_path).unwrap_or_default();
        if current != output {
            return Err(format!("{} is out of date; run porf types", options.output));
        }
        println!("Types are up to date at {}", options.output);
        return Ok(());
    }

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    fs::write(&output_path, output)
        .map_err(|err| format!("failed to write {}: {err}", output_path.display()))?;
    println!("Types written to {}", options.output);
    Ok(())
}

fn parse_typegen_args(args: &[String]) -> Result<TypegenOptions, String> {
    let mut options = TypegenOptions {
        config_paths: Vec::new(),
        cwd: std::env::current_dir().map_err(|err| format!("failed to read cwd: {err}"))?,
        env: None,
        env_interface: "Env".to_string(),
        include_runtime: true,
        include_env: true,
        strict_vars: true,
        check: false,
        print: false,
        entrypoint: None,
        entrypoint_explicit: false,
        output: String::new(),
        help: false,
    };
    let mut positionals = Vec::new();

    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        if !is_typegen_flag(arg) {
            positionals.push(arg.clone());
            index += 1;
            continue;
        }

        let (flag_name, inline_value) = split_flag_value(arg);
        match flag_name {
            "--help" | "-h" => options.help = true,
            "--config" | "-c" => {
                let (value, next) = read_typegen_option_value(args, index, arg, flag_name)?;
                options.config_paths.push(value);
                index = next;
            }
            "--cwd" => {
                let (value, next) = read_typegen_option_value(args, index, arg, flag_name)?;
                options.cwd = PathBuf::from(value);
                index = next;
            }
            "--env" | "-e" => {
                let (value, next) = read_typegen_option_value(args, index, arg, flag_name)?;
                options.env = Some(value);
                index = next;
            }
            "--env-interface" => {
                let (value, next) = read_typegen_option_value(args, index, arg, flag_name)?;
                if !is_ts_identifier(&value) {
                    return Err(format!("Invalid TypeScript interface name: {value}"));
                }
                options.env_interface = value;
                index = next;
            }
            "--entrypoint" | "--entry" | "--main" => {
                let (value, next) = read_typegen_option_value(args, index, arg, flag_name)?;
                options.entrypoint = Some(value);
                options.entrypoint_explicit = true;
                index = next;
            }
            "--out" | "--output" | "-o" => {
                let (value, next) = read_typegen_option_value(args, index, arg, flag_name)?;
                options.output = value;
                index = next;
            }
            "--include-runtime" => {
                options.include_runtime = parse_typegen_bool(inline_value, true)?;
            }
            "--no-include-runtime" => options.include_runtime = false,
            "--include-env" => {
                options.include_env = parse_typegen_bool(inline_value, true)?;
            }
            "--no-include-env" => options.include_env = false,
            "--strict-vars" => {
                options.strict_vars = parse_typegen_bool(inline_value, true)?;
            }
            "--no-strict-vars" => options.strict_vars = false,
            "--check" => {
                options.check = parse_typegen_bool(inline_value, true)?;
            }
            "--print" => {
                options.print = parse_typegen_bool(inline_value, true)?;
            }
            _ => return Err(format!("Unknown types option: {flag_name}")),
        }
        index += 1;
    }

    for positional in positionals {
        if positional.ends_with(".d.ts") {
            if !options.output.is_empty() {
                return Err(format!(
                    "Multiple output paths were provided: {}, {}",
                    options.output, positional
                ));
            }
            options.output = positional;
            continue;
        }

        if options.entrypoint.is_none() {
            options.entrypoint = Some(positional);
            options.entrypoint_explicit = true;
            continue;
        }

        if options.output.is_empty() {
            options.output = positional;
            continue;
        }

        return Err(format!("Unexpected positional argument: {positional}"));
    }

    if options.output.is_empty() {
        options.output = DEFAULT_TYPE_OUTPUT.to_string();
    }
    if !options.output.ends_with(".d.ts") {
        return Err(format!(
            "Type output path must end in .d.ts: {}",
            options.output
        ));
    }

    Ok(options)
}

fn is_typegen_flag(value: &str) -> bool {
    value.starts_with('-')
}

fn split_flag_value(arg: &str) -> (&str, Option<&str>) {
    arg.split_once('=')
        .map(|(flag, value)| (flag, Some(value)))
        .unwrap_or((arg, None))
}

fn read_typegen_option_value(
    args: &[String],
    index: usize,
    raw_arg: &str,
    flag_name: &str,
) -> Result<(String, usize), String> {
    if let Some((_, value)) = raw_arg.split_once('=') {
        return Ok((value.to_string(), index));
    }
    let next_index = index + 1;
    let value = args
        .get(next_index)
        .ok_or_else(|| format!("{flag_name} expects a value"))?;
    if is_typegen_flag(value) {
        return Err(format!("{flag_name} expects a value"));
    }
    Ok((value.clone(), next_index))
}

fn parse_typegen_bool(value: Option<&str>, fallback: bool) -> Result<bool, String> {
    let Some(value) = value else {
        return Ok(fallback);
    };
    match value.to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => Err(format!("Expected a boolean value, got {value}")),
    }
}

fn resolve_from(base: &Path, value: &str) -> PathBuf {
    let path = PathBuf::from(value);
    if path.is_absolute() {
        path
    } else {
        base.join(path)
    }
}

fn discover_typegen_config(cwd: &Path) -> Vec<PathBuf> {
    DEFAULT_CONFIG_NAMES
        .iter()
        .map(|name| cwd.join(name))
        .find(|candidate| candidate.exists())
        .into_iter()
        .collect()
}

fn load_typegen_inputs(options: &TypegenOptions) -> Result<TypegenInputs, String> {
    let config_paths = if options.config_paths.is_empty() {
        discover_typegen_config(&options.cwd)
    } else {
        options
            .config_paths
            .iter()
            .map(|config_path| resolve_from(&options.cwd, config_path))
            .collect()
    };
    let configs = config_paths
        .iter()
        .map(|config_path| read_typegen_config(config_path))
        .collect::<Result<Vec<_>, _>>()?;
    let config_dir = config_paths
        .first()
        .and_then(|path| path.parent())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| options.cwd.clone());
    let config_main = configs
        .iter()
        .find_map(|config| config.get("main").and_then(Value::as_str))
        .map(ToOwned::to_owned);
    let entrypoint = options.entrypoint.clone().or(config_main);
    let entrypoint_base = if options.entrypoint.is_some() {
        &options.cwd
    } else {
        &config_dir
    };
    let entrypoint_info = read_typegen_entrypoint(
        entrypoint.as_deref(),
        entrypoint_base,
        options.entrypoint_explicit,
    )?;
    let bindings = collect_typegen_bindings(&configs, options);

    Ok(TypegenInputs {
        configs,
        config_paths,
        entrypoint_info,
        bindings,
    })
}

fn read_typegen_config(path: &Path) -> Result<Value, String> {
    let source = fs::read_to_string(path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    if path.extension().and_then(|ext| ext.to_str()) == Some("toml") {
        parse_typegen_toml(&source, path)
    } else {
        parse_typegen_json_like(&source, path)
    }
}

fn parse_typegen_json_like(source: &str, path: &Path) -> Result<Value, String> {
    serde_json::from_str(&strip_trailing_json_commas(&strip_json_comments(source)))
        .map_err(|err| format!("Failed to parse {}: {err}", path.display()))
}

fn strip_json_comments(source: &str) -> String {
    let chars = source.chars().collect::<Vec<_>>();
    let mut out = String::new();
    let mut in_string = false;
    let mut quote = '\0';
    let mut escaped = false;
    let mut index = 0;
    while index < chars.len() {
        let ch = chars[index];
        let next = chars.get(index + 1).copied();

        if in_string {
            out.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == quote {
                in_string = false;
            }
            index += 1;
            continue;
        }

        if ch == '"' || ch == '\'' {
            in_string = true;
            quote = ch;
            out.push(ch);
            index += 1;
            continue;
        }

        if ch == '/' && next == Some('/') {
            while index < chars.len() && chars[index] != '\n' {
                index += 1;
            }
            out.push('\n');
            continue;
        }

        if ch == '/' && next == Some('*') {
            index += 2;
            while index + 1 < chars.len() && !(chars[index] == '*' && chars[index + 1] == '/') {
                index += 1;
            }
            index = (index + 2).min(chars.len());
            continue;
        }

        out.push(ch);
        index += 1;
    }
    out
}

fn strip_trailing_json_commas(source: &str) -> String {
    let chars = source.chars().collect::<Vec<_>>();
    let mut out = String::new();
    let mut in_string = false;
    let mut quote = '\0';
    let mut escaped = false;
    let mut index = 0;
    while index < chars.len() {
        let ch = chars[index];
        if in_string {
            out.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == quote {
                in_string = false;
            }
            index += 1;
            continue;
        }

        if ch == '"' || ch == '\'' {
            in_string = true;
            quote = ch;
            out.push(ch);
            index += 1;
            continue;
        }

        if ch == ',' {
            let mut next_index = index + 1;
            while chars
                .get(next_index)
                .copied()
                .is_some_and(char::is_whitespace)
            {
                next_index += 1;
            }
            if matches!(chars.get(next_index), Some('}' | ']')) {
                index += 1;
                continue;
            }
        }

        out.push(ch);
        index += 1;
    }
    out
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TomlTarget {
    ObjectPath(Vec<String>),
    ArrayLast(Vec<String>),
}

fn parse_typegen_toml(source: &str, path: &Path) -> Result<Value, String> {
    let mut root = Value::Object(Map::new());
    let mut current = TomlTarget::ObjectPath(Vec::new());

    for raw_line in source.lines() {
        let line = strip_toml_comment(raw_line).trim().to_string();
        if line.is_empty() {
            continue;
        }

        if line.starts_with("[[") && line.ends_with("]]") {
            let parts = split_toml_path(line[2..line.len() - 2].trim());
            push_toml_array_table(&mut root, &parts)
                .map_err(|err| format!("Failed to parse {}: {err}", path.display()))?;
            current = TomlTarget::ArrayLast(parts);
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            let parts = split_toml_path(line[1..line.len() - 1].trim());
            ensure_value_object_path(&mut root, &parts)
                .map_err(|err| format!("Failed to parse {}: {err}", path.display()))?;
            current = TomlTarget::ObjectPath(parts);
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            return Err(format!(
                "Failed to parse {}: Invalid TOML line: {line}",
                path.display()
            ));
        };
        let key_parts = split_toml_path(key.trim());
        let value = parse_toml_value(value.trim())
            .map_err(|err| format!("Failed to parse {}: {err}", path.display()))?;
        set_toml_current_value(&mut root, &current, &key_parts, value)
            .map_err(|err| format!("Failed to parse {}: {err}", path.display()))?;
    }

    Ok(root)
}

fn strip_toml_comment(line: &str) -> String {
    let mut out = String::new();
    let mut in_string = false;
    let mut quote = '\0';
    let mut escaped = false;
    for ch in line.chars() {
        if in_string {
            out.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == quote {
                in_string = false;
            }
            continue;
        }
        if ch == '"' || ch == '\'' {
            in_string = true;
            quote = ch;
            out.push(ch);
            continue;
        }
        if ch == '#' {
            break;
        }
        out.push(ch);
    }
    out
}

fn split_toml(value: &str, delimiter: char) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut depth = 0i32;
    let mut in_string = false;
    let mut quote = '\0';
    let mut escaped = false;

    for ch in value.chars() {
        if in_string {
            current.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == quote {
                in_string = false;
            }
            continue;
        }

        if ch == '"' || ch == '\'' {
            in_string = true;
            quote = ch;
            current.push(ch);
            continue;
        }

        if ch == '[' || ch == '{' {
            depth += 1;
        }
        if ch == ']' || ch == '}' {
            depth -= 1;
        }

        if ch == delimiter && depth == 0 {
            parts.push(current.trim().to_string());
            current.clear();
        } else {
            current.push(ch);
        }
    }

    if !current.trim().is_empty() || value.ends_with(delimiter) {
        parts.push(current.trim().to_string());
    }
    parts
}

fn split_toml_path(value: &str) -> Vec<String> {
    split_toml(value, '.')
        .into_iter()
        .map(|part| part.trim().trim_matches('"').trim_matches('\'').to_string())
        .collect()
}

fn parse_toml_value(raw: &str) -> Result<Value, String> {
    let value = raw.trim();
    if (value.starts_with('"') && value.ends_with('"'))
        || (value.starts_with('\'') && value.ends_with('\''))
    {
        if value.starts_with('"') {
            return serde_json::from_str(value)
                .map_err(|err| format!("Invalid TOML string {value}: {err}"));
        }
        return Ok(Value::String(value[1..value.len() - 1].to_string()));
    }
    if value == "true" {
        return Ok(Value::Bool(true));
    }
    if value == "false" {
        return Ok(Value::Bool(false));
    }
    if let Ok(number) = value.parse::<i64>() {
        return Ok(Value::Number(number.into()));
    }
    if let Ok(number) = value.parse::<f64>() {
        if let Some(number) = serde_json::Number::from_f64(number) {
            return Ok(Value::Number(number));
        }
    }
    if value.starts_with('[') && value.ends_with(']') {
        let inner = value[1..value.len() - 1].trim();
        if inner.is_empty() {
            return Ok(Value::Array(Vec::new()));
        }
        return split_toml(inner, ',')
            .into_iter()
            .map(|part| parse_toml_value(&part))
            .collect::<Result<Vec<_>, _>>()
            .map(Value::Array);
    }
    if value.starts_with('{') && value.ends_with('}') {
        let mut out = Value::Object(Map::new());
        let inner = value[1..value.len() - 1].trim();
        if inner.is_empty() {
            return Ok(out);
        }
        for part in split_toml(inner, ',') {
            let Some((key, item_value)) = part.split_once('=') else {
                return Err(format!("Invalid inline TOML table entry: {part}"));
            };
            let key_parts = split_toml_path(key.trim());
            let item_value = parse_toml_value(item_value.trim())?;
            set_nested_value(&mut out, &key_parts, item_value)?;
        }
        return Ok(out);
    }
    Ok(Value::String(value.to_string()))
}

fn ensure_value_object_path<'a>(
    root: &'a mut Value,
    parts: &[String],
) -> Result<&'a mut Map<String, Value>, String> {
    if !root.is_object() {
        *root = Value::Object(Map::new());
    }

    let mut current = root;
    for part in parts {
        if !current.is_object() {
            *current = Value::Object(Map::new());
        }
        let object = current.as_object_mut().expect("value should be object");
        current = object
            .entry(part.clone())
            .or_insert_with(|| Value::Object(Map::new()));
    }

    if !current.is_object() {
        *current = Value::Object(Map::new());
    }
    current
        .as_object_mut()
        .ok_or_else(|| "expected TOML object".to_string())
}

fn set_nested_value(root: &mut Value, parts: &[String], value: Value) -> Result<(), String> {
    let Some((last, parent_parts)) = parts.split_last() else {
        return Err("empty TOML key".to_string());
    };
    let parent = ensure_value_object_path(root, parent_parts)?;
    parent.insert(last.clone(), value);
    Ok(())
}

fn push_toml_array_table(root: &mut Value, parts: &[String]) -> Result<(), String> {
    let Some((last, parent_parts)) = parts.split_last() else {
        return Err("empty TOML array table".to_string());
    };
    let parent = ensure_value_object_path(root, parent_parts)?;
    let entry = parent
        .entry(last.clone())
        .or_insert_with(|| Value::Array(Vec::new()));
    if !entry.is_array() {
        *entry = Value::Array(Vec::new());
    }
    entry
        .as_array_mut()
        .ok_or_else(|| "expected TOML array".to_string())?
        .push(Value::Object(Map::new()));
    Ok(())
}

fn get_nested_array_mut<'a>(root: &'a mut Value, parts: &[String]) -> Option<&'a mut Vec<Value>> {
    let mut current = root;
    for part in parts {
        current = current.as_object_mut()?.get_mut(part)?;
    }
    current.as_array_mut()
}

fn set_toml_current_value(
    root: &mut Value,
    current: &TomlTarget,
    key_parts: &[String],
    value: Value,
) -> Result<(), String> {
    match current {
        TomlTarget::ObjectPath(path) => {
            let mut parts = path.clone();
            parts.extend_from_slice(key_parts);
            set_nested_value(root, &parts, value)
        }
        TomlTarget::ArrayLast(path) => {
            let array = get_nested_array_mut(root, path)
                .ok_or_else(|| "current TOML array table is missing".to_string())?;
            let last = array
                .last_mut()
                .ok_or_else(|| "current TOML array table is empty".to_string())?;
            set_nested_value(last, key_parts, value)
        }
    }
}

fn get_nested_value<'a>(value: &'a Value, parts: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for part in parts {
        current = current.get(*part)?;
    }
    Some(current)
}

fn clone_without_env(config: &Value) -> Value {
    let Some(object) = config.as_object() else {
        return Value::Object(Map::new());
    };
    Value::Object(
        object
            .iter()
            .filter(|(key, _)| key.as_str() != "env")
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect(),
    )
}

fn merge_config_value(base: &Value, override_value: Option<&Value>) -> Value {
    let Some(override_value) = override_value else {
        return base.clone();
    };
    match (base.as_object(), override_value.as_object()) {
        (Some(base_object), Some(override_object)) => {
            let mut out = base_object.clone();
            for (key, value) in override_object {
                let merged = out
                    .get(key)
                    .map(|base_value| merge_config_value(base_value, Some(value)))
                    .unwrap_or_else(|| value.clone());
                out.insert(key.clone(), merged);
            }
            Value::Object(out)
        }
        _ => override_value.clone(),
    }
}

fn arrayish_values(value: Option<&Value>) -> Vec<&Value> {
    match value {
        Some(Value::Array(items)) => items.iter().collect(),
        Some(Value::Null) | None => Vec::new(),
        Some(value) => vec![value],
    }
}

fn add_typegen_binding(
    bindings: &mut BTreeMap<String, TypegenBinding>,
    name: Option<&str>,
    type_name: &str,
    optional: bool,
) {
    let Some(name) = name.filter(|name| !name.is_empty()) else {
        return;
    };
    let entry = bindings
        .entry(name.to_string())
        .or_insert_with(|| TypegenBinding {
            name: name.to_string(),
            types: BTreeSet::new(),
            optional: true,
        });
    entry.types.insert(type_name.to_string());
    entry.optional &= optional;
}

fn extract_typegen_bindings_from_config(
    config: &Value,
    bindings: &mut BTreeMap<String, TypegenBinding>,
    optional: bool,
    strict_vars: bool,
) {
    if let Some(vars) = config.get("vars").and_then(Value::as_object) {
        for (name, value) in vars {
            add_typegen_binding(
                bindings,
                Some(name),
                &if strict_vars {
                    typegen_literal_type(value)
                } else {
                    typegen_loose_var_type(value)
                },
                optional,
            );
        }
    }

    for name in arrayish_values(get_nested_value(config, &["secrets", "required"])) {
        add_typegen_binding(bindings, name.as_str(), "string", optional);
    }

    for (parts, type_name) in TYPEGEN_BINDING_SPECS {
        for item in arrayish_values(get_nested_value(config, parts)) {
            let binding = item
                .get("binding")
                .and_then(Value::as_str)
                .or_else(|| item.get("name").and_then(Value::as_str));
            add_typegen_binding(bindings, binding, type_name, optional);
        }
    }

    for (parts, type_name) in TYPEGEN_SINGLETON_BINDING_SPECS {
        let binding = get_nested_value(config, parts).and_then(|item| {
            item.get("binding")
                .and_then(Value::as_str)
                .or_else(|| item.get("name").and_then(Value::as_str))
        });
        add_typegen_binding(bindings, binding, type_name, optional);
    }

    for item in arrayish_values(get_nested_value(config, &["unsafe", "bindings"])) {
        let binding = item
            .get("name")
            .and_then(Value::as_str)
            .or_else(|| item.get("binding").and_then(Value::as_str));
        add_typegen_binding(bindings, binding, "unknown", optional);
    }
}

fn collect_typegen_bindings(configs: &[Value], options: &TypegenOptions) -> Vec<TypegenBinding> {
    let mut bindings = BTreeMap::new();
    for config in configs {
        let base = clone_without_env(config);
        if let Some(env_name) = &options.env {
            let env_config = config.get("env").and_then(|env| env.get(env_name));
            let merged = merge_config_value(&base, env_config);
            extract_typegen_bindings_from_config(
                &merged,
                &mut bindings,
                false,
                options.strict_vars,
            );
            continue;
        }

        extract_typegen_bindings_from_config(&base, &mut bindings, false, options.strict_vars);
        if let Some(envs) = config.get("env").and_then(Value::as_object) {
            for env_config in envs.values() {
                extract_typegen_bindings_from_config(
                    env_config,
                    &mut bindings,
                    true,
                    options.strict_vars,
                );
            }
        }
    }
    bindings.into_values().collect()
}

fn read_typegen_entrypoint(
    entrypoint: Option<&str>,
    base_dir: &Path,
    explicit: bool,
) -> Result<Option<TypegenEntrypointInfo>, String> {
    let Some(entrypoint) = entrypoint else {
        return Ok(None);
    };
    let resolved = resolve_from(base_dir, entrypoint);
    if !resolved.exists() {
        if explicit {
            return Err(format!("Entrypoint does not exist: {entrypoint}"));
        }
        return Ok(Some(TypegenEntrypointInfo {
            path: entrypoint.to_string(),
            syntax: "unknown".to_string(),
            handlers: Vec::new(),
        }));
    }

    let source = fs::read_to_string(&resolved)
        .map_err(|err| format!("failed to read entrypoint {}: {err}", resolved.display()))?;
    let service_worker = TYPEGEN_KNOWN_HANDLERS
        .iter()
        .take(5)
        .any(|handler| source_has_add_event_listener(&source, handler));
    let has_default_export = source.contains("export default");
    let handlers = TYPEGEN_KNOWN_HANDLERS
        .iter()
        .copied()
        .filter(|handler| source_has_handler(&source, handler))
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    let syntax = if service_worker {
        "service-worker"
    } else if has_default_export {
        "module"
    } else {
        "unknown"
    };

    Ok(Some(TypegenEntrypointInfo {
        path: entrypoint.to_string(),
        syntax: syntax.to_string(),
        handlers,
    }))
}

fn source_has_add_event_listener(source: &str, event: &str) -> bool {
    [
        format!("addEventListener(\"{event}\""),
        format!("addEventListener('{event}'"),
        format!("addEventListener (\"{event}\""),
        format!("addEventListener ('{event}'"),
    ]
    .iter()
    .any(|needle| source.contains(needle))
}

fn source_has_handler(source: &str, handler: &str) -> bool {
    for (index, _) in source.match_indices(handler) {
        let before = source[..index].chars().next_back();
        if before.is_some_and(is_identifier_continue) {
            continue;
        }
        let after_index = index + handler.len();
        let mut chars = source[after_index..].chars();
        let first = chars.next();
        if first.is_some_and(is_identifier_continue) {
            continue;
        }
        let next = first
            .into_iter()
            .chain(chars)
            .find(|ch| !ch.is_whitespace());
        if matches!(next, Some('(' | ':')) {
            return true;
        }
    }
    false
}

fn render_typegen_declarations(inputs: &TypegenInputs, options: &TypegenOptions) -> String {
    let mut lines = Vec::new();
    let config_list = if inputs.config_paths.is_empty() {
        "none".to_string()
    } else {
        inputs
            .config_paths
            .iter()
            .map(|path| relative_for_comment(&options.cwd, path))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let entrypoint_label = inputs
        .entrypoint_info
        .as_ref()
        .map(|entrypoint| entrypoint.path.as_str())
        .unwrap_or("none");

    lines.push(
        "// Generated by Porffor. Regenerate with `porf types` after config changes.".to_string(),
    );
    lines.push(format!("// Config: {config_list}"));
    lines.push(format!("// Entrypoint: {entrypoint_label}"));
    if let Some(env) = &options.env {
        lines.push(format!("// Environment: {env}"));
    }
    if let Some(entrypoint) = &inputs.entrypoint_info {
        lines.push(format!("// Entrypoint syntax: {}", entrypoint.syntax));
        if !entrypoint.handlers.is_empty() {
            lines.push(format!(
                "// Detected handlers: {}",
                entrypoint.handlers.join(", ")
            ));
        }
    }
    lines.push(String::new());

    if options.include_runtime {
        lines.push(typegen_runtime_declarations().trim_end().to_string());
        lines.push(String::new());
    }

    if options.include_env {
        lines.push("declare namespace Porffor {".to_string());
        lines.push(format!("  interface {} {{", options.env_interface));
        if inputs.bindings.is_empty() {
            lines.push("    // No bindings were found in the provided config.".to_string());
        } else {
            for binding in &inputs.bindings {
                lines.push(format!(
                    "    {}{}: {};",
                    typegen_prop_name(&binding.name),
                    if binding.optional { "?" } else { "" },
                    format_typegen_binding_type(binding)
                ));
            }
        }
        lines.push("  }".to_string());

        let compatibility_date = inputs
            .configs
            .iter()
            .find_map(|config| config.get("compatibility_date"));
        let compatibility_flags = inputs
            .configs
            .iter()
            .flat_map(|config| arrayish_values(config.get("compatibility_flags")))
            .collect::<Vec<_>>();
        if compatibility_date.is_some()
            || !compatibility_flags.is_empty()
            || inputs.entrypoint_info.is_some()
        {
            lines.push(String::new());
            lines.push("  interface WorkerConfiguration {".to_string());
            if let Some(compatibility_date) = compatibility_date {
                lines.push(format!(
                    "    compatibilityDate: {};",
                    typegen_literal_type(compatibility_date)
                ));
            }
            if !compatibility_flags.is_empty() {
                let flag_types = compatibility_flags
                    .iter()
                    .map(|flag| typegen_literal_type(flag))
                    .collect::<Vec<_>>();
                lines.push(format!(
                    "    compatibilityFlags: readonly ({})[];",
                    flag_types.join(" | ")
                ));
            }
            if let Some(entrypoint) = &inputs.entrypoint_info {
                lines.push(format!(
                    "    entrypoint: {};",
                    typegen_ts_string(&entrypoint.path)
                ));
                lines.push(format!(
                    "    syntax: {};",
                    typegen_ts_string(&entrypoint.syntax)
                ));
            }
            lines.push("  }".to_string());
        }

        lines.push("}".to_string());
        lines.push(String::new());
        lines.push(format!(
            "interface {} extends Porffor.{} {{}}",
            options.env_interface, options.env_interface
        ));
        lines.push(String::new());
    }

    format!("{}\n", lines.join("\n").trim_end())
}

fn typegen_runtime_declarations() -> &'static str {
    r#"declare interface ExecutionContext {
  waitUntil(promise: Promise<unknown>): void;
  passThroughOnException(): void;
}

declare interface ScheduledController {
  readonly scheduledTime: number;
  readonly cron: string;
  noRetry(): void;
}

declare interface Message<Body = unknown> {
  readonly id: string;
  readonly timestamp: Date;
  readonly body: Body;
  ack(): void;
  retry(options?: QueueRetryOptions): void;
}

declare interface MessageBatch<Body = unknown> {
  readonly queue: string;
  readonly messages: readonly Message<Body>[];
  ackAll(): void;
  retryAll(options?: QueueRetryOptions): void;
}

declare interface QueueRetryOptions {
  delaySeconds?: number;
}

declare interface ExportedHandler<Env = unknown> {
  fetch?(request: Request, env: Env, ctx: ExecutionContext): Response | Promise<Response>;
  scheduled?(controller: ScheduledController, env: Env, ctx: ExecutionContext): void | Promise<void>;
  queue?(batch: MessageBatch, env: Env, ctx: ExecutionContext): void | Promise<void>;
  email?(message: unknown, env: Env, ctx: ExecutionContext): void | Promise<void>;
  tail?(events: readonly unknown[], env: Env, ctx: ExecutionContext): void | Promise<void>;
  trace?(traces: readonly unknown[], env: Env, ctx: ExecutionContext): void | Promise<void>;
  alarm?(controller: unknown, env: Env, ctx: ExecutionContext): void | Promise<void>;
}

declare interface Fetcher {
  fetch(input: RequestInfo | URL, init?: RequestInit): Promise<Response>;
}

declare interface KVNamespace {
  get(key: string, options?: unknown): Promise<unknown>;
  put(key: string, value: unknown, options?: unknown): Promise<void>;
  delete(key: string): Promise<void>;
  list(options?: unknown): Promise<unknown>;
}

declare interface R2Bucket {
  get(key: string, options?: unknown): Promise<unknown>;
  put(key: string, value: unknown, options?: unknown): Promise<unknown>;
  delete(keys: string | readonly string[]): Promise<void>;
  list(options?: unknown): Promise<unknown>;
}

declare interface D1PreparedStatement {
  bind(...values: unknown[]): D1PreparedStatement;
  first<T = unknown>(columnName?: string): Promise<T | null>;
  run<T = unknown>(): Promise<T>;
  all<T = unknown>(): Promise<T>;
  raw<T = unknown>(): Promise<T[]>;
}

declare interface D1Database {
  prepare(query: string): D1PreparedStatement;
  batch<T = unknown>(statements: readonly D1PreparedStatement[]): Promise<T[]>;
  exec(query: string): Promise<unknown>;
  dump(): Promise<ArrayBuffer>;
}

declare interface DurableObjectId {
  readonly name?: string;
  toString(): string;
}

declare interface DurableObjectStub {
  fetch(input: RequestInfo | URL, init?: RequestInit): Promise<Response>;
}

declare interface DurableObjectNamespace {
  newUniqueId(options?: unknown): DurableObjectId;
  idFromName(name: string): DurableObjectId;
  idFromString(id: string): DurableObjectId;
  get(id: DurableObjectId): DurableObjectStub;
}

declare interface Queue<Body = unknown> {
  send(message: Body, options?: unknown): Promise<void>;
  sendBatch(messages: readonly { body: Body; options?: unknown }[]): Promise<void>;
}

declare interface AnalyticsEngineDataset {
  writeDataPoint(event: unknown): void;
}

declare interface VectorizeIndex {
  query(vector: readonly number[], options?: unknown): Promise<unknown>;
  insert(vectors: readonly unknown[]): Promise<unknown>;
  upsert(vectors: readonly unknown[]): Promise<unknown>;
  deleteByIds(ids: readonly string[]): Promise<unknown>;
}

declare interface Ai {
  run(model: string, inputs: unknown, options?: unknown): Promise<unknown>;
}

declare interface AiSearch {
  search(query: unknown, options?: unknown): Promise<unknown>;
}

declare interface AiSearchNamespace {
  search(query: unknown, options?: unknown): Promise<unknown>;
}

declare interface BrowserRendering {
  launch(options?: unknown): Promise<unknown>;
}

declare interface ImagesBinding {
  input(value: unknown): unknown;
}

declare interface Hyperdrive {
  readonly connectionString: string;
}

declare interface Workflow {
  create(options?: unknown): Promise<unknown>;
  get(id: string): Promise<unknown>;
}

declare interface Pipeline {
  send(records: readonly unknown[]): Promise<void>;
}

declare interface DispatchNamespace {
  get(name: string, args?: unknown): Fetcher;
}

declare interface SendEmail {
  send(message: unknown): Promise<void>;
}
"#
}

fn relative_for_comment(cwd: &Path, path: &Path) -> String {
    path.strip_prefix(cwd)
        .ok()
        .filter(|relative| !relative.as_os_str().is_empty())
        .unwrap_or(path)
        .display()
        .to_string()
}

fn typegen_prop_name(name: &str) -> String {
    if is_ts_identifier(name) {
        name.to_string()
    } else {
        typegen_ts_string(name)
    }
}

fn typegen_literal_type(value: &Value) -> String {
    match value {
        Value::String(value) => typegen_ts_string(value),
        Value::Number(value) => value.to_string(),
        Value::Bool(value) => value.to_string(),
        Value::Null => "null".to_string(),
        Value::Array(values) => {
            let types = values
                .iter()
                .map(typegen_literal_type)
                .collect::<BTreeSet<_>>();
            if types.is_empty() {
                "readonly unknown[]".to_string()
            } else {
                format!(
                    "readonly ({})[]",
                    types.into_iter().collect::<Vec<_>>().join(" | ")
                )
            }
        }
        Value::Object(_) => "unknown".to_string(),
    }
}

fn typegen_loose_var_type(value: &Value) -> String {
    match value {
        Value::String(_) => "string",
        Value::Number(_) => "number",
        Value::Bool(_) => "boolean",
        Value::Null => "null",
        Value::Array(_) => "readonly unknown[]",
        Value::Object(_) => "Record<string, unknown>",
    }
    .to_string()
}

fn format_typegen_binding_type(binding: &TypegenBinding) -> String {
    if binding.types.len() == 1 {
        binding.types.iter().next().cloned().unwrap_or_default()
    } else {
        binding
            .types
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .join(" | ")
    }
}

fn typegen_ts_string(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| format!("\"{}\"", value.replace('"', "\\\"")))
}

fn is_ts_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    is_identifier_start(first) && chars.all(is_identifier_continue)
}

fn is_identifier_start(ch: char) -> bool {
    ch == '_' || ch == '$' || ch.is_ascii_alphabetic()
}

fn is_identifier_continue(ch: char) -> bool {
    is_identifier_start(ch) || ch.is_ascii_digit()
}

fn handle_test262_command(args: Vec<String>) -> Result<(), String> {
    if args.is_empty() {
        return Err(format!("test262 needs a subcommand\n\n{}", usage()));
    }

    let subcommand = args[0].clone();
    let parsed = parse_test262_args(&args[1..])?;
    let runner = ConformanceRunner::with_config(parsed.config);
    let execution_backend = parsed.run_config.execution_backend;

    match subcommand.as_str() {
        "sync" => {
            let pinned = runner.pinned_revisions();
            if !runner.config().suite_root.join("test").exists() {
                return Err(format!(
                    "vendored suite missing at {}",
                    runner.config().suite_root.display()
                ));
            }
            println!("suite_root: {}", runner.config().suite_root.display());
            println!("test262 revision: {}", pinned.test262);
            println!("execution_backend: {}", execution_backend.as_str());
            println!(
                "local harness: {}",
                runner.config().local_harness_path.display()
            );
            Ok(())
        }
        "list" => {
            let manifest = runner.discover_suite(parsed.filter.as_deref())?;
            println!("count: {}", manifest.cases.len());
            for case in manifest.cases.iter().take(50) {
                println!("{}", case.path);
            }
            if manifest.cases.len() > 50 {
                println!("... {} more", manifest.cases.len() - 50);
            }
            Ok(())
        }
        "run" => {
            let summary = runner.run_full(parsed.run_config)?;
            println!("execution_backend: {}", execution_backend.as_str());
            println!("total: {}", summary.total);
            println!("passed: {}", summary.passed);
            for kind in porffor_test262::FailureKind::ALL {
                let count = summary.counts_per_kind.get(&kind).copied().unwrap_or(0);
                println!("{}: {}", kind.as_str(), count);
            }
            println!("outcomes:");
            for outcome in OutcomeKind::ALL {
                let count = summary
                    .counts_per_outcome
                    .get(&outcome)
                    .copied()
                    .unwrap_or(0);
                println!("  {}: {}", outcome.as_str(), count);
            }
            for failure in summary.failures.iter().take(10) {
                println!(
                    "failure: {} [{}:{}] {}",
                    failure.test_path,
                    failure.outcome.as_str(),
                    failure.kind.as_str(),
                    failure.detail
                );
            }
            Ok(())
        }
        "report" => {
            let summary = runner.run_full(parsed.run_config)?;
            let report = runner.baseline_report(&summary);
            println!("execution_backend: {}", execution_backend.as_str());
            println!("total: {}", report.total);
            println!("passed: {}", report.passed);
            println!("failed: {}", report.failed);
            println!("outcomes:");
            for outcome in OutcomeKind::ALL {
                let count = summary
                    .counts_per_outcome
                    .get(&outcome)
                    .copied()
                    .unwrap_or(0);
                println!("  {}: {}", outcome.as_str(), count);
            }
            for bucket in report.buckets {
                println!("bucket: {} ({})", bucket.kind.as_str(), bucket.total);
                for (subtree, count) in bucket.top_subtrees.iter().take(5) {
                    println!("  {subtree}: {count}");
                }
            }
            Ok(())
        }
        "report-all" => {
            if parsed.filter.is_some() {
                return Err(
                    "report-all does not take a filter; it always runs the top-level matrix"
                        .to_string(),
                );
            }
            let report =
                runner.aggregate_baseline_report(&runner.run_top_level_matrix(parsed.run_config)?);
            println!("execution_backend: {}", execution_backend.as_str());
            println!("total: {}", report.total);
            println!("passed: {}", report.passed);
            println!("failed: {}", report.failed);
            for kind in porffor_test262::FailureKind::ALL {
                let count = report.counts_per_kind.get(&kind).copied().unwrap_or(0);
                println!("{}: {}", kind.as_str(), count);
            }
            println!("outcomes:");
            for outcome in OutcomeKind::ALL {
                let count = report
                    .counts_per_outcome
                    .get(&outcome)
                    .copied()
                    .unwrap_or(0);
                println!("  {}: {}", outcome.as_str(), count);
            }
            println!("origins:");
            for origin in porffor_test262::FailureOrigin::ALL {
                let count = report.counts_per_origin.get(&origin).copied().unwrap_or(0);
                println!("  {}: {}", origin.as_str(), count);
            }
            println!("targets:");
            for entry in report.entries {
                println!(
                    "  {}: {}/{} passed",
                    entry.filter, entry.passed, entry.total
                );
            }
            Ok(())
        }
        "publish-status" => {
            if parsed.filter.is_some() {
                return Err(
                    "publish-status does not take a filter; it always runs the top-level matrix"
                        .to_string(),
                );
            }
            if parsed.run_config.max_matrix_nodes.is_some() {
                return Err(
                    "publish-status does not allow --max-matrix-nodes; use report-all or run the full matrix"
                        .to_string(),
                );
            }
            let verified = match runner.load_verified_aggregate_summary(
                &parsed.run_config.snapshot_name,
                execution_backend,
            ) {
                Ok(verified) => verified,
                Err(err)
                    if err.contains("missing aggregate snapshot")
                        || err.contains("aggregate snapshot incomplete") =>
                {
                    let mut resume_run_config = parsed.run_config.clone();
                    resume_run_config.resume = true;
                    runner.run_top_level_matrix(resume_run_config)?;
                    runner.load_verified_aggregate_summary(
                        &parsed.run_config.snapshot_name,
                        execution_backend,
                    )?
                }
                Err(err) => return Err(err),
            };
            let refresh_date = current_utc_date_string()?;
            let fake_counts = fake_suite_counts()?;
            let artifact = build_published_status_artifact(
                &fake_counts,
                &verified,
                execution_backend,
                &refresh_date,
            );
            let status_paths = write_published_status_artifact(
                &runner.config().snapshot_dir,
                execution_backend,
                &artifact,
            )?;
            let readme_path = parsed.readme_path.unwrap_or_else(default_readme_path);
            rewrite_current_status_block(&readme_path, &artifact)?;

            println!("execution_backend: {}", execution_backend.as_str());
            println!("refresh_date: {}", refresh_date);
            println!("total: {}", verified.summary.total);
            println!("passed: {}", verified.summary.passed);
            println!("failed: {}", verified.summary.failed);
            println!("manifest_hash: {}", verified.manifest_hash);
            println!("pinned_ecma262: {}", verified.pinned_revisions.ecma262);
            println!("pinned_test262: {}", verified.pinned_revisions.test262);
            for entry in &artifact.real_suite.counts_per_kind {
                println!("kind_{}: {}", entry.label, entry.count);
            }
            for entry in &artifact.real_suite.counts_per_outcome {
                println!("outcome_{}: {}", entry.label, entry.count);
            }
            for entry in &artifact.real_suite.counts_per_origin {
                println!("origin_{}: {}", entry.label, entry.count);
            }
            println!(
                "snapshot_json: {}",
                verified.snapshot_paths.json_path.display()
            );
            println!(
                "snapshot_txt: {}",
                verified.snapshot_paths.txt_path.display()
            );
            println!("status_json: {}", status_paths.json_path.display());
            println!("status_txt: {}", status_paths.txt_path.display());
            println!("readme_path: {}", readme_path.display());
            println!("top_targets:");
            if artifact.real_suite.top_targets.is_empty() {
                println!("  none");
            } else {
                for entry in &artifact.real_suite.top_targets {
                    println!(
                        "  {}: {}/{} passed",
                        entry.filter, entry.passed, entry.total
                    );
                }
            }
            Ok(())
        }
        "progress-status" => {
            if parsed.filter.is_some() {
                return Err(
                    "progress-status does not take a filter; it always reads the top-level matrix"
                        .to_string(),
                );
            }
            let progress = runner.load_aggregate_progress_summary(
                &parsed.run_config.snapshot_name,
                execution_backend,
            )?;
            let success = progress
                .summary
                .counts_per_outcome
                .get(&OutcomeKind::Success)
                .copied()
                .unwrap_or(0);
            let not_implemented = progress
                .summary
                .counts_per_outcome
                .get(&OutcomeKind::NotImplemented)
                .copied()
                .unwrap_or(0);
            let crash = progress
                .summary
                .counts_per_outcome
                .get(&OutcomeKind::Crash)
                .copied()
                .unwrap_or(0);
            let bug = progress
                .summary
                .counts_per_outcome
                .get(&OutcomeKind::Bug)
                .copied()
                .unwrap_or(0);
            let unobserved_total = progress.target_total.saturating_sub(progress.summary.total);
            let remaining_to_green = unobserved_total + not_implemented + crash + bug;

            println!("execution_backend: {}", execution_backend.as_str());
            println!("complete={}", progress.complete);
            println!(
                "matrix_nodes_completed: {}",
                progress.matrix_nodes_completed
            );
            println!("matrix_nodes_total: {}", progress.matrix_nodes_total);
            println!("observed_total: {}", progress.summary.total);
            println!("target_total: {}", progress.target_total);
            println!("unobserved_total: {}", unobserved_total);
            println!("current_success: {}", success);
            println!(
                "current_success_full: {}/{}",
                success, progress.target_total
            );
            println!(
                "remaining_observed_failures: {}",
                not_implemented + crash + bug
            );
            println!("remaining_to_green: {}", remaining_to_green);
            println!("manifest_hash: {}", progress.manifest_hash);
            println!("pinned_ecma262: {}", progress.pinned_revisions.ecma262);
            println!("pinned_test262: {}", progress.pinned_revisions.test262);
            println!(
                "snapshot_json: {}",
                progress.snapshot_paths.json_path.display()
            );
            println!("outcomes:");
            for outcome in OutcomeKind::ALL {
                let count = progress
                    .summary
                    .counts_per_outcome
                    .get(&outcome)
                    .copied()
                    .unwrap_or(0);
                println!("  {}: {}", outcome.as_str(), count);
            }
            println!(
                "burn_down: NotImplemented={} Crash={} Bug={}",
                not_implemented, crash, bug
            );
            println!("not_run: {}", unobserved_total);
            Ok(())
        }
        "triage-status" => {
            if parsed.filter.is_some() {
                return Err(
                    "triage-status does not take a filter; it ranks completed failing matrix nodes"
                        .to_string(),
                );
            }
            let entries = runner
                .load_matrix_triage_entries(&parsed.run_config.snapshot_name, execution_backend)?;
            println!("execution_backend: {}", execution_backend.as_str());
            println!("failing_nodes: {}", entries.len());
            println!("ranking: Crash,Bug,NotImplemented,failed");
            if entries.is_empty() {
                println!("  none");
            } else {
                for entry in entries.iter().take(25) {
                    println!(
                        "node: {} filter={} passed={}/{} failed={} Crash={} Bug={} NotImplemented={}",
                        entry.node_id,
                        entry.filter,
                        entry.passed,
                        entry.total,
                        entry.failed,
                        entry.crash,
                        entry.bug,
                        entry.not_implemented
                    );
                }
            }
            Ok(())
        }
        "failure-details" => {
            let node_selector = parsed.filter.as_deref().ok_or_else(|| {
                "failure-details needs a matrix node id or exact filter".to_string()
            })?;
            let details = runner.load_matrix_failure_details(
                &parsed.run_config.snapshot_name,
                execution_backend,
                node_selector,
            )?;
            println!("execution_backend: {}", execution_backend.as_str());
            println!("node_id: {}", details.node_id);
            println!("filter: {}", details.filter);
            println!("matrix_path: {}", details.matrix_path.join("/"));
            println!(
                "passed: {}/{} failed={}",
                details.passed, details.total, details.failed
            );
            println!("detail_groups: {}", details.groups.len());
            for group in details.groups.iter().take(25) {
                println!(
                    "detail: count={} outcome={} kind={} origin={} hash={}",
                    group.count,
                    group.outcome.as_str(),
                    group.kind.as_str(),
                    group.origin.as_str(),
                    group.detail_hash
                );
                println!("  {}", group.detail);
                for test in &group.representative_tests {
                    println!("  test: {}", test);
                }
            }
            Ok(())
        }
        "generate-backlog" => {
            if parsed.filter.is_some() {
                return Err(
                    "generate-backlog does not take a filter; it reads a verified aggregate snapshot"
                        .to_string(),
                );
            }
            let (artifact, paths) =
                runner.generate_backlog(&parsed.run_config.snapshot_name, execution_backend)?;
            println!("execution_backend: {}", execution_backend.as_str());
            println!("snapshot_name: {}", artifact.snapshot_name);
            println!("pinned_ecma262: {}", artifact.pinned_revisions.ecma262);
            println!("pinned_test262: {}", artifact.pinned_revisions.test262);
            println!("total: {}", artifact.total);
            println!("passed: {}", artifact.passed);
            println!("failed: {}", artifact.failed);
            println!("records: {}", artifact.records.len());
            println!("backlog_json: {}", paths.json_path.display());
            println!("backlog_txt: {}", paths.txt_path.display());
            println!("by_task:");
            for (task, count) in &artifact.summary_by_task {
                println!("  {task}: {count}");
            }
            Ok(())
        }
        "compare-snapshots" => {
            let base_snapshot_name = parsed.filter.as_deref().ok_or_else(|| {
                "compare-snapshots needs a base snapshot name; candidate uses --snapshot-name"
                    .to_string()
            })?;
            let comparison = runner.compare_snapshots(
                base_snapshot_name,
                &parsed.run_config.snapshot_name,
                execution_backend,
            )?;
            println!("execution_backend: {}", execution_backend.as_str());
            println!("base_snapshot: {}", comparison.base_snapshot_name);
            println!("candidate_snapshot: {}", comparison.candidate_snapshot_name);
            println!("pinned_ecma262: {}", comparison.pinned_revisions.ecma262);
            println!("pinned_test262: {}", comparison.pinned_revisions.test262);
            println!("base_total: {}", comparison.base_total);
            println!("candidate_total: {}", comparison.candidate_total);
            println!("added_passes: {}", comparison.added_passes.len());
            for path in comparison.added_passes.iter().take(25) {
                println!("  pass: {path}");
            }
            println!("regressions: {}", comparison.regressions.len());
            for path in comparison.regressions.iter().take(25) {
                println!("  regression: {path}");
            }
            println!(
                "changed_failure_hashes: {}",
                comparison.changed_failure_hashes.len()
            );
            for change in comparison.changed_failure_hashes.iter().take(25) {
                println!(
                    "  hash: {} {:016x}->{:016x}",
                    change.test_path, change.base_hash, change.candidate_hash
                );
            }
            Ok(())
        }
        "shard" => {
            let summary = runner.run_shard(parsed.run_config)?;
            println!("execution_backend: {}", execution_backend.as_str());
            println!("shard: {}/{}", summary.shard_index + 1, summary.shard_count);
            println!("total: {}", summary.total);
            println!("passed: {}", summary.passed);
            println!("failed: {}", summary.failures.len());
            Ok(())
        }
        "compare-js-oracle" => {
            let comparison = try_compare_with_js_oracle(runner.config(), parsed.filter.as_deref())?;
            println!("rust_count: {}", comparison.rust_count);
            match comparison.js_count {
                Some(js_count) => println!("js_count: {}", js_count),
                None => println!("js_count: unavailable"),
            }
            match comparison.matches {
                Some(matches) => println!("matches: {}", matches),
                None => println!("matches: unavailable"),
            }
            if let Some(reason) = comparison.unavailable_reason {
                println!("oracle_status: unavailable");
                println!("oracle_reason: {}", reason);
            }
            Ok(())
        }
        _ => Err(format!("unknown test262 subcommand: {subcommand}")),
    }
}

fn parse_test262_args(args: &[String]) -> Result<ParsedTest262Args, String> {
    let mut config = SuiteConfig::default();
    let mut filter = None::<String>;
    let mut run_config = RunConfig::default();
    // CLI-level product default: Wasm-AOT is the only backend whose results
    // count as conformance (AGENTS.md). `porffor-test262::RunConfig`'s own
    // struct default stays out of this lane's scope; the CLI entry point
    // overrides it here so `porf test262 ...` is Wasm-AOT flag-free.
    // `--execution-backend spec-exec` remains available as an explicit,
    // internal/debug-only differential-oracle override (see usage()).
    run_config.execution_backend = ExecutionBackend::WasmAot;
    let mut readme_path = None::<PathBuf>;

    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--suite-root" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| "--suite-root needs a value".to_string())?;
                config.suite_root = PathBuf::from(value);
            }
            "--snapshot-dir" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| "--snapshot-dir needs a value".to_string())?;
                config.snapshot_dir = PathBuf::from(value);
            }
            "--threads" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| "--threads needs a value".to_string())?;
                config.worker_count = value
                    .parse::<usize>()
                    .map_err(|err| format!("invalid --threads value {value}: {err}"))?;
            }
            "--timeout-ms" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| "--timeout-ms needs a value".to_string())?;
                config.timeout_ms = value
                    .parse::<u64>()
                    .map_err(|err| format!("invalid --timeout-ms value {value}: {err}"))?;
            }
            "--execution-backend" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| "--execution-backend needs a value".to_string())?;
                run_config.execution_backend = parse_execution_backend(value)?;
            }
            "--resume" => {
                run_config.resume = true;
            }
            "--snapshot-name" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| "--snapshot-name needs a value".to_string())?;
                run_config.snapshot_name = value.clone();
            }
            "--max-matrix-nodes" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| "--max-matrix-nodes needs a value".to_string())?;
                run_config.max_matrix_nodes =
                    Some(value.parse::<usize>().map_err(|err| {
                        format!("invalid --max-matrix-nodes value {value}: {err}")
                    })?);
            }
            "--readme-path" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| "--readme-path needs a value".to_string())?;
                readme_path = Some(PathBuf::from(value));
            }
            value if value.contains('/') && value.split('/').count() == 2 && filter.is_none() => {
                let parts = value.split('/').collect::<Vec<_>>();
                match (parts[0].parse::<usize>(), parts[1].parse::<usize>()) {
                    (Ok(one_based_index), Ok(shard_count)) => {
                        run_config.shard_index = one_based_index.saturating_sub(1);
                        run_config.shard_count = shard_count.max(1);
                    }
                    _ => filter = Some(value.to_string()),
                }
            }
            value if !value.starts_with('-') && filter.is_none() => {
                filter = Some(value.to_string());
            }
            value => return Err(format!("unknown test262 arg: {value}")),
        }
        index += 1;
    }

    if config.suite_root == PathBuf::from("test262/vendor/test262") {
        let root = PathBuf::from("test262");
        config.local_harness_path = root.join("harness.js");
        if config.snapshot_dir == SuiteConfig::default().snapshot_dir {
            config.snapshot_dir = root.join("snapshots");
        }
    } else if config.local_harness_path == SuiteConfig::default().local_harness_path {
        let guessed_root = config
            .suite_root
            .parent()
            .and_then(Path::parent)
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        config.local_harness_path = guessed_root.join("harness.js");
        if config.snapshot_dir == SuiteConfig::default().snapshot_dir {
            config.snapshot_dir = guessed_root.join("snapshots");
        }
    }

    if run_config.execution_backend == ExecutionBackend::WasmAot {
        let wasm_harness_path = config
            .local_harness_path
            .with_file_name("harness-wasm-aot.js");
        let wasm_harness_exists = wasm_harness_path.exists()
            || (!wasm_harness_path.is_absolute() && repo_root().join(&wasm_harness_path).exists());
        if wasm_harness_exists {
            config.local_harness_path = wasm_harness_path;
        }
    }

    if std::env::var_os("PORFFOR_TEST262_DISABLE_CASE_RUNNER").is_none() {
        config.case_runner_bin = std::env::current_exe().ok();
    }

    run_config.filter = filter.clone();

    Ok(ParsedTest262Args {
        config,
        filter,
        run_config,
        readme_path,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedRunArgs {
    backend: ExecutionBackend,
    path: Option<String>,
}

fn parse_run_args(args: &[String]) -> Result<ParsedRunArgs, String> {
    // Product default: Wasm-AOT. `--execution-backend spec-exec` is an
    // explicit, internal/debug-only differential-oracle override (never a
    // silent fallback) — see usage() and ExecutionBackend's docs.
    let mut backend = ExecutionBackend::WasmAot;
    let mut path = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--execution-backend" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| "--execution-backend needs a value".to_string())?;
                backend = parse_execution_backend(value)?;
            }
            value if !value.starts_with('-') && path.is_none() => {
                path = Some(value.to_string());
            }
            value => return Err(format!("unknown run arg: {value}")),
        }
        index += 1;
    }
    Ok(ParsedRunArgs { backend, path })
}

fn parse_execution_backend(value: &str) -> Result<ExecutionBackend, String> {
    match value {
        "spec" | "spec-exec" => Ok(ExecutionBackend::SpecExec),
        "wasm" | "wasm-aot" => Ok(ExecutionBackend::WasmAot),
        _ => Err(format!(
            "unknown execution backend: {value} (expected spec or wasm)"
        )),
    }
}

fn read_source(path: &str) -> Result<String, String> {
    fs::read_to_string(path).map_err(|err| format!("failed to read {path}: {err}"))
}

fn is_module_path(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext == "mjs")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_test262_args_reads_filter_and_shard() {
        let parsed = parse_test262_args(&[
            "1/4".to_string(),
            "language/expressions".to_string(),
            "--timeout-ms".to_string(),
            "50".to_string(),
        ])
        .expect("args should parse");
        assert_eq!(parsed.run_config.shard_index, 0);
        assert_eq!(parsed.run_config.shard_count, 4);
        assert_eq!(parsed.filter.as_deref(), Some("language/expressions"));
        assert_eq!(parsed.config.timeout_ms, 50);
        assert!(parsed.config.case_runner_bin.is_some());
        // Product default: `porf test262 ...` runs Wasm-AOT flag-free.
        assert_eq!(
            parsed.run_config.execution_backend,
            ExecutionBackend::WasmAot
        );
    }

    #[test]
    fn parse_run_args_defaults_to_wasm_aot_backend() {
        let parsed = parse_run_args(&["script.js".to_string()]).expect("run args should parse");
        assert_eq!(parsed.backend, ExecutionBackend::WasmAot);
        assert_eq!(parsed.path.as_deref(), Some("script.js"));
    }

    #[test]
    fn parse_run_args_spec_exec_requires_explicit_flag() {
        let parsed = parse_run_args(&[
            "--execution-backend".to_string(),
            "spec-exec".to_string(),
            "script.js".to_string(),
        ])
        .expect("run args should parse");
        assert_eq!(parsed.backend, ExecutionBackend::SpecExec);
    }

    #[test]
    fn parse_test262_args_defaults_to_wasm_aot_backend() {
        let parsed = parse_test262_args(&[]).expect("empty args should parse");
        assert_eq!(
            parsed.run_config.execution_backend,
            ExecutionBackend::WasmAot
        );
    }

    #[test]
    fn parse_test262_args_spec_exec_requires_explicit_flag() {
        // wasm-aot is the harness default (tasks/25); spec-exec requires the
        // explicit --execution-backend flag exercised here.
        let parsed =
            parse_test262_args(&["--execution-backend".to_string(), "spec-exec".to_string()])
                .expect("explicit spec-exec backend should parse");
        assert_eq!(
            parsed.run_config.execution_backend,
            ExecutionBackend::SpecExec
        );
    }

    #[test]
    fn parse_test262_args_reads_execution_backend() {
        let parsed = parse_test262_args(&["--execution-backend".to_string(), "wasm".to_string()])
            .expect("backend should parse");
        assert_eq!(
            parsed.run_config.execution_backend,
            ExecutionBackend::WasmAot
        );
    }

    #[test]
    fn parse_test262_args_uses_wasm_aot_harness_when_present() {
        let parsed = parse_test262_args(&[
            "--execution-backend".to_string(),
            "wasm-aot".to_string(),
            "--suite-root".to_string(),
            "test262/vendor/test262".to_string(),
        ])
        .expect("backend should parse");
        assert_eq!(
            parsed.config.local_harness_path,
            PathBuf::from("test262").join("harness-wasm-aot.js")
        );
    }
}
