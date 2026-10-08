//! The release command executes fresh evidence rather than accepting names.

use std::path::PathBuf;

use lila_test262::{conformance_closure::run_closure, LocalHarnessSource, SuiteConfig};

pub(super) fn run(args: &[String]) -> Result<(), String> {
    let mut config = SuiteConfig::default();
    config.local_harness = LocalHarnessSource::EmbeddedWasmAot;
    config.worker_count = 1;
    config.case_runner_bin = Some(
        std::env::current_exe()
            .map_err(|error| format!("cannot select the running closure case worker: {error}"))?,
    );
    let mut index = 0;
    let mut supplied_snapshot_dir = false;
    while index < args.len() {
        match args[index].as_str() {
            "--snapshot-dir" => {
                if supplied_snapshot_dir {
                    return Err("closure accepts --snapshot-dir once".into());
                }
                index += 1;
                let value = args
                    .get(index)
                    .filter(|value| !value.is_empty() && !value.starts_with("--"))
                    .ok_or("closure --snapshot-dir requires a path")?;
                config.snapshot_dir = PathBuf::from(value);
                supplied_snapshot_dir = true;
            }
            option => {
                return Err(format!(
                    "unsupported closure option `{option}`; close-release always executes two fresh, full pinned Wasm-AOT runs with the standard timeout and one worker"
                ));
            }
        }
        index += 1;
    }
    let report = run_closure(config)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&report)
            .map_err(|error| { format!("cannot render closure verdict: {error}") })?
    );
    if report.is_passed() {
        Ok(())
    } else {
        Err(format!(
            "Test262 release closure rejected; complete evidence retained in {}",
            report.report_path().display()
        ))
    }
}
