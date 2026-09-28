use super::*;
use crate::functions::NewTargetPrototypeFallback;

#[must_use]
pub(super) struct ReservedCollatorObjectLocal(u32);

#[must_use]
pub(super) struct InitializedCollatorObjectLocal(u32);

impl FunctionBuilder<'_> {
    pub(super) fn emit_reserve_collator_object(
        &mut self,
        function: &mut Function,
    ) -> Result<ReservedCollatorObjectLocal, EmitError> {
        let object = self.reserve_temp_local();
        let prototype = self.reserve_temp_local();
        let tag = self.reserve_temp_local();
        let result = (|| {
            self.compile_new_target_to_locals(prototype, tag, function)?;
            function.instruction(&Instruction::LocalGet(tag));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(self.current_env_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            function.instruction(&Instruction::GlobalGet(
                INTL_COLLATOR_CONSTRUCTOR_GLOBAL_INDEX,
            ));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(self.current_env_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalSet(prototype));
            self.emit_col_set_const(tag, ValueKind::Function.tag() as i64, function);
            self.emit_required_new_target_realm_ordinary_prototype(
                prototype,
                tag,
                OrdinaryDefaultPrototype::IntlCollator,
                prototype,
                tag,
                function,
            )?;
            function.instruction(&Instruction::Else);
            self.emit_new_target_prototype_to_locals(
                INTL_COLLATOR_PROTOTYPE_GLOBAL_INDEX,
                NewTargetPrototypeFallback::RequiredResolvedRealmOrdinary(
                    OrdinaryDefaultPrototype::IntlCollator,
                ),
                prototype,
                tag,
                function,
            )?;
            function.instruction(&Instruction::End);
            self.emit_alloc_plain_object_with_prototype_and_tag(
                Some(prototype),
                Some(tag),
                None,
                function,
            )?;
            function.instruction(&Instruction::LocalSet(object));
            Ok(())
        })();
        self.release_temp_local(tag);
        self.release_temp_local(prototype);
        if let Err(error) = result {
            self.release_temp_local(object);
            return Err(error);
        }
        Ok(ReservedCollatorObjectLocal(object))
    }

    pub(super) fn emit_initialize_collator_object(
        &self,
        reserved: ReservedCollatorObjectLocal,
        record: u32,
        function: &mut Function,
    ) -> InitializedCollatorObjectLocal {
        self.store_i64_const_at_offset(
            reserved.0,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            OBJECT_INTERNAL_BRAND_INTL_COLLATOR,
            function,
        );
        self.store_i64_local_at_offset(
            reserved.0,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            record,
            function,
        );
        InitializedCollatorObjectLocal(reserved.0)
    }

    pub(super) fn emit_publish_collator_object(
        &mut self,
        initialized: InitializedCollatorObjectLocal,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(initialized.0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.release_temp_local(initialized.0);
    }
}
