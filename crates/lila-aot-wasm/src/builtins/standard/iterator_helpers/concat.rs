use super::*;

impl FunctionBuilder<'_> {
    pub(in crate::builtins::standard) fn emit_native_iterator_concat_create(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let arguments = schema.reserve_gc_local(f).initialize(
            self.body_entry_locals()
                .map(|entry| entry.arguments())
                .ok_or_else(|| EmitError::unsupported("missing actual concat arguments"))?
                .load(schema, f),
            f,
        );
        let count = schema.reserve_i32_local(f);
        schema
            .array_type::<ValueArray>()
            .length(&arguments, schema, f);
        count.store(f);
        let index = schema.reserve_i32_local(f);
        f.instruction(&Instruction::I32Const(0));
        index.store(f);
        let source = schema.reserve_value_local(f);
        let method = schema.reserve_value_local(f);
        let undefined = schema.reserve_value_local(f);
        undefined.set_undefined(f);
        let output = schema.reserve_completion(f);
        output.initialize(f);
        let pending = schema.reserve_completion(f);
        pending.initialize(f);
        let stored = schema.reserve_gc_local(f).initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(&undefined, f),
            f,
        );
        let seed = schema.reserve_gc_local(f).initialize(
            schema.struct_type::<IteratorConcatEntry>().construct(
                (
                    GcOperand::reference(&stored, schema),
                    GcOperand::reference(&stored, schema),
                ),
                f,
            ),
            f,
        );
        let construction = IteratorConcatEntriesConstruction::allocate(schema, &seed, count, f);
        seed.clear(f);
        stored.clear(f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        let completed = self.open_frame(ControlFrameKind::Block, f);
        let again = self.open_frame(ControlFrameKind::Loop, f);
        index.load(f);
        count.load(f);
        f.instruction(&Instruction::I32GeU);
        self.emit_branch_if_to_target(completed, f);
        self.emit_argument_vector_entry_to_value(&arguments, index, &source, f);
        self.emit_is_heap_object_like_tag_i32(source.tag(), f);
        f.instruction(&Instruction::I32Eqz);
        self.emit_helper_type_error_if(
            RuntimeErrorMessage::ITERATOR_CONCAT_ARGUMENTS_MUST_BE_OBJECTS,
            &output,
            exit,
            f,
        )?;
        let symbol = schema.reserve_gc_local(f).initialize(
            self.emit_well_known_symbol_reference(lila_ir::WellKnownSymbol::Iterator, f)?,
            f,
        );
        let key = PropertyKeyLocals::from_symbol(schema, &symbol, f);
        symbol.clear(f);
        self.emit_object_read(&source, &source, &key, &pending, f)?;
        key.clear(f);
        self.emit_helper_abrupt_exit(&pending, &output, exit, f);
        method.copy_from(pending.value(), f);
        self.emit_is_callable_i32(&method, f)?;
        f.instruction(&Instruction::I32Eqz);
        self.emit_helper_type_error_if(
            RuntimeErrorMessage::ITERATOR_CONCAT_ITERATOR_METHOD_MUST_BE_CALLABLE,
            &output,
            exit,
            f,
        )?;
        let iterable = schema.reserve_gc_local(f).initialize(
            schema.struct_type::<StoredValue>().from_value(&source, f),
            f,
        );
        let open = schema.reserve_gc_local(f).initialize(
            schema.struct_type::<StoredValue>().from_value(&method, f),
            f,
        );
        let entry = schema.reserve_gc_local(f).initialize(
            schema.struct_type::<IteratorConcatEntry>().construct(
                (
                    GcOperand::reference(&iterable, schema),
                    GcOperand::reference(&open, schema),
                ),
                f,
            ),
            f,
        );
        construction.write(index, &entry, schema, f);
        entry.clear(f);
        open.clear(f);
        iterable.clear(f);
        index.load(f);
        f.instruction(&Instruction::I32Const(1));
        f.instruction(&Instruction::I32Add);
        index.store(f);
        self.emit_branch_to_target(again, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        let entries = schema
            .reserve_gc_local(f)
            .initialize(construction.publish(schema, f), f);
        let realm = schema
            .reserve_gc_local(f)
            .initialize(self.emit_current_function_realm(f), f);
        let header = self.emit_helper_header(&realm, f)?;
        let object = schema.reserve_gc_local(f).initialize(
            schema.struct_type::<IteratorConcatHelper>().construct(
                (
                    GcOperand::reference(&header, schema),
                    GcOperand::reference(&entries, schema),
                    GcOperand::null(schema),
                    GcOperand::i64(0),
                    GcOperand::boolean(false),
                    GcOperand::boolean(false),
                    GcOperand::boolean(false),
                    GcOperand::reference(&realm, schema),
                    GcOperand::boolean(false),
                ),
                f,
            ),
            f,
        );
        output.value().set_reference(&object, schema, f);
        output.set_kind(CompletionKind::Normal, f);
        object.clear(f);
        header.clear(f);
        realm.clear(f);
        entries.clear(f);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        self.completion().copy_from(&output, f);
        arguments.clear(f);
        source.clear(f);
        method.clear(f);
        undefined.clear(f);
        pending.clear(f);
        output.clear(f);
        schema.release_i32_local(index, f);
        schema.release_i32_local(count, f);
        Ok(())
    }

    pub(in crate::builtins::standard) fn emit_native_iterator_concat_resume(
        &mut self,
        operation: IteratorHelperOperation,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let receiver = schema.reserve_value_local(f);
        self.compile_this_to_locals(&receiver, f)?;
        let source = schema.reserve_value_local(f);
        let method = schema.reserve_value_local(f);
        let value = schema.reserve_value_local(f);
        let output = schema.reserve_completion(f);
        output.initialize(f);
        let pending = schema.reserve_completion(f);
        pending.initialize(f);
        let flag = schema.reserve_i32_local(f);
        let done = schema.reserve_i32_local(f);
        let index = schema.reserve_i64_local(f);
        let count = schema.reserve_i32_local(f);
        let position = schema.reserve_i32_local(f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        HelperKind::Concat.test(&receiver, schema, f);
        f.instruction(&Instruction::I32Eqz);
        self.emit_helper_type_error_if(
            RuntimeErrorMessage::ITERATOR_CONCAT_HELPER_CALLED_ON_INCOMPATIBLE_RECEIVER,
            &output,
            exit,
            f,
        )?;
        let object = schema.reserve_gc_local(f).initialize(
            receiver.cast_reference::<IteratorConcatHelper>(schema, f),
            f,
        );
        let state = HelperState::Concat(
            schema
                .reserve_gc_local(f)
                .initialize(object.load(schema, f), f),
        );
        state.read_flag(HelperFlag::Executing, flag, schema, f);
        flag.load(f);
        self.emit_helper_type_error_if(HelperKind::Concat.running_message(), &output, exit, f)?;
        state.read_flag(HelperFlag::Done, flag, schema, f);
        flag.load(f);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_helper_done_result(&output, f)?;
        self.emit_branch_to_target(exit, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let stop = self.open_frame(ControlFrameKind::Block, f);
        match operation {
            IteratorHelperOperation::Return => {
                state.read_flag(HelperFlag::Started, flag, schema, f);
                flag.load(f);
                self.open_frame(ControlFrameKind::If, f);
                state.write_flag(HelperFlag::Executing, true, schema, f);
                f.instruction(&Instruction::Else);
                state.write_flag(HelperFlag::Done, true, schema, f);
                self.pop_control(ControlFrameKind::If);
                f.instruction(&Instruction::End);
                schema
                    .field(IteratorConcatHelperSchema::ACTIVE)
                    .read(&object, schema, f)
                    .store(flag, f);
                flag.load(f);
                self.open_frame(ControlFrameKind::If, f);
                let current = schema.reserve_gc_local(f).initialize(
                    schema
                        .field(IteratorConcatHelperSchema::CURRENT)
                        .read(&object, schema, f)
                        .reference()
                        .require_non_null(f),
                    f,
                );
                pending.initialize(f);
                self.emit_helper_close_record(&current, &pending, &output, f)?;
                current.clear(f);
                self.emit_helper_abrupt_exit(&output, &pending, stop, f);
                self.pop_control(ControlFrameKind::If);
                f.instruction(&Instruction::End);
                state.write_flag(HelperFlag::Done, true, schema, f);
                self.emit_helper_done_result(&output, f)?;
            }
            IteratorHelperOperation::Next => {
                state.write_flag(HelperFlag::Started, true, schema, f);
                state.write_flag(HelperFlag::Executing, true, schema, f);
                let entries = schema.reserve_gc_local(f).initialize(
                    schema
                        .field(IteratorConcatHelperSchema::ENTRIES)
                        .read(&object, schema, f)
                        .reference(),
                    f,
                );
                schema
                    .array_type::<IteratorConcatEntries>()
                    .length(&entries, schema, f);
                count.store(f);
                let again = self.open_frame(ControlFrameKind::Loop, f);
                schema
                    .field(IteratorConcatHelperSchema::ACTIVE)
                    .read(&object, schema, f)
                    .store(flag, f);
                flag.load(f);
                self.open_frame(ControlFrameKind::If, f);
                let current = schema.reserve_gc_local(f).initialize(
                    schema
                        .field(IteratorConcatHelperSchema::CURRENT)
                        .read(&object, schema, f)
                        .reference()
                        .require_non_null(f),
                    f,
                );
                self.emit_helper_step(&current, done, Some(&value), &pending, f)?;
                self.emit_helper_abrupt_exit(&pending, &output, stop, f);
                done.load(f);
                self.open_frame(ControlFrameKind::If, f);
                schema.field(IteratorConcatHelperSchema::ACTIVE).write(
                    &object,
                    GcOperand::boolean(false),
                    schema,
                    f,
                );
                schema.field(IteratorConcatHelperSchema::CURRENT).write(
                    &object,
                    GcOperand::null(schema),
                    schema,
                    f,
                );
                f.instruction(&Instruction::Else);
                self.emit_iterator_result_object_from_locals(&value, false, &output, f)?;
                self.emit_branch_to_target(stop, f);
                self.pop_control(ControlFrameKind::If);
                f.instruction(&Instruction::End);
                current.clear(f);
                self.pop_control(ControlFrameKind::If);
                f.instruction(&Instruction::End);
                schema
                    .field(IteratorConcatHelperSchema::NEXT_ENTRY)
                    .read(&object, schema, f)
                    .store_i64(index, f);
                index.load(f);
                count.load(f);
                f.instruction(&Instruction::I64ExtendI32U);
                f.instruction(&Instruction::I64GeU);
                self.open_frame(ControlFrameKind::If, f);
                state.write_flag(HelperFlag::Done, true, schema, f);
                self.emit_helper_done_result(&output, f)?;
                self.emit_branch_to_target(stop, f);
                self.pop_control(ControlFrameKind::If);
                f.instruction(&Instruction::End);
                index.load(f);
                f.instruction(&Instruction::I32WrapI64);
                position.store(f);
                let entry = schema.reserve_gc_local(f).initialize(
                    schema
                        .array_type::<IteratorConcatEntries>()
                        .read(&entries, position, schema, f)
                        .reference(),
                    f,
                );
                let iterable = schema.reserve_gc_local(f).initialize(
                    schema
                        .field(IteratorConcatEntrySchema::ITERABLE)
                        .read(&entry, schema, f)
                        .reference(),
                    f,
                );
                let open = schema.reserve_gc_local(f).initialize(
                    schema
                        .field(IteratorConcatEntrySchema::METHOD)
                        .read(&entry, schema, f)
                        .reference(),
                    f,
                );
                schema
                    .struct_type::<StoredValue>()
                    .read_into(&iterable, &source, schema, f);
                schema
                    .struct_type::<StoredValue>()
                    .read_into(&open, &method, schema, f);
                let argv = self.emit_pre_evaluated_arg_vector(&[], f);
                self.emit_function_or_proxy_call_with_argv(&method, &source, &argv, &pending, f)?;
                argv.clear(f);
                self.emit_helper_abrupt_exit(&pending, &output, stop, f);
                self.emit_helper_require_object(pending.value(), &output, stop, f)?;
                let current = self
                    .emit_helper_direct_record(pending.value(), &output, stop, f)?
                    .into_record();
                schema.field(IteratorConcatHelperSchema::CURRENT).write(
                    &object,
                    GcOperand::nullable_reference(&current, schema),
                    schema,
                    f,
                );
                schema.field(IteratorConcatHelperSchema::ACTIVE).write(
                    &object,
                    GcOperand::boolean(true),
                    schema,
                    f,
                );
                self.emit_increment_local(index, 1, f);
                schema.field(IteratorConcatHelperSchema::NEXT_ENTRY).write(
                    &object,
                    GcOperand::i64_local(index),
                    schema,
                    f,
                );
                current.clear(f);
                open.clear(f);
                iterable.clear(f);
                entry.clear(f);
                self.emit_branch_to_target(again, f);
                self.pop_control(ControlFrameKind::Loop);
                f.instruction(&Instruction::End);
                entries.clear(f);
            }
        }
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        state.write_flag(HelperFlag::Executing, false, schema, f);
        output.kind().load(f);
        f.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        state.write_flag(HelperFlag::Done, true, schema, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        state.read_flag(HelperFlag::Done, flag, schema, f);
        flag.load(f);
        self.open_frame(ControlFrameKind::If, f);
        schema.field(IteratorConcatHelperSchema::ACTIVE).write(
            &object,
            GcOperand::boolean(false),
            schema,
            f,
        );
        schema.field(IteratorConcatHelperSchema::CURRENT).write(
            &object,
            GcOperand::null(schema),
            schema,
            f,
        );
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        state.clear(f);
        object.clear(f);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        self.completion().copy_from(&output, f);
        output.clear(f);
        pending.clear(f);
        receiver.clear(f);
        source.clear(f);
        method.clear(f);
        value.clear(f);
        schema.release_i32_local(flag, f);
        schema.release_i32_local(done, f);
        schema.release_i32_local(count, f);
        schema.release_i32_local(position, f);
        schema.release_i64_local(index, f);
        Ok(())
    }
}
