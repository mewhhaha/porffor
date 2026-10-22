//! Explicit native-policy cohorts; separate from the ordinary V8 score.
use scriptw_engine::{
    CompileOptions, CompilerIdentity, CompilerSourceRevision, Engine, ExecutionBackend,
    HostOutputEvent, ObservedCompletion, RealmBuilder, WasmProcessMemoryObservation,
    WasmProfileNativeMode,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};

fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 6 {
        return Err("usage: native-policy-probe FIXTURES SAMPLES WARMUPS CASE fast|size-optimized ARTIFACT_DIR".into());
    }
    let samples: usize = args[1].parse().map_err(|_| "invalid samples")?;
    let warmups: usize = args[2].parse().map_err(|_| "invalid warmups")?;
    if !(3..=100).contains(&samples) || !(1..=100).contains(&warmups) {
        return Err("samples must be 3..100 and warmups 1..100".into());
    }
    let requested = match args[4].as_str() {
        "fast" => WasmProfileNativeMode::Fast,
        "size-optimized" => WasmProfileNativeMode::SizeOptimized,
        _ => return Err("unknown native mode".into()),
    };
    scriptw_engine::configure_compilation_jobs(1)?;
    let root = PathBuf::from(&args[0]);
    let plan: Value =
        serde_json::from_slice(&fs::read(root.join("manifest.json")).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let cases: Vec<_> = plan["execution"]
        .as_array()
        .ok_or("missing execution inventory")?
        .iter()
        .filter(|case| case["file"] == args[3])
        .collect();
    if cases.len() != 1 {
        return Err("CASE must select exactly one manifest entry".into());
    }
    let case = cases[0];
    let name = case["file"].as_str().ok_or("missing filename")?;
    if PathBuf::from(name).components().count() != 1 || !name.ends_with(".js") {
        return Err("fixture must be a basename".into());
    }
    let expected = case["stdout"].as_str().ok_or("missing expected stdout")?;
    let source = fs::read_to_string(root.join(name)).map_err(|e| e.to_string())?;
    let engine = Engine::new(RealmBuilder::new().build());
    let compilation = engine
        .profile_script_compilation(
            &source,
            CompileOptions {
                filename: Some(name.into()),
                ..CompileOptions::default()
            },
        )
        .map_err(|e| e.to_string())?;
    let path = PathBuf::from(&args[5]).join(format!("{name}.wasm"));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    std::io::Write::write_all(&mut file, &compilation.artifact.bytes).map_err(|e| e.to_string())?;
    let mut observations = Vec::new();
    for index in 0..warmups + samples {
        let p = engine
            .profile_wasm_execution_with_native_mode(
                &compilation.artifact,
                Some(60_000),
                false,
                requested,
            )
            .map_err(|e| e.to_string())?;
        let mut stdout = String::new();
        for event in &p.outcome.output_events {
            match event {
                HostOutputEvent::PrintLine(line) => {
                    stdout.push_str(line);
                    stdout.push('\n');
                }
                HostOutputEvent::ReportedException(line) => {
                    return Err(format!("reported exception: {line}"))
                }
            }
        }
        if p.native_mode != requested
            || p.outcome.backend_used != ExecutionBackend::WasmAot
            || !matches!(p.outcome.completion, ObservedCompletion::Normal(_))
            || stdout != expected
        {
            return Err(format!(
                "cohort execution differs: {:?}; output {stdout:?}",
                p.outcome
            ));
        }
        let rss = match &p.process_memory {
            WasmProcessMemoryObservation::Sampled(s) => {
                json!({"status":"sampled","scope":"runtime-invocation-process-rss","initial_bytes":s.initial_rss_bytes(),"final_bytes":s.final_rss_bytes(),"maximum_bytes":s.maximum_rss_bytes(),"sample_count":s.sample_count()})
            }
            WasmProcessMemoryObservation::Unavailable(reason) => {
                json!({"status":"unavailable","reason":format!("{reason:?}")})
            }
        };
        observations.push(json!({"warmup":index<warmups,"actual_native_mode":p.native_mode.as_str(),"runtime_policy":p.runtime_policy,"stdout":stdout,
            "module_cache":format!("{:?}",p.module_cache),"execution_ns":p.timings.execution_and_completion.as_nanos(),"total_ns":p.timings.total.as_nanos(),"engine_ns":p.timings.engine.as_nanos(),"module_ns":p.timings.module.as_nanos(),"instantiate_ns":p.timings.instantiate.as_nanos(),
            "rss":rss,"gc_capacity_before_bytes":p.heap.capacity_before_execution_bytes,"gc_capacity_after_bytes":p.heap.capacity_after_execution_bytes}));
    }
    let identity = CompilerIdentity::current()?;
    let revision = match identity.source_revision() {
        CompilerSourceRevision::GitCommit(commit) => Some(commit.as_str()),
        CompilerSourceRevision::UnversionedArchive => None,
    };
    println!("{}",serde_json::to_string(&json!({"schema":"scriptw-native-policy-cohort-v1","source_commit":revision,"source_fingerprint":identity.source_fingerprint().to_string(),"executable_sha256":identity.executable_sha256().to_string(),"source_compilation_profile":"portable","requested_native_mode":requested.as_str(),"samples":samples,"warmups":warmups,"case":name,"source_sha256":format!("{:x}",Sha256::digest(source.as_bytes())),"wasm_sha256":format!("{:x}",Sha256::digest(&compilation.artifact.bytes)),"wasm_bytes":compilation.artifact.bytes.len(),"observations":observations,"boundary":"Explicit forced native-policy cohort; fresh Store/realm; not an ordinary V8 score; forced Fast fails rather than silently falling back."})).map_err(|e|e.to_string())?);
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
