//! A proven fixed-point Run preserves every virtual ordered continuation.
use super::*;
use crate::builtins::regexp::counted::required_choice_run::ProvenRequiredChoiceRun;
mod natural;
use natural::{NaturalRegion, UnitDelta};
mod logical;
pub(super) use logical::LogicalRunEntry;
mod exhaustion;
mod path;
mod tree;
pub(in crate::builtins::regexp) use tree::CheckedTemplateRunNode;

pub(super) const HEADER_BYTES: u64 = 144;

/// Addresses are derived only from a checked Run in the same exclusive arena.
struct RunLayout {
    address: I64Local,
    count: I64Local,
    index: I64Local,
    row: I64Local,
    mincap: I64Local,
    maxcap: I64Local,
    remaining_used: I64Local,
    affine_used: I64Local,
    base_used: I64Local,
    maximum_kind: I64Local,
    current_kind_override: I64Local,
    observed_group: I64Local,
    pending_complete: I64Local,
    template_bytes: I64Local,
    node_count: I64Local,
    table_bytes: I64Local,
    definition: I64Local,
    remaining: I64Local,
    affine: I64Local,
    base: I64Local,
    prefix_bytes: I64Local,
    offsets: I64Local,
    template: I64Local,
    table: I64Local,
    length: I64Local,
}

impl RunLayout {
    fn reserve(address: I64Local, builder: &FunctionBuilder<'_>, f: &mut Function) -> Self {
        let schema = builder.runtime_schema();
        Self {
            address,
            count: schema.reserve_i64_local(f),
            index: schema.reserve_i64_local(f),
            row: schema.reserve_i64_local(f),
            mincap: schema.reserve_i64_local(f),
            maxcap: schema.reserve_i64_local(f),
            remaining_used: schema.reserve_i64_local(f),
            affine_used: schema.reserve_i64_local(f),
            base_used: schema.reserve_i64_local(f),
            maximum_kind: schema.reserve_i64_local(f),
            current_kind_override: schema.reserve_i64_local(f),
            observed_group: schema.reserve_i64_local(f),
            pending_complete: schema.reserve_i64_local(f),
            template_bytes: schema.reserve_i64_local(f),
            node_count: schema.reserve_i64_local(f),
            table_bytes: schema.reserve_i64_local(f),
            definition: address,
            remaining: schema.reserve_i64_local(f),
            affine: schema.reserve_i64_local(f),
            base: schema.reserve_i64_local(f),
            prefix_bytes: schema.reserve_i64_local(f),
            offsets: schema.reserve_i64_local(f),
            template: schema.reserve_i64_local(f),
            table: schema.reserve_i64_local(f),
            length: schema.reserve_i64_local(f),
        }
    }

    fn fields(&self) -> [(u64, I64Local); 15] {
        [
            (24, self.count),
            (32, self.index),
            (40, self.row),
            (48, self.mincap),
            (56, self.maxcap),
            (64, self.remaining_used),
            (72, self.affine_used),
            (80, self.base_used),
            (88, self.maximum_kind),
            (96, self.current_kind_override),
            (104, self.observed_group),
            (112, self.pending_complete),
            (120, self.template_bytes),
            (128, self.node_count),
            (136, self.table_bytes),
        ]
    }

    fn derive_addresses(&self, _workspace: &MatcherWorkspace, f: &mut Function) {
        self.address.load(f);
        f.instruction(&Instruction::I64Const(HEADER_BYTES as i64));
        f.instruction(&Instruction::I64Add);
        self.remaining.store(f);
        self.remaining.load(f);
        self.mincap.load(f);
        f.instruction(&Instruction::I64Const(4));
        f.instruction(&Instruction::I64Mul);
        f.instruction(&Instruction::I64Add);
        self.affine.store(f);
        self.affine.load(f);
        self.mincap.load(f);
        f.instruction(&Instruction::I64Const(4));
        f.instruction(&Instruction::I64Mul);
        f.instruction(&Instruction::I64Add);
        self.base.store(f);
        self.base.load(f);
        self.maxcap.load(f);
        f.instruction(&Instruction::I64Const(4));
        f.instruction(&Instruction::I64Mul);
        f.instruction(&Instruction::I64Add);
        f.instruction(&Instruction::I64Const(7));
        f.instruction(&Instruction::I64Add);
        f.instruction(&Instruction::I64Const(-8));
        f.instruction(&Instruction::I64And);
        self.address.load(f);
        f.instruction(&Instruction::I64Sub);
        self.prefix_bytes.store(f);
        self.definition.load(f);
        self.prefix_bytes.load(f);
        f.instruction(&Instruction::I64Add);
        self.offsets.store(f);
        self.offsets.load(f);
        self.count.load(f);
        f.instruction(&Instruction::I64Const(8));
        f.instruction(&Instruction::I64Mul);
        f.instruction(&Instruction::I64Add);
        self.template.store(f);
        self.template.load(f);
        self.template_bytes.load(f);
        f.instruction(&Instruction::I64Add);
        self.table.store(f);
        self.table.load(f);
        self.definition.load(f);
        f.instruction(&Instruction::I64Sub);
        self.table_bytes.load(f);
        f.instruction(&Instruction::I64Add);
        self.length.store(f);
    }

