use super::*;
pub(in super::super) enum CellSelection<'a> {
    ClearedTarget,
    Token(&'a ValueLocals),
}
impl FunctionBuilder<'_> {
    pub(in super::super) fn emit_find_finalization_cell(
        &mut self,
        registry: &GcLocal<FinalizationRegistryObject>,
        selection: CellSelection<'_>,
        remove: bool,
        f: &mut Function,
    ) -> Result<GcLocal<FinalizationCell, Nullable>, EmitError> {
        let s = self.runtime_schema();
        let current = s.reserve_gc_local(f).initialize(
            s.field(FinalizationRegistryObjectSchema::CELLS)
                .read(registry, s, f)
                .reference(),
            f,
        );
        let previous = s
            .reserve_gc_local::<FinalizationCell, Nullable>(f)
            .initialize_null(s, f);
        let selected = s
            .reserve_gc_local::<FinalizationCell, Nullable>(f)
            .initialize_null(s, f);
        let matched = s.reserve_i32_local(f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        let again = self.open_frame(ControlFrameKind::Loop, f);
        current.load(s, f).is_null(f);
        self.emit_branch_if_to_target(exit, f);
        let cell = s
            .reserve_gc_local(f)
            .initialize(current.load(s, f).require_non_null(f), f);
        let next = s.reserve_gc_local(f).initialize(
            s.field(FinalizationCellSchema::NEXT)
                .read(&cell, s, f)
                .reference(),
            f,
        );
        let resource = s.reserve_gc_local(f).initialize(
            s.field(FinalizationCellSchema::RESOURCE)
                .read(&cell, s, f)
                .reference()
                .require_non_null(f),
            f,
        );
        match selection {
            CellSelection::ClearedTarget => {
                self.weak_import(GcHostImport::FinalizationTargetAlive)?
                    .finalization_target_alive(&resource, matched, s, f);
                matched.load(f);
                f.instruction(&Instruction::I32Eqz);
                matched.store(f);
            }
            CellSelection::Token(token) => self
                .weak_import(GcHostImport::FinalizationTokenMatches)?
                .finalization_token_matches(&resource, token, matched, s, f),
        }
        matched.load(f);
        self.open_frame(ControlFrameKind::If, f);
        selected.replace(cell.load(s, f).nullable(), f);
        if remove {
            self.weak_import(GcHostImport::FinalizationCellRemove)?
                .remove_finalization_cell(&resource, s, f);
            previous.load(s, f).is_null(f);
            self.open_frame(ControlFrameKind::If, f);
            s.field(FinalizationRegistryObjectSchema::CELLS).write(
                registry,
                GcOperand::reference(&next, s),
                s,
                f,
            );
            f.instruction(&Instruction::Else);
            let prev = s
                .reserve_gc_local(f)
                .initialize(previous.load(s, f).require_non_null(f), f);
            s.field(FinalizationCellSchema::NEXT).write(
                &prev,
                GcOperand::reference(&next, s),
                s,
                f,
            );
            prev.clear(f);
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
            s.field(FinalizationCellSchema::NEXT)
                .write(&cell, GcOperand::null(s), s, f);
            s.field(FinalizationCellSchema::RESOURCE)
                .write(&cell, GcOperand::null(s), s, f);
        }
        self.emit_branch_to_target(exit, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        previous.replace(cell.load(s, f).nullable(), f);
        current.replace(next.load(s, f), f);
        self.emit_branch_to_target(again, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        resource.clear(f);
        next.clear(f);
        cell.clear(f);
        s.release_i32_local(matched, f);
        previous.clear(f);
        current.clear(f);
        Ok(selected)
    }
}
