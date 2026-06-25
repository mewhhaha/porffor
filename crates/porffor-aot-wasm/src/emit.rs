use std::borrow::Cow;

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
    pub(crate) functions: &'a BTreeMap<FunctionId, WasmFunctionMeta>,
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
    let uses_heap = true;
    let host_builtins = all_host_builtins();
    let uses_host_print = host_builtins.contains(&HostBuiltinId::Print);
    let imported_function_count = u32::from(uses_host_print);
    let standard_builtins = StandardBuiltinId::all_functions();
    let function_metas = build_function_metas(
        script.functions.as_slice(),
        standard_builtins,
        host_builtins,
        imported_function_count,
    );
    let string_pool = StringPool::collect(script, &function_metas);
    let uses_function_table = true;
    let mut main_builder =
        FunctionBuilder::new_main(script, &string_pool, &function_metas, uses_heap);
    let main_function = main_builder.compile()?;
    let mut compiled_functions =
        Vec::with_capacity(script.functions.len() + standard_builtins.len() + host_builtins.len());
    for function in &script.functions {
        let mut builder = FunctionBuilder::new_function(
            function,
            &script.global_bindings,
            &string_pool,
            &function_metas,
            uses_heap,
        );
        compiled_functions.push(builder.compile()?);
    }
    for builtin in standard_builtins {
        let mut builder = FunctionBuilder::new_standard_builtin(
            *builtin,
            &string_pool,
            &function_metas,
            uses_heap,
            should_stub_standard_builtin(script, *builtin),
        );
        compiled_functions.push(builder.compile_builtin()?);
    }
    for builtin in host_builtins {
        let mut builder =
            FunctionBuilder::new_host_builtin(*builtin, &string_pool, &function_metas, uses_heap);
        compiled_functions.push(builder.compile_builtin()?);
    }

    let mut types = TypeSection::new();
    types.ty().function([], [ValType::I64]);
    if uses_function_table {
        types.ty().function(
            function_param_types(),
            [ValType::I64, ValType::I64, ValType::I64, ValType::I64],
        );
    }
    if uses_host_print {
        types.ty().function([ValType::I32, ValType::I32], []);
    }

    let callable_function_count =
        script.functions.len() + standard_builtins.len() + host_builtins.len();
    let main_wasm_index = imported_function_count;

    let mut functions = FunctionSection::new();
    functions.function(0);
    for _ in 0..callable_function_count {
        functions.function(JS_FUNCTION_TYPE_INDEX);
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
        for _ in 0..8 {
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
        memories.memory(MemoryType {
            minimum: initial_memory_pages(string_pool.bytes.len(), uses_heap),
            maximum: None,
            memory64: false,
            shared: false,
            page_size_log2: None,
        });
        module.section(&memories);
        exports.export("memory", ExportKind::Memory, 0);
        debug_dump.push("memory: exported linear memory".to_string());

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

    Ok(WasmArtifact {
        bytes: module.finish(),
        invariant_note: "direct-js-to-wasm module",
        debug_dump: debug_dump.join("\n"),
    })
}

impl<'a> FunctionBuilder<'a> {
    fn new_main(
        script: &'a ScriptIr,
        strings: &'a StringPool,
        functions: &'a BTreeMap<FunctionId, WasmFunctionMeta>,
        uses_heap: bool,
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
        )
    }

    fn new_function(
        function: &'a FunctionIr,
        global_bindings: &'a [ScriptGlobalBindingIr],
        strings: &'a StringPool,
        functions: &'a BTreeMap<FunctionId, WasmFunctionMeta>,
        uses_heap: bool,
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
        )
    }

    fn new_host_builtin(
        builtin: HostBuiltinId,
        strings: &'a StringPool,
        functions: &'a BTreeMap<FunctionId, WasmFunctionMeta>,
        uses_heap: bool,
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
        )
    }

    fn new_standard_builtin(
        builtin: StandardBuiltinId,
        strings: &'a StringPool,
        functions: &'a BTreeMap<FunctionId, WasmFunctionMeta>,
        uses_heap: bool,
        stub_body: bool,
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
        functions: &'a BTreeMap<FunctionId, WasmFunctionMeta>,
        function_id: Option<FunctionId>,
        function_flavor: FunctionFlavor,
        strict: bool,
        self_binding_name: Option<String>,
        script_global_bindings: BTreeMap<String, ScriptGlobalBindingKind>,
        uses_heap: bool,
        return_abi: ReturnAbi,
        is_derived_constructor: bool,
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
        }
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
                    "standard builtin body is not emitted unless referenced directly",
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
                Some(HostBuiltinId::Gc) => self.compile_host_gc_builtin(&mut function),
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
