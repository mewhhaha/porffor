use super::*;
use crate::emit::{async_generator_for_await_is_transparent_yield, ControlTarget};
use crate::generator_delegation::AsyncGeneratorDelegationKind;
use lila_ir::{
    ArrayDestructuringEvaluationIr, AsyncForOfIteratorPlanIr, AsyncResumeModeIr, AsyncTryPlanIr,
    ObjectDestructuringPatternIr,
};

fn innermost_target(left: ControlTarget, right: ControlTarget) -> ControlTarget {
    if left.frame >= right.frame {
        left
    } else {
        right
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn innermost_target_uses_the_later_control_frame() {
        let outer = ControlTarget {
            frame: 2,
            environment_depth: 1,
            label: LabelDepth::for_test(3),
        };
        let inner = ControlTarget {
            frame: 5,
            environment_depth: 3,
            label: LabelDepth::for_test(6),
        };

        assert_eq!(innermost_target(outer, inner), inner);
        assert_eq!(innermost_target(inner, outer), inner);
    }

    #[test]
    fn environment_hops_is_the_depth_difference() {
        assert_eq!(environment_hops(0, 0), 0);
        assert_eq!(environment_hops(4, 1), 3);
    }

    #[test]
    #[should_panic(expected = "control target environment must enclose the current environment")]
    fn environment_hops_rejects_a_deeper_target() {
        environment_hops(1, 2);
    }

    #[test]
    fn finalizer_crosses_only_branches_to_outer_frames() {
        let finalizer = ControlTarget {
            frame: 4,
            environment_depth: 0,
            label: LabelDepth::for_test(5),
        };
        let outer_branch = ControlTarget {
            frame: 1,
            environment_depth: 0,
            label: LabelDepth::for_test(2),
        };
        let inner_branch = ControlTarget {
            frame: 6,
            environment_depth: 0,
            label: LabelDepth::for_test(7),
        };

        assert!(finalizer_crosses_branch(finalizer, outer_branch));
        assert!(!finalizer_crosses_branch(finalizer, finalizer));
        assert!(!finalizer_crosses_branch(finalizer, inner_branch));
    }
}

fn environment_hops(current_depth: u32, target_depth: u32) -> u32 {
    current_depth
        .checked_sub(target_depth)
        .expect("control target environment must enclose the current environment")
}

fn finalizer_crosses_branch(finalizer: ControlTarget, branch_target: ControlTarget) -> bool {
    finalizer.frame > branch_target.frame
}

fn iteration_environment_owns_binding(
    lexical_environment: Option<&ForInOfEnvironmentIr>,
    name: &str,
) -> bool {
    lexical_environment
        .and_then(|environment| environment.iteration_environment.as_ref())
        .is_some_and(|environment| {
            environment
                .bindings
                .iter()
                .any(|binding| binding.name == name)
        })
}

#[derive(Clone, Copy)]
pub(crate) struct SyncIteratorLocals {
    pub(crate) iterator_payload: u32,
    pub(crate) iterator_tag: u32,
    pub(crate) next_payload: u32,
    pub(crate) next_tag: u32,
    pub(crate) key: u32,
    pub(crate) result_payload: u32,
    pub(crate) result_tag: u32,
    pub(crate) done_payload: u32,
    pub(crate) done_tag: u32,
    pub(crate) value_payload: u32,
    pub(crate) value_tag: u32,
}

#[derive(Clone, Copy)]
struct DestructuringIteratorLocals {
    iterator_payload: u32,
    iterator_tag: u32,
    next_payload: u32,
    next_tag: u32,
    key: u32,
    result_payload: u32,
    result_tag: u32,
    done_payload: u32,
    done_tag: u32,
    value_payload: u32,
    value_tag: u32,
    return_payload: u32,
    return_tag: u32,
    done: u32,
    close_saved_payload: u32,
    close_saved_tag: u32,
    close_saved_completion: u32,
    close_saved_aux: u32,
}

/// The activation layout shared by the two execution kinds that can own a
/// `for-await-of` suspension.
///
/// Ordinary async functions strictly decode their two-way resume completion.
/// Async generators retain their separate five-way resume-kind domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ForAwaitActivationLayout {
    AsyncFunction,
    AsyncGenerator,
}

impl ForAwaitActivationLayout {
    const fn resume_state_offset(self) -> u64 {
        match self {
            Self::AsyncFunction => HEAP_ASYNC_RESUME_STATE_OFFSET,
            Self::AsyncGenerator => HEAP_ASYNC_GENERATOR_RESUME_STATE_OFFSET,
        }
    }

    const fn resume_payload_offset(self) -> u64 {
        match self {
            Self::AsyncFunction => HEAP_ASYNC_RESUME_PAYLOAD_OFFSET,
            Self::AsyncGenerator => HEAP_ASYNC_GENERATOR_RESUME_PAYLOAD_OFFSET,
        }
    }

    const fn resume_tag_offset(self) -> u64 {
        match self {
            Self::AsyncFunction => HEAP_ASYNC_RESUME_TAG_OFFSET,
            Self::AsyncGenerator => HEAP_ASYNC_GENERATOR_RESUME_TAG_OFFSET,
        }
    }

    const fn is_async_generator(self) -> bool {
        match self {
            Self::AsyncFunction => false,
            Self::AsyncGenerator => true,
        }
    }
}

impl DestructuringIteratorLocals {
    fn protocol(self) -> SyncIteratorLocals {
        SyncIteratorLocals {
            iterator_payload: self.iterator_payload,
            iterator_tag: self.iterator_tag,
            next_payload: self.next_payload,
            next_tag: self.next_tag,
            key: self.key,
            result_payload: self.result_payload,
            result_tag: self.result_tag,
            done_payload: self.done_payload,
            done_tag: self.done_tag,
            value_payload: self.value_payload,
            value_tag: self.value_tag,
        }
    }
}

#[derive(Clone, Copy)]
enum DestructuringIteratorStepKind {
    Elision,
    Value,
}

