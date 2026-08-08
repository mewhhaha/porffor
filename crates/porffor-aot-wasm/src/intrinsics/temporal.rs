//! `temporal` intrinsic installation.
//!
//! Extracted verbatim from `builtins/bootstrap.rs::init_builtin_constructor_object`.
//! Property installation order is observable through `Object.keys`, so the
//! statement order inside each installer is load-bearing — do not reorder.

use super::super::*;
use super::IntrinsicInstall;

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn install_temporal_instant_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // Re-bind the shared preamble values under the names the moved body
        // already uses, so the body below is a verbatim copy of the arm it
        // replaced. Most families read only a few of them.
        #[allow(unused_variables)]
        let IntrinsicInstall {
            builtin,
            meta,
            prototype_global_index,
            constructor_global_index,
            object_local,
            key_local,
            payload_local,
            tag_local,
            prototype_object_local,
        } = *context;

        // Property installation order is observable through `Object.keys`, so
        // the statics go on in specification order and every one of them is
        // installed before the prototype is touched.
        for (name, builtin) in [
            ("from", StandardBuiltinId::TemporalInstantFrom),
            (
                "fromEpochMilliseconds",
                StandardBuiltinId::TemporalInstantFromEpochMilliseconds,
            ),
            (
                "fromEpochNanoseconds",
                StandardBuiltinId::TemporalInstantFromEpochNanoseconds,
            ),
            ("compare", StandardBuiltinId::TemporalInstantCompare),
        ] {
            let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                    builtin.debug_name()
                ))
            })?;
            self.emit_object_define_function_data(object_local, name, meta, function)?;
        }
        function.instruction(&Instruction::GlobalGet(prototype_global_index));
        function.instruction(&Instruction::LocalSet(prototype_object_local));
        for (name, builtin) in [
            (
                "epochMilliseconds",
                StandardBuiltinId::TemporalInstantPrototypeEpochMillisecondsGetter,
            ),
            (
                "epochNanoseconds",
                StandardBuiltinId::TemporalInstantPrototypeEpochNanosecondsGetter,
            ),
        ] {
            let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                    builtin.debug_name()
                ))
            })?;
            function.instruction(&Instruction::I64Const(self.strings.payload(name)));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_function_value_payload(meta, function)?;
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            self.emit_object_append_accessor_property_with_flags(
                prototype_object_local,
                key_local,
                Some((payload_local, tag_local)),
                None,
                false,
                true,
                function,
            )?;
        }
        let to_string_meta = self
            .functions
            .get(&StandardBuiltinId::TemporalInstantPrototypeToString.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Temporal.Instant.prototype.toString`",
                )
            })?;
        self.emit_object_define_function_data(
            prototype_object_local,
            "toString",
            to_string_meta,
            function,
        )?;
        let equals_meta = self
            .functions
            .get(&StandardBuiltinId::TemporalInstantPrototypeEquals.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Temporal.Instant.prototype.equals`",
                )
            })?;
        self.emit_object_define_function_data(
            prototype_object_local,
            "equals",
            equals_meta,
            function,
        )?;
        // `toJSON` and `toString` share an emitter but never a function object,
        // so each gets its own meta here; installing them from one meta would
        // make `Temporal.Instant.prototype.toJSON === ...toString` true, which
        // `toJSON/prop-desc.js` and `toJSON/name.js` observe.
        for (name, builtin) in [
            ("toJSON", StandardBuiltinId::TemporalInstantPrototypeToJson),
            (
                "valueOf",
                StandardBuiltinId::TemporalInstantPrototypeValueOf,
            ),
        ] {
            let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                    builtin.debug_name()
                ))
            })?;
            self.emit_object_define_function_data(prototype_object_local, name, meta, function)?;
        }
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("Temporal.Instant"),
        ));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            prototype_object_local,
            key_local,
            payload_local,
            tag_local,
            false,
            false,
            true,
            function,
        )?;

        Ok(())
    }

    /// Property installation order is observable through `Object.keys`, so the
    /// sequence here — statics on the constructor, then prototype accessors,
    /// then prototype methods, then `Symbol.toStringTag` — matches the order
    /// declared in `lowering::temporal_plain_date_prototype_shape`.
    pub(crate) fn install_temporal_plain_date_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        #[allow(unused_variables)]
        let IntrinsicInstall {
            builtin,
            meta,
            prototype_global_index,
            constructor_global_index,
            object_local,
            key_local,
            payload_local,
            tag_local,
            prototype_object_local,
        } = *context;

        for (name, builtin) in [
            ("from", StandardBuiltinId::TemporalPlainDateFrom),
            ("compare", StandardBuiltinId::TemporalPlainDateCompare),
        ] {
            let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                    builtin.debug_name()
                ))
            })?;
            self.emit_object_define_function_data(object_local, name, meta, function)?;
        }
        function.instruction(&Instruction::GlobalGet(prototype_global_index));
        function.instruction(&Instruction::LocalSet(prototype_object_local));
        for (name, builtin) in [
            (
                "calendarId",
                StandardBuiltinId::TemporalPlainDatePrototypeCalendarIdGetter,
            ),
            (
                "era",
                StandardBuiltinId::TemporalPlainDatePrototypeEraGetter,
            ),
            (
                "eraYear",
                StandardBuiltinId::TemporalPlainDatePrototypeEraYearGetter,
            ),
            (
                "year",
                StandardBuiltinId::TemporalPlainDatePrototypeYearGetter,
            ),
            (
                "month",
                StandardBuiltinId::TemporalPlainDatePrototypeMonthGetter,
            ),
            (
                "monthCode",
                StandardBuiltinId::TemporalPlainDatePrototypeMonthCodeGetter,
            ),
            (
                "day",
                StandardBuiltinId::TemporalPlainDatePrototypeDayGetter,
            ),
            (
                "dayOfWeek",
                StandardBuiltinId::TemporalPlainDatePrototypeDayOfWeekGetter,
            ),
            (
                "dayOfYear",
                StandardBuiltinId::TemporalPlainDatePrototypeDayOfYearGetter,
            ),
            (
                "weekOfYear",
                StandardBuiltinId::TemporalPlainDatePrototypeWeekOfYearGetter,
            ),
            (
                "yearOfWeek",
                StandardBuiltinId::TemporalPlainDatePrototypeYearOfWeekGetter,
            ),
            (
                "daysInWeek",
                StandardBuiltinId::TemporalPlainDatePrototypeDaysInWeekGetter,
            ),
            (
                "daysInMonth",
                StandardBuiltinId::TemporalPlainDatePrototypeDaysInMonthGetter,
            ),
            (
                "daysInYear",
                StandardBuiltinId::TemporalPlainDatePrototypeDaysInYearGetter,
            ),
            (
                "monthsInYear",
                StandardBuiltinId::TemporalPlainDatePrototypeMonthsInYearGetter,
            ),
            (
                "inLeapYear",
                StandardBuiltinId::TemporalPlainDatePrototypeInLeapYearGetter,
            ),
        ] {
            let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                    builtin.debug_name()
                ))
            })?;
            function.instruction(&Instruction::I64Const(self.strings.payload(name)));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_function_value_payload(meta, function)?;
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            self.emit_object_append_accessor_property_with_flags(
                prototype_object_local,
                key_local,
                Some((payload_local, tag_local)),
                None,
                false,
                true,
                function,
            )?;
        }
        for (name, builtin) in [
            ("with", StandardBuiltinId::TemporalPlainDatePrototypeWith),
            (
                "withCalendar",
                StandardBuiltinId::TemporalPlainDatePrototypeWithCalendar,
            ),
            (
                "equals",
                StandardBuiltinId::TemporalPlainDatePrototypeEquals,
            ),
            (
                "toString",
                StandardBuiltinId::TemporalPlainDatePrototypeToString,
            ),
            (
                "toJSON",
                StandardBuiltinId::TemporalPlainDatePrototypeToJson,
            ),
            (
                "toLocaleString",
                StandardBuiltinId::TemporalPlainDatePrototypeToLocaleString,
            ),
            (
                "valueOf",
                StandardBuiltinId::TemporalPlainDatePrototypeValueOf,
            ),
            ("add", StandardBuiltinId::TemporalPlainDatePrototypeAdd),
            (
                "subtract",
                StandardBuiltinId::TemporalPlainDatePrototypeSubtract,
            ),
            ("until", StandardBuiltinId::TemporalPlainDatePrototypeUntil),
            ("since", StandardBuiltinId::TemporalPlainDatePrototypeSince),
            (
                "toPlainDateTime",
                StandardBuiltinId::TemporalPlainDatePrototypeToPlainDateTime,
            ),
            (
                "toPlainYearMonth",
                StandardBuiltinId::TemporalPlainDatePrototypeToPlainYearMonth,
            ),
            (
                "toPlainMonthDay",
                StandardBuiltinId::TemporalPlainDatePrototypeToPlainMonthDay,
            ),
        ] {
            let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                    builtin.debug_name()
                ))
            })?;
            self.emit_object_define_function_data(prototype_object_local, name, meta, function)?;
        }
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("Temporal.PlainDate"),
        ));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            prototype_object_local,
            key_local,
            payload_local,
            tag_local,
            false,
            false,
            true,
            function,
        )?;

        Ok(())
    }

    /// Temporal proposal 5.2/5.3: `Temporal.PlainDateTime`'s statics, then the
    /// twenty-two accessors, then the prototype methods, then
    /// `Symbol.toStringTag` - the order
    /// `lowering::temporal_plain_date_time_prototype_shape` declares, because
    /// property order is observable through `Object.keys`.
    pub(crate) fn install_temporal_plain_date_time_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        #[allow(unused_variables)]
        let IntrinsicInstall {
            builtin,
            meta,
            prototype_global_index,
            constructor_global_index,
            object_local,
            key_local,
            payload_local,
            tag_local,
            prototype_object_local,
        } = *context;

        for (name, builtin) in [
            ("from", StandardBuiltinId::TemporalPlainDateTimeFrom),
            ("compare", StandardBuiltinId::TemporalPlainDateTimeCompare),
        ] {
            let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                    builtin.debug_name()
                ))
            })?;
            self.emit_object_define_function_data(object_local, name, meta, function)?;
        }
        function.instruction(&Instruction::GlobalGet(prototype_global_index));
        function.instruction(&Instruction::LocalSet(prototype_object_local));
        for (name, builtin) in [
            (
                "calendarId",
                StandardBuiltinId::TemporalPlainDateTimePrototypeCalendarIdGetter,
            ),
            (
                "era",
                StandardBuiltinId::TemporalPlainDateTimePrototypeEraGetter,
            ),
            (
                "eraYear",
                StandardBuiltinId::TemporalPlainDateTimePrototypeEraYearGetter,
            ),
            (
                "year",
                StandardBuiltinId::TemporalPlainDateTimePrototypeYearGetter,
            ),
            (
                "month",
                StandardBuiltinId::TemporalPlainDateTimePrototypeMonthGetter,
            ),
            (
                "monthCode",
                StandardBuiltinId::TemporalPlainDateTimePrototypeMonthCodeGetter,
            ),
            (
                "day",
                StandardBuiltinId::TemporalPlainDateTimePrototypeDayGetter,
            ),
            (
                "hour",
                StandardBuiltinId::TemporalPlainDateTimePrototypeHourGetter,
            ),
            (
                "minute",
                StandardBuiltinId::TemporalPlainDateTimePrototypeMinuteGetter,
            ),
            (
                "second",
                StandardBuiltinId::TemporalPlainDateTimePrototypeSecondGetter,
            ),
            (
                "millisecond",
                StandardBuiltinId::TemporalPlainDateTimePrototypeMillisecondGetter,
            ),
            (
                "microsecond",
                StandardBuiltinId::TemporalPlainDateTimePrototypeMicrosecondGetter,
            ),
            (
                "nanosecond",
                StandardBuiltinId::TemporalPlainDateTimePrototypeNanosecondGetter,
            ),
            (
                "dayOfWeek",
                StandardBuiltinId::TemporalPlainDateTimePrototypeDayOfWeekGetter,
            ),
            (
                "dayOfYear",
                StandardBuiltinId::TemporalPlainDateTimePrototypeDayOfYearGetter,
            ),
            (
                "weekOfYear",
                StandardBuiltinId::TemporalPlainDateTimePrototypeWeekOfYearGetter,
            ),
            (
                "yearOfWeek",
                StandardBuiltinId::TemporalPlainDateTimePrototypeYearOfWeekGetter,
            ),
            (
                "daysInWeek",
                StandardBuiltinId::TemporalPlainDateTimePrototypeDaysInWeekGetter,
            ),
            (
                "daysInMonth",
                StandardBuiltinId::TemporalPlainDateTimePrototypeDaysInMonthGetter,
            ),
            (
                "daysInYear",
                StandardBuiltinId::TemporalPlainDateTimePrototypeDaysInYearGetter,
            ),
            (
                "monthsInYear",
                StandardBuiltinId::TemporalPlainDateTimePrototypeMonthsInYearGetter,
            ),
            (
                "inLeapYear",
                StandardBuiltinId::TemporalPlainDateTimePrototypeInLeapYearGetter,
            ),
        ] {
            let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                    builtin.debug_name()
                ))
            })?;
            function.instruction(&Instruction::I64Const(self.strings.payload(name)));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_function_value_payload(meta, function)?;
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            self.emit_object_append_accessor_property_with_flags(
                prototype_object_local,
                key_local,
                Some((payload_local, tag_local)),
                None,
                false,
                true,
                function,
            )?;
        }
        for (name, builtin) in [
            (
                "with",
                StandardBuiltinId::TemporalPlainDateTimePrototypeWith,
            ),
            (
                "withPlainTime",
                StandardBuiltinId::TemporalPlainDateTimePrototypeWithPlainTime,
            ),
            (
                "withCalendar",
                StandardBuiltinId::TemporalPlainDateTimePrototypeWithCalendar,
            ),
            ("add", StandardBuiltinId::TemporalPlainDateTimePrototypeAdd),
            (
                "subtract",
                StandardBuiltinId::TemporalPlainDateTimePrototypeSubtract,
            ),
            (
                "until",
                StandardBuiltinId::TemporalPlainDateTimePrototypeUntil,
            ),
            (
                "since",
                StandardBuiltinId::TemporalPlainDateTimePrototypeSince,
            ),
            (
                "round",
                StandardBuiltinId::TemporalPlainDateTimePrototypeRound,
            ),
            (
                "equals",
                StandardBuiltinId::TemporalPlainDateTimePrototypeEquals,
            ),
            (
                "toString",
                StandardBuiltinId::TemporalPlainDateTimePrototypeToString,
            ),
            (
                "toJSON",
                StandardBuiltinId::TemporalPlainDateTimePrototypeToJson,
            ),
            (
                "toLocaleString",
                StandardBuiltinId::TemporalPlainDateTimePrototypeToLocaleString,
            ),
            (
                "valueOf",
                StandardBuiltinId::TemporalPlainDateTimePrototypeValueOf,
            ),
            (
                "toPlainDate",
                StandardBuiltinId::TemporalPlainDateTimePrototypeToPlainDate,
            ),
            (
                "toPlainTime",
                StandardBuiltinId::TemporalPlainDateTimePrototypeToPlainTime,
            ),
            (
                "toZonedDateTime",
                StandardBuiltinId::TemporalPlainDateTimePrototypeToZonedDateTime,
            ),
        ] {
            let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                    builtin.debug_name()
                ))
            })?;
            self.emit_object_define_function_data(prototype_object_local, name, meta, function)?;
        }
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("Temporal.PlainDateTime"),
        ));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            prototype_object_local,
            key_local,
            payload_local,
            tag_local,
            false,
            false,
            true,
            function,
        )?;

        Ok(())
    }

    /// Temporal proposal 7.2/7.3: `Temporal.Duration`'s statics, then the ten
    /// unit accessors plus `sign`/`blank`, then the prototype methods, then
    /// `Symbol.toStringTag` — the order `lowering::temporal_duration_prototype_shape`
    /// declares, because property order is observable through `Object.keys`.
    pub(crate) fn install_temporal_duration_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        #[allow(unused_variables)]
        let IntrinsicInstall {
            builtin,
            meta,
            prototype_global_index,
            constructor_global_index,
            object_local,
            key_local,
            payload_local,
            tag_local,
            prototype_object_local,
        } = *context;

        for (name, builtin) in [
            ("from", StandardBuiltinId::TemporalDurationFrom),
            ("compare", StandardBuiltinId::TemporalDurationCompare),
        ] {
            let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                    builtin.debug_name()
                ))
            })?;
            self.emit_object_define_function_data(object_local, name, meta, function)?;
        }
        function.instruction(&Instruction::GlobalGet(prototype_global_index));
        function.instruction(&Instruction::LocalSet(prototype_object_local));
        for (name, builtin) in [
            (
                "years",
                StandardBuiltinId::TemporalDurationPrototypeYearsGetter,
            ),
            (
                "months",
                StandardBuiltinId::TemporalDurationPrototypeMonthsGetter,
            ),
            (
                "weeks",
                StandardBuiltinId::TemporalDurationPrototypeWeeksGetter,
            ),
            (
                "days",
                StandardBuiltinId::TemporalDurationPrototypeDaysGetter,
            ),
            (
                "hours",
                StandardBuiltinId::TemporalDurationPrototypeHoursGetter,
            ),
            (
                "minutes",
                StandardBuiltinId::TemporalDurationPrototypeMinutesGetter,
            ),
            (
                "seconds",
                StandardBuiltinId::TemporalDurationPrototypeSecondsGetter,
            ),
            (
                "milliseconds",
                StandardBuiltinId::TemporalDurationPrototypeMillisecondsGetter,
            ),
            (
                "microseconds",
                StandardBuiltinId::TemporalDurationPrototypeMicrosecondsGetter,
            ),
            (
                "nanoseconds",
                StandardBuiltinId::TemporalDurationPrototypeNanosecondsGetter,
            ),
            (
                "sign",
                StandardBuiltinId::TemporalDurationPrototypeSignGetter,
            ),
            (
                "blank",
                StandardBuiltinId::TemporalDurationPrototypeBlankGetter,
            ),
        ] {
            let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                    builtin.debug_name()
                ))
            })?;
            function.instruction(&Instruction::I64Const(self.strings.payload(name)));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_function_value_payload(meta, function)?;
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            self.emit_object_append_accessor_property_with_flags(
                prototype_object_local,
                key_local,
                Some((payload_local, tag_local)),
                None,
                false,
                true,
                function,
            )?;
        }
        for (name, builtin) in [
            ("with", StandardBuiltinId::TemporalDurationPrototypeWith),
            (
                "negated",
                StandardBuiltinId::TemporalDurationPrototypeNegated,
            ),
            ("abs", StandardBuiltinId::TemporalDurationPrototypeAbs),
            ("add", StandardBuiltinId::TemporalDurationPrototypeAdd),
            (
                "subtract",
                StandardBuiltinId::TemporalDurationPrototypeSubtract,
            ),
            ("round", StandardBuiltinId::TemporalDurationPrototypeRound),
            ("total", StandardBuiltinId::TemporalDurationPrototypeTotal),
            (
                "toString",
                StandardBuiltinId::TemporalDurationPrototypeToString,
            ),
            ("toJSON", StandardBuiltinId::TemporalDurationPrototypeToJson),
            (
                "toLocaleString",
                StandardBuiltinId::TemporalDurationPrototypeToLocaleString,
            ),
            (
                "valueOf",
                StandardBuiltinId::TemporalDurationPrototypeValueOf,
            ),
        ] {
            let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                    builtin.debug_name()
                ))
            })?;
            self.emit_object_define_function_data(prototype_object_local, name, meta, function)?;
        }
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("Temporal.Duration"),
        ));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            prototype_object_local,
            key_local,
            payload_local,
            tag_local,
            false,
            false,
            true,
            function,
        )?;

        Ok(())
    }

    /// Temporal proposal 4.2/4.3: `Temporal.PlainTime`'s statics, then the six
    /// unit accessors, then the prototype methods, then `Symbol.toStringTag` —
    /// the order `lowering::temporal_plain_time_prototype_shape` declares,
    /// because property order is observable through `Object.keys`.
    pub(crate) fn install_temporal_plain_time_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        #[allow(unused_variables)]
        let IntrinsicInstall {
            builtin,
            meta,
            prototype_global_index,
            constructor_global_index,
            object_local,
            key_local,
            payload_local,
            tag_local,
            prototype_object_local,
        } = *context;

        for (name, builtin) in [
            ("from", StandardBuiltinId::TemporalPlainTimeFrom),
            ("compare", StandardBuiltinId::TemporalPlainTimeCompare),
        ] {
            let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                    builtin.debug_name()
                ))
            })?;
            self.emit_object_define_function_data(object_local, name, meta, function)?;
        }
        function.instruction(&Instruction::GlobalGet(prototype_global_index));
        function.instruction(&Instruction::LocalSet(prototype_object_local));
        for (name, builtin) in [
            (
                "hour",
                StandardBuiltinId::TemporalPlainTimePrototypeHourGetter,
            ),
            (
                "minute",
                StandardBuiltinId::TemporalPlainTimePrototypeMinuteGetter,
            ),
            (
                "second",
                StandardBuiltinId::TemporalPlainTimePrototypeSecondGetter,
            ),
            (
                "millisecond",
                StandardBuiltinId::TemporalPlainTimePrototypeMillisecondGetter,
            ),
            (
                "microsecond",
                StandardBuiltinId::TemporalPlainTimePrototypeMicrosecondGetter,
            ),
            (
                "nanosecond",
                StandardBuiltinId::TemporalPlainTimePrototypeNanosecondGetter,
            ),
        ] {
            let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                    builtin.debug_name()
                ))
            })?;
            function.instruction(&Instruction::I64Const(self.strings.payload(name)));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_function_value_payload(meta, function)?;
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            self.emit_object_append_accessor_property_with_flags(
                prototype_object_local,
                key_local,
                Some((payload_local, tag_local)),
                None,
                false,
                true,
                function,
            )?;
        }
        for (name, builtin) in [
            ("add", StandardBuiltinId::TemporalPlainTimePrototypeAdd),
            (
                "subtract",
                StandardBuiltinId::TemporalPlainTimePrototypeSubtract,
            ),
            ("with", StandardBuiltinId::TemporalPlainTimePrototypeWith),
            ("until", StandardBuiltinId::TemporalPlainTimePrototypeUntil),
            ("since", StandardBuiltinId::TemporalPlainTimePrototypeSince),
            ("round", StandardBuiltinId::TemporalPlainTimePrototypeRound),
            (
                "equals",
                StandardBuiltinId::TemporalPlainTimePrototypeEquals,
            ),
            (
                "toString",
                StandardBuiltinId::TemporalPlainTimePrototypeToString,
            ),
            (
                "toJSON",
                StandardBuiltinId::TemporalPlainTimePrototypeToJson,
            ),
            (
                "toLocaleString",
                StandardBuiltinId::TemporalPlainTimePrototypeToLocaleString,
            ),
            (
                "valueOf",
                StandardBuiltinId::TemporalPlainTimePrototypeValueOf,
            ),
        ] {
            let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                    builtin.debug_name()
                ))
            })?;
            self.emit_object_define_function_data(prototype_object_local, name, meta, function)?;
        }
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("Temporal.PlainTime"),
        ));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            prototype_object_local,
            key_local,
            payload_local,
            tag_local,
            false,
            false,
            true,
            function,
        )?;

        Ok(())
    }

    pub(crate) fn install_temporal_zoned_date_time_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // Re-bind the shared preamble values under the names the moved body
        // already uses, so the body below is a verbatim copy of the arm it
        // replaced. Most families read only a few of them.
        #[allow(unused_variables)]
        let IntrinsicInstall {
            builtin,
            meta,
            prototype_global_index,
            constructor_global_index,
            object_local,
            key_local,
            payload_local,
            tag_local,
            prototype_object_local,
        } = *context;

        let from_meta = self
            .functions
            .get(&StandardBuiltinId::TemporalZonedDateTimeFrom.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Temporal.ZonedDateTime.from`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "from", from_meta, function)?;
        function.instruction(&Instruction::GlobalGet(prototype_global_index));
        function.instruction(&Instruction::LocalSet(prototype_object_local));
        for (name, builtin) in [
            (
                "epochMilliseconds",
                StandardBuiltinId::TemporalZonedDateTimePrototypeEpochMillisecondsGetter,
            ),
            (
                "epochNanoseconds",
                StandardBuiltinId::TemporalZonedDateTimePrototypeEpochNanosecondsGetter,
            ),
            (
                "offset",
                StandardBuiltinId::TemporalZonedDateTimePrototypeOffsetGetter,
            ),
            (
                "offsetNanoseconds",
                StandardBuiltinId::TemporalZonedDateTimePrototypeOffsetNanosecondsGetter,
            ),
            (
                "timeZoneId",
                StandardBuiltinId::TemporalZonedDateTimePrototypeTimeZoneIdGetter,
            ),
            (
                "calendarId",
                StandardBuiltinId::TemporalZonedDateTimePrototypeCalendarIdGetter,
            ),
            (
                "year",
                StandardBuiltinId::TemporalZonedDateTimePrototypeYearGetter,
            ),
            (
                "month",
                StandardBuiltinId::TemporalZonedDateTimePrototypeMonthGetter,
            ),
            (
                "monthCode",
                StandardBuiltinId::TemporalZonedDateTimePrototypeMonthCodeGetter,
            ),
            (
                "day",
                StandardBuiltinId::TemporalZonedDateTimePrototypeDayGetter,
            ),
            (
                "hour",
                StandardBuiltinId::TemporalZonedDateTimePrototypeHourGetter,
            ),
            (
                "minute",
                StandardBuiltinId::TemporalZonedDateTimePrototypeMinuteGetter,
            ),
            (
                "second",
                StandardBuiltinId::TemporalZonedDateTimePrototypeSecondGetter,
            ),
            (
                "millisecond",
                StandardBuiltinId::TemporalZonedDateTimePrototypeMillisecondGetter,
            ),
            (
                "microsecond",
                StandardBuiltinId::TemporalZonedDateTimePrototypeMicrosecondGetter,
            ),
            (
                "nanosecond",
                StandardBuiltinId::TemporalZonedDateTimePrototypeNanosecondGetter,
            ),
        ] {
            let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                    builtin.debug_name()
                ))
            })?;
            function.instruction(&Instruction::I64Const(self.strings.payload(name)));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_function_value_payload(meta, function)?;
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            self.emit_object_append_accessor_property_with_flags(
                prototype_object_local,
                key_local,
                Some((payload_local, tag_local)),
                None,
                false,
                true,
                function,
            )?;
        }
        let equals_meta = self
            .functions
            .get(&StandardBuiltinId::TemporalZonedDateTimePrototypeEquals.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Temporal.ZonedDateTime.prototype.equals`",
                )
            })?;
        self.emit_object_define_function_data(
            prototype_object_local,
            "equals",
            equals_meta,
            function,
        )?;
        let to_instant_meta = self
            .functions
            .get(&StandardBuiltinId::TemporalZonedDateTimePrototypeToInstant.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Temporal.ZonedDateTime.prototype.toInstant`",
                )
            })?;
        self.emit_object_define_function_data(
            prototype_object_local,
            "toInstant",
            to_instant_meta,
            function,
        )?;
        let with_time_zone_meta = self
            .functions
            .get(
                &StandardBuiltinId::TemporalZonedDateTimePrototypeWithTimeZone
                    .function_id(),
            )
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Temporal.ZonedDateTime.prototype.withTimeZone`",
                )
            })?;
        self.emit_object_define_function_data(
            prototype_object_local,
            "withTimeZone",
            with_time_zone_meta,
            function,
        )?;
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("Temporal.ZonedDateTime"),
        ));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            prototype_object_local,
            key_local,
            payload_local,
            tag_local,
            false,
            false,
            true,
            function,
        )?;

        Ok(())
    }

    /// Property installation order is observable through `Object.keys`, so the
    /// sequence here - statics on the constructor, then prototype accessors,
    /// then prototype methods, then `Symbol.toStringTag` - matches the order
    /// `lowering::temporal_plain_year_month_prototype_shape` declares.
    pub(crate) fn install_temporal_plain_year_month_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        #[allow(unused_variables)]
        let IntrinsicInstall {
            builtin,
            meta,
            prototype_global_index,
            constructor_global_index,
            object_local,
            key_local,
            payload_local,
            tag_local,
            prototype_object_local,
        } = *context;

        for (name, builtin) in [
            ("from", StandardBuiltinId::TemporalPlainYearMonthFrom),
            ("compare", StandardBuiltinId::TemporalPlainYearMonthCompare),
        ] {
            let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                    builtin.debug_name()
                ))
            })?;
            self.emit_object_define_function_data(object_local, name, meta, function)?;
        }
        function.instruction(&Instruction::GlobalGet(prototype_global_index));
        function.instruction(&Instruction::LocalSet(prototype_object_local));
        for (name, builtin) in [
            (
                "calendarId",
                StandardBuiltinId::TemporalPlainYearMonthPrototypeCalendarIdGetter,
            ),
            (
                "era",
                StandardBuiltinId::TemporalPlainYearMonthPrototypeEraGetter,
            ),
            (
                "eraYear",
                StandardBuiltinId::TemporalPlainYearMonthPrototypeEraYearGetter,
            ),
            (
                "year",
                StandardBuiltinId::TemporalPlainYearMonthPrototypeYearGetter,
            ),
            (
                "month",
                StandardBuiltinId::TemporalPlainYearMonthPrototypeMonthGetter,
            ),
            (
                "monthCode",
                StandardBuiltinId::TemporalPlainYearMonthPrototypeMonthCodeGetter,
            ),
            (
                "daysInYear",
                StandardBuiltinId::TemporalPlainYearMonthPrototypeDaysInYearGetter,
            ),
            (
                "daysInMonth",
                StandardBuiltinId::TemporalPlainYearMonthPrototypeDaysInMonthGetter,
            ),
            (
                "monthsInYear",
                StandardBuiltinId::TemporalPlainYearMonthPrototypeMonthsInYearGetter,
            ),
            (
                "inLeapYear",
                StandardBuiltinId::TemporalPlainYearMonthPrototypeInLeapYearGetter,
            ),
        ] {
            let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                    builtin.debug_name()
                ))
            })?;
            function.instruction(&Instruction::I64Const(self.strings.payload(name)));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_function_value_payload(meta, function)?;
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            self.emit_object_append_accessor_property_with_flags(
                prototype_object_local,
                key_local,
                Some((payload_local, tag_local)),
                None,
                false,
                true,
                function,
            )?;
        }
        for (name, builtin) in [
            (
                "with",
                StandardBuiltinId::TemporalPlainYearMonthPrototypeWith,
            ),
            ("add", StandardBuiltinId::TemporalPlainYearMonthPrototypeAdd),
            (
                "subtract",
                StandardBuiltinId::TemporalPlainYearMonthPrototypeSubtract,
            ),
            (
                "until",
                StandardBuiltinId::TemporalPlainYearMonthPrototypeUntil,
            ),
            (
                "since",
                StandardBuiltinId::TemporalPlainYearMonthPrototypeSince,
            ),
            (
                "equals",
                StandardBuiltinId::TemporalPlainYearMonthPrototypeEquals,
            ),
            (
                "toString",
                StandardBuiltinId::TemporalPlainYearMonthPrototypeToString,
            ),
            (
                "toJSON",
                StandardBuiltinId::TemporalPlainYearMonthPrototypeToJson,
            ),
            (
                "toLocaleString",
                StandardBuiltinId::TemporalPlainYearMonthPrototypeToLocaleString,
            ),
            (
                "valueOf",
                StandardBuiltinId::TemporalPlainYearMonthPrototypeValueOf,
            ),
            (
                "toPlainDate",
                StandardBuiltinId::TemporalPlainYearMonthPrototypeToPlainDate,
            ),
        ] {
            let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                    builtin.debug_name()
                ))
            })?;
            self.emit_object_define_function_data(prototype_object_local, name, meta, function)?;
        }
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("Temporal.PlainYearMonth"),
        ));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            prototype_object_local,
            key_local,
            payload_local,
            tag_local,
            false,
            false,
            true,
            function,
        )?;

        Ok(())
    }

    /// Property installation order is observable through `Object.keys`, so the
    /// sequence here - statics on the constructor, then prototype accessors,
    /// then prototype methods, then `Symbol.toStringTag` - matches the order
    /// `lowering::temporal_plain_month_day_prototype_shape` declares.
    pub(crate) fn install_temporal_plain_month_day_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        #[allow(unused_variables)]
        let IntrinsicInstall {
            builtin,
            meta,
            prototype_global_index,
            constructor_global_index,
            object_local,
            key_local,
            payload_local,
            tag_local,
            prototype_object_local,
        } = *context;

        for (name, builtin) in [("from", StandardBuiltinId::TemporalPlainMonthDayFrom)] {
            let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                    builtin.debug_name()
                ))
            })?;
            self.emit_object_define_function_data(object_local, name, meta, function)?;
        }
        function.instruction(&Instruction::GlobalGet(prototype_global_index));
        function.instruction(&Instruction::LocalSet(prototype_object_local));
        for (name, builtin) in [
            (
                "calendarId",
                StandardBuiltinId::TemporalPlainMonthDayPrototypeCalendarIdGetter,
            ),
            (
                "monthCode",
                StandardBuiltinId::TemporalPlainMonthDayPrototypeMonthCodeGetter,
            ),
            (
                "day",
                StandardBuiltinId::TemporalPlainMonthDayPrototypeDayGetter,
            ),
        ] {
            let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                    builtin.debug_name()
                ))
            })?;
            function.instruction(&Instruction::I64Const(self.strings.payload(name)));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_function_value_payload(meta, function)?;
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            self.emit_object_append_accessor_property_with_flags(
                prototype_object_local,
                key_local,
                Some((payload_local, tag_local)),
                None,
                false,
                true,
                function,
            )?;
        }
        for (name, builtin) in [
            (
                "with",
                StandardBuiltinId::TemporalPlainMonthDayPrototypeWith,
            ),
            (
                "equals",
                StandardBuiltinId::TemporalPlainMonthDayPrototypeEquals,
            ),
            (
                "toString",
                StandardBuiltinId::TemporalPlainMonthDayPrototypeToString,
            ),
            (
                "toJSON",
                StandardBuiltinId::TemporalPlainMonthDayPrototypeToJson,
            ),
            (
                "toLocaleString",
                StandardBuiltinId::TemporalPlainMonthDayPrototypeToLocaleString,
            ),
            (
                "valueOf",
                StandardBuiltinId::TemporalPlainMonthDayPrototypeValueOf,
            ),
            (
                "toPlainDate",
                StandardBuiltinId::TemporalPlainMonthDayPrototypeToPlainDate,
            ),
        ] {
            let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                    builtin.debug_name()
                ))
            })?;
            self.emit_object_define_function_data(prototype_object_local, name, meta, function)?;
        }
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("Temporal.PlainMonthDay"),
        ));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            prototype_object_local,
            key_local,
            payload_local,
            tag_local,
            false,
            false,
            true,
            function,
        )?;

        Ok(())
    }
}
