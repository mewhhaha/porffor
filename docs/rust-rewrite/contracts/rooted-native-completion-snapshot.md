# Rooted native completion snapshots

`Engine::observe_script_graph` and `observe_module_graph` accept the original
source, compilation and run options, then checked `SnapshotLimits`. They return
`GraphRunOutcome` with the original Normal or Throw completion kind, a complete
validated graph or typed snapshot rejection, captured output, and the actual
backend. Parse, link, host, timeout and backend failures remain execution errors.
An agent failure remains an execution error even when root capture succeeds or
rejects. A concurrent root Throw retains separate root failure provenance in
the aggregate; neither graph data nor diagnostics supply an exception brand.

The native route emits an opt-in artifact with a separate cache-key domain.
Ordinary product and category-only observation artifacts have no snapshot
exports. Immutable null globals witness the actual declaration-derived Wasm GC
types; the host compares canonical type identity before accessing fields. Field
ordinals, descriptor masks, function protocols and intrinsic/Symbol slots come
from the same owners consumed by code generation.

An opt-in linked GC inventory retains every actual Realm record at its original
allocation, together with the first entry Realm and the agent's well-known
Symbol table. Inventory extent is checked before copying its roots. Realm IDs
in the graph follow actual encounters, with entry Realm zero, rather than
allocation order. Intrinsic anchors compare raw identities in the retained
Realm intrinsic tables; assignments to public constructor properties do not
change this authority.

The original main completion stays rooted in the execution RootScope through
job drainage and the Module settlement check. Graph mode branches before the
legacy scalar observer and before diagnostic string rendering. The adapter
reads raw GC descriptors, prototypes and identity only. It executes no getter,
Proxy trap, coercion, iterator, Wasm projection export or JavaScript wrapper.

The initial graph domain includes all primitives, local/registered/well-known
Symbols, ordinary objects, Arrays, ordinary and native FunctionObject records,
and the NativeErrorObject wrapper's ordinary public property graph. Functions
retain constructability and their actual context Realm, without claiming code
or closure equivalence. Arrays retain holes, occupied integer-index order,
the virtual complete length descriptor, then string and Symbol insertion order.
Accessors retain their raw get/set values and are never called.

Proxy, bound/generator/async functions, boxed values, Promises and other exotic
or internal-slot records are explicit rejections. Occupied private-element
records also reject. No unsupported layout is read through a guessed ordinary
header. Rejection preserves the original completion kind and output; it never
publishes a partial graph or compares as equal to another rejection.

All dynamic traversal uses the shared budgets. UTF16 is charged before the
copy, property buffers stay within the remaining property cap, and BigInt
conversion bounds limb work and decimal chunks before allocating the final
string. Identities stay rooted until capture finishes. The Engine and AOT
controls cover opt-in export separation, cyclic roots, Array holes and
attributes, uncaught Throw after jobs, getter non-execution, Symbol identity,
created-Realm anchors, ordinary Error graphs and explicit budget/exotic failure.

The 2026-10-07 workspace type check passes. Native controls pass created-Realm
anchors, ordinary Error graphs and explicit exotic/budget rejection. The
post-job Throw keeps its original root and output but graph admission rejects a
duplicate own property key. Inferred function naming appended a second `name`
after allocation; its producer repair updates the descriptor through ordinary
definition. `tasks-loop-name1` passes the original graph control unchanged,
including the retained root, job output, Array descriptors and Symbol aliases.
Snapshot admission continues rejecting duplicate keys.
