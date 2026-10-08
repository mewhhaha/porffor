//! Native bytes are admitted in the controller, arbitrary target work in workers.
use lila_test262::differential::{RobustnessCampaignPlan, RobustnessInput, RobustnessTarget,
    GeneratedCampaignCancellation, run_robustness_campaign, replay_robustness, SpecExecOracle,
    ArithmeticReductionLimit, minimize_robustness};
use std::path::PathBuf;

struct Arguments { input: PathBuf, output: PathBuf, target: RobustnessTarget, seed: u64, cases: usize,
    timeout_ms: u64, worker_bin: Option<PathBuf>, oracle: SpecExecOracle }
impl Arguments {
    fn parse(args: &[String]) -> Result<Self, String> {
        let (mut input, mut output, mut target, mut seed, mut cases, mut timeout, mut worker_bin, mut oracle) = (None, None, None, None, None, None, None, None);
        let mut index = 0;
        while index < args.len() {
            let flag = &args[index]; let value = args.get(index + 1).filter(|value| !value.is_empty() && !value.starts_with('-'))
                .ok_or_else(|| format!("{flag} needs a value"))?;
            match flag.as_str() {
                "--input" if input.is_none() => input = Some(PathBuf::from(value)),
                "--output-dir" if output.is_none() => output = Some(PathBuf::from(value)),
                "--target" if target.is_none() => target = Some(RobustnessTarget::from_name(value).ok_or_else(|| format!("unknown robustness target: {value}"))?),
                "--seed" if seed.is_none() => seed = Some(value.parse::<u64>().map_err(|_| "--seed needs a u64 integer")?),
                "--cases" if cases.is_none() => cases = Some(value.parse::<usize>().map_err(|_| "--cases needs an integer")?),
                "--timeout-ms" if timeout.is_none() => timeout = Some(value.parse::<u64>().map_err(|_| "--timeout-ms needs an integer")?),
                "--worker-bin" if worker_bin.is_none() => worker_bin = Some(PathBuf::from(value)),
                "--oracle" if oracle.is_none() => oracle = Some(super::parse_differential_oracle(value)?),
                "--input" | "--output-dir" | "--target" | "--seed" | "--cases" | "--timeout-ms" | "--worker-bin" | "--oracle" => return Err(format!("{flag} may only be specified once")),
                _ => return Err(format!("unknown robustness option: {flag}")),
            }
            index += 2;
        }
        Ok(Self { input: input.ok_or("robustness needs --input PATH")?, output: output.ok_or("robustness needs --output-dir PATH")?,
            target: target.ok_or("robustness needs --target NAME")?, seed: seed.ok_or("robustness needs --seed N")?,
            cases: cases.ok_or("robustness needs --cases N")?, timeout_ms: timeout.ok_or("robustness needs --timeout-ms N")?,
            worker_bin, oracle: oracle.ok_or("robustness needs explicit --oracle spec-exec to select the developer worker build")? })
    }
}
pub(super) fn run(args: &[String]) -> Result<(), String> {
    let arguments = Arguments::parse(args)?;
    let input = RobustnessInput::from_bytes_file("robustness/input", arguments.target, arguments.timeout_ms, &arguments.input)
        .map_err(|error| error.to_string())?;
    let plan = RobustnessCampaignPlan::new(input, arguments.seed, arguments.cases).map_err(|error| error.to_string())?;
    let runner = super::differential_worker_runner(arguments.worker_bin)?;
    let report = run_robustness_campaign(&plan, arguments.oracle, &runner,
        &GeneratedCampaignCancellation::default(), &arguments.output).map_err(|error| error.to_string())?;
    println!("{}", serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?);
    if report.completed_without_failure() { Ok(()) } else { Err("robustness attempt failed or remains incomplete; exact inputs and observations are retained".into()) }
}
pub(super) fn replay(args: &[String]) -> Result<(), String> {
    let arguments = super::parse_differential_replay_args(args)?;
    let input = RobustnessInput::load(&arguments.case_path).map_err(|error| error.to_string())?;
    let runner = super::differential_worker_runner(arguments.worker_bin)?;
    let observation = replay_robustness(&input, arguments.oracle, &runner).map_err(|error| error.to_string())?;
    println!("{}", serde_json::to_string_pretty(&observation).map_err(|error| error.to_string())?);
    if observation.completed_without_failure() { Ok(()) } else { Err("robustness replay failed; see the exact worker stages/result above".into()) }
}

struct ReductionArguments { input: PathBuf, output: PathBuf, limit: ArithmeticReductionLimit,
    worker_bin: Option<PathBuf>, oracle: SpecExecOracle }
