//! Typed schema vocabulary for the Wasm-GC object model.
//!
//! This module owns the typed schema and the final raw Wasm-GC encoding
//! boundary. Registration borrows the central module sections, while function
//! emission receives only opaque lifecycle operations. T05's object-model
//! cutover must be atomic: until that cutover, JavaScript objects remain on the
//! existing linear-memory path. These types let the emitter describe the
//! replacement without representing a GC reference as an integer or confusing
//! a linear-memory address with one.

#![allow(
    dead_code,
    reason = "T05 schema precedes its atomic semantic-object cutover"
)]

use core::marker::PhantomData;

use wasm_encoder::{
    BlockType, ConstExpr, Encode, FieldType, GlobalSection, GlobalType, HeapType, Instruction,
    RefType, Section, StorageType, TypeSection, ValType,
};

use crate::Function;

mod sealed {
    pub trait Sealed {}
}

/// Marker implemented only by layouts in the central Wasm-GC schema.
///
/// Keeping this sealed makes the schema the exhaustive source of heap types;
/// an emitter submodule cannot silently invent an unregistered layout.
pub(crate) trait GcHeapType: sealed::Sealed + Copy + 'static {}

/// A type-section index that can name only `T`'s declared Wasm-GC type.
///
/// The raw index remains available at the final `wasm-encoder` boundary, but
/// it cannot be exchanged with another heap type's index before that point.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub(crate) struct GcTypeIndex<T: GcHeapType> {
    raw: u32,
    ty: PhantomData<fn() -> T>,
}

impl<T: GcHeapType> GcTypeIndex<T> {
    const fn new(raw: u32) -> Self {
        Self {
            raw,
            ty: PhantomData,
        }
    }

    const fn raw(self) -> u32 {
        self.raw
    }
}

impl<T: GcHeapType> Clone for GcTypeIndex<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: GcHeapType> Copy for GcTypeIndex<T> {}

/// A field's declaration-order index within one GC struct type.
///
/// This is distinct from [`GcTypeIndex`], so a type index cannot accidentally
/// be passed to a `struct.get`/`struct.set` field position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub(crate) struct GcFieldOrdinal(u32);

impl GcFieldOrdinal {
    const fn new(raw: u32) -> Self {
        Self(raw)
    }

    const fn raw(self) -> u32 {
        self.0
    }
}

pub(crate) trait GcFieldMutability: sealed::Sealed + Copy + 'static {
    const MUTABLE: bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Immutable {}

impl sealed::Sealed for Immutable {}
impl GcFieldMutability for Immutable {
    const MUTABLE: bool = false;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mutable {}

impl sealed::Sealed for Mutable {}
impl GcFieldMutability for Mutable {
    const MUTABLE: bool = true;
}

pub(crate) trait GcFieldNullability: sealed::Sealed + Copy + 'static {
    const NULLABLE: bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NonNullable {}

impl sealed::Sealed for NonNullable {}
impl GcFieldNullability for NonNullable {
    const NULLABLE: bool = false;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Nullable {}

impl sealed::Sealed for Nullable {}
impl GcFieldNullability for Nullable {
    const NULLABLE: bool = true;
}

/// Scalar Wasm storage markers used by GC fields.
///
/// They are types rather than an enum value so an individual [`GcField`] owns
/// its storage contract at compile time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum I32Value {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum I64Value {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum F64Value {}

impl sealed::Sealed for I32Value {}
impl sealed::Sealed for I64Value {}
impl sealed::Sealed for F64Value {}

/// The storage-type witness for a strong typed Wasm-GC reference.
///
/// This is intentionally a zero-sized schema marker, not a `u32`/`u64`
/// handle. Wasm references must remain references in emitted code. There is no
/// weak counterpart: the current Wasm-GC lower bound has no weak reference or
/// ephemeron storage type, so spelling one here would be a false capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GcRef<T: GcHeapType>(PhantomData<fn() -> T>);

impl<T: GcHeapType> sealed::Sealed for GcRef<T> {}

/// The index of one mutable, nullable Wasm global that roots a strong GC
/// reference to `T`.
///
/// The nullable state is the lifecycle boundary: the global is null before
/// main establishes the root and after every shared main exit clears it. This
/// type names the global slot only; it cannot contain a reference value, a
/// linear-memory address, or the index of a scalar global.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub(crate) struct GcRootGlobal<T: GcHeapType> {
    raw: u32,
    target: PhantomData<fn() -> T>,
}

impl<T: GcHeapType> GcRootGlobal<T> {
    const fn new(raw: u32) -> Self {
        Self {
            raw,
            target: PhantomData,
        }
    }

