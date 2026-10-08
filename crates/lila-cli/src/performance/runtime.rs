//! Fresh product Stores execute the original fixed corpus's retained Wasm.
use super::*;
use super::compiler::{footprint, ARTIFACT_MAX_BYTES};
use lila_engine::{CompileOptions, Engine, RealmBuilder, ObservedCompletion, HostOutputEvent,
    WasmArtifactFootprint, WasmRuntimeMetricUnavailable, WasmRuntimeModuleCacheOutcome,
    WasmRuntimeProfile, wasm_runtime_profile_policy};
use serde::{Deserialize, Serialize};
mod process_memory;
use process_memory::{Measurement as ProcessMemoryMeasurement, Summary as ProcessMemorySummary};

const POLICY: &str = "product-runtime-v2;fixed20;serial;compile-once;one-warmup-per-case;fresh-store;retain-native-module-cache;no-forced-gc;process-rss-10ms-plus-endpoints;no-runtime-budget";
const SCOPE: &str = "emitted-product-wasm;engine-module-store-linker-instantiation-export-execution-completion;total-includes-profile-sampler-retirement;total-excludes-worker-start-store-drop-and-js-compilation;gc-capacity-boundaries;sampled-process-rss-including-other-threads-and-retained-caches";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Stage { Engine, Module, StoreAndLinker, Instantiate, ExportLookup, ExecutionAndCompletion, Total }
impl Stage {
    const ALL: [Self; 7] = [Self::Engine, Self::Module, Self::StoreAndLinker, Self::Instantiate,
        Self::ExportLookup, Self::ExecutionAndCompletion, Self::Total];
    fn index(self) -> usize {
        match self { Self::Engine => 0, Self::Module => 1, Self::StoreAndLinker => 2,
            Self::Instantiate => 3, Self::ExportLookup => 4, Self::ExecutionAndCompletion => 5, Self::Total => 6 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum ModuleCache { Hit, Miss, Bypassed }
impl From<WasmRuntimeModuleCacheOutcome> for ModuleCache {
    fn from(value: WasmRuntimeModuleCacheOutcome) -> Self {
        match value { WasmRuntimeModuleCacheOutcome::Hit => Self::Hit,
            WasmRuntimeModuleCacheOutcome::Miss => Self::Miss, WasmRuntimeModuleCacheOutcome::Bypassed => Self::Bypassed }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Unavailable { WasmtimeHasNoPublicCounter, NoPerInvocationPeakSampler }
impl From<WasmRuntimeMetricUnavailable> for Unavailable {
    fn from(value: WasmRuntimeMetricUnavailable) -> Self {
        match value { WasmRuntimeMetricUnavailable::WasmtimeHasNoPublicCounter => Self::WasmtimeHasNoPublicCounter,
            WasmRuntimeMetricUnavailable::NoPerInvocationPeakSampler => Self::NoPerInvocationPeakSampler }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct UnavailableMetrics {
    allocation_count: Unavailable,
    allocation_bytes: Unavailable,
    live_gc_bytes: Unavailable,
    collection_count: Unavailable,
    gc_pause_time: Unavailable,
    peak_gc_heap_bytes: Unavailable,
}
impl UnavailableMetrics {
    fn current_contract() -> Self {
        let counter = Unavailable::WasmtimeHasNoPublicCounter;
        let peak = Unavailable::NoPerInvocationPeakSampler;
        Self { allocation_count: counter, allocation_bytes: counter, live_gc_bytes: counter,
            collection_count: counter, gc_pause_time: counter, peak_gc_heap_bytes: peak }
    }
    fn from_profile(profile: &WasmRuntimeProfile) -> Self {
        Self { allocation_count: profile.allocation_count.into(), allocation_bytes: profile.allocation_bytes.into(),
            live_gc_bytes: profile.live_gc_bytes.into(), collection_count: profile.collection_count.into(),
            gc_pause_time: profile.gc_pause_time.into(), peak_gc_heap_bytes: profile.peak_gc_heap_bytes.into() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ArtifactEvidence { wasm: Log, footprint: serde_json::Value }
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Measurement {
    stage_ns: [u64; 7],
    module_cache: ModuleCache,
    gc_capacity_before_execution_bytes: u64,
    gc_capacity_after_execution_bytes: u64,
    process_memory: ProcessMemoryMeasurement,
    unavailable: UnavailableMetrics,
    observation: Log,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    fixture: String,
    artifact: Option<ArtifactEvidence>,
    warmup: Option<Measurement>,
    samples: Vec<Measurement>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StageSummary { stage: Stage, timing: stats::Summary }
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ByteSummary {
    samples: usize, minimum_bytes: u64, median_bytes: u64,
    p90_bytes: u64, p95_bytes: u64, maximum_bytes: u64,
}
impl ByteSummary {
    fn from_samples(values: &[u64]) -> Result<Self, String> {
        let distribution = stats::Summary::from_samples(values)?;
        Ok(Self { samples: distribution.samples, minimum_bytes: distribution.minimum_ns,
            median_bytes: distribution.median_ns, p90_bytes: distribution.p90_ns,
            p95_bytes: distribution.p95_ns, maximum_bytes: distribution.maximum_ns })
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CaseSummary {
    fixture: String,
    stages: Vec<StageSummary>,
    gc_capacity_before_execution: ByteSummary,
    gc_capacity_after_execution: ByteSummary,
    process_memory: ProcessMemorySummary,
    unavailable: UnavailableMetrics,
    footprint: serde_json::Value,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StageChange { stage: Stage, timing: stats::Change }
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ByteChange {
    baseline_median_bytes: u64, candidate_median_bytes: u64, median_delta_bytes: i128,
    baseline_p95_bytes: u64, candidate_p95_bytes: u64, p95_delta_bytes: i128,
}
impl ByteChange {
    fn between(before: &ByteSummary, after: &ByteSummary) -> Self {
        Self { baseline_median_bytes: before.median_bytes, candidate_median_bytes: after.median_bytes,
            median_delta_bytes: i128::from(after.median_bytes) - i128::from(before.median_bytes),
            baseline_p95_bytes: before.p95_bytes, candidate_p95_bytes: after.p95_bytes,
            p95_delta_bytes: i128::from(after.p95_bytes) - i128::from(before.p95_bytes) }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CaseChange {
    fixture: String, stages: Vec<StageChange>,
    gc_capacity_before_execution: ByteChange, gc_capacity_after_execution: ByteChange,
    sampled_process_rss: Option<ByteChange>,
    baseline_footprint: serde_json::Value, candidate_footprint: serde_json::Value,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Comparison { baseline_sha256: String, baseline_compiler: CompilerProvenance, cases: Vec<CaseChange> }
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RuntimeReport {
    version: u32, measurement_policy: String, measurement_scope: String,
    runtime_policy: String, identity: ProfileIdentity, completion: Completion,
    cases: Vec<Case>, summaries: Vec<CaseSummary>, comparison: Option<Comparison>,
}

fn summarize_case(case: &Case, fixture: &Fixture, count: usize) -> Result<CaseSummary, String> {
    let artifact = case.artifact.as_ref().ok_or("runtime case has no completed compilation")?;
    let warmup = case.warmup.as_ref().ok_or("runtime case has no successful warmup")?;
    if case.fixture != fixture.name || case.samples.len() != count || !(3..=100).contains(&count) {
        return Err("runtime case inventory differs from its source plan".into());
    }
    artifact.wasm.validate()?;
    for sample in std::iter::once(warmup).chain(&case.samples) {
        sample.observation.validate()?;
        if sample.unavailable != UnavailableMetrics::current_contract() {
            return Err("runtime report claims metrics the selected runtime does not expose".into());
        }
        sample.process_memory.validate_for_invocation(sample.stage_ns[Stage::Total.index()])?;
        let phases: u128 = sample.stage_ns[..6].iter().map(|value| u128::from(*value)).sum();
        if phases > u128::from(sample.stage_ns[Stage::Total.index()]) {
            return Err("runtime total is smaller than its disjoint measured phases".into());
        }
    }
    let stages = Stage::ALL.into_iter().map(|stage| {
        stats::Summary::from_samples(&case.samples.iter().map(|sample| sample.stage_ns[stage.index()]).collect::<Vec<_>>())
            .map(|timing| StageSummary { stage, timing })
    }).collect::<Result<_, _>>()?;
    Ok(CaseSummary { fixture: case.fixture.clone(), stages,
        gc_capacity_before_execution: ByteSummary::from_samples(&case.samples.iter()
            .map(|sample| sample.gc_capacity_before_execution_bytes).collect::<Vec<_>>())?,
        gc_capacity_after_execution: ByteSummary::from_samples(&case.samples.iter()
            .map(|sample| sample.gc_capacity_after_execution_bytes).collect::<Vec<_>>())?,
        process_memory: ProcessMemorySummary::from_samples(case.samples.iter().map(|sample| &sample.process_memory))?,
        unavailable: warmup.unavailable.clone(), footprint: artifact.footprint.clone() })
}

impl RuntimeReport {
    fn summaries(&self) -> Result<Vec<CaseSummary>, String> {
        if self.version != 2 || self.measurement_policy != POLICY || self.measurement_scope != SCOPE
            || self.runtime_policy != wasm_runtime_profile_policy() || self.completion != Completion::Complete
            || !(3..=100).contains(&self.identity.samples_per_case)
            || self.cases.len() != self.identity.fixtures.len() {
            return Err("runtime profile is incomplete or has a foreign measurement contract".into());
        }
        self.cases.iter().zip(&self.identity.fixtures)
            .map(|(case, fixture)| summarize_case(case, fixture, self.identity.samples_per_case)).collect()
    }

    fn verify_evidence(&self, source: &Path) -> Result<(), String> {
        if self.summaries()? != self.summaries { return Err("runtime summaries disagree with retained raw measurements".into()); }
        let root = source.parent().unwrap_or_else(|| Path::new("."));
        for fixture in &self.identity.fixtures {
            let mut components = Path::new(&fixture.name).components();
            if !matches!(components.next(), Some(std::path::Component::Normal(_))) || components.next().is_some() {
                return Err("runtime fixture identity must be a single retained filename".into());
            }
            verify_log(root, &Log { path: format!("workload/{}", fixture.name), bytes: fixture.bytes,
                sha256: fixture.sha256.clone() }, BASELINE_MAX_BYTES)?;
        }
        for case in &self.cases {
            let artifact = case.artifact.as_ref().ok_or("missing runtime artifact")?;
            let bytes = verify_log(root, &artifact.wasm, ARTIFACT_MAX_BYTES)?;
            let actual = footprint(WasmArtifactFootprint::from_module_bytes(&bytes).map_err(|error| error.to_string())?);
            if actual != artifact.footprint { return Err("retained runtime Wasm footprint changed".into()); }
            for sample in case.warmup.iter().chain(&case.samples) {
                let bytes = verify_log(root, &sample.observation, BASELINE_MAX_BYTES)?;
                let observation: Observation = serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
                observation.validate()?;
                if observation.process_memory != sample.process_memory {
                    return Err("runtime RSS observation differs from its retained invocation evidence".into());
                }
            }
        }
        Ok(())
    }
}

fn verify_log(root: &Path, log: &Log, bound: u64) -> Result<Vec<u8>, String> {
    log.validate()?;
    let bytes = read_bounded(&root.join(&log.path), bound)?;
    if bytes.len() != log.bytes || digest(&bytes) != log.sha256 { return Err("retained runtime evidence changed".into()); }
    Ok(bytes)
}

/// Budget projection cannot bypass original output, sample and Wasm admission.
pub(super) fn budget_metrics(path: &Path, bytes: &[u8]) -> Result<super::budget::AdmittedMetrics, String> {
    use super::budget::{AdmittedMetrics, Metric, Statistic};
    let report: RuntimeReport = serde_json::from_slice(bytes).map_err(|error| error.to_string())?;
    report.verify_evidence(path)?;
    let mut result = AdmittedMetrics::new(report.identity);
    for case in report.summaries {
        for stage in case.stages {
            let metric = match stage.stage {
                Stage::Engine => Metric::RuntimeEngine, Stage::Module => Metric::RuntimeModule,
                Stage::StoreAndLinker => Metric::RuntimeStoreAndLinker,
                Stage::Instantiate => Metric::RuntimeInstantiate, Stage::ExportLookup => Metric::RuntimeExportLookup,
                Stage::ExecutionAndCompletion => Metric::RuntimeExecutionAndCompletion, Stage::Total => Metric::RuntimeTotal,
            };
            result.timing(&case.fixture, metric, &stage.timing)?;
        }
        for (metric, summary) in [(Metric::GcCapacityBefore, case.gc_capacity_before_execution),
            (Metric::GcCapacityAfter, case.gc_capacity_after_execution)] {
            result.insert(&case.fixture, metric, Statistic::Median, summary.median_bytes)?;
            result.insert(&case.fixture, metric, Statistic::P95, summary.p95_bytes)?;
        }
        if let Some(summary) = case.process_memory.available() {
            result.insert(&case.fixture, Metric::SampledProcessRss, Statistic::Median, summary.median_bytes)?;
            result.insert(&case.fixture, Metric::SampledProcessRss, Statistic::P95, summary.p95_bytes)?;
        }
        result.footprint(&case.fixture, &case.footprint)?;
    }
    Ok(result)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Observation { backend: String, completion_kind: String, completion_type: String,
    completion_debug: String, output: Vec<String>, process_memory: ProcessMemoryMeasurement }
impl Observation {
    fn from_profile(profile: &WasmRuntimeProfile) -> Result<Self, String> {
        if profile.outcome.backend_used != lila_engine::ExecutionBackend::WasmAot {
            return Err("runtime observation must come from the product Wasm backend".into());
        }
        let (completion_kind, value) = match &profile.outcome.completion {
            ObservedCompletion::Normal(value) => ("normal", value),
            ObservedCompletion::Throw(value) => ("throw", value),
        };
        Ok(Self { backend: "wasm-aot".into(), completion_kind: completion_kind.into(),
            completion_type: value.type_name().into(), completion_debug: format!("{value:?}"),
            process_memory: ProcessMemoryMeasurement::from_profile(profile)?,
            output: profile.outcome.output_events.iter().map(|event| match event {
                HostOutputEvent::PrintLine(text) => text.clone(),
            }).collect() })
    }
    fn validate(&self) -> Result<(), String> {
        self.process_memory.validate()?;
        if self.backend != "wasm-aot" || self.completion_kind != "normal"
            || !matches!(self.completion_type.as_str(), "undefined" | "null" | "boolean" | "number" | "string" | "bigint" | "symbol" | "object") {
            return Err("runtime baseline has a failed or foreign execution observation".into());
        }
        Ok(())
    }
}

fn measure(engine: &Engine, artifact: &lila_engine::Artifact, output: &Path, index: usize, repetition: &str) -> Result<Measurement, String> {
    let profile = engine.profile_wasm_execution(artifact, None, true).map_err(|error| error.to_string())?;
    if profile.runtime_policy != wasm_runtime_profile_policy() { return Err("runtime policy differs from planned product execution".into()); }
    let observed = Observation::from_profile(&profile)?;
    let observation = serde_json::to_vec_pretty(&observed).map_err(|error| error.to_string())?;
    let path = format!("observations/{index:02}-{repetition}.json");
    write_new(&output.join(&path), &observation)?;
    if observed.completion_kind == "throw" {
        return Err(format!("benchmark threw an ECMAScript {}; retained {path}: {}",
            observed.completion_type, observed.completion_debug));
    }
    let unavailable = UnavailableMetrics::from_profile(&profile);
    let process_memory = observed.process_memory.clone();
    let spans = profile.timings;
    let mut stage_ns = [0; 7];
    for (result, duration) in stage_ns.iter_mut().zip([spans.engine, spans.module, spans.store_and_linker,
        spans.instantiate, spans.export_lookup, spans.execution_and_completion, spans.total]) {
        *result = duration.as_nanos().try_into().map_err(|_| "runtime span exceeded nanosecond report capacity")?;
    }
    process_memory.validate_for_invocation(stage_ns[Stage::Total.index()])?;
    Ok(Measurement { stage_ns, module_cache: profile.module_cache.into(),
        gc_capacity_before_execution_bytes: profile.heap.capacity_before_execution_bytes.try_into()
            .map_err(|_| "GC capacity exceeds report byte capacity")?,
        gc_capacity_after_execution_bytes: profile.heap.capacity_after_execution_bytes.try_into()
            .map_err(|_| "GC capacity exceeds report byte capacity")?, unavailable, process_memory,
        observation: Log { path, bytes: observation.len(), sha256: digest(&observation) } })
}

fn persist(output: &Path, report: &RuntimeReport) -> Result<(), String> {
    write_new(&output.join("report.next.json"), &serde_json::to_vec_pretty(report).map_err(|error| error.to_string())?)?;
    fs::rename(output.join("report.next.json"), output.join("report.json")).map_err(|error| error.to_string())
}

pub(super) fn command(options: Options) -> Result<PathBuf, String> {
    lila_engine::configure_compilation_jobs(1)?;
    let root = options.fixture_root.canonicalize().map_err(|error| error.to_string())?;
    require_fixtures(&root, &chunk_cases::CASES)?;
    let mut identity = ProfileIdentity::from(capture_identity(&options, &root)?);
    identity.configuration.backend = "wasm-aot-emitted-runtime".into();
    let baseline = options.baseline.as_ref().map(|path| {
        let bytes = read_bounded(path, BASELINE_MAX_BYTES)?;
        let report: RuntimeReport = serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        let mut comparable = report.identity.clone();
        comparable.compiler = identity.compiler.clone();
        if comparable != identity { return Err("runtime baseline differs in workload, machine, build or resource configuration".into()); }
        report.verify_evidence(path)?;
        Ok::<_, String>((report, bytes))
    }).transpose()?;
    fs::create_dir(&options.output).map_err(|error| format!("runtime report requires a fresh output directory: {error}"))?;
    let output = options.output.canonicalize().map_err(|error| error.to_string())?;
    for directory in ["artifacts", "workload", "observations"] { fs::create_dir(output.join(directory)).map_err(|error| error.to_string())?; }
    for (name, source) in chunk_cases::CASES.iter().zip(chunk_cases::SOURCES) { write_new(&output.join("workload").join(name), source.as_bytes())?; }
    if let Some((_, bytes)) = &baseline { write_new(&output.join("baseline.json"), bytes)?; }
    let mut report = RuntimeReport { version: 2, measurement_policy: POLICY.into(), measurement_scope: SCOPE.into(),
        runtime_policy: wasm_runtime_profile_policy(), cases: identity.fixtures.iter().map(|fixture| Case {
            fixture: fixture.name.clone(), artifact: None, warmup: None, samples: vec![] }).collect(),
        identity, completion: Completion::Running, summaries: vec![], comparison: None };
    persist(&output, &report)?;
    let engine = Engine::new(RealmBuilder::new().build());
    let result = (|| {
        for (index, source) in chunk_cases::SOURCES.into_iter().enumerate() {
            let name = report.cases[index].fixture.clone();
            let compilation = engine.profile_script_compilation(source, CompileOptions {
                filename: Some(root.join(&name).to_string_lossy().into_owned()),
                module_loading_policy: lila_engine::ModuleLoadingPolicy::RejectAll,
                ..CompileOptions::default() }).map_err(|error| format!("{name}: {error}"))?;
            let artifact = compilation.artifact;
            let bytes = artifact.bytes.clone();
            if bytes.len() as u64 > ARTIFACT_MAX_BYTES { return Err("runtime artifact exceeds its existing evidence limit".into()); }
            let sha256 = digest(&bytes);
            let path = format!("artifacts/{sha256}.wasm");
            let wasm = Log { path, bytes: bytes.len(), sha256 };
            if output.join(&wasm.path).exists() { verify_log(&output, &wasm, ARTIFACT_MAX_BYTES)?; }
            else { write_new(&output.join(&wasm.path), &bytes)?; }
            report.cases[index].artifact = Some(ArtifactEvidence { wasm, footprint: footprint(compilation.footprint) });
            persist(&output, &report)?;
            report.cases[index].warmup = Some(measure(&engine, &artifact, &output, index, "warmup")?);
            persist(&output, &report)?;
            for repetition in 0..options.samples {
                report.cases[index].samples.push(measure(&engine, &artifact, &output, index, &format!("sample-{repetition:03}"))?);
                persist(&output, &report)?;
            }
        }
        report.completion = Completion::Complete;
        report.summaries = report.summaries()?;
        if let Some((baseline, bytes)) = &baseline {
            let cases = baseline.summaries.iter().zip(&report.summaries).map(|(before, after)| CaseChange {
                fixture: after.fixture.clone(), stages: before.stages.iter().zip(&after.stages).map(|(before, after)| StageChange {
                    stage: after.stage, timing: stats::Change::between(&before.timing, &after.timing) }).collect(),
                gc_capacity_before_execution: ByteChange::between(&before.gc_capacity_before_execution, &after.gc_capacity_before_execution),
                gc_capacity_after_execution: ByteChange::between(&before.gc_capacity_after_execution, &after.gc_capacity_after_execution),
                sampled_process_rss: ProcessMemorySummary::change(&before.process_memory, &after.process_memory),
                baseline_footprint: before.footprint.clone(), candidate_footprint: after.footprint.clone() }).collect();
            report.comparison = Some(Comparison { baseline_sha256: digest(bytes), baseline_compiler: baseline.identity.compiler.clone(), cases });
        }
        Ok::<_, String>(())
    })();
    if let Err(reason) = result {
        report.completion = Completion::Failed { reason: reason.clone() };
        report.summaries.clear(); report.comparison = None;
        persist(&output, &report)?;
        return Err(format!("{reason}; retained {}", output.join("report.json").display()));
    }
    persist(&output, &report)?;
    Ok(output.join("report.json"))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture_case() -> (Case, Fixture) {
        let observation = Log { path: "observations/sample.json".into(), bytes: 1, sha256: "1".repeat(64) };
        let measured = |n| Measurement { stage_ns: [n,n,n,n,n,n,n*10], module_cache: ModuleCache::Hit,
            gc_capacity_before_execution_bytes: n*100, gc_capacity_after_execution_bytes: n*200,
            process_memory: process_memory::fixture_measurement(n * 1024),
            unavailable: UnavailableMetrics::current_contract(), observation: observation.clone() };
        (Case { fixture: "sample.js".into(), artifact: Some(ArtifactEvidence { wasm: Log {
                path: "artifacts/sample.wasm".into(), bytes: 8, sha256: "0".repeat(64) }, footprint: serde_json::json!({"module_bytes":8}) }),
            warmup: Some(measured(1000)), samples: vec![measured(1), measured(9), measured(5)] },
            Fixture { name: "sample.js".into(), bytes: 1, sha256: "2".repeat(64) })
    }
    #[test]
    fn runtime_statistics_exclude_warmup_preserve_capacity_units_and_reject_missing_phases() {
        let (case, fixture) = fixture_case();
        let summary = summarize_case(&case, &fixture, 3).unwrap();
        assert_eq!(summary.stages[Stage::ExecutionAndCompletion.index()].timing.median_ns, 5);
        assert_eq!(summary.gc_capacity_after_execution.median_bytes, 1000);
        assert_eq!(summary.gc_capacity_before_execution.maximum_bytes, 900);
        assert_eq!(summary.process_memory.available().unwrap().median_bytes, 5 * 1024);
        assert_eq!(summary.process_memory.available().unwrap().maximum_bytes, 9 * 1024);
        let mut missing = case.clone(); missing.samples.pop();
        assert!(summarize_case(&missing, &fixture, 3).is_err());
        let mut impossible = case.clone(); impossible.samples[1].stage_ns[6] = 1;
        assert!(summarize_case(&impossible, &fixture, 3).is_err());
        let mut claimed_count = serde_json::to_value(&case).unwrap();
        claimed_count["samples"][0]["unavailable"]["collection_count"] = 42.into();
        assert!(serde_json::from_value::<Case>(claimed_count).is_err());
        let mut invented = case; invented.samples[2].unavailable.collection_count = Unavailable::NoPerInvocationPeakSampler;
        assert!(summarize_case(&invented, &fixture, 3).is_err());
    }

    #[test]
    fn runtime_mode_requires_idle_acknowledgment_and_cannot_select_whole_cli_gates() {
        let args = ["runtime", "--output-dir", "fresh", "--samples", "3", "--machine-label", "host", "--idle-machine"];
        let strings = |args: &[&str]| args.iter().map(|value| (*value).to_owned()).collect();
        let options = parse(strings(&args)).unwrap();
        assert_eq!(options.mode, MeasurementMode::Runtime);
        assert!(options.gates.is_empty());
        assert!(parse(strings(&args[..args.len()-1])).is_err());
        let mut wrong = args.to_vec(); wrong.extend(["--gate", "warm-exact"]);
        assert!(parse(strings(&wrong)).is_err());
    }

    #[test]
    fn failed_or_interrupted_runtime_reports_and_modified_summaries_cannot_be_baselines() {
        let args = ["runtime", "--output-dir", "fresh", "--samples", "3", "--machine-label", "host", "--idle-machine"];
        let options = parse(args.into_iter().map(str::to_owned).collect()).unwrap();
        let (case, fixture) = fixture_case();
        let identity = ProfileIdentity::from(Identity { compiler: CompilerProvenance::current().unwrap(),
            build: Build { rustc_verbose: "test".into(), target: "test".into(), host: "test".into(),
                profile: "test".into(), opt_level: "0".into(), debug: "true".into(), rustflags: "".into(), spec_exec_oracle_linked: false },
            platform: Platform { os: "test".into(), architecture: "test".into(), kernel: "test".into(), cpu_model: "test".into(),
                logical_cpus_available: 1, physical_memory: "test".into(), process_cpu_affinity: None, cgroup_limits: vec![], machine_label: options.machine_label },
            configuration: Configuration { backend: "wasm-aot-emitted-runtime".into(), fixture_root: "test".into(), working_directory: "test".into(),
                host_surface: "product".into(), compilation_jobs: 1, cache_paths_and_limits: vec![], environment: vec![], idle_machine_acknowledged: true },
            measurement_policy: POLICY.into(), corpus_sha256: "0".repeat(64), fixtures: vec![fixture], gates: vec![], samples_per_gate: 3 });
        let mut report = RuntimeReport { version: 2, measurement_policy: POLICY.into(), measurement_scope: SCOPE.into(),
            runtime_policy: wasm_runtime_profile_policy(), identity, completion: Completion::Complete,
            cases: vec![case], summaries: vec![], comparison: None };
        report.summaries = report.summaries().unwrap();
        let original = report.summaries.clone();
        report.summaries[0].gc_capacity_after_execution.median_bytes += 1;
        assert_ne!(report.summaries().unwrap(), report.summaries);
        assert!(report.verify_evidence(Path::new("unused/report.json")).is_err());
        report.summaries = original;
        report.version = 1;
        assert!(report.summaries().is_err());
        report.version = 2;
        report.completion = Completion::Running;
        assert!(report.summaries().is_err());
        report.completion = Completion::Failed { reason: "real trap".into() };
        assert!(report.summaries().is_err());
    }
}
