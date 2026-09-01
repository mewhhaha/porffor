//! `Temporal.PlainTime` codegen: record layout, validation, constructor and
//! the six unit accessors.
//!
//! Temporal proposal 4. A `PlainTime` is a wall-clock time with no date, no
//! time zone and no calendar — six small integers and nothing else. That makes
//! it the one Temporal type whose whole value fits in a single `i64`: the
//! nanosecond-of-day, in `[0, 86400 x 10^9)`. Almost every operation here goes
//! through that scalar and back, which is why `emit_temporal_plain_time_total_nanoseconds`
//! and `emit_temporal_plain_time_from_nanoseconds` are the two functions to
//! read first.

use super::super::*;
use super::temporal_options::{TemporalOverflow, TemporalTimeUnit, TemporalUnit};

impl TemporalTimeUnit {
    const fn plain_time_field_index(self) -> usize {
        match self {
            Self::Hour => 0,
            Self::Minute => 1,
            Self::Second => 2,
            Self::Millisecond => 3,
            Self::Microsecond => 4,
            Self::Nanosecond => 5,
        }
    }

    const fn plain_time_record_offset(self) -> u64 {
        match self {
            Self::Hour => HEAP_TEMPORAL_PLAIN_TIME_HOUR_OFFSET,
            Self::Minute => HEAP_TEMPORAL_PLAIN_TIME_MINUTE_OFFSET,
            Self::Second => HEAP_TEMPORAL_PLAIN_TIME_SECOND_OFFSET,
            Self::Millisecond => HEAP_TEMPORAL_PLAIN_TIME_MILLISECOND_OFFSET,
            Self::Microsecond => HEAP_TEMPORAL_PLAIN_TIME_MICROSECOND_OFFSET,
            Self::Nanosecond => HEAP_TEMPORAL_PLAIN_TIME_NANOSECOND_OFFSET,
        }
    }

    const fn plain_time_field_maximum(self) -> i64 {
        match self {
            Self::Hour => 23,
            Self::Minute | Self::Second => 59,
            Self::Millisecond | Self::Microsecond | Self::Nanosecond => 999,
        }
    }
}

/// `ToTemporalTimeRecord` reads the property bag in alphabetical order and the
/// reads are observable, so the order here is load-bearing. Each entry is
/// `(property name, index into the declaration-order arrays)`.
pub(crate) const TEMPORAL_PLAIN_TIME_ALPHABETICAL_FIELDS: [(&str, usize); 6] = [
    ("hour", 0),
    ("microsecond", 4),
    ("millisecond", 3),
    ("minute", 1),
    ("nanosecond", 5),
    ("second", 2),
];

