use super::*;
use crate::functions::OrdinaryDefaultPrototype;

enum StringCharacterConstructor {
    FromCharCode,
    FromCodePoint,
}

impl FunctionBuilder<'_> {
    pub(crate) fn emit_string_constructor_builtin(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let argument = s.reserve_value_local(f);
        let new_target = s.reserve_value_local(f);
        let primitive = s.reserve_value_local(f);
        let pending = s.reserve_completion(f);
        let output = s.reserve_completion(f);
        self.emit_builtin_arg_to_value(0, &argument, f);
        self.compile_new_target_to_locals(&new_target, f)?;
        output.initialize(f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        self.emit_builtin_arg_is_present_i32(0, f);
        self.open_frame(ControlFrameKind::If, f);
        new_target.tag().load(f);
        f.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Undefined.tag()));
        f.instruction(&Instruction::I32Eq);
        argument.tag().load(f);
        f.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Symbol.tag()));
        f.instruction(&Instruction::I32Eq);
        f.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, f);
        let symbol = s
            .reserve_gc_local(f)
            .initialize(argument.cast_reference::<SymbolValue>(s, f), f);
        let descriptive = s
            .reserve_gc_local(f)
            .initialize(self.emit_symbol_descriptive_string(&symbol, f)?, f);
        primitive.set_reference(&descriptive, s, f);
        descriptive.clear(f);
        symbol.clear(f);
        f.instruction(&Instruction::Else);
        self.emit_value_to_string_payload(&argument, &pending, f)?;
        self.emit_native_string_abrupt_exit(&pending, &output, exit, f);
        primitive.copy_from(pending.value(), f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::Else);
        let empty = s
            .reserve_gc_local(f)
            .initialize(self.emit_native_string_static("", f), f);
        primitive.set_reference(&empty, s, f);
        empty.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        output.set_normal(&primitive, f);
        new_target.tag().load(f);
        f.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Undefined.tag()));
        f.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, f);
        // Complete the String conversion before the observable prototype Get.
        self.emit_get_prototype_from_constructor(
            &new_target,
            OrdinaryDefaultPrototype::String,
            &pending,
            f,
        )?;
        self.emit_native_string_abrupt_exit(&pending, &output, exit, f);
        let header = s.reserve_gc_local(f).initialize(
            self.emit_alloc_plain_object_with_prototype(Some(pending.value()), f)?,
            f,
        );
        let string = s
            .reserve_gc_local(f)
            .initialize(primitive.cast_reference::<StringValue>(s, f), f);
        self.emit_initialize_string_object_length(&header, &string, f)?;
        string.clear(f);
        let stored = s
            .reserve_gc_local(f)
            .initialize(s.struct_type::<StoredValue>().from_value(&primitive, f), f);
        let boxed = s.reserve_gc_local(f).initialize(
            s.struct_type::<PrimitiveBox>().construct(
                (
                    GcOperand::reference(&header, s),
                    GcOperand::reference(&stored, s),
                ),
                f,
            ),
            f,
        );
        let value = s.reserve_value_local(f);
        value.set_reference(&boxed, s, f);
        output.set_normal(&value, f);
        value.clear(f);
        boxed.clear(f);
        stored.clear(f);
        header.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        self.completion().copy_from(&output, f);
        output.clear(f);
        pending.clear(f);
        primitive.clear(f);
        new_target.clear(f);
        argument.clear(f);
        Ok(())
    }

    pub(crate) fn emit_string_from_char_code_builtin(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_native_string_character_constructor(StringCharacterConstructor::FromCharCode, f)
    }
    pub(crate) fn emit_string_from_code_point_builtin(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_native_string_character_constructor(StringCharacterConstructor::FromCodePoint, f)
    }

    fn emit_native_string_character_constructor(
        &mut self,
        operation: StringCharacterConstructor,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let arguments = s.reserve_gc_local(f).initialize(
            self.body_entry_locals()
                .expect("native String constructor")
                .arguments()
                .load(s, f),
            f,
        );
        let count = s.reserve_i32_local(f);
        let index = s.reserve_i32_local(f);
        let length = s.reserve_i32_local(f);
        let ordinal = s.reserve_i32_local(f);
        let unit = s.reserve_i32_local(f);
        let code = s.reserve_i64_local(f);
        let argument = s.reserve_value_local(f);
        let pending = s.reserve_completion(f);
        let output = s.reserve_completion(f);
        let accumulated = s
            .reserve_gc_local(f)
            .initialize(self.emit_native_string_static("", f), f);
        s.array_type::<ValueArray>().length(&arguments, s, f);
        count.store(f);
        f.instruction(&Instruction::I32Const(0));
        index.store(f);
        output.initialize(f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        let done = self.open_frame(ControlFrameKind::Block, f);
        let next = self.open_frame(ControlFrameKind::Loop, f);
        index.load(f);
        count.load(f);
        f.instruction(&Instruction::I32GeU);
        self.emit_branch_if_to_target(done, f);
        self.emit_argument_vector_entry_to_value(&arguments, index, &argument, f);
        self.emit_value_to_number_payload(&argument, &pending, f)?;
        self.emit_native_string_abrupt_exit(&pending, &output, exit, f);
        match &operation {
            StringCharacterConstructor::FromCharCode => {
                self.emit_to_uint16_i64_from_number_payload(pending.value().scalar(), code, f);
                f.instruction(&Instruction::I32Const(1));
                length.store(f);
            }
            StringCharacterConstructor::FromCodePoint => {
                pending.value().scalar().load(f);
                f.instruction(&Instruction::F64ReinterpretI64);
                pending.value().scalar().load(f);
                f.instruction(&Instruction::F64ReinterpretI64);
                f.instruction(&Instruction::F64Trunc);
                f.instruction(&Instruction::F64Ne);
                pending.value().scalar().load(f);
                f.instruction(&Instruction::F64ReinterpretI64);
                f.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                f.instruction(&Instruction::F64Lt);
                f.instruction(&Instruction::I32Or);
                pending.value().scalar().load(f);
                f.instruction(&Instruction::F64ReinterpretI64);
                f.instruction(&Instruction::F64Const(Ieee64::from(0x10ffff as f64)));
                f.instruction(&Instruction::F64Gt);
                f.instruction(&Instruction::I32Or);
                self.emit_native_string_error_if(RuntimeErrorMessage::STRING_FROMCODEPOINT_ARGUMENT_MUST_BE_AN_INTEGER_FROM_0_THROUGH_0X10FFFF,NativeErrorKind::RangeError,&output,exit,f)?;
                pending.value().scalar().load(f);
                f.instruction(&Instruction::F64ReinterpretI64);
                f.instruction(&Instruction::I64TruncSatF64U);
                code.store(f);
                code.load(f);
                f.instruction(&Instruction::I64Const(0xffff));
                f.instruction(&Instruction::I64GtU);
                f.instruction(&Instruction::I32Const(1));
                f.instruction(&Instruction::I32Add);
                length.store(f);
            }
        }
        let construction = StringConstruction::allocate(s, s.reserve_gc_local(f), length, f);
        f.instruction(&Instruction::I32Const(0));
        ordinal.store(f);
        length.load(f);
        f.instruction(&Instruction::I32Const(2));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        code.load(f);
        f.instruction(&Instruction::I64Const(0x10000));
        f.instruction(&Instruction::I64Sub);
        f.instruction(&Instruction::I64Const(10));
        f.instruction(&Instruction::I64ShrU);
        f.instruction(&Instruction::I64Const(0xd800));
        f.instruction(&Instruction::I64Add);
        f.instruction(&Instruction::I32WrapI64);
        unit.store(f);
        construction.write(ordinal, unit, s, f);
        f.instruction(&Instruction::I32Const(1));
        ordinal.store(f);
        code.load(f);
        f.instruction(&Instruction::I64Const(0x10000));
        f.instruction(&Instruction::I64Sub);
        f.instruction(&Instruction::I64Const(0x3ff));
        f.instruction(&Instruction::I64And);
        f.instruction(&Instruction::I64Const(0xdc00));
        f.instruction(&Instruction::I64Add);
        f.instruction(&Instruction::I32WrapI64);
        unit.store(f);
        construction.write(ordinal, unit, s, f);
        f.instruction(&Instruction::Else);
        code.load(f);
        f.instruction(&Instruction::I32WrapI64);
        unit.store(f);
        construction.write(ordinal, unit, s, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let part = s
            .reserve_gc_local(f)
            .initialize(construction.publish(s, f), f);
        accumulated.replace(self.emit_concat_gc_strings(&accumulated, &part, f), f);
        part.clear(f);
        argument.set_undefined(f);
        index.load(f);
        f.instruction(&Instruction::I32Const(1));
        f.instruction(&Instruction::I32Add);
        index.store(f);
        self.emit_branch_to_target(next, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        self.emit_native_string_normal_reference(&accumulated, &output, f);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        self.completion().copy_from(&output, f);
        accumulated.clear(f);
        output.clear(f);
        pending.clear(f);
        argument.clear(f);
        s.release_i64_local(code, f);
        s.release_i32_local(unit, f);
        s.release_i32_local(ordinal, f);
        s.release_i32_local(length, f);
        s.release_i32_local(index, f);
        s.release_i32_local(count, f);
        arguments.clear(f);
        Ok(())
    }

    pub(super) fn emit_native_string_numeric_key(
        &mut self,
        index: I64Local,
        f: &mut Function,
    ) -> Result<PropertyKeyLocals, EmitError> {
        let s = self.runtime_schema();
        let bits = s.reserve_i64_local(f);
        index.load(f);
        f.instruction(&Instruction::F64ConvertI64U);
        f.instruction(&Instruction::I64ReinterpretF64);
        bits.store(f);
        let text = s
            .call_helper(
                crate::runtime_helpers::NumberToStringArguments::new(bits),
                self.runtime_helper_base()?,
                f,
            )
            .bind(s, s.reserve_gc_local(f), f);
        let key = PropertyKeyLocals::from_string(s, &text, f);
        text.clear(f);
        s.release_i64_local(bits, f);
        Ok(key)
    }

    pub(crate) fn emit_string_raw_builtin(&mut self, f: &mut Function) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let template = s.reserve_value_local(f);
        let raw = s.reserve_value_local(f);
        let output = s.reserve_completion(f);
        let pending = s.reserve_completion(f);
        let literal = s.reserve_completion(f);
        let length = s.reserve_i64_local(f);
        let index = s.reserve_i64_local(f);
        let arg_index = s.reserve_i32_local(f);
        let arguments = s.reserve_gc_local(f).initialize(
            self.body_entry_locals()
                .expect("String.raw entry")
                .arguments()
                .load(s, f),
            f,
        );
        let accumulated = s
            .reserve_gc_local(f)
            .initialize(self.emit_native_string_static("", f), f);
        self.emit_builtin_arg_to_value(0, &template, f);
        output.initialize(f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        self.emit_value_to_current_function_realm_object_locals(&template, &pending, f)?;
        self.emit_native_string_abrupt_exit(&pending, &output, exit, f);
        template.copy_from(pending.value(), f);
        let key_text = s
            .reserve_gc_local(f)
            .initialize(self.emit_native_string_static("raw", f), f);
        let key = PropertyKeyLocals::from_string(s, &key_text, f);
        key_text.clear(f);
        self.emit_object_read(&template, &template, &key, &pending, f)?;
        key.clear(f);
        self.emit_native_string_abrupt_exit(&pending, &output, exit, f);
        raw.copy_from(pending.value(), f);
        self.emit_value_to_current_function_realm_object_locals(&raw, &pending, f)?;
        self.emit_native_string_abrupt_exit(&pending, &output, exit, f);
        raw.copy_from(pending.value(), f);
        let length_text = s
            .reserve_gc_local(f)
            .initialize(self.emit_native_string_static("length", f), f);
        let key = PropertyKeyLocals::from_string(s, &length_text, f);
        length_text.clear(f);
        self.emit_object_read(&raw, &raw, &key, &pending, f)?;
        key.clear(f);
        self.emit_native_string_abrupt_exit(&pending, &output, exit, f);
        template.copy_from(pending.value(), f);
        self.emit_to_length_i64_from_value_locals(&template, length, &pending, f)?;
        self.emit_native_string_abrupt_exit(&pending, &output, exit, f);
        f.instruction(&Instruction::I64Const(0));
        index.store(f);
        let done = self.open_frame(ControlFrameKind::Block, f);
        let next = self.open_frame(ControlFrameKind::Loop, f);
        index.load(f);
        length.load(f);
        f.instruction(&Instruction::I64GeU);
        self.emit_branch_if_to_target(done, f);
        let key = self.emit_native_string_numeric_key(index, f)?;
        self.emit_object_read(&raw, &raw, &key, &literal, f)?;
        key.clear(f);
        self.emit_native_string_abrupt_exit(&literal, &output, exit, f);
        self.emit_value_to_string_payload(literal.value(), &pending, f)?;
        self.emit_native_string_abrupt_exit(&pending, &output, exit, f);
        let part = s
            .reserve_gc_local(f)
            .initialize(pending.value().cast_reference::<StringValue>(s, f), f);
        accumulated.replace(self.emit_concat_gc_strings(&accumulated, &part, f), f);
        part.clear(f);
        index.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        index.store(f);
        index.load(f);
        length.load(f);
        f.instruction(&Instruction::I64GeU);
        self.emit_branch_if_to_target(done, f);
        index.load(f);
        self.body_entry_locals()
            .expect("String.raw entry")
            .argument_count()
            .load(f);
        f.instruction(&Instruction::I64LtU);
        self.open_frame(ControlFrameKind::If, f);
        index.load(f);
        f.instruction(&Instruction::I32WrapI64);
        arg_index.store(f);
        self.emit_argument_vector_entry_to_value(&arguments, arg_index, &template, f);
        self.emit_value_to_string_payload(&template, &pending, f)?;
        self.emit_native_string_abrupt_exit(&pending, &output, exit, f);
        let part = s
            .reserve_gc_local(f)
            .initialize(pending.value().cast_reference::<StringValue>(s, f), f);
        accumulated.replace(self.emit_concat_gc_strings(&accumulated, &part, f), f);
        part.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.emit_branch_to_target(next, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        self.emit_native_string_normal_reference(&accumulated, &output, f);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        self.completion().copy_from(&output, f);
        accumulated.clear(f);
        arguments.clear(f);
        s.release_i32_local(arg_index, f);
        s.release_i64_local(index, f);
        s.release_i64_local(length, f);
        literal.clear(f);
        pending.clear(f);
        output.clear(f);
        raw.clear(f);
        template.clear(f);
        Ok(())
    }
}
