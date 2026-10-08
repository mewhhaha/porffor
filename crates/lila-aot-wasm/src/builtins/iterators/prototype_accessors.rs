//! Iterator prototype setters ignore inherited properties without bypassing
//! the receiver's own descriptor, strict Set or DefineOwnProperty methods.
use super::*;

#[derive(Clone, Copy)]
pub(crate) enum IteratorPrototypeWeirdSetter {
    Constructor,
    ToStringTag,
}

impl IteratorPrototypeWeirdSetter {
    const fn incompatible_receiver_message(self) -> RuntimeErrorMessage {
        match self {
            Self::Constructor => RuntimeErrorMessage::ITERATOR_PROTOTYPE_CONSTRUCTOR_SETTER_CALLED_ON_INCOMPATIBLE_RECEIVER,
            Self::ToStringTag => RuntimeErrorMessage::ITERATOR_PROTOTYPE_SYMBOL_TOSTRINGTAG_SETTER_CALLED_ON_INCOMPATIBLE_RECEIVER,
        }
    }
    const fn home_receiver_message(self) -> RuntimeErrorMessage {
        match self {
            Self::Constructor => RuntimeErrorMessage::CANNOT_ASSIGN_TO_READ_ONLY_PROPERTY_CONSTRUCTOR_OF_ITERATOR_PROTOTYPE,
            Self::ToStringTag => RuntimeErrorMessage::CANNOT_ASSIGN_TO_READ_ONLY_PROPERTY_SYMBOL_TOSTRINGTAG_OF_ITERATOR_PROTOTYPE,
        }
    }
    fn key(
        self,
        builder: &mut FunctionBuilder<'_>,
        function: &mut Function,
    ) -> Result<PropertyKeyLocals, EmitError> {
        match self {
            Self::Constructor => builder.emit_iterator_named_key("constructor", function),
            Self::ToStringTag => {
                let schema = builder.runtime_schema();
                let symbol = schema.reserve_gc_local(function).initialize(
                    builder.emit_well_known_symbol_reference(
                        lila_ir::WellKnownSymbol::ToStringTag,
                        function,
                    )?,
                    function,
                );
                let key = PropertyKeyLocals::from_symbol(schema, &symbol, function);
                symbol.clear(function);
                Ok(key)
            }
        }
    }
}

impl FunctionBuilder<'_> {
    pub(crate) fn emit_iterator_prototype_weird_setter(
        &mut self,
        setter: IteratorPrototypeWeirdSetter,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let receiver = schema.reserve_value_local(function);
        let value = schema.reserve_value_local(function);
        let home = schema.reserve_value_local(function);
        let pending = schema.reserve_completion(function);
        let result = schema.reserve_completion(function);
        result.initialize(function);
        receiver.copy_from(
            self.body_entry_locals()
                .ok_or_else(|| {
                    EmitError::unsupported("Iterator prototype setter requires its builtin entry")
                })?
                .this_value(),
            function,
        );
        self.emit_builtin_arg_to_value(0, &value, function);
        let exit = self.open_frame(ControlFrameKind::Block, function);
        self.emit_is_heap_object_like_tag_i32(receiver.tag(), function);
        function.instruction(&Instruction::I32Eqz);
        self.emit_native_iterator_error_if(
            setter.incompatible_receiver_message(),
            &result,
            exit,
            function,
        )?;
        let realm = schema
            .reserve_gc_local(function)
            .initialize(self.emit_current_function_realm(function), function);
        self.emit_load_non_array_realm_intrinsic(
            &realm,
            NonArrayRealmIntrinsicSlot::IteratorPrototype,
            &home,
            function,
        );
        realm.clear(function);
        receiver.reference().load(function);
        home.reference().load(function);
        function.instruction(&Instruction::RefEq);
        self.emit_native_iterator_error_if(
            setter.home_receiver_message(),
            &result,
            exit,
            function,
        )?;
        let key = setter.key(self, function)?;
        let descriptor = self.emit_proxy_target_own_descriptor(&receiver, &key, function)?;
        pending.copy_from(self.completion(), function);
        self.emit_native_iterator_abrupt_exit(&pending, &result, exit, function);
        descriptor.emit_found_i32(schema, function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_object_write_strict(&receiver, &key, &value, &pending, function)?;
        function.instruction(&Instruction::Else);
        self.emit_create_data_property_or_throw(&receiver, &key, &value, &pending, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        descriptor.clear(function);
        key.clear(function);
        self.emit_native_iterator_abrupt_exit(&pending, &result, exit, function);
        home.set_undefined(function);
        result.set_normal(&home, function);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.completion().copy_from(&result, function);
        result.clear(function);
        pending.clear(function);
        home.clear(function);
        value.clear(function);
        receiver.clear(function);
        Ok(())
    }
}
