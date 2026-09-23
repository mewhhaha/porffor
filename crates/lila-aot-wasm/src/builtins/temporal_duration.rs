//! Canonical Duration fields retain integral Number bits, including values wider
//! than i64. Time arithmetic decodes those integers into exact seconds and
//! signed nanosecond remainders; no epoch arithmetic uses floating division.

use super::super::*;
use super::temporal_options::{TemporalUnit, TEMPORAL_UNIT_SECONDS};
use crate::intrinsics::temporal::TemporalIntrinsicFamily;

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

const TEMPORAL_DURATION_FIELD_OFFSETS: [u64; 10] = [
    HEAP_TEMPORAL_DURATION_YEARS_OFFSET,
    HEAP_TEMPORAL_DURATION_MONTHS_OFFSET,
    HEAP_TEMPORAL_DURATION_WEEKS_OFFSET,
    HEAP_TEMPORAL_DURATION_DAYS_OFFSET,
    HEAP_TEMPORAL_DURATION_HOURS_OFFSET,
    HEAP_TEMPORAL_DURATION_MINUTES_OFFSET,
    HEAP_TEMPORAL_DURATION_SECONDS_OFFSET,
    HEAP_TEMPORAL_DURATION_MILLISECONDS_OFFSET,
    HEAP_TEMPORAL_DURATION_MICROSECONDS_OFFSET,
    HEAP_TEMPORAL_DURATION_NANOSECONDS_OFFSET,
];

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

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn reserve_temporal_duration_field_locals(&mut self) -> TemporalDurationFields {
        TemporalDurationFields::new(std::array::from_fn(|_| self.reserve_temp_local()))
    }

    pub(crate) fn release_temporal_duration_field_locals(
        &mut self,
        fields: TemporalDurationFields,
    ) {
        for local in fields.number_bits_locals().iter().rev() {
            self.release_temp_local(*local);
        }
    }

    pub(crate) fn emit_temporal_duration_zero_fields(
        &mut self,
        fields: &TemporalDurationFields,
        function: &mut Function,
    ) {
        for local in fields.number_bits_locals() {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(*local));
        }
    }

    /// `ToIntegerIfIntegral`: `ToNumber`, then reject anything that is not an
    /// integral finite Number with a RangeError. The result stays in `f64`
    /// bits, preserving the entire integer Number domain. Range validation runs
    /// after every field's observable conversion, and negative zero becomes +0.
    pub(crate) fn emit_temporal_duration_field_to_number(
        &mut self,
        value_payload_local: u32,
        value_tag_local: u32,
        output_bits_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_value_to_number_payload(value_tag_local, value_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(output_bits_local));
        self.emit_return_current_completion_if_throw(function);
        // Not integral: NaN, either infinity, or a value with a fraction.
        function.instruction(&Instruction::LocalGet(output_bits_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(output_bits_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::LocalGet(output_bits_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Abs);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::MAX)));
        function.instruction(&Instruction::F64Gt);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.Duration field must be an integer",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_temporal_duration_canonicalize_zero(output_bits_local, function);
        Ok(())
    }

    /// `DurationSign`: the sign of the first non-zero field, or 0.
    pub(crate) fn emit_temporal_duration_sign(
        &mut self,
        field_locals: &TemporalDurationFields,
        output_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(output_local));
        for local in field_locals.number_bits_locals().iter().rev() {
            function.instruction(&Instruction::LocalGet(*local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(*local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64LtS);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            function.instruction(&Instruction::I64Const(-1));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalSet(output_local));
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
        seconds_local: u32,
        subsecond_local: u32,
        function: &mut Function,
    ) {
        let quotient = self.reserve_temp_local();
        let remainder = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(seconds_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(subsecond_local));
        for (unit, scale) in TEMPORAL_UNIT_SECONDS {
            if unit.is_larger_than(first_unit) {
                continue;
            }
            function.instruction(&Instruction::LocalGet(seconds_local));
            function.instruction(&Instruction::LocalGet(fields.number_bits(unit)));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::I64TruncF64S);
            function.instruction(&Instruction::I64Const(scale));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(seconds_local));
        }
        for unit in TemporalDurationSubsecondUnit::ALL {
            self.emit_temporal_duration_number_divmod(
                fields.number_bits(unit.temporal_unit()),
                unit,
                quotient,
                remainder,
                function,
            );
            function.instruction(&Instruction::LocalGet(seconds_local));
            function.instruction(&Instruction::LocalGet(quotient));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(seconds_local));
            function.instruction(&Instruction::LocalGet(subsecond_local));
            function.instruction(&Instruction::LocalGet(remainder));
            function.instruction(&Instruction::I64Const(unit.nanoseconds()));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(subsecond_local));
        }
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(seconds_local));
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::LocalSet(subsecond_local));
        self.release_temp_local(remainder);
        self.release_temp_local(quotient);
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
        let sign_local = self.reserve_temp_local();
        let seconds_local = self.reserve_temp_local();
        let subsecond_local = self.reserve_temp_local();

        for (unit, bound) in TemporalUnit::ALL
            .into_iter()
            .zip(TEMPORAL_DURATION_FIELD_BOUNDS)
        {
            function.instruction(&Instruction::LocalGet(field_locals.number_bits(unit)));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::F64Abs);
            function.instruction(&Instruction::F64Const(Ieee64::from(bound)));
            function.instruction(&Instruction::F64Ge);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_current_function_realm_range_error(
                "Invalid Temporal.Duration: fields must not exceed the supported range",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
        }

        self.emit_temporal_duration_sign(field_locals, sign_local, function);
        for local in field_locals.number_bits_locals() {
            function.instruction(&Instruction::LocalGet(*local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64LtS);
            function.instruction(&Instruction::LocalGet(sign_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64GtS);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::LocalGet(*local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64GtS);
            function.instruction(&Instruction::LocalGet(sign_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64LtS);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::I32Or);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_current_function_realm_range_error(
                "Invalid Temporal.Duration: fields must not exceed the supported range",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
        }

        self.emit_temporal_duration_normalize_seconds(
            field_locals,
            TemporalUnit::Day,
            seconds_local,
            subsecond_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(TEMPORAL_DURATION_MAXIMUM_SECONDS));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Invalid Temporal.Duration: fields must not exceed the supported range",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.release_temp_local(subsecond_local);
        self.release_temp_local(seconds_local);
        self.release_temp_local(sign_local);
        Ok(())
    }

    /// `CreateTemporalDuration` without the validation, for callers that have
    /// already run `emit_temporal_duration_reject_invalid`.
    pub(crate) fn emit_alloc_temporal_duration(
        &mut self,
        field_locals: &TemporalDurationFields,
        prototype_payload_local: Option<u32>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object_payload_local = self.reserve_temp_local();
        let record_local = self.reserve_temp_local();
        let prototype_local = match prototype_payload_local {
            Some(local) => local,
            None => {
                let local = self.reserve_temp_local();
                self.emit_load_current_builtin_temporal_prototype(
                    TemporalIntrinsicFamily::Duration,
                    local,
                    function,
                );
                local
            }
        };
        self.emit_alloc_plain_object_with_prototype(Some(prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(object_payload_local));
        self.emit_heap_alloc_const(HEAP_TEMPORAL_DURATION_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(record_local));
        for (local, offset) in field_locals
            .number_bits_locals()
            .iter()
            .zip(TEMPORAL_DURATION_FIELD_OFFSETS)
        {
            self.store_i64_local_at_offset(record_local, offset, *local, function);
        }
        self.store_i64_const_at_offset(
            object_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            OBJECT_INTERNAL_BRAND_TEMPORAL_DURATION,
            function,
        );
        self.store_i64_local_at_offset(
            object_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            record_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(object_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        if prototype_payload_local.is_none() {
            self.release_temp_local(prototype_local);
        }
        self.release_temp_local(record_local);
        self.release_temp_local(object_payload_local);
        Ok(())
    }

    /// `CreateTemporalDuration`: validate, then allocate on the intrinsic
    /// prototype.
    pub(crate) fn emit_create_temporal_duration(
        &mut self,
        field_locals: &TemporalDurationFields,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_duration_reject_invalid(field_locals, function)?;
        self.emit_alloc_temporal_duration(field_locals, None, function)
    }

    /// Leaves an `i32` on the stack: 1 when the value carries
    /// `[[InitializedTemporalDuration]]`.
    pub(crate) fn emit_temporal_duration_brand_check_i32(
        &mut self,
        payload_local: u32,
        tag_local: u32,
        brand_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(brand_local));
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            brand_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TEMPORAL_DURATION as i64,
        ));
        function.instruction(&Instruction::I64Eq);
    }

    /// The `[[InitializedTemporalDuration]]` brand check on `this`. On failure
    /// it throws and returns, so callers may treat `record_local` as live.
    pub(crate) fn emit_temporal_duration_record_from_receiver(
        &mut self,
        record_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let receiver_brand_local = self.reserve_temp_local();

        self.compile_this_to_locals(receiver_payload_local, receiver_tag_local, function)?;
        self.emit_temporal_duration_brand_check_i32(
            receiver_payload_local,
            receiver_tag_local,
            receiver_brand_local,
            function,
        );
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.Duration receiver does not have [[InitializedTemporalDuration]]",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            record_local,
            function,
        );

        self.release_temp_local(receiver_brand_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    pub(crate) fn emit_temporal_duration_load_record(
        &mut self,
        record_local: u32,
        field_locals: &TemporalDurationFields,
        function: &mut Function,
    ) {
        for index in 0..10 {
            self.load_i64_to_local_from_offset(
                record_local,
                TEMPORAL_DURATION_FIELD_OFFSETS[index],
                field_locals.number_bits_locals()[index],
                function,
            );
        }
    }

    /// Brand-check `this` and load all ten fields in one step, the preamble
    /// every prototype method shares.
    pub(crate) fn emit_temporal_duration_fields_from_receiver(
        &mut self,
        field_locals: &TemporalDurationFields,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        self.emit_temporal_duration_record_from_receiver(record_local, function)?;
        self.emit_temporal_duration_load_record(record_local, field_locals, function);
        self.release_temp_local(record_local);
        Ok(())
    }

    /// Temporal proposal 7.1.1: `Temporal.Duration(years, ..., nanoseconds)`.
    pub(crate) fn emit_temporal_duration_constructor(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();
        let new_target_payload_local = self.reserve_temp_local();
        let new_target_tag_local = self.reserve_temp_local();
        let field_locals = self.reserve_temporal_duration_field_locals();

        self.compile_new_target_to_locals(
            new_target_payload_local,
            new_target_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(new_target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.Duration constructor requires new",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        // Every argument is coerced before any range check runs, because the
        // `ToNumber` calls are observable and the spec orders them first. An
        // absent argument is 0 without any coercion at all.
        for index in 0..10 {
            self.emit_builtin_arg_to_locals(
                index,
                argument_payload_local,
                argument_tag_local,
                function,
            );
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(
                field_locals.number_bits_locals()[index],
            ));
            function.instruction(&Instruction::LocalGet(argument_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_temporal_duration_field_to_number(
                argument_payload_local,
                argument_tag_local,
                field_locals.number_bits_locals()[index],
                function,
            )?;
            function.instruction(&Instruction::End);
        }
        self.emit_temporal_duration_reject_invalid(&field_locals, function)?;
        let prototype_tag_local = self.reserve_temp_local();
        self.emit_new_target_prototype_to_locals(
            TEMPORAL_DURATION_PROTOTYPE_GLOBAL_INDEX,
            crate::functions::NewTargetPrototypeFallback::RealmIntrinsic(
                TemporalIntrinsicFamily::Duration.prototype_slot().offset(),
            ),
            prototype_payload_local,
            prototype_tag_local,
            function,
        )?;
        self.release_temp_local(prototype_tag_local);
        self.emit_alloc_temporal_duration(&field_locals, Some(prototype_payload_local), function)?;

        self.release_temporal_duration_field_locals(field_locals);
        for local in [
            new_target_tag_local,
            new_target_payload_local,
            prototype_payload_local,
            argument_tag_local,
            argument_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Every `Temporal.Duration.prototype` accessor: the ten unit getters plus
    /// `sign` and `blank`.
    pub(crate) fn emit_temporal_duration_field(
        &mut self,
        builtin: StandardBuiltinId,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let field_locals = self.reserve_temporal_duration_field_locals();
        self.emit_temporal_duration_fields_from_receiver(&field_locals, function)?;

        let unit_index = match builtin {
            StandardBuiltinId::TemporalDurationPrototypeYearsGetter => Some(0),
            StandardBuiltinId::TemporalDurationPrototypeMonthsGetter => Some(1),
            StandardBuiltinId::TemporalDurationPrototypeWeeksGetter => Some(2),
            StandardBuiltinId::TemporalDurationPrototypeDaysGetter => Some(3),
            StandardBuiltinId::TemporalDurationPrototypeHoursGetter => Some(4),
            StandardBuiltinId::TemporalDurationPrototypeMinutesGetter => Some(5),
            StandardBuiltinId::TemporalDurationPrototypeSecondsGetter => Some(6),
            StandardBuiltinId::TemporalDurationPrototypeMillisecondsGetter => Some(7),
            StandardBuiltinId::TemporalDurationPrototypeMicrosecondsGetter => Some(8),
            StandardBuiltinId::TemporalDurationPrototypeNanosecondsGetter => Some(9),
            _ => None,
        };
        match unit_index {
            Some(index) => {
                function.instruction(&Instruction::LocalGet(
                    field_locals.number_bits_locals()[index],
                ));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
            }
            None => {
                let sign_local = self.reserve_temp_local();
                self.emit_temporal_duration_sign(&field_locals, sign_local, function);
                if matches!(
                    builtin,
                    StandardBuiltinId::TemporalDurationPrototypeBlankGetter
                ) {
                    function.instruction(&Instruction::LocalGet(sign_local));
                    function.instruction(&Instruction::I64Eqz);
                    function.instruction(&Instruction::I64ExtendI32U);
                    function.instruction(&Instruction::LocalSet(self.result_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                    function.instruction(&Instruction::LocalSet(self.result_tag_local));
                } else {
                    function.instruction(&Instruction::LocalGet(sign_local));
                    function.instruction(&Instruction::F64ConvertI64S);
                    function.instruction(&Instruction::I64ReinterpretF64);
                    function.instruction(&Instruction::LocalSet(self.result_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                    function.instruction(&Instruction::LocalSet(self.result_tag_local));
                }
                self.release_temp_local(sign_local);
            }
        }

        self.release_temporal_duration_field_locals(field_locals);
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

    /// Both unary transforms rebuild the duration from transformed fields, and
    /// neither can leave the valid range that the receiver already occupies,
    /// so no re-validation is needed.
    fn emit_temporal_duration_with_field_transform(
        &mut self,
        transform: TemporalDurationFieldTransform,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let field_locals = self.reserve_temporal_duration_field_locals();
        self.emit_temporal_duration_fields_from_receiver(&field_locals, function)?;
        match transform {
            TemporalDurationFieldTransform::Negate => {
                self.emit_temporal_duration_negate_fields(&field_locals, function);
            }
            TemporalDurationFieldTransform::AbsoluteValue => {
                for local in field_locals.number_bits_locals() {
                    function.instruction(&Instruction::LocalGet(*local));
                    function.instruction(&Instruction::I64Const(i64::MAX));
                    function.instruction(&Instruction::I64And);
                    function.instruction(&Instruction::LocalSet(*local));
                }
            }
        }
        self.emit_alloc_temporal_duration(&field_locals, None, function)?;
        self.release_temporal_duration_field_locals(field_locals);
        Ok(())
    }

    /// Temporal proposal 7.3.24. The proposal deliberately forbids implicit
    /// comparison, so this always throws.
    pub(crate) fn emit_temporal_duration_value_of(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_throw_current_function_realm_type_error(
            "Temporal.Duration does not support implicit conversion; use compare()",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        Ok(())
    }
}
