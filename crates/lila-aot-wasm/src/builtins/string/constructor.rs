use super::*;
use crate::functions::{NewTargetPrototypeFallback, OrdinaryDefaultPrototype};

impl FunctionBuilder<'_> {
    pub(crate) fn emit_string_constructor_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument_payload = self.reserve_temp_local();
        let argument_tag = self.reserve_temp_local();
        let new_target_payload = self.reserve_temp_local();
        let new_target_tag = self.reserve_temp_local();
        let string_payload = self.reserve_temp_local();
        let string_tag = self.reserve_temp_local();
        self.emit_builtin_arg_to_locals(0, argument_payload, argument_tag, function);
        self.compile_new_target_to_locals(new_target_payload, new_target_tag, function)?;
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(string_tag));

        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(string_payload));
        function.instruction(&Instruction::Else);
        // The Symbol descriptive-string exception belongs only to [[Call]].
        function.instruction(&Instruction::LocalGet(new_target_tag));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(argument_tag));
        function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_symbol_descriptive_string_to_local(argument_payload, string_payload, function)?;
        function.instruction(&Instruction::Else);
        let primitive = self.emit_tagged_to_primitive_locals_in_current_function_realm(
            ToPrimitiveHint::String,
            argument_payload,
            argument_tag,
            function,
        )?;
        self.emit_current_function_realm_primitive_to_string_local(
            primitive,
            string_payload,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(new_target_tag));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(string_payload));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(string_tag));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Else);
        // String(value) converts before the observable NewTarget.prototype Get.
        // Generic receiver preallocation would reverse that required order.
        let prototype_payload = self.reserve_temp_local();
        let prototype_tag = self.reserve_temp_local();
        self.emit_new_target_prototype_to_locals(
            STRING_PROTOTYPE_GLOBAL_INDEX,
            NewTargetPrototypeFallback::RequiredResolvedRealmOrdinary(
                OrdinaryDefaultPrototype::String,
            ),
            prototype_payload,
            prototype_tag,
            function,
        )?;
        self.emit_alloc_plain_object_with_prototype_and_tag(
            Some(prototype_payload),
            Some(prototype_tag),
            None,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(self.result_local));
        self.emit_store_boxed_primitive_metadata(
            self.result_local,
            BOXED_PRIMITIVE_KIND_STRING,
            string_payload,
            string_tag,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.release_temp_local(prototype_tag);
        self.release_temp_local(prototype_payload);
        function.instruction(&Instruction::End);

        self.release_temp_local(string_tag);
        self.release_temp_local(string_payload);
        self.release_temp_local(new_target_tag);
        self.release_temp_local(new_target_payload);
        self.release_temp_local(argument_tag);
        self.release_temp_local(argument_payload);
        Ok(())
    }
}