    const fn raw(self) -> u32 {
        self.raw
    }
}

impl<T: GcHeapType> Clone for GcRootGlobal<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: GcHeapType> Copy for GcRootGlobal<T> {}

/// A memory32 byte address owned by a particular GC layout.
///
/// The owner parameter prevents side-storage addresses for two layouts from
/// being exchanged. It does not make the address a JavaScript object identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub(crate) struct LinearAddr<Owner: GcHeapType> {
    raw: u32,
    owner: PhantomData<fn() -> Owner>,
}

impl<Owner: GcHeapType> LinearAddr<Owner> {
    pub(crate) const fn new(raw: u32) -> Self {
        Self {
            raw,
            owner: PhantomData,
        }
    }

    pub(crate) const fn raw(self) -> u32 {
        self.raw
    }
}

impl<Owner: GcHeapType> sealed::Sealed for LinearAddr<Owner> {}

/// A validated memory32 byte span with one statically named GC owner.
///
/// Fields are private and construction checks the one-past-end address, so a
/// span cannot wrap memory32. The `2^32` one-past-end value is valid even
/// though it does not fit in [`LinearAddr`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LinearSpan<Owner: GcHeapType> {
    start: LinearAddr<Owner>,
    byte_len: u32,
}

impl<Owner: GcHeapType> LinearSpan<Owner> {
    const MEMORY32_END: u64 = u32::MAX as u64 + 1;

    pub(crate) const fn new(start: LinearAddr<Owner>, byte_len: u32) -> Option<Self> {
        let end = start.raw() as u64 + byte_len as u64;
        if end <= Self::MEMORY32_END {
            Some(Self { start, byte_len })
        } else {
            None
        }
    }

    pub(crate) const fn start(self) -> LinearAddr<Owner> {
        self.start
    }

    pub(crate) const fn byte_len(self) -> u32 {
        self.byte_len
    }

    pub(crate) const fn end_exclusive(self) -> u64 {
        self.start.raw() as u64 + self.byte_len as u64
    }
}

/// Sealed relation between a field owner, its value, and nullability.
///
/// Scalar and linear-address fields are non-nullable. Only [`GcRef`] admits
/// [`Nullable`], so `GcField<_, I64Value, _, Nullable>` does not type-check.
pub(crate) trait GcFieldValue<Owner, Nullability>: sealed::Sealed
where
    Owner: GcHeapType,
    Nullability: GcFieldNullability,
{
}

impl<Owner: GcHeapType> GcFieldValue<Owner, NonNullable> for I32Value {}
impl<Owner: GcHeapType> GcFieldValue<Owner, NonNullable> for I64Value {}
impl<Owner: GcHeapType> GcFieldValue<Owner, NonNullable> for F64Value {}
impl<Owner: GcHeapType> GcFieldValue<Owner, NonNullable> for LinearAddr<Owner> {}
impl<Owner: GcHeapType, Target: GcHeapType> GcFieldValue<Owner, NonNullable> for GcRef<Target> {}
impl<Owner: GcHeapType, Target: GcHeapType> GcFieldValue<Owner, Nullable> for GcRef<Target> {}

/// One field in one declared Wasm-GC struct.
///
/// `Owner`, `Value`, `Mutability`, and `Nullability` are all part of the type;
/// the encoder can therefore accept exactly the field shape an operation is
/// valid for instead of accepting four independent booleans and integers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GcField<Owner, Value, Mutability, Nullability>
where
    Owner: GcHeapType,
    Value: GcFieldValue<Owner, Nullability>,
    Mutability: GcFieldMutability,
    Nullability: GcFieldNullability,
{
    ordinal: GcFieldOrdinal,
    shape: PhantomData<fn() -> (Owner, Value, Mutability, Nullability)>,
}

