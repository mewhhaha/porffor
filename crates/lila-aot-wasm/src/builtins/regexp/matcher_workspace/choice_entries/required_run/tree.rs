//! The immutable tree keeps each original reset domain. Its separate table
//! contains only node metadata and natural limbs, never a second saved slab.
use super::*;

const TABLE_HEADER: u64 = 24;

pub(in crate::builtins::regexp) struct CheckedTemplateRunNode<'node> {
    workspace: &'node MatcherWorkspace,
    definition: I64Local,
}

impl CheckedTemplateRunNode<'_> {
    pub(in crate::builtins::regexp) fn row_offset(&self, out: I64Local, f: &mut Function) {
        self.load(40, out, f);
    }
    pub(in crate::builtins::regexp) fn minimum_capacity(&self, out: I64Local, f: &mut Function) {
        self.load(48, out, f);
    }
    pub(in crate::builtins::regexp) fn maximum_capacity(&self, out: I64Local, f: &mut Function) {
        self.load(56, out, f);
    }
    pub(in crate::builtins::regexp) fn maximum_kind(&self, out: I64Local, f: &mut Function) {
        self.load(88, out, f);
    }
    fn load(&self, offset: u64, out: I64Local, f: &mut Function) {
        // This address is admitted by the original workspace's bounded walk.
        ChoiceStack::load(self.definition, offset, f);
        out.store(f);
    }
}

fn copy_bytes(destination: I64Local, source: I64Local, bytes: I64Local, f: &mut Function) {
    destination.load(f);
    f.instruction(&Instruction::I32WrapI64);
    source.load(f);
    f.instruction(&Instruction::I32WrapI64);
    bytes.load(f);
    f.instruction(&Instruction::I32WrapI64);
    f.instruction(&Instruction::MemoryCopy {
        src_mem: 0,
        dst_mem: 0,
    });
}

fn taint(state: I64Local, f: &mut Function) {
    for (offset, value) in [(104, 1), (112, 0)] {
        state.load(f);
        f.instruction(&Instruction::I32WrapI64);
        f.instruction(&Instruction::I64Const(value));
        f.instruction(&Instruction::I64Store(FunctionBuilder::memarg8(offset)));
    }
}

fn address_at(base: I64Local, relative: I64Local, out: I64Local, f: &mut Function) {
    base.load(f);
    relative.load(f);
    f.instruction(&Instruction::I64Add);
    out.store(f);
}

fn add_const(base: I64Local, amount: u64, out: I64Local, f: &mut Function) {
    base.load(f);
    f.instruction(&Instruction::I64Const(amount as i64));
    f.instruction(&Instruction::I64Add);
    out.store(f);
}

