# Host import function indices authority

Status: implemented; includes scalar roles, typed GC capabilities and required runtime-semantics rejection.

## Closed host import roles

Twenty-six optional scalar roles and one required rejection role cross emission
planning through the non-copyable `HostImportFunctionIndices` authority. Each
position accepts a distinct, non-derived Rust role type. The optional roles are
Number power, wall clock, monotonic clock, sleep, agent call, random, and twenty
`math_*` transcendentals. The twenty-seven named registry getters are the sole
raw-index projections into Wasm `call` instructions.

Distinct constructor roles prevent callers from being able to
transpose two raw `Option<u32>` positions.

Previously `emit.rs` passed adjacent `Option<u32>` indices to
`FunctionMetaRegistry::new`. Transposing two values compiled and could select the
wrong imported capability or signature. The typed constructor rejects such a
transposition. One complete producer builds the authority, and the registry
stores it intact.

The required `reject_runtime_semantics` index follows the scalar roles. Its
closed-domain `i64` reason code returns no value. This compiler capability error
is not a JavaScript completion and remains non-passing in Test262. See
[dynamic-source-capability.md](dynamic-source-capability.md) and
[temporal-named-zone-authority.md](temporal-named-zone-authority.md).

## GC host capabilities

GC-reference signatures have a separate closed `GcHostImport` authority.
Shared-buffer allocation, Intl provider calls and system timezone snapshots
moved here; their retired scalar role types have no product references.
`GcHostImports::plan` exhaustively selects each static signature and assigns a
checked function index. `DeclaredGcHostImport` carries the role with that index;
callers retrieve it by its enum variant rather than transposing raw integers.

The current domain has 34 entries: 15 collection, byte-array, shared-buffer,
agent, waiter, Intl and timezone capabilities, followed by 19 weak collection,
weak reference, kept-root job and finalization capabilities. Allocation,
ephemeron operations, dereferencing, registry/cell lifecycle and cleanup error
reporting have distinct signatures and roles. The exact macro entry inventory
is pinned by `host_import_function_indices_structure`; its recursive Rust
lexical census excludes comments and string/character literal forms.

## Evidence

The source controls pin all 27 scalar roles and projections, the sole complete
producer, intact registry storage, all 34 GC declarations, checked index
assignment and absence of retired scalar roles. The 2026-10-11 complete workspace
probe exposed its stale 15-entry GC inventory after weak integration. The
corrected inventory retains the closed boundary. All 25 controls in its complete
`runtime_link` target pass in the [repair checkpoint](../workspace-inventory-repairs-20261011.md).

The historical 2026-08-27 authority checkpoint passed five focused structure
controls, its scoped type check and six existing Number, Date, Math.random,
Intl, Test262 agent and Atomics emission witnesses. That earlier receipt does
not establish the newer GC capability inventory or complete suite acceptance.
