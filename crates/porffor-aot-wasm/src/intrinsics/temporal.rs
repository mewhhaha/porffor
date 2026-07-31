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

        let from_meta = self
            .functions
            .get(&StandardBuiltinId::TemporalInstantFrom.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Temporal.Instant.from`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "from", from_meta, function)?;
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
}
