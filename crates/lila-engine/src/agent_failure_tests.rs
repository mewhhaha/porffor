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