impl RunLayout {
    /// All table identities are relative to this containing immutable record.
    /// No old absolute arena address survives copying an embedded child.
    pub(super) fn state_for(
        &self,
        relative: I64Local,
        state: I64Local,
        parent: I64Local,
        workspace: &MatcherWorkspace,
        builder: &FunctionBuilder<'_>,
        f: &mut Function,
    ) {
        relative.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::If(BlockType::Empty));
        self.address.load(f);
        state.store(f);
        f.instruction(&Instruction::I64Const(EMPTY_LINK));
        parent.store(f);
        f.instruction(&Instruction::Else);
        let schema = builder.runtime_schema();
        let cursor = schema.reserve_i64_local(f);
        let end = schema.reserve_i64_local(f);
        let length = schema.reserve_i64_local(f);
        self.table.load(f);
        cursor.store(f);
        self.table.load(f);
        self.table_bytes.load(f);
        f.instruction(&Instruction::I64Add);
        end.store(f);
        f.instruction(&Instruction::Block(BlockType::Empty));
        f.instruction(&Instruction::Loop(BlockType::Empty));
        cursor.load(f);
        end.load(f);
        f.instruction(&Instruction::I64GeU);
        reject(workspace, builder, f);
        ChoiceStack::load(cursor, 0, f);
        length.store(f);
        length.load(f);
        f.instruction(&Instruction::I64Const((TABLE_HEADER + HEADER_BYTES) as i64));
        f.instruction(&Instruction::I64LtU);
        length.load(f);
        end.load(f);
        cursor.load(f);
        f.instruction(&Instruction::I64Sub);
        f.instruction(&Instruction::I64GtU);
        f.instruction(&Instruction::I32Or);
        reject(workspace, builder, f);
        ChoiceStack::load(cursor, 8, f);
        relative.load(f);
        f.instruction(&Instruction::I64Eq);
        f.instruction(&Instruction::If(BlockType::Empty));
        add_const(cursor, TABLE_HEADER, state, f);
        ChoiceStack::load(cursor, 16, f);
        parent.store(f);
        f.instruction(&Instruction::Br(2));
        f.instruction(&Instruction::End);
        cursor.load(f);
        length.load(f);
        f.instruction(&Instruction::I64Add);
        cursor.store(f);
        f.instruction(&Instruction::Br(0));
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        for local in [length, end, cursor] {
            schema.release_i64_local(local, f);
        }
        f.instruction(&Instruction::End);
    }

    pub(super) fn template_address(
        &self,
        index: I64Local,
        _workspace: &MatcherWorkspace,
        out: I64Local,
        f: &mut Function,
    ) {
        self.offsets.load(f);
        index.load(f);
        f.instruction(&Instruction::I64Const(8));
        f.instruction(&Instruction::I64Mul);
        f.instruction(&Instruction::I64Add);
        f.instruction(&Instruction::I32WrapI64);
        f.instruction(&Instruction::I64Load(FunctionBuilder::memarg8(0)));
        self.definition.load(f);
        f.instruction(&Instruction::I64Add);
        out.store(f);
    }

    fn check_children(
        &self,
        workspace: &MatcherWorkspace,
        builder: &FunctionBuilder<'_>,
        f: &mut Function,
    ) {
        let schema = builder.runtime_schema();
        let index = schema.reserve_i64_local(f);
        let cursor = schema.reserve_i64_local(f);
        let actual = schema.reserve_i64_local(f);
        let end = schema.reserve_i64_local(f);
        let length = schema.reserve_i64_local(f);
        let descendants = schema.reserve_i64_local(f);
        f.instruction(&Instruction::I64Const(0));
        index.store(f);
        self.template.load(f);
        cursor.store(f);
        self.template.load(f);
        self.template_bytes.load(f);
        f.instruction(&Instruction::I64Add);
        end.store(f);
        f.instruction(&Instruction::I64Const(0));
        descendants.store(f);
        f.instruction(&Instruction::Block(BlockType::Empty));
        f.instruction(&Instruction::Loop(BlockType::Empty));
        index.load(f);
        self.count.load(f);
        f.instruction(&Instruction::I64GeU);
        f.instruction(&Instruction::BrIf(1));
        self.template_address(index, workspace, actual, f);
        actual.load(f);
        cursor.load(f);
        f.instruction(&Instruction::I64Ne);
        reject(workspace, builder, f);
        end.load(f);
        cursor.load(f);
        f.instruction(&Instruction::I64Sub);
        f.instruction(&Instruction::I64Const(24));
        f.instruction(&Instruction::I64LtU);
        reject(workspace, builder, f);
        ChoiceStack::load(cursor, 0, f);
        length.store(f);
        length.load(f);
        f.instruction(&Instruction::I64Const(24));
        f.instruction(&Instruction::I64LtU);
        length.load(f);
        end.load(f);
        cursor.load(f);
        f.instruction(&Instruction::I64Sub);
        f.instruction(&Instruction::I64GtU);
        f.instruction(&Instruction::I32Or);
        length.load(f);
        f.instruction(&Instruction::I64Const(7));
        f.instruction(&Instruction::I64And);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::I32Eqz);
        f.instruction(&Instruction::I32Or);
        ChoiceStack::load(cursor, 16, f);
        f.instruction(&Instruction::I64Const(ChoiceEntryKind::RequiredRun.word()));
        f.instruction(&Instruction::I64GtU);
        f.instruction(&Instruction::I32Or);
        reject(workspace, builder, f);
        ChoiceStack::load(cursor, 16, f);
        f.instruction(&Instruction::I64Const(ChoiceEntryKind::RequiredRun.word()));
        f.instruction(&Instruction::I64Eq);
        f.instruction(&Instruction::If(BlockType::Empty));
        length.load(f);
        f.instruction(&Instruction::I64Const(HEADER_BYTES as i64));
        f.instruction(&Instruction::I64LtU);
        reject(workspace, builder, f);
        descendants.load(f);
        ChoiceStack::load(cursor, 128, f);
        f.instruction(&Instruction::I64Add);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        descendants.store(f);
        descendants.load(f);
        workspace.maximum_capacity.load(f);
        f.instruction(&Instruction::I64GtU);
        reject(workspace, builder, f);
        f.instruction(&Instruction::Else);
        length.load(f);
        workspace.snapshot_bytes.load(f);
        f.instruction(&Instruction::I64Ne);
        reject(workspace, builder, f);
        f.instruction(&Instruction::End);
        cursor.load(f);
        length.load(f);
        f.instruction(&Instruction::I64Add);
        cursor.store(f);
        index.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        index.store(f);
        f.instruction(&Instruction::Br(0));
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        cursor.load(f);
        end.load(f);
        f.instruction(&Instruction::I64Ne);
        reject(workspace, builder, f);
        descendants.load(f);
        self.node_count.load(f);
        f.instruction(&Instruction::I64Ne);
        reject(workspace, builder, f);
        for local in [descendants, length, end, actual, cursor, index] {
            schema.release_i64_local(local, f);
        }
    }

    pub(super) fn check_state_shape(
        &self,
        workspace: &MatcherWorkspace,
        builder: &FunctionBuilder<'_>,
        f: &mut Function,
    ) {
        self.validate(workspace, builder, f);
        self.remaining().validate(workspace, builder, f);
        self.affine().validate(workspace, builder, f);
        self.base().validate(workspace, builder, f);
        for (offset, local) in [
            (24, self.count),
            (40, self.row),
            (48, self.mincap),
            (56, self.maxcap),
            (88, self.maximum_kind),
            (120, self.template_bytes),
            (128, self.node_count),
            (136, self.table_bytes),
        ] {
            ChoiceStack::load(self.definition, offset, f);
            local.load(f);
            f.instruction(&Instruction::I64Ne);
            reject(workspace, builder, f);
        }
        self.check_children(workspace, builder, f);
    }

    pub(super) fn check_table(
        &self,
        workspace: &MatcherWorkspace,
        builder: &FunctionBuilder<'_>,
        f: &mut Function,
    ) {
        let schema = builder.runtime_schema();
        let cursor = schema.reserve_i64_local(f);
        let end = schema.reserve_i64_local(f);
        let count = schema.reserve_i64_local(f);
        let relative = schema.reserve_i64_local(f);
        let definition = schema.reserve_i64_local(f);
        let state = schema.reserve_i64_local(f);
        let length = schema.reserve_i64_local(f);
        let parent = schema.reserve_i64_local(f);
        let last = schema.reserve_i64_local(f);
        self.table.load(f);
        cursor.store(f);
        self.table.load(f);
        self.table_bytes.load(f);
        f.instruction(&Instruction::I64Add);
        end.store(f);
        f.instruction(&Instruction::I64Const(0));
        count.store(f);
        f.instruction(&Instruction::I64Const(0));
        last.store(f);
        f.instruction(&Instruction::Block(BlockType::Empty));
        f.instruction(&Instruction::Loop(BlockType::Empty));
        cursor.load(f);
        end.load(f);
        f.instruction(&Instruction::I64Eq);
        f.instruction(&Instruction::BrIf(1));
        cursor.load(f);
        end.load(f);
        f.instruction(&Instruction::I64GtU);
        end.load(f);
        cursor.load(f);
        f.instruction(&Instruction::I64Sub);
        f.instruction(&Instruction::I64Const((TABLE_HEADER + HEADER_BYTES) as i64));
        f.instruction(&Instruction::I64LtU);
        f.instruction(&Instruction::I32Or);
        reject(workspace, builder, f);
        ChoiceStack::load(cursor, 0, f);
        length.store(f);
        ChoiceStack::load(cursor, 8, f);
        relative.store(f);
        ChoiceStack::load(cursor, 16, f);
        parent.store(f);
        relative.load(f);
        last.load(f);
        f.instruction(&Instruction::I64LeU);
        relative.load(f);
        self.template.load(f);
        self.definition.load(f);
        f.instruction(&Instruction::I64Sub);
        f.instruction(&Instruction::I64LtU);
        f.instruction(&Instruction::I32Or);
        relative.load(f);
        self.table.load(f);
        self.definition.load(f);
        f.instruction(&Instruction::I64Sub);
        f.instruction(&Instruction::I64GeU);
        f.instruction(&Instruction::I32Or);
        self.table.load(f);
        self.definition.load(f);
        f.instruction(&Instruction::I64Sub);
        relative.load(f);
        f.instruction(&Instruction::I64Sub);
        f.instruction(&Instruction::I64Const(HEADER_BYTES as i64));
        f.instruction(&Instruction::I64LtU);
        f.instruction(&Instruction::I32Or);
        parent.load(f);
        relative.load(f);
        f.instruction(&Instruction::I64GeU);
        f.instruction(&Instruction::I32Or);
        length.load(f);
        end.load(f);
        cursor.load(f);
        f.instruction(&Instruction::I64Sub);
        f.instruction(&Instruction::I64GtU);
        f.instruction(&Instruction::I32Or);
        reject(workspace, builder, f);
        address_at(self.definition, relative, definition, f);
        add_const(cursor, TABLE_HEADER, state, f);
        ChoiceStack::load(definition, 16, f);
        f.instruction(&Instruction::I64Const(ChoiceEntryKind::RequiredRun.word()));
        f.instruction(&Instruction::I64Ne);
        reject(workspace, builder, f);
        let child = RunLayout::read_state(definition, state, workspace, builder, f);
        child.check_state_shape(workspace, builder, f);
        length.load(f);
        child.prefix_bytes.load(f);
        f.instruction(&Instruction::I64Const(TABLE_HEADER as i64));
        f.instruction(&Instruction::I64Add);
        f.instruction(&Instruction::I64Ne);
        reject(workspace, builder, f);
        child.release(builder, f);
        self.check_direct_parent(definition, parent, workspace, builder, f);
        relative.load(f);
        last.store(f);
        cursor.load(f);
        length.load(f);
        f.instruction(&Instruction::I64Add);
        cursor.store(f);
        count.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        count.store(f);
        count.load(f);
        self.node_count.load(f);
        f.instruction(&Instruction::I64GtU);
        reject(workspace, builder, f);
        f.instruction(&Instruction::Br(0));
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        count.load(f);
        self.node_count.load(f);
        f.instruction(&Instruction::I64Ne);
        reject(workspace, builder, f);
        for local in [
            last, parent, length, state, definition, relative, count, end, cursor,
        ] {
            schema.release_i64_local(local, f);
        }
    }

    fn check_direct_parent(
        &self,
        child: I64Local,
        parent: I64Local,
        workspace: &MatcherWorkspace,
        builder: &FunctionBuilder<'_>,
        f: &mut Function,
    ) {
        let schema = builder.runtime_schema();
        let definition = schema.reserve_i64_local(f);
        let state = schema.reserve_i64_local(f);
        let grandparent = schema.reserve_i64_local(f);
        let index = schema.reserve_i64_local(f);
        let address = schema.reserve_i64_local(f);
        let found = schema.reserve_i32_local(f);
        self.state_for(parent, state, grandparent, workspace, builder, f);
        address_at(self.definition, parent, definition, f);
        ChoiceStack::load(definition, 16, f);
        f.instruction(&Instruction::I64Const(ChoiceEntryKind::RequiredRun.word()));
        f.instruction(&Instruction::I64Ne);
        reject(workspace, builder, f);
        let owner = RunLayout::read_state(definition, state, workspace, builder, f);
        owner.check_state_shape(workspace, builder, f);
        f.instruction(&Instruction::I64Const(0));
        index.store(f);
        f.instruction(&Instruction::I32Const(0));
        found.store(f);
        f.instruction(&Instruction::Block(BlockType::Empty));
        f.instruction(&Instruction::Loop(BlockType::Empty));
        index.load(f);
        owner.count.load(f);
        f.instruction(&Instruction::I64GeU);
        f.instruction(&Instruction::BrIf(1));
        owner.template_address(index, workspace, address, f);
        address.load(f);
        child.load(f);
        f.instruction(&Instruction::I64Eq);
        found.load(f);
        f.instruction(&Instruction::I32Or);
        found.store(f);
        index.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        index.store(f);
        f.instruction(&Instruction::Br(0));
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        found.load(f);
        f.instruction(&Instruction::I32Eqz);
        reject(workspace, builder, f);
        owner.release(builder, f);
        schema.release_i32_local(found, f);
        for local in [address, index, grandparent, state, definition] {
            schema.release_i64_local(local, f);
        }
    }

    fn check_distinct_ancestry(
        &self,
        row: I64Local,
        parent: I64Local,
        workspace: &MatcherWorkspace,
        builder: &FunctionBuilder<'_>,
        f: &mut Function,
    ) {
        let schema = builder.runtime_schema();
        let relative = schema.reserve_i64_local(f);
        let state = schema.reserve_i64_local(f);
        let next = schema.reserve_i64_local(f);
        let hops = schema.reserve_i64_local(f);
        parent.load(f);
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
        self.node_count.load(f);
        f.instruction(&Instruction::I64GtU);
        reject(workspace, builder, f);
        self.state_for(relative, state, next, workspace, builder, f);
        ChoiceStack::load(state, 40, f);
        row.load(f);
        f.instruction(&Instruction::I64Eq);
        reject(workspace, builder, f);
        next.load(f);
        relative.store(f);
        hops.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        hops.store(f);
        f.instruction(&Instruction::Br(0));
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        for local in [hops, next, state, relative] {
            schema.release_i64_local(local, f);
        }
    }

    /// Iterative DFS uses the actual parent IDs in the bounded table. Each
    /// immutable leaf is visited once; multiplicities are never unfolded.
    fn walk(
        &self,
        workspace: &MatcherWorkspace,
        builder: &mut FunctionBuilder<'_>,
        f: &mut Function,
        mut leaf: impl for<'leaf> FnMut(
            ChoiceSnapshotView<'leaf>,
            &mut FunctionBuilder<'_>,
            &mut Function,
        ) -> Result<(), EmitError>,
        mut node: impl for<'node> FnMut(
            CheckedTemplateRunNode<'node>,
            I64Local,
            &mut FunctionBuilder<'_>,
            &mut Function,
        ) -> Result<(), EmitError>,
    ) -> Result<(), EmitError> {
        let schema = builder.runtime_schema();
        let current = schema.reserve_i64_local(f);
        let container = schema.reserve_i64_local(f);
        let relative = schema.reserve_i64_local(f);
        let state = schema.reserve_i64_local(f);
        let parent = schema.reserve_i64_local(f);
        let expected_parent = schema.reserve_i64_local(f);
        let seen = schema.reserve_i64_local(f);
        self.check_table(workspace, builder, f);
        self.definition.load(f);
        current.store(f);
        f.instruction(&Instruction::I64Const(EMPTY_LINK));
        container.store(f);
        f.instruction(&Instruction::I64Const(0));
        seen.store(f);
        f.instruction(&Instruction::Block(BlockType::Empty));
        f.instruction(&Instruction::Loop(BlockType::Empty));
        ChoiceStack::load(current, 16, f);
        f.instruction(&Instruction::I64Const(ChoiceEntryKind::RequiredRun.word()));
        f.instruction(&Instruction::I64Eq);
        f.instruction(&Instruction::If(BlockType::Empty));
        current.load(f);
        self.definition.load(f);
        f.instruction(&Instruction::I64Sub);
        relative.store(f);
        self.state_for(relative, state, parent, workspace, builder, f);
        parent.load(f);
        container.load(f);
        f.instruction(&Instruction::I64Ne);
        reject(workspace, builder, f);
        let child = RunLayout::read_state(current, state, workspace, builder, f);
        child.check_state_shape(workspace, builder, f);
        let original = RunLayout::read(current, workspace, builder, f);
        original.check_table(workspace, builder, f);
        original.release(builder, f);
        self.check_distinct_ancestry(child.row, parent, workspace, builder, f);
        node(
            CheckedTemplateRunNode {
                workspace,
                definition: current,
            },
            state,
            builder,
            f,
        )?;
        relative.load(f);
        container.store(f);
        child.template.load(f);
        current.store(f);
        child.release(builder, f);
        seen.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        seen.store(f);
        f.instruction(&Instruction::Else);
        ChoiceStack::load(current, 0, f);
        workspace.snapshot_bytes.load(f);
        f.instruction(&Instruction::I64Ne);
        reject(workspace, builder, f);
        leaf(
            ChoiceSnapshotView {
                workspace,
                address: current,
            },
            builder,
            f,
        )?;
        current.load(f);
        workspace.snapshot_bytes.load(f);
        f.instruction(&Instruction::I64Add);
        current.store(f);
        // Ascend after the last direct child, skipping the serialized table.
        f.instruction(&Instruction::Block(BlockType::Empty));
        f.instruction(&Instruction::Loop(BlockType::Empty));
        address_at(self.definition, container, relative, f);
        self.state_for(container, state, parent, workspace, builder, f);
        let owner = RunLayout::read_state(relative, state, workspace, builder, f);
        current.load(f);
        owner.table.load(f);
        f.instruction(&Instruction::I64GtU);
        reject(workspace, builder, f);
        current.load(f);
        owner.table.load(f);
        f.instruction(&Instruction::I64Ne);
        f.instruction(&Instruction::BrIf(1));
        container.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::BrIf(4));
        owner.definition.load(f);
        owner.length.load(f);
        f.instruction(&Instruction::I64Add);
        current.store(f);
        parent.load(f);
        expected_parent.store(f);
        expected_parent.load(f);
        container.store(f);
        owner.release(builder, f);
        f.instruction(&Instruction::Br(0));
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        // Locals belonging to the compile-time owner above have already been
        // released; all runtime paths retain only the bounded walk locals.
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::Br(0));
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        seen.load(f);
        self.node_count.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        f.instruction(&Instruction::I64Ne);
        reject(workspace, builder, f);
        for local in [
            seen,
            expected_parent,
            parent,
            state,
            relative,
            container,
            current,
        ] {
            schema.release_i64_local(local, f);
        }
        Ok(())
    }
}

