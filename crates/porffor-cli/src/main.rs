use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use porffor_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};
use porffor_test262::{try_compare_with_js_oracle, ConformanceRunner, RunConfig, SuiteConfig};

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

    let engine = Engine::new(RealmBuilder::new().build());
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

struct ParsedTest262Args {
    config: SuiteConfig,
    filter: Option<String>,
    run_config: RunConfig,
}

fn parse_test262_args(args: &[String]) -> Result<ParsedTest262Args, String> {
    let mut config = SuiteConfig::default();
    let mut filter = None::<String>;
    let mut run_config = RunConfig::default();

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
