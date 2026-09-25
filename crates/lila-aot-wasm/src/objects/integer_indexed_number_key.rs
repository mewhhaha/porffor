//! Integer-indexed exotic objects reached through a Number property key.
//!
//! `ta[i]` with a Number `i` is specified as ToPropertyKey(i) = ToString(i)
//! followed, inside the TypedArray's `[[Get]]`/`[[Set]]`/`[[HasProperty]]`, by
//! CanonicalNumericIndexString of that String. For every Number `n` the round
//! trip is the identity — Number::toString produces the shortest String that
//! parses back to `n` — with the single exception of -0, whose String form is
//! `"0"` and so names +0. ToString of a Number has no side effects and cannot
//! fail, so deferring it is unobservable.
//!
//! This module therefore lets a runtime Number key reach a TypedArray without
//! its String ever being built, which matters because Lila's heap is a bump
//! allocator with no collector: the String (and the canonical-form String the
//! spec's round trip compares it with) would otherwise leak on every element
//! access. Any other receiver still gets the String, materialised only once the
//! integer-indexed case has been ruled out.

use super::*;

/// The property key of an ordinary property Reference after the ToPropertyKey
/// step of GetValue/PutValue.
///
/// Either a canonical property key (a String payload, or a Symbol payload with
/// `PROPERTY_KEY_SYMBOL_MARKER`), or a Number whose ToString was deferred. The
/// Number form is only ever produced for a TypedArray target, which is what
/// lets [`FunctionBuilder::emit_reference_get_value`] and
/// [`FunctionBuilder::emit_reference_set_result`] treat a Number tag as "take
/// the integer-indexed path" without re-checking the target. The fields are
/// private and the only constructor is
/// [`FunctionBuilder::emit_reference_property_key_locals`], so raw key locals
/// that skipped that check cannot reach either consumer.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ReferencePropertyKeyLocals {
    payload: u32,
    tag: u32,
}

impl ReferencePropertyKeyLocals {
    pub(crate) const fn payload(self) -> u32 {
        self.payload
    }

    pub(crate) const fn tag(self) -> u32 {
        self.tag
    }
}

