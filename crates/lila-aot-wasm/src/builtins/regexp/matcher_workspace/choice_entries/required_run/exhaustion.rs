//! Failed-group traces belong to the actual active ancestor path. Descending
//! into a copied node preserves its parent's trace; reset retires child flags.
use super::*;

impl ChoiceStack<'_> {
    pub(in crate::builtins::regexp::matcher_workspace::choice_entries) fn clear_active_run(
        self,
        f: &mut Function,
    ) {
        for local in [
            self.workspace.active_run_offset,
            self.workspace.active_run_row,
            self.workspace.active_run_node,
        ] {
            f.instruction(&Instruction::I64Const(EMPTY_LINK));
            local.store(f);
        }
    }

    fn with_active_run(
        self,
        builder: &FunctionBuilder<'_>,
        f: &mut Function,
        finish: impl for<'entry> FnOnce(ChoiceEntry<'entry>, &mut Function),
    ) {
        self.workspace.active_run_offset.load(f);
        f.instruction(&Instruction::I64Const(EMPTY_LINK));
        f.instruction(&Instruction::I64Ne);
        f.instruction(&Instruction::If(BlockType::Empty));
        let address = builder.runtime_schema().reserve_i64_local(f);
        self.check_entry(builder, self.workspace.active_run_offset, address, f);
        let entry = ChoiceEntry {
            stack: self,
            offset: self.workspace.active_run_offset,
            address,
        };
        entry.is_kind(ChoiceEntryKind::RequiredRun, f);
        f.instruction(&Instruction::I32Eqz);
        reject(self.workspace, builder, f);
        finish(entry, f);
        builder.runtime_schema().release_i64_local(address, f);
        f.instruction(&Instruction::End);
    }

    fn visit_active_ancestry(
        self,
        builder: &FunctionBuilder<'_>,
        f: &mut Function,
        mut finish: impl FnMut(I64Local, &mut Function),
    ) {
        self.with_active_run(builder, f, |entry, f| {
            let workspace = self.workspace;
            let schema = builder.runtime_schema();
            let root = RunLayout::read(entry.address, workspace, builder, f);
            let relative = schema.reserve_i64_local(f);
            let state = schema.reserve_i64_local(f);
            let parent = schema.reserve_i64_local(f);
            let hops = schema.reserve_i64_local(f);
            workspace.active_run_node.load(f);
            relative.store(f);
            f.instruction(&Instruction::I64Const(0));
            hops.store(f);
            f.instruction(&Instruction::Block(BlockType::Empty));
            f.instruction(&Instruction::Loop(BlockType::Empty));
            relative.load(f);
            f.instruction(&Instruction::I64Const(EMPTY_LINK));
            f.instruction(&Instruction::I64Eq);
            f.instruction(&Instruction::BrIf(1));
            hops.load(f);
            root.node_count.load(f);
            f.instruction(&Instruction::I64GtU);
            reject(workspace, builder, f);
            root.state_for(relative, state, parent, workspace, builder, f);
            finish(state, f);
            parent.load(f);
            relative.store(f);
            hops.load(f);
            f.instruction(&Instruction::I64Const(1));
            f.instruction(&Instruction::I64Add);
            hops.store(f);
            f.instruction(&Instruction::Br(0));
            f.instruction(&Instruction::End);
            f.instruction(&Instruction::End);
            for local in [hops, parent, state, relative] {
                schema.release_i64_local(local, f);
            }
            root.release(builder, f);
        });
    }

    pub(in crate::builtins::regexp::matcher_workspace::choice_entries) fn invalidate_active_run(
        self,
        builder: &FunctionBuilder<'_>,
        f: &mut Function,
    ) {
        self.visit_active_ancestry(builder, f, |state, f| {
            for (offset, value) in [(104, 1), (112, 0)] {
                state.load(f);
                f.instruction(&Instruction::I32WrapI64);
                f.instruction(&Instruction::I64Const(value));
                f.instruction(&Instruction::I64Store(FunctionBuilder::memarg8(offset)));
            }
        });
        self.clear_active_run(f);
    }

    pub(in crate::builtins::regexp::matcher_workspace::choice_entries) fn clear_removed_active_run(
        self,
        removed: I64Local,
        f: &mut Function,
    ) {
        self.workspace.active_run_offset.load(f);
        f.instruction(&Instruction::I64Const(EMPTY_LINK));
        f.instruction(&Instruction::I64Ne);
        self.workspace.active_run_offset.load(f);
        removed.load(f);
        f.instruction(&Instruction::I64GeU);
        f.instruction(&Instruction::I32And);
        f.instruction(&Instruction::If(BlockType::Empty));
        self.clear_active_run(f);
        f.instruction(&Instruction::End);
    }

    /// Actual End observes only its admitted row. An inner End must not taint
    /// an unobserved outer row; all active ancestor identities remain available.
    pub(in crate::builtins::regexp) fn observe_repeat_end(
        self,
        builder: &FunctionBuilder<'_>,
        state: I64Local,
        f: &mut Function,
    ) {
        self.visit_active_ancestry(builder, f, |saved, f| {
            self.workspace.base.load(f);
            ChoiceStack::load(saved, 40, f);
            f.instruction(&Instruction::I64Add);
            state.load(f);
            f.instruction(&Instruction::I64Eq);
            f.instruction(&Instruction::If(BlockType::Empty));
            for (offset, value) in [(104, 1), (112, 0)] {
                saved.load(f);
                f.instruction(&Instruction::I32WrapI64);
                f.instruction(&Instruction::I64Const(value));
                f.instruction(&Instruction::I64Store(FunctionBuilder::memarg8(offset)));
            }
            f.instruction(&Instruction::End);
        });
    }
}

impl ChoiceEntry<'_> {
    pub(super) fn begin_backtracked_node(
        &self,
        layout: &RunLayout,
        relative: I64Local,
        builder: &FunctionBuilder<'_>,
        eligible: I32Local,
        f: &mut Function,
    ) {
        let workspace = self.stack.workspace;
        workspace.active_run_offset.load(f);
        self.offset.load(f);
        f.instruction(&Instruction::I64Eq);
        f.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        layout.pending_complete.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Eq);
        layout.observed_group.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::I32And);
        f.instruction(&Instruction::Else);
        self.stack.invalidate_active_run(builder, f);
        f.instruction(&Instruction::I32Const(0));
        f.instruction(&Instruction::End);
        eligible.store(f);
        self.offset.load(f);
        workspace.active_run_offset.store(f);
        relative.load(f);
        workspace.active_run_node.store(f);
        workspace.base.load(f);
        layout.row.load(f);
        f.instruction(&Instruction::I64Add);
        workspace.active_run_row.store(f);
        f.instruction(&Instruction::I64Const(0));
        layout.pending_complete.store(f);
        ChoiceStack::store(layout.address, 112, layout.pending_complete, f);
    }
}
