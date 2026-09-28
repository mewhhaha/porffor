use super::*;

impl FunctionBuilder<'_> {
    pub(super) fn emit_rtf_result_object(
        &mut self,
        function: &mut Function,
    ) -> Result<u32, EmitError> {
        let object = self.reserve_temp_local();
        let prototype = self.reserve_temp_local();
        let realm = self.reserve_temp_local();
        let intrinsics = self.reserve_temp_local();
        let result = (|| {
            function.instruction(&Instruction::LocalGet(self.current_env_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
            function.instruction(&Instruction::LocalSet(prototype));
            function.instruction(&Instruction::Else);
            self.load_i64_to_local_from_offset(
                self.current_env_local,
                HEAP_FUNCTION_DEFINING_REALM_OFFSET,
                realm,
                function,
            );
            function.instruction(&Instruction::LocalGet(realm));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Unreachable);
            function.instruction(&Instruction::End);
            self.load_i64_to_local_from_offset(
                realm,
                HEAP_REALM_INTRINSICS_OFFSET,
                intrinsics,
                function,
            );
            function.instruction(&Instruction::LocalGet(intrinsics));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Unreachable);
            function.instruction(&Instruction::End);
            self.load_i64_to_local_from_offset(
                intrinsics,
                HEAP_REALM_INTRINSICS_OBJECT_PROTOTYPE_OFFSET,
                prototype,
                function,
            );
            function.instruction(&Instruction::End);
            self.emit_alloc_plain_object_with_prototype(Some(prototype), None, function)?;
            function.instruction(&Instruction::LocalSet(object));
            Ok(())
        })();
        self.release_temp_local(intrinsics);
        self.release_temp_local(realm);
        self.release_temp_local(prototype);
        match result {
            Ok(()) => Ok(object),
            Err(error) => {
                self.release_temp_local(object);
                Err(error)
            }
        }
    }

    fn emit_rtf_resolved_property(
        &mut self,
        object: u32,
        name: &str,
        value: TaggedLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key = self.reserve_temp_local();
        self.emit_rtf_set_string(key, name, function);
        self.emit_object_append_data_property_with_flags(
            object,
            key,
            value.payload,
            value.tag,
            true,
            true,
            true,
            function,
        )?;
        self.release_temp_local(key);
        Ok(())
    }

    fn emit_rtf_resolved_string_property(
        &mut self,
        record: u32,
        object: u32,
        offset: u64,
        name: &str,
        value: TaggedLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_rtf_load_string(record, offset, value.payload, function);
        self.emit_rtf_set_const(value.tag, ValueKind::String.tag() as i64, function);
        self.emit_rtf_resolved_property(object, name, value, function)
    }

    fn emit_rtf_resolved_choice_property(
        &mut self,
        record: u32,
        object: u32,
        offset: u64,
        name: &str,
        allowed: &[(&str, i64)],
        value: TaggedLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let code = self.reserve_temp_local();
        self.emit_rtf_load_word(record, offset, code, function);
        self.emit_rtf_set_const(value.payload, 0, function);
        for &(spelling, expected) in allowed {
            self.emit_rtf_if_eq(code, expected, function);
            self.emit_rtf_set_string(value.payload, spelling, function);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(value.payload));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.emit_rtf_set_const(value.tag, ValueKind::String.tag() as i64, function);
        self.emit_rtf_resolved_property(object, name, value, function)?;
        self.release_temp_local(code);
        Ok(())
    }

    pub(crate) fn emit_intl_relative_time_format_resolved_options(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let record = self.reserve_temp_local();
        let value = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        self.emit_rtf_record_from_receiver(receiver, record, function)?;
        let object = self.emit_rtf_result_object(function)?;
        self.emit_rtf_resolved_string_property(
            record,
            object,
            HEAP_INTL_RTF_LOCALE_OFFSET,
            "locale",
            value,
            function,
        )?;
        self.emit_rtf_resolved_choice_property(
            record,
            object,
            HEAP_INTL_RTF_STYLE_OFFSET,
            "style",
            &[("long", 1), ("short", 2), ("narrow", 3)],
            value,
            function,
        )?;
        self.emit_rtf_resolved_choice_property(
            record,
            object,
            HEAP_INTL_RTF_NUMERIC_OFFSET,
            "numeric",
            &[("always", 1), ("auto", 2)],
            value,
            function,
        )?;
        self.emit_rtf_resolved_string_property(
            record,
            object,
            HEAP_INTL_RTF_NUMBERING_SYSTEM_OFFSET,
            "numberingSystem",
            value,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(object));
        function.instruction(&Instruction::LocalSet(self.result_local));
        self.emit_rtf_set_const(
            self.result_tag_local,
            ValueKind::Object.tag() as i64,
            function,
        );
        for local in [
            object,
            value.tag,
            value.payload,
            record,
            receiver.tag,
            receiver.payload,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
