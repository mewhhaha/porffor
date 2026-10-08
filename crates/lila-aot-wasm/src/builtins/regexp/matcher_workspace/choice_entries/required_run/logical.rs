//! Actual ordered continuations traverse a checked immutable tree and its
//! source-sized state table. A logical leaf retains its complete ancestor path.
use super::path::{PathWord, PlaybackPath};
use super::*;

pub(in crate::builtins::regexp) struct LogicalRunEntry<'entry> {
    physical: &'entry ChoiceEntry<'entry>,
    root: &'entry RunLayout,
    path: &'entry PlaybackPath,
    depth: I64Local,
    template: I64Local,
}

#[derive(Clone, Copy)]
enum RunEntryConsumption {
    FailedBacktrack,
    AssertionTruncate,
}

impl RunLayout {
    fn validate_live(
        &self,
        workspace: &MatcherWorkspace,
        builder: &FunctionBuilder<'_>,
        f: &mut Function,
    ) {
        self.check_state_shape(workspace, builder, f);
        self.check_table(workspace, builder, f);
        self.remaining_used.load(f);
        f.instruction(&Instruction::I64Eqz);
        self.affine_used.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::I32Or);
        reject(workspace, builder, f);
    }

    fn selected_kind(
        &self,
        template: I64Local,
        index: I64Local,
        older: I64Local,
        out: I64Local,
        workspace: &MatcherWorkspace,
        builder: &FunctionBuilder<'_>,
        f: &mut Function,
    ) {
        ChoiceStack::load(template, 0, f);
        workspace.snapshot_bytes.load(f);
        f.instruction(&Instruction::I64Ne);
        ChoiceStack::load(template, 16, f);
        f.instruction(&Instruction::I64Const(
            ChoiceEntryKind::LazyProgressAttempt.word(),
        ));
        f.instruction(&Instruction::I64GtU);
        f.instruction(&Instruction::I32Or);
        reject(workspace, builder, f);
        ChoiceStack::load(template, 16, f);
        out.store(f);
        older.load(f);
        f.instruction(&Instruction::I64Eqz);
        index.load(f);
        self.index.load(f);
        f.instruction(&Instruction::I64Eq);
        f.instruction(&Instruction::I32And);
        self.current_kind_override.load(f);
        f.instruction(&Instruction::I64Const(EMPTY_LINK));
        f.instruction(&Instruction::I64Ne);
        f.instruction(&Instruction::I32And);
        f.instruction(&Instruction::If(BlockType::Empty));
        out.load(f);
        f.instruction(&Instruction::I64Const(
            ChoiceEntryKind::LazyProgressChoice.word(),
        ));
        f.instruction(&Instruction::I64Ne);
        reject(workspace, builder, f);
        self.current_kind_override.load(f);
        out.store(f);
        f.instruction(&Instruction::End);
    }

    fn has_older_group(&self, f: &mut Function) {
        self.remaining_used.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64GtU);
        self.remaining_used.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Eq);
        self.remaining.load(f);
        f.instruction(&Instruction::I32WrapI64);
        f.instruction(&Instruction::I32Load(FunctionBuilder::memarg32(0)));
        f.instruction(&Instruction::I32Const(1));
        f.instruction(&Instruction::I32GtU);
        f.instruction(&Instruction::I32And);
        f.instruction(&Instruction::I32Or);
    }

    fn advance_group(
        &self,
        root: &RunLayout,
        workspace: &MatcherWorkspace,
        builder: &FunctionBuilder<'_>,
        f: &mut Function,
    ) {
        self.remaining()
            .change_one(UnitDelta::Decrement, workspace, builder, f);
        self.remaining_used.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::I32Eqz);
        f.instruction(&Instruction::If(BlockType::Empty));
        self.affine()
            .change_one(UnitDelta::Increment, workspace, builder, f);
        self.count.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Sub);
        self.index.store(f);
        // This node now owns a full group. Its captured descendants still
        // start tainted, including any assertion-truncated initial suffix.
        f.instruction(&Instruction::I64Const(0));
        self.observed_group.store(f);
        self.reset_children(root, workspace, builder, f);
        f.instruction(&Instruction::End);
    }

    fn publish_cursor(&self, f: &mut Function) {
        self.publish_used(f);
        for (offset, value) in [
            (32, self.index),
            (96, self.current_kind_override),
            (104, self.observed_group),
            (112, self.pending_complete),
        ] {
            ChoiceStack::store(self.address, offset, value, f);
        }
    }

    fn consume(
        &self,
        root: &RunLayout,
        reason: RunEntryConsumption,
        workspace: &MatcherWorkspace,
        builder: &FunctionBuilder<'_>,
        f: &mut Function,
    ) {
        self.index.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::If(BlockType::Empty));
        if let RunEntryConsumption::FailedBacktrack = reason {
            self.observed_group.load(f);
            f.instruction(&Instruction::I64Eqz);
            f.instruction(&Instruction::I64ExtendI32U);
            self.pending_complete.store(f);
            f.instruction(&Instruction::I64Const(0));
            self.observed_group.store(f);
        }
        self.advance_group(root, workspace, builder, f);
        f.instruction(&Instruction::Else);
        self.index.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Sub);
        self.index.store(f);
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::I64Const(EMPTY_LINK));
        self.current_kind_override.store(f);
        self.publish_cursor(f);
    }

    fn restore_affine_row(
        &self,
        older: I64Local,
        workspace: &MatcherWorkspace,
        builder: &FunctionBuilder<'_>,
        f: &mut Function,
    ) {
        let schema = builder.runtime_schema();
        let row = schema.reserve_i64_local(f);
        let minlimbs = schema.reserve_i64_local(f);
        let maxlimbs = schema.reserve_i64_local(f);
        let minused = schema.reserve_i64_local(f);
        let maxused = schema.reserve_i64_local(f);
        workspace.base.load(f);
        self.row.load(f);
        f.instruction(&Instruction::I64Add);
        row.store(f);
        row.load(f);
        f.instruction(&Instruction::I64Const(
            REGEXP_REPEAT_STATE_HEADER_SIZE as i64,
        ));
        f.instruction(&Instruction::I64Add);
        minlimbs.store(f);
        minlimbs.load(f);
        self.mincap.load(f);
        f.instruction(&Instruction::I64Const(4));
        f.instruction(&Instruction::I64Mul);
        f.instruction(&Instruction::I64Add);
        maxlimbs.store(f);
        let minimum = NaturalRegion {
            limbs: minlimbs,
            capacity: self.mincap,
            used: minused,
        };
        minimum.copy_from(self.affine(), f);
        older.load(f);
        f.instruction(&Instruction::I32WrapI64);
        f.instruction(&Instruction::If(BlockType::Empty));
        minimum.change_one(UnitDelta::Increment, workspace, builder, f);
        f.instruction(&Instruction::End);
        ChoiceStack::store(row, RegExpRepeatStateWord::MinimumUsed.offset(), minused, f);
        self.maximum_kind.load(f);
        f.instruction(&Instruction::I64Const(
            RegExpRepeatMaximumKind::Finite.word() as i64,
        ));
        f.instruction(&Instruction::I64Eq);
        f.instruction(&Instruction::If(BlockType::Empty));
        self.base().add_into(
            minimum,
            NaturalRegion {
                limbs: maxlimbs,
                capacity: self.maxcap,
                used: maxused,
            },
            workspace,
            builder,
            f,
        );
        ChoiceStack::store(row, RegExpRepeatStateWord::MaximumUsed.offset(), maxused, f);
        f.instruction(&Instruction::End);
        for local in [maxused, minused, maxlimbs, minlimbs, row] {
            schema.release_i64_local(local, f);
        }
    }
}

