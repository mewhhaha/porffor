//! %TypedArray%.prototype.fill: one value conversion between two witnesses.

use super::super::*;
use super::binary_data::{TypedArrayViewLocals, TypedArrayWitnessUse};

impl FunctionBuilder<'_> {
    pub(super) fn compile_typed_array_prototype_fill_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver = self.this_payload_local.expect("builtin receiver payload");
        let receiver_tag = self.this_tag_local.expect("builtin receiver tag");
        let buffer = self.reserve_temp_local();
        let buffer_flags = self.reserve_temp_local();
        let byte_offset = self.reserve_temp_local();
        let stored_byte_length = self.reserve_temp_local();
        let bytes_per_element = self.reserve_temp_local();
        let element_kind = self.reserve_temp_local();
        let initial_length = self.reserve_temp_local();
        let current_length = self.reserve_temp_local();
        let value_payload = self.reserve_temp_local();
        let value_tag = self.reserve_temp_local();
        let element_payload = self.reserve_temp_local();
        let stored_element_payload = self.reserve_temp_local();
        let argument_payload = self.reserve_temp_local();
        let argument_tag = self.reserve_temp_local();
        let start = self.reserve_temp_local();
        let end = self.reserve_temp_local();
        let index = self.reserve_temp_local();
        let buffer_address = self.reserve_temp_local();
        let element_address = self.reserve_temp_local();

        self.emit_is_typed_array_i32(receiver, receiver_tag, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "TypedArray.prototype.fill requires TypedArray",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_load_typed_array_private_state(
            receiver,
            buffer,
            byte_offset,
            stored_byte_length,
            bytes_per_element,
            function,
        );
        let view = TypedArrayViewLocals::new(
            receiver,
            buffer,
            byte_offset,
            stored_byte_length,
            bytes_per_element,
        );
        self.emit_load_array_buffer_flags(buffer, buffer_flags, function);
        function.instruction(&Instruction::LocalGet(buffer_flags));
        function.instruction(&Instruction::I64Const(
            ArrayBufferFlag::Immutable.word() as i64
        ));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "TypedArray.prototype.fill backing buffer is immutable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_typed_array_witness(
            &view,
            TypedArrayWitnessUse::ValidatedMethodEntry {
                length_local: initial_length,
            },
            function,
        )?;
        self.load_i64_to_local_from_offset(
            receiver,
            HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET,
            element_kind,
            function,
        );
        self.emit_builtin_arg_to_locals(0, value_payload, value_tag, function);
        // This conversion performs ToNumber or ToBigInt and retains the storage
        // payload. The write loop cannot invoke a value coercion again.
        self.emit_value_to_typed_array_element_payload(
            element_kind,
            value_tag,
            value_payload,
            element_payload,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);

        self.emit_builtin_arg_to_locals(1, argument_payload, argument_tag, function);
        self.emit_value_to_number_payload(argument_tag, argument_payload, function)?;
        function.instruction(&Instruction::LocalSet(argument_payload));
        self.emit_return_current_completion_if_throw(function);
        self.emit_to_integer_or_infinity_number_payload_from_number_payload(
            argument_payload,
            argument_payload,
            function,
        );
        self.emit_array_slice_clamped_index(argument_payload, initial_length, start, function);

        function.instruction(&Instruction::LocalGet(initial_length));
        function.instruction(&Instruction::LocalSet(end));
        self.emit_builtin_arg_to_locals(2, argument_payload, argument_tag, function);
        function.instruction(&Instruction::LocalGet(argument_tag));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_number_payload(argument_tag, argument_payload, function)?;
        function.instruction(&Instruction::LocalSet(argument_payload));
        self.emit_return_current_completion_if_throw(function);
        self.emit_to_integer_or_infinity_number_payload_from_number_payload(
            argument_payload,
            argument_payload,
            function,
        );
        self.emit_array_slice_clamped_index(argument_payload, initial_length, end, function);
        function.instruction(&Instruction::End);

        // Validation occurs even when start >= end. Coercions may detach the
        // buffer or make a fixed view out of bounds without leaving any writes.
        self.emit_typed_array_witness(
            &view,
            TypedArrayWitnessUse::ValidatedMethodEntry {
                length_local: current_length,
            },
            function,
        )?;
        function.instruction(&Instruction::LocalGet(current_length));
        function.instruction(&Instruction::LocalGet(end));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(current_length));
        function.instruction(&Instruction::LocalSet(end));
        function.instruction(&Instruction::End);
        self.emit_load_array_buffer_data(buffer, buffer_address, function);
        function.instruction(&Instruction::LocalGet(start));
        function.instruction(&Instruction::LocalSet(index));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(end));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(buffer_address));
        function.instruction(&Instruction::LocalGet(byte_offset));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(bytes_per_element));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(element_address));
        // Integer stores consume their numeric payload local. Preserve the
        // converted value for the next element.
        function.instruction(&Instruction::LocalGet(element_payload));
        function.instruction(&Instruction::LocalSet(stored_element_payload));
        self.emit_store_number_payload_to_typed_array_address_by_kind(
            bytes_per_element,
            element_kind,
            element_address,
            stored_element_payload,
            self.buffer_memory_index(),
            function,
        );
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(receiver));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        for local in [
            element_address,
            buffer_address,
            index,
            end,
            start,
            argument_tag,
            argument_payload,
            stored_element_payload,
            element_payload,
            value_tag,
            value_payload,
            current_length,
            initial_length,
            element_kind,
            bytes_per_element,
            stored_byte_length,
            byte_offset,
            buffer_flags,
            buffer,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