impl RunLayout {
    /// Measure only retained physical nodes. The exact multiplicity is never
    /// used as an allocation size or a traversal bound.
    pub(super) fn measure_template(
        &self,
        source: I64Local,
        workspace: &MatcherWorkspace,
        builder: &FunctionBuilder<'_>,
        f: &mut Function,
    ) {
        let schema = builder.runtime_schema();
        let cursor = schema.reserve_i64_local(f);
        let end = schema.reserve_i64_local(f);
        source.load(f);
        cursor.store(f);
        source.load(f);
        self.template_bytes.load(f);
        f.instruction(&Instruction::I64Add);
        end.store(f);
        for out in [self.node_count, self.table_bytes] {
            f.instruction(&Instruction::I64Const(0));
            out.store(f);
        }
        f.instruction(&Instruction::Block(BlockType::Empty));
        f.instruction(&Instruction::Loop(BlockType::Empty));
        cursor.load(f);
        end.load(f);
        f.instruction(&Instruction::I64GeU);
        f.instruction(&Instruction::BrIf(1));
        ChoiceStack::load(cursor, 16, f);
        f.instruction(&Instruction::I64Const(ChoiceEntryKind::RequiredRun.word()));
        f.instruction(&Instruction::I64Eq);
        f.instruction(&Instruction::If(BlockType::Empty));
        let child = RunLayout::read(cursor, workspace, builder, f);
        child.check_state_shape(workspace, builder, f);
        child.check_table(workspace, builder, f);
        self.node_count.load(f);
        child.node_count.load(f);
        f.instruction(&Instruction::I64Add);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        self.node_count.store(f);
        self.table_bytes.load(f);
        child.table_bytes.load(f);
        f.instruction(&Instruction::I64Add);
        child.prefix_bytes.load(f);
        f.instruction(&Instruction::I64Add);
        f.instruction(&Instruction::I64Const(TABLE_HEADER as i64));
        f.instruction(&Instruction::I64Add);
        self.table_bytes.store(f);
        child.release(builder, f);
        f.instruction(&Instruction::End);
        cursor.load(f);
        ChoiceStack::load(cursor, 0, f);
        f.instruction(&Instruction::I64Add);
        cursor.store(f);
        f.instruction(&Instruction::Br(0));
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        cursor.load(f);
        end.load(f);
        f.instruction(&Instruction::I64Ne);
        reject(workspace, builder, f);
        for value in [self.node_count, self.table_bytes] {
            value.load(f);
            workspace.maximum_capacity.load(f);
            f.instruction(&Instruction::I64GtU);
            reject(workspace, builder, f);
        }
        schema.release_i64_local(end, f);
        schema.release_i64_local(cursor, f);
    }