impl LogicalRunEntry<'_> {
    pub(in crate::builtins::regexp::matcher_workspace::choice_entries) fn restore_cursor(
        &self,
        cursor: RegExpInputCursor,
        f: &mut Function,
    ) {
        for (offset, local) in [
            (40, cursor.byte),
            (48, cursor.utf16),
            (56, cursor.on_low_surrogate),
        ] {
            ChoiceStack::load(self.template, offset, f);
            local.store(f);
        }
    }
    pub(in crate::builtins::regexp::matcher_workspace::choice_entries) fn load_utf16(
        &self,
        f: &mut Function,
    ) {
        ChoiceStack::load(self.template, 48, f);
    }
    fn restore_slice(&self, start: Option<I64Local>, bytes: I64Local, f: &mut Function) {
        let workspace = self.physical.stack.workspace;
        workspace.base.load(f);
        if let Some(start) = start {
            start.load(f);
            f.instruction(&Instruction::I64Add);
        }
        f.instruction(&Instruction::I32WrapI64);
        self.template.load(f);
        f.instruction(&Instruction::I64Const(SNAPSHOT_HEADER_BYTES as i64));
        f.instruction(&Instruction::I64Add);
        if let Some(start) = start {
            start.load(f);
            f.instruction(&Instruction::I64Add);
        }
        f.instruction(&Instruction::I32WrapI64);
        bytes.load(f);
        f.instruction(&Instruction::I32WrapI64);
        f.instruction(&Instruction::MemoryCopy {
            src_mem: 0,
            dst_mem: 0,
        });
    }
    pub(in crate::builtins::regexp::matcher_workspace::choice_entries) fn restore_captures(
        &self,
        f: &mut Function,
    ) {
        self.restore_slice(None, self.physical.stack.workspace.capture_bytes, f);
    }

    fn patch_ancestors(&self, builder: &FunctionBuilder<'_>, f: &mut Function) {
        let workspace = self.physical.stack.workspace;
        let schema = builder.runtime_schema();
        let level = schema.reserve_i64_local(f);
        let definition = schema.reserve_i64_local(f);
        let state = schema.reserve_i64_local(f);
        let older = schema.reserve_i64_local(f);
        self.depth.load(f);
        level.store(f);
        f.instruction(&Instruction::Block(BlockType::Empty));
        f.instruction(&Instruction::Loop(BlockType::Empty));
        self.path
            .read(level, PathWord::Definition, definition, schema, f);
        self.path.read(level, PathWord::State, state, schema, f);
        self.path.read(level, PathWord::Older, older, schema, f);
        let node = RunLayout::read_state(definition, state, workspace, builder, f);
        node.restore_affine_row(older, workspace, builder, f);
        node.release(builder, f);
        level.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::BrIf(1));
        level.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Sub);
        level.store(f);
        f.instruction(&Instruction::Br(0));
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        for local in [older, state, definition, level] {
            schema.release_i64_local(local, f);
        }
    }
    pub(in crate::builtins::regexp::matcher_workspace::choice_entries) fn restore_repeats(
        &self,
        builder: &FunctionBuilder<'_>,
        f: &mut Function,
    ) {
        let workspace = self.physical.stack.workspace;
        self.restore_slice(Some(workspace.capture_bytes), workspace.repeat_bytes, f);
        self.patch_ancestors(builder, f);
    }
    fn restore_state(&self, builder: &FunctionBuilder<'_>, f: &mut Function) {
        self.restore_slice(None, self.physical.stack.workspace.live_bytes, f);
        self.patch_ancestors(builder, f);
    }

    /// Assertion truncation commits only the selected path. Removed newer
    /// alternatives cannot become a failed-group witness, including descendants.
    pub(in crate::builtins::regexp::matcher_workspace::choice_entries) fn discard_through(
        &self,
        builder: &FunctionBuilder<'_>,
        f: &mut Function,
    ) {
        let workspace = self.physical.stack.workspace;
        let schema = builder.runtime_schema();
        self.physical.stack.invalidate_active_run(builder, f);
        let level = schema.reserve_i64_local(f);
        let definition = schema.reserve_i64_local(f);
        let relative = schema.reserve_i64_local(f);
        let state = schema.reserve_i64_local(f);
        let parent = schema.reserve_i64_local(f);
        let index = schema.reserve_i64_local(f);
        let older = schema.reserve_i64_local(f);
        f.instruction(&Instruction::I64Const(0));
        level.store(f);
        f.instruction(&Instruction::Block(BlockType::Empty));
        f.instruction(&Instruction::Loop(BlockType::Empty));
        self.path
            .read(level, PathWord::Definition, definition, schema, f);
        definition.load(f);
        self.root.definition.load(f);
        f.instruction(&Instruction::I64Sub);
        relative.store(f);
        self.root
            .state_for(relative, state, parent, workspace, builder, f);
        let node = RunLayout::read_state(definition, state, workspace, builder, f);
        self.path.read(level, PathWord::Index, index, schema, f);
        self.path.read(level, PathWord::Older, older, schema, f);
        older.load(f);
        f.instruction(&Instruction::I32WrapI64);
        f.instruction(&Instruction::If(BlockType::Empty));
        node.advance_group(self.root, workspace, builder, f);
        f.instruction(&Instruction::End);
        index.load(f);
        node.index.store(f);
        f.instruction(&Instruction::I64Const(EMPTY_LINK));
        node.current_kind_override.store(f);
        f.instruction(&Instruction::I64Const(1));
        node.observed_group.store(f);
        f.instruction(&Instruction::I64Const(0));
        node.pending_complete.store(f);
        node.publish_cursor(f);
        level.load(f);
        self.depth.load(f);
        f.instruction(&Instruction::I64Eq);
        f.instruction(&Instruction::If(BlockType::Empty));
        node.consume(
            self.root,
            RunEntryConsumption::AssertionTruncate,
            workspace,
            builder,
            f,
        );
        f.instruction(&Instruction::Br(2));
        f.instruction(&Instruction::End);
        node.release(builder, f);
        level.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        level.store(f);
        f.instruction(&Instruction::Br(0));
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        // A fully removed embedded node has no selected continuation left.
        self.depth.load(f);
        level.store(f);
        f.instruction(&Instruction::Block(BlockType::Empty));
        f.instruction(&Instruction::Loop(BlockType::Empty));
        self.path
            .read(level, PathWord::Definition, definition, schema, f);
        definition.load(f);
        self.root.definition.load(f);
        f.instruction(&Instruction::I64Sub);
        relative.store(f);
        self.root
            .state_for(relative, state, parent, workspace, builder, f);
        ChoiceStack::load(state, 64, f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::I32Eqz);
        f.instruction(&Instruction::BrIf(1));
        level.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::BrIf(1));
        level.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Sub);
        level.store(f);
        self.path
            .read(level, PathWord::Definition, definition, schema, f);
        definition.load(f);
        self.root.definition.load(f);
        f.instruction(&Instruction::I64Sub);
        relative.store(f);
        self.root
            .state_for(relative, state, parent, workspace, builder, f);
        let node = RunLayout::read_state(definition, state, workspace, builder, f);
        node.consume(
            self.root,
            RunEntryConsumption::AssertionTruncate,
            workspace,
            builder,
            f,
        );
        node.release(builder, f);
        f.instruction(&Instruction::Br(0));
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        self.physical.publish_or_retire(self.root, f);
        for local in [older, index, parent, state, relative, definition, level] {
            schema.release_i64_local(local, f);
        }
    }
}

