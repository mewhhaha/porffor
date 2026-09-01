use super::*;

enum DateTimeValueSource {
    ReceiverSlot { payload_local: u32, tag_local: u32 },
    RealmHostClock,
}

enum DateLocalStringFormat {
    Date,
    Time,
    DateAndTime,
}

impl<'a> FunctionBuilder<'a> {
    fn emit_date_time_value_from_source(
        &mut self,
        source: DateTimeValueSource,
        dest_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match source {
            DateTimeValueSource::ReceiverSlot {
                payload_local,
                tag_local,
            } => {
                self.emit_date_value_payload(payload_local, tag_local, dest_payload_local, function)
            }
            DateTimeValueSource::RealmHostClock => {
                let wall_clock_millis_import_function_index = self
                    .functions
                    .wall_clock_millis_import_function_index()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "Date current time requires the lila_host.wall_clock_millis import",
                        )
                    })?;
                function.instruction(&Instruction::Call(wall_clock_millis_import_function_index));
                function.instruction(&Instruction::I64ReinterpretF64);
                function.instruction(&Instruction::LocalSet(dest_payload_local));
                Ok(())
            }
        }
    }

    pub(crate) fn emit_date_current_time_payload(
        &mut self,
        dest_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_date_time_value_from_source(
            DateTimeValueSource::RealmHostClock,
            dest_payload_local,
            function,
        )
    }

    pub(crate) fn emit_date_function_call(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_date_local_string(
            DateTimeValueSource::RealmHostClock,
            DateLocalStringFormat::DateAndTime,
            function,
        )
    }

    fn emit_date_local_string(
        &mut self,
        source: DateTimeValueSource,
        format: DateLocalStringFormat,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let (includes_date, includes_time) = match format {
            DateLocalStringFormat::Date => (true, false),
            DateLocalStringFormat::Time => (false, true),
            DateLocalStringFormat::DateAndTime => (true, true),
        };
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

        self.emit_date_time_value_from_source(source, time_payload_local, function)?;
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

        if includes_date {
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

        if includes_date && includes_time {
            function.instruction(&Instruction::I64Const(self.strings.payload(" ")));
            function.instruction(&Instruction::LocalSet(piece_payload_local));
            self.emit_concat_string_payloads_local(
                output_payload_local,
                piece_payload_local,
                function,
            )?;
            function.instruction(&Instruction::LocalSet(output_payload_local));
        }

        if includes_time {
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
        self.emit_date_local_string(
            DateTimeValueSource::ReceiverSlot {
                payload_local: self.this_payload_local.unwrap(),
                tag_local: self.this_tag_local.unwrap(),
            },
            DateLocalStringFormat::Date,
            function,
        )
    }

    pub(crate) fn emit_date_to_time_string(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_date_local_string(
            DateTimeValueSource::ReceiverSlot {
                payload_local: self.this_payload_local.unwrap(),
                tag_local: self.this_tag_local.unwrap(),
            },
            DateLocalStringFormat::Time,
            function,
        )
    }

    pub(crate) fn emit_date_to_string(&mut self, function: &mut Function) -> Result<(), EmitError> {
        self.emit_date_local_string(
            DateTimeValueSource::ReceiverSlot {
                payload_local: self.this_payload_local.unwrap(),
                tag_local: self.this_tag_local.unwrap(),
            },
            DateLocalStringFormat::DateAndTime,
            function,
        )
    }
}
