//! `Temporal.Instant` members that need no duration arithmetic, no rounding, no
//! time zone and no calendar.
//!
//! Temporal proposal 8.2.4 `compare`, 8.2.2 `fromEpochMilliseconds`, 8.2.3
//! `fromEpochNanoseconds`, 8.3.11 `toJSON` and 8.3.12 `valueOf`. Every one of
//! them is a thin composition of machinery `Temporal.Instant.from`,
//! `prototype.equals` and `prototype.toString` already exercise, so the record
//! layout, the range check and the allocation all live in
//! `builtins/temporal.rs` and are reused unchanged from here.
//!
//! The one new idea is the [`UnvalidatedEpochNanoseconds`] ->
//! [`EpochNanoseconds`] pair. The first is a named-field `(payload, tag)` local
//! pair, so the two same-typed locals cannot be swapped in silence at a call
//! site; the second is that pair after `IsValidEpochNanoseconds` accepted it.
//! `EpochNanoseconds`'s field is private to this module and its only
//! constructor is [`FunctionBuilder::emit_temporal_instant_validated_epoch`],
//! so an allocation path in this file cannot skip the range check — the type it
//! needs simply cannot be produced any other way.

use super::super::*;
use crate::operations::BigIntNumberPolicy;

/// `Temporal.Instant.fromEpochMilliseconds` step 2 rejects a non-integral
/// Number through `NumberToBigInt`, which is a **RangeError**, not the
/// TypeError the surrounding coercions raise. `fromEpochMilliseconds/
/// non-integer.js` pins `undefined`, `±Infinity`, `NaN`, `1.3` and `-0.5`.
///
/// Interned unconditionally in the `data.rs` string pool: `StringPool::payload`
/// panics for a message that is not there, so a missing row is a compiler
/// panic for every program that roots `Temporal.Instant`, not a wrong answer.
pub(crate) const TEMPORAL_INSTANT_NON_INTEGRAL_EPOCH_MILLISECONDS_MESSAGE: &str =
    "Temporal.Instant.fromEpochMilliseconds requires an integral Number";

/// `Temporal.Instant.prototype.valueOf` exists only to make implicit numeric
/// conversion throw; `valueOf/basic.js` asserts that `instant < instant` is a
/// TypeError, which only holds once this property shadows the ordinary
/// `OrdinaryToPrimitive` fallthrough to `toString`.
///
/// Interned unconditionally in the `data.rs` string pool, for the same reason
/// as the message above.
pub(crate) const TEMPORAL_INSTANT_VALUE_OF_MESSAGE: &str =
    "Temporal.Instant does not support implicit conversion; use compare() or equals()";

/// A `(payload, tag)` local pair holding epoch nanoseconds that nothing has
/// range-checked yet.
///
/// Named fields rather than two positional `u32` parameters. The two locals are
/// the same type and both spelled `..._local`, so a positional pair let a call
/// site swap them in silence — and a swapped tag is not a loud failure: it makes
/// `emit_temporal_instant_validate_range` miss its `tag == HEAP_BIGINT_VALUE_TAG`
/// test, take the else branch, and skip validation altogether, quietly accepting
/// an out-of-range epoch. Spelled at a struct literal, the swap has to be
/// written down.
#[derive(Clone, Copy)]
pub(crate) struct UnvalidatedEpochNanoseconds {
    pub(crate) payload_local: u32,
    pub(crate) tag_local: u32,
}

/// The same pair, after `emit_temporal_instant_validate_range` has accepted it.
///
/// The wrapped field is module-private and the only constructor is
/// [`FunctionBuilder::emit_temporal_instant_validated_epoch`], so an allocation
/// path in this file cannot skip the range check — the type it needs simply
/// cannot be produced any other way. `IsValidEpochNanoseconds` is therefore
/// checked exactly once per allocation site, and "I forgot to range-check"
/// stops being expressible rather than being left for a test to notice.
#[derive(Clone, Copy)]
struct EpochNanoseconds(UnvalidatedEpochNanoseconds);

impl<'a> FunctionBuilder<'a> {
    /// The only way to build an [`EpochNanoseconds`].
    ///
    /// `emit_temporal_instant_validate_range` returns normally when the value is
    /// in range and emits a throwing early return when it is not, so the token
    /// this hands back is only observed on the in-range path.
    fn emit_temporal_instant_validated_epoch(
        &mut self,
        epoch: UnvalidatedEpochNanoseconds,
        function: &mut Function,
    ) -> Result<EpochNanoseconds, EmitError> {
        self.emit_temporal_instant_validate_range(epoch.payload_local, epoch.tag_local, function)?;
        Ok(EpochNanoseconds(epoch))
    }

