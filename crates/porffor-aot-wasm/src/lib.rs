use std::{borrow::Cow, collections::{BTreeMap, BTreeSet}};

use porffor_ir::{
    ArithmeticBinaryOp, BindingMode, BlockIr, EqualityBinaryOp, ExprIr, ForInitIr, FunctionFlavor,
    FunctionIr, FunctionId, FunctionParamIr, LogicalBinaryOp, NumericUpdateOp, ObjectPropertyIr,
    OwnedEnvBindingIr, ProgramIr, PropertyKeyIr, RelationalBinaryOp, ScriptIr, StatementIr,
    SwitchCaseIr, TypedExpr, UnaryNumericOp, LEXICAL_ARGUMENTS_NAME, LEXICAL_THIS_NAME,
    UpdateReturnMode, ValueKind, VarDeclaratorIr,
};
use wasm_encoder::{
    BlockType, CodeSection, ConstExpr, DataSection, ElementSection, Elements, ExportKind,
    ExportSection, Function, FunctionSection, GlobalSection, GlobalType, Ieee64, Instruction,
    MemArg, MemorySection, MemoryType, Module, RefType, TableSection, TableType, TypeSection,
    ValType,
};

const RESULT_TAG_EXPORT: &str = "result_tag";
const RESULT_TAG_GLOBAL_INDEX: u32 = 0;
const HEAP_PTR_GLOBAL_INDEX: u32 = 1;
const JS_FUNCTION_TYPE_INDEX: u32 = 1;
const WASM_PAGE_SIZE: u64 = 65_536;
const MIN_HEAP_CAPACITY: u64 = 1;
const HEAP_HEADER_SIZE: u64 = 24;
const HEAP_OBJECT_ENTRY_SIZE: u64 = 64;
const HEAP_ARRAY_ENTRY_SIZE: u64 = 16;
const HEAP_ARGUMENTS_MAPPED_COUNT_OFFSET: u64 = 24;
const HEAP_ARGUMENTS_ENV_HANDLE_OFFSET: u64 = 32;
const HEAP_PTR_OFFSET: u64 = 0;
const HEAP_LEN_OFFSET: u64 = 8;
const HEAP_CAP_OFFSET: u64 = 16;
const HEAP_OBJECT_KEY_OFFSET: u64 = 0;
const HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET: u64 = 8;
const HEAP_OBJECT_DATA_TAG_OFFSET: u64 = 16;
const HEAP_OBJECT_DATA_PAYLOAD_OFFSET: u64 = 24;
const HEAP_OBJECT_GETTER_TAG_OFFSET: u64 = 32;
const HEAP_OBJECT_GETTER_PAYLOAD_OFFSET: u64 = 40;
const HEAP_OBJECT_SETTER_TAG_OFFSET: u64 = 48;
const HEAP_OBJECT_SETTER_PAYLOAD_OFFSET: u64 = 56;
const HEAP_ARRAY_TAG_OFFSET: u64 = 0;
const HEAP_ARRAY_PAYLOAD_OFFSET: u64 = 8;
const ENV_PARENT_OFFSET: u64 = 0;
const ENV_SLOT_BASE_OFFSET: u64 = 8;
const ENV_SLOT_SIZE: u64 = 16;
const ENV_SLOT_TAG_OFFSET: u64 = 0;
const ENV_SLOT_PAYLOAD_OFFSET: u64 = 8;
const OBJECT_DESCRIPTOR_DATA: u64 = 0;
const OBJECT_DESCRIPTOR_ACCESSOR: u64 = 1;
const JS_FUNCTION_PARAM_COUNT: usize = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmArtifact {
    pub bytes: Vec<u8>,
    pub invariant_note: &'static str,
    pub debug_dump: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmitError {
    message: String,
}

impl EmitError {
    pub fn unsupported(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl core::fmt::Display for EmitError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for EmitError {}

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
    let string_pool = StringPool::collect(script);
    let uses_heap = string_pool.uses_heap || script_uses_env(script) || script_uses_calls(script);
    let function_metas = build_function_metas(script.functions.as_slice());
    let uses_function_table = script_uses_function_table(script);
    let mut main_builder =
        FunctionBuilder::new_main(script, &string_pool, &function_metas, uses_heap);
    let main_function = main_builder.compile()?;
    let mut compiled_functions = Vec::with_capacity(script.functions.len());
    for function in &script.functions {
        let mut builder =
            FunctionBuilder::new_function(
                function,
                &string_pool,
                &function_metas,
                uses_heap,
            );
        compiled_functions.push(builder.compile()?);
    }

    let mut types = TypeSection::new();
    types.ty().function([], [ValType::I64]);
    if uses_function_table {
        types.ty().function(
            function_param_types(),
            [ValType::I64, ValType::I64],
        );
    }

    let mut functions = FunctionSection::new();
    functions.function(0);
    for _ in &script.functions {
        functions.function(JS_FUNCTION_TYPE_INDEX);
    }

    let mut exports = ExportSection::new();
    exports.export("main", ExportKind::Func, 0);
    exports.export(
        RESULT_TAG_EXPORT,
        ExportKind::Global,
        RESULT_TAG_GLOBAL_INDEX,
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
    if uses_heap {
        globals.global(
            GlobalType {
                val_type: ValType::I64,
                mutable: true,
                shared: false,
            },
            &ConstExpr::i64_const(align_heap_start(string_pool.bytes.len()) as i64),
        );
    }

    let mut code = CodeSection::new();
    code.function(&main_function);
    for function in &compiled_functions {
        code.function(function);
    }

    let mut module = Module::new();
    module.section(&types);
    module.section(&functions);
    if uses_function_table {
        let mut tables = TableSection::new();
        tables.table(TableType {
            element_type: RefType::FUNCREF,
            minimum: script.functions.len() as u64,
            maximum: Some(script.functions.len() as u64),
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
        format!("internal functions: {}", script.functions.len()),
        format!("export global: {RESULT_TAG_EXPORT}"),
    ];

    if !string_pool.bytes.is_empty() || uses_heap {
        let mut memories = MemorySection::new();
        memories.memory(MemoryType {
            minimum: 1,
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
            &ConstExpr::i32_const(0),
            string_pool.bytes.iter().copied(),
        );
        module.section(&globals);
        module.section(&exports);
        if uses_function_table {
            let mut elements = ElementSection::new();
            let function_indexes = (1..=script.functions.len() as u32).collect::<Vec<_>>();
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
            let function_indexes = (1..=script.functions.len() as u32).collect::<Vec<_>>();
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

#[derive(Debug, Clone, Copy)]
struct StringRef {
    offset: u32,
    len: u32,
}

#[derive(Debug, Default)]
struct StringPool {
    bytes: Vec<u8>,
    refs: BTreeMap<String, StringRef>,
    uses_heap: bool,
}

impl StringPool {
    fn collect(script: &ScriptIr) -> Self {
        let mut pool = Self::default();
        for function in &script.functions {
            pool.collect_block(&function.body);
        }
        pool.collect_block(&script.body);
        pool
    }

    fn collect_block(&mut self, block: &BlockIr) {
        for statement in &block.statements {
            self.collect_statement(statement);
        }
    }

    fn collect_statement(&mut self, statement: &StatementIr) {
        match statement {
            StatementIr::Empty
            | StatementIr::Debugger
            | StatementIr::Break { .. }
            | StatementIr::Continue { .. } => {}
            StatementIr::Lexical { init, .. } | StatementIr::Expression(init) => {
                self.collect_expr(init)
            }
            StatementIr::Return(value) => self.collect_expr(value),
            StatementIr::Var(declarators) => self.collect_var_declarators(declarators),
            StatementIr::Block(block) => self.collect_block(block),
            StatementIr::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.collect_expr(condition);
                self.collect_statement(then_branch);
                if let Some(else_branch) = else_branch {
                    self.collect_statement(else_branch);
                }
            }
            StatementIr::While { condition, body } => {
                self.collect_expr(condition);
                self.collect_statement(body);
            }
            StatementIr::DoWhile { body, condition } => {
                self.collect_statement(body);
                self.collect_expr(condition);
            }
            StatementIr::For {
                init,
                test,
                update,
                body,
            } => {
                if let Some(init) = init {
                    self.collect_for_init(init);
                }
                if let Some(test) = test {
                    self.collect_expr(test);
                }
                if let Some(update) = update {
                    self.collect_expr(update);
                }
                self.collect_statement(body);
            }
            StatementIr::Switch {
                discriminant,
                cases,
            } => {
                self.collect_expr(discriminant);
                for case in cases {
                    if let Some(condition) = &case.condition {
                        self.collect_expr(condition);
                    }
                    self.collect_block(&case.body);
                }
            }
            StatementIr::Labelled { statement, .. } => self.collect_statement(statement),
        }
    }

    fn collect_for_init(&mut self, init: &ForInitIr) {
        match init {
            ForInitIr::Lexical { init, .. } => self.collect_expr(init),
            ForInitIr::Var(declarators) => self.collect_var_declarators(declarators),
            ForInitIr::Expression(expr) => self.collect_expr(expr),
        }
    }

    fn collect_var_declarators(&mut self, declarators: &[VarDeclaratorIr]) {
        for declarator in declarators {
            if let Some(init) = &declarator.init {
                self.collect_expr(init);
            }
        }
    }

    fn collect_expr(&mut self, expr: &TypedExpr) {
        match &expr.expr {
            ExprIr::String(value) => self.intern_string(value),
            ExprIr::ObjectLiteral(properties) => {
                self.uses_heap = true;
                for property in properties {
                    match property {
                        ObjectPropertyIr::Data { key, value, .. } => {
                            self.intern_string(key);
                            self.collect_expr(value);
                        }
                        ObjectPropertyIr::Method { key, function }
                        | ObjectPropertyIr::Getter { key, function }
                        | ObjectPropertyIr::Setter { key, function } => {
                            self.intern_string(key);
                            self.collect_expr(function);
                        }
                    }
                }
            }
            ExprIr::ArrayLiteral(elements) => {
                self.uses_heap = true;
                for element in elements {
                    self.collect_expr(element);
                }
            }
            ExprIr::PropertyRead { target, key } => {
                self.uses_heap = true;
                self.collect_expr(target);
                self.collect_property_key(key);
            }
            ExprIr::PropertyWrite { target, key, value } => {
                self.uses_heap = true;
                self.collect_expr(target);
                self.collect_property_key(key);
                self.collect_expr(value);
            }
            ExprIr::AssignIdentifier { value, .. }
            | ExprIr::CompoundAssignIdentifier { value, .. }
            | ExprIr::UnaryNumber { expr: value, .. }
            | ExprIr::LogicalNot { expr: value } => self.collect_expr(value),
            ExprIr::UpdateIdentifier { .. } => {}
            ExprIr::BinaryNumber { lhs, rhs, .. }
            | ExprIr::CompareNumber { lhs, rhs, .. }
            | ExprIr::StrictEquality { lhs, rhs, .. }
            | ExprIr::LogicalShortCircuit { lhs, rhs, .. } => {
                self.collect_expr(lhs);
                self.collect_expr(rhs);
            }
            ExprIr::CallNamed { args, .. } => {
                self.uses_heap = true;
                for arg in args {
                    self.collect_expr(arg);
                }
            }
            ExprIr::CallIndirect { callee, args } => {
                self.uses_heap = true;
                self.collect_expr(callee);
                for arg in args {
                    self.collect_expr(arg);
                }
            }
            ExprIr::CallMethod { receiver, key, args } => {
                self.uses_heap = true;
                self.collect_expr(receiver);
                self.collect_property_key(key);
                for arg in args {
                    self.collect_expr(arg);
                }
            }
            ExprIr::Undefined
            | ExprIr::Null
            | ExprIr::Boolean(_)
            | ExprIr::Number(_)
            | ExprIr::FunctionValue(_)
            | ExprIr::This
            | ExprIr::Arguments
            | ExprIr::Identifier(_) => {}
        }
    }

    fn collect_property_key(&mut self, key: &PropertyKeyIr) {
        match key {
            PropertyKeyIr::StaticString(value) => self.intern_string(value),
            PropertyKeyIr::ArrayLength => {}
            PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                self.collect_expr(expr)
            }
        }
    }

    fn intern_string(&mut self, value: &str) {
        if self.refs.contains_key(value) {
            return;
        }
        let offset = self.bytes.len() as u32;
        let bytes = value.as_bytes();
        self.bytes.extend_from_slice(bytes);
        self.refs.insert(
            value.to_string(),
            StringRef {
                offset,
                len: bytes.len() as u32,
            },
        );
    }

    fn payload(&self, value: &str) -> i64 {
        let string = self.refs.get(value).expect("string must exist in pool");
        (((string.offset as u64) << 32) | string.len as u64) as i64
    }
}

fn align_heap_start(bytes: usize) -> u64 {
    ((bytes as u64) + 7) & !7
}

#[derive(Debug, Clone, Copy)]
enum ControlFrameKind {
    If,
    Block,
    Loop,
}

#[derive(Debug, Clone, Copy)]
struct LoopTargets {
    continue_frame: usize,
}

#[derive(Debug, Clone)]
struct LabelTargets {
    name: String,
    break_frame: usize,
    continue_frame: Option<usize>,
}

#[derive(Debug, Clone, Copy)]
enum BindingStorage {
    Fixed { payload_local: u32, kind: ValueKind },
    Dynamic { tag_local: u32, payload_local: u32 },
    EnvSlot { slot: u32, hops: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReturnAbi {
    MainExport,
    MultiValue,
}

#[derive(Debug, Clone)]
struct WasmFunctionMeta {
    name: String,
    wasm_index: u32,
    table_index: u32,
}

struct FunctionBuilder<'a> {
    body: &'a BlockIr,
    params: &'a [FunctionParamIr],
    owned_env_bindings: &'a [OwnedEnvBindingIr],
    captured_bindings: &'a [porffor_ir::CapturedBindingIr],
    strings: &'a StringPool,
    functions: &'a BTreeMap<FunctionId, WasmFunctionMeta>,
    function_id: Option<FunctionId>,
    function_flavor: FunctionFlavor,
    self_binding_name: Option<String>,
    uses_heap: bool,
    return_abi: ReturnAbi,
    binding_scopes: Vec<BTreeMap<String, BindingStorage>>,
    hoisted_vars: Vec<String>,
    next_binding_local: u32,
    total_binding_local_count: u32,
    temp_local_count: u32,
    current_env_local: u32,
    result_local: u32,
    result_tag_local: u32,
    scratch_local: u32,
    temp_local_base: u32,
    temp_stack_depth: u32,
    this_payload_local: Option<u32>,
    this_tag_local: Option<u32>,
    control_stack: Vec<ControlFrameKind>,
    breakable_stack: Vec<usize>,
    loop_stack: Vec<LoopTargets>,
    label_stack: Vec<LabelTargets>,
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
            None,
            uses_heap,
            ReturnAbi::MainExport,
        )
    }

    fn new_function(
        function: &'a FunctionIr,
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
            (!function.is_expression || function.is_named_expression)
                .then(|| function.name.clone()),
            uses_heap,
            ReturnAbi::MultiValue,
        )
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
        self_binding_name: Option<String>,
        uses_heap: bool,
        return_abi: ReturnAbi,
    ) -> Self {
        let hoisted_vars = collect_hoisted_vars_block_root(body);
        let self_binding_local_count = usize::from(self_binding_name.is_some());
        let param_local_count = count_param_locals(return_abi) as u32;
        let needs_arguments_binding_locals =
            matches!(return_abi, ReturnAbi::MultiValue) && function_flavor == FunctionFlavor::Ordinary;
        let captured_arguments_local_count = if captured_bindings
            .iter()
            .any(|binding| binding.name == LEXICAL_ARGUMENTS_NAME)
        {
            2
        } else {
            0
        };
        let total_binding_local_count =
            (count_block_lexicals(body)
                + self_binding_local_count
                + count_param_binding_locals(params, owned_env_bindings)
                + if needs_arguments_binding_locals { 2 } else { 0 }
                + captured_arguments_local_count) as u32
                + (hoisted_vars.len() as u32 * 2);
        let temp_local_count = count_block_temp_locals(body).max(64) as u32;
        let current_env_local = param_local_count + total_binding_local_count;
        Self {
            body,
            params,
            owned_env_bindings,
            captured_bindings,
            strings,
            functions,
            function_id,
            function_flavor,
            self_binding_name,
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
            scratch_local: current_env_local + 3,
            temp_local_base: current_env_local + 4,
            temp_stack_depth: 0,
            this_payload_local: matches!(return_abi, ReturnAbi::MultiValue).then_some(1),
            this_tag_local: matches!(return_abi, ReturnAbi::MultiValue).then_some(2),
            control_stack: Vec::new(),
            breakable_stack: Vec::new(),
            loop_stack: Vec::new(),
            label_stack: Vec::new(),
        }
    }

    fn local_count(&self) -> usize {
        self.total_binding_local_count as usize + 4 + self.temp_local_count as usize
    }

    fn compile(&mut self) -> Result<Function, EmitError> {
        let mut function =
            Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, self.local_count()));

        self.push_scope();
        self.init_current_env(&mut function)?;
        self.bind_captured_bindings(&mut function);
        self.bind_self_function(&mut function)?;
        self.bind_parameters(&mut function)?;
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
        self.compile_block_contents(self.body, &mut function)?;
        self.pop_scope();

        match self.return_abi {
            ReturnAbi::MainExport => {
                function.instruction(&Instruction::LocalGet(self.result_tag_local));
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::GlobalSet(RESULT_TAG_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalGet(self.result_local));
            }
            ReturnAbi::MultiValue => {
                function.instruction(&Instruction::LocalGet(self.result_local));
                function.instruction(&Instruction::LocalGet(self.result_tag_local));
            }
        }
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
        }
        self.release_temp_local(parent_env_local);
        Ok(())
    }

    fn bind_captured_bindings(&mut self, function: &mut Function) {
        for binding in self.captured_bindings {
            if binding.name == LEXICAL_ARGUMENTS_NAME {
                let payload_local = self.next_binding_local;
                let tag_local = self.next_binding_local + 1;
                self.next_binding_local += 2;
                let storage = BindingStorage::Dynamic {
                    tag_local,
                    payload_local,
                };
                self.read_env_slot_to_locals(
                    binding.slot,
                    binding.hops,
                    payload_local,
                    tag_local,
                    function,
                );
                self.binding_scopes
                    .last_mut()
                    .expect("binding scope stack must exist")
                    .insert(binding.name.clone(), storage);
            } else {
                self.binding_scopes
                    .last_mut()
                    .expect("binding scope stack must exist")
                    .insert(
                        binding.name.clone(),
                        BindingStorage::EnvSlot {
                            slot: binding.slot,
                            hops: binding.hops,
                        },
                    );
            }
        }
    }

    fn bind_parameters(&mut self, function: &mut Function) -> Result<(), EmitError> {
        if matches!(self.return_abi, ReturnAbi::MultiValue)
            && self.function_flavor == FunctionFlavor::Ordinary
        {
            let payload_local = self.next_binding_local;
            let tag_local = self.next_binding_local + 1;
            self.next_binding_local += 2;
            let arguments_storage = BindingStorage::Dynamic {
                tag_local,
                payload_local,
            };
            self.binding_scopes
                .last_mut()
                .expect("binding scope stack must exist")
                .insert(LEXICAL_ARGUMENTS_NAME.to_string(), arguments_storage);
            self.initialize_arguments_binding(arguments_storage, function)?;
            if let Some(slot) = self.owned_env_slot(LEXICAL_ARGUMENTS_NAME) {
                self.write_env_slot_from_locals(slot, 0, payload_local, tag_local, function);
            }
        }

        for param in self.params {
            let storage = self.allocate_dynamic_binding_storage(&param.name);
            self.binding_scopes
                .last_mut()
                .expect("binding scope stack must exist")
                .insert(param.name.clone(), storage);
        }

        for (index, param) in self.params.iter().enumerate() {
            let storage = self.lookup_binding(&param.name).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: missing parameter binding `{}`",
                    param.name
                ))
            })?;
            if param.is_rest {
                self.initialize_rest_parameter(index, storage, function)?;
                continue;
            }
            self.initialize_parameter(index, param, storage, function)?;
        }
        Ok(())
    }

    fn allocate_dynamic_binding_storage(&mut self, name: &str) -> BindingStorage {
        if let Some(slot) = self.owned_env_slot(name) {
            BindingStorage::EnvSlot { slot, hops: 0 }
        } else {
            let payload_local = self.next_binding_local;
            let tag_local = self.next_binding_local + 1;
            self.next_binding_local += 2;
            BindingStorage::Dynamic {
                tag_local,
                payload_local,
            }
        }
    }

    fn initialize_arguments_binding(
        &mut self,
        storage: BindingStorage,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        self.emit_arguments_object_payload(function)?;
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.write_binding_from_locals(storage, payload_local, tag_local, function);
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        Ok(())
    }

    fn initialize_parameter(
        &mut self,
        index: usize,
        param: &FunctionParamIr,
        storage: BindingStorage,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        self.read_argument_at_index(index, payload_local, tag_local, function);

        if let Some(default_init) = &param.default_init {
            function.instruction(&Instruction::LocalGet(tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.compile_expr_to_locals(default_init, payload_local, tag_local, function)?;
            function.instruction(&Instruction::End);
        }

        self.write_binding_from_locals(storage, payload_local, tag_local, function);
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        Ok(())
    }

    fn initialize_rest_parameter(
        &mut self,
        index: usize,
        storage: BindingStorage,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        self.emit_rest_array_payload(index, function)?;
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.write_binding_from_locals(storage, payload_local, tag_local, function);
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        Ok(())
    }

    fn bind_self_function(&mut self, function: &mut Function) -> Result<(), EmitError> {
        let (Some(function_id), Some(function_name)) =
            (self.function_id.as_ref(), self.self_binding_name.as_ref())
        else {
            return Ok(());
        };
        let Some(meta) = self.functions.get(function_id) else {
            return Err(EmitError::unsupported(format!(
                "unsupported in porffor wasm-aot first slice: unknown function `{function_id}`"
            )));
        };
        let storage = if let Some(slot) = self.owned_env_slot(function_name) {
            BindingStorage::EnvSlot { slot, hops: 0 }
        } else {
            let payload_local = self.next_binding_local;
            self.next_binding_local += 1;
            BindingStorage::Fixed {
                payload_local,
                kind: ValueKind::Function,
            }
        };
        self.binding_scopes
            .last_mut()
            .expect("binding scope stack must exist")
            .insert(function_name.clone(), storage);
        self.emit_function_value_payload(meta.table_index, function);
        match storage {
            BindingStorage::Fixed { payload_local, .. } => {
                function.instruction(&Instruction::LocalSet(payload_local));
            }
            BindingStorage::EnvSlot { .. } => {
                function.instruction(&Instruction::LocalSet(self.scratch_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                self.write_binding_from_locals(
                    storage,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                );
            }
            BindingStorage::Dynamic { .. } => unreachable!(),
        }
        Ok(())
    }

    fn compile_this_payload(&mut self, function: &mut Function) -> Result<(), EmitError> {
        match self.function_flavor {
            FunctionFlavor::Ordinary => {
                let Some(this_payload_local) = self.this_payload_local else {
                    return Err(EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: top-level `this`",
                    ));
                };
                function.instruction(&Instruction::LocalGet(this_payload_local));
            }
            FunctionFlavor::Arrow => {
                let storage = self.lookup_binding(LEXICAL_THIS_NAME).ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: missing lexical `this` capture",
                    )
                })?;
                self.read_binding_payload(storage, function);
            }
        }
        Ok(())
    }

    fn compile_this_to_locals(
        &mut self,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match self.function_flavor {
            FunctionFlavor::Ordinary => {
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
                function.instruction(&Instruction::LocalGet(this_payload_local));
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::LocalGet(this_tag_local));
                function.instruction(&Instruction::LocalSet(tag_local));
            }
            FunctionFlavor::Arrow => {
                let storage = self.lookup_binding(LEXICAL_THIS_NAME).ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: missing lexical `this` capture",
                    )
                })?;
                self.read_binding_to_locals(storage, payload_local, tag_local, function);
            }
        }
        Ok(())
    }

    const fn argc_param_local(&self) -> u32 {
        3
    }

    const fn argv_param_local(&self) -> u32 {
        4
    }

    fn has_simple_parameter_list(&self) -> bool {
        self.params
            .iter()
            .all(|param| !param.is_rest && param.default_init.is_none())
    }

    fn uses_mapped_arguments_object(&self) -> bool {
        self.has_simple_parameter_list() && self.owned_env_slot(LEXICAL_ARGUMENTS_NAME).is_none()
    }

    fn read_argument_at_index(
        &mut self,
        index: usize,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        let index_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(index as i64));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        self.emit_array_read(
            self.argv_param_local(),
            index_local,
            payload_local,
            tag_local,
            function,
        );
        function.instruction(&Instruction::End);
        self.release_temp_local(index_local);
    }

    fn owned_env_slot(&self, name: &str) -> Option<u32> {
        self.owned_env_bindings
            .iter()
            .find(|binding| binding.name == name)
            .map(|binding| binding.slot)
    }

    fn compile_block_contents(
        &mut self,
        block: &BlockIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if block.statements.is_empty() {
            self.emit_statement_result(function, ValueKind::Undefined);
            return Ok(());
        }

        for statement in &block.statements {
            self.compile_statement(statement, function)?;
        }

        Ok(())
    }

    fn compile_statement(
        &mut self,
        statement: &StatementIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match statement {
            StatementIr::Empty => {
                self.emit_statement_result(function, ValueKind::Undefined);
            }
            StatementIr::Lexical { mode, name, init } => {
                let storage = self.allocate_binding(name.clone(), *mode, init.kind);
                self.compile_expr_to_binding(init, storage, function)?;
                self.emit_statement_result(function, ValueKind::Undefined);
            }
            StatementIr::Expression(expr) => {
                if expr.kind == ValueKind::Dynamic {
                    self.compile_expr_to_locals(
                        expr,
                        self.result_local,
                        self.result_tag_local,
                        function,
                    )?;
                } else {
                    self.compile_expr_payload(expr, function)?;
                    self.finish_statement_payload(function, expr.kind);
                }
            }
            StatementIr::Var(declarators) => {
                self.compile_var_declarators(declarators, function)?;
                self.emit_statement_result(function, ValueKind::Undefined);
            }
            StatementIr::Block(block) => {
                self.push_scope();
                self.compile_block_contents(block, function)?;
                self.pop_scope();
            }
            StatementIr::Labelled { labels, statement } => {
                self.compile_labelled_statement(labels, statement, function)?;
            }
            StatementIr::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.compile_truthy_i32(condition, function)?;
                function.instruction(&Instruction::If(BlockType::Empty));
                self.push_control(ControlFrameKind::If);
                self.compile_statement(then_branch, function)?;
                function.instruction(&Instruction::Else);
                if let Some(else_branch) = else_branch {
                    self.compile_statement(else_branch, function)?;
                } else {
                    self.emit_statement_result(function, ValueKind::Undefined);
                }
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
            }
            StatementIr::While { condition, body } => {
                self.compile_while(condition, body, &[], function)?;
            }
            StatementIr::DoWhile { body, condition } => {
                self.compile_do_while(body, condition, &[], function)?;
            }
            StatementIr::For {
                init,
                test,
                update,
                body,
            } => {
                self.compile_for(
                    init.as_ref(),
                    test.as_ref(),
                    update.as_ref(),
                    body,
                    &[],
                    function,
                )?;
            }
            StatementIr::Switch {
                discriminant,
                cases,
            } => {
                self.compile_switch(discriminant, cases, &[], function)?;
            }
            StatementIr::Debugger => {
                self.emit_statement_result(function, ValueKind::Undefined);
            }
            StatementIr::Return(value) => {
                self.compile_expr_to_locals(
                    value,
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(self.result_local));
                function.instruction(&Instruction::LocalGet(self.result_tag_local));
                function.instruction(&Instruction::Return);
            }
            StatementIr::Break { label } => self.compile_break(label.as_deref(), function)?,
            StatementIr::Continue { label } => self.compile_continue(label.as_deref(), function)?,
        }
        Ok(())
    }

    fn compile_labelled_statement(
        &mut self,
        labels: &[String],
        statement: &StatementIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match statement {
            StatementIr::Block(block) => {
                function.instruction(&Instruction::Block(BlockType::Empty));
                let break_frame = self.push_control(ControlFrameKind::Block);
                self.push_labels(labels, break_frame, None);
                self.push_scope();
                self.compile_block_contents(block, function)?;
                self.pop_scope();
                self.pop_labels(labels.len());
                self.pop_control(ControlFrameKind::Block);
                function.instruction(&Instruction::End);
            }
            StatementIr::While { condition, body } => {
                self.compile_while(condition, body, labels, function)?;
            }
            StatementIr::DoWhile { body, condition } => {
                self.compile_do_while(body, condition, labels, function)?;
            }
            StatementIr::For {
                init,
                test,
                update,
                body,
            } => {
                self.compile_for(
                    init.as_ref(),
                    test.as_ref(),
                    update.as_ref(),
                    body,
                    labels,
                    function,
                )?;
            }
            StatementIr::Switch {
                discriminant,
                cases,
            } => {
                self.compile_switch(discriminant, cases, labels, function)?;
            }
            _ => {
                return Err(EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: label on unsupported statement kind",
                ));
            }
        }
        Ok(())
    }

    fn compile_while(
        &mut self,
        condition: &TypedExpr,
        body: &StatementIr,
        labels: &[String],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_statement_result(function, ValueKind::Undefined);
        function.instruction(&Instruction::Block(BlockType::Empty));
        let break_frame = self.push_control(ControlFrameKind::Block);
        self.breakable_stack.push(break_frame);
        function.instruction(&Instruction::Loop(BlockType::Empty));
        let continue_frame = self.push_control(ControlFrameKind::Loop);
        self.loop_stack.push(LoopTargets { continue_frame });
        self.push_labels(labels, break_frame, Some(continue_frame));
        self.compile_truthy_i32(condition, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(self.depth_to(break_frame)));
        self.compile_statement(body, function)?;
        function.instruction(&Instruction::Br(self.depth_to(continue_frame)));
        self.pop_labels(labels.len());
        self.loop_stack.pop();
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.breakable_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn compile_do_while(
        &mut self,
        body: &StatementIr,
        condition: &TypedExpr,
        labels: &[String],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_statement_result(function, ValueKind::Undefined);
        function.instruction(&Instruction::Block(BlockType::Empty));
        let break_frame = self.push_control(ControlFrameKind::Block);
        self.breakable_stack.push(break_frame);
        function.instruction(&Instruction::Loop(BlockType::Empty));
        let loop_frame = self.push_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::Block(BlockType::Empty));
        let continue_frame = self.push_control(ControlFrameKind::Block);
        self.loop_stack.push(LoopTargets { continue_frame });
        self.push_labels(labels, break_frame, Some(continue_frame));
        self.compile_statement(body, function)?;
        self.pop_labels(labels.len());
        self.loop_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.compile_truthy_i32(condition, function)?;
        function.instruction(&Instruction::BrIf(self.depth_to(loop_frame)));
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.breakable_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn compile_for(
        &mut self,
        init: Option<&ForInitIr>,
        test: Option<&TypedExpr>,
        update: Option<&TypedExpr>,
        body: &StatementIr,
        labels: &[String],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.push_scope();
        if let Some(init) = init {
            self.compile_for_init(init, function)?;
        }
        self.emit_statement_result(function, ValueKind::Undefined);
        function.instruction(&Instruction::Block(BlockType::Empty));
        let break_frame = self.push_control(ControlFrameKind::Block);
        self.breakable_stack.push(break_frame);
        function.instruction(&Instruction::Loop(BlockType::Empty));
        let loop_frame = self.push_control(ControlFrameKind::Loop);
        if let Some(test) = test {
            self.compile_truthy_i32(test, function)?;
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::BrIf(self.depth_to(break_frame)));
        }
        function.instruction(&Instruction::Block(BlockType::Empty));
        let continue_frame = self.push_control(ControlFrameKind::Block);
        self.loop_stack.push(LoopTargets { continue_frame });
        self.push_labels(labels, break_frame, Some(continue_frame));
        self.compile_statement(body, function)?;
        self.pop_labels(labels.len());
        self.loop_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        if let Some(update) = update {
            self.compile_expr_payload(update, function)?;
            function.instruction(&Instruction::Drop);
        }
        function.instruction(&Instruction::Br(self.depth_to(loop_frame)));
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.breakable_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.pop_scope();
        Ok(())
    }

    fn compile_switch(
        &mut self,
        discriminant: &TypedExpr,
        cases: &[SwitchCaseIr],
        labels: &[String],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let discriminant_payload_local = self.reserve_temp_local();
        let discriminant_tag_local = self.reserve_temp_local();
        let selected_local = self.reserve_temp_local();
        let active_local = self.reserve_temp_local();
        let default_index = cases
            .iter()
            .enumerate()
            .find_map(|(index, case)| case.condition.is_none().then_some(index as i64));

        self.emit_statement_result(function, ValueKind::Undefined);
        self.push_scope();
        self.compile_expr_to_locals(
            discriminant,
            discriminant_payload_local,
            discriminant_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(selected_local));

        for (index, case) in cases.iter().enumerate() {
            let Some(condition) = &case.condition else {
                continue;
            };
            function.instruction(&Instruction::LocalGet(selected_local));
            function.instruction(&Instruction::I64Const(-1));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.compile_switch_case_match(
                discriminant,
                discriminant_payload_local,
                discriminant_tag_local,
                condition,
                function,
            )?;
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(index as i64));
            function.instruction(&Instruction::LocalSet(selected_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }

        if let Some(default_index) = default_index {
            function.instruction(&Instruction::LocalGet(selected_local));
            function.instruction(&Instruction::I64Const(-1));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(default_index));
            function.instruction(&Instruction::LocalSet(selected_local));
            function.instruction(&Instruction::End);
        }

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(active_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        let break_frame = self.push_control(ControlFrameKind::Block);
        self.breakable_stack.push(break_frame);
        self.push_labels(labels, break_frame, None);
        for (index, case) in cases.iter().enumerate() {
            function.instruction(&Instruction::LocalGet(active_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(selected_local));
            function.instruction(&Instruction::I64Const(index as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(active_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalGet(active_local));
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.push_control(ControlFrameKind::If);
            self.compile_switch_case_body(&case.body, function)?;
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        self.pop_labels(labels.len());
        self.breakable_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.pop_scope();
        self.release_temp_local(active_local);
        self.release_temp_local(selected_local);
        self.release_temp_local(discriminant_tag_local);
        self.release_temp_local(discriminant_payload_local);
        Ok(())
    }

    fn compile_switch_case_body(
        &mut self,
        block: &BlockIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        for statement in &block.statements {
            self.compile_statement(statement, function)?;
        }
        Ok(())
    }

    fn compile_switch_case_match(
        &mut self,
        discriminant: &TypedExpr,
        discriminant_payload_local: u32,
        discriminant_tag_local: u32,
        condition: &TypedExpr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if discriminant.kind != ValueKind::Dynamic
            && condition.kind != ValueKind::Dynamic
            && discriminant.kind != condition.kind
        {
            function.instruction(&Instruction::I32Const(0));
            return Ok(());
        }

        if discriminant.kind != ValueKind::Dynamic && condition.kind != ValueKind::Dynamic {
            self.compile_expr_payload(condition, function)?;
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            match discriminant.kind {
                ValueKind::Number => {
                    function.instruction(&Instruction::LocalGet(discriminant_payload_local));
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::LocalGet(self.scratch_local));
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::F64Eq);
                }
                _ => {
                    function.instruction(&Instruction::LocalGet(discriminant_payload_local));
                    function.instruction(&Instruction::LocalGet(self.scratch_local));
                    function.instruction(&Instruction::I64Eq);
                }
            }
            return Ok(());
        }

        self.compile_expr_to_locals(
            condition,
            self.scratch_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_tagged_payload_equality_i32(
            discriminant_tag_local,
            discriminant_payload_local,
            self.result_tag_local,
            self.scratch_local,
            function,
        )?;
        Ok(())
    }

    fn compile_break(
        &mut self,
        label: Option<&str>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let break_frame = if let Some(label) = label {
            self.label_stack
                .iter()
                .rev()
                .find(|targets| targets.name == label)
                .map(|targets| targets.break_frame)
                .ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: unknown label `{label}`"
                    ))
                })?
        } else {
            *self.breakable_stack.last().ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: break outside loop or switch",
                )
            })?
        };
        function.instruction(&Instruction::Br(self.depth_to(break_frame)));
        Ok(())
    }

    fn compile_continue(
        &mut self,
        label: Option<&str>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let continue_frame = if let Some(label) = label {
            self.label_stack
                .iter()
                .rev()
                .find(|targets| targets.name == label)
                .and_then(|targets| targets.continue_frame)
                .ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: continue to non-loop label `{label}`"
                    ))
                })?
        } else {
            self.loop_stack
                .last()
                .copied()
                .ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: continue outside loop",
                    )
                })?
                .continue_frame
        };
        function.instruction(&Instruction::Br(self.depth_to(continue_frame)));
        Ok(())
    }

    fn compile_for_init(
        &mut self,
        init: &ForInitIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match init {
            ForInitIr::Lexical { mode, name, init } => {
                let storage = self.allocate_binding(name.clone(), *mode, init.kind);
                self.compile_expr_to_binding(init, storage, function)?;
            }
            ForInitIr::Var(declarators) => {
                self.compile_var_declarators(declarators, function)?;
            }
            ForInitIr::Expression(expr) => {
                self.compile_expr_payload(expr, function)?;
                function.instruction(&Instruction::Drop);
            }
        }
        Ok(())
    }

    fn compile_var_declarators(
        &mut self,
        declarators: &[VarDeclaratorIr],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        for declarator in declarators {
            let Some(init) = &declarator.init else {
                continue;
            };
            let storage = self.lookup_binding(&declarator.name).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: unbound identifier `{}`",
                    declarator.name
                ))
            })?;
            self.compile_expr_to_binding(init, storage, function)?;
        }
        Ok(())
    }

    fn compile_expr_payload(
        &mut self,
        expr: &TypedExpr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if expr.kind == ValueKind::Dynamic {
            self.compile_expr_to_locals(expr, self.scratch_local, self.result_tag_local, function)?;
            function.instruction(&Instruction::LocalGet(self.scratch_local));
            return Ok(());
        }

        match &expr.expr {
            ExprIr::Undefined | ExprIr::Null => {
                self.emit_undefined_payload(function);
            }
            ExprIr::Boolean(value) => {
                function.instruction(&Instruction::I64Const(i64::from(*value)));
            }
            ExprIr::Number(bits) => {
                function.instruction(&Instruction::I64Const(*bits as i64));
            }
            ExprIr::String(value) => {
                function.instruction(&Instruction::I64Const(self.strings.payload(value)));
            }
            ExprIr::FunctionValue(function_id) => {
                let meta = self.functions.get(function_id).ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: unknown function value `{function_id}`"
                    ))
                })?;
                self.emit_function_value_payload(meta.table_index, function);
            }
            ExprIr::This => {
                self.compile_this_payload(function)?;
            }
            ExprIr::Arguments => {
                let storage = self.lookup_binding(LEXICAL_ARGUMENTS_NAME).ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: missing `arguments` binding",
                    )
                })?;
                self.read_binding_payload(storage, function);
            }
            ExprIr::ObjectLiteral(properties) => {
                self.compile_object_literal_payload(properties, function)?;
            }
            ExprIr::ArrayLiteral(elements) => {
                self.compile_array_literal_payload(elements, function)?;
            }
            ExprIr::Identifier(name) => {
                let storage = self.lookup_binding(name).ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: unbound identifier `{name}`"
                    ))
                })?;
                self.read_binding_payload(storage, function);
            }
            ExprIr::AssignIdentifier { name, value } => {
                let storage = self.lookup_binding(name).ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: unbound identifier `{name}`"
                    ))
                })?;
                self.compile_expr_to_binding(value, storage, function)?;
                self.read_binding_payload(storage, function);
            }
            ExprIr::PropertyWrite { target, key, value } => {
                self.compile_property_write_payload(target, key, value, function)?;
            }
            ExprIr::UpdateIdentifier {
                name,
                op,
                return_mode,
            } => {
                let storage = self.lookup_binding(name).ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: unbound identifier `{name}`"
                    ))
                })?;
                let value_local = self.reserve_temp_local();
                let tag_local = self.reserve_temp_local();
                self.read_binding_to_locals(storage, value_local, tag_local, function);
                match return_mode {
                    UpdateReturnMode::Prefix => {
                        function.instruction(&Instruction::LocalGet(value_local));
                        self.emit_update_delta(*op, function);
                        function.instruction(&Instruction::LocalSet(self.scratch_local));
                        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                        function.instruction(&Instruction::LocalSet(self.result_tag_local));
                        self.write_binding_from_locals(
                            storage,
                            self.scratch_local,
                            self.result_tag_local,
                            function,
                        );
                        function.instruction(&Instruction::LocalGet(self.scratch_local));
                    }
                    UpdateReturnMode::Postfix => {
                        function.instruction(&Instruction::LocalGet(value_local));
                        function.instruction(&Instruction::LocalSet(self.scratch_local));
                        function.instruction(&Instruction::LocalGet(value_local));
                        self.emit_update_delta(*op, function);
                        function.instruction(&Instruction::LocalSet(value_local));
                        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                        function.instruction(&Instruction::LocalSet(tag_local));
                        self.write_binding_from_locals(storage, value_local, tag_local, function);
                        function.instruction(&Instruction::LocalGet(self.scratch_local));
                    }
                };
                self.release_temp_local(tag_local);
                self.release_temp_local(value_local);
            }
            ExprIr::CompoundAssignIdentifier { name, op, value } => {
                let storage = self.lookup_binding(name).ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: unbound identifier `{name}`"
                    ))
                })?;
                let temp_local = self.reserve_temp_local();
                let tag_local = self.reserve_temp_local();
                self.read_binding_to_locals(storage, temp_local, tag_local, function);
                self.compile_expr_payload(value, function)?;
                function.instruction(&Instruction::LocalSet(self.scratch_local));
                if matches!(op, ArithmeticBinaryOp::Mod) {
                    function.instruction(&Instruction::LocalGet(temp_local));
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::LocalGet(temp_local));
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::LocalGet(self.scratch_local));
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::F64Div);
                    function.instruction(&Instruction::F64Trunc);
                    function.instruction(&Instruction::LocalGet(self.scratch_local));
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::F64Mul);
                    function.instruction(&Instruction::F64Sub);
                } else {
                    function.instruction(&Instruction::LocalGet(temp_local));
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::LocalGet(self.scratch_local));
                    function.instruction(&Instruction::F64ReinterpretI64);
                    match op {
                        ArithmeticBinaryOp::Add => function.instruction(&Instruction::F64Add),
                        ArithmeticBinaryOp::Sub => function.instruction(&Instruction::F64Sub),
                        ArithmeticBinaryOp::Mul => function.instruction(&Instruction::F64Mul),
                        ArithmeticBinaryOp::Div => function.instruction(&Instruction::F64Div),
                        ArithmeticBinaryOp::Mod => unreachable!(),
                    };
                }
                function.instruction(&Instruction::I64ReinterpretF64);
                function.instruction(&Instruction::LocalSet(self.scratch_local));
                function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
                self.write_binding_from_locals(storage, self.scratch_local, tag_local, function);
                function.instruction(&Instruction::LocalGet(self.scratch_local));
                self.release_temp_local(tag_local);
                self.release_temp_local(temp_local);
            }
            ExprIr::UnaryNumber { op, expr } => {
                self.compile_expr_payload(expr, function)?;
                match op {
                    UnaryNumericOp::Plus => {}
                    UnaryNumericOp::Minus => {
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Neg);
                        function.instruction(&Instruction::I64ReinterpretF64);
                    }
                }
            }
            ExprIr::LogicalNot { expr } => {
                self.compile_truthy_i32(expr, function)?;
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::I64ExtendI32U);
            }
            ExprIr::BinaryNumber { op, lhs, rhs } => {
                if matches!(op, ArithmeticBinaryOp::Mod) {
                    self.compile_expr_payload(lhs, function)?;
                    function.instruction(&Instruction::LocalSet(self.result_local));
                    self.compile_expr_payload(rhs, function)?;
                    function.instruction(&Instruction::LocalSet(self.scratch_local));
                    function.instruction(&Instruction::LocalGet(self.result_local));
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::LocalGet(self.result_local));
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::LocalGet(self.scratch_local));
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::F64Div);
                    function.instruction(&Instruction::F64Trunc);
                    function.instruction(&Instruction::LocalGet(self.scratch_local));
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::F64Mul);
                    function.instruction(&Instruction::F64Sub);
                    function.instruction(&Instruction::I64ReinterpretF64);
                } else {
                    self.compile_expr_payload(lhs, function)?;
                    function.instruction(&Instruction::F64ReinterpretI64);
                    self.compile_expr_payload(rhs, function)?;
                    function.instruction(&Instruction::F64ReinterpretI64);
                    match op {
                        ArithmeticBinaryOp::Add => function.instruction(&Instruction::F64Add),
                        ArithmeticBinaryOp::Sub => function.instruction(&Instruction::F64Sub),
                        ArithmeticBinaryOp::Mul => function.instruction(&Instruction::F64Mul),
                        ArithmeticBinaryOp::Div => function.instruction(&Instruction::F64Div),
                        ArithmeticBinaryOp::Mod => unreachable!(),
                    };
                    function.instruction(&Instruction::I64ReinterpretF64);
                }
            }
            ExprIr::CompareNumber { op, lhs, rhs } => {
                self.compile_expr_payload(lhs, function)?;
                function.instruction(&Instruction::F64ReinterpretI64);
                self.compile_expr_payload(rhs, function)?;
                function.instruction(&Instruction::F64ReinterpretI64);
                match op {
                    RelationalBinaryOp::LessThan => function.instruction(&Instruction::F64Lt),
                    RelationalBinaryOp::LessThanOrEqual => {
                        function.instruction(&Instruction::F64Le)
                    }
                    RelationalBinaryOp::GreaterThan => function.instruction(&Instruction::F64Gt),
                    RelationalBinaryOp::GreaterThanOrEqual => {
                        function.instruction(&Instruction::F64Ge)
                    }
                };
                function.instruction(&Instruction::I64ExtendI32U);
            }
            ExprIr::StrictEquality { op, lhs, rhs } => {
                self.compile_strict_equality_i32(lhs, rhs, function)?;
                if matches!(op, EqualityBinaryOp::StrictNotEqual) {
                    function.instruction(&Instruction::I32Eqz);
                }
                function.instruction(&Instruction::I64ExtendI32U);
            }
            ExprIr::LogicalShortCircuit { op, lhs, rhs } => {
                self.compile_expr_to_locals(
                    lhs,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                self.compile_truthy_tagged_i32(
                    self.result_tag_local,
                    self.scratch_local,
                    function,
                )?;
                function.instruction(&Instruction::If(BlockType::Empty));
                match op {
                    LogicalBinaryOp::And => {
                        self.compile_expr_to_locals(
                            rhs,
                            self.scratch_local,
                            self.result_tag_local,
                            function,
                        )?;
                    }
                    LogicalBinaryOp::Or => {}
                }
                function.instruction(&Instruction::Else);
                match op {
                    LogicalBinaryOp::And => {}
                    LogicalBinaryOp::Or => {
                        self.compile_expr_to_locals(
                            rhs,
                            self.scratch_local,
                            self.result_tag_local,
                            function,
                        )?;
                    }
                }
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::LocalGet(self.scratch_local));
            }
            ExprIr::CallNamed { name, args } => {
                self.emit_call(name, args, function)?;
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
            }
            ExprIr::CallIndirect { callee, args } => {
                self.emit_indirect_call(callee, None, args, function)?;
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
            }
            ExprIr::CallMethod { receiver, key, args } => {
                self.emit_method_call(receiver, key, args, function)?;
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
            }
            ExprIr::PropertyRead { target, key } => {
                self.compile_property_read_to_locals(
                    target,
                    key,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(self.scratch_local));
            }
        }
        Ok(())
    }

    fn compile_expr_to_binding(
        &mut self,
        expr: &TypedExpr,
        storage: BindingStorage,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match storage {
            BindingStorage::Fixed { .. } => {
                self.compile_expr_payload(expr, function)?;
                self.store_payload_to_binding(storage, function);
            }
            BindingStorage::Dynamic {
                tag_local,
                payload_local,
            } => {
                self.compile_expr_to_locals(expr, payload_local, tag_local, function)?;
            }
            BindingStorage::EnvSlot { .. } => {
                self.compile_expr_to_locals(expr, self.scratch_local, self.result_tag_local, function)?;
                self.write_binding_from_locals(
                    storage,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                );
            }
        }
        Ok(())
    }

    fn compile_expr_to_locals(
        &mut self,
        expr: &TypedExpr,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if expr.kind != ValueKind::Dynamic {
            self.compile_expr_payload(expr, function)?;
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(expr.kind.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            return Ok(());
        }

        match &expr.expr {
            ExprIr::This => {
                self.compile_this_to_locals(payload_local, tag_local, function)?;
            }
            ExprIr::Arguments => {
                let storage = self.lookup_binding(LEXICAL_ARGUMENTS_NAME).ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: missing `arguments` binding",
                    )
                })?;
                self.read_binding_to_locals(storage, payload_local, tag_local, function);
            }
            ExprIr::Identifier(name) => {
                let storage = self.lookup_binding(name).ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: unbound identifier `{name}`"
                    ))
                })?;
                self.read_binding_to_locals(storage, payload_local, tag_local, function);
            }
            ExprIr::AssignIdentifier { name, value } => {
                let storage = self.lookup_binding(name).ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: unbound identifier `{name}`"
                    ))
                })?;
                self.compile_expr_to_binding(value, storage, function)?;
                self.read_binding_to_locals(storage, payload_local, tag_local, function);
            }
            ExprIr::PropertyRead { target, key } => {
                self.compile_property_read_to_locals(
                    target,
                    key,
                    payload_local,
                    tag_local,
                    function,
                )?;
            }
            ExprIr::PropertyWrite { target, key, value } => {
                self.compile_property_write_to_locals(
                    target,
                    key,
                    value,
                    payload_local,
                    tag_local,
                    function,
                )?;
            }
            ExprIr::LogicalShortCircuit { op, lhs, rhs } => {
                self.compile_expr_to_locals(lhs, payload_local, tag_local, function)?;
                self.compile_truthy_tagged_i32(tag_local, payload_local, function)?;
                function.instruction(&Instruction::If(BlockType::Empty));
                match op {
                    LogicalBinaryOp::And => {
                        self.compile_expr_to_locals(rhs, payload_local, tag_local, function)?;
                    }
                    LogicalBinaryOp::Or => {}
                }
                function.instruction(&Instruction::Else);
                match op {
                    LogicalBinaryOp::And => {}
                    LogicalBinaryOp::Or => {
                        self.compile_expr_to_locals(rhs, payload_local, tag_local, function)?;
                    }
                }
                function.instruction(&Instruction::End);
            }
            ExprIr::CallNamed { name, args } => {
                self.emit_call(name, args, function)?;
                function.instruction(&Instruction::LocalSet(tag_local));
                function.instruction(&Instruction::LocalSet(payload_local));
            }
            ExprIr::CallIndirect { callee, args } => {
                self.emit_indirect_call(callee, None, args, function)?;
                function.instruction(&Instruction::LocalSet(tag_local));
                function.instruction(&Instruction::LocalSet(payload_local));
            }
            ExprIr::CallMethod { receiver, key, args } => {
                self.emit_method_call(receiver, key, args, function)?;
                function.instruction(&Instruction::LocalSet(tag_local));
                function.instruction(&Instruction::LocalSet(payload_local));
            }
            _ => {
                return Err(EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: dynamic expression form",
                ));
            }
        }
        Ok(())
    }

    fn compile_object_literal_payload(
        &mut self,
        properties: &[ObjectPropertyIr],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let capacity = (properties.len() as u64).max(MIN_HEAP_CAPACITY);
        self.emit_heap_alloc_const(HEAP_HEADER_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(object_local));
        self.emit_heap_alloc_const(capacity * HEAP_OBJECT_ENTRY_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(buffer_local));
        self.store_i64_local_at_offset(object_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.store_i64_const_at_offset(object_local, HEAP_LEN_OFFSET, 0, function);
        self.store_i64_const_at_offset(object_local, HEAP_CAP_OFFSET, capacity, function);

        for property in properties {
            let key_local = self.reserve_temp_local();
            let key = match property {
                ObjectPropertyIr::Data { key, .. }
                | ObjectPropertyIr::Method { key, .. }
                | ObjectPropertyIr::Getter { key, .. }
                | ObjectPropertyIr::Setter { key, .. } => key,
            };
            function.instruction(&Instruction::I64Const(self.strings.payload(key)));
            function.instruction(&Instruction::LocalSet(key_local));
            match property {
                ObjectPropertyIr::Data { value, .. } | ObjectPropertyIr::Method { function: value, .. } => {
                    let value_payload = self.reserve_temp_local();
                    let value_tag = self.reserve_temp_local();
                    self.compile_expr_to_locals(value, value_payload, value_tag, function)?;
                    self.emit_object_define_data(
                        object_local,
                        key_local,
                        value_payload,
                        value_tag,
                        function,
                    )?;
                    self.release_temp_local(value_tag);
                    self.release_temp_local(value_payload);
                }
                ObjectPropertyIr::Getter { function: getter, .. } => {
                    let getter_payload = self.reserve_temp_local();
                    let getter_tag = self.reserve_temp_local();
                    self.compile_expr_to_locals(getter, getter_payload, getter_tag, function)?;
                    self.emit_object_define_accessor(
                        object_local,
                        key_local,
                        Some((getter_payload, getter_tag)),
                        None,
                        function,
                    )?;
                    self.release_temp_local(getter_tag);
                    self.release_temp_local(getter_payload);
                }
                ObjectPropertyIr::Setter { function: setter, .. } => {
                    let setter_payload = self.reserve_temp_local();
                    let setter_tag = self.reserve_temp_local();
                    self.compile_expr_to_locals(setter, setter_payload, setter_tag, function)?;
                    self.emit_object_define_accessor(
                        object_local,
                        key_local,
                        None,
                        Some((setter_payload, setter_tag)),
                        function,
                    )?;
                    self.release_temp_local(setter_tag);
                    self.release_temp_local(setter_payload);
                }
            }
            self.release_temp_local(key_local);
        }

        function.instruction(&Instruction::LocalGet(object_local));
        self.release_temp_local(buffer_local);
        self.release_temp_local(object_local);
        Ok(())
    }

    fn compile_array_literal_payload(
        &mut self,
        elements: &[TypedExpr],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let array_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let capacity = (elements.len() as u64).max(MIN_HEAP_CAPACITY);
        self.emit_heap_alloc_const(HEAP_HEADER_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(array_local));
        self.emit_heap_alloc_const(capacity * HEAP_ARRAY_ENTRY_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(buffer_local));
        self.store_i64_local_at_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.store_i64_const_at_offset(
            array_local,
            HEAP_LEN_OFFSET,
            elements.len() as u64,
            function,
        );
        self.store_i64_const_at_offset(array_local, HEAP_CAP_OFFSET, capacity, function);

        let entry_local = self.reserve_temp_local();
        for (index, element) in elements.iter().enumerate() {
            let value_payload = self.reserve_temp_local();
            let value_tag = self.reserve_temp_local();
            self.compile_expr_to_locals(element, value_payload, value_tag, function)?;
            function.instruction(&Instruction::LocalGet(buffer_local));
            function.instruction(&Instruction::I64Const(
                (index as u64 * HEAP_ARRAY_ENTRY_SIZE) as i64,
            ));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(entry_local));
            self.store_i64_local_at_offset(entry_local, HEAP_ARRAY_TAG_OFFSET, value_tag, function);
            self.store_i64_local_at_offset(
                entry_local,
                HEAP_ARRAY_PAYLOAD_OFFSET,
                value_payload,
                function,
            );
            self.release_temp_local(value_tag);
            self.release_temp_local(value_payload);
        }
        self.release_temp_local(entry_local);

        function.instruction(&Instruction::LocalGet(array_local));
        self.release_temp_local(buffer_local);
        self.release_temp_local(array_local);
        Ok(())
    }

    fn compile_property_read_to_locals(
        &mut self,
        target: &TypedExpr,
        key: &PropertyKeyIr,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let target_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        self.compile_expr_to_locals(target, target_local, target_tag_local, function)?;

        match target.kind {
            ValueKind::Object => {
                let key_local = self.compile_object_key_to_local(key, function)?;
                self.emit_object_read(target_local, key_local, payload_local, tag_local, function)?;
                self.release_temp_local(key_local);
            }
            ValueKind::Array => {
                match key {
                    PropertyKeyIr::ArrayLength => {
                        self.emit_array_length(target_local, payload_local, tag_local, function);
                    }
                    _ => {
                        let index_local = self.compile_array_index_to_local(key, function)?;
                        self.emit_array_read(
                            target_local,
                            index_local,
                            payload_local,
                            tag_local,
                            function,
                        );
                        self.release_temp_local(index_local);
                    }
                }
            }
            ValueKind::Arguments => match key {
                PropertyKeyIr::ArrayLength => {
                    self.emit_arguments_length(target_local, payload_local, tag_local, function);
                }
                _ => {
                    let index_local = self.compile_array_index_to_local(key, function)?;
                    self.emit_arguments_read(
                        target_local,
                        index_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    self.release_temp_local(index_local);
                }
            },
            _ => {
                self.release_temp_local(target_tag_local);
                self.release_temp_local(target_local);
                return Err(EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: property access on non-object target",
                ));
            }
        }

        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_local);
        Ok(())
    }

    fn compile_property_write_payload(
        &mut self,
        target: &TypedExpr,
        key: &PropertyKeyIr,
        value: &TypedExpr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        self.compile_property_write_to_locals(
            target,
            key,
            value,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(payload_local));
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        Ok(())
    }

    fn compile_property_write_to_locals(
        &mut self,
        target: &TypedExpr,
        key: &PropertyKeyIr,
        value: &TypedExpr,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let target_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        self.compile_expr_to_locals(target, target_local, target_tag_local, function)?;

        match target.kind {
            ValueKind::Object => {
                let key_local = self.compile_object_key_to_local(key, function)?;
                self.compile_expr_to_locals(value, payload_local, tag_local, function)?;
                self.emit_object_write(target_local, key_local, payload_local, tag_local, function)?;
                self.release_temp_local(key_local);
            }
            ValueKind::Array => {
                let index_local = self.compile_array_index_to_local(key, function)?;
                self.compile_expr_to_locals(value, payload_local, tag_local, function)?;
                self.emit_array_write(
                    target_local,
                    index_local,
                    payload_local,
                    tag_local,
                    function,
                )?;
                self.release_temp_local(index_local);
            }
            ValueKind::Arguments => {
                let index_local = self.compile_array_index_to_local(key, function)?;
                self.compile_expr_to_locals(value, payload_local, tag_local, function)?;
                self.emit_arguments_write(
                    target_local,
                    index_local,
                    payload_local,
                    tag_local,
                    function,
                )?;
                self.release_temp_local(index_local);
            }
            _ => {
                self.release_temp_local(target_tag_local);
                self.release_temp_local(target_local);
                return Err(EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: property access on non-object target",
                ));
            }
        }

        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_local);
        Ok(())
    }

    fn compile_object_key_to_local(
        &mut self,
        key: &PropertyKeyIr,
        function: &mut Function,
    ) -> Result<u32, EmitError> {
        let key_local = self.reserve_temp_local();
        match key {
            PropertyKeyIr::StaticString(value) => {
                function.instruction(&Instruction::I64Const(self.strings.payload(value)));
            }
            PropertyKeyIr::ArrayLength => {
                self.release_temp_local(key_local);
                return Err(EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: object key kind",
                ));
            }
            PropertyKeyIr::StringExpr(expr) => {
                self.compile_expr_payload(expr, function)?;
            }
            PropertyKeyIr::ArrayIndex(_) => {
                self.release_temp_local(key_local);
                return Err(EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: object key kind",
                ));
            }
        }
        function.instruction(&Instruction::LocalSet(key_local));
        Ok(key_local)
    }

    fn compile_array_index_to_local(
        &mut self,
        key: &PropertyKeyIr,
        function: &mut Function,
    ) -> Result<u32, EmitError> {
        let index_local = self.reserve_temp_local();
        let PropertyKeyIr::ArrayIndex(expr) = key else {
            self.release_temp_local(index_local);
            return Err(EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: array index kind",
            ));
        };
        self.compile_expr_payload(expr, function)?;
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(index_local));
        Ok(index_local)
    }

    fn emit_heap_alloc_const(&mut self, size: u64, function: &mut Function) -> Result<(), EmitError> {
        let size_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(Self::align_heap_size(size) as i64));
        function.instruction(&Instruction::LocalSet(size_local));
        self.emit_heap_alloc_from_local(size_local, function)?;
        self.release_temp_local(size_local);
        Ok(())
    }

    fn emit_heap_alloc_from_local(
        &mut self,
        size_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if !self.uses_heap {
            return Err(EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: heap value without memory",
            ));
        }
        let alloc_local = self.reserve_temp_local();
        let end_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(HEAP_PTR_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(alloc_local));
        function.instruction(&Instruction::LocalGet(alloc_local));
        function.instruction(&Instruction::LocalGet(size_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(end_local));

        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::MemorySize(0));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(WASM_PAGE_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::MemorySize(0));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(WASM_PAGE_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const((WASM_PAGE_SIZE - 1) as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(WASM_PAGE_SIZE as i64));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::MemoryGrow(0));
        function.instruction(&Instruction::Drop);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::GlobalSet(HEAP_PTR_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalGet(alloc_local));
        self.release_temp_local(end_local);
        self.release_temp_local(alloc_local);
        Ok(())
    }

    const fn align_heap_size(size: u64) -> u64 {
        (size + 7) & !7
    }

    fn emit_object_read(
        &mut self,
        object_local: u32,
        key_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let getter_payload_local = self.reserve_temp_local();
        let getter_tag_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(object_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(object_local, HEAP_LEN_OFFSET, len_local, function);
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));

        self.load_i64_from_offset(entry_local, HEAP_OBJECT_KEY_OFFSET, function);
        function.instruction(&Instruction::LocalGet(key_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_DATA as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(entry_local, HEAP_OBJECT_DATA_TAG_OFFSET, tag_local, function);
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_GETTER_TAG_OFFSET,
            getter_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_GETTER_PAYLOAD_OFFSET,
            getter_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(getter_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_handle_call(
            getter_payload_local,
            getter_tag_local,
            Some((object_local, None)),
            &[],
            function,
        )?;
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(getter_tag_local);
        self.release_temp_local(getter_payload_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    fn emit_object_grow_buffer(
        &mut self,
        object_local: u32,
        buffer_local: u32,
        len_local: u32,
        cap_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let new_cap_local = self.reserve_temp_local();
        let size_local = self.reserve_temp_local();
        let new_buffer_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let old_entry_local = self.reserve_temp_local();
        let new_entry_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(MIN_HEAP_CAPACITY as i64));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(new_cap_local));

        function.instruction(&Instruction::LocalGet(new_cap_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(size_local));
        self.emit_heap_alloc_from_local(size_local, function)?;
        function.instruction(&Instruction::LocalSet(new_buffer_local));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(old_entry_local));

        function.instruction(&Instruction::LocalGet(new_buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(new_entry_local));

        for offset in [
            HEAP_OBJECT_KEY_OFFSET,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            HEAP_OBJECT_DATA_TAG_OFFSET,
            HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
            HEAP_OBJECT_GETTER_TAG_OFFSET,
            HEAP_OBJECT_GETTER_PAYLOAD_OFFSET,
            HEAP_OBJECT_SETTER_TAG_OFFSET,
            HEAP_OBJECT_SETTER_PAYLOAD_OFFSET,
        ] {
            self.load_i64_from_offset(old_entry_local, offset, function);
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.store_i64_local_at_offset(new_entry_local, offset, self.scratch_local, function);
        }

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(new_buffer_local));
        function.instruction(&Instruction::LocalSet(buffer_local));
        function.instruction(&Instruction::LocalGet(new_cap_local));
        function.instruction(&Instruction::LocalSet(cap_local));
        self.store_i64_local_at_offset(object_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.store_i64_local_at_offset(object_local, HEAP_CAP_OFFSET, cap_local, function);

        self.release_temp_local(new_entry_local);
        self.release_temp_local(old_entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(new_buffer_local);
        self.release_temp_local(size_local);
        self.release_temp_local(new_cap_local);
        Ok(())
    }

    fn emit_object_define_data(
        &mut self,
        object_local: u32,
        key_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_object_define_entry(
            object_local,
            key_local,
            Some((payload_local, tag_local)),
            None,
            None,
            function,
        )
    }

    fn emit_object_define_accessor(
        &mut self,
        object_local: u32,
        key_local: u32,
        getter: Option<(u32, u32)>,
        setter: Option<(u32, u32)>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_object_define_entry(object_local, key_local, None, getter, setter, function)
    }

    fn emit_object_define_entry(
        &mut self,
        object_local: u32,
        key_local: u32,
        data: Option<(u32, u32)>,
        getter: Option<(u32, u32)>,
        setter: Option<(u32, u32)>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let cap_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let existing_descriptor_kind_local = self.reserve_temp_local();
        let getter_tag_local = self.reserve_temp_local();
        let getter_payload_local = self.reserve_temp_local();
        let setter_tag_local = self.reserve_temp_local();
        let setter_payload_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(object_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(object_local, HEAP_LEN_OFFSET, len_local, function);
        self.load_i64_to_local_from_offset(object_local, HEAP_CAP_OFFSET, cap_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));

        let has_data = data.is_some();
        let has_getter = getter.is_some();
        let has_setter = setter.is_some();
        let descriptor_kind = if has_data {
            OBJECT_DESCRIPTOR_DATA
        } else {
            OBJECT_DESCRIPTOR_ACCESSOR
        };
        function.instruction(&Instruction::I64Const(descriptor_kind as i64));
        function.instruction(&Instruction::LocalSet(descriptor_kind_local));
        if let Some((data_payload_local, data_tag_local)) = data {
            function.instruction(&Instruction::LocalGet(data_tag_local));
            function.instruction(&Instruction::LocalSet(self.result_tag_local));
            function.instruction(&Instruction::LocalGet(data_payload_local));
            function.instruction(&Instruction::LocalSet(self.scratch_local));
        } else {
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(self.result_tag_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(self.scratch_local));
        }
        if let Some((getter_payload, getter_tag)) = getter {
            function.instruction(&Instruction::LocalGet(getter_tag));
            function.instruction(&Instruction::LocalSet(getter_tag_local));
            function.instruction(&Instruction::LocalGet(getter_payload));
            function.instruction(&Instruction::LocalSet(getter_payload_local));
        } else {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(getter_tag_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(getter_payload_local));
        }
        if let Some((setter_payload, setter_tag)) = setter {
            function.instruction(&Instruction::LocalGet(setter_tag));
            function.instruction(&Instruction::LocalSet(setter_tag_local));
            function.instruction(&Instruction::LocalGet(setter_payload));
            function.instruction(&Instruction::LocalSet(setter_payload_local));
        } else {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(setter_tag_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(setter_payload_local));
        }

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));

        self.load_i64_from_offset(entry_local, HEAP_OBJECT_KEY_OFFSET, function);
        function.instruction(&Instruction::LocalGet(key_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        if has_data {
            self.store_i64_local_at_offset(
                entry_local,
                HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
                descriptor_kind_local,
                function,
            );
            self.store_i64_local_at_offset(
                entry_local,
                HEAP_OBJECT_DATA_TAG_OFFSET,
                self.result_tag_local,
                function,
            );
            self.store_i64_local_at_offset(
                entry_local,
                HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
                self.scratch_local,
                function,
            );
            self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_GETTER_TAG_OFFSET, 0, function);
            self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_GETTER_PAYLOAD_OFFSET, 0, function);
            self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_SETTER_TAG_OFFSET, 0, function);
            self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_SETTER_PAYLOAD_OFFSET, 0, function);
        } else {
            self.load_i64_to_local_from_offset(
                entry_local,
                HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
                existing_descriptor_kind_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
            function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            if !has_getter {
                self.load_i64_to_local_from_offset(
                    entry_local,
                    HEAP_OBJECT_GETTER_TAG_OFFSET,
                    getter_tag_local,
                    function,
                );
                self.load_i64_to_local_from_offset(
                    entry_local,
                    HEAP_OBJECT_GETTER_PAYLOAD_OFFSET,
                    getter_payload_local,
                    function,
                );
            }
            if !has_setter {
                self.load_i64_to_local_from_offset(
                    entry_local,
                    HEAP_OBJECT_SETTER_TAG_OFFSET,
                    setter_tag_local,
                    function,
                );
                self.load_i64_to_local_from_offset(
                    entry_local,
                    HEAP_OBJECT_SETTER_PAYLOAD_OFFSET,
                    setter_payload_local,
                    function,
                );
            }
            function.instruction(&Instruction::End);
            self.store_i64_local_at_offset(
                entry_local,
                HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
                descriptor_kind_local,
                function,
            );
            self.store_i64_local_at_offset(
                entry_local,
                HEAP_OBJECT_GETTER_TAG_OFFSET,
                getter_tag_local,
                function,
            );
            self.store_i64_local_at_offset(
                entry_local,
                HEAP_OBJECT_GETTER_PAYLOAD_OFFSET,
                getter_payload_local,
                function,
            );
            self.store_i64_local_at_offset(
                entry_local,
                HEAP_OBJECT_SETTER_TAG_OFFSET,
                setter_tag_local,
                function,
            );
            self.store_i64_local_at_offset(
                entry_local,
                HEAP_OBJECT_SETTER_PAYLOAD_OFFSET,
                setter_payload_local,
                function,
            );
            self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_DATA_TAG_OFFSET, 0, function);
            self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_DATA_PAYLOAD_OFFSET, 0, function);
        }
        function.instruction(&Instruction::Br(3));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_grow_buffer(object_local, buffer_local, len_local, cap_local, function)?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_local_at_offset(entry_local, HEAP_OBJECT_KEY_OFFSET, key_local, function);
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        if has_data {
            self.store_i64_local_at_offset(
                entry_local,
                HEAP_OBJECT_DATA_TAG_OFFSET,
                self.result_tag_local,
                function,
            );
            self.store_i64_local_at_offset(
                entry_local,
                HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
                self.scratch_local,
                function,
            );
            self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_GETTER_TAG_OFFSET, 0, function);
            self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_GETTER_PAYLOAD_OFFSET, 0, function);
            self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_SETTER_TAG_OFFSET, 0, function);
            self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_SETTER_PAYLOAD_OFFSET, 0, function);
        } else {
            self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_DATA_TAG_OFFSET, 0, function);
            self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_DATA_PAYLOAD_OFFSET, 0, function);
            if has_getter {
                self.store_i64_local_at_offset(
                    entry_local,
                    HEAP_OBJECT_GETTER_TAG_OFFSET,
                    getter_tag_local,
                    function,
                );
                self.store_i64_local_at_offset(
                    entry_local,
                    HEAP_OBJECT_GETTER_PAYLOAD_OFFSET,
                    getter_payload_local,
                    function,
                );
            } else {
                self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_GETTER_TAG_OFFSET, 0, function);
                self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_GETTER_PAYLOAD_OFFSET, 0, function);
            }
            if has_setter {
                self.store_i64_local_at_offset(
                    entry_local,
                    HEAP_OBJECT_SETTER_TAG_OFFSET,
                    setter_tag_local,
                    function,
                );
                self.store_i64_local_at_offset(
                    entry_local,
                    HEAP_OBJECT_SETTER_PAYLOAD_OFFSET,
                    setter_payload_local,
                    function,
                );
            } else {
                self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_SETTER_TAG_OFFSET, 0, function);
                self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_SETTER_PAYLOAD_OFFSET, 0, function);
            }
        }
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(len_local));
        self.store_i64_local_at_offset(object_local, HEAP_LEN_OFFSET, len_local, function);
        function.instruction(&Instruction::End);

        self.release_temp_local(setter_payload_local);
        self.release_temp_local(setter_tag_local);
        self.release_temp_local(getter_payload_local);
        self.release_temp_local(getter_tag_local);
        self.release_temp_local(existing_descriptor_kind_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(cap_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    fn emit_object_write(
        &mut self,
        object_local: u32,
        key_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let cap_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let setter_payload_local = self.reserve_temp_local();
        let setter_tag_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(object_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(object_local, HEAP_LEN_OFFSET, len_local, function);
        self.load_i64_to_local_from_offset(object_local, HEAP_CAP_OFFSET, cap_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));

        self.load_i64_from_offset(entry_local, HEAP_OBJECT_KEY_OFFSET, function);
        function.instruction(&Instruction::LocalGet(key_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_DATA as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_local_at_offset(entry_local, HEAP_OBJECT_DATA_TAG_OFFSET, tag_local, function);
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_SETTER_TAG_OFFSET,
            setter_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_SETTER_PAYLOAD_OFFSET,
            setter_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(setter_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_handle_call(
            setter_payload_local,
            setter_tag_local,
            Some((object_local, None)),
            &[(payload_local, tag_local)],
            function,
        )?;
        function.instruction(&Instruction::Drop);
        function.instruction(&Instruction::Drop);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(3));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_grow_buffer(object_local, buffer_local, len_local, cap_local, function)?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_local_at_offset(entry_local, HEAP_OBJECT_KEY_OFFSET, key_local, function);
        self.store_i64_const_at_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            OBJECT_DESCRIPTOR_DATA,
            function,
        );
        self.store_i64_local_at_offset(entry_local, HEAP_OBJECT_DATA_TAG_OFFSET, tag_local, function);
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_GETTER_TAG_OFFSET, 0, function);
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_GETTER_PAYLOAD_OFFSET, 0, function);
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_SETTER_TAG_OFFSET, 0, function);
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_SETTER_PAYLOAD_OFFSET, 0, function);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(len_local));
        self.store_i64_local_at_offset(object_local, HEAP_LEN_OFFSET, len_local, function);
        function.instruction(&Instruction::End);

        self.release_temp_local(setter_tag_local);
        self.release_temp_local(setter_payload_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(cap_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    fn emit_array_read(
        &mut self,
        array_local: u32,
        index_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(array_local, HEAP_LEN_OFFSET, len_local, function);
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(0));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(entry_local, HEAP_ARRAY_TAG_OFFSET, tag_local, function);
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        function.instruction(&Instruction::End);

        self.release_temp_local(entry_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
    }

    fn emit_array_length(
        &mut self,
        array_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        self.load_i64_to_local_from_offset(array_local, HEAP_LEN_OFFSET, payload_local, function);
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
    }

    fn emit_array_grow_buffer(
        &mut self,
        array_local: u32,
        buffer_local: u32,
        len_local: u32,
        cap_local: u32,
        required_index_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let new_cap_local = self.reserve_temp_local();
        let required_len_local = self.reserve_temp_local();
        let size_local = self.reserve_temp_local();
        let new_buffer_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let old_entry_local = self.reserve_temp_local();
        let new_entry_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(MIN_HEAP_CAPACITY as i64));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(new_cap_local));

        function.instruction(&Instruction::LocalGet(required_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(required_len_local));
        function.instruction(&Instruction::LocalGet(required_len_local));
        function.instruction(&Instruction::LocalGet(new_cap_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(required_len_local));
        function.instruction(&Instruction::LocalSet(new_cap_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(new_cap_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(size_local));
        self.emit_heap_alloc_from_local(size_local, function)?;
        function.instruction(&Instruction::LocalSet(new_buffer_local));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(old_entry_local));

        function.instruction(&Instruction::LocalGet(new_buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(new_entry_local));

        for offset in [HEAP_ARRAY_TAG_OFFSET, HEAP_ARRAY_PAYLOAD_OFFSET] {
            self.load_i64_from_offset(old_entry_local, offset, function);
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.store_i64_local_at_offset(new_entry_local, offset, self.scratch_local, function);
        }

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(new_buffer_local));
        function.instruction(&Instruction::LocalSet(buffer_local));
        function.instruction(&Instruction::LocalGet(new_cap_local));
        function.instruction(&Instruction::LocalSet(cap_local));
        self.store_i64_local_at_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.store_i64_local_at_offset(array_local, HEAP_CAP_OFFSET, cap_local, function);

        self.release_temp_local(new_entry_local);
        self.release_temp_local(old_entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(new_buffer_local);
        self.release_temp_local(size_local);
        self.release_temp_local(required_len_local);
        self.release_temp_local(new_cap_local);
        Ok(())
    }

    fn emit_array_write(
        &mut self,
        array_local: u32,
        index_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let cap_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(array_local, HEAP_LEN_OFFSET, len_local, function);
        self.load_i64_to_local_from_offset(array_local, HEAP_CAP_OFFSET, cap_local, function);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_grow_buffer(
            array_local,
            buffer_local,
            len_local,
            cap_local,
            index_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_local_at_offset(entry_local, HEAP_ARRAY_TAG_OFFSET, tag_local, function);
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_ARRAY_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(len_local));
        self.store_i64_local_at_offset(array_local, HEAP_LEN_OFFSET, len_local, function);
        function.instruction(&Instruction::End);

        self.release_temp_local(entry_local);
        self.release_temp_local(cap_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    fn emit_call_args_vector(
        &mut self,
        args: &[TypedExpr],
        function: &mut Function,
    ) -> Result<(u32, u32), EmitError> {
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let capacity = (args.len() as u64).max(MIN_HEAP_CAPACITY);

        function.instruction(&Instruction::I64Const(args.len() as i64));
        function.instruction(&Instruction::LocalSet(argc_local));
        self.emit_heap_alloc_const(HEAP_HEADER_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(argv_local));
        self.emit_heap_alloc_const(capacity * HEAP_ARRAY_ENTRY_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(buffer_local));
        self.store_i64_local_at_offset(argv_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.store_i64_const_at_offset(argv_local, HEAP_LEN_OFFSET, args.len() as u64, function);
        self.store_i64_const_at_offset(argv_local, HEAP_CAP_OFFSET, capacity, function);

        for (index, arg) in args.iter().enumerate() {
            self.compile_expr_to_locals(arg, self.scratch_local, self.result_tag_local, function)?;
            function.instruction(&Instruction::LocalGet(buffer_local));
            function.instruction(&Instruction::I64Const((index as u64 * HEAP_ARRAY_ENTRY_SIZE) as i64));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(entry_local));
            self.store_i64_local_at_offset(entry_local, HEAP_ARRAY_TAG_OFFSET, self.result_tag_local, function);
            self.store_i64_local_at_offset(entry_local, HEAP_ARRAY_PAYLOAD_OFFSET, self.scratch_local, function);
        }

        self.release_temp_local(entry_local);
        self.release_temp_local(buffer_local);
        Ok((argc_local, argv_local))
    }

    fn emit_pre_evaluated_arg_vector(
        &mut self,
        args: &[(u32, u32)],
        argc_local: u32,
        argv_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let capacity = (args.len() as u64).max(MIN_HEAP_CAPACITY);

        function.instruction(&Instruction::I64Const(args.len() as i64));
        function.instruction(&Instruction::LocalSet(argc_local));
        self.emit_heap_alloc_const(HEAP_HEADER_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(argv_local));
        self.emit_heap_alloc_const(capacity * HEAP_ARRAY_ENTRY_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(buffer_local));
        self.store_i64_local_at_offset(argv_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.store_i64_const_at_offset(argv_local, HEAP_LEN_OFFSET, args.len() as u64, function);
        self.store_i64_const_at_offset(argv_local, HEAP_CAP_OFFSET, capacity, function);

        for (index, (arg_payload_local, arg_tag_local)) in args.iter().enumerate() {
            function.instruction(&Instruction::LocalGet(buffer_local));
            function.instruction(&Instruction::I64Const((index as u64 * HEAP_ARRAY_ENTRY_SIZE) as i64));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(entry_local));
            self.store_i64_local_at_offset(entry_local, HEAP_ARRAY_TAG_OFFSET, *arg_tag_local, function);
            self.store_i64_local_at_offset(entry_local, HEAP_ARRAY_PAYLOAD_OFFSET, *arg_payload_local, function);
        }

        self.release_temp_local(entry_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    fn emit_rest_array_payload(
        &mut self,
        start_index: usize,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let rest_len_local = self.reserve_temp_local();
        let array_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let size_local = self.reserve_temp_local();
        let src_buffer_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let src_entry_local = self.reserve_temp_local();
        let dst_entry_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Const(start_index as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Const(start_index as i64));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(rest_len_local));

        self.emit_heap_alloc_const(HEAP_HEADER_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(array_local));
        function.instruction(&Instruction::LocalGet(rest_len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(MIN_HEAP_CAPACITY as i64));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(rest_len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        function.instruction(&Instruction::LocalGet(self.scratch_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(size_local));
        self.emit_heap_alloc_from_local(size_local, function)?;
        function.instruction(&Instruction::LocalSet(buffer_local));
        self.store_i64_local_at_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.store_i64_local_at_offset(array_local, HEAP_LEN_OFFSET, rest_len_local, function);
        self.store_i64_local_at_offset(array_local, HEAP_CAP_OFFSET, self.scratch_local, function);

        self.load_i64_to_local_from_offset(self.argv_param_local(), HEAP_PTR_OFFSET, src_buffer_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(rest_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(src_buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(start_index as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(src_entry_local));

        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(dst_entry_local));

        for offset in [HEAP_ARRAY_TAG_OFFSET, HEAP_ARRAY_PAYLOAD_OFFSET] {
            self.load_i64_from_offset(src_entry_local, offset, function);
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.store_i64_local_at_offset(dst_entry_local, offset, self.scratch_local, function);
        }

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(array_local));
        self.release_temp_local(dst_entry_local);
        self.release_temp_local(src_entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(src_buffer_local);
        self.release_temp_local(size_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(array_local);
        self.release_temp_local(rest_len_local);
        Ok(())
    }

    fn emit_arguments_object_payload(&mut self, function: &mut Function) -> Result<(), EmitError> {
        let arguments_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let size_local = self.reserve_temp_local();
        let src_buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let mapped_count_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let src_entry_local = self.reserve_temp_local();
        let dst_entry_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::LocalSet(len_local));
        if self.uses_mapped_arguments_object() {
            function.instruction(&Instruction::LocalGet(len_local));
            function.instruction(&Instruction::I64Const(self.params.len() as i64));
            function.instruction(&Instruction::I64GtU);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            function.instruction(&Instruction::I64Const(self.params.len() as i64));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(len_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalSet(mapped_count_local));
        } else {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(mapped_count_local));
        }

        self.emit_heap_alloc_const(HEAP_ARGUMENTS_ENV_HANDLE_OFFSET + 8, function)?;
        function.instruction(&Instruction::LocalSet(arguments_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(MIN_HEAP_CAPACITY as i64));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        function.instruction(&Instruction::LocalGet(self.scratch_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(size_local));
        self.emit_heap_alloc_from_local(size_local, function)?;
        function.instruction(&Instruction::LocalSet(buffer_local));
        self.store_i64_local_at_offset(arguments_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.store_i64_local_at_offset(arguments_local, HEAP_LEN_OFFSET, len_local, function);
        self.store_i64_local_at_offset(arguments_local, HEAP_CAP_OFFSET, self.scratch_local, function);
        self.store_i64_local_at_offset(arguments_local, HEAP_ARGUMENTS_MAPPED_COUNT_OFFSET, mapped_count_local, function);
        if self.uses_mapped_arguments_object() {
            self.store_i64_local_at_offset(arguments_local, HEAP_ARGUMENTS_ENV_HANDLE_OFFSET, self.current_env_local, function);
        } else {
            self.store_i64_const_at_offset(arguments_local, HEAP_ARGUMENTS_ENV_HANDLE_OFFSET, 0, function);
        }

        self.load_i64_to_local_from_offset(self.argv_param_local(), HEAP_PTR_OFFSET, src_buffer_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(src_buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(src_entry_local));

        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(dst_entry_local));

        for offset in [HEAP_ARRAY_TAG_OFFSET, HEAP_ARRAY_PAYLOAD_OFFSET] {
            self.load_i64_from_offset(src_entry_local, offset, function);
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.store_i64_local_at_offset(dst_entry_local, offset, self.scratch_local, function);
        }

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(arguments_local));
        self.release_temp_local(dst_entry_local);
        self.release_temp_local(src_entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(mapped_count_local);
        self.release_temp_local(len_local);
        self.release_temp_local(src_buffer_local);
        self.release_temp_local(size_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(arguments_local);
        Ok(())
    }

    fn emit_arguments_length(
        &mut self,
        arguments_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        self.load_i64_to_local_from_offset(arguments_local, HEAP_LEN_OFFSET, payload_local, function);
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
    }

    fn emit_arguments_read(
        &mut self,
        arguments_local: u32,
        index_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let mapped_count_local = self.reserve_temp_local();
        let env_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(arguments_local, HEAP_ARGUMENTS_MAPPED_COUNT_OFFSET, mapped_count_local, function);
        self.load_i64_to_local_from_offset(arguments_local, HEAP_ARGUMENTS_ENV_HANDLE_OFFSET, env_local, function);
        self.load_i64_to_local_from_offset(arguments_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(arguments_local, HEAP_LEN_OFFSET, len_local, function);
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(mapped_count_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(env_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(ENV_SLOT_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Add);
        function.instruction(&Instruction::I64Load(Self::memarg64(ENV_SLOT_BASE_OFFSET + ENV_SLOT_PAYLOAD_OFFSET)));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(env_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(ENV_SLOT_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Add);
        function.instruction(&Instruction::I64Load(Self::memarg64(ENV_SLOT_BASE_OFFSET + ENV_SLOT_TAG_OFFSET)));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(entry_local, HEAP_ARRAY_PAYLOAD_OFFSET, payload_local, function);
        self.load_i64_to_local_from_offset(entry_local, HEAP_ARRAY_TAG_OFFSET, tag_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(entry_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(env_local);
        self.release_temp_local(mapped_count_local);
        Ok(())
    }

    fn emit_arguments_write(
        &mut self,
        arguments_local: u32,
        index_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let mapped_count_local = self.reserve_temp_local();
        let env_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let cap_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(arguments_local, HEAP_ARGUMENTS_MAPPED_COUNT_OFFSET, mapped_count_local, function);
        self.load_i64_to_local_from_offset(arguments_local, HEAP_ARGUMENTS_ENV_HANDLE_OFFSET, env_local, function);
        self.load_i64_to_local_from_offset(arguments_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(arguments_local, HEAP_LEN_OFFSET, len_local, function);
        self.load_i64_to_local_from_offset(arguments_local, HEAP_CAP_OFFSET, cap_local, function);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(mapped_count_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(env_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(ENV_SLOT_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Add);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Store(Self::memarg64(ENV_SLOT_BASE_OFFSET + ENV_SLOT_TAG_OFFSET)));
        function.instruction(&Instruction::LocalGet(env_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(ENV_SLOT_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Add);
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::I64Store(Self::memarg64(ENV_SLOT_BASE_OFFSET + ENV_SLOT_PAYLOAD_OFFSET)));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_grow_buffer(arguments_local, buffer_local, len_local, cap_local, index_local, function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_local_at_offset(entry_local, HEAP_ARRAY_TAG_OFFSET, tag_local, function);
        self.store_i64_local_at_offset(entry_local, HEAP_ARRAY_PAYLOAD_OFFSET, payload_local, function);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(len_local));
        self.store_i64_local_at_offset(arguments_local, HEAP_LEN_OFFSET, len_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(entry_local);
        self.release_temp_local(cap_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(env_local);
        self.release_temp_local(mapped_count_local);
        Ok(())
    }

    fn load_i64_to_local_from_offset(
        &self,
        base_local: u32,
        offset: u64,
        dest_local: u32,
        function: &mut Function,
    ) {
        self.load_i64_from_offset(base_local, offset, function);
        function.instruction(&Instruction::LocalSet(dest_local));
    }

    fn load_i64_from_offset(&self, base_local: u32, offset: u64, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(base_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(Self::memarg64(offset)));
    }

    fn store_i64_const_at_offset(
        &self,
        base_local: u32,
        offset: u64,
        value: u64,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(base_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Const(value as i64));
        function.instruction(&Instruction::I64Store(Self::memarg64(offset)));
    }

    fn store_i64_local_at_offset(
        &self,
        base_local: u32,
        offset: u64,
        value_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(base_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::I64Store(Self::memarg64(offset)));
    }

    fn emit_function_value_payload(&self, table_index: u32, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Const(table_index as i64));
        function.instruction(&Instruction::I64Or);
    }

    fn emit_function_handle_call(
        &mut self,
        callee_payload_local: u32,
        callee_tag_local: u32,
        this_locals: Option<(u32, Option<u32>)>,
        args: &[(u32, u32)],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let callee_env_local = self.reserve_temp_local();
        let table_index_local = self.reserve_temp_local();
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(callee_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(callee_payload_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(callee_env_local));
        function.instruction(&Instruction::LocalGet(callee_payload_local));
        function.instruction(&Instruction::I64Const(0xFFFF_FFFFu64 as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(table_index_local));

        function.instruction(&Instruction::LocalGet(callee_env_local));
        if let Some((this_payload_local, this_tag_local)) = this_locals {
            function.instruction(&Instruction::LocalGet(this_payload_local));
            if let Some(this_tag_local) = this_tag_local {
                function.instruction(&Instruction::LocalGet(this_tag_local));
            } else {
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
            }
        } else {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        }
        self.emit_pre_evaluated_arg_vector(args, argc_local, argv_local, function)?;
        function.instruction(&Instruction::LocalGet(argc_local));
        function.instruction(&Instruction::LocalGet(argv_local));

        function.instruction(&Instruction::LocalGet(table_index_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::CallIndirect {
            type_index: JS_FUNCTION_TYPE_INDEX,
            table_index: 0,
        });

        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(table_index_local);
        self.release_temp_local(callee_env_local);
        Ok(())
    }

    fn env_slot_offset(slot: u32, field_offset: u64) -> u64 {
        ENV_SLOT_BASE_OFFSET + slot as u64 * ENV_SLOT_SIZE + field_offset
    }

    fn resolve_env_handle_local(
        &mut self,
        hops: u32,
        function: &mut Function,
    ) -> u32 {
        let env_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::LocalSet(env_local));
        for _ in 0..hops {
            self.load_i64_to_local_from_offset(env_local, ENV_PARENT_OFFSET, env_local, function);
        }
        env_local
    }

    fn read_env_slot_to_locals(
        &mut self,
        slot: u32,
        hops: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        let env_local = self.resolve_env_handle_local(hops, function);
        self.load_i64_to_local_from_offset(
            env_local,
            Self::env_slot_offset(slot, ENV_SLOT_PAYLOAD_OFFSET),
            payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            env_local,
            Self::env_slot_offset(slot, ENV_SLOT_TAG_OFFSET),
            tag_local,
            function,
        );
        self.release_temp_local(env_local);
    }

    fn write_env_slot_from_locals(
        &mut self,
        slot: u32,
        hops: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        let env_local = self.resolve_env_handle_local(hops, function);
        self.store_i64_local_at_offset(
            env_local,
            Self::env_slot_offset(slot, ENV_SLOT_TAG_OFFSET),
            tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            env_local,
            Self::env_slot_offset(slot, ENV_SLOT_PAYLOAD_OFFSET),
            payload_local,
            function,
        );
        self.release_temp_local(env_local);
    }

    fn initialize_binding_undefined(
        &mut self,
        storage: BindingStorage,
        function: &mut Function,
    ) {
        match storage {
            BindingStorage::Fixed { payload_local, .. } => {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(payload_local));
            }
            BindingStorage::Dynamic {
                tag_local,
                payload_local,
            } => {
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(payload_local));
            }
            BindingStorage::EnvSlot { slot, hops } => {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(self.scratch_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                self.write_env_slot_from_locals(
                    slot,
                    hops,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                );
            }
        }
    }

    fn write_binding_from_locals(
        &mut self,
        storage: BindingStorage,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        match storage {
            BindingStorage::Fixed { payload_local: binding_payload_local, .. } => {
                function.instruction(&Instruction::LocalGet(payload_local));
                function.instruction(&Instruction::LocalSet(binding_payload_local));
            }
            BindingStorage::Dynamic {
                tag_local: binding_tag_local,
                payload_local: binding_payload_local,
            } => {
                function.instruction(&Instruction::LocalGet(payload_local));
                function.instruction(&Instruction::LocalSet(binding_payload_local));
                function.instruction(&Instruction::LocalGet(tag_local));
                function.instruction(&Instruction::LocalSet(binding_tag_local));
            }
            BindingStorage::EnvSlot { slot, hops } => {
                self.write_env_slot_from_locals(slot, hops, payload_local, tag_local, function);
            }
        }
    }

    const fn memarg64(offset: u64) -> MemArg {
        MemArg {
            offset,
            align: 3,
            memory_index: 0,
        }
    }

    fn emit_call(
        &mut self,
        name: &str,
        args: &[TypedExpr],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let meta = self.functions.values().find(|meta| meta.name == name).ok_or_else(|| {
            EmitError::unsupported(format!(
                "unsupported in porffor wasm-aot first slice: direct call to unknown top-level function `{name}`"
            ))
        })?;
        let (argc_local, argv_local) = self.emit_call_args_vector(args, function)?;

        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalGet(argc_local));
        function.instruction(&Instruction::LocalGet(argv_local));

        function.instruction(&Instruction::Call(meta.wasm_index));
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        Ok(())
    }

    fn emit_indirect_call(
        &mut self,
        callee: &TypedExpr,
        this_arg: Option<&TypedExpr>,
        args: &[TypedExpr],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let callee_payload_local = self.reserve_temp_local();
        let callee_tag_local = self.reserve_temp_local();
        let callee_env_local = self.reserve_temp_local();
        let table_index_local = self.reserve_temp_local();
        let (argc_local, argv_local) = self.emit_call_args_vector(args, function)?;
        self.compile_expr_to_locals(callee, callee_payload_local, callee_tag_local, function)?;

        function.instruction(&Instruction::LocalGet(callee_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(callee_payload_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(callee_env_local));
        function.instruction(&Instruction::LocalGet(callee_payload_local));
        function.instruction(&Instruction::I64Const(0xFFFF_FFFFu64 as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(table_index_local));

        function.instruction(&Instruction::LocalGet(callee_env_local));
        if let Some(this_arg) = this_arg {
            self.compile_expr_to_locals(this_arg, self.scratch_local, self.result_tag_local, function)?;
            function.instruction(&Instruction::LocalGet(self.scratch_local));
            function.instruction(&Instruction::LocalGet(self.result_tag_local));
        } else {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        }
        function.instruction(&Instruction::LocalGet(argc_local));
        function.instruction(&Instruction::LocalGet(argv_local));
        function.instruction(&Instruction::LocalGet(table_index_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::CallIndirect {
            type_index: JS_FUNCTION_TYPE_INDEX,
            table_index: 0,
        });

        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(table_index_local);
        self.release_temp_local(callee_env_local);
        self.release_temp_local(callee_tag_local);
        self.release_temp_local(callee_payload_local);
        Ok(())
    }

    fn emit_method_call(
        &mut self,
        receiver: &TypedExpr,
        key: &PropertyKeyIr,
        args: &[TypedExpr],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let callee_payload_local = self.reserve_temp_local();
        let callee_tag_local = self.reserve_temp_local();
        let callee_env_local = self.reserve_temp_local();
        let table_index_local = self.reserve_temp_local();
        let (argc_local, argv_local) = self.emit_call_args_vector(args, function)?;

        self.compile_expr_to_locals(
            receiver,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        match receiver.kind {
            ValueKind::Object => {
                let key_local = self.compile_object_key_to_local(key, function)?;
                self.emit_object_read(
                    receiver_payload_local,
                    key_local,
                    callee_payload_local,
                    callee_tag_local,
                    function,
                )?;
                self.release_temp_local(key_local);
            }
            ValueKind::Array => {
                let index_local = self.compile_array_index_to_local(key, function)?;
                self.emit_array_read(
                    receiver_payload_local,
                    index_local,
                    callee_payload_local,
                    callee_tag_local,
                    function,
                );
                self.release_temp_local(index_local);
            }
            _ => {
                self.release_temp_local(callee_tag_local);
                self.release_temp_local(callee_payload_local);
                self.release_temp_local(receiver_tag_local);
                self.release_temp_local(receiver_payload_local);
                return Err(EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: property access on non-object target",
                ));
            }
        }

        function.instruction(&Instruction::LocalGet(callee_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(callee_payload_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(callee_env_local));
        function.instruction(&Instruction::LocalGet(callee_payload_local));
        function.instruction(&Instruction::I64Const(0xFFFF_FFFFu64 as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(table_index_local));
        function.instruction(&Instruction::LocalGet(callee_env_local));
        function.instruction(&Instruction::LocalGet(receiver_payload_local));
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::LocalGet(argc_local));
        function.instruction(&Instruction::LocalGet(argv_local));
        function.instruction(&Instruction::LocalGet(table_index_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::CallIndirect {
            type_index: JS_FUNCTION_TYPE_INDEX,
            table_index: 0,
        });

        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(table_index_local);
        self.release_temp_local(callee_env_local);
        self.release_temp_local(callee_tag_local);
        self.release_temp_local(callee_payload_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    fn emit_update_delta(&self, op: NumericUpdateOp, function: &mut Function) {
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
        match op {
            NumericUpdateOp::Increment => function.instruction(&Instruction::F64Add),
            NumericUpdateOp::Decrement => function.instruction(&Instruction::F64Sub),
        };
        function.instruction(&Instruction::I64ReinterpretF64);
    }

    fn compile_truthy_i32(
        &mut self,
        expr: &TypedExpr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if expr.kind == ValueKind::Dynamic {
            self.compile_expr_to_locals(expr, self.scratch_local, self.result_tag_local, function)?;
            self.compile_truthy_tagged_i32(self.result_tag_local, self.scratch_local, function)
        } else {
            self.compile_expr_payload(expr, function)?;
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.compile_truthy_local_i32(expr.kind, self.scratch_local, function)
        }
    }

    fn compile_truthy_local_i32(
        &self,
        kind: ValueKind,
        local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match kind {
            ValueKind::Undefined | ValueKind::Null => {
                function.instruction(&Instruction::I32Const(0));
            }
            ValueKind::Object | ValueKind::Array | ValueKind::Function | ValueKind::Arguments => {
                function.instruction(&Instruction::I32Const(1));
            }
            ValueKind::Boolean => {
                function.instruction(&Instruction::LocalGet(local));
                function.instruction(&Instruction::I32WrapI64);
            }
            ValueKind::String => {
                function.instruction(&Instruction::LocalGet(local));
                function.instruction(&Instruction::I64Const(0xFFFF_FFFFu64 as i64));
                function.instruction(&Instruction::I64And);
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::I32Eqz);
            }
            ValueKind::Number => {
                function.instruction(&Instruction::LocalGet(local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                function.instruction(&Instruction::F64Eq);
                function.instruction(&Instruction::LocalGet(local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::LocalGet(local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Ne);
                function.instruction(&Instruction::I32Or);
                function.instruction(&Instruction::I32Eqz);
            }
            ValueKind::Dynamic => {
                return Err(EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: dynamic truthiness kind",
                ));
            }
        }
        Ok(())
    }

    fn compile_truthy_tagged_i32(
        &self,
        tag_local: u32,
        payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::I64Const(0xFFFF_FFFFu64 as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.compile_truthy_local_i32(ValueKind::Number, payload_local, function)?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn compile_strict_equality_i32(
        &mut self,
        lhs: &TypedExpr,
        rhs: &TypedExpr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if lhs.kind != ValueKind::Dynamic && rhs.kind != ValueKind::Dynamic && lhs.kind != rhs.kind
        {
            function.instruction(&Instruction::I32Const(0));
            return Ok(());
        }

        if lhs.kind != ValueKind::Dynamic && rhs.kind != ValueKind::Dynamic {
            match lhs.kind {
                ValueKind::Number => {
                    self.compile_expr_payload(lhs, function)?;
                    function.instruction(&Instruction::F64ReinterpretI64);
                    self.compile_expr_payload(rhs, function)?;
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::F64Eq);
                }
                _ => {
                    self.compile_expr_payload(lhs, function)?;
                    self.compile_expr_payload(rhs, function)?;
                    function.instruction(&Instruction::I64Eq);
                }
            }
            return Ok(());
        }

        let lhs_payload = self.reserve_temp_local();
        let lhs_tag = self.reserve_temp_local();
        let rhs_payload = self.reserve_temp_local();
        let rhs_tag = self.reserve_temp_local();
        self.compile_expr_to_locals(lhs, lhs_payload, lhs_tag, function)?;
        self.compile_expr_to_locals(rhs, rhs_payload, rhs_tag, function)?;
        self.emit_tagged_payload_equality_i32(
            lhs_tag,
            lhs_payload,
            rhs_tag,
            rhs_payload,
            function,
        )?;
        self.release_temp_local(rhs_tag);
        self.release_temp_local(rhs_payload);
        self.release_temp_local(lhs_tag);
        self.release_temp_local(lhs_payload);
        Ok(())
    }

    fn emit_tagged_payload_equality_i32(
        &self,
        lhs_tag_local: u32,
        lhs_payload_local: u32,
        rhs_tag_local: u32,
        rhs_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(lhs_tag_local));
        function.instruction(&Instruction::LocalGet(rhs_tag_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::LocalGet(lhs_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::LocalGet(lhs_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(rhs_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(lhs_payload_local));
        function.instruction(&Instruction::LocalGet(rhs_payload_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn emit_statement_result(&self, function: &mut Function, kind: ValueKind) {
        self.emit_undefined_payload(function);
        self.finish_statement_payload(function, kind);
    }

    fn finish_statement_payload(&self, function: &mut Function, kind: ValueKind) {
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(kind.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
    }

    fn emit_undefined_payload(&self, function: &mut Function) {
        function.instruction(&Instruction::I64Const(0));
    }

    fn store_payload_to_binding(&self, storage: BindingStorage, function: &mut Function) {
        match storage {
            BindingStorage::Fixed { payload_local, .. } => {
                function.instruction(&Instruction::LocalSet(payload_local));
            }
            BindingStorage::Dynamic { .. } | BindingStorage::EnvSlot { .. } => {
                panic!("dynamic binding write needs tagged path");
            }
        }
    }

    fn read_binding_payload(&mut self, storage: BindingStorage, function: &mut Function) {
        match storage {
            BindingStorage::Fixed { payload_local, .. }
            | BindingStorage::Dynamic { payload_local, .. } => {
                function.instruction(&Instruction::LocalGet(payload_local));
            }
            BindingStorage::EnvSlot { .. } => {
                self.read_binding_to_locals(
                    storage,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                );
                function.instruction(&Instruction::LocalGet(self.scratch_local));
            }
        }
    }

    fn read_binding_to_locals(
        &mut self,
        storage: BindingStorage,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        match storage {
            BindingStorage::Fixed {
                payload_local: binding_payload_local,
                kind,
            } => {
                function.instruction(&Instruction::LocalGet(binding_payload_local));
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(kind.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
            }
            BindingStorage::Dynamic {
                tag_local: binding_tag_local,
                payload_local: binding_payload_local,
            } => {
                function.instruction(&Instruction::LocalGet(binding_payload_local));
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::LocalGet(binding_tag_local));
                function.instruction(&Instruction::LocalSet(tag_local));
            }
            BindingStorage::EnvSlot { slot, hops } => {
                self.read_env_slot_to_locals(slot, hops, payload_local, tag_local, function);
            }
        }
    }

    fn allocate_binding(
        &mut self,
        name: String,
        mode: BindingMode,
        kind: ValueKind,
    ) -> BindingStorage {
        let storage = match mode {
            BindingMode::Let | BindingMode::Const if self.owned_env_slot(&name).is_some() => {
                BindingStorage::EnvSlot {
                    slot: self.owned_env_slot(&name).expect("owned env slot should exist"),
                    hops: 0,
                }
            }
            BindingMode::Let | BindingMode::Const => {
                let payload_local = self.next_binding_local;
                self.next_binding_local += 1;
                BindingStorage::Fixed {
                    payload_local,
                    kind,
                }
            }
            BindingMode::Var => panic!("var bindings are hoisted"),
        };
        self.binding_scopes
            .last_mut()
            .expect("binding scope stack must exist")
            .insert(name, storage);
        storage
    }

    fn lookup_binding(&self, name: &str) -> Option<BindingStorage> {
        self.binding_scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
    }

    fn push_scope(&mut self) {
        self.binding_scopes.push(BTreeMap::new());
    }

    fn pop_scope(&mut self) {
        self.binding_scopes.pop();
    }

    fn push_control(&mut self, kind: ControlFrameKind) -> usize {
        let index = self.control_stack.len();
        self.control_stack.push(kind);
        index
    }

    fn pop_control(&mut self, expected: ControlFrameKind) {
        let actual = self
            .control_stack
            .pop()
            .expect("control stack must not underflow");
        assert!(matches!(
            (actual, expected),
            (ControlFrameKind::If, ControlFrameKind::If)
                | (ControlFrameKind::Block, ControlFrameKind::Block)
                | (ControlFrameKind::Loop, ControlFrameKind::Loop)
        ));
    }

    fn depth_to(&self, target_index: usize) -> u32 {
        (self.control_stack.len() - 1 - target_index) as u32
    }

    fn push_labels(
        &mut self,
        labels: &[String],
        break_frame: usize,
        continue_frame: Option<usize>,
    ) {
        for label in labels {
            self.label_stack.push(LabelTargets {
                name: label.clone(),
                break_frame,
                continue_frame,
            });
        }
    }

    fn pop_labels(&mut self, count: usize) {
        for _ in 0..count {
            self.label_stack.pop();
        }
    }

    fn reserve_temp_local(&mut self) -> u32 {
        assert!(self.temp_stack_depth < self.temp_local_count);
        let local = self.temp_local_base + self.temp_stack_depth;
        self.temp_stack_depth += 1;
        local
    }

    fn release_temp_local(&mut self, local: u32) {
        assert!(self.temp_stack_depth > 0);
        self.temp_stack_depth -= 1;
        let expected = self.temp_local_base + self.temp_stack_depth;
        assert_eq!(local, expected);
    }
}

fn build_function_metas(functions: &[FunctionIr]) -> BTreeMap<FunctionId, WasmFunctionMeta> {
    functions
        .iter()
        .enumerate()
        .map(|(index, function)| {
            (
                function.id.clone(),
                WasmFunctionMeta {
                    name: function.name.clone(),
                    wasm_index: (index + 1) as u32,
                    table_index: index as u32,
                },
            )
        })
        .collect()
}

fn function_param_types() -> Vec<ValType> {
    std::iter::repeat_n(ValType::I64, JS_FUNCTION_PARAM_COUNT).collect()
}

fn count_param_locals(return_abi: ReturnAbi) -> usize {
    match return_abi {
        ReturnAbi::MainExport => 0,
        ReturnAbi::MultiValue => JS_FUNCTION_PARAM_COUNT,
    }
}

fn count_param_binding_locals(
    params: &[FunctionParamIr],
    owned_env_bindings: &[OwnedEnvBindingIr],
) -> usize {
    let owned = owned_env_bindings
        .iter()
        .map(|binding| binding.name.as_str())
        .collect::<BTreeSet<_>>();
    let mut locals = 0;
    for param in params {
        if !owned.contains(param.name.as_str()) {
            locals += 2;
        }
    }
    locals
}

fn script_uses_env(script: &ScriptIr) -> bool {
    !script.owned_env_bindings.is_empty()
        || script
            .functions
            .iter()
            .any(|function| !function.owned_env_bindings.is_empty())
}

fn script_uses_calls(script: &ScriptIr) -> bool {
    script
        .functions
        .iter()
        .any(|function| block_uses_calls(&function.body))
        || block_uses_calls(&script.body)
}

fn script_uses_function_table(script: &ScriptIr) -> bool {
    script
        .functions
        .iter()
        .any(|function| block_uses_function_table(&function.body))
        || block_uses_function_table(&script.body)
}

fn block_uses_function_table(block: &BlockIr) -> bool {
    block.statements.iter().any(statement_uses_function_table)
}

fn block_uses_calls(block: &BlockIr) -> bool {
    block.statements.iter().any(statement_uses_calls)
}

fn statement_uses_calls(statement: &StatementIr) -> bool {
    match statement {
        StatementIr::Empty
        | StatementIr::Debugger
        | StatementIr::Break { .. }
        | StatementIr::Continue { .. } => false,
        StatementIr::Lexical { init, .. } | StatementIr::Expression(init) => expr_uses_calls(init),
        StatementIr::Return(value) => expr_uses_calls(value),
        StatementIr::Var(declarators) => declarators
            .iter()
            .filter_map(|declarator| declarator.init.as_ref())
            .any(expr_uses_calls),
        StatementIr::Block(block) => block_uses_calls(block),
        StatementIr::If {
            condition,
            then_branch,
            else_branch,
        } => expr_uses_calls(condition)
            || statement_uses_calls(then_branch)
            || else_branch
                .as_deref()
                .map(statement_uses_calls)
                .unwrap_or(false),
        StatementIr::While { condition, body } => {
            expr_uses_calls(condition) || statement_uses_calls(body)
        }
        StatementIr::DoWhile { body, condition } => {
            statement_uses_calls(body) || expr_uses_calls(condition)
        }
        StatementIr::For { init, test, update, body } => {
            init.as_ref().map(for_init_uses_calls).unwrap_or(false)
                || test.as_ref().map(expr_uses_calls).unwrap_or(false)
                || update.as_ref().map(expr_uses_calls).unwrap_or(false)
                || statement_uses_calls(body)
        }
        StatementIr::Switch { discriminant, cases } => {
            expr_uses_calls(discriminant)
                || cases.iter().any(|case| {
                    case.condition.as_ref().map(expr_uses_calls).unwrap_or(false)
                        || block_uses_calls(&case.body)
                })
        }
        StatementIr::Labelled { statement, .. } => statement_uses_calls(statement),
    }
}

fn for_init_uses_calls(init: &ForInitIr) -> bool {
    match init {
        ForInitIr::Lexical { init, .. } | ForInitIr::Expression(init) => expr_uses_calls(init),
        ForInitIr::Var(declarators) => declarators
            .iter()
            .filter_map(|declarator| declarator.init.as_ref())
            .any(expr_uses_calls),
    }
}

fn statement_uses_function_table(statement: &StatementIr) -> bool {
    match statement {
        StatementIr::Empty | StatementIr::Debugger | StatementIr::Break { .. } | StatementIr::Continue { .. } => false,
        StatementIr::Lexical { init, .. } | StatementIr::Expression(init) | StatementIr::Return(init) => {
            expr_uses_function_table(init)
        }
        StatementIr::Var(declarators) => declarators
            .iter()
            .filter_map(|declarator| declarator.init.as_ref())
            .any(expr_uses_function_table),
        StatementIr::Block(block) => block_uses_function_table(block),
        StatementIr::If { condition, then_branch, else_branch } => {
            expr_uses_function_table(condition)
                || statement_uses_function_table(then_branch)
                || else_branch
                    .as_deref()
                    .map(statement_uses_function_table)
                    .unwrap_or(false)
        }
        StatementIr::While { condition, body } => {
            expr_uses_function_table(condition) || statement_uses_function_table(body)
        }
        StatementIr::DoWhile { body, condition } => {
            statement_uses_function_table(body) || expr_uses_function_table(condition)
        }
        StatementIr::For { init, test, update, body } => {
            init.as_ref().map(for_init_uses_function_table).unwrap_or(false)
                || test.as_ref().map(expr_uses_function_table).unwrap_or(false)
                || update.as_ref().map(expr_uses_function_table).unwrap_or(false)
                || statement_uses_function_table(body)
        }
        StatementIr::Switch { discriminant, cases } => {
            expr_uses_function_table(discriminant)
                || cases.iter().any(|case| {
                    case.condition
                        .as_ref()
                        .map(expr_uses_function_table)
                        .unwrap_or(false)
                        || block_uses_function_table(&case.body)
                })
        }
        StatementIr::Labelled { statement, .. } => statement_uses_function_table(statement),
    }
}

fn for_init_uses_function_table(init: &ForInitIr) -> bool {
    match init {
        ForInitIr::Lexical { init, .. } | ForInitIr::Expression(init) => expr_uses_function_table(init),
        ForInitIr::Var(declarators) => declarators
            .iter()
            .filter_map(|declarator| declarator.init.as_ref())
            .any(expr_uses_function_table),
    }
}

fn expr_uses_function_table(expr: &TypedExpr) -> bool {
    match &expr.expr {
        ExprIr::FunctionValue(_) | ExprIr::CallIndirect { .. } | ExprIr::CallMethod { .. } => true,
        ExprIr::AssignIdentifier { value, .. }
        | ExprIr::CompoundAssignIdentifier { value, .. }
        | ExprIr::UnaryNumber { expr: value, .. }
        | ExprIr::LogicalNot { expr: value } => expr_uses_function_table(value),
        ExprIr::ObjectLiteral(properties) => properties.iter().any(|property| match property {
            ObjectPropertyIr::Data { value, .. } => expr_uses_function_table(value),
            ObjectPropertyIr::Method { function, .. }
            | ObjectPropertyIr::Getter { function, .. }
            | ObjectPropertyIr::Setter { function, .. } => expr_uses_function_table(function),
        }),
        ExprIr::ArrayLiteral(elements) => elements.iter().any(expr_uses_function_table),
        ExprIr::PropertyRead { target, key } => {
            matches!(target.kind, ValueKind::Object)
                || expr_uses_function_table(target)
                || match key {
                    PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => false,
                    PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                        expr_uses_function_table(expr)
                    }
                }
        }
        ExprIr::PropertyWrite { target, key, value } => {
            matches!(target.kind, ValueKind::Object)
                || expr_uses_function_table(target)
                || expr_uses_function_table(value)
                || match key {
                    PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => false,
                    PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                        expr_uses_function_table(expr)
                    }
                }
        }
        ExprIr::BinaryNumber { lhs, rhs, .. }
        | ExprIr::CompareNumber { lhs, rhs, .. }
        | ExprIr::StrictEquality { lhs, rhs, .. }
        | ExprIr::LogicalShortCircuit { lhs, rhs, .. } => {
            expr_uses_function_table(lhs) || expr_uses_function_table(rhs)
        }
        ExprIr::CallNamed { args, .. } => args.iter().any(expr_uses_function_table),
        ExprIr::Arguments => false,
        ExprIr::Undefined
        | ExprIr::Null
        | ExprIr::Boolean(_)
        | ExprIr::Number(_)
        | ExprIr::String(_)
        | ExprIr::This
        | ExprIr::Identifier(_)
        | ExprIr::UpdateIdentifier { .. } => false,
    }
}

fn expr_uses_calls(expr: &TypedExpr) -> bool {
    match &expr.expr {
        ExprIr::CallNamed { .. } | ExprIr::CallIndirect { .. } | ExprIr::CallMethod { .. } => true,
        ExprIr::AssignIdentifier { value, .. }
        | ExprIr::CompoundAssignIdentifier { value, .. }
        | ExprIr::UnaryNumber { expr: value, .. }
        | ExprIr::LogicalNot { expr: value } => expr_uses_calls(value),
        ExprIr::ObjectLiteral(properties) => properties.iter().any(|property| match property {
            ObjectPropertyIr::Data { value, .. } => expr_uses_calls(value),
            ObjectPropertyIr::Method { function, .. }
            | ObjectPropertyIr::Getter { function, .. }
            | ObjectPropertyIr::Setter { function, .. } => expr_uses_calls(function),
        }),
        ExprIr::ArrayLiteral(elements) => elements.iter().any(expr_uses_calls),
        ExprIr::PropertyRead { target, key } => {
            expr_uses_calls(target)
                || match key {
                    PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => false,
                    PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                        expr_uses_calls(expr)
                    }
                }
        }
        ExprIr::PropertyWrite { target, key, value } => {
            expr_uses_calls(target)
                || expr_uses_calls(value)
                || match key {
                    PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => false,
                    PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                        expr_uses_calls(expr)
                    }
                }
        }
        ExprIr::BinaryNumber { lhs, rhs, .. }
        | ExprIr::CompareNumber { lhs, rhs, .. }
        | ExprIr::StrictEquality { lhs, rhs, .. }
        | ExprIr::LogicalShortCircuit { lhs, rhs, .. } => {
            expr_uses_calls(lhs) || expr_uses_calls(rhs)
        }
        ExprIr::Arguments
        | ExprIr::Undefined
        | ExprIr::Null
        | ExprIr::Boolean(_)
        | ExprIr::Number(_)
        | ExprIr::String(_)
        | ExprIr::FunctionValue(_)
        | ExprIr::This
        | ExprIr::Identifier(_)
        | ExprIr::UpdateIdentifier { .. } => false,
    }
}

fn count_block_lexicals(block: &BlockIr) -> usize {
    block.statements.iter().map(count_statement_lexicals).sum()
}

fn count_block_temp_locals(block: &BlockIr) -> usize {
    block
        .statements
        .iter()
        .map(count_statement_temp_locals)
        .max()
        .unwrap_or(0)
}

fn count_statement_lexicals(statement: &StatementIr) -> usize {
    match statement {
        StatementIr::Empty
        | StatementIr::Var(_)
        | StatementIr::Expression(_)
        | StatementIr::Debugger
        | StatementIr::Return(_)
        | StatementIr::Break { .. }
        | StatementIr::Continue { .. } => 0,
        StatementIr::Lexical { .. } => 1,
        StatementIr::Block(block) => count_block_lexicals(block),
        StatementIr::If {
            then_branch,
            else_branch,
            ..
        } => {
            count_statement_lexicals(then_branch)
                + else_branch
                    .as_deref()
                    .map(count_statement_lexicals)
                    .unwrap_or(0)
        }
        StatementIr::While { body, .. } | StatementIr::DoWhile { body, .. } => {
            count_statement_lexicals(body)
        }
        StatementIr::For { init, body, .. } => {
            init.as_ref()
                .map(|init| match init {
                    ForInitIr::Lexical { .. } => 1,
                    ForInitIr::Var(_) => 0,
                    ForInitIr::Expression(_) => 0,
                })
                .unwrap_or(0)
                + count_statement_lexicals(body)
        }
        StatementIr::Switch { cases, .. } => cases
            .iter()
            .map(|case| count_block_lexicals(&case.body))
            .sum(),
        StatementIr::Labelled { statement, .. } => count_statement_lexicals(statement),
    }
}

fn count_statement_temp_locals(statement: &StatementIr) -> usize {
    match statement {
        StatementIr::Empty
        | StatementIr::Debugger
        | StatementIr::Break { .. }
        | StatementIr::Continue { .. } => 0,
        StatementIr::Return(value) => count_expr_temp_locals(value),
        StatementIr::Var(declarators) => declarators
            .iter()
            .filter_map(|declarator| declarator.init.as_ref())
            .map(count_expr_temp_locals)
            .max()
            .unwrap_or(0),
        StatementIr::Lexical { init, .. } | StatementIr::Expression(init) => {
            count_expr_temp_locals(init)
        }
        StatementIr::Block(block) => count_block_temp_locals(block),
        StatementIr::If {
            condition,
            then_branch,
            else_branch,
        } => count_expr_temp_locals(condition)
            .max(count_statement_temp_locals(then_branch))
            .max(
                else_branch
                    .as_deref()
                    .map(count_statement_temp_locals)
                    .unwrap_or(0),
            ),
        StatementIr::While { condition, body } => {
            count_expr_temp_locals(condition).max(count_statement_temp_locals(body))
        }
        StatementIr::DoWhile { body, condition } => {
            count_statement_temp_locals(body).max(count_expr_temp_locals(condition))
        }
        StatementIr::For {
            init,
            test,
            update,
            body,
        } => init
            .as_ref()
            .map(count_for_init_temp_locals)
            .unwrap_or(0)
            .max(test.as_ref().map(count_expr_temp_locals).unwrap_or(0))
            .max(update.as_ref().map(count_expr_temp_locals).unwrap_or(0))
            .max(count_statement_temp_locals(body)),
        StatementIr::Switch {
            discriminant,
            cases,
        } => {
            let case_max = cases
                .iter()
                .map(|case| {
                    case.condition
                        .as_ref()
                        .map(count_expr_temp_locals)
                        .unwrap_or(0)
                        .max(count_block_temp_locals(&case.body))
                })
                .max()
                .unwrap_or(0);
            4 + count_expr_temp_locals(discriminant).max(case_max)
        }
        StatementIr::Labelled { statement, .. } => count_statement_temp_locals(statement),
    }
}

fn count_for_init_temp_locals(init: &ForInitIr) -> usize {
    match init {
        ForInitIr::Lexical { init, .. } => count_expr_temp_locals(init),
        ForInitIr::Var(declarators) => declarators
            .iter()
            .filter_map(|declarator| declarator.init.as_ref())
            .map(count_expr_temp_locals)
            .max()
            .unwrap_or(0),
        ForInitIr::Expression(expr) => count_expr_temp_locals(expr),
    }
}

fn count_expr_temp_locals(expr: &TypedExpr) -> usize {
    match &expr.expr {
        ExprIr::ObjectLiteral(properties) => {
            let child = properties
                .iter()
                .map(|property| match property {
                    ObjectPropertyIr::Data { value, .. } => count_expr_temp_locals(value),
                    ObjectPropertyIr::Method { function, .. }
                    | ObjectPropertyIr::Getter { function, .. }
                    | ObjectPropertyIr::Setter { function, .. } => count_expr_temp_locals(function),
                })
                .max()
                .unwrap_or(0);
            child.max(12)
        }
        ExprIr::ArrayLiteral(elements) => {
            let child = elements
                .iter()
                .map(count_expr_temp_locals)
                .max()
                .unwrap_or(0);
            child.max(6)
        }
        ExprIr::PropertyRead { target, key } => {
            let child = count_expr_temp_locals(target).max(match key {
                PropertyKeyIr::StaticString(_) => 0,
                PropertyKeyIr::ArrayLength => 0,
                PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                    count_expr_temp_locals(expr)
                }
            });
            child.max(12)
        }
        ExprIr::PropertyWrite { target, key, value } => {
            let child = count_expr_temp_locals(target)
                .max(count_expr_temp_locals(value))
                .max(match key {
                    PropertyKeyIr::StaticString(_) => 0,
                    PropertyKeyIr::ArrayLength => 0,
                    PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                        count_expr_temp_locals(expr)
                    }
                });
            child.max(12)
        }
        ExprIr::UpdateIdentifier { return_mode, .. } => match return_mode {
            UpdateReturnMode::Prefix => 0,
            UpdateReturnMode::Postfix => 1,
        },
        ExprIr::CompoundAssignIdentifier { op, value, .. } => {
            let child = count_expr_temp_locals(value);
            let _ = op;
            1 + child
        }
        ExprIr::AssignIdentifier { value, .. }
        | ExprIr::UnaryNumber { expr: value, .. }
        | ExprIr::LogicalNot { expr: value } => count_expr_temp_locals(value),
        ExprIr::BinaryNumber { lhs, rhs, .. }
        | ExprIr::CompareNumber { lhs, rhs, .. }
        | ExprIr::LogicalShortCircuit { lhs, rhs, .. } => {
            count_expr_temp_locals(lhs).max(count_expr_temp_locals(rhs))
        }
        ExprIr::StrictEquality { lhs, rhs, .. } => {
            let child = count_expr_temp_locals(lhs).max(count_expr_temp_locals(rhs));
            if lhs.kind == ValueKind::Dynamic || rhs.kind == ValueKind::Dynamic {
                child.max(4)
            } else {
                child
            }
        }
        ExprIr::CallNamed { args, .. } => args
            .iter()
            .map(count_expr_temp_locals)
            .max()
            .unwrap_or(0)
            .max(4),
        ExprIr::CallIndirect { callee, args } => count_expr_temp_locals(callee)
            .max(args.iter().map(count_expr_temp_locals).max().unwrap_or(0))
            .max(6),
        ExprIr::CallMethod { receiver, key, args } => {
            let key_child = match key {
                PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => 0,
                PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                    count_expr_temp_locals(expr)
                }
            };
            count_expr_temp_locals(receiver)
                .max(key_child)
                .max(args.iter().map(count_expr_temp_locals).max().unwrap_or(0))
                .max(8)
        }
        ExprIr::Arguments => 0,
        ExprIr::Undefined
        | ExprIr::Null
        | ExprIr::Boolean(_)
        | ExprIr::Number(_)
        | ExprIr::String(_)
        | ExprIr::FunctionValue(_)
        | ExprIr::This
        | ExprIr::Identifier(_) => 0,
    }
}

fn collect_hoisted_vars_block_root(block: &BlockIr) -> Vec<String> {
    let mut names = BTreeSet::new();
    collect_hoisted_vars_block(block, &mut names);
    names.into_iter().collect()
}

fn collect_hoisted_vars_block(block: &BlockIr, names: &mut BTreeSet<String>) {
    for statement in &block.statements {
        collect_hoisted_vars_statement(statement, names);
    }
}

fn collect_hoisted_vars_statement(statement: &StatementIr, names: &mut BTreeSet<String>) {
    match statement {
        StatementIr::Var(declarators) => {
            for declarator in declarators {
                names.insert(declarator.name.clone());
            }
        }
        StatementIr::Block(block) => collect_hoisted_vars_block(block, names),
        StatementIr::If {
            then_branch,
            else_branch,
            ..
        } => {
            collect_hoisted_vars_statement(then_branch, names);
            if let Some(else_branch) = else_branch {
                collect_hoisted_vars_statement(else_branch, names);
            }
        }
        StatementIr::While { body, .. }
        | StatementIr::DoWhile { body, .. }
        | StatementIr::Labelled {
            statement: body, ..
        } => collect_hoisted_vars_statement(body, names),
        StatementIr::For { init, body, .. } => {
            if let Some(ForInitIr::Var(declarators)) = init {
                for declarator in declarators {
                    names.insert(declarator.name.clone());
                }
            }
            collect_hoisted_vars_statement(body, names);
        }
        StatementIr::Switch { cases, .. } => {
            for case in cases {
                collect_hoisted_vars_block(&case.body, names);
            }
        }
        StatementIr::Empty
        | StatementIr::Lexical { .. }
        | StatementIr::Expression(_)
        | StatementIr::Debugger
        | StatementIr::Return(_)
        | StatementIr::Break { .. }
        | StatementIr::Continue { .. } => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use porffor_front::{parse, ParseOptions};
    use porffor_ir::lower;

    fn emit_script(source: &str) -> Result<WasmArtifact, EmitError> {
        let source = parse(source, ParseOptions::script()).expect("script should parse");
        emit(&lower(&source))
    }

    #[test]
    fn emitted_module_validates() {
        let artifact = emit_script("let x = 40; const y = 2; x + y;").expect("emit should work");
        wasmparser::Validator::new()
            .validate_all(&artifact.bytes)
            .expect("module should validate");
        assert!(artifact.debug_dump.contains("export func: main"));
        assert!(artifact.debug_dump.contains("export global: result_tag"));
    }

    #[test]
    fn string_script_emits_memory_and_data() {
        let artifact = emit_script("const s = \"hi\"; s;").expect("emit should work");
        assert!(artifact
            .debug_dump
            .contains("memory: exported linear memory"));
        assert!(artifact.debug_dump.contains("data segments: 1"));
    }

    #[test]
    fn supports_assignment_branching_and_loops() {
        let artifact = emit_script(
            "let i = 0; let sum = 0; for (; i < 5; i = i + 1) { if (i === 2) { continue; } if (i === 4) { break; } sum = sum + i; } sum;",
        )
        .expect("emit should work");
        wasmparser::Validator::new()
            .validate_all(&artifact.bytes)
            .expect("module should validate");
    }

    #[test]
    fn supports_updates_and_compound_assignment() {
        let artifact = emit_script("let sum = 0; for (let i = 0; i < 4; i++) { sum += i; } sum;")
            .expect("emit should work");
        wasmparser::Validator::new()
            .validate_all(&artifact.bytes)
            .expect("module should validate");
    }

    #[test]
    fn supports_switch_labels_and_debugger() {
        let artifact = emit_script(
            "let x = 0; outer: while (x < 3) { x += 1; switch (x) { case 1: continue outer; case 2: debugger; break outer; default: break; } } x;",
        )
        .expect("emit should work");
        wasmparser::Validator::new()
            .validate_all(&artifact.bytes)
            .expect("module should validate");
    }

    #[test]
    fn supports_direct_function_calls_and_recursion() {
        let artifact = emit_script(
            "function up(n) { if (n === 0) { return 0; } return up(n - 1) + 1; } up(3);",
        )
        .expect("emit should work");
        wasmparser::Validator::new()
            .validate_all(&artifact.bytes)
            .expect("module should validate");
        assert!(artifact.debug_dump.contains("internal functions: 1"));
    }

    #[test]
    fn object_and_array_scripts_emit_memory_without_imports() {
        let artifact =
            emit_script("let o = { x: 1 }; let a = [1]; a[2] = 4; o.x;").expect("emit should work");
        wasmparser::Validator::new()
            .validate_all(&artifact.bytes)
            .expect("module should validate");
        assert!(artifact
            .debug_dump
            .contains("memory: exported linear memory"));
        assert!(artifact.debug_dump.contains("data segments: 1"));
    }

    #[test]
    fn supports_object_return_from_function() {
        let artifact =
            emit_script("function box(x) { let o = { x: x }; return o; } let o = box(2); o.x;")
                .expect("emit should work");
        wasmparser::Validator::new()
            .validate_all(&artifact.bytes)
            .expect("module should validate");
    }

    #[test]
    fn supports_chained_heap_access_and_array_length() {
        let artifact = emit_script(
            "function box() { let o = { inner: { x: 2 } }; return o; } let a = [1, 2, 3]; box().inner.x + a.length;",
        )
        .expect("emit should work");
        wasmparser::Validator::new()
            .validate_all(&artifact.bytes)
            .expect("module should validate");
    }

    #[test]
    fn supports_heap_growth_beyond_initial_capacity() {
        let source = format!(
            "let o = {{}}; {} o.k64;",
            (0..65)
                .map(|index| format!("o[\"k{index}\"] = {index};"))
                .collect::<Vec<_>>()
                .join(" ")
        );
        let artifact = emit_script(&source).expect("emit should work");
        wasmparser::Validator::new()
            .validate_all(&artifact.bytes)
            .expect("module should validate");
        assert!(artifact.debug_dump.contains("memory: exported linear memory"));
    }

    #[test]
    fn unsupported_script_returns_precise_error() {
        let err = emit_script("\"a\" + \"b\";").expect_err("string plus should fail");
        assert!(err
            .to_string()
            .contains("unsupported in porffor wasm-aot first slice"));
    }

    #[test]
    fn coercive_compound_assignment_returns_precise_error() {
        let err = emit_script("let s = \"a\"; s += \"b\";")
            .expect_err("string compound assignment should fail");
        assert!(err
            .to_string()
            .contains("unsupported in porffor wasm-aot first slice"));
    }
}
