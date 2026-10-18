//! Compile proven empty direct eval to a shared, stateless Wasm completion kernel.
use super::*;
use crate::{ValidatedRuntimeArtifact, WasmRuntimeValueTag};
use lila_ir::{CompletionKindIr, EmptyRuntimeDirectEval};
use wasm_encoder::{
    CodeSection, ExportKind, ExportSection, Function, FunctionSection, Instruction, Module,
    RefType, TypeSection, ValType,
};

pub(super) const EXPORT: &str = "lila.empty_direct_eval";

impl NativeProgramTemplate {
    /// Parsing, early errors and lowering of the actual source are mandatory
    /// before this proof is available. No source, object, closure, declaration,
    /// environment or caller operand can escape this empty program; its fresh
    /// P allocation is dead and its sole completion code is source-independent.
    pub fn for_empty_direct_eval(
        _proof: &EmptyRuntimeDirectEval<'_>,
        runtime: &ValidatedRuntimeArtifact,
    ) -> Self {
        let mut module = Module::new();
        let mut types = TypeSection::new();
        types.ty().function(
            [],
            [
                ValType::I32,
                ValType::I64,
                ValType::Ref(RefType::EQREF),
                ValType::I32,
                ValType::I32,
                ValType::I32,
            ],
        );
        module.section(&types);
        let mut functions = FunctionSection::new();
        functions.function(0);
        module.section(&functions);
        let mut exports = ExportSection::new();
        exports.export(EXPORT, ExportKind::Func, 0);
        module.section(&exports);
        let mut body = Function::new([]);
        body.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Undefined.tag()));
        body.instruction(&Instruction::I64Const(0));
        body.instruction(&Instruction::RefNull(RefType::EQREF.heap_type));
        body.instruction(&Instruction::I32Const(
            CompletionKindIr::Normal.abi_code() as i32
        ));
        body.instruction(&Instruction::I32Const(0));
        body.instruction(&Instruction::I32Const(2)); // Entered.
        body.instruction(&Instruction::End);
        let mut code = CodeSection::new();
        code.function(&body);
        module.section(&code);
        Self {
            bytes: module.finish().into(),
            runtime: Arc::clone(runtime.runtime()),
            images: Vec::new(),
            factories: Vec::new(),
            literal_operands: Vec::new(),
            payload: None,
            entry: TemplateEntry::EmptyDirectEval,
        }
    }
}

impl NativeProgramTemplate {
    pub fn permits_instance_reuse(&self) -> bool {
        match self.entry {
            TemplateEntry::Program => self
                .payload
                .as_ref()
                .is_some_and(|p| p.permits_instance_reuse()),
            TemplateEntry::EmptyDirectEval => true,
        }
    }
    pub fn empty_direct_eval_export(&self) -> Option<&'static str> {
        match self.entry {
            TemplateEntry::Program => None,
            TemplateEntry::EmptyDirectEval => Some(empty_direct_eval::EXPORT),
        }
    }
}
