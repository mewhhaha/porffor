use super::*;

impl<'a> FunctionBuilder<'a> {
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
}
