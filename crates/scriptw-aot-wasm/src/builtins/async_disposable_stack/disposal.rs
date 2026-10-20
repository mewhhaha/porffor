use super::*;

impl FunctionBuilder<'_> {
    fn emit_ads_disposal_capture(&self, f: &mut Function) -> GcLocal<AsyncDisposableStackDisposal> {
        let s = self.runtime_schema();
        let capture =
            self.emit_ads_capture(BuiltinClosureCaptureKind::AsyncDisposableStackDisposal, f);
        let state = s.reserve_gc_local(f).initialize(
            s.struct_type::<BuiltinClosureCapture>()
                .field(BuiltinClosureCaptureSchema::ASYNC_DISPOSABLE_STACK_DISPOSAL)
                .read(&capture, s, f)
                .reference()
                .require_non_null(f),
            f,
        );
        capture.clear(f);
        state
    }

    fn emit_ads_record_error(
        &mut self,
        state: &GcLocal<AsyncDisposableStackDisposal>,
        error: &ValueLocals,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let kind = GcI32DomainLocal::new(s, AsyncDisposableStackDisposeCompletionKind::Normal, f);
        let folded = s.reserve_value_local(f);
        folded.copy_from(error, f);
        s.struct_type::<AsyncDisposableStackDisposal>()
            .field(AsyncDisposableStackDisposalSchema::COMPLETION_KIND)
            .read(state, s, f)
            .store_domain(&kind, f);
        kind.load(f);
        f.instruction(&Instruction::I32Const(
            AsyncDisposableStackDisposeCompletionKind::Throw.wire_code(),
        ));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        let stored = s.reserve_gc_local(f).initialize(
            s.struct_type::<AsyncDisposableStackDisposal>()
                .field(AsyncDisposableStackDisposalSchema::ERROR)
                .read(state, s, f)
                .reference(),
            f,
        );
        let suppressed = s.reserve_value_local(f);
        s.struct_type::<StoredValue>()
            .read_into(&stored, &suppressed, s, f);
        let realm = s.reserve_gc_local(f).initialize(
            s.struct_type::<AsyncDisposableStackDisposal>()
                .field(AsyncDisposableStackDisposalSchema::REALM)
                .read(state, s, f)
                .reference(),
            f,
        );
        let prototype = s.reserve_value_local(f);
        self.emit_load_non_array_realm_intrinsic(
            &realm,
            NonArrayRealmIntrinsicSlot::SuppressedErrorPrototype,
            &prototype,
            f,
        );
        let result = s.reserve_completion(f);
        self.emit_alloc_suppressed_error_instance(
            None,
            error,
            &suppressed,
            &prototype,
            &result,
            f,
        )?;
        folded.copy_from(result.value(), f);
        result.clear(f);
        prototype.clear(f);
        realm.clear(f);
        suppressed.clear(f);
        stored.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let stored = s
            .reserve_gc_local(f)
            .initialize(s.struct_type::<StoredValue>().from_value(&folded, f), f);
        s.struct_type::<AsyncDisposableStackDisposal>()
            .field(AsyncDisposableStackDisposalSchema::ERROR)
            .write(state, GcOperand::reference(&stored, s), s, f);
        s.struct_type::<AsyncDisposableStackDisposal>()
            .field(AsyncDisposableStackDisposalSchema::COMPLETION_KIND)
            .write(
                state,
                GcOperand::constant(AsyncDisposableStackDisposeCompletionKind::Throw),
                s,
                f,
            );
        stored.clear(f);
        folded.clear(f);
        kind.clear(s, f);
        Ok(())
    }

    fn emit_ads_await(
        &mut self,
        state: &GcLocal<AsyncDisposableStackDisposal>,
        value: &ValueLocals,
        pending: &CompletionLocals,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let realm = s.reserve_gc_local(f).initialize(
            s.struct_type::<AsyncDisposableStackDisposal>()
                .field(AsyncDisposableStackDisposalSchema::REALM)
                .read(state, s, f)
                .reference(),
            f,
        );
        let fulfilled = s.reserve_value_local(f);
        let rejected = s.reserve_value_local(f);
        self.emit_ads_closure(
            AdsClosure::Disposal(state, PromiseSettlement::Fulfill),
            &realm,
            &fulfilled,
            f,
        )?;
        self.emit_ads_closure(
            AdsClosure::Disposal(state, PromiseSettlement::Reject),
            &realm,
            &rejected,
            f,
        )?;
        let constructor = self.emit_current_function_realm_intrinsic_promise_constructor(f);
        let capability =
            self.emit_new_current_function_realm_intrinsic_promise_capability(constructor, f)?;
        // Completion(Await) has occurred even if PromiseResolve fails synchronously.
        s.struct_type::<AsyncDisposableStackDisposal>()
            .field(AsyncDisposableStackDisposalSchema::HAS_AWAITED)
            .write(state, GcOperand::boolean(true), s, f);
        self.emit_intrinsic_await_with_handlers(value, &fulfilled, &rejected, &capability, f)?;
        pending.copy_from(self.completion(), f);
        capability.clear(f);
        rejected.clear(f);
        fulfilled.clear(f);
        realm.clear(f);
        Ok(())
    }

    fn emit_ads_finish(
        &mut self,
        state: &GcLocal<AsyncDisposableStackDisposal>,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let promise = s.reserve_gc_local(f).initialize(
            s.struct_type::<AsyncDisposableStackDisposal>()
                .field(AsyncDisposableStackDisposalSchema::PROMISE)
                .read(state, s, f)
                .reference(),
            f,
        );
        let kind = GcI32DomainLocal::new(s, AsyncDisposableStackDisposeCompletionKind::Normal, f);
        s.struct_type::<AsyncDisposableStackDisposal>()
            .field(AsyncDisposableStackDisposalSchema::COMPLETION_KIND)
            .read(state, s, f)
            .store_domain(&kind, f);
        let value = s.reserve_value_local(f);
        value.set_undefined(f);
        kind.load(f);
        f.instruction(&Instruction::I32Const(
            AsyncDisposableStackDisposeCompletionKind::Throw.wire_code(),
        ));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        let stored = s.reserve_gc_local(f).initialize(
            s.struct_type::<AsyncDisposableStackDisposal>()
                .field(AsyncDisposableStackDisposalSchema::ERROR)
                .read(state, s, f)
                .reference(),
            f,
        );
        s.struct_type::<StoredValue>()
            .read_into(&stored, &value, s, f);
        self.emit_settle_promise_record(&promise, PromiseSettlement::Reject, &value, f)?;
        stored.clear(f);
        f.instruction(&Instruction::Else);
        self.emit_settle_promise_record(&promise, PromiseSettlement::Fulfill, &value, f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        value.clear(f);
        kind.clear(s, f);
        promise.clear(f);
        Ok(())
    }

    fn emit_ads_dispose_step(
        &mut self,
        state: &GcLocal<AsyncDisposableStackDisposal>,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let entries = s.reserve_gc_local(f).initialize(
            s.struct_type::<AsyncDisposableStackDisposal>()
                .field(AsyncDisposableStackDisposalSchema::RESOURCES)
                .read(state, s, f)
                .reference(),
            f,
        );
        let next_index = s.reserve_i64_local(f);
        let index = s.reserve_i32_local(f);
        let kind = s.reserve_i32_local(f);
        let needs_await = s.reserve_i32_local(f);
        let has_awaited = s.reserve_i32_local(f);
        let method = s.reserve_value_local(f);
        let value = s.reserve_value_local(f);
        let undefined = s.reserve_value_local(f);
        undefined.set_undefined(f);
        let called = s.reserve_completion(f);
        let awaited = s.reserve_completion(f);
        let empty_arguments = self.emit_pre_evaluated_arg_vector(&[], f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        let next = self.open_frame(ControlFrameKind::Loop, f);
        s.struct_type::<AsyncDisposableStackDisposal>()
            .field(AsyncDisposableStackDisposalSchema::NEXT_INDEX)
            .read(state, s, f)
            .store_i64(next_index, f);
        next_index.load(f);
        f.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, f);
        // Empty async resources request one final Await only if no method Await occurred.
        s.struct_type::<AsyncDisposableStackDisposal>()
            .field(AsyncDisposableStackDisposalSchema::NEEDS_AWAIT)
            .read(state, s, f)
            .store(needs_await, f);
        s.struct_type::<AsyncDisposableStackDisposal>()
            .field(AsyncDisposableStackDisposalSchema::HAS_AWAITED)
            .read(state, s, f)
            .store(has_awaited, f);
        needs_await.load(f);
        has_awaited.load(f);
        f.instruction(&Instruction::I32Eqz);
        f.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_ads_await(state, &undefined, &awaited, f)?;
        awaited.kind().load(f);
        f.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_ads_record_error(state, awaited.value(), f)?;
        self.completion().initialize(f);
        f.instruction(&Instruction::Else);
        self.emit_branch_to_target(exit, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.emit_ads_finish(state, f)?;
        self.emit_branch_to_target(exit, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        next_index.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Sub);
        next_index.store(f);
        s.struct_type::<AsyncDisposableStackDisposal>()
            .field(AsyncDisposableStackDisposalSchema::NEXT_INDEX)
            .write(state, GcOperand::i64_local(next_index), s, f);
        next_index.load(f);
        f.instruction(&Instruction::I32WrapI64);
        index.store(f);
        let entry = s.reserve_gc_local(f).initialize(
            s.array_type::<AsyncDisposableResourceTable>()
                .read(&entries, index, s, f)
                .reference()
                .require_non_null(f),
            f,
        );
        s.array_type::<AsyncDisposableResourceTable>().write(
            &entries,
            index,
            GcOperand::null(s),
            s,
            f,
        );
        s.struct_type::<AsyncDisposableResource>()
            .field(AsyncDisposableResourceSchema::KIND)
            .read(&entry, s, f)
            .store(kind, f);
        let stored_value = s.reserve_gc_local(f).initialize(
            s.struct_type::<AsyncDisposableResource>()
                .field(AsyncDisposableResourceSchema::VALUE)
                .read(&entry, s, f)
                .reference(),
            f,
        );
        let stored_method = s.reserve_gc_local(f).initialize(
            s.struct_type::<AsyncDisposableResource>()
                .field(AsyncDisposableResourceSchema::METHOD)
                .read(&entry, s, f)
                .reference(),
            f,
        );
        s.struct_type::<StoredValue>()
            .read_into(&stored_value, &value, s, f);
        s.struct_type::<StoredValue>()
            .read_into(&stored_method, &method, s, f);
        stored_method.clear(f);
        stored_value.clear(f);
        entry.clear(f);
        kind.load(f);
        f.instruction(&Instruction::I32Const(
            AsyncDisposableStackEntryKind::Empty.word() as i32,
        ));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        s.struct_type::<AsyncDisposableStackDisposal>()
            .field(AsyncDisposableStackDisposalSchema::NEEDS_AWAIT)
            .write(state, GcOperand::boolean(true), s, f);
        self.emit_branch_to_target(next, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        called.initialize(f);
        for entry_kind in AsyncDisposableStackEntryKind::ALL {
            let Some(call) = entry_kind.dispose_call() else {
                continue;
            };
            kind.load(f);
            f.instruction(&Instruction::I32Const(entry_kind.word() as i32));
            f.instruction(&Instruction::I32Eq);
            self.open_frame(ControlFrameKind::If, f);
            match call {
                AsyncDisposableStackDisposeCall::ResourceReceiver => self
                    .emit_function_or_proxy_call_with_argv(
                        &method,
                        &value,
                        &empty_arguments,
                        &called,
                        f,
                    )?,
                AsyncDisposableStackDisposeCall::UndefinedReceiverWithResourceArgument => {
                    let arguments = self.emit_pre_evaluated_arg_vector(&[&value], f);
                    self.emit_function_or_proxy_call_with_argv(
                        &method, &undefined, &arguments, &called, f,
                    )?;
                    arguments.clear(f);
                }
                AsyncDisposableStackDisposeCall::UndefinedReceiverNoArguments => self
                    .emit_function_or_proxy_call_with_argv(
                        &method,
                        &undefined,
                        &empty_arguments,
                        &called,
                        f,
                    )?,
            }
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
        }
        called.kind().load(f);
        f.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_ads_record_error(state, called.value(), f)?;
        f.instruction(&Instruction::Else);
        self.emit_ads_await(state, called.value(), &awaited, f)?;
        awaited.kind().load(f);
        f.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_ads_record_error(state, awaited.value(), f)?;
        self.completion().initialize(f);
        f.instruction(&Instruction::Else);
        self.emit_branch_to_target(exit, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        method.set_undefined(f);
        value.set_undefined(f);
        self.emit_branch_to_target(next, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        empty_arguments.clear(f);
        awaited.clear(f);
        called.clear(f);
        undefined.clear(f);
        value.clear(f);
        method.clear(f);
        s.release_i32_local(has_awaited, f);
        s.release_i32_local(needs_await, f);
        s.release_i32_local(kind, f);
        s.release_i32_local(index, f);
        s.release_i64_local(next_index, f);
        entries.clear(f);
        Ok(())
    }

    pub(crate) fn emit_async_disposable_stack_dispose_async(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        // Allocate the called builtin's intrinsic capability before brand validation.
        let constructor = self.emit_current_function_realm_intrinsic_promise_constructor(f);
        let capability =
            self.emit_new_current_function_realm_intrinsic_promise_capability(constructor, f)?;
        let promise = s.reserve_value_local(f);
        self.emit_read_promise_capability_promise(&capability, &promise, f);
        let record = s
            .reserve_gc_local(f)
            .initialize(promise.cast_reference::<PromiseObject>(s, f), f);
        let receiver = s.reserve_value_local(f);
        self.compile_this_to_locals(&receiver, f)?;
        let output = s.reserve_completion(f);
        output.initialize(f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        let stack = self.emit_ads_brand(&receiver, &output, exit, f)?;
        let status = s.reserve_i32_local(f);
        s.struct_type::<AsyncDisposableStackObject>()
            .field(AsyncDisposableStackObjectSchema::STATE)
            .read(&stack, s, f)
            .store(status, f);
        status.load(f);
        f.instruction(&Instruction::I32Const(
            AsyncDisposableStackState::Disposed.word() as i32,
        ));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        let undefined = s.reserve_value_local(f);
        undefined.set_undefined(f);
        self.emit_settle_promise_record(&record, PromiseSettlement::Fulfill, &undefined, f)?;
        undefined.clear(f);
        self.emit_branch_to_target(exit, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        s.struct_type::<AsyncDisposableStackObject>()
            .field(AsyncDisposableStackObjectSchema::STATE)
            .write(
                &stack,
                GcOperand::constant(AsyncDisposableStackState::Disposed),
                s,
                f,
            );
        let resources = s.reserve_gc_local(f).initialize(
            s.struct_type::<AsyncDisposableStackObject>()
                .field(AsyncDisposableStackObjectSchema::RESOURCES)
                .read(&stack, s, f)
                .reference(),
            f,
        );
        let entries = s.reserve_gc_local(f).initialize(
            s.struct_type::<AsyncDisposableResourceStack>()
                .field(AsyncDisposableResourceStackSchema::RESOURCES)
                .read(&resources, s, f)
                .reference(),
            f,
        );
        let count = s.reserve_i64_local(f);
        s.struct_type::<AsyncDisposableResourceStack>()
            .field(AsyncDisposableResourceStackSchema::ENTRY_COUNT)
            .read(&resources, s, f)
            .store_i64(count, f);
        let empty = self.emit_empty_async_disposable_resource_stack(f);
        s.struct_type::<AsyncDisposableStackObject>()
            .field(AsyncDisposableStackObjectSchema::RESOURCES)
            .write(&stack, GcOperand::reference(&empty, s), s, f);
        let realm = self.emit_execution_realm(f);
        let undefined = s.reserve_value_local(f);
        undefined.set_undefined(f);
        let error = s
            .reserve_gc_local(f)
            .initialize(s.struct_type::<StoredValue>().from_value(&undefined, f), f);
        let state = s.reserve_gc_local(f).initialize(
            s.struct_type::<AsyncDisposableStackDisposal>().construct(
                (
                    GcOperand::reference(&record, s),
                    GcOperand::reference(&realm, s),
                    GcOperand::reference(&entries, s),
                    GcOperand::i64_local(count),
                    GcOperand::constant(AsyncDisposableStackDisposeCompletionKind::Normal),
                    GcOperand::reference(&error, s),
                    GcOperand::boolean(false),
                    GcOperand::boolean(false),
                ),
                f,
            ),
            f,
        );
        self.emit_ads_dispose_step(&state, f)?;
        state.clear(f);
        error.clear(f);
        undefined.clear(f);
        realm.clear(f);
        empty.clear(f);
        s.release_i64_local(count, f);
        entries.clear(f);
        resources.clear(f);
        s.release_i32_local(status, f);
        stack.clear(f);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        output.kind().load(f);
        f.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_settle_promise_record(&record, PromiseSettlement::Reject, output.value(), f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.completion().set_normal(&promise, f);
        output.clear(f);
        receiver.clear(f);
        record.clear(f);
        promise.clear(f);
        capability.clear(f);
        Ok(())
    }

    pub(crate) fn emit_async_disposable_stack_dispose_async_fulfilled(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let state = self.emit_ads_disposal_capture(f);
        self.emit_ads_dispose_step(&state, f)?;
        let undefined = self.runtime_schema().reserve_value_local(f);
        undefined.set_undefined(f);
        self.completion().set_normal(&undefined, f);
        undefined.clear(f);
        state.clear(f);
        Ok(())
    }
    pub(crate) fn emit_async_disposable_stack_dispose_async_rejected(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let state = self.emit_ads_disposal_capture(f);
        let reason = self.runtime_schema().reserve_value_local(f);
        self.emit_builtin_arg_to_value(0, &reason, f);
        self.emit_ads_record_error(&state, &reason, f)?;
        self.emit_ads_dispose_step(&state, f)?;
        reason.set_undefined(f);
        self.completion().set_normal(&reason, f);
        reason.clear(f);
        state.clear(f);
        Ok(())
    }
}
