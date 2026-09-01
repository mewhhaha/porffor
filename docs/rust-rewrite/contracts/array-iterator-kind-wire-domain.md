# Array iterator kind wire domain

Status: implemented for the ordinary Array-like and private TypedArray iterator
records.

## Boundary

`ArrayIteratorKind::{Key, Value, KeyAndValue}` is the sole Rust authority for
the three stable stored words. Its macro owns the complete row list, the word
projection and a compile-time uniqueness assertion. The type intentionally has
no clone, copy, debug, equality or default capability. Constructors accept a
borrowed kind and serialize its word only when storing the ordinary named slot
or the private TypedArray record.

The ordinary iterator carrier remains an own named property. Its `next` path
therefore compares the Number exactly with every stable row and treats a
non-number, fractional, non-finite, negative or unknown value as an
incompatible-receiver boundary error. The TypedArray carrier is a private
internal record; an unknown word there violates a compiler invariant and emits
Wasm `unreachable`. Both valid-word consumers walk `ArrayIteratorKind::ALL` and
use an exhaustive Rust match for key, value and key-value-pair behavior. No
unknown value defaults to value iteration.

Static method-call lowering parses exactly `keys`, `values` and `entries` into
the closed `StaticArrayIteratorMethod` domain. Its exhaustive projection
selects an `ArrayIteratorKind`; every other spelling remains an ordinary
source property call and is not an iterator producer.

## Durable evidence

`array_iterator_kind_wire_domain_structure.rs` pins the exact three rows and
stable words, absence of convenience capabilities and raw constants, borrowed
typed constructors, all producer sites, both `ALL` decoders, all three
exhaustive semantic arms, and the distinct ordinary-error/private-trap invalid
word policies.

The retired static-generator backend absence target additionally prevents a
fourth synthetic method or marker protocol from being attached to this domain.

The existing `wasm_array_iterator_receiver_policy.js` and
`wasm_typedarray_iterators.js` fixtures cover all ordinary and TypedArray
producer kinds and terminal results. The selected pinned Test262 leaves are the
three Array `iteration.js` cases and the three TypedArray `return-itor.js`
cases for `keys`, `values` and `entries`.

## Verification

On 2026-08-27, the bounded structure target passed `5/5`, the generic Array
iterator-policy and strict TypedArray CLI fixtures passed `2/2`, and the six
selected pinned Test262 leaves passed both ordinary and strict variants
(`12/12`) with every failure bucket at zero. The strict fixture initially
exposed a separate `%TypedArray%` prototype-publication regression, which was
repaired without weakening descriptor validation and is covered by its own
contract. `cargo xc`, formatting, diff, task-plan and module-boundary checks are
green. The following shared 683-dump semantic golden passed `2/2` in 655.10
seconds, adding and removing none. The expanded Array fixture is its sole
retained non-accounting change; the other 682 retained summaries differ only in
accounting fields. This migration changes no published conformance count.

## Deferrals

This domain does not complete iterator closing, generator state machines,
Array exotic semantics, resizable or detached TypedArray buffers, or the T15,
T16 or T17 conformance surfaces.