    fn read(
        address: I64Local,
        workspace: &MatcherWorkspace,
        builder: &FunctionBuilder<'_>,
        f: &mut Function,
    ) -> Self {
        Self::read_state(address, address, workspace, builder, f)
    }

    fn read_state(
        definition: I64Local,
        state: I64Local,
        workspace: &MatcherWorkspace,
        builder: &FunctionBuilder<'_>,
        f: &mut Function,
    ) -> Self {
        let mut layout = Self::reserve(state, builder, f);
        layout.definition = definition;
        for (offset, local) in layout.fields() {
            ChoiceStack::load(state, offset, f);
            local.store(f);
        }
        layout.derive_addresses(workspace, f);
        layout
    }

    fn validate(
        &self,
        workspace: &MatcherWorkspace,
        builder: &FunctionBuilder<'_>,
        f: &mut Function,
    ) {
        self.count.load(f);
        f.instruction(&Instruction::I64Eqz);
        self.index.load(f);
        self.count.load(f);
        f.instruction(&Instruction::I64GeU);
        f.instruction(&Instruction::I32Or);
        self.mincap.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::I32Or);
        for local in [
            self.count,
            self.mincap,
            self.maxcap,
            self.row,
            self.node_count,
            self.template_bytes,
            self.table_bytes,
        ] {
            local.load(f);
            workspace.maximum_capacity.load(f);
            f.instruction(&Instruction::I64GtU);
            f.instruction(&Instruction::I32Or);
        }
        self.maximum_kind.load(f);
        f.instruction(&Instruction::I64Const(
            RegExpRepeatMaximumKind::Unbounded.word() as i64,
        ));
        f.instruction(&Instruction::I64GtU);
        f.instruction(&Instruction::I32Or);
        self.row.load(f);
        workspace.capture_bytes.load(f);
        f.instruction(&Instruction::I64LtU);
        f.instruction(&Instruction::I32Or);
        self.row.load(f);
        f.instruction(&Instruction::I64Const(
            REGEXP_REPEAT_STATE_HEADER_SIZE as i64,
        ));
        f.instruction(&Instruction::I64Add);
        self.mincap.load(f);
        self.maxcap.load(f);
        f.instruction(&Instruction::I64Add);
        f.instruction(&Instruction::I64Const(4));
        f.instruction(&Instruction::I64Mul);
        f.instruction(&Instruction::I64Add);
        workspace.live_bytes.load(f);
        f.instruction(&Instruction::I64GtU);
        f.instruction(&Instruction::I32Or);
        ChoiceStack::load(self.definition, 0, f);
        self.length.load(f);
        f.instruction(&Instruction::I64Ne);
        f.instruction(&Instruction::I32Or);
        self.template_bytes.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::I32Or);
        self.template_bytes.load(f);
        f.instruction(&Instruction::I64Const(7));
        f.instruction(&Instruction::I64And);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::I32Eqz);
        f.instruction(&Instruction::I32Or);
        self.current_kind_override.load(f);
        f.instruction(&Instruction::I64Const(EMPTY_LINK));
        f.instruction(&Instruction::I64Ne);
        self.current_kind_override.load(f);
        f.instruction(&Instruction::I64Const(
            ChoiceEntryKind::LazyProgressAttempt.word(),
        ));
        f.instruction(&Instruction::I64Ne);
        f.instruction(&Instruction::I32And);
        f.instruction(&Instruction::I32Or);
        for flag in [self.observed_group, self.pending_complete] {
            flag.load(f);
            f.instruction(&Instruction::I64Const(1));
            f.instruction(&Instruction::I64GtU);
            f.instruction(&Instruction::I32Or);
        }
        reject(workspace, builder, f);
    }

    fn remaining(&self) -> NaturalRegion {
        NaturalRegion {
            limbs: self.remaining,
            capacity: self.mincap,
            used: self.remaining_used,
        }
    }
    fn affine(&self) -> NaturalRegion {
        NaturalRegion {
            limbs: self.affine,
            capacity: self.mincap,
            used: self.affine_used,
        }
    }
    fn base(&self) -> NaturalRegion {
        NaturalRegion {
            limbs: self.base,
            capacity: self.maxcap,
            used: self.base_used,
        }
    }
    fn publish_used(&self, f: &mut Function) {
        for (offset, local) in [
            (64, self.remaining_used),
            (72, self.affine_used),
            (80, self.base_used),
        ] {
            ChoiceStack::store(self.address, offset, local, f);
        }
    }
    fn release(self, builder: &FunctionBuilder<'_>, f: &mut Function) {
        let schema = builder.runtime_schema();
        for local in [
            self.length,
            self.table,
            self.template,
            self.offsets,
            self.prefix_bytes,
            self.base,
            self.affine,
            self.remaining,
            self.table_bytes,
            self.node_count,
            self.template_bytes,
            self.pending_complete,
            self.observed_group,
            self.current_kind_override,
            self.maximum_kind,
            self.base_used,
            self.affine_used,
            self.remaining_used,
            self.maxcap,
            self.mincap,
            self.row,
            self.index,
            self.count,
        ] {
            schema.release_i64_local(local, f);
        }
    }
}

