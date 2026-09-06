use super::super::*;
use super::binary_data::{TypedArrayViewLocals, TypedArrayWitnessUse};
use crate::control_flow::SyncIteratorConsumer;
use crate::objects::{
    StoredDescriptorDataLocals, StoredDescriptorGetterLocals, StoredDescriptorLocals,
    StoredDescriptorSetterLocals, TaggedLocals, WasmDescriptor, WasmLocals, WasmPartialDescriptor,
};
use lila_ir::property_descriptor::{classify, DescriptorSide, KindTerms, Presence};
use lila_ir::{ArrayAccumulationElementIr, ArrayAccumulationIr, ArrayAccumulationTargetIr};

mod callback_iteration;
mod copy_within;
mod find_via_predicate;
use callback_iteration::ArrayCallbackIterationKind;
use copy_within::ArrayCopyWithinDirection;

/// One array-descriptor field whose value locals always exist, while its
/// 6.2.6 presence may be decided when the program runs.
///
/// Unlike the old `(Option<value>, Option<present_local>)` pair, this cannot
/// express "the field has a run-time presence flag but no value carrier".
fn array_descriptor_field<T>(value: T, present: Option<u32>) -> Presence<T, u32> {
    match present {
        None => Presence::Present(value),
        Some(present) => Presence::Runtime { present, value },
    }
}

enum ArrayNamedStringKeySelection {
    All,
    EnumerableOnly,
}

enum ArraySortOutput {
    Receiver,
    Copy,
}

enum ToLocaleStringReceiverKind {
    ArrayLike,
    TypedArray,
}

pub(crate) enum ArrayInheritedIndexSetState {
    Unhandled,
    Setter,
    OrdinaryRejected,
    Handled,
    ProxyRejected,
}

impl ArrayInheritedIndexSetState {
    pub(crate) const fn code(&self) -> i64 {
        match self {
            Self::Unhandled => 0,
            Self::Setter => 1,
            Self::OrdinaryRejected => 2,
            Self::Handled => 3,
            Self::ProxyRejected => 4,
        }
    }
}

/// Element method and receiver locals that have passed ECMAScript `IsCallable`.
///
/// This token is deliberately private and non-`Copy`. Its sole consumer takes
/// ownership before emitting Proxy-aware `Call` with the paired receiver.
#[must_use = "a validated toLocaleString invocation must be consumed by Call"]
struct ValidatedToLocaleStringInvocationLocals {
    method: TaggedLocals,
    receiver: TaggedLocals,
}

enum ArrayCallbackReceiverKind {
    ArrayLike,
    TypedArray,
}

enum ArrayReduceDirection {
    LeftToRight,
    RightToLeft,
}

impl ArrayReduceDirection {
    const fn method_name(&self, receiver_kind: &ArrayCallbackReceiverKind) -> &'static str {
        match (receiver_kind, self) {
            (ArrayCallbackReceiverKind::ArrayLike, Self::LeftToRight) => "Array.prototype.reduce",
            (ArrayCallbackReceiverKind::ArrayLike, Self::RightToLeft) => {
                "Array.prototype.reduceRight"
            }
            (ArrayCallbackReceiverKind::TypedArray, Self::LeftToRight) => {
                "TypedArray.prototype.reduce"
            }
            (ArrayCallbackReceiverKind::TypedArray, Self::RightToLeft) => {
                "TypedArray.prototype.reduceRight"
            }
        }
    }

    const fn typed_array_receiver_error(&self) -> &'static str {
        match self {
            Self::LeftToRight => "TypedArray.prototype.reduce requires a TypedArray",
            Self::RightToLeft => "TypedArray.prototype.reduceRight requires a TypedArray",
        }
    }

    const fn callback_not_callable_error(
        &self,
        receiver_kind: &ArrayCallbackReceiverKind,
    ) -> &'static str {
        match (receiver_kind, self) {
            (ArrayCallbackReceiverKind::ArrayLike, Self::LeftToRight) => {
                "Array.prototype.reduce callback is not callable"
            }
            (ArrayCallbackReceiverKind::ArrayLike, Self::RightToLeft) => {
                "Array.prototype.reduceRight callback is not callable"
            }
            (ArrayCallbackReceiverKind::TypedArray, Self::LeftToRight) => {
                "TypedArray.prototype.reduce callback is not callable"
            }
            (ArrayCallbackReceiverKind::TypedArray, Self::RightToLeft) => {
                "TypedArray.prototype.reduceRight callback is not callable"
            }
        }
    }
}

enum TypedArrayQuantifierKind {
    Every,
    Some,
}

enum TypedArraySearchKind {
    Includes,
    IndexOf,
    LastIndexOf,
}

