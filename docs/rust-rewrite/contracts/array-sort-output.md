# Array sort output domain

Status: implemented as a source-equivalent Wasm-AOT invariant boundary.

## Closed policy

`Array.prototype.sort` and `Array.prototype.toSorted` share receiver conversion,
length observation, element collection, comparison and stable sorting. Their
output behavior is selected by the private two-row
`ArraySortOutput::{Receiver, Copy}` domain. This is a private, non-derived domain
whose raw consumer cannot be called outside `array.rs`.

The standard dispatcher can call only two fixed entries:
`compile_array_prototype_sort_builtin` and
`compile_array_prototype_to_sorted_builtin`. It cannot name the output domain,
select a variant or call `compile_array_sort_with_output` directly.

The domain derives no capabilities. Its one typed consumer borrows it in four
exhaustive matches that own every output-sensitive decision:

- `Copy` validates the Array length and allocates the result, while `Receiver`
  keeps the original receiver as the result;
- `Copy` reads every indexed property so holes become `undefined`, while
  `Receiver` preserves the existing `HasProperty` policy;
- `Copy` publishes sorted entries into the new Array, while `Receiver` uses the
  existing ordinary-object or TypedArray write path; and
- `Receiver` deletes trailing source properties, while `Copy` performs no
  source deletion.

There is no equality, Boolean, wildcard, default or unreachable projection.
Adding another output therefore requires an explicit choice at all four
semantic boundaries.

## Preserved behavior

The change retains the existing validation, instruction and local ordering.
In particular, the `toSorted` Array-length error remains before indexed reads;
collection still precedes comparison; publication still precedes the
Receiver-only deletion pass; and result publication and reverse local release
remain common. No runtime tag, Wasm word or JavaScript-visible representation
was added.

## Durable evidence

`array_sort_output_structure.rs` pins the two-row non-capable domain, the two
fixed entries and catalog routes, the recursive production-source census, all
four borrowed exhaustive projections and their distinct allocation, presence,
publication and deletion policies.

The existing Array sort CLI fixtures cover stable sorting, inherited and
sparse entries, Proxy operation order, receiver identity, trailing deletion
and abrupt writes. Existing focused Test262 witnesses include
`sort/precise-prototype-element.js`,
`sort/precise-getter-deletes-successor.js`,
`toSorted/holes-not-preserved.js`, `toSorted/frozen-this-value.js`,
`toSorted/comparefn-called-after-get-elements.js` and
`toSorted/length-exceeding-array-length-limit.js`.

The earlier semantic checkpoint's exact sort core, observability and error CLI
fixtures pass `3/3`. Its six focused Test262 leaves pass all `12/12`
sloppy/strict Wasm-AOT executions with every failure bucket at zero.

## Fixed-dispatch source equivalence

No instruction-emitting statement changed. Restoring only the former crate
visibility produces the exact original four-line output-domain selection with
SHA-256
`1745b093aab4e0643c08de0b1d402f3770ef5a9618635ae7b31ec318a8c74c4c`.
Restoring the former visibility and name of the private 604-line raw algorithm
produces its exact original SHA-256
`aa8c4c988b2c5e64568cfc9f4a294c98a32144af941450cf59ac882948afbf25`.

At the fixed-dispatch checkpoint, `cargo xc` is green, the strengthened output
target passes `4/4`, its dispatch-owner neighbor passes `5/5`, and the three
exact Array sort CLI controls pass `3/3`. Formatting, module-boundary,
task-plan and exact Test262 shortcut gates are green.

## Scope

This source-equivalent closure adds no fixture and has no new Array behavior.
It changes no sorting, comparison, Array exotic, TypedArray, species, Realm or
error semantics. It makes no broad Array/Test262 or published-status claim and
does not close T16.
