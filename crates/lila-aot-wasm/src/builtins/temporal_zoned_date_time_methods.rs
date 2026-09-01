//! `Temporal.ZonedDateTime.prototype` arithmetic and calendar methods:
//! `add`, `subtract`, `until`, `since`, `withCalendar`.
//!
//! Split out of `temporal.rs` (record, constructor, accessors, `equals`,
//! `toInstant`, `withTimeZone`, `toPlainDateTime`) the same way
//! `temporal_plain_date_time_methods.rs` is split out of
//! `temporal_plain_date_time.rs`. Both halves are `impl FunctionBuilder`
//! blocks; the boundary is pinned by `scripts/check-module-boundaries.sh`.
//!
//! # Why these five did not exist
//!
//! Before batch 6 `Temporal.ZonedDateTime.prototype` carried 18 accessors and
//! exactly four data methods. `zdt.add(duration)` therefore read `undefined`
//! off the prototype and the call site threw
//! `TypeError: value is not callable` — the label measured on all 28
//! `intl402/Temporal/ZonedDateTime/prototype/{add,subtract,since,until}/era-boundary-*.js`
//! cases. Nothing was mis-rooted and no internal `AddZonedDateTime` was
//! stranded: the members simply were not there.
//!
//! # The shape, and its limit
//!
//! `add`/`subtract`/`until`/`since` are **composition, not new arithmetic**:
//!
//! ```text
//! ZonedDateTime --toPlainDateTime--> PlainDateTime
//!                                        |  Temporal.PlainDateTime.prototype.{add,subtract,until,since}
//!                                        v
//!                     PlainDateTime --toZonedDateTime(receiver's time zone)--> ZonedDateTime
//!                     (until/since stop at the Duration and return it)
//! ```
//!
//! Both legs already exist and are already correct: `toPlainDateTime` IS the
//! spec's `GetPlainDateTimeFor` and carries the receiver's calendar payload, so
//! era-aware calendar arithmetic happens in the PlainDateTime bodies with the
//! right calendar; `toZonedDateTime` IS `GetEpochNanosecondsFor` and carries the
//! calendar back. Delegating rather than re-deriving is what keeps one
//! implementation of `AddISODate`/`CalendarDateUntil` in the tree.
//!
//! **STATED LIMIT — this is not the spec answer across a DST transition.**
//! `AddDurationToZonedDateTime` adds the date part in the calendar, re-derives
//! epoch nanoseconds with `compatible` disambiguation, and only then adds the
//! time part as an absolute duration; `DifferenceZonedDateTime` does the mirror
//! image. The round trip here does *all* of it in local wall-clock time, which
//! agrees with the spec exactly when the local offset is constant across the
//! interval. Every one of the 28 era-boundary cases uses `timeZone: "UTC"`, and
//! this backend resolves only `UTC` and fixed numeric offsets anyway (see
//! `emit_temporal_plain_date_time_to_zoned_date_time`'s own note), so the gap is
//! unreachable from the currently supported time-zone set — but it is a real
//! gap the moment named IANA zones land, and it is recorded as owned debt in
//! `target/lane-notes/zdt-arithmetic-surface-b6-integration.md` rather than left
//! for a green gate to imply it is finished.
//!
//! `withCalendar` is not composition and carries no such caveat: it rewrites one
//! field of the record, so it is exact.

use super::super::*;

/// Which of the two arithmetic members is being emitted.
///
/// This was a `bool` named `subtract`, copying
/// `emit_temporal_plain_date_time_add_or_subtract`. The two call sites are
/// adjacent arms of one `match` in `builtins/standard.rs` spelled
/// `(false, function)` and `(true, function)`, so a transposition compiles,
/// formats, passes every type check, and makes `zdt.add(d)` subtract — a
/// silent wrong answer in the exact family this lane exists to fix. The
/// closed set gets a closed type, and the delegate it maps to is a total
/// function rather than an `if` inside the emitter.
enum ZonedDateTimeArithmetic {
    Add,
    Subtract,
}

