//! `errors` intrinsic installation.
//!
//! Extracted verbatim from `builtins/bootstrap.rs::init_builtin_constructor_object`.
//! Property installation order is observable through `Object.keys`, so the
//! statement order inside each installer is load-bearing — do not reorder.

use super::super::*;
use super::IntrinsicInstall;

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn install_error_constructor_intrinsics(
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

        let prototype_object_local = self.reserve_temp_local();
        let to_string_meta = self
            .functions
            .get(&StandardBuiltinId::ErrorPrototypeToString.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Error.prototype.toString`",
                )
            })?;
        function.instruction(&Instruction::GlobalGet(prototype_global_index));
        function.instruction(&Instruction::LocalSet(prototype_object_local));
        self.emit_object_define_function_data(
            prototype_object_local,
            "toString",
            to_string_meta,
            function,
        )?;
        self.install_error_prototype_stack_accessor(
            prototype_object_local,
            key_local,
            payload_local,
            tag_local,
            function,
        )?;
        let is_error_meta = self
            .functions
            .get(&StandardBuiltinId::ErrorIsError.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Error.isError`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "isError", is_error_meta, function)?;
        self.release_temp_local(prototype_object_local);

        Ok(())
    }

    /// `Error.prototype.stack` (proposal-error-stack-accessor): an accessor
    /// with both functions, `{ [[Enumerable]]: false, [[Configurable]]: true }`.
    /// Both functions keep the entry realm's zero environment; the setter
    /// derives its `%Error.prototype%` home object from it.
    fn install_error_prototype_stack_accessor(
        &mut self,
        prototype_object_local: u32,
        key_local: u32,
        getter_payload_local: u32,
        getter_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let [getter_meta, setter_meta] = [
            StandardBuiltinId::ErrorPrototypeStackGetter,
            StandardBuiltinId::ErrorPrototypeStackSetter,
        ]
        .map(|builtin| {
            self.functions
                .get(&builtin.function_id())
                .cloned()
                .ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in lila wasm-aot first slice: missing builtin meta `{}`",
                        builtin.debug_name()
                    ))
                })
        });
        let (getter_meta, setter_meta) = (getter_meta?, setter_meta?);
        let setter_payload_local = self.reserve_temp_local();
        let setter_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(self.strings.payload("stack")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_function_value_payload(&getter_meta, function)?;
        function.instruction(&Instruction::LocalSet(getter_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(getter_tag_local));
        self.emit_function_value_payload(&setter_meta, function)?;
        function.instruction(&Instruction::LocalSet(setter_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(setter_tag_local));
        self.emit_object_append_accessor_property_with_flags(
            prototype_object_local,
            key_local,
            Some((getter_payload_local, getter_tag_local)),
            Some((setter_payload_local, setter_tag_local)),
            false,
            true,
            function,
        )?;

        self.release_temp_local(setter_tag_local);
        self.release_temp_local(setter_payload_local);
        Ok(())
    }
}
