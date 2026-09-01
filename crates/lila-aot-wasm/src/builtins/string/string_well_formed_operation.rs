use super::*;

enum StringWellFormedOperation {
    Check,
    Repair,
}

impl FunctionBuilder<'_> {
    pub(crate) fn emit_string_is_well_formed_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        emit(self, StringWellFormedOperation::Check, function)
    }

    pub(crate) fn emit_string_to_well_formed_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        emit(self, StringWellFormedOperation::Repair, function)
    }
}

fn emit(
    builder: &mut FunctionBuilder<'_>,
    operation: StringWellFormedOperation,
    function: &mut Function,
) -> Result<(), EmitError> {
    let receiver_payload_local = builder.this_payload_local.ok_or_else(|| {
        EmitError::unsupported(
            "unsupported in lila wasm-aot first slice: missing String.prototype well-formed receiver",
        )
    })?;
    let receiver_tag_local = builder.this_tag_local.ok_or_else(|| {
        EmitError::unsupported(
            "unsupported in lila wasm-aot first slice: missing String.prototype well-formed receiver",
        )
    })?;
    let string_local = builder.reserve_temp_local();

    builder.compile_nullish_tagged_i32(receiver_tag_local, function)?;
    function.instruction(&Instruction::If(BlockType::Empty));
    builder.emit_throw_current_function_realm_type_error(
        "String.prototype method receiver is null or undefined",
        builder.result_local,
        builder.result_tag_local,
        function,
    )?;
    function.instruction(&Instruction::Else);
    builder.emit_value_to_string_payload(receiver_payload_local, receiver_tag_local, function)?;
    function.instruction(&Instruction::LocalSet(string_local));
    builder.set_completion_kind(CompletionKind::Normal, function);
    builder.emit_return_current_completion_if_throw(function);

    match operation {
        StringWellFormedOperation::Check => {
            builder.emit_string_is_well_formed_payload_from_local(string_local, function)?;
            function.instruction(&Instruction::LocalSet(builder.result_local));
            function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
            function.instruction(&Instruction::LocalSet(builder.result_tag_local));
        }
        StringWellFormedOperation::Repair => {
            builder.emit_string_to_well_formed_payload_from_local(string_local, function)?;
            function.instruction(&Instruction::LocalSet(builder.result_local));
            function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
            function.instruction(&Instruction::LocalSet(builder.result_tag_local));
        }
    }
    function.instruction(&Instruction::End);

    builder.release_temp_local(string_local);
    Ok(())
}
