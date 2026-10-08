use super::*;
use crate::functions::ArgumentListConstruction;
impl FunctionBuilder<'_> {
    pub(in crate::builtins::standard) fn emit_native_iterator_terminal(
        &mut self,
        kind: IteratorTerminalKind,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        match kind {
            IteratorTerminalKind::ToArray => self.emit_iterator_terminal_toarray(f),
            IteratorTerminalKind::ForEach => self.emit_iterator_terminal_foreach(f),
            IteratorTerminalKind::Every => self.emit_iterator_terminal_every(f),
            IteratorTerminalKind::Some => self.emit_iterator_terminal_some(f),
            IteratorTerminalKind::Find => self.emit_iterator_terminal_find(f),
            IteratorTerminalKind::Reduce => self.emit_iterator_terminal_reduce(f),
        }
    }
    fn emit_iterator_terminal_toarray(&mut self, f: &mut Function) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let receiver = schema.reserve_value_local(f);
        self.compile_this_to_locals(&receiver, f)?;
        let value = schema.reserve_value_local(f);
        let output = schema.reserve_completion(f);
        output.initialize(f);
        let pending = schema.reserve_completion(f);
        pending.initialize(f);
        let done = schema.reserve_i32_local(f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        self.emit_helper_require_object(&receiver, &output, exit, f)?;
        let record = self
            .emit_helper_direct_record(&receiver, &output, exit, f)?
            .into_record();
        let construction = ArgumentListConstruction::new(schema, f);
        let collected = self.open_frame(ControlFrameKind::Block, f);
        let again = self.open_frame(ControlFrameKind::Loop, f);
        self.emit_helper_step(&record, done, Some(&value), &pending, f)?;
        self.emit_helper_abrupt_exit(&pending, &output, exit, f);
        done.load(f);
        self.emit_branch_if_to_target(collected, f);
        construction.append(&value, schema, f);
        self.emit_branch_to_target(again, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        let list = construction.finish(self, f);
        let array = self.emit_array_from_argument_list(&list, f)?;
        output.value().set_reference(&array, schema, f);
        output.set_kind(CompletionKind::Normal, f);
        array.clear(f);
        list.clear(f);
        record.clear(f);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        self.completion().copy_from(&output, f);
        output.clear(f);
        pending.clear(f);
        value.clear(f);
        receiver.clear(f);
        schema.release_i32_local(done, f);
        Ok(())
    }
    fn emit_iterator_terminal_foreach(&mut self, f: &mut Function) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let receiver = schema.reserve_value_local(f);
        self.compile_this_to_locals(&receiver, f)?;
        let callback = schema.reserve_value_local(f);
        self.emit_builtin_arg_to_value(0, &callback, f);
        let value = schema.reserve_value_local(f);
        let index = schema.reserve_value_local(f);
        let output = schema.reserve_completion(f);
        output.initialize(f);
        let pending = schema.reserve_completion(f);
        pending.initialize(f);
        let done = schema.reserve_i32_local(f);
        let counter = schema.reserve_f64_local(f);
        f.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        counter.store(f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        self.emit_helper_require_object(&receiver, &output, exit, f)?;
        self.emit_helper_callback_admission(
            &receiver,
            &callback,
            RuntimeErrorMessage::ITERATOR_PROTOTYPE_FOREACH_CALLBACK_MUST_BE_CALLABLE,
            &output,
            exit,
            f,
        )?;
        let record = self
            .emit_helper_direct_record(&receiver, &output, exit, f)?
            .into_record();
        let finished = self.open_frame(ControlFrameKind::Block, f);
        let again = self.open_frame(ControlFrameKind::Loop, f);
        self.emit_helper_step(&record, done, Some(&value), &pending, f)?;
        self.emit_helper_abrupt_exit(&pending, &output, exit, f);
        done.load(f);
        self.emit_branch_if_to_target(finished, f);
        counter.load(f);
        f.instruction(&Instruction::I64ReinterpretF64);
        index.scalar().store(f);
        index.set_number(index.scalar(), f);
        self.emit_helper_call(&callback, &[&value, &index], &pending, f)?;
        pending.kind().load(f);
        f.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_helper_close_record(&record, &pending, &output, f)?;
        self.emit_branch_to_target(exit, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        counter.load(f);
        f.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
        f.instruction(&Instruction::F64Add);
        counter.store(f);
        self.emit_branch_to_target(again, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        value.set_undefined(f);
        output.set_normal(&value, f);
        record.clear(f);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        self.completion().copy_from(&output, f);
        schema.release_f64_local(counter, f);
        schema.release_i32_local(done, f);
        output.clear(f);
        pending.clear(f);
        index.clear(f);
        value.clear(f);
        callback.clear(f);
        receiver.clear(f);
        Ok(())
    }
    fn emit_iterator_terminal_every(&mut self, f: &mut Function) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let receiver = schema.reserve_value_local(f);
        self.compile_this_to_locals(&receiver, f)?;
        let callback = schema.reserve_value_local(f);
        self.emit_builtin_arg_to_value(0, &callback, f);
        let value = schema.reserve_value_local(f);
        let index = schema.reserve_value_local(f);
        let output = schema.reserve_completion(f);
        output.initialize(f);
        let pending = schema.reserve_completion(f);
        pending.initialize(f);
        let done = schema.reserve_i32_local(f);
        let counter = schema.reserve_f64_local(f);
        f.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        counter.store(f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        self.emit_helper_require_object(&receiver, &output, exit, f)?;
        self.emit_helper_callback_admission(
            &receiver,
            &callback,
            RuntimeErrorMessage::ITERATOR_PROTOTYPE_EVERY_CALLBACK_MUST_BE_CALLABLE,
            &output,
            exit,
            f,
        )?;
        let record = self
            .emit_helper_direct_record(&receiver, &output, exit, f)?
            .into_record();
        let finished = self.open_frame(ControlFrameKind::Block, f);
        let again = self.open_frame(ControlFrameKind::Loop, f);
        self.emit_helper_step(&record, done, Some(&value), &pending, f)?;
        self.emit_helper_abrupt_exit(&pending, &output, exit, f);
        done.load(f);
        self.emit_branch_if_to_target(finished, f);
        counter.load(f);
        f.instruction(&Instruction::I64ReinterpretF64);
        index.scalar().store(f);
        index.set_number(index.scalar(), f);
        self.emit_helper_call(&callback, &[&value, &index], &pending, f)?;
        pending.kind().load(f);
        f.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_helper_close_record(&record, &pending, &output, f)?;
        self.emit_branch_to_target(exit, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.compile_truthy_tagged_i32(pending.value(), f)?;
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        value.set_scalar(ScalarValue::Boolean(false), f);
        pending.set_normal(&value, f);
        self.emit_helper_close_record(&record, &pending, &output, f)?;
        self.emit_branch_to_target(exit, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        counter.load(f);
        f.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
        f.instruction(&Instruction::F64Add);
        counter.store(f);
        self.emit_branch_to_target(again, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        value.set_scalar(ScalarValue::Boolean(true), f);
        output.set_normal(&value, f);
        record.clear(f);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        self.completion().copy_from(&output, f);
        schema.release_f64_local(counter, f);
        schema.release_i32_local(done, f);
        output.clear(f);
        pending.clear(f);
        index.clear(f);
        value.clear(f);
        callback.clear(f);
        receiver.clear(f);
        Ok(())
    }
    fn emit_iterator_terminal_some(&mut self, f: &mut Function) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let receiver = schema.reserve_value_local(f);
        self.compile_this_to_locals(&receiver, f)?;
        let callback = schema.reserve_value_local(f);
        self.emit_builtin_arg_to_value(0, &callback, f);
        let value = schema.reserve_value_local(f);
        let index = schema.reserve_value_local(f);
        let output = schema.reserve_completion(f);
        output.initialize(f);
        let pending = schema.reserve_completion(f);
        pending.initialize(f);
        let done = schema.reserve_i32_local(f);
        let counter = schema.reserve_f64_local(f);
        f.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        counter.store(f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        self.emit_helper_require_object(&receiver, &output, exit, f)?;
        self.emit_helper_callback_admission(
            &receiver,
            &callback,
            RuntimeErrorMessage::ITERATOR_PROTOTYPE_SOME_CALLBACK_MUST_BE_CALLABLE,
            &output,
            exit,
            f,
        )?;
        let record = self
            .emit_helper_direct_record(&receiver, &output, exit, f)?
            .into_record();
        let finished = self.open_frame(ControlFrameKind::Block, f);
        let again = self.open_frame(ControlFrameKind::Loop, f);
        self.emit_helper_step(&record, done, Some(&value), &pending, f)?;
        self.emit_helper_abrupt_exit(&pending, &output, exit, f);
        done.load(f);
        self.emit_branch_if_to_target(finished, f);
        counter.load(f);
        f.instruction(&Instruction::I64ReinterpretF64);
        index.scalar().store(f);
        index.set_number(index.scalar(), f);
        self.emit_helper_call(&callback, &[&value, &index], &pending, f)?;
        pending.kind().load(f);
        f.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_helper_close_record(&record, &pending, &output, f)?;
        self.emit_branch_to_target(exit, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.compile_truthy_tagged_i32(pending.value(), f)?;
        self.open_frame(ControlFrameKind::If, f);
        value.set_scalar(ScalarValue::Boolean(true), f);
        pending.set_normal(&value, f);
        self.emit_helper_close_record(&record, &pending, &output, f)?;
        self.emit_branch_to_target(exit, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        counter.load(f);
        f.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
        f.instruction(&Instruction::F64Add);
        counter.store(f);
        self.emit_branch_to_target(again, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        value.set_scalar(ScalarValue::Boolean(false), f);
        output.set_normal(&value, f);
        record.clear(f);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        self.completion().copy_from(&output, f);
        schema.release_f64_local(counter, f);
        schema.release_i32_local(done, f);
        output.clear(f);
        pending.clear(f);
        index.clear(f);
        value.clear(f);
        callback.clear(f);
        receiver.clear(f);
        Ok(())
    }
    fn emit_iterator_terminal_find(&mut self, f: &mut Function) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let receiver = schema.reserve_value_local(f);
        self.compile_this_to_locals(&receiver, f)?;
        let callback = schema.reserve_value_local(f);
        self.emit_builtin_arg_to_value(0, &callback, f);
        let value = schema.reserve_value_local(f);
        let index = schema.reserve_value_local(f);
        let output = schema.reserve_completion(f);
        output.initialize(f);
        let pending = schema.reserve_completion(f);
        pending.initialize(f);
        let done = schema.reserve_i32_local(f);
        let counter = schema.reserve_f64_local(f);
        f.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        counter.store(f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        self.emit_helper_require_object(&receiver, &output, exit, f)?;
        self.emit_helper_callback_admission(
            &receiver,
            &callback,
            RuntimeErrorMessage::ITERATOR_PROTOTYPE_FIND_CALLBACK_MUST_BE_CALLABLE,
            &output,
            exit,
            f,
        )?;
        let record = self
            .emit_helper_direct_record(&receiver, &output, exit, f)?
            .into_record();
        let finished = self.open_frame(ControlFrameKind::Block, f);
        let again = self.open_frame(ControlFrameKind::Loop, f);
        self.emit_helper_step(&record, done, Some(&value), &pending, f)?;
        self.emit_helper_abrupt_exit(&pending, &output, exit, f);
        done.load(f);
        self.emit_branch_if_to_target(finished, f);
        counter.load(f);
        f.instruction(&Instruction::I64ReinterpretF64);
        index.scalar().store(f);
        index.set_number(index.scalar(), f);
        self.emit_helper_call(&callback, &[&value, &index], &pending, f)?;
        pending.kind().load(f);
        f.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_helper_close_record(&record, &pending, &output, f)?;
        self.emit_branch_to_target(exit, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.compile_truthy_tagged_i32(pending.value(), f)?;
        self.open_frame(ControlFrameKind::If, f);
        pending.set_normal(&value, f);
        self.emit_helper_close_record(&record, &pending, &output, f)?;
        self.emit_branch_to_target(exit, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        counter.load(f);
        f.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
        f.instruction(&Instruction::F64Add);
        counter.store(f);
        self.emit_branch_to_target(again, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        value.set_undefined(f);
        output.set_normal(&value, f);
        record.clear(f);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        self.completion().copy_from(&output, f);
        schema.release_f64_local(counter, f);
        schema.release_i32_local(done, f);
        output.clear(f);
        pending.clear(f);
        index.clear(f);
        value.clear(f);
        callback.clear(f);
        receiver.clear(f);
        Ok(())
    }
    fn emit_iterator_terminal_reduce(&mut self, f: &mut Function) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let receiver = schema.reserve_value_local(f);
        self.compile_this_to_locals(&receiver, f)?;
        let callback = schema.reserve_value_local(f);
        self.emit_builtin_arg_to_value(0, &callback, f);
        let value = schema.reserve_value_local(f);
        let index = schema.reserve_value_local(f);
        let output = schema.reserve_completion(f);
        output.initialize(f);
        let pending = schema.reserve_completion(f);
        pending.initialize(f);
        let done = schema.reserve_i32_local(f);
        let counter = schema.reserve_f64_local(f);
        f.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        counter.store(f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        self.emit_helper_require_object(&receiver, &output, exit, f)?;
        self.emit_helper_callback_admission(
            &receiver,
            &callback,
            RuntimeErrorMessage::ITERATOR_PROTOTYPE_REDUCE_REDUCER_MUST_BE_CALLABLE,
            &output,
            exit,
            f,
        )?;
        let record = self
            .emit_helper_direct_record(&receiver, &output, exit, f)?
            .into_record();
        let accumulator = schema.reserve_value_local(f);
        self.emit_builtin_arg_to_value(1, &accumulator, f);
        self.body_entry_locals()
            .map(|entry| entry.argument_count())
            .ok_or_else(|| EmitError::unsupported("missing actual reducer argument count"))?
            .load(f);
        f.instruction(&Instruction::I64Const(2));
        f.instruction(&Instruction::I64LtU);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_helper_step(&record, done, Some(&accumulator), &pending, f)?;
        self.emit_helper_abrupt_exit(&pending, &output, exit, f);
        done.load(f);
        self.emit_helper_type_error_if(
            RuntimeErrorMessage::ITERATOR_PROTOTYPE_REDUCE_OF_EMPTY_ITERATOR_WITH_NO_INITIAL_VALUE,
            &output,
            exit,
            f,
        )?;
        f.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
        counter.store(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let finished = self.open_frame(ControlFrameKind::Block, f);
        let again = self.open_frame(ControlFrameKind::Loop, f);
        self.emit_helper_step(&record, done, Some(&value), &pending, f)?;
        self.emit_helper_abrupt_exit(&pending, &output, exit, f);
        done.load(f);
        self.emit_branch_if_to_target(finished, f);
        counter.load(f);
        f.instruction(&Instruction::I64ReinterpretF64);
        index.scalar().store(f);
        index.set_number(index.scalar(), f);
        self.emit_helper_call(&callback, &[&accumulator, &value, &index], &pending, f)?;
        pending.kind().load(f);
        f.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_helper_close_record(&record, &pending, &output, f)?;
        self.emit_branch_to_target(exit, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        accumulator.copy_from(pending.value(), f);
        counter.load(f);
        f.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
        f.instruction(&Instruction::F64Add);
        counter.store(f);
        self.emit_branch_to_target(again, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        output.set_normal(&accumulator, f);
        accumulator.clear(f);
        record.clear(f);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        self.completion().copy_from(&output, f);
        schema.release_f64_local(counter, f);
        schema.release_i32_local(done, f);
        output.clear(f);
        pending.clear(f);
        index.clear(f);
        value.clear(f);
        callback.clear(f);
        receiver.clear(f);
        Ok(())
    }
}
