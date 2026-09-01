# Array and TypedArray `reverse` dispatch ownership

Status: implemented and dry-reviewed for the Wasm-AOT compiler on 2026-08-28.
Focused runtime and broader Array verification remain at the shared checkpoint.

## Distinct canonical algorithms

`compile_array_prototype_reverse_builtin` is the sole compiler owner for the
generic `Array.prototype.reverse` algorithm. It applies `ToObject`, obtains
`LengthOfArrayLike`, observes lower and upper property presence, conditionally
gets each value, performs the required strict writes or deletions, and returns
the converted receiver.

`compile_typed_array_prototype_reverse_builtin` is the distinct strict
TypedArray owner. It validates the TypedArray receiver, obtains a private-state
length witness, swaps integer-indexed elements, and never observes an own
`length` property. These algorithms are not interchangeable.

The deleted dense Array emitter remains absent. Neither direct call branch
implements element traversal, storage mutation, receiver conversion or error
policy.

## Closed static dispatch

The static `reverse()` seam locally owns a capability-free
`ReverseMethodDispatch::{TypedArrayCanonical, ArrayCanonical, GenericGetCall}`
authority. One precedence chain constructs exactly one state:

- a shape whose data property has the sole target
  `StandardBuiltinId::TypedArrayPrototypeReverse` selects the strict TypedArray
  owner first;
- a shape whose data property has the sole target
  `StandardBuiltinId::ArrayPrototypeReverse` selects the generic Array owner;
  and
- every unproven receiver selects ordinary property Get and Call fallthrough,
  preserving user-defined methods and accessors.

One exhaustive match consumes the decision and owns both canonical emissions
plus the no-emission fallthrough. The authority has no derives, wildcard arm,
default or independently consumable target booleans, so restoring unconditional
Array routing or letting two target tests choose separate calls requires
changing the closed construction and projection together.

Both optimized paths use `emit_array_direct_builtin_method_call`, so receiver
evaluation precedes complete left-to-right argument and spread evaluation,
which precedes the canonical call. Receiver classification is compile-time
metadata inspection and adds no observable operation.

The former unconditional branch always selected the Array owner from the
property spelling alone. It therefore sent `typedArray.reverse()` through an
algorithm that observes `length` as an ordinary property and could also bypass
an Array receiver's own `reverse` method. Array kind and Array heap shape are
deliberately not sufficient authority: property identity must be proven.

## Durable evidence

`array_reverse_algorithm_owner_structure.rs` recursively pins:

- the three-state capability-free authority, one producer and one exhaustive
  consumer for every state, and strict TypedArray-first precedence;
- exactly one direct producer for each canonical standard builtin;
- complete receiver and argument forwarding in both optimized paths;
- ordinary fallthrough for receivers matching neither proof;
- absence of the deleted dense reverse owner;
- one compiler and standard dispatcher consumer for each canonical algorithm;
  and
- the generic Array owner's lower/upper presence, Get, write and deletion
  order.

`wasm_array_reverse_own_method_dispatch.js` installs an own `reverse` method on
an Array, evaluates ordinary and spread arguments left to right, and requires
the custom call result, receiver identity and unchanged elements. It fails if
Array kind or shape bypasses the property through unconditional canonical
routing.

The existing `wasm_typedarray_prototype_reverse.js` fixture defines an own
throwing `length` getter on a TypedArray, then requires `reverse()` to use the
internal TypedArray length and swap the indexed values without invoking that
getter. Its exact CLI test is
`typed_array::run_wasm_backend_reverses_typedarray_in_place_and_returns_receiver`.
The exact Array override test is
`array::run_wasm_backend_calls_an_arrays_own_reverse_method`.

## Verification

The generic Array reverse body remains pinned at
`6bd42e25ba1e1235dd4f0a08d8df88c5891ed2d05b15e2667a55f6b7cbed7688` and
the strict TypedArray reverse body at
`f930f4c6b07e2729928bc23fea4268d8d09d6d197eda363b8aa8da1321c41a0e`.
The recursive source census contains one compiler and one dispatcher consumer
for each body, two direct producers in one ordered static branch, and no dense
reverse emitter. Targeted formatting, fixture syntax, the scoped diff check,
module-boundary audit and task-plan audit are green. The recursive structure
target passes `5/5`, both exact CLI controls pass `2/2`, and the shared
`cargo xc` checkpoint is green.

The pinned Array controls `get_if_present_with_delete.js`,
`S15.4.4.8_A4_T1.js` and `S15.4.4.8_A2_T2.js`, plus the TypedArray reverse
controls `get-length-uses-internal-arraylength.js`, `invoked-as-method.js` and
`this-is-not-typedarray-instance.js`, pass all `12/12` Wasm-AOT executions
with every failure bucket at zero.

## Nonclaims

This closure changes neither canonical algorithm, property installation,
Array exotic storage, another mutator, published conformance status nor a
Test262 materializer. It does not claim the broader Array or TypedArray trees
green.
