use super::*;

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

#[test]
fn shard_option_is_explicit_positive_unique_and_independent_of_filter_order() {
    for args in [
        strings(&["--shard", "2/4", "1/2"]),
        strings(&["1/2", "--shard", "2/4"]),
    ] {
        let parsed = parse_test262_args(&args).unwrap();
        assert_eq!(parsed.filter.as_deref(), Some("1/2"));
        assert_eq!(
            (parsed.run_config.shard_index, parsed.run_config.shard_count),
            (1, 4)
        );
        assert!(Test262Command::Shard.admit(parsed).is_ok());
    }
    let filter = parse_test262_args(&strings(&["1/2"])).unwrap();
    assert_eq!(filter.filter.as_deref(), Some("1/2"));
    assert!(Test262Command::Shard.admit(filter).is_err());
    for bad in ["0/1", "1/0", "0/0", "3/2", "1/2/3", "x/2", "1", ""] {
        assert!(
            parse_test262_args(&strings(&["--shard", bad])).is_err(),
            "{bad}"
        );
    }
    assert!(parse_test262_args(&strings(&["--shard", "1/2", "--shard", "2/2"])).is_err());
}

#[test]
fn options_are_admitted_only_by_commands_that_use_them() {
    for (command, args) in [
        (Test262Command::List, vec!["--threads", "1"]),
        (Test262Command::Sync, vec!["unused-filter"]),
        (Test262Command::Run, vec!["--shard", "1/2"]),
        (Test262Command::Run, vec!["--readme-path", "README.md"]),
        (Test262Command::Run, vec!["--max-matrix-nodes", "1"]),
        (Test262Command::ProgressStatus, vec!["--resume"]),
        (Test262Command::TriageStatus, vec!["--timeout-ms", "20"]),
        (
            Test262Command::PublishStatus,
            vec!["--max-matrix-nodes", "1"],
        ),
        (Test262Command::RunList, vec!["--matrix-node", "a"]),
    ] {
        let parsed = parse_test262_args(&strings(&args)).unwrap();
        assert!(command.admit(parsed).is_err(), "{command:?} {args:?}");
    }
    for (command, args) in [
        (Test262Command::List, vec!["1/2", "--suite-root", "missing"]),
        (
            Test262Command::Run,
            vec!["--matrix-node", "language/expressions", "--resume"],
        ),
        (Test262Command::ReportAll, vec!["--max-matrix-nodes", "1"]),
        (
            Test262Command::PublishStatus,
            vec!["--readme-path", "README.md"],
        ),
        (
            Test262Command::ProgressStatus,
            vec!["--snapshot-name", "existing"],
        ),
        (Test262Command::FailureDetails, vec!["language"]),
    ] {
        assert!(
            command
                .admit(parse_test262_args(&strings(&args)).unwrap())
                .is_ok(),
            "{command:?} {args:?}"
        );
    }
}

#[test]
fn option_values_are_consumed_once_including_flag_like_worker_paths() {
    let parsed = parse_test262_case_worker_args(&strings(&[
        "raw-script:fixture.js",
        "--snapshot-name",
        "--case-harness",
        "--case-harness",
        "none",
    ]))
    .unwrap();
    assert_eq!(parsed.run_config.snapshot_name, "--case-harness");
    assert_eq!(parsed.config.local_harness, LocalHarnessSource::None);
    assert!(parse_test262_args(&strings(&["--threads"]))
        .unwrap_err()
        .contains("needs a value"));
    assert!(parse_test262_args(&strings(&["--threads", ""]))
        .unwrap_err()
        .contains("invalid"));
    assert!(parse_test262_args(&strings(&["--resume", "--resume"]))
        .unwrap_err()
        .contains("only be specified once"));
    assert!(
        parse_test262_case_worker_args(&strings(&["--case-harness-file"]))
            .unwrap_err()
            .contains("requires a value")
    );
}

#[test]
fn global_options_are_consumed_in_source_order_without_reordering_arguments() {
    let mut args = strings(&[
        "run",
        "--jobs",
        "1",
        "input.js",
        "--intl-profile",
        "minimal",
        "--host-surface",
        "test262",
        "--module",
    ]);
    let parsed = take_global_options(&mut args).unwrap();
    assert_eq!(args, strings(&["run", "input.js", "--module"]));
    assert_eq!(parsed.jobs, Some(1));
    assert_eq!(parsed.host_surface_policy, HostSurfacePolicy::Test262);
    assert!(matches!(
        parsed.intl_profile,
        Some(IntlCompilationProfile::Minimal)
    ));
    let error = take_global_options(&mut strings(&["--intl-profile", "--jobs", "1"]))
        .err()
        .unwrap();
    assert!(
        error.contains("unknown --intl-profile value: --jobs"),
        "{error}"
    );
}
