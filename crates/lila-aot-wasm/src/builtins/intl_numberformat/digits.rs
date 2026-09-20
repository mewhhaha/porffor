use super::*;

/// No numeric conversion can consume this state until all four later rounding
/// options have been observed. Raw values remain rooted across their hooks.
pub(super) struct ObservedNumberDigitOptions {
    values: [TaggedLocals; 4],
}
pub(super) struct NumberDigitOptionsReady {
    observed: ObservedNumberDigitOptions,
    priority: u32,
}
const DIGIT_PROPERTIES: [&str; 4] = [
    "minimumFractionDigits",
    "maximumFractionDigits",
    "minimumSignificantDigits",
    "maximumSignificantDigits",
];
impl ObservedNumberDigitOptions {
    pub(super) fn read(
        builder: &mut FunctionBuilder<'_>,
        options: TaggedLocals,
        function: &mut Function,
    ) -> Result<Self, EmitError> {
        let values = core::array::from_fn(|_| {
            TaggedLocals::new(builder.reserve_temp_local(), builder.reserve_temp_local())
        });
        for (property, value) in DIGIT_PROPERTIES.into_iter().zip(values) {
            builder.emit_nf_get_option(options, property, value, function)?;
        }
        Ok(Self { values })
    }
    pub(super) fn observe_rounding(
        self,
        builder: &mut FunctionBuilder<'_>,
        options: TaggedLocals,
        selected: &NfOptionsLocals,
        function: &mut Function,
    ) -> Result<NumberDigitOptionsReady, EmitError> {
        let priority = builder.reserve_temp_local();
        let fallback = builder.reserve_temp_local();
        let recognized = builder.reserve_temp_local();
        builder.emit_nf_set_const(fallback, 1, function);
        builder.emit_nf_number_option(
            options,
            "roundingIncrement",
            1,
            5000,
            fallback,
            selected.word(NfWord::RoundingIncrement),
            function,
        )?;
        builder.emit_nf_set_const(recognized, 0, function);
        for increment in [1].into_iter().chain(
            NonUnitRoundingIncrement::ALL
                .iter()
                .map(|increment| u64::from(increment.value())),
        ) {
            builder.emit_nf_if_eq(
                selected.word(NfWord::RoundingIncrement),
                increment,
                function,
            );
            builder.emit_nf_set_const(recognized, 1, function);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(recognized));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        builder.emit_nf_range_error("Invalid roundingIncrement option", function)?;
        function.instruction(&Instruction::End);
        builder.emit_nf_choice_option(
            options,
            "roundingMode",
            RoundingMode::OPTIONS,
            RoundingMode::HalfExpand.wire_code(),
            selected.word(NfWord::RoundingMode),
            function,
        )?;
        builder.emit_nf_choice_option(
            options,
            "roundingPriority",
            RoundingPriority::OPTIONS,
            RoundingPriority::Auto.wire_code(),
            priority,
            function,
        )?;
        builder.emit_nf_choice_option(
            options,
            "trailingZeroDisplay",
            TrailingZeroDisplay::OPTIONS,
            TrailingZeroDisplay::Auto.wire_code(),
            selected.word(NfWord::TrailingZero),
            function,
        )?;
        builder.release_temp_local(recognized);
        builder.release_temp_local(fallback);
        Ok(NumberDigitOptionsReady {
            observed: self,
            priority,
        })
    }
}
impl NumberDigitOptionsReady {
    pub(super) fn finish(
        self,
        builder: &mut FunctionBuilder<'_>,
        selected: &NfOptionsLocals,
        currency: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let [minimum_fraction, maximum_fraction, minimum_significant, maximum_significant] =
            self.observed.values;
        let has_fraction = builder.reserve_temp_local();
        let has_significant = builder.reserve_temp_local();
        let need_fraction = builder.reserve_temp_local();
        let need_significant = builder.reserve_temp_local();
        let default_minimum = builder.reserve_temp_local();
        let default_maximum = builder.reserve_temp_local();
        let fallback = builder.reserve_temp_local();
        let fraction_minimum = selected.word(NfWord::MinimumFraction);
        let fraction_maximum = selected.word(NfWord::MaximumFraction);
        let significant_minimum = selected.word(NfWord::MinimumSignificant);
        let significant_maximum = selected.word(NfWord::MaximumSignificant);
        for (left, right, destination) in [
            (minimum_fraction, maximum_fraction, has_fraction),
            (minimum_significant, maximum_significant, has_significant),
        ] {
            for value in [left, right] {
                function.instruction(&Instruction::LocalGet(value.tag));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::I64Ne);
            }
            function.instruction(&Instruction::I32Or);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::LocalSet(destination));
        }
        builder.emit_nf_set_const(need_fraction, 1, function);
        builder.emit_nf_set_const(need_significant, 1, function);
        builder.emit_nf_if_eq(self.priority, RoundingPriority::Auto.wire_code(), function);
        builder.emit_nf_copy(has_significant, need_significant, function);
        function.instruction(&Instruction::LocalGet(has_fraction));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(selected.word(NfWord::Notation)));
        function.instruction(&Instruction::I64Const(
            NotationOption::Compact.wire_code() as i64
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(has_significant));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        builder.emit_nf_set_const(need_fraction, 0, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        builder.emit_nf_set_const(default_minimum, 0, function);
        builder.emit_nf_set_const(default_maximum, 3, function);
        builder.emit_nf_if_eq(
            selected.word(NfWord::Style),
            StyleOption::Percent.wire_code(),
            function,
        );
        builder.emit_nf_set_const(default_maximum, 0, function);
        function.instruction(&Instruction::End);
        builder.emit_nf_if_eq(
            selected.word(NfWord::Style),
            StyleOption::Currency.wire_code(),
            function,
        );
        builder.emit_nf_if_eq(
            selected.word(NfWord::Notation),
            NotationOption::Standard.wire_code(),
            function,
        );
        builder.emit_nf_currency_digits(currency, default_minimum, function)?;
        builder.emit_nf_copy(default_minimum, default_maximum, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(
            selected.word(NfWord::RoundingIncrement),
        ));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        builder.emit_nf_copy(default_minimum, default_maximum, function);
        function.instruction(&Instruction::End);

        builder.emit_nf_if_nonzero(need_significant, function);
        builder.emit_nf_set_const(fallback, 1, function);
        builder.emit_nf_coerce_digit(
            minimum_significant,
            DIGIT_PROPERTIES[2],
            1,
            21,
            fallback,
            significant_minimum,
            function,
        )?;
        builder.emit_nf_set_const(fallback, 21, function);
        builder.emit_nf_coerce_digit(
            maximum_significant,
            DIGIT_PROPERTIES[3],
            1,
            21,
            fallback,
            significant_maximum,
            function,
        )?;
        builder.emit_nf_check_digit_range(significant_minimum, significant_maximum, function)?;
        function.instruction(&Instruction::End);

        builder.emit_nf_if_nonzero(need_fraction, function);
        builder.emit_nf_if_nonzero(has_fraction, function);
        // Each supplied bound is independently converted before the defaults
        // combine them. This preserves both hooks even for a reversed pair.
        builder.emit_nf_set_const(fallback, 0, function);
        builder.emit_nf_coerce_digit(
            minimum_fraction,
            DIGIT_PROPERTIES[0],
            0,
            100,
            fallback,
            fraction_minimum,
            function,
        )?;
        builder.emit_nf_coerce_digit(
            maximum_fraction,
            DIGIT_PROPERTIES[1],
            0,
            100,
            fallback,
            fraction_maximum,
            function,
        )?;
        builder.emit_nf_if_eq(
            minimum_fraction.tag,
            ValueKind::Undefined.tag() as u64,
            function,
        );
        function.instruction(&Instruction::LocalGet(default_minimum));
        function.instruction(&Instruction::LocalGet(fraction_maximum));
        function.instruction(&Instruction::LocalGet(default_minimum));
        function.instruction(&Instruction::LocalGet(fraction_maximum));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::Select);
        function.instruction(&Instruction::LocalSet(fraction_minimum));
        function.instruction(&Instruction::Else);
        builder.emit_nf_if_eq(
            maximum_fraction.tag,
            ValueKind::Undefined.tag() as u64,
            function,
        );
        function.instruction(&Instruction::LocalGet(default_maximum));
        function.instruction(&Instruction::LocalGet(fraction_minimum));
        function.instruction(&Instruction::LocalGet(default_maximum));
        function.instruction(&Instruction::LocalGet(fraction_minimum));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::Select);
        function.instruction(&Instruction::LocalSet(fraction_maximum));
        function.instruction(&Instruction::Else);
        builder.emit_nf_check_digit_range(fraction_minimum, fraction_maximum, function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        builder.emit_nf_copy(default_minimum, fraction_minimum, function);
        builder.emit_nf_copy(default_maximum, fraction_maximum, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        builder.emit_nf_set_const(
            selected.word(NfWord::Precision),
            NumberPrecisionKind::Fraction.wire_code() as i64,
            function,
        );
        builder.emit_nf_if_nonzero(need_significant, function);
        builder.emit_nf_set_const(
            selected.word(NfWord::Precision),
            NumberPrecisionKind::Significant.wire_code() as i64,
            function,
        );
        function.instruction(&Instruction::End);
        for (priority, precision) in [
            (RoundingPriority::MorePrecision, NumberPrecisionKind::More),
            (RoundingPriority::LessPrecision, NumberPrecisionKind::Less),
        ] {
            builder.emit_nf_if_eq(self.priority, priority.wire_code(), function);
            builder.emit_nf_set_const(
                selected.word(NfWord::Precision),
                precision.wire_code() as i64,
                function,
            );
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(need_fraction));
        function.instruction(&Instruction::LocalGet(need_significant));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        builder.emit_nf_set_const(fraction_minimum, 0, function);
        builder.emit_nf_set_const(fraction_maximum, 0, function);
        builder.emit_nf_set_const(significant_minimum, 1, function);
        builder.emit_nf_set_const(significant_maximum, 2, function);
        builder.emit_nf_set_const(
            selected.word(NfWord::Precision),
            NumberPrecisionKind::More.wire_code() as i64,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(
            selected.word(NfWord::RoundingIncrement),
        ));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(selected.word(NfWord::Precision)));
        function.instruction(&Instruction::I64Const(
            NumberPrecisionKind::Fraction.wire_code() as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        builder.emit_nf_type_error(NF_INCREMENT_PRECISION, function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(fraction_minimum));
        function.instruction(&Instruction::LocalGet(fraction_maximum));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        builder.emit_nf_range_error(NF_INCREMENT_RANGE, function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for local in [
            fallback,
            default_maximum,
            default_minimum,
            need_significant,
            need_fraction,
            has_significant,
            has_fraction,
        ] {
            builder.release_temp_local(local);
        }
        builder.release_temp_local(self.priority);
        for value in self.observed.values.into_iter().rev() {
            builder.release_temp_local(value.tag);
            builder.release_temp_local(value.payload);
        }
        Ok(())
    }
}
impl FunctionBuilder<'_> {
    fn emit_nf_check_digit_range(
        &mut self,
        minimum: u32,
        maximum: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(minimum));
        function.instruction(&Instruction::LocalGet(maximum));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_nf_range_error(NF_DIGIT_RANGE, function)?;
        function.instruction(&Instruction::End);
        Ok(())
    }
    fn emit_nf_currency_digits(
        &mut self,
        currency: u32,
        destination: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let profiles = lila_intl::number_format::embedded_number_profiles().map_err(|error| {
            EmitError::unsupported(format!("NumberFormat currency profile failed: {error}"))
        })?;
        let fractions = profiles.currency_fractions();
        self.emit_nf_set_const(
            destination,
            i64::from(fractions.default_digits().get()),
            function,
        );
        let offset = self.reserve_temp_local();
        let length = self.reserve_temp_local();
        let code = self.reserve_temp_local();
        self.emit_unpack_string_payload(currency, offset, length, function);
        function.instruction(&Instruction::LocalGet(length));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(offset));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load16U(MemArg {
            offset: 0,
            align: 0,
            memory_index: 0,
        }));
        function.instruction(&Instruction::LocalGet(offset));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load8U(MemArg {
            offset: 2,
            align: 0,
            memory_index: 0,
        }));
        function.instruction(&Instruction::I64Const(16));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(code));
        for row in fractions.overrides() {
            let [first, second, third] = row.code();
            let expected = u32::from_le_bytes([first, second, third, 0]);
            self.emit_nf_if_eq(code, u64::from(expected), function);
            self.emit_nf_set_const(destination, i64::from(row.digits().get()), function);
            function.instruction(&Instruction::End);
        }
        for local in [code, length, offset] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
