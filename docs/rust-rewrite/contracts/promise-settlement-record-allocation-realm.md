# Promise settlement-record allocation Realm

Status: implemented with focused structural verification on 2026-08-28.

## Ownership

`Promise.allSettled` creates each fulfillment or rejection record while its
element callback is executing. That record uses `%Object.prototype%` from the
defining Realm of the self-backed callback. Neither the active job Realm nor
the Promise constructor selects its prototype. The standard and keyed
all-settled callbacks are the only allocation consumers.

## Typed boundary

The private
`builtins/promise/promise_settlement_record_allocation.rs` owner contains the
non-`Copy`, must-use `PromiseSettlementRecordAllocationContext`, its sole
factory and its consuming allocator. The methods are visible only to the parent
Promise family. The parent may pass their inferred context at its two retained
call sites, but cannot name, import, re-export, construct or project the raw
prototype local.

The factory requires a nonzero self-backed environment, then loads the defining
Realm, intrinsic record and `%Object.prototype%` slot. Every absent authority
traps; it does not consult `CURRENT_REALM_GLOBAL_INDEX` or the entry
`OBJECT_PROTOTYPE_GLOBAL_INDEX`. The allocator consumes the context only after
emitting the fallible plain-object allocation, then releases the prototype
local. Record properties and their order remain in the parent callbacks.

## Focused evidence

The recursive structure witness pins the private module, sole four-use carrier,
one construction, two projections, both child-owned methods, exact two-caller
censuses and the unchanged standard/keyed record property routes. The
source-equivalent owner move selected the exact four-line carrier block at
SHA-256
`fe4dae7d7c230d964adbff8da382a9268049d19ee3fb8262544788d09802bbc2`
and the exact 61-line method block at SHA-256
`9d5475d82fb38f03f9967e900e13ea31f79cc1fc96ec7a3c8089f84067e16fc1`.
Their combined 65 selected lines retain SHA-256
`a3a37f83754be67d87f914f86ed5d823cb9a0d0147d4cbf762e2ef619f1c66c9`.
The resulting 70-line child has SHA-256
`ca39265d56101bd92f0dd324aad35e115ef4dfb51e47822a0850841c81327305`,
and the parent decreases from 9,457 to 9,391 lines. Method and caller bodies are
unchanged; only child ownership and parent-facing `pub(super)` spelling were
added. The recursive structure target passes `7/7`; the exact created-Realm
Promise allocation CLI witness passes `1/1`; and the shared `cargo xc`
checkpoint is green. Module boundaries, task-plan policy, workspace formatting
and diff hygiene are green.

```sh
cargo test -p lila-aot-wasm --test promise_callback_created_allocation_realm_structure --quiet
bash scripts/check-module-boundaries.sh
bash scripts/check-task-plan.sh
```

Semantic-golden and published-status verification do not apply to this
source-equivalent ownership move.

## Nonclaims

This boundary does not change all-settled property contents or order, keyed
record null-prototype policy, outer result collection allocation, callback
Realm publication, Promise capability settlement, AggregateError, heap layout
or any user-visible Promise behavior.
