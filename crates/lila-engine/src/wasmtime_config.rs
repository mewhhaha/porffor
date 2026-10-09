//! The native compilation settings shared by Cargo's producer and every
//! product Engine. Runtime adapters add cache/pool/diagnostic policy only.

use crate::wasmtime_policy::PRODUCT_WASMTIME_POLICY;
use wasmtime::{Config, OptLevel, RegallocAlgorithm, WasmBacktraceDetails};

pub(crate) const CONFIGURATION_SCHEMA: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum WasmNativeCompilationMode {
    Fast,
    SizeOptimized,
}

impl WasmNativeCompilationMode {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Fast => "fast",
            Self::SizeOptimized => "size-optimized",
        }
    }
}

/// Large AOT native frames use at most half the real compiler/execution stack.
pub(crate) const WASM_MAX_STACK_SIZE: usize = 32 * 1024 * 1024;
/// Both lowering/emission and synchronous Wasm need a real host stack of this size.
pub(crate) const ENGINE_WORKER_STACK_SIZE: usize = 64 * 1024 * 1024;
const _: () = assert!(WASM_MAX_STACK_SIZE > 0 && WASM_MAX_STACK_SIZE < ENGINE_WORKER_STACK_SIZE);

/// Existing per-store cap and linear-memory reservation; embedding changes neither.
pub(crate) const WASM_STORE_MEMORY_CAP_BYTES: usize = 1024 * 1024 * 1024;
const WASM_LINEAR_MEMORY_GUARD_BYTES: u64 = 32 * 1024 * 1024;

pub(crate) fn base_config(
    mode: WasmNativeCompilationMode,
    backtrace_details: WasmBacktraceDetails,
) -> Config {
    let mut config = Config::new();
    config.wasm_backtrace_details(backtrace_details);
    config.cranelift_opt_level(match mode {
        WasmNativeCompilationMode::Fast => OptLevel::None,
        WasmNativeCompilationMode::SizeOptimized => OptLevel::SpeedAndSize,
    });
    config.cranelift_regalloc_algorithm(RegallocAlgorithm::SinglePass);
    config.max_wasm_stack(WASM_MAX_STACK_SIZE);
    config.async_stack_size(ENGINE_WORKER_STACK_SIZE);
    // Shared memories cannot relocate. Their reservation covers the existing
    // StoreLimits cap; ordinary memories retain the same growth/guard policy.
    config.memory_reservation(WASM_STORE_MEMORY_CAP_BYTES as u64);
    config.memory_reservation_for_growth(0);
    config.memory_may_move(true);
    config.memory_guard_size(WASM_LINEAR_MEMORY_GUARD_BYTES);
    config.guard_before_linear_memory(true);
    PRODUCT_WASMTIME_POLICY.configure(&mut config);
    // Every product profile retains the existing execution-time epoch checks.
    config.epoch_interruption(true);
    config
}
