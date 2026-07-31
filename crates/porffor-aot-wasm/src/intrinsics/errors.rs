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
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Error.prototype.toString`",
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
        let is_error_meta = self
            .functions
            .get(&StandardBuiltinId::ErrorIsError.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Error.isError`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "isError", is_error_meta, function)?;
        self.release_temp_local(prototype_object_local);

        Ok(())
    }
}
