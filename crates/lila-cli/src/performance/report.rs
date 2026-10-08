//! Closed report admission preserves workload, spans and raw measurement evidence.
use super::stats::{Change, Summary};
use lila_test262::CompilerProvenance;
use serde::{Deserialize, Serialize};
pub(super) const VERSION: u32 = 1;
pub(super) const POLICY: &str =
    "existing-cli-gates-v1;serial;warmup-each-sample;nearest-rank-percentiles";
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum Gate {
    WarmExact,
    WarmChunk,
    ColdExact,
}
impl Gate {
    pub const ALL: [Self; 3] = [Self::WarmExact, Self::WarmChunk, Self::ColdExact];
    pub fn label(self) -> &'static str {
        match self {
            Self::WarmExact => "warm-exact",
            Self::WarmChunk => "warm-chunk",
            Self::ColdExact => "cold-exact",
        }
    }
    pub fn limit_ns(self) -> u64 {
        match self {
            Self::WarmExact => 1_000_000_000,
            Self::WarmChunk | Self::ColdExact => 5_000_000_000,
        }
    }
    pub fn cases(self) -> Vec<&'static str> {
        match self {
            Self::WarmExact => vec!["wasm_functions.js"],
            Self::WarmChunk => super::chunk_cases::CASES.to_vec(),
            Self::ColdExact => vec!["wasm_host_output.js"],
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Build {
    pub rustc_verbose: String,
    pub target: String,
    pub host: String,
    pub profile: String,
    pub opt_level: String,
    pub debug: String,
    pub rustflags: String,
    pub spec_exec_oracle_linked: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Platform {
    pub os: String,
    pub architecture: String,
    pub kernel: String,
    pub cpu_model: String,
    pub logical_cpus_available: usize,
    pub physical_memory: String,
    pub process_cpu_affinity: Option<String>,
    pub cgroup_limits: Vec<(String, String)>,
    pub machine_label: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Configuration {
    pub backend: String,
    pub fixture_root: String,
    pub working_directory: String,
    pub host_surface: String,
    pub compilation_jobs: usize,
    pub cache_paths_and_limits: Vec<(String, String, Option<u64>)>,
    pub environment: Vec<(String, Option<String>)>,
    pub idle_machine_acknowledged: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Fixture {
    pub name: String,
    pub bytes: usize,
    pub sha256: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Identity {
    pub compiler: CompilerProvenance,
    pub build: Build,
    pub platform: Platform,
    pub configuration: Configuration,
    pub measurement_policy: String,
    pub corpus_sha256: String,
    pub fixtures: Vec<Fixture>,
    pub gates: Vec<Gate>,
    pub samples_per_gate: usize,
}

/// Compiler-stage and emitted-runtime reports bind the same corpus, machine,
/// executable and build facts while retaining their own measurement policies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ProfileIdentity {
    pub compiler: CompilerProvenance,
    pub build: Build,
    pub platform: Platform,
    pub configuration: Configuration,
    pub corpus_sha256: String,
    pub fixtures: Vec<Fixture>,
    pub samples_per_case: usize,
}
impl From<Identity> for ProfileIdentity {
    fn from(value: Identity) -> Self {
        let Identity { compiler, build, platform, configuration, corpus_sha256, fixtures,
            samples_per_gate, measurement_policy: _, gates: _ } = value;
        Self { compiler, build, platform, configuration, corpus_sha256, fixtures,
            samples_per_case: samples_per_gate }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Log {
    pub path: String,
    pub bytes: usize,
    pub sha256: String,
}
impl Log {
    pub fn validate(&self) -> Result<(), String> {
        lila_engine::CompilerDigest::parse(&self.sha256)?;
        let path = std::path::Path::new(&self.path);
        if self.path.is_empty()
            || path.is_absolute()
            || path
                .components()
                .any(|part| !matches!(part, std::path::Component::Normal(_)))
        {
            return Err("log paths must remain inside their performance artifact directory".into());
        }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Invocation {
    pub case: String,
    pub elapsed_ns: u64,
    pub exit_code: i32,
    pub stdout: Log,
    pub stderr: Log,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Prune {
    pub exit_code: i32,
    pub stdout: Log,
    pub stderr: Log,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Sample {
    pub gate: Gate,
    pub repetition: usize,
    pub warmup: Vec<Invocation>,
    pub prune: Option<Prune>,
    pub measured: Vec<Invocation>,
    pub aggregate_elapsed_ns: Option<u64>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "kebab-case", deny_unknown_fields)]
pub(super) enum Completion {
    Running,
    Failed { reason: String },
    Complete,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CaseSummary {
    pub case: String,
    pub timing: Summary,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct GateSummary {
    pub gate: Gate,
    pub aggregate: Summary,
    pub cases: Vec<CaseSummary>,
    pub budget_ns: u64,
    pub over_budget_samples: usize,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CaseComparison {
    pub case: String,
    pub change: Change,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct GateComparison {
    pub gate: Gate,
    pub aggregate: Change,
    pub cases: Vec<CaseComparison>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Comparison {
    pub baseline_sha256: String,
    pub baseline_compiler: CompilerProvenance,
    pub candidate_compiler: CompilerProvenance,
    pub gates: Vec<GateComparison>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Report {
    pub version: u32,
    pub started_unix_seconds: u64,
    pub identity: Identity,
    pub completion: Completion,
    pub samples: Vec<Sample>,
    pub summaries: Vec<GateSummary>,
    pub comparison: Option<Comparison>,
    pub measurement_scope: String,
    pub full_conformance_verified: bool,
}
impl Report {
    pub fn summaries(&self) -> Result<Vec<GateSummary>, String> {
        if self.version != VERSION || self.identity.measurement_policy != POLICY {
            return Err("unsupported performance schema or measurement policy".into());
        }
        if !(3..=100).contains(&self.identity.samples_per_gate)
            || self.identity.gates.is_empty()
            || self.identity.gates.len() > Gate::ALL.len()
            || self
                .identity
                .gates
                .iter()
                .enumerate()
                .any(|(index, gate)| self.identity.gates[..index].contains(gate))
        {
            return Err("invalid performance sample plan".into());
        }
        if !matches!(self.completion, Completion::Complete) {
            return Err("incomplete or failed timing cannot become a baseline".into());
        }
        if self.full_conformance_verified || self.measurement_scope != super::MEASUREMENT_SCOPE {
            return Err("timing cannot claim subsystem or conformance measurements".into());
        }
        if self.samples.len() != self.identity.gates.len() * self.identity.samples_per_gate {
            return Err("missing or duplicated planned samples".into());
        }
        let mut result = Vec::new();
        for (gate_index, &gate) in self.identity.gates.iter().enumerate() {
            let names = gate.cases();
            let start = gate_index * self.identity.samples_per_gate;
            let samples = &self.samples[start..start + self.identity.samples_per_gate];
            let mut elapsed = Vec::new();
            for (index, sample) in samples.iter().enumerate() {
                if sample.gate != gate || sample.repetition != index {
                    return Err("sample order differs from its plan".into());
                }
                let validate = |runs: &[Invocation]| -> Result<(), String> {
                    if runs.len() != names.len()
                        || runs
                            .iter()
                            .zip(&names)
                            .any(|(run, name)| run.case != *name || run.exit_code != 0)
                    {
                        return Err("missing, reordered or failed fixture invocation".into());
                    }
                    for run in runs {
                        run.stdout.validate()?;
                        run.stderr.validate()?;
                    }
                    Ok(())
                };
                validate(&sample.measured)?;
                match gate {
                    Gate::ColdExact => {
                        if !sample.warmup.is_empty() {
                            return Err("cold sample must not warm its measured input".into());
                        }
                        let prune = sample.prune.as_ref().ok_or("cold sample omits its prune")?;
                        if prune.exit_code != 0 {
                            return Err("cold cache prune failed".into());
                        }
                        prune.stdout.validate()?;
                        prune.stderr.validate()?;
                    }
                    Gate::WarmExact | Gate::WarmChunk => {
                        validate(&sample.warmup)?;
                        if sample.prune.is_some() {
                            return Err("warm sample must not prune caches".into());
                        }
                    }
                }
                let total = sample
                    .aggregate_elapsed_ns
                    .ok_or("sample lacks aggregate span")?;
                let contained: u128 = sample
                    .measured
                    .iter()
                    .map(|run| u128::from(run.elapsed_ns))
                    .sum();
                if contained > u128::from(total) {
                    return Err("aggregate span is shorter than contained case spans".into());
                }
                elapsed.push(total);
            }
            result.push(GateSummary {
                gate,
                aggregate: Summary::from_samples(&elapsed)?,
                cases: names
                    .iter()
                    .enumerate()
                    .map(|(index, &name)| {
                        Ok(CaseSummary {
                            case: name.into(),
                            timing: Summary::from_samples(
                                &samples
                                    .iter()
                                    .map(|sample| sample.measured[index].elapsed_ns)
                                    .collect::<Vec<_>>(),
                            )?,
                        })
                    })
                    .collect::<Result<_, String>>()?,
                budget_ns: gate.limit_ns(),
                over_budget_samples: elapsed.iter().filter(|&&ns| ns > gate.limit_ns()).count(),
            });
        }
        Ok(result)
    }
    pub fn logs(&self) -> Vec<&Log> {
        let mut logs = Vec::new();
        for sample in &self.samples {
            for run in sample.warmup.iter().chain(&sample.measured) {
                logs.push(&run.stdout);
                logs.push(&run.stderr);
            }
            if let Some(prune) = &sample.prune {
                logs.push(&prune.stdout);
                logs.push(&prune.stderr);
            }
        }
        logs
    }
}
/// A complete compatible report, admitted before timing or cache mutation.
pub(super) struct Baseline {
    report: Report,
    digest: String,
}
impl Baseline {
    pub fn admit(report: Report, digest: String, candidate: &Identity) -> Result<Self, String> {
        lila_engine::CompilerDigest::parse(&digest)?;
        if report.summaries()? != report.summaries {
            return Err("baseline summaries disagree with raw samples".into());
        }
        require_compatible(&report.identity, candidate)?;
        Ok(Self { report, digest })
    }
    pub fn compare(&self, candidate: &Report) -> Result<Comparison, String> {
        require_compatible(&self.report.identity, &candidate.identity)?;
        let summaries = candidate.summaries()?;
        Ok(Comparison {
            baseline_sha256: self.digest.clone(),
            baseline_compiler: self.report.identity.compiler.clone(),
            candidate_compiler: candidate.identity.compiler.clone(),
            gates: self
                .report
                .summaries
                .iter()
                .zip(&summaries)
                .map(|(before, after)| GateComparison {
                    gate: after.gate,
                    aggregate: Change::between(&before.aggregate, &after.aggregate),
                    cases: before
                        .cases
                        .iter()
                        .zip(&after.cases)
                        .map(|(before, after)| CaseComparison {
                            case: after.case.clone(),
                            change: Change::between(&before.timing, &after.timing),
                        })
                        .collect(),
                })
                .collect(),
        })
    }
}

fn require_compatible(before: &Identity, after: &Identity) -> Result<(), String> {
    // Exhaustive destructuring makes adding a new identity dimension require
    // an explicit comparison decision. Only compiler build/image may differ.
    let Identity {
        compiler: _,
        build,
        platform,
        configuration,
        measurement_policy,
        corpus_sha256,
        fixtures,
        gates,
        samples_per_gate,
    } = before;
    if build != &after.build
        || platform != &after.platform
        || configuration != &after.configuration
        || measurement_policy != &after.measurement_policy
        || corpus_sha256 != &after.corpus_sha256
        || fixtures != &after.fixtures
        || gates != &after.gates
        || samples_per_gate != &after.samples_per_gate
    {
        return Err(
            "incompatible baseline: workload, platform, toolchain, configuration or spans changed"
                .into(),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn model() -> Report {
        let log = Log {
            path: "logs/model.stdout".into(),
            bytes: 0,
            sha256: super::super::digest(&[]),
        };
        let run = |ns| Invocation {
            case: "wasm_functions.js".into(),
            elapsed_ns: ns,
            exit_code: 0,
            stdout: log.clone(),
            stderr: log.clone(),
        };
        let mut report = Report {
            version: VERSION,
            started_unix_seconds: 0,
            identity: Identity {
                compiler: CompilerProvenance::current().unwrap(),
                build: Build {
                    rustc_verbose: "test model toolchain".into(),
                    target: "test".into(),
                    host: "test".into(),
                    profile: "test".into(),
                    opt_level: "0".into(),
                    debug: "true".into(),
                    rustflags: String::new(),
                    spec_exec_oracle_linked: false,
                },
                platform: Platform {
                    os: "unit-fixture".into(),
                    architecture: "unit-fixture".into(),
                    kernel: "unit-fixture".into(),
                    cpu_model: "unit-fixture".into(),
                    logical_cpus_available: 1,
                    physical_memory: "unit-fixture".into(),
                    process_cpu_affinity: None,
                    cgroup_limits: Vec::new(),
                    machine_label: "unit-fixture".into(),
                },
                configuration: Configuration {
                    backend: "wasm-aot".into(),
                    fixture_root: "/unit-fixture".into(),
                    working_directory: "/unit-fixture".into(),
                    host_surface: "product".into(),
                    compilation_jobs: 1,
                    cache_paths_and_limits: Vec::new(),
                    environment: Vec::new(),
                    idle_machine_acknowledged: true,
                },
                measurement_policy: POLICY.into(),
                corpus_sha256: super::super::digest(b"unit-fixture"),
                fixtures: Vec::new(),
                gates: vec![Gate::WarmExact],
                samples_per_gate: 3,
            },
            completion: Completion::Complete,
            samples: (0..3)
                .map(|index| Sample {
                    gate: Gate::WarmExact,
                    repetition: index,
                    warmup: vec![run(1)],
                    prune: None,
                    measured: vec![run(10 + index as u64)],
                    aggregate_elapsed_ns: Some(20 + index as u64),
                })
                .collect(),
            summaries: Vec::new(),
            comparison: None,
            measurement_scope: super::super::MEASUREMENT_SCOPE.into(),
            full_conformance_verified: false,
        };
        report.summaries = report.summaries().unwrap();
        report
    }
    fn admitted(report: Report) -> Result<Baseline, String> {
        Baseline::admit(
            report.clone(),
            super::super::digest(b"report model"),
            &model().identity,
        )
    }
    #[test]
    fn missing_reordered_failed_or_interrupted_runs_never_become_baselines() {
        let mut missing = model();
        missing.samples.pop();
        assert!(admitted(missing).is_err());
        let mut reordered = model();
        reordered.samples.swap(0, 1);
        assert!(admitted(reordered).is_err());
        let mut failed = model();
        failed.samples[0].measured[0].exit_code = 1;
        assert!(admitted(failed).is_err());
        let mut incomplete = model();
        incomplete.completion = Completion::Running;
        assert!(admitted(incomplete).is_err());
        let mut failed = model();
        failed.completion = Completion::Failed {
            reason: "interrupted fixture".into(),
        };
        assert!(admitted(failed).is_err());
    }
    #[test]
    fn forged_summaries_changed_spans_or_short_aggregate_measurements_are_rejected() {
        let mut forged = model();
        forged.summaries[0].aggregate.median_ns = 0;
        assert!(admitted(forged).is_err());
        let mut changed = model();
        changed.identity.measurement_policy.push_str("changed");
        assert!(admitted(changed).is_err());
        let mut short = model();
        short.samples[0].aggregate_elapsed_ns = Some(1);
        assert!(admitted(short).is_err());
        let mut warm = model();
        warm.samples[0].warmup.clear();
        assert!(admitted(warm).is_err());
        let mut claim = model();
        claim.full_conformance_verified = true;
        assert!(admitted(claim).is_err());
    }
    #[test]
    fn compatibility_rejects_changed_workload_platform_toolchain_and_resource_policy() {
        let mut variants = Vec::new();
        let mut value = model();
        value.identity.corpus_sha256 = super::super::digest(b"different corpus");
        variants.push(value);
        let mut value = model();
        value.identity.platform.machine_label = "other machine".into();
        variants.push(value);
        let mut value = model();
        value.identity.build.opt_level = "3".into();
        variants.push(value);
        let mut value = model();
        value.identity.configuration.compilation_jobs = 2;
        variants.push(value);
        let mut value = model();
        value.identity.configuration.fixture_root = "/different/cache-filename".into();
        variants.push(value);
        for report in variants {
            assert!(admitted(report).is_err());
        }
    }
    #[test]
    fn compiler_images_may_differ_but_comparisons_recompute_raw_sample_changes() {
        let baseline = admitted(model()).unwrap();
        let mut candidate = model();
        let mut identity = serde_json::to_value(&candidate.identity.compiler).unwrap();
        identity["executable_sha256"] = serde_json::Value::String("1".repeat(64));
        candidate.identity.compiler = serde_json::from_value(identity).unwrap();
        for sample in &mut candidate.samples {
            sample.measured[0].elapsed_ns += 10;
            sample.aggregate_elapsed_ns = sample.aggregate_elapsed_ns.map(|ns| ns + 10);
        }
        let change = baseline.compare(&candidate).unwrap();
        assert_ne!(change.baseline_compiler, change.candidate_compiler);
        assert_eq!(change.gates[0].aggregate.median_delta_ns, 10);
        assert_eq!(change.gates[0].cases[0].change.median_delta_ns, 10);
    }
    #[test]
    fn cold_samples_require_successful_prune_without_a_warmup_and_log_paths_are_confined() {
        let mut report = model();
        report.identity.gates = vec![Gate::ColdExact];
        for sample in &mut report.samples {
            sample.gate = Gate::ColdExact;
            sample.measured[0].case = "wasm_host_output.js".into();
        }
        assert!(report.summaries().is_err());
        for sample in &mut report.samples {
            sample.warmup.clear();
            let log = sample.measured[0].stdout.clone();
            sample.prune = Some(Prune {
                exit_code: 0,
                stdout: log.clone(),
                stderr: log,
            });
        }
        assert!(report.summaries().is_ok());
        report.samples[0].measured[0].stdout.path = "../outside.log".into();
        assert!(report.summaries().is_err());
    }
    #[test]
    fn baseline_admission_requires_the_actual_retained_log_bytes() {
        let report = model();
        let root = std::env::temp_dir().join(format!("lila-perf-log-model-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        std::fs::create_dir(root.join("logs")).unwrap();
        let path = root.join("report.json");
        super::super::write_new(&path, &serde_json::to_vec(&report).unwrap()).unwrap();
        let log = root.join("logs/model.stdout");
        assert!(super::super::verify_retained_evidence(&report, &path).is_err());
        super::super::write_new(&log, &[]).unwrap();
        assert!(super::super::verify_retained_evidence(&report, &path).is_ok());
        std::fs::write(&log, b"changed after measurement").unwrap();
        assert!(super::super::verify_retained_evidence(&report, &path).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}
