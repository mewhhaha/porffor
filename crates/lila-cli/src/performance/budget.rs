//! A budget is evaluated only against original admitted repeated measurements.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum Metric {
    CompilerPrepare, CompilerLower, CompilerEmit, CompilerValidate,
    RuntimeEngine, RuntimeModule, RuntimeStoreAndLinker, RuntimeInstantiate,
    RuntimeExportLookup, RuntimeExecutionAndCompletion, RuntimeTotal,
    ModuleBytes, DefinedFunctions, ImportedFunctions, CodeBodyBytes,
    LargestCodeBodyBytes, IdenticalExtraBodies, IdenticalExtraBodyBytes, DataPayloadBytes,
    GcCapacityBefore, GcCapacityAfter, SampledProcessRss,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum Unit { Nanoseconds, Bytes, Count }
impl Metric {
    fn unit(self) -> Unit {
        match self {
            Self::CompilerPrepare | Self::CompilerLower | Self::CompilerEmit | Self::CompilerValidate
            | Self::RuntimeEngine | Self::RuntimeModule | Self::RuntimeStoreAndLinker | Self::RuntimeInstantiate
            | Self::RuntimeExportLookup | Self::RuntimeExecutionAndCompletion | Self::RuntimeTotal => Unit::Nanoseconds,
            Self::ModuleBytes | Self::CodeBodyBytes | Self::LargestCodeBodyBytes | Self::IdenticalExtraBodyBytes
            | Self::DataPayloadBytes | Self::GcCapacityBefore | Self::GcCapacityAfter | Self::SampledProcessRss => Unit::Bytes,
            Self::DefinedFunctions | Self::ImportedFunctions | Self::IdenticalExtraBodies => Unit::Count,
        }
    }
    fn admits(self, statistic: Statistic) -> bool {
        match self {
            Self::CompilerPrepare | Self::CompilerLower | Self::CompilerEmit | Self::CompilerValidate
            | Self::RuntimeEngine | Self::RuntimeModule | Self::RuntimeStoreAndLinker | Self::RuntimeInstantiate
            | Self::RuntimeExportLookup | Self::RuntimeExecutionAndCompletion | Self::RuntimeTotal
            | Self::GcCapacityBefore | Self::GcCapacityAfter | Self::SampledProcessRss => matches!(statistic, Statistic::Median | Statistic::P95),
            Self::ModuleBytes | Self::DefinedFunctions | Self::ImportedFunctions | Self::CodeBodyBytes
            | Self::LargestCodeBodyBytes | Self::IdenticalExtraBodies | Self::IdenticalExtraBodyBytes
            | Self::DataPayloadBytes => statistic == Statistic::Exact,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum Statistic { Exact, Median, P95 }
type Key = (String, Metric, Statistic);

/// Created by the compiler/runtime report owners after all evidence is checked.
pub(super) struct AdmittedMetrics {
    identity: ProfileIdentity,
    values: BTreeMap<Key, u64>,
}
impl AdmittedMetrics {
    pub(super) fn new(identity: ProfileIdentity) -> Self { Self { identity, values: BTreeMap::new() } }
    pub(super) fn insert(&mut self, fixture: &str, metric: Metric, statistic: Statistic, value: u64) -> Result<(), String> {
        if !metric.admits(statistic) || !self.identity.fixtures.iter().any(|entry| entry.name == fixture)
            || self.values.insert((fixture.into(), metric, statistic), value).is_some() {
            return Err("duplicate, foreign or wrongly dimensioned performance metric".into());
        }
        Ok(())
    }
    pub(super) fn timing(&mut self, fixture: &str, metric: Metric, summary: &stats::Summary) -> Result<(), String> {
        self.insert(fixture, metric, Statistic::Median, summary.median_ns)?;
        self.insert(fixture, metric, Statistic::P95, summary.p95_ns)
    }
    pub(super) fn footprint(&mut self, fixture: &str, footprint: &serde_json::Value) -> Result<(), String> {
        for (key, metric) in [("module_bytes", Metric::ModuleBytes), ("defined_functions", Metric::DefinedFunctions),
            ("imported_functions", Metric::ImportedFunctions), ("code_body_bytes", Metric::CodeBodyBytes),
            ("largest_code_body_bytes", Metric::LargestCodeBodyBytes), ("identical_extra_bodies", Metric::IdenticalExtraBodies),
            ("identical_extra_body_bytes", Metric::IdenticalExtraBodyBytes), ("data_payload_bytes", Metric::DataPayloadBytes)] {
            let value = footprint.get(key).and_then(serde_json::Value::as_u64).ok_or_else(|| format!("admitted footprint has no integer {key}"))?;
            self.insert(fixture, metric, Statistic::Exact, value)?;
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Rule {
    fixture: String,
    metric: Metric,
    statistic: Statistic,
    absolute_maximum: Option<u64>,
    maximum_increase_basis_points: Option<u32>,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PolicyWire { version: u32, minimum_samples: usize, rules: Vec<Rule> }
struct CheckedPolicy(PolicyWire);
impl CheckedPolicy {
    fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let wire: PolicyWire = serde_json::from_slice(bytes).map_err(|error| error.to_string())?;
        if wire.version != 1 || !(3..=100).contains(&wire.minimum_samples) || !(1..=1024).contains(&wire.rules.len()) {
            return Err("budget requires version 1, 3..100 minimum samples and 1..1024 rules".into());
        }
        let mut seen = BTreeSet::new();
        for rule in &wire.rules {
            if rule.fixture.is_empty() || rule.fixture.len() > 256 || !rule.metric.admits(rule.statistic)
                || (rule.absolute_maximum.is_none() && rule.maximum_increase_basis_points.is_none())
                || rule.maximum_increase_basis_points.is_some_and(|value| value > 1_000_000)
                || !seen.insert((rule.fixture.clone(), rule.metric, rule.statistic)) {
                return Err("budget rule is empty, duplicated, wrongly dimensioned or outside its bound".into());
            }
        }
        Ok(Self(wire))
    }
    fn evaluate(&self, baseline: &AdmittedMetrics, candidate: &AdmittedMetrics) -> Result<Vec<Decision>, String> {
        let mut identity = baseline.identity.clone();
        identity.compiler = candidate.identity.compiler.clone();
        if identity != candidate.identity || !identity.configuration.idle_machine_acknowledged
            || identity.samples_per_case < self.0.minimum_samples {
            return Err("budget needs comparable completed repeated measurements on the same idle machine, toolchain, workload and resource configuration".into());
        }
        self.0.rules.iter().map(|rule| {
            let key = (rule.fixture.clone(), rule.metric, rule.statistic);
            let before = *baseline.values.get(&key).ok_or("budget metric is unavailable in the admitted baseline")?;
            let after = *candidate.values.get(&key).ok_or("budget metric is unavailable in the admitted candidate")?;
            let absolute_passed = rule.absolute_maximum.map(|limit| after <= limit);
            let relative_passed = rule.maximum_increase_basis_points.map(|points| {
                // Integral comparison retains exact zero baselines and cannot
                // hide an overflow, rounded percentage or NaN.
                u128::from(after) * 10_000 <= u128::from(before) * (10_000 + u128::from(points))
            });
            Ok(Decision { fixture: rule.fixture.clone(), metric: rule.metric, statistic: rule.statistic,
                unit: rule.metric.unit(), baseline: before, candidate: after,
                absolute_maximum: rule.absolute_maximum, maximum_increase_basis_points: rule.maximum_increase_basis_points,
                passed: absolute_passed.unwrap_or(true) && relative_passed.unwrap_or(true), absolute_passed, relative_passed })
        }).collect()
    }
}
#[derive(Debug, Serialize)]
struct Decision {
    fixture: String, metric: Metric, statistic: Statistic, unit: Unit,
    baseline: u64, candidate: u64, absolute_maximum: Option<u64>, maximum_increase_basis_points: Option<u32>,
    absolute_passed: Option<bool>, relative_passed: Option<bool>, passed: bool,
}
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "kebab-case")]
enum Mode { Compiler, Runtime }
struct Options { mode: Mode, baseline: PathBuf, candidate: PathBuf, policy: PathBuf, output: PathBuf }
fn parse(args: Vec<String>) -> Result<Options, String> {
    let mut args = args.into_iter().skip(1);
    let (mut mode, mut baseline, mut candidate, mut policy, mut output) = (None, None, None, None, None);
    while let Some(flag) = args.next() {
        let value = args.next().ok_or_else(|| format!("{flag} needs a value"))?;
        match flag.as_str() {
            "--mode" if mode.is_none() => mode = Some(match value.as_str() {
                "compiler" => Mode::Compiler, "runtime" => Mode::Runtime,
                _ => return Err("budget mode must be compiler or runtime".into()),
            }),
            "--baseline" if baseline.is_none() => baseline = Some(PathBuf::from(value)),
            "--candidate" if candidate.is_none() => candidate = Some(PathBuf::from(value)),
            "--policy" if policy.is_none() => policy = Some(PathBuf::from(value)),
            "--output-dir" if output.is_none() => output = Some(PathBuf::from(value)),
            _ => return Err(format!("unknown or duplicate performance check option: {flag}")),
        }
    }
    Ok(Options { mode: mode.ok_or("performance check needs --mode compiler|runtime")?,
        baseline: baseline.ok_or("performance check needs --baseline PATH")?, candidate: candidate.ok_or("performance check needs --candidate PATH")?,
        policy: policy.ok_or("performance check needs --policy PATH")?, output: output.ok_or("performance check needs --output-dir PATH")? })
}
#[derive(Serialize)]
struct BudgetReport {
    version: u32, mode: Mode, scope: &'static str,
    baseline_path: PathBuf, candidate_path: PathBuf,
    baseline_sha256: String, candidate_sha256: String, policy_sha256: String,
    baseline_identity: ProfileIdentity, candidate_identity: ProfileIdentity,
    passed: bool, rules: Vec<Decision>,
}
pub(super) fn command(args: Vec<String>) -> Result<PathBuf, String> {
    let options = parse(args)?;
    let policy_bytes = read_bounded(&options.policy, 1024 * 1024)?;
    let policy = CheckedPolicy::from_bytes(&policy_bytes)?;
    let before_bytes = read_bounded(&options.baseline, BASELINE_MAX_BYTES)?;
    let after_bytes = read_bounded(&options.candidate, BASELINE_MAX_BYTES)?;
    let admit = match options.mode { Mode::Compiler => compiler::budget_metrics, Mode::Runtime => runtime::budget_metrics };
    let before = admit(&options.baseline, &before_bytes)?;
    let after = admit(&options.candidate, &after_bytes)?;
    let rules = policy.evaluate(&before, &after)?;
    let passed = rules.iter().all(|rule| rule.passed);
    let report = BudgetReport { version: 1, mode: options.mode,
        scope: "explicit-policy-over-original-admitted-repeated-samples;no-measurement-or-program-execution;not-conformance",
        baseline_path: options.baseline, candidate_path: options.candidate,
        baseline_sha256: digest(&before_bytes), candidate_sha256: digest(&after_bytes), policy_sha256: digest(&policy_bytes),
        baseline_identity: before.identity, candidate_identity: after.identity, passed, rules };
    fs::create_dir(&options.output).map_err(|error| format!("budget output must be a fresh directory: {error}"))?;
    for (name, bytes) in [("baseline.json", before_bytes), ("candidate.json", after_bytes), ("policy.json", policy_bytes)] {
        write_new(&options.output.join(name), &bytes)?;
    }
    let path = options.output.join("report.json");
    write_new(&path, &serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?)?;
    if !passed { return Err(format!("performance budget exceeded; retained {}", path.display())); }
    Ok(path)
}

#[cfg(test)]
mod tests;
