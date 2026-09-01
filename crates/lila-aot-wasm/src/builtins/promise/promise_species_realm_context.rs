use super::*;

#[must_use = "Promise species Realm context must be consumed"]
pub(super) struct PromiseSpeciesRealmContext {
    default_constructor_payload_local: u32,
    type_error_prototype_local: u32,
}

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_current_function_promise_species_realm_context(
        &mut self,
        function: &mut Function,
    ) -> PromiseSpeciesRealmContext {
        let default_constructor_payload_local = self.reserve_temp_local();
        let type_error_prototype_local = self.reserve_temp_local();
        let realm_local = self.reserve_temp_local();
        let intrinsics_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(PROMISE_CONSTRUCTOR_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(default_constructor_payload_local));
        function.instruction(&Instruction::GlobalGet(TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(type_error_prototype_local));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            self.current_env_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            realm_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(realm_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            realm_local,
            HEAP_REALM_INTRINSICS_OFFSET,
            intrinsics_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(intrinsics_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        for (offset, destination_local) in [
            (
                HEAP_REALM_INTRINSICS_PROMISE_CONSTRUCTOR_OFFSET,
                default_constructor_payload_local,
            ),
            (
                HEAP_REALM_INTRINSICS_TYPE_ERROR_PROTOTYPE_OFFSET,
                type_error_prototype_local,
            ),
        ] {
            self.load_i64_to_local_from_offset(
                intrinsics_local,
                offset,
                destination_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(destination_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Unreachable);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::End);

        self.release_temp_local(intrinsics_local);
        self.release_temp_local(realm_local);
        PromiseSpeciesRealmContext {
            default_constructor_payload_local,
            type_error_prototype_local,
        }
    }

    pub(super) fn emit_promise_species_constructor(
        &mut self,
        context: PromiseSpeciesRealmContext,
        promise_payload_local: u32,
        promise_tag_local: u32,
        species_constructor_payload_local: u32,
        species_constructor_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let constructor_payload_local = self.reserve_temp_local();
        let constructor_tag_local = self.reserve_temp_local();
        let species_payload_local = self.reserve_temp_local();
        let species_tag_local = self.reserve_temp_local();
        let species_is_constructor_local = self.reserve_temp_local();

        let result = (|| -> Result<(), EmitError> {
            function.instruction(&Instruction::LocalGet(
                context.default_constructor_payload_local,
            ));
            function.instruction(&Instruction::LocalSet(species_constructor_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::LocalSet(species_constructor_tag_local));

            function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_object_read(
                promise_payload_local,
                promise_tag_local,
                promise_payload_local,
                promise_tag_local,
                key_local,
                constructor_payload_local,
                constructor_tag_local,
                function,
            )?;
            self.emit_return_current_completion_if_throw(function);

            function.instruction(&Instruction::LocalGet(constructor_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_is_heap_object_like_tag_i32(constructor_tag_local, function);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_runtime_error_with_prototype_local(
                TYPE_ERROR_NAME,
                "Promise constructor property is not an object",
                context.type_error_prototype_local,
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);

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
            self.emit_return_current_completion_if_throw(function);
            self.compile_nullish_tagged_i32(species_tag_local, function)?;
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_is_constructor_i32(species_tag_local, species_payload_local, function)?;
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::LocalSet(species_is_constructor_local));
            function.instruction(&Instruction::LocalGet(species_is_constructor_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_runtime_error_with_prototype_local(
                TYPE_ERROR_NAME,
                "Promise species is not a constructor",
                context.type_error_prototype_local,
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalGet(species_payload_local));
            function.instruction(&Instruction::LocalSet(species_constructor_payload_local));
            function.instruction(&Instruction::LocalGet(species_tag_local));
            function.instruction(&Instruction::LocalSet(species_constructor_tag_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            Ok(())
        })();

        self.release_temp_local(species_is_constructor_local);
        self.release_temp_local(species_tag_local);
        self.release_temp_local(species_payload_local);
        self.release_temp_local(constructor_tag_local);
        self.release_temp_local(constructor_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(context.type_error_prototype_local);
        self.release_temp_local(context.default_constructor_payload_local);
        result
    }
}