impl<Owner, Value, Mutability, Nullability> GcField<Owner, Value, Mutability, Nullability>
where
    Owner: GcHeapType,
    Value: GcFieldValue<Owner, Nullability>,
    Mutability: GcFieldMutability,
    Nullability: GcFieldNullability,
{
    const fn new(ordinal: GcFieldOrdinal) -> Self {
        Self {
            ordinal,
            shape: PhantomData,
        }
    }

    const fn ordinal(self) -> GcFieldOrdinal {
        self.ordinal
    }
}

/// The first GC type in the migration: a capability/ABI witness only.
///
/// It carries no JavaScript object and therefore does not create a second live
/// object model while the linear heap is still active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeGcAnchor {}

impl sealed::Sealed for RuntimeGcAnchor {}
impl GcHeapType for RuntimeGcAnchor {}

/// A capability-only holder with one strong edge to [`RuntimeGcAnchor`].
///
/// Like the anchor, this is not a JavaScript object. It exists to make the
/// first executable reference-bearing field exercise the same typed schema
/// path future semantic layouts will use without creating a second object
/// model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeGcAnchorHolder {}

impl sealed::Sealed for RuntimeGcAnchorHolder {}
impl GcHeapType for RuntimeGcAnchorHolder {}

/// Assigned type index plus the fixed schema of [`RuntimeGcAnchor`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RuntimeGcAnchorSchema {
    type_index: GcTypeIndex<RuntimeGcAnchor>,
}

impl RuntimeGcAnchorSchema {
    /// Bumped only when the emitted GC value ABI changes incompatibly.
    const ABI_VERSION: i32 = 1;
    const FIELD_COUNT: u32 = 1;
    const ABI_VERSION_FIELD: GcField<RuntimeGcAnchor, I32Value, Immutable, NonNullable> =
        GcField::new(GcFieldOrdinal::new(0));

    const fn new(type_index: GcTypeIndex<RuntimeGcAnchor>) -> Self {
        Self { type_index }
    }

    const fn type_index(self) -> GcTypeIndex<RuntimeGcAnchor> {
        self.type_index
    }
}

/// Assigned type index plus the fixed schema of [`RuntimeGcAnchorHolder`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RuntimeGcAnchorHolderSchema {
    type_index: GcTypeIndex<RuntimeGcAnchorHolder>,
}

impl RuntimeGcAnchorHolderSchema {
    const FIELD_COUNT: u32 = 1;
    const ANCHOR_FIELD: GcField<
        RuntimeGcAnchorHolder,
        GcRef<RuntimeGcAnchor>,
        Immutable,
        NonNullable,
    > = GcField::new(GcFieldOrdinal::new(0));

    const fn new(type_index: GcTypeIndex<RuntimeGcAnchorHolder>) -> Self {
        Self { type_index }
    }

    const fn type_index(self) -> GcTypeIndex<RuntimeGcAnchorHolder> {
        self.type_index
    }
}

/// The runtime-visible GC portion of the module's central type registry.
///
/// Registration and global-section finalization are the only operations
/// exposed to module assembly. Raw type indices and field ordinals never leave
/// this module, so a caller cannot guess an index or pair a field with a
/// different owner before encoding. Finalizing the complete scalar global
/// section derives and appends the sole typed root, then keeps its construction
/// and extraction private.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct RuntimeModuleTypes {
    gc_anchor: RuntimeGcAnchorSchema,
    gc_anchor_holder: RuntimeGcAnchorHolderSchema,
}

impl RuntimeModuleTypes {
    pub(crate) fn register(types: &mut TypeSection) -> Self {
        let gc_anchor = RuntimeGcAnchorSchema::new(gc_struct_with_i32_field(
            types,
            RuntimeGcAnchorSchema::ABI_VERSION_FIELD,
        ));
        let gc_anchor_holder = RuntimeGcAnchorHolderSchema::new(gc_struct_with_ref_field(
            types,
            RuntimeGcAnchorHolderSchema::ANCHOR_FIELD,
            gc_anchor.type_index(),
        ));

        Self {
            gc_anchor,
            gc_anchor_holder,
        }
    }

