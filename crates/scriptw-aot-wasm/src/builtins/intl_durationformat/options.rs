use super::*;
/// Every unit record and native option domain is complete before publication.
pub(super) struct CompletedDurationOptionsLocals {
    pub(super) style: GcI32DomainLocal<DurationStyle>,
    pub(super) fractional_digits: GcI32DomainLocal<Option<DurationFractionalDigits>>,
    pub(super) units: [GcLocal<IntlDurationUnitOptions>; 10],
}
impl CompletedDurationOptionsLocals {
    pub(super) fn clear(self, schema: &RuntimeSchema, f: &mut Function) {
        for unit in self.units.into_iter().rev() {
            unit.clear(f);
        }
        self.fractional_digits.clear(schema, f);
        self.style.clear(schema, f);
    }
}
trait DurationUnitErrorOptions {
    fn style_option(self) -> IntlErrorOption;
    fn display_option(self) -> IntlErrorOption;
}
impl DurationUnitErrorOptions for DurationUnit {
    fn style_option(self) -> IntlErrorOption {
        match self {
            Self::Year => IntlErrorOption::Years,
            Self::Month => IntlErrorOption::Months,
            Self::Week => IntlErrorOption::Weeks,
            Self::Day => IntlErrorOption::Days,
            Self::Hour => IntlErrorOption::Hours,
            Self::Minute => IntlErrorOption::Minutes,
            Self::Second => IntlErrorOption::Seconds,
            Self::Millisecond => IntlErrorOption::Milliseconds,
            Self::Microsecond => IntlErrorOption::Microseconds,
            Self::Nanosecond => IntlErrorOption::Nanoseconds,
        }
    }
    fn display_option(self) -> IntlErrorOption {
        match self {
            Self::Year => IntlErrorOption::YearsDisplay,
            Self::Month => IntlErrorOption::MonthsDisplay,
            Self::Week => IntlErrorOption::WeeksDisplay,
            Self::Day => IntlErrorOption::DaysDisplay,
            Self::Hour => IntlErrorOption::HoursDisplay,
            Self::Minute => IntlErrorOption::MinutesDisplay,
            Self::Second => IntlErrorOption::SecondsDisplay,
            Self::Millisecond => IntlErrorOption::MillisecondsDisplay,
            Self::Microsecond => IntlErrorOption::MicrosecondsDisplay,
            Self::Nanosecond => IntlErrorOption::NanosecondsDisplay,
        }
    }
}
fn emit_duration_previous_numeric(
    previous: &GcI32DomainLocal<Option<DurationUnitStyle>>,
    f: &mut Function,
) {
    emit_domain_is(previous, Some(DurationUnitStyle::Numeric), f);
    emit_domain_is(previous, Some(DurationUnitStyle::TwoDigit), f);
    f.instruction(&Instruction::I32Or);
}
impl FunctionBuilder<'_> {
    pub(super) fn emit_duration_options_object(
        &mut self,
        options: &ValueLocals,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        emit_tag_is(options, WasmRuntimeValueTag::Undefined, f);
        self.open_frame(ControlFrameKind::If, f);
        let empty = schema
            .reserve_gc_local(f)
            .initialize(self.emit_alloc_plain_object_with_prototype(None, f)?, f);
        options.set_reference(&empty, schema, f);
        empty.clear(f);
        f.instruction(&Instruction::Else);
        self.emit_is_heap_object_like_tag_i32(options.tag(), f);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_intl_number_type_error(
            RuntimeErrorMessage::INTL_DURATIONFORMAT_OPTIONS_MUST_BE_AN_OBJECT,
            f,
        )?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        Ok(())
    }
    pub(super) fn emit_complete_duration_options(
        &mut self,
        options: &ValueLocals,
        two_digit_hours: I32Local,
        f: &mut Function,
    ) -> Result<CompletedDurationOptionsLocals, EmitError> {
        let schema = self.runtime_schema();
        let base = GcI32DomainLocal::new(schema, DurationStyle::Short, f);
        let digits = GcI32DomainLocal::new(schema, None::<DurationFractionalDigits>, f);
        self.emit_intl_number_choice_option(
            options,
            IntlErrorOption::Style,
            DurationStyle::ALL.iter().map(|v| (v.name(), *v)),
            DurationStyle::Short,
            &base,
            f,
        )?;
        let previous = GcI32DomainLocal::new(schema, None::<DurationUnitStyle>, f);
        let previous_fractional = schema.reserve_i32_local(f);
        set_i32(previous_fractional, 0, f);
        let requested_style = GcI32DomainLocal::new(schema, None::<DurationUnitStyle>, f);
        let style = GcI32DomainLocal::new(schema, DurationUnitStyle::Short, f);
        let display_default = GcI32DomainLocal::new(schema, DurationDisplay::Auto, f);
        let requested_display = GcI32DomainLocal::new(schema, None::<DurationDisplay>, f);
        let display = GcI32DomainLocal::new(schema, DurationDisplay::Auto, f);
        let fractional = schema.reserve_i32_local(f);
        let mut units = Vec::with_capacity(DurationUnit::ALL.len());
        for &unit in DurationUnit::ALL {
            let index = unit.index();
            self.emit_intl_number_choice_option(
                options,
                unit.style_option(),
                DurationUnitStyle::ALL
                    .iter()
                    .filter(|s| {
                        (4..=6).contains(&index)
                            || s.index() < DurationUnitStyle::Numeric.index()
                            || index >= 7 && **s == DurationUnitStyle::Numeric
                    })
                    .map(|s| (s.name(), Some(*s))),
                None,
                &requested_style,
                f,
            )?;
            display_default.set_constant(DurationDisplay::Always, f);
            // Absence is the native optional domain, never an invented style code.
            emit_domain_is(&requested_style, None, f);
            self.open_frame(ControlFrameKind::If, f);
            emit_domain_is(&base, DurationStyle::Digital, f);
            self.open_frame(ControlFrameKind::If, f);
            style.set_constant(
                if index >= 4 {
                    DurationUnitStyle::Numeric
                } else {
                    DurationUnitStyle::Short
                },
                f,
            );
            display_default.set_constant(
                if (4..=6).contains(&index) {
                    DurationDisplay::Always
                } else {
                    DurationDisplay::Auto
                },
                f,
            );
            f.instruction(&Instruction::Else);
            emit_duration_previous_numeric(&previous, f);
            self.open_frame(ControlFrameKind::If, f);
            style.set_constant(DurationUnitStyle::Numeric, f);
            display_default.set_constant(
                if (5..=6).contains(&index) {
                    DurationDisplay::Always
                } else {
                    DurationDisplay::Auto
                },
                f,
            );
            f.instruction(&Instruction::Else);
            for (base_style, unit_style) in [
                (DurationStyle::Long, DurationUnitStyle::Long),
                (DurationStyle::Short, DurationUnitStyle::Short),
                (DurationStyle::Narrow, DurationUnitStyle::Narrow),
            ] {
                emit_domain_is(&base, base_style, f);
                self.open_frame(ControlFrameKind::If, f);
                style.set_constant(unit_style, f);
                self.pop_control(ControlFrameKind::If);
                f.instruction(&Instruction::End);
            }
            display_default.set_constant(DurationDisplay::Auto, f);
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
            f.instruction(&Instruction::Else);
            for &candidate in DurationUnitStyle::ALL {
                emit_domain_is(&requested_style, Some(candidate), f);
                self.open_frame(ControlFrameKind::If, f);
                style.set_constant(candidate, f);
                self.pop_control(ControlFrameKind::If);
                f.instruction(&Instruction::End);
            }
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
            set_i32(fractional, 0, f);
            if index >= 7 {
                emit_domain_is(&style, DurationUnitStyle::Numeric, f);
                fractional.store(f);
                fractional.load(f);
                self.open_frame(ControlFrameKind::If, f);
                display_default.set_constant(DurationDisplay::Auto, f);
                self.pop_control(ControlFrameKind::If);
                f.instruction(&Instruction::End);
            }
            // Display Get and coercion precede validation and locale width promotion.
            self.emit_intl_number_choice_option(
                options,
                unit.display_option(),
                DurationDisplay::ALL.iter().map(|v| (v.name(), Some(*v))),
                None,
                &requested_display,
                f,
            )?;
            display.copy_from(&display_default, f);
            for &candidate in DurationDisplay::ALL {
                emit_domain_is(&requested_display, Some(candidate), f);
                self.open_frame(ControlFrameKind::If, f);
                display.set_constant(candidate, f);
                self.pop_control(ControlFrameKind::If);
                f.instruction(&Instruction::End);
            }
            fractional.load(f);
            emit_domain_is(&display, DurationDisplay::Always, f);
            f.instruction(&Instruction::I32And);
            self.open_frame(ControlFrameKind::If, f);
            self.emit_intl_number_range_error(
                RuntimeErrorMessage::INVALID_FRACTIONAL_DURATION_DISPLAY,
                f,
            )?;
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
            previous_fractional.load(f);
            fractional.load(f);
            f.instruction(&Instruction::I32Eqz);
            f.instruction(&Instruction::I32And);
            emit_duration_previous_numeric(&previous, f);
            emit_domain_is(&style, DurationUnitStyle::Long, f);
            emit_domain_is(&style, DurationUnitStyle::Short, f);
            f.instruction(&Instruction::I32Or);
            emit_domain_is(&style, DurationUnitStyle::Narrow, f);
            f.instruction(&Instruction::I32Or);
            f.instruction(&Instruction::I32And);
            f.instruction(&Instruction::I32Or);
            self.open_frame(ControlFrameKind::If, f);
            self.emit_intl_number_range_error(
                RuntimeErrorMessage::INVALID_DURATION_UNIT_STYLE_SEQUENCE,
                f,
            )?;
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
            if index == DurationUnit::Hour.index() {
                two_digit_hours.load(f);
                self.open_frame(ControlFrameKind::If, f);
                style.set_constant(DurationUnitStyle::TwoDigit, f);
                self.pop_control(ControlFrameKind::If);
                f.instruction(&Instruction::End);
            }
            if matches!(unit, DurationUnit::Minute | DurationUnit::Second) {
                emit_duration_previous_numeric(&previous, f);
                self.open_frame(ControlFrameKind::If, f);
                style.set_constant(DurationUnitStyle::TwoDigit, f);
                self.pop_control(ControlFrameKind::If);
                f.instruction(&Instruction::End);
            }
            if (4..=8).contains(&index) {
                for &candidate in DurationUnitStyle::ALL {
                    emit_domain_is(&style, candidate, f);
                    self.open_frame(ControlFrameKind::If, f);
                    previous.set_constant(Some(candidate), f);
                    self.pop_control(ControlFrameKind::If);
                    f.instruction(&Instruction::End);
                }
                fractional.load(f);
                previous_fractional.store(f);
            }
            units.push(
                schema.reserve_gc_local(f).initialize(
                    schema
                        .struct_type::<IntlDurationUnitOptions>()
                        .construct((style.operand(), display.operand()), f),
                    f,
                ),
            );
        }
        let value = schema.reserve_value_local(f);
        self.emit_intl_number_get_option(options, "fractionalDigits", &value, f)?;
        emit_tag_is(&value, WasmRuntimeValueTag::Undefined, f);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        let fallback = schema.reserve_i32_local(f);
        set_i32(fallback, 0, f);
        let count = schema.reserve_i32_local(f);
        self.emit_intl_number_coerce_digit(
            &value,
            IntlErrorOption::FractionalDigits,
            0,
            9,
            fallback,
            count,
            f,
        )?;
        for candidate in 0..=9 {
            count.load(f);
            f.instruction(&Instruction::I32Const(candidate));
            f.instruction(&Instruction::I32Eq);
            self.open_frame(ControlFrameKind::If, f);
            digits.set_constant(
                Some(
                    DurationFractionalDigits::new(candidate as u8)
                        .expect("constant native fractional digit"),
                ),
                f,
            );
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
        }
        schema.release_i32_local(count, f);
        schema.release_i32_local(fallback, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        value.clear(f);
        schema.release_i32_local(fractional, f);
        display.clear(schema, f);
        requested_display.clear(schema, f);
        display_default.clear(schema, f);
        style.clear(schema, f);
        requested_style.clear(schema, f);
        schema.release_i32_local(previous_fractional, f);
        previous.clear(schema, f);
        Ok(CompletedDurationOptionsLocals {
            style: base,
            fractional_digits: digits,
            units: units
                .try_into()
                .unwrap_or_else(|_| unreachable!("ten native duration units")),
        })
    }
}
