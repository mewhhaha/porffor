//! Private native-provider bytes. No JavaScript reference crosses the host ABI.
use super::super::*;
use crate::gc_types::*;
use lila_intl::{IntlHostOp, INTL_GC_REQUEST_PREFIX_BYTES};

fn store_i32(value: i32, out: I32Local, function: &mut Function) {
    function.instruction(&Instruction::I32Const(value));
    out.store(function);
}
fn trap_if(function: &mut Function) {
    function.instruction(&Instruction::If(BlockType::Empty));
    function.instruction(&Instruction::Unreachable);
    function.instruction(&Instruction::End);
}

/// A private, exact-length byte message. Growth copies only byte data into a
/// new rooted array; publication consumes the mutable construction owner.
pub(crate) struct IntlByteArrayBuilder {
    bytes: GcLocal<ByteArray>,
    length: I32Local,
}
impl IntlByteArrayBuilder {
    pub(crate) fn new(schema: &RuntimeSchema, function: &mut Function) -> Self {
        let length = schema.reserve_i32_local(function);
        store_i32(0, length, function);
        let bytes = schema
            .reserve_gc_local::<ByteArray, NonNullable>(function)
            .initialize(
                schema
                    .array_type::<ByteArray>()
                    .filled(GcOperand::i32(0), length, function),
                function,
            );
        Self { bytes, length }
    }
    pub(crate) fn with_operation(
        operation: IntlHostOp,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) -> Self {
        const _: () = assert!(INTL_GC_REQUEST_PREFIX_BYTES == core::mem::size_of::<u64>());
        let message = Self::new(schema, function);
        message.append_u64_constant(operation.wire() as u64, schema, function);
        message
    }
    /// The widened addition precedes allocation or any array index. Messages
    /// outside the selected runtime's signed array extent are resource traps.
    fn grow(&self, amount: I64Local, schema: &RuntimeSchema, function: &mut Function) -> I32Local {
        let start = schema.reserve_i32_local(function);
        let next_length = schema.reserve_i32_local(function);
        let wide = schema.reserve_i64_local(function);
        self.length.load(function);
        start.store(function);
        self.length.load(function);
        function.instruction(&Instruction::I64ExtendI32U);
        amount.load(function);
        function.instruction(&Instruction::I64Add);
        wide.store(function);
        wide.load(function);
        function.instruction(&Instruction::I64Const(i32::MAX as i64));
        function.instruction(&Instruction::I64GtU);
        trap_if(function);
        wide.load(function);
        function.instruction(&Instruction::I32WrapI64);
        next_length.store(function);
        let next = schema
            .reserve_gc_local::<ByteArray, NonNullable>(function)
            .initialize(
                schema
                    .array_type::<ByteArray>()
                    .filled(GcOperand::i32(0), next_length, function),
                function,
            );
        let index = schema.reserve_i32_local(function);
        let byte = schema.reserve_i32_local(function);
        store_i32(0, index, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        index.load(function);
        self.length.load(function);
        function.instruction(&Instruction::I32GeU);
        function.instruction(&Instruction::BrIf(1));
        schema
            .array_type::<ByteArray>()
            .read(&self.bytes, index, schema, function)
            .store(byte, function);
        schema.array_type::<ByteArray>().write(
            &next,
            index,
            GcOperand::i32_local(byte),
            schema,
            function,
        );
        index.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        index.store(function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.bytes.replace(next.load(schema, function), function);
        next_length.load(function);
        self.length.store(function);
        schema.release_i32_local(byte, function);
        schema.release_i32_local(index, function);
        next.clear(function);
        schema.release_i64_local(wide, function);
        schema.release_i32_local(next_length, function);
        start
    }
    pub(crate) fn append_u64(
        &self,
        value: I64Local,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) {
        let amount = schema.reserve_i64_local(function);
        function.instruction(&Instruction::I64Const(8));
        amount.store(function);
        let start = self.grow(amount, schema, function);
        let index = schema.reserve_i32_local(function);
        let byte = schema.reserve_i32_local(function);
        for offset in 0..8 {
            start.load(function);
            function.instruction(&Instruction::I32Const(offset));
            function.instruction(&Instruction::I32Add);
            index.store(function);
            value.load(function);
            function.instruction(&Instruction::I64Const(offset as i64 * 8));
            function.instruction(&Instruction::I64ShrU);
            function.instruction(&Instruction::I32WrapI64);
            byte.store(function);
            schema.array_type::<ByteArray>().write(
                &self.bytes,
                index,
                GcOperand::i32_local(byte),
                schema,
                function,
            );
        }
        schema.release_i32_local(byte, function);
        schema.release_i32_local(index, function);
        schema.release_i32_local(start, function);
        schema.release_i64_local(amount, function);
    }
    pub(crate) fn append_u64_constant(
        &self,
        value: u64,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) {
        let word = schema.reserve_i64_local(function);
        function.instruction(&Instruction::I64Const(value as i64));
        word.store(function);
        self.append_u64(word, schema, function);
        schema.release_i64_local(word, function);
    }
    /// Opaque selected provider plans remain bytes, with no JavaScript String encoding.
    pub(crate) fn append_immutable_bytes(
        &self,
        bytes: &GcLocal<ImmutableByteArray>,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) {
        let count = schema.reserve_i32_local(function);
        schema
            .array_type::<ImmutableByteArray>()
            .length(bytes, schema, function);
        count.store(function);
        let width = schema.reserve_i64_local(function);
        count.load(function);
        function.instruction(&Instruction::I64ExtendI32U);
        width.store(function);
        self.append_u64(width, schema, function);
        let start = self.grow(width, schema, function);
        let index = schema.reserve_i32_local(function);
        let destination = schema.reserve_i32_local(function);
        let byte = schema.reserve_i32_local(function);
        store_i32(0, index, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        index.load(function);
        count.load(function);
        function.instruction(&Instruction::I32GeU);
        function.instruction(&Instruction::BrIf(1));
        schema
            .array_type::<ImmutableByteArray>()
            .read(bytes, index, schema, function)
            .store(byte, function);
        start.load(function);
        index.load(function);
        function.instruction(&Instruction::I32Add);
        destination.store(function);
        schema.array_type::<ByteArray>().write(
            &self.bytes,
            destination,
            GcOperand::i32_local(byte),
            schema,
            function,
        );
        index.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        index.store(function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        schema.release_i32_local(byte, function);
        schema.release_i32_local(destination, function);
        schema.release_i32_local(index, function);
        schema.release_i32_local(start, function);
        schema.release_i64_local(width, function);
        schema.release_i32_local(count, function);
    }
    pub(crate) fn append_utf16(
        &self,
        text: &GcLocal<StringValue>,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) {
        let units = schema
            .reserve_gc_local::<CodeUnitArray, NonNullable>(function)
            .initialize(
                schema
                    .struct_type::<StringValue>()
                    .field(StringValueSchema::CODE_UNITS)
                    .read(text, schema, function)
                    .reference(),
                function,
            );
        let count = schema.reserve_i32_local(function);
        let size = schema.reserve_i64_local(function);
        schema
            .array_type::<CodeUnitArray>()
            .length(&units, schema, function);
        count.store(function);
        count.load(function);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Mul);
        size.store(function);
        self.append_u64(size, schema, function);
        let start = self.grow(size, schema, function);
        let index = schema.reserve_i32_local(function);
        let position = schema.reserve_i32_local(function);
        let unit = schema.reserve_i32_local(function);
        let byte = schema.reserve_i32_local(function);
        store_i32(0, index, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        index.load(function);
        count.load(function);
        function.instruction(&Instruction::I32GeU);
        function.instruction(&Instruction::BrIf(1));
        schema
            .array_type::<CodeUnitArray>()
            .read(&units, index, schema, function)
            .store(unit, function);
        for offset in 0..2 {
            start.load(function);
            index.load(function);
            function.instruction(&Instruction::I32Const(2));
            function.instruction(&Instruction::I32Mul);
            function.instruction(&Instruction::I32Add);
            function.instruction(&Instruction::I32Const(offset));
            function.instruction(&Instruction::I32Add);
            position.store(function);
            unit.load(function);
            function.instruction(&Instruction::I32Const(offset * 8));
            function.instruction(&Instruction::I32ShrU);
            byte.store(function);
            schema.array_type::<ByteArray>().write(
                &self.bytes,
                position,
                GcOperand::i32_local(byte),
                schema,
                function,
            );
        }
        index.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        index.store(function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for local in [byte, unit, position, index, start] {
            schema.release_i32_local(local, function);
        }
        schema.release_i64_local(size, function);
        schema.release_i32_local(count, function);
        units.clear(function);
    }
    pub(crate) fn append_utf8(
        &self,
        text: &GcLocal<StringValue>,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) {
        self.append_utf8_inner(text, true, schema, function);
    }
    pub(crate) fn append_remaining_utf8(
        &self,
        text: &GcLocal<StringValue>,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) {
        self.append_utf8_inner(text, false, schema, function);
    }
    fn append_utf8_inner(
        &self,
        text: &GcLocal<StringValue>,
        prefixed: bool,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) {
        let units = schema
            .reserve_gc_local::<CodeUnitArray, NonNullable>(function)
            .initialize(
                schema
                    .struct_type::<StringValue>()
                    .field(StringValueSchema::CODE_UNITS)
                    .read(text, schema, function)
                    .reference(),
                function,
            );
        let count = schema.reserve_i32_local(function);
        let index = schema.reserve_i32_local(function);
        let point = schema.reserve_i32_local(function);
        let size = schema.reserve_i64_local(function);
        schema
            .array_type::<CodeUnitArray>()
            .length(&units, schema, function);
        count.store(function);
        store_i32(0, index, function);
        function.instruction(&Instruction::I64Const(0));
        size.store(function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        index.load(function);
        count.load(function);
        function.instruction(&Instruction::I32GeU);
        function.instruction(&Instruction::BrIf(1));
        emit_utf16_point(&units, count, index, point, schema, function);
        size.load(function);
        for (limit, width) in [(0x80, 1), (0x800, 2), (0x10000, 3)] {
            point.load(function);
            function.instruction(&Instruction::I32Const(limit));
            function.instruction(&Instruction::I32LtU);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            function.instruction(&Instruction::I64Const(width));
            function.instruction(&Instruction::Else);
        }
        function.instruction(&Instruction::I64Const(4));
        for _ in 0..3 {
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::I64Add);
        size.store(function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        if prefixed {
            self.append_u64(size, schema, function);
        }
        let position = self.grow(size, schema, function);
        let byte = schema.reserve_i32_local(function);
        store_i32(0, index, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        index.load(function);
        count.load(function);
        function.instruction(&Instruction::I32GeU);
        function.instruction(&Instruction::BrIf(1));
        emit_utf16_point(&units, count, index, point, schema, function);
        for (limit, width) in [(0x80, 1), (0x800, 2), (0x10000, 3)] {
            point.load(function);
            function.instruction(&Instruction::I32Const(limit));
            function.instruction(&Instruction::I32LtU);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.write_utf8_point(point, width, position, byte, schema, function);
            function.instruction(&Instruction::Else);
        }
        self.write_utf8_point(point, 4, position, byte, schema, function);
        for _ in 0..3 {
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        schema.release_i32_local(byte, function);
        schema.release_i32_local(position, function);
        schema.release_i64_local(size, function);
        schema.release_i32_local(point, function);
        schema.release_i32_local(index, function);
        schema.release_i32_local(count, function);
        units.clear(function);
    }
    fn write_utf8_point(
        &self,
        point: I32Local,
        width: u32,
        position: I32Local,
        byte: I32Local,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) {
        for offset in 0..width {
            point.load(function);
            let shift = (width - offset - 1) * 6;
            if shift != 0 {
                function.instruction(&Instruction::I32Const(shift as i32));
                function.instruction(&Instruction::I32ShrU);
            }
            if width != 1 {
                if offset == 0 {
                    function.instruction(&Instruction::I32Const(match width {
                        2 => 0xc0,
                        3 => 0xe0,
                        4 => 0xf0,
                        _ => unreachable!(),
                    }));
                } else {
                    function.instruction(&Instruction::I32Const(0x3f));
                    function.instruction(&Instruction::I32And);
                    function.instruction(&Instruction::I32Const(0x80));
                }
                function.instruction(&Instruction::I32Or);
            }
            byte.store(function);
            schema.array_type::<ByteArray>().write(
                &self.bytes,
                position,
                GcOperand::i32_local(byte),
                schema,
                function,
            );
            position.load(function);
            function.instruction(&Instruction::I32Const(1));
            function.instruction(&Instruction::I32Add);
            position.store(function);
        }
    }
    pub(crate) fn finish(
        self,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) -> GcLocal<ByteArray> {
        schema.release_i32_local(self.length, function);
        self.bytes
    }
}

/// Advance one UTF-16 character. UTF-8 wire text uses replacement for a lone
/// surrogate; mathematical String input instead uses the exact UTF-16 codec.
fn emit_utf16_point(
    units: &GcLocal<CodeUnitArray>,
    count: I32Local,
    index: I32Local,
    point: I32Local,
    schema: &RuntimeSchema,
    function: &mut Function,
) {
    schema
        .array_type::<CodeUnitArray>()
        .read(units, index, schema, function)
        .store(point, function);
    index.load(function);
    function.instruction(&Instruction::I32Const(1));
    function.instruction(&Instruction::I32Add);
    index.store(function);
    point.load(function);
    function.instruction(&Instruction::I32Const(0xd800));
    function.instruction(&Instruction::I32Sub);
    function.instruction(&Instruction::I32Const(0x7ff));
    function.instruction(&Instruction::I32LeU);
    function.instruction(&Instruction::If(BlockType::Empty));
    let next = schema.reserve_i32_local(function);
    let paired = schema.reserve_i32_local(function);
    store_i32(0, paired, function);
    point.load(function);
    function.instruction(&Instruction::I32Const(0xdc00));
    function.instruction(&Instruction::I32LtU);
    index.load(function);
    count.load(function);
    function.instruction(&Instruction::I32LtU);
    function.instruction(&Instruction::I32And);
    function.instruction(&Instruction::If(BlockType::Empty));
    schema
        .array_type::<CodeUnitArray>()
        .read(units, index, schema, function)
        .store(next, function);
    next.load(function);
    function.instruction(&Instruction::I32Const(0xdc00));
    function.instruction(&Instruction::I32Sub);
    function.instruction(&Instruction::I32Const(0x3ff));
    function.instruction(&Instruction::I32LeU);
    function.instruction(&Instruction::If(BlockType::Empty));
    point.load(function);
    function.instruction(&Instruction::I32Const(0xd800));
    function.instruction(&Instruction::I32Sub);
    function.instruction(&Instruction::I32Const(10));
    function.instruction(&Instruction::I32Shl);
    next.load(function);
    function.instruction(&Instruction::I32Const(0xdc00));
    function.instruction(&Instruction::I32Sub);
    function.instruction(&Instruction::I32Add);
    function.instruction(&Instruction::I32Const(0x10000));
    function.instruction(&Instruction::I32Add);
    point.store(function);
    index.load(function);
    function.instruction(&Instruction::I32Const(1));
    function.instruction(&Instruction::I32Add);
    index.store(function);
    store_i32(1, paired, function);
    function.instruction(&Instruction::End);
    function.instruction(&Instruction::End);
    paired.load(function);
    function.instruction(&Instruction::I32Eqz);
    function.instruction(&Instruction::If(BlockType::Empty));
    store_i32(0xfffd, point, function);
    function.instruction(&Instruction::End);
    schema.release_i32_local(paired, function);
    schema.release_i32_local(next, function);
    function.instruction(&Instruction::End);
}

/// A bounded cursor over a rooted response. Every read validates the complete
/// span before touching bytes; UTF-8 rejects malformed host responses.
pub(crate) struct IntlByteArrayReader<'a> {
    bytes: &'a GcLocal<ByteArray>,
    cursor: I32Local,
    length: I32Local,
}
impl<'a> IntlByteArrayReader<'a> {
    pub(crate) fn new(
        bytes: &'a GcLocal<ByteArray>,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) -> Self {
        let cursor = schema.reserve_i32_local(function);
        let length = schema.reserve_i32_local(function);
        store_i32(0, cursor, function);
        schema
            .array_type::<ByteArray>()
            .length(bytes, schema, function);
        length.store(function);
        length.load(function);
        function.instruction(&Instruction::I32Const(i32::MAX));
        function.instruction(&Instruction::I32GtU);
        trap_if(function);
        Self {
            bytes,
            cursor,
            length,
        }
    }
    fn require(&self, width: I64Local, function: &mut Function) {
        width.load(function);
        self.length.load(function);
        function.instruction(&Instruction::I64ExtendI32U);
        self.cursor.load(function);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64GtU);
        trap_if(function);
    }
    pub(crate) fn read_u8(
        &self,
        output: I32Local,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) {
        self.cursor.load(function);
        self.length.load(function);
        function.instruction(&Instruction::I32GeU);
        trap_if(function);
        schema
            .array_type::<ByteArray>()
            .read(self.bytes, self.cursor, schema, function)
            .store(output, function);
        self.cursor.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        self.cursor.store(function);
    }
    pub(crate) fn read_u64(
        &self,
        output: I64Local,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) {
        let width = schema.reserve_i64_local(function);
        function.instruction(&Instruction::I64Const(8));
        width.store(function);
        self.require(width, function);
        let byte = schema.reserve_i32_local(function);
        function.instruction(&Instruction::I64Const(0));
        output.store(function);
        for offset in 0..8 {
            self.read_u8(byte, schema, function);
            output.load(function);
            byte.load(function);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::I64Const(offset * 8));
            function.instruction(&Instruction::I64Shl);
            function.instruction(&Instruction::I64Or);
            output.store(function);
        }
        schema.release_i32_local(byte, function);
        schema.release_i64_local(width, function);
    }
    pub(crate) fn read_utf16(
        &self,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) -> GcLocal<StringValue> {
        let width = schema.reserve_i64_local(function);
        self.read_u64(width, schema, function);
        self.require(width, function);
        width.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        trap_if(function);
        let count = schema.reserve_i32_local(function);
        width.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I32WrapI64);
        count.store(function);
        let string = StringConstruction::allocate(
            schema,
            schema.reserve_gc_local::<CodeUnitArray, NonNullable>(function),
            count,
            function,
        );
        let index = schema.reserve_i32_local(function);
        let low = schema.reserve_i32_local(function);
        let high = schema.reserve_i32_local(function);
        store_i32(0, index, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        index.load(function);
        count.load(function);
        function.instruction(&Instruction::I32GeU);
        function.instruction(&Instruction::BrIf(1));
        self.read_u8(low, schema, function);
        self.read_u8(high, schema, function);
        low.load(function);
        high.load(function);
        function.instruction(&Instruction::I32Const(8));
        function.instruction(&Instruction::I32Shl);
        function.instruction(&Instruction::I32Or);
        low.store(function);
        string.write(index, low, schema, function);
        index.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        index.store(function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        schema.release_i32_local(high, function);
        schema.release_i32_local(low, function);
        schema.release_i32_local(index, function);
        let output = schema
            .reserve_gc_local::<StringValue, NonNullable>(function)
            .initialize(string.publish(schema, function), function);
        schema.release_i32_local(count, function);
        schema.release_i64_local(width, function);
        output
    }
    pub(crate) fn read_utf8(
        &self,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) -> GcLocal<StringValue> {
        let width = schema.reserve_i64_local(function);
        self.read_u64(width, schema, function);
        self.require(width, function);
        let end = schema.reserve_i32_local(function);
        self.cursor.load(function);
        width.load(function);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Add);
        end.store(function);
        let result = self.read_utf8_to(end, schema, function);
        schema.release_i32_local(end, function);
        schema.release_i64_local(width, function);
        result
    }
    pub(crate) fn consume_remaining_utf8(
        &self,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) -> GcLocal<StringValue> {
        self.read_utf8_to(self.length, schema, function)
    }
    fn read_utf8_to(
        &self,
        end: I32Local,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) -> GcLocal<StringValue> {
        let start = schema.reserve_i32_local(function);
        let count = schema.reserve_i32_local(function);
        let point = schema.reserve_i32_local(function);
        self.cursor.load(function);
        start.store(function);
        store_i32(0, count, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.cursor.load(function);
        end.load(function);
        function.instruction(&Instruction::I32GeU);
        function.instruction(&Instruction::BrIf(1));
        self.read_utf8_point(end, point, schema, function);
        count.load(function);
        function.instruction(&Instruction::I32Const(1));
        point.load(function);
        function.instruction(&Instruction::I32Const(0xffff));
        function.instruction(&Instruction::I32GtU);
        function.instruction(&Instruction::I32Add);
        function.instruction(&Instruction::I32Add);
        count.store(function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        let string = StringConstruction::allocate(
            schema,
            schema.reserve_gc_local::<CodeUnitArray, NonNullable>(function),
            count,
            function,
        );
        start.load(function);
        self.cursor.store(function);
        store_i32(0, count, function);
        let unit = schema.reserve_i32_local(function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.cursor.load(function);
        end.load(function);
        function.instruction(&Instruction::I32GeU);
        function.instruction(&Instruction::BrIf(1));
        self.read_utf8_point(end, point, schema, function);
        point.load(function);
        function.instruction(&Instruction::I32Const(0xffff));
        function.instruction(&Instruction::I32GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        point.load(function);
        function.instruction(&Instruction::I32Const(0x10000));
        function.instruction(&Instruction::I32Sub);
        function.instruction(&Instruction::I32Const(10));
        function.instruction(&Instruction::I32ShrU);
        function.instruction(&Instruction::I32Const(0xd800));
        function.instruction(&Instruction::I32Add);
        unit.store(function);
        string.write(count, unit, schema, function);
        count.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        count.store(function);
        point.load(function);
        function.instruction(&Instruction::I32Const(0x3ff));
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Const(0xdc00));
        function.instruction(&Instruction::I32Add);
        unit.store(function);
        function.instruction(&Instruction::Else);
        point.load(function);
        unit.store(function);
        function.instruction(&Instruction::End);
        string.write(count, unit, schema, function);
        count.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        count.store(function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        schema.release_i32_local(unit, function);
        let output = schema
            .reserve_gc_local::<StringValue, NonNullable>(function)
            .initialize(string.publish(schema, function), function);
        schema.release_i32_local(point, function);
        schema.release_i32_local(count, function);
        schema.release_i32_local(start, function);
        output
    }
    fn read_utf8_point(
        &self,
        end: I32Local,
        point: I32Local,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) {
        let first = schema.reserve_i32_local(function);
        let width = schema.reserve_i32_local(function);
        let byte = schema.reserve_i32_local(function);
        self.read_u8(first, schema, function);
        store_i32(0, width, function);
        first.load(function);
        function.instruction(&Instruction::I32Const(0x80));
        function.instruction(&Instruction::I32LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        first.load(function);
        point.store(function);
        function.instruction(&Instruction::Else);
        for (low, high, count, mask) in [
            (0xc2, 0xdf, 1, 0x1f),
            (0xe0, 0xef, 2, 0x0f),
            (0xf0, 0xf4, 3, 0x07),
        ] {
            first.load(function);
            function.instruction(&Instruction::I32Const(low));
            function.instruction(&Instruction::I32GeU);
            first.load(function);
            function.instruction(&Instruction::I32Const(high));
            function.instruction(&Instruction::I32LeU);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::If(BlockType::Empty));
            store_i32(count, width, function);
            first.load(function);
            function.instruction(&Instruction::I32Const(mask));
            function.instruction(&Instruction::I32And);
            point.store(function);
            function.instruction(&Instruction::End);
        }
        width.load(function);
        function.instruction(&Instruction::I32Eqz);
        trap_if(function);
        self.cursor.load(function);
        width.load(function);
        function.instruction(&Instruction::I32Add);
        end.load(function);
        function.instruction(&Instruction::I32GtU);
        trap_if(function);
        for offset in 0..3 {
            width.load(function);
            function.instruction(&Instruction::I32Const(offset));
            function.instruction(&Instruction::I32GtU);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.read_u8(byte, schema, function);
            byte.load(function);
            function.instruction(&Instruction::I32Const(0xc0));
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::I32Const(0x80));
            function.instruction(&Instruction::I32Ne);
            trap_if(function);
            point.load(function);
            function.instruction(&Instruction::I32Const(6));
            function.instruction(&Instruction::I32Shl);
            byte.load(function);
            function.instruction(&Instruction::I32Const(0x3f));
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::I32Or);
            point.store(function);
            function.instruction(&Instruction::End);
        }
        for (count, minimum) in [(1, 0x80), (2, 0x800), (3, 0x10000)] {
            width.load(function);
            function.instruction(&Instruction::I32Const(count));
            function.instruction(&Instruction::I32Eq);
            point.load(function);
            function.instruction(&Instruction::I32Const(minimum));
            function.instruction(&Instruction::I32LtU);
            function.instruction(&Instruction::I32And);
            trap_if(function);
        }
        point.load(function);
        function.instruction(&Instruction::I32Const(0x10ffff));
        function.instruction(&Instruction::I32GtU);
        point.load(function);
        function.instruction(&Instruction::I32Const(0xd800));
        function.instruction(&Instruction::I32Sub);
        function.instruction(&Instruction::I32Const(0x7ff));
        function.instruction(&Instruction::I32LeU);
        function.instruction(&Instruction::I32Or);
        trap_if(function);
        function.instruction(&Instruction::End);
        schema.release_i32_local(byte, function);
        schema.release_i32_local(width, function);
        schema.release_i32_local(first, function);
    }
    pub(crate) fn finish(self, schema: &RuntimeSchema, function: &mut Function) {
        self.cursor.load(function);
        self.length.load(function);
        function.instruction(&Instruction::I32Ne);
        trap_if(function);
        schema.release_i32_local(self.length, function);
        schema.release_i32_local(self.cursor, function);
    }
}

impl FunctionBuilder<'_> {
    pub(crate) fn emit_intl_provider_byte_call(
        &self,
        request: &GcLocal<ByteArray>,
        function: &mut Function,
    ) -> Result<GcLocal<ByteArray, Nullable>, EmitError> {
        let import = self
            .functions
            .gc_host_imports()
            .get(GcHostImport::IntlProviderCall)
            .ok_or_else(|| EmitError::unsupported("missing checked Intl GC host import"))?;
        let schema = self.runtime_schema();
        let output = import.call_intl_provider(request, schema, function)?;
        Ok(schema
            .reserve_gc_local::<ByteArray, Nullable>(function)
            .initialize(output, function))
    }
}

use super::intl::CanonicalLocaleListLocals;
use super::intl_number::IntlMathematicalValueLocals;
use lila_intl::number_format::options::LocaleMatcher;
use lila_intl::{NumberConfigurationWord, NumberNumericKind, NUMBER_WIRE_VERSION};

/// Each variant owns its complete source-ready operation and field order.
pub(in crate::builtins) enum IntlNumberProviderRequest<'a> {
    Resolve {
        locales: &'a CanonicalLocaleListLocals,
        matcher: &'a GcI32DomainLocal<LocaleMatcher>,
        numbering_system: &'a GcLocal<StringValue>,
    },
    Supported {
        locales: &'a CanonicalLocaleListLocals,
        matcher: &'a GcI32DomainLocal<LocaleMatcher>,
    },
    Scalar {
        record: &'a GcLocal<IntlNumberFormatObject>,
        input: &'a IntlMathematicalValueLocals,
    },
    Range {
        record: &'a GcLocal<IntlNumberFormatObject>,
        start: &'a IntlMathematicalValueLocals,
        end: &'a IntlMathematicalValueLocals,
    },
}
pub(in crate::builtins) struct IntlNumberProviderResponse {
    bytes: GcLocal<ByteArray>,
    operation: IntlHostOp,
}
impl IntlNumberProviderResponse {
    pub(in crate::builtins) fn reader(
        &self,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) -> IntlByteArrayReader<'_> {
        let reader = IntlByteArrayReader::new(&self.bytes, schema, function);
        let word = schema.reserve_i64_local(function);
        for expected in [
            NUMBER_WIRE_VERSION,
            u64::from(self.operation.code()) * 2 + 1,
        ] {
            reader.read_u64(word, schema, function);
            word.load(function);
            function.instruction(&Instruction::I64Const(expected as i64));
            function.instruction(&Instruction::I64Ne);
            trap_if(function);
        }
        schema.release_i64_local(word, function);
        reader
    }
    pub(in crate::builtins) fn clear(self, function: &mut Function) {
        self.bytes.clear(function);
    }
}
impl IntlByteArrayReader<'_> {
    pub(crate) fn require_records(
        &self,
        count: I64Local,
        minimum_bytes: u64,
        function: &mut Function,
    ) {
        assert!(minimum_bytes != 0);
        count.load(function);
        self.length.load(function);
        self.cursor.load(function);
        function.instruction(&Instruction::I32Sub);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(minimum_bytes as i64));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::I64GtU);
        trap_if(function);
    }
}
impl FunctionBuilder<'_> {
    pub(in crate::builtins) fn emit_intl_number_provider_call(
        &mut self,
        request: IntlNumberProviderRequest<'_>,
        function: &mut Function,
    ) -> Result<IntlNumberProviderResponse, EmitError> {
        let operation = match &request {
            IntlNumberProviderRequest::Resolve { .. } => IntlHostOp::ResolveNumberLocale,
            IntlNumberProviderRequest::Supported { .. } => IntlHostOp::SupportedNumberLocales,
            IntlNumberProviderRequest::Scalar { .. } => IntlHostOp::FormatNumberParts,
            IntlNumberProviderRequest::Range { .. } => IntlHostOp::FormatNumberRangeParts,
        };
        let schema = self.runtime_schema();
        let message = IntlByteArrayBuilder::with_operation(operation, schema, function);
        message.append_u64_constant(NUMBER_WIRE_VERSION, schema, function);
        message.append_u64_constant(u64::from(operation.code()) * 2, schema, function);
        match request {
            IntlNumberProviderRequest::Resolve {
                locales,
                matcher,
                numbering_system,
            } => {
                append_domain(&message, matcher, schema, function);
                message.append_utf8(numbering_system, schema, function);
                self.emit_intl_wire_canonical_locales(&message, locales, function)?;
            }
            IntlNumberProviderRequest::Supported { locales, matcher } => {
                append_domain(&message, matcher, schema, function);
                self.emit_intl_wire_canonical_locales(&message, locales, function)?;
            }
            IntlNumberProviderRequest::Scalar { record, input } => {
                self.emit_intl_number_wire_configuration(&message, record, function);
                append_intl_mathematical_value(&message, input, schema, function);
            }
            IntlNumberProviderRequest::Range { record, start, end } => {
                self.emit_intl_number_wire_configuration(&message, record, function);
                append_intl_mathematical_value(&message, start, schema, function);
                append_intl_mathematical_value(&message, end, schema, function);
            }
        }
        let message = message.finish(schema, function);
        let reply = self.emit_intl_provider_byte_call(&message, function)?;
        reply.load(schema, function).is_null(function);
        self.open_frame(ControlFrameKind::If, function);
        if operation == IntlHostOp::FormatNumberRangeParts {
            self.emit_intl_number_range_error(
                RuntimeErrorMessage::INTL_NUMBERFORMAT_RANGE_ENDPOINTS_MUST_NOT_BE_NAN,
                function,
            )?;
        } else {
            function.instruction(&Instruction::Unreachable);
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let bytes = schema.reserve_gc_local(function).initialize(
            reply.load(schema, function).require_non_null(function),
            function,
        );
        reply.clear(function);
        message.clear(function);
        Ok(IntlNumberProviderResponse { bytes, operation })
    }
    pub(in crate::builtins) fn emit_intl_wire_canonical_locales(
        &mut self,
        message: &IntlByteArrayBuilder,
        locales: &CanonicalLocaleListLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let count = schema.reserve_i32_local(function);
        let index = schema.reserve_i32_local(function);
        let wide = schema.reserve_i64_local(function);
        schema
            .array_type::<ValueArray>()
            .length(locales.values(), schema, function);
        count.store(function);
        count.load(function);
        function.instruction(&Instruction::I64ExtendI32U);
        wide.store(function);
        message.append_u64(wide, schema, function);
        store_i32(0, index, function);
        let done = self.open_frame(ControlFrameKind::Block, function);
        let next = self.open_frame(ControlFrameKind::Loop, function);
        index.load(function);
        count.load(function);
        function.instruction(&Instruction::I32GeU);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_branch_to_target(done, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let value = schema.reserve_value_local(function);
        self.emit_argument_vector_entry_to_value(locales.values(), index, &value, function);
        let text = schema.reserve_gc_local(function).initialize(
            value.cast_reference::<StringValue>(schema, function),
            function,
        );
        message.append_utf8(&text, schema, function);
        text.clear(function);
        value.clear(function);
        index.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        index.store(function);
        self.emit_branch_to_target(next, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        schema.release_i64_local(wide, function);
        schema.release_i32_local(index, function);
        schema.release_i32_local(count, function);
        Ok(())
    }
    fn emit_intl_number_wire_configuration(
        &self,
        message: &IntlByteArrayBuilder,
        record: &GcLocal<IntlNumberFormatObject>,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let nf = schema.struct_type::<IntlNumberFormatObject>();
        for field in [
            IntlNumberFormatObjectSchema::LOCALE,
            IntlNumberFormatObjectSchema::DATA_LOCALE,
            IntlNumberFormatObjectSchema::NUMBERING_SYSTEM,
        ] {
            let text = schema.reserve_gc_local(function).initialize(
                nf.field(field).read(record, schema, function).reference(),
                function,
            );
            message.append_utf8(&text, schema, function);
            text.clear(function);
        }
        let rounding = schema.reserve_gc_local(function).initialize(
            nf.field(IntlNumberFormatObjectSchema::ROUNDING)
                .read(record, schema, function)
                .reference(),
            function,
        );
        let rules = schema.struct_type::<IntlNumberRounding>();
        let code = schema.reserve_i32_local(function);
        let wide = schema.reserve_i64_local(function);
        for word in NumberConfigurationWord::ALL {
            use NumberConfigurationWord as W;
            match word {
                W::Style => nf
                    .field(IntlNumberFormatObjectSchema::STYLE)
                    .read(record, schema, function)
                    .store(code, function),
                W::CurrencyDisplay => nf
                    .field(IntlNumberFormatObjectSchema::CURRENCY_DISPLAY)
                    .read(record, schema, function)
                    .store(code, function),
                W::CurrencySign => nf
                    .field(IntlNumberFormatObjectSchema::CURRENCY_SIGN)
                    .read(record, schema, function)
                    .store(code, function),
                W::UnitDisplay => nf
                    .field(IntlNumberFormatObjectSchema::UNIT_DISPLAY)
                    .read(record, schema, function)
                    .store(code, function),
                W::Notation => nf
                    .field(IntlNumberFormatObjectSchema::NOTATION)
                    .read(record, schema, function)
                    .store(code, function),
                W::CompactDisplay => nf
                    .field(IntlNumberFormatObjectSchema::COMPACT_DISPLAY)
                    .read(record, schema, function)
                    .store(code, function),
                W::MinimumInteger => rules
                    .field(IntlNumberRoundingSchema::MINIMUM_INTEGER)
                    .read(&rounding, schema, function)
                    .store(code, function),
                W::Precision => rules
                    .field(IntlNumberRoundingSchema::PRECISION)
                    .read(&rounding, schema, function)
                    .store(code, function),
                W::MinimumFraction => rules
                    .field(IntlNumberRoundingSchema::MINIMUM_FRACTION)
                    .read(&rounding, schema, function)
                    .store(code, function),
                W::MaximumFraction => rules
                    .field(IntlNumberRoundingSchema::MAXIMUM_FRACTION)
                    .read(&rounding, schema, function)
                    .store(code, function),
                W::MinimumSignificant => rules
                    .field(IntlNumberRoundingSchema::MINIMUM_SIGNIFICANT)
                    .read(&rounding, schema, function)
                    .store(code, function),
                W::MaximumSignificant => rules
                    .field(IntlNumberRoundingSchema::MAXIMUM_SIGNIFICANT)
                    .read(&rounding, schema, function)
                    .store(code, function),
                W::RoundingIncrement => rules
                    .field(IntlNumberRoundingSchema::ROUNDING_INCREMENT)
                    .read(&rounding, schema, function)
                    .store(code, function),
                W::RoundingMode => rules
                    .field(IntlNumberRoundingSchema::ROUNDING_MODE)
                    .read(&rounding, schema, function)
                    .store(code, function),
                W::TrailingZero => rules
                    .field(IntlNumberRoundingSchema::TRAILING_ZERO)
                    .read(&rounding, schema, function)
                    .store(code, function),
                W::Grouping => nf
                    .field(IntlNumberFormatObjectSchema::GROUPING)
                    .read(record, schema, function)
                    .store(code, function),
                W::SignDisplay => nf
                    .field(IntlNumberFormatObjectSchema::SIGN_DISPLAY)
                    .read(record, schema, function)
                    .store(code, function),
            }
            // Native wire uses zero for inactive fields. GC None remains -1,
            // and an active fraction count zero is preserved independently.
            code.load(function);
            function.instruction(&Instruction::I32Const(0));
            code.load(function);
            function.instruction(&Instruction::I32Const(0));
            function.instruction(&Instruction::I32GeS);
            function.instruction(&Instruction::Select);
            function.instruction(&Instruction::I64ExtendI32U);
            wide.store(function);
            message.append_u64(wide, schema, function);
        }
        let style_text = schema.reserve_gc_local(function).initialize(
            nf.field(IntlNumberFormatObjectSchema::STYLE_TEXT)
                .read(record, schema, function)
                .reference(),
            function,
        );
        message.append_utf8(&style_text, schema, function);
        style_text.clear(function);
        schema.release_i64_local(wide, function);
        schema.release_i32_local(code, function);
        rounding.clear(function);
    }
}
fn append_domain<V: GcI32Constant>(
    message: &IntlByteArrayBuilder,
    domain: &GcI32DomainLocal<V>,
    schema: &RuntimeSchema,
    function: &mut Function,
) {
    let word = schema.reserve_i64_local(function);
    domain.load(function);
    function.instruction(&Instruction::I64ExtendI32U);
    word.store(function);
    message.append_u64(word, schema, function);
    schema.release_i64_local(word, function);
}
pub(in crate::builtins) fn append_intl_mathematical_value(
    message: &IntlByteArrayBuilder,
    input: &IntlMathematicalValueLocals,
    schema: &RuntimeSchema,
    function: &mut Function,
) {
    append_domain(message, input.kind(), schema, function);
    input.kind().load(function);
    function.instruction(&Instruction::I32Const(NumberNumericKind::String.encode()));
    function.instruction(&Instruction::I32Eq);
    function.instruction(&Instruction::If(BlockType::Empty));
    message.append_utf16(input.text(), schema, function);
    function.instruction(&Instruction::Else);
    message.append_utf8(input.text(), schema, function);
    function.instruction(&Instruction::End);
}
