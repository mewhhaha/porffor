use super::*;
use std::marker::PhantomData;
// A fixed source order owns every Get/ToNumber before the next field is observed.
trait DurationFieldStep {
    const UNIT: DurationUnit;
    type Next;
}
struct Days;
struct Hours;
struct Microseconds;
struct Milliseconds;
struct Minutes;
struct Months;
struct Nanoseconds;
struct Seconds;
struct Weeks;
struct Years;
struct AllFields;
macro_rules! step {
    ($state:ident, $unit:ident, $next:ident) => {
        impl DurationFieldStep for $state {
            const UNIT: DurationUnit = DurationUnit::$unit;
            type Next = $next;
        }
    };
}
step!(Days, Day, Hours);
step!(Hours, Hour, Microseconds);
step!(Microseconds, Microsecond, Milliseconds);
step!(Milliseconds, Millisecond, Minutes);
step!(Minutes, Minute, Months);
step!(Months, Month, Nanoseconds);
step!(Nanoseconds, Nanosecond, Seconds);
step!(Seconds, Second, Weeks);
step!(Weeks, Week, Years);
step!(Years, Year, AllFields);
struct DurationBagSequence<'a, State> {
    fields: &'a [I64Local; 10],
    present: I32Local,
    state: PhantomData<State>,
}
struct CompleteDurationBag;
impl DurationBagSequence<'_, AllFields> {
    fn complete(
        self,
        builder: &mut FunctionBuilder<'_>,
        f: &mut Function,
    ) -> Result<CompleteDurationBag, EmitError> {
        self.present.load(f);
        f.instruction(&Instruction::I32Eqz);
        builder.open_frame(ControlFrameKind::If, f);
        builder.emit_intl_number_type_error(DU_INPUT_ERROR, f)?;
        builder.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        Ok(CompleteDurationBag)
    }
}
impl<'a, State: DurationFieldStep> DurationBagSequence<'a, State> {
    fn read(
        self,
        builder: &mut FunctionBuilder<'_>,
        input: &ValueLocals,
        f: &mut Function,
    ) -> Result<DurationBagSequence<'a, State::Next>, EmitError> {
        let schema = builder.runtime_schema();
        let value = schema.reserve_value_local(f);
        let unit = State::UNIT;
        builder.emit_intl_number_get_option(input, &format!("{}s", unit.name()), &value, f)?;
        emit_tag_is(&value, WasmRuntimeValueTag::Undefined, f);
        f.instruction(&Instruction::I32Eqz);
        builder.open_frame(ControlFrameKind::If, f);
        set_i32(self.present, 1, f);
        let pending = schema.reserve_completion(f);
        pending.initialize(f);
        builder.emit_value_to_number_payload(&value, &pending, f)?;
        builder.emit_intl_number_adopt_completion(&pending, f);
        let bits = self.fields[unit.index()];
        pending.value().scalar().load(f);
        bits.store(f);
        // Field failure is immediate; native uniform-sign/exact aggregate bounds
        // are checked only after the complete ordered bag has been observed.
        bits.load(f);
        f.instruction(&Instruction::F64ReinterpretI64);
        bits.load(f);
        f.instruction(&Instruction::F64ReinterpretI64);
        f.instruction(&Instruction::F64Trunc);
        f.instruction(&Instruction::F64Ne);
        bits.load(f);
        f.instruction(&Instruction::F64ReinterpretI64);
        f.instruction(&Instruction::F64Abs);
        f.instruction(&Instruction::F64Const(Ieee64::from(f64::MAX)));
        f.instruction(&Instruction::F64Gt);
        f.instruction(&Instruction::I32Or);
        builder.open_frame(ControlFrameKind::If, f);
        builder.emit_intl_number_range_error(DU_FIELD_ERROR, f)?;
        builder.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        pending.clear(f);
        builder.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        value.clear(f);
        Ok(DurationBagSequence {
            fields: self.fields,
            present: self.present,
            state: PhantomData,
        })
    }
}
impl FunctionBuilder<'_> {
    pub(super) fn emit_duration_zero_number_fields(&self, f: &mut Function) -> [I64Local; 10] {
        let schema = self.runtime_schema();
        std::array::from_fn(|_| {
            let local = schema.reserve_i64_local(f);
            f.instruction(&Instruction::I64Const(0));
            local.store(f);
            local
        })
    }
    pub(super) fn emit_duration_stored_number_fields(
        &self,
        record: &GcLocal<TemporalDurationObject>,
        fields: &[I64Local; 10],
        f: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let st = schema.struct_type::<TemporalDurationObject>();
        let value = schema.reserve_f64_local(f);
        for (field, output) in [
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
        ]
        .into_iter()
        .zip(fields)
        {
            st.field(field).read(record, schema, f).store_f64(value, f);
            value.load(f);
            f.instruction(&Instruction::I64ReinterpretF64);
            output.store(f);
        }
        schema.release_f64_local(value, f);
    }
    pub(super) fn emit_complete_duration_inputs(
        &mut self,
        f: &mut Function,
    ) -> Result<CompletedDurationFormatInputsLocals, EmitError> {
        let schema = self.runtime_schema();
        let fields = self.emit_duration_zero_number_fields(f);
        let input = schema.reserve_value_local(f);
        self.emit_builtin_arg_to_value(0, &input, f);
        emit_tag_is(&input, WasmRuntimeValueTag::String, f);
        self.open_frame(ControlFrameKind::If, f);
        // Reuse the saved intrinsic's actual Temporal grammar/validation owner.
        // Restrict this call to String; bags retain DurationFormat's read order.
        let meta = self
            .functions
            .get(&StandardBuiltinId::TemporalDurationFrom.function_id())
            .cloned()
            .ok_or_else(|| EmitError::unsupported("Temporal.Duration.from intrinsic is absent"))?;
        let parsed = schema.reserve_completion(f);
        parsed.initialize(f);
        self.emit_direct_js_call(&meta, None, &[&input], &parsed, f)?;
        self.emit_intl_number_adopt_completion(&parsed, f);
        let stored = schema.reserve_gc_local(f).initialize(
            parsed
                .value()
                .cast_reference::<TemporalDurationObject>(schema, f),
            f,
        );
        self.emit_duration_stored_number_fields(&stored, &fields, f);
        stored.clear(f);
        parsed.clear(f);
        f.instruction(&Instruction::Else);
        self.emit_is_heap_object_like_tag_i32(input.tag(), f);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_intl_number_type_error(DU_INPUT_ERROR, f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        input.reference().load(f);
        f.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<TemporalDurationObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        self.open_frame(ControlFrameKind::If, f);
        let stored = schema
            .reserve_gc_local(f)
            .initialize(input.cast_reference::<TemporalDurationObject>(schema, f), f);
        self.emit_duration_stored_number_fields(&stored, &fields, f);
        stored.clear(f);
        f.instruction(&Instruction::Else);
        let present = schema.reserve_i32_local(f);
        set_i32(present, 0, f);
        let sequence: DurationBagSequence<'_, Days> = DurationBagSequence {
            fields: &fields,
            present,
            state: PhantomData,
        };
        let CompleteDurationBag = sequence
            .read(self, &input, f)?
            .read(self, &input, f)?
            .read(self, &input, f)?
            .read(self, &input, f)?
            .read(self, &input, f)?
            .read(self, &input, f)?
            .read(self, &input, f)?
            .read(self, &input, f)?
            .read(self, &input, f)?
            .read(self, &input, f)?
            .complete(self, f)?;
        schema.release_i32_local(present, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        input.clear(f);
        Ok(CompletedDurationFormatInputsLocals {
            number_bits: fields,
        })
    }
}
