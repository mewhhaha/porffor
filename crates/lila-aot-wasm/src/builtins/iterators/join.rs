//! `Iterator.prototype.join ( separator )` (proposal-iterator-join).

use super::direct_record::TaggedLocals;
use super::*;
use crate::operations::PrimitiveToStringAbruptRoute;

const NON_OBJECT_MESSAGE: &str = "Iterator.prototype.join called on non-object";
const NEXT_RESULT_NOT_OBJECT_MESSAGE: &str = "Iterator.prototype.join next result must be object";

/// Every string the `join` emitter interns.
pub(super) fn iterator_join_pool_strings() -> impl Iterator<Item = &'static str> {
    [NON_OBJECT_MESSAGE, NEXT_RESULT_NOT_OBJECT_MESSAGE, ",", ""].into_iter()
}

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn emit_iterator_prototype_join(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver = self.direct_iterator_receiver("Iterator.prototype.join")?;
        // 1-2. If obj is not an Object, throw a TypeError exception.
        self.emit_require_object_receiver(receiver, NON_OBJECT_MESSAGE, function)?;
        // 3. iterated = { [[Iterator]]: obj, [[NextMethod]]: undefined, [[Done]]: false }.
        let close = self.reserve_iterator_close_on_throw_locals(receiver);
        let [separator_payload_local, separator_tag_local, primitive_payload_local, primitive_tag_local, separator_string_local, joined_string_local, piece_string_local, first_local, next_payload_local, next_tag_local] =
            self.reserve_temp_locals();
        let step = self.reserve_direct_step_locals();
        let primitive = TaggedLocals {
            payload: primitive_payload_local,
            tag: primitive_tag_local,
        };
        let next = TaggedLocals {
            payload: next_payload_local,
            tag: next_tag_local,
        };

        // 4. If separator is undefined, let sep be ",".
        self.emit_builtin_arg_to_locals(0, separator_payload_local, separator_tag_local, function);
        function.instruction(&Instruction::I64Const(self.strings.payload(",")));
        function.instruction(&Instruction::LocalSet(separator_string_local));
        function.instruction(&Instruction::LocalGet(separator_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        // 5. Else, let sep be Completion(ToString(separator)).
        // IfAbruptCloseIterator(sep, iterated).
        self.emit_join_to_string_closing(
            TaggedLocals {
                payload: separator_payload_local,
                tag: separator_tag_local,
            },
            primitive,
            separator_string_local,
            close,
            function,
        )?;
        function.instruction(&Instruction::End);

        // 6. Set iterated to ? GetIteratorDirect(obj).
        self.emit_get_iterator_direct_next(receiver, next, close.key_local, function)?;
        // 7-8. Let result be the empty String. Let first be true.
        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(joined_string_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(first_local));

        // 9. Repeat,
        function.instruction(&Instruction::Loop(BlockType::Empty));
        // a-b. Let value be ? IteratorStepValue(iterated). If value is done,
        // return result.
        self.emit_direct_iterator_step_done_i32(
            receiver,
            next,
            &step,
            NEXT_RESULT_NOT_OBJECT_MESSAGE,
            function,
        )?;
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(joined_string_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_direct_iterator_step_value(&step, function)?;
        let value = step.value();
        // c-d. If first is true, set first to false; else append sep.
        function.instruction(&Instruction::LocalGet(first_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_concat_string_payloads_local(
            joined_string_local,
            separator_string_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(joined_string_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(first_local));
        // e. If value is neither undefined nor null, append
        // Completion(ToString(value)), closing `iterated` on an abrupt one.
        function.instruction(&Instruction::LocalGet(value.tag));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(value.tag));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_join_to_string_closing(value, primitive, piece_string_local, close, function)?;
        self.emit_concat_string_payloads_local(joined_string_local, piece_string_local, function)?;
        function.instruction(&Instruction::LocalSet(joined_string_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);

        self.release_direct_step_locals(step);
        self.release_temp_locals([
            separator_payload_local,
            separator_tag_local,
            primitive_payload_local,
            primitive_tag_local,
            separator_string_local,
            joined_string_local,
            piece_string_local,
            first_local,
            next_payload_local,
            next_tag_local,
        ]);
        self.release_iterator_close_on_throw_locals(close);
        Ok(())
    }

    /// `ToString(input)` into `string_local`; an abrupt completion closes
    /// `iterated` (preserving the throw) and returns it.
    fn emit_join_to_string_closing(
        &mut self,
        input: TaggedLocals,
        primitive: TaggedLocals,
        string_local: u32,
        close: IteratorCloseOnThrowLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_tagged_to_primitive_locals(
            ToPrimitiveHint::String,
            input.payload,
            input.tag,
            primitive.payload,
            primitive.tag,
            ToPrimitiveAbruptRoute::IteratorCloseAndReturn(close),
            function,
        )?;
        self.emit_primitive_to_string_payload(
            primitive.payload,
            primitive.tag,
            PrimitiveToStringAbruptRoute::IteratorCloseAndReturn(close),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(string_local));
        Ok(())
    }
}
