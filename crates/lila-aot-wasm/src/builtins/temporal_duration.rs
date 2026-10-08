//! Canonical Duration fields retain integral Number bits, including values wider
//! than i64. Time arithmetic decodes those integers into exact seconds and
//! signed nanosecond remainders; no epoch arithmetic uses floating division.

use super::super::*;
use super::temporal_options::{TemporalUnit, TEMPORAL_UNIT_SECONDS};
use crate::gc_types::*;
use crate::intrinsics::temporal::{TemporalIntrinsicFamily, TemporalPrototypeSource};

mod fields;
pub(crate) use fields::{
    TemporalDurationFields, TemporalDurationNumberProjection, TemporalDurationSubsecondUnit,
};

/// `IsValidDuration` step 3: years, months and weeks are each capped below
/// 2^32.
const TEMPORAL_DURATION_DATE_FIELD_BOUND: f64 = 4_294_967_296.0;
/// Necessary per-field bounds. The exact normalized total is checked separately.
/// All bounds are representable binary64 values; the date-unit ceilings reject
/// the first integral value whose contribution alone reaches 2^53 seconds.
const TEMPORAL_DURATION_FIELD_BOUNDS: [f64; 10] = [
    TEMPORAL_DURATION_DATE_FIELD_BOUND,
    TEMPORAL_DURATION_DATE_FIELD_BOUND,
    TEMPORAL_DURATION_DATE_FIELD_BOUND,
    104_249_991_375.0,
    2_501_999_792_984.0,
    150_119_987_579_017.0,
    9_007_199_254_740_992.0,
    9_007_199_254_740_992_000.0,
    9_007_199_254_740_992_000_000.0,
    9_007_199_254_740_992_000_000_000.0,
];

/// `abs(totalSeconds)` must stay strictly below this.
const TEMPORAL_DURATION_MAXIMUM_SECONDS: i64 = 9_007_199_254_740_992;

#[derive(Clone, Copy)]
enum TemporalDurationFieldTransform {
    Negate,
    AbsoluteValue,
}

/// Declaration order: the constructor argument order, and the order the fields
/// are written into the record.
pub(crate) const TEMPORAL_DURATION_FIELD_NAMES: [&str; 10] = [
    "years",
    "months",
    "weeks",
    "days",
    "hours",
    "minutes",
    "seconds",
    "milliseconds",
    "microseconds",
    "nanoseconds",
];

/// `ToTemporalPartialDurationRecord` reads the property bag in alphabetical
/// order, and the reads are observable, so the order here is load-bearing.
/// Each entry is `(property name, index into the declaration-order arrays)`.
pub(crate) const TEMPORAL_DURATION_ALPHABETICAL_FIELDS: [(&str, usize); 10] = [
    ("days", 3),
    ("hours", 4),
    ("microseconds", 8),
    ("milliseconds", 7),
    ("minutes", 5),
    ("months", 1),
    ("nanoseconds", 9),
    ("seconds", 6),
    ("weeks", 2),
    ("years", 0),
];

/// Actual getter publication; its unit domain is the existing ten Duration fields.
pub(super) enum TemporalDurationField {
    Unit(TemporalUnit),
    Sign,
    Blank,
}

