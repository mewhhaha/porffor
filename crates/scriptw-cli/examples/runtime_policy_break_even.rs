//! A single policy per child keeps compiler/allocator retention out of other cohorts.
use scriptw_engine::{
    CompileOptions, CompilerIdentity, CompilerSourceRevision, Engine, ExecutionBackend,
    HostOutputEvent, ObservedCompletion, RealmBuilder, WasmProcessMemoryObservation,
    WasmProfileGcInitialCapacity, WasmProfileInlining, WasmProfileNativeMode,
    WasmProfileRegisterAllocation, WasmRuntimeProfileOptions,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{fs, io::Write, path::Path};

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn policy(options: WasmRuntimeProfileOptions) -> Value {
    json!({"native_mode": options.native_mode().as_str(),
        "gc_initial_mib": options.initial_gc_capacity().mebibytes(),
        "register_allocation": options.register_allocation().as_str(),
        "native_inlining": options.native_inlining().as_str()})
}

fn run() -> Result<Value, String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 9 {
        return Err("usage: runtime_policy_break_even FIXTURES CASE SAMPLES WARMUPS fast|speed|size-optimized 16|64 single-pass|backtracking disabled|bounded NEW_ARTIFACT_DIR".into());
    }
    let samples: usize = args[2].parse().map_err(|_| "invalid samples")?;
    let warmups: usize = args[3].parse().map_err(|_| "invalid warmups")?;
    if !(3..=100).contains(&samples) || !(3..=100).contains(&warmups) {
        return Err("samples/warmups must be 3..100".into());
    }
    let mode = match args[4].as_str() {
        "fast" => WasmProfileNativeMode::Fast,
        "speed" => WasmProfileNativeMode::Speed,
        "size-optimized" => WasmProfileNativeMode::SizeOptimized,
        _ => return Err("unknown native mode".into()),
    };
    let capacity = match args[5].as_str() {
        "16" => WasmProfileGcInitialCapacity::MiB16,
        "64" => WasmProfileGcInitialCapacity::MiB64,
        _ => return Err("capacity must be 16 or 64 MiB".into()),
    };
    let allocation = match args[6].as_str() {
        "single-pass" => WasmProfileRegisterAllocation::SinglePass,
        "backtracking" => WasmProfileRegisterAllocation::Backtracking,
        _ => return Err("unknown register allocator".into()),
    };
    let inlining = match args[7].as_str() {
        "disabled" => WasmProfileInlining::Disabled,
        "bounded" => WasmProfileInlining::Bounded,
        _ => return Err("native inlining must be disabled or bounded".into()),
    };
    let requested =
        WasmRuntimeProfileOptions::new(mode, capacity, allocation).with_native_inlining(inlining);
    scriptw_engine::configure_compilation_jobs(1)?;
    let fixtures = Path::new(&args[0]);
    let plan: Value = serde_json::from_slice(
        &fs::read(fixtures.join("manifest.json")).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    if plan["schema"] != "scriptw-v8-workloads-v1" || Path::new(&args[1]).components().count() != 1
    {
        return Err("invalid fixture plan or case basename".into());
    }
    let selected: Vec<_> = plan["execution"]
        .as_array()
        .ok_or("missing execution plan")?
        .iter()
        .filter(|entry| entry["file"] == args[1])
        .collect();
    if selected.len() != 1 {
        return Err("CASE must select one checked fixture".into());
    }
    let expected = selected[0]["stdout"]
        .as_str()
        .ok_or("missing checked output")?;
    let source = fs::read_to_string(fixtures.join(&args[1])).map_err(|e| e.to_string())?;
    let output = Path::new(&args[8]);
    fs::create_dir(output).map_err(|e| e.to_string())?;
    let engine = Engine::new(RealmBuilder::new().build());
    let compilation = engine
        .profile_script_compilation(
            &source,
            CompileOptions {
                filename: Some(args[1].clone()),
                ..CompileOptions::default()
            },
        )
        .map_err(|e| e.to_string())?;
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output.join("program.wasm"))
        .and_then(|mut file| file.write_all(&compilation.artifact.bytes))
        .map_err(|e| e.to_string())?;
    let mut native = Vec::new();
    // One discarded startup observation; every P compilation still has fresh
    // Engine state and both native code caches disabled.
    for index in 0..samples + 1 {
        let p = engine
            .profile_program_native_compilation(&compilation.artifact, requested)
            .map_err(|e| e.to_string())?;
        if p.options != requested {
            return Err("observed cold compiler policy differs".into());
        }
        native.push(json!({"warmup": index == 0, "actual_policy": policy(p.options), "engine_ns": p.engine_creation.as_nanos(),
            "compile_and_validate_ns": p.compile_and_validate.as_nanos(),
            "native_text_bytes": p.native_text_bytes, "serialized_native_bytes": p.serialized_native_bytes,
            "native_module_cache": "disabled", "incremental_function_cache": "disabled"}));
    }
    let mut execution = Vec::new();
    for index in 0..warmups + samples {
        let p = engine
            .profile_wasm_execution_with_options(
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
        if p.options != requested
            || p.outcome.backend_used != ExecutionBackend::WasmAot
            || !matches!(p.outcome.completion, ObservedCompletion::Normal(_))
            || stdout != expected
        {
            return Err(format!(
                "policy/output differs: {:?}; {stdout:?}",
                p.outcome
            ));
        }
        let memory = match p.process_memory {
            WasmProcessMemoryObservation::Sampled(s) => {
                json!({"status":"sampled", "maximum_rss_bytes": s.maximum_rss_bytes()})
            }
            WasmProcessMemoryObservation::Unavailable(reason) => {
                json!({"status":"unavailable", "reason":format!("{reason:?}")})
            }
        };
        execution.push(json!({"warmup": index < warmups, "actual_policy":policy(p.options), "stdout":stdout,
            "execution_and_completion_ns": p.timings.execution_and_completion.as_nanos(),
            "total_invocation_ns":p.timings.total.as_nanos(), "module_cache":format!("{:?}",p.module_cache),
            "matched_runtime_image_cache":format!("{:?}",p.matched_runtime_image_cache),
            "gc_capacity_before_bytes":p.heap.capacity_before_execution_bytes,
            "gc_capacity_after_bytes":p.heap.capacity_after_execution_bytes, "process_memory":memory,
            "module_loading_ns":{"runtime":p.module_loading.runtime.as_nanos(),
                "program":p.module_loading.program.as_nanos(),
                "runtime_module_cache":p.module_loading.runtime_cache.map(|outcome|format!("{outcome:?}"))}}));
    }
    let identity = CompilerIdentity::current()?;
    let commit = match identity.source_revision() {
        CompilerSourceRevision::GitCommit(commit) => Some(commit.as_str()),
        CompilerSourceRevision::UnversionedArchive => None,
    };
    Ok(
        json!({"schema":"scriptw-native-policy-break-even-v1", "source_commit":commit,
        "source_fingerprint":identity.source_fingerprint().to_string(),
        "executable_sha256":identity.executable_sha256().to_string(), "case":args[1],
        "source_sha256":hash(source.as_bytes()), "program_sha256":hash(&compilation.artifact.bytes),
        "program_bytes":compilation.artifact.bytes.len(),
        "requested_policy":policy(requested),
        "native_inlining_limits":requested.native_inlining().limits().map(|limits|json!({
            "small_callee_size":limits.small_callee_size(),
            "sum_size_threshold":limits.sum_size_threshold(),
            "authority":"pinned Wasmtime 47 source defaults, not Engine threshold counters"})),
        "samples":samples,"warmups":warmups,"native_compilation":native,"execution":execution,
        "linked_runtime":compilation.artifact.runtime.as_ref().map(|r|json!({"key":r.key().as_bytes().iter().map(|b|format!("{b:02x}")).collect::<String>(),"sha256":hash(r.bytes()),"bytes":r.bytes().len()})),
        "boundaries":{"cold_compilation":"P validation/native compilation only; R is neither compiled nor loaded in this span; Engine creation separate; caches disabled",
            "execution":"fresh Store and Realm, warmed explicit matching optimized R; bootstrap/completion included; no isolated steady-state claim",
            "memory":"sampled whole process RSS, including prior P compiler and optimized-R producer allocator retention; not live JS bytes"}}),
    )
}

fn main() {
    match run() {
        Ok(report) => println!(
            "{}",
            serde_json::to_string(&report).expect("JSON report serializes")
        ),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
