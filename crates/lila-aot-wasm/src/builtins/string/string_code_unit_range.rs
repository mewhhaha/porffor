use super::*;

enum StringRangeOperation {
    Slice,
    Substring,
    Substr,
}

/// Only the normalizer below can construct the completed interval; publishing
/// consumes it so no caller can mutate one bound after selecting its length.
#[must_use]
struct NormalizedStringRange {
    start: I64Local,
    end: I64Local,
}
impl NormalizedStringRange {
    fn materialize(
        self,
        b: &FunctionBuilder<'_>,
        string: &GcLocal<StringValue>,
        f: &mut Function,
    ) -> GcStackReference<StringValue> {
        let result = b.emit_gc_string_slice(string, self.start, self.end, f);
        b.runtime_schema().release_i64_local(self.end, f);
        b.runtime_schema().release_i64_local(self.start, f);
        result
    }
}

impl FunctionBuilder<'_> {
    pub(crate) fn compile_string_slice_range_builtin(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_native_string_range(StringRangeOperation::Slice, f)
    }
    pub(crate) fn compile_string_substring_range_builtin(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_native_string_range(StringRangeOperation::Substring, f)
    }
    pub(crate) fn emit_string_prototype_substr_builtin(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_native_string_range(StringRangeOperation::Substr, f)
    }

    fn emit_native_string_range(
        &mut self,
        operation: StringRangeOperation,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_with_native_string_receiver(f, |b, string, output, exit, f| {
            let s = b.runtime_schema();
            let first = s.reserve_value_local(f);
            let second = s.reserve_value_local(f);
            let pending = s.reserve_completion(f);
            let length = s.reserve_i64_local(f);
            let start = s.reserve_i64_local(f);
            let end = s.reserve_i64_local(f);
            let temporary = s.reserve_i64_local(f);
            b.emit_native_gc_string_length(string, length, f);
            b.emit_builtin_arg_to_value(0, &first, f);
            b.emit_builtin_arg_to_value(1, &second, f);
            b.emit_value_to_number_payload(&first, &pending, f)?;
            b.emit_native_string_abrupt_exit(&pending, output, exit, f);
            match &operation {
                StringRangeOperation::Slice | StringRangeOperation::Substr => b
                    .emit_to_slice_index_clamped_to_string_len(
                        pending.value().scalar(),
                        length,
                        start,
                        f,
                    ),
                StringRangeOperation::Substring => b.emit_to_integer_clamped_to_string_len(
                    pending.value().scalar(),
                    length,
                    start,
                    f,
                ),
            }
            second.tag().load(f);
            f.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Undefined.tag()));
            f.instruction(&Instruction::I32Eq);
            b.open_frame(ControlFrameKind::If, f);
            length.load(f);
            end.store(f);
            f.instruction(&Instruction::Else);
            b.emit_value_to_number_payload(&second, &pending, f)?;
            b.emit_native_string_abrupt_exit(&pending, output, exit, f);
            match &operation {
                StringRangeOperation::Slice => b.emit_to_slice_index_clamped_to_string_len(
                    pending.value().scalar(),
                    length,
                    end,
                    f,
                ),
                StringRangeOperation::Substring => b.emit_to_integer_clamped_to_string_len(
                    pending.value().scalar(),
                    length,
                    end,
                    f,
                ),
                StringRangeOperation::Substr => {
                    b.emit_to_integer_clamped_to_string_len(
                        pending.value().scalar(),
                        length,
                        temporary,
                        f,
                    );
                    start.load(f);
                    temporary.load(f);
                    f.instruction(&Instruction::I64Add);
                    end.store(f);
                    end.load(f);
                    length.load(f);
                    f.instruction(&Instruction::I64GtU);
                    b.open_frame(ControlFrameKind::If, f);
                    length.load(f);
                    end.store(f);
                    b.pop_control(ControlFrameKind::If);
                    f.instruction(&Instruction::End);
                }
            }
            b.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
            match operation {
                StringRangeOperation::Slice | StringRangeOperation::Substr => {
                    end.load(f);
                    start.load(f);
                    f.instruction(&Instruction::I64LtU);
                    b.open_frame(ControlFrameKind::If, f);
                    start.load(f);
                    end.store(f);
                    b.pop_control(ControlFrameKind::If);
                    f.instruction(&Instruction::End);
                }
                StringRangeOperation::Substring => {
                    end.load(f);
                    start.load(f);
                    f.instruction(&Instruction::I64LtU);
                    b.open_frame(ControlFrameKind::If, f);
                    end.load(f);
                    temporary.store(f);
                    start.load(f);
                    end.store(f);
                    temporary.load(f);
                    start.store(f);
                    b.pop_control(ControlFrameKind::If);
                    f.instruction(&Instruction::End);
                }
            }
            let range = NormalizedStringRange { start, end };
            let selected = s
                .reserve_gc_local(f)
                .initialize(range.materialize(b, string, f), f);
            b.emit_native_string_normal_reference(&selected, output, f);
            selected.clear(f);
            s.release_i64_local(temporary, f);
            s.release_i64_local(length, f);
            pending.clear(f);
            second.clear(f);
            first.clear(f);
            Ok(())
        })
    }
}
