//! `Iterator.prototype.includes ( searchElement [ , skippedElements ] )`
//! (proposal-iterator-includes).

use super::direct_record::{ArgumentErrorKind, TaggedLocals};
use super::*;

const NON_OBJECT_MESSAGE: &str = "Iterator.prototype.includes called on non-object";
const SKIPPED_NOT_INTEGRAL_MESSAGE: &str =
    "Iterator.prototype.includes skippedElements must be an integral Number or an infinity";
const SKIPPED_OUT_OF_RANGE_MESSAGE: &str =
    "Iterator.prototype.includes skippedElements must be between 0 and 2^53 - 1";
const NEXT_RESULT_NOT_OBJECT_MESSAGE: &str =
    "Iterator.prototype.includes next result must be object";

/// Every string the `includes` emitter interns.
pub(super) fn iterator_includes_pool_strings() -> impl Iterator<Item = &'static str> {
    [
        NON_OBJECT_MESSAGE,
        SKIPPED_NOT_INTEGRAL_MESSAGE,
        SKIPPED_OUT_OF_RANGE_MESSAGE,
        NEXT_RESULT_NOT_OBJECT_MESSAGE,
    ]
    .into_iter()
}

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn emit_iterator_prototype_includes(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver = self.direct_iterator_receiver("Iterator.prototype.includes")?;
        // 1-2. If obj is not an Object, throw a TypeError exception.
        self.emit_require_object_receiver(receiver, NON_OBJECT_MESSAGE, function)?;
        // 3. iterated = { [[Iterator]]: obj, [[NextMethod]]: undefined, [[Done]]: false }.
        let close = self.reserve_iterator_close_on_throw_locals(receiver);
        let [search_payload_local, search_tag_local, skipped_payload_local, skipped_tag_local, to_skip_local, skipped_local, next_payload_local, next_tag_local] =
            self.reserve_temp_locals();
        let step = self.reserve_direct_step_locals();
        let next = TaggedLocals {
            payload: next_payload_local,
            tag: next_tag_local,
        };
        self.emit_builtin_arg_to_locals(0, search_payload_local, search_tag_local, function);
        self.emit_builtin_arg_to_locals(1, skipped_payload_local, skipped_tag_local, function);

        // 4. If skippedElements is undefined, let toSkip be +0𝔽.
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(to_skip_local));
        function.instruction(&Instruction::LocalGet(skipped_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        // 5. Else, if skippedElements is not one of +∞𝔽, -∞𝔽, or an integral
        // Number, throw a TypeError that closes `iterated`. No coercion.
        function.instruction(&Instruction::LocalGet(skipped_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_closing_direct_iterator(
            ArgumentErrorKind::Type,
            SKIPPED_NOT_INTEGRAL_MESSAGE,
            close,
            function,
        )?;
        function.instruction(&Instruction::End);
        // NaN, or finite (`x - x == 0`) and not integral (`trunc(x) != x`).
        function.instruction(&Instruction::LocalGet(skipped_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(skipped_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::LocalGet(skipped_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(skipped_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Sub);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::LocalGet(skipped_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::LocalGet(skipped_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_closing_direct_iterator(
            ArgumentErrorKind::Type,
            SKIPPED_NOT_INTEGRAL_MESSAGE,
            close,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(skipped_payload_local));
        function.instruction(&Instruction::LocalSet(to_skip_local));
        function.instruction(&Instruction::End);

        // 6. If toSkip < -0𝔽 (so -∞𝔽, but not -0𝔽), throw a RangeError.
        // 7. If toSkip is finite and toSkip > 𝔽(2**53 - 1), throw a RangeError.
        function.instruction(&Instruction::LocalGet(to_skip_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::LocalGet(to_skip_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(
            9_007_199_254_740_991.0,
        )));
        function.instruction(&Instruction::F64Gt);
        function.instruction(&Instruction::LocalGet(to_skip_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_closing_direct_iterator(
            ArgumentErrorKind::Range,
            SKIPPED_OUT_OF_RANGE_MESSAGE,
            close,
            function,
        )?;
        function.instruction(&Instruction::End);

        // 8. Let skipped be +0𝔽.
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(skipped_local));
        // 9. Set iterated to ? GetIteratorDirect(obj).
        self.emit_get_iterator_direct_next(receiver, next, close.key_local, function)?;

        // 10. Repeat,
        function.instruction(&Instruction::Loop(BlockType::Empty));
        // a-b. Let value be ? IteratorStepValue(iterated). If value is done,
        // return false.
        self.emit_direct_iterator_step_done_i32(
            receiver,
            next,
            &step,
            NEXT_RESULT_NOT_OBJECT_MESSAGE,
            function,
        )?;
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_direct_iterator_step_value(&step, function)?;
        let value = step.value();
        // c. If skipped < toSkip, set skipped to skipped + 1𝔽.
        function.instruction(&Instruction::LocalGet(skipped_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(to_skip_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(skipped_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(skipped_local));
        // Continue the Repeat: out of this `if` to the loop.
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        // d. Else if SameValueZero(value, searchElement) is true, return
        // ? IteratorClose(iterated, NormalCompletion(true)).
        self.emit_tagged_payload_same_value_zero_i32(
            value.tag,
            value.payload,
            search_tag_local,
            search_payload_local,
            function,
        )?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_iterator_close(
            receiver.payload,
            receiver.tag,
            close.key_local,
            close.return_payload_local,
            close.return_tag_local,
            close.result_payload_local,
            close.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);

        self.release_direct_step_locals(step);
        self.release_temp_locals([
            search_payload_local,
            search_tag_local,
            skipped_payload_local,
            skipped_tag_local,
            to_skip_local,
            skipped_local,
            next_payload_local,
            next_tag_local,
        ]);
        self.release_iterator_close_on_throw_locals(close);
        Ok(())
    }
}
