use super::*;
use crate::gc_types::{
    CodeUnitArray, FunctionObject, GcLocal, GcStackReference, Nullable, StringConstruction,
    StringValue, StringValueSchema, SymbolValue, SymbolValueSchema,
};
use crate::operations::PropertyKeyLocals;

#[derive(Clone, Copy)]
pub(crate) enum FunctionNamePrefix {
    None,
    Getter,
    Setter,
}

impl FunctionBuilder<'_> {
    /// Concatenation copies exact UTF-16 units into one immutable StringValue.
    pub(crate) fn emit_concat_gc_strings(
        &self,
        left: &GcLocal<StringValue>,
        right: &GcLocal<StringValue>,
        function: &mut Function,
    ) -> GcStackReference<StringValue> {
        let schema = self.runtime_schema();
        let units = schema.array_type::<CodeUnitArray>();
        let left_units = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StringValue>()
                .field(StringValueSchema::CODE_UNITS)
                .read(left, schema, function)
                .reference(),
            function,
        );
        let right_units = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StringValue>()
                .field(StringValueSchema::CODE_UNITS)
                .read(right, schema, function)
                .reference(),
            function,
        );
        let left_length = schema.reserve_i32_local(function);
        let right_length = schema.reserve_i32_local(function);
        let total = schema.reserve_i32_local(function);
        let source_index = schema.reserve_i32_local(function);
        let destination_index = schema.reserve_i32_local(function);
        let unit = schema.reserve_i32_local(function);
        units.length(&left_units, schema, function);
        left_length.store(function);
        units.length(&right_units, schema, function);
        right_length.store(function);
        left_length.load(function);
        right_length.load(function);
        function.instruction(&Instruction::I32Add);
        total.store(function);
        total.load(function);
        left_length.load(function);
        function.instruction(&Instruction::I32LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        let output = StringConstruction::allocate(
            schema,
            schema.reserve_gc_local(function),
            total,
            function,
        );
        function.instruction(&Instruction::I32Const(0));
        destination_index.store(function);
        for (source, length) in [(&left_units, left_length), (&right_units, right_length)] {
            function.instruction(&Instruction::I32Const(0));
            source_index.store(function);
            function.instruction(&Instruction::Block(BlockType::Empty));
            function.instruction(&Instruction::Loop(BlockType::Empty));
            source_index.load(function);
            length.load(function);
            function.instruction(&Instruction::I32GeU);
            function.instruction(&Instruction::BrIf(1));
            units
                .read(source, source_index, schema, function)
                .store(unit, function);
            output.write(destination_index, unit, schema, function);
            source_index.load(function);
            function.instruction(&Instruction::I32Const(1));
            function.instruction(&Instruction::I32Add);
            source_index.store(function);
            destination_index.load(function);
            function.instruction(&Instruction::I32Const(1));
            function.instruction(&Instruction::I32Add);
            destination_index.store(function);
            function.instruction(&Instruction::Br(0));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }
        let value = output.publish(schema, function);
        schema.release_i32_local(unit, function);
        schema.release_i32_local(destination_index, function);
        schema.release_i32_local(source_index, function);
        schema.release_i32_local(total, function);
        schema.release_i32_local(right_length, function);
        schema.release_i32_local(left_length, function);
        right_units.clear(function);
        left_units.clear(function);
        value
    }

    /// SymbolDescriptiveString is distinct from implicit ToString, which
    /// continues to reject a Symbol primitive.
    pub(crate) fn emit_symbol_descriptive_string(
        &mut self,
        symbol: &GcLocal<SymbolValue>,
        function: &mut Function,
    ) -> Result<GcStackReference<StringValue>, EmitError> {
        let schema = self.runtime_schema();
        let description = schema
            .reserve_gc_local::<StringValue, Nullable>(function)
            .initialize(
                schema
                    .struct_type::<SymbolValue>()
                    .field(SymbolValueSchema::DESCRIPTION)
                    .read(symbol, schema, function)
                    .reference(),
                function,
            );
        let output = schema
            .reserve_gc_local::<StringValue, Nullable>(function)
            .initialize_null(schema, function);
        description.load(schema, function);
        function.instruction(&Instruction::RefIsNull);
        self.open_frame(ControlFrameKind::If, function);
        output.replace(
            self.emit_interned_string_reference("Symbol()", function)?
                .nullable(),
            function,
        );
        function.instruction(&Instruction::Else);
        let text = schema.reserve_gc_local(function).initialize(
            description
                .load(schema, function)
                .require_non_null(function),
            function,
        );
        let prefix = schema.reserve_gc_local(function).initialize(
            self.emit_interned_string_reference("Symbol(", function)?,
            function,
        );
        let partial = schema.reserve_gc_local(function).initialize(
            self.emit_concat_gc_strings(&prefix, &text, function),
            function,
        );
        let suffix = schema.reserve_gc_local(function).initialize(
            self.emit_interned_string_reference(")", function)?,
            function,
        );
        output.replace(
            self.emit_concat_gc_strings(&partial, &suffix, function)
                .nullable(),
            function,
        );
        suffix.clear(function);
        partial.clear(function);
        prefix.clear(function);
        text.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let value = output.load(schema, function).require_non_null(function);
        output.clear(function);
        description.clear(function);
        Ok(value)
    }

    pub(crate) fn emit_set_function_name(
        &mut self,
        callable: &GcLocal<FunctionObject>,
        property_key: &PropertyKeyLocals,
        prefix: FunctionNamePrefix,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let name = schema
            .reserve_gc_local::<StringValue, Nullable>(function)
            .initialize_null(schema, function);
        property_key.value().tag().load(function);
        function.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Symbol as i32));
        function.instruction(&Instruction::I32Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        let symbol = schema.reserve_gc_local(function).initialize(
            property_key
                .value()
                .cast_reference::<SymbolValue>(schema, function),
            function,
        );
        let description = schema
            .reserve_gc_local::<StringValue, Nullable>(function)
            .initialize(
                schema
                    .struct_type::<SymbolValue>()
                    .field(SymbolValueSchema::DESCRIPTION)
                    .read(&symbol, schema, function)
                    .reference(),
                function,
            );
        description.load(schema, function);
        function.instruction(&Instruction::RefIsNull);
        function.instruction(&Instruction::If(BlockType::Empty));
        name.replace(
            self.emit_interned_string_reference("", function)?
                .nullable(),
            function,
        );
        function.instruction(&Instruction::Else);
        let description_value = schema.reserve_gc_local(function).initialize(
            description
                .load(schema, function)
                .require_non_null(function),
            function,
        );
        let open = schema.reserve_gc_local(function).initialize(
            self.emit_interned_string_reference("[", function)?,
            function,
        );
        let partial = schema.reserve_gc_local(function).initialize(
            self.emit_concat_gc_strings(&open, &description_value, function),
            function,
        );
        let close = schema.reserve_gc_local(function).initialize(
            self.emit_interned_string_reference("]", function)?,
            function,
        );
        name.replace(
            self.emit_concat_gc_strings(&partial, &close, function)
                .nullable(),
            function,
        );
        close.clear(function);
        partial.clear(function);
        open.clear(function);
        description_value.clear(function);
        function.instruction(&Instruction::End);
        description.clear(function);
        symbol.clear(function);
        function.instruction(&Instruction::Else);
        name.replace(
            property_key
                .value()
                .cast_reference::<StringValue>(schema, function)
                .nullable(),
            function,
        );
        function.instruction(&Instruction::End);
        let prefix = match prefix {
            FunctionNamePrefix::None => None,
            FunctionNamePrefix::Getter => Some("get "),
            FunctionNamePrefix::Setter => Some("set "),
        };
        let final_name = schema.reserve_gc_local(function).initialize(
            name.load(schema, function).require_non_null(function),
            function,
        );
        if let Some(prefix) = prefix {
            let prefix = schema.reserve_gc_local(function).initialize(
                self.emit_interned_string_reference(prefix, function)?,
                function,
            );
            final_name.replace(
                self.emit_concat_gc_strings(&prefix, &final_name, function),
                function,
            );
            prefix.clear(function);
        }
        let value = schema.reserve_value_local(function);
        value.set_reference(&final_name, schema, function);
        let target = schema.reserve_value_local(function);
        target.set_reference(callable, schema, function);
        let pending = schema.reserve_completion(function);
        let name_key = self.emit_function_string_key("name", function)?;
        // Allocation has already published the configurable name descriptor.
        // Definition replaces it in place, retaining one key and its order.
        self.emit_object_define_data_with_configurable(
            &target, &name_key, &value, false, false, true, &pending, function,
        )?;
        name_key.clear(function);
        target.clear(function);
        value.clear(function);
        final_name.clear(function);
        name.clear(function);
        pending.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.completion().copy_from(&pending, function);
        self.emit_propagate_current_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        pending.clear(function);
        Ok(())
    }
}
