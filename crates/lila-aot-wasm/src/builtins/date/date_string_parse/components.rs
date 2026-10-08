use super::*;

#[derive(Clone, Copy)]
pub(super) enum DateParseForm {
    Iso,
    Display { weekday: I64Local },
}

pub(super) struct DateParseComponents {
    pub(super) year: I64Local,
    pub(super) month: I64Local,
    pub(super) date: I64Local,
    pub(super) hour: I64Local,
    pub(super) minute: I64Local,
    pub(super) second: I64Local,
    pub(super) millisecond: I64Local,
    pub(super) offset: I64Local,
    local_time: I64Local,
}

impl DateParseComponents {
    pub(super) fn new(builder: &mut FunctionBuilder<'_>, function: &mut Function) -> Self {
        let parts = Self {
            year: builder.runtime_schema().reserve_i64_local(function),
            month: builder.runtime_schema().reserve_i64_local(function),
            date: builder.runtime_schema().reserve_i64_local(function),
            hour: builder.runtime_schema().reserve_i64_local(function),
            minute: builder.runtime_schema().reserve_i64_local(function),
            second: builder.runtime_schema().reserve_i64_local(function),
            millisecond: builder.runtime_schema().reserve_i64_local(function),
            offset: builder.runtime_schema().reserve_i64_local(function),
            local_time: builder.runtime_schema().reserve_i64_local(function),
        };
        // Month is zero-based for MakeDay. All absent ISO elements receive
        // their specified defaults, including reduced date-time forms.
        for local in parts.locals() {
            let initial = if local == parts.date { 1.0 } else { 0.0 };
            function.instruction(&Instruction::F64Const(Ieee64::from(initial)));
            function.instruction(&Instruction::I64ReinterpretF64);
            local.store(function);
        }
        parts
    }

    fn locals(&self) -> [I64Local; 9] {
        [
            self.year,
            self.month,
            self.date,
            self.hour,
            self.minute,
            self.second,
            self.millisecond,
            self.offset,
            self.local_time,
        ]
    }

    pub(super) fn use_local_time(&self, function: &mut Function) {
        function.instruction(&Instruction::I64Const(1));
        self.local_time.store(function);
    }

    pub(super) fn use_utc_time(&self, function: &mut Function) {
        function.instruction(&Instruction::I64Const(0));
        self.local_time.store(function);
    }

    /// Validate the written calendar date before applying the special ISO
    /// end-of-day rollover, then the UTC offset, then TimeClip. In particular,
    /// normalizing 24:00 must not normalize an invalid written calendar date.
    pub(super) fn finish(
        self,
        builder: &mut FunctionBuilder<'_>,
        cursor: &DateParseCursor,
        form: DateParseForm,
        dest: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let rollover = builder.runtime_schema().reserve_i64_local(function);
        let day = builder.runtime_schema().reserve_i64_local(function);
        let time = builder.runtime_schema().reserve_i64_local(function);
        let prepared_validation = builder.reserve_date_make_date_result(function);
        let actual: [I64Local; 7] =
            std::array::from_fn(|_| builder.runtime_schema().reserve_i64_local(function));
        function.instruction(&Instruction::I64Const(0));
        rollover.store(function);
        match form {
            DateParseForm::Iso => {
                self.hour.load(function);
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Const(Ieee64::from(24.0)));
                function.instruction(&Instruction::F64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                for local in [self.minute, self.second, self.millisecond] {
                    local.load(function);
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                    function.instruction(&Instruction::F64Eq);
                    cursor.require(function);
                }
                function.instruction(&Instruction::I64Const(1));
                rollover.store(function);
                function.instruction(&Instruction::I64Const(0));
                self.hour.store(function);
                function.instruction(&Instruction::End);
            }
            DateParseForm::Display { .. } => {}
        }
        let fields = [
            self.year,
            self.month,
            self.date,
            self.hour,
            self.minute,
            self.second,
            self.millisecond,
        ];
        let validation = builder.emit_date_make_date_from_components_into(
            prepared_validation,
            &fields,
            function,
        );
        validation.payload().load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(86_400_000.0)));
        function.instruction(&Instruction::F64Div);
        function.instruction(&Instruction::F64Floor);
        function.instruction(&Instruction::I64ReinterpretF64);
        day.store(function);
        builder.emit_date_components_from_time(
            validation.payload(),
            actual[0],
            actual[1],
            actual[2],
            actual[3],
            actual[4],
            actual[5],
            actual[6],
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
            actual.load(function);
            function.instruction(&Instruction::F64ReinterpretI64);
            expected.load(function);
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::F64Eq);
            cursor.require(function);
        }
        match form {
            DateParseForm::Iso => {}
            DateParseForm::Display { weekday } => {
                day.load(function);
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Const(Ieee64::from(4.0)));
                function.instruction(&Instruction::F64Add);
                function.instruction(&Instruction::I64ReinterpretF64);
                time.store(function);
                builder.emit_date_positive_mod(time, 7.0, function);
                weekday.load(function);
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Eq);
                cursor.require(function);
            }
        }
        cursor.require_end(function);
        cursor.valid.load(function);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        // Calendar validation precedes ISO 24:00 rollover. Re-run actual
        // MakeDate on the rolled component inputs; completed proofs are never
        // mutated to impersonate a different mathematical result.
        self.date.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        rollover.load(function);
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::I64ReinterpretF64);
        self.date.store(function);
        let prepared = builder.reserve_date_make_date_result(function);
        let date = builder.emit_date_make_date_from_components_into(prepared, &fields, function);
        self.local_time.load(function);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        let prepared_clip = builder.reserve_date_clip_result(function);
        let clipped =
            builder.emit_date_utc_time_clip_from_make_date_into(prepared_clip, &date, function)?;
        clipped.payload().load(function);
        dest.store(function);
        clipped.release(builder, function);
        function.instruction(&Instruction::Else);
        let adjusted = builder.runtime_schema().reserve_i64_local(function);
        date.payload().load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        self.offset.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Sub);
        function.instruction(&Instruction::I64ReinterpretF64);
        adjusted.store(function);
        let prepared_clip = builder.reserve_date_clip_result(function);
        let clipped = builder.emit_date_time_clip_into(prepared_clip, adjusted, function);
        clipped.payload().load(function);
        dest.store(function);
        clipped.release(builder, function);
        builder
            .runtime_schema()
            .release_i64_local(adjusted, function);
        function.instruction(&Instruction::End);
        date.release(builder, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NAN)));
        function.instruction(&Instruction::I64ReinterpretF64);
        dest.store(function);
        function.instruction(&Instruction::End);
        for local in actual.into_iter().rev() {
            builder.runtime_schema().release_i64_local(local, function);
        }
        validation.release(builder, function);
        for local in [time, day, rollover] {
            builder.runtime_schema().release_i64_local(local, function);
        }
        for local in self.locals().into_iter().rev() {
            builder.runtime_schema().release_i64_local(local, function);
        }
        Ok(())
    }
}
