use super::*;

#[derive(Clone, Copy)]
enum ArgumentsSpecialProperty {
    Length,
    Callee,
}

impl ArgumentsSpecialProperty {
    const ALL: [Self; 2] = [Self::Length, Self::Callee];

    fn name(self) -> &'static str {
        match self {
            Self::Length => "length",
            Self::Callee => "callee",
        }
    }

    fn descriptor_offset(self) -> u64 {
        match self {
            Self::Length => HEAP_ARGUMENTS_LENGTH_DESCRIPTOR_KIND_OFFSET,
            Self::Callee => HEAP_ARGUMENTS_CALLEE_DESCRIPTOR_KIND_OFFSET,
        }
    }

    fn value_offsets(self) -> (u64, u64) {
        match self {
            Self::Length => (
                HEAP_ARGUMENTS_LENGTH_VALUE_OFFSET,
                HEAP_ARGUMENTS_LENGTH_VALUE_TAG_OFFSET,
            ),
            Self::Callee => (
                HEAP_ARGUMENTS_CALLEE_VALUE_PAYLOAD_OFFSET,
                HEAP_ARGUMENTS_CALLEE_VALUE_TAG_OFFSET,
            ),
        }
    }

    fn getter_offsets(self) -> (u64, u64) {
        match self {
            Self::Length => (
                HEAP_ARGUMENTS_LENGTH_GETTER_PAYLOAD_OFFSET,
                HEAP_ARGUMENTS_LENGTH_GETTER_TAG_OFFSET,
            ),
            Self::Callee => self.value_offsets(),
        }
    }

    fn setter_offsets(self) -> (u64, u64) {
        match self {
            Self::Length => (
                HEAP_ARGUMENTS_LENGTH_SETTER_PAYLOAD_OFFSET,
                HEAP_ARGUMENTS_LENGTH_SETTER_TAG_OFFSET,
            ),
            Self::Callee => (
                HEAP_ARGUMENTS_CALLEE_SETTER_PAYLOAD_OFFSET,
                HEAP_ARGUMENTS_CALLEE_SETTER_TAG_OFFSET,
            ),
        }
    }
}

