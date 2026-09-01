use super::*;

#[must_use]
struct UnitIndexLocal(u32);

#[must_use]
struct UnitLengthLocal(u32);

#[must_use]
struct RangeLengthLocal(u32);

#[must_use = "a normalized String range must be materialized"]
struct MaterializableRangeLocals {
    start: UnitIndexLocal,
    end: UnitIndexLocal,
    length: RangeLengthLocal,
}

enum Method {
    Slice,
    Substring,
}

impl Method {
    fn emit_normalized_index(
        &self,
        builder: &mut FunctionBuilder<'_>,
        number_payload_local: u32,
        string_length: &UnitLengthLocal,
        index: &UnitIndexLocal,
        function: &mut Function,
    ) {
        match self {
            Self::Slice => builder.emit_to_slice_index_clamped_to_string_len(
                number_payload_local,
                string_length.0,
                index.0,
                function,
            ),
            Self::Substring => builder.emit_to_integer_clamped_to_string_len(
                number_payload_local,
                string_length.0,
                index.0,
                function,
            ),
        }
    }

    fn emit_range(
        &self,
        start: &UnitIndexLocal,
        end: &UnitIndexLocal,
        length: &RangeLengthLocal,
        function: &mut Function,
    ) {
        match self {
            Self::Slice => {
                function.instruction(&Instruction::LocalGet(end.0));
                function.instruction(&Instruction::LocalGet(start.0));
                function.instruction(&Instruction::I64LtU);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(length.0));
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::LocalGet(end.0));
                function.instruction(&Instruction::LocalGet(start.0));
                function.instruction(&Instruction::I64Sub);
                function.instruction(&Instruction::LocalSet(length.0));
                function.instruction(&Instruction::End);
            }
            Self::Substring => {
                function.instruction(&Instruction::LocalGet(start.0));
                function.instruction(&Instruction::LocalGet(end.0));
                function.instruction(&Instruction::I64LeU);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(end.0));
                function.instruction(&Instruction::LocalGet(start.0));
                function.instruction(&Instruction::I64Sub);
                function.instruction(&Instruction::LocalSet(length.0));
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::LocalGet(start.0));
                function.instruction(&Instruction::LocalGet(end.0));
                function.instruction(&Instruction::I64Sub);
                function.instruction(&Instruction::LocalSet(length.0));
                function.instruction(&Instruction::LocalGet(end.0));
                function.instruction(&Instruction::LocalSet(start.0));
                function.instruction(&Instruction::End);
            }
        }
    }
}

impl MaterializableRangeLocals {
    fn emit_payload(
        self,
        builder: &mut FunctionBuilder<'_>,
        string_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        builder.emit_utf16_code_unit_range_payload_from_locals(
            string_local,
            self.start.0,
            self.length.0,
            function,
        )?;
        builder.release_temp_local(self.length.0);
        builder.release_temp_local(self.end.0);
        builder.release_temp_local(self.start.0);
        Ok(())
    }
}

pub(super) fn emit_slice(
    builder: &mut FunctionBuilder<'_>,
    function: &mut Function,
) -> Result<(), EmitError> {
    emit(builder, Method::Slice, function)
}

pub(super) fn emit_substring(
    builder: &mut FunctionBuilder<'_>,
    function: &mut Function,
) -> Result<(), EmitError> {
    emit(builder, Method::Substring, function)
}

fn emit(
    builder: &mut FunctionBuilder<'_>,
    method: Method,
    function: &mut Function,
) -> Result<(), EmitError> {
    let receiver_payload_local = builder.this_payload_local.ok_or_else(|| {
        EmitError::unsupported(
            "unsupported in lila wasm-aot first slice: missing String range receiver",
        )
    })?;
    let receiver_tag_local = builder.this_tag_local.ok_or_else(|| {
        EmitError::unsupported(
            "unsupported in lila wasm-aot first slice: missing String range receiver",
        )
    })?;
    let string_local = builder.reserve_temp_local();
    let string_offset_local = builder.reserve_temp_local();
    let string_byte_length_local = builder.reserve_temp_local();
    let string_length = UnitLengthLocal(builder.reserve_temp_local());
    let start_payload_local = builder.reserve_temp_local();
    let start_tag_local = builder.reserve_temp_local();
    let end_payload_local = builder.reserve_temp_local();
    let end_tag_local = builder.reserve_temp_local();
    let start = UnitIndexLocal(builder.reserve_temp_local());
    let end = UnitIndexLocal(builder.reserve_temp_local());
    let range_length = RangeLengthLocal(builder.reserve_temp_local());

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
    builder.emit_unpack_string_payload(
        string_local,
        string_offset_local,
        string_byte_length_local,
        function,
    );
    builder.emit_utf16_code_unit_len_from_utf8_locals(
        string_offset_local,
        string_byte_length_local,
        string_length.0,
        function,
    );

    builder.emit_builtin_arg_to_locals(0, start_payload_local, start_tag_local, function);
    builder.emit_value_to_number_payload(start_tag_local, start_payload_local, function)?;
    function.instruction(&Instruction::LocalSet(start_payload_local));
    builder.set_completion_kind(CompletionKind::Normal, function);
    builder.emit_return_current_completion_if_throw(function);
    method.emit_normalized_index(
        builder,
        start_payload_local,
        &string_length,
        &start,
        function,
    );

    builder.emit_builtin_arg_to_locals(1, end_payload_local, end_tag_local, function);
    function.instruction(&Instruction::LocalGet(end_tag_local));
    function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
    function.instruction(&Instruction::I64Eq);
    function.instruction(&Instruction::If(BlockType::Empty));
    function.instruction(&Instruction::LocalGet(string_length.0));
    function.instruction(&Instruction::LocalSet(end.0));
    function.instruction(&Instruction::Else);
    builder.emit_value_to_number_payload(end_tag_local, end_payload_local, function)?;
    function.instruction(&Instruction::LocalSet(end_payload_local));
    builder.set_completion_kind(CompletionKind::Normal, function);
    builder.emit_return_current_completion_if_throw(function);
    method.emit_normalized_index(builder, end_payload_local, &string_length, &end, function);
    function.instruction(&Instruction::End);

    method.emit_range(&start, &end, &range_length, function);
    MaterializableRangeLocals {
        start,
        end,
        length: range_length,
    }
    .emit_payload(builder, string_local, function)?;
    function.instruction(&Instruction::LocalSet(builder.result_local));
    function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
    function.instruction(&Instruction::LocalSet(builder.result_tag_local));
    builder.set_completion_kind(CompletionKind::Normal, function);
    function.instruction(&Instruction::End);

    builder.release_temp_local(end_tag_local);
    builder.release_temp_local(end_payload_local);
    builder.release_temp_local(start_tag_local);
    builder.release_temp_local(start_payload_local);
    builder.release_temp_local(string_length.0);
    builder.release_temp_local(string_byte_length_local);
    builder.release_temp_local(string_offset_local);
    builder.release_temp_local(string_local);
    Ok(())
}
