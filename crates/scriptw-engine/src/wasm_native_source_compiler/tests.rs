use super::*;
use std::{
    sync::OnceLock,
    time::{Duration, Instant},
};

fn runtime() -> Arc<RuntimeArtifact> {
    static RUNTIME: OnceLock<Arc<RuntimeArtifact>> = OnceLock::new();
    Arc::clone(RUNTIME.get_or_init(|| {
        std::thread::Builder::new()
            .name("native-context-profile-fixture".into())
            .stack_size(64 * 1024 * 1024)
            .spawn(|| {
                scriptw_aot_wasm::runtime_artifact_with_inputs(
                    &scriptw_intl::IntlDataSelection::new(IntlCompilationProfile::Minimal),
                    scriptw_aot_wasm::RuntimeArtifactInputs::new(None)
                        .with_source_compilation_profile(
                            scriptw_aot_wasm::RuntimeSourceCompilationProfile::NativeHost,
                        ),
                )
                .unwrap()
            })
            .unwrap()
            .join()
            .unwrap()
    }))
}

#[test]
fn native_context_retains_original_runtime_execution_and_armed_deadline() {
    let engine = shared_wasm_engine().unwrap();
    let linker = WasmtimeLinker::new(&engine);
    let runtime = runtime();
    let execution = Arc::new(WasmAgentExecutionControl::new());
    execution.arm(Instant::now(), None);
    let context = NativeSourceCompilerContext::new(
        &linker,
        Arc::clone(&runtime),
        WasmCompilationPolicy::from_options(&CompileOptions::default()),
        Arc::clone(&execution),
        WasmNativeCompilationMode::Fast,
    )
    .unwrap();
    assert!(Arc::ptr_eq(context.runtime(), &runtime));
    assert!(Arc::ptr_eq(&context.execution, &execution));
    assert_eq!(context.execution().remaining().unwrap(), None);
    assert!(context.check_budget().is_ok());
}

#[test]
fn native_context_does_not_arm_unarmed_or_refresh_expired_execution() {
    let engine = shared_wasm_engine().unwrap();
    let linker = WasmtimeLinker::new(&engine);
    let runtime = runtime();
    for expired in [false, true] {
        let execution = Arc::new(WasmAgentExecutionControl::new());
        if expired {
            execution.arm(Instant::now() - Duration::from_secs(1), Some(1));
        }
        let result = NativeSourceCompilerContext::new(
            &linker,
            Arc::clone(&runtime),
            WasmCompilationPolicy::from_options(&CompileOptions::default()),
            Arc::clone(&execution),
            WasmNativeCompilationMode::Fast,
        );
        let error = result
            .err()
            .expect("the original invalid deadline is retained");
        assert_eq!(
            error.wasm_execution_failure_kind(),
            Some(if expired {
                WasmExecutionFailureKind::Timeout
            } else {
                WasmExecutionFailureKind::Trap
            })
        );
        assert!(execution.remaining().is_err());
    }
}

#[test]
fn native_context_profile_mismatch_is_an_admission_error_without_syntax_reclassification() {
    let engine = shared_wasm_engine().unwrap();
    let execution = Arc::new(WasmAgentExecutionControl::new());
    execution.arm(Instant::now(), None);
    let error = NativeSourceCompilerContext::new(
        &WasmtimeLinker::new(&engine),
        runtime(),
        WasmCompilationPolicy::from_options(&CompileOptions {
            intl_profile: IntlCompilationProfile::Conformance,
            ..CompileOptions::default()
        }),
        execution,
        WasmNativeCompilationMode::Fast,
    )
    .err()
    .unwrap();
    assert!(error
        .to_string()
        .contains("native source runtime admission failed"));
    assert!(error.parse_diagnostic().is_none());
    assert!(error.ir_diagnostic().is_none());
}