enum ArrayAtReceiverPolicy {
    GenericArrayLike,
    TypedArray,
}

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn emit_init_array_exotic_slots(&self, array_local: u32, function: &mut Function) {
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_PROP_DESCRIPTOR_KIND_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_PROP_DATA_TAG_OFFSET,
            ValueKind::Undefined.tag() as u64,
            function,
        );
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_PROP_DATA_PAYLOAD_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_PROP_GETTER_TAG_OFFSET,
            ValueKind::Undefined.tag() as u64,
            function,
        );
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_PROP_GETTER_PAYLOAD_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_PROP_SETTER_TAG_OFFSET,
            ValueKind::Undefined.tag() as u64,
            function,
        );
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_PROP_SETTER_PAYLOAD_OFFSET,
            0,
            function,
        );
        for (descriptor_offset, tag_offset, payload_offset) in [
            (
                HEAP_ARRAY_INDEX_PROP_DESCRIPTOR_KIND_OFFSET,
                HEAP_ARRAY_INDEX_PROP_DATA_TAG_OFFSET,
                HEAP_ARRAY_INDEX_PROP_DATA_PAYLOAD_OFFSET,
            ),
            (
                HEAP_ARRAY_INPUT_PROP_DESCRIPTOR_KIND_OFFSET,
                HEAP_ARRAY_INPUT_PROP_DATA_TAG_OFFSET,
                HEAP_ARRAY_INPUT_PROP_DATA_PAYLOAD_OFFSET,
            ),
        ] {
            self.store_i64_const_at_offset(array_local, descriptor_offset, 0, function);
            self.store_i64_const_at_offset(
                array_local,
                tag_offset,
                ValueKind::Undefined.tag() as u64,
                function,
            );
            self.store_i64_const_at_offset(array_local, payload_offset, 0, function);
        }
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_PTR_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_LEN_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_CAP_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(array_local, HEAP_ARRAY_NAMED_PROPS_PTR_OFFSET, 0, function);
        self.store_i64_const_at_offset(array_local, HEAP_ARRAY_NAMED_PROPS_LEN_OFFSET, 0, function);
        self.store_i64_const_at_offset(array_local, HEAP_ARRAY_NAMED_PROPS_CAP_OFFSET, 0, function);
    }

    pub(crate) fn emit_array_constructor_read(
        &mut self,
        array_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            array_local,
            receiver_tag_local,
            array_local,
            receiver_tag_local,
            key_local,
            payload_local,
            tag_local,
            function,
        )?;

        self.release_temp_local(key_local);
        self.release_temp_local(receiver_tag_local);
        Ok(())
    }

    pub(crate) fn emit_mark_skip_species_for_cross_realm_array_constructor(
        &mut self,
        constructor_payload_local: u32,
        constructor_table_index_local: u32,
        skip_species_local: u32,
        array_constructor_table_index: i64,
        function: &mut Function,
    ) {
        let constructor_realm_local = self.reserve_temp_local();
        let current_function_realm_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(constructor_table_index_local));
        function.instruction(&Instruction::I64Const(array_constructor_table_index));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            constructor_payload_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            constructor_realm_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(constructor_payload_local));
        function.instruction(&Instruction::GlobalGet(ARRAY_CONSTRUCTOR_GLOBAL_INDEX));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(skip_species_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            self.current_env_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            current_function_realm_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(constructor_realm_local));
        function.instruction(&Instruction::LocalGet(current_function_realm_local));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(skip_species_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(current_function_realm_local);
        self.release_temp_local(constructor_realm_local);
    }

    pub(crate) fn compile_array_literal_payload(
        &mut self,
        elements: &[TypedExpr],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let array_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let capacity = (elements.len() as u64).max(MIN_HEAP_CAPACITY);
        self.emit_heap_alloc_const(HEAP_ARRAY_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(array_local));
        self.emit_heap_alloc_const(capacity * HEAP_ARRAY_ENTRY_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(buffer_local));
        self.store_i64_local_at_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.store_i64_const_at_offset(
            array_local,
            HEAP_LEN_OFFSET,
            elements.len() as u64,
            function,
        );
        self.store_i64_const_at_offset(array_local, HEAP_CAP_OFFSET, capacity, function);
        function.instruction(&Instruction::GlobalGet(ARRAY_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.store_i64_local_at_offset(
            array_local,
            HEAP_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_PROTOTYPE_TAG_OFFSET,
            ValueKind::Array.tag() as u64,
            function,
        );
        self.emit_init_array_exotic_slots(array_local, function);

        let entry_local = self.reserve_temp_local();
        let present_index_local = self.reserve_temp_local();
        for (index, element) in elements.iter().enumerate() {
            function.instruction(&Instruction::LocalGet(buffer_local));
            function.instruction(&Instruction::I64Const(
                (index as u64 * HEAP_ARRAY_ENTRY_SIZE) as i64,
            ));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(entry_local));
            if matches!(element.expr, ExprIr::ArrayHole) {
                self.store_i64_const_at_offset(
                    entry_local,
                    HEAP_ARRAY_TAG_OFFSET,
                    HEAP_ARRAY_HOLE_TAG as u64,
                    function,
                );
                self.store_i64_const_at_offset(entry_local, HEAP_ARRAY_PAYLOAD_OFFSET, 0, function);
                self.store_i64_const_at_offset(
                    entry_local,
                    HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
                    0,
                    function,
                );
                continue;
            }
            let value_payload = self.reserve_temp_local();
            let value_tag = self.reserve_temp_local();
            self.compile_expr_to_locals(element, value_payload, value_tag, function)?;
            self.emit_propagate_throw_from_locals_if_needed(value_payload, value_tag, function)?;
            self.store_i64_local_at_offset(entry_local, HEAP_ARRAY_TAG_OFFSET, value_tag, function);
            self.store_i64_local_at_offset(
                entry_local,
                HEAP_ARRAY_PAYLOAD_OFFSET,
                value_payload,
                function,
            );
            self.store_i64_const_at_offset(
                entry_local,
                HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
                ARRAY_DESCRIPTOR_NORMAL_DATA,
                function,
            );
            function.instruction(&Instruction::I64Const(index as i64));
            function.instruction(&Instruction::LocalSet(present_index_local));
            self.emit_array_append_present_index(
                array_local,
                present_index_local,
                value_payload,
                value_tag,
                function,
            )?;
            self.release_temp_local(value_tag);
            self.release_temp_local(value_payload);
        }
        self.release_temp_local(present_index_local);
        self.release_temp_local(entry_local);

        function.instruction(&Instruction::LocalGet(array_local));
        self.release_temp_local(buffer_local);
        self.release_temp_local(array_local);
        Ok(())
    }

    /// Emits 13.2.4.1 ArrayAccumulation against a compiler-owned fresh array.
    ///
    /// There is deliberately no dense-array spread path: every spread reads
    /// `@@iterator` and runs the sync iterator protocol. Direct fresh-array
    /// writes implement CreateDataPropertyOrThrow without consulting inherited
    /// setters. Once the logical index reaches 2^32-1 it becomes an ordinary
    /// named property and stops advancing `length`; elisions at that boundary
    /// throw RangeError instead.
    pub(crate) fn compile_array_accumulation_payload(
        &mut self,
        accumulation: &ArrayAccumulationIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let array_local = self.reserve_temp_local();
        let array_tag_local = self.reserve_temp_local();
        let next_index_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();
        let next_index_carrier_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();

        let suspension_slots = match accumulation.target() {
            ArrayAccumulationTargetIr::Fresh => {
                self.compile_array_literal_payload(&[], function)?;
                function.instruction(&Instruction::LocalSet(array_local));
                function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
                function.instruction(&Instruction::LocalSet(array_tag_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(next_index_local));
                None
            }
            ArrayAccumulationTargetIr::SuspensionOwned(slots) => {
                let array_storage =
                    self.lookup_binding(slots.array().as_str()).ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "array accumulator binding `{}` was not allocated",
                            slots.array().as_str()
                        ))
                    })?;
                let next_index_storage = self
                    .lookup_binding(slots.next_index().as_str())
                    .ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "array accumulator index binding `{}` was not allocated",
                            slots.next_index().as_str()
                        ))
                    })?;
                self.read_binding_to_locals(array_storage, array_local, array_tag_local, function)?;
                self.read_binding_to_locals(
                    next_index_storage,
                    next_index_local,
                    next_index_carrier_tag_local,
                    function,
                )?;
                Some(next_index_storage)
            }
        };

        for element in accumulation.elements() {
            match element {
                ArrayAccumulationElementIr::Elision => {
                    function.instruction(&Instruction::LocalGet(next_index_local));
                    function.instruction(&Instruction::I64Const(MAX_ARRAY_LENGTH as i64));
                    function.instruction(&Instruction::I64GeU);
                    self.open_frame(ControlFrameKind::If, function);
                    self.emit_throw_runtime_error(
                        RANGE_ERROR_NAME,
                        "Invalid array length",
                        self.result_local,
                        self.result_tag_local,
                        function,
                    )?;
                    self.emit_propagate_current_throw(function);
                    self.pop_control(ControlFrameKind::If);
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::LocalGet(next_index_local));
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::I64Add);
                    function.instruction(&Instruction::LocalSet(next_index_local));
                    self.store_i64_local_at_offset(
                        array_local,
                        HEAP_LEN_OFFSET,
                        next_index_local,
                        function,
                    );
                }
                ArrayAccumulationElementIr::Value(value) => {
                    self.compile_expr_to_locals(
                        value,
                        value_payload_local,
                        value_tag_local,
                        function,
                    )?;
                    self.emit_propagate_throw_from_locals_if_needed(
                        value_payload_local,
                        value_tag_local,
                        function,
                    )?;
                    self.emit_array_accumulation_append(
                        array_local,
                        next_index_local,
                        index_number_payload_local,
                        key_local,
                        value_payload_local,
                        value_tag_local,
                        function,
                    )?;
                }
                ArrayAccumulationElementIr::Spread(spread) => {
                    let source_payload_local = self.reserve_temp_local();
                    let source_tag_local = self.reserve_temp_local();
                    let method_payload_local = self.reserve_temp_local();
                    let method_tag_local = self.reserve_temp_local();
                    let iterator_locals = self.reserve_sync_iterator_locals();
                    let done_local = self.reserve_temp_local();
                    let consumer = SyncIteratorConsumer::ArrayAccumulation;

                    self.compile_expr_to_locals(
                        &spread.value,
                        source_payload_local,
                        source_tag_local,
                        function,
                    )?;
                    self.emit_propagate_throw_from_locals_if_needed(
                        source_payload_local,
                        source_tag_local,
                        function,
                    )?;
                    self.emit_get_iterator_from_value_locals(
                        spread.value.value_info(),
                        source_payload_local,
                        source_tag_local,
                        method_payload_local,
                        method_tag_local,
                        &iterator_locals,
                        &consumer,
                        function,
                    )?;
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(done_local));

                    let break_target = self.open_frame(ControlFrameKind::Block, function);
                    let loop_target = self.open_frame(ControlFrameKind::Loop, function);
                    self.emit_sync_iterator_step_value(
                        &iterator_locals,
                        done_local,
                        &consumer,
                        function,
                    )?;
                    function.instruction(&Instruction::LocalGet(done_local));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::I64Ne);
                    self.emit_branch_if_to_target(break_target, function);
                    self.emit_array_accumulation_append(
                        array_local,
                        next_index_local,
                        index_number_payload_local,
                        key_local,
                        iterator_locals.value_payload,
                        iterator_locals.value_tag,
                        function,
                    )?;
                    self.emit_branch_to_target(loop_target, function);
                    self.pop_control(ControlFrameKind::Loop);
                    function.instruction(&Instruction::End);
                    self.pop_control(ControlFrameKind::Block);
                    function.instruction(&Instruction::End);

                    self.release_temp_local(done_local);
                    self.release_sync_iterator_locals(iterator_locals);
                    self.release_temp_local(method_tag_local);
                    self.release_temp_local(method_payload_local);
                    self.release_temp_local(source_tag_local);
                    self.release_temp_local(source_payload_local);
                }
            }
        }

        if let Some(next_index_storage) = suspension_slots {
            function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
            function.instruction(&Instruction::LocalSet(next_index_carrier_tag_local));
            self.write_binding_from_locals(
                next_index_storage,
                next_index_local,
                next_index_carrier_tag_local,
                function,
            );
        }

        function.instruction(&Instruction::LocalGet(array_local));
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(next_index_carrier_tag_local);
        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(next_index_local);
        self.release_temp_local(array_tag_local);
        self.release_temp_local(array_local);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_array_accumulation_append(
        &mut self,
        array_local: u32,
        next_index_local: u32,
        index_number_payload_local: u32,
        key_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // The suspension-owned carrier is an exact u64. Refuse the next
        // contribution at its upper bound instead of allowing Wasm addition to
        // wrap the logical ArrayAccumulation index to zero.
        function.instruction(&Instruction::LocalGet(next_index_local));
        function.instruction(&Instruction::I64Const(u64::MAX as i64));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_runtime_error(
            RANGE_ERROR_NAME,
            "Array accumulation index exceeds exact backend range",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_current_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(next_index_local));
        function.instruction(&Instruction::I64Const(MAX_ARRAY_LENGTH as i64));
        function.instruction(&Instruction::I64LtU);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_array_write(
            array_local,
            next_index_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(next_index_local));
        function.instruction(&Instruction::I64Const(MAX_ARRAY_LENGTH as i64));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(self.strings.payload("4294967295")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(next_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_number_to_string_payload(index_number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_local));
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_array_define_named_data_property(
            array_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(next_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(next_index_local));
        Ok(())
    }

    pub(crate) fn emit_array_sparse_present_read(
        &mut self,
        array_local: u32,
        index_local: u32,
        payload_local: u32,
        tag_local: u32,
        found_output_local: Option<u32>,
        function: &mut Function,
    ) {
        let list_ptr_local = self.reserve_temp_local();
        let list_len_local = self.reserve_temp_local();
        let list_index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let candidate_index_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_PTR_OFFSET,
            list_ptr_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_LEN_OFFSET,
            list_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(list_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(list_ptr_local));
        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_PRESENT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_INDEX_OFFSET,
            candidate_index_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_TAG_OFFSET,
            tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_HOLE_TAG));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        if let Some(found_output_local) = found_output_local {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(found_output_local));
        }
        function.instruction(&Instruction::Else);
        if let Some(found_output_local) = found_output_local {
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(found_output_local));
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::LocalSet(list_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(list_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(candidate_index_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(list_index_local);
        self.release_temp_local(list_len_local);
        self.release_temp_local(list_ptr_local);
    }

    pub(crate) fn emit_array_sparse_present_get(
        &mut self,
        array_local: u32,
        index_local: u32,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        payload_local: u32,
        tag_local: u32,
        found_output_local: Option<u32>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let list_ptr_local = self.reserve_temp_local();
        let list_len_local = self.reserve_temp_local();
        let list_index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let candidate_index_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let getter_payload_local = self.reserve_temp_local();
        let getter_tag_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_PTR_OFFSET,
            list_ptr_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_LEN_OFFSET,
            list_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(list_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(list_ptr_local));
        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_PRESENT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_INDEX_OFFSET,
            candidate_index_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_TAG_OFFSET,
            getter_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_PAYLOAD_OFFSET,
            getter_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        if let Some(found_output_local) = found_output_local {
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(found_output_local));
        }
        self.emit_is_callable_i32(getter_tag_local, getter_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        if self.outline_proxy_call {
            self.emit_function_or_proxy_call_leave_throw_completion(
                getter_payload_local,
                getter_tag_local,
                receiver_payload_local,
                receiver_tag_local,
                &[],
                payload_local,
                tag_local,
                function,
            )?;
            self.emit_break_current_completion_if_throw(5, function);
        } else {
            // The shared Proxy-call helper snapshots its own internal argv
            // through this Array path. Those entries cannot be accessors; keep
            // the unreachable accessor branch finite while compiling that
            // helper instead of recursively embedding another Proxy dispatcher.
            self.emit_function_handle_call_with_throw_propagation(
                getter_payload_local,
                getter_tag_local,
                Some((receiver_payload_local, Some(receiver_tag_local))),
                &[],
                payload_local,
                tag_local,
                function,
            )?;
        }
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(getter_tag_local));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::LocalGet(getter_payload_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_HOLE_TAG));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        if let Some(found_output_local) = found_output_local {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(found_output_local));
        }
        function.instruction(&Instruction::Else);
        if let Some(found_output_local) = found_output_local {
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(found_output_local));
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::LocalSet(list_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(list_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(getter_tag_local);
        self.release_temp_local(getter_payload_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(candidate_index_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(list_index_local);
        self.release_temp_local(list_len_local);
        self.release_temp_local(list_ptr_local);
        Ok(())
    }

    pub(crate) fn emit_array_read(
        &mut self,
        array_local: u32,
        index_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let cap_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(array_local, HEAP_LEN_OFFSET, len_local, function);
        self.load_i64_to_local_from_offset(array_local, HEAP_CAP_OFFSET, cap_local, function);
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(MAX_ARRAY_LENGTH as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("4294967295")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_array_named_prop_read(
            array_local,
            key_local,
            payload_local,
            tag_local,
            None,
            function,
        );
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(0));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_sparse_present_read(
            array_local,
            index_local,
            payload_local,
            tag_local,
            None,
            function,
        );
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(entry_local, HEAP_ARRAY_TAG_OFFSET, tag_local, function);
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_HOLE_TAG));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(key_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(cap_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
    }

    pub(crate) fn emit_array_index_get(
        &mut self,
        array_local: u32,
        index_local: u32,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        payload_local: u32,
        tag_local: u32,
        found_output_local: Option<u32>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let cap_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let getter_payload_local = self.reserve_temp_local();
        let getter_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(array_local, HEAP_LEN_OFFSET, len_local, function);
        self.load_i64_to_local_from_offset(array_local, HEAP_CAP_OFFSET, cap_local, function);
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        if let Some(found_output_local) = found_output_local {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(found_output_local));
        }

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(MAX_ARRAY_LENGTH as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("4294967295")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_array_named_prop_read(
            array_local,
            key_local,
            payload_local,
            tag_local,
            found_output_local,
            function,
        );
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(0));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_sparse_present_get(
            array_local,
            index_local,
            receiver_payload_local,
            receiver_tag_local,
            payload_local,
            tag_local,
            found_output_local,
            function,
        )?;
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        if let Some(found_output_local) = found_output_local {
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(found_output_local));
        }
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_PAYLOAD_OFFSET,
            getter_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_TAG_OFFSET,
            getter_tag_local,
            function,
        );
        self.emit_is_callable_i32(getter_tag_local, getter_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        if self.outline_proxy_call {
            self.emit_function_or_proxy_call_leave_throw_completion(
                getter_payload_local,
                getter_tag_local,
                receiver_payload_local,
                receiver_tag_local,
                &[],
                payload_local,
                tag_local,
                function,
            )?;
        } else {
            // See the sparse-present path above: this branch is emitted only
            // while compiling the Proxy-call helper's internal argv snapshot.
            self.emit_function_handle_call_without_throw_propagation(
                getter_payload_local,
                getter_tag_local,
                Some((receiver_payload_local, Some(receiver_tag_local))),
                &[],
                payload_local,
                tag_local,
                function,
            )?;
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(entry_local, HEAP_ARRAY_TAG_OFFSET, tag_local, function);
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_HOLE_TAG));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::Else);
        if let Some(found_output_local) = found_output_local {
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(found_output_local));
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(key_local);
        self.release_temp_local(getter_tag_local);
        self.release_temp_local(getter_payload_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(cap_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    pub(crate) fn emit_array_index_get_with_prototype(
        &mut self,
        array_local: u32,
        index_local: u32,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let found_local = self.reserve_temp_local();
        let prototype_local = self.reserve_temp_local();
        let prototype_tag_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();

        self.emit_array_index_get(
            array_local,
            index_local,
            receiver_payload_local,
            receiver_tag_local,
            payload_local,
            tag_local,
            Some(found_local),
            function,
        )?;
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_PROTOTYPE_OFFSET,
            prototype_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_PROTOTYPE_TAG_OFFSET,
            prototype_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_number_to_string_payload(index_number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_payload_local));
        self.emit_object_read_ordinary(
            prototype_local,
            prototype_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_payload_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(key_payload_local);
        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(prototype_tag_local);
        self.release_temp_local(prototype_local);
        self.release_temp_local(found_local);
        Ok(())
    }

    pub(crate) fn emit_array_own_index_present_i64(
        &mut self,
        array_local: u32,
        index_local: u32,
        found_local: u32,
        function: &mut Function,
    ) {
        let buffer_local = self.reserve_temp_local();
        let cap_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let scratch_payload_local = self.reserve_temp_local();
        let scratch_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(found_local));
        self.load_i64_to_local_from_offset(array_local, HEAP_CAP_OFFSET, cap_local, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(MAX_DENSE_ARRAY_INDEX as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_sparse_present_read(
            array_local,
            index_local,
            scratch_payload_local,
            scratch_tag_local,
            Some(found_local),
            function,
        );
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(scratch_tag_local);
        self.release_temp_local(scratch_payload_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(cap_local);
        self.release_temp_local(buffer_local);
    }

    pub(crate) fn emit_array_assignment_write(
        &mut self,
        array_local: u32,
        index_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let present_local = self.reserve_temp_local();
        let state_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(
            ArrayInheritedIndexSetState::Unhandled.code(),
        ));
        function.instruction(&Instruction::LocalSet(state_local));
        self.emit_array_own_index_present_i64(array_local, index_local, present_local, function);
        function.instruction(&Instruction::LocalGet(present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_inherited_index_set_state(
            array_local,
            index_local,
            payload_local,
            tag_local,
            state_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(
            ArrayInheritedIndexSetState::OrdinaryRejected.code(),
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_write_set_failure_else("Cannot assign to read only property", function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(
            ArrayInheritedIndexSetState::ProxyRejected.code(),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_assignment_proxy_set_false_result_if_strict(function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_set_index_result(
            array_local,
            index_local,
            payload_local,
            tag_local,
            present_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_write_set_failure_else("Cannot assign to array index", function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(state_local);
        self.release_temp_local(present_local);
        Ok(())
    }

    pub(crate) fn emit_array_inherited_index_set_state(
        &mut self,
        array_local: u32,
        index_local: u32,
        payload_local: u32,
        tag_local: u32,
        state_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let prototype_local = self.reserve_temp_local();
        let prototype_tag_local = self.reserve_temp_local();
        let next_prototype_local = self.reserve_temp_local();
        let prototype_buffer_local = self.reserve_temp_local();
        let prototype_len_local = self.reserve_temp_local();
        let prototype_index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let entry_key_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let setter_payload_local = self.reserve_temp_local();
        let setter_tag_local = self.reserve_temp_local();
        let setter_result_payload_local = self.reserve_temp_local();
        let setter_result_tag_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let found_local = self.reserve_temp_local();
        let prototype_proxy_kind_local = self.reserve_temp_local();
        let reflect_set_payload_local = self.reserve_temp_local();
        let reflect_set_tag_local = self.reserve_temp_local();
        let reflect_set_result_payload_local = self.reserve_temp_local();
        let reflect_set_result_tag_local = self.reserve_temp_local();
        let prototype_brand_local = self.reserve_temp_local();
        let typed_array_index_local = self.reserve_temp_local();
        let typed_array_index_valid_local = self.reserve_temp_local();

        let reflect_set_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectSet.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Reflect.set`",
                )
            })?;

        function.instruction(&Instruction::I64Const(
            ArrayInheritedIndexSetState::Unhandled.code(),
        ));
        function.instruction(&Instruction::LocalSet(state_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        self.emit_index_to_flat_map_key_local(
            index_local,
            index_number_payload_local,
            key_payload_local,
            function,
        )?;
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_PROTOTYPE_OFFSET,
            prototype_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_PROTOTYPE_TAG_OFFSET,
            prototype_tag_local,
            function,
        );

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(prototype_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(found_local));

        self.load_i64_to_local_from_offset(
            prototype_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            prototype_brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TYPED_ARRAY as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(prototype_tag_local));
        self.emit_typed_array_valid_integer_index_i32(
            prototype_local,
            index_number_payload_local,
            typed_array_index_local,
            typed_array_index_valid_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(typed_array_index_valid_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            ArrayInheritedIndexSetState::Handled.code(),
        ));
        function.instruction(&Instruction::LocalSet(state_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::LocalSet(prototype_tag_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(prototype_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_descriptor_kind_for_index(
            prototype_local,
            index_local,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_accessor_setter_for_index(
            prototype_local,
            index_local,
            setter_payload_local,
            setter_tag_local,
            function,
        );
        self.emit_is_callable_i32(setter_tag_local, setter_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            ArrayInheritedIndexSetState::Setter.code(),
        ));
        function.instruction(&Instruction::LocalSet(state_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(
            ArrayInheritedIndexSetState::OrdinaryRejected.code(),
        ));
        function.instruction(&Instruction::LocalSet(state_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_WRITABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            ArrayInheritedIndexSetState::OrdinaryRejected.code(),
        ));
        function.instruction(&Instruction::LocalSet(state_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);

        function.instruction(&Instruction::LocalGet(prototype_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            prototype_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            prototype_proxy_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_proxy_kind_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_value_payload(&reflect_set_meta, function)?;
        function.instruction(&Instruction::LocalSet(reflect_set_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(reflect_set_tag_local));
        self.emit_function_handle_call(
            reflect_set_payload_local,
            reflect_set_tag_local,
            None,
            &[
                (prototype_local, prototype_tag_local),
                (key_payload_local, key_tag_local),
                (payload_local, tag_local),
                (array_local, receiver_tag_local),
            ],
            reflect_set_result_payload_local,
            reflect_set_result_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.compile_truthy_tagged_i32(
            reflect_set_result_tag_local,
            reflect_set_result_payload_local,
            function,
        )?;
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            ArrayInheritedIndexSetState::Handled.code(),
        ));
        function.instruction(&Instruction::LocalSet(state_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(
            ArrayInheritedIndexSetState::ProxyRejected.code(),
        ));
        function.instruction(&Instruction::LocalSet(state_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(prototype_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(prototype_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            prototype_local,
            HEAP_PTR_OFFSET,
            prototype_buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            prototype_local,
            HEAP_LEN_OFFSET,
            prototype_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(prototype_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(prototype_index_local));
        function.instruction(&Instruction::LocalGet(prototype_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(prototype_buffer_local));
        function.instruction(&Instruction::LocalGet(prototype_index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            entry_key_local,
            function,
        );
        self.emit_string_payload_equality_i32(entry_key_local, key_payload_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(found_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_SETTER_TAG_OFFSET,
            setter_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_SETTER_PAYLOAD_OFFSET,
            setter_payload_local,
            function,
        );
        self.emit_is_callable_i32(setter_tag_local, setter_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            ArrayInheritedIndexSetState::Setter.code(),
        ));
        function.instruction(&Instruction::LocalSet(state_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(
            ArrayInheritedIndexSetState::OrdinaryRejected.code(),
        ));
        function.instruction(&Instruction::LocalSet(state_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_WRITABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            ArrayInheritedIndexSetState::OrdinaryRejected.code(),
        ));
        function.instruction(&Instruction::LocalSet(state_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(prototype_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(prototype_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(found_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(prototype_local));
        function.instruction(&Instruction::Else);
        self.emit_load_prototype_to_current_locals(
            prototype_local,
            prototype_tag_local,
            next_prototype_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(
            ArrayInheritedIndexSetState::Setter.code(),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_handle_call(
            setter_payload_local,
            setter_tag_local,
            Some((array_local, Some(receiver_tag_local))),
            &[(payload_local, tag_local)],
            setter_result_payload_local,
            setter_result_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            setter_result_payload_local,
            setter_result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.release_temp_local(typed_array_index_valid_local);
        self.release_temp_local(typed_array_index_local);
        self.release_temp_local(prototype_brand_local);
        self.release_temp_local(reflect_set_result_tag_local);
        self.release_temp_local(reflect_set_result_payload_local);
        self.release_temp_local(reflect_set_tag_local);
        self.release_temp_local(reflect_set_payload_local);
        self.release_temp_local(prototype_proxy_kind_local);
        self.release_temp_local(found_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(setter_result_tag_local);
        self.release_temp_local(setter_result_payload_local);
        self.release_temp_local(setter_tag_local);
        self.release_temp_local(setter_payload_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_key_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(prototype_index_local);
        self.release_temp_local(prototype_len_local);
        self.release_temp_local(prototype_buffer_local);
        self.release_temp_local(next_prototype_local);
        self.release_temp_local(prototype_tag_local);
        self.release_temp_local(prototype_local);
        Ok(())
    }

    pub(crate) fn emit_string_index_read(
        &mut self,
        string_payload_local: u32,
        index_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let offset_local = self.reserve_temp_local();
        let byte_len_local = self.reserve_temp_local();
        let unit_len_local = self.reserve_temp_local();
        let byte_index_local = self.reserve_temp_local();
        let unit_index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let codepoint_local = self.reserve_temp_local();
        let advance_local = self.reserve_temp_local();
        let unit_advance_local = self.reserve_temp_local();
        let temp_local = self.reserve_temp_local();
        let code_unit_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));

        self.emit_unpack_string_payload(
            string_payload_local,
            offset_local,
            byte_len_local,
            function,
        );
        self.emit_utf16_code_unit_len_from_utf8_locals(
            offset_local,
            byte_len_local,
            unit_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(unit_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(byte_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(unit_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_index_local));
        function.instruction(&Instruction::LocalGet(byte_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(offset_local, byte_index_local, byte_local, function);
        self.emit_decode_utf8_scalar_at_index(
            offset_local,
            byte_index_local,
            byte_len_local,
            byte_local,
            codepoint_local,
            advance_local,
            temp_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(0xFFFF));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(unit_advance_local));
        function.instruction(&Instruction::LocalGet(unit_index_local));
        function.instruction(&Instruction::LocalGet(unit_advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(0xFFFF));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(temp_local));
        function.instruction(&Instruction::LocalGet(unit_index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0xD800));
        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(code_unit_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0xDC00));
        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::I64Const(0x3FF));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(code_unit_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::LocalSet(code_unit_local));
        function.instruction(&Instruction::End);
        self.emit_string_payload_from_utf16_code_unit_local(
            code_unit_local,
            payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(byte_index_local));
        function.instruction(&Instruction::LocalGet(advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(byte_index_local));
        function.instruction(&Instruction::LocalGet(unit_index_local));
        function.instruction(&Instruction::LocalGet(unit_advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(unit_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(code_unit_local);
        self.release_temp_local(temp_local);
        self.release_temp_local(unit_advance_local);
        self.release_temp_local(advance_local);
        self.release_temp_local(codepoint_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(unit_index_local);
        self.release_temp_local(byte_index_local);
        self.release_temp_local(unit_len_local);
        self.release_temp_local(byte_len_local);
        self.release_temp_local(offset_local);
        Ok(())
    }

    pub(crate) fn emit_string_payload_from_utf16_code_unit_local(
        &mut self,
        code_unit_local: u32,
        payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let len_local = self.reserve_temp_local();
        let offset_local = self.reserve_temp_local();
        let pos_local = self.reserve_temp_local();
        let temp_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(code_unit_local));
        function.instruction(&Instruction::I64Const(0x80));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(code_unit_local));
        function.instruction(&Instruction::I64Const(0x800));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_heap_alloc_from_local(len_local, function)?;
        function.instruction(&Instruction::LocalSet(offset_local));
        function.instruction(&Instruction::LocalGet(offset_local));
        function.instruction(&Instruction::LocalSet(pos_local));
        self.emit_store_utf8_codepoint(pos_local, code_unit_local, temp_local, function);
        self.emit_pack_string_payload(offset_local, len_local, function);
        function.instruction(&Instruction::LocalSet(payload_local));

        self.release_temp_local(temp_local);
        self.release_temp_local(pos_local);
        self.release_temp_local(offset_local);
        self.release_temp_local(len_local);
        Ok(())
    }

    pub(crate) fn emit_array_length(
        &mut self,
        array_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        self.load_i64_to_local_from_offset(array_local, HEAP_LEN_OFFSET, payload_local, function);
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
    }

    pub(crate) fn emit_array_or_object_length_read(
        &mut self,
        target_local: u32,
        target_tag_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_length(target_local, payload_local, tag_local, function);
        function.instruction(&Instruction::Else);

        let key_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            target_local,
            target_tag_local,
            target_local,
            target_tag_local,
            key_local,
            payload_local,
            tag_local,
            function,
        )?;
        self.release_temp_local(key_local);

        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_array_length_writable_i64(
        &mut self,
        array_local: u32,
        writable_local: u32,
        function: &mut Function,
    ) {
        let descriptor_kind_local = self.reserve_temp_local();
        let stored_key_local = self.reserve_temp_local();
        let length_key_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(writable_local));
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_PROP_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_PROP_GETTER_PAYLOAD_OFFSET,
            stored_key_local,
            function,
        );
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(length_key_local));
        self.emit_string_payload_equality_i32(stored_key_local, length_key_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_WRITABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(writable_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(length_key_local);
        self.release_temp_local(stored_key_local);
        self.release_temp_local(descriptor_kind_local);
    }

    pub(crate) fn emit_array_store_length_writable_descriptor(
        &mut self,
        array_local: u32,
        writable_payload_local: u32,
        function: &mut Function,
    ) {
        let descriptor_kind_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(
            (ARRAY_DESCRIPTOR_OWN_PROPERTY | OBJECT_DESCRIPTOR_DATA) as i64,
        ));
        function.instruction(&Instruction::LocalSet(descriptor_kind_local));
        function.instruction(&Instruction::LocalGet(writable_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_WRITABLE as i64));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(descriptor_kind_local));
        function.instruction(&Instruction::End);
        self.store_i64_local_at_offset(
            array_local,
            HEAP_ARRAY_PROP_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_PROP_DATA_TAG_OFFSET,
            ValueKind::Undefined.tag() as u64,
            function,
        );
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_PROP_DATA_PAYLOAD_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_PROP_GETTER_TAG_OFFSET,
            ValueKind::Undefined.tag() as u64,
            function,
        );
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_PROP_GETTER_PAYLOAD_OFFSET,
            self.strings.payload("length") as u64,
            function,
        );
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_PROP_SETTER_TAG_OFFSET,
            ValueKind::Undefined.tag() as u64,
            function,
        );
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_PROP_SETTER_PAYLOAD_OFFSET,
            0,
            function,
        );

        self.release_temp_local(descriptor_kind_local);
    }

    pub(crate) fn emit_to_repeat_count_i64_from_value_locals(
        &mut self,
        tag_local: u32,
        payload_local: u32,
        dest_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_value_to_number_payload(tag_local, payload_local, function)?;
        function.instruction(&Instruction::LocalSet(payload_local));
        self.emit_return_current_completion_if_throw(function);

        self.emit_to_integer_or_infinity_number_payload_from_number_payload(
            payload_local,
            payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "repeat count must be non-negative and finite",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncSatF64U);
        function.instruction(&Instruction::LocalSet(dest_local));
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_array_grow_buffer(
        &mut self,
        array_local: u32,
        buffer_local: u32,
        len_local: u32,
        cap_local: u32,
        required_index_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let new_cap_local = self.reserve_temp_local();
        let required_len_local = self.reserve_temp_local();
        let size_local = self.reserve_temp_local();
        let new_buffer_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let old_entry_local = self.reserve_temp_local();
        let new_entry_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(MIN_HEAP_CAPACITY as i64));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(new_cap_local));

        function.instruction(&Instruction::LocalGet(required_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(required_len_local));
        function.instruction(&Instruction::LocalGet(required_len_local));
        function.instruction(&Instruction::LocalGet(new_cap_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(required_len_local));
        function.instruction(&Instruction::LocalSet(new_cap_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(new_cap_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(size_local));
        self.emit_heap_alloc_from_local(size_local, function)?;
        function.instruction(&Instruction::LocalSet(new_buffer_local));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(old_entry_local));

        function.instruction(&Instruction::LocalGet(new_buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(new_entry_local));

        for offset in [
            HEAP_ARRAY_TAG_OFFSET,
            HEAP_ARRAY_PAYLOAD_OFFSET,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            HEAP_ARRAY_SETTER_TAG_OFFSET,
            HEAP_ARRAY_SETTER_PAYLOAD_OFFSET,
        ] {
            self.load_i64_from_offset(old_entry_local, offset, function);
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.store_i64_local_at_offset(new_entry_local, offset, self.scratch_local, function);
        }

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(new_buffer_local));
        function.instruction(&Instruction::LocalSet(buffer_local));
        function.instruction(&Instruction::LocalGet(new_cap_local));
        function.instruction(&Instruction::LocalSet(cap_local));
        self.store_i64_local_at_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.store_i64_local_at_offset(array_local, HEAP_CAP_OFFSET, cap_local, function);

        self.release_temp_local(new_entry_local);
        self.release_temp_local(old_entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(new_buffer_local);
        self.release_temp_local(size_local);
        self.release_temp_local(required_len_local);
        self.release_temp_local(new_cap_local);
        Ok(())
    }

    pub(crate) fn emit_array_append_present_index(
        &mut self,
        array_local: u32,
        index_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let list_ptr_local = self.reserve_temp_local();
        let list_len_local = self.reserve_temp_local();
        let list_cap_local = self.reserve_temp_local();
        let new_cap_local = self.reserve_temp_local();
        let size_local = self.reserve_temp_local();
        let new_list_ptr_local = self.reserve_temp_local();
        let copy_index_local = self.reserve_temp_local();
        let old_entry_local = self.reserve_temp_local();
        let new_entry_local = self.reserve_temp_local();
        let candidate_index_local = self.reserve_temp_local();
        let found_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_PTR_OFFSET,
            list_ptr_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_LEN_OFFSET,
            list_len_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_CAP_OFFSET,
            list_cap_local,
            function,
        );

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(copy_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(copy_index_local));
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(list_ptr_local));
        function.instruction(&Instruction::LocalGet(copy_index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_PRESENT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(old_entry_local));
        self.load_i64_to_local_from_offset(
            old_entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_INDEX_OFFSET,
            candidate_index_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_local_at_offset(
            old_entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_TAG_OFFSET,
            tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            old_entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::LocalSet(copy_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(copy_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(copy_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::LocalGet(list_cap_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(list_cap_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(list_cap_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(new_cap_local));

        function.instruction(&Instruction::LocalGet(new_cap_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_PRESENT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(size_local));
        self.emit_heap_alloc_from_local(size_local, function)?;
        function.instruction(&Instruction::LocalSet(new_list_ptr_local));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(copy_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(copy_index_local));
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(list_ptr_local));
        function.instruction(&Instruction::LocalGet(copy_index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_PRESENT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(old_entry_local));
        function.instruction(&Instruction::LocalGet(new_list_ptr_local));
        function.instruction(&Instruction::LocalGet(copy_index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_PRESENT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(new_entry_local));
        for offset in [
            HEAP_ARRAY_PRESENT_ENTRY_INDEX_OFFSET,
            HEAP_ARRAY_PRESENT_ENTRY_TAG_OFFSET,
            HEAP_ARRAY_PRESENT_ENTRY_PAYLOAD_OFFSET,
            HEAP_ARRAY_PRESENT_ENTRY_SETTER_TAG_OFFSET,
            HEAP_ARRAY_PRESENT_ENTRY_SETTER_PAYLOAD_OFFSET,
            HEAP_ARRAY_PRESENT_ENTRY_DESCRIPTOR_KIND_OFFSET,
        ] {
            self.load_i64_from_offset(old_entry_local, offset, function);
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.store_i64_local_at_offset(new_entry_local, offset, self.scratch_local, function);
        }
        function.instruction(&Instruction::LocalGet(copy_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(copy_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(new_list_ptr_local));
        function.instruction(&Instruction::LocalSet(list_ptr_local));
        function.instruction(&Instruction::LocalGet(new_cap_local));
        function.instruction(&Instruction::LocalSet(list_cap_local));
        self.store_i64_local_at_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_PTR_OFFSET,
            list_ptr_local,
            function,
        );
        self.store_i64_local_at_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_CAP_OFFSET,
            list_cap_local,
            function,
        );
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(list_ptr_local));
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_PRESENT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(new_entry_local));
        self.store_i64_local_at_offset(
            new_entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_INDEX_OFFSET,
            index_local,
            function,
        );
        self.store_i64_local_at_offset(
            new_entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_TAG_OFFSET,
            tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            new_entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        self.store_i64_const_at_offset(
            new_entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_DESCRIPTOR_KIND_OFFSET,
            ARRAY_DESCRIPTOR_NORMAL_DATA,
            function,
        );
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(list_len_local));
        self.store_i64_local_at_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_LEN_OFFSET,
            list_len_local,
            function,
        );
        function.instruction(&Instruction::End);

        self.release_temp_local(found_local);
        self.release_temp_local(candidate_index_local);
        self.release_temp_local(new_entry_local);
        self.release_temp_local(old_entry_local);
        self.release_temp_local(copy_index_local);
        self.release_temp_local(new_list_ptr_local);
        self.release_temp_local(size_local);
        self.release_temp_local(new_cap_local);
        self.release_temp_local(list_cap_local);
        self.release_temp_local(list_len_local);
        self.release_temp_local(list_ptr_local);
        Ok(())
    }

    pub(crate) fn emit_array_write(
        &mut self,
        array_local: u32,
        index_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if self.outline_array_write {
            if let Some(helper) = self.array_write_helper_function_index() {
                let helper_payload_local = self.reserve_temp_local();
                let helper_tag_local = self.reserve_temp_local();
                function.instruction(&Instruction::LocalGet(array_local));
                function.instruction(&Instruction::LocalGet(index_local));
                function.instruction(&Instruction::LocalGet(payload_local));
                function.instruction(&Instruction::LocalGet(tag_local));
                function.instruction(&Instruction::LocalGet(self.current_env_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::Call(helper));
                self.store_call_results(helper_payload_local, helper_tag_local, function);
                self.emit_propagate_throw_from_locals_if_needed(
                    helper_payload_local,
                    helper_tag_local,
                    function,
                )?;
                self.release_temp_local(helper_tag_local);
                self.release_temp_local(helper_payload_local);
                return Ok(());
            }
        }
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let cap_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let existing_descriptor_kind_local = self.reserve_temp_local();

        self.emit_array_descriptor_kind_for_index(
            array_local,
            index_local,
            existing_descriptor_kind_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_NON_EXTENSIBLE_OFFSET,
            self.scratch_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(self.scratch_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Cannot add property to non-extensible array",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(array_local, HEAP_LEN_OFFSET, len_local, function);
        self.load_i64_to_local_from_offset(array_local, HEAP_CAP_OFFSET, cap_local, function);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(MAX_DENSE_ARRAY_INDEX as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_append_present_index(
            array_local,
            index_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(len_local));
        self.store_i64_local_at_offset(array_local, HEAP_LEN_OFFSET, len_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64Const(
            SPARSE_ARRAY_DENSE_GROW_FACTOR as i64,
        ));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_append_present_index(
            array_local,
            index_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(len_local));
        self.store_i64_local_at_offset(array_local, HEAP_LEN_OFFSET, len_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.emit_array_grow_buffer(
            array_local,
            buffer_local,
            cap_local,
            cap_local,
            index_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            existing_descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_append_present_index(
            array_local,
            index_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.store_i64_local_at_offset(entry_local, HEAP_ARRAY_TAG_OFFSET, tag_local, function);
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_ARRAY_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        self.store_i64_const_at_offset(
            entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            ARRAY_DESCRIPTOR_NORMAL_DATA,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(len_local));
        self.store_i64_local_at_offset(array_local, HEAP_LEN_OFFSET, len_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(existing_descriptor_kind_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(cap_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    pub(crate) fn emit_array_accessor_setter_for_index(
        &mut self,
        array_local: u32,
        index_local: u32,
        setter_payload_local: u32,
        setter_tag_local: u32,
        function: &mut Function,
    ) {
        let cap_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let list_ptr_local = self.reserve_temp_local();
        let list_len_local = self.reserve_temp_local();
        let list_index_local = self.reserve_temp_local();
        let candidate_index_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(setter_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(setter_tag_local));
        self.load_i64_to_local_from_offset(array_local, HEAP_CAP_OFFSET, cap_local, function);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_SETTER_PAYLOAD_OFFSET,
            setter_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_SETTER_TAG_OFFSET,
            setter_tag_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_PTR_OFFSET,
            list_ptr_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_LEN_OFFSET,
            list_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(list_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(list_ptr_local));
        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_PRESENT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_INDEX_OFFSET,
            candidate_index_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_SETTER_PAYLOAD_OFFSET,
            setter_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_SETTER_TAG_OFFSET,
            setter_tag_local,
            function,
        );
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(list_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(candidate_index_local);
        self.release_temp_local(list_index_local);
        self.release_temp_local(list_len_local);
        self.release_temp_local(list_ptr_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(cap_local);
    }

    pub(crate) fn emit_array_set_index_result(
        &mut self,
        array_local: u32,
        index_local: u32,
        payload_local: u32,
        tag_local: u32,
        result_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let descriptor_kind_local = self.reserve_temp_local();
        let non_extensible_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let length_writable_local = self.reserve_temp_local();
        let setter_payload_local = self.reserve_temp_local();
        let setter_tag_local = self.reserve_temp_local();
        let setter_result_payload_local = self.reserve_temp_local();
        let setter_result_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        self.emit_array_descriptor_kind_for_index(
            array_local,
            index_local,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_WRITABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_write(array_local, index_local, payload_local, tag_local, function)?;
        self.emit_store_array_descriptor_for_index(
            array_local,
            index_local,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_array_accessor_setter_for_index(
            array_local,
            index_local,
            setter_payload_local,
            setter_tag_local,
            function,
        );
        self.emit_is_callable_i32(setter_tag_local, setter_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        if self.outline_proxy_call {
            self.emit_function_or_proxy_call_leave_throw_completion(
                setter_payload_local,
                setter_tag_local,
                array_local,
                self.scratch_local,
                &[(payload_local, tag_local)],
                setter_result_payload_local,
                setter_result_tag_local,
                function,
            )?;
        } else {
            self.emit_function_handle_call(
                setter_payload_local,
                setter_tag_local,
                Some((array_local, Some(self.scratch_local))),
                &[(payload_local, tag_local)],
                setter_result_payload_local,
                setter_result_tag_local,
                function,
            )?;
        }
        self.emit_propagate_throw_from_locals_if_needed(
            setter_result_payload_local,
            setter_result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);

        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_NON_EXTENSIBLE_OFFSET,
            non_extensible_local,
            function,
        );
        self.load_i64_to_local_from_offset(array_local, HEAP_LEN_OFFSET, len_local, function);
        self.emit_array_length_writable_i64(array_local, length_writable_local, function);
        function.instruction(&Instruction::LocalGet(non_extensible_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::LocalGet(length_writable_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_write(array_local, index_local, payload_local, tag_local, function)?;
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(setter_result_tag_local);
        self.release_temp_local(setter_result_payload_local);
        self.release_temp_local(setter_tag_local);
        self.release_temp_local(setter_payload_local);
        self.release_temp_local(length_writable_local);
        self.release_temp_local(len_local);
        self.release_temp_local(non_extensible_local);
        self.release_temp_local(descriptor_kind_local);
        Ok(())
    }

    pub(crate) fn emit_array_create_data_property_silent(
        &mut self,
        array_local: u32,
        index_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let can_define_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(can_define_local));
        self.load_i64_to_local_from_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(array_local, HEAP_LEN_OFFSET, len_local, function);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(0));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_DESCRIPTOR_CONFIGURABLE as i64,
        ));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(can_define_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(can_define_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_write(array_local, index_local, payload_local, tag_local, function)?;
        function.instruction(&Instruction::End);

        self.release_temp_local(can_define_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    /// Implements ArraySetLength steps 3--5 for a value descriptor before
    /// changing an array's backing store.  `ToUint32` and `ToNumber` must be
    /// performed independently on the original value: in particular, an
    /// object's numeric conversion can have observable effects twice.
    pub(crate) fn emit_array_set_length_from_value(
        &mut self,
        array_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        writable_payload_local: u32,
        writable_present_local: u32,
        allow_define_local: u32,
        success_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let uint32_number_payload_local = self.reserve_temp_local();
        let number_payload_local = self.reserve_temp_local();
        let uint32_local = self.reserve_temp_local();

        // ToUint32(value) begins with its own ToNumber(value).
        self.emit_value_to_number_payload_without_throw_return(
            value_tag_local,
            value_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(uint32_number_payload_local));
        self.emit_propagate_throw_from_locals_if_needed(
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        // This is the actual ToUint32 modulo operation, not a saturating
        // conversion.  It is deliberately kept local: ArraySetLength must
        // compare this first conversion with a separately-observed ToNumber.
        self.emit_to_uint32_i64_from_number_payload(
            uint32_number_payload_local,
            uint32_local,
            function,
        );

        // ArraySetLength deliberately performs ToNumber on the original
        // descriptor value again, rather than reusing ToUint32's conversion.
        self.emit_value_to_number_payload_without_throw_return(
            value_tag_local,
            value_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(number_payload_local));
        self.emit_propagate_throw_from_locals_if_needed(
            self.result_local,
            self.result_tag_local,
            function,
        )?;

        // A valid array length is exactly a uint32 Number.  Comparing the
        // independently converted values also handles changing conversions.
        function.instruction(&Instruction::LocalGet(uint32_local));
        function.instruction(&Instruction::I64Const(u32::MAX as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::LocalGet(uint32_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        // ArraySetLength throws in the current execution context's Realm, not
        // the Array object's Realm. Created-realm standard builtins carry their
        // defining Realm through the function environment. Runtime write/set
        // helpers receive that environment explicitly; ordinary user code uses
        // the main-Realm fallback.
        let has_standard_builtin_realm = self
            .function_id
            .as_ref()
            .and_then(|function_id| StandardBuiltinId::from_function_id(function_id))
            .is_some();
        if has_standard_builtin_realm || self.object_write_strict_flag_local.is_some() {
            self.emit_throw_current_function_realm_range_error(
                "Invalid array length",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
        } else {
            self.emit_throw_runtime_error(
                RANGE_ERROR_NAME,
                "Invalid array length",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
        }
        self.emit_propagate_throw_from_locals_if_needed(
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        // Descriptor fields incompatible with the non-configurable,
        // non-enumerable length property are an ordinary failure, but only
        // after ArraySetLength's two observable conversions and range check.
        function.instruction(&Instruction::LocalGet(allow_define_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(success_local));
        function.instruction(&Instruction::Else);
        // Preserve the established grow/shrink implementation, which now only
        // receives a prevalidated integral number payload.
        function.instruction(&Instruction::LocalGet(uint32_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(uint32_number_payload_local));
        self.emit_array_set_length_from_number_payload(
            array_local,
            uint32_number_payload_local,
            writable_payload_local,
            writable_present_local,
            success_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.release_temp_local(uint32_local);
        self.release_temp_local(number_payload_local);
        self.release_temp_local(uint32_number_payload_local);
        Ok(())
    }

    /// ArraySetLength for a descriptor without [[Value]].
    pub(crate) fn emit_array_set_length_without_value(
        &mut self,
        array_local: u32,
        writable_payload_local: u32,
        writable_present_local: u32,
        success_local: u32,
        function: &mut Function,
    ) {
        let old_writable_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(success_local));
        self.emit_array_length_writable_i64(array_local, old_writable_local, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(old_writable_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(writable_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(writable_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::BrIf(0));
        function.instruction(&Instruction::LocalGet(writable_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(writable_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_store_length_writable_descriptor(
            array_local,
            writable_payload_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(success_local));
        function.instruction(&Instruction::End);
        self.release_temp_local(old_writable_local);
    }

    pub(crate) fn emit_array_set_length_from_number_payload(
        &mut self,
        array_local: u32,
        length_payload_local: u32,
        writable_payload_local: u32,
        writable_present_local: u32,
        success_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let new_len_local = self.reserve_temp_local();
        let old_len_local = self.reserve_temp_local();
        let cap_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let fill_index_local = self.reserve_temp_local();
        let fill_descriptor_kind_local = self.reserve_temp_local();
        let writable_local = self.reserve_temp_local();
        let requested_len_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(success_local));

        function.instruction(&Instruction::LocalGet(length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncSatF64U);
        function.instruction(&Instruction::LocalSet(new_len_local));
        function.instruction(&Instruction::LocalGet(new_len_local));
        function.instruction(&Instruction::LocalSet(requested_len_local));
        self.load_i64_to_local_from_offset(array_local, HEAP_LEN_OFFSET, old_len_local, function);
        self.load_i64_to_local_from_offset(array_local, HEAP_CAP_OFFSET, cap_local, function);
        self.load_i64_to_local_from_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.emit_array_length_writable_i64(array_local, writable_local, function);

        // A non-writable length only permits an exact no-op.  Assignment and
        // DefineProperty callers turn this ordinary failure into their own
        // strict/throw/false result instead of making ArraySetLength throw.
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(writable_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(new_len_local));
        function.instruction(&Instruction::LocalGet(old_len_local));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(writable_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(writable_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::BrIf(0));

        // Growing `length` alone must not force the dense backing buffer to
        // cover the whole new length: indices beyond `MAX_DENSE_ARRAY_INDEX`
        // are served by the sparse present-indexes path everywhere else
        // (`emit_array_write`, `emit_array_read`), so eagerly densifying up to
        // e.g. `length = 4294967295` would try to allocate a buffer sized for
        // ~4 billion 24-byte entries (~100GB) and trap on OOM. Only grow the
        // dense buffer here when the new length stays within the same dense
        // range the write path is willing to densify to.
        function.instruction(&Instruction::LocalGet(new_len_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(new_len_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(new_len_local));
        function.instruction(&Instruction::LocalGet(old_len_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(new_len_local));
        function.instruction(&Instruction::I64Const(MAX_DENSE_ARRAY_INDEX as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(new_len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(fill_index_local));
        self.emit_array_grow_buffer(
            array_local,
            buffer_local,
            old_len_local,
            cap_local,
            fill_index_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(new_len_local));
        function.instruction(&Instruction::LocalGet(old_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(old_len_local));
        function.instruction(&Instruction::LocalSet(fill_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        // ArraySetLength visits existing own properties in descending order;
        // do not walk every hole between a sparse high index and the target.
        self.emit_array_retreat_to_previous_present_index(
            array_local,
            fill_index_local,
            old_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(fill_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(fill_index_local));
        function.instruction(&Instruction::LocalGet(new_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_descriptor_kind_for_index(
            array_local,
            fill_index_local,
            fill_descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(fill_descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(fill_descriptor_kind_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_DESCRIPTOR_CONFIGURABLE as i64,
        ));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(fill_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(new_len_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(fill_descriptor_kind_local));
        self.emit_store_array_descriptor_for_index(
            array_local,
            fill_index_local,
            fill_descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.store_i64_local_at_offset(array_local, HEAP_LEN_OFFSET, new_len_local, function);
        function.instruction(&Instruction::LocalGet(writable_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(writable_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_store_length_writable_descriptor(
            array_local,
            writable_payload_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(new_len_local));
        function.instruction(&Instruction::LocalGet(requested_len_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(success_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(requested_len_local);
        self.release_temp_local(writable_local);
        self.release_temp_local(fill_descriptor_kind_local);
        self.release_temp_local(fill_index_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(cap_local);
        self.release_temp_local(old_len_local);
        self.release_temp_local(new_len_local);
        Ok(())
    }

    pub(crate) fn emit_known_array_index_from_property_key(
        &mut self,
        key_local: u32,
        index_local: u32,
        found_local: u32,
        function: &mut Function,
    ) {
        let string_offset_local = self.reserve_temp_local();
        let string_len_local = self.reserve_temp_local();
        let byte_index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let digit_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(byte_index_local));

        self.emit_property_key_payload_is_symbol_i32(key_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(found_local));
        self.emit_unpack_string_payload(key_local, string_offset_local, string_len_local, function);

        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_string_byte(string_offset_local, byte_index_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));

        self.emit_load_string_byte(string_offset_local, byte_index_local, byte_local, function);
        self.emit_byte_is_digit_i32(byte_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(digit_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::LocalGet(byte_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(byte_index_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(MAX_ARRAY_LENGTH as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(digit_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(byte_index_local);
        self.release_temp_local(string_len_local);
        self.release_temp_local(string_offset_local);
    }

    pub(crate) fn emit_array_descriptor_flags_to_local(
        &mut self,
        descriptor_base: u64,
        writable_payload_local: Option<u32>,
        enumerable_payload_local: u32,
        configurable_payload_local: u32,
        descriptor_kind_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(descriptor_base as i64));
        function.instruction(&Instruction::LocalSet(descriptor_kind_local));
        if let Some(writable_payload_local) = writable_payload_local {
            function.instruction(&Instruction::LocalGet(writable_payload_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(descriptor_kind_local));
            function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_WRITABLE as i64));
            function.instruction(&Instruction::I64Or);
            function.instruction(&Instruction::LocalSet(descriptor_kind_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(enumerable_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ENUMERABLE as i64));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(descriptor_kind_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(configurable_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_DESCRIPTOR_CONFIGURABLE as i64,
        ));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(descriptor_kind_local));
        function.instruction(&Instruction::End);
    }

    pub(crate) fn emit_store_array_descriptor_at_index(
        &mut self,
        array_local: u32,
        index_local: u32,
        descriptor_kind_local: u32,
        function: &mut Function,
    ) {
        let buffer_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        self.release_temp_local(entry_local);
        self.release_temp_local(buffer_local);
    }

    pub(crate) fn emit_store_array_sparse_present_descriptor_at_index(
        &mut self,
        array_local: u32,
        index_local: u32,
        descriptor_kind_local: u32,
        function: &mut Function,
    ) {
        let list_ptr_local = self.reserve_temp_local();
        let list_len_local = self.reserve_temp_local();
        let list_index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let candidate_index_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_PTR_OFFSET,
            list_ptr_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_LEN_OFFSET,
            list_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(list_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(list_ptr_local));
        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_PRESENT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_INDEX_OFFSET,
            candidate_index_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::LocalSet(list_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(list_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(candidate_index_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(list_index_local);
        self.release_temp_local(list_len_local);
        self.release_temp_local(list_ptr_local);
    }

    pub(crate) fn emit_store_array_descriptor_for_index(
        &mut self,
        array_local: u32,
        index_local: u32,
        descriptor_kind_local: u32,
        function: &mut Function,
    ) {
        let cap_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(array_local, HEAP_CAP_OFFSET, cap_local, function);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_store_array_descriptor_at_index(
            array_local,
            index_local,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_store_array_sparse_present_descriptor_at_index(
            array_local,
            index_local,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::End);
        self.release_temp_local(cap_local);
    }

    fn emit_store_array_accessor_setter_for_index(
        &mut self,
        array_local: u32,
        index_local: u32,
        setter_payload_local: u32,
        setter_tag_local: u32,
        function: &mut Function,
    ) {
        let cap_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let list_ptr_local = self.reserve_temp_local();
        let list_len_local = self.reserve_temp_local();
        let list_index_local = self.reserve_temp_local();
        let candidate_index_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(array_local, HEAP_CAP_OFFSET, cap_local, function);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_ARRAY_SETTER_TAG_OFFSET,
            setter_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_ARRAY_SETTER_PAYLOAD_OFFSET,
            setter_payload_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_PTR_OFFSET,
            list_ptr_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_LEN_OFFSET,
            list_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(list_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(list_ptr_local));
        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_PRESENT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_INDEX_OFFSET,
            candidate_index_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_SETTER_TAG_OFFSET,
            setter_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_SETTER_PAYLOAD_OFFSET,
            setter_payload_local,
            function,
        );
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(list_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(candidate_index_local);
        self.release_temp_local(list_index_local);
        self.release_temp_local(list_len_local);
        self.release_temp_local(list_ptr_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(cap_local);
    }

    pub(crate) fn emit_array_descriptor_side_present_to_local(
        terms: &KindTerms<WasmLocals>,
        result_local: u32,
        function: &mut Function,
    ) {
        if terms.statically_true {
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(result_local));
            return;
        }

        let flags = terms.runtime_flags();
        if flags.is_empty() {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(result_local));
            return;
        }
        for (index, present_local) in flags.iter().enumerate() {
            function.instruction(&Instruction::LocalGet(*present_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            if index > 0 {
                function.instruction(&Instruction::I32Or);
            }
        }
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(result_local));
    }

    pub(crate) fn emit_array_index_effective_flag(
        &mut self,
        field: Presence<u32, u32>,
        existing_descriptor_kind_local: u32,
        flag: DescriptorMask,
        result_local: u32,
        function: &mut Function,
    ) {
        let emit_existing = |function: &mut Function| {
            function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
            function.instruction(&Instruction::I64Const(flag.as_i64()));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::I64ExtendI32U);
        };
        match field {
            Presence::Absent => emit_existing(function),
            Presence::Present(value_local) => {
                function.instruction(&Instruction::LocalGet(value_local));
            }
            Presence::Runtime {
                present: present_local,
                value: value_local,
            } => {
                function.instruction(&Instruction::LocalGet(present_local));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
                emit_existing(function);
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::LocalGet(value_local));
                function.instruction(&Instruction::End);
            }
        }
        function.instruction(&Instruction::LocalSet(result_local));
    }

    pub(crate) fn emit_array_index_effective_value(
        field: Presence<TaggedLocals, u32>,
        existing: TaggedLocals,
        result: TaggedLocals,
        function: &mut Function,
    ) {
        match field {
            Presence::Absent => {
                function.instruction(&Instruction::LocalGet(existing.payload));
                function.instruction(&Instruction::LocalSet(result.payload));
                function.instruction(&Instruction::LocalGet(existing.tag));
                function.instruction(&Instruction::LocalSet(result.tag));
            }
            Presence::Present(value) => {
                function.instruction(&Instruction::LocalGet(value.payload));
                function.instruction(&Instruction::LocalSet(result.payload));
                function.instruction(&Instruction::LocalGet(value.tag));
                function.instruction(&Instruction::LocalSet(result.tag));
            }
            Presence::Runtime { present, value } => {
                function.instruction(&Instruction::LocalGet(present));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(existing.payload));
                function.instruction(&Instruction::LocalSet(result.payload));
                function.instruction(&Instruction::LocalGet(existing.tag));
                function.instruction(&Instruction::LocalSet(result.tag));
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::LocalGet(value.payload));
                function.instruction(&Instruction::LocalSet(result.payload));
                function.instruction(&Instruction::LocalGet(value.tag));
                function.instruction(&Instruction::LocalSet(result.tag));
                function.instruction(&Instruction::End);
            }
        }
    }

    /// Array exotic `[[DefineOwnProperty]]` for an array index.
    ///
    /// The validated descriptor is the only semantic input. Existing storage
    /// is read first, all 10.1.6.3 compatibility checks finish next, and only
    /// then may an element, accessor, descriptor word or array length change.
    pub(crate) fn emit_array_define_index_descriptor(
        &mut self,
        array_local: u32,
        index_local: u32,
        descriptor: WasmDescriptor,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let classification = classify(&descriptor);
        let data_terms = classification.terms(DescriptorSide::Data);
        let accessor_terms = classification.terms(DescriptorSide::Accessor);

        let existing_descriptor_kind_local = self.reserve_temp_local();
        let requested_data_local = self.reserve_temp_local();
        let requested_accessor_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let writable_local = self.reserve_temp_local();
        let enumerable_local = self.reserve_temp_local();
        let configurable_local = self.reserve_temp_local();
        let existing_value =
            TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let existing_setter =
            TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let stored_value = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let stored_setter = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());

        self.emit_array_descriptor_kind_for_index(
            array_local,
            index_local,
            existing_descriptor_kind_local,
            function,
        );
        // This is the raw element/getter carrier, not `[[Get]]`; descriptor
        // validation must never invoke the existing getter.
        self.emit_array_read(
            array_local,
            index_local,
            existing_value.payload,
            existing_value.tag,
            function,
        );
        self.emit_array_accessor_setter_for_index(
            array_local,
            index_local,
            existing_setter.payload,
            existing_setter.tag,
            function,
        );
        Self::emit_array_descriptor_side_present_to_local(
            &data_terms,
            requested_data_local,
            function,
        );
        Self::emit_array_descriptor_side_present_to_local(
            &accessor_terms,
            requested_accessor_local,
            function,
        );

        // Reuse the canonical stored-descriptor validator. The Array layout
        // projects its shared data/getter carrier into both typed roles; the
        // stored kind selects which role is observable.
        let validation_success_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(validation_success_local));
        function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_validate_stored_descriptor(
            existing_descriptor_kind_local,
            StoredDescriptorLocals::new(
                StoredDescriptorDataLocals::new(existing_value),
                StoredDescriptorGetterLocals::new(existing_value),
                StoredDescriptorSetterLocals::new(existing_setter),
            ),
            &descriptor,
            validation_success_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(validation_success_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Cannot redefine array index property",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.release_temp_local(validation_success_local);
        let descriptor = descriptor.into_partial();
        self.emit_array_index_effective_flag(
            descriptor.enumerable,
            existing_descriptor_kind_local,
            DescriptorMask::ENUMERABLE,
            enumerable_local,
            function,
        );
        self.emit_array_index_effective_flag(
            descriptor.configurable,
            existing_descriptor_kind_local,
            DescriptorMask::CONFIGURABLE,
            configurable_local,
            function,
        );

        // Apply only after the validation block above has closed.
        function.instruction(&Instruction::LocalGet(requested_accessor_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
        function.instruction(&Instruction::I64Const(DescriptorMask::ACCESSOR.as_i64()));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(existing_value.payload));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(existing_value.tag));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(existing_setter.payload));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(existing_setter.tag));
        function.instruction(&Instruction::End);
        Self::emit_array_index_effective_value(
            descriptor.get,
            existing_value,
            stored_value,
            function,
        );
        Self::emit_array_index_effective_value(
            descriptor.set,
            existing_setter,
            stored_setter,
            function,
        );
        self.emit_array_descriptor_flags_to_local(
            ARRAY_DESCRIPTOR_OWN_PROPERTY | OBJECT_DESCRIPTOR_ACCESSOR,
            None,
            enumerable_local,
            configurable_local,
            descriptor_kind_local,
            function,
        );
        self.emit_array_write(
            array_local,
            index_local,
            stored_value.payload,
            stored_value.tag,
            function,
        )?;
        self.emit_store_array_accessor_setter_for_index(
            array_local,
            index_local,
            stored_setter.payload,
            stored_setter.tag,
            function,
        );
        self.emit_store_array_descriptor_for_index(
            array_local,
            index_local,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::Else);

        // A runtime-generic descriptor preserves an existing accessor without
        // touching either accessor carrier. A missing property completes to a
        // data property, as does an actual data descriptor.
        function.instruction(&Instruction::LocalGet(requested_data_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
        function.instruction(&Instruction::I64Const(DescriptorMask::ACCESSOR.as_i64()));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_descriptor_flags_to_local(
            ARRAY_DESCRIPTOR_OWN_PROPERTY | OBJECT_DESCRIPTOR_ACCESSOR,
            None,
            enumerable_local,
            configurable_local,
            descriptor_kind_local,
            function,
        );
        self.emit_store_array_descriptor_for_index(
            array_local,
            index_local,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
        function.instruction(&Instruction::I64Const(DescriptorMask::ACCESSOR.as_i64()));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(existing_value.payload));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(existing_value.tag));
        function.instruction(&Instruction::End);
        Self::emit_array_index_effective_value(
            descriptor.value,
            existing_value,
            stored_value,
            function,
        );
        self.emit_array_index_effective_flag(
            descriptor.writable,
            existing_descriptor_kind_local,
            DescriptorMask::WRITABLE,
            writable_local,
            function,
        );
        self.emit_array_descriptor_flags_to_local(
            ARRAY_DESCRIPTOR_OWN_PROPERTY | OBJECT_DESCRIPTOR_DATA,
            Some(writable_local),
            enumerable_local,
            configurable_local,
            descriptor_kind_local,
            function,
        );
        self.emit_array_write(
            array_local,
            index_local,
            stored_value.payload,
            stored_value.tag,
            function,
        )?;
        self.emit_store_array_descriptor_for_index(
            array_local,
            index_local,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        for local in [
            stored_setter.tag,
            stored_setter.payload,
            stored_value.tag,
            stored_value.payload,
            existing_setter.tag,
            existing_setter.payload,
            existing_value.tag,
            existing_value.payload,
            configurable_local,
            enumerable_local,
            writable_local,
            descriptor_kind_local,
            requested_accessor_local,
            requested_data_local,
            existing_descriptor_kind_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn emit_array_named_props_grow_buffer(
        &mut self,
        array_local: u32,
        buffer_local: u32,
        len_local: u32,
        cap_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let new_cap_local = self.reserve_temp_local();
        let size_local = self.reserve_temp_local();
        let new_buffer_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let old_entry_local = self.reserve_temp_local();
        let new_entry_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(MIN_HEAP_CAPACITY as i64));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(new_cap_local));

        function.instruction(&Instruction::LocalGet(new_cap_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(size_local));
        self.emit_heap_alloc_from_local(size_local, function)?;
        function.instruction(&Instruction::LocalSet(new_buffer_local));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(old_entry_local));

        function.instruction(&Instruction::LocalGet(new_buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(new_entry_local));

        for offset in [
            HEAP_OBJECT_KEY_OFFSET,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            HEAP_OBJECT_DATA_TAG_OFFSET,
            HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
            HEAP_OBJECT_GETTER_TAG_OFFSET,
            HEAP_OBJECT_GETTER_PAYLOAD_OFFSET,
            HEAP_OBJECT_SETTER_TAG_OFFSET,
            HEAP_OBJECT_SETTER_PAYLOAD_OFFSET,
        ] {
            self.load_i64_from_offset(old_entry_local, offset, function);
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.store_i64_local_at_offset(new_entry_local, offset, self.scratch_local, function);
        }

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(new_buffer_local));
        function.instruction(&Instruction::LocalSet(buffer_local));
        function.instruction(&Instruction::LocalGet(new_cap_local));
        function.instruction(&Instruction::LocalSet(cap_local));
        self.store_i64_local_at_offset(
            array_local,
            HEAP_ARRAY_NAMED_PROPS_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.store_i64_local_at_offset(
            array_local,
            HEAP_ARRAY_NAMED_PROPS_CAP_OFFSET,
            cap_local,
            function,
        );

        self.release_temp_local(new_entry_local);
        self.release_temp_local(old_entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(new_buffer_local);
        self.release_temp_local(size_local);
        self.release_temp_local(new_cap_local);
        Ok(())
    }

    pub(crate) fn emit_descriptor_flag_payload_from_new_descriptor(
        &mut self,
        requested_payload_local: u32,
        present_local: Option<u32>,
        flag_payload_local: u32,
        function: &mut Function,
    ) {
        if let Some(present_local) = present_local {
            function.instruction(&Instruction::LocalGet(present_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(flag_payload_local));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(requested_payload_local));
            function.instruction(&Instruction::LocalSet(flag_payload_local));
            function.instruction(&Instruction::End);
        } else {
            function.instruction(&Instruction::LocalGet(requested_payload_local));
            function.instruction(&Instruction::LocalSet(flag_payload_local));
        }
    }

    pub(crate) fn emit_descriptor_flag_payload_from_existing_descriptor(
        &mut self,
        existing_descriptor_kind_local: u32,
        requested_payload_local: u32,
        present_local: Option<u32>,
        flag: DescriptorMask,
        flag_payload_local: u32,
        function: &mut Function,
    ) {
        if let Some(present_local) = present_local {
            function.instruction(&Instruction::LocalGet(present_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
            function.instruction(&Instruction::I64Const(flag.as_i64()));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::LocalSet(flag_payload_local));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(requested_payload_local));
            function.instruction(&Instruction::LocalSet(flag_payload_local));
            function.instruction(&Instruction::End);
        } else {
            function.instruction(&Instruction::LocalGet(requested_payload_local));
            function.instruction(&Instruction::LocalSet(flag_payload_local));
        }
    }

    pub(crate) fn emit_array_define_named_data_descriptor(
        &mut self,
        array_local: u32,
        key_local: u32,
        payload_local: u32,
        tag_local: u32,
        writable_payload_local: u32,
        enumerable_payload_local: u32,
        configurable_payload_local: u32,
        value_present_local: Option<u32>,
        writable_present_local: Option<u32>,
        enumerable_present_local: Option<u32>,
        configurable_present_local: Option<u32>,
        validation_success_local: Option<u32>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let cap_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let existing_descriptor_kind_local = self.reserve_temp_local();
        let stored_data_tag_local = self.reserve_temp_local();
        let stored_data_payload_local = self.reserve_temp_local();
        let writable_flag_payload_local = self.reserve_temp_local();
        let enumerable_flag_payload_local = self.reserve_temp_local();
        let configurable_flag_payload_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_NAMED_PROPS_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_NAMED_PROPS_LEN_OFFSET,
            len_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_NAMED_PROPS_CAP_OFFSET,
            cap_local,
            function,
        );

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            self.scratch_local,
            function,
        );
        self.emit_property_key_payload_equality_i32(self.scratch_local, key_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            existing_descriptor_kind_local,
            function,
        );
        if let Some(validation_success_local) = validation_success_local {
            let descriptor = WasmPartialDescriptor {
                value: array_descriptor_field(
                    TaggedLocals::new(payload_local, tag_local),
                    value_present_local,
                ),
                writable: array_descriptor_field(writable_payload_local, writable_present_local),
                get: Presence::Absent,
                set: Presence::Absent,
                enumerable: array_descriptor_field(
                    enumerable_payload_local,
                    enumerable_present_local,
                ),
                configurable: array_descriptor_field(
                    configurable_payload_local,
                    configurable_present_local,
                ),
            }
            .validate()
            .expect("an array data descriptor cannot fail 6.2.6.5 step 9");
            self.emit_validate_array_named_descriptor(
                entry_local,
                existing_descriptor_kind_local,
                &descriptor,
                validation_success_local,
                function,
            )?;
            function.instruction(&Instruction::LocalGet(validation_success_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Br(3));
            function.instruction(&Instruction::End);
        }
        if let (Some(value_present_local), Some(writable_present_local)) =
            (value_present_local, writable_present_local)
        {
            function.instruction(&Instruction::LocalGet(value_present_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::LocalGet(writable_present_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_descriptor_flag_payload_from_existing_descriptor(
                existing_descriptor_kind_local,
                writable_payload_local,
                Some(writable_present_local),
                DescriptorMask::WRITABLE,
                writable_flag_payload_local,
                function,
            );
            self.emit_descriptor_flag_payload_from_existing_descriptor(
                existing_descriptor_kind_local,
                enumerable_payload_local,
                enumerable_present_local,
                DescriptorMask::ENUMERABLE,
                enumerable_flag_payload_local,
                function,
            );
            self.emit_descriptor_flag_payload_from_existing_descriptor(
                existing_descriptor_kind_local,
                configurable_payload_local,
                configurable_present_local,
                DescriptorMask::CONFIGURABLE,
                configurable_flag_payload_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
            function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_array_descriptor_flags_to_local(
                ARRAY_DESCRIPTOR_OWN_PROPERTY | OBJECT_DESCRIPTOR_DATA,
                Some(writable_flag_payload_local),
                enumerable_flag_payload_local,
                configurable_flag_payload_local,
                descriptor_kind_local,
                function,
            );
            function.instruction(&Instruction::Else);
            self.emit_array_descriptor_flags_to_local(
                ARRAY_DESCRIPTOR_OWN_PROPERTY | OBJECT_DESCRIPTOR_ACCESSOR,
                None,
                enumerable_flag_payload_local,
                configurable_flag_payload_local,
                descriptor_kind_local,
                function,
            );
            function.instruction(&Instruction::End);
            self.store_i64_local_at_offset(
                entry_local,
                HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
                descriptor_kind_local,
                function,
            );
            function.instruction(&Instruction::Br(4));
            function.instruction(&Instruction::End);
        }
        if let Some(value_present_local) = value_present_local {
            function.instruction(&Instruction::LocalGet(value_present_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
            function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.load_i64_to_local_from_offset(
                entry_local,
                HEAP_OBJECT_DATA_TAG_OFFSET,
                stored_data_tag_local,
                function,
            );
            self.load_i64_to_local_from_offset(
                entry_local,
                HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
                stored_data_payload_local,
                function,
            );
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(stored_data_tag_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(stored_data_payload_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(tag_local));
            function.instruction(&Instruction::LocalSet(stored_data_tag_local));
            function.instruction(&Instruction::LocalGet(payload_local));
            function.instruction(&Instruction::LocalSet(stored_data_payload_local));
            function.instruction(&Instruction::End);
        } else {
            function.instruction(&Instruction::LocalGet(tag_local));
            function.instruction(&Instruction::LocalSet(stored_data_tag_local));
            function.instruction(&Instruction::LocalGet(payload_local));
            function.instruction(&Instruction::LocalSet(stored_data_payload_local));
        }
        self.emit_descriptor_flag_payload_from_existing_descriptor(
            existing_descriptor_kind_local,
            writable_payload_local,
            writable_present_local,
            DescriptorMask::WRITABLE,
            writable_flag_payload_local,
            function,
        );
        self.emit_descriptor_flag_payload_from_existing_descriptor(
            existing_descriptor_kind_local,
            enumerable_payload_local,
            enumerable_present_local,
            DescriptorMask::ENUMERABLE,
            enumerable_flag_payload_local,
            function,
        );
        self.emit_descriptor_flag_payload_from_existing_descriptor(
            existing_descriptor_kind_local,
            configurable_payload_local,
            configurable_present_local,
            DescriptorMask::CONFIGURABLE,
            configurable_flag_payload_local,
            function,
        );
        self.emit_array_descriptor_flags_to_local(
            ARRAY_DESCRIPTOR_OWN_PROPERTY | OBJECT_DESCRIPTOR_DATA,
            Some(writable_flag_payload_local),
            enumerable_flag_payload_local,
            configurable_flag_payload_local,
            descriptor_kind_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_DATA_TAG_OFFSET,
            stored_data_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
            stored_data_payload_local,
            function,
        );
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_GETTER_TAG_OFFSET, 0, function);
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_GETTER_PAYLOAD_OFFSET, 0, function);
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_SETTER_TAG_OFFSET, 0, function);
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_SETTER_PAYLOAD_OFFSET, 0, function);
        function.instruction(&Instruction::Br(3));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        if let Some(value_present_local) = value_present_local {
            function.instruction(&Instruction::LocalGet(value_present_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(stored_data_tag_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(stored_data_payload_local));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(tag_local));
            function.instruction(&Instruction::LocalSet(stored_data_tag_local));
            function.instruction(&Instruction::LocalGet(payload_local));
            function.instruction(&Instruction::LocalSet(stored_data_payload_local));
            function.instruction(&Instruction::End);
        } else {
            function.instruction(&Instruction::LocalGet(tag_local));
            function.instruction(&Instruction::LocalSet(stored_data_tag_local));
            function.instruction(&Instruction::LocalGet(payload_local));
            function.instruction(&Instruction::LocalSet(stored_data_payload_local));
        }
        self.emit_descriptor_flag_payload_from_new_descriptor(
            writable_payload_local,
            writable_present_local,
            writable_flag_payload_local,
            function,
        );
        self.emit_descriptor_flag_payload_from_new_descriptor(
            enumerable_payload_local,
            enumerable_present_local,
            enumerable_flag_payload_local,
            function,
        );
        self.emit_descriptor_flag_payload_from_new_descriptor(
            configurable_payload_local,
            configurable_present_local,
            configurable_flag_payload_local,
            function,
        );
        self.emit_array_descriptor_flags_to_local(
            ARRAY_DESCRIPTOR_OWN_PROPERTY | OBJECT_DESCRIPTOR_DATA,
            Some(writable_flag_payload_local),
            enumerable_flag_payload_local,
            configurable_flag_payload_local,
            descriptor_kind_local,
            function,
        );
        if let Some(validation_success_local) = validation_success_local {
            self.load_i64_to_local_from_offset(
                array_local,
                HEAP_ARRAY_NON_EXTENSIBLE_OFFSET,
                self.scratch_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(self.scratch_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(validation_success_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalGet(validation_success_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Br(1));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_named_props_grow_buffer(
            array_local,
            buffer_local,
            len_local,
            cap_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_local_at_offset(entry_local, HEAP_OBJECT_KEY_OFFSET, key_local, function);
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_DATA_TAG_OFFSET,
            stored_data_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
            stored_data_payload_local,
            function,
        );
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_GETTER_TAG_OFFSET, 0, function);
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_GETTER_PAYLOAD_OFFSET, 0, function);
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_SETTER_TAG_OFFSET, 0, function);
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_SETTER_PAYLOAD_OFFSET, 0, function);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(len_local));
        self.store_i64_local_at_offset(
            array_local,
            HEAP_ARRAY_NAMED_PROPS_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::End);

        self.release_temp_local(configurable_flag_payload_local);
        self.release_temp_local(enumerable_flag_payload_local);
        self.release_temp_local(writable_flag_payload_local);
        self.release_temp_local(stored_data_payload_local);
        self.release_temp_local(stored_data_tag_local);
        self.release_temp_local(existing_descriptor_kind_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(cap_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    pub(crate) fn emit_array_define_named_accessor_descriptor(
        &mut self,
        array_local: u32,
        key_local: u32,
        getter_payload_local: u32,
        getter_tag_local: u32,
        setter_payload_local: u32,
        setter_tag_local: u32,
        enumerable_payload_local: u32,
        configurable_payload_local: u32,
        getter_present_local: Option<u32>,
        setter_present_local: Option<u32>,
        enumerable_present_local: Option<u32>,
        configurable_present_local: Option<u32>,
        validation_success_local: Option<u32>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let cap_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let existing_descriptor_kind_local = self.reserve_temp_local();
        let stored_getter_tag_local = self.reserve_temp_local();
        let stored_getter_payload_local = self.reserve_temp_local();
        let stored_setter_tag_local = self.reserve_temp_local();
        let stored_setter_payload_local = self.reserve_temp_local();
        let enumerable_flag_payload_local = self.reserve_temp_local();
        let configurable_flag_payload_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_NAMED_PROPS_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_NAMED_PROPS_LEN_OFFSET,
            len_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_NAMED_PROPS_CAP_OFFSET,
            cap_local,
            function,
        );

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            self.scratch_local,
            function,
        );
        self.emit_property_key_payload_equality_i32(self.scratch_local, key_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            existing_descriptor_kind_local,
            function,
        );
        if let Some(validation_success_local) = validation_success_local {
            let descriptor = WasmPartialDescriptor {
                value: Presence::Absent,
                writable: Presence::Absent,
                get: array_descriptor_field(
                    TaggedLocals::new(getter_payload_local, getter_tag_local),
                    getter_present_local,
                ),
                set: array_descriptor_field(
                    TaggedLocals::new(setter_payload_local, setter_tag_local),
                    setter_present_local,
                ),
                enumerable: array_descriptor_field(
                    enumerable_payload_local,
                    enumerable_present_local,
                ),
                configurable: array_descriptor_field(
                    configurable_payload_local,
                    configurable_present_local,
                ),
            }
            .validate()
            .expect("an array accessor descriptor cannot fail 6.2.6.5 step 9");
            self.emit_validate_array_named_descriptor(
                entry_local,
                existing_descriptor_kind_local,
                &descriptor,
                validation_success_local,
                function,
            )?;
            function.instruction(&Instruction::LocalGet(validation_success_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Br(3));
            function.instruction(&Instruction::End);
        }
        if let (Some(getter_present_local), Some(setter_present_local)) =
            (getter_present_local, setter_present_local)
        {
            function.instruction(&Instruction::LocalGet(getter_present_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::LocalGet(setter_present_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_descriptor_flag_payload_from_existing_descriptor(
                existing_descriptor_kind_local,
                enumerable_payload_local,
                enumerable_present_local,
                DescriptorMask::ENUMERABLE,
                enumerable_flag_payload_local,
                function,
            );
            self.emit_descriptor_flag_payload_from_existing_descriptor(
                existing_descriptor_kind_local,
                configurable_payload_local,
                configurable_present_local,
                DescriptorMask::CONFIGURABLE,
                configurable_flag_payload_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
            function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            let writable_flag_payload_local = self.reserve_temp_local();
            self.emit_descriptor_flag_payload_from_existing_descriptor(
                existing_descriptor_kind_local,
                0,
                None,
                DescriptorMask::WRITABLE,
                writable_flag_payload_local,
                function,
            );
            self.emit_array_descriptor_flags_to_local(
                ARRAY_DESCRIPTOR_OWN_PROPERTY | OBJECT_DESCRIPTOR_DATA,
                Some(writable_flag_payload_local),
                enumerable_flag_payload_local,
                configurable_flag_payload_local,
                descriptor_kind_local,
                function,
            );
            self.release_temp_local(writable_flag_payload_local);
            function.instruction(&Instruction::Else);
            self.emit_array_descriptor_flags_to_local(
                ARRAY_DESCRIPTOR_OWN_PROPERTY | OBJECT_DESCRIPTOR_ACCESSOR,
                None,
                enumerable_flag_payload_local,
                configurable_flag_payload_local,
                descriptor_kind_local,
                function,
            );
            function.instruction(&Instruction::End);
            self.store_i64_local_at_offset(
                entry_local,
                HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
                descriptor_kind_local,
                function,
            );
            function.instruction(&Instruction::Br(4));
            function.instruction(&Instruction::End);
        }
        if let Some(getter_present_local) = getter_present_local {
            function.instruction(&Instruction::LocalGet(getter_present_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
            function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.load_i64_to_local_from_offset(
                entry_local,
                HEAP_OBJECT_GETTER_TAG_OFFSET,
                stored_getter_tag_local,
                function,
            );
            self.load_i64_to_local_from_offset(
                entry_local,
                HEAP_OBJECT_GETTER_PAYLOAD_OFFSET,
                stored_getter_payload_local,
                function,
            );
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(stored_getter_tag_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(stored_getter_payload_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(getter_tag_local));
            function.instruction(&Instruction::LocalSet(stored_getter_tag_local));
            function.instruction(&Instruction::LocalGet(getter_payload_local));
            function.instruction(&Instruction::LocalSet(stored_getter_payload_local));
            function.instruction(&Instruction::End);
        } else {
            function.instruction(&Instruction::LocalGet(getter_tag_local));
            function.instruction(&Instruction::LocalSet(stored_getter_tag_local));
            function.instruction(&Instruction::LocalGet(getter_payload_local));
            function.instruction(&Instruction::LocalSet(stored_getter_payload_local));
        }
        if let Some(setter_present_local) = setter_present_local {
            function.instruction(&Instruction::LocalGet(setter_present_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
            function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.load_i64_to_local_from_offset(
                entry_local,
                HEAP_OBJECT_SETTER_TAG_OFFSET,
                stored_setter_tag_local,
                function,
            );
            self.load_i64_to_local_from_offset(
                entry_local,
                HEAP_OBJECT_SETTER_PAYLOAD_OFFSET,
                stored_setter_payload_local,
                function,
            );
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(stored_setter_tag_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(stored_setter_payload_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(setter_tag_local));
            function.instruction(&Instruction::LocalSet(stored_setter_tag_local));
            function.instruction(&Instruction::LocalGet(setter_payload_local));
            function.instruction(&Instruction::LocalSet(stored_setter_payload_local));
            function.instruction(&Instruction::End);
        } else {
            function.instruction(&Instruction::LocalGet(setter_tag_local));
            function.instruction(&Instruction::LocalSet(stored_setter_tag_local));
            function.instruction(&Instruction::LocalGet(setter_payload_local));
            function.instruction(&Instruction::LocalSet(stored_setter_payload_local));
        }
        self.emit_descriptor_flag_payload_from_existing_descriptor(
            existing_descriptor_kind_local,
            enumerable_payload_local,
            enumerable_present_local,
            DescriptorMask::ENUMERABLE,
            enumerable_flag_payload_local,
            function,
        );
        self.emit_descriptor_flag_payload_from_existing_descriptor(
            existing_descriptor_kind_local,
            configurable_payload_local,
            configurable_present_local,
            DescriptorMask::CONFIGURABLE,
            configurable_flag_payload_local,
            function,
        );
        self.emit_array_descriptor_flags_to_local(
            ARRAY_DESCRIPTOR_OWN_PROPERTY | OBJECT_DESCRIPTOR_ACCESSOR,
            None,
            enumerable_flag_payload_local,
            configurable_flag_payload_local,
            descriptor_kind_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        self.store_i64_const_at_offset(
            entry_local,
            HEAP_OBJECT_DATA_TAG_OFFSET,
            ValueKind::Undefined.tag() as u64,
            function,
        );
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_DATA_PAYLOAD_OFFSET, 0, function);
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_GETTER_TAG_OFFSET,
            stored_getter_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_GETTER_PAYLOAD_OFFSET,
            stored_getter_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_SETTER_TAG_OFFSET,
            stored_setter_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_SETTER_PAYLOAD_OFFSET,
            stored_setter_payload_local,
            function,
        );
        function.instruction(&Instruction::Br(3));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        if let Some(getter_present_local) = getter_present_local {
            function.instruction(&Instruction::LocalGet(getter_present_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(stored_getter_tag_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(stored_getter_payload_local));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(getter_tag_local));
            function.instruction(&Instruction::LocalSet(stored_getter_tag_local));
            function.instruction(&Instruction::LocalGet(getter_payload_local));
            function.instruction(&Instruction::LocalSet(stored_getter_payload_local));
            function.instruction(&Instruction::End);
        } else {
            function.instruction(&Instruction::LocalGet(getter_tag_local));
            function.instruction(&Instruction::LocalSet(stored_getter_tag_local));
            function.instruction(&Instruction::LocalGet(getter_payload_local));
            function.instruction(&Instruction::LocalSet(stored_getter_payload_local));
        }
        if let Some(setter_present_local) = setter_present_local {
            function.instruction(&Instruction::LocalGet(setter_present_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(stored_setter_tag_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(stored_setter_payload_local));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(setter_tag_local));
            function.instruction(&Instruction::LocalSet(stored_setter_tag_local));
            function.instruction(&Instruction::LocalGet(setter_payload_local));
            function.instruction(&Instruction::LocalSet(stored_setter_payload_local));
            function.instruction(&Instruction::End);
        } else {
            function.instruction(&Instruction::LocalGet(setter_tag_local));
            function.instruction(&Instruction::LocalSet(stored_setter_tag_local));
            function.instruction(&Instruction::LocalGet(setter_payload_local));
            function.instruction(&Instruction::LocalSet(stored_setter_payload_local));
        }
        self.emit_descriptor_flag_payload_from_new_descriptor(
            enumerable_payload_local,
            enumerable_present_local,
            enumerable_flag_payload_local,
            function,
        );
        self.emit_descriptor_flag_payload_from_new_descriptor(
            configurable_payload_local,
            configurable_present_local,
            configurable_flag_payload_local,
            function,
        );
        self.emit_array_descriptor_flags_to_local(
            ARRAY_DESCRIPTOR_OWN_PROPERTY | OBJECT_DESCRIPTOR_ACCESSOR,
            None,
            enumerable_flag_payload_local,
            configurable_flag_payload_local,
            descriptor_kind_local,
            function,
        );
        if let Some(validation_success_local) = validation_success_local {
            self.load_i64_to_local_from_offset(
                array_local,
                HEAP_ARRAY_NON_EXTENSIBLE_OFFSET,
                self.scratch_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(self.scratch_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(validation_success_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalGet(validation_success_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Br(1));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_named_props_grow_buffer(
            array_local,
            buffer_local,
            len_local,
            cap_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_local_at_offset(entry_local, HEAP_OBJECT_KEY_OFFSET, key_local, function);
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        self.store_i64_const_at_offset(
            entry_local,
            HEAP_OBJECT_DATA_TAG_OFFSET,
            ValueKind::Undefined.tag() as u64,
            function,
        );
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_DATA_PAYLOAD_OFFSET, 0, function);
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_GETTER_TAG_OFFSET,
            stored_getter_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_GETTER_PAYLOAD_OFFSET,
            stored_getter_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_SETTER_TAG_OFFSET,
            stored_setter_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_SETTER_PAYLOAD_OFFSET,
            stored_setter_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(len_local));
        self.store_i64_local_at_offset(
            array_local,
            HEAP_ARRAY_NAMED_PROPS_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::End);

        self.release_temp_local(configurable_flag_payload_local);
        self.release_temp_local(enumerable_flag_payload_local);
        self.release_temp_local(stored_setter_payload_local);
        self.release_temp_local(stored_setter_tag_local);
        self.release_temp_local(stored_getter_payload_local);
        self.release_temp_local(stored_getter_tag_local);
        self.release_temp_local(existing_descriptor_kind_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(cap_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    pub(crate) fn emit_array_define_named_data_property(
        &mut self,
        array_local: u32,
        key_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let writable_payload_local = self.reserve_temp_local();
        let enumerable_payload_local = self.reserve_temp_local();
        let configurable_payload_local = self.reserve_temp_local();
        let success_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(writable_payload_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(enumerable_payload_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(configurable_payload_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(success_local));
        self.emit_array_define_named_data_descriptor(
            array_local,
            key_local,
            payload_local,
            tag_local,
            writable_payload_local,
            enumerable_payload_local,
            configurable_payload_local,
            None,
            None,
            None,
            None,
            Some(success_local),
            function,
        )?;
        function.instruction(&Instruction::LocalGet(success_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_write_set_failure_else(
            "Cannot add property to non-extensible array",
            function,
        )?;
        function.instruction(&Instruction::End);
        self.release_temp_local(success_local);
        self.release_temp_local(configurable_payload_local);
        self.release_temp_local(enumerable_payload_local);
        self.release_temp_local(writable_payload_local);
        Ok(())
    }

    pub(crate) fn emit_array_define_builtin_named_data_property(
        &mut self,
        array_local: u32,
        descriptor_offset: u64,
        tag_offset: u64,
        payload_offset: u64,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        self.store_i64_const_at_offset(
            array_local,
            descriptor_offset,
            (ARRAY_DESCRIPTOR_OWN_PROPERTY | OBJECT_DESCRIPTOR_DATA) as u64,
            function,
        );
        self.store_i64_local_at_offset(array_local, tag_offset, tag_local, function);
        self.store_i64_local_at_offset(array_local, payload_offset, payload_local, function);
    }

    pub(crate) fn emit_array_read_builtin_named_data_property(
        &mut self,
        array_local: u32,
        descriptor_offset: u64,
        tag_offset: u64,
        payload_offset: u64,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        let descriptor_kind_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            array_local,
            descriptor_offset,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(array_local, payload_offset, payload_local, function);
        self.load_i64_to_local_from_offset(array_local, tag_offset, tag_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::End);
        self.release_temp_local(descriptor_kind_local);
    }

    pub(crate) fn emit_array_named_prop_read(
        &mut self,
        array_local: u32,
        key_local: u32,
        payload_local: u32,
        tag_local: u32,
        found_output_local: Option<u32>,
        function: &mut Function,
    ) {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let stored_key_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        if let Some(found_output_local) = found_output_local {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(found_output_local));
        }
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_NAMED_PROPS_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_NAMED_PROPS_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            stored_key_local,
            function,
        );
        self.emit_property_key_payload_equality_i32(stored_key_local, key_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        if let Some(found_output_local) = found_output_local {
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(found_output_local));
        }
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DATA_TAG_OFFSET,
            tag_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(stored_key_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
    }

    pub(crate) fn emit_array_named_prop_descriptor_read(
        &mut self,
        array_local: u32,
        key_local: u32,
        result_payload_local: u32,
        result_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let entry_key_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let writable_payload_local = self.reserve_temp_local();
        let enumerable_payload_local = self.reserve_temp_local();
        let configurable_payload_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let getter_payload_local = self.reserve_temp_local();
        let getter_tag_local = self.reserve_temp_local();
        let setter_payload_local = self.reserve_temp_local();
        let setter_tag_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_NAMED_PROPS_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_NAMED_PROPS_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            entry_key_local,
            function,
        );
        self.emit_property_key_payload_equality_i32(entry_key_local, key_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        for (flag, payload_local) in [
            (OBJECT_DESCRIPTOR_WRITABLE, writable_payload_local),
            (OBJECT_DESCRIPTOR_ENUMERABLE, enumerable_payload_local),
            (OBJECT_DESCRIPTOR_CONFIGURABLE, configurable_payload_local),
        ] {
            function.instruction(&Instruction::LocalGet(descriptor_kind_local));
            function.instruction(&Instruction::I64Const(flag as i64));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::LocalSet(payload_local));
        }
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DATA_TAG_OFFSET,
            value_tag_local,
            function,
        );
        self.emit_alloc_data_descriptor_from_locals_with_flag_locals(
            value_payload_local,
            value_tag_local,
            writable_payload_local,
            enumerable_payload_local,
            configurable_payload_local,
            result_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(result_tag_local));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_GETTER_PAYLOAD_OFFSET,
            getter_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_GETTER_TAG_OFFSET,
            getter_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_SETTER_PAYLOAD_OFFSET,
            setter_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_SETTER_TAG_OFFSET,
            setter_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(getter_tag_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(getter_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(setter_tag_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(setter_tag_local));
        function.instruction(&Instruction::End);
        self.emit_alloc_accessor_descriptor_from_locals_with_flag_local(
            getter_payload_local,
            getter_tag_local,
            setter_payload_local,
            setter_tag_local,
            enumerable_payload_local,
            configurable_payload_local,
            result_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(result_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(setter_tag_local);
        self.release_temp_local(setter_payload_local);
        self.release_temp_local(getter_tag_local);
        self.release_temp_local(getter_payload_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(configurable_payload_local);
        self.release_temp_local(enumerable_payload_local);
        self.release_temp_local(writable_payload_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_key_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    fn emit_array_named_string_props_count(
        &mut self,
        array_local: u32,
        count_local: u32,
        selection: ArrayNamedStringKeySelection,
        function: &mut Function,
    ) {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_NAMED_PROPS_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_NAMED_PROPS_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            self.scratch_local,
            function,
        );
        self.emit_property_key_payload_is_symbol_i32(self.scratch_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        match &selection {
            ArrayNamedStringKeySelection::All => {}
            ArrayNamedStringKeySelection::EnumerableOnly => {
                function.instruction(&Instruction::LocalGet(descriptor_kind_local));
                function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ENUMERABLE as i64));
                function.instruction(&Instruction::I64And);
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::If(BlockType::Empty));
            }
        }
        function.instruction(&Instruction::LocalGet(count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(count_local));
        match &selection {
            ArrayNamedStringKeySelection::All => {}
            ArrayNamedStringKeySelection::EnumerableOnly => {
                function.instruction(&Instruction::End);
            }
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
    }

    fn emit_array_named_string_props_write_keys(
        &mut self,
        array_local: u32,
        result_payload_local: u32,
        write_index_local: u32,
        selection: ArrayNamedStringKeySelection,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_NAMED_PROPS_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_NAMED_PROPS_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            key_payload_local,
            function,
        );
        self.emit_property_key_payload_is_symbol_i32(key_payload_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        match &selection {
            ArrayNamedStringKeySelection::All => {}
            ArrayNamedStringKeySelection::EnumerableOnly => {
                function.instruction(&Instruction::LocalGet(descriptor_kind_local));
                function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ENUMERABLE as i64));
                function.instruction(&Instruction::I64And);
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::If(BlockType::Empty));
            }
        }
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        self.emit_array_write(
            result_payload_local,
            write_index_local,
            key_payload_local,
            key_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        match &selection {
            ArrayNamedStringKeySelection::All => {}
            ArrayNamedStringKeySelection::EnumerableOnly => {
                function.instruction(&Instruction::End);
            }
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    pub(super) fn emit_array_all_named_string_props_count(
        &mut self,
        array_local: u32,
        count_local: u32,
        function: &mut Function,
    ) {
        self.emit_array_named_string_props_count(
            array_local,
            count_local,
            ArrayNamedStringKeySelection::All,
            function,
        );
    }

    pub(super) fn emit_array_enumerable_named_string_props_count(
        &mut self,
        array_local: u32,
        count_local: u32,
        function: &mut Function,
    ) {
        self.emit_array_named_string_props_count(
            array_local,
            count_local,
            ArrayNamedStringKeySelection::EnumerableOnly,
            function,
        );
    }

    pub(super) fn emit_array_all_named_string_props_write_keys(
        &mut self,
        array_local: u32,
        result_payload_local: u32,
        write_index_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_array_named_string_props_write_keys(
            array_local,
            result_payload_local,
            write_index_local,
            ArrayNamedStringKeySelection::All,
            function,
        )
    }

    pub(super) fn emit_array_enumerable_named_string_props_write_keys(
        &mut self,
        array_local: u32,
        result_payload_local: u32,
        write_index_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_array_named_string_props_write_keys(
            array_local,
            result_payload_local,
            write_index_local,
            ArrayNamedStringKeySelection::EnumerableOnly,
            function,
        )
    }

    pub(crate) fn emit_array_delete_property_key(
        &mut self,
        array_local: u32,
        key_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        let index_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Else);
        self.emit_string_index_0_to_4_or_minus_one(key_local, index_local, function);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_delete(array_local, index_local, result_local, function);
        function.instruction(&Instruction::Else);
        self.emit_array_named_prop_delete(array_local, key_local, result_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(index_local);
    }

    pub(crate) fn emit_array_named_prop_delete(
        &mut self,
        array_local: u32,
        key_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let shift_index_local = self.reserve_temp_local();
        let current_entry_local = self.reserve_temp_local();
        let next_entry_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));

        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_NAMED_PROPS_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_NAMED_PROPS_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            key_payload_local,
            function,
        );
        self.emit_property_key_payload_equality_i32(key_payload_local, key_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_DESCRIPTOR_CONFIGURABLE as i64,
        ));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalSet(shift_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(shift_index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(shift_index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(current_entry_local));
        function.instruction(&Instruction::LocalGet(current_entry_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(next_entry_local));

        for offset in [
            HEAP_OBJECT_KEY_OFFSET,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            HEAP_OBJECT_DATA_TAG_OFFSET,
            HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
            HEAP_OBJECT_GETTER_TAG_OFFSET,
            HEAP_OBJECT_GETTER_PAYLOAD_OFFSET,
            HEAP_OBJECT_SETTER_TAG_OFFSET,
            HEAP_OBJECT_SETTER_PAYLOAD_OFFSET,
        ] {
            self.load_i64_from_offset(next_entry_local, offset, function);
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.store_i64_local_at_offset(
                current_entry_local,
                offset,
                self.scratch_local,
                function,
            );
        }

        function.instruction(&Instruction::LocalGet(shift_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(shift_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(len_local));
        self.store_i64_local_at_offset(
            array_local,
            HEAP_ARRAY_NAMED_PROPS_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(next_entry_local);
        self.release_temp_local(current_entry_local);
        self.release_temp_local(shift_index_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
    }

    pub(crate) fn emit_array_delete(
        &mut self,
        array_local: u32,
        index_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));
        self.load_i64_to_local_from_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(array_local, HEAP_LEN_OFFSET, len_local, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(0));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_DESCRIPTOR_CONFIGURABLE as i64,
        ));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        self.store_i64_const_at_offset(
            entry_local,
            HEAP_ARRAY_TAG_OFFSET,
            HEAP_ARRAY_HOLE_TAG as u64,
            function,
        );
        self.store_i64_const_at_offset(entry_local, HEAP_ARRAY_PAYLOAD_OFFSET, 0, function);
        self.store_i64_const_at_offset(entry_local, HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET, 0, function);
        function.instruction(&Instruction::End);

        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
    }

    pub(crate) fn emit_array_has_index_i32(
        &mut self,
        array_local: u32,
        index_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let cap_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let list_ptr_local = self.reserve_temp_local();
        let list_len_local = self.reserve_temp_local();
        let list_index_local = self.reserve_temp_local();
        let candidate_index_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        self.load_i64_to_local_from_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(array_local, HEAP_LEN_OFFSET, len_local, function);
        self.load_i64_to_local_from_offset(array_local, HEAP_CAP_OFFSET, cap_local, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(0));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_PTR_OFFSET,
            list_ptr_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_LEN_OFFSET,
            list_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(list_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(list_ptr_local));
        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_PRESENT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_INDEX_OFFSET,
            candidate_index_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(list_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(candidate_index_local);
        self.release_temp_local(list_index_local);
        self.release_temp_local(list_len_local);
        self.release_temp_local(list_ptr_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(cap_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
    }

    /// Reads the own-property descriptor kind for `index_local` on an
    /// array-shaped heap object (array or arguments object), consulting the
    /// dense buffer when the index is within `cap` and falling back to the
    /// sparse present-index list otherwise. Writes `0` into `result_local`
    /// when the index has no own property (out of `len` bounds, or simply
    /// absent from both the dense buffer and the present-index list).
    ///
    /// This mirrors the bounds-checked lookup used by
    /// [`Self::emit_array_has_index_i32`] and
    /// [`Self::emit_array_advance_to_next_present_index`]; callers that need
    /// to inspect flags (e.g. `OBJECT_DESCRIPTOR_ENUMERABLE`) for an index
    /// already known to be present should use this instead of indexing the
    /// dense buffer directly, which is unsafe for indices `>= cap`.
    pub(crate) fn emit_array_descriptor_kind_for_index(
        &mut self,
        array_local: u32,
        index_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let cap_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let list_ptr_local = self.reserve_temp_local();
        let list_len_local = self.reserve_temp_local();
        let list_index_local = self.reserve_temp_local();
        let candidate_index_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        self.load_i64_to_local_from_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(array_local, HEAP_LEN_OFFSET, len_local, function);
        self.load_i64_to_local_from_offset(array_local, HEAP_CAP_OFFSET, cap_local, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(0));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            result_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_PTR_OFFSET,
            list_ptr_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_LEN_OFFSET,
            list_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(list_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(list_ptr_local));
        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_PRESENT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_INDEX_OFFSET,
            candidate_index_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_DESCRIPTOR_KIND_OFFSET,
            result_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::LocalSet(list_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(list_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(candidate_index_local);
        self.release_temp_local(list_index_local);
        self.release_temp_local(list_len_local);
        self.release_temp_local(list_ptr_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(cap_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
    }

    pub(crate) fn emit_array_advance_to_next_present_index(
        &mut self,
        array_local: u32,
        index_local: u32,
        len_local: u32,
        function: &mut Function,
    ) {
        let buffer_local = self.reserve_temp_local();
        let cap_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let list_ptr_local = self.reserve_temp_local();
        let list_len_local = self.reserve_temp_local();
        let list_index_local = self.reserve_temp_local();
        let candidate_index_local = self.reserve_temp_local();
        let best_index_local = self.reserve_temp_local();
        let list_entry_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(array_local, HEAP_CAP_OFFSET, cap_local, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(0));

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_PTR_OFFSET,
            list_ptr_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_LEN_OFFSET,
            list_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalSet(best_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(list_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(list_ptr_local));
        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_PRESENT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(list_entry_local));
        self.load_i64_to_local_from_offset(
            list_entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_INDEX_OFFSET,
            candidate_index_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::LocalGet(best_index_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::LocalSet(best_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            list_entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::LocalSet(best_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(list_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(best_index_local));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(list_entry_local);
        self.release_temp_local(best_index_local);
        self.release_temp_local(candidate_index_local);
        self.release_temp_local(list_index_local);
        self.release_temp_local(list_len_local);
        self.release_temp_local(list_ptr_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(cap_local);
        self.release_temp_local(buffer_local);
    }

    pub(crate) fn emit_array_retreat_to_previous_present_index(
        &mut self,
        array_local: u32,
        index_local: u32,
        len_local: u32,
        function: &mut Function,
    ) {
        let buffer_local = self.reserve_temp_local();
        let cap_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let list_ptr_local = self.reserve_temp_local();
        let list_len_local = self.reserve_temp_local();
        let list_index_local = self.reserve_temp_local();
        let candidate_index_local = self.reserve_temp_local();
        let best_index_local = self.reserve_temp_local();
        let list_entry_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(array_local, HEAP_CAP_OFFSET, cap_local, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::BrIf(0));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_PTR_OFFSET,
            list_ptr_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_LEN_OFFSET,
            list_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(best_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(list_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(list_ptr_local));
        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_PRESENT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(list_entry_local));
        self.load_i64_to_local_from_offset(
            list_entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_INDEX_OFFSET,
            candidate_index_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::LocalGet(best_index_local));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::LocalSet(best_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            list_entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::LocalSet(best_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(list_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(best_index_local));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(list_entry_local);
        self.release_temp_local(best_index_local);
        self.release_temp_local(candidate_index_local);
        self.release_temp_local(list_index_local);
        self.release_temp_local(list_len_local);
        self.release_temp_local(list_ptr_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(cap_local);
        self.release_temp_local(buffer_local);
    }

    pub(crate) fn emit_string_index_0_to_4_or_minus_one(
        &mut self,
        key_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        let string_offset_local = self.reserve_temp_local();
        let string_len_local = self.reserve_temp_local();
        let byte_index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let digit_local = self.reserve_temp_local();
        let found_local = self.reserve_temp_local();

        self.emit_property_key_payload_is_symbol_i32(key_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Else);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(byte_index_local));

        self.emit_unpack_string_payload(key_local, string_offset_local, string_len_local, function);

        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_string_byte(string_offset_local, byte_index_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));

        self.emit_load_string_byte(string_offset_local, byte_index_local, byte_local, function);
        self.emit_byte_is_digit_i32(byte_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(digit_local));
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::LocalGet(byte_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(byte_index_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I64Const(MAX_ARRAY_LENGTH as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(found_local);
        self.release_temp_local(digit_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(byte_index_local);
        self.release_temp_local(string_len_local);
        self.release_temp_local(string_offset_local);
    }

    pub(crate) fn emit_index_to_flat_map_key_local(
        &mut self,
        index_local: u32,
        number_payload_local: u32,
        key_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(number_payload_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("0")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("1")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("2")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("3")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("4")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::Else);
        self.emit_number_to_string_payload(number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn compile_array_prototype_flat_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let array_constructor_table_index = self
            .functions
            .get(&StandardBuiltinId::ArrayConstructor.function_id())
            .map(|meta| meta.table_index as i64)
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array`",
                )
            })?;
        let this_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Array.prototype.flat receiver",
            )
        })?;
        let this_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Array.prototype.flat receiver tag",
            )
        })?;
        let arg_payload_local = self.reserve_temp_local();
        let arg_tag_local = self.reserve_temp_local();
        let depth_local = self.reserve_temp_local();
        let result_payload_local = self.reserve_temp_local();
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let stack_values_local = self.reserve_temp_local();
        let stack_depths_local = self.reserve_temp_local();
        let zero_local = self.reserve_temp_local();
        let stack_len_local = self.reserve_temp_local();
        let out_index_local = self.reserve_temp_local();
        let current_payload_local = self.reserve_temp_local();
        let current_tag_local = self.reserve_temp_local();
        let current_depth_local = self.reserve_temp_local();
        let current_len_local = self.reserve_temp_local();
        let src_index_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let number_tag_local = self.reserve_temp_local();
        let next_depth_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let object_length_payload_local = self.reserve_temp_local();
        let object_length_tag_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();
        let prototype_local = self.reserve_temp_local();
        let constructor_payload_local = self.reserve_temp_local();
        let constructor_tag_local = self.reserve_temp_local();
        let constructor_table_index_local = self.reserve_temp_local();
        let skip_species_local = self.reserve_temp_local();
        let species_payload_local = self.reserve_temp_local();
        let species_tag_local = self.reserve_temp_local();
        let has_property_local = self.reserve_temp_local();
        let flatten_payload_local = self.reserve_temp_local();
        let flatten_tag_local = self.reserve_temp_local();
        let insert_index_local = self.reserve_temp_local();
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Array.prototype.flat called on null or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(this_payload_local));
        function.instruction(&Instruction::GlobalGet(SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Array.prototype.flat called on null or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(zero_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(number_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(species_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(species_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(skip_species_local));

        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(depth_local));
        function.instruction(&Instruction::Else);
        self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(depth_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_PROTOTYPE_OFFSET,
            prototype_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Cannot convert object to number",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(depth_local));
        function.instruction(&Instruction::Else);
        self.emit_value_to_number_payload(arg_tag_local, arg_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(arg_payload_local));
        function.instruction(&Instruction::LocalGet(arg_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(arg_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(depth_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(arg_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Le);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(depth_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(arg_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(i64::MAX));
        function.instruction(&Instruction::LocalSet(depth_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(arg_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(depth_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_constructor_read(
            this_payload_local,
            constructor_payload_local,
            constructor_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Array.prototype.flat constructor is not object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            constructor_payload_local,
            HEAP_FUNCTION_TABLE_INDEX_OFFSET,
            constructor_table_index_local,
            function,
        );
        self.emit_mark_skip_species_for_cross_realm_array_constructor(
            constructor_payload_local,
            constructor_table_index_local,
            skip_species_local,
            array_constructor_table_index,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(skip_species_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.property_key_symbol_payload("Symbol.species"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            constructor_payload_local,
            constructor_tag_local,
            constructor_payload_local,
            constructor_tag_local,
            key_local,
            species_payload_local,
            species_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(species_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(species_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(species_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Array.prototype.flat constructor is not object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_alloc_array_payload_with_length(zero_local, result_payload_local, function)?;
        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::LocalSet(target_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(target_tag_local));
        function.instruction(&Instruction::LocalGet(species_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_pre_evaluated_arg_vector(
            &[(zero_local, number_tag_local)],
            argc_local,
            argv_local,
            function,
        )?;
        self.emit_function_handle_construct_with_argv(
            species_payload_local,
            species_tag_local,
            species_payload_local,
            species_tag_local,
            argc_local,
            argv_local,
            target_payload_local,
            target_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            target_payload_local,
            target_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_alloc_array_payload_with_length(zero_local, stack_values_local, function)?;
        self.emit_alloc_array_payload_with_length(zero_local, stack_depths_local, function)?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(stack_len_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(out_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(flatten_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(flatten_tag_local));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            this_payload_local,
            HEAP_LEN_OFFSET,
            current_len_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_length(
            this_payload_local,
            object_length_payload_local,
            object_length_tag_local,
            function,
        )?;
        self.emit_to_length_i64_from_value_locals(
            object_length_tag_local,
            object_length_payload_local,
            current_len_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            this_payload_local,
            this_tag_local,
            this_payload_local,
            this_tag_local,
            key_local,
            object_length_payload_local,
            object_length_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            object_length_payload_local,
            object_length_tag_local,
            function,
        )?;
        self.emit_value_to_number_payload(
            object_length_tag_local,
            object_length_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(object_length_payload_local));
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Le);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(i64::MAX));
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Array.prototype.flat receiver is not array",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            this_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            object_length_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            this_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            flatten_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            this_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            flatten_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(flatten_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            this_payload_local,
            this_tag_local,
            this_payload_local,
            this_tag_local,
            key_local,
            constructor_payload_local,
            constructor_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(src_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(insert_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::LocalGet(current_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_index_to_flat_map_key_local(
            src_index_local,
            index_number_payload_local,
            key_local,
            function,
        )?;
        self.emit_object_has_property_i32(
            this_payload_local,
            this_tag_local,
            key_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(has_property_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_read(
            this_payload_local,
            this_tag_local,
            this_payload_local,
            this_tag_local,
            key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_flat_append_depth_one_value(
            target_payload_local,
            target_tag_local,
            out_index_local,
            element_payload_local,
            element_tag_local,
            depth_local,
            key_local,
            has_property_local,
            object_length_payload_local,
            object_length_tag_local,
            index_number_payload_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(src_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(current_len_local));
        function.instruction(&Instruction::LocalSet(src_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(src_index_local));
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_index_get_with_prototype(
            this_payload_local,
            src_index_local,
            this_payload_local,
            this_tag_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_read(
            this_payload_local,
            src_index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("0")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("1")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("2")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("3")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_number_to_string_payload(index_number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_object_read_ordinary(
            this_payload_local,
            this_tag_local,
            this_payload_local,
            this_tag_local,
            key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_array_write(
            stack_values_local,
            stack_len_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_array_write(
            stack_depths_local,
            stack_len_local,
            depth_local,
            number_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(stack_len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(stack_len_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(stack_len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(stack_len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(stack_len_local));
        self.emit_array_read(
            stack_values_local,
            stack_len_local,
            current_payload_local,
            current_tag_local,
            function,
        );
        self.emit_array_read(
            stack_depths_local,
            stack_len_local,
            current_depth_local,
            arg_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(current_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(current_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            object_length_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            flatten_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            flatten_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(flatten_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(current_depth_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(current_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_LEN_OFFSET,
            current_len_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            current_payload_local,
            current_tag_local,
            current_payload_local,
            current_tag_local,
            key_local,
            object_length_payload_local,
            object_length_tag_local,
            function,
        )?;
        self.emit_value_to_number_payload(
            object_length_tag_local,
            object_length_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(object_length_payload_local));
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(current_depth_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(next_depth_local));
        function.instruction(&Instruction::LocalGet(current_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(src_index_local));
        function.instruction(&Instruction::LocalGet(stack_len_local));
        function.instruction(&Instruction::LocalSet(insert_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::LocalGet(current_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_index_to_flat_map_key_local(
            src_index_local,
            index_number_payload_local,
            key_local,
            function,
        )?;
        self.emit_object_has_property_i32(
            current_payload_local,
            current_tag_local,
            key_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(has_property_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_read(
            current_payload_local,
            current_tag_local,
            current_payload_local,
            current_tag_local,
            key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_array_stack_insert(
            stack_values_local,
            stack_depths_local,
            insert_index_local,
            stack_len_local,
            element_payload_local,
            element_tag_local,
            next_depth_local,
            number_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(src_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(current_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(current_len_local));
        function.instruction(&Instruction::LocalSet(src_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(src_index_local));
        self.emit_array_read(
            current_payload_local,
            src_index_local,
            element_payload_local,
            element_tag_local,
            function,
        );
        self.emit_array_write(
            stack_values_local,
            stack_len_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_array_write(
            stack_depths_local,
            stack_len_local,
            next_depth_local,
            number_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(stack_len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(stack_len_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_flat_target_write(
            target_payload_local,
            target_tag_local,
            out_index_local,
            current_payload_local,
            current_tag_local,
            index_number_payload_local,
            key_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(out_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(out_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);

        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(insert_index_local);
        self.release_temp_local(flatten_tag_local);
        self.release_temp_local(flatten_payload_local);
        self.release_temp_local(has_property_local);
        self.release_temp_local(species_tag_local);
        self.release_temp_local(species_payload_local);
        self.release_temp_local(skip_species_local);
        self.release_temp_local(constructor_table_index_local);
        self.release_temp_local(constructor_tag_local);
        self.release_temp_local(constructor_payload_local);
        self.release_temp_local(prototype_local);
        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(object_length_tag_local);
        self.release_temp_local(object_length_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(next_depth_local);
        self.release_temp_local(number_tag_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(src_index_local);
        self.release_temp_local(current_len_local);
        self.release_temp_local(current_depth_local);
        self.release_temp_local(current_tag_local);
        self.release_temp_local(current_payload_local);
        self.release_temp_local(out_index_local);
        self.release_temp_local(stack_len_local);
        self.release_temp_local(zero_local);
        self.release_temp_local(stack_depths_local);
        self.release_temp_local(stack_values_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        self.release_temp_local(result_payload_local);
        self.release_temp_local(depth_local);
        self.release_temp_local(arg_tag_local);
        self.release_temp_local(arg_payload_local);
        Ok(())
    }

    pub(crate) fn emit_flat_append_depth_one_value(
        &mut self,
        target_payload_local: u32,
        target_tag_local: u32,
        out_index_local: u32,
        element_payload_local: u32,
        element_tag_local: u32,
        depth_local: u32,
        key_local: u32,
        has_property_local: u32,
        object_length_payload_local: u32,
        object_length_tag_local: u32,
        index_number_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let flatten_payload_local = self.reserve_temp_local();
        let flatten_tag_local = self.reserve_temp_local();
        let child_len_local = self.reserve_temp_local();
        let child_index_local = self.reserve_temp_local();
        let child_payload_local = self.reserve_temp_local();
        let child_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(flatten_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(flatten_tag_local));

        function.instruction(&Instruction::LocalGet(element_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            element_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            object_length_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            element_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            flatten_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            element_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            flatten_tag_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(depth_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::LocalGet(element_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(flatten_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(element_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            element_payload_local,
            HEAP_LEN_OFFSET,
            child_len_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            element_payload_local,
            element_tag_local,
            element_payload_local,
            element_tag_local,
            key_local,
            object_length_payload_local,
            object_length_tag_local,
            function,
        )?;
        self.emit_value_to_number_payload(
            object_length_tag_local,
            object_length_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(object_length_payload_local));
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(child_len_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(child_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(child_index_local));
        function.instruction(&Instruction::LocalGet(child_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(element_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_has_index_i32(
            element_payload_local,
            child_index_local,
            has_property_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_index_to_flat_map_key_local(
            child_index_local,
            index_number_payload_local,
            key_local,
            function,
        )?;
        self.emit_object_has_property_i32(
            element_payload_local,
            element_tag_local,
            key_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(has_property_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(element_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_read(
            element_payload_local,
            child_index_local,
            child_payload_local,
            child_tag_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_object_read(
            element_payload_local,
            element_tag_local,
            element_payload_local,
            element_tag_local,
            key_local,
            child_payload_local,
            child_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_flat_target_write(
            target_payload_local,
            target_tag_local,
            out_index_local,
            child_payload_local,
            child_tag_local,
            index_number_payload_local,
            key_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(out_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(out_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(child_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(child_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_flat_target_write(
            target_payload_local,
            target_tag_local,
            out_index_local,
            element_payload_local,
            element_tag_local,
            index_number_payload_local,
            key_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(out_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(out_index_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(child_tag_local);
        self.release_temp_local(child_payload_local);
        self.release_temp_local(child_index_local);
        self.release_temp_local(child_len_local);
        self.release_temp_local(flatten_tag_local);
        self.release_temp_local(flatten_payload_local);
        Ok(())
    }

    pub(crate) fn emit_flat_target_write(
        &mut self,
        target_payload_local: u32,
        target_tag_local: u32,
        out_index_local: u32,
        payload_local: u32,
        tag_local: u32,
        index_number_payload_local: u32,
        key_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_write(
            target_payload_local,
            out_index_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(out_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_number_to_string_payload(index_number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_create_data_property_or_throw(
            target_payload_local,
            target_tag_local,
            key_local,
            payload_local,
            tag_local,
            "Array.prototype.flatMap cannot define non-configurable target property",
            "Array.prototype.flatMap cannot add property to non-extensible target",
            None,
            function,
        )?;
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_array_stack_insert(
        &mut self,
        stack_values_local: u32,
        stack_depths_local: u32,
        insert_index_local: u32,
        stack_len_local: u32,
        element_payload_local: u32,
        element_tag_local: u32,
        depth_payload_local: u32,
        depth_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let shift_index_local = self.reserve_temp_local();
        let shifted_payload_local = self.reserve_temp_local();
        let shifted_tag_local = self.reserve_temp_local();
        let shifted_depth_payload_local = self.reserve_temp_local();
        let shifted_depth_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(stack_len_local));
        function.instruction(&Instruction::LocalSet(shift_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(shift_index_local));
        function.instruction(&Instruction::LocalGet(insert_index_local));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(shift_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_array_read(
            stack_values_local,
            self.scratch_local,
            shifted_payload_local,
            shifted_tag_local,
            function,
        );
        self.emit_array_read(
            stack_depths_local,
            self.scratch_local,
            shifted_depth_payload_local,
            shifted_depth_tag_local,
            function,
        );
        self.emit_array_write(
            stack_values_local,
            shift_index_local,
            shifted_payload_local,
            shifted_tag_local,
            function,
        )?;
        self.emit_array_write(
            stack_depths_local,
            shift_index_local,
            shifted_depth_payload_local,
            shifted_depth_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(shift_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(shift_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_array_write(
            stack_values_local,
            insert_index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_array_write(
            stack_depths_local,
            insert_index_local,
            depth_payload_local,
            depth_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(stack_len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(stack_len_local));

        self.release_temp_local(shifted_depth_tag_local);
        self.release_temp_local(shifted_depth_payload_local);
        self.release_temp_local(shifted_tag_local);
        self.release_temp_local(shifted_payload_local);
        self.release_temp_local(shift_index_local);
        Ok(())
    }

    pub(crate) fn emit_concat_create_target_property(
        &mut self,
        target_payload_local: u32,
        target_tag_local: u32,
        index_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        index_number_payload_local: u32,
        key_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        let array_index_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalSet(array_index_local));
        self.emit_array_write(
            target_payload_local,
            array_index_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.release_temp_local(array_index_local);
        function.instruction(&Instruction::Else);
        self.emit_index_to_flat_map_key_local(
            index_local,
            index_number_payload_local,
            key_local,
            function,
        )?;
        self.emit_create_data_property_or_throw(
            target_payload_local,
            target_tag_local,
            key_local,
            value_payload_local,
            value_tag_local,
            "Array.prototype.concat cannot define non-configurable target property",
            "Array.prototype.concat cannot add property to non-extensible target",
            None,
            function,
        )?;
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_concat_length_of_array_like(
        &mut self,
        item_payload_local: u32,
        item_tag_local: u32,
        length_local: u32,
        object_length_payload_local: u32,
        object_length_tag_local: u32,
        key_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(item_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_length(
            item_payload_local,
            object_length_payload_local,
            object_length_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(length_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(item_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_length(
            item_payload_local,
            object_length_payload_local,
            object_length_tag_local,
            function,
        )?;
        self.emit_value_to_number_payload(
            object_length_tag_local,
            object_length_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(object_length_payload_local));
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(length_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            item_payload_local,
            item_tag_local,
            item_payload_local,
            item_tag_local,
            key_local,
            object_length_payload_local,
            object_length_tag_local,
            function,
        )?;
        self.emit_value_to_number_payload(
            object_length_tag_local,
            object_length_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(object_length_payload_local));
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(length_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Le);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(length_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(i64::MAX));
        function.instruction(&Instruction::LocalSet(length_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(length_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_concat_typed_array_has_index_i32(
        &mut self,
        item_payload_local: u32,
        item_tag_local: u32,
        index_local: u32,
        result_local: u32,
        typed_array_like_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_payload_local = self.reserve_temp_local();
        let byte_offset_local = self.reserve_temp_local();
        let stored_byte_length_local = self.reserve_temp_local();
        let bytes_per_element_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(typed_array_like_local));

        self.emit_is_typed_array_i32(item_payload_local, item_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(typed_array_like_local));
        self.emit_load_typed_array_private_state(
            item_payload_local,
            buffer_payload_local,
            byte_offset_local,
            stored_byte_length_local,
            bytes_per_element_local,
            function,
        );
        let typed_array_view = TypedArrayViewLocals::new(
            item_payload_local,
            buffer_payload_local,
            byte_offset_local,
            stored_byte_length_local,
            bytes_per_element_local,
        );
        self.emit_typed_array_witness(
            &typed_array_view,
            TypedArrayWitnessUse::IntegerIndexedProperty {
                index_local,
                result_local,
            },
            function,
        )?;
        function.instruction(&Instruction::End);

        self.release_temp_local(bytes_per_element_local);
        self.release_temp_local(stored_byte_length_local);
        self.release_temp_local(byte_offset_local);
        self.release_temp_local(buffer_payload_local);
        Ok(())
    }

    pub(crate) fn compile_array_prototype_concat_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let this_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Array.prototype.concat receiver",
            )
        })?;
        let this_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Array.prototype.concat receiver tag",
            )
        })?;
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let zero_local = self.reserve_temp_local();
        let number_tag_local = self.reserve_temp_local();
        let constructor_payload_local = self.reserve_temp_local();
        let constructor_tag_local = self.reserve_temp_local();
        let constructor_table_index_local = self.reserve_temp_local();
        let skip_species_local = self.reserve_temp_local();
        let species_payload_local = self.reserve_temp_local();
        let species_tag_local = self.reserve_temp_local();
        let item_index_local = self.reserve_temp_local();
        let arg_index_local = self.reserve_temp_local();
        let item_payload_local = self.reserve_temp_local();
        let item_tag_local = self.reserve_temp_local();
        let src_index_local = self.reserve_temp_local();
        let src_len_local = self.reserve_temp_local();
        let out_index_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let has_property_local = self.reserve_temp_local();
        let spreadable_payload_local = self.reserve_temp_local();
        let spreadable_tag_local = self.reserve_temp_local();
        let spreadable_flag_local = self.reserve_temp_local();
        let typed_array_like_local = self.reserve_temp_local();
        let object_length_payload_local = self.reserve_temp_local();
        let object_length_tag_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();
        let array_constructor_table_index = self
            .functions
            .get(&StandardBuiltinId::ArrayConstructor.function_id())
            .map(|meta| meta.table_index as i64)
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array`",
                )
            })?;

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Array.prototype.concat called on null or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(this_payload_local));
        function.instruction(&Instruction::GlobalGet(SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Array.prototype.concat called on null or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(zero_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(number_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(species_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(species_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(skip_species_local));

        self.emit_array_iteration_to_object(this_payload_local, this_tag_local, function)?;

        self.emit_is_array_i64(
            this_payload_local,
            this_tag_local,
            spreadable_flag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(spreadable_flag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_constructor_read(
            this_payload_local,
            constructor_payload_local,
            constructor_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            this_payload_local,
            this_tag_local,
            this_payload_local,
            this_tag_local,
            key_local,
            constructor_payload_local,
            constructor_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_current_function_realm_type_error(
            "Array.prototype.concat constructor is not object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            constructor_payload_local,
            HEAP_FUNCTION_TABLE_INDEX_OFFSET,
            constructor_table_index_local,
            function,
        );
        self.emit_mark_skip_species_for_cross_realm_array_constructor(
            constructor_payload_local,
            constructor_table_index_local,
            skip_species_local,
            array_constructor_table_index,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(skip_species_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.property_key_symbol_payload("Symbol.species"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            constructor_payload_local,
            constructor_tag_local,
            constructor_payload_local,
            constructor_tag_local,
            key_local,
            species_payload_local,
            species_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(species_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(species_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(species_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_current_function_realm_type_error(
            "Array.prototype.concat constructor is not object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_alloc_array_payload_with_length(zero_local, target_payload_local, function)?;
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(target_tag_local));
        function.instruction(&Instruction::LocalGet(species_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_pre_evaluated_arg_vector(
            &[(zero_local, number_tag_local)],
            argc_local,
            argv_local,
            function,
        )?;
        self.emit_function_handle_construct_with_argv(
            species_payload_local,
            species_tag_local,
            species_payload_local,
            species_tag_local,
            argc_local,
            argv_local,
            target_payload_local,
            target_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            target_payload_local,
            target_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(item_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(out_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(item_index_local));
        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(item_index_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(this_payload_local));
        function.instruction(&Instruction::LocalSet(item_payload_local));
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::LocalSet(item_tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(item_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(arg_index_local));
        let saved_out_index_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(out_index_local));
        function.instruction(&Instruction::LocalSet(saved_out_index_local));
        self.emit_array_read(
            self.argv_param_local(),
            arg_index_local,
            item_payload_local,
            item_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(saved_out_index_local));
        function.instruction(&Instruction::LocalSet(out_index_local));
        self.release_temp_local(saved_out_index_local);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(has_property_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(spreadable_flag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(spreadable_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(spreadable_tag_local));

        function.instruction(&Instruction::LocalGet(item_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(item_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(item_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(item_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.isConcatSpreadable"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::LocalGet(item_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_is_concat_spreadable_read(
            item_payload_local,
            spreadable_payload_local,
            spreadable_tag_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_object_read(
            item_payload_local,
            item_tag_local,
            item_payload_local,
            item_tag_local,
            key_local,
            spreadable_payload_local,
            spreadable_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_propagate_throw_from_locals_if_needed(
            spreadable_payload_local,
            spreadable_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(spreadable_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.compile_truthy_tagged_i32(spreadable_tag_local, spreadable_payload_local, function)?;
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(spreadable_flag_local));
        function.instruction(&Instruction::Else);
        self.emit_is_array_i64(
            item_payload_local,
            item_tag_local,
            spreadable_flag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(spreadable_flag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_concat_length_of_array_like(
            item_payload_local,
            item_tag_local,
            src_len_local,
            object_length_payload_local,
            object_length_tag_local,
            key_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(MAX_SAFE_INTEGER as i64));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalGet(out_index_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Array.prototype.concat result exceeds maximum safe length",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(src_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_index_to_flat_map_key_local(
            src_index_local,
            index_number_payload_local,
            key_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(typed_array_like_local));
        function.instruction(&Instruction::LocalGet(item_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_has_property_i32(
            item_payload_local,
            item_tag_local,
            key_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(item_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_has_index_i32(
            item_payload_local,
            src_index_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_concat_typed_array_has_index_i32(
            item_payload_local,
            item_tag_local,
            src_index_local,
            has_property_local,
            typed_array_like_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(typed_array_like_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_has_property_i32(
            item_payload_local,
            item_tag_local,
            key_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(has_property_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(item_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_index_get_with_prototype(
            item_payload_local,
            src_index_local,
            item_payload_local,
            item_tag_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(item_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_read(
            item_payload_local,
            src_index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(typed_array_like_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_or_object_index_read_from_locals(
            item_payload_local,
            item_tag_local,
            src_index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_object_read(
            item_payload_local,
            item_tag_local,
            item_payload_local,
            item_tag_local,
            key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_propagate_throw_from_locals_if_needed(
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_concat_create_target_property(
            target_payload_local,
            target_tag_local,
            out_index_local,
            element_payload_local,
            element_tag_local,
            index_number_payload_local,
            key_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(src_index_local));
        function.instruction(&Instruction::LocalGet(out_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(out_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(spreadable_flag_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(out_index_local));
        function.instruction(&Instruction::I64Const(MAX_SAFE_INTEGER as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Array.prototype.concat result exceeds maximum safe length",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_concat_create_target_property(
            target_payload_local,
            target_tag_local,
            out_index_local,
            item_payload_local,
            item_tag_local,
            index_number_payload_local,
            key_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(out_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(out_index_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(item_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(item_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_local_at_offset(
            target_payload_local,
            HEAP_LEN_OFFSET,
            out_index_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(out_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_write(
            target_payload_local,
            target_tag_local,
            key_local,
            index_number_payload_local,
            number_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);

        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(key_local);
        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(object_length_tag_local);
        self.release_temp_local(object_length_payload_local);
        self.release_temp_local(typed_array_like_local);
        self.release_temp_local(spreadable_flag_local);
        self.release_temp_local(spreadable_tag_local);
        self.release_temp_local(spreadable_payload_local);
        self.release_temp_local(has_property_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(out_index_local);
        self.release_temp_local(src_len_local);
        self.release_temp_local(src_index_local);
        self.release_temp_local(item_tag_local);
        self.release_temp_local(item_payload_local);
        self.release_temp_local(arg_index_local);
        self.release_temp_local(item_index_local);
        self.release_temp_local(species_tag_local);
        self.release_temp_local(species_payload_local);
        self.release_temp_local(skip_species_local);
        self.release_temp_local(constructor_table_index_local);
        self.release_temp_local(constructor_tag_local);
        self.release_temp_local(constructor_payload_local);
        self.release_temp_local(number_tag_local);
        self.release_temp_local(zero_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        Ok(())
    }

    pub(crate) fn compile_array_prototype_flat_map_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let this_payload_local = self
            .this_payload_local
            .ok_or_else(|| EmitError::unsupported("missing Array.prototype.flatMap receiver"))?;
        let this_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported("missing Array.prototype.flatMap receiver tag")
        })?;
        let mapper_payload_local = self.reserve_temp_local();
        let mapper_tag_local = self.reserve_temp_local();
        let this_arg_payload_local = self.reserve_temp_local();
        let this_arg_tag_local = self.reserve_temp_local();
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let source_len_local = self.reserve_temp_local();
        let source_index_local = self.reserve_temp_local();
        let target_index_local = self.reserve_temp_local();
        let zero_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let length_payload_local = self.reserve_temp_local();
        let length_tag_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let mapped_payload_local = self.reserve_temp_local();
        let mapped_tag_local = self.reserve_temp_local();
        let mapped_len_local = self.reserve_temp_local();
        let mapped_index_local = self.reserve_temp_local();
        let child_payload_local = self.reserve_temp_local();
        let child_tag_local = self.reserve_temp_local();
        let present_local = self.reserve_temp_local();
        let is_array_local = self.reserve_temp_local();
        let index_payload_local = self.reserve_temp_local();
        let number_tag_local = self.reserve_temp_local();
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();

        // ToObject and LengthOfArrayLike precede IsCallable and ArraySpeciesCreate.
        // In particular, a TypedArray's observable length property is not its
        // private element count. Get/HasProperty below own live buffer witnesses.
        self.emit_array_iteration_length_before_callback_validation(
            this_payload_local,
            this_tag_local,
            key_local,
            length_payload_local,
            length_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(length_payload_local));
        function.instruction(&Instruction::LocalSet(source_len_local));
        self.emit_builtin_arg_to_locals(0, mapper_payload_local, mapper_tag_local, function);
        self.emit_is_callable_i32(mapper_tag_local, mapper_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Array.prototype.flatMap mapper is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_builtin_arg_to_locals(1, this_arg_payload_local, this_arg_tag_local, function);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(zero_local));
        self.emit_array_species_create(
            this_payload_local,
            this_tag_local,
            zero_local,
            target_payload_local,
            target_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(number_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(source_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(target_index_local));

        // FlattenIntoArray with a mapper and depth one: only present source
        // properties invoke the callback; only actual Arrays flatten its result.
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(source_index_local));
        function.instruction(&Instruction::LocalGet(source_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_index_to_flat_map_key_local(
            source_index_local,
            index_payload_local,
            key_local,
            function,
        )?;
        self.emit_object_has_property_i32(
            this_payload_local,
            this_tag_local,
            key_local,
            present_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_typed_array_or_object_index_read_from_locals(
            this_payload_local,
            this_tag_local,
            source_index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(source_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_payload_local));
        self.emit_pre_evaluated_arg_vector(
            &[
                (element_payload_local, element_tag_local),
                (index_payload_local, number_tag_local),
                (this_payload_local, this_tag_local),
            ],
            argc_local,
            argv_local,
            function,
        )?;
        self.emit_function_or_proxy_call_with_argv_leave_throw_completion(
            mapper_payload_local,
            mapper_tag_local,
            this_arg_payload_local,
            this_arg_tag_local,
            argc_local,
            argv_local,
            mapped_payload_local,
            mapped_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            mapped_payload_local,
            mapped_tag_local,
            function,
        )?;
        self.emit_is_array_i64(
            mapped_payload_local,
            mapped_tag_local,
            is_array_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(is_array_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_flat_map_append(
            target_payload_local,
            target_tag_local,
            target_index_local,
            mapped_payload_local,
            mapped_tag_local,
            key_local,
            index_payload_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        // Keep the original Proxy as the Get/HasProperty receiver: IsArray may
        // inspect its target, but must not bypass traps or revocation afterward.
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            mapped_payload_local,
            mapped_tag_local,
            mapped_payload_local,
            mapped_tag_local,
            key_local,
            length_payload_local,
            length_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            length_payload_local,
            length_tag_local,
            function,
        )?;
        self.emit_to_length_i64_from_value_locals(
            length_tag_local,
            length_payload_local,
            mapped_len_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(mapped_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(mapped_index_local));
        function.instruction(&Instruction::LocalGet(mapped_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_index_to_flat_map_key_local(
            mapped_index_local,
            index_payload_local,
            key_local,
            function,
        )?;
        self.emit_object_has_property_i32(
            mapped_payload_local,
            mapped_tag_local,
            key_local,
            present_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_object_read(
            mapped_payload_local,
            mapped_tag_local,
            mapped_payload_local,
            mapped_tag_local,
            key_local,
            child_payload_local,
            child_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            child_payload_local,
            child_tag_local,
            function,
        )?;
        self.emit_flat_map_append(
            target_payload_local,
            target_tag_local,
            target_index_local,
            child_payload_local,
            child_tag_local,
            key_local,
            index_payload_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(mapped_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(mapped_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(source_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(source_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(number_tag_local);
        self.release_temp_local(index_payload_local);
        self.release_temp_local(is_array_local);
        self.release_temp_local(present_local);
        self.release_temp_local(child_tag_local);
        self.release_temp_local(child_payload_local);
        self.release_temp_local(mapped_index_local);
        self.release_temp_local(mapped_len_local);
        self.release_temp_local(mapped_tag_local);
        self.release_temp_local(mapped_payload_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(length_tag_local);
        self.release_temp_local(length_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(zero_local);
        self.release_temp_local(target_index_local);
        self.release_temp_local(source_index_local);
        self.release_temp_local(source_len_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        self.release_temp_local(this_arg_tag_local);
        self.release_temp_local(this_arg_payload_local);
        self.release_temp_local(mapper_tag_local);
        self.release_temp_local(mapper_payload_local);
        Ok(())
    }

    fn emit_flat_map_append(
        &mut self,
        target_payload_local: u32,
        target_tag_local: u32,
        target_index_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        key_local: u32,
        index_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(target_index_local));
        function.instruction(&Instruction::I64Const(MAX_SAFE_INTEGER as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Array.prototype.flatMap result exceeds the maximum safe length",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_index_to_flat_map_key_local(
            target_index_local,
            index_payload_local,
            key_local,
            function,
        )?;
        self.emit_array_target_create_data_property_or_throw(
            target_payload_local,
            target_tag_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(target_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(target_index_local));
        Ok(())
    }

    pub(crate) fn emit_array_iteration_length_before_callback_validation(
        &mut self,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        key_local: u32,
        length_payload_local: u32,
        length_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_value_to_current_function_realm_object_locals(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            length_payload_local,
            length_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            length_payload_local,
            length_tag_local,
            function,
        )?;
        self.emit_to_length_i64_from_value_locals(
            length_tag_local,
            length_payload_local,
            length_payload_local,
            function,
        )?;
        Ok(())
    }

    pub(crate) fn compile_array_prototype_map_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_array_callback_iteration(function, ArrayCallbackIterationKind::Map)
    }

    pub(crate) fn compile_typed_array_prototype_slice_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing TypedArray.prototype.slice receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing TypedArray.prototype.slice receiver tag",
            )
        })?;
        let typed_array_brand_local = self.reserve_temp_local();
        let source_buffer_payload_local = self.reserve_temp_local();
        let source_buffer_pointer_local = self.reserve_temp_local();
        let source_byte_offset_local = self.reserve_temp_local();
        let source_stored_byte_length_local = self.reserve_temp_local();
        let source_bytes_per_element_local = self.reserve_temp_local();
        let source_element_kind_local = self.reserve_temp_local();
        let source_length_local = self.reserve_temp_local();
        let start_payload_local = self.reserve_temp_local();
        let start_tag_local = self.reserve_temp_local();
        let start_index_local = self.reserve_temp_local();
        let end_payload_local = self.reserve_temp_local();
        let end_tag_local = self.reserve_temp_local();
        let end_index_local = self.reserve_temp_local();
        let count_local = self.reserve_temp_local();
        let constructor_key_local = self.reserve_temp_local();
        let constructor_payload_local = self.reserve_temp_local();
        let constructor_tag_local = self.reserve_temp_local();
        let constructor_property_payload_local = self.reserve_temp_local();
        let constructor_property_tag_local = self.reserve_temp_local();
        let species_payload_local = self.reserve_temp_local();
        let species_tag_local = self.reserve_temp_local();
        let count_payload_local = self.reserve_temp_local();
        let number_tag_local = self.reserve_temp_local();
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let target_buffer_payload_local = self.reserve_temp_local();
        let target_buffer_pointer_local = self.reserve_temp_local();
        let target_byte_offset_local = self.reserve_temp_local();
        let target_element_kind_local = self.reserve_temp_local();
        let current_source_length_local = self.reserve_temp_local();
        let copied_element_count_local = self.reserve_temp_local();
        let copied_byte_count_local = self.reserve_temp_local();
        let copy_index_local = self.reserve_temp_local();
        let source_address_local = self.reserve_temp_local();
        let target_address_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let target_index_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(typed_array_brand_local));
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
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
            "TypedArray.prototype.slice requires a TypedArray",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_load_typed_array_private_state(
            receiver_payload_local,
            source_buffer_payload_local,
            source_byte_offset_local,
            source_stored_byte_length_local,
            source_bytes_per_element_local,
            function,
        );
        let source_view = TypedArrayViewLocals::new(
            receiver_payload_local,
            source_buffer_payload_local,
            source_byte_offset_local,
            source_stored_byte_length_local,
            source_bytes_per_element_local,
        );
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET,
            source_element_kind_local,
            function,
        );
        self.emit_typed_array_witness(
            &source_view,
            TypedArrayWitnessUse::ValidatedMethodEntry {
                length_local: source_length_local,
            },
            function,
        )?;

        self.emit_builtin_arg_to_locals(0, start_payload_local, start_tag_local, function);
        self.emit_value_to_number_payload(start_tag_local, start_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(start_payload_local));
        self.emit_return_current_completion_if_throw(function);
        self.emit_to_integer_or_infinity_number_payload_from_number_payload(
            start_payload_local,
            start_payload_local,
            function,
        );
        self.emit_array_slice_clamped_index(
            start_payload_local,
            source_length_local,
            start_index_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(source_length_local));
        function.instruction(&Instruction::LocalSet(end_index_local));
        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_builtin_arg_to_locals(1, end_payload_local, end_tag_local, function);
        function.instruction(&Instruction::LocalGet(end_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_number_payload(end_tag_local, end_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(end_payload_local));
        self.emit_return_current_completion_if_throw(function);
        self.emit_to_integer_or_infinity_number_payload_from_number_payload(
            end_payload_local,
            end_payload_local,
            function,
        );
        self.emit_array_slice_clamped_index(
            end_payload_local,
            source_length_local,
            end_index_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(count_local));
        function.instruction(&Instruction::LocalGet(end_index_local));
        function.instruction(&Instruction::LocalGet(start_index_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(end_index_local));
        function.instruction(&Instruction::LocalGet(start_index_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(count_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(constructor_payload_local));
        for (constructor, _) in typed_array_constructor_bytes_per_element_entries() {
            let constructor_global_index = standard_builtin_constructor_global_index(constructor)
                .ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in lila wasm-aot first slice: missing typed array constructor global `{}`",
                        constructor.debug_name()
                    ))
                })?;
            function.instruction(&Instruction::LocalGet(source_element_kind_local));
            function.instruction(&Instruction::I64Const(
                typed_array_element_kind(constructor) as i64,
            ));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::GlobalGet(constructor_global_index));
            function.instruction(&Instruction::LocalSet(constructor_payload_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(constructor_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "TypedArray.prototype.slice has unknown element type",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(constructor_tag_local));

        function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
        function.instruction(&Instruction::LocalSet(constructor_key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            constructor_key_local,
            constructor_property_payload_local,
            constructor_property_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(constructor_property_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_is_heap_object_like_tag_i32(constructor_property_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "TypedArray.prototype.slice constructor property is not an object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(
            self.strings.property_key_symbol_payload("Symbol.species"),
        ));
        function.instruction(&Instruction::LocalSet(constructor_key_local));
        self.emit_object_read(
            constructor_property_payload_local,
            constructor_property_tag_local,
            constructor_property_payload_local,
            constructor_property_tag_local,
            constructor_key_local,
            species_payload_local,
            species_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.compile_nullish_tagged_i32(species_tag_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_is_constructor_i32(species_tag_local, species_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "TypedArray.prototype.slice species is not a constructor",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(species_payload_local));
        function.instruction(&Instruction::LocalSet(constructor_payload_local));
        function.instruction(&Instruction::LocalGet(species_tag_local));
        function.instruction(&Instruction::LocalSet(constructor_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(count_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(count_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(number_tag_local));
        self.emit_pre_evaluated_arg_vector(
            &[(count_payload_local, number_tag_local)],
            argc_local,
            argv_local,
            function,
        )?;
        self.emit_function_or_proxy_construct_with_argv(
            constructor_payload_local,
            constructor_tag_local,
            constructor_payload_local,
            constructor_tag_local,
            argc_local,
            argv_local,
            target_payload_local,
            target_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_validate_typed_array_from_constructed_target(
            target_payload_local,
            target_tag_local,
            count_payload_local,
            function,
        )?;
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET,
            target_element_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(source_element_kind_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(target_element_kind_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I32Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "TypedArray.prototype.slice species content type differs",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(count_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_typed_array_witness(
            &source_view,
            TypedArrayWitnessUse::ValidatedMethodEntry {
                length_local: current_source_length_local,
            },
            function,
        )?;
        function.instruction(&Instruction::LocalGet(end_index_local));
        function.instruction(&Instruction::LocalGet(current_source_length_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(current_source_length_local));
        function.instruction(&Instruction::LocalSet(end_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(copied_element_count_local));
        function.instruction(&Instruction::LocalGet(end_index_local));
        function.instruction(&Instruction::LocalGet(start_index_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(end_index_local));
        function.instruction(&Instruction::LocalGet(start_index_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(copied_element_count_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(source_element_kind_local));
        function.instruction(&Instruction::LocalGet(target_element_kind_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_array_buffer_data(
            source_buffer_payload_local,
            source_buffer_pointer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_TYPED_ARRAY_VIEWED_BUFFER_OFFSET,
            target_buffer_payload_local,
            function,
        );
        self.emit_load_array_buffer_data(
            target_buffer_payload_local,
            target_buffer_pointer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_TYPED_ARRAY_BYTE_OFFSET,
            target_byte_offset_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(source_buffer_pointer_local));
        function.instruction(&Instruction::LocalGet(source_byte_offset_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(start_index_local));
        function.instruction(&Instruction::LocalGet(source_bytes_per_element_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(source_address_local));
        function.instruction(&Instruction::LocalGet(target_buffer_pointer_local));
        function.instruction(&Instruction::LocalGet(target_byte_offset_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(target_address_local));

        // Slice copies same-type elements as ascending bytes. Forward copying
        // is observable when a species constructor returns an overlapping view.
        function.instruction(&Instruction::LocalGet(copied_element_count_local));
        function.instruction(&Instruction::LocalGet(source_bytes_per_element_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(copied_byte_count_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(copy_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(copy_index_local));
        function.instruction(&Instruction::LocalGet(copied_byte_count_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(target_address_local));
        function.instruction(&Instruction::LocalGet(copy_index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(source_address_local));
        function.instruction(&Instruction::LocalGet(copy_index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(self.buffer_memarg8(0)));
        function.instruction(&Instruction::I32Store8(self.buffer_memarg8(0)));
        function.instruction(&Instruction::LocalGet(copy_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(copy_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);

        function.instruction(&Instruction::LocalGet(start_index_local));
        function.instruction(&Instruction::LocalSet(copy_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(copy_index_local));
        function.instruction(&Instruction::LocalGet(end_index_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_typed_array_or_object_index_read_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            copy_index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(copy_index_local));
        function.instruction(&Instruction::LocalGet(start_index_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(target_index_local));
        self.emit_typed_array_element_write_from_locals(
            target_payload_local,
            target_index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(copy_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(copy_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(target_index_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(target_address_local);
        self.release_temp_local(source_address_local);
        self.release_temp_local(copy_index_local);
        self.release_temp_local(copied_byte_count_local);
        self.release_temp_local(copied_element_count_local);
        self.release_temp_local(current_source_length_local);
        self.release_temp_local(target_element_kind_local);
        self.release_temp_local(target_byte_offset_local);
        self.release_temp_local(target_buffer_pointer_local);
        self.release_temp_local(target_buffer_payload_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(number_tag_local);
        self.release_temp_local(count_payload_local);
        self.release_temp_local(species_tag_local);
        self.release_temp_local(species_payload_local);
        self.release_temp_local(constructor_property_tag_local);
        self.release_temp_local(constructor_property_payload_local);
        self.release_temp_local(constructor_tag_local);
        self.release_temp_local(constructor_payload_local);
        self.release_temp_local(constructor_key_local);
        self.release_temp_local(count_local);
        self.release_temp_local(end_index_local);
        self.release_temp_local(end_tag_local);
        self.release_temp_local(end_payload_local);
        self.release_temp_local(start_index_local);
        self.release_temp_local(start_tag_local);
        self.release_temp_local(start_payload_local);
        self.release_temp_local(source_length_local);
        self.release_temp_local(source_element_kind_local);
        self.release_temp_local(source_bytes_per_element_local);
        self.release_temp_local(source_stored_byte_length_local);
        self.release_temp_local(source_byte_offset_local);
        self.release_temp_local(source_buffer_pointer_local);
        self.release_temp_local(source_buffer_payload_local);
        self.release_temp_local(typed_array_brand_local);
        Ok(())
    }

    pub(crate) fn compile_typed_array_prototype_map_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing TypedArray.prototype.map receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing TypedArray.prototype.map receiver tag",
            )
        })?;
        let typed_array_brand_local = self.reserve_temp_local();
        let buffer_payload_local = self.reserve_temp_local();
        let byte_offset_local = self.reserve_temp_local();
        let byte_length_local = self.reserve_temp_local();
        let bytes_per_element_local = self.reserve_temp_local();
        let element_kind_local = self.reserve_temp_local();
        let length_local = self.reserve_temp_local();
        let callback_payload_local = self.reserve_temp_local();
        let callback_tag_local = self.reserve_temp_local();
        let this_arg_payload_local = self.reserve_temp_local();
        let this_arg_tag_local = self.reserve_temp_local();
        let constructor_key_local = self.reserve_temp_local();
        let constructor_payload_local = self.reserve_temp_local();
        let constructor_tag_local = self.reserve_temp_local();
        let constructor_property_payload_local = self.reserve_temp_local();
        let constructor_property_tag_local = self.reserve_temp_local();
        let species_payload_local = self.reserve_temp_local();
        let species_tag_local = self.reserve_temp_local();
        let length_payload_local = self.reserve_temp_local();
        let length_tag_local = self.reserve_temp_local();
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let target_element_kind_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let mapped_payload_local = self.reserve_temp_local();
        let mapped_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(typed_array_brand_local));
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
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
            "TypedArray.prototype.map requires a TypedArray",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_load_typed_array_private_state(
            receiver_payload_local,
            buffer_payload_local,
            byte_offset_local,
            byte_length_local,
            bytes_per_element_local,
            function,
        );
        let receiver_view = TypedArrayViewLocals::new(
            receiver_payload_local,
            buffer_payload_local,
            byte_offset_local,
            byte_length_local,
            bytes_per_element_local,
        );
        self.emit_typed_array_witness(
            &receiver_view,
            TypedArrayWitnessUse::ValidatedMethodEntry { length_local },
            function,
        )?;
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET,
            element_kind_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "TypedArray.prototype.map callback is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_builtin_arg_to_locals(0, callback_payload_local, callback_tag_local, function);
        self.emit_is_callable_i32(callback_tag_local, callback_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "TypedArray.prototype.map callback is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(this_arg_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(this_arg_tag_local));
        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_builtin_arg_to_locals(1, this_arg_payload_local, this_arg_tag_local, function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(constructor_payload_local));
        for (constructor, _) in typed_array_constructor_bytes_per_element_entries() {
            let constructor_global_index = standard_builtin_constructor_global_index(constructor)
                .ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in lila wasm-aot first slice: missing typed array constructor global `{}`",
                        constructor.debug_name()
                    ))
                })?;
            function.instruction(&Instruction::LocalGet(element_kind_local));
            function.instruction(&Instruction::I64Const(
                typed_array_element_kind(constructor) as i64,
            ));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::GlobalGet(constructor_global_index));
            function.instruction(&Instruction::LocalSet(constructor_payload_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(constructor_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "TypedArray.prototype.map has unknown element type",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(constructor_tag_local));

        function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
        function.instruction(&Instruction::LocalSet(constructor_key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            constructor_key_local,
            constructor_property_payload_local,
            constructor_property_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(constructor_property_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_is_heap_object_like_tag_i32(constructor_property_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "TypedArray.prototype.map constructor property is not an object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(
            self.strings.property_key_symbol_payload("Symbol.species"),
        ));
        function.instruction(&Instruction::LocalSet(constructor_key_local));
        self.emit_object_read(
            constructor_property_payload_local,
            constructor_property_tag_local,
            constructor_property_payload_local,
            constructor_property_tag_local,
            constructor_key_local,
            species_payload_local,
            species_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.compile_nullish_tagged_i32(species_tag_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_is_constructor_i32(species_tag_local, species_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "TypedArray.prototype.map species is not a constructor",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(species_payload_local));
        function.instruction(&Instruction::LocalSet(constructor_payload_local));
        function.instruction(&Instruction::LocalGet(species_tag_local));
        function.instruction(&Instruction::LocalSet(constructor_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(length_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(length_tag_local));
        self.emit_pre_evaluated_arg_vector(
            &[(length_payload_local, length_tag_local)],
            argc_local,
            argv_local,
            function,
        )?;
        self.emit_function_or_proxy_construct_with_argv(
            constructor_payload_local,
            constructor_tag_local,
            constructor_payload_local,
            constructor_tag_local,
            argc_local,
            argv_local,
            target_payload_local,
            target_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_validate_typed_array_from_constructed_target(
            target_payload_local,
            target_tag_local,
            length_payload_local,
            function,
        )?;
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET,
            target_element_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(target_element_kind_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I32Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "TypedArray.prototype.map species content type differs",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        self.emit_typed_array_or_object_index_read_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_function_or_proxy_call_leave_throw_completion(
            callback_payload_local,
            callback_tag_local,
            this_arg_payload_local,
            this_arg_tag_local,
            &[
                (element_payload_local, element_tag_local),
                (index_number_payload_local, length_tag_local),
                (receiver_payload_local, receiver_tag_local),
            ],
            mapped_payload_local,
            mapped_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_typed_array_element_write_from_locals(
            target_payload_local,
            index_local,
            mapped_payload_local,
            mapped_tag_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(mapped_tag_local);
        self.release_temp_local(mapped_payload_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(index_local);
        self.release_temp_local(target_element_kind_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(length_tag_local);
        self.release_temp_local(length_payload_local);
        self.release_temp_local(species_tag_local);
        self.release_temp_local(species_payload_local);
        self.release_temp_local(constructor_property_tag_local);
        self.release_temp_local(constructor_property_payload_local);
        self.release_temp_local(constructor_tag_local);
        self.release_temp_local(constructor_payload_local);
        self.release_temp_local(constructor_key_local);
        self.release_temp_local(this_arg_tag_local);
        self.release_temp_local(this_arg_payload_local);
        self.release_temp_local(callback_tag_local);
        self.release_temp_local(callback_payload_local);
        self.release_temp_local(length_local);
        self.release_temp_local(element_kind_local);
        self.release_temp_local(bytes_per_element_local);
        self.release_temp_local(byte_length_local);
        self.release_temp_local(byte_offset_local);
        self.release_temp_local(buffer_payload_local);
        self.release_temp_local(typed_array_brand_local);
        Ok(())
    }

    pub(crate) fn compile_typed_array_prototype_filter_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing TypedArray.prototype.filter receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing TypedArray.prototype.filter receiver tag",
            )
        })?;
        let typed_array_brand_local = self.reserve_temp_local();
        let buffer_payload_local = self.reserve_temp_local();
        let byte_offset_local = self.reserve_temp_local();
        let byte_length_local = self.reserve_temp_local();
        let bytes_per_element_local = self.reserve_temp_local();
        let element_kind_local = self.reserve_temp_local();
        let length_local = self.reserve_temp_local();
        let callback_payload_local = self.reserve_temp_local();
        let callback_tag_local = self.reserve_temp_local();
        let this_arg_payload_local = self.reserve_temp_local();
        let this_arg_tag_local = self.reserve_temp_local();
        let selected_payload_local = self.reserve_temp_local();
        let selected_tag_local = self.reserve_temp_local();
        let selected_count_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let predicate_payload_local = self.reserve_temp_local();
        let predicate_tag_local = self.reserve_temp_local();
        let constructor_key_local = self.reserve_temp_local();
        let constructor_payload_local = self.reserve_temp_local();
        let constructor_tag_local = self.reserve_temp_local();
        let constructor_property_payload_local = self.reserve_temp_local();
        let constructor_property_tag_local = self.reserve_temp_local();
        let species_payload_local = self.reserve_temp_local();
        let species_tag_local = self.reserve_temp_local();
        let selected_length_payload_local = self.reserve_temp_local();
        let number_tag_local = self.reserve_temp_local();
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let target_element_kind_local = self.reserve_temp_local();
        let write_index_local = self.reserve_temp_local();
        let selected_element_payload_local = self.reserve_temp_local();
        let selected_element_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(typed_array_brand_local));
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
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
            "TypedArray.prototype.filter requires a TypedArray",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_load_typed_array_private_state(
            receiver_payload_local,
            buffer_payload_local,
            byte_offset_local,
            byte_length_local,
            bytes_per_element_local,
            function,
        );
        let receiver_view = TypedArrayViewLocals::new(
            receiver_payload_local,
            buffer_payload_local,
            byte_offset_local,
            byte_length_local,
            bytes_per_element_local,
        );
        self.emit_typed_array_witness(
            &receiver_view,
            TypedArrayWitnessUse::ValidatedMethodEntry { length_local },
            function,
        )?;
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET,
            element_kind_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "TypedArray.prototype.filter callback is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_builtin_arg_to_locals(0, callback_payload_local, callback_tag_local, function);
        self.emit_is_callable_i32(callback_tag_local, callback_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "TypedArray.prototype.filter callback is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(this_arg_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(this_arg_tag_local));
        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_builtin_arg_to_locals(1, this_arg_payload_local, this_arg_tag_local, function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(selected_count_local));
        self.emit_alloc_array_payload_with_length(
            selected_count_local,
            selected_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(selected_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(number_tag_local));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        self.emit_typed_array_or_object_index_read_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_function_or_proxy_call_leave_throw_completion(
            callback_payload_local,
            callback_tag_local,
            this_arg_payload_local,
            this_arg_tag_local,
            &[
                (element_payload_local, element_tag_local),
                (index_number_payload_local, number_tag_local),
                (receiver_payload_local, receiver_tag_local),
            ],
            predicate_payload_local,
            predicate_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.compile_truthy_tagged_i32(predicate_tag_local, predicate_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_write(
            selected_payload_local,
            selected_count_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(selected_count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(selected_count_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(constructor_payload_local));
        for (constructor, _) in typed_array_constructor_bytes_per_element_entries() {
            let constructor_global_index = standard_builtin_constructor_global_index(constructor)
                .ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in lila wasm-aot first slice: missing typed array constructor global `{}`",
                        constructor.debug_name()
                    ))
                })?;
            function.instruction(&Instruction::LocalGet(element_kind_local));
            function.instruction(&Instruction::I64Const(
                typed_array_element_kind(constructor) as i64,
            ));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::GlobalGet(constructor_global_index));
            function.instruction(&Instruction::LocalSet(constructor_payload_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(constructor_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "TypedArray.prototype.filter has unknown element type",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(constructor_tag_local));

        function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
        function.instruction(&Instruction::LocalSet(constructor_key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            constructor_key_local,
            constructor_property_payload_local,
            constructor_property_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(constructor_property_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_is_heap_object_like_tag_i32(constructor_property_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "TypedArray.prototype.filter constructor property is not an object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(
            self.strings.property_key_symbol_payload("Symbol.species"),
        ));
        function.instruction(&Instruction::LocalSet(constructor_key_local));
        self.emit_object_read(
            constructor_property_payload_local,
            constructor_property_tag_local,
            constructor_property_payload_local,
            constructor_property_tag_local,
            constructor_key_local,
            species_payload_local,
            species_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.compile_nullish_tagged_i32(species_tag_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_is_constructor_i32(species_tag_local, species_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "TypedArray.prototype.filter species is not a constructor",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(species_payload_local));
        function.instruction(&Instruction::LocalSet(constructor_payload_local));
        function.instruction(&Instruction::LocalGet(species_tag_local));
        function.instruction(&Instruction::LocalSet(constructor_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(selected_count_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(selected_length_payload_local));
        self.emit_pre_evaluated_arg_vector(
            &[(selected_length_payload_local, number_tag_local)],
            argc_local,
            argv_local,
            function,
        )?;
        self.emit_function_or_proxy_construct_with_argv(
            constructor_payload_local,
            constructor_tag_local,
            constructor_payload_local,
            constructor_tag_local,
            argc_local,
            argv_local,
            target_payload_local,
            target_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_validate_typed_array_from_constructed_target(
            target_payload_local,
            target_tag_local,
            selected_length_payload_local,
            function,
        )?;
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET,
            target_element_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(element_kind_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(target_element_kind_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I32Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "TypedArray.prototype.filter species content type differs",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::LocalGet(selected_count_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_index_get(
            selected_payload_local,
            write_index_local,
            selected_payload_local,
            selected_tag_local,
            selected_element_payload_local,
            selected_element_tag_local,
            None,
            function,
        )?;
        self.emit_typed_array_element_write_from_locals(
            target_payload_local,
            write_index_local,
            selected_element_payload_local,
            selected_element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(selected_element_tag_local);
        self.release_temp_local(selected_element_payload_local);
        self.release_temp_local(write_index_local);
        self.release_temp_local(target_element_kind_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(number_tag_local);
        self.release_temp_local(selected_length_payload_local);
        self.release_temp_local(species_tag_local);
        self.release_temp_local(species_payload_local);
        self.release_temp_local(constructor_property_tag_local);
        self.release_temp_local(constructor_property_payload_local);
        self.release_temp_local(constructor_tag_local);
        self.release_temp_local(constructor_payload_local);
        self.release_temp_local(constructor_key_local);
        self.release_temp_local(predicate_tag_local);
        self.release_temp_local(predicate_payload_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(index_local);
        self.release_temp_local(selected_count_local);
        self.release_temp_local(selected_tag_local);
        self.release_temp_local(selected_payload_local);
        self.release_temp_local(this_arg_tag_local);
        self.release_temp_local(this_arg_payload_local);
        self.release_temp_local(callback_tag_local);
        self.release_temp_local(callback_payload_local);
        self.release_temp_local(length_local);
        self.release_temp_local(element_kind_local);
        self.release_temp_local(bytes_per_element_local);
        self.release_temp_local(byte_length_local);
        self.release_temp_local(byte_offset_local);
        self.release_temp_local(buffer_payload_local);
        self.release_temp_local(typed_array_brand_local);
        Ok(())
    }

    pub(crate) fn compile_typed_array_prototype_every_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_typed_array_prototype_quantifier_builtin(
            function,
            TypedArrayQuantifierKind::Every,
        )
    }

    pub(crate) fn compile_typed_array_prototype_some_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_typed_array_prototype_quantifier_builtin(
            function,
            TypedArrayQuantifierKind::Some,
        )
    }

    fn compile_typed_array_prototype_quantifier_builtin(
        &mut self,
        function: &mut Function,
        quantifier: TypedArrayQuantifierKind,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(format!(
                "unsupported in lila wasm-aot first slice: missing {} receiver",
                match &quantifier {
                    TypedArrayQuantifierKind::Every => "TypedArray.prototype.every",
                    TypedArrayQuantifierKind::Some => "TypedArray.prototype.some",
                }
            ))
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(format!(
                "unsupported in lila wasm-aot first slice: missing {} receiver tag",
                match &quantifier {
                    TypedArrayQuantifierKind::Every => "TypedArray.prototype.every",
                    TypedArrayQuantifierKind::Some => "TypedArray.prototype.some",
                }
            ))
        })?;
        let receiver_brand_local = self.reserve_temp_local();
        let receiver_buffer_local = self.reserve_temp_local();
        let receiver_byte_offset_local = self.reserve_temp_local();
        let receiver_byte_length_local = self.reserve_temp_local();
        let receiver_bytes_per_element_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let index_payload_local = self.reserve_temp_local();
        let index_tag_local = self.reserve_temp_local();
        let callback_payload_local = self.reserve_temp_local();
        let callback_tag_local = self.reserve_temp_local();
        let this_arg_payload_local = self.reserve_temp_local();
        let this_arg_tag_local = self.reserve_temp_local();
        let callback_result_payload_local = self.reserve_temp_local();
        let callback_result_tag_local = self.reserve_temp_local();
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(receiver_brand_local));
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            receiver_brand_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(receiver_brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TYPED_ARRAY as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            match &quantifier {
                TypedArrayQuantifierKind::Every => "TypedArray every method requires a TypedArray",
                TypedArrayQuantifierKind::Some => "TypedArray some method requires a TypedArray",
            },
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_load_typed_array_private_state(
            receiver_payload_local,
            receiver_buffer_local,
            receiver_byte_offset_local,
            receiver_byte_length_local,
            receiver_bytes_per_element_local,
            function,
        );
        let receiver_view = TypedArrayViewLocals::new(
            receiver_payload_local,
            receiver_buffer_local,
            receiver_byte_offset_local,
            receiver_byte_length_local,
            receiver_bytes_per_element_local,
        );
        self.emit_typed_array_witness(
            &receiver_view,
            TypedArrayWitnessUse::ValidatedMethodEntry {
                length_local: len_local,
            },
            function,
        )?;

        self.emit_builtin_arg_to_locals(0, callback_payload_local, callback_tag_local, function);
        self.emit_is_callable_i32(callback_tag_local, callback_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            match &quantifier {
                TypedArrayQuantifierKind::Every => {
                    "TypedArray.prototype.every callback is not callable"
                }
                TypedArrayQuantifierKind::Some => {
                    "TypedArray.prototype.some callback is not callable"
                }
            },
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(this_arg_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(this_arg_tag_local));
        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_builtin_arg_to_locals(1, this_arg_payload_local, this_arg_tag_local, function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        self.emit_typed_array_or_object_index_read_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(index_tag_local));
        self.emit_pre_evaluated_arg_vector(
            &[
                (element_payload_local, element_tag_local),
                (index_payload_local, index_tag_local),
                (receiver_payload_local, receiver_tag_local),
            ],
            argc_local,
            argv_local,
            function,
        )?;
        self.emit_function_or_proxy_call_with_argv_leave_throw_completion(
            callback_payload_local,
            callback_tag_local,
            this_arg_payload_local,
            this_arg_tag_local,
            argc_local,
            argv_local,
            callback_result_payload_local,
            callback_result_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            callback_result_payload_local,
            callback_result_tag_local,
            function,
        )?;
        self.compile_truthy_tagged_i32(
            callback_result_tag_local,
            callback_result_payload_local,
            function,
        )?;
        match &quantifier {
            TypedArrayQuantifierKind::Every => {
                function.instruction(&Instruction::I32Eqz);
            }
            TypedArrayQuantifierKind::Some => {}
        }
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(match &quantifier {
            TypedArrayQuantifierKind::Every => 0,
            TypedArrayQuantifierKind::Some => 1,
        }));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(match &quantifier {
            TypedArrayQuantifierKind::Every => 1,
            TypedArrayQuantifierKind::Some => 0,
        }));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(callback_result_tag_local);
        self.release_temp_local(callback_result_payload_local);
        self.release_temp_local(this_arg_tag_local);
        self.release_temp_local(this_arg_payload_local);
        self.release_temp_local(callback_tag_local);
        self.release_temp_local(callback_payload_local);
        self.release_temp_local(index_tag_local);
        self.release_temp_local(index_payload_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(receiver_bytes_per_element_local);
        self.release_temp_local(receiver_byte_length_local);
        self.release_temp_local(receiver_byte_offset_local);
        self.release_temp_local(receiver_buffer_local);
        self.release_temp_local(receiver_brand_local);
        Ok(())
    }

    pub(crate) fn compile_array_prototype_every_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_array_callback_iteration(function, ArrayCallbackIterationKind::Every)
    }

    pub(crate) fn compile_array_prototype_some_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_array_callback_iteration(function, ArrayCallbackIterationKind::Some)
    }

    pub(crate) fn compile_array_prototype_filter_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_array_callback_iteration(function, ArrayCallbackIterationKind::Filter)
    }

    pub(crate) fn emit_array_direct_builtin_method_call(
        &mut self,
        builtin: StandardBuiltinId,
        label: &str,
        receiver: &TypedExpr,
        args: &[TypedExpr],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let meta = self
            .functions
            .get(&builtin.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `{label}`"
                ))
            })?;
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();

        self.compile_expr_to_locals(
            receiver,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        let (argc_local, argv_local) = self.emit_call_args_vector(args, function)?;
        self.emit_direct_js_call_with_argv(
            &meta,
            Some((receiver_payload_local, Some(receiver_tag_local))),
            argc_local,
            argv_local,
            payload_local,
            tag_local,
            function,
        )?;

        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    pub(crate) fn compile_array_prototype_join_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let this_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Array.prototype.join receiver",
            )
        })?;
        let this_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Array.prototype.join receiver tag",
            )
        })?;
        let separator_payload_local = self.reserve_temp_local();
        let separator_tag_local = self.reserve_temp_local();
        self.emit_builtin_arg_to_locals(0, separator_payload_local, separator_tag_local, function);
        self.emit_array_join_generic_from_locals(
            this_payload_local,
            this_tag_local,
            separator_payload_local,
            separator_tag_local,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.release_temp_local(separator_tag_local);
        self.release_temp_local(separator_payload_local);
        Ok(())
    }

    pub(crate) fn compile_typed_array_prototype_join_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing TypedArray.prototype.join receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing TypedArray.prototype.join receiver tag",
            )
        })?;
        let typed_brand_local = self.reserve_temp_local();
        let buffer_payload_local = self.reserve_temp_local();
        let byte_offset_local = self.reserve_temp_local();
        let stored_byte_length_local = self.reserve_temp_local();
        let bytes_per_element_local = self.reserve_temp_local();
        let length_local = self.reserve_temp_local();
        let separator_payload_local = self.reserve_temp_local();
        let separator_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(typed_brand_local));
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            typed_brand_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(typed_brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TYPED_ARRAY as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "TypedArray.prototype.join requires a TypedArray",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_load_typed_array_private_state(
            receiver_payload_local,
            buffer_payload_local,
            byte_offset_local,
            stored_byte_length_local,
            bytes_per_element_local,
            function,
        );
        let view = TypedArrayViewLocals::new(
            receiver_payload_local,
            buffer_payload_local,
            byte_offset_local,
            stored_byte_length_local,
            bytes_per_element_local,
        );
        self.emit_typed_array_witness(
            &view,
            TypedArrayWitnessUse::ValidatedMethodEntry { length_local },
            function,
        )?;

        self.emit_builtin_arg_to_locals(0, separator_payload_local, separator_tag_local, function);
        self.emit_array_join_with_length_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            separator_payload_local,
            separator_tag_local,
            length_local,
            self.result_local,
            self.result_tag_local,
            function,
        )?;

        self.release_temp_local(separator_tag_local);
        self.release_temp_local(separator_payload_local);
        self.release_temp_local(length_local);
        self.release_temp_local(bytes_per_element_local);
        self.release_temp_local(stored_byte_length_local);
        self.release_temp_local(byte_offset_local);
        self.release_temp_local(buffer_payload_local);
        self.release_temp_local(typed_brand_local);
        Ok(())
    }

    fn emit_array_join_generic_from_locals(
        &mut self,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        separator_payload_local: u32,
        separator_tag_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let len_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();

        // Array.prototype.join is generic: ToObject and LengthOfArrayLike both
        // precede separator coercion, and the length is captured exactly once.
        self.emit_value_to_current_function_realm_object_locals(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_to_length_i64_from_value_locals(
            element_tag_local,
            element_payload_local,
            len_local,
            function,
        )?;

        self.emit_array_join_with_length_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            separator_payload_local,
            separator_tag_local,
            len_local,
            payload_local,
            tag_local,
            function,
        )?;

        self.release_temp_local(key_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(len_local);
        Ok(())
    }

    fn emit_array_join_with_length_from_locals(
        &mut self,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        separator_payload_local: u32,
        separator_tag_local: u32,
        len_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let index_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let element_string_local = self.reserve_temp_local();
        let joined_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(joined_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(separator_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload(",")));
        function.instruction(&Instruction::LocalSet(separator_payload_local));
        function.instruction(&Instruction::Else);
        self.emit_value_to_string_payload(separator_payload_local, separator_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(separator_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(0));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_concat_string_payloads_local(joined_local, separator_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(joined_local));
        function.instruction(&Instruction::End);

        self.emit_typed_array_or_object_index_read_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.compile_nullish_tagged_i32(element_tag_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_string_payload(element_payload_local, element_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(element_string_local));
        self.emit_return_current_completion_if_throw(function);
        self.emit_concat_string_payloads_local(joined_local, element_string_local, function)?;
        function.instruction(&Instruction::LocalSet(joined_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(joined_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_NORMAL));
        function.instruction(&Instruction::LocalSet(self.completion_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.completion_aux_local));

        self.release_temp_local(joined_local);
        self.release_temp_local(element_string_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(index_local);
        Ok(())
    }

    pub(crate) fn emit_array_species_create(
        &mut self,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        length_local: u32,
        target_payload_local: u32,
        target_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let array_constructor_table_index = self
            .functions
            .get(&StandardBuiltinId::ArrayConstructor.function_id())
            .map(|meta| meta.table_index as i64)
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array`",
                )
            })?;
        let current_payload_local = self.reserve_temp_local();
        let current_tag_local = self.reserve_temp_local();
        let proxy_kind_local = self.reserve_temp_local();
        let is_array_local = self.reserve_temp_local();
        let constructor_payload_local = self.reserve_temp_local();
        let constructor_tag_local = self.reserve_temp_local();
        let constructor_table_index_local = self.reserve_temp_local();
        let skip_species_local = self.reserve_temp_local();
        let species_payload_local = self.reserve_temp_local();
        let species_tag_local = self.reserve_temp_local();
        let is_constructor_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let length_payload_local = self.reserve_temp_local();
        let number_tag_local = self.reserve_temp_local();
        let realm_local = self.reserve_temp_local();
        let prototype_local = self.reserve_temp_local();
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(receiver_payload_local));
        function.instruction(&Instruction::LocalSet(current_payload_local));
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::LocalSet(current_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(is_array_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(current_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(is_array_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(current_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            proxy_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(proxy_kind_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(proxy_kind_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Proxy handler is null",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            current_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            current_payload_local,
            function,
        );
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(species_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(species_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(skip_species_local));

        function.instruction(&Instruction::LocalGet(is_array_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_constructor_read(
            receiver_payload_local,
            constructor_payload_local,
            constructor_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            constructor_payload_local,
            constructor_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_propagate_throw_from_locals_if_needed(
            constructor_payload_local,
            constructor_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(constructor_payload_local));
        function.instruction(&Instruction::LocalSet(species_payload_local));
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::LocalSet(species_tag_local));

        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            constructor_payload_local,
            HEAP_FUNCTION_TABLE_INDEX_OFFSET,
            constructor_table_index_local,
            function,
        );
        self.emit_mark_skip_species_for_cross_realm_array_constructor(
            constructor_payload_local,
            constructor_table_index_local,
            skip_species_local,
            array_constructor_table_index,
            function,
        );
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(skip_species_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_is_heap_object_like_tag_i32(constructor_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.property_key_symbol_payload("Symbol.species"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            constructor_payload_local,
            constructor_tag_local,
            constructor_payload_local,
            constructor_tag_local,
            key_local,
            species_payload_local,
            species_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            species_payload_local,
            species_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(species_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(species_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(species_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(species_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(species_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(species_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::I64Const(MAX_ARRAY_LENGTH as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Invalid array length",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_alloc_array_payload_with_length(length_local, target_payload_local, function)?;
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(target_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(realm_local));
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            self.current_env_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            realm_local,
            function,
        );
        function.instruction(&Instruction::End);
        self.emit_load_realm_intrinsic_prototype_or_global(
            realm_local,
            HEAP_REALM_INTRINSICS_ARRAY_PROTOTYPE_OFFSET,
            ARRAY_PROTOTYPE_GLOBAL_INDEX,
            prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            target_payload_local,
            HEAP_PROTOTYPE_OFFSET,
            prototype_local,
            function,
        );
        self.store_i64_const_at_offset(
            target_payload_local,
            HEAP_ARRAY_PROTOTYPE_TAG_OFFSET,
            ValueKind::Array.tag() as u64,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_is_constructor_i32(species_tag_local, species_payload_local, function)?;
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(is_constructor_local));
        function.instruction(&Instruction::LocalGet(is_constructor_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Array species constructor is not a constructor",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(length_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(number_tag_local));
        self.emit_pre_evaluated_arg_vector(
            &[(length_payload_local, number_tag_local)],
            argc_local,
            argv_local,
            function,
        )?;
        self.emit_function_or_proxy_construct_with_argv(
            species_payload_local,
            species_tag_local,
            species_payload_local,
            species_tag_local,
            argc_local,
            argv_local,
            target_payload_local,
            target_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            target_payload_local,
            target_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(prototype_local);
        self.release_temp_local(realm_local);
        self.release_temp_local(number_tag_local);
        self.release_temp_local(length_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(is_constructor_local);
        self.release_temp_local(species_tag_local);
        self.release_temp_local(species_payload_local);
        self.release_temp_local(skip_species_local);
        self.release_temp_local(constructor_table_index_local);
        self.release_temp_local(constructor_tag_local);
        self.release_temp_local(constructor_payload_local);
        self.release_temp_local(is_array_local);
        self.release_temp_local(proxy_kind_local);
        self.release_temp_local(current_tag_local);
        self.release_temp_local(current_payload_local);
        Ok(())
    }

    fn emit_delete_property_or_throw(
        &mut self,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        key_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let deleted_local = self.reserve_temp_local();
        self.emit_object_delete(
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            deleted_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(deleted_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Cannot delete property",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.release_temp_local(deleted_local);
        Ok(())
    }

    pub(super) fn compile_array_prototype_sort_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_array_sort_with_output(ArraySortOutput::Receiver, function)
    }

    pub(super) fn compile_array_prototype_to_sorted_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_array_sort_with_output(ArraySortOutput::Copy, function)
    }

    fn compile_array_sort_with_output(
        &mut self,
        output: ArraySortOutput,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let this_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Array.prototype.sort receiver",
            )
        })?;
        let this_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Array.prototype.sort receiver tag",
            )
        })?;
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let length_payload_local = self.reserve_temp_local();
        let length_tag_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let compare_payload_local = self.reserve_temp_local();
        let compare_tag_local = self.reserve_temp_local();
        let has_compare_local = self.reserve_temp_local();
        let undefined_this_payload_local = self.reserve_temp_local();
        let undefined_this_tag_local = self.reserve_temp_local();
        let source_index_local = self.reserve_temp_local();
        let source_number_payload_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let has_property_local = self.reserve_temp_local();
        let typed_array_byte_length_payload_local = self.reserve_temp_local();
        let typed_array_byte_length_tag_local = self.reserve_temp_local();
        let receiver_is_typed_array_local = self.reserve_temp_local();
        let collected_payload_local = self.reserve_temp_local();
        let collected_tag_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let buffer_len_local = self.reserve_temp_local();
        let buffer_cap_local = self.reserve_temp_local();
        let new_buffer_local = self.reserve_temp_local();
        let new_buffer_cap_local = self.reserve_temp_local();
        let new_buffer_size_local = self.reserve_temp_local();
        let copy_index_local = self.reserve_temp_local();
        let source_entry_local = self.reserve_temp_local();
        let destination_entry_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let sort_index_local = self.reserve_temp_local();
        let previous_index_local = self.reserve_temp_local();
        let preceding_index_local = self.reserve_temp_local();
        let previous_entry_local = self.reserve_temp_local();
        let sort_key_payload_local = self.reserve_temp_local();
        let sort_key_tag_local = self.reserve_temp_local();
        let previous_payload_local = self.reserve_temp_local();
        let previous_tag_local = self.reserve_temp_local();
        let should_shift_local = self.reserve_temp_local();
        let compare_result_payload_local = self.reserve_temp_local();
        let compare_result_tag_local = self.reserve_temp_local();
        let compare_number_payload_local = self.reserve_temp_local();
        let key_string_payload_local = self.reserve_temp_local();
        let previous_string_payload_local = self.reserve_temp_local();
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, compare_payload_local, compare_tag_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(has_compare_local));
        function.instruction(&Instruction::LocalGet(compare_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_is_callable_i32(compare_tag_local, compare_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(has_compare_local));
        function.instruction(&Instruction::Else);
        self.emit_throw_current_function_realm_type_error(
            "value is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(this_payload_local));
        function.instruction(&Instruction::LocalSet(receiver_payload_local));
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::LocalSet(receiver_tag_local));
        self.emit_value_to_current_function_realm_object_locals(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;

        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            length_payload_local,
            length_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            length_payload_local,
            length_tag_local,
            function,
        )?;
        self.emit_to_length_i64_from_value_locals(
            length_tag_local,
            length_payload_local,
            len_local,
            function,
        )?;

        match &output {
            ArraySortOutput::Copy => {
                function.instruction(&Instruction::LocalGet(len_local));
                function.instruction(&Instruction::I64Const(u32::MAX as i64));
                function.instruction(&Instruction::I64GtU);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_throw_current_function_realm_range_error(
                    "Invalid array length",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_return_current_completion(function);
                function.instruction(&Instruction::End);
                self.emit_alloc_array_payload_with_length(
                    len_local,
                    target_payload_local,
                    function,
                )?;
                function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
                function.instruction(&Instruction::LocalSet(target_tag_local));
            }
            ArraySortOutput::Receiver => {
                function.instruction(&Instruction::LocalGet(receiver_payload_local));
                function.instruction(&Instruction::LocalSet(target_payload_local));
                function.instruction(&Instruction::LocalGet(receiver_tag_local));
                function.instruction(&Instruction::LocalSet(target_tag_local));
            }
        }

        self.emit_is_typed_array_i32(receiver_payload_local, receiver_tag_local, function);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(receiver_is_typed_array_local));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_this_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_this_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(buffer_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(buffer_len_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(buffer_cap_local));

        // Collect only present values. The private payload/tag buffer grows by
        // the number found, never by the possibly huge array-like length.
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(source_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(source_index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_index_to_flat_map_key_local(
            source_index_local,
            source_number_payload_local,
            key_local,
            function,
        )?;
        match &output {
            ArraySortOutput::Copy => {
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(has_property_local));
            }
            ArraySortOutput::Receiver => {
                self.emit_object_has_property_i32(
                    receiver_payload_local,
                    receiver_tag_local,
                    key_local,
                    has_property_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(receiver_is_typed_array_local));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(has_property_local));
                function.instruction(&Instruction::End);
            }
        }
        function.instruction(&Instruction::LocalGet(has_property_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(receiver_is_typed_array_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            collected_payload_local,
            collected_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_typed_array_or_object_index_read_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            source_index_local,
            collected_payload_local,
            collected_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_propagate_throw_from_locals_if_needed(
            collected_payload_local,
            collected_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(buffer_len_local));
        function.instruction(&Instruction::LocalGet(buffer_cap_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(buffer_cap_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(buffer_cap_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(new_buffer_cap_local));
        function.instruction(&Instruction::LocalGet(new_buffer_cap_local));
        function.instruction(&Instruction::I64Const(16));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(new_buffer_size_local));
        self.emit_heap_alloc_from_local(new_buffer_size_local, function)?;
        function.instruction(&Instruction::LocalSet(new_buffer_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(copy_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(copy_index_local));
        function.instruction(&Instruction::LocalGet(buffer_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(copy_index_local));
        function.instruction(&Instruction::I64Const(16));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(source_entry_local));
        function.instruction(&Instruction::LocalGet(new_buffer_local));
        function.instruction(&Instruction::LocalGet(copy_index_local));
        function.instruction(&Instruction::I64Const(16));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(destination_entry_local));
        self.load_i64_from_offset(source_entry_local, 0, function);
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.store_i64_local_at_offset(destination_entry_local, 0, self.scratch_local, function);
        self.load_i64_from_offset(source_entry_local, 8, function);
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.store_i64_local_at_offset(destination_entry_local, 8, self.scratch_local, function);
        function.instruction(&Instruction::LocalGet(copy_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(copy_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(new_buffer_local));
        function.instruction(&Instruction::LocalSet(buffer_local));
        function.instruction(&Instruction::LocalGet(new_buffer_cap_local));
        function.instruction(&Instruction::LocalSet(buffer_cap_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(buffer_len_local));
        function.instruction(&Instruction::I64Const(16));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_local_at_offset(entry_local, 0, collected_tag_local, function);
        self.store_i64_local_at_offset(entry_local, 8, collected_payload_local, function);
        function.instruction(&Instruction::LocalGet(buffer_len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(buffer_len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(source_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(source_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        // Insertion sort is stable because a key moves only when it compares
        // strictly before the preceding value.
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(sort_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(sort_index_local));
        function.instruction(&Instruction::LocalGet(buffer_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(sort_index_local));
        function.instruction(&Instruction::I64Const(16));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(entry_local, 0, sort_key_tag_local, function);
        self.load_i64_to_local_from_offset(entry_local, 8, sort_key_payload_local, function);
        function.instruction(&Instruction::LocalGet(sort_index_local));
        function.instruction(&Instruction::LocalSet(previous_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(previous_index_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(previous_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(preceding_index_local));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(preceding_index_local));
        function.instruction(&Instruction::I64Const(16));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(previous_entry_local));
        self.load_i64_to_local_from_offset(previous_entry_local, 0, previous_tag_local, function);
        self.load_i64_to_local_from_offset(
            previous_entry_local,
            8,
            previous_payload_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(should_shift_local));
        function.instruction(&Instruction::LocalGet(sort_key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(previous_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(should_shift_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(has_compare_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_string_payload(sort_key_payload_local, sort_key_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(key_string_payload_local));
        self.emit_return_current_completion_if_throw(function);
        self.emit_value_to_string_payload(previous_payload_local, previous_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(previous_string_payload_local));
        self.emit_return_current_completion_if_throw(function);
        self.emit_string_payload_utf16_compare_i32(
            key_string_payload_local,
            previous_string_payload_local,
            function,
        );
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::I32LtS);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(should_shift_local));
        function.instruction(&Instruction::Else);
        self.emit_function_or_proxy_call_leave_throw_completion(
            compare_payload_local,
            compare_tag_local,
            undefined_this_payload_local,
            undefined_this_tag_local,
            &[
                (sort_key_payload_local, sort_key_tag_local),
                (previous_payload_local, previous_tag_local),
            ],
            compare_result_payload_local,
            compare_result_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            compare_result_payload_local,
            compare_result_tag_local,
            function,
        )?;
        self.emit_value_to_number_payload(
            compare_result_tag_local,
            compare_result_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(compare_number_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(compare_number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(should_shift_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(should_shift_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(previous_index_local));
        function.instruction(&Instruction::I64Const(16));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_local_at_offset(entry_local, 0, previous_tag_local, function);
        self.store_i64_local_at_offset(entry_local, 8, previous_payload_local, function);
        function.instruction(&Instruction::LocalGet(preceding_index_local));
        function.instruction(&Instruction::LocalSet(previous_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(previous_index_local));
        function.instruction(&Instruction::I64Const(16));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_local_at_offset(entry_local, 0, sort_key_tag_local, function);
        self.store_i64_local_at_offset(entry_local, 8, sort_key_payload_local, function);
        function.instruction(&Instruction::LocalGet(sort_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(sort_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(source_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(source_index_local));
        function.instruction(&Instruction::LocalGet(buffer_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(source_index_local));
        function.instruction(&Instruction::I64Const(16));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(entry_local, 0, collected_tag_local, function);
        self.load_i64_to_local_from_offset(entry_local, 8, collected_payload_local, function);
        self.emit_index_to_flat_map_key_local(
            source_index_local,
            source_number_payload_local,
            key_local,
            function,
        )?;
        match &output {
            ArraySortOutput::Copy => {
                self.emit_array_write(
                    target_payload_local,
                    source_index_local,
                    collected_payload_local,
                    collected_tag_local,
                    function,
                )?;
            }
            ArraySortOutput::Receiver => {
                function.instruction(&Instruction::LocalGet(receiver_is_typed_array_local));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_object_write_strict(
                    receiver_payload_local,
                    receiver_tag_local,
                    key_local,
                    collected_payload_local,
                    collected_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::Else);
                self.emit_typed_array_element_write_from_locals(
                    receiver_payload_local,
                    source_index_local,
                    collected_payload_local,
                    collected_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::End);
            }
        }
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(source_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(source_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        match &output {
            ArraySortOutput::Receiver => {
                function.instruction(&Instruction::LocalGet(buffer_len_local));
                function.instruction(&Instruction::LocalSet(source_index_local));
                function.instruction(&Instruction::Block(BlockType::Empty));
                function.instruction(&Instruction::Loop(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(source_index_local));
                function.instruction(&Instruction::LocalGet(len_local));
                function.instruction(&Instruction::I64GeU);
                function.instruction(&Instruction::BrIf(1));
                self.emit_index_to_flat_map_key_local(
                    source_index_local,
                    source_number_payload_local,
                    key_local,
                    function,
                )?;
                self.emit_delete_property_or_throw(
                    receiver_payload_local,
                    receiver_tag_local,
                    key_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(source_index_local));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::LocalSet(source_index_local));
                function.instruction(&Instruction::Br(0));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
            }
            ArraySortOutput::Copy => {}
        }

        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);

        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        self.release_temp_local(previous_string_payload_local);
        self.release_temp_local(key_string_payload_local);
        self.release_temp_local(compare_number_payload_local);
        self.release_temp_local(compare_result_tag_local);
        self.release_temp_local(compare_result_payload_local);
        self.release_temp_local(should_shift_local);
        self.release_temp_local(previous_tag_local);
        self.release_temp_local(previous_payload_local);
        self.release_temp_local(sort_key_tag_local);
        self.release_temp_local(sort_key_payload_local);
        self.release_temp_local(previous_entry_local);
        self.release_temp_local(preceding_index_local);
        self.release_temp_local(previous_index_local);
        self.release_temp_local(sort_index_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(destination_entry_local);
        self.release_temp_local(source_entry_local);
        self.release_temp_local(copy_index_local);
        self.release_temp_local(new_buffer_size_local);
        self.release_temp_local(new_buffer_cap_local);
        self.release_temp_local(new_buffer_local);
        self.release_temp_local(buffer_cap_local);
        self.release_temp_local(buffer_len_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(collected_tag_local);
        self.release_temp_local(collected_payload_local);
        self.release_temp_local(receiver_is_typed_array_local);
        self.release_temp_local(typed_array_byte_length_tag_local);
        self.release_temp_local(typed_array_byte_length_payload_local);
        self.release_temp_local(has_property_local);
        self.release_temp_local(key_local);
        self.release_temp_local(source_number_payload_local);
        self.release_temp_local(source_index_local);
        self.release_temp_local(undefined_this_tag_local);
        self.release_temp_local(undefined_this_payload_local);
        self.release_temp_local(has_compare_local);
        self.release_temp_local(compare_tag_local);
        self.release_temp_local(compare_payload_local);
        self.release_temp_local(len_local);
        self.release_temp_local(length_tag_local);
        self.release_temp_local(length_payload_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_array_target_create_data_property_or_throw(
        &mut self,
        target_payload_local: u32,
        target_tag_local: u32,
        key_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        let descriptor_payload_local = self.reserve_temp_local();
        let descriptor_tag_local = self.reserve_temp_local();
        let bool_payload_local = self.reserve_temp_local();
        let bool_tag_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let define_property_payload_local = self.reserve_temp_local();
        let define_property_tag_local = self.reserve_temp_local();
        let call_payload_local = self.reserve_temp_local();
        let call_tag_local = self.reserve_temp_local();
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(descriptor_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(descriptor_tag_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(bool_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(bool_tag_local));
        for (name, payload, tag) in [
            ("value", value_payload_local, value_tag_local),
            ("writable", bool_payload_local, bool_tag_local),
            ("enumerable", bool_payload_local, bool_tag_local),
            ("configurable", bool_payload_local, bool_tag_local),
        ] {
            function.instruction(&Instruction::I64Const(self.strings.payload(name)));
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.emit_object_define_data(
                descriptor_payload_local,
                self.scratch_local,
                payload,
                tag,
                function,
            )?;
        }
        let define_property_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectDefineProperty.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Object.defineProperty`",
                )
            })?;
        self.emit_function_value_payload(&define_property_meta, function)?;
        function.instruction(&Instruction::LocalSet(define_property_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(define_property_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        self.emit_function_handle_call(
            define_property_payload_local,
            define_property_tag_local,
            None,
            &[
                (target_payload_local, target_tag_local),
                (key_local, key_tag_local),
                (descriptor_payload_local, descriptor_tag_local),
            ],
            call_payload_local,
            call_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            call_payload_local,
            call_tag_local,
            function,
        )?;
        self.release_temp_local(call_tag_local);
        self.release_temp_local(call_payload_local);
        self.release_temp_local(define_property_tag_local);
        self.release_temp_local(define_property_payload_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(bool_tag_local);
        self.release_temp_local(bool_payload_local);
        self.release_temp_local(descriptor_tag_local);
        self.release_temp_local(descriptor_payload_local);
        function.instruction(&Instruction::Else);
        self.emit_create_data_property_or_throw(
            target_payload_local,
            target_tag_local,
            key_local,
            value_payload_local,
            value_tag_local,
            "Cannot define non-configurable target property",
            "Cannot add property to non-extensible target",
            None,
            function,
        )?;
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn compile_array_prototype_fill_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let this_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Array.prototype.fill receiver",
            )
        })?;
        let this_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Array.prototype.fill receiver tag",
            )
        })?;
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let length_payload_local = self.reserve_temp_local();
        let length_tag_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let start_payload_local = self.reserve_temp_local();
        let start_tag_local = self.reserve_temp_local();
        let end_payload_local = self.reserve_temp_local();
        let end_tag_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let relative_start_local = self.reserve_temp_local();
        let relative_end_local = self.reserve_temp_local();
        let start_local = self.reserve_temp_local();
        let end_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let number_payload_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let receiver_is_typed_array_local = self.reserve_temp_local();
        let receiver_brand_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(this_payload_local));
        function.instruction(&Instruction::LocalSet(receiver_payload_local));
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::LocalSet(receiver_tag_local));
        self.emit_value_to_current_function_realm_object_locals(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(receiver_is_typed_array_local));
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            receiver_brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(receiver_brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TYPED_ARRAY as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(receiver_is_typed_array_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            length_payload_local,
            length_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            length_payload_local,
            length_tag_local,
            function,
        )?;
        self.emit_to_length_i64_from_value_locals(
            length_tag_local,
            length_payload_local,
            len_local,
            function,
        )?;

        self.emit_builtin_arg_to_locals(0, value_payload_local, value_tag_local, function);
        self.emit_builtin_arg_to_locals(1, start_payload_local, start_tag_local, function);
        self.emit_value_to_number_payload(start_tag_local, start_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(start_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(start_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncSatF64S);
        function.instruction(&Instruction::LocalSet(relative_start_local));
        function.instruction(&Instruction::LocalGet(relative_start_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalGet(relative_start_local));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(start_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalGet(relative_start_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(start_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalSet(start_local));
        function.instruction(&Instruction::LocalGet(relative_start_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(relative_start_local));
        function.instruction(&Instruction::LocalSet(start_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalSet(end_local));
        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_builtin_arg_to_locals(2, end_payload_local, end_tag_local, function);
        function.instruction(&Instruction::LocalGet(end_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_number_payload(end_tag_local, end_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(end_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(end_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncSatF64S);
        function.instruction(&Instruction::LocalSet(relative_end_local));
        function.instruction(&Instruction::LocalGet(relative_end_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalGet(relative_end_local));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(end_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalGet(relative_end_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(end_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalSet(end_local));
        function.instruction(&Instruction::LocalGet(relative_end_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(relative_end_local));
        function.instruction(&Instruction::LocalSet(end_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_index_to_flat_map_key_local(
            index_local,
            number_payload_local,
            key_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(receiver_is_typed_array_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_write(
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_typed_array_element_write_from_locals(
            receiver_payload_local,
            index_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(receiver_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(receiver_brand_local);
        self.release_temp_local(receiver_is_typed_array_local);
        self.release_temp_local(key_local);
        self.release_temp_local(number_payload_local);
        self.release_temp_local(index_local);
        self.release_temp_local(end_local);
        self.release_temp_local(start_local);
        self.release_temp_local(relative_end_local);
        self.release_temp_local(relative_start_local);
        self.release_temp_local(len_local);
        self.release_temp_local(end_tag_local);
        self.release_temp_local(end_payload_local);
        self.release_temp_local(start_tag_local);
        self.release_temp_local(start_payload_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(length_tag_local);
        self.release_temp_local(length_payload_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    pub(crate) fn emit_array_slice_clamped_index(
        &mut self,
        relative_index_payload_local: u32,
        len_local: u32,
        index_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(relative_index_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(relative_index_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::F64Neg);
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalGet(relative_index_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64S);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(relative_index_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::F64Ge);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(relative_index_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    pub(crate) fn compile_array_prototype_slice_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let this_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Array.prototype.slice receiver",
            )
        })?;
        let this_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Array.prototype.slice receiver tag",
            )
        })?;
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let length_payload_local = self.reserve_temp_local();
        let length_tag_local = self.reserve_temp_local();
        let start_payload_local = self.reserve_temp_local();
        let start_tag_local = self.reserve_temp_local();
        let end_payload_local = self.reserve_temp_local();
        let end_tag_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let first_local = self.reserve_temp_local();
        let final_local = self.reserve_temp_local();
        let count_local = self.reserve_temp_local();
        let source_index_local = self.reserve_temp_local();
        let target_index_local = self.reserve_temp_local();
        let source_key_local = self.reserve_temp_local();
        let target_key_local = self.reserve_temp_local();
        let source_number_payload_local = self.reserve_temp_local();
        let target_number_payload_local = self.reserve_temp_local();
        let has_property_local = self.reserve_temp_local();
        let typed_array_like_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let target_length_payload_local = self.reserve_temp_local();
        let number_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(this_payload_local));
        function.instruction(&Instruction::LocalSet(receiver_payload_local));
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::LocalSet(receiver_tag_local));
        self.emit_value_to_current_function_realm_object_locals(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;

        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(source_key_local));
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_length(
            receiver_payload_local,
            length_payload_local,
            length_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            source_key_local,
            length_payload_local,
            length_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_propagate_throw_from_locals_if_needed(
            length_payload_local,
            length_tag_local,
            function,
        )?;
        self.emit_to_length_i64_from_value_locals(
            length_tag_local,
            length_payload_local,
            len_local,
            function,
        )?;

        self.emit_builtin_arg_to_locals(0, start_payload_local, start_tag_local, function);
        self.emit_value_to_number_payload(start_tag_local, start_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(start_payload_local));
        self.emit_return_current_completion_if_throw(function);
        self.emit_to_integer_or_infinity_number_payload_from_number_payload(
            start_payload_local,
            start_payload_local,
            function,
        );
        self.emit_array_slice_clamped_index(start_payload_local, len_local, first_local, function);

        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalSet(final_local));
        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_builtin_arg_to_locals(1, end_payload_local, end_tag_local, function);
        function.instruction(&Instruction::LocalGet(end_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_number_payload(end_tag_local, end_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(end_payload_local));
        self.emit_return_current_completion_if_throw(function);
        self.emit_to_integer_or_infinity_number_payload_from_number_payload(
            end_payload_local,
            end_payload_local,
            function,
        );
        self.emit_array_slice_clamped_index(end_payload_local, len_local, final_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(count_local));
        function.instruction(&Instruction::LocalGet(final_local));
        function.instruction(&Instruction::LocalGet(first_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(final_local));
        function.instruction(&Instruction::LocalGet(first_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(count_local));
        function.instruction(&Instruction::End);

        self.emit_array_species_create(
            receiver_payload_local,
            receiver_tag_local,
            count_local,
            target_payload_local,
            target_tag_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(first_local));
        function.instruction(&Instruction::LocalSet(source_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(target_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(source_index_local));
        function.instruction(&Instruction::LocalGet(final_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_index_to_flat_map_key_local(
            source_index_local,
            source_number_payload_local,
            source_key_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_has_index_i32(
            receiver_payload_local,
            source_index_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_concat_typed_array_has_index_i32(
            receiver_payload_local,
            receiver_tag_local,
            source_index_local,
            has_property_local,
            typed_array_like_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(typed_array_like_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_has_property_i32(
            receiver_payload_local,
            receiver_tag_local,
            source_key_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(has_property_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_read(
            receiver_payload_local,
            source_index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(typed_array_like_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_or_object_index_read_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            source_index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            source_key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_propagate_throw_from_locals_if_needed(
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_index_to_flat_map_key_local(
            target_index_local,
            target_number_payload_local,
            target_key_local,
            function,
        )?;
        self.emit_array_target_create_data_property_or_throw(
            target_payload_local,
            target_tag_local,
            target_key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(source_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(source_index_local));
        function.instruction(&Instruction::LocalGet(target_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(target_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(target_length_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(number_tag_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(target_key_local));
        self.emit_object_write_strict(
            target_payload_local,
            target_tag_local,
            target_key_local,
            target_length_payload_local,
            number_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);

        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);

        self.release_temp_local(number_tag_local);
        self.release_temp_local(target_length_payload_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(typed_array_like_local);
        self.release_temp_local(has_property_local);
        self.release_temp_local(target_number_payload_local);
        self.release_temp_local(source_number_payload_local);
        self.release_temp_local(target_key_local);
        self.release_temp_local(source_key_local);
        self.release_temp_local(target_index_local);
        self.release_temp_local(source_index_local);
        self.release_temp_local(count_local);
        self.release_temp_local(final_local);
        self.release_temp_local(first_local);
        self.release_temp_local(len_local);
        self.release_temp_local(end_tag_local);
        self.release_temp_local(end_payload_local);
        self.release_temp_local(start_tag_local);
        self.release_temp_local(start_payload_local);
        self.release_temp_local(length_tag_local);
        self.release_temp_local(length_payload_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    pub(crate) fn compile_array_prototype_splice_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let this_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Array.prototype.splice receiver",
            )
        })?;
        let this_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Array.prototype.splice receiver tag",
            )
        })?;
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let length_payload_local = self.reserve_temp_local();
        let length_tag_local = self.reserve_temp_local();
        let start_payload_local = self.reserve_temp_local();
        let start_tag_local = self.reserve_temp_local();
        let delete_count_payload_local = self.reserve_temp_local();
        let delete_count_tag_local = self.reserve_temp_local();
        let relative_start_local = self.reserve_temp_local();
        let relative_delete_count_local = self.reserve_temp_local();
        let actual_start_local = self.reserve_temp_local();
        let actual_delete_count_local = self.reserve_temp_local();
        let item_count_local = self.reserve_temp_local();
        let new_length_local = self.reserve_temp_local();
        let deleted_payload_local = self.reserve_temp_local();
        let deleted_tag_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let argument_index_local = self.reserve_temp_local();
        let from_index_local = self.reserve_temp_local();
        let to_index_local = self.reserve_temp_local();
        let from_key_local = self.reserve_temp_local();
        let to_key_local = self.reserve_temp_local();
        let from_number_payload_local = self.reserve_temp_local();
        let to_number_payload_local = self.reserve_temp_local();
        let has_property_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let number_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(this_payload_local));
        function.instruction(&Instruction::LocalSet(receiver_payload_local));
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::LocalSet(receiver_tag_local));
        self.emit_value_to_current_function_realm_object_locals(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;

        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(from_key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            from_key_local,
            length_payload_local,
            length_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            length_payload_local,
            length_tag_local,
            function,
        )?;
        self.emit_to_length_i64_from_value_locals(
            length_tag_local,
            length_payload_local,
            len_local,
            function,
        )?;

        self.emit_builtin_arg_to_locals(0, start_payload_local, start_tag_local, function);
        self.emit_value_to_number_payload(start_tag_local, start_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(start_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(start_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncSatF64S);
        function.instruction(&Instruction::LocalSet(relative_start_local));
        function.instruction(&Instruction::LocalGet(relative_start_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalGet(relative_start_local));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(actual_start_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalGet(relative_start_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(actual_start_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalSet(actual_start_local));
        function.instruction(&Instruction::LocalGet(relative_start_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(relative_start_local));
        function.instruction(&Instruction::LocalSet(actual_start_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(item_count_local));
        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(item_count_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(actual_delete_count_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalGet(actual_start_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(actual_delete_count_local));
        function.instruction(&Instruction::Else);
        self.emit_builtin_arg_to_locals(
            1,
            delete_count_payload_local,
            delete_count_tag_local,
            function,
        );
        self.emit_value_to_number_payload(
            delete_count_tag_local,
            delete_count_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(delete_count_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(delete_count_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncSatF64S);
        function.instruction(&Instruction::LocalSet(relative_delete_count_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(actual_delete_count_local));
        function.instruction(&Instruction::LocalGet(relative_delete_count_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalGet(actual_start_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(actual_delete_count_local));
        function.instruction(&Instruction::LocalGet(relative_delete_count_local));
        function.instruction(&Instruction::LocalGet(actual_delete_count_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(relative_delete_count_local));
        function.instruction(&Instruction::LocalSet(actual_delete_count_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(item_count_local));
        function.instruction(&Instruction::I64Const(MAX_SAFE_INTEGER as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Array.prototype.push length exceeds safe integer",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(MAX_SAFE_INTEGER as i64));
        function.instruction(&Instruction::LocalGet(item_count_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(new_length_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalGet(actual_delete_count_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalGet(new_length_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Array.prototype.push length exceeds safe integer",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalGet(actual_delete_count_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalGet(item_count_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(new_length_local));

        self.emit_array_species_create(
            receiver_payload_local,
            receiver_tag_local,
            actual_delete_count_local,
            deleted_payload_local,
            deleted_tag_local,
            function,
        )?;

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(actual_delete_count_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(actual_start_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(from_index_local));
        self.emit_index_to_flat_map_key_local(
            from_index_local,
            from_number_payload_local,
            from_key_local,
            function,
        )?;
        self.emit_object_has_property_i32(
            receiver_payload_local,
            receiver_tag_local,
            from_key_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(has_property_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            from_key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_index_to_flat_map_key_local(
            index_local,
            to_number_payload_local,
            to_key_local,
            function,
        )?;
        self.emit_array_target_create_data_property_or_throw(
            deleted_payload_local,
            deleted_tag_local,
            to_key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(number_tag_local));
        function.instruction(&Instruction::LocalGet(actual_delete_count_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(length_payload_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(to_key_local));
        self.emit_object_write(
            deleted_payload_local,
            deleted_tag_local,
            to_key_local,
            length_payload_local,
            number_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);

        function.instruction(&Instruction::LocalGet(item_count_local));
        function.instruction(&Instruction::LocalGet(actual_delete_count_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(actual_start_local));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalGet(actual_delete_count_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(actual_delete_count_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(from_index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(item_count_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(to_index_local));
        self.emit_index_to_flat_map_key_local(
            from_index_local,
            from_number_payload_local,
            from_key_local,
            function,
        )?;
        self.emit_object_has_property_i32(
            receiver_payload_local,
            receiver_tag_local,
            from_key_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(has_property_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            from_key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_index_to_flat_map_key_local(
            to_index_local,
            to_number_payload_local,
            to_key_local,
            function,
        )?;
        self.emit_object_write(
            receiver_payload_local,
            receiver_tag_local,
            to_key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::Else);
        self.emit_index_to_flat_map_key_local(
            to_index_local,
            to_number_payload_local,
            to_key_local,
            function,
        )?;
        self.emit_delete_property_or_throw(
            receiver_payload_local,
            receiver_tag_local,
            to_key_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(new_length_local));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(index_local));
        self.emit_index_to_flat_map_key_local(
            index_local,
            to_number_payload_local,
            to_key_local,
            function,
        )?;
        self.emit_delete_property_or_throw(
            receiver_payload_local,
            receiver_tag_local,
            to_key_local,
            function,
        )?;
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(item_count_local));
        function.instruction(&Instruction::LocalGet(actual_delete_count_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalGet(actual_delete_count_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(actual_start_local));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(actual_delete_count_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(from_index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(item_count_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(to_index_local));
        self.emit_index_to_flat_map_key_local(
            from_index_local,
            from_number_payload_local,
            from_key_local,
            function,
        )?;
        self.emit_object_has_property_i32(
            receiver_payload_local,
            receiver_tag_local,
            from_key_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(has_property_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            from_key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_index_to_flat_map_key_local(
            to_index_local,
            to_number_payload_local,
            to_key_local,
            function,
        )?;
        self.emit_object_write(
            receiver_payload_local,
            receiver_tag_local,
            to_key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::Else);
        self.emit_index_to_flat_map_key_local(
            to_index_local,
            to_number_payload_local,
            to_key_local,
            function,
        )?;
        self.emit_delete_property_or_throw(
            receiver_payload_local,
            receiver_tag_local,
            to_key_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::LocalSet(argument_index_local));
        function.instruction(&Instruction::LocalGet(actual_start_local));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(argument_index_local));
        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            self.argv_param_local(),
            argument_index_local,
            element_payload_local,
            element_tag_local,
            function,
        );
        self.emit_index_to_flat_map_key_local(
            index_local,
            to_number_payload_local,
            to_key_local,
            function,
        )?;
        self.emit_object_write(
            receiver_payload_local,
            receiver_tag_local,
            to_key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(argument_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(argument_index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(new_length_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(length_payload_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(to_key_local));
        self.emit_object_write(
            receiver_payload_local,
            receiver_tag_local,
            to_key_local,
            length_payload_local,
            number_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);

        function.instruction(&Instruction::LocalGet(deleted_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(deleted_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);

        self.release_temp_local(number_tag_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(has_property_local);
        self.release_temp_local(to_number_payload_local);
        self.release_temp_local(from_number_payload_local);
        self.release_temp_local(to_key_local);
        self.release_temp_local(from_key_local);
        self.release_temp_local(to_index_local);
        self.release_temp_local(from_index_local);
        self.release_temp_local(argument_index_local);
        self.release_temp_local(index_local);
        self.release_temp_local(deleted_tag_local);
        self.release_temp_local(deleted_payload_local);
        self.release_temp_local(new_length_local);
        self.release_temp_local(item_count_local);
        self.release_temp_local(actual_delete_count_local);
        self.release_temp_local(actual_start_local);
        self.release_temp_local(relative_delete_count_local);
        self.release_temp_local(relative_start_local);
        self.release_temp_local(delete_count_tag_local);
        self.release_temp_local(delete_count_payload_local);
        self.release_temp_local(start_tag_local);
        self.release_temp_local(start_payload_local);
        self.release_temp_local(length_tag_local);
        self.release_temp_local(length_payload_local);
        self.release_temp_local(len_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    pub(crate) fn emit_array_splice_from_array_method_call(
        &mut self,
        receiver: &TypedExpr,
        args: &[TypedExpr],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if args.len() != 3 {
            return Err(EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: Array.prototype.splice array spread requires one source array",
            ));
        }
        let start = match &args[0].expr {
            ExprIr::Number(bits) => f64::from_bits(*bits),
            _ => {
                return Err(EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: Array.prototype.splice start must be static number",
                ));
            }
        };
        let delete_count = match &args[1].expr {
            ExprIr::Number(bits) => f64::from_bits(*bits),
            _ => {
                return Err(EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: Array.prototype.splice deleteCount must be static zero",
                ));
            }
        };
        if delete_count != 0.0 {
            return Err(EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: Array.prototype.splice only supports zero deleteCount",
            ));
        }

        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let insert_payload_local = self.reserve_temp_local();
        let insert_tag_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let insert_len_local = self.reserve_temp_local();
        let start_local = self.reserve_temp_local();
        let read_index_local = self.reserve_temp_local();
        let write_index_local = self.reserve_temp_local();
        let insert_index_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let deleted_len_local = self.reserve_temp_local();
        let deleted_payload_local = self.reserve_temp_local();

        self.compile_expr_to_locals(
            receiver,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Array.prototype.splice receiver is not array",
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.compile_expr_to_locals(&args[2], insert_payload_local, insert_tag_local, function)?;
        function.instruction(&Instruction::LocalGet(insert_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Array.prototype.splice receiver is not array",
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_LEN_OFFSET,
            len_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            insert_payload_local,
            HEAP_LEN_OFFSET,
            insert_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(
            if start.is_finite() && start > 0.0 {
                start.trunc() as i64
            } else {
                0
            },
        ));
        function.instruction(&Instruction::LocalSet(start_local));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalSet(start_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalSet(read_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(read_index_local));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(read_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(read_index_local));
        self.emit_array_read(
            receiver_payload_local,
            read_index_local,
            element_payload_local,
            element_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(read_index_local));
        function.instruction(&Instruction::LocalGet(insert_len_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        self.emit_array_write(
            receiver_payload_local,
            write_index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(insert_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(insert_index_local));
        function.instruction(&Instruction::LocalGet(insert_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            insert_payload_local,
            insert_index_local,
            element_payload_local,
            element_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::LocalGet(insert_index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        self.emit_array_write(
            receiver_payload_local,
            write_index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(insert_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(insert_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(deleted_len_local));
        self.emit_alloc_array_payload_with_length(
            deleted_len_local,
            deleted_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(deleted_payload_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));

        self.release_temp_local(deleted_payload_local);
        self.release_temp_local(deleted_len_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(insert_index_local);
        self.release_temp_local(write_index_local);
        self.release_temp_local(read_index_local);
        self.release_temp_local(start_local);
        self.release_temp_local(insert_len_local);
        self.release_temp_local(len_local);
        self.release_temp_local(insert_tag_local);
        self.release_temp_local(insert_payload_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    pub(crate) fn compile_array_prototype_includes_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Array.prototype.includes receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Array.prototype.includes receiver tag",
            )
        })?;
        let search_payload_local = self.reserve_temp_local();
        let search_tag_local = self.reserve_temp_local();
        let from_payload_local = self.reserve_temp_local();
        let from_tag_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, search_payload_local, search_tag_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(from_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(from_tag_local));
        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_builtin_arg_to_locals(1, from_payload_local, from_tag_local, function);
        function.instruction(&Instruction::End);

        self.emit_array_includes_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            search_payload_local,
            search_tag_local,
            from_payload_local,
            from_tag_local,
            self.result_local,
            self.result_tag_local,
            function,
        )?;

        self.release_temp_local(from_tag_local);
        self.release_temp_local(from_payload_local);
        self.release_temp_local(search_tag_local);
        self.release_temp_local(search_payload_local);
        Ok(())
    }

    pub(crate) fn compile_array_prototype_index_of_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Array.prototype.indexOf receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Array.prototype.indexOf receiver tag",
            )
        })?;
        let search_payload_local = self.reserve_temp_local();
        let search_tag_local = self.reserve_temp_local();
        let from_payload_local = self.reserve_temp_local();
        let from_tag_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, search_payload_local, search_tag_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(from_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(from_tag_local));
        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_builtin_arg_to_locals(1, from_payload_local, from_tag_local, function);
        function.instruction(&Instruction::End);

        self.emit_array_index_of_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            search_payload_local,
            search_tag_local,
            from_payload_local,
            from_tag_local,
            self.result_local,
            self.result_tag_local,
            function,
        )?;

        self.release_temp_local(from_tag_local);
        self.release_temp_local(from_payload_local);
        self.release_temp_local(search_tag_local);
        self.release_temp_local(search_payload_local);
        Ok(())
    }

    pub(crate) fn compile_array_prototype_last_index_of_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Array.prototype.lastIndexOf receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Array.prototype.lastIndexOf receiver tag",
            )
        })?;
        let search_payload_local = self.reserve_temp_local();
        let search_tag_local = self.reserve_temp_local();
        let from_payload_local = self.reserve_temp_local();
        let from_tag_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, search_payload_local, search_tag_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(from_payload_local));
        // Internal sentinel: omitted fromIndex differs from explicit undefined.
        function.instruction(&Instruction::I64Const(ValueKind::Dynamic.tag() as i64));
        function.instruction(&Instruction::LocalSet(from_tag_local));
        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_builtin_arg_to_locals(1, from_payload_local, from_tag_local, function);
        function.instruction(&Instruction::End);

        self.emit_array_last_index_of_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            search_payload_local,
            search_tag_local,
            from_payload_local,
            from_tag_local,
            self.result_local,
            self.result_tag_local,
            function,
        )?;

        self.release_temp_local(from_tag_local);
        self.release_temp_local(from_payload_local);
        self.release_temp_local(search_tag_local);
        self.release_temp_local(search_payload_local);
        Ok(())
    }

    pub(crate) fn compile_typed_array_prototype_includes_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_typed_array_search_builtin(TypedArraySearchKind::Includes, function)
    }

    pub(crate) fn compile_typed_array_prototype_index_of_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_typed_array_search_builtin(TypedArraySearchKind::IndexOf, function)
    }

    pub(crate) fn compile_typed_array_prototype_last_index_of_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_typed_array_search_builtin(TypedArraySearchKind::LastIndexOf, function)
    }

    fn compile_typed_array_search_builtin(
        &mut self,
        search_kind: TypedArraySearchKind,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let method_name = match &search_kind {
            TypedArraySearchKind::Includes => "TypedArray.prototype.includes",
            TypedArraySearchKind::IndexOf => "TypedArray.prototype.indexOf",
            TypedArraySearchKind::LastIndexOf => "TypedArray.prototype.lastIndexOf",
        };
        let incompatible_receiver_message = match &search_kind {
            TypedArraySearchKind::Includes => "TypedArray.prototype.includes requires a TypedArray",
            TypedArraySearchKind::IndexOf => "TypedArray.prototype.indexOf requires a TypedArray",
            TypedArraySearchKind::LastIndexOf => {
                "TypedArray.prototype.lastIndexOf requires a TypedArray"
            }
        };
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(format!(
                "unsupported in lila wasm-aot first slice: missing {method_name} receiver"
            ))
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(format!(
                "unsupported in lila wasm-aot first slice: missing {method_name} receiver tag"
            ))
        })?;
        let search_payload_local = self.reserve_temp_local();
        let search_tag_local = self.reserve_temp_local();
        let from_payload_local = self.reserve_temp_local();
        let from_tag_local = self.reserve_temp_local();
        let typed_brand_local = self.reserve_temp_local();
        let typed_buffer_payload_local = self.reserve_temp_local();
        let typed_byte_offset_local = self.reserve_temp_local();
        let typed_stored_byte_length_local = self.reserve_temp_local();
        let typed_bytes_per_element_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let element_present_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let typed_view = TypedArrayViewLocals::new(
            receiver_payload_local,
            typed_buffer_payload_local,
            typed_byte_offset_local,
            typed_stored_byte_length_local,
            typed_bytes_per_element_local,
        );

        self.emit_builtin_arg_to_locals(0, search_payload_local, search_tag_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(from_payload_local));
        function.instruction(&Instruction::I64Const(match &search_kind {
            TypedArraySearchKind::Includes | TypedArraySearchKind::IndexOf => {
                ValueKind::Undefined.tag() as i64
            }
            TypedArraySearchKind::LastIndexOf => ValueKind::Dynamic.tag() as i64,
        }));
        function.instruction(&Instruction::LocalSet(from_tag_local));
        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_builtin_arg_to_locals(1, from_payload_local, from_tag_local, function);
        function.instruction(&Instruction::End);

        match &search_kind {
            TypedArraySearchKind::Includes => {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
            }
            TypedArraySearchKind::IndexOf | TypedArraySearchKind::LastIndexOf => {
                function.instruction(&Instruction::I64Const((-1.0f64).to_bits() as i64));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
            }
        }
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(typed_brand_local));
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            typed_brand_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(typed_brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TYPED_ARRAY as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            incompatible_receiver_message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_load_typed_array_private_state(
            receiver_payload_local,
            typed_buffer_payload_local,
            typed_byte_offset_local,
            typed_stored_byte_length_local,
            typed_bytes_per_element_local,
            function,
        );
        self.emit_typed_array_witness(
            &typed_view,
            TypedArrayWitnessUse::ValidatedMethodEntry {
                length_local: len_local,
            },
            function,
        )?;

        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);

        match &search_kind {
            TypedArraySearchKind::Includes | TypedArraySearchKind::IndexOf => {
                function.instruction(&Instruction::LocalGet(from_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(index_local));
                function.instruction(&Instruction::Else);
                self.emit_value_to_number_payload(from_tag_local, from_payload_local, function)?;
                function.instruction(&Instruction::LocalSet(from_payload_local));
                self.emit_return_current_completion_if_throw(function);
                self.emit_to_slice_index_clamped_to_string_len(
                    from_payload_local,
                    len_local,
                    index_local,
                    function,
                );
                function.instruction(&Instruction::End);
            }
            TypedArraySearchKind::LastIndexOf => {
                function.instruction(&Instruction::LocalGet(from_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Dynamic.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(len_local));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Sub);
                function.instruction(&Instruction::LocalSet(index_local));
                function.instruction(&Instruction::Else);
                self.emit_value_to_number_payload(from_tag_local, from_payload_local, function)?;
                function.instruction(&Instruction::LocalSet(from_payload_local));
                self.emit_return_current_completion_if_throw(function);
                function.instruction(&Instruction::LocalGet(from_payload_local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::I64TruncSatF64S);
                function.instruction(&Instruction::LocalSet(index_local));
                function.instruction(&Instruction::LocalGet(index_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64LtS);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(len_local));
                function.instruction(&Instruction::LocalGet(index_local));
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::LocalSet(index_local));
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::LocalGet(index_local));
                function.instruction(&Instruction::LocalGet(len_local));
                function.instruction(&Instruction::I64GeU);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(len_local));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Sub);
                function.instruction(&Instruction::LocalSet(index_local));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
            }
        }

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        match &search_kind {
            TypedArraySearchKind::Includes | TypedArraySearchKind::IndexOf => {
                function.instruction(&Instruction::LocalGet(len_local));
                function.instruction(&Instruction::I64GeU);
            }
            TypedArraySearchKind::LastIndexOf => {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64LtS);
            }
        }
        function.instruction(&Instruction::BrIf(1));

        self.emit_typed_array_witness(
            &typed_view,
            TypedArrayWitnessUse::IntegerIndexedProperty {
                index_local,
                result_local: element_present_local,
            },
            function,
        )?;
        function.instruction(&Instruction::LocalGet(element_present_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_or_object_index_read_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(element_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(element_tag_local));
        function.instruction(&Instruction::End);

        match &search_kind {
            TypedArraySearchKind::Includes => {}
            TypedArraySearchKind::IndexOf | TypedArraySearchKind::LastIndexOf => {
                function.instruction(&Instruction::LocalGet(element_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::If(BlockType::Empty));
            }
        }
        match &search_kind {
            TypedArraySearchKind::Includes => {
                self.emit_tagged_payload_same_value_zero_i32(
                    element_tag_local,
                    element_payload_local,
                    search_tag_local,
                    search_payload_local,
                    function,
                )?;
            }
            TypedArraySearchKind::IndexOf | TypedArraySearchKind::LastIndexOf => {
                self.emit_tagged_payload_equality_i32(
                    element_tag_local,
                    element_payload_local,
                    search_tag_local,
                    search_payload_local,
                    function,
                )?;
            }
        }
        function.instruction(&Instruction::If(BlockType::Empty));
        match &search_kind {
            TypedArraySearchKind::Includes => {
                function.instruction(&Instruction::I64Const(1));
            }
            TypedArraySearchKind::IndexOf | TypedArraySearchKind::LastIndexOf => {
                function.instruction(&Instruction::LocalGet(index_local));
                function.instruction(&Instruction::F64ConvertI64U);
                function.instruction(&Instruction::I64ReinterpretF64);
            }
        }
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::Br(match &search_kind {
            TypedArraySearchKind::Includes => 2,
            TypedArraySearchKind::IndexOf | TypedArraySearchKind::LastIndexOf => 3,
        }));
        function.instruction(&Instruction::End);
        match &search_kind {
            TypedArraySearchKind::Includes => {}
            TypedArraySearchKind::IndexOf | TypedArraySearchKind::LastIndexOf => {
                function.instruction(&Instruction::End);
            }
        }

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        match &search_kind {
            TypedArraySearchKind::Includes | TypedArraySearchKind::IndexOf => {
                function.instruction(&Instruction::I64Add);
            }
            TypedArraySearchKind::LastIndexOf => {
                function.instruction(&Instruction::I64Sub);
            }
        }
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(element_present_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(typed_bytes_per_element_local);
        self.release_temp_local(typed_stored_byte_length_local);
        self.release_temp_local(typed_byte_offset_local);
        self.release_temp_local(typed_buffer_payload_local);
        self.release_temp_local(typed_brand_local);
        self.release_temp_local(from_tag_local);
        self.release_temp_local(from_payload_local);
        self.release_temp_local(search_tag_local);
        self.release_temp_local(search_payload_local);
        Ok(())
    }

    pub(super) fn compile_array_prototype_at_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_array_like_at_builtin(ArrayAtReceiverPolicy::GenericArrayLike, function)
    }

    pub(super) fn compile_typed_array_prototype_at_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_array_like_at_builtin(ArrayAtReceiverPolicy::TypedArray, function)
    }

    fn compile_array_like_at_builtin(
        &mut self,
        receiver_policy: ArrayAtReceiverPolicy,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Array.prototype.at receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Array.prototype.at receiver tag",
            )
        })?;
        let index_payload_local = self.reserve_temp_local();
        let index_tag_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, index_payload_local, index_tag_local, function);
        self.emit_array_at_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            index_payload_local,
            index_tag_local,
            self.result_local,
            self.result_tag_local,
            receiver_policy,
            function,
        )?;

        self.release_temp_local(index_tag_local);
        self.release_temp_local(index_payload_local);
        Ok(())
    }

    pub(crate) fn compile_array_prototype_to_reversed_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Array.prototype.toReversed receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Array.prototype.toReversed receiver tag",
            )
        })?;
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let source_index_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let length_payload_local = self.reserve_temp_local();
        let length_tag_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();

        self.emit_value_to_object_locals(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            length_payload_local,
            length_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            length_payload_local,
            length_tag_local,
            function,
        )?;
        self.emit_to_length_i64_from_value_locals(
            length_tag_local,
            length_payload_local,
            len_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(u32::MAX as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Invalid array length",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_alloc_array_payload_with_length(len_local, self.result_local, function)?;
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(source_index_local));

        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_index_get_with_prototype(
            receiver_payload_local,
            source_index_local,
            receiver_payload_local,
            receiver_tag_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_typed_array_or_object_index_read_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            source_index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_propagate_throw_from_locals_if_needed(
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_array_write(
            self.result_local,
            index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(length_tag_local);
        self.release_temp_local(length_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(source_index_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        Ok(())
    }

    pub(crate) fn compile_array_prototype_with_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Array.prototype.with receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Array.prototype.with receiver tag",
            )
        })?;
        let index_payload_local = self.reserve_temp_local();
        let index_tag_local = self.reserve_temp_local();
        let replacement_payload_local = self.reserve_temp_local();
        let replacement_tag_local = self.reserve_temp_local();
        let relative_index_payload_local = self.reserve_temp_local();
        let actual_index_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let copy_index_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let length_payload_local = self.reserve_temp_local();
        let length_tag_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, index_payload_local, index_tag_local, function);
        self.emit_builtin_arg_to_locals(
            1,
            replacement_payload_local,
            replacement_tag_local,
            function,
        );
        self.emit_value_to_object_locals(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            length_payload_local,
            length_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            length_payload_local,
            length_tag_local,
            function,
        )?;
        self.emit_to_length_i64_from_value_locals(
            length_tag_local,
            length_payload_local,
            len_local,
            function,
        )?;

        self.emit_value_to_number_payload(index_tag_local, index_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(index_payload_local));
        self.emit_return_current_completion_if_throw(function);
        self.emit_to_integer_or_infinity_number_payload_from_number_payload(
            index_payload_local,
            relative_index_payload_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(relative_index_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Ge);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(relative_index_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::F64Ge);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Array.prototype.with index out of range",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(relative_index_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(actual_index_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(relative_index_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::F64Neg);
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Array.prototype.with index out of range",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalGet(relative_index_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64S);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(actual_index_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(u32::MAX as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Invalid array length",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_alloc_array_payload_with_length(len_local, self.result_local, function)?;
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(copy_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(copy_index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(copy_index_local));
        function.instruction(&Instruction::LocalGet(actual_index_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(replacement_payload_local));
        function.instruction(&Instruction::LocalSet(element_payload_local));
        function.instruction(&Instruction::LocalGet(replacement_tag_local));
        function.instruction(&Instruction::LocalSet(element_tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_index_get_with_prototype(
            receiver_payload_local,
            copy_index_local,
            receiver_payload_local,
            receiver_tag_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_typed_array_or_object_index_read_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            copy_index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_propagate_throw_from_locals_if_needed(
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_array_write(
            self.result_local,
            copy_index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(copy_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(copy_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(length_tag_local);
        self.release_temp_local(length_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(copy_index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(actual_index_local);
        self.release_temp_local(relative_index_payload_local);
        self.release_temp_local(replacement_tag_local);
        self.release_temp_local(replacement_payload_local);
        self.release_temp_local(index_tag_local);
        self.release_temp_local(index_payload_local);
        Ok(())
    }

    pub(crate) fn compile_array_prototype_to_spliced_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Array.prototype.toSpliced receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Array.prototype.toSpliced receiver tag",
            )
        })?;
        let start_payload_local = self.reserve_temp_local();
        let start_tag_local = self.reserve_temp_local();
        let delete_payload_local = self.reserve_temp_local();
        let delete_tag_local = self.reserve_temp_local();
        let relative_start_payload_local = self.reserve_temp_local();
        let delete_count_payload_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let actual_start_local = self.reserve_temp_local();
        let actual_delete_count_local = self.reserve_temp_local();
        let remaining_count_local = self.reserve_temp_local();
        let insert_count_local = self.reserve_temp_local();
        let new_len_local = self.reserve_temp_local();
        let source_index_local = self.reserve_temp_local();
        let destination_index_local = self.reserve_temp_local();
        let argument_index_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let length_payload_local = self.reserve_temp_local();
        let length_tag_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, start_payload_local, start_tag_local, function);
        self.emit_builtin_arg_to_locals(1, delete_payload_local, delete_tag_local, function);
        self.emit_value_to_object_locals(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            length_payload_local,
            length_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            length_payload_local,
            length_tag_local,
            function,
        )?;
        self.emit_to_length_i64_from_value_locals(
            length_tag_local,
            length_payload_local,
            len_local,
            function,
        )?;

        self.emit_value_to_number_payload(start_tag_local, start_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(start_payload_local));
        self.emit_return_current_completion_if_throw(function);
        self.emit_to_integer_or_infinity_number_payload_from_number_payload(
            start_payload_local,
            relative_start_payload_local,
            function,
        );
        self.emit_array_slice_clamped_index(
            relative_start_payload_local,
            len_local,
            actual_start_local,
            function,
        );

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(actual_delete_count_local));
        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalGet(actual_start_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(actual_delete_count_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_number_payload(delete_tag_local, delete_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(delete_payload_local));
        self.emit_return_current_completion_if_throw(function);
        self.emit_to_integer_or_infinity_number_payload_from_number_payload(
            delete_payload_local,
            delete_count_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalGet(actual_start_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(remaining_count_local));
        function.instruction(&Instruction::LocalGet(delete_count_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Gt);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(delete_count_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(remaining_count_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::F64Ge);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(remaining_count_local));
        function.instruction(&Instruction::LocalSet(actual_delete_count_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(delete_count_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(actual_delete_count_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(insert_count_local));
        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(insert_count_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalGet(insert_count_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(actual_delete_count_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(new_len_local));
        function.instruction(&Instruction::LocalGet(new_len_local));
        function.instruction(&Instruction::I64Const(9_007_199_254_740_991));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Array.prototype.toSpliced result exceeds maximum safe length",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(new_len_local));
        function.instruction(&Instruction::I64Const(u32::MAX as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Invalid array length",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_alloc_array_payload_with_length(new_len_local, self.result_local, function)?;
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(source_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(destination_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(source_index_local));
        function.instruction(&Instruction::LocalGet(actual_start_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_index_get_with_prototype(
            receiver_payload_local,
            source_index_local,
            receiver_payload_local,
            receiver_tag_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_typed_array_or_object_index_read_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            source_index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_propagate_throw_from_locals_if_needed(
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_array_write(
            self.result_local,
            destination_index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(source_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(source_index_local));
        function.instruction(&Instruction::LocalGet(destination_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(destination_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::LocalSet(argument_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(argument_index_local));
        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            self.argv_param_local(),
            argument_index_local,
            element_payload_local,
            element_tag_local,
            function,
        );
        self.emit_array_write(
            self.result_local,
            destination_index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(argument_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(argument_index_local));
        function.instruction(&Instruction::LocalGet(destination_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(destination_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(actual_start_local));
        function.instruction(&Instruction::LocalGet(actual_delete_count_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(source_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(source_index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_index_get_with_prototype(
            receiver_payload_local,
            source_index_local,
            receiver_payload_local,
            receiver_tag_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_typed_array_or_object_index_read_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            source_index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_propagate_throw_from_locals_if_needed(
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_array_write(
            self.result_local,
            destination_index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(source_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(source_index_local));
        function.instruction(&Instruction::LocalGet(destination_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(destination_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(length_tag_local);
        self.release_temp_local(length_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(argument_index_local);
        self.release_temp_local(destination_index_local);
        self.release_temp_local(source_index_local);
        self.release_temp_local(new_len_local);
        self.release_temp_local(insert_count_local);
        self.release_temp_local(remaining_count_local);
        self.release_temp_local(actual_delete_count_local);
        self.release_temp_local(actual_start_local);
        self.release_temp_local(len_local);
        self.release_temp_local(delete_count_payload_local);
        self.release_temp_local(relative_start_payload_local);
        self.release_temp_local(delete_tag_local);
        self.release_temp_local(delete_payload_local);
        self.release_temp_local(start_tag_local);
        self.release_temp_local(start_payload_local);
        Ok(())
    }

    pub(crate) fn compile_array_prototype_reverse_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let this_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Array.prototype.reverse receiver",
            )
        })?;
        let this_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Array.prototype.reverse receiver tag",
            )
        })?;
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let middle_local = self.reserve_temp_local();
        let lower_local = self.reserve_temp_local();
        let upper_local = self.reserve_temp_local();
        let number_payload_local = self.reserve_temp_local();
        let lower_key_local = self.reserve_temp_local();
        let upper_key_local = self.reserve_temp_local();
        let length_payload_local = self.reserve_temp_local();
        let length_tag_local = self.reserve_temp_local();
        let lower_present_local = self.reserve_temp_local();
        let upper_present_local = self.reserve_temp_local();
        let lower_payload_local = self.reserve_temp_local();
        let lower_tag_local = self.reserve_temp_local();
        let upper_payload_local = self.reserve_temp_local();
        let upper_tag_local = self.reserve_temp_local();
        let receiver_is_typed_array_local = self.reserve_temp_local();

        self.emit_value_to_current_function_realm_object_locals(
            this_payload_local,
            this_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(lower_key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            lower_key_local,
            length_payload_local,
            length_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_to_length_i64_from_value_locals(
            length_tag_local,
            length_payload_local,
            len_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(receiver_is_typed_array_local));
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            receiver_is_typed_array_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(receiver_is_typed_array_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TYPED_ARRAY as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(receiver_is_typed_array_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(middle_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(lower_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(lower_local));
        function.instruction(&Instruction::LocalGet(middle_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalGet(lower_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(upper_local));

        function.instruction(&Instruction::LocalGet(lower_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(number_payload_local));
        self.emit_number_to_string_payload(number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(lower_key_local));
        function.instruction(&Instruction::LocalGet(upper_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(number_payload_local));
        self.emit_number_to_string_payload(number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(upper_key_local));

        self.emit_object_has_property_i32(
            receiver_payload_local,
            receiver_tag_local,
            lower_key_local,
            lower_present_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(receiver_is_typed_array_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(lower_present_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(lower_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(receiver_is_typed_array_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_or_object_index_read_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            lower_local,
            lower_payload_local,
            lower_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            lower_key_local,
            lower_payload_local,
            lower_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);

        self.emit_object_has_property_i32(
            receiver_payload_local,
            receiver_tag_local,
            upper_key_local,
            upper_present_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(receiver_is_typed_array_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(upper_present_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(upper_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(receiver_is_typed_array_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_or_object_index_read_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            upper_local,
            upper_payload_local,
            upper_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            upper_key_local,
            upper_payload_local,
            upper_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(lower_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(upper_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(receiver_is_typed_array_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_element_write_from_locals(
            receiver_payload_local,
            lower_local,
            upper_payload_local,
            upper_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_object_write_strict(
            receiver_payload_local,
            receiver_tag_local,
            lower_key_local,
            upper_payload_local,
            upper_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(receiver_is_typed_array_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_element_write_from_locals(
            receiver_payload_local,
            upper_local,
            lower_payload_local,
            lower_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_object_write_strict(
            receiver_payload_local,
            receiver_tag_local,
            upper_key_local,
            lower_payload_local,
            lower_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::Else);
        self.emit_delete_property_or_throw(
            receiver_payload_local,
            receiver_tag_local,
            lower_key_local,
            function,
        )?;
        self.emit_object_write_strict(
            receiver_payload_local,
            receiver_tag_local,
            upper_key_local,
            lower_payload_local,
            lower_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(upper_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_write_strict(
            receiver_payload_local,
            receiver_tag_local,
            lower_key_local,
            upper_payload_local,
            upper_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_delete_property_or_throw(
            receiver_payload_local,
            receiver_tag_local,
            upper_key_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(lower_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(lower_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(receiver_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(receiver_is_typed_array_local);
        self.release_temp_local(upper_tag_local);
        self.release_temp_local(upper_payload_local);
        self.release_temp_local(lower_tag_local);
        self.release_temp_local(lower_payload_local);
        self.release_temp_local(upper_present_local);
        self.release_temp_local(lower_present_local);
        self.release_temp_local(length_tag_local);
        self.release_temp_local(length_payload_local);
        self.release_temp_local(upper_key_local);
        self.release_temp_local(lower_key_local);
        self.release_temp_local(number_payload_local);
        self.release_temp_local(upper_local);
        self.release_temp_local(lower_local);
        self.release_temp_local(middle_local);
        self.release_temp_local(len_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    pub(crate) fn compile_array_prototype_copy_within_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let this_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Array.prototype.copyWithin receiver",
            )
        })?;
        let this_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Array.prototype.copyWithin receiver tag",
            )
        })?;
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let length_payload_local = self.reserve_temp_local();
        let length_tag_local = self.reserve_temp_local();
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let to_local = self.reserve_temp_local();
        let from_local = self.reserve_temp_local();
        let final_local = self.reserve_temp_local();
        let count_local = self.reserve_temp_local();
        let available_local = self.reserve_temp_local();
        let direction_local = self.reserve_temp_local();
        let from_key_local = self.reserve_temp_local();
        let to_key_local = self.reserve_temp_local();
        let number_payload_local = self.reserve_temp_local();
        let from_present_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let receiver_is_typed_array_local = self.reserve_temp_local();

        self.emit_value_to_current_function_realm_object_locals(
            this_payload_local,
            this_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(from_key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            from_key_local,
            length_payload_local,
            length_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_to_length_i64_from_value_locals(
            length_tag_local,
            length_payload_local,
            len_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(receiver_is_typed_array_local));
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            receiver_is_typed_array_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(receiver_is_typed_array_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TYPED_ARRAY as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(receiver_is_typed_array_local));
        function.instruction(&Instruction::End);

        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        self.emit_value_to_number_payload(argument_tag_local, argument_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(argument_payload_local));
        self.emit_return_current_completion_if_throw(function);
        self.emit_to_integer_or_infinity_number_payload_from_number_payload(
            argument_payload_local,
            argument_payload_local,
            function,
        );
        self.emit_array_slice_clamped_index(argument_payload_local, len_local, to_local, function);

        self.emit_builtin_arg_to_locals(1, argument_payload_local, argument_tag_local, function);
        self.emit_value_to_number_payload(argument_tag_local, argument_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(argument_payload_local));
        self.emit_return_current_completion_if_throw(function);
        self.emit_to_integer_or_infinity_number_payload_from_number_payload(
            argument_payload_local,
            argument_payload_local,
            function,
        );
        self.emit_array_slice_clamped_index(
            argument_payload_local,
            len_local,
            from_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalSet(final_local));
        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_builtin_arg_to_locals(2, argument_payload_local, argument_tag_local, function);
        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_number_payload(argument_tag_local, argument_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(argument_payload_local));
        self.emit_return_current_completion_if_throw(function);
        self.emit_to_integer_or_infinity_number_payload_from_number_payload(
            argument_payload_local,
            argument_payload_local,
            function,
        );
        self.emit_array_slice_clamped_index(
            argument_payload_local,
            len_local,
            final_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(count_local));
        function.instruction(&Instruction::LocalGet(final_local));
        function.instruction(&Instruction::LocalGet(from_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(final_local));
        function.instruction(&Instruction::LocalGet(from_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(count_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalGet(to_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(available_local));
        function.instruction(&Instruction::LocalGet(available_local));
        function.instruction(&Instruction::LocalGet(count_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(available_local));
        function.instruction(&Instruction::LocalSet(count_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_array_copy_within_traversal_start(
            ArrayCopyWithinDirection::Forward,
            from_local,
            to_local,
            count_local,
            direction_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(from_local));
        function.instruction(&Instruction::LocalGet(to_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(to_local));
        function.instruction(&Instruction::LocalGet(from_local));
        function.instruction(&Instruction::LocalGet(count_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_copy_within_traversal_start(
            ArrayCopyWithinDirection::Backward,
            from_local,
            to_local,
            count_local,
            direction_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(count_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        self.emit_index_to_flat_map_key_local(
            from_local,
            number_payload_local,
            from_key_local,
            function,
        )?;
        self.emit_index_to_flat_map_key_local(
            to_local,
            number_payload_local,
            to_key_local,
            function,
        )?;
        self.emit_object_has_property_i32(
            receiver_payload_local,
            receiver_tag_local,
            from_key_local,
            from_present_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(receiver_is_typed_array_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(from_present_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(from_present_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(receiver_is_typed_array_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_or_object_index_read_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            from_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            from_key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(receiver_is_typed_array_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_element_write_from_locals(
            receiver_payload_local,
            to_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_object_write_strict(
            receiver_payload_local,
            receiver_tag_local,
            to_key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::Else);
        self.emit_delete_property_or_throw(
            receiver_payload_local,
            receiver_tag_local,
            to_key_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(from_local));
        function.instruction(&Instruction::LocalGet(direction_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(from_local));
        function.instruction(&Instruction::LocalGet(to_local));
        function.instruction(&Instruction::LocalGet(direction_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(to_local));
        function.instruction(&Instruction::LocalGet(count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(count_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(receiver_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(receiver_is_typed_array_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(from_present_local);
        self.release_temp_local(number_payload_local);
        self.release_temp_local(to_key_local);
        self.release_temp_local(from_key_local);
        self.release_temp_local(direction_local);
        self.release_temp_local(available_local);
        self.release_temp_local(count_local);
        self.release_temp_local(final_local);
        self.release_temp_local(from_local);
        self.release_temp_local(to_local);
        self.release_temp_local(len_local);
        self.release_temp_local(argument_tag_local);
        self.release_temp_local(argument_payload_local);
        self.release_temp_local(length_tag_local);
        self.release_temp_local(length_payload_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_array_at_from_locals(
        &mut self,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        index_payload_local: u32,
        index_tag_local: u32,
        payload_local: u32,
        tag_local: u32,
        receiver_policy: ArrayAtReceiverPolicy,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let len_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let length_payload_local = self.reserve_temp_local();
        let length_tag_local = self.reserve_temp_local();
        let typed_array_brand_local = self.reserve_temp_local();
        let typed_receiver_local = self.reserve_temp_local();
        let typed_buffer_payload_local = self.reserve_temp_local();
        let typed_byte_offset_local = self.reserve_temp_local();
        let typed_stored_byte_length_local = self.reserve_temp_local();
        let typed_bytes_per_element_local = self.reserve_temp_local();
        let relative_index_local = self.reserve_temp_local();
        let negative_bound_local = self.reserve_temp_local();
        let k_local = self.reserve_temp_local();
        let element_present_local = self.reserve_temp_local();
        let typed_view = TypedArrayViewLocals::new(
            receiver_payload_local,
            typed_buffer_payload_local,
            typed_byte_offset_local,
            typed_stored_byte_length_local,
            typed_bytes_per_element_local,
        );

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(typed_receiver_local));

        self.emit_array_iteration_to_object(receiver_payload_local, receiver_tag_local, function)?;

        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        match &receiver_policy {
            ArrayAtReceiverPolicy::GenericArrayLike => {
                self.load_i64_to_local_from_offset(
                    receiver_payload_local,
                    HEAP_LEN_OFFSET,
                    len_local,
                    function,
                );
            }
            ArrayAtReceiverPolicy::TypedArray => {
                self.emit_throw_current_function_realm_type_error(
                    "TypedArray.prototype.at called on incompatible receiver",
                    payload_local,
                    tag_local,
                    function,
                )?;
                self.emit_return_current_completion(function);
            }
        }
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            typed_array_brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(typed_array_brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TYPED_ARRAY as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(typed_receiver_local));
        self.emit_load_typed_array_private_state(
            receiver_payload_local,
            typed_buffer_payload_local,
            typed_byte_offset_local,
            typed_stored_byte_length_local,
            typed_bytes_per_element_local,
            function,
        );
        let witness_use = match &receiver_policy {
            ArrayAtReceiverPolicy::GenericArrayLike => {
                TypedArrayWitnessUse::ArrayLikeLengthSnapshot {
                    length_local: len_local,
                }
            }
            ArrayAtReceiverPolicy::TypedArray => TypedArrayWitnessUse::ValidatedMethodEntry {
                length_local: len_local,
            },
        };
        self.emit_typed_array_witness(&typed_view, witness_use, function)?;
        function.instruction(&Instruction::Else);
        match &receiver_policy {
            ArrayAtReceiverPolicy::GenericArrayLike => {
                function.instruction(&Instruction::I64Const(self.strings.payload("length")));
                function.instruction(&Instruction::LocalSet(key_local));
                self.emit_object_read(
                    receiver_payload_local,
                    receiver_tag_local,
                    receiver_payload_local,
                    receiver_tag_local,
                    key_local,
                    length_payload_local,
                    length_tag_local,
                    function,
                )?;
                self.emit_return_current_completion_if_throw(function);
                self.emit_to_length_i64_from_value_locals(
                    length_tag_local,
                    length_payload_local,
                    len_local,
                    function,
                )?;
            }
            ArrayAtReceiverPolicy::TypedArray => {
                self.emit_throw_current_function_realm_type_error(
                    "TypedArray.prototype.at called on incompatible receiver",
                    payload_local,
                    tag_local,
                    function,
                )?;
                self.emit_return_current_completion(function);
            }
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        match &receiver_policy {
            ArrayAtReceiverPolicy::GenericArrayLike => {
                self.emit_array_iteration_nullish_receiver_throw_or_zero_length(
                    receiver_tag_local,
                    len_local,
                    payload_local,
                    tag_local,
                    "Array.prototype.at called on null or undefined",
                    function,
                )?;
            }
            ArrayAtReceiverPolicy::TypedArray => {
                self.emit_throw_current_function_realm_type_error(
                    "TypedArray.prototype.at called on incompatible receiver",
                    payload_local,
                    tag_local,
                    function,
                )?;
                self.emit_return_current_completion(function);
            }
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_value_to_number_payload(index_tag_local, index_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(index_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(index_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncSatF64S);
        function.instruction(&Instruction::LocalSet(relative_index_local));

        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalSet(k_local));

        function.instruction(&Instruction::LocalGet(relative_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(negative_bound_local));
        function.instruction(&Instruction::LocalGet(relative_index_local));
        function.instruction(&Instruction::LocalGet(negative_bound_local));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalGet(relative_index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(k_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(relative_index_local));
        function.instruction(&Instruction::LocalSet(k_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(k_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_witness(
            &typed_view,
            TypedArrayWitnessUse::IntegerIndexedProperty {
                index_local: k_local,
                result_local: element_present_local,
            },
            function,
        )?;
        function.instruction(&Instruction::LocalGet(element_present_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_or_object_index_read_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            k_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_typed_array_or_object_index_read_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            k_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(element_present_local);
        self.release_temp_local(k_local);
        self.release_temp_local(negative_bound_local);
        self.release_temp_local(relative_index_local);
        self.release_temp_local(typed_bytes_per_element_local);
        self.release_temp_local(typed_stored_byte_length_local);
        self.release_temp_local(typed_byte_offset_local);
        self.release_temp_local(typed_buffer_payload_local);
        self.release_temp_local(typed_receiver_local);
        self.release_temp_local(typed_array_brand_local);
        self.release_temp_local(length_tag_local);
        self.release_temp_local(length_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(len_local);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_array_includes_from_locals(
        &mut self,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        search_payload_local: u32,
        search_tag_local: u32,
        from_payload_local: u32,
        from_tag_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let typed_receiver_local = self.reserve_temp_local();
        let typed_buffer_payload_local = self.reserve_temp_local();
        let typed_byte_offset_local = self.reserve_temp_local();
        let typed_stored_byte_length_local = self.reserve_temp_local();
        let typed_bytes_per_element_local = self.reserve_temp_local();
        let element_present_local = self.reserve_temp_local();
        let typed_view = TypedArrayViewLocals::new(
            receiver_payload_local,
            typed_buffer_payload_local,
            typed_byte_offset_local,
            typed_stored_byte_length_local,
            typed_bytes_per_element_local,
        );

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(typed_receiver_local));

        // Array.prototype.includes is intentionally generic. Perform the
        // observable ToObject/LengthOfArrayLike steps even for Arrays,
        // Arguments objects, and branded TypedArrays; their indexed Get path
        // may still specialize after the length snapshot is complete.
        self.emit_value_to_current_function_realm_object_locals(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_to_length_i64_from_value_locals(
            element_tag_local,
            element_payload_local,
            len_local,
            function,
        )?;
        self.emit_is_typed_array_i32(receiver_payload_local, receiver_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(typed_receiver_local));
        self.emit_load_typed_array_private_state(
            receiver_payload_local,
            typed_buffer_payload_local,
            typed_byte_offset_local,
            typed_stored_byte_length_local,
            typed_bytes_per_element_local,
            function,
        );
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(from_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Else);
        self.emit_value_to_number_payload(from_tag_local, from_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(from_payload_local));
        self.emit_return_current_completion_if_throw(function);
        self.emit_to_slice_index_clamped_to_string_len(
            from_payload_local,
            len_local,
            index_local,
            function,
        );
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_index_get_with_prototype(
            receiver_payload_local,
            index_local,
            receiver_payload_local,
            receiver_tag_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_witness(
            &typed_view,
            TypedArrayWitnessUse::IntegerIndexedProperty {
                index_local,
                result_local: element_present_local,
            },
            function,
        )?;
        function.instruction(&Instruction::LocalGet(element_present_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_or_object_index_read_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(element_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(element_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_typed_array_or_object_index_read_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_tagged_payload_same_value_zero_i32(
            element_tag_local,
            element_payload_local,
            search_tag_local,
            search_payload_local,
            function,
        )?;
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(element_present_local);
        self.release_temp_local(typed_bytes_per_element_local);
        self.release_temp_local(typed_stored_byte_length_local);
        self.release_temp_local(typed_byte_offset_local);
        self.release_temp_local(typed_buffer_payload_local);
        self.release_temp_local(typed_receiver_local);
        self.release_temp_local(key_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_array_index_of_from_locals(
        &mut self,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        search_payload_local: u32,
        search_tag_local: u32,
        from_payload_local: u32,
        from_tag_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();
        let has_property_local = self.reserve_temp_local();
        let typed_receiver_local = self.reserve_temp_local();
        let typed_buffer_payload_local = self.reserve_temp_local();
        let typed_byte_offset_local = self.reserve_temp_local();
        let typed_stored_byte_length_local = self.reserve_temp_local();
        let typed_bytes_per_element_local = self.reserve_temp_local();
        let array_hole_prototype_clean_local = self.reserve_temp_local();
        let typed_view = TypedArrayViewLocals::new(
            receiver_payload_local,
            typed_buffer_payload_local,
            typed_byte_offset_local,
            typed_stored_byte_length_local,
            typed_bytes_per_element_local,
        );

        function.instruction(&Instruction::I64Const((-1.0f64).to_bits() as i64));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(typed_receiver_local));

        self.emit_array_iteration_to_object(receiver_payload_local, receiver_tag_local, function)?;

        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_length(
            receiver_payload_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_to_length_i64_from_value_locals(
            element_tag_local,
            element_payload_local,
            len_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_is_typed_array_i32(receiver_payload_local, receiver_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(typed_receiver_local));
        self.emit_load_typed_array_private_state(
            receiver_payload_local,
            typed_buffer_payload_local,
            typed_byte_offset_local,
            typed_stored_byte_length_local,
            typed_bytes_per_element_local,
            function,
        );
        self.emit_typed_array_witness(
            &typed_view,
            TypedArrayWitnessUse::ArrayLikeLengthSnapshot {
                length_local: len_local,
            },
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_to_length_i64_from_value_locals(
            element_tag_local,
            element_payload_local,
            len_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_array_iteration_nullish_receiver_throw_or_zero_length(
            receiver_tag_local,
            len_local,
            payload_local,
            tag_local,
            "Array.prototype.indexOf called on null or undefined",
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(from_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Else);
        self.emit_value_to_number_payload(from_tag_local, from_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(from_payload_local));
        self.emit_return_current_completion_if_throw(function);
        self.emit_to_slice_index_clamped_to_string_len(
            from_payload_local,
            len_local,
            index_local,
            function,
        );
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        self.emit_array_hole_prototype_clean_i32(
            receiver_payload_local,
            receiver_tag_local,
            index_local,
            len_local,
            array_hole_prototype_clean_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(array_hole_prototype_clean_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_advance_to_next_present_index(
            receiver_payload_local,
            index_local,
            len_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        self.emit_index_to_flat_map_key_local(
            index_local,
            index_number_payload_local,
            key_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(has_property_local));
        function.instruction(&Instruction::LocalGet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_witness(
            &typed_view,
            TypedArrayWitnessUse::IntegerIndexedProperty {
                index_local,
                result_local: has_property_local,
            },
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_has_index_i32(
            receiver_payload_local,
            index_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_object_has_property_i32(
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(has_property_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_or_object_index_read_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_index_get_with_prototype(
            receiver_payload_local,
            index_local,
            receiver_payload_local,
            receiver_tag_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_typed_array_or_object_index_read_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_tagged_payload_equality_i32(
            element_tag_local,
            element_payload_local,
            search_tag_local,
            search_payload_local,
            function,
        )?;
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::Br(3));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(array_hole_prototype_clean_local);
        self.release_temp_local(typed_bytes_per_element_local);
        self.release_temp_local(typed_stored_byte_length_local);
        self.release_temp_local(typed_byte_offset_local);
        self.release_temp_local(typed_buffer_payload_local);
        self.release_temp_local(typed_receiver_local);
        self.release_temp_local(has_property_local);
        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_array_last_index_of_from_locals(
        &mut self,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        search_payload_local: u32,
        search_tag_local: u32,
        from_payload_local: u32,
        from_tag_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();
        let has_property_local = self.reserve_temp_local();
        let done_local = self.reserve_temp_local();
        let typed_receiver_local = self.reserve_temp_local();
        let typed_buffer_payload_local = self.reserve_temp_local();
        let typed_byte_offset_local = self.reserve_temp_local();
        let typed_stored_byte_length_local = self.reserve_temp_local();
        let typed_bytes_per_element_local = self.reserve_temp_local();
        let array_hole_prototype_clean_local = self.reserve_temp_local();
        let typed_view = TypedArrayViewLocals::new(
            receiver_payload_local,
            typed_buffer_payload_local,
            typed_byte_offset_local,
            typed_stored_byte_length_local,
            typed_bytes_per_element_local,
        );

        function.instruction(&Instruction::I64Const((-1.0f64).to_bits() as i64));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(done_local));

        self.emit_array_iteration_to_object(receiver_payload_local, receiver_tag_local, function)?;

        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_length(
            receiver_payload_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_to_length_i64_from_value_locals(
            element_tag_local,
            element_payload_local,
            len_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_is_typed_array_i32(receiver_payload_local, receiver_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(typed_receiver_local));
        self.emit_load_typed_array_private_state(
            receiver_payload_local,
            typed_buffer_payload_local,
            typed_byte_offset_local,
            typed_stored_byte_length_local,
            typed_bytes_per_element_local,
            function,
        );
        self.emit_typed_array_witness(
            &typed_view,
            TypedArrayWitnessUse::ArrayLikeLengthSnapshot {
                length_local: len_local,
            },
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_to_length_i64_from_value_locals(
            element_tag_local,
            element_payload_local,
            len_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_array_iteration_nullish_receiver_throw_or_zero_length(
            receiver_tag_local,
            len_local,
            payload_local,
            tag_local,
            "Array.prototype.lastIndexOf called on null or undefined",
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(from_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Dynamic.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Else);
        self.emit_value_to_number_payload(from_tag_local, from_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(from_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(from_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncSatF64S);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::BrIf(1));

        self.emit_array_hole_prototype_clean_i32(
            receiver_payload_local,
            receiver_tag_local,
            index_local,
            len_local,
            array_hole_prototype_clean_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(array_hole_prototype_clean_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_retreat_to_previous_present_index(
            receiver_payload_local,
            index_local,
            len_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::BrIf(1));

        self.emit_index_to_flat_map_key_local(
            index_local,
            index_number_payload_local,
            key_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(has_property_local));
        function.instruction(&Instruction::LocalGet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_witness(
            &typed_view,
            TypedArrayWitnessUse::IntegerIndexedProperty {
                index_local,
                result_local: has_property_local,
            },
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_has_index_i32(
            receiver_payload_local,
            index_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_object_has_property_i32(
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(has_property_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_or_object_index_read_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_index_get_with_prototype(
            receiver_payload_local,
            index_local,
            receiver_payload_local,
            receiver_tag_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_typed_array_or_object_index_read_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_tagged_payload_equality_i32(
            element_tag_local,
            element_payload_local,
            search_tag_local,
            search_payload_local,
            function,
        )?;
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(done_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(done_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(array_hole_prototype_clean_local);
        self.release_temp_local(typed_bytes_per_element_local);
        self.release_temp_local(typed_stored_byte_length_local);
        self.release_temp_local(typed_byte_offset_local);
        self.release_temp_local(typed_buffer_payload_local);
        self.release_temp_local(typed_receiver_local);
        self.release_temp_local(done_local);
        self.release_temp_local(has_property_local);
        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        Ok(())
    }

    fn emit_array_iteration_to_object(
        &mut self,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_nullish_tagged_i32(receiver_tag_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_value_to_current_function_realm_object_locals(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_array_iteration_nullish_receiver_throw_or_zero_length(
        &mut self,
        receiver_tag_local: u32,
        len_local: u32,
        payload_local: u32,
        tag_local: u32,
        message: &str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            message,
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_object_prototype_to_string_result_from_locals(
        &mut self,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let tag_payload_local = self.reserve_temp_local();
        let is_array_local = self.reserve_temp_local();
        let brand_local = self.reserve_temp_local();
        let to_string_tag_key_local = self.reserve_temp_local();
        let custom_tag_payload_local = self.reserve_temp_local();
        let custom_tag_tag_local = self.reserve_temp_local();
        let prefix_local = self.reserve_temp_local();
        let suffix_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(COMPLETION_KIND_NORMAL));
        function.instruction(&Instruction::LocalSet(self.completion_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.completion_aux_local));

        function.instruction(&Instruction::I64Const(
            self.strings.payload("[object Object]"),
        ));
        function.instruction(&Instruction::LocalSet(tag_payload_local));

        for (kind, tag) in [
            (ValueKind::Undefined, "[object Undefined]"),
            (ValueKind::Null, "[object Null]"),
            (ValueKind::Boolean, "[object Boolean]"),
            (ValueKind::Number, "[object Number]"),
            (ValueKind::String, "[object String]"),
            (ValueKind::Symbol, "[object Symbol]"),
            (ValueKind::Object, "[object Object]"),
            (ValueKind::Array, "[object Array]"),
            (ValueKind::Function, "[object Function]"),
            (ValueKind::Arguments, "[object Arguments]"),
            (ValueKind::BigInt, "[object BigInt]"),
        ] {
            function.instruction(&Instruction::LocalGet(receiver_tag_local));
            function.instruction(&Instruction::I64Const(kind.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(self.strings.payload(tag)));
            function.instruction(&Instruction::LocalSet(tag_payload_local));
            function.instruction(&Instruction::End);
        }

        self.emit_is_array_i64(
            receiver_payload_local,
            receiver_tag_local,
            is_array_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(is_array_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("[object Array]"),
        ));
        function.instruction(&Instruction::LocalSet(tag_payload_local));
        function.instruction(&Instruction::End);

        self.emit_is_callable_i32(receiver_tag_local, receiver_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("[object Function]"),
        ));
        function.instruction(&Instruction::LocalSet(tag_payload_local));
        function.instruction(&Instruction::End);

        self.emit_is_heap_object_like_tag_i32(receiver_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            brand_local,
            function,
        );
        for (boxed_kind, tag) in [
            (BOXED_PRIMITIVE_KIND_BOOLEAN, "[object Boolean]"),
            (BOXED_PRIMITIVE_KIND_NUMBER, "[object Number]"),
            (BOXED_PRIMITIVE_KIND_STRING, "[object String]"),
            (BOXED_PRIMITIVE_KIND_SYMBOL, "[object Symbol]"),
            (BOXED_PRIMITIVE_KIND_BIGINT, "[object BigInt]"),
        ] {
            function.instruction(&Instruction::LocalGet(brand_local));
            function.instruction(&Instruction::I64Const(boxed_kind as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(self.strings.payload(tag)));
            function.instruction(&Instruction::LocalSet(tag_payload_local));
            function.instruction(&Instruction::End);
        }
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(brand_local));
        function.instruction(&Instruction::I64Const(OBJECT_INTERNAL_BRAND_ERROR as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("[object Error]"),
        ));
        function.instruction(&Instruction::LocalSet(tag_payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(brand_local));
        function.instruction(&Instruction::I64Const(OBJECT_INTERNAL_BRAND_DATE as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("[object Date]"),
        ));
        function.instruction(&Instruction::LocalSet(tag_payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(brand_local));
        function.instruction(&Instruction::I64Const(OBJECT_INTERNAL_BRAND_REGEXP as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("[object RegExp]"),
        ));
        function.instruction(&Instruction::LocalSet(tag_payload_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(to_string_tag_key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            to_string_tag_key_local,
            custom_tag_payload_local,
            custom_tag_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(custom_tag_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("[object ")));
        function.instruction(&Instruction::LocalSet(prefix_local));
        self.emit_concat_string_payloads_local(prefix_local, custom_tag_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(prefix_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("]")));
        function.instruction(&Instruction::LocalSet(suffix_local));
        self.emit_concat_string_payloads_local(prefix_local, suffix_local, function)?;
        function.instruction(&Instruction::LocalSet(tag_payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(tag_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_NORMAL));
        function.instruction(&Instruction::LocalSet(self.completion_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.completion_aux_local));

        self.release_temp_local(suffix_local);
        self.release_temp_local(prefix_local);
        self.release_temp_local(custom_tag_tag_local);
        self.release_temp_local(custom_tag_payload_local);
        self.release_temp_local(to_string_tag_key_local);
        self.release_temp_local(brand_local);
        self.release_temp_local(is_array_local);
        self.release_temp_local(tag_payload_local);
        Ok(())
    }

    pub(crate) fn compile_typed_array_prototype_to_string_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing TypedArray.prototype.toString receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing TypedArray.prototype.toString receiver tag",
            )
        })?;
        let object_payload_local = self.reserve_temp_local();
        let object_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let method_payload_local = self.reserve_temp_local();
        let method_tag_local = self.reserve_temp_local();
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();

        self.emit_value_to_current_function_realm_object_locals(
            receiver_payload_local,
            receiver_tag_local,
            object_payload_local,
            object_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload("join")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            object_payload_local,
            object_tag_local,
            object_payload_local,
            object_tag_local,
            key_local,
            method_payload_local,
            method_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);

        self.emit_is_callable_i32(method_tag_local, method_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_pre_evaluated_arg_vector(&[], argc_local, argv_local, function)?;
        self.emit_function_or_proxy_call_with_argv_leave_throw_completion(
            method_payload_local,
            method_tag_local,
            object_payload_local,
            object_tag_local,
            argc_local,
            argv_local,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::Else);
        self.emit_object_prototype_to_string_result_from_locals(
            object_payload_local,
            object_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(method_tag_local);
        self.release_temp_local(method_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(object_tag_local);
        self.release_temp_local(object_payload_local);
        Ok(())
    }

    pub(crate) fn compile_array_prototype_to_locale_string_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_to_locale_string_builtin(ToLocaleStringReceiverKind::ArrayLike, function)
    }

    pub(crate) fn compile_typed_array_prototype_to_locale_string_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_to_locale_string_builtin(ToLocaleStringReceiverKind::TypedArray, function)
    }

    fn emit_validate_to_locale_string_invocation(
        &mut self,
        receiver_kind: &ToLocaleStringReceiverKind,
        method: TaggedLocals,
        receiver: TaggedLocals,
        function: &mut Function,
    ) -> Result<ValidatedToLocaleStringInvocationLocals, EmitError> {
        self.emit_is_callable_i32(method.tag, method.payload, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        let error_message = match receiver_kind {
            ToLocaleStringReceiverKind::ArrayLike => {
                "Array.prototype.toLocaleString element method is not callable"
            }
            ToLocaleStringReceiverKind::TypedArray => {
                "TypedArray.prototype.toLocaleString element method is not callable"
            }
        };
        self.emit_throw_current_function_realm_type_error(
            error_message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        Ok(ValidatedToLocaleStringInvocationLocals { method, receiver })
    }

    fn emit_call_validated_to_locale_string_invocation(
        &mut self,
        invocation: ValidatedToLocaleStringInvocationLocals,
        result: TaggedLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let ValidatedToLocaleStringInvocationLocals { method, receiver } = invocation;

        self.emit_function_or_proxy_call_leave_throw_completion(
            method.payload,
            method.tag,
            receiver.payload,
            receiver.tag,
            &[],
            result.payload,
            result.tag,
            function,
        )
    }

    fn compile_to_locale_string_builtin(
        &mut self,
        receiver_kind: ToLocaleStringReceiverKind,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let method_name = match &receiver_kind {
            ToLocaleStringReceiverKind::ArrayLike => "Array.prototype.toLocaleString",
            ToLocaleStringReceiverKind::TypedArray => "TypedArray.prototype.toLocaleString",
        };
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(format!(
                "unsupported in lila wasm-aot first slice: missing {method_name} receiver"
            ))
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(format!(
                "unsupported in lila wasm-aot first slice: missing {method_name} receiver tag"
            ))
        })?;
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let original_element_payload_local = self.reserve_temp_local();
        let original_element_tag_local = self.reserve_temp_local();
        let method_payload_local = self.reserve_temp_local();
        let method_tag_local = self.reserve_temp_local();
        let element_string_local = self.reserve_temp_local();
        let joined_local = self.reserve_temp_local();
        let typed_receiver_local = self.reserve_temp_local();
        let typed_buffer_payload_local = self.reserve_temp_local();
        let typed_brand_local = self.reserve_temp_local();
        let typed_byte_offset_local = self.reserve_temp_local();
        let typed_byte_length_local = self.reserve_temp_local();
        let typed_bytes_per_element_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(joined_local));

        let typed_array_entry = match &receiver_kind {
            ToLocaleStringReceiverKind::ArrayLike => false,
            ToLocaleStringReceiverKind::TypedArray => true,
        };
        if typed_array_entry {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(typed_brand_local));
            function.instruction(&Instruction::LocalGet(receiver_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.load_i64_to_local_from_offset(
                receiver_payload_local,
                HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
                typed_brand_local,
                function,
            );
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalGet(typed_brand_local));
            function.instruction(&Instruction::I64Const(
                OBJECT_INTERNAL_BRAND_TYPED_ARRAY as i64,
            ));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_current_function_realm_type_error(
                "TypedArray.prototype.toLocaleString requires TypedArray",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);

            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(typed_receiver_local));
            self.emit_load_typed_array_private_state(
                receiver_payload_local,
                typed_buffer_payload_local,
                typed_byte_offset_local,
                typed_byte_length_local,
                typed_bytes_per_element_local,
                function,
            );
            let typed_view = TypedArrayViewLocals::new(
                receiver_payload_local,
                typed_buffer_payload_local,
                typed_byte_offset_local,
                typed_byte_length_local,
                typed_bytes_per_element_local,
            );
            self.emit_typed_array_witness(
                &typed_view,
                TypedArrayWitnessUse::ValidatedMethodEntry {
                    length_local: len_local,
                },
                function,
            )?;
        } else {
            self.compile_nullish_tagged_i32(receiver_tag_local, function)?;
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_current_function_realm_type_error(
                "Array.prototype.toLocaleString called on null or undefined",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);

            self.emit_array_iteration_to_object(
                receiver_payload_local,
                receiver_tag_local,
                function,
            )?;

            function.instruction(&Instruction::LocalGet(receiver_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::LocalGet(receiver_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.load_i64_to_local_from_offset(
                receiver_payload_local,
                HEAP_LEN_OFFSET,
                len_local,
                function,
            );
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(receiver_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::LocalGet(receiver_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_is_typed_array_i32(receiver_payload_local, receiver_tag_local, function);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(typed_receiver_local));
            self.emit_load_typed_array_private_state(
                receiver_payload_local,
                typed_buffer_payload_local,
                typed_byte_offset_local,
                typed_byte_length_local,
                typed_bytes_per_element_local,
                function,
            );
            let typed_view = TypedArrayViewLocals::new(
                receiver_payload_local,
                typed_buffer_payload_local,
                typed_byte_offset_local,
                typed_byte_length_local,
                typed_bytes_per_element_local,
            );
            self.emit_typed_array_witness(
                &typed_view,
                TypedArrayWitnessUse::ArrayLikeLengthSnapshot {
                    length_local: len_local,
                },
                function,
            )?;
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::I64Const(self.strings.payload("length")));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_object_read(
                receiver_payload_local,
                receiver_tag_local,
                receiver_payload_local,
                receiver_tag_local,
                key_local,
                element_payload_local,
                element_tag_local,
                function,
            )?;
            self.emit_return_current_completion_if_throw(function);
            self.emit_to_length_i64_from_value_locals(
                element_tag_local,
                element_payload_local,
                len_local,
                function,
            )?;
            self.emit_return_current_completion_if_throw(function);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload(",")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_concat_string_payloads_local(joined_local, key_local, function)?;
        function.instruction(&Instruction::LocalSet(joined_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_index_get_with_prototype(
            receiver_payload_local,
            index_local,
            receiver_payload_local,
            receiver_tag_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_read(
            receiver_payload_local,
            index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_or_object_index_read_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_index_to_flat_map_key_local(
            index_local,
            index_number_payload_local,
            key_local,
            function,
        )?;
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(element_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(element_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.compile_nullish_tagged_i32(element_tag_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(element_payload_local));
        function.instruction(&Instruction::LocalSet(original_element_payload_local));
        function.instruction(&Instruction::LocalGet(element_tag_local));
        function.instruction(&Instruction::LocalSet(original_element_tag_local));
        self.emit_array_iteration_to_object(element_payload_local, element_tag_local, function)?;
        function.instruction(&Instruction::LocalGet(element_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(element_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("toLocaleString"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            element_payload_local,
            element_tag_local,
            original_element_payload_local,
            original_element_tag_local,
            key_local,
            method_payload_local,
            method_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        let invocation = self.emit_validate_to_locale_string_invocation(
            &receiver_kind,
            TaggedLocals::new(method_payload_local, method_tag_local),
            TaggedLocals::new(original_element_payload_local, original_element_tag_local),
            function,
        )?;
        self.emit_call_validated_to_locale_string_invocation(
            invocation,
            TaggedLocals::new(element_payload_local, element_tag_local),
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_value_to_string_payload(element_payload_local, element_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(element_string_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::Else);
        self.emit_value_to_string_payload(element_payload_local, element_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(element_string_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);
        self.emit_concat_string_payloads_local(joined_local, element_string_local, function)?;
        function.instruction(&Instruction::LocalSet(joined_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(joined_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(typed_bytes_per_element_local);
        self.release_temp_local(typed_byte_length_local);
        self.release_temp_local(typed_byte_offset_local);
        self.release_temp_local(typed_brand_local);
        self.release_temp_local(typed_buffer_payload_local);
        self.release_temp_local(typed_receiver_local);
        self.release_temp_local(joined_local);
        self.release_temp_local(element_string_local);
        self.release_temp_local(method_tag_local);
        self.release_temp_local(method_payload_local);
        self.release_temp_local(original_element_tag_local);
        self.release_temp_local(original_element_payload_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        Ok(())
    }

    pub(crate) fn emit_object_has_array_index_key_in_range_i32(
        &mut self,
        object_payload_local: u32,
        start_index_local: u32,
        end_len_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let entry_index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let candidate_index_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));

        self.load_i64_to_local_from_offset(
            object_payload_local,
            HEAP_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            object_payload_local,
            HEAP_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(entry_index_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(entry_index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(entry_index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            key_local,
            function,
        );
        self.emit_string_index_0_to_4_or_minus_one(key_local, candidate_index_local, function);

        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::LocalGet(start_index_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::LocalGet(end_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(entry_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(candidate_index_local);
        self.release_temp_local(key_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(entry_index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
    }

    pub(crate) fn emit_array_hole_prototype_clean_i32(
        &mut self,
        array_payload_local: u32,
        array_tag_local: u32,
        index_local: u32,
        len_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        let prototype_payload_local = self.reserve_temp_local();
        let parent_prototype_local = self.reserve_temp_local();
        let prototype_has_index_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::LocalGet(array_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            array_payload_local,
            HEAP_PROTOTYPE_OFFSET,
            prototype_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_payload_local));
        function.instruction(&Instruction::GlobalGet(ARRAY_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            prototype_payload_local,
            HEAP_PROTOTYPE_OFFSET,
            parent_prototype_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(parent_prototype_local));
        function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(parent_prototype_local));
        self.load_i64_to_local_from_offset(
            parent_prototype_local,
            HEAP_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(self.scratch_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));

        self.load_i64_to_local_from_offset(
            prototype_payload_local,
            HEAP_LEN_OFFSET,
            self.scratch_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(self.scratch_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);

        self.emit_object_has_array_index_key_in_range_i32(
            prototype_payload_local,
            index_local,
            len_local,
            prototype_has_index_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_has_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_has_array_index_key_in_range_i32(
            parent_prototype_local,
            index_local,
            len_local,
            prototype_has_index_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_has_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(prototype_has_index_local);
        self.release_temp_local(parent_prototype_local);
        self.release_temp_local(prototype_payload_local);
    }

    fn emit_array_reduce_loop_entry(
        &self,
        direction: &ArrayReduceDirection,
        index_local: u32,
        len_local: u32,
        function: &mut Function,
    ) {
        match direction {
            ArrayReduceDirection::LeftToRight => {
                function.instruction(&Instruction::LocalGet(index_local));
                function.instruction(&Instruction::LocalGet(len_local));
                function.instruction(&Instruction::I64GeU);
                function.instruction(&Instruction::BrIf(1));
            }
            ArrayReduceDirection::RightToLeft => {
                function.instruction(&Instruction::LocalGet(index_local));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::BrIf(1));
                function.instruction(&Instruction::LocalGet(index_local));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Sub);
                function.instruction(&Instruction::LocalSet(index_local));
            }
        }
    }

    fn emit_array_reduce_advance(
        &self,
        direction: &ArrayReduceDirection,
        index_local: u32,
        function: &mut Function,
    ) {
        match direction {
            ArrayReduceDirection::LeftToRight => {
                function.instruction(&Instruction::LocalGet(index_local));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::LocalSet(index_local));
            }
            ArrayReduceDirection::RightToLeft => {}
        }
    }

    fn compile_array_like_reduce_builtin(
        &mut self,
        function: &mut Function,
        receiver_kind: ArrayCallbackReceiverKind,
        direction: ArrayReduceDirection,
    ) -> Result<(), EmitError> {
        let method_name = direction.method_name(&receiver_kind);
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(format!(
                "unsupported in lila wasm-aot first slice: missing {method_name} receiver"
            ))
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(format!(
                "unsupported in lila wasm-aot first slice: missing {method_name} receiver tag"
            ))
        })?;
        let callback_payload_local = self.reserve_temp_local();
        let callback_tag_local = self.reserve_temp_local();
        let undefined_this_payload_local = self.reserve_temp_local();
        let undefined_this_tag_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let has_property_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let accumulator_payload_local = self.reserve_temp_local();
        let accumulator_tag_local = self.reserve_temp_local();
        let found_local = self.reserve_temp_local();
        let index_payload_local = self.reserve_temp_local();
        let index_tag_local = self.reserve_temp_local();
        let typed_receiver_local = self.reserve_temp_local();
        let typed_brand_local = self.reserve_temp_local();
        let typed_buffer_payload_local = self.reserve_temp_local();
        let typed_byte_offset_local = self.reserve_temp_local();
        let typed_stored_byte_length_local = self.reserve_temp_local();
        let typed_bytes_per_element_local = self.reserve_temp_local();
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();
        let typed_view = TypedArrayViewLocals::new(
            receiver_payload_local,
            typed_buffer_payload_local,
            typed_byte_offset_local,
            typed_stored_byte_length_local,
            typed_bytes_per_element_local,
        );

        match &receiver_kind {
            ArrayCallbackReceiverKind::ArrayLike => {
                // Generic methods begin with ToObject and a single
                // LengthOfArrayLike snapshot; all later indexed operations remain
                // fully observable.
                self.emit_value_to_object_locals(
                    receiver_payload_local,
                    receiver_tag_local,
                    receiver_payload_local,
                    receiver_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(typed_receiver_local));
                function.instruction(&Instruction::LocalGet(receiver_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.load_i64_to_local_from_offset(
                    receiver_payload_local,
                    HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
                    typed_brand_local,
                    function,
                );
                function.instruction(&Instruction::LocalGet(typed_brand_local));
                function.instruction(&Instruction::I64Const(
                    OBJECT_INTERNAL_BRAND_TYPED_ARRAY as i64,
                ));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(typed_receiver_local));
                self.emit_load_typed_array_private_state(
                    receiver_payload_local,
                    typed_buffer_payload_local,
                    typed_byte_offset_local,
                    typed_stored_byte_length_local,
                    typed_bytes_per_element_local,
                    function,
                );
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);

                function.instruction(&Instruction::LocalGet(receiver_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.load_i64_to_local_from_offset(
                    receiver_payload_local,
                    HEAP_LEN_OFFSET,
                    len_local,
                    function,
                );
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::LocalGet(receiver_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_arguments_length(
                    receiver_payload_local,
                    element_payload_local,
                    element_tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(
                    element_payload_local,
                    element_tag_local,
                    function,
                )?;
                self.emit_to_length_i64_from_value_locals(
                    element_tag_local,
                    element_payload_local,
                    len_local,
                    function,
                )?;
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::I64Const(self.strings.payload("length")));
                function.instruction(&Instruction::LocalSet(key_local));
                self.emit_object_read(
                    receiver_payload_local,
                    receiver_tag_local,
                    receiver_payload_local,
                    receiver_tag_local,
                    key_local,
                    element_payload_local,
                    element_tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(
                    element_payload_local,
                    element_tag_local,
                    function,
                )?;
                self.emit_to_length_i64_from_value_locals(
                    element_tag_local,
                    element_payload_local,
                    len_local,
                    function,
                )?;
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
            }
            ArrayCallbackReceiverKind::TypedArray => {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(typed_brand_local));
                function.instruction(&Instruction::LocalGet(receiver_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.load_i64_to_local_from_offset(
                    receiver_payload_local,
                    HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
                    typed_brand_local,
                    function,
                );
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::LocalGet(typed_brand_local));
                function.instruction(&Instruction::I64Const(
                    OBJECT_INTERNAL_BRAND_TYPED_ARRAY as i64,
                ));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_throw_current_function_realm_type_error(
                    direction.typed_array_receiver_error(),
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_return_current_completion(function);
                function.instruction(&Instruction::End);

                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(typed_receiver_local));
                self.emit_load_typed_array_private_state(
                    receiver_payload_local,
                    typed_buffer_payload_local,
                    typed_byte_offset_local,
                    typed_stored_byte_length_local,
                    typed_bytes_per_element_local,
                    function,
                );
                self.emit_typed_array_witness(
                    &typed_view,
                    TypedArrayWitnessUse::ValidatedMethodEntry {
                        length_local: len_local,
                    },
                    function,
                )?;
            }
        }

        // IsCallable follows the length observation, and includes callable
        // proxies rather than only direct Function-tag values.
        self.emit_builtin_arg_to_locals(0, callback_payload_local, callback_tag_local, function);
        self.emit_is_callable_i32(callback_tag_local, callback_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_current_function_realm_type_error(
            direction.callback_not_callable_error(&receiver_kind),
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_this_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_this_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(found_local));
        match &direction {
            ArrayReduceDirection::LeftToRight => {
                function.instruction(&Instruction::I64Const(0));
            }
            ArrayReduceDirection::RightToLeft => {
                function.instruction(&Instruction::LocalGet(len_local));
            }
        }
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_builtin_arg_to_locals(
            1,
            accumulator_payload_local,
            accumulator_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::End);

        // First pass when no initial value: HasProperty then Get in direction.
        function.instruction(&Instruction::LocalGet(found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.emit_array_reduce_loop_entry(&direction, index_local, len_local, function);
        match &receiver_kind {
            ArrayCallbackReceiverKind::ArrayLike => {
                self.emit_index_to_flat_map_key_local(
                    index_local,
                    index_payload_local,
                    key_local,
                    function,
                )?;
            }
            ArrayCallbackReceiverKind::TypedArray => {}
        }
        match &receiver_kind {
            ArrayCallbackReceiverKind::ArrayLike => {
                self.emit_array_reduce_has_property(
                    receiver_payload_local,
                    receiver_tag_local,
                    typed_receiver_local,
                    &typed_view,
                    index_local,
                    key_local,
                    has_property_local,
                    function,
                )?;
            }
            ArrayCallbackReceiverKind::TypedArray => {
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(has_property_local));
            }
        }
        function.instruction(&Instruction::LocalGet(has_property_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_reduce_get_index(
            receiver_payload_local,
            receiver_tag_local,
            typed_receiver_local,
            index_local,
            key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(element_payload_local));
        function.instruction(&Instruction::LocalSet(accumulator_payload_local));
        function.instruction(&Instruction::LocalGet(element_tag_local));
        function.instruction(&Instruction::LocalSet(accumulator_tag_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(found_local));
        self.emit_array_reduce_advance(&direction, index_local, function);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.emit_array_reduce_advance(&direction, index_local, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            match &receiver_kind {
                ArrayCallbackReceiverKind::ArrayLike => {
                    "Reduce of empty array with no initial value"
                }
                ArrayCallbackReceiverKind::TypedArray => {
                    "Reduce of empty typed array with no initial value"
                }
            },
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        // Main pass: the length remains fixed, but property existence and
        // retrieval are re-evaluated on every iteration.
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.emit_array_reduce_loop_entry(&direction, index_local, len_local, function);
        match &receiver_kind {
            ArrayCallbackReceiverKind::ArrayLike => {
                self.emit_index_to_flat_map_key_local(
                    index_local,
                    index_payload_local,
                    key_local,
                    function,
                )?;
            }
            ArrayCallbackReceiverKind::TypedArray => {}
        }
        match &receiver_kind {
            ArrayCallbackReceiverKind::ArrayLike => {
                self.emit_array_reduce_has_property(
                    receiver_payload_local,
                    receiver_tag_local,
                    typed_receiver_local,
                    &typed_view,
                    index_local,
                    key_local,
                    has_property_local,
                    function,
                )?;
            }
            ArrayCallbackReceiverKind::TypedArray => {
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(has_property_local));
            }
        }
        function.instruction(&Instruction::LocalGet(has_property_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_reduce_get_index(
            receiver_payload_local,
            receiver_tag_local,
            typed_receiver_local,
            index_local,
            key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(index_tag_local));
        self.emit_pre_evaluated_arg_vector(
            &[
                (accumulator_payload_local, accumulator_tag_local),
                (element_payload_local, element_tag_local),
                (index_payload_local, index_tag_local),
                (receiver_payload_local, receiver_tag_local),
            ],
            argc_local,
            argv_local,
            function,
        )?;
        self.emit_function_or_proxy_call_with_argv_leave_throw_completion(
            callback_payload_local,
            callback_tag_local,
            undefined_this_payload_local,
            undefined_this_tag_local,
            argc_local,
            argv_local,
            accumulator_payload_local,
            accumulator_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            accumulator_payload_local,
            accumulator_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_array_reduce_advance(&direction, index_local, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(accumulator_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(accumulator_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(typed_bytes_per_element_local);
        self.release_temp_local(typed_stored_byte_length_local);
        self.release_temp_local(typed_byte_offset_local);
        self.release_temp_local(typed_buffer_payload_local);
        self.release_temp_local(typed_brand_local);
        self.release_temp_local(typed_receiver_local);
        self.release_temp_local(index_tag_local);
        self.release_temp_local(index_payload_local);
        self.release_temp_local(found_local);
        self.release_temp_local(accumulator_tag_local);
        self.release_temp_local(accumulator_payload_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(has_property_local);
        self.release_temp_local(key_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(undefined_this_tag_local);
        self.release_temp_local(undefined_this_payload_local);
        self.release_temp_local(callback_tag_local);
        self.release_temp_local(callback_payload_local);
        Ok(())
    }

    pub(super) fn compile_array_reduce_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_array_like_reduce_builtin(
            function,
            ArrayCallbackReceiverKind::ArrayLike,
            ArrayReduceDirection::LeftToRight,
        )
    }

    pub(super) fn compile_array_reduce_right_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_array_like_reduce_builtin(
            function,
            ArrayCallbackReceiverKind::ArrayLike,
            ArrayReduceDirection::RightToLeft,
        )
    }

    pub(super) fn compile_typed_array_reduce_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_array_like_reduce_builtin(
            function,
            ArrayCallbackReceiverKind::TypedArray,
            ArrayReduceDirection::LeftToRight,
        )
    }

    pub(super) fn compile_typed_array_reduce_right_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_array_like_reduce_builtin(
            function,
            ArrayCallbackReceiverKind::TypedArray,
            ArrayReduceDirection::RightToLeft,
        )
    }

    fn emit_array_reduce_has_property(
        &mut self,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        typed_receiver_local: u32,
        typed_view: &TypedArrayViewLocals,
        index_local: u32,
        key_local: u32,
        has_property_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let prototype_payload_local = self.reserve_temp_local();
        let prototype_tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_has_index_i32(
            receiver_payload_local,
            index_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(has_property_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_PROTOTYPE_OFFSET,
            prototype_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(prototype_tag_local));
        self.emit_object_own_property_present(
            prototype_payload_local,
            prototype_tag_local,
            key_local,
            has_property_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(has_property_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_has_property_i32(
            prototype_payload_local,
            prototype_tag_local,
            key_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_witness(
            typed_view,
            TypedArrayWitnessUse::IntegerIndexedProperty {
                index_local: index_local,
                result_local: has_property_local,
            },
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_object_has_property_i32(
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(prototype_tag_local);
        self.release_temp_local(prototype_payload_local);
        Ok(())
    }

    fn emit_array_reduce_get_index(
        &mut self,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        typed_receiver_local: u32,
        index_local: u32,
        key_local: u32,
        element_payload_local: u32,
        element_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let own_property_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();
        let prototype_tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_index_get_with_prototype(
            receiver_payload_local,
            index_local,
            receiver_payload_local,
            receiver_tag_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_has_index_i32(
            receiver_payload_local,
            index_local,
            own_property_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(own_property_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_PROTOTYPE_OFFSET,
            prototype_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(element_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(element_tag_local));
        function.instruction(&Instruction::Else);
        self.emit_object_read(
            prototype_payload_local,
            prototype_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_arguments_read(
            receiver_payload_local,
            index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_or_object_index_read_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(prototype_tag_local);
        self.release_temp_local(prototype_payload_local);
        self.release_temp_local(own_property_local);
        Ok(())
    }

    pub(super) fn compile_array_prototype_for_each_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_array_like_for_each_builtin(function, ArrayCallbackReceiverKind::ArrayLike)
    }

    pub(super) fn compile_typed_array_prototype_for_each_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_array_like_for_each_builtin(function, ArrayCallbackReceiverKind::TypedArray)
    }

    fn compile_array_like_for_each_builtin(
        &mut self,
        function: &mut Function,
        receiver_kind: ArrayCallbackReceiverKind,
    ) -> Result<(), EmitError> {
        let method_name = match &receiver_kind {
            ArrayCallbackReceiverKind::ArrayLike => "Array.prototype.forEach",
            ArrayCallbackReceiverKind::TypedArray => "TypedArray.prototype.forEach",
        };
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(format!(
                "unsupported in lila wasm-aot first slice: missing {method_name} receiver",
            ))
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(format!(
                "unsupported in lila wasm-aot first slice: missing {method_name} receiver tag",
            ))
        })?;
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let index_payload_local = self.reserve_temp_local();
        let index_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let has_property_local = self.reserve_temp_local();
        let this_arg_payload_local = self.reserve_temp_local();
        let this_arg_tag_local = self.reserve_temp_local();
        let callback_payload_local = self.reserve_temp_local();
        let callback_tag_local = self.reserve_temp_local();
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();
        let typed_receiver_local = self.reserve_temp_local();
        let typed_brand_local = self.reserve_temp_local();
        let typed_buffer_payload_local = self.reserve_temp_local();
        let typed_byte_offset_local = self.reserve_temp_local();
        let typed_stored_byte_length_local = self.reserve_temp_local();
        let typed_bytes_per_element_local = self.reserve_temp_local();
        let array_hole_prototype_clean_local = self.reserve_temp_local();
        let typed_view = TypedArrayViewLocals::new(
            receiver_payload_local,
            typed_buffer_payload_local,
            typed_byte_offset_local,
            typed_stored_byte_length_local,
            typed_bytes_per_element_local,
        );

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(typed_receiver_local));

        self.emit_builtin_arg_to_locals(0, callback_payload_local, callback_tag_local, function);
        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_builtin_arg_to_locals(1, this_arg_payload_local, this_arg_tag_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(this_arg_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(this_arg_tag_local));
        function.instruction(&Instruction::End);

        match &receiver_kind {
            ArrayCallbackReceiverKind::ArrayLike => {
                self.emit_array_iteration_to_object(
                    receiver_payload_local,
                    receiver_tag_local,
                    function,
                )?;

                function.instruction(&Instruction::LocalGet(receiver_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::LocalGet(receiver_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I32Or);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.load_i64_to_local_from_offset(
                    receiver_payload_local,
                    HEAP_LEN_OFFSET,
                    len_local,
                    function,
                );
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::LocalGet(receiver_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::LocalGet(receiver_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I32Or);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_is_typed_array_i32(receiver_payload_local, receiver_tag_local, function);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(typed_receiver_local));
                self.emit_load_typed_array_private_state(
                    receiver_payload_local,
                    typed_buffer_payload_local,
                    typed_byte_offset_local,
                    typed_stored_byte_length_local,
                    typed_bytes_per_element_local,
                    function,
                );
                self.emit_typed_array_witness(
                    &typed_view,
                    TypedArrayWitnessUse::ArrayLikeLengthSnapshot {
                        length_local: len_local,
                    },
                    function,
                )?;
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::I64Const(self.strings.payload("length")));
                function.instruction(&Instruction::LocalSet(key_local));
                self.emit_object_read(
                    receiver_payload_local,
                    receiver_tag_local,
                    receiver_payload_local,
                    receiver_tag_local,
                    key_local,
                    element_payload_local,
                    element_tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(
                    element_payload_local,
                    element_tag_local,
                    function,
                )?;
                self.emit_to_length_i64_from_value_locals(
                    element_tag_local,
                    element_payload_local,
                    len_local,
                    function,
                )?;
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::Else);
                self.emit_array_iteration_nullish_receiver_throw_or_zero_length(
                    receiver_tag_local,
                    len_local,
                    self.result_local,
                    self.result_tag_local,
                    "Array.prototype.forEach called on null or undefined",
                    function,
                )?;
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
            }
            ArrayCallbackReceiverKind::TypedArray => {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(typed_brand_local));
                function.instruction(&Instruction::LocalGet(receiver_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.load_i64_to_local_from_offset(
                    receiver_payload_local,
                    HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
                    typed_brand_local,
                    function,
                );
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::LocalGet(typed_brand_local));
                function.instruction(&Instruction::I64Const(
                    OBJECT_INTERNAL_BRAND_TYPED_ARRAY as i64,
                ));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_throw_current_function_realm_type_error(
                    "TypedArray.prototype.forEach requires a TypedArray",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_return_current_completion(function);
                function.instruction(&Instruction::End);

                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(typed_receiver_local));
                self.emit_load_typed_array_private_state(
                    receiver_payload_local,
                    typed_buffer_payload_local,
                    typed_byte_offset_local,
                    typed_stored_byte_length_local,
                    typed_bytes_per_element_local,
                    function,
                );
                self.emit_typed_array_witness(
                    &typed_view,
                    TypedArrayWitnessUse::ValidatedMethodEntry {
                        length_local: len_local,
                    },
                    function,
                )?;
            }
        }

        self.emit_is_callable_i32(callback_tag_local, callback_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            match &receiver_kind {
                ArrayCallbackReceiverKind::ArrayLike => {
                    "Array.prototype.forEach callback is not callable"
                }
                ArrayCallbackReceiverKind::TypedArray => {
                    "TypedArray.prototype.forEach callback is not callable"
                }
            },
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        self.emit_array_hole_prototype_clean_i32(
            receiver_payload_local,
            receiver_tag_local,
            index_local,
            len_local,
            array_hole_prototype_clean_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(array_hole_prototype_clean_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_advance_to_next_present_index(
            receiver_payload_local,
            index_local,
            len_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        match &receiver_kind {
            ArrayCallbackReceiverKind::ArrayLike => {
                self.emit_index_to_flat_map_key_local(
                    index_local,
                    index_payload_local,
                    key_local,
                    function,
                )?;
            }
            ArrayCallbackReceiverKind::TypedArray => {}
        }
        match &receiver_kind {
            ArrayCallbackReceiverKind::ArrayLike => {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(has_property_local));
                function.instruction(&Instruction::LocalGet(receiver_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_arguments_has_index_i32(
                    receiver_payload_local,
                    index_local,
                    has_property_local,
                    function,
                )?;
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::LocalGet(typed_receiver_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_typed_array_witness(
                    &typed_view,
                    TypedArrayWitnessUse::IntegerIndexedProperty {
                        index_local: index_local,
                        result_local: has_property_local,
                    },
                    function,
                )?;
                function.instruction(&Instruction::Else);
                self.emit_object_has_property_i32(
                    receiver_payload_local,
                    receiver_tag_local,
                    key_local,
                    has_property_local,
                    function,
                )?;
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
            }
            ArrayCallbackReceiverKind::TypedArray => {
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(has_property_local));
            }
        }

        function.instruction(&Instruction::LocalGet(has_property_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_index_get_with_prototype(
            receiver_payload_local,
            index_local,
            receiver_payload_local,
            receiver_tag_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_read(
            receiver_payload_local,
            index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_or_object_index_read_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_propagate_throw_from_locals_if_needed(
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(index_tag_local));
        self.emit_pre_evaluated_arg_vector(
            &[
                (element_payload_local, element_tag_local),
                (index_payload_local, index_tag_local),
                (receiver_payload_local, receiver_tag_local),
            ],
            argc_local,
            argv_local,
            function,
        )?;
        self.emit_function_or_proxy_call_with_argv_leave_throw_completion(
            callback_payload_local,
            callback_tag_local,
            this_arg_payload_local,
            this_arg_tag_local,
            argc_local,
            argv_local,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(array_hole_prototype_clean_local);
        self.release_temp_local(typed_bytes_per_element_local);
        self.release_temp_local(typed_stored_byte_length_local);
        self.release_temp_local(typed_byte_offset_local);
        self.release_temp_local(typed_buffer_payload_local);
        self.release_temp_local(typed_brand_local);
        self.release_temp_local(typed_receiver_local);
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(callback_tag_local);
        self.release_temp_local(callback_payload_local);
        self.release_temp_local(this_arg_tag_local);
        self.release_temp_local(this_arg_payload_local);
        self.release_temp_local(has_property_local);
        self.release_temp_local(key_local);
        self.release_temp_local(index_tag_local);
        self.release_temp_local(index_payload_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        Ok(())
    }

    pub(crate) fn emit_alloc_array_payload_with_length(
        &mut self,
        len_local: u32,
        payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if let Some(array_alloc_function_index) = self.array_alloc_function_index {
            let buffer_local = self.reserve_temp_local();
            function.instruction(&Instruction::LocalGet(len_local));
            function.instruction(&Instruction::Call(array_alloc_function_index));
            function.instruction(&Instruction::LocalSet(buffer_local));
            function.instruction(&Instruction::LocalSet(payload_local));
            self.release_temp_local(buffer_local);
            return Ok(());
        }
        let array_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let cap_local = self.reserve_temp_local();
        let size_local = self.reserve_temp_local();
        self.emit_heap_alloc_const(HEAP_ARRAY_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(array_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(MIN_HEAP_CAPACITY as i64));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(cap_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(size_local));
        self.emit_heap_alloc_from_local(size_local, function)?;
        function.instruction(&Instruction::LocalSet(buffer_local));
        self.store_i64_local_at_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.store_i64_local_at_offset(array_local, HEAP_LEN_OFFSET, len_local, function);
        self.store_i64_local_at_offset(array_local, HEAP_CAP_OFFSET, cap_local, function);
        function.instruction(&Instruction::GlobalGet(ARRAY_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.store_i64_local_at_offset(
            array_local,
            HEAP_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_PROTOTYPE_TAG_OFFSET,
            ValueKind::Array.tag() as u64,
            function,
        );
        self.emit_init_array_exotic_slots(array_local, function);
        function.instruction(&Instruction::LocalGet(array_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        self.release_temp_local(size_local);
        self.release_temp_local(cap_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(array_local);
        Ok(())
    }

    pub(crate) fn emit_alloc_array_payload_with_length_in_current_function_realm(
        &mut self,
        len_local: u32,
        payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_alloc_array_payload_with_length(len_local, payload_local, function)?;
        let prototype = self.emit_load_current_function_realm_array_prototype(function);
        self.emit_install_current_function_realm_array_prototype(
            payload_local,
            prototype,
            function,
        );
        Ok(())
    }

    pub(crate) fn emit_array_like_snapshot_payload(
        &mut self,
        input_payload_local: u32,
        input_tag_local: u32,
        payload_local: u32,
        wrong_type_message: &str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let len_local = self.reserve_temp_local();
        let dst_payload_local = self.reserve_temp_local();
        let dst_buffer_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();

        self.emit_is_heap_object_like_tag_i32(input_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            wrong_type_message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            input_payload_local,
            input_tag_local,
            input_payload_local,
            input_tag_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_to_length_i64_from_value_locals(
            value_tag_local,
            value_payload_local,
            len_local,
            function,
        )?;
        self.emit_alloc_array_payload_with_length(len_local, dst_payload_local, function)?;
        self.load_i64_to_local_from_offset(
            dst_payload_local,
            HEAP_PTR_OFFSET,
            dst_buffer_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_index_to_flat_map_key_local(
            index_local,
            index_number_payload_local,
            key_local,
            function,
        )?;
        self.emit_object_read(
            input_payload_local,
            input_tag_local,
            input_payload_local,
            input_tag_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(dst_buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_ARRAY_TAG_OFFSET,
            value_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_ARRAY_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        self.store_i64_const_at_offset(
            entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            ARRAY_DESCRIPTOR_NORMAL_DATA,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(dst_payload_local));
        function.instruction(&Instruction::LocalSet(payload_local));

        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(dst_buffer_local);
        self.release_temp_local(dst_payload_local);
        self.release_temp_local(len_local);
        Ok(())
    }
}
