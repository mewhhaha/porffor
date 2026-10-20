//! Stage-three chunks/windows preserve cached direct iteration and List identity.
use super::*;
mod buffer_queue;
mod buffer_resume;

#[derive(Clone, Copy)]
pub(in crate::builtins::standard) enum BufferedIteratorKind {
    Chunks,
    Windows,
}

impl FunctionBuilder<'_> {
    fn emit_buffer_argument_error_if(
        &mut self,
        range: bool,
        receiver: &ValueLocals,
        pending: &CompletionLocals,
        output: &CompletionLocals,
        exit: ControlTarget,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.open_frame(ControlFrameKind::If, f);
        if range {
            self.emit_throw_current_function_realm_range_error(
                RuntimeErrorMessage::ITERATOR_BUFFER_SIZE_OUT_OF_RANGE,
                pending,
                f,
            )?;
        } else {
            self.emit_throw_current_function_realm_type_error(
                RuntimeErrorMessage::ITERATOR_BUFFER_ARGUMENT_INVALID,
                pending,
                f,
            )?;
        }
        // The provisional record has not read next; IteratorClose preserves the
        // original argument error even if return acquisition or invocation fails.
        self.emit_iterator_close_with_completion(receiver, pending, output, f)?;
        self.emit_branch_to_target(exit, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        Ok(())
    }

    pub(in crate::builtins::standard) fn emit_buffered_iterator_create(
        &mut self,
        kind: BufferedIteratorKind,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let receiver = s.reserve_value_local(f);
        self.compile_this_to_locals(&receiver, f)?;
        let input = s.reserve_value_local(f);
        self.emit_builtin_arg_to_value(0, &input, f);
        let output = s.reserve_completion(f);
        output.initialize(f);
        let pending = s.reserve_completion(f);
        pending.initialize(f);
        let size = s.reserve_i64_local(f);
        let number = s.reserve_f64_local(f);
        let mode = GcI32DomainLocal::new(
            s,
            match kind {
                BufferedIteratorKind::Chunks => IteratorBufferMode::Chunks,
                BufferedIteratorKind::Windows => IteratorBufferMode::WindowsOnlyFull,
            },
            f,
        );
        let exit = self.open_frame(ControlFrameKind::Block, f);
        self.emit_helper_require_object(&receiver, &output, exit, f)?;
        input.tag().load(f);
        f.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Number as i32));
        f.instruction(&Instruction::I32Ne);
        self.emit_buffer_argument_error_if(false, &receiver, &pending, &output, exit, f)?;
        input.scalar().load(f);
        f.instruction(&Instruction::F64ReinterpretI64);
        number.store(f);
        number.load(f);
        f.instruction(&Instruction::F64Trunc);
        number.load(f);
        f.instruction(&Instruction::F64Ne);
        number.load(f);
        f.instruction(&Instruction::F64Abs);
        f.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        f.instruction(&Instruction::F64Eq);
        f.instruction(&Instruction::I32Or);
        self.emit_buffer_argument_error_if(false, &receiver, &pending, &output, exit, f)?;
        number.load(f);
        f.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
        f.instruction(&Instruction::F64Lt);
        number.load(f);
        f.instruction(&Instruction::F64Const(Ieee64::from(4294967295.0)));
        f.instruction(&Instruction::F64Gt);
        f.instruction(&Instruction::I32Or);
        self.emit_buffer_argument_error_if(true, &receiver, &pending, &output, exit, f)?;
        number.load(f);
        f.instruction(&Instruction::I64TruncF64U);
        size.store(f);
        if matches!(kind, BufferedIteratorKind::Windows) {
            self.emit_builtin_arg_to_value(1, &input, f);
            input.tag().load(f);
            f.instruction(&Instruction::I32Const(
                WasmRuntimeValueTag::Undefined as i32,
            ));
            f.instruction(&Instruction::I32Ne);
            self.open_frame(ControlFrameKind::If, f);
            input.tag().load(f);
            f.instruction(&Instruction::I32Const(WasmRuntimeValueTag::String as i32));
            f.instruction(&Instruction::I32Ne);
            self.emit_buffer_argument_error_if(false, &receiver, &pending, &output, exit, f)?;
            let text = s
                .reserve_gc_local(f)
                .initialize(input.cast_reference::<StringValue>(s, f), f);
            let matched = s.reserve_i32_local(f);
            f.instruction(&Instruction::I32Const(0));
            matched.store(f);
            for (name, constant) in [
                ("only-full", IteratorBufferMode::WindowsOnlyFull),
                ("allow-partial", IteratorBufferMode::WindowsAllowPartial),
            ] {
                let candidate = s
                    .reserve_gc_local(f)
                    .initialize(self.emit_interned_string_reference(name, f)?, f);
                self.emit_string_payload_equality_i32(&text, &candidate, f);
                self.open_frame(ControlFrameKind::If, f);
                mode.set_constant(constant, f);
                f.instruction(&Instruction::I32Const(1));
                matched.store(f);
                self.pop_control(ControlFrameKind::If);
                f.instruction(&Instruction::End);
                candidate.clear(f);
            }
            matched.load(f);
            f.instruction(&Instruction::I32Eqz);
            self.emit_buffer_argument_error_if(false, &receiver, &pending, &output, exit, f)?;
            s.release_i32_local(matched, f);
            text.clear(f);
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
        }
        let record = self
            .emit_helper_direct_record(&receiver, &output, exit, f)?
            .into_record();
        let realm = s
            .reserve_gc_local(f)
            .initialize(self.emit_current_function_realm(f), f);
        let header = self.emit_helper_header(&realm, f)?;
        let buffer = s.reserve_gc_local(f).initialize(
            s.struct_type::<IteratorBuffer>().construct(
                (GcOperand::null(s), GcOperand::null(s), GcOperand::i64(0)),
                f,
            ),
            f,
        );
        let helper = s.reserve_gc_local(f).initialize(
            s.struct_type::<IteratorBufferedHelper>().construct(
                (
                    GcOperand::reference(&header, s),
                    GcOperand::reference(&record, s),
                    GcOperand::reference(&buffer, s),
                    mode.operand(),
                    GcOperand::i64_local(size),
                    GcOperand::boolean(false),
                    GcOperand::boolean(false),
                    GcOperand::reference(&realm, s),
                    GcOperand::boolean(false),
                ),
                f,
            ),
            f,
        );
        output.value().set_reference(&helper, s, f);
        output.set_kind(CompletionKind::Normal, f);
        helper.clear(f);
        buffer.clear(f);
        header.clear(f);
        realm.clear(f);
        record.clear(f);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        self.completion().copy_from(&output, f);
        mode.clear(s, f);
        s.release_i64_local(size, f);
        s.release_f64_local(number, f);
        pending.clear(f);
        output.clear(f);
        input.clear(f);
        receiver.clear(f);
        Ok(())
    }
}
