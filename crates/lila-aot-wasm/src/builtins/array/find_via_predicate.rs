use super::*;

enum FindViaPredicateKind {
    Find,
    FindIndex,
    FindLast,
    FindLastIndex,
}

enum FindDirection {
    Ascending,
    Descending,
}

enum FindProjection {
    Value,
    Index,
}

impl FindViaPredicateKind {
    const fn direction(&self) -> FindDirection {
        match self {
            Self::Find | Self::FindIndex => FindDirection::Ascending,
            Self::FindLast | Self::FindLastIndex => FindDirection::Descending,
        }
    }

    const fn projection(&self) -> FindProjection {
        match self {
            Self::Find | Self::FindLast => FindProjection::Value,
            Self::FindIndex | Self::FindLastIndex => FindProjection::Index,
        }
    }

    const fn array_method_name(&self) -> &'static str {
        match self {
            Self::Find => "Array.prototype.find",
            Self::FindIndex => "Array.prototype.findIndex",
            Self::FindLast => "Array.prototype.findLast",
            Self::FindLastIndex => "Array.prototype.findLastIndex",
        }
    }

    const fn typed_array_method_name(&self) -> &'static str {
        match self {
            Self::Find => "TypedArray.prototype.find",
            Self::FindIndex => "TypedArray.prototype.findIndex",
            Self::FindLast => "TypedArray.prototype.findLast",
            Self::FindLastIndex => "TypedArray.prototype.findLastIndex",
        }
    }

    const fn array_predicate_not_callable_message(&self) -> &'static str {
        match self {
            Self::Find => "Array.prototype.find predicate is not callable",
            Self::FindIndex => "Array.prototype.findIndex predicate is not callable",
            Self::FindLast => "Array.prototype.findLast predicate is not callable",
            Self::FindLastIndex => "Array.prototype.findLastIndex predicate is not callable",
        }
    }

    const fn typed_array_predicate_not_callable_message(&self) -> &'static str {
        match self {
            Self::Find => "TypedArray.prototype.find predicate is not callable",
            Self::FindIndex => "TypedArray.prototype.findIndex predicate is not callable",
            Self::FindLast => "TypedArray.prototype.findLast predicate is not callable",
            Self::FindLastIndex => "TypedArray.prototype.findLastIndex predicate is not callable",
        }
    }
}

/// Predicate locals that have passed ECMAScript `IsCallable`.
///
/// Call borrows this private, non-Copy witness. Its locals must stay reserved
/// until the entire Wasm loop has been emitted: releasing them at the Call
/// emission site lets later loop instructions overwrite the next iteration's
/// predicate. The loop owner consumes the witness only after both End opcodes.
#[must_use = "a validated find predicate must be retained through its emitted loop"]
struct ValidatedFindPredicateLocals(TaggedLocals);

#[cfg(test)]
mod find_via_predicate_tests {
    use super::*;

    #[test]
    fn four_kinds_fix_direction_and_projection() {
        assert!(matches!(
            FindViaPredicateKind::Find.direction(),
            FindDirection::Ascending
        ));
        assert!(matches!(
            FindViaPredicateKind::FindIndex.direction(),
            FindDirection::Ascending
        ));
        assert!(matches!(
            FindViaPredicateKind::FindLast.direction(),
            FindDirection::Descending
        ));
        assert!(matches!(
            FindViaPredicateKind::FindLastIndex.direction(),
            FindDirection::Descending
        ));
        assert!(matches!(
            FindViaPredicateKind::Find.projection(),
            FindProjection::Value
        ));
        assert!(matches!(
            FindViaPredicateKind::FindLast.projection(),
            FindProjection::Value
        ));
        assert!(matches!(
            FindViaPredicateKind::FindIndex.projection(),
            FindProjection::Index
        ));
        assert!(matches!(
            FindViaPredicateKind::FindLastIndex.projection(),
            FindProjection::Index
        ));
    }
}

