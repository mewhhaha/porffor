# TypedArray iterator buffer witness

Status: typed GC source migration; compilation and execution pending.

Each active TypedArray iterator next borrows its actual `TypedArrayObject` and
calls the shared `emit_validate_typed_array_view` owner. That owner acquires
one backing-store observation and derives a whole-element length, distinguishing
detached and out-of-bounds views with a whole defining-function Realm TypeError.
The Array iterator next entry uses the same validation when its retained
Array-like receiver is a concrete TypedArray. Native TypedArray producer entry
retains its required initial validation before record construction.

Validation occurs before the done test and index mutation. Failure leaves the
retained array and index unchanged, permitting a fixed view to resume at the
same index after regrowth. Successful exhaustion clears the retained typed
array and makes Done permanent; later growth cannot restart it. Element reads
consume the existing typed element/buffer owner. There is no `TypedArrayViewLocals`
raw slot reconstruction, manual heap pointer or independent cached-length policy.

The new `aot_gc_iterator_entries.rs` cases cover detach, fixed-view shrink/error/
regrow, length-tracking growth/shrink, entries, and permanent completion. Existing
TypedArray CLI matrices and their Realm controls remain unchanged. The former
`typed_array_iterator_witness_structure.rs` raw representation mirror retires.

The [current next algorithm](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-%arrayiteratorprototype%.next)
requires a fresh bounds check on each active step. Prior focused iterator passes
are historical evidence for the old representation. The GC batch has no new
compile, runtime or Test262 result yet, and this contract makes no T17 closure
or conformance-count claim.
