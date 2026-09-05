use super::*;

#[derive(Clone, Copy)]
pub(super) enum DateParseForm {
    Iso,
    Display { weekday: u32 },
}

pub(super) struct DateParseComponents {
    pub(super) year: u32,
    pub(super) month: u32,
    pub(super) date: u32,
    pub(super) hour: u32,
    pub(super) minute: u32,
    pub(super) second: u32,
    pub(super) millisecond: u32,
    pub(super) offset: u32,
}

impl DateParseComponents {
    pub(super) fn new(builder: &mut FunctionBuilder<'_>, function: &mut Function) -> Self {
        let parts = Self {
            year: builder.reserve_temp_local(),
            month: builder.reserve_temp_local(),
            date: builder.reserve_temp_local(),
            hour: builder.reserve_temp_local(),
            minute: builder.reserve_temp_local(),
            second: builder.reserve_temp_local(),
            millisecond: builder.reserve_temp_local(),
            offset: builder.reserve_temp_local(),
        };
        // Month is zero-based for MakeDay. All absent ISO elements receive
        // their specified defaults, including reduced date-time forms.
        for local in parts.locals() {
            function.instruction(&Instruction::F64Const(Ieee64::from(if local == parts.date {
                1.0
            } else {
                0.0
            }))));
            function.instruction(&Instruction::I64ReinterpretF64);
            function.instruction(&Instruction::LocalSet(local));
        }
        parts
    }

    fn locals(&self) -> [u32; 8] {
        [
            self.year,
            self.month,
            self.date,
            self.hour,
            self.minute,
            self.second,
            self.millisecond,
            self.offset,
        ]
    }

    /// Validate the written calendar date before applying the special ISO
    /// end-of-day rollover, then the UTC offset, then TimeClip. In particular,
    /// normalizing 24:00 must not normalize an invalid written calendar date.
    pub(super) fn finish(
        self,
        builder: &mut FunctionBuilder<'_>,
        cursor: &DateParseCursor,
        form: DateParseForm,
        dest: u32,
        function: &mut Function,
    ) {
        let rollover = builder.reserve_temp_local();
        let day = builder.reserve_temp_local();
        let time = builder.reserve_temp_local();
        let parsed = builder.reserve_temp_local();
        let actual: [u32; 7] = std::array::from_fn(|_| builder.reserve_temp_local());
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(rollover));
        match form {
            DateParseForm::Iso => {
                function.instruction(&Instruction::LocalGet(self.hour));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Const(Ieee64::from(24.0)));
                function.instruction(&Instruction::F64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                for local in [self.minute, self.second, self.millisecond] {
                    function.instruction(&Instruction::LocalGet(local));
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                    function.instruction(&Instruction::F64Eq);
                    cursor.require(function);
                }
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(rollover));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(self.hour));
                function.instruction(&Instruction::End);
            }
            DateParseForm::Display { .. } => {}
        }
        builder.emit_date_make_day(self.year, self.month, self.date, day, function);
        builder.emit_date_make_time(
            self.hour,
            self.minute,
            self.second,
            self.millisecond,
            time,
            function,
        );
        function.instruction(&Instruction::LocalGet(day));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(86_400_000.0)));
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::LocalGet(time));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(parsed));
        builder.emit_date_components_from_time(
            parsed, actual[0], actual[1], actual[2], actual[3], actual[4], actual[5], actual[6],
            function,
        );
        for (actual, expected) in actual.into_iter().zip([
            self.year,
            self.month,
            self.date,
            self.hour,
            self.minute,
            self.second,
            self.millisecond,
        ]) {
            function.instruction(&Instruction::LocalGet(actual));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::LocalGet(expected));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::F64Eq);
            cursor.require(function);
        }
        match form {
            DateParseForm::Iso => {}
            DateParseForm::Display { weekday } => {
                function.instruction(&Instruction::LocalGet(day));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Const(Ieee64::from(4.0)));
                function.instruction(&Instruction::F64Add);
                function.instruction(&Instruction::I64ReinterpretF64);
                function.instruction(&Instruction::LocalSet(time));
                builder.emit_date_positive_mod(time, 7.0, function);
                function.instruction(&Instruction::LocalGet(weekday));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Eq);
                cursor.require(function);
            }
        }
        cursor.require_end(function);
        function.instruction(&Instruction::LocalGet(cursor.valid));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(parsed));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(rollover));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::F64Const(Ieee64::from(86_400_000.0)));
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::LocalGet(self.offset));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Sub);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(parsed));
        builder.emit_date_time_clip(parsed, dest, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NAN)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(dest));
        function.instruction(&Instruction::End);
        for local in actual.into_iter().rev() {
            builder.release_temp_local(local);
        }
        for local in [parsed, time, day, rollover] {
            builder.release_temp_local(local);
        }
        for local in self.locals().into_iter().rev() {
            builder.release_temp_local(local);
        }
    }
}
