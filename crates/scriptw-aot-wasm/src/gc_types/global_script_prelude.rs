//! Program-local harness phase. Its index follows every shared R global.
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GlobalScriptPreludePhase(u32);
impl GlobalScriptPreludePhase {
    pub(super) fn declare(globals: &mut GlobalLedger) -> Self {
        let index = globals.len();
        globals.global(
            GlobalType {
                val_type: ValType::I32,
                mutable: true,
                shared: false,
            },
            &ConstExpr::i32_const(0),
        );
        Self(index)
    }
    pub(crate) fn enter(self, function: &mut Function) {
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::GlobalSet(self.0));
    }
    pub(crate) fn completed(self, function: &mut Function) {
        function.instruction(&Instruction::I32Const(2));
        function.instruction(&Instruction::GlobalSet(self.0));
    }
    pub(crate) fn export(self, exports: &mut wasm_encoder::ExportSection) {
        exports.export(
            crate::GLOBAL_SCRIPT_PRELUDE_PHASE_EXPORT,
            wasm_encoder::ExportKind::Global,
            self.0,
        );
    }
}
