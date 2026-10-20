//! Feature-required executable for isolated developer differential replay.

use scriptw_test262::differential::{run_differential_worker, SpecExecOracle};
use std::ffi::OsString;
use std::path::PathBuf;

fn expect(args: &mut impl Iterator<Item = OsString>, literal: &str) -> Result<(), String> {
    if args.next().as_deref() == Some(std::ffi::OsStr::new(literal)) {
        Ok(())
    } else {
        Err(format!("worker requires {literal}"))
    }
}

fn run() -> Result<(), String> {
    let mut args = std::env::args_os().skip(1);
    expect(&mut args, "--jobs")?;
    let jobs = args
        .next()
        .and_then(|value| value.into_string().ok())
        .and_then(|value| value.parse::<std::num::NonZeroUsize>().ok())
        .ok_or("worker --jobs must be a positive integer")?;
    expect(&mut args, "differential")?;
    expect(&mut args, "__worker")?;
    expect(&mut args, "--request")?;
    let request = PathBuf::from(args.next().ok_or("worker request path is missing")?);
    expect(&mut args, "--journal")?;
    let journal = PathBuf::from(args.next().ok_or("worker journal path is missing")?);
    expect(&mut args, "--oracle")?;
    expect(&mut args, "spec-exec")?;
    if args.next().is_some() {
        return Err("worker refuses extra arguments".into());
    }
    let oracle = SpecExecOracle::explicitly_enabled();
    run_differential_worker(&request, &journal, oracle, jobs.get())
        .map_err(|error| error.to_string())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
