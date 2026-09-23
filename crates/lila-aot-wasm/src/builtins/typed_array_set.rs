//! %TypedArray%.prototype.set and ordered same-kind byte copies.

use super::super::*;
use super::binary_data::{TypedArrayViewLocals, TypedArrayWitnessUse};

impl FunctionBuilder<'_> {
    pub(super) fn compile_typed_array_prototype_set_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing TypedArray.prototype.set receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing TypedArray.prototype.set receiver tag",
            )
        })?;
        let receiver_brand_local = self.reserve_temp_local();
        let receiver_buffer_local = self.reserve_temp_local();
        let receiver_byte_offset_local = self.reserve_temp_local();
        let receiver_stored_byte_length_local = self.reserve_temp_local();
        let receiver_bytes_per_element_local = self.reserve_temp_local();
        let receiver_length_local = self.reserve_temp_local();
        let source_payload_local = self.reserve_temp_local();
        let source_tag_local = self.reserve_temp_local();
        let source_brand_local = self.reserve_temp_local();
        let source_buffer_local = self.reserve_temp_local();
        let source_byte_offset_local = self.reserve_temp_local();
        let source_stored_byte_length_local = self.reserve_temp_local();
        let source_bytes_per_element_local = self.reserve_temp_local();
        let source_length_local = self.reserve_temp_local();
        let offset_payload_local = self.reserve_temp_local();
        let offset_tag_local = self.reserve_temp_local();
        let offset_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let target_index_local = self.reserve_temp_local();
        let temporary_size_local = self.reserve_temp_local();
        let temporary_local = self.reserve_temp_local();
        let temporary_entry_local = self.reserve_temp_local();
        let receiver_element_kind_local = self.reserve_temp_local();
        let source_element_kind_local = self.reserve_temp_local();
        let receiver_buffer_address_local = self.reserve_temp_local();
        let source_buffer_address_local = self.reserve_temp_local();
        let receiver_address_local = self.reserve_temp_local();
        let source_address_local = self.reserve_temp_local();

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
            "TypedArray.prototype.set requires TypedArray",
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
            receiver_stored_byte_length_local,
            receiver_bytes_per_element_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET,
            receiver_element_kind_local,
            function,
        );
        let receiver_view = TypedArrayViewLocals::new(
            receiver_payload_local,
            receiver_buffer_local,
            receiver_byte_offset_local,
            receiver_stored_byte_length_local,
            receiver_bytes_per_element_local,
        );
        self.emit_typed_array_witness(
            &receiver_view,
            TypedArrayWitnessUse::ValidatedMethodEntry {
                length_local: receiver_length_local,
            },
            function,
        )?;

        self.emit_builtin_arg_to_locals(1, offset_payload_local, offset_tag_local, function);
        self.emit_to_index_i64_from_value_locals(
            offset_tag_local,
            offset_payload_local,
            offset_local,
            "TypedArray.prototype.set offset is out of range",
            function,
        )?;
        self.emit_typed_array_witness(
            &receiver_view,
            TypedArrayWitnessUse::ValidatedMethodEntry {
                length_local: receiver_length_local,
            },
            function,
        )?;
        self.emit_builtin_arg_to_locals(0, source_payload_local, source_tag_local, function);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(source_brand_local));
        function.instruction(&Instruction::LocalGet(source_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            source_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            source_brand_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(source_brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TYPED_ARRAY as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));

        self.emit_load_typed_array_private_state(
            source_payload_local,
            source_buffer_local,
            source_byte_offset_local,
            source_stored_byte_length_local,
            source_bytes_per_element_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            source_payload_local,
            HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET,
            source_element_kind_local,
            function,
        );
        let source_view = TypedArrayViewLocals::new(
            source_payload_local,
            source_buffer_local,
            source_byte_offset_local,
            source_stored_byte_length_local,
            source_bytes_per_element_local,
        );
        self.emit_typed_array_witness(
            &source_view,
            TypedArrayWitnessUse::ValidatedMethodEntry {
                length_local: source_length_local,
            },
            function,
        )?;

        function.instruction(&Instruction::LocalGet(source_length_local));
        function.instruction(&Instruction::LocalGet(receiver_length_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "TypedArray.prototype.set source is too large",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(offset_local));
        function.instruction(&Instruction::LocalGet(receiver_length_local));
        function.instruction(&Instruction::LocalGet(source_length_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "TypedArray.prototype.set source is too large",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_typed_array_bigint_element_kind_i32(receiver_element_kind_local, function);
        self.emit_typed_array_bigint_element_kind_i32(source_element_kind_local, function);
        function.instruction(&Instruction::I32Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "TypedArray.prototype.set source and target content types differ",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(receiver_element_kind_local));
        function.instruction(&Instruction::LocalGet(source_element_kind_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_array_buffer_data(
            receiver_buffer_local,
            receiver_buffer_address_local,
            function,
        );
        self.emit_load_array_buffer_data(
            source_buffer_local,
            source_buffer_address_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(source_buffer_address_local));
        function.instruction(&Instruction::LocalGet(source_byte_offset_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(source_address_local));
        function.instruction(&Instruction::LocalGet(receiver_buffer_address_local));
        function.instruction(&Instruction::LocalGet(receiver_byte_offset_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(offset_local));
        function.instruction(&Instruction::LocalGet(receiver_bytes_per_element_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(receiver_address_local));
        function.instruction(&Instruction::LocalGet(source_length_local));
        function.instruction(&Instruction::LocalGet(source_bytes_per_element_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(temporary_size_local));
        // The backing-store address also identifies aliased SharedArrayBuffer
        // wrappers. Snapshot the source before any writes to the same block.
        function.instruction(&Instruction::LocalGet(source_buffer_address_local));
        function.instruction(&Instruction::LocalGet(receiver_buffer_address_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_heap_alloc_from_local(temporary_size_local, function)?;
        function.instruction(&Instruction::LocalSet(temporary_local));
        self.emit_typed_array_copy_bytes_in_order(
            source_address_local,
            temporary_local,
            temporary_size_local,
            self.buffer_memory_index(),
            0,
            function,
        );
        self.emit_typed_array_copy_bytes_in_order(
            temporary_local,
            receiver_address_local,
            temporary_size_local,
            0,
            self.buffer_memory_index(),
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_typed_array_copy_bytes_in_order(
            source_address_local,
            receiver_address_local,
            temporary_size_local,
            self.buffer_memory_index(),
            self.buffer_memory_index(),
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);

        function.instruction(&Instruction::LocalGet(source_length_local));
        function.instruction(&Instruction::I64Const(16));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(temporary_size_local));
        self.emit_heap_alloc_from_local(temporary_size_local, function)?;
        function.instruction(&Instruction::LocalSet(temporary_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(source_length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_typed_array_or_object_index_read_from_locals(
            source_payload_local,
            source_tag_local,
            index_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(temporary_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(16));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(temporary_entry_local));
        function.instruction(&Instruction::LocalGet(temporary_entry_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::I64Store(Self::memarg64(0)));
        function.instruction(&Instruction::LocalGet(temporary_entry_local));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Store(Self::memarg64(0)));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(source_length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(temporary_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(16));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(temporary_entry_local));
        function.instruction(&Instruction::LocalGet(temporary_entry_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(Self::memarg64(0)));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::LocalGet(temporary_entry_local));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(Self::memarg64(0)));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        function.instruction(&Instruction::LocalGet(offset_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(target_index_local));
        self.emit_typed_array_element_write_from_locals(
            receiver_payload_local,
            target_index_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);

        self.emit_value_to_current_function_realm_object_locals(
            source_payload_local,
            source_tag_local,
            source_payload_local,
            source_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            source_payload_local,
            source_tag_local,
            source_payload_local,
            source_tag_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_to_length_i64_from_value_locals(
            value_tag_local,
            value_payload_local,
            source_length_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(source_length_local));
        function.instruction(&Instruction::LocalGet(receiver_length_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "TypedArray.prototype.set source is too large",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(offset_local));
        function.instruction(&Instruction::LocalGet(receiver_length_local));
        function.instruction(&Instruction::LocalGet(source_length_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "TypedArray.prototype.set source is too large",
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
        function.instruction(&Instruction::LocalGet(source_length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_typed_array_or_object_index_read_from_locals(
            source_payload_local,
            source_tag_local,
            index_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(offset_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(target_index_local));
        self.emit_typed_array_element_write_from_locals(
            receiver_payload_local,
            target_index_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(source_address_local);
        self.release_temp_local(receiver_address_local);
        self.release_temp_local(source_buffer_address_local);
        self.release_temp_local(receiver_buffer_address_local);
        self.release_temp_local(source_element_kind_local);
        self.release_temp_local(receiver_element_kind_local);
        self.release_temp_local(temporary_entry_local);
        self.release_temp_local(temporary_local);
        self.release_temp_local(temporary_size_local);
        self.release_temp_local(target_index_local);
        self.release_temp_local(index_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(offset_local);
        self.release_temp_local(offset_tag_local);
        self.release_temp_local(offset_payload_local);
        self.release_temp_local(source_length_local);
        self.release_temp_local(source_bytes_per_element_local);
        self.release_temp_local(source_stored_byte_length_local);
        self.release_temp_local(source_byte_offset_local);
        self.release_temp_local(source_buffer_local);
        self.release_temp_local(source_brand_local);
        self.release_temp_local(source_tag_local);
        self.release_temp_local(source_payload_local);
        self.release_temp_local(receiver_length_local);
        self.release_temp_local(receiver_bytes_per_element_local);
        self.release_temp_local(receiver_stored_byte_length_local);
        self.release_temp_local(receiver_byte_offset_local);
        self.release_temp_local(receiver_buffer_local);
        self.release_temp_local(receiver_brand_local);
        Ok(())
    }

    pub(super) fn emit_typed_array_copy_bytes_in_order(
        &mut self,
        source_address_local: u32,
        target_address_local: u32,
        byte_count_local: u32,
        source_memory_index: u32,
        target_memory_index: u32,
        function: &mut Function,
    ) {
        let index_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(byte_count_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(target_address_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(source_address_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8_in(
            source_memory_index,
            0,
        )));
        function.instruction(&Instruction::I32Store8(Self::memarg8_in(
            target_memory_index,
            0,
        )));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(index_local);
    }
}
