use super::super::*;
use crate::functions::OrdinaryDefaultPrototype;
use crate::gc_types::{
    CompletionLocals, DisposableResource, DisposableResourceSchema, DisposableResourceStack,
    DisposableResourceStackSchema, DisposableResourceTable, DisposableStack, DisposableStackSchema,
    GcI32DomainLocal, GcLocal, GcNullability, GcOperand, I64Local, OrdinaryObject, StoredValue,
    ValueLocals,
};
use crate::objects::PropertyKeyLocals;
use lila_ir::NativeErrorKind;
mod capability_transfer;

#[must_use]
struct PendingDisposableStackRecordLocal {
    header: GcLocal<OrdinaryObject>,
    resources: GcLocal<DisposableResourceStack>,
}

impl FunctionBuilder<'_> {
    fn emit_disposable_stack_push_entry(
        &mut self,
        resources: &GcLocal<DisposableResourceStack>,
        kind: DisposableStackEntryKind,
        value: &ValueLocals,
        method: &ValueLocals,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let count = s.reserve_i64_local(f);
        let capacity = s.reserve_i32_local(f);
        let index = s.reserve_i32_local(f);
        let entries = s.reserve_gc_local(f).initialize(
            s.struct_type::<DisposableResourceStack>()
                .field(DisposableResourceStackSchema::RESOURCES)
                .read(resources, s, f)
                .reference(),
            f,
        );
        s.struct_type::<DisposableResourceStack>()
            .field(DisposableResourceStackSchema::ENTRY_COUNT)
            .read(resources, s, f)
            .store_i64(count, f);
        s.array_type::<DisposableResourceTable>()
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
            s.array_type::<DisposableResourceTable>()
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
            s.array_type::<DisposableResourceTable>()
                .read(&entries, index, s, f)
                .reference()
                .require_non_null(f),
            f,
        );
        s.array_type::<DisposableResourceTable>().write(
            &grown,
            index,
            GcOperand::nullable_reference(&entry, s),
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
        s.struct_type::<DisposableResourceStack>()
            .field(DisposableResourceStackSchema::RESOURCES)
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
            s.struct_type::<DisposableResource>().construct(
                (
                    GcOperand::constant(kind),
                    GcOperand::reference(&stored_value, s),
                    GcOperand::reference(&stored_method, s),
                ),
                f,
            ),
            f,
        );
        s.array_type::<DisposableResourceTable>().write(
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
        s.struct_type::<DisposableResourceStack>()
            .field(DisposableResourceStackSchema::ENTRY_COUNT)
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
}
struct PendingDisposableStack<'v>(&'v GcLocal<DisposableStack>);
#[must_use]
struct DisposableStackDisposalLocals {
    resources: GcLocal<DisposableResourceStack>,
    entries: GcLocal<DisposableResourceTable>,
    next_index: I64Local,
}
enum DisposableStackRegistration {
    Use,
    Adopt,
    Defer,
}

