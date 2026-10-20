//! Closed local/UTC Date dispatch retains the captured receiver before hooks.

use super::*;

#[derive(Clone, Copy)]
pub(in crate::builtins) enum DateTimeBasis {
    Local,
    Utc,
}

#[derive(Clone, Copy)]
pub(in crate::builtins) enum DateComponentGetter {
    FullYear(DateTimeBasis),
    Month(DateTimeBasis),
    Date(DateTimeBasis),
    Day(DateTimeBasis),
    Hours(DateTimeBasis),
    Minutes(DateTimeBasis),
    Seconds(DateTimeBasis),
    Milliseconds(DateTimeBasis),
    Year,
    TimezoneOffset,
}

impl DateComponentGetter {
    fn basis(self) -> DateTimeBasis {
        match self {
            Self::FullYear(basis)
            | Self::Month(basis)
            | Self::Date(basis)
            | Self::Day(basis)
            | Self::Hours(basis)
            | Self::Minutes(basis)
            | Self::Seconds(basis)
            | Self::Milliseconds(basis) => basis,
            Self::Year | Self::TimezoneOffset => DateTimeBasis::Local,
        }
    }
}

#[derive(Clone, Copy)]
pub(in crate::builtins) enum DateComponentSetter {
    FullYear(DateTimeBasis),
    Month(DateTimeBasis),
    Date(DateTimeBasis),
    Hours(DateTimeBasis),
    Minutes(DateTimeBasis),
    Seconds(DateTimeBasis),
    Milliseconds(DateTimeBasis),
    Year,
}

impl DateComponentSetter {
    fn basis(self) -> DateTimeBasis {
        match self {
            Self::FullYear(basis)
            | Self::Month(basis)
            | Self::Date(basis)
            | Self::Hours(basis)
            | Self::Minutes(basis)
            | Self::Seconds(basis)
            | Self::Milliseconds(basis) => basis,
            Self::Year => DateTimeBasis::Local,
        }
    }
    fn replaced_fields(self) -> &'static [usize] {
        match self {
            Self::FullYear(_) => &[0, 1, 2],
            Self::Month(_) => &[1, 2],
            Self::Date(_) => &[2],
            Self::Hours(_) => &[3, 4, 5, 6],
            Self::Minutes(_) => &[4, 5, 6],
            Self::Seconds(_) => &[5, 6],
            Self::Milliseconds(_) => &[6],
            Self::Year => &[0],
        }
    }
}

