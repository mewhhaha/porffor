use super::super::*;

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
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
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
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Date method receiver is not Date",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
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
