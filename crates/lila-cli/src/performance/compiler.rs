//! Separate compiler-stage evidence uses the same corpus and host identities.
use super::*;
use lila_engine::{CompileOptions, Engine, RealmBuilder, ScriptCompilationProfile, WasmArtifactFootprint};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const PROFILE_POLICY: &str = "product-compiler-stages-v1;serial;one-warmup-per-case;no-program-cache;wasmtime-product-validation";
pub(super) const ARTIFACT_MAX_BYTES: u64 = 512 * 1024 * 1024;
const SCOPE: &str = "prepare-includes-parse-and-dependency-admission;lower-includes-spec-ir;emit-includes-wasm-and-metadata;validation-excludes-engine-setup;no-js-execution-or-runtime-memory-metrics";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Stage { Prepare, Lower, Emit, Validate }
impl Stage {
    const ALL: [Self; 4] = [Self::Prepare, Self::Lower, Self::Emit, Self::Validate];
    fn index(self) -> usize {
        match self { Self::Prepare => 0, Self::Lower => 1, Self::Emit => 2, Self::Validate => 3 }
    }
}

use super::report::ProfileIdentity as CompilerIdentity;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Measurement {
    stage_ns: [u64; 4],
    artifact: Log,
    footprint: serde_json::Value,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    fixture: String,
    warmup: Option<Measurement>,
    samples: Vec<Measurement>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StageSummary { stage: Stage, timing: stats::Summary }
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CaseSummary { fixture: String, stages: Vec<StageSummary>, footprint: serde_json::Value }
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StageChange { stage: Stage, timing: stats::Change }
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CaseChange {
    fixture: String,
    stages: Vec<StageChange>,
    baseline_footprint: serde_json::Value,
    candidate_footprint: serde_json::Value,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Comparison { baseline_sha256: String, baseline_compiler: CompilerProvenance, cases: Vec<CaseChange> }
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CompilerReport {
    version: u32,
    measurement_policy: String,
    measurement_scope: String,
    identity: CompilerIdentity,
    completion: Completion,
    cases: Vec<Case>,
    summaries: Vec<CaseSummary>,
    comparison: Option<Comparison>,
}

pub(super) fn footprint(value: WasmArtifactFootprint) -> serde_json::Value {
    let WasmArtifactFootprint { module_bytes, imported_functions, defined_functions,
        code_body_bytes, largest_code_body_bytes, identical_extra_bodies,
        identical_extra_body_bytes, data_payload_bytes, custom_sections } = value;
    serde_json::json!({
        "module_bytes": module_bytes, "imported_functions": imported_functions,
        "defined_functions": defined_functions, "code_body_bytes": code_body_bytes,
        "largest_code_body_bytes": largest_code_body_bytes, "identical_extra_bodies": identical_extra_bodies,
        "identical_extra_body_bytes": identical_extra_body_bytes, "data_payload_bytes": data_payload_bytes,
        "custom_sections": custom_sections.into_iter().map(|section| serde_json::json!({
            "name": section.name, "payload_bytes": section.payload_bytes,
        })).collect::<Vec<_>>(),
    })
}

impl CompilerReport {
    fn summaries(&self) -> Result<Vec<CaseSummary>, String> {
        if self.version != 1 || self.measurement_policy != PROFILE_POLICY || self.measurement_scope != SCOPE
            || self.completion != Completion::Complete || !(3..=100).contains(&self.identity.samples_per_case)
            || self.cases.len() != self.identity.fixtures.len() {
            return Err("compiler profile is incomplete or has a foreign measurement contract".into());
        }
        self.cases.iter().zip(&self.identity.fixtures)
            .map(|(case, fixture)| summarize_case(case, fixture, self.identity.samples_per_case)).collect()
    }

    fn verify_evidence(&self, source: &Path) -> Result<(), String> {
        if self.summaries()? != self.summaries { return Err("compiler summaries disagree with retained raw samples".into()); }
        let root = source.parent().unwrap_or_else(|| Path::new("."));
        let mut admitted = BTreeMap::new();
        for fixture in &self.identity.fixtures {
            let mut components = Path::new(&fixture.name).components();
            if !matches!(components.next(), Some(std::path::Component::Normal(_))) || components.next().is_some() {
                return Err("compiler fixture identity must be a single retained filename".into());
            }
            let path = root.join("workload").join(&fixture.name);
            Log { path: format!("workload/{}", fixture.name), bytes: fixture.bytes, sha256: fixture.sha256.clone() }.validate()?;
            let bytes = read_bounded(&path, BASELINE_MAX_BYTES)?;
            if bytes.len() != fixture.bytes || digest(&bytes) != fixture.sha256 { return Err("retained compiler workload changed".into()); }
        }
        for case in &self.cases {
            for sample in case.warmup.iter().chain(&case.samples) {
                sample.artifact.validate()?;
                if !admitted.contains_key(&sample.artifact.path) {
                    let bytes = read_bounded(&root.join(&sample.artifact.path), ARTIFACT_MAX_BYTES)?;
                    let actual = footprint(WasmArtifactFootprint::from_module_bytes(&bytes).map_err(|error| error.to_string())?);
                    admitted.insert(sample.artifact.path.clone(), (bytes.len(), digest(&bytes), actual));
                }
                let (bytes, sha256, actual) = &admitted[&sample.artifact.path];
                if *bytes != sample.artifact.bytes || *sha256 != sample.artifact.sha256 || *actual != sample.footprint {
                    return Err("compiler artifact identity or footprint differs from retained Wasm".into());
                }
            }
        }
        Ok(())
    }
}

fn summarize_case(case: &Case, fixture: &Fixture, samples: usize) -> Result<CaseSummary, String> {
    let warmup = case.warmup.as_ref().ok_or("compiler case has no completed warmup")?;
    if case.fixture != fixture.name || case.samples.len() != samples || !(3..=100).contains(&samples) {
        return Err("compiler profile case inventory differs from its actual source plan".into());
    }
    warmup.artifact.validate()?;
    for sample in &case.samples {
        sample.artifact.validate()?;
        if sample.artifact.sha256 != warmup.artifact.sha256 || sample.artifact.bytes != warmup.artifact.bytes
            || sample.footprint != warmup.footprint {
            return Err(format!("{} produced different Wasm artifacts across repeated compilation", case.fixture));
        }
    }
    let stages = Stage::ALL.into_iter().map(|stage| {
        stats::Summary::from_samples(&case.samples.iter().map(|sample| sample.stage_ns[stage.index()]).collect::<Vec<_>>())
            .map(|timing| StageSummary { stage, timing })
    }).collect::<Result<_, _>>()?;
    Ok(CaseSummary { fixture: case.fixture.clone(), stages, footprint: warmup.footprint.clone() })
}

/// Budget projection follows original raw-sample and artifact admission.
pub(super) fn budget_metrics(path: &Path, bytes: &[u8]) -> Result<super::budget::AdmittedMetrics, String> {
    use super::budget::{AdmittedMetrics, Metric};
    let report: CompilerReport = serde_json::from_slice(bytes).map_err(|error| error.to_string())?;
    report.verify_evidence(path)?;
    let mut result = AdmittedMetrics::new(report.identity);
    for case in report.summaries {
        for stage in case.stages {
            let metric = match stage.stage {
                Stage::Prepare => Metric::CompilerPrepare, Stage::Lower => Metric::CompilerLower,
                Stage::Emit => Metric::CompilerEmit, Stage::Validate => Metric::CompilerValidate,
            };
            result.timing(&case.fixture, metric, &stage.timing)?;
        }
        result.footprint(&case.fixture, &case.footprint)?;
    }
    Ok(result)
}

fn persist_profile(output: &Path, report: &CompilerReport) -> Result<(), String> {
    let next = output.join("report.next.json");
    write_new(&next, &serde_json::to_vec_pretty(report).map_err(|error| error.to_string())?)?;
    fs::rename(&next, output.join("report.json")).map_err(|error| error.to_string())
}

fn measure(engine: &Engine, source: &str, name: &str, output: &Path) -> Result<Measurement, String> {
    let ScriptCompilationProfile { preparation, lowering, emission, validation, artifact, footprint: facts } = engine
        .profile_script_compilation(source, CompileOptions { filename: Some(name.into()),
            module_loading_policy: lila_engine::ModuleLoadingPolicy::RejectAll, ..CompileOptions::default() })
        .map_err(|error| format!("{name}: {error}"))?;
    let mut stage_ns = [0; 4];
    for (result, span) in stage_ns.iter_mut().zip([preparation, lowering, emission, validation]) {
        *result = span.as_nanos().try_into().map_err(|_| "compiler measurement exceeded nanosecond report capacity")?;
    }
    if artifact.bytes.len() as u64 > ARTIFACT_MAX_BYTES { return Err("compiler profile artifact exceeds its retained evidence limit".into()); }
    let sha256 = digest(&artifact.bytes);
    let path = format!("artifacts/{sha256}.wasm");
    let destination = output.join(&path);
    if destination.exists() {
        if read_bounded(&destination, ARTIFACT_MAX_BYTES)? != artifact.bytes { return Err("retained compiler artifact changed".into()); }
    } else { write_new(&destination, &artifact.bytes)?; }
    Ok(Measurement { stage_ns, artifact: Log { path, bytes: artifact.bytes.len(), sha256 }, footprint: footprint(facts) })
}

pub(super) fn command(options: Options) -> Result<PathBuf, String> {
    lila_engine::configure_compilation_jobs(1)?;
    let root = options.fixture_root.canonicalize().map_err(|error| error.to_string())?;
    require_fixtures(&root, &chunk_cases::CASES)?;
    let mut identity = CompilerIdentity::from(capture_identity(&options, &root)?);
    identity.configuration.backend = "wasm-aot-compiler-stages".into();
    let baseline = options.baseline.as_ref().map(|path| {
        let bytes = read_bounded(path, BASELINE_MAX_BYTES)?;
        let report: CompilerReport = serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        let mut comparable = report.identity.clone();
        comparable.compiler = identity.compiler.clone();
        if comparable != identity { return Err("compiler baseline differs in workload, platform, toolchain or resource configuration".into()); }
        report.verify_evidence(path)?;
        Ok::<_, String>((report, bytes))
    }).transpose()?;
    fs::create_dir(&options.output).map_err(|error| format!("compiler report requires a fresh output directory: {error}"))?;
    let output = options.output.canonicalize().map_err(|error| error.to_string())?;
    for directory in ["artifacts", "workload"] { fs::create_dir(output.join(directory)).map_err(|error| error.to_string())?; }
    for (name, source) in chunk_cases::CASES.iter().zip(chunk_cases::SOURCES) { write_new(&output.join("workload").join(name), source.as_bytes())?; }
    if let Some((_, bytes)) = &baseline { write_new(&output.join("baseline.json"), bytes)?; }
    let mut report = CompilerReport { version: 1, measurement_policy: PROFILE_POLICY.into(), measurement_scope: SCOPE.into(),
        cases: identity.fixtures.iter().map(|fixture| Case { fixture: fixture.name.clone(), warmup: None, samples: vec![] }).collect(),
        identity, completion: Completion::Running, summaries: vec![], comparison: None };
    persist_profile(&output, &report)?;
    let engine = Engine::new(RealmBuilder::new().build());
    let result = (|| {
        for (index, source) in chunk_cases::SOURCES.into_iter().enumerate() {
            let name = report.cases[index].fixture.clone();
            report.cases[index].warmup = Some(measure(&engine, source, &name, &output)?);
            persist_profile(&output, &report)?;
            for _ in 0..options.samples {
                report.cases[index].samples.push(measure(&engine, source, &name, &output)?);
                persist_profile(&output, &report)?;
            }
        }
        report.completion = Completion::Complete;
        report.summaries = report.summaries()?;
        if let Some((baseline, bytes)) = &baseline {
            let cases = baseline.summaries.iter().zip(&report.summaries).map(|(before, after)| CaseChange {
                fixture: after.fixture.clone(), baseline_footprint: before.footprint.clone(), candidate_footprint: after.footprint.clone(),
                stages: before.stages.iter().zip(&after.stages).map(|(before, after)| StageChange {
                    stage: after.stage, timing: stats::Change::between(&before.timing, &after.timing),
                }).collect(),
            }).collect();
            report.comparison = Some(Comparison { baseline_sha256: digest(bytes), baseline_compiler: baseline.identity.compiler.clone(), cases });
        }
        Ok::<_, String>(())
    })();
    if let Err(reason) = result {
        report.completion = Completion::Failed { reason: reason.clone() };
        report.summaries.clear(); report.comparison = None;
        persist_profile(&output, &report)?;
        return Err(format!("{reason}; retained {}", output.join("report.json").display()));
    }
    persist_profile(&output, &report)?;
    Ok(output.join("report.json"))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn case() -> (Case, Fixture) {
        let measurement = |stage_ns| Measurement { stage_ns, footprint: serde_json::json!({"module_bytes":8}),
            artifact: Log { path: "artifacts/sample.wasm".into(), bytes: 8, sha256: "0".repeat(64) } };
        (Case { fixture: "sample.js".into(), warmup: Some(measurement([10000;4])),
            samples: vec![measurement([1,10,100,1000]), measurement([9,20,200,2000]), measurement([5,30,300,3000])] },
            Fixture { name: "sample.js".into(), bytes: 1, sha256: "1".repeat(64) })
    }
    #[test]
    fn compiler_stage_distributions_exclude_warmup_and_require_all_artifact_identities() {
        let (case, fixture) = case();
        let summary = summarize_case(&case, &fixture, 3).unwrap();
        assert_eq!(summary.stages.iter().map(|stage| (stage.stage, stage.timing.median_ns)).collect::<Vec<_>>(),
            [(Stage::Prepare,5),(Stage::Lower,20),(Stage::Emit,200),(Stage::Validate,2000)]);
        let mut missing = case.clone(); missing.samples.pop();
        assert!(summarize_case(&missing, &fixture, 3).is_err());
        let mut no_warmup = case.clone(); no_warmup.warmup = None;
        assert!(summarize_case(&no_warmup, &fixture, 3).is_err());
        let mut changed = case.clone(); changed.samples[1].artifact.sha256 = "f".repeat(64);
        assert!(summarize_case(&changed, &fixture, 3).is_err());
        let mut forged = case; forged.samples[2].footprint["module_bytes"] = 9.into();
        assert!(summarize_case(&forged, &fixture, 3).is_err());
    }
    #[test]
    fn compiler_report_admission_requires_idle_acknowledgment_and_its_own_complete_stages() {
        let args = ["compile", "--output-dir", "fresh", "--samples", "3", "--machine-label", "fixture", "--idle-machine"];
        let strings = |args: &[&str]| args.iter().map(|value| (*value).to_owned()).collect();
        let options = parse(strings(&args)).unwrap();
        assert_eq!(options.mode, MeasurementMode::Compiler);
        assert!(options.gates.is_empty());
        assert!(parse(strings(&args[..args.len()-1])).is_err());
        let mut foreign = args.to_vec(); foreign.extend(["--gate", "warm-exact"]);
        assert!(parse(strings(&foreign)).is_err());
    }
}
