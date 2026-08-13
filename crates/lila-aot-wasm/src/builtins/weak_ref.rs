use super::super::*;
use crate::functions::NewTargetPrototypeFallback;

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn emit_weak_ref_constructor(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let new_target_payload_local = self.reserve_temp_local();
        let new_target_tag_local = self.reserve_temp_local();
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();
        let prototype_tag_local = self.reserve_temp_local();
        let weak_ref_payload_local = self.reserve_temp_local();
        let weak_ref_record_local = self.reserve_temp_local();

        self.compile_new_target_to_locals(
            new_target_payload_local,
            new_target_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(new_target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "WeakRef constructor requires new",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_builtin_arg_to_locals(0, target_payload_local, target_tag_local, function);
        self.emit_can_be_held_weakly_i32(target_payload_local, target_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_weak_ref_invalid_target_type_error(function)?;
        function.instruction(&Instruction::End);

        self.emit_new_target_prototype_to_locals(
            WEAK_REF_PROTOTYPE_GLOBAL_INDEX,
            NewTargetPrototypeFallback::RealmIntrinsic(
                HEAP_REALM_INTRINSICS_WEAK_REF_PROTOTYPE_OFFSET,
            ),
            prototype_payload_local,
            prototype_tag_local,
            function,
        )?;
        self.emit_alloc_plain_object_with_prototype_and_tag(
            Some(prototype_payload_local),
            Some(prototype_tag_local),
            None,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(weak_ref_payload_local));
        self.emit_heap_alloc_const(HEAP_WEAK_REF_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(weak_ref_record_local));
        self.store_i64_local_at_offset(
            weak_ref_record_local,
            HEAP_WEAK_REF_TARGET_TAG_OFFSET,
            target_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            weak_ref_record_local,
            HEAP_WEAK_REF_TARGET_PAYLOAD_OFFSET,
            target_payload_local,
            function,
        );
        self.store_i64_const_at_offset(
            weak_ref_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            OBJECT_INTERNAL_BRAND_WEAK_REF,
            function,
        );
        self.store_i64_local_at_offset(
            weak_ref_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            weak_ref_record_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(weak_ref_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(weak_ref_record_local);
        self.release_temp_local(weak_ref_payload_local);
        self.release_temp_local(prototype_tag_local);
        self.release_temp_local(prototype_payload_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        self.release_temp_local(new_target_tag_local);
        self.release_temp_local(new_target_payload_local);
        Ok(())
    }

    pub(crate) fn emit_weak_ref_deref(&mut self, function: &mut Function) -> Result<(), EmitError> {
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let receiver_brand_local = self.reserve_temp_local();
        let weak_ref_record_local = self.reserve_temp_local();

        self.compile_this_to_locals(receiver_payload_local, receiver_tag_local, function)?;
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_weak_ref_incompatible_receiver_type_error(function)?;
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            receiver_brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(receiver_brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_WEAK_REF as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_weak_ref_incompatible_receiver_type_error(function)?;
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            weak_ref_record_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            weak_ref_record_local,
            HEAP_WEAK_REF_TARGET_PAYLOAD_OFFSET,
            self.result_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            weak_ref_record_local,
            HEAP_WEAK_REF_TARGET_TAG_OFFSET,
            self.result_tag_local,
            function,
        );

        self.release_temp_local(weak_ref_record_local);
        self.release_temp_local(receiver_brand_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    fn emit_weak_ref_invalid_target_type_error(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_throw_current_function_realm_type_error(
            "WeakRef target cannot be held weakly",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        Ok(())
    }

    fn emit_weak_ref_incompatible_receiver_type_error(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_throw_current_function_realm_type_error(
            "WeakRef.prototype.deref receiver does not have [[WeakRefTarget]]",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        Ok(())
    }
}
