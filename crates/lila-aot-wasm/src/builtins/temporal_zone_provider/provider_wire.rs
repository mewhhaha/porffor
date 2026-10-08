//! Closed named-zone requests over the sole rooted GC provider byte transport.
use super::*;
use crate::builtins::intl_provider_wire::IntlByteArrayBuilder;
use lila_intl::IntlHostOp;

#[derive(Clone, Copy)]
pub(super) enum TemporalDataOperation {
    Lookup,
    Offset,
    Inverse,
    Transition,
}
impl TemporalDataOperation {
    fn host_op(self) -> IntlHostOp {
        match self {
            Self::Lookup => IntlHostOp::LookupNamedTimeZone,
            Self::Offset => IntlHostOp::NamedTimeZoneOffset,
            Self::Inverse => IntlHostOp::PossibleNamedTimeZoneEpochs,
            Self::Transition => IntlHostOp::FindNamedTimeZoneTransition,
        }
    }
    fn bounds(self) -> (i64, i64) {
        match self {
            Self::Lookup => (18, 526),
            Self::Offset => (8, 8),
            Self::Inverse => (40, 4_147_192),
            Self::Transition => (16, 16),
        }
    }
}

/// Only checked provider replies or the exact fixed-offset producer mint this
/// owner. Offsets are byte coordinates, never JavaScript reference payloads.
pub(in crate::builtins) struct TemporalDataResponse {
    bytes: GcLocal<ByteArray>,
    length: I64Local,
}
impl TemporalDataResponse {
    fn from_bytes(
        bytes: GcLocal<ByteArray>,
        operation: TemporalDataOperation,
        builder: &FunctionBuilder<'_>,
        f: &mut Function,
    ) -> Self {
        let schema = builder.runtime_schema();
        let length = schema.reserve_i64_local(f);
        schema.array_type::<ByteArray>().length(&bytes, schema, f);
        f.instruction(&Instruction::I64ExtendI32U);
        length.store(f);
        let (minimum, maximum) = operation.bounds();
        length.load(f);
        f.instruction(&Instruction::I64Const(minimum));
        f.instruction(&Instruction::I64LtU);
        length.load(f);
        f.instruction(&Instruction::I64Const(maximum));
        f.instruction(&Instruction::I64GtU);
        f.instruction(&Instruction::I32Or);
        builder.emit_temporal_provider_corruption_if_i32(f);
        Self { bytes, length }
    }
    pub(super) fn length(&self) -> I64Local {
        self.length
    }
    pub(super) fn replace_from(
        &self,
        other: Self,
        builder: &FunctionBuilder<'_>,
        f: &mut Function,
    ) {
        let schema = builder.runtime_schema();
        self.bytes.replace(other.bytes.load(schema, f), f);
        other.length.load(f);
        self.length.store(f);
        other.clear(builder, f);
    }
    fn require_range(
        &self,
        start: I64Local,
        width: I64Local,
        builder: &FunctionBuilder<'_>,
        f: &mut Function,
    ) {
        // Subtraction is guarded before use; an unsigned wrapped sum cannot
        // turn an invalid host range into a valid GC index.
        start.load(f);
        self.length.load(f);
        f.instruction(&Instruction::I64GtU);
        builder.emit_temporal_provider_corruption_if_i32(f);
        width.load(f);
        self.length.load(f);
        start.load(f);
        f.instruction(&Instruction::I64Sub);
        f.instruction(&Instruction::I64GtU);
        builder.emit_temporal_provider_corruption_if_i32(f);
    }
    pub(super) fn read_word(
        &self,
        start: I64Local,
        out: I64Local,
        builder: &FunctionBuilder<'_>,
        f: &mut Function,
    ) {
        let schema = builder.runtime_schema();
        let width = schema.reserve_i64_local(f);
        f.instruction(&Instruction::I64Const(8));
        width.store(f);
        self.require_range(start, width, builder, f);
        let index = schema.reserve_i32_local(f);
        let byte = schema.reserve_i32_local(f);
        f.instruction(&Instruction::I64Const(0));
        out.store(f);
        for offset in 0..8 {
            start.load(f);
            f.instruction(&Instruction::I64Const(offset));
            f.instruction(&Instruction::I64Add);
            f.instruction(&Instruction::I32WrapI64);
            index.store(f);
            schema
                .array_type::<ByteArray>()
                .read(&self.bytes, index, schema, f)
                .store(byte, f);
            out.load(f);
            byte.load(f);
            f.instruction(&Instruction::I64ExtendI32U);
            f.instruction(&Instruction::I64Const(offset * 8));
            f.instruction(&Instruction::I64Shl);
            f.instruction(&Instruction::I64Or);
            out.store(f);
        }
        schema.release_i32_local(byte, f);
        schema.release_i32_local(index, f);
        schema.release_i64_local(width, f);
    }
    pub(super) fn read_word_at(
        &self,
        start: I64Local,
        offset: i64,
        out: I64Local,
        builder: &FunctionBuilder<'_>,
        f: &mut Function,
    ) {
        let coordinate = builder.runtime_schema().reserve_i64_local(f);
        start.load(f);
        f.instruction(&Instruction::I64Const(offset));
        f.instruction(&Instruction::I64Add);
        coordinate.store(f);
        self.read_word(coordinate, out, builder, f);
        builder.runtime_schema().release_i64_local(coordinate, f);
    }
    pub(super) fn read_constant_word(
        &self,
        offset: i64,
        out: I64Local,
        builder: &FunctionBuilder<'_>,
        f: &mut Function,
    ) {
        let coordinate = builder.runtime_schema().reserve_i64_local(f);
        f.instruction(&Instruction::I64Const(offset));
        coordinate.store(f);
        self.read_word(coordinate, out, builder, f);
        builder.runtime_schema().release_i64_local(coordinate, f);
    }
    pub(super) fn read_identifier(
        &self,
        start: I64Local,
        width: I64Local,
        builder: &FunctionBuilder<'_>,
        f: &mut Function,
    ) -> GcLocal<StringValue> {
        let schema = builder.runtime_schema();
        self.require_range(start, width, builder, f);
        width.load(f);
        f.instruction(&Instruction::I64Eqz);
        width.load(f);
        f.instruction(&Instruction::I64Const(
            lila_intl::MAX_TIME_ZONE_IDENTIFIER_BYTES as i64,
        ));
        f.instruction(&Instruction::I64GtU);
        f.instruction(&Instruction::I32Or);
        builder.emit_temporal_provider_corruption_if_i32(f);
        let count = schema.reserve_i32_local(f);
        width.load(f);
        f.instruction(&Instruction::I32WrapI64);
        count.store(f);
        let construction =
            StringConstruction::allocate(schema, schema.reserve_gc_local(f), count, f);
        let index = schema.reserve_i32_local(f);
        let position = schema.reserve_i32_local(f);
        let byte = schema.reserve_i32_local(f);
        f.instruction(&Instruction::I32Const(0));
        index.store(f);
        f.instruction(&Instruction::Block(BlockType::Empty));
        f.instruction(&Instruction::Loop(BlockType::Empty));
        index.load(f);
        count.load(f);
        f.instruction(&Instruction::I32GeU);
        f.instruction(&Instruction::BrIf(1));
        start.load(f);
        index.load(f);
        f.instruction(&Instruction::I64ExtendI32U);
        f.instruction(&Instruction::I64Add);
        f.instruction(&Instruction::I32WrapI64);
        position.store(f);
        schema
            .array_type::<ByteArray>()
            .read(&self.bytes, position, schema, f)
            .store(byte, f);
        byte.load(f);
        f.instruction(&Instruction::I32Const(0x7f));
        f.instruction(&Instruction::I32GtU);
        builder.emit_temporal_provider_corruption_if_i32(f);
        construction.write(index, byte, schema, f);
        index.load(f);
        f.instruction(&Instruction::I32Const(1));
        f.instruction(&Instruction::I32Add);
        index.store(f);
        f.instruction(&Instruction::Br(0));
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        let result = schema
            .reserve_gc_local(f)
            .initialize(construction.publish(schema, f), f);
        schema.release_i32_local(byte, f);
        schema.release_i32_local(position, f);
        schema.release_i32_local(index, f);
        schema.release_i32_local(count, f);
        result
    }
    pub(super) fn clear(self, builder: &FunctionBuilder<'_>, f: &mut Function) {
        builder.runtime_schema().release_i64_local(self.length, f);
        self.bytes.clear(f);
    }
}

