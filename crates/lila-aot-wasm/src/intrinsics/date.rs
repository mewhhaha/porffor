//! `date` intrinsic installation.
//!
//! Extracted verbatim from `builtins/bootstrap.rs::init_builtin_constructor_object`.
//! Property installation order is observable through `Object.keys`, so the
//! statement order inside each installer is load-bearing — do not reorder.

use super::super::*;
use super::IntrinsicInstall;

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn install_date_constructor_intrinsics(
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

        let now_meta = self
            .functions
            .get(&StandardBuiltinId::DateNow.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Date.now`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "now", now_meta, function)?;
        let parse_meta = self
            .functions
            .get(&StandardBuiltinId::DateParse.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Date.parse`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "parse", parse_meta, function)?;
        let date_utc_meta = self
            .functions
            .get(&StandardBuiltinId::DateUtc.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Date.UTC`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "UTC", date_utc_meta, function)?;
        function.instruction(&Instruction::GlobalGet(prototype_global_index));
        function.instruction(&Instruction::LocalSet(object_local));
        let utc_meta = self
            .functions
            .get(&StandardBuiltinId::DatePrototypeToUtcString.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Date.prototype.toUTCString`",
                )
            })?;
        for (name, builtin) in [
            ("getTime", StandardBuiltinId::DatePrototypeGetTime),
            ("setTime", StandardBuiltinId::DatePrototypeSetTime),
            ("valueOf", StandardBuiltinId::DatePrototypeValueOf),
            ("getFullYear", StandardBuiltinId::DatePrototypeGetFullYear),
            (
                "getUTCFullYear",
                StandardBuiltinId::DatePrototypeGetUtcFullYear,
            ),
            ("getMonth", StandardBuiltinId::DatePrototypeGetMonth),
            ("getUTCMonth", StandardBuiltinId::DatePrototypeGetUtcMonth),
            ("getDate", StandardBuiltinId::DatePrototypeGetDate),
            ("getUTCDate", StandardBuiltinId::DatePrototypeGetUtcDate),
            ("getDay", StandardBuiltinId::DatePrototypeGetDay),
            ("getUTCDay", StandardBuiltinId::DatePrototypeGetUtcDay),
            ("getHours", StandardBuiltinId::DatePrototypeGetHours),
            ("getUTCHours", StandardBuiltinId::DatePrototypeGetUtcHours),
            ("getMinutes", StandardBuiltinId::DatePrototypeGetMinutes),
            (
                "getUTCMinutes",
                StandardBuiltinId::DatePrototypeGetUtcMinutes,
            ),
            ("getSeconds", StandardBuiltinId::DatePrototypeGetSeconds),
            (
                "getUTCSeconds",
                StandardBuiltinId::DatePrototypeGetUtcSeconds,
            ),
            (
                "getMilliseconds",
                StandardBuiltinId::DatePrototypeGetMilliseconds,
            ),
            (
                "getUTCMilliseconds",
                StandardBuiltinId::DatePrototypeGetUtcMilliseconds,
            ),
            (
                "getTimezoneOffset",
                StandardBuiltinId::DatePrototypeGetTimezoneOffset,
            ),
            ("getYear", StandardBuiltinId::DatePrototypeGetYear),
            ("setYear", StandardBuiltinId::DatePrototypeSetYear),
            ("setFullYear", StandardBuiltinId::DatePrototypeSetFullYear),
            (
                "setUTCFullYear",
                StandardBuiltinId::DatePrototypeSetUtcFullYear,
            ),
            ("setMonth", StandardBuiltinId::DatePrototypeSetMonth),
            ("setUTCMonth", StandardBuiltinId::DatePrototypeSetUtcMonth),
            ("setDate", StandardBuiltinId::DatePrototypeSetDate),
            ("setUTCDate", StandardBuiltinId::DatePrototypeSetUtcDate),
            ("setHours", StandardBuiltinId::DatePrototypeSetHours),
            ("setUTCHours", StandardBuiltinId::DatePrototypeSetUtcHours),
            ("setMinutes", StandardBuiltinId::DatePrototypeSetMinutes),
            (
                "setUTCMinutes",
                StandardBuiltinId::DatePrototypeSetUtcMinutes,
            ),
            ("setSeconds", StandardBuiltinId::DatePrototypeSetSeconds),
            (
                "setUTCSeconds",
                StandardBuiltinId::DatePrototypeSetUtcSeconds,
            ),
            (
                "setMilliseconds",
                StandardBuiltinId::DatePrototypeSetMilliseconds,
            ),
            (
                "setUTCMilliseconds",
                StandardBuiltinId::DatePrototypeSetUtcMilliseconds,
            ),
            ("toISOString", StandardBuiltinId::DatePrototypeToIsoString),
            ("toJSON", StandardBuiltinId::DatePrototypeToJson),
            ("toDateString", StandardBuiltinId::DatePrototypeToDateString),
            (
                "toLocaleDateString",
                StandardBuiltinId::DatePrototypeToLocaleDateString,
            ),
            (
                "toLocaleString",
                StandardBuiltinId::DatePrototypeToLocaleString,
            ),
            (
                "toLocaleTimeString",
                StandardBuiltinId::DatePrototypeToLocaleTimeString,
            ),
            (
                "toTemporalInstant",
                StandardBuiltinId::DatePrototypeToTemporalInstant,
            ),
            ("toTimeString", StandardBuiltinId::DatePrototypeToTimeString),
            ("toString", StandardBuiltinId::DatePrototypeToString),
        ] {
            let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `{}`",
                    builtin.debug_name()
                ))
            })?;
            self.emit_object_define_function_data(object_local, name, meta, function)?;
        }
        let to_primitive_meta = self
            .functions
            .get(&StandardBuiltinId::DatePrototypeToPrimitive.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Date.prototype[Symbol.toPrimitive]`",
                )
            })?;
        let to_primitive_key_local = self.reserve_temp_local();
        let to_primitive_payload_local = self.reserve_temp_local();
        let to_primitive_tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toPrimitive"),
        ));
        function.instruction(&Instruction::LocalSet(to_primitive_key_local));
        self.emit_function_value_payload(&to_primitive_meta, function)?;
        function.instruction(&Instruction::LocalSet(to_primitive_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(to_primitive_tag_local));
        self.emit_object_append_data_property_with_flags(
            object_local,
            to_primitive_key_local,
            to_primitive_payload_local,
            to_primitive_tag_local,
            false,
            false,
            true,
            function,
        )?;
        self.release_temp_local(to_primitive_tag_local);
        self.release_temp_local(to_primitive_payload_local);
        self.release_temp_local(to_primitive_key_local);
        let utc_payload_local = self.reserve_temp_local();
        let utc_tag_local = self.reserve_temp_local();
        self.emit_function_value_payload(utc_meta, function)?;
        function.instruction(&Instruction::LocalSet(utc_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(utc_tag_local));
        self.emit_object_append_local_data_property_with_flags(
            object_local,
            "toUTCString",
            utc_payload_local,
            utc_tag_local,
            true,
            false,
            true,
            function,
        )?;
        self.emit_object_append_local_data_property_with_flags(
            object_local,
            "toGMTString",
            utc_payload_local,
            utc_tag_local,
            true,
            false,
            true,
            function,
        )?;
        self.release_temp_local(utc_tag_local);
        self.release_temp_local(utc_payload_local);

        Ok(())
    }
}
