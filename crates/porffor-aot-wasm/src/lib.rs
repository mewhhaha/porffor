use std::collections::BTreeMap;

use porffor_ir::{
    ArithmeticBinaryOp, ExprIr, ProgramIr, ScriptIr, StatementIr, TypedExpr, UnaryNumericOp,
    ValueKind,
};
use wasm_encoder::{
    CodeSection, ConstExpr, DataSection, ExportKind, ExportSection, Function, FunctionSection,
    Ieee64, Instruction, MemorySection, MemoryType, Module, TypeSection, ValType,
};

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
    let mut builder = FunctionBuilder::new(script, &string_pool);
    let function = builder.compile()?;

    let mut types = TypeSection::new();
    types.ty().function([], [ValType::I64]);

    let mut functions = FunctionSection::new();
    functions.function(0);

    let mut exports = ExportSection::new();
    exports.export("main", ExportKind::Func, 0);

    let mut code = CodeSection::new();
    code.function(&function);

    let mut module = Module::new();
    module.section(&types);
    module.section(&functions);

    let mut debug_dump = vec![
        "module: js-aot".to_string(),
        "export func: main -> i64".to_string(),
        format!("static result kind: {}", script.result_kind.as_str()),
        format!("locals: {}", builder.local_count()),
    ];

    if !string_pool.bytes.is_empty() {
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
        module.section(&exports);
        module.section(&code);
        module.section(&data);
        debug_dump.push("data segments: 1".to_string());
    } else {
        module.section(&exports);
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
}

impl StringPool {
    fn collect(script: &ScriptIr) -> Self {
        let mut pool = Self::default();
        for statement in &script.statements {
            match statement {
                StatementIr::Lexical { init, .. } => pool.collect_expr(init),
                StatementIr::Expression(expr) => pool.collect_expr(expr),
            }
        }
        pool
    }

    fn collect_expr(&mut self, expr: &TypedExpr) {
        match &expr.expr {
            ExprIr::String(value) => {
                if self.refs.contains_key(value) {
                    return;
                }
                let offset = self.bytes.len() as u32;
                let bytes = value.as_bytes();
                self.bytes.extend_from_slice(bytes);
                self.refs.insert(
                    value.clone(),
                    StringRef {
                        offset,
                        len: bytes.len() as u32,
                    },
                );
            }
            ExprIr::UnaryNumber { expr, .. } | ExprIr::LogicalNot { expr } => {
                self.collect_expr(expr);
            }
            ExprIr::BinaryNumber { lhs, rhs, .. } => {
                self.collect_expr(lhs);
                self.collect_expr(rhs);
            }
            ExprIr::Undefined
            | ExprIr::Null
            | ExprIr::Boolean(_)
            | ExprIr::Number(_)
            | ExprIr::Identifier(_) => {}
        }
    }

    fn payload(&self, value: &str) -> i64 {
        let string = self.refs.get(value).expect("string must exist in pool");
        (((string.offset as u64) << 32) | string.len as u64) as i64
    }
}

struct FunctionBuilder<'a> {
    script: &'a ScriptIr,
    strings: &'a StringPool,
    bindings: BTreeMap<String, u32>,
    binding_types: Vec<ValueKind>,
    result_local: u32,
    scratch_local: u32,
}

impl<'a> FunctionBuilder<'a> {
    fn new(script: &'a ScriptIr, strings: &'a StringPool) -> Self {
        Self {
            script,
            strings,
            bindings: BTreeMap::new(),
            binding_types: Vec::new(),
            result_local: 0,
            scratch_local: 0,
        }
    }

    fn local_count(&self) -> usize {
        self.script
            .statements
            .iter()
            .filter(|statement| matches!(statement, StatementIr::Lexical { .. }))
            .count()
            + 2
    }

    fn compile(&mut self) -> Result<Function, EmitError> {
        let binding_count = self
            .script
            .statements
            .iter()
            .filter(|statement| matches!(statement, StatementIr::Lexical { .. }))
            .count() as u32;

        self.result_local = binding_count;
        self.scratch_local = binding_count + 1;

        let mut function =
            Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, self.local_count()));

        let mut next_binding_local = 0u32;
        for statement in &self.script.statements {
            match statement {
                StatementIr::Lexical {
                    mode: _,
                    name,
                    init,
                } => {
                    self.compile_expr_payload(init, &mut function)?;
                    function.instruction(&Instruction::LocalSet(next_binding_local));
                    self.bindings.insert(name.clone(), next_binding_local);
                    self.binding_types.push(init.kind);
                    next_binding_local += 1;
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(self.result_local));
                }
                StatementIr::Expression(expr) => {
                    self.compile_expr_payload(expr, &mut function)?;
                    function.instruction(&Instruction::LocalSet(self.result_local));
                }
            }
        }

        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::End);
        Ok(function)
    }

    fn compile_expr_payload(
        &self,
        expr: &TypedExpr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match &expr.expr {
            ExprIr::Undefined | ExprIr::Null => {
                function.instruction(&Instruction::I64Const(0));
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
            ExprIr::Identifier(name) => {
                let local = self.bindings.get(name).copied().ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: unbound identifier `{name}`"
                    ))
                })?;
                function.instruction(&Instruction::LocalGet(local));
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
        }
        Ok(())
    }

    fn compile_truthy_i32(
        &self,
        expr: &TypedExpr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match expr.kind {
            ValueKind::Undefined | ValueKind::Null => {
                function.instruction(&Instruction::I32Const(0));
            }
            ValueKind::Boolean => {
                self.compile_expr_payload(expr, function)?;
                function.instruction(&Instruction::I32WrapI64);
            }
            ValueKind::String => {
                self.compile_expr_payload(expr, function)?;
                function.instruction(&Instruction::I64Const(0xFFFF_FFFFu64 as i64));
                function.instruction(&Instruction::I64And);
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::I32Eqz);
            }
            ValueKind::Number => {
                self.compile_expr_payload(expr, function)?;
                function.instruction(&Instruction::LocalSet(self.scratch_local));
                function.instruction(&Instruction::LocalGet(self.scratch_local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                function.instruction(&Instruction::F64Eq);
                function.instruction(&Instruction::LocalGet(self.scratch_local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::LocalGet(self.scratch_local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Ne);
                function.instruction(&Instruction::I32Or);
                function.instruction(&Instruction::I32Eqz);
            }
        }
        Ok(())
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
    fn unsupported_script_returns_precise_error() {
        let err = emit_script("\"a\" + \"b\";").expect_err("string plus should fail");
        assert!(err
            .to_string()
            .contains("unsupported in porffor wasm-aot first slice"));
    }
}
