use super::*;
use crate::objects::{WasmDescriptor, WasmPartialDescriptor};
use lila_ir::property_descriptor::Presence;

impl<'a> FunctionBuilder<'a> {
    /// The canonical numeric-index branch of TypedArray [[DefineOwnProperty]].
    /// Ordinary rejection is a Boolean result; element coercion keeps its
    /// original abrupt completion. The caller already owns the brand and
    /// CanonicalNumericIndexString checks.
    pub(in crate::builtins) fn emit_typed_array_define_index_property(
        &mut self,
        target_payload_local: u32,
        numeric_index_payload_local: u32,
        descriptor: &WasmDescriptor,
        result_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let index_local = self.reserve_temp_local();
        let valid_index_local = self.reserve_temp_local();
        let descriptor = descriptor.as_partial();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        self.emit_typed_array_valid_integer_index_i32(
            target_payload_local,
            numeric_index_payload_local,
            index_local,
            valid_index_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(valid_index_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(0));

        // Step 1.b.ii: an element of an immutable-backed TypedArray is the
        // non-writable, non-configurable data property
        // `{ [[Value]]: current, [[Enumerable]]: true }`, so the definition is
        // ValidateAndApplyPropertyDescriptor against that descriptor with no
        // object to apply to. No value coercion is performed.
        self.emit_typed_array_buffer_is_immutable_i32(target_payload_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_immutable_typed_array_element_define(
            target_payload_local,
            index_local,
            descriptor,
            result_local,
            function,
        )?;
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        for flag in [
            &descriptor.configurable,
            &descriptor.enumerable,
            &descriptor.writable,
        ] {
            match flag {
                Presence::Absent => {}
                Presence::Present(value) => {
                    function.instruction(&Instruction::LocalGet(*value));
                    function.instruction(&Instruction::I64Eqz);
                    function.instruction(&Instruction::BrIf(0));
                }
                Presence::Runtime { present, value } => {
                    function.instruction(&Instruction::LocalGet(*present));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::I64Ne);
                    function.instruction(&Instruction::LocalGet(*value));
                    function.instruction(&Instruction::I64Eqz);
                    function.instruction(&Instruction::I32And);
                    function.instruction(&Instruction::BrIf(0));
                }
            }
        }
        for accessor in [&descriptor.get, &descriptor.set] {
            match accessor {
                Presence::Absent => {}
                Presence::Present(_) => {
                    function.instruction(&Instruction::Br(0));
                }
                Presence::Runtime { present, .. } => {
                    function.instruction(&Instruction::LocalGet(*present));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::I64Ne);
                    function.instruction(&Instruction::BrIf(0));
                }
            }
        }

        if let Some(value) = descriptor.value.value() {
            if let Some(present) = descriptor.value.runtime_flag() {
                function.instruction(&Instruction::LocalGet(*present));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::If(BlockType::Empty));
            }
            // This writer converts first, then takes a fresh buffer witness.
            // Detachment or resize during a successful conversion suppresses
            // the write but does not turn this definition into rejection.
            self.emit_typed_array_element_write_from_locals(
                target_payload_local,
                index_local,
                value.payload,
                value.tag,
                function,
            )?;
            self.emit_return_current_completion_if_throw(function);
            if descriptor.value.runtime_flag().is_some() {
                function.instruction(&Instruction::End);
            }
        }
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(valid_index_local);
        self.release_temp_local(index_local);
        Ok(())
    }

    /// ValidateAndApplyPropertyDescriptor(`O`, `P`, false, `Desc`, `current`)
    /// for `current` = `{ [[Value]]: element, [[Writable]]: false,
    /// [[Enumerable]]: true, [[Configurable]]: false }`. Sets `result_local`
    /// to 1 exactly when every present field of `Desc` agrees with `current`.
    fn emit_immutable_typed_array_element_define(
        &mut self,
        target_payload_local: u32,
        index_local: u32,
        descriptor: &WasmPartialDescriptor,
        result_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let current_payload_local = self.reserve_temp_local();
        let current_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::Block(BlockType::Empty));
        // Configurable and Writable must not be true; Enumerable must not be
        // false. Each rejection leaves `result_local` at false.
        for (flag, rejected_when_true) in [
            (&descriptor.configurable, true),
            (&descriptor.enumerable, false),
            (&descriptor.writable, true),
        ] {
            let emit_rejected_value = |function: &mut Function, value: u32| {
                function.instruction(&Instruction::LocalGet(value));
                function.instruction(&Instruction::I64Eqz);
                if rejected_when_true {
                    function.instruction(&Instruction::I32Eqz);
                }
            };
            match flag {
                Presence::Absent => {}
                Presence::Present(value) => {
                    emit_rejected_value(function, *value);
                    function.instruction(&Instruction::BrIf(0));
                }
                Presence::Runtime { present, value } => {
                    function.instruction(&Instruction::LocalGet(*present));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::I64Ne);
                    emit_rejected_value(function, *value);
                    function.instruction(&Instruction::I32And);
                    function.instruction(&Instruction::BrIf(0));
                }
            }
        }
        // An accessor descriptor cannot redefine a non-configurable data
        // property.
        for accessor in [&descriptor.get, &descriptor.set] {
            match accessor {
                Presence::Absent => {}
                Presence::Present(_) => {
                    function.instruction(&Instruction::Br(0));
                }
                Presence::Runtime { present, .. } => {
                    function.instruction(&Instruction::LocalGet(*present));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::I64Ne);
                    function.instruction(&Instruction::BrIf(0));
                }
            }
        }
        // A present [[Value]] must be SameValue with the current element,
        // compared without coercion.
        if let Some(value) = descriptor.value.value() {
            if let Some(present) = descriptor.value.runtime_flag() {
                function.instruction(&Instruction::LocalGet(*present));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::If(BlockType::Empty));
            }
            self.emit_typed_array_element_read_from_locals(
                target_payload_local,
                index_local,
                current_payload_local,
                current_tag_local,
                function,
            )?;
            self.emit_tagged_payload_same_value_i32(
                value.tag,
                value.payload,
                current_tag_local,
                current_payload_local,
                function,
            )?;
            function.instruction(&Instruction::I32Eqz);
            // Inside the presence guard the enclosing Block is one label out.
            let reject_depth = u32::from(descriptor.value.runtime_flag().is_some());
            function.instruction(&Instruction::BrIf(reject_depth));
            if descriptor.value.runtime_flag().is_some() {
                function.instruction(&Instruction::End);
            }
        }
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(current_tag_local);
        self.release_temp_local(current_payload_local);
        Ok(())
    }
}
