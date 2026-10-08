# Native host signatures and recursive runtime types

Function signatures without concrete Lila GC references are registered as
singleton Wasm types before the runtime's recursion group. Their canonical
identities then agree with independently constructed Wasmtime host functions.
Placing an otherwise identical scalar signature inside the entire runtime group
previously passed Wasm validation but failed native import binding.

`StaticSignature` derives group membership from its actual parameter and result
types. A constant assertion requires the independent signatures to precede the
recursive signatures, so their assigned indices retain declaration order.
JavaScript callable signatures, concrete GC host signatures, runtime helpers and
semantic layouts keep the same shared recursive graph. Concrete GC host imports
continue to bind using their actual module function types.

The existing encoded-registry control validates the complete module and checks
the recursion group at every declared static signature index. The original
native async-generator lifecycle regression exercises actual host linking.

The 2026-10-07 `converters1` checkpoint passed the workspace type check and 28
focused controls, then completed native compilation in 80.6 seconds under the
4096 MiB cap. Instantiation rejected `agent_can_suspend`. This source repair and
its extended encoded-registry control await the next joined checkpoint.
