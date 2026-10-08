use super::*;

enum AdsRegistration {
    Use,
    Adopt,
    Defer,
}
impl FunctionBuilder<'_> {
    fn emit_ads_push_entry(
        &mut self,
        resources: &GcLocal<AsyncDisposableResourceStack>,
        kind: AsyncDisposableStackEntryKind,
        value: &ValueLocals,
        method: &ValueLocals,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let count = s.reserve_i64_local(f);
        let capacity = s.reserve_i32_local(f);
        let index = s.reserve_i32_local(f);
        let entries = s.reserve_gc_local(f).initialize(
            s.struct_type::<AsyncDisposableResourceStack>()
                .field(AsyncDisposableResourceStackSchema::RESOURCES)
                .read(resources, s, f)
                .reference(),
            f,
        );
        s.struct_type::<AsyncDisposableResourceStack>()
            .field(AsyncDisposableResourceStackSchema::ENTRY_COUNT)
            .read(resources, s, f)
            .store_i64(count, f);
        s.array_type::<AsyncDisposableResourceTable>()
            .length(&entries, s, f);
        capacity.store(f);
        count.load(f);
        capacity.load(f);
        f.instruction(&Instruction::I64ExtendI32U);
        f.instruction(&Instruction::I64GtU);
        self.open_frame(ControlFrameKind::If, f);
        f.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        count.load(f);
        f.instruction(&Instruction::I64Const(u32::MAX as i64));
        f.instruction(&Instruction::I64GeU);
        self.open_frame(ControlFrameKind::If, f);
        f.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        count.load(f);
        capacity.load(f);
        f.instruction(&Instruction::I64ExtendI32U);
        f.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, f);
        let next_capacity = s.reserve_i64_local(f);
        capacity.load(f);
        f.instruction(&Instruction::I64ExtendI32U);
        f.instruction(&Instruction::I64Const(2));
        f.instruction(&Instruction::I64Mul);
        next_capacity.store(f);
        next_capacity.load(f);
        f.instruction(&Instruction::I64Const(4));
        f.instruction(&Instruction::I64LtU);
        self.open_frame(ControlFrameKind::If, f);
        f.instruction(&Instruction::I64Const(4));
        next_capacity.store(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        next_capacity.load(f);
        f.instruction(&Instruction::I64Const(u32::MAX as i64));
        f.instruction(&Instruction::I64GtU);
        self.open_frame(ControlFrameKind::If, f);
        f.instruction(&Instruction::I64Const(u32::MAX as i64));
        next_capacity.store(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        next_capacity.load(f);
        f.instruction(&Instruction::I32WrapI64);
        capacity.store(f);
        let grown = s.reserve_gc_local(f).initialize(
            s.array_type::<AsyncDisposableResourceTable>()
                .filled(GcOperand::null(s), capacity, f),
            f,
        );
        f.instruction(&Instruction::I32Const(0));
        index.store(f);
        let copied = self.open_frame(ControlFrameKind::Block, f);
        let next = self.open_frame(ControlFrameKind::Loop, f);
        index.load(f);
        f.instruction(&Instruction::I64ExtendI32U);
        count.load(f);
        f.instruction(&Instruction::I64GeU);
        self.emit_branch_if_to_target(copied, f);
        let entry = s.reserve_gc_local(f).initialize(
            s.array_type::<AsyncDisposableResourceTable>()
                .read(&entries, index, s, f)
                .reference(),
            f,
        );
        s.array_type::<AsyncDisposableResourceTable>().write(
            &grown,
            index,
            GcOperand::reference(&entry, s),
            s,
            f,
        );
        entry.clear(f);
        index.load(f);
        f.instruction(&Instruction::I32Const(1));
        f.instruction(&Instruction::I32Add);
        index.store(f);
        self.emit_branch_to_target(next, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        s.struct_type::<AsyncDisposableResourceStack>()
            .field(AsyncDisposableResourceStackSchema::RESOURCES)
            .write(resources, GcOperand::reference(&grown, s), s, f);
        entries.replace(grown.load(s, f), f);
        grown.clear(f);
        s.release_i64_local(next_capacity, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        count.load(f);
        f.instruction(&Instruction::I32WrapI64);
        index.store(f);
        let stored_value = s
            .reserve_gc_local(f)
            .initialize(s.struct_type::<StoredValue>().from_value(value, f), f);
        let stored_method = s
            .reserve_gc_local(f)
            .initialize(s.struct_type::<StoredValue>().from_value(method, f), f);
        let entry = s.reserve_gc_local(f).initialize(
            s.struct_type::<AsyncDisposableResource>().construct(
                (
                    GcOperand::constant(kind),
                    GcOperand::reference(&stored_value, s),
                    GcOperand::reference(&stored_method, s),
                ),
                f,
            ),
            f,
        );
        s.array_type::<AsyncDisposableResourceTable>().write(
            &entries,
            index,
            GcOperand::nullable_reference(&entry, s),
            s,
            f,
        );
        count.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        count.store(f);
        s.struct_type::<AsyncDisposableResourceStack>()
            .field(AsyncDisposableResourceStackSchema::ENTRY_COUNT)
            .write(resources, GcOperand::i64_local(count), s, f);
        entry.clear(f);
        stored_method.clear(f);
        stored_value.clear(f);
        entries.clear(f);
        s.release_i32_local(index, f);
        s.release_i32_local(capacity, f);
        s.release_i64_local(count, f);
        Ok(())
    }

    fn emit_ads_get_method(
        &mut self,
        resource: &ValueLocals,
        symbol: lila_ir::WellKnownSymbol,
        method: &ValueLocals,
        pending: &CompletionLocals,
        output: &CompletionLocals,
        exit: ControlTarget,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let symbol = s
            .reserve_gc_local(f)
            .initialize(self.emit_well_known_symbol_reference(symbol, f)?, f);
        let key = PropertyKeyLocals::from_symbol(s, &symbol, f);
        self.emit_object_read(resource, resource, &key, pending, f)?;
        key.clear(f);
        symbol.clear(f);
        self.emit_ads_abrupt_exit(pending, output, exit, f);
        method.copy_from(pending.value(), f);
        self.compile_nullish_tagged_i32(method.tag(), f)?;
        self.open_frame(ControlFrameKind::If, f);
        method.set_undefined(f);
        f.instruction(&Instruction::Else);
        self.emit_is_callable_i32(method, f)?;
        f.instruction(&Instruction::I32Eqz);
        self.emit_ads_type_error_if(
            RuntimeErrorMessage::ASYNCDISPOSABLESTACK_PROTOTYPE_USE_DISPOSE_METHOD_IS_NOT_CALLABLE,
            output,
            exit,
            f,
        )?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        Ok(())
    }

    fn emit_ads_registration(
        &mut self,
        operation: AdsRegistration,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_with_ads(AdsReceiverPolicy::Pending,f,|b,stack,output,exit,f| {
            let s=b.runtime_schema();
            // AddDisposableResource owns this List before GetDisposeMethod can move the stack.
            let resources=s.reserve_gc_local(f).initialize(s.struct_type::<AsyncDisposableStackObject>().field(AsyncDisposableStackObjectSchema::RESOURCES).read(stack,s,f).reference(),f);
            let value=s.reserve_value_local(f); let method=s.reserve_value_local(f); let pending=s.reserve_completion(f);
            value.set_undefined(f); method.set_undefined(f);
            match operation {
                AdsRegistration::Use=> {
                    b.emit_builtin_arg_to_value(0,&value,f);
                    b.compile_nullish_tagged_i32(value.tag(),f)?;
                    b.open_frame(ControlFrameKind::If,f);
                    b.emit_ads_push_entry(&resources,AsyncDisposableStackEntryKind::Empty,&method,&method,f)?;
                    f.instruction(&Instruction::Else);
                    b.emit_is_heap_object_like_tag_i32(value.tag(),f); f.instruction(&Instruction::I32Eqz);
                    b.emit_ads_type_error_if(RuntimeErrorMessage::ASYNCDISPOSABLESTACK_PROTOTYPE_USE_VALUE_IS_NOT_AN_OBJECT,output,exit,f)?;
                    b.emit_ads_get_method(&value,lila_ir::WellKnownSymbol::AsyncDispose,&method,&pending,output,exit,f)?;
                    method.tag().load(f); f.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Undefined.tag())); f.instruction(&Instruction::I32Eq);
                    b.open_frame(ControlFrameKind::If,f);
                    b.emit_ads_get_method(&value,lila_ir::WellKnownSymbol::Dispose,&method,&pending,output,exit,f)?;
                    method.tag().load(f); f.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Undefined.tag())); f.instruction(&Instruction::I32Ne);
                    b.open_frame(ControlFrameKind::If,f);
                    let stored=s.reserve_gc_local(f).initialize(s.struct_type::<StoredValue>().from_value(&method,f),f);
                    let context=s.reserve_gc_local(f).initialize(s.struct_type::<AsyncDisposableStackSyncDisposeContext>().construct((GcOperand::reference(&stored,s),),f),f);
                    let realm=b.emit_execution_realm(f);
                    b.emit_ads_closure(AdsClosure::SyncDispose(&context),&realm,&method,f)?;
                    realm.clear(f);context.clear(f);stored.clear(f);
                    b.pop_control(ControlFrameKind::If);f.instruction(&Instruction::End);
                    b.pop_control(ControlFrameKind::If);f.instruction(&Instruction::End);
                    method.tag().load(f); f.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Undefined.tag())); f.instruction(&Instruction::I32Eq);
                    b.emit_ads_type_error_if(RuntimeErrorMessage::ASYNCDISPOSABLESTACK_PROTOTYPE_USE_VALUE_IS_NOT_DISPOSABLE,output,exit,f)?;
                    b.emit_ads_push_entry(&resources,AsyncDisposableStackEntryKind::Use,&value,&method,f)?;
                    b.pop_control(ControlFrameKind::If); f.instruction(&Instruction::End);
                }
                AdsRegistration::Adopt=> {
                    b.emit_builtin_arg_to_value(0,&value,f);b.emit_builtin_arg_to_value(1,&method,f);
                    b.emit_is_callable_i32(&method,f)?;f.instruction(&Instruction::I32Eqz);
                    b.emit_ads_type_error_if(RuntimeErrorMessage::ASYNCDISPOSABLESTACK_PROTOTYPE_ADOPT_ONDISPOSEASYNC_IS_NOT_CALLABLE,output,exit,f)?;
                    b.emit_ads_push_entry(&resources,AsyncDisposableStackEntryKind::Adopt,&value,&method,f)?;
                }
                AdsRegistration::Defer=> {
                    b.emit_builtin_arg_to_value(0,&method,f);b.emit_is_callable_i32(&method,f)?;f.instruction(&Instruction::I32Eqz);
                    b.emit_ads_type_error_if(RuntimeErrorMessage::ASYNCDISPOSABLESTACK_PROTOTYPE_DEFER_ONDISPOSEASYNC_IS_NOT_CALLABLE,output,exit,f)?;
                    b.emit_ads_push_entry(&resources,AsyncDisposableStackEntryKind::Defer,&value,&method,f)?;
                }
            }
            output.set_normal(&value,f);
            pending.clear(f);method.clear(f);value.clear(f);resources.clear(f); Ok(())
        })
    }
    pub(crate) fn emit_async_disposable_stack_use(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_ads_registration(AdsRegistration::Use, f)
    }
    pub(crate) fn emit_async_disposable_stack_adopt(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_ads_registration(AdsRegistration::Adopt, f)
    }
    pub(crate) fn emit_async_disposable_stack_defer(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_ads_registration(AdsRegistration::Defer, f)
    }

    pub(crate) fn emit_async_disposable_stack_sync_dispose(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let capture = self.emit_ads_capture(
            BuiltinClosureCaptureKind::AsyncDisposableStackSyncDispose,
            f,
        );
        let context = s.reserve_gc_local(f).initialize(
            s.struct_type::<BuiltinClosureCapture>()
                .field(BuiltinClosureCaptureSchema::ASYNC_DISPOSABLE_STACK_SYNC_DISPOSE)
                .read(&capture, s, f)
                .reference()
                .require_non_null(f),
            f,
        );
        let stored = s.reserve_gc_local(f).initialize(
            s.struct_type::<AsyncDisposableStackSyncDisposeContext>()
                .field(AsyncDisposableStackSyncDisposeContextSchema::METHOD)
                .read(&context, s, f)
                .reference(),
            f,
        );
        let method = s.reserve_value_local(f);
        s.struct_type::<StoredValue>()
            .read_into(&stored, &method, s, f);
        let receiver = s.reserve_value_local(f);
        self.compile_this_to_locals(&receiver, f)?;
        // GetDisposeMethod allocates the acquisition-Realm intrinsic capability before Call.
        let constructor = self.emit_current_function_realm_intrinsic_promise_constructor(f);
        let capability =
            self.emit_new_current_function_realm_intrinsic_promise_capability(constructor, f)?;
        let promise = s.reserve_value_local(f);
        self.emit_read_promise_capability_promise(&capability, &promise, f);
        let called = s.reserve_completion(f);
        let settled = s.reserve_completion(f);
        let undefined = s.reserve_value_local(f);
        undefined.set_undefined(f);
        let arguments = self.emit_pre_evaluated_arg_vector(&[], f);
        self.emit_function_or_proxy_call_with_argv(&method, &receiver, &arguments, &called, f)?;
        called.kind().load(f);
        f.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_call_promise_capability(
            &capability,
            PromiseSettlement::Reject,
            called.value(),
            &settled,
            f,
        )?;
        f.instruction(&Instruction::Else);
        // The returned value is discarded without observing a Promise or thenable.
        self.emit_call_promise_capability(
            &capability,
            PromiseSettlement::Fulfill,
            &undefined,
            &settled,
            f,
        )?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.completion().set_normal(&promise, f);
        arguments.clear(f);
        undefined.clear(f);
        settled.clear(f);
        called.clear(f);
        promise.clear(f);
        capability.clear(f);
        receiver.clear(f);
        method.clear(f);
        stored.clear(f);
        context.clear(f);
        capture.clear(f);
        Ok(())
    }
}
