use super::*;

impl FunctionBuilder<'_> {
    fn emit_nf_is_unicode_type_i32(
        &mut self,
        payload_local: u32,
        ok_local: u32,
        function: &mut Function,
    ) {
        let offset_local = self.reserve_temp_local();
        let length_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let run_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(payload_local, offset_local, length_local, function);
        self.emit_nf_set_const(ok_local, 1, function);
        self.emit_nf_set_const(index_local, 0, function);
        self.emit_nf_set_const(run_local, 0, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(offset_local, index_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const('-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        // A separator closes a run, which must have been 3..=8 long.
        function.instruction(&Instruction::LocalGet(run_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::LocalGet(run_local));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_nf_set_const(ok_local, 0, function);
        function.instruction(&Instruction::Br(3));
        function.instruction(&Instruction::End);
        self.emit_nf_set_const(run_local, 0, function);
        function.instruction(&Instruction::Else);
        self.emit_nf_is_alphanum_i32(byte_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_nf_set_const(ok_local, 0, function);
        function.instruction(&Instruction::Br(3));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(run_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(run_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        // The final run has no separator to close it.
        function.instruction(&Instruction::LocalGet(ok_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(run_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::LocalGet(run_local));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_nf_set_const(ok_local, 0, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        for local in [
            byte_local,
            run_local,
            index_local,
            length_local,
            offset_local,
        ] {
            self.release_temp_local(local);
        }
    }
    fn emit_nf_is_alphanum_i32(&self, byte_local: u32, function: &mut Function) {
        for (low, high) in [('0', '9'), ('A', 'Z'), ('a', 'z')] {
            function.instruction(&Instruction::LocalGet(byte_local));
            function.instruction(&Instruction::I64Const(low as i64));
            function.instruction(&Instruction::I64GeU);
            function.instruction(&Instruction::LocalGet(byte_local));
            function.instruction(&Instruction::I64Const(high as i64));
            function.instruction(&Instruction::I64LeU);
            function.instruction(&Instruction::I32And);
        }
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Or);
    }
}

impl FunctionBuilder<'_> {
    pub(super) fn emit_nf_currency_code(
        &mut self,
        value: u32,
        destination: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let offset = self.reserve_temp_local();
        let length = self.reserve_temp_local();
        let pointer = self.reserve_temp_local();
        let index = self.reserve_temp_local();
        let byte = self.reserve_temp_local();
        self.emit_unpack_string_payload(value, offset, length, function);
        function.instruction(&Instruction::LocalGet(length));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_nf_range_error("Invalid currency option", function)?;
        function.instruction(&Instruction::End);
        self.emit_heap_alloc_const(3, function)?;
        function.instruction(&Instruction::LocalSet(pointer));
        for position in 0..3 {
            self.emit_nf_set_const(index, position, function);
            self.emit_load_string_byte(offset, index, byte, function);
            function.instruction(&Instruction::LocalGet(byte));
            function.instruction(&Instruction::I64Const(0x20));
            function.instruction(&Instruction::I64Or);
            function.instruction(&Instruction::I64Const(b'a' as i64));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::I64Const(25));
            function.instruction(&Instruction::I64GtU);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_nf_range_error("Invalid currency option", function)?;
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalGet(pointer));
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::LocalGet(byte));
            function.instruction(&Instruction::I64Const(!0x20));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Store8(MemArg {
                offset: position as u64,
                align: 0,
                memory_index: 0,
            }));
        }
        self.emit_pack_string_payload(pointer, length, function);
        function.instruction(&Instruction::LocalSet(destination));
        for local in [byte, index, pointer, length, offset] {
            self.release_temp_local(local);
        }
        Ok(())
    }
    pub(super) fn emit_nf_unit_identifier(
        &mut self,
        value: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let recognized = self.reserve_temp_local();
        let offset = self.reserve_temp_local();
        let length = self.reserve_temp_local();
        let index = self.reserve_temp_local();
        let part = self.reserve_temp_local();
        let part_length = self.reserve_temp_local();
        let split = self.reserve_temp_local();
        let suffix = self.reserve_temp_local();
        let expected = self.reserve_temp_local();
        let codes: Vec<_> = SingleUnit::ALL
            .iter()
            .enumerate()
            .map(|(index, unit)| (unit.name(), index as i64 + 1))
            .collect();
        self.emit_nf_select_spelling(value, &codes, recognized, function);
        function.instruction(&Instruction::LocalGet(recognized));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_unpack_string_payload(value, offset, length, function);
        self.emit_nf_set_const(split, 0, function);
        self.emit_nf_set_const(index, 0, function);
        self.emit_nf_set_const(part_length, 5, function);
        self.emit_nf_set_string(expected, "-per-", function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Const(5));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(length));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(offset));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(part));
        self.emit_pack_string_payload(part, part_length, function);
        function.instruction(&Instruction::LocalSet(part));
        self.emit_string_payload_equality_i32(part, expected, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_nf_copy(index, split, function);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.emit_nf_advance_wire_cursor(index, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_nf_if_nonzero(split, function);
        self.emit_pack_string_payload(offset, split, function);
        function.instruction(&Instruction::LocalSet(part));
        self.emit_nf_select_spelling(part, &codes, recognized, function);
        self.emit_nf_if_nonzero(recognized, function);
        function.instruction(&Instruction::LocalGet(split));
        function.instruction(&Instruction::I64Const(5));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(split));
        function.instruction(&Instruction::LocalGet(offset));
        function.instruction(&Instruction::LocalGet(split));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(part));
        function.instruction(&Instruction::LocalGet(length));
        function.instruction(&Instruction::LocalGet(split));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(suffix));
        self.emit_pack_string_payload(part, suffix, function);
        function.instruction(&Instruction::LocalSet(part));
        self.emit_nf_select_spelling(part, &codes, recognized, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(recognized));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_nf_range_error("Invalid unit option", function)?;
        function.instruction(&Instruction::End);
        for local in [
            expected,
            suffix,
            split,
            part_length,
            part,
            index,
            length,
            offset,
            recognized,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }
    pub(super) fn emit_nf_numbering_option(
        &mut self,
        options: TaggedLocals,
        destination: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let value = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let valid = self.reserve_temp_local();
        self.emit_nf_set_string(destination, "", function);
        self.emit_nf_string_value(options, "numberingSystem", value, function)?;
        function.instruction(&Instruction::LocalGet(value.tag));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_nf_is_unicode_type_i32(value.payload, valid, function);
        function.instruction(&Instruction::LocalGet(valid));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_nf_range_error("Invalid numberingSystem option", function)?;
        function.instruction(&Instruction::End);
        self.emit_nf_copy(value.payload, destination, function);
        function.instruction(&Instruction::End);
        for local in [valid, value.tag, value.payload] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
