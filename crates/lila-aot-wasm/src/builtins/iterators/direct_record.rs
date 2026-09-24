//! Shared emission for the `Iterator.prototype` methods whose receiver is the
//! iterator itself: `GetIteratorDirect ( obj )`, the argument-validation
//! closes of the not-yet-acquired record
//! `{ [[Iterator]]: obj, [[NextMethod]]: undefined, [[Done]]: false }`, and
//! `IteratorStepValue` over the acquired record.
//!
//! Every abrupt completion produced here is routed to the *active* throw
//! target: the enclosing [`FunctionBuilder::emit_capturing_throws`] block when
//! the caller must observe the throw (an Iterator Helper completing its
//! generator state), or a return from the builtin otherwise. None of these
//! operations closes the iterator on its own: `IteratorStepValue` failures are
//! never `IfAbruptCloseIterator` sites.

use super::*;

/// One JavaScript value held in a payload/tag local pair.
#[derive(Clone, Copy)]
pub(super) struct TaggedLocals {
    pub(super) payload: u32,
    pub(super) tag: u32,
}

/// The working set of one `IteratorStepValue` over a direct iterator record.
///
/// Only the step helpers below write these locals; the value is exposed read
/// only through [`DirectStepLocals::value`].
pub(super) struct DirectStepLocals {
    key: u32,
    result: TaggedLocals,
    done: TaggedLocals,
    value: TaggedLocals,
}

impl DirectStepLocals {
    /// The `IteratorValue` of the last step, valid after
    /// [`FunctionBuilder::emit_direct_iterator_step_value`].
    pub(super) const fn value(&self) -> TaggedLocals {
        self.value
    }
}

/// The error an argument-validation step raises before the iterator record is
/// acquired.
#[derive(Clone, Copy)]
pub(super) enum ArgumentErrorKind {
    Type,
    Range,
}

impl<'a> FunctionBuilder<'a> {
    /// Reserves `N` temporaries. They must be returned through
    /// [`FunctionBuilder::release_temp_locals`], which owns the reverse-order
    /// discipline of the temp-local stack.
    pub(super) fn reserve_temp_locals<const N: usize>(&mut self) -> [u32; N] {
        std::array::from_fn(|_| self.reserve_temp_local())
    }

    pub(super) fn release_temp_locals<const N: usize>(&mut self, locals: [u32; N]) {
        for local in locals.into_iter().rev() {
            self.release_temp_local(local);
        }
    }

