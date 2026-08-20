//! Explicit-resource-management intrinsic shells.
//!
//! The synchronous stack installer owns only `@@toStringTag`; common
//! bootstrap installs the constructor link. Real disposal members remain
//! absent until their algorithms exist.

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
