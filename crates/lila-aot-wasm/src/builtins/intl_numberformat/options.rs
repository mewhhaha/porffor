use super::*;

impl FunctionBuilder<'_> {
    pub(super) fn emit_nf_get_option(
        &mut self,
        options: TaggedLocals,
        property: &str,
        value: TaggedLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key = self.reserve_temp_local();
        self.emit_nf_set_string(key, property, function);
        self.emit_object_read(
            options.payload,
            options.tag,
            options.payload,
            options.tag,
            key,
            value.payload,
            value.tag,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.release_temp_local(key);
        Ok(())
    }
    fn emit_nf_to_string(
        &mut self,
        value: TaggedLocals,
        destination: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let primitive = self.emit_tagged_to_primitive_locals_in_current_function_realm(
            ToPrimitiveHint::String,
            value.payload,
            value.tag,
            function,
        )?;
        self.emit_current_function_realm_primitive_to_string_local(primitive, destination, function)
    }
    pub(super) fn emit_nf_string_value(
        &mut self,
        options: TaggedLocals,
        property: &str,
        value: TaggedLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_nf_get_option(options, property, value, function)?;
        function.instruction(&Instruction::LocalGet(value.tag));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_nf_to_string(value, value.payload, function)?;
        self.emit_nf_set_const(value.tag, ValueKind::String.tag() as i64, function);
        function.instruction(&Instruction::End);
        Ok(())
    }
    pub(super) fn emit_nf_select_spelling(
        &mut self,
        text: u32,
        codes: &[(&str, i64)],
        destination: u32,
        function: &mut Function,
    ) {
        let expected = self.reserve_temp_local();
        self.emit_nf_set_const(destination, 0, function);
        for (spelling, code) in codes {
            self.emit_nf_set_string(expected, spelling, function);
            self.emit_string_payload_equality_i32(text, expected, function);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_nf_set_const(destination, *code, function);
            function.instruction(&Instruction::End);
        }
        self.release_temp_local(expected);
    }
    pub(super) fn emit_nf_choice_option(
        &mut self,
        options: TaggedLocals,
        property: &str,
        codes: &[(&str, i64)],
        default: u64,
        destination: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let value = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        self.emit_nf_string_value(options, property, value, function)?;
        self.emit_nf_set_const(destination, default as i64, function);
        function.instruction(&Instruction::LocalGet(value.tag));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_nf_select_spelling(value.payload, codes, destination, function);
        function.instruction(&Instruction::LocalGet(destination));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_nf_range_error(&format!("Invalid {property} option"), function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(value.tag);
        self.release_temp_local(value.payload);
        Ok(())
    }
    pub(super) fn emit_nf_coerce_digit(
        &mut self,
        value: TaggedLocals,
        property: &str,
        minimum: u32,
        maximum: u32,
        fallback: u32,
        destination: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_nf_if_eq(value.tag, ValueKind::Undefined.tag() as u64, function);
        self.emit_nf_copy(fallback, destination, function);
        function.instruction(&Instruction::Else);
        let number = self.reserve_temp_local();
        self.emit_value_to_number_payload(value.tag, value.payload, function)?;
        function.instruction(&Instruction::LocalSet(number));
        self.emit_return_current_completion_if_throw(function);
        for instruction in [
            Instruction::LocalGet(number),
            Instruction::F64ReinterpretI64,
            Instruction::LocalGet(number),
            Instruction::F64ReinterpretI64,
            Instruction::F64Ne,
            Instruction::LocalGet(number),
            Instruction::F64ReinterpretI64,
            Instruction::F64Const(Ieee64::from(minimum as f64)),
            Instruction::F64Lt,
            Instruction::I32Or,
            Instruction::LocalGet(number),
            Instruction::F64ReinterpretI64,
            Instruction::F64Const(Ieee64::from(maximum as f64)),
            Instruction::F64Gt,
            Instruction::I32Or,
        ] {
            function.instruction(&instruction);
        }
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_nf_range_error(&format!("Invalid {property} option"), function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(number));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Floor);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(destination));
        self.release_temp_local(number);
        function.instruction(&Instruction::End);
        Ok(())
    }
    pub(super) fn emit_nf_number_option(
        &mut self,
        options: TaggedLocals,
        property: &str,
        minimum: u32,
        maximum: u32,
        fallback: u32,
        destination: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let value = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        self.emit_nf_get_option(options, property, value, function)?;
        self.emit_nf_coerce_digit(
            value,
            property,
            minimum,
            maximum,
            fallback,
            destination,
            function,
        )?;
        self.release_temp_local(value.tag);
        self.release_temp_local(value.payload);
        Ok(())
    }
    pub(super) fn emit_nf_options_object(
        &mut self,
        options: TaggedLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_nf_if_eq(options.tag, ValueKind::Undefined.tag() as u64, function);
        self.emit_alloc_plain_object_with_prototype(None, None, function)?;
        function.instruction(&Instruction::LocalSet(options.payload));
        self.emit_nf_set_const(options.tag, ValueKind::Object.tag() as i64, function);
        function.instruction(&Instruction::Else);
        self.emit_value_to_current_function_realm_object_locals(
            options.payload,
            options.tag,
            options.payload,
            options.tag,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);
        Ok(())
    }
    pub(super) fn emit_nf_grouping_option(
        &mut self,
        options: TaggedLocals,
        notation: u32,
        destination: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let value = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let default = self.reserve_temp_local();
        let equal = self.reserve_temp_local();
        let expected = self.reserve_temp_local();
        self.emit_nf_set_const(default, Grouping::Auto.wire_code() as i64, function);
        self.emit_nf_if_eq(notation, NotationOption::Compact.wire_code(), function);
        self.emit_nf_set_const(default, Grouping::MinTwo.wire_code() as i64, function);
        function.instruction(&Instruction::End);
        self.emit_nf_copy(default, destination, function);
        self.emit_nf_get_option(options, "useGrouping", value, function)?;
        function.instruction(&Instruction::LocalGet(value.tag));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_to_boolean_payload_from_tagged_locals(value.tag, value.payload, function)?;
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_nf_set_const(destination, Grouping::Never.wire_code() as i64, function);
        function.instruction(&Instruction::Else);
        self.emit_nf_if_eq(value.tag, ValueKind::Boolean.tag() as u64, function);
        self.emit_nf_set_const(destination, Grouping::Always.wire_code() as i64, function);
        function.instruction(&Instruction::Else);
        self.emit_nf_to_string(value, value.payload, function)?;
        self.emit_nf_set_const(equal, 0, function);
        for spelling in ["true", "false"] {
            self.emit_nf_set_string(expected, spelling, function);
            self.emit_string_payload_equality_i32(value.payload, expected, function);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::LocalGet(equal));
            function.instruction(&Instruction::I64Or);
            function.instruction(&Instruction::LocalSet(equal));
        }
        self.emit_nf_if_nonzero(equal, function);
        self.emit_nf_copy(default, destination, function);
        function.instruction(&Instruction::Else);
        self.emit_nf_select_spelling(
            value.payload,
            &[("auto", 1), ("always", 2), ("min2", 3)],
            destination,
            function,
        );
        function.instruction(&Instruction::LocalGet(destination));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_nf_range_error("Invalid useGrouping option", function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for local in [expected, equal, default, value.tag, value.payload] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