impl ZonedDateTimeArithmetic {
    /// The `Temporal.PlainDateTime.prototype` member this delegates to. The
    /// sign flip lives inside that one body, which is the only place in the
    /// tree that knows how to negate all ten duration slots.
    fn plain_date_time_builtin(self) -> StandardBuiltinId {
        match self {
            Self::Add => StandardBuiltinId::TemporalPlainDateTimePrototypeAdd,
            Self::Subtract => StandardBuiltinId::TemporalPlainDateTimePrototypeSubtract,
        }
    }
}

/// Which of the two difference members is being emitted. Same reasoning as
/// [`ZonedDateTimeArithmetic`]: `until` and `since` differ only in operand
/// order, so a transposed `bool` produces a correctly-shaped `Temporal.Duration`
/// with the wrong sign.
enum ZonedDateTimeDifference {
    Until,
    Since,
}

impl ZonedDateTimeDifference {
    fn plain_date_time_builtin(self) -> StandardBuiltinId {
        match self {
            Self::Until => StandardBuiltinId::TemporalPlainDateTimePrototypeUntil,
            Self::Since => StandardBuiltinId::TemporalPlainDateTimePrototypeSince,
        }
    }
}

impl<'a> FunctionBuilder<'a> {
    /// Temporal proposal 6.3.x `Temporal.ZonedDateTime.prototype.withCalendar`.
    ///
    /// Structurally the twin of
    /// [`Self::emit_temporal_zoned_date_time_with_time_zone`]: read the
    /// receiver's record, resolve the one argument, and allocate a fresh
    /// ZonedDateTime with that one slot replaced. The epoch nanoseconds are
    /// unchanged, which is the whole content of the operation — a calendar
    /// re-labels an instant, it does not move it.
    ///
    /// This is on the critical path for the gate, not a bonus: `since` and
    /// `until`'s `era-boundary-*.js` files call `one.withCalendar("iso8601")`
    /// to build the ISO oracle they compare the `weeks`/`days` answers against
    /// (`since/era-boundary-gregory.js:65`), so those 14 cases need five
    /// callables, not four.
    pub(crate) fn emit_temporal_zoned_date_time_with_calendar(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let calendar_tag_local = self.reserve_temp_local();
        let epoch_payload_local = self.reserve_temp_local();
        let epoch_tag_local = self.reserve_temp_local();
        let time_zone_payload_local = self.reserve_temp_local();
        let time_zone_tag_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();

        self.emit_temporal_zoned_date_time_record_from_receiver(record_local, function)?;
        self.emit_builtin_arg_to_locals(0, calendar_payload_local, calendar_tag_local, function);
        // `ToTemporalCalendarIdentifier` treats `undefined` as `iso8601`, which
        // is right for a property bag with no `calendar` key and wrong here:
        // `zdt.withCalendar()` must throw rather than silently switch the
        // receiver to ISO. `emit_temporal_plain_date_time_with_calendar` guards
        // the same way, for the same reason.
        function.instruction(&Instruction::LocalGet(calendar_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.ZonedDateTime calendar must be a string",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_temporal_to_temporal_calendar_identifier(
            calendar_payload_local,
            calendar_tag_local,
            "Temporal.ZonedDateTime calendar must be a string",
            function,
        )?;
        // The identifier helper leaves a canonical calendar string in
        // `calendar_payload_local` and a `String` tag in `calendar_tag_local`
        // on every path it returns from, so the record's calendar tag is
        // written from the local rather than re-asserted as a constant.
        for (offset, local) in [
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
                epoch_payload_local,
            ),
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_TAG_OFFSET,
                epoch_tag_local,
            ),
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_PAYLOAD_OFFSET,
                time_zone_payload_local,
            ),
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_TAG_OFFSET,
                time_zone_tag_local,
            ),
        ] {
            self.load_i64_to_local_from_offset(record_local, offset, local, function);
        }
        function.instruction(&Instruction::GlobalGet(
            TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_payload_local));
        self.emit_alloc_temporal_zoned_date_time(
            epoch_payload_local,
            epoch_tag_local,
            time_zone_payload_local,
            time_zone_tag_local,
            calendar_payload_local,
            calendar_tag_local,
            prototype_payload_local,
            function,
        )?;

        for local in [
            prototype_payload_local,
            time_zone_tag_local,
            time_zone_payload_local,
            epoch_tag_local,
            epoch_payload_local,
            calendar_tag_local,
            calendar_payload_local,
            record_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(super) fn emit_temporal_zoned_date_time_add_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_zoned_date_time_add_or_subtract(ZonedDateTimeArithmetic::Add, function)
    }

    pub(super) fn emit_temporal_zoned_date_time_subtract_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_zoned_date_time_add_or_subtract(
            ZonedDateTimeArithmetic::Subtract,
            function,
        )
    }

    pub(super) fn emit_temporal_zoned_date_time_until_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_zoned_date_time_until_or_since(ZonedDateTimeDifference::Until, function)
    }

    pub(super) fn emit_temporal_zoned_date_time_since_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_zoned_date_time_until_or_since(ZonedDateTimeDifference::Since, function)
    }

    /// Temporal proposal 6.3.x `add` and `subtract`, both through the
    /// PlainDateTime round trip described in this module's header.
    ///
    /// `subtract` is not negated here: the sign flip happens once, inside
    /// [`Self::emit_temporal_plain_date_time_add_or_subtract`], which is the
    /// only place in the tree that knows how to negate all ten duration slots.
    /// This emitter chooses which of the two PlainDateTime builtins to call and
    /// nothing else, so `add`/`subtract` cannot disagree about what a negative
    /// `months` means.
    ///
    /// # DIVERGENCE 5: operation order (the one the lane's list of four missed)
    ///
    /// Leg 1 below runs the receiver -> PlainDateTime conversion *before* the
    /// duration argument and the options bag are coerced, because both
    /// coercions live inside the PlainDateTime delegate
    /// (`ToTemporalDuration`, then the overflow option). Leg 1 is a real throw
    /// site: it reaches `emit_alloc_temporal_plain_date_time`, whose first act
    /// is `emit_temporal_reject_date_time_lower_bound` — which is precisely why
    /// the `emit_return_current_completion_if_throw` below it exists.
    ///
    /// Spec `AddDurationToZonedDateTime` does `ToTemporalDuration` first and
    /// `GetPlainDateTimeFor` later. So for a receiver near the representable
    /// limit combined with a duration bag carrying observable or throwing
    /// getters, the spec reports the argument's error and this reports
    /// RangeError. Witness in the 566-case hand-off:
    /// `built-ins/Temporal/ZonedDateTime/prototype/add/`
    /// `throw-when-intermediate-datetime-outside-valid-limits.js`. The plain
    /// `add/order-of-operations.js` does *not* find it — its receiver is
    /// `new Temporal.ZonedDateTime(0n, "UTC")`, nowhere near a limit.
    ///
    /// Not fixed here. The fix is a duration/options pre-coercion hoisted ahead
    /// of leg 1, which costs a second `ToTemporalDuration` call site, and it
    /// should not be done without a run.
    fn emit_temporal_zoned_date_time_add_or_subtract(
        &mut self,
        arithmetic: ZonedDateTimeArithmetic,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let time_zone_payload_local = self.reserve_temp_local();
        let time_zone_tag_local = self.reserve_temp_local();
        let duration_payload_local = self.reserve_temp_local();
        let duration_tag_local = self.reserve_temp_local();
        let options_payload_local = self.reserve_temp_local();
        let options_tag_local = self.reserve_temp_local();
        let plain_payload_local = self.reserve_temp_local();
        let plain_tag_local = self.reserve_temp_local();
        let shifted_payload_local = self.reserve_temp_local();
        let shifted_tag_local = self.reserve_temp_local();
        let zoned_payload_local = self.reserve_temp_local();
        let zoned_tag_local = self.reserve_temp_local();

        // Brand check first, and it is the observable one: a non-ZonedDateTime
        // receiver must throw before either argument is coerced.
        self.emit_temporal_zoned_date_time_record_from_receiver(record_local, function)?;
        for (offset, local) in [
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_PAYLOAD_OFFSET,
                time_zone_payload_local,
            ),
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_TAG_OFFSET,
                time_zone_tag_local,
            ),
        ] {
            self.load_i64_to_local_from_offset(record_local, offset, local, function);
        }
        self.emit_builtin_arg_to_locals(0, duration_payload_local, duration_tag_local, function);
        self.emit_builtin_arg_to_locals(1, options_payload_local, options_tag_local, function);

        // Leg 1: GetPlainDateTimeFor. Emitted inline rather than called,
        // because its receiver is *this* function's `this`. It carries the
        // receiver's calendar payload into the PlainDateTime, which is what
        // makes the arithmetic below era-aware.
        //
        // Inlining costs the throw propagation an explicit line:
        // `emit_direct_js_call` ends in `emit_propagate_throw_from_locals_if_needed`,
        // an inline emitter does not. `CreateTemporalDateTime` runs
        // `ISODateTimeWithinLimits`, so this is reachable, and without the
        // guard a throw completion would be handed on as a *receiver*.
        self.emit_temporal_zoned_date_time_to_plain_date_time(function)?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalSet(plain_payload_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalSet(plain_tag_local));

        // Leg 2: the calendar arithmetic itself, unchanged and unduplicated.
        let arithmetic_builtin = arithmetic.plain_date_time_builtin();
        let arithmetic_meta = self
            .functions
            .get(&arithmetic_builtin.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `{}`",
                    arithmetic_builtin.debug_name()
                ))
            })?;
        self.emit_direct_js_call(
            &arithmetic_meta,
            Some((plain_payload_local, Some(plain_tag_local))),
            &[
                (duration_payload_local, duration_tag_local),
                (options_payload_local, options_tag_local),
            ],
            shifted_payload_local,
            shifted_tag_local,
            function,
        )?;

        // Leg 3: back to the receiver's time zone. The record already holds a
        // canonical time-zone string, so this is the same value the receiver
        // was built with, not a re-parse of user input.
        let to_zoned_meta = self
            .functions
            .get(&StandardBuiltinId::TemporalPlainDateTimePrototypeToZonedDateTime.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Temporal.PlainDateTime.prototype.toZonedDateTime`",
                )
            })?;
        self.emit_direct_js_call(
            &to_zoned_meta,
            Some((shifted_payload_local, Some(shifted_tag_local))),
            &[(time_zone_payload_local, time_zone_tag_local)],
            zoned_payload_local,
            zoned_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(zoned_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(zoned_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        for local in [
            zoned_tag_local,
            zoned_payload_local,
            shifted_tag_local,
            shifted_payload_local,
            plain_tag_local,
            plain_payload_local,
            options_tag_local,
            options_payload_local,
            duration_tag_local,
            duration_payload_local,
            time_zone_tag_local,
            time_zone_payload_local,
            record_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Temporal proposal 6.3.x `until` and `since`, both through
    /// `DifferenceTemporalZonedDateTime`.
    ///
    /// The result is a `Temporal.Duration`, which is why the ZonedDateTime arm
    /// of `RuntimeBootstrapPlan::require_standard_builtin` roots
    /// `TemporalDurationConstructor` — see the comment there; without it the
    /// PlainDateTime body this delegates to allocates against a prototype
    /// global nothing bootstrapped.
    ///
    /// Two spec checks are performed *here* rather than being left to the
    /// PlainDateTime delegate, because the delegate cannot see them: the
    /// PlainDateTime pair produced by the round trip has already lost the time
    /// zone, and the calendar check has to name the ZonedDateTime operation in
    /// its message.
    ///
    /// ORDERING DIVERGENCE, recorded rather than hidden: the spec reads the
    /// options bag (`GetDifferenceSettings`) *between* the calendar check and
    /// the time-zone check. Here both checks run first and the options bag is
    /// read immediately before the delegate. That is observable only for an
    /// options object with side-effecting getters passed together with a
    /// mismatched time zone; none of the 28 era-boundary cases is such a case
    /// (all are `"UTC"` on both sides with a plain object literal for options).
    ///
    /// The two types' default-largest-unit difference is resolved before that
    /// runtime call. One shared settings producer uses the ZonedDateTime
    /// `"hour"` fallback, and a linear witness is consumed into an unreachable
    /// normalized options object. The PlainDateTime delegate therefore sees an
    /// explicit largest unit while every user getter and conversion hook is
    /// still observed once. This keeps one copy of the arithmetic body and is
    /// pinned by the `defaults-to-returning-hours`, `largestunit-undefined` and
    /// `largestunit-default` families for both operations.
    fn emit_temporal_zoned_date_time_until_or_since(
        &mut self,
        difference: ZonedDateTimeDifference,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let time_zone_payload_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let options_payload_local = self.reserve_temp_local();
        let options_tag_local = self.reserve_temp_local();
        let delegate_options_payload_local = self.reserve_temp_local();
        let delegate_options_tag_local = self.reserve_temp_local();
        let other_payload_local = self.reserve_temp_local();
        let other_tag_local = self.reserve_temp_local();
        let other_record_local = self.reserve_temp_local();
        let other_time_zone_payload_local = self.reserve_temp_local();
        let other_calendar_payload_local = self.reserve_temp_local();
        let plain_payload_local = self.reserve_temp_local();
        let plain_tag_local = self.reserve_temp_local();
        let other_plain_payload_local = self.reserve_temp_local();
        let other_plain_tag_local = self.reserve_temp_local();
        let duration_payload_local = self.reserve_temp_local();
        let duration_tag_local = self.reserve_temp_local();

        self.emit_temporal_zoned_date_time_record_from_receiver(record_local, function)?;
        for (offset, local) in [
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_PAYLOAD_OFFSET,
                time_zone_payload_local,
            ),
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_CALENDAR_PAYLOAD_OFFSET,
                calendar_payload_local,
            ),
        ] {
            self.load_i64_to_local_from_offset(record_local, offset, local, function);
        }
        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        self.emit_builtin_arg_to_locals(1, options_payload_local, options_tag_local, function);

        // `ToTemporalZonedDateTime(other)`. Same idiom
        // `emit_temporal_zoned_date_time_equals` uses, and for the same reason:
        // `from` is the only entry point that accepts every legal spelling of
        // `other` (an instance, a property bag, an ISO string).
        let from_meta = self
            .functions
            .get(&StandardBuiltinId::TemporalZonedDateTimeFrom.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Temporal.ZonedDateTime.from`",
                )
            })?;
        self.emit_direct_js_call(
            &from_meta,
            None,
            &[(argument_payload_local, argument_tag_local)],
            other_payload_local,
            other_tag_local,
            function,
        )?;
        self.load_i64_to_local_from_offset(
            other_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            other_record_local,
            function,
        );
        for (offset, local) in [
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_PAYLOAD_OFFSET,
                other_time_zone_payload_local,
            ),
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_CALENDAR_PAYLOAD_OFFSET,
                other_calendar_payload_local,
            ),
        ] {
            self.load_i64_to_local_from_offset(other_record_local, offset, local, function);
        }
        // `CalendarEquals` — RangeError, and before any option is read.
        self.emit_temporal_require_same_calendar(
            calendar_payload_local,
            other_calendar_payload_local,
            TemporalDifferenceGuard::ZonedDateTimeSameCalendar,
            function,
        )?;
        // `TimeZoneEquals`. Comparing the two canonical time-zone strings is
        // sound for exactly the set of zones this backend resolves (`UTC` and
        // fixed numeric offsets, canonicalised on the way into the record), and
        // it is what makes the wall-clock round trip below a legitimate
        // implementation of the difference: with one shared constant offset,
        // local-time arithmetic and instant arithmetic agree.
        //
        // DELIBERATELY OVER-STRICT, and this is the choice, not an oversight.
        // The spec applies `TimeZoneEquals` only when `largestUnit` is a *date*
        // unit; a time-unit difference across two zones is legal and answers
        // the plain instant difference. `largestUnit` is resolved into a
        // linear settings witness below. Moving that read
        // before this guard and conditioning the guard on the witness would
        // repair a separate observable-order/cross-zone seam; it is
        // deliberately not ridden along here. Given the choice between
        // throwing a spec-shaped RangeError in a case the spec allows and
        // returning a *wrong number* (differencing two wall-clock readings
        // taken in different zones is meaningless), this takes the loud error.
        // Cross-zone time-unit differences remain a known, named gap.
        self.emit_string_payload_equality_i32(
            time_zone_payload_local,
            other_time_zone_payload_local,
            function,
        );
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            TemporalDifferenceGuard::ZonedDateTimeSameTimeZone.message(),
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        // Both sides to PlainDateTime: the receiver inline (its `this` is ours),
        // `other` through a call because its `this` is not. The explicit throw
        // guard is the one `emit_direct_js_call` would have supplied; see the
        // sibling comment in `emit_temporal_zoned_date_time_add_or_subtract`.
        self.emit_temporal_zoned_date_time_to_plain_date_time(function)?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalSet(plain_payload_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalSet(plain_tag_local));

        let to_plain_meta = self
            .functions
            .get(&StandardBuiltinId::TemporalZonedDateTimePrototypeToPlainDateTime.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Temporal.ZonedDateTime.prototype.toPlainDateTime`",
                )
            })?;
        self.emit_direct_js_call(
            &to_plain_meta,
            Some((other_payload_local, Some(other_tag_local))),
            &[],
            other_plain_payload_local,
            other_plain_tag_local,
            function,
        )?;

        // Resolve the user bag under ZonedDateTime's hour fallback exactly
        // once. The returned private bag contains only validated primitives,
        // so the existing PlainDateTime body can read it without repeating an
        // observable getter or conversion.
        self.emit_temporal_zoned_date_time_difference_delegate_options(
            options_payload_local,
            options_tag_local,
            delegate_options_payload_local,
            delegate_options_tag_local,
            function,
        )?;

        let difference_builtin = difference.plain_date_time_builtin();
        let difference_meta = self
            .functions
            .get(&difference_builtin.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `{}`",
                    difference_builtin.debug_name()
                ))
            })?;
        self.emit_direct_js_call(
            &difference_meta,
            Some((plain_payload_local, Some(plain_tag_local))),
            &[
                (other_plain_payload_local, other_plain_tag_local),
                (delegate_options_payload_local, delegate_options_tag_local),
            ],
            duration_payload_local,
            duration_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(duration_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(duration_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        for local in [
            duration_tag_local,
            duration_payload_local,
            other_plain_tag_local,
            other_plain_payload_local,
            plain_tag_local,
            plain_payload_local,
            other_calendar_payload_local,
            other_time_zone_payload_local,
            other_record_local,
            other_tag_local,
            other_payload_local,
            delegate_options_tag_local,
            delegate_options_payload_local,
            options_tag_local,
            options_payload_local,
            argument_tag_local,
            argument_payload_local,
            calendar_payload_local,
            time_zone_payload_local,
            record_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
