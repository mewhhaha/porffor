use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
    sync::LazyLock,
};

use porffor_ir::{
    private_brand_key, private_data_key, ArithmeticBinaryOp, BindingMode, BlockIr,
    CallableToStringRepresentation, ClassDefinitionIr, ClassFunctionKind, ClassHeritageKind,
    ClassMethodPlacementIr, DeleteIdentifierKindIr, EqualityBinaryOp, ExprIr, ForInitIr,
    FunctionFlavor, FunctionId, FunctionIr, FunctionParamIr, HostBuiltinId, KindSet,
    LogicalBinaryOp, NumericUpdateOp, ObjectPropertyIr, OwnedEnvBindingIr, PrivateNameId,
    ProgramIr, PropertyKeyIr, RelationalBinaryOp, ScriptGlobalBindingIr, ScriptGlobalBindingKind,
    ScriptIr, StandardBuiltinId, StatementIr, SwitchCaseIr, ToPrimitiveHint, TypedExpr,
    UnaryNumericOp, UpdateReturnMode, ValueInfo, ValueKind, VarDeclaratorIr, AGGREGATE_ERROR_NAME,
    ARRAY_BUFFER_BYTE_LENGTH_SLOT, ARRAY_BUFFER_DATA_PTR_SLOT, ARRAY_BUFFER_NAME, ARRAY_NAME,
    BOOLEAN_NAME, DATA_VIEW_BYTE_LENGTH_SLOT, DATA_VIEW_BYTE_OFFSET_SLOT, DATA_VIEW_DATA_PTR_SLOT,
    DATA_VIEW_NAME, ERROR_NAME, EVAL_ERROR_NAME, FUNCTION_NAME, GLOBAL_THIS_NAME,
    LEXICAL_ARGUMENTS_NAME, LEXICAL_NEW_TARGET_NAME, LEXICAL_THIS_NAME, NUMBER_NAME, OBJECT_NAME,
    PRINT_NAME, RANGE_ERROR_NAME, REFERENCE_ERROR_NAME, STRING_NAME, SYNTAX_ERROR_NAME,
    TYPE_ERROR_NAME, URI_ERROR_NAME,
};
use wasm_encoder::{
    BlockType, CodeSection, ConstExpr, DataSection, ElementSection, Elements, ExportKind,
    ExportSection, Function, FunctionSection, GlobalSection, GlobalType, Ieee64, ImportSection,
    Instruction, MemArg, MemorySection, MemoryType, Module, RefType, TableSection, TableType,
    TypeSection, ValType,
};

const RESULT_TAG_EXPORT: &str = "result_tag";
const COMPLETION_KIND_EXPORT: &str = "completion_kind";
const COMPLETION_AUX_EXPORT: &str = "completion_aux";
const HOST_IMPORT_MODULE: &str = "porf_host";
const HOST_IMPORT_PRINT_LINE_UTF8: &str = "print_line_utf8";
const RESULT_TAG_GLOBAL_INDEX: u32 = 0;
const COMPLETION_KIND_GLOBAL_INDEX: u32 = 1;
const COMPLETION_AUX_GLOBAL_INDEX: u32 = 2;
const HEAP_PTR_GLOBAL_INDEX: u32 = 3;
const SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX: u32 = 4;
const OBJECT_PROTOTYPE_GLOBAL_INDEX: u32 = 5;
const FUNCTION_PROTOTYPE_GLOBAL_INDEX: u32 = 6;
const ARRAY_PROTOTYPE_GLOBAL_INDEX: u32 = 7;
const NUMBER_PROTOTYPE_GLOBAL_INDEX: u32 = 8;
const STRING_PROTOTYPE_GLOBAL_INDEX: u32 = 9;
const BOOLEAN_PROTOTYPE_GLOBAL_INDEX: u32 = 10;
const ERROR_PROTOTYPE_GLOBAL_INDEX: u32 = 11;
const TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX: u32 = 12;
const REFERENCE_ERROR_PROTOTYPE_GLOBAL_INDEX: u32 = 13;
const EVAL_ERROR_PROTOTYPE_GLOBAL_INDEX: u32 = 14;
const RANGE_ERROR_PROTOTYPE_GLOBAL_INDEX: u32 = 15;
const SYNTAX_ERROR_PROTOTYPE_GLOBAL_INDEX: u32 = 16;
const URI_ERROR_PROTOTYPE_GLOBAL_INDEX: u32 = 17;
const AGGREGATE_ERROR_PROTOTYPE_GLOBAL_INDEX: u32 = 18;
const FUNCTION_CONSTRUCTOR_GLOBAL_INDEX: u32 = 19;
const OBJECT_CONSTRUCTOR_GLOBAL_INDEX: u32 = 20;
const ARRAY_CONSTRUCTOR_GLOBAL_INDEX: u32 = 21;
const NUMBER_CONSTRUCTOR_GLOBAL_INDEX: u32 = 22;
const STRING_CONSTRUCTOR_GLOBAL_INDEX: u32 = 23;
const BOOLEAN_CONSTRUCTOR_GLOBAL_INDEX: u32 = 24;
const ERROR_CONSTRUCTOR_GLOBAL_INDEX: u32 = 25;
const TYPE_ERROR_CONSTRUCTOR_GLOBAL_INDEX: u32 = 26;
const REFERENCE_ERROR_CONSTRUCTOR_GLOBAL_INDEX: u32 = 27;
const EVAL_ERROR_CONSTRUCTOR_GLOBAL_INDEX: u32 = 28;
const AGGREGATE_ERROR_CONSTRUCTOR_GLOBAL_INDEX: u32 = 29;
const RANGE_ERROR_CONSTRUCTOR_GLOBAL_INDEX: u32 = 30;
const SYNTAX_ERROR_CONSTRUCTOR_GLOBAL_INDEX: u32 = 31;
const URI_ERROR_CONSTRUCTOR_GLOBAL_INDEX: u32 = 32;
const ARRAY_BUFFER_PROTOTYPE_GLOBAL_INDEX: u32 = 33;
const DATA_VIEW_PROTOTYPE_GLOBAL_INDEX: u32 = 34;
const ARRAY_BUFFER_CONSTRUCTOR_GLOBAL_INDEX: u32 = 35;
const DATA_VIEW_CONSTRUCTOR_GLOBAL_INDEX: u32 = 36;
const JS_FUNCTION_TYPE_INDEX: u32 = 1;
const HOST_PRINT_IMPORT_TYPE_INDEX: u32 = 2;
const HOST_PRINT_IMPORT_FUNCTION_INDEX: u32 = 0;
const WASM_PAGE_SIZE: u64 = 65_536;
const STATIC_DATA_OFFSET: u32 = 4096;
const MIN_HEAP_CAPACITY: u64 = 1;
const HEAP_HEADER_SIZE: u64 = 56;
const HEAP_FUNCTION_OBJECT_SIZE: u64 = 80;
const HEAP_OBJECT_ENTRY_SIZE: u64 = 64;
const HEAP_ARRAY_ENTRY_SIZE: u64 = 16;
const HEAP_BOUND_FUNCTION_RECORD_SIZE: u64 = 40;
const HEAP_ARGUMENTS_MAPPED_COUNT_OFFSET: u64 = 32;
const HEAP_ARGUMENTS_ENV_HANDLE_OFFSET: u64 = 40;
const HEAP_OBJECT_BOXED_KIND_OFFSET: u64 = 32;
const HEAP_OBJECT_BOXED_TAG_OFFSET: u64 = 40;
const HEAP_OBJECT_BOXED_PAYLOAD_OFFSET: u64 = 48;
const HEAP_PTR_OFFSET: u64 = 0;
const HEAP_LEN_OFFSET: u64 = 8;
const HEAP_CAP_OFFSET: u64 = 16;
const HEAP_PROTOTYPE_OFFSET: u64 = 24;
const HEAP_FUNCTION_TABLE_INDEX_OFFSET: u64 = 32;
const HEAP_FUNCTION_ENV_HANDLE_OFFSET: u64 = 40;
const HEAP_FUNCTION_FLAGS_OFFSET: u64 = 48;
const HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET: u64 = 56;
const HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET: u64 = 64;
const HEAP_FUNCTION_TO_STRING_PAYLOAD_OFFSET: u64 = 72;
const HEAP_BOUND_FUNCTION_TARGET_TAG_OFFSET: u64 = 0;
const HEAP_BOUND_FUNCTION_TARGET_PAYLOAD_OFFSET: u64 = 8;
const HEAP_BOUND_FUNCTION_THIS_TAG_OFFSET: u64 = 16;
const HEAP_BOUND_FUNCTION_THIS_PAYLOAD_OFFSET: u64 = 24;
const HEAP_BOUND_FUNCTION_ARGS_PAYLOAD_OFFSET: u64 = 32;
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
const OBJECT_DESCRIPTOR_ACCESSOR: u64 = 1;
const OBJECT_DESCRIPTOR_CONFIGURABLE: u64 = 2;
const OBJECT_DESCRIPTOR_DATA: u64 = 0;
const BOXED_PRIMITIVE_KIND_NONE: u64 = 0;
const BOXED_PRIMITIVE_KIND_NUMBER: u64 = 1;
const BOXED_PRIMITIVE_KIND_STRING: u64 = 2;
const BOXED_PRIMITIVE_KIND_BOOLEAN: u64 = 3;
const FUNCTION_FLAG_CONSTRUCTABLE: u64 = 1;
const FUNCTION_FLAG_CLASS_CONSTRUCTOR: u64 = 2;
const FUNCTION_FLAG_BOUND: u64 = 4;
const JS_FUNCTION_PARAM_COUNT: usize = 7;
const COMPLETION_KIND_NORMAL: i64 = 0;
const COMPLETION_KIND_THROW: i64 = 1;
const COMPLETION_KIND_RETURN: i64 = 2;
const COMPLETION_KIND_BREAK: i64 = 3;
const COMPLETION_KIND_CONTINUE: i64 = 4;
const HEAP_ARRAY_HOLE_TAG: i64 = ValueKind::Dynamic.tag() as i64;
static EMPTY_BLOCK: LazyLock<BlockIr> = LazyLock::new(|| BlockIr {
    statements: Vec::new(),
    result_kind: ValueKind::Undefined,
});

