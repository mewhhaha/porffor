//! A program-owned JS wrapper calls only its declared scalar import.
use super::*;
use scriptw_ir::foreign::{ForeignCallIr, ForeignResultType, ForeignScalarType};

impl FunctionBuilder<'_> {
    pub(super) fn emit_foreign_call(
        &mut self,
        call: &ForeignCallIr,
        output: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let planned = self
            .functions
            .foreign_imports()
            .function(call.function())?
            .clone();
        if call.operands().len() != planned.parameters.len() {
            return Err(EmitError::unsupported(
                "foreign wrapper operand arity differs from its declared import",
            ));
        }
        let error_tag = self.functions.foreign_imports().error_tag();
        let schema = self.runtime_schema();
        let mut arguments = Vec::with_capacity(call.operands().len());
        for operand in call.operands() {
            let value = schema.reserve_value_local(function);
            self.compile_expr_to_value(operand, &value, function)?;
            self.emit_propagate_current_throw_if_needed(function);
            arguments.push(value);
        }
        // Ordinary Call evaluated every actual JS argument, including extras.
        // Marshalling never applies ToNumber or invokes a user coercion hook.
        for (argument, parameter) in arguments.iter().zip(&planned.parameters) {
            match parameter {
                ForeignScalarType::Number => {
                    argument.tag().load(function);
                    function
                        .instruction(&Instruction::I32Const(WasmRuntimeValueTag::Number as i32));
                    function.instruction(&Instruction::I32Ne);
                    self.open_frame(ControlFrameKind::If, function);
                    self.emit_expression_native_error(
                        NativeErrorKind::TypeError,
                        RuntimeErrorMessage::FOREIGN_NUMBER_ARGUMENT_REQUIRED,
                        function,
                    )?;
                    self.pop_control(ControlFrameKind::If);
                    function.instruction(&Instruction::End);
                }
            }
        }
        let done = error_tag.map(|tag| {
            function.instruction(&Instruction::Block(BlockType::Empty));
            let done = function.label_depth();
            function.instruction(&Instruction::Block(BlockType::Result(ValType::I32)));
            // The immediately enclosing result block is this catch's label 0.
            // Only the explicitly declared i32-code tag reaches that block.
            function.instruction(&Instruction::TryTable(
                BlockType::Empty,
                vec![tag.catch(0)].into(),
            ));
            done
        });
        for argument in &arguments {
            argument.scalar().load(function);
            function.instruction(&Instruction::F64ReinterpretI64);
        }
        planned.index.emit_call(function);
        match planned.result {
            ForeignResultType::Number => {
                let bits = schema.reserve_i64_local(function);
                function.instruction(&Instruction::I64ReinterpretF64);
                bits.store(function);
                output.set_number(bits, function);
                schema.release_i64_local(bits, function);
            }
            ForeignResultType::Void => output.set_undefined(function),
        }
        if let Some(done) = done {
            function.branch_to_label(done);
            function.instruction(&Instruction::End); // try_table
            function.instruction(&Instruction::Unreachable); // only catch supplies i32
            function.instruction(&Instruction::End); // catch result block
            let code = schema.reserve_i32_local(function);
            code.store(function);
            self.emit_foreign_error_code(code, function)?;
            schema.release_i32_local(code, function);
            function.instruction(&Instruction::End); // successful-call block
        }
        for argument in arguments {
            argument.clear(function);
        }
        Ok(())
    }

    fn emit_foreign_error_code(
        &mut self,
        code: crate::gc_types::I32Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let pending = schema.reserve_completion(function);
        pending.initialize(function);
        self.emit_throw_runtime_error(
            NativeErrorKind::Error,
            RuntimeErrorMessage::FOREIGN_HOST_OPERATION_FAILED,
            &pending,
            function,
        )?;
        let code_value = schema.reserve_value_local(function);
        let bits = schema.reserve_i64_local(function);
        code.load(function);
        function.instruction(&Instruction::F64ConvertI32S);
        function.instruction(&Instruction::I64ReinterpretF64);
        bits.store(function);
        code_value.set_number(bits, function);
        schema.release_i64_local(bits, function);
        let text = schema.reserve_gc_local(function).initialize(
            self.emit_interned_string_reference("code", function)?,
            function,
        );
        let key = PropertyKeyLocals::from_string(schema, &text, function);
        let definition = schema.reserve_completion(function);
        definition.initialize(function);
        self.emit_create_data_property_or_throw(
            pending.value(),
            &key,
            &code_value,
            &definition,
            function,
        )?;
        self.completion().copy_from(&definition, function);
        self.emit_propagate_current_throw_if_needed(function);
        key.clear(function);
        text.clear(function);
        code_value.clear(function);
        definition.clear(function);
        self.completion().copy_from(&pending, function);
        pending.clear(function);
        self.emit_propagate_current_throw_if_needed(function);
        Ok(())
    }
}
