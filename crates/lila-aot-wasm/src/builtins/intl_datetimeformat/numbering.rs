use super::numbering_systems::DtfNumberingSystem;
use super::*;

pub(super) struct DtfNumberingLocals {
    pub(super) digits_offset: u32,
    pub(super) digit_utf8_width: u32,
    pub(super) decimal_separator: u32,
}

impl FunctionBuilder<'_> {
    pub(super) fn emit_dtf_numbering_system(
        &mut self,
        record_local: u32,
        function: &mut Function,
    ) -> DtfNumberingLocals {
        let numbering = DtfNumberingLocals {
            digits_offset: self.reserve_temp_local(),
            digit_utf8_width: self.reserve_temp_local(),
            decimal_separator: self.reserve_temp_local(),
        };
        let identifier_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_INTL_DTF_NUMBERING_SYSTEM_OFFSET,
            identifier_local,
            function,
        );
        self.emit_dtf_numbering_constants(DEFAULT_NUMBERING_SYSTEM, &numbering, function);
        for system in NUMBERING_SYSTEMS {
            if system.identifier() == DEFAULT_NUMBERING_SYSTEM.identifier() {
                continue;
            }
            // ResolveLocale stores a pooled canonical literal from the same
            // table. No caller-provided string payload reaches this slot.
            self.emit_dtf_if_code_eq(
                identifier_local,
                self.strings.payload(system.identifier()),
                function,
            );
            self.emit_dtf_numbering_constants(system, &numbering, function);
            function.instruction(&Instruction::End);
        }
        self.release_temp_local(identifier_local);
        numbering
    }

    fn emit_dtf_numbering_constants(
        &mut self,
        system: DtfNumberingSystem,
        numbering: &DtfNumberingLocals,
        function: &mut Function,
    ) {
        self.emit_dtf_set_const(
            numbering.digits_offset,
            (self.strings.payload(system.digits()) as u64 >> 32) as i64,
            function,
        );
        self.emit_dtf_set_const(
            numbering.digit_utf8_width,
            system.digit_utf8_width() as i64,
            function,
        );
        self.emit_dtf_set_string(
            numbering.decimal_separator,
            system.decimal_separator(),
            function,
        );
    }

    pub(super) fn emit_dtf_number_string(
        &mut self,
        number_local: u32,
        width: u32,
        dest_local: u32,
        numbering: &DtfNumberingLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_dtf_ascii_number_string(number_local, width, dest_local, function)?;
        self.emit_dtf_localize_ascii_digits(dest_local, numbering, function)
    }

    /// Translate only the decimal digits of a generated numeric field or GMT
    /// label. Canonical time-zone identifiers never enter this display path.
    pub(super) fn emit_dtf_localize_ascii_digits(
        &mut self,
        value_local: u32,
        numbering: &DtfNumberingLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(numbering.digits_offset));
        function.instruction(&Instruction::I64Const(
            (self.strings.payload(DEFAULT_NUMBERING_SYSTEM.digits()) as u64 >> 32) as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));

        let source_offset = self.reserve_temp_local();
        let source_length = self.reserve_temp_local();
        let capacity = self.reserve_temp_local();
        let output_offset = self.reserve_temp_local();
        let input_index = self.reserve_temp_local();
        let output_length = self.reserve_temp_local();
        let byte = self.reserve_temp_local();
        let piece_source = self.reserve_temp_local();
        let piece_length = self.reserve_temp_local();
        let piece_destination = self.reserve_temp_local();

        self.emit_unpack_string_payload(value_local, source_offset, source_length, function);
        // A field is bounded by the Date/Temporal year range; a GMT label is
        // shorter still. Each ASCII byte needs at most one selected digit.
        function.instruction(&Instruction::LocalGet(source_length));
        function.instruction(&Instruction::LocalGet(numbering.digit_utf8_width));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(!7_i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(capacity));
        self.emit_heap_alloc_from_local(capacity, function)?;
        function.instruction(&Instruction::LocalSet(output_offset));
        self.emit_dtf_set_const(input_index, 0, function);
        self.emit_dtf_set_const(output_length, 0, function);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(input_index));
        function.instruction(&Instruction::LocalGet(source_length));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(source_offset, input_index, byte, function);
        function.instruction(&Instruction::LocalGet(byte));
        function.instruction(&Instruction::I64Const(i64::from(b'0')));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(numbering.digits_offset));
        function.instruction(&Instruction::LocalGet(byte));
        function.instruction(&Instruction::I64Const(i64::from(b'0')));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalGet(numbering.digit_utf8_width));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(piece_source));
        function.instruction(&Instruction::LocalGet(numbering.digit_utf8_width));
        function.instruction(&Instruction::LocalSet(piece_length));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(source_offset));
        function.instruction(&Instruction::LocalGet(input_index));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(piece_source));
        self.emit_dtf_set_const(piece_length, 1, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(output_offset));
        function.instruction(&Instruction::LocalGet(output_length));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(piece_destination));
        self.emit_copy_bytes(piece_source, piece_destination, piece_length, function);
        function.instruction(&Instruction::LocalGet(output_length));
        function.instruction(&Instruction::LocalGet(piece_length));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(output_length));
        function.instruction(&Instruction::LocalGet(input_index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(input_index));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_pack_string_payload(output_offset, output_length, function);
        function.instruction(&Instruction::LocalSet(value_local));

        for local in [
            piece_destination,
            piece_length,
            piece_source,
            byte,
            output_length,
            input_index,
            output_offset,
            capacity,
            source_length,
            source_offset,
        ] {
            self.release_temp_local(local);
        }
        function.instruction(&Instruction::End);
        Ok(())
    }
}
