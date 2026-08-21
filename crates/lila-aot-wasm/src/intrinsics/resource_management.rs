//! Explicit-resource-management intrinsics.
//!
//! `%DisposableStack.prototype%` is installed as one unit so its method
//! identities, accessor shape and symbol alias cannot drift independently.

use super::super::*;
use super::IntrinsicInstall;

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn install_disposable_stack_constructor_intrinsics(
        &mut self,
        _context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let prototype_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::GlobalGet(
            DISPOSABLE_STACK_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_local));
        for (name, builtin) in [
            ("use", StandardBuiltinId::DisposableStackPrototypeUse),
            ("adopt", StandardBuiltinId::DisposableStackPrototypeAdopt),
            ("defer", StandardBuiltinId::DisposableStackPrototypeDefer),
            ("move", StandardBuiltinId::DisposableStackPrototypeMove),
        ] {
            let meta = self
                .functions
                .get(&builtin.function_id())
                .cloned()
                .ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in lila wasm-aot: missing builtin meta `{}`",
                        builtin.debug_name()
                    ))
                })?;
            self.emit_object_define_function_data(prototype_local, name, &meta, function)?;
        }

        // One function-value allocation backs both properties. Test262 checks
        // identity, so installing two equivalent builtins would still be wrong.
        let dispose_meta = self
            .functions
            .get(&StandardBuiltinId::DisposableStackPrototypeDispose.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot: missing builtin meta `DisposableStack.prototype.dispose`",
                )
            })?;
        self.emit_object_define_function_data_with_aliases(
            prototype_local,
            "dispose",
            &["Symbol.dispose"],
            &dispose_meta,
            function,
        )?;

        let disposed_getter = self
            .functions
            .get(&StandardBuiltinId::DisposableStackPrototypeDisposedGetter.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot: missing builtin meta `get DisposableStack.prototype.disposed`",
                )
            })?;
        function.instruction(&Instruction::I64Const(self.strings.payload("disposed")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_function_value_payload(&disposed_getter, function)?;
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_accessor_property_with_flags(
            prototype_local,
            key_local,
            Some((payload_local, tag_local)),
            None,
            false,
            true,
            function,
        )?;

        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("DisposableStack"),
        ));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            prototype_local,
            key_local,
            payload_local,
            tag_local,
            false,
            false,
            true,
            function,
        )?;

        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(prototype_local);
        Ok(())
    }
}
