//! `get Error.prototype.stack` (proposal-error-stack-accessor). The setter is
//! `SetterThatIgnoresPrototypeProperties` and lives with the other setters of
//! that shape in `builtins/object/setter_ignoring_prototype_properties.rs`.

use super::*;

/// The implementation-defined String that represents the stack trace of an
/// object with [[ErrorData]]. Lila records no stack frames when an error is
/// created, so every trace is the empty trace, represented by the empty
/// String. Returning a constant keeps the getter free of observable reads
/// (it must not reach `name`/`message` getters or Proxy traps).
const ERROR_STACK_TRACE_STRING: &str = "";

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_error_prototype_stack_getter(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self
            .this_payload_local
            .ok_or_else(|| EmitError::unsupported("missing get Error.prototype.stack receiver"))?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported("missing get Error.prototype.stack receiver tag")
        })?;
        let brand_local = self.reserve_temp_local();

        // 1. Let E be the this value.
        // 2. If E is not an Object, throw a TypeError exception.
        self.emit_is_heap_object_like_tag_i32(receiver_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Error.prototype.stack getter called on incompatible receiver",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        // 3. If E does not have an [[ErrorData]] internal slot, return
        //    undefined. The slot is the object's own brand, never inherited
        //    and never forwarded by a Proxy.
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(brand_local));
        function.instruction(&Instruction::I64Const(OBJECT_INTERNAL_BRAND_ERROR as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        // 4. Return an implementation-defined string that represents the
        //    stack trace of E.
        function.instruction(&Instruction::I64Const(
            self.strings.payload(ERROR_STACK_TRACE_STRING),
        ));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(brand_local);
        Ok(())
    }
}