impl ReductionArguments {
    fn parse(args: &[String]) -> Result<Self, String> {
        let input = args.first().filter(|path| !path.is_empty() && !path.starts_with('-'))
            .ok_or("minimize-robustness needs a native input.json path")?;
        let (mut output, mut limit, mut worker_bin, mut oracle) = (None, None, None, None);
        let mut index = 1;
        while index < args.len() {
            let flag = &args[index]; let value = args.get(index + 1).filter(|value| !value.is_empty() && !value.starts_with('-'))
                .ok_or_else(|| format!("{flag} needs a value"))?;
            match flag.as_str() {
                "--output-dir" if output.is_none() => output = Some(PathBuf::from(value)),
                "--replays" if limit.is_none() => limit = Some(ArithmeticReductionLimit::new(value.parse::<usize>()
                    .map_err(|_| "--replays needs an integer")?).map_err(|error| error.to_string())?),
                "--worker-bin" if worker_bin.is_none() => worker_bin = Some(PathBuf::from(value)),
                "--oracle" if oracle.is_none() => oracle = Some(super::parse_differential_oracle(value)?),
                "--output-dir" | "--replays" | "--worker-bin" | "--oracle" => return Err(format!("{flag} may only be specified once")),
                _ => return Err(format!("unknown robustness reduction option: {flag}")),
            }
            index += 2;
        }
        Ok(Self { input: PathBuf::from(input), output: output.ok_or("minimize-robustness needs --output-dir PATH")?,
            limit: limit.ok_or("minimize-robustness needs --replays N")?, worker_bin,
            oracle: oracle.ok_or("minimize-robustness needs explicit --oracle spec-exec")? })
    }
}
pub(super) fn minimize(args: &[String]) -> Result<(), String> {
    let arguments = ReductionArguments::parse(args)?;
    // Strict native admission happens before opening the selected worker.
    let input = RobustnessInput::load(&arguments.input).map_err(|error| error.to_string())?;
    let runner = super::differential_worker_runner(arguments.worker_bin)?;
    let report = minimize_robustness(&input, arguments.limit, arguments.oracle, &runner,
        &GeneratedCampaignCancellation::default(), &arguments.output).map_err(|error| error.to_string())?;
    println!("{}", serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?);
    // A reduction never repairs its crash or publishes executable corpus green.
    Err("robustness reduction retains failed or non-reducible evidence; see the exact replay artifact and decisions above".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_ir_target_uses_explicit_worker_policy_without_source_goal_overrides() {
        let args = ["--input", "native.bin", "--output-dir", "fresh", "--target", "ir-admission", "--seed", "19", "--cases", "8", "--timeout-ms", "1000", "--oracle", "spec-exec"].map(str::to_string);
        let parsed = Arguments::parse(&args).unwrap();
        assert_eq!(parsed.target, RobustnessTarget::IrAdmission {});
        assert_eq!(parsed.input, PathBuf::from("native.bin"));
        assert_eq!((parsed.seed, parsed.cases, parsed.timeout_ms), (19, 8, 1000));
        assert!(Arguments::parse(&args[..12]).is_err());
        for (flag, value) in [("--goal", "module"), ("--target", "script")] {
            let mut overridden = args.to_vec();
            overridden.extend([flag.into(), value.into()]);
            assert!(Arguments::parse(&overridden).is_err());
        }
    }
    #[test]
    fn closed_targets_and_explicit_budgets_refuse_duplicate_or_foreign_flags() {
        let args = ["--input", "source", "--output-dir", "fresh", "--target", "script", "--seed", "1", "--cases", "2", "--timeout-ms", "1000", "--oracle", "spec-exec"].map(str::to_string);
        assert!(Arguments::parse(&args).is_ok());
        let mut foreign = args.to_vec(); foreign[5] = "uri".into(); assert!(Arguments::parse(&foreign).is_err());
        let mut duplicate = args.to_vec(); duplicate.extend(["--seed".into(), "2".into()]); assert!(Arguments::parse(&duplicate).is_err());
        assert!(Arguments::parse(&args[..12]).is_err());
    }
    #[test]
    fn bounded_uri_prelude_and_filesystem_targets_use_the_original_selected_worker_route() {
        for target in ["encode-uri", "encode-uri-component", "decode-uri", "decode-uri-component", "prelude", "filesystem-resolver"] {
            let args = ["--input", "payload.json", "--output-dir", "fresh", "--target", target, "--seed", "1", "--cases", "2", "--timeout-ms", "1000", "--oracle", "spec-exec"].map(str::to_string);
            let parsed = Arguments::parse(&args).unwrap();
            assert_eq!(parsed.target, RobustnessTarget::from_name(target).unwrap());
            assert_eq!(parsed.cases, 2);
            let mut changed = args.to_vec(); changed.extend(["--suite-root".into(), "/ambient".into()]);
            assert!(Arguments::parse(&changed).is_err());
        }
    }
    #[test]
    fn minimizer_uses_checked_replay_budget_and_cannot_override_original_target_or_deadline() {
        let args = ["repro.input.json", "--output-dir", "fresh", "--replays", "8", "--oracle", "spec-exec"].map(str::to_string);
        assert_eq!(ReductionArguments::parse(&args).unwrap().limit.get(), 8);
        for value in ["0", "513", "not-an-integer"] {
            let mut invalid = args.to_vec(); invalid[4] = value.into(); assert!(ReductionArguments::parse(&invalid).is_err());
        }
        for flag in ["--target", "--timeout-ms", "--replays"] {
            let mut invalid = args.to_vec(); invalid.extend([flag.into(), "1".into()]); assert!(ReductionArguments::parse(&invalid).is_err());
        }
        assert!(ReductionArguments::parse(&args[..5]).is_err());
    }
}
