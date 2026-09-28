use super::provider_wire::{CollatorOperation, CollatorResponseReader, CollatorWireField};
use super::*;

impl FunctionBuilder<'_> {
    pub(crate) fn emit_intl_collator_compare_function(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let collator = self.reserve_temp_local();
        let record = self.reserve_temp_local();
        let left = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let right = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let left_string = self.reserve_temp_local();
        let right_string = self.reserve_temp_local();
        let request = self.reserve_temp_local();
        let response = self.reserve_temp_local();
        let result = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            self.current_env_local,
            HEAP_FUNCTION_BUILTIN_CLOSURE_CONTEXT_OFFSET,
            collator,
            function,
        );
        self.load_i64_to_local_from_offset(
            collator,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            record,
            function,
        );

        self.emit_builtin_arg_to_locals(0, left.payload, left.tag, function);
        self.emit_col_to_string(left, left_string, function)?;
        self.emit_builtin_arg_to_locals(1, right.payload, right.tag, function);
        self.emit_col_to_string(right, right_string, function)?;
        self.emit_col_provider_request(
            CollatorOperation::Compare,
            &[
                CollatorWireField::Configuration(record),
                CollatorWireField::Utf16(left_string),
                CollatorWireField::Utf16(right_string),
            ],
            request,
            function,
        )?;
        self.emit_col_provider_call(CollatorOperation::Compare, request, response, function)?;
        let reader =
            CollatorResponseReader::new(self, response, CollatorOperation::Compare, function)?;
        reader.word(self, result, function);
        function.instruction(&Instruction::LocalGet(result));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(result));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        reader.finish(self, function);
        function.instruction(&Instruction::LocalGet(result));
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(self.result_local));
        self.emit_col_set_const(
            self.result_tag_local,
            ValueKind::Number.tag() as i64,
            function,
        );

        for local in [
            result,
            response,
            request,
            right_string,
            left_string,
            right.tag,
            right.payload,
            left.tag,
            left.payload,
            record,
            collator,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