/// `nsPerDay`. Derived from the unit table rather than restated, so the two
/// cannot drift; the `panic!` arm is const-evaluated and would fail the build
/// if `day` ever stopped having a fixed length.
pub(crate) const NANOSECONDS_PER_TEMPORAL_DAY: i64 = match TemporalUnit::Day.nanoseconds() {
    Some(nanoseconds) => nanoseconds,
    None => panic!("the day unit has a fixed nanosecond length"),
};

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn reserve_temporal_plain_time_field_locals(&mut self) -> [u32; 6] {
        let mut locals = [0_u32; 6];
        for slot in locals.iter_mut() {
            *slot = self.reserve_temp_local();
        }
        locals
    }

    pub(crate) fn release_temporal_plain_time_field_locals(&mut self, locals: [u32; 6]) {
        for local in locals.iter().rev() {
            self.release_temp_local(*local);
        }
    }

    pub(crate) fn emit_alloc_temporal_plain_time(
        &mut self,
        field_locals: &[u32; 6],
        prototype_payload_local: Option<u32>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object_payload_local = self.reserve_temp_local();
        let record_local = self.reserve_temp_local();
        let prototype_local = match prototype_payload_local {
            Some(local) => local,
            None => {
                let local = self.reserve_temp_local();
                function.instruction(&Instruction::GlobalGet(
                    TEMPORAL_PLAIN_TIME_PROTOTYPE_GLOBAL_INDEX,
                ));
                function.instruction(&Instruction::LocalSet(local));
                local
            }
        };
        self.emit_alloc_plain_object_with_prototype(Some(prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(object_payload_local));
        self.emit_heap_alloc_const(HEAP_TEMPORAL_PLAIN_TIME_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(record_local));
        for unit in TemporalTimeUnit::ALL {
            self.store_i64_local_at_offset(
                record_local,
                unit.plain_time_record_offset(),
                field_locals[unit.plain_time_field_index()],
                function,
            );
        }
        self.store_i64_const_at_offset(
            object_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_TIME,
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

    /// Leaves an `i32` on the stack: 1 when the value carries
    /// `[[InitializedTemporalTime]]`.
    pub(crate) fn emit_temporal_plain_time_brand_check_i32(
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
            OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_TIME as i64,
        ));
        function.instruction(&Instruction::I64Eq);
    }

    /// The `[[InitializedTemporalTime]]` brand check on `this`, leaving the six
    /// fields loaded. On failure it throws and returns, so callers may treat
    /// the fields as live afterwards.
    pub(crate) fn emit_temporal_plain_time_fields_from_receiver(
        &mut self,
        field_locals: &[u32; 6],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let receiver_brand_local = self.reserve_temp_local();
        let record_local = self.reserve_temp_local();

        self.compile_this_to_locals(receiver_payload_local, receiver_tag_local, function)?;
        self.emit_temporal_plain_time_brand_check_i32(
            receiver_payload_local,
            receiver_tag_local,
            receiver_brand_local,
            function,
        );
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.PlainTime receiver does not have [[InitializedTemporalTime]]",
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
        self.emit_temporal_plain_time_load_record(record_local, field_locals, function);

        for local in [
            record_local,
            receiver_brand_local,
            receiver_tag_local,
            receiver_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn emit_temporal_plain_time_load_record(
        &mut self,
        record_local: u32,
        field_locals: &[u32; 6],
        function: &mut Function,
    ) {
        for unit in TemporalTimeUnit::ALL {
            self.load_i64_to_local_from_offset(
                record_local,
                unit.plain_time_record_offset(),
                field_locals[unit.plain_time_field_index()],
                function,
            );
        }
    }

    /// `RejectTime`: every field must already be in range.
    pub(crate) fn emit_temporal_reject_time(
        &mut self,
        field_locals: &[u32; 6],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        for unit in TemporalTimeUnit::ALL {
            let field_local = field_locals[unit.plain_time_field_index()];
            function.instruction(&Instruction::LocalGet(field_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64LtS);
            function.instruction(&Instruction::LocalGet(field_local));
            function.instruction(&Instruction::I64Const(unit.plain_time_field_maximum()));
            function.instruction(&Instruction::I64GtS);
            function.instruction(&Instruction::I32Or);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_current_function_realm_range_error(
                "Temporal.PlainTime field is out of range",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
        }
        Ok(())
    }

    /// `RegulateTime`: clamp under `constrain`, throw under `reject`.
    pub(crate) fn emit_temporal_regulate_time(
        &mut self,
        field_locals: &[u32; 6],
        overflow_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(overflow_local));
        function.instruction(&Instruction::I64Const(TemporalOverflow::Reject.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_reject_time(field_locals, function)?;
        function.instruction(&Instruction::Else);
        for unit in TemporalTimeUnit::ALL {
            let field_local = field_locals[unit.plain_time_field_index()];
            function.instruction(&Instruction::LocalGet(field_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64LtS);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(field_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalGet(field_local));
            function.instruction(&Instruction::I64Const(unit.plain_time_field_maximum()));
            function.instruction(&Instruction::I64GtS);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(unit.plain_time_field_maximum()));
            function.instruction(&Instruction::LocalSet(field_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::End);
        Ok(())
    }

    /// The nanosecond-of-day scalar. Bounded by `RejectTime`, so it always
    /// stays under `86400 x 10^9` and never comes near the `i64` ceiling.
    pub(crate) fn emit_temporal_plain_time_total_nanoseconds(
        &mut self,
        field_locals: &[u32; 6],
        output_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(0));
        for unit in TemporalTimeUnit::ALL {
            let field_local = field_locals[unit.plain_time_field_index()];
            function.instruction(&Instruction::LocalGet(field_local));
            function.instruction(&Instruction::I64Const(unit.nanoseconds()));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::I64Add);
        }
        function.instruction(&Instruction::LocalSet(output_local));
    }

    /// `BalanceTime` on a nanosecond-of-day scalar that may have run off either
    /// end of the day: the result wraps, because `PlainTime` arithmetic has no
    /// date to carry into.
    pub(crate) fn emit_temporal_plain_time_from_nanoseconds(
        &mut self,
        nanoseconds_local: u32,
        field_locals: &[u32; 6],
        function: &mut Function,
    ) {
        let remaining_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(nanoseconds_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_TEMPORAL_DAY));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::LocalSet(remaining_local));
        // `I64RemS` truncates toward zero, so a negative input leaves a
        // negative remainder; one day restores the floor semantics wrapping
        // needs.
        function.instruction(&Instruction::LocalGet(remaining_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(remaining_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_TEMPORAL_DAY));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(remaining_local));
        function.instruction(&Instruction::End);
        for (index, divisor) in [
            (5_usize, 1_000_i64),
            (4, 1_000),
            (3, 1_000),
            (2, 60),
            (1, 60),
        ] {
            function.instruction(&Instruction::LocalGet(remaining_local));
            function.instruction(&Instruction::I64Const(divisor));
            function.instruction(&Instruction::I64RemS);
            function.instruction(&Instruction::LocalSet(field_locals[index]));
            function.instruction(&Instruction::LocalGet(remaining_local));
            function.instruction(&Instruction::I64Const(divisor));
            function.instruction(&Instruction::I64DivS);
            function.instruction(&Instruction::LocalSet(remaining_local));
        }
        function.instruction(&Instruction::LocalGet(remaining_local));
        function.instruction(&Instruction::LocalSet(field_locals[0]));
        self.release_temp_local(remaining_local);
    }

    /// Round a signed nanosecond count to a whole number of `quantum_local`
    /// nanoseconds, reusing the Duration rounding-mode decision table.
    pub(crate) fn emit_temporal_plain_time_round_nanoseconds(
        &mut self,
        nanoseconds_local: u32,
        quantum_local: u32,
        mode_local: u32,
        function: &mut Function,
    ) {
        let sign_local = self.reserve_temp_local();
        let magnitude_local = self.reserve_temp_local();
        let quotient_local = self.reserve_temp_local();
        let remainder_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(sign_local));
        function.instruction(&Instruction::LocalGet(nanoseconds_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(sign_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(nanoseconds_local));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(magnitude_local));
        function.instruction(&Instruction::LocalGet(magnitude_local));
        function.instruction(&Instruction::LocalGet(quantum_local));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(quotient_local));
        function.instruction(&Instruction::LocalGet(magnitude_local));
        function.instruction(&Instruction::LocalGet(quantum_local));
        function.instruction(&Instruction::I64RemU);
        function.instruction(&Instruction::LocalSet(remainder_local));
        self.emit_temporal_duration_round_up_i32(
            remainder_local,
            quantum_local,
            quotient_local,
            sign_local,
            mode_local,
            function,
        );
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(quotient_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(quotient_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(quantum_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(nanoseconds_local));

        for local in [remainder_local, quotient_local, magnitude_local, sign_local] {
            self.release_temp_local(local);
        }
    }

    /// Temporal proposal 4.1: `Temporal.PlainTime([hour[, minute[, second[,
    /// millisecond[, microsecond[, nanosecond]]]]]])`. Every argument is
    /// optional and defaults to zero, so `length` is 0 even though six are
    /// read.
    pub(crate) fn emit_temporal_plain_time_constructor(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();
        let new_target_payload_local = self.reserve_temp_local();
        let new_target_tag_local = self.reserve_temp_local();
        let field_locals = self.reserve_temporal_plain_time_field_locals();

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
            "Temporal.PlainTime constructor requires new",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        for index in 0..6_usize {
            self.emit_builtin_arg_to_locals(
                index,
                argument_payload_local,
                argument_tag_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(argument_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(field_locals[index]));
            function.instruction(&Instruction::Else);
            self.emit_temporal_to_integer_with_truncation(
                argument_payload_local,
                argument_tag_local,
                field_locals[index],
                "Temporal.PlainTime field must be an integer",
                function,
            )?;
            function.instruction(&Instruction::End);
        }
        self.emit_temporal_reject_time(&field_locals, function)?;
        self.emit_error_new_target_prototype_to_local(
            TEMPORAL_PLAIN_TIME_PROTOTYPE_GLOBAL_INDEX,
            None,
            prototype_payload_local,
            function,
        )?;
        self.emit_alloc_temporal_plain_time(
            &field_locals,
            Some(prototype_payload_local),
            function,
        )?;

        self.release_temporal_plain_time_field_locals(field_locals);
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

    /// Every `Temporal.PlainTime.prototype` accessor: one record read and one
    /// `i64`-to-Number conversion, selected by field index.
    pub(crate) fn emit_temporal_plain_time_field(
        &mut self,
        unit: TemporalTimeUnit,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let field_locals = self.reserve_temporal_plain_time_field_locals();
        self.emit_temporal_plain_time_fields_from_receiver(&field_locals, function)?;
        function.instruction(&Instruction::LocalGet(
            field_locals[unit.plain_time_field_index()],
        ));
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.release_temporal_plain_time_field_locals(field_locals);
        Ok(())
    }

    /// Temporal deliberately forbids implicit comparison, so `valueOf` always
    /// throws — `a < b` on two times must be a loud error, not a silent string
    /// comparison.
    pub(crate) fn emit_temporal_plain_time_value_of(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_throw_current_function_realm_type_error(
            "Temporal.PlainTime does not support implicit conversion; use compare() or equals()",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        Ok(())
    }
}
