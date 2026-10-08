//! Private whole-corpus CLI route; worker selection remains the product seam.

use lila_test262::differential::SpecExecOracle;
use std::path::PathBuf;

struct Arguments {
    output: PathBuf,
    corpus_root: Option<PathBuf>,
    worker_bin: Option<PathBuf>,
    oracle: SpecExecOracle,
}

impl Arguments {
    fn parse(args: &[String]) -> Result<Self, String> {
        let mut output = None;
        let mut corpus_root = None;
        let mut worker_bin = None;
        let mut oracle = None;
        let mut index = 0;
        while index < args.len() {
            let option = &args[index];
            let value = args
                .get(index + 1)
                .filter(|value| !value.is_empty() && !value.starts_with('-'))
                .ok_or_else(|| format!("{option} needs a value"))?;
            match option.as_str() {
                "--output-dir" if output.is_none() => output = Some(PathBuf::from(value)),
                "--corpus-root" if corpus_root.is_none() => {
                    corpus_root = Some(PathBuf::from(value))
                }
                "--worker-bin" if worker_bin.is_none() => worker_bin = Some(PathBuf::from(value)),
                "--oracle" if oracle.is_none() => {
                    oracle = Some(super::parse_differential_oracle(value)?)
                }
                "--output-dir" | "--corpus-root" | "--worker-bin" | "--oracle" => {
                    return Err(format!("{option} may only be specified once"))
                }
                _ => {
                    return Err(format!(
                        "unknown differential replay-corpus option: {option}"
                    ))
                }
            }
            index += 2;
        }
        Ok(Self {
            output: output
                .ok_or_else(|| "differential replay-corpus needs --output-dir PATH".to_owned())?,
            corpus_root,
            worker_bin,
            oracle: oracle.ok_or_else(|| {
                "differential replay-corpus requires explicit --oracle spec-exec".to_owned()
            })?,
        })
    }
}

pub(super) fn run(args: &[String]) -> Result<(), String> {
    let arguments = Arguments::parse(args)?;
    #[cfg(not(feature = "spec-exec-oracle"))]
    {
        let _ = arguments;
        Err(lila_test262::differential::DifferentialError::OracleNotLinked.to_string())
    }
    #[cfg(feature = "spec-exec-oracle")]
    {
        use lila_test262::differential::{replay_corpus, DifferentialCorpus};
        let runner = super::differential_worker_runner(arguments.worker_bin)?;
        let corpus = match arguments.corpus_root {
            Some(path) => {
                DifferentialCorpus::from_directory(&path).map_err(|error| error.to_string())?
            }
            None => DifferentialCorpus::compiled(),
        };
        let report = replay_corpus(corpus, arguments.oracle, &runner, &arguments.output)
            .map_err(|error| error.to_string())?;
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
        );
        if report.is_green() {
            Ok(())
        } else {
            Err(format!(
                "differential corpus retains {} failed cases out of {} in {}",
                report.failed(),
                report.total(),
                arguments.output.display()
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parse(args: &[&str]) -> Result<Arguments, String> {
        Arguments::parse(
            &args
                .iter()
                .map(|value| (*value).to_owned())
                .collect::<Vec<_>>(),
        )
    }
    #[test]
    fn whole_corpus_requires_explicit_output_and_oracle() {
        assert!(parse(&["--output-dir", "reports"]).is_err());
        assert!(parse(&["--oracle", "spec-exec"]).is_err());
        assert!(parse(&["--output-dir", "reports", "--oracle", "wasm-aot"]).is_err());
        let args = parse(&["--output-dir", "reports", "--oracle", "spec-exec"]).unwrap();
        assert_eq!(args.output, PathBuf::from("reports"));
        assert_eq!(args.oracle, SpecExecOracle::explicitly_enabled());
    }
    #[test]
    fn whole_corpus_has_one_inventory_and_no_subset_switch() {
        for extra in [
            ["--filter", "one"],
            ["--output-dir", "other"],
            ["--oracle", "spec-exec"],
        ] {
            let mut args = vec!["--output-dir", "reports", "--oracle", "spec-exec"];
            args.extend(extra);
            assert!(parse(&args).is_err());
        }
        let args = parse(&[
            "--output-dir",
            "reports",
            "--corpus-root",
            "cases",
            "--worker-bin",
            "worker",
            "--oracle",
            "spec-exec",
        ])
        .unwrap();
        assert_eq!(args.corpus_root, Some(PathBuf::from("cases")));
        assert_eq!(args.worker_bin, Some(PathBuf::from("worker")));
    }
}
