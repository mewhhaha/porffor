//! Exact Test262 execution seeds remain separate from primitive corpus inputs.
use lila_test262::differential::{
    replay_test262_seed, SpecExecOracle, Test262ReplaySeed, Test262SeedPlan, Test262SeedSelection,
};
use lila_test262::{LocalHarnessSource, SuiteConfig};
use std::io::Write;
use std::path::PathBuf;

struct SeedArgs {
    config: SuiteConfig,
    base: String,
    candidate: String,
    selection: Test262SeedSelection,
    limit: usize,
    output: PathBuf,
}
impl SeedArgs {
    fn parse(args: &[String]) -> Result<Self, String> {
        let (
            mut base,
            mut candidate,
            mut selection,
            mut limit,
            mut output,
            mut suite,
            mut snapshots,
            mut timeout,
        ) = (None, None, None, None, None, None, None, None);
        let mut index = 0;
        while index < args.len() {
            let flag = args[index].as_str();
            let value = args
                .get(index + 1)
                .filter(|s| !s.is_empty() && !s.starts_with('-'))
                .ok_or_else(|| format!("{flag} needs a value"))?;
            match flag {
                "--base" if base.is_none() => base = Some(value.clone()),
                "--candidate" if candidate.is_none() => candidate = Some(value.clone()),
                "--selection" if selection.is_none() => {
                    selection = Some(match value.as_str() {
                        "failures" => Test262SeedSelection::CandidateFailures,
                        "newly-green" => Test262SeedSelection::NewlyGreen,
                        _ => return Err("--selection must be failures or newly-green".into()),
                    })
                }
                "--max-seeds" if limit.is_none() => {
                    limit = Some(
                        value
                            .parse::<usize>()
                            .map_err(|_| "--max-seeds needs an integer")?,
                    )
                }
                "--output-dir" if output.is_none() => output = Some(PathBuf::from(value)),
                "--suite-root" if suite.is_none() => suite = Some(PathBuf::from(value)),
                "--snapshot-dir" if snapshots.is_none() => snapshots = Some(PathBuf::from(value)),
                "--timeout-ms" if timeout.is_none() => {
                    timeout = Some(
                        value
                            .parse::<u64>()
                            .map_err(|_| "--timeout-ms needs an integer")?,
                    )
                }
                "--base" | "--candidate" | "--selection" | "--max-seeds" | "--output-dir"
                | "--suite-root" | "--snapshot-dir" | "--timeout-ms" => {
                    return Err(format!("{flag} may only be specified once"))
                }
                _ => return Err(format!("unknown Test262 seed option: {flag}")),
            }
            index += 2;
        }
        let mut config = SuiteConfig::default();
        config.local_harness = LocalHarnessSource::EmbeddedWasmAot;
        if let Some(suite) = suite {
            config.suite_root = suite;
        }
        if let Some(snapshots) = snapshots {
            config.snapshot_dir = snapshots;
        }
        if let Some(timeout) = timeout {
            config.timeout_ms = timeout;
        }
        Ok(Self {
            config,
            base: base.ok_or("seed-test262 needs --base NAME")?,
            candidate: candidate.ok_or("seed-test262 needs --candidate NAME")?,
            selection: selection.ok_or("seed-test262 needs --selection failures|newly-green")?,
            limit: limit.ok_or("seed-test262 needs --max-seeds N")?,
            output: output.ok_or("seed-test262 needs --output-dir PATH")?,
        })
    }
}

pub(super) fn seed(args: &[String]) -> Result<(), String> {
    let arguments = SeedArgs::parse(args)?;
    let plan = Test262SeedPlan::new(
        &arguments.config,
        &arguments.base,
        &arguments.candidate,
        arguments.selection,
        arguments.limit,
    )
    .map_err(|e| e.to_string())?;
    plan.write(&arguments.output).map_err(|e| e.to_string())?;
    println!(
        "retained {} exact execution seeds of {} eligible snapshot executions in {}",
        plan.seeds().len(),
        plan.eligible(),
        arguments.output.display()
    );
    Ok(())
}