    pub(super) fn initialize_tree(
        &self,
        workspace: &MatcherWorkspace,
        builder: &FunctionBuilder<'_>,
        f: &mut Function,
    ) {
        let schema = builder.runtime_schema();
        let index = schema.reserve_i64_local(f);
        let cursor = schema.reserve_i64_local(f);
        let output = schema.reserve_i64_local(f);
        let relative = schema.reserve_i64_local(f);
        let entry_length = schema.reserve_i64_local(f);
        let state = schema.reserve_i64_local(f);
        let source = schema.reserve_i64_local(f);
        let end = schema.reserve_i64_local(f);
        let parent = schema.reserve_i64_local(f);
        f.instruction(&Instruction::I64Const(0));
        index.store(f);
        self.template.load(f);
        cursor.store(f);
        self.table.load(f);
        output.store(f);
        f.instruction(&Instruction::Block(BlockType::Empty));
        f.instruction(&Instruction::Loop(BlockType::Empty));
        index.load(f);
        self.count.load(f);
        f.instruction(&Instruction::I64GeU);
        f.instruction(&Instruction::BrIf(1));
        cursor.load(f);
        self.definition.load(f);
        f.instruction(&Instruction::I64Sub);
        relative.store(f);
        self.offsets.load(f);
        index.load(f);
        f.instruction(&Instruction::I64Const(8));
        f.instruction(&Instruction::I64Mul);
        f.instruction(&Instruction::I64Add);
        f.instruction(&Instruction::I32WrapI64);
        relative.load(f);
        f.instruction(&Instruction::I64Store(FunctionBuilder::memarg8(0)));
        ChoiceStack::load(cursor, 16, f);
        f.instruction(&Instruction::I64Const(ChoiceEntryKind::RequiredRun.word()));
        f.instruction(&Instruction::I64Eq);
        f.instruction(&Instruction::If(BlockType::Empty));
        let child = RunLayout::read(cursor, workspace, builder, f);
        child.prefix_bytes.load(f);
        f.instruction(&Instruction::I64Const(TABLE_HEADER as i64));
        f.instruction(&Instruction::I64Add);
        entry_length.store(f);
        ChoiceStack::store(output, 0, entry_length, f);
        ChoiceStack::store(output, 8, relative, f);
        f.instruction(&Instruction::I64Const(0));
        parent.store(f);
        ChoiceStack::store(output, 16, parent, f);
        add_const(output, TABLE_HEADER, state, f);
        copy_bytes(state, cursor, child.prefix_bytes, f);
        taint(state, f);
        output.load(f);
        entry_length.load(f);
        f.instruction(&Instruction::I64Add);
        output.store(f);
        copy_bytes(output, child.table, child.table_bytes, f);
        output.load(f);
        child.table_bytes.load(f);
        f.instruction(&Instruction::I64Add);
        end.store(f);
        f.instruction(&Instruction::Block(BlockType::Empty));
        f.instruction(&Instruction::Loop(BlockType::Empty));
        output.load(f);
        end.load(f);
        f.instruction(&Instruction::I64GeU);
        f.instruction(&Instruction::BrIf(1));
        // Rebase both identities, never an old absolute arena offset.
        for offset in [8, 16] {
            ChoiceStack::load(output, offset, f);
            relative.load(f);
            f.instruction(&Instruction::I64Add);
            source.store(f);
            ChoiceStack::store(output, offset, source, f);
        }
        add_const(output, TABLE_HEADER, state, f);
        taint(state, f);
        output.load(f);
        ChoiceStack::load(output, 0, f);
        f.instruction(&Instruction::I64Add);
        output.store(f);
        f.instruction(&Instruction::Br(0));
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        child.release(builder, f);
        f.instruction(&Instruction::End);
        cursor.load(f);
        ChoiceStack::load(cursor, 0, f);
        f.instruction(&Instruction::I64Add);
        cursor.store(f);
        index.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        index.store(f);
        f.instruction(&Instruction::Br(0));
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        output.load(f);
        self.table.load(f);
        self.table_bytes.load(f);
        f.instruction(&Instruction::I64Add);
        f.instruction(&Instruction::I64Ne);
        reject(workspace, builder, f);
        for local in [
            parent,
            end,
            source,
            state,
            entry_length,
            relative,
            output,
            cursor,
            index,
        ] {
            schema.release_i64_local(local, f);
        }
    }

