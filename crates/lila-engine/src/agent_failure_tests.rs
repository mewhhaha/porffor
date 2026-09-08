use super::*;

fn group_with_worker(worker: WasmAgentWorker) -> WasmAgentGroup {
    let engine = WasmtimeEngine::default();
    let realm = RealmBuilder::new().build();
    let started_at = realm.host_clock().monotonic_instant();
    let memory = WasmtimeSharedMemory::new(&engine, wasmtime::MemoryType::shared(1, 1))
        .expect("one shared page for the lifecycle fixture");
    WasmAgentGroup {
        engine,
        realm,
        shared_memory_backing: Arc::new(WasmSharedMemoryBacking {
            memory,
            next_offset: Mutex::new(8),
            async_waiters: Mutex::new(WasmAgentAsyncWaiterRegistry {
                next_id: 1,
                waiters: VecDeque::new(),
            }),
        }),
        prelude: Arc::from(""),
        compile_policy: WasmAgentCompilePolicy::from_root(&CompileOptions::default()),
        timeout_ms: Some(1_000),
        started_at,
        reports: Mutex::new(VecDeque::new()),
        worker_artifacts: Mutex::new(HashMap::new()),
        workers: Mutex::new(vec![worker]),
    }
}

#[test]
fn broadcast_retains_a_disconnected_worker_until_its_failure_is_joined() {
    let (commands, receiver) = std::sync::mpsc::channel();
    let (disconnected, observed) = std::sync::mpsc::sync_channel(0);
    let join = std::thread::spawn(move || {
        drop(receiver);
        disconnected.send(()).expect("owner observes disconnection");
        Err(EngineError::from_runtime_dynamic_source_operation(
            DynamicSourceRuntimeOperation::Eval,
        ))
    });
    observed.recv().expect("worker channel is already closed");
    let group = group_with_worker(WasmAgentWorker { commands, join });
    assert_eq!(
        group.broadcast(WasmAgentBroadcast {
            data_offset: 0,
            byte_length: 0,
            max_byte_length: 0,
            flags: 0,
        }),
        0
    );
    assert_eq!(group.workers.lock().expect("worker lock").len(), 1);
    let failure = group.finish().expect_err("worker failure cannot disappear");
    assert_eq!(
        failure.runtime_dynamic_source_operations(),
        vec![DynamicSourceRuntimeOperation::Eval]
    );
}

#[test]
fn structured_root_throw_and_worker_capability_keep_both_failures() {
    configure_compilation_jobs(1).expect("one bounded compilation worker");
    // The same source and options as the legacy integration regression reuse
    // its cached module while exercising the private structured agent seam.
    let worker = "var holder = { invoke: eval }; \
        var hook = new Proxy(function() {}, {}); hook(); holder.invoke('1');";
    let source = format!("__lilaAgentStart({worker:?}); throw new TypeError('root marker');");
    let engine = Engine::new(RealmBuilder::new().build());
    let options = CompileOptions {
        host_surface_policy: HostSurfacePolicy::Test262,
        ..CompileOptions::default()
    };
    let failure = run_on_sized_stack(|| {
        let artifact = engine.load_or_compile_program_wasm_on_current_thread(
            &source,
            ParseGoal::Script,
            options.clone(),
            program_wasm_cache(),
        )?;
        engine
            .execute_with_wasm_bytes_inner_with_agents(
                &artifact.bytes,
                Some(30_000),
                true,
                WasmModuleMemoryCachePolicy::BypassRetention,
                Some(WasmAgentHarness {
                    prelude: Arc::from(""),
                    compile_policy: WasmAgentCompilePolicy::from_root(&options),
                }),
                None,
                &WasmExecutionMode::Structured,
            )
            .and_then(WasmExecutionOutcome::into_structured)
    })
    .expect_err("the structured root throw and worker failure must both survive");
    assert_eq!(
        failure.wasm_execution_failure_kind(),
        Some(WasmExecutionFailureKind::ConcurrentFailure)
    );
    assert_eq!(
        failure.runtime_dynamic_source_operations(),
        vec![DynamicSourceRuntimeOperation::Eval]
    );
    assert_eq!(failure.wasm_javascript_exception_constructor_name(), None);
    assert!(failure
        .message()
        .contains("uncaught ECMAScript throw (object)"));
    for private_text in ["TypeError", "root marker", "handle@"] {
        assert!(!failure.message().contains(private_text), "{failure}");
    }
}
