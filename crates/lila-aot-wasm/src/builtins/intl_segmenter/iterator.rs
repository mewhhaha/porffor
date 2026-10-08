use super::*;
impl FunctionBuilder<'_> {
    pub(crate) fn emit_intl_segments_iterator(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let segments = self.emit_segments_record(f)?;
        let schema = self.runtime_schema();
        let header = self.emit_segment_intrinsic_header(SegmentIntrinsic::IteratorPrototype, f)?;
        // The strong Segments edge retains the Segmenter, exact String and completed partition.
        let result = schema.reserve_gc_local(f).initialize(
            schema.struct_type::<IntlSegmentIteratorObject>().construct(
                (
                    GcOperand::reference(&header, schema),
                    GcOperand::reference(&segments, schema),
                    GcOperand::i64(0),
                    GcOperand::boolean(false),
                ),
                f,
            ),
            f,
        );
        self.completion().initialize(f);
        self.completion().value().set_reference(&result, schema, f);
        result.clear(f);
        header.clear(f);
        segments.clear(f);
        Ok(())
    }
    pub(crate) fn emit_intl_segment_iterator_next(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let iterator = self.emit_segment_iterator_record(f)?;
        let schema = self.runtime_schema();
        let st = schema.struct_type::<IntlSegmentIteratorObject>();
        let done = schema.reserve_i32_local(f);
        st.field(IntlSegmentIteratorObjectSchema::DONE)
            .read(&iterator, schema, f)
            .store(done, f);
        let value = schema.reserve_value_local(f);
        value.set_undefined(f);
        done.load(f);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        let segments = schema.reserve_gc_local(f).initialize(
            st.field(IntlSegmentIteratorObjectSchema::SEGMENTS)
                .read(&iterator, schema, f)
                .reference(),
            f,
        );
        let table = schema.reserve_gc_local(f).initialize(
            schema
                .struct_type::<IntlSegmentsObject>()
                .field(IntlSegmentsObjectSchema::BOUNDARIES)
                .read(&segments, schema, f)
                .reference(),
            f,
        );
        let count = schema.reserve_i32_local(f);
        schema
            .array_type::<SegmentBoundaryTable>()
            .length(&table, schema, f);
        count.store(f);
        let row = schema.reserve_i64_local(f);
        st.field(IntlSegmentIteratorObjectSchema::NEXT_INDEX)
            .read(&iterator, schema, f)
            .store_i64(row, f);
        let ordinal = schema.reserve_i32_local(f);
        row.load(f);
        count.load(f);
        f.instruction(&Instruction::I64ExtendI32U);
        f.instruction(&Instruction::I64GeU);
        self.open_frame(ControlFrameKind::If, f);
        set_i32(done, 1, f);
        st.field(IntlSegmentIteratorObjectSchema::DONE).write(
            &iterator,
            GcOperand::boolean(true),
            schema,
            f,
        );
        f.instruction(&Instruction::Else);
        row.load(f);
        f.instruction(&Instruction::I32WrapI64);
        ordinal.store(f);
        let result = self.emit_segment_data_object(&segments, ordinal, f)?;
        value.set_reference(&result, schema, f);
        result.clear(f);
        row.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        row.store(f);
        st.field(IntlSegmentIteratorObjectSchema::NEXT_INDEX).write(
            &iterator,
            GcOperand::i64_local(row),
            schema,
            f,
        );
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        schema.release_i32_local(ordinal, f);
        schema.release_i64_local(row, f);
        schema.release_i32_local(count, f);
        table.clear(f);
        segments.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        // Every call, including repeated completion, creates its own IteratorResult.
        let realm = schema
            .reserve_gc_local(f)
            .initialize(self.emit_current_function_realm(f), f);
        let result = self.emit_iterator_result_object_in_realm(&realm, &value, done, f)?;
        self.completion().initialize(f);
        self.completion().value().set_reference(&result, schema, f);
        result.clear(f);
        realm.clear(f);
        value.clear(f);
        schema.release_i32_local(done, f);
        iterator.clear(f);
        Ok(())
    }
}