    /// Reset descendants from this node's ORIGINAL template. A parent's reset
    /// instead copies this record's captured CURRENT table through this same
    /// direct-child operation; neither reset overwrites an immutable baseline.
    pub(super) fn reset_children(
        &self,
        root: &RunLayout,
        workspace: &MatcherWorkspace,
        builder: &FunctionBuilder<'_>,
        f: &mut Function,
    ) {
        let schema = builder.runtime_schema();
        let index = schema.reserve_i64_local(f);
        let child_address = schema.reserve_i64_local(f);
        let relative = schema.reserve_i64_local(f);
        let state = schema.reserve_i64_local(f);
        let parent = schema.reserve_i64_local(f);
        let cursor = schema.reserve_i64_local(f);
        let end = schema.reserve_i64_local(f);
        let source = schema.reserve_i64_local(f);
        let target_relative = schema.reserve_i64_local(f);
        f.instruction(&Instruction::I64Const(0));
        index.store(f);
        f.instruction(&Instruction::Block(BlockType::Empty));
        f.instruction(&Instruction::Loop(BlockType::Empty));
        index.load(f);
        self.count.load(f);
        f.instruction(&Instruction::I64GeU);
        f.instruction(&Instruction::BrIf(1));
        self.template_address(index, workspace, child_address, f);
        ChoiceStack::load(child_address, 16, f);
        f.instruction(&Instruction::I64Const(ChoiceEntryKind::RequiredRun.word()));
        f.instruction(&Instruction::I64Eq);
        f.instruction(&Instruction::If(BlockType::Empty));
        child_address.load(f);
        root.definition.load(f);
        f.instruction(&Instruction::I64Sub);
        relative.store(f);
        root.state_for(relative, state, parent, workspace, builder, f);
        let child = RunLayout::read(child_address, workspace, builder, f);
        copy_bytes(state, child_address, child.prefix_bytes, f);
        taint(state, f);
        child.table.load(f);
        cursor.store(f);
        child.table.load(f);
        child.table_bytes.load(f);
        f.instruction(&Instruction::I64Add);
        end.store(f);
        f.instruction(&Instruction::Block(BlockType::Empty));
        f.instruction(&Instruction::Loop(BlockType::Empty));
        cursor.load(f);
        end.load(f);
        f.instruction(&Instruction::I64GeU);
        f.instruction(&Instruction::BrIf(1));
        ChoiceStack::load(cursor, 8, f);
        relative.load(f);
        f.instruction(&Instruction::I64Add);
        target_relative.store(f);
        root.state_for(target_relative, state, parent, workspace, builder, f);
        add_const(cursor, TABLE_HEADER, source, f);
        ChoiceStack::load(cursor, 0, f);
        f.instruction(&Instruction::I64Const(TABLE_HEADER as i64));
        f.instruction(&Instruction::I64Sub);
        target_relative.store(f);
        copy_bytes(state, source, target_relative, f);
        taint(state, f);
        cursor.load(f);
        ChoiceStack::load(cursor, 0, f);
        f.instruction(&Instruction::I64Add);
        cursor.store(f);
        f.instruction(&Instruction::Br(0));
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        child.release(builder, f);
        f.instruction(&Instruction::End);
        index.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        index.store(f);
        f.instruction(&Instruction::Br(0));
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        for local in [
            target_relative,
            source,
            end,
            cursor,
            parent,
            state,
            relative,
            child_address,
            index,
        ] {
            schema.release_i64_local(local, f);
        }
    }
}

