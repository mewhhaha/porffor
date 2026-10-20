//! Typed weak host calls; references never travel as numeric identities.
use super::*;
mod finalization;

fn resource_operand(resource: &GcLocal<HostResource>, s: &RuntimeSchema, f: &mut Function) {
    let _ = s.field(HostResourceSchema::RESOURCE).read(resource, s, f);
}

fn bind_resource(
    kind: HostResourceKind,
    s: &RuntimeSchema,
    f: &mut Function,
) -> GcStackReference<HostResource> {
    let native = NativeSharedBufferResult::bind(f);
    let resource = s.struct_type::<HostResource>().construct(
        (
            GcOperand::constant(kind),
            GcOperand {
                source: OperandSource::NonNullLocal(native.index),
                shape: PhantomData,
            },
        ),
        f,
    );
    native.clear(f);
    resource
}

impl DeclaredGcHostImport {
    pub(crate) fn allocate_weak_map(
        self,
        s: &RuntimeSchema,
        f: &mut Function,
    ) -> GcStackReference<HostResource> {
        assert_eq!(self.import(), GcHostImport::WeakMapAllocate);
        self.emit_call_instruction(f);
        bind_resource(HostResourceKind::WeakMap, s, f)
    }
    pub(crate) fn allocate_weak_set(
        self,
        s: &RuntimeSchema,
        f: &mut Function,
    ) -> GcStackReference<HostResource> {
        assert_eq!(self.import(), GcHostImport::WeakSetAllocate);
        self.emit_call_instruction(f);
        bind_resource(HostResourceKind::WeakSet, s, f)
    }
    pub(crate) fn put_weak_map(
        self,
        owner: &GcLocal<WeakMapObject>,
        resource: &GcLocal<HostResource>,
        key: &ValueLocals,
        value: &GcLocal<StoredValue>,
        s: &RuntimeSchema,
        f: &mut Function,
    ) {
        assert_eq!(self.import(), GcHostImport::WeakMapSet);
        owner.load(s, f);
        resource_operand(resource, s, f);
        key.reference().load(f);
        value.load(s, f);
        self.emit_call_instruction(f);
    }
    pub(crate) fn add_weak_set(
        self,
        owner: &GcLocal<WeakSetObject>,
        resource: &GcLocal<HostResource>,
        key: &ValueLocals,
        s: &RuntimeSchema,
        f: &mut Function,
    ) {
        assert_eq!(self.import(), GcHostImport::WeakSetAdd);
        owner.load(s, f);
        resource_operand(resource, s, f);
        key.reference().load(f);
        self.emit_call_instruction(f);
    }
    pub(crate) fn get_weak_map(
        self,
        resource: &GcLocal<HostResource>,
        key: &ValueLocals,
        s: &RuntimeSchema,
        f: &mut Function,
    ) -> GcStackReference<StoredValue, Nullable> {
        assert_eq!(self.import(), GcHostImport::WeakMapGet);
        resource_operand(resource, s, f);
        key.reference().load(f);
        self.emit_call_instruction(f);
        f.instruction(&Instruction::RefCastNullable(
            s.reference_type::<StoredValue>(GcNullability::Nullable)
                .heap_type,
        ));
        GcStackReference::new()
    }
    pub(crate) fn weak_map_membership(
        self,
        resource: &GcLocal<HostResource>,
        key: &ValueLocals,
        result: I32Local,
        s: &RuntimeSchema,
        f: &mut Function,
    ) {
        assert!(matches!(
            self.import(),
            GcHostImport::WeakMapHas | GcHostImport::WeakMapDelete
        ));
        resource_operand(resource, s, f);
        key.reference().load(f);
        self.emit_call_instruction(f);
        result.store(f);
    }
    pub(crate) fn weak_set_membership(
        self,
        resource: &GcLocal<HostResource>,
        key: &ValueLocals,
        result: I32Local,
        s: &RuntimeSchema,
        f: &mut Function,
    ) {
        assert!(matches!(
            self.import(),
            GcHostImport::WeakSetHas | GcHostImport::WeakSetDelete
        ));
        resource_operand(resource, s, f);
        key.reference().load(f);
        self.emit_call_instruction(f);
        result.store(f);
    }
    pub(crate) fn allocate_weak_ref(
        self,
        owner: &GcLocal<WeakRefObject>,
        target: &ValueLocals,
        s: &RuntimeSchema,
        f: &mut Function,
    ) -> GcStackReference<HostResource> {
        assert_eq!(self.import(), GcHostImport::WeakRefAllocate);
        owner.load(s, f);
        target.reference().load(f);
        self.emit_call_instruction(f);
        bind_resource(HostResourceKind::WeakRef, s, f)
    }
    pub(crate) fn deref_weak_ref(
        self,
        resource: &GcLocal<HostResource>,
        result: &ValueLocals,
        s: &RuntimeSchema,
        f: &mut Function,
    ) {
        assert_eq!(self.import(), GcHostImport::WeakRefDeref);
        resource_operand(resource, s, f);
        self.emit_call_instruction(f);
        f.instruction(&Instruction::LocalSet(result.reference().index));
    }
}
