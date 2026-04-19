use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use porffor_engine::{
    CompileOptions, Engine, ExecutionBackend, HostHooks, RealmBuilder, RunOptions,
};
use porffor_test262::{
    try_compare_with_js_oracle, ConformanceRunner, FailureKind, FailureOrigin, RunConfig,
    SuiteConfig, VerifiedAggregateSummary,
};
use serde::Serialize;

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
struct PublishedRealSuiteStatus {
    backend: String,
    refresh_date: String,
    manifest_hash: u64,
    passed: usize,
    total: usize,
    failed: usize,
    pinned_revisions: PublishedPinnedRevisions,
    counts_per_kind: Vec<PublishedCountEntry>,
    counts_per_origin: Vec<PublishedCountEntry>,
    top_targets: Vec<PublishedTargetEntry>,
    snapshot_paths: PublishedSnapshotPaths,
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
  run [--execution-backend spec|wasm] <file>
                                        compile and run a script through Rust engine path
  repl                                  reserved for the Rust REPL shell
  build wasm <file>                     compile JavaScript directly to Wasm
  build c <file>                        emit C from shared IR
  build native <file>                   emit native artifact from shared IR
  test262 sync [--suite-root PATH]
  test262 list [filter] [--suite-root PATH]
  test262 run [filter] [options]
  test262 shard <index>/<total> [filter] [options]
  test262 report [filter] [options]
  test262 report-all [options]
  test262 publish-status [options]
  test262 compare-js-oracle [filter] [--suite-root PATH]
  inspect <file>                        show compile pipeline summary

test262 options:
  --suite-root PATH
  --snapshot-dir PATH
  --threads N
  --timeout-ms N
  --execution-backend spec|wasm
  --resume
  --snapshot-name NAME
  --max-matrix-nodes N
  --readme-path PATH
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
            let mut backend = ExecutionBackend::SpecExec;
            let mut path = None;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--execution-backend" => {
                        let value = args
                            .next()
                            .ok_or_else(|| "--execution-backend needs a value".to_string())?;
                        backend = parse_execution_backend(&value)?;
                    }
                    value if !value.starts_with('-') && path.is_none() => {
                        path = Some(value.to_string());
                    }
                    value => return Err(format!("unknown run arg: {value}")),
                }
            }
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
        ..SuiteConfig::default()
    }
}

