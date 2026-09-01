//! `promise` intrinsic installation.
//!
//! Extracted verbatim from `builtins/bootstrap.rs::init_builtin_constructor_object`.
//! Property installation order is observable through `Object.keys`, so the
//! statement order inside each installer is load-bearing — do not reorder.

use super::super::*;
use super::IntrinsicInstall;

pub(crate) const PROMISE_PROTOTYPE_METHOD_PUBLICATIONS: [StandardBuiltinId; 3] = [
    StandardBuiltinId::PromisePrototypeThen,
    StandardBuiltinId::PromisePrototypeCatch,
    StandardBuiltinId::PromisePrototypeFinally,
];

pub(crate) const PROMISE_STATIC_METHOD_PUBLICATIONS: [StandardBuiltinId; 10] = [
    StandardBuiltinId::PromiseResolve,
    StandardBuiltinId::PromiseReject,
    StandardBuiltinId::PromiseAll,
    StandardBuiltinId::PromiseAllSettled,
    StandardBuiltinId::PromiseAllKeyed,
    StandardBuiltinId::PromiseAllSettledKeyed,
    StandardBuiltinId::PromiseAny,
    StandardBuiltinId::PromiseRace,
    StandardBuiltinId::PromiseWithResolvers,
    StandardBuiltinId::PromiseTry,
];

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn install_promise_constructor_intrinsics(
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

        let promise_object_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(PROMISE_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(promise_object_local));
        for builtin in PROMISE_PROTOTYPE_METHOD_PUBLICATIONS {
            let name = builtin.native_function_name().ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in lila wasm-aot first slice: missing native name for `{}`",
                    builtin.debug_name()
                ))
            })?;
            let meta = self
                .functions
                .get(&builtin.function_id())
                .cloned()
                .ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in lila wasm-aot first slice: missing builtin meta `{}`",
                        builtin.debug_name()
                    ))
                })?;
            self.emit_object_define_function_data(promise_object_local, name, &meta, function)?;
        }
        let to_string_tag_key_local = self.reserve_temp_local();
        let to_string_tag_payload_local = self.reserve_temp_local();
        let to_string_tag_tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(to_string_tag_key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("Promise")));
        function.instruction(&Instruction::LocalSet(to_string_tag_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(to_string_tag_tag_local));
        self.emit_object_append_data_property_with_flags(
            promise_object_local,
            to_string_tag_key_local,
            to_string_tag_payload_local,
            to_string_tag_tag_local,
            false,
            false,
            true,
            function,
        )?;
        self.release_temp_local(to_string_tag_tag_local);
        self.release_temp_local(to_string_tag_payload_local);
        self.release_temp_local(to_string_tag_key_local);
        function.instruction(&Instruction::GlobalGet(PROMISE_CONSTRUCTOR_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(promise_object_local));
        for builtin in PROMISE_STATIC_METHOD_PUBLICATIONS {
            let name = builtin.native_function_name().ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in lila wasm-aot first slice: missing native name for `{}`",
                    builtin.debug_name()
                ))
            })?;
            let meta = self
                .functions
                .get(&builtin.function_id())
                .cloned()
                .ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in lila wasm-aot first slice: missing builtin meta `{}`",
                        builtin.debug_name()
                    ))
                })?;
            self.emit_object_define_function_data(promise_object_local, name, &meta, function)?;
        }
        let species_key_local = self.reserve_temp_local();
        let species_getter_payload_local = self.reserve_temp_local();
        let species_getter_tag_local = self.reserve_temp_local();
        let species_meta = self
            .functions
            .get(&StandardBuiltinId::PromiseSpeciesGetter.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Promise[Symbol.species]`",
                )
            })?;
        function.instruction(&Instruction::I64Const(
            self.strings.property_key_symbol_payload("Symbol.species"),
        ));
        function.instruction(&Instruction::LocalSet(species_key_local));
        self.emit_function_value_payload(species_meta, function)?;
        function.instruction(&Instruction::LocalSet(species_getter_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(species_getter_tag_local));
        self.emit_object_append_accessor_property_with_flags(
            promise_object_local,
            species_key_local,
            Some((species_getter_payload_local, species_getter_tag_local)),
            None,
            false,
            true,
            function,
        )?;
        self.release_temp_local(species_getter_tag_local);
        self.release_temp_local(species_getter_payload_local);
        self.release_temp_local(species_key_local);
        self.release_temp_local(promise_object_local);

        Ok(())
    }
}