fn standard_builtin_constructor_global_index(builtin: StandardBuiltinId) -> Option<u32> {
    match builtin {
        StandardBuiltinId::FunctionConstructor => Some(FUNCTION_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::AggregateErrorConstructor => {
            Some(AGGREGATE_ERROR_CONSTRUCTOR_GLOBAL_INDEX)
        }
        StandardBuiltinId::ObjectConstructor => Some(OBJECT_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::ArrayConstructor => Some(ARRAY_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::ArrayBufferConstructor => Some(ARRAY_BUFFER_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::DataViewConstructor => Some(DATA_VIEW_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::NumberConstructor => Some(NUMBER_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::StringConstructor => Some(STRING_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::BooleanConstructor => Some(BOOLEAN_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::ErrorConstructor => Some(ERROR_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::EvalErrorConstructor => Some(EVAL_ERROR_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::RangeErrorConstructor => Some(RANGE_ERROR_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::SyntaxErrorConstructor => Some(SYNTAX_ERROR_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::TypeErrorConstructor => Some(TYPE_ERROR_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::URIErrorConstructor => Some(URI_ERROR_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::ReferenceErrorConstructor => {
            Some(REFERENCE_ERROR_CONSTRUCTOR_GLOBAL_INDEX)
        }
        StandardBuiltinId::FunctionPrototypeCall
        | StandardBuiltinId::FunctionPrototypeApply
        | StandardBuiltinId::FunctionPrototypeBind
        | StandardBuiltinId::FunctionPrototypeToString
        | StandardBuiltinId::ObjectCreate
        | StandardBuiltinId::ObjectGetPrototypeOf
        | StandardBuiltinId::ArrayIsArray
        | StandardBuiltinId::DataViewPrototypeGetUint8
        | StandardBuiltinId::ErrorPrototypeToString
        | StandardBuiltinId::BoundFunctionInvoker => None,
    }
}

fn error_prototype_global_index(name: &str) -> u32 {
    match name {
        ERROR_NAME => ERROR_PROTOTYPE_GLOBAL_INDEX,
        EVAL_ERROR_NAME => EVAL_ERROR_PROTOTYPE_GLOBAL_INDEX,
        AGGREGATE_ERROR_NAME => AGGREGATE_ERROR_PROTOTYPE_GLOBAL_INDEX,
        RANGE_ERROR_NAME => RANGE_ERROR_PROTOTYPE_GLOBAL_INDEX,
        SYNTAX_ERROR_NAME => SYNTAX_ERROR_PROTOTYPE_GLOBAL_INDEX,
        TYPE_ERROR_NAME => TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX,
        URI_ERROR_NAME => URI_ERROR_PROTOTYPE_GLOBAL_INDEX,
        REFERENCE_ERROR_NAME => REFERENCE_ERROR_PROTOTYPE_GLOBAL_INDEX,
        _ => OBJECT_PROTOTYPE_GLOBAL_INDEX,
    }
}

fn boxed_prototype_global_index(builtin: StandardBuiltinId) -> Option<u32> {
    match builtin {
        StandardBuiltinId::NumberConstructor => Some(NUMBER_PROTOTYPE_GLOBAL_INDEX),
        StandardBuiltinId::StringConstructor => Some(STRING_PROTOTYPE_GLOBAL_INDEX),
        StandardBuiltinId::BooleanConstructor => Some(BOOLEAN_PROTOTYPE_GLOBAL_INDEX),
        _ => None,
    }
}

fn boxed_primitive_kind_tag(builtin: StandardBuiltinId) -> Option<u64> {
    match builtin {
        StandardBuiltinId::NumberConstructor => Some(BOXED_PRIMITIVE_KIND_NUMBER),
        StandardBuiltinId::StringConstructor => Some(BOXED_PRIMITIVE_KIND_STRING),
        StandardBuiltinId::BooleanConstructor => Some(BOXED_PRIMITIVE_KIND_BOOLEAN),
        _ => None,
    }
}

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
    let uses_heap = true;
    let uses_host_print = script.host_builtins.contains(&HostBuiltinId::Print);
    let imported_function_count = u32::from(uses_host_print);
    let standard_builtins = StandardBuiltinId::all_functions();
    let function_metas = build_function_metas(
        script.functions.as_slice(),
        standard_builtins,
        script.host_builtins.as_slice(),
        imported_function_count,
    );
    let string_pool = StringPool::collect(script, &function_metas);
    let uses_function_table = true;
    let mut main_builder =
        FunctionBuilder::new_main(script, &string_pool, &function_metas, uses_heap);
    let main_function = main_builder.compile()?;
    let mut compiled_functions = Vec::with_capacity(
        script.functions.len() + standard_builtins.len() + script.host_builtins.len(),
    );
    for function in &script.functions {
        let mut builder =
            FunctionBuilder::new_function(function, &string_pool, &function_metas, uses_heap);
        compiled_functions.push(builder.compile()?);
    }
    for builtin in standard_builtins {
        let mut builder = FunctionBuilder::new_standard_builtin(
            *builtin,
            &string_pool,
            &function_metas,
            uses_heap,
        );
        compiled_functions.push(builder.compile_builtin()?);
    }
    for builtin in &script.host_builtins {
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
        script.functions.len() + standard_builtins.len() + script.host_builtins.len();
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
        for _ in 0..32 {
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
        format!("export global: {RESULT_TAG_EXPORT}"),
        format!("export global: {COMPLETION_KIND_EXPORT}"),
        format!("export global: {COMPLETION_AUX_EXPORT}"),
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
    fn collect(script: &ScriptIr, function_metas: &BTreeMap<FunctionId, WasmFunctionMeta>) -> Self {
        let mut pool = Self::default();
        for value in [
            "",
            " ",
            ": ",
            ",",
            "undefined",
            "null",
            "true",
            "false",
            "NaN",
            "Infinity",
            "-Infinity",
            "[object Object]",
            "[object Arguments]",
            "prototype",
            "constructor",
            "valueOf",
            "toString",
            "object",
            "boolean",
            "number",
            "string",
            "function",
            "function(handle@",
            ")",
            "length",
            "name",
            "message",
            "errors",
            FUNCTION_NAME,
            OBJECT_NAME,
            ARRAY_NAME,
            ARRAY_BUFFER_NAME,
            DATA_VIEW_NAME,
            NUMBER_NAME,
            STRING_NAME,
            BOOLEAN_NAME,
            ERROR_NAME,
            EVAL_ERROR_NAME,
            AGGREGATE_ERROR_NAME,
            RANGE_ERROR_NAME,
            SYNTAX_ERROR_NAME,
            TYPE_ERROR_NAME,
            URI_ERROR_NAME,
            REFERENCE_ERROR_NAME,
            "call",
            "apply",
            "bind",
            "create",
            "getPrototypeOf",
            "isArray",
            "getUint8",
            "f",
            "v",
            ARRAY_BUFFER_DATA_PTR_SLOT,
            ARRAY_BUFFER_BYTE_LENGTH_SLOT,
            DATA_VIEW_DATA_PTR_SLOT,
            DATA_VIEW_BYTE_OFFSET_SLOT,
            DATA_VIEW_BYTE_LENGTH_SLOT,
            "EvalError",
            "AggregateError",
            "RangeError",
            "SyntaxError",
            "TypeError",
            "URIError",
            "ReferenceError",
            "class constructor cannot be invoked without `new`",
            "dynamic Function constructor unsupported",
            "Function.prototype.call receiver is not callable",
            "Function.prototype.apply receiver is not callable",
            "Function.prototype.bind receiver is not callable",
            "Function.prototype.toString receiver is not callable",
            "Function.prototype.call primitive thisArg boxing unsupported",
            "Function.prototype.apply primitive thisArg boxing unsupported",
            "Function.prototype.call/apply thisArg adaptation failed",
            "Function.prototype.apply argument list must be array or arguments",
            "Error.prototype.toString receiver is not object",
            "AggregateError errors input must be array or arguments",
            "DataView getUint8 index out of bounds",
            "right-hand side of `in` is not an object",
            "must call super() before accessing `this`",
            "derived constructor must call super() before returning",
            "derived constructor may only return object or undefined",
            "super() called twice in derived constructor",
            "super() invalid in class extending null",
            "super property access on null base",
            "private field access on wrong object",
            GLOBAL_THIS_NAME,
            PRINT_NAME,
        ] {
            pool.intern_string(value);
        }
        for binding in &script.global_bindings {
            pool.intern_string(&binding.name);
        }
        for meta in function_metas.values() {
            pool.intern_string(&meta.name);
            pool.intern_string(&meta.to_string_value);
        }
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
            StatementIr::Throw(value) => self.collect_expr(value),
            StatementIr::Var(declarators) => self.collect_var_declarators(declarators),
            StatementIr::Block(block) => self.collect_block(block),
            StatementIr::TryCatch {
                try_block,
                catch_block,
                ..
            } => {
                self.collect_block(try_block);
                self.collect_block(catch_block);
            }
            StatementIr::TryFinally {
                try_block,
                finally_block,
            } => {
                self.collect_block(try_block);
                self.collect_block(finally_block);
            }
            StatementIr::TryCatchFinally {
                try_block,
                catch_block,
                finally_block,
                ..
            } => {
                self.collect_block(try_block);
                self.collect_block(catch_block);
                self.collect_block(finally_block);
            }
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
            | ExprIr::LogicalNot { expr: value }
            | ExprIr::Void { expr: value }
            | ExprIr::DeleteValue { expr: value } => self.collect_expr(value),
            ExprIr::DeleteIdentifier { name, .. } | ExprIr::DeleteGlobalProperty { name } => {
                self.uses_heap = true;
                self.intern_string(name);
            }
            ExprIr::DeleteProperty { target, key } => {
                self.uses_heap = true;
                self.collect_expr(target);
                self.collect_property_key(key);
            }
            ExprIr::TypeOf { expr } => {
                self.intern_string("undefined");
                self.intern_string("object");
                self.intern_string("boolean");
                self.intern_string("number");
                self.intern_string("string");
                self.intern_string("function");
                self.collect_expr(expr);
            }
            ExprIr::TypeOfUnresolvedIdentifier { .. } => {
                self.intern_string("undefined");
            }
            ExprIr::NewTarget => {}
            ExprIr::UpdateIdentifier { .. } => {}
            ExprIr::BinaryNumber { lhs, rhs, .. }
            | ExprIr::CoerciveBinaryNumber { lhs, rhs, .. }
            | ExprIr::CompareNumber { lhs, rhs, .. }
            | ExprIr::CompareValue { lhs, rhs, .. }
            | ExprIr::StrictEquality { lhs, rhs, .. }
            | ExprIr::LooseEquality { lhs, rhs, .. }
            | ExprIr::LogicalShortCircuit { lhs, rhs, .. }
            | ExprIr::Comma { lhs, rhs } => {
                for value in [
                    "",
                    ",",
                    "[object Object]",
                    "[object Arguments]",
                    "valueOf",
                    "toString",
                ] {
                    self.intern_string(value);
                }
                self.collect_expr(lhs);
                self.collect_expr(rhs);
            }
            ExprIr::StringConcat { lhs, rhs } => {
                self.uses_heap = true;
                self.intern_string("undefined");
                self.intern_string("null");
                self.intern_string("true");
                self.intern_string("false");
                self.intern_string("NaN");
                self.intern_string("Infinity");
                self.intern_string("-Infinity");
                self.intern_string("[object Object]");
                self.intern_string("[object Arguments]");
                self.intern_string("");
                self.intern_string(",");
                self.collect_expr(lhs);
                self.collect_expr(rhs);
            }
            ExprIr::CoerciveAdd { lhs, rhs } => {
                self.uses_heap = true;
                for value in [
                    "",
                    ",",
                    "undefined",
                    "null",
                    "true",
                    "false",
                    "NaN",
                    "Infinity",
                    "-Infinity",
                    "[object Object]",
                    "[object Arguments]",
                    "valueOf",
                    "toString",
                ] {
                    self.intern_string(value);
                }
                self.collect_expr(lhs);
                self.collect_expr(rhs);
            }
            ExprIr::CallNamed { name, args } => {
                self.uses_heap = true;
                self.intern_string(name);
                for arg in args {
                    self.collect_expr(arg);
                }
            }
            ExprIr::GlobalPropertyRead { name } => {
                self.uses_heap = true;
                self.intern_string(name);
            }
            ExprIr::GlobalPropertyWrite { name, value, .. } => {
                self.uses_heap = true;
                self.intern_string(name);
                self.collect_expr(value);
            }
            ExprIr::GlobalPropertyUpdate { name, .. } => {
                self.uses_heap = true;
                self.intern_string(name);
            }
            ExprIr::GlobalPropertyCompoundAssign { name, value, .. } => {
                self.uses_heap = true;
                self.intern_string(name);
                self.collect_expr(value);
            }
            ExprIr::CallIndirect { callee, args, .. } => {
                self.uses_heap = true;
                self.collect_expr(callee);
                for arg in args {
                    self.collect_expr(arg);
                }
            }
            ExprIr::Construct { callee, args } => {
                self.uses_heap = true;
                self.intern_string("prototype");
                self.collect_expr(callee);
                for arg in args {
                    self.collect_expr(arg);
                }
            }
            ExprIr::CallMethod {
                receiver,
                key,
                args,
            } => {
                self.uses_heap = true;
                self.collect_expr(receiver);
                self.collect_property_key(key);
                for arg in args {
                    self.collect_expr(arg);
                }
            }
            ExprIr::InstanceOf { lhs, rhs } => {
                self.uses_heap = true;
                self.intern_string("prototype");
                self.collect_expr(lhs);
                self.collect_expr(rhs);
            }
            ExprIr::In { lhs, rhs } => {
                self.uses_heap = true;
                self.collect_expr(lhs);
                self.collect_expr(rhs);
            }
            ExprIr::SuperConstruct { args } => {
                self.uses_heap = true;
                for arg in args {
                    self.collect_expr(arg);
                }
            }
            ExprIr::SuperPropertyRead { key } => {
                self.uses_heap = true;
                self.collect_property_key(key);
            }
            ExprIr::SuperPropertyWrite { key, value } => {
                self.uses_heap = true;
                self.collect_property_key(key);
                self.collect_expr(value);
            }
            ExprIr::ClassDefinition(_)
            | ExprIr::PrivateRead { .. }
            | ExprIr::PrivateWrite { .. }
            | ExprIr::PrivateIn { .. } => {
                self.uses_heap = true;
                if let ExprIr::ClassDefinition(class) = &expr.expr {
                    self.intern_string("prototype");
                    self.intern_string("constructor");
                    for method in &class.public_methods {
                        self.intern_string(&method.key);
                    }
                    for method in &class.private_methods {
                        self.intern_string(&private_data_key(method.private_name_id));
                        self.intern_string(&private_brand_key(method.private_name_id));
                    }
                    for field in &class.fields {
                        if let Some(key) = &field.key {
                            self.intern_string(key);
                        } else if let Some(private_name_id) = field.private_name_id {
                            self.intern_string(&private_data_key(private_name_id));
                            self.intern_string(&private_brand_key(private_name_id));
                        }
                    }
                    if let Some(heritage) = &class.heritage {
                        self.collect_expr(heritage);
                    }
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
        let offset = STATIC_DATA_OFFSET + self.bytes.len() as u32;
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
        let string = self
            .refs
            .get(value)
            .unwrap_or_else(|| panic!("string `{value}` must exist in pool"));
        (((string.offset as u64) << 32) | string.len as u64) as i64
    }
}

fn align_heap_start(bytes: usize) -> u64 {
    ((STATIC_DATA_OFFSET as u64 + bytes as u64) + 7) & !7
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompletionKind {
    Normal,
    Throw,
    Return,
    Break,
    Continue,
}

impl CompletionKind {
    const fn code(self) -> i64 {
        match self {
            Self::Normal => COMPLETION_KIND_NORMAL,
            Self::Throw => COMPLETION_KIND_THROW,
            Self::Return => COMPLETION_KIND_RETURN,
            Self::Break => COMPLETION_KIND_BREAK,
            Self::Continue => COMPLETION_KIND_CONTINUE,
        }
    }
}

#[derive(Debug, Clone)]
struct WasmFunctionMeta {
    name: String,
    to_string_value: String,
    wasm_index: u32,
    table_index: u32,
    constructable: bool,
    class_kind: ClassFunctionKind,
    class_heritage_kind: ClassHeritageKind,
    is_static_class_member: bool,
    is_derived_constructor: bool,
    is_synthetic_default_derived_constructor: bool,
    super_constructor_target: Option<FunctionId>,
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
    script_global_bindings: BTreeMap<String, ScriptGlobalBindingKind>,
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
    completion_local: u32,
    completion_aux_local: u32,
    derived_this_initialized_local: Option<u32>,
    scratch_local: u32,
    temp_local_base: u32,
    temp_stack_depth: u32,
    this_payload_local: Option<u32>,
    this_tag_local: Option<u32>,
    control_stack: Vec<ControlFrameKind>,
    breakable_stack: Vec<usize>,
    loop_stack: Vec<LoopTargets>,
    label_stack: Vec<LabelTargets>,
    throw_handler_stack: Vec<usize>,
    finally_stack: Vec<usize>,
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
            BTreeMap::new(),
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
    ) -> Self {
        Self::new(
            &EMPTY_BLOCK,
            &[],
            &[],
            &[],
            strings,
            functions,
            Some(builtin.function_id()),
            FunctionFlavor::Ordinary,
            None,
            BTreeMap::new(),
            uses_heap,
            ReturnAbi::MultiValue,
            false,
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
        }
    }

    fn local_count(&self) -> usize {
        self.total_binding_local_count as usize
            + 6
            + usize::from(self.derived_this_initialized_local.is_some())
            + self.temp_local_count as usize
    }

    const fn is_main(&self) -> bool {
        matches!(self.return_abi, ReturnAbi::MainExport)
    }

    fn is_script_global_binding(&self, name: &str) -> bool {
        self.is_main()
            && self
                .script_global_bindings
                .get(name)
                .is_some_and(|kind| *kind != ScriptGlobalBindingKind::Intrinsic)
    }

    fn emit_alloc_plain_object_with_prototype(
        &mut self,
        prototype_local: Option<u32>,
        prototype_global: Option<u32>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        self.emit_heap_alloc_const(HEAP_HEADER_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(object_local));
        self.emit_heap_alloc_const(MIN_HEAP_CAPACITY * HEAP_OBJECT_ENTRY_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(buffer_local));
        self.store_i64_local_at_offset(object_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.store_i64_const_at_offset(object_local, HEAP_LEN_OFFSET, 0, function);
        self.store_i64_const_at_offset(object_local, HEAP_CAP_OFFSET, MIN_HEAP_CAPACITY, function);
        if let Some(prototype_local) = prototype_local {
            self.store_i64_local_at_offset(
                object_local,
                HEAP_PROTOTYPE_OFFSET,
                prototype_local,
                function,
            );
        } else if let Some(prototype_global) = prototype_global {
            function.instruction(&Instruction::GlobalGet(prototype_global));
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.store_i64_local_at_offset(
                object_local,
                HEAP_PROTOTYPE_OFFSET,
                self.scratch_local,
                function,
            );
        } else {
            self.store_i64_const_at_offset(object_local, HEAP_PROTOTYPE_OFFSET, 0, function);
        }
        function.instruction(&Instruction::LocalGet(object_local));
        self.release_temp_local(buffer_local);
        self.release_temp_local(object_local);
        Ok(())
    }

    fn emit_store_boxed_primitive_metadata(
        &mut self,
        object_local: u32,
        boxed_kind: u64,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) {
        self.store_i64_const_at_offset(
            object_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            boxed_kind,
            function,
        );
        self.store_i64_local_at_offset(
            object_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            value_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            object_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
    }

    fn emit_boxed_string_length_number_payload(
        &mut self,
        string_payload_local: u32,
        number_payload_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(string_payload_local));
        function.instruction(&Instruction::I64Const(0xFFFF_FFFFu64 as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(number_payload_local));
    }

    fn emit_alloc_boxed_wrapper_from_locals(
        &mut self,
        prototype_global_index: u32,
        boxed_kind: u64,
        value_payload_local: u32,
        value_tag_local: u32,
        result_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object_local = self.reserve_temp_local();
        self.emit_alloc_plain_object_with_prototype(None, Some(prototype_global_index), function)?;
        function.instruction(&Instruction::LocalSet(object_local));
        self.emit_store_boxed_primitive_metadata(
            object_local,
            boxed_kind,
            value_payload_local,
            value_tag_local,
            function,
        );
        if boxed_kind == BOXED_PRIMITIVE_KIND_STRING {
            let key_local = self.reserve_temp_local();
            let length_payload_local = self.reserve_temp_local();
            let length_tag_local = self.reserve_temp_local();
            function.instruction(&Instruction::I64Const(self.strings.payload("length")));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_boxed_string_length_number_payload(
                value_payload_local,
                length_payload_local,
                function,
            );
            function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
            function.instruction(&Instruction::LocalSet(length_tag_local));
            self.emit_object_define_data(
                object_local,
                key_local,
                length_payload_local,
                length_tag_local,
                function,
            )?;
            self.release_temp_local(length_tag_local);
            self.release_temp_local(length_payload_local);
            self.release_temp_local(key_local);
        }
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalSet(result_payload_local));
        self.release_temp_local(object_local);
        Ok(())
    }

    fn emit_alloc_boxed_wrapper_for_builtin(
        &mut self,
        builtin: StandardBuiltinId,
        value_payload_local: u32,
        value_tag_local: u32,
        result_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let prototype_global_index = boxed_prototype_global_index(builtin).ok_or_else(|| {
            EmitError::unsupported(format!(
                "unsupported in porffor wasm-aot first slice: missing boxed prototype `{}`",
                builtin.debug_name()
            ))
        })?;
        let boxed_kind = boxed_primitive_kind_tag(builtin).ok_or_else(|| {
            EmitError::unsupported(format!(
                "unsupported in porffor wasm-aot first slice: missing boxed primitive kind `{}`",
                builtin.debug_name()
            ))
        })?;
        self.emit_alloc_boxed_wrapper_from_locals(
            prototype_global_index,
            boxed_kind,
            value_payload_local,
            value_tag_local,
            result_payload_local,
            function,
        )
    }

    fn emit_object_define_string_data(
        &mut self,
        object_local: u32,
        key: &str,
        value: &str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(self.strings.payload(key)));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload(value)));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_define_data(object_local, key_local, payload_local, tag_local, function)?;
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);
        Ok(())
    }

    fn emit_object_define_function_data(
        &mut self,
        object_local: u32,
        key: &str,
        meta: &WasmFunctionMeta,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(self.strings.payload(key)));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_function_value_payload(meta, function)?;
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_define_data(object_local, key_local, payload_local, tag_local, function)?;
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);
        Ok(())
    }

    fn emit_object_define_number_data_from_i64_local(
        &mut self,
        object_local: u32,
        key: &str,
        value_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(self.strings.payload(key)));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_define_data(object_local, key_local, payload_local, tag_local, function)?;
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);
        Ok(())
    }

    fn emit_object_read_number_slot_to_i64_local(
        &mut self,
        object_local: u32,
        key: &str,
        dest_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(object_tag_local));
        function.instruction(&Instruction::I64Const(self.strings.payload(key)));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            object_local,
            object_tag_local,
            object_local,
            object_tag_local,
            key_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(dest_local));
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(object_tag_local);
        Ok(())
    }

    fn init_builtin_constructor_object(
        &mut self,
        builtin: StandardBuiltinId,
        prototype_global_index: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
            EmitError::unsupported(format!(
                "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                builtin.debug_name()
            ))
        })?;
        let constructor_global_index =
            standard_builtin_constructor_global_index(builtin).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: missing builtin constructor global `{}`",
                    builtin.debug_name()
                ))
            })?;
        let object_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();

        self.emit_function_value_payload(meta, function)?;
        function.instruction(&Instruction::LocalSet(object_local));
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::GlobalSet(constructor_global_index));

        if builtin.constructable() {
            self.store_i64_const_at_offset(
                object_local,
                HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
                ValueKind::Object.tag() as u64,
                function,
            );
            function.instruction(&Instruction::GlobalGet(prototype_global_index));
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.store_i64_local_at_offset(
                object_local,
                HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
                self.scratch_local,
                function,
            );
            function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
            function.instruction(&Instruction::LocalSet(key_local));
            function.instruction(&Instruction::GlobalGet(prototype_global_index));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            self.emit_object_define_data(
                object_local,
                key_local,
                payload_local,
                tag_local,
                function,
            )?;

            function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
            function.instruction(&Instruction::LocalSet(key_local));
            function.instruction(&Instruction::LocalGet(object_local));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            function.instruction(&Instruction::GlobalGet(prototype_global_index));
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.emit_object_define_data(
                self.scratch_local,
                key_local,
                payload_local,
                tag_local,
                function,
            )?;
        }

        match builtin {
            StandardBuiltinId::FunctionConstructor => {
                let prototype_object_local = self.reserve_temp_local();
                let call_meta = self
                    .functions
                    .get(&StandardBuiltinId::FunctionPrototypeCall.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Function.prototype.call`",
                        )
                    })?;
                function.instruction(&Instruction::GlobalGet(prototype_global_index));
                function.instruction(&Instruction::LocalSet(prototype_object_local));
                self.emit_object_define_function_data(
                    prototype_object_local,
                    "call",
                    call_meta,
                    function,
                )?;
                let apply_meta = self
                    .functions
                    .get(&StandardBuiltinId::FunctionPrototypeApply.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Function.prototype.apply`",
                        )
                    })?;
                function.instruction(&Instruction::GlobalGet(prototype_global_index));
                function.instruction(&Instruction::LocalSet(prototype_object_local));
                self.emit_object_define_function_data(
                    prototype_object_local,
                    "apply",
                    apply_meta,
                    function,
                )?;
                let bind_meta = self
                    .functions
                    .get(&StandardBuiltinId::FunctionPrototypeBind.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Function.prototype.bind`",
                        )
                    })?;
                function.instruction(&Instruction::GlobalGet(prototype_global_index));
                function.instruction(&Instruction::LocalSet(prototype_object_local));
                self.emit_object_define_function_data(
                    prototype_object_local,
                    "bind",
                    bind_meta,
                    function,
                )?;
                let to_string_meta = self
                    .functions
                    .get(&StandardBuiltinId::FunctionPrototypeToString.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Function.prototype.toString`",
                        )
                    })?;
                function.instruction(&Instruction::GlobalGet(prototype_global_index));
                function.instruction(&Instruction::LocalSet(prototype_object_local));
                self.emit_object_define_function_data(
                    prototype_object_local,
                    "toString",
                    to_string_meta,
                    function,
                )?;
                self.release_temp_local(prototype_object_local);
            }
            StandardBuiltinId::ObjectConstructor => {
                let create_meta = self
                    .functions
                    .get(&StandardBuiltinId::ObjectCreate.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.create`",
                        )
                    })?;
                self.emit_object_define_function_data(
                    object_local,
                    "create",
                    create_meta,
                    function,
                )?;
                let get_proto_meta = self
                    .functions
                    .get(&StandardBuiltinId::ObjectGetPrototypeOf.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.getPrototypeOf`",
                        )
                    })?;
                self.emit_object_define_function_data(
                    object_local,
                    "getPrototypeOf",
                    get_proto_meta,
                    function,
                )?;
            }
            StandardBuiltinId::ArrayConstructor => {
                let is_array_meta = self
                    .functions
                    .get(&StandardBuiltinId::ArrayIsArray.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.isArray`",
                        )
                    })?;
                self.emit_object_define_function_data(
                    object_local,
                    "isArray",
                    is_array_meta,
                    function,
                )?;
            }
            StandardBuiltinId::DataViewConstructor => {
                let prototype_object_local = self.reserve_temp_local();
                let get_uint8_meta = self
                    .functions
                    .get(&StandardBuiltinId::DataViewPrototypeGetUint8.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `DataView.prototype.getUint8`",
                        )
                    })?;
                function.instruction(&Instruction::GlobalGet(prototype_global_index));
                function.instruction(&Instruction::LocalSet(prototype_object_local));
                self.emit_object_define_function_data(
                    prototype_object_local,
                    "getUint8",
                    get_uint8_meta,
                    function,
                )?;
                self.release_temp_local(prototype_object_local);
            }
            StandardBuiltinId::ErrorConstructor => {
                let prototype_object_local = self.reserve_temp_local();
                let to_string_meta = self
                    .functions
                    .get(&StandardBuiltinId::ErrorPrototypeToString.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Error.prototype.toString`",
                        )
                    })?;
                function.instruction(&Instruction::GlobalGet(prototype_global_index));
                function.instruction(&Instruction::LocalSet(prototype_object_local));
                self.emit_object_define_function_data(
                    prototype_object_local,
                    "toString",
                    to_string_meta,
                    function,
                )?;
                self.release_temp_local(prototype_object_local);
            }
            StandardBuiltinId::NumberConstructor
            | StandardBuiltinId::StringConstructor
            | StandardBuiltinId::BooleanConstructor
            | StandardBuiltinId::ArrayBufferConstructor
            | StandardBuiltinId::EvalErrorConstructor
            | StandardBuiltinId::AggregateErrorConstructor
            | StandardBuiltinId::RangeErrorConstructor
            | StandardBuiltinId::SyntaxErrorConstructor
            | StandardBuiltinId::TypeErrorConstructor
            | StandardBuiltinId::URIErrorConstructor
            | StandardBuiltinId::ReferenceErrorConstructor
            | StandardBuiltinId::FunctionPrototypeCall
            | StandardBuiltinId::FunctionPrototypeApply
            | StandardBuiltinId::FunctionPrototypeBind
            | StandardBuiltinId::FunctionPrototypeToString
            | StandardBuiltinId::ObjectCreate
            | StandardBuiltinId::ObjectGetPrototypeOf
            | StandardBuiltinId::ArrayIsArray
            | StandardBuiltinId::DataViewPrototypeGetUint8
            | StandardBuiltinId::ErrorPrototypeToString
            | StandardBuiltinId::BoundFunctionInvoker => {}
        }

        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(object_local);
        Ok(())
    }

    fn init_runtime_roots(&mut self, function: &mut Function) -> Result<(), EmitError> {
        if !self.is_main() {
            return Ok(());
        }
        self.emit_alloc_plain_object_with_prototype(None, None, function)?;
        function.instruction(&Instruction::GlobalSet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(FUNCTION_PROTOTYPE_GLOBAL_INDEX));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(ARRAY_PROTOTYPE_GLOBAL_INDEX));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(NUMBER_PROTOTYPE_GLOBAL_INDEX));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(STRING_PROTOTYPE_GLOBAL_INDEX));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(BOOLEAN_PROTOTYPE_GLOBAL_INDEX));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(ERROR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::GlobalGet(ERROR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_object_define_string_data(self.scratch_local, "name", ERROR_NAME, function)?;
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(ERROR_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::GlobalGet(TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_object_define_string_data(self.scratch_local, "name", TYPE_ERROR_NAME, function)?;
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(ERROR_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            REFERENCE_ERROR_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::GlobalGet(
            REFERENCE_ERROR_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_object_define_string_data(
            self.scratch_local,
            "name",
            REFERENCE_ERROR_NAME,
            function,
        )?;
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(ERROR_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(EVAL_ERROR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::GlobalGet(EVAL_ERROR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_object_define_string_data(self.scratch_local, "name", EVAL_ERROR_NAME, function)?;
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(ERROR_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            AGGREGATE_ERROR_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::GlobalGet(
            AGGREGATE_ERROR_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_object_define_string_data(
            self.scratch_local,
            "name",
            AGGREGATE_ERROR_NAME,
            function,
        )?;
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(ERROR_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(RANGE_ERROR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::GlobalGet(RANGE_ERROR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_object_define_string_data(
            self.scratch_local,
            "name",
            RANGE_ERROR_NAME,
            function,
        )?;
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(ERROR_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(SYNTAX_ERROR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::GlobalGet(SYNTAX_ERROR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_object_define_string_data(
            self.scratch_local,
            "name",
            SYNTAX_ERROR_NAME,
            function,
        )?;
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(ERROR_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(URI_ERROR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::GlobalGet(URI_ERROR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_object_define_string_data(self.scratch_local, "name", URI_ERROR_NAME, function)?;
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(ARRAY_BUFFER_PROTOTYPE_GLOBAL_INDEX));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(DATA_VIEW_PROTOTYPE_GLOBAL_INDEX));
        self.init_builtin_constructor_object(
            StandardBuiltinId::FunctionConstructor,
            FUNCTION_PROTOTYPE_GLOBAL_INDEX,
            function,
        )?;
        self.init_builtin_constructor_object(
            StandardBuiltinId::ObjectConstructor,
            OBJECT_PROTOTYPE_GLOBAL_INDEX,
            function,
        )?;
        self.init_builtin_constructor_object(
            StandardBuiltinId::ArrayConstructor,
            ARRAY_PROTOTYPE_GLOBAL_INDEX,
            function,
        )?;
        self.init_builtin_constructor_object(
            StandardBuiltinId::ArrayBufferConstructor,
            ARRAY_BUFFER_PROTOTYPE_GLOBAL_INDEX,
            function,
        )?;
        self.init_builtin_constructor_object(
            StandardBuiltinId::DataViewConstructor,
            DATA_VIEW_PROTOTYPE_GLOBAL_INDEX,
            function,
        )?;
        self.init_builtin_constructor_object(
            StandardBuiltinId::NumberConstructor,
            NUMBER_PROTOTYPE_GLOBAL_INDEX,
            function,
        )?;
        self.init_builtin_constructor_object(
            StandardBuiltinId::StringConstructor,
            STRING_PROTOTYPE_GLOBAL_INDEX,
            function,
        )?;
        self.init_builtin_constructor_object(
            StandardBuiltinId::BooleanConstructor,
            BOOLEAN_PROTOTYPE_GLOBAL_INDEX,
            function,
        )?;
        self.init_builtin_constructor_object(
            StandardBuiltinId::ErrorConstructor,
            ERROR_PROTOTYPE_GLOBAL_INDEX,
            function,
        )?;
        self.init_builtin_constructor_object(
            StandardBuiltinId::EvalErrorConstructor,
            EVAL_ERROR_PROTOTYPE_GLOBAL_INDEX,
            function,
        )?;
        self.init_builtin_constructor_object(
            StandardBuiltinId::AggregateErrorConstructor,
            AGGREGATE_ERROR_PROTOTYPE_GLOBAL_INDEX,
            function,
        )?;
        self.init_builtin_constructor_object(
            StandardBuiltinId::RangeErrorConstructor,
            RANGE_ERROR_PROTOTYPE_GLOBAL_INDEX,
            function,
        )?;
        self.init_builtin_constructor_object(
            StandardBuiltinId::SyntaxErrorConstructor,
            SYNTAX_ERROR_PROTOTYPE_GLOBAL_INDEX,
            function,
        )?;
        self.init_builtin_constructor_object(
            StandardBuiltinId::TypeErrorConstructor,
            TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX,
            function,
        )?;
        self.init_builtin_constructor_object(
            StandardBuiltinId::URIErrorConstructor,
            URI_ERROR_PROTOTYPE_GLOBAL_INDEX,
            function,
        )?;
        self.init_builtin_constructor_object(
            StandardBuiltinId::ReferenceErrorConstructor,
            REFERENCE_ERROR_PROTOTYPE_GLOBAL_INDEX,
            function,
        )?;
        Ok(())
    }

    fn init_script_global_object(&mut self, function: &mut Function) -> Result<(), EmitError> {
        if !self.is_main() {
            return Ok(());
        }

        let object_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        let capacity = (self.script_global_bindings.len() as u64).max(MIN_HEAP_CAPACITY);

        self.emit_heap_alloc_const(HEAP_HEADER_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(object_local));
        self.emit_heap_alloc_const(capacity * HEAP_OBJECT_ENTRY_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(buffer_local));
        self.store_i64_local_at_offset(object_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.store_i64_const_at_offset(object_local, HEAP_LEN_OFFSET, 0, function);
        self.store_i64_const_at_offset(object_local, HEAP_CAP_OFFSET, capacity, function);
        function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.store_i64_local_at_offset(
            object_local,
            HEAP_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.store_i64_local_at_offset(
            object_local,
            HEAP_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::GlobalSet(SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX));

        for binding in self
            .script_global_bindings
            .clone()
            .into_iter()
            .map(|(name, kind)| ScriptGlobalBindingIr { name, kind })
        {
            function.instruction(&Instruction::I64Const(self.strings.payload(&binding.name)));
            function.instruction(&Instruction::LocalSet(key_local));
            match binding.kind {
                ScriptGlobalBindingKind::Intrinsic => {
                    function.instruction(&Instruction::LocalGet(object_local));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
                ScriptGlobalBindingKind::Var => {
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
                ScriptGlobalBindingKind::Function => {
                    let meta = self.functions.values().find(|meta| meta.name == binding.name).ok_or_else(
                        || {
                            EmitError::unsupported(format!(
                                "unsupported in porffor wasm-aot first slice: unknown script-global function `{}`",
                                binding.name
                            ))
                        },
                    )?;
                    self.emit_function_value_payload(meta, function)?;
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
                ScriptGlobalBindingKind::BuiltinFunction(builtin) => {
                    let global_index =
                        standard_builtin_constructor_global_index(builtin).ok_or_else(|| {
                            EmitError::unsupported(format!(
                                "unsupported in porffor wasm-aot first slice: non-global builtin `{}`",
                                builtin.debug_name()
                            ))
                        })?;
                    function.instruction(&Instruction::GlobalGet(global_index));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
                ScriptGlobalBindingKind::HostFunction(builtin) => {
                    let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "unsupported in porffor wasm-aot first slice: unknown script-global host function `{}`",
                            builtin.as_str()
                        ))
                    })?;
                    self.emit_function_value_payload(meta, function)?;
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
            }
            self.emit_object_define_data_with_configurable(
                object_local,
                key_local,
                payload_local,
                tag_local,
                !matches!(
                    binding.kind,
                    ScriptGlobalBindingKind::Intrinsic
                        | ScriptGlobalBindingKind::Var
                        | ScriptGlobalBindingKind::Function
                ),
                function,
            )?;
        }

        if let Some(slot) = self.owned_env_slot(LEXICAL_THIS_NAME) {
            function.instruction(&Instruction::LocalGet(object_local));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            self.write_env_slot_from_locals(slot, 0, payload_local, tag_local, function);
        }

        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(object_local);
        Ok(())
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
        }
        self.compile_block_contents(self.body, &mut function)?;
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
            self.compile_standard_builtin(builtin, &mut function)?;
        } else {
            match HostBuiltinId::from_function_id(&function_id) {
                Some(HostBuiltinId::Print) => self.compile_host_print_builtin(&mut function)?,
                Some(HostBuiltinId::Gc) => self.compile_host_gc_builtin(&mut function),
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
        self.emit_function_value_payload(meta, function)?;
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
                if let Some(derived_this_initialized_local) = self.derived_this_initialized_local {
                    function.instruction(&Instruction::LocalGet(derived_this_initialized_local));
                    function.instruction(&Instruction::I64Eqz);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_throw_runtime_error(
                        "ReferenceError",
                        "must call super() before accessing `this`",
                        self.result_local,
                        self.result_tag_local,
                        function,
                    )?;
                    if let Some(target) = self.throw_handler_stack.last() {
                        function.instruction(&Instruction::Br(self.depth_to(*target) + 1));
                    } else {
                        self.emit_return_current_completion(function);
                    }
                    function.instruction(&Instruction::End);
                }
                if let Some(this_payload_local) = self.this_payload_local {
                    function.instruction(&Instruction::LocalGet(this_payload_local));
                } else if self.is_main() {
                    function
                        .instruction(&Instruction::GlobalGet(SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX));
                } else {
                    return Err(EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: top-level `this`",
                    ));
                }
            }
            FunctionFlavor::Arrow => {
                if let Some(storage) = self.lookup_binding(LEXICAL_THIS_NAME) {
                    self.read_binding_payload(storage, function);
                } else {
                    function
                        .instruction(&Instruction::GlobalGet(SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX));
                }
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
                if let Some(derived_this_initialized_local) = self.derived_this_initialized_local {
                    function.instruction(&Instruction::LocalGet(derived_this_initialized_local));
                    function.instruction(&Instruction::I64Eqz);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_throw_runtime_error(
                        "ReferenceError",
                        "must call super() before accessing `this`",
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    if let Some(target) = self.throw_handler_stack.last() {
                        function.instruction(&Instruction::Br(self.depth_to(*target) + 1));
                    } else {
                        self.emit_return_current_completion(function);
                    }
                    function.instruction(&Instruction::End);
                }
                if let (Some(this_payload_local), Some(this_tag_local)) =
                    (self.this_payload_local, self.this_tag_local)
                {
                    function.instruction(&Instruction::LocalGet(this_payload_local));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::LocalGet(this_tag_local));
                    function.instruction(&Instruction::LocalSet(tag_local));
                } else if self.is_main() {
                    function
                        .instruction(&Instruction::GlobalGet(SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                } else {
                    return Err(EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: missing `this` tag local",
                    ));
                }
            }
            FunctionFlavor::Arrow => {
                if let Some(storage) = self.lookup_binding(LEXICAL_THIS_NAME) {
                    self.read_binding_to_locals(storage, payload_local, tag_local, function);
                } else {
                    function
                        .instruction(&Instruction::GlobalGet(SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
            }
        }
        Ok(())
    }

    const fn argc_param_local(&self) -> u32 {
        5
    }

    const fn argv_param_local(&self) -> u32 {
        6
    }

    const fn new_target_payload_local(&self) -> Option<u32> {
        if matches!(self.return_abi, ReturnAbi::MultiValue) {
            Some(3)
        } else {
            None
        }
    }

    const fn new_target_tag_local(&self) -> Option<u32> {
        if matches!(self.return_abi, ReturnAbi::MultiValue) {
            Some(4)
        } else {
            None
        }
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
                self.mirror_binding_to_global_object(name, storage, function)?;
                self.emit_statement_result(function, ValueKind::Undefined);
            }
            StatementIr::Expression(expr) => {
                if !expr.possible_kinds.is_singleton() {
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
            StatementIr::Throw(value) => {
                self.compile_expr_to_locals(
                    value,
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.set_completion_kind(CompletionKind::Throw, function);
                if let Some(target) = self.throw_handler_stack.last() {
                    function.instruction(&Instruction::Br(self.depth_to(*target)));
                } else if let Some(target) = self.finally_stack.last() {
                    function.instruction(&Instruction::Br(self.depth_to(*target)));
                } else {
                    self.emit_return_current_completion(function);
                }
            }
            StatementIr::TryCatch {
                try_block,
                catch_name,
                catch_block,
            } => {
                self.compile_try_catch(try_block, catch_name, catch_block, function)?;
            }
            StatementIr::TryFinally {
                try_block,
                finally_block,
            } => {
                self.compile_try_finally(try_block, finally_block, function)?;
            }
            StatementIr::TryCatchFinally {
                try_block,
                catch_name,
                catch_block,
                finally_block,
            } => {
                self.compile_try_catch_finally(
                    try_block,
                    catch_name,
                    catch_block,
                    finally_block,
                    function,
                )?;
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
                if let Some(target) = self.finally_stack.last() {
                    self.set_completion_kind(CompletionKind::Return, function);
                    function.instruction(&Instruction::Br(self.depth_to(*target)));
                } else {
                    self.normalize_derived_constructor_result(function)?;
                    self.set_completion_kind(CompletionKind::Normal, function);
                    self.emit_return_current_completion(function);
                }
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

    fn compile_try_catch(
        &mut self,
        try_block: &BlockIr,
        catch_name: &str,
        catch_block: &BlockIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Block(BlockType::Empty));
        let catch_frame = self.push_control(ControlFrameKind::Block);
        self.throw_handler_stack.push(catch_frame);
        self.push_scope();
        self.compile_block_contents(try_block, function)?;
        self.pop_scope();
        self.throw_handler_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        self.push_scope();
        let catch_storage = self.allocate_dynamic_binding_storage(catch_name);
        self.binding_scopes
            .last_mut()
            .expect("binding scope stack must exist")
            .insert(catch_name.to_string(), catch_storage);
        self.write_binding_from_locals(
            catch_storage,
            self.result_local,
            self.result_tag_local,
            function,
        );
        self.set_completion_kind(CompletionKind::Normal, function);
        self.compile_block_contents(catch_block, function)?;
        self.pop_scope();
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn compile_try_finally(
        &mut self,
        try_block: &BlockIr,
        finally_block: &BlockIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let saved_payload_local = self.reserve_temp_local();
        let saved_tag_local = self.reserve_temp_local();
        let saved_completion_local = self.reserve_temp_local();
        let saved_aux_local = self.reserve_temp_local();

        function.instruction(&Instruction::Block(BlockType::Empty));
        let _outer_frame = self.push_control(ControlFrameKind::Block);
        function.instruction(&Instruction::Block(BlockType::Empty));
        let finally_frame = self.push_control(ControlFrameKind::Block);
        self.finally_stack.push(finally_frame);
        self.push_scope();
        self.compile_block_contents(try_block, function)?;
        self.pop_scope();
        self.finally_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

        self.save_current_completion(
            saved_payload_local,
            saved_tag_local,
            saved_completion_local,
            saved_aux_local,
            function,
        );
        self.set_completion_kind(CompletionKind::Normal, function);
        self.push_scope();
        self.compile_block_contents(finally_block, function)?;
        self.pop_scope();
        self.emit_resume_after_finally(
            saved_payload_local,
            saved_tag_local,
            saved_completion_local,
            saved_aux_local,
            function,
        )?;
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.release_temp_local(saved_aux_local);
        self.release_temp_local(saved_completion_local);
        self.release_temp_local(saved_tag_local);
        self.release_temp_local(saved_payload_local);
        Ok(())
    }

    fn compile_try_catch_finally(
        &mut self,
        try_block: &BlockIr,
        catch_name: &str,
        catch_block: &BlockIr,
        finally_block: &BlockIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let saved_payload_local = self.reserve_temp_local();
        let saved_tag_local = self.reserve_temp_local();
        let saved_completion_local = self.reserve_temp_local();
        let saved_aux_local = self.reserve_temp_local();

        function.instruction(&Instruction::Block(BlockType::Empty));
        let _outer_frame = self.push_control(ControlFrameKind::Block);
        function.instruction(&Instruction::Block(BlockType::Empty));
        let finally_frame = self.push_control(ControlFrameKind::Block);
        function.instruction(&Instruction::Block(BlockType::Empty));
        let catch_skip_frame = self.push_control(ControlFrameKind::Block);
        function.instruction(&Instruction::Block(BlockType::Empty));
        let catch_frame = self.push_control(ControlFrameKind::Block);
        self.throw_handler_stack.push(catch_frame);
        self.finally_stack.push(finally_frame);
        self.push_scope();
        self.compile_block_contents(try_block, function)?;
        self.pop_scope();
        self.throw_handler_stack.pop();
        function.instruction(&Instruction::Br(self.depth_to(catch_skip_frame)));
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

        self.push_scope();
        let catch_storage = self.allocate_dynamic_binding_storage(catch_name);
        self.binding_scopes
            .last_mut()
            .expect("binding scope stack must exist")
            .insert(catch_name.to_string(), catch_storage);
        self.write_binding_from_locals(
            catch_storage,
            self.result_local,
            self.result_tag_local,
            function,
        );
        self.set_completion_kind(CompletionKind::Normal, function);
        self.compile_block_contents(catch_block, function)?;
        self.pop_scope();
        self.finally_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

        self.save_current_completion(
            saved_payload_local,
            saved_tag_local,
            saved_completion_local,
            saved_aux_local,
            function,
        );
        self.set_completion_kind(CompletionKind::Normal, function);
        self.push_scope();
        self.compile_block_contents(finally_block, function)?;
        self.pop_scope();
        self.emit_resume_after_finally(
            saved_payload_local,
            saved_tag_local,
            saved_completion_local,
            saved_aux_local,
            function,
        )?;
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.release_temp_local(saved_aux_local);
        self.release_temp_local(saved_completion_local);
        self.release_temp_local(saved_tag_local);
        self.release_temp_local(saved_payload_local);
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
        if let Some(target) = self.finally_stack.last() {
            self.set_completion_kind_with_aux(CompletionKind::Break, break_frame as i64, function);
            function.instruction(&Instruction::Br(self.depth_to(*target)));
            return Ok(());
        }
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
        if let Some(target) = self.finally_stack.last() {
            self.set_completion_kind_with_aux(
                CompletionKind::Continue,
                continue_frame as i64,
                function,
            );
            function.instruction(&Instruction::Br(self.depth_to(*target)));
            return Ok(());
        }
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
            self.mirror_binding_to_global_object(&declarator.name, storage, function)?;
        }
        Ok(())
    }

    fn compile_expr_payload(
        &mut self,
        expr: &TypedExpr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if !expr.possible_kinds.is_singleton() {
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
                self.emit_function_value_payload(meta, function)?;
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
                if name == GLOBAL_THIS_NAME {
                    function
                        .instruction(&Instruction::GlobalGet(SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX));
                    return Ok(());
                }
                let storage = self.lookup_binding(name).ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: unbound identifier `{name}`"
                    ))
                })?;
                self.read_binding_payload(storage, function);
            }
            ExprIr::GlobalPropertyRead { name } => {
                self.emit_global_property_read(
                    name,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(self.scratch_local));
            }
            ExprIr::AssignIdentifier { name, value } => {
                let storage = self.lookup_binding(name).ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: unbound identifier `{name}`"
                    ))
                })?;
                self.compile_expr_to_binding(value, storage, function)?;
                self.mirror_binding_to_global_object(name, storage, function)?;
                self.read_binding_payload(storage, function);
            }
            ExprIr::GlobalPropertyWrite { name, value, .. } => {
                let value_local = self.reserve_temp_local();
                let tag_local = self.reserve_temp_local();
                self.compile_expr_to_locals(value, value_local, tag_local, function)?;
                self.emit_global_property_write(name, value_local, tag_local, function)?;
                function.instruction(&Instruction::LocalGet(value_local));
                self.release_temp_local(tag_local);
                self.release_temp_local(value_local);
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
                        function
                            .instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                        function.instruction(&Instruction::LocalSet(self.result_tag_local));
                        self.write_binding_from_locals(
                            storage,
                            self.scratch_local,
                            self.result_tag_local,
                            function,
                        );
                        self.mirror_binding_to_global_object(name, storage, function)?;
                        function.instruction(&Instruction::LocalGet(self.scratch_local));
                    }
                    UpdateReturnMode::Postfix => {
                        function.instruction(&Instruction::LocalGet(value_local));
                        function.instruction(&Instruction::LocalSet(self.scratch_local));
                        function.instruction(&Instruction::LocalGet(value_local));
                        self.emit_update_delta(*op, function);
                        function.instruction(&Instruction::LocalSet(value_local));
                        function
                            .instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                        function.instruction(&Instruction::LocalSet(tag_local));
                        self.write_binding_from_locals(storage, value_local, tag_local, function);
                        self.mirror_binding_to_global_object(name, storage, function)?;
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
                self.mirror_binding_to_global_object(name, storage, function)?;
                function.instruction(&Instruction::LocalGet(self.scratch_local));
                self.release_temp_local(tag_local);
                self.release_temp_local(temp_local);
            }
            ExprIr::GlobalPropertyUpdate {
                name,
                op,
                return_mode,
            } => {
                let value_local = self.reserve_temp_local();
                let tag_local = self.reserve_temp_local();
                self.emit_global_property_read(name, value_local, tag_local, function)?;
                match return_mode {
                    UpdateReturnMode::Prefix => {
                        function.instruction(&Instruction::LocalGet(value_local));
                        self.emit_update_delta(*op, function);
                        function.instruction(&Instruction::LocalSet(self.scratch_local));
                        function
                            .instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                        function.instruction(&Instruction::LocalSet(self.result_tag_local));
                        self.emit_global_property_write(
                            name,
                            self.scratch_local,
                            self.result_tag_local,
                            function,
                        )?;
                        function.instruction(&Instruction::LocalGet(self.scratch_local));
                    }
                    UpdateReturnMode::Postfix => {
                        function.instruction(&Instruction::LocalGet(value_local));
                        function.instruction(&Instruction::LocalSet(self.scratch_local));
                        function.instruction(&Instruction::LocalGet(value_local));
                        self.emit_update_delta(*op, function);
                        function.instruction(&Instruction::LocalSet(value_local));
                        function
                            .instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                        function.instruction(&Instruction::LocalSet(tag_local));
                        self.emit_global_property_write(name, value_local, tag_local, function)?;
                        function.instruction(&Instruction::LocalGet(self.scratch_local));
                    }
                }
                self.release_temp_local(tag_local);
                self.release_temp_local(value_local);
            }
            ExprIr::GlobalPropertyCompoundAssign { name, op, value } => {
                let temp_local = self.reserve_temp_local();
                let tag_local = self.reserve_temp_local();
                self.emit_global_property_read(name, temp_local, tag_local, function)?;
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
                self.emit_global_property_write(name, self.scratch_local, tag_local, function)?;
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
            ExprIr::Void { expr } => {
                self.compile_expr_to_locals(
                    expr,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_undefined_payload(function);
            }
            ExprIr::DeleteValue { expr } => {
                self.compile_expr_to_locals(
                    expr,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::I64Const(1));
            }
            ExprIr::DeleteIdentifier { kind, .. } => {
                let value = if matches!(kind, DeleteIdentifierKindIr::NonDeletable) {
                    0
                } else {
                    1
                };
                function.instruction(&Instruction::I64Const(value));
            }
            ExprIr::DeleteGlobalProperty { name } => {
                let result_local = self.reserve_temp_local();
                self.emit_global_property_delete(name, result_local, function)?;
                function.instruction(&Instruction::LocalGet(result_local));
                self.release_temp_local(result_local);
            }
            ExprIr::DeleteProperty { target, key } => {
                self.compile_delete_property_i32(target, key, function)?;
                function.instruction(&Instruction::I64ExtendI32U);
            }
            ExprIr::TypeOf { expr } => {
                self.compile_typeof_payload(expr, function)?;
            }
            ExprIr::TypeOfUnresolvedIdentifier { .. } => {
                function.instruction(&Instruction::I64Const(self.strings.payload("undefined")));
            }
            ExprIr::NewTarget => {
                self.compile_new_target_to_locals(
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                );
                function.instruction(&Instruction::LocalGet(self.scratch_local));
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
            ExprIr::CoerciveBinaryNumber { op, lhs, rhs } => {
                let numeric_only = lhs
                    .possible_kinds
                    .is_subset_of(KindSet::PRIMITIVE_ONLY.without(ValueKind::String))
                    && rhs
                        .possible_kinds
                        .is_subset_of(KindSet::PRIMITIVE_ONLY.without(ValueKind::String));
                if matches!(op, ArithmeticBinaryOp::Mod) {
                    if numeric_only {
                        self.compile_expr_to_number_payload_nonstring(lhs, function)?;
                    } else {
                        self.compile_expr_to_number_payload(lhs, function)?;
                    }
                    function.instruction(&Instruction::LocalSet(self.result_local));
                    if numeric_only {
                        self.compile_expr_to_number_payload_nonstring(rhs, function)?;
                    } else {
                        self.compile_expr_to_number_payload(rhs, function)?;
                    }
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
                    if numeric_only {
                        self.compile_expr_to_number_payload_nonstring(lhs, function)?;
                    } else {
                        self.compile_expr_to_number_payload(lhs, function)?;
                    }
                    function.instruction(&Instruction::F64ReinterpretI64);
                    if numeric_only {
                        self.compile_expr_to_number_payload_nonstring(rhs, function)?;
                    } else {
                        self.compile_expr_to_number_payload(rhs, function)?;
                    }
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
            ExprIr::CoerciveAdd { lhs, rhs } => {
                self.compile_coercive_add_to_locals(
                    lhs,
                    rhs,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(self.scratch_local));
            }
            ExprIr::StringConcat { lhs, rhs } => {
                self.compile_string_concat_payload(lhs, rhs, function)?;
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
            ExprIr::CompareValue { op, lhs, rhs } => {
                if lhs
                    .possible_kinds
                    .is_subset_of(KindSet::PRIMITIVE_ONLY.without(ValueKind::String))
                    && rhs
                        .possible_kinds
                        .is_subset_of(KindSet::PRIMITIVE_ONLY.without(ValueKind::String))
                {
                    self.compile_expr_to_number_payload_nonstring(lhs, function)?;
                    function.instruction(&Instruction::F64ReinterpretI64);
                    self.compile_expr_to_number_payload_nonstring(rhs, function)?;
                    function.instruction(&Instruction::F64ReinterpretI64);
                    match op {
                        RelationalBinaryOp::LessThan => function.instruction(&Instruction::F64Lt),
                        RelationalBinaryOp::LessThanOrEqual => {
                            function.instruction(&Instruction::F64Le)
                        }
                        RelationalBinaryOp::GreaterThan => {
                            function.instruction(&Instruction::F64Gt)
                        }
                        RelationalBinaryOp::GreaterThanOrEqual => {
                            function.instruction(&Instruction::F64Ge)
                        }
                    };
                } else {
                    self.compile_compare_value_i32(*op, lhs, rhs, function)?;
                }
                function.instruction(&Instruction::I64ExtendI32U);
            }
            ExprIr::StrictEquality { op, lhs, rhs } => {
                self.compile_strict_equality_i32(lhs, rhs, function)?;
                if matches!(op, EqualityBinaryOp::StrictNotEqual) {
                    function.instruction(&Instruction::I32Eqz);
                }
                function.instruction(&Instruction::I64ExtendI32U);
            }
            ExprIr::LooseEquality { op, lhs, rhs } => {
                if !lhs.possible_kinds.contains(ValueKind::String)
                    && !rhs.possible_kinds.contains(ValueKind::String)
                {
                    self.compile_loose_equality_nonstring_i32(lhs, rhs, function)?;
                } else {
                    self.compile_loose_equality_i32(lhs, rhs, function)?;
                }
                if matches!(op, EqualityBinaryOp::LooseNotEqual) {
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
                    LogicalBinaryOp::Coalesce => {
                        self.compile_expr_to_locals(
                            rhs,
                            self.scratch_local,
                            self.result_tag_local,
                            function,
                        )?;
                    }
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
                    LogicalBinaryOp::Coalesce => {}
                }
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::LocalGet(self.scratch_local));
            }
            ExprIr::Comma { lhs, rhs } => {
                self.compile_expr_to_locals(
                    lhs,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                self.compile_expr_payload(rhs, function)?;
            }
            ExprIr::CallNamed { name, args } => {
                self.emit_call(
                    name,
                    args,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(self.scratch_local));
            }
            ExprIr::CallIndirect {
                callee,
                this_arg,
                args,
            } => {
                self.emit_indirect_call(
                    callee,
                    this_arg.as_deref(),
                    args,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(self.scratch_local));
            }
            ExprIr::Construct { callee, args } => {
                self.emit_construct(
                    callee,
                    args,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(self.scratch_local));
            }
            ExprIr::ClassDefinition(class) => {
                self.compile_class_definition_payload(class, function)?;
            }
            ExprIr::CallMethod {
                receiver,
                key,
                args,
            } => {
                self.emit_method_call(
                    receiver,
                    key,
                    args,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(self.scratch_local));
            }
            ExprIr::InstanceOf { lhs, rhs } => {
                self.emit_instanceof_i32(lhs, rhs, function)?;
                function.instruction(&Instruction::I64ExtendI32U);
            }
            ExprIr::In { lhs, rhs } => {
                self.emit_in_i32(lhs, rhs, function)?;
                function.instruction(&Instruction::I64ExtendI32U);
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
            ExprIr::SuperConstruct { args } => {
                let (argc_local, argv_local) = self.emit_call_args_vector(args, function)?;
                self.emit_super_construct_with_arg_vector(
                    argc_local,
                    argv_local,
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(self.result_local));
                self.release_temp_local(argv_local);
                self.release_temp_local(argc_local);
            }
            ExprIr::SuperPropertyRead { key } => {
                let super_base_local = self.reserve_temp_local();
                let super_base_tag_local = self.reserve_temp_local();
                self.emit_load_super_base(super_base_local, super_base_tag_local, function)?;
                self.emit_throw_if_null_super_base(
                    super_base_local,
                    super_base_tag_local,
                    function,
                )?;
                let Some(this_payload_local) = self.this_payload_local else {
                    return Err(EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: super outside class method",
                    ));
                };
                let Some(this_tag_local) = self.this_tag_local else {
                    return Err(EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: super outside class method",
                    ));
                };
                let key_local = self.compile_object_key_to_local(key, function)?;
                self.emit_object_read(
                    super_base_local,
                    super_base_tag_local,
                    this_payload_local,
                    this_tag_local,
                    key_local,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                self.release_temp_local(key_local);
                self.release_temp_local(super_base_tag_local);
                self.release_temp_local(super_base_local);
                function.instruction(&Instruction::LocalGet(self.scratch_local));
            }
            ExprIr::SuperPropertyWrite { key, value } => {
                let super_base_local = self.reserve_temp_local();
                let super_base_tag_local = self.reserve_temp_local();
                self.emit_load_super_base(super_base_local, super_base_tag_local, function)?;
                self.emit_throw_if_null_super_base(
                    super_base_local,
                    super_base_tag_local,
                    function,
                )?;
                let key_local = self.compile_object_key_to_local(key, function)?;
                self.compile_expr_to_locals(
                    value,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_object_write(
                    super_base_local,
                    super_base_tag_local,
                    key_local,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                self.release_temp_local(key_local);
                self.release_temp_local(super_base_tag_local);
                self.release_temp_local(super_base_local);
                function.instruction(&Instruction::LocalGet(self.scratch_local));
            }
            ExprIr::PrivateRead {
                target,
                private_name_id,
            } => {
                self.compile_private_read_to_locals(
                    target,
                    *private_name_id,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(self.scratch_local));
            }
            ExprIr::PrivateWrite {
                target,
                private_name_id,
                value,
            } => {
                self.compile_private_write_to_locals(
                    target,
                    *private_name_id,
                    value,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(self.scratch_local));
            }
            ExprIr::PrivateIn {
                private_name_id,
                rhs,
            } => {
                let rhs_payload_local = self.reserve_temp_local();
                let rhs_tag_local = self.reserve_temp_local();
                let key_local = self.reserve_temp_local();
                let read_payload_local = self.reserve_temp_local();
                let read_tag_local = self.reserve_temp_local();
                let result_local = self.reserve_temp_local();

                self.compile_expr_to_locals(rhs, rhs_payload_local, rhs_tag_local, function)?;
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(result_local));
                self.emit_is_heap_object_like_tag_i32(rhs_tag_local, function);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(
                    self.strings.payload(&private_brand_key(*private_name_id)) as i64,
                ));
                function.instruction(&Instruction::LocalSet(key_local));
                self.emit_object_read(
                    rhs_payload_local,
                    rhs_tag_local,
                    rhs_payload_local,
                    rhs_tag_local,
                    key_local,
                    read_payload_local,
                    read_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(read_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::LocalGet(read_payload_local));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I32And);
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::LocalSet(result_local));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::LocalGet(result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));

                self.release_temp_local(result_local);
                self.release_temp_local(read_tag_local);
                self.release_temp_local(read_payload_local);
                self.release_temp_local(key_local);
                self.release_temp_local(rhs_tag_local);
                self.release_temp_local(rhs_payload_local);
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
                self.compile_expr_to_locals(
                    expr,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
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
        if expr.possible_kinds.is_singleton() && !expr_result_tag_is_runtime_dynamic(&expr.expr) {
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
            ExprIr::NewTarget => {
                self.compile_new_target_to_locals(payload_local, tag_local, function);
            }
            ExprIr::Identifier(name) => {
                if name == GLOBAL_THIS_NAME {
                    function
                        .instruction(&Instruction::GlobalGet(SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                    return Ok(());
                }
                let storage = self.lookup_binding(name).ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: unbound identifier `{name}`"
                    ))
                })?;
                self.read_binding_to_locals(storage, payload_local, tag_local, function);
            }
            ExprIr::GlobalPropertyRead { name } => {
                self.emit_global_property_read(name, payload_local, tag_local, function)?;
            }
            ExprIr::AssignIdentifier { name, value } => {
                let storage = self.lookup_binding(name).ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: unbound identifier `{name}`"
                    ))
                })?;
                self.compile_expr_to_binding(value, storage, function)?;
                self.mirror_binding_to_global_object(name, storage, function)?;
                self.read_binding_to_locals(storage, payload_local, tag_local, function);
            }
            ExprIr::GlobalPropertyWrite { name, value, .. } => {
                self.compile_expr_to_locals(value, payload_local, tag_local, function)?;
                self.emit_global_property_write(name, payload_local, tag_local, function)?;
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
            ExprIr::SuperPropertyRead { key } => {
                let super_base_local = self.reserve_temp_local();
                let super_base_tag_local = self.reserve_temp_local();
                self.emit_load_super_base(super_base_local, super_base_tag_local, function)?;
                self.emit_throw_if_null_super_base(
                    super_base_local,
                    super_base_tag_local,
                    function,
                )?;
                let Some(this_payload_local) = self.this_payload_local else {
                    return Err(EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: super outside class method",
                    ));
                };
                let Some(this_tag_local) = self.this_tag_local else {
                    return Err(EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: super outside class method",
                    ));
                };
                let key_local = self.compile_object_key_to_local(key, function)?;
                self.emit_object_read(
                    super_base_local,
                    super_base_tag_local,
                    this_payload_local,
                    this_tag_local,
                    key_local,
                    payload_local,
                    tag_local,
                    function,
                )?;
                self.release_temp_local(key_local);
                self.release_temp_local(super_base_tag_local);
                self.release_temp_local(super_base_local);
            }
            ExprIr::SuperPropertyWrite { key, value } => {
                let super_base_local = self.reserve_temp_local();
                let super_base_tag_local = self.reserve_temp_local();
                self.emit_load_super_base(super_base_local, super_base_tag_local, function)?;
                self.emit_throw_if_null_super_base(
                    super_base_local,
                    super_base_tag_local,
                    function,
                )?;
                let key_local = self.compile_object_key_to_local(key, function)?;
                self.compile_expr_to_locals(value, payload_local, tag_local, function)?;
                self.emit_object_write(
                    super_base_local,
                    super_base_tag_local,
                    key_local,
                    payload_local,
                    tag_local,
                    function,
                )?;
                self.release_temp_local(key_local);
                self.release_temp_local(super_base_tag_local);
                self.release_temp_local(super_base_local);
            }
            ExprIr::LogicalShortCircuit { op, lhs, rhs } => {
                self.compile_expr_to_locals(lhs, payload_local, tag_local, function)?;
                match op {
                    LogicalBinaryOp::Coalesce => {
                        self.compile_nullish_tagged_i32(tag_local, function)?;
                    }
                    LogicalBinaryOp::And | LogicalBinaryOp::Or => {
                        self.compile_truthy_tagged_i32(tag_local, payload_local, function)?;
                    }
                }
                function.instruction(&Instruction::If(BlockType::Empty));
                match op {
                    LogicalBinaryOp::And => {
                        self.compile_expr_to_locals(rhs, payload_local, tag_local, function)?;
                    }
                    LogicalBinaryOp::Or => {}
                    LogicalBinaryOp::Coalesce => {
                        self.compile_expr_to_locals(rhs, payload_local, tag_local, function)?;
                    }
                }
                function.instruction(&Instruction::Else);
                match op {
                    LogicalBinaryOp::And => {}
                    LogicalBinaryOp::Or => {
                        self.compile_expr_to_locals(rhs, payload_local, tag_local, function)?;
                    }
                    LogicalBinaryOp::Coalesce => {}
                }
                function.instruction(&Instruction::End);
            }
            ExprIr::Comma { lhs, rhs } => {
                self.compile_expr_to_locals(
                    lhs,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                self.compile_expr_to_locals(rhs, payload_local, tag_local, function)?;
            }
            ExprIr::GlobalPropertyUpdate { .. } | ExprIr::GlobalPropertyCompoundAssign { .. } => {
                self.compile_expr_payload(expr, function)?;
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(expr.kind.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
            }
            ExprIr::CoerciveAdd { lhs, rhs } => {
                self.compile_coercive_add_to_locals(lhs, rhs, payload_local, tag_local, function)?;
            }
            ExprIr::CallNamed { name, args } => {
                self.emit_call(name, args, payload_local, tag_local, function)?;
            }
            ExprIr::CallIndirect {
                callee,
                this_arg,
                args,
            } => {
                self.emit_indirect_call(
                    callee,
                    this_arg.as_deref(),
                    args,
                    payload_local,
                    tag_local,
                    function,
                )?;
            }
            ExprIr::Construct { callee, args } => {
                self.emit_construct(callee, args, payload_local, tag_local, function)?;
            }
            ExprIr::CallMethod {
                receiver,
                key,
                args,
            } => {
                self.emit_method_call(receiver, key, args, payload_local, tag_local, function)?;
            }
            ExprIr::InstanceOf { lhs, rhs } => {
                self.emit_instanceof_i32(lhs, rhs, function)?;
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
            }
            ExprIr::PrivateRead {
                target,
                private_name_id,
            } => {
                self.compile_private_read_to_locals(
                    target,
                    *private_name_id,
                    payload_local,
                    tag_local,
                    function,
                )?;
            }
            ExprIr::PrivateWrite {
                target,
                private_name_id,
                value,
            } => {
                self.compile_private_write_to_locals(
                    target,
                    *private_name_id,
                    value,
                    payload_local,
                    tag_local,
                    function,
                )?;
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
        function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.store_i64_local_at_offset(
            object_local,
            HEAP_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );

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
                ObjectPropertyIr::Data { value, .. }
                | ObjectPropertyIr::Method {
                    function: value, ..
                } => {
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
                ObjectPropertyIr::Getter {
                    function: getter, ..
                } => {
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
                ObjectPropertyIr::Setter {
                    function: setter, ..
                } => {
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
        function.instruction(&Instruction::GlobalGet(ARRAY_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.store_i64_local_at_offset(
            array_local,
            HEAP_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );

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

    fn compile_class_definition_payload(
        &mut self,
        class: &ClassDefinitionIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let constructor_meta = self
            .functions
            .get(&class.constructor_function_id)
            .ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: unknown class constructor `{}`",
                    class.constructor_function_id
                ))
            })?;
        let constructor_local = self.reserve_temp_local();
        let constructor_tag_local = self.reserve_temp_local();
        let heritage_payload_local = self.reserve_temp_local();
        let heritage_tag_local = self.reserve_temp_local();
        let prototype_key_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();
        let prototype_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();

        self.emit_function_value_payload(constructor_meta, function)?;
        function.instruction(&Instruction::LocalSet(constructor_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(heritage_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(heritage_tag_local));

        match class.heritage_kind {
            ClassHeritageKind::Constructable => {
                let Some(heritage) = &class.heritage else {
                    return Err(EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: missing class heritage",
                    ));
                };
                self.compile_expr_to_locals(
                    heritage,
                    heritage_payload_local,
                    heritage_tag_local,
                    function,
                )?;
                self.store_i64_local_at_offset(
                    constructor_local,
                    HEAP_PROTOTYPE_OFFSET,
                    heritage_payload_local,
                    function,
                );
            }
            ClassHeritageKind::Null | ClassHeritageKind::None => {}
        }

        function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
        function.instruction(&Instruction::LocalSet(prototype_key_local));
        if class.heritage_kind == ClassHeritageKind::Constructable {
            self.emit_object_read(
                heritage_payload_local,
                heritage_tag_local,
                heritage_payload_local,
                heritage_tag_local,
                prototype_key_local,
                value_payload_local,
                value_tag_local,
                function,
            )?;
        }

        if class.heritage_kind == ClassHeritageKind::Constructable {
            self.emit_is_heap_object_like_tag_i32(value_tag_local, function);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_alloc_plain_object_with_prototype(Some(value_payload_local), None, function)?;
            function.instruction(&Instruction::LocalSet(prototype_payload_local));
            function.instruction(&Instruction::Else);
            self.emit_alloc_plain_object_with_prototype(
                None,
                Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
                function,
            )?;
            function.instruction(&Instruction::LocalSet(prototype_payload_local));
            function.instruction(&Instruction::End);
        } else if class.heritage_kind == ClassHeritageKind::Null {
            self.emit_alloc_plain_object_with_prototype(None, None, function)?;
            function.instruction(&Instruction::LocalSet(prototype_payload_local));
        } else {
            self.emit_alloc_plain_object_with_prototype(
                None,
                Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
                function,
            )?;
            function.instruction(&Instruction::LocalSet(prototype_payload_local));
        }
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(prototype_tag_local));
        self.store_i64_local_at_offset(
            constructor_local,
            HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
            prototype_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            constructor_local,
            HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
            prototype_payload_local,
            function,
        );
        self.emit_object_define_data(
            constructor_local,
            prototype_key_local,
            prototype_payload_local,
            prototype_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::LocalGet(constructor_local));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_object_define_data(
            prototype_payload_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;

        for method in &class.public_methods {
            function.instruction(&Instruction::I64Const(self.strings.payload(&method.key)));
            function.instruction(&Instruction::LocalSet(key_local));
            let target_local = match method.placement {
                ClassMethodPlacementIr::Instance => prototype_payload_local,
                ClassMethodPlacementIr::Static => constructor_local,
            };
            match method.kind {
                ClassFunctionKind::Method => {
                    let temp_object_local = self.reserve_temp_local();
                    let temp_object_tag_local = self.reserve_temp_local();
                    let mut function_targets = BTreeSet::new();
                    function_targets.insert(method.function_id.clone());
                    let function_expr = TypedExpr::from_info(
                        ValueInfo {
                            kind: ValueKind::Function,
                            possible_kinds: KindSet::from_kind(ValueKind::Function),
                            heap_shape: None,
                            function_targets,
                        },
                        ExprIr::FunctionValue(method.function_id.clone()),
                    );
                    self.compile_object_literal_payload(
                        &[ObjectPropertyIr::Method {
                            key: method.key.clone(),
                            function: function_expr,
                        }],
                        function,
                    )?;
                    function.instruction(&Instruction::LocalSet(temp_object_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                    function.instruction(&Instruction::LocalSet(temp_object_tag_local));
                    self.emit_object_read(
                        temp_object_local,
                        temp_object_tag_local,
                        temp_object_local,
                        temp_object_tag_local,
                        key_local,
                        value_payload_local,
                        value_tag_local,
                        function,
                    )?;
                    self.release_temp_local(temp_object_tag_local);
                    self.release_temp_local(temp_object_local);
                    self.emit_object_define_data(
                        target_local,
                        key_local,
                        value_payload_local,
                        value_tag_local,
                        function,
                    )?;
                }
                ClassFunctionKind::Getter => {
                    let meta = self.functions.get(&method.function_id).ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "unsupported in porffor wasm-aot first slice: unknown class method `{}`",
                            method.function_id
                        ))
                    })?;
                    self.emit_function_value_payload(meta, function)?;
                    function.instruction(&Instruction::LocalSet(value_payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                    function.instruction(&Instruction::LocalSet(value_tag_local));
                    self.emit_object_define_accessor(
                        target_local,
                        key_local,
                        Some((value_payload_local, value_tag_local)),
                        None,
                        function,
                    )?;
                }
                ClassFunctionKind::Setter => {
                    let meta = self.functions.get(&method.function_id).ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "unsupported in porffor wasm-aot first slice: unknown class method `{}`",
                            method.function_id
                        ))
                    })?;
                    self.emit_function_value_payload(meta, function)?;
                    function.instruction(&Instruction::LocalSet(value_payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                    function.instruction(&Instruction::LocalSet(value_tag_local));
                    self.emit_object_define_accessor(
                        target_local,
                        key_local,
                        None,
                        Some((value_payload_local, value_tag_local)),
                        function,
                    )?;
                }
                ClassFunctionKind::None | ClassFunctionKind::Constructor => {
                    return Err(EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: class method kind",
                    ));
                }
            }
        }
        for method in &class.private_methods {
            function.instruction(&Instruction::I64Const(
                self.strings
                    .payload(&private_data_key(method.private_name_id)),
            ));
            function.instruction(&Instruction::LocalSet(key_local));
            let target_local = match method.placement {
                ClassMethodPlacementIr::Instance => prototype_payload_local,
                ClassMethodPlacementIr::Static => constructor_local,
            };
            match method.kind {
                ClassFunctionKind::Method => {
                    let temp_object_local = self.reserve_temp_local();
                    let temp_object_tag_local = self.reserve_temp_local();
                    let hidden_key = private_data_key(method.private_name_id);
                    let mut function_targets = BTreeSet::new();
                    function_targets.insert(method.function_id.clone());
                    let function_expr = TypedExpr::from_info(
                        ValueInfo {
                            kind: ValueKind::Function,
                            possible_kinds: KindSet::from_kind(ValueKind::Function),
                            heap_shape: None,
                            function_targets,
                        },
                        ExprIr::FunctionValue(method.function_id.clone()),
                    );
                    self.compile_object_literal_payload(
                        &[ObjectPropertyIr::Method {
                            key: hidden_key,
                            function: function_expr,
                        }],
                        function,
                    )?;
                    function.instruction(&Instruction::LocalSet(temp_object_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                    function.instruction(&Instruction::LocalSet(temp_object_tag_local));
                    self.emit_object_read(
                        temp_object_local,
                        temp_object_tag_local,
                        temp_object_local,
                        temp_object_tag_local,
                        key_local,
                        value_payload_local,
                        value_tag_local,
                        function,
                    )?;
                    self.release_temp_local(temp_object_tag_local);
                    self.release_temp_local(temp_object_local);
                    self.emit_object_define_data(
                        target_local,
                        key_local,
                        value_payload_local,
                        value_tag_local,
                        function,
                    )?;
                }
                ClassFunctionKind::Getter => {
                    let meta = self.functions.get(&method.function_id).ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "unsupported in porffor wasm-aot first slice: unknown class method `{}`",
                            method.function_id
                        ))
                    })?;
                    self.emit_function_value_payload(meta, function)?;
                    function.instruction(&Instruction::LocalSet(value_payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                    function.instruction(&Instruction::LocalSet(value_tag_local));
                    self.emit_object_define_accessor(
                        target_local,
                        key_local,
                        Some((value_payload_local, value_tag_local)),
                        None,
                        function,
                    )?;
                }
                ClassFunctionKind::Setter => {
                    let meta = self.functions.get(&method.function_id).ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "unsupported in porffor wasm-aot first slice: unknown class method `{}`",
                            method.function_id
                        ))
                    })?;
                    self.emit_function_value_payload(meta, function)?;
                    function.instruction(&Instruction::LocalSet(value_payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                    function.instruction(&Instruction::LocalSet(value_tag_local));
                    self.emit_object_define_accessor(
                        target_local,
                        key_local,
                        None,
                        Some((value_payload_local, value_tag_local)),
                        function,
                    )?;
                }
                ClassFunctionKind::None | ClassFunctionKind::Constructor => {
                    return Err(EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: class method kind",
                    ));
                }
            }
        }

        let mut static_private_brands = BTreeSet::new();
        for method in &class.private_methods {
            if method.placement == ClassMethodPlacementIr::Static {
                static_private_brands.insert(method.private_name_id);
            }
        }
        for field in &class.fields {
            if field.is_private && field.placement == ClassMethodPlacementIr::Static {
                if let Some(private_name_id) = field.private_name_id {
                    static_private_brands.insert(private_name_id);
                }
            }
        }
        for private_name_id in static_private_brands {
            function.instruction(&Instruction::I64Const(
                self.strings.payload(&private_brand_key(private_name_id)),
            ));
            function.instruction(&Instruction::LocalSet(key_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(value_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
            function.instruction(&Instruction::LocalSet(value_tag_local));
            self.emit_object_write(
                constructor_local,
                constructor_tag_local,
                key_local,
                value_payload_local,
                value_tag_local,
                function,
            )?;
        }

        for field in &class.fields {
            if field.placement != ClassMethodPlacementIr::Static {
                continue;
            }
            let key = if let Some(key) = &field.key {
                key.clone()
            } else if let Some(private_name_id) = field.private_name_id {
                private_data_key(private_name_id)
            } else {
                return Err(EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: malformed class field",
                ));
            };
            function.instruction(&Instruction::I64Const(self.strings.payload(&key)));
            function.instruction(&Instruction::LocalSet(key_local));
            if let Some(init_function_id) = &field.init_function_id {
                let meta = self.functions.get(init_function_id).ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: unknown class field init `{init_function_id}`"
                    ))
                })?;
                self.emit_direct_js_call(
                    meta,
                    Some((constructor_local, Some(constructor_tag_local))),
                    &[],
                    value_payload_local,
                    value_tag_local,
                    function,
                )?;
            } else {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(value_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::LocalSet(value_tag_local));
            }
            self.emit_object_write(
                constructor_local,
                constructor_tag_local,
                key_local,
                value_payload_local,
                value_tag_local,
                function,
            )?;
        }

        for block in &class.static_blocks {
            let meta = self.functions.get(&block.function_id).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: unknown class static block `{}`",
                    block.function_id
                ))
            })?;
            self.emit_direct_js_call(
                meta,
                Some((constructor_local, Some(constructor_tag_local))),
                &[],
                value_payload_local,
                value_tag_local,
                function,
            )?;
        }

        function.instruction(&Instruction::LocalGet(constructor_local));
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(prototype_tag_local);
        self.release_temp_local(prototype_payload_local);
        self.release_temp_local(prototype_key_local);
        self.release_temp_local(heritage_tag_local);
        self.release_temp_local(heritage_payload_local);
        self.release_temp_local(constructor_tag_local);
        self.release_temp_local(constructor_local);
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
            ValueKind::Object | ValueKind::Function => {
                let key_local = self.compile_object_key_to_local(key, function)?;
                self.emit_object_read(
                    target_local,
                    target_tag_local,
                    target_local,
                    target_tag_local,
                    key_local,
                    payload_local,
                    tag_local,
                    function,
                )?;
                self.release_temp_local(key_local);
            }
            ValueKind::Array => match key {
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
            },
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
            ValueKind::Dynamic => match key {
                PropertyKeyIr::StaticString(_) | PropertyKeyIr::StringExpr(_) => {
                    let key_local = self.compile_object_key_to_local(key, function)?;
                    function.instruction(&Instruction::LocalGet(target_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::LocalGet(target_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::I32Or);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_object_read(
                        target_local,
                        target_tag_local,
                        target_local,
                        target_tag_local,
                        key_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::Else);
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                    function.instruction(&Instruction::End);
                    self.release_temp_local(key_local);
                }
                PropertyKeyIr::ArrayLength => {
                    function.instruction(&Instruction::LocalGet(target_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_array_length(target_local, payload_local, tag_local, function);
                    function.instruction(&Instruction::Else);
                    function.instruction(&Instruction::LocalGet(target_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_arguments_length(target_local, payload_local, tag_local, function);
                    function.instruction(&Instruction::Else);
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::End);
                }
                _ => {
                    let index_local = self.compile_array_index_to_local(key, function)?;
                    function.instruction(&Instruction::LocalGet(target_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_array_read(
                        target_local,
                        index_local,
                        payload_local,
                        tag_local,
                        function,
                    );
                    function.instruction(&Instruction::Else);
                    function.instruction(&Instruction::LocalGet(target_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_arguments_read(
                        target_local,
                        index_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::Else);
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::End);
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
            ValueKind::Object | ValueKind::Function => {
                let key_local = self.compile_object_key_to_local(key, function)?;
                self.compile_expr_to_locals(value, payload_local, tag_local, function)?;
                self.emit_object_write(
                    target_local,
                    target_tag_local,
                    key_local,
                    payload_local,
                    tag_local,
                    function,
                )?;
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

    fn compile_delete_property_i32(
        &mut self,
        target: &TypedExpr,
        key: &PropertyKeyIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let target_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let result_local = self.reserve_temp_local();
        self.compile_expr_to_locals(target, target_local, target_tag_local, function)?;

        match target.kind {
            ValueKind::Object | ValueKind::Function => {
                let key_local = self.compile_object_key_to_local(key, function)?;
                self.emit_object_delete(
                    target_local,
                    target_tag_local,
                    key_local,
                    result_local,
                    function,
                )?;
                self.release_temp_local(key_local);
            }
            ValueKind::Array => {
                let index_local = self.compile_array_index_to_local(key, function)?;
                self.emit_array_delete(target_local, index_local, result_local, function);
                self.release_temp_local(index_local);
            }
            ValueKind::Arguments => {
                self.release_temp_local(result_local);
                self.release_temp_local(target_tag_local);
                self.release_temp_local(target_local);
                return Err(EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: delete on arguments object",
                ));
            }
            _ => {
                self.release_temp_local(result_local);
                self.release_temp_local(target_tag_local);
                self.release_temp_local(target_local);
                return Err(EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: delete on non-object target",
                ));
            }
        }

        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I32WrapI64);
        self.release_temp_local(result_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_local);
        Ok(())
    }

    fn emit_in_i32(
        &mut self,
        lhs: &TypedExpr,
        rhs: &TypedExpr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let lhs_payload_local = self.reserve_temp_local();
        let lhs_tag_local = self.reserve_temp_local();
        let rhs_payload_local = self.reserve_temp_local();
        let rhs_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let result_local = self.reserve_temp_local();

        self.compile_expr_to_primitive_locals(
            lhs,
            ToPrimitiveHint::String,
            lhs_payload_local,
            lhs_tag_local,
            function,
        )?;
        self.compile_expr_to_locals(rhs, rhs_payload_local, rhs_tag_local, function)?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));

        match rhs.kind {
            ValueKind::Object | ValueKind::Function => {
                self.emit_value_to_string_payload(lhs_payload_local, lhs_tag_local, function)?;
                function.instruction(&Instruction::LocalSet(key_local));
                self.emit_object_has_property_i32(
                    rhs_payload_local,
                    rhs_tag_local,
                    key_local,
                    result_local,
                    function,
                )?;
            }
            ValueKind::Array => {
                if lhs.kind == ValueKind::Number {
                    function.instruction(&Instruction::LocalGet(lhs_payload_local));
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::I64TruncF64U);
                    function.instruction(&Instruction::LocalSet(key_local));
                    self.emit_array_has_index_i32(
                        rhs_payload_local,
                        key_local,
                        result_local,
                        function,
                    );
                } else {
                    self.emit_value_to_string_payload(lhs_payload_local, lhs_tag_local, function)?;
                    function.instruction(&Instruction::LocalSet(key_local));
                    function.instruction(&Instruction::LocalGet(key_local));
                    function.instruction(&Instruction::I64Const(self.strings.payload("length")));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::LocalSet(result_local));
                    function.instruction(&Instruction::End);
                }
            }
            ValueKind::Arguments => {
                if lhs.kind == ValueKind::Number {
                    function.instruction(&Instruction::LocalGet(lhs_payload_local));
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::I64TruncF64U);
                    function.instruction(&Instruction::LocalSet(key_local));
                    self.emit_arguments_has_index_i32(
                        rhs_payload_local,
                        key_local,
                        result_local,
                        function,
                    )?;
                } else {
                    self.emit_value_to_string_payload(lhs_payload_local, lhs_tag_local, function)?;
                    function.instruction(&Instruction::LocalSet(key_local));
                    function.instruction(&Instruction::LocalGet(key_local));
                    function.instruction(&Instruction::I64Const(self.strings.payload("length")));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::LocalSet(result_local));
                    function.instruction(&Instruction::End);
                }
            }
            _ => {
                self.emit_throw_runtime_error(
                    "TypeError",
                    "right-hand side of `in` is not an object",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                if let Some(target) = self.throw_handler_stack.last() {
                    function.instruction(&Instruction::Br(self.depth_to(*target)));
                } else {
                    self.emit_return_current_completion(function);
                }
            }
        }

        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I32WrapI64);
        self.release_temp_local(result_local);
        self.release_temp_local(key_local);
        self.release_temp_local(rhs_tag_local);
        self.release_temp_local(rhs_payload_local);
        self.release_temp_local(lhs_tag_local);
        self.release_temp_local(lhs_payload_local);
        Ok(())
    }

    fn compile_private_read_to_locals(
        &mut self,
        target: &TypedExpr,
        private_name_id: PrivateNameId,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let brand_key_local = self.reserve_temp_local();
        let brand_payload_local = self.reserve_temp_local();
        let brand_tag_local = self.reserve_temp_local();
        let data_key_local = self.reserve_temp_local();

        self.compile_expr_to_locals(target, target_payload_local, target_tag_local, function)?;
        self.emit_private_brand_guard(
            target_payload_local,
            target_tag_local,
            private_name_id,
            brand_key_local,
            brand_payload_local,
            brand_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(
            self.strings.payload(&private_data_key(private_name_id)),
        ));
        function.instruction(&Instruction::LocalSet(data_key_local));
        self.emit_object_read(
            target_payload_local,
            target_tag_local,
            target_payload_local,
            target_tag_local,
            data_key_local,
            payload_local,
            tag_local,
            function,
        )?;

        self.release_temp_local(data_key_local);
        self.release_temp_local(brand_tag_local);
        self.release_temp_local(brand_payload_local);
        self.release_temp_local(brand_key_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        Ok(())
    }

    fn compile_private_write_to_locals(
        &mut self,
        target: &TypedExpr,
        private_name_id: PrivateNameId,
        value: &TypedExpr,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let brand_key_local = self.reserve_temp_local();
        let brand_payload_local = self.reserve_temp_local();
        let brand_tag_local = self.reserve_temp_local();
        let data_key_local = self.reserve_temp_local();

        self.compile_expr_to_locals(target, target_payload_local, target_tag_local, function)?;
        self.emit_private_brand_guard(
            target_payload_local,
            target_tag_local,
            private_name_id,
            brand_key_local,
            brand_payload_local,
            brand_tag_local,
            function,
        )?;
        self.compile_expr_to_locals(value, payload_local, tag_local, function)?;
        function.instruction(&Instruction::I64Const(
            self.strings.payload(&private_data_key(private_name_id)),
        ));
        function.instruction(&Instruction::LocalSet(data_key_local));
        self.emit_object_write(
            target_payload_local,
            target_tag_local,
            data_key_local,
            payload_local,
            tag_local,
            function,
        )?;

        self.release_temp_local(data_key_local);
        self.release_temp_local(brand_tag_local);
        self.release_temp_local(brand_payload_local);
        self.release_temp_local(brand_key_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        Ok(())
    }

    fn emit_private_brand_guard(
        &mut self,
        target_payload_local: u32,
        target_tag_local: u32,
        private_name_id: PrivateNameId,
        brand_key_local: u32,
        brand_payload_local: u32,
        brand_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_is_heap_object_like_tag_i32(target_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload(&private_brand_key(private_name_id)),
        ));
        function.instruction(&Instruction::LocalSet(brand_key_local));
        self.emit_object_read(
            target_payload_local,
            target_tag_local,
            target_payload_local,
            target_tag_local,
            brand_key_local,
            brand_payload_local,
            brand_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(brand_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(brand_payload_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            "TypeError",
            "private field access on wrong object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        if let Some(target) = self.throw_handler_stack.last() {
            function.instruction(&Instruction::Br(self.depth_to(*target) + 2));
        } else {
            self.emit_return_current_completion(function);
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            "TypeError",
            "private field access on wrong object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        if let Some(target) = self.throw_handler_stack.last() {
            function.instruction(&Instruction::Br(self.depth_to(*target) + 1));
        } else {
            self.emit_return_current_completion(function);
        }
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn normalize_derived_constructor_result(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let Some(derived_this_initialized_local) = self.derived_this_initialized_local else {
            return Ok(());
        };
        let Some(this_payload_local) = self.this_payload_local else {
            return Ok(());
        };
        let Some(this_tag_local) = self.this_tag_local else {
            return Ok(());
        };

        self.emit_is_heap_object_like_tag_i32(self.result_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            "TypeError",
            "derived constructor may only return object or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        if let Some(target) = self.throw_handler_stack.last() {
            function.instruction(&Instruction::Br(self.depth_to(*target) + 3));
        } else {
            self.emit_return_current_completion(function);
        }
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(derived_this_initialized_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            "ReferenceError",
            "derived constructor must call super() before returning",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        if let Some(target) = self.throw_handler_stack.last() {
            function.instruction(&Instruction::Br(self.depth_to(*target) + 2));
        } else {
            self.emit_return_current_completion(function);
        }
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
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

    fn emit_heap_alloc_const(
        &mut self,
        size: u64,
        function: &mut Function,
    ) -> Result<(), EmitError> {
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
        object_tag_local: u32,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        key_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let current_local = self.reserve_temp_local();
        let prototype_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let getter_payload_local = self.reserve_temp_local();
        let getter_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            object_local,
            HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
            tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            object_local,
            HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalSet(current_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(current_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));

        self.load_i64_to_local_from_offset(current_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(current_local, HEAP_LEN_OFFSET, len_local, function);
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
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DATA_TAG_OFFSET,
            tag_local,
            function,
        );
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
            Some((receiver_payload_local, Some(receiver_tag_local))),
            &[],
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(4));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            current_local,
            HEAP_PROTOTYPE_OFFSET,
            prototype_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_local));
        function.instruction(&Instruction::LocalSet(current_local));
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
        self.release_temp_local(prototype_local);
        self.release_temp_local(current_local);
        function.instruction(&Instruction::End);
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
        self.emit_object_define_data_with_configurable(
            object_local,
            key_local,
            payload_local,
            tag_local,
            true,
            function,
        )
    }

    fn emit_object_define_data_with_configurable(
        &mut self,
        object_local: u32,
        key_local: u32,
        payload_local: u32,
        tag_local: u32,
        configurable: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_object_define_entry(
            object_local,
            key_local,
            Some((payload_local, tag_local)),
            None,
            None,
            configurable,
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
        self.emit_object_define_entry(
            object_local,
            key_local,
            None,
            getter,
            setter,
            true,
            function,
        )
    }

    fn emit_object_define_entry(
        &mut self,
        object_local: u32,
        key_local: u32,
        data: Option<(u32, u32)>,
        getter: Option<(u32, u32)>,
        setter: Option<(u32, u32)>,
        configurable: bool,
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
        let stored_data_tag_local = self.reserve_temp_local();
        let stored_data_payload_local = self.reserve_temp_local();

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
        } | if configurable {
            OBJECT_DESCRIPTOR_CONFIGURABLE
        } else {
            0
        };
        function.instruction(&Instruction::I64Const(descriptor_kind as i64));
        function.instruction(&Instruction::LocalSet(descriptor_kind_local));
        if let Some((data_payload_local, data_tag_local)) = data {
            function.instruction(&Instruction::LocalGet(data_tag_local));
            function.instruction(&Instruction::LocalSet(stored_data_tag_local));
            function.instruction(&Instruction::LocalGet(data_payload_local));
            function.instruction(&Instruction::LocalSet(stored_data_payload_local));
        } else {
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(stored_data_tag_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(stored_data_payload_local));
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
                stored_data_tag_local,
                function,
            );
            self.store_i64_local_at_offset(
                entry_local,
                HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
                stored_data_payload_local,
                function,
            );
            self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_GETTER_TAG_OFFSET, 0, function);
            self.store_i64_const_at_offset(
                entry_local,
                HEAP_OBJECT_GETTER_PAYLOAD_OFFSET,
                0,
                function,
            );
            self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_SETTER_TAG_OFFSET, 0, function);
            self.store_i64_const_at_offset(
                entry_local,
                HEAP_OBJECT_SETTER_PAYLOAD_OFFSET,
                0,
                function,
            );
        } else {
            self.load_i64_to_local_from_offset(
                entry_local,
                HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
                existing_descriptor_kind_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
            function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
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
            self.store_i64_const_at_offset(
                entry_local,
                HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
                0,
                function,
            );
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
                stored_data_tag_local,
                function,
            );
            self.store_i64_local_at_offset(
                entry_local,
                HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
                stored_data_payload_local,
                function,
            );
            self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_GETTER_TAG_OFFSET, 0, function);
            self.store_i64_const_at_offset(
                entry_local,
                HEAP_OBJECT_GETTER_PAYLOAD_OFFSET,
                0,
                function,
            );
            self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_SETTER_TAG_OFFSET, 0, function);
            self.store_i64_const_at_offset(
                entry_local,
                HEAP_OBJECT_SETTER_PAYLOAD_OFFSET,
                0,
                function,
            );
        } else {
            self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_DATA_TAG_OFFSET, 0, function);
            self.store_i64_const_at_offset(
                entry_local,
                HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
                0,
                function,
            );
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
                self.store_i64_const_at_offset(
                    entry_local,
                    HEAP_OBJECT_GETTER_TAG_OFFSET,
                    0,
                    function,
                );
                self.store_i64_const_at_offset(
                    entry_local,
                    HEAP_OBJECT_GETTER_PAYLOAD_OFFSET,
                    0,
                    function,
                );
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
                self.store_i64_const_at_offset(
                    entry_local,
                    HEAP_OBJECT_SETTER_TAG_OFFSET,
                    0,
                    function,
                );
                self.store_i64_const_at_offset(
                    entry_local,
                    HEAP_OBJECT_SETTER_PAYLOAD_OFFSET,
                    0,
                    function,
                );
            }
        }
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(len_local));
        self.store_i64_local_at_offset(object_local, HEAP_LEN_OFFSET, len_local, function);
        function.instruction(&Instruction::End);

        self.release_temp_local(stored_data_payload_local);
        self.release_temp_local(stored_data_tag_local);
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
        object_tag_local: u32,
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

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_local_at_offset(
            object_local,
            HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
            tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            object_local,
            HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

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
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_DATA_TAG_OFFSET,
            tag_local,
            function,
        );
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
            self.scratch_local,
            self.result_tag_local,
            function,
        )?;
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
            OBJECT_DESCRIPTOR_DATA | OBJECT_DESCRIPTOR_CONFIGURABLE,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_DATA_TAG_OFFSET,
            tag_local,
            function,
        );
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
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn emit_object_delete(
        &mut self,
        object_local: u32,
        object_tag_local: u32,
        key_local: u32,
        result_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let shift_index_local = self.reserve_temp_local();
        let current_entry_local = self.reserve_temp_local();
        let next_entry_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));

        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_const_at_offset(
            object_local,
            HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
            ValueKind::Undefined.tag() as u64,
            function,
        );
        self.store_i64_const_at_offset(
            object_local,
            HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(
            object_local,
            HEAP_FUNCTION_TO_STRING_PAYLOAD_OFFSET,
            self.strings.payload("function () { [native code] }") as u64,
            function,
        );
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(object_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(object_local, HEAP_LEN_OFFSET, len_local, function);
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
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            key_payload_local,
            function,
        );
        self.emit_string_payload_equality_i32(key_payload_local, key_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_DESCRIPTOR_CONFIGURABLE as i64,
        ));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalSet(shift_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(shift_index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(shift_index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(current_entry_local));
        function.instruction(&Instruction::LocalGet(current_entry_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(next_entry_local));

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
            self.load_i64_from_offset(next_entry_local, offset, function);
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.store_i64_local_at_offset(
                current_entry_local,
                offset,
                self.scratch_local,
                function,
            );
        }

        function.instruction(&Instruction::LocalGet(shift_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(shift_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(len_local));
        self.store_i64_local_at_offset(object_local, HEAP_LEN_OFFSET, len_local, function);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(next_entry_local);
        self.release_temp_local(current_entry_local);
        self.release_temp_local(shift_index_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    fn emit_object_has_property_i32(
        &mut self,
        object_local: u32,
        object_tag_local: u32,
        key_local: u32,
        result_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let current_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let prototype_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalSet(current_local));

        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            object_local,
            HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
            self.result_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(current_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));

        self.load_i64_to_local_from_offset(current_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(current_local, HEAP_LEN_OFFSET, len_local, function);
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
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            key_payload_local,
            function,
        );
        self.emit_string_payload_equality_i32(key_payload_local, key_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            current_local,
            HEAP_PROTOTYPE_OFFSET,
            prototype_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_local));
        function.instruction(&Instruction::LocalSet(current_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(prototype_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(current_local);
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
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_HOLE_TAG));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::End);
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
        let fill_index_local = self.reserve_temp_local();
        let fill_entry_local = self.reserve_temp_local();

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
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalSet(fill_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(fill_index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(fill_index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(fill_entry_local));
        self.store_i64_const_at_offset(
            fill_entry_local,
            HEAP_ARRAY_TAG_OFFSET,
            HEAP_ARRAY_HOLE_TAG as u64,
            function,
        );
        self.store_i64_const_at_offset(fill_entry_local, HEAP_ARRAY_PAYLOAD_OFFSET, 0, function);
        function.instruction(&Instruction::LocalGet(fill_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(fill_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(len_local));
        self.store_i64_local_at_offset(array_local, HEAP_LEN_OFFSET, len_local, function);
        function.instruction(&Instruction::End);

        self.release_temp_local(fill_entry_local);
        self.release_temp_local(fill_index_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(cap_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    fn emit_array_delete(
        &mut self,
        array_local: u32,
        index_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));
        self.load_i64_to_local_from_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(array_local, HEAP_LEN_OFFSET, len_local, function);
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
        self.store_i64_const_at_offset(
            entry_local,
            HEAP_ARRAY_TAG_OFFSET,
            HEAP_ARRAY_HOLE_TAG as u64,
            function,
        );
        self.store_i64_const_at_offset(entry_local, HEAP_ARRAY_PAYLOAD_OFFSET, 0, function);
        function.instruction(&Instruction::End);

        self.release_temp_local(entry_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
    }

    fn emit_array_has_index_i32(
        &mut self,
        array_local: u32,
        index_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        self.load_i64_to_local_from_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(array_local, HEAP_LEN_OFFSET, len_local, function);
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
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_TAG_OFFSET,
            self.result_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_HOLE_TAG));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(entry_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
    }

    fn emit_arguments_has_index_i32(
        &mut self,
        arguments_local: u32,
        index_local: u32,
        result_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let mapped_count_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_ARGUMENTS_MAPPED_COUNT_OFFSET,
            mapped_count_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(arguments_local, HEAP_LEN_OFFSET, len_local, function);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(0));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(mapped_count_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_TAG_OFFSET,
            self.result_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_HOLE_TAG));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(entry_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(mapped_count_local);
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
            function.instruction(&Instruction::I64Const(
                (index as u64 * HEAP_ARRAY_ENTRY_SIZE) as i64,
            ));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(entry_local));
            self.store_i64_local_at_offset(
                entry_local,
                HEAP_ARRAY_TAG_OFFSET,
                self.result_tag_local,
                function,
            );
            self.store_i64_local_at_offset(
                entry_local,
                HEAP_ARRAY_PAYLOAD_OFFSET,
                self.scratch_local,
                function,
            );
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
            function.instruction(&Instruction::I64Const(
                (index as u64 * HEAP_ARRAY_ENTRY_SIZE) as i64,
            ));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(entry_local));
            self.store_i64_local_at_offset(
                entry_local,
                HEAP_ARRAY_TAG_OFFSET,
                *arg_tag_local,
                function,
            );
            self.store_i64_local_at_offset(
                entry_local,
                HEAP_ARRAY_PAYLOAD_OFFSET,
                *arg_payload_local,
                function,
            );
        }

        self.release_temp_local(entry_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    fn compile_host_print_builtin(&mut self, function: &mut Function) -> Result<(), EmitError> {
        let argc_local = self.argc_param_local();
        let argv_local = self.argv_param_local();
        let output_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let arg_payload_local = self.reserve_temp_local();
        let arg_tag_local = self.reserve_temp_local();
        let arg_string_local = self.reserve_temp_local();
        let space_string_local = self.reserve_temp_local();
        let ptr_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::I64Const(self.strings.payload(" ")));
        function.instruction(&Instruction::LocalSet(space_string_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(argc_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_concat_string_payloads_local(output_local, space_string_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::End);

        self.emit_array_read(
            argv_local,
            index_local,
            arg_payload_local,
            arg_tag_local,
            function,
        );
        self.emit_value_to_string_payload(arg_payload_local, arg_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(arg_string_local));
        self.emit_concat_string_payloads_local(output_local, arg_string_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_unpack_string_payload(output_local, ptr_local, len_local, function);
        function.instruction(&Instruction::LocalGet(ptr_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::Call(HOST_PRINT_IMPORT_FUNCTION_INDEX));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(len_local);
        self.release_temp_local(ptr_local);
        self.release_temp_local(space_string_local);
        self.release_temp_local(arg_string_local);
        self.release_temp_local(arg_tag_local);
        self.release_temp_local(arg_payload_local);
        self.release_temp_local(index_local);
        self.release_temp_local(output_local);
        Ok(())
    }

    fn compile_host_gc_builtin(&mut self, function: &mut Function) {
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
    }

    fn emit_builtin_arg_to_locals(
        &mut self,
        index: usize,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        let argc_local = self.argc_param_local();
        let argv_local = self.argv_param_local();
        function.instruction(&Instruction::LocalGet(argc_local));
        function.instruction(&Instruction::I64Const(index as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(index as i64));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_array_read(
            argv_local,
            self.scratch_local,
            payload_local,
            tag_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::End);
    }

    fn emit_alloc_error_instance_from_locals(
        &mut self,
        name: &str,
        message_payload_local: Option<u32>,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(error_prototype_global_index(name)),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(object_local));
        if let Some(message_payload_local) = message_payload_local {
            function.instruction(&Instruction::I64Const(self.strings.payload("message")));
            function.instruction(&Instruction::LocalSet(key_local));
            function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
            function.instruction(&Instruction::LocalSet(value_tag_local));
            self.emit_object_define_data(
                object_local,
                key_local,
                message_payload_local,
                value_tag_local,
                function,
            )?;
        }
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.release_temp_local(value_tag_local);
        self.release_temp_local(key_local);
        self.release_temp_local(object_local);
        Ok(())
    }

    fn emit_alloc_aggregate_error_instance_from_locals(
        &mut self,
        message_payload_local: Option<u32>,
        errors_payload_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(AGGREGATE_ERROR_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(object_local));
        if let Some(message_payload_local) = message_payload_local {
            function.instruction(&Instruction::I64Const(self.strings.payload("message")));
            function.instruction(&Instruction::LocalSet(key_local));
            function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
            function.instruction(&Instruction::LocalSet(value_tag_local));
            self.emit_object_define_data(
                object_local,
                key_local,
                message_payload_local,
                value_tag_local,
                function,
            )?;
        }
        function.instruction(&Instruction::I64Const(self.strings.payload("errors")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_object_define_data(
            object_local,
            key_local,
            errors_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.release_temp_local(value_tag_local);
        self.release_temp_local(key_local);
        self.release_temp_local(object_local);
        Ok(())
    }

    fn emit_alloc_array_payload_with_length(
        &mut self,
        len_local: u32,
        payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let array_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let cap_local = self.reserve_temp_local();
        let size_local = self.reserve_temp_local();
        self.emit_heap_alloc_const(HEAP_HEADER_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(array_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(MIN_HEAP_CAPACITY as i64));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(cap_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(size_local));
        self.emit_heap_alloc_from_local(size_local, function)?;
        function.instruction(&Instruction::LocalSet(buffer_local));
        self.store_i64_local_at_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.store_i64_local_at_offset(array_local, HEAP_LEN_OFFSET, len_local, function);
        self.store_i64_local_at_offset(array_local, HEAP_CAP_OFFSET, cap_local, function);
        function.instruction(&Instruction::GlobalGet(ARRAY_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.store_i64_local_at_offset(
            array_local,
            HEAP_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(array_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        self.release_temp_local(size_local);
        self.release_temp_local(cap_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(array_local);
        Ok(())
    }

    fn emit_array_like_snapshot_payload(
        &mut self,
        input_payload_local: u32,
        input_tag_local: u32,
        payload_local: u32,
        wrong_type_message: &str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let len_local = self.reserve_temp_local();
        let dst_payload_local = self.reserve_temp_local();
        let dst_buffer_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(len_local));
        self.emit_alloc_array_payload_with_length(len_local, payload_local, function)?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            input_payload_local,
            HEAP_LEN_OFFSET,
            len_local,
            function,
        );
        self.emit_alloc_array_payload_with_length(len_local, dst_payload_local, function)?;
        self.load_i64_to_local_from_offset(
            dst_payload_local,
            HEAP_PTR_OFFSET,
            dst_buffer_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            input_payload_local,
            index_local,
            value_payload_local,
            value_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(dst_buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_ARRAY_TAG_OFFSET,
            value_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_ARRAY_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(dst_payload_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            input_payload_local,
            HEAP_LEN_OFFSET,
            len_local,
            function,
        );
        self.emit_alloc_array_payload_with_length(len_local, dst_payload_local, function)?;
        self.load_i64_to_local_from_offset(
            dst_payload_local,
            HEAP_PTR_OFFSET,
            dst_buffer_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_arguments_read(
            input_payload_local,
            index_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(dst_buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_ARRAY_TAG_OFFSET,
            value_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_ARRAY_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(dst_payload_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            wrong_type_message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
            self.result_local,
            self.result_tag_local,
            1,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(dst_buffer_local);
        self.release_temp_local(dst_payload_local);
        self.release_temp_local(len_local);
        Ok(())
    }

    fn emit_adapt_call_this_arg(
        &mut self,
        input_payload_local: u32,
        input_tag_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        self.emit_is_heap_object_like_tag_i32(input_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(input_payload_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_boxed_wrapper_from_locals(
            NUMBER_PROTOTYPE_GLOBAL_INDEX,
            BOXED_PRIMITIVE_KIND_NUMBER,
            input_payload_local,
            input_tag_local,
            payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_boxed_wrapper_from_locals(
            STRING_PROTOTYPE_GLOBAL_INDEX,
            BOXED_PRIMITIVE_KIND_STRING,
            input_payload_local,
            input_tag_local,
            payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_boxed_wrapper_from_locals(
            BOOLEAN_PROTOTYPE_GLOBAL_INDEX,
            BOXED_PRIMITIVE_KIND_BOOLEAN,
            input_payload_local,
            input_tag_local,
            payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Function.prototype.call/apply thisArg adaptation failed",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
            self.result_local,
            self.result_tag_local,
            4,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn emit_load_bound_function_record(
        &mut self,
        record_local: u32,
        target_payload_local: u32,
        target_tag_local: u32,
        bound_this_payload_local: u32,
        bound_this_tag_local: u32,
        bound_args_payload_local: u32,
        function: &mut Function,
    ) {
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_BOUND_FUNCTION_TARGET_PAYLOAD_OFFSET,
            target_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_BOUND_FUNCTION_TARGET_TAG_OFFSET,
            target_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_BOUND_FUNCTION_THIS_PAYLOAD_OFFSET,
            bound_this_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_BOUND_FUNCTION_THIS_TAG_OFFSET,
            bound_this_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_BOUND_FUNCTION_ARGS_PAYLOAD_OFFSET,
            bound_args_payload_local,
            function,
        );
    }

    fn emit_concat_argv_payloads(
        &mut self,
        lhs_payload_local: u32,
        rhs_payload_local: u32,
        result_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let lhs_len_local = self.reserve_temp_local();
        let rhs_len_local = self.reserve_temp_local();
        let total_len_local = self.reserve_temp_local();
        let dst_payload_local = self.reserve_temp_local();
        let dst_buffer_local = self.reserve_temp_local();
        let lhs_index_local = self.reserve_temp_local();
        let rhs_index_local = self.reserve_temp_local();
        let dst_index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            lhs_payload_local,
            HEAP_LEN_OFFSET,
            lhs_len_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            rhs_payload_local,
            HEAP_LEN_OFFSET,
            rhs_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(lhs_len_local));
        function.instruction(&Instruction::LocalGet(rhs_len_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(total_len_local));

        self.emit_alloc_array_with_len_local(
            total_len_local,
            dst_payload_local,
            dst_buffer_local,
            function,
        )?;

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(lhs_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(dst_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(lhs_index_local));
        function.instruction(&Instruction::LocalGet(lhs_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            lhs_payload_local,
            lhs_index_local,
            value_payload_local,
            value_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(dst_buffer_local));
        function.instruction(&Instruction::LocalGet(dst_index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_ARRAY_TAG_OFFSET,
            value_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_ARRAY_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(lhs_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(lhs_index_local));
        function.instruction(&Instruction::LocalGet(dst_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(dst_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(rhs_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(rhs_index_local));
        function.instruction(&Instruction::LocalGet(rhs_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            rhs_payload_local,
            rhs_index_local,
            value_payload_local,
            value_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(dst_buffer_local));
        function.instruction(&Instruction::LocalGet(dst_index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_ARRAY_TAG_OFFSET,
            value_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_ARRAY_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(rhs_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(rhs_index_local));
        function.instruction(&Instruction::LocalGet(dst_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(dst_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(dst_payload_local));
        function.instruction(&Instruction::LocalSet(result_payload_local));

        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(dst_index_local);
        self.release_temp_local(rhs_index_local);
        self.release_temp_local(lhs_index_local);
        self.release_temp_local(dst_buffer_local);
        self.release_temp_local(dst_payload_local);
        self.release_temp_local(total_len_local);
        self.release_temp_local(rhs_len_local);
        self.release_temp_local(lhs_len_local);
        Ok(())
    }

    fn emit_alloc_array_with_len_local(
        &mut self,
        len_local: u32,
        payload_local: u32,
        buffer_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let cap_local = self.reserve_temp_local();
        let size_local = self.reserve_temp_local();

        self.emit_heap_alloc_const(HEAP_HEADER_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(MIN_HEAP_CAPACITY as i64));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(cap_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(size_local));
        self.emit_heap_alloc_from_local(size_local, function)?;
        function.instruction(&Instruction::LocalSet(buffer_local));
        self.store_i64_local_at_offset(payload_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.store_i64_local_at_offset(payload_local, HEAP_LEN_OFFSET, len_local, function);
        self.store_i64_local_at_offset(payload_local, HEAP_CAP_OFFSET, cap_local, function);
        function.instruction(&Instruction::GlobalGet(ARRAY_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.store_i64_local_at_offset(
            payload_local,
            HEAP_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );

        self.release_temp_local(size_local);
        self.release_temp_local(cap_local);
        Ok(())
    }

    fn emit_unwrap_bound_new_target(
        &mut self,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        let flags_local = self.reserve_temp_local();
        let record_local = self.reserve_temp_local();
        let next_payload_local = self.reserve_temp_local();
        let next_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_function_flags(payload_local, flags_local, function);
        function.instruction(&Instruction::LocalGet(flags_local));
        function.instruction(&Instruction::I64Const(FUNCTION_FLAG_BOUND as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        self.load_i64_to_local_from_offset(
            payload_local,
            HEAP_FUNCTION_ENV_HANDLE_OFFSET,
            record_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_BOUND_FUNCTION_TARGET_PAYLOAD_OFFSET,
            next_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_BOUND_FUNCTION_TARGET_TAG_OFFSET,
            next_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(next_payload_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(next_tag_local));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(next_tag_local);
        self.release_temp_local(next_payload_local);
        self.release_temp_local(record_local);
        self.release_temp_local(flags_local);
    }

    fn emit_alloc_bound_function_value(
        &mut self,
        target_payload_local: u32,
        target_tag_local: u32,
        bound_this_payload_local: u32,
        bound_this_tag_local: u32,
        bound_args_payload_local: u32,
        payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let meta = self
            .functions
            .get(&StandardBuiltinId::BoundFunctionInvoker.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `[[BoundFunctionInvoke]]`",
                )
            })?;
        let object_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let record_local = self.reserve_temp_local();
        let flags_local = self.reserve_temp_local();
        let prototype_key_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();
        let prototype_tag_local = self.reserve_temp_local();

        self.emit_heap_alloc_const(HEAP_BOUND_FUNCTION_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(record_local));
        self.store_i64_local_at_offset(
            record_local,
            HEAP_BOUND_FUNCTION_TARGET_TAG_OFFSET,
            target_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_BOUND_FUNCTION_TARGET_PAYLOAD_OFFSET,
            target_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_BOUND_FUNCTION_THIS_TAG_OFFSET,
            bound_this_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_BOUND_FUNCTION_THIS_PAYLOAD_OFFSET,
            bound_this_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_BOUND_FUNCTION_ARGS_PAYLOAD_OFFSET,
            bound_args_payload_local,
            function,
        );

        self.emit_load_function_constructable_flag(target_payload_local, flags_local, function);
        self.emit_heap_alloc_const(HEAP_FUNCTION_OBJECT_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(object_local));
        self.emit_heap_alloc_const(MIN_HEAP_CAPACITY * HEAP_OBJECT_ENTRY_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(buffer_local));
        self.store_i64_local_at_offset(object_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.store_i64_const_at_offset(object_local, HEAP_LEN_OFFSET, 0, function);
        self.store_i64_const_at_offset(object_local, HEAP_CAP_OFFSET, MIN_HEAP_CAPACITY, function);
        function.instruction(&Instruction::GlobalGet(FUNCTION_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.store_i64_local_at_offset(
            object_local,
            HEAP_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_const_at_offset(
            object_local,
            HEAP_FUNCTION_TABLE_INDEX_OFFSET,
            meta.table_index as u64,
            function,
        );
        self.store_i64_local_at_offset(
            object_local,
            HEAP_FUNCTION_ENV_HANDLE_OFFSET,
            record_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(flags_local));
        function.instruction(&Instruction::I64Const(FUNCTION_FLAG_BOUND as i64));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(flags_local));
        self.store_i64_local_at_offset(
            object_local,
            HEAP_FUNCTION_FLAGS_OFFSET,
            flags_local,
            function,
        );
        self.store_i64_const_at_offset(
            object_local,
            HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
            ValueKind::Undefined.tag() as u64,
            function,
        );
        self.store_i64_const_at_offset(
            object_local,
            HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(
            object_local,
            HEAP_FUNCTION_TO_STRING_PAYLOAD_OFFSET,
            self.strings.payload(meta.to_string_value.as_str()) as u64,
            function,
        );

        function.instruction(&Instruction::LocalGet(flags_local));
        function.instruction(&Instruction::I64Const(FUNCTION_FLAG_CONSTRUCTABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
        function.instruction(&Instruction::LocalSet(prototype_key_local));
        self.emit_object_read(
            target_payload_local,
            target_tag_local,
            target_payload_local,
            target_tag_local,
            prototype_key_local,
            prototype_payload_local,
            prototype_tag_local,
            function,
        )?;
        self.store_i64_local_at_offset(
            object_local,
            HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
            prototype_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            object_local,
            HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
            prototype_payload_local,
            function,
        );
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalSet(payload_local));

        self.release_temp_local(prototype_tag_local);
        self.release_temp_local(prototype_payload_local);
        self.release_temp_local(prototype_key_local);
        self.release_temp_local(flags_local);
        self.release_temp_local(record_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(object_local);
        Ok(())
    }

    fn emit_function_handle_construct_with_argv(
        &mut self,
        callee_payload_local: u32,
        callee_tag_local: u32,
        new_target_payload_local: u32,
        new_target_tag_local: u32,
        argc_local: u32,
        argv_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let callee_env_local = self.reserve_temp_local();
        let table_index_local = self.reserve_temp_local();
        let proto_key_local = self.reserve_temp_local();
        let proto_payload_local = self.reserve_temp_local();
        let proto_tag_local = self.reserve_temp_local();
        let instance_local = self.reserve_temp_local();
        let call_payload_local = self.reserve_temp_local();
        let call_tag_local = self.reserve_temp_local();
        let call_completion_local = self.reserve_temp_local();

        self.emit_load_function_object_fields(
            callee_payload_local,
            callee_env_local,
            table_index_local,
            function,
        );
        function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
        function.instruction(&Instruction::LocalSet(proto_key_local));
        self.emit_object_read(
            callee_payload_local,
            callee_tag_local,
            callee_payload_local,
            callee_tag_local,
            proto_key_local,
            proto_payload_local,
            proto_tag_local,
            function,
        )?;

        function.instruction(&Instruction::Block(BlockType::Empty));
        self.emit_is_heap_object_like_tag_i32(proto_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_plain_object_with_prototype(Some(proto_payload_local), None, function)?;
        function.instruction(&Instruction::LocalSet(instance_local));
        function.instruction(&Instruction::Else);
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(instance_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(callee_env_local));
        function.instruction(&Instruction::LocalGet(instance_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalGet(new_target_payload_local));
        function.instruction(&Instruction::LocalGet(new_target_tag_local));
        function.instruction(&Instruction::LocalGet(argc_local));
        function.instruction(&Instruction::LocalGet(argv_local));
        function.instruction(&Instruction::LocalGet(table_index_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::CallIndirect {
            type_index: JS_FUNCTION_TYPE_INDEX,
            table_index: 0,
        });
        self.store_call_results_to(
            call_payload_local,
            call_tag_local,
            call_completion_local,
            self.completion_aux_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(call_completion_local));
        function.instruction(&Instruction::LocalSet(self.completion_local));
        self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
            call_payload_local,
            call_tag_local,
            1,
            function,
        );

        self.emit_is_heap_object_like_tag_i32(call_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(call_payload_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(call_tag_local));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(instance_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(call_completion_local);
        self.release_temp_local(call_tag_local);
        self.release_temp_local(call_payload_local);
        self.release_temp_local(instance_local);
        self.release_temp_local(proto_tag_local);
        self.release_temp_local(proto_payload_local);
        self.release_temp_local(proto_key_local);
        self.release_temp_local(table_index_local);
        self.release_temp_local(callee_env_local);
        Ok(())
    }

    fn compile_standard_builtin(
        &mut self,
        builtin: StandardBuiltinId,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match builtin {
            StandardBuiltinId::FunctionConstructor => {
                self.emit_throw_runtime_error(
                    TYPE_ERROR_NAME,
                    "dynamic Function constructor unsupported",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
            }
            StandardBuiltinId::FunctionPrototypeCall => {
                let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: missing Function.prototype.call receiver",
                    )
                })?;
                let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: missing Function.prototype.call receiver",
                    )
                })?;
                let this_arg_payload_local = self.reserve_temp_local();
                let this_arg_tag_local = self.reserve_temp_local();
                let call_this_payload_local = self.reserve_temp_local();
                let call_this_tag_local = self.reserve_temp_local();
                let argc_local = self.reserve_temp_local();
                let argv_local = self.reserve_temp_local();

                self.emit_builtin_arg_to_locals(
                    0,
                    this_arg_payload_local,
                    this_arg_tag_local,
                    function,
                );
                self.emit_adapt_call_this_arg(
                    this_arg_payload_local,
                    this_arg_tag_local,
                    call_this_payload_local,
                    call_this_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(self.argc_param_local()));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64GtU);
                function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
                function.instruction(&Instruction::LocalGet(self.argc_param_local()));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Sub);
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::LocalSet(argc_local));
                self.emit_rest_array_payload(1, function)?;
                function.instruction(&Instruction::LocalSet(argv_local));

                function.instruction(&Instruction::LocalGet(receiver_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_function_handle_call_with_argv(
                    receiver_payload_local,
                    receiver_tag_local,
                    Some((call_this_payload_local, Some(call_this_tag_local))),
                    argc_local,
                    argv_local,
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::Else);
                self.emit_throw_runtime_error(
                    TYPE_ERROR_NAME,
                    "Function.prototype.call receiver is not callable",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
                    self.result_local,
                    self.result_tag_local,
                    1,
                    function,
                );
                function.instruction(&Instruction::End);

                self.release_temp_local(argv_local);
                self.release_temp_local(argc_local);
                self.release_temp_local(call_this_tag_local);
                self.release_temp_local(call_this_payload_local);
                self.release_temp_local(this_arg_tag_local);
                self.release_temp_local(this_arg_payload_local);
            }
            StandardBuiltinId::FunctionPrototypeApply => {
                let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: missing Function.prototype.apply receiver",
                    )
                })?;
                let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: missing Function.prototype.apply receiver",
                    )
                })?;
                let this_arg_payload_local = self.reserve_temp_local();
                let this_arg_tag_local = self.reserve_temp_local();
                let apply_args_payload_local = self.reserve_temp_local();
                let apply_args_tag_local = self.reserve_temp_local();
                let call_this_payload_local = self.reserve_temp_local();
                let call_this_tag_local = self.reserve_temp_local();
                let argc_local = self.reserve_temp_local();
                let argv_local = self.reserve_temp_local();

                self.emit_builtin_arg_to_locals(
                    0,
                    this_arg_payload_local,
                    this_arg_tag_local,
                    function,
                );
                self.emit_builtin_arg_to_locals(
                    1,
                    apply_args_payload_local,
                    apply_args_tag_local,
                    function,
                );
                self.emit_adapt_call_this_arg(
                    this_arg_payload_local,
                    this_arg_tag_local,
                    call_this_payload_local,
                    call_this_tag_local,
                    function,
                )?;
                self.emit_array_like_snapshot_payload(
                    apply_args_payload_local,
                    apply_args_tag_local,
                    argv_local,
                    "Function.prototype.apply argument list must be array or arguments",
                    function,
                )?;
                self.load_i64_to_local_from_offset(
                    argv_local,
                    HEAP_LEN_OFFSET,
                    argc_local,
                    function,
                );

                function.instruction(&Instruction::LocalGet(receiver_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_function_handle_call_with_argv(
                    receiver_payload_local,
                    receiver_tag_local,
                    Some((call_this_payload_local, Some(call_this_tag_local))),
                    argc_local,
                    argv_local,
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::Else);
                self.emit_throw_runtime_error(
                    TYPE_ERROR_NAME,
                    "Function.prototype.apply receiver is not callable",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
                    self.result_local,
                    self.result_tag_local,
                    1,
                    function,
                );
                function.instruction(&Instruction::End);

                self.release_temp_local(argv_local);
                self.release_temp_local(argc_local);
                self.release_temp_local(call_this_tag_local);
                self.release_temp_local(call_this_payload_local);
                self.release_temp_local(apply_args_tag_local);
                self.release_temp_local(apply_args_payload_local);
                self.release_temp_local(this_arg_tag_local);
                self.release_temp_local(this_arg_payload_local);
            }
            StandardBuiltinId::FunctionPrototypeBind => {
                let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: missing Function.prototype.bind receiver",
                    )
                })?;
                let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: missing Function.prototype.bind receiver",
                    )
                })?;
                let this_arg_payload_local = self.reserve_temp_local();
                let this_arg_tag_local = self.reserve_temp_local();
                let bound_this_payload_local = self.reserve_temp_local();
                let bound_this_tag_local = self.reserve_temp_local();
                let bound_args_payload_local = self.reserve_temp_local();

                self.emit_builtin_arg_to_locals(
                    0,
                    this_arg_payload_local,
                    this_arg_tag_local,
                    function,
                );
                self.emit_adapt_call_this_arg(
                    this_arg_payload_local,
                    this_arg_tag_local,
                    bound_this_payload_local,
                    bound_this_tag_local,
                    function,
                )?;
                self.emit_rest_array_payload(1, function)?;
                function.instruction(&Instruction::LocalSet(bound_args_payload_local));

                function.instruction(&Instruction::LocalGet(receiver_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_alloc_bound_function_value(
                    receiver_payload_local,
                    receiver_tag_local,
                    bound_this_payload_local,
                    bound_this_tag_local,
                    bound_args_payload_local,
                    self.result_local,
                    function,
                )?;
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                function.instruction(&Instruction::Else);
                self.emit_throw_runtime_error(
                    TYPE_ERROR_NAME,
                    "Function.prototype.bind receiver is not callable",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
                    self.result_local,
                    self.result_tag_local,
                    1,
                    function,
                );
                function.instruction(&Instruction::End);

                self.release_temp_local(bound_args_payload_local);
                self.release_temp_local(bound_this_tag_local);
                self.release_temp_local(bound_this_payload_local);
                self.release_temp_local(this_arg_tag_local);
                self.release_temp_local(this_arg_payload_local);
            }
            StandardBuiltinId::ObjectConstructor => {
                let arg_payload_local = self.reserve_temp_local();
                let arg_tag_local = self.reserve_temp_local();
                self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
                function.instruction(&Instruction::Block(BlockType::Empty));
                for kind in [
                    ValueKind::Object,
                    ValueKind::Array,
                    ValueKind::Function,
                    ValueKind::Arguments,
                ] {
                    function.instruction(&Instruction::LocalGet(arg_tag_local));
                    function.instruction(&Instruction::I64Const(kind.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::LocalGet(arg_payload_local));
                    function.instruction(&Instruction::LocalSet(self.result_local));
                    function.instruction(&Instruction::LocalGet(arg_tag_local));
                    function.instruction(&Instruction::LocalSet(self.result_tag_local));
                    function.instruction(&Instruction::Br(1));
                    function.instruction(&Instruction::End);
                }
                function.instruction(&Instruction::LocalGet(arg_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_alloc_boxed_wrapper_from_locals(
                    NUMBER_PROTOTYPE_GLOBAL_INDEX,
                    BOXED_PRIMITIVE_KIND_NUMBER,
                    arg_payload_local,
                    arg_tag_local,
                    self.result_local,
                    function,
                )?;
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                function.instruction(&Instruction::Br(1));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::LocalGet(arg_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_alloc_boxed_wrapper_from_locals(
                    STRING_PROTOTYPE_GLOBAL_INDEX,
                    BOXED_PRIMITIVE_KIND_STRING,
                    arg_payload_local,
                    arg_tag_local,
                    self.result_local,
                    function,
                )?;
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                function.instruction(&Instruction::Br(1));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::LocalGet(arg_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_alloc_boxed_wrapper_from_locals(
                    BOOLEAN_PROTOTYPE_GLOBAL_INDEX,
                    BOXED_PRIMITIVE_KIND_BOOLEAN,
                    arg_payload_local,
                    arg_tag_local,
                    self.result_local,
                    function,
                )?;
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                function.instruction(&Instruction::Br(1));
                function.instruction(&Instruction::End);
                self.emit_alloc_plain_object_with_prototype(
                    None,
                    Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
                    function,
                )?;
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                function.instruction(&Instruction::End);
                self.release_temp_local(arg_tag_local);
                self.release_temp_local(arg_payload_local);
            }
            StandardBuiltinId::ObjectCreate => {
                let arg_payload_local = self.reserve_temp_local();
                let arg_tag_local = self.reserve_temp_local();
                self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
                function.instruction(&Instruction::Block(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(arg_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_alloc_plain_object_with_prototype(None, None, function)?;
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                function.instruction(&Instruction::Br(1));
                function.instruction(&Instruction::End);
                self.emit_alloc_plain_object_with_prototype(
                    Some(arg_payload_local),
                    None,
                    function,
                )?;
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                function.instruction(&Instruction::End);
                self.release_temp_local(arg_tag_local);
                self.release_temp_local(arg_payload_local);
            }
            StandardBuiltinId::ObjectGetPrototypeOf => {
                let arg_payload_local = self.reserve_temp_local();
                let arg_tag_local = self.reserve_temp_local();
                self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
                self.load_i64_to_local_from_offset(
                    arg_payload_local,
                    HEAP_PROTOTYPE_OFFSET,
                    self.result_local,
                    function,
                );
                function.instruction(&Instruction::LocalGet(self.result_local));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                function.instruction(&Instruction::End);
                self.release_temp_local(arg_tag_local);
                self.release_temp_local(arg_payload_local);
            }
            StandardBuiltinId::ArrayConstructor => {
                self.emit_rest_array_payload(0, function)?;
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
            }
            StandardBuiltinId::ArrayIsArray => {
                let arg_payload_local = self.reserve_temp_local();
                let arg_tag_local = self.reserve_temp_local();
                self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
                function.instruction(&Instruction::LocalGet(arg_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                self.release_temp_local(arg_tag_local);
                self.release_temp_local(arg_payload_local);
            }
            StandardBuiltinId::ArrayBufferConstructor => {
                let arg_payload_local = self.reserve_temp_local();
                let arg_tag_local = self.reserve_temp_local();
                let length_payload_local = self.reserve_temp_local();
                let byte_length_local = self.reserve_temp_local();
                let data_ptr_local = self.reserve_temp_local();
                let object_local = self.reserve_temp_local();
                let zero_index_local = self.reserve_temp_local();

                self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
                function.instruction(&Instruction::LocalGet(self.argc_param_local()));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                function.instruction(&Instruction::I64ReinterpretF64);
                function.instruction(&Instruction::LocalSet(length_payload_local));
                function.instruction(&Instruction::Else);
                self.emit_value_to_number_payload(arg_tag_local, arg_payload_local, function)?;
                function.instruction(&Instruction::LocalSet(length_payload_local));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::LocalGet(length_payload_local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::I64TruncF64U);
                function.instruction(&Instruction::LocalSet(byte_length_local));

                self.emit_heap_alloc_from_local(byte_length_local, function)?;
                function.instruction(&Instruction::LocalSet(data_ptr_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(zero_index_local));
                function.instruction(&Instruction::Block(BlockType::Empty));
                function.instruction(&Instruction::Loop(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(zero_index_local));
                function.instruction(&Instruction::LocalGet(byte_length_local));
                function.instruction(&Instruction::I64GeU);
                function.instruction(&Instruction::BrIf(1));
                function.instruction(&Instruction::LocalGet(data_ptr_local));
                function.instruction(&Instruction::LocalGet(zero_index_local));
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::I32Const(0));
                function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
                function.instruction(&Instruction::LocalGet(zero_index_local));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::LocalSet(zero_index_local));
                function.instruction(&Instruction::Br(0));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);

                self.emit_alloc_plain_object_with_prototype(
                    None,
                    Some(ARRAY_BUFFER_PROTOTYPE_GLOBAL_INDEX),
                    function,
                )?;
                function.instruction(&Instruction::LocalSet(object_local));
                self.emit_object_define_number_data_from_i64_local(
                    object_local,
                    ARRAY_BUFFER_DATA_PTR_SLOT,
                    data_ptr_local,
                    function,
                )?;
                self.emit_object_define_number_data_from_i64_local(
                    object_local,
                    ARRAY_BUFFER_BYTE_LENGTH_SLOT,
                    byte_length_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(object_local));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));

                self.release_temp_local(zero_index_local);
                self.release_temp_local(object_local);
                self.release_temp_local(data_ptr_local);
                self.release_temp_local(byte_length_local);
                self.release_temp_local(length_payload_local);
                self.release_temp_local(arg_tag_local);
                self.release_temp_local(arg_payload_local);
            }
            StandardBuiltinId::DataViewConstructor => {
                let buffer_payload_local = self.reserve_temp_local();
                let buffer_tag_local = self.reserve_temp_local();
                let offset_payload_local = self.reserve_temp_local();
                let offset_tag_local = self.reserve_temp_local();
                let length_payload_local = self.reserve_temp_local();
                let length_tag_local = self.reserve_temp_local();
                let data_ptr_local = self.reserve_temp_local();
                let buffer_byte_length_local = self.reserve_temp_local();
                let byte_offset_local = self.reserve_temp_local();
                let byte_length_local = self.reserve_temp_local();
                let object_local = self.reserve_temp_local();

                self.emit_builtin_arg_to_locals(0, buffer_payload_local, buffer_tag_local, function);
                self.emit_builtin_arg_to_locals(1, offset_payload_local, offset_tag_local, function);
                self.emit_builtin_arg_to_locals(2, length_payload_local, length_tag_local, function);
                self.emit_object_read_number_slot_to_i64_local(
                    buffer_payload_local,
                    ARRAY_BUFFER_DATA_PTR_SLOT,
                    data_ptr_local,
                    function,
                )?;
                self.emit_object_read_number_slot_to_i64_local(
                    buffer_payload_local,
                    ARRAY_BUFFER_BYTE_LENGTH_SLOT,
                    buffer_byte_length_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(self.argc_param_local()));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64GtU);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_value_to_number_payload(offset_tag_local, offset_payload_local, function)?;
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::I64TruncF64U);
                function.instruction(&Instruction::LocalSet(byte_offset_local));
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(byte_offset_local));
                function.instruction(&Instruction::End);

                function.instruction(&Instruction::LocalGet(self.argc_param_local()));
                function.instruction(&Instruction::I64Const(2));
                function.instruction(&Instruction::I64GtU);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_value_to_number_payload(length_tag_local, length_payload_local, function)?;
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::I64TruncF64U);
                function.instruction(&Instruction::LocalSet(byte_length_local));
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::LocalGet(buffer_byte_length_local));
                function.instruction(&Instruction::LocalGet(byte_offset_local));
                function.instruction(&Instruction::I64Sub);
                function.instruction(&Instruction::LocalSet(byte_length_local));
                function.instruction(&Instruction::End);

                self.emit_alloc_plain_object_with_prototype(
                    None,
                    Some(DATA_VIEW_PROTOTYPE_GLOBAL_INDEX),
                    function,
                )?;
                function.instruction(&Instruction::LocalSet(object_local));
                self.emit_object_define_number_data_from_i64_local(
                    object_local,
                    DATA_VIEW_DATA_PTR_SLOT,
                    data_ptr_local,
                    function,
                )?;
                self.emit_object_define_number_data_from_i64_local(
                    object_local,
                    DATA_VIEW_BYTE_OFFSET_SLOT,
                    byte_offset_local,
                    function,
                )?;
                self.emit_object_define_number_data_from_i64_local(
                    object_local,
                    DATA_VIEW_BYTE_LENGTH_SLOT,
                    byte_length_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(object_local));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));

                self.release_temp_local(object_local);
                self.release_temp_local(byte_length_local);
                self.release_temp_local(byte_offset_local);
                self.release_temp_local(buffer_byte_length_local);
                self.release_temp_local(data_ptr_local);
                self.release_temp_local(length_tag_local);
                self.release_temp_local(length_payload_local);
                self.release_temp_local(offset_tag_local);
                self.release_temp_local(offset_payload_local);
                self.release_temp_local(buffer_tag_local);
                self.release_temp_local(buffer_payload_local);
            }
            StandardBuiltinId::DataViewPrototypeGetUint8 => {
                let this_payload_local = self.this_payload_local.ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: missing DataView receiver",
                    )
                })?;
                let index_payload_local = self.reserve_temp_local();
                let index_tag_local = self.reserve_temp_local();
                let data_ptr_local = self.reserve_temp_local();
                let byte_offset_local = self.reserve_temp_local();
                let byte_length_local = self.reserve_temp_local();
                let index_local = self.reserve_temp_local();
                let address_local = self.reserve_temp_local();

                self.emit_builtin_arg_to_locals(0, index_payload_local, index_tag_local, function);
                self.emit_object_read_number_slot_to_i64_local(
                    this_payload_local,
                    DATA_VIEW_DATA_PTR_SLOT,
                    data_ptr_local,
                    function,
                )?;
                self.emit_object_read_number_slot_to_i64_local(
                    this_payload_local,
                    DATA_VIEW_BYTE_OFFSET_SLOT,
                    byte_offset_local,
                    function,
                )?;
                self.emit_object_read_number_slot_to_i64_local(
                    this_payload_local,
                    DATA_VIEW_BYTE_LENGTH_SLOT,
                    byte_length_local,
                    function,
                )?;
                self.emit_value_to_number_payload(index_tag_local, index_payload_local, function)?;
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::I64TruncF64U);
                function.instruction(&Instruction::LocalSet(index_local));
                function.instruction(&Instruction::LocalGet(index_local));
                function.instruction(&Instruction::LocalGet(byte_length_local));
                function.instruction(&Instruction::I64GeU);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_throw_runtime_error(
                    RANGE_ERROR_NAME,
                    "DataView getUint8 index out of bounds",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
                    self.result_local,
                    self.result_tag_local,
                    1,
                    function,
                );
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::LocalGet(data_ptr_local));
                function.instruction(&Instruction::LocalGet(byte_offset_local));
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::LocalGet(index_local));
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::LocalSet(address_local));
                function.instruction(&Instruction::LocalGet(address_local));
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::F64ConvertI64U);
                function.instruction(&Instruction::I64ReinterpretF64);
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));

                self.release_temp_local(address_local);
                self.release_temp_local(index_local);
                self.release_temp_local(byte_length_local);
                self.release_temp_local(byte_offset_local);
                self.release_temp_local(data_ptr_local);
                self.release_temp_local(index_tag_local);
                self.release_temp_local(index_payload_local);
            }
            StandardBuiltinId::NumberConstructor
            | StandardBuiltinId::StringConstructor
            | StandardBuiltinId::BooleanConstructor => {
                let arg_payload_local = self.reserve_temp_local();
                let arg_tag_local = self.reserve_temp_local();
                let primitive_payload_local = self.reserve_temp_local();
                let primitive_tag_local = self.reserve_temp_local();
                let has_arg_local = self.reserve_temp_local();
                self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
                function.instruction(&Instruction::LocalGet(self.argc_param_local()));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64GtU);
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::LocalSet(has_arg_local));
                match builtin {
                    StandardBuiltinId::NumberConstructor => {
                        function.instruction(&Instruction::LocalGet(has_arg_local));
                        function.instruction(&Instruction::I64Eqz);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(primitive_payload_local));
                        function
                            .instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                        function.instruction(&Instruction::LocalSet(primitive_tag_local));
                        function.instruction(&Instruction::Else);
                        self.emit_value_to_number_payload(
                            arg_tag_local,
                            arg_payload_local,
                            function,
                        )?;
                        function.instruction(&Instruction::LocalSet(primitive_payload_local));
                        function
                            .instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                        function.instruction(&Instruction::LocalSet(primitive_tag_local));
                        function.instruction(&Instruction::End);
                    }
                    StandardBuiltinId::StringConstructor => {
                        function.instruction(&Instruction::LocalGet(has_arg_local));
                        function.instruction(&Instruction::I64Eqz);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::I64Const(self.strings.payload("")));
                        function.instruction(&Instruction::LocalSet(primitive_payload_local));
                        function
                            .instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                        function.instruction(&Instruction::LocalSet(primitive_tag_local));
                        function.instruction(&Instruction::Else);
                        self.emit_value_to_string_payload(
                            arg_payload_local,
                            arg_tag_local,
                            function,
                        )?;
                        function.instruction(&Instruction::LocalSet(primitive_payload_local));
                        function
                            .instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                        function.instruction(&Instruction::LocalSet(primitive_tag_local));
                        function.instruction(&Instruction::End);
                    }
                    StandardBuiltinId::BooleanConstructor => {
                        function.instruction(&Instruction::LocalGet(has_arg_local));
                        function.instruction(&Instruction::I64Eqz);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::I64Const(0));
                        function.instruction(&Instruction::LocalSet(primitive_payload_local));
                        function.instruction(&Instruction::Else);
                        self.compile_truthy_tagged_i32(arg_tag_local, arg_payload_local, function)?;
                        function.instruction(&Instruction::I64ExtendI32U);
                        function.instruction(&Instruction::LocalSet(primitive_payload_local));
                        function.instruction(&Instruction::End);
                        function
                            .instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                        function.instruction(&Instruction::LocalSet(primitive_tag_local));
                    }
                    _ => unreachable!(),
                }
                function.instruction(&Instruction::LocalGet(self.new_target_tag_local().unwrap()));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(primitive_payload_local));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::LocalGet(primitive_tag_local));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                function.instruction(&Instruction::Else);
                self.emit_alloc_boxed_wrapper_for_builtin(
                    builtin,
                    primitive_payload_local,
                    primitive_tag_local,
                    self.result_local,
                    function,
                )?;
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                function.instruction(&Instruction::End);
                self.release_temp_local(has_arg_local);
                self.release_temp_local(primitive_tag_local);
                self.release_temp_local(primitive_payload_local);
                self.release_temp_local(arg_tag_local);
                self.release_temp_local(arg_payload_local);
            }
            StandardBuiltinId::ErrorConstructor
            | StandardBuiltinId::EvalErrorConstructor
            | StandardBuiltinId::AggregateErrorConstructor
            | StandardBuiltinId::RangeErrorConstructor
            | StandardBuiltinId::SyntaxErrorConstructor
            | StandardBuiltinId::TypeErrorConstructor
            | StandardBuiltinId::URIErrorConstructor
            | StandardBuiltinId::ReferenceErrorConstructor => {
                if builtin == StandardBuiltinId::AggregateErrorConstructor {
                    let errors_arg_payload_local = self.reserve_temp_local();
                    let errors_arg_tag_local = self.reserve_temp_local();
                    let message_arg_payload_local = self.reserve_temp_local();
                    let message_arg_tag_local = self.reserve_temp_local();
                    let errors_payload_local = self.reserve_temp_local();
                    let message_payload_local = self.reserve_temp_local();
                    self.emit_builtin_arg_to_locals(
                        0,
                        errors_arg_payload_local,
                        errors_arg_tag_local,
                        function,
                    );
                    self.emit_builtin_arg_to_locals(
                        1,
                        message_arg_payload_local,
                        message_arg_tag_local,
                        function,
                    );
                    self.emit_array_like_snapshot_payload(
                        errors_arg_payload_local,
                        errors_arg_tag_local,
                        errors_payload_local,
                        "AggregateError errors input must be array or arguments",
                        function,
                    )?;
                    function.instruction(&Instruction::LocalGet(message_arg_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_alloc_aggregate_error_instance_from_locals(
                        None,
                        errors_payload_local,
                        self.result_local,
                        self.result_tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::Else);
                    self.emit_value_to_string_payload(
                        message_arg_payload_local,
                        message_arg_tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::LocalSet(message_payload_local));
                    self.emit_alloc_aggregate_error_instance_from_locals(
                        Some(message_payload_local),
                        errors_payload_local,
                        self.result_local,
                        self.result_tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::End);
                    self.release_temp_local(message_payload_local);
                    self.release_temp_local(errors_payload_local);
                    self.release_temp_local(message_arg_tag_local);
                    self.release_temp_local(message_arg_payload_local);
                    self.release_temp_local(errors_arg_tag_local);
                    self.release_temp_local(errors_arg_payload_local);
                    return Ok(());
                }
                let arg_payload_local = self.reserve_temp_local();
                let arg_tag_local = self.reserve_temp_local();
                let message_payload_local = self.reserve_temp_local();
                self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
                function.instruction(&Instruction::LocalGet(arg_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_alloc_error_instance_from_locals(
                    builtin.debug_name(),
                    None,
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::Else);
                self.emit_value_to_string_payload(arg_payload_local, arg_tag_local, function)?;
                function.instruction(&Instruction::LocalSet(message_payload_local));
                self.emit_alloc_error_instance_from_locals(
                    builtin.debug_name(),
                    Some(message_payload_local),
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::End);
                self.release_temp_local(message_payload_local);
                self.release_temp_local(arg_tag_local);
                self.release_temp_local(arg_payload_local);
            }
            StandardBuiltinId::FunctionPrototypeToString => {
                let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: missing Function.prototype.toString receiver",
                    )
                })?;
                let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: missing Function.prototype.toString receiver",
                    )
                })?;
                function.instruction(&Instruction::LocalGet(receiver_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.load_i64_to_local_from_offset(
                    receiver_payload_local,
                    HEAP_FUNCTION_TO_STRING_PAYLOAD_OFFSET,
                    self.result_local,
                    function,
                );
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                function.instruction(&Instruction::Else);
                self.emit_throw_runtime_error(
                    TYPE_ERROR_NAME,
                    "Function.prototype.toString receiver is not callable",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::End);
            }
            StandardBuiltinId::ErrorPrototypeToString => {
                let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: missing Error.prototype.toString receiver",
                    )
                })?;
                let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: missing Error.prototype.toString receiver",
                    )
                })?;
                let name_key_local = self.reserve_temp_local();
                let message_key_local = self.reserve_temp_local();
                let name_payload_local = self.reserve_temp_local();
                let name_tag_local = self.reserve_temp_local();
                let message_payload_local = self.reserve_temp_local();
                let message_tag_local = self.reserve_temp_local();
                let name_string_local = self.reserve_temp_local();
                let message_string_local = self.reserve_temp_local();
                let separator_local = self.reserve_temp_local();

                function.instruction(&Instruction::LocalGet(receiver_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::LocalGet(receiver_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I32Or);
                function.instruction(&Instruction::LocalGet(receiver_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I32Or);
                function.instruction(&Instruction::LocalGet(receiver_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I32Or);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(self.strings.payload("name")));
                function.instruction(&Instruction::LocalSet(name_key_local));
                function.instruction(&Instruction::I64Const(self.strings.payload("message")));
                function.instruction(&Instruction::LocalSet(message_key_local));

                function.instruction(&Instruction::LocalGet(receiver_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::LocalGet(receiver_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I32Or);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_object_read(
                    receiver_payload_local,
                    receiver_tag_local,
                    receiver_payload_local,
                    receiver_tag_local,
                    name_key_local,
                    name_payload_local,
                    name_tag_local,
                    function,
                )?;
                self.emit_object_read(
                    receiver_payload_local,
                    receiver_tag_local,
                    receiver_payload_local,
                    receiver_tag_local,
                    message_key_local,
                    message_payload_local,
                    message_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(name_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::LocalSet(name_tag_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(message_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::LocalSet(message_tag_local));
                function.instruction(&Instruction::End);

                function.instruction(&Instruction::LocalGet(name_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(self.strings.payload(ERROR_NAME)));
                function.instruction(&Instruction::LocalSet(name_string_local));
                function.instruction(&Instruction::Else);
                self.emit_value_to_string_payload(name_payload_local, name_tag_local, function)?;
                function.instruction(&Instruction::LocalSet(name_string_local));
                function.instruction(&Instruction::End);

                function.instruction(&Instruction::LocalGet(message_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(self.strings.payload("")));
                function.instruction(&Instruction::LocalSet(message_string_local));
                function.instruction(&Instruction::Else);
                self.emit_value_to_string_payload(
                    message_payload_local,
                    message_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalSet(message_string_local));
                function.instruction(&Instruction::End);

                function.instruction(&Instruction::I64Const(self.strings.payload("")));
                function.instruction(&Instruction::LocalSet(separator_local));
                self.emit_string_payload_equality_i32(name_string_local, separator_local, function);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(message_string_local));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::Else);
                self.emit_string_payload_equality_i32(
                    message_string_local,
                    separator_local,
                    function,
                );
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(name_string_local));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::I64Const(self.strings.payload(": ")));
                function.instruction(&Instruction::LocalSet(separator_local));
                self.emit_concat_string_payloads_local(
                    name_string_local,
                    separator_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalSet(name_string_local));
                self.emit_concat_string_payloads_local(
                    name_string_local,
                    message_string_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                function.instruction(&Instruction::Else);
                self.emit_throw_runtime_error(
                    TYPE_ERROR_NAME,
                    "Error.prototype.toString receiver is not object",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
                    self.result_local,
                    self.result_tag_local,
                    1,
                    function,
                );
                function.instruction(&Instruction::End);

                self.release_temp_local(separator_local);
                self.release_temp_local(message_string_local);
                self.release_temp_local(name_string_local);
                self.release_temp_local(message_tag_local);
                self.release_temp_local(message_payload_local);
                self.release_temp_local(name_tag_local);
                self.release_temp_local(name_payload_local);
                self.release_temp_local(message_key_local);
                self.release_temp_local(name_key_local);
            }
            StandardBuiltinId::BoundFunctionInvoker => {
                let record_local = self.current_env_local;
                let target_payload_local = self.reserve_temp_local();
                let target_tag_local = self.reserve_temp_local();
                let bound_this_payload_local = self.reserve_temp_local();
                let bound_this_tag_local = self.reserve_temp_local();
                let bound_args_payload_local = self.reserve_temp_local();
                let merged_argv_local = self.reserve_temp_local();
                let merged_argc_local = self.reserve_temp_local();
                let forwarded_new_target_payload_local = self.reserve_temp_local();
                let forwarded_new_target_tag_local = self.reserve_temp_local();

                self.emit_load_bound_function_record(
                    record_local,
                    target_payload_local,
                    target_tag_local,
                    bound_this_payload_local,
                    bound_this_tag_local,
                    bound_args_payload_local,
                    function,
                );
                self.emit_concat_argv_payloads(
                    bound_args_payload_local,
                    self.argv_param_local(),
                    merged_argv_local,
                    function,
                )?;
                self.load_i64_to_local_from_offset(
                    merged_argv_local,
                    HEAP_LEN_OFFSET,
                    merged_argc_local,
                    function,
                );
                self.compile_new_target_to_locals(
                    forwarded_new_target_payload_local,
                    forwarded_new_target_tag_local,
                    function,
                );

                function.instruction(&Instruction::LocalGet(forwarded_new_target_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_function_handle_call_with_argv(
                    target_payload_local,
                    target_tag_local,
                    Some((bound_this_payload_local, Some(bound_this_tag_local))),
                    merged_argc_local,
                    merged_argv_local,
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::Else);
                self.emit_unwrap_bound_new_target(
                    forwarded_new_target_payload_local,
                    forwarded_new_target_tag_local,
                    function,
                );
                self.emit_function_handle_construct_with_argv(
                    target_payload_local,
                    target_tag_local,
                    forwarded_new_target_payload_local,
                    forwarded_new_target_tag_local,
                    merged_argc_local,
                    merged_argv_local,
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::End);

                self.release_temp_local(forwarded_new_target_tag_local);
                self.release_temp_local(forwarded_new_target_payload_local);
                self.release_temp_local(merged_argc_local);
                self.release_temp_local(merged_argv_local);
                self.release_temp_local(bound_args_payload_local);
                self.release_temp_local(bound_this_tag_local);
                self.release_temp_local(bound_this_payload_local);
                self.release_temp_local(target_tag_local);
                self.release_temp_local(target_payload_local);
            }
        }
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

        self.load_i64_to_local_from_offset(
            self.argv_param_local(),
            HEAP_PTR_OFFSET,
            src_buffer_local,
            function,
        );
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
        self.store_i64_local_at_offset(
            arguments_local,
            HEAP_CAP_OFFSET,
            self.scratch_local,
            function,
        );
        function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.store_i64_local_at_offset(
            arguments_local,
            HEAP_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_local_at_offset(
            arguments_local,
            HEAP_ARGUMENTS_MAPPED_COUNT_OFFSET,
            mapped_count_local,
            function,
        );
        if self.uses_mapped_arguments_object() {
            self.store_i64_local_at_offset(
                arguments_local,
                HEAP_ARGUMENTS_ENV_HANDLE_OFFSET,
                self.current_env_local,
                function,
            );
        } else {
            self.store_i64_const_at_offset(
                arguments_local,
                HEAP_ARGUMENTS_ENV_HANDLE_OFFSET,
                0,
                function,
            );
        }

        self.load_i64_to_local_from_offset(
            self.argv_param_local(),
            HEAP_PTR_OFFSET,
            src_buffer_local,
            function,
        );
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
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_LEN_OFFSET,
            payload_local,
            function,
        );
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

        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_ARGUMENTS_MAPPED_COUNT_OFFSET,
            mapped_count_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_ARGUMENTS_ENV_HANDLE_OFFSET,
            env_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_PTR_OFFSET,
            buffer_local,
            function,
        );
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
        function.instruction(&Instruction::I64Load(Self::memarg64(
            ENV_SLOT_BASE_OFFSET + ENV_SLOT_PAYLOAD_OFFSET,
        )));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(env_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(ENV_SLOT_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Add);
        function.instruction(&Instruction::I64Load(Self::memarg64(
            ENV_SLOT_BASE_OFFSET + ENV_SLOT_TAG_OFFSET,
        )));
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
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
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

        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_ARGUMENTS_MAPPED_COUNT_OFFSET,
            mapped_count_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_ARGUMENTS_ENV_HANDLE_OFFSET,
            env_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_PTR_OFFSET,
            buffer_local,
            function,
        );
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
        function.instruction(&Instruction::I64Store(Self::memarg64(
            ENV_SLOT_BASE_OFFSET + ENV_SLOT_TAG_OFFSET,
        )));
        function.instruction(&Instruction::LocalGet(env_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(ENV_SLOT_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Add);
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::I64Store(Self::memarg64(
            ENV_SLOT_BASE_OFFSET + ENV_SLOT_PAYLOAD_OFFSET,
        )));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_grow_buffer(
            arguments_local,
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

    fn emit_function_value_payload(
        &mut self,
        meta: &WasmFunctionMeta,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let prototype_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let proto_value_local = self.reserve_temp_local();
        let proto_tag_local = self.reserve_temp_local();

        self.emit_heap_alloc_const(HEAP_FUNCTION_OBJECT_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(object_local));
        self.emit_heap_alloc_const(MIN_HEAP_CAPACITY * HEAP_OBJECT_ENTRY_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(buffer_local));
        self.store_i64_local_at_offset(object_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.store_i64_const_at_offset(object_local, HEAP_LEN_OFFSET, 0, function);
        self.store_i64_const_at_offset(object_local, HEAP_CAP_OFFSET, MIN_HEAP_CAPACITY, function);
        function.instruction(&Instruction::GlobalGet(FUNCTION_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.store_i64_local_at_offset(
            object_local,
            HEAP_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_const_at_offset(
            object_local,
            HEAP_FUNCTION_TABLE_INDEX_OFFSET,
            meta.table_index as u64,
            function,
        );
        self.store_i64_local_at_offset(
            object_local,
            HEAP_FUNCTION_ENV_HANDLE_OFFSET,
            self.current_env_local,
            function,
        );
        self.store_i64_const_at_offset(
            object_local,
            HEAP_FUNCTION_FLAGS_OFFSET,
            if meta.constructable {
                FUNCTION_FLAG_CONSTRUCTABLE
                    | if meta.class_kind == ClassFunctionKind::Constructor {
                        FUNCTION_FLAG_CLASS_CONSTRUCTOR
                    } else {
                        0
                    }
            } else {
                0
            },
            function,
        );
        self.store_i64_const_at_offset(
            object_local,
            HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
            ValueKind::Undefined.tag() as u64,
            function,
        );
        self.store_i64_const_at_offset(
            object_local,
            HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(
            object_local,
            HEAP_FUNCTION_TO_STRING_PAYLOAD_OFFSET,
            self.strings.payload(meta.to_string_value.as_str()) as u64,
            function,
        );

        if meta.constructable {
            self.emit_alloc_plain_object_with_prototype(
                None,
                Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
                function,
            )?;
            function.instruction(&Instruction::LocalSet(prototype_local));
            function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
            function.instruction(&Instruction::LocalSet(proto_tag_local));
            self.store_i64_local_at_offset(
                object_local,
                HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
                proto_tag_local,
                function,
            );
            self.store_i64_local_at_offset(
                object_local,
                HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
                prototype_local,
                function,
            );
            function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
            function.instruction(&Instruction::LocalSet(key_local));
            function.instruction(&Instruction::LocalGet(prototype_local));
            function.instruction(&Instruction::LocalSet(proto_value_local));
            self.emit_object_define_data(
                object_local,
                key_local,
                proto_value_local,
                proto_tag_local,
                function,
            )?;

            function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
            function.instruction(&Instruction::LocalSet(key_local));
            function.instruction(&Instruction::LocalGet(object_local));
            function.instruction(&Instruction::LocalSet(proto_value_local));
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::LocalSet(proto_tag_local));
            self.emit_object_define_data(
                prototype_local,
                key_local,
                proto_value_local,
                proto_tag_local,
                function,
            )?;
        }

        function.instruction(&Instruction::LocalGet(object_local));
        self.release_temp_local(proto_tag_local);
        self.release_temp_local(proto_value_local);
        self.release_temp_local(key_local);
        self.release_temp_local(prototype_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(object_local);
        Ok(())
    }

    fn emit_load_function_object_fields(
        &mut self,
        function_object_local: u32,
        env_local: u32,
        table_index_local: u32,
        function: &mut Function,
    ) {
        self.load_i64_to_local_from_offset(
            function_object_local,
            HEAP_FUNCTION_ENV_HANDLE_OFFSET,
            env_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            function_object_local,
            HEAP_FUNCTION_TABLE_INDEX_OFFSET,
            table_index_local,
            function,
        );
    }

    fn emit_load_function_flags(
        &mut self,
        function_object_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        self.load_i64_to_local_from_offset(
            function_object_local,
            HEAP_FUNCTION_FLAGS_OFFSET,
            result_local,
            function,
        );
    }

    fn emit_load_function_constructable_flag(
        &mut self,
        function_object_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        self.emit_load_function_flags(function_object_local, result_local, function);
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I64Const(FUNCTION_FLAG_CONSTRUCTABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(result_local));
    }

    fn emit_function_handle_call(
        &mut self,
        callee_payload_local: u32,
        callee_tag_local: u32,
        this_locals: Option<(u32, Option<u32>)>,
        args: &[(u32, u32)],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();
        self.emit_pre_evaluated_arg_vector(args, argc_local, argv_local, function)?;
        self.emit_function_handle_call_with_argv(
            callee_payload_local,
            callee_tag_local,
            this_locals,
            argc_local,
            argv_local,
            payload_local,
            tag_local,
            function,
        )?;
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        Ok(())
    }

    fn emit_function_handle_call_with_argv(
        &mut self,
        callee_payload_local: u32,
        callee_tag_local: u32,
        this_locals: Option<(u32, Option<u32>)>,
        argc_local: u32,
        argv_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let callee_env_local = self.reserve_temp_local();
        let table_index_local = self.reserve_temp_local();
        let flags_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(callee_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);

        self.emit_load_function_object_fields(
            callee_payload_local,
            callee_env_local,
            table_index_local,
            function,
        );
        self.emit_load_function_flags(callee_payload_local, flags_local, function);

        function.instruction(&Instruction::LocalGet(flags_local));
        function.instruction(&Instruction::I64Const(
            FUNCTION_FLAG_CLASS_CONSTRUCTOR as i64,
        ));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(callee_env_local));
        if let Some((this_payload_local, this_tag_local)) = this_locals {
            function.instruction(&Instruction::LocalGet(this_payload_local));
            if let Some(this_tag_local) = this_tag_local {
                function.instruction(&Instruction::LocalGet(this_tag_local));
            } else {
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
            }
        } else {
            self.emit_default_this(function);
        }
        self.emit_undefined_new_target(function);
        function.instruction(&Instruction::LocalGet(argc_local));
        function.instruction(&Instruction::LocalGet(argv_local));
        function.instruction(&Instruction::LocalGet(table_index_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::CallIndirect {
            type_index: JS_FUNCTION_TYPE_INDEX,
            table_index: 0,
        });
        self.store_call_results(payload_local, tag_local, function);
        self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
            payload_local,
            tag_local,
            1,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "class constructor cannot be invoked without `new`",
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
            payload_local,
            tag_local,
            1,
            function,
        );
        function.instruction(&Instruction::End);

        self.release_temp_local(flags_local);
        self.release_temp_local(table_index_local);
        self.release_temp_local(callee_env_local);
        Ok(())
    }

    fn emit_super_construct_with_arg_vector(
        &mut self,
        argc_local: u32,
        argv_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let super_base_local = self.reserve_temp_local();
        let super_base_tag_local = self.reserve_temp_local();
        let ctor_key_local = self.reserve_temp_local();
        let ctor_payload_local = self.reserve_temp_local();
        let ctor_tag_local = self.reserve_temp_local();
        let call_payload_local = self.reserve_temp_local();
        let call_tag_local = self.reserve_temp_local();
        let call_completion_local = self.reserve_temp_local();
        let callee_env_local = self.reserve_temp_local();
        let table_index_local = self.reserve_temp_local();
        let new_target_payload_local = self.reserve_temp_local();
        let new_target_tag_local = self.reserve_temp_local();
        let proto_key_local = self.reserve_temp_local();
        let proto_payload_local = self.reserve_temp_local();
        let proto_tag_local = self.reserve_temp_local();
        let Some(this_payload_local) = self.this_payload_local else {
            return Err(EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: super outside derived constructor",
            ));
        };
        let Some(this_tag_local) = self.this_tag_local else {
            return Err(EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: super outside derived constructor",
            ));
        };
        let Some(derived_this_initialized_local) = self.derived_this_initialized_local else {
            return Err(EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: super outside derived constructor",
            ));
        };

        function.instruction(&Instruction::LocalGet(derived_this_initialized_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            "ReferenceError",
            "super() called twice in derived constructor",
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
            payload_local,
            tag_local,
            1,
            function,
        );
        function.instruction(&Instruction::End);

        if self
            .current_function_meta()
            .is_some_and(|meta| meta.class_heritage_kind == ClassHeritageKind::Null)
        {
            self.emit_throw_runtime_error(
                "TypeError",
                "super() invalid in class extending null",
                payload_local,
                tag_local,
                function,
            )?;
            self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
                payload_local,
                tag_local,
                0,
                function,
            );
            self.release_temp_local(proto_tag_local);
            self.release_temp_local(proto_payload_local);
            self.release_temp_local(proto_key_local);
            self.release_temp_local(new_target_tag_local);
            self.release_temp_local(new_target_payload_local);
            self.release_temp_local(table_index_local);
            self.release_temp_local(callee_env_local);
            self.release_temp_local(call_completion_local);
            self.release_temp_local(call_tag_local);
            self.release_temp_local(call_payload_local);
            self.release_temp_local(ctor_tag_local);
            self.release_temp_local(ctor_payload_local);
            self.release_temp_local(ctor_key_local);
            self.release_temp_local(super_base_tag_local);
            self.release_temp_local(super_base_local);
            return Ok(());
        }

        let super_constructor_target = self
            .current_function_meta()
            .and_then(|meta| meta.super_constructor_target.clone());
        if let Some(super_constructor_target) = super_constructor_target {
            let super_constructor_meta =
                self.functions
                    .get(&super_constructor_target)
                    .ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "unsupported in porffor wasm-aot first slice: unknown super constructor `{super_constructor_target}`"
                        ))
                    })?;
            self.emit_function_value_payload(super_constructor_meta, function)?;
            function.instruction(&Instruction::LocalSet(ctor_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::LocalSet(ctor_tag_local));
        } else {
            self.emit_load_super_base(super_base_local, super_base_tag_local, function)?;
            function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
            function.instruction(&Instruction::LocalSet(ctor_key_local));
            self.emit_object_read(
                super_base_local,
                super_base_tag_local,
                super_base_local,
                super_base_tag_local,
                ctor_key_local,
                ctor_payload_local,
                ctor_tag_local,
                function,
            )?;
        }
        self.compile_new_target_to_locals(new_target_payload_local, new_target_tag_local, function);
        function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
        function.instruction(&Instruction::LocalSet(proto_key_local));
        self.emit_object_read(
            new_target_payload_local,
            new_target_tag_local,
            new_target_payload_local,
            new_target_tag_local,
            proto_key_local,
            proto_payload_local,
            proto_tag_local,
            function,
        )?;
        self.emit_is_heap_object_like_tag_i32(proto_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_plain_object_with_prototype(Some(proto_payload_local), None, function)?;
        function.instruction(&Instruction::LocalSet(this_payload_local));
        function.instruction(&Instruction::Else);
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(this_payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(this_tag_local));
        self.emit_load_function_object_fields(
            ctor_payload_local,
            callee_env_local,
            table_index_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(callee_env_local));
        function.instruction(&Instruction::LocalGet(this_payload_local));
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::LocalGet(new_target_payload_local));
        function.instruction(&Instruction::LocalGet(new_target_tag_local));
        function.instruction(&Instruction::LocalGet(argc_local));
        function.instruction(&Instruction::LocalGet(argv_local));
        function.instruction(&Instruction::LocalGet(table_index_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::CallIndirect {
            type_index: JS_FUNCTION_TYPE_INDEX,
            table_index: 0,
        });
        self.store_call_results_to(
            call_payload_local,
            call_tag_local,
            call_completion_local,
            self.completion_aux_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(call_completion_local));
        function.instruction(&Instruction::LocalSet(self.completion_local));
        self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
            call_payload_local,
            call_tag_local,
            0,
            function,
        );
        self.emit_is_heap_object_like_tag_i32(call_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(call_payload_local));
        function.instruction(&Instruction::LocalSet(this_payload_local));
        function.instruction(&Instruction::LocalGet(call_tag_local));
        function.instruction(&Instruction::LocalSet(this_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(derived_this_initialized_local));
        function.instruction(&Instruction::LocalGet(this_payload_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::LocalSet(tag_local));

        self.release_temp_local(proto_tag_local);
        self.release_temp_local(proto_payload_local);
        self.release_temp_local(proto_key_local);
        self.release_temp_local(new_target_tag_local);
        self.release_temp_local(new_target_payload_local);
        self.release_temp_local(table_index_local);
        self.release_temp_local(callee_env_local);
        self.release_temp_local(call_completion_local);
        self.release_temp_local(call_tag_local);
        self.release_temp_local(call_payload_local);
        self.release_temp_local(ctor_tag_local);
        self.release_temp_local(ctor_payload_local);
        self.release_temp_local(ctor_key_local);
        self.release_temp_local(super_base_tag_local);
        self.release_temp_local(super_base_local);
        Ok(())
    }

    fn env_slot_offset(slot: u32, field_offset: u64) -> u64 {
        ENV_SLOT_BASE_OFFSET + slot as u64 * ENV_SLOT_SIZE + field_offset
    }

    fn resolve_env_handle_local(&mut self, hops: u32, function: &mut Function) -> u32 {
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

    fn initialize_binding_undefined(&mut self, storage: BindingStorage, function: &mut Function) {
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
            BindingStorage::Fixed {
                payload_local: binding_payload_local,
                ..
            } => {
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

    fn emit_default_this(&self, function: &mut Function) {
        function.instruction(&Instruction::GlobalGet(SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
    }

    fn emit_undefined_new_target(&self, function: &mut Function) {
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
    }

    fn store_call_results(&self, payload_local: u32, tag_local: u32, function: &mut Function) {
        function.instruction(&Instruction::LocalSet(self.completion_aux_local));
        function.instruction(&Instruction::LocalSet(self.completion_local));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::LocalSet(payload_local));
    }

    fn store_call_results_to(
        &self,
        payload_local: u32,
        tag_local: u32,
        completion_local: u32,
        aux_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalSet(aux_local));
        function.instruction(&Instruction::LocalSet(completion_local));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::LocalSet(payload_local));
    }

    fn compile_new_target_to_locals(
        &mut self,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        if self.function_flavor == FunctionFlavor::Arrow {
            if let Some(storage) = self.lookup_binding(LEXICAL_NEW_TARGET_NAME) {
                self.read_binding_to_locals(storage, payload_local, tag_local, function);
                return;
            }
        }
        if let (Some(new_target_payload_local), Some(new_target_tag_local)) =
            (self.new_target_payload_local(), self.new_target_tag_local())
        {
            function.instruction(&Instruction::LocalGet(new_target_payload_local));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::LocalGet(new_target_tag_local));
            function.instruction(&Instruction::LocalSet(tag_local));
        } else {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
        }
    }

    fn emit_runtime_error_object(
        &mut self,
        name: &str,
        message: &str,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();

        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(error_prototype_global_index(name)),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(object_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("message")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload(message)));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_object_define_data(
            object_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));

        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(object_local);
        Ok(())
    }

    fn emit_throw_runtime_error(
        &mut self,
        name: &str,
        message: &str,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_runtime_error_object(name, message, payload_local, tag_local, function)?;
        self.emit_throw_from_locals(payload_local, tag_local, function);
        Ok(())
    }

    fn current_function_meta(&self) -> Option<&WasmFunctionMeta> {
        self.function_id
            .as_ref()
            .and_then(|function_id| self.functions.get(function_id))
    }

    fn emit_load_super_base(
        &mut self,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let Some(this_payload_local) = self.this_payload_local else {
            return Err(EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: super outside class method",
            ));
        };
        let Some(_this_tag_local) = self.this_tag_local else {
            return Err(EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: super outside class method",
            ));
        };
        let static_member = self
            .current_function_meta()
            .map(|meta| meta.is_static_class_member)
            .unwrap_or(false);
        self.load_i64_to_local_from_offset(
            this_payload_local,
            HEAP_PROTOTYPE_OFFSET,
            payload_local,
            function,
        );
        if !static_member {
            self.load_i64_to_local_from_offset(
                payload_local,
                HEAP_PROTOTYPE_OFFSET,
                payload_local,
                function,
            );
        }
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(if static_member {
            ValueKind::Function.tag()
        } else {
            ValueKind::Object.tag()
        } as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn emit_throw_if_null_super_base(
        &mut self,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            "TypeError",
            "super property access on null base",
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
            payload_local,
            tag_local,
            0,
            function,
        );
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn emit_global_property_read(
        &mut self,
        name: &str,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let object_local = self.reserve_temp_local();
        let object_tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(self.strings.payload(name)));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::GlobalGet(SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(object_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(object_tag_local));
        self.emit_object_read(
            object_local,
            object_tag_local,
            object_local,
            object_tag_local,
            key_local,
            payload_local,
            tag_local,
            function,
        )?;
        self.release_temp_local(object_tag_local);
        self.release_temp_local(object_local);
        self.release_temp_local(key_local);
        Ok(())
    }

    fn emit_global_property_write(
        &mut self,
        name: &str,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let object_local = self.reserve_temp_local();
        let object_tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(self.strings.payload(name)));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::GlobalGet(SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(object_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(object_tag_local));
        self.emit_object_write(
            object_local,
            object_tag_local,
            key_local,
            payload_local,
            tag_local,
            function,
        )?;
        self.release_temp_local(object_tag_local);
        self.release_temp_local(object_local);
        self.release_temp_local(key_local);
        Ok(())
    }

    fn emit_global_property_delete(
        &mut self,
        name: &str,
        result_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let object_local = self.reserve_temp_local();
        let object_tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(self.strings.payload(name)));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::GlobalGet(SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(object_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(object_tag_local));
        self.emit_object_delete(
            object_local,
            object_tag_local,
            key_local,
            result_local,
            function,
        )?;
        self.release_temp_local(object_tag_local);
        self.release_temp_local(object_local);
        self.release_temp_local(key_local);
        Ok(())
    }

    fn mirror_binding_to_global_object(
        &mut self,
        name: &str,
        storage: BindingStorage,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if !self.is_script_global_binding(name) {
            return Ok(());
        }

        let key_local = self.reserve_temp_local();
        let object_local = self.reserve_temp_local();
        let object_tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(self.strings.payload(name)));
        function.instruction(&Instruction::LocalSet(key_local));
        self.read_binding_to_locals(storage, self.scratch_local, self.result_tag_local, function);
        function.instruction(&Instruction::GlobalGet(SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(object_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(object_tag_local));
        self.emit_object_write(
            object_local,
            object_tag_local,
            key_local,
            self.scratch_local,
            self.result_tag_local,
            function,
        )?;
        self.release_temp_local(object_tag_local);
        self.release_temp_local(object_local);
        self.release_temp_local(key_local);
        Ok(())
    }

    const fn memarg64(offset: u64) -> MemArg {
        MemArg {
            offset,
            align: 3,
            memory_index: 0,
        }
    }

    const fn memarg8(offset: u64) -> MemArg {
        MemArg {
            offset,
            align: 0,
            memory_index: 0,
        }
    }

    fn emit_call(
        &mut self,
        name: &str,
        args: &[TypedExpr],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let meta = self.functions.values().find(|meta| meta.name == name).ok_or_else(|| {
            EmitError::unsupported(format!(
                "unsupported in porffor wasm-aot first slice: direct call to unknown top-level function `{name}`"
            ))
        })?;
        let wasm_index = meta.wasm_index;
        let is_class_constructor = meta.class_kind == ClassFunctionKind::Constructor;
        let (argc_local, argv_local) = self.emit_call_args_vector(args, function)?;
        let callee_payload_local = self.reserve_temp_local();
        let callee_tag_local = self.reserve_temp_local();
        let callee_env_local = self.reserve_temp_local();
        let callee_table_index_local = self.reserve_temp_local();

        if is_class_constructor {
            self.emit_throw_runtime_error(
                "TypeError",
                "class constructor cannot be invoked without `new`",
                payload_local,
                tag_local,
                function,
            )?;
            if let Some(target) = self.throw_handler_stack.last() {
                function.instruction(&Instruction::Br(self.depth_to(*target)));
            } else {
                self.emit_return_current_completion(function);
            }
        } else {
            if let Some(storage) = self.lookup_binding(name) {
                self.read_binding_to_locals(
                    storage,
                    callee_payload_local,
                    callee_tag_local,
                    function,
                );
                self.emit_load_function_object_fields(
                    callee_payload_local,
                    callee_env_local,
                    callee_table_index_local,
                    function,
                );
                function.instruction(&Instruction::LocalGet(callee_env_local));
            } else {
                let key_local = self.reserve_temp_local();
                let global_object_local = self.reserve_temp_local();
                let global_object_tag_local = self.reserve_temp_local();
                function.instruction(&Instruction::I64Const(self.strings.payload(name)));
                function.instruction(&Instruction::LocalSet(key_local));
                function.instruction(&Instruction::GlobalGet(SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalSet(global_object_local));
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::LocalSet(global_object_tag_local));
                self.emit_object_read(
                    global_object_local,
                    global_object_tag_local,
                    global_object_local,
                    global_object_tag_local,
                    key_local,
                    callee_payload_local,
                    callee_tag_local,
                    function,
                )?;
                self.emit_load_function_object_fields(
                    callee_payload_local,
                    callee_env_local,
                    callee_table_index_local,
                    function,
                );
                function.instruction(&Instruction::LocalGet(callee_env_local));
                self.release_temp_local(global_object_tag_local);
                self.release_temp_local(global_object_local);
                self.release_temp_local(key_local);
            }
            self.emit_default_this(function);
            self.emit_undefined_new_target(function);
            function.instruction(&Instruction::LocalGet(argc_local));
            function.instruction(&Instruction::LocalGet(argv_local));
            function.instruction(&Instruction::Call(wasm_index));
            self.store_call_results(payload_local, tag_local, function);
            self.emit_propagate_throw_from_locals_if_needed(payload_local, tag_local, function);
        }
        self.release_temp_local(callee_table_index_local);
        self.release_temp_local(callee_env_local);
        self.release_temp_local(callee_tag_local);
        self.release_temp_local(callee_payload_local);
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        Ok(())
    }

    fn emit_direct_js_call(
        &mut self,
        meta: &WasmFunctionMeta,
        this_locals: Option<(u32, Option<u32>)>,
        args: &[(u32, u32)],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();

        if meta.class_kind == ClassFunctionKind::Constructor {
            self.emit_throw_runtime_error(
                "TypeError",
                "class constructor cannot be invoked without `new`",
                payload_local,
                tag_local,
                function,
            )?;
            if let Some(target) = self.throw_handler_stack.last() {
                function.instruction(&Instruction::Br(self.depth_to(*target)));
            } else {
                self.emit_return_current_completion(function);
            }
        } else {
            function.instruction(&Instruction::LocalGet(self.current_env_local));
            if let Some((this_payload_local, this_tag_local)) = this_locals {
                function.instruction(&Instruction::LocalGet(this_payload_local));
                if let Some(this_tag_local) = this_tag_local {
                    function.instruction(&Instruction::LocalGet(this_tag_local));
                } else {
                    function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                }
            } else {
                self.emit_default_this(function);
            }
            self.emit_pre_evaluated_arg_vector(args, argc_local, argv_local, function)?;
            self.emit_undefined_new_target(function);
            function.instruction(&Instruction::LocalGet(argc_local));
            function.instruction(&Instruction::LocalGet(argv_local));
            function.instruction(&Instruction::Call(meta.wasm_index));
            self.store_call_results(payload_local, tag_local, function);
            self.emit_propagate_throw_from_locals_if_needed(payload_local, tag_local, function);
        }

        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        Ok(())
    }

    fn emit_indirect_call(
        &mut self,
        callee: &TypedExpr,
        this_arg: Option<&TypedExpr>,
        args: &[TypedExpr],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let callee_payload_local = self.reserve_temp_local();
        let callee_tag_local = self.reserve_temp_local();
        let callee_env_local = self.reserve_temp_local();
        let table_index_local = self.reserve_temp_local();
        let flags_local = self.reserve_temp_local();
        self.compile_expr_to_locals(callee, callee_payload_local, callee_tag_local, function)?;

        function.instruction(&Instruction::LocalGet(callee_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);

        let this_locals = if let Some(this_arg) = this_arg {
            let this_payload_local = self.reserve_temp_local();
            let this_tag_local = self.reserve_temp_local();
            self.compile_expr_to_locals(this_arg, this_payload_local, this_tag_local, function)?;
            Some((this_payload_local, this_tag_local))
        } else {
            None
        };
        let (argc_local, argv_local) = self.emit_call_args_vector(args, function)?;

        self.emit_load_function_object_fields(
            callee_payload_local,
            callee_env_local,
            table_index_local,
            function,
        );
        self.emit_load_function_flags(callee_payload_local, flags_local, function);
        function.instruction(&Instruction::LocalGet(flags_local));
        function.instruction(&Instruction::I64Const(
            FUNCTION_FLAG_CLASS_CONSTRUCTOR as i64,
        ));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(callee_env_local));
        if let Some((this_payload_local, this_tag_local)) = this_locals {
            function.instruction(&Instruction::LocalGet(this_payload_local));
            function.instruction(&Instruction::LocalGet(this_tag_local));
        } else {
            self.emit_default_this(function);
        }
        self.emit_undefined_new_target(function);
        function.instruction(&Instruction::LocalGet(argc_local));
        function.instruction(&Instruction::LocalGet(argv_local));
        function.instruction(&Instruction::LocalGet(table_index_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::CallIndirect {
            type_index: JS_FUNCTION_TYPE_INDEX,
            table_index: 0,
        });
        self.store_call_results(payload_local, tag_local, function);
        self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
            payload_local,
            tag_local,
            1,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            "TypeError",
            "class constructor cannot be invoked without `new`",
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
            payload_local,
            tag_local,
            1,
            function,
        );
        function.instruction(&Instruction::End);

        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        if let Some((this_payload_local, this_tag_local)) = this_locals {
            self.release_temp_local(this_tag_local);
            self.release_temp_local(this_payload_local);
        }
        self.release_temp_local(flags_local);
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
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let callee_payload_local = self.reserve_temp_local();
        let callee_tag_local = self.reserve_temp_local();
        let callee_env_local = self.reserve_temp_local();
        let table_index_local = self.reserve_temp_local();
        let flags_local = self.reserve_temp_local();

        self.compile_expr_to_locals(
            receiver,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        match receiver.kind {
            ValueKind::Object | ValueKind::Function => {
                let key_local = self.compile_object_key_to_local(key, function)?;
                self.emit_object_read(
                    receiver_payload_local,
                    receiver_tag_local,
                    receiver_payload_local,
                    receiver_tag_local,
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
            ValueKind::Arguments => {
                let index_local = self.compile_array_index_to_local(key, function)?;
                self.emit_arguments_read(
                    receiver_payload_local,
                    index_local,
                    callee_payload_local,
                    callee_tag_local,
                    function,
                )?;
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

        let (argc_local, argv_local) = self.emit_call_args_vector(args, function)?;

        self.emit_function_handle_call_with_argv(
            callee_payload_local,
            callee_tag_local,
            Some((receiver_payload_local, Some(receiver_tag_local))),
            argc_local,
            argv_local,
            payload_local,
            tag_local,
            function,
        )?;

        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(flags_local);
        self.release_temp_local(table_index_local);
        self.release_temp_local(callee_env_local);
        self.release_temp_local(callee_tag_local);
        self.release_temp_local(callee_payload_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    fn emit_construct(
        &mut self,
        callee: &TypedExpr,
        args: &[TypedExpr],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let callee_payload_local = self.reserve_temp_local();
        let callee_tag_local = self.reserve_temp_local();

        self.compile_expr_to_locals(callee, callee_payload_local, callee_tag_local, function)?;
        let (argc_local, argv_local) = self.emit_call_args_vector(args, function)?;
        self.emit_function_handle_construct_with_argv(
            callee_payload_local,
            callee_tag_local,
            callee_payload_local,
            callee_tag_local,
            argc_local,
            argv_local,
            payload_local,
            tag_local,
            function,
        )?;

        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(callee_tag_local);
        self.release_temp_local(callee_payload_local);
        Ok(())
    }

    fn emit_instanceof_i32(
        &mut self,
        lhs: &TypedExpr,
        rhs: &TypedExpr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let lhs_payload_local = self.reserve_temp_local();
        let lhs_tag_local = self.reserve_temp_local();
        let rhs_payload_local = self.reserve_temp_local();
        let rhs_tag_local = self.reserve_temp_local();
        let proto_key_local = self.reserve_temp_local();
        let rhs_proto_payload_local = self.reserve_temp_local();
        let rhs_proto_tag_local = self.reserve_temp_local();
        let search_local = self.reserve_temp_local();
        let next_proto_local = self.reserve_temp_local();
        let found_local = self.reserve_temp_local();

        self.compile_expr_to_locals(lhs, lhs_payload_local, lhs_tag_local, function)?;
        self.compile_expr_to_locals(rhs, rhs_payload_local, rhs_tag_local, function)?;
        function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
        function.instruction(&Instruction::LocalSet(proto_key_local));
        self.emit_object_read(
            rhs_payload_local,
            rhs_tag_local,
            rhs_payload_local,
            rhs_tag_local,
            proto_key_local,
            rhs_proto_payload_local,
            rhs_proto_tag_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(lhs_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        for kind in [ValueKind::Array, ValueKind::Function, ValueKind::Arguments] {
            function.instruction(&Instruction::LocalGet(lhs_tag_local));
            function.instruction(&Instruction::I64Const(kind.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
        }
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::LocalGet(lhs_payload_local));
        function.instruction(&Instruction::LocalSet(search_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(search_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        self.load_i64_from_offset(search_local, HEAP_PROTOTYPE_OFFSET, function);
        function.instruction(&Instruction::LocalTee(next_proto_local));
        function.instruction(&Instruction::LocalGet(rhs_proto_payload_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(next_proto_local));
        function.instruction(&Instruction::LocalSet(search_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(found_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::End);

        self.release_temp_local(found_local);
        self.release_temp_local(next_proto_local);
        self.release_temp_local(search_local);
        self.release_temp_local(rhs_proto_tag_local);
        self.release_temp_local(rhs_proto_payload_local);
        self.release_temp_local(proto_key_local);
        self.release_temp_local(rhs_tag_local);
        self.release_temp_local(rhs_payload_local);
        self.release_temp_local(lhs_tag_local);
        self.release_temp_local(lhs_payload_local);
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
        if !expr.possible_kinds.is_singleton() {
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

    fn compile_nullish_tagged_i32(
        &self,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        Ok(())
    }

    fn compile_expr_to_primitive_locals(
        &mut self,
        expr: &TypedExpr,
        hint: ToPrimitiveHint,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if expr.possible_kinds.is_subset_of(KindSet::PRIMITIVE_ONLY) {
            self.compile_expr_to_locals(expr, payload_local, tag_local, function)?;
            return Ok(());
        }

        let raw_payload_local = self.reserve_temp_local();
        let raw_tag_local = self.reserve_temp_local();
        self.compile_expr_to_locals(expr, raw_payload_local, raw_tag_local, function)?;
        self.emit_tagged_to_primitive_locals(
            hint,
            raw_payload_local,
            raw_tag_local,
            payload_local,
            tag_local,
            function,
        )?;
        self.release_temp_local(raw_tag_local);
        self.release_temp_local(raw_payload_local);
        Ok(())
    }

    fn emit_tagged_to_primitive_locals(
        &mut self,
        hint: ToPrimitiveHint,
        input_payload_local: u32,
        input_tag_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_to_primitive_locals(
            hint,
            input_payload_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_to_string_locals(input_payload_local, payload_local, tag_local, function)?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("[object Arguments]"),
        ));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(input_payload_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn emit_object_to_primitive_locals(
        &mut self,
        hint: ToPrimitiveHint,
        object_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let hook_names: &[&str] = match hint {
            ToPrimitiveHint::String => &["toString", "valueOf"],
            ToPrimitiveHint::Default | ToPrimitiveHint::Number => &["valueOf", "toString"],
        };

        let boxed_kind_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            object_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            boxed_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_NONE as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            object_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            object_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            tag_local,
            function,
        );
        function.instruction(&Instruction::Else);
        let hook_value_payload = self.reserve_temp_local();
        let hook_value_tag = self.reserve_temp_local();
        let call_result_payload = self.reserve_temp_local();
        let call_result_tag = self.reserve_temp_local();
        let primitive_result_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(primitive_result_local));

        for hook_name in hook_names {
            let key_local = self.reserve_temp_local();
            function.instruction(&Instruction::I64Const(self.strings.payload(hook_name)));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_object_read(
                object_local,
                self.result_tag_local,
                object_local,
                self.result_tag_local,
                key_local,
                hook_value_payload,
                hook_value_tag,
                function,
            )?;
            function.instruction(&Instruction::LocalGet(hook_value_tag));
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_function_handle_call(
                hook_value_payload,
                hook_value_tag,
                Some((object_local, None)),
                &[],
                call_result_payload,
                call_result_tag,
                function,
            )?;
            self.emit_is_primitive_tag_i32(call_result_tag, function);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(call_result_payload));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::LocalGet(call_result_tag));
            function.instruction(&Instruction::LocalSet(tag_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(primitive_result_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            self.release_temp_local(key_local);
        }

        function.instruction(&Instruction::LocalGet(primitive_result_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("[object Object]"),
        ));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(primitive_result_local);
        self.release_temp_local(call_result_tag);
        self.release_temp_local(call_result_payload);
        self.release_temp_local(hook_value_tag);
        self.release_temp_local(hook_value_payload);
        function.instruction(&Instruction::End);
        self.release_temp_local(boxed_kind_local);
        Ok(())
    }

    fn emit_array_to_string_locals(
        &mut self,
        array_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let element_string_local = self.reserve_temp_local();
        let result_string_local = self.reserve_temp_local();
        let comma_string_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(array_local, HEAP_LEN_OFFSET, len_local, function);
        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(result_string_local));
        function.instruction(&Instruction::I64Const(self.strings.payload(",")));
        function.instruction(&Instruction::LocalSet(comma_string_local));
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
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_PAYLOAD_OFFSET,
            element_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_TAG_OFFSET,
            element_tag_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_concat_string_payloads_local(result_string_local, comma_string_local, function)?;
        function.instruction(&Instruction::LocalSet(result_string_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(element_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(element_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(element_string_local));
        function.instruction(&Instruction::Else);
        self.emit_array_element_to_string_payload(
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(element_string_local));
        function.instruction(&Instruction::End);

        self.emit_concat_string_payloads_local(
            result_string_local,
            element_string_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(result_string_local));

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(result_string_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));

        self.release_temp_local(comma_string_local);
        self.release_temp_local(result_string_local);
        self.release_temp_local(element_string_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    fn emit_array_element_to_string_payload(
        &mut self,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(self.strings.payload("false")));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("true")));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        self.emit_number_to_string_payload(payload_local, function)?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        let primitive_payload_local = self.reserve_temp_local();
        let primitive_tag_local = self.reserve_temp_local();
        self.emit_object_to_primitive_locals(
            ToPrimitiveHint::String,
            payload_local,
            primitive_payload_local,
            primitive_tag_local,
            function,
        )?;
        self.emit_primitive_to_string_payload(
            primitive_payload_local,
            primitive_tag_local,
            function,
        )?;
        self.release_temp_local(primitive_tag_local);
        self.release_temp_local(primitive_payload_local);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("[object Object]"),
        ));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        self.emit_function_to_string_payload(payload_local, function)?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("[object Arguments]"),
        ));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn emit_concat_string_payloads_local(
        &mut self,
        lhs_string_local: u32,
        rhs_string_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let lhs_offset = self.reserve_temp_local();
        let lhs_len = self.reserve_temp_local();
        let rhs_offset = self.reserve_temp_local();
        let rhs_len = self.reserve_temp_local();
        let total_len = self.reserve_temp_local();
        let dst_offset = self.reserve_temp_local();
        let rhs_dst_offset = self.reserve_temp_local();

        self.emit_unpack_string_payload(lhs_string_local, lhs_offset, lhs_len, function);
        self.emit_unpack_string_payload(rhs_string_local, rhs_offset, rhs_len, function);
        function.instruction(&Instruction::LocalGet(lhs_len));
        function.instruction(&Instruction::LocalGet(rhs_len));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(total_len));
        self.emit_heap_alloc_from_local(total_len, function)?;
        function.instruction(&Instruction::LocalSet(dst_offset));
        self.emit_copy_bytes(lhs_offset, dst_offset, lhs_len, function);
        function.instruction(&Instruction::LocalGet(dst_offset));
        function.instruction(&Instruction::LocalGet(lhs_len));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(rhs_dst_offset));
        self.emit_copy_bytes(rhs_offset, rhs_dst_offset, rhs_len, function);
        self.emit_pack_string_payload(dst_offset, total_len, function);

        self.release_temp_local(rhs_dst_offset);
        self.release_temp_local(dst_offset);
        self.release_temp_local(total_len);
        self.release_temp_local(rhs_len);
        self.release_temp_local(rhs_offset);
        self.release_temp_local(lhs_len);
        self.release_temp_local(lhs_offset);
        Ok(())
    }

    fn emit_is_primitive_tag_i32(&self, tag_local: u32, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        for kind in [
            ValueKind::Null,
            ValueKind::Boolean,
            ValueKind::Number,
            ValueKind::String,
        ] {
            function.instruction(&Instruction::LocalGet(tag_local));
            function.instruction(&Instruction::I64Const(kind.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
        }
    }

    fn emit_is_heap_object_like_tag_i32(&self, tag_local: u32, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        for kind in [ValueKind::Array, ValueKind::Function, ValueKind::Arguments] {
            function.instruction(&Instruction::LocalGet(tag_local));
            function.instruction(&Instruction::I64Const(kind.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
        }
    }

    fn compile_coercive_add_to_locals(
        &mut self,
        lhs: &TypedExpr,
        rhs: &TypedExpr,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let lhs_payload = self.reserve_temp_local();
        let lhs_tag = self.reserve_temp_local();
        let rhs_payload = self.reserve_temp_local();
        let rhs_tag = self.reserve_temp_local();
        let lhs_string_local = self.reserve_temp_local();
        let rhs_string_local = self.reserve_temp_local();

        self.compile_expr_to_primitive_locals(
            lhs,
            ToPrimitiveHint::Default,
            lhs_payload,
            lhs_tag,
            function,
        )?;
        self.compile_expr_to_primitive_locals(
            rhs,
            ToPrimitiveHint::Default,
            rhs_payload,
            rhs_tag,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(lhs_tag));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(rhs_tag));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_string_payload(lhs_payload, lhs_tag, function)?;
        function.instruction(&Instruction::LocalSet(lhs_string_local));
        self.emit_value_to_string_payload(rhs_payload, rhs_tag, function)?;
        function.instruction(&Instruction::LocalSet(rhs_string_local));
        self.emit_concat_string_payloads_local(lhs_string_local, rhs_string_local, function)?;
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        self.emit_value_to_number_payload(lhs_tag, lhs_payload, function)?;
        function.instruction(&Instruction::F64ReinterpretI64);
        self.emit_value_to_number_payload(rhs_tag, rhs_payload, function)?;
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(rhs_string_local);
        self.release_temp_local(lhs_string_local);
        self.release_temp_local(rhs_tag);
        self.release_temp_local(rhs_payload);
        self.release_temp_local(lhs_tag);
        self.release_temp_local(lhs_payload);
        Ok(())
    }

    fn compile_expr_to_number_payload(
        &mut self,
        expr: &TypedExpr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if expr.kind == ValueKind::Number {
            self.compile_expr_payload(expr, function)?;
            return Ok(());
        }

        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        self.compile_expr_to_primitive_locals(
            expr,
            ToPrimitiveHint::Number,
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_value_to_number_payload(tag_local, payload_local, function)?;
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        Ok(())
    }

    fn compile_expr_to_number_payload_nonstring(
        &mut self,
        expr: &TypedExpr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if expr.kind == ValueKind::Number {
            self.compile_expr_payload(expr, function)?;
            return Ok(());
        }

        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        self.compile_expr_to_primitive_locals(
            expr,
            ToPrimitiveHint::Number,
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_nonstring_value_to_number_payload(tag_local, payload_local, function)?;
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        Ok(())
    }

    fn emit_nan_payload(&self, function: &mut Function) {
        function.instruction(&Instruction::I64Const(f64::NAN.to_bits() as i64));
    }

    fn emit_value_to_number_payload(
        &mut self,
        tag_local: u32,
        payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        let primitive_payload_local = self.reserve_temp_local();
        let primitive_tag_local = self.reserve_temp_local();
        self.emit_object_to_primitive_locals(
            ToPrimitiveHint::Number,
            payload_local,
            primitive_payload_local,
            primitive_tag_local,
            function,
        )?;
        self.emit_primitive_to_number_payload(
            primitive_tag_local,
            primitive_payload_local,
            function,
        )?;
        self.release_temp_local(primitive_tag_local);
        self.release_temp_local(primitive_payload_local);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        let primitive_payload_local = self.reserve_temp_local();
        let primitive_tag_local = self.reserve_temp_local();
        self.emit_array_to_string_locals(
            payload_local,
            primitive_payload_local,
            primitive_tag_local,
            function,
        )?;
        self.emit_primitive_to_number_payload(
            primitive_tag_local,
            primitive_payload_local,
            function,
        )?;
        self.release_temp_local(primitive_tag_local);
        self.release_temp_local(primitive_payload_local);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        let primitive_payload_local = self.reserve_temp_local();
        let primitive_tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(
            self.strings.payload("[object Arguments]"),
        ));
        function.instruction(&Instruction::LocalSet(primitive_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(primitive_tag_local));
        self.emit_primitive_to_number_payload(
            primitive_tag_local,
            primitive_payload_local,
            function,
        )?;
        self.release_temp_local(primitive_tag_local);
        self.release_temp_local(primitive_payload_local);
        function.instruction(&Instruction::Else);
        self.emit_primitive_to_number_payload(tag_local, payload_local, function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn emit_primitive_to_number_payload(
        &mut self,
        tag_local: u32,
        payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        self.emit_nan_payload(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        self.emit_string_to_number_payload(payload_local, function)?;
        function.instruction(&Instruction::Else);
        self.emit_nan_payload(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn emit_nonstring_value_to_number_payload(
        &self,
        tag_local: u32,
        payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        self.emit_nan_payload(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn emit_string_to_number_payload(
        &mut self,
        string_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let offset_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let start_local = self.reserve_temp_local();
        let end_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let digit_local = self.reserve_temp_local();
        let output_local = self.reserve_temp_local();
        let result_local = self.reserve_temp_local();
        let frac_scale_local = self.reserve_temp_local();
        let saw_digit_local = self.reserve_temp_local();
        let dot_seen_local = self.reserve_temp_local();
        let negative_local = self.reserve_temp_local();
        let invalid_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(string_payload_local, offset_local, len_local, function);
        function.instruction(&Instruction::LocalGet(offset_local));
        function.instruction(&Instruction::LocalSet(start_local));
        function.instruction(&Instruction::LocalGet(offset_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(end_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(byte_local));
        self.emit_is_ascii_whitespace_i32(byte_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(start_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(byte_local));
        self.emit_is_ascii_whitespace_i32(byte_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalSet(end_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(negative_local));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(byte_local));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'+' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(negative_local));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(start_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(frac_scale_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(saw_digit_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(dot_seen_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::LocalSet(index_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(byte_local));

        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'9' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(saw_digit_local));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(digit_local));
        function.instruction(&Instruction::LocalGet(dot_seen_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(10.0)));
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(frac_scale_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(10.0)));
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(frac_scale_local));
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::LocalGet(frac_scale_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Div);
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'.' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(dot_seen_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(dot_seen_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(saw_digit_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(negative_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Neg);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Else);
        self.emit_nan_payload(function);
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(output_local));

        self.release_temp_local(invalid_local);
        self.release_temp_local(negative_local);
        self.release_temp_local(dot_seen_local);
        self.release_temp_local(saw_digit_local);
        self.release_temp_local(frac_scale_local);
        self.release_temp_local(result_local);
        self.release_temp_local(output_local);
        self.release_temp_local(digit_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(index_local);
        self.release_temp_local(end_local);
        self.release_temp_local(start_local);
        self.release_temp_local(len_local);
        self.release_temp_local(offset_local);
        Ok(())
    }

    fn emit_is_ascii_whitespace_i32(&self, byte_local: u32, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b' ' as i64));
        function.instruction(&Instruction::I64Eq);
        for byte in [b'\t', b'\n', 0x0B, 0x0C, b'\r'] {
            function.instruction(&Instruction::LocalGet(byte_local));
            function.instruction(&Instruction::I64Const(byte as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
        }
    }

    fn compile_loose_equality_i32(
        &mut self,
        lhs: &TypedExpr,
        rhs: &TypedExpr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if lhs.possible_kinds.is_subset_of(KindSet::PRIMITIVE_ONLY)
            && rhs.possible_kinds.is_subset_of(KindSet::PRIMITIVE_ONLY)
        {
            let lhs_payload = self.reserve_temp_local();
            let lhs_tag = self.reserve_temp_local();
            let rhs_payload = self.reserve_temp_local();
            let rhs_tag = self.reserve_temp_local();
            self.compile_expr_to_locals(lhs, lhs_payload, lhs_tag, function)?;
            self.compile_expr_to_locals(rhs, rhs_payload, rhs_tag, function)?;
            self.emit_loose_tagged_equality_i32(
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
            return Ok(());
        }

        let lhs_raw_payload = self.reserve_temp_local();
        let lhs_raw_tag = self.reserve_temp_local();
        let rhs_raw_payload = self.reserve_temp_local();
        let rhs_raw_tag = self.reserve_temp_local();
        let lhs_payload = self.reserve_temp_local();
        let lhs_tag = self.reserve_temp_local();
        let rhs_payload = self.reserve_temp_local();
        let rhs_tag = self.reserve_temp_local();
        let done_local = self.reserve_temp_local();

        self.compile_expr_to_locals(lhs, lhs_raw_payload, lhs_raw_tag, function)?;
        self.compile_expr_to_locals(rhs, rhs_raw_payload, rhs_raw_tag, function)?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(done_local));
        function.instruction(&Instruction::LocalGet(lhs_raw_tag));
        function.instruction(&Instruction::LocalGet(rhs_raw_tag));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(lhs_raw_tag));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(lhs_raw_tag));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(lhs_raw_tag));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(lhs_raw_payload));
        function.instruction(&Instruction::LocalGet(rhs_raw_payload));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(done_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(done_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.compile_expr_to_primitive_locals(
            lhs,
            ToPrimitiveHint::Default,
            lhs_payload,
            lhs_tag,
            function,
        )?;
        self.compile_expr_to_primitive_locals(
            rhs,
            ToPrimitiveHint::Default,
            rhs_payload,
            rhs_tag,
            function,
        )?;
        self.emit_loose_tagged_equality_i32(lhs_tag, lhs_payload, rhs_tag, rhs_payload, function)?;
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(done_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(done_local));
        function.instruction(&Instruction::I32WrapI64);

        self.release_temp_local(done_local);
        self.release_temp_local(rhs_tag);
        self.release_temp_local(rhs_payload);
        self.release_temp_local(lhs_tag);
        self.release_temp_local(lhs_payload);
        self.release_temp_local(rhs_raw_tag);
        self.release_temp_local(rhs_raw_payload);
        self.release_temp_local(lhs_raw_tag);
        self.release_temp_local(lhs_raw_payload);
        return Ok(());
    }

    fn compile_loose_equality_nonstring_i32(
        &mut self,
        lhs: &TypedExpr,
        rhs: &TypedExpr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if !lhs.possible_kinds.is_subset_of(KindSet::PRIMITIVE_ONLY)
            || !rhs.possible_kinds.is_subset_of(KindSet::PRIMITIVE_ONLY)
        {
            return self.compile_loose_equality_i32(lhs, rhs, function);
        }
        let lhs_payload = self.reserve_temp_local();
        let lhs_tag = self.reserve_temp_local();
        let rhs_payload = self.reserve_temp_local();
        let rhs_tag = self.reserve_temp_local();
        let temp_number_local = self.reserve_temp_local();
        self.compile_expr_to_locals(lhs, lhs_payload, lhs_tag, function)?;
        self.compile_expr_to_locals(rhs, rhs_payload, rhs_tag, function)?;
        function.instruction(&Instruction::LocalGet(lhs_tag));
        function.instruction(&Instruction::LocalGet(rhs_tag));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_nonstring_tagged_payload_equality_i32(
            lhs_tag,
            lhs_payload,
            rhs_tag,
            rhs_payload,
            function,
        );
        function.instruction(&Instruction::Else);
        self.compile_nullish_tagged_i32(lhs_tag, function)?;
        self.compile_nullish_tagged_i32(rhs_tag, function)?;
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(lhs_tag));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_nonstring_value_to_number_payload(lhs_tag, lhs_payload, function)?;
        function.instruction(&Instruction::LocalSet(temp_number_local));
        function.instruction(&Instruction::LocalGet(temp_number_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        self.emit_nonstring_value_to_number_payload(rhs_tag, rhs_payload, function)?;
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(rhs_tag));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_nonstring_value_to_number_payload(rhs_tag, rhs_payload, function)?;
        function.instruction(&Instruction::LocalSet(temp_number_local));
        self.emit_nonstring_value_to_number_payload(lhs_tag, lhs_payload, function)?;
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(temp_number_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(temp_number_local);
        self.release_temp_local(rhs_tag);
        self.release_temp_local(rhs_payload);
        self.release_temp_local(lhs_tag);
        self.release_temp_local(lhs_payload);
        Ok(())
    }

    fn emit_nonstring_tagged_payload_equality_i32(
        &self,
        lhs_tag_local: u32,
        lhs_payload_local: u32,
        _rhs_tag_local: u32,
        rhs_payload_local: u32,
        function: &mut Function,
    ) {
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
    }

    fn emit_loose_tagged_equality_i32(
        &mut self,
        lhs_tag_local: u32,
        lhs_payload_local: u32,
        rhs_tag_local: u32,
        rhs_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let temp_number_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(lhs_tag_local));
        function.instruction(&Instruction::LocalGet(rhs_tag_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_tagged_payload_equality_i32(
            lhs_tag_local,
            lhs_payload_local,
            rhs_tag_local,
            rhs_payload_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.compile_nullish_tagged_i32(lhs_tag_local, function)?;
        self.compile_nullish_tagged_i32(rhs_tag_local, function)?;
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(lhs_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_value_to_number_payload(lhs_tag_local, lhs_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(temp_number_local));
        self.emit_number_payload_loose_equal_i32(
            temp_number_local,
            rhs_tag_local,
            rhs_payload_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(rhs_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_value_to_number_payload(rhs_tag_local, rhs_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(temp_number_local));
        self.emit_number_payload_loose_equal_i32(
            temp_number_local,
            lhs_tag_local,
            lhs_payload_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(lhs_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(rhs_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_number_payload_loose_equal_i32(
            lhs_payload_local,
            rhs_tag_local,
            rhs_payload_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(lhs_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(rhs_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_number_payload_loose_equal_i32(
            rhs_payload_local,
            lhs_tag_local,
            lhs_payload_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(temp_number_local);
        Ok(())
    }

    fn emit_number_payload_loose_equal_i32(
        &mut self,
        number_payload_local: u32,
        other_tag_local: u32,
        other_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let other_number_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(other_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(other_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(other_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_string_to_number_payload(other_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(other_number_local));
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(other_number_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(other_number_local);
        Ok(())
    }

    fn compile_compare_value_i32(
        &mut self,
        op: RelationalBinaryOp,
        lhs: &TypedExpr,
        rhs: &TypedExpr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if lhs.possible_kinds.is_subset_of(KindSet::PRIMITIVE_ONLY)
            && rhs.possible_kinds.is_subset_of(KindSet::PRIMITIVE_ONLY)
        {
            let lhs_payload = self.reserve_temp_local();
            let lhs_tag = self.reserve_temp_local();
            let rhs_payload = self.reserve_temp_local();
            let rhs_tag = self.reserve_temp_local();
            self.compile_expr_to_locals(lhs, lhs_payload, lhs_tag, function)?;
            self.compile_expr_to_locals(rhs, rhs_payload, rhs_tag, function)?;
            self.emit_compare_tagged_i32(op, lhs_tag, lhs_payload, rhs_tag, rhs_payload, function)?;
            self.release_temp_local(rhs_tag);
            self.release_temp_local(rhs_payload);
            self.release_temp_local(lhs_tag);
            self.release_temp_local(lhs_payload);
            return Ok(());
        }
        let lhs_payload = self.reserve_temp_local();
        let lhs_tag = self.reserve_temp_local();
        let rhs_payload = self.reserve_temp_local();
        let rhs_tag = self.reserve_temp_local();
        self.compile_expr_to_primitive_locals(
            lhs,
            ToPrimitiveHint::Number,
            lhs_payload,
            lhs_tag,
            function,
        )?;
        self.compile_expr_to_primitive_locals(
            rhs,
            ToPrimitiveHint::Number,
            rhs_payload,
            rhs_tag,
            function,
        )?;
        self.emit_compare_tagged_i32(op, lhs_tag, lhs_payload, rhs_tag, rhs_payload, function)?;
        self.release_temp_local(rhs_tag);
        self.release_temp_local(rhs_payload);
        self.release_temp_local(lhs_tag);
        self.release_temp_local(lhs_payload);
        Ok(())
    }

    fn emit_compare_tagged_i32(
        &mut self,
        op: RelationalBinaryOp,
        lhs_tag_local: u32,
        lhs_payload_local: u32,
        rhs_tag_local: u32,
        rhs_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let lhs_number_local = self.reserve_temp_local();
        let rhs_number_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(lhs_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(rhs_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_string_payload_compare_i32(op, lhs_payload_local, rhs_payload_local, function);
        function.instruction(&Instruction::Else);
        self.emit_value_to_number_payload(lhs_tag_local, lhs_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(lhs_number_local));
        self.emit_value_to_number_payload(rhs_tag_local, rhs_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(rhs_number_local));
        function.instruction(&Instruction::LocalGet(lhs_number_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(rhs_number_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        match op {
            RelationalBinaryOp::LessThan => function.instruction(&Instruction::F64Lt),
            RelationalBinaryOp::LessThanOrEqual => function.instruction(&Instruction::F64Le),
            RelationalBinaryOp::GreaterThan => function.instruction(&Instruction::F64Gt),
            RelationalBinaryOp::GreaterThanOrEqual => function.instruction(&Instruction::F64Ge),
        };
        function.instruction(&Instruction::End);
        self.release_temp_local(rhs_number_local);
        self.release_temp_local(lhs_number_local);
        Ok(())
    }

    fn emit_string_payload_compare_i32(
        &mut self,
        op: RelationalBinaryOp,
        lhs_payload_local: u32,
        rhs_payload_local: u32,
        function: &mut Function,
    ) {
        let lhs_offset = self.reserve_temp_local();
        let lhs_len = self.reserve_temp_local();
        let rhs_offset = self.reserve_temp_local();
        let rhs_len = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let lhs_addr_local = self.reserve_temp_local();
        let rhs_addr_local = self.reserve_temp_local();
        let lhs_byte_local = self.reserve_temp_local();
        let rhs_byte_local = self.reserve_temp_local();
        let result_local = self.reserve_temp_local();
        let done_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(lhs_payload_local, lhs_offset, lhs_len, function);
        self.emit_unpack_string_payload(rhs_payload_local, rhs_offset, rhs_len, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(done_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(lhs_len));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(rhs_len));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(lhs_offset));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(lhs_addr_local));
        function.instruction(&Instruction::LocalGet(rhs_offset));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(rhs_addr_local));

        function.instruction(&Instruction::LocalGet(lhs_addr_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(lhs_byte_local));
        function.instruction(&Instruction::LocalGet(rhs_addr_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(rhs_byte_local));

        function.instruction(&Instruction::LocalGet(lhs_byte_local));
        function.instruction(&Instruction::LocalGet(rhs_byte_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(lhs_byte_local));
        function.instruction(&Instruction::LocalGet(rhs_byte_local));
        match op {
            RelationalBinaryOp::LessThan => function.instruction(&Instruction::I64LtU),
            RelationalBinaryOp::LessThanOrEqual => function.instruction(&Instruction::I64LeU),
            RelationalBinaryOp::GreaterThan => function.instruction(&Instruction::I64GtU),
            RelationalBinaryOp::GreaterThanOrEqual => function.instruction(&Instruction::I64GeU),
        };
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(done_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(done_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(lhs_len));
        function.instruction(&Instruction::LocalGet(rhs_len));
        match op {
            RelationalBinaryOp::LessThan => function.instruction(&Instruction::I64LtU),
            RelationalBinaryOp::LessThanOrEqual => function.instruction(&Instruction::I64LeU),
            RelationalBinaryOp::GreaterThan => function.instruction(&Instruction::I64GtU),
            RelationalBinaryOp::GreaterThanOrEqual => function.instruction(&Instruction::I64GeU),
        };
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I32WrapI64);

        self.release_temp_local(done_local);
        self.release_temp_local(result_local);
        self.release_temp_local(rhs_byte_local);
        self.release_temp_local(lhs_byte_local);
        self.release_temp_local(rhs_addr_local);
        self.release_temp_local(lhs_addr_local);
        self.release_temp_local(index_local);
        self.release_temp_local(rhs_len);
        self.release_temp_local(rhs_offset);
        self.release_temp_local(lhs_len);
        self.release_temp_local(lhs_offset);
    }

    fn compile_typeof_payload(
        &mut self,
        expr: &TypedExpr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if expr.possible_kinds.is_singleton() {
            self.emit_typeof_payload_for_kind(expr.kind, function);
            return Ok(());
        }
        self.compile_expr_to_locals(expr, self.scratch_local, self.result_tag_local, function)?;
        self.emit_typeof_payload_from_tag_local(self.result_tag_local, function);
        Ok(())
    }

    fn emit_typeof_payload_for_kind(&self, kind: ValueKind, function: &mut Function) {
        let value = match kind {
            ValueKind::Undefined => "undefined",
            ValueKind::Null | ValueKind::Object | ValueKind::Array | ValueKind::Arguments => {
                "object"
            }
            ValueKind::Boolean => "boolean",
            ValueKind::Number => "number",
            ValueKind::String => "string",
            ValueKind::Function => "function",
            ValueKind::Dynamic => unreachable!(),
        };
        function.instruction(&Instruction::I64Const(self.strings.payload(value)));
    }

    fn emit_typeof_payload_from_tag_local(&self, tag_local: u32, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(self.strings.payload("undefined")));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(self.strings.payload("boolean")));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(self.strings.payload("number")));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(self.strings.payload("string")));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(self.strings.payload("function")));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("object")));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    fn compile_string_concat_payload(
        &mut self,
        lhs: &TypedExpr,
        rhs: &TypedExpr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let lhs_payload = self.reserve_temp_local();
        let lhs_tag = self.reserve_temp_local();
        let rhs_payload = self.reserve_temp_local();
        let rhs_tag = self.reserve_temp_local();
        let lhs_string = self.reserve_temp_local();
        let rhs_string = self.reserve_temp_local();
        let lhs_offset = self.reserve_temp_local();
        let lhs_len = self.reserve_temp_local();
        let rhs_offset = self.reserve_temp_local();
        let rhs_len = self.reserve_temp_local();
        let total_len = self.reserve_temp_local();
        let dst_offset = self.reserve_temp_local();
        let rhs_dst_offset = self.reserve_temp_local();

        self.compile_expr_to_locals(lhs, lhs_payload, lhs_tag, function)?;
        self.emit_value_to_string_payload(lhs_payload, lhs_tag, function)?;
        function.instruction(&Instruction::LocalSet(lhs_string));
        self.compile_expr_to_locals(rhs, rhs_payload, rhs_tag, function)?;
        self.emit_value_to_string_payload(rhs_payload, rhs_tag, function)?;
        function.instruction(&Instruction::LocalSet(rhs_string));

        self.emit_unpack_string_payload(lhs_string, lhs_offset, lhs_len, function);
        self.emit_unpack_string_payload(rhs_string, rhs_offset, rhs_len, function);

        function.instruction(&Instruction::LocalGet(lhs_len));
        function.instruction(&Instruction::LocalGet(rhs_len));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(total_len));
        self.emit_heap_alloc_from_local(total_len, function)?;
        function.instruction(&Instruction::LocalSet(dst_offset));

        self.emit_copy_bytes(lhs_offset, dst_offset, lhs_len, function);
        function.instruction(&Instruction::LocalGet(dst_offset));
        function.instruction(&Instruction::LocalGet(lhs_len));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(rhs_dst_offset));
        self.emit_copy_bytes(rhs_offset, rhs_dst_offset, rhs_len, function);
        self.emit_pack_string_payload(dst_offset, total_len, function);

        self.release_temp_local(rhs_dst_offset);
        self.release_temp_local(dst_offset);
        self.release_temp_local(total_len);
        self.release_temp_local(rhs_len);
        self.release_temp_local(rhs_offset);
        self.release_temp_local(lhs_len);
        self.release_temp_local(lhs_offset);
        self.release_temp_local(rhs_string);
        self.release_temp_local(lhs_string);
        self.release_temp_local(rhs_tag);
        self.release_temp_local(rhs_payload);
        self.release_temp_local(lhs_tag);
        self.release_temp_local(lhs_payload);
        Ok(())
    }

    fn emit_function_to_string_payload(
        &mut self,
        payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let table_index_local = self.reserve_temp_local();
        let table_number_local = self.reserve_temp_local();
        let prefix_local = self.reserve_temp_local();
        let number_string_local = self.reserve_temp_local();
        let joined_local = self.reserve_temp_local();
        let suffix_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::I64Const(0xFFFF_FFFFu64 as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(table_index_local));
        function.instruction(&Instruction::LocalGet(table_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(table_number_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("function(handle@"),
        ));
        function.instruction(&Instruction::LocalSet(prefix_local));
        self.emit_number_to_string_payload(table_number_local, function)?;
        function.instruction(&Instruction::LocalSet(number_string_local));
        self.emit_concat_string_payloads_local(prefix_local, number_string_local, function)?;
        function.instruction(&Instruction::LocalSet(joined_local));
        function.instruction(&Instruction::I64Const(self.strings.payload(")")));
        function.instruction(&Instruction::LocalSet(suffix_local));
        self.emit_concat_string_payloads_local(joined_local, suffix_local, function)?;

        self.release_temp_local(suffix_local);
        self.release_temp_local(joined_local);
        self.release_temp_local(number_string_local);
        self.release_temp_local(prefix_local);
        self.release_temp_local(table_number_local);
        self.release_temp_local(table_index_local);
        Ok(())
    }

    fn emit_value_to_string_payload(
        &mut self,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(self.strings.payload("undefined")));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(self.strings.payload("null")));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(self.strings.payload("false")));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("true")));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        self.emit_number_to_string_payload(payload_local, function)?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        self.emit_function_to_string_payload(payload_local, function)?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        let primitive_payload_local = self.reserve_temp_local();
        let primitive_tag_local = self.reserve_temp_local();
        self.emit_object_to_primitive_locals(
            ToPrimitiveHint::String,
            payload_local,
            primitive_payload_local,
            primitive_tag_local,
            function,
        )?;
        self.emit_primitive_to_string_payload(
            primitive_payload_local,
            primitive_tag_local,
            function,
        )?;
        self.release_temp_local(primitive_tag_local);
        self.release_temp_local(primitive_payload_local);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        let array_string_local = self.reserve_temp_local();
        let array_tag_local = self.reserve_temp_local();
        self.emit_array_to_string_locals(
            payload_local,
            array_string_local,
            array_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(array_string_local));
        self.release_temp_local(array_tag_local);
        self.release_temp_local(array_string_local);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("[object Arguments]"),
        ));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn emit_primitive_to_string_payload(
        &mut self,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(self.strings.payload("undefined")));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(self.strings.payload("null")));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(self.strings.payload("false")));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("true")));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        self.emit_number_to_string_payload(payload_local, function)?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn emit_number_to_string_payload(
        &mut self,
        payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let output_local = self.reserve_temp_local();
        let sign_local = self.reserve_temp_local();
        let abs_local = self.reserve_temp_local();
        let int_f_local = self.reserve_temp_local();
        let int_u_local = self.reserve_temp_local();
        let frac_scaled_local = self.reserve_temp_local();
        let frac_width_local = self.reserve_temp_local();
        let int_digits_local = self.reserve_temp_local();
        let total_len_local = self.reserve_temp_local();
        let dst_offset_local = self.reserve_temp_local();
        let int_start_local = self.reserve_temp_local();
        let frac_start_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("NaN")));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Abs);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(abs_local));
        function.instruction(&Instruction::LocalGet(abs_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(self.strings.payload("-Infinity")));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("Infinity")));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(sign_local));
        function.instruction(&Instruction::LocalGet(abs_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(int_f_local));
        function.instruction(&Instruction::LocalGet(int_f_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(int_u_local));
        function.instruction(&Instruction::LocalGet(abs_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(int_f_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Sub);
        function.instruction(&Instruction::F64Const(Ieee64::from(1_000_000.0)));
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::F64Nearest);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(frac_scaled_local));
        function.instruction(&Instruction::LocalGet(frac_scaled_local));
        function.instruction(&Instruction::I64Const(1_000_000));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(int_u_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(int_u_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(frac_scaled_local));
        function.instruction(&Instruction::End);
        self.emit_count_decimal_digits_u64(int_u_local, int_digits_local, function);
        self.emit_fraction_width_local(frac_scaled_local, frac_width_local, function);
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::LocalGet(int_digits_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(frac_width_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(frac_width_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(total_len_local));
        self.emit_heap_alloc_from_local(total_len_local, function)?;
        function.instruction(&Instruction::LocalSet(dst_offset_local));
        function.instruction(&Instruction::LocalGet(dst_offset_local));
        function.instruction(&Instruction::LocalSet(int_start_local));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_ascii_byte_i64(dst_offset_local, b'-', function);
        function.instruction(&Instruction::LocalGet(dst_offset_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(int_start_local));
        function.instruction(&Instruction::End);
        self.emit_write_decimal_u64(int_u_local, int_start_local, int_digits_local, function);
        function.instruction(&Instruction::LocalGet(int_start_local));
        function.instruction(&Instruction::LocalGet(int_digits_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(frac_start_local));
        function.instruction(&Instruction::LocalGet(frac_width_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.store_ascii_byte_i64(frac_start_local, b'.', function);
        function.instruction(&Instruction::LocalGet(frac_start_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(frac_start_local));
        self.emit_write_decimal_u64(
            frac_scaled_local,
            frac_start_local,
            frac_width_local,
            function,
        );
        function.instruction(&Instruction::End);
        self.emit_pack_string_payload(dst_offset_local, total_len_local, function);
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(output_local));

        self.release_temp_local(frac_start_local);
        self.release_temp_local(int_start_local);
        self.release_temp_local(dst_offset_local);
        self.release_temp_local(total_len_local);
        self.release_temp_local(int_digits_local);
        self.release_temp_local(frac_width_local);
        self.release_temp_local(frac_scaled_local);
        self.release_temp_local(int_u_local);
        self.release_temp_local(int_f_local);
        self.release_temp_local(abs_local);
        self.release_temp_local(sign_local);
        self.release_temp_local(output_local);
        Ok(())
    }

    fn emit_fraction_width_local(
        &mut self,
        frac_scaled_local: u32,
        width_local: u32,
        function: &mut Function,
    ) {
        let temp_local = self.reserve_temp_local();
        let zeros_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(frac_scaled_local));
        function.instruction(&Instruction::LocalSet(temp_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(zeros_local));
        function.instruction(&Instruction::LocalGet(frac_scaled_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(width_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64RemU);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(temp_local));
        function.instruction(&Instruction::LocalGet(zeros_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(zeros_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::LocalGet(zeros_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(width_local));
        function.instruction(&Instruction::End);
        self.release_temp_local(zeros_local);
        self.release_temp_local(temp_local);
    }

    fn emit_count_decimal_digits_u64(
        &mut self,
        value_local: u32,
        digits_local: u32,
        function: &mut Function,
    ) {
        let temp_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::LocalSet(temp_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(digits_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(temp_local));
        function.instruction(&Instruction::LocalGet(digits_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(digits_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(temp_local);
    }

    fn emit_write_decimal_u64(
        &mut self,
        value_local: u32,
        start_offset_local: u32,
        digits_local: u32,
        function: &mut Function,
    ) {
        let temp_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let pos_local = self.reserve_temp_local();
        let digit_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::LocalSet(temp_local));
        function.instruction(&Instruction::LocalGet(start_offset_local));
        function.instruction(&Instruction::LocalGet(digits_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(pos_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(digits_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(pos_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(pos_local));
        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64RemU);
        function.instruction(&Instruction::LocalSet(digit_local));
        function.instruction(&Instruction::LocalGet(pos_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(temp_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(digit_local);
        self.release_temp_local(pos_local);
        self.release_temp_local(index_local);
        self.release_temp_local(temp_local);
    }

    fn store_ascii_byte_i64(&self, offset_local: u32, byte: u8, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(offset_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Const(i32::from(byte)));
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
    }

    fn emit_unpack_string_payload(
        &self,
        payload_local: u32,
        offset_local: u32,
        len_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(offset_local));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::I64Const(0xFFFF_FFFFu64 as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(len_local));
    }

    fn emit_pack_string_payload(&self, offset_local: u32, len_local: u32, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(offset_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Or);
    }

    fn emit_copy_bytes(
        &mut self,
        src_offset_local: u32,
        dst_offset_local: u32,
        len_local: u32,
        function: &mut Function,
    ) {
        let index_local = self.reserve_temp_local();
        let src_addr_local = self.reserve_temp_local();
        let dst_addr_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(src_offset_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(src_addr_local));
        function.instruction(&Instruction::LocalGet(src_addr_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(byte_local));
        function.instruction(&Instruction::LocalGet(dst_offset_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(dst_addr_local));
        function.instruction(&Instruction::LocalGet(dst_addr_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(byte_local);
        self.release_temp_local(dst_addr_local);
        self.release_temp_local(src_addr_local);
        self.release_temp_local(index_local);
    }

    fn emit_string_payload_equality_i32(
        &mut self,
        lhs_payload_local: u32,
        rhs_payload_local: u32,
        function: &mut Function,
    ) {
        let lhs_offset = self.reserve_temp_local();
        let lhs_len = self.reserve_temp_local();
        let rhs_offset = self.reserve_temp_local();
        let rhs_len = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let lhs_addr_local = self.reserve_temp_local();
        let rhs_addr_local = self.reserve_temp_local();
        let lhs_byte_local = self.reserve_temp_local();
        let rhs_byte_local = self.reserve_temp_local();
        let result_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(lhs_payload_local, lhs_offset, lhs_len, function);
        self.emit_unpack_string_payload(rhs_payload_local, rhs_offset, rhs_len, function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::LocalGet(lhs_len));
        function.instruction(&Instruction::LocalGet(rhs_len));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(lhs_len));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(lhs_offset));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(lhs_addr_local));
        function.instruction(&Instruction::LocalGet(rhs_offset));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(rhs_addr_local));
        function.instruction(&Instruction::LocalGet(lhs_addr_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(lhs_byte_local));
        function.instruction(&Instruction::LocalGet(rhs_addr_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(rhs_byte_local));
        function.instruction(&Instruction::LocalGet(lhs_byte_local));
        function.instruction(&Instruction::LocalGet(rhs_byte_local));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I32WrapI64);

        self.release_temp_local(result_local);
        self.release_temp_local(rhs_byte_local);
        self.release_temp_local(lhs_byte_local);
        self.release_temp_local(rhs_addr_local);
        self.release_temp_local(lhs_addr_local);
        self.release_temp_local(index_local);
        self.release_temp_local(rhs_len);
        self.release_temp_local(rhs_offset);
        self.release_temp_local(lhs_len);
        self.release_temp_local(lhs_offset);
    }

    fn compile_strict_equality_i32(
        &mut self,
        lhs: &TypedExpr,
        rhs: &TypedExpr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if lhs.possible_kinds.is_singleton()
            && rhs.possible_kinds.is_singleton()
            && lhs.kind != rhs.kind
        {
            function.instruction(&Instruction::I32Const(0));
            return Ok(());
        }

        if lhs.possible_kinds.is_singleton() && rhs.possible_kinds.is_singleton() {
            match lhs.kind {
                ValueKind::Number => {
                    self.compile_expr_payload(lhs, function)?;
                    function.instruction(&Instruction::F64ReinterpretI64);
                    self.compile_expr_payload(rhs, function)?;
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::F64Eq);
                }
                ValueKind::String => {
                    self.compile_expr_payload(lhs, function)?;
                    function.instruction(&Instruction::LocalSet(self.scratch_local));
                    self.compile_expr_payload(rhs, function)?;
                    function.instruction(&Instruction::LocalSet(self.result_local));
                    self.emit_string_payload_equality_i32(
                        self.scratch_local,
                        self.result_local,
                        function,
                    );
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
        &mut self,
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
        function.instruction(&Instruction::LocalGet(lhs_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_string_payload_equality_i32(lhs_payload_local, rhs_payload_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(lhs_payload_local));
        function.instruction(&Instruction::LocalGet(rhs_payload_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::End);
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
        self.set_completion_kind(CompletionKind::Normal, function);
    }

    fn emit_undefined_payload(&self, function: &mut Function) {
        function.instruction(&Instruction::I64Const(0));
    }

    fn save_current_completion(
        &self,
        payload_local: u32,
        tag_local: u32,
        completion_local: u32,
        aux_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::LocalSet(completion_local));
        function.instruction(&Instruction::LocalGet(self.completion_aux_local));
        function.instruction(&Instruction::LocalSet(aux_local));
    }

    fn restore_saved_completion(
        &self,
        payload_local: u32,
        tag_local: u32,
        completion_local: u32,
        aux_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::LocalGet(completion_local));
        function.instruction(&Instruction::LocalSet(self.completion_local));
        function.instruction(&Instruction::LocalGet(aux_local));
        function.instruction(&Instruction::LocalSet(self.completion_aux_local));
    }

    fn set_completion_kind(&self, kind: CompletionKind, function: &mut Function) {
        function.instruction(&Instruction::I64Const(kind.code()));
        function.instruction(&Instruction::LocalSet(self.completion_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.completion_aux_local));
    }

    fn set_completion_kind_with_aux(
        &self,
        kind: CompletionKind,
        aux: i64,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(kind.code()));
        function.instruction(&Instruction::LocalSet(self.completion_local));
        function.instruction(&Instruction::I64Const(aux));
        function.instruction(&Instruction::LocalSet(self.completion_aux_local));
    }

    fn emit_return_current_completion(&self, function: &mut Function) {
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
                function.instruction(&Instruction::Return);
            }
            ReturnAbi::MultiValue => {
                function.instruction(&Instruction::LocalGet(self.result_local));
                function.instruction(&Instruction::LocalGet(self.result_tag_local));
                function.instruction(&Instruction::LocalGet(self.completion_local));
                function.instruction(&Instruction::LocalGet(self.completion_aux_local));
                function.instruction(&Instruction::Return);
            }
        }
    }

    fn emit_resume_after_finally(
        &mut self,
        saved_payload_local: u32,
        saved_tag_local: u32,
        saved_completion_local: u32,
        saved_aux_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_NORMAL));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.restore_saved_completion(
            saved_payload_local,
            saved_tag_local,
            saved_completion_local,
            saved_aux_local,
            function,
        );
        self.emit_dispatch_current_completion_with_extra_depth(1, function)?;
        function.instruction(&Instruction::Else);
        self.emit_dispatch_current_completion_with_extra_depth(1, function)?;
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn emit_dispatch_branch_completion(
        &self,
        targets: &[(u32, usize)],
        extra_depth: u32,
        function: &mut Function,
    ) {
        for (target_id, frame) in targets {
            function.instruction(&Instruction::LocalGet(self.completion_aux_local));
            function.instruction(&Instruction::I64Const(*target_id as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Br(self.depth_to(*frame) + extra_depth));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::Unreachable);
    }

    fn active_break_targets(&self) -> Vec<(u32, usize)> {
        let mut targets = Vec::new();
        for frame in self.breakable_stack.iter().rev() {
            let target_id = *frame as u32;
            if !targets.iter().any(|(id, _)| *id == target_id) {
                targets.push((target_id, *frame));
            }
        }
        targets
    }

    fn active_continue_targets(&self) -> Vec<(u32, usize)> {
        let mut targets = Vec::new();
        for target in self.loop_stack.iter().rev() {
            let target_id = target.continue_frame as u32;
            if !targets.iter().any(|(id, _)| *id == target_id) {
                targets.push((target_id, target.continue_frame));
            }
        }
        targets
    }

    fn emit_dispatch_current_completion(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_dispatch_current_completion_with_extra_depth(0, function)
    }

    fn emit_dispatch_current_completion_with_extra_depth(
        &mut self,
        extra_depth: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        if let Some(target) = self.throw_handler_stack.last() {
            function.instruction(&Instruction::Br(self.depth_to(*target) + 1 + extra_depth));
        } else if let Some(target) = self.finally_stack.last() {
            function.instruction(&Instruction::Br(self.depth_to(*target) + 1 + extra_depth));
        } else {
            self.emit_return_current_completion(function);
        }
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_RETURN));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        if let Some(target) = self.finally_stack.last() {
            function.instruction(&Instruction::Br(self.depth_to(*target) + 2 + extra_depth));
        } else {
            self.normalize_derived_constructor_result(function)?;
            self.set_completion_kind(CompletionKind::Normal, function);
            self.emit_return_current_completion(function);
        }
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_BREAK));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        if let Some(target) = self.finally_stack.last() {
            function.instruction(&Instruction::Br(self.depth_to(*target) + 3 + extra_depth));
        } else {
            let targets = self.active_break_targets();
            self.emit_dispatch_branch_completion(&targets, 4 + extra_depth, function);
        }
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_CONTINUE));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        if let Some(target) = self.finally_stack.last() {
            function.instruction(&Instruction::Br(self.depth_to(*target) + 4 + extra_depth));
        } else {
            let targets = self.active_continue_targets();
            self.emit_dispatch_branch_completion(&targets, 5 + extra_depth, function);
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn emit_throw_from_locals(&self, payload_local: u32, tag_local: u32, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Throw, function);
    }

    fn emit_propagate_throw_from_locals_if_needed(
        &self,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_from_locals(payload_local, tag_local, function);
        if let Some(target) = self.throw_handler_stack.last() {
            function.instruction(&Instruction::Br(self.depth_to(*target) + 1));
        } else {
            self.emit_return_current_completion(function);
        }
        function.instruction(&Instruction::End);
    }

    fn emit_propagate_throw_from_locals_if_needed_with_extra_depth(
        &self,
        payload_local: u32,
        tag_local: u32,
        extra_depth: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_from_locals(payload_local, tag_local, function);
        if let Some(target) = self.throw_handler_stack.last() {
            function.instruction(&Instruction::Br(self.depth_to(*target) + 1 + extra_depth));
        } else {
            self.emit_return_current_completion(function);
        }
        function.instruction(&Instruction::End);
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
                    slot: self
                        .owned_env_slot(&name)
                        .expect("owned env slot should exist"),
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

fn build_function_metas(
    functions: &[FunctionIr],
    standard_builtins: &[StandardBuiltinId],
    host_builtins: &[HostBuiltinId],
    imported_function_count: u32,
) -> BTreeMap<FunctionId, WasmFunctionMeta> {
    let mut metas = BTreeMap::new();
    let mut callable_index = 0u32;
    for function in functions {
        metas.insert(
            function.id.clone(),
            WasmFunctionMeta {
                name: function.name.clone(),
                to_string_value: function.to_string_representation.materialize(),
                wasm_index: imported_function_count + 1 + callable_index,
                table_index: callable_index,
                constructable: function.constructable,
                class_kind: function.class_kind,
                class_heritage_kind: function.class_heritage_kind,
                is_static_class_member: function.is_static_class_member,
                is_derived_constructor: function.is_derived_constructor,
                is_synthetic_default_derived_constructor: function
                    .is_synthetic_default_derived_constructor,
                super_constructor_target: function.super_constructor_target.clone(),
            },
        );
        callable_index += 1;
    }
    for builtin in standard_builtins {
        metas.insert(
            builtin.function_id(),
            WasmFunctionMeta {
                name: builtin.debug_name().to_string(),
                to_string_value: match builtin {
                    StandardBuiltinId::BoundFunctionInvoker => {
                        CallableToStringRepresentation::NativeAnonymous.materialize()
                    }
                    _ => builtin
                        .native_function_name()
                        .map(|name| {
                            CallableToStringRepresentation::NativeNamed(name.to_string())
                                .materialize()
                        })
                        .unwrap_or_else(|| {
                            CallableToStringRepresentation::NativeAnonymous.materialize()
                        }),
                },
                wasm_index: imported_function_count + 1 + callable_index,
                table_index: callable_index,
                constructable: builtin.constructable(),
                class_kind: ClassFunctionKind::None,
                class_heritage_kind: ClassHeritageKind::None,
                is_static_class_member: false,
                is_derived_constructor: false,
                is_synthetic_default_derived_constructor: false,
                super_constructor_target: None,
            },
        );
        callable_index += 1;
    }
    for builtin in host_builtins {
        metas.insert(
            builtin.function_id(),
            WasmFunctionMeta {
                name: builtin.as_str().to_string(),
                to_string_value: CallableToStringRepresentation::NativeNamed(
                    builtin.as_str().to_string(),
                )
                .materialize(),
                wasm_index: imported_function_count + 1 + callable_index,
                table_index: callable_index,
                constructable: false,
                class_kind: ClassFunctionKind::None,
                class_heritage_kind: ClassHeritageKind::None,
                is_static_class_member: false,
                is_derived_constructor: false,
                is_synthetic_default_derived_constructor: false,
                super_constructor_target: None,
            },
        );
        callable_index += 1;
    }
    metas
}

fn function_param_types() -> Vec<ValType> {
    std::iter::repeat_n(ValType::I64, JS_FUNCTION_PARAM_COUNT).collect()
}

fn expr_result_tag_is_runtime_dynamic(expr: &ExprIr) -> bool {
    matches!(
        expr,
        ExprIr::CallNamed { .. }
            | ExprIr::CallIndirect { .. }
            | ExprIr::CallMethod { .. }
            | ExprIr::Construct { .. }
            | ExprIr::SuperConstruct { .. }
    )
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

fn script_uses_function_heap(script: &ScriptIr) -> bool {
    script
        .functions
        .iter()
        .any(|function| function.flavor == FunctionFlavor::Ordinary)
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
        StatementIr::Return(value) | StatementIr::Throw(value) => expr_uses_calls(value),
        StatementIr::Var(declarators) => declarators
            .iter()
            .filter_map(|declarator| declarator.init.as_ref())
            .any(expr_uses_calls),
        StatementIr::Block(block) => block_uses_calls(block),
        StatementIr::TryCatch {
            try_block,
            catch_block,
            ..
        } => block_uses_calls(try_block) || block_uses_calls(catch_block),
        StatementIr::TryFinally {
            try_block,
            finally_block,
        } => block_uses_calls(try_block) || block_uses_calls(finally_block),
        StatementIr::TryCatchFinally {
            try_block,
            catch_block,
            finally_block,
            ..
        } => {
            block_uses_calls(try_block)
                || block_uses_calls(catch_block)
                || block_uses_calls(finally_block)
        }
        StatementIr::If {
            condition,
            then_branch,
            else_branch,
        } => {
            expr_uses_calls(condition)
                || statement_uses_calls(then_branch)
                || else_branch
                    .as_deref()
                    .map(statement_uses_calls)
                    .unwrap_or(false)
        }
        StatementIr::While { condition, body } => {
            expr_uses_calls(condition) || statement_uses_calls(body)
        }
        StatementIr::DoWhile { body, condition } => {
            statement_uses_calls(body) || expr_uses_calls(condition)
        }
        StatementIr::For {
            init,
            test,
            update,
            body,
        } => {
            init.as_ref().map(for_init_uses_calls).unwrap_or(false)
                || test.as_ref().map(expr_uses_calls).unwrap_or(false)
                || update.as_ref().map(expr_uses_calls).unwrap_or(false)
                || statement_uses_calls(body)
        }
        StatementIr::Switch {
            discriminant,
            cases,
        } => {
            expr_uses_calls(discriminant)
                || cases.iter().any(|case| {
                    case.condition
                        .as_ref()
                        .map(expr_uses_calls)
                        .unwrap_or(false)
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
        StatementIr::Empty
        | StatementIr::Debugger
        | StatementIr::Break { .. }
        | StatementIr::Continue { .. } => false,
        StatementIr::Lexical { init, .. }
        | StatementIr::Expression(init)
        | StatementIr::Return(init)
        | StatementIr::Throw(init) => expr_uses_function_table(init),
        StatementIr::Var(declarators) => declarators
            .iter()
            .filter_map(|declarator| declarator.init.as_ref())
            .any(expr_uses_function_table),
        StatementIr::Block(block) => block_uses_function_table(block),
        StatementIr::TryCatch {
            try_block,
            catch_block,
            ..
        } => block_uses_function_table(try_block) || block_uses_function_table(catch_block),
        StatementIr::TryFinally {
            try_block,
            finally_block,
        } => block_uses_function_table(try_block) || block_uses_function_table(finally_block),
        StatementIr::TryCatchFinally {
            try_block,
            catch_block,
            finally_block,
            ..
        } => {
            block_uses_function_table(try_block)
                || block_uses_function_table(catch_block)
                || block_uses_function_table(finally_block)
        }
        StatementIr::If {
            condition,
            then_branch,
            else_branch,
        } => {
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
        StatementIr::For {
            init,
            test,
            update,
            body,
        } => {
            init.as_ref()
                .map(for_init_uses_function_table)
                .unwrap_or(false)
                || test.as_ref().map(expr_uses_function_table).unwrap_or(false)
                || update
                    .as_ref()
                    .map(expr_uses_function_table)
                    .unwrap_or(false)
                || statement_uses_function_table(body)
        }
        StatementIr::Switch {
            discriminant,
            cases,
        } => {
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
        ForInitIr::Lexical { init, .. } | ForInitIr::Expression(init) => {
            expr_uses_function_table(init)
        }
        ForInitIr::Var(declarators) => declarators
            .iter()
            .filter_map(|declarator| declarator.init.as_ref())
            .any(expr_uses_function_table),
    }
}

fn expr_uses_function_table(expr: &TypedExpr) -> bool {
    match &expr.expr {
        ExprIr::FunctionValue(_)
        | ExprIr::CallIndirect { .. }
        | ExprIr::CallMethod { .. }
        | ExprIr::Construct { .. }
        | ExprIr::ClassDefinition(_)
        | ExprIr::SuperConstruct { .. }
        | ExprIr::SuperPropertyRead { .. }
        | ExprIr::SuperPropertyWrite { .. }
        | ExprIr::PrivateRead { .. }
        | ExprIr::PrivateWrite { .. }
        | ExprIr::PrivateIn { .. } => true,
        ExprIr::GlobalPropertyRead { .. } | ExprIr::GlobalPropertyUpdate { .. } => false,
        ExprIr::GlobalPropertyWrite { value, .. }
        | ExprIr::GlobalPropertyCompoundAssign { value, .. } => expr_uses_function_table(value),
        ExprIr::AssignIdentifier { value, .. }
        | ExprIr::CompoundAssignIdentifier { value, .. }
        | ExprIr::UnaryNumber { expr: value, .. }
        | ExprIr::LogicalNot { expr: value }
        | ExprIr::TypeOf { expr: value }
        | ExprIr::Void { expr: value }
        | ExprIr::DeleteValue { expr: value } => expr_uses_function_table(value),
        ExprIr::DeleteIdentifier { .. } | ExprIr::DeleteGlobalProperty { .. } => false,
        ExprIr::TypeOfUnresolvedIdentifier { .. } => false,
        ExprIr::NewTarget => false,
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
        ExprIr::DeleteProperty { target, key } => {
            matches!(target.kind, ValueKind::Object)
                || expr_uses_function_table(target)
                || match key {
                    PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => false,
                    PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                        expr_uses_function_table(expr)
                    }
                }
        }
        ExprIr::BinaryNumber { lhs, rhs, .. }
        | ExprIr::CoerciveAdd { lhs, rhs }
        | ExprIr::CoerciveBinaryNumber { lhs, rhs, .. }
        | ExprIr::CompareNumber { lhs, rhs, .. }
        | ExprIr::CompareValue { lhs, rhs, .. }
        | ExprIr::StrictEquality { lhs, rhs, .. }
        | ExprIr::LooseEquality { lhs, rhs, .. }
        | ExprIr::LogicalShortCircuit { lhs, rhs, .. }
        | ExprIr::In { lhs, rhs }
        | ExprIr::StringConcat { lhs, rhs }
        | ExprIr::Comma { lhs, rhs } => {
            expr_uses_function_table(lhs)
                || expr_uses_function_table(rhs)
                || lhs.possible_kinds.contains(ValueKind::Object)
                || rhs.possible_kinds.contains(ValueKind::Object)
        }
        ExprIr::CallNamed { args, .. } => args.iter().any(expr_uses_function_table),
        ExprIr::InstanceOf { lhs, rhs } => {
            expr_uses_function_table(lhs) || expr_uses_function_table(rhs)
        }
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
        ExprIr::CallNamed { .. }
        | ExprIr::CallIndirect { .. }
        | ExprIr::CallMethod { .. }
        | ExprIr::Construct { .. }
        | ExprIr::ClassDefinition(_)
        | ExprIr::SuperConstruct { .. }
        | ExprIr::SuperPropertyRead { .. }
        | ExprIr::SuperPropertyWrite { .. }
        | ExprIr::PrivateRead { .. }
        | ExprIr::PrivateWrite { .. }
        | ExprIr::PrivateIn { .. } => true,
        ExprIr::GlobalPropertyRead { .. } | ExprIr::GlobalPropertyUpdate { .. } => false,
        ExprIr::GlobalPropertyWrite { value, .. }
        | ExprIr::GlobalPropertyCompoundAssign { value, .. } => expr_uses_calls(value),
        ExprIr::AssignIdentifier { value, .. }
        | ExprIr::CompoundAssignIdentifier { value, .. }
        | ExprIr::UnaryNumber { expr: value, .. }
        | ExprIr::LogicalNot { expr: value }
        | ExprIr::TypeOf { expr: value }
        | ExprIr::Void { expr: value }
        | ExprIr::DeleteValue { expr: value } => expr_uses_calls(value),
        ExprIr::DeleteIdentifier { .. } | ExprIr::DeleteGlobalProperty { .. } => false,
        ExprIr::TypeOfUnresolvedIdentifier { .. } => false,
        ExprIr::NewTarget => false,
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
        ExprIr::DeleteProperty { target, key } => {
            expr_uses_calls(target)
                || match key {
                    PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => false,
                    PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                        expr_uses_calls(expr)
                    }
                }
        }
        ExprIr::BinaryNumber { lhs, rhs, .. }
        | ExprIr::CoerciveAdd { lhs, rhs }
        | ExprIr::CoerciveBinaryNumber { lhs, rhs, .. }
        | ExprIr::CompareNumber { lhs, rhs, .. }
        | ExprIr::CompareValue { lhs, rhs, .. }
        | ExprIr::StrictEquality { lhs, rhs, .. }
        | ExprIr::LooseEquality { lhs, rhs, .. }
        | ExprIr::LogicalShortCircuit { lhs, rhs, .. }
        | ExprIr::In { lhs, rhs }
        | ExprIr::StringConcat { lhs, rhs }
        | ExprIr::Comma { lhs, rhs } => {
            expr_uses_calls(lhs)
                || expr_uses_calls(rhs)
                || lhs.possible_kinds.contains(ValueKind::Object)
                || rhs.possible_kinds.contains(ValueKind::Object)
        }
        ExprIr::InstanceOf { lhs, rhs } => expr_uses_calls(lhs) || expr_uses_calls(rhs),
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
        | StatementIr::Throw(_)
        | StatementIr::Break { .. }
        | StatementIr::Continue { .. } => 0,
        StatementIr::Lexical { .. } => 1,
        StatementIr::Block(block) => count_block_lexicals(block),
        StatementIr::TryCatch { catch_block, .. } => 2 + count_block_lexicals(catch_block),
        StatementIr::TryFinally { finally_block, .. } => count_block_lexicals(finally_block),
        StatementIr::TryCatchFinally {
            catch_block,
            finally_block,
            ..
        } => 2 + count_block_lexicals(catch_block) + count_block_lexicals(finally_block),
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
        StatementIr::Return(value) | StatementIr::Throw(value) => count_expr_temp_locals(value),
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
        StatementIr::TryCatch {
            try_block,
            catch_block,
            ..
        } => count_block_temp_locals(try_block)
            .max(count_block_temp_locals(catch_block))
            .max(2),
        StatementIr::TryFinally {
            try_block,
            finally_block,
        } => count_block_temp_locals(try_block).max(count_block_temp_locals(finally_block)),
        StatementIr::TryCatchFinally {
            try_block,
            catch_block,
            finally_block,
            ..
        } => count_block_temp_locals(try_block)
            .max(count_block_temp_locals(catch_block))
            .max(count_block_temp_locals(finally_block))
            .max(2),
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
        ExprIr::GlobalPropertyRead { .. } => 12,
        ExprIr::GlobalPropertyWrite { value, .. } => count_expr_temp_locals(value).max(12),
        ExprIr::GlobalPropertyUpdate { return_mode, .. } => match return_mode {
            UpdateReturnMode::Prefix => 12,
            UpdateReturnMode::Postfix => 13,
        },
        ExprIr::GlobalPropertyCompoundAssign { value, .. } => count_expr_temp_locals(value).max(13),
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
        ExprIr::DeleteProperty { target, key } => {
            let child = count_expr_temp_locals(target).max(match key {
                PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => 0,
                PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                    count_expr_temp_locals(expr)
                }
            });
            child.max(12)
        }
        ExprIr::DeleteIdentifier { .. } => 0,
        ExprIr::DeleteGlobalProperty { .. } => 12,
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
        | ExprIr::LogicalNot { expr: value }
        | ExprIr::TypeOf { expr: value }
        | ExprIr::Void { expr: value }
        | ExprIr::DeleteValue { expr: value } => count_expr_temp_locals(value),
        ExprIr::TypeOfUnresolvedIdentifier { .. } => 0,
        ExprIr::NewTarget => 0,
        ExprIr::BinaryNumber { lhs, rhs, .. }
        | ExprIr::CoerciveBinaryNumber { lhs, rhs, .. }
        | ExprIr::CompareNumber { lhs, rhs, .. }
        | ExprIr::CompareValue { lhs, rhs, .. }
        | ExprIr::LogicalShortCircuit { lhs, rhs, .. }
        | ExprIr::In { lhs, rhs } => count_expr_temp_locals(lhs).max(count_expr_temp_locals(rhs)),
        ExprIr::StringConcat { lhs, rhs } => count_expr_temp_locals(lhs)
            .max(count_expr_temp_locals(rhs))
            .max(18),
        ExprIr::CoerciveAdd { lhs, rhs } => count_expr_temp_locals(lhs)
            .max(count_expr_temp_locals(rhs))
            .max(96),
        ExprIr::Comma { lhs, rhs } => count_expr_temp_locals(lhs).max(count_expr_temp_locals(rhs)),
        ExprIr::StrictEquality { lhs, rhs, .. } => {
            let child = count_expr_temp_locals(lhs).max(count_expr_temp_locals(rhs));
            if lhs.kind == ValueKind::Dynamic || rhs.kind == ValueKind::Dynamic {
                child.max(4)
            } else {
                child
            }
        }
        ExprIr::LooseEquality { lhs, rhs, .. } => count_expr_temp_locals(lhs)
            .max(count_expr_temp_locals(rhs))
            .max(5),
        ExprIr::CallNamed { args, .. } => args
            .iter()
            .map(count_expr_temp_locals)
            .max()
            .unwrap_or(0)
            .max(4),
        ExprIr::CallIndirect {
            callee,
            args,
            this_arg,
        } => count_expr_temp_locals(callee)
            .max(this_arg.as_deref().map(count_expr_temp_locals).unwrap_or(0))
            .max(args.iter().map(count_expr_temp_locals).max().unwrap_or(0))
            .max(6),
        ExprIr::Construct { callee, args } => count_expr_temp_locals(callee)
            .max(args.iter().map(count_expr_temp_locals).max().unwrap_or(0))
            .max(10),
        ExprIr::CallMethod {
            receiver,
            key,
            args,
        } => {
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
        ExprIr::InstanceOf { lhs, rhs } => count_expr_temp_locals(lhs)
            .max(count_expr_temp_locals(rhs))
            .max(8),
        ExprIr::ClassDefinition(_) => 24,
        ExprIr::SuperConstruct { args } => args
            .iter()
            .map(count_expr_temp_locals)
            .max()
            .unwrap_or(0)
            .max(12),
        ExprIr::SuperPropertyRead { key } => match key {
            PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => 8,
            PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                count_expr_temp_locals(expr).max(8)
            }
        },
        ExprIr::SuperPropertyWrite { key, value } => {
            let key_child = match key {
                PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => 0,
                PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                    count_expr_temp_locals(expr)
                }
            };
            count_expr_temp_locals(value).max(key_child).max(10)
        }
        ExprIr::PrivateRead { target, .. } => count_expr_temp_locals(target).max(8),
        ExprIr::PrivateWrite { target, value, .. } => count_expr_temp_locals(target)
            .max(count_expr_temp_locals(value))
            .max(10),
        ExprIr::PrivateIn { rhs, .. } => count_expr_temp_locals(rhs).max(8),
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
        StatementIr::TryCatch {
            try_block,
            catch_block,
            ..
        } => {
            collect_hoisted_vars_block(try_block, names);
            collect_hoisted_vars_block(catch_block, names);
        }
        StatementIr::TryFinally {
            try_block,
            finally_block,
        } => {
            collect_hoisted_vars_block(try_block, names);
            collect_hoisted_vars_block(finally_block, names);
        }
        StatementIr::TryCatchFinally {
            try_block,
            catch_block,
            finally_block,
            ..
        } => {
            collect_hoisted_vars_block(try_block, names);
            collect_hoisted_vars_block(catch_block, names);
            collect_hoisted_vars_block(finally_block, names);
        }
        StatementIr::Empty
        | StatementIr::Lexical { .. }
        | StatementIr::Expression(_)
        | StatementIr::Debugger
        | StatementIr::Return(_)
        | StatementIr::Throw(_)
        | StatementIr::Break { .. }
        | StatementIr::Continue { .. } => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use porffor_front::{parse, ParseOptions};
    use porffor_ir::lower;
    use wasmparser::{Operator, Parser, Payload};

    fn emit_script(source: &str) -> Result<WasmArtifact, EmitError> {
        let source = parse(source, ParseOptions::script()).expect("script should parse");
        emit(&lower(&source))
    }

    fn data_segment_bytes(bytes: &[u8]) -> Vec<u8> {
        let mut collected = Vec::new();
        for payload in Parser::new(0).parse_all(bytes) {
            match payload.expect("wasm parse should succeed") {
                Payload::DataSection(reader) => {
                    for segment in reader {
                        let segment = segment.expect("data segment should decode");
                        match segment.kind {
                            wasmparser::DataKind::Active { .. } | wasmparser::DataKind::Passive => {
                                collected.extend_from_slice(segment.data);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        collected
    }

    fn contains_i64_const(bytes: &[u8], needle: i64) -> bool {
        for payload in Parser::new(0).parse_all(bytes) {
            if let Payload::CodeSectionEntry(body) = payload.expect("wasm parse should succeed") {
                let mut reader = body
                    .get_operators_reader()
                    .expect("operators should decode");
                while !reader.eof() {
                    if let Operator::I64Const { value } =
                        reader.read().expect("operator should decode")
                    {
                        if value == needle {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    fn global_init_i64s(bytes: &[u8]) -> Vec<i64> {
        let mut values = Vec::new();
        for payload in Parser::new(0).parse_all(bytes) {
            if let Payload::GlobalSection(reader) = payload.expect("wasm parse should succeed") {
                for global in reader {
                    let global = global.expect("global should decode");
                    if let wasmparser::ValType::I64 = global.ty.content_type {
                        let mut init = global.init_expr.get_operators_reader();
                        match init.read().expect("global init op should decode") {
                            Operator::I64Const { value } => values.push(value),
                            op => panic!("unexpected i64 global init op: {op:?}"),
                        }
                    }
                }
            }
        }
        values
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
    fn preseeded_string_bytes_and_literal_payloads_are_stable() {
        let artifact = emit_script("\",\";").expect("emit should work");
        let data = data_segment_bytes(&artifact.bytes);
        assert!(
            data.starts_with(b" : ,undefinednulltruefalse"),
            "unexpected data prefix: {:?}",
            &data[..data.len().min(32)]
        );
        assert!(
            contains_i64_const(
                &artifact.bytes,
                ((((STATIC_DATA_OFFSET as u64) + 3) << 32) | 1) as i64,
            ),
            "comma literal payload should be emitted as packed offset/len"
        );
        let globals = global_init_i64s(&artifact.bytes);
        assert!(
            globals.contains(&(align_heap_start(data.len()) as i64)),
            "heap ptr global should start after static data"
        );
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
        assert!(artifact.debug_dump.contains("internal functions: "));
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
        assert!(artifact
            .debug_dump
            .contains("memory: exported linear memory"));
    }

    #[test]
    fn supports_dynamic_primitive_string_concat() {
        let artifact = emit_script("\"a\" + \"b\";").expect("emit should work");
        wasmparser::Validator::new()
            .validate_all(&artifact.bytes)
            .expect("module should validate");
    }

    #[test]
    fn supports_primitive_coercion_core() {
        let artifact = emit_script("1 == \"1\"; \"2\" - 1; \"10\" > \"2\"; void 1; (1, 2);")
            .expect("emit should work");
        wasmparser::Validator::new()
            .validate_all(&artifact.bytes)
            .expect("module should validate");
    }

    #[test]
    fn supports_heap_coercion_core() {
        let artifact = emit_script(
            "\"a\" + {}; let o = { valueOf() { return 2; } }; o + 1; [1, 2] + 3; ({}) == 1; [2] < 3; function f() { return arguments + \"\"; } f(1, 2);",
        )
        .expect("heap coercion should emit");
        wasmparser::Validator::new()
            .validate_all(&artifact.bytes)
            .expect("module should validate");
    }

    #[test]
    fn supports_dynamic_value_plus_proven_string() {
        let artifact = emit_script(
            "function choose(flag) { if (flag) return 1; return {}; } function format(message) { return message + \" suffix\"; } format(choose(true));",
        )
        .expect("dynamic plus string should emit");
        wasmparser::Validator::new()
            .validate_all(&artifact.bytes)
            .expect("module should validate");
    }

    #[test]
    fn supports_noop_host_gc_builtin() {
        let artifact = emit_script("if (typeof gc === \"function\") { gc(); }")
            .expect("gc host builtin should emit");
        wasmparser::Validator::new()
            .validate_all(&artifact.bytes)
            .expect("module should validate");
    }

    #[test]
    fn unsupported_heap_coercion_script_returns_precise_error() {
        let err = emit_script("let o = { valueOf() { return {}; } }; o + 1;")
            .expect_err("heap hook returning heap should fail");
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
