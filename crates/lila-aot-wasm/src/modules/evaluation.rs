//! Phase-aware runtime DFS and private body entry. No static SCC schedule is used.

use super::runtime::ModuleRuntimeOperation;
use super::*;
use crate::builtins::ModuleReactionContinuation;
use crate::objects::TaggedLocals;

const FRAME_RECORD: u64 = 0;
const FRAME_DEPENDENCIES: u64 = 8;
const FRAME_COUNT: u64 = 16;
const FRAME_NEXT: u64 = 24;
const FRAME_RETURNED_CHILD: u64 = 32;
const FRAME_SIZE: u64 = 40;

impl FunctionBuilder<'_> {
    pub(super) fn emit_module_execute_runtime(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let state = self.reserve_temp_local();
        let kind = self.reserve_temp_local();
        let activation = self.reserve_temp_local();
        let promise = self.reserve_temp_local();
        let promise_record = self.reserve_temp_local();
        self.emit_load_module_body_state_strict(0, state, function);
        function.instruction(&Instruction::LocalGet(state));
        function.instruction(&Instruction::I64Const(
            ModuleBodyState::NotStarted.word() as i64
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.store_i64_const_at_offset(
            0,
            MODULE_BODY_STATE_OFFSET,
            ModuleBodyState::Executing.word(),
            function,
        );
        self.emit_load_module_activation_kind_strict(0, kind, function);
        function.instruction(&Instruction::LocalGet(kind));
        function.instruction(&Instruction::I64Const(
            ModuleActivationKind::Synchronous.word() as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_private_module_resume(
            0,
            TaggedLocals::new(self.result_local, self.result_tag_local),
            function,
        )?;
        self.store_i64_const_at_offset(
            0,
            MODULE_BODY_STATE_OFFSET,
            ModuleBodyState::Completed.word(),
            function,
        );
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(0, MODULE_ACTIVATION_OFFSET, activation, function);
        self.emit_load_async_module_entry_mode(activation, state, function);
        function.instruction(&Instruction::LocalGet(state));
        function.instruction(&Instruction::I64Const(
            AsyncModuleEntryMode::Execute.word() as i64
        ));
        function.instruction(&Instruction::I64Ne);
        self.load_i64_to_local_from_offset(
            activation,
            HEAP_ASYNC_RESUME_STATE_OFFSET,
            state,
            function,
        );
        function.instruction(&Instruction::LocalGet(state));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            activation,
            HEAP_ASYNC_PROMISE_PAYLOAD_OFFSET,
            promise,
            function,
        );
        self.load_i64_to_local_from_offset(
            activation,
            HEAP_ASYNC_PROMISE_RECORD_OFFSET,
            promise_record,
            function,
        );
        let realm = self.emit_module_execution_realm_context(0, function);
        self.emit_module_promise_reactions(
            0,
            promise_record,
            &realm,
            ModuleReactionContinuation::Body,
            function,
        )?;
        self.release_async_execution_realm_context(realm);
        self.emit_invoke_async_activation(
            activation,
            self.result_local,
            self.result_tag_local,
            function,
        );
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
            promise_record,
            PromiseSettlement::Reject,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_statement_result(function, ValueKind::Undefined);
        self.emit_settle_promise_record(
            promise_record,
            PromiseSettlement::Fulfill,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.store_i64_const_at_offset(activation, HEAP_ASYNC_COMPLETED_OFFSET, 1, function);
        function.instruction(&Instruction::End);
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_statement_result(function, ValueKind::Undefined);
        function.instruction(&Instruction::End);
        for local in [promise_record, promise, activation, kind, state] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    fn emit_module_effective_dependencies(
        &mut self,
        record: u32,
        limit: u32,
        dependencies: u32,
        dependency_count: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let requests = self.reserve_temp_local();
        let request_count = self.reserve_temp_local();
        let index = self.reserve_temp_local();
        let request = self.reserve_temp_local();
        let phase = self.reserve_temp_local();
        let target = self.reserve_temp_local();
        let gathered = self.reserve_temp_local();
        let gathered_count = self.reserve_temp_local();
        let gathered_index = self.reserve_temp_local();
        let address = self.reserve_temp_local();
        let found = self.reserve_temp_local();
        self.emit_module_allocate_words(limit, 8, dependencies, function)?;
        for local in [index, dependency_count] {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(local));
        }
        self.load_i64_to_local_from_offset(record, MODULE_REQUESTS_OFFSET, requests, function);
        self.load_i64_to_local_from_offset(
            record,
            MODULE_REQUEST_COUNT_OFFSET,
            request_count,
            function,
        );
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(request_count));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_module_array_address(requests, index, MODULE_REQUEST_SIZE, request, function);
        self.emit_load_module_request_phase_strict(request, phase, function);
        self.load_i64_to_local_from_offset(request, MODULE_REQUEST_TARGET_OFFSET, target, function);
        function.instruction(&Instruction::LocalGet(phase));
        function.instruction(&Instruction::I64Const(
            ModuleRequestPhase::Evaluation.word() as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_module_list_contains(dependencies, dependency_count, target, found, function);
        function.instruction(&Instruction::LocalGet(found));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_module_bounded_append(dependencies, dependency_count, limit, target, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_module_runtime_call(
            ModuleRuntimeOperation::Gather,
            &[target],
            gathered,
            gathered_count,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(gathered_index));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(gathered_index));
        function.instruction(&Instruction::LocalGet(gathered_count));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_module_array_address(gathered, gathered_index, 8, address, function);
        self.load_i64_to_local_from_offset(address, 0, target, function);
        self.emit_module_list_contains(dependencies, dependency_count, target, found, function);
        function.instruction(&Instruction::LocalGet(found));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_module_bounded_append(dependencies, dependency_count, limit, target, function);
        function.instruction(&Instruction::End);
        self.emit_module_increment(gathered_index, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_module_increment(index, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for local in [
            found,
            address,
            gathered_index,
            gathered_count,
            gathered,
            target,
            phase,
            request,
            index,
            request_count,
            requests,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    fn emit_module_enter_evaluation(
        &mut self,
        record: u32,
        frames: u32,
        depth: u32,
        active: u32,
        active_count: u32,
        index: u32,
        limit: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let frame = self.reserve_temp_local();
        let dependencies = self.reserve_temp_local();
        let count = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(depth));
        function.instruction(&Instruction::LocalGet(limit));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.store_i64_const_at_offset(
            record,
            MODULE_STATE_OFFSET,
            ModuleEvaluationState::Evaluating.word(),
            function,
        );
        self.store_i64_local_at_offset(record, MODULE_DFS_INDEX_OFFSET, index, function);
        self.store_i64_local_at_offset(record, MODULE_DFS_ANCESTOR_OFFSET, index, function);
        self.store_i64_const_at_offset(
            record,
            MODULE_PENDING_ASYNC_DEPENDENCIES_OFFSET,
            0,
            function,
        );
        self.emit_module_increment(index, 1, function);
        self.emit_module_effective_dependencies(record, limit, dependencies, count, function)?;
        self.emit_module_bounded_append(active, active_count, limit, record, function);
        self.emit_module_array_address(frames, depth, FRAME_SIZE, frame, function);
        self.store_i64_local_at_offset(frame, FRAME_RECORD, record, function);
        self.store_i64_local_at_offset(frame, FRAME_DEPENDENCIES, dependencies, function);
        self.store_i64_local_at_offset(frame, FRAME_COUNT, count, function);
        self.store_i64_const_at_offset(frame, FRAME_NEXT, 0, function);
        self.store_i64_const_at_offset(frame, FRAME_RETURNED_CHILD, 0, function);
        self.emit_module_increment(depth, 1, function);
        for local in [count, dependencies, frame] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    fn emit_module_register_parent(
        &mut self,
        child: u32,
        parent: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let graph = self.reserve_temp_local();
        let count = self.reserve_temp_local();
        let limit = self.reserve_temp_local();
        let occurrence = self.reserve_temp_local();
        let tail = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(child, MODULE_GRAPH_OFFSET, graph, function);
        self.load_i64_to_local_from_offset(
            graph,
            MODULE_GRAPH_PARENT_COUNT_OFFSET,
            count,
            function,
        );
        self.load_i64_to_local_from_offset(
            graph,
            MODULE_GRAPH_PARENT_BUDGET_OFFSET,
            limit,
            function,
        );
        function.instruction(&Instruction::LocalGet(count));
        function.instruction(&Instruction::LocalGet(limit));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.emit_module_increment(count, 1, function);
        self.store_i64_local_at_offset(graph, MODULE_GRAPH_PARENT_COUNT_OFFSET, count, function);
        self.emit_heap_alloc_const(MODULE_PARENT_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(occurrence));
        self.store_i64_local_at_offset(occurrence, MODULE_PARENT_MODULE_OFFSET, parent, function);
        self.store_i64_const_at_offset(occurrence, MODULE_PARENT_NEXT_OFFSET, 0, function);
        self.load_i64_to_local_from_offset(child, MODULE_ASYNC_PARENTS_TAIL_OFFSET, tail, function);
        function.instruction(&Instruction::LocalGet(tail));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_local_at_offset(
            child,
            MODULE_ASYNC_PARENTS_HEAD_OFFSET,
            occurrence,
            function,
        );
        function.instruction(&Instruction::Else);
        self.store_i64_local_at_offset(tail, MODULE_PARENT_NEXT_OFFSET, occurrence, function);
        function.instruction(&Instruction::End);
        self.store_i64_local_at_offset(
            child,
            MODULE_ASYNC_PARENTS_TAIL_OFFSET,
            occurrence,
            function,
        );
        self.load_i64_to_local_from_offset(
            parent,
            MODULE_PENDING_ASYNC_DEPENDENCIES_OFFSET,
            count,
            function,
        );
        self.emit_module_increment(count, 1, function);
        self.store_i64_local_at_offset(
            parent,
            MODULE_PENDING_ASYNC_DEPENDENCIES_OFFSET,
            count,
            function,
        );
        for local in [tail, occurrence, limit, count, graph] {
            self.release_temp_local(local);
        }
        Ok(())
    }
    pub(super) fn emit_module_evaluation_runtime(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let root = self.reserve_temp_local();
        let state = self.reserve_temp_local();
        let graph = self.reserve_temp_local();
        let limit = self.reserve_temp_local();
        let promise = self.reserve_temp_local();
        let promise_record = self.reserve_temp_local();
        let frames = self.reserve_temp_local();
        let depth = self.reserve_temp_local();
        let active = self.reserve_temp_local();
        let active_count = self.reserve_temp_local();
        let dfs_index = self.reserve_temp_local();
        let frame = self.reserve_temp_local();
        let record = self.reserve_temp_local();
        let child = self.reserve_temp_local();
        let candidate_root = self.reserve_temp_local();
        let ancestor = self.reserve_temp_local();
        let child_ancestor = self.reserve_temp_local();
        let dependencies = self.reserve_temp_local();
        let count = self.reserve_temp_local();
        let next = self.reserve_temp_local();
        let address = self.reserve_temp_local();
        let order = self.reserve_temp_local();
        let pending = self.reserve_temp_local();
        let kind = self.reserve_temp_local();
        let failed = self.reserve_temp_local();
        let error_payload = self.reserve_temp_local();
        let error_tag = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(0));
        function.instruction(&Instruction::LocalSet(root));
        self.emit_load_module_state_strict(root, state, function);
        function.instruction(&Instruction::LocalGet(state));
        function.instruction(&Instruction::I64Const(
            ModuleEvaluationState::EvaluatingAsync.word() as i64,
        ));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            root,
            MODULE_CYCLE_ROOT_OFFSET,
            candidate_root,
            function,
        );
        function.instruction(&Instruction::LocalGet(candidate_root));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(candidate_root));
        function.instruction(&Instruction::LocalSet(root));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            root,
            MODULE_EVALUATION_PROMISE_OFFSET,
            promise,
            function,
        );
        function.instruction(&Instruction::LocalGet(promise));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_module_state_strict(root, state, function);
        function.instruction(&Instruction::LocalGet(state));
        function.instruction(&Instruction::I64Const(
            ModuleEvaluationState::Evaluating.word() as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        // User deferred access checks readiness before Evaluate. A private
        // reentry can reuse an existing capability, never start an active body.
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        let realm = self.emit_module_execution_realm_context(root, function);
        let context = self.emit_async_execution_promise_allocation_context(&realm, function);
        self.emit_alloc_promise_with_prototype(context, promise, promise_record, function)?;
        self.release_async_execution_realm_context(realm);
        self.store_i64_local_at_offset(root, MODULE_EVALUATION_PROMISE_OFFSET, promise, function);
        self.emit_load_module_state_strict(root, state, function);
        function.instruction(&Instruction::LocalGet(state));
        function.instruction(&Instruction::I64Const(
            ModuleEvaluationState::Linked.word() as i64
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(root, MODULE_GRAPH_OFFSET, graph, function);
        self.load_i64_to_local_from_offset(graph, MODULE_GRAPH_COUNT_OFFSET, limit, function);
        self.emit_module_allocate_words(limit, FRAME_SIZE, frames, function)?;
        self.emit_module_allocate_words(limit, 8, active, function)?;
        for local in [depth, active_count, dfs_index, failed] {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(local));
        }
        self.emit_module_enter_evaluation(
            root,
            frames,
            depth,
            active,
            active_count,
            dfs_index,
            limit,
            function,
        )?;
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(depth));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(failed));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(depth));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(next));
        self.emit_module_array_address(frames, next, FRAME_SIZE, frame, function);
        self.load_i64_to_local_from_offset(frame, FRAME_RECORD, record, function);
        self.load_i64_to_local_from_offset(frame, FRAME_RETURNED_CHILD, child, function);
        function.instruction(&Instruction::LocalGet(child));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.store_i64_const_at_offset(frame, FRAME_RETURNED_CHILD, 0, function);
        self.emit_load_module_completion_strict(child, state, function);
        function.instruction(&Instruction::LocalGet(state));
        function.instruction(&Instruction::I64Const(
            ModuleEvaluationCompletion::Throw.word() as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            child,
            MODULE_ERROR_PAYLOAD_OFFSET,
            error_payload,
            function,
        );
        self.load_i64_to_local_from_offset(child, MODULE_ERROR_TAG_OFFSET, error_tag, function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(failed));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.emit_load_module_state_strict(child, state, function);
        function.instruction(&Instruction::LocalGet(state));
        function.instruction(&Instruction::I64Const(
            ModuleEvaluationState::Evaluating.word() as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(record, MODULE_DFS_ANCESTOR_OFFSET, ancestor, function);
        self.load_i64_to_local_from_offset(
            child,
            MODULE_DFS_ANCESTOR_OFFSET,
            child_ancestor,
            function,
        );
        function.instruction(&Instruction::LocalGet(ancestor));
        function.instruction(&Instruction::LocalGet(child_ancestor));
        function.instruction(&Instruction::LocalGet(ancestor));
        function.instruction(&Instruction::LocalGet(child_ancestor));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::Select);
        function.instruction(&Instruction::LocalSet(ancestor));
        self.store_i64_local_at_offset(record, MODULE_DFS_ANCESTOR_OFFSET, ancestor, function);
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(child, MODULE_CYCLE_ROOT_OFFSET, child, function);
        function.instruction(&Instruction::LocalGet(child));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.emit_load_module_completion_strict(child, state, function);
        function.instruction(&Instruction::LocalGet(state));
        function.instruction(&Instruction::I64Const(
            ModuleEvaluationCompletion::Throw.word() as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            child,
            MODULE_ERROR_PAYLOAD_OFFSET,
            error_payload,
            function,
        );
        self.load_i64_to_local_from_offset(child, MODULE_ERROR_TAG_OFFSET, error_tag, function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(failed));
        function.instruction(&Instruction::Br(3));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(child, MODULE_ASYNC_ORDER_OFFSET, order, function);
        function.instruction(&Instruction::LocalGet(order));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_module_register_parent(child, record, function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(frame, FRAME_NEXT, next, function);
        self.load_i64_to_local_from_offset(frame, FRAME_COUNT, count, function);
        function.instruction(&Instruction::LocalGet(next));
        function.instruction(&Instruction::LocalGet(count));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(frame, FRAME_DEPENDENCIES, dependencies, function);
        self.emit_module_array_address(dependencies, next, 8, address, function);
        self.load_i64_to_local_from_offset(address, 0, child, function);
        self.emit_module_increment(next, 1, function);
        self.store_i64_local_at_offset(frame, FRAME_NEXT, next, function);
        self.store_i64_local_at_offset(frame, FRAME_RETURNED_CHILD, child, function);
        self.emit_load_module_state_strict(child, state, function);
        function.instruction(&Instruction::LocalGet(state));
        function.instruction(&Instruction::I64Const(
            ModuleEvaluationState::Linked.word() as i64
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_module_enter_evaluation(
            child,
            frames,
            depth,
            active,
            active_count,
            dfs_index,
            limit,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        self.emit_load_module_activation_kind_strict(record, kind, function);
        self.load_i64_to_local_from_offset(
            record,
            MODULE_PENDING_ASYNC_DEPENDENCIES_OFFSET,
            pending,
            function,
        );
        function.instruction(&Instruction::LocalGet(kind));
        function.instruction(&Instruction::I64Const(
            ModuleActivationKind::Async.word() as i64
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(pending));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            graph,
            MODULE_GRAPH_NEXT_ASYNC_ORDER_OFFSET,
            order,
            function,
        );
        self.store_i64_local_at_offset(record, MODULE_ASYNC_ORDER_OFFSET, order, function);
        self.emit_module_increment(order, 1, function);
        self.store_i64_local_at_offset(
            graph,
            MODULE_GRAPH_NEXT_ASYNC_ORDER_OFFSET,
            order,
            function,
        );
        function.instruction(&Instruction::LocalGet(pending));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_module_runtime_call(
            ModuleRuntimeOperation::Execute,
            &[record],
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_module_runtime_call(
            ModuleRuntimeOperation::Execute,
            &[record],
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalSet(error_payload));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalSet(error_tag));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(failed));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(record, MODULE_DFS_INDEX_OFFSET, next, function);
        self.load_i64_to_local_from_offset(record, MODULE_DFS_ANCESTOR_OFFSET, ancestor, function);
        function.instruction(&Instruction::LocalGet(next));
        function.instruction(&Instruction::LocalGet(ancestor));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(active_count));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.emit_module_increment(active_count, -1, function);
        self.emit_module_array_address(active, active_count, 8, address, function);
        self.load_i64_to_local_from_offset(address, 0, child, function);
        self.store_i64_local_at_offset(child, MODULE_CYCLE_ROOT_OFFSET, record, function);
        self.load_i64_to_local_from_offset(child, MODULE_ASYNC_ORDER_OFFSET, order, function);
        function.instruction(&Instruction::LocalGet(order));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_const_at_offset(
            child,
            MODULE_STATE_OFFSET,
            ModuleEvaluationState::Evaluated.word(),
            function,
        );
        self.store_i64_const_at_offset(
            child,
            MODULE_COMPLETION_OFFSET,
            ModuleEvaluationCompletion::Normal.word(),
            function,
        );
        function.instruction(&Instruction::Else);
        self.store_i64_const_at_offset(
            child,
            MODULE_STATE_OFFSET,
            ModuleEvaluationState::EvaluatingAsync.word(),
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(child));
        function.instruction(&Instruction::LocalGet(record));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::BrIf(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_module_increment(depth, -1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(failed));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(active_count));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        self.emit_module_increment(active_count, -1, function);
        self.emit_module_array_address(active, active_count, 8, address, function);
        self.load_i64_to_local_from_offset(address, 0, record, function);
        self.emit_module_cache_throw(record, error_payload, error_tag, function);
        self.emit_module_settle_evaluation_if_terminal(record, function)?;
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_module_settle_evaluation_if_terminal(root, function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(promise));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);
        for local in [
            error_tag,
            error_payload,
            failed,
            kind,
            pending,
            order,
            address,
            next,
            count,
            dependencies,
            child_ancestor,
            ancestor,
            candidate_root,
            child,
            record,
            frame,
            dfs_index,
            active_count,
            active,
            depth,
            frames,
            promise_record,
            promise,
            limit,
            graph,
            state,
            root,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
