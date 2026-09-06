//! The shared observable loop for Array map/filter/every/some.
//!
//! These are generic Array methods, not TypedArray methods. ToObject and one
//! LengthOfArrayLike precede callback validation; live HasProperty/Get own
//! integer-indexed exotic checks. The closed kind controls only result policy.

use super::*;

#[derive(Clone, Copy)]
pub(super) enum ArrayCallbackIterationKind {
    Map,
    Filter,
    Every,
    Some,
}

impl ArrayCallbackIterationKind {
    const fn callback_error(self) -> &'static str {
        match self {
            Self::Map => "Array.prototype.map mapper is not callable",
            Self::Filter => "Array.prototype.filter callback is not callable",
            Self::Every => "Array.prototype.every callback is not callable",
            Self::Some => "Array.prototype.some callback is not callable",
        }
    }
}

impl<'a> FunctionBuilder<'a> {
    pub(super) fn compile_array_callback_iteration(
        &mut self,
        function: &mut Function,
        kind: ArrayCallbackIterationKind,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self
            .this_payload_local
            .ok_or_else(|| EmitError::unsupported("missing Array callback iteration receiver"))?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported("missing Array callback iteration receiver tag")
        })?;
        let callback_payload_local = self.reserve_temp_local();
        let callback_tag_local = self.reserve_temp_local();
        let this_arg_payload_local = self.reserve_temp_local();
        let this_arg_tag_local = self.reserve_temp_local();
        let length_local = self.reserve_temp_local();
        let length_tag_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let index_payload_local = self.reserve_temp_local();
        let number_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let present_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let callback_result_payload_local = self.reserve_temp_local();
        let callback_result_tag_local = self.reserve_temp_local();
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let target_index_local = self.reserve_temp_local();
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();

        // The captured bound must survive callback validation, species effects,
        // and every callback. Do not substitute private Array/TypedArray extent.
        self.emit_array_iteration_length_before_callback_validation(
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            length_local,
            length_tag_local,
            function,
        )?;
        self.emit_builtin_arg_to_locals(0, callback_payload_local, callback_tag_local, function);
        self.emit_is_callable_i32(callback_tag_local, callback_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            kind.callback_error(),
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_builtin_arg_to_locals(1, this_arg_payload_local, this_arg_tag_local, function);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(target_index_local));
        // This is a compiler-time choice: quantifiers emit no species operation
        // or result allocation. Map preserves holes/length; filter packs values.
        match kind {
            ArrayCallbackIterationKind::Map => self.emit_array_species_create(
                receiver_payload_local,
                receiver_tag_local,
                length_local,
                target_payload_local,
                target_tag_local,
                function,
            )?,
            ArrayCallbackIterationKind::Filter => self.emit_array_species_create(
                receiver_payload_local,
                receiver_tag_local,
                target_index_local,
                target_payload_local,
                target_tag_local,
                function,
            )?,
            ArrayCallbackIterationKind::Every | ArrayCallbackIterationKind::Some => {}
        }
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
        self.emit_index_to_flat_map_key_local(
            index_local,
            index_payload_local,
            key_local,
            function,
        )?;
        self.emit_object_has_property_i32(
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            present_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(present_local));
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
        self.emit_propagate_throw_from_locals_if_needed(
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_payload_local));
        self.emit_pre_evaluated_arg_vector(
            &[
                (element_payload_local, element_tag_local),
                (index_payload_local, number_tag_local),
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
        match kind {
            ArrayCallbackIterationKind::Map => {
                self.emit_index_to_flat_map_key_local(
                    index_local,
                    index_payload_local,
                    key_local,
                    function,
                )?;
                self.emit_array_target_create_data_property_or_throw(
                    target_payload_local,
                    target_tag_local,
                    key_local,
                    callback_result_payload_local,
                    callback_result_tag_local,
                    function,
                )?;
                self.emit_return_current_completion_if_throw(function);
            }
            ArrayCallbackIterationKind::Filter => {
                self.compile_truthy_tagged_i32(
                    callback_result_tag_local,
                    callback_result_payload_local,
                    function,
                )?;
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_index_to_flat_map_key_local(
                    target_index_local,
                    index_payload_local,
                    key_local,
                    function,
                )?;
                // Filter keeps the value read before Call, not a second Get
                // after a callback that may have changed/deleted the property.
                self.emit_array_target_create_data_property_or_throw(
                    target_payload_local,
                    target_tag_local,
                    key_local,
                    element_payload_local,
                    element_tag_local,
                    function,
                )?;
                self.emit_return_current_completion_if_throw(function);
                function.instruction(&Instruction::LocalGet(target_index_local));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::LocalSet(target_index_local));
                function.instruction(&Instruction::End);
            }
            ArrayCallbackIterationKind::Every => {
                self.compile_truthy_tagged_i32(
                    callback_result_tag_local,
                    callback_result_payload_local,
                    function,
                )?;
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                self.emit_return_current_completion(function);
                function.instruction(&Instruction::End);
            }
            ArrayCallbackIterationKind::Some => {
                self.compile_truthy_tagged_i32(
                    callback_result_tag_local,
                    callback_result_payload_local,
                    function,
                )?;
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                self.emit_return_current_completion(function);
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
        match kind {
            ArrayCallbackIterationKind::Map | ArrayCallbackIterationKind::Filter => {
                function.instruction(&Instruction::LocalGet(target_payload_local));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::LocalGet(target_tag_local));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
            }
            ArrayCallbackIterationKind::Every => {
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
            }
            ArrayCallbackIterationKind::Some => {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
            }
        }
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(target_index_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        self.release_temp_local(callback_result_tag_local);
        self.release_temp_local(callback_result_payload_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(present_local);
        self.release_temp_local(key_local);
        self.release_temp_local(number_tag_local);
        self.release_temp_local(index_payload_local);
        self.release_temp_local(index_local);
        self.release_temp_local(length_tag_local);
        self.release_temp_local(length_local);
        self.release_temp_local(this_arg_tag_local);
        self.release_temp_local(this_arg_payload_local);
        self.release_temp_local(callback_tag_local);
        self.release_temp_local(callback_payload_local);
        Ok(())
    }
}