impl ChoiceEntry<'_> {
    fn publish_or_retire(&self, root: &RunLayout, f: &mut Function) {
        ChoiceStack::load(self.address, 64, f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::If(BlockType::Empty));
        self.discard_through(f);
        f.instruction(&Instruction::Else);
        self.offset.load(f);
        self.stack.workspace.choice_top.store(f);
        self.offset.load(f);
        root.length.load(f);
        f.instruction(&Instruction::I64Add);
        self.stack.workspace.choice_used.store(f);
        f.instruction(&Instruction::End);
    }

    pub(in crate::builtins::regexp::matcher_workspace::choice_entries) fn find_required_run(
        &self,
        builder: &mut FunctionBuilder<'_>,
        predicate: ChoiceSearchPredicate,
        found: I32Local,
        f: &mut Function,
        finish: &mut impl for<'entry> FnMut(
            LogicalChoiceEntry<'entry>,
            &mut FunctionBuilder<'_>,
            &mut Function,
        ) -> Result<(), EmitError>,
    ) -> Result<(), EmitError> {
        let workspace = self.stack.workspace;
        let schema = builder.runtime_schema();
        let root = RunLayout::read(self.address, workspace, builder, f);
        root.validate_live(workspace, builder, f);
        let path = PlaybackPath::new(&root, workspace, builder, f);
        let depth = schema.reserve_i64_local(f);
        let definition = schema.reserve_i64_local(f);
        let state = schema.reserve_i64_local(f);
        let context = schema.reserve_i64_local(f);
        let index = schema.reserve_i64_local(f);
        let older = schema.reserve_i64_local(f);
        let template = schema.reserve_i64_local(f);
        let kind = schema.reserve_i64_local(f);
        let relative = schema.reserve_i64_local(f);
        let parent = schema.reserve_i64_local(f);
        let descend = schema.reserve_i32_local(f);
        f.instruction(&Instruction::I64Const(0));
        depth.store(f);
        path.initialize(
            depth,
            self.address,
            self.address,
            self.address,
            workspace,
            builder,
            f,
        );
        f.instruction(&Instruction::I32Const(0));
        found.store(f);
        f.instruction(&Instruction::Block(BlockType::Empty));
        f.instruction(&Instruction::Loop(BlockType::Empty));
        f.instruction(&Instruction::I32Const(0));
        descend.store(f);
        path.read(depth, PathWord::Definition, definition, schema, f);
        path.read(depth, PathWord::State, state, schema, f);
        path.read(depth, PathWord::Context, context, schema, f);
        path.read(depth, PathWord::Index, index, schema, f);
        path.read(depth, PathWord::Older, older, schema, f);
        let node = RunLayout::read_state(definition, state, workspace, builder, f);
        node.check_state_shape(workspace, builder, f);
        node.remaining_used.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::I32Eqz);
        f.instruction(&Instruction::If(BlockType::Empty));
        node.template_address(index, workspace, template, f);
        ChoiceStack::load(template, 16, f);
        kind.store(f);
        kind.load(f);
        f.instruction(&Instruction::I64Const(ChoiceEntryKind::RequiredRun.word()));
        f.instruction(&Instruction::I64Eq);
        f.instruction(&Instruction::If(BlockType::Empty));
        older.load(f);
        f.instruction(&Instruction::I32WrapI64);
        f.instruction(&Instruction::If(BlockType::Empty));
        definition.load(f);
        context.store(f);
        f.instruction(&Instruction::End);
        let baseline = RunLayout::read(context, workspace, builder, f);
        template.load(f);
        context.load(f);
        f.instruction(&Instruction::I64Sub);
        relative.store(f);
        baseline.state_for(relative, state, parent, workspace, builder, f);
        baseline.release(builder, f);
        depth.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        depth.store(f);
        path.initialize(depth, template, state, context, workspace, builder, f);
        f.instruction(&Instruction::I32Const(1));
        descend.store(f);
        f.instruction(&Instruction::Else);
        node.selected_kind(template, index, older, kind, workspace, builder, f);
        predicate.emit(template, Some(kind), f);
        f.instruction(&Instruction::If(BlockType::Empty));
        finish(
            LogicalChoiceEntry::Run(LogicalRunEntry {
                physical: self,
                root: &root,
                path: &path,
                depth,
                template,
            }),
            builder,
            f,
        )?;
        f.instruction(&Instruction::I32Const(1));
        found.store(f);
        f.instruction(&Instruction::Br(4));
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        node.release(builder, f);
        descend.load(f);
        f.instruction(&Instruction::BrIf(0));
        // At most the current partial group and one older full group at each
        // source level. Advancing a preview never mutates retained playback.
        f.instruction(&Instruction::Block(BlockType::Empty));
        f.instruction(&Instruction::Loop(BlockType::Empty));
        path.read(depth, PathWord::Definition, definition, schema, f);
        path.read(depth, PathWord::State, state, schema, f);
        path.read(depth, PathWord::Index, index, schema, f);
        path.read(depth, PathWord::Older, older, schema, f);
        let node = RunLayout::read_state(definition, state, workspace, builder, f);
        index.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::I32Eqz);
        node.remaining_used.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::I32Eqz);
        f.instruction(&Instruction::I32And);
        f.instruction(&Instruction::If(BlockType::Empty));
        index.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Sub);
        index.store(f);
        path.write(depth, PathWord::Index, index, schema, f);
        f.instruction(&Instruction::Br(2));
        f.instruction(&Instruction::Else);
        older.load(f);
        f.instruction(&Instruction::I64Eqz);
        node.has_older_group(f);
        f.instruction(&Instruction::I32And);
        node.remaining_used.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::I32Eqz);
        f.instruction(&Instruction::I32And);
        f.instruction(&Instruction::If(BlockType::Empty));
        f.instruction(&Instruction::I64Const(1));
        older.store(f);
        node.count.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Sub);
        index.store(f);
        path.write(depth, PathWord::Older, older, schema, f);
        path.write(depth, PathWord::Index, index, schema, f);
        f.instruction(&Instruction::Br(3));
        f.instruction(&Instruction::Else);
        depth.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::BrIf(5));
        depth.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Sub);
        depth.store(f);
        f.instruction(&Instruction::Br(2));
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        node.release(builder, f);
        f.instruction(&Instruction::Br(0));
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::Br(0));
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        for local in [
            parent, relative, kind, template, older, index, context, state, definition, depth,
        ] {
            schema.release_i64_local(local, f);
        }
        schema.release_i32_local(descend, f);
        path.release(builder, f);
        root.release(builder, f);
        Ok(())
    }

    /// Nested exhaustion is propagated only after the selected continuation
    /// actually fails back here. A child's last fallback remains selected once.
    pub(in crate::builtins::regexp) fn restore_required_run(
        &self,
        builder: &FunctionBuilder<'_>,
        pc: I64Local,
        cursor: RegExpInputCursor,
        selected: I32Local,
        f: &mut Function,
    ) {
        let workspace = self.stack.workspace;
        let schema = builder.runtime_schema();
        let root = RunLayout::read(self.address, workspace, builder, f);
        root.validate_live(workspace, builder, f);
        let path = PlaybackPath::new(&root, workspace, builder, f);
        let depth = schema.reserve_i64_local(f);
        let definition = schema.reserve_i64_local(f);
        let state = schema.reserve_i64_local(f);
        let relative = schema.reserve_i64_local(f);
        let parent = schema.reserve_i64_local(f);
        let template = schema.reserve_i64_local(f);
        let kind = schema.reserve_i64_local(f);
        let zero = schema.reserve_i64_local(f);
        let index = schema.reserve_i64_local(f);
        f.instruction(&Instruction::I64Const(0));
        depth.store(f);
        f.instruction(&Instruction::I64Const(0));
        zero.store(f);
        path.initialize(
            depth,
            self.address,
            self.address,
            self.address,
            workspace,
            builder,
            f,
        );
        f.instruction(&Instruction::I32Const(0));
        selected.store(f);
        f.instruction(&Instruction::Block(BlockType::Empty));
        f.instruction(&Instruction::Loop(BlockType::Empty));
        path.read(depth, PathWord::Definition, definition, schema, f);
        definition.load(f);
        root.definition.load(f);
        f.instruction(&Instruction::I64Sub);
        relative.store(f);
        root.state_for(relative, state, parent, workspace, builder, f);
        let node = RunLayout::read_state(definition, state, workspace, builder, f);
        node.check_state_shape(workspace, builder, f);
        self.begin_backtracked_node(&node, relative, builder, selected, f);
        selected.load(f);
        f.instruction(&Instruction::If(BlockType::Empty));
        // Only an actually exhausted unobserved group can purge identical older
        // groups. No successful End continuation is inferred from byte equality.
        f.instruction(&Instruction::I64Const(0));
        node.remaining_used.store(f);
        node.remaining.load(f);
        f.instruction(&Instruction::I32WrapI64);
        f.instruction(&Instruction::I32Const(0));
        node.mincap.load(f);
        f.instruction(&Instruction::I64Const(4));
        f.instruction(&Instruction::I64Mul);
        f.instruction(&Instruction::I32WrapI64);
        f.instruction(&Instruction::MemoryFill(0));
        node.publish_used(f);
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::I32Const(0));
        selected.store(f);
        node.remaining_used.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::If(BlockType::Empty));
        depth.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::If(BlockType::Empty));
        self.discard_through(f);
        f.instruction(&Instruction::Br(3));
        f.instruction(&Instruction::End);
        depth.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Sub);
        depth.store(f);
        path.read(depth, PathWord::Definition, definition, schema, f);
        definition.load(f);
        root.definition.load(f);
        f.instruction(&Instruction::I64Sub);
        relative.store(f);
        root.state_for(relative, state, parent, workspace, builder, f);
        let owner = RunLayout::read_state(definition, state, workspace, builder, f);
        owner.consume(
            &root,
            RunEntryConsumption::FailedBacktrack,
            workspace,
            builder,
            f,
        );
        owner.release(builder, f);
        f.instruction(&Instruction::Br(1));
        f.instruction(&Instruction::Else);
        node.template_address(node.index, workspace, template, f);
        ChoiceStack::load(template, 16, f);
        kind.store(f);
        kind.load(f);
        f.instruction(&Instruction::I64Const(ChoiceEntryKind::RequiredRun.word()));
        f.instruction(&Instruction::I64Eq);
        f.instruction(&Instruction::If(BlockType::Empty));
        template.load(f);
        root.definition.load(f);
        f.instruction(&Instruction::I64Sub);
        relative.store(f);
        root.state_for(relative, state, parent, workspace, builder, f);
        depth.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        depth.store(f);
        path.initialize(depth, template, state, self.address, workspace, builder, f);
        f.instruction(&Instruction::Br(2));
        f.instruction(&Instruction::Else);
        node.index.load(f);
        index.store(f);
        path.write(depth, PathWord::Index, index, schema, f);
        node.selected_kind(template, index, zero, kind, workspace, builder, f);
        let logical = LogicalRunEntry {
            physical: self,
            root: &root,
            path: &path,
            depth,
            template,
        };
        kind.load(f);
        f.instruction(&Instruction::I64Const(
            ChoiceEntryKind::LazyProgressAttempt.word(),
        ));
        f.instruction(&Instruction::I64Eq);
        f.instruction(&Instruction::If(BlockType::Empty));
        node.consume(
            &root,
            RunEntryConsumption::FailedBacktrack,
            workspace,
            builder,
            f,
        );
        f.instruction(&Instruction::Else);
        ChoiceStack::load(template, 24, f);
        pc.store(f);
        logical.restore_cursor(cursor, f);
        logical.restore_state(builder, f);
        kind.load(f);
        f.instruction(&Instruction::I64Const(
            ChoiceEntryKind::LazyProgressChoice.word(),
        ));
        f.instruction(&Instruction::I64Eq);
        f.instruction(&Instruction::If(BlockType::Empty));
        f.instruction(&Instruction::I64Const(
            ChoiceEntryKind::LazyProgressAttempt.word(),
        ));
        node.current_kind_override.store(f);
        node.publish_cursor(f);
        f.instruction(&Instruction::Else);
        node.consume(
            &root,
            RunEntryConsumption::FailedBacktrack,
            workspace,
            builder,
            f,
        );
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::I32Const(1));
        selected.store(f);
        f.instruction(&Instruction::End);
        self.publish_or_retire(&root, f);
        f.instruction(&Instruction::Br(3));
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        node.release(builder, f);
        f.instruction(&Instruction::Br(0));
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        for local in [
            index, zero, kind, template, parent, relative, state, definition, depth,
        ] {
            schema.release_i64_local(local, f);
        }
        path.release(builder, f);
        root.release(builder, f);
    }
}