impl FunctionBuilder<'_> {
    pub(crate) fn reserve_temporal_duration_field_locals(
        &mut self,
        function: &mut Function,
    ) -> TemporalDurationFields {
        TemporalDurationFields::new(std::array::from_fn(|_| {
            self.runtime_schema().reserve_i64_local(function)
        }))
    }

    pub(crate) fn release_temporal_duration_field_locals(
        &mut self,
        fields: TemporalDurationFields,
        function: &mut Function,
    ) {
        for local in fields.number_bits_locals().iter().rev() {
            self.runtime_schema().release_i64_local(*local, function);
        }
    }

    pub(crate) fn emit_temporal_duration_zero_fields(
        &self,
        fields: &TemporalDurationFields,
        function: &mut Function,
    ) {
        for local in fields.number_bits_locals() {
            function.instruction(&Instruction::I64Const(0));
            local.store(function);
        }
    }

    /// ToIntegerIfIntegral performs the original observable ToNumber once.
    /// Canonical fields remain Number bits, with negative zero retired to +0.
    pub(crate) fn emit_temporal_duration_field_to_number(
        &mut self,
        input: &ValueLocals,
        output: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let pending = self.runtime_schema().reserve_completion(function);
        self.emit_value_to_number_payload(input, &pending, function)?;
        self.completion().copy_from(&pending, function);
        self.emit_propagate_current_throw_if_needed(function);
        pending.value().scalar().load(function);
        output.store(function);
        pending.clear(function);
        output.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        output.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::F64Ne);
        output.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Abs);
        function.instruction(&Instruction::F64Const(f64::MAX.into()));
        function.instruction(&Instruction::F64Gt);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::RangeError,
            RuntimeErrorMessage::TEMPORAL_DURATION_FIELD_MUST_BE_AN_INTEGER,
            function,
        )?;

        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_duration_canonicalize_zero(output, function);
        Ok(())
    }

    /// `DurationSign`: the sign of the first non-zero field, or 0.
    pub(crate) fn emit_temporal_duration_sign(
        &mut self,
        field_locals: &TemporalDurationFields,
        output_local: I64Local,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(0));
        (output_local).store(function);
        for local in field_locals.number_bits_locals().iter().rev() {
            (*local).load(function);
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            self.open_frame(ControlFrameKind::If, function);
            (*local).load(function);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64LtS);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            function.instruction(&Instruction::I64Const(-1));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::End);
            (output_local).store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
    }

    /// Split the sub-day fields into whole seconds and a signed remainder in
    /// (-10^9, 10^9). Both outputs carry the duration's sign.
    ///
    /// `first_unit` is the coarsest unit to fold in: `Day` takes days and
    /// everything below, which is what `IsValidDuration` needs; `Hour` takes
    /// hours down, which is the unit `TemporalDurationToString` rebalances
    /// across; `Second` takes only the seconds-and-below tail, which is what it
    /// prints when no rounding happens.
    ///
    /// It is a `TemporalUnit`, not the bare field index it used to be: the
    /// index and the emitted unit code are different numberings that happen to
    /// coincide, and one caller was already writing `TEMPORAL_UNIT_DAY as
    /// usize` to bridge them.
    pub(crate) fn emit_temporal_duration_normalize_seconds(
        &mut self,
        fields: &TemporalDurationFields,
        first_unit: TemporalUnit,
        seconds_local: I64Local,
        subsecond_local: I64Local,
        function: &mut Function,
    ) {
        let quotient = self.runtime_schema().reserve_i64_local(function);
        let remainder = self.runtime_schema().reserve_i64_local(function);
        function.instruction(&Instruction::I64Const(0));
        (seconds_local).store(function);
        function.instruction(&Instruction::I64Const(0));
        (subsecond_local).store(function);
        for (unit, scale) in TEMPORAL_UNIT_SECONDS {
            if unit.is_larger_than(first_unit) {
                continue;
            }
            (seconds_local).load(function);
            (fields.number_bits(unit)).load(function);
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::I64TruncF64S);
            function.instruction(&Instruction::I64Const(scale));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::I64Add);
            (seconds_local).store(function);
        }
        for unit in TemporalDurationSubsecondUnit::ALL {
            self.emit_temporal_duration_number_divmod(
                fields.number_bits(unit.temporal_unit()),
                unit,
                quotient,
                remainder,
                function,
            );
            (seconds_local).load(function);
            (quotient).load(function);
            function.instruction(&Instruction::I64Add);
            (seconds_local).store(function);
            (subsecond_local).load(function);
            (remainder).load(function);
            function.instruction(&Instruction::I64Const(unit.nanoseconds()));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::I64Add);
            (subsecond_local).store(function);
        }
        (seconds_local).load(function);
        (subsecond_local).load(function);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::I64Add);
        (seconds_local).store(function);
        (subsecond_local).load(function);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64RemS);
        (subsecond_local).store(function);
        self.runtime_schema().release_i64_local(remainder, function);
        self.runtime_schema().release_i64_local(quotient, function);
    }

    /// `IsValidDuration` steps 2 and 5: every non-zero field must share the
    /// duration's sign, and the normalized second count must stay below 2^53.
    /// Number fields are bounded before any integer projection; exact division
    /// of wide subsecond fields then makes the total-range decision.
    pub(crate) fn emit_temporal_duration_reject_invalid(
        &mut self,
        field_locals: &TemporalDurationFields,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let sign_local = self.runtime_schema().reserve_i64_local(function);
        let seconds_local = self.runtime_schema().reserve_i64_local(function);
        let subsecond_local = self.runtime_schema().reserve_i64_local(function);

        for (unit, bound) in TemporalUnit::ALL
            .into_iter()
            .zip(TEMPORAL_DURATION_FIELD_BOUNDS)
        {
            (field_locals.number_bits(unit)).load(function);
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::F64Abs);
            function.instruction(&Instruction::F64Const(Ieee64::from(bound)));
            function.instruction(&Instruction::F64Ge);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_temporal_error_and_return(lila_ir::NativeErrorKind::RangeError, RuntimeErrorMessage::INVALID_TEMPORAL_DURATION_FIELDS_MUST_NOT_EXCEED_THE_SUPPORTED_RANGE, function)?;
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }

        self.emit_temporal_duration_sign(field_locals, sign_local, function);
        for local in field_locals.number_bits_locals() {
            (*local).load(function);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64LtS);
            (sign_local).load(function);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64GtS);
            function.instruction(&Instruction::I32And);
            (*local).load(function);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64GtS);
            (sign_local).load(function);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64LtS);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::I32Or);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_temporal_error_and_return(lila_ir::NativeErrorKind::RangeError, RuntimeErrorMessage::INVALID_TEMPORAL_DURATION_FIELDS_MUST_NOT_EXCEED_THE_SUPPORTED_RANGE, function)?;
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }

        self.emit_temporal_duration_normalize_seconds(
            field_locals,
            TemporalUnit::Day,
            seconds_local,
            subsecond_local,
            function,
        );
        (seconds_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(0));
        (seconds_local).load(function);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::Else);
        (seconds_local).load(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(TEMPORAL_DURATION_MAXIMUM_SECONDS));
        function.instruction(&Instruction::I64GeS);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(lila_ir::NativeErrorKind::RangeError, RuntimeErrorMessage::INVALID_TEMPORAL_DURATION_FIELDS_MUST_NOT_EXCEED_THE_SUPPORTED_RANGE, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.runtime_schema()
            .release_i64_local(subsecond_local, function);
        self.runtime_schema()
            .release_i64_local(seconds_local, function);
        self.runtime_schema()
            .release_i64_local(sign_local, function);
        Ok(())
    }

    /// The caller has completed IsValidDuration or reads an existing valid record.
    pub(crate) fn emit_alloc_temporal_duration(
        &mut self,
        fields: &TemporalDurationFields,
        prototype: TemporalPrototypeSource<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let numbers: [F64Local; 10] = std::array::from_fn(|index| {
            let number = schema.reserve_f64_local(function);
            fields.number_bits_locals()[index].load(function);
            function.instruction(&Instruction::F64ReinterpretI64);
            number.store(function);
            number
        });
        let header = schema.reserve_gc_local(function).initialize(
            self.emit_alloc_temporal_object_header(
                TemporalIntrinsicFamily::Duration,
                prototype,
                function,
            )?,
            function,
        );
        let record = schema.reserve_gc_local(function).initialize(
            schema.struct_type::<TemporalDurationObject>().construct(
                (
                    GcOperand::reference(&header, schema),
                    GcOperand::f64_local(numbers[0]),
                    GcOperand::f64_local(numbers[1]),
                    GcOperand::f64_local(numbers[2]),
                    GcOperand::f64_local(numbers[3]),
                    GcOperand::f64_local(numbers[4]),
                    GcOperand::f64_local(numbers[5]),
                    GcOperand::f64_local(numbers[6]),
                    GcOperand::f64_local(numbers[7]),
                    GcOperand::f64_local(numbers[8]),
                    GcOperand::f64_local(numbers[9]),
                ),
                function,
            ),
            function,
        );
        self.completion()
            .value()
            .set_reference(&record, schema, function);
        self.completion()
            .set_normal(self.completion().value(), function);
        record.clear(function);
        header.clear(function);
        for number in numbers.into_iter().rev() {
            schema.release_f64_local(number, function);
        }
        Ok(())
    }

    pub(crate) fn emit_create_temporal_duration(
        &mut self,
        fields: &TemporalDurationFields,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_duration_reject_invalid(fields, function)?;
        self.emit_alloc_temporal_duration(fields, TemporalPrototypeSource::Intrinsic, function)
    }

    pub(crate) fn emit_temporal_duration_brand_check_i32(
        &self,
        value: &ValueLocals,
        function: &mut Function,
    ) {
        value.reference().load(function);
        function.instruction(&Instruction::RefTestNonNull(
            self.runtime_schema()
                .reference_type::<TemporalDurationObject>(GcNullability::NonNullable)
                .heap_type,
        ));
    }

    pub(crate) fn emit_temporal_duration_record_from_receiver(
        &mut self,
        function: &mut Function,
    ) -> Result<GcLocal<TemporalDurationObject>, EmitError> {
        self.emit_temporal_record_from_receiver::<TemporalDurationObject>(function)
    }

    pub(crate) fn emit_temporal_duration_load_record(
        &self,
        record: &GcLocal<TemporalDurationObject>,
        fields: &TemporalDurationFields,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let number = schema.reserve_f64_local(function);
        for (bits, field) in fields.number_bits_locals().iter().zip([
            TemporalDurationObjectSchema::YEARS,
            TemporalDurationObjectSchema::MONTHS,
            TemporalDurationObjectSchema::WEEKS,
            TemporalDurationObjectSchema::DAYS,
            TemporalDurationObjectSchema::HOURS,
            TemporalDurationObjectSchema::MINUTES,
            TemporalDurationObjectSchema::SECONDS,
            TemporalDurationObjectSchema::MILLISECONDS,
            TemporalDurationObjectSchema::MICROSECONDS,
            TemporalDurationObjectSchema::NANOSECONDS,
        ]) {
            schema
                .field(field)
                .read(record, schema, function)
                .store_f64(number, function);
            number.load(function);
            function.instruction(&Instruction::I64ReinterpretF64);
            bits.store(function);
        }
        schema.release_f64_local(number, function);
    }

    pub(crate) fn emit_temporal_duration_fields_from_receiver(
        &mut self,
        fields: &TemporalDurationFields,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record = self.emit_temporal_duration_record_from_receiver(function)?;
        self.emit_temporal_duration_load_record(&record, fields, function);
        record.clear(function);
        Ok(())
    }

    pub(crate) fn emit_temporal_duration_constructor(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        self.body_entry_locals()
            .ok_or_else(|| EmitError::unsupported("Duration constructor has no callable entry"))?
            .new_target()
            .tag()
            .load(function);
        function.instruction(&Instruction::I32Const(
            WasmRuntimeValueTag::Undefined as i32,
        ));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_DURATION_CONSTRUCTOR_REQUIRES_NEW,
            function,
        )?;

        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let argument = schema.reserve_value_local(function);
        let fields = self.reserve_temporal_duration_field_locals(function);
        for index in 0..10 {
            self.emit_builtin_arg_to_value(index, &argument, function);
            let field = fields.number_bits_locals()[index];
            function.instruction(&Instruction::I64Const(0));
            field.store(function);
            argument.tag().load(function);
            function.instruction(&Instruction::I32Const(
                WasmRuntimeValueTag::Undefined as i32,
            ));
            function.instruction(&Instruction::I32Ne);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_temporal_duration_field_to_number(&argument, field, function)?;

            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        self.emit_temporal_duration_reject_invalid(&fields, function)?;
        let prototype =
            self.emit_temporal_constructor_prototype(TemporalIntrinsicFamily::Duration, function)?;
        self.emit_alloc_temporal_duration(
            &fields,
            TemporalPrototypeSource::Constructor(&prototype),
            function,
        )?;
        prototype.release(function);
        self.release_temporal_duration_field_locals(fields, function);
        argument.clear(function);
        Ok(())
    }

    pub(crate) fn emit_temporal_duration_field(
        &mut self,
        field: TemporalDurationField,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let fields = self.reserve_temporal_duration_field_locals(function);
        self.emit_temporal_duration_fields_from_receiver(&fields, function)?;
        match field {
            TemporalDurationField::Unit(unit) => {
                self.completion()
                    .value()
                    .set_number(fields.number_bits(unit), function);
            }
            TemporalDurationField::Sign => {
                let sign = schema.reserve_i64_local(function);
                self.emit_temporal_duration_sign(&fields, sign, function);
                sign.load(function);
                function.instruction(&Instruction::F64ConvertI64S);
                function.instruction(&Instruction::I64ReinterpretF64);
                sign.store(function);
                self.completion().value().set_number(sign, function);
                schema.release_i64_local(sign, function);
            }
            TemporalDurationField::Blank => {
                let sign = schema.reserve_i64_local(function);
                let blank = schema.reserve_i32_local(function);
                self.emit_temporal_duration_sign(&fields, sign, function);
                sign.load(function);
                function.instruction(&Instruction::I64Eqz);
                blank.store(function);
                self.completion().value().set_boolean(blank, function);
                schema.release_i32_local(blank, function);
                schema.release_i64_local(sign, function);
            }
        }
        self.completion()
            .set_normal(self.completion().value(), function);
        self.release_temporal_duration_field_locals(fields, function);
        Ok(())
    }

    pub(crate) fn emit_temporal_duration_negated(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_duration_with_field_transform(
            TemporalDurationFieldTransform::Negate,
            function,
        )
    }
    pub(crate) fn emit_temporal_duration_abs(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_duration_with_field_transform(
            TemporalDurationFieldTransform::AbsoluteValue,
            function,
        )
    }
    fn emit_temporal_duration_with_field_transform(
        &mut self,
        transform: TemporalDurationFieldTransform,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let fields = self.reserve_temporal_duration_field_locals(function);
        self.emit_temporal_duration_fields_from_receiver(&fields, function)?;
        match transform {
            TemporalDurationFieldTransform::Negate => {
                self.emit_temporal_duration_negate_fields(&fields, function)
            }
            TemporalDurationFieldTransform::AbsoluteValue => {
                for field in fields.number_bits_locals() {
                    field.load(function);
                    function.instruction(&Instruction::I64Const(i64::MAX));
                    function.instruction(&Instruction::I64And);
                    field.store(function);
                }
            }
        }
        self.emit_alloc_temporal_duration(&fields, TemporalPrototypeSource::Intrinsic, function)?;
        self.release_temporal_duration_field_locals(fields, function);
        Ok(())
    }
    pub(crate) fn emit_temporal_duration_value_of(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_DURATION_DOES_NOT_SUPPORT_IMPLICIT_CONVERSION_USE_COMPARE,
            function,
        )
    }
}
