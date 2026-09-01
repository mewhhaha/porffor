# Array `at` receiver policy

Status: normative for Lila's shared Array and TypedArray `at` emitter.

## Boundary

`ArrayAtReceiverPolicy` is the capability-free, two-row authority for the
shared `at` implementation:

| Policy | Producers | Receiver behavior |
| --- | --- | --- |
| `GenericArrayLike` | `Array.prototype.at` | applies the generic `LengthOfArrayLike` and indexed-read rules, including observable ordinary-object and primitive handling |
| `TypedArray` | `%TypedArray%.prototype.at`; the direct TypedArray method-call lowering selects that standard entry | requires a validated TypedArray and uses the validated-method-entry buffer witness |

The two fixed builtin entries each move their private `ArrayAtReceiverPolicy`
once into the shared emitter. Standard dispatch cannot import, construct or
pass the raw policy. Direct TypedArray lowering selects the strict standard
entry rather than constructing another policy path. The emitter borrows the authority
through four direct exhaustive matches: Array/Arguments handling, TypedArray
witness selection, ordinary Object/Function handling, and primitive/nullish
handling. It has no Boolean, equality, default, wildcard, unreachable or
numeric projection. A new receiver policy therefore cannot compile until all
four semantic decisions are stated.

This source-equivalent closure adds no Wasm or heap representation. It preserves
receiver classification, error messages, observable `length` reads, TypedArray
witness selection, index coercion and temporary-local release order.

## Durable witnesses

`array_at_receiver_policy_structure.rs` lexically pins the capability-free
private declaration, recursive 13-use authority census, exact two fixed policy
constructors, the direct strict-entry selection, both forwarding routes, all four complete
projection bodies and their order before index conversion. Its scanner
excludes comments and literal contents from
identifier and route censuses while retaining exact literals in body
fingerprints.

`wasm_array_at_runtime_kinds.js` covers inherited Array reads, Arguments,
ordinary objects and functions, strings, Proxy ordering and abrupt completion,
generic-policy TypedArray reads, strict TypedArray rejection, resizable buffers,
index coercion and out-of-bounds behavior.

## Focused verification

```sh
cargo test -p lila-aot-wasm --test array_at_receiver_policy_structure
cargo test -p lila-aot-wasm --test typed_array_search_kind_structure
cargo test -p lila-cli --test cli array::run_wasm_backend_succeeds_for_supported_array_at_runtime_kinds_fixture -- --exact --test-threads=1
rustfmt --edition 2021 --check crates/lila-aot-wasm/src/builtins/array.rs crates/lila-aot-wasm/tests/array_at_receiver_policy_structure.rs crates/lila-aot-wasm/tests/typed_array_search_kind_structure.rs
git diff --check -- crates/lila-aot-wasm/src/builtins/array.rs crates/lila-aot-wasm/tests/array_at_receiver_policy_structure.rs crates/lila-aot-wasm/tests/typed_array_search_kind_structure.rs docs/rust-rewrite/contracts/array-at-receiver-policy.md tasks/16-arrays-and-array-builtins.md
```

On 2026-08-28, the owned structure target passed `3/3`, the direct-entry owner
target passed `4/4`, and the exact runtime-kinds CLI witness passed `1/1`.
Independent review confirmed the 14-use authority census, the two standard
policy constructors and all four complete receiver-policy bodies. Targeted Rust
formatting and the scoped diff check passed. The neighboring TypedArray
search-kind target, broad workspace compile and Array Test262 verification were
not rerun in this direct-entry lane.

Batch AR makes the raw policy and shared compiler private to `array.rs` and
exposes only fixed Array and TypedArray semantic entries. The frozen 34-line
raw compiler has SHA-256
`4888ef68f6f42b58d9e14480d5381cf64018176ed21504a10fc6883dac564aaa`;
normalizing its private name and visibility reproduces that hash exactly. At
the 2026-08-28 Batch AR checkpoint, `cargo xc` is green, the strengthened
structure target passes `4/4`, and the exact runtime-kinds CLI control passes
`1/1`. This source-equivalent tightening claims no new Array behavior and no new
TypedArray behavior, and no Batch AR Test262 or semantic-golden result.

## Nonclaims

This invariant does not expand supported receiver shapes, TypedArray buffer
semantics, Proxy behavior, generic indexed reads or Test262 coverage. It does
not alter the separate TypedArray search-kind domain or claim broad Array and
TypedArray conformance.
