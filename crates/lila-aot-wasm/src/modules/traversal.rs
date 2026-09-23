//! Side-effect-free iterative walks for deferred readiness and async gathering.

use super::*;

#[derive(Clone, Copy)]
enum ModuleTraversal {
    Readiness,
    AsyncDependencies,
}

impl FunctionBuilder<'_> {
    pub(super) fn emit_module_scc_evaluated(
        &mut self,
        record: u32,
        evaluated: u32,
        function: &mut Function,
    ) {
        let root = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(record, MODULE_CYCLE_ROOT_OFFSET, root, function);
        function.instruction(&Instruction::LocalGet(root));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(record));
        function.instruction(&Instruction::LocalSet(root));
        function.instruction(&Instruction::End);
        self.emit_load_module_state_strict(root, evaluated, function);
        function.instruction(&Instruction::LocalGet(evaluated));
        function.instruction(&Instruction::I64Const(
            ModuleEvaluationState::Evaluated.word() as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(evaluated));
        self.release_temp_local(root);
    }

    pub(super) fn emit_module_readiness_runtime(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_module_traversal(ModuleTraversal::Readiness, function)
    }

    pub(super) fn emit_module_gather_runtime(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_module_traversal(ModuleTraversal::AsyncDependencies, function)
    }

    fn emit_module_traversal(
        &mut self,
        purpose: ModuleTraversal,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let graph = self.reserve_temp_local();
        let limit = self.reserve_temp_local();
        let frames = self.reserve_temp_local();
        let depth = self.reserve_temp_local();
        let seen = self.reserve_temp_local();
        let seen_count = self.reserve_temp_local();
        let output = self.reserve_temp_local();
        let output_count = self.reserve_temp_local();
        let candidate = self.reserve_temp_local();
        let found = self.reserve_temp_local();
        let complete = self.reserve_temp_local();
        let state = self.reserve_temp_local();
        let kind = self.reserve_temp_local();
        let frame = self.reserve_temp_local();
        let record = self.reserve_temp_local();
        let next = self.reserve_temp_local();
        let count = self.reserve_temp_local();
        let requests = self.reserve_temp_local();
        let request = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(0, MODULE_GRAPH_OFFSET, graph, function);
        self.load_i64_to_local_from_offset(graph, MODULE_GRAPH_COUNT_OFFSET, limit, function);
        self.emit_module_allocate_words(limit, 16, frames, function)?;
        self.emit_module_allocate_words(limit, 8, seen, function)?;
        if matches!(purpose, ModuleTraversal::AsyncDependencies) {
            self.emit_module_allocate_words(limit, 8, output, function)?;
        }
        for local in [depth, seen_count, output_count] {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(local));
        }
        function.instruction(&Instruction::LocalGet(0));
        function.instruction(&Instruction::LocalSet(candidate));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(candidate));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_module_list_contains(seen, seen_count, candidate, found, function);
        function.instruction(&Instruction::LocalGet(found));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_module_bounded_append(seen, seen_count, limit, candidate, function);
        self.emit_load_module_state_strict(candidate, state, function);
        self.emit_load_module_activation_kind_strict(candidate, kind, function);
        self.emit_module_scc_evaluated(candidate, complete, function);
        match purpose {
            ModuleTraversal::Readiness => {
                function.instruction(&Instruction::LocalGet(complete));
                function.instruction(&Instruction::I64Eqz);
            }
            ModuleTraversal::AsyncDependencies => {
                function.instruction(&Instruction::LocalGet(state));
                function.instruction(&Instruction::I64Const(
                    ModuleEvaluationState::Evaluating.word() as i64,
                ));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::LocalGet(complete));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::I32And);
            }
        }
        function.instruction(&Instruction::If(BlockType::Empty));
        if matches!(purpose, ModuleTraversal::Readiness) {
            function.instruction(&Instruction::LocalGet(state));
            function.instruction(&Instruction::I64Const(
                ModuleEvaluationState::Evaluating.word() as i64,
            ));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::LocalGet(state));
            function.instruction(&Instruction::I64Const(
                ModuleEvaluationState::EvaluatingAsync.word() as i64,
            ));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
            function.instruction(&Instruction::LocalGet(kind));
            function.instruction(&Instruction::I64Const(
                ModuleActivationKind::Async.word() as i64
            ));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(self.result_local));
            function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
            function.instruction(&Instruction::LocalSet(self.result_tag_local));
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
        } else {
            function.instruction(&Instruction::LocalGet(kind));
            function.instruction(&Instruction::I64Const(
                ModuleActivationKind::Async.word() as i64
            ));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_module_bounded_append(output, output_count, limit, candidate, function);
            function.instruction(&Instruction::Else);
        }
        // One cursor per entered record preserves DFS order without pushing
        // duplicate child occurrences or consuming native call-stack depth.
        function.instruction(&Instruction::LocalGet(depth));
        function.instruction(&Instruction::LocalGet(limit));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.emit_module_array_address(frames, depth, 16, frame, function);
        self.store_i64_local_at_offset(frame, 0, candidate, function);
        self.store_i64_const_at_offset(frame, 8, 0, function);
        self.emit_module_increment(depth, 1, function);
        if matches!(purpose, ModuleTraversal::AsyncDependencies) {
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(candidate));
        function.instruction(&Instruction::LocalGet(depth));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(depth));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(next));
        self.emit_module_array_address(frames, next, 16, frame, function);
        self.load_i64_to_local_from_offset(frame, 0, record, function);
        self.load_i64_to_local_from_offset(frame, 8, next, function);
        self.load_i64_to_local_from_offset(record, MODULE_REQUEST_COUNT_OFFSET, count, function);
        function.instruction(&Instruction::LocalGet(next));
        function.instruction(&Instruction::LocalGet(count));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(record, MODULE_REQUESTS_OFFSET, requests, function);
        self.emit_module_array_address(requests, next, MODULE_REQUEST_SIZE, request, function);
        self.emit_load_module_request_phase_strict(request, state, function);
        self.load_i64_to_local_from_offset(
            request,
            MODULE_REQUEST_TARGET_OFFSET,
            candidate,
            function,
        );
        self.emit_module_increment(next, 1, function);
        self.store_i64_local_at_offset(frame, 8, next, function);
        function.instruction(&Instruction::Else);
        self.emit_module_increment(depth, -1, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        match purpose {
            ModuleTraversal::Readiness => {
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
            }
            ModuleTraversal::AsyncDependencies => {
                // The private helper ABI returns the list pointer and count.
                function.instruction(&Instruction::LocalGet(output));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::LocalGet(output_count));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
            }
        }
        for local in [
            request,
            requests,
            count,
            next,
            record,
            frame,
            kind,
            state,
            complete,
            found,
            candidate,
            output_count,
            output,
            seen_count,
            seen,
            depth,
            frames,
            limit,
            graph,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
