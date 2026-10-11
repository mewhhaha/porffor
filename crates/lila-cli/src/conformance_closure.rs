//! The release command executes fresh evidence rather than accepting names.

use std::path::PathBuf;

use lila_test262::{conformance_closure::run_closure, LocalHarnessSource, SuiteConfig};

fn config_for_args(args: &[String]) -> Result<SuiteConfig, String> {
    let mut config = SuiteConfig::default();
    config.local_harness = LocalHarnessSource::EmbeddedWasmAot;
    config.worker_count = 1;
    config.case_runner_bin = Some(
        std::env::current_exe()
            .map_err(|error| format!("cannot select the running closure case worker: {error}"))?,
    );
    let mut index = 0;
    let mut supplied_snapshot_dir = false;
    let mut supplied_reuse = false;
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
            "--reuse-case-workers" => {
                if supplied_reuse {
                    return Err("closure accepts --reuse-case-workers once".into());
                }
                config.reuse_case_workers = true;
                supplied_reuse = true;
            }
            option => {
                return Err(format!(
                    "unsupported closure option `{option}`; close-release always executes two fresh, full pinned Wasm-AOT runs with the standard timeout and one worker"
                ));
            }
        }
        index += 1;
    }
    Ok(config)
}

pub(super) fn run(args: &[String]) -> Result<(), String> {
    let config = config_for_args(args)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worker_reuse_is_explicit_and_preserves_full_serial_closure_policy() {
        let default = config_for_args(&[]).unwrap();
        assert!(!default.reuse_case_workers);
        let config = config_for_args(&[
            "--reuse-case-workers".into(),
            "--snapshot-dir".into(),
            "explicit evidence".into(),
        ])
        .unwrap();
        assert!(config.reuse_case_workers);
        assert_eq!(config.snapshot_dir, PathBuf::from("explicit evidence"));
        assert_eq!(config.worker_count, 1);
        assert_eq!(config.timeout_ms, SuiteConfig::default().timeout_ms);
        assert_eq!(config.local_harness, LocalHarnessSource::EmbeddedWasmAot);
        assert_eq!(
            config.case_runner_bin,
            Some(std::env::current_exe().unwrap())
        );
        assert!(
            config_for_args(&["--reuse-case-workers".into(), "--reuse-case-workers".into(),])
                .is_err()
        );
        for forbidden in ["--threads", "--timeout-ms", "--resume", "--matrix-node"] {
            assert!(config_for_args(&["--reuse-case-workers".into(), forbidden.into()]).is_err());
        }
    }
}
