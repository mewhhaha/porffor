//! `regexp` intrinsic installation.
//!
//! Extracted verbatim from `builtins/bootstrap.rs::init_builtin_constructor_object`.
//! Property installation order is observable through `Object.keys`, so the
//! statement order inside each installer is load-bearing — do not reorder.

use super::super::*;
use super::IntrinsicInstall;

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn install_regexp_constructor_intrinsics(
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

        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        let exec_meta = self
            .functions
            .get(&StandardBuiltinId::RegExpPrototypeExec.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `RegExp.prototype.exec`",
                )
            })?;
        let test_meta = self
            .functions
            .get(&StandardBuiltinId::RegExpPrototypeTest.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `RegExp.prototype.test`",
                )
            })?;
        let compile_meta = self
            .functions
            .get(&StandardBuiltinId::RegExpPrototypeCompile.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `RegExp.prototype.compile`",
                )
            })?;
        let to_string_meta = self
            .functions
            .get(&StandardBuiltinId::RegExpPrototypeToString.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `RegExp.prototype.toString`",
                )
            })?;
        let match_all_meta = self
            .functions
            .get(&StandardBuiltinId::RegExpPrototypeSymbolMatchAll.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `RegExp.prototype[Symbol.matchAll]`",
                )
            })?;
        let split_meta = self
            .functions
            .get(&StandardBuiltinId::RegExpPrototypeSymbolSplit.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `RegExp.prototype[Symbol.split]`",
                )
            })?;
        let replace_meta = self
            .functions
            .get(&StandardBuiltinId::RegExpPrototypeSymbolReplace.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `RegExp.prototype[Symbol.replace]`",
                )
            })?;
        function.instruction(&Instruction::GlobalGet(REGEXP_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(object_local));
        self.emit_object_define_function_data(object_local, "compile", &compile_meta, function)?;
        self.emit_object_define_function_data(object_local, "toString", &to_string_meta, function)?;
        function.instruction(&Instruction::I64Const(self.strings.payload("exec")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_function_value_payload(&exec_meta, function)?;
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::GlobalSet(
            REGEXP_PROTOTYPE_EXEC_FUNCTION_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            object_local,
            key_local,
            payload_local,
            tag_local,
            true,
            false,
            true,
            function,
        )?;
        self.emit_object_define_function_data(object_local, "test", &test_meta, function)?;
        for (name, getter) in [
            ("source", StandardBuiltinId::RegExpPrototypeSourceGetter),
            (
                "hasIndices",
                StandardBuiltinId::RegExpPrototypeHasIndicesGetter,
            ),
            ("global", StandardBuiltinId::RegExpPrototypeGlobalGetter),
            (
                "ignoreCase",
                StandardBuiltinId::RegExpPrototypeIgnoreCaseGetter,
            ),
            (
                "multiline",
                StandardBuiltinId::RegExpPrototypeMultilineGetter,
            ),
            ("dotAll", StandardBuiltinId::RegExpPrototypeDotAllGetter),
            ("unicode", StandardBuiltinId::RegExpPrototypeUnicodeGetter),
            (
                "unicodeSets",
                StandardBuiltinId::RegExpPrototypeUnicodeSetsGetter,
            ),
            ("sticky", StandardBuiltinId::RegExpPrototypeStickyGetter),
            ("flags", StandardBuiltinId::RegExpPrototypeFlagsGetter),
        ] {
            let getter_meta = self
                .functions
                .get(&getter.function_id())
                .cloned()
                .ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in lila wasm-aot first slice: missing builtin meta `{}`",
                        getter.debug_name()
                    ))
                })?;
            function.instruction(&Instruction::I64Const(self.strings.payload(name)));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_function_value_payload(&getter_meta, function)?;
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            self.emit_object_append_accessor_property_with_flags(
                object_local,
                key_local,
                Some((payload_local, tag_local)),
                None,
                false,
                true,
                function,
            )?;
        }
        function.instruction(&Instruction::I64Const(
            self.strings.property_key_symbol_payload("Symbol.match"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::GlobalGet(
            REGEXP_PROTOTYPE_SYMBOL_MATCH_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            object_local,
            key_local,
            payload_local,
            tag_local,
            true,
            false,
            true,
            function,
        )?;
        function.instruction(&Instruction::I64Const(
            self.strings.property_key_symbol_payload("Symbol.matchAll"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_function_value_payload(&match_all_meta, function)?;
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            object_local,
            key_local,
            payload_local,
            tag_local,
            true,
            false,
            true,
            function,
        )?;
        function.instruction(&Instruction::I64Const(
            self.strings.property_key_symbol_payload("Symbol.replace"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_function_value_payload(&replace_meta, function)?;
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            object_local,
            key_local,
            payload_local,
            tag_local,
            true,
            false,
            true,
            function,
        )?;
        function.instruction(&Instruction::I64Const(
            self.strings.property_key_symbol_payload("Symbol.search"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::GlobalGet(
            REGEXP_PROTOTYPE_SYMBOL_SEARCH_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            object_local,
            key_local,
            payload_local,
            tag_local,
            true,
            false,
            true,
            function,
        )?;
        function.instruction(&Instruction::I64Const(
            self.strings.property_key_symbol_payload("Symbol.split"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_function_value_payload(&split_meta, function)?;
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            object_local,
            key_local,
            payload_local,
            tag_local,
            true,
            false,
            true,
            function,
        )?;
        function.instruction(&Instruction::GlobalGet(REGEXP_CONSTRUCTOR_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(object_local));
        let escape_meta = self
            .functions
            .get(&StandardBuiltinId::RegExpEscape.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `RegExp.escape`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "escape", &escape_meta, function)?;
        let species_meta = self
            .functions
            .get(&StandardBuiltinId::RegExpSpeciesGetter.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `RegExp[Symbol.species]`",
                )
            })?;
        function.instruction(&Instruction::I64Const(
            self.strings.property_key_symbol_payload("Symbol.species"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_function_value_payload(species_meta, function)?;
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_accessor_property_with_flags(
            object_local,
            key_local,
            Some((payload_local, tag_local)),
            None,
            false,
            true,
            function,
        )?;
        let getter_meta = self
            .functions
            .get(&StandardBuiltinId::RegExpLegacyStaticGetter.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `RegExp legacy static getter`",
                )
            })?;
        self.emit_function_value_payload(getter_meta, function)?;
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        let getter = (payload_local, tag_local);
        let setter_payload_local = self.reserve_temp_local();
        let setter_tag_local = self.reserve_temp_local();
        let setter_meta = self
            .functions
            .get(&StandardBuiltinId::RegExpLegacyStaticSetter.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `RegExp legacy static setter`",
                )
            })?;
        self.emit_function_value_payload(setter_meta, function)?;
        function.instruction(&Instruction::LocalSet(setter_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(setter_tag_local));
        let setter = (setter_payload_local, setter_tag_local);
        for name in ["input", "$_"] {
            function.instruction(&Instruction::I64Const(self.strings.payload(name)));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_object_append_accessor_property_with_flags(
                object_local,
                key_local,
                Some(getter),
                Some(setter),
                false,
                true,
                function,
            )?;
        }
        for name in [
            "lastMatch",
            "$&",
            "lastParen",
            "$+",
            "leftContext",
            "$`",
            "rightContext",
            "$'",
            "$1",
            "$2",
            "$3",
            "$4",
            "$5",
            "$6",
            "$7",
            "$8",
            "$9",
        ] {
            let payload = self.strings.payload(name);
            function.instruction(&Instruction::I64Const(payload));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_object_append_accessor_property_with_flags(
                object_local,
                key_local,
                Some(getter),
                None,
                false,
                true,
                function,
            )?;
        }
        self.release_temp_local(setter_tag_local);
        self.release_temp_local(setter_payload_local);
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);

        Ok(())
    }
}