impl FunctionBuilder<'_> {
    /// CanonicalNumericIndexString(ToString(`n`)) for a Number `n`: `n`
    /// itself, except that -0 names +0.
    pub(crate) fn emit_number_property_key_numeric_index(
        &self,
        key_payload_local: u32,
        numeric_index_payload_local: u32,
        function: &mut Function,
    ) {
        // -0 and +0 differ only in the sign bit, so +0 is the i64 zero.
        function.instruction(&Instruction::LocalGet(key_payload_local));
        function.instruction(&Instruction::I64Const((-0.0f64).to_bits() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(key_payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(numeric_index_payload_local));
    }

    /// The element index a Number key names, as the TypedArray witness's
    /// unsigned index: ℝ(n) for an integral `n` in [0, 2^64) (-0 included, as
    /// 0), and `u64::MAX` otherwise. No TypedArray is that long, so an index
    /// that is not a valid integer index — NaN, ±∞, fractional, negative — is
    /// rejected by the witness exactly like one past the end.
    fn emit_number_property_key_element_index(
        &self,
        key_payload_local: u32,
        index_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        // Not integral (NaN compares unequal to itself).
        function.instruction(&Instruction::LocalGet(key_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(key_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::BrIf(0));
        // Negative; -0 is not below 0 and truncates to index 0.
        function.instruction(&Instruction::LocalGet(key_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::BrIf(0));
        function.instruction(&Instruction::LocalGet(key_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(
            18_446_744_073_709_551_616.0,
        )));
        function.instruction(&Instruction::F64Ge);
        function.instruction(&Instruction::BrIf(0));
        function.instruction(&Instruction::LocalGet(key_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::End);
    }

    /// Completes a deferred ToPropertyKey: a Number key becomes ToString of
    /// itself. String and Symbol keys are left untouched.
    pub(crate) fn emit_materialize_number_property_key(
        &mut self,
        key_payload_local: u32,
        key_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_number_to_string_payload(key_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        function.instruction(&Instruction::End);
        Ok(())
    }

    /// ToPropertyKey for a key whose consumer accepts a deferred Number key
    /// (the dynamic `[[Get]]` composite): a Number stays a Number, every other
    /// value is converted in place with its abrupt completion propagated.
    pub(crate) fn emit_value_to_property_key_locals_deferring_number(
        &mut self,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_property_key_locals(payload_local, tag_local, function)?;
        self.emit_propagate_throw_from_locals_if_needed(payload_local, tag_local, function)?;
        function.instruction(&Instruction::End);
        Ok(())
    }

    /// [`Self::compile_object_key_to_locals`] for a consumer that accepts a
    /// deferred Number key (the dynamic `[[Get]]` composite): a computed key
    /// that evaluates to a Number is left a Number instead of being turned
    /// into a String up front.
    pub(crate) fn compile_object_key_to_locals_deferring_number(
        &mut self,
        key: &PropertyKeyIr,
        key_payload_local: u32,
        key_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match key {
            PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => {
                self.compile_object_key_to_locals(key, key_payload_local, key_tag_local, function)
            }
            PropertyKeyIr::StringExpr(expr) if !expr.possible_kinds.contains(ValueKind::Number) => {
                self.compile_object_key_to_locals(key, key_payload_local, key_tag_local, function)
            }
            PropertyKeyIr::StringExpr(expr) => {
                self.compile_expr_to_locals(expr, key_payload_local, key_tag_local, function)?;
                self.emit_propagate_throw_from_locals_if_needed(
                    key_payload_local,
                    key_tag_local,
                    function,
                )?;
                self.emit_value_to_property_key_locals_deferring_number(
                    key_payload_local,
                    key_tag_local,
                    function,
                )?;
                // Same Symbol-key encoding step as `compile_object_key_to_locals`.
                function.instruction(&Instruction::LocalGet(key_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(key_payload_local));
                function.instruction(&Instruction::I64Const(PROPERTY_KEY_SYMBOL_MARKER as i64));
                function.instruction(&Instruction::I64Or);
                function.instruction(&Instruction::LocalSet(key_payload_local));
                function.instruction(&Instruction::End);
                Ok(())
            }
            PropertyKeyIr::ArrayIndex(expr) => {
                self.compile_expr_payload(expr, function)?;
                function.instruction(&Instruction::LocalSet(key_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                function.instruction(&Instruction::LocalSet(key_tag_local));
                Ok(())
            }
        }
    }

    /// `[[Get]]` of an Array element named by a runtime Number key, read with
    /// the Array itself as receiver, when ToString of the Number is an array
    /// index (an integral Number in [0, 2^32 - 1); -0 names index 0). This is
    /// the element read with prototype walk that `a[i]` uses when `a` is
    /// statically an Array, so it needs no String for the key. Sets
    /// `handled_local` to 1 when it answered; any other combination (not an
    /// Array, a different receiver, a key that is not an array index) is left
    /// to the caller's generic path.
    pub(crate) fn emit_array_number_key_get_if_array_index(
        &mut self,
        target: TaggedLocals,
        receiver: TaggedLocals,
        key_payload_local: u32,
        key_tag_local: u32,
        value: TaggedLocals,
        handled_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let index_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(handled_local));
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(target.tag));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(receiver.payload));
        function.instruction(&Instruction::LocalGet(target.payload));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(receiver.tag));
        function.instruction(&Instruction::LocalGet(target.tag));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_number_property_key_element_index(key_payload_local, index_local, function);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(MAX_ARRAY_LENGTH as i64));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_or_object_index_read_from_locals(
            target.payload,
            target.tag,
            index_local,
            value.payload,
            value.tag,
            function,
        )?;
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(handled_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(index_local);
        Ok(())
    }

    /// `i32`: the key is a runtime Number and the object is a TypedArray.
    fn emit_number_key_on_typed_array_i32(
        &mut self,
        object_payload_local: u32,
        object_tag_local: u32,
        key_tag_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_is_typed_array_i32(object_payload_local, object_tag_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::End);
    }

    /// The ToPropertyKey step of an ordinary Reference's GetValue/PutValue,
    /// applied after ToObject produced `target_object`.
    ///
    /// A Number key on a TypedArray target is kept as a Number (see the module
    /// documentation); every other key is converted by ToPropertyKey, whose
    /// abrupt completion is routed to the active handler.
    pub(crate) fn emit_reference_property_key_locals(
        &mut self,
        target_object_payload_local: u32,
        target_object_tag_local: u32,
        key_payload_local: u32,
        key_tag_local: u32,
        function: &mut Function,
    ) -> Result<ReferencePropertyKeyLocals, EmitError> {
        self.emit_number_key_on_typed_array_i32(
            target_object_payload_local,
            target_object_tag_local,
            key_tag_local,
            function,
        );
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_property_key_locals(key_payload_local, key_tag_local, function)?;
        self.emit_propagate_throw_from_locals_if_needed(
            key_payload_local,
            key_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        Ok(ReferencePropertyKeyLocals {
            payload: key_payload_local,
            tag: key_tag_local,
        })
    }

    /// `[[Get]]` of an ordinary Reference whose key went through
    /// [`Self::emit_reference_property_key_locals`].
    ///
    /// A Number key means the target is a TypedArray, whose `[[Get]]` for a
    /// canonical numeric key is TypedArrayGetElement and ignores the receiver.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_reference_get_value(
        &mut self,
        target_object_payload_local: u32,
        target_object_tag_local: u32,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        key: ReferencePropertyKeyLocals,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let index_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(key.tag));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_number_property_key_element_index(key.payload, index_local, function);
        self.emit_typed_array_or_object_index_read_from_locals(
            target_object_payload_local,
            target_object_tag_local,
            index_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_object_read_with_key_tag(
            target_object_payload_local,
            target_object_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key.payload,
            Some(key.tag),
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.release_temp_local(index_local);
        Ok(())
    }

    /// `[[Set]]` of an ordinary Reference whose key went through
    /// [`Self::emit_reference_property_key_locals`], leaving the Boolean
    /// result in `set_result_local` for PutValue's strict-mode check.
    ///
    /// A Number key means the target is a TypedArray; the target is ToObject
    /// of the Reference base and a TypedArray is an object, so the receiver is
    /// the target itself. TypedArray `[[Set]]` with SameValue(O, Receiver)
    /// then returns false for an immutable buffer before touching the value,
    /// and otherwise performs TypedArraySetElement — ToNumber/ToBigInt of the
    /// value first, then the IsValidIntegerIndex check — and returns true.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_reference_set_result(
        &mut self,
        target_object_payload_local: u32,
        target_object_tag_local: u32,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        key: ReferencePropertyKeyLocals,
        value_payload_local: u32,
        value_tag_local: u32,
        set_result_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let index_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(key.tag));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_buffer_is_immutable_i32(target_object_payload_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(set_result_local));
        function.instruction(&Instruction::Else);
        self.emit_number_property_key_element_index(key.payload, index_local, function);
        self.emit_known_typed_array_element_write_from_locals(
            target_object_payload_local,
            target_object_tag_local,
            index_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(set_result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_ordinary_set_result_via_helper(
            target_object_payload_local,
            target_object_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key.payload,
            key.tag,
            value_payload_local,
            value_tag_local,
            set_result_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.release_temp_local(index_local);
        Ok(())
    }

    /// `[[HasProperty]]` for the `in` operator when the key is a runtime
    /// Number and the object a TypedArray: IsValidIntegerIndex of the
    /// Number's canonical numeric index. Sets `handled_local` to 1 when it
    /// answered, leaving every other combination to the generic path.
    pub(crate) fn emit_typed_array_number_key_has_property_i32(
        &mut self,
        object_payload_local: u32,
        object_tag_local: u32,
        key_payload_local: u32,
        key_tag_local: u32,
        result_local: u32,
        handled_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let numeric_index_payload_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(handled_local));
        self.emit_number_key_on_typed_array_i32(
            object_payload_local,
            object_tag_local,
            key_tag_local,
            function,
        );
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_number_property_key_numeric_index(
            key_payload_local,
            numeric_index_payload_local,
            function,
        );
        self.emit_typed_array_valid_integer_index_i32(
            object_payload_local,
            numeric_index_payload_local,
            index_local,
            result_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(handled_local));
        function.instruction(&Instruction::End);
        self.release_temp_local(index_local);
        self.release_temp_local(numeric_index_payload_local);
        Ok(())
    }
}
