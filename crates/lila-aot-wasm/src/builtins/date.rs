//! Date retains a typed private time record through every observable hook.
use super::super::*;
use crate::functions::OrdinaryDefaultPrototype;
use crate::gc_types::{
    CodeUnitArray, DateObject, DateObjectSchema, GcLocal, GcNullability, GcOperand, I64Local,
    Nullable, ScalarValue, StringValue, StringValueSchema, ValueLocals,
};
use crate::intrinsics::temporal::TemporalPrototypeSource;

mod components;
mod constructor;
mod date_string_parse;
pub(in crate::builtins) use components::{DateComponentGetter, DateComponentSetter, DateTimeBasis};
mod zone;
mod zone_data;
pub(in crate::builtins) use zone::{DateLocalCoordinateLocals, DateUtcCoordinateLocals};
mod local_string;
mod locale_string;
pub(crate) use locale_string::DateLocaleFormat;

pub(in crate::builtins) struct PreparedDateClipLocals {
    payload: I64Local,
}
/// Only the TimeClip emitter can construct this storage proof.
pub(in crate::builtins) struct CompletedDateClipLocals {
    payload: I64Local,
}
pub(in crate::builtins) struct PreparedDateMakeDateLocals {
    payload: I64Local,
}
pub(in crate::builtins) struct CompletedDateMakeDateLocals {
    payload: I64Local,
}
struct BrandedDateRecordLocals {
    object: GcLocal<DateObject>,
}
pub(in crate::builtins) struct CapturedDateValueLocals {
    record: BrandedDateRecordLocals,
    payload: I64Local,
}
/// Borrowed only inside the actual finite-time branch.
pub(in crate::builtins) struct FiniteDateValueLocals {
    payload: I64Local,
}

