use super::super::*;
use crate::functions::FunctionRealmRevokedRoute;

const HEAP_PROMISE_RESOLVING_CONTEXT_SIZE: u64 = 24;
const HEAP_PROMISE_RESOLVING_CONTEXT_RECORD_OFFSET: u64 = 0;
const HEAP_PROMISE_RESOLVING_CONTEXT_ALREADY_RESOLVED_OFFSET: u64 = 8;
const HEAP_PROMISE_RESOLVING_CONTEXT_PAYLOAD_OFFSET: u64 = 16;
const HEAP_PROMISE_THENABLE_JOB_RECORD_SIZE: u64 = 48;
const HEAP_PROMISE_THENABLE_JOB_PROMISE_RECORD_OFFSET: u64 = 0;
const HEAP_PROMISE_THENABLE_JOB_PROMISE_PAYLOAD_OFFSET: u64 = 8;
const HEAP_PROMISE_THENABLE_JOB_THENABLE_PAYLOAD_OFFSET: u64 = 16;
const HEAP_PROMISE_THENABLE_JOB_THENABLE_TAG_OFFSET: u64 = 24;
const HEAP_PROMISE_THENABLE_JOB_THEN_PAYLOAD_OFFSET: u64 = 32;
const HEAP_PROMISE_THENABLE_JOB_THEN_TAG_OFFSET: u64 = 40;
const HEAP_PROMISE_ALL_SHARED_CONTEXT_SIZE: u64 = 32;
const HEAP_PROMISE_ALL_SHARED_REMAINING_OFFSET: u64 = 0;
const HEAP_PROMISE_ALL_SHARED_VALUES_OFFSET: u64 = 8;
const HEAP_PROMISE_ALL_SHARED_RESOLVE_PAYLOAD_OFFSET: u64 = 16;
const HEAP_PROMISE_ALL_SHARED_RESOLVE_TAG_OFFSET: u64 = 24;
const HEAP_PROMISE_ALL_ELEMENT_CONTEXT_SIZE: u64 = 24;
const HEAP_PROMISE_ALL_ELEMENT_INDEX_OFFSET: u64 = 0;
const HEAP_PROMISE_ALL_ELEMENT_SHARED_OFFSET: u64 = 8;
const HEAP_PROMISE_ALL_ELEMENT_ALREADY_CALLED_OFFSET: u64 = 16;
const HEAP_PROMISE_KEYED_ELEMENT_CONTEXT_SIZE: u64 = 32;
const HEAP_PROMISE_KEYED_ELEMENT_KEY_PAYLOAD_OFFSET: u64 = 0;
const HEAP_PROMISE_KEYED_ELEMENT_KEY_TAG_OFFSET: u64 = 8;
const HEAP_PROMISE_KEYED_ELEMENT_SHARED_OFFSET: u64 = 16;
const HEAP_PROMISE_KEYED_ELEMENT_ALREADY_CALLED_OFFSET: u64 = 24;
const HEAP_PROMISE_FINALLY_CONTEXT_SIZE: u64 = 32;
const HEAP_PROMISE_FINALLY_ON_FINALLY_PAYLOAD_OFFSET: u64 = 0;
const HEAP_PROMISE_FINALLY_ON_FINALLY_TAG_OFFSET: u64 = 8;
const HEAP_PROMISE_FINALLY_CONSTRUCTOR_PAYLOAD_OFFSET: u64 = 16;
const HEAP_PROMISE_FINALLY_CONSTRUCTOR_TAG_OFFSET: u64 = 24;
const HEAP_PROMISE_FINALLY_VALUE_CONTEXT_SIZE: u64 = 16;
const HEAP_PROMISE_FINALLY_VALUE_PAYLOAD_OFFSET: u64 = 0;
const HEAP_PROMISE_FINALLY_VALUE_TAG_OFFSET: u64 = 8;

/// A fully selected pending-job payload. The only queue append function
/// accepts this type, so every job shape must provide its argument and realm
/// policy before it can enter the shared FIFO.
#[derive(Clone, Copy)]
enum PromiseJobToEnqueue {
    Reaction {
        reaction_record_local: u32,
        argument_payload_local: u32,
        argument_tag_local: u32,
    },
    ResolveThenable {
        thenable_job_local: u32,
        then_payload_local: u32,
        then_tag_local: u32,
    },
}

#[derive(Clone, Copy)]
enum AsyncAwaitContinuation {
    AsyncFunction,
    AsyncGeneratorBody,
    AsyncGeneratorYield,
    AsyncGeneratorYieldReturn,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PromiseCombinatorMode {
    Values,
    SettledRecords,
    FirstFulfillment,
}

impl PromiseCombinatorMode {
    const fn builtin_name(self) -> &'static str {
        match self {
            Self::Values => "Promise.all",
            Self::SettledRecords => "Promise.allSettled",
            Self::FirstFulfillment => "Promise.any",
        }
    }
}