impl FunctionBuilder<'_> {
    fn emit_empty_disposable_resource_stack(
        &mut self,
        f: &mut Function,
    ) -> GcLocal<DisposableResourceStack> {
        let s = self.runtime_schema();
        let entries = s
            .reserve_gc_local(f)
            .initialize(s.array_type::<DisposableResourceTable>().fixed([], f), f);
        let list = s.reserve_gc_local(f).initialize(
            s.struct_type::<DisposableResourceStack>()
                .construct((GcOperand::reference(&entries, s), GcOperand::i64(0)), f),
            f,
        );
        entries.clear(f);
        list
    }

    fn emit_alloc_pending_disposable_stack_record(
        &mut self,
        header: GcLocal<OrdinaryObject>,
        f: &mut Function,
    ) -> PendingDisposableStackRecordLocal {
        let resources = self.emit_empty_disposable_resource_stack(f);
        PendingDisposableStackRecordLocal { header, resources }
    }

    fn emit_finalize_disposable_stack_instance(
        &mut self,
        pending: PendingDisposableStackRecordLocal,
        f: &mut Function,
    ) -> GcLocal<DisposableStack> {
        let s = self.runtime_schema();
        let stack = s.reserve_gc_local(f).initialize(
            s.struct_type::<DisposableStack>().construct(
                (
                    GcOperand::reference(&pending.header, s),
                    GcOperand::reference(&pending.resources, s),
                    GcOperand::constant(DisposableStackState::Pending),
                ),
                f,
            ),
            f,
        );
        pending.resources.clear(f);
        pending.header.clear(f);
        stack
    }

    fn emit_disposable_stack_abrupt_exit(
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

    fn emit_disposable_stack_error_if(
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

    fn emit_with_disposable_stack(
        &mut self,
        f: &mut Function,
        consume: impl FnOnce(
            &mut Self,
            &GcLocal<DisposableStack>,
            &CompletionLocals,
            ControlTarget,
            &mut Function,
        ) -> Result<(), EmitError>,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let receiver = s.reserve_value_local(f);
        let output = s.reserve_completion(f);
        output.initialize(f);
        self.compile_this_to_locals(&receiver, f)?;
        let exit = self.open_frame(ControlFrameKind::Block, f);
        self.emit_is_heap_object_like_tag_i32(receiver.tag(), f);
        f.instruction(&Instruction::I32Eqz);
        self.emit_disposable_stack_error_if(
            RuntimeErrorMessage::DISPOSABLESTACK_METHOD_RECEIVER_IS_NOT_AN_OBJECT,
            &output,
            exit,
            f,
        )?;
        receiver.reference().load(f);
        f.instruction(&Instruction::RefTestNonNull(
            s.reference_type::<DisposableStack>(GcNullability::NonNullable)
                .heap_type,
        ));
        f.instruction(&Instruction::I32Eqz);
        self.emit_disposable_stack_error_if(
            RuntimeErrorMessage::DISPOSABLESTACK_METHOD_RECEIVER_DOES_NOT_HAVE_DISPOSABLESTATE,
            &output,
            exit,
            f,
        )?;
        let stack = s
            .reserve_gc_local(f)
            .initialize(receiver.cast_reference::<DisposableStack>(s, f), f);
        consume(self, &stack, &output, exit, f)?;
        stack.clear(f);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        self.completion().copy_from(&output, f);
        output.clear(f);
        receiver.clear(f);
        Ok(())
    }

    fn emit_with_pending_disposable_stack(
        &mut self,
        f: &mut Function,
        consume: impl FnOnce(
            &mut Self,
            PendingDisposableStack<'_>,
            &CompletionLocals,
            ControlTarget,
            &mut Function,
        ) -> Result<(), EmitError>,
    ) -> Result<(), EmitError> {
        self.emit_with_disposable_stack(f, |b, stack, output, exit, f| {
            let s = b.runtime_schema();
            let state = GcI32DomainLocal::new(s, DisposableStackState::Pending, f);
            s.struct_type::<DisposableStack>()
                .field(DisposableStackSchema::STATE)
                .read(stack, s, f)
                .store_domain(&state, f);
            state.load(f);
            f.instruction(&Instruction::I32Const(
                DisposableStackState::Disposed.word() as i32,
            ));
            f.instruction(&Instruction::I32Eq);
            b.open_frame(ControlFrameKind::If, f);
            b.emit_throw_current_function_realm_error(
                NativeErrorKind::ReferenceError,
                RuntimeErrorMessage::DISPOSABLESTACK_IS_ALREADY_DISPOSED,
                output,
                f,
            )?;
            b.emit_branch_to_target(exit, f);
            b.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
            state.clear(s, f);
            consume(b, PendingDisposableStack(stack), output, exit, f)
        })
    }

    pub(crate) fn emit_disposable_stack_constructor(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let new_target = s.reserve_value_local(f);
        let value = s.reserve_value_local(f);
        let output = s.reserve_completion(f);
        self.compile_new_target_to_locals(&new_target, f)?;
        let exit = self.open_frame(ControlFrameKind::Block, f);
        new_target.tag().load(f);
        f.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Undefined.tag()));
        f.instruction(&Instruction::I32Eq);
        self.emit_disposable_stack_error_if(
            RuntimeErrorMessage::DISPOSABLESTACK_CONSTRUCTOR_REQUIRES_NEW,
            &output,
            exit,
            f,
        )?;
        self.emit_get_prototype_from_constructor(
            &new_target,
            OrdinaryDefaultPrototype::DisposableStack,
            &output,
            f,
        )?;
        self.emit_disposable_stack_abrupt_exit(&output, &output, exit, f);
        let header = s.reserve_gc_local(f).initialize(
            self.emit_alloc_plain_object_with_prototype(Some(output.value()), f)?,
            f,
        );
        let pending = self.emit_alloc_pending_disposable_stack_record(header, f);
        let stack = self.emit_finalize_disposable_stack_instance(pending, f);
        value.set_reference(&stack, s, f);
        output.set_normal(&value, f);
        stack.clear(f);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        self.completion().copy_from(&output, f);
        output.clear(f);
        value.clear(f);
        new_target.clear(f);
        Ok(())
    }

    fn emit_disposable_stack_registration(
        &mut self,
        operation: DisposableStackRegistration,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_with_pending_disposable_stack(f, |b, stack, output, exit, f| {
            let s = b.runtime_schema();
            // Capture the actual List before observable GetDisposeMethod.
            let resources = s.reserve_gc_local(f).initialize(
                s.struct_type::<DisposableStack>().field(DisposableStackSchema::RESOURCE_STACK).read(stack.0, s, f).reference(), f,
            );
            let value = s.reserve_value_local(f);
            let method = s.reserve_value_local(f);
            let pending = s.reserve_completion(f);
            value.set_undefined(f);
            match operation {
                DisposableStackRegistration::Use => {
                    b.emit_builtin_arg_to_value(0, &value, f);
                    output.set_normal(&value, f);
                    b.compile_nullish_tagged_i32(value.tag(), f)?;
                    f.instruction(&Instruction::I32Eqz);
                    b.open_frame(ControlFrameKind::If, f);
                    b.emit_is_heap_object_like_tag_i32(value.tag(), f);
                    f.instruction(&Instruction::I32Eqz);
                    b.emit_disposable_stack_error_if(RuntimeErrorMessage::DISPOSABLESTACK_PROTOTYPE_USE_VALUE_IS_NOT_AN_OBJECT, output, exit, f)?;
                    let symbol = s.reserve_gc_local(f).initialize(b.emit_well_known_symbol_reference(lila_ir::WellKnownSymbol::Dispose, f)?, f);
                    let key = PropertyKeyLocals::from_symbol(s, &symbol, f);
                    b.emit_object_read(&value, &value, &key, &pending, f)?;
                    b.emit_disposable_stack_abrupt_exit(&pending, output, exit, f);
                    method.copy_from(pending.value(), f);
                    b.compile_nullish_tagged_i32(method.tag(), f)?;
                    b.emit_disposable_stack_error_if(RuntimeErrorMessage::DISPOSABLESTACK_PROTOTYPE_USE_VALUE_IS_NOT_DISPOSABLE, output, exit, f)?;
                    b.emit_is_callable_i32(&method, f)?;
                    f.instruction(&Instruction::I32Eqz);
                    b.emit_disposable_stack_error_if(RuntimeErrorMessage::DISPOSABLESTACK_PROTOTYPE_USE_DISPOSE_METHOD_IS_NOT_CALLABLE, output, exit, f)?;
                    b.emit_disposable_stack_push_entry(&resources, DisposableStackEntryKind::Use, &value, &method, f)?;
                    key.clear(f);
                    symbol.clear(f);
                    b.pop_control(ControlFrameKind::If);
                    f.instruction(&Instruction::End);
                }
                DisposableStackRegistration::Adopt => {
                    b.emit_builtin_arg_to_value(0, &value, f);
                    b.emit_builtin_arg_to_value(1, &method, f);
                    b.emit_is_callable_i32(&method, f)?;
                    f.instruction(&Instruction::I32Eqz);
                    b.emit_disposable_stack_error_if(RuntimeErrorMessage::DISPOSABLESTACK_PROTOTYPE_ADOPT_ONDISPOSE_IS_NOT_CALLABLE, output, exit, f)?;
                    b.emit_disposable_stack_push_entry(&resources, DisposableStackEntryKind::Adopt, &value, &method, f)?;
                    output.set_normal(&value, f);
                }
                DisposableStackRegistration::Defer => {
                    b.emit_builtin_arg_to_value(0, &method, f);
                    b.emit_is_callable_i32(&method, f)?;
                    f.instruction(&Instruction::I32Eqz);
                    b.emit_disposable_stack_error_if(RuntimeErrorMessage::DISPOSABLESTACK_PROTOTYPE_DEFER_ONDISPOSE_IS_NOT_CALLABLE, output, exit, f)?;
                    b.emit_disposable_stack_push_entry(&resources, DisposableStackEntryKind::Defer, &value, &method, f)?;
                    output.set_normal(&value, f);
                }
            }
            pending.clear(f);
            method.clear(f);
            value.clear(f);
            resources.clear(f);
            Ok(())
        })
    }

    pub(crate) fn emit_disposable_stack_use(&mut self, f: &mut Function) -> Result<(), EmitError> {
        self.emit_disposable_stack_registration(DisposableStackRegistration::Use, f)
    }
    pub(crate) fn emit_disposable_stack_adopt(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_disposable_stack_registration(DisposableStackRegistration::Adopt, f)
    }
    pub(crate) fn emit_disposable_stack_defer(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_disposable_stack_registration(DisposableStackRegistration::Defer, f)
    }

    pub(crate) fn emit_disposable_stack_move(&mut self, f: &mut Function) -> Result<(), EmitError> {
        self.emit_with_pending_disposable_stack(f, |b, source, output, _, f| {
            let s = b.runtime_schema();
            let header = s.reserve_gc_local(f).initialize(
                b.emit_alloc_current_function_realm_disposable_stack_object(f)?,
                f,
            );
            let pending = b.emit_alloc_pending_disposable_stack_record(header, f);
            let transfer = b.emit_take_disposable_stack_capability(&source, f);
            let pending =
                b.emit_install_transferred_disposable_stack_capability(pending, transfer, f);
            let stack = b.emit_finalize_disposable_stack_instance(pending, f);
            let value = s.reserve_value_local(f);
            value.set_reference(&stack, s, f);
            output.set_normal(&value, f);
            value.clear(f);
            stack.clear(f);
            Ok(())
        })
    }

    pub(crate) fn emit_disposable_stack_disposed_getter(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_with_disposable_stack(f, |b, stack, output, _, f| {
            let s = b.runtime_schema();
            let state = GcI32DomainLocal::new(s, DisposableStackState::Pending, f);
            let truth = s.reserve_i32_local(f);
            s.struct_type::<DisposableStack>()
                .field(DisposableStackSchema::STATE)
                .read(stack, s, f)
                .store_domain(&state, f);
            state.load(f);
            f.instruction(&Instruction::I32Const(
                DisposableStackState::Disposed.word() as i32,
            ));
            f.instruction(&Instruction::I32Eq);
            truth.store(f);
            output.value().set_boolean(truth, f);
            output.set_normal(output.value(), f);
            s.release_i32_local(truth, f);
            state.clear(s, f);
            Ok(())
        })
    }
}

