use super::*;

impl FunctionBuilder<'_> {
    pub(in crate::builtins::standard) fn emit_buffered_iterator_resume(
        &mut self,
        operation: IteratorHelperOperation,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let receiver = s.reserve_value_local(f);
        self.compile_this_to_locals(&receiver, f)?;
        let output = s.reserve_completion(f);
        output.initialize(f);
        let pending = s.reserve_completion(f);
        pending.initialize(f);
        let value = s.reserve_value_local(f);
        let flag = s.reserve_i32_local(f);
        let done = s.reserve_i32_local(f);
        let count = s.reserve_i64_local(f);
        let size = s.reserve_i64_local(f);
        let mode = GcI32DomainLocal::new(s, IteratorBufferMode::Chunks, f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        HelperKind::Buffered.test(&receiver, s, f);
        f.instruction(&Instruction::I32Eqz);
        self.emit_helper_type_error_if(
            RuntimeErrorMessage::ITERATOR_HELPER_CALLED_ON_INCOMPATIBLE_RECEIVER,
            &output,
            exit,
            f,
        )?;
        let object = s
            .reserve_gc_local(f)
            .initialize(receiver.cast_reference::<IteratorBufferedHelper>(s, f), f);
        let record = s.reserve_gc_local(f).initialize(
            s.field(IteratorBufferedHelperSchema::RECORD)
                .read(&object, s, f)
                .reference(),
            f,
        );
        let buffer = s.reserve_gc_local(f).initialize(
            s.field(IteratorBufferedHelperSchema::BUFFER)
                .read(&object, s, f)
                .reference(),
            f,
        );
        s.field(IteratorBufferedHelperSchema::SIZE)
            .read(&object, s, f)
            .store_i64(size, f);
        s.field(IteratorBufferedHelperSchema::MODE)
            .read(&object, s, f)
            .store_domain(&mode, f);
        let state = HelperState::Buffered(s.reserve_gc_local(f).initialize(object.load(s, f), f));
        state.read_flag(HelperFlag::Executing, flag, s, f);
        flag.load(f);
        self.emit_helper_type_error_if(HelperKind::Buffered.running_message(), &output, exit, f)?;
        state.read_flag(HelperFlag::Done, flag, s, f);
        flag.load(f);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_helper_done_result(&output, f)?;
        self.emit_branch_to_target(exit, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let stop = self.open_frame(ControlFrameKind::Block, f);
        match operation {
            IteratorHelperOperation::Return => {
                state.read_flag(HelperFlag::Started, flag, s, f);
                flag.load(f);
                self.open_frame(ControlFrameKind::If, f);
                state.write_flag(HelperFlag::Executing, true, s, f);
                f.instruction(&Instruction::Else);
                state.write_flag(HelperFlag::Done, true, s, f);
                self.pop_control(ControlFrameKind::If);
                f.instruction(&Instruction::End);
                self.emit_helper_close_record(&record, &pending, &output, f)?;
                state.write_flag(HelperFlag::Done, true, s, f);
                self.emit_buffer_clear(&buffer, f);
                self.emit_helper_abrupt_exit(&output, &pending, stop, f);
                self.emit_helper_done_result(&output, f)?;
            }
            IteratorHelperOperation::Next => {
                state.write_flag(HelperFlag::Executing, true, s, f);
                state.write_flag(HelperFlag::Started, true, s, f);
                let again = self.open_frame(ControlFrameKind::Loop, f);
                self.emit_helper_step(&record, done, Some(&value), &pending, f)?;
                self.emit_helper_abrupt_exit(&pending, &output, stop, f);
                s.field(IteratorBufferSchema::COUNT)
                    .read(&buffer, s, f)
                    .store_i64(count, f);
                done.load(f);
                self.open_frame(ControlFrameKind::If, f);
                state.write_flag(HelperFlag::Done, true, s, f);
                mode.load(f);
                f.instruction(&Instruction::I32Const(IteratorBufferMode::Chunks.encode()));
                f.instruction(&Instruction::I32Eq);
                mode.load(f);
                f.instruction(&Instruction::I32Const(
                    IteratorBufferMode::WindowsAllowPartial.encode(),
                ));
                f.instruction(&Instruction::I32Eq);
                count.load(f);
                size.load(f);
                f.instruction(&Instruction::I64LtU);
                f.instruction(&Instruction::I32And);
                f.instruction(&Instruction::I32Or);
                count.load(f);
                f.instruction(&Instruction::I64Eqz);
                f.instruction(&Instruction::I32Eqz);
                f.instruction(&Instruction::I32And);
                self.open_frame(ControlFrameKind::If, f);
                let array = self.emit_buffer_array(&buffer, f)?;
                value.set_reference(&array, s, f);
                self.emit_iterator_result_object_from_locals(&value, false, &output, f)?;
                array.clear(f);
                f.instruction(&Instruction::Else);
                self.emit_helper_done_result(&output, f)?;
                self.pop_control(ControlFrameKind::If);
                f.instruction(&Instruction::End);
                self.emit_buffer_clear(&buffer, f);
                self.emit_branch_to_target(stop, f);
                self.pop_control(ControlFrameKind::If);
                f.instruction(&Instruction::End);
                // Windows keep a full buffer until a successful next step.
                // Exhaustion therefore cannot yield a partial suffix.
                mode.load(f);
                f.instruction(&Instruction::I32Const(IteratorBufferMode::Chunks.encode()));
                f.instruction(&Instruction::I32Ne);
                count.load(f);
                size.load(f);
                f.instruction(&Instruction::I64Eq);
                f.instruction(&Instruction::I32And);
                self.open_frame(ControlFrameKind::If, f);
                self.emit_buffer_drop_first(&buffer, f);
                self.pop_control(ControlFrameKind::If);
                f.instruction(&Instruction::End);
                self.emit_buffer_append(&buffer, &value, f);
                s.field(IteratorBufferSchema::COUNT)
                    .read(&buffer, s, f)
                    .store_i64(count, f);
                count.load(f);
                size.load(f);
                f.instruction(&Instruction::I64Eq);
                self.open_frame(ControlFrameKind::If, f);
                let array = self.emit_buffer_array(&buffer, f)?;
                value.set_reference(&array, s, f);
                self.emit_iterator_result_object_from_locals(&value, false, &output, f)?;
                array.clear(f);
                mode.load(f);
                f.instruction(&Instruction::I32Const(IteratorBufferMode::Chunks.encode()));
                f.instruction(&Instruction::I32Eq);
                self.open_frame(ControlFrameKind::If, f);
                self.emit_buffer_clear(&buffer, f);
                self.pop_control(ControlFrameKind::If);
                f.instruction(&Instruction::End);
                self.emit_branch_to_target(stop, f);
                self.pop_control(ControlFrameKind::If);
                f.instruction(&Instruction::End);
                self.emit_branch_to_target(again, f);
                self.pop_control(ControlFrameKind::Loop);
                f.instruction(&Instruction::End);
            }
        }
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        state.write_flag(HelperFlag::Executing, false, s, f);
        output.kind().load(f);
        f.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        state.write_flag(HelperFlag::Done, true, s, f);
        self.emit_buffer_clear(&buffer, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        state.clear(f);
        buffer.clear(f);
        record.clear(f);
        object.clear(f);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        self.completion().copy_from(&output, f);
        mode.clear(s, f);
        s.release_i64_local(size, f);
        s.release_i64_local(count, f);
        s.release_i32_local(done, f);
        s.release_i32_local(flag, f);
        value.clear(f);
        pending.clear(f);
        output.clear(f);
        receiver.clear(f);
        Ok(())
    }
}
