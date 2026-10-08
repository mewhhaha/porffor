use super::*;
/// Only a complete, checked native partition can become the retained Segments table.
struct CheckedPartitionLocals {
    boundaries: GcLocal<SegmentBoundaryTable>,
}
struct ObservedSegmentPartition<'a> {
    response: SegmentProviderResponse,
    input: &'a CompletedSegmentInputLocals,
}
impl FunctionBuilder<'_> {
    pub(crate) fn emit_intl_segmenter_segment(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        // Branding precedes input ToString; one provider call completes the retained partition.
        let record = self.emit_segmenter_record(f)?;
        let schema = self.runtime_schema();
        let value = schema.reserve_value_local(f);
        self.emit_builtin_arg_to_value(0, &value, f);
        let string = self.emit_intl_number_to_string(&value, f)?;
        let units = schema.reserve_gc_local(f).initialize(
            schema
                .struct_type::<StringValue>()
                .field(StringValueSchema::CODE_UNITS)
                .read(&string, schema, f)
                .reference(),
            f,
        );
        let unit_count = schema.reserve_i32_local(f);
        schema
            .array_type::<CodeUnitArray>()
            .length(&units, schema, f);
        unit_count.store(f);
        units.clear(f);
        let input = CompletedSegmentInputLocals { string, unit_count };
        let partition = self.emit_segment_native_partition(&record, &input, f)?;
        let header = self.emit_segment_intrinsic_header(SegmentIntrinsic::SegmentsPrototype, f)?;
        let result = schema.reserve_gc_local(f).initialize(
            schema.struct_type::<IntlSegmentsObject>().construct(
                (
                    GcOperand::reference(&header, schema),
                    GcOperand::reference(&record, schema),
                    GcOperand::reference(&input.string, schema),
                    GcOperand::reference(&partition.boundaries, schema),
                ),
                f,
            ),
            f,
        );
        self.completion().initialize(f);
        self.completion().value().set_reference(&result, schema, f);
        result.clear(f);
        header.clear(f);
        partition.boundaries.clear(f);
        input.clear(schema, f);
        value.clear(f);
        record.clear(f);
        Ok(())
    }
    fn emit_segment_native_partition(
        &mut self,
        record: &GcLocal<IntlSegmenterObject>,
        input: &CompletedSegmentInputLocals,
        f: &mut Function,
    ) -> Result<CheckedPartitionLocals, EmitError> {
        let observed = ObservedSegmentPartition {
            response: self.emit_segment_provider_call(
                SegmentProviderRequest::Partition { record, input },
                f,
            )?,
            input,
        };
        let schema = self.runtime_schema();
        let reader = observed.response.reader(schema, f);
        let actual = schema.reserve_i64_local(f);
        let granularity = GcI32DomainLocal::new(schema, SegmenterGranularity::Grapheme, f);
        schema
            .struct_type::<IntlSegmenterObject>()
            .field(IntlSegmenterObjectSchema::GRANULARITY)
            .read(record, schema, f)
            .store_domain(&granularity, f);
        reader.read_u64(actual, schema, f);
        actual.load(f);
        granularity.load(f);
        f.instruction(&Instruction::I64ExtendI32U);
        f.instruction(&Instruction::I64Ne);
        self.open_frame(ControlFrameKind::If, f);
        f.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let length = schema.reserve_i64_local(f);
        reader.read_u64(length, schema, f);
        length.load(f);
        observed.input.unit_count.load(f);
        f.instruction(&Instruction::I64ExtendI32U);
        f.instruction(&Instruction::I64Ne);
        self.open_frame(ControlFrameKind::If, f);
        f.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let count_word = schema.reserve_i64_local(f);
        reader.read_u64(count_word, schema, f);
        reader.require_records(count_word, 16, f);
        // The checked ByteArray extent bounds count below i32 before narrowing.
        let count = schema.reserve_i32_local(f);
        count_word.load(f);
        f.instruction(&Instruction::I32WrapI64);
        count.store(f);
        let construction = SegmentBoundaryConstruction::allocate(schema, count, f);
        let index = schema.reserve_i32_local(f);
        set_i32(index, 0, f);
        let previous = schema.reserve_i64_local(f);
        f.instruction(&Instruction::I64Const(0));
        previous.store(f);
        let end = schema.reserve_i64_local(f);
        let annotation = schema.reserve_i64_local(f);
        let word_like = GcI32DomainLocal::new(schema, None::<bool>, f);
        let unit_index = schema.reserve_i64_local(f);
        let first = schema.reserve_i32_local(f);
        let second = schema.reserve_i32_local(f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        let repeat = self.open_frame(ControlFrameKind::Loop, f);
        index.load(f);
        count.load(f);
        f.instruction(&Instruction::I32GeU);
        self.emit_branch_if_to_target(exit, f);
        reader.read_u64(end, schema, f);
        reader.read_u64(annotation, schema, f);
        end.load(f);
        previous.load(f);
        f.instruction(&Instruction::I64LeU);
        end.load(f);
        length.load(f);
        f.instruction(&Instruction::I64GtU);
        f.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, f);
        f.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        emit_domain_is(&granularity, SegmenterGranularity::Word, f);
        self.open_frame(ControlFrameKind::If, f);
        annotation.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64LtU);
        annotation.load(f);
        f.instruction(&Instruction::I64Const(2));
        f.instruction(&Instruction::I64GtU);
        f.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, f);
        f.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        word_like.set_constant(Some(false), f);
        annotation.load(f);
        f.instruction(&Instruction::I64Const(2));
        f.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, f);
        word_like.set_constant(Some(true), f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::Else);
        annotation.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        f.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        word_like.set_constant(None, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        // The native checked partition never separates a paired surrogate.
        end.load(f);
        length.load(f);
        f.instruction(&Instruction::I64LtU);
        self.open_frame(ControlFrameKind::If, f);
        end.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Sub);
        unit_index.store(f);
        self.emit_gc_string_code_unit_i32(&observed.input.string, unit_index, f);
        first.store(f);
        self.emit_gc_string_code_unit_i32(&observed.input.string, end, f);
        second.store(f);
        first.load(f);
        f.instruction(&Instruction::I32Const(0xd800));
        f.instruction(&Instruction::I32Sub);
        f.instruction(&Instruction::I32Const(0x3ff));
        f.instruction(&Instruction::I32LeU);
        second.load(f);
        f.instruction(&Instruction::I32Const(0xdc00));
        f.instruction(&Instruction::I32Sub);
        f.instruction(&Instruction::I32Const(0x3ff));
        f.instruction(&Instruction::I32LeU);
        f.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, f);
        f.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let boundary = schema.reserve_gc_local(f).initialize(
            schema
                .struct_type::<SegmentBoundary>()
                .construct((GcOperand::i64_local(end), word_like.operand()), f),
            f,
        );
        construction.write(index, &boundary, schema, f);
        boundary.clear(f);
        end.load(f);
        previous.store(f);
        index.load(f);
        f.instruction(&Instruction::I32Const(1));
        f.instruction(&Instruction::I32Add);
        index.store(f);
        self.emit_branch_to_target(repeat, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        previous.load(f);
        length.load(f);
        f.instruction(&Instruction::I64Ne);
        self.open_frame(ControlFrameKind::If, f);
        f.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        reader.finish(schema, f);
        let boundaries = schema
            .reserve_gc_local(f)
            .initialize(construction.publish(schema, f), f);
        schema.release_i32_local(second, f);
        schema.release_i32_local(first, f);
        schema.release_i64_local(unit_index, f);
        word_like.clear(schema, f);
        schema.release_i64_local(annotation, f);
        schema.release_i64_local(end, f);
        schema.release_i64_local(previous, f);
        schema.release_i32_local(index, f);
        schema.release_i32_local(count, f);
        schema.release_i64_local(count_word, f);
        schema.release_i64_local(length, f);
        granularity.clear(schema, f);
        schema.release_i64_local(actual, f);
        observed.response.clear(f);
        Ok(CheckedPartitionLocals { boundaries })
    }
}
