# Array toLocaleString: element invocation follow-up

## Scope

This candidate is rebased on main `abdcf56f1bd88d5debbb1d8c291f2e7213f77371`,
which includes the merged observable map/filter/every/some callback work from
#17. That merge moved unrelated Array code but left this `toLocaleString`
element-dispatch owner unchanged.

This change only extends the element-dispatch predicate in
`compile_to_locale_string_builtin`. Array and arguments values have dedicated
heap tags, but still require observable `toLocaleString` lookup and invocation.
They now enter the existing Object/Function invocation path rather than its
ordinary ToString fallback.

The existing operations retain the original element as receiver, read the
method once, validate callability, call callable Proxies, convert the returned
value to a string, and propagate abrupt completions before advancing the index.
No second method-call implementation, interpreter fallback, source rewrite,
expected-failure entry, Test262 pin change, or published-count edit is added.

References:
- [Array.prototype.toLocaleString](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.tolocalestring)
- [Invoke](https://tc39.es/ecma262/multipage/abstract-operations.html#sec-invoke)

The change does not claim an ECMA-402 argument-forwarding implementation. The
regressions deliberately do not require a specific number of reserved/locale
arguments. They retain the repository's existing separator behavior.

## Regression target and evidence limits

`crates/lila-engine/tests/aot_array_to_locale_string.rs` contains 18 active
regressions, each explicitly using `ExecutionBackend::WasmAot`. They cover Array
and arguments methods, inherited and own getters, original receivers, callable
Proxy methods, non-callable methods, abrupt getters/calls/result conversions,
live later elements, recursive default Array methods, minimal builtin planning,
and unchanged Object/Function/TypedArray/primitive controls.

All 18 exact JavaScript programs evaluated to true in Node 22.16.0. That checks
reference expectations only; it is not execution evidence for this patch.

The concrete baseline was reproduced on predecessor main
`65d1b70382d03e6bb1ffc17a5394c05125d8bbc5` with the #16 diagnostics CLI from
workflow run `34019160540`, artifact `9984914674`. Its recovered `array.rs` blob
was `7b07d79f774a8048aa309aaeaf5c6c8e9b4fb293`. The nested Array override returned
`8` instead of `custom`. The same run confirmed two separate generic-length
defects: a Uint8Array with an own length of 1 returned `7,9` rather than `7`, and
a two-argument arguments object with length redefined to 1 returned `4,5` rather
than `4`.

Current main's Array source is blob
`135c3568acdf5a4f5d9da69b0489eb6860e7ec44`; inspection after #17 confirms the
same unextended Object/Function-only element predicate remains in this owner.
The callback merge therefore supplies no evidence that these `toLocaleString`
failures were fixed incidentally.

Fresh executions of the larger regression programs encountered killed processes
and time limits in the constrained local environment. Those are not counted as
semantic failures or passes. No patched Rust build, patched Wasm execution,
real Test262 subtree, or whole-suite before/after result is available yet.
**The candidate remains a draft until the actual rebased head is verified.**

The Array conformance workflow adds a complete, nonempty engine-inventory check,
the retained TypedArray witness structure guard, and a nonempty retained CLI
selection. Existing flatMap verification stays enabled. CI must pass on the
actual proposed head; reference execution and source inspection cannot replace it.

## Required validation before merge

```sh
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo test --locked -p lila-engine --test aot_array_to_locale_string -- --test-threads=1
cargo test --locked -p lila-aot-wasm --test typed_array_to_locale_string_witness_structure
cargo test --locked -p lila-cli --test cli -- array_to_locale_string --test-threads=1
cargo build --locked -p lila-cli
./target/debug/lila test262 run built-ins/Array/prototype/toLocaleString/ --execution-backend wasm --threads 1 --jobs 1 --timeout-ms 60000 --snapshot-dir /tmp/locale-test262 --snapshot-name locale-review
```

Run the same pinned real subtree before and after, record every failure and its
owner, and preserve the existing ordinary CI and fake-suite gates. Do not infer
an all-green subtree from these commands: generic-length defects remain. The
reference count is not a new Test262 denominator or a full-suite percentage.

## Next work

The two reproduced generic-length defects require a distinct migration of the
ArrayLike entry to observable LengthOfArrayLike. Preserve the direct TypedArray
method's validated private witness; it intentionally has a different length
policy. Update the existing generic witness contract and tests as part of that
migration instead of merely weakening their private-layout assertions. Cover
own/inherited accessors, coercion and exception order, overridden arguments
length, resizable/detached buffers, Proxies, and live indexed reads.

For a measurable advance toward literal 100%, obtain a complete current-pin
Wasm-AOT snapshot, rank actual failures by shared semantic owner, and select the
next coherent batch from that inventory. The copied species implementations in
flat and concat remain review candidates after the merged callback work; they
are not fixed here.