impl ChoiceEntry<'_> {
    pub(in crate::builtins::regexp) fn with_template_snapshots(
        &self,
        builder: &mut FunctionBuilder<'_>,
        f: &mut Function,
        mut leaf: impl for<'leaf> FnMut(
            ChoiceSnapshotView<'leaf>,
            &mut FunctionBuilder<'_>,
            &mut Function,
        ) -> Result<(), EmitError>,
        mut node: impl for<'node> FnMut(
            CheckedTemplateRunNode<'node>,
            &mut FunctionBuilder<'_>,
            &mut Function,
        ) -> Result<(), EmitError>,
    ) -> Result<(), EmitError> {
        self.is_snapshot(f);
        f.instruction(&Instruction::If(BlockType::Empty));
        leaf(
            ChoiceSnapshotView {
                workspace: self.stack.workspace,
                address: self.address,
            },
            builder,
            f,
        )?;
        f.instruction(&Instruction::Else);
        let layout = RunLayout::read(self.address, self.stack.workspace, builder, f);
        layout.walk(
            self.stack.workspace,
            builder,
            f,
            leaf,
            |view, _state, builder, f| node(view, builder, f),
        )?;
        layout.release(builder, f);
        f.instruction(&Instruction::End);
        Ok(())
    }

    pub(in crate::builtins::regexp) fn compare_template_tree(
        &self,
        other: &ChoiceEntry<'_>,
        builder: &mut FunctionBuilder<'_>,
        eligible: I32Local,
        f: &mut Function,
        mut leaf_pair: impl for<'left, 'right> FnMut(
            ChoiceSnapshotView<'left>,
            ChoiceSnapshotView<'right>,
            &mut FunctionBuilder<'_>,
            &mut Function,
        ) -> Result<(), EmitError>,
    ) -> Result<(), EmitError> {
        assert!(
            core::ptr::eq(self.stack.workspace, other.stack.workspace),
            "paired trees retain one actual workspace"
        );
        let workspace = self.stack.workspace;
        let schema = builder.runtime_schema();
        other.with_template_snapshots(builder, f, |_, _, _| Ok(()), |_, _, _| Ok(()))?;
        self.load(16, f);
        other.load(16, f);
        f.instruction(&Instruction::I64Eq);
        intersect(eligible, f);
        self.is_snapshot(f);
        f.instruction(&Instruction::If(BlockType::Empty));
        eligible.load(f);
        f.instruction(&Instruction::If(BlockType::Empty));
        leaf_pair(
            ChoiceSnapshotView {
                workspace,
                address: self.address,
            },
            ChoiceSnapshotView {
                workspace,
                address: other.address,
            },
            builder,
            f,
        )?;
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::Else);
        // Structural shape is checked before any right-hand leaf projection.
        // A lawful different tree rejects the proof, not the matcher program.
        other.is_kind(ChoiceEntryKind::RequiredRun, f);
        intersect(eligible, f);
        eligible.load(f);
        f.instruction(&Instruction::If(BlockType::Empty));
        let left = RunLayout::read(self.address, workspace, builder, f);
        let right = RunLayout::read(other.address, workspace, builder, f);
        let relative = schema.reserve_i64_local(f);
        let right_address = schema.reserve_i64_local(f);
        let right_state = schema.reserve_i64_local(f);
        let parent = schema.reserve_i64_local(f);
        left.walk(
            workspace,
            builder,
            f,
            |view, builder, f| {
                assert!(
                    core::ptr::eq(view.workspace, workspace),
                    "leaf reader retains its original arena"
                );
                eligible.load(f);
                f.instruction(&Instruction::If(BlockType::Empty));
                view.address.load(f);
                self.address.load(f);
                f.instruction(&Instruction::I64Sub);
                relative.store(f);
                address_at(other.address, relative, right_address, f);
                leaf_pair(
                    view,
                    ChoiceSnapshotView {
                        workspace,
                        address: right_address,
                    },
                    builder,
                    f,
                )?;
                f.instruction(&Instruction::End);
                Ok(())
            },
            |view, state, builder, f| {
                assert!(
                    core::ptr::eq(view.workspace, workspace),
                    "node reader retains its original arena"
                );
                eligible.load(f);
                f.instruction(&Instruction::If(BlockType::Empty));
                view.definition.load(f);
                self.address.load(f);
                f.instruction(&Instruction::I64Sub);
                relative.store(f);
                address_at(other.address, relative, right_address, f);
                for offset in [0, 24, 40, 48, 56, 88, 120, 128, 136] {
                    ChoiceStack::load(view.definition, offset, f);
                    ChoiceStack::load(right_address, offset, f);
                    f.instruction(&Instruction::I64Eq);
                    intersect(eligible, f);
                }
                eligible.load(f);
                f.instruction(&Instruction::If(BlockType::Empty));
                right.state_for(relative, right_state, parent, workspace, builder, f);
                let original = RunLayout::read(view.definition, workspace, builder, f);
                let paired = RunLayout::read(right_address, workspace, builder, f);
                compare_prefix(
                    view.definition,
                    right_address,
                    &original,
                    eligible,
                    builder,
                    f,
                );
                compare_prefix(state, right_state, &original, eligible, builder, f);
                compare_region(
                    original.offsets,
                    paired.offsets,
                    original.count,
                    8,
                    eligible,
                    builder,
                    f,
                );
                compare_tables(&original, &paired, eligible, workspace, builder, f);
                paired.release(builder, f);
                original.release(builder, f);
                f.instruction(&Instruction::End);
                f.instruction(&Instruction::End);
                Ok(())
            },
        )?;
        for local in [parent, right_state, right_address, relative] {
            schema.release_i64_local(local, f);
        }
        right.release(builder, f);
        left.release(builder, f);
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        Ok(())
    }
}