pub(in crate::builtins) enum NamedTimeZoneDataRequest<'a> {
    Offset {
        zone: &'a ResolvedTemporalZoneLocals,
        instant: &'a NormalizedTemporalInstantLocals,
    },
    Inverse {
        zone: &'a ResolvedTemporalZoneLocals,
        local: &'a TemporalLocalCoordinateLocals,
    },
    Transition {
        zone: &'a ResolvedTemporalZoneLocals,
        instant: &'a NormalizedTemporalInstantLocals,
        direction: &'a TemporalTransitionDirectionLocals,
    },
}
impl NamedTimeZoneDataRequest<'_> {
    fn operation(&self) -> TemporalDataOperation {
        match self {
            Self::Offset { .. } => TemporalDataOperation::Offset,
            Self::Inverse { .. } => TemporalDataOperation::Inverse,
            Self::Transition { .. } => TemporalDataOperation::Transition,
        }
    }
    fn identifier(&self) -> &GcLocal<StringValue> {
        match self {
            Self::Offset { zone, .. }
            | Self::Inverse { zone, .. }
            | Self::Transition { zone, .. } => zone.identifier(),
        }
    }
    fn coordinate(&self) -> (I64Local, I64Local) {
        match self {
            Self::Offset { instant, .. } | Self::Transition { instant, .. } => {
                (instant.floor_seconds(), instant.nanosecond())
            }
            Self::Inverse { local, .. } => (local.floor_seconds(), local.nanosecond()),
        }
    }
}
impl FunctionBuilder<'_> {
    pub(in crate::builtins) fn emit_named_time_zone_data_call(
        &mut self,
        request: NamedTimeZoneDataRequest<'_>,
        f: &mut Function,
    ) -> Result<TemporalDataResponse, EmitError> {
        let schema = self.runtime_schema();
        let operation = request.operation();
        let message = IntlByteArrayBuilder::with_operation(operation.host_op(), schema, f);
        let (seconds, nano) = request.coordinate();
        message.append_u64(seconds, schema, f);
        message.append_u64(nano, schema, f);
        match &request {
            NamedTimeZoneDataRequest::Transition { direction, .. } => {
                message.append_u64(direction.local(), schema, f)
            }
            NamedTimeZoneDataRequest::Offset { .. } | NamedTimeZoneDataRequest::Inverse { .. } => {
                message.append_u64_constant(0, schema, f)
            }
        }
        message.append_utf8(request.identifier(), schema, f);
        self.emit_temporal_provider_message(operation, message.finish(schema, f), f)
    }
    pub(super) fn emit_temporal_lookup_data(
        &mut self,
        identifier: &GcLocal<StringValue>,
        f: &mut Function,
    ) -> Result<TemporalDataResponse, EmitError> {
        let schema = self.runtime_schema();
        let message =
            IntlByteArrayBuilder::with_operation(IntlHostOp::LookupNamedTimeZone, schema, f);
        message.append_remaining_utf8(identifier, schema, f);
        self.emit_temporal_provider_message(
            TemporalDataOperation::Lookup,
            message.finish(schema, f),
            f,
        )
    }
    fn emit_temporal_provider_message(
        &mut self,
        operation: TemporalDataOperation,
        message: GcLocal<ByteArray>,
        f: &mut Function,
    ) -> Result<TemporalDataResponse, EmitError> {
        let schema = self.runtime_schema();
        let reply = self.emit_intl_provider_byte_call(&message, f)?;
        reply.load(schema, f).is_null(f);
        self.open_frame(ControlFrameKind::If, f);
        match operation {
            TemporalDataOperation::Lookup => self.emit_temporal_error_and_return(
                lila_ir::NativeErrorKind::RangeError,
                RuntimeErrorMessage::INVALID_TEMPORAL_TIME_ZONE_IDENTIFIER,
                f,
            )?,
            TemporalDataOperation::Offset
            | TemporalDataOperation::Inverse
            | TemporalDataOperation::Transition => {
                f.instruction(&Instruction::Unreachable);
            }
        }
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let bytes = schema
            .reserve_gc_local(f)
            .initialize(reply.load(schema, f).require_non_null(f), f);
        reply.clear(f);
        message.clear(f);
        Ok(TemporalDataResponse::from_bytes(bytes, operation, self, f))
    }
    pub(super) fn emit_temporal_fixed_inverse_data(
        &self,
        seconds: I64Local,
        nano: I64Local,
        offset: I64Local,
        f: &mut Function,
    ) -> TemporalDataResponse {
        let schema = self.runtime_schema();
        let message = IntlByteArrayBuilder::new(schema, f);
        message.append_u64_constant(1, schema, f);
        message.append_u64_constant(1, schema, f);
        let epoch = schema.reserve_i64_local(f);
        seconds.load(f);
        offset.load(f);
        f.instruction(&Instruction::I64Sub);
        epoch.store(f);
        message.append_u64(epoch, schema, f);
        message.append_u64(nano, schema, f);
        message.append_u64(offset, schema, f);
        schema.release_i64_local(epoch, f);
        TemporalDataResponse::from_bytes(
            message.finish(schema, f),
            TemporalDataOperation::Inverse,
            self,
            f,
        )
    }
    pub(in crate::builtins) fn emit_temporal_require_canonical_nano(
        &self,
        nano: I64Local,
        function: &mut Function,
    ) {
        (nano).load(function);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64GeU);
        self.emit_temporal_provider_corruption_if_i32(function);
    }
    pub(in crate::builtins) fn emit_temporal_require_offset_seconds(
        &self,
        offset: I64Local,
        function: &mut Function,
    ) {
        (offset).load(function);
        function.instruction(&Instruction::I64Const(-86_400));
        function.instruction(&Instruction::I64LeS);
        (offset).load(function);
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::I32Or);
        self.emit_temporal_provider_corruption_if_i32(function);
    }
    pub(in crate::builtins) fn emit_temporal_require_raw_epoch_context(
        &self,
        seconds: I64Local,
        function: &mut Function,
    ) {
        let limit = 8_640_000_000_000 + 3 * 86_400;
        (seconds).load(function);
        function.instruction(&Instruction::I64Const(-limit));
        function.instruction(&Instruction::I64LtS);
        (seconds).load(function);
        function.instruction(&Instruction::I64Const(limit));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32Or);
        self.emit_temporal_provider_corruption_if_i32(function);
    }
}