    /// Consumes the complete open global section, derives the root from its
    /// actual next index, appends it, and seals the section together with the
    /// only matching runtime schema.
    pub(crate) fn finalize_globals(self, mut globals: GlobalSection) -> FinalizedModuleGlobals {
        let runtime_schema = RuntimeModuleSchema {
            types: self,
            gc_anchor_root: GcRootGlobal::new(globals.len()),
        };
        runtime_schema.append_root_global(&mut globals);
        FinalizedModuleGlobals {
            section: globals,
            runtime_schema,
        }
    }
}

/// Complete, opaque runtime GC schema carried only by the main-function role.
///
/// The anchor type is stored once in `types`; the root adds only its typed
/// global index. Declaration, initialization and cleanup therefore cannot
/// acquire two independently supplied indices for the same anchor layout.
#[derive(Debug, PartialEq, Eq)]
struct RuntimeModuleSchema {
    types: RuntimeModuleTypes,
    gc_anchor_root: GcRootGlobal<RuntimeGcAnchor>,
}

impl RuntimeModuleSchema {
    /// Appends the sole runtime GC root after every previously registered
    /// global. Only [`RuntimeModuleTypes::finalize_globals`] can bind and invoke
    /// this operation, so module assembly cannot encode the root's raw type or
    /// index or append another global after it.
    fn append_root_global(&self, globals: &mut GlobalSection) {
        assert_eq!(
            globals.len(),
            self.gc_anchor_root.raw(),
            "runtime GC root must be appended after every pre-existing global"
        );
        let anchor_type = HeapType::Concrete(self.types.gc_anchor.type_index().raw());
        globals.global(
            GlobalType {
                val_type: ValType::Ref(RefType {
                    nullable: true,
                    heap_type: anchor_type,
                }),
                mutable: true,
                shared: false,
            },
            &ConstExpr::ref_null(anchor_type),
        );
    }

    /// Constructs the capability anchor and holder, traverses the holder's
    /// typed strong edge and establishes the main-lifetime root.
    fn emit_initialize_anchor_root(&self, function: &mut Function) {
        function.instruction(&Instruction::I32Const(RuntimeGcAnchorSchema::ABI_VERSION));
        emit_struct_new(function, self.types.gc_anchor.type_index());
        emit_struct_new(function, self.types.gc_anchor_holder.type_index());
        emit_struct_get(
            function,
            self.types.gc_anchor_holder.type_index(),
            RuntimeGcAnchorHolderSchema::ANCHOR_FIELD,
        );
        emit_root_set(function, self.gc_anchor_root);
    }

    /// Verifies the capability anchor ABI and clears the main-lifetime root.
    fn emit_verify_and_clear_anchor_root(&self, function: &mut Function) {
        emit_root_get(function, self.gc_anchor_root);
        function.instruction(&Instruction::RefAsNonNull);
        emit_struct_get(
            function,
            self.types.gc_anchor.type_index(),
            RuntimeGcAnchorSchema::ABI_VERSION_FIELD,
        );
        function.instruction(&Instruction::I32Const(RuntimeGcAnchorSchema::ABI_VERSION));
        function.instruction(&Instruction::I32Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        emit_ref_null(function, self.types.gc_anchor.type_index());
        emit_root_set(function, self.gc_anchor_root);
    }
}

/// A global section sealed after the runtime root, paired with the only schema
/// whose typed global index names that section.
///
/// The raw section and schema are both private. This value implements the Wasm
/// section traits itself, and main holds a reference to this exact package for
/// its opaque lifecycle operations. No caller can clone the raw
/// [`GlobalSection`], append another global, extract a copyable schema, or pair
/// lifecycle instructions from one package with another package's section.
pub(crate) struct FinalizedModuleGlobals {
    section: GlobalSection,
    runtime_schema: RuntimeModuleSchema,
}

impl FinalizedModuleGlobals {
    pub(crate) fn emit_initialize_anchor_root(&self, function: &mut Function) {
        self.runtime_schema.emit_initialize_anchor_root(function);
    }

