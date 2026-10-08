//! Validated private byte messages. No JavaScript reference enters the provider ABI.
use super::*;
pub(super) struct DtfProviderResponse {
    bytes: GcLocal<ByteArray>,
    operation: IntlHostOp,
}
impl DtfProviderResponse {
    pub(super) fn reader(
        &self,
        schema: &RuntimeSchema,
        f: &mut Function,
    ) -> IntlByteArrayReader<'_> {
        let reader = IntlByteArrayReader::new(&self.bytes, schema, f);
        let word = schema.reserve_i64_local(f);
        for expected in [
            DATE_TIME_WIRE_VERSION,
            2 * u64::from(self.operation.code()) + 1,
        ] {
            reader.read_u64(word, schema, f);
            word.load(f);
            f.instruction(&Instruction::I64Const(expected as i64));
            f.instruction(&Instruction::I64Ne);
            trap_if(f);
        }
        schema.release_i64_local(word, f);
        reader
    }
    pub(super) fn clear(self, f: &mut Function) {
        self.bytes.clear(f);
    }
}
fn trap_if(f: &mut Function) {
    f.instruction(&Instruction::If(BlockType::Empty));
    f.instruction(&Instruction::Unreachable);
    f.instruction(&Instruction::End);
}
pub(super) fn dtf_append_domain<V: GcI32Constant>(
    message: &IntlByteArrayBuilder,
    domain: &GcI32DomainLocal<V>,
    schema: &RuntimeSchema,
    f: &mut Function,
) {
    let word = schema.reserve_i64_local(f);
    domain.load(f);
    f.instruction(&Instruction::I64ExtendI32U);
    word.store(f);
    message.append_u64(word, schema, f);
    schema.release_i64_local(word, f);
}
pub(super) fn dtf_append_optional<V: GcI32Constant>(
    message: &IntlByteArrayBuilder,
    domain: &GcI32DomainLocal<Option<V>>,
    schema: &RuntimeSchema,
    f: &mut Function,
) where
    Option<V>: GcI32Constant,
{
    let word = schema.reserve_i64_local(f);
    emit_domain_is(domain, None, f);
    f.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
    f.instruction(&Instruction::I64Const(0));
    f.instruction(&Instruction::Else);
    domain.load(f);
    f.instruction(&Instruction::I64ExtendI32U);
    f.instruction(&Instruction::End);
    word.store(f);
    message.append_u64(word, schema, f);
    schema.release_i64_local(word, f);
}
impl FunctionBuilder<'_> {
    pub(super) fn emit_dtf_request(
        &self,
        operation: IntlHostOp,
        f: &mut Function,
    ) -> IntlByteArrayBuilder {
        let schema = self.runtime_schema();
        let message = IntlByteArrayBuilder::with_operation(operation, schema, f);
        message.append_u64_constant(DATE_TIME_WIRE_VERSION, schema, f);
        message.append_u64_constant(2 * u64::from(operation.code()), schema, f);
        message
    }
    pub(super) fn emit_dtf_provider_call(
        &mut self,
        operation: IntlHostOp,
        message: IntlByteArrayBuilder,
        f: &mut Function,
    ) -> Result<DtfProviderResponse, EmitError> {
        let schema = self.runtime_schema();
        let request = message.finish(schema, f);
        let reply = self.emit_intl_provider_byte_call(&request, f)?;
        reply.load(schema, f).is_null(f);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_intl_number_type_error(INTL_DTF_EMPTY_TEMPORAL_FORMAT, f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let bytes = schema
            .reserve_gc_local(f)
            .initialize(reply.load(schema, f).require_non_null(f), f);
        reply.clear(f);
        request.clear(f);
        Ok(DtfProviderResponse { bytes, operation })
    }
    pub(super) fn emit_dtf_read_domain<V: GcI32Constant + Copy>(
        &self,
        reader: &IntlByteArrayReader<'_>,
        out: &GcI32DomainLocal<V>,
        variants: impl IntoIterator<Item = (V, u64)>,
        f: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let word = schema.reserve_i64_local(f);
        let found = schema.reserve_i32_local(f);
        reader.read_u64(word, schema, f);
        set_i32(found, 0, f);
        for (variant, code) in variants {
            word.load(f);
            f.instruction(&Instruction::I64Const(code as i64));
            f.instruction(&Instruction::I64Eq);
            f.instruction(&Instruction::If(BlockType::Empty));
            out.set_constant(variant, f);
            set_i32(found, 1, f);
            f.instruction(&Instruction::End);
        }
        found.load(f);
        f.instruction(&Instruction::I32Eqz);
        trap_if(f);
        schema.release_i32_local(found, f);
        schema.release_i64_local(word, f);
    }
    pub(super) fn emit_dtf_read_optional<V: GcI32Constant + Copy>(
        &self,
        reader: &IntlByteArrayReader<'_>,
        out: &GcI32DomainLocal<Option<V>>,
        variants: impl IntoIterator<Item = (V, u64)>,
        f: &mut Function,
    ) where
        Option<V>: GcI32Constant,
    {
        self.emit_dtf_read_domain(
            reader,
            out,
            core::iter::once((None, 0)).chain(variants.into_iter().map(|(v, c)| (Some(v), c))),
            f,
        );
    }
    pub(super) fn emit_dtf_read_plan(
        &self,
        reader: &IntlByteArrayReader<'_>,
        f: &mut Function,
    ) -> GcLocal<ImmutableByteArray> {
        let schema = self.runtime_schema();
        let length = schema.reserve_i64_local(f);
        reader.read_u64(length, schema, f);
        reader.require_records(length, 1, f);
        length.load(f);
        f.instruction(&Instruction::I64Eqz);
        length.load(f);
        f.instruction(&Instruction::I64Const(i32::MAX as i64));
        f.instruction(&Instruction::I64GtU);
        f.instruction(&Instruction::I32Or);
        trap_if(f);
        let count = schema.reserve_i32_local(f);
        length.load(f);
        f.instruction(&Instruction::I32WrapI64);
        count.store(f);
        let construction = ImmutableByteArrayConstruction::allocate(schema, count, f);
        let index = schema.reserve_i32_local(f);
        let byte = schema.reserve_i32_local(f);
        set_i32(index, 0, f);
        f.instruction(&Instruction::Block(BlockType::Empty));
        f.instruction(&Instruction::Loop(BlockType::Empty));
        index.load(f);
        count.load(f);
        f.instruction(&Instruction::I32GeU);
        f.instruction(&Instruction::BrIf(1));
        reader.read_u8(byte, schema, f);
        construction.write(index, byte, schema, f);
        index.load(f);
        f.instruction(&Instruction::I32Const(1));
        f.instruction(&Instruction::I32Add);
        index.store(f);
        f.instruction(&Instruction::Br(0));
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        let plan = schema
            .reserve_gc_local(f)
            .initialize(construction.publish(schema, f), f);
        schema.release_i32_local(byte, f);
        schema.release_i32_local(index, f);
        schema.release_i32_local(count, f);
        schema.release_i64_local(length, f);
        plan
    }
}
