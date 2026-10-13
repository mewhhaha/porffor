# Iterator includes and join

The frozen current-pin baseline exposes 112 failing executions from two absent
methods: 40 includes sources and 16 join sources, each in sloppy and strict mode.
Both methods now have distinct builtin identities, realm-owned prototype
properties, normal function metadata, IR shapes and native Wasm bodies.

`join` follows `sec-iterator.prototype.join` in ECMA262 revision
`5345883164f463e87f8b40aca4956157ecba8783`. The retained spec HTML SHA-256 is
`5194892f5a5f595962929b7099e028b1db44c8b68c44f4ce20f77b266507d57d`.
Separator conversion precedes `GetIteratorDirect`; its failure closes the
provisional receiver. Iterator-step failures propagate without a further close.
Element conversion failures close the captured iterator, preserving the original
throw. Undefined and null contribute empty strings; other values use live
ToString, with UTF-16 concatenation and a separator between each pair of values.

`includes` follows the stage-three
[Iterator Includes proposal](https://github.com/tc39/proposal-iterator-includes/blob/63a7f4c95d84240f57a5cf79beca0eb2cd8911f3/spec.emu),
revision `63a7f4c95d84240f57a5cf79beca0eb2cd8911f3`, whose `spec.emu` SHA-256 is
`c1ca0b6a4b652aeb1cdfb39f87d05859a37754e68c3bfdaa6f332ae717dc9b08`.
This proposal remains distinct from the pinned ECMAScript standard. The pinned
Test262 fixtures name the `iterator-includes` feature and use the same algorithm.
Its optional skippedElements accepts undefined, integral Numbers and infinities
without coercion. Negative values and finite values above `2**53 - 1` are range
errors; all other invalid values are type errors. Argument validation closes the
provisional receiver before reading next, retaining its original error.
Iteration uses cached next and `IteratorStepValue`, including the value getters
of skipped elements. SameValueZero matches close with NormalCompletion(true),
so close failures remain observable. Exhaustion returns false without closing.

Both implementations reuse the existing typed IteratorRecord and completion
protocol. They keep receiver validation before argument processing and create
errors in the called intrinsic's realm. No fixture, deadline or runner outcome is
changed. Native strict/sloppy controls cover metadata, equality, skip boundaries,
live coercions, next/value effects, UTF-16 and close precedence. Exact current-pin
family replay and broad native verification remain pending.
