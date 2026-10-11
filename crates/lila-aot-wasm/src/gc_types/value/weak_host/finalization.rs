use super::*;
impl DeclaredGcHostImport {
    pub(crate) fn track_finalization_registry(
        self,
        registry: &GcLocal<FinalizationRegistryObject>,
        s: &RuntimeSchema,
        f: &mut Function,
    ) {
        assert_eq!(self.import(), GcHostImport::FinalizationTrack);
        registry.load(s, f);
        self.emit_call_instruction(f);
    }
    pub(crate) fn next_finalization_registry(
        self,
        s: &RuntimeSchema,
        f: &mut Function,
    ) -> GcStackReference<FinalizationRegistryObject, Nullable> {
        assert_eq!(self.import(), GcHostImport::FinalizationNextRegistry);
        self.emit_call_instruction(f);
        f.instruction(&Instruction::RefCastNullable(
            s.reference_type::<FinalizationRegistryObject>(GcNullability::Nullable)
                .heap_type,
        ));
        GcStackReference::new()
    }
    pub(crate) fn allocate_finalization_cell(
        self,
        owner: &GcLocal<FinalizationCell>,
        target: &ValueLocals,
        token: &ValueLocals,
        s: &RuntimeSchema,
        f: &mut Function,
    ) -> GcStackReference<HostResource> {
        assert_eq!(self.import(), GcHostImport::FinalizationCellAllocate);
        owner.load(s, f);
        target.reference().load(f);
        token.reference().load(f);
        self.emit_call_instruction(f);
        bind_resource(HostResourceKind::FinalizationCell, s, f)
    }
    pub(crate) fn finalization_target_alive(
        self,
        resource: &GcLocal<HostResource>,
        alive: I32Local,
        s: &RuntimeSchema,
        f: &mut Function,
    ) {
        assert_eq!(self.import(), GcHostImport::FinalizationTargetAlive);
        resource_operand(resource, s, f);
        self.emit_call_instruction(f);
        alive.store(f);
    }
    pub(crate) fn finalization_token_matches(
        self,
        resource: &GcLocal<HostResource>,
        token: &ValueLocals,
        matched: I32Local,
        s: &RuntimeSchema,
        f: &mut Function,
    ) {
        assert_eq!(self.import(), GcHostImport::FinalizationTokenMatches);
        resource_operand(resource, s, f);
        token.reference().load(f);
        self.emit_call_instruction(f);
        matched.store(f);
    }
    pub(crate) fn remove_finalization_cell(
        self,
        resource: &GcLocal<HostResource>,
        s: &RuntimeSchema,
        f: &mut Function,
    ) {
        assert_eq!(self.import(), GcHostImport::FinalizationCellRemove);
        resource_operand(resource, s, f);
        self.emit_call_instruction(f);
    }
    pub(crate) fn report_cleanup_exception(
        self,
        value: &ValueLocals,
        name: &GcLocal<StringValue, Nullable>,
        message: &GcLocal<StringValue, Nullable>,
        s: &RuntimeSchema,
        f: &mut Function,
    ) {
        assert_eq!(self.import(), GcHostImport::ReportCleanupException);
        value.emit(f);
        name.load(s, f);
        message.load(s, f);
        self.emit_call_instruction(f);
    }
}
