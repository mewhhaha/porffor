//! `%DisposableStack%` synchronous resource lifecycle.
//!
//! See `docs/rust-rewrite/contracts/disposable-stack-synchronous-lifecycle.md`.

use super::super::*;
use crate::functions::NewTargetPrototypeFallback;

#[must_use = "a pending DisposableStack record must be consumed by the instance finalizer"]
struct PendingDisposableStackRecordLocal(u32);

#[must_use = "a transferred DisposableStack capability must be installed exactly once"]
struct TransferredDisposableStackCapabilityLocals {
    entries_ptr: u32,
    entries_len: u32,
    entries_cap: u32,
}

#[must_use = "an active DisposableStack disposal must be consumed by its LIFO walker"]
struct DisposableStackDisposalLocals {
    entries_ptr: u32,
    next_index: u32,
    has_error: u32,
    error_payload: u32,
    error_tag: u32,
}

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn emit_disposable_stack_constructor(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let new_target_payload_local = self.reserve_temp_local();
        let new_target_tag_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();
        let prototype_tag_local = self.reserve_temp_local();
        let stack_object_local = self.reserve_temp_local();

        self.compile_new_target_to_locals(
            new_target_payload_local,
            new_target_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(new_target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "DisposableStack constructor requires new",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_new_target_prototype_to_locals(
            DISPOSABLE_STACK_PROTOTYPE_GLOBAL_INDEX,
            NewTargetPrototypeFallback::CurrentGlobal,
            prototype_payload_local,
            prototype_tag_local,
            function,
        )?;
        self.emit_alloc_plain_object_with_prototype_and_tag(
            Some(prototype_payload_local),
            Some(prototype_tag_local),
            None,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(stack_object_local));

        let pending_record = self.emit_alloc_pending_disposable_stack_record(function)?;
        self.emit_finalize_disposable_stack_instance(stack_object_local, pending_record, function);

        self.release_temp_local(stack_object_local);
        self.release_temp_local(prototype_tag_local);
        self.release_temp_local(prototype_payload_local);
        self.release_temp_local(new_target_tag_local);
        self.release_temp_local(new_target_payload_local);
        Ok(())
    }

    pub(crate) fn emit_disposable_stack_use(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let stack_record_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let method_payload_local = self.reserve_temp_local();
        let method_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();

        self.emit_disposable_stack_record_from_receiver(stack_record_local, function)?;
        self.emit_disposable_stack_require_pending(stack_record_local, function)?;
        self.emit_builtin_arg_to_locals(0, value_payload_local, value_tag_local, function);

        // Sync-dispose nullish resources do not create an entry.
        self.emit_disposable_stack_is_nullish_i32(value_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_disposable_stack_return_value(
            value_payload_local,
            value_tag_local,
            true,
            function,
        );
        function.instruction(&Instruction::End);

        self.emit_is_heap_object_like_tag_i32(value_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_disposable_stack_type_error(
            "DisposableStack.prototype.use value is not an object",
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(
            self.strings.property_key_symbol_payload("Symbol.dispose"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_disposable_stack_get_method(
            value_payload_local,
            value_tag_local,
            key_local,
            method_payload_local,
            method_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(method_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_disposable_stack_type_error(
            "DisposableStack.prototype.use value is not disposable",
            function,
        )?;
        function.instruction(&Instruction::End);

        self.emit_disposable_stack_push_entry(
            stack_record_local,
            DisposableStackEntryKind::Use,
            value_payload_local,
            value_tag_local,
            method_payload_local,
            method_tag_local,
            function,
        )?;
        self.emit_disposable_stack_return_value(
            value_payload_local,
            value_tag_local,
            false,
            function,
        );

        self.release_temp_local(key_local);
        self.release_temp_local(method_tag_local);
        self.release_temp_local(method_payload_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(stack_record_local);
        Ok(())
    }

    pub(crate) fn emit_disposable_stack_adopt(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let stack_record_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let method_payload_local = self.reserve_temp_local();
        let method_tag_local = self.reserve_temp_local();

        self.emit_disposable_stack_record_from_receiver(stack_record_local, function)?;
        self.emit_disposable_stack_require_pending(stack_record_local, function)?;
        self.emit_builtin_arg_to_locals(0, value_payload_local, value_tag_local, function);
        self.emit_builtin_arg_to_locals(1, method_payload_local, method_tag_local, function);
        self.emit_is_callable_i32(method_tag_local, method_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_disposable_stack_type_error(
            "DisposableStack.prototype.adopt onDispose is not callable",
            function,
        )?;
        function.instruction(&Instruction::End);

        self.emit_disposable_stack_push_entry(
            stack_record_local,
            DisposableStackEntryKind::Adopt,
            value_payload_local,
            value_tag_local,
            method_payload_local,
            method_tag_local,
            function,
        )?;
        self.emit_disposable_stack_return_value(
            value_payload_local,
            value_tag_local,
            false,
            function,
        );

        self.release_temp_local(method_tag_local);
        self.release_temp_local(method_payload_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(stack_record_local);
        Ok(())
    }

    pub(crate) fn emit_disposable_stack_defer(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let stack_record_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let method_payload_local = self.reserve_temp_local();
        let method_tag_local = self.reserve_temp_local();

        self.emit_disposable_stack_record_from_receiver(stack_record_local, function)?;
        self.emit_disposable_stack_require_pending(stack_record_local, function)?;
        self.emit_builtin_arg_to_locals(0, method_payload_local, method_tag_local, function);
        self.emit_is_callable_i32(method_tag_local, method_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_disposable_stack_type_error(
            "DisposableStack.prototype.defer onDispose is not callable",
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_disposable_stack_push_entry(
            stack_record_local,
            DisposableStackEntryKind::Defer,
            value_payload_local,
            value_tag_local,
            method_payload_local,
            method_tag_local,
            function,
        )?;
        self.emit_disposable_stack_return_undefined(function);

        self.release_temp_local(method_tag_local);
        self.release_temp_local(method_payload_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(stack_record_local);
        Ok(())
    }

    pub(crate) fn emit_disposable_stack_move(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let stack_record_local = self.reserve_temp_local();
        let moved_object_local = self.reserve_temp_local();

        self.emit_disposable_stack_record_from_receiver(stack_record_local, function)?;
        self.emit_disposable_stack_require_pending(stack_record_local, function)?;
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(DISPOSABLE_STACK_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(moved_object_local));

        // Keep the destination record below the transfer locals so the temp
        // allocator's LIFO rule mirrors the semantic ownership transfer.
        let pending_record = self.emit_alloc_pending_disposable_stack_record(function)?;
        let transfer = self.emit_take_disposable_stack_capability(stack_record_local, function);
        let pending_record = self.emit_install_transferred_disposable_stack_capability(
            pending_record,
            transfer,
            function,
        );
        self.emit_finalize_disposable_stack_instance(moved_object_local, pending_record, function);

        self.release_temp_local(moved_object_local);
        self.release_temp_local(stack_record_local);
        Ok(())
    }

    pub(crate) fn emit_disposable_stack_dispose(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let stack_record_local = self.reserve_temp_local();
        let state_local = self.reserve_temp_local();

        self.emit_disposable_stack_record_from_receiver(stack_record_local, function)?;
        self.load_i64_to_local_from_offset(
            stack_record_local,
            HEAP_DISPOSABLE_STACK_STATE_OFFSET,
            state_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(
            DisposableStackState::Disposed.word() as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_disposable_stack_return_undefined(function);
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        let disposal = self.emit_begin_disposable_stack_disposal(stack_record_local, function);
        self.emit_consume_disposable_stack_disposal(disposal, function)?;

        self.release_temp_local(state_local);
        self.release_temp_local(stack_record_local);
        Ok(())
    }

    pub(crate) fn emit_disposable_stack_disposed_getter(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let stack_record_local = self.reserve_temp_local();
        let state_local = self.reserve_temp_local();

        self.emit_disposable_stack_record_from_receiver(stack_record_local, function)?;
        self.load_i64_to_local_from_offset(
            stack_record_local,
            HEAP_DISPOSABLE_STACK_STATE_OFFSET,
            state_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(
            DisposableStackState::Disposed.word() as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);

        self.release_temp_local(state_local);
        self.release_temp_local(stack_record_local);
        Ok(())
    }

    fn emit_alloc_pending_disposable_stack_record(
        &mut self,
        function: &mut Function,
    ) -> Result<PendingDisposableStackRecordLocal, EmitError> {
        let record_local = self.reserve_temp_local();

        self.emit_heap_alloc_const(HEAP_DISPOSABLE_STACK_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(record_local));
        self.store_i64_const_at_offset(
            record_local,
            HEAP_DISPOSABLE_STACK_STATE_OFFSET,
            DisposableStackState::Pending.word(),
            function,
        );
        for offset in [
            HEAP_DISPOSABLE_STACK_ENTRIES_PTR_OFFSET,
            HEAP_DISPOSABLE_STACK_ENTRIES_LEN_OFFSET,
            HEAP_DISPOSABLE_STACK_ENTRIES_CAP_OFFSET,
        ] {
            self.store_i64_const_at_offset(record_local, offset, 0, function);
        }

        Ok(PendingDisposableStackRecordLocal(record_local))
    }

    fn emit_finalize_disposable_stack_instance(
        &mut self,
        object_local: u32,
        record: PendingDisposableStackRecordLocal,
        function: &mut Function,
    ) {
        self.store_i64_const_at_offset(
            object_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            OBJECT_INTERNAL_BRAND_DISPOSABLE_STACK,
            function,
        );
        self.store_i64_local_at_offset(
            object_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            record.0,
            function,
        );
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);
        self.release_temp_local(record.0);
    }

    fn emit_take_disposable_stack_capability(
        &mut self,
        source_record_local: u32,
        function: &mut Function,
    ) -> TransferredDisposableStackCapabilityLocals {
        let entries_ptr = self.reserve_temp_local();
        let entries_len = self.reserve_temp_local();
        let entries_cap = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            source_record_local,
            HEAP_DISPOSABLE_STACK_ENTRIES_PTR_OFFSET,
            entries_ptr,
            function,
        );
        self.load_i64_to_local_from_offset(
            source_record_local,
            HEAP_DISPOSABLE_STACK_ENTRIES_LEN_OFFSET,
            entries_len,
            function,
        );
        self.load_i64_to_local_from_offset(
            source_record_local,
            HEAP_DISPOSABLE_STACK_ENTRIES_CAP_OFFSET,
            entries_cap,
            function,
        );
        for offset in [
            HEAP_DISPOSABLE_STACK_ENTRIES_PTR_OFFSET,
            HEAP_DISPOSABLE_STACK_ENTRIES_LEN_OFFSET,
            HEAP_DISPOSABLE_STACK_ENTRIES_CAP_OFFSET,
        ] {
            self.store_i64_const_at_offset(source_record_local, offset, 0, function);
        }
        self.store_i64_const_at_offset(
            source_record_local,
            HEAP_DISPOSABLE_STACK_STATE_OFFSET,
            DisposableStackState::Disposed.word(),
            function,
        );

        TransferredDisposableStackCapabilityLocals {
            entries_ptr,
            entries_len,
            entries_cap,
        }
    }

    fn emit_install_transferred_disposable_stack_capability(
        &mut self,
        record: PendingDisposableStackRecordLocal,
        transfer: TransferredDisposableStackCapabilityLocals,
        function: &mut Function,
    ) -> PendingDisposableStackRecordLocal {
        for (offset, local) in [
            (
                HEAP_DISPOSABLE_STACK_ENTRIES_PTR_OFFSET,
                transfer.entries_ptr,
            ),
            (
                HEAP_DISPOSABLE_STACK_ENTRIES_LEN_OFFSET,
                transfer.entries_len,
            ),
            (
                HEAP_DISPOSABLE_STACK_ENTRIES_CAP_OFFSET,
                transfer.entries_cap,
            ),
        ] {
            self.store_i64_local_at_offset(record.0, offset, local, function);
        }

        self.release_temp_local(transfer.entries_cap);
        self.release_temp_local(transfer.entries_len);
        self.release_temp_local(transfer.entries_ptr);
        record
    }

    /// The Pending -> Disposed transition precedes every callback. The record's
    /// visible length is detached while the non-Copy witness owns the walk.
    fn emit_begin_disposable_stack_disposal(
        &mut self,
        stack_record_local: u32,
        function: &mut Function,
    ) -> DisposableStackDisposalLocals {
        let entries_ptr = self.reserve_temp_local();
        let next_index = self.reserve_temp_local();
        let has_error = self.reserve_temp_local();
        let error_payload = self.reserve_temp_local();
        let error_tag = self.reserve_temp_local();

        self.store_i64_const_at_offset(
            stack_record_local,
            HEAP_DISPOSABLE_STACK_STATE_OFFSET,
            DisposableStackState::Disposed.word(),
            function,
        );
        self.load_i64_to_local_from_offset(
            stack_record_local,
            HEAP_DISPOSABLE_STACK_ENTRIES_PTR_OFFSET,
            entries_ptr,
            function,
        );
        self.load_i64_to_local_from_offset(
            stack_record_local,
            HEAP_DISPOSABLE_STACK_ENTRIES_LEN_OFFSET,
            next_index,
            function,
        );
        self.store_i64_const_at_offset(
            stack_record_local,
            HEAP_DISPOSABLE_STACK_ENTRIES_LEN_OFFSET,
            0,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(has_error));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(error_payload));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(error_tag));

        DisposableStackDisposalLocals {
            entries_ptr,
            next_index,
            has_error,
            error_payload,
            error_tag,
        }
    }

    fn emit_consume_disposable_stack_disposal(
        &mut self,
        disposal: DisposableStackDisposalLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let entry_local = self.reserve_temp_local();
        let kind_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let method_payload_local = self.reserve_temp_local();
        let method_tag_local = self.reserve_temp_local();
        let call_result_payload_local = self.reserve_temp_local();
        let call_result_tag_local = self.reserve_temp_local();
        let thrown_payload_local = self.reserve_temp_local();
        let thrown_tag_local = self.reserve_temp_local();
        let undefined_payload_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));

        // [0] is the loop and [1] its enclosing completion block.
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(disposal.next_index));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(disposal.next_index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(disposal.next_index));

        function.instruction(&Instruction::LocalGet(disposal.entries_ptr));
        function.instruction(&Instruction::LocalGet(disposal.next_index));
        function.instruction(&Instruction::I64Const(
            HEAP_DISPOSABLE_STACK_ENTRY_SIZE as i64,
        ));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        for (offset, local) in [
            (HEAP_DISPOSABLE_STACK_ENTRY_KIND_OFFSET, kind_local),
            (
                HEAP_DISPOSABLE_STACK_ENTRY_VALUE_PAYLOAD_OFFSET,
                value_payload_local,
            ),
            (
                HEAP_DISPOSABLE_STACK_ENTRY_VALUE_TAG_OFFSET,
                value_tag_local,
            ),
            (
                HEAP_DISPOSABLE_STACK_ENTRY_METHOD_PAYLOAD_OFFSET,
                method_payload_local,
            ),
            (
                HEAP_DISPOSABLE_STACK_ENTRY_METHOD_TAG_OFFSET,
                method_tag_local,
            ),
        ] {
            self.load_i64_to_local_from_offset(entry_local, offset, local, function);
        }

        // Every kind has exactly one call convention; the chain is derived
        // from ALL and the match is exhaustive over the convention domain.
        self.set_completion_kind(CompletionKind::Normal, function);
        let calls = DisposableStackEntryKind::ALL
            .into_iter()
            .map(|kind| (kind, kind.dispose_call()))
            .collect::<Vec<_>>();
        let last_call_index = calls.len() - 1;
        let no_arguments: [(u32, u32); 0] = [];
        let resource_argument = [(value_payload_local, value_tag_local)];
        for (index, (kind, call)) in calls.iter().enumerate() {
            if index < last_call_index {
                function.instruction(&Instruction::LocalGet(kind_local));
                function.instruction(&Instruction::I64Const(kind.word() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
            }
            let (this_payload, this_tag, arguments): (u32, u32, &[(u32, u32)]) = match call {
                DisposableStackDisposeCall::ResourceReceiver => {
                    (value_payload_local, value_tag_local, &no_arguments)
                }
                DisposableStackDisposeCall::UndefinedReceiverWithResourceArgument => (
                    undefined_payload_local,
                    undefined_tag_local,
                    &resource_argument,
                ),
                DisposableStackDisposeCall::UndefinedReceiverNoArguments => {
                    (undefined_payload_local, undefined_tag_local, &no_arguments)
                }
            };
            self.emit_function_or_proxy_call_leave_throw_completion(
                method_payload_local,
                method_tag_local,
                this_payload,
                this_tag,
                arguments,
                call_result_payload_local,
                call_result_tag_local,
                function,
            )?;
            if index < last_call_index {
                function.instruction(&Instruction::Else);
            }
        }
        for _ in 0..last_call_index {
            function.instruction(&Instruction::End);
        }

        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalSet(thrown_payload_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalSet(thrown_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_disposable_stack_record_error(
            &disposal,
            thrown_payload_local,
            thrown_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_disposable_stack_return_undefined(function);
        function.instruction(&Instruction::LocalGet(disposal.has_error));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(disposal.error_payload));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(disposal.error_tag));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Throw, function);
        function.instruction(&Instruction::End);

        self.release_temp_local(undefined_tag_local);
        self.release_temp_local(undefined_payload_local);
        self.release_temp_local(thrown_tag_local);
        self.release_temp_local(thrown_payload_local);
        self.release_temp_local(call_result_tag_local);
        self.release_temp_local(call_result_payload_local);
        self.release_temp_local(method_tag_local);
        self.release_temp_local(method_payload_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(kind_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(disposal.error_tag);
        self.release_temp_local(disposal.error_payload);
        self.release_temp_local(disposal.has_error);
        self.release_temp_local(disposal.next_index);
        self.release_temp_local(disposal.entries_ptr);
        Ok(())
    }

    fn emit_disposable_stack_record_error(
        &mut self,
        disposal: &DisposableStackDisposalLocals,
        new_error_payload_local: u32,
        new_error_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let prototype_local = self.reserve_temp_local();
        let combined_payload_local = self.reserve_temp_local();
        let combined_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(new_error_payload_local));
        function.instruction(&Instruction::LocalSet(combined_payload_local));
        function.instruction(&Instruction::LocalGet(new_error_tag_local));
        function.instruction(&Instruction::LocalSet(combined_tag_local));
        function.instruction(&Instruction::LocalGet(disposal.has_error));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(
            SUPPRESSED_ERROR_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_local));
        self.emit_alloc_suppressed_error_instance_from_locals(
            None,
            new_error_payload_local,
            new_error_tag_local,
            disposal.error_payload,
            disposal.error_tag,
            prototype_local,
            combined_payload_local,
            combined_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(combined_payload_local));
        function.instruction(&Instruction::LocalSet(disposal.error_payload));
        function.instruction(&Instruction::LocalGet(combined_tag_local));
        function.instruction(&Instruction::LocalSet(disposal.error_tag));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(disposal.has_error));

        self.release_temp_local(combined_tag_local);
        self.release_temp_local(combined_payload_local);
        self.release_temp_local(prototype_local);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_disposable_stack_push_entry(
        &mut self,
        stack_record_local: u32,
        kind: DisposableStackEntryKind,
        value_payload_local: u32,
        value_tag_local: u32,
        method_payload_local: u32,
        method_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let entries_ptr_local = self.reserve_temp_local();
        let entries_len_local = self.reserve_temp_local();
        let entries_cap_local = self.reserve_temp_local();
        let new_cap_local = self.reserve_temp_local();
        let allocation_size_local = self.reserve_temp_local();
        let new_entries_ptr_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let old_entry_local = self.reserve_temp_local();
        let new_entry_local = self.reserve_temp_local();
        let copied_value_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            stack_record_local,
            HEAP_DISPOSABLE_STACK_ENTRIES_PTR_OFFSET,
            entries_ptr_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            stack_record_local,
            HEAP_DISPOSABLE_STACK_ENTRIES_LEN_OFFSET,
            entries_len_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            stack_record_local,
            HEAP_DISPOSABLE_STACK_ENTRIES_CAP_OFFSET,
            entries_cap_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(entries_len_local));
        function.instruction(&Instruction::LocalGet(entries_cap_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(entries_cap_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(MIN_HEAP_CAPACITY as i64));
        function.instruction(&Instruction::LocalSet(new_cap_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(entries_cap_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(new_cap_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(new_cap_local));
        function.instruction(&Instruction::I64Const(
            HEAP_DISPOSABLE_STACK_ENTRY_SIZE as i64,
        ));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(allocation_size_local));
        self.emit_heap_alloc_from_local(allocation_size_local, function)?;
        function.instruction(&Instruction::LocalSet(new_entries_ptr_local));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(entries_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        for (base_local, indexed_entry_local) in [
            (entries_ptr_local, old_entry_local),
            (new_entries_ptr_local, new_entry_local),
        ] {
            function.instruction(&Instruction::LocalGet(base_local));
            function.instruction(&Instruction::LocalGet(index_local));
            function.instruction(&Instruction::I64Const(
                HEAP_DISPOSABLE_STACK_ENTRY_SIZE as i64,
            ));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(indexed_entry_local));
        }
        for offset in [
            HEAP_DISPOSABLE_STACK_ENTRY_KIND_OFFSET,
            HEAP_DISPOSABLE_STACK_ENTRY_VALUE_TAG_OFFSET,
            HEAP_DISPOSABLE_STACK_ENTRY_VALUE_PAYLOAD_OFFSET,
            HEAP_DISPOSABLE_STACK_ENTRY_METHOD_TAG_OFFSET,
            HEAP_DISPOSABLE_STACK_ENTRY_METHOD_PAYLOAD_OFFSET,
        ] {
            self.load_i64_to_local_from_offset(
                old_entry_local,
                offset,
                copied_value_local,
                function,
            );
            self.store_i64_local_at_offset(new_entry_local, offset, copied_value_local, function);
        }
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.store_i64_local_at_offset(
            stack_record_local,
            HEAP_DISPOSABLE_STACK_ENTRIES_PTR_OFFSET,
            new_entries_ptr_local,
            function,
        );
        self.store_i64_local_at_offset(
            stack_record_local,
            HEAP_DISPOSABLE_STACK_ENTRIES_CAP_OFFSET,
            new_cap_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(new_entries_ptr_local));
        function.instruction(&Instruction::LocalSet(entries_ptr_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(entries_ptr_local));
        function.instruction(&Instruction::LocalGet(entries_len_local));
        function.instruction(&Instruction::I64Const(
            HEAP_DISPOSABLE_STACK_ENTRY_SIZE as i64,
        ));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_const_at_offset(
            entry_local,
            HEAP_DISPOSABLE_STACK_ENTRY_KIND_OFFSET,
            kind.word(),
            function,
        );
        for (offset, local) in [
            (
                HEAP_DISPOSABLE_STACK_ENTRY_VALUE_PAYLOAD_OFFSET,
                value_payload_local,
            ),
            (
                HEAP_DISPOSABLE_STACK_ENTRY_VALUE_TAG_OFFSET,
                value_tag_local,
            ),
            (
                HEAP_DISPOSABLE_STACK_ENTRY_METHOD_PAYLOAD_OFFSET,
                method_payload_local,
            ),
            (
                HEAP_DISPOSABLE_STACK_ENTRY_METHOD_TAG_OFFSET,
                method_tag_local,
            ),
        ] {
            self.store_i64_local_at_offset(entry_local, offset, local, function);
        }
        function.instruction(&Instruction::LocalGet(entries_len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entries_len_local));
        self.store_i64_local_at_offset(
            stack_record_local,
            HEAP_DISPOSABLE_STACK_ENTRIES_LEN_OFFSET,
            entries_len_local,
            function,
        );

        self.release_temp_local(entry_local);
        self.release_temp_local(copied_value_local);
        self.release_temp_local(new_entry_local);
        self.release_temp_local(old_entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(new_entries_ptr_local);
        self.release_temp_local(allocation_size_local);
        self.release_temp_local(new_cap_local);
        self.release_temp_local(entries_cap_local);
        self.release_temp_local(entries_len_local);
        self.release_temp_local(entries_ptr_local);
        Ok(())
    }

    fn emit_disposable_stack_get_method(
        &mut self,
        value_payload_local: u32,
        value_tag_local: u32,
        key_local: u32,
        method_payload_local: u32,
        method_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_object_read_without_throw_propagation(
            value_payload_local,
            value_tag_local,
            value_payload_local,
            value_tag_local,
            key_local,
            method_payload_local,
            method_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_disposable_stack_is_nullish_i32(method_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(method_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(method_tag_local));
        function.instruction(&Instruction::Else);
        self.emit_is_callable_i32(method_tag_local, method_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_disposable_stack_type_error(
            "DisposableStack.prototype.use dispose method is not callable",
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn emit_disposable_stack_record_from_receiver(
        &mut self,
        stack_record_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let receiver_brand_local = self.reserve_temp_local();

        self.compile_this_to_locals(receiver_payload_local, receiver_tag_local, function)?;
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_disposable_stack_type_error(
            "DisposableStack method receiver is not an object",
            function,
        )?;
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            receiver_brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(receiver_brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_DISPOSABLE_STACK as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_disposable_stack_type_error(
            "DisposableStack method receiver does not have [[DisposableState]]",
            function,
        )?;
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            stack_record_local,
            function,
        );

        self.release_temp_local(receiver_brand_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    fn emit_disposable_stack_require_pending(
        &mut self,
        stack_record_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let state_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            stack_record_local,
            HEAP_DISPOSABLE_STACK_STATE_OFFSET,
            state_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(
            DisposableStackState::Disposed.word() as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_error(
            REFERENCE_ERROR_NAME,
            "DisposableStack is already disposed",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.release_temp_local(state_local);
        Ok(())
    }

    fn emit_disposable_stack_is_nullish_i32(&mut self, tag_local: u32, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
    }

    fn emit_disposable_stack_return_value(
        &mut self,
        payload_local: u32,
        tag_local: u32,
        return_now: bool,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);
        if return_now {
            self.emit_return_current_completion(function);
        }
    }

    fn emit_disposable_stack_return_undefined(&mut self, function: &mut Function) {
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);
    }

    fn emit_disposable_stack_type_error(
        &mut self,
        message: &'static str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_throw_current_function_realm_type_error(
            message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        Ok(())
    }
}
