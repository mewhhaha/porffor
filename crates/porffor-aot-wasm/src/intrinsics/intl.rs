//! `Intl` intrinsic installation.
//!
//! Property installation order is observable through `Object.keys`, so the
//! statement order inside the installer is load-bearing — do not reorder.

use super::super::*;
use super::IntrinsicInstall;

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn install_intl_locale_constructor_intrinsics(
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

        function.instruction(&Instruction::GlobalGet(prototype_global_index));
        function.instruction(&Instruction::LocalSet(prototype_object_local));
        for (name, getter) in [
            (
                "language",
                StandardBuiltinId::IntlLocalePrototypeLanguageGetter,
            ),
            ("script", StandardBuiltinId::IntlLocalePrototypeScriptGetter),
            ("region", StandardBuiltinId::IntlLocalePrototypeRegionGetter),
            (
                "baseName",
                StandardBuiltinId::IntlLocalePrototypeBaseNameGetter,
            ),
        ] {
            let getter_meta = self.functions.get(&getter.function_id()).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                    getter.debug_name()
                ))
            })?;
            function.instruction(&Instruction::I64Const(self.strings.payload(name)));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_function_value_payload(getter_meta, function)?;
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
            .get(&StandardBuiltinId::IntlLocalePrototypeToString.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Intl.Locale.prototype.toString`",
                )
            })?;
        self.emit_object_define_function_data(
            prototype_object_local,
            "toString",
            to_string_meta,
            function,
        )?;
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("Intl.Locale")));
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
