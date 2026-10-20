use super::*;
impl FunctionBuilder<'_> {
    pub(crate) fn emit_temporal_duration_to_locale_string(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        // Temporal brand admission precedes all locale/option observations.
        let schema = self.runtime_schema();
        let receiver = schema.reserve_value_local(f);
        receiver.copy_from(
            self.body_entry_locals()
                .ok_or_else(|| {
                    EmitError::unsupported("Temporal.Duration.toLocaleString lacks callable entry")
                })?
                .this_value(),
            f,
        );
        receiver.reference().load(f);
        f.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<TemporalDurationObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_intl_number_type_error(RuntimeErrorMessage::TEMPORAL_DURATION_RECEIVER_DOES_NOT_HAVE_INITIALIZEDTEMPORALDURATION, f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let stored = schema.reserve_gc_local(f).initialize(
            receiver.cast_reference::<TemporalDurationObject>(schema, f),
            f,
        );
        let fields = self.emit_duration_zero_number_fields(f);
        self.emit_duration_stored_number_fields(&stored, &fields, f);
        stored.clear(f);
        receiver.clear(f);
        let input = CompletedDurationFormatInputsLocals {
            number_bits: fields,
        };
        let record = self.emit_initialize_duration_format_record(f)?;
        self.emit_duration_partition_output(&record, &input, DurationFormatOutput::String, f)?;
        record.clear(f);
        input.clear(schema, f);
        Ok(())
    }
}
