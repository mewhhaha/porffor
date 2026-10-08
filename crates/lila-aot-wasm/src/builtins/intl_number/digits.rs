use super::*;

struct PreparedNumberRounding {
    integer: GcI32DomainLocal<IntegerDigitCount>,
    precision: GcI32DomainLocal<NumberPrecisionKind>,
    minimum_fraction: GcI32DomainLocal<Option<FractionDigitCount>>,
    maximum_fraction: GcI32DomainLocal<Option<FractionDigitCount>>,
    minimum_significant: GcI32DomainLocal<Option<SignificantDigitCount>>,
    maximum_significant: GcI32DomainLocal<Option<SignificantDigitCount>>,
    increment: GcI32DomainLocal<RoundingIncrement>,
    mode: GcI32DomainLocal<RoundingMode>,
    trailing: GcI32DomainLocal<TrailingZeroDisplay>,
}
impl PreparedNumberRounding {
    fn new(schema: &RuntimeSchema, function: &mut Function) -> Self {
        Self {
            integer: GcI32DomainLocal::new(
                schema,
                IntegerDigitCount::new(IntegerDigitCount::MIN).expect("native minimum"),
                function,
            ),
            precision: GcI32DomainLocal::new(schema, NumberPrecisionKind::Fraction, function),
            minimum_fraction: GcI32DomainLocal::new(schema, None, function),
            maximum_fraction: GcI32DomainLocal::new(schema, None, function),
            minimum_significant: GcI32DomainLocal::new(schema, None, function),
            maximum_significant: GcI32DomainLocal::new(schema, None, function),
            increment: GcI32DomainLocal::new(schema, RoundingIncrement::One, function),
            mode: GcI32DomainLocal::new(schema, RoundingMode::HalfExpand, function),
            trailing: GcI32DomainLocal::new(schema, TrailingZeroDisplay::Auto, function),
        }
    }
    fn publish(
        self,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) -> GcLocal<IntlNumberRounding> {
        let record = schema.reserve_gc_local(function).initialize(
            schema.struct_type::<IntlNumberRounding>().construct(
                (
                    self.integer.operand(),
                    self.precision.operand(),
                    self.minimum_fraction.operand(),
                    self.maximum_fraction.operand(),
                    self.minimum_significant.operand(),
                    self.maximum_significant.operand(),
                    self.increment.operand(),
                    self.mode.operand(),
                    self.trailing.operand(),
                ),
                function,
            ),
            function,
        );
        self.trailing.clear(schema, function);
        self.mode.clear(schema, function);
        self.increment.clear(schema, function);
        self.maximum_significant.clear(schema, function);
        self.minimum_significant.clear(schema, function);
        self.maximum_fraction.clear(schema, function);
        self.minimum_fraction.clear(schema, function);
        self.precision.clear(schema, function);
        self.integer.clear(schema, function);
        record
    }
}
/// These four unconverted values remain rooted until every later rounding
/// option has been observed. Only the consuming ready phase can coerce them.
struct ObservedNumberDigitOptions {
    values: [ValueLocals; 4],
}
struct NumberDigitOptionsReady {
    observed: ObservedNumberDigitOptions,
    priority: GcI32DomainLocal<RoundingPriority>,
}
const DIGIT_PROPERTIES: [IntlErrorOption; 4] = [
    IntlErrorOption::MinimumFractionDigits,
    IntlErrorOption::MaximumFractionDigits,
    IntlErrorOption::MinimumSignificantDigits,
    IntlErrorOption::MaximumSignificantDigits,
];
impl ObservedNumberDigitOptions {
    fn read(
        builder: &mut FunctionBuilder<'_>,
        options: &ValueLocals,
        function: &mut Function,
    ) -> Result<Self, EmitError> {
        let schema = builder.runtime_schema();
        let values = core::array::from_fn(|_| schema.reserve_value_local(function));
        for (property, value) in DIGIT_PROPERTIES.into_iter().zip(values.iter()) {
            builder.emit_intl_number_get_option(options, property.property(), value, function)?;
        }
        Ok(Self { values })
    }
    fn observe_rounding(
        self,
        builder: &mut FunctionBuilder<'_>,
        options: &ValueLocals,
        selected: &PreparedNumberRounding,
        function: &mut Function,
    ) -> Result<NumberDigitOptionsReady, EmitError> {
        let schema = builder.runtime_schema();
        let count = schema.reserve_i32_local(function);
        let fallback = schema.reserve_i32_local(function);
        let recognized = schema.reserve_i32_local(function);
        set_i32(fallback, 1, function);
        builder.emit_intl_number_number_option(
            options,
            IntlErrorOption::RoundingIncrement,
            1,
            5000,
            fallback,
            count,
            function,
        )?;
        set_i32(recognized, 0, function);
        for increment in core::iter::once(RoundingIncrement::One).chain(
            NonUnitRoundingIncrement::ALL
                .iter()
                .copied()
                .map(RoundingIncrement::Multiple),
        ) {
            count.load(function);
            function.instruction(&Instruction::I32Const(increment.encode()));
            function.instruction(&Instruction::I32Eq);
            builder.open_frame(ControlFrameKind::If, function);
            selected.increment.set_constant(increment, function);
            set_i32(recognized, 1, function);
            builder.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        recognized.load(function);
        function.instruction(&Instruction::I32Eqz);
        builder.open_frame(ControlFrameKind::If, function);
        builder.emit_intl_number_range_error(
            RuntimeErrorMessage::INVALID_ROUNDINGINCREMENT_OPTION,
            function,
        )?;
        builder.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        builder.emit_intl_number_choice_option(
            options,
            IntlErrorOption::RoundingMode,
            RoundingMode::ALL.iter().map(|v| (v.name(), *v)),
            RoundingMode::HalfExpand,
            &selected.mode,
            function,
        )?;
        let priority = GcI32DomainLocal::new(schema, RoundingPriority::Auto, function);
        builder.emit_intl_number_choice_option(
            options,
            IntlErrorOption::RoundingPriority,
            RoundingPriority::ALL.iter().map(|v| (v.name(), *v)),
            RoundingPriority::Auto,
            &priority,
            function,
        )?;
        builder.emit_intl_number_choice_option(
            options,
            IntlErrorOption::TrailingZeroDisplay,
            TrailingZeroDisplay::ALL.iter().map(|v| (v.name(), *v)),
            TrailingZeroDisplay::Auto,
            &selected.trailing,
            function,
        )?;
        schema.release_i32_local(recognized, function);
        schema.release_i32_local(fallback, function);
        schema.release_i32_local(count, function);
        Ok(NumberDigitOptionsReady {
            observed: self,
            priority,
        })
    }
}
impl NumberDigitOptionsReady {
    fn finish(
        self,
        builder: &mut FunctionBuilder<'_>,
        selected: PreparedNumberRounding,
        notation: &GcI32DomainLocal<NotationOption>,
        defaults: IntlDigitDefaults<'_>,
        function: &mut Function,
    ) -> Result<GcLocal<IntlNumberRounding>, EmitError> {
        let schema = builder.runtime_schema();
        let [minimum_fraction, maximum_fraction, minimum_significant, maximum_significant] =
            self.observed.values;
        let has_fraction = schema.reserve_i32_local(function);
        let has_significant = schema.reserve_i32_local(function);
        let need_fraction = schema.reserve_i32_local(function);
        let need_significant = schema.reserve_i32_local(function);
        let default_minimum = schema.reserve_i32_local(function);
        let default_maximum = schema.reserve_i32_local(function);
        let fallback = schema.reserve_i32_local(function);
        let fraction_minimum = schema.reserve_i32_local(function);
        let fraction_maximum = schema.reserve_i32_local(function);
        let significant_minimum = schema.reserve_i32_local(function);
        let significant_maximum = schema.reserve_i32_local(function);
        for (left, right, out) in [
            (&minimum_fraction, &maximum_fraction, has_fraction),
            (&minimum_significant, &maximum_significant, has_significant),
        ] {
            emit_tag_is(left, WasmRuntimeValueTag::Undefined, function);
            function.instruction(&Instruction::I32Eqz);
            emit_tag_is(right, WasmRuntimeValueTag::Undefined, function);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::I32Or);
            out.store(function);
        }
        set_i32(need_fraction, 1, function);
        set_i32(need_significant, 1, function);
        emit_domain_is(&self.priority, RoundingPriority::Auto, function);
        builder.open_frame(ControlFrameKind::If, function);
        has_significant.load(function);
        need_significant.store(function);
        has_fraction.load(function);
        function.instruction(&Instruction::I32Eqz);
        emit_domain_is(notation, NotationOption::Compact, function);
        function.instruction(&Instruction::I32And);
        has_significant.load(function);
        function.instruction(&Instruction::I32Or);
        builder.open_frame(ControlFrameKind::If, function);
        set_i32(need_fraction, 0, function);
        builder.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        builder.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        set_i32(default_minimum, 0, function);
        set_i32(default_maximum, 3, function);
        if let IntlDigitDefaults::NumberFormat { style, currency } = defaults {
            emit_domain_is(style, StyleOption::Percent, function);
            builder.open_frame(ControlFrameKind::If, function);
            set_i32(default_maximum, 0, function);
            builder.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            emit_domain_is(style, StyleOption::Currency, function);
            emit_domain_is(notation, NotationOption::Standard, function);
            function.instruction(&Instruction::I32And);
            builder.open_frame(ControlFrameKind::If, function);
            builder.emit_intl_number_currency_digits(currency, default_minimum, function)?;
            default_minimum.load(function);
            default_maximum.store(function);
            builder.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        emit_domain_is(&selected.increment, RoundingIncrement::One, function);
        function.instruction(&Instruction::I32Eqz);
        builder.open_frame(ControlFrameKind::If, function);
        default_minimum.load(function);
        default_maximum.store(function);
        builder.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        need_significant.load(function);
        builder.open_frame(ControlFrameKind::If, function);
        set_i32(fallback, 1, function);
        builder.emit_intl_number_coerce_digit(
            &minimum_significant,
            DIGIT_PROPERTIES[2],
            u32::from(SignificantDigitCount::MIN),
            u32::from(SignificantDigitCount::MAX),
            fallback,
            significant_minimum,
            function,
        )?;
        set_i32(fallback, 21, function);
        builder.emit_intl_number_coerce_digit(
            &maximum_significant,
            DIGIT_PROPERTIES[3],
            u32::from(SignificantDigitCount::MIN),
            u32::from(SignificantDigitCount::MAX),
            fallback,
            significant_maximum,
            function,
        )?;
        builder.emit_intl_number_check_digit_range(
            significant_minimum,
            significant_maximum,
            function,
        )?;
        selected
            .minimum_significant
            .set_checked_count(significant_minimum, function);
        selected
            .maximum_significant
            .set_checked_count(significant_maximum, function);
        builder.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        need_fraction.load(function);
        builder.open_frame(ControlFrameKind::If, function);
        has_fraction.load(function);
        builder.open_frame(ControlFrameKind::If, function);
        set_i32(fallback, 0, function);
        // Both supplied values are independently converted before either
        // fallback is combined, including a reversed pair's second hook.
        builder.emit_intl_number_coerce_digit(
            &minimum_fraction,
            DIGIT_PROPERTIES[0],
            u32::from(FractionDigitCount::MIN),
            u32::from(FractionDigitCount::MAX),
            fallback,
            fraction_minimum,
            function,
        )?;
        builder.emit_intl_number_coerce_digit(
            &maximum_fraction,
            DIGIT_PROPERTIES[1],
            u32::from(FractionDigitCount::MIN),
            u32::from(FractionDigitCount::MAX),
            fallback,
            fraction_maximum,
            function,
        )?;
        emit_tag_is(&minimum_fraction, WasmRuntimeValueTag::Undefined, function);
        builder.open_frame(ControlFrameKind::If, function);
        default_minimum.load(function);
        fraction_maximum.load(function);
        default_minimum.load(function);
        fraction_maximum.load(function);
        function.instruction(&Instruction::I32LtU);
        function.instruction(&Instruction::Select);
        fraction_minimum.store(function);
        function.instruction(&Instruction::Else);
        emit_tag_is(&maximum_fraction, WasmRuntimeValueTag::Undefined, function);
        builder.open_frame(ControlFrameKind::If, function);
        default_maximum.load(function);
        fraction_minimum.load(function);
        default_maximum.load(function);
        fraction_minimum.load(function);
        function.instruction(&Instruction::I32GtU);
        function.instruction(&Instruction::Select);
        fraction_maximum.store(function);
        function.instruction(&Instruction::Else);
        builder.emit_intl_number_check_digit_range(fraction_minimum, fraction_maximum, function)?;
        builder.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        builder.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        default_minimum.load(function);
        fraction_minimum.store(function);
        default_maximum.load(function);
        fraction_maximum.store(function);
        builder.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        selected
            .minimum_fraction
            .set_checked_count(fraction_minimum, function);
        selected
            .maximum_fraction
            .set_checked_count(fraction_maximum, function);
        builder.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        selected
            .precision
            .set_constant(NumberPrecisionKind::Fraction, function);
        need_significant.load(function);
        builder.open_frame(ControlFrameKind::If, function);
        selected
            .precision
            .set_constant(NumberPrecisionKind::Significant, function);
        builder.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        for (priority, precision) in [
            (RoundingPriority::MorePrecision, NumberPrecisionKind::More),
            (RoundingPriority::LessPrecision, NumberPrecisionKind::Less),
        ] {
            emit_domain_is(&self.priority, priority, function);
            builder.open_frame(ControlFrameKind::If, function);
            selected.precision.set_constant(precision, function);
            builder.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        need_fraction.load(function);
        need_significant.load(function);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        builder.open_frame(ControlFrameKind::If, function);
        selected.minimum_fraction.set_constant(
            Some(FractionDigitCount::new(0).expect("native compact default")),
            function,
        );
        selected.maximum_fraction.set_constant(
            Some(FractionDigitCount::new(0).expect("native compact default")),
            function,
        );
        selected.minimum_significant.set_constant(
            Some(SignificantDigitCount::new(1).expect("native compact default")),
            function,
        );
        selected.maximum_significant.set_constant(
            Some(SignificantDigitCount::new(2).expect("native compact default")),
            function,
        );
        selected
            .precision
            .set_constant(NumberPrecisionKind::More, function);
        builder.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        emit_domain_is(&selected.increment, RoundingIncrement::One, function);
        function.instruction(&Instruction::I32Eqz);
        builder.open_frame(ControlFrameKind::If, function);
        emit_domain_is(&selected.precision, NumberPrecisionKind::Fraction, function);
        function.instruction(&Instruction::I32Eqz);
        builder.open_frame(ControlFrameKind::If, function);
        builder.emit_intl_number_type_error(INTL_INCREMENT_PRECISION, function)?;
        builder.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        selected.minimum_fraction.load(function);
        selected.maximum_fraction.load(function);
        function.instruction(&Instruction::I32Ne);
        builder.open_frame(ControlFrameKind::If, function);
        builder.emit_intl_number_range_error(INTL_INCREMENT_RANGE, function)?;
        builder.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        builder.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        for local in [
            significant_maximum,
            significant_minimum,
            fraction_maximum,
            fraction_minimum,
            fallback,
            default_maximum,
            default_minimum,
            need_significant,
            need_fraction,
            has_significant,
            has_fraction,
        ] {
            schema.release_i32_local(local, function);
        }
        self.priority.clear(schema, function);
        maximum_significant.clear(function);
        minimum_significant.clear(function);
        maximum_fraction.clear(function);
        minimum_fraction.clear(function);
        Ok(selected.publish(schema, function))
    }
}
impl FunctionBuilder<'_> {
    pub(in crate::builtins) fn emit_intl_number_digit_options(
        &mut self,
        options: &ValueLocals,
        notation: &GcI32DomainLocal<NotationOption>,
        defaults: IntlDigitDefaults<'_>,
        function: &mut Function,
    ) -> Result<GcLocal<IntlNumberRounding>, EmitError> {
        let schema = self.runtime_schema();
        let selected = PreparedNumberRounding::new(schema, function);
        let count = schema.reserve_i32_local(function);
        let fallback = schema.reserve_i32_local(function);
        set_i32(fallback, 1, function);
        self.emit_intl_number_number_option(
            options,
            IntlErrorOption::MinimumIntegerDigits,
            u32::from(IntegerDigitCount::MIN),
            u32::from(IntegerDigitCount::MAX),
            fallback,
            count,
            function,
        )?;
        selected.integer.set_checked_count(count, function);
        schema.release_i32_local(fallback, function);
        schema.release_i32_local(count, function);
        ObservedNumberDigitOptions::read(self, options, function)?
            .observe_rounding(self, options, &selected, function)?
            .finish(self, selected, notation, defaults, function)
    }
    fn emit_intl_number_check_digit_range(
        &mut self,
        minimum: I32Local,
        maximum: I32Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        minimum.load(function);
        maximum.load(function);
        function.instruction(&Instruction::I32GtU);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_intl_number_range_error(INTL_DIGIT_RANGE, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }
    fn emit_intl_number_currency_digits(
        &mut self,
        currency: &GcLocal<StringValue>,
        out: I32Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let profiles = lila_intl::number_format::embedded_number_profiles().map_err(|error| {
            EmitError::unsupported(format!("NumberFormat currency profile failed: {error}"))
        })?;
        let fractions = profiles.currency_fractions();
        set_i32(out, i32::from(fractions.default_digits().get()), function);
        let schema = self.runtime_schema();
        let units = schema
            .reserve_gc_local::<CodeUnitArray, NonNullable>(function)
            .initialize(
                schema
                    .struct_type::<StringValue>()
                    .field(StringValueSchema::CODE_UNITS)
                    .read(currency, schema, function)
                    .reference(),
                function,
            );
        let length = schema.reserve_i32_local(function);
        let index = schema.reserve_i32_local(function);
        let unit = schema.reserve_i32_local(function);
        let code = schema.reserve_i32_local(function);
        schema
            .array_type::<CodeUnitArray>()
            .length(&units, schema, function);
        length.store(function);
        length.load(function);
        function.instruction(&Instruction::I32Const(3));
        function.instruction(&Instruction::I32Ne);
        builder_unreachable(function);
        set_i32(code, 0, function);
        for position in 0..3 {
            set_i32(index, position, function);
            schema
                .array_type::<CodeUnitArray>()
                .read(&units, index, schema, function)
                .store(unit, function);
            code.load(function);
            unit.load(function);
            function.instruction(&Instruction::I32Const(position * 8));
            function.instruction(&Instruction::I32Shl);
            function.instruction(&Instruction::I32Or);
            code.store(function);
        }
        for row in fractions.overrides() {
            let [first, second, third] = row.code();
            let expected = u32::from_le_bytes([first, second, third, 0]);
            code.load(function);
            function.instruction(&Instruction::I32Const(expected as i32));
            function.instruction(&Instruction::I32Eq);
            self.open_frame(ControlFrameKind::If, function);
            set_i32(out, i32::from(row.digits().get()), function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        for local in [code, unit, index, length] {
            schema.release_i32_local(local, function);
        }
        units.clear(function);
        Ok(())
    }
}

fn builder_unreachable(function: &mut Function) {
    function.instruction(&Instruction::If(BlockType::Empty));
    function.instruction(&Instruction::Unreachable);
    function.instruction(&Instruction::End);
}