    /// `CreateTemporalInstant(epochNanoseconds)` with the *intrinsic*
    /// `%Temporal.Instant.prototype%`.
    ///
    /// Both `fromEpochMilliseconds` and `fromEpochNanoseconds` are plain
    /// functions, not constructors: `subclassing-ignored.js` requires that the
    /// receiver is never consulted, so this deliberately does not go through
    /// `emit_error_new_target_prototype_to_local` the way the
    /// `Temporal.Instant` constructor does.
    fn emit_alloc_validated_temporal_instant(
        &mut self,
        epoch: EpochNanoseconds,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let prototype_payload_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(
            TEMPORAL_INSTANT_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_payload_local));
        self.emit_alloc_temporal_instant(
            epoch.0.payload_local,
            epoch.0.tag_local,
            prototype_payload_local,
            function,
        )?;
        self.release_temp_local(prototype_payload_local);
        Ok(())
    }

    /// `ℤ(epochMilliseconds) × 10^6`, in whichever BigInt representation fits.
    ///
    /// The product needs up to 128 bits, so it is built from two 32-bit limbs
    /// of the magnitude: the inline `i64` form is only usable when the high
    /// limb is zero and the low limb fits a signed `i64`, and everything else
    /// becomes a two-limb heap BigInt with an explicit sign word.
    ///
    /// `milliseconds_local` is a signed `i64`. Shared by
    /// `Date.prototype.toTemporalInstant` and
    /// `Temporal.Instant.fromEpochMilliseconds`, which differ only in how they
    /// obtain it — the widening itself is identical, and duplicating it once
    /// already produced two copies that could drift apart on the sign path.
    pub(crate) fn emit_temporal_epoch_milliseconds_to_epoch_nanoseconds(
        &mut self,
        milliseconds_local: u32,
        destination: UnvalidatedEpochNanoseconds,
        function: &mut Function,
    ) -> Result<UnvalidatedEpochNanoseconds, EmitError> {
        // Handed back so a caller chains `produce -> validate` on one value
        // instead of naming the same two locals again at the validator.
        let UnvalidatedEpochNanoseconds {
            payload_local: nanoseconds_payload_local,
            tag_local: nanoseconds_tag_local,
        } = destination;
        let magnitude_local = self.reserve_temp_local();
        let negative_local = self.reserve_temp_local();
        let low_word_local = self.reserve_temp_local();
        let low_product_local = self.reserve_temp_local();
        let high_product_local = self.reserve_temp_local();
        let low_limb_local = self.reserve_temp_local();
        let high_limb_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(milliseconds_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(negative_local));
        function.instruction(&Instruction::LocalGet(negative_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(milliseconds_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(milliseconds_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(magnitude_local));

        function.instruction(&Instruction::LocalGet(magnitude_local));
        function.instruction(&Instruction::I64Const(u32::MAX as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(low_word_local));
        function.instruction(&Instruction::LocalGet(low_word_local));
        function.instruction(&Instruction::I64Const(1_000_000));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(low_product_local));
        function.instruction(&Instruction::LocalGet(magnitude_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(1_000_000));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(high_product_local));
        function.instruction(&Instruction::LocalGet(low_product_local));
        function.instruction(&Instruction::LocalGet(high_product_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(low_limb_local));
        function.instruction(&Instruction::LocalGet(high_product_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalGet(low_limb_local));
        function.instruction(&Instruction::LocalGet(low_product_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(high_limb_local));

        function.instruction(&Instruction::LocalGet(high_limb_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(low_limb_local));
        function.instruction(&Instruction::I64Const(i64::MAX));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(negative_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(low_limb_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(low_limb_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(nanoseconds_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
        function.instruction(&Instruction::LocalSet(nanoseconds_tag_local));
        function.instruction(&Instruction::Else);
        let record_local = self.reserve_temp_local();
        let limbs_local = self.reserve_temp_local();
        let limb_count_local = self.reserve_temp_local();
        self.emit_heap_alloc_const(HEAP_BIGINT_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(record_local));
        self.emit_heap_alloc_const(16, function)?;
        function.instruction(&Instruction::LocalSet(limbs_local));
        self.store_i64_local_at_offset(limbs_local, 0, low_limb_local, function);
        self.store_i64_local_at_offset(limbs_local, 8, high_limb_local, function);
        function.instruction(&Instruction::LocalGet(high_limb_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(limb_count_local));
        function.instruction(&Instruction::LocalGet(negative_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(low_word_local));
        self.store_i64_local_at_offset(
            record_local,
            HEAP_BIGINT_SIGN_OFFSET,
            low_word_local,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_BIGINT_LIMBS_PTR_OFFSET,
            limbs_local,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_BIGINT_LIMBS_LEN_OFFSET,
            limb_count_local,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_BIGINT_LIMBS_CAP_OFFSET,
            limb_count_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(record_local));
        function.instruction(&Instruction::LocalSet(nanoseconds_payload_local));
        function.instruction(&Instruction::I64Const(HEAP_BIGINT_VALUE_TAG));
        function.instruction(&Instruction::LocalSet(nanoseconds_tag_local));
        self.release_temp_local(limb_count_local);
        self.release_temp_local(limbs_local);
        self.release_temp_local(record_local);
        function.instruction(&Instruction::End);

        for local in [
            high_limb_local,
            low_limb_local,
            high_product_local,
            low_product_local,
            low_word_local,
            negative_local,
            magnitude_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(destination)
    }

    /// Temporal proposal 8.2.4 `Temporal.Instant.compare`.
    ///
    /// ```text
    /// 1. Set one to ? ToTemporalInstant(one).
    /// 2. Set two to ? ToTemporalInstant(two).
    /// 3. Return 𝔽(CompareEpochNanoseconds(one.[[EpochNanoseconds]],
    ///                                     two.[[EpochNanoseconds]])).
    /// ```
    ///
    /// `Temporal.Instant.from` *is* `ToTemporalInstant`, including the
    /// `[[InitializedTemporalZonedDateTime]]` fast path
    /// `compare/argument-zoneddatetime.js` exercises, so both arguments are
    /// routed through it exactly the way `prototype.equals` routes its single
    /// argument. The two calls stay strictly ordered: `compare/
    /// argument-string-with-offset-not-valid-epoch-nanoseconds.js` throws a
    /// `Test262Error` from the second argument's `toString` and requires the
    /// first argument's RangeError to win.
    ///
    /// The three-way result is folded from two relational comparisons rather
    /// than a fresh sentinel, so the `-1`/`0`/`1` domain has one definition —
    /// the same shape `emit_temporal_plain_date_compare` ends with.
    pub(crate) fn emit_temporal_instant_compare(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let instant_payload_local = self.reserve_temp_local();
        let instant_tag_local = self.reserve_temp_local();
        let record_local = self.reserve_temp_local();
        let left_epoch_payload_local = self.reserve_temp_local();
        let left_epoch_tag_local = self.reserve_temp_local();
        let right_epoch_payload_local = self.reserve_temp_local();
        let right_epoch_tag_local = self.reserve_temp_local();
        let comparison_local = self.reserve_temp_local();

        let instant_from_meta = self
            .functions
            .get(&StandardBuiltinId::TemporalInstantFrom.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Temporal.Instant.from`",
                )
            })?;

        for (index, epoch_payload_local, epoch_tag_local) in [
            (0, left_epoch_payload_local, left_epoch_tag_local),
            (1, right_epoch_payload_local, right_epoch_tag_local),
        ] {
            self.emit_builtin_arg_to_locals(
                index,
                argument_payload_local,
                argument_tag_local,
                function,
            );
            self.emit_direct_js_call(
                &instant_from_meta,
                None,
                &[(argument_payload_local, argument_tag_local)],
                instant_payload_local,
                instant_tag_local,
                function,
            )?;
            self.load_i64_to_local_from_offset(
                instant_payload_local,
                HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
                record_local,
                function,
            );
            self.load_i64_to_local_from_offset(
                record_local,
                HEAP_TEMPORAL_INSTANT_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
                epoch_payload_local,
                function,
            );
            self.load_i64_to_local_from_offset(
                record_local,
                HEAP_TEMPORAL_INSTANT_EPOCH_NANOSECONDS_TAG_OFFSET,
                epoch_tag_local,
                function,
            );
        }

        // One call into `bigint_arithmetic_helper`, not two. The helper's own
        // answer is already the `-1`/`0`/`1` this returns; asking
        // `emit_bigint_relational_i32` for `<` and then for `>` would pay for
        // it twice and give the three-way domain two producers.
        self.emit_bigint_compare_i64(
            left_epoch_payload_local,
            left_epoch_tag_local,
            right_epoch_payload_local,
            right_epoch_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(comparison_local));

        function.instruction(&Instruction::LocalGet(comparison_local));
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        for local in [
            comparison_local,
            right_epoch_tag_local,
            right_epoch_payload_local,
            left_epoch_tag_local,
            left_epoch_payload_local,
            record_local,
            instant_tag_local,
            instant_payload_local,
            argument_tag_local,
            argument_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Temporal proposal 8.2.3 `Temporal.Instant.fromEpochNanoseconds`.
    ///
    /// ```text
    /// 1. Set epochNanoseconds to ? ToBigInt(epochNanoseconds).
    /// 2. If IsValidEpochNanoseconds(epochNanoseconds) is false, throw RangeError.
    /// 3. Return ! CreateTemporalInstant(epochNanoseconds).
    /// ```
    ///
    /// This is the `Temporal.Instant` constructor minus the `new.target` block:
    /// `ToBigInt` (not `ToNumber`) is why `fromEpochNanoseconds(42)` and
    /// `fromEpochNanoseconds(null)` are TypeErrors while `(-limit - 1n)` is a
    /// RangeError.
    pub(crate) fn emit_temporal_instant_from_epoch_nanoseconds(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let nanoseconds_payload_local = self.reserve_temp_local();
        let nanoseconds_tag_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        self.emit_value_to_bigint_locals(
            argument_tag_local,
            argument_payload_local,
            BigIntNumberPolicy::RejectNumber,
            nanoseconds_payload_local,
            nanoseconds_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        let epoch = self.emit_temporal_instant_validated_epoch(
            UnvalidatedEpochNanoseconds {
                payload_local: nanoseconds_payload_local,
                tag_local: nanoseconds_tag_local,
            },
            function,
        )?;
        self.emit_alloc_validated_temporal_instant(epoch, function)?;

        for local in [
            nanoseconds_tag_local,
            nanoseconds_payload_local,
            argument_tag_local,
            argument_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Temporal proposal 8.2.2 `Temporal.Instant.fromEpochMilliseconds`.
    ///
    /// ```text
    /// 1. Set epochMilliseconds to ? ToNumber(epochMilliseconds).
    /// 2. Set epochMilliseconds to ? NumberToBigInt(epochMilliseconds).
    /// 3. Let epochNanoseconds be epochMilliseconds × ℤ(10**6).
    /// 4. If IsValidEpochNanoseconds(epochNanoseconds) is false, throw RangeError.
    /// 5. Return ! CreateTemporalInstant(epochNanoseconds).
    /// ```
    ///
    /// Step 1 is why `fromEpochMilliseconds(42n)` and `fromEpochMilliseconds(
    /// Symbol())` are TypeErrors; step 2 is why `undefined`, `±Infinity`,
    /// `NaN`, `1.3` and `-0.5` are RangeErrors rather than TypeErrors.
    ///
    /// Step 4 is not re-derived here: `I64TruncSatF64S` saturates a magnitude
    /// beyond `i64` to `i64::MAX`, whose product with `10^6` is still far above
    /// the epoch-nanosecond limit, so the shared range check rejects it for the
    /// same reason it rejects `limit + 1`.
    pub(crate) fn emit_temporal_instant_from_epoch_milliseconds(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let milliseconds_local = self.reserve_temp_local();
        let nanoseconds_payload_local = self.reserve_temp_local();
        let nanoseconds_tag_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        self.emit_value_to_number_payload(argument_tag_local, argument_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(argument_payload_local));
        self.emit_return_current_completion_if_throw(function);

        // `NumberToBigInt` step 1: reject anything that is not an integral
        // Number. `v != trunc(v)` covers NaN (which is unequal to itself) and
        // every fractional value including `-0.5`; `trunc` is the identity on
        // the infinities, so they need the separate magnitude test. `-0` is
        // integral and converts to `0n`, so it must survive both tests.
        function.instruction(&Instruction::LocalGet(argument_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(argument_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::LocalGet(argument_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Abs);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::MAX)));
        function.instruction(&Instruction::F64Gt);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            TEMPORAL_INSTANT_NON_INTEGRAL_EPOCH_MILLISECONDS_MESSAGE,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(argument_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncSatF64S);
        function.instruction(&Instruction::LocalSet(milliseconds_local));
        let widened = self.emit_temporal_epoch_milliseconds_to_epoch_nanoseconds(
            milliseconds_local,
            UnvalidatedEpochNanoseconds {
                payload_local: nanoseconds_payload_local,
                tag_local: nanoseconds_tag_local,
            },
            function,
        )?;
        let epoch = self.emit_temporal_instant_validated_epoch(widened, function)?;
        self.emit_alloc_validated_temporal_instant(epoch, function)?;

        for local in [
            nanoseconds_tag_local,
            nanoseconds_payload_local,
            milliseconds_local,
            argument_tag_local,
            argument_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Temporal proposal 8.3.12 `Temporal.Instant.prototype.valueOf`.
    ///
    /// An unconditional TypeError, before any brand check: step 1 of the
    /// specification is `throw a TypeError exception`, and `valueOf/branding.js`
    /// calls it on `undefined`, `null`, `true`, `""`, a Symbol, `1`, `{}`, the
    /// constructor and the prototype, expecting a TypeError from every one.
    pub(crate) fn emit_temporal_instant_value_of(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_throw_current_function_realm_type_error(
            TEMPORAL_INSTANT_VALUE_OF_MESSAGE,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        Ok(())
    }
}