impl FiniteDateValueLocals {
    pub(in crate::builtins) fn payload(&self) -> I64Local {
        self.payload
    }
}
impl CompletedDateMakeDateLocals {
    pub(in crate::builtins) fn payload(&self) -> I64Local {
        self.payload
    }
    pub(in crate::builtins) fn release(
        self,
        builder: &mut FunctionBuilder<'_>,
        function: &mut Function,
    ) {
        builder
            .runtime_schema()
            .release_i64_local(self.payload, function);
    }
}
impl CompletedDateClipLocals {
    pub(in crate::builtins) fn payload(&self) -> I64Local {
        self.payload
    }
    pub(in crate::builtins) fn release(
        self,
        builder: &mut FunctionBuilder<'_>,
        function: &mut Function,
    ) {
        builder
            .runtime_schema()
            .release_i64_local(self.payload, function);
    }
    pub(in crate::builtins) fn emit_valid_branch(
        &self,
        builder: &mut FunctionBuilder<'_>,
        function: &mut Function,
        valid: impl FnOnce(
            &mut FunctionBuilder<'_>,
            &mut Function,
            &FiniteDateValueLocals,
        ) -> Result<(), EmitError>,
        invalid: impl FnOnce(&mut FunctionBuilder<'_>, &mut Function) -> Result<(), EmitError>,
    ) -> Result<(), EmitError> {
        builder.emit_date_finite_branch(self.payload, function, valid, invalid)
    }
}
impl CapturedDateValueLocals {
    pub(in crate::builtins) fn payload(&self) -> I64Local {
        self.payload
    }
    pub(in crate::builtins) fn release(
        self,
        builder: &mut FunctionBuilder<'_>,
        function: &mut Function,
    ) {
        builder
            .runtime_schema()
            .release_i64_local(self.payload, function);
        self.record.object.clear(function);
    }
    pub(in crate::builtins) fn emit_valid_branch(
        &self,
        builder: &mut FunctionBuilder<'_>,
        function: &mut Function,
        valid: impl FnOnce(
            &mut FunctionBuilder<'_>,
            &mut Function,
            &FiniteDateValueLocals,
        ) -> Result<(), EmitError>,
        invalid: impl FnOnce(&mut FunctionBuilder<'_>, &mut Function) -> Result<(), EmitError>,
    ) -> Result<(), EmitError> {
        builder.emit_date_finite_branch(self.payload, function, valid, invalid)
    }
}

impl FunctionBuilder<'_> {
    pub(crate) fn emit_date_value_of_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let receiver = schema.reserve_value_local(function);
        let value = schema.reserve_value_local(function);
        self.compile_this_to_locals(&receiver, function)?;
        let captured = self.emit_date_capture_value(&receiver, function)?;
        value.set_number(captured.payload(), function);
        self.completion().set_normal(&value, function);
        captured.release(self, function);
        value.clear(function);
        receiver.clear(function);
        Ok(())
    }
    pub(crate) fn emit_date_now(&mut self, function: &mut Function) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let bits = schema.reserve_i64_local(function);
        let value = schema.reserve_value_local(function);
        self.emit_date_current_time_payload(bits, function)?;
        value.set_number(bits, function);
        self.completion().set_normal(&value, function);
        value.clear(function);
        schema.release_i64_local(bits, function);
        Ok(())
    }
    fn emit_date_finite_branch(
        &mut self,
        payload: I64Local,
        function: &mut Function,
        valid: impl FnOnce(
            &mut FunctionBuilder<'_>,
            &mut Function,
            &FiniteDateValueLocals,
        ) -> Result<(), EmitError>,
        invalid: impl FnOnce(&mut FunctionBuilder<'_>, &mut Function) -> Result<(), EmitError>,
    ) -> Result<(), EmitError> {
        payload.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        payload.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Eq);
        self.open_frame(ControlFrameKind::If, function);
        valid(self, function, &FiniteDateValueLocals { payload })?;
        function.instruction(&Instruction::Else);
        invalid(self, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }
    /// RefTest establishes the private brand without Get, Proxy traps or
    /// public marker properties. The NN record exists only in the present arm.
    fn emit_date_record_branch(
        &mut self,
        value: &ValueLocals,
        function: &mut Function,
        present: impl FnOnce(
            &mut FunctionBuilder<'_>,
            &mut Function,
            &BrandedDateRecordLocals,
        ) -> Result<(), EmitError>,
        absent: impl FnOnce(&mut FunctionBuilder<'_>, &mut Function) -> Result<(), EmitError>,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        value.reference().load(function);
        function.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<DateObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        self.open_frame(ControlFrameKind::If, function);
        let record = BrandedDateRecordLocals {
            object: schema.reserve_gc_local(function).initialize(
                value.cast_reference::<DateObject>(schema, function),
                function,
            ),
        };
        present(self, function, &record)?;
        record.object.clear(function);
        function.instruction(&Instruction::Else);
        absent(self, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }
    fn emit_date_record_value_into(
        &self,
        record: &BrandedDateRecordLocals,
        destination: I64Local,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let number = schema.reserve_f64_local(function);
        schema
            .struct_type::<DateObject>()
            .field(DateObjectSchema::TIME_VALUE)
            .read(&record.object, schema, function)
            .store_f64(number, function);
        number.load(function);
        function.instruction(&Instruction::I64ReinterpretF64);
        destination.store(function);
        schema.release_f64_local(number, function);
    }
    pub(in crate::builtins) fn emit_date_capture_value(
        &mut self,
        value: &ValueLocals,
        function: &mut Function,
    ) -> Result<CapturedDateValueLocals, EmitError> {
        let schema = self.runtime_schema();
        let object = schema
            .reserve_gc_local::<DateObject, Nullable>(function)
            .initialize_null(schema, function);
        let bits = schema.reserve_i64_local(function);
        self.emit_date_record_branch(
            value,
            function,
            |builder, function, record| {
                object.replace(record.object.load(schema, function).nullable(), function);
                builder.emit_date_record_value_into(record, bits, function);
                Ok(())
            },
            |builder, function| {
                let error = schema.reserve_completion(function);
                builder.emit_throw_current_function_realm_type_error(
                    RuntimeErrorMessage::DATE_METHOD_RECEIVER_IS_NOT_DATE,
                    &error,
                    function,
                )?;
                builder.completion().copy_from(&error, function);
                error.clear(function);
                builder.emit_propagate_current_throw(function);
                Ok(())
            },
        )?;
        let retained = schema.reserve_gc_local(function).initialize(
            object.load(schema, function).require_non_null(function),
            function,
        );
        object.clear(function);
        Ok(CapturedDateValueLocals {
            record: BrandedDateRecordLocals { object: retained },
            payload: bits,
        })
    }
    pub(crate) fn emit_date_value_payload(
        &mut self,
        value: &ValueLocals,
        destination: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let captured = self.emit_date_capture_value(value, function)?;
        captured.payload().load(function);
        destination.store(function);
        captured.release(self, function);
        Ok(())
    }
    fn emit_date_store_clip(
        &self,
        captured: &CapturedDateValueLocals,
        clip: &CompletedDateClipLocals,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let number = schema.reserve_f64_local(function);
        clip.payload().load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        number.store(function);
        schema
            .struct_type::<DateObject>()
            .field(DateObjectSchema::TIME_VALUE)
            .write(
                &captured.record.object,
                GcOperand::f64_local(number),
                schema,
                function,
            );
        schema.release_f64_local(number, function);
    }
    pub(in crate::builtins) fn reserve_date_clip_result(
        &mut self,
        function: &mut Function,
    ) -> PreparedDateClipLocals {
        PreparedDateClipLocals {
            payload: self.runtime_schema().reserve_i64_local(function),
        }
    }
    pub(in crate::builtins) fn emit_date_time_clip_into(
        &mut self,
        prepared: PreparedDateClipLocals,
        number_payload: I64Local,
        function: &mut Function,
    ) -> CompletedDateClipLocals {
        self.emit_date_time_clip(number_payload, prepared.payload, function);
        CompletedDateClipLocals {
            payload: prepared.payload,
        }
    }
    pub(in crate::builtins) fn reserve_date_make_date_result(
        &mut self,
        function: &mut Function,
    ) -> PreparedDateMakeDateLocals {
        PreparedDateMakeDateLocals {
            payload: self.runtime_schema().reserve_i64_local(function),
        }
    }
    pub(in crate::builtins) fn emit_date_make_date_from_components_into(
        &mut self,
        prepared: PreparedDateMakeDateLocals,
        fields: &[I64Local; 7],
        function: &mut Function,
    ) -> CompletedDateMakeDateLocals {
        let time = self.runtime_schema().reserve_i64_local(function);
        self.emit_date_make_day(fields[0], fields[1], fields[2], prepared.payload, function);
        self.emit_date_make_time(fields[3], fields[4], fields[5], fields[6], time, function);
        prepared.payload.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(86_400_000.0)));
        function.instruction(&Instruction::F64Mul);
        time.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::I64ReinterpretF64);
        prepared.payload.store(function);
        self.runtime_schema().release_i64_local(time, function);
        CompletedDateMakeDateLocals {
            payload: prepared.payload,
        }
    }
    fn emit_date_make_full_year(
        &mut self,
        source: I64Local,
        destination: I64Local,
        function: &mut Function,
    ) {
        source.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Abs);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::If(BlockType::Empty));
        source.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::I64ReinterpretF64);
        destination.store(function);
        destination.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Ge);
        destination.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(99.0)));
        function.instruction(&Instruction::F64Le);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        destination.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(1900.0)));
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::I64ReinterpretF64);
        destination.store(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NAN)));
        function.instruction(&Instruction::I64ReinterpretF64);
        destination.store(function);
        function.instruction(&Instruction::End);
    }

    pub(crate) fn emit_date_time_clip(
        &mut self,
        input_payload_local: I64Local,
        dest_payload_local: I64Local,
        function: &mut Function,
    ) {
        input_payload_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        input_payload_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        input_payload_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(
            8_640_000_000_000_000.0,
        )));
        function.instruction(&Instruction::F64Gt);
        function.instruction(&Instruction::I32Or);
        input_payload_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(
            -8_640_000_000_000_000.0,
        )));
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Result(ValType::F64)));
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NAN)));
        function.instruction(&Instruction::Else);
        input_payload_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::I64ReinterpretF64);
        dest_payload_local.store(function);
    }

    pub(crate) fn emit_date_day_from_year(
        &mut self,
        year_payload_local: I64Local,
        dest_payload_local: I64Local,
        function: &mut Function,
    ) {
        year_payload_local.load(function);
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
            year_payload_local.load(function);
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
        dest_payload_local.store(function);
    }

    pub(crate) fn emit_date_is_leap_year(
        &mut self,
        year_payload_local: I64Local,
        function: &mut Function,
    ) {
        let div4_local = self.runtime_schema().reserve_i64_local(function);
        let div100_local = self.runtime_schema().reserve_i64_local(function);
        let div400_local = self.runtime_schema().reserve_i64_local(function);
        for (divisor, destination) in [
            (4.0, div4_local),
            (100.0, div100_local),
            (400.0, div400_local),
        ] {
            year_payload_local.load(function);
            function.instruction(&Instruction::F64ReinterpretI64);
            year_payload_local.load(function);
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::F64Const(Ieee64::from(divisor)));
            function.instruction(&Instruction::F64Div);
            function.instruction(&Instruction::F64Floor);
            function.instruction(&Instruction::F64Const(Ieee64::from(divisor)));
            function.instruction(&Instruction::F64Mul);
            function.instruction(&Instruction::F64Eq);
            function.instruction(&Instruction::I64ExtendI32U);
            destination.store(function);
        }
        div4_local.load(function);
        function.instruction(&Instruction::I32WrapI64);
        div100_local.load(function);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Eqz);
        div400_local.load(function);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32And);
        self.runtime_schema()
            .release_i64_local(div400_local, function);
        self.runtime_schema()
            .release_i64_local(div100_local, function);
        self.runtime_schema()
            .release_i64_local(div4_local, function);
    }

    pub(crate) fn emit_date_month_day(
        &mut self,
        year_payload_local: I64Local,
        month_payload_local: I64Local,
        dest_payload_local: I64Local,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::I64ReinterpretF64);
        dest_payload_local.store(function);
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
            month_payload_local.load(function);
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
            dest_payload_local.store(function);
            function.instruction(&Instruction::End);
        }
    }

    pub(crate) fn emit_date_month_from_day_within_year(
        &mut self,
        year_payload_local: I64Local,
        day_payload_local: I64Local,
        dest_payload_local: I64Local,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::I64ReinterpretF64);
        dest_payload_local.store(function);
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
            day_payload_local.load(function);
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
            dest_payload_local.store(function);
            function.instruction(&Instruction::End);
        }
    }

    pub(crate) fn emit_date_make_day(
        &mut self,
        year_payload_local: I64Local,
        month_payload_local: I64Local,
        date_payload_local: I64Local,
        dest_payload_local: I64Local,
        function: &mut Function,
    ) {
        let ym_local = self.runtime_schema().reserve_i64_local(function);
        let mn_local = self.runtime_schema().reserve_i64_local(function);
        let month_int_local = self.runtime_schema().reserve_i64_local(function);
        let day_from_year_local = self.runtime_schema().reserve_i64_local(function);
        let month_day_local = self.runtime_schema().reserve_i64_local(function);

        month_payload_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::I64ReinterpretF64);
        month_int_local.store(function);

        year_payload_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        month_int_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(12.0)));
        function.instruction(&Instruction::F64Div);
        function.instruction(&Instruction::F64Floor);
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::I64ReinterpretF64);
        ym_local.store(function);

        month_int_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        month_int_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(12.0)));
        function.instruction(&Instruction::F64Div);
        function.instruction(&Instruction::F64Floor);
        function.instruction(&Instruction::F64Const(Ieee64::from(12.0)));
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::F64Sub);
        function.instruction(&Instruction::I64ReinterpretF64);
        mn_local.store(function);

        self.emit_date_day_from_year(ym_local, day_from_year_local, function);
        self.emit_date_month_day(ym_local, mn_local, month_day_local, function);
        day_from_year_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        month_day_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Add);
        date_payload_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
        function.instruction(&Instruction::F64Sub);
        function.instruction(&Instruction::I64ReinterpretF64);
        dest_payload_local.store(function);

        self.runtime_schema()
            .release_i64_local(month_day_local, function);
        self.runtime_schema()
            .release_i64_local(day_from_year_local, function);
        self.runtime_schema()
            .release_i64_local(month_int_local, function);
        self.runtime_schema().release_i64_local(mn_local, function);
        self.runtime_schema().release_i64_local(ym_local, function);
    }

    pub(crate) fn emit_date_year_from_time(
        &mut self,
        time_payload_local: I64Local,
        dest_payload_local: I64Local,
        function: &mut Function,
    ) {
        let day_local = self.runtime_schema().reserve_i64_local(function);
        let year_local = self.runtime_schema().reserve_i64_local(function);
        let year_day_local = self.runtime_schema().reserve_i64_local(function);
        let next_year_local = self.runtime_schema().reserve_i64_local(function);
        let done_local = self.runtime_schema().reserve_i64_local(function);

        time_payload_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(86_400_000.0)));
        function.instruction(&Instruction::F64Div);
        function.instruction(&Instruction::F64Floor);
        function.instruction(&Instruction::I64ReinterpretF64);
        day_local.store(function);

        day_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(365.2425)));
        function.instruction(&Instruction::F64Div);
        function.instruction(&Instruction::F64Floor);
        function.instruction(&Instruction::F64Const(Ieee64::from(1970.0)));
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::I64ReinterpretF64);
        year_local.store(function);

        function.instruction(&Instruction::I64Const(0));
        done_local.store(function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        done_local.load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::BrIf(1));
        self.emit_date_day_from_year(year_local, year_day_local, function);
        day_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        year_day_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::If(BlockType::Empty));
        year_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
        function.instruction(&Instruction::F64Sub);
        function.instruction(&Instruction::I64ReinterpretF64);
        year_local.store(function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        year_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::I64ReinterpretF64);
        next_year_local.store(function);
        self.emit_date_day_from_year(next_year_local, year_day_local, function);
        day_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        year_day_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ge);
        function.instruction(&Instruction::If(BlockType::Empty));
        next_year_local.load(function);
        year_local.store(function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        done_local.store(function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        year_local.load(function);
        dest_payload_local.store(function);

        self.runtime_schema()
            .release_i64_local(done_local, function);
        self.runtime_schema()
            .release_i64_local(next_year_local, function);
        self.runtime_schema()
            .release_i64_local(year_day_local, function);
        self.runtime_schema()
            .release_i64_local(year_local, function);
        self.runtime_schema().release_i64_local(day_local, function);
    }

    pub(crate) fn emit_date_day_from_time(
        &mut self,
        time_payload_local: I64Local,
        dest_payload_local: I64Local,
        function: &mut Function,
    ) {
        time_payload_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(86_400_000.0)));
        function.instruction(&Instruction::F64Div);
        function.instruction(&Instruction::F64Floor);
        function.instruction(&Instruction::I64ReinterpretF64);
        dest_payload_local.store(function);
    }

    pub(crate) fn emit_date_positive_mod(
        &mut self,
        value_payload_local: I64Local,
        modulo: f64,
        function: &mut Function,
    ) {
        value_payload_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        value_payload_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(modulo)));
        function.instruction(&Instruction::F64Div);
        function.instruction(&Instruction::F64Floor);
        function.instruction(&Instruction::F64Const(Ieee64::from(modulo)));
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::F64Sub);
    }

    pub(crate) fn emit_date_make_time(
        &mut self,
        hour_payload_local: I64Local,
        minute_payload_local: I64Local,
        second_payload_local: I64Local,
        ms_payload_local: I64Local,
        dest_payload_local: I64Local,
        function: &mut Function,
    ) {
        for (local, scale, first) in [
            (hour_payload_local, 3_600_000.0, true),
            (minute_payload_local, 60_000.0, false),
            (second_payload_local, 1_000.0, false),
            (ms_payload_local, 1.0, false),
        ] {
            local.load(function);
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::F64Trunc);
            function.instruction(&Instruction::F64Const(Ieee64::from(scale)));
            function.instruction(&Instruction::F64Mul);
            if !first {
                function.instruction(&Instruction::F64Add);
            }
        }
        function.instruction(&Instruction::I64ReinterpretF64);
        dest_payload_local.store(function);
    }

    pub(crate) fn emit_date_components_from_time(
        &mut self,
        time_payload_local: I64Local,
        year_payload_local: I64Local,
        month_payload_local: I64Local,
        date_payload_local: I64Local,
        hour_payload_local: I64Local,
        minute_payload_local: I64Local,
        second_payload_local: I64Local,
        ms_payload_local: I64Local,
        function: &mut Function,
    ) {
        let day_payload_local = self.runtime_schema().reserve_i64_local(function);
        let month_day_payload_local = self.runtime_schema().reserve_i64_local(function);

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
        day_payload_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        month_day_payload_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Sub);
        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::I64ReinterpretF64);
        date_payload_local.store(function);

        self.emit_date_positive_mod(time_payload_local, 86_400_000.0, function);
        function.instruction(&Instruction::F64Const(Ieee64::from(3_600_000.0)));
        function.instruction(&Instruction::F64Div);
        function.instruction(&Instruction::F64Floor);
        function.instruction(&Instruction::I64ReinterpretF64);
        hour_payload_local.store(function);
        self.emit_date_positive_mod(time_payload_local, 3_600_000.0, function);
        function.instruction(&Instruction::F64Const(Ieee64::from(60_000.0)));
        function.instruction(&Instruction::F64Div);
        function.instruction(&Instruction::F64Floor);
        function.instruction(&Instruction::I64ReinterpretF64);
        minute_payload_local.store(function);
        self.emit_date_positive_mod(time_payload_local, 60_000.0, function);
        function.instruction(&Instruction::F64Const(Ieee64::from(1_000.0)));
        function.instruction(&Instruction::F64Div);
        function.instruction(&Instruction::F64Floor);
        function.instruction(&Instruction::I64ReinterpretF64);
        second_payload_local.store(function);
        self.emit_date_positive_mod(time_payload_local, 1_000.0, function);
        function.instruction(&Instruction::I64ReinterpretF64);
        ms_payload_local.store(function);

        self.runtime_schema()
            .release_i64_local(month_day_payload_local, function);
        self.runtime_schema()
            .release_i64_local(day_payload_local, function);
    }

    fn emit_date_append_literal(
        &self,
        output: &GcLocal<StringValue>,
        text: &str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let piece = schema.reserve_gc_local(function).initialize(
            self.emit_interned_string_reference(text, function)?,
            function,
        );
        output.replace(
            self.emit_concat_gc_strings(output, &piece, function),
            function,
        );
        piece.clear(function);
        Ok(())
    }

    pub(crate) fn emit_date_append_padded_decimal(
        &mut self,
        output: &GcLocal<StringValue>,
        number: I64Local,
        minimum_width: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        for remaining in (1..minimum_width).rev() {
            number.load(function);
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::F64Const(Ieee64::from(
                10_f64.powi(remaining as i32),
            )));
            function.instruction(&Instruction::F64Lt);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_date_append_literal(output, "0", function)?;
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        let schema = self.runtime_schema();
        let piece = schema.reserve_gc_local(function).initialize(
            self.emit_number_to_string_payload(number, function)?,
            function,
        );
        output.replace(
            self.emit_concat_gc_strings(output, &piece, function),
            function,
        );
        piece.clear(function);
        Ok(())
    }

    fn emit_date_select_label(
        &mut self,
        component: I64Local,
        labels: &[&str],
        output: &GcLocal<StringValue>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // The caller supplies a finite Date calendar component, whose range
        // fixes the label set before this formatting-only projection.
        output.replace(
            self.emit_interned_string_reference(labels[0], function)?,
            function,
        );
        for (index, label) in labels.iter().enumerate().skip(1) {
            component.load(function);
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::F64Const(Ieee64::from(index as f64)));
            function.instruction(&Instruction::F64Eq);
            self.open_frame(ControlFrameKind::If, function);
            output.replace(
                self.emit_interned_string_reference(label, function)?,
                function,
            );
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        Ok(())
    }

    fn emit_date_append_year(
        &mut self,
        output: &GcLocal<StringValue>,
        year: I64Local,
        minimum_width: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let magnitude = schema.reserve_i64_local(function);
        year.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Lt);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_date_append_literal(output, "-", function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        year.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Abs);
        function.instruction(&Instruction::I64ReinterpretF64);
        magnitude.store(function);
        self.emit_date_append_padded_decimal(output, magnitude, minimum_width, function)?;
        schema.release_i64_local(magnitude, function);
        Ok(())
    }

    fn emit_date_append_calendar_date(
        &mut self,
        output: &GcLocal<StringValue>,
        coordinate: I64Local,
        fields: &[I64Local; 7],
        utc: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let weekday = schema.reserve_i64_local(function);
        let piece = schema
            .reserve_gc_local(function)
            .initialize(self.emit_interned_string_reference("", function)?, function);
        self.emit_date_day_from_time(coordinate, weekday, function);
        weekday.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(4.0)));
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::I64ReinterpretF64);
        weekday.store(function);
        self.emit_date_positive_mod(weekday, 7.0, function);
        function.instruction(&Instruction::I64ReinterpretF64);
        weekday.store(function);
        self.emit_date_select_label(
            weekday,
            &["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"],
            &piece,
            function,
        )?;
        output.replace(
            self.emit_concat_gc_strings(output, &piece, function),
            function,
        );
        self.emit_date_append_literal(output, if utc { ", " } else { " " }, function)?;
        if utc {
            self.emit_date_append_padded_decimal(output, fields[2], 2, function)?;
            self.emit_date_append_literal(output, " ", function)?;
        }
        self.emit_date_select_label(
            fields[1],
            &[
                "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
            ],
            &piece,
            function,
        )?;
        output.replace(
            self.emit_concat_gc_strings(output, &piece, function),
            function,
        );
        self.emit_date_append_literal(output, " ", function)?;
        if !utc {
            self.emit_date_append_padded_decimal(output, fields[2], 2, function)?;
            self.emit_date_append_literal(output, " ", function)?;
        }
        self.emit_date_append_year(output, fields[0], 4, function)?;
        piece.clear(function);
        schema.release_i64_local(weekday, function);
        Ok(())
    }

    fn emit_date_append_clock_time(
        &mut self,
        output: &GcLocal<StringValue>,
        fields: &[I64Local; 7],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        for index in [3, 4, 5] {
            self.emit_date_append_padded_decimal(output, fields[index], 2, function)?;
            if index != 5 {
                self.emit_date_append_literal(output, ":", function)?;
            }
        }
        Ok(())
    }

    pub(crate) fn emit_date_to_utc_string(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let receiver = schema.reserve_value_local(function);
        let value = schema.reserve_value_local(function);
        let fields: [I64Local; 7] = std::array::from_fn(|_| schema.reserve_i64_local(function));
        let output = schema
            .reserve_gc_local(function)
            .initialize(self.emit_interned_string_reference("", function)?, function);
        self.compile_this_to_locals(&receiver, function)?;
        let captured = self.emit_date_capture_value(&receiver, function)?;
        captured.emit_valid_branch(
            self,
            function,
            |builder, function, finite| {
                builder.emit_date_components_from_time(
                    finite.payload(),
                    fields[0],
                    fields[1],
                    fields[2],
                    fields[3],
                    fields[4],
                    fields[5],
                    fields[6],
                    function,
                );
                builder.emit_date_append_calendar_date(
                    &output,
                    finite.payload(),
                    &fields,
                    true,
                    function,
                )?;
                builder.emit_date_append_literal(&output, " ", function)?;
                builder.emit_date_append_clock_time(&output, &fields, function)?;
                builder.emit_date_append_literal(&output, " GMT", function)
            },
            |builder, function| {
                output.replace(
                    builder.emit_interned_string_reference("Invalid Date", function)?,
                    function,
                );
                Ok(())
            },
        )?;
        value.set_reference(&output, schema, function);
        self.completion().set_normal(&value, function);
        captured.release(self, function);
        output.clear(function);
        for field in fields.into_iter().rev() {
            schema.release_i64_local(field, function);
        }
        value.clear(function);
        receiver.clear(function);
        Ok(())
    }

    pub(crate) fn emit_date_to_iso_string(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let receiver = schema.reserve_value_local(function);
        let value = schema.reserve_value_local(function);
        let result = schema.reserve_completion(function);
        let fields: [I64Local; 7] = std::array::from_fn(|_| schema.reserve_i64_local(function));
        let output = schema
            .reserve_gc_local(function)
            .initialize(self.emit_interned_string_reference("", function)?, function);
        self.compile_this_to_locals(&receiver, function)?;
        let captured = self.emit_date_capture_value(&receiver, function)?;
        captured.emit_valid_branch(
            self,
            function,
            |builder, function, finite| {
                builder.emit_date_components_from_time(
                    finite.payload(),
                    fields[0],
                    fields[1],
                    fields[2],
                    fields[3],
                    fields[4],
                    fields[5],
                    fields[6],
                    function,
                );
                fields[0].load(function);
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                function.instruction(&Instruction::F64Lt);
                fields[0].load(function);
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Const(Ieee64::from(9999.0)));
                function.instruction(&Instruction::F64Gt);
                function.instruction(&Instruction::I32Or);
                builder.open_frame(ControlFrameKind::If, function);
                fields[0].load(function);
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                function.instruction(&Instruction::F64Ge);
                builder.open_frame(ControlFrameKind::If, function);
                builder.emit_date_append_literal(&output, "+", function)?;
                builder.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
                builder.emit_date_append_year(&output, fields[0], 6, function)?;
                function.instruction(&Instruction::Else);
                builder.emit_date_append_year(&output, fields[0], 4, function)?;
                builder.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
                builder.emit_date_append_literal(&output, "-", function)?;
                fields[1].load(function);
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
                function.instruction(&Instruction::F64Add);
                function.instruction(&Instruction::I64ReinterpretF64);
                fields[1].store(function);
                builder.emit_date_append_padded_decimal(&output, fields[1], 2, function)?;
                builder.emit_date_append_literal(&output, "-", function)?;
                builder.emit_date_append_padded_decimal(&output, fields[2], 2, function)?;
                builder.emit_date_append_literal(&output, "T", function)?;
                builder.emit_date_append_clock_time(&output, &fields, function)?;
                builder.emit_date_append_literal(&output, ".", function)?;
                builder.emit_date_append_padded_decimal(&output, fields[6], 3, function)?;
                builder.emit_date_append_literal(&output, "Z", function)?;
                value.set_reference(&output, schema, function);
                result.set_normal(&value, function);
                Ok(())
            },
            |builder, function| {
                builder.emit_throw_current_function_realm_range_error(
                    RuntimeErrorMessage::DATE_VALUE_IS_NOT_FINITE,
                    &result,
                    function,
                )
            },
        )?;
        self.completion().copy_from(&result, function);
        captured.release(self, function);
        output.clear(function);
        for field in fields.into_iter().rev() {
            schema.release_i64_local(field, function);
        }
        result.clear(function);
        value.clear(function);
        receiver.clear(function);
        Ok(())
    }

    pub(crate) fn emit_date_to_temporal_instant(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let receiver = schema.reserve_value_local(function);
        let result = schema.reserve_completion(function);
        let milliseconds = schema.reserve_completion(function);
        let scale = schema.reserve_completion(function);
        let bits = schema.reserve_i64_local(function);
        self.compile_this_to_locals(&receiver, function)?;
        let captured = self.emit_date_capture_value(&receiver, function)?;
        captured.emit_valid_branch(
            self,
            function,
            |builder, function, finite| {
                // The branded Date slot is an integral Number bounded by TimeClip.
                // Widen before multiplication so the exact nanoseconds can exceed i64.
                let cleanup = builder.open_frame(ControlFrameKind::Block, function);
                builder.emit_number_to_bigint_locals(finite.payload(), &milliseconds, function)?;
                builder.emit_date_native_throw_exit(&milliseconds, &result, cleanup, function);
                function.instruction(&Instruction::F64Const(Ieee64::from(1_000_000.0)));
                function.instruction(&Instruction::I64ReinterpretF64);
                bits.store(function);
                builder.emit_number_to_bigint_locals(bits, &scale, function)?;
                builder.emit_date_native_throw_exit(&scale, &result, cleanup, function);
                builder.emit_bigint_binary_op_to_locals(
                    crate::bigint::BigIntHelperOp::Mul,
                    milliseconds.value(),
                    scale.value(),
                    &result,
                    function,
                )?;
                result.kind().load(function);
                function.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
                function.instruction(&Instruction::I32Eq);
                builder.emit_branch_if_to_target(cleanup, function);
                let nanos = schema.reserve_gc_local(function).initialize(
                    result
                        .value()
                        .cast_reference::<crate::gc_types::BigIntValue>(schema, function),
                    function,
                );
                let header = schema.reserve_gc_local(function).initialize(
                    builder.emit_alloc_temporal_object_header(
                        crate::intrinsics::temporal::TemporalIntrinsicFamily::Instant,
                        TemporalPrototypeSource::Intrinsic,
                        function,
                    )?,
                    function,
                );
                let instant = schema.reserve_gc_local(function).initialize(
                    schema
                        .struct_type::<crate::gc_types::TemporalInstantObject>()
                        .construct(
                            (
                                GcOperand::reference(&header, schema),
                                GcOperand::reference(&nanos, schema),
                            ),
                            function,
                        ),
                    function,
                );
                receiver.set_reference(&instant, schema, function);
                result.set_normal(&receiver, function);
                instant.clear(function);
                header.clear(function);
                nanos.clear(function);
                builder.pop_control(ControlFrameKind::Block);
                function.instruction(&Instruction::End);
                Ok(())
            },
            |builder, function| {
                builder.emit_throw_current_function_realm_range_error(
                    RuntimeErrorMessage::DATE_VALUE_IS_NOT_FINITE,
                    &result,
                    function,
                )
            },
        )?;
        self.completion().copy_from(&result, function);
        captured.release(self, function);
        schema.release_i64_local(bits, function);
        scale.clear(function);
        milliseconds.clear(function);
        result.clear(function);
        receiver.clear(function);
        Ok(())
    }

    pub(crate) fn emit_date_to_json(&mut self, function: &mut Function) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let receiver = schema.reserve_value_local(function);
        let object = schema.reserve_completion(function);
        let primitive = schema.reserve_completion(function);
        let method = schema.reserve_completion(function);
        let result = schema.reserve_completion(function);
        let name = schema.reserve_gc_local(function).initialize(
            self.emit_interned_string_reference("toISOString", function)?,
            function,
        );
        let key = crate::operations::PropertyKeyLocals::from_string(schema, &name, function);
        self.compile_this_to_locals(&receiver, function)?;
        let cleanup = self.open_frame(ControlFrameKind::Block, function);
        self.emit_value_to_current_function_realm_object_locals(&receiver, &object, function)?;
        self.emit_date_native_throw_exit(&object, &result, cleanup, function);
        self.emit_tagged_to_primitive_locals_pending(
            ToPrimitiveHint::Number,
            object.value(),
            &primitive,
            function,
        )?;
        self.emit_date_native_throw_exit(&primitive, &result, cleanup, function);
        primitive.value().tag().load(function);
        function.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Number.tag()));
        function.instruction(&Instruction::I32Eq);
        primitive.value().scalar().load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Abs);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        receiver.set_scalar(ScalarValue::Null, function);
        result.set_normal(&receiver, function);
        self.emit_branch_to_target(cleanup, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_object_read(object.value(), object.value(), &key, &method, function)?;
        self.emit_date_native_throw_exit(&method, &result, cleanup, function);
        self.emit_is_callable_i32(method.value(), function)?;
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_current_function_realm_type_error(
            RuntimeErrorMessage::DATE_TOISOSTRING_METHOD_IS_NOT_CALLABLE,
            &result,
            function,
        )?;
        self.emit_branch_to_target(cleanup, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let arguments = self.emit_pre_evaluated_arg_vector(&[], function);
        self.emit_function_or_proxy_call_with_argv(
            method.value(),
            object.value(),
            &arguments,
            &result,
            function,
        )?;
        arguments.clear(function);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.completion().copy_from(&result, function);
        key.clear(function);
        name.clear(function);
        result.clear(function);
        method.clear(function);
        primitive.clear(function);
        object.clear(function);
        receiver.clear(function);
        Ok(())
    }

    pub(crate) fn emit_date_to_primitive(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let receiver = schema.reserve_value_local(function);
        let hint = schema.reserve_value_local(function);
        let result = schema.reserve_completion(function);
        let equal = schema.reserve_i32_local(function);
        let fold = schema.reserve_i32_local(function);
        function.instruction(&Instruction::I32Const(0));
        fold.store(function);
        self.compile_this_to_locals(&receiver, function)?;
        let cleanup = self.open_frame(ControlFrameKind::Block, function);
        self.emit_is_heap_object_like_tag_i32(receiver.tag(), function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_current_function_realm_type_error(
            RuntimeErrorMessage::DATE_PROTOTYPE_SYMBOL_TOPRIMITIVE_RECEIVER_IS_NOT_AN_OBJECT,
            &result,
            function,
        )?;
        self.emit_branch_to_target(cleanup, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_builtin_arg_to_value(0, &hint, function);
        hint.tag().load(function);
        function.instruction(&Instruction::I32Const(WasmRuntimeValueTag::String.tag()));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        let string = schema.reserve_gc_local(function).initialize(
            hint.cast_reference::<StringValue>(schema, function),
            function,
        );
        for (spelling, preference) in [
            ("string", ToPrimitiveHint::String),
            ("default", ToPrimitiveHint::String),
            ("number", ToPrimitiveHint::Number),
        ] {
            let expected = schema.reserve_gc_local(function).initialize(
                self.emit_interned_string_reference(spelling, function)?,
                function,
            );
            self.emit_gc_string_equality(&string, &expected, fold, equal, function);
            expected.clear(function);
            equal.load(function);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_ordinary_to_primitive_pending(preference, &receiver, &result, function)?;
            self.emit_branch_to_target(cleanup, function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        string.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_throw_current_function_realm_type_error(RuntimeErrorMessage::DATE_PROTOTYPE_SYMBOL_TOPRIMITIVE_HINT_MUST_BE_DEFAULT_NUMBER_OR_STRING, &result, function)?;
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.completion().copy_from(&result, function);
        schema.release_i32_local(fold, function);
        schema.release_i32_local(equal, function);
        result.clear(function);
        hint.clear(function);
        receiver.clear(function);
        Ok(())
    }

    pub(crate) fn emit_date_day_within_year(
        &mut self,
        time_payload_local: I64Local,
        year_payload_local: I64Local,
        dest_payload_local: I64Local,
        function: &mut Function,
    ) {
        let day_local = self.runtime_schema().reserve_i64_local(function);
        let year_day_local = self.runtime_schema().reserve_i64_local(function);
        self.emit_date_day_from_time(time_payload_local, day_local, function);
        self.emit_date_day_from_year(year_payload_local, year_day_local, function);
        day_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        year_day_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Sub);
        function.instruction(&Instruction::I64ReinterpretF64);
        dest_payload_local.store(function);
        self.runtime_schema()
            .release_i64_local(year_day_local, function);
        self.runtime_schema().release_i64_local(day_local, function);
    }
}