impl<'a> FunctionBuilder<'a> {
    fn emit_validate_find_predicate(
        &mut self,
        predicate_not_callable_message: &'static str,
        function: &mut Function,
    ) -> Result<ValidatedFindPredicateLocals, EmitError> {
        let predicate_payload_local = self.reserve_temp_local();
        let predicate_tag_local = self.reserve_temp_local();
        let predicate = TaggedLocals::new(predicate_payload_local, predicate_tag_local);

        self.emit_builtin_arg_to_locals(0, predicate.payload, predicate.tag, function);
        self.emit_is_callable_i32(predicate.tag, predicate.payload, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            predicate_not_callable_message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        Ok(ValidatedFindPredicateLocals(predicate))
    }

    fn emit_call_validated_find_predicate(
        &mut self,
        predicate: &ValidatedFindPredicateLocals,
        this_argument: TaggedLocals,
        element: TaggedLocals,
        index: TaggedLocals,
        receiver: TaggedLocals,
        argc_local: u32,
        argv_local: u32,
        result: TaggedLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let predicate = &predicate.0;
        self.emit_pre_evaluated_arg_vector(
            &[
                (element.payload, element.tag),
                (index.payload, index.tag),
                (receiver.payload, receiver.tag),
            ],
            argc_local,
            argv_local,
            function,
        )?;
        self.emit_function_or_proxy_call_with_argv_leave_throw_completion(
            predicate.payload,
            predicate.tag,
            this_argument.payload,
            this_argument.tag,
            argc_local,
            argv_local,
            result.payload,
            result.tag,
            function,
        )?;
        Ok(())
    }

    fn release_find_predicate(&mut self, predicate: ValidatedFindPredicateLocals) {
        let ValidatedFindPredicateLocals(predicate) = predicate;
        self.release_temp_local(predicate.tag);
        self.release_temp_local(predicate.payload);
    }

    fn emit_initialize_find_result(&self, projection: &FindProjection, function: &mut Function) {
        match projection {
            FindProjection::Value => {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
            }
            FindProjection::Index => {
                function.instruction(&Instruction::I64Const((-1.0f64).to_bits() as i64));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
            }
        }
    }

    fn emit_initialize_find_index(
        &self,
        direction: &FindDirection,
        len_local: u32,
        index_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        match direction {
            FindDirection::Ascending => {}
            FindDirection::Descending => {
                function.instruction(&Instruction::LocalGet(len_local));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::LocalGet(len_local));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Sub);
                function.instruction(&Instruction::LocalSet(index_local));
                function.instruction(&Instruction::End);
            }
        }
    }

    fn emit_project_find_match(
        &self,
        projection: &FindProjection,
        element: TaggedLocals,
        index: TaggedLocals,
        function: &mut Function,
    ) {
        match projection {
            FindProjection::Value => {
                function.instruction(&Instruction::LocalGet(element.payload));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::LocalGet(element.tag));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
            }
            FindProjection::Index => {
                function.instruction(&Instruction::LocalGet(index.payload));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::LocalGet(index.tag));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
            }
        }
    }

    fn emit_advance_find_index(
        &self,
        direction: &FindDirection,
        index_local: u32,
        function: &mut Function,
    ) {
        match direction {
            FindDirection::Ascending => {
                function.instruction(&Instruction::LocalGet(index_local));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::LocalSet(index_local));
            }
            FindDirection::Descending => {
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

    pub(in crate::builtins) fn compile_typed_array_prototype_find_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_typed_array_find_with_kind(function, FindViaPredicateKind::Find)
    }

    pub(in crate::builtins) fn compile_typed_array_prototype_find_index_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_typed_array_find_with_kind(function, FindViaPredicateKind::FindIndex)
    }

    pub(in crate::builtins) fn compile_typed_array_prototype_find_last_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_typed_array_find_with_kind(function, FindViaPredicateKind::FindLast)
    }

    pub(in crate::builtins) fn compile_typed_array_prototype_find_last_index_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_typed_array_find_with_kind(function, FindViaPredicateKind::FindLastIndex)
    }

    fn compile_typed_array_find_with_kind(
        &mut self,
        function: &mut Function,
        find_kind: FindViaPredicateKind,
    ) -> Result<(), EmitError> {
        let method_name = find_kind.typed_array_method_name();
        let predicate_not_callable_message = find_kind.typed_array_predicate_not_callable_message();
        let direction = find_kind.direction();
        let projection = find_kind.projection();
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
            "TypedArray find method requires a TypedArray",
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

        let predicate =
            self.emit_validate_find_predicate(predicate_not_callable_message, function)?;
        self.emit_builtin_arg_to_locals(1, this_arg_payload_local, this_arg_tag_local, function);
        self.emit_initialize_find_index(&direction, len_local, index_local, function);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        // FindViaPredicate performs Get, not HasProperty: holes are visited,
        // and properties, Proxy traps and integer-indexed state stay live.
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
        self.emit_call_validated_find_predicate(
            &predicate,
            TaggedLocals::new(this_arg_payload_local, this_arg_tag_local),
            TaggedLocals::new(element_payload_local, element_tag_local),
            TaggedLocals::new(index_payload_local, index_tag_local),
            TaggedLocals::new(receiver_payload_local, receiver_tag_local),
            argc_local,
            argv_local,
            TaggedLocals::new(callback_result_payload_local, callback_result_tag_local),
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
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_project_find_match(
            &projection,
            TaggedLocals::new(element_payload_local, element_tag_local),
            TaggedLocals::new(index_payload_local, index_tag_local),
            function,
        );
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_advance_find_index(&direction, index_local, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        // Observable helpers may use completion result locals as scratch.
        // Establish the no-match result after the loop, not before those calls.
        self.emit_initialize_find_result(&projection, function);
        self.release_find_predicate(predicate);
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(callback_result_tag_local);
        self.release_temp_local(callback_result_payload_local);
        self.release_temp_local(this_arg_tag_local);
        self.release_temp_local(this_arg_payload_local);
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

    pub(in crate::builtins) fn compile_array_prototype_find_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_array_find_with_kind(function, FindViaPredicateKind::Find)
    }

    pub(in crate::builtins) fn compile_array_prototype_find_index_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_array_find_with_kind(function, FindViaPredicateKind::FindIndex)
    }

    pub(in crate::builtins) fn compile_array_prototype_find_last_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_array_find_with_kind(function, FindViaPredicateKind::FindLast)
    }

    pub(in crate::builtins) fn compile_array_prototype_find_last_index_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_array_find_with_kind(function, FindViaPredicateKind::FindLastIndex)
    }

    fn compile_array_find_with_kind(
        &mut self,
        function: &mut Function,
        find_kind: FindViaPredicateKind,
    ) -> Result<(), EmitError> {
        let method_name = find_kind.array_method_name();
        let predicate_not_callable_message = find_kind.array_predicate_not_callable_message();
        let direction = find_kind.direction();
        let projection = find_kind.projection();
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
        let key_local = self.reserve_temp_local();
        let length_tag_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let index_payload_local = self.reserve_temp_local();
        let index_tag_local = self.reserve_temp_local();
        let this_arg_payload_local = self.reserve_temp_local();
        let this_arg_tag_local = self.reserve_temp_local();
        let callback_result_payload_local = self.reserve_temp_local();
        let callback_result_tag_local = self.reserve_temp_local();
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();

        // Generic Array methods observe public length even on Arguments and
        // TypedArrays. The shared operation owns ToObject, Get and ToLength,
        // including abrupt completion before predicate validation.
        self.emit_array_like_length_snapshot(
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            len_local,
            length_tag_local,
            function,
        )?;
        let predicate =
            self.emit_validate_find_predicate(predicate_not_callable_message, function)?;
        self.emit_builtin_arg_to_locals(1, this_arg_payload_local, this_arg_tag_local, function);
        self.emit_initialize_find_index(&direction, len_local, index_local, function);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        // FindViaPredicate performs Get, not HasProperty: holes are visited,
        // and properties, Proxy traps and integer-indexed state stay live.
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
        self.emit_call_validated_find_predicate(
            &predicate,
            TaggedLocals::new(this_arg_payload_local, this_arg_tag_local),
            TaggedLocals::new(element_payload_local, element_tag_local),
            TaggedLocals::new(index_payload_local, index_tag_local),
            TaggedLocals::new(receiver_payload_local, receiver_tag_local),
            argc_local,
            argv_local,
            TaggedLocals::new(callback_result_payload_local, callback_result_tag_local),
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
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_project_find_match(
            &projection,
            TaggedLocals::new(element_payload_local, element_tag_local),
            TaggedLocals::new(index_payload_local, index_tag_local),
            function,
        );
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_advance_find_index(&direction, index_local, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        // Observable helpers may use completion result locals as scratch.
        // Establish the no-match result after the loop, not before those calls.
        self.emit_initialize_find_result(&projection, function);
        self.release_find_predicate(predicate);
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(callback_result_tag_local);
        self.release_temp_local(callback_result_payload_local);
        self.release_temp_local(this_arg_tag_local);
        self.release_temp_local(this_arg_payload_local);
        self.release_temp_local(index_tag_local);
        self.release_temp_local(index_payload_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(length_tag_local);
        self.release_temp_local(key_local);
        Ok(())
    }
}
