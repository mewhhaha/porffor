use super::*;

impl FunctionBuilder<'_> {
    /// Builds one pooled literal from the passive UTF-16 segment and stores it
    /// at its slot.
    fn emit_pooled_string_slot(
        &self,
        table: &GcLocal<PooledStringTable>,
        index: I32Local,
        ordinal: crate::data::PooledStringIndex,
        literal: &crate::data::PooledCodeUnits,
        function: &mut Function,
    ) {
        let schema = self.schema;
        let units_slot = schema.reserve_gc_local::<CodeUnitArray, NonNullable>(function);
        let builder =
            StringConstruction::from_pooled_literal(schema, units_slot, literal, function);
        let string_slot = schema.reserve_gc_local::<StringValue, NonNullable>(function);
        let string = string_slot.initialize(builder.publish(schema, function), function);
        function.instruction(&Instruction::I32Const(ordinal.ordinal() as i32));
        index.store(function);
        schema.array_type::<PooledStringTable>().write(
            table,
            index,
            GcOperand::nullable_reference(&string, schema),
            schema,
            function,
        );
        string.clear(function);
    }

    /// R owns exactly its stable prefix, the well-known symbols and registry.
    /// There is no parameter through which a P can resize the shared table.
    pub(crate) fn compile_pooled_strings_initialize_helper(
        &mut self,
    ) -> Result<Function, EmitError> {
        self.strings.require_runtime_only()?;
        let mut function = self.begin_helper_body(RuntimeHelperId::PooledStringsInitialize);
        let parameters = self
            .helper_parameters::<crate::runtime_helpers::PooledStringsInitializeParameters>(
                &mut function,
            );
        let schema = self.schema;
        let function_body = &mut function;
        let length = schema.reserve_i32_local(function_body);
        let index = schema.reserve_i32_local(function_body);
        length.set_constant(
            self.strings.compiler_owned_boundary().strings() as i32,
            function_body,
        );
        let table_slot = schema.reserve_gc_local::<PooledStringTable, NonNullable>(function_body);
        let table = table_slot.initialize(
            schema.array_type::<PooledStringTable>().filled(
                GcOperand::null(schema),
                length,
                function_body,
            ),
            function_body,
        );
        for (ordinal, literal) in self.strings.runtime_pooled_string_initializers() {
            self.emit_pooled_string_slot(&table, index, ordinal, literal, function_body);
        }
        let mut symbols = Vec::new();
        for symbol in lila_ir::WellKnownSymbol::ALL {
            let description_slot =
                schema.reserve_gc_local::<StringValue, NonNullable>(function_body);
            function_body.instruction(&Instruction::I32Const(
                self.strings
                    .pooled_string_index(symbol.description())?
                    .ordinal() as i32,
            ));
            index.store(function_body);
            let description = description_slot.initialize(
                schema
                    .array_type::<PooledStringTable>()
                    .read(&table, index, schema, function_body)
                    .reference()
                    .require_non_null(function_body),
                function_body,
            );
            let symbol_slot = schema.reserve_gc_local::<SymbolValue, NonNullable>(function_body);
            let value = symbol_slot.initialize(
                schema.struct_type::<SymbolValue>().construct(
                    (
                        GcOperand::nullable_reference(&description, schema),
                        GcOperand::null(schema),
                        GcOperand::i64(0),
                    ),
                    function_body,
                ),
                function_body,
            );
            description.clear(function_body);
            symbols.push(value);
        }
        let symbols_slot =
            schema.reserve_gc_local::<WellKnownSymbolTable, NonNullable>(function_body);
        let symbol_table = symbols_slot.initialize(
            schema.array_type::<WellKnownSymbolTable>().fixed(
                symbols
                    .iter()
                    .map(|symbol| GcOperand::reference(symbol, schema)),
                function_body,
            ),
            function_body,
        );
        schema.publish_pooled_strings(&table, function_body);
        schema.publish_well_known_symbols(&symbol_table, function_body);
        let registry = schema
            .reserve_gc_local::<RegisteredSymbolTable, NonNullable>(function_body)
            .initialize(
                schema
                    .array_type::<RegisteredSymbolTable>()
                    .fixed([], function_body),
                function_body,
            );
        schema.replace_symbol_registry(&registry, function_body);
        registry.clear(function_body);
        symbol_table.clear(function_body);
        for symbol in symbols.into_iter().rev() {
            symbol.clear(function_body);
        }
        table.clear(function_body);
        schema.release_i32_local(index, function_body);
        schema.release_i32_local(length, function_body);
        crate::runtime_helpers::HelperParameters::release(parameters, function_body);
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }
}
