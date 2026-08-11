use super::super::*;

#[derive(Clone, Copy)]
enum DateLocalStringFormat {
    Date,
    Time,
    DateAndTime,
}

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn emit_date_value_payload(
        &mut self,
        object_payload_local: u32,
        object_tag_local: u32,
        dest_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let slot_tag_local = self.reserve_temp_local();
        let brand_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Date method receiver is not Date",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            object_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(brand_local));
        function.instruction(&Instruction::I64Const(OBJECT_INTERNAL_BRAND_DATE as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Date method receiver is not Date",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(
            self.strings.payload(DATE_VALUE_SLOT),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            object_payload_local,
            object_tag_local,
            object_payload_local,
            object_tag_local,
            key_local,
            dest_payload_local,
            slot_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(slot_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Date method receiver is not Date",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.release_temp_local(brand_local);
        self.release_temp_local(slot_tag_local);
        self.release_temp_local(key_local);
        Ok(())
    }

    pub(crate) fn emit_date_time_clip(
        &mut self,
        input_payload_local: u32,
        dest_payload_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(input_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(input_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::LocalGet(input_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(
            8_640_000_000_000_000.0,
        )));
        function.instruction(&Instruction::F64Gt);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(input_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(
            -8_640_000_000_000_000.0,
        )));
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Result(ValType::F64)));
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NAN)));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(input_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(dest_payload_local));
    }

    pub(crate) fn emit_date_day_from_year(
        &mut self,
        year_payload_local: u32,
        dest_payload_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(year_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(1970.0)));
        function.instruction(&Instruction::F64Sub);
        function.instruction(&Instruction::F64Const(Ieee64::from(365.0)));
        function.instruction(&Instruction::F64Mul);
        for (offset, divisor, add) in [
            (1969.0, 4.0, true),
            (1901.0, 100.0, false),
            (1601.0, 400.0, true),
        ] {
            function.instruction(&Instruction::LocalGet(year_payload_local));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::F64Const(Ieee64::from(offset)));
            function.instruction(&Instruction::F64Sub);
            function.instruction(&Instruction::F64Const(Ieee64::from(divisor)));
            function.instruction(&Instruction::F64Div);
            function.instruction(&Instruction::F64Floor);
            function.instruction(&if add {
                Instruction::F64Add
            } else {
                Instruction::F64Sub
            });
        }
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(dest_payload_local));
    }

    pub(crate) fn emit_date_is_leap_year(
        &mut self,
        year_payload_local: u32,
        function: &mut Function,
    ) {
        let div4_local = self.reserve_temp_local();
        let div100_local = self.reserve_temp_local();
        let div400_local = self.reserve_temp_local();
        for divisor in [4.0, 100.0, 400.0] {
            function.instruction(&Instruction::LocalGet(year_payload_local));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::LocalGet(year_payload_local));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::F64Const(Ieee64::from(divisor)));
            function.instruction(&Instruction::F64Div);
            function.instruction(&Instruction::F64Floor);
            function.instruction(&Instruction::F64Const(Ieee64::from(divisor)));
            function.instruction(&Instruction::F64Mul);
            function.instruction(&Instruction::F64Eq);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::LocalSet(match divisor as i32 {
                4 => div4_local,
                100 => div100_local,
                400 => div400_local,
                _ => unreachable!(),
            }));
        }
        function.instruction(&Instruction::LocalGet(div4_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(div100_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::LocalGet(div400_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32And);
        self.release_temp_local(div400_local);
        self.release_temp_local(div100_local);
        self.release_temp_local(div4_local);
    }

    pub(crate) fn emit_date_month_day(
        &mut self,
        year_payload_local: u32,
        month_payload_local: u32,
        dest_payload_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(dest_payload_local));
        for (month, common, leap) in [
            (1.0, 31.0, 31.0),
            (2.0, 59.0, 60.0),
            (3.0, 90.0, 91.0),
            (4.0, 120.0, 121.0),
            (5.0, 151.0, 152.0),
            (6.0, 181.0, 182.0),
            (7.0, 212.0, 213.0),
            (8.0, 243.0, 244.0),
            (9.0, 273.0, 274.0),
            (10.0, 304.0, 305.0),
            (11.0, 334.0, 335.0),
        ] {
            function.instruction(&Instruction::LocalGet(month_payload_local));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::F64Const(Ieee64::from(month)));
            function.instruction(&Instruction::F64Ge);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_date_is_leap_year(year_payload_local, function);
            function.instruction(&Instruction::If(BlockType::Result(ValType::F64)));
            function.instruction(&Instruction::F64Const(Ieee64::from(leap)));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::F64Const(Ieee64::from(common)));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::I64ReinterpretF64);
            function.instruction(&Instruction::LocalSet(dest_payload_local));
            function.instruction(&Instruction::End);
        }
    }

    pub(crate) fn emit_date_month_from_day_within_year(
        &mut self,
        year_payload_local: u32,
        day_payload_local: u32,
        dest_payload_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(dest_payload_local));
        for (month, common, leap) in [
            (1.0, 31.0, 31.0),
            (2.0, 59.0, 60.0),
            (3.0, 90.0, 91.0),
            (4.0, 120.0, 121.0),
            (5.0, 151.0, 152.0),
            (6.0, 181.0, 182.0),
            (7.0, 212.0, 213.0),
            (8.0, 243.0, 244.0),
            (9.0, 273.0, 274.0),
            (10.0, 304.0, 305.0),
            (11.0, 334.0, 335.0),
        ] {
            function.instruction(&Instruction::LocalGet(day_payload_local));
            function.instruction(&Instruction::F64ReinterpretI64);
            self.emit_date_is_leap_year(year_payload_local, function);
            function.instruction(&Instruction::If(BlockType::Result(ValType::F64)));
            function.instruction(&Instruction::F64Const(Ieee64::from(leap)));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::F64Const(Ieee64::from(common)));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::F64Ge);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::F64Const(Ieee64::from(month)));
            function.instruction(&Instruction::I64ReinterpretF64);
            function.instruction(&Instruction::LocalSet(dest_payload_local));
            function.instruction(&Instruction::End);
        }
    }

    pub(crate) fn emit_date_make_day(
        &mut self,
        year_payload_local: u32,
        month_payload_local: u32,
        date_payload_local: u32,
        dest_payload_local: u32,
        function: &mut Function,
    ) {
        let ym_local = self.reserve_temp_local();
        let mn_local = self.reserve_temp_local();
        let month_int_local = self.reserve_temp_local();
        let day_from_year_local = self.reserve_temp_local();
        let month_day_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(month_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(month_int_local));

        function.instruction(&Instruction::LocalGet(year_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::LocalGet(month_int_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(12.0)));
        function.instruction(&Instruction::F64Div);
        function.instruction(&Instruction::F64Floor);
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(ym_local));

        function.instruction(&Instruction::LocalGet(month_int_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(month_int_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(12.0)));
        function.instruction(&Instruction::F64Div);
        function.instruction(&Instruction::F64Floor);
        function.instruction(&Instruction::F64Const(Ieee64::from(12.0)));
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::F64Sub);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(mn_local));

        self.emit_date_day_from_year(ym_local, day_from_year_local, function);
        self.emit_date_month_day(ym_local, mn_local, month_day_local, function);
        function.instruction(&Instruction::LocalGet(day_from_year_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(month_day_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::LocalGet(date_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
        function.instruction(&Instruction::F64Sub);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(dest_payload_local));

        self.release_temp_local(month_day_local);
        self.release_temp_local(day_from_year_local);
        self.release_temp_local(month_int_local);
        self.release_temp_local(mn_local);
        self.release_temp_local(ym_local);
    }

    pub(crate) fn emit_date_year_from_time(
        &mut self,
        time_payload_local: u32,
        dest_payload_local: u32,
        function: &mut Function,
    ) {
        let day_local = self.reserve_temp_local();
        let year_local = self.reserve_temp_local();
        let year_day_local = self.reserve_temp_local();
        let next_year_local = self.reserve_temp_local();
        let done_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(time_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(86_400_000.0)));
        function.instruction(&Instruction::F64Div);
        function.instruction(&Instruction::F64Floor);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(day_local));

        function.instruction(&Instruction::LocalGet(day_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(365.2425)));
        function.instruction(&Instruction::F64Div);
        function.instruction(&Instruction::F64Floor);
        function.instruction(&Instruction::F64Const(Ieee64::from(1970.0)));
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(year_local));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(done_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(done_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::BrIf(1));
        self.emit_date_day_from_year(year_local, year_day_local, function);
        function.instruction(&Instruction::LocalGet(day_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(year_day_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
        function.instruction(&Instruction::F64Sub);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(year_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(next_year_local));
        self.emit_date_day_from_year(next_year_local, year_day_local, function);
        function.instruction(&Instruction::LocalGet(day_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(year_day_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ge);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(next_year_local));
        function.instruction(&Instruction::LocalSet(year_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(done_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::LocalSet(dest_payload_local));

        self.release_temp_local(done_local);
        self.release_temp_local(next_year_local);
        self.release_temp_local(year_day_local);
        self.release_temp_local(year_local);
        self.release_temp_local(day_local);
    }

    pub(crate) fn emit_date_day_from_time(
        &mut self,
        time_payload_local: u32,
        dest_payload_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(time_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(86_400_000.0)));
        function.instruction(&Instruction::F64Div);
        function.instruction(&Instruction::F64Floor);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(dest_payload_local));
    }

    pub(crate) fn emit_date_positive_mod(
        &mut self,
        value_payload_local: u32,
        modulo: f64,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(modulo)));
        function.instruction(&Instruction::F64Div);
        function.instruction(&Instruction::F64Floor);
        function.instruction(&Instruction::F64Const(Ieee64::from(modulo)));
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::F64Sub);
    }

    pub(crate) fn emit_date_time_within_day(
        &mut self,
        value_payload_local: u32,
        dest_payload_local: u32,
        function: &mut Function,
    ) {
        self.emit_date_positive_mod(value_payload_local, 86_400_000.0, function);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(dest_payload_local));
    }

    pub(crate) fn emit_date_make_time(
        &mut self,
        hour_payload_local: u32,
        minute_payload_local: u32,
        second_payload_local: u32,
        ms_payload_local: u32,
        dest_payload_local: u32,
        function: &mut Function,
    ) {
        for (local, scale, first) in [
            (hour_payload_local, 3_600_000.0, true),
            (minute_payload_local, 60_000.0, false),
            (second_payload_local, 1_000.0, false),
            (ms_payload_local, 1.0, false),
        ] {
            function.instruction(&Instruction::LocalGet(local));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::F64Trunc);
            function.instruction(&Instruction::F64Const(Ieee64::from(scale)));
            function.instruction(&Instruction::F64Mul);
            if !first {
                function.instruction(&Instruction::F64Add);
            }
        }
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(dest_payload_local));
    }

    pub(crate) fn emit_date_components_from_time(
        &mut self,
        time_payload_local: u32,
        year_payload_local: u32,
        month_payload_local: u32,
        date_payload_local: u32,
        hour_payload_local: u32,
        minute_payload_local: u32,
        second_payload_local: u32,
        ms_payload_local: u32,
        function: &mut Function,
    ) {
        let day_payload_local = self.reserve_temp_local();
        let month_day_payload_local = self.reserve_temp_local();

        self.emit_date_year_from_time(time_payload_local, year_payload_local, function);
        self.emit_date_day_within_year(
            time_payload_local,
            year_payload_local,
            day_payload_local,
            function,
        );
        self.emit_date_month_from_day_within_year(
            year_payload_local,
            day_payload_local,
            month_payload_local,
            function,
        );
        self.emit_date_month_day(
            year_payload_local,
            month_payload_local,
            month_day_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(day_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(month_day_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Sub);
        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(date_payload_local));

        self.emit_date_positive_mod(time_payload_local, 86_400_000.0, function);
        function.instruction(&Instruction::F64Const(Ieee64::from(3_600_000.0)));
        function.instruction(&Instruction::F64Div);
        function.instruction(&Instruction::F64Floor);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(hour_payload_local));
        self.emit_date_positive_mod(time_payload_local, 3_600_000.0, function);
        function.instruction(&Instruction::F64Const(Ieee64::from(60_000.0)));
        function.instruction(&Instruction::F64Div);
        function.instruction(&Instruction::F64Floor);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(minute_payload_local));
        self.emit_date_positive_mod(time_payload_local, 60_000.0, function);
        function.instruction(&Instruction::F64Const(Ieee64::from(1_000.0)));
        function.instruction(&Instruction::F64Div);
        function.instruction(&Instruction::F64Floor);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(second_payload_local));
        self.emit_date_positive_mod(time_payload_local, 1_000.0, function);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(ms_payload_local));

        self.release_temp_local(month_day_payload_local);
        self.release_temp_local(day_payload_local);
    }

    fn emit_date_iso_expect_byte(
        &self,
        string_offset_local: u32,
        cursor_local: u32,
        byte_local: u32,
        valid_local: u32,
        expected: u8,
        function: &mut Function,
    ) {
        self.emit_load_string_byte(string_offset_local, cursor_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(valid_local));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(expected as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(cursor_local));
    }

    fn emit_date_iso_decimal(
        &self,
        string_offset_local: u32,
        cursor_local: u32,
        byte_local: u32,
        valid_local: u32,
        digits: usize,
        dest_payload_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(dest_payload_local));
        for _ in 0..digits {
            self.emit_load_string_byte(string_offset_local, cursor_local, byte_local, function);
            function.instruction(&Instruction::LocalGet(valid_local));
            function.instruction(&Instruction::LocalGet(byte_local));
            function.instruction(&Instruction::I64Const(b'0' as i64));
            function.instruction(&Instruction::I64GeU);
            function.instruction(&Instruction::LocalGet(byte_local));
            function.instruction(&Instruction::I64Const(b'9' as i64));
            function.instruction(&Instruction::I64LeU);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::LocalSet(valid_local));
            function.instruction(&Instruction::LocalGet(dest_payload_local));
            function.instruction(&Instruction::I64Const(10));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::LocalGet(byte_local));
            function.instruction(&Instruction::I64Const(b'0' as i64));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(dest_payload_local));
            function.instruction(&Instruction::LocalGet(cursor_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(cursor_local));
        }
        function.instruction(&Instruction::LocalGet(dest_payload_local));
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(dest_payload_local));
    }

    pub(crate) fn emit_date_parse_iso_string(
        &mut self,
        string_payload_local: u32,
        dest_payload_local: u32,
        function: &mut Function,
    ) {
        let string_offset_local = self.reserve_temp_local();
        let string_len_local = self.reserve_temp_local();
        let cursor_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let valid_local = self.reserve_temp_local();
        let signed_year_local = self.reserve_temp_local();
        let negative_year_local = self.reserve_temp_local();
        let year_payload_local = self.reserve_temp_local();
        let month_payload_local = self.reserve_temp_local();
        let date_payload_local = self.reserve_temp_local();
        let hour_payload_local = self.reserve_temp_local();
        let minute_payload_local = self.reserve_temp_local();
        let second_payload_local = self.reserve_temp_local();
        let ms_payload_local = self.reserve_temp_local();
        let timezone_sign_local = self.reserve_temp_local();
        let timezone_hour_payload_local = self.reserve_temp_local();
        let timezone_minute_payload_local = self.reserve_temp_local();
        let timezone_offset_payload_local = self.reserve_temp_local();
        let day_payload_local = self.reserve_temp_local();
        let time_payload_local = self.reserve_temp_local();
        let parsed_time_payload_local = self.reserve_temp_local();
        let actual_year_payload_local = self.reserve_temp_local();
        let actual_month_payload_local = self.reserve_temp_local();
        let actual_date_payload_local = self.reserve_temp_local();
        let actual_hour_payload_local = self.reserve_temp_local();
        let actual_minute_payload_local = self.reserve_temp_local();
        let actual_second_payload_local = self.reserve_temp_local();
        let actual_ms_payload_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(
            string_payload_local,
            string_offset_local,
            string_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(cursor_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(signed_year_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(negative_year_local));

        self.emit_load_string_byte(string_offset_local, cursor_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'+' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(signed_year_local));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(negative_year_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(cursor_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(signed_year_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(valid_local));
        self.emit_date_iso_decimal(
            string_offset_local,
            cursor_local,
            byte_local,
            valid_local,
            4,
            year_payload_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(valid_local));
        self.emit_date_iso_decimal(
            string_offset_local,
            cursor_local,
            byte_local,
            valid_local,
            6,
            year_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(negative_year_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(year_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Neg);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(year_payload_local));
        function.instruction(&Instruction::LocalGet(valid_local));
        function.instruction(&Instruction::LocalGet(year_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        for local in [month_payload_local, date_payload_local] {
            function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
            function.instruction(&Instruction::I64ReinterpretF64);
            function.instruction(&Instruction::LocalSet(local));
        }
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_date_iso_expect_byte(
            string_offset_local,
            cursor_local,
            byte_local,
            valid_local,
            b'-',
            function,
        );
        self.emit_date_iso_decimal(
            string_offset_local,
            cursor_local,
            byte_local,
            valid_local,
            2,
            month_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_date_iso_expect_byte(
            string_offset_local,
            cursor_local,
            byte_local,
            valid_local,
            b'-',
            function,
        );
        self.emit_date_iso_decimal(
            string_offset_local,
            cursor_local,
            byte_local,
            valid_local,
            2,
            date_payload_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for local in [
            hour_payload_local,
            minute_payload_local,
            second_payload_local,
            ms_payload_local,
            timezone_hour_payload_local,
            timezone_minute_payload_local,
            timezone_offset_payload_local,
        ] {
            function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
            function.instruction(&Instruction::I64ReinterpretF64);
            function.instruction(&Instruction::LocalSet(local));
        }
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(timezone_sign_local));

        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_date_iso_expect_byte(
            string_offset_local,
            cursor_local,
            byte_local,
            valid_local,
            b'T',
            function,
        );
        self.emit_date_iso_decimal(
            string_offset_local,
            cursor_local,
            byte_local,
            valid_local,
            2,
            hour_payload_local,
            function,
        );
        self.emit_date_iso_expect_byte(
            string_offset_local,
            cursor_local,
            byte_local,
            valid_local,
            b':',
            function,
        );
        self.emit_date_iso_decimal(
            string_offset_local,
            cursor_local,
            byte_local,
            valid_local,
            2,
            minute_payload_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_string_byte(string_offset_local, cursor_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b':' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_date_iso_expect_byte(
            string_offset_local,
            cursor_local,
            byte_local,
            valid_local,
            b':',
            function,
        );
        self.emit_date_iso_decimal(
            string_offset_local,
            cursor_local,
            byte_local,
            valid_local,
            2,
            second_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_string_byte(string_offset_local, cursor_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'.' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_date_iso_expect_byte(
            string_offset_local,
            cursor_local,
            byte_local,
            valid_local,
            b'.',
            function,
        );
        self.emit_date_iso_decimal(
            string_offset_local,
            cursor_local,
            byte_local,
            valid_local,
            3,
            ms_payload_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_string_byte(string_offset_local, cursor_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'Z' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_date_iso_expect_byte(
            string_offset_local,
            cursor_local,
            byte_local,
            valid_local,
            b'Z',
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'+' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(timezone_sign_local));
        function.instruction(&Instruction::LocalGet(valid_local));
        function.instruction(&Instruction::LocalGet(timezone_sign_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(cursor_local));
        self.emit_date_iso_decimal(
            string_offset_local,
            cursor_local,
            byte_local,
            valid_local,
            2,
            timezone_hour_payload_local,
            function,
        );
        self.emit_date_iso_expect_byte(
            string_offset_local,
            cursor_local,
            byte_local,
            valid_local,
            b':',
            function,
        );
        self.emit_date_iso_decimal(
            string_offset_local,
            cursor_local,
            byte_local,
            valid_local,
            2,
            timezone_minute_payload_local,
            function,
        );
        for (local, upper_bound) in [
            (timezone_hour_payload_local, 23.0),
            (timezone_minute_payload_local, 59.0),
        ] {
            function.instruction(&Instruction::LocalGet(valid_local));
            function.instruction(&Instruction::LocalGet(local));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::F64Const(Ieee64::from(upper_bound)));
            function.instruction(&Instruction::F64Le);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::LocalSet(valid_local));
        }
        function.instruction(&Instruction::LocalGet(timezone_hour_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(60.0)));
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::LocalGet(timezone_minute_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::F64Const(Ieee64::from(60_000.0)));
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::LocalGet(timezone_sign_local));
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(timezone_offset_payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(month_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
        function.instruction(&Instruction::F64Sub);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(month_payload_local));
        self.emit_date_make_day(
            year_payload_local,
            month_payload_local,
            date_payload_local,
            day_payload_local,
            function,
        );
        self.emit_date_make_time(
            hour_payload_local,
            minute_payload_local,
            second_payload_local,
            ms_payload_local,
            time_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(day_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(86_400_000.0)));
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::LocalGet(time_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(parsed_time_payload_local));

        self.emit_date_components_from_time(
            parsed_time_payload_local,
            actual_year_payload_local,
            actual_month_payload_local,
            actual_date_payload_local,
            actual_hour_payload_local,
            actual_minute_payload_local,
            actual_second_payload_local,
            actual_ms_payload_local,
            function,
        );
        for (actual, expected) in [
            (actual_year_payload_local, year_payload_local),
            (actual_month_payload_local, month_payload_local),
            (actual_date_payload_local, date_payload_local),
            (actual_hour_payload_local, hour_payload_local),
            (actual_minute_payload_local, minute_payload_local),
            (actual_second_payload_local, second_payload_local),
            (actual_ms_payload_local, ms_payload_local),
        ] {
            function.instruction(&Instruction::LocalGet(valid_local));
            function.instruction(&Instruction::LocalGet(actual));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::LocalGet(expected));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::F64Eq);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::LocalSet(valid_local));
        }
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalGet(valid_local));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(parsed_time_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(timezone_offset_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Sub);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(parsed_time_payload_local));
        self.emit_date_time_clip(parsed_time_payload_local, dest_payload_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NAN)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(dest_payload_local));
        function.instruction(&Instruction::End);

        for local in [
            actual_ms_payload_local,
            actual_second_payload_local,
            actual_minute_payload_local,
            actual_hour_payload_local,
            actual_date_payload_local,
            actual_month_payload_local,
            actual_year_payload_local,
            parsed_time_payload_local,
            time_payload_local,
            day_payload_local,
            timezone_offset_payload_local,
            timezone_minute_payload_local,
            timezone_hour_payload_local,
            timezone_sign_local,
            ms_payload_local,
            second_payload_local,
            minute_payload_local,
            hour_payload_local,
            date_payload_local,
            month_payload_local,
            year_payload_local,
            negative_year_local,
            signed_year_local,
            valid_local,
            byte_local,
            cursor_local,
            string_len_local,
            string_offset_local,
        ] {
            self.release_temp_local(local);
        }
    }

    pub(crate) fn emit_date_parse_string(
        &mut self,
        string_payload_local: u32,
        dest_payload_local: u32,
        function: &mut Function,
    ) {
        let known_epoch_string_local = self.reserve_temp_local();

        function
            .instruction(&Instruction::I64Const(self.strings.payload(
                "Thu Jan 01 1970 00:00:00 GMT+0000 (Coordinated Universal Time)",
            )));
        function.instruction(&Instruction::LocalSet(known_epoch_string_local));
        self.emit_string_payload_equality_i32(
            string_payload_local,
            known_epoch_string_local,
            function,
        );
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(dest_payload_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(
            self.strings.payload("Thu, 01 Jan 1970 00:00:00 GMT"),
        ));
        function.instruction(&Instruction::LocalSet(known_epoch_string_local));
        self.emit_string_payload_equality_i32(
            string_payload_local,
            known_epoch_string_local,
            function,
        );
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(dest_payload_local));
        function.instruction(&Instruction::Else);
        self.emit_date_parse_iso_string(string_payload_local, dest_payload_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(known_epoch_string_local);
    }

    pub(crate) fn emit_date_component_setter(
        &mut self,
        builtin: StandardBuiltinId,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let old_payload_local = self.reserve_temp_local();
        let new_payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        let year_payload_local = self.reserve_temp_local();
        let month_payload_local = self.reserve_temp_local();
        let date_payload_local = self.reserve_temp_local();
        let hour_payload_local = self.reserve_temp_local();
        let minute_payload_local = self.reserve_temp_local();
        let second_payload_local = self.reserve_temp_local();
        let ms_payload_local = self.reserve_temp_local();
        let arg0_payload_local = self.reserve_temp_local();
        let arg1_payload_local = self.reserve_temp_local();
        let arg2_payload_local = self.reserve_temp_local();
        let arg3_payload_local = self.reserve_temp_local();
        let day_payload_local = self.reserve_temp_local();
        let time_payload_local = self.reserve_temp_local();

        let max_args = match builtin {
            StandardBuiltinId::DatePrototypeSetFullYear
            | StandardBuiltinId::DatePrototypeSetUtcFullYear
            | StandardBuiltinId::DatePrototypeSetMinutes
            | StandardBuiltinId::DatePrototypeSetUtcMinutes => 3,
            StandardBuiltinId::DatePrototypeSetMonth
            | StandardBuiltinId::DatePrototypeSetUtcMonth
            | StandardBuiltinId::DatePrototypeSetSeconds
            | StandardBuiltinId::DatePrototypeSetUtcSeconds => 2,
            StandardBuiltinId::DatePrototypeSetHours
            | StandardBuiltinId::DatePrototypeSetUtcHours => 4,
            StandardBuiltinId::DatePrototypeSetDate
            | StandardBuiltinId::DatePrototypeSetUtcDate
            | StandardBuiltinId::DatePrototypeSetMilliseconds
            | StandardBuiltinId::DatePrototypeSetUtcMilliseconds => 1,
            _ => unreachable!(),
        };
        let is_full_year = matches!(
            builtin,
            StandardBuiltinId::DatePrototypeSetFullYear
                | StandardBuiltinId::DatePrototypeSetUtcFullYear
        );

        self.emit_date_value_payload(
            self.this_payload_local.unwrap(),
            self.this_tag_local.unwrap(),
            old_payload_local,
            function,
        )?;

        for (index, local) in [
            arg0_payload_local,
            arg1_payload_local,
            arg2_payload_local,
            arg3_payload_local,
        ]
        .into_iter()
        .enumerate()
        .take(max_args)
        {
            if index == 0 {
                self.emit_builtin_arg_to_locals(index, local, tag_local, function);
                self.emit_value_to_number_payload(tag_local, local, function)?;
                function.instruction(&Instruction::LocalSet(local));
                self.emit_return_current_completion_if_throw(function);
            } else {
                function.instruction(&Instruction::LocalGet(self.argc_param_local()));
                function.instruction(&Instruction::I64Const((index + 1) as i64));
                function.instruction(&Instruction::I64GeU);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_builtin_arg_to_locals(index, local, tag_local, function);
                self.emit_value_to_number_payload(tag_local, local, function)?;
                function.instruction(&Instruction::LocalSet(local));
                self.emit_return_current_completion_if_throw(function);
                function.instruction(&Instruction::End);
            }
        }

        function.instruction(&Instruction::LocalGet(old_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(old_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        if is_full_year {
            for (local, value) in [
                (year_payload_local, 1970.0),
                (month_payload_local, 0.0),
                (date_payload_local, 1.0),
                (hour_payload_local, 0.0),
                (minute_payload_local, 0.0),
                (second_payload_local, 0.0),
                (ms_payload_local, 0.0),
            ] {
                function.instruction(&Instruction::F64Const(Ieee64::from(value)));
                function.instruction(&Instruction::I64ReinterpretF64);
                function.instruction(&Instruction::LocalSet(local));
            }
        } else {
            function.instruction(&Instruction::F64Const(Ieee64::from(f64::NAN)));
            function.instruction(&Instruction::I64ReinterpretF64);
            function.instruction(&Instruction::LocalSet(new_payload_local));
        }
        function.instruction(&Instruction::Else);
        self.emit_date_components_from_time(
            old_payload_local,
            year_payload_local,
            month_payload_local,
            date_payload_local,
            hour_payload_local,
            minute_payload_local,
            second_payload_local,
            ms_payload_local,
            function,
        );
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(old_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(old_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Eq);
        if is_full_year {
            function.instruction(&Instruction::I32Const(1));
            function.instruction(&Instruction::I32Or);
        }
        function.instruction(&Instruction::If(BlockType::Empty));
        match builtin {
            StandardBuiltinId::DatePrototypeSetFullYear
            | StandardBuiltinId::DatePrototypeSetUtcFullYear => {
                function.instruction(&Instruction::LocalGet(arg0_payload_local));
                function.instruction(&Instruction::LocalSet(year_payload_local));
                for (index, arg, dest) in [
                    (1, arg1_payload_local, month_payload_local),
                    (2, arg2_payload_local, date_payload_local),
                ] {
                    function.instruction(&Instruction::LocalGet(self.argc_param_local()));
                    function.instruction(&Instruction::I64Const((index + 1) as i64));
                    function.instruction(&Instruction::I64GeU);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::LocalGet(arg));
                    function.instruction(&Instruction::LocalSet(dest));
                    function.instruction(&Instruction::End);
                }
            }
            StandardBuiltinId::DatePrototypeSetMonth
            | StandardBuiltinId::DatePrototypeSetUtcMonth => {
                function.instruction(&Instruction::LocalGet(arg0_payload_local));
                function.instruction(&Instruction::LocalSet(month_payload_local));
                function.instruction(&Instruction::LocalGet(self.argc_param_local()));
                function.instruction(&Instruction::I64Const(2));
                function.instruction(&Instruction::I64GeU);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(arg1_payload_local));
                function.instruction(&Instruction::LocalSet(date_payload_local));
                function.instruction(&Instruction::End);
            }
            StandardBuiltinId::DatePrototypeSetDate
            | StandardBuiltinId::DatePrototypeSetUtcDate => {
                function.instruction(&Instruction::LocalGet(arg0_payload_local));
                function.instruction(&Instruction::LocalSet(date_payload_local));
            }
            StandardBuiltinId::DatePrototypeSetHours
            | StandardBuiltinId::DatePrototypeSetUtcHours => {
                for (index, arg, dest) in [
                    (0, arg0_payload_local, hour_payload_local),
                    (1, arg1_payload_local, minute_payload_local),
                    (2, arg2_payload_local, second_payload_local),
                    (3, arg3_payload_local, ms_payload_local),
                ] {
                    if index == 0 {
                        function.instruction(&Instruction::LocalGet(arg));
                        function.instruction(&Instruction::LocalSet(dest));
                    } else {
                        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
                        function.instruction(&Instruction::I64Const((index + 1) as i64));
                        function.instruction(&Instruction::I64GeU);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::LocalGet(arg));
                        function.instruction(&Instruction::LocalSet(dest));
                        function.instruction(&Instruction::End);
                    }
                }
            }
            StandardBuiltinId::DatePrototypeSetMinutes
            | StandardBuiltinId::DatePrototypeSetUtcMinutes => {
                for (index, arg, dest) in [
                    (0, arg0_payload_local, minute_payload_local),
                    (1, arg1_payload_local, second_payload_local),
                    (2, arg2_payload_local, ms_payload_local),
                ] {
                    if index == 0 {
                        function.instruction(&Instruction::LocalGet(arg));
                        function.instruction(&Instruction::LocalSet(dest));
                    } else {
                        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
                        function.instruction(&Instruction::I64Const((index + 1) as i64));
                        function.instruction(&Instruction::I64GeU);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::LocalGet(arg));
                        function.instruction(&Instruction::LocalSet(dest));
                        function.instruction(&Instruction::End);
                    }
                }
            }
            StandardBuiltinId::DatePrototypeSetSeconds
            | StandardBuiltinId::DatePrototypeSetUtcSeconds => {
                function.instruction(&Instruction::LocalGet(arg0_payload_local));
                function.instruction(&Instruction::LocalSet(second_payload_local));
                function.instruction(&Instruction::LocalGet(self.argc_param_local()));
                function.instruction(&Instruction::I64Const(2));
                function.instruction(&Instruction::I64GeU);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(arg1_payload_local));
                function.instruction(&Instruction::LocalSet(ms_payload_local));
                function.instruction(&Instruction::End);
            }
            StandardBuiltinId::DatePrototypeSetMilliseconds
            | StandardBuiltinId::DatePrototypeSetUtcMilliseconds => {
                function.instruction(&Instruction::LocalGet(arg0_payload_local));
                function.instruction(&Instruction::LocalSet(ms_payload_local));
            }
            _ => unreachable!(),
        }
        self.emit_date_make_day(
            year_payload_local,
            month_payload_local,
            date_payload_local,
            day_payload_local,
            function,
        );
        self.emit_date_make_time(
            hour_payload_local,
            minute_payload_local,
            second_payload_local,
            ms_payload_local,
            time_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(day_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(86_400_000.0)));
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::LocalGet(time_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(new_payload_local));
        self.emit_date_time_clip(new_payload_local, new_payload_local, function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(old_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(old_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Eq);
        if is_full_year {
            function.instruction(&Instruction::I32Const(1));
            function.instruction(&Instruction::I32Or);
        }
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_define_local_data(
            self.this_payload_local.unwrap(),
            DATE_VALUE_SLOT,
            new_payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(new_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(time_payload_local);
        self.release_temp_local(day_payload_local);
        self.release_temp_local(arg3_payload_local);
        self.release_temp_local(arg2_payload_local);
        self.release_temp_local(arg1_payload_local);
        self.release_temp_local(arg0_payload_local);
        self.release_temp_local(ms_payload_local);
        self.release_temp_local(second_payload_local);
        self.release_temp_local(minute_payload_local);
        self.release_temp_local(hour_payload_local);
        self.release_temp_local(date_payload_local);
        self.release_temp_local(month_payload_local);
        self.release_temp_local(year_payload_local);
        self.release_temp_local(tag_local);
        self.release_temp_local(new_payload_local);
        self.release_temp_local(old_payload_local);
        Ok(())
    }

    pub(crate) fn emit_date_append_padded_decimal(
        &mut self,
        output_payload_local: u32,
        number_payload_local: u32,
        minimum_width: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let piece_payload_local = self.reserve_temp_local();
        for remaining_digits in (1..minimum_width).rev() {
            function.instruction(&Instruction::LocalGet(number_payload_local));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::F64Const(Ieee64::from(
                10_f64.powi(remaining_digits as i32),
            )));
            function.instruction(&Instruction::F64Lt);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(self.strings.payload("0")));
            function.instruction(&Instruction::LocalSet(piece_payload_local));
            self.emit_concat_string_payloads_local(
                output_payload_local,
                piece_payload_local,
                function,
            )?;
            function.instruction(&Instruction::LocalSet(output_payload_local));
            function.instruction(&Instruction::End);
        }
        self.emit_number_to_string_payload(number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        self.emit_concat_string_payloads_local(
            output_payload_local,
            piece_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(output_payload_local));
        self.release_temp_local(piece_payload_local);
        Ok(())
    }

    fn emit_date_local_string(
        &mut self,
        format: DateLocalStringFormat,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let time_payload_local = self.reserve_temp_local();
        let year_payload_local = self.reserve_temp_local();
        let month_payload_local = self.reserve_temp_local();
        let date_payload_local = self.reserve_temp_local();
        let hour_payload_local = self.reserve_temp_local();
        let minute_payload_local = self.reserve_temp_local();
        let second_payload_local = self.reserve_temp_local();
        let ms_payload_local = self.reserve_temp_local();
        let weekday_payload_local = self.reserve_temp_local();
        let output_payload_local = self.reserve_temp_local();
        let piece_payload_local = self.reserve_temp_local();
        let absolute_year_payload_local = self.reserve_temp_local();

        self.emit_date_value_payload(
            self.this_payload_local.unwrap(),
            self.this_tag_local.unwrap(),
            time_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(time_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(time_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("Invalid Date")));
        function.instruction(&Instruction::LocalSet(output_payload_local));
        function.instruction(&Instruction::Else);

        self.emit_date_components_from_time(
            time_payload_local,
            year_payload_local,
            month_payload_local,
            date_payload_local,
            hour_payload_local,
            minute_payload_local,
            second_payload_local,
            ms_payload_local,
            function,
        );
        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(output_payload_local));

        if matches!(
            format,
            DateLocalStringFormat::Date | DateLocalStringFormat::DateAndTime
        ) {
            self.emit_date_day_from_time(time_payload_local, weekday_payload_local, function);
            function.instruction(&Instruction::LocalGet(weekday_payload_local));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::F64Const(Ieee64::from(4.0)));
            function.instruction(&Instruction::F64Add);
            function.instruction(&Instruction::I64ReinterpretF64);
            function.instruction(&Instruction::LocalSet(weekday_payload_local));
            self.emit_date_positive_mod(weekday_payload_local, 7.0, function);
            function.instruction(&Instruction::I64ReinterpretF64);
            function.instruction(&Instruction::LocalSet(weekday_payload_local));

            function.instruction(&Instruction::I64Const(self.strings.payload("Sun")));
            function.instruction(&Instruction::LocalSet(output_payload_local));
            for (weekday, name) in [
                (1.0, "Mon"),
                (2.0, "Tue"),
                (3.0, "Wed"),
                (4.0, "Thu"),
                (5.0, "Fri"),
                (6.0, "Sat"),
            ] {
                function.instruction(&Instruction::LocalGet(weekday_payload_local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Const(Ieee64::from(weekday)));
                function.instruction(&Instruction::F64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(self.strings.payload(name)));
                function.instruction(&Instruction::LocalSet(output_payload_local));
                function.instruction(&Instruction::End);
            }
            function.instruction(&Instruction::I64Const(self.strings.payload(" ")));
            function.instruction(&Instruction::LocalSet(piece_payload_local));
            self.emit_concat_string_payloads_local(
                output_payload_local,
                piece_payload_local,
                function,
            )?;
            function.instruction(&Instruction::LocalSet(output_payload_local));

            function.instruction(&Instruction::I64Const(self.strings.payload("Jan")));
            function.instruction(&Instruction::LocalSet(piece_payload_local));
            for (month, name) in [
                (1.0, "Feb"),
                (2.0, "Mar"),
                (3.0, "Apr"),
                (4.0, "May"),
                (5.0, "Jun"),
                (6.0, "Jul"),
                (7.0, "Aug"),
                (8.0, "Sep"),
                (9.0, "Oct"),
                (10.0, "Nov"),
                (11.0, "Dec"),
            ] {
                function.instruction(&Instruction::LocalGet(month_payload_local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Const(Ieee64::from(month)));
                function.instruction(&Instruction::F64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(self.strings.payload(name)));
                function.instruction(&Instruction::LocalSet(piece_payload_local));
                function.instruction(&Instruction::End);
            }
            self.emit_concat_string_payloads_local(
                output_payload_local,
                piece_payload_local,
                function,
            )?;
            function.instruction(&Instruction::LocalSet(output_payload_local));
            function.instruction(&Instruction::I64Const(self.strings.payload(" ")));
            function.instruction(&Instruction::LocalSet(piece_payload_local));
            self.emit_concat_string_payloads_local(
                output_payload_local,
                piece_payload_local,
                function,
            )?;
            function.instruction(&Instruction::LocalSet(output_payload_local));
            self.emit_date_append_padded_decimal(
                output_payload_local,
                date_payload_local,
                2,
                function,
            )?;
            function.instruction(&Instruction::I64Const(self.strings.payload(" ")));
            function.instruction(&Instruction::LocalSet(piece_payload_local));
            self.emit_concat_string_payloads_local(
                output_payload_local,
                piece_payload_local,
                function,
            )?;
            function.instruction(&Instruction::LocalSet(output_payload_local));

            function.instruction(&Instruction::LocalGet(year_payload_local));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
            function.instruction(&Instruction::F64Lt);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            function.instruction(&Instruction::I64Const(self.strings.payload("-")));
            function.instruction(&Instruction::LocalSet(piece_payload_local));
            self.emit_concat_string_payloads_local(
                output_payload_local,
                piece_payload_local,
                function,
            )?;
            function.instruction(&Instruction::LocalSet(output_payload_local));
            function.instruction(&Instruction::LocalGet(year_payload_local));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::F64Neg);
            function.instruction(&Instruction::I64ReinterpretF64);
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(year_payload_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalSet(absolute_year_payload_local));
            self.emit_date_append_padded_decimal(
                output_payload_local,
                absolute_year_payload_local,
                4,
                function,
            )?;
        }

        if matches!(format, DateLocalStringFormat::DateAndTime) {
            function.instruction(&Instruction::I64Const(self.strings.payload(" ")));
            function.instruction(&Instruction::LocalSet(piece_payload_local));
            self.emit_concat_string_payloads_local(
                output_payload_local,
                piece_payload_local,
                function,
            )?;
            function.instruction(&Instruction::LocalSet(output_payload_local));
        }

        if matches!(
            format,
            DateLocalStringFormat::Time | DateLocalStringFormat::DateAndTime
        ) {
            for (component_payload_local, separator) in [
                (hour_payload_local, ":"),
                (minute_payload_local, ":"),
                (
                    second_payload_local,
                    " GMT+0000 (Coordinated Universal Time)",
                ),
            ] {
                self.emit_date_append_padded_decimal(
                    output_payload_local,
                    component_payload_local,
                    2,
                    function,
                )?;
                function.instruction(&Instruction::I64Const(self.strings.payload(separator)));
                function.instruction(&Instruction::LocalSet(piece_payload_local));
                self.emit_concat_string_payloads_local(
                    output_payload_local,
                    piece_payload_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalSet(output_payload_local));
            }
        }

        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(output_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        for local in [
            absolute_year_payload_local,
            piece_payload_local,
            output_payload_local,
            weekday_payload_local,
            ms_payload_local,
            second_payload_local,
            minute_payload_local,
            hour_payload_local,
            date_payload_local,
            month_payload_local,
            year_payload_local,
            time_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn emit_date_to_date_string(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_date_local_string(DateLocalStringFormat::Date, function)
    }

    pub(crate) fn emit_date_to_time_string(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_date_local_string(DateLocalStringFormat::Time, function)
    }

    pub(crate) fn emit_date_to_string(&mut self, function: &mut Function) -> Result<(), EmitError> {
        self.emit_date_local_string(DateLocalStringFormat::DateAndTime, function)
    }

    pub(crate) fn emit_date_to_utc_string(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let time_payload_local = self.reserve_temp_local();
        let year_payload_local = self.reserve_temp_local();
        let month_payload_local = self.reserve_temp_local();
        let date_payload_local = self.reserve_temp_local();
        let hour_payload_local = self.reserve_temp_local();
        let minute_payload_local = self.reserve_temp_local();
        let second_payload_local = self.reserve_temp_local();
        let ms_payload_local = self.reserve_temp_local();
        let weekday_payload_local = self.reserve_temp_local();
        let output_payload_local = self.reserve_temp_local();
        let piece_payload_local = self.reserve_temp_local();
        let absolute_year_payload_local = self.reserve_temp_local();

        self.emit_date_value_payload(
            self.this_payload_local.unwrap(),
            self.this_tag_local.unwrap(),
            time_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(time_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(time_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("Invalid Date")));
        function.instruction(&Instruction::LocalSet(output_payload_local));
        function.instruction(&Instruction::Else);

        self.emit_date_components_from_time(
            time_payload_local,
            year_payload_local,
            month_payload_local,
            date_payload_local,
            hour_payload_local,
            minute_payload_local,
            second_payload_local,
            ms_payload_local,
            function,
        );
        self.emit_date_day_from_time(time_payload_local, weekday_payload_local, function);
        function.instruction(&Instruction::LocalGet(weekday_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(4.0)));
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(weekday_payload_local));
        self.emit_date_positive_mod(weekday_payload_local, 7.0, function);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(weekday_payload_local));

        function.instruction(&Instruction::I64Const(self.strings.payload("Sun")));
        function.instruction(&Instruction::LocalSet(output_payload_local));
        for (weekday, name) in [
            (1.0, "Mon"),
            (2.0, "Tue"),
            (3.0, "Wed"),
            (4.0, "Thu"),
            (5.0, "Fri"),
            (6.0, "Sat"),
        ] {
            function.instruction(&Instruction::LocalGet(weekday_payload_local));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::F64Const(Ieee64::from(weekday)));
            function.instruction(&Instruction::F64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(self.strings.payload(name)));
            function.instruction(&Instruction::LocalSet(output_payload_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::I64Const(self.strings.payload(", ")));
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        self.emit_concat_string_payloads_local(
            output_payload_local,
            piece_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(output_payload_local));
        self.emit_date_append_padded_decimal(
            output_payload_local,
            date_payload_local,
            2,
            function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload(" ")));
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        self.emit_concat_string_payloads_local(
            output_payload_local,
            piece_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(output_payload_local));

        function.instruction(&Instruction::I64Const(self.strings.payload("Jan")));
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        for (month, name) in [
            (1.0, "Feb"),
            (2.0, "Mar"),
            (3.0, "Apr"),
            (4.0, "May"),
            (5.0, "Jun"),
            (6.0, "Jul"),
            (7.0, "Aug"),
            (8.0, "Sep"),
            (9.0, "Oct"),
            (10.0, "Nov"),
            (11.0, "Dec"),
        ] {
            function.instruction(&Instruction::LocalGet(month_payload_local));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::F64Const(Ieee64::from(month)));
            function.instruction(&Instruction::F64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(self.strings.payload(name)));
            function.instruction(&Instruction::LocalSet(piece_payload_local));
            function.instruction(&Instruction::End);
        }
        self.emit_concat_string_payloads_local(
            output_payload_local,
            piece_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(output_payload_local));
        function.instruction(&Instruction::I64Const(self.strings.payload(" ")));
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        self.emit_concat_string_payloads_local(
            output_payload_local,
            piece_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(output_payload_local));

        function.instruction(&Instruction::LocalGet(year_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(self.strings.payload("-")));
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        self.emit_concat_string_payloads_local(
            output_payload_local,
            piece_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(output_payload_local));
        function.instruction(&Instruction::LocalGet(year_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Neg);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(year_payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(absolute_year_payload_local));
        self.emit_date_append_padded_decimal(
            output_payload_local,
            absolute_year_payload_local,
            4,
            function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload(" ")));
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        self.emit_concat_string_payloads_local(
            output_payload_local,
            piece_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(output_payload_local));

        for (component_payload_local, separator) in [
            (hour_payload_local, ":"),
            (minute_payload_local, ":"),
            (second_payload_local, " GMT"),
        ] {
            self.emit_date_append_padded_decimal(
                output_payload_local,
                component_payload_local,
                2,
                function,
            )?;
            function.instruction(&Instruction::I64Const(self.strings.payload(separator)));
            function.instruction(&Instruction::LocalSet(piece_payload_local));
            self.emit_concat_string_payloads_local(
                output_payload_local,
                piece_payload_local,
                function,
            )?;
            function.instruction(&Instruction::LocalSet(output_payload_local));
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(output_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(absolute_year_payload_local);
        self.release_temp_local(piece_payload_local);
        self.release_temp_local(output_payload_local);
        self.release_temp_local(weekday_payload_local);
        self.release_temp_local(ms_payload_local);
        self.release_temp_local(second_payload_local);
        self.release_temp_local(minute_payload_local);
        self.release_temp_local(hour_payload_local);
        self.release_temp_local(date_payload_local);
        self.release_temp_local(month_payload_local);
        self.release_temp_local(year_payload_local);
        self.release_temp_local(time_payload_local);
        Ok(())
    }

    pub(crate) fn emit_date_to_iso_string(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let time_payload_local = self.reserve_temp_local();
        let year_payload_local = self.reserve_temp_local();
        let month_payload_local = self.reserve_temp_local();
        let date_payload_local = self.reserve_temp_local();
        let hour_payload_local = self.reserve_temp_local();
        let minute_payload_local = self.reserve_temp_local();
        let second_payload_local = self.reserve_temp_local();
        let ms_payload_local = self.reserve_temp_local();
        let output_payload_local = self.reserve_temp_local();
        let piece_payload_local = self.reserve_temp_local();
        let absolute_year_payload_local = self.reserve_temp_local();

        self.emit_date_value_payload(
            self.this_payload_local.unwrap(),
            self.this_tag_local.unwrap(),
            time_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(time_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(time_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Date value is not finite",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_date_components_from_time(
            time_payload_local,
            year_payload_local,
            month_payload_local,
            date_payload_local,
            hour_payload_local,
            minute_payload_local,
            second_payload_local,
            ms_payload_local,
            function,
        );
        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(output_payload_local));

        function.instruction(&Instruction::LocalGet(year_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("-")));
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        self.emit_concat_string_payloads_local(
            output_payload_local,
            piece_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(output_payload_local));
        function.instruction(&Instruction::LocalGet(year_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Neg);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(absolute_year_payload_local));
        self.emit_date_append_padded_decimal(
            output_payload_local,
            absolute_year_payload_local,
            6,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(year_payload_local));
        function.instruction(&Instruction::LocalSet(absolute_year_payload_local));
        function.instruction(&Instruction::LocalGet(year_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(9_999.0)));
        function.instruction(&Instruction::F64Gt);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("+")));
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        self.emit_concat_string_payloads_local(
            output_payload_local,
            piece_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(output_payload_local));
        self.emit_date_append_padded_decimal(
            output_payload_local,
            absolute_year_payload_local,
            6,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_date_append_padded_decimal(
            output_payload_local,
            absolute_year_payload_local,
            4,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("-")));
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        self.emit_concat_string_payloads_local(
            output_payload_local,
            piece_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(output_payload_local));
        function.instruction(&Instruction::LocalGet(month_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(month_payload_local));

        for (component_payload_local, minimum_width, separator) in [
            (month_payload_local, 2, "-"),
            (date_payload_local, 2, "T"),
            (hour_payload_local, 2, ":"),
            (minute_payload_local, 2, ":"),
            (second_payload_local, 2, "."),
            (ms_payload_local, 3, "Z"),
        ] {
            self.emit_date_append_padded_decimal(
                output_payload_local,
                component_payload_local,
                minimum_width,
                function,
            )?;
            function.instruction(&Instruction::I64Const(self.strings.payload(separator)));
            function.instruction(&Instruction::LocalSet(piece_payload_local));
            self.emit_concat_string_payloads_local(
                output_payload_local,
                piece_payload_local,
                function,
            )?;
            function.instruction(&Instruction::LocalSet(output_payload_local));
        }
        function.instruction(&Instruction::LocalGet(output_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(absolute_year_payload_local);
        self.release_temp_local(piece_payload_local);
        self.release_temp_local(output_payload_local);
        self.release_temp_local(ms_payload_local);
        self.release_temp_local(second_payload_local);
        self.release_temp_local(minute_payload_local);
        self.release_temp_local(hour_payload_local);
        self.release_temp_local(date_payload_local);
        self.release_temp_local(month_payload_local);
        self.release_temp_local(year_payload_local);
        self.release_temp_local(time_payload_local);
        Ok(())
    }

    pub(crate) fn emit_date_to_temporal_instant(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let time_payload_local = self.reserve_temp_local();
        let milliseconds_local = self.reserve_temp_local();
        let nanoseconds_payload_local = self.reserve_temp_local();
        let nanoseconds_tag_local = self.reserve_temp_local();
        let instant_prototype_local = self.reserve_temp_local();

        self.emit_date_value_payload(
            self.this_payload_local.unwrap(),
            self.this_tag_local.unwrap(),
            time_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(time_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(time_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Date value is not finite",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(time_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64S);
        function.instruction(&Instruction::LocalSet(milliseconds_local));
        // No range check follows, and that is not an omission: `TimeClip` has
        // already bounded the time value to ±8.64e15 ms, whose nanosecond
        // widening is exactly the `IsValidEpochNanoseconds` limit.
        self.emit_temporal_epoch_milliseconds_to_epoch_nanoseconds(
            milliseconds_local,
            UnvalidatedEpochNanoseconds {
                payload_local: nanoseconds_payload_local,
                tag_local: nanoseconds_tag_local,
            },
            function,
        )?;

        function.instruction(&Instruction::GlobalGet(
            TEMPORAL_INSTANT_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(instant_prototype_local));
        self.emit_alloc_temporal_instant(
            nanoseconds_payload_local,
            nanoseconds_tag_local,
            instant_prototype_local,
            function,
        )?;

        self.release_temp_local(instant_prototype_local);
        self.release_temp_local(nanoseconds_tag_local);
        self.release_temp_local(nanoseconds_payload_local);
        self.release_temp_local(milliseconds_local);
        self.release_temp_local(time_payload_local);
        Ok(())
    }

    pub(crate) fn emit_date_to_json(&mut self, function: &mut Function) -> Result<(), EmitError> {
        let object_payload_local = self.reserve_temp_local();
        let object_tag_local = self.reserve_temp_local();
        let primitive_payload_local = self.reserve_temp_local();
        let primitive_tag_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let method_payload_local = self.reserve_temp_local();
        let method_tag_local = self.reserve_temp_local();

        self.emit_value_to_current_function_realm_object_locals(
            self.this_payload_local.unwrap(),
            self.this_tag_local.unwrap(),
            object_payload_local,
            object_tag_local,
            function,
        )?;
        self.emit_tagged_to_primitive_locals(
            ToPrimitiveHint::Number,
            object_payload_local,
            object_tag_local,
            primitive_payload_local,
            primitive_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);

        function.instruction(&Instruction::LocalGet(primitive_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(primitive_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(primitive_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::LocalGet(primitive_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(primitive_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NEG_INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Else);

        function.instruction(&Instruction::I64Const(self.strings.payload("toISOString")));
        function.instruction(&Instruction::LocalSet(key_payload_local));
        self.emit_object_read(
            object_payload_local,
            object_tag_local,
            object_payload_local,
            object_tag_local,
            key_payload_local,
            method_payload_local,
            method_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_is_callable_i32(method_tag_local, method_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Date toISOString method is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_function_or_proxy_call_leave_throw_completion(
            method_payload_local,
            method_tag_local,
            object_payload_local,
            object_tag_local,
            &[],
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);

        self.release_temp_local(method_tag_local);
        self.release_temp_local(method_payload_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(primitive_tag_local);
        self.release_temp_local(primitive_payload_local);
        self.release_temp_local(object_tag_local);
        self.release_temp_local(object_payload_local);
        Ok(())
    }

    pub(crate) fn emit_date_to_primitive(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.unwrap();
        let receiver_tag_local = self.this_tag_local.unwrap();
        let hint_payload_local = self.reserve_temp_local();
        let hint_tag_local = self.reserve_temp_local();
        let string_hint_payload_local = self.reserve_temp_local();
        let default_hint_payload_local = self.reserve_temp_local();
        let number_hint_payload_local = self.reserve_temp_local();
        let string_first_local = self.reserve_temp_local();
        let valid_hint_local = self.reserve_temp_local();
        let found_primitive_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let method_payload_local = self.reserve_temp_local();
        let method_tag_local = self.reserve_temp_local();
        let call_payload_local = self.reserve_temp_local();
        let call_tag_local = self.reserve_temp_local();

        self.emit_is_heap_object_like_tag_i32(receiver_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Date.prototype[Symbol.toPrimitive] receiver is not an object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_builtin_arg_to_locals(0, hint_payload_local, hint_tag_local, function);
        function.instruction(&Instruction::I64Const(self.strings.payload("string")));
        function.instruction(&Instruction::LocalSet(string_hint_payload_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("default")));
        function.instruction(&Instruction::LocalSet(default_hint_payload_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("number")));
        function.instruction(&Instruction::LocalSet(number_hint_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(string_first_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_hint_local));

        function.instruction(&Instruction::LocalGet(hint_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        for (expected_hint_local, string_first) in [
            (string_hint_payload_local, true),
            (default_hint_payload_local, true),
            (number_hint_payload_local, false),
        ] {
            self.emit_string_payload_equality_i32(
                hint_payload_local,
                expected_hint_local,
                function,
            );
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(valid_hint_local));
            if string_first {
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(string_first_local));
            }
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(valid_hint_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Date.prototype[Symbol.toPrimitive] hint must be \"default\", \"number\", or \"string\"",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(found_primitive_local));
        for second_attempt in [false, true] {
            function.instruction(&Instruction::LocalGet(found_primitive_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(string_first_local));
            if second_attempt {
                function.instruction(&Instruction::I64Eqz);
            } else {
                function.instruction(&Instruction::I32WrapI64);
            }
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            function.instruction(&Instruction::I64Const(self.strings.payload("toString")));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::I64Const(self.strings.payload("valueOf")));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalSet(key_payload_local));
            self.emit_object_read(
                receiver_payload_local,
                receiver_tag_local,
                receiver_payload_local,
                receiver_tag_local,
                key_payload_local,
                method_payload_local,
                method_tag_local,
                function,
            )?;
            self.emit_return_current_completion_if_throw(function);
            self.emit_is_callable_i32(method_tag_local, method_payload_local, function)?;
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_function_or_proxy_call_leave_throw_completion(
                method_payload_local,
                method_tag_local,
                receiver_payload_local,
                receiver_tag_local,
                &[],
                call_payload_local,
                call_tag_local,
                function,
            )?;
            self.emit_return_current_completion_if_throw(function);
            self.emit_is_primitive_tag_i32(call_tag_local, function);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(call_payload_local));
            function.instruction(&Instruction::LocalSet(self.result_local));
            function.instruction(&Instruction::LocalGet(call_tag_local));
            function.instruction(&Instruction::LocalSet(self.result_tag_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(found_primitive_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }

        function.instruction(&Instruction::LocalGet(found_primitive_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Cannot convert object to primitive value",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        for local in [
            call_tag_local,
            call_payload_local,
            method_tag_local,
            method_payload_local,
            key_payload_local,
            found_primitive_local,
            valid_hint_local,
            string_first_local,
            number_hint_payload_local,
            default_hint_payload_local,
            string_hint_payload_local,
            hint_tag_local,
            hint_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn emit_date_day_within_year(
        &mut self,
        time_payload_local: u32,
        year_payload_local: u32,
        dest_payload_local: u32,
        function: &mut Function,
    ) {
        let day_local = self.reserve_temp_local();
        let year_day_local = self.reserve_temp_local();
        self.emit_date_day_from_time(time_payload_local, day_local, function);
        self.emit_date_day_from_year(year_payload_local, year_day_local, function);
        function.instruction(&Instruction::LocalGet(day_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(year_day_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Sub);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(dest_payload_local));
        self.release_temp_local(year_day_local);
        self.release_temp_local(day_local);
    }
}
