//! `numeric` intrinsic installation.
//!
//! Extracted verbatim from `builtins/bootstrap.rs::init_builtin_constructor_object`.
//! Property installation order is observable through `Object.keys`, so the
//! statement order inside each installer is load-bearing — do not reorder.

use super::super::*;
use super::IntrinsicInstall;
use crate::functions::NonArrayRealmIntrinsicSlot;

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn install_big_int_constructor_intrinsics(
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

        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(prototype_object_local));
        function.instruction(&Instruction::GlobalGet(CURRENT_REALM_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_store_non_array_realm_intrinsic(
            self.scratch_local,
            NonArrayRealmIntrinsicSlot::BigIntPrototype,
            prototype_object_local,
            function,
        );
        self.store_i64_const_at_offset(
            object_local,
            HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
            ValueKind::Object.tag() as u64,
            function,
        );
        self.store_i64_local_at_offset(
            object_local,
            HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
            prototype_object_local,
            function,
        );
        function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::LocalGet(prototype_object_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_define_data_with_configurable(
            object_local,
            key_local,
            payload_local,
            tag_local,
            false,
            false,
            false,
            function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_define_data(
            prototype_object_local,
            key_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("BigInt")));
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
        for (name, builtin) in [
            ("toString", StandardBuiltinId::BigIntPrototypeToString),
            (
                "toLocaleString",
                StandardBuiltinId::BigIntPrototypeToLocaleString,
            ),
            ("valueOf", StandardBuiltinId::BigIntPrototypeValueOf),
        ] {
            if let Some(meta) = self.functions.get(&builtin.function_id()) {
                self.emit_object_define_function_data(
                    prototype_object_local,
                    name,
                    meta,
                    function,
                )?;
            }
        }
        for (name, builtin) in [
            ("asIntN", StandardBuiltinId::BigIntAsIntN),
            ("asUintN", StandardBuiltinId::BigIntAsUintN),
        ] {
            if let Some(meta) = self.functions.get(&builtin.function_id()) {
                self.emit_object_define_function_data(object_local, name, meta, function)?;
            }
        }

        Ok(())
    }

    pub(crate) fn install_number_constructor_intrinsics(
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

        function.instruction(&Instruction::GlobalGet(NUMBER_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(prototype_object_local));
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_store_boxed_primitive_metadata(
            prototype_object_local,
            BOXED_PRIMITIVE_KIND_NUMBER,
            payload_local,
            tag_local,
            function,
        );
        for (name, value) in [
            ("NaN", f64::NAN),
            ("POSITIVE_INFINITY", f64::INFINITY),
            ("NEGATIVE_INFINITY", f64::NEG_INFINITY),
            ("MAX_VALUE", f64::MAX),
            ("MIN_VALUE", f64::from_bits(1)),
            ("EPSILON", f64::EPSILON),
            ("MAX_SAFE_INTEGER", 9007199254740991.0),
            ("MIN_SAFE_INTEGER", -9007199254740991.0),
        ] {
            function.instruction(&Instruction::I64Const(self.strings.payload(name)));
            function.instruction(&Instruction::LocalSet(key_local));
            function.instruction(&Instruction::F64Const(Ieee64::from(value)));
            function.instruction(&Instruction::I64ReinterpretF64);
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            self.emit_object_append_data_property_with_flags(
                object_local,
                key_local,
                payload_local,
                tag_local,
                false,
                false,
                false,
                function,
            )?;
        }
        let is_integer_meta = self
            .functions
            .get(&StandardBuiltinId::NumberIsInteger.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Number.isInteger`",
                )
            })?;
        self.emit_object_define_function_data(
            object_local,
            "isInteger",
            is_integer_meta,
            function,
        )?;
        let is_safe_integer_meta = self
            .functions
            .get(&StandardBuiltinId::NumberIsSafeInteger.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Number.isSafeInteger`",
                )
            })?;
        self.emit_object_define_function_data(
            object_local,
            "isSafeInteger",
            is_safe_integer_meta,
            function,
        )?;
        let is_finite_meta = self
            .functions
            .get(&StandardBuiltinId::NumberIsFinite.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Number.isFinite`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "isFinite", is_finite_meta, function)?;
        let is_nan_meta = self
            .functions
            .get(&StandardBuiltinId::NumberIsNaN.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Number.isNaN`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "isNaN", is_nan_meta, function)?;
        let parse_int_meta = self
            .functions
            .get(&HostBuiltinId::ParseInt.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Number.parseInt`",
                )
            })?;
        self.emit_ensure_canonical_host_function(
            &parse_int_meta,
            PARSE_INT_FUNCTION_GLOBAL_INDEX,
            function,
        )?;
        self.emit_object_define_function_global_data(
            object_local,
            "parseInt",
            PARSE_INT_FUNCTION_GLOBAL_INDEX,
            function,
        )?;
        let parse_float_meta = self
            .functions
            .get(&HostBuiltinId::ParseFloat.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Number.parseFloat`",
                )
            })?;
        self.emit_ensure_canonical_host_function(
            &parse_float_meta,
            PARSE_FLOAT_FUNCTION_GLOBAL_INDEX,
            function,
        )?;
        self.emit_object_define_function_global_data(
            object_local,
            "parseFloat",
            PARSE_FLOAT_FUNCTION_GLOBAL_INDEX,
            function,
        )?;
        function.instruction(&Instruction::GlobalGet(NUMBER_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(prototype_object_local));
        for (name, builtin) in [
            ("toFixed", StandardBuiltinId::NumberPrototypeToFixed),
            (
                "toExponential",
                StandardBuiltinId::NumberPrototypeToExponential,
            ),
            ("toPrecision", StandardBuiltinId::NumberPrototypeToPrecision),
            ("toString", StandardBuiltinId::NumberPrototypeToString),
            (
                "toLocaleString",
                StandardBuiltinId::NumberPrototypeToLocaleString,
            ),
            ("valueOf", StandardBuiltinId::NumberPrototypeValueOf),
        ] {
            let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `{}`",
                    builtin.debug_name()
                ))
            })?;
            self.emit_object_define_function_data(prototype_object_local, name, meta, function)?;
        }

        Ok(())
    }

    pub(crate) fn install_boolean_constructor_intrinsics(
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

        function.instruction(&Instruction::GlobalGet(BOOLEAN_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(prototype_object_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_store_boxed_primitive_metadata(
            prototype_object_local,
            BOXED_PRIMITIVE_KIND_BOOLEAN,
            payload_local,
            tag_local,
            function,
        );
        for (name, builtin) in [
            ("toString", StandardBuiltinId::BooleanPrototypeToString),
            ("valueOf", StandardBuiltinId::BooleanPrototypeValueOf),
        ] {
            let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `{}`",
                    builtin.debug_name()
                ))
            })?;
            self.emit_object_define_function_data(prototype_object_local, name, meta, function)?;
        }

        Ok(())
    }
}
