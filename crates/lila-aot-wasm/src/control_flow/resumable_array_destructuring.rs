use super::*;
use lila_ir::{
    IteratorRecordIr, ResumableArrayDestructuringIr, ResumableArrayDestructuringStepIr,
    ResumableIteratorCloseIr,
};

/// The three activation slots of a resumable array destructuring's Iterator
/// Record, resolved once per step.
struct ResumableDestructuringSlots {
    iterator: BindingStorage,
    next_method: BindingStorage,
    done: BindingStorage,
}

impl FunctionBuilder<'_> {
    /// `ExprIr::ResumableArrayDestructuring`: one step of a synchronous
    /// generator's ArrayAssignmentPattern whose elements suspend (13.15.5.2).
    ///
    /// Every step loads the Iterator Record from its activation slots into the
    /// same working set `compile_array_destructure_from_value_locals` uses and
    /// stores `[[Done]]` back before its completion leaves the step, so a
    /// throwing element still tells the enclosing close whether the iterator
    /// is exhausted. Every step produces undefined.
    pub(crate) fn compile_resumable_array_destructuring(
        &mut self,
        destructuring: &ResumableArrayDestructuringIr,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let slots = self.resumable_destructuring_slots(destructuring.record())?;
        let locals = DestructuringIteratorLocals {
            iterator_payload: self.reserve_temp_local(),
            iterator_tag: self.reserve_temp_local(),
            next_payload: self.reserve_temp_local(),
            next_tag: self.reserve_temp_local(),
            key: self.reserve_temp_local(),
            result_payload: self.reserve_temp_local(),
            result_tag: self.reserve_temp_local(),
            done_payload: self.reserve_temp_local(),
            done_tag: self.reserve_temp_local(),
            value_payload: self.reserve_temp_local(),
            value_tag: self.reserve_temp_local(),
            return_payload: self.reserve_temp_local(),
            return_tag: self.reserve_temp_local(),
            done: self.reserve_temp_local(),
            close_saved_payload: self.reserve_temp_local(),
            close_saved_tag: self.reserve_temp_local(),
            close_saved_completion: self.reserve_temp_local(),
            close_saved_aux: self.reserve_temp_local(),
        };
        let consumer = SyncIteratorConsumer::ArrayDestructuring;
        match destructuring.step() {
            ResumableArrayDestructuringStepIr::Open { value, protocol: _ } => {
                let method_payload = self.reserve_temp_local();
                let method_tag = self.reserve_temp_local();
                self.compile_expr_to_locals(
                    value,
                    locals.value_payload,
                    locals.value_tag,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(
                    locals.value_payload,
                    locals.value_tag,
                    function,
                )?;
                self.emit_get_iterator_from_value_locals(
                    value.value_info(),
                    locals.value_payload,
                    locals.value_tag,
                    method_payload,
                    method_tag,
                    &locals.protocol(),
                    &consumer,
                    function,
                )?;
                self.write_binding_from_locals(
                    slots.iterator,
                    locals.iterator_payload,
                    locals.iterator_tag,
                    function,
                );
                self.write_binding_from_locals(
                    slots.next_method,
                    locals.next_payload,
                    locals.next_tag,
                    function,
                );
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(locals.done));
                self.write_resumable_destructuring_done(slots.done, &locals, function);
                self.release_temp_local(method_tag);
                self.release_temp_local(method_payload);
            }
            ResumableArrayDestructuringStepIr::Elements(elements) => {
                self.read_resumable_destructuring_record(&slots, &locals, function)?;
                let exit_target = self.open_frame(ControlFrameKind::Block, function);
                let abrupt_target = self.open_frame(ControlFrameKind::Block, function);
                self.finally_stack.push(abrupt_target);
                for element in elements {
                    self.compile_array_destructuring_element(
                        element, &locals, &consumer, function,
                    )?;
                }
                self.finally_stack.pop();
                self.write_resumable_destructuring_done(slots.done, &locals, function);
                self.emit_branch_to_target(exit_target, function);
                self.pop_control(ControlFrameKind::Block);
                function.instruction(&Instruction::End);
                // An element threw: publish `[[Done]]` for the synthesized
                // catch block's close, then continue the throw.
                self.write_resumable_destructuring_done(slots.done, &locals, function);
                self.emit_propagate_current_completion_if_throw(function);
                self.pop_control(ControlFrameKind::Block);
                function.instruction(&Instruction::End);
            }
            ResumableArrayDestructuringStepIr::Close(close) => {
                self.read_resumable_destructuring_record(&slots, &locals, function)?;
                function.instruction(&Instruction::LocalGet(locals.done));
                function.instruction(&Instruction::I64Eqz);
                self.open_frame(ControlFrameKind::If, function);
                match close {
                    ResumableIteratorCloseIr::NormalOrReturn => {
                        self.emit_iterator_close(
                            locals.iterator_payload,
                            locals.iterator_tag,
                            locals.key,
                            locals.return_payload,
                            locals.return_tag,
                            locals.result_payload,
                            locals.result_tag,
                            function,
                        )?;
                    }
                    ResumableIteratorCloseIr::Throw => {
                        function.instruction(&Instruction::I64Const(1));
                        function.instruction(&Instruction::LocalSet(locals.done));
                        self.write_resumable_destructuring_done(slots.done, &locals, function);
                        self.emit_iterator_close_preserving_current_throw(
                            IteratorCloseOnThrowLocals {
                                iterator_payload_local: locals.iterator_payload,
                                iterator_tag_local: locals.iterator_tag,
                                key_local: locals.key,
                                return_payload_local: locals.return_payload,
                                return_tag_local: locals.return_tag,
                                result_payload_local: locals.result_payload,
                                result_tag_local: locals.result_tag,
                                saved_payload_local: locals.close_saved_payload,
                                saved_tag_local: locals.close_saved_tag,
                                saved_completion_local: locals.close_saved_completion,
                                saved_aux_local: locals.close_saved_aux,
                            },
                            function,
                        )?;
                    }
                }
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
            }
        }
        self.emit_undefined_payload(function);
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        for local in [
            locals.close_saved_aux,
            locals.close_saved_completion,
            locals.close_saved_tag,
            locals.close_saved_payload,
            locals.done,
            locals.return_tag,
            locals.return_payload,
            locals.value_tag,
            locals.value_payload,
            locals.done_tag,
            locals.done_payload,
            locals.result_tag,
            locals.result_payload,
            locals.key,
            locals.next_tag,
            locals.next_payload,
            locals.iterator_tag,
            locals.iterator_payload,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    fn resumable_destructuring_slots(
        &self,
        record: &IteratorRecordIr,
    ) -> Result<ResumableDestructuringSlots, EmitError> {
        let slot = |name: &str| -> Result<BindingStorage, EmitError> {
            match self.lookup_binding(name) {
                Some(storage @ BindingStorage::EnvSlot { .. }) => Ok(storage),
                Some(_) => Err(EmitError::unsupported(format!(
                    "resumable array destructuring slot `{name}` is not activation-owned"
                ))),
                None => Err(EmitError::unsupported(format!(
                    "resumable array destructuring slot `{name}` is not in scope"
                ))),
            }
        };
        Ok(ResumableDestructuringSlots {
            iterator: slot(record.iterator().as_str())?,
            next_method: slot(record.next_method().as_str())?,
            done: slot(record.done().as_str())?,
        })
    }

    fn read_resumable_destructuring_record(
        &mut self,
        slots: &ResumableDestructuringSlots,
        locals: &DestructuringIteratorLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.read_binding_to_locals(
            slots.iterator,
            locals.iterator_payload,
            locals.iterator_tag,
            function,
        )?;
        self.read_binding_to_locals(
            slots.next_method,
            locals.next_payload,
            locals.next_tag,
            function,
        )?;
        self.read_binding_to_locals(slots.done, locals.done_payload, locals.done_tag, function)?;
        function.instruction(&Instruction::LocalGet(locals.done_payload));
        function.instruction(&Instruction::LocalSet(locals.done));
        Ok(())
    }

    fn write_resumable_destructuring_done(
        &mut self,
        done_slot: BindingStorage,
        locals: &DestructuringIteratorLocals,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(locals.done));
        function.instruction(&Instruction::LocalSet(locals.done_payload));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(locals.done_tag));
        self.write_binding_from_locals(done_slot, locals.done_payload, locals.done_tag, function);
    }
}