impl FunctionBuilder<'_> {
    pub(super) fn emit_arguments_special_property_get(
        &mut self,
        target: TaggedLocals,
        receiver: TaggedLocals,
        key: u32,
        found: u32,
        result: TaggedLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let descriptor = self.reserve_temp_local();
        let getter = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        function.instruction(&Instruction::LocalGet(target.tag));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        for property in ArgumentsSpecialProperty::ALL {
            function.instruction(&Instruction::I64Const(
                self.strings.payload(property.name()),
            ));
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.emit_property_key_payload_equality_i32(key, self.scratch_local, function);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.load_i64_to_local_from_offset(
                target.payload,
                property.descriptor_offset(),
                descriptor,
                function,
            );
            function.instruction(&Instruction::LocalGet(descriptor));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            // A present accessor with no getter still shadows the prototype.
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(found));
            function.instruction(&Instruction::LocalGet(descriptor));
            function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            let (payload, tag) = property.value_offsets();
            self.load_i64_to_local_from_offset(target.payload, payload, result.payload, function);
            self.load_i64_to_local_from_offset(target.payload, tag, result.tag, function);
            function.instruction(&Instruction::Else);
            let (payload, tag) = property.getter_offsets();
            self.load_i64_to_local_from_offset(target.payload, payload, getter.payload, function);
            self.load_i64_to_local_from_offset(target.payload, tag, getter.tag, function);
            self.emit_is_callable_i32(getter.tag, getter.payload, function)?;
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_function_or_proxy_call_leave_throw_completion(
                getter.payload,
                getter.tag,
                receiver.payload,
                receiver.tag,
                &[],
                result.payload,
                result.tag,
                function,
            )?;
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(result.payload));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(result.tag));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::End);
        self.release_temp_local(getter.tag);
        self.release_temp_local(getter.payload);
        self.release_temp_local(descriptor);
        Ok(())
    }

    pub(super) fn emit_arguments_special_property_set(
        &mut self,
        target: TaggedLocals,
        receiver: TaggedLocals,
        key: TaggedLocals,
        value: TaggedLocals,
        found: u32,
        result: u32,
        allow_receiver_generic_write_fallback: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let descriptor = self.reserve_temp_local();
        let setter = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let completion = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        function.instruction(&Instruction::LocalGet(target.tag));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        for property in ArgumentsSpecialProperty::ALL {
            function.instruction(&Instruction::I64Const(
                self.strings.payload(property.name()),
            ));
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.emit_property_key_payload_equality_i32(key.payload, self.scratch_local, function);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.load_i64_to_local_from_offset(
                target.payload,
                property.descriptor_offset(),
                descriptor,
                function,
            );
            function.instruction(&Instruction::LocalGet(descriptor));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(found));
            function.instruction(&Instruction::LocalGet(descriptor));
            function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(descriptor));
            function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_WRITABLE as i64));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_ordinary_set_data_on_receiver_result(
                receiver.payload,
                receiver.tag,
                key.payload,
                key.tag,
                value.payload,
                value.tag,
                result,
                allow_receiver_generic_write_fallback,
                function,
            )?;
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::Else);
            let (payload, tag) = property.setter_offsets();
            self.load_i64_to_local_from_offset(target.payload, payload, setter.payload, function);
            self.load_i64_to_local_from_offset(target.payload, tag, setter.tag, function);
            self.emit_is_callable_i32(setter.tag, setter.payload, function)?;
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_function_or_proxy_call_leave_throw_completion(
                setter.payload,
                setter.tag,
                receiver.payload,
                receiver.tag,
                &[(value.payload, value.tag)],
                completion.payload,
                completion.tag,
                function,
            )?;
            self.emit_propagate_throw_from_locals_if_needed(
                completion.payload,
                completion.tag,
                function,
            )?;
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(result));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::End);
        self.release_temp_local(completion.tag);
        self.release_temp_local(completion.payload);
        self.release_temp_local(setter.tag);
        self.release_temp_local(setter.payload);
        self.release_temp_local(descriptor);
        Ok(())
    }

    pub(super) fn emit_arguments_special_property_receiver_set(
        &mut self,
        receiver: TaggedLocals,
        key: u32,
        value: TaggedLocals,
        handled: u32,
        result: u32,
        function: &mut Function,
    ) {
        let descriptor = self.reserve_temp_local();
        let non_extensible = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(handled));
        function.instruction(&Instruction::LocalGet(receiver.tag));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        for property in ArgumentsSpecialProperty::ALL {
            function.instruction(&Instruction::I64Const(
                self.strings.payload(property.name()),
            ));
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.emit_property_key_payload_equality_i32(key, self.scratch_local, function);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(handled));
            self.load_i64_to_local_from_offset(
                receiver.payload,
                property.descriptor_offset(),
                descriptor,
                function,
            );
            function.instruction(&Instruction::LocalGet(descriptor));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.load_i64_to_local_from_offset(
                receiver.payload,
                HEAP_ARGUMENTS_NON_EXTENSIBLE_OFFSET,
                non_extensible,
                function,
            );
            function.instruction(&Instruction::LocalGet(non_extensible));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.store_i64_const_at_offset(
                receiver.payload,
                property.descriptor_offset(),
                ARRAY_DESCRIPTOR_OWN_PROPERTY | ARRAY_DESCRIPTOR_NORMAL_DATA,
                function,
            );
            let (payload, tag) = property.value_offsets();
            self.store_i64_local_at_offset(receiver.payload, payload, value.payload, function);
            self.store_i64_local_at_offset(receiver.payload, tag, value.tag, function);
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(result));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::Else);
            // OrdinarySet updates a receiver data descriptor; it never calls a
            // receiver accessor that differs from the target's descriptor.
            function.instruction(&Instruction::LocalGet(descriptor));
            function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::LocalGet(descriptor));
            function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_WRITABLE as i64));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::If(BlockType::Empty));
            let (payload, tag) = property.value_offsets();
            self.store_i64_local_at_offset(receiver.payload, payload, value.payload, function);
            self.store_i64_local_at_offset(receiver.payload, tag, value.tag, function);
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(result));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::End);
        self.release_temp_local(non_extensible);
        self.release_temp_local(descriptor);
    }
}