struct ReplayArgs {
    seed: PathBuf,
    output: PathBuf,
    oracle: SpecExecOracle,
    worker: Option<PathBuf>,
}
impl ReplayArgs {
    fn parse(args: &[String]) -> Result<Self, String> {
        let (mut seed, mut output, mut oracle, mut worker) = (None, None, None, None);
        let mut index = 0;
        while index < args.len() {
            let flag = args[index].as_str();
            let value = args
                .get(index + 1)
                .filter(|s| !s.is_empty() && !s.starts_with('-'))
                .ok_or_else(|| format!("{flag} needs a value"))?;
            match flag {
                "--seed" if seed.is_none() => seed = Some(PathBuf::from(value)),
                "--output-report" if output.is_none() => output = Some(PathBuf::from(value)),
                "--oracle" if oracle.is_none() => {
                    oracle = Some(super::parse_differential_oracle(value)?)
                }
                "--worker-bin" if worker.is_none() => worker = Some(PathBuf::from(value)),
                "--seed" | "--output-report" | "--oracle" | "--worker-bin" => {
                    return Err(format!("{flag} may only be specified once"))
                }
                _ => return Err(format!("unknown Test262 replay option: {flag}")),
            }
            index += 2;
        }
        Ok(Self {
            seed: seed.ok_or("replay-test262 needs --seed PATH")?,
            output: output.ok_or("replay-test262 needs --output-report PATH")?,
            oracle: oracle.ok_or("replay-test262 requires explicit --oracle spec-exec")?,
            worker,
        })
    }
}
pub(super) fn replay(args: &[String]) -> Result<(), String> {
    let arguments = ReplayArgs::parse(args)?;
    let seed = Test262ReplaySeed::load(&arguments.seed).map_err(|e| e.to_string())?;
    let runner = super::differential_worker_runner(arguments.worker)?;
    // Own a fresh durable result location before running arbitrary source.
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&arguments.output)
        .map_err(|e| e.to_string())?;
    file.write_all(b"{\"state\":\"incomplete\"}\n")
        .and_then(|()| file.sync_all())
        .map_err(|e| e.to_string())?;
    let report =
        replay_test262_seed(&seed, arguments.oracle, &runner).map_err(|e| e.to_string())?;
    let bytes = serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?;
    let mut pending_name = arguments
        .output
        .file_name()
        .ok_or("report needs a file name")?
        .to_os_string();
    pending_name.push(".pending");
    let pending = arguments.output.with_file_name(pending_name);
    let mut complete = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&pending)
        .map_err(|e| e.to_string())?;
    complete
        .write_all(&bytes)
        .and_then(|()| complete.sync_all())
        .map_err(|e| e.to_string())?;
    drop(complete);
    drop(file);
    std::fs::rename(pending, &arguments.output).map_err(|e| e.to_string())?;
    println!("{}", String::from_utf8(bytes).map_err(|e| e.to_string())?);
    if report.is_green() {
        Ok(())
    } else {
        Err("Test262 replay retains failed, unsupported or worker observations; matching red is not green".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bridge_cli_requires_exact_selection_and_durable_output() {
        let args = [
            "--base",
            "old",
            "--candidate",
            "new",
            "--selection",
            "newly-green",
            "--max-seeds",
            "4",
            "--output-dir",
            "fresh",
        ]
        .map(str::to_string);
        assert!(SeedArgs::parse(&args).is_ok());
        let mut unknown = args.to_vec();
        unknown[5] = "all-green".into();
        assert!(SeedArgs::parse(&unknown).is_err());
        let mut duplicate = args.to_vec();
        duplicate.extend(["--base".into(), "other".into()]);
        assert!(SeedArgs::parse(&duplicate).is_err());
        let replay = [
            "--seed",
            "seed.json",
            "--output-report",
            "report.json",
            "--oracle",
            "spec-exec",
        ]
        .map(str::to_string);
        assert!(ReplayArgs::parse(&replay).is_ok());
        assert!(ReplayArgs::parse(&replay[..4]).is_err());
    }
}
