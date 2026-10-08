use super::*;
impl FunctionBuilder<'_> {
    fn emit_plural_category_result(
        &mut self,
        response: &PluralProviderResponse,
        record: &GcLocal<IntlPluralRulesObject>,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let reader = response.reader(schema, f);
        let code = schema.reserve_i64_local(f);
        let recognized = schema.reserve_i32_local(f);
        let mask = schema.reserve_i32_local(f);
        let out = schema.reserve_value_local(f);
        schema
            .struct_type::<IntlPluralRulesObject>()
            .field(IntlPluralRulesObjectSchema::CATEGORIES)
            .read(record, schema, f)
            .store(mask, f);
        reader.read_u64(code, schema, f);
        set_i32(recognized, 0, f);
        for category in PluralCategory::ALL {
            code.load(f);
            f.instruction(&Instruction::I64Const(category.wire_code() as i64));
            f.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, f);
            mask.load(f);
            f.instruction(&Instruction::I32Const(1 << category.index()));
            f.instruction(&Instruction::I32And);
            f.instruction(&Instruction::I32Eqz);
            self.open_frame(ControlFrameKind::If, f);
            f.instruction(&Instruction::Unreachable);
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
            let text = schema
                .reserve_gc_local(f)
                .initialize(self.emit_interned_string_reference(category.name(), f)?, f);
            out.set_reference(&text, schema, f);
            text.clear(f);
            set_i32(recognized, 1, f);
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
        }
        recognized.load(f);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        f.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        reader.finish(schema, f);
        self.completion().initialize(f);
        self.completion().value().copy_from(&out, f);
        out.clear(f);
        schema.release_i32_local(mask, f);
        schema.release_i32_local(recognized, f);
        schema.release_i64_local(code, f);
        Ok(())
    }
    pub(crate) fn emit_intl_plural_rules_select(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let record = self.emit_plural_record_from_receiver(f)?;
        let schema = self.runtime_schema();
        let value = schema.reserve_value_local(f);
        self.emit_builtin_arg_to_value(0, &value, f);
        let input = self.emit_intl_mathematical_value(&value, f)?;
        let response = self.emit_plural_provider_call(
            PluralProviderRequest::Select {
                record: &record,
                input: &input,
            },
            f,
        )?;
        self.emit_plural_category_result(&response, &record, f)?;
        response.clear(f);
        input.clear(schema, f);
        value.clear(f);
        record.clear(f);
        Ok(())
    }
    pub(crate) fn emit_intl_plural_rules_select_range(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let record = self.emit_plural_record_from_receiver(f)?;
        let schema = self.runtime_schema();
        let left = schema.reserve_value_local(f);
        let right = schema.reserve_value_local(f);
        self.emit_builtin_arg_to_value(0, &left, f);
        self.emit_builtin_arg_to_value(1, &right, f);
        for value in [&left, &right] {
            emit_tag_is(value, WasmRuntimeValueTag::Undefined, f);
            self.open_frame(ControlFrameKind::If, f);
            self.emit_intl_number_type_error(PR_RANGE_UNDEFINED, f)?;
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
        }
        let start = self.emit_intl_mathematical_value(&left, f)?;
        let end = self.emit_intl_mathematical_value(&right, f)?;
        // Both complete coercions precede the provider's NaN endpoint decision.
        let response = self.emit_plural_provider_call(
            PluralProviderRequest::Range {
                record: &record,
                start: &start,
                end: &end,
            },
            f,
        )?;
        self.emit_plural_category_result(&response, &record, f)?;
        response.clear(f);
        end.clear(schema, f);
        start.clear(schema, f);
        right.clear(f);
        left.clear(f);
        record.clear(f);
        Ok(())
    }
}
