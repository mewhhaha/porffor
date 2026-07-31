//! `function` intrinsic installation.
//!
//! Extracted verbatim from `builtins/bootstrap.rs::init_builtin_constructor_object`.
//! Property installation order is observable through `Object.keys`, so the
//! statement order inside each installer is load-bearing — do not reorder.

use super::super::*;
use super::IntrinsicInstall;

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn install_function_constructor_intrinsics(
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
        let call_meta = self
            .functions
            .get(&StandardBuiltinId::FunctionPrototypeCall.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Function.prototype.call`",
                )
            })?;
        function.instruction(&Instruction::GlobalGet(prototype_global_index));
        function.instruction(&Instruction::LocalSet(prototype_object_local));
        self.emit_object_define_function_data(prototype_object_local, "call", call_meta, function)?;
        let apply_meta = self
            .functions
            .get(&StandardBuiltinId::FunctionPrototypeApply.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Function.prototype.apply`",
                )
            })?;
        function.instruction(&Instruction::GlobalGet(prototype_global_index));
        function.instruction(&Instruction::LocalSet(prototype_object_local));
        self.emit_object_define_function_data(
            prototype_object_local,
            "apply",
            apply_meta,
            function,
        )?;
        let bind_meta = self
            .functions
            .get(&StandardBuiltinId::FunctionPrototypeBind.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Function.prototype.bind`",
                )
            })?;
        function.instruction(&Instruction::GlobalGet(prototype_global_index));
        function.instruction(&Instruction::LocalSet(prototype_object_local));
        self.emit_object_define_function_data(prototype_object_local, "bind", bind_meta, function)?;
        let to_string_meta = self
            .functions
            .get(&StandardBuiltinId::FunctionPrototypeToString.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Function.prototype.toString`",
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
        self.release_temp_local(prototype_object_local);

        Ok(())
    }
}