fn fake_suite_counts() -> Result<FakeSuiteCounts, String> {
    let runner = ConformanceRunner::with_config(fake_suite_config());
    let full_total = runner.discover_suite(None)?.cases.len();
    let wasm_safe_total = runner.discover_suite(Some("language/wasm/pass"))?.cases.len();
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
            counts_per_kind: sorted_kind_counts(&summary.summary.counts_per_kind),
            counts_per_origin: sorted_origin_counts(&summary.summary.counts_per_origin),
            top_targets: top_target_entries(summary),
            snapshot_paths: PublishedSnapshotPaths {
                json: summary.snapshot_paths.json_path.display().to_string(),
                txt: summary.snapshot_paths.txt_path.display().to_string(),
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
    out.push_str(&format!("snapshot_json={}\n", real.snapshot_paths.json));
    out.push_str(&format!("snapshot_txt={}\n", real.snapshot_paths.txt));
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

fn top_target_labels(entries: &[PublishedTargetEntry], limit: usize) -> String {
    let labels = entries
        .iter()
        .take(limit)
        .map(|entry| format!("`{}: {}/{} passed`", entry.filter, entry.passed, entry.total))
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
        "## Current Status\n<!-- porffor-status:start -->\nRust rewrite status must be read in layers, not one vanity number:\n- Fake wasm-safe Test262 subset: `{}/{}` green\n- Fake full Rust rewrite suite: `{}/{}` green\n- Pinned real Test262 baseline (`{}`, refreshed `{}`): `{}/{}` {} (`{}`)\n- Pinned revisions: `ecma262={}` `test262={}`\n- Biggest current real failing kinds: {}\n- Biggest current real failing origins: {}\n- Worst current real matrix targets: {}\n\nAs of `{}`, Rust Wasm-AOT path is at 100% of repo fake coverage, not 100% ECMAScript. Project is still off literal 100% until full pinned real Test262 run is green for Rust path.\n\nStatus refresh commands:\n- `cargo test -p porffor-engine --quiet`\n- `cargo test -p porffor-cli --quiet`\n- `./target/debug/porf test262 run language/wasm/pass --suite-root crates/porffor-test262/tests/fixtures/fake_test262/vendor/test262 --execution-backend wasm`\n- `./target/debug/porf test262 run --suite-root crates/porffor-test262/tests/fixtures/fake_test262/vendor/test262`\n- `./target/debug/porf test262 publish-status --execution-backend {}`\n\nWhen counts move, update this block in same change. Do not claim full Test262 `100%` from fake-suite numbers.\n<!-- porffor-status:end -->",
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
        real.pinned_revisions.ecma262,
        real.pinned_revisions.test262,
        top_nonzero_labels(&real.counts_per_kind, 3),
        top_nonzero_labels(&real.counts_per_origin, 3),
        top_target_labels(&real.top_targets, 3),
        real.refresh_date,
        real.backend,
    )
}

fn rewrite_current_status_block(readme_path: &Path, artifact: &PublishedStatusArtifact) -> Result<(), String> {
    let raw = fs::read_to_string(readme_path)
        .map_err(|err| format!("failed to read {}: {err}", readme_path.display()))?;
    let replacement = render_current_status_block(artifact);
    let updated = if let (Some(start), Some(end)) = (
        raw.find("<!-- porffor-status:start -->"),
        raw.find("<!-- porffor-status:end -->"),
    ) {
        let section_start = raw[..start]
            .rfind("## Current Status")
            .ok_or_else(|| format!("missing `## Current Status` before status marker in {}", readme_path.display()))?;
        let after_end = end + "<!-- porffor-status:end -->".len();
        format!("{}{}{}", &raw[..section_start], replacement, &raw[after_end..])
    } else {
        let section_start = raw
            .find("## Current Status")
            .ok_or_else(|| format!("missing `## Current Status` in {}", readme_path.display()))?;
        let section_end = raw[section_start + "## Current Status".len()..]
            .find("\n## ")
            .map(|offset| section_start + "## Current Status".len() + offset)
            .unwrap_or(raw.len());
        format!("{}{}{}", &raw[..section_start], replacement, &raw[section_end..])
    };
    fs::write(readme_path, updated)
        .map_err(|err| format!("failed to write {}: {err}", readme_path.display()))
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
            for failure in summary.failures.iter().take(10) {
                println!(
                    "failure: {} [{}] {}",
                    failure.test_path,
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
            let verified = match runner
                .load_verified_aggregate_summary(&parsed.run_config.snapshot_name, execution_backend)
            {
                Ok(verified) => verified,
                Err(err)
                    if err.contains("missing aggregate snapshot")
                        || err.contains("aggregate snapshot incomplete") =>
                {
                    runner.run_top_level_matrix(parsed.run_config.clone())?;
                    runner.load_verified_aggregate_summary(
                        &parsed.run_config.snapshot_name,
                        execution_backend,
                    )?
                }
                Err(err) => return Err(err),
            };
            let refresh_date = current_utc_date_string()?;
            let fake_counts = fake_suite_counts()?;
            let artifact =
                build_published_status_artifact(&fake_counts, &verified, execution_backend, &refresh_date);
            let status_paths =
                write_published_status_artifact(&runner.config().snapshot_dir, execution_backend, &artifact)?;
            let readme_path = parsed.readme_path.unwrap_or_else(default_readme_path);
            rewrite_current_status_block(&readme_path, &artifact)?;

            println!("execution_backend: {}", execution_backend.as_str());
            println!("refresh_date: {}", refresh_date);
            println!("total: {}", verified.summary.total);
            println!("passed: {}", verified.summary.passed);
            println!("failed: {}", verified.summary.failed);
            println!("manifest_hash: {}", verified.manifest_hash);
            println!(
                "pinned_ecma262: {}",
                verified.pinned_revisions.ecma262
            );
            println!("pinned_test262: {}", verified.pinned_revisions.test262);
            for entry in &artifact.real_suite.counts_per_kind {
                println!("kind_{}: {}", entry.label, entry.count);
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
                run_config.max_matrix_nodes = Some(
                    value
                        .parse::<usize>()
                        .map_err(|err| format!("invalid --max-matrix-nodes value {value}: {err}"))?,
                );
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

    run_config.filter = filter.clone();

    Ok(ParsedTest262Args {
        config,
        filter,
        run_config,
        readme_path,
    })
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
}
