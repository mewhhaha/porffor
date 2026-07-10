use std::borrow::Cow;

use crate::functions::{
    emit_array_alloc_helper_function, emit_function_object_alloc_helper_function,
};
use crate::objects::{
    emit_object_append_accessor_property_helper_function,
    emit_object_append_data_property_helper_function, emit_plain_object_alloc_helper_function,
};
use porffor_ir::{HostBuiltinId, ProgramIr, ScriptIr, StandardBuiltinId, ValueKind};
use wasm_encoder::{
    CodeSection, ConstExpr, DataSection, ElementSection, Elements, ExportKind, ExportSection,
    Function, FunctionSection, GlobalSection, GlobalType, ImportSection, Instruction,
    MemorySection, MemoryType, Module, RefType, TableSection, TableType, TypeSection, ValType,
};

use super::*;

#[derive(Debug, Clone, Copy)]
pub(crate) enum ControlFrameKind {
    If,
    Block,
    Loop,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct IteratorCloseOnThrowLocals {
    pub(crate) iterator_payload_local: u32,
    pub(crate) iterator_tag_local: u32,
    pub(crate) key_local: u32,
    pub(crate) return_payload_local: u32,
    pub(crate) return_tag_local: u32,
    pub(crate) result_payload_local: u32,
    pub(crate) result_tag_local: u32,
    pub(crate) saved_payload_local: u32,
    pub(crate) saved_tag_local: u32,
    pub(crate) saved_completion_local: u32,
    pub(crate) saved_aux_local: u32,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct LoopTargets {
    pub(crate) continue_frame: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct LabelTargets {
    pub(crate) name: String,
    pub(crate) break_frame: usize,
    pub(crate) continue_frame: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BindingStorage {
    Fixed { payload_local: u32, kind: ValueKind },
    Dynamic { tag_local: u32, payload_local: u32 },
    EnvSlot { slot: u32, hops: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReturnAbi {
    MainExport,
    MultiValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CompletionKind {
    Normal,
    Throw,
    Return,
    Break,
    Continue,
}

impl CompletionKind {
    pub(crate) const fn code(self) -> i64 {
        match self {
            Self::Normal => COMPLETION_KIND_NORMAL,
            Self::Throw => COMPLETION_KIND_THROW,
            Self::Return => COMPLETION_KIND_RETURN,
            Self::Break => COMPLETION_KIND_BREAK,
            Self::Continue => COMPLETION_KIND_CONTINUE,
        }
    }
}

pub(crate) struct FunctionBuilder<'a> {
    pub(crate) body: &'a BlockIr,
    pub(crate) params: &'a [FunctionParamIr],
    pub(crate) owned_env_bindings: &'a [OwnedEnvBindingIr],
    pub(crate) captured_bindings: &'a [porffor_ir::CapturedBindingIr],
    pub(crate) strings: &'a StringPool,
    pub(crate) functions: &'a FunctionMetaRegistry,
    pub(crate) function_id: Option<FunctionId>,
    pub(crate) function_flavor: FunctionFlavor,
    pub(crate) strict: bool,
    pub(crate) self_binding_name: Option<String>,
    pub(crate) script_global_bindings: BTreeMap<String, ScriptGlobalBindingKind>,
    pub(crate) uses_heap: bool,
    pub(crate) return_abi: ReturnAbi,
    pub(crate) binding_scopes: Vec<BTreeMap<String, BindingStorage>>,
    pub(crate) hoisted_vars: Vec<String>,
    pub(crate) next_binding_local: u32,
    pub(crate) total_binding_local_count: u32,
    pub(crate) temp_local_count: u32,
    pub(crate) current_env_local: u32,
    pub(crate) result_local: u32,
    pub(crate) result_tag_local: u32,
    pub(crate) completion_local: u32,
    pub(crate) completion_aux_local: u32,
    pub(crate) derived_this_initialized_local: Option<u32>,
    pub(crate) scratch_local: u32,
    pub(crate) temp_local_base: u32,
    pub(crate) temp_stack_depth: u32,
    pub(crate) this_payload_local: Option<u32>,
    pub(crate) this_tag_local: Option<u32>,
    pub(crate) control_stack: Vec<ControlFrameKind>,
    pub(crate) breakable_stack: Vec<usize>,
    pub(crate) loop_stack: Vec<LoopTargets>,
    pub(crate) label_stack: Vec<LabelTargets>,
    pub(crate) throw_handler_stack: Vec<usize>,
    pub(crate) finally_stack: Vec<usize>,
    pub(crate) stub_standard_builtin_body: bool,
    pub(crate) runtime_bootstrap_plan: RuntimeBootstrapPlan,
    pub(crate) heap_alloc_function_index: Option<u32>,
    pub(crate) object_append_data_property_function_index: Option<u32>,
    pub(crate) object_append_accessor_property_function_index: Option<u32>,
    pub(crate) function_object_alloc_function_index: Option<u32>,
    pub(crate) plain_object_alloc_function_index: Option<u32>,
    pub(crate) array_alloc_function_index: Option<u32>,
    /// When false, `emit_object_read_ordinary` inlines its body instead of
    /// emitting a call to the shared object-read runtime helper. Set false only
    /// while compiling the object-read helper itself (to avoid self-recursion).
    pub(crate) outline_object_read: bool,
    /// When false, `emit_object_write` inlines its body instead of emitting a
    /// call to the shared object-write runtime helper. Set false only while
    /// compiling the object-write helper itself.
    pub(crate) outline_object_write: bool,
    /// When false, `emit_object_define_data_with_flag_locals` inlines its body
    /// instead of emitting a call to the shared object-define-data helper. Set
    /// false only while compiling that helper itself. Realm/global bootstrap
    /// defines hundreds of data properties, so outlining this keeps the
    /// bootstrap-style functions well under Cranelift's per-function limit.
    pub(crate) outline_object_define_data: bool,
    /// When false, `emit_function_or_proxy_call_with_argv_inner` inlines the
    /// proxy-aware call-dispatch state machine instead of calling the shared
    /// helper. Only the proxy-enabled dispatch is outlined; the plain call path
    /// stays inline. Set false only while compiling that helper itself.
    pub(crate) outline_proxy_call: bool,
    /// When false, `emit_function_or_proxy_construct_with_argv` inlines the
    /// proxy-aware construct-dispatch state machine instead of calling the
    /// shared helper. Set false only while compiling that helper itself.
    pub(crate) outline_proxy_construct: bool,
    /// When false, `emit_string_payload_equality_i32` inlines its byte-compare
    /// loop instead of calling the shared string-equality helper. Set false only
    /// while compiling that helper itself. Builtin bodies compare interned
    /// string payloads at thousands of sites (property-name matching, key
    /// switches), and the inline loop is ~65 instructions per site, so outlining
    /// it keeps the largest builtin bodies under Cranelift's per-function
    /// virtual-register limit.
    pub(crate) outline_string_equality: bool,
    /// When false, `emit_number_to_string_payload` inlines its digit-emission
    /// state machine instead of calling the shared helper. Set false only while
    /// compiling that helper itself. Number formatting appears in nearly every
    /// builtin (ToString of numeric results, join/serialize paths), and the
    /// inline expansion is several KB per site.
    pub(crate) outline_number_to_string: bool,
    /// When false, `emit_string_to_number_payload` inlines its parse state
    /// machine instead of calling the shared helper. Set false only while
    /// compiling that helper itself. String-to-number parsing (ToNumber of
    /// string operands) is similarly several KB per inline site.
    pub(crate) outline_string_to_number: bool,
    /// When false, `emit_value_to_string_payload` inlines the full dynamic
    /// ToString composite (per-kind dispatch, ToPrimitive on objects, array
    /// join, function source text) instead of calling the shared helper. Set
    /// false only while compiling that helper itself. Every dynamic string
    /// concatenation and ToString site otherwise pays tens of KB inline.
    pub(crate) outline_value_to_string: bool,
}

pub fn emit(program: &ProgramIr) -> Result<WasmArtifact, EmitError> {
    let script = program.script.as_ref().ok_or_else(|| {
        EmitError::unsupported("unsupported in porffor wasm-aot first slice: no lowered script ir")
    })?;
    if let Some(diagnostic) = program
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.kind == porffor_ir::IrDiagnosticKind::Unsupported)
    {
        return Err(EmitError::unsupported(diagnostic.message.clone()));
    }
    emit_script(script)
}

fn emit_script(script: &ScriptIr) -> Result<WasmArtifact, EmitError> {
    // Emission fixpoint over the builtin stub partitions (standard and host).
    // The seed partitions come from the script text
    // (`should_stub_standard_builtin`; host builtins the script references).
    // Each pass records, via `FunctionMetaRegistry`, every builtin whose
    // function value was materialized somewhere (its funcref-table slot became
    // runtime-reachable) or whose body was direct-called. Any such builtin
    // that was stubbed this pass is force-compiled on the next pass, so no
    // reachable function value can ever resolve to the shared "not emitted"
    // stub. The forced sets grow monotonically and are bounded by the builtin
    // counts, so the loop terminates; in practice it converges in 2-4 passes.
    let mut forced = ForcedBuiltins::default();
    loop {
        let (artifact, touched_stubbed) = emit_script_with_forced_builtins(script, &forced)?;
        if touched_stubbed.standard.is_empty() && touched_stubbed.host.is_empty() {
            return Ok(artifact);
        }
        forced.standard.extend(touched_stubbed.standard);
        forced.host.extend(touched_stubbed.host);
    }
}

/// Builtins whose real bodies must be emitted regardless of what the script
/// text references, because a previous emission pass proved they are
/// dynamically reachable (see `emit_script`).
#[derive(Default)]
struct ForcedBuiltins {
    standard: BTreeSet<StandardBuiltinId>,
    host: BTreeSet<HostBuiltinId>,
}

fn emit_script_with_forced_builtins(
    script: &ScriptIr,
    forced: &ForcedBuiltins,
) -> Result<(WasmArtifact, ForcedBuiltins), EmitError> {
    let uses_heap = true;
    let uses_shared_memory = script_references_memory_atomics(script);
    let mut compiled_host_builtins = script.host_builtins.clone();
    for builtin in all_host_builtins() {
        if forced.host.contains(builtin) && !compiled_host_builtins.contains(builtin) {
            compiled_host_builtins.push(*builtin);
        }
    }
    let stubbed_host_builtins = all_host_builtins()
        .iter()
        .copied()
        .filter(|builtin| !compiled_host_builtins.contains(builtin))
        .collect::<Vec<_>>();
    let uses_host_print = compiled_host_builtins.contains(&HostBuiltinId::Print);
    let imported_function_count = u32::from(uses_host_print);
    let mut compiled_standard_builtins = Vec::new();
    let mut stubbed_standard_builtins = Vec::new();
    for builtin in StandardBuiltinId::all_functions() {
        if !forced.standard.contains(builtin) && should_stub_standard_builtin(script, *builtin) {
            stubbed_standard_builtins.push(*builtin);
        } else {
            compiled_standard_builtins.push(*builtin);
        }
    }
    let uses_json_stringify =
        compiled_standard_builtins.contains(&StandardBuiltinId::JsonStringify);
    let runtime_bootstrap_plan =
        RuntimeBootstrapPlan::from_script(script, &compiled_standard_builtins);
    let has_shared_stub =
        !stubbed_standard_builtins.is_empty() || !stubbed_host_builtins.is_empty();
    let function_metas = FunctionMetaRegistry::new(build_function_metas(
        script.functions.as_slice(),
        &compiled_standard_builtins,
        &stubbed_standard_builtins,
        &compiled_host_builtins,
        &stubbed_host_builtins,
        imported_function_count,
    ));
    let emitted_standard_builtins = emitted_compiled_standard_builtins(&compiled_standard_builtins);
    let string_pool = StringPool::collect(script, function_metas.metas());
    let uses_function_table = true;
    let callable_function_count = script.functions.len()
        + emitted_standard_builtins.len()
        + usize::from(has_shared_stub)
        + compiled_host_builtins.len();
    let heap_alloc_function_index =
        uses_heap.then_some(imported_function_count + 1 + callable_function_count as u32);
    let object_append_data_property_function_index =
        heap_alloc_function_index.map(|heap_alloc_function_index| heap_alloc_function_index + 1);
    let object_append_accessor_property_function_index = object_append_data_property_function_index
        .map(|append_function_index| append_function_index + 1);
    let function_object_alloc_function_index = object_append_accessor_property_function_index
        .map(|append_function_index| append_function_index + 1);
    let plain_object_alloc_function_index = function_object_alloc_function_index
        .map(|function_object_alloc_function_index| function_object_alloc_function_index + 1);
    let array_alloc_function_index = plain_object_alloc_function_index
        .map(|plain_object_alloc_function_index| plain_object_alloc_function_index + 1);
    let mut main_builder = FunctionBuilder::new_main(
        script,
        &string_pool,
        &function_metas,
        uses_heap,
        runtime_bootstrap_plan.clone(),
        heap_alloc_function_index,
        object_append_data_property_function_index,
        object_append_accessor_property_function_index,
        function_object_alloc_function_index,
        plain_object_alloc_function_index,
        array_alloc_function_index,
    );
    let main_function = main_builder.compile()?;
    let mut compiled_functions = Vec::with_capacity(callable_function_count);
    for function in &script.functions {
        let mut builder = FunctionBuilder::new_function(
            function,
            &script.global_bindings,
            &string_pool,
            &function_metas,
            uses_heap,
            runtime_bootstrap_plan.clone(),
            heap_alloc_function_index,
            object_append_data_property_function_index,
            object_append_accessor_property_function_index,
            function_object_alloc_function_index,
            plain_object_alloc_function_index,
            array_alloc_function_index,
        );
        compiled_functions.push(builder.compile()?);
    }
    for builtin in &emitted_standard_builtins {
        let mut builder = FunctionBuilder::new_standard_builtin(
            *builtin,
            &string_pool,
            &function_metas,
            uses_heap,
            false,
            runtime_bootstrap_plan.clone(),
            heap_alloc_function_index,
            object_append_data_property_function_index,
            object_append_accessor_property_function_index,
            function_object_alloc_function_index,
            plain_object_alloc_function_index,
            array_alloc_function_index,
        );
        compiled_functions.push(builder.compile_builtin()?);
    }
    if has_shared_stub {
        let stub_builtin = stubbed_standard_builtins
            .first()
            .copied()
            .unwrap_or(StandardBuiltinId::FunctionConstructor);
        let mut builder = FunctionBuilder::new_standard_builtin(
            stub_builtin,
            &string_pool,
            &function_metas,
            uses_heap,
            true,
            runtime_bootstrap_plan.clone(),
            heap_alloc_function_index,
            object_append_data_property_function_index,
            object_append_accessor_property_function_index,
            function_object_alloc_function_index,
            plain_object_alloc_function_index,
            array_alloc_function_index,
        );
        compiled_functions.push(builder.compile_builtin()?);
    }
    for builtin in &compiled_host_builtins {
        let mut builder = FunctionBuilder::new_host_builtin(
            *builtin,
            &string_pool,
            &function_metas,
            uses_heap,
            heap_alloc_function_index,
            object_append_data_property_function_index,
            object_append_accessor_property_function_index,
            function_object_alloc_function_index,
            plain_object_alloc_function_index,
            array_alloc_function_index,
        );
        compiled_functions.push(builder.compile_builtin()?);
    }

    // Shared object-read / object-write runtime helpers. These carry the large
    // property-access state machines that would otherwise be inlined at every
    // read/write site, blowing single functions past Cranelift's per-function
    // code-size limit. They are emitted once, directly after the heap helpers,
    // and are reached with plain `call`s (never through the funcref table).
    let object_read_helper_function = uses_heap
        .then(|| {
            let mut builder = FunctionBuilder::new_runtime_operation_helper(
                &string_pool,
                &function_metas,
                uses_heap,
                runtime_bootstrap_plan.clone(),
                heap_alloc_function_index,
                object_append_data_property_function_index,
                object_append_accessor_property_function_index,
                function_object_alloc_function_index,
                plain_object_alloc_function_index,
                array_alloc_function_index,
            );
            builder.compile_object_read_helper()
        })
        .transpose()?;
    let object_write_helper_function = uses_heap
        .then(|| {
            let mut builder = FunctionBuilder::new_runtime_operation_helper(
                &string_pool,
                &function_metas,
                uses_heap,
                runtime_bootstrap_plan.clone(),
                heap_alloc_function_index,
                object_append_data_property_function_index,
                object_append_accessor_property_function_index,
                function_object_alloc_function_index,
                plain_object_alloc_function_index,
                array_alloc_function_index,
            );
            builder.compile_object_write_helper()
        })
        .transpose()?;
    let object_define_data_helper_function = uses_heap
        .then(|| {
            let mut builder = FunctionBuilder::new_runtime_operation_helper(
                &string_pool,
                &function_metas,
                uses_heap,
                runtime_bootstrap_plan.clone(),
                heap_alloc_function_index,
                object_append_data_property_function_index,
                object_append_accessor_property_function_index,
                function_object_alloc_function_index,
                plain_object_alloc_function_index,
                array_alloc_function_index,
            );
            builder.compile_object_define_data_helper()
        })
        .transpose()?;
    let proxy_call_helper_function = uses_heap
        .then(|| {
            let mut builder = FunctionBuilder::new_runtime_operation_helper(
                &string_pool,
                &function_metas,
                uses_heap,
                runtime_bootstrap_plan.clone(),
                heap_alloc_function_index,
                object_append_data_property_function_index,
                object_append_accessor_property_function_index,
                function_object_alloc_function_index,
                plain_object_alloc_function_index,
                array_alloc_function_index,
            );
            builder.compile_proxy_call_helper()
        })
        .transpose()?;
    let proxy_construct_helper_function = uses_heap
        .then(|| {
            let mut builder = FunctionBuilder::new_runtime_operation_helper(
                &string_pool,
                &function_metas,
                uses_heap,
                runtime_bootstrap_plan.clone(),
                heap_alloc_function_index,
                object_append_data_property_function_index,
                object_append_accessor_property_function_index,
                function_object_alloc_function_index,
                plain_object_alloc_function_index,
                array_alloc_function_index,
            );
            builder.compile_proxy_construct_helper()
        })
        .transpose()?;
    let string_equality_helper_function = uses_heap
        .then(|| {
            let mut builder = FunctionBuilder::new_runtime_operation_helper(
                &string_pool,
                &function_metas,
                uses_heap,
                runtime_bootstrap_plan.clone(),
                heap_alloc_function_index,
                object_append_data_property_function_index,
                object_append_accessor_property_function_index,
                function_object_alloc_function_index,
                plain_object_alloc_function_index,
                array_alloc_function_index,
            );
            builder.compile_string_equality_helper()
        })
        .transpose()?;
    let number_to_string_helper_function = uses_heap
        .then(|| {
            let mut builder = FunctionBuilder::new_runtime_operation_helper(
                &string_pool,
                &function_metas,
                uses_heap,
                runtime_bootstrap_plan.clone(),
                heap_alloc_function_index,
                object_append_data_property_function_index,
                object_append_accessor_property_function_index,
                function_object_alloc_function_index,
                plain_object_alloc_function_index,
                array_alloc_function_index,
            );
            builder.compile_number_to_string_helper()
        })
        .transpose()?;
    let string_to_number_helper_function = uses_heap
        .then(|| {
            let mut builder = FunctionBuilder::new_runtime_operation_helper(
                &string_pool,
                &function_metas,
                uses_heap,
                runtime_bootstrap_plan.clone(),
                heap_alloc_function_index,
                object_append_data_property_function_index,
                object_append_accessor_property_function_index,
                function_object_alloc_function_index,
                plain_object_alloc_function_index,
                array_alloc_function_index,
            );
            builder.compile_string_to_number_helper()
        })
        .transpose()?;
    let value_to_string_helper_function = uses_heap
        .then(|| {
            let mut builder = FunctionBuilder::new_runtime_operation_helper(
                &string_pool,
                &function_metas,
                uses_heap,
                runtime_bootstrap_plan.clone(),
                heap_alloc_function_index,
                object_append_data_property_function_index,
                object_append_accessor_property_function_index,
                function_object_alloc_function_index,
                plain_object_alloc_function_index,
                array_alloc_function_index,
            );
            builder.compile_value_to_string_helper()
        })
        .transpose()?;
    let json_stringify_value_helper_function = (uses_heap && uses_json_stringify)
        .then(|| {
            let mut builder = FunctionBuilder::new_runtime_operation_helper(
                &string_pool,
                &function_metas,
                uses_heap,
                runtime_bootstrap_plan.clone(),
                heap_alloc_function_index,
                object_append_data_property_function_index,
                object_append_accessor_property_function_index,
                function_object_alloc_function_index,
                plain_object_alloc_function_index,
                array_alloc_function_index,
            );
            builder.compile_json_stringify_value_helper()
        })
        .transpose()?;

    let mut types = TypeSection::new();
    types.ty().function([], [ValType::I64]);
    if uses_function_table {
        types.ty().function(
            function_param_types(),
            [ValType::I64, ValType::I64, ValType::I64, ValType::I64],
        );
    }
    types.ty().function([ValType::I64], [ValType::I64]);
    types.ty().function(
        [
            ValType::I64,
            ValType::I64,
            ValType::I64,
            ValType::I64,
            ValType::I64,
        ],
        [],
    );
    types.ty().function(
        [
            ValType::I64,
            ValType::I64,
            ValType::I64,
            ValType::I64,
            ValType::I64,
            ValType::I64,
            ValType::I64,
        ],
        [],
    );
    types.ty().function(
        [
            ValType::I64,
            ValType::I64,
            ValType::I64,
            ValType::I64,
            ValType::I64,
            ValType::I64,
            ValType::I64,
            ValType::I64,
            ValType::I64,
        ],
        [ValType::I64],
    );
    types
        .ty()
        .function([ValType::I64, ValType::I64], [ValType::I64]);
    types
        .ty()
        .function([ValType::I64], [ValType::I64, ValType::I64]);
    if uses_host_print {
        types.ty().function([ValType::I32, ValType::I32], []);
    }

    let main_wasm_index = imported_function_count;

    let mut functions = FunctionSection::new();
    functions.function(0);
    for _ in 0..callable_function_count {
        functions.function(JS_FUNCTION_TYPE_INDEX);
    }
    if uses_heap {
        functions.function(HEAP_ALLOC_TYPE_INDEX);
        functions.function(OBJECT_APPEND_DATA_PROPERTY_TYPE_INDEX);
        functions.function(OBJECT_APPEND_ACCESSOR_PROPERTY_TYPE_INDEX);
        functions.function(FUNCTION_OBJECT_ALLOC_TYPE_INDEX);
        functions.function(PLAIN_OBJECT_ALLOC_TYPE_INDEX);
        functions.function(ARRAY_ALLOC_TYPE_INDEX);
        // object-read + object-write runtime helpers share the JS function type.
        functions.function(JS_FUNCTION_TYPE_INDEX);
        functions.function(JS_FUNCTION_TYPE_INDEX);
        // object-define-data helper: seven i64 params, no results.
        functions.function(OBJECT_APPEND_ACCESSOR_PROPERTY_TYPE_INDEX);
        // proxy call + construct dispatch helpers share the JS function type.
        functions.function(JS_FUNCTION_TYPE_INDEX);
        functions.function(JS_FUNCTION_TYPE_INDEX);
        // string-payload-equality helper (also the JS function type).
        functions.function(JS_FUNCTION_TYPE_INDEX);
        // number-to-string + string-to-number conversion helpers.
        functions.function(JS_FUNCTION_TYPE_INDEX);
        functions.function(JS_FUNCTION_TYPE_INDEX);
        // dynamic ToString (value-to-string) helper.
        functions.function(JS_FUNCTION_TYPE_INDEX);
        // JSON.stringify value helper (only when JSON.stringify is compiled).
        if uses_json_stringify {
            functions.function(JS_FUNCTION_TYPE_INDEX);
        }
    }

    let mut exports = ExportSection::new();
    exports.export("main", ExportKind::Func, main_wasm_index);
    exports.export(
        RESULT_TAG_EXPORT,
        ExportKind::Global,
        RESULT_TAG_GLOBAL_INDEX,
    );
    exports.export(
        COMPLETION_KIND_EXPORT,
        ExportKind::Global,
        COMPLETION_KIND_GLOBAL_INDEX,
    );
    exports.export(
        COMPLETION_AUX_EXPORT,
        ExportKind::Global,
        COMPLETION_AUX_GLOBAL_INDEX,
    );
    exports.export(
        THROW_ERROR_NAME_EXPORT,
        ExportKind::Global,
        throw_error_name_global_index(uses_heap),
    );

    let mut globals = GlobalSection::new();
    globals.global(
        GlobalType {
            val_type: ValType::I32,
            mutable: true,
            shared: false,
        },
        &ConstExpr::i32_const(ValueKind::Undefined.tag()),
    );
    globals.global(
        GlobalType {
            val_type: ValType::I32,
            mutable: true,
            shared: false,
        },
        &ConstExpr::i32_const(COMPLETION_KIND_NORMAL as i32),
    );
    globals.global(
        GlobalType {
            val_type: ValType::I32,
            mutable: true,
            shared: false,
        },
        &ConstExpr::i32_const(0),
    );
    if uses_heap {
        globals.global(
            GlobalType {
                val_type: ValType::I64,
                mutable: true,
                shared: false,
            },
            &ConstExpr::i64_const(align_heap_start(string_pool.bytes.len()) as i64),
        );
        globals.global(
            GlobalType {
                val_type: ValType::I64,
                mutable: true,
                shared: false,
            },
            &ConstExpr::i64_const(0),
        );
        for _ in 0..59 {
            globals.global(
                GlobalType {
                    val_type: ValType::I64,
                    mutable: true,
                    shared: false,
                },
                &ConstExpr::i64_const(0),
            );
        }
    }
    globals.global(
        GlobalType {
            val_type: ValType::I64,
            mutable: true,
            shared: false,
        },
        &ConstExpr::i64_const(0),
    );
    if uses_heap {
        for _ in 0..13 {
            globals.global(
                GlobalType {
                    val_type: ValType::I64,
                    mutable: true,
                    shared: false,
                },
                &ConstExpr::i64_const(0),
            );
        }
    }

    let mut code = CodeSection::new();
    code.function(&main_function);
    for function in &compiled_functions {
        code.function(function);
    }
    if uses_heap {
        code.function(&emit_heap_alloc_helper_function());
        code.function(&emit_object_append_data_property_helper_function(
            heap_alloc_function_index.expect("heap helper index must exist when heap is enabled"),
        ));
        code.function(&emit_object_append_accessor_property_helper_function(
            heap_alloc_function_index.expect("heap helper index must exist when heap is enabled"),
        ));
        code.function(&emit_function_object_alloc_helper_function(
            heap_alloc_function_index.expect("heap helper index must exist when heap is enabled"),
            object_append_data_property_function_index
                .expect("object append helper index must exist when heap is enabled"),
        ));
        code.function(&emit_plain_object_alloc_helper_function(
            heap_alloc_function_index.expect("heap helper index must exist when heap is enabled"),
        ));
        code.function(&emit_array_alloc_helper_function(
            heap_alloc_function_index.expect("heap helper index must exist when heap is enabled"),
        ));
        code.function(
            object_read_helper_function
                .as_ref()
                .expect("object-read helper must exist when heap is enabled"),
        );
        code.function(
            object_write_helper_function
                .as_ref()
                .expect("object-write helper must exist when heap is enabled"),
        );
        code.function(
            object_define_data_helper_function
                .as_ref()
                .expect("object-define-data helper must exist when heap is enabled"),
        );
        code.function(
            proxy_call_helper_function
                .as_ref()
                .expect("proxy-call helper must exist when heap is enabled"),
        );
        code.function(
            proxy_construct_helper_function
                .as_ref()
                .expect("proxy-construct helper must exist when heap is enabled"),
        );
        code.function(
            string_equality_helper_function
                .as_ref()
                .expect("string-equality helper must exist when heap is enabled"),
        );
        code.function(
            number_to_string_helper_function
                .as_ref()
                .expect("number-to-string helper must exist when heap is enabled"),
        );
        code.function(
            string_to_number_helper_function
                .as_ref()
                .expect("string-to-number helper must exist when heap is enabled"),
        );
        code.function(
            value_to_string_helper_function
                .as_ref()
                .expect("value-to-string helper must exist when heap is enabled"),
        );
        if let Some(json_stringify_value_helper_function) =
            json_stringify_value_helper_function.as_ref()
        {
            code.function(json_stringify_value_helper_function);
        }
    }

    let mut module = Module::new();
    module.section(&types);
    if uses_host_print {
        let mut imports = ImportSection::new();
        imports.import(
            HOST_IMPORT_MODULE,
            HOST_IMPORT_PRINT_LINE_UTF8,
            wasm_encoder::EntityType::Function(HOST_PRINT_IMPORT_TYPE_INDEX),
        );
        module.section(&imports);
    }
    module.section(&functions);
    if uses_function_table {
        let mut tables = TableSection::new();
        tables.table(TableType {
            element_type: RefType::FUNCREF,
            minimum: callable_function_count as u64,
            maximum: Some(callable_function_count as u64),
            table64: false,
            shared: false,
        });
        module.section(&tables);
    }

    let mut debug_dump = vec![
        "module: js-aot".to_string(),
        "export func: main -> i64".to_string(),
        format!("static result kind: {}", script.result_kind().as_str()),
        format!("locals: {}", main_builder.local_count()),
        format!("internal functions: {}", callable_function_count),
        format!(
            "runtime helper functions: {}",
            if uses_heap { 15 } else { 0 }
        ),
        format!(
            "standard builtin bodies: {} real, {} shared-stubbed",
            emitted_standard_builtins.len(),
            stubbed_standard_builtins.len()
        ),
        format!(
            "runtime bootstrap: {} standard roots, full globals={}",
            runtime_bootstrap_plan.standard_roots.len(),
            runtime_bootstrap_plan.full_standard_globals
        ),
        format!(
            "standard builtin real names: {}",
            emitted_standard_builtins
                .iter()
                .map(|builtin| builtin.debug_name())
                .collect::<Vec<_>>()
                .join(", ")
        ),
        format!(
            "host builtin bodies: {} real, {} shared-stubbed",
            compiled_host_builtins.len(),
            stubbed_host_builtins.len()
        ),
        format!(
            "host builtin real names: {}",
            compiled_host_builtins
                .iter()
                .map(|builtin| builtin.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ),
        format!("global registry slots: {}", GLOBAL_INDEX_REGISTRY.len()),
        format!("completion kind slots: {}", COMPLETION_KIND_REGISTRY.len()),
        format!("export global: {RESULT_TAG_EXPORT}"),
        format!("export global: {COMPLETION_KIND_EXPORT}"),
        format!("export global: {COMPLETION_AUX_EXPORT}"),
        format!("export global: {THROW_ERROR_NAME_EXPORT}"),
    ];
    if uses_host_print {
        debug_dump.push(format!(
            "import func: {HOST_IMPORT_MODULE}.{HOST_IMPORT_PRINT_LINE_UTF8}"
        ));
    } else {
        debug_dump.push("imports: 0".to_string());
    }

    if !string_pool.bytes.is_empty() || uses_heap {
        let mut memories = MemorySection::new();
        let initial_pages = initial_memory_pages(string_pool.bytes.len(), uses_heap);
        memories.memory(MemoryType {
            minimum: initial_pages,
            maximum: uses_shared_memory.then_some(65_536),
            memory64: false,
            shared: uses_shared_memory,
            page_size_log2: None,
        });
        module.section(&memories);
        exports.export("memory", ExportKind::Memory, 0);
        if uses_shared_memory {
            debug_dump.push("memory: exported shared linear memory".to_string());
        } else {
            debug_dump.push("memory: exported linear memory".to_string());
        }

        let mut data = DataSection::new();
        data.active(
            0,
            &ConstExpr::i32_const(STATIC_DATA_OFFSET as i32),
            string_pool.bytes.iter().copied(),
        );
        module.section(&globals);
        module.section(&exports);
        if uses_function_table {
            let mut elements = ElementSection::new();
            let first_callable_wasm_index = imported_function_count + 1;
            let function_indexes = (first_callable_wasm_index
                ..first_callable_wasm_index + callable_function_count as u32)
                .collect::<Vec<_>>();
            elements.active(
                Some(0),
                &ConstExpr::i32_const(0),
                Elements::Functions(Cow::Owned(function_indexes)),
            );
            module.section(&elements);
        }
        module.section(&code);
        if !string_pool.bytes.is_empty() {
            module.section(&data);
            debug_dump.push("data segments: 1".to_string());
        } else {
            debug_dump.push("data segments: 0".to_string());
        }
        if uses_heap {
            debug_dump.push("heap: enabled".to_string());
        }
    } else {
        module.section(&globals);
        module.section(&exports);
        if uses_function_table {
            let mut elements = ElementSection::new();
            let first_callable_wasm_index = imported_function_count + 1;
            let function_indexes = (first_callable_wasm_index
                ..first_callable_wasm_index + callable_function_count as u32)
                .collect::<Vec<_>>();
            elements.active(
                Some(0),
                &ConstExpr::i32_const(0),
                Elements::Functions(Cow::Owned(function_indexes)),
            );
            module.section(&elements);
        }
        module.section(&code);
        debug_dump.push("memory: none".to_string());
        debug_dump.push("data segments: 0".to_string());
    }

    // Builtins codegen proved reachable (materialized or direct-called) while
    // their body was stubbed this pass: the caller force-compiles these and
    // re-emits (see `emit_script`).
    let touched_stubbed = ForcedBuiltins {
        standard: function_metas
            .touched_standard_builtins()
            .into_iter()
            .filter(|builtin| stubbed_standard_builtins.contains(builtin))
            .collect(),
        host: function_metas
            .touched_host_builtins()
            .into_iter()
            .filter(|builtin| stubbed_host_builtins.contains(builtin))
            .collect(),
    };

    Ok((
        WasmArtifact {
            bytes: module.finish(),
            invariant_note: "direct-js-to-wasm module",
            debug_dump: debug_dump.join("\n"),
        },
        touched_stubbed,
    ))
}

impl<'a> FunctionBuilder<'a> {
    fn new_main(
        script: &'a ScriptIr,
        strings: &'a StringPool,
        functions: &'a FunctionMetaRegistry,
        uses_heap: bool,
        runtime_bootstrap_plan: RuntimeBootstrapPlan,
        heap_alloc_function_index: Option<u32>,
        object_append_data_property_function_index: Option<u32>,
        object_append_accessor_property_function_index: Option<u32>,
        function_object_alloc_function_index: Option<u32>,
        plain_object_alloc_function_index: Option<u32>,
        array_alloc_function_index: Option<u32>,
    ) -> Self {
        Self::new(
            &script.body,
            &[],
            script.owned_env_bindings.as_slice(),
            &[],
            strings,
            functions,
            None,
            FunctionFlavor::Ordinary,
            script.strict,
            None,
            script
                .global_bindings
                .iter()
                .map(|binding| (binding.name.clone(), binding.kind))
                .collect(),
            uses_heap,
            ReturnAbi::MainExport,
            false,
            runtime_bootstrap_plan,
            heap_alloc_function_index,
            object_append_data_property_function_index,
            object_append_accessor_property_function_index,
            function_object_alloc_function_index,
            plain_object_alloc_function_index,
            array_alloc_function_index,
        )
    }

    fn new_function(
        function: &'a FunctionIr,
        global_bindings: &'a [ScriptGlobalBindingIr],
        strings: &'a StringPool,
        functions: &'a FunctionMetaRegistry,
        uses_heap: bool,
        runtime_bootstrap_plan: RuntimeBootstrapPlan,
        heap_alloc_function_index: Option<u32>,
        object_append_data_property_function_index: Option<u32>,
        object_append_accessor_property_function_index: Option<u32>,
        function_object_alloc_function_index: Option<u32>,
        plain_object_alloc_function_index: Option<u32>,
        array_alloc_function_index: Option<u32>,
    ) -> Self {
        Self::new(
            &function.body,
            function.params.as_slice(),
            function.owned_env_bindings.as_slice(),
            function.captured_bindings.as_slice(),
            strings,
            functions,
            Some(function.id.clone()),
            function.flavor,
            function.strict,
            (!function.is_expression || function.is_named_expression)
                .then(|| function.name.clone()),
            global_bindings
                .iter()
                .map(|binding| (binding.name.clone(), binding.kind))
                .collect(),
            uses_heap,
            ReturnAbi::MultiValue,
            function.is_derived_constructor,
            runtime_bootstrap_plan,
            heap_alloc_function_index,
            object_append_data_property_function_index,
            object_append_accessor_property_function_index,
            function_object_alloc_function_index,
            plain_object_alloc_function_index,
            array_alloc_function_index,
        )
    }

    fn new_host_builtin(
        builtin: HostBuiltinId,
        strings: &'a StringPool,
        functions: &'a FunctionMetaRegistry,
        uses_heap: bool,
        heap_alloc_function_index: Option<u32>,
        object_append_data_property_function_index: Option<u32>,
        object_append_accessor_property_function_index: Option<u32>,
        function_object_alloc_function_index: Option<u32>,
        plain_object_alloc_function_index: Option<u32>,
        array_alloc_function_index: Option<u32>,
    ) -> Self {
        let function_id = builtin.function_id();
        Self::new(
            &EMPTY_BLOCK,
            &[],
            &[],
            &[],
            strings,
            functions,
            Some(function_id),
            FunctionFlavor::Ordinary,
            true,
            None,
            BTreeMap::new(),
            uses_heap,
            ReturnAbi::MultiValue,
            false,
            RuntimeBootstrapPlan::default(),
            heap_alloc_function_index,
            object_append_data_property_function_index,
            object_append_accessor_property_function_index,
            function_object_alloc_function_index,
            plain_object_alloc_function_index,
            array_alloc_function_index,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn new_runtime_operation_helper(
        strings: &'a StringPool,
        functions: &'a FunctionMetaRegistry,
        uses_heap: bool,
        runtime_bootstrap_plan: RuntimeBootstrapPlan,
        heap_alloc_function_index: Option<u32>,
        object_append_data_property_function_index: Option<u32>,
        object_append_accessor_property_function_index: Option<u32>,
        function_object_alloc_function_index: Option<u32>,
        plain_object_alloc_function_index: Option<u32>,
        array_alloc_function_index: Option<u32>,
    ) -> Self {
        Self::new(
            &EMPTY_BLOCK,
            &[],
            &[],
            &[],
            strings,
            functions,
            None,
            FunctionFlavor::Ordinary,
            true,
            None,
            BTreeMap::new(),
            uses_heap,
            ReturnAbi::MultiValue,
            false,
            runtime_bootstrap_plan,
            heap_alloc_function_index,
            object_append_data_property_function_index,
            object_append_accessor_property_function_index,
            function_object_alloc_function_index,
            plain_object_alloc_function_index,
            array_alloc_function_index,
        )
    }

    fn new_standard_builtin(
        builtin: StandardBuiltinId,
        strings: &'a StringPool,
        functions: &'a FunctionMetaRegistry,
        uses_heap: bool,
        stub_body: bool,
        runtime_bootstrap_plan: RuntimeBootstrapPlan,
        heap_alloc_function_index: Option<u32>,
        object_append_data_property_function_index: Option<u32>,
        object_append_accessor_property_function_index: Option<u32>,
        function_object_alloc_function_index: Option<u32>,
        plain_object_alloc_function_index: Option<u32>,
        array_alloc_function_index: Option<u32>,
    ) -> Self {
        let mut builder = Self::new(
            &EMPTY_BLOCK,
            &[],
            &[],
            &[],
            strings,
            functions,
            Some(builtin.function_id()),
            FunctionFlavor::Ordinary,
            true,
            None,
            BTreeMap::new(),
            uses_heap,
            ReturnAbi::MultiValue,
            false,
            runtime_bootstrap_plan,
            heap_alloc_function_index,
            object_append_data_property_function_index,
            object_append_accessor_property_function_index,
            function_object_alloc_function_index,
            plain_object_alloc_function_index,
            array_alloc_function_index,
        );
        builder.stub_standard_builtin_body = stub_body;
        builder
    }

    fn new(
        body: &'a BlockIr,
        params: &'a [FunctionParamIr],
        owned_env_bindings: &'a [OwnedEnvBindingIr],
        captured_bindings: &'a [porffor_ir::CapturedBindingIr],
        strings: &'a StringPool,
        functions: &'a FunctionMetaRegistry,
        function_id: Option<FunctionId>,
        function_flavor: FunctionFlavor,
        strict: bool,
        self_binding_name: Option<String>,
        script_global_bindings: BTreeMap<String, ScriptGlobalBindingKind>,
        uses_heap: bool,
        return_abi: ReturnAbi,
        is_derived_constructor: bool,
        runtime_bootstrap_plan: RuntimeBootstrapPlan,
        heap_alloc_function_index: Option<u32>,
        object_append_data_property_function_index: Option<u32>,
        object_append_accessor_property_function_index: Option<u32>,
        function_object_alloc_function_index: Option<u32>,
        plain_object_alloc_function_index: Option<u32>,
        array_alloc_function_index: Option<u32>,
    ) -> Self {
        let hoisted_vars = collect_hoisted_vars_block_root(body);
        let self_binding_local_count = usize::from(self_binding_name.is_some());
        let param_local_count = count_param_locals(return_abi) as u32;
        let needs_arguments_binding_locals = matches!(return_abi, ReturnAbi::MultiValue)
            && function_flavor == FunctionFlavor::Ordinary;
        let captured_arguments_local_count = if captured_bindings
            .iter()
            .any(|binding| binding.name == LEXICAL_ARGUMENTS_NAME)
        {
            2
        } else {
            0
        };
        let total_binding_local_count = (count_block_lexicals(body)
            + self_binding_local_count
            + count_param_binding_locals(params, owned_env_bindings)
            + if needs_arguments_binding_locals { 2 } else { 0 }
            + captured_arguments_local_count) as u32
            + (hoisted_vars.len() as u32 * 2);
        let temp_local_count = count_block_temp_locals(body).max(2048) as u32;
        let current_env_local = param_local_count + total_binding_local_count;
        let derived_this_initialized_local =
            is_derived_constructor.then_some(current_env_local + 5);
        let scratch_local = current_env_local + 5 + u32::from(is_derived_constructor);
        Self {
            body,
            params,
            owned_env_bindings,
            captured_bindings,
            strings,
            functions,
            function_id,
            function_flavor,
            strict,
            self_binding_name,
            script_global_bindings,
            uses_heap,
            return_abi,
            hoisted_vars,
            binding_scopes: Vec::new(),
            next_binding_local: param_local_count,
            total_binding_local_count,
            temp_local_count,
            current_env_local,
            result_local: current_env_local + 1,
            result_tag_local: current_env_local + 2,
            completion_local: current_env_local + 3,
            completion_aux_local: current_env_local + 4,
            derived_this_initialized_local,
            scratch_local,
            temp_local_base: scratch_local + 1,
            temp_stack_depth: 0,
            this_payload_local: matches!(return_abi, ReturnAbi::MultiValue).then_some(1),
            this_tag_local: matches!(return_abi, ReturnAbi::MultiValue).then_some(2),
            control_stack: Vec::new(),
            breakable_stack: Vec::new(),
            loop_stack: Vec::new(),
            label_stack: Vec::new(),
            throw_handler_stack: Vec::new(),
            finally_stack: Vec::new(),
            stub_standard_builtin_body: false,
            runtime_bootstrap_plan,
            heap_alloc_function_index,
            object_append_data_property_function_index,
            object_append_accessor_property_function_index,
            function_object_alloc_function_index,
            plain_object_alloc_function_index,
            array_alloc_function_index,
            outline_object_read: true,
            outline_object_write: true,
            outline_object_define_data: true,
            outline_proxy_call: true,
            outline_proxy_construct: true,
            outline_string_equality: true,
            outline_number_to_string: true,
            outline_string_to_number: true,
            outline_value_to_string: true,
        }
    }

    /// Wasm function index of the shared object-read runtime helper. It is
    /// emitted immediately after the six heap/object allocation helpers.
    pub(crate) fn object_read_helper_function_index(&self) -> Option<u32> {
        self.heap_alloc_function_index.map(|base| base + 6)
    }

    /// Wasm function index of the shared object-write runtime helper.
    pub(crate) fn object_write_helper_function_index(&self) -> Option<u32> {
        self.heap_alloc_function_index.map(|base| base + 7)
    }

    /// Wasm function index of the shared object-define-data runtime helper.
    pub(crate) fn object_define_data_helper_function_index(&self) -> Option<u32> {
        self.heap_alloc_function_index.map(|base| base + 8)
    }

    /// Wasm function index of the shared proxy-aware call-dispatch helper.
    pub(crate) fn proxy_call_helper_function_index(&self) -> Option<u32> {
        self.heap_alloc_function_index.map(|base| base + 9)
    }

    /// Wasm function index of the shared proxy-aware construct-dispatch helper.
    pub(crate) fn proxy_construct_helper_function_index(&self) -> Option<u32> {
        self.heap_alloc_function_index.map(|base| base + 10)
    }

    /// Wasm function index of the shared string-equality helper.
    pub(crate) fn string_equality_helper_function_index(&self) -> Option<u32> {
        self.heap_alloc_function_index.map(|base| base + 11)
    }

    /// Wasm function index of the shared number-to-string helper.
    pub(crate) fn number_to_string_helper_function_index(&self) -> Option<u32> {
        self.heap_alloc_function_index.map(|base| base + 12)
    }

    /// Wasm function index of the shared string-to-number helper.
    pub(crate) fn string_to_number_helper_function_index(&self) -> Option<u32> {
        self.heap_alloc_function_index.map(|base| base + 13)
    }

    /// Wasm function index of the shared dynamic ToString (value-to-string)
    /// helper.
    pub(crate) fn value_to_string_helper_function_index(&self) -> Option<u32> {
        self.heap_alloc_function_index.map(|base| base + 14)
    }

    /// Wasm function index of the shared JSON.stringify value helper. Emitted
    /// only when `JSON.stringify` is compiled, immediately after the
    /// value-to-string helper (the last unconditional runtime helper), so its
    /// index never shifts the preceding fixed-offset helpers.
    pub(crate) fn json_stringify_value_helper_function_index(&self) -> Option<u32> {
        self.heap_alloc_function_index.map(|base| base + 15)
    }

    pub(crate) fn local_count(&self) -> usize {
        self.total_binding_local_count as usize
            + 6
            + usize::from(self.derived_this_initialized_local.is_some())
            + self.temp_local_count as usize
    }

    pub(crate) const fn is_main(&self) -> bool {
        matches!(self.return_abi, ReturnAbi::MainExport)
    }

    pub(crate) fn is_script_global_binding(&self, name: &str) -> bool {
        self.script_global_bindings
            .get(name)
            .is_some_and(|kind| *kind != ScriptGlobalBindingKind::Intrinsic)
    }

    pub(crate) fn should_read_script_global_property(&self, name: &str) -> bool {
        !self.is_main()
            && name != LEXICAL_THIS_NAME
            && name != LEXICAL_ARGUMENTS_NAME
            && self.lookup_binding(name).is_none()
    }

    pub(crate) fn reserve_temp_local(&mut self) -> u32 {
        assert!(self.temp_stack_depth < self.temp_local_count);
        let local = self.temp_local_base + self.temp_stack_depth;
        self.temp_stack_depth += 1;
        local
    }

    pub(crate) fn release_temp_local(&mut self, local: u32) {
        assert!(self.temp_stack_depth > 0);
        self.temp_stack_depth -= 1;
        let expected = self.temp_local_base + self.temp_stack_depth;
        assert_eq!(local, expected);
    }

    fn compile(&mut self) -> Result<Function, EmitError> {
        let mut function =
            Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, self.local_count()));

        self.push_scope();
        self.ensure_heap_ptr_after_static_data(&mut function);
        self.init_current_realm(&mut function)?;
        self.init_current_env(&mut function)?;
        self.init_runtime_roots(&mut function)?;
        self.init_script_global_object(&mut function)?;
        self.bind_captured_bindings(&mut function);
        self.bind_self_function(&mut function)?;
        self.bind_parameters(&mut function)?;
        self.set_completion_kind(CompletionKind::Normal, &mut function);
        if let Some(derived_this_initialized_local) = self.derived_this_initialized_local {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(derived_this_initialized_local));
        }
        self.emit_statement_result(&mut function, ValueKind::Undefined);
        for name in self.hoisted_vars.clone() {
            let storage = if let Some(slot) = self.owned_env_slot(&name) {
                BindingStorage::EnvSlot { slot, hops: 0 }
            } else {
                let tag_local = self.next_binding_local;
                let payload_local = self.next_binding_local + 1;
                self.next_binding_local += 2;
                BindingStorage::Dynamic {
                    tag_local,
                    payload_local,
                }
            };
            self.binding_scopes
                .last_mut()
                .expect("binding scope stack must exist")
                .insert(name, storage);
            self.initialize_binding_undefined(storage, &mut function);
        }
        if self
            .current_function_meta()
            .is_some_and(|meta| meta.is_synthetic_default_derived_constructor)
        {
            self.emit_super_construct_with_arg_vector(
                self.argc_param_local(),
                self.argv_param_local(),
                self.result_local,
                self.result_tag_local,
                &mut function,
            )?;
            self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
                self.result_local,
                self.result_tag_local,
                0,
                &mut function,
            )?;
        }
        self.compile_block_contents(self.body, &mut function)?;
        if matches!(self.return_abi, ReturnAbi::MultiValue)
            && !self
                .current_function_meta()
                .is_some_and(|meta| meta.class_kind == ClassFunctionKind::Constructor)
        {
            self.emit_statement_result(&mut function, ValueKind::Undefined);
        }
        self.normalize_derived_constructor_result(&mut function)?;
        self.pop_scope();

        match self.return_abi {
            ReturnAbi::MainExport => {
                function.instruction(&Instruction::LocalGet(self.result_tag_local));
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::GlobalSet(RESULT_TAG_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalGet(self.completion_local));
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::GlobalSet(COMPLETION_KIND_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalGet(self.completion_aux_local));
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::GlobalSet(COMPLETION_AUX_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalGet(self.result_local));
            }
            ReturnAbi::MultiValue => {
                function.instruction(&Instruction::LocalGet(self.result_local));
                function.instruction(&Instruction::LocalGet(self.result_tag_local));
                function.instruction(&Instruction::LocalGet(self.completion_local));
                function.instruction(&Instruction::LocalGet(self.completion_aux_local));
            }
        }
        function.instruction(&Instruction::End);
        Ok(function)
    }

    fn ensure_heap_ptr_after_static_data(&self, function: &mut Function) {
        if !self.is_main() || !self.uses_heap {
            return;
        }
        let heap_start = align_heap_start(self.strings.bytes.len()) as i64;
        function.instruction(&Instruction::I64Const(heap_start));
        function.instruction(&Instruction::GlobalSet(HEAP_PTR_GLOBAL_INDEX));
    }

    fn init_current_realm(&mut self, function: &mut Function) -> Result<(), EmitError> {
        if !self.is_main() || !self.uses_heap {
            return Ok(());
        }
        let realm_local = self.reserve_temp_local();
        self.emit_alloc_realm_record(1, 1, realm_local, function)?;
        function.instruction(&Instruction::LocalGet(realm_local));
        function.instruction(&Instruction::GlobalSet(CURRENT_REALM_GLOBAL_INDEX));
        self.release_temp_local(realm_local);
        Ok(())
    }

    fn compile_builtin(&mut self) -> Result<Function, EmitError> {
        let Some(function_id) = self.function_id.clone() else {
            return Err(EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing builtin id",
            ));
        };
        let mut function =
            Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, self.local_count()));
        self.push_scope();
        self.init_current_env(&mut function)?;
        self.set_completion_kind(CompletionKind::Normal, &mut function);
        self.emit_statement_result(&mut function, ValueKind::Undefined);
        if let Some(builtin) = StandardBuiltinId::from_function_id(&function_id) {
            if self.stub_standard_builtin_body {
                self.emit_throw_runtime_error(
                    TYPE_ERROR_NAME,
                    &format!(
                        "standard builtin body is not emitted unless referenced directly: {}",
                        builtin.debug_name()
                    ),
                    self.result_local,
                    self.result_tag_local,
                    &mut function,
                )?;
                self.emit_return_current_completion(&mut function);
            } else {
                self.compile_standard_builtin(builtin, &mut function)?;
            }
        } else {
            match HostBuiltinId::from_function_id(&function_id) {
                Some(HostBuiltinId::Print) => self.compile_host_print_builtin(&mut function)?,
                Some(HostBuiltinId::Gc) => self.compile_host_gc_builtin(&mut function)?,
                Some(HostBuiltinId::AssertThrows) => {
                    self.compile_host_assert_throws_builtin(&mut function)?
                }
                Some(HostBuiltinId::IsConstructor) => {
                    self.compile_host_is_constructor_builtin(&mut function)?
                }
                Some(HostBuiltinId::CreateRealm) => {
                    self.compile_host_create_realm_builtin(&mut function)?
                }
                Some(HostBuiltinId::ParseInt) => {
                    self.compile_host_parse_int_builtin(&mut function)?
                }
                Some(HostBuiltinId::ParseFloat) => {
                    self.compile_host_parse_float_builtin(&mut function)?
                }
                Some(HostBuiltinId::DetachArrayBuffer) => {
                    self.compile_host_detach_array_buffer_builtin(&mut function)?
                }
                None => {
                    return Err(EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: unknown builtin `{function_id}`"
                    )));
                }
            }
        }
        self.pop_scope();
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::LocalGet(self.completion_aux_local));
        function.instruction(&Instruction::End);
        Ok(function)
    }

    /// Compiles the shared object-read runtime helper. Rather than inlining the
    /// large ordinary/proxy property-read state machine at every read site, that
    /// sequence is emitted exactly once here and reached with a plain `call`.
    ///
    /// Wasm signature is [`JS_FUNCTION_TYPE_INDEX`] (seven i64 params, four i64
    /// results). Params: 0=object payload, 1=object tag, 2=receiver payload,
    /// 3=receiver tag, 4=key payload. Params 5/6 are unused. Results are the
    /// standard `(result, result_tag, completion, completion_aux)` tuple.
    fn compile_object_read_helper(&mut self) -> Result<Function, EmitError> {
        let mut function =
            Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, self.local_count()));
        self.outline_object_read = false;
        self.push_scope();
        self.set_completion_kind(CompletionKind::Normal, &mut function);
        self.emit_statement_result(&mut function, ValueKind::Undefined);
        self.emit_object_read_ordinary_inner(
            0,
            1,
            2,
            3,
            4,
            self.result_local,
            self.result_tag_local,
            None,
            &mut function,
        )?;
        self.pop_scope();
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::LocalGet(self.completion_aux_local));
        function.instruction(&Instruction::End);
        Ok(function)
    }

    /// Compiles the shared object-write runtime helper. The large ordinary/proxy
    /// property-write state machine is emitted once here and reached with a
    /// plain `call`.
    ///
    /// Wasm signature is [`JS_FUNCTION_TYPE_INDEX`]. Params: 0=object payload,
    /// 1=object tag, 2=key payload, 3=value payload, 4=value tag. Params 5/6 are
    /// unused. On a setter/proxy throw the thrown value is surfaced through the
    /// `(result, result_tag, completion, completion_aux)` result tuple.
    fn compile_object_write_helper(&mut self) -> Result<Function, EmitError> {
        let mut function =
            Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, self.local_count()));
        self.outline_object_write = false;
        self.push_scope();
        self.set_completion_kind(CompletionKind::Normal, &mut function);
        self.emit_statement_result(&mut function, ValueKind::Undefined);
        self.emit_object_write(0, 1, 2, 3, 4, &mut function)?;
        self.pop_scope();
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::LocalGet(self.completion_aux_local));
        function.instruction(&Instruction::End);
        Ok(function)
    }

    /// Compiles the shared object-define-data runtime helper. Bootstrap-style
    /// code (realm setup, `$262.createRealm`) defines hundreds of data
    /// properties; emitting the define state machine once here keeps those
    /// functions under Cranelift's per-function limit.
    ///
    /// Wasm signature is [`OBJECT_APPEND_ACCESSOR_PROPERTY_TYPE_INDEX`] (seven
    /// i64 params, no results). Params: 0=object payload, 1=key payload,
    /// 2=value payload, 3=value tag, 4=writable, 5=enumerable, 6=configurable.
    fn compile_object_define_data_helper(&mut self) -> Result<Function, EmitError> {
        let mut function =
            Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, self.local_count()));
        self.outline_object_define_data = false;
        self.push_scope();
        self.set_completion_kind(CompletionKind::Normal, &mut function);
        self.emit_object_define_data_with_flag_locals(0, 1, 2, 3, 4, 5, 6, &mut function)?;
        self.pop_scope();
        function.instruction(&Instruction::End);
        Ok(function)
    }

    /// Compiles the shared proxy-aware call-dispatch helper. The proxy call
    /// state machine (walk the proxy chain, invoke the `apply` trap, otherwise
    /// `call_indirect`) is emitted once here and reached with a plain `call`.
    ///
    /// Wasm signature is [`JS_FUNCTION_TYPE_INDEX`]. Params: 0=callee payload,
    /// 1=callee tag, 2=this payload, 3=this tag, 4=argc, 5=argv. Params 6 is
    /// unused. Results are the `(result, result_tag, completion, aux)` tuple;
    /// throws are surfaced through the completion rather than propagated.
    fn compile_proxy_call_helper(&mut self) -> Result<Function, EmitError> {
        let mut function =
            Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, self.local_count()));
        self.outline_proxy_call = false;
        self.push_scope();
        self.set_completion_kind(CompletionKind::Normal, &mut function);
        self.emit_statement_result(&mut function, ValueKind::Undefined);
        self.emit_function_or_proxy_call_with_argv_inner(
            0,
            1,
            2,
            3,
            4,
            5,
            self.result_local,
            self.result_tag_local,
            false,
            &mut function,
        )?;
        self.pop_scope();
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::LocalGet(self.completion_aux_local));
        function.instruction(&Instruction::End);
        Ok(function)
    }

    /// Compiles the shared proxy-aware construct-dispatch helper.
    ///
    /// Wasm signature is [`JS_FUNCTION_TYPE_INDEX`]. Params: 0=callee payload,
    /// 1=callee tag, 2=new.target payload, 3=new.target tag, 4=argc, 5=argv.
    /// Param 6 is unused. Results are the `(result, result_tag, completion,
    /// aux)` tuple.
    fn compile_proxy_construct_helper(&mut self) -> Result<Function, EmitError> {
        let mut function =
            Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, self.local_count()));
        self.outline_proxy_construct = false;
        self.push_scope();
        self.set_completion_kind(CompletionKind::Normal, &mut function);
        self.emit_statement_result(&mut function, ValueKind::Undefined);
        self.emit_function_or_proxy_construct_with_argv(
            0,
            1,
            2,
            3,
            4,
            5,
            self.result_local,
            self.result_tag_local,
            &mut function,
        )?;
        self.pop_scope();
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::LocalGet(self.completion_aux_local));
        function.instruction(&Instruction::End);
        Ok(function)
    }

    /// Compiles the shared string-payload-equality helper. Builtin bodies
    /// compare interned string payloads at thousands of sites (property-name
    /// matching, key switches); the ~65-instruction byte-compare loop is emitted
    /// once here and reached with a plain `call`, keeping the largest builtin
    /// bodies under Cranelift's per-function virtual-register limit.
    ///
    /// Wasm signature is [`JS_FUNCTION_TYPE_INDEX`]. Params: 0=lhs string
    /// payload, 1=rhs string payload. Params 2-6 are unused. Results are the
    /// standard four-i64 tuple with the comparison result (0 or 1) in the first
    /// slot; the other three are always zero.
    fn compile_string_equality_helper(&mut self) -> Result<Function, EmitError> {
        let mut function =
            Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, self.local_count()));
        self.outline_string_equality = false;
        self.push_scope();
        self.emit_string_payload_equality_i32(0, 1, &mut function);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(self.result_local));
        self.pop_scope();
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::End);
        Ok(function)
    }

    /// Compiles the shared number-to-string helper (the ECMAScript Number→
    /// String digit-emission state machine, several KB per inline copy).
    ///
    /// Wasm signature is [`JS_FUNCTION_TYPE_INDEX`]. Params: 0=number payload
    /// (f64 bits). Params 1-6 are unused. Results are the standard four-i64
    /// tuple with the string payload in the first slot; the rest are zero.
    fn compile_number_to_string_helper(&mut self) -> Result<Function, EmitError> {
        let mut function =
            Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, self.local_count()));
        self.outline_number_to_string = false;
        self.push_scope();
        self.emit_number_to_string_payload(0, &mut function)?;
        function.instruction(&Instruction::LocalSet(self.result_local));
        self.pop_scope();
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::End);
        Ok(function)
    }

    /// Compiles the shared string-to-number helper (the ECMAScript String→
    /// Number parse state machine, several KB per inline copy).
    ///
    /// Wasm signature is [`JS_FUNCTION_TYPE_INDEX`]. Params: 0=string payload.
    /// Params 1-6 are unused. Results are the standard four-i64 tuple with the
    /// number payload (f64 bits) in the first slot; the rest are zero.
    fn compile_string_to_number_helper(&mut self) -> Result<Function, EmitError> {
        let mut function =
            Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, self.local_count()));
        self.outline_string_to_number = false;
        self.push_scope();
        self.emit_string_to_number_payload(0, &mut function)?;
        function.instruction(&Instruction::LocalSet(self.result_local));
        self.pop_scope();
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::End);
        Ok(function)
    }

    /// Compiles the shared dynamic ToString helper (per-kind dispatch,
    /// ToPrimitive on objects, array join, function source text — tens of KB
    /// per inline copy, and dynamic string concatenation hits it constantly).
    ///
    /// Wasm signature is [`JS_FUNCTION_TYPE_INDEX`]. Params: 0=value payload,
    /// 1=value tag. Params 2-6 are unused. Results are the standard four-i64
    /// tuple: on normal completion the string payload is in the first slot; a
    /// ToPrimitive/Symbol throw is surfaced through the completion slots.
    fn compile_value_to_string_helper(&mut self) -> Result<Function, EmitError> {
        let mut function =
            Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, self.local_count()));
        self.outline_value_to_string = false;
        self.push_scope();
        self.set_completion_kind(CompletionKind::Normal, &mut function);
        self.emit_statement_result(&mut function, ValueKind::Undefined);
        self.emit_value_to_string_payload(0, 1, &mut function)?;
        function.instruction(&Instruction::LocalSet(self.result_local));
        self.pop_scope();
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::LocalGet(self.completion_aux_local));
        function.instruction(&Instruction::End);
        Ok(function)
    }

    fn init_current_env(&mut self, function: &mut Function) -> Result<(), EmitError> {
        match self.return_abi {
            ReturnAbi::MainExport => {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(self.current_env_local));
            }
            ReturnAbi::MultiValue => {
                function.instruction(&Instruction::LocalGet(0));
                function.instruction(&Instruction::LocalSet(self.current_env_local));
            }
        }

        if self.owned_env_bindings.is_empty() {
            return Ok(());
        }

        let parent_env_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::LocalSet(parent_env_local));
        self.emit_heap_alloc_const(
            ENV_SLOT_BASE_OFFSET + self.owned_env_bindings.len() as u64 * ENV_SLOT_SIZE,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(self.current_env_local));
        self.store_i64_local_at_offset(
            self.current_env_local,
            ENV_PARENT_OFFSET,
            parent_env_local,
            function,
        );
        for binding in self.owned_env_bindings {
            self.store_i64_const_at_offset(
                self.current_env_local,
                ENV_SLOT_BASE_OFFSET + binding.slot as u64 * ENV_SLOT_SIZE + ENV_SLOT_TAG_OFFSET,
                ValueKind::Undefined.tag() as u64,
                function,
            );
            self.store_i64_const_at_offset(
                self.current_env_local,
                ENV_SLOT_BASE_OFFSET
                    + binding.slot as u64 * ENV_SLOT_SIZE
                    + ENV_SLOT_PAYLOAD_OFFSET,
                0,
                function,
            );
        }
        if self.function_flavor == FunctionFlavor::Ordinary {
            if let Some(slot) = self.owned_env_slot(LEXICAL_THIS_NAME) {
                if self.is_main() {
                    self.release_temp_local(parent_env_local);
                    return Ok(());
                }
                let Some(this_payload_local) = self.this_payload_local else {
                    return Err(EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: top-level `this`",
                    ));
                };
                let Some(this_tag_local) = self.this_tag_local else {
                    return Err(EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: missing `this` tag local",
                    ));
                };
                self.write_binding_from_locals(
                    BindingStorage::EnvSlot { slot, hops: 0 },
                    this_payload_local,
                    this_tag_local,
                    function,
                );
            }
            if let Some(slot) = self.owned_env_slot(LEXICAL_NEW_TARGET_NAME) {
                let Some(new_target_payload_local) = self.new_target_payload_local() else {
                    return Err(EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: missing `new.target` payload local",
                    ));
                };
                let Some(new_target_tag_local) = self.new_target_tag_local() else {
                    return Err(EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: missing `new.target` tag local",
                    ));
                };
                self.write_binding_from_locals(
                    BindingStorage::EnvSlot { slot, hops: 0 },
                    new_target_payload_local,
                    new_target_tag_local,
                    function,
                );
            }
        }
        self.release_temp_local(parent_env_local);
        Ok(())
    }

    pub(crate) const fn memarg32(offset: u64) -> MemArg {
        MemArg {
            offset,
            align: 2,
            memory_index: 0,
        }
    }

    pub(crate) const fn memarg16(offset: u64) -> MemArg {
        MemArg {
            offset,
            align: 1,
            memory_index: 0,
        }
    }

    pub(crate) const fn memarg8(offset: u64) -> MemArg {
        MemArg {
            offset,
            align: 0,
            memory_index: 0,
        }
    }
}