fn intersect(eligible: I32Local, f: &mut Function) {
    eligible.load(f);
    f.instruction(&Instruction::I32And);
    eligible.store(f);
}

fn compare_region(
    left: I64Local,
    right: I64Local,
    count: I64Local,
    width: u64,
    eligible: I32Local,
    builder: &FunctionBuilder<'_>,
    f: &mut Function,
) {
    let schema = builder.runtime_schema();
    let index = schema.reserve_i64_local(f);
    f.instruction(&Instruction::I64Const(0));
    index.store(f);
    f.instruction(&Instruction::Block(BlockType::Empty));
    f.instruction(&Instruction::Loop(BlockType::Empty));
    index.load(f);
    count.load(f);
    f.instruction(&Instruction::I64GeU);
    f.instruction(&Instruction::BrIf(1));
    for address in [left, right] {
        address.load(f);
        index.load(f);
        f.instruction(&Instruction::I64Const(width as i64));
        f.instruction(&Instruction::I64Mul);
        f.instruction(&Instruction::I64Add);
        f.instruction(&Instruction::I32WrapI64);
        f.instruction(&Instruction::I64Load(FunctionBuilder::memarg8(0)));
    }
    f.instruction(&Instruction::I64Eq);
    intersect(eligible, f);
    index.load(f);
    f.instruction(&Instruction::I64Const(1));
    f.instruction(&Instruction::I64Add);
    index.store(f);
    f.instruction(&Instruction::Br(0));
    f.instruction(&Instruction::End);
    f.instruction(&Instruction::End);
    schema.release_i64_local(index, f);
}

