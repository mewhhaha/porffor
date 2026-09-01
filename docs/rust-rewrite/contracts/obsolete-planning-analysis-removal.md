# Obsolete planning analysis removal

Status: implemented as a source-equivalent T02 reachability closure.

The backend planner contained two complete recursive analyses for whether a
script used calls or the function table, plus trivial unused script-level
environment and function-heap projections. No product planner entry called any
of their four roots. The only other references were two assertions in the
regexp-literal planning test; that test retains its observable bootstrap-root
and stubbing assertions.

The same reachability audit found three independent planning-only remnants:
the large deferred-builtin classifier, `FunctionMetaRegistry::iter`, and a
copied `WasmFunctionMeta::super_constructor_target` field that was written at
all three constructors but never read. The original IR field remains live in
`data.rs`; only its unused Wasm metadata copy is gone. Registry `values` and
`metas`, active function-reference planning, parameter/local counting and
builtin stubbing remain unchanged.

The deleted 288-line deferred-builtin classifier has SHA-256
`be7c5a1e0e9fe6fefc2c8a5db187f192c1e5f55764eeee29d940dc26ad94a177`.
The deleted registry iterator has SHA-256
`17b31c1feb5348b2f1e2dc0cdf24a618519ddebf55d39105288c6b898d8fb88f`.
The deleted 841-line analysis island has SHA-256
`4050159124cc94d7b65ee22e7bd566c9b600bf5bbb55b5815ca6f4ef537e3ea8`.
The removed metadata field plus its three writes have combined SHA-256
`34679124b57a9e0716f4a604d29f5383ffd4c91ce3d1fdb8aa509c65951df238`.
The two removed test-only assertions have SHA-256
`c99ecf4f2aca412f218f8e5a6be29cacb0fe51d34635533114bc0d74e698bba5`.

This deletion has no new JavaScript behavior and changes no emitted Wasm: no
removed analysis or metadata projection had a product consumer. It adds no
Test262 materialization, capability claim or published count.

At the Batch BU checkpoint, `cargo xc` is green without the corresponding
planning dead-code diagnostics, the focused absence target passes `3/3`, and
the retained regexp-literal bootstrap-root unit passes `1/1`.
