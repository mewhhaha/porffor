use super::*;

/// The only UTF-16 reading authority used by Date parsing. Bounds checks
/// precede GC array reads; EOF cannot observe an adjacent value.
pub(super) struct DateParseCursor {
    units: GcLocal<CodeUnitArray>,
    length: I64Local,
    index: I64Local,
    byte: I64Local,
    pub(super) valid: I64Local,
}

impl DateParseCursor {
    pub(super) fn new(
        builder: &mut FunctionBuilder<'_>,
        source: &GcLocal<StringValue>,
        function: &mut Function,
    ) -> Self {
        let schema = builder.runtime_schema();
        let units = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StringValue>()
                .field(StringValueSchema::CODE_UNITS)
                .read(source, schema, function)
                .reference(),
            function,
        );
        let cursor = Self {
            units,
            length: schema.reserve_i64_local(function),
            index: schema.reserve_i64_local(function),
            byte: schema.reserve_i64_local(function),
            valid: schema.reserve_i64_local(function),
        };
        schema
            .array_type::<CodeUnitArray>()
            .length(&cursor.units, schema, function);
        function.instruction(&Instruction::I64ExtendI32U);
        cursor.length.store(function);
        function.instruction(&Instruction::I64Const(0));
        cursor.index.store(function);
        function.instruction(&Instruction::I64Const(1));
        cursor.valid.store(function);
        cursor
    }

    fn read_unit(
        &self,
        builder: &FunctionBuilder<'_>,
        position: I64Local,
        output: I64Local,
        function: &mut Function,
    ) {
        let schema = builder.runtime_schema();
        let index = schema.reserve_i32_local(function);
        let unit = schema.reserve_i32_local(function);
        position.load(function);
        function.instruction(&Instruction::I32WrapI64);
        index.store(function);
        schema
            .array_type::<CodeUnitArray>()
            .read(&self.units, index, schema, function)
            .store(unit, function);
        unit.load(function);
        function.instruction(&Instruction::I64ExtendI32U);
        output.store(function);
        schema.release_i32_local(unit, function);
        schema.release_i32_local(index, function);
    }

    fn peek(&self, builder: &FunctionBuilder<'_>, function: &mut Function) {
        self.index.load(function);
        self.length.load(function);
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.read_unit(builder, self.index, self.byte, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        self.byte.store(function);
        function.instruction(&Instruction::End);
    }

    pub(super) fn at(&self, builder: &FunctionBuilder<'_>, expected: u8, function: &mut Function) {
        self.peek(builder, function);
        self.byte.load(function);
        function.instruction(&Instruction::I64Const(expected as i64));
        function.instruction(&Instruction::I64Eq);
    }

    pub(super) fn advance(&self, function: &mut Function) {
        self.index.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        self.index.store(function);
    }

    /// Look ahead without consuming input or loading beyond the source span.
    pub(super) fn at_bytes(
        &self,
        builder: &mut FunctionBuilder<'_>,
        expected: &[u8],
        function: &mut Function,
    ) {
        let position = builder.runtime_schema().reserve_i64_local(function);
        let matched = builder.runtime_schema().reserve_i64_local(function);
        function.instruction(&Instruction::I64Const(1));
        matched.store(function);
        for (delta, byte) in expected.iter().enumerate() {
            self.index.load(function);
            function.instruction(&Instruction::I64Const(delta as i64));
            function.instruction(&Instruction::I64Add);
            position.store(function);
            position.load(function);
            self.length.load(function);
            function.instruction(&Instruction::I64LtU);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.read_unit(builder, position, self.byte, function);
            self.byte.load(function);
            function.instruction(&Instruction::I64Const(*byte as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I64ExtendI32U);
            matched.load(function);
            function.instruction(&Instruction::I64And);
            matched.store(function);
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::I64Const(0));
            matched.store(function);
            function.instruction(&Instruction::End);
        }
        matched.load(function);
        function.instruction(&Instruction::I32WrapI64);
        builder
            .runtime_schema()
            .release_i64_local(matched, function);
        builder
            .runtime_schema()
            .release_i64_local(position, function);
    }

    /// The emitted non-UTC display suffix contains a bounded ASCII primary
    /// identifier followed by a comma. Its explicit offset determines parsing.
    pub(super) fn display_time_zone_name(
        &self,
        builder: &mut FunctionBuilder<'_>,
        function: &mut Function,
    ) {
        let count = builder.runtime_schema().reserve_i64_local(function);
        function.instruction(&Instruction::I64Const(0));
        count.store(function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.at(builder, b',', function);
        function.instruction(&Instruction::BrIf(1));
        count.load(function);
        function.instruction(&Instruction::I64Const(
            lila_intl::MAX_TIME_ZONE_IDENTIFIER_BYTES as i64,
        ));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::I32Const(0));
        for (lower, upper) in [(b'A', b'Z'), (b'a', b'z'), (b'0', b'9')] {
            self.byte.load(function);
            function.instruction(&Instruction::I64Const(lower as i64));
            function.instruction(&Instruction::I64GeU);
            self.byte.load(function);
            function.instruction(&Instruction::I64Const(upper as i64));
            function.instruction(&Instruction::I64LeU);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::I32Or);
        }
        for byte in b"/._+-:" {
            self.byte.load(function);
            function.instruction(&Instruction::I64Const(*byte as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
        }
        self.require(function);
        self.advance(function);
        count.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        count.store(function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        count.load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtU);
        self.require(function);
        self.at(builder, b',', function);
        self.require(function);
        builder.runtime_schema().release_i64_local(count, function);
    }

    /// Consume an i32 predicate without allowing a later success to erase a
    /// previous failure. Every parser starts with a fresh validity local.
    pub(super) fn require(&self, function: &mut Function) {
        function.instruction(&Instruction::I64ExtendI32U);
        self.valid.load(function);
        function.instruction(&Instruction::I64And);
        self.valid.store(function);
    }

    pub(super) fn expect(
        &self,
        builder: &FunctionBuilder<'_>,
        bytes: &[u8],
        function: &mut Function,
    ) {
        for &byte in bytes {
            self.at(builder, byte, function);
            self.require(function);
            self.advance(function);
        }
    }

    fn digit(&self, function: &mut Function) {
        self.byte.load(function);
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64GeU);
        self.byte.load(function);
        function.instruction(&Instruction::I64Const(b'9' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
    }

    fn append_digit(&self, dest: I64Local, function: &mut Function) {
        dest.load(function);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Mul);
        self.byte.load(function);
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Add);
        dest.store(function);
        self.advance(function);
    }

    pub(super) fn decimal(
        &self,
        builder: &FunctionBuilder<'_>,
        digits: usize,
        dest: I64Local,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(0));
        dest.store(function);
        for _ in 0..digits {
            self.peek(builder, function);
            self.digit(function);
            self.require(function);
            self.append_digit(dest, function);
        }
        dest.load(function);
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        dest.store(function);
    }

    pub(super) fn display_year(
        &self,
        builder: &mut FunctionBuilder<'_>,
        dest: I64Local,
        function: &mut Function,
    ) {
        let negative = builder.runtime_schema().reserve_i64_local(function);
        let count = builder.runtime_schema().reserve_i64_local(function);
        self.at(builder, b'-', function);
        function.instruction(&Instruction::I64ExtendI32U);
        negative.store(function);
        negative.load(function);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.advance(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        count.store(function);
        function.instruction(&Instruction::I64Const(0));
        dest.store(function);
        // The emitted display formats have at least four and at most six year
        // digits throughout the TimeClip range. No unbounded integer parsing.
        for _ in 0..6 {
            self.peek(builder, function);
            self.digit(function);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.append_digit(dest, function);
            count.load(function);
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Add);
            count.store(function);
            function.instruction(&Instruction::End);
        }
        count.load(function);
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64GeU);
        self.require(function);
        negative.load(function);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        dest.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.require(function);
        function.instruction(&Instruction::I64Const(0));
        dest.load(function);
        function.instruction(&Instruction::I64Sub);
        dest.store(function);
        function.instruction(&Instruction::End);
        dest.load(function);
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        dest.store(function);
        builder.runtime_schema().release_i64_local(count, function);
        builder
            .runtime_schema()
            .release_i64_local(negative, function);
    }

    pub(super) fn name(
        &self,
        builder: &mut FunctionBuilder<'_>,
        names: &[[u8; 3]],
        dest: I64Local,
        function: &mut Function,
    ) {
        let packed = builder.runtime_schema().reserve_i64_local(function);
        function.instruction(&Instruction::I64Const(0));
        packed.store(function);
        for _ in 0..3 {
            self.peek(builder, function);
            packed.load(function);
            function.instruction(&Instruction::I64Const(256));
            function.instruction(&Instruction::I64Mul);
            self.byte.load(function);
            function.instruction(&Instruction::I64Add);
            packed.store(function);
            self.advance(function);
        }
        function.instruction(&Instruction::I64Const(-1));
        dest.store(function);
        for (index, name) in names.iter().enumerate() {
            let value = ((name[0] as i64) << 16) | ((name[1] as i64) << 8) | name[2] as i64;
            packed.load(function);
            function.instruction(&Instruction::I64Const(value));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(index as i64));
            dest.store(function);
            function.instruction(&Instruction::End);
        }
        dest.load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        self.require(function);
        dest.load(function);
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        dest.store(function);
        builder.runtime_schema().release_i64_local(packed, function);
    }

    pub(super) fn require_end(&self, function: &mut Function) {
        self.index.load(function);
        self.length.load(function);
        function.instruction(&Instruction::I64Eq);
        self.require(function);
    }

    pub(super) fn release(self, builder: &mut FunctionBuilder<'_>, function: &mut Function) {
        self.units.clear(function);
        for local in [self.valid, self.byte, self.index, self.length] {
            builder.runtime_schema().release_i64_local(local, function);
        }
    }
}