fn compare_prefix(
    left: I64Local,
    right: I64Local,
    shape: &RunLayout,
    eligible: I32Local,
    builder: &FunctionBuilder<'_>,
    f: &mut Function,
) {
    let schema = builder.runtime_schema();
    let index = schema.reserve_i64_local(f);
    let bytes = schema.reserve_i64_local(f);
    shape.mincap.load(f);
    f.instruction(&Instruction::I64Const(2));
    f.instruction(&Instruction::I64Mul);
    shape.maxcap.load(f);
    f.instruction(&Instruction::I64Add);
    f.instruction(&Instruction::I64Const(4));
    f.instruction(&Instruction::I64Mul);
    f.instruction(&Instruction::I64Const(HEADER_BYTES as i64));
    f.instruction(&Instruction::I64Add);
    bytes.store(f);
    f.instruction(&Instruction::I64Const(0));
    index.store(f);
    f.instruction(&Instruction::Block(BlockType::Empty));
    f.instruction(&Instruction::Loop(BlockType::Empty));
    index.load(f);
    bytes.load(f);
    f.instruction(&Instruction::I64GeU);
    f.instruction(&Instruction::BrIf(1));
    // Links and conservative trace flags are not semantic template state.
    index.load(f);
    f.instruction(&Instruction::I64Const(8));
    f.instruction(&Instruction::I64LtU);
    index.load(f);
    f.instruction(&Instruction::I64Const(16));
    f.instruction(&Instruction::I64GeU);
    index.load(f);
    f.instruction(&Instruction::I64Const(104));
    f.instruction(&Instruction::I64LtU);
    f.instruction(&Instruction::I32And);
    f.instruction(&Instruction::I32Or);
    index.load(f);
    f.instruction(&Instruction::I64Const(120));
    f.instruction(&Instruction::I64GeU);
    f.instruction(&Instruction::I32Or);
    f.instruction(&Instruction::If(BlockType::Empty));
    for address in [left, right] {
        address.load(f);
        index.load(f);
        f.instruction(&Instruction::I64Add);
        f.instruction(&Instruction::I32WrapI64);
        f.instruction(&Instruction::I32Load8U(FunctionBuilder::memarg8(0)));
    }
    f.instruction(&Instruction::I32Eq);
    intersect(eligible, f);
    f.instruction(&Instruction::End);
    index.load(f);
    f.instruction(&Instruction::I64Const(1));
    f.instruction(&Instruction::I64Add);
    index.store(f);
    f.instruction(&Instruction::Br(0));
    f.instruction(&Instruction::End);
    f.instruction(&Instruction::End);
    schema.release_i64_local(bytes, f);
    schema.release_i64_local(index, f);
}

fn compare_tables(
    left: &RunLayout,
    right: &RunLayout,
    eligible: I32Local,
    workspace: &MatcherWorkspace,
    builder: &FunctionBuilder<'_>,
    f: &mut Function,
) {
    let schema = builder.runtime_schema();
    let a = schema.reserve_i64_local(f);
    let b = schema.reserve_i64_local(f);
    let end = schema.reserve_i64_local(f);
    let definition = schema.reserve_i64_local(f);
    let state_a = schema.reserve_i64_local(f);
    let state_b = schema.reserve_i64_local(f);
    left.table.load(f);
    a.store(f);
    right.table.load(f);
    b.store(f);
    left.table.load(f);
    left.table_bytes.load(f);
    f.instruction(&Instruction::I64Add);
    end.store(f);
    f.instruction(&Instruction::Block(BlockType::Empty));
    f.instruction(&Instruction::Loop(BlockType::Empty));
    a.load(f);
    end.load(f);
    f.instruction(&Instruction::I64GeU);
    f.instruction(&Instruction::BrIf(1));
    eligible.load(f);
    f.instruction(&Instruction::I32Eqz);
    f.instruction(&Instruction::BrIf(1));
    for offset in [0, 8, 16] {
        ChoiceStack::load(a, offset, f);
        ChoiceStack::load(b, offset, f);
        f.instruction(&Instruction::I64Eq);
        intersect(eligible, f);
    }
    eligible.load(f);
    f.instruction(&Instruction::If(BlockType::Empty));
    ChoiceStack::load(a, 8, f);
    left.definition.load(f);
    f.instruction(&Instruction::I64Add);
    definition.store(f);
    add_const(a, TABLE_HEADER, state_a, f);
    add_const(b, TABLE_HEADER, state_b, f);
    let shape = RunLayout::read(definition, workspace, builder, f);
    compare_prefix(state_a, state_b, &shape, eligible, builder, f);
    shape.release(builder, f);
    f.instruction(&Instruction::End);
    for address in [a, b] {
        address.load(f);
        ChoiceStack::load(address, 0, f);
        f.instruction(&Instruction::I64Add);
        address.store(f);
    }
    f.instruction(&Instruction::Br(0));
    f.instruction(&Instruction::End);
    f.instruction(&Instruction::End);
    for local in [state_b, state_a, definition, end, b, a] {
        schema.release_i64_local(local, f);
    }
}
