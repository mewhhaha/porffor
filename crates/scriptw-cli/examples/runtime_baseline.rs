//! Opt-in same-source Wasmtime measurements for scripts/benchmark-v8.py.
use scriptw_engine::{
    CompileOptions, CompilerIdentity, CompilerSourceRevision, Engine, ExecutionBackend,
    HostOutputEvent, ObservedCompletion, RealmBuilder,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};

fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 3 {
        return Err("usage: runtime-baseline FIXTURES SAMPLES WARMUPS".into());
    }
    let root = PathBuf::from(&args[0]);
    let samples: usize = args[1].parse().map_err(|_| "invalid sample count")?;
    let warmups: usize = args[2].parse().map_err(|_| "invalid warmup count")?;
    if !(3..=100).contains(&samples) || !(1..=100).contains(&warmups) {
        return Err("samples must be 3..100 and warmups 1..100".into());
    }
    scriptw_engine::configure_compilation_jobs(1)?;
    let plan: Value =
        serde_json::from_slice(&fs::read(root.join("manifest.json")).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let cases = plan["execution"]
        .as_array()
        .ok_or("missing execution inventory")?;
    let engine = Engine::new(RealmBuilder::new().build());
    let mut results = Vec::new();
    for case in cases {
        let name = case["file"].as_str().ok_or("missing filename")?;
        if PathBuf::from(name).components().count() != 1 {
            return Err("fixture must be a basename".into());
        }
        let expected = case["stdout"].as_str().ok_or("missing expected output")?;
        let source = fs::read_to_string(root.join(name)).map_err(|e| e.to_string())?;
        let compilation = engine
            .profile_script_compilation(
                &source,
                CompileOptions {
                    filename: Some(name.into()),
                    ..CompileOptions::default()
                },
            )
            .map_err(|e| e.to_string())?;
        let mut observations = Vec::new();
        for index in 0..warmups + samples {
            let p = engine
                .profile_wasm_execution(&compilation.artifact, Some(60_000), false)
                .map_err(|e| format!("{name}: {e}"))?;
            let mut output = String::new();
            for event in &p.outcome.output_events {
                match event {
                    HostOutputEvent::PrintLine(line) => {
                        output.push_str(line);
                        output.push('\n');
                    }
                    HostOutputEvent::ReportedException(line) => {
                        return Err(format!("{name}: {line}"))
                    }
                }
            }
            if p.outcome.backend_used != ExecutionBackend::WasmAot
                || !matches!(p.outcome.completion, ObservedCompletion::Normal(_))
                || output != expected
            {
                return Err(format!(
                    "{name}: unexpected product execution: {:?}; output {output:?}",
                    p.outcome
                ));
            }
            observations.push(json!({"warmup": index < warmups,
                "execution_ns": p.timings.execution_and_completion.as_nanos(),
                "total_ns": p.timings.total.as_nanos(), "module_ns": p.timings.module.as_nanos(),
                "module_cache": format!("{:?}", p.module_cache), "stdout": output,
                "runtime_policy": p.runtime_policy}));
        }
        results.push(json!({"file": name, "source_sha256": format!("{:x}", Sha256::digest(source.as_bytes())),
            "wasm_sha256": format!("{:x}", Sha256::digest(&compilation.artifact.bytes)),
            "wasm_bytes": compilation.artifact.bytes.len(), "observations": observations}));
    }
    let identity = CompilerIdentity::current()?;
    let revision = match identity.source_revision() {
        CompilerSourceRevision::GitCommit(commit) => Some(commit.as_str()),
        CompilerSourceRevision::UnversionedArchive => None,
    };
    println!(
        "{}",
        serde_json::to_string(&json!({"schema": "scriptw-wasmtime-baseline-v1",
        "source_commit": revision, "source_fingerprint": identity.source_fingerprint().to_string(),
        "executable_sha256": identity.executable_sha256().to_string(),
        "samples": samples, "warmups": warmups, "cases": results}))
        .map_err(|e| e.to_string())?
    );
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
