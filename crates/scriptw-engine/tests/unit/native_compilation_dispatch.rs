use super::*;

#[test]
fn sized_worker_stack_admission_is_thread_local_and_restores_after_unwinding() {
    assert!(!ENGINE_WORKER_STACK_ACTIVE.get());
    run_on_sized_stack(|| {
        assert!(ENGINE_WORKER_STACK_ACTIVE.get());
        let panic = std::panic::catch_unwind(|| {
            let _nested = EngineWorkerStackGuard::enter();
            assert!(ENGINE_WORKER_STACK_ACTIVE.get());
            panic!("exercise nested admission unwinding");
        });
        assert!(panic.is_err());
        assert!(ENGINE_WORKER_STACK_ACTIVE.get());
        assert!(!std::thread::spawn(|| ENGINE_WORKER_STACK_ACTIVE.get())
            .join()
            .unwrap());
    });
    assert!(!ENGINE_WORKER_STACK_ACTIVE.get());
}

#[test]
fn serial_native_compilation_stays_on_the_admitted_worker() {
    run_on_sized_stack(|| {
        let caller = std::thread::current().id();
        let actual = on_native_compilation_thread(1, || std::thread::current().id()).unwrap();
        assert_eq!(actual, caller);
    });
}

#[test]
fn serial_native_compilation_keeps_unmarked_callers_off_small_stacks() {
    assert!(!ENGINE_WORKER_STACK_ACTIVE.get());
    let caller = std::thread::current().id();
    let actual = on_native_compilation_thread(1, || std::thread::current().id()).unwrap();
    assert_ne!(actual, caller);
}

#[test]
fn parallel_native_compilation_retains_the_shared_pool() {
    run_on_sized_stack(|| {
        let caller = std::thread::current().id();
        let actual = on_native_compilation_thread(2, || std::thread::current().id()).unwrap();
        assert_ne!(actual, caller);
    });
}
