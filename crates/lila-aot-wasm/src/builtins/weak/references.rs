use super::*;
impl FunctionBuilder<'_> {
    pub(super) fn emit_weak_ref_constructor(&mut self, f: &mut Function) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let target = s.reserve_value_local(f);
        let result = s.reserve_value_local(f);
        self.emit_builtin_arg_to_value(0, &target, f);
        let header = self.emit_weak_header(WeakFamily::Ref, Some(&target), f)?;
        let record = s.reserve_gc_local(f).initialize(
            s.struct_type::<WeakRefObject>().construct(
                (
                    GcOperand::reference(&header, s),
                    GcOperand::null(s),
                    GcOperand::i32_local(target.tag()),
                ),
                f,
            ),
            f,
        );
        let resource = s.reserve_gc_local(f).initialize(
            self.weak_import(GcHostImport::WeakRefAllocate)?
                .allocate_weak_ref(&record, &target, s, f),
            f,
        );
        s.field(WeakRefObjectSchema::RESOURCE).write(
            &record,
            GcOperand::nullable_reference(&resource, s),
            s,
            f,
        );
        result.set_reference(&record, s, f);
        self.completion().set_normal(&result, f);
        resource.clear(f);
        record.clear(f);
        header.clear(f);
        result.clear(f);
        target.clear(f);
        Ok(())
    }
    pub(super) fn emit_weak_ref_deref(&mut self, f: &mut Function) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let record = self.emit_weak_receiver::<WeakRefObject>(WeakFamily::Ref, f)?;
        let resource = s.reserve_gc_local(f).initialize(
            s.field(WeakRefObjectSchema::RESOURCE)
                .read(&record, s, f)
                .reference()
                .require_non_null(f),
            f,
        );
        let result = s.reserve_value_local(f);
        result.set_undefined(f);
        self.weak_import(GcHostImport::WeakRefDeref)?
            .deref_weak_ref(&resource, &result, s, f);
        result.reference().load(f);
        f.instruction(&Instruction::RefIsNull);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        s.field(WeakRefObjectSchema::TARGET_TAG)
            .read(&record, s, f)
            .store(result.tag(), f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.completion().set_normal(&result, f);
        result.clear(f);
        resource.clear(f);
        record.clear(f);
        Ok(())
    }
}
