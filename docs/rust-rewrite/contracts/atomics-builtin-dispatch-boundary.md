# Atomics builtin dispatch boundary

Status: implemented and verified for the current Wasm-AOT Atomics family.

## Invariant

`AtomicsBuiltin` is a private, non-derived selection domain owned by
`builtins/atomics.rs`. The raw `emit_atomics_builtin` selector is private to
the same module. Each of the fourteen catalog cases enters that domain through
one fixed sibling-visible method, so `standard.rs` cannot import, construct or
forward the raw policy value.

Entry- and created-Realm publication share the exact ordered
`ATOMICS_PUBLICATION_ORDER: [StandardBuiltinId; 14]` list. Publication code can
iterate those catalog identities but cannot name `AtomicsBuiltin` or maintain a
second projection from the emitter domain. The preserved order is `add`, `and`,
`compareExchange`, `exchange`, `load`, `notify`, `or`, `pause`, `store`, `sub`,
`wait`, `waitAsync`, `xor`, then `isLockFree`.

The module audit enumerates every fixed entry and route, requires the sole
private selector, pins the publication list's fifteen `StandardBuiltinId`
mentions, and requires one publication loop in each Realm installer. The
structural target independently pins the exact non-derived domain, all fourteen
fixed entry-to-variant-to-catalog routes, the ordered publication list, and the
absence of raw policy in `standard.rs`, `bootstrap.rs` and `host.rs`.

## Source-equivalence witness

No instruction-emitting body changed. Reconstructing the former derived,
sibling-visible `AtomicsBuiltin` declaration and the former sibling-visible raw
selector from the current private forms produces the original exact 39-line
selection with SHA-256
`3382f4b6d98ca6acfb04ad9c9f452bd1f93bf65f9d3334e0cef0f17583366231`.
The old `AtomicsBuiltin::PUBLICATION_ORDER` plus exhaustive
`atomics_standard_builtin` projection was collapsed to its exact ordered
`StandardBuiltinId` result; both publication loops retain their order and
catalog metadata lookups.

## Verification

- `cargo xc` passes; existing workspace warnings remain.
- `cargo test -p lila-aot-wasm --test created_realm_atomics_publication_structure -- --test-threads=1` passes `5/5`.
- The seven neighboring Atomics structure targets pass `27/27`.
- The exact entry-Realm `atomics_add_surface`, created-Realm borrowed-method,
  and created-Realm `waitAsync` CLI controls each pass `1/1`.
- Formatting, module-boundary, task-plan and exact Test262 shortcut gates pass.

## Nonclaims

This is source-equivalent compile-time hardening. It adds no Atomics operation,
shared-memory behavior, agent behavior, Test262 pass or published-status change,
and it does not close T17.
