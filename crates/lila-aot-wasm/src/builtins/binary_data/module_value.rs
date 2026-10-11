//! Raw Bytes Module evaluation allocates in its defining module Realm.
use super::*;

impl FunctionBuilder<'_> {
    pub(crate) fn emit_bytes_module_value(
        &mut self,
        plan: &lila_ir::BytesModuleValueIr,
        output: &ValueLocals,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let realm = self.data_module_realm(plan.module(), f)?;
        let s = self.runtime_schema();
        let pending = s.reserve_completion(f);
        pending.initialize(f);
        let prototype = s.reserve_value_local(f);
        let buffer_value = s.reserve_value_local(f);
        let length = s.reserve_i64_local(f);
        let offset = s.reserve_i64_local(f);
        let fixed = s.reserve_i32_local(f);
        let byte = s.reserve_i32_local(f);
        let data = self.strings.module_bytes(plan.module())?;
        f.instruction(&Instruction::I64Const(i64::from(data.length())));
        length.store(f);
        offset.set_constant(0, f);
        fixed.set_constant(0, f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        self.emit_load_non_array_realm_intrinsic(
            &realm,
            NonArrayRealmIntrinsicSlot::ArrayBufferPrototype,
            &prototype,
            f,
        );
        let buffer = self.emit_binary_allocate_array_buffer(
            &prototype, length, length, fixed, true, &pending, exit, f,
        )?;
        buffer_value.set_reference(&buffer, s, f);
        let owner = self.emit_binary_buffer_owner(&buffer_value, &pending, exit, f)?;
        let access = self.emit_binary_buffer_access(&owner, f);
        // Only this unpublished allocation is initialized through internal
        // backing access. Public writes observe its immutable ArrayBuffer flag.
        let done = self.open_frame(ControlFrameKind::Block, f);
        let next = self.open_frame(ControlFrameKind::Loop, f);
        offset.load(f);
        length.load(f);
        f.instruction(&Instruction::I64GeU);
        self.emit_branch_if_to_target(done, f);
        data.emit_byte(offset, f);
        byte.store(f);
        access.write_byte(offset, byte, self, f)?;
        self.emit_increment_local(offset, 1, f);
        self.emit_branch_to_target(next, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        access.clear(s, f);
        offset.set_constant(0, f);
        self.emit_load_non_array_realm_intrinsic(
            &realm,
            NonArrayRealmIntrinsicSlot::Uint8ArrayPrototype,
            &prototype,
            f,
        );
        let array = self.emit_binary_allocate_typed_array(
            TypedArrayElementKind::Uint8,
            &prototype,
            &owner,
            offset,
            length,
            fixed,
            f,
        )?;
        buffer_value.set_reference(&array, s, f);
        pending.set_normal(&buffer_value, f);
        array.clear(f);
        owner.clear(f);
        buffer.clear(f);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        self.completion().copy_from(&pending, f);
        output.copy_from(pending.value(), f);
        pending.clear(f);
        buffer_value.clear(f);
        prototype.clear(f);
        s.release_i32_local(byte, f);
        s.release_i32_local(fixed, f);
        s.release_i64_local(offset, f);
        s.release_i64_local(length, f);
        realm.clear(f);
        self.emit_propagate_current_throw_if_needed(f);
        Ok(())
    }
}
