//! The radix coercion and validation precede the non-coercing GC formatter.

use super::*;
use crate::gc_types::{CompletionLocals, I64Local, ValueLocals};

struct PreparedBigIntRadix(I64Local);

impl FunctionBuilder<'_> {
    pub(super) fn emit_bigint_radix_string_result(
        &mut self,
        bigint: &ValueLocals,
        output: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let argument = schema.reserve_value_local(function);
        let converted = schema.reserve_completion(function);
        let radix = schema.reserve_i64_local(function);
        function.instruction(&Instruction::I64Const(10));
        radix.store(function);
        self.emit_builtin_arg_to_value(0, &argument, function);
        let exit = self.open_frame(ControlFrameKind::Block, function);
        argument.tag().load(function);
        function.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Undefined.tag()));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_value_to_number_payload(&argument, &converted, function)?;
        output.copy_from(&converted, function);
        converted.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Normal.code() as i32));
        function.instruction(&Instruction::I32Ne);
        self.emit_branch_if_to_target(exit, function);
        converted.value().scalar().load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::I64TruncSatF64S);
        radix.store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        radix.load(function);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64LtS);
        radix.load(function);
        function.instruction(&Instruction::I64Const(36));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_current_function_realm_range_error(
            RuntimeErrorMessage::BIGINT_PROTOTYPE_TOSTRING_RADIX_OUT_OF_RANGE,
            output,
            function,
        )?;
        self.emit_branch_to_target(exit, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let prepared = PreparedBigIntRadix(radix);
        self.emit_bigint_to_radix_string_payload(bigint, prepared.0, output, function)?;
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        schema.release_i64_local(radix, function);
        converted.clear(function);
        argument.clear(function);
        Ok(())
    }
}
