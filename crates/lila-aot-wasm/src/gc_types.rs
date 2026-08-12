//! Typed schema vocabulary for the Wasm-GC object model.
//!
//! Encoding stays in the central module type registry; this module supplies
//! the typed schema it consumes. T05's object-model cutover must be atomic:
//! until that cutover, JavaScript objects remain on the existing linear-memory
//! path. These types let the emitter describe the replacement without
//! representing a GC reference as an integer or confusing a linear-memory
//! address with one.

#![allow(
    dead_code,
    reason = "T05 schema precedes its atomic semantic-object cutover"
)]

use core::marker::PhantomData;

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
    pub(crate) const fn new(raw: u32) -> Self {
        Self {
            raw,
            ty: PhantomData,
        }
    }

    pub(crate) const fn raw(self) -> u32 {
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
    pub(crate) const fn new(raw: u32) -> Self {
        Self(raw)
    }

    pub(crate) const fn raw(self) -> u32 {
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
    pub(crate) const fn new(ordinal: GcFieldOrdinal) -> Self {
        Self {
            ordinal,
            shape: PhantomData,
        }
    }

    pub(crate) const fn ordinal(self) -> GcFieldOrdinal {
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
    pub(crate) const ABI_VERSION: i32 = 1;
    pub(crate) const FIELD_COUNT: u32 = 1;
    pub(crate) const ABI_VERSION_FIELD: GcField<RuntimeGcAnchor, I32Value, Immutable, NonNullable> =
        GcField::new(GcFieldOrdinal::new(0));

    pub(crate) const fn new(type_index: GcTypeIndex<RuntimeGcAnchor>) -> Self {
        Self { type_index }
    }

    pub(crate) const fn type_index(self) -> GcTypeIndex<RuntimeGcAnchor> {
        self.type_index
    }
}

/// Assigned type index plus the fixed schema of [`RuntimeGcAnchorHolder`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RuntimeGcAnchorHolderSchema {
    type_index: GcTypeIndex<RuntimeGcAnchorHolder>,
}

impl RuntimeGcAnchorHolderSchema {
    pub(crate) const FIELD_COUNT: u32 = 1;
    pub(crate) const ANCHOR_FIELD: GcField<
        RuntimeGcAnchorHolder,
        GcRef<RuntimeGcAnchor>,
        Immutable,
        NonNullable,
    > = GcField::new(GcFieldOrdinal::new(0));

    pub(crate) const fn new(type_index: GcTypeIndex<RuntimeGcAnchorHolder>) -> Self {
        Self { type_index }
    }

    pub(crate) const fn type_index(self) -> GcTypeIndex<RuntimeGcAnchorHolder> {
        self.type_index
    }
}

const _: () = assert!(RuntimeGcAnchorSchema::FIELD_COUNT == 1);
const _: () = assert!(RuntimeGcAnchorSchema::ABI_VERSION_FIELD.ordinal().raw() == 0);
const _: () = assert!(RuntimeGcAnchorHolderSchema::FIELD_COUNT == 1);
const _: () = assert!(RuntimeGcAnchorHolderSchema::ANCHOR_FIELD.ordinal().raw() == 0);
