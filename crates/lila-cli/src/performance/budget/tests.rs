//! Synthetic admitted distributions test budget policy, not benchmark speed.
use super::*;

fn identity() -> ProfileIdentity {
    ProfileIdentity {
        compiler: CompilerProvenance::current().unwrap(),
        build: Build { rustc_verbose: "fixture-rustc".into(), target: "fixture-target".into(), host: "fixture-host".into(),
            profile: "fixture".into(), opt_level: "0".into(), debug: "true".into(), rustflags: "".into(), spec_exec_oracle_linked: false },
        platform: Platform { os: "fixture-os".into(), architecture: "fixture-arch".into(), kernel: "fixture-kernel".into(),
            cpu_model: "fixture-cpu".into(), logical_cpus_available: 1, physical_memory: "4096MiB".into(),
            process_cpu_affinity: Some("1".into()), cgroup_limits: vec![], machine_label: "fixture-idle".into() },
        configuration: Configuration { backend: "wasm-aot-compiler-stages".into(), fixture_root: "/fixture".into(),
            working_directory: "/fixture".into(), host_surface: "default".into(), compilation_jobs: 1,
            cache_paths_and_limits: vec![], environment: vec![], idle_machine_acknowledged: true },
        corpus_sha256: digest(b"fixture corpus"),
        fixtures: vec![Fixture { name: "fixture.js".into(), bytes: 2, sha256: digest(b"0;") }], samples_per_case: 5,
    }
}
fn timings(values: &[u64]) -> AdmittedMetrics {
    let mut identity = identity(); identity.samples_per_case = values.len();
    let mut result = AdmittedMetrics::new(identity);
    result.timing("fixture.js", Metric::CompilerPrepare, &stats::Summary::from_samples(values).unwrap()).unwrap();
    result
}
fn policy(statistic: Statistic, absolute: Option<u64>, relative: Option<u32>) -> CheckedPolicy {
    let wire = PolicyWire { version: 1, minimum_samples: 3, rules: vec![Rule {
        fixture: "fixture.js".into(), metric: Metric::CompilerPrepare, statistic,
        absolute_maximum: absolute, maximum_increase_basis_points: relative,
    }] };
    CheckedPolicy::from_bytes(&serde_json::to_vec(&wire).unwrap()).unwrap()
}

#[test]
fn exact_percentages_keep_zero_baselines_boundaries_and_large_values() {
    let baseline = timings(&[10; 5]); let candidate = timings(&[11; 5]);
    assert!(policy(Statistic::Median, Some(11), Some(1000)).evaluate(&baseline, &candidate).unwrap()[0].passed);
    let denied = policy(Statistic::Median, Some(11), Some(999)).evaluate(&baseline, &candidate).unwrap();
    assert_eq!(denied[0].absolute_passed, Some(true)); assert_eq!(denied[0].relative_passed, Some(false));
    assert!(!denied[0].passed);
    assert!(!policy(Statistic::Median, None, Some(1_000_000)).evaluate(&timings(&[0; 5]), &timings(&[1; 5])).unwrap()[0].passed);
    assert!(policy(Statistic::Median, None, Some(0)).evaluate(&timings(&[0; 5]), &timings(&[0; 5])).unwrap()[0].passed);
    assert!(policy(Statistic::P95, Some(u64::MAX), Some(1_000_000))
        .evaluate(&timings(&[u64::MAX; 5]), &timings(&[u64::MAX; 5])).unwrap()[0].passed);
}

#[test]
fn outliers_are_retained_without_replacing_the_selected_statistic() {
    let before = timings(&[10, 10, 10, 10, 1000]);
    let after = timings(&[10, 11, 10, 10, 2000]);
    assert!(policy(Statistic::Median, None, Some(0)).evaluate(&before, &after).unwrap()[0].passed);
    assert!(!policy(Statistic::P95, None, Some(0)).evaluate(&before, &after).unwrap()[0].passed);
    let strict = CheckedPolicy(PolicyWire { version: 1, minimum_samples: 6, rules: policy(Statistic::Median, Some(20), None).0.rules });
    assert!(strict.evaluate(&before, &after).is_err());
}

#[test]
fn wrong_workload_hardware_build_dimensions_and_unavailable_metrics_refuse() {
    let before = timings(&[10; 5]);
    let mutations: &[fn(&mut AdmittedMetrics)] = &[
        |after| after.identity.platform.cpu_model.push('!'),
        |after| after.identity.platform.machine_label.push('!'),
        |after| after.identity.build.opt_level = "3".into(),
        |after| after.identity.configuration.compilation_jobs = 2,
        |after| after.identity.configuration.idle_machine_acknowledged = false,
        |after| after.identity.corpus_sha256 = digest(b"other workload"),
        |after| { after.values.clear(); },
    ];
    for mutate in mutations {
        let mut after = timings(&[10; 5]); mutate(&mut after);
        assert!(policy(Statistic::Median, Some(10), None).evaluate(&before, &after).is_err());
    }
    let mut after = timings(&[10; 5]);
    let mut compiler = serde_json::to_value(&after.identity.compiler).unwrap();
    compiler["executable_sha256"] = "1".repeat(64).into();
    after.identity.compiler = serde_json::from_value(compiler).unwrap();
    assert!(policy(Statistic::Median, Some(10), None).evaluate(&before, &after).unwrap()[0].passed);
    assert!(after.insert("fixture.js", Metric::CompilerPrepare, Statistic::Exact, 10).is_err());
    assert!(after.insert("foreign.js", Metric::ModuleBytes, Statistic::Exact, 10).is_err());
}

#[test]
fn malformed_empty_duplicate_and_unmeasured_policies_are_not_silent_skips() {
    let valid = serde_json::to_value(&policy(Statistic::Median, Some(10), None).0).unwrap();
    let mutations: &[fn(&mut serde_json::Value)] = &[
        |value| value["version"] = 2.into(),
        |value| value["minimum_samples"] = 1.into(),
        |value| value["rules"] = serde_json::json!([]),
        |value| value["rules"][0]["metric"] = "allocation-rate".into(),
        |value| value["rules"][0]["statistic"] = "exact".into(),
        |value| value["rules"][0]["absolute_maximum"] = serde_json::Value::Null,
        |value| value["rules"][0]["maximum_increase_basis_points"] = 1_000_001.into(),
        |value| { let rule = value["rules"][0].clone(); value["rules"].as_array_mut().unwrap().push(rule); },
        |value| value["ignore_missing"] = true.into(),
    ];
    for mutate in mutations {
        let mut damaged = valid.clone(); mutate(&mut damaged);
        assert!(CheckedPolicy::from_bytes(&serde_json::to_vec(&damaged).unwrap()).is_err());
    }
}