pub(super) fn validate_header(
    address: I64Local,
    workspace: &MatcherWorkspace,
    builder: &FunctionBuilder<'_>,
    f: &mut Function,
) {
    let layout = RunLayout::read(address, workspace, builder, f);
    layout.validate(workspace, builder, f);
    layout.release(builder, f);
}

impl ChoiceStack<'_> {
    /// The only writer consumes actual two-group proof; no raw span can mint a Run.
    pub(in crate::builtins::regexp) fn append_required_run(
        self,
        builder: &mut FunctionBuilder<'_>,
        proof: &ProvenRequiredChoiceRun<'_, '_>,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        assert!(
            core::ptr::eq(self.workspace, proof.workspace()),
            "Run proof retains its original workspace"
        );
        self.invalidate_active_run(builder, f);
        let schema = builder.runtime_schema();
        let address = schema.reserve_i64_local(f);
        let required = schema.reserve_i64_local(f);
        let previous = schema.reserve_i64_local(f);
        let source = schema.reserve_i64_local(f);
        let source_used = schema.reserve_i64_local(f);
        self.address(proof.replaced_oldest(), address, f);
        ChoiceStack::load(address, 8, f);
        previous.store(f);
        let layout = RunLayout::reserve(address, builder, f);
        for (input, output) in [
            (proof.entry_count(), layout.count),
            (proof.row_offset(), layout.row),
            (proof.minimum_capacity(), layout.mincap),
            (proof.maximum_capacity(), layout.maxcap),
            (proof.maximum_kind(), layout.maximum_kind),
            (proof.template_bytes(), layout.template_bytes),
        ] {
            input.load(f);
            output.store(f);
        }
        layout.count.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Sub);
        layout.index.store(f);
        f.instruction(&Instruction::I64Const(EMPTY_LINK));
        layout.current_kind_override.store(f);
        for flag in [layout.observed_group, layout.pending_complete] {
            f.instruction(&Instruction::I64Const(0));
            flag.store(f);
        }
        self.address(proof.template_oldest(), source, f);
        layout.measure_template(source, self.workspace, builder, f);
        layout.derive_addresses(self.workspace, f);
        proof.replaced_oldest().load(f);
        layout.length.load(f);
        f.instruction(&Instruction::I64Add);
        required.store(f);
        self.workspace.ensure_choice_bytes(builder, required, f)?;
        // MemoryCopy handles overlap when the compact header lies within an old group.
        layout.template.load(f);
        f.instruction(&Instruction::I32WrapI64);
        source.load(f);
        f.instruction(&Instruction::I32WrapI64);
        layout.template_bytes.load(f);
        f.instruction(&Instruction::I32WrapI64);
        f.instruction(&Instruction::MemoryCopy {
            src_mem: 0,
            dst_mem: 0,
        });
        proof.load_minimum_used(builder, source_used, f);
        layout.remaining().copy_from(
            NaturalRegion {
                limbs: proof.minimum_limbs(),
                capacity: proof.minimum_capacity(),
                used: source_used,
            },
            f,
        );
        layout
            .remaining()
            .change_one(UnitDelta::Increment, self.workspace, builder, f);
        layout.affine().set_one(f);
        proof.load_maximum_used(builder, source_used, f);
        layout.base().copy_from(
            NaturalRegion {
                limbs: proof.maximum_limbs(),
                capacity: proof.maximum_capacity(),
                used: source_used,
            },
            f,
        );
        ChoiceStack::store(address, 0, layout.length, f);
        ChoiceStack::store(address, 8, previous, f);
        address.load(f);
        f.instruction(&Instruction::I32WrapI64);
        f.instruction(&Instruction::I64Const(ChoiceEntryKind::RequiredRun.word()));
        f.instruction(&Instruction::I64Store(FunctionBuilder::memarg8(16)));
        for (offset, local) in layout.fields() {
            ChoiceStack::store(address, offset, local, f);
        }
        layout.initialize_tree(self.workspace, builder, f);
        proof.replaced_oldest().load(f);
        self.workspace.choice_top.store(f);
        required.load(f);
        self.workspace.choice_used.store(f);
        layout.release(builder, f);
        for local in [source_used, source, previous, required, address] {
            schema.release_i64_local(local, f);
        }
        Ok(())
    }
}

fn reject(workspace: &MatcherWorkspace, builder: &FunctionBuilder<'_>, f: &mut Function) {
    f.instruction(&Instruction::If(BlockType::Empty));
    workspace.fail(builder, RegExpMatcherFailure::CorruptProgram, f);
    f.instruction(&Instruction::End);
}
