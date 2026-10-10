# Array indexed storage in the Wasm GC backend

Status: the MAIN139 fresh-literal initialization changes are authored on
2026-10-05 and await whole-batch compilation and runtime verification.

ArrayObject retains one ArrayIndexStorage reference and an independent unsigned
Array length. ArrayCreate allocates eight empty hash buckets regardless of that
length. ArgumentsObject retains a separate geometrically grown descriptor table
with absent null slots and an independently sized ParameterMap; it is not an
alternative Array representation. There is no dense Array backing store, hole
bitmap, numeric semantic address or size threshold that changes Array behavior.

The only initial storage constructor creates a nonzero power-of-two bucket
table and zero occupied entries. Two registered private GC mutation helpers
publish and delete admitted indices from zero through 2^32-2. Lookup compares
complete index words in a strong reference chain. Replacing a descriptor leaves
the occupied count unchanged. A new node contains its complete descriptor
before its bucket edge and count are published. Deletion unlinks the node before
decrementing the count and dropping temporary roots.

Growth doubles the bucket count above 75% load; deletion halves it below 25%
load, with eight buckets as the minimum. Rehash captures each old NEXT edge
before relinking its node into the completed replacement table. No semantic
address or GC reference bits participate in hashing. Physical allocation limits
are runtime resource failures, not invented JavaScript index restrictions.

OwnKeys and ArraySetLength obtain one exact occupied-index snapshot. Its private
construction owner fills the numeric GC array and applies in-place heapsort
before publication. Published snapshots expose read and length operations only.
OwnKeys visits occupied indices in ascending order. Length reduction visits
them in descending order, deleting configurable properties and restoring length
to the first undeletable index plus one. A requested nonwritable length remains
nonwritable when that deletion fails. Sparse holes never size an allocation or
drive a length-reduction scan.

DefineOwnProperty retains the shared partial-descriptor validation and merge
owner. Prototype lookup, Proxy operations, accessors, nonextensibility and the
independent length-writable flag continue through the actual object algorithms.
Native RegExp, enumerable-property, argument-list and Promise results, frozen
template Arrays and Realm Array.prototype all publish through this storage.
Length changes stay explicit at their actual semantic producers.

Fixed Array literals have a private, non-Copy fresh initialization owner.
Its only constructor allocates the actual ArrayObject with the source Realm's
intrinsic Array prototype and checked logical length; there is no entry point
from an existing Array. The owner retains the array root and a single immutable
StoredValue for absent accessors until consuming finish publishes the complete
Value. Length is checked from usize into u32 before emission, so a non-hole
literal ordinal is at most 2^32-2 and never silently narrows to an index word.

Element evaluation remains left to right, followed by its abrupt-completion
check, complete descriptor construction and indexed publication. Every present
element gets standard writable/enumerable/configurable data attributes and the
whole stored value. Elisions publish nothing; their slots remain absent even
when a prototype supplies an inherited value, while an explicit undefined
remains an own property. An abrupt element leaves the destination unpublished
and prevents all later evaluations. The existing GC indexed publication helper
owns occupancy and hashing once, so each literal element emits no numeric key
formatting, generic DefineData dispatch or repeated property-bookkeeping body.
Spread and suspension-owned accumulation keep their existing observable path.
This does not add a dense backing representation or change physical storage
according to literal length.

The six finite Engine controls in aot_gc_sparse_array_storage.rs cover maximum
indices and length-only allocation, sorted occupied keys, replacement/unlink,
complete accessors and shrink rollback, inheritance/Proxy identity, native
result descriptors, and fresh literal lifecycle. The fresh literal control
checks whole values, holes versus own undefined, descriptor and length flags,
prototype setter and public Reflect poisoning, evaluation and abrupt order,
and the intrinsic source Realm after replacing the global Array constructor.
Each control covers strict and nonstrict source. The new control remains unrun;
source completion does not establish whole-batch or Test262 readiness. The
existing 256-byte per-element density limit remains unchanged.
