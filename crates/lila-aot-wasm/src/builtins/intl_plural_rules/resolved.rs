use super::*;

impl FunctionBuilder<'_> {
    fn emit_pr_resolved_property(
        &mut self,
        object: u32,
        name: &str,
        value: TaggedLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key = self.reserve_temp_local();
        self.emit_pr_set_string(key, name, function);
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

    fn emit_pr_result_object(&mut self, function: &mut Function) -> Result<(), EmitError> {
        let prototype = self.reserve_temp_local();
        let realm = self.reserve_temp_local();
        let intrinsics = self.reserve_temp_local();
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
        for (source, offset, destination) in [
            (realm, HEAP_REALM_INTRINSICS_OFFSET, intrinsics),
            (
                intrinsics,
                HEAP_REALM_INTRINSICS_OBJECT_PROTOTYPE_OFFSET,
                prototype,
            ),
        ] {
            function.instruction(&Instruction::LocalGet(source));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Unreachable);
            function.instruction(&Instruction::End);
            self.load_i64_to_local_from_offset(source, offset, destination, function);
        }
        function.instruction(&Instruction::End);
        self.emit_alloc_plain_object_with_prototype(Some(prototype), None, function)?;
        for local in [intrinsics, realm, prototype] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    fn emit_pr_resolved_string(
        &mut self,
        record: u32,
        object: u32,
        offset: u64,
        property: &str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let value = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        self.load_i64_to_local_from_offset(record, offset, value.payload, function);
        self.emit_pr_set_const(value.tag, ValueKind::String.tag() as i64, function);
        self.emit_pr_resolved_property(object, property, value, function)?;
        self.release_temp_local(value.tag);
        self.release_temp_local(value.payload);
        Ok(())
    }

    fn emit_pr_resolved_number_word(
        &mut self,
        record: u32,
        object: u32,
        word: NfWord,
        property: &str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let value = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        self.load_i64_to_local_from_offset(
            record,
            HEAP_INTL_PR_WORDS_OFFSET + (word.index() as u64 * 8),
            value.payload,
            function,
        );
        function.instruction(&Instruction::LocalGet(value.payload));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(value.payload));
        self.emit_pr_set_const(value.tag, ValueKind::Number.tag() as i64, function);
        self.emit_pr_resolved_property(object, property, value, function)?;
        self.release_temp_local(value.tag);
        self.release_temp_local(value.payload);
        Ok(())
    }

    fn emit_pr_resolved_choice(
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
        self.emit_pr_set_const(value.payload, 0, function);
        for &(expected, name) in choices {
            self.emit_pr_if_eq(code, expected as i64, function);
            self.emit_pr_set_string(value.payload, name, function);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(value.payload));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.emit_pr_set_const(value.tag, ValueKind::String.tag() as i64, function);
        self.emit_pr_resolved_property(object, property, value, function)?;
        self.release_temp_local(value.tag);
        self.release_temp_local(value.payload);
        self.release_temp_local(code);
        Ok(())
    }

    fn emit_pr_resolved_categories(
        &mut self,
        record: u32,
        object: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let mask = self.reserve_temp_local();
        let count = self.reserve_temp_local();
        let output = self.reserve_temp_local();
        let index = self.reserve_temp_local();
        let value = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        self.load_i64_to_local_from_offset(
            record,
            HEAP_INTL_PR_CATEGORY_MASK_OFFSET,
            mask,
            function,
        );
        self.load_i64_to_local_from_offset(
            record,
            HEAP_INTL_PR_CATEGORY_COUNT_OFFSET,
            count,
            function,
        );
        self.emit_alloc_array_payload_with_length_in_current_function_realm(
            count, output, function,
        )?;
        self.emit_pr_set_const(index, 0, function);
        self.emit_pr_set_const(value.tag, ValueKind::String.tag() as i64, function);
        for (bit, name) in [
            (1_i64, "zero"),
            (2, "one"),
            (4, "two"),
            (8, "few"),
            (16, "many"),
            (32, "other"),
        ] {
            function.instruction(&Instruction::LocalGet(mask));
            function.instruction(&Instruction::I64Const(bit));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_pr_set_string(value.payload, name, function);
            self.emit_array_write(output, index, value.payload, value.tag, function)?;
            function.instruction(&Instruction::LocalGet(index));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(index));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(count));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.emit_pr_set_const(value.tag, ValueKind::Array.tag() as i64, function);
        function.instruction(&Instruction::LocalGet(output));
        function.instruction(&Instruction::LocalSet(value.payload));
        self.emit_pr_resolved_property(object, "pluralCategories", value, function)?;
        for local in [value.tag, value.payload, index, output, count, mask] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn emit_intl_plural_rules_resolved_options(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let record = self.reserve_temp_local();
        let object = self.reserve_temp_local();
        let notation = self.reserve_temp_local();
        let precision = self.reserve_temp_local();
        let value = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        self.emit_pr_require_record(receiver, record, function)?;
        self.emit_pr_result_object(function)?;
        function.instruction(&Instruction::LocalSet(object));

        self.emit_pr_resolved_string(
            record,
            object,
            HEAP_INTL_PR_LOCALE_OFFSET,
            "locale",
            function,
        )?;
        self.emit_pr_resolved_choice(
            record,
            object,
            HEAP_INTL_PR_TYPE_OFFSET,
            "type",
            &[(1, "cardinal"), (2, "ordinal")],
            function,
        )?;
        self.emit_pr_resolved_choice(
            record,
            object,
            HEAP_INTL_PR_WORDS_OFFSET + (NfWord::Notation.index() as u64 * 8),
            "notation",
            &[
                (1, "standard"),
                (2, "scientific"),
                (3, "engineering"),
                (4, "compact"),
            ],
            function,
        )?;
        self.load_i64_to_local_from_offset(
            record,
            HEAP_INTL_PR_WORDS_OFFSET + (NfWord::Notation.index() as u64 * 8),
            notation,
            function,
        );
        self.emit_pr_if_eq(notation, 4, function);
        self.emit_pr_resolved_choice(
            record,
            object,
            HEAP_INTL_PR_WORDS_OFFSET + (NfWord::CompactDisplay.index() as u64 * 8),
            "compactDisplay",
            &[(1, "short"), (2, "long")],
            function,
        )?;
        function.instruction(&Instruction::End);

        self.emit_pr_resolved_number_word(
            record,
            object,
            NfWord::MinimumInteger,
            "minimumIntegerDigits",
            function,
        )?;
        self.load_i64_to_local_from_offset(
            record,
            HEAP_INTL_PR_WORDS_OFFSET + (NfWord::Precision.index() as u64 * 8),
            precision,
            function,
        );
        self.emit_pr_if_eq(precision, 2, function);
        function.instruction(&Instruction::Else);
        self.emit_pr_resolved_number_word(
            record,
            object,
            NfWord::MinimumFraction,
            "minimumFractionDigits",
            function,
        )?;
        self.emit_pr_resolved_number_word(
            record,
            object,
            NfWord::MaximumFraction,
            "maximumFractionDigits",
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_pr_if_eq(precision, 1, function);
        function.instruction(&Instruction::Else);
        self.emit_pr_resolved_number_word(
            record,
            object,
            NfWord::MinimumSignificant,
            "minimumSignificantDigits",
            function,
        )?;
        self.emit_pr_resolved_number_word(
            record,
            object,
            NfWord::MaximumSignificant,
            "maximumSignificantDigits",
            function,
        )?;
        function.instruction(&Instruction::End);

        self.emit_pr_resolved_categories(record, object, function)?;
        self.emit_pr_resolved_number_word(
            record,
            object,
            NfWord::RoundingIncrement,
            "roundingIncrement",
            function,
        )?;
        self.emit_pr_resolved_choice(
            record,
            object,
            HEAP_INTL_PR_WORDS_OFFSET + (NfWord::RoundingMode.index() as u64 * 8),
            "roundingMode",
            &[
                (1, "ceil"),
                (2, "floor"),
                (3, "expand"),
                (4, "trunc"),
                (5, "halfCeil"),
                (6, "halfFloor"),
                (7, "halfExpand"),
                (8, "halfTrunc"),
                (9, "halfEven"),
            ],
            function,
        )?;
        self.emit_pr_resolved_choice(
            record,
            object,
            HEAP_INTL_PR_WORDS_OFFSET + (NfWord::Precision.index() as u64 * 8),
            "roundingPriority",
            &[
                (1, "auto"),
                (2, "auto"),
                (3, "morePrecision"),
                (4, "lessPrecision"),
            ],
            function,
        )?;
        self.emit_pr_resolved_choice(
            record,
            object,
            HEAP_INTL_PR_WORDS_OFFSET + (NfWord::TrailingZero.index() as u64 * 8),
            "trailingZeroDisplay",
            &[(1, "auto"), (2, "stripIfInteger")],
            function,
        )?;

        function.instruction(&Instruction::LocalGet(object));
        function.instruction(&Instruction::LocalSet(self.result_local));
        self.emit_pr_set_const(
            self.result_tag_local,
            ValueKind::Object.tag() as i64,
            function,
        );
        for local in [
            value.tag,
            value.payload,
            precision,
            notation,
            object,
            record,
            receiver.tag,
            receiver.payload,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
