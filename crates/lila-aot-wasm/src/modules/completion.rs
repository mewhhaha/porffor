//! Intrinsic module Promise completion, async parents and deferred-import joins.

use super::runtime::ModuleRuntimeOperation;
use super::*;
use crate::builtins::ModuleReactionContinuation;

impl FunctionBuilder<'_> {
    pub(super) fn emit_module_cache_throw(
        &self,
        record: u32,
        payload: u32,
        tag: u32,
        function: &mut Function,
    ) {
        self.store_i64_local_at_offset(record, MODULE_ERROR_PAYLOAD_OFFSET, payload, function);
        self.store_i64_local_at_offset(record, MODULE_ERROR_TAG_OFFSET, tag, function);
        self.store_i64_const_at_offset(
            record,
            MODULE_COMPLETION_OFFSET,
            ModuleEvaluationCompletion::Throw.word(),
            function,
        );
        self.store_i64_const_at_offset(
            record,
            MODULE_STATE_OFFSET,
            ModuleEvaluationState::Evaluated.word(),
            function,
        );
        self.store_i64_const_at_offset(record, MODULE_ASYNC_ORDER_OFFSET, 0, function);
    }

    pub(super) fn emit_module_settle_evaluation_if_terminal(
        &mut self,
        module: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let promise = self.reserve_temp_local();
        let record = self.reserve_temp_local();
        let state = self.reserve_temp_local();
        let payload = self.reserve_temp_local();
        let tag = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            module,
            MODULE_EVALUATION_PROMISE_OFFSET,
            promise,
            function,
        );
        function.instruction(&Instruction::LocalGet(promise));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_load_module_state_strict(module, state, function);
        function.instruction(&Instruction::LocalGet(state));
        function.instruction(&Instruction::I64Const(
            ModuleEvaluationState::Evaluated.word() as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            promise,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            record,
            function,
        );
        self.emit_load_module_completion_strict(module, state, function);
        function.instruction(&Instruction::LocalGet(state));
        function.instruction(&Instruction::I64Const(
            ModuleEvaluationCompletion::Throw.word() as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(module, MODULE_ERROR_PAYLOAD_OFFSET, payload, function);
        self.load_i64_to_local_from_offset(module, MODULE_ERROR_TAG_OFFSET, tag, function);
        self.emit_settle_promise_record(record, PromiseSettlement::Reject, payload, tag, function)?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(state));
        function.instruction(&Instruction::I64Const(
            ModuleEvaluationCompletion::Normal.word() as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag));
        self.emit_settle_promise_record(
            record,
            PromiseSettlement::Fulfill,
            payload,
            tag,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for local in [tag, payload, state, record, promise] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn emit_run_module_body_reaction(
        &mut self,
        reaction: u32,
        rejected: u32,
        payload: u32,
        tag: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if self.functions.module_execution_record_count() == 0 {
            function.instruction(&Instruction::Unreachable);
            return Ok(());
        }
        let module = self.reserve_temp_local();
        let state = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            reaction,
            HEAP_PROMISE_REACTION_CAPABILITY_OFFSET,
            module,
            function,
        );
        self.emit_load_module_body_state_strict(module, state, function);
        function.instruction(&Instruction::LocalGet(state));
        function.instruction(&Instruction::I64Const(
            ModuleBodyState::Executing.word() as i64
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.store_i64_const_at_offset(
            module,
            MODULE_BODY_STATE_OFFSET,
            ModuleBodyState::Completed.word(),
            function,
        );
        function.instruction(&Instruction::LocalGet(rejected));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_module_runtime_call(
            ModuleRuntimeOperation::Rejected,
            &[module, payload, tag],
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_module_runtime_call(
            ModuleRuntimeOperation::Fulfilled,
            &[module],
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.set_completion_kind(CompletionKind::Normal, function);
        self.release_temp_local(state);
        self.release_temp_local(module);
        Ok(())
    }

    pub(super) fn emit_module_rejected_runtime(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let graph = self.reserve_temp_local();
        let limit = self.reserve_temp_local();
        let queue = self.reserve_temp_local();
        let count = self.reserve_temp_local();
        let index = self.reserve_temp_local();
        let address = self.reserve_temp_local();
        let module = self.reserve_temp_local();
        let occurrence = self.reserve_temp_local();
        let parent = self.reserve_temp_local();
        let state = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(0, MODULE_GRAPH_OFFSET, graph, function);
        self.load_i64_to_local_from_offset(graph, MODULE_GRAPH_COUNT_OFFSET, limit, function);
        self.emit_module_allocate_words(limit, 8, queue, function)?;
        for local in [count, index] {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(local));
        }
        self.emit_load_module_completion_strict(0, state, function);
        function.instruction(&Instruction::LocalGet(state));
        function.instruction(&Instruction::I64Const(
            ModuleEvaluationCompletion::Throw.word() as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_module_state_strict(0, state, function);
        function.instruction(&Instruction::LocalGet(state));
        function.instruction(&Instruction::I64Const(
            ModuleEvaluationState::EvaluatingAsync.word() as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.emit_module_cache_throw(0, 1, 2, function);
        self.emit_module_bounded_append(queue, count, limit, 0, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(count));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_module_array_address(queue, index, 8, address, function);
        self.load_i64_to_local_from_offset(address, 0, module, function);
        self.emit_module_settle_evaluation_if_terminal(module, function)?;
        self.load_i64_to_local_from_offset(
            module,
            MODULE_ASYNC_PARENTS_HEAD_OFFSET,
            occurrence,
            function,
        );
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(occurrence));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        self.load_i64_to_local_from_offset(
            occurrence,
            MODULE_PARENT_MODULE_OFFSET,
            parent,
            function,
        );
        self.emit_load_module_completion_strict(parent, state, function);
        function.instruction(&Instruction::LocalGet(state));
        function.instruction(&Instruction::I64Const(
            ModuleEvaluationCompletion::Throw.word() as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_module_state_strict(parent, state, function);
        function.instruction(&Instruction::LocalGet(state));
        function.instruction(&Instruction::I64Const(
            ModuleEvaluationState::EvaluatingAsync.word() as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.emit_module_cache_throw(parent, 1, 2, function);
        self.emit_module_bounded_append(queue, count, limit, parent, function);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            occurrence,
            MODULE_PARENT_NEXT_OFFSET,
            occurrence,
            function,
        );
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_module_increment(index, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_statement_result(function, ValueKind::Undefined);
        for local in [
            state, parent, occurrence, module, address, index, count, queue, limit, graph,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn emit_run_module_join_reaction(
        &mut self,
        reaction: u32,
        rejected: u32,
        payload: u32,
        tag: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if self.functions.module_execution_record_count() == 0 {
            function.instruction(&Instruction::Unreachable);
            return Ok(());
        }
        let join = self.reserve_temp_local();
        let remaining = self.reserve_temp_local();
        let record = self.reserve_temp_local();
        let undefined = self.reserve_temp_local();
        let undefined_tag = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            reaction,
            HEAP_PROMISE_REACTION_CAPABILITY_OFFSET,
            join,
            function,
        );
        self.load_i64_to_local_from_offset(join, MODULE_JOIN_REMAINING_OFFSET, remaining, function);
        function.instruction(&Instruction::LocalGet(remaining));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.emit_module_increment(remaining, -1, function);
        self.store_i64_local_at_offset(join, MODULE_JOIN_REMAINING_OFFSET, remaining, function);
        self.load_i64_to_local_from_offset(
            join,
            MODULE_JOIN_PROMISE_RECORD_OFFSET,
            record,
            function,
        );
        function.instruction(&Instruction::LocalGet(rejected));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_settle_promise_record(record, PromiseSettlement::Reject, payload, tag, function)?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(remaining));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag));
        self.emit_settle_promise_record(
            record,
            PromiseSettlement::Fulfill,
            undefined,
            undefined_tag,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.set_completion_kind(CompletionKind::Normal, function);
        for local in [undefined_tag, undefined, record, remaining, join] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(super) fn emit_module_deferred_import_runtime(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let dependencies = self.reserve_temp_local();
        let count = self.reserve_temp_local();
        let promises = self.reserve_temp_local();
        let index = self.reserve_temp_local();
        let address = self.reserve_temp_local();
        let module = self.reserve_temp_local();
        let join = self.reserve_temp_local();
        let promise = self.reserve_temp_local();
        let record = self.reserve_temp_local();
        let selected_promise = self.reserve_temp_local();
        let selected_record = self.reserve_temp_local();
        let selected_tag = self.reserve_temp_local();
        self.emit_module_runtime_call(
            ModuleRuntimeOperation::Gather,
            &[0],
            dependencies,
            count,
            function,
        )?;
        self.emit_module_allocate_words(count, 8, promises, function)?;
        let realm = self.emit_module_execution_realm_context(0, function);
        let context = self.emit_async_execution_promise_allocation_context(&realm, function);
        self.emit_alloc_promise_with_prototype(context, promise, record, function)?;
        self.emit_heap_alloc_const(MODULE_JOIN_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(join));
        self.store_i64_local_at_offset(join, MODULE_JOIN_PROMISE_OFFSET, promise, function);
        self.store_i64_local_at_offset(join, MODULE_JOIN_PROMISE_RECORD_OFFSET, record, function);
        self.store_i64_local_at_offset(join, MODULE_JOIN_REMAINING_OFFSET, count, function);
        self.load_i64_to_local_from_offset(0, MODULE_REALM_OFFSET, selected_record, function);
        self.store_i64_local_at_offset(join, MODULE_JOIN_REALM_OFFSET, selected_record, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(count));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_module_array_address(dependencies, index, 8, address, function);
        self.load_i64_to_local_from_offset(address, 0, module, function);
        self.emit_module_runtime_call(
            ModuleRuntimeOperation::Evaluate,
            &[module],
            selected_promise,
            selected_tag,
            function,
        )?;
        self.emit_module_array_address(promises, index, 8, address, function);
        self.store_i64_local_at_offset(address, 0, selected_promise, function);
        self.emit_module_increment(index, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        // Evaluate every selected target before registering any join reaction.
        // Already-settled promises enqueue jobs, so interleaving these loops
        // would change the order relative to later targets' first source Await.
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(count));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_module_array_address(promises, index, 8, address, function);
        self.load_i64_to_local_from_offset(address, 0, selected_promise, function);
        self.load_i64_to_local_from_offset(
            selected_promise,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            selected_record,
            function,
        );
        self.emit_module_promise_reactions(
            join,
            selected_record,
            &realm,
            ModuleReactionContinuation::Join,
            function,
        )?;
        self.emit_module_increment(index, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_async_execution_realm_context(realm);
        function.instruction(&Instruction::LocalGet(count));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_statement_result(function, ValueKind::Undefined);
        self.emit_settle_promise_record(
            record,
            PromiseSettlement::Fulfill,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(promise));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);
        for local in [
            selected_tag,
            selected_record,
            selected_promise,
            record,
            promise,
            join,
            module,
            address,
            index,
            promises,
            count,
            dependencies,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }
    pub(super) fn emit_module_fulfilled_runtime(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let graph = self.reserve_temp_local();
        let limit = self.reserve_temp_local();
        let queue = self.reserve_temp_local();
        let queue_count = self.reserve_temp_local();
        let queue_index = self.reserve_temp_local();
        let executions = self.reserve_temp_local();
        let execution_count = self.reserve_temp_local();
        let execution_index = self.reserve_temp_local();
        let address = self.reserve_temp_local();
        let module = self.reserve_temp_local();
        let occurrence = self.reserve_temp_local();
        let parent = self.reserve_temp_local();
        let root = self.reserve_temp_local();
        let state = self.reserve_temp_local();
        let found = self.reserve_temp_local();
        let pending = self.reserve_temp_local();
        let order = self.reserve_temp_local();
        let smallest_order = self.reserve_temp_local();
        let selected = self.reserve_temp_local();
        let selected_address = self.reserve_temp_local();
        let kind = self.reserve_temp_local();
        self.emit_load_module_completion_strict(0, state, function);
        function.instruction(&Instruction::LocalGet(state));
        function.instruction(&Instruction::I64Const(
            ModuleEvaluationCompletion::Throw.word() as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_load_module_state_strict(0, state, function);
        function.instruction(&Instruction::LocalGet(state));
        function.instruction(&Instruction::I64Const(
            ModuleEvaluationState::EvaluatingAsync.word() as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.store_i64_const_at_offset(
            0,
            MODULE_STATE_OFFSET,
            ModuleEvaluationState::Evaluated.word(),
            function,
        );
        self.store_i64_const_at_offset(
            0,
            MODULE_COMPLETION_OFFSET,
            ModuleEvaluationCompletion::Normal.word(),
            function,
        );
        self.store_i64_const_at_offset(0, MODULE_ASYNC_ORDER_OFFSET, 0, function);
        self.emit_module_settle_evaluation_if_terminal(0, function)?;
        self.load_i64_to_local_from_offset(0, MODULE_GRAPH_OFFSET, graph, function);
        self.load_i64_to_local_from_offset(graph, MODULE_GRAPH_COUNT_OFFSET, limit, function);
        self.emit_module_allocate_words(limit, 8, queue, function)?;
        self.emit_module_allocate_words(limit, 8, executions, function)?;
        for local in [queue_count, queue_index, execution_count] {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(local));
        }
        self.emit_module_bounded_append(queue, queue_count, limit, 0, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(queue_index));
        function.instruction(&Instruction::LocalGet(queue_count));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_module_array_address(queue, queue_index, 8, address, function);
        self.load_i64_to_local_from_offset(address, 0, module, function);
        self.load_i64_to_local_from_offset(
            module,
            MODULE_ASYNC_PARENTS_HEAD_OFFSET,
            occurrence,
            function,
        );
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(occurrence));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        self.load_i64_to_local_from_offset(
            occurrence,
            MODULE_PARENT_MODULE_OFFSET,
            parent,
            function,
        );
        self.emit_module_list_contains(executions, execution_count, parent, found, function);
        self.emit_load_module_completion_strict(parent, state, function);
        function.instruction(&Instruction::LocalGet(state));
        function.instruction(&Instruction::I64Const(
            ModuleEvaluationCompletion::Throw.word() as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        // A later sibling can abort DFS before this parent's SCC closes.
        // Its cached failure is final even though CycleRoot is still empty.
        self.load_i64_to_local_from_offset(parent, MODULE_CYCLE_ROOT_OFFSET, root, function);
        function.instruction(&Instruction::LocalGet(root));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.emit_load_module_completion_strict(root, state, function);
        function.instruction(&Instruction::LocalGet(state));
        function.instruction(&Instruction::I64Const(
            ModuleEvaluationCompletion::Throw.word() as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(found));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_module_state_strict(parent, state, function);
        function.instruction(&Instruction::LocalGet(state));
        function.instruction(&Instruction::I64Const(
            ModuleEvaluationState::EvaluatingAsync.word() as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            parent,
            MODULE_PENDING_ASYNC_DEPENDENCIES_OFFSET,
            pending,
            function,
        );
        function.instruction(&Instruction::LocalGet(pending));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.emit_module_increment(pending, -1, function);
        self.store_i64_local_at_offset(
            parent,
            MODULE_PENDING_ASYNC_DEPENDENCIES_OFFSET,
            pending,
            function,
        );
        function.instruction(&Instruction::LocalGet(pending));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_module_bounded_append(executions, execution_count, limit, parent, function);
        self.emit_load_module_activation_kind_strict(parent, kind, function);
        function.instruction(&Instruction::LocalGet(kind));
        function.instruction(&Instruction::I64Const(
            ModuleActivationKind::Synchronous.word() as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_module_bounded_append(queue, queue_count, limit, parent, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            occurrence,
            MODULE_PARENT_NEXT_OFFSET,
            occurrence,
            function,
        );
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_module_increment(queue_index, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        // Gather every eligible synchronous ancestor before executing source,
        // then choose the exact global async-evaluation order. A same-cycle
        // peer is not a barrier unless a registered dependency requires it.
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        for local in [selected, execution_index] {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(local));
        }
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(smallest_order));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(execution_index));
        function.instruction(&Instruction::LocalGet(execution_count));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_module_array_address(executions, execution_index, 8, address, function);
        self.load_i64_to_local_from_offset(address, 0, parent, function);
        function.instruction(&Instruction::LocalGet(parent));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(parent, MODULE_ASYNC_ORDER_OFFSET, order, function);
        function.instruction(&Instruction::LocalGet(order));
        function.instruction(&Instruction::LocalGet(smallest_order));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        for (source, target) in [
            (parent, selected),
            (order, smallest_order),
            (address, selected_address),
        ] {
            function.instruction(&Instruction::LocalGet(source));
            function.instruction(&Instruction::LocalSet(target));
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_module_increment(execution_index, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(selected));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        self.store_i64_const_at_offset(selected_address, 0, 0, function);
        self.emit_load_module_completion_strict(selected, state, function);
        function.instruction(&Instruction::LocalGet(state));
        function.instruction(&Instruction::I64Const(
            ModuleEvaluationCompletion::Throw.word() as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_module_activation_kind_strict(selected, kind, function);
        self.emit_module_runtime_call(
            ModuleRuntimeOperation::Execute,
            &[selected],
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(kind));
        function.instruction(&Instruction::I64Const(
            ModuleActivationKind::Synchronous.word() as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_module_runtime_call(
            ModuleRuntimeOperation::Rejected,
            &[selected, self.result_local, self.result_tag_local],
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.store_i64_const_at_offset(
            selected,
            MODULE_STATE_OFFSET,
            ModuleEvaluationState::Evaluated.word(),
            function,
        );
        self.store_i64_const_at_offset(
            selected,
            MODULE_COMPLETION_OFFSET,
            ModuleEvaluationCompletion::Normal.word(),
            function,
        );
        self.store_i64_const_at_offset(selected, MODULE_ASYNC_ORDER_OFFSET, 0, function);
        self.emit_module_settle_evaluation_if_terminal(selected, function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_statement_result(function, ValueKind::Undefined);
        for local in [
            kind,
            selected_address,
            selected,
            smallest_order,
            order,
            pending,
            found,
            state,
            root,
            parent,
            occurrence,
            module,
            address,
            execution_index,
            execution_count,
            executions,
            queue_index,
            queue_count,
            queue,
            limit,
            graph,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
