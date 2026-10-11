use super::*;

fn group_with_worker(worker: WasmAgentWorker) -> WasmAgentGroup {
    group_with_workers(vec![worker])
}

fn group_with_workers(workers: Vec<WasmAgentWorker>) -> WasmAgentGroup {
    let engine = shared_wasm_engine().expect("required runtime for the lifecycle fixture");
    let realm = RealmBuilder::new().build();
    let started_at = realm.host_clock().monotonic_instant();
    let memory = WasmtimeSharedMemory::new(&engine, wasmtime::MemoryType::shared(1, 1))
        .expect("one shared page for the lifecycle fixture");
    let execution_control = WasmAgentExecutionControl::new();
    execution_control.arm(std::time::Instant::now(), None);
    WasmAgentGroup {
        engine,
        native_compilation_mode: WasmNativeCompilationMode::Fast,
        realm,
        shared_memory_backing: WasmSharedMemoryBacking::new(memory),
        prelude: Arc::from(""),
        compile_policy: WasmAgentCompilePolicy::from_root(&CompileOptions::default()),
        timeout_ms: Some(1_000),
        execution_control,
        started_at,
        reports: Mutex::new(VecDeque::new()),
        worker_artifacts: Mutex::new(HashMap::new()),
        workers: Mutex::new(workers),
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
        group
            .broadcast(WasmAgentBroadcast {
                resource: group
                    .shared_memory_backing
                    .allocate(0, 0, false)
                    .expect("valid fixed resource")
                    .expect("bounded resource allocation"),
                id: WasmAgentMessageId::Int32(0),
            })
            .expect("disconnection leaves its worker failure to finish"),
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
fn broadcast_waits_for_every_retrieval_without_holding_the_worker_registry() {
    use std::sync::mpsc::{self, RecvTimeoutError};
    use std::time::Duration;
    let (queued, deliveries) = mpsc::channel();
    let mut workers = Vec::new();
    for _ in 0..2 {
        let (commands, receiver) = mpsc::channel();
        let queued = queued.clone();
        let join = std::thread::spawn(move || {
            match receiver.recv().expect("one broadcast command") {
                WasmAgentCommand::Broadcast {
                    broadcast,
                    retrieved,
                } => {
                    assert_eq!(broadcast.id, WasmAgentMessageId::Int32(-17));
                    queued
                        .send(retrieved)
                        .expect("fixture observes each retrieval gate");
                }
                WasmAgentCommand::Shutdown => panic!("broadcast precedes shutdown"),
            }
            assert!(matches!(receiver.recv(), Ok(WasmAgentCommand::Shutdown)));
            Ok(())
        });
        workers.push(WasmAgentWorker { commands, join });
    }
    drop(queued);
    let group = Arc::new(group_with_workers(workers));
    let broadcast = WasmAgentBroadcast {
        resource: group
            .shared_memory_backing
            .allocate(0, 0, false)
            .expect("fixed native resource")
            .expect("bounded native allocation"),
        id: WasmAgentMessageId::Int32(-17),
    };
    let (returned, result) = mpsc::channel();
    let owner = Arc::clone(&group);
    let broadcast_thread = std::thread::spawn(move || {
        returned
            .send(owner.broadcast(broadcast))
            .expect("owner observes completion");
    });
    // Both commands must be queued before waiting for the first recipient.
    let first = deliveries
        .recv_timeout(Duration::from_secs(5))
        .expect("first recipient queued");
    let second = deliveries
        .recv_timeout(Duration::from_secs(5))
        .expect("second recipient queued");
    assert!(
        group.workers.try_lock().is_ok(),
        "recipients never need the registry lock"
    );
    assert!(matches!(
        result.recv_timeout(Duration::from_millis(30)),
        Err(RecvTimeoutError::Timeout)
    ));
    first
        .send(())
        .expect("first recipient retrieves the resource");
    assert!(matches!(
        result.recv_timeout(Duration::from_millis(30)),
        Err(RecvTimeoutError::Timeout)
    ));
    second
        .send(())
        .expect("second recipient retrieves the resource");
    assert_eq!(
        result
            .recv_timeout(Duration::from_secs(5))
            .expect("all receipts complete")
            .unwrap(),
        2
    );
    broadcast_thread.join().expect("broadcast owner joins");
    group
        .finish()
        .expect("both workers retain normal completion");
}

#[test]
fn a_worker_that_drops_a_queued_receipt_keeps_its_failure_owner() {
    let (commands, receiver) = std::sync::mpsc::channel();
    let join = std::thread::spawn(move || {
        let command = receiver
            .recv()
            .expect("delivery was queued before the failure");
        assert!(matches!(command, WasmAgentCommand::Broadcast { .. }));
        drop(command);
        Err(EngineError::from_runtime_dynamic_source_operation(
            DynamicSourceRuntimeOperation::Eval,
        ))
    });
    let group = group_with_worker(WasmAgentWorker { commands, join });
    let count = group
        .broadcast(WasmAgentBroadcast {
            resource: group
                .shared_memory_backing
                .allocate(0, 0, false)
                .expect("fixed native resource")
                .expect("bounded native allocation"),
            id: WasmAgentMessageId::Int32(0),
        })
        .expect("disconnected receipt remains owned by finish");
    assert_eq!(count, 0);
    assert_eq!(group.workers.lock().expect("worker registry").len(), 1);
    assert_eq!(
        group
            .finish()
            .expect_err("the queued worker failure survives")
            .runtime_dynamic_source_operations(),
        vec![DynamicSourceRuntimeOperation::Eval]
    );
}

#[test]
fn native_broadcast_waits_for_both_workers_to_retrieve_their_resource() {
    configure_compilation_jobs(1).expect("one compiler worker");
    let worker =
        "__lilaAgentSleep(100); __lilaAgentReport('retrieving'); __lilaAgentReceiveBroadcast();";
    let source = format!(
        "__lilaAgentStart({worker:?}); __lilaAgentStart({worker:?}); \
         __lilaAgentBroadcast(new SharedArrayBuffer(4), -17); \
         if (__lilaAgentGetReport() !== 'retrieving') throw new Error('first retrieval'); \
         if (__lilaAgentGetReport() !== 'retrieving') throw new Error('second retrieval'); 42;"
    );
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_wasm_aot_script_with_agents(
            &source,
            CompileOptions {
                host_surface_policy: HostSurfacePolicy::Test262,
                ..CompileOptions::default()
            },
            Some(120_000),
            true,
            String::new(),
        )
        .expect("both workers retrieve before the parent observes their queued reports");
    assert!(outcome.note.contains("42"), "{}", outcome.note);
}

#[test]
fn a_native_broadcast_wait_and_worker_sleep_share_the_parent_timeout() {
    configure_compilation_jobs(1).expect("one compiler worker");
    let worker = "__lilaAgentSleep(600000); __lilaAgentReceiveBroadcast();";
    let source =
        format!("__lilaAgentStart({worker:?}); __lilaAgentBroadcast(new SharedArrayBuffer(4), 0);");
    let started = std::time::Instant::now();
    let error = Engine::new(RealmBuilder::new().build())
        .run_wasm_aot_script_with_agents(
            &source,
            CompileOptions {
                host_surface_policy: HostSurfacePolicy::Test262,
                ..CompileOptions::default()
            },
            Some(10_000),
            true,
            String::new(),
        )
        .expect_err("retrieval and worker cleanup cannot wait for the ten-minute sleep");
    assert_eq!(
        error.wasm_execution_failure_kind(),
        Some(WasmExecutionFailureKind::Timeout)
    );
    assert!(
        started.elapsed() < std::time::Duration::from_secs(60),
        "{error}"
    );
}

#[test]
fn structured_root_throw_and_worker_capability_keep_both_failures() {
    configure_compilation_jobs(1).expect("one bounded compilation worker");
    // The same source and options as the integration regression
    // (`EVAL_WORKER` in tests/aot_realm_modules/aot_dynamic_source_capability.rs) reuse its cached
    // module while exercising the private structured agent seam. The source
    // must stay unknown until run time: a literal such as `'1'` is now an
    // AOT-known indirect-eval source that the worker compiles and runs
    // successfully, which would leave no worker failure to retain.
    let worker = "var holder = { invoke: eval }; \
        var hook = new Proxy(function() {}, {}); hook(); \
        holder.invoke('1/*' + Math.random() + '*/');";
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
                artifact.program(),
                // Cold worker compilation runs inside Agent.start's root
                // execution deadline, as in the integration regression.
                Some(120_000),
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

#[test]
fn native_agents_preserve_each_bigint_id_and_store_local_buffer_wrapper() {
    configure_compilation_jobs(1).expect("one compiler worker");
    let worker = r#"
        var expected = [0n, 0n, 18446744073709551616n,
            -340282366920938463463374607431768211457n, -1, 0, 0];
        var previous;
        BigInt = function () { throw new Error('ambient BigInt conversion'); };
        for (var i = 0; i < expected.length; i++) {
            var message = __lilaAgentReceiveBroadcast();
            if (message[1] !== expected[i]) throw new Error('message ID changed');
            if (typeof message[1] !== typeof expected[i]) throw new Error('ID primitive changed');
            if (!(message[0] instanceof SharedArrayBuffer)) throw new Error('recipient prototype');
            if (message[0] === previous) throw new Error('wrapper reused across deliveries');
            if (new Int32Array(message[0])[0] !== 42) throw new Error('shared backing changed');
            previous = message[0];
        }
        __lilaAgentReport('ids-preserved');
        __lilaAgentLeaving();
    "#;
    let source = format!(
        r#"
        __lilaAgentStart({worker:?}); __lilaAgentStart({worker:?});
        var sab = new SharedArrayBuffer(4); new Int32Array(sab)[0] = 42;
        BigInt.prototype.toString = function () {{ throw new Error('observable ID conversion'); }};
        __lilaAgentBroadcast(sab, 0n);
        __lilaAgentBroadcast(sab, -0n);
        __lilaAgentBroadcast(sab, 18446744073709551616n);
        __lilaAgentBroadcast(sab, -340282366920938463463374607431768211457n);
        var conversions = 0;
        __lilaAgentBroadcast(sab, {{ valueOf: function () {{ conversions++; return 4294967295; }} }});
        if (conversions !== 1) throw new Error('ID converted more than once');
        __lilaAgentBroadcast(sab, NaN);
        __lilaAgentBroadcast(sab, 4294967296);
        var reports = 0;
        while (reports < 2) {{
            var report = __lilaAgentGetReport();
            if (report === null) __lilaAgentSleep(1);
            else {{ if (report !== 'ids-preserved') throw new Error('wrong report'); reports++; }}
        }}
        42;
    "#
    );
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_wasm_aot_script_with_agents(
            &source,
            CompileOptions {
                host_surface_policy: HostSurfacePolicy::Test262,
                ..CompileOptions::default()
            },
            Some(120_000),
            true,
            String::new(),
        )
        .expect("both native workers receive each exact primitive ID and a local wrapper");
    assert!(outcome.note.contains("42"), "{}", outcome.note);
}

#[test]
fn native_agent_broadcast_checks_the_buffer_before_id_coercion() {
    configure_compilation_jobs(1).expect("one compiler worker");
    let source = r#"
        if (__lilaAgentBroadcast.length !== 2) throw new Error('broadcast arity');
        var calls = 0;
        var id = { valueOf: function () { calls++; throw new Error('ID coercion'); } };
        var rejected = false;
        try { __lilaAgentBroadcast({}, id); }
        catch (error) { rejected = error instanceof TypeError; }
        if (!rejected || calls !== 0) throw new Error('buffer check order');
        var sab = new SharedArrayBuffer(4);
        try { __lilaAgentBroadcast(sab, id); }
        catch (error) { if (error.message !== 'ID coercion') throw error; }
        if (calls !== 1) throw new Error('single coercion');
        rejected = false;
        try { __lilaAgentBroadcast(sab, Object(1n)); }
        catch (error) { rejected = error instanceof TypeError; }
        if (!rejected) throw new Error('boxed BigInt is not the primitive branch');
        rejected = false;
        try { __lilaAgentBroadcast(sab, Symbol()); }
        catch (error) { rejected = error instanceof TypeError; }
        if (!rejected) throw new Error('Symbol coercion');
        __lilaAgentBroadcast(sab, 1n);
        42;
    "#;
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_wasm_aot_script_with_agents(
            source,
            CompileOptions {
                host_surface_policy: HostSurfacePolicy::Test262,
                ..CompileOptions::default()
            },
            Some(120_000),
            true,
            String::new(),
        )
        .expect("buffer validation and primitive ID coercion retain their observable order");
    assert!(outcome.note.contains("42"), "{}", outcome.note);
}
