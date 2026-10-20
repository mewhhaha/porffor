use super::*;
impl FunctionBuilder<'_> {
    pub(super) fn emit_segment_data_object(
        &mut self,
        record: &GcLocal<IntlSegmentsObject>,
        row: I32Local,
        f: &mut Function,
    ) -> Result<GcLocal<OrdinaryObject>, EmitError> {
        let schema = self.runtime_schema();
        let st = schema.struct_type::<IntlSegmentsObject>();
        let input = schema.reserve_gc_local(f).initialize(
            st.field(IntlSegmentsObjectSchema::INPUT)
                .read(record, schema, f)
                .reference(),
            f,
        );
        let table = schema.reserve_gc_local(f).initialize(
            st.field(IntlSegmentsObjectSchema::BOUNDARIES)
                .read(record, schema, f)
                .reference(),
            f,
        );
        let selected = schema.reserve_gc_local(f).initialize(
            schema
                .array_type::<SegmentBoundaryTable>()
                .read(&table, row, schema, f)
                .reference(),
            f,
        );
        let start = schema.reserve_i64_local(f);
        f.instruction(&Instruction::I64Const(0));
        start.store(f);
        let end = schema.reserve_i64_local(f);
        schema
            .struct_type::<SegmentBoundary>()
            .field(SegmentBoundarySchema::END)
            .read(&selected, schema, f)
            .store_i64(end, f);
        let word_like = GcI32DomainLocal::new(schema, None::<bool>, f);
        schema
            .struct_type::<SegmentBoundary>()
            .field(SegmentBoundarySchema::WORD_LIKE)
            .read(&selected, schema, f)
            .store_domain(&word_like, f);
        let previous = schema.reserve_i32_local(f);
        row.load(f);
        f.instruction(&Instruction::I32Eqz);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        row.load(f);
        f.instruction(&Instruction::I32Const(1));
        f.instruction(&Instruction::I32Sub);
        previous.store(f);
        let boundary = schema.reserve_gc_local(f).initialize(
            schema
                .array_type::<SegmentBoundaryTable>()
                .read(&table, previous, schema, f)
                .reference(),
            f,
        );
        schema
            .struct_type::<SegmentBoundary>()
            .field(SegmentBoundarySchema::END)
            .read(&boundary, schema, f)
            .store_i64(start, f);
        boundary.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let result = self.emit_intl_number_result_object(f)?;
        let value = schema.reserve_value_local(f);
        let segment = schema
            .reserve_gc_local(f)
            .initialize(self.emit_gc_string_slice(&input, start, end, f), f);
        value.set_reference(&segment, schema, f);
        self.emit_intl_number_append_result_property(&result, "segment", &value, f)?;
        segment.clear(f);
        let bits = schema.reserve_i64_local(f);
        start.load(f);
        f.instruction(&Instruction::F64ConvertI64U);
        f.instruction(&Instruction::I64ReinterpretF64);
        bits.store(f);
        value.set_number(bits, f);
        self.emit_intl_number_append_result_property(&result, "index", &value, f)?;
        value.set_reference(&input, schema, f);
        self.emit_intl_number_append_result_property(&result, "input", &value, f)?;
        emit_domain_is(&word_like, None, f);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        let boolean = schema.reserve_i32_local(f);
        word_like.load(f);
        boolean.store(f);
        value.set_boolean(boolean, f);
        self.emit_intl_number_append_result_property(&result, "isWordLike", &value, f)?;
        schema.release_i32_local(boolean, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        schema.release_i64_local(bits, f);
        value.clear(f);
        schema.release_i32_local(previous, f);
        word_like.clear(schema, f);
        schema.release_i64_local(end, f);
        schema.release_i64_local(start, f);
        selected.clear(f);
        table.clear(f);
        input.clear(f);
        Ok(result)
    }
    pub(crate) fn emit_intl_segments_containing(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let record = self.emit_segments_record(f)?;
        let schema = self.runtime_schema();
        let value = schema.reserve_value_local(f);
        self.emit_builtin_arg_to_value(0, &value, f);
        let converted = schema.reserve_completion(f);
        converted.initialize(f);
        self.emit_value_to_number_payload(&value, &converted, f)?;
        self.emit_intl_number_adopt_completion(&converted, f);
        let bits = schema.reserve_i64_local(f);
        converted.value().scalar().load(f);
        bits.store(f);
        converted.clear(f);
        // ToIntegerOrInfinity: NaN becomes zero; truncation preserves the zero interval.
        bits.load(f);
        f.instruction(&Instruction::F64ReinterpretI64);
        bits.load(f);
        f.instruction(&Instruction::F64ReinterpretI64);
        f.instruction(&Instruction::F64Ne);
        self.open_frame(ControlFrameKind::If, f);
        f.instruction(&Instruction::I64Const(0));
        bits.store(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        bits.load(f);
        f.instruction(&Instruction::F64ReinterpretI64);
        f.instruction(&Instruction::F64Trunc);
        f.instruction(&Instruction::I64ReinterpretF64);
        bits.store(f);
        let input = schema.reserve_gc_local(f).initialize(
            schema
                .struct_type::<IntlSegmentsObject>()
                .field(IntlSegmentsObjectSchema::INPUT)
                .read(&record, schema, f)
                .reference(),
            f,
        );
        let units = schema.reserve_gc_local(f).initialize(
            schema
                .struct_type::<StringValue>()
                .field(StringValueSchema::CODE_UNITS)
                .read(&input, schema, f)
                .reference(),
            f,
        );
        let length = schema.reserve_i32_local(f);
        schema
            .array_type::<CodeUnitArray>()
            .length(&units, schema, f);
        length.store(f);
        let table = schema.reserve_gc_local(f).initialize(
            schema
                .struct_type::<IntlSegmentsObject>()
                .field(IntlSegmentsObjectSchema::BOUNDARIES)
                .read(&record, schema, f)
                .reference(),
            f,
        );
        let count = schema.reserve_i32_local(f);
        schema
            .array_type::<SegmentBoundaryTable>()
            .length(&table, schema, f);
        count.store(f);
        let index = schema.reserve_i64_local(f);
        let row = schema.reserve_i32_local(f);
        let end = schema.reserve_i64_local(f);
        let output = schema.reserve_value_local(f);
        output.set_undefined(f);
        // Infinity and negative integers fail these bounds before the integer conversion.
        bits.load(f);
        f.instruction(&Instruction::F64ReinterpretI64);
        f.instruction(&Instruction::F64Const(0.0.into()));
        f.instruction(&Instruction::F64Ge);
        bits.load(f);
        f.instruction(&Instruction::F64ReinterpretI64);
        length.load(f);
        f.instruction(&Instruction::F64ConvertI32U);
        f.instruction(&Instruction::F64Lt);
        f.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, f);
        bits.load(f);
        f.instruction(&Instruction::F64ReinterpretI64);
        f.instruction(&Instruction::I64TruncF64U);
        index.store(f);
        set_i32(row, 0, f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        let repeat = self.open_frame(ControlFrameKind::Loop, f);
        row.load(f);
        count.load(f);
        f.instruction(&Instruction::I32GeU);
        self.emit_branch_if_to_target(exit, f);
        let boundary = schema.reserve_gc_local(f).initialize(
            schema
                .array_type::<SegmentBoundaryTable>()
                .read(&table, row, schema, f)
                .reference(),
            f,
        );
        schema
            .struct_type::<SegmentBoundary>()
            .field(SegmentBoundarySchema::END)
            .read(&boundary, schema, f)
            .store_i64(end, f);
        boundary.clear(f);
        index.load(f);
        end.load(f);
        f.instruction(&Instruction::I64LtU);
        self.open_frame(ControlFrameKind::If, f);
        let result = self.emit_segment_data_object(&record, row, f)?;
        output.set_reference(&result, schema, f);
        result.clear(f);
        self.emit_branch_to_target(exit, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        row.load(f);
        f.instruction(&Instruction::I32Const(1));
        f.instruction(&Instruction::I32Add);
        row.store(f);
        self.emit_branch_to_target(repeat, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.completion().set_normal(&output, f);
        output.clear(f);
        schema.release_i64_local(end, f);
        schema.release_i32_local(row, f);
        schema.release_i64_local(index, f);
        schema.release_i32_local(count, f);
        table.clear(f);
        schema.release_i32_local(length, f);
        units.clear(f);
        input.clear(f);
        schema.release_i64_local(bits, f);
        value.clear(f);
        record.clear(f);
        Ok(())
    }
}
