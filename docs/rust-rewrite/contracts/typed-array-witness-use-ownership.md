# TypedArray witness-use ownership

Status: normative for the AOT TypedArray buffer-witness use boundary.

## Semantic boundary

`TypedArrayWitnessUse` carries the complete purpose of one fresh backing-store
observation. Its variants distinguish throwing method entry, non-throwing
Array-like length capture, integer-index presence, and the three accessor
results. Each value also owns the destination locals into which that single
observation may publish its result.

This policy is a move-only witness-use authority. It implements neither
`Clone` nor `Copy`. `emit_typed_array_witness` first borrows it for the
method-entry validation decision and completes the shared cached-length and
element-length calculation. The final consuming projection then moves that
authority and publishes exactly one result. Code after that projection cannot
reuse the authority for a second result publication. The validation
match deliberately binds none of the payload locals, while the consuming match
is the sole payload owner.

The four variants remain exhaustive in both decisions. Adding a use therefore
requires an explicit validation policy and an explicit result algorithm; no
catch-all can silently inherit the behavior of an existing consumer. View
locals remain a separate immutable description because many algorithms
legitimately take later live integer-index observations from the same view.
`TypedArrayViewLocals` itself is non-`Clone` and non-`Copy`: each of its 46
producers constructs one owned five-local carrier, and every live observation
borrows that carrier. Algorithms that require multiple observations therefore
reuse one authority by shared borrow instead of forking independent copies of
the private-slot roles.

## Durable guard

`typed_array_witness_use_ownership_structure` performs a recursive
Rust-lexical census that excludes comments and every Rust string/byte/C-string
literal form. It pins the private attribute-free declaration, exact four
variants, all current producer and consumer routes, the sole typed witness
boundary, and the borrowed-validation-before-owned-result order. A lexical
probe prevents comments, nested comments, raw identifiers and literals from
making the census vacuous.

Batch AG extends the same guard to the view carrier. It pins 56 exact product
mentions, 46 constructors, two borrowed type boundaries, the attribute-free
five-field declaration, and the absence of manual clone, copy, debug, default,
comparison, ordering or hashing implementations. The unchanged declaration is
`64a7e96e10f1d53150a94e915656bd69b2a050449e7fa73b2954093ddd1b5390`, its
constructor implementation is
`7ff4343576674f15b704921718176ace71d92df0927299bfed696ee008a10f80`, and
the shared witness emitter remains
`61daf0915471d6f3f2ac4e62dd3792bb940a318c7a9199676fe327ea852ec226`.

This is source-equivalent ownership hardening. It does not change buffer
observation, detachment or resize behavior, add a new TypedArray consumer,
retire a Test262 rewrite, or claim full T17 conformance. Focused compilation,
the ownership guard and a neighboring witness structure target own the
checkpoint; broad conformance and semantic-golden runs remain deferred. At the
current checkpoint, the package-level ownership target passes `4/4`, the
neighboring Atomics TypedArray-witness target passes `5/5`, and the exact
TypedArray iterator CLI witness passes `1/1`. A standalone run of the older
iterator structure target is `1/2`: its Realm-validation subtest passes, while
its other subtest stops at a stale `StandardBuiltinId::ArrayPrototypeKeys`
source marker before reaching this ownership seam. That unrelated marker is
not reported as green.

Batch AG changes no producer or witness instruction body. At the shared
checkpoint, `cargo xc` is green, the expanded ownership structure target passes
`5/5`, and the exact
`typed_array::run_wasm_backend_copies_typedarray_bytes_with_spec_ordering` CLI
witness passes `1/1`. The pinned
`built-ins/TypedArray/prototype/copyWithin/resizable-buffer.js`,
`built-ins/TypedArray/prototype/copyWithin/coerced-values-start-detached.js` and
`built-ins/TypedArray/prototype/copyWithin/coerced-values-end-detached.js`
leaves pass all `6/6` Wasm-AOT executions with every failure bucket at zero.
Batch AG did not rerun the semantic golden.
