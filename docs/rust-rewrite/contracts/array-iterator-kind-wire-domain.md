# Array iterator kind domain

Status: typed GC source migration; compilation and execution pending.

`gc_types::ArrayIterationKind::{Key, Value, KeyAndValue}` is the single closed
kind authority for both `ArrayIteratorObject` and `TypedArrayIteratorObject`.
The GC schema stores the declared constant codec in a private immutable field.
Creation receives this enum directly. The former independent `ArrayIteratorKind`
word domain and JS-visible Number carrier are retired, so user properties cannot
change the internal iteration kind or forge an iterator receiver.

Both concrete next consumers select the three codec rows and exhaustively
match their semantics. Keys publish an index Number without element Get;
values publish the actual element; entries publish the real two-element Array.
An unknown internal code is a compiler invariant violation, never a default
value-iteration branch. No source-name call shortcut is introduced.

The old `array_iterator_kind_wire_domain_structure.rs` raw-spelling mirror is
retired. Existing keys/values/entries CLI controls are retained, and the new GC
iterator control adds a live entries result while preserving all old targets.
The earlier 2026-08-27 structure, CLI, pinned leaf and golden results belong to
the predecessor representation and remain historical evidence.

The new source is not compiled or executed during the all-task dry phase. It
changes no published conformance count and does not claim iterator closing,
generator completion, or complete Array/TypedArray conformance.
