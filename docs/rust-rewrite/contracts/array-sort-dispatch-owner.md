# Array and TypedArray `sort` dispatch ownership

Status: implemented and dry-reviewed for the Wasm-AOT compiler on 2026-08-28.
Focused runtime and broader conformance verification remain at the shared
checkpoint.

## Distinct canonical algorithms

The private `compile_array_sort_with_output(ArraySortOutput::Receiver, ...)`
owns the generic `Array.prototype.sort` algorithm. The standard catalog reaches
it only through `compile_array_prototype_sort_builtin`. It applies `ToObject`,
observes the ordinary `length` property, preserves holes while collecting
values, compares default values as strings, publishes through ordinary or
integer-indexed writes, deletes trailing properties and returns the receiver.

`compile_typed_array_prototype_sort_builtin` and its private stable-sort body
own strict `TypedArray.prototype.sort`. They validate the TypedArray, obtain an
internal private-state length and element kind, apply numeric or BigInt default
ordering, publish integer-indexed values and never use an own `length` property
as the sort bound. The two owners are observably different and unchanged by
this routing closure.

The private `ArraySortOutput::{Receiver, Copy}` authority continues to own only
the generic `sort`/`toSorted` result policy behind two fixed entries. It is not
used to collapse the strict TypedArray algorithm into the generic owner.

## Closed static dispatch

The static `sort()` seam locally owns a capability-free
`SortMethodDispatch::{TypedArrayCanonical, ArrayCanonical, GenericGetCall}`
authority. One precedence chain constructs exactly one state:

- a data property whose sole target is
  `StandardBuiltinId::TypedArrayPrototypeSort` selects the strict TypedArray
  owner first;
- a data property whose sole target is
  `StandardBuiltinId::ArrayPrototypeSort` selects the generic Array owner; and
- an absent, accessor or ambiguous target selects ordinary property Get and
  Call fallthrough.

One exhaustive match owns both direct canonical emissions and the no-emission
fallthrough. The authority derives no capabilities and has no wildcard,
default, kind-only shortcut or independently consumable target Boolean. Array
kind and Array heap shape therefore cannot bypass an own `sort` override, and
adding another receiver route requires changing the closed producer and
consumer together.

Both canonical arms use `emit_array_direct_builtin_method_call`, retaining
receiver evaluation before complete left-to-right argument/spread evaluation
and the call. The generic fallthrough retains EvaluateCall property Get,
argument and Call behavior for user-defined methods and accessors.

## Durable evidence

`array_sort_dispatch_owner_structure.rs` recursively pins:

- the exact capability-free three-state domain, one producer and one exhaustive
  consumer per state, and strict TypedArray-first precedence;
- two singleton shape-target proofs and no kind or Array-shape shortcut;
- complete receiver and argument forwarding through exactly two canonical
  direct calls;
- one generic Array compiler and standard `Receiver` consumer;
- one strict TypedArray compiler and standard consumer;
- the generic owner's ordinary length and string-order boundaries; and
- the strict owner's private length witness and typed stable-sort boundary.

`wasm_array_sort_own_method_dispatch.js` installs an own `sort` method on an
Array, observes its receiver and ordinary/spread arguments, returns a custom
result and requires the elements to remain unchanged. It fails if Array kind or
shape restores unconditional canonical routing.

The existing `wasm_typedarray_prototype_sort.js` defines an own `length` of 50
on a six-element `Uint16Array`, requires that property to remain unchanged, and
requires numeric default order `1, 2, 3, 11, 22, 111`. It fails if the static
call enters generic Array string-order and ordinary-length semantics.

## Verification

The generic Array sort body remains pinned at
`20aa3a5afff0f855e5c574ba03d4fc38be8649be093faa091e99ee3c593a8ba2`.
The strict TypedArray sort entry and private stable-sort body remain pinned
together at
`0936699959dc0e3e55f343e7b37101ebf6d13d9ab9bb32cb0df6896e6c2c23b4`.
The recursive structure target passes `5/5`, and the two exact CLI controls
pass `2/2`. The shared `cargo xc`, formatting, fixture syntax, diff,
module-boundary and task-plan checks are green.

Pinned shared Test262 controls are Array
`precise-getter-deletes-successor.js`, `precise-comparefn-throws.js` and
`call-with-primitive.js`, plus TypedArray `arraylength-internal.js`,
`invoked-as-method.js` and `this-is-not-typedarray-instance.js`.
All six leaves pass their sloppy and strict variants for `12/12` Wasm-AOT
executions with every failure bucket at zero.

## Nonclaims

This closure changes neither canonical algorithm, property installation,
another Array method, published conformance status nor a Test262 materializer.
The later fixed-entry closure narrows `ArraySortOutput` visibility without
changing its variants or projections. Neither checkpoint claims the broader
Array or TypedArray trees green.
