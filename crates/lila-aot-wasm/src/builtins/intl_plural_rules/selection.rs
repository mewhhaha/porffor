use super::super::intl_numberformat::numeric_input::NfNumericLocals;
use super::provider_wire::{
    PluralRulesOperation, PluralRulesResponseReader, PluralRulesWireField, PluralRulesWireWord,
};
use super::*;

impl FunctionBuilder<'_> {
    fn emit_pr_category_result(
        &mut self,
        response: u32,
        operation: PluralRulesOperation,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let reader = PluralRulesResponseReader::new(self, response, operation, function)?;
        let code = self.reserve_temp_local();
        let value = self.reserve_temp_local();
        reader.word(self, code, function);
        self.emit_pr_set_const(value, 0, function);
        for (expected, category) in [
            (1, "zero"),
            (2, "one"),
            (3, "two"),
            (4, "few"),
            (5, "many"),
            (6, "other"),
        ] {
            self.emit_pr_if_eq(code, expected, function);
            self.emit_pr_set_string(value, category, function);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(value));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(value));
        function.instruction(&Instruction::LocalSet(self.result_local));
        self.emit_pr_set_const(
            self.result_tag_local,
            ValueKind::String.tag() as i64,
            function,
        );
        self.release_temp_local(value);
        self.release_temp_local(code);
        reader.finish(self, function);
        Ok(())
    }

    pub(crate) fn emit_intl_plural_rules_select(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let record = self.reserve_temp_local();
        let input = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let numeric = NfNumericLocals::reserve(self);
        let request = self.reserve_temp_local();
        let response = self.reserve_temp_local();
        self.emit_pr_require_record(receiver, record, function)?;
        self.emit_builtin_arg_to_locals(0, input.payload, input.tag, function);
        self.emit_nf_observe_numeric(input, &numeric, function)?;
        self.emit_pr_provider_request(
            PluralRulesOperation::SelectCategory,
            &[
                PluralRulesWireField::Configuration(record),
                PluralRulesWireField::Word(PluralRulesWireWord::Constant(1)),
                PluralRulesWireField::NumberInput {
                    kind: numeric.kind,
                    bytes: numeric.bytes,
                },
            ],
            request,
            function,
        )?;
        self.emit_pr_provider_call(
            PluralRulesOperation::SelectCategory,
            request,
            response,
            function,
        )?;
        self.emit_pr_category_result(response, PluralRulesOperation::SelectCategory, function)?;
        self.release_temp_local(response);
        self.release_temp_local(request);
        numeric.release(self);
        for local in [
            input.tag,
            input.payload,
            record,
            receiver.tag,
            receiver.payload,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn emit_intl_plural_rules_select_range(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let record = self.reserve_temp_local();
        let start = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let end = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let start_numeric = NfNumericLocals::reserve(self);
        let end_numeric = NfNumericLocals::reserve(self);
        let request = self.reserve_temp_local();
        let response = self.reserve_temp_local();

        self.emit_pr_require_record(receiver, record, function)?;
        self.emit_builtin_arg_to_locals(0, start.payload, start.tag, function);
        self.emit_builtin_arg_to_locals(1, end.payload, end.tag, function);
        self.emit_pr_if_eq(start.tag, ValueKind::Undefined.tag() as i64, function);
        self.emit_pr_type_error(PR_RANGE_UNDEFINED, function)?;
        function.instruction(&Instruction::End);
        self.emit_pr_if_eq(end.tag, ValueKind::Undefined.tag() as i64, function);
        self.emit_pr_type_error(PR_RANGE_UNDEFINED, function)?;
        function.instruction(&Instruction::End);

        self.emit_nf_observe_numeric(start, &start_numeric, function)?;
        self.emit_nf_observe_numeric(end, &end_numeric, function)?;
        self.emit_pr_provider_request(
            PluralRulesOperation::SelectCategory,
            &[
                PluralRulesWireField::Configuration(record),
                PluralRulesWireField::Word(PluralRulesWireWord::Constant(2)),
                PluralRulesWireField::NumberInput {
                    kind: start_numeric.kind,
                    bytes: start_numeric.bytes,
                },
                PluralRulesWireField::NumberInput {
                    kind: end_numeric.kind,
                    bytes: end_numeric.bytes,
                },
            ],
            request,
            function,
        )?;
        self.emit_pr_provider_call(
            PluralRulesOperation::SelectCategory,
            request,
            response,
            function,
        )?;
        self.emit_pr_category_result(response, PluralRulesOperation::SelectCategory, function)?;
        self.release_temp_local(response);
        self.release_temp_local(request);
        end_numeric.release(self);
        start_numeric.release(self);
        for local in [
            end.tag,
            end.payload,
            start.tag,
            start.payload,
            record,
            receiver.tag,
            receiver.payload,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
