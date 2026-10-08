//! Temporal.Instant uses the completed GC epoch proof for every allocation.
//! Observable conversion stays ordered in the executing native Realm.
use super::super::*;
use super::temporal::TemporalEpochNanoseconds;
use super::temporal::TemporalTimeZoneStringGoal;
use super::temporal_zone_provider::TemporalZonedAllocationInput;
use crate::gc_types::*;
use crate::intrinsics::temporal::{TemporalIntrinsicFamily, TemporalPrototypeSource};
use crate::operations::BigIntNumberPolicy;

mod methods;
mod round;
pub(super) use methods::{InstantArithmetic, InstantDifference};

impl FunctionBuilder<'_> {
    pub(in crate::builtins) fn emit_temporal_instant_record_from_receiver(
        &mut self,
        f: &mut Function,
    ) -> Result<GcLocal<TemporalInstantObject>, EmitError> {
        self.emit_temporal_record_from_receiver::<TemporalInstantObject>(f)
    }

    /// ToTemporalInstant has two internal-slot paths. Only the remaining path
    /// performs String-hint ToPrimitive and requires its result to be String.
    pub(in crate::builtins) fn emit_temporal_to_instant_epoch(
        &mut self,
        input: &ValueLocals,
        f: &mut Function,
    ) -> Result<TemporalEpochNanoseconds, EmitError> {
        let schema = self.runtime_schema();
        let selected = schema.reserve_value_local(f);
        selected.set_undefined(f);
        input.reference().load(f);
        f.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<TemporalInstantObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        self.open_frame(ControlFrameKind::If, f);
        let instant = schema
            .reserve_gc_local(f)
            .initialize(input.cast_reference::<TemporalInstantObject>(schema, f), f);
        let epoch = schema.reserve_gc_local(f).initialize(
            schema
                .struct_type::<TemporalInstantObject>()
                .field(TemporalInstantObjectSchema::EPOCH_NANOSECONDS)
                .read(&instant, schema, f)
                .reference(),
            f,
        );
        selected.set_reference(&epoch, schema, f);
        epoch.clear(f);
        instant.clear(f);
        f.instruction(&Instruction::Else);
        input.reference().load(f);
        f.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<TemporalZonedDateTimeObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        self.open_frame(ControlFrameKind::If, f);
        let zoned = schema.reserve_gc_local(f).initialize(
            input.cast_reference::<TemporalZonedDateTimeObject>(schema, f),
            f,
        );
        let epoch = schema.reserve_gc_local(f).initialize(
            schema
                .struct_type::<TemporalZonedDateTimeObject>()
                .field(TemporalZonedDateTimeObjectSchema::EPOCH_NANOSECONDS)
                .read(&zoned, schema, f)
                .reference(),
            f,
        );
        selected.set_reference(&epoch, schema, f);
        epoch.clear(f);
        zoned.clear(f);
        f.instruction(&Instruction::Else);
        let primitive = self.emit_tagged_to_primitive_locals_in_current_function_realm(
            ToPrimitiveHint::String,
            input,
            f,
        )?;
        let text_value = schema.reserve_value_local(f);
        self.emit_current_function_realm_primitive_to_tagged_locals(primitive, &text_value, f);
        text_value.tag().load(f);
        f.instruction(&Instruction::I32Const(WasmRuntimeValueTag::String.tag()));
        f.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_INSTANT_FROM_REQUIRES_A_STRING_OR_TEMPORAL_INSTANT,
            f,
        )?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let text = schema
            .reserve_gc_local(f)
            .initialize(text_value.cast_reference::<StringValue>(schema, f), f);
        let epoch = self.emit_temporal_parse_instant_string(&text, f)?;
        selected.set_reference(&epoch, schema, f);
        epoch.clear(f);
        text.clear(f);
        text_value.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let value = schema
            .reserve_gc_local(f)
            .initialize(selected.cast_reference::<BigIntValue>(schema, f), f);
        let epoch = self.emit_temporal_instant_validated_epoch(&value, f)?;
        value.clear(f);
        selected.clear(f);
        Ok(epoch)
    }

    pub(in crate::builtins) fn emit_temporal_instant_epoch_from_record(
        &mut self,
        record: &GcLocal<TemporalInstantObject>,
        f: &mut Function,
    ) -> Result<TemporalEpochNanoseconds, EmitError> {
        let schema = self.runtime_schema();
        let value = schema.reserve_gc_local(f).initialize(
            schema
                .struct_type::<TemporalInstantObject>()
                .field(TemporalInstantObjectSchema::EPOCH_NANOSECONDS)
                .read(record, schema, f)
                .reference(),
            f,
        );
        let epoch = self.emit_temporal_instant_validated_epoch(&value, f)?;
        value.clear(f);
        Ok(epoch)
    }

    pub(crate) fn emit_temporal_instant_from(&mut self, f: &mut Function) -> Result<(), EmitError> {
        let input = self.runtime_schema().reserve_value_local(f);
        self.emit_builtin_arg_to_value(0, &input, f);
        let epoch = self.emit_temporal_to_instant_epoch(&input, f)?;
        self.emit_alloc_temporal_instant(&epoch, TemporalPrototypeSource::Intrinsic, f)?;
        epoch.clear(self, f);
        input.clear(f);
        Ok(())
    }

    pub(crate) fn emit_temporal_instant_constructor(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.body_entry_locals()
            .ok_or_else(|| EmitError::unsupported("Temporal.Instant constructor entry absent"))?
            .new_target()
            .tag()
            .load(f);
        f.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Undefined.tag()));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_INSTANT_CONSTRUCTOR_REQUIRES_NEW,
            f,
        )?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let input = self.runtime_schema().reserve_value_local(f);
        self.emit_builtin_arg_to_value(0, &input, f);
        let epoch = self.emit_temporal_epoch_from_value(&input, f)?;
        let prototype =
            self.emit_temporal_constructor_prototype(TemporalIntrinsicFamily::Instant, f)?;
        self.emit_alloc_temporal_instant(
            &epoch,
            TemporalPrototypeSource::Constructor(&prototype),
            f,
        )?;
        prototype.release(f);
        epoch.clear(self, f);
        input.clear(f);
        Ok(())
    }

    pub(in crate::builtins) fn emit_temporal_epoch_from_value(
        &mut self,
        input: &ValueLocals,
        f: &mut Function,
    ) -> Result<TemporalEpochNanoseconds, EmitError> {
        let schema = self.runtime_schema();
        let pending = schema.reserve_completion(f);
        self.emit_value_to_bigint_locals(input, BigIntNumberPolicy::RejectNumber, &pending, f)?;
        self.completion().copy_from(&pending, f);
        self.emit_propagate_current_throw_if_needed(f);
        let value = schema
            .reserve_gc_local(f)
            .initialize(pending.value().cast_reference::<BigIntValue>(schema, f), f);
        let epoch = self.emit_temporal_instant_validated_epoch(&value, f)?;
        value.clear(f);
        pending.clear(f);
        Ok(epoch)
    }

    pub(crate) fn emit_temporal_instant_from_epoch_nanoseconds(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let input = self.runtime_schema().reserve_value_local(f);
        self.emit_builtin_arg_to_value(0, &input, f);
        let epoch = self.emit_temporal_epoch_from_value(&input, f)?;
        self.emit_alloc_temporal_instant(&epoch, TemporalPrototypeSource::Intrinsic, f)?;
        epoch.clear(self, f);
        input.clear(f);
        Ok(())
    }

    /// Two 32-bit partial products form the exact 128-bit magnitude. There is
    /// one canonical GC BigInt publication even when the result fits i64.
    pub(in crate::builtins) fn emit_temporal_epoch_milliseconds_to_epoch_nanoseconds(
        &mut self,
        milliseconds: I64Local,
        f: &mut Function,
    ) -> GcLocal<BigIntValue> {
        let schema = self.runtime_schema();
        let negative = schema.reserve_i32_local(f);
        let magnitude = schema.reserve_i64_local(f);
        let low_product = schema.reserve_i64_local(f);
        let high_product = schema.reserve_i64_local(f);
        let low = schema.reserve_i64_local(f);
        let high = schema.reserve_i64_local(f);
        let index = schema.reserve_i32_local(f);
        milliseconds.load(f);
        f.instruction(&Instruction::I64Const(0));
        f.instruction(&Instruction::I64LtS);
        negative.store(f);
        negative.load(f);
        f.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        f.instruction(&Instruction::I64Const(0));
        milliseconds.load(f);
        f.instruction(&Instruction::I64Sub);
        f.instruction(&Instruction::Else);
        milliseconds.load(f);
        f.instruction(&Instruction::End);
        magnitude.store(f);
        magnitude.load(f);
        f.instruction(&Instruction::I64Const(u32::MAX as i64));
        f.instruction(&Instruction::I64And);
        f.instruction(&Instruction::I64Const(1_000_000));
        f.instruction(&Instruction::I64Mul);
        low_product.store(f);
        magnitude.load(f);
        f.instruction(&Instruction::I64Const(32));
        f.instruction(&Instruction::I64ShrU);
        f.instruction(&Instruction::I64Const(1_000_000));
        f.instruction(&Instruction::I64Mul);
        high_product.store(f);
        low_product.load(f);
        high_product.load(f);
        f.instruction(&Instruction::I64Const(32));
        f.instruction(&Instruction::I64Shl);
        f.instruction(&Instruction::I64Add);
        low.store(f);
        high_product.load(f);
        f.instruction(&Instruction::I64Const(32));
        f.instruction(&Instruction::I64ShrU);
        low.load(f);
        low_product.load(f);
        f.instruction(&Instruction::I64LtU);
        f.instruction(&Instruction::I64ExtendI32U);
        f.instruction(&Instruction::I64Add);
        high.store(f);
        f.instruction(&Instruction::I32Const(2));
        index.store(f);
        let slot = schema.reserve_gc_local(f);
        let construction = BigIntConstruction::allocate(schema, slot, index, f);
        f.instruction(&Instruction::I32Const(0));
        index.store(f);
        construction.write(index, low, schema, f);
        f.instruction(&Instruction::I32Const(1));
        index.store(f);
        construction.write(index, high, schema, f);
        let result = schema
            .reserve_gc_local(f)
            .initialize(construction.publish(negative, schema, f), f);
        schema.release_i32_local(index, f);
        for local in [high, low, high_product, low_product, magnitude] {
            schema.release_i64_local(local, f);
        }
        schema.release_i32_local(negative, f);
        result
    }

    pub(crate) fn emit_temporal_instant_from_epoch_milliseconds(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let input = schema.reserve_value_local(f);
        let pending = schema.reserve_completion(f);
        let milliseconds = schema.reserve_i64_local(f);
        self.emit_builtin_arg_to_value(0, &input, f);
        self.emit_value_to_number_payload(&input, &pending, f)?;
        self.completion().copy_from(&pending, f);
        self.emit_propagate_current_throw_if_needed(f);
        for _ in 0..2 {
            pending.value().scalar().load(f);
            f.instruction(&Instruction::F64ReinterpretI64);
        }
        f.instruction(&Instruction::F64Trunc);
        f.instruction(&Instruction::F64Ne);
        pending.value().scalar().load(f);
        f.instruction(&Instruction::F64ReinterpretI64);
        f.instruction(&Instruction::F64Abs);
        f.instruction(&Instruction::F64Const(f64::MAX.into()));
        f.instruction(&Instruction::F64Gt);
        f.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::RangeError,
            RuntimeErrorMessage::TEMPORAL_INSTANT_FROMEPOCHMILLISECONDS_REQUIRES_AN_INTEGRAL_NUMBER,
            f,
        )?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        pending.value().scalar().load(f);
        f.instruction(&Instruction::F64ReinterpretI64);
        f.instruction(&Instruction::I64TruncSatF64S);
        milliseconds.store(f);
        let value = self.emit_temporal_epoch_milliseconds_to_epoch_nanoseconds(milliseconds, f);
        let epoch = self.emit_temporal_instant_validated_epoch(&value, f)?;
        self.emit_alloc_temporal_instant(&epoch, TemporalPrototypeSource::Intrinsic, f)?;
        epoch.clear(self, f);
        value.clear(f);
        schema.release_i64_local(milliseconds, f);
        pending.clear(f);
        input.clear(f);
        Ok(())
    }

    pub(crate) fn emit_temporal_instant_compare(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let input = schema.reserve_value_local(f);
        self.emit_builtin_arg_to_value(0, &input, f);
        let left = self.emit_temporal_to_instant_epoch(&input, f)?;
        self.emit_builtin_arg_to_value(1, &input, f);
        let right = self.emit_temporal_to_instant_epoch(&input, f)?;
        let comparison = schema.reserve_i32_local(f);
        let bits = schema.reserve_i64_local(f);
        self.emit_bigint_compare(left.value(), right.value(), comparison, f);
        comparison.load(f);
        f.instruction(&Instruction::F64ConvertI32S);
        f.instruction(&Instruction::I64ReinterpretF64);
        bits.store(f);
        self.completion().value().set_number(bits, f);
        self.completion().set_normal(self.completion().value(), f);
        schema.release_i64_local(bits, f);
        schema.release_i32_local(comparison, f);
        right.clear(self, f);
        left.clear(self, f);
        input.clear(f);
        Ok(())
    }

    pub(crate) fn emit_temporal_instant_equals(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let receiver = self.emit_temporal_instant_record_from_receiver(f)?;
        let value = schema.reserve_gc_local(f).initialize(
            schema
                .struct_type::<TemporalInstantObject>()
                .field(TemporalInstantObjectSchema::EPOCH_NANOSECONDS)
                .read(&receiver, schema, f)
                .reference(),
            f,
        );
        let input = schema.reserve_value_local(f);
        self.emit_builtin_arg_to_value(0, &input, f);
        let other = self.emit_temporal_to_instant_epoch(&input, f)?;
        let comparison = schema.reserve_i32_local(f);
        self.emit_bigint_compare(&value, other.value(), comparison, f);
        comparison.load(f);
        f.instruction(&Instruction::I32Eqz);
        comparison.store(f);
        self.completion().value().set_boolean(comparison, f);
        self.completion().set_normal(self.completion().value(), f);
        schema.release_i32_local(comparison, f);
        other.clear(self, f);
        input.clear(f);
        value.clear(f);
        receiver.clear(f);
        Ok(())
    }

    pub(crate) fn emit_temporal_instant_epoch_nanoseconds(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let receiver = self.emit_temporal_instant_record_from_receiver(f)?;
        let epoch = schema.reserve_gc_local(f).initialize(
            schema
                .struct_type::<TemporalInstantObject>()
                .field(TemporalInstantObjectSchema::EPOCH_NANOSECONDS)
                .read(&receiver, schema, f)
                .reference(),
            f,
        );
        self.completion().value().set_reference(&epoch, schema, f);
        self.completion().set_normal(self.completion().value(), f);
        epoch.clear(f);
        receiver.clear(f);
        Ok(())
    }

    pub(crate) fn emit_temporal_instant_epoch_milliseconds(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let receiver = self.emit_temporal_instant_record_from_receiver(f)?;
        let value = schema.reserve_gc_local(f).initialize(
            schema
                .struct_type::<TemporalInstantObject>()
                .field(TemporalInstantObjectSchema::EPOCH_NANOSECONDS)
                .read(&receiver, schema, f)
                .reference(),
            f,
        );
        let epoch = self.emit_temporal_instant_validated_epoch(&value, f)?;
        let bits = schema.reserve_i64_local(f);
        epoch.floor_seconds().load(f);
        f.instruction(&Instruction::I64Const(1000));
        f.instruction(&Instruction::I64Mul);
        epoch.nanosecond().load(f);
        f.instruction(&Instruction::I64Const(1_000_000));
        f.instruction(&Instruction::I64DivU);
        f.instruction(&Instruction::I64Add);
        f.instruction(&Instruction::F64ConvertI64S);
        f.instruction(&Instruction::I64ReinterpretF64);
        bits.store(f);
        self.completion().value().set_number(bits, f);
        self.completion().set_normal(self.completion().value(), f);
        schema.release_i64_local(bits, f);
        epoch.clear(self, f);
        value.clear(f);
        receiver.clear(f);
        Ok(())
    }

    pub(crate) fn emit_temporal_instant_to_locale_string(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_intl_dtf_temporal_to_locale_string(
            super::intl_datetimeformat::DtfTemporalKind::Instant,
            f,
        )
    }
    pub(crate) fn emit_temporal_instant_value_of(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_error_and_return(lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_INSTANT_DOES_NOT_SUPPORT_IMPLICIT_CONVERSION_USE_COMPARE_OR_EQUALS,f)
    }

    pub(crate) fn emit_temporal_instant_to_zoned_date_time_iso(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let record = self.emit_temporal_branded_instant_receiver(f)?;
        let input = schema.reserve_value_local(f);
        self.emit_builtin_arg_to_value(0, &input, f);
        input.tag().load(f);
        f.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Undefined.tag()));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_INSTANT_PROTOTYPE_TOZONEDDATETIMEISO_REQUIRES_A_TIME_ZONE,
            f,
        )?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let zone = self.emit_temporal_zoned_date_time_time_zone(
            &input,
            TemporalTimeZoneStringGoal::Object,
            f,
        )?;
        let instant = self.emit_temporal_normalized_instant_from_instant_record(&record, f)?;
        let calendar = self.emit_temporal_iso_calendar_slot(f)?;
        self.emit_alloc_temporal_zoned_date_time(
            TemporalZonedAllocationInput::new(&instant, &zone, &calendar),
            TemporalPrototypeSource::Intrinsic,
            f,
        )?;
        calendar.release(self, f);
        instant.release(self, f);
        zone.release(self, f);
        input.clear(f);
        record.release(f);
        Ok(())
    }
}
