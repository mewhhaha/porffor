use super::super::*;
use super::binary_data::{TypedArrayViewLocals, TypedArrayWitnessUse};
use lila_runtime::AgentHostOperation;

mod wait_async_result;

enum AtomicsBuiltin {
    Add,
    And,
    CompareExchange,
    Exchange,
    IsLockFree,
    Load,
    Notify,
    Or,
    Pause,
    Store,
    Sub,
    Wait,
    WaitAsync,
    Xor,
}

pub(super) const ATOMICS_PUBLICATION_ORDER: [StandardBuiltinId; 14] = [
    StandardBuiltinId::AtomicsAdd,
    StandardBuiltinId::AtomicsAnd,
    StandardBuiltinId::AtomicsCompareExchange,
    StandardBuiltinId::AtomicsExchange,
    StandardBuiltinId::AtomicsLoad,
    StandardBuiltinId::AtomicsNotify,
    StandardBuiltinId::AtomicsOr,
    StandardBuiltinId::AtomicsPause,
    StandardBuiltinId::AtomicsStore,
    StandardBuiltinId::AtomicsSub,
    StandardBuiltinId::AtomicsWait,
    StandardBuiltinId::AtomicsWaitAsync,
    StandardBuiltinId::AtomicsXor,
    StandardBuiltinId::AtomicsIsLockFree,
];

enum AtomicsIntegerOperation {
    Load,
    Add,
    And,
    CompareExchange,
    Exchange,
    Or,
    Store,
    Sub,
    Xor,
}

impl AtomicsIntegerOperation {
    fn value_arg_count(&self) -> u8 {
        match self {
            Self::Load => 0,
            Self::CompareExchange => 2,
            Self::Add
            | Self::And
            | Self::Exchange
            | Self::Or
            | Self::Store
            | Self::Sub
            | Self::Xor => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AtomicsRmwOperation {
    Add,
    And,
    Exchange,
    Or,
    Sub,
    Xor,
}

enum AtomicsIntegerElementKindRequirement {
    AnyInteger,
    Waitable,
}

#[must_use = "an Atomics integer element-kind local must be validated"]
struct PendingAtomicsIntegerElementKindLocal(u32);

#[must_use = "a validated Atomics integer element-kind local must be released"]
struct ValidatedAtomicsIntegerElementKindLocal(u32);

impl ValidatedAtomicsIntegerElementKindLocal {
    const fn local(&self) -> u32 {
        self.0
    }

    const fn into_local(self) -> u32 {
        self.0
    }
}

enum AtomicsWaitOutcome {
    Ok,
    NotEqual,
    TimedOut,
}

impl AtomicsWaitOutcome {
    fn spelling(&self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::NotEqual => "not-equal",
            Self::TimedOut => "timed-out",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AtomicsWaitAsyncTimeoutCheckpointMode {
    Drain,
    Poll,
}

impl<'a> FunctionBuilder<'a> {
    fn emit_validate_atomics_integer_element_kind(
        &mut self,
        typed_array_payload_local: u32,
        pending: PendingAtomicsIntegerElementKindLocal,
        requirement: AtomicsIntegerElementKindRequirement,
        type_error_message: &str,
        function: &mut Function,
    ) -> Result<ValidatedAtomicsIntegerElementKindLocal, EmitError> {
        let PendingAtomicsIntegerElementKindLocal(element_kind_local) = pending;
        self.load_i64_to_local_from_offset(
            typed_array_payload_local,
            HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET,
            element_kind_local,
            function,
        );
        match requirement {
            AtomicsIntegerElementKindRequirement::AnyInteger => {
                self.emit_atomics_friendly_element_kind_i32(element_kind_local, function);
            }
            AtomicsIntegerElementKindRequirement::Waitable => {
                function.instruction(&Instruction::LocalGet(element_kind_local));
                function.instruction(&Instruction::I64Const(5));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::LocalGet(element_kind_local));
                function.instruction(&Instruction::I64Const(10));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I32Or);
            }
        }
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_current_function_realm_type_error(
            type_error_message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        Ok(ValidatedAtomicsIntegerElementKindLocal(element_kind_local))
    }

    pub(super) fn emit_atomics_add_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_atomics_builtin(AtomicsBuiltin::Add, function)
    }

    pub(super) fn emit_atomics_and_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_atomics_builtin(AtomicsBuiltin::And, function)
    }

    pub(super) fn emit_atomics_compare_exchange_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_atomics_builtin(AtomicsBuiltin::CompareExchange, function)
    }

    pub(super) fn emit_atomics_exchange_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_atomics_builtin(AtomicsBuiltin::Exchange, function)
    }

