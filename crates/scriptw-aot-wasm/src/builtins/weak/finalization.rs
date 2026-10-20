use super::*;
pub(super) mod cells;
impl FunctionBuilder<'_> {
    pub(super) fn emit_finalization_constructor(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let callback = s.reserve_value_local(f);
        let result = s.reserve_value_local(f);
        self.emit_builtin_arg_to_value(0, &callback, f);
        let header = self.emit_weak_header(WeakFamily::Registry, Some(&callback), f)?;
        let stored = s
            .reserve_gc_local(f)
            .initialize(s.struct_type::<StoredValue>().from_value(&callback, f), f);
        let realm = s
            .reserve_gc_local(f)
            .initialize(self.emit_current_function_realm(f), f);
        let record = s.reserve_gc_local(f).initialize(
            s.struct_type::<FinalizationRegistryObject>().construct(
                (
                    GcOperand::reference(&header, s),
                    GcOperand::reference(&stored, s),
                    GcOperand::reference(&realm, s),
                    GcOperand::null(s),
                    GcOperand::constant(false),
                ),
                f,
            ),
            f,
        );
        self.weak_import(GcHostImport::FinalizationTrack)?
            .track_finalization_registry(&record, s, f);
        result.set_reference(&record, s, f);
        self.completion().set_normal(&result, f);
        record.clear(f);
        realm.clear(f);
        stored.clear(f);
        header.clear(f);
        result.clear(f);
        callback.clear(f);
        Ok(())
    }
    pub(super) fn emit_finalization_register(&mut self, f: &mut Function) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let registry =
            self.emit_weak_receiver::<FinalizationRegistryObject>(WeakFamily::Registry, f)?;
        let target = s.reserve_value_local(f);
        let holdings = s.reserve_value_local(f);
        let token = s.reserve_value_local(f);
        self.emit_builtin_arg_to_value(0, &target, f);
        self.emit_builtin_arg_to_value(1, &holdings, f);
        self.emit_builtin_arg_to_value(2, &token, f);
        self.emit_whole_can_be_held_weakly_i32(&target, f);
        f.instruction(&Instruction::I32Eqz);
        self.emit_weak_assert(
            RuntimeErrorMessage::FINALIZATIONREGISTRY_TARGET_CANNOT_BE_HELD_WEAKLY,
            f,
        )?;
        self.emit_tagged_payload_same_value_i32(&target, &holdings, f)?;
        self.emit_weak_assert(RuntimeErrorMessage::FINALIZATIONREGISTRY_TARGET_AND_HOLDINGS_MUST_NOT_BE_THE_SAME_VALUE, f)?;
        token.tag().load(f);
        f.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Undefined.tag()));
        f.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_whole_can_be_held_weakly_i32(&token, f);
        f.instruction(&Instruction::I32Eqz);
        self.emit_weak_assert(
            RuntimeErrorMessage::FINALIZATIONREGISTRY_UNREGISTER_TOKEN_CANNOT_BE_HELD_WEAKLY,
            f,
        )?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let stored = s
            .reserve_gc_local(f)
            .initialize(s.struct_type::<StoredValue>().from_value(&holdings, f), f);
        let head = s.reserve_gc_local(f).initialize(
            s.field(FinalizationRegistryObjectSchema::CELLS)
                .read(&registry, s, f)
                .reference(),
            f,
        );
        let cell = s.reserve_gc_local(f).initialize(
            s.struct_type::<FinalizationCell>().construct(
                (
                    GcOperand::reference(&stored, s),
                    GcOperand::null(s),
                    GcOperand::reference(&head, s),
                ),
                f,
            ),
            f,
        );
        let resource = s.reserve_gc_local(f).initialize(
            self.weak_import(GcHostImport::FinalizationCellAllocate)?
                .allocate_finalization_cell(&cell, &target, &token, s, f),
            f,
        );
        s.field(FinalizationCellSchema::RESOURCE).write(
            &cell,
            GcOperand::nullable_reference(&resource, s),
            s,
            f,
        );
        s.field(FinalizationRegistryObjectSchema::CELLS).write(
            &registry,
            GcOperand::nullable_reference(&cell, s),
            s,
            f,
        );
        self.completion().initialize(f);
        resource.clear(f);
        cell.clear(f);
        head.clear(f);
        stored.clear(f);
        token.clear(f);
        holdings.clear(f);
        target.clear(f);
        registry.clear(f);
        Ok(())
    }
    pub(super) fn emit_finalization_unregister(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let registry =
            self.emit_weak_receiver::<FinalizationRegistryObject>(WeakFamily::Registry, f)?;
        let token = s.reserve_value_local(f);
        let removed = s.reserve_i32_local(f);
        self.emit_builtin_arg_to_value(0, &token, f);
        self.emit_whole_can_be_held_weakly_i32(&token, f);
        f.instruction(&Instruction::I32Eqz);
        self.emit_weak_assert(
            RuntimeErrorMessage::FINALIZATIONREGISTRY_UNREGISTER_TOKEN_CANNOT_BE_HELD_WEAKLY,
            f,
        )?;
        removed.set_constant(0, f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        let again = self.open_frame(ControlFrameKind::Loop, f);
        let cell = self.emit_find_finalization_cell(
            &registry,
            cells::CellSelection::Token(&token),
            true,
            f,
        )?;
        cell.load(s, f).is_null(f);
        self.emit_branch_if_to_target(exit, f);
        removed.set_constant(1, f);
        self.emit_branch_to_target(again, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        cell.clear(f);
        self.emit_collection_boolean(removed, f);
        s.release_i32_local(removed, f);
        token.clear(f);
        registry.clear(f);
        Ok(())
    }
}
