use super::*;

#[must_use = "a completed direct record must be captured or consumed"]
pub(super) struct HelperDirectRecord(GcLocal<IteratorRecord>);
impl HelperDirectRecord {
    pub(super) fn into_record(self) -> GcLocal<IteratorRecord> {
        self.0
    }
}

impl FunctionBuilder<'_> {
    pub(super) fn emit_helper_key(
        &mut self,
        name: &str,
        f: &mut Function,
    ) -> Result<PropertyKeyLocals, EmitError> {
        let schema = self.runtime_schema();
        let text = schema
            .reserve_gc_local(f)
            .initialize(self.emit_interned_string_reference(name, f)?, f);
        let key = PropertyKeyLocals::from_string(schema, &text, f);
        text.clear(f);
        Ok(key)
    }
    pub(super) fn emit_helper_type_error_if(
        &mut self,
        message: RuntimeErrorMessage,
        output: &CompletionLocals,
        exit: ControlTarget,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.open_frame(ControlFrameKind::If, f);
        self.emit_throw_current_function_realm_type_error(message, output, f)?;
        self.emit_branch_to_target(exit, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        Ok(())
    }
    pub(super) fn emit_helper_abrupt_exit(
        &mut self,
        pending: &CompletionLocals,
        output: &CompletionLocals,
        exit: ControlTarget,
        f: &mut Function,
    ) {
        pending.kind().load(f);
        f.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        output.copy_from(pending, f);
        self.emit_branch_to_target(exit, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
    }
    pub(super) fn emit_helper_require_object(
        &mut self,
        value: &ValueLocals,
        output: &CompletionLocals,
        exit: ControlTarget,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_is_heap_object_like_tag_i32(value.tag(), f);
        f.instruction(&Instruction::I32Eqz);
        self.emit_helper_type_error_if(
            RuntimeErrorMessage::ITERATOR_HELPER_CALLED_ON_INCOMPATIBLE_RECEIVER,
            output,
            exit,
            f,
        )
    }
    pub(super) fn emit_helper_direct_record(
        &mut self,
        receiver: &ValueLocals,
        output: &CompletionLocals,
        exit: ControlTarget,
        f: &mut Function,
    ) -> Result<HelperDirectRecord, EmitError> {
        let schema = self.runtime_schema();
        let pending = schema.reserve_completion(f);
        pending.initialize(f);
        let key = self.emit_helper_key("next", f)?;
        self.emit_object_read(receiver, receiver, &key, &pending, f)?;
        key.clear(f);
        self.emit_helper_abrupt_exit(&pending, output, exit, f);
        // GetIteratorDirect caches any next value; callability is deferred.
        let iterator = schema.reserve_gc_local(f).initialize(
            schema.struct_type::<StoredValue>().from_value(receiver, f),
            f,
        );
        let next = schema.reserve_gc_local(f).initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(pending.value(), f),
            f,
        );
        let record = schema.reserve_gc_local(f).initialize(
            schema.struct_type::<IteratorRecord>().construct(
                (
                    GcOperand::reference(&iterator, schema),
                    GcOperand::reference(&next, schema),
                    GcOperand::boolean(false),
                ),
                f,
            ),
            f,
        );
        next.clear(f);
        iterator.clear(f);
        pending.clear(f);
        Ok(HelperDirectRecord(record))
    }
    pub(super) fn emit_helper_close_receiver_on_throw(
        &mut self,
        receiver: &ValueLocals,
        pending: &CompletionLocals,
        output: &CompletionLocals,
        exit: ControlTarget,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        pending.kind().load(f);
        f.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_iterator_close_with_completion(receiver, pending, output, f)?;
        self.emit_branch_to_target(exit, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        Ok(())
    }
    pub(super) fn emit_helper_callback_admission(
        &mut self,
        receiver: &ValueLocals,
        callback: &ValueLocals,
        message: RuntimeErrorMessage,
        output: &CompletionLocals,
        exit: ControlTarget,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let pending = schema.reserve_completion(f);
        pending.initialize(f);
        self.emit_is_callable_i32(callback, f)?;
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_throw_current_function_realm_type_error(message, &pending, f)?;
        self.emit_iterator_close_with_completion(receiver, &pending, output, f)?;
        self.emit_branch_to_target(exit, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        pending.clear(f);
        Ok(())
    }
    pub(super) fn emit_helper_header(
        &mut self,
        realm: &GcLocal<RealmRecord>,
        f: &mut Function,
    ) -> Result<GcLocal<OrdinaryObject>, EmitError> {
        let schema = self.runtime_schema();
        let prototype = schema.reserve_value_local(f);
        self.emit_load_non_array_realm_intrinsic(
            realm,
            NonArrayRealmIntrinsicSlot::IteratorHelperPrototype,
            &prototype,
            f,
        );
        let header = schema.reserve_gc_local(f).initialize(
            self.emit_alloc_plain_object_with_prototype(Some(&prototype), f)?,
            f,
        );
        prototype.clear(f);
        Ok(header)
    }
    pub(super) fn emit_helper_done_result(
        &mut self,
        output: &CompletionLocals,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let value = schema.reserve_value_local(f);
        value.set_undefined(f);
        self.emit_iterator_result_object_from_locals(&value, true, output, f)?;
        value.clear(f);
        Ok(())
    }
    pub(super) fn emit_helper_step(
        &mut self,
        record: &GcLocal<IteratorRecord>,
        done: I32Local,
        value: Option<&ValueLocals>,
        output: &CompletionLocals,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let owned = OwnedSyncIterator::from_helper_record(
            schema
                .reserve_gc_local(f)
                .initialize(record.load(schema, f), f),
        );
        match value {
            Some(value) => {
                self.emit_sync_iterator_step_value_into(&owned, done, value, output, f)?
            }
            None => self.emit_sync_iterator_step_without_value_into(&owned, done, output, f)?,
        }
        owned.clear(f);
        Ok(())
    }
    pub(super) fn emit_helper_close_record(
        &mut self,
        record: &GcLocal<IteratorRecord>,
        pending: &CompletionLocals,
        output: &CompletionLocals,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let owned = OwnedSyncIterator::from_helper_record(
            schema
                .reserve_gc_local(f)
                .initialize(record.load(schema, f), f),
        );
        self.emit_sync_iterator_close(&owned, pending, output, f)?;
        owned.clear(f);
        Ok(())
    }
    pub(super) fn emit_helper_call(
        &mut self,
        callback: &ValueLocals,
        args: &[&ValueLocals],
        output: &CompletionLocals,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let receiver = schema.reserve_value_local(f);
        receiver.set_undefined(f);
        let argv = self.emit_pre_evaluated_arg_vector(args, f);
        self.emit_function_or_proxy_call_with_argv(callback, &receiver, &argv, output, f)?;
        argv.clear(f);
        receiver.clear(f);
        Ok(())
    }

    /// GetIteratorFlattenable(reject-primitives), with no close on a failed
    /// acquisition and no eager callability check of the cached next.
    pub(super) fn emit_helper_flattenable_record(
        &mut self,
        mapped: &ValueLocals,
        output: &CompletionLocals,
        exit: ControlTarget,
        f: &mut Function,
    ) -> Result<HelperDirectRecord, EmitError> {
        let schema = self.runtime_schema();
        let receiver = schema.reserve_value_local(f);
        receiver.copy_from(mapped, f);
        let pending = schema.reserve_completion(f);
        pending.initialize(f);
        let method = schema.reserve_value_local(f);
        self.emit_helper_require_object(mapped, output, exit, f)?;
        let symbol = schema.reserve_gc_local(f).initialize(
            self.emit_well_known_symbol_reference(lila_ir::WellKnownSymbol::Iterator, f)?,
            f,
        );
        let key = PropertyKeyLocals::from_symbol(schema, &symbol, f);
        symbol.clear(f);
        self.emit_object_read(mapped, mapped, &key, &pending, f)?;
        key.clear(f);
        self.emit_helper_abrupt_exit(&pending, output, exit, f);
        method.copy_from(pending.value(), f);
        self.compile_nullish_tagged_i32(method.tag(), f)?;
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_is_callable_i32(&method, f)?;
        f.instruction(&Instruction::I32Eqz);
        self.emit_helper_type_error_if(
            RuntimeErrorMessage::ITERATOR_PROTOTYPE_FLATMAP_INNER_ITERATOR_METHOD_MUST_BE_CALLABLE,
            output,
            exit,
            f,
        )?;
        let argv = self.emit_pre_evaluated_arg_vector(&[], f);
        self.emit_function_or_proxy_call_with_argv(&method, mapped, &argv, &pending, f)?;
        argv.clear(f);
        self.emit_helper_abrupt_exit(&pending, output, exit, f);
        receiver.copy_from(pending.value(), f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.emit_helper_require_object(&receiver, output, exit, f)?;
        let record = self.emit_helper_direct_record(&receiver, output, exit, f)?;
        method.clear(f);
        pending.clear(f);
        receiver.clear(f);
        Ok(record)
    }
}