    /// The builtin's `this` value.
    pub(super) fn direct_iterator_receiver(&self, method: &str) -> Result<TaggedLocals, EmitError> {
        let payload = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(format!(
                "unsupported in lila wasm-aot first slice: missing {method} receiver"
            ))
        })?;
        let tag = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(format!(
                "unsupported in lila wasm-aot first slice: missing {method} receiver tag"
            ))
        })?;
        Ok(TaggedLocals { payload, tag })
    }

    /// `If obj is not an Object, throw a TypeError exception.`
    pub(super) fn emit_require_object_receiver(
        &mut self,
        receiver: TaggedLocals,
        message: &'static str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_is_heap_object_like_tag_i32(receiver.tag, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        Ok(())
    }

    /// The locals `IteratorClose ( iterated, throwCompletion )` needs, for the
    /// record whose `[[Iterator]]` is `iterator`.
    pub(super) fn reserve_iterator_close_on_throw_locals(
        &mut self,
        iterator: TaggedLocals,
    ) -> IteratorCloseOnThrowLocals {
        let [key_local, return_payload_local, return_tag_local, result_payload_local, result_tag_local, saved_payload_local, saved_tag_local, saved_completion_local, saved_aux_local] =
            self.reserve_temp_locals();
        IteratorCloseOnThrowLocals {
            iterator_payload_local: iterator.payload,
            iterator_tag_local: iterator.tag,
            key_local,
            return_payload_local,
            return_tag_local,
            result_payload_local,
            result_tag_local,
            saved_payload_local,
            saved_tag_local,
            saved_completion_local,
            saved_aux_local,
        }
    }

    pub(super) fn release_iterator_close_on_throw_locals(
        &mut self,
        close: IteratorCloseOnThrowLocals,
    ) {
        self.release_temp_locals([
            close.key_local,
            close.return_payload_local,
            close.return_tag_local,
            close.result_payload_local,
            close.result_tag_local,
            close.saved_payload_local,
            close.saved_tag_local,
            close.saved_completion_local,
            close.saved_aux_local,
        ]);
    }

    /// `Let error be ThrowCompletion(a newly created <kind> object).`
    /// `Return ? IteratorClose(iterated, error).`
    ///
    /// Emitted inside the caller's failing branch. It always returns from the
    /// builtin, and the error wins over anything `IteratorClose` throws.
    pub(super) fn emit_throw_closing_direct_iterator(
        &mut self,
        kind: ArgumentErrorKind,
        message: &'static str,
        close: IteratorCloseOnThrowLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match kind {
            ArgumentErrorKind::Type => self.emit_throw_current_function_realm_type_error(
                message,
                self.result_local,
                self.result_tag_local,
                function,
            )?,
            ArgumentErrorKind::Range => self.emit_throw_current_function_realm_range_error(
                message,
                self.result_local,
                self.result_tag_local,
                function,
            )?,
        }
        self.emit_iterator_close_preserving_current_throw(close, function)?;
        self.emit_return_current_completion(function);
        Ok(())
    }

    /// `Set iterated to ? GetIteratorDirect(obj).`: one `[[Get]]` of `next`,
    /// with no callability check (a non-callable `next` fails at the first
    /// `IteratorStep`).
    pub(super) fn emit_get_iterator_direct_next(
        &mut self,
        receiver: TaggedLocals,
        next: TaggedLocals,
        key_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::I64Const(self.strings.payload("next")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            receiver.payload,
            receiver.tag,
            receiver.payload,
            receiver.tag,
            key_local,
            next.payload,
            next.tag,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(next.payload, next.tag, function)
    }

    pub(super) fn reserve_direct_step_locals(&mut self) -> DirectStepLocals {
        let [key, result_payload, result_tag, done_payload, done_tag, value_payload, value_tag] =
            self.reserve_temp_locals();
        DirectStepLocals {
            key,
            result: TaggedLocals {
                payload: result_payload,
                tag: result_tag,
            },
            done: TaggedLocals {
                payload: done_payload,
                tag: done_tag,
            },
            value: TaggedLocals {
                payload: value_payload,
                tag: value_tag,
            },
        }
    }

    pub(super) fn release_direct_step_locals(&mut self, step: DirectStepLocals) {
        self.release_temp_locals([
            step.key,
            step.result.payload,
            step.result.tag,
            step.done.payload,
            step.done.tag,
            step.value.payload,
            step.value.tag,
        ]);
    }

    /// `IteratorStep ( iteratorRecord )` up to its completion test: calls
    /// `[[NextMethod]]` on `[[Iterator]]`, requires an Object result and pushes
    /// `ToBoolean(? Get(result, "done"))` as an `i32`.
    pub(super) fn emit_direct_iterator_step_done_i32(
        &mut self,
        iterator: TaggedLocals,
        next: TaggedLocals,
        step: &DirectStepLocals,
        result_not_object_message: &'static str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_function_or_proxy_call_leave_throw_completion(
            next.payload,
            next.tag,
            iterator.payload,
            iterator.tag,
            &[],
            step.result.payload,
            step.result.tag,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            step.result.payload,
            step.result.tag,
            function,
        )?;
        self.emit_is_heap_object_like_tag_i32(step.result.tag, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            result_not_object_message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_current_throw(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(self.strings.payload("done")));
        function.instruction(&Instruction::LocalSet(step.key));
        self.emit_object_read(
            step.result.payload,
            step.result.tag,
            step.result.payload,
            step.result.tag,
            step.key,
            step.done.payload,
            step.done.tag,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            step.done.payload,
            step.done.tag,
            function,
        )?;
        self.compile_truthy_tagged_i32(step.done.tag, step.done.payload, function)
    }

    /// `IteratorValue ( iteratorResult )` for the result the last
    /// [`FunctionBuilder::emit_direct_iterator_step_done_i32`] produced.
    pub(super) fn emit_direct_iterator_step_value(
        &mut self,
        step: &DirectStepLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::I64Const(self.strings.payload("value")));
        function.instruction(&Instruction::LocalSet(step.key));
        self.emit_object_read(
            step.result.payload,
            step.result.tag,
            step.result.payload,
            step.result.tag,
            step.key,
            step.value.payload,
            step.value.tag,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            step.value.payload,
            step.value.tag,
            function,
        )
    }

    /// Emits `body` inside one block that is the active throw target, so every
    /// throw `body` routes to the active handler lands after the block with
    /// `completion_local == THROW` and the thrown value in `result_local`.
    /// Normal exits from `body` must return from the builtin themselves.
    pub(super) fn emit_capturing_throws(
        &mut self,
        function: &mut Function,
        body: impl FnOnce(&mut Self, &mut Function) -> Result<(), EmitError>,
    ) -> Result<(), EmitError> {
        let capture_frame = self.open_frame(ControlFrameKind::Block, function);
        self.finally_stack.push(capture_frame);
        let emitted = body(self, function);
        self.finally_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        emitted
    }

    /// `CreateIteratorResultObject(undefined, true)` as the builtin's result,
    /// then returns.
    pub(super) fn emit_return_done_iterator_result(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let [value_payload_local, value_tag_local] = self.reserve_temp_locals();
        self.emit_undefined_payload(function);
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_iterator_result_object_from_locals(
            value_payload_local,
            value_tag_local,
            true,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        self.release_temp_locals([value_payload_local, value_tag_local]);
        Ok(())
    }
}
