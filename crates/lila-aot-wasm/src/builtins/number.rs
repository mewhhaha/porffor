use super::super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum NumberBuiltin {
    Constructor,
    IsInteger,
    IsSafeInteger,
    IsFinite,
    IsNaN,
    PrototypeToExponential,
    PrototypeToFixed,
    PrototypeToPrecision,
    PrototypeToString,
    PrototypeToLocaleString,
    PrototypeValueOf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NumberPrototypeOperation {
    ToExponential,
    ToFixed,
    ToPrecision,
    ToString,
    ToLocaleString,
    ValueOf,
}

impl<'a> FunctionBuilder<'a> {
    fn emit_number_constructor_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let arg_payload_local = self.reserve_temp_local();
        let arg_tag_local = self.reserve_temp_local();
        let primitive_payload_local = self.reserve_temp_local();
        let primitive_tag_local = self.reserve_temp_local();
        let has_arg_local = self.reserve_temp_local();
        self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(has_arg_local));
        function.instruction(&Instruction::LocalGet(has_arg_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(primitive_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(primitive_tag_local));
        function.instruction(&Instruction::Else);
        self.emit_value_to_number_payload_allow_bigint(arg_tag_local, arg_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(primitive_payload_local));
        // A ToPrimitive throw inside the conversion above leaves
        // completion=THROW with the original error already in
        // `self.result_local`/`self.result_tag_local` (untouched,
        // since the number-conversion helper skips further
        // processing on throw). Propagate to the active
        // try/catch handler here (this arm is nested one
        // untracked `If` deep, the `has_arg_local` check above)
        // instead of falling through and stamping a bogus
        // Number tag over the thrown error.
        self.emit_propagate_throw_from_locals_if_needed(
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(primitive_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(primitive_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(primitive_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.release_temp_local(has_arg_local);
        self.release_temp_local(primitive_tag_local);
        self.release_temp_local(primitive_payload_local);
        self.release_temp_local(arg_tag_local);
        self.release_temp_local(arg_payload_local);
        Ok(())
    }

    fn emit_number_prototype_builtin(
        &mut self,
        operation: NumberPrototypeOperation,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Number prototype receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Number prototype receiver",
            )
        })?;
        let boxed_kind_local = self.reserve_temp_local();
        let number_payload_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(receiver_payload_local));
        function.instruction(&Instruction::LocalSet(number_payload_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            boxed_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_NUMBER as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            number_payload_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_throw_current_function_realm_type_error(
            "Number.prototype method requires a Number receiver",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_throw_current_function_realm_type_error(
            "Number.prototype method requires a Number receiver",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        match operation {
            NumberPrototypeOperation::ValueOf => {
                function.instruction(&Instruction::LocalGet(number_payload_local));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
            }
            NumberPrototypeOperation::ToFixed => {
                self.emit_number_to_fixed_payload(number_payload_local, function)?;
            }
            NumberPrototypeOperation::ToExponential => {
                self.emit_number_to_exponential_payload(number_payload_local, function)?;
            }
            NumberPrototypeOperation::ToPrecision => {
                self.emit_number_to_precision_payload(number_payload_local, function)?;
            }
            NumberPrototypeOperation::ToString => {
                self.emit_number_to_string_with_radix_result(number_payload_local, function)?;
            }
            NumberPrototypeOperation::ToLocaleString => {
                self.emit_number_to_string_payload(number_payload_local, function)?;
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
            }
        }

        self.release_temp_local(number_payload_local);
        self.release_temp_local(boxed_kind_local);
        Ok(())
    }

    pub(super) fn emit_number_builtin(
        &mut self,
        builtin: NumberBuiltin,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match builtin {
            NumberBuiltin::Constructor => self.emit_number_constructor_builtin(function)?,
            NumberBuiltin::IsInteger => {
                let arg_payload_local = self.reserve_temp_local();
                let arg_tag_local = self.reserve_temp_local();
                self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::LocalGet(arg_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(arg_payload_local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Trunc);
                function.instruction(&Instruction::LocalGet(arg_payload_local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Eq);
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::LocalSet(self.result_local));
                for infinite in [f64::INFINITY, f64::NEG_INFINITY] {
                    function.instruction(&Instruction::LocalGet(arg_payload_local));
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::F64Const(Ieee64::from(infinite)));
                    function.instruction(&Instruction::F64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(self.result_local));
                    function.instruction(&Instruction::End);
                }
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                self.release_temp_local(arg_tag_local);
                self.release_temp_local(arg_payload_local);
            }
            NumberBuiltin::IsSafeInteger => {
                let arg_payload_local = self.reserve_temp_local();
                let arg_tag_local = self.reserve_temp_local();
                self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::LocalGet(arg_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(arg_payload_local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Trunc);
                function.instruction(&Instruction::LocalGet(arg_payload_local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Eq);
                function.instruction(&Instruction::LocalGet(arg_payload_local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Abs);
                function.instruction(&Instruction::F64Const(Ieee64::from(
                    9_007_199_254_740_991.0,
                )));
                function.instruction(&Instruction::F64Le);
                function.instruction(&Instruction::I32And);
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                self.release_temp_local(arg_tag_local);
                self.release_temp_local(arg_payload_local);
            }
            NumberBuiltin::IsNaN => {
                let arg_payload_local = self.reserve_temp_local();
                let arg_tag_local = self.reserve_temp_local();
                self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::LocalGet(arg_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(arg_payload_local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::LocalGet(arg_payload_local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Ne);
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                self.release_temp_local(arg_tag_local);
                self.release_temp_local(arg_payload_local);
            }
            NumberBuiltin::IsFinite => {
                let arg_payload_local = self.reserve_temp_local();
                let arg_tag_local = self.reserve_temp_local();
                self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::LocalGet(arg_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(arg_payload_local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::LocalGet(arg_payload_local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Eq);
                for infinite in [f64::INFINITY, f64::NEG_INFINITY] {
                    function.instruction(&Instruction::LocalGet(arg_payload_local));
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::F64Const(Ieee64::from(infinite)));
                    function.instruction(&Instruction::F64Ne);
                    function.instruction(&Instruction::I32And);
                }
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                self.release_temp_local(arg_tag_local);
                self.release_temp_local(arg_payload_local);
            }
            NumberBuiltin::PrototypeToExponential => self
                .emit_number_prototype_builtin(NumberPrototypeOperation::ToExponential, function)?,
            NumberBuiltin::PrototypeToFixed => {
                self.emit_number_prototype_builtin(NumberPrototypeOperation::ToFixed, function)?
            }
            NumberBuiltin::PrototypeToPrecision => {
                self.emit_number_prototype_builtin(NumberPrototypeOperation::ToPrecision, function)?
            }
            NumberBuiltin::PrototypeToString => {
                self.emit_number_prototype_builtin(NumberPrototypeOperation::ToString, function)?
            }
            NumberBuiltin::PrototypeToLocaleString => self.emit_number_prototype_builtin(
                NumberPrototypeOperation::ToLocaleString,
                function,
            )?,
            NumberBuiltin::PrototypeValueOf => {
                self.emit_number_prototype_builtin(NumberPrototypeOperation::ValueOf, function)?
            }
        }
        Ok(())
    }
}