impl FunctionBuilder<'_> {
    pub(in crate::builtins) fn emit_date_component_getter(
        &mut self,
        operation: DateComponentGetter,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let receiver = schema.reserve_value_local(function);
        let result = schema.reserve_value_local(function);
        let coordinate = schema.reserve_i64_local(function);
        let bits = schema.reserve_i64_local(function);
        let fields: [I64Local; 7] = std::array::from_fn(|_| schema.reserve_i64_local(function));
        self.compile_this_to_locals(&receiver, function)?;
        let captured = self.emit_date_capture_value(&receiver, function)?;
        captured.emit_valid_branch(
            self,
            function,
            |builder, function, finite| {
                match operation.basis() {
                    DateTimeBasis::Local => {
                        let projection = builder.emit_date_local_projection(finite, function)?;
                        if matches!(operation, DateComponentGetter::TimezoneOffset) {
                            projection.offset_seconds().load(function);
                            function.instruction(&Instruction::F64ConvertI64S);
                            function.instruction(&Instruction::F64Neg);
                            function.instruction(&Instruction::F64Const(Ieee64::from(60.0)));
                            function.instruction(&Instruction::F64Div);
                            // Normalize a zero offset to positive zero; retain
                            // fractional historical offsets expressed in seconds.
                            function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                            function.instruction(&Instruction::F64Add);
                            function.instruction(&Instruction::I64ReinterpretF64);
                            bits.store(function);
                        } else {
                            projection.local_payload().load(function);
                            coordinate.store(function);
                        }
                        projection.release(builder, function);
                    }
                    DateTimeBasis::Utc => {
                        finite.payload().load(function);
                        coordinate.store(function);
                    }
                }
                if !matches!(operation, DateComponentGetter::TimezoneOffset) {
                    builder.emit_date_components_from_time(
                        coordinate, fields[0], fields[1], fields[2], fields[3], fields[4],
                        fields[5], fields[6], function,
                    );
                    match operation {
                        DateComponentGetter::FullYear(_) => fields[0].load(function),
                        DateComponentGetter::Month(_) => fields[1].load(function),
                        DateComponentGetter::Date(_) => fields[2].load(function),
                        DateComponentGetter::Hours(_) => fields[3].load(function),
                        DateComponentGetter::Minutes(_) => fields[4].load(function),
                        DateComponentGetter::Seconds(_) => fields[5].load(function),
                        DateComponentGetter::Milliseconds(_) => fields[6].load(function),
                        DateComponentGetter::Year => {
                            fields[0].load(function);
                            function.instruction(&Instruction::F64ReinterpretI64);
                            function.instruction(&Instruction::F64Const(Ieee64::from(1900.0)));
                            function.instruction(&Instruction::F64Sub);
                            function.instruction(&Instruction::I64ReinterpretF64);
                        }
                        DateComponentGetter::Day(_) => {
                            builder.emit_date_day_from_time(coordinate, fields[0], function);
                            fields[0].load(function);
                            function.instruction(&Instruction::F64ReinterpretI64);
                            function.instruction(&Instruction::F64Const(Ieee64::from(4.0)));
                            function.instruction(&Instruction::F64Add);
                            function.instruction(&Instruction::I64ReinterpretF64);
                            fields[0].store(function);
                            builder.emit_date_positive_mod(fields[0], 7.0, function);
                            function.instruction(&Instruction::I64ReinterpretF64);
                        }
                        DateComponentGetter::TimezoneOffset => {}
                    }
                    bits.store(function);
                }
                Ok(())
            },
            |_builder, function| {
                captured.payload().load(function);
                bits.store(function);
                Ok(())
            },
        )?;
        result.set_number(bits, function);
        self.completion().set_normal(&result, function);
        captured.release(self, function);
        for field in fields.into_iter().rev() {
            schema.release_i64_local(field, function);
        }
        schema.release_i64_local(bits, function);
        schema.release_i64_local(coordinate, function);
        result.clear(function);
        receiver.clear(function);
        Ok(())
    }

    fn emit_date_setter_number_argument(
        &mut self,
        index: usize,
        destination: I64Local,
        value: &ValueLocals,
        pending: &crate::gc_types::CompletionLocals,
        output: &crate::gc_types::CompletionLocals,
        exit: ControlTarget,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_builtin_arg_to_value(index, value, function);
        self.emit_date_number_coercion(value, destination, pending, output, exit, function)
    }

    /// Recovery zero belongs to local arithmetic; it cannot mint a finite UTC
    /// Date slot and accidentally receive LocalTime(+0).
    fn emit_date_setter_coordinate(
        &mut self,
        captured: &CapturedDateValueLocals,
        operation: DateComponentSetter,
        coordinate: I64Local,
        value: &ValueLocals,
        output: &crate::gc_types::CompletionLocals,
        exit: ControlTarget,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        captured.emit_valid_branch(
            self,
            function,
            |builder, function, finite| {
                match operation.basis() {
                    DateTimeBasis::Local => {
                        let projection = builder.emit_date_local_projection(finite, function)?;
                        projection.local_payload().load(function);
                        coordinate.store(function);
                        projection.release(builder, function);
                    }
                    DateTimeBasis::Utc => {
                        finite.payload().load(function);
                        coordinate.store(function);
                    }
                }
                Ok(())
            },
            |builder, function| {
                match operation {
                    DateComponentSetter::FullYear(_) | DateComponentSetter::Year => {
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        coordinate.store(function);
                    }
                    DateComponentSetter::Month(_)
                    | DateComponentSetter::Date(_)
                    | DateComponentSetter::Hours(_)
                    | DateComponentSetter::Minutes(_)
                    | DateComponentSetter::Seconds(_)
                    | DateComponentSetter::Milliseconds(_) => {
                        // No write: argument hooks may already have mutated the
                        // retained receiver, and captured NaN does not erase it.
                        value.set_number(captured.payload(), function);
                        output.set_normal(value, function);
                        builder.emit_branch_to_target(exit, function);
                    }
                }
                Ok(())
            },
        )
    }

    pub(in crate::builtins) fn emit_date_component_setter(
        &mut self,
        operation: DateComponentSetter,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let receiver = schema.reserve_value_local(function);
        let value = schema.reserve_value_local(function);
        let pending = schema.reserve_completion(function);
        let output = schema.reserve_completion(function);
        output.initialize(function);
        let coordinate = schema.reserve_i64_local(function);
        let fields: [I64Local; 7] = std::array::from_fn(|_| schema.reserve_i64_local(function));
        let supplied: [I64Local; 4] = std::array::from_fn(|_| schema.reserve_i64_local(function));
        self.compile_this_to_locals(&receiver, function)?;
        let captured = self.emit_date_capture_value(&receiver, function)?;
        let cleanup = self.open_frame(ControlFrameKind::Block, function);
        let replaced = operation.replaced_fields();
        match operation {
            DateComponentSetter::FullYear(DateTimeBasis::Local) => {
                self.emit_date_setter_number_argument(
                    0,
                    supplied[0],
                    &value,
                    &pending,
                    &output,
                    cleanup,
                    function,
                )?;
                self.emit_date_setter_coordinate(
                    &captured, operation, coordinate, &value, &output, cleanup, function,
                )?;
            }
            DateComponentSetter::FullYear(DateTimeBasis::Utc) | DateComponentSetter::Year => {
                self.emit_date_setter_coordinate(
                    &captured, operation, coordinate, &value, &output, cleanup, function,
                )?;
                self.emit_date_setter_number_argument(
                    0,
                    supplied[0],
                    &value,
                    &pending,
                    &output,
                    cleanup,
                    function,
                )?;
                if matches!(operation, DateComponentSetter::Year) {
                    self.emit_date_make_full_year(supplied[0], supplied[0], function);
                }
            }
            DateComponentSetter::Month(_)
            | DateComponentSetter::Date(_)
            | DateComponentSetter::Hours(_)
            | DateComponentSetter::Minutes(_)
            | DateComponentSetter::Seconds(_)
            | DateComponentSetter::Milliseconds(_) => {
                for index in 0..replaced.len() {
                    if index > 0 {
                        self.emit_builtin_arg_is_present_i32(index, function);
                        self.open_frame(ControlFrameKind::If, function);
                    }
                    self.emit_date_setter_number_argument(
                        index,
                        supplied[index],
                        &value,
                        &pending,
                        &output,
                        cleanup,
                        function,
                    )?;
                    if index > 0 {
                        self.pop_control(ControlFrameKind::If);
                        function.instruction(&Instruction::End);
                    }
                }
                self.emit_date_setter_coordinate(
                    &captured, operation, coordinate, &value, &output, cleanup, function,
                )?;
            }
        }
        self.emit_date_components_from_time(
            coordinate, fields[0], fields[1], fields[2], fields[3], fields[4], fields[5],
            fields[6], function,
        );
        for (index, field) in replaced.iter().copied().enumerate() {
            if index > 0 {
                self.emit_builtin_arg_is_present_i32(index, function);
                self.open_frame(ControlFrameKind::If, function);
            }
            if index > 0 && matches!(operation, DateComponentSetter::FullYear(_)) {
                self.emit_date_setter_number_argument(
                    index,
                    supplied[index],
                    &value,
                    &pending,
                    &output,
                    cleanup,
                    function,
                )?;
            }
            supplied[index].load(function);
            fields[field].store(function);
            if index > 0 {
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
            }
        }
        let prepared_date = self.reserve_date_make_date_result(function);
        let date = self.emit_date_make_date_from_components_into(prepared_date, &fields, function);
        let prepared_clip = self.reserve_date_clip_result(function);
        let clip = match operation.basis() {
            DateTimeBasis::Local => {
                self.emit_date_utc_time_clip_from_make_date_into(prepared_clip, &date, function)?
            }
            DateTimeBasis::Utc => {
                self.emit_date_time_clip_into(prepared_clip, date.payload(), function)
            }
        };
        self.emit_date_store_clip(&captured, &clip, function);
        value.set_number(clip.payload(), function);
        output.set_normal(&value, function);
        clip.release(self, function);
        date.release(self, function);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.completion().copy_from(&output, function);
        captured.release(self, function);
        for supplied in supplied.into_iter().rev() {
            schema.release_i64_local(supplied, function);
        }
        for field in fields.into_iter().rev() {
            schema.release_i64_local(field, function);
        }
        schema.release_i64_local(coordinate, function);
        output.clear(function);
        pending.clear(function);
        value.clear(function);
        receiver.clear(function);
        Ok(())
    }
}