impl AsyncAwaitContinuation {
    fn reaction_callback_kind(self) -> PromiseReactionCallbackKind {
        match self {
            Self::AsyncFunction => PromiseReactionCallbackKind::AsyncFunction,
            Self::AsyncGeneratorBody => PromiseReactionCallbackKind::AsyncGeneratorAwait,
            Self::AsyncGeneratorYield => PromiseReactionCallbackKind::AsyncGeneratorYield,
            Self::AsyncGeneratorYieldReturn => {
                PromiseReactionCallbackKind::AsyncGeneratorYieldReturn
            }
        }
    }
}
impl<'a> FunctionBuilder<'a> {
    pub(crate) fn emit_alloc_promise_with_prototype(
        &mut self,
        prototype_payload_local: u32,
        promise_payload_local: u32,
        promise_record_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_alloc_plain_object_with_prototype(Some(prototype_payload_local), None, function)?;
        function.instruction(&Instruction::LocalSet(promise_payload_local));
        self.emit_heap_alloc_const(HEAP_PROMISE_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(promise_record_local));
        self.store_i64_const_at_offset(
            promise_record_local,
            HEAP_PROMISE_STATE_OFFSET,
            PROMISE_STATE_PENDING,
            function,
        );
        self.store_i64_const_at_offset(
            promise_record_local,
            HEAP_PROMISE_RESULT_TAG_OFFSET,
            ValueKind::Undefined.tag() as u64,
            function,
        );
        for offset in [
            HEAP_PROMISE_RESULT_PAYLOAD_OFFSET,
            HEAP_PROMISE_FULFILL_REACTIONS_OFFSET,
            HEAP_PROMISE_REJECT_REACTIONS_OFFSET,
            HEAP_PROMISE_IS_HANDLED_OFFSET,
            HEAP_PROMISE_HOST_DATA_OFFSET,
            HEAP_PROMISE_UNHANDLED_NEXT_OFFSET,
        ] {
            self.store_i64_const_at_offset(promise_record_local, offset, 0, function);
        }
        function.instruction(&Instruction::GlobalGet(CURRENT_REALM_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.store_i64_local_at_offset(
            promise_record_local,
            HEAP_PROMISE_REALM_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_const_at_offset(
            promise_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            OBJECT_INTERNAL_BRAND_PROMISE,
            function,
        );
        self.store_i64_const_at_offset(
            promise_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            BOXED_PRIMITIVE_KIND_NONE,
            function,
        );
        self.store_i64_const_at_offset(
            promise_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            ValueKind::Object.tag() as u64,
            function,
        );
        self.store_i64_local_at_offset(
            promise_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            promise_record_local,
            function,
        );
        Ok(())
    }

    fn emit_create_promise_resolving_functions(
        &mut self,
        promise_payload_local: u32,
        promise_record_local: u32,
        resolving_context_local: u32,
        resolve_function_local: u32,
        reject_function_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_heap_alloc_const(HEAP_PROMISE_RESOLVING_CONTEXT_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(resolving_context_local));
        self.store_i64_local_at_offset(
            resolving_context_local,
            HEAP_PROMISE_RESOLVING_CONTEXT_RECORD_OFFSET,
            promise_record_local,
            function,
        );
        self.store_i64_const_at_offset(
            resolving_context_local,
            HEAP_PROMISE_RESOLVING_CONTEXT_ALREADY_RESOLVED_OFFSET,
            0,
            function,
        );
        self.store_i64_local_at_offset(
            resolving_context_local,
            HEAP_PROMISE_RESOLVING_CONTEXT_PAYLOAD_OFFSET,
            promise_payload_local,
            function,
        );

        for (builtin, resolving_function_local) in [
            (
                StandardBuiltinId::PromiseResolveFunction,
                resolve_function_local,
            ),
            (
                StandardBuiltinId::PromiseRejectFunction,
                reject_function_local,
            ),
        ] {
            let meta = self
                .functions
                .get(&builtin.function_id())
                .cloned()
                .ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in lila wasm-aot first slice: missing builtin meta `{}`",
                        builtin.debug_name()
                    ))
                })?;
            self.emit_function_value_payload(&meta, function)?;
            function.instruction(&Instruction::LocalSet(resolving_function_local));
            self.store_i64_local_at_offset(
                resolving_function_local,
                HEAP_FUNCTION_ENV_HANDLE_OFFSET,
                resolving_context_local,
                function,
            );
        }
        Ok(())
    }

    fn emit_enqueue_promise_reaction_job(
        &mut self,
        reaction_record_local: u32,
        argument_payload_local: u32,
        argument_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_enqueue_promise_job(
            PromiseJobToEnqueue::Reaction {
                reaction_record_local,
                argument_payload_local,
                argument_tag_local,
            },
            function,
        )
    }

    fn emit_enqueue_promise_reaction_list(
        &mut self,
        reaction_list_local: u32,
        argument_payload_local: u32,
        argument_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let reaction_record_local = self.reserve_temp_local();
        let next_reaction_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(reaction_list_local));
        function.instruction(&Instruction::LocalSet(reaction_record_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(reaction_record_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        self.load_i64_to_local_from_offset(
            reaction_record_local,
            HEAP_PROMISE_REACTION_NEXT_OFFSET,
            next_reaction_local,
            function,
        );
        self.emit_enqueue_promise_reaction_job(
            reaction_record_local,
            argument_payload_local,
            argument_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(next_reaction_local));
        function.instruction(&Instruction::LocalSet(reaction_record_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(next_reaction_local);
        self.release_temp_local(reaction_record_local);
        Ok(())
    }

    fn emit_enqueue_promise_thenable_job(
        &mut self,
        promise_payload_local: u32,
        promise_record_local: u32,
        thenable_payload_local: u32,
        thenable_tag_local: u32,
        then_payload_local: u32,
        then_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let thenable_job_local = self.reserve_temp_local();

        self.emit_heap_alloc_const(HEAP_PROMISE_THENABLE_JOB_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(thenable_job_local));
        for (offset, value_local) in [
            (
                HEAP_PROMISE_THENABLE_JOB_PROMISE_RECORD_OFFSET,
                promise_record_local,
            ),
            (
                HEAP_PROMISE_THENABLE_JOB_PROMISE_PAYLOAD_OFFSET,
                promise_payload_local,
            ),
            (
                HEAP_PROMISE_THENABLE_JOB_THENABLE_PAYLOAD_OFFSET,
                thenable_payload_local,
            ),
            (
                HEAP_PROMISE_THENABLE_JOB_THENABLE_TAG_OFFSET,
                thenable_tag_local,
            ),
            (
                HEAP_PROMISE_THENABLE_JOB_THEN_PAYLOAD_OFFSET,
                then_payload_local,
            ),
            (HEAP_PROMISE_THENABLE_JOB_THEN_TAG_OFFSET, then_tag_local),
        ] {
            self.store_i64_local_at_offset(thenable_job_local, offset, value_local, function);
        }

        let result = self.emit_enqueue_promise_job(
            PromiseJobToEnqueue::ResolveThenable {
                thenable_job_local,
                then_payload_local,
                then_tag_local,
            },
            function,
        );
        self.release_temp_local(thenable_job_local);
        result
    }

    fn emit_promise_job_callback_realm_to_local(
        &mut self,
        callback_payload_local: u32,
        callback_tag_local: u32,
        realm_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let realm_result =
            self.emit_get_function_realm(callback_payload_local, callback_tag_local, function);
        let resolved_realm = self.emit_route_function_realm_result(
            realm_result,
            FunctionRealmRevokedRoute::UseCurrentRealm,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(resolved_realm.index()));
        function.instruction(&Instruction::LocalSet(realm_local));
        self.release_resolved_function_realm_local(resolved_realm);
        Ok(())
    }

    fn emit_promise_reaction_job_realm_to_local(
        &mut self,
        reaction_record_local: u32,
        realm_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let callback_kind_local = self.reserve_temp_local();
        let handler_payload_local = self.reserve_temp_local();
        let handler_tag_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            reaction_record_local,
            HEAP_PROMISE_REACTION_CALLBACK_KIND_OFFSET,
            callback_kind_local,
            function,
        );
        let mut open_dispatch_arms = 0;
        for kind in PromiseReactionCallbackKind::ALL {
            function.instruction(&Instruction::LocalGet(callback_kind_local));
            function.instruction(&Instruction::I64Const(kind.word() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            match kind.realm_source() {
                PromiseReactionRealmSource::HandlerOrNull => {
                    // An empty handler carries the spec's null realm. The job
                    // drain maps that sentinel to its saved checkpoint realm
                    // instead of installing an invalid realm record.
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(realm_local));
                    self.load_i64_to_local_from_offset(
                        reaction_record_local,
                        HEAP_PROMISE_REACTION_HANDLER_PAYLOAD_OFFSET,
                        handler_payload_local,
                        function,
                    );
                    self.load_i64_to_local_from_offset(
                        reaction_record_local,
                        HEAP_PROMISE_REACTION_HANDLER_TAG_OFFSET,
                        handler_tag_local,
                        function,
                    );
                    self.emit_is_callable_i32(handler_tag_local, handler_payload_local, function)?;
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_promise_job_callback_realm_to_local(
                        handler_payload_local,
                        handler_tag_local,
                        realm_local,
                        function,
                    )?;
                    function.instruction(&Instruction::End);
                }
                PromiseReactionRealmSource::Captured => {
                    self.load_i64_to_local_from_offset(
                        reaction_record_local,
                        HEAP_PROMISE_REACTION_REALM_OFFSET,
                        realm_local,
                        function,
                    );
                }
            }
            function.instruction(&Instruction::Else);
            open_dispatch_arms += 1;
        }
        function.instruction(&Instruction::Unreachable);
        for _ in 0..open_dispatch_arms {
            function.instruction(&Instruction::End);
        }

        self.release_temp_local(handler_tag_local);
        self.release_temp_local(handler_payload_local);
        self.release_temp_local(callback_kind_local);
        Ok(())
    }

    fn emit_enqueue_promise_job(
        &mut self,
        job: PromiseJobToEnqueue,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let job_record_local = self.reserve_temp_local();
        let queue_tail_local = self.reserve_temp_local();
        let realm_local = self.reserve_temp_local();

        self.emit_heap_alloc_const(HEAP_PENDING_JOB_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(job_record_local));
        self.store_i64_const_at_offset(
            job_record_local,
            HEAP_PENDING_JOB_CALLBACK_TAG_OFFSET,
            ValueKind::Object.tag() as u64,
            function,
        );

        let kind = match job {
            PromiseJobToEnqueue::Reaction {
                reaction_record_local,
                argument_payload_local,
                argument_tag_local,
            } => {
                self.store_i64_local_at_offset(
                    job_record_local,
                    HEAP_PENDING_JOB_CALLBACK_PAYLOAD_OFFSET,
                    reaction_record_local,
                    function,
                );
                self.store_i64_local_at_offset(
                    job_record_local,
                    HEAP_PENDING_JOB_ARG_TAG_OFFSET,
                    argument_tag_local,
                    function,
                );
                self.store_i64_local_at_offset(
                    job_record_local,
                    HEAP_PENDING_JOB_ARG_PAYLOAD_OFFSET,
                    argument_payload_local,
                    function,
                );
                self.emit_promise_reaction_job_realm_to_local(
                    reaction_record_local,
                    realm_local,
                    function,
                )?;
                PromiseJobKind::Reaction
            }
            PromiseJobToEnqueue::ResolveThenable {
                thenable_job_local,
                then_payload_local,
                then_tag_local,
            } => {
                self.store_i64_local_at_offset(
                    job_record_local,
                    HEAP_PENDING_JOB_CALLBACK_PAYLOAD_OFFSET,
                    thenable_job_local,
                    function,
                );
                self.store_i64_const_at_offset(
                    job_record_local,
                    HEAP_PENDING_JOB_ARG_TAG_OFFSET,
                    ValueKind::Undefined.tag() as u64,
                    function,
                );
                self.store_i64_const_at_offset(
                    job_record_local,
                    HEAP_PENDING_JOB_ARG_PAYLOAD_OFFSET,
                    0,
                    function,
                );
                self.emit_promise_job_callback_realm_to_local(
                    then_payload_local,
                    then_tag_local,
                    realm_local,
                    function,
                )?;
                PromiseJobKind::ResolveThenable
            }
        };

        self.store_i64_local_at_offset(
            job_record_local,
            HEAP_PENDING_JOB_REALM_OFFSET,
            realm_local,
            function,
        );
        self.store_i64_const_at_offset(job_record_local, HEAP_PENDING_JOB_NEXT_OFFSET, 0, function);
        self.store_i64_const_at_offset(
            job_record_local,
            HEAP_PENDING_JOB_KIND_OFFSET,
            kind.word(),
            function,
        );

        // This is the sole Promise-job FIFO append. A new payload variant can
        // select its record fields above, but cannot bypass queue ordering.
        function.instruction(&Instruction::GlobalGet(PROMISE_JOB_QUEUE_TAIL_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(queue_tail_local));
        function.instruction(&Instruction::LocalGet(queue_tail_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(job_record_local));
        function.instruction(&Instruction::GlobalSet(PROMISE_JOB_QUEUE_HEAD_GLOBAL_INDEX));
        function.instruction(&Instruction::Else);
        self.store_i64_local_at_offset(
            queue_tail_local,
            HEAP_PENDING_JOB_NEXT_OFFSET,
            job_record_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(job_record_local));
        function.instruction(&Instruction::GlobalSet(PROMISE_JOB_QUEUE_TAIL_GLOBAL_INDEX));

        self.release_temp_local(realm_local);
        self.release_temp_local(queue_tail_local);
        self.release_temp_local(job_record_local);
        Ok(())
    }

    pub(crate) fn emit_resolve_promise_record(
        &mut self,
        promise_payload_local: u32,
        promise_record_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let then_key_local = self.reserve_temp_local();
        let then_payload_local = self.reserve_temp_local();
        let then_tag_local = self.reserve_temp_local();

        self.emit_is_heap_object_like_tag_i32(value_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::LocalGet(promise_payload_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Promise cannot resolve to itself",
            then_payload_local,
            then_tag_local,
            function,
        )?;
        self.emit_settle_promise_record(
            promise_record_local,
            PROMISE_STATE_REJECTED,
            then_payload_local,
            then_tag_local,
            function,
        )?;
        self.set_completion_kind(CompletionKind::Normal, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("then")));
        function.instruction(&Instruction::LocalSet(then_key_local));
        self.emit_object_read_without_throw_propagation(
            value_payload_local,
            value_tag_local,
            value_payload_local,
            value_tag_local,
            then_key_local,
            then_payload_local,
            then_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_settle_promise_record(
            promise_record_local,
            PROMISE_STATE_REJECTED,
            then_payload_local,
            then_tag_local,
            function,
        )?;
        self.set_completion_kind(CompletionKind::Normal, function);
        function.instruction(&Instruction::Else);
        self.emit_is_callable_i32(then_tag_local, then_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_enqueue_promise_thenable_job(
            promise_payload_local,
            promise_record_local,
            value_payload_local,
            value_tag_local,
            then_payload_local,
            then_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_settle_promise_record(
            promise_record_local,
            PROMISE_STATE_FULFILLED,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_settle_promise_record(
            promise_record_local,
            PROMISE_STATE_FULFILLED,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.release_temp_local(then_tag_local);
        self.release_temp_local(then_payload_local);
        self.release_temp_local(then_key_local);
        Ok(())
    }

    pub(crate) fn emit_settle_promise_record(
        &mut self,
        promise_record_local: u32,
        state: u64,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let state_local = self.reserve_temp_local();
        let reaction_list_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            promise_record_local,
            HEAP_PROMISE_STATE_OFFSET,
            state_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(PROMISE_STATE_PENDING as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_const_at_offset(
            promise_record_local,
            HEAP_PROMISE_STATE_OFFSET,
            state,
            function,
        );
        self.store_i64_local_at_offset(
            promise_record_local,
            HEAP_PROMISE_RESULT_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            promise_record_local,
            HEAP_PROMISE_RESULT_TAG_OFFSET,
            value_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            promise_record_local,
            if state == PROMISE_STATE_FULFILLED {
                HEAP_PROMISE_FULFILL_REACTIONS_OFFSET
            } else {
                HEAP_PROMISE_REJECT_REACTIONS_OFFSET
            },
            reaction_list_local,
            function,
        );
        self.emit_enqueue_promise_reaction_list(
            reaction_list_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        if state == PROMISE_STATE_REJECTED {
            // 27.2.1.7 RejectPromise step 7: if [[IsHandled]] is false, notify
            // the host that a rejection went untracked.
            self.emit_track_unhandled_rejection(promise_record_local, function);
        }
        function.instruction(&Instruction::End);
        self.release_temp_local(reaction_list_local);
        self.release_temp_local(state_local);
        Ok(())
    }

    /// Appends `promise_record_local` to the host unhandled-rejection list when
    /// the promise still has no handler. Membership is only a candidate mark:
    /// `emit_report_unhandled_rejection` re-reads `[[IsHandled]]` after the job
    /// queue drains, so a handler attached from a later job clears the report.
    fn emit_track_unhandled_rejection(
        &mut self,
        promise_record_local: u32,
        function: &mut Function,
    ) {
        let is_handled_local = self.reserve_temp_local();
        let tail_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            promise_record_local,
            HEAP_PROMISE_IS_HANDLED_OFFSET,
            is_handled_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(is_handled_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_const_at_offset(
            promise_record_local,
            HEAP_PROMISE_UNHANDLED_NEXT_OFFSET,
            0,
            function,
        );
        function.instruction(&Instruction::GlobalGet(
            PROMISE_UNHANDLED_REJECTION_TAIL_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(tail_local));
        function.instruction(&Instruction::LocalGet(tail_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(promise_record_local));
        function.instruction(&Instruction::GlobalSet(
            PROMISE_UNHANDLED_REJECTION_HEAD_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::Else);
        self.store_i64_local_at_offset(
            tail_local,
            HEAP_PROMISE_UNHANDLED_NEXT_OFFSET,
            promise_record_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(promise_record_local));
        function.instruction(&Instruction::GlobalSet(
            PROMISE_UNHANDLED_REJECTION_TAIL_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::End);

        self.release_temp_local(tail_local);
        self.release_temp_local(is_handled_local);
    }

    /// Turns the oldest still-unhandled rejection into a throw completion for
    /// the main export. Runs once, after the job queue has drained, so every
    /// handler that a job could still attach has already been attached.
    pub(crate) fn emit_report_unhandled_rejection(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let state_local = self.reserve_temp_local();
        let is_handled_local = self.reserve_temp_local();
        let found_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::GlobalGet(
            PROMISE_UNHANDLED_REJECTION_HEAD_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(record_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(record_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_PROMISE_STATE_OFFSET,
            state_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_PROMISE_IS_HANDLED_OFFSET,
            is_handled_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(PROMISE_STATE_REJECTED as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(is_handled_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(record_local));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_PROMISE_UNHANDLED_NEXT_OFFSET,
            record_local,
            function,
        );
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        // The list is consumed exactly once; drop it so nothing re-reports.
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::GlobalSet(
            PROMISE_UNHANDLED_REJECTION_HEAD_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::GlobalSet(
            PROMISE_UNHANDLED_REJECTION_TAIL_GLOBAL_INDEX,
        ));

        function.instruction(&Instruction::LocalGet(found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_NORMAL));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            found_local,
            HEAP_PROMISE_RESULT_PAYLOAD_OFFSET,
            self.result_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            found_local,
            HEAP_PROMISE_RESULT_TAG_OFFSET,
            self.result_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::GlobalSet(throw_error_name_global_index(
            self.uses_heap,
        )));
        self.emit_capture_throw_error_name(self.result_local, self.result_tag_local, function)?;
        self.set_completion_kind_with_aux(CompletionKind::Throw, -1, function);
        function.instruction(&Instruction::End);

        self.release_temp_local(found_local);
        self.release_temp_local(is_handled_local);
        self.release_temp_local(state_local);
        self.release_temp_local(record_local);
        Ok(())
    }

    pub(crate) fn emit_promise_constructor(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let executor_payload_local = self.reserve_temp_local();
        let executor_tag_local = self.reserve_temp_local();
        let new_target_payload_local = self.reserve_temp_local();
        let new_target_tag_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();
        let promise_payload_local = self.reserve_temp_local();
        let promise_record_local = self.reserve_temp_local();
        let resolving_context_local = self.reserve_temp_local();
        let resolve_function_local = self.reserve_temp_local();
        let reject_function_local = self.reserve_temp_local();
        let function_tag_local = self.reserve_temp_local();
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();
        let call_payload_local = self.reserve_temp_local();
        let call_tag_local = self.reserve_temp_local();
        let undefined_payload_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();

        self.compile_new_target_to_locals(
            new_target_payload_local,
            new_target_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(new_target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Promise constructor requires new",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_builtin_arg_to_locals(0, executor_payload_local, executor_tag_local, function);
        self.emit_is_callable_i32(executor_tag_local, executor_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Promise executor is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_error_new_target_prototype_to_local(
            PROMISE_PROTOTYPE_GLOBAL_INDEX,
            None,
            prototype_payload_local,
            function,
        )?;
        self.emit_alloc_promise_with_prototype(
            prototype_payload_local,
            promise_payload_local,
            promise_record_local,
            function,
        )?;

        self.emit_create_promise_resolving_functions(
            promise_payload_local,
            promise_record_local,
            resolving_context_local,
            resolve_function_local,
            reject_function_local,
            function,
        )?;

        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(function_tag_local));
        self.emit_pre_evaluated_arg_vector(
            &[
                (resolve_function_local, function_tag_local),
                (reject_function_local, function_tag_local),
            ],
            argc_local,
            argv_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));
        self.emit_function_handle_call_with_argv_without_throw_propagation(
            executor_payload_local,
            executor_tag_local,
            Some((undefined_payload_local, Some(undefined_tag_local))),
            argc_local,
            argv_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_handle_call_without_throw_propagation(
            reject_function_local,
            function_tag_local,
            Some((undefined_payload_local, Some(undefined_tag_local))),
            &[(call_payload_local, call_tag_local)],
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(promise_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);

        self.release_temp_local(undefined_tag_local);
        self.release_temp_local(undefined_payload_local);
        self.release_temp_local(call_tag_local);
        self.release_temp_local(call_payload_local);
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(function_tag_local);
        self.release_temp_local(reject_function_local);
        self.release_temp_local(resolve_function_local);
        self.release_temp_local(resolving_context_local);
        self.release_temp_local(promise_record_local);
        self.release_temp_local(promise_payload_local);
        self.release_temp_local(prototype_payload_local);
        self.release_temp_local(new_target_tag_local);
        self.release_temp_local(new_target_payload_local);
        self.release_temp_local(executor_tag_local);
        self.release_temp_local(executor_payload_local);
        Ok(())
    }

    fn emit_promise_species_constructor(
        &mut self,
        promise_payload_local: u32,
        promise_tag_local: u32,
        species_constructor_payload_local: u32,
        species_constructor_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let constructor_payload_local = self.reserve_temp_local();
        let constructor_tag_local = self.reserve_temp_local();
        let species_payload_local = self.reserve_temp_local();
        let species_tag_local = self.reserve_temp_local();
        let species_is_constructor_local = self.reserve_temp_local();

        function.instruction(&Instruction::GlobalGet(PROMISE_CONSTRUCTOR_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(species_constructor_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(species_constructor_tag_local));

        function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            promise_payload_local,
            promise_tag_local,
            promise_payload_local,
            promise_tag_local,
            key_local,
            constructor_payload_local,
            constructor_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);

        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_is_heap_object_like_tag_i32(constructor_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Promise constructor property is not an object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(
            self.strings.property_key_symbol_payload("Symbol.species"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            constructor_payload_local,
            constructor_tag_local,
            constructor_payload_local,
            constructor_tag_local,
            key_local,
            species_payload_local,
            species_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.compile_nullish_tagged_i32(species_tag_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_is_constructor_i32(species_tag_local, species_payload_local, function)?;
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(species_is_constructor_local));
        function.instruction(&Instruction::LocalGet(species_is_constructor_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Promise species is not a constructor",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(species_payload_local));
        function.instruction(&Instruction::LocalSet(species_constructor_payload_local));
        function.instruction(&Instruction::LocalGet(species_tag_local));
        function.instruction(&Instruction::LocalSet(species_constructor_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(species_is_constructor_local);
        self.release_temp_local(species_tag_local);
        self.release_temp_local(species_payload_local);
        self.release_temp_local(constructor_tag_local);
        self.release_temp_local(constructor_payload_local);
        self.release_temp_local(key_local);
        Ok(())
    }

    pub(crate) fn emit_new_promise_capability(
        &mut self,
        constructor_payload_local: u32,
        constructor_tag_local: u32,
        capability_record_local: u32,
        promise_payload_local: u32,
        promise_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let executor_payload_local = self.reserve_temp_local();
        let executor_tag_local = self.reserve_temp_local();
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();
        let resolve_payload_local = self.reserve_temp_local();
        let resolve_tag_local = self.reserve_temp_local();
        let reject_payload_local = self.reserve_temp_local();
        let reject_tag_local = self.reserve_temp_local();

        self.emit_is_constructor_i32(constructor_tag_local, constructor_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Promise capability constructor is not a constructor",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_heap_alloc_const(HEAP_PROMISE_CAPABILITY_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(capability_record_local));
        for tag_offset in [
            HEAP_PROMISE_CAPABILITY_PROMISE_TAG_OFFSET,
            HEAP_PROMISE_CAPABILITY_RESOLVE_TAG_OFFSET,
            HEAP_PROMISE_CAPABILITY_REJECT_TAG_OFFSET,
        ] {
            self.store_i64_const_at_offset(
                capability_record_local,
                tag_offset,
                ValueKind::Undefined.tag() as u64,
                function,
            );
        }
        for payload_offset in [
            HEAP_PROMISE_CAPABILITY_PROMISE_PAYLOAD_OFFSET,
            HEAP_PROMISE_CAPABILITY_RESOLVE_PAYLOAD_OFFSET,
            HEAP_PROMISE_CAPABILITY_REJECT_PAYLOAD_OFFSET,
        ] {
            self.store_i64_const_at_offset(capability_record_local, payload_offset, 0, function);
        }

        let executor_meta = self
            .functions
            .get(&StandardBuiltinId::PromiseCapabilityExecutor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing Promise capability executor builtin",
                )
            })?;
        self.emit_function_value_payload(&executor_meta, function)?;
        function.instruction(&Instruction::LocalSet(executor_payload_local));
        self.store_i64_local_at_offset(
            executor_payload_local,
            HEAP_FUNCTION_ENV_HANDLE_OFFSET,
            capability_record_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(executor_tag_local));
        self.emit_pre_evaluated_arg_vector(
            &[(executor_payload_local, executor_tag_local)],
            argc_local,
            argv_local,
            function,
        )?;
        self.emit_function_or_proxy_construct_with_argv(
            constructor_payload_local,
            constructor_tag_local,
            constructor_payload_local,
            constructor_tag_local,
            argc_local,
            argv_local,
            promise_payload_local,
            promise_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);

        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_RESOLVE_PAYLOAD_OFFSET,
            resolve_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_RESOLVE_TAG_OFFSET,
            resolve_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_REJECT_PAYLOAD_OFFSET,
            reject_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_REJECT_TAG_OFFSET,
            reject_tag_local,
            function,
        );
        self.emit_is_callable_i32(resolve_tag_local, resolve_payload_local, function)?;
        self.emit_is_callable_i32(reject_tag_local, reject_payload_local, function)?;
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Promise capability did not initialize callable resolving functions",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.store_i64_local_at_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_PROMISE_PAYLOAD_OFFSET,
            promise_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_PROMISE_TAG_OFFSET,
            promise_tag_local,
            function,
        );
        self.set_completion_kind(CompletionKind::Normal, function);

        self.release_temp_local(reject_tag_local);
        self.release_temp_local(reject_payload_local);
        self.release_temp_local(resolve_tag_local);
        self.release_temp_local(resolve_payload_local);
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(executor_tag_local);
        self.release_temp_local(executor_payload_local);
        Ok(())
    }

    fn emit_initialize_promise_reaction(
        &mut self,
        reaction_record_local: u32,
        capability_record_local: u32,
        handler_payload_local: u32,
        handler_tag_local: u32,
        reaction_type: PromiseReactionType,
        callback_kind: PromiseReactionCallbackKind,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_heap_alloc_const(HEAP_PROMISE_REACTION_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(reaction_record_local));
        self.store_i64_local_at_offset(
            reaction_record_local,
            HEAP_PROMISE_REACTION_CAPABILITY_OFFSET,
            capability_record_local,
            function,
        );
        self.store_i64_local_at_offset(
            reaction_record_local,
            HEAP_PROMISE_REACTION_HANDLER_TAG_OFFSET,
            handler_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            reaction_record_local,
            HEAP_PROMISE_REACTION_HANDLER_PAYLOAD_OFFSET,
            handler_payload_local,
            function,
        );
        match callback_kind.realm_source() {
            PromiseReactionRealmSource::HandlerOrNull => self.store_i64_const_at_offset(
                reaction_record_local,
                HEAP_PROMISE_REACTION_REALM_OFFSET,
                0,
                function,
            ),
            PromiseReactionRealmSource::Captured => {
                function.instruction(&Instruction::GlobalGet(CURRENT_REALM_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalSet(self.scratch_local));
                self.store_i64_local_at_offset(
                    reaction_record_local,
                    HEAP_PROMISE_REACTION_REALM_OFFSET,
                    self.scratch_local,
                    function,
                );
            }
        }
        self.store_i64_const_at_offset(
            reaction_record_local,
            HEAP_PROMISE_REACTION_NEXT_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(
            reaction_record_local,
            HEAP_PROMISE_REACTION_CALLBACK_KIND_OFFSET,
            callback_kind.word(),
            function,
        );
        self.store_i64_const_at_offset(
            reaction_record_local,
            HEAP_PROMISE_REACTION_TYPE_OFFSET,
            reaction_type.word(),
            function,
        );
        Ok(())
    }

    fn emit_append_promise_reaction(
        &mut self,
        promise_record_local: u32,
        reaction_list_offset: u64,
        reaction_record_local: u32,
        function: &mut Function,
    ) {
        let list_head_local = self.reserve_temp_local();
        let current_reaction_local = self.reserve_temp_local();
        let next_reaction_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            promise_record_local,
            reaction_list_offset,
            list_head_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(list_head_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_local_at_offset(
            promise_record_local,
            reaction_list_offset,
            reaction_record_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(list_head_local));
        function.instruction(&Instruction::LocalSet(current_reaction_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            current_reaction_local,
            HEAP_PROMISE_REACTION_NEXT_OFFSET,
            next_reaction_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(next_reaction_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_local_at_offset(
            current_reaction_local,
            HEAP_PROMISE_REACTION_NEXT_OFFSET,
            reaction_record_local,
            function,
        );
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(next_reaction_local));
        function.instruction(&Instruction::LocalSet(current_reaction_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(next_reaction_local);
        self.release_temp_local(current_reaction_local);
        self.release_temp_local(list_head_local);
    }

    fn emit_intrinsic_promise_resolve_to_locals(
        &mut self,
        value_payload_local: u32,
        value_tag_local: u32,
        promise_payload_local: u32,
        promise_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let resolve_payload_local = self.reserve_temp_local();
        let resolve_tag_local = self.reserve_temp_local();
        let constructor_payload_local = self.reserve_temp_local();
        let constructor_tag_local = self.reserve_temp_local();
        let resolve_meta = self
            .functions
            .get(&StandardBuiltinId::PromiseResolve.function_id())
            .cloned()
            .ok_or_else(|| EmitError::unsupported("missing intrinsic Promise.resolve builtin"))?;

        self.emit_function_value_payload(&resolve_meta, function)?;
        function.instruction(&Instruction::LocalSet(resolve_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(resolve_tag_local));
        function.instruction(&Instruction::GlobalGet(PROMISE_CONSTRUCTOR_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(constructor_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(constructor_tag_local));
        self.emit_function_handle_call_without_throw_propagation(
            resolve_payload_local,
            resolve_tag_local,
            Some((constructor_payload_local, Some(constructor_tag_local))),
            &[(value_payload_local, value_tag_local)],
            promise_payload_local,
            promise_tag_local,
            function,
        )?;

        self.release_temp_local(constructor_tag_local);
        self.release_temp_local(constructor_payload_local);
        self.release_temp_local(resolve_tag_local);
        self.release_temp_local(resolve_payload_local);
        Ok(())
    }

    /// `AsyncFromSyncIteratorContinuation` (27.1.4.4) steps 5 and 14 ONLY, for
    /// the `for await` path.
    ///
    /// Steps 6.a and 13 — the `IteratorClose` obligation — are NOT discharged
    /// here. On this path they are discharged separately by
    /// `compile_async_for_of_iterator` in `control_flow.rs`, off
    /// `close_on_rejection_storage`; the async-generator delegation path
    /// discharges them in `emit_async_from_sync_close_on_rejection` below,
    /// against a different guard.
    ///
    /// So one spec obligation has two independent implementations in this
    /// backend and they can drift. The delegation path's fixture
    /// (`wasm_async_from_sync_iterator_close_on_rejection.js`) covers an absent
    /// `return`, a non-callable `return`, a throwing `return`, `done: true` and
    /// `closeOnRejection === false`; nothing covers those five over a `for await`
    /// driver. Duplicating the fixture's cases over `for await` is the follow-up
    /// that would pin both continuations with one oracle.
    pub(crate) fn emit_async_from_sync_value_continuation(
        &mut self,
        value_payload_local: u32,
        value_tag_local: u32,
        continuation_payload_local: u32,
        continuation_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::LocalSet(continuation_payload_local));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::LocalSet(continuation_tag_local));
        Ok(())
    }

    /// `AsyncFromSyncIteratorContinuation` (27.1.4.4) steps 6.a and 13, for the
    /// async-generator delegation path.
    ///
    /// The spec splits one obligation across two steps that this backend
    /// reaches through a single edge:
    ///
    /// - step 6.a — `PromiseResolve(%Promise%, value)` abrupt-completes, so
    ///   `valueWrapper` is set to `IteratorClose(syncIteratorRecord, valueWrapper)`;
    /// - step 13 — the value wrapper settles as a rejection, so the `onRejected`
    ///   closure performs `IteratorClose(syncIteratorRecord, ThrowCompletion(error))`.
    ///
    /// They converge because `emit_intrinsic_await_reactions` turns an abrupt
    /// `PromiseResolve` into an already-rejected wrapper promise and then
    /// attaches the same reject reaction (see the `COMPLETION_KIND_THROW` arm
    /// there). So both spec steps arrive here, in the async-generator await
    /// job, as `resume_kind == ASYNC_GENERATOR_RESUME_KIND_REJECT` — and one
    /// emission discharges both.
    ///
    /// The three-part guard is the spec's, not a heuristic:
    ///
    /// - `[[AwaitingSyncValue]]` distinguishes *this* await from every other
    ///   await an async generator can be suspended in. It is set by
    ///   `compile_async_generator_delegation` immediately before it awaits the
    ///   value of a **sync** iterator's result, and it is the only state in
    ///   which an async-from-sync value wrapper is in flight. The flag is
    ///   *consumed* here — read, then cleared — so a close can happen at most
    ///   once per await, which is the `returnCount === 1` invariant expressed in
    ///   the state rather than in a test.
    /// - `done` is step 12's condition. `done === true` selects step 12, which
    ///   installs no `onRejected` at all, so a rejected wrapper must reject the
    ///   capability without closing. The stored `done` is the raw value read
    ///   from the iterator result, so it is coerced with `ToBoolean` exactly as
    ///   the delegation's own test does.
    /// - `closeOnRejection` is false for exactly one caller,
    ///   `%AsyncFromSyncIteratorPrototype%.return` (27.1.4.2.3), which this
    ///   backend reaches as a delegation resumed with
    ///   `ASYNC_GENERATOR_RESUME_KIND_RETURN`. There the sync `return` has
    ///   *already* been called to produce the result being unwrapped, so
    ///   closing again would call it twice — the double close that
    ///   `sameValue(returnCount, 1)` cannot see but a counting fixture can.
    ///
    /// `IteratorClose` is invoked with a throw completion, whose 7.4.9 shape is
    /// exactly what `emit_iterator_close_preserving_current_throw` emits: step 6
    /// returns the *original* completion before step 7 can consider the close's
    /// own result, so a `return` that throws, a `return` that is not callable,
    /// and a `return` that answers a non-object are all swallowed, while an
    /// absent or nullish `return` skips the call entirely. The rejection reason
    /// the generator is resumed with is therefore untouched by this emission.
    fn emit_async_from_sync_close_on_rejection(
        &mut self,
        activation_local: u32,
        resume_kind_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let delegate_record_local = self.reserve_temp_local();
        let awaiting_sync_value_local = self.reserve_temp_local();
        let pending_kind_local = self.reserve_temp_local();
        let done_payload_local = self.reserve_temp_local();
        let done_tag_local = self.reserve_temp_local();
        let close_iterator_payload_local = self.reserve_temp_local();
        let close_iterator_tag_local = self.reserve_temp_local();
        let close_key_local = self.reserve_temp_local();
        let close_return_payload_local = self.reserve_temp_local();
        let close_return_tag_local = self.reserve_temp_local();
        let close_result_payload_local = self.reserve_temp_local();
        let close_result_tag_local = self.reserve_temp_local();
        let close_saved_payload_local = self.reserve_temp_local();
        let close_saved_tag_local = self.reserve_temp_local();
        let close_saved_completion_local = self.reserve_temp_local();
        let close_saved_aux_local = self.reserve_temp_local();

        // Frame A: only a rejected await can owe an IteratorClose.
        function.instruction(&Instruction::LocalGet(resume_kind_local));
        function.instruction(&Instruction::I64Const(
            ASYNC_GENERATOR_RESUME_KIND_REJECT as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));

        // Frame B: a cheap pre-filter, and ONLY that. A non-zero record means a
        // delegation was entered — it does NOT mean one is still live. The four
        // sites that zero `HEAP_ASYNC_GENERATOR_DELEGATE_RECORD_OFFSET`
        // (`generator_delegation.rs`) are all reachable only from the
        // `resume_kind == FULFILL` arm and its `ForAwaitYield` sub-branches; the
        // `REJECT` arm returns without clearing. So the first time this emission
        // closes an iterator, the record stays non-zero for the rest of that
        // generator's life and every later rejecting await in the same
        // activation passes this frame.
        //
        // `[[AwaitingSyncValue]]` below is the real liveness test, and the
        // read-then-clear of it is load-bearing rather than tidy-up.
        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_DELEGATE_RECORD_OFFSET,
            delegate_record_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(delegate_record_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));

        self.load_i64_to_local_from_offset(
            delegate_record_local,
            HEAP_GENERATOR_DELEGATE_AWAITING_SYNC_VALUE_OFFSET,
            awaiting_sync_value_local,
            function,
        );
        // Consume the flag before deciding: the await it describes is over
        // either way, and clearing it here makes a second close structurally
        // unreachable rather than merely unreached.
        //
        // Why the flag cannot go stale, in full — the step that carries it is
        // the one a later editor needs and it is not the two clears. The flag is
        // set immediately before the await (`generator_delegation.rs`, the
        // `async_iterator == 0` arm) and it has exactly TWO clears: the
        // `resume_kind == FULFILL` arm, and this one. That is exhaustive
        // because a generator in `ASYNC_GENERATOR_STATE_SUSPENDED_AWAIT` cannot
        // be resumed by `.next()`/`.throw()`/`.return()` at all —
        // `builtins/standard.rs`'s `AsyncGeneratorPrototype{Next,Return,Throw}`
        // dispatch tests only `SUSPENDED_YIELD` and `SUSPENDED_START` — so every
        // resume that can observe the flag comes from the await job itself and
        // is FULFILL or REJECT.
        self.store_i64_const_at_offset(
            delegate_record_local,
            HEAP_GENERATOR_DELEGATE_AWAITING_SYNC_VALUE_OFFSET,
            0,
            function,
        );
        self.load_i64_to_local_from_offset(
            delegate_record_local,
            HEAP_GENERATOR_DELEGATE_PENDING_KIND_OFFSET,
            pending_kind_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            delegate_record_local,
            HEAP_GENERATOR_DELEGATE_RESULT_DONE_PAYLOAD_OFFSET,
            done_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            delegate_record_local,
            HEAP_GENERATOR_DELEGATE_RESULT_DONE_TAG_OFFSET,
            done_tag_local,
            function,
        );
        // Materialise `ToBoolean(done)` into a local before building the
        // condition, so no operand is left under the nested blocks
        // `compile_truthy_tagged_i32` opens.
        self.compile_truthy_tagged_i32(done_tag_local, done_payload_local, function)?;
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(done_payload_local));

        // Frame C: `[[AwaitingSyncValue]]` and `closeOnRejection` and `done is false`.
        //
        // Two of these three terms are NOT exercised by
        // `built-ins/AsyncFromSyncIteratorPrototype`. Counted over all 38 corpus
        // files: none pairs a rejecting value with `done: true`, and none pairs
        // a `.return()`-during-`yield*` with a rejecting value and a `return`
        // counter. Deleting the `pending_kind != RETURN` term or the
        // `ToBoolean(done) == false` term therefore keeps that node at 38/38
        // while silently closing a done-true iterator, or calling `return`
        // twice. Their only oracle is the counting fixture
        // `wasm_async_from_sync_iterator_close_on_rejection.js` (markers `f` and
        // `h`, both counts). Do not "simplify" this guard against a green node.
        function.instruction(&Instruction::LocalGet(awaiting_sync_value_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(pending_kind_local));
        function.instruction(&Instruction::I64Const(
            ASYNC_GENERATOR_RESUME_KIND_RETURN as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(done_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));

        self.load_i64_to_local_from_offset(
            delegate_record_local,
            HEAP_GENERATOR_DELEGATE_ITERATOR_PAYLOAD_OFFSET,
            close_iterator_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            delegate_record_local,
            HEAP_GENERATOR_DELEGATE_ITERATOR_TAG_OFFSET,
            close_iterator_tag_local,
            function,
        );
        self.emit_iterator_close_preserving_current_throw(
            IteratorCloseOnThrowLocals {
                iterator_payload_local: close_iterator_payload_local,
                iterator_tag_local: close_iterator_tag_local,
                key_local: close_key_local,
                return_payload_local: close_return_payload_local,
                return_tag_local: close_return_tag_local,
                result_payload_local: close_result_payload_local,
                result_tag_local: close_result_tag_local,
                saved_payload_local: close_saved_payload_local,
                saved_tag_local: close_saved_tag_local,
                saved_completion_local: close_saved_completion_local,
                saved_aux_local: close_saved_aux_local,
            },
            function,
        )?;

        function.instruction(&Instruction::End); // frame C
        function.instruction(&Instruction::End); // frame B
        function.instruction(&Instruction::End); // frame A

        self.release_temp_local(close_saved_aux_local);
        self.release_temp_local(close_saved_completion_local);
        self.release_temp_local(close_saved_tag_local);
        self.release_temp_local(close_saved_payload_local);
        self.release_temp_local(close_result_tag_local);
        self.release_temp_local(close_result_payload_local);
        self.release_temp_local(close_return_tag_local);
        self.release_temp_local(close_return_payload_local);
        self.release_temp_local(close_key_local);
        self.release_temp_local(close_iterator_tag_local);
        self.release_temp_local(close_iterator_payload_local);
        self.release_temp_local(done_tag_local);
        self.release_temp_local(done_payload_local);
        self.release_temp_local(pending_kind_local);
        self.release_temp_local(awaiting_sync_value_local);
        self.release_temp_local(delegate_record_local);
        Ok(())
    }

    pub(crate) fn emit_async_await_reactions(
        &mut self,
        activation_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_await_reactions(
            activation_local,
            value_payload_local,
            value_tag_local,
            AsyncAwaitContinuation::AsyncFunction,
            function,
        )
    }

    pub(crate) fn emit_async_generator_await_reactions(
        &mut self,
        activation_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_await_reactions(
            activation_local,
            value_payload_local,
            value_tag_local,
            AsyncAwaitContinuation::AsyncGeneratorBody,
            function,
        )
    }

    pub(crate) fn emit_async_generator_yield_reactions(
        &mut self,
        activation_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_await_reactions(
            activation_local,
            value_payload_local,
            value_tag_local,
            AsyncAwaitContinuation::AsyncGeneratorYield,
            function,
        )
    }

    pub(crate) fn emit_async_generator_yield_return_reactions(
        &mut self,
        activation_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_await_reactions(
            activation_local,
            value_payload_local,
            value_tag_local,
            AsyncAwaitContinuation::AsyncGeneratorYieldReturn,
            function,
        )
    }

    pub(crate) fn emit_intrinsic_await_with_handlers(
        &mut self,
        value_payload_local: u32,
        value_tag_local: u32,
        on_fulfilled_payload_local: u32,
        on_fulfilled_tag_local: u32,
        on_rejected_payload_local: u32,
        on_rejected_tag_local: u32,
        throwaway_capability_record_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_intrinsic_await_reactions(
            throwaway_capability_record_local,
            value_payload_local,
            value_tag_local,
            on_fulfilled_payload_local,
            on_fulfilled_tag_local,
            on_rejected_payload_local,
            on_rejected_tag_local,
            PromiseReactionCallbackKind::Default,
            function,
        )
    }

    fn emit_await_reactions(
        &mut self,
        activation_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        continuation: AsyncAwaitContinuation,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let undefined_payload_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));
        let result = self.emit_intrinsic_await_reactions(
            activation_local,
            value_payload_local,
            value_tag_local,
            undefined_payload_local,
            undefined_tag_local,
            undefined_payload_local,
            undefined_tag_local,
            continuation.reaction_callback_kind(),
            function,
        );
        self.release_temp_local(undefined_tag_local);
        self.release_temp_local(undefined_payload_local);
        result
    }

    fn emit_intrinsic_await_reactions(
        &mut self,
        reaction_capability_record_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        on_fulfilled_payload_local: u32,
        on_fulfilled_tag_local: u32,
        on_rejected_payload_local: u32,
        on_rejected_tag_local: u32,
        reaction_callback_kind: PromiseReactionCallbackKind,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let awaited_promise_payload_local = self.reserve_temp_local();
        let awaited_promise_record_local = self.reserve_temp_local();
        let rejected_promise_capability_local = self.reserve_temp_local();
        let rejected_promise_constructor_payload_local = self.reserve_temp_local();
        let rejected_promise_constructor_tag_local = self.reserve_temp_local();
        let resolve_error_payload_local = self.reserve_temp_local();
        let resolve_error_tag_local = self.reserve_temp_local();
        let source_state_local = self.reserve_temp_local();
        let source_result_payload_local = self.reserve_temp_local();
        let source_result_tag_local = self.reserve_temp_local();
        let fulfill_reaction_local = self.reserve_temp_local();
        let reject_reaction_local = self.reserve_temp_local();

        self.emit_intrinsic_promise_resolve_to_locals(
            value_payload_local,
            value_tag_local,
            awaited_promise_payload_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(awaited_promise_payload_local));
        function.instruction(&Instruction::LocalSet(resolve_error_payload_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalSet(resolve_error_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);
        function.instruction(&Instruction::GlobalGet(PROMISE_CONSTRUCTOR_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(
            rejected_promise_constructor_payload_local,
        ));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(
            rejected_promise_constructor_tag_local,
        ));
        self.emit_new_promise_capability(
            rejected_promise_constructor_payload_local,
            rejected_promise_constructor_tag_local,
            rejected_promise_capability_local,
            awaited_promise_payload_local,
            self.result_tag_local,
            function,
        )?;
        self.load_i64_to_local_from_offset(
            awaited_promise_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            awaited_promise_record_local,
            function,
        );
        self.emit_settle_promise_record(
            awaited_promise_record_local,
            PROMISE_STATE_REJECTED,
            resolve_error_payload_local,
            resolve_error_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            awaited_promise_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            awaited_promise_record_local,
            function,
        );

        self.emit_initialize_promise_reaction(
            fulfill_reaction_local,
            reaction_capability_record_local,
            on_fulfilled_payload_local,
            on_fulfilled_tag_local,
            PromiseReactionType::Fulfill,
            reaction_callback_kind,
            function,
        )?;
        self.emit_initialize_promise_reaction(
            reject_reaction_local,
            reaction_capability_record_local,
            on_rejected_payload_local,
            on_rejected_tag_local,
            PromiseReactionType::Reject,
            reaction_callback_kind,
            function,
        )?;

        self.load_i64_to_local_from_offset(
            awaited_promise_record_local,
            HEAP_PROMISE_STATE_OFFSET,
            source_state_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            awaited_promise_record_local,
            HEAP_PROMISE_RESULT_PAYLOAD_OFFSET,
            source_result_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            awaited_promise_record_local,
            HEAP_PROMISE_RESULT_TAG_OFFSET,
            source_result_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(source_state_local));
        function.instruction(&Instruction::I64Const(PROMISE_STATE_PENDING as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_append_promise_reaction(
            awaited_promise_record_local,
            HEAP_PROMISE_FULFILL_REACTIONS_OFFSET,
            fulfill_reaction_local,
            function,
        );
        self.emit_append_promise_reaction(
            awaited_promise_record_local,
            HEAP_PROMISE_REJECT_REACTIONS_OFFSET,
            reject_reaction_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(source_state_local));
        function.instruction(&Instruction::I64Const(PROMISE_STATE_FULFILLED as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_enqueue_promise_reaction_job(
            fulfill_reaction_local,
            source_result_payload_local,
            source_result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_enqueue_promise_reaction_job(
            reject_reaction_local,
            source_result_payload_local,
            source_result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.store_i64_const_at_offset(
            awaited_promise_record_local,
            HEAP_PROMISE_IS_HANDLED_OFFSET,
            1,
            function,
        );
        self.set_completion_kind(CompletionKind::Normal, function);

        self.release_temp_local(reject_reaction_local);
        self.release_temp_local(fulfill_reaction_local);
        self.release_temp_local(source_result_tag_local);
        self.release_temp_local(source_result_payload_local);
        self.release_temp_local(source_state_local);
        self.release_temp_local(resolve_error_tag_local);
        self.release_temp_local(resolve_error_payload_local);
        self.release_temp_local(rejected_promise_constructor_tag_local);
        self.release_temp_local(rejected_promise_constructor_payload_local);
        self.release_temp_local(rejected_promise_capability_local);
        self.release_temp_local(awaited_promise_record_local);
        self.release_temp_local(awaited_promise_payload_local);
        Ok(())
    }

    pub(crate) fn emit_async_generator_await_return_reactions(
        &mut self,
        activation_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        resolved_promise_payload_local: u32,
        resolved_promise_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let resolved_promise_record_local = self.reserve_temp_local();
        let source_state_local = self.reserve_temp_local();
        let source_result_payload_local = self.reserve_temp_local();
        let source_result_tag_local = self.reserve_temp_local();
        let fulfill_reaction_local = self.reserve_temp_local();
        let reject_reaction_local = self.reserve_temp_local();
        let undefined_payload_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();

        self.emit_intrinsic_promise_resolve_to_locals(
            value_payload_local,
            value_tag_local,
            resolved_promise_payload_local,
            resolved_promise_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            resolved_promise_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            resolved_promise_record_local,
            function,
        );

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));
        self.emit_initialize_promise_reaction(
            fulfill_reaction_local,
            activation_local,
            undefined_payload_local,
            undefined_tag_local,
            PromiseReactionType::Fulfill,
            PromiseReactionCallbackKind::AsyncGeneratorAwaitReturn,
            function,
        )?;
        self.emit_initialize_promise_reaction(
            reject_reaction_local,
            activation_local,
            undefined_payload_local,
            undefined_tag_local,
            PromiseReactionType::Reject,
            PromiseReactionCallbackKind::AsyncGeneratorAwaitReturn,
            function,
        )?;

        self.load_i64_to_local_from_offset(
            resolved_promise_record_local,
            HEAP_PROMISE_STATE_OFFSET,
            source_state_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            resolved_promise_record_local,
            HEAP_PROMISE_RESULT_PAYLOAD_OFFSET,
            source_result_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            resolved_promise_record_local,
            HEAP_PROMISE_RESULT_TAG_OFFSET,
            source_result_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(source_state_local));
        function.instruction(&Instruction::I64Const(PROMISE_STATE_PENDING as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_append_promise_reaction(
            resolved_promise_record_local,
            HEAP_PROMISE_FULFILL_REACTIONS_OFFSET,
            fulfill_reaction_local,
            function,
        );
        self.emit_append_promise_reaction(
            resolved_promise_record_local,
            HEAP_PROMISE_REJECT_REACTIONS_OFFSET,
            reject_reaction_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(source_state_local));
        function.instruction(&Instruction::I64Const(PROMISE_STATE_FULFILLED as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_enqueue_promise_reaction_job(
            fulfill_reaction_local,
            source_result_payload_local,
            source_result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_enqueue_promise_reaction_job(
            reject_reaction_local,
            source_result_payload_local,
            source_result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.store_i64_const_at_offset(
            resolved_promise_record_local,
            HEAP_PROMISE_IS_HANDLED_OFFSET,
            1,
            function,
        );
        function.instruction(&Instruction::End);

        self.release_temp_local(undefined_tag_local);
        self.release_temp_local(undefined_payload_local);
        self.release_temp_local(reject_reaction_local);
        self.release_temp_local(fulfill_reaction_local);
        self.release_temp_local(source_result_tag_local);
        self.release_temp_local(source_result_payload_local);
        self.release_temp_local(source_state_local);
        self.release_temp_local(resolved_promise_record_local);
        Ok(())
    }

    pub(crate) fn emit_promise_prototype_then(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Promise.prototype.then receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Promise.prototype.then receiver tag",
            )
        })?;
        let valid_receiver_local = self.reserve_temp_local();
        let brand_local = self.reserve_temp_local();
        let source_record_local = self.reserve_temp_local();
        let source_state_local = self.reserve_temp_local();
        let source_result_payload_local = self.reserve_temp_local();
        let source_result_tag_local = self.reserve_temp_local();
        let on_fulfilled_payload_local = self.reserve_temp_local();
        let on_fulfilled_tag_local = self.reserve_temp_local();
        let on_rejected_payload_local = self.reserve_temp_local();
        let on_rejected_tag_local = self.reserve_temp_local();
        let result_promise_payload_local = self.reserve_temp_local();
        let result_promise_tag_local = self.reserve_temp_local();
        let capability_record_local = self.reserve_temp_local();
        let species_constructor_payload_local = self.reserve_temp_local();
        let species_constructor_tag_local = self.reserve_temp_local();
        let fulfill_reaction_local = self.reserve_temp_local();
        let reject_reaction_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_receiver_local));
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(brand_local));
        function.instruction(&Instruction::I64Const(OBJECT_INTERNAL_BRAND_PROMISE as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(valid_receiver_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(valid_receiver_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Promise.prototype.then called on incompatible receiver",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            source_record_local,
            function,
        );
        self.emit_builtin_arg_to_locals(
            0,
            on_fulfilled_payload_local,
            on_fulfilled_tag_local,
            function,
        );
        self.emit_builtin_arg_to_locals(
            1,
            on_rejected_payload_local,
            on_rejected_tag_local,
            function,
        );
        self.emit_promise_species_constructor(
            receiver_payload_local,
            receiver_tag_local,
            species_constructor_payload_local,
            species_constructor_tag_local,
            function,
        )?;
        self.emit_new_promise_capability(
            species_constructor_payload_local,
            species_constructor_tag_local,
            capability_record_local,
            result_promise_payload_local,
            result_promise_tag_local,
            function,
        )?;
        self.emit_initialize_promise_reaction(
            fulfill_reaction_local,
            capability_record_local,
            on_fulfilled_payload_local,
            on_fulfilled_tag_local,
            PromiseReactionType::Fulfill,
            PromiseReactionCallbackKind::Default,
            function,
        )?;
        self.emit_initialize_promise_reaction(
            reject_reaction_local,
            capability_record_local,
            on_rejected_payload_local,
            on_rejected_tag_local,
            PromiseReactionType::Reject,
            PromiseReactionCallbackKind::Default,
            function,
        )?;

        self.load_i64_to_local_from_offset(
            source_record_local,
            HEAP_PROMISE_STATE_OFFSET,
            source_state_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            source_record_local,
            HEAP_PROMISE_RESULT_PAYLOAD_OFFSET,
            source_result_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            source_record_local,
            HEAP_PROMISE_RESULT_TAG_OFFSET,
            source_result_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(source_state_local));
        function.instruction(&Instruction::I64Const(PROMISE_STATE_PENDING as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_append_promise_reaction(
            source_record_local,
            HEAP_PROMISE_FULFILL_REACTIONS_OFFSET,
            fulfill_reaction_local,
            function,
        );
        self.emit_append_promise_reaction(
            source_record_local,
            HEAP_PROMISE_REJECT_REACTIONS_OFFSET,
            reject_reaction_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(source_state_local));
        function.instruction(&Instruction::I64Const(PROMISE_STATE_FULFILLED as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_enqueue_promise_reaction_job(
            fulfill_reaction_local,
            source_result_payload_local,
            source_result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_enqueue_promise_reaction_job(
            reject_reaction_local,
            source_result_payload_local,
            source_result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.store_i64_const_at_offset(
            source_record_local,
            HEAP_PROMISE_IS_HANDLED_OFFSET,
            1,
            function,
        );

        function.instruction(&Instruction::LocalGet(result_promise_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(result_promise_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);

        self.release_temp_local(reject_reaction_local);
        self.release_temp_local(fulfill_reaction_local);
        self.release_temp_local(species_constructor_tag_local);
        self.release_temp_local(species_constructor_payload_local);
        self.release_temp_local(capability_record_local);
        self.release_temp_local(result_promise_tag_local);
        self.release_temp_local(result_promise_payload_local);
        self.release_temp_local(on_rejected_tag_local);
        self.release_temp_local(on_rejected_payload_local);
        self.release_temp_local(on_fulfilled_tag_local);
        self.release_temp_local(on_fulfilled_payload_local);
        self.release_temp_local(source_result_tag_local);
        self.release_temp_local(source_result_payload_local);
        self.release_temp_local(source_state_local);
        self.release_temp_local(source_record_local);
        self.release_temp_local(brand_local);
        self.release_temp_local(valid_receiver_local);
        Ok(())
    }

    pub(crate) fn emit_promise_prototype_catch(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Promise.prototype.catch receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Promise.prototype.catch receiver tag",
            )
        })?;
        let object_payload_local = self.reserve_temp_local();
        let object_tag_local = self.reserve_temp_local();
        let then_key_local = self.reserve_temp_local();
        let then_payload_local = self.reserve_temp_local();
        let then_tag_local = self.reserve_temp_local();
        let on_rejected_payload_local = self.reserve_temp_local();
        let on_rejected_tag_local = self.reserve_temp_local();
        let undefined_payload_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(
            0,
            on_rejected_payload_local,
            on_rejected_tag_local,
            function,
        );
        self.emit_value_to_object_locals(
            receiver_payload_local,
            receiver_tag_local,
            object_payload_local,
            object_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload("then")));
        function.instruction(&Instruction::LocalSet(then_key_local));
        self.emit_object_read(
            object_payload_local,
            object_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            then_key_local,
            then_payload_local,
            then_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));
        self.emit_function_or_proxy_call_leave_throw_completion(
            then_payload_local,
            then_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            &[
                (undefined_payload_local, undefined_tag_local),
                (on_rejected_payload_local, on_rejected_tag_local),
            ],
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.set_completion_kind(CompletionKind::Normal, function);

        self.release_temp_local(undefined_tag_local);
        self.release_temp_local(undefined_payload_local);
        self.release_temp_local(on_rejected_tag_local);
        self.release_temp_local(on_rejected_payload_local);
        self.release_temp_local(then_tag_local);
        self.release_temp_local(then_payload_local);
        self.release_temp_local(then_key_local);
        self.release_temp_local(object_tag_local);
        self.release_temp_local(object_payload_local);
        Ok(())
    }

    pub(crate) fn emit_promise_prototype_finally(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Promise.prototype.finally receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Promise.prototype.finally receiver tag",
            )
        })?;
        let on_finally_payload_local = self.reserve_temp_local();
        let on_finally_tag_local = self.reserve_temp_local();
        let constructor_payload_local = self.reserve_temp_local();
        let constructor_tag_local = self.reserve_temp_local();
        let context_local = self.reserve_temp_local();
        let then_finally_payload_local = self.reserve_temp_local();
        let then_finally_tag_local = self.reserve_temp_local();
        let catch_finally_payload_local = self.reserve_temp_local();
        let catch_finally_tag_local = self.reserve_temp_local();
        let then_key_local = self.reserve_temp_local();
        let then_payload_local = self.reserve_temp_local();
        let then_tag_local = self.reserve_temp_local();

        let then_finally_meta = self
            .functions
            .get(&StandardBuiltinId::PromiseThenFinally.function_id())
            .cloned()
            .ok_or_else(|| EmitError::unsupported("missing Promise then-finally builtin"))?;
        let catch_finally_meta = self
            .functions
            .get(&StandardBuiltinId::PromiseCatchFinally.function_id())
            .cloned()
            .ok_or_else(|| EmitError::unsupported("missing Promise catch-finally builtin"))?;

        self.emit_is_heap_object_like_tag_i32(receiver_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Promise.prototype.finally called on non-object receiver",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_builtin_arg_to_locals(
            0,
            on_finally_payload_local,
            on_finally_tag_local,
            function,
        );
        self.emit_promise_species_constructor(
            receiver_payload_local,
            receiver_tag_local,
            constructor_payload_local,
            constructor_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(on_finally_payload_local));
        function.instruction(&Instruction::LocalSet(then_finally_payload_local));
        function.instruction(&Instruction::LocalGet(on_finally_tag_local));
        function.instruction(&Instruction::LocalSet(then_finally_tag_local));
        function.instruction(&Instruction::LocalGet(on_finally_payload_local));
        function.instruction(&Instruction::LocalSet(catch_finally_payload_local));
        function.instruction(&Instruction::LocalGet(on_finally_tag_local));
        function.instruction(&Instruction::LocalSet(catch_finally_tag_local));
        self.emit_is_callable_i32(on_finally_tag_local, on_finally_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_heap_alloc_const(HEAP_PROMISE_FINALLY_CONTEXT_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(context_local));
        for (offset, value_local) in [
            (
                HEAP_PROMISE_FINALLY_ON_FINALLY_PAYLOAD_OFFSET,
                on_finally_payload_local,
            ),
            (
                HEAP_PROMISE_FINALLY_ON_FINALLY_TAG_OFFSET,
                on_finally_tag_local,
            ),
            (
                HEAP_PROMISE_FINALLY_CONSTRUCTOR_PAYLOAD_OFFSET,
                constructor_payload_local,
            ),
            (
                HEAP_PROMISE_FINALLY_CONSTRUCTOR_TAG_OFFSET,
                constructor_tag_local,
            ),
        ] {
            self.store_i64_local_at_offset(context_local, offset, value_local, function);
        }
        self.emit_function_value_payload(&then_finally_meta, function)?;
        function.instruction(&Instruction::LocalSet(then_finally_payload_local));
        self.store_i64_local_at_offset(
            then_finally_payload_local,
            HEAP_FUNCTION_ENV_HANDLE_OFFSET,
            context_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(then_finally_tag_local));
        self.emit_function_value_payload(&catch_finally_meta, function)?;
        function.instruction(&Instruction::LocalSet(catch_finally_payload_local));
        self.store_i64_local_at_offset(
            catch_finally_payload_local,
            HEAP_FUNCTION_ENV_HANDLE_OFFSET,
            context_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(catch_finally_tag_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("then")));
        function.instruction(&Instruction::LocalSet(then_key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            then_key_local,
            then_payload_local,
            then_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_function_or_proxy_call_leave_throw_completion(
            then_payload_local,
            then_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            &[
                (then_finally_payload_local, then_finally_tag_local),
                (catch_finally_payload_local, catch_finally_tag_local),
            ],
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.set_completion_kind(CompletionKind::Normal, function);

        self.release_temp_local(then_tag_local);
        self.release_temp_local(then_payload_local);
        self.release_temp_local(then_key_local);
        self.release_temp_local(catch_finally_tag_local);
        self.release_temp_local(catch_finally_payload_local);
        self.release_temp_local(then_finally_tag_local);
        self.release_temp_local(then_finally_payload_local);
        self.release_temp_local(context_local);
        self.release_temp_local(constructor_tag_local);
        self.release_temp_local(constructor_payload_local);
        self.release_temp_local(on_finally_tag_local);
        self.release_temp_local(on_finally_payload_local);
        Ok(())
    }

    pub(crate) fn emit_promise_finally_continuation(
        &mut self,
        rejected: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let context_local = self.reserve_temp_local();
        let original_payload_local = self.reserve_temp_local();
        let original_tag_local = self.reserve_temp_local();
        let on_finally_payload_local = self.reserve_temp_local();
        let on_finally_tag_local = self.reserve_temp_local();
        let constructor_payload_local = self.reserve_temp_local();
        let constructor_tag_local = self.reserve_temp_local();
        let undefined_payload_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();
        let cleanup_payload_local = self.reserve_temp_local();
        let cleanup_tag_local = self.reserve_temp_local();
        let promise_resolve_payload_local = self.reserve_temp_local();
        let promise_resolve_tag_local = self.reserve_temp_local();
        let cleanup_promise_payload_local = self.reserve_temp_local();
        let cleanup_promise_tag_local = self.reserve_temp_local();
        let value_context_local = self.reserve_temp_local();
        let continuation_payload_local = self.reserve_temp_local();
        let continuation_tag_local = self.reserve_temp_local();
        let then_key_local = self.reserve_temp_local();
        let then_payload_local = self.reserve_temp_local();
        let then_tag_local = self.reserve_temp_local();

        let promise_resolve_meta = self
            .functions
            .get(&StandardBuiltinId::PromiseResolve.function_id())
            .cloned()
            .ok_or_else(|| EmitError::unsupported("missing Promise.resolve builtin"))?;
        let continuation_builtin = if rejected {
            StandardBuiltinId::PromiseThrower
        } else {
            StandardBuiltinId::PromiseValueThunk
        };
        let continuation_meta = self
            .functions
            .get(&continuation_builtin.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported("missing Promise finally continuation builtin")
            })?;

        function.instruction(&Instruction::LocalGet(0));
        function.instruction(&Instruction::LocalSet(context_local));
        self.emit_builtin_arg_to_locals(0, original_payload_local, original_tag_local, function);
        for (offset, value_local) in [
            (
                HEAP_PROMISE_FINALLY_ON_FINALLY_PAYLOAD_OFFSET,
                on_finally_payload_local,
            ),
            (
                HEAP_PROMISE_FINALLY_ON_FINALLY_TAG_OFFSET,
                on_finally_tag_local,
            ),
            (
                HEAP_PROMISE_FINALLY_CONSTRUCTOR_PAYLOAD_OFFSET,
                constructor_payload_local,
            ),
            (
                HEAP_PROMISE_FINALLY_CONSTRUCTOR_TAG_OFFSET,
                constructor_tag_local,
            ),
        ] {
            self.load_i64_to_local_from_offset(context_local, offset, value_local, function);
        }
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));
        self.emit_function_or_proxy_call_leave_throw_completion(
            on_finally_payload_local,
            on_finally_tag_local,
            undefined_payload_local,
            undefined_tag_local,
            &[],
            cleanup_payload_local,
            cleanup_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);

        self.emit_function_value_payload(&promise_resolve_meta, function)?;
        function.instruction(&Instruction::LocalSet(promise_resolve_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(promise_resolve_tag_local));
        self.emit_function_or_proxy_call_leave_throw_completion(
            promise_resolve_payload_local,
            promise_resolve_tag_local,
            constructor_payload_local,
            constructor_tag_local,
            &[(cleanup_payload_local, cleanup_tag_local)],
            cleanup_promise_payload_local,
            cleanup_promise_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);

        self.emit_heap_alloc_const(HEAP_PROMISE_FINALLY_VALUE_CONTEXT_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(value_context_local));
        self.store_i64_local_at_offset(
            value_context_local,
            HEAP_PROMISE_FINALLY_VALUE_PAYLOAD_OFFSET,
            original_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            value_context_local,
            HEAP_PROMISE_FINALLY_VALUE_TAG_OFFSET,
            original_tag_local,
            function,
        );
        self.emit_function_value_payload(&continuation_meta, function)?;
        function.instruction(&Instruction::LocalSet(continuation_payload_local));
        self.store_i64_local_at_offset(
            continuation_payload_local,
            HEAP_FUNCTION_ENV_HANDLE_OFFSET,
            value_context_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(continuation_tag_local));

        function.instruction(&Instruction::I64Const(self.strings.payload("then")));
        function.instruction(&Instruction::LocalSet(then_key_local));
        self.emit_object_read(
            cleanup_promise_payload_local,
            cleanup_promise_tag_local,
            cleanup_promise_payload_local,
            cleanup_promise_tag_local,
            then_key_local,
            then_payload_local,
            then_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_function_or_proxy_call_leave_throw_completion(
            then_payload_local,
            then_tag_local,
            cleanup_promise_payload_local,
            cleanup_promise_tag_local,
            &[(continuation_payload_local, continuation_tag_local)],
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.set_completion_kind(CompletionKind::Normal, function);

        self.release_temp_local(then_tag_local);
        self.release_temp_local(then_payload_local);
        self.release_temp_local(then_key_local);
        self.release_temp_local(continuation_tag_local);
        self.release_temp_local(continuation_payload_local);
        self.release_temp_local(value_context_local);
        self.release_temp_local(cleanup_promise_tag_local);
        self.release_temp_local(cleanup_promise_payload_local);
        self.release_temp_local(promise_resolve_tag_local);
        self.release_temp_local(promise_resolve_payload_local);
        self.release_temp_local(cleanup_tag_local);
        self.release_temp_local(cleanup_payload_local);
        self.release_temp_local(undefined_tag_local);
        self.release_temp_local(undefined_payload_local);
        self.release_temp_local(constructor_tag_local);
        self.release_temp_local(constructor_payload_local);
        self.release_temp_local(on_finally_tag_local);
        self.release_temp_local(on_finally_payload_local);
        self.release_temp_local(original_tag_local);
        self.release_temp_local(original_payload_local);
        self.release_temp_local(context_local);
        Ok(())
    }

    pub(crate) fn emit_promise_finally_value_thunk(
        &mut self,
        throws: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let context_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(0));
        function.instruction(&Instruction::LocalSet(context_local));
        self.load_i64_to_local_from_offset(
            context_local,
            HEAP_PROMISE_FINALLY_VALUE_PAYLOAD_OFFSET,
            self.result_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            context_local,
            HEAP_PROMISE_FINALLY_VALUE_TAG_OFFSET,
            self.result_tag_local,
            function,
        );
        self.set_completion_kind(
            if throws {
                CompletionKind::Throw
            } else {
                CompletionKind::Normal
            },
            function,
        );

        self.release_temp_local(context_local);
        Ok(())
    }

    fn emit_run_async_continuation_job(
        &mut self,
        reaction_record_local: u32,
        reaction_is_rejected_local: u32,
        argument_payload_local: u32,
        argument_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let activation_local = self.reserve_temp_local();
        let function_env_local = self.reserve_temp_local();
        let function_table_index_local = self.reserve_temp_local();
        let this_payload_local = self.reserve_temp_local();
        let this_tag_local = self.reserve_temp_local();
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();
        let completed_local = self.reserve_temp_local();
        let body_payload_local = self.reserve_temp_local();
        let body_tag_local = self.reserve_temp_local();
        let promise_payload_local = self.reserve_temp_local();
        let promise_record_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            reaction_record_local,
            HEAP_PROMISE_REACTION_CAPABILITY_OFFSET,
            activation_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_ASYNC_COMPLETED_OFFSET,
            completed_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(completed_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        for (offset, destination_local) in [
            (HEAP_ASYNC_FUNCTION_ENV_OFFSET, function_env_local),
            (
                HEAP_ASYNC_FUNCTION_TABLE_INDEX_OFFSET,
                function_table_index_local,
            ),
            (HEAP_ASYNC_THIS_PAYLOAD_OFFSET, this_payload_local),
            (HEAP_ASYNC_THIS_TAG_OFFSET, this_tag_local),
            (HEAP_ASYNC_ARGC_OFFSET, argc_local),
            (HEAP_ASYNC_ARGV_OFFSET, argv_local),
            (HEAP_ASYNC_PROMISE_PAYLOAD_OFFSET, promise_payload_local),
            (HEAP_ASYNC_PROMISE_RECORD_OFFSET, promise_record_local),
        ] {
            self.load_i64_to_local_from_offset(
                activation_local,
                offset,
                destination_local,
                function,
            );
        }
        self.store_i64_local_at_offset(
            activation_local,
            HEAP_ASYNC_RESUME_PAYLOAD_OFFSET,
            argument_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            activation_local,
            HEAP_ASYNC_RESUME_TAG_OFFSET,
            argument_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(reaction_is_rejected_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_store_async_function_resume_completion(
            activation_local,
            AsyncFunctionResumeCompletion::Normal,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_store_async_function_resume_completion(
            activation_local,
            AsyncFunctionResumeCompletion::Throw,
            function,
        );
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(function_env_local));
        function.instruction(&Instruction::LocalGet(this_payload_local));
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::LocalGet(activation_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalGet(argc_local));
        function.instruction(&Instruction::LocalGet(argv_local));
        function.instruction(&Instruction::LocalGet(function_table_index_local));
        function.instruction(&Instruction::I32WrapI64);
        self.set_completion_kind(CompletionKind::Normal, function);
        function.instruction(&Instruction::CallIndirect {
            type_index: JS_FUNCTION_TYPE_INDEX,
            table_index: 0,
        });
        self.store_call_results(body_payload_local, body_tag_local, function);

        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(self.completion_aux_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_settle_promise_record(
            promise_record_local,
            PROMISE_STATE_REJECTED,
            body_payload_local,
            body_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_resolve_promise_record(
            promise_payload_local,
            promise_record_local,
            body_payload_local,
            body_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.store_i64_const_at_offset(activation_local, HEAP_ASYNC_COMPLETED_OFFSET, 1, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.set_completion_kind(CompletionKind::Normal, function);

        self.release_temp_local(promise_record_local);
        self.release_temp_local(promise_payload_local);
        self.release_temp_local(body_tag_local);
        self.release_temp_local(body_payload_local);
        self.release_temp_local(completed_local);
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(this_tag_local);
        self.release_temp_local(this_payload_local);
        self.release_temp_local(function_table_index_local);
        self.release_temp_local(function_env_local);
        self.release_temp_local(activation_local);
        Ok(())
    }

    fn emit_remove_async_generator_queue_head(
        &mut self,
        activation_local: u32,
        request_local: u32,
        next_request_local: u32,
        function: &mut Function,
    ) {
        self.load_i64_to_local_from_offset(
            request_local,
            HEAP_ASYNC_GENERATOR_REQUEST_NEXT_OFFSET,
            next_request_local,
            function,
        );
        self.store_i64_local_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_QUEUE_HEAD_OFFSET,
            next_request_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(next_request_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_const_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_QUEUE_TAIL_OFFSET,
            0,
            function,
        );
        function.instruction(&Instruction::End);
    }

    pub(crate) fn emit_complete_async_generator_step(
        &mut self,
        activation_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        completion_kind_local: u32,
        done: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let request_local = self.reserve_temp_local();
        let next_request_local = self.reserve_temp_local();
        let promise_payload_local = self.reserve_temp_local();
        let promise_record_local = self.reserve_temp_local();
        let iterator_result_payload_local = self.reserve_temp_local();
        let iterator_result_tag_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_ACTIVE_REQUEST_OFFSET,
            request_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(request_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        for (offset, destination_local) in [
            (
                HEAP_ASYNC_GENERATOR_REQUEST_PROMISE_PAYLOAD_OFFSET,
                promise_payload_local,
            ),
            (
                HEAP_ASYNC_GENERATOR_REQUEST_PROMISE_RECORD_OFFSET,
                promise_record_local,
            ),
        ] {
            self.load_i64_to_local_from_offset(request_local, offset, destination_local, function);
        }
        self.emit_remove_async_generator_queue_head(
            activation_local,
            request_local,
            next_request_local,
            function,
        );
        self.store_i64_const_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_ACTIVE_REQUEST_OFFSET,
            0,
            function,
        );
        function.instruction(&Instruction::LocalGet(completion_kind_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_settle_promise_record(
            promise_record_local,
            PROMISE_STATE_REJECTED,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(completion_kind_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_NORMAL));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_iterator_result_object_from_locals(
            value_payload_local,
            value_tag_local,
            done,
            iterator_result_payload_local,
            iterator_result_tag_local,
            function,
        )?;
        self.emit_resolve_promise_record(
            promise_payload_local,
            promise_record_local,
            iterator_result_payload_local,
            iterator_result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.set_completion_kind(CompletionKind::Normal, function);

        self.release_temp_local(iterator_result_tag_local);
        self.release_temp_local(iterator_result_payload_local);
        self.release_temp_local(promise_record_local);
        self.release_temp_local(promise_payload_local);
        self.release_temp_local(next_request_local);
        self.release_temp_local(request_local);
        Ok(())
    }

    pub(crate) fn emit_drain_async_generator_queue(
        &mut self,
        activation_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let request_local = self.reserve_temp_local();
        let completion_kind_local = self.reserve_temp_local();
        let completion_payload_local = self.reserve_temp_local();
        let completion_tag_local = self.reserve_temp_local();
        let undefined_payload_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();
        let resolved_promise_payload_local = self.reserve_temp_local();
        let resolved_promise_tag_local = self.reserve_temp_local();
        let stop_draining_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(stop_draining_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_QUEUE_HEAD_OFFSET,
            request_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(request_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        for (offset, destination_local) in [
            (
                HEAP_ASYNC_GENERATOR_REQUEST_COMPLETION_KIND_OFFSET,
                completion_kind_local,
            ),
            (
                HEAP_ASYNC_GENERATOR_REQUEST_COMPLETION_PAYLOAD_OFFSET,
                completion_payload_local,
            ),
            (
                HEAP_ASYNC_GENERATOR_REQUEST_COMPLETION_TAG_OFFSET,
                completion_tag_local,
            ),
        ] {
            self.load_i64_to_local_from_offset(request_local, offset, destination_local, function);
        }
        self.store_i64_local_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_ACTIVE_REQUEST_OFFSET,
            request_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(completion_kind_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_NORMAL));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_complete_async_generator_step(
            activation_local,
            undefined_payload_local,
            undefined_tag_local,
            completion_kind_local,
            true,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(completion_kind_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_complete_async_generator_step(
            activation_local,
            completion_payload_local,
            completion_tag_local,
            completion_kind_local,
            true,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(completion_kind_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_RETURN));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_async_generator_await_return_reactions(
            activation_local,
            completion_payload_local,
            completion_tag_local,
            resolved_promise_payload_local,
            resolved_promise_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.set_completion_kind(CompletionKind::Normal, function);
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::LocalSet(completion_kind_local));
        self.emit_complete_async_generator_step(
            activation_local,
            resolved_promise_payload_local,
            resolved_promise_tag_local,
            completion_kind_local,
            true,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(stop_draining_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.set_completion_kind(CompletionKind::Normal, function);
        function.instruction(&Instruction::LocalGet(stop_draining_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(stop_draining_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_const_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_ACTIVE_REQUEST_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_EXECUTION_STATE_OFFSET,
            ASYNC_GENERATOR_STATE_COMPLETED,
            function,
        );
        function.instruction(&Instruction::End);

        self.release_temp_local(stop_draining_local);
        self.release_temp_local(resolved_promise_tag_local);
        self.release_temp_local(resolved_promise_payload_local);
        self.release_temp_local(undefined_tag_local);
        self.release_temp_local(undefined_payload_local);
        self.release_temp_local(completion_tag_local);
        self.release_temp_local(completion_payload_local);
        self.release_temp_local(completion_kind_local);
        self.release_temp_local(request_local);
        Ok(())
    }

    fn emit_run_async_generator_await_job(
        &mut self,
        reaction_record_local: u32,
        reaction_is_rejected_local: u32,
        argument_payload_local: u32,
        argument_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let activation_local = self.reserve_temp_local();
        let execution_state_local = self.reserve_temp_local();
        let body_status_local = self.reserve_temp_local();
        let active_request_local = self.reserve_temp_local();
        let queue_head_local = self.reserve_temp_local();
        let resume_kind_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            reaction_record_local,
            HEAP_PROMISE_REACTION_CAPABILITY_OFFSET,
            activation_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_EXECUTION_STATE_OFFSET,
            execution_state_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(execution_state_local));
        function.instruction(&Instruction::I64Const(
            ASYNC_GENERATOR_STATE_SUSPENDED_AWAIT as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_BODY_STATUS_OFFSET,
            body_status_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(body_status_local));
        function.instruction(&Instruction::I64Const(
            ASYNC_GENERATOR_BODY_STATUS_AWAIT as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_ACTIVE_REQUEST_OFFSET,
            active_request_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(active_request_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_QUEUE_HEAD_OFFSET,
            queue_head_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(active_request_local));
        function.instruction(&Instruction::LocalGet(queue_head_local));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(reaction_is_rejected_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            ASYNC_GENERATOR_RESUME_KIND_FULFILL as i64,
        ));
        function.instruction(&Instruction::LocalSet(resume_kind_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(
            ASYNC_GENERATOR_RESUME_KIND_REJECT as i64,
        ));
        function.instruction(&Instruction::LocalSet(resume_kind_local));
        function.instruction(&Instruction::End);

        // `AsyncFromSyncIteratorContinuation` steps 6.a and 13. This runs
        // before the body is resumed, and leaves both the resume payload and
        // the current completion untouched: the generator is still resumed
        // with the *original* rejection reason.
        //
        // Known microtask-ordering deviation, recorded rather than fixed. In the
        // spec the close happens in the valueWrapper's `onRejected` reaction job
        // and the generator's `Await(innerResult)` reaction is a LATER job; here
        // the close and `emit_start_async_generator_body` below run in ONE
        // invocation of the await job, because this backend never materialises
        // the AsyncFromSync wrapper promise. Observable only when the sync
        // `return` method itself schedules a microtask: under the spec that
        // microtask runs before the generator resumes, here after. This follows
        // from the pre-existing job fusion, not from the close emission, and no
        // case in `built-ins/AsyncFromSyncIteratorPrototype` observes it.
        self.emit_async_from_sync_close_on_rejection(
            activation_local,
            resume_kind_local,
            function,
        )?;

        self.store_i64_local_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_RESUME_PAYLOAD_OFFSET,
            argument_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_RESUME_TAG_OFFSET,
            argument_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_RESUME_KIND_OFFSET,
            resume_kind_local,
            function,
        );
        self.emit_start_async_generator_body(activation_local, function)?;

        self.release_temp_local(resume_kind_local);
        self.release_temp_local(queue_head_local);
        self.release_temp_local(active_request_local);
        self.release_temp_local(body_status_local);
        self.release_temp_local(execution_state_local);
        self.release_temp_local(activation_local);
        Ok(())
    }

    fn emit_run_async_generator_await_return_job(
        &mut self,
        reaction_record_local: u32,
        reaction_is_rejected_local: u32,
        argument_payload_local: u32,
        argument_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let activation_local = self.reserve_temp_local();
        let completion_kind_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            reaction_record_local,
            HEAP_PROMISE_REACTION_CAPABILITY_OFFSET,
            activation_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(reaction_is_rejected_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_NORMAL));
        function.instruction(&Instruction::LocalSet(completion_kind_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::LocalSet(completion_kind_local));
        function.instruction(&Instruction::End);
        self.emit_complete_async_generator_step(
            activation_local,
            argument_payload_local,
            argument_tag_local,
            completion_kind_local,
            true,
            function,
        )?;
        self.emit_drain_async_generator_queue(activation_local, function)?;
        self.release_temp_local(completion_kind_local);
        self.release_temp_local(activation_local);
        Ok(())
    }

    fn emit_run_async_generator_yield_return_job(
        &mut self,
        reaction_record_local: u32,
        reaction_is_rejected_local: u32,
        argument_payload_local: u32,
        argument_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let activation_local = self.reserve_temp_local();
        let execution_state_local = self.reserve_temp_local();
        let body_status_local = self.reserve_temp_local();
        let active_request_local = self.reserve_temp_local();
        let queue_head_local = self.reserve_temp_local();
        let resume_kind_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            reaction_record_local,
            HEAP_PROMISE_REACTION_CAPABILITY_OFFSET,
            activation_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_EXECUTION_STATE_OFFSET,
            execution_state_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(execution_state_local));
        function.instruction(&Instruction::I64Const(
            ASYNC_GENERATOR_STATE_SUSPENDED_AWAIT as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_BODY_STATUS_OFFSET,
            body_status_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(body_status_local));
        function.instruction(&Instruction::I64Const(
            ASYNC_GENERATOR_BODY_STATUS_AWAIT as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_ACTIVE_REQUEST_OFFSET,
            active_request_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(active_request_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_QUEUE_HEAD_OFFSET,
            queue_head_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(active_request_local));
        function.instruction(&Instruction::LocalGet(queue_head_local));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(reaction_is_rejected_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            ASYNC_GENERATOR_RESUME_KIND_RETURN as i64,
        ));
        function.instruction(&Instruction::LocalSet(resume_kind_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(
            ASYNC_GENERATOR_RESUME_KIND_THROW as i64,
        ));
        function.instruction(&Instruction::LocalSet(resume_kind_local));
        function.instruction(&Instruction::End);

        self.store_i64_local_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_RESUME_PAYLOAD_OFFSET,
            argument_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_RESUME_TAG_OFFSET,
            argument_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_RESUME_KIND_OFFSET,
            resume_kind_local,
            function,
        );
        self.emit_start_async_generator_body(activation_local, function)?;

        self.release_temp_local(resume_kind_local);
        self.release_temp_local(queue_head_local);
        self.release_temp_local(active_request_local);
        self.release_temp_local(body_status_local);
        self.release_temp_local(execution_state_local);
        self.release_temp_local(activation_local);
        Ok(())
    }

    fn emit_run_async_generator_yield_job(
        &mut self,
        reaction_record_local: u32,
        reaction_is_rejected_local: u32,
        argument_payload_local: u32,
        argument_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let activation_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            reaction_record_local,
            HEAP_PROMISE_REACTION_CAPABILITY_OFFSET,
            activation_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(reaction_is_rejected_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        let resume_body_local = self.reserve_temp_local();
        self.emit_complete_async_generator_yield(
            activation_local,
            argument_payload_local,
            argument_tag_local,
            resume_body_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(resume_body_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_start_async_generator_body(activation_local, function)?;
        function.instruction(&Instruction::End);
        self.release_temp_local(resume_body_local);

        function.instruction(&Instruction::Else);
        self.store_i64_local_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_RESUME_PAYLOAD_OFFSET,
            argument_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_RESUME_TAG_OFFSET,
            argument_tag_local,
            function,
        );
        self.store_i64_const_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_RESUME_KIND_OFFSET,
            ASYNC_GENERATOR_RESUME_KIND_REJECT,
            function,
        );
        self.emit_start_async_generator_body(activation_local, function)?;
        function.instruction(&Instruction::End);

        self.release_temp_local(activation_local);
        Ok(())
    }

    pub(crate) fn emit_complete_async_generator_yield(
        &mut self,
        activation_local: u32,
        yield_payload_local: u32,
        yield_tag_local: u32,
        resume_body_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let completion_kind_local = self.reserve_temp_local();
        let request_local = self.reserve_temp_local();
        let request_payload_local = self.reserve_temp_local();
        let request_tag_local = self.reserve_temp_local();
        let request_completion_kind_local = self.reserve_temp_local();
        let resume_kind_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(resume_body_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_NORMAL));
        function.instruction(&Instruction::LocalSet(completion_kind_local));
        self.emit_complete_async_generator_step(
            activation_local,
            yield_payload_local,
            yield_tag_local,
            completion_kind_local,
            false,
            function,
        )?;
        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_QUEUE_HEAD_OFFSET,
            request_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(request_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.store_i64_local_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_ACTIVE_REQUEST_OFFSET,
            request_local,
            function,
        );
        for (offset, destination_local) in [
            (
                HEAP_ASYNC_GENERATOR_REQUEST_COMPLETION_PAYLOAD_OFFSET,
                request_payload_local,
            ),
            (
                HEAP_ASYNC_GENERATOR_REQUEST_COMPLETION_TAG_OFFSET,
                request_tag_local,
            ),
            (
                HEAP_ASYNC_GENERATOR_REQUEST_COMPLETION_KIND_OFFSET,
                request_completion_kind_local,
            ),
        ] {
            self.load_i64_to_local_from_offset(request_local, offset, destination_local, function);
        }
        function.instruction(&Instruction::LocalGet(request_completion_kind_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_RETURN));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_async_generator_yield_return_reactions(
            activation_local,
            request_payload_local,
            request_tag_local,
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
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(resume_body_local));
        self.store_i64_local_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_RESUME_PAYLOAD_OFFSET,
            request_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_RESUME_TAG_OFFSET,
            request_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(
            ASYNC_GENERATOR_RESUME_KIND_NORMAL as i64,
        ));
        function.instruction(&Instruction::LocalSet(resume_kind_local));
        function.instruction(&Instruction::LocalGet(request_completion_kind_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            ASYNC_GENERATOR_RESUME_KIND_THROW as i64,
        ));
        function.instruction(&Instruction::LocalSet(resume_kind_local));
        function.instruction(&Instruction::End);
        self.store_i64_local_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_RESUME_KIND_OFFSET,
            resume_kind_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(resume_kind_local);
        self.release_temp_local(request_completion_kind_local);
        self.release_temp_local(request_tag_local);
        self.release_temp_local(request_payload_local);
        self.release_temp_local(request_local);
        self.release_temp_local(completion_kind_local);
        Ok(())
    }

    fn emit_run_promise_reaction_callback(
        &mut self,
        kind: PromiseReactionCallbackKind,
        reaction_record_local: u32,
        reaction_is_rejected_local: u32,
        argument_payload_local: u32,
        argument_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match kind {
            PromiseReactionCallbackKind::Default => self.emit_run_default_promise_reaction_job(
                reaction_record_local,
                reaction_is_rejected_local,
                argument_payload_local,
                argument_tag_local,
                function,
            ),
            PromiseReactionCallbackKind::AsyncFunction => self.emit_run_async_continuation_job(
                reaction_record_local,
                reaction_is_rejected_local,
                argument_payload_local,
                argument_tag_local,
                function,
            ),
            PromiseReactionCallbackKind::AsyncGeneratorAwaitReturn => self
                .emit_run_async_generator_await_return_job(
                    reaction_record_local,
                    reaction_is_rejected_local,
                    argument_payload_local,
                    argument_tag_local,
                    function,
                ),
            PromiseReactionCallbackKind::AsyncGeneratorAwait => self
                .emit_run_async_generator_await_job(
                    reaction_record_local,
                    reaction_is_rejected_local,
                    argument_payload_local,
                    argument_tag_local,
                    function,
                ),
            PromiseReactionCallbackKind::AsyncGeneratorYield => self
                .emit_run_async_generator_yield_job(
                    reaction_record_local,
                    reaction_is_rejected_local,
                    argument_payload_local,
                    argument_tag_local,
                    function,
                ),
            PromiseReactionCallbackKind::AsyncGeneratorYieldReturn => self
                .emit_run_async_generator_yield_return_job(
                    reaction_record_local,
                    reaction_is_rejected_local,
                    argument_payload_local,
                    argument_tag_local,
                    function,
                ),
        }
    }

    fn emit_decode_promise_reaction_type(
        &self,
        reaction_type_word_local: u32,
        reaction_is_rejected_local: u32,
        function: &mut Function,
    ) {
        let mut open_dispatch_arms = 0;
        for reaction_type in PromiseReactionType::ALL {
            function.instruction(&Instruction::LocalGet(reaction_type_word_local));
            function.instruction(&Instruction::I64Const(reaction_type.word() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(if reaction_type.is_rejected() {
                1
            } else {
                0
            }));
            function.instruction(&Instruction::LocalSet(reaction_is_rejected_local));
            function.instruction(&Instruction::Else);
            open_dispatch_arms += 1;
        }
        function.instruction(&Instruction::Unreachable);
        for _ in 0..open_dispatch_arms {
            function.instruction(&Instruction::End);
        }
    }

    fn emit_run_promise_reaction_job(
        &mut self,
        reaction_record_local: u32,
        argument_payload_local: u32,
        argument_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let callback_kind_local = self.reserve_temp_local();
        let reaction_type_word_local = self.reserve_temp_local();
        let reaction_is_rejected_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            reaction_record_local,
            HEAP_PROMISE_REACTION_CALLBACK_KIND_OFFSET,
            callback_kind_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            reaction_record_local,
            HEAP_PROMISE_REACTION_TYPE_OFFSET,
            reaction_type_word_local,
            function,
        );
        self.emit_decode_promise_reaction_type(
            reaction_type_word_local,
            reaction_is_rejected_local,
            function,
        );

        let mut open_dispatch_arms = 0;
        for kind in PromiseReactionCallbackKind::ALL {
            function.instruction(&Instruction::LocalGet(callback_kind_local));
            function.instruction(&Instruction::I64Const(kind.word() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_run_promise_reaction_callback(
                kind,
                reaction_record_local,
                reaction_is_rejected_local,
                argument_payload_local,
                argument_tag_local,
                function,
            )?;
            function.instruction(&Instruction::Else);
            open_dispatch_arms += 1;
        }
        function.instruction(&Instruction::Unreachable);
        for _ in 0..open_dispatch_arms {
            function.instruction(&Instruction::End);
        }

        self.release_temp_local(reaction_is_rejected_local);
        self.release_temp_local(reaction_type_word_local);
        self.release_temp_local(callback_kind_local);
        Ok(())
    }

    fn emit_run_default_promise_reaction_job(
        &mut self,
        reaction_record_local: u32,
        reaction_is_rejected_local: u32,
        argument_payload_local: u32,
        argument_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let capability_record_local = self.reserve_temp_local();
        let resolve_payload_local = self.reserve_temp_local();
        let resolve_tag_local = self.reserve_temp_local();
        let reject_payload_local = self.reserve_temp_local();
        let reject_tag_local = self.reserve_temp_local();
        let handler_payload_local = self.reserve_temp_local();
        let handler_tag_local = self.reserve_temp_local();
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();
        let undefined_payload_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();
        let call_payload_local = self.reserve_temp_local();
        let call_tag_local = self.reserve_temp_local();
        let selected_function_payload_local = self.reserve_temp_local();
        let selected_function_tag_local = self.reserve_temp_local();
        let selected_argument_payload_local = self.reserve_temp_local();
        let selected_argument_tag_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            reaction_record_local,
            HEAP_PROMISE_REACTION_CAPABILITY_OFFSET,
            capability_record_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_RESOLVE_PAYLOAD_OFFSET,
            resolve_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_RESOLVE_TAG_OFFSET,
            resolve_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_REJECT_PAYLOAD_OFFSET,
            reject_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_REJECT_TAG_OFFSET,
            reject_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            reaction_record_local,
            HEAP_PROMISE_REACTION_HANDLER_PAYLOAD_OFFSET,
            handler_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            reaction_record_local,
            HEAP_PROMISE_REACTION_HANDLER_TAG_OFFSET,
            handler_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));
        self.emit_is_callable_i32(handler_tag_local, handler_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_pre_evaluated_arg_vector(
            &[(argument_payload_local, argument_tag_local)],
            argc_local,
            argv_local,
            function,
        )?;
        self.emit_function_or_proxy_call_with_argv_leave_throw_completion(
            handler_payload_local,
            handler_tag_local,
            undefined_payload_local,
            undefined_tag_local,
            argc_local,
            argv_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(reject_payload_local));
        function.instruction(&Instruction::LocalSet(selected_function_payload_local));
        function.instruction(&Instruction::LocalGet(reject_tag_local));
        function.instruction(&Instruction::LocalSet(selected_function_tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(resolve_payload_local));
        function.instruction(&Instruction::LocalSet(selected_function_payload_local));
        function.instruction(&Instruction::LocalGet(resolve_tag_local));
        function.instruction(&Instruction::LocalSet(selected_function_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(call_payload_local));
        function.instruction(&Instruction::LocalSet(selected_argument_payload_local));
        function.instruction(&Instruction::LocalGet(call_tag_local));
        function.instruction(&Instruction::LocalSet(selected_argument_tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(reaction_is_rejected_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(resolve_payload_local));
        function.instruction(&Instruction::LocalSet(selected_function_payload_local));
        function.instruction(&Instruction::LocalGet(resolve_tag_local));
        function.instruction(&Instruction::LocalSet(selected_function_tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(reject_payload_local));
        function.instruction(&Instruction::LocalSet(selected_function_payload_local));
        function.instruction(&Instruction::LocalGet(reject_tag_local));
        function.instruction(&Instruction::LocalSet(selected_function_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(argument_payload_local));
        function.instruction(&Instruction::LocalSet(selected_argument_payload_local));
        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::LocalSet(selected_argument_tag_local));
        function.instruction(&Instruction::End);

        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_pre_evaluated_arg_vector(
            &[(selected_argument_payload_local, selected_argument_tag_local)],
            argc_local,
            argv_local,
            function,
        )?;
        self.emit_function_or_proxy_call_with_argv_leave_throw_completion(
            selected_function_payload_local,
            selected_function_tag_local,
            undefined_payload_local,
            undefined_tag_local,
            argc_local,
            argv_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        self.set_completion_kind(CompletionKind::Normal, function);

        self.release_temp_local(selected_argument_tag_local);
        self.release_temp_local(selected_argument_payload_local);
        self.release_temp_local(selected_function_tag_local);
        self.release_temp_local(selected_function_payload_local);
        self.release_temp_local(call_tag_local);
        self.release_temp_local(call_payload_local);
        self.release_temp_local(undefined_tag_local);
        self.release_temp_local(undefined_payload_local);
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(handler_tag_local);
        self.release_temp_local(handler_payload_local);
        self.release_temp_local(reject_tag_local);
        self.release_temp_local(reject_payload_local);
        self.release_temp_local(resolve_tag_local);
        self.release_temp_local(resolve_payload_local);
        self.release_temp_local(capability_record_local);
        Ok(())
    }

    fn emit_run_promise_thenable_job(
        &mut self,
        thenable_job_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let promise_record_local = self.reserve_temp_local();
        let promise_payload_local = self.reserve_temp_local();
        let thenable_payload_local = self.reserve_temp_local();
        let thenable_tag_local = self.reserve_temp_local();
        let then_payload_local = self.reserve_temp_local();
        let then_tag_local = self.reserve_temp_local();
        let resolving_context_local = self.reserve_temp_local();
        let resolve_function_local = self.reserve_temp_local();
        let reject_function_local = self.reserve_temp_local();
        let function_tag_local = self.reserve_temp_local();
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();
        let call_payload_local = self.reserve_temp_local();
        let call_tag_local = self.reserve_temp_local();
        let already_resolved_local = self.reserve_temp_local();

        for (offset, value_local) in [
            (
                HEAP_PROMISE_THENABLE_JOB_PROMISE_RECORD_OFFSET,
                promise_record_local,
            ),
            (
                HEAP_PROMISE_THENABLE_JOB_PROMISE_PAYLOAD_OFFSET,
                promise_payload_local,
            ),
            (
                HEAP_PROMISE_THENABLE_JOB_THENABLE_PAYLOAD_OFFSET,
                thenable_payload_local,
            ),
            (
                HEAP_PROMISE_THENABLE_JOB_THENABLE_TAG_OFFSET,
                thenable_tag_local,
            ),
            (
                HEAP_PROMISE_THENABLE_JOB_THEN_PAYLOAD_OFFSET,
                then_payload_local,
            ),
            (HEAP_PROMISE_THENABLE_JOB_THEN_TAG_OFFSET, then_tag_local),
        ] {
            self.load_i64_to_local_from_offset(thenable_job_local, offset, value_local, function);
        }
        self.emit_create_promise_resolving_functions(
            promise_payload_local,
            promise_record_local,
            resolving_context_local,
            resolve_function_local,
            reject_function_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(function_tag_local));
        self.emit_pre_evaluated_arg_vector(
            &[
                (resolve_function_local, function_tag_local),
                (reject_function_local, function_tag_local),
            ],
            argc_local,
            argv_local,
            function,
        )?;
        self.emit_function_or_proxy_call_with_argv_leave_throw_completion(
            then_payload_local,
            then_tag_local,
            thenable_payload_local,
            thenable_tag_local,
            argc_local,
            argv_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            resolving_context_local,
            HEAP_PROMISE_RESOLVING_CONTEXT_ALREADY_RESOLVED_OFFSET,
            already_resolved_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(already_resolved_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_const_at_offset(
            resolving_context_local,
            HEAP_PROMISE_RESOLVING_CONTEXT_ALREADY_RESOLVED_OFFSET,
            1,
            function,
        );
        self.emit_settle_promise_record(
            promise_record_local,
            PROMISE_STATE_REJECTED,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.set_completion_kind(CompletionKind::Normal, function);

        self.release_temp_local(already_resolved_local);
        self.release_temp_local(call_tag_local);
        self.release_temp_local(call_payload_local);
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(function_tag_local);
        self.release_temp_local(reject_function_local);
        self.release_temp_local(resolve_function_local);
        self.release_temp_local(resolving_context_local);
        self.release_temp_local(then_tag_local);
        self.release_temp_local(then_payload_local);
        self.release_temp_local(thenable_tag_local);
        self.release_temp_local(thenable_payload_local);
        self.release_temp_local(promise_payload_local);
        self.release_temp_local(promise_record_local);
        Ok(())
    }

    fn emit_run_promise_job(
        &mut self,
        kind: PromiseJobKind,
        callback_payload_local: u32,
        argument_payload_local: u32,
        argument_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match kind {
            PromiseJobKind::Reaction => self.emit_run_promise_reaction_job(
                callback_payload_local,
                argument_payload_local,
                argument_tag_local,
                function,
            ),
            PromiseJobKind::ResolveThenable => {
                self.emit_run_promise_thenable_job(callback_payload_local, function)
            }
        }
    }

    pub(crate) fn emit_drain_promise_jobs(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let saved_result_local = self.reserve_temp_local();
        let saved_result_tag_local = self.reserve_temp_local();
        let saved_completion_local = self.reserve_temp_local();
        let saved_completion_aux_local = self.reserve_temp_local();
        let saved_throw_error_name_local = self.reserve_temp_local();
        let saved_throw_error_message_local = self.reserve_temp_local();
        let saved_realm_local = self.reserve_temp_local();
        let job_record_local = self.reserve_temp_local();
        let next_job_local = self.reserve_temp_local();
        let reaction_record_local = self.reserve_temp_local();
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let job_realm_local = self.reserve_temp_local();
        let job_kind_local = self.reserve_temp_local();

        for (source, destination) in [
            (self.result_local, saved_result_local),
            (self.result_tag_local, saved_result_tag_local),
            (self.completion_local, saved_completion_local),
            (self.completion_aux_local, saved_completion_aux_local),
        ] {
            function.instruction(&Instruction::LocalGet(source));
            function.instruction(&Instruction::LocalSet(destination));
        }
        function.instruction(&Instruction::GlobalGet(throw_error_name_global_index(
            self.uses_heap,
        )));
        function.instruction(&Instruction::LocalSet(saved_throw_error_name_local));
        function.instruction(&Instruction::GlobalGet(throw_error_message_global_index(
            self.uses_heap,
        )));
        function.instruction(&Instruction::LocalSet(saved_throw_error_message_local));
        function.instruction(&Instruction::GlobalGet(CURRENT_REALM_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(saved_realm_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(PROMISE_JOB_QUEUE_HEAD_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(job_record_local));
        function.instruction(&Instruction::LocalGet(job_record_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        self.load_i64_to_local_from_offset(
            job_record_local,
            HEAP_PENDING_JOB_NEXT_OFFSET,
            next_job_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(next_job_local));
        function.instruction(&Instruction::GlobalSet(PROMISE_JOB_QUEUE_HEAD_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalGet(next_job_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::GlobalSet(PROMISE_JOB_QUEUE_TAIL_GLOBAL_INDEX));
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            job_record_local,
            HEAP_PENDING_JOB_CALLBACK_PAYLOAD_OFFSET,
            reaction_record_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            job_record_local,
            HEAP_PENDING_JOB_ARG_PAYLOAD_OFFSET,
            argument_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            job_record_local,
            HEAP_PENDING_JOB_ARG_TAG_OFFSET,
            argument_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            job_record_local,
            HEAP_PENDING_JOB_REALM_OFFSET,
            job_realm_local,
            function,
        );
        // A null Promise-job realm means the job evaluates no handler code.
        // Restore the host checkpoint realm for that job rather than leaking
        // the previous queued job's realm or installing the null sentinel.
        function.instruction(&Instruction::LocalGet(saved_realm_local));
        function.instruction(&Instruction::GlobalSet(CURRENT_REALM_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalGet(job_realm_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(job_realm_local));
        function.instruction(&Instruction::GlobalSet(CURRENT_REALM_GLOBAL_INDEX));
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            job_record_local,
            HEAP_PENDING_JOB_KIND_OFFSET,
            job_kind_local,
            function,
        );
        let mut open_dispatch_arms = 0;
        for kind in PromiseJobKind::ALL {
            function.instruction(&Instruction::LocalGet(job_kind_local));
            function.instruction(&Instruction::I64Const(kind.word() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_run_promise_job(
                kind,
                reaction_record_local,
                argument_payload_local,
                argument_tag_local,
                function,
            )?;
            function.instruction(&Instruction::Else);
            open_dispatch_arms += 1;
        }
        function.instruction(&Instruction::Unreachable);
        for _ in 0..open_dispatch_arms {
            function.instruction(&Instruction::End);
        }
        if self
            .functions
            .monotonic_clock_nanos_import_function_index()
            .is_some()
        {
            self.emit_poll_atomics_wait_async_timeouts(function)?;
            function.instruction(&Instruction::Drop);
        }
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(saved_realm_local));
        function.instruction(&Instruction::GlobalSet(CURRENT_REALM_GLOBAL_INDEX));
        for (source, destination) in [
            (saved_result_local, self.result_local),
            (saved_result_tag_local, self.result_tag_local),
            (saved_completion_local, self.completion_local),
            (saved_completion_aux_local, self.completion_aux_local),
        ] {
            function.instruction(&Instruction::LocalGet(source));
            function.instruction(&Instruction::LocalSet(destination));
        }
        function.instruction(&Instruction::LocalGet(saved_throw_error_name_local));
        function.instruction(&Instruction::GlobalSet(throw_error_name_global_index(
            self.uses_heap,
        )));
        function.instruction(&Instruction::LocalGet(saved_throw_error_message_local));
        function.instruction(&Instruction::GlobalSet(throw_error_message_global_index(
            self.uses_heap,
        )));

        self.release_temp_local(job_kind_local);
        self.release_temp_local(job_realm_local);
        self.release_temp_local(argument_tag_local);
        self.release_temp_local(argument_payload_local);
        self.release_temp_local(reaction_record_local);
        self.release_temp_local(next_job_local);
        self.release_temp_local(job_record_local);
        self.release_temp_local(saved_realm_local);
        self.release_temp_local(saved_throw_error_message_local);
        self.release_temp_local(saved_throw_error_name_local);
        self.release_temp_local(saved_completion_aux_local);
        self.release_temp_local(saved_completion_local);
        self.release_temp_local(saved_result_tag_local);
        self.release_temp_local(saved_result_local);
        Ok(())
    }

    pub(crate) fn emit_promise_capability_executor(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let capability_record_local = self.reserve_temp_local();
        let resolve_payload_local = self.reserve_temp_local();
        let resolve_tag_local = self.reserve_temp_local();
        let reject_payload_local = self.reserve_temp_local();
        let reject_tag_local = self.reserve_temp_local();
        let existing_resolve_tag_local = self.reserve_temp_local();
        let existing_reject_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(0));
        function.instruction(&Instruction::LocalSet(capability_record_local));
        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_RESOLVE_TAG_OFFSET,
            existing_resolve_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_REJECT_TAG_OFFSET,
            existing_reject_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(existing_resolve_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(existing_reject_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Promise capability executor called more than once",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_builtin_arg_to_locals(0, resolve_payload_local, resolve_tag_local, function);
        self.emit_builtin_arg_to_locals(1, reject_payload_local, reject_tag_local, function);
        self.store_i64_local_at_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_RESOLVE_TAG_OFFSET,
            resolve_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_RESOLVE_PAYLOAD_OFFSET,
            resolve_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_REJECT_TAG_OFFSET,
            reject_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_REJECT_PAYLOAD_OFFSET,
            reject_payload_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);

        self.release_temp_local(existing_reject_tag_local);
        self.release_temp_local(existing_resolve_tag_local);
        self.release_temp_local(reject_tag_local);
        self.release_temp_local(reject_payload_local);
        self.release_temp_local(resolve_tag_local);
        self.release_temp_local(resolve_payload_local);
        self.release_temp_local(capability_record_local);
        Ok(())
    }

    pub(crate) fn emit_promise_static_settle(
        &mut self,
        state: u64,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let constructor_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Promise static method receiver",
            )
        })?;
        let constructor_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Promise static method receiver tag",
            )
        })?;
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let capability_record_local = self.reserve_temp_local();
        let promise_payload_local = self.reserve_temp_local();
        let promise_tag_local = self.reserve_temp_local();
        let resolve_payload_local = self.reserve_temp_local();
        let resolve_tag_local = self.reserve_temp_local();
        let reject_payload_local = self.reserve_temp_local();
        let reject_tag_local = self.reserve_temp_local();
        let call_payload_local = self.reserve_temp_local();
        let call_tag_local = self.reserve_temp_local();
        let undefined_payload_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, value_payload_local, value_tag_local, function);
        if state == PROMISE_STATE_FULFILLED {
            self.emit_is_heap_object_like_tag_i32(constructor_tag_local, function);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_current_function_realm_type_error(
                "Promise.resolve receiver is not an object",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);

            let brand_local = self.reserve_temp_local();
            let constructor_key_local = self.reserve_temp_local();
            let value_constructor_payload_local = self.reserve_temp_local();
            let value_constructor_tag_local = self.reserve_temp_local();
            function.instruction(&Instruction::LocalGet(value_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.load_i64_to_local_from_offset(
                value_payload_local,
                HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
                brand_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(brand_local));
            function.instruction(&Instruction::I64Const(OBJECT_INTERNAL_BRAND_PROMISE as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
            function.instruction(&Instruction::LocalSet(constructor_key_local));
            self.emit_object_read(
                value_payload_local,
                value_tag_local,
                value_payload_local,
                value_tag_local,
                constructor_key_local,
                value_constructor_payload_local,
                value_constructor_tag_local,
                function,
            )?;
            function.instruction(&Instruction::LocalGet(self.completion_local));
            function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(value_constructor_payload_local));
            function.instruction(&Instruction::LocalSet(self.result_local));
            function.instruction(&Instruction::LocalGet(value_constructor_tag_local));
            function.instruction(&Instruction::LocalSet(self.result_tag_local));
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalGet(value_constructor_tag_local));
            function.instruction(&Instruction::LocalGet(constructor_tag_local));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::LocalGet(value_constructor_payload_local));
            function.instruction(&Instruction::LocalGet(constructor_payload_local));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(value_payload_local));
            function.instruction(&Instruction::LocalSet(self.result_local));
            function.instruction(&Instruction::LocalGet(value_tag_local));
            function.instruction(&Instruction::LocalSet(self.result_tag_local));
            self.set_completion_kind(CompletionKind::Normal, function);
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            self.release_temp_local(value_constructor_tag_local);
            self.release_temp_local(value_constructor_payload_local);
            self.release_temp_local(constructor_key_local);
            self.release_temp_local(brand_local);
        }

        self.emit_new_promise_capability(
            constructor_payload_local,
            constructor_tag_local,
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            function,
        )?;
        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_RESOLVE_PAYLOAD_OFFSET,
            resolve_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_RESOLVE_TAG_OFFSET,
            resolve_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_REJECT_PAYLOAD_OFFSET,
            reject_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_REJECT_TAG_OFFSET,
            reject_tag_local,
            function,
        );
        self.emit_is_callable_i32(resolve_tag_local, resolve_payload_local, function)?;
        self.emit_is_callable_i32(reject_tag_local, reject_payload_local, function)?;
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Promise capability did not initialize callable resolving functions",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));
        self.emit_function_or_proxy_call_leave_throw_completion(
            if state == PROMISE_STATE_FULFILLED {
                resolve_payload_local
            } else {
                reject_payload_local
            },
            if state == PROMISE_STATE_FULFILLED {
                resolve_tag_local
            } else {
                reject_tag_local
            },
            undefined_payload_local,
            undefined_tag_local,
            &[(value_payload_local, value_tag_local)],
            call_payload_local,
            call_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(call_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(call_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(promise_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(promise_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);

        self.release_temp_local(undefined_tag_local);
        self.release_temp_local(undefined_payload_local);
        self.release_temp_local(call_tag_local);
        self.release_temp_local(call_payload_local);
        self.release_temp_local(reject_tag_local);
        self.release_temp_local(reject_payload_local);
        self.release_temp_local(resolve_tag_local);
        self.release_temp_local(resolve_payload_local);
        self.release_temp_local(promise_tag_local);
        self.release_temp_local(promise_payload_local);
        self.release_temp_local(capability_record_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        Ok(())
    }

    pub(crate) fn emit_promise_with_resolvers(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let constructor_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Promise.withResolvers receiver",
            )
        })?;
        let constructor_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Promise.withResolvers receiver tag",
            )
        })?;
        let capability_record_local = self.reserve_temp_local();
        let promise_payload_local = self.reserve_temp_local();
        let promise_tag_local = self.reserve_temp_local();
        let resolve_payload_local = self.reserve_temp_local();
        let resolve_tag_local = self.reserve_temp_local();
        let reject_payload_local = self.reserve_temp_local();
        let reject_tag_local = self.reserve_temp_local();
        let result_object_local = self.reserve_temp_local();

        self.emit_new_promise_capability(
            constructor_payload_local,
            constructor_tag_local,
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            function,
        )?;
        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_RESOLVE_PAYLOAD_OFFSET,
            resolve_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_RESOLVE_TAG_OFFSET,
            resolve_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_REJECT_PAYLOAD_OFFSET,
            reject_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_REJECT_TAG_OFFSET,
            reject_tag_local,
            function,
        );

        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(result_object_local));
        self.emit_object_define_local_data_with_flags(
            result_object_local,
            "promise",
            promise_payload_local,
            promise_tag_local,
            true,
            true,
            true,
            function,
        )?;
        self.emit_object_define_local_data_with_flags(
            result_object_local,
            "resolve",
            resolve_payload_local,
            resolve_tag_local,
            true,
            true,
            true,
            function,
        )?;
        self.emit_object_define_local_data_with_flags(
            result_object_local,
            "reject",
            reject_payload_local,
            reject_tag_local,
            true,
            true,
            true,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(result_object_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);

        self.release_temp_local(result_object_local);
        self.release_temp_local(reject_tag_local);
        self.release_temp_local(reject_payload_local);
        self.release_temp_local(resolve_tag_local);
        self.release_temp_local(resolve_payload_local);
        self.release_temp_local(promise_tag_local);
        self.release_temp_local(promise_payload_local);
        self.release_temp_local(capability_record_local);
        Ok(())
    }

    pub(crate) fn emit_promise_try(&mut self, function: &mut Function) -> Result<(), EmitError> {
        let constructor_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Promise.try receiver",
            )
        })?;
        let constructor_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Promise.try receiver tag",
            )
        })?;
        let capability_record_local = self.reserve_temp_local();
        let promise_payload_local = self.reserve_temp_local();
        let promise_tag_local = self.reserve_temp_local();
        let callback_payload_local = self.reserve_temp_local();
        let callback_tag_local = self.reserve_temp_local();
        let callback_argc_local = self.reserve_temp_local();
        let callback_argv_local = self.reserve_temp_local();
        let callback_arg_index_local = self.reserve_temp_local();
        let source_arg_index_local = self.reserve_temp_local();
        let arg_payload_local = self.reserve_temp_local();
        let arg_tag_local = self.reserve_temp_local();
        let undefined_payload_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();
        let callback_result_payload_local = self.reserve_temp_local();
        let callback_result_tag_local = self.reserve_temp_local();
        let settle_payload_local = self.reserve_temp_local();
        let settle_tag_local = self.reserve_temp_local();
        let settle_call_payload_local = self.reserve_temp_local();
        let settle_call_tag_local = self.reserve_temp_local();

        self.emit_new_promise_capability(
            constructor_payload_local,
            constructor_tag_local,
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            function,
        )?;
        self.emit_builtin_arg_to_locals(0, callback_payload_local, callback_tag_local, function);

        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(callback_argc_local));
        self.emit_alloc_array_payload_with_length(
            callback_argc_local,
            callback_argv_local,
            function,
        )?;

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(callback_arg_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(callback_arg_index_local));
        function.instruction(&Instruction::LocalGet(callback_argc_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(callback_arg_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(source_arg_index_local));
        self.emit_array_read(
            self.argv_param_local(),
            source_arg_index_local,
            arg_payload_local,
            arg_tag_local,
            function,
        );
        self.emit_array_write(
            callback_argv_local,
            callback_arg_index_local,
            arg_payload_local,
            arg_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(callback_arg_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(callback_arg_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));
        self.emit_function_or_proxy_call_with_argv_leave_throw_completion(
            callback_payload_local,
            callback_tag_local,
            undefined_payload_local,
            undefined_tag_local,
            callback_argc_local,
            callback_argv_local,
            callback_result_payload_local,
            callback_result_tag_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_REJECT_PAYLOAD_OFFSET,
            settle_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_REJECT_TAG_OFFSET,
            settle_tag_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_RESOLVE_PAYLOAD_OFFSET,
            settle_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_RESOLVE_TAG_OFFSET,
            settle_tag_local,
            function,
        );
        function.instruction(&Instruction::End);
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_function_or_proxy_call_leave_throw_completion(
            settle_payload_local,
            settle_tag_local,
            undefined_payload_local,
            undefined_tag_local,
            &[(callback_result_payload_local, callback_result_tag_local)],
            settle_call_payload_local,
            settle_call_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(settle_call_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(settle_call_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(promise_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(promise_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);

        self.release_temp_local(settle_call_tag_local);
        self.release_temp_local(settle_call_payload_local);
        self.release_temp_local(settle_tag_local);
        self.release_temp_local(settle_payload_local);
        self.release_temp_local(callback_result_tag_local);
        self.release_temp_local(callback_result_payload_local);
        self.release_temp_local(undefined_tag_local);
        self.release_temp_local(undefined_payload_local);
        self.release_temp_local(arg_tag_local);
        self.release_temp_local(arg_payload_local);
        self.release_temp_local(source_arg_index_local);
        self.release_temp_local(callback_arg_index_local);
        self.release_temp_local(callback_argv_local);
        self.release_temp_local(callback_argc_local);
        self.release_temp_local(callback_tag_local);
        self.release_temp_local(callback_payload_local);
        self.release_temp_local(promise_tag_local);
        self.release_temp_local(promise_payload_local);
        self.release_temp_local(capability_record_local);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_promise_combinator_reject_current_throw(
        &mut self,
        capability_record_local: u32,
        promise_payload_local: u32,
        promise_tag_local: u32,
        iterator_acquired_local: u32,
        iterator_close: IteratorCloseOnThrowLocals,
        error_payload_local: u32,
        error_tag_local: u32,
        call_payload_local: u32,
        call_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let reject_payload_local = self.reserve_temp_local();
        let reject_tag_local = self.reserve_temp_local();
        let undefined_payload_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(iterator_acquired_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_iterator_close_preserving_current_throw(iterator_close, function)?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalSet(error_payload_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalSet(error_tag_local));
        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_REJECT_PAYLOAD_OFFSET,
            reject_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_REJECT_TAG_OFFSET,
            reject_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_function_or_proxy_call_leave_throw_completion(
            reject_payload_local,
            reject_tag_local,
            undefined_payload_local,
            undefined_tag_local,
            &[(error_payload_local, error_tag_local)],
            call_payload_local,
            call_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(promise_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(promise_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.release_temp_local(undefined_tag_local);
        self.release_temp_local(undefined_payload_local);
        self.release_temp_local(reject_tag_local);
        self.release_temp_local(reject_payload_local);
        Ok(())
    }

    pub(crate) fn emit_promise_all_keyed_resolve_element(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_promise_all_keyed_element(false, PROMISE_STATE_FULFILLED, function)
    }

    pub(crate) fn emit_promise_all_settled_keyed_element(
        &mut self,
        state: u64,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_promise_all_keyed_element(true, state, function)
    }

    fn emit_promise_all_keyed_element(
        &mut self,
        settled_record: bool,
        state: u64,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let element_context_local = self.reserve_temp_local();
        let already_called_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let shared_context_local = self.reserve_temp_local();
        let result_payload_local = self.reserve_temp_local();
        let remaining_local = self.reserve_temp_local();
        let resolve_payload_local = self.reserve_temp_local();
        let resolve_tag_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let undefined_payload_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();
        let call_payload_local = self.reserve_temp_local();
        let call_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(0));
        function.instruction(&Instruction::LocalSet(element_context_local));
        self.load_i64_to_local_from_offset(
            element_context_local,
            HEAP_PROMISE_KEYED_ELEMENT_ALREADY_CALLED_OFFSET,
            already_called_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(already_called_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_const_at_offset(
            element_context_local,
            HEAP_PROMISE_KEYED_ELEMENT_ALREADY_CALLED_OFFSET,
            1,
            function,
        );
        self.load_i64_to_local_from_offset(
            element_context_local,
            HEAP_PROMISE_KEYED_ELEMENT_KEY_PAYLOAD_OFFSET,
            key_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            element_context_local,
            HEAP_PROMISE_KEYED_ELEMENT_KEY_TAG_OFFSET,
            key_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            element_context_local,
            HEAP_PROMISE_KEYED_ELEMENT_SHARED_OFFSET,
            shared_context_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_VALUES_OFFSET,
            result_payload_local,
            function,
        );
        self.emit_builtin_arg_to_locals(0, value_payload_local, value_tag_local, function);

        if settled_record {
            let record_payload_local = self.reserve_temp_local();
            let status_payload_local = self.reserve_temp_local();
            let status_tag_local = self.reserve_temp_local();
            let (status, result_property) = if state == PROMISE_STATE_FULFILLED {
                ("fulfilled", "value")
            } else {
                ("rejected", "reason")
            };
            self.emit_alloc_plain_object_with_prototype(
                None,
                Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
                function,
            )?;
            function.instruction(&Instruction::LocalSet(record_payload_local));
            function.instruction(&Instruction::I64Const(self.strings.payload(status)));
            function.instruction(&Instruction::LocalSet(status_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
            function.instruction(&Instruction::LocalSet(status_tag_local));
            self.emit_object_define_local_data_with_flags(
                record_payload_local,
                "status",
                status_payload_local,
                status_tag_local,
                true,
                true,
                true,
                function,
            )?;
            self.emit_object_define_local_data_with_flags(
                record_payload_local,
                result_property,
                value_payload_local,
                value_tag_local,
                true,
                true,
                true,
                function,
            )?;
            function.instruction(&Instruction::LocalGet(record_payload_local));
            function.instruction(&Instruction::LocalSet(value_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
            function.instruction(&Instruction::LocalSet(value_tag_local));
            self.release_temp_local(status_tag_local);
            self.release_temp_local(status_payload_local);
            self.release_temp_local(record_payload_local);
        }
        self.emit_object_define_enumerable_data(
            result_payload_local,
            key_payload_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;

        self.load_i64_to_local_from_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_REMAINING_OFFSET,
            remaining_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(remaining_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(remaining_local));
        self.store_i64_local_at_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_REMAINING_OFFSET,
            remaining_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(remaining_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_RESOLVE_PAYLOAD_OFFSET,
            resolve_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_RESOLVE_TAG_OFFSET,
            resolve_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_function_or_proxy_call_leave_throw_completion(
            resolve_payload_local,
            resolve_tag_local,
            undefined_payload_local,
            undefined_tag_local,
            &[(result_payload_local, value_tag_local)],
            call_payload_local,
            call_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(call_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(call_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);

        for local in [
            call_tag_local,
            call_payload_local,
            undefined_tag_local,
            undefined_payload_local,
            value_tag_local,
            value_payload_local,
            resolve_tag_local,
            resolve_payload_local,
            remaining_local,
            result_payload_local,
            shared_context_local,
            key_tag_local,
            key_payload_local,
            already_called_local,
            element_context_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn emit_promise_all_resolve_element(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let element_context_local = self.reserve_temp_local();
        let already_called_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let shared_context_local = self.reserve_temp_local();
        let values_payload_local = self.reserve_temp_local();
        let remaining_local = self.reserve_temp_local();
        let resolve_payload_local = self.reserve_temp_local();
        let resolve_tag_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let undefined_payload_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();
        let call_payload_local = self.reserve_temp_local();
        let call_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(0));
        function.instruction(&Instruction::LocalSet(element_context_local));
        self.load_i64_to_local_from_offset(
            element_context_local,
            HEAP_PROMISE_ALL_ELEMENT_ALREADY_CALLED_OFFSET,
            already_called_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(already_called_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_const_at_offset(
            element_context_local,
            HEAP_PROMISE_ALL_ELEMENT_ALREADY_CALLED_OFFSET,
            1,
            function,
        );
        self.load_i64_to_local_from_offset(
            element_context_local,
            HEAP_PROMISE_ALL_ELEMENT_INDEX_OFFSET,
            index_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            element_context_local,
            HEAP_PROMISE_ALL_ELEMENT_SHARED_OFFSET,
            shared_context_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_VALUES_OFFSET,
            values_payload_local,
            function,
        );
        self.emit_builtin_arg_to_locals(0, value_payload_local, value_tag_local, function);
        self.emit_array_write(
            values_payload_local,
            index_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;

        self.load_i64_to_local_from_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_REMAINING_OFFSET,
            remaining_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(remaining_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(remaining_local));
        self.store_i64_local_at_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_REMAINING_OFFSET,
            remaining_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(remaining_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_RESOLVE_PAYLOAD_OFFSET,
            resolve_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_RESOLVE_TAG_OFFSET,
            resolve_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_function_or_proxy_call_leave_throw_completion(
            resolve_payload_local,
            resolve_tag_local,
            undefined_payload_local,
            undefined_tag_local,
            &[(values_payload_local, value_tag_local)],
            call_payload_local,
            call_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(call_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(call_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);

        self.release_temp_local(call_tag_local);
        self.release_temp_local(call_payload_local);
        self.release_temp_local(undefined_tag_local);
        self.release_temp_local(undefined_payload_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(resolve_tag_local);
        self.release_temp_local(resolve_payload_local);
        self.release_temp_local(remaining_local);
        self.release_temp_local(values_payload_local);
        self.release_temp_local(shared_context_local);
        self.release_temp_local(index_local);
        self.release_temp_local(already_called_local);
        self.release_temp_local(element_context_local);
        Ok(())
    }

    pub(crate) fn emit_promise_all_settled_element(
        &mut self,
        state: u64,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let element_context_local = self.reserve_temp_local();
        let already_called_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let shared_context_local = self.reserve_temp_local();
        let values_payload_local = self.reserve_temp_local();
        let remaining_local = self.reserve_temp_local();
        let resolve_payload_local = self.reserve_temp_local();
        let resolve_tag_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let record_payload_local = self.reserve_temp_local();
        let status_payload_local = self.reserve_temp_local();
        let status_tag_local = self.reserve_temp_local();
        let undefined_payload_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();
        let call_payload_local = self.reserve_temp_local();
        let call_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(0));
        function.instruction(&Instruction::LocalSet(element_context_local));
        self.load_i64_to_local_from_offset(
            element_context_local,
            HEAP_PROMISE_ALL_ELEMENT_ALREADY_CALLED_OFFSET,
            already_called_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(already_called_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_const_at_offset(
            element_context_local,
            HEAP_PROMISE_ALL_ELEMENT_ALREADY_CALLED_OFFSET,
            1,
            function,
        );
        self.load_i64_to_local_from_offset(
            element_context_local,
            HEAP_PROMISE_ALL_ELEMENT_INDEX_OFFSET,
            index_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            element_context_local,
            HEAP_PROMISE_ALL_ELEMENT_SHARED_OFFSET,
            shared_context_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_VALUES_OFFSET,
            values_payload_local,
            function,
        );
        self.emit_builtin_arg_to_locals(0, value_payload_local, value_tag_local, function);

        let (status, result_property) = if state == PROMISE_STATE_FULFILLED {
            ("fulfilled", "value")
        } else {
            ("rejected", "reason")
        };
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(record_payload_local));
        function.instruction(&Instruction::I64Const(self.strings.payload(status)));
        function.instruction(&Instruction::LocalSet(status_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(status_tag_local));
        self.emit_object_define_local_data_with_flags(
            record_payload_local,
            "status",
            status_payload_local,
            status_tag_local,
            true,
            true,
            true,
            function,
        )?;
        self.emit_object_define_local_data_with_flags(
            record_payload_local,
            result_property,
            value_payload_local,
            value_tag_local,
            true,
            true,
            true,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_array_write(
            values_payload_local,
            index_local,
            record_payload_local,
            value_tag_local,
            function,
        )?;

        self.load_i64_to_local_from_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_REMAINING_OFFSET,
            remaining_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(remaining_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(remaining_local));
        self.store_i64_local_at_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_REMAINING_OFFSET,
            remaining_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(remaining_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_RESOLVE_PAYLOAD_OFFSET,
            resolve_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_RESOLVE_TAG_OFFSET,
            resolve_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_function_or_proxy_call_leave_throw_completion(
            resolve_payload_local,
            resolve_tag_local,
            undefined_payload_local,
            undefined_tag_local,
            &[(values_payload_local, value_tag_local)],
            call_payload_local,
            call_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(call_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(call_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);

        for local in [
            call_tag_local,
            call_payload_local,
            undefined_tag_local,
            undefined_payload_local,
            status_tag_local,
            status_payload_local,
            record_payload_local,
            value_tag_local,
            value_payload_local,
            resolve_tag_local,
            resolve_payload_local,
            remaining_local,
            values_payload_local,
            shared_context_local,
            index_local,
            already_called_local,
            element_context_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn emit_promise_any_reject_element(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let element_context_local = self.reserve_temp_local();
        let already_called_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let shared_context_local = self.reserve_temp_local();
        let errors_payload_local = self.reserve_temp_local();
        let remaining_local = self.reserve_temp_local();
        let reject_payload_local = self.reserve_temp_local();
        let reject_tag_local = self.reserve_temp_local();
        let reason_payload_local = self.reserve_temp_local();
        let reason_tag_local = self.reserve_temp_local();
        let aggregate_prototype_local = self.reserve_temp_local();
        let aggregate_payload_local = self.reserve_temp_local();
        let aggregate_tag_local = self.reserve_temp_local();
        let undefined_payload_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();
        let call_payload_local = self.reserve_temp_local();
        let call_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(0));
        function.instruction(&Instruction::LocalSet(element_context_local));
        self.load_i64_to_local_from_offset(
            element_context_local,
            HEAP_PROMISE_ALL_ELEMENT_ALREADY_CALLED_OFFSET,
            already_called_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(already_called_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_const_at_offset(
            element_context_local,
            HEAP_PROMISE_ALL_ELEMENT_ALREADY_CALLED_OFFSET,
            1,
            function,
        );
        self.load_i64_to_local_from_offset(
            element_context_local,
            HEAP_PROMISE_ALL_ELEMENT_INDEX_OFFSET,
            index_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            element_context_local,
            HEAP_PROMISE_ALL_ELEMENT_SHARED_OFFSET,
            shared_context_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_VALUES_OFFSET,
            errors_payload_local,
            function,
        );
        self.emit_builtin_arg_to_locals(0, reason_payload_local, reason_tag_local, function);
        self.emit_array_write(
            errors_payload_local,
            index_local,
            reason_payload_local,
            reason_tag_local,
            function,
        )?;

        self.load_i64_to_local_from_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_REMAINING_OFFSET,
            remaining_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(remaining_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(remaining_local));
        self.store_i64_local_at_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_REMAINING_OFFSET,
            remaining_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(remaining_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_RESOLVE_PAYLOAD_OFFSET,
            reject_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_RESOLVE_TAG_OFFSET,
            reject_tag_local,
            function,
        );
        function.instruction(&Instruction::GlobalGet(
            AGGREGATE_ERROR_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(aggregate_prototype_local));
        self.emit_alloc_aggregate_error_instance_from_locals(
            None,
            errors_payload_local,
            aggregate_prototype_local,
            aggregate_payload_local,
            aggregate_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));
        self.emit_function_or_proxy_call_leave_throw_completion(
            reject_payload_local,
            reject_tag_local,
            undefined_payload_local,
            undefined_tag_local,
            &[(aggregate_payload_local, aggregate_tag_local)],
            call_payload_local,
            call_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(call_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(call_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);

        for local in [
            call_tag_local,
            call_payload_local,
            undefined_tag_local,
            undefined_payload_local,
            aggregate_tag_local,
            aggregate_payload_local,
            aggregate_prototype_local,
            reason_tag_local,
            reason_payload_local,
            reject_tag_local,
            reject_payload_local,
            remaining_local,
            errors_payload_local,
            shared_context_local,
            index_local,
            already_called_local,
            element_context_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn emit_promise_race(&mut self, function: &mut Function) -> Result<(), EmitError> {
        let constructor_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Promise.race receiver",
            )
        })?;
        let constructor_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Promise.race receiver tag",
            )
        })?;
        let capability_record_local = self.reserve_temp_local();
        let promise_payload_local = self.reserve_temp_local();
        let promise_tag_local = self.reserve_temp_local();
        let resolve_payload_local = self.reserve_temp_local();
        let resolve_tag_local = self.reserve_temp_local();
        let reject_payload_local = self.reserve_temp_local();
        let reject_tag_local = self.reserve_temp_local();
        let promise_resolve_payload_local = self.reserve_temp_local();
        let promise_resolve_tag_local = self.reserve_temp_local();
        let iterable_payload_local = self.reserve_temp_local();
        let iterable_tag_local = self.reserve_temp_local();
        let iterable_object_payload_local = self.reserve_temp_local();
        let iterable_object_tag_local = self.reserve_temp_local();
        let iterator_method_payload_local = self.reserve_temp_local();
        let iterator_method_tag_local = self.reserve_temp_local();
        let iterator_payload_local = self.reserve_temp_local();
        let iterator_tag_local = self.reserve_temp_local();
        let next_payload_local = self.reserve_temp_local();
        let next_tag_local = self.reserve_temp_local();
        let iterator_acquired_local = self.reserve_temp_local();
        let next_result_payload_local = self.reserve_temp_local();
        let next_result_tag_local = self.reserve_temp_local();
        let done_payload_local = self.reserve_temp_local();
        let done_tag_local = self.reserve_temp_local();
        let next_value_payload_local = self.reserve_temp_local();
        let next_value_tag_local = self.reserve_temp_local();
        let next_promise_payload_local = self.reserve_temp_local();
        let next_promise_tag_local = self.reserve_temp_local();
        let then_payload_local = self.reserve_temp_local();
        let then_tag_local = self.reserve_temp_local();
        let call_payload_local = self.reserve_temp_local();
        let call_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let undefined_payload_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();
        let error_payload_local = self.reserve_temp_local();
        let error_tag_local = self.reserve_temp_local();
        let close_return_payload_local = self.reserve_temp_local();
        let close_return_tag_local = self.reserve_temp_local();
        let close_result_payload_local = self.reserve_temp_local();
        let close_result_tag_local = self.reserve_temp_local();
        let close_saved_payload_local = self.reserve_temp_local();
        let close_saved_tag_local = self.reserve_temp_local();
        let close_saved_completion_local = self.reserve_temp_local();
        let close_saved_aux_local = self.reserve_temp_local();

        let iterator_close = IteratorCloseOnThrowLocals {
            iterator_payload_local,
            iterator_tag_local,
            key_local,
            return_payload_local: close_return_payload_local,
            return_tag_local: close_return_tag_local,
            result_payload_local: close_result_payload_local,
            result_tag_local: close_result_tag_local,
            saved_payload_local: close_saved_payload_local,
            saved_tag_local: close_saved_tag_local,
            saved_completion_local: close_saved_completion_local,
            saved_aux_local: close_saved_aux_local,
        };
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(iterator_acquired_local));
        self.emit_new_promise_capability(
            constructor_payload_local,
            constructor_tag_local,
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            function,
        )?;
        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_RESOLVE_PAYLOAD_OFFSET,
            resolve_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_RESOLVE_TAG_OFFSET,
            resolve_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_REJECT_PAYLOAD_OFFSET,
            reject_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_REJECT_TAG_OFFSET,
            reject_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));

        function.instruction(&Instruction::I64Const(self.strings.payload("resolve")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read_without_throw_propagation(
            constructor_payload_local,
            constructor_tag_local,
            constructor_payload_local,
            constructor_tag_local,
            key_local,
            promise_resolve_payload_local,
            promise_resolve_tag_local,
            function,
        )?;
        self.emit_promise_combinator_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            iterator_acquired_local,
            iterator_close,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        self.emit_is_callable_i32(
            promise_resolve_tag_local,
            promise_resolve_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Promise.race constructor resolve property is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_promise_combinator_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            iterator_acquired_local,
            iterator_close,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.emit_builtin_arg_to_locals(0, iterable_payload_local, iterable_tag_local, function);
        function.instruction(&Instruction::LocalGet(iterable_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(iterable_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Promise.race input is not iterable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_promise_combinator_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            iterator_acquired_local,
            iterator_close,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_value_to_current_function_realm_object_locals(
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
        self.emit_object_read_without_throw_propagation(
            iterable_object_payload_local,
            iterable_object_tag_local,
            iterable_payload_local,
            iterable_tag_local,
            key_local,
            iterator_method_payload_local,
            iterator_method_tag_local,
            function,
        )?;
        self.emit_promise_combinator_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            iterator_acquired_local,
            iterator_close,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        self.emit_is_callable_i32(
            iterator_method_tag_local,
            iterator_method_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Promise.race iterator method is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_promise_combinator_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            iterator_acquired_local,
            iterator_close,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_function_or_proxy_call_leave_throw_completion(
            iterator_method_payload_local,
            iterator_method_tag_local,
            iterable_payload_local,
            iterable_tag_local,
            &[],
            iterator_payload_local,
            iterator_tag_local,
            function,
        )?;
        self.emit_promise_combinator_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            iterator_acquired_local,
            iterator_close,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        self.emit_is_heap_object_like_tag_i32(iterator_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Promise.race iterator method must return an object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_promise_combinator_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            iterator_acquired_local,
            iterator_close,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(self.strings.payload("next")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read_without_throw_propagation(
            iterator_payload_local,
            iterator_tag_local,
            iterator_payload_local,
            iterator_tag_local,
            key_local,
            next_payload_local,
            next_tag_local,
            function,
        )?;
        self.emit_promise_combinator_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            iterator_acquired_local,
            iterator_close,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        self.emit_is_callable_i32(next_tag_local, next_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Promise.race iterator next method is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_promise_combinator_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            iterator_acquired_local,
            iterator_close,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(iterator_acquired_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.emit_function_or_proxy_call_leave_throw_completion(
            next_payload_local,
            next_tag_local,
            iterator_payload_local,
            iterator_tag_local,
            &[],
            next_result_payload_local,
            next_result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(iterator_acquired_local));
        function.instruction(&Instruction::End);
        self.emit_promise_combinator_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            iterator_acquired_local,
            iterator_close,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        self.emit_is_heap_object_like_tag_i32(next_result_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(iterator_acquired_local));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Promise.race iterator next result must be an object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_promise_combinator_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            iterator_acquired_local,
            iterator_close,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("done")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read_without_throw_propagation(
            next_result_payload_local,
            next_result_tag_local,
            next_result_payload_local,
            next_result_tag_local,
            key_local,
            done_payload_local,
            done_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(iterator_acquired_local));
        function.instruction(&Instruction::End);
        self.emit_promise_combinator_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            iterator_acquired_local,
            iterator_close,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        self.compile_truthy_tagged_i32(done_tag_local, done_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(iterator_acquired_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("value")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read_without_throw_propagation(
            next_result_payload_local,
            next_result_tag_local,
            next_result_payload_local,
            next_result_tag_local,
            key_local,
            next_value_payload_local,
            next_value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(iterator_acquired_local));
        function.instruction(&Instruction::End);
        self.emit_promise_combinator_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            iterator_acquired_local,
            iterator_close,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;

        self.emit_function_or_proxy_call_leave_throw_completion(
            promise_resolve_payload_local,
            promise_resolve_tag_local,
            constructor_payload_local,
            constructor_tag_local,
            &[(next_value_payload_local, next_value_tag_local)],
            next_promise_payload_local,
            next_promise_tag_local,
            function,
        )?;
        self.emit_promise_combinator_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            iterator_acquired_local,
            iterator_close,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload("then")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read_without_throw_propagation(
            next_promise_payload_local,
            next_promise_tag_local,
            next_promise_payload_local,
            next_promise_tag_local,
            key_local,
            then_payload_local,
            then_tag_local,
            function,
        )?;
        self.emit_promise_combinator_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            iterator_acquired_local,
            iterator_close,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        self.emit_function_or_proxy_call_leave_throw_completion(
            then_payload_local,
            then_tag_local,
            next_promise_payload_local,
            next_promise_tag_local,
            &[
                (resolve_payload_local, resolve_tag_local),
                (reject_payload_local, reject_tag_local),
            ],
            call_payload_local,
            call_tag_local,
            function,
        )?;
        self.emit_promise_combinator_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            iterator_acquired_local,
            iterator_close,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(promise_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(promise_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);

        for local in [
            close_saved_aux_local,
            close_saved_completion_local,
            close_saved_tag_local,
            close_saved_payload_local,
            close_result_tag_local,
            close_result_payload_local,
            close_return_tag_local,
            close_return_payload_local,
            error_tag_local,
            error_payload_local,
            undefined_tag_local,
            undefined_payload_local,
            key_local,
            call_tag_local,
            call_payload_local,
            then_tag_local,
            then_payload_local,
            next_promise_tag_local,
            next_promise_payload_local,
            next_value_tag_local,
            next_value_payload_local,
            done_tag_local,
            done_payload_local,
            next_result_tag_local,
            next_result_payload_local,
            iterator_acquired_local,
            next_tag_local,
            next_payload_local,
            iterator_tag_local,
            iterator_payload_local,
            iterator_method_tag_local,
            iterator_method_payload_local,
            iterable_object_tag_local,
            iterable_object_payload_local,
            iterable_tag_local,
            iterable_payload_local,
            promise_resolve_tag_local,
            promise_resolve_payload_local,
            reject_tag_local,
            reject_payload_local,
            resolve_tag_local,
            resolve_payload_local,
            promise_tag_local,
            promise_payload_local,
            capability_record_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_promise_keyed_reject_current_throw(
        &mut self,
        capability_record_local: u32,
        promise_payload_local: u32,
        promise_tag_local: u32,
        error_payload_local: u32,
        error_tag_local: u32,
        call_payload_local: u32,
        call_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let reject_payload_local = self.reserve_temp_local();
        let reject_tag_local = self.reserve_temp_local();
        let undefined_payload_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalSet(error_payload_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalSet(error_tag_local));
        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_REJECT_PAYLOAD_OFFSET,
            reject_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_REJECT_TAG_OFFSET,
            reject_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_function_or_proxy_call_leave_throw_completion(
            reject_payload_local,
            reject_tag_local,
            undefined_payload_local,
            undefined_tag_local,
            &[(error_payload_local, error_tag_local)],
            call_payload_local,
            call_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(promise_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(promise_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.release_temp_local(undefined_tag_local);
        self.release_temp_local(undefined_payload_local);
        self.release_temp_local(reject_tag_local);
        self.release_temp_local(reject_payload_local);
        Ok(())
    }

    pub(crate) fn emit_promise_all_keyed(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_promise_keyed(PromiseCombinatorMode::Values, function)
    }

    pub(crate) fn emit_promise_all_settled_keyed(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_promise_keyed(PromiseCombinatorMode::SettledRecords, function)
    }

    fn emit_promise_keyed(
        &mut self,
        mode: PromiseCombinatorMode,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let builtin_name = if mode == PromiseCombinatorMode::Values {
            "Promise.allKeyed"
        } else {
            "Promise.allSettledKeyed"
        };
        let constructor_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(format!(
                "unsupported in lila wasm-aot first slice: missing {builtin_name} receiver"
            ))
        })?;
        let constructor_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(format!(
                "unsupported in lila wasm-aot first slice: missing {builtin_name} receiver tag"
            ))
        })?;
        let capability_record_local = self.reserve_temp_local();
        let promise_payload_local = self.reserve_temp_local();
        let promise_tag_local = self.reserve_temp_local();
        let resolve_payload_local = self.reserve_temp_local();
        let resolve_tag_local = self.reserve_temp_local();
        let reject_payload_local = self.reserve_temp_local();
        let reject_tag_local = self.reserve_temp_local();
        let promise_resolve_payload_local = self.reserve_temp_local();
        let promise_resolve_tag_local = self.reserve_temp_local();
        let promises_payload_local = self.reserve_temp_local();
        let promises_tag_local = self.reserve_temp_local();
        let keys_payload_local = self.reserve_temp_local();
        let keys_tag_local = self.reserve_temp_local();
        let keys_length_local = self.reserve_temp_local();
        let key_index_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let key_property_payload_local = self.reserve_temp_local();
        let descriptor_payload_local = self.reserve_temp_local();
        let descriptor_tag_local = self.reserve_temp_local();
        let enumerable_key_local = self.reserve_temp_local();
        let enumerable_payload_local = self.reserve_temp_local();
        let enumerable_tag_local = self.reserve_temp_local();
        let property_value_payload_local = self.reserve_temp_local();
        let property_value_tag_local = self.reserve_temp_local();
        let next_promise_payload_local = self.reserve_temp_local();
        let next_promise_tag_local = self.reserve_temp_local();
        let then_payload_local = self.reserve_temp_local();
        let then_tag_local = self.reserve_temp_local();
        let result_payload_local = self.reserve_temp_local();
        let shared_context_local = self.reserve_temp_local();
        let remaining_local = self.reserve_temp_local();
        let element_context_local = self.reserve_temp_local();
        let resolve_element_payload_local = self.reserve_temp_local();
        let resolve_element_tag_local = self.reserve_temp_local();
        let reject_element_payload_local = self.reserve_temp_local();
        let reject_element_tag_local = self.reserve_temp_local();
        let undefined_payload_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();
        let call_payload_local = self.reserve_temp_local();
        let call_tag_local = self.reserve_temp_local();
        let error_payload_local = self.reserve_temp_local();
        let error_tag_local = self.reserve_temp_local();

        let own_keys_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectOwnKeys.function_id())
            .cloned()
            .ok_or_else(|| EmitError::unsupported("missing Reflect.ownKeys builtin"))?;
        let get_own_descriptor_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectGetOwnPropertyDescriptor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported("missing Reflect.getOwnPropertyDescriptor builtin")
            })?;
        let resolve_element_builtin = if mode == PromiseCombinatorMode::Values {
            StandardBuiltinId::PromiseAllKeyedResolveElement
        } else {
            StandardBuiltinId::PromiseAllSettledKeyedResolveElement
        };
        let resolve_element_meta = self
            .functions
            .get(&resolve_element_builtin.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported("missing keyed Promise resolve element builtin")
            })?;
        let reject_element_meta = if mode == PromiseCombinatorMode::SettledRecords {
            Some(
                self.functions
                    .get(&StandardBuiltinId::PromiseAllSettledKeyedRejectElement.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "missing Promise.allSettledKeyed reject element builtin",
                        )
                    })?,
            )
        } else {
            None
        };

        self.emit_new_promise_capability(
            constructor_payload_local,
            constructor_tag_local,
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            function,
        )?;
        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_RESOLVE_PAYLOAD_OFFSET,
            resolve_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_RESOLVE_TAG_OFFSET,
            resolve_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_REJECT_PAYLOAD_OFFSET,
            reject_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_REJECT_TAG_OFFSET,
            reject_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));

        function.instruction(&Instruction::I64Const(self.strings.payload("resolve")));
        function.instruction(&Instruction::LocalSet(enumerable_key_local));
        self.emit_object_read_without_throw_propagation(
            constructor_payload_local,
            constructor_tag_local,
            constructor_payload_local,
            constructor_tag_local,
            enumerable_key_local,
            promise_resolve_payload_local,
            promise_resolve_tag_local,
            function,
        )?;
        self.emit_promise_keyed_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        self.emit_is_callable_i32(
            promise_resolve_tag_local,
            promise_resolve_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Promise keyed constructor resolve property is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_promise_keyed_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.emit_builtin_arg_to_locals(0, promises_payload_local, promises_tag_local, function);
        self.emit_is_heap_object_like_tag_i32(promises_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Promise keyed input must be an object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_promise_keyed_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.emit_direct_js_call(
            &own_keys_meta,
            None,
            &[(promises_payload_local, promises_tag_local)],
            keys_payload_local,
            keys_tag_local,
            function,
        )?;
        self.emit_promise_keyed_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        self.load_i64_to_local_from_offset(
            keys_payload_local,
            HEAP_LEN_OFFSET,
            keys_length_local,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(None, None, function)?;
        function.instruction(&Instruction::LocalSet(result_payload_local));
        self.emit_heap_alloc_const(HEAP_PROMISE_ALL_SHARED_CONTEXT_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(shared_context_local));
        self.store_i64_const_at_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_REMAINING_OFFSET,
            1,
            function,
        );
        self.store_i64_local_at_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_VALUES_OFFSET,
            result_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_RESOLVE_PAYLOAD_OFFSET,
            resolve_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_RESOLVE_TAG_OFFSET,
            resolve_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(key_index_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("enumerable")));
        function.instruction(&Instruction::LocalSet(enumerable_key_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(key_index_local));
        function.instruction(&Instruction::LocalGet(keys_length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            keys_payload_local,
            key_index_local,
            key_payload_local,
            key_tag_local,
            function,
        );
        // `Reflect.ownKeys` yields String/Symbol *values*; every internal
        // property-key consumer below (the `[[Get]]` on the source object, the
        // `[[DefineOwnProperty]]` on the result, and the key handed to the
        // resolve-element closure) needs the internal encoding, which re-applies
        // `PROPERTY_KEY_SYMBOL_MARKER` for symbols. Without it a symbol key is
        // stored as a bogus string key: `Object.keys` then reports it and reads
        // a garbage payload. The value form stays for
        // `Reflect.getOwnPropertyDescriptor`, which applies ToPropertyKey itself.
        self.emit_property_key_payload_from_value_local(
            key_payload_local,
            key_tag_local,
            key_property_payload_local,
            function,
        );
        function.instruction(&Instruction::I64Const(self.strings.payload("enumerable")));
        function.instruction(&Instruction::LocalSet(enumerable_key_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        self.emit_direct_js_call(
            &get_own_descriptor_meta,
            None,
            &[
                (promises_payload_local, promises_tag_local),
                (key_payload_local, key_tag_local),
            ],
            descriptor_payload_local,
            descriptor_tag_local,
            function,
        )?;
        self.emit_promise_keyed_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(descriptor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(0));
        self.emit_object_read_without_throw_propagation(
            descriptor_payload_local,
            descriptor_tag_local,
            descriptor_payload_local,
            descriptor_tag_local,
            enumerable_key_local,
            enumerable_payload_local,
            enumerable_tag_local,
            function,
        )?;
        self.emit_promise_keyed_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        self.compile_truthy_tagged_i32(enumerable_tag_local, enumerable_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(0));
        self.emit_object_read_without_throw_propagation(
            promises_payload_local,
            promises_tag_local,
            promises_payload_local,
            promises_tag_local,
            key_property_payload_local,
            property_value_payload_local,
            property_value_tag_local,
            function,
        )?;
        self.emit_promise_keyed_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        self.emit_object_define_enumerable_data(
            result_payload_local,
            key_property_payload_local,
            undefined_payload_local,
            undefined_tag_local,
            function,
        )?;
        self.emit_function_or_proxy_call_leave_throw_completion(
            promise_resolve_payload_local,
            promise_resolve_tag_local,
            constructor_payload_local,
            constructor_tag_local,
            &[(property_value_payload_local, property_value_tag_local)],
            next_promise_payload_local,
            next_promise_tag_local,
            function,
        )?;
        self.emit_promise_keyed_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;

        self.emit_heap_alloc_const(HEAP_PROMISE_KEYED_ELEMENT_CONTEXT_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(element_context_local));
        self.store_i64_local_at_offset(
            element_context_local,
            HEAP_PROMISE_KEYED_ELEMENT_KEY_PAYLOAD_OFFSET,
            key_property_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            element_context_local,
            HEAP_PROMISE_KEYED_ELEMENT_KEY_TAG_OFFSET,
            key_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            element_context_local,
            HEAP_PROMISE_KEYED_ELEMENT_SHARED_OFFSET,
            shared_context_local,
            function,
        );
        self.store_i64_const_at_offset(
            element_context_local,
            HEAP_PROMISE_KEYED_ELEMENT_ALREADY_CALLED_OFFSET,
            0,
            function,
        );
        self.emit_function_value_payload(&resolve_element_meta, function)?;
        function.instruction(&Instruction::LocalSet(resolve_element_payload_local));
        self.store_i64_local_at_offset(
            resolve_element_payload_local,
            HEAP_FUNCTION_ENV_HANDLE_OFFSET,
            element_context_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(resolve_element_tag_local));
        if let Some(reject_element_meta) = &reject_element_meta {
            self.emit_function_value_payload(reject_element_meta, function)?;
            function.instruction(&Instruction::LocalSet(reject_element_payload_local));
            self.store_i64_local_at_offset(
                reject_element_payload_local,
                HEAP_FUNCTION_ENV_HANDLE_OFFSET,
                element_context_local,
                function,
            );
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::LocalSet(reject_element_tag_local));
        } else {
            function.instruction(&Instruction::LocalGet(reject_payload_local));
            function.instruction(&Instruction::LocalSet(reject_element_payload_local));
            function.instruction(&Instruction::LocalGet(reject_tag_local));
            function.instruction(&Instruction::LocalSet(reject_element_tag_local));
        }
        self.load_i64_to_local_from_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_REMAINING_OFFSET,
            remaining_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(remaining_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(remaining_local));
        self.store_i64_local_at_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_REMAINING_OFFSET,
            remaining_local,
            function,
        );
        function.instruction(&Instruction::I64Const(self.strings.payload("then")));
        function.instruction(&Instruction::LocalSet(enumerable_key_local));
        self.emit_object_read_without_throw_propagation(
            next_promise_payload_local,
            next_promise_tag_local,
            next_promise_payload_local,
            next_promise_tag_local,
            enumerable_key_local,
            then_payload_local,
            then_tag_local,
            function,
        )?;
        self.emit_promise_keyed_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        self.emit_function_or_proxy_call_leave_throw_completion(
            then_payload_local,
            then_tag_local,
            next_promise_payload_local,
            next_promise_tag_local,
            &[
                (resolve_element_payload_local, resolve_element_tag_local),
                (reject_element_payload_local, reject_element_tag_local),
            ],
            call_payload_local,
            call_tag_local,
            function,
        )?;
        self.emit_promise_keyed_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(key_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(key_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_REMAINING_OFFSET,
            remaining_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(remaining_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(remaining_local));
        self.store_i64_local_at_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_REMAINING_OFFSET,
            remaining_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(remaining_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(property_value_tag_local));
        self.emit_function_or_proxy_call_leave_throw_completion(
            resolve_payload_local,
            resolve_tag_local,
            undefined_payload_local,
            undefined_tag_local,
            &[(result_payload_local, property_value_tag_local)],
            call_payload_local,
            call_tag_local,
            function,
        )?;
        self.emit_promise_keyed_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(promise_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(promise_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);

        for local in [
            error_tag_local,
            error_payload_local,
            call_tag_local,
            call_payload_local,
            undefined_tag_local,
            undefined_payload_local,
            reject_element_tag_local,
            reject_element_payload_local,
            resolve_element_tag_local,
            resolve_element_payload_local,
            element_context_local,
            remaining_local,
            shared_context_local,
            result_payload_local,
            then_tag_local,
            then_payload_local,
            next_promise_tag_local,
            next_promise_payload_local,
            property_value_tag_local,
            property_value_payload_local,
            enumerable_tag_local,
            enumerable_payload_local,
            enumerable_key_local,
            descriptor_tag_local,
            descriptor_payload_local,
            key_property_payload_local,
            key_tag_local,
            key_payload_local,
            key_index_local,
            keys_length_local,
            keys_tag_local,
            keys_payload_local,
            promises_tag_local,
            promises_payload_local,
            promise_resolve_tag_local,
            promise_resolve_payload_local,
            reject_tag_local,
            reject_payload_local,
            resolve_tag_local,
            resolve_payload_local,
            promise_tag_local,
            promise_payload_local,
            capability_record_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn emit_promise_all(&mut self, function: &mut Function) -> Result<(), EmitError> {
        self.emit_promise_combinator(PromiseCombinatorMode::Values, function)
    }

    pub(crate) fn emit_promise_all_settled(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_promise_combinator(PromiseCombinatorMode::SettledRecords, function)
    }

    pub(crate) fn emit_promise_any(&mut self, function: &mut Function) -> Result<(), EmitError> {
        self.emit_promise_combinator(PromiseCombinatorMode::FirstFulfillment, function)
    }

    fn emit_promise_combinator(
        &mut self,
        mode: PromiseCombinatorMode,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let builtin_name = mode.builtin_name();
        let constructor_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(format!(
                "unsupported in lila wasm-aot first slice: missing {builtin_name} receiver"
            ))
        })?;
        let constructor_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(format!(
                "unsupported in lila wasm-aot first slice: missing {builtin_name} receiver tag"
            ))
        })?;
        let capability_record_local = self.reserve_temp_local();
        let promise_payload_local = self.reserve_temp_local();
        let promise_tag_local = self.reserve_temp_local();
        let resolve_payload_local = self.reserve_temp_local();
        let resolve_tag_local = self.reserve_temp_local();
        let promise_resolve_payload_local = self.reserve_temp_local();
        let promise_resolve_tag_local = self.reserve_temp_local();
        let iterable_payload_local = self.reserve_temp_local();
        let iterable_tag_local = self.reserve_temp_local();
        let iterable_object_payload_local = self.reserve_temp_local();
        let iterable_object_tag_local = self.reserve_temp_local();
        let iterator_method_payload_local = self.reserve_temp_local();
        let iterator_method_tag_local = self.reserve_temp_local();
        let iterator_payload_local = self.reserve_temp_local();
        let iterator_tag_local = self.reserve_temp_local();
        let next_payload_local = self.reserve_temp_local();
        let next_tag_local = self.reserve_temp_local();
        let iterator_acquired_local = self.reserve_temp_local();
        let next_result_payload_local = self.reserve_temp_local();
        let next_result_tag_local = self.reserve_temp_local();
        let done_payload_local = self.reserve_temp_local();
        let done_tag_local = self.reserve_temp_local();
        let next_value_payload_local = self.reserve_temp_local();
        let next_value_tag_local = self.reserve_temp_local();
        let next_promise_payload_local = self.reserve_temp_local();
        let next_promise_tag_local = self.reserve_temp_local();
        let then_payload_local = self.reserve_temp_local();
        let then_tag_local = self.reserve_temp_local();
        let reject_payload_local = self.reserve_temp_local();
        let reject_tag_local = self.reserve_temp_local();
        let call_payload_local = self.reserve_temp_local();
        let call_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let values_payload_local = self.reserve_temp_local();
        let shared_context_local = self.reserve_temp_local();
        let remaining_local = self.reserve_temp_local();
        let element_context_local = self.reserve_temp_local();
        let resolve_element_payload_local = self.reserve_temp_local();
        let resolve_element_tag_local = self.reserve_temp_local();
        let reject_element_payload_local = self.reserve_temp_local();
        let reject_element_tag_local = self.reserve_temp_local();
        let undefined_payload_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();
        let error_payload_local = self.reserve_temp_local();
        let error_tag_local = self.reserve_temp_local();
        let close_return_payload_local = self.reserve_temp_local();
        let close_return_tag_local = self.reserve_temp_local();
        let close_result_payload_local = self.reserve_temp_local();
        let close_result_tag_local = self.reserve_temp_local();
        let close_saved_payload_local = self.reserve_temp_local();
        let close_saved_tag_local = self.reserve_temp_local();
        let close_saved_completion_local = self.reserve_temp_local();
        let close_saved_aux_local = self.reserve_temp_local();

        let iterator_close = IteratorCloseOnThrowLocals {
            iterator_payload_local,
            iterator_tag_local,
            key_local,
            return_payload_local: close_return_payload_local,
            return_tag_local: close_return_tag_local,
            result_payload_local: close_result_payload_local,
            result_tag_local: close_result_tag_local,
            saved_payload_local: close_saved_payload_local,
            saved_tag_local: close_saved_tag_local,
            saved_completion_local: close_saved_completion_local,
            saved_aux_local: close_saved_aux_local,
        };
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(iterator_acquired_local));
        self.emit_new_promise_capability(
            constructor_payload_local,
            constructor_tag_local,
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            function,
        )?;
        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_RESOLVE_PAYLOAD_OFFSET,
            resolve_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_RESOLVE_TAG_OFFSET,
            resolve_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_REJECT_PAYLOAD_OFFSET,
            reject_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_REJECT_TAG_OFFSET,
            reject_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));

        function.instruction(&Instruction::I64Const(self.strings.payload("resolve")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read_without_throw_propagation(
            constructor_payload_local,
            constructor_tag_local,
            constructor_payload_local,
            constructor_tag_local,
            key_local,
            promise_resolve_payload_local,
            promise_resolve_tag_local,
            function,
        )?;
        self.emit_promise_combinator_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            iterator_acquired_local,
            iterator_close,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        self.emit_is_callable_i32(
            promise_resolve_tag_local,
            promise_resolve_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Promise.all constructor resolve property is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_promise_combinator_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            iterator_acquired_local,
            iterator_close,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.emit_builtin_arg_to_locals(0, iterable_payload_local, iterable_tag_local, function);
        function.instruction(&Instruction::LocalGet(iterable_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(iterable_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Promise.all input is not iterable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_promise_combinator_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            iterator_acquired_local,
            iterator_close,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_value_to_current_function_realm_object_locals(
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
        self.emit_object_read_without_throw_propagation(
            iterable_object_payload_local,
            iterable_object_tag_local,
            iterable_payload_local,
            iterable_tag_local,
            key_local,
            iterator_method_payload_local,
            iterator_method_tag_local,
            function,
        )?;
        self.emit_promise_combinator_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            iterator_acquired_local,
            iterator_close,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        self.emit_is_callable_i32(
            iterator_method_tag_local,
            iterator_method_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Promise.all iterator method is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_promise_combinator_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            iterator_acquired_local,
            iterator_close,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_function_or_proxy_call_leave_throw_completion(
            iterator_method_payload_local,
            iterator_method_tag_local,
            iterable_payload_local,
            iterable_tag_local,
            &[],
            iterator_payload_local,
            iterator_tag_local,
            function,
        )?;
        self.emit_promise_combinator_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            iterator_acquired_local,
            iterator_close,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        self.emit_is_heap_object_like_tag_i32(iterator_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Promise.all iterator method must return an object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_promise_combinator_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            iterator_acquired_local,
            iterator_close,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(self.strings.payload("next")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read_without_throw_propagation(
            iterator_payload_local,
            iterator_tag_local,
            iterator_payload_local,
            iterator_tag_local,
            key_local,
            next_payload_local,
            next_tag_local,
            function,
        )?;
        self.emit_promise_combinator_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            iterator_acquired_local,
            iterator_close,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        self.emit_is_callable_i32(next_tag_local, next_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Promise.all iterator next method is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_promise_combinator_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            iterator_acquired_local,
            iterator_close,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(iterator_acquired_local));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        self.emit_alloc_array_payload_with_length(index_local, values_payload_local, function)?;
        self.emit_heap_alloc_const(HEAP_PROMISE_ALL_SHARED_CONTEXT_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(shared_context_local));
        self.store_i64_const_at_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_REMAINING_OFFSET,
            1,
            function,
        );
        self.store_i64_local_at_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_VALUES_OFFSET,
            values_payload_local,
            function,
        );
        let (settlement_payload_local, settlement_tag_local) =
            if mode == PromiseCombinatorMode::FirstFulfillment {
                (reject_payload_local, reject_tag_local)
            } else {
                (resolve_payload_local, resolve_tag_local)
            };
        self.store_i64_local_at_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_RESOLVE_PAYLOAD_OFFSET,
            settlement_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_RESOLVE_TAG_OFFSET,
            settlement_tag_local,
            function,
        );

        let resolve_element_builtin = match mode {
            PromiseCombinatorMode::Values => Some(StandardBuiltinId::PromiseAllResolveElement),
            PromiseCombinatorMode::SettledRecords => {
                Some(StandardBuiltinId::PromiseAllSettledResolveElement)
            }
            PromiseCombinatorMode::FirstFulfillment => None,
        };
        let resolve_element_meta = resolve_element_builtin
            .map(|builtin| {
                self.functions
                    .get(&builtin.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "missing {builtin_name} resolve element builtin"
                        ))
                    })
            })
            .transpose()?;
        let reject_element_builtin = match mode {
            PromiseCombinatorMode::SettledRecords => {
                Some(StandardBuiltinId::PromiseAllSettledRejectElement)
            }
            PromiseCombinatorMode::FirstFulfillment => {
                Some(StandardBuiltinId::PromiseAnyRejectElement)
            }
            PromiseCombinatorMode::Values => None,
        };
        let reject_element_meta = reject_element_builtin
            .map(|builtin| {
                self.functions
                    .get(&builtin.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "missing {builtin_name} reject element builtin"
                        ))
                    })
            })
            .transpose()?;
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.emit_function_or_proxy_call_leave_throw_completion(
            next_payload_local,
            next_tag_local,
            iterator_payload_local,
            iterator_tag_local,
            &[],
            next_result_payload_local,
            next_result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(iterator_acquired_local));
        function.instruction(&Instruction::End);
        self.emit_promise_combinator_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            iterator_acquired_local,
            iterator_close,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        self.emit_is_heap_object_like_tag_i32(next_result_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(iterator_acquired_local));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Promise.all iterator next result must be an object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_promise_combinator_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            iterator_acquired_local,
            iterator_close,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("done")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read_without_throw_propagation(
            next_result_payload_local,
            next_result_tag_local,
            next_result_payload_local,
            next_result_tag_local,
            key_local,
            done_payload_local,
            done_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(iterator_acquired_local));
        function.instruction(&Instruction::End);
        self.emit_promise_combinator_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            iterator_acquired_local,
            iterator_close,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        self.compile_truthy_tagged_i32(done_tag_local, done_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(iterator_acquired_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("value")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read_without_throw_propagation(
            next_result_payload_local,
            next_result_tag_local,
            next_result_payload_local,
            next_result_tag_local,
            key_local,
            next_value_payload_local,
            next_value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(iterator_acquired_local));
        function.instruction(&Instruction::End);
        self.emit_promise_combinator_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            iterator_acquired_local,
            iterator_close,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(9_007_199_254_740_991));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            RANGE_ERROR_NAME,
            "Promise.all iterable contains too many values",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_promise_combinator_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            iterator_acquired_local,
            iterator_close,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.emit_array_write(
            values_payload_local,
            index_local,
            undefined_payload_local,
            undefined_tag_local,
            function,
        )?;
        self.emit_function_or_proxy_call_leave_throw_completion(
            promise_resolve_payload_local,
            promise_resolve_tag_local,
            constructor_payload_local,
            constructor_tag_local,
            &[(next_value_payload_local, next_value_tag_local)],
            next_promise_payload_local,
            next_promise_tag_local,
            function,
        )?;
        self.emit_promise_combinator_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            iterator_acquired_local,
            iterator_close,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;

        self.emit_heap_alloc_const(HEAP_PROMISE_ALL_ELEMENT_CONTEXT_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(element_context_local));
        self.store_i64_local_at_offset(
            element_context_local,
            HEAP_PROMISE_ALL_ELEMENT_INDEX_OFFSET,
            index_local,
            function,
        );
        self.store_i64_local_at_offset(
            element_context_local,
            HEAP_PROMISE_ALL_ELEMENT_SHARED_OFFSET,
            shared_context_local,
            function,
        );
        self.store_i64_const_at_offset(
            element_context_local,
            HEAP_PROMISE_ALL_ELEMENT_ALREADY_CALLED_OFFSET,
            0,
            function,
        );
        if let Some(resolve_element_meta) = &resolve_element_meta {
            self.emit_function_value_payload(resolve_element_meta, function)?;
            function.instruction(&Instruction::LocalSet(resolve_element_payload_local));
            self.store_i64_local_at_offset(
                resolve_element_payload_local,
                HEAP_FUNCTION_ENV_HANDLE_OFFSET,
                element_context_local,
                function,
            );
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::LocalSet(resolve_element_tag_local));
        }
        if let Some(reject_element_meta) = &reject_element_meta {
            self.emit_function_value_payload(reject_element_meta, function)?;
            function.instruction(&Instruction::LocalSet(reject_element_payload_local));
            self.store_i64_local_at_offset(
                reject_element_payload_local,
                HEAP_FUNCTION_ENV_HANDLE_OFFSET,
                element_context_local,
                function,
            );
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::LocalSet(reject_element_tag_local));
        }
        self.load_i64_to_local_from_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_REMAINING_OFFSET,
            remaining_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(remaining_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(remaining_local));
        self.store_i64_local_at_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_REMAINING_OFFSET,
            remaining_local,
            function,
        );

        function.instruction(&Instruction::I64Const(self.strings.payload("then")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read_without_throw_propagation(
            next_promise_payload_local,
            next_promise_tag_local,
            next_promise_payload_local,
            next_promise_tag_local,
            key_local,
            then_payload_local,
            then_tag_local,
            function,
        )?;
        self.emit_promise_combinator_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            iterator_acquired_local,
            iterator_close,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        let (on_fulfilled_payload_local, on_fulfilled_tag_local) =
            if mode == PromiseCombinatorMode::FirstFulfillment {
                (resolve_payload_local, resolve_tag_local)
            } else {
                (resolve_element_payload_local, resolve_element_tag_local)
            };
        let (on_rejected_payload_local, on_rejected_tag_local) = match mode {
            PromiseCombinatorMode::Values => (reject_payload_local, reject_tag_local),
            PromiseCombinatorMode::SettledRecords | PromiseCombinatorMode::FirstFulfillment => {
                (reject_element_payload_local, reject_element_tag_local)
            }
        };
        self.emit_function_or_proxy_call_leave_throw_completion(
            then_payload_local,
            then_tag_local,
            next_promise_payload_local,
            next_promise_tag_local,
            &[
                (on_fulfilled_payload_local, on_fulfilled_tag_local),
                (on_rejected_payload_local, on_rejected_tag_local),
            ],
            call_payload_local,
            call_tag_local,
            function,
        )?;
        self.emit_promise_combinator_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            iterator_acquired_local,
            iterator_close,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_REMAINING_OFFSET,
            remaining_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(remaining_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(remaining_local));
        self.store_i64_local_at_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_REMAINING_OFFSET,
            remaining_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(remaining_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        if mode == PromiseCombinatorMode::FirstFulfillment {
            function.instruction(&Instruction::GlobalGet(
                AGGREGATE_ERROR_PROTOTYPE_GLOBAL_INDEX,
            ));
            function.instruction(&Instruction::LocalSet(element_context_local));
            self.emit_alloc_aggregate_error_instance_from_locals(
                None,
                values_payload_local,
                element_context_local,
                next_value_payload_local,
                next_value_tag_local,
                function,
            )?;
            self.emit_function_or_proxy_call_leave_throw_completion(
                reject_payload_local,
                reject_tag_local,
                undefined_payload_local,
                undefined_tag_local,
                &[(next_value_payload_local, next_value_tag_local)],
                call_payload_local,
                call_tag_local,
                function,
            )?;
        } else {
            function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
            function.instruction(&Instruction::LocalSet(next_value_tag_local));
            self.emit_function_or_proxy_call_leave_throw_completion(
                resolve_payload_local,
                resolve_tag_local,
                undefined_payload_local,
                undefined_tag_local,
                &[(values_payload_local, next_value_tag_local)],
                call_payload_local,
                call_tag_local,
                function,
            )?;
        }
        self.emit_promise_combinator_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            iterator_acquired_local,
            iterator_close,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(promise_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(promise_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);

        for local in [
            close_saved_aux_local,
            close_saved_completion_local,
            close_saved_tag_local,
            close_saved_payload_local,
            close_result_tag_local,
            close_result_payload_local,
            close_return_tag_local,
            close_return_payload_local,
            error_tag_local,
            error_payload_local,
            undefined_tag_local,
            undefined_payload_local,
            reject_element_tag_local,
            reject_element_payload_local,
            resolve_element_tag_local,
            resolve_element_payload_local,
            element_context_local,
            remaining_local,
            shared_context_local,
            values_payload_local,
            index_local,
            key_local,
            call_tag_local,
            call_payload_local,
            reject_tag_local,
            reject_payload_local,
            then_tag_local,
            then_payload_local,
            next_promise_tag_local,
            next_promise_payload_local,
            next_value_tag_local,
            next_value_payload_local,
            done_tag_local,
            done_payload_local,
            next_result_tag_local,
            next_result_payload_local,
            iterator_acquired_local,
            next_tag_local,
            next_payload_local,
            iterator_tag_local,
            iterator_payload_local,
            iterator_method_tag_local,
            iterator_method_payload_local,
            iterable_object_tag_local,
            iterable_object_payload_local,
            iterable_tag_local,
            iterable_payload_local,
            promise_resolve_tag_local,
            promise_resolve_payload_local,
            resolve_tag_local,
            resolve_payload_local,
            promise_tag_local,
            promise_payload_local,
            capability_record_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn emit_promise_resolving_function(
        &mut self,
        state: u64,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let resolving_context_local = self.reserve_temp_local();
        let promise_record_local = self.reserve_temp_local();
        let promise_payload_local = self.reserve_temp_local();
        let already_resolved_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(0));
        function.instruction(&Instruction::LocalSet(resolving_context_local));
        self.load_i64_to_local_from_offset(
            resolving_context_local,
            HEAP_PROMISE_RESOLVING_CONTEXT_ALREADY_RESOLVED_OFFSET,
            already_resolved_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(already_resolved_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_const_at_offset(
            resolving_context_local,
            HEAP_PROMISE_RESOLVING_CONTEXT_ALREADY_RESOLVED_OFFSET,
            1,
            function,
        );
        self.load_i64_to_local_from_offset(
            resolving_context_local,
            HEAP_PROMISE_RESOLVING_CONTEXT_RECORD_OFFSET,
            promise_record_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            resolving_context_local,
            HEAP_PROMISE_RESOLVING_CONTEXT_PAYLOAD_OFFSET,
            promise_payload_local,
            function,
        );
        self.emit_builtin_arg_to_locals(0, value_payload_local, value_tag_local, function);
        if state == PROMISE_STATE_FULFILLED {
            self.emit_resolve_promise_record(
                promise_payload_local,
                promise_record_local,
                value_payload_local,
                value_tag_local,
                function,
            )?;
        } else {
            self.emit_settle_promise_record(
                promise_record_local,
                state,
                value_payload_local,
                value_tag_local,
                function,
            )?;
        }
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);

        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(already_resolved_local);
        self.release_temp_local(promise_payload_local);
        self.release_temp_local(promise_record_local);
        self.release_temp_local(resolving_context_local);
        Ok(())
    }
}