    pub(super) fn emit_atomics_is_lock_free_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_atomics_builtin(AtomicsBuiltin::IsLockFree, function)
    }

    pub(super) fn emit_atomics_load_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_atomics_builtin(AtomicsBuiltin::Load, function)
    }

    pub(super) fn emit_atomics_notify_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_atomics_builtin(AtomicsBuiltin::Notify, function)
    }

    pub(super) fn emit_atomics_or_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_atomics_builtin(AtomicsBuiltin::Or, function)
    }

    pub(super) fn emit_atomics_pause_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_atomics_builtin(AtomicsBuiltin::Pause, function)
    }

    pub(super) fn emit_atomics_store_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_atomics_builtin(AtomicsBuiltin::Store, function)
    }

    pub(super) fn emit_atomics_sub_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_atomics_builtin(AtomicsBuiltin::Sub, function)
    }

    pub(super) fn emit_atomics_wait_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_atomics_builtin(AtomicsBuiltin::Wait, function)
    }

    pub(super) fn emit_atomics_wait_async_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_atomics_builtin(AtomicsBuiltin::WaitAsync, function)
    }

    pub(super) fn emit_atomics_xor_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_atomics_builtin(AtomicsBuiltin::Xor, function)
    }

    fn emit_atomics_builtin(
        &mut self,
        builtin: AtomicsBuiltin,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match builtin {
            AtomicsBuiltin::Add => self.emit_atomics_add(function),
            AtomicsBuiltin::And => self.emit_atomics_and(function),
            AtomicsBuiltin::CompareExchange => self.emit_atomics_compare_exchange(function),
            AtomicsBuiltin::Exchange => self.emit_atomics_exchange(function),
            AtomicsBuiltin::IsLockFree => self.emit_atomics_is_lock_free(function),
            AtomicsBuiltin::Load => self.emit_atomics_load(function),
            AtomicsBuiltin::Notify => self.emit_atomics_notify(function),
            AtomicsBuiltin::Or => self.emit_atomics_or(function),
            AtomicsBuiltin::Pause => self.emit_atomics_pause(function),
            AtomicsBuiltin::Store => self.emit_atomics_store(function),
            AtomicsBuiltin::Sub => self.emit_atomics_sub(function),
            AtomicsBuiltin::Wait => self.emit_atomics_wait(function),
            AtomicsBuiltin::WaitAsync => self.emit_atomics_wait_async(function),
            AtomicsBuiltin::Xor => self.emit_atomics_xor(function),
        }
    }

    fn emit_atomics_is_lock_free(&mut self, function: &mut Function) -> Result<(), EmitError> {
        let size_payload_local = self.reserve_temp_local();
        let size_tag_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, size_payload_local, size_tag_local, function);
        self.emit_value_to_number_payload(size_tag_local, size_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(size_payload_local));
        self.emit_return_current_completion_if_throw(function);

        function.instruction(&Instruction::LocalGet(size_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(4.0)));
        function.instruction(&Instruction::F64Ge);
        function.instruction(&Instruction::LocalGet(size_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(5.0)));
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(size_tag_local);
        self.release_temp_local(size_payload_local);
        Ok(())
    }

    fn emit_atomics_load(&mut self, function: &mut Function) -> Result<(), EmitError> {
        self.emit_atomics_integer_operation(AtomicsIntegerOperation::Load, function)
    }

    fn emit_atomics_add(&mut self, function: &mut Function) -> Result<(), EmitError> {
        self.emit_atomics_integer_operation(AtomicsIntegerOperation::Add, function)
    }

    fn emit_atomics_and(&mut self, function: &mut Function) -> Result<(), EmitError> {
        self.emit_atomics_integer_operation(AtomicsIntegerOperation::And, function)
    }

    fn emit_atomics_compare_exchange(&mut self, function: &mut Function) -> Result<(), EmitError> {
        self.emit_atomics_integer_operation(AtomicsIntegerOperation::CompareExchange, function)
    }

    fn emit_atomics_exchange(&mut self, function: &mut Function) -> Result<(), EmitError> {
        self.emit_atomics_integer_operation(AtomicsIntegerOperation::Exchange, function)
    }

    fn emit_atomics_or(&mut self, function: &mut Function) -> Result<(), EmitError> {
        self.emit_atomics_integer_operation(AtomicsIntegerOperation::Or, function)
    }

    fn emit_atomics_store(&mut self, function: &mut Function) -> Result<(), EmitError> {
        self.emit_atomics_integer_operation(AtomicsIntegerOperation::Store, function)
    }

    fn emit_atomics_sub(&mut self, function: &mut Function) -> Result<(), EmitError> {
        self.emit_atomics_integer_operation(AtomicsIntegerOperation::Sub, function)
    }

    fn emit_atomics_xor(&mut self, function: &mut Function) -> Result<(), EmitError> {
        self.emit_atomics_integer_operation(AtomicsIntegerOperation::Xor, function)
    }

    fn emit_atomics_pause(&mut self, function: &mut Function) -> Result<(), EmitError> {
        let iteration_payload_local = self.reserve_temp_local();
        let iteration_tag_local = self.reserve_temp_local();
        let valid_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, iteration_payload_local, iteration_tag_local, function);

        function.instruction(&Instruction::LocalGet(iteration_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(valid_local));

        function.instruction(&Instruction::LocalGet(iteration_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(iteration_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::LocalGet(iteration_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::LocalGet(iteration_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Abs);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(valid_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Atomics.pause iterationNumber must be a finite integral Number",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(valid_local);
        self.release_temp_local(iteration_tag_local);
        self.release_temp_local(iteration_payload_local);

        Ok(())
    }

    fn emit_atomics_notify(&mut self, function: &mut Function) -> Result<(), EmitError> {
        let typed_array_payload_local = self.reserve_temp_local();
        let typed_array_tag_local = self.reserve_temp_local();
        let index_payload_local = self.reserve_temp_local();
        let index_tag_local = self.reserve_temp_local();
        let count_payload_local = self.reserve_temp_local();
        let count_tag_local = self.reserve_temp_local();
        let typed_array_brand_local = self.reserve_temp_local();
        let buffer_payload_local = self.reserve_temp_local();
        let buffer_tag_local = self.reserve_temp_local();
        let data_ptr_local = self.reserve_temp_local();
        let byte_offset_local = self.reserve_temp_local();
        let stored_byte_length_local = self.reserve_temp_local();
        let bytes_per_element_local = self.reserve_temp_local();
        let element_length_local = self.reserve_temp_local();
        let element_kind_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let count_local = self.reserve_temp_local();
        let address_local = self.reserve_temp_local();
        let waiter_local = self.reserve_temp_local();
        let waiter_next_local = self.reserve_temp_local();
        let previous_waiter_local = self.reserve_temp_local();
        let waiter_state_local = self.reserve_temp_local();
        let waiter_address_local = self.reserve_temp_local();
        let claimed_local = self.reserve_temp_local();
        let promise_record_local = self.reserve_temp_local();
        let outcome_tag_local = self.reserve_temp_local();
        let deadline_nanos_local = self.reserve_temp_local();
        let monotonic_now_local = self.reserve_temp_local();
        let waiter_host_id_local = self.reserve_temp_local();
        let agent_call_function_index = self.functions.agent_call_import_function_index();

        self.emit_builtin_arg_to_locals(
            0,
            typed_array_payload_local,
            typed_array_tag_local,
            function,
        );
        self.emit_builtin_arg_to_locals(1, index_payload_local, index_tag_local, function);
        self.emit_builtin_arg_to_locals(2, count_payload_local, count_tag_local, function);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(typed_array_brand_local));
        function.instruction(&Instruction::LocalGet(typed_array_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            typed_array_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            typed_array_brand_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(typed_array_brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TYPED_ARRAY as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Atomics.notify requires an Int32Array or BigInt64Array",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_load_typed_array_private_state(
            typed_array_payload_local,
            buffer_payload_local,
            byte_offset_local,
            stored_byte_length_local,
            bytes_per_element_local,
            function,
        );
        let typed_array_view = TypedArrayViewLocals::new(
            typed_array_payload_local,
            buffer_payload_local,
            byte_offset_local,
            stored_byte_length_local,
            bytes_per_element_local,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(buffer_tag_local));
        self.emit_require_array_buffer_or_shared_array_buffer(
            buffer_payload_local,
            buffer_tag_local,
            "Atomics.notify requires an Int32Array or BigInt64Array",
            function,
        )?;

        self.load_i64_to_local_from_offset(
            typed_array_payload_local,
            HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET,
            element_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(5));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_current_function_realm_type_error(
            "Atomics.notify requires an Int32Array or BigInt64Array",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_load_array_buffer_data(buffer_payload_local, data_ptr_local, function);
        function.instruction(&Instruction::LocalGet(data_ptr_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "TypedArray backing buffer is detached",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_typed_array_witness(
            &typed_array_view,
            TypedArrayWitnessUse::ValidatedMethodEntry {
                length_local: element_length_local,
            },
            function,
        )?;

        self.emit_value_to_number_payload(index_tag_local, index_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(index_payload_local));
        self.emit_return_current_completion_if_throw(function);
        self.emit_to_index_from_number_payload(
            index_payload_local,
            index_local,
            "Atomics.notify index out of range",
            function,
        )?;
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(element_length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Atomics.notify index out of range",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(i32::MAX as i64));
        function.instruction(&Instruction::LocalSet(count_local));
        function.instruction(&Instruction::LocalGet(count_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_number_payload(count_tag_local, count_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(count_payload_local));
        self.emit_return_current_completion_if_throw(function);
        self.emit_to_integer_or_infinity_number_payload_from_number_payload(
            count_payload_local,
            count_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(count_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Le);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(count_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(count_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(i32::MAX as f64)));
        function.instruction(&Instruction::F64Min);
        function.instruction(&Instruction::I32TruncSatF64U);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(count_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(data_ptr_local));
        function.instruction(&Instruction::LocalGet(byte_offset_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(bytes_per_element_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(address_local));

        if let Some(agent_call_function_index) = agent_call_function_index {
            function.instruction(&Instruction::I64Const(
                AgentHostOperation::NotifyAsyncWaiters.wire(),
            ));
            function.instruction(&Instruction::LocalGet(address_local));
            function.instruction(&Instruction::LocalGet(count_local));
            function.instruction(&Instruction::Call(agent_call_function_index));
            function.instruction(&Instruction::LocalSet(claimed_local));
        } else {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(claimed_local));
        }
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(previous_waiter_local));
        if let Some(monotonic_clock_function_index) =
            self.functions.monotonic_clock_nanos_import_function_index()
        {
            function.instruction(&Instruction::Call(monotonic_clock_function_index));
            function.instruction(&Instruction::LocalSet(monotonic_now_local));
        }
        function.instruction(&Instruction::GlobalGet(
            ATOMICS_ASYNC_WAITER_ACTIVE_LIST_HEAD_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(waiter_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(waiter_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        self.load_i64_to_local_from_offset(
            waiter_local,
            HEAP_ATOMICS_ASYNC_WAITER_NEXT_OFFSET,
            waiter_next_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            waiter_local,
            HEAP_ATOMICS_ASYNC_WAITER_STATE_OFFSET,
            waiter_state_local,
            function,
        );
        if agent_call_function_index.is_none()
            && self
                .functions
                .monotonic_clock_nanos_import_function_index()
                .is_some()
        {
            function.instruction(&Instruction::LocalGet(waiter_state_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Else);
            self.load_i64_to_local_from_offset(
                waiter_local,
                HEAP_ATOMICS_ASYNC_WAITER_DEADLINE_NANOS_OFFSET,
                deadline_nanos_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(deadline_nanos_local));
            function.instruction(&Instruction::I64Const(i64::MAX));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::LocalGet(deadline_nanos_local));
            function.instruction(&Instruction::LocalGet(monotonic_now_local));
            function.instruction(&Instruction::I64LeU);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.store_i64_const_at_offset(
                waiter_local,
                HEAP_ATOMICS_ASYNC_WAITER_STATE_OFFSET,
                0,
                function,
            );
            self.load_i64_to_local_from_offset(
                waiter_local,
                HEAP_ATOMICS_ASYNC_WAITER_PROMISE_RECORD_OFFSET,
                promise_record_local,
                function,
            );
            function.instruction(&Instruction::I64Const(
                self.strings
                    .payload(AtomicsWaitOutcome::TimedOut.spelling()),
            ));
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
            function.instruction(&Instruction::LocalSet(outcome_tag_local));
            self.emit_settle_promise_record(
                promise_record_local,
                PromiseSettlement::Fulfill,
                self.scratch_local,
                outcome_tag_local,
                function,
            )?;
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(waiter_state_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(waiter_state_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        if agent_call_function_index.is_some() {
            function.instruction(&Instruction::I32Const(1));
        } else {
            function.instruction(&Instruction::LocalGet(claimed_local));
            function.instruction(&Instruction::LocalGet(count_local));
            function.instruction(&Instruction::I64LtU);
        }
        function.instruction(&Instruction::If(BlockType::Empty));
        if agent_call_function_index.is_some() {
            function.instruction(&Instruction::I32Const(1));
        } else {
            self.load_i64_to_local_from_offset(
                waiter_local,
                HEAP_ATOMICS_ASYNC_WAITER_ADDRESS_OFFSET,
                waiter_address_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(waiter_address_local));
            function.instruction(&Instruction::LocalGet(address_local));
            function.instruction(&Instruction::I64Eq);
        }
        function.instruction(&Instruction::If(BlockType::Empty));
        if let Some(agent_call_function_index) = agent_call_function_index {
            self.load_i64_to_local_from_offset(
                waiter_local,
                HEAP_ATOMICS_ASYNC_WAITER_HOST_ID_OFFSET,
                waiter_host_id_local,
                function,
            );
            function.instruction(&Instruction::I64Const(
                AgentHostOperation::PollAsyncWaiter.wire(),
            ));
            function.instruction(&Instruction::LocalGet(waiter_host_id_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::Call(agent_call_function_index));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Eq);
        } else {
            function.instruction(&Instruction::LocalGet(waiter_local));
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::I32Load(Self::memarg32(
                HEAP_ATOMICS_ASYNC_WAITER_STATE_OFFSET,
            )));
            function.instruction(&Instruction::I32Const(1));
            function.instruction(&Instruction::I32Eq);
        }
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(waiter_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::I32Store(Self::memarg32(
            HEAP_ATOMICS_ASYNC_WAITER_STATE_OFFSET,
        )));
        self.load_i64_to_local_from_offset(
            waiter_local,
            HEAP_ATOMICS_ASYNC_WAITER_PROMISE_RECORD_OFFSET,
            promise_record_local,
            function,
        );
        function.instruction(&Instruction::I64Const(
            self.strings.payload(AtomicsWaitOutcome::Ok.spelling()),
        ));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(outcome_tag_local));
        self.emit_settle_promise_record(
            promise_record_local,
            PromiseSettlement::Fulfill,
            self.scratch_local,
            outcome_tag_local,
            function,
        )?;
        if agent_call_function_index.is_none() {
            function.instruction(&Instruction::LocalGet(claimed_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(claimed_local));
        }
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(waiter_state_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(waiter_state_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(previous_waiter_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(waiter_next_local));
        function.instruction(&Instruction::GlobalSet(
            ATOMICS_ASYNC_WAITER_ACTIVE_LIST_HEAD_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::Else);
        self.store_i64_local_at_offset(
            previous_waiter_local,
            HEAP_ATOMICS_ASYNC_WAITER_NEXT_OFFSET,
            waiter_next_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(waiter_local));
        function.instruction(&Instruction::LocalSet(previous_waiter_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(waiter_next_local));
        function.instruction(&Instruction::LocalSet(waiter_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(count_local));
        function.instruction(&Instruction::LocalGet(claimed_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::MemoryAtomicNotify(Self::shared_memarg32(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalGet(claimed_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(waiter_host_id_local);
        self.release_temp_local(monotonic_now_local);
        self.release_temp_local(deadline_nanos_local);
        self.release_temp_local(outcome_tag_local);
        self.release_temp_local(promise_record_local);
        self.release_temp_local(claimed_local);
        self.release_temp_local(waiter_address_local);
        self.release_temp_local(waiter_state_local);
        self.release_temp_local(previous_waiter_local);
        self.release_temp_local(waiter_next_local);
        self.release_temp_local(waiter_local);
        self.release_temp_local(address_local);
        self.release_temp_local(count_local);
        self.release_temp_local(index_local);
        self.release_temp_local(element_kind_local);
        self.release_temp_local(element_length_local);
        self.release_temp_local(bytes_per_element_local);
        self.release_temp_local(stored_byte_length_local);
        self.release_temp_local(byte_offset_local);
        self.release_temp_local(data_ptr_local);
        self.release_temp_local(buffer_tag_local);
        self.release_temp_local(buffer_payload_local);
        self.release_temp_local(typed_array_brand_local);
        self.release_temp_local(count_tag_local);
        self.release_temp_local(count_payload_local);
        self.release_temp_local(index_tag_local);
        self.release_temp_local(index_payload_local);
        self.release_temp_local(typed_array_tag_local);
        self.release_temp_local(typed_array_payload_local);

        Ok(())
    }

    fn emit_atomics_require_agent_can_suspend(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::Call(
            HOST_AGENT_CAN_SUSPEND_IMPORT_FUNCTION_INDEX,
        ));
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Atomics.wait cannot suspend the current agent",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn emit_atomics_wait_return_string(
        &mut self,
        outcome: AtomicsWaitOutcome,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(
            self.strings.payload(outcome.spelling()),
        ));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
    }

    fn emit_atomics_wait_async(&mut self, function: &mut Function) -> Result<(), EmitError> {
        let typed_array_payload_local = self.reserve_temp_local();
        let typed_array_tag_local = self.reserve_temp_local();
        let index_payload_local = self.reserve_temp_local();
        let index_tag_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let timeout_payload_local = self.reserve_temp_local();
        let timeout_tag_local = self.reserve_temp_local();
        let typed_array_brand_local = self.reserve_temp_local();
        let buffer_payload_local = self.reserve_temp_local();
        let buffer_tag_local = self.reserve_temp_local();
        let buffer_brand_local = self.reserve_temp_local();
        let data_ptr_local = self.reserve_temp_local();
        let byte_offset_local = self.reserve_temp_local();
        let stored_byte_length_local = self.reserve_temp_local();
        let bytes_per_element_local = self.reserve_temp_local();
        let element_length_local = self.reserve_temp_local();
        let pending_element_kind = PendingAtomicsIntegerElementKindLocal(self.reserve_temp_local());
        let index_local = self.reserve_temp_local();
        let address_local = self.reserve_temp_local();
        let expected_raw_local = self.reserve_temp_local();
        let current_raw_local = self.reserve_temp_local();
        let deadline_nanos_local = self.reserve_temp_local();
        let timeout_nanos_local = self.reserve_temp_local();
        let monotonic_now_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(
            0,
            typed_array_payload_local,
            typed_array_tag_local,
            function,
        );
        self.emit_builtin_arg_to_locals(1, index_payload_local, index_tag_local, function);
        self.emit_builtin_arg_to_locals(2, value_payload_local, value_tag_local, function);
        self.emit_builtin_arg_to_locals(3, timeout_payload_local, timeout_tag_local, function);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(typed_array_brand_local));
        function.instruction(&Instruction::LocalGet(typed_array_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            typed_array_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            typed_array_brand_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(typed_array_brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TYPED_ARRAY as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Atomics.waitAsync requires a shared Int32Array or BigInt64Array",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_load_typed_array_private_state(
            typed_array_payload_local,
            buffer_payload_local,
            byte_offset_local,
            stored_byte_length_local,
            bytes_per_element_local,
            function,
        );
        let typed_array_view = TypedArrayViewLocals::new(
            typed_array_payload_local,
            buffer_payload_local,
            byte_offset_local,
            stored_byte_length_local,
            bytes_per_element_local,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(buffer_tag_local));
        self.emit_require_array_buffer_or_shared_array_buffer(
            buffer_payload_local,
            buffer_tag_local,
            "Atomics.waitAsync requires a shared Int32Array or BigInt64Array",
            function,
        )?;

        let element_kind = self.emit_validate_atomics_integer_element_kind(
            typed_array_payload_local,
            pending_element_kind,
            AtomicsIntegerElementKindRequirement::Waitable,
            "Atomics.waitAsync requires a shared Int32Array or BigInt64Array",
            function,
        )?;

        self.load_i64_to_local_from_offset(
            buffer_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            buffer_brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(buffer_brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_SHARED_ARRAY_BUFFER as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Atomics.waitAsync requires a shared Int32Array or BigInt64Array",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_load_array_buffer_data(buffer_payload_local, data_ptr_local, function);
        function.instruction(&Instruction::LocalGet(data_ptr_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "TypedArray backing buffer is detached",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_typed_array_witness(
            &typed_array_view,
            TypedArrayWitnessUse::ValidatedMethodEntry {
                length_local: element_length_local,
            },
            function,
        )?;

        self.emit_value_to_number_payload(index_tag_local, index_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(index_payload_local));
        self.emit_return_current_completion_if_throw(function);
        self.emit_to_index_from_number_payload(
            index_payload_local,
            index_local,
            "Atomics.waitAsync index out of range",
            function,
        )?;
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(element_length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Atomics.waitAsync index out of range",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(element_kind.local()));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_to_bigint_u64_word_from_value_locals(
            value_tag_local,
            value_payload_local,
            expected_raw_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_value_to_number_payload(value_tag_local, value_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(value_payload_local));
        self.emit_return_current_completion_if_throw(function);
        self.emit_integer_typed_array_value_i64(value_payload_local, function);
        function.instruction(&Instruction::LocalSet(expected_raw_local));
        self.emit_atomics_normalize_integer_element_i64(
            expected_raw_local,
            &element_kind,
            function,
        );
        function.instruction(&Instruction::LocalSet(expected_raw_local));
        function.instruction(&Instruction::End);
        self.emit_return_current_completion_if_throw(function);

        function.instruction(&Instruction::LocalGet(timeout_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(timeout_payload_local));
        function.instruction(&Instruction::Else);
        self.emit_value_to_number_payload(timeout_tag_local, timeout_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(timeout_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(data_ptr_local));
        function.instruction(&Instruction::LocalGet(byte_offset_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(bytes_per_element_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(address_local));

        self.emit_atomics_load_integer_element_to_i64(
            address_local,
            &element_kind,
            current_raw_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(current_raw_local));
        function.instruction(&Instruction::LocalGet(expected_raw_local));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_atomics_wait_async_return_object(AtomicsWaitOutcome::NotEqual, function)?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(timeout_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Le);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_atomics_wait_async_return_object(AtomicsWaitOutcome::TimedOut, function)?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(i64::MAX));
        function.instruction(&Instruction::LocalSet(deadline_nanos_local));
        function.instruction(&Instruction::LocalGet(timeout_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Call(
            self.functions
                .monotonic_clock_nanos_import_function_index()
                .expect("Atomics.waitAsync requires the monotonic clock import"),
        ));
        function.instruction(&Instruction::LocalSet(monotonic_now_local));
        function.instruction(&Instruction::LocalGet(timeout_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(1_000_000.0)));
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::F64Ceil);
        function.instruction(&Instruction::F64Const(Ieee64::from(i64::MAX as f64)));
        function.instruction(&Instruction::F64Min);
        function.instruction(&Instruction::I64TruncSatF64U);
        function.instruction(&Instruction::LocalSet(timeout_nanos_local));
        function.instruction(&Instruction::LocalGet(timeout_nanos_local));
        function.instruction(&Instruction::I64Const(i64::MAX));
        function.instruction(&Instruction::LocalGet(monotonic_now_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(monotonic_now_local));
        function.instruction(&Instruction::LocalGet(timeout_nanos_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(deadline_nanos_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_atomics_wait_async_return_promise(address_local, deadline_nanos_local, function)?;
        self.emit_return_current_completion(function);

        self.release_temp_local(monotonic_now_local);
        self.release_temp_local(timeout_nanos_local);
        self.release_temp_local(deadline_nanos_local);
        self.release_temp_local(current_raw_local);
        self.release_temp_local(expected_raw_local);
        self.release_temp_local(address_local);
        self.release_temp_local(index_local);
        self.release_temp_local(element_kind.into_local());
        self.release_temp_local(element_length_local);
        self.release_temp_local(bytes_per_element_local);
        self.release_temp_local(stored_byte_length_local);
        self.release_temp_local(byte_offset_local);
        self.release_temp_local(data_ptr_local);
        self.release_temp_local(buffer_brand_local);
        self.release_temp_local(buffer_tag_local);
        self.release_temp_local(buffer_payload_local);
        self.release_temp_local(typed_array_brand_local);
        self.release_temp_local(timeout_tag_local);
        self.release_temp_local(timeout_payload_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(index_tag_local);
        self.release_temp_local(index_payload_local);
        self.release_temp_local(typed_array_tag_local);
        self.release_temp_local(typed_array_payload_local);

        Ok(())
    }

    pub(crate) fn emit_drain_atomics_wait_async_timeouts(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_atomics_wait_async_timeout_checkpoint(
            AtomicsWaitAsyncTimeoutCheckpointMode::Drain,
            function,
        )
    }

    pub(crate) fn emit_poll_atomics_wait_async_timeouts(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_atomics_wait_async_timeout_checkpoint(
            AtomicsWaitAsyncTimeoutCheckpointMode::Poll,
            function,
        )
    }

    fn emit_atomics_wait_async_timeout_checkpoint(
        &mut self,
        mode: AtomicsWaitAsyncTimeoutCheckpointMode,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let saved_result_local = self.reserve_temp_local();
        let saved_result_tag_local = self.reserve_temp_local();
        let saved_completion_local = self.reserve_temp_local();
        let saved_completion_aux_local = self.reserve_temp_local();
        let waiter_local = self.reserve_temp_local();
        let waiter_next_local = self.reserve_temp_local();
        let previous_waiter_local = self.reserve_temp_local();
        let waiter_state_local = self.reserve_temp_local();
        let deadline_nanos_local = self.reserve_temp_local();
        let nearest_deadline_nanos_local = self.reserve_temp_local();
        let monotonic_now_local = self.reserve_temp_local();
        let promise_record_local = self.reserve_temp_local();
        let outcome_tag_local = self.reserve_temp_local();
        let settled_count_local = self.reserve_temp_local();
        let active_count_local = self.reserve_temp_local();
        let waiter_host_id_local = self.reserve_temp_local();
        let host_waiter_status_local = self.reserve_temp_local();
        let agent_call_function_index = self.functions.agent_call_import_function_index();

        for (source, destination) in [
            (self.result_local, saved_result_local),
            (self.result_tag_local, saved_result_tag_local),
            (self.completion_local, saved_completion_local),
            (self.completion_aux_local, saved_completion_aux_local),
        ] {
            function.instruction(&Instruction::LocalGet(source));
            function.instruction(&Instruction::LocalSet(destination));
        }
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(settled_count_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::Call(
            self.functions
                .monotonic_clock_nanos_import_function_index()
                .expect("Atomics.waitAsync requires the monotonic clock import"),
        ));
        function.instruction(&Instruction::LocalSet(monotonic_now_local));
        function.instruction(&Instruction::I64Const(i64::MAX));
        function.instruction(&Instruction::LocalSet(nearest_deadline_nanos_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(active_count_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(previous_waiter_local));
        function.instruction(&Instruction::GlobalGet(
            ATOMICS_ASYNC_WAITER_ACTIVE_LIST_HEAD_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(waiter_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(waiter_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        self.load_i64_to_local_from_offset(
            waiter_local,
            HEAP_ATOMICS_ASYNC_WAITER_NEXT_OFFSET,
            waiter_next_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            waiter_local,
            HEAP_ATOMICS_ASYNC_WAITER_STATE_OFFSET,
            waiter_state_local,
            function,
        );
        if let Some(agent_call_function_index) = agent_call_function_index {
            function.instruction(&Instruction::LocalGet(waiter_state_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Else);
            self.load_i64_to_local_from_offset(
                waiter_local,
                HEAP_ATOMICS_ASYNC_WAITER_HOST_ID_OFFSET,
                waiter_host_id_local,
                function,
            );
            function.instruction(&Instruction::I64Const(
                AgentHostOperation::PollAsyncWaiter.wire(),
            ));
            function.instruction(&Instruction::LocalGet(waiter_host_id_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::Call(agent_call_function_index));
            function.instruction(&Instruction::LocalSet(host_waiter_status_local));
            function.instruction(&Instruction::LocalGet(host_waiter_status_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Else);
            self.store_i64_const_at_offset(
                waiter_local,
                HEAP_ATOMICS_ASYNC_WAITER_STATE_OFFSET,
                0,
                function,
            );
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(waiter_state_local));
            function.instruction(&Instruction::LocalGet(host_waiter_status_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.load_i64_to_local_from_offset(
                waiter_local,
                HEAP_ATOMICS_ASYNC_WAITER_PROMISE_RECORD_OFFSET,
                promise_record_local,
                function,
            );
            function.instruction(&Instruction::I64Const(
                self.strings.payload(AtomicsWaitOutcome::Ok.spelling()),
            ));
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
            function.instruction(&Instruction::LocalSet(outcome_tag_local));
            self.emit_settle_promise_record(
                promise_record_local,
                PromiseSettlement::Fulfill,
                self.scratch_local,
                outcome_tag_local,
                function,
            )?;
            function.instruction(&Instruction::LocalGet(settled_count_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(settled_count_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(waiter_state_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            waiter_local,
            HEAP_ATOMICS_ASYNC_WAITER_DEADLINE_NANOS_OFFSET,
            deadline_nanos_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(deadline_nanos_local));
        function.instruction(&Instruction::I64Const(i64::MAX));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(deadline_nanos_local));
        function.instruction(&Instruction::LocalGet(monotonic_now_local));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        if let Some(agent_call_function_index) = agent_call_function_index {
            function.instruction(&Instruction::I64Const(
                AgentHostOperation::CancelAsyncWaiter.wire(),
            ));
            function.instruction(&Instruction::LocalGet(waiter_host_id_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::Call(agent_call_function_index));
            function.instruction(&Instruction::LocalSet(host_waiter_status_local));
        }
        self.store_i64_const_at_offset(
            waiter_local,
            HEAP_ATOMICS_ASYNC_WAITER_STATE_OFFSET,
            0,
            function,
        );
        self.load_i64_to_local_from_offset(
            waiter_local,
            HEAP_ATOMICS_ASYNC_WAITER_PROMISE_RECORD_OFFSET,
            promise_record_local,
            function,
        );
        if agent_call_function_index.is_some() {
            function.instruction(&Instruction::LocalGet(host_waiter_status_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            function.instruction(&Instruction::I64Const(
                self.strings.payload(AtomicsWaitOutcome::Ok.spelling()),
            ));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::I64Const(
                self.strings
                    .payload(AtomicsWaitOutcome::TimedOut.spelling()),
            ));
            function.instruction(&Instruction::End);
        } else {
            function.instruction(&Instruction::I64Const(
                self.strings
                    .payload(AtomicsWaitOutcome::TimedOut.spelling()),
            ));
        }
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(outcome_tag_local));
        self.emit_settle_promise_record(
            promise_record_local,
            PromiseSettlement::Fulfill,
            self.scratch_local,
            outcome_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(settled_count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(settled_count_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(waiter_state_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(deadline_nanos_local));
        function.instruction(&Instruction::LocalGet(nearest_deadline_nanos_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(deadline_nanos_local));
        function.instruction(&Instruction::LocalSet(nearest_deadline_nanos_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(waiter_state_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(active_count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(active_count_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(waiter_state_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(previous_waiter_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(waiter_next_local));
        function.instruction(&Instruction::GlobalSet(
            ATOMICS_ASYNC_WAITER_ACTIVE_LIST_HEAD_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::Else);
        self.store_i64_local_at_offset(
            previous_waiter_local,
            HEAP_ATOMICS_ASYNC_WAITER_NEXT_OFFSET,
            waiter_next_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(waiter_local));
        function.instruction(&Instruction::LocalSet(previous_waiter_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(waiter_next_local));
        function.instruction(&Instruction::LocalSet(waiter_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        match mode {
            AtomicsWaitAsyncTimeoutCheckpointMode::Drain => {}
            AtomicsWaitAsyncTimeoutCheckpointMode::Poll => {
                function.instruction(&Instruction::Br(1));
            }
        }
        function.instruction(&Instruction::LocalGet(active_count_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(nearest_deadline_nanos_local));
        function.instruction(&Instruction::I64Const(i64::MAX));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        if agent_call_function_index.is_some() {
            function.instruction(&Instruction::I64Const(1_000_000));
            function.instruction(&Instruction::Call(
                self.functions
                    .sleep_nanos_import_function_index()
                    .expect("Atomics.waitAsync requires the sleep import"),
            ));
            function.instruction(&Instruction::Br(1));
        } else {
            function.instruction(&Instruction::Br(2));
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Call(
            self.functions
                .monotonic_clock_nanos_import_function_index()
                .expect("Atomics.waitAsync requires the monotonic clock import"),
        ));
        function.instruction(&Instruction::LocalSet(monotonic_now_local));
        function.instruction(&Instruction::LocalGet(nearest_deadline_nanos_local));
        function.instruction(&Instruction::LocalGet(monotonic_now_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(nearest_deadline_nanos_local));
        function.instruction(&Instruction::LocalGet(monotonic_now_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::Call(
            self.functions
                .sleep_nanos_import_function_index()
                .expect("Atomics.waitAsync requires the sleep import"),
        ));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        for (source, destination) in [
            (saved_result_local, self.result_local),
            (saved_result_tag_local, self.result_tag_local),
            (saved_completion_local, self.completion_local),
            (saved_completion_aux_local, self.completion_aux_local),
        ] {
            function.instruction(&Instruction::LocalGet(source));
            function.instruction(&Instruction::LocalSet(destination));
        }
        function.instruction(&Instruction::LocalGet(settled_count_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);

        self.release_temp_local(host_waiter_status_local);
        self.release_temp_local(waiter_host_id_local);
        self.release_temp_local(active_count_local);
        self.release_temp_local(settled_count_local);
        self.release_temp_local(outcome_tag_local);
        self.release_temp_local(promise_record_local);
        self.release_temp_local(monotonic_now_local);
        self.release_temp_local(nearest_deadline_nanos_local);
        self.release_temp_local(deadline_nanos_local);
        self.release_temp_local(waiter_state_local);
        self.release_temp_local(previous_waiter_local);
        self.release_temp_local(waiter_next_local);
        self.release_temp_local(waiter_local);
        self.release_temp_local(saved_completion_aux_local);
        self.release_temp_local(saved_completion_local);
        self.release_temp_local(saved_result_tag_local);
        self.release_temp_local(saved_result_local);
        Ok(())
    }

    fn emit_atomics_wait(&mut self, function: &mut Function) -> Result<(), EmitError> {
        let typed_array_payload_local = self.reserve_temp_local();
        let typed_array_tag_local = self.reserve_temp_local();
        let index_payload_local = self.reserve_temp_local();
        let index_tag_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let timeout_payload_local = self.reserve_temp_local();
        let timeout_tag_local = self.reserve_temp_local();
        let typed_array_brand_local = self.reserve_temp_local();
        let buffer_payload_local = self.reserve_temp_local();
        let buffer_tag_local = self.reserve_temp_local();
        let buffer_brand_local = self.reserve_temp_local();
        let data_ptr_local = self.reserve_temp_local();
        let byte_offset_local = self.reserve_temp_local();
        let stored_byte_length_local = self.reserve_temp_local();
        let bytes_per_element_local = self.reserve_temp_local();
        let element_length_local = self.reserve_temp_local();
        let pending_element_kind = PendingAtomicsIntegerElementKindLocal(self.reserve_temp_local());
        let index_local = self.reserve_temp_local();
        let address_local = self.reserve_temp_local();
        let expected_raw_local = self.reserve_temp_local();
        let current_raw_local = self.reserve_temp_local();
        let timeout_nanoseconds_local = self.reserve_temp_local();
        let wait_result_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(
            0,
            typed_array_payload_local,
            typed_array_tag_local,
            function,
        );
        self.emit_builtin_arg_to_locals(1, index_payload_local, index_tag_local, function);
        self.emit_builtin_arg_to_locals(2, value_payload_local, value_tag_local, function);
        self.emit_builtin_arg_to_locals(3, timeout_payload_local, timeout_tag_local, function);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(typed_array_brand_local));
        function.instruction(&Instruction::LocalGet(typed_array_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            typed_array_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            typed_array_brand_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(typed_array_brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TYPED_ARRAY as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Atomics.wait requires a shared Int32Array or BigInt64Array",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_load_typed_array_private_state(
            typed_array_payload_local,
            buffer_payload_local,
            byte_offset_local,
            stored_byte_length_local,
            bytes_per_element_local,
            function,
        );
        let typed_array_view = TypedArrayViewLocals::new(
            typed_array_payload_local,
            buffer_payload_local,
            byte_offset_local,
            stored_byte_length_local,
            bytes_per_element_local,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(buffer_tag_local));
        self.emit_require_array_buffer_or_shared_array_buffer(
            buffer_payload_local,
            buffer_tag_local,
            "Atomics.wait requires a shared Int32Array or BigInt64Array",
            function,
        )?;

        let element_kind = self.emit_validate_atomics_integer_element_kind(
            typed_array_payload_local,
            pending_element_kind,
            AtomicsIntegerElementKindRequirement::Waitable,
            "Atomics.wait requires a shared Int32Array or BigInt64Array",
            function,
        )?;

        self.load_i64_to_local_from_offset(
            buffer_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            buffer_brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(buffer_brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_SHARED_ARRAY_BUFFER as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Atomics.wait requires a shared Int32Array or BigInt64Array",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_load_array_buffer_data(buffer_payload_local, data_ptr_local, function);
        function.instruction(&Instruction::LocalGet(data_ptr_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "TypedArray backing buffer is detached",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_typed_array_witness(
            &typed_array_view,
            TypedArrayWitnessUse::ValidatedMethodEntry {
                length_local: element_length_local,
            },
            function,
        )?;

        self.emit_value_to_number_payload(index_tag_local, index_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(index_payload_local));
        self.emit_return_current_completion_if_throw(function);
        self.emit_to_index_from_number_payload(
            index_payload_local,
            index_local,
            "Atomics.wait index out of range",
            function,
        )?;
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(element_length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Atomics.wait index out of range",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(element_kind.local()));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_to_bigint_u64_word_from_value_locals(
            value_tag_local,
            value_payload_local,
            expected_raw_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_value_to_number_payload(value_tag_local, value_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(value_payload_local));
        self.emit_return_current_completion_if_throw(function);
        self.emit_integer_typed_array_value_i64(value_payload_local, function);
        function.instruction(&Instruction::LocalSet(expected_raw_local));
        self.emit_atomics_normalize_integer_element_i64(
            expected_raw_local,
            &element_kind,
            function,
        );
        function.instruction(&Instruction::LocalSet(expected_raw_local));
        function.instruction(&Instruction::End);
        self.emit_return_current_completion_if_throw(function);

        function.instruction(&Instruction::LocalGet(timeout_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(timeout_payload_local));
        function.instruction(&Instruction::Else);
        self.emit_value_to_number_payload(timeout_tag_local, timeout_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(timeout_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(data_ptr_local));
        function.instruction(&Instruction::LocalGet(byte_offset_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(bytes_per_element_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(address_local));

        self.emit_atomics_require_agent_can_suspend(function)?;
        self.emit_atomics_load_integer_element_to_i64(
            address_local,
            &element_kind,
            current_raw_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(current_raw_local));
        function.instruction(&Instruction::LocalGet(expected_raw_local));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_atomics_wait_return_string(AtomicsWaitOutcome::NotEqual, function);
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(timeout_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Le);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_atomics_wait_return_string(AtomicsWaitOutcome::TimedOut, function);
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(timeout_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(timeout_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(timeout_nanoseconds_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(timeout_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(timeout_nanoseconds_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(timeout_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(1_000_000.0)));
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::I64TruncSatF64S);
        function.instruction(&Instruction::LocalSet(timeout_nanoseconds_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(element_kind.local()));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(expected_raw_local));
        function.instruction(&Instruction::LocalGet(timeout_nanoseconds_local));
        function.instruction(&Instruction::MemoryAtomicWait64(Self::shared_memarg64(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(wait_result_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(expected_raw_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(timeout_nanoseconds_local));
        function.instruction(&Instruction::MemoryAtomicWait32(Self::shared_memarg32(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(wait_result_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(wait_result_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_atomics_wait_return_string(AtomicsWaitOutcome::Ok, function);
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(wait_result_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_atomics_wait_return_string(AtomicsWaitOutcome::NotEqual, function);
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_atomics_wait_return_string(AtomicsWaitOutcome::TimedOut, function);

        self.release_temp_local(wait_result_local);
        self.release_temp_local(timeout_nanoseconds_local);
        self.release_temp_local(current_raw_local);
        self.release_temp_local(expected_raw_local);
        self.release_temp_local(address_local);
        self.release_temp_local(index_local);
        self.release_temp_local(element_kind.into_local());
        self.release_temp_local(element_length_local);
        self.release_temp_local(bytes_per_element_local);
        self.release_temp_local(stored_byte_length_local);
        self.release_temp_local(byte_offset_local);
        self.release_temp_local(data_ptr_local);
        self.release_temp_local(buffer_brand_local);
        self.release_temp_local(buffer_tag_local);
        self.release_temp_local(buffer_payload_local);
        self.release_temp_local(typed_array_brand_local);
        self.release_temp_local(timeout_tag_local);
        self.release_temp_local(timeout_payload_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(index_tag_local);
        self.release_temp_local(index_payload_local);
        self.release_temp_local(typed_array_tag_local);
        self.release_temp_local(typed_array_payload_local);

        Ok(())
    }

    fn emit_atomics_integer_operation(
        &mut self,
        operation: AtomicsIntegerOperation,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let type_error_message = match &operation {
            AtomicsIntegerOperation::Add => "Atomics.add requires an integer typed array",
            AtomicsIntegerOperation::And => "Atomics.and requires an integer typed array",
            AtomicsIntegerOperation::CompareExchange => {
                "Atomics.compareExchange requires an integer typed array"
            }
            AtomicsIntegerOperation::Exchange => "Atomics.exchange requires an integer typed array",
            AtomicsIntegerOperation::Load => "Atomics.load requires an integer typed array",
            AtomicsIntegerOperation::Or => "Atomics.or requires an integer typed array",
            AtomicsIntegerOperation::Store => "Atomics.store requires an integer typed array",
            AtomicsIntegerOperation::Sub => "Atomics.sub requires an integer typed array",
            AtomicsIntegerOperation::Xor => "Atomics.xor requires an integer typed array",
        };
        let range_error_message = match &operation {
            AtomicsIntegerOperation::Add => "Atomics.add index out of range",
            AtomicsIntegerOperation::And => "Atomics.and index out of range",
            AtomicsIntegerOperation::CompareExchange => {
                "Atomics.compareExchange index out of range"
            }
            AtomicsIntegerOperation::Exchange => "Atomics.exchange index out of range",
            AtomicsIntegerOperation::Load => "Atomics.load index out of range",
            AtomicsIntegerOperation::Or => "Atomics.or index out of range",
            AtomicsIntegerOperation::Store => "Atomics.store index out of range",
            AtomicsIntegerOperation::Sub => "Atomics.sub index out of range",
            AtomicsIntegerOperation::Xor => "Atomics.xor index out of range",
        };
        let value_arg_count = operation.value_arg_count();

        let typed_array_payload_local = self.reserve_temp_local();
        let typed_array_tag_local = self.reserve_temp_local();
        let index_payload_local = self.reserve_temp_local();
        let index_tag_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let replacement_payload_local = self.reserve_temp_local();
        let replacement_tag_local = self.reserve_temp_local();
        let typed_array_brand_local = self.reserve_temp_local();
        let buffer_payload_local = self.reserve_temp_local();
        let buffer_tag_local = self.reserve_temp_local();
        let data_ptr_local = self.reserve_temp_local();
        let byte_offset_local = self.reserve_temp_local();
        let stored_byte_length_local = self.reserve_temp_local();
        let bytes_per_element_local = self.reserve_temp_local();
        let element_length_local = self.reserve_temp_local();
        let pending_element_kind = PendingAtomicsIntegerElementKindLocal(self.reserve_temp_local());
        let index_local = self.reserve_temp_local();
        let address_local = self.reserve_temp_local();
        let old_raw_local = self.reserve_temp_local();
        let value_raw_local = self.reserve_temp_local();
        let replacement_raw_local = self.reserve_temp_local();
        let value_bigint_payload_local = self.reserve_temp_local();
        let value_bigint_tag_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(
            0,
            typed_array_payload_local,
            typed_array_tag_local,
            function,
        );
        self.emit_builtin_arg_to_locals(1, index_payload_local, index_tag_local, function);
        if value_arg_count > 0 {
            self.emit_builtin_arg_to_locals(2, value_payload_local, value_tag_local, function);
        }
        if value_arg_count > 1 {
            self.emit_builtin_arg_to_locals(
                3,
                replacement_payload_local,
                replacement_tag_local,
                function,
            );
        }

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(typed_array_brand_local));
        function.instruction(&Instruction::LocalGet(typed_array_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            typed_array_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            typed_array_brand_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(typed_array_brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TYPED_ARRAY as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            type_error_message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_load_typed_array_private_state(
            typed_array_payload_local,
            buffer_payload_local,
            byte_offset_local,
            stored_byte_length_local,
            bytes_per_element_local,
            function,
        );
        let typed_array_view = TypedArrayViewLocals::new(
            typed_array_payload_local,
            buffer_payload_local,
            byte_offset_local,
            stored_byte_length_local,
            bytes_per_element_local,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(buffer_tag_local));
        self.emit_require_array_buffer_or_shared_array_buffer(
            buffer_payload_local,
            buffer_tag_local,
            type_error_message,
            function,
        )?;

        let element_kind = self.emit_validate_atomics_integer_element_kind(
            typed_array_payload_local,
            pending_element_kind,
            AtomicsIntegerElementKindRequirement::AnyInteger,
            type_error_message,
            function,
        )?;

        self.emit_load_array_buffer_data(buffer_payload_local, data_ptr_local, function);
        function.instruction(&Instruction::LocalGet(data_ptr_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "TypedArray backing buffer is detached",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_typed_array_witness(
            &typed_array_view,
            TypedArrayWitnessUse::ValidatedMethodEntry {
                length_local: element_length_local,
            },
            function,
        )?;

        self.emit_value_to_number_payload(index_tag_local, index_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(index_payload_local));
        self.emit_return_current_completion_if_throw(function);
        self.emit_to_index_from_number_payload(
            index_payload_local,
            index_local,
            range_error_message,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(element_length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            range_error_message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        if value_arg_count > 0 {
            self.emit_validated_atomics_bigint_element_kind_i32(&element_kind, function);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_to_bigint_value_and_u64_word_from_value_locals(
                value_tag_local,
                value_payload_local,
                value_bigint_payload_local,
                value_bigint_tag_local,
                value_raw_local,
                function,
            )?;
            function.instruction(&Instruction::Else);
            self.emit_value_to_number_payload(value_tag_local, value_payload_local, function)?;
            function.instruction(&Instruction::LocalSet(value_payload_local));
            self.emit_return_current_completion_if_throw(function);
            self.emit_to_integer_or_infinity_number_payload_from_number_payload(
                value_payload_local,
                value_payload_local,
                function,
            );
            self.emit_integer_typed_array_value_i64(value_payload_local, function);
            function.instruction(&Instruction::LocalSet(value_raw_local));
            function.instruction(&Instruction::End);
            self.emit_return_current_completion_if_throw(function);
        }

        if value_arg_count > 1 {
            self.emit_validated_atomics_bigint_element_kind_i32(&element_kind, function);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_to_bigint_u64_word_from_value_locals(
                replacement_tag_local,
                replacement_payload_local,
                replacement_raw_local,
                function,
            )?;
            function.instruction(&Instruction::Else);
            self.emit_value_to_number_payload(
                replacement_tag_local,
                replacement_payload_local,
                function,
            )?;
            function.instruction(&Instruction::LocalSet(replacement_payload_local));
            self.emit_return_current_completion_if_throw(function);
            self.emit_integer_typed_array_value_i64(replacement_payload_local, function);
            function.instruction(&Instruction::LocalSet(replacement_raw_local));
            function.instruction(&Instruction::End);
            self.emit_return_current_completion_if_throw(function);
        }

        function.instruction(&Instruction::LocalGet(data_ptr_local));
        function.instruction(&Instruction::LocalGet(byte_offset_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(bytes_per_element_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(address_local));

        match &operation {
            AtomicsIntegerOperation::Store => {
                self.emit_atomics_store_integer_element_from_i64(
                    address_local,
                    &element_kind,
                    value_raw_local,
                    function,
                );
                self.emit_validated_atomics_bigint_element_kind_i32(&element_kind, function);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(value_bigint_payload_local));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::LocalGet(value_bigint_tag_local));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::LocalGet(value_payload_local));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                function.instruction(&Instruction::End);
            }
            AtomicsIntegerOperation::Load => {
                self.emit_atomics_load_integer_element_to_i64(
                    address_local,
                    &element_kind,
                    old_raw_local,
                    function,
                );
            }
            AtomicsIntegerOperation::CompareExchange => {
                self.emit_atomics_normalize_integer_element_i64(
                    value_raw_local,
                    &element_kind,
                    function,
                );
                function.instruction(&Instruction::LocalSet(value_raw_local));
                self.emit_atomics_compare_exchange_integer_element_to_i64(
                    address_local,
                    &element_kind,
                    value_raw_local,
                    replacement_raw_local,
                    old_raw_local,
                    function,
                );
            }
            AtomicsIntegerOperation::Add => {
                self.emit_atomics_rmw_integer_element_to_i64(
                    address_local,
                    &element_kind,
                    value_raw_local,
                    AtomicsRmwOperation::Add,
                    old_raw_local,
                    function,
                );
            }
            AtomicsIntegerOperation::And => self.emit_atomics_rmw_integer_element_to_i64(
                address_local,
                &element_kind,
                value_raw_local,
                AtomicsRmwOperation::And,
                old_raw_local,
                function,
            ),
            AtomicsIntegerOperation::Exchange => self.emit_atomics_rmw_integer_element_to_i64(
                address_local,
                &element_kind,
                value_raw_local,
                AtomicsRmwOperation::Exchange,
                old_raw_local,
                function,
            ),
            AtomicsIntegerOperation::Or => self.emit_atomics_rmw_integer_element_to_i64(
                address_local,
                &element_kind,
                value_raw_local,
                AtomicsRmwOperation::Or,
                old_raw_local,
                function,
            ),
            AtomicsIntegerOperation::Sub => self.emit_atomics_rmw_integer_element_to_i64(
                address_local,
                &element_kind,
                value_raw_local,
                AtomicsRmwOperation::Sub,
                old_raw_local,
                function,
            ),
            AtomicsIntegerOperation::Xor => self.emit_atomics_rmw_integer_element_to_i64(
                address_local,
                &element_kind,
                value_raw_local,
                AtomicsRmwOperation::Xor,
                old_raw_local,
                function,
            ),
        }

        match &operation {
            AtomicsIntegerOperation::Store => {}
            AtomicsIntegerOperation::Load
            | AtomicsIntegerOperation::Add
            | AtomicsIntegerOperation::And
            | AtomicsIntegerOperation::CompareExchange
            | AtomicsIntegerOperation::Exchange
            | AtomicsIntegerOperation::Or
            | AtomicsIntegerOperation::Sub
            | AtomicsIntegerOperation::Xor => {
                self.emit_validated_atomics_bigint_element_kind_i32(&element_kind, function);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(element_kind.local()));
                function.instruction(&Instruction::I64Const(11));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::LocalGet(old_raw_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64LtS);
                function.instruction(&Instruction::I32And);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_alloc_one_limb_bigint(1, old_raw_local, function)?;
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(HEAP_BIGINT_VALUE_TAG));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::LocalGet(old_raw_local));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::Else);
                self.emit_atomics_signed_number_element_kind_i32(&element_kind, function);
                function.instruction(&Instruction::If(BlockType::Result(ValType::F64)));
                function.instruction(&Instruction::LocalGet(old_raw_local));
                function.instruction(&Instruction::F64ConvertI64S);
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::LocalGet(old_raw_local));
                function.instruction(&Instruction::F64ConvertI64U);
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::I64ReinterpretF64);
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                function.instruction(&Instruction::End);
            }
        }

        self.release_temp_local(value_bigint_tag_local);
        self.release_temp_local(value_bigint_payload_local);
        self.release_temp_local(replacement_raw_local);
        self.release_temp_local(value_raw_local);
        self.release_temp_local(old_raw_local);
        self.release_temp_local(address_local);
        self.release_temp_local(index_local);
        self.release_temp_local(element_kind.into_local());
        self.release_temp_local(element_length_local);
        self.release_temp_local(bytes_per_element_local);
        self.release_temp_local(stored_byte_length_local);
        self.release_temp_local(byte_offset_local);
        self.release_temp_local(data_ptr_local);
        self.release_temp_local(buffer_tag_local);
        self.release_temp_local(buffer_payload_local);
        self.release_temp_local(typed_array_brand_local);
        self.release_temp_local(replacement_tag_local);
        self.release_temp_local(replacement_payload_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(index_tag_local);
        self.release_temp_local(index_payload_local);
        self.release_temp_local(typed_array_tag_local);
        self.release_temp_local(typed_array_payload_local);

        Ok(())
    }

    fn emit_atomics_friendly_element_kind_i32(
        &mut self,
        element_kind_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(5));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(11));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
    }

    fn emit_atomics_normalize_integer_element_i64(
        &mut self,
        value_local: u32,
        element_kind: &ValidatedAtomicsIntegerElementKindLocal,
        function: &mut Function,
    ) {
        let element_kind_local = element_kind.local();
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::I64Const(56));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Const(56));
        function.instruction(&Instruction::I64ShrS);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::I64Const(48));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Const(48));
        function.instruction(&Instruction::I64ShrS);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(5));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrS);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::I64Const(0xff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::I64Const(0xffff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(9));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::I64Const(0xffff_ffff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    pub(super) fn emit_atomics_bigint_element_kind_i32(
        &mut self,
        element_kind_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(11));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
    }

    fn emit_validated_atomics_bigint_element_kind_i32(
        &mut self,
        element_kind: &ValidatedAtomicsIntegerElementKindLocal,
        function: &mut Function,
    ) {
        self.emit_atomics_bigint_element_kind_i32(element_kind.local(), function);
    }

    fn emit_atomics_signed_number_element_kind_i32(
        &mut self,
        element_kind: &ValidatedAtomicsIntegerElementKindLocal,
        function: &mut Function,
    ) {
        let element_kind_local = element_kind.local();
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(5));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
    }

    fn emit_atomics_rmw_integer_element_to_i64(
        &mut self,
        address_local: u32,
        element_kind: &ValidatedAtomicsIntegerElementKindLocal,
        value_local: u32,
        operation: AtomicsRmwOperation,
        output_local: u32,
        function: &mut Function,
    ) {
        let element_kind_local = element_kind.local();
        self.emit_validated_atomics_bigint_element_kind_i32(element_kind, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(value_local));
        match operation {
            AtomicsRmwOperation::Add => {
                function.instruction(&Instruction::I64AtomicRmwAdd(Self::shared_memarg64(0)));
            }
            AtomicsRmwOperation::And => {
                function.instruction(&Instruction::I64AtomicRmwAnd(Self::shared_memarg64(0)));
            }
            AtomicsRmwOperation::Exchange => {
                function.instruction(&Instruction::I64AtomicRmwXchg(Self::shared_memarg64(0)));
            }
            AtomicsRmwOperation::Or => {
                function.instruction(&Instruction::I64AtomicRmwOr(Self::shared_memarg64(0)));
            }
            AtomicsRmwOperation::Sub => {
                function.instruction(&Instruction::I64AtomicRmwSub(Self::shared_memarg64(0)));
            }
            AtomicsRmwOperation::Xor => {
                function.instruction(&Instruction::I64AtomicRmwXor(Self::shared_memarg64(0)));
            }
        }
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::I32WrapI64);
        match operation {
            AtomicsRmwOperation::Add => {
                function.instruction(&Instruction::I32AtomicRmw8AddU(Self::shared_memarg8(0)));
            }
            AtomicsRmwOperation::And => {
                function.instruction(&Instruction::I32AtomicRmw8AndU(Self::shared_memarg8(0)));
            }
            AtomicsRmwOperation::Exchange => {
                function.instruction(&Instruction::I32AtomicRmw8XchgU(Self::shared_memarg8(0)));
            }
            AtomicsRmwOperation::Or => {
                function.instruction(&Instruction::I32AtomicRmw8OrU(Self::shared_memarg8(0)));
            }
            AtomicsRmwOperation::Sub => {
                function.instruction(&Instruction::I32AtomicRmw8SubU(Self::shared_memarg8(0)));
            }
            AtomicsRmwOperation::Xor => {
                function.instruction(&Instruction::I32AtomicRmw8XorU(Self::shared_memarg8(0)));
            }
        }
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::I32WrapI64);
        match operation {
            AtomicsRmwOperation::Add => {
                function.instruction(&Instruction::I32AtomicRmw16AddU(Self::shared_memarg16(0)));
            }
            AtomicsRmwOperation::And => {
                function.instruction(&Instruction::I32AtomicRmw16AndU(Self::shared_memarg16(0)));
            }
            AtomicsRmwOperation::Exchange => {
                function.instruction(&Instruction::I32AtomicRmw16XchgU(Self::shared_memarg16(0)));
            }
            AtomicsRmwOperation::Or => {
                function.instruction(&Instruction::I32AtomicRmw16OrU(Self::shared_memarg16(0)));
            }
            AtomicsRmwOperation::Sub => {
                function.instruction(&Instruction::I32AtomicRmw16SubU(Self::shared_memarg16(0)));
            }
            AtomicsRmwOperation::Xor => {
                function.instruction(&Instruction::I32AtomicRmw16XorU(Self::shared_memarg16(0)));
            }
        }
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::I32WrapI64);
        match operation {
            AtomicsRmwOperation::Add => {
                function.instruction(&Instruction::I32AtomicRmwAdd(Self::shared_memarg32(0)));
            }
            AtomicsRmwOperation::And => {
                function.instruction(&Instruction::I32AtomicRmwAnd(Self::shared_memarg32(0)));
            }
            AtomicsRmwOperation::Exchange => {
                function.instruction(&Instruction::I32AtomicRmwXchg(Self::shared_memarg32(0)));
            }
            AtomicsRmwOperation::Or => {
                function.instruction(&Instruction::I32AtomicRmwOr(Self::shared_memarg32(0)));
            }
            AtomicsRmwOperation::Sub => {
                function.instruction(&Instruction::I32AtomicRmwSub(Self::shared_memarg32(0)));
            }
            AtomicsRmwOperation::Xor => {
                function.instruction(&Instruction::I32AtomicRmwXor(Self::shared_memarg32(0)));
            }
        }
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_atomics_normalize_integer_element_i64(output_local, element_kind, function);
        function.instruction(&Instruction::LocalSet(output_local));
    }

    fn emit_atomics_compare_exchange_integer_element_to_i64(
        &mut self,
        address_local: u32,
        element_kind: &ValidatedAtomicsIntegerElementKindLocal,
        expected_local: u32,
        replacement_local: u32,
        output_local: u32,
        function: &mut Function,
    ) {
        let element_kind_local = element_kind.local();
        self.emit_validated_atomics_bigint_element_kind_i32(element_kind, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(expected_local));
        function.instruction(&Instruction::LocalGet(replacement_local));
        function.instruction(&Instruction::I64AtomicRmwCmpxchg(Self::shared_memarg64(0)));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(expected_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(replacement_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32AtomicRmw8CmpxchgU(Self::shared_memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(expected_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(replacement_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32AtomicRmw16CmpxchgU(Self::shared_memarg16(
            0,
        )));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(expected_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(replacement_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32AtomicRmwCmpxchg(Self::shared_memarg32(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_atomics_normalize_integer_element_i64(output_local, element_kind, function);
        function.instruction(&Instruction::LocalSet(output_local));
    }

    fn emit_atomics_load_integer_element_to_i64(
        &mut self,
        address_local: u32,
        element_kind: &ValidatedAtomicsIntegerElementKindLocal,
        output_local: u32,
        function: &mut Function,
    ) {
        let element_kind_local = element_kind.local();
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32AtomicLoad8U(Self::shared_memarg8(0)));
        function.instruction(&Instruction::I32Const(24));
        function.instruction(&Instruction::I32Shl);
        function.instruction(&Instruction::I32Const(24));
        function.instruction(&Instruction::I32ShrS);
        function.instruction(&Instruction::I64ExtendI32S);
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32AtomicLoad16U(Self::shared_memarg16(0)));
        function.instruction(&Instruction::I32Const(16));
        function.instruction(&Instruction::I32Shl);
        function.instruction(&Instruction::I32Const(16));
        function.instruction(&Instruction::I32ShrS);
        function.instruction(&Instruction::I64ExtendI32S);
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(5));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32AtomicLoad(Self::shared_memarg32(0)));
        function.instruction(&Instruction::I64ExtendI32S);
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32AtomicLoad8U(Self::shared_memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32AtomicLoad16U(Self::shared_memarg16(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(9));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32AtomicLoad(Self::shared_memarg32(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64AtomicLoad(Self::shared_memarg64(0)));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    fn emit_atomics_store_integer_element_from_i64(
        &mut self,
        address_local: u32,
        element_kind: &ValidatedAtomicsIntegerElementKindLocal,
        value_local: u32,
        function: &mut Function,
    ) {
        let element_kind_local = element_kind.local();
        self.emit_validated_atomics_bigint_element_kind_i32(element_kind, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::I64AtomicStore(Self::shared_memarg64(0)));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32AtomicStore8(Self::shared_memarg8(0)));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32AtomicStore16(Self::shared_memarg16(0)));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32AtomicStore(Self::shared_memarg32(0)));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }
}