impl FunctionBuilder<'_> {
    fn emit_begin_disposable_stack_disposal(
        &mut self,
        stack: &GcLocal<DisposableStack>,
        f: &mut Function,
    ) -> DisposableStackDisposalLocals {
        let s = self.runtime_schema();
        // Publish disposed before the first user callback.
        s.struct_type::<DisposableStack>()
            .field(DisposableStackSchema::STATE)
            .write(
                stack,
                GcOperand::constant(DisposableStackState::Disposed),
                s,
                f,
            );
        let resources = s.reserve_gc_local(f).initialize(
            s.struct_type::<DisposableStack>()
                .field(DisposableStackSchema::RESOURCE_STACK)
                .read(stack, s, f)
                .reference(),
            f,
        );
        let entries = s.reserve_gc_local(f).initialize(
            s.struct_type::<DisposableResourceStack>()
                .field(DisposableResourceStackSchema::RESOURCES)
                .read(&resources, s, f)
                .reference(),
            f,
        );
        let next_index = s.reserve_i64_local(f);
        s.struct_type::<DisposableResourceStack>()
            .field(DisposableResourceStackSchema::ENTRY_COUNT)
            .read(&resources, s, f)
            .store_i64(next_index, f);
        DisposableStackDisposalLocals {
            resources,
            entries,
            next_index,
        }
    }

    fn emit_consume_disposable_stack_disposal(
        &mut self,
        disposal: DisposableStackDisposalLocals,
        output: &CompletionLocals,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let index = s.reserve_i32_local(f);
        let kind_word = GcI32DomainLocal::new(s, DisposableStackEntryKind::Use, f);
        let value = s.reserve_value_local(f);
        let method = s.reserve_value_local(f);
        let receiver = s.reserve_value_local(f);
        let called = s.reserve_completion(f);
        let combined = s.reserve_completion(f);
        let empty_arguments = self.emit_pre_evaluated_arg_vector(&[], f);
        let done = self.open_frame(ControlFrameKind::Block, f);
        let next = self.open_frame(ControlFrameKind::Loop, f);
        disposal.next_index.load(f);
        f.instruction(&Instruction::I64Eqz);
        self.emit_branch_if_to_target(done, f);
        disposal.next_index.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Sub);
        disposal.next_index.store(f);
        disposal.next_index.load(f);
        f.instruction(&Instruction::I32WrapI64);
        index.store(f);
        let entry = s.reserve_gc_local(f).initialize(
            s.array_type::<DisposableResourceTable>()
                .read(&disposal.entries, index, s, f)
                .reference()
                .require_non_null(f),
            f,
        );
        // Detach the entry before executing its callback.
        s.array_type::<DisposableResourceTable>().write(
            &disposal.entries,
            index,
            GcOperand::null(s),
            s,
            f,
        );
        s.struct_type::<DisposableResource>()
            .field(DisposableResourceSchema::KIND)
            .read(&entry, s, f)
            .store_domain(&kind_word, f);
        let stored_value = s.reserve_gc_local(f).initialize(
            s.struct_type::<DisposableResource>()
                .field(DisposableResourceSchema::VALUE)
                .read(&entry, s, f)
                .reference(),
            f,
        );
        let stored_method = s.reserve_gc_local(f).initialize(
            s.struct_type::<DisposableResource>()
                .field(DisposableResourceSchema::METHOD)
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
        called.initialize(f);
        for kind in DisposableStackEntryKind::ALL {
            kind_word.load(f);
            f.instruction(&Instruction::I32Const(kind.word() as i32));
            f.instruction(&Instruction::I32Eq);
            self.open_frame(ControlFrameKind::If, f);
            match kind.dispose_call() {
                DisposableStackDisposeCall::ResourceReceiver => {
                    self.emit_function_or_proxy_call_with_argv(
                        &method,
                        &value,
                        &empty_arguments,
                        &called,
                        f,
                    )?;
                }
                DisposableStackDisposeCall::UndefinedReceiverWithResourceArgument => {
                    receiver.set_undefined(f);
                    let arguments = self.emit_pre_evaluated_arg_vector(&[&value], f);
                    self.emit_function_or_proxy_call_with_argv(
                        &method, &receiver, &arguments, &called, f,
                    )?;
                    arguments.clear(f);
                }
                DisposableStackDisposeCall::UndefinedReceiverNoArguments => {
                    receiver.set_undefined(f);
                    self.emit_function_or_proxy_call_with_argv(
                        &method,
                        &receiver,
                        &empty_arguments,
                        &called,
                        f,
                    )?;
                }
            }
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
        }
        // This synchronous stack discards normal return values, including Promises.
        called.kind().load(f);
        f.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        output.kind().load(f);
        f.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        let realm = self.emit_execution_realm(f);
        let prototype = s.reserve_value_local(f);
        self.emit_load_non_array_realm_intrinsic(
            &realm,
            crate::functions::NonArrayRealmIntrinsicSlot::SuppressedErrorPrototype,
            &prototype,
            f,
        );
        self.emit_alloc_suppressed_error_instance(
            None,
            called.value(),
            output.value(),
            &prototype,
            &combined,
            f,
        )?;
        output.set_throw(combined.value(), f);
        prototype.clear(f);
        realm.clear(f);
        f.instruction(&Instruction::Else);
        output.copy_from(&called, f);
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
        let empty = s
            .reserve_gc_local(f)
            .initialize(s.array_type::<DisposableResourceTable>().fixed([], f), f);
        s.struct_type::<DisposableResourceStack>()
            .field(DisposableResourceStackSchema::RESOURCES)
            .write(&disposal.resources, GcOperand::reference(&empty, s), s, f);
        s.struct_type::<DisposableResourceStack>()
            .field(DisposableResourceStackSchema::ENTRY_COUNT)
            .write(&disposal.resources, GcOperand::i64(0), s, f);
        empty.clear(f);
        empty_arguments.clear(f);
        combined.clear(f);
        called.clear(f);
        receiver.clear(f);
        method.clear(f);
        value.clear(f);
        kind_word.clear(s, f);
        s.release_i32_local(index, f);
        s.release_i64_local(disposal.next_index, f);
        disposal.entries.clear(f);
        disposal.resources.clear(f);
        Ok(())
    }

    pub(crate) fn emit_disposable_stack_dispose(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_with_disposable_stack(f, |b, stack, output, exit, f| {
            let s = b.runtime_schema();
            let state = GcI32DomainLocal::new(s, DisposableStackState::Pending, f);
            s.struct_type::<DisposableStack>()
                .field(DisposableStackSchema::STATE)
                .read(stack, s, f)
                .store_domain(&state, f);
            state.load(f);
            f.instruction(&Instruction::I32Const(
                DisposableStackState::Disposed.word() as i32,
            ));
            f.instruction(&Instruction::I32Eq);
            b.emit_branch_if_to_target(exit, f);
            state.clear(s, f);
            let disposal = b.emit_begin_disposable_stack_disposal(stack, f);
            b.emit_consume_disposable_stack_disposal(disposal, output, f)
        })
    }
}