    pub(crate) fn emit_verify_and_clear_anchor_root(&self, function: &mut Function) {
        self.runtime_schema
            .emit_verify_and_clear_anchor_root(function);
    }
}

impl Encode for FinalizedModuleGlobals {
    fn encode(&self, sink: &mut Vec<u8>) {
        self.section.encode(sink);
    }
}

impl Section for FinalizedModuleGlobals {
    fn id(&self) -> u8 {
        self.section.id()
    }
}

fn gc_struct_with_i32_field<T, Mutability>(
    types: &mut TypeSection,
    field: GcField<T, I32Value, Mutability, NonNullable>,
) -> GcTypeIndex<T>
where
    T: GcHeapType,
    Mutability: GcFieldMutability,
{
    assert_eq!(
        field.ordinal().raw(),
        0,
        "a one-field GC struct must declare field ordinal zero"
    );
    let index = GcTypeIndex::new(types.len());
    types.ty().struct_([FieldType {
        element_type: StorageType::Val(ValType::I32),
        mutable: Mutability::MUTABLE,
    }]);
    index
}

fn gc_struct_with_ref_field<Owner, Target, Mutability, Nullability>(
    types: &mut TypeSection,
    field: GcField<Owner, GcRef<Target>, Mutability, Nullability>,
    target: GcTypeIndex<Target>,
) -> GcTypeIndex<Owner>
where
    Owner: GcHeapType,
    Target: GcHeapType,
    Mutability: GcFieldMutability,
    Nullability: GcFieldNullability,
    GcRef<Target>: GcFieldValue<Owner, Nullability>,
{
    assert_eq!(
        field.ordinal().raw(),
        0,
        "a one-field GC struct must declare field ordinal zero"
    );
    let index = GcTypeIndex::new(types.len());
    types.ty().struct_([FieldType {
        element_type: StorageType::Val(ValType::Ref(RefType {
            nullable: Nullability::NULLABLE,
            heap_type: HeapType::Concrete(target.raw()),
        })),
        mutable: Mutability::MUTABLE,
    }]);
    index
}

fn emit_struct_new<T: GcHeapType>(function: &mut Function, ty: GcTypeIndex<T>) {
    function.instruction(&Instruction::StructNew(ty.raw()));
}

fn emit_struct_get<Owner, Value, Mutability, Nullability>(
    function: &mut Function,
    owner: GcTypeIndex<Owner>,
    field: GcField<Owner, Value, Mutability, Nullability>,
) where
    Owner: GcHeapType,
    Value: GcFieldValue<Owner, Nullability>,
    Mutability: GcFieldMutability,
    Nullability: GcFieldNullability,
{
    function.instruction(&Instruction::StructGet {
        struct_type_index: owner.raw(),
        field_index: field.ordinal().raw(),
    });
}

fn emit_root_get<T: GcHeapType>(function: &mut Function, root: GcRootGlobal<T>) {
    function.instruction(&Instruction::GlobalGet(root.raw()));
}

fn emit_root_set<T: GcHeapType>(function: &mut Function, root: GcRootGlobal<T>) {
    function.instruction(&Instruction::GlobalSet(root.raw()));
}

fn emit_ref_null<T: GcHeapType>(function: &mut Function, ty: GcTypeIndex<T>) {
    function.instruction(&Instruction::RefNull(HeapType::Concrete(ty.raw())));
}

const _: () = assert!(RuntimeGcAnchorSchema::FIELD_COUNT == 1);
const _: () = assert!(RuntimeGcAnchorSchema::ABI_VERSION_FIELD.ordinal().raw() == 0);
const _: () = assert!(RuntimeGcAnchorHolderSchema::FIELD_COUNT == 1);
const _: () = assert!(RuntimeGcAnchorHolderSchema::ANCHOR_FIELD.ordinal().raw() == 0);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finalized_globals_bind_the_root_to_the_actual_next_index() {
        for existing_global_count in [0_u32, 1, 7] {
            let mut types = TypeSection::new();
            let runtime = RuntimeModuleTypes::register(&mut types);
            let mut globals = GlobalSection::new();
            for _ in 0..existing_global_count {
                globals.global(
                    GlobalType {
                        val_type: ValType::I64,
                        mutable: true,
                        shared: false,
                    },
                    &ConstExpr::i64_const(0),
                );
            }

            let finalized = runtime.finalize_globals(globals);
            assert_eq!(
                finalized.runtime_schema.gc_anchor_root.raw(),
                existing_global_count,
                "the typed root must bind the encoded section's actual next index"
            );
            assert_eq!(
                finalized.section.len(),
                existing_global_count + 1,
                "finalization must append exactly one root"
            );
        }
    }
}
