//! The release command executes fresh evidence rather than accepting names.

use std::num::NonZeroUsize;
use std::path::PathBuf;

use lila_test262::{
    conformance_closure::run_closure_with_workers, LocalHarnessSource, SuiteConfig,
};

fn config_for_args(args: &[String]) -> Result<(SuiteConfig, NonZeroUsize), String> {
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
    let mut supplied_threads = false;
    let mut workers = NonZeroUsize::MIN;
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
            "--threads" => {
                if supplied_threads {
                    return Err("closure accepts --threads once".into());
                }
                index += 1;
                let value = args
                    .get(index)
                    .ok_or("closure --threads requires a positive integer")?;
                if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
                    return Err("closure --threads requires a positive integer".into());
                }
                workers = value
                    .parse::<NonZeroUsize>()
                    .map_err(|_| "closure --threads requires a positive integer within usize")?;
                supplied_threads = true;
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
                    "unsupported closure option `{option}`; close-release always executes two fresh, full pinned Wasm-AOT runs with the standard timeout and an explicitly bounded worker count"
                ));
            }
        }
        index += 1;
    }
    Ok((config, workers))
}

pub(super) fn run(args: &[String]) -> Result<(), String> {
    let (config, workers) = config_for_args(args)?;
    let report = run_closure_with_workers(config, workers)?;
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
    fn explicit_case_workers_compose_with_reuse_without_changing_other_policy() {
        for arguments in [
            vec!["--threads", "4", "--reuse-case-workers"],
            vec!["--reuse-case-workers", "--threads", "4"],
        ] {
            let args = arguments.into_iter().map(String::from).collect::<Vec<_>>();
            let (config, workers) = config_for_args(&args).unwrap();
            assert_eq!(workers.get(), 4);
            assert!(config.reuse_case_workers);
            // Only the explicit native workers API may alter case scheduling.
            assert_eq!(config.worker_count, 1);
            assert_eq!(config.timeout_ms, 60_000);
            assert_eq!(config.local_harness, LocalHarnessSource::EmbeddedWasmAot);
            assert_eq!(
                config.case_runner_bin,
                Some(std::env::current_exe().unwrap())
            );
        }
        for arguments in [
            vec!["--threads"],
            vec!["--threads", "0"],
            vec!["--threads", "-1"],
            vec!["--threads", "+4"],
            vec!["--threads", "1.5"],
            vec!["--threads", "999999999999999999999999999999"],
            vec!["--threads", "2", "--threads", "3"],
        ] {
            let args = arguments.into_iter().map(String::from).collect::<Vec<_>>();
            assert!(config_for_args(&args).is_err());
        }
    }

    #[test]
    fn worker_reuse_is_explicit_and_preserves_full_serial_closure_policy() {
        let (default, workers) = config_for_args(&[]).unwrap();
        assert_eq!(workers.get(), 1);
        assert!(!default.reuse_case_workers);
        let (config, workers) = config_for_args(&[
            "--reuse-case-workers".into(),
            "--snapshot-dir".into(),
            "explicit evidence".into(),
        ])
        .unwrap();
        assert!(config.reuse_case_workers);
        assert_eq!(workers.get(), 1);
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
        for forbidden in ["--timeout-ms", "--resume", "--matrix-node"] {
            assert!(config_for_args(&["--reuse-case-workers".into(), forbidden.into()]).is_err());
        }
    }
}