enum PreparedDestructuringTarget {
    Direct,
    Property {
        target: TypedExpr,
        target_payload: u32,
        target_tag: u32,
        key: DestructuringPropertyKeyIr,
        key_payload: Option<u32>,
        key_tag: Option<u32>,
        /// Carried through from
        /// [`lila_ir::DestructuringTargetIr::AssignmentProperty`] so the
        /// write-back can install it. `put_destructuring_target` matches the
        /// target with `..` and reads the prepared value instead, so the
        /// strictness the write uses is the one `prepare_destructuring_target`
        /// saw on the *same* element.
        strictness: Strictness,
    },
    Private {
        target_payload: u32,
        target_tag: u32,
        private_name_id: PrivateNameId,
    },
}

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn emit_statement_result(&self, function: &mut Function, kind: ValueKind) {
        self.emit_undefined_payload(function);
        self.finish_statement_payload(function, kind);
    }

    pub(crate) fn finish_statement_payload(&self, function: &mut Function, kind: ValueKind) {
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(kind.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);
    }

    pub(crate) fn emit_undefined_payload(&self, function: &mut Function) {
        function.instruction(&Instruction::I64Const(0));
    }

    pub(crate) fn save_current_completion(
        &self,
        payload_local: u32,
        tag_local: u32,
        completion_local: u32,
        aux_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::LocalSet(completion_local));
        function.instruction(&Instruction::LocalGet(self.completion_aux_local));
        function.instruction(&Instruction::LocalSet(aux_local));
    }

    pub(crate) fn restore_saved_completion(
        &self,
        payload_local: u32,
        tag_local: u32,
        completion_local: u32,
        aux_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::LocalGet(completion_local));
        function.instruction(&Instruction::LocalSet(self.completion_local));
        function.instruction(&Instruction::LocalGet(aux_local));
        function.instruction(&Instruction::LocalSet(self.completion_aux_local));
    }

    fn emit_push_generator_pending_completion(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_push_pending_completion(
            HEAP_GENERATOR_PENDING_COMPLETION_HEAD_OFFSET,
            HEAP_GENERATOR_PENDING_COMPLETION_DEPTH_OFFSET,
            function,
        )
    }

    fn emit_push_async_pending_completion(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let (head_offset, depth_offset) = self
            .async_pending_completion_offsets()
            .expect("async finalization requires an async function activation");
        self.emit_push_pending_completion(head_offset, depth_offset, function)
    }

    fn emit_push_pending_completion(
        &mut self,
        head_offset: u64,
        depth_offset: u64,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let activation_local = self.new_target_payload_local().ok_or_else(|| {
            EmitError::unsupported("resumable finalization requires the function call ABI")
        })?;
        let previous_head_local = self.reserve_temp_local();
        let record_local = self.reserve_temp_local();
        let depth_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            activation_local,
            head_offset,
            previous_head_local,
            function,
        );
        self.emit_heap_alloc_const(HEAP_PENDING_COMPLETION_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(record_local));
        self.store_i64_local_at_offset(
            record_local,
            HEAP_PENDING_COMPLETION_NEXT_OFFSET,
            previous_head_local,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_PENDING_COMPLETION_PAYLOAD_OFFSET,
            self.result_local,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_PENDING_COMPLETION_TAG_OFFSET,
            self.result_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_PENDING_COMPLETION_KIND_OFFSET,
            self.completion_local,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_PENDING_COMPLETION_AUX_OFFSET,
            self.completion_aux_local,
            function,
        );
        self.store_i64_local_at_offset(activation_local, head_offset, record_local, function);
        self.load_i64_to_local_from_offset(activation_local, depth_offset, depth_local, function);
        function.instruction(&Instruction::LocalGet(depth_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(depth_local));
        self.store_i64_local_at_offset(activation_local, depth_offset, depth_local, function);

        self.release_temp_local(depth_local);
        self.release_temp_local(record_local);
        self.release_temp_local(previous_head_local);
        Ok(())
    }

    fn emit_pop_and_restore_generator_pending_completion(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_pop_and_restore_pending_completion(
            HEAP_GENERATOR_PENDING_COMPLETION_HEAD_OFFSET,
            HEAP_GENERATOR_PENDING_COMPLETION_DEPTH_OFFSET,
            function,
        )
    }

    fn emit_pop_and_restore_async_pending_completion(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let (head_offset, depth_offset) = self
            .async_pending_completion_offsets()
            .expect("async finalization requires an async function activation");
        self.emit_pop_and_restore_pending_completion(head_offset, depth_offset, function)
    }

    fn emit_pop_and_restore_pending_completion(
        &mut self,
        head_offset: u64,
        depth_offset: u64,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let activation_local = self.new_target_payload_local().ok_or_else(|| {
            EmitError::unsupported("resumable finalization requires the function call ABI")
        })?;
        let record_local = self.reserve_temp_local();
        let next_local = self.reserve_temp_local();
        let depth_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(activation_local, head_offset, record_local, function);
        function.instruction(&Instruction::LocalGet(record_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_PENDING_COMPLETION_NEXT_OFFSET,
            next_local,
            function,
        );
        self.store_i64_local_at_offset(activation_local, head_offset, next_local, function);
        self.load_i64_to_local_from_offset(activation_local, depth_offset, depth_local, function);
        function.instruction(&Instruction::LocalGet(depth_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(depth_local));
        self.store_i64_local_at_offset(activation_local, depth_offset, depth_local, function);
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_PENDING_COMPLETION_PAYLOAD_OFFSET,
            self.result_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_PENDING_COMPLETION_TAG_OFFSET,
            self.result_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_PENDING_COMPLETION_KIND_OFFSET,
            self.completion_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_PENDING_COMPLETION_AUX_OFFSET,
            self.completion_aux_local,
            function,
        );

        self.release_temp_local(depth_local);
        self.release_temp_local(next_local);
        self.release_temp_local(record_local);
        Ok(())
    }

    fn emit_discard_generator_pending_completion(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_discard_pending_completion(
            HEAP_GENERATOR_PENDING_COMPLETION_HEAD_OFFSET,
            HEAP_GENERATOR_PENDING_COMPLETION_DEPTH_OFFSET,
            function,
        )
    }

    fn emit_discard_async_pending_completion(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let (head_offset, depth_offset) = self
            .async_pending_completion_offsets()
            .expect("async finalization requires an async function activation");
        self.emit_discard_pending_completion(head_offset, depth_offset, function)
    }

    fn async_pending_completion_offsets(&self) -> Option<(u64, u64)> {
        match self.current_function_meta()?.protocol.execution_kind() {
            FunctionExecutionKind::Async => Some((
                HEAP_ASYNC_PENDING_COMPLETION_HEAD_OFFSET,
                HEAP_ASYNC_PENDING_COMPLETION_DEPTH_OFFSET,
            )),
            FunctionExecutionKind::AsyncGenerator => Some((
                HEAP_ASYNC_GENERATOR_PENDING_COMPLETION_HEAD_OFFSET,
                HEAP_ASYNC_GENERATOR_PENDING_COMPLETION_DEPTH_OFFSET,
            )),
            FunctionExecutionKind::Ordinary | FunctionExecutionKind::Generator => None,
        }
    }

    fn emit_discard_pending_completion(
        &mut self,
        head_offset: u64,
        depth_offset: u64,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let saved_payload_local = self.reserve_temp_local();
        let saved_tag_local = self.reserve_temp_local();
        let saved_completion_local = self.reserve_temp_local();
        let saved_aux_local = self.reserve_temp_local();
        self.save_current_completion(
            saved_payload_local,
            saved_tag_local,
            saved_completion_local,
            saved_aux_local,
            function,
        );
        self.emit_pop_and_restore_pending_completion(head_offset, depth_offset, function)?;
        self.restore_saved_completion(
            saved_payload_local,
            saved_tag_local,
            saved_completion_local,
            saved_aux_local,
            function,
        );
        self.release_temp_local(saved_aux_local);
        self.release_temp_local(saved_completion_local);
        self.release_temp_local(saved_tag_local);
        self.release_temp_local(saved_payload_local);
        Ok(())
    }

    pub(crate) fn set_completion_kind(&self, kind: CompletionKind, function: &mut Function) {
        function.instruction(&Instruction::I64Const(kind.code()));
        function.instruction(&Instruction::LocalSet(self.completion_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.completion_aux_local));
    }

    pub(crate) fn set_completion_kind_with_aux(
        &self,
        kind: CompletionKind,
        aux: i64,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(kind.code()));
        function.instruction(&Instruction::LocalSet(self.completion_local));
        function.instruction(&Instruction::I64Const(aux));
        function.instruction(&Instruction::LocalSet(self.completion_aux_local));
    }

    pub(crate) fn emit_return_current_completion(&self, function: &mut Function) {
        if let Some(target) = self.completion_exit.main_job_checkpoint_target() {
            self.emit_branch_to_target(target, function);
            return;
        }
        for _ in 0..self.environment_depth {
            self.load_i64_to_local_from_offset(
                self.current_env_local,
                ENV_PARENT_OFFSET,
                self.current_env_local,
                function,
            );
        }
        self.verify_and_clear_runtime_gc_anchor_root(function);
        match self.return_abi() {
            ReturnAbi::MainExport => {
                function.instruction(&Instruction::LocalGet(self.result_tag_local));
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::GlobalSet(RESULT_TAG_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalGet(self.completion_local));
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::GlobalSet(COMPLETION_KIND_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalGet(self.completion_aux_local));
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::GlobalSet(COMPLETION_AUX_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalGet(self.result_local));
                function.instruction(&Instruction::Return);
            }
            ReturnAbi::MultiValue => {
                function.instruction(&Instruction::LocalGet(self.result_local));
                function.instruction(&Instruction::LocalGet(self.result_tag_local));
                function.instruction(&Instruction::LocalGet(self.completion_local));
                function.instruction(&Instruction::LocalGet(self.completion_aux_local));
                function.instruction(&Instruction::Return);
            }
        }
    }

    pub(crate) fn emit_return_current_completion_if_throw(&mut self, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
    }

    pub(crate) fn emit_propagate_current_completion_if_throw(&mut self, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_propagate_current_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
    }

    pub(crate) fn emit_propagate_current_throw(&self, function: &mut Function) {
        if let Some(target) = self.active_throw_target() {
            self.emit_branch_to_target(target, function);
        } else {
            self.emit_return_current_completion(function);
        }
    }

    /// Breaks `depth` labels out of the *caller's own* region when the current
    /// completion is a throw.
    ///
    /// This is deliberately not a [`ControlTarget`] branch and deliberately
    /// still takes a raw immediate: the caller opened those frames itself and
    /// closes them itself, and the immediate is relative to the position this
    /// call stands at, which the label-depth work does not move. See the
    /// "what it deliberately does not do" section of `code_sink.rs`.
    pub(crate) fn emit_break_current_completion_if_throw(
        &self,
        depth: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Br(depth));
        function.instruction(&Instruction::End);
    }

    pub(crate) fn emit_throw_from_locals(
        &mut self,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::LocalSet(self.completion_local));
        Ok(())
    }

    /// CLOSED DEFECT — the note this replaces described a live one.
    ///
    /// The `Br` below used to be `depth_to(target) + 1 + extra_depth`, where
    /// `depth_to` counted only frames pushed through `push_control`, the `+ 1`
    /// covered this function's own raw `If`, and every *other* raw
    /// `Instruction::If`/`Block`/`Loop` between the innermost tracked frame and
    /// this call had to be declared by the caller through an `extra_depth`
    /// argument. Several callers declared nothing, and two arms of one
    /// `if`/`else` chain in `builtins/array.rs` declared different numbers for
    /// the same frame count.
    ///
    /// What that cost, on the binary before this change:
    ///
    /// * a property read that throws inside a `for` landed on the loop back
    ///   edge instead of the handler — the loop spun ~560,000 times and
    ///   trapped;
    /// * the same read inside a `switch` had its throw discarded silently.
    ///
    /// A flat `try { ... } catch` was `depth_to == 0` and worked, which is why
    /// probes kept passing. The named path (`a.zzz`) and the computed path
    /// (`a[k]`) failed identically, so it was upstream of any one emitter.
    ///
    /// Two things cannot detect a wrong depth, and both were offered as
    /// evidence at the time: rung G, because a `Br` immediate is the same width
    /// either way, and wasm validation, because both the right and the wrong
    /// label index are in range. Only running the program can tell — which is
    /// why `crates/lila-cli/tests/cli/throw_propagation.rs` runs it.
    ///
    /// The repair was structural rather than arithmetical: `ControlTarget` now
    /// carries the real Wasm label its frame opened, taken from the sink that
    /// every instruction in this crate is written through, so an undeclared
    /// frame is not a thing a caller can have. There is no `extra_depth`
    /// parameter to get wrong, in this function or anywhere else. See
    /// `code_sink.rs`.
    pub(crate) fn emit_propagate_throw_from_locals_if_needed(
        &mut self,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_from_locals(payload_local, tag_local, function)?;
        if let Some(target) = self.active_throw_target() {
            self.emit_branch_to_target(target, function);
        } else {
            self.emit_return_current_completion(function);
        }
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_resume_after_finally(
        &mut self,
        saved_payload_local: u32,
        saved_tag_local: u32,
        saved_completion_local: u32,
        saved_aux_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_NORMAL));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.restore_saved_completion(
            saved_payload_local,
            saved_tag_local,
            saved_completion_local,
            saved_aux_local,
            function,
        );
        self.emit_dispatch_current_completion(function)?;
        function.instruction(&Instruction::Else);
        self.emit_dispatch_current_completion(function)?;
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_dispatch_branch_completion(
        &self,
        targets: &[(u32, ControlTarget)],
        function: &mut Function,
    ) {
        for (target_id, branch_target) in targets {
            function.instruction(&Instruction::LocalGet(self.completion_aux_local));
            function.instruction(&Instruction::I64Const(*target_id as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            let target = self
                .active_finally_target_for_branch(*branch_target)
                .unwrap_or(*branch_target);
            self.emit_branch_to_target(target, function);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::Unreachable);
    }

    /// Every frame a pending Break completion can name once a finalizer has
    /// swallowed the original `br`.
    ///
    /// `breakable_stack` only holds the iteration and switch statements that
    /// an unlabelled `break` can reach. A labelled Block is also a break
    /// target (ECMA-262 14.13: `BreakStatement : break LabelIdentifier ;` names
    /// any enclosing LabelledStatement), and it registers itself on
    /// `label_stack` alone — so a `break label` out of a `try`/`finally` inside
    /// one would otherwise resume into no target at all and fall through to
    /// the dispatcher's trap.
    pub(crate) fn active_break_targets(&self) -> Vec<(u32, ControlTarget)> {
        let mut targets: Vec<(u32, ControlTarget)> = Vec::new();
        let frames = self
            .breakable_stack
            .iter()
            .rev()
            .copied()
            .chain(self.label_stack.iter().rev().map(|label| label.break_frame));
        for target in frames {
            let target_id = target.frame as u32;
            if !targets.iter().any(|(id, _)| *id == target_id) {
                targets.push((target_id, target));
            }
        }
        targets
    }

    pub(crate) fn active_continue_targets(&self) -> Vec<(u32, ControlTarget)> {
        let mut targets = Vec::new();
        for target in self.loop_stack.iter().rev() {
            let target_id = target.continue_frame.frame as u32;
            if !targets.iter().any(|(id, _)| *id == target_id) {
                targets.push((target_id, target.continue_frame));
            }
        }
        targets
    }

    pub(crate) fn active_throw_target(&self) -> Option<ControlTarget> {
        match (self.throw_handler_stack.last(), self.finally_stack.last()) {
            (Some(handler), Some(finalizer)) => Some(innermost_target(*handler, *finalizer)),
            (Some(handler), None) => Some(*handler),
            (None, Some(finalizer)) => Some(*finalizer),
            (None, None) => None,
        }
    }

    fn active_finally_target_for_branch(
        &self,
        branch_target: ControlTarget,
    ) -> Option<ControlTarget> {
        self.finally_stack
            .last()
            .copied()
            .filter(|finalizer| finalizer_crosses_branch(*finalizer, branch_target))
    }

    /// Routes the pending completion — throw, return, break or continue — to
    /// whichever frame owns it.
    ///
    /// The four `1`/`2`/`4`/`5` compensations this used to add on top of
    /// `depth_to` were exactly the count of `If` frames opened by the
    /// `if`/`else` chain below (and, for the two dispatch arms, the extra `If`
    /// that `emit_dispatch_branch_completion` opens per target). They were
    /// right, and they are now double counts: the sink sees those `If`s. There
    /// is no `extra_depth` to forward, so a caller standing inside frames of
    /// its own no longer has to declare them.
    pub(crate) fn emit_dispatch_current_completion(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        if let Some(target) = self.active_throw_target() {
            self.emit_branch_to_target(target, function);
        } else {
            self.emit_return_current_completion(function);
        }
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_RETURN));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        if let Some(target) = self.finally_stack.last().copied() {
            self.emit_branch_to_target(target, function);
        } else {
            self.normalize_derived_constructor_result(function)?;
            self.set_completion_kind(CompletionKind::Normal, function);
            self.emit_return_current_completion(function);
        }
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_BREAK));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        let targets = self.active_break_targets();
        self.emit_dispatch_branch_completion(&targets, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_CONTINUE));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        let targets = self.active_continue_targets();
        self.emit_dispatch_branch_completion(&targets, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn emit_dispatch_async_generator_completion(&self, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        if let Some(target) = self.active_throw_target() {
            self.emit_branch_to_target(target, function);
        } else {
            self.emit_return_current_completion(function);
        }
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_RETURN));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        if let Some(target) = self.finally_stack.last().copied() {
            self.emit_branch_to_target(target, function);
        } else {
            self.emit_return_current_completion(function);
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    fn emit_dispatch_async_completion(&mut self, function: &mut Function) -> Result<(), EmitError> {
        if self.current_function_meta().is_some_and(|meta| {
            meta.protocol.execution_kind() == FunctionExecutionKind::AsyncGenerator
        }) {
            self.emit_dispatch_async_generator_completion(function);
            return Ok(());
        }
        self.emit_dispatch_current_completion(function)
    }

    /// Opens a Wasm control frame *and* records it, in one call.
    ///
    /// The frame instruction is written here rather than by the caller, so the
    /// "emit the frame, then push the entry" ordering that ~190 call sites used
    /// to follow by hand cannot be got backwards — and the label recorded in
    /// the returned [`ControlTarget`] is the sink's depth *after* the frame is
    /// open, which is the label a branch to this frame must name.
    pub(crate) fn open_frame(
        &mut self,
        kind: ControlFrameKind,
        function: &mut Function,
    ) -> ControlTarget {
        function.instruction(&match kind {
            ControlFrameKind::If => Instruction::If(BlockType::Empty),
            ControlFrameKind::Block => Instruction::Block(BlockType::Empty),
            ControlFrameKind::Loop => Instruction::Loop(BlockType::Empty),
        });
        let target = ControlTarget {
            frame: self.control_stack.len(),
            environment_depth: self.environment_depth,
            label: function.label_depth(),
        };
        self.control_stack.push(kind);
        target
    }

    /// Pops the tracked entry `open_frame` pushed.
    ///
    /// The matching `End` is still written by the caller and is still counted
    /// by the sink, so this does not need the body: the tracked stack no longer
    /// carries any branch arithmetic, only the frame *identity* used to order
    /// targets and to name one in the completion dispatcher.
    pub(crate) fn pop_control(&mut self, expected: ControlFrameKind) {
        let actual = self
            .control_stack
            .pop()
            .expect("control stack must not underflow");
        assert!(matches!(
            (actual, expected),
            (ControlFrameKind::If, ControlFrameKind::If)
                | (ControlFrameKind::Block, ControlFrameKind::Block)
                | (ControlFrameKind::Loop, ControlFrameKind::Loop)
        ));
    }

    fn emit_unwind_environments_to_target(&self, target: ControlTarget, function: &mut Function) {
        for _ in 0..environment_hops(self.environment_depth, target.environment_depth) {
            self.load_i64_to_local_from_offset(
                self.current_env_local,
                ENV_PARENT_OFFSET,
                self.current_env_local,
                function,
            );
        }
    }

    pub(crate) fn emit_branch_to_target(&self, target: ControlTarget, function: &mut Function) {
        self.emit_unwind_environments_to_target(target, function);
        function.branch_to_label(target.label);
    }

    pub(crate) fn emit_branch_if_to_target(&self, target: ControlTarget, function: &mut Function) {
        if target.environment_depth == self.environment_depth {
            function.branch_if_to_label(target.label);
            return;
        }

        // The `If` opened here used to need an `extra_depth + 1` on the branch
        // inside it. The sink counts it, so the compensation is gone.
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_branch_to_target(target, function);
        function.instruction(&Instruction::End);
    }

    pub(crate) fn push_labels(
        &mut self,
        labels: &[String],
        break_frame: ControlTarget,
        continue_frame: Option<ControlTarget>,
    ) {
        for label in labels {
            self.label_stack.push(LabelTargets {
                name: label.clone(),
                break_frame,
                continue_frame,
            });
        }
    }

    pub(crate) fn pop_labels(&mut self, count: usize) {
        for _ in 0..count {
            self.label_stack.pop();
        }
    }

    pub(crate) fn compile_block_contents(
        &mut self,
        block: &BlockIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let async_resume_state_offset = self.async_await_resume_state_offset();
        let linear_async_entry_state = async_resume_state_offset.and_then(|_| {
            block
                .statements
                .iter()
                .find_map(Self::async_statement_entry_state)
        });
        if let (Some(entry_state), Some(resume_state_offset)) =
            (linear_async_entry_state, async_resume_state_offset)
        {
            return self.compile_async_block_contents(
                block,
                entry_state,
                !std::ptr::eq(block, self.body),
                resume_state_offset,
                function,
            );
        }
        let linear_generator_entry_state = self
            .current_function_meta()
            .is_some_and(|meta| meta.protocol.execution_kind() == FunctionExecutionKind::Generator)
            .then(|| {
                block
                    .statements
                    .iter()
                    .find_map(Self::generator_statement_entry_state)
            })
            .flatten();
        if let Some(entry_state) = linear_generator_entry_state {
            return self.compile_generator_block_contents(
                block,
                entry_state,
                !std::ptr::eq(block, self.body),
                function,
            );
        }

        if let Some(environment) = &block.lexical_environment {
            self.emit_enter_lexical_environment(environment, function)?;
        }
        let resumable_root_body = std::ptr::eq(block, self.body)
            && self.current_function_meta().is_some_and(|meta| {
                matches!(
                    meta.protocol.execution_kind(),
                    FunctionExecutionKind::Generator
                        | FunctionExecutionKind::Async
                        | FunctionExecutionKind::AsyncGenerator
                )
            });
        if !resumable_root_body {
            self.initialize_direct_lexical_bindings(&block.statements, function);
        }
        if block.statements.is_empty() {
            self.emit_statement_result(function, ValueKind::Undefined);
            if block.lexical_environment.is_some() {
                self.emit_leave_lexical_environment(function);
            }
            return Ok(());
        }

        for statement in &block.statements {
            self.compile_statement(statement, function)?;
        }

        if block.lexical_environment.is_some() {
            self.emit_leave_lexical_environment(function);
        }

        Ok(())
    }

    fn compile_async_block_contents(
        &mut self,
        block: &BlockIr,
        entry_state: u32,
        initialize_bindings: bool,
        resume_state_offset: u64,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if let Some(environment) = &block.lexical_environment {
            self.emit_enter_lexical_environment(environment, function)?;
        }
        let activation_local = self
            .new_target_payload_local()
            .expect("async body must use the function call ABI");
        if initialize_bindings {
            self.load_i64_to_local_from_offset(
                activation_local,
                resume_state_offset,
                self.scratch_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(self.scratch_local));
            function.instruction(&Instruction::I64Const(entry_state as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.initialize_direct_lexical_bindings(&block.statements, function);
            function.instruction(&Instruction::End);
        }
        self.compile_async_statement_sequence(
            &block.statements,
            entry_state,
            resume_state_offset,
            function,
        )?;
        if block.lexical_environment.is_some() {
            self.emit_leave_lexical_environment(function);
        }
        Ok(())
    }

    fn compile_async_statement_sequence(
        &mut self,
        statements: &[StatementIr],
        entry_state: u32,
        resume_state_offset: u64,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let activation_local = self
            .new_target_payload_local()
            .expect("async body must use the function call ABI");
        let mut segment_state = entry_state;
        for statement in statements {
            if let Some(exit_state) = Self::async_statement_exit_state(statement) {
                self.compile_statement(statement, function)?;
                segment_state = exit_state;
                continue;
            }

            self.load_i64_to_local_from_offset(
                activation_local,
                resume_state_offset,
                self.scratch_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(self.scratch_local));
            function.instruction(&Instruction::I64Const(segment_state as i64));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            self.compile_statement(statement, function)?;
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        Ok(())
    }

    fn async_await_resume_state_offset(&self) -> Option<u64> {
        match self.current_function_meta()?.protocol.execution_kind() {
            FunctionExecutionKind::Async => Some(HEAP_ASYNC_RESUME_STATE_OFFSET),
            FunctionExecutionKind::AsyncGenerator => Some(HEAP_ASYNC_GENERATOR_RESUME_STATE_OFFSET),
            FunctionExecutionKind::Ordinary | FunctionExecutionKind::Generator => None,
        }
    }

    fn async_statement_entry_state(statement: &StatementIr) -> Option<u32> {
        match statement {
            StatementIr::AsyncAwait { suspend_state, .. } => Some(*suspend_state),
            StatementIr::GeneratorYield { suspend_state, .. } => Some(*suspend_state),
            StatementIr::GeneratorLoop { entry_state, .. }
            | StatementIr::GeneratorIf { entry_state, .. } => Some(*entry_state),
            StatementIr::LexicalBlock(statements) => statements
                .iter()
                .find_map(Self::async_statement_entry_state),
            StatementIr::Block(block) => block
                .statements
                .iter()
                .find_map(Self::async_statement_entry_state),
            StatementIr::TryCatch {
                async_plan: Some(plan),
                ..
            }
            | StatementIr::TryFinally {
                async_plan: Some(plan),
                ..
            }
            | StatementIr::TryCatchFinally {
                async_plan: Some(plan),
                ..
            } => Some(plan.entry_state),
            StatementIr::ForOfIterator {
                async_plan: Some(plan),
                ..
            } => Some(plan.entry_state),
            _ => None,
        }
    }

    fn async_statement_exit_state(statement: &StatementIr) -> Option<u32> {
        match statement {
            StatementIr::AsyncAwait { resume_state, .. } => Some(*resume_state),
            StatementIr::GeneratorYield { resume_state, .. } => Some(*resume_state),
            StatementIr::GeneratorLoop { exit_state, .. }
            | StatementIr::GeneratorIf { exit_state, .. } => Some(*exit_state),
            StatementIr::LexicalBlock(statements) => statements
                .iter()
                .rev()
                .find_map(Self::async_statement_exit_state),
            StatementIr::Block(block) => block
                .statements
                .iter()
                .rev()
                .find_map(Self::async_statement_exit_state),
            StatementIr::TryCatch {
                async_plan: Some(plan),
                ..
            }
            | StatementIr::TryFinally {
                async_plan: Some(plan),
                ..
            }
            | StatementIr::TryCatchFinally {
                async_plan: Some(plan),
                ..
            } => Some(plan.exit_state),
            StatementIr::ForOfIterator {
                async_plan: Some(plan),
                ..
            } => Some(plan.exit_state),
            _ => None,
        }
    }

    fn generator_statement_entry_state(statement: &StatementIr) -> Option<u32> {
        match statement {
            StatementIr::GeneratorYield { suspend_state, .. } => Some(*suspend_state),
            StatementIr::LexicalBlock(statements) => statements
                .iter()
                .find_map(Self::generator_statement_entry_state),
            StatementIr::Block(block) => block
                .statements
                .iter()
                .find_map(Self::generator_statement_entry_state),
            StatementIr::GeneratorLoop { entry_state, .. }
            | StatementIr::GeneratorIf { entry_state, .. } => Some(*entry_state),
            StatementIr::TryCatch {
                generator_plan: Some(plan),
                ..
            }
            | StatementIr::TryFinally {
                generator_plan: Some(plan),
                ..
            }
            | StatementIr::TryCatchFinally {
                generator_plan: Some(plan),
                ..
            } => Some(plan.entry_state),
            _ => None,
        }
    }

    fn generator_statement_exit_state(statement: &StatementIr) -> Option<u32> {
        match statement {
            StatementIr::GeneratorYield { resume_state, .. } => Some(*resume_state),
            StatementIr::LexicalBlock(statements) => statements
                .iter()
                .rev()
                .find_map(Self::generator_statement_exit_state),
            StatementIr::Block(block) => block
                .statements
                .iter()
                .rev()
                .find_map(Self::generator_statement_exit_state),
            StatementIr::GeneratorLoop { exit_state, .. }
            | StatementIr::GeneratorIf { exit_state, .. } => Some(*exit_state),
            StatementIr::TryCatch {
                generator_plan: Some(plan),
                ..
            }
            | StatementIr::TryFinally {
                generator_plan: Some(plan),
                ..
            }
            | StatementIr::TryCatchFinally {
                generator_plan: Some(plan),
                ..
            } => Some(plan.exit_state),
            _ => None,
        }
    }

    fn compile_generator_block_contents(
        &mut self,
        block: &BlockIr,
        entry_state: u32,
        initialize_bindings: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if let Some(environment) = &block.lexical_environment {
            self.emit_enter_lexical_environment(environment, function)?;
        }
        if initialize_bindings {
            let activation_local = self
                .new_target_payload_local()
                .expect("generator body must use the function call ABI");
            self.load_i64_to_local_from_offset(
                activation_local,
                HEAP_GENERATOR_RESUME_STATE_OFFSET,
                self.scratch_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(self.scratch_local));
            function.instruction(&Instruction::I64Const(entry_state as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.initialize_direct_lexical_bindings(&block.statements, function);
            function.instruction(&Instruction::End);
        }
        if block.statements.is_empty() {
            self.emit_statement_result(function, ValueKind::Undefined);
            if block.lexical_environment.is_some() {
                self.emit_leave_lexical_environment(function);
            }
            return Ok(());
        }

        self.compile_generator_statement_sequence(&block.statements, entry_state, function)?;

        if block.lexical_environment.is_some() {
            self.emit_leave_lexical_environment(function);
        }
        Ok(())
    }

    fn compile_generator_statement_sequence(
        &mut self,
        statements: &[StatementIr],
        entry_state: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let activation_local = self
            .new_target_payload_local()
            .expect("generator body must use the function call ABI");
        let mut segment_state = entry_state;
        for statement in statements {
            if let Some(exit_state) = Self::generator_statement_exit_state(statement) {
                self.compile_statement(statement, function)?;
                segment_state = exit_state;
                continue;
            }

            self.load_i64_to_local_from_offset(
                activation_local,
                HEAP_GENERATOR_RESUME_STATE_OFFSET,
                self.scratch_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(self.scratch_local));
            function.instruction(&Instruction::I64Const(segment_state as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.compile_statement(statement, function)?;
            function.instruction(&Instruction::End);
        }
        Ok(())
    }

    pub(crate) fn initialize_direct_lexical_bindings(
        &mut self,
        statements: &[StatementIr],
        function: &mut Function,
    ) {
        for statement in statements {
            match statement {
                StatementIr::Lexical { mode, name, init } => {
                    let storage = self
                        .lookup_current_scope_binding(name)
                        .or_else(|| self.lookup_binding(name))
                        .unwrap_or_else(|| self.allocate_binding(name.clone(), *mode, init.kind));
                    self.initialize_binding_uninitialized(storage, function);
                }
                StatementIr::LexicalBlock(statements) => {
                    self.initialize_direct_lexical_bindings(statements, function);
                }
                StatementIr::Expression(TypedExpr {
                    expr:
                        ExprIr::ArrayDestructure {
                            pattern,
                            evaluation,
                            ..
                        },
                    ..
                }) => match *evaluation {
                    ArrayDestructuringEvaluationIr::BindingInitialization => {
                        pattern.visit_bindings(&mut |mode, name| {
                            if mode == BindingMode::Var {
                                return;
                            }
                            let storage = self
                                .lookup_current_scope_binding(name)
                                .or_else(|| self.lookup_binding(name))
                                .unwrap_or_else(|| {
                                    self.allocate_binding(
                                        name.to_string(),
                                        mode,
                                        ValueKind::Dynamic,
                                    )
                                });
                            self.initialize_binding_uninitialized(storage, function);
                        });
                    }
                    ArrayDestructuringEvaluationIr::AssignmentEvaluation => {}
                },
                StatementIr::Expression(TypedExpr {
                    expr: ExprIr::ObjectDestructure { pattern, .. },
                    ..
                }) => {
                    pattern.visit_bindings(&mut |mode, name| {
                        if mode == BindingMode::Var {
                            return;
                        }
                        let storage = self
                            .lookup_current_scope_binding(name)
                            .or_else(|| self.lookup_binding(name))
                            .unwrap_or_else(|| {
                                self.allocate_binding(name.to_string(), mode, ValueKind::Dynamic)
                            });
                        self.initialize_binding_uninitialized(storage, function);
                    });
                }
                _ => {}
            }
        }
    }

    /// Compiles one `StatementIr::GeneratorLoop` for a resumable async body.
    ///
    /// Each wasm invocation of an async body runs at most one loop iteration:
    /// the suspension returns to the job queue and the driver re-enters the
    /// function from the top. `resume_state_offset` names the activation slot
    /// holding that state, which differs between a plain async function
    /// (`HEAP_ASYNC_RESUME_STATE_OFFSET`) and an async generator
    /// (`HEAP_ASYNC_GENERATOR_RESUME_STATE_OFFSET`); everything else about the
    /// loop shape is identical, so both share this emitter.
    ///
    /// The loop-carried state (`init` bindings, the iteration variable) lives in
    /// the activation record, so re-entry skips `init` and picks the iteration
    /// back up mid-body — ECMA-262 27.7.5.3 AsyncBlockStart resuming inside
    /// 14.7 iteration statements.
    fn compile_resumable_async_loop(
        &mut self,
        statement: &StatementIr,
        resume_state_offset: u64,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let StatementIr::GeneratorLoop {
            init,
            test,
            update,
            before_suspension,
            suspension_statement,
            after_suspension,
            entry_state,
            resume_state,
            exit_state,
        } = statement
        else {
            unreachable!("async loop compiler requires a generator loop");
        };
        let activation_local = self.new_target_payload_local().ok_or_else(|| {
            EmitError::unsupported("resumable async loop requires the function call ABI")
        })?;
        let state_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            activation_local,
            resume_state_offset,
            state_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(*entry_state as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(*resume_state as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);

        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(*entry_state as i64));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.initialize_direct_lexical_bindings(before_suspension, function);
        self.initialize_direct_lexical_bindings(after_suspension, function);
        if let Some(init) = init {
            self.compile_for_init(init, function)?;
            self.emit_dispatch_async_completion(function)?;
        }
        function.instruction(&Instruction::Else);
        self.compile_statement(suspension_statement, function)?;
        for statement in after_suspension {
            self.compile_statement(statement, function)?;
        }
        if let Some(update) = update {
            self.compile_expr_payload(update, function)?;
            function.instruction(&Instruction::Drop);
            self.emit_dispatch_async_completion(function)?;
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        let condition_local = self.reserve_temp_local();
        if let Some(test) = test {
            self.compile_truthy_i32(test, function)?;
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::LocalSet(condition_local));
            self.emit_dispatch_async_completion(function)?;
        } else {
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(condition_local));
        }
        function.instruction(&Instruction::LocalGet(condition_local));
        function.instruction(&Instruction::I32WrapI64);
        self.release_temp_local(condition_local);
        self.open_frame(ControlFrameKind::If, function);
        self.store_i64_const_at_offset(
            activation_local,
            resume_state_offset,
            u64::from(*entry_state),
            function,
        );
        self.initialize_direct_lexical_bindings(before_suspension, function);
        self.initialize_direct_lexical_bindings(after_suspension, function);
        for statement in before_suspension {
            self.compile_statement(statement, function)?;
        }
        self.compile_statement(suspension_statement, function)?;
        function.instruction(&Instruction::Else);
        self.store_i64_const_at_offset(
            activation_local,
            resume_state_offset,
            u64::from(*exit_state),
            function,
        );
        self.emit_statement_result(function, ValueKind::Undefined);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.release_temp_local(state_local);
        Ok(())
    }

    fn compile_async_generator_if(
        &mut self,
        statement: &StatementIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let StatementIr::GeneratorIf {
            condition,
            then_before_yield,
            then_yield_statement,
            then_after_yield,
            else_before_yield,
            else_yield_statement,
            else_after_yield,
            entry_state,
            then_resume_state,
            else_resume_state,
            exit_state,
        } = statement
        else {
            unreachable!("async-generator branch compiler requires a generator branch");
        };
        let activation_local = self.new_target_payload_local().ok_or_else(|| {
            EmitError::unsupported("async-generator branch requires the function call ABI")
        })?;
        let state_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_RESUME_STATE_OFFSET,
            state_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(*entry_state as i64));
        function.instruction(&Instruction::I64Eq);
        for resume_state in [then_resume_state, else_resume_state].into_iter().flatten() {
            function.instruction(&Instruction::LocalGet(state_local));
            function.instruction(&Instruction::I64Const(*resume_state as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
        }
        function.instruction(&Instruction::If(BlockType::Empty));

        self.compile_async_generator_if_resume_test(*then_resume_state, state_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        if let Some(yield_statement) = then_yield_statement {
            self.compile_statement(yield_statement, function)?;
        }
        for statement in then_after_yield {
            self.compile_statement(statement, function)?;
        }
        self.complete_async_generator_if_branch(activation_local, *exit_state, function);
        function.instruction(&Instruction::Else);

        self.compile_async_generator_if_resume_test(*else_resume_state, state_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        if let Some(yield_statement) = else_yield_statement {
            self.compile_statement(yield_statement, function)?;
        }
        for statement in else_after_yield {
            self.compile_statement(statement, function)?;
        }
        self.complete_async_generator_if_branch(activation_local, *exit_state, function);
        function.instruction(&Instruction::Else);

        self.compile_truthy_i32(condition, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        for statement in then_before_yield {
            self.compile_statement(statement, function)?;
        }
        self.compile_async_generator_if_initial_branch(
            activation_local,
            then_yield_statement.as_deref(),
            *exit_state,
            function,
        )?;
        function.instruction(&Instruction::Else);
        for statement in else_before_yield {
            self.compile_statement(statement, function)?;
        }
        self.compile_async_generator_if_initial_branch(
            activation_local,
            else_yield_statement.as_deref(),
            *exit_state,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(state_local);
        Ok(())
    }

    fn compile_async_generator_if_resume_test(
        &self,
        resume_state: Option<u32>,
        state_local: u32,
        function: &mut Function,
    ) {
        let Some(resume_state) = resume_state else {
            function.instruction(&Instruction::I32Const(0));
            return;
        };
        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(resume_state as i64));
        function.instruction(&Instruction::I64Eq);
    }

    fn compile_async_generator_if_initial_branch(
        &mut self,
        activation_local: u32,
        yield_statement: Option<&StatementIr>,
        exit_state: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let Some(yield_statement) = yield_statement else {
            self.complete_async_generator_if_branch(activation_local, exit_state, function);
            return Ok(());
        };
        let StatementIr::GeneratorYield { suspend_state, .. } = yield_statement else {
            return Err(EmitError::unsupported(
                "async-generator branch must contain one direct yield",
            ));
        };
        self.store_i64_const_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_RESUME_STATE_OFFSET,
            u64::from(*suspend_state),
            function,
        );
        self.compile_statement(yield_statement, function)
    }

    fn complete_async_generator_if_branch(
        &mut self,
        activation_local: u32,
        exit_state: u32,
        function: &mut Function,
    ) {
        self.store_i64_const_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_RESUME_STATE_OFFSET,
            u64::from(exit_state),
            function,
        );
        self.emit_statement_result(function, ValueKind::Undefined);
    }

    fn compile_async_generator_yield(
        &mut self,
        value: &TypedExpr,
        form: &YieldForm,
        suspend_state: u32,
        resume_state: u32,
        resume_mode: &GeneratorResumeModeIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match form {
            YieldForm::Plain => {}
            YieldForm::Delegate(_) => {
                return self.compile_async_generator_delegation(
                    value,
                    suspend_state,
                    resume_state,
                    resume_mode,
                    AsyncGeneratorDelegationKind::YieldStar,
                    function,
                );
            }
        }
        if matches!(resume_mode, GeneratorResumeModeIr::AssignProperty(_)) {
            return Err(EmitError::unsupported(
                "async-generator body dispatcher does not yet support property-assignment yield resumption",
            ));
        }

        let activation_local = self.new_target_payload_local().ok_or_else(|| {
            EmitError::unsupported("async-generator suspension requires the function call ABI")
        })?;
        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_RESUME_STATE_OFFSET,
            self.scratch_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(self.scratch_local));
        function.instruction(&Instruction::I64Const(suspend_state as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.compile_expr_to_locals(value, self.result_local, self.result_tag_local, function)?;
        self.emit_propagate_throw_from_locals_if_needed(
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.store_i64_const_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_RESUME_STATE_OFFSET,
            u64::from(resume_state),
            function,
        );
        self.store_i64_local_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_BODY_RESULT_PAYLOAD_OFFSET,
            self.result_local,
            function,
        );
        self.store_i64_local_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_BODY_RESULT_TAG_OFFSET,
            self.result_tag_local,
            function,
        );
        self.store_i64_const_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_BODY_STATUS_OFFSET,
            ASYNC_GENERATOR_BODY_STATUS_YIELD,
            function,
        );
        self.store_i64_const_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_EXECUTION_STATE_OFFSET,
            ASYNC_GENERATOR_STATE_SUSPENDED_YIELD,
            function,
        );
        self.set_completion_kind_with_aux(
            CompletionKind::Normal,
            i64::from(resume_state),
            function,
        );
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_RESUME_STATE_OFFSET,
            self.scratch_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(self.scratch_local));
        function.instruction(&Instruction::I64Const(resume_state as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        let resume_kind_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_RESUME_KIND_OFFSET,
            resume_kind_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_RESUME_PAYLOAD_OFFSET,
            self.result_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_RESUME_TAG_OFFSET,
            self.result_tag_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(resume_kind_local));
        function.instruction(&Instruction::I64Const(
            ASYNC_GENERATOR_RESUME_KIND_RETURN as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.set_completion_kind_with_aux(
            CompletionKind::Return,
            ASYNC_GENERATOR_RETURN_VALUE_ALREADY_AWAITED as i64,
            function,
        );
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(resume_kind_local));
        function.instruction(&Instruction::I64Const(
            ASYNC_GENERATOR_RESUME_KIND_THROW as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(resume_kind_local));
        function.instruction(&Instruction::I64Const(
            ASYNC_GENERATOR_RESUME_KIND_REJECT as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.set_completion_kind(CompletionKind::Throw, function);
        function.instruction(&Instruction::End);
        self.release_temp_local(resume_kind_local);
        self.emit_dispatch_async_generator_completion(function);

        match resume_mode {
            GeneratorResumeModeIr::Ignore => {
                self.emit_statement_result(function, ValueKind::Undefined);
            }
            GeneratorResumeModeIr::Return => {
                self.set_completion_kind(CompletionKind::Return, function);
                self.emit_dispatch_async_generator_completion(function);
            }
            GeneratorResumeModeIr::AssignIdentifier(name) => {
                if self.is_script_global_binding(name) && self.lookup_binding(name).is_none() {
                    self.emit_global_property_write(
                        name,
                        self.result_local,
                        self.result_tag_local,
                        function,
                    )?;
                } else {
                    let storage = self.lookup_binding(name).ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "unsupported in lila wasm-aot first slice: unbound identifier `{name}`"
                        ))
                    })?;
                    self.write_binding_from_locals(
                        storage,
                        self.result_local,
                        self.result_tag_local,
                        function,
                    );
                    self.mirror_binding_to_global_object(name, storage, function)?;
                }
                self.emit_statement_result(function, ValueKind::Undefined);
            }
            GeneratorResumeModeIr::AssignProperty(_) => unreachable!(),
        }
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn compile_async_generator_await(
        &mut self,
        value: &TypedExpr,
        suspend_state: u32,
        resume_state: u32,
        resume_mode: &AsyncResumeModeIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let activation_local = self.new_target_payload_local().ok_or_else(|| {
            EmitError::unsupported("async-generator suspension requires the function call ABI")
        })?;
        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_RESUME_STATE_OFFSET,
            self.scratch_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(self.scratch_local));
        function.instruction(&Instruction::I64Const(suspend_state as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.compile_expr_to_locals(value, self.result_local, self.result_tag_local, function)?;
        self.emit_propagate_throw_from_locals_if_needed(
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.store_i64_const_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_RESUME_STATE_OFFSET,
            u64::from(resume_state),
            function,
        );
        self.store_i64_local_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_BODY_RESULT_PAYLOAD_OFFSET,
            self.result_local,
            function,
        );
        self.store_i64_local_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_BODY_RESULT_TAG_OFFSET,
            self.result_tag_local,
            function,
        );
        self.emit_async_generator_await_reactions(
            activation_local,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.store_i64_const_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_BODY_STATUS_OFFSET,
            ASYNC_GENERATOR_BODY_STATUS_AWAIT,
            function,
        );
        self.store_i64_const_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_EXECUTION_STATE_OFFSET,
            ASYNC_GENERATOR_STATE_SUSPENDED_AWAIT,
            function,
        );
        self.set_completion_kind_with_aux(
            CompletionKind::Normal,
            i64::from(resume_state),
            function,
        );
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_RESUME_STATE_OFFSET,
            self.scratch_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(self.scratch_local));
        function.instruction(&Instruction::I64Const(resume_state as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        let resume_kind_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_RESUME_KIND_OFFSET,
            resume_kind_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_RESUME_PAYLOAD_OFFSET,
            self.result_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_RESUME_TAG_OFFSET,
            self.result_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(resume_kind_local));
        function.instruction(&Instruction::I64Const(
            ASYNC_GENERATOR_RESUME_KIND_REJECT as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(resume_kind_local));
        function.instruction(&Instruction::I64Const(
            ASYNC_GENERATOR_RESUME_KIND_THROW as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.set_completion_kind(CompletionKind::Throw, function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(resume_kind_local));
        function.instruction(&Instruction::I64Const(
            ASYNC_GENERATOR_RESUME_KIND_RETURN as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.set_completion_kind(CompletionKind::Return, function);
        function.instruction(&Instruction::End);
        self.release_temp_local(resume_kind_local);
        self.emit_dispatch_async_generator_completion(function);

        match resume_mode {
            AsyncResumeModeIr::Ignore => {
                self.emit_statement_result(function, ValueKind::Undefined);
            }
            AsyncResumeModeIr::Return => {
                self.set_completion_kind_with_aux(
                    CompletionKind::Return,
                    ASYNC_GENERATOR_RETURN_VALUE_ALREADY_AWAITED as i64,
                    function,
                );
                self.emit_dispatch_async_generator_completion(function);
            }
            AsyncResumeModeIr::AssignIdentifier(name) => {
                if self.is_script_global_binding(name) && self.lookup_binding(name).is_none() {
                    self.emit_global_property_write(
                        name,
                        self.result_local,
                        self.result_tag_local,
                        function,
                    )?;
                } else {
                    let storage = self.lookup_binding(name).ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "unsupported in lila wasm-aot first slice: unbound identifier `{name}`"
                        ))
                    })?;
                    self.write_binding_from_locals(
                        storage,
                        self.result_local,
                        self.result_tag_local,
                        function,
                    );
                    self.mirror_binding_to_global_object(name, storage, function)?;
                }
                self.emit_statement_result(function, ValueKind::Undefined);
            }
        }
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn compile_statement(
        &mut self,
        statement: &StatementIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if self.current_function_meta().is_some_and(|meta| {
            meta.protocol.execution_kind() == FunctionExecutionKind::AsyncGenerator
        }) {
            match statement {
                StatementIr::GeneratorYield {
                    value,
                    form,
                    suspend_state,
                    resume_state,
                    resume_mode,
                } => {
                    return self.compile_async_generator_yield(
                        value,
                        form,
                        *suspend_state,
                        *resume_state,
                        resume_mode,
                        function,
                    );
                }
                StatementIr::AsyncAwait {
                    value,
                    suspend_state,
                    resume_state,
                    resume_mode,
                } => {
                    return self.compile_async_generator_await(
                        value,
                        *suspend_state,
                        *resume_state,
                        resume_mode,
                        function,
                    );
                }
                StatementIr::GeneratorLoop { .. } => {
                    return self.compile_resumable_async_loop(
                        statement,
                        HEAP_ASYNC_GENERATOR_RESUME_STATE_OFFSET,
                        function,
                    );
                }
                StatementIr::GeneratorIf { .. } => {
                    return self.compile_async_generator_if(statement, function);
                }
                _ => {}
            }
        }

        // A plain async function reaches its loop bodies through the same
        // one-iteration-per-invocation state machine; only the activation slot
        // holding the resume state differs.
        if matches!(statement, StatementIr::GeneratorLoop { .. })
            && self
                .current_function_meta()
                .is_some_and(|meta| meta.protocol.execution_kind() == FunctionExecutionKind::Async)
        {
            return self.compile_resumable_async_loop(
                statement,
                HEAP_ASYNC_RESUME_STATE_OFFSET,
                function,
            );
        }

        match statement {
            StatementIr::ModuleUnitOnce { module, block } => {
                self.emit_module_unit_once(*module, block, function)?;
            }
            StatementIr::Empty => {
                self.emit_statement_result(function, ValueKind::Undefined);
            }
            StatementIr::Lexical { mode, name, init } => {
                let storage = self
                    .lookup_current_scope_binding(name)
                    .or_else(|| self.lookup_binding(name))
                    .unwrap_or_else(|| self.allocate_binding(name.clone(), *mode, init.kind));
                self.initialize_binding_uninitialized(storage, function);
                let value_local = self.reserve_temp_local();
                let tag_local = self.reserve_temp_local();
                self.compile_expr_to_locals(init, value_local, tag_local, function)?;
                self.emit_propagate_throw_from_locals_if_needed(value_local, tag_local, function)?;
                self.write_binding_from_locals(storage, value_local, tag_local, function);
                self.release_temp_local(tag_local);
                self.release_temp_local(value_local);
                self.emit_statement_result(function, ValueKind::Undefined);
            }
            StatementIr::AnnexBFunctionCopy {
                source_name,
                block_storage_name,
                target,
            } => {
                let source = self.lookup_binding(block_storage_name).ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "Annex B declaration `{source_name}` is missing block binding `{block_storage_name}`"
                    ))
                })?;
                self.read_binding_to_locals(
                    source,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                match target {
                    AnnexBFunctionCopyTargetIr::OwnerBinding { storage_name } => {
                        let target = self.lookup_owner_binding(storage_name).ok_or_else(|| {
                            EmitError::unsupported(format!(
                                "Annex B declaration `{source_name}` is missing owner binding `{storage_name}`"
                            ))
                        })?;
                        self.write_binding_from_locals(
                            target,
                            self.scratch_local,
                            self.result_tag_local,
                            function,
                        );
                    }
                    AnnexBFunctionCopyTargetIr::ScriptGlobal { name } => {
                        let target = self.lookup_owner_binding(name).ok_or_else(|| {
                            EmitError::unsupported(format!(
                                "Annex B declaration `{source_name}` is missing script-global binding `{name}`"
                            ))
                        })?;
                        self.write_binding_from_locals(
                            target,
                            self.scratch_local,
                            self.result_tag_local,
                            function,
                        );
                        self.mirror_binding_to_global_object(name, target, function)?;
                    }
                }
                self.emit_statement_result(function, ValueKind::Undefined);
            }
            StatementIr::Expression(expr) => {
                if !expr.possible_kinds.is_singleton()
                    || expr_result_tag_is_runtime_dynamic(&expr.expr)
                {
                    self.compile_expr_to_locals(
                        expr,
                        self.result_local,
                        self.result_tag_local,
                        function,
                    )?;
                    self.emit_propagate_throw_from_locals_if_needed(
                        self.result_local,
                        self.result_tag_local,
                        function,
                    )?;
                } else {
                    self.compile_expr_payload(expr, function)?;
                    // A thrown completion always co-homes the thrown value in
                    // `result_local` (every `emit_throw_*` / helper-call
                    // `store_call_results_to` sets it). The statement value left
                    // on the stack by `compile_expr_payload` is the *normal*
                    // result and is unrelated on the throw path, so propagate the
                    // throw from `result_local` — not `scratch_local`, which holds
                    // unrelated scratch state and would replace a real Error
                    // instance (e.g. a strict read-only / setter / Proxy write
                    // TypeError) with garbage.
                    self.emit_propagate_throw_from_locals_if_needed(
                        self.result_local,
                        self.result_tag_local,
                        function,
                    )?;
                    self.finish_statement_payload(function, expr.kind);
                }
            }
            StatementIr::GeneratorYield {
                value,
                form,
                suspend_state,
                resume_state,
                resume_mode,
            } => {
                match form {
                    YieldForm::Plain => {}
                    YieldForm::Delegate(_) => {
                        return self.compile_generator_delegation(
                            value,
                            *suspend_state,
                            *resume_state,
                            resume_mode,
                            function,
                        );
                    }
                }
                let activation_local = self.new_target_payload_local().ok_or_else(|| {
                    EmitError::unsupported("generator suspension requires the function call ABI")
                })?;
                self.load_i64_to_local_from_offset(
                    activation_local,
                    HEAP_GENERATOR_RESUME_STATE_OFFSET,
                    self.scratch_local,
                    function,
                );
                function.instruction(&Instruction::LocalGet(self.scratch_local));
                function.instruction(&Instruction::I64Const(*suspend_state as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                if let GeneratorResumeModeIr::AssignProperty(reference) = resume_mode {
                    self.prepare_suspended_property_reference(
                        reference,
                        activation_local,
                        function,
                    )?;
                }
                self.compile_expr_to_locals(
                    value,
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.store_i64_const_at_offset(
                    activation_local,
                    HEAP_GENERATOR_RESUME_STATE_OFFSET,
                    u64::from(*resume_state),
                    function,
                );
                self.set_completion_kind_with_aux(
                    CompletionKind::Normal,
                    i64::from(*resume_state),
                    function,
                );
                self.emit_return_current_completion(function);
                function.instruction(&Instruction::End);

                self.load_i64_to_local_from_offset(
                    activation_local,
                    HEAP_GENERATOR_RESUME_STATE_OFFSET,
                    self.scratch_local,
                    function,
                );
                function.instruction(&Instruction::LocalGet(self.scratch_local));
                function.instruction(&Instruction::I64Const(*resume_state as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                let resume_kind_local = self.reserve_temp_local();
                self.load_i64_to_local_from_offset(
                    activation_local,
                    HEAP_GENERATOR_RESUME_KIND_OFFSET,
                    resume_kind_local,
                    function,
                );
                function.instruction(&Instruction::LocalGet(resume_kind_local));
                function.instruction(&Instruction::I64Const(GENERATOR_RESUME_KIND_RETURN as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.load_i64_to_local_from_offset(
                    activation_local,
                    HEAP_GENERATOR_RESUME_PAYLOAD_OFFSET,
                    self.result_local,
                    function,
                );
                self.load_i64_to_local_from_offset(
                    activation_local,
                    HEAP_GENERATOR_RESUME_TAG_OFFSET,
                    self.result_tag_local,
                    function,
                );
                self.set_completion_kind(CompletionKind::Return, function);
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::LocalGet(resume_kind_local));
                function.instruction(&Instruction::I64Const(GENERATOR_RESUME_KIND_THROW as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.load_i64_to_local_from_offset(
                    activation_local,
                    HEAP_GENERATOR_RESUME_PAYLOAD_OFFSET,
                    self.result_local,
                    function,
                );
                self.load_i64_to_local_from_offset(
                    activation_local,
                    HEAP_GENERATOR_RESUME_TAG_OFFSET,
                    self.result_tag_local,
                    function,
                );
                self.set_completion_kind(CompletionKind::Throw, function);
                function.instruction(&Instruction::End);
                self.release_temp_local(resume_kind_local);
                self.emit_dispatch_current_completion(function)?;
                function.instruction(&Instruction::End);

                if matches!(resume_mode, GeneratorResumeModeIr::Return) {
                    self.load_i64_to_local_from_offset(
                        activation_local,
                        HEAP_GENERATOR_RESUME_STATE_OFFSET,
                        self.scratch_local,
                        function,
                    );
                    function.instruction(&Instruction::LocalGet(self.scratch_local));
                    function.instruction(&Instruction::I64Const(*resume_state as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.load_i64_to_local_from_offset(
                        activation_local,
                        HEAP_GENERATOR_RESUME_PAYLOAD_OFFSET,
                        self.result_local,
                        function,
                    );
                    self.load_i64_to_local_from_offset(
                        activation_local,
                        HEAP_GENERATOR_RESUME_TAG_OFFSET,
                        self.result_tag_local,
                        function,
                    );
                    self.set_completion_kind(CompletionKind::Normal, function);
                    self.emit_return_current_completion(function);
                    function.instruction(&Instruction::End);
                }
                if let GeneratorResumeModeIr::AssignIdentifier(name) = resume_mode {
                    self.load_i64_to_local_from_offset(
                        activation_local,
                        HEAP_GENERATOR_RESUME_STATE_OFFSET,
                        self.scratch_local,
                        function,
                    );
                    function.instruction(&Instruction::LocalGet(self.scratch_local));
                    function.instruction(&Instruction::I64Const(*resume_state as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.load_i64_to_local_from_offset(
                        activation_local,
                        HEAP_GENERATOR_RESUME_PAYLOAD_OFFSET,
                        self.result_local,
                        function,
                    );
                    self.load_i64_to_local_from_offset(
                        activation_local,
                        HEAP_GENERATOR_RESUME_TAG_OFFSET,
                        self.result_tag_local,
                        function,
                    );
                    if self.is_script_global_binding(name) && self.lookup_binding(name).is_none() {
                        self.emit_global_property_write(
                            name,
                            self.result_local,
                            self.result_tag_local,
                            function,
                        )?;
                    } else {
                        let storage = self.lookup_binding(name).ok_or_else(|| {
                            EmitError::unsupported(format!(
                                "unsupported in lila wasm-aot first slice: unbound identifier `{name}`"
                            ))
                        })?;
                        self.write_binding_from_locals(
                            storage,
                            self.result_local,
                            self.result_tag_local,
                            function,
                        );
                        self.mirror_binding_to_global_object(name, storage, function)?;
                    }
                    function.instruction(&Instruction::End);
                }
                if let GeneratorResumeModeIr::AssignProperty(reference) = resume_mode {
                    self.load_i64_to_local_from_offset(
                        activation_local,
                        HEAP_GENERATOR_RESUME_STATE_OFFSET,
                        self.scratch_local,
                        function,
                    );
                    function.instruction(&Instruction::LocalGet(self.scratch_local));
                    function.instruction(&Instruction::I64Const(*resume_state as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.load_i64_to_local_from_offset(
                        activation_local,
                        HEAP_GENERATOR_RESUME_PAYLOAD_OFFSET,
                        self.result_local,
                        function,
                    );
                    self.load_i64_to_local_from_offset(
                        activation_local,
                        HEAP_GENERATOR_RESUME_TAG_OFFSET,
                        self.result_tag_local,
                        function,
                    );
                    self.write_suspended_property_reference(
                        reference,
                        activation_local,
                        self.result_local,
                        self.result_tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::End);
                }
            }
            StatementIr::AsyncAwait {
                value,
                suspend_state,
                resume_state,
                resume_mode,
            } => {
                let activation_local = self.new_target_payload_local().ok_or_else(|| {
                    EmitError::unsupported("async suspension requires the function call ABI")
                })?;
                self.load_i64_to_local_from_offset(
                    activation_local,
                    HEAP_ASYNC_RESUME_STATE_OFFSET,
                    self.scratch_local,
                    function,
                );
                function.instruction(&Instruction::LocalGet(self.scratch_local));
                function.instruction(&Instruction::I64Const(*suspend_state as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.compile_expr_to_locals(
                    value,
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.store_i64_const_at_offset(
                    activation_local,
                    HEAP_ASYNC_RESUME_STATE_OFFSET,
                    u64::from(*resume_state),
                    function,
                );
                self.emit_async_await_reactions(
                    activation_local,
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                self.set_completion_kind_with_aux(
                    CompletionKind::Normal,
                    i64::from(*resume_state),
                    function,
                );
                self.emit_return_current_completion(function);
                function.instruction(&Instruction::End);

                self.load_i64_to_local_from_offset(
                    activation_local,
                    HEAP_ASYNC_RESUME_STATE_OFFSET,
                    self.scratch_local,
                    function,
                );
                function.instruction(&Instruction::LocalGet(self.scratch_local));
                function.instruction(&Instruction::I64Const(*resume_state as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                let resume_is_throw_local = self.reserve_temp_local();
                self.emit_load_async_function_resume_is_throw(
                    activation_local,
                    resume_is_throw_local,
                    function,
                );
                self.load_i64_to_local_from_offset(
                    activation_local,
                    HEAP_ASYNC_RESUME_PAYLOAD_OFFSET,
                    self.result_local,
                    function,
                );
                self.load_i64_to_local_from_offset(
                    activation_local,
                    HEAP_ASYNC_RESUME_TAG_OFFSET,
                    self.result_tag_local,
                    function,
                );
                function.instruction(&Instruction::LocalGet(resume_is_throw_local));
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.set_completion_kind(CompletionKind::Throw, function);
                self.emit_dispatch_current_completion(function)?;
                function.instruction(&Instruction::End);
                self.release_temp_local(resume_is_throw_local);

                match resume_mode {
                    AsyncResumeModeIr::Ignore => {
                        self.emit_statement_result(function, ValueKind::Undefined);
                    }
                    AsyncResumeModeIr::Return => {
                        self.set_completion_kind(CompletionKind::Return, function);
                        self.emit_dispatch_current_completion(function)?;
                    }
                    AsyncResumeModeIr::AssignIdentifier(name) => {
                        if self.is_script_global_binding(name)
                            && self.lookup_binding(name).is_none()
                        {
                            self.emit_global_property_write(
                                name,
                                self.result_local,
                                self.result_tag_local,
                                function,
                            )?;
                        } else {
                            let storage = self.lookup_binding(name).ok_or_else(|| {
                                EmitError::unsupported(format!(
                                    "unsupported in lila wasm-aot first slice: unbound identifier `{name}`"
                                ))
                            })?;
                            self.write_binding_from_locals(
                                storage,
                                self.result_local,
                                self.result_tag_local,
                                function,
                            );
                            self.mirror_binding_to_global_object(name, storage, function)?;
                        }
                        self.emit_statement_result(function, ValueKind::Undefined);
                    }
                }
                function.instruction(&Instruction::End);
            }
            StatementIr::GeneratorLoop {
                init,
                test,
                update,
                before_suspension,
                suspension_statement,
                after_suspension,
                entry_state,
                resume_state,
                exit_state,
            } => {
                let activation_local = self.new_target_payload_local().ok_or_else(|| {
                    EmitError::unsupported("generator loop requires the function call ABI")
                })?;
                let state_local = self.reserve_temp_local();
                self.load_i64_to_local_from_offset(
                    activation_local,
                    HEAP_GENERATOR_RESUME_STATE_OFFSET,
                    state_local,
                    function,
                );
                function.instruction(&Instruction::LocalGet(state_local));
                function.instruction(&Instruction::I64Const(*entry_state as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::LocalGet(state_local));
                function.instruction(&Instruction::I64Const(*resume_state as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I32Or);
                function.instruction(&Instruction::If(BlockType::Empty));

                function.instruction(&Instruction::LocalGet(state_local));
                function.instruction(&Instruction::I64Const(*entry_state as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                if let Some(init) = init {
                    self.compile_for_init(init, function)?;
                }
                function.instruction(&Instruction::Else);
                self.compile_statement(suspension_statement, function)?;
                for statement in after_suspension {
                    self.compile_statement(statement, function)?;
                }
                if let Some(update) = update {
                    self.compile_expr_payload(update, function)?;
                    function.instruction(&Instruction::Drop);
                }
                function.instruction(&Instruction::End);

                if let Some(test) = test {
                    self.compile_truthy_i32(test, function)?;
                } else {
                    function.instruction(&Instruction::I32Const(1));
                }
                function.instruction(&Instruction::If(BlockType::Empty));
                for statement in before_suspension {
                    self.compile_statement(statement, function)?;
                }
                let StatementIr::GeneratorYield { value, .. } = suspension_statement.as_ref()
                else {
                    return Err(EmitError::unsupported(
                        "generator loop must contain one direct yield",
                    ));
                };
                self.compile_expr_to_locals(
                    value,
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.store_i64_const_at_offset(
                    activation_local,
                    HEAP_GENERATOR_RESUME_STATE_OFFSET,
                    u64::from(*resume_state),
                    function,
                );
                self.set_completion_kind_with_aux(
                    CompletionKind::Normal,
                    i64::from(*resume_state),
                    function,
                );
                self.emit_return_current_completion(function);
                function.instruction(&Instruction::Else);
                self.store_i64_const_at_offset(
                    activation_local,
                    HEAP_GENERATOR_RESUME_STATE_OFFSET,
                    u64::from(*exit_state),
                    function,
                );
                self.emit_statement_result(function, ValueKind::Undefined);
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
                self.release_temp_local(state_local);
            }
            StatementIr::GeneratorIf {
                condition,
                then_before_yield,
                then_yield_statement,
                then_after_yield,
                else_before_yield,
                else_yield_statement,
                else_after_yield,
                entry_state,
                then_resume_state,
                else_resume_state,
                exit_state,
            } => {
                let activation_local = self.new_target_payload_local().ok_or_else(|| {
                    EmitError::unsupported("generator branch requires the function call ABI")
                })?;
                let state_local = self.reserve_temp_local();
                self.load_i64_to_local_from_offset(
                    activation_local,
                    HEAP_GENERATOR_RESUME_STATE_OFFSET,
                    state_local,
                    function,
                );
                function.instruction(&Instruction::LocalGet(state_local));
                function.instruction(&Instruction::I64Const(*entry_state as i64));
                function.instruction(&Instruction::I64Eq);
                if let Some(resume_state) = then_resume_state {
                    function.instruction(&Instruction::LocalGet(state_local));
                    function.instruction(&Instruction::I64Const(*resume_state as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::I32Or);
                }
                if let Some(resume_state) = else_resume_state {
                    function.instruction(&Instruction::LocalGet(state_local));
                    function.instruction(&Instruction::I64Const(*resume_state as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::I32Or);
                }
                function.instruction(&Instruction::If(BlockType::Empty));

                if let Some(resume_state) = then_resume_state {
                    function.instruction(&Instruction::LocalGet(state_local));
                    function.instruction(&Instruction::I64Const(*resume_state as i64));
                    function.instruction(&Instruction::I64Eq);
                } else {
                    function.instruction(&Instruction::I32Const(0));
                }
                function.instruction(&Instruction::If(BlockType::Empty));
                if let Some(yield_statement) = then_yield_statement {
                    self.compile_statement(yield_statement, function)?;
                }
                for statement in then_after_yield {
                    self.compile_statement(statement, function)?;
                }
                self.store_i64_const_at_offset(
                    activation_local,
                    HEAP_GENERATOR_RESUME_STATE_OFFSET,
                    u64::from(*exit_state),
                    function,
                );
                self.emit_statement_result(function, ValueKind::Undefined);
                function.instruction(&Instruction::Else);

                if let Some(resume_state) = else_resume_state {
                    function.instruction(&Instruction::LocalGet(state_local));
                    function.instruction(&Instruction::I64Const(*resume_state as i64));
                    function.instruction(&Instruction::I64Eq);
                } else {
                    function.instruction(&Instruction::I32Const(0));
                }
                function.instruction(&Instruction::If(BlockType::Empty));
                if let Some(yield_statement) = else_yield_statement {
                    self.compile_statement(yield_statement, function)?;
                }
                for statement in else_after_yield {
                    self.compile_statement(statement, function)?;
                }
                self.store_i64_const_at_offset(
                    activation_local,
                    HEAP_GENERATOR_RESUME_STATE_OFFSET,
                    u64::from(*exit_state),
                    function,
                );
                self.emit_statement_result(function, ValueKind::Undefined);
                function.instruction(&Instruction::Else);

                self.compile_truthy_i32(condition, function)?;
                function.instruction(&Instruction::If(BlockType::Empty));
                for statement in then_before_yield {
                    self.compile_statement(statement, function)?;
                }
                if let (Some(yield_statement), Some(resume_state)) =
                    (then_yield_statement, then_resume_state)
                {
                    let StatementIr::GeneratorYield { value, .. } = yield_statement.as_ref() else {
                        return Err(EmitError::unsupported(
                            "generator branch must contain one direct yield",
                        ));
                    };
                    self.compile_expr_to_locals(
                        value,
                        self.result_local,
                        self.result_tag_local,
                        function,
                    )?;
                    self.emit_propagate_throw_from_locals_if_needed(
                        self.result_local,
                        self.result_tag_local,
                        function,
                    )?;
                    self.store_i64_const_at_offset(
                        activation_local,
                        HEAP_GENERATOR_RESUME_STATE_OFFSET,
                        u64::from(*resume_state),
                        function,
                    );
                    self.set_completion_kind_with_aux(
                        CompletionKind::Normal,
                        i64::from(*resume_state),
                        function,
                    );
                    self.emit_return_current_completion(function);
                } else {
                    self.store_i64_const_at_offset(
                        activation_local,
                        HEAP_GENERATOR_RESUME_STATE_OFFSET,
                        u64::from(*exit_state),
                        function,
                    );
                    self.emit_statement_result(function, ValueKind::Undefined);
                }
                function.instruction(&Instruction::Else);
                for statement in else_before_yield {
                    self.compile_statement(statement, function)?;
                }
                if let (Some(yield_statement), Some(resume_state)) =
                    (else_yield_statement, else_resume_state)
                {
                    let StatementIr::GeneratorYield { value, .. } = yield_statement.as_ref() else {
                        return Err(EmitError::unsupported(
                            "generator branch must contain one direct yield",
                        ));
                    };
                    self.compile_expr_to_locals(
                        value,
                        self.result_local,
                        self.result_tag_local,
                        function,
                    )?;
                    self.emit_propagate_throw_from_locals_if_needed(
                        self.result_local,
                        self.result_tag_local,
                        function,
                    )?;
                    self.store_i64_const_at_offset(
                        activation_local,
                        HEAP_GENERATOR_RESUME_STATE_OFFSET,
                        u64::from(*resume_state),
                        function,
                    );
                    self.set_completion_kind_with_aux(
                        CompletionKind::Normal,
                        i64::from(*resume_state),
                        function,
                    );
                    self.emit_return_current_completion(function);
                } else {
                    self.store_i64_const_at_offset(
                        activation_local,
                        HEAP_GENERATOR_RESUME_STATE_OFFSET,
                        u64::from(*exit_state),
                        function,
                    );
                    self.emit_statement_result(function, ValueKind::Undefined);
                }
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
                self.release_temp_local(state_local);
            }
            StatementIr::Var(declarators) => {
                self.compile_var_declarators(declarators, function)?;
                self.emit_statement_result(function, ValueKind::Undefined);
            }
            StatementIr::ParameterInitialization { .. } => {
                self.emit_statement_result(function, ValueKind::Undefined);
            }
            StatementIr::LexicalBlock(statements) => {
                let async_resume_state_offset = self.async_await_resume_state_offset();
                let async_entry_state = async_resume_state_offset.and_then(|_| {
                    statements
                        .iter()
                        .find_map(Self::async_statement_entry_state)
                });
                if let (Some(entry_state), Some(resume_state_offset)) =
                    (async_entry_state, async_resume_state_offset)
                {
                    let activation_local = self
                        .new_target_payload_local()
                        .expect("async body must use the function call ABI");
                    self.load_i64_to_local_from_offset(
                        activation_local,
                        resume_state_offset,
                        self.scratch_local,
                        function,
                    );
                    function.instruction(&Instruction::LocalGet(self.scratch_local));
                    function.instruction(&Instruction::I64Const(entry_state as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.initialize_direct_lexical_bindings(statements, function);
                    function.instruction(&Instruction::End);
                    self.compile_async_statement_sequence(
                        statements,
                        entry_state,
                        resume_state_offset,
                        function,
                    )?;
                    return Ok(());
                }
                if let Some(entry_state) = statements
                    .iter()
                    .find_map(Self::generator_statement_entry_state)
                {
                    let activation_local = self
                        .new_target_payload_local()
                        .expect("generator body must use the function call ABI");
                    self.load_i64_to_local_from_offset(
                        activation_local,
                        HEAP_GENERATOR_RESUME_STATE_OFFSET,
                        self.scratch_local,
                        function,
                    );
                    function.instruction(&Instruction::LocalGet(self.scratch_local));
                    function.instruction(&Instruction::I64Const(entry_state as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.initialize_direct_lexical_bindings(statements, function);
                    function.instruction(&Instruction::End);
                    self.compile_generator_statement_sequence(statements, entry_state, function)?;
                    return Ok(());
                }
                for statement in statements {
                    let StatementIr::Lexical { mode, name, init } = statement else {
                        continue;
                    };
                    let storage = self
                        .lookup_current_scope_binding(name)
                        .or_else(|| self.lookup_binding(name))
                        .unwrap_or_else(|| self.allocate_binding(name.clone(), *mode, init.kind));
                    self.initialize_binding_uninitialized(storage, function);
                }
                for statement in statements {
                    self.compile_statement(statement, function)?;
                }
            }
            StatementIr::Block(block) => {
                self.push_scope();
                self.compile_block_contents(block, function)?;
                self.pop_scope();
            }
            StatementIr::Labelled { labels, statement } => {
                self.compile_labelled_statement(labels, statement, function)?;
            }
            StatementIr::Throw(value) => {
                self.compile_expr_to_locals(
                    value,
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::GlobalSet(throw_error_name_global_index(
                    self.uses_heap,
                )));
                self.emit_capture_throw_error_name(
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.set_completion_kind(CompletionKind::Throw, function);
                function.instruction(&Instruction::I64Const(-1));
                function.instruction(&Instruction::LocalSet(self.completion_aux_local));
                if let Some(target) = self.active_throw_target() {
                    self.emit_branch_to_target(target, function);
                } else {
                    self.emit_return_current_completion(function);
                }
            }
            StatementIr::TryCatch {
                try_block,
                catch_name,
                catch_source_name,
                catch_parameter_environment,
                catch_block,
                generator_plan,
                async_plan,
            } => {
                if let (Some(async_plan), Some(_)) =
                    (async_plan, self.async_await_resume_state_offset())
                {
                    self.compile_async_try_catch(
                        try_block,
                        catch_name,
                        catch_source_name,
                        catch_parameter_environment.as_ref(),
                        catch_block,
                        *async_plan,
                        function,
                    )?;
                } else if let Some(generator_plan) = generator_plan {
                    self.compile_generator_try_catch(
                        try_block,
                        catch_name,
                        catch_source_name,
                        catch_parameter_environment.as_ref(),
                        catch_block,
                        *generator_plan,
                        function,
                    )?;
                } else {
                    self.compile_try_catch(
                        try_block,
                        catch_name,
                        catch_source_name,
                        catch_parameter_environment.as_ref(),
                        catch_block,
                        function,
                    )?;
                }
            }
            StatementIr::TryFinally {
                try_block,
                finally_block,
                generator_plan,
                async_plan,
            } => {
                if let (Some(async_plan), Some(_)) =
                    (async_plan, self.async_await_resume_state_offset())
                {
                    self.compile_async_try_finally(
                        try_block,
                        finally_block,
                        *async_plan,
                        function,
                    )?;
                } else if let Some(generator_plan) = generator_plan {
                    self.compile_generator_try_finally(
                        try_block,
                        finally_block,
                        *generator_plan,
                        function,
                    )?;
                } else {
                    self.compile_try_finally(try_block, finally_block, function)?;
                }
            }
            StatementIr::TryCatchFinally {
                try_block,
                catch_name,
                catch_source_name,
                catch_parameter_environment,
                catch_block,
                finally_block,
                generator_plan,
                async_plan,
            } => {
                if let (Some(async_plan), Some(_)) =
                    (async_plan, self.async_await_resume_state_offset())
                {
                    self.compile_async_try_catch_finally(
                        try_block,
                        catch_name,
                        catch_source_name,
                        catch_parameter_environment.as_ref(),
                        catch_block,
                        finally_block,
                        *async_plan,
                        function,
                    )?;
                } else if let Some(generator_plan) = generator_plan {
                    self.compile_generator_try_catch_finally(
                        try_block,
                        catch_name,
                        catch_source_name,
                        catch_parameter_environment.as_ref(),
                        catch_block,
                        finally_block,
                        *generator_plan,
                        function,
                    )?;
                } else {
                    self.compile_try_catch_finally(
                        try_block,
                        catch_name,
                        catch_source_name,
                        catch_parameter_environment.as_ref(),
                        catch_block,
                        finally_block,
                        function,
                    )?;
                }
            }
            StatementIr::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.compile_truthy_i32(condition, function)?;
                self.open_frame(ControlFrameKind::If, function);
                self.compile_statement(then_branch, function)?;
                function.instruction(&Instruction::Else);
                if let Some(else_branch) = else_branch {
                    self.compile_statement(else_branch, function)?;
                } else {
                    self.emit_statement_result(function, ValueKind::Undefined);
                }
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
            }
            StatementIr::While { condition, body } => {
                self.compile_while(condition, body, &[], function)?;
            }
            StatementIr::DoWhile { body, condition } => {
                self.compile_do_while(body, condition, &[], function)?;
            }
            StatementIr::For {
                init,
                test,
                update,
                body,
                lexical_environment,
            } => {
                self.compile_for(
                    init.as_ref(),
                    test.as_ref(),
                    update.as_ref(),
                    body,
                    lexical_environment.as_ref(),
                    &[],
                    function,
                )?;
            }
            StatementIr::ForOfArray {
                mode,
                name,
                iterable,
                body,
                lexical_environment,
                ..
            } => {
                self.compile_for_of_array(
                    *mode,
                    name,
                    iterable,
                    body,
                    lexical_environment.as_ref(),
                    &[],
                    function,
                )?;
            }
            StatementIr::ForOfString {
                mode,
                name,
                iterable,
                body,
                lexical_environment,
                ..
            } => self.compile_for_of_string(
                *mode,
                name,
                iterable,
                body,
                lexical_environment.as_ref(),
                &[],
                function,
            )?,
            StatementIr::ForOfIterator {
                mode,
                name,
                iterable,
                body,
                lexical_environment,
                async_plan,
                ..
            } => {
                if let Some(async_plan) = async_plan {
                    if self.current_function_meta().is_some_and(|meta| {
                        meta.protocol.execution_kind() == FunctionExecutionKind::AsyncGenerator
                    }) && async_generator_for_await_is_transparent_yield(name, body)
                    {
                        self.compile_async_generator_delegation(
                            iterable,
                            async_plan.entry_state,
                            async_plan.exit_state,
                            &GeneratorResumeModeIr::Ignore,
                            AsyncGeneratorDelegationKind::ForAwaitYield,
                            function,
                        )?;
                        return Ok(());
                    }
                    self.compile_async_for_of_iterator(
                        *mode,
                        name,
                        iterable,
                        body,
                        lexical_environment.as_ref(),
                        async_plan,
                        &[],
                        function,
                    )?;
                } else {
                    self.compile_for_of_iterator(
                        *mode,
                        name,
                        iterable,
                        body,
                        lexical_environment.as_ref(),
                        &[],
                        function,
                    )?;
                }
            }
            StatementIr::ForInArray {
                mode,
                name,
                target,
                body,
                lexical_environment,
            } => self.compile_for_in_array(
                *mode,
                name,
                target,
                body,
                lexical_environment.as_ref(),
                &[],
                function,
            )?,
            StatementIr::ForInString {
                mode,
                name,
                target,
                body,
                lexical_environment,
            } => self.compile_for_in_string(
                *mode,
                name,
                target,
                body,
                lexical_environment.as_ref(),
                &[],
                function,
            )?,
            StatementIr::ForInObject {
                mode,
                name,
                target,
                body,
                lexical_environment,
            } => self.compile_for_in_object(
                *mode,
                name,
                target,
                body,
                lexical_environment.as_ref(),
                &[],
                function,
            )?,
            StatementIr::Switch {
                discriminant,
                lexical_environment,
                lexical_declarations,
                cases,
            } => {
                self.compile_switch(
                    discriminant,
                    lexical_environment.as_ref(),
                    lexical_declarations,
                    cases,
                    &[],
                    function,
                )?;
            }
            StatementIr::Debugger => {
                self.emit_statement_result(function, ValueKind::Undefined);
            }
            StatementIr::Return(value) => {
                if self.strict
                    && self.throw_handler_stack.is_empty()
                    && self.finally_stack.is_empty()
                    && !self.is_derived_constructor
                {
                    self.compile_return_position_expr(value, function)?;
                    return Ok(());
                }
                self.compile_expr_to_locals(
                    value,
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                if let Some(target) = self.finally_stack.last().copied() {
                    self.set_completion_kind(CompletionKind::Return, function);
                    self.emit_branch_to_target(target, function);
                } else {
                    self.set_completion_kind(CompletionKind::Return, function);
                    self.normalize_derived_constructor_result(function)?;
                    self.set_completion_kind(CompletionKind::Normal, function);
                    self.emit_return_current_completion(function);
                }
            }
            StatementIr::Break { label } => self.compile_break(label.as_deref(), function)?,
            StatementIr::Continue { label } => self.compile_continue(label.as_deref(), function)?,
        }
        Ok(())
    }

    fn compile_return_position_expr(
        &mut self,
        value: &TypedExpr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match &value.expr {
            ExprIr::CallIndirect {
                callee,
                this_arg,
                args,
                static_regexp_compilation: None,
            } if self.emit_tail_indirect_call(callee, this_arg.as_deref(), args, function)? => {}
            ExprIr::Conditional {
                condition,
                then_expr,
                else_expr,
            } => {
                self.compile_expr_to_locals(
                    condition,
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.compile_truthy_tagged_i32(self.result_tag_local, self.result_local, function)?;
                function.instruction(&Instruction::If(BlockType::Empty));
                self.compile_return_position_expr(then_expr, function)?;
                function.instruction(&Instruction::Else);
                self.compile_return_position_expr(else_expr, function)?;
                function.instruction(&Instruction::End);
            }
            ExprIr::LogicalShortCircuit { op, lhs, rhs } => {
                self.compile_expr_to_locals(
                    lhs,
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                match op {
                    LogicalBinaryOp::Coalesce => {
                        self.compile_nullish_tagged_i32(self.result_tag_local, function)?;
                    }
                    LogicalBinaryOp::And | LogicalBinaryOp::Or => {
                        self.compile_truthy_tagged_i32(
                            self.result_tag_local,
                            self.result_local,
                            function,
                        )?;
                    }
                }
                function.instruction(&Instruction::If(BlockType::Empty));
                match op {
                    LogicalBinaryOp::And | LogicalBinaryOp::Coalesce => {
                        self.compile_return_position_expr(rhs, function)?;
                    }
                    LogicalBinaryOp::Or => self.emit_return_from_result_locals(function),
                }
                function.instruction(&Instruction::Else);
                match op {
                    LogicalBinaryOp::And | LogicalBinaryOp::Coalesce => {
                        self.emit_return_from_result_locals(function);
                    }
                    LogicalBinaryOp::Or => {
                        self.compile_return_position_expr(rhs, function)?;
                    }
                }
                function.instruction(&Instruction::End);
            }
            ExprIr::Comma { lhs, rhs } => {
                self.compile_expr_to_locals(
                    lhs,
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.compile_return_position_expr(rhs, function)?;
            }
            _ => {
                self.compile_expr_to_locals(
                    value,
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_return_from_result_locals(function);
            }
        }
        Ok(())
    }

    fn emit_return_from_result_locals(&self, function: &mut Function) {
        self.set_completion_kind(CompletionKind::Return, function);
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_return_current_completion(function);
    }

    pub(crate) fn compile_labelled_statement(
        &mut self,
        labels: &[String],
        statement: &StatementIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match statement {
            StatementIr::Block(block) => {
                let break_frame = self.open_frame(ControlFrameKind::Block, function);
                self.push_labels(labels, break_frame, None);
                self.push_scope();
                self.compile_block_contents(block, function)?;
                self.pop_scope();
                self.pop_labels(labels.len());
                self.pop_control(ControlFrameKind::Block);
                function.instruction(&Instruction::End);
            }
            StatementIr::While { condition, body } => {
                self.compile_while(condition, body, labels, function)?;
            }
            StatementIr::DoWhile { body, condition } => {
                self.compile_do_while(body, condition, labels, function)?;
            }
            StatementIr::For {
                init,
                test,
                update,
                body,
                lexical_environment,
            } => {
                self.compile_for(
                    init.as_ref(),
                    test.as_ref(),
                    update.as_ref(),
                    body,
                    lexical_environment.as_ref(),
                    labels,
                    function,
                )?;
            }
            StatementIr::ForOfArray {
                mode,
                name,
                iterable,
                body,
                lexical_environment,
                ..
            } => {
                self.compile_for_of_array(
                    *mode,
                    name,
                    iterable,
                    body,
                    lexical_environment.as_ref(),
                    labels,
                    function,
                )?;
            }
            StatementIr::ForOfString {
                mode,
                name,
                iterable,
                body,
                lexical_environment,
                ..
            } => self.compile_for_of_string(
                *mode,
                name,
                iterable,
                body,
                lexical_environment.as_ref(),
                labels,
                function,
            )?,
            StatementIr::ForOfIterator {
                mode,
                name,
                iterable,
                body,
                lexical_environment,
                async_plan,
                ..
            } => {
                if let Some(async_plan) = async_plan {
                    self.compile_async_for_of_iterator(
                        *mode,
                        name,
                        iterable,
                        body,
                        lexical_environment.as_ref(),
                        async_plan,
                        labels,
                        function,
                    )?;
                } else {
                    self.compile_for_of_iterator(
                        *mode,
                        name,
                        iterable,
                        body,
                        lexical_environment.as_ref(),
                        labels,
                        function,
                    )?;
                }
            }
            StatementIr::ForInArray {
                mode,
                name,
                target,
                body,
                lexical_environment,
            } => self.compile_for_in_array(
                *mode,
                name,
                target,
                body,
                lexical_environment.as_ref(),
                labels,
                function,
            )?,
            StatementIr::ForInString {
                mode,
                name,
                target,
                body,
                lexical_environment,
            } => self.compile_for_in_string(
                *mode,
                name,
                target,
                body,
                lexical_environment.as_ref(),
                labels,
                function,
            )?,
            StatementIr::ForInObject {
                mode,
                name,
                target,
                body,
                lexical_environment,
            } => self.compile_for_in_object(
                *mode,
                name,
                target,
                body,
                lexical_environment.as_ref(),
                labels,
                function,
            )?,
            StatementIr::Switch {
                discriminant,
                lexical_environment,
                lexical_declarations,
                cases,
            } => {
                self.compile_switch(
                    discriminant,
                    lexical_environment.as_ref(),
                    lexical_declarations,
                    cases,
                    labels,
                    function,
                )?;
            }
            _ => {
                let break_frame = self.open_frame(ControlFrameKind::Block, function);
                self.push_labels(labels, break_frame, None);
                self.compile_statement(statement, function)?;
                self.pop_labels(labels.len());
                self.pop_control(ControlFrameKind::Block);
                function.instruction(&Instruction::End);
            }
        }
        Ok(())
    }

    pub(crate) fn compile_try_catch(
        &mut self,
        try_block: &BlockIr,
        catch_name: &str,
        catch_source_name: &str,
        catch_parameter_environment: Option<&LexicalEnvironmentIr>,
        catch_block: &BlockIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let _outer_frame = self.open_frame(ControlFrameKind::Block, function);
        let catch_frame = self.open_frame(ControlFrameKind::Block, function);
        self.throw_handler_stack.push(catch_frame);
        self.push_scope();
        self.compile_block_contents(try_block, function)?;
        self.pop_scope();
        self.throw_handler_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        self.push_scope();
        if let Some(environment) = catch_parameter_environment {
            self.emit_enter_lexical_environment(environment, function)?;
        }
        let catch_storage = self
            .lookup_current_scope_binding(catch_name)
            .unwrap_or_else(|| self.allocate_dynamic_binding_storage(catch_name));
        self.binding_scopes
            .last_mut()
            .expect("binding scope stack must exist")
            .insert(catch_name.to_string(), catch_storage);
        if catch_source_name != catch_name {
            self.binding_scopes
                .last_mut()
                .expect("binding scope stack must exist")
                .insert(catch_source_name.to_string(), catch_storage);
        }
        self.write_binding_from_locals(
            catch_storage,
            self.result_local,
            self.result_tag_local,
            function,
        );
        self.set_completion_kind(CompletionKind::Normal, function);
        self.push_scope();
        self.compile_block_contents(catch_block, function)?;
        self.pop_scope();
        if catch_parameter_environment.is_some() {
            self.emit_leave_lexical_environment(function);
        }
        self.pop_scope();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn emit_generator_state_in_range(
        &self,
        activation_local: u32,
        start_state: u32,
        end_state: u32,
        function: &mut Function,
    ) {
        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_GENERATOR_RESUME_STATE_OFFSET,
            self.scratch_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(self.scratch_local));
        function.instruction(&Instruction::I64Const(start_state as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(self.scratch_local));
        function.instruction(&Instruction::I64Const(end_state as i64));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32And);
    }

    fn emit_set_generator_resume_state(
        &self,
        activation_local: u32,
        state: u32,
        function: &mut Function,
    ) {
        self.store_i64_const_at_offset(
            activation_local,
            HEAP_GENERATOR_RESUME_STATE_OFFSET,
            u64::from(state),
            function,
        );
    }

    fn emit_async_state_in_range(
        &self,
        activation_local: u32,
        start_state: u32,
        end_state: u32,
        function: &mut Function,
    ) {
        let resume_state_offset = self
            .async_await_resume_state_offset()
            .expect("async state range requires an async function activation");
        self.load_i64_to_local_from_offset(
            activation_local,
            resume_state_offset,
            self.scratch_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(self.scratch_local));
        function.instruction(&Instruction::I64Const(start_state as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(self.scratch_local));
        function.instruction(&Instruction::I64Const(end_state as i64));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32And);
    }

    fn emit_async_try_state_in_range(
        &self,
        activation_local: u32,
        start_state: u32,
        end_state: u32,
        function: &mut Function,
    ) {
        if !self.current_function_meta().is_some_and(|meta| {
            meta.protocol.execution_kind() == FunctionExecutionKind::AsyncGenerator
        }) {
            self.emit_async_state_in_range(activation_local, start_state, end_state, function);
            return;
        }

        let resume_state_offset = self
            .async_await_resume_state_offset()
            .expect("async try state range requires an async function activation");
        self.load_i64_to_local_from_offset(
            activation_local,
            resume_state_offset,
            self.scratch_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(self.scratch_local));
        function.instruction(&Instruction::I64Const(start_state as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(self.scratch_local));
        function.instruction(&Instruction::I64Const(end_state as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
    }

    fn emit_async_finalizer_needs_pending_completion(
        &self,
        activation_local: u32,
        finally_entry_state: u32,
        function: &mut Function,
    ) {
        let resume_state_offset = self
            .async_await_resume_state_offset()
            .expect("async finalizer requires an async function activation");
        self.load_i64_to_local_from_offset(
            activation_local,
            resume_state_offset,
            self.scratch_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(self.scratch_local));
        function.instruction(&Instruction::I64Const(finally_entry_state as i64));
        if self.current_function_meta().is_some_and(|meta| {
            meta.protocol.execution_kind() == FunctionExecutionKind::AsyncGenerator
        }) {
            function.instruction(&Instruction::I64LeU);
        } else {
            function.instruction(&Instruction::I64LtU);
        }
    }

    fn emit_set_async_resume_state(
        &self,
        activation_local: u32,
        state: u32,
        function: &mut Function,
    ) {
        let resume_state_offset = self
            .async_await_resume_state_offset()
            .expect("async resume state requires an async function activation");
        self.store_i64_const_at_offset(
            activation_local,
            resume_state_offset,
            u64::from(state),
            function,
        );
    }

    fn compile_generator_try_catch(
        &mut self,
        try_block: &BlockIr,
        catch_name: &str,
        catch_source_name: &str,
        catch_parameter_environment: Option<&LexicalEnvironmentIr>,
        catch_block: &BlockIr,
        generator_plan: GeneratorTryPlanIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let activation_local = self.new_target_payload_local().ok_or_else(|| {
            EmitError::unsupported("generator exception handling requires the function call ABI")
        })?;
        let catch_entry_state = generator_plan.catch_entry_state.ok_or_else(|| {
            EmitError::unsupported("generator try/catch is missing its catch resume state")
        })?;
        let catch_exit_state = generator_plan.catch_exit_state.ok_or_else(|| {
            EmitError::unsupported("generator try/catch is missing its catch exit state")
        })?;

        self.emit_generator_state_in_range(
            activation_local,
            generator_plan.entry_state,
            generator_plan.exit_state,
            function,
        );
        self.open_frame(ControlFrameKind::If, function);
        let outer_frame = self.open_frame(ControlFrameKind::Block, function);
        let catch_frame = self.open_frame(ControlFrameKind::Block, function);
        self.throw_handler_stack.push(catch_frame);

        self.emit_generator_state_in_range(
            activation_local,
            generator_plan.entry_state,
            generator_plan.try_exit_state,
            function,
        );
        self.open_frame(ControlFrameKind::If, function);
        self.push_scope();
        self.compile_generator_block_contents(
            try_block,
            generator_plan.entry_state,
            true,
            function,
        )?;
        self.pop_scope();
        self.emit_set_generator_resume_state(activation_local, generator_plan.exit_state, function);
        self.emit_branch_to_target(outer_frame, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.throw_handler_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

        self.push_scope();
        let catch_storage = self
            .lookup_current_scope_binding(catch_name)
            .unwrap_or_else(|| self.allocate_dynamic_binding_storage(catch_name));
        self.binding_scopes
            .last_mut()
            .expect("binding scope stack must exist")
            .insert(catch_name.to_string(), catch_storage);
        if catch_source_name != catch_name {
            self.binding_scopes
                .last_mut()
                .expect("binding scope stack must exist")
                .insert(catch_source_name.to_string(), catch_storage);
        }

        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        if let Some(environment) = catch_parameter_environment {
            self.emit_enter_lexical_environment(environment, function)?;
        }
        self.write_binding_from_locals(
            catch_storage,
            self.result_local,
            self.result_tag_local,
            function,
        );
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_set_generator_resume_state(activation_local, catch_entry_state, function);
        function.instruction(&Instruction::End);

        self.push_scope();
        self.compile_generator_block_contents(catch_block, catch_entry_state, true, function)?;
        self.pop_scope();
        if catch_parameter_environment.is_some() {
            self.emit_leave_lexical_environment(function);
        }
        self.pop_scope();
        self.emit_set_generator_resume_state(activation_local, generator_plan.exit_state, function);

        debug_assert_eq!(catch_exit_state, generator_plan.exit_state);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn compile_generator_try_finally(
        &mut self,
        try_block: &BlockIr,
        finally_block: &BlockIr,
        generator_plan: GeneratorTryPlanIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let activation_local = self.new_target_payload_local().ok_or_else(|| {
            EmitError::unsupported("generator finalization requires the function call ABI")
        })?;
        let finally_entry_state = generator_plan.finally_entry_state.ok_or_else(|| {
            EmitError::unsupported("generator try/finally is missing its finalizer resume state")
        })?;
        let finally_exit_state = generator_plan.finally_exit_state.ok_or_else(|| {
            EmitError::unsupported("generator try/finally is missing its finalizer exit state")
        })?;

        self.emit_generator_state_in_range(
            activation_local,
            generator_plan.entry_state,
            generator_plan.exit_state,
            function,
        );
        self.open_frame(ControlFrameKind::If, function);
        self.open_frame(ControlFrameKind::Block, function);
        let finally_entry_frame = self.open_frame(ControlFrameKind::Block, function);
        self.finally_stack.push(finally_entry_frame);

        self.emit_generator_state_in_range(
            activation_local,
            generator_plan.entry_state,
            generator_plan.try_exit_state,
            function,
        );
        self.open_frame(ControlFrameKind::If, function);
        self.push_scope();
        self.compile_generator_block_contents(
            try_block,
            generator_plan.entry_state,
            true,
            function,
        )?;
        self.pop_scope();
        self.emit_branch_to_target(finally_entry_frame, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.finally_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_GENERATOR_RESUME_STATE_OFFSET,
            self.scratch_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(self.scratch_local));
        function.instruction(&Instruction::I64Const(finally_entry_state as i64));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_push_generator_pending_completion(function)?;
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_set_generator_resume_state(activation_local, finally_entry_state, function);
        function.instruction(&Instruction::End);

        let finalizer_epilogue_frame = self.open_frame(ControlFrameKind::Block, function);
        self.finally_stack.push(finalizer_epilogue_frame);
        self.generator_finalizer_depth += 1;
        self.push_scope();
        self.compile_generator_block_contents(finally_block, finally_entry_state, true, function)?;
        self.pop_scope();
        self.generator_finalizer_depth -= 1;
        self.finally_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_NORMAL));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_pop_and_restore_generator_pending_completion(function)?;
        function.instruction(&Instruction::Else);
        self.emit_discard_generator_pending_completion(function)?;
        function.instruction(&Instruction::End);
        self.emit_set_generator_resume_state(activation_local, generator_plan.exit_state, function);
        self.emit_dispatch_current_completion(function)?;

        debug_assert_eq!(finally_exit_state, generator_plan.exit_state);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn compile_generator_try_catch_finally(
        &mut self,
        try_block: &BlockIr,
        catch_name: &str,
        catch_source_name: &str,
        catch_parameter_environment: Option<&LexicalEnvironmentIr>,
        catch_block: &BlockIr,
        finally_block: &BlockIr,
        generator_plan: GeneratorTryPlanIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let activation_local = self.new_target_payload_local().ok_or_else(|| {
            EmitError::unsupported("generator exception handling requires the function call ABI")
        })?;
        let catch_entry_state = generator_plan.catch_entry_state.ok_or_else(|| {
            EmitError::unsupported("generator try/catch is missing its catch resume state")
        })?;
        let catch_exit_state = generator_plan.catch_exit_state.ok_or_else(|| {
            EmitError::unsupported("generator try/catch is missing its catch exit state")
        })?;
        let finally_entry_state = generator_plan.finally_entry_state.ok_or_else(|| {
            EmitError::unsupported("generator try/finally is missing its finalizer resume state")
        })?;
        let finally_exit_state = generator_plan.finally_exit_state.ok_or_else(|| {
            EmitError::unsupported("generator try/finally is missing its finalizer exit state")
        })?;

        self.emit_generator_state_in_range(
            activation_local,
            generator_plan.entry_state,
            generator_plan.exit_state,
            function,
        );
        self.open_frame(ControlFrameKind::If, function);
        self.open_frame(ControlFrameKind::Block, function);
        self.open_frame(ControlFrameKind::Block, function);
        let catch_skip_frame = self.open_frame(ControlFrameKind::Block, function);
        self.finally_stack.push(catch_skip_frame);
        let catch_frame = self.open_frame(ControlFrameKind::Block, function);
        self.throw_handler_stack.push(catch_frame);

        self.emit_generator_state_in_range(
            activation_local,
            generator_plan.entry_state,
            generator_plan.try_exit_state,
            function,
        );
        self.open_frame(ControlFrameKind::If, function);
        self.push_scope();
        self.compile_generator_block_contents(
            try_block,
            generator_plan.entry_state,
            true,
            function,
        )?;
        self.pop_scope();
        self.emit_branch_to_target(catch_skip_frame, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.throw_handler_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        self.emit_generator_state_in_range(
            activation_local,
            catch_entry_state,
            catch_exit_state,
            function,
        );
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        self.push_scope();
        let catch_storage = self
            .lookup_current_scope_binding(catch_name)
            .unwrap_or_else(|| self.allocate_dynamic_binding_storage(catch_name));
        self.binding_scopes
            .last_mut()
            .expect("binding scope stack must exist")
            .insert(catch_name.to_string(), catch_storage);
        if catch_source_name != catch_name {
            self.binding_scopes
                .last_mut()
                .expect("binding scope stack must exist")
                .insert(catch_source_name.to_string(), catch_storage);
        }

        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        if let Some(environment) = catch_parameter_environment {
            self.emit_enter_lexical_environment(environment, function)?;
        }
        self.write_binding_from_locals(
            catch_storage,
            self.result_local,
            self.result_tag_local,
            function,
        );
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_set_generator_resume_state(activation_local, catch_entry_state, function);
        function.instruction(&Instruction::End);

        self.push_scope();
        self.compile_generator_block_contents(catch_block, catch_entry_state, true, function)?;
        self.pop_scope();
        if catch_parameter_environment.is_some() {
            self.emit_leave_lexical_environment(function);
        }
        self.pop_scope();
        self.emit_branch_to_target(catch_skip_frame, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.finally_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_GENERATOR_RESUME_STATE_OFFSET,
            self.scratch_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(self.scratch_local));
        function.instruction(&Instruction::I64Const(finally_entry_state as i64));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_push_generator_pending_completion(function)?;
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_set_generator_resume_state(activation_local, finally_entry_state, function);
        function.instruction(&Instruction::End);

        let finalizer_epilogue_frame = self.open_frame(ControlFrameKind::Block, function);
        self.finally_stack.push(finalizer_epilogue_frame);
        self.generator_finalizer_depth += 1;
        self.push_scope();
        self.compile_generator_block_contents(finally_block, finally_entry_state, true, function)?;
        self.pop_scope();
        self.generator_finalizer_depth -= 1;
        self.finally_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_NORMAL));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_pop_and_restore_generator_pending_completion(function)?;
        function.instruction(&Instruction::Else);
        self.emit_discard_generator_pending_completion(function)?;
        function.instruction(&Instruction::End);
        self.emit_set_generator_resume_state(activation_local, generator_plan.exit_state, function);
        self.emit_dispatch_current_completion(function)?;

        debug_assert_eq!(catch_exit_state, finally_entry_state);
        debug_assert_eq!(finally_exit_state, generator_plan.exit_state);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn compile_async_try_catch(
        &mut self,
        try_block: &BlockIr,
        catch_name: &str,
        catch_source_name: &str,
        catch_parameter_environment: Option<&LexicalEnvironmentIr>,
        catch_block: &BlockIr,
        async_plan: AsyncTryPlanIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let activation_local = self.new_target_payload_local().ok_or_else(|| {
            EmitError::unsupported("async exception handling requires the function call ABI")
        })?;
        let resume_state_offset = self.async_await_resume_state_offset().ok_or_else(|| {
            EmitError::unsupported("async exception handling requires an async activation")
        })?;
        let catch_entry_state = async_plan.catch_entry_state.ok_or_else(|| {
            EmitError::unsupported("async try/catch is missing its catch resume state")
        })?;
        let catch_exit_state = async_plan.catch_exit_state.ok_or_else(|| {
            EmitError::unsupported("async try/catch is missing its catch exit state")
        })?;

        self.emit_async_try_state_in_range(
            activation_local,
            async_plan.entry_state,
            async_plan.exit_state,
            function,
        );
        self.open_frame(ControlFrameKind::If, function);
        let outer_frame = self.open_frame(ControlFrameKind::Block, function);
        let catch_frame = self.open_frame(ControlFrameKind::Block, function);
        self.throw_handler_stack.push(catch_frame);

        self.emit_async_try_state_in_range(
            activation_local,
            async_plan.entry_state,
            async_plan.try_exit_state,
            function,
        );
        self.open_frame(ControlFrameKind::If, function);
        self.push_scope();
        self.compile_async_block_contents(
            try_block,
            async_plan.entry_state,
            true,
            resume_state_offset,
            function,
        )?;
        self.pop_scope();
        self.emit_set_async_resume_state(activation_local, async_plan.exit_state, function);
        self.emit_branch_to_target(outer_frame, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.throw_handler_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

        self.push_scope();
        let catch_storage = self
            .lookup_current_scope_binding(catch_name)
            .unwrap_or_else(|| self.allocate_dynamic_binding_storage(catch_name));
        self.binding_scopes
            .last_mut()
            .expect("binding scope stack must exist")
            .insert(catch_name.to_string(), catch_storage);
        if catch_source_name != catch_name {
            self.binding_scopes
                .last_mut()
                .expect("binding scope stack must exist")
                .insert(catch_source_name.to_string(), catch_storage);
        }

        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        let thrown_payload_local = self.reserve_temp_local();
        let thrown_tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalSet(thrown_payload_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalSet(thrown_tag_local));
        if let Some(environment) = catch_parameter_environment {
            self.emit_enter_lexical_environment(environment, function)?;
        }
        self.write_binding_from_locals(
            catch_storage,
            thrown_payload_local,
            thrown_tag_local,
            function,
        );
        self.release_temp_local(thrown_tag_local);
        self.release_temp_local(thrown_payload_local);
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_set_async_resume_state(activation_local, catch_entry_state, function);
        function.instruction(&Instruction::End);

        self.push_scope();
        self.compile_async_block_contents(
            catch_block,
            catch_entry_state,
            true,
            resume_state_offset,
            function,
        )?;
        self.pop_scope();
        if catch_parameter_environment.is_some() {
            self.emit_leave_lexical_environment(function);
        }
        self.pop_scope();
        self.emit_set_async_resume_state(activation_local, async_plan.exit_state, function);

        debug_assert_eq!(catch_exit_state, async_plan.exit_state);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn compile_async_try_catch_finally(
        &mut self,
        try_block: &BlockIr,
        catch_name: &str,
        catch_source_name: &str,
        catch_parameter_environment: Option<&LexicalEnvironmentIr>,
        catch_block: &BlockIr,
        finally_block: &BlockIr,
        async_plan: AsyncTryPlanIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let activation_local = self.new_target_payload_local().ok_or_else(|| {
            EmitError::unsupported("async exception handling requires the function call ABI")
        })?;
        let resume_state_offset = self.async_await_resume_state_offset().ok_or_else(|| {
            EmitError::unsupported("async exception handling requires an async activation")
        })?;
        let catch_entry_state = async_plan.catch_entry_state.ok_or_else(|| {
            EmitError::unsupported("async try/catch is missing its catch resume state")
        })?;
        let catch_exit_state = async_plan.catch_exit_state.ok_or_else(|| {
            EmitError::unsupported("async try/catch is missing its catch exit state")
        })?;
        let finally_entry_state = async_plan.finally_entry_state.ok_or_else(|| {
            EmitError::unsupported("async try/finally is missing its finalizer resume state")
        })?;
        let finally_exit_state = async_plan.finally_exit_state.ok_or_else(|| {
            EmitError::unsupported("async try/finally is missing its finalizer exit state")
        })?;

        self.emit_async_try_state_in_range(
            activation_local,
            async_plan.entry_state,
            async_plan.exit_state,
            function,
        );
        self.open_frame(ControlFrameKind::If, function);
        self.open_frame(ControlFrameKind::Block, function);
        self.open_frame(ControlFrameKind::Block, function);
        let catch_skip_frame = self.open_frame(ControlFrameKind::Block, function);
        self.finally_stack.push(catch_skip_frame);
        let catch_frame = self.open_frame(ControlFrameKind::Block, function);
        self.throw_handler_stack.push(catch_frame);

        self.emit_async_try_state_in_range(
            activation_local,
            async_plan.entry_state,
            async_plan.try_exit_state,
            function,
        );
        self.open_frame(ControlFrameKind::If, function);
        self.push_scope();
        self.compile_async_block_contents(
            try_block,
            async_plan.entry_state,
            true,
            resume_state_offset,
            function,
        )?;
        self.pop_scope();
        self.emit_branch_to_target(catch_skip_frame, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.throw_handler_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        self.emit_async_try_state_in_range(
            activation_local,
            catch_entry_state,
            catch_exit_state,
            function,
        );
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        self.push_scope();
        let catch_storage = self
            .lookup_current_scope_binding(catch_name)
            .unwrap_or_else(|| self.allocate_dynamic_binding_storage(catch_name));
        self.binding_scopes
            .last_mut()
            .expect("binding scope stack must exist")
            .insert(catch_name.to_string(), catch_storage);
        if catch_source_name != catch_name {
            self.binding_scopes
                .last_mut()
                .expect("binding scope stack must exist")
                .insert(catch_source_name.to_string(), catch_storage);
        }

        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        let thrown_payload_local = self.reserve_temp_local();
        let thrown_tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalSet(thrown_payload_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalSet(thrown_tag_local));
        if let Some(environment) = catch_parameter_environment {
            self.emit_enter_lexical_environment(environment, function)?;
        }
        self.write_binding_from_locals(
            catch_storage,
            thrown_payload_local,
            thrown_tag_local,
            function,
        );
        self.release_temp_local(thrown_tag_local);
        self.release_temp_local(thrown_payload_local);
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_set_async_resume_state(activation_local, catch_entry_state, function);
        function.instruction(&Instruction::End);

        self.push_scope();
        self.compile_async_block_contents(
            catch_block,
            catch_entry_state,
            true,
            resume_state_offset,
            function,
        )?;
        self.pop_scope();
        if catch_parameter_environment.is_some() {
            self.emit_leave_lexical_environment(function);
        }
        self.pop_scope();
        self.emit_branch_to_target(catch_skip_frame, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.finally_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

        self.emit_async_finalizer_needs_pending_completion(
            activation_local,
            finally_entry_state,
            function,
        );
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_push_async_pending_completion(function)?;
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_set_async_resume_state(activation_local, finally_entry_state, function);
        function.instruction(&Instruction::End);

        let finalizer_epilogue_frame = self.open_frame(ControlFrameKind::Block, function);
        self.finally_stack.push(finalizer_epilogue_frame);
        self.push_scope();
        self.compile_async_block_contents(
            finally_block,
            finally_entry_state,
            true,
            resume_state_offset,
            function,
        )?;
        self.pop_scope();
        self.finally_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_NORMAL));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_pop_and_restore_async_pending_completion(function)?;
        function.instruction(&Instruction::Else);
        self.emit_discard_async_pending_completion(function)?;
        function.instruction(&Instruction::End);
        self.emit_set_async_resume_state(activation_local, async_plan.exit_state, function);
        self.emit_dispatch_async_completion(function)?;

        debug_assert_eq!(catch_exit_state, finally_entry_state);
        debug_assert_eq!(finally_exit_state, async_plan.exit_state);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn compile_async_try_finally(
        &mut self,
        try_block: &BlockIr,
        finally_block: &BlockIr,
        async_plan: AsyncTryPlanIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let activation_local = self.new_target_payload_local().ok_or_else(|| {
            EmitError::unsupported("async finalization requires the function call ABI")
        })?;
        let resume_state_offset = self.async_await_resume_state_offset().ok_or_else(|| {
            EmitError::unsupported("async finalization requires an async activation")
        })?;
        let finally_entry_state = async_plan.finally_entry_state.ok_or_else(|| {
            EmitError::unsupported("async try/finally is missing its finalizer resume state")
        })?;
        let finally_exit_state = async_plan.finally_exit_state.ok_or_else(|| {
            EmitError::unsupported("async try/finally is missing its finalizer exit state")
        })?;

        self.emit_async_try_state_in_range(
            activation_local,
            async_plan.entry_state,
            async_plan.exit_state,
            function,
        );
        self.open_frame(ControlFrameKind::If, function);
        self.open_frame(ControlFrameKind::Block, function);
        let finally_entry_frame = self.open_frame(ControlFrameKind::Block, function);
        self.finally_stack.push(finally_entry_frame);

        self.emit_async_try_state_in_range(
            activation_local,
            async_plan.entry_state,
            async_plan.try_exit_state,
            function,
        );
        self.open_frame(ControlFrameKind::If, function);
        self.push_scope();
        self.compile_async_block_contents(
            try_block,
            async_plan.entry_state,
            true,
            resume_state_offset,
            function,
        )?;
        self.pop_scope();
        self.emit_branch_to_target(finally_entry_frame, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.finally_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

        self.emit_async_finalizer_needs_pending_completion(
            activation_local,
            finally_entry_state,
            function,
        );
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_push_async_pending_completion(function)?;
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_set_async_resume_state(activation_local, finally_entry_state, function);
        function.instruction(&Instruction::End);

        let finalizer_epilogue_frame = self.open_frame(ControlFrameKind::Block, function);
        self.finally_stack.push(finalizer_epilogue_frame);
        self.push_scope();
        self.compile_async_block_contents(
            finally_block,
            finally_entry_state,
            true,
            resume_state_offset,
            function,
        )?;
        self.pop_scope();
        self.finally_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_NORMAL));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_pop_and_restore_async_pending_completion(function)?;
        function.instruction(&Instruction::Else);
        self.emit_discard_async_pending_completion(function)?;
        function.instruction(&Instruction::End);
        self.emit_set_async_resume_state(activation_local, async_plan.exit_state, function);
        self.emit_dispatch_async_completion(function)?;

        debug_assert_eq!(finally_exit_state, async_plan.exit_state);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn compile_try_finally(
        &mut self,
        try_block: &BlockIr,
        finally_block: &BlockIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let saved_payload_local = self.reserve_temp_local();
        let saved_tag_local = self.reserve_temp_local();
        let saved_completion_local = self.reserve_temp_local();
        let saved_aux_local = self.reserve_temp_local();

        let _outer_frame = self.open_frame(ControlFrameKind::Block, function);
        let finally_frame = self.open_frame(ControlFrameKind::Block, function);
        self.finally_stack.push(finally_frame);
        self.push_scope();
        self.compile_block_contents(try_block, function)?;
        self.pop_scope();
        self.finally_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

        self.save_current_completion(
            saved_payload_local,
            saved_tag_local,
            saved_completion_local,
            saved_aux_local,
            function,
        );
        self.set_completion_kind(CompletionKind::Normal, function);
        self.push_scope();
        self.compile_block_contents(finally_block, function)?;
        self.pop_scope();
        self.emit_resume_after_finally(
            saved_payload_local,
            saved_tag_local,
            saved_completion_local,
            saved_aux_local,
            function,
        )?;
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.release_temp_local(saved_aux_local);
        self.release_temp_local(saved_completion_local);
        self.release_temp_local(saved_tag_local);
        self.release_temp_local(saved_payload_local);
        Ok(())
    }

    pub(crate) fn compile_try_catch_finally(
        &mut self,
        try_block: &BlockIr,
        catch_name: &str,
        catch_source_name: &str,
        catch_parameter_environment: Option<&LexicalEnvironmentIr>,
        catch_block: &BlockIr,
        finally_block: &BlockIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let saved_payload_local = self.reserve_temp_local();
        let saved_tag_local = self.reserve_temp_local();
        let saved_completion_local = self.reserve_temp_local();
        let saved_aux_local = self.reserve_temp_local();

        let _outer_frame = self.open_frame(ControlFrameKind::Block, function);
        let _finally_frame = self.open_frame(ControlFrameKind::Block, function);
        let catch_skip_frame = self.open_frame(ControlFrameKind::Block, function);
        let catch_frame = self.open_frame(ControlFrameKind::Block, function);
        self.throw_handler_stack.push(catch_frame);
        // `br` targets exit the selected block. In this layout, branching to
        // `finally_frame` would therefore skip the finalizer itself. The
        // catch-skip block instead ends immediately before the finalizer, so
        // it is the continuation target for abrupt completions from either
        // the try or catch body.
        self.finally_stack.push(catch_skip_frame);
        self.push_scope();
        self.compile_block_contents(try_block, function)?;
        self.pop_scope();
        self.throw_handler_stack.pop();
        self.emit_branch_to_target(catch_skip_frame, function);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

        self.push_scope();
        if let Some(environment) = catch_parameter_environment {
            self.emit_enter_lexical_environment(environment, function)?;
        }
        let catch_storage = self
            .lookup_current_scope_binding(catch_name)
            .unwrap_or_else(|| self.allocate_dynamic_binding_storage(catch_name));
        self.binding_scopes
            .last_mut()
            .expect("binding scope stack must exist")
            .insert(catch_name.to_string(), catch_storage);
        if catch_source_name != catch_name {
            self.binding_scopes
                .last_mut()
                .expect("binding scope stack must exist")
                .insert(catch_source_name.to_string(), catch_storage);
        }
        self.write_binding_from_locals(
            catch_storage,
            self.result_local,
            self.result_tag_local,
            function,
        );
        self.set_completion_kind(CompletionKind::Normal, function);
        self.push_scope();
        self.compile_block_contents(catch_block, function)?;
        self.pop_scope();
        if catch_parameter_environment.is_some() {
            self.emit_leave_lexical_environment(function);
        }
        self.pop_scope();
        self.finally_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

        self.save_current_completion(
            saved_payload_local,
            saved_tag_local,
            saved_completion_local,
            saved_aux_local,
            function,
        );
        self.set_completion_kind(CompletionKind::Normal, function);
        self.push_scope();
        self.compile_block_contents(finally_block, function)?;
        self.pop_scope();
        self.emit_resume_after_finally(
            saved_payload_local,
            saved_tag_local,
            saved_completion_local,
            saved_aux_local,
            function,
        )?;
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.release_temp_local(saved_aux_local);
        self.release_temp_local(saved_completion_local);
        self.release_temp_local(saved_tag_local);
        self.release_temp_local(saved_payload_local);
        Ok(())
    }

    pub(crate) fn compile_while(
        &mut self,
        condition: &TypedExpr,
        body: &StatementIr,
        labels: &[String],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_statement_result(function, ValueKind::Undefined);
        let break_frame = self.open_frame(ControlFrameKind::Block, function);
        self.breakable_stack.push(break_frame);
        let continue_frame = self.open_frame(ControlFrameKind::Loop, function);
        self.loop_stack.push(LoopTargets { continue_frame });
        self.push_labels(labels, break_frame, Some(continue_frame));
        self.compile_truthy_i32(condition, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.branch_if_to_label(break_frame.label);
        self.compile_statement(body, function)?;
        function.branch_to_label(continue_frame.label);
        self.pop_labels(labels.len());
        self.loop_stack.pop();
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.breakable_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn compile_do_while(
        &mut self,
        body: &StatementIr,
        condition: &TypedExpr,
        labels: &[String],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_statement_result(function, ValueKind::Undefined);
        let break_frame = self.open_frame(ControlFrameKind::Block, function);
        self.breakable_stack.push(break_frame);
        let loop_frame = self.open_frame(ControlFrameKind::Loop, function);
        let continue_frame = self.open_frame(ControlFrameKind::Block, function);
        self.loop_stack.push(LoopTargets { continue_frame });
        self.push_labels(labels, break_frame, Some(continue_frame));
        self.compile_statement(body, function)?;
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.compile_truthy_i32(condition, function)?;
        function.branch_if_to_label(loop_frame.label);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.breakable_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn compile_for(
        &mut self,
        init: Option<&ForInitIr>,
        test: Option<&TypedExpr>,
        update: Option<&TypedExpr>,
        body: &StatementIr,
        lexical_environment: Option<&ForLexicalEnvironmentIr>,
        labels: &[String],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.push_scope();
        self.emit_statement_result(function, ValueKind::Undefined);
        let break_frame = self.open_frame(ControlFrameKind::Block, function);
        self.breakable_stack.push(break_frame);
        let runtime_environment = lexical_environment.map(|environment| LexicalEnvironmentIr {
            bindings: environment.bindings.clone(),
        });
        if let Some(environment) = &runtime_environment {
            self.emit_enter_lexical_environment(environment, function)?;
        }
        if let Some(init) = init {
            self.compile_for_init(init, function)?;
        }
        if let Some(environment) = lexical_environment {
            self.emit_replace_lexical_environment(environment, function)?;
        }
        let loop_frame = self.open_frame(ControlFrameKind::Loop, function);
        if let Some(test) = test {
            self.compile_truthy_i32(test, function)?;
            function.instruction(&Instruction::I32Eqz);
            self.emit_branch_if_to_target(break_frame, function);
        }
        let continue_frame = self.open_frame(ControlFrameKind::Block, function);
        self.loop_stack.push(LoopTargets { continue_frame });
        self.push_labels(labels, break_frame, Some(continue_frame));
        self.compile_statement(body, function)?;
        self.pop_labels(labels.len());
        self.loop_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        if let Some(environment) = lexical_environment {
            self.emit_replace_lexical_environment(environment, function)?;
        }
        if let Some(update) = update {
            self.compile_expr_payload(update, function)?;
            function.instruction(&Instruction::Drop);
        }
        function.branch_to_label(loop_frame.label);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.breakable_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        if runtime_environment.is_some() {
            self.end_lexical_environment_scope();
        }
        self.pop_scope();
        Ok(())
    }

    pub(crate) fn compile_switch(
        &mut self,
        discriminant: &TypedExpr,
        lexical_environment: Option<&LexicalEnvironmentIr>,
        lexical_declarations: &[StatementIr],
        cases: &[SwitchCaseIr],
        labels: &[String],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let discriminant_payload_local = self.reserve_temp_local();
        let discriminant_tag_local = self.reserve_temp_local();
        let selected_local = self.reserve_temp_local();
        let active_local = self.reserve_temp_local();
        let default_index = cases
            .iter()
            .enumerate()
            .find_map(|(index, case)| case.condition.is_none().then_some(index as i64));

        self.emit_statement_result(function, ValueKind::Undefined);
        self.compile_expr_to_locals(
            discriminant,
            discriminant_payload_local,
            discriminant_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            discriminant_payload_local,
            discriminant_tag_local,
            function,
        )?;
        self.push_scope();
        if let Some(environment) = lexical_environment {
            self.emit_enter_lexical_environment(environment, function)?;
        }
        for case in cases {
            self.initialize_direct_lexical_bindings(&case.body.statements, function);
        }
        for declaration in lexical_declarations {
            self.compile_statement(declaration, function)?;
        }
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(selected_local));

        for (index, case) in cases.iter().enumerate() {
            let Some(condition) = &case.condition else {
                continue;
            };
            function.instruction(&Instruction::LocalGet(selected_local));
            function.instruction(&Instruction::I64Const(-1));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            self.compile_switch_case_match(
                discriminant,
                discriminant_payload_local,
                discriminant_tag_local,
                condition,
                function,
            )?;
            self.emit_propagate_throw_from_locals_if_needed(
                self.scratch_local,
                self.result_tag_local,
                function,
            )?;
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(index as i64));
            function.instruction(&Instruction::LocalSet(selected_local));
            function.instruction(&Instruction::End);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }

        if let Some(default_index) = default_index {
            function.instruction(&Instruction::LocalGet(selected_local));
            function.instruction(&Instruction::I64Const(-1));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(default_index));
            function.instruction(&Instruction::LocalSet(selected_local));
            function.instruction(&Instruction::End);
        }

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(active_local));
        let break_frame = self.open_frame(ControlFrameKind::Block, function);
        self.breakable_stack.push(break_frame);
        for (index, case) in cases.iter().enumerate() {
            function.instruction(&Instruction::LocalGet(active_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(selected_local));
            function.instruction(&Instruction::I64Const(index as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(active_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalGet(active_local));
            function.instruction(&Instruction::I32WrapI64);
            self.open_frame(ControlFrameKind::If, function);
            self.compile_switch_case_body(&case.body, function)?;
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        self.pop_labels(labels.len());
        self.breakable_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        if lexical_environment.is_some() {
            self.emit_leave_lexical_environment(function);
        }
        self.pop_scope();
        self.release_temp_local(active_local);
        self.release_temp_local(selected_local);
        self.release_temp_local(discriminant_tag_local);
        self.release_temp_local(discriminant_payload_local);
        Ok(())
    }

    pub(crate) fn compile_switch_case_body(
        &mut self,
        block: &BlockIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        for statement in &block.statements {
            self.compile_statement(statement, function)?;
        }
        Ok(())
    }

    pub(crate) fn compile_switch_case_match(
        &mut self,
        discriminant: &TypedExpr,
        discriminant_payload_local: u32,
        discriminant_tag_local: u32,
        condition: &TypedExpr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if discriminant.kind != ValueKind::Dynamic
            && condition.kind != ValueKind::Dynamic
            && discriminant.kind != condition.kind
        {
            self.compile_expr_to_locals(
                condition,
                self.scratch_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_tagged_payload_equality_i32(
                discriminant_tag_local,
                discriminant_payload_local,
                self.result_tag_local,
                self.scratch_local,
                function,
            )?;
            return Ok(());
        }

        if discriminant.kind != ValueKind::Dynamic && condition.kind != ValueKind::Dynamic {
            self.compile_expr_to_locals(
                condition,
                self.scratch_local,
                self.result_tag_local,
                function,
            )?;
            match discriminant.kind {
                ValueKind::Number => {
                    function.instruction(&Instruction::LocalGet(discriminant_payload_local));
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::LocalGet(self.scratch_local));
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::F64Eq);
                }
                ValueKind::String => {
                    self.emit_string_payload_equality_i32(
                        discriminant_payload_local,
                        self.scratch_local,
                        function,
                    );
                }
                _ => {
                    function.instruction(&Instruction::LocalGet(discriminant_payload_local));
                    function.instruction(&Instruction::LocalGet(self.scratch_local));
                    function.instruction(&Instruction::I64Eq);
                }
            }
            return Ok(());
        }

        self.compile_expr_to_locals(
            condition,
            self.scratch_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_tagged_payload_equality_i32(
            discriminant_tag_local,
            discriminant_payload_local,
            self.result_tag_local,
            self.scratch_local,
            function,
        )?;
        Ok(())
    }

    pub(crate) fn compile_break(
        &mut self,
        label: Option<&str>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let break_frame = if let Some(label) = label {
            self.label_stack
                .iter()
                .rev()
                .find(|targets| targets.name == label)
                .map(|targets| targets.break_frame)
                .ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in lila wasm-aot first slice: unknown label `{label}`"
                    ))
                })?
        } else {
            *self.breakable_stack.last().ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: break outside loop or switch",
                )
            })?
        };
        if let Some(target) = self.active_finally_target_for_branch(break_frame) {
            self.set_completion_kind_with_aux(
                CompletionKind::Break,
                break_frame.frame as i64,
                function,
            );
            self.emit_branch_to_target(target, function);
            return Ok(());
        }
        self.emit_branch_to_target(break_frame, function);
        Ok(())
    }

    pub(crate) fn compile_continue(
        &mut self,
        label: Option<&str>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let continue_frame = if let Some(label) = label {
            self.label_stack
                .iter()
                .rev()
                .find(|targets| targets.name == label)
                .and_then(|targets| targets.continue_frame)
                .ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in lila wasm-aot first slice: continue to non-loop label `{label}`"
                    ))
                })?
        } else {
            self.loop_stack
                .last()
                .copied()
                .ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in lila wasm-aot first slice: continue outside loop",
                    )
                })?
                .continue_frame
        };
        if let Some(target) = self.active_finally_target_for_branch(continue_frame) {
            self.set_completion_kind_with_aux(
                CompletionKind::Continue,
                continue_frame.frame as i64,
                function,
            );
            self.emit_branch_to_target(target, function);
            return Ok(());
        }
        self.emit_branch_to_target(continue_frame, function);
        Ok(())
    }

    pub(crate) fn compile_for_init(
        &mut self,
        init: &ForInitIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match init {
            ForInitIr::Lexical { mode, name, init } => {
                let storage = self
                    .lookup_current_scope_binding(name)
                    .unwrap_or_else(|| self.allocate_binding(name.clone(), *mode, init.kind));
                self.initialize_binding_uninitialized(storage, function);
                self.compile_expr_to_binding(init, storage, function)?;
            }
            ForInitIr::LexicalBlock(bindings) => {
                for binding in bindings {
                    let storage = self
                        .lookup_current_scope_binding(&binding.name)
                        .unwrap_or_else(|| {
                            self.allocate_binding(
                                binding.name.clone(),
                                binding.mode,
                                binding.init.kind,
                            )
                        });
                    self.initialize_binding_uninitialized(storage, function);
                }
                for binding in bindings {
                    let storage = self
                        .lookup_current_scope_binding(&binding.name)
                        .expect("for lexical binding should be allocated");
                    self.compile_expr_to_binding(&binding.init, storage, function)?;
                }
            }
            ForInitIr::Var(declarators) => {
                self.compile_var_declarators(declarators, function)?;
            }
            ForInitIr::Expression(expr) => {
                self.compile_expr_payload(expr, function)?;
                function.instruction(&Instruction::Drop);
            }
            ForInitIr::Statements(statements) => {
                // Compiled directly in the loop's scope - `compile_for` has
                // already pushed it - so a pattern head's bindings stay visible
                // to the test, update and body.
                for statement in statements {
                    self.compile_statement(statement, function)?;
                }
            }
        }
        Ok(())
    }

    pub(crate) fn compile_var_declarators(
        &mut self,
        declarators: &[VarDeclaratorIr],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        for declarator in declarators {
            let Some(init) = &declarator.init else {
                continue;
            };
            let storage = self.lookup_binding(&declarator.name);
            if self.is_script_global_binding(&declarator.name) && storage.is_none() {
                let value_local = self.reserve_temp_local();
                let tag_local = self.reserve_temp_local();
                self.compile_expr_to_locals(init, value_local, tag_local, function)?;
                self.emit_propagate_throw_from_locals_if_needed(value_local, tag_local, function)?;
                self.emit_global_property_write(
                    &declarator.name,
                    value_local,
                    tag_local,
                    function,
                )?;
                self.release_temp_local(tag_local);
                self.release_temp_local(value_local);
            } else {
                let storage = storage.ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in lila wasm-aot first slice: unbound identifier `{}`",
                        declarator.name
                    ))
                })?;
                self.compile_expr_to_binding(init, storage, function)?;
                self.mirror_binding_to_global_object(&declarator.name, storage, function)?;
            }
        }
        Ok(())
    }

    pub(crate) fn emit_to_integer_clamped_to_string_len(
        &mut self,
        number_payload_local: u32,
        string_len_local: u32,
        out_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(out_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::LocalSet(out_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NEG_INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(out_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::I64TruncF64S);
        function.instruction(&Instruction::LocalSet(out_local));
        function.instruction(&Instruction::LocalGet(out_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(out_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(out_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::LocalSet(out_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    pub(crate) fn emit_to_slice_index_clamped_to_string_len(
        &mut self,
        number_payload_local: u32,
        string_len_local: u32,
        out_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(out_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::LocalSet(out_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NEG_INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(out_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::I64TruncF64S);
        function.instruction(&Instruction::LocalSet(out_local));
        function.instruction(&Instruction::LocalGet(out_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::LocalGet(out_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(out_local));
        function.instruction(&Instruction::LocalGet(out_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(out_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(out_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::LocalSet(out_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    pub(crate) fn compile_for_of_array(
        &mut self,
        mode: BindingMode,
        name: &str,
        iterable: &TypedExpr,
        body: &StatementIr,
        lexical_environment: Option<&ForInOfEnvironmentIr>,
        labels: &[String],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let array_local = self.reserve_temp_local();
        let array_tag_local = self.reserve_temp_local();
        let len_payload_local = self.reserve_temp_local();
        let len_tag_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();

        if let Some(environment) = lexical_environment {
            self.emit_enter_for_in_of_tdz_scope(mode, environment, function)?;
        }
        self.compile_expr_to_locals(iterable, array_local, array_tag_local, function)?;
        if let Some(environment) = lexical_environment {
            self.emit_leave_for_in_of_tdz_scope(environment, function);
        }

        self.push_scope();
        let storage_without_environment = if mode == BindingMode::Var {
            Some(self.lookup_binding(name).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in lila wasm-aot first slice: unbound for-of var `{name}`"
                ))
            })?)
        } else if !iteration_environment_owns_binding(lexical_environment, name) {
            Some(self.allocate_binding(name.to_string(), mode, ValueKind::Dynamic))
        } else {
            None
        };
        if mode == BindingMode::Var {
            self.binding_scopes
                .last_mut()
                .expect("binding scope stack must exist")
                .insert(
                    name.to_string(),
                    storage_without_environment.expect("for-of var storage must exist"),
                );
        }
        self.emit_array_length(array_local, len_payload_local, len_tag_local, function);
        function.instruction(&Instruction::LocalGet(len_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(len_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        self.emit_statement_result(function, ValueKind::Undefined);
        let break_frame = self.open_frame(ControlFrameKind::Block, function);
        self.breakable_stack.push(break_frame);
        let loop_frame = self.open_frame(ControlFrameKind::Loop, function);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_payload_local));
        function.instruction(&Instruction::I64GeU);
        function.branch_if_to_label(break_frame.label);
        self.emit_array_read(
            array_local,
            index_local,
            value_payload_local,
            value_tag_local,
            function,
        );
        if let Some(environment) =
            lexical_environment.and_then(|environment| environment.iteration_environment.as_ref())
        {
            self.emit_enter_lexical_environment(environment, function)?;
        }
        let storage = self
            .lookup_current_scope_binding(name)
            .or(storage_without_environment)
            .expect("for-of lexical storage must be allocated before assignment");
        self.write_binding_from_locals(storage, value_payload_local, value_tag_local, function);
        self.mirror_binding_to_global_object(name, storage, function)?;
        let continue_frame = self.open_frame(ControlFrameKind::Block, function);
        self.loop_stack.push(LoopTargets { continue_frame });
        self.push_labels(labels, break_frame, Some(continue_frame));
        self.compile_statement(body, function)?;
        self.pop_labels(labels.len());
        self.loop_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        if lexical_environment
            .and_then(|environment| environment.iteration_environment.as_ref())
            .is_some()
        {
            self.emit_leave_lexical_environment(function);
        }
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.branch_to_label(loop_frame.label);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.breakable_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.pop_scope();
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_tag_local);
        self.release_temp_local(len_payload_local);
        self.release_temp_local(array_tag_local);
        self.release_temp_local(array_local);
        Ok(())
    }

    pub(crate) fn compile_for_of_string(
        &mut self,
        mode: BindingMode,
        name: &str,
        iterable: &TypedExpr,
        body: &StatementIr,
        lexical_environment: Option<&ForInOfEnvironmentIr>,
        labels: &[String],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let string_payload_local = self.reserve_temp_local();
        let string_tag_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let byte_len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let codepoint_local = self.reserve_temp_local();
        let advance_local = self.reserve_temp_local();
        let temp_local = self.reserve_temp_local();
        let char_offset_local = self.reserve_temp_local();
        let char_pos_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();

        if let Some(environment) = lexical_environment {
            self.emit_enter_for_in_of_tdz_scope(mode, environment, function)?;
        }
        self.compile_expr_to_locals(iterable, string_payload_local, string_tag_local, function)?;
        if let Some(environment) = lexical_environment {
            self.emit_leave_for_in_of_tdz_scope(environment, function);
        }

        self.push_scope();
        let storage_without_environment = if mode == BindingMode::Var {
            Some(self.lookup_binding(name).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in lila wasm-aot first slice: unbound for-of var `{name}`"
                ))
            })?)
        } else if !iteration_environment_owns_binding(lexical_environment, name) {
            Some(self.allocate_binding(name.to_string(), mode, ValueKind::String))
        } else {
            None
        };
        if mode == BindingMode::Var {
            self.binding_scopes
                .last_mut()
                .expect("binding scope stack must exist")
                .insert(
                    name.to_string(),
                    storage_without_environment.expect("for-of var storage must exist"),
                );
        }
        function.instruction(&Instruction::LocalGet(string_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "for-of target is not iterable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_current_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.emit_unpack_string_payload(
            string_payload_local,
            buffer_local,
            byte_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        self.emit_statement_result(function, ValueKind::Undefined);
        let break_frame = self.open_frame(ControlFrameKind::Block, function);
        self.breakable_stack.push(break_frame);
        let loop_frame = self.open_frame(ControlFrameKind::Loop, function);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(byte_len_local));
        function.instruction(&Instruction::I64GeU);
        function.branch_if_to_label(break_frame.label);
        self.emit_load_string_byte(buffer_local, index_local, byte_local, function);
        self.emit_decode_utf8_scalar_at_index(
            buffer_local,
            index_local,
            byte_len_local,
            byte_local,
            codepoint_local,
            advance_local,
            temp_local,
            function,
        );
        self.emit_heap_alloc_const(4, function)?;
        function.instruction(&Instruction::LocalSet(char_offset_local));
        function.instruction(&Instruction::LocalGet(char_offset_local));
        function.instruction(&Instruction::LocalSet(char_pos_local));
        self.emit_store_utf8_codepoint(char_pos_local, codepoint_local, temp_local, function);
        function.instruction(&Instruction::LocalGet(char_pos_local));
        function.instruction(&Instruction::LocalGet(char_offset_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(byte_local));
        self.emit_pack_string_payload(char_offset_local, byte_local, function);
        function.instruction(&Instruction::LocalSet(value_payload_local));
        if let Some(environment) =
            lexical_environment.and_then(|environment| environment.iteration_environment.as_ref())
        {
            self.emit_enter_lexical_environment(environment, function)?;
        }
        let storage = self
            .lookup_current_scope_binding(name)
            .or(storage_without_environment)
            .expect("for-of lexical storage must be allocated before assignment");
        self.write_binding_from_locals(storage, value_payload_local, string_tag_local, function);
        self.mirror_binding_to_global_object(name, storage, function)?;
        let continue_frame = self.open_frame(ControlFrameKind::Block, function);
        self.loop_stack.push(LoopTargets { continue_frame });
        self.push_labels(labels, break_frame, Some(continue_frame));
        self.compile_statement(body, function)?;
        self.pop_labels(labels.len());
        self.loop_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        if lexical_environment
            .and_then(|environment| environment.iteration_environment.as_ref())
            .is_some()
        {
            self.emit_leave_lexical_environment(function);
        }
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.branch_to_label(loop_frame.label);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.breakable_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.pop_scope();
        self.release_temp_local(value_payload_local);
        self.release_temp_local(char_pos_local);
        self.release_temp_local(char_offset_local);
        self.release_temp_local(temp_local);
        self.release_temp_local(advance_local);
        self.release_temp_local(codepoint_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(index_local);
        self.release_temp_local(byte_len_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(string_tag_local);
        self.release_temp_local(string_payload_local);
        Ok(())
    }

    /// Read a well-known-symbol method off a `for await (… of …)` head value.
    ///
    /// `PropertyKeyIr::StaticString("Symbol.asyncIterator")` is *not* the
    /// well-known symbol: `compile_object_key_to_locals` lowers a static string
    /// key to `strings.payload(name)` tagged `String`, so it looks up the
    /// ordinary string property `"Symbol.asyncIterator"` and always misses.
    /// A symbol key has to carry `PROPERTY_KEY_SYMBOL_MARKER`, which the
    /// `StringExpr` path ORs in when the key expression is `Symbol`-kinded.
    /// This mirrors `emit_generator_delegate_property_read`, the `yield*`
    /// equivalent, and keeps primitive receivers (strings, numbers) working by
    /// going through the dynamic read.
    fn emit_for_await_well_known_symbol_read(
        &mut self,
        key: &str,
        target_payload_local: u32,
        target_tag_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        debug_assert!(key.starts_with("Symbol."));
        let target = TypedExpr::from_info(
            ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::all_runtime_tags()
                    .without(ValueKind::Undefined)
                    .without(ValueKind::Null),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            },
            ExprIr::Undefined,
        );
        let symbol_key = TypedExpr::from_info(
            ValueInfo::new(ValueKind::Symbol),
            ExprIr::String(key.to_string()),
        );
        self.compile_property_read_from_locals(
            &target,
            &PropertyKeyIr::StringExpr(Box::new(symbol_key)),
            target_payload_local,
            target_tag_local,
            value_payload_local,
            value_tag_local,
            function,
        )
    }

    /// Leave `low <= state <= high` (unsigned) on the stack as an i32 boolean.
    ///
    /// The for-await emitter tests its plan states as spans rather than as
    /// individual equalities, because the states a suspension in the loop body
    /// resumes into are allocated *inside* the loop's span and are not known
    /// to the loop itself — only their bounds are.
    fn emit_state_in_inclusive_range_i32(
        state_local: u32,
        low: u32,
        high: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(i64::from(low)));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(i64::from(high)));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
    }

    /// Load the completion of a `for-await-of` suspension into one normalized
    /// i64 `is_throw` boolean.
    ///
    /// Ordinary async functions cross the strict two-word heap boundary.
    /// Async generators keep their existing rule that only the dedicated
    /// rejection kind turns an awaited result into a throw completion.
    fn emit_load_for_await_resume_is_throw(
        &mut self,
        layout: ForAwaitActivationLayout,
        activation_local: u32,
        is_throw_local: u32,
        function: &mut Function,
    ) {
        match layout {
            ForAwaitActivationLayout::AsyncFunction => self
                .emit_load_async_function_resume_is_throw(
                    activation_local,
                    is_throw_local,
                    function,
                ),
            ForAwaitActivationLayout::AsyncGenerator => {
                self.load_i64_to_local_from_offset(
                    activation_local,
                    HEAP_ASYNC_GENERATOR_RESUME_KIND_OFFSET,
                    is_throw_local,
                    function,
                );
                function.instruction(&Instruction::LocalGet(is_throw_local));
                function.instruction(&Instruction::I64Const(
                    ASYNC_GENERATOR_RESUME_KIND_REJECT as i64,
                ));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::LocalSet(is_throw_local));
            }
        }
    }

    pub(crate) fn compile_async_for_of_iterator(
        &mut self,
        mode: BindingMode,
        name: &str,
        iterable: &TypedExpr,
        body: &StatementIr,
        lexical_environment: Option<&ForInOfEnvironmentIr>,
        async_plan: &AsyncForOfIteratorPlanIr,
        labels: &[String],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let activation_local = self.new_target_payload_local().ok_or_else(|| {
            EmitError::unsupported("for-await-of requires the async function call ABI")
        })?;
        // The loop replays itself out of its plan states, and the two gates
        // below read the plan as a *span* rather than as a set of three
        // numbers: the loop is entered for any state in
        // `[entry_state, close_resume_state]`, and one iteration is resumed
        // for any state in `[value_resume_state, close_resume_state)`. The
        // half-open upper end is what carries a suspension inside the body:
        // `AsyncGeneratorSuspensionCollector::visit_for_of_loop`
        // (lila-ir/src/lowering_helpers.rs) suspends `ForAwaitNext`, then
        // visits the body — whose suspensions chain off `next()`'s resume
        // state — then `reserve()`s one state, then suspends `ForAwaitClose`,
        // so every state the body can resume into lies strictly between
        // `value_resume_state` and `close_resume_state`. A body with no
        // suspension is the degenerate case `entry, entry+1, entry+2,
        // entry+3`, where the span collapses back onto the three states the
        // loop owns itself.
        //
        // Both gates therefore depend on the plan being strictly ordered, and
        // an out-of-order plan is unobservable at compile time: it would
        // silently route a `next()` resume into the iterator-close path (or
        // skip the loop entirely). Refuse instead of guessing.
        if !(async_plan.entry_state < async_plan.value_resume_state
            && async_plan.value_resume_state < async_plan.close_resume_state
            && async_plan.close_resume_state < async_plan.exit_state)
        {
            return Err(EmitError::unsupported(format!(
                "for-await-of resume plan is not strictly ordered (entry {}, value {}, close {}, exit {})",
                async_plan.entry_state,
                async_plan.value_resume_state,
                async_plan.close_resume_state,
                async_plan.exit_state,
            )));
        }
        // Whether one iteration can be split across more than one wasm
        // invocation. This is the same predicate `compile_block_contents`
        // uses to decide that the body compiles as a resumable statement
        // sequence, so the gates below and the body's own dispatch cannot
        // disagree about which states exist.
        let body_suspends = Self::async_statement_entry_state(body).is_some();
        if body_suspends {
            // A per-iteration environment is entered at the loop head and left
            // after the body, both inside the same invocation. Split the
            // iteration and the resume would enter a second environment while
            // the first is still current, and leave only one of them.
            if lexical_environment
                .and_then(|environment| environment.iteration_environment.as_ref())
                .is_some()
            {
                return Err(EmitError::unsupported(
                    "for-await-of with a per-iteration lexical environment and a body suspension",
                ));
            }
            // `compile_async_block_contents` enters a body block's own
            // environment unconditionally, i.e. once per invocation, and
            // leaves it only on the invocation that runs the block to its end.
            if matches!(body, StatementIr::Block(block) if block.lexical_environment.is_some()) {
                return Err(EmitError::unsupported(
                    "for-await-of with a block-scoped body environment and a body suspension",
                ));
            }
        }
        let resume_layout = match self
            .current_function_meta()
            .map(|meta| meta.protocol.execution_kind())
        {
            Some(FunctionExecutionKind::Async) => ForAwaitActivationLayout::AsyncFunction,
            Some(FunctionExecutionKind::AsyncGenerator) => ForAwaitActivationLayout::AsyncGenerator,
            Some(FunctionExecutionKind::Ordinary | FunctionExecutionKind::Generator) | None => {
                return Err(EmitError::unsupported(
                    "for-await-of requires an async function or async-generator activation",
                ));
            }
        };
        let resume_state_offset = resume_layout.resume_state_offset();
        let resume_payload_offset = resume_layout.resume_payload_offset();
        let resume_tag_offset = resume_layout.resume_tag_offset();
        let is_async_generator = resume_layout.is_async_generator();
        let state_local = self.reserve_temp_local();
        let iterable_payload_local = self.reserve_temp_local();
        let iterable_tag_local = self.reserve_temp_local();
        let iterator_payload_local = self.reserve_temp_local();
        let iterator_tag_local = self.reserve_temp_local();
        let next_payload_local = self.reserve_temp_local();
        let next_tag_local = self.reserve_temp_local();
        let async_iterator_payload_local = self.reserve_temp_local();
        let async_iterator_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let method_payload_local = self.reserve_temp_local();
        let method_tag_local = self.reserve_temp_local();
        let close_get_method_aux_local = self.reserve_temp_local();
        let result_payload_local = self.reserve_temp_local();
        let result_tag_local = self.reserve_temp_local();
        let done_payload_local = self.reserve_temp_local();
        let done_tag_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let continuation_payload_local = self.reserve_temp_local();
        let continuation_tag_local = self.reserve_temp_local();
        let resume_is_throw_local = self.reserve_temp_local();
        let saved_payload_local = self.reserve_temp_local();
        let saved_tag_local = self.reserve_temp_local();
        let saved_completion_local = self.reserve_temp_local();
        let saved_aux_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            activation_local,
            resume_state_offset,
            state_local,
            function,
        );
        // Enter the loop for every state it owns *and* every state its body
        // owns. For a suspension-free body the span is exactly
        // `{entry, entry+1, entry+2}`, the three states the previous equality
        // test admitted; for a suspending body it additionally admits the body's
        // resume states, which is what stops a body resume from falling straight
        // through the loop and completing the generator.
        Self::emit_state_in_inclusive_range_i32(
            state_local,
            async_plan.entry_state,
            async_plan.close_resume_state,
            function,
        );
        self.open_frame(ControlFrameKind::If, function);

        self.push_scope();
        let storage_without_environment = if mode == BindingMode::Var {
            Some(self.lookup_binding(name).ok_or_else(|| {
                EmitError::unsupported(format!("unbound for-await-of var `{name}`"))
            })?)
        } else if !iteration_environment_owns_binding(lexical_environment, name) {
            Some(self.allocate_binding(name.to_string(), mode, ValueKind::Dynamic))
        } else {
            None
        };
        // The loop variable is written on the invocation that resumes from
        // `next()` and read by whatever the body does afterwards. Once the body
        // can suspend those stop being the same invocation, so the binding has
        // to live in an environment slot: wasm locals are reset on every
        // resume. `collect_owner_root_bindings_from_statement`
        // (lila-ir/src/analysis.rs) gives every for-of loop binding of a
        // resumable owner an activation slot, so this guards that invariant
        // rather than a shape callers are expected to write.
        if body_suspends
            && !matches!(
                storage_without_environment,
                Some(BindingStorage::EnvSlot { .. })
            )
        {
            return Err(EmitError::unsupported(format!(
                "for-await-of binding `{name}` does not survive a suspension in the loop body"
            )));
        }
        if mode == BindingMode::Var {
            self.binding_scopes
                .last_mut()
                .expect("binding scope stack must exist")
                .insert(
                    name.to_string(),
                    storage_without_environment.expect("for-await-of var storage must exist"),
                );
        }
        let iterator_storage = self.allocate_binding(
            async_plan.record.iterator().as_str().to_string(),
            BindingMode::Let,
            ValueKind::Object,
        );
        let next_storage = self.allocate_binding(
            async_plan.record.next_method().as_str().to_string(),
            BindingMode::Let,
            ValueKind::Dynamic,
        );
        let async_iterator_storage = self.allocate_binding(
            async_plan.async_iterator_binding.clone(),
            BindingMode::Let,
            ValueKind::Boolean,
        );
        let done_storage = self.allocate_binding(
            async_plan.record.done().as_str().to_string(),
            BindingMode::Let,
            ValueKind::Boolean,
        );
        let close_on_rejection_storage = self.allocate_binding(
            async_plan.close_on_rejection_binding.clone(),
            BindingMode::Let,
            ValueKind::Boolean,
        );

        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(async_plan.entry_state as i64));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        if let Some(environment) = lexical_environment {
            self.emit_enter_for_in_of_tdz_scope(mode, environment, function)?;
        }
        self.compile_expr_to_locals(
            iterable,
            iterable_payload_local,
            iterable_tag_local,
            function,
        )?;
        if let Some(environment) = lexical_environment {
            self.emit_leave_for_in_of_tdz_scope(environment, function);
        }
        let iterable_is_statically_nullish =
            matches!(iterable.kind, ValueKind::Undefined | ValueKind::Null);
        if iterable_is_statically_nullish {
            self.emit_throw_runtime_error(
                TYPE_ERROR_NAME,
                "for-await-of target is not iterable",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_propagate_current_throw(function);
        } else {
            self.emit_for_await_well_known_symbol_read(
                "Symbol.asyncIterator",
                iterable_payload_local,
                iterable_tag_local,
                method_payload_local,
                method_tag_local,
                function,
            )?;
        }
        self.emit_propagate_current_completion_if_throw(function);
        self.compile_nullish_tagged_i32(method_tag_local, function)?;
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(done_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(done_tag_local));
        if !iterable_is_statically_nullish {
            self.emit_for_await_well_known_symbol_read(
                "Symbol.iterator",
                iterable_payload_local,
                iterable_tag_local,
                method_payload_local,
                method_tag_local,
                function,
            )?;
        }
        self.emit_propagate_current_completion_if_throw(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(done_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(done_tag_local));
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.write_binding_from_locals(
            async_iterator_storage,
            done_payload_local,
            done_tag_local,
            function,
        );
        self.emit_is_callable_i32(method_tag_local, method_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "for-await-of iterator method must be callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_current_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_function_or_proxy_call_leave_throw_completion(
            method_payload_local,
            method_tag_local,
            iterable_payload_local,
            iterable_tag_local,
            &[],
            iterator_payload_local,
            iterator_tag_local,
            function,
        )?;
        self.emit_propagate_current_completion_if_throw(function);
        self.emit_is_heap_object_like_tag_i32(iterator_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "for-await-of iterator method must return object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_current_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(self.strings.payload("next")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            iterator_payload_local,
            iterator_tag_local,
            iterator_payload_local,
            iterator_tag_local,
            key_local,
            next_payload_local,
            next_tag_local,
            function,
        )?;
        self.emit_propagate_current_completion_if_throw(function);
        self.emit_is_callable_i32(next_tag_local, next_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "for-await-of iterator next must be callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_current_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.write_binding_from_locals(
            iterator_storage,
            iterator_payload_local,
            iterator_tag_local,
            function,
        );
        self.write_binding_from_locals(next_storage, next_payload_local, next_tag_local, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        let break_frame = self.open_frame(ControlFrameKind::Block, function);
        self.breakable_stack.push(break_frame);

        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(async_plan.close_resume_state as i64));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_load_for_await_resume_is_throw(
            resume_layout,
            activation_local,
            resume_is_throw_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            activation_local,
            resume_payload_offset,
            value_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            activation_local,
            resume_tag_offset,
            value_tag_local,
            function,
        );
        self.emit_pop_and_restore_async_pending_completion(function)?;
        self.store_i64_const_at_offset(
            activation_local,
            resume_state_offset,
            u64::from(async_plan.exit_state),
            function,
        );
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(resume_is_throw_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Throw, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.read_binding_to_locals(
            async_iterator_storage,
            async_iterator_payload_local,
            async_iterator_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(resume_is_throw_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(async_iterator_payload_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_is_heap_object_like_tag_i32(value_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "for-await-of async iterator return result must be object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_dispatch_current_completion(function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        // Everything from here to the close of this frame is one iteration:
        // unpack the awaited `next()` result, bind the loop variable, run the
        // body, then decide whether the iterator has to be closed. It is
        // entered for `value_resume_state` — the invocation that resumes from
        // `await next()` — and, when the body can suspend, for every state
        // between that and `close_resume_state`, which is where the collector
        // put the body's own resume states. Those are the invocations that
        // finish an iteration whose body was cut in half.
        if body_suspends {
            Self::emit_state_in_inclusive_range_i32(
                state_local,
                async_plan.value_resume_state,
                async_plan.close_resume_state - 1,
                function,
            );
        } else {
            function.instruction(&Instruction::LocalGet(state_local));
            function.instruction(&Instruction::I64Const(async_plan.value_resume_state as i64));
            function.instruction(&Instruction::I64Eq);
        }
        self.open_frame(ControlFrameKind::If, function);
        // Unpacking the result reads the activation's resume payload/tag/kind,
        // which describe the `await` that just settled. On a body resume they
        // describe the body's own suspension instead, and the iteration's
        // `value` was consumed an invocation ago, so this whole section runs
        // only on the invocation that came back from `next()`.
        if body_suspends {
            function.instruction(&Instruction::LocalGet(state_local));
            function.instruction(&Instruction::I64Const(async_plan.value_resume_state as i64));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
        }
        self.emit_load_for_await_resume_is_throw(
            resume_layout,
            activation_local,
            resume_is_throw_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            activation_local,
            resume_payload_offset,
            value_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            activation_local,
            resume_tag_offset,
            value_tag_local,
            function,
        );
        self.read_binding_to_locals(
            async_iterator_storage,
            async_iterator_payload_local,
            async_iterator_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(async_iterator_payload_local));
        function.instruction(&Instruction::I32WrapI64);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::LocalGet(resume_is_throw_local));
        function.instruction(&Instruction::I32WrapI64);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Throw, function);
        self.emit_dispatch_current_completion(function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_is_heap_object_like_tag_i32(value_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "for-await-of async iterator next result must be object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_dispatch_current_completion(function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(self.strings.payload("done")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            value_payload_local,
            value_tag_local,
            value_payload_local,
            value_tag_local,
            key_local,
            done_payload_local,
            done_tag_local,
            function,
        )?;
        self.emit_propagate_current_completion_if_throw(function);
        self.compile_truthy_tagged_i32(done_tag_local, done_payload_local, function)?;
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(done_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(done_tag_local));
        self.write_binding_from_locals(done_storage, done_payload_local, done_tag_local, function);
        function.instruction(&Instruction::LocalGet(done_payload_local));
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(self.strings.payload("value")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            value_payload_local,
            value_tag_local,
            value_payload_local,
            value_tag_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_propagate_current_completion_if_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        if body_suspends {
            // Close the `state == value_resume_state` gate around unpacking.
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        let continue_frame = self.open_frame(ControlFrameKind::Block, function);
        self.loop_stack.push(LoopTargets { continue_frame });
        self.push_labels(labels, break_frame, Some(continue_frame));
        let finally_frame = self.open_frame(ControlFrameKind::Block, function);
        self.finally_stack.push(finally_frame);
        // A sync iterator whose `next()` promise rejected reports the rejection
        // here rather than through the async-iterator path. It reads the resume
        // kind and the async-iterator flag, both of which only describe the
        // `next()` await, so on a body resume it is skipped rather than left to
        // be accidentally right about locals this invocation never assigned.
        if body_suspends {
            function.instruction(&Instruction::LocalGet(state_local));
            function.instruction(&Instruction::I64Const(async_plan.value_resume_state as i64));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
        }
        function.instruction(&Instruction::LocalGet(resume_is_throw_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(async_iterator_payload_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Throw, function);
        self.read_binding_to_locals(
            close_on_rejection_storage,
            method_payload_local,
            method_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(method_payload_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        let outer_finalizer = self.finally_stack.iter().rev().nth(1).copied();
        let surrounding_throw_target =
            match (self.throw_handler_stack.last().copied(), outer_finalizer) {
                (Some(handler), Some(finalizer)) => Some(innermost_target(handler, finalizer)),
                (Some(handler), None) => Some(handler),
                (None, Some(finalizer)) => Some(finalizer),
                (None, None) => None,
            };
        if let Some(target) = surrounding_throw_target {
            self.emit_branch_to_target(target, function);
        } else {
            self.emit_return_current_completion(function);
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_dispatch_current_completion(function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        if body_suspends {
            // Close the `state == value_resume_state` gate around the
            // sync-iterator rejection pre-check.
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        // `done` is read from the loop's own suspension-owned binding, not from
        // a local, so this test is meaningful on a body resume too: the
        // `next()` that produced the in-flight iteration wrote `false` there,
        // and the iteration is not finished, so the loop correctly does not
        // break out from under a half-run body.
        self.read_binding_to_locals(done_storage, done_payload_local, done_tag_local, function)?;
        function.instruction(&Instruction::LocalGet(done_payload_local));
        function.instruction(&Instruction::I32WrapI64);
        function.branch_if_to_label(break_frame.label);
        // Binding the loop variable belongs to the start of an iteration. On a
        // body resume the iteration is already under way, the value locals hold
        // nothing this invocation assigned, and the binding still holds the
        // value the body was suspended with — so rebinding would overwrite it
        // with garbage. A per-iteration environment is refused above when the
        // body can suspend, so entering one here stays a value-path-only step.
        if body_suspends {
            function.instruction(&Instruction::LocalGet(state_local));
            function.instruction(&Instruction::I64Const(async_plan.value_resume_state as i64));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
        }
        if let Some(environment) =
            lexical_environment.and_then(|environment| environment.iteration_environment.as_ref())
        {
            self.emit_enter_lexical_environment(environment, function)?;
        }
        let storage = self
            .lookup_current_scope_binding(name)
            .or(storage_without_environment)
            .expect("for-await-of lexical storage must exist");
        self.write_binding_from_locals(storage, value_payload_local, value_tag_local, function);
        self.mirror_binding_to_global_object(name, storage, function)?;
        if body_suspends {
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        // Emitted at the same control depth as before the gates above, so every
        // `break`/`continue`/`return` inside the body still resolves to the same
        // frame it did when a suspending body was refused outright.
        self.compile_statement(body, function)?;
        self.finally_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.pop_labels(labels.len());
        self.loop_stack.pop();
        self.save_current_completion(
            saved_payload_local,
            saved_tag_local,
            saved_completion_local,
            saved_aux_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(saved_completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_CONTINUE));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(saved_aux_local));
        function.instruction(&Instruction::I64Const(continue_frame.frame as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_NORMAL));
        function.instruction(&Instruction::LocalSet(saved_completion_local));
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        // Pairs with the per-iteration enter at the loop head, and reads
        // `resume_is_throw_local`. Both are sound only while an iteration begins
        // and ends inside one invocation, which is why a per-iteration
        // environment and a suspending body are refused together above.
        if lexical_environment
            .and_then(|environment| environment.iteration_environment.as_ref())
            .is_some()
        {
            debug_assert!(
                !body_suspends,
                "a per-iteration environment and a body suspension must have been refused"
            );
            function.instruction(&Instruction::LocalGet(resume_is_throw_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_leave_lexical_environment(function);
            function.instruction(&Instruction::End);
        }
        self.emit_iterator_close_condition_i32(
            saved_completion_local,
            saved_aux_local,
            continue_frame,
            function,
        );
        self.open_frame(ControlFrameKind::If, function);
        self.read_binding_to_locals(
            iterator_storage,
            iterator_payload_local,
            iterator_tag_local,
            function,
        )?;
        self.read_binding_to_locals(
            async_iterator_storage,
            async_iterator_payload_local,
            async_iterator_tag_local,
            function,
        )?;
        self.restore_saved_completion(
            saved_payload_local,
            saved_tag_local,
            saved_completion_local,
            saved_aux_local,
            function,
        );
        self.emit_push_async_pending_completion(function)?;
        self.set_completion_kind(CompletionKind::Normal, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("return")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read_without_throw_propagation(
            iterator_payload_local,
            iterator_tag_local,
            iterator_payload_local,
            iterator_tag_local,
            key_local,
            method_payload_local,
            method_tag_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_NORMAL));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::LocalGet(method_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(method_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_is_callable_i32(method_tag_local, method_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "for-await-of iterator return must be callable",
            method_payload_local,
            method_tag_local,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::LocalGet(self.completion_aux_local));
        function.instruction(&Instruction::LocalSet(close_get_method_aux_local));
        self.emit_pop_and_restore_async_pending_completion(function)?;
        self.store_i64_const_at_offset(
            activation_local,
            resume_state_offset,
            u64::from(async_plan.exit_state),
            function,
        );
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Ne);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::LocalGet(method_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(method_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::LocalSet(self.completion_local));
        function.instruction(&Instruction::LocalGet(close_get_method_aux_local));
        function.instruction(&Instruction::LocalSet(self.completion_aux_local));
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_dispatch_current_completion(function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_NORMAL));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::LocalGet(method_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(method_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_NORMAL));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::LocalGet(method_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_function_handle_call_without_throw_propagation(
            method_payload_local,
            method_tag_local,
            Some((iterator_payload_local, Some(iterator_tag_local))),
            &[],
            result_payload_local,
            result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_function_or_proxy_call_leave_throw_completion(
            method_payload_local,
            method_tag_local,
            iterator_payload_local,
            iterator_tag_local,
            &[],
            result_payload_local,
            result_tag_local,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(async_iterator_payload_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_NORMAL));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_is_heap_object_like_tag_i32(result_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "for-await-of iterator return result must be object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_NORMAL));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(self.strings.payload("done")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read_without_throw_propagation(
            result_payload_local,
            result_tag_local,
            result_payload_local,
            result_tag_local,
            key_local,
            done_payload_local,
            done_tag_local,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_NORMAL));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(self.strings.payload("value")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read_without_throw_propagation(
            result_payload_local,
            result_tag_local,
            result_payload_local,
            result_tag_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        let rejected_promise_record_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(PROMISE_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_alloc_promise_with_prototype(
            self.scratch_local,
            continuation_payload_local,
            rejected_promise_record_local,
            function,
        )?;
        self.emit_settle_promise_record(
            rejected_promise_record_local,
            PromiseSettlement::Reject,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(continuation_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);
        self.release_temp_local(rejected_promise_record_local);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(async_iterator_payload_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::LocalSet(continuation_payload_local));
        function.instruction(&Instruction::LocalGet(result_tag_local));
        function.instruction(&Instruction::LocalSet(continuation_tag_local));
        function.instruction(&Instruction::Else);
        self.emit_async_from_sync_value_continuation(
            value_payload_local,
            value_tag_local,
            continuation_payload_local,
            continuation_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.store_i64_const_at_offset(
            activation_local,
            resume_state_offset,
            u64::from(async_plan.close_resume_state),
            function,
        );
        if is_async_generator {
            self.emit_async_generator_await_reactions(
                activation_local,
                continuation_payload_local,
                continuation_tag_local,
                function,
            )?;
            self.store_i64_const_at_offset(
                activation_local,
                HEAP_ASYNC_GENERATOR_BODY_STATUS_OFFSET,
                ASYNC_GENERATOR_BODY_STATUS_AWAIT,
                function,
            );
            self.store_i64_const_at_offset(
                activation_local,
                HEAP_ASYNC_GENERATOR_EXECUTION_STATE_OFFSET,
                ASYNC_GENERATOR_STATE_SUSPENDED_AWAIT,
                function,
            );
        } else {
            self.emit_async_await_reactions(
                activation_local,
                continuation_payload_local,
                continuation_tag_local,
                function,
            )?;
        }
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind_with_aux(
            CompletionKind::Normal,
            i64::from(async_plan.close_resume_state),
            function,
        );
        self.emit_return_current_completion(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.restore_saved_completion(
            saved_payload_local,
            saved_tag_local,
            saved_completion_local,
            saved_aux_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_NORMAL));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_dispatch_current_completion(function)?;
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.read_binding_to_locals(
            iterator_storage,
            iterator_payload_local,
            iterator_tag_local,
            function,
        )?;
        self.read_binding_to_locals(next_storage, next_payload_local, next_tag_local, function)?;
        self.read_binding_to_locals(
            async_iterator_storage,
            async_iterator_payload_local,
            async_iterator_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(done_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(done_tag_local));
        self.write_binding_from_locals(done_storage, done_payload_local, done_tag_local, function);
        self.write_binding_from_locals(
            close_on_rejection_storage,
            done_payload_local,
            done_tag_local,
            function,
        );
        self.emit_function_or_proxy_call_leave_throw_completion(
            next_payload_local,
            next_tag_local,
            iterator_payload_local,
            iterator_tag_local,
            &[],
            result_payload_local,
            result_tag_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(async_iterator_payload_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_dispatch_current_completion(function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(async_iterator_payload_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_NORMAL));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_is_heap_object_like_tag_i32(result_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "for-await-of iterator next result must be object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_NORMAL));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(self.strings.payload("done")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read_without_throw_propagation(
            result_payload_local,
            result_tag_local,
            result_payload_local,
            result_tag_local,
            key_local,
            done_payload_local,
            done_tag_local,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_NORMAL));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.compile_truthy_tagged_i32(done_tag_local, done_payload_local, function)?;
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(done_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(done_tag_local));
        self.write_binding_from_locals(done_storage, done_payload_local, done_tag_local, function);
        function.instruction(&Instruction::I64Const(self.strings.payload("value")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read_without_throw_propagation(
            result_payload_local,
            result_tag_local,
            result_payload_local,
            result_tag_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_NORMAL));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::LocalGet(done_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(method_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(method_tag_local));
        self.write_binding_from_locals(
            close_on_rejection_storage,
            method_payload_local,
            method_tag_local,
            function,
        );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        let rejected_promise_record_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(PROMISE_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_alloc_promise_with_prototype(
            self.scratch_local,
            continuation_payload_local,
            rejected_promise_record_local,
            function,
        )?;
        self.emit_settle_promise_record(
            rejected_promise_record_local,
            PromiseSettlement::Reject,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(continuation_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);
        self.release_temp_local(rejected_promise_record_local);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(async_iterator_payload_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::LocalSet(continuation_payload_local));
        function.instruction(&Instruction::LocalGet(result_tag_local));
        function.instruction(&Instruction::LocalSet(continuation_tag_local));
        function.instruction(&Instruction::Else);
        self.emit_async_from_sync_value_continuation(
            value_payload_local,
            value_tag_local,
            continuation_payload_local,
            continuation_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.store_i64_const_at_offset(
            activation_local,
            resume_state_offset,
            u64::from(async_plan.value_resume_state),
            function,
        );
        if is_async_generator {
            self.emit_async_generator_await_reactions(
                activation_local,
                continuation_payload_local,
                continuation_tag_local,
                function,
            )?;
            self.store_i64_const_at_offset(
                activation_local,
                HEAP_ASYNC_GENERATOR_BODY_STATUS_OFFSET,
                ASYNC_GENERATOR_BODY_STATUS_AWAIT,
                function,
            );
            self.store_i64_const_at_offset(
                activation_local,
                HEAP_ASYNC_GENERATOR_EXECUTION_STATE_OFFSET,
                ASYNC_GENERATOR_STATE_SUSPENDED_AWAIT,
                function,
            );
        } else {
            self.emit_async_await_reactions(
                activation_local,
                continuation_payload_local,
                continuation_tag_local,
                function,
            )?;
        }
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind_with_aux(
            CompletionKind::Normal,
            i64::from(async_plan.value_resume_state),
            function,
        );
        self.emit_return_current_completion(function);

        self.breakable_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.store_i64_const_at_offset(
            activation_local,
            resume_state_offset,
            u64::from(async_plan.exit_state),
            function,
        );
        self.emit_statement_result(function, ValueKind::Undefined);
        self.pop_scope();
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.release_temp_local(saved_aux_local);
        self.release_temp_local(saved_completion_local);
        self.release_temp_local(saved_tag_local);
        self.release_temp_local(saved_payload_local);
        self.release_temp_local(resume_is_throw_local);
        self.release_temp_local(continuation_tag_local);
        self.release_temp_local(continuation_payload_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(done_tag_local);
        self.release_temp_local(done_payload_local);
        self.release_temp_local(result_tag_local);
        self.release_temp_local(result_payload_local);
        self.release_temp_local(close_get_method_aux_local);
        self.release_temp_local(method_tag_local);
        self.release_temp_local(method_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(async_iterator_tag_local);
        self.release_temp_local(async_iterator_payload_local);
        self.release_temp_local(next_tag_local);
        self.release_temp_local(next_payload_local);
        self.release_temp_local(iterator_tag_local);
        self.release_temp_local(iterator_payload_local);
        self.release_temp_local(iterable_tag_local);
        self.release_temp_local(iterable_payload_local);
        self.release_temp_local(state_local);
        Ok(())
    }

    pub(crate) fn compile_for_of_iterator(
        &mut self,
        mode: BindingMode,
        name: &str,
        iterable: &TypedExpr,
        body: &StatementIr,
        lexical_environment: Option<&ForInOfEnvironmentIr>,
        labels: &[String],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let iterable_payload_local = self.reserve_temp_local();
        let iterable_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let method_payload_local = self.reserve_temp_local();
        let method_tag_local = self.reserve_temp_local();
        let iterator_payload_local = self.reserve_temp_local();
        let iterator_tag_local = self.reserve_temp_local();
        let next_payload_local = self.reserve_temp_local();
        let next_tag_local = self.reserve_temp_local();
        let result_payload_local = self.reserve_temp_local();
        let result_tag_local = self.reserve_temp_local();
        let done_payload_local = self.reserve_temp_local();
        let done_tag_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let saved_payload_local = self.reserve_temp_local();
        let saved_tag_local = self.reserve_temp_local();
        let saved_completion_local = self.reserve_temp_local();
        let saved_aux_local = self.reserve_temp_local();
        let close_saved_payload_local = self.reserve_temp_local();
        let close_saved_tag_local = self.reserve_temp_local();
        let close_saved_completion_local = self.reserve_temp_local();
        let close_saved_aux_local = self.reserve_temp_local();

        if let Some(environment) = lexical_environment {
            self.emit_enter_for_in_of_tdz_scope(mode, environment, function)?;
        }
        self.compile_expr_to_locals(
            iterable,
            iterable_payload_local,
            iterable_tag_local,
            function,
        )?;
        if let Some(environment) = lexical_environment {
            self.emit_leave_for_in_of_tdz_scope(environment, function);
        }

        self.push_scope();
        let storage_without_environment = if mode == BindingMode::Var {
            Some(self.lookup_binding(name).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in lila wasm-aot first slice: unbound for-of var `{name}`"
                ))
            })?)
        } else if !iteration_environment_owns_binding(lexical_environment, name) {
            Some(self.allocate_binding(name.to_string(), mode, ValueKind::Dynamic))
        } else {
            None
        };
        if mode == BindingMode::Var {
            self.binding_scopes
                .last_mut()
                .expect("binding scope stack must exist")
                .insert(
                    name.to_string(),
                    storage_without_environment.expect("for-of var storage must exist"),
                );
        }
        // GetIterator(obj) is `GetMethod(obj, @@iterator)` followed by a call, and
        // GetMethod routes through ToObject. Only `undefined` and `null` fail that
        // conversion, so every other primitive (strings, numbers, booleans,
        // symbols, bigints) has to reach its wrapper prototype rather than being
        // rejected here.
        self.compile_nullish_tagged_i32(iterable_tag_local, function)?;
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "for-of target is not iterable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_current_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        let iterable_object_payload_local = self.reserve_temp_local();
        let iterable_object_tag_local = self.reserve_temp_local();
        self.emit_value_to_object_locals(
            iterable_payload_local,
            iterable_tag_local,
            iterable_object_payload_local,
            iterable_object_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(
            self.strings.property_key_symbol_payload("Symbol.iterator"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        // The receiver stays the original value: `@@iterator` is looked up on the
        // wrapper object but invoked with the primitive as `this`.
        self.emit_object_read(
            iterable_object_payload_local,
            iterable_object_tag_local,
            iterable_payload_local,
            iterable_tag_local,
            key_local,
            method_payload_local,
            method_tag_local,
            function,
        )?;
        self.release_temp_local(iterable_object_tag_local);
        self.release_temp_local(iterable_object_payload_local);
        self.emit_propagate_throw_from_locals_if_needed(
            method_payload_local,
            method_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(method_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "for-of iterator method must be callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_current_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.emit_function_handle_call(
            method_payload_local,
            method_tag_local,
            Some((iterable_payload_local, Some(iterable_tag_local))),
            &[],
            iterator_payload_local,
            iterator_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            iterator_payload_local,
            iterator_tag_local,
            function,
        )?;
        self.emit_is_heap_object_like_tag_i32(iterator_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "for-of iterator method must return object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_current_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("next")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            iterator_payload_local,
            iterator_tag_local,
            iterator_payload_local,
            iterator_tag_local,
            key_local,
            next_payload_local,
            next_tag_local,
            function,
        )?;
        self.emit_propagate_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(next_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "for-of iterator next must be callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_current_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.emit_statement_result(function, ValueKind::Undefined);
        let break_frame = self.open_frame(ControlFrameKind::Block, function);
        self.breakable_stack.push(break_frame);
        let loop_frame = self.open_frame(ControlFrameKind::Loop, function);

        self.emit_function_handle_call(
            next_payload_local,
            next_tag_local,
            Some((iterator_payload_local, Some(iterator_tag_local))),
            &[],
            result_payload_local,
            result_tag_local,
            function,
        )?;
        self.emit_propagate_current_completion_if_throw(function);
        self.emit_is_heap_object_like_tag_i32(result_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "for-of iterator next result must be object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_current_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("done")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            result_payload_local,
            result_tag_local,
            result_payload_local,
            result_tag_local,
            key_local,
            done_payload_local,
            done_tag_local,
            function,
        )?;
        self.emit_propagate_current_completion_if_throw(function);
        self.compile_truthy_tagged_i32(done_tag_local, done_payload_local, function)?;
        function.branch_if_to_label(break_frame.label);

        function.instruction(&Instruction::I64Const(self.strings.payload("value")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            result_payload_local,
            result_tag_local,
            result_payload_local,
            result_tag_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_propagate_current_completion_if_throw(function);
        if let Some(environment) =
            lexical_environment.and_then(|environment| environment.iteration_environment.as_ref())
        {
            self.emit_enter_lexical_environment(environment, function)?;
        }
        let storage = self
            .lookup_current_scope_binding(name)
            .or(storage_without_environment)
            .expect("for-of lexical storage must be allocated before assignment");
        self.write_binding_from_locals(storage, value_payload_local, value_tag_local, function);
        self.mirror_binding_to_global_object(name, storage, function)?;

        let continue_frame = self.open_frame(ControlFrameKind::Block, function);
        self.loop_stack.push(LoopTargets { continue_frame });
        self.push_labels(labels, break_frame, Some(continue_frame));
        let finally_frame = self.open_frame(ControlFrameKind::Block, function);
        self.finally_stack.push(finally_frame);
        self.compile_statement(body, function)?;
        self.finally_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.pop_labels(labels.len());
        self.loop_stack.pop();

        self.save_current_completion(
            saved_payload_local,
            saved_tag_local,
            saved_completion_local,
            saved_aux_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(saved_completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_CONTINUE));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(saved_aux_local));
        function.instruction(&Instruction::I64Const(continue_frame.frame as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_NORMAL));
        function.instruction(&Instruction::LocalSet(saved_completion_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(saved_completion_local));
        function.instruction(&Instruction::LocalSet(self.completion_local));
        if lexical_environment
            .and_then(|environment| environment.iteration_environment.as_ref())
            .is_some()
        {
            self.emit_leave_lexical_environment(function);
        }
        self.emit_iterator_close_condition_i32(
            saved_completion_local,
            saved_aux_local,
            continue_frame,
            function,
        );
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::LocalGet(saved_completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.restore_saved_completion(
            saved_payload_local,
            saved_tag_local,
            saved_completion_local,
            saved_aux_local,
            function,
        );
        self.emit_iterator_close_preserving_current_throw(
            IteratorCloseOnThrowLocals {
                iterator_payload_local,
                iterator_tag_local,
                key_local,
                return_payload_local: method_payload_local,
                return_tag_local: method_tag_local,
                result_payload_local,
                result_tag_local,
                saved_payload_local: close_saved_payload_local,
                saved_tag_local: close_saved_tag_local,
                saved_completion_local: close_saved_completion_local,
                saved_aux_local: close_saved_aux_local,
            },
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_iterator_close(
            iterator_payload_local,
            iterator_tag_local,
            key_local,
            method_payload_local,
            method_tag_local,
            result_payload_local,
            result_tag_local,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(saved_completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.restore_saved_completion(
            saved_payload_local,
            saved_tag_local,
            saved_completion_local,
            saved_aux_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_NORMAL));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_dispatch_current_completion(function)?;
        function.instruction(&Instruction::End);

        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        function.branch_to_label(loop_frame.label);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.breakable_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.pop_scope();
        self.release_temp_local(close_saved_aux_local);
        self.release_temp_local(close_saved_completion_local);
        self.release_temp_local(close_saved_tag_local);
        self.release_temp_local(close_saved_payload_local);
        self.release_temp_local(saved_aux_local);
        self.release_temp_local(saved_completion_local);
        self.release_temp_local(saved_tag_local);
        self.release_temp_local(saved_payload_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(done_tag_local);
        self.release_temp_local(done_payload_local);
        self.release_temp_local(result_tag_local);
        self.release_temp_local(result_payload_local);
        self.release_temp_local(next_tag_local);
        self.release_temp_local(next_payload_local);
        self.release_temp_local(iterator_tag_local);
        self.release_temp_local(iterator_payload_local);
        self.release_temp_local(method_tag_local);
        self.release_temp_local(method_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(iterable_tag_local);
        self.release_temp_local(iterable_payload_local);
        Ok(())
    }

    pub(crate) fn compile_object_destructure_to_locals(
        &mut self,
        value: &TypedExpr,
        pattern: &ObjectDestructuringPatternIr,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let source_payload = self.reserve_temp_local();
        let source_tag = self.reserve_temp_local();
        self.compile_expr_to_locals(value, source_payload, source_tag, function)?;
        self.emit_propagate_throw_from_locals_if_needed(source_payload, source_tag, function)?;
        let result = self.compile_object_destructure_from_value_locals(
            source_payload,
            source_tag,
            pattern,
            payload_local,
            tag_local,
            function,
        );
        self.release_temp_local(source_tag);
        self.release_temp_local(source_payload);
        result
    }

    fn compile_object_destructure_from_value_locals(
        &mut self,
        source_payload: u32,
        source_tag: u32,
        pattern: &ObjectDestructuringPatternIr,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let source_object_payload = self.reserve_temp_local();
        let source_object_tag = self.reserve_temp_local();
        let property_value_payload = self.reserve_temp_local();
        let property_value_tag = self.reserve_temp_local();
        let mut excluded_keys = Vec::with_capacity(pattern.properties.len());

        self.compile_nullish_tagged_i32(source_tag, function)?;
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Cannot destructure undefined or null",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_current_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_value_to_object_locals(
            source_payload,
            source_tag,
            source_object_payload,
            source_object_tag,
            function,
        )?;

        for property in &pattern.properties {
            let key_payload = self.reserve_temp_local();
            let key_tag = self.reserve_temp_local();
            match &property.key {
                DestructuringPropertyKeyIr::Static(key) => {
                    function.instruction(&Instruction::I64Const(self.strings.payload(key)));
                    function.instruction(&Instruction::LocalSet(key_payload));
                    function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                    function.instruction(&Instruction::LocalSet(key_tag));
                }
                DestructuringPropertyKeyIr::Computed(key) => {
                    self.compile_expr_to_locals(key, key_payload, key_tag, function)?;
                    self.emit_propagate_throw_from_locals_if_needed(
                        key_payload,
                        key_tag,
                        function,
                    )?;
                    self.emit_value_to_property_key_locals(key_payload, key_tag, function)?;
                }
            }
            let prepared = self.prepare_destructuring_target(&property.target, function)?;
            self.emit_object_read(
                source_object_payload,
                source_object_tag,
                source_object_payload,
                source_object_tag,
                key_payload,
                property_value_payload,
                property_value_tag,
                function,
            )?;
            self.emit_propagate_current_completion_if_throw(function);
            if let Some(default) = &property.default {
                function.instruction(&Instruction::LocalGet(property_value_tag));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                self.open_frame(ControlFrameKind::If, function);
                self.compile_expr_to_locals(
                    default,
                    property_value_payload,
                    property_value_tag,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(
                    property_value_payload,
                    property_value_tag,
                    function,
                )?;
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
            }
            self.put_destructuring_target(
                &property.target,
                prepared,
                property_value_payload,
                property_value_tag,
                function,
            )?;
            excluded_keys.push((key_payload, key_tag));
        }

        if let Some(rest) = &pattern.rest {
            let prepared = self.prepare_destructuring_target(rest, function)?;
            self.emit_copy_data_properties_rest(
                source_object_payload,
                source_object_tag,
                &excluded_keys,
                property_value_payload,
                property_value_tag,
                function,
            )?;
            self.put_destructuring_target(
                rest,
                prepared,
                property_value_payload,
                property_value_tag,
                function,
            )?;
        }

        function.instruction(&Instruction::LocalGet(source_payload));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(source_tag));
        function.instruction(&Instruction::LocalSet(tag_local));

        for (key_payload, key_tag) in excluded_keys.into_iter().rev() {
            self.release_temp_local(key_tag);
            self.release_temp_local(key_payload);
        }
        self.release_temp_local(property_value_tag);
        self.release_temp_local(property_value_payload);
        self.release_temp_local(source_object_tag);
        self.release_temp_local(source_object_payload);
        Ok(())
    }

    fn emit_copy_data_properties_rest(
        &mut self,
        source_payload: u32,
        source_tag: u32,
        excluded_keys: &[(u32, u32)],
        target_payload: u32,
        target_tag: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(target_payload));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(target_tag));

        self.emit_copy_data_properties_into(
            source_payload,
            source_tag,
            excluded_keys,
            target_payload,
            function,
        )
    }

    pub(crate) fn emit_copy_data_properties_into(
        &mut self,
        source_payload: u32,
        source_tag: u32,
        excluded_keys: &[(u32, u32)],
        target_payload: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let own_keys_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectOwnKeys.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Reflect.ownKeys`",
                )
            })?;
        let get_own_descriptor_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectGetOwnPropertyDescriptor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Reflect.getOwnPropertyDescriptor`",
                )
            })?;
        let keys_payload = self.reserve_temp_local();
        let keys_tag = self.reserve_temp_local();
        let keys_length = self.reserve_temp_local();
        let key_index = self.reserve_temp_local();
        let key_payload = self.reserve_temp_local();
        let key_tag = self.reserve_temp_local();
        let key_internal_payload = self.reserve_temp_local();
        let descriptor_payload = self.reserve_temp_local();
        let descriptor_tag = self.reserve_temp_local();
        let enumerable_key = self.reserve_temp_local();
        let enumerable_payload = self.reserve_temp_local();
        let enumerable_tag = self.reserve_temp_local();

        self.emit_direct_js_call(
            &own_keys_meta,
            None,
            &[(source_payload, source_tag)],
            keys_payload,
            keys_tag,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(keys_payload, keys_tag, function)?;
        self.load_i64_to_local_from_offset(keys_payload, HEAP_LEN_OFFSET, keys_length, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(key_index));
        function.instruction(&Instruction::I64Const(self.strings.payload("enumerable")));
        function.instruction(&Instruction::LocalSet(enumerable_key));

        let copy_break = self.open_frame(ControlFrameKind::Block, function);
        let copy_loop = self.open_frame(ControlFrameKind::Loop, function);
        function.instruction(&Instruction::LocalGet(key_index));
        function.instruction(&Instruction::LocalGet(keys_length));
        function.instruction(&Instruction::I64GeU);
        self.emit_branch_if_to_target(copy_break, function);
        self.emit_array_read(keys_payload, key_index, key_payload, key_tag, function);

        let skip_key = self.open_frame(ControlFrameKind::Block, function);
        for (excluded_payload, excluded_tag) in excluded_keys {
            self.emit_tagged_payload_same_value_i32(
                key_tag,
                key_payload,
                *excluded_tag,
                *excluded_payload,
                function,
            )?;
            self.emit_branch_if_to_target(skip_key, function);
        }

        self.emit_direct_js_call(
            &get_own_descriptor_meta,
            None,
            &[(source_payload, source_tag), (key_payload, key_tag)],
            descriptor_payload,
            descriptor_tag,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            descriptor_payload,
            descriptor_tag,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(descriptor_tag));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        self.emit_branch_if_to_target(skip_key, function);

        self.emit_object_read(
            descriptor_payload,
            descriptor_tag,
            descriptor_payload,
            descriptor_tag,
            enumerable_key,
            enumerable_payload,
            enumerable_tag,
            function,
        )?;
        self.emit_propagate_current_completion_if_throw(function);
        self.compile_truthy_tagged_i32(enumerable_tag, enumerable_payload, function)?;
        function.instruction(&Instruction::I32Eqz);
        self.emit_branch_if_to_target(skip_key, function);

        // `Reflect.ownKeys` yields keys as JS values; both the [[Get]] and the
        // CreateDataPropertyOrThrow below index on the internal property-key
        // payload, which re-marks symbol keys.
        self.emit_property_key_payload_from_value_local(
            key_payload,
            key_tag,
            key_internal_payload,
            function,
        );
        self.emit_object_read_with_key_tag(
            source_payload,
            source_tag,
            source_payload,
            source_tag,
            key_internal_payload,
            Some(key_tag),
            enumerable_payload,
            enumerable_tag,
            function,
        )?;
        self.emit_propagate_current_completion_if_throw(function);
        self.emit_object_define_enumerable_data(
            target_payload,
            key_internal_payload,
            enumerable_payload,
            enumerable_tag,
            function,
        )?;
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(key_index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(key_index));
        self.emit_branch_to_target(copy_loop, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

        self.release_temp_local(enumerable_tag);
        self.release_temp_local(enumerable_payload);
        self.release_temp_local(enumerable_key);
        self.release_temp_local(descriptor_tag);
        self.release_temp_local(descriptor_payload);
        self.release_temp_local(key_internal_payload);
        self.release_temp_local(key_tag);
        self.release_temp_local(key_payload);
        self.release_temp_local(key_index);
        self.release_temp_local(keys_length);
        self.release_temp_local(keys_tag);
        self.release_temp_local(keys_payload);
        Ok(())
    }

    pub(crate) fn compile_array_destructure_to_locals(
        &mut self,
        value: &TypedExpr,
        pattern: &ArrayDestructuringPatternIr,
        evaluation: ArrayDestructuringEvaluationIr,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let source_payload = self.reserve_temp_local();
        let source_tag = self.reserve_temp_local();
        self.compile_expr_to_locals(value, source_payload, source_tag, function)?;
        self.emit_propagate_throw_from_locals_if_needed(source_payload, source_tag, function)?;
        self.compile_array_destructure_from_value_locals(
            value.value_info(),
            source_payload,
            source_tag,
            pattern,
            function,
        )?;
        match evaluation {
            ArrayDestructuringEvaluationIr::BindingInitialization => {
                self.emit_undefined_payload(function);
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
            }
            ArrayDestructuringEvaluationIr::AssignmentEvaluation => {
                function.instruction(&Instruction::LocalGet(source_payload));
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::LocalGet(source_tag));
                function.instruction(&Instruction::LocalSet(tag_local));
            }
        }
        self.release_temp_local(source_tag);
        self.release_temp_local(source_payload);
        Ok(())
    }

    pub(crate) fn compile_array_destructure_from_value_locals(
        &mut self,
        value_info: ValueInfo,
        source_payload: u32,
        source_tag: u32,
        pattern: &ArrayDestructuringPatternIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let method_payload = self.reserve_temp_local();
        let method_tag = self.reserve_temp_local();
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

        self.emit_get_iterator_from_value_locals(
            value_info,
            source_payload,
            source_tag,
            method_payload,
            method_tag,
            locals.protocol(),
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(locals.done));

        let exit_target = self.open_frame(ControlFrameKind::Block, function);
        let abrupt_target = self.open_frame(ControlFrameKind::Block, function);
        self.finally_stack.push(abrupt_target);
        for element in &pattern.elements {
            self.compile_array_destructuring_element(element, locals, function)?;
        }
        self.finally_stack.pop();

        function.instruction(&Instruction::LocalGet(locals.done));
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
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
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_branch_to_target(exit_target, function);

        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(locals.done));
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
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
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_propagate_current_completion_if_throw(function);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

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
            method_tag,
            method_payload,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Reserves the common GetIterator/IteratorStep/IteratorValue working set.
    /// The matching release method owns the reverse-order discipline so a new
    /// iterator consumer cannot silently corrupt the temp-local stack.
    pub(crate) fn reserve_sync_iterator_locals(&mut self) -> SyncIteratorLocals {
        SyncIteratorLocals {
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
        }
    }

    pub(crate) fn release_sync_iterator_locals(&mut self, locals: SyncIteratorLocals) {
        for local in [
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
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_get_iterator_from_value_locals(
        &mut self,
        value_info: ValueInfo,
        source_payload: u32,
        source_tag: u32,
        method_payload: u32,
        method_tag: u32,
        locals: SyncIteratorLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // The arguments exotic object resolves `@@iterator` through a dedicated
        // arm in `compile_property_read_from_locals`, keyed on the *static* name
        // rather than on a runtime key local. Routing it through the generic
        // symbol-key read below misses that arm and leaves `arguments` without
        // an iterator, so `const [x, y] = arguments;` throws TypeError.
        if value_info.kind == ValueKind::Arguments {
            let source_name = "$array.destructure.source";
            self.push_scope();
            self.binding_scopes
                .last_mut()
                .expect("binding scope stack must exist")
                .insert(
                    source_name.to_string(),
                    BindingStorage::Dynamic {
                        tag_local: source_tag,
                        payload_local: source_payload,
                    },
                );
            let source =
                TypedExpr::from_info(value_info, ExprIr::Identifier(source_name.to_string()));
            let method = TypedExpr::from_info(
                ValueInfo {
                    kind: ValueKind::Dynamic,
                    possible_kinds: KindSet::all_runtime_tags(),
                    heap_shape: None,
                    function_targets: BTreeSet::new(),
                },
                ExprIr::PropertyRead {
                    target: Box::new(source),
                    key: PropertyKeyIr::StaticString("Symbol.iterator".to_string()),
                },
            );
            self.compile_expr_to_locals(&method, method_payload, method_tag, function)?;
            self.pop_scope();
            return self.finish_get_iterator_from_method(
                source_payload,
                source_tag,
                method_payload,
                method_tag,
                locals,
                function,
            );
        }

        // GetIterator always reads the well-known `@@iterator` symbol key, never a
        // string property literally named "Symbol.iterator", so the key payload has
        // to carry the property-key symbol marker for every receiver shape (arrays,
        // strings, generators, user iterables, proxies).
        let source_object_payload = self.reserve_temp_local();
        let source_object_tag = self.reserve_temp_local();
        self.compile_nullish_tagged_i32(source_tag, function)?;
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "destructuring value is not iterable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_current_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        // Primitive sources (notably strings) resolve `@@iterator` through their
        // wrapper prototype; ToObject leaves objects, arrays and functions alone.
        self.emit_value_to_object_locals(
            source_payload,
            source_tag,
            source_object_payload,
            source_object_tag,
            function,
        )?;
        function.instruction(&Instruction::I64Const(
            self.strings.property_key_symbol_payload("Symbol.iterator"),
        ));
        function.instruction(&Instruction::LocalSet(locals.key));
        self.emit_object_read(
            source_object_payload,
            source_object_tag,
            source_payload,
            source_tag,
            locals.key,
            method_payload,
            method_tag,
            function,
        )?;
        self.release_temp_local(source_object_tag);
        self.release_temp_local(source_object_payload);
        self.finish_get_iterator_from_method(
            source_payload,
            source_tag,
            method_payload,
            method_tag,
            locals,
            function,
        )
    }

    /// The half of GetIterator after the `@@iterator` method has been loaded:
    /// callability check, the call itself, the object-result check, and caching
    /// `next`. Shared because the arguments exotic object reaches the method
    /// through a different read than every other receiver shape.
    #[allow(clippy::too_many_arguments)]
    fn finish_get_iterator_from_method(
        &mut self,
        source_payload: u32,
        source_tag: u32,
        method_payload: u32,
        method_tag: u32,
        locals: SyncIteratorLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_propagate_throw_from_locals_if_needed(method_payload, method_tag, function)?;
        self.emit_is_callable_i32(method_tag, method_payload, function)?;
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "destructuring value is not iterable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_current_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(method_tag));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_function_handle_call(
            method_payload,
            method_tag,
            Some((source_payload, Some(source_tag))),
            &[],
            locals.iterator_payload,
            locals.iterator_tag,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_function_or_proxy_call_leave_throw_completion(
            method_payload,
            method_tag,
            source_payload,
            source_tag,
            &[],
            locals.iterator_payload,
            locals.iterator_tag,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_propagate_throw_from_locals_if_needed(
            locals.iterator_payload,
            locals.iterator_tag,
            function,
        )?;
        self.emit_is_heap_object_like_tag_i32(locals.iterator_tag, function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "destructuring iterator method must return object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_current_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("next")));
        function.instruction(&Instruction::LocalSet(locals.key));
        self.emit_object_read(
            locals.iterator_payload,
            locals.iterator_tag,
            locals.iterator_payload,
            locals.iterator_tag,
            locals.key,
            locals.next_payload,
            locals.next_tag,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            locals.next_payload,
            locals.next_tag,
            function,
        )?;
        Ok(())
    }

    fn compile_array_destructuring_element(
        &mut self,
        element: &ArrayDestructuringElementIr,
        locals: DestructuringIteratorLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match element {
            ArrayDestructuringElementIr::Elision => {
                function.instruction(&Instruction::LocalGet(locals.done));
                function.instruction(&Instruction::I64Eqz);
                self.open_frame(ControlFrameKind::If, function);
                self.emit_destructuring_iterator_step(
                    locals,
                    DestructuringIteratorStepKind::Elision,
                    function,
                )?;
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
            }
            ArrayDestructuringElementIr::Target { target, default } => {
                let prepared = self.prepare_destructuring_target(target, function)?;
                self.emit_undefined_payload(function);
                function.instruction(&Instruction::LocalSet(locals.value_payload));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::LocalSet(locals.value_tag));
                function.instruction(&Instruction::LocalGet(locals.done));
                function.instruction(&Instruction::I64Eqz);
                self.open_frame(ControlFrameKind::If, function);
                self.emit_destructuring_iterator_step(
                    locals,
                    DestructuringIteratorStepKind::Value,
                    function,
                )?;
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
                if let Some(default) = default {
                    function.instruction(&Instruction::LocalGet(locals.value_tag));
                    function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    self.open_frame(ControlFrameKind::If, function);
                    self.compile_expr_to_locals(
                        default,
                        locals.value_payload,
                        locals.value_tag,
                        function,
                    )?;
                    self.emit_propagate_throw_from_locals_if_needed(
                        locals.value_payload,
                        locals.value_tag,
                        function,
                    )?;
                    self.pop_control(ControlFrameKind::If);
                    function.instruction(&Instruction::End);
                }
                self.put_destructuring_target(
                    target,
                    prepared,
                    locals.value_payload,
                    locals.value_tag,
                    function,
                )?;
            }
            ArrayDestructuringElementIr::Rest { target } => {
                let prepared = self.prepare_destructuring_target(target, function)?;
                let rest_payload = self.reserve_temp_local();
                self.compile_array_literal_payload(&[], function)?;
                function.instruction(&Instruction::LocalSet(rest_payload));
                let rest_index = self.reserve_temp_local();
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(rest_index));
                let rest_break = self.open_frame(ControlFrameKind::Block, function);
                let rest_loop = self.open_frame(ControlFrameKind::Loop, function);
                function.instruction(&Instruction::LocalGet(locals.done));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::I32Eqz);
                self.emit_branch_if_to_target(rest_break, function);
                self.emit_destructuring_iterator_step(
                    locals,
                    DestructuringIteratorStepKind::Value,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(locals.done));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::I32Eqz);
                self.emit_branch_if_to_target(rest_break, function);
                self.emit_array_write(
                    rest_payload,
                    rest_index,
                    locals.value_payload,
                    locals.value_tag,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(rest_index));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::LocalSet(rest_index));
                self.emit_branch_to_target(rest_loop, function);
                self.pop_control(ControlFrameKind::Loop);
                function.instruction(&Instruction::End);
                self.pop_control(ControlFrameKind::Block);
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
                function.instruction(&Instruction::LocalSet(locals.value_tag));
                function.instruction(&Instruction::LocalGet(rest_payload));
                function.instruction(&Instruction::LocalSet(locals.value_payload));
                self.release_temp_local(rest_index);
                self.release_temp_local(rest_payload);
                self.put_destructuring_target(
                    target,
                    prepared,
                    locals.value_payload,
                    locals.value_tag,
                    function,
                )?;
            }
        }
        Ok(())
    }

    fn emit_destructuring_iterator_step(
        &mut self,
        locals: DestructuringIteratorLocals,
        step_kind: DestructuringIteratorStepKind,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(locals.done));
        function.instruction(&Instruction::LocalGet(locals.next_tag));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_function_handle_call(
            locals.next_payload,
            locals.next_tag,
            Some((locals.iterator_payload, Some(locals.iterator_tag))),
            &[],
            locals.result_payload,
            locals.result_tag,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_function_or_proxy_call_leave_throw_completion(
            locals.next_payload,
            locals.next_tag,
            locals.iterator_payload,
            locals.iterator_tag,
            &[],
            locals.result_payload,
            locals.result_tag,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_propagate_throw_from_locals_if_needed(
            locals.result_payload,
            locals.result_tag,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(locals.done));
        self.emit_is_heap_object_like_tag_i32(locals.result_tag, function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(locals.done));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "destructuring iterator next result must be object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_current_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("done")));
        function.instruction(&Instruction::LocalSet(locals.key));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(locals.done));
        self.emit_object_read(
            locals.result_payload,
            locals.result_tag,
            locals.result_payload,
            locals.result_tag,
            locals.key,
            locals.done_payload,
            locals.done_tag,
            function,
        )?;
        self.emit_propagate_current_completion_if_throw(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(locals.done));
        self.compile_truthy_tagged_i32(locals.done_tag, locals.done_payload, function)?;
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(locals.done));
        self.emit_undefined_payload(function);
        function.instruction(&Instruction::LocalSet(locals.value_payload));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(locals.value_tag));
        function.instruction(&Instruction::Else);
        if matches!(step_kind, DestructuringIteratorStepKind::Value) {
            function.instruction(&Instruction::I64Const(self.strings.payload("value")));
            function.instruction(&Instruction::LocalSet(locals.key));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(locals.done));
            self.emit_object_read(
                locals.result_payload,
                locals.result_tag,
                locals.result_payload,
                locals.result_tag,
                locals.key,
                locals.value_payload,
                locals.value_tag,
                function,
            )?;
            self.emit_propagate_current_completion_if_throw(function);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(locals.done));
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }

    /// Runs one sync IteratorStep/IteratorValue pair without introducing an
    /// IteratorClose path. This is the exact control shape required by
    /// ArrayAccumulation: any abrupt completion propagates directly, `done` is
    /// set for a completed iterator, and `value` is read only on the false arm.
    pub(crate) fn emit_sync_iterator_step_value(
        &mut self,
        locals: SyncIteratorLocals,
        done: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(done));
        function.instruction(&Instruction::LocalGet(locals.next_tag));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_function_handle_call(
            locals.next_payload,
            locals.next_tag,
            Some((locals.iterator_payload, Some(locals.iterator_tag))),
            &[],
            locals.result_payload,
            locals.result_tag,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_function_or_proxy_call_leave_throw_completion(
            locals.next_payload,
            locals.next_tag,
            locals.iterator_payload,
            locals.iterator_tag,
            &[],
            locals.result_payload,
            locals.result_tag,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_propagate_throw_from_locals_if_needed(
            locals.result_payload,
            locals.result_tag,
            function,
        )?;
        self.emit_is_heap_object_like_tag_i32(locals.result_tag, function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "iterator next result must be object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_current_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("done")));
        function.instruction(&Instruction::LocalSet(locals.key));
        self.emit_object_read(
            locals.result_payload,
            locals.result_tag,
            locals.result_payload,
            locals.result_tag,
            locals.key,
            locals.done_payload,
            locals.done_tag,
            function,
        )?;
        self.emit_propagate_current_completion_if_throw(function);
        self.compile_truthy_tagged_i32(locals.done_tag, locals.done_payload, function)?;
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(done));
        function.instruction(&Instruction::LocalGet(done));
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(self.strings.payload("value")));
        function.instruction(&Instruction::LocalSet(locals.key));
        self.emit_object_read(
            locals.result_payload,
            locals.result_tag,
            locals.result_payload,
            locals.result_tag,
            locals.key,
            locals.value_payload,
            locals.value_tag,
            function,
        )?;
        self.emit_propagate_current_completion_if_throw(function);
        function.instruction(&Instruction::Else);
        self.emit_undefined_payload(function);
        function.instruction(&Instruction::LocalSet(locals.value_payload));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(locals.value_tag));
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn prepare_destructuring_target(
        &mut self,
        target: &DestructuringTargetIr,
        function: &mut Function,
    ) -> Result<PreparedDestructuringTarget, EmitError> {
        if let DestructuringTargetIr::AssignmentPrivate {
            target,
            private_name_id,
        } = target
        {
            let target_payload = self.reserve_temp_local();
            let target_tag = self.reserve_temp_local();
            self.compile_expr_to_locals(target, target_payload, target_tag, function)?;
            self.emit_propagate_throw_from_locals_if_needed(target_payload, target_tag, function)?;
            return Ok(PreparedDestructuringTarget::Private {
                target_payload,
                target_tag,
                private_name_id: *private_name_id,
            });
        }

        let DestructuringTargetIr::AssignmentProperty {
            target,
            key,
            strictness,
        } = target
        else {
            return Ok(PreparedDestructuringTarget::Direct);
        };

        let target_payload = self.reserve_temp_local();
        let target_tag = self.reserve_temp_local();
        self.compile_expr_to_locals(target, target_payload, target_tag, function)?;
        self.emit_propagate_throw_from_locals_if_needed(target_payload, target_tag, function)?;
        let (key_payload, key_tag) = match key {
            DestructuringPropertyKeyIr::Static(_) => (None, None),
            DestructuringPropertyKeyIr::Computed(key) => {
                let key_payload = self.reserve_temp_local();
                let key_tag = self.reserve_temp_local();
                self.compile_expr_to_locals(key, key_payload, key_tag, function)?;
                self.emit_propagate_throw_from_locals_if_needed(key_payload, key_tag, function)?;
                (Some(key_payload), Some(key_tag))
            }
        };
        Ok(PreparedDestructuringTarget::Property {
            target: target.clone(),
            target_payload,
            target_tag,
            key: key.clone(),
            key_payload,
            key_tag,
            strictness: *strictness,
        })
    }

    fn put_destructuring_target(
        &mut self,
        target: &DestructuringTargetIr,
        prepared: PreparedDestructuringTarget,
        value_payload: u32,
        value_tag: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match target {
            DestructuringTargetIr::Binding { mode, name } => {
                let storage = self
                    .lookup_current_scope_binding(name)
                    .or_else(|| self.lookup_binding(name))
                    .unwrap_or_else(|| {
                        self.allocate_binding(name.clone(), *mode, ValueKind::Dynamic)
                    });
                self.write_binding_from_locals(storage, value_payload, value_tag, function);
                self.mirror_binding_to_global_object(name, storage, function)?;
            }
            DestructuringTargetIr::AssignmentIdentifier(reference) => {
                match reference.write_disposition() {
                    IdentifierWriteDisposition::MutableBinding { storage_name } => {
                        let storage = self.lookup_binding(storage_name).ok_or_else(|| {
                            EmitError::unsupported(format!(
                                "unsupported in lila wasm-aot first slice: unbound destructuring assignment `{storage_name}`"
                            ))
                        })?;
                        self.write_binding_from_locals(storage, value_payload, value_tag, function);
                        self.mirror_binding_to_global_object(storage_name, storage, function)?;
                    }
                    IdentifierWriteDisposition::IgnoreImmutableBinding => {}
                    IdentifierWriteDisposition::Throw { error } => {
                        // The value and any default initializer have already
                        // been evaluated. Emitting the abrupt completion here
                        // is 13.15.5.3's PutValue position, not an eager target
                        // preparation failure.
                        self.emit_throw_runtime_error(
                            error.kind().as_str(),
                            error.message(),
                            self.result_local,
                            self.result_tag_local,
                            function,
                        )?;
                        self.emit_propagate_current_throw(function);
                    }
                    IdentifierWriteDisposition::Global {
                        referenced_name,
                        strictness,
                    } => {
                        // PutValue steps 2.a and 3.d consume the Reference's
                        // carried `[[Strict]]`. The old `global: bool` arm used
                        // the unchecked writer, so strict destructuring could
                        // create an unresolvable implicit global.
                        self.with_reference_strictness(
                            strictness,
                            function,
                            |emitter, function| {
                                emitter.emit_global_property_write_checked(
                                    referenced_name,
                                    value_payload,
                                    value_tag,
                                    strictness,
                                    function,
                                )
                            },
                        )?;
                    }
                }
            }
            DestructuringTargetIr::AssignmentProperty { .. } => {
                let PreparedDestructuringTarget::Property {
                    target,
                    target_payload,
                    target_tag,
                    key,
                    key_payload,
                    key_tag,
                    strictness,
                } = prepared
                else {
                    unreachable!("property destructuring target must be prepared")
                };
                let target_name = "$array.destructure.target";
                let value_name = "$array.destructure.value";
                let key_name = "$array.destructure.key";
                self.push_scope();
                let scope = self
                    .binding_scopes
                    .last_mut()
                    .expect("binding scope stack must exist");
                scope.insert(
                    target_name.to_string(),
                    BindingStorage::Dynamic {
                        tag_local: target_tag,
                        payload_local: target_payload,
                    },
                );
                scope.insert(
                    value_name.to_string(),
                    BindingStorage::Dynamic {
                        tag_local: value_tag,
                        payload_local: value_payload,
                    },
                );
                if let (Some(key_payload), Some(key_tag)) = (key_payload, key_tag) {
                    scope.insert(
                        key_name.to_string(),
                        BindingStorage::Dynamic {
                            tag_local: key_tag,
                            payload_local: key_payload,
                        },
                    );
                }
                let target_expr = TypedExpr::from_info(
                    target.value_info(),
                    ExprIr::Identifier(target_name.to_string()),
                );
                let property_key = match key {
                    DestructuringPropertyKeyIr::Static(name) => PropertyKeyIr::StaticString(name),
                    DestructuringPropertyKeyIr::Computed(key) => {
                        let raw_key = TypedExpr::from_info(
                            key.value_info(),
                            ExprIr::Identifier(key_name.to_string()),
                        );
                        PropertyKeyIr::StringExpr(Box::new(TypedExpr::spec_to_property_key(
                            raw_key,
                        )))
                    }
                };
                let value_expr = TypedExpr::from_info(
                    ValueInfo {
                        kind: ValueKind::Dynamic,
                        possible_kinds: KindSet::all_runtime_tags(),
                        heap_shape: None,
                        function_targets: BTreeSet::new(),
                    },
                    ExprIr::Identifier(value_name.to_string()),
                );
                // 13.15.5.4's PutValue, under *this* Reference's `[[Strict]]`.
                // Without the scope the write would fall back to
                // `ambient_object_write_strict_flag_word`, i.e. the mode of the
                // Wasm function the pattern was emitted into.
                self.with_reference_strictness(strictness, function, |emitter, function| {
                    emitter.compile_property_write_to_locals(
                        &target_expr,
                        &property_key,
                        &value_expr,
                        emitter.scratch_local,
                        emitter.result_tag_local,
                        function,
                    )
                })?;
                self.emit_propagate_current_completion_if_throw(function);
                self.pop_scope();
                if let Some(key_tag) = key_tag {
                    self.release_temp_local(key_tag);
                }
                if let Some(key_payload) = key_payload {
                    self.release_temp_local(key_payload);
                }
                self.release_temp_local(target_tag);
                self.release_temp_local(target_payload);
            }
            DestructuringTargetIr::AssignmentPrivate { .. } => {
                let PreparedDestructuringTarget::Private {
                    target_payload,
                    target_tag,
                    private_name_id,
                } = prepared
                else {
                    unreachable!("private destructuring target must be prepared")
                };
                self.emit_private_write_from_locals(
                    target_payload,
                    target_tag,
                    private_name_id,
                    value_payload,
                    value_tag,
                    function,
                )?;
                self.release_temp_local(target_tag);
                self.release_temp_local(target_payload);
            }
            DestructuringTargetIr::NestedArray(pattern) => {
                self.compile_array_destructure_from_value_locals(
                    ValueInfo {
                        kind: ValueKind::Dynamic,
                        possible_kinds: KindSet::all_runtime_tags(),
                        heap_shape: None,
                        function_targets: BTreeSet::new(),
                    },
                    value_payload,
                    value_tag,
                    pattern,
                    function,
                )?;
            }
            DestructuringTargetIr::NestedObject(pattern) => {
                self.compile_object_destructure_from_value_locals(
                    value_payload,
                    value_tag,
                    pattern,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
            }
        }
        Ok(())
    }

    pub(crate) fn emit_iterator_close_condition_i32(
        &self,
        completion_local: u32,
        aux_local: u32,
        current_continue_target: ControlTarget,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_RETURN));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_BREAK));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_CONTINUE));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(aux_local));
        function.instruction(&Instruction::I64Const(current_continue_target.frame as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
    }

    pub(crate) fn emit_iterator_close(
        &mut self,
        iterator_payload_local: u32,
        iterator_tag_local: u32,
        key_local: u32,
        return_payload_local: u32,
        return_tag_local: u32,
        result_payload_local: u32,
        result_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::I64Const(self.strings.payload("return")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_undefined_payload(function);
        function.instruction(&Instruction::LocalSet(return_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(return_tag_local));
        self.emit_object_read(
            iterator_payload_local,
            iterator_tag_local,
            iterator_payload_local,
            iterator_tag_local,
            key_local,
            return_payload_local,
            return_tag_local,
            function,
        )?;
        self.emit_propagate_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(return_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(return_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_is_callable_i32(return_tag_local, return_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "IteratorClose return method must be callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_current_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(return_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_function_handle_call(
            return_payload_local,
            return_tag_local,
            Some((iterator_payload_local, Some(iterator_tag_local))),
            &[],
            result_payload_local,
            result_tag_local,
            function,
        )?;
        self.emit_propagate_current_completion_if_throw(function);
        function.instruction(&Instruction::Else);
        self.emit_function_or_proxy_call_leave_throw_completion(
            return_payload_local,
            return_tag_local,
            iterator_payload_local,
            iterator_tag_local,
            &[],
            result_payload_local,
            result_tag_local,
            function,
        )?;
        self.emit_propagate_current_completion_if_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_is_heap_object_like_tag_i32(result_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "IteratorClose return result must be object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_current_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_exhaust_static_generator_iterator_if_marked(
            iterator_payload_local,
            iterator_tag_local,
            key_local,
            function,
        )?;
        Ok(())
    }

    pub(crate) fn emit_exhaust_static_generator_iterator_if_marked(
        &mut self,
        iterator_payload_local: u32,
        iterator_tag_local: u32,
        key_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let marker_present_local = self.reserve_temp_local();
        let marker_payload_local = self.reserve_temp_local();
        let marker_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(
            self.strings.payload(LILA_STATIC_GENERATOR_ITERATOR_SLOT),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_own_data_field_read(
            iterator_payload_local,
            iterator_tag_local,
            key_local,
            marker_present_local,
            marker_payload_local,
            marker_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(marker_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_define_bool_data(
            iterator_payload_local,
            "$ArrayIterator.done",
            true,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.release_temp_local(marker_tag_local);
        self.release_temp_local(marker_payload_local);
        self.release_temp_local(marker_present_local);
        Ok(())
    }

    pub(crate) fn emit_iterator_close_preserving_current_throw(
        &mut self,
        close: IteratorCloseOnThrowLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.save_current_completion(
            close.saved_payload_local,
            close.saved_tag_local,
            close.saved_completion_local,
            close.saved_aux_local,
            function,
        );
        self.emit_iterator_close_preserving_saved_throw(close, function)
    }

    pub(crate) fn emit_iterator_close_preserving_saved_throw(
        &mut self,
        close: IteratorCloseOnThrowLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.set_completion_kind(CompletionKind::Normal, function);
        let close_frame = self.open_frame(ControlFrameKind::Block, function);
        self.finally_stack.push(close_frame);
        self.emit_iterator_close(
            close.iterator_payload_local,
            close.iterator_tag_local,
            close.key_local,
            close.return_payload_local,
            close.return_tag_local,
            close.result_payload_local,
            close.result_tag_local,
            function,
        )?;
        self.finally_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.restore_saved_completion(
            close.saved_payload_local,
            close.saved_tag_local,
            close.saved_completion_local,
            close.saved_aux_local,
            function,
        );
        Ok(())
    }

    pub(crate) fn emit_iterator_flat_map_close_outer_after_throw(
        &mut self,
        helper_payload_local: u32,
        close: IteratorCloseOnThrowLocals,
        clear_inner_active: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_iterator_close_preserving_current_throw(close, function)?;
        self.emit_object_define_bool_data(
            helper_payload_local,
            "$IteratorFlatMapDone",
            true,
            function,
        )?;
        if clear_inner_active {
            self.emit_object_define_bool_data(
                helper_payload_local,
                "$IteratorFlatMapInnerActive",
                false,
                function,
            )?;
        }
        self.emit_object_define_bool_data(
            helper_payload_local,
            "$IteratorFlatMapExecuting",
            false,
            function,
        )?;
        Ok(())
    }

    pub(crate) fn compile_for_in_array(
        &mut self,
        mode: BindingMode,
        name: &str,
        target: &TypedExpr,
        body: &StatementIr,
        lexical_environment: Option<&ForInOfEnvironmentIr>,
        labels: &[String],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_for_in_object(
            mode,
            name,
            target,
            body,
            lexical_environment,
            labels,
            function,
        )
    }

    pub(crate) fn compile_for_in_string(
        &mut self,
        mode: BindingMode,
        name: &str,
        target: &TypedExpr,
        body: &StatementIr,
        lexical_environment: Option<&ForInOfEnvironmentIr>,
        labels: &[String],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let string_payload_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let byte_len_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();

        if let Some(environment) = lexical_environment {
            self.emit_enter_for_in_of_tdz_scope(mode, environment, function)?;
        }
        self.compile_expr_to_locals(target, target_payload_local, target_tag_local, function)?;
        if let Some(environment) = lexical_environment {
            self.emit_leave_for_in_of_tdz_scope(environment, function);
        }

        self.push_scope();
        let storage_without_environment = if mode == BindingMode::Var {
            Some(self.lookup_binding(name).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in lila wasm-aot first slice: unbound for-in var `{name}`"
                ))
            })?)
        } else if !iteration_environment_owns_binding(lexical_environment, name) {
            Some(self.allocate_binding(name.to_string(), mode, ValueKind::String))
        } else {
            None
        };
        if mode == BindingMode::Var {
            self.binding_scopes
                .last_mut()
                .expect("binding scope stack must exist")
                .insert(
                    name.to_string(),
                    storage_without_environment.expect("for-in var storage must exist"),
                );
        }
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(string_payload_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::LocalSet(string_payload_local));
        function.instruction(&Instruction::End);
        self.emit_is_heap_object_like_tag_i32(target_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            self.scratch_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(self.scratch_local));
        function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_STRING as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            string_payload_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_unpack_string_payload(
            string_payload_local,
            buffer_local,
            byte_len_local,
            function,
        );
        self.emit_utf16_code_unit_len_from_utf8_locals(
            buffer_local,
            byte_len_local,
            len_local,
            function,
        );

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        self.emit_statement_result(function, ValueKind::Undefined);
        let break_frame = self.open_frame(ControlFrameKind::Block, function);
        self.breakable_stack.push(break_frame);
        let loop_frame = self.open_frame(ControlFrameKind::Loop, function);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.branch_if_to_label(break_frame.label);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_number_to_string_payload(index_number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        if let Some(environment) =
            lexical_environment.and_then(|environment| environment.iteration_environment.as_ref())
        {
            self.emit_enter_lexical_environment(environment, function)?;
        }
        let storage = self
            .lookup_current_scope_binding(name)
            .or(storage_without_environment)
            .expect("for-in lexical storage must be allocated before assignment");
        self.write_binding_from_locals(storage, key_payload_local, key_tag_local, function);
        self.mirror_binding_to_global_object(name, storage, function)?;
        let continue_frame = self.open_frame(ControlFrameKind::Block, function);
        self.loop_stack.push(LoopTargets { continue_frame });
        self.push_labels(labels, break_frame, Some(continue_frame));
        self.compile_statement(body, function)?;
        self.pop_labels(labels.len());
        self.loop_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        if lexical_environment
            .and_then(|environment| environment.iteration_environment.as_ref())
            .is_some()
        {
            self.emit_leave_lexical_environment(function);
        }
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.branch_to_label(loop_frame.label);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.breakable_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.pop_scope();
        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(byte_len_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(string_payload_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        Ok(())
    }

    pub(crate) fn emit_array_contains_string_payload(
        &mut self,
        array_local: u32,
        len_local: u32,
        key_payload_local: u32,
        found_local: u32,
        function: &mut Function,
    ) {
        let index_local = self.reserve_temp_local();
        let candidate_payload_local = self.reserve_temp_local();
        let candidate_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            array_local,
            index_local,
            candidate_payload_local,
            candidate_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(candidate_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        self.emit_string_payload_equality_i32(candidate_payload_local, key_payload_local, function);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(candidate_tag_local);
        self.release_temp_local(candidate_payload_local);
        self.release_temp_local(index_local);
    }

    pub(crate) fn emit_for_in_append_unvisited_string_keys(
        &mut self,
        keys_array_local: u32,
        keys_len_local: u32,
        visited_array_local: u32,
        visited_len_local: u32,
        result_array_local: u32,
        result_len_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let index_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let found_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(keys_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            keys_array_local,
            index_local,
            key_payload_local,
            key_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_contains_string_payload(
            visited_array_local,
            visited_len_local,
            key_payload_local,
            found_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_write(
            result_array_local,
            result_len_local,
            key_payload_local,
            key_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(result_len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(result_len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(found_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(index_local);
        Ok(())
    }

    pub(crate) fn emit_for_in_mark_visited_string_keys(
        &mut self,
        keys_array_local: u32,
        keys_len_local: u32,
        visited_array_local: u32,
        visited_len_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let index_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let found_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(keys_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            keys_array_local,
            index_local,
            key_payload_local,
            key_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_contains_string_payload(
            visited_array_local,
            visited_len_local,
            key_payload_local,
            found_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_write(
            visited_array_local,
            visited_len_local,
            key_payload_local,
            key_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(visited_len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(visited_len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(found_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(index_local);
        Ok(())
    }

    pub(crate) fn emit_for_in_object_key_snapshot(
        &mut self,
        object_local: u32,
        object_tag_local: u32,
        result_array_local: u32,
        result_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let keys_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectKeys.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Object.keys`",
                )
            })?;
        let own_names_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectGetOwnPropertyNames.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Object.getOwnPropertyNames`",
                )
            })?;
        let current_local = self.reserve_temp_local();
        let current_tag_local = self.reserve_temp_local();
        let enumerable_keys_local = self.reserve_temp_local();
        let enumerable_keys_tag_local = self.reserve_temp_local();
        let enumerable_keys_len_local = self.reserve_temp_local();
        let own_names_local = self.reserve_temp_local();
        let own_names_tag_local = self.reserve_temp_local();
        let own_names_len_local = self.reserve_temp_local();
        let visited_array_local = self.reserve_temp_local();
        let visited_tag_local = self.reserve_temp_local();
        let result_len_local = self.reserve_temp_local();
        let visited_len_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_len_local));
        self.emit_alloc_array_payload_with_length(result_len_local, result_array_local, function)?;
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(result_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(visited_len_local));
        self.emit_alloc_array_payload_with_length(
            visited_len_local,
            visited_array_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(visited_tag_local));

        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalSet(current_local));
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::LocalSet(current_tag_local));
        let prototype_break_frame = self.open_frame(ControlFrameKind::Block, function);
        let prototype_loop_frame = self.open_frame(ControlFrameKind::Loop, function);
        function.instruction(&Instruction::LocalGet(current_local));
        function.instruction(&Instruction::I64Eqz);
        function.branch_if_to_label(prototype_break_frame.label);

        self.emit_direct_js_call(
            &keys_meta,
            None,
            &[(current_local, current_tag_local)],
            enumerable_keys_local,
            enumerable_keys_tag_local,
            function,
        )?;
        self.emit_propagate_current_completion_if_throw(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(enumerable_keys_len_local));
        function.instruction(&Instruction::LocalGet(enumerable_keys_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            enumerable_keys_local,
            HEAP_LEN_OFFSET,
            enumerable_keys_len_local,
            function,
        );
        function.instruction(&Instruction::End);
        self.emit_for_in_append_unvisited_string_keys(
            enumerable_keys_local,
            enumerable_keys_len_local,
            visited_array_local,
            visited_len_local,
            result_array_local,
            result_len_local,
            function,
        )?;

        self.emit_direct_js_call(
            &own_names_meta,
            None,
            &[(current_local, current_tag_local)],
            own_names_local,
            own_names_tag_local,
            function,
        )?;
        self.emit_propagate_current_completion_if_throw(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(own_names_len_local));
        function.instruction(&Instruction::LocalGet(own_names_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            own_names_local,
            HEAP_LEN_OFFSET,
            own_names_len_local,
            function,
        );
        function.instruction(&Instruction::End);
        self.emit_for_in_mark_visited_string_keys(
            own_names_local,
            own_names_len_local,
            visited_array_local,
            visited_len_local,
            function,
        )?;

        self.load_i64_to_local_from_offset(
            current_local,
            HEAP_PROTOTYPE_OFFSET,
            current_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(current_tag_local));
        function.branch_to_label(prototype_loop_frame.label);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

        self.release_temp_local(visited_len_local);
        self.release_temp_local(result_len_local);
        self.release_temp_local(visited_tag_local);
        self.release_temp_local(visited_array_local);
        self.release_temp_local(own_names_len_local);
        self.release_temp_local(own_names_tag_local);
        self.release_temp_local(own_names_local);
        self.release_temp_local(enumerable_keys_len_local);
        self.release_temp_local(enumerable_keys_tag_local);
        self.release_temp_local(enumerable_keys_local);
        self.release_temp_local(current_tag_local);
        self.release_temp_local(current_local);
        Ok(())
    }

    pub(crate) fn compile_for_in_object(
        &mut self,
        mode: BindingMode,
        name: &str,
        target: &TypedExpr,
        body: &StatementIr,
        lexical_environment: Option<&ForInOfEnvironmentIr>,
        labels: &[String],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object_local = self.reserve_temp_local();
        let object_tag_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let entry_tag_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();
        let should_iterate_local = self.reserve_temp_local();

        if let Some(environment) = lexical_environment {
            self.emit_enter_for_in_of_tdz_scope(mode, environment, function)?;
        }
        self.compile_expr_to_locals(target, object_local, object_tag_local, function)?;
        if let Some(environment) = lexical_environment {
            self.emit_leave_for_in_of_tdz_scope(environment, function);
        }

        self.push_scope();
        let storage_without_environment = if mode == BindingMode::Var {
            Some(self.lookup_binding(name).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in lila wasm-aot first slice: unbound for-in var `{name}`"
                ))
            })?)
        } else if !iteration_environment_owns_binding(lexical_environment, name) {
            Some(self.allocate_binding(name.to_string(), mode, ValueKind::String))
        } else {
            None
        };
        if mode == BindingMode::Var {
            self.binding_scopes
                .last_mut()
                .expect("binding scope stack must exist")
                .insert(
                    name.to_string(),
                    storage_without_environment.expect("for-in var storage must exist"),
                );
        }
        self.emit_statement_result(function, ValueKind::Undefined);
        if target.kind != ValueKind::Dynamic {
            self.emit_for_in_object_key_snapshot(
                object_local,
                object_tag_local,
                buffer_local,
                entry_tag_local,
                function,
            )?;
            self.emit_propagate_current_completion_if_throw(function);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(len_local));
            function.instruction(&Instruction::LocalGet(entry_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.load_i64_to_local_from_offset(buffer_local, HEAP_LEN_OFFSET, len_local, function);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(index_local));
            let break_frame = self.open_frame(ControlFrameKind::Block, function);
            self.breakable_stack.push(break_frame);
            let loop_frame = self.open_frame(ControlFrameKind::Loop, function);
            function.instruction(&Instruction::LocalGet(index_local));
            function.instruction(&Instruction::LocalGet(len_local));
            function.instruction(&Instruction::I64GeU);
            function.branch_if_to_label(break_frame.label);
            self.emit_array_read(
                buffer_local,
                index_local,
                key_payload_local,
                key_tag_local,
                function,
            );
            if let Some(environment) = lexical_environment
                .and_then(|environment| environment.iteration_environment.as_ref())
            {
                self.emit_enter_lexical_environment(environment, function)?;
            }
            let storage = self
                .lookup_current_scope_binding(name)
                .or(storage_without_environment)
                .expect("for-in lexical storage must be allocated before assignment");
            self.write_binding_from_locals(storage, key_payload_local, key_tag_local, function);
            self.mirror_binding_to_global_object(name, storage, function)?;
            let continue_frame = self.open_frame(ControlFrameKind::Block, function);
            self.loop_stack.push(LoopTargets { continue_frame });
            self.push_labels(labels, break_frame, Some(continue_frame));
            self.compile_statement(body, function)?;
            self.pop_labels(labels.len());
            self.loop_stack.pop();
            self.pop_control(ControlFrameKind::Block);
            function.instruction(&Instruction::End);
            if lexical_environment
                .and_then(|environment| environment.iteration_environment.as_ref())
                .is_some()
            {
                self.emit_leave_lexical_environment(function);
            }
            function.instruction(&Instruction::LocalGet(index_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(index_local));
            function.branch_to_label(loop_frame.label);
            self.pop_control(ControlFrameKind::Loop);
            function.instruction(&Instruction::End);
            self.breakable_stack.pop();
            self.pop_control(ControlFrameKind::Block);
            function.instruction(&Instruction::End);
            self.pop_scope();
            self.release_temp_local(should_iterate_local);
            self.release_temp_local(index_number_payload_local);
            self.release_temp_local(key_tag_local);
            self.release_temp_local(key_payload_local);
            self.release_temp_local(descriptor_kind_local);
            self.release_temp_local(entry_tag_local);
            self.release_temp_local(entry_local);
            self.release_temp_local(index_local);
            self.release_temp_local(len_local);
            self.release_temp_local(buffer_local);
            self.release_temp_local(object_tag_local);
            self.release_temp_local(object_local);
            return Ok(());
        }
        if target.kind == ValueKind::Dynamic {
            self.emit_is_heap_object_like_tag_i32(object_tag_local, function);
            self.open_frame(ControlFrameKind::If, function);
        }
        self.load_i64_to_local_from_offset(object_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(object_local, HEAP_LEN_OFFSET, len_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        let break_frame = self.open_frame(ControlFrameKind::Block, function);
        self.breakable_stack.push(break_frame);
        let loop_frame = self.open_frame(ControlFrameKind::Loop, function);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.branch_if_to_label(break_frame.label);
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        if target.kind == ValueKind::Dynamic {
            function.instruction(&Instruction::LocalGet(object_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
            function.instruction(&Instruction::End);
        } else {
            function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        }
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(should_iterate_local));
        if target.kind == ValueKind::Dynamic {
            function.instruction(&Instruction::LocalGet(object_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.load_i64_to_local_from_offset(
                entry_local,
                HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
                descriptor_kind_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(descriptor_kind_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(descriptor_kind_local));
            function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ENUMERABLE as i64));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(should_iterate_local));
            function.instruction(&Instruction::LocalGet(index_local));
            function.instruction(&Instruction::F64ConvertI64U);
            function.instruction(&Instruction::I64ReinterpretF64);
            function.instruction(&Instruction::LocalSet(index_number_payload_local));
            self.emit_number_to_string_payload(index_number_payload_local, function)?;
            function.instruction(&Instruction::LocalSet(key_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
            function.instruction(&Instruction::LocalSet(key_tag_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::Else);
        }
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ENUMERABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(should_iterate_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            key_payload_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        function.instruction(&Instruction::End);
        if target.kind == ValueKind::Dynamic {
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(should_iterate_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        self.open_frame(ControlFrameKind::If, function);
        if let Some(environment) =
            lexical_environment.and_then(|environment| environment.iteration_environment.as_ref())
        {
            self.emit_enter_lexical_environment(environment, function)?;
        }
        let storage = self
            .lookup_current_scope_binding(name)
            .or(storage_without_environment)
            .expect("for-in lexical storage must be allocated before assignment");
        self.write_binding_from_locals(storage, key_payload_local, key_tag_local, function);
        self.mirror_binding_to_global_object(name, storage, function)?;
        let continue_frame = self.open_frame(ControlFrameKind::Block, function);
        self.loop_stack.push(LoopTargets { continue_frame });
        self.push_labels(labels, break_frame, Some(continue_frame));
        self.compile_statement(body, function)?;
        self.pop_labels(labels.len());
        self.loop_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        if lexical_environment
            .and_then(|environment| environment.iteration_environment.as_ref())
            .is_some()
        {
            self.emit_leave_lexical_environment(function);
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.branch_to_label(loop_frame.label);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.breakable_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        if target.kind == ValueKind::Dynamic {
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        self.pop_scope();
        self.release_temp_local(should_iterate_local);
        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_tag_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(object_tag_local);
        self.release_temp_local(object_local);
        Ok(())
    }
}
