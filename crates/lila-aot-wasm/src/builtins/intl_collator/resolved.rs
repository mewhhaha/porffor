use super::*;

impl FunctionBuilder<'_> {
    fn emit_col_resolved_string(
        &mut self,
        record: u32,
        object: u32,
        offset: u64,
        property: &str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let value = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        self.load_i64_to_local_from_offset(record, offset, value.payload, function);
        self.emit_col_set_const(value.tag, ValueKind::String.tag() as i64, function);
        self.emit_col_resolved_property(object, property, value, function)?;
        self.release_temp_local(value.tag);
        self.release_temp_local(value.payload);
        Ok(())
    }

    fn emit_col_resolved_boolean(
        &mut self,
        record: u32,
        object: u32,
        offset: u64,
        property: &str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let value = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        self.load_i64_to_local_from_offset(record, offset, value.payload, function);
        self.emit_col_set_const(value.tag, ValueKind::Boolean.tag() as i64, function);
        self.emit_col_resolved_property(object, property, value, function)?;
        self.release_temp_local(value.tag);
        self.release_temp_local(value.payload);
        Ok(())
    }

    fn emit_col_resolved_choice(
        &mut self,
        record: u32,
        object: u32,
        offset: u64,
        property: &str,
        choices: &[(u64, &str)],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let code = self.reserve_temp_local();
        let value = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        self.load_i64_to_local_from_offset(record, offset, code, function);
        self.emit_col_set_const(value.payload, 0, function);
        for &(expected, name) in choices {
            self.emit_col_if_eq(code, expected as i64, function);
            self.emit_col_set_string(value.payload, name, function);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(value.payload));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.emit_col_set_const(value.tag, ValueKind::String.tag() as i64, function);
        self.emit_col_resolved_property(object, property, value, function)?;
        for local in [value.tag, value.payload, code] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn emit_intl_collator_resolved_options(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let record = self.reserve_temp_local();
        let object = self.reserve_temp_local();
        self.emit_col_require_record(receiver, record, function)?;
        self.emit_col_result_object(function)?;
        function.instruction(&Instruction::LocalSet(object));

        self.emit_col_resolved_string(
            record,
            object,
            HEAP_INTL_COL_LOCALE_OFFSET,
            "locale",
            function,
        )?;
        self.emit_col_resolved_choice(
            record,
            object,
            HEAP_INTL_COL_USAGE_OFFSET,
            "usage",
            &[(0, "sort"), (1, "search")],
            function,
        )?;
        self.emit_col_resolved_choice(
            record,
            object,
            HEAP_INTL_COL_SENSITIVITY_OFFSET,
            "sensitivity",
            &[(1, "base"), (2, "accent"), (3, "case"), (4, "variant")],
            function,
        )?;
        self.emit_col_resolved_boolean(
            record,
            object,
            HEAP_INTL_COL_IGNORE_PUNCTUATION_OFFSET,
            "ignorePunctuation",
            function,
        )?;
        self.emit_col_resolved_string(
            record,
            object,
            HEAP_INTL_COL_COLLATION_OFFSET,
            "collation",
            function,
        )?;
        self.emit_col_resolved_boolean(
            record,
            object,
            HEAP_INTL_COL_NUMERIC_OFFSET,
            "numeric",
            function,
        )?;
        self.emit_col_resolved_choice(
            record,
            object,
            HEAP_INTL_COL_CASE_FIRST_OFFSET,
            "caseFirst",
            &[(1, "false"), (2, "upper"), (3, "lower")],
            function,
        )?;
        function.instruction(&Instruction::LocalGet(object));
        function.instruction(&Instruction::LocalSet(self.result_local));
        self.emit_col_set_const(
            self.result_tag_local,
            ValueKind::Object.tag() as i64,
            function,
        );
        for local in [object, record, receiver.tag, receiver.payload] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn emit_intl_collator_compare_getter(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let record = self.reserve_temp_local();
        let bound = self.reserve_temp_local();
        self.emit_col_require_record(receiver, record, function)?;
        self.load_i64_to_local_from_offset(
            record,
            HEAP_INTL_COL_BOUND_COMPARE_OFFSET,
            bound,
            function,
        );
        function.instruction(&Instruction::LocalGet(bound));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        let meta = self
            .functions
            .get(&StandardBuiltinId::IntlCollatorCompareFunction.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported("missing Collator compare function dependency")
            })?;
        self.emit_current_builtin_realm_closure_value(&meta, receiver.payload, bound, function)?;
        self.store_i64_local_at_offset(record, HEAP_INTL_COL_BOUND_COMPARE_OFFSET, bound, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(bound));
        function.instruction(&Instruction::LocalSet(self.result_local));
        self.emit_col_set_const(
            self.result_tag_local,
            ValueKind::Function.tag() as i64,
            function,
        );
        for local in [bound, record, receiver.tag, receiver.payload] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
