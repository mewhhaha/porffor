use super::*;

/// The only byte-reading authority used by the Date parsers. EOF is a sentinel,
/// never a load from an adjacent string or from beyond linear memory.
pub(super) struct DateParseCursor {
    offset: u32,
    length: u32,
    index: u32,
    byte: u32,
    pub(super) valid: u32,
}

impl DateParseCursor {
    pub(super) fn new(
        builder: &mut FunctionBuilder<'_>,
        source: u32,
        function: &mut Function,
    ) -> Self {
        let cursor = Self {
            offset: builder.reserve_temp_local(),
            length: builder.reserve_temp_local(),
            index: builder.reserve_temp_local(),
            byte: builder.reserve_temp_local(),
            valid: builder.reserve_temp_local(),
        };
        builder.emit_unpack_string_payload(source, cursor.offset, cursor.length, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(cursor.index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(cursor.valid));
        cursor
    }

    fn peek(&self, builder: &FunctionBuilder<'_>, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(self.index));
        function.instruction(&Instruction::LocalGet(self.length));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        builder.emit_load_string_byte(self.offset, self.index, self.byte, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.byte));
        function.instruction(&Instruction::End);
    }

    pub(super) fn at(&self, builder: &FunctionBuilder<'_>, expected: u8, function: &mut Function) {
        self.peek(builder, function);
        function.instruction(&Instruction::LocalGet(self.byte));
        function.instruction(&Instruction::I64Const(expected as i64));
        function.instruction(&Instruction::I64Eq);
    }

    pub(super) fn advance(&self, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(self.index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(self.index));
    }

    /// Consume an i32 predicate without allowing a later success to erase a
    /// previous failure. Every parser starts with a fresh validity local.
    pub(super) fn require(&self, function: &mut Function) {
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalGet(self.valid));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(self.valid));
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
        function.instruction(&Instruction::LocalGet(self.byte));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(self.byte));
        function.instruction(&Instruction::I64Const(b'9' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
    }

    fn append_digit(&self, dest: u32, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(dest));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(self.byte));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(dest));
        self.advance(function);
    }

    pub(super) fn decimal(
        &self,
        builder: &FunctionBuilder<'_>,
        digits: usize,
        dest: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(dest));
        for _ in 0..digits {
            self.peek(builder, function);
            self.digit(function);
            self.require(function);
            self.append_digit(dest, function);
        }
        function.instruction(&Instruction::LocalGet(dest));
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(dest));
    }

    pub(super) fn display_year(
        &self,
        builder: &mut FunctionBuilder<'_>,
        dest: u32,
        function: &mut Function,
    ) {
        let negative = builder.reserve_temp_local();
        let count = builder.reserve_temp_local();
        self.at(builder, b'-', function);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(negative));
        function.instruction(&Instruction::LocalGet(negative));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.advance(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(count));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(dest));
        // The emitted display formats have at least four and at most six year
        // digits throughout the TimeClip range. No unbounded integer parsing.
        for _ in 0..6 {
            self.peek(builder, function);
            self.digit(function);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.append_digit(dest, function);
            function.instruction(&Instruction::LocalGet(count));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(count));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(count));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64GeU);
        self.require(function);
        function.instruction(&Instruction::LocalGet(negative));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(dest));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.require(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(dest));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(dest));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(dest));
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(dest));
        builder.release_temp_local(count);
        builder.release_temp_local(negative);
    }

    pub(super) fn name(
        &self,
        builder: &mut FunctionBuilder<'_>,
        names: &[[u8; 3]],
        dest: u32,
        function: &mut Function,
    ) {
        let packed = builder.reserve_temp_local();
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(packed));
        for _ in 0..3 {
            self.peek(builder, function);
            function.instruction(&Instruction::LocalGet(packed));
            function.instruction(&Instruction::I64Const(256));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::LocalGet(self.byte));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(packed));
            self.advance(function);
        }
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(dest));
        for (index, name) in names.iter().enumerate() {
            let value = ((name[0] as i64) << 16) | ((name[1] as i64) << 8) | name[2] as i64;
            function.instruction(&Instruction::LocalGet(packed));
            function.instruction(&Instruction::I64Const(value));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(index as i64));
            function.instruction(&Instruction::LocalSet(dest));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(dest));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        self.require(function);
        function.instruction(&Instruction::LocalGet(dest));
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(dest));
        builder.release_temp_local(packed);
    }

    pub(super) fn require_end(&self, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(self.index));
        function.instruction(&Instruction::LocalGet(self.length));
        function.instruction(&Instruction::I64Eq);
        self.require(function);
    }

    pub(super) fn release(self, builder: &mut FunctionBuilder<'_>) {
        for local in [self.valid, self.byte, self.index, self.length, self.offset] {
            builder.release_temp_local(local);
        }
    }
}
