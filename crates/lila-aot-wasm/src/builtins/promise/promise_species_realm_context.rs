use super::*;

#[must_use = "species context must be consumed"]
pub(super) struct PromiseSpeciesRealmContext {
    constructor: ValueLocals,
    type_error: ValueLocals,
}

impl FunctionBuilder<'_> {
    pub(super) fn emit_current_function_promise_species_realm_context(
        &mut self,
        function: &mut Function,
    ) -> PromiseSpeciesRealmContext {
        let schema = self.runtime_schema();
        let realm = schema
            .reserve_gc_local(function)
            .initialize(self.emit_current_function_realm(function), function);
        let constructor = schema.reserve_value_local(function);
        let type_error = schema.reserve_value_local(function);
        self.emit_load_non_array_realm_intrinsic(
            &realm,
            NonArrayRealmIntrinsicSlot::PromiseConstructor,
            &constructor,
            function,
        );
        self.emit_load_non_array_realm_intrinsic(
            &realm,
            NonArrayRealmIntrinsicSlot::TypeErrorPrototype,
            &type_error,
            function,
        );
        realm.clear(function);
        PromiseSpeciesRealmContext {
            constructor,
            type_error,
        }
    }
    pub(super) fn emit_promise_species_constructor(
        &mut self,
        context: PromiseSpeciesRealmContext,
        promise: &ValueLocals,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        result.set_normal(&context.constructor, function);
        let pending = schema.reserve_completion(function);
        let key_string = schema.reserve_gc_local(function).initialize(
            self.emit_interned_string_reference("constructor", function)?,
            function,
        );
        let key = PropertyKeyLocals::from_string(schema, &key_string, function);
        let exit = self.open_frame(ControlFrameKind::Block, function);
        self.emit_object_read_with_throw_routing(
            promise,
            promise,
            &key,
            &pending,
            AccessorThrowRouting::LeaveInCompletion,
            function,
        )?;
        pending.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        result.copy_from(&pending, function);
        self.emit_branch_to_target(exit, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        pending.value().tag().load(function);
        function.instruction(&Instruction::I32Const(ValueKind::Undefined.tag() as i32));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_is_heap_object_like_tag_i32(pending.value().tag(), function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_runtime_error_with_prototype(
            NativeErrorKind::TypeError,
            RuntimeErrorMessage::PROMISE_CONSTRUCTOR_PROPERTY_IS_NOT_AN_OBJECT,
            &context.type_error,
            result,
            function,
        )?;
        self.emit_branch_to_target(exit, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let constructor = schema.reserve_value_local(function);
        constructor.copy_from(pending.value(), function);
        let symbol = schema.reserve_gc_local(function).initialize(
            self.emit_well_known_symbol_reference(WellKnownSymbol::Species, function)?,
            function,
        );
        let species_key = PropertyKeyLocals::from_symbol(schema, &symbol, function);
        self.emit_object_read_with_throw_routing(
            &constructor,
            &constructor,
            &species_key,
            &pending,
            AccessorThrowRouting::LeaveInCompletion,
            function,
        )?;
        pending.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        result.copy_from(&pending, function);
        self.emit_branch_to_target(exit, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.compile_nullish_tagged_i32(pending.value().tag(), function)?;
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_is_constructor_i32(pending.value(), function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_runtime_error_with_prototype(
            NativeErrorKind::TypeError,
            RuntimeErrorMessage::PROMISE_SPECIES_IS_NOT_A_CONSTRUCTOR,
            &context.type_error,
            result,
            function,
        )?;
        self.emit_branch_to_target(exit, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        result.set_normal(pending.value(), function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        species_key.clear(function);
        symbol.clear(function);
        constructor.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        key.clear(function);
        key_string.clear(function);
        pending.clear(function);
        context.type_error.clear(function);
        context.constructor.clear(function);
        Ok(())
    }
}
