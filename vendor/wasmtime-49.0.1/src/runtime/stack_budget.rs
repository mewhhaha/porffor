//! Native Wasm stack metadata and soft-stack budgets for compiled functions.

/// Post-register-allocation native stack metadata for one defined Wasm function.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WasmFunctionStackLayout {
    stack_size_bound_bytes: u64,
    sp_to_fp_offset_bytes: u32,
}

impl WasmFunctionStackLayout {
    pub(crate) fn new(stack_size_bound_bytes: u64, sp_to_fp_offset_bytes: u32) -> Self {
        Self {
            stack_size_bound_bytes,
            sp_to_fp_offset_bytes,
        }
    }

    /// Return the conservative complete native frame bound in bytes.
    ///
    /// The bound includes Cranelift's post-register-allocation incoming,
    /// tail-call, setup, callee-save, fixed-storage, and outgoing-argument
    /// areas. This can exceed the actual downward SP movement of the frame.
    pub const fn stack_size_bound_bytes(self) -> u64 {
        self.stack_size_bound_bytes
    }

    /// Return the active-frame distance from SP up to the preserved FP.
    pub const fn sp_to_fp_offset_bytes(self) -> u32 {
        self.sp_to_fp_offset_bytes
    }
}

/// The remaining Wasmtime soft Wasm stack at a generated guard wrapper.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WasmStackBudget {
    remaining_bytes: usize,
    target_frame: WasmFunctionStackLayout,
    guard_wrapper_frame: WasmFunctionStackLayout,
}

impl WasmStackBudget {
    pub(crate) fn new(
        remaining_bytes: usize,
        target_frame: WasmFunctionStackLayout,
        guard_wrapper_frame: WasmFunctionStackLayout,
    ) -> Self {
        Self {
            remaining_bytes,
            target_frame,
            guard_wrapper_frame,
        }
    }

    /// Return bytes remaining between the wrapper's active Wasm SP and
    /// Wasmtime's configured soft Wasm stack limit.
    ///
    /// The value is measured from the Wasm wrapper frame, before returning
    /// from the host callback. Host callback stack use is outside this soft
    /// Wasm budget and relies on Wasmtime's native host-stack headroom.
    pub const fn remaining_bytes(self) -> usize {
        self.remaining_bytes
    }

    /// Return the target body's conservative native-frame bound.
    pub const fn target_frame(self) -> WasmFunctionStackLayout {
        self.target_frame
    }

    /// Return the guard wrapper's conservative native-frame bound.
    pub const fn guard_wrapper_frame(self) -> WasmFunctionStackLayout {
        self.guard_wrapper_frame
    }
}
