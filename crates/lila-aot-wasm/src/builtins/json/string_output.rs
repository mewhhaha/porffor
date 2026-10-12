//! Private UTF-16 assembly of strings already serialized in observable order.
use super::*;

/// Spare capacity belongs only to this unpublished owner. Consuming finish
/// copies the used prefix into an exact-length immutable String.
#[must_use]
pub(super) struct StringOutputConstruction {
    output: StringConstruction,
    length: I32Local,
    capacity: I32Local,
}

impl StringOutputConstruction {
    pub(super) fn new(schema: &RuntimeSchema, function: &mut Function) -> Self {
        let length = schema.reserve_i32_local(function);
        let capacity = schema.reserve_i32_local(function);
        length.set_constant(0, function);
        capacity.set_constant(128, function);
        let output = StringConstruction::allocate(
            schema,
            schema.reserve_gc_local(function),
            capacity,
            function,
        );
        Self {
            output,
            length,
            capacity,
        }
    }

    pub(super) fn append(
        &self,
        string: &GcLocal<StringValue>,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) {
        let source = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StringValue>()
                .field(StringValueSchema::CODE_UNITS)
                .read(string, schema, function)
                .reference(),
            function,
        );
        let part_length = schema.reserve_i32_local(function);
        schema
            .array_type::<CodeUnitArray>()
            .length(&source, schema, function);
        part_length.store(function);
        let next_length = self.reserve_append(part_length, schema, function);
        self.output
            .copy_units_from(self.length, &source, part_length, schema, function);
        next_length.load(function);
        self.length.store(function);
        schema.release_i32_local(next_length, function);
        schema.release_i32_local(part_length, function);
        source.clear(function);
    }

    /// Fixed compiler text contributes only its UTF-16 units. No temporary
    /// JavaScript string or observable conversion is needed for JSON punctuation.
    pub(super) fn append_literal(
        &self,
        text: &str,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) {
        let units: Vec<_> = text.encode_utf16().collect();
        let part_length = schema.reserve_i32_local(function);
        part_length.set_constant(
            i32::try_from(units.len()).expect("compiler literal fits the Wasm string domain"),
            function,
        );
        let next_length = self.reserve_append(part_length, schema, function);
        let index = schema.reserve_i32_local(function);
        let unit = schema.reserve_i32_local(function);
        for (offset, code_unit) in units.into_iter().enumerate() {
            self.length.load(function);
            function.instruction(&Instruction::I32Const(
                i32::try_from(offset).expect("compiler literal offset fits the Wasm string domain"),
            ));
            function.instruction(&Instruction::I32Add);
            index.store(function);
            unit.set_constant(i32::from(code_unit), function);
            self.output.write(index, unit, schema, function);
        }
        next_length.load(function);
        self.length.store(function);
        schema.release_i32_local(unit, function);
        schema.release_i32_local(index, function);
        schema.release_i32_local(next_length, function);
        schema.release_i32_local(part_length, function);
    }

    /// Reserve the next prefix before either bulk copy or literal unit writes.
    fn reserve_append(
        &self,
        part_length: I32Local,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) -> I32Local {
        let next_length = schema.reserve_i32_local(function);
        self.length.load(function);
        part_length.load(function);
        function.instruction(&Instruction::I32Add);
        next_length.store(function);
        next_length.load(function);
        self.length.load(function);
        function.instruction(&Instruction::I32LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);

        next_length.load(function);
        self.capacity.load(function);
        function.instruction(&Instruction::I32GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        let grown_capacity = schema.reserve_i32_local(function);
        self.capacity.load(function);
        self.capacity.load(function);
        function.instruction(&Instruction::I32Add);
        grown_capacity.store(function);
        grown_capacity.load(function);
        self.capacity.load(function);
        function.instruction(&Instruction::I32LtU);
        grown_capacity.load(function);
        next_length.load(function);
        function.instruction(&Instruction::I32LtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        next_length.load(function);
        grown_capacity.store(function);
        function.instruction(&Instruction::End);
        self.output
            .grow(grown_capacity, self.length, schema, function);
        grown_capacity.load(function);
        self.capacity.store(function);
        schema.release_i32_local(grown_capacity, function);
        function.instruction(&Instruction::End);

        next_length
    }

    pub(super) fn finish(
        self,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) -> GcLocal<StringValue> {
        let result = schema.reserve_gc_local(function).initialize(
            self.output.publish_prefix(self.length, schema, function),
            function,
        );
        schema.release_i32_local(self.capacity, function);
        schema.release_i32_local(self.length, function);
        result
    }
}
