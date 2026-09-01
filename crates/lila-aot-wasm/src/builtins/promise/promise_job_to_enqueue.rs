use super::*;

/// A fully selected pending-job payload. The only queue append function
/// accepts this type, so every job shape must provide its argument and realm
/// policy before it can enter the shared FIFO.
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

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_enqueue_promise_reaction_job(
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

    pub(super) fn emit_enqueue_promise_thenable_job(
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
}
