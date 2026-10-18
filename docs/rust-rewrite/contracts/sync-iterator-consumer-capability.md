# Sync iterator consumer capability

Status: implemented closed diagnostic policy and linear native iterator owner;
the complete 115-control `structure_async` target passes in the
[2026-10-11 repair checkpoint](../workspace-inventory-repairs-20261011.md).

The diagnostic policy is capability-free; its Copy/Clone values cannot own
or release the native iterator retained by `OwnedSyncIterator`.

## Closed consumer domain

`SyncIteratorConsumer` has sixteen variants:
`ArrayDestructuring`, `ArrayAccumulation`, `ArrayFrom`, `ForOf`,
`MathSumPrecise`, `ListFormat`, `AggregateError`, `MapConstructor`,
`SetConstructor`, `WeakMapConstructor`, `WeakSetConstructor`,
`ObjectFromEntries`, `MapGroupBy`, `ObjectGroupBy`, `SetLike` and
`IteratorHelper`. It is a Copy/Clone diagnostic policy, so copying it cannot copy
or release a native iterator. The separate `OwnedSyncIterator` is non-copyable,
non-cloneable and must be cleared or captured by its consumer. It retains the
actual GC `IteratorRecord` together with the policy.

`SyncIteratorProtocolError` remains the private four-variant failure domain:
`NotIterable`, `MethodResultNotObject`, `NextNotCallable` and
`NextResultNotObject`. One projector exhaustively matches the borrowed consumer
and error, with exactly 64 diagnostic rows and no default branch. WeakMap and
WeakSet constructors now own their full four-error rows; their live iterable
walks call the existing shared acquisition owner.

The shared path follows GetMethod, GetIterator, GetIteratorFromMethod,
IteratorNext, IteratorComplete, IteratorValue and IteratorStepValue. Each
semantic owner acquires one native record, borrows it for stepping and consumes
its lifecycle through clear, capture or close. Ordinary array spread propagates
step, done and value abrupt completions without IteratorClose. Destructuring
retains its elision and early-close rules; ListFormat closes for a non-string
value but preserves acquisition/step abrupts; AggregateError's IteratorToList
propagates acquisition/step abrupts directly.

## Current source census

Literal token counts below are defined across backend `src/**/*.rs`, including
declarations; they do not count runtime executions or establish acceptance:

- 111 `SyncIteratorConsumer` tokens, 73 in `control_flow.rs`;
- 70 `SyncIteratorProtocolError` tokens: declaration, projector parameter,
  two acquisition error constructors, 64 mapping rows and two step constructors;
- 17 `emit_get_sync_iterator(` occurrences: one declaration and sixteen
  ordinary semantic acquisition sites;
- five `emit_get_sync_iterator_direct(` occurrences: declaration, internal
  acquisition forwarding and three direct semantic owners;
- four `self.emit_sync_iterator_protocol_type_error(` call sites, with one
  physical projector declaration; and
- two `&iterator.consumer` borrows in the shared step owner.

Layout 13 adds a distinct typed ArrayAccumulation step-value kernel in R.
Its caller passes the actual cached iterator record and the original executing
Realm, environment and program; the complete Completion crosses the boundary.
Only ArrayAccumulation value steps select this kernel. Its parameter owner
retains that consumer's four diagnostic rows; array-pattern elision and close
retain their separate kernels. P owns acquisition, append, abrupt propagation,
DONE readback and final release. A physical artifact control follows the
ordinary P record constructor and call to the named R body and checks the full
ABI, cached-next reads and DONE writes; retained pattern controls still require
the suspension-owned record reload. The release-owner census is 37 type tokens,
including eight in the shared helper file, with eleven constructor-text matches
and two external helper-record constructors.

These nineteen semantic acquisition sites have distinct ownership obligations;
they are not nineteen instances of every error constructor. The structure
control pins the exact sixteen-variant declaration and four rows per consumer,
the sole physical acquisition/step/projector owners and actual borrowed record
lifecycle. It rejects retired raw policy and local-owner types.

## Realm invariant

Diagnostics use the executing function Realm. Non-nullish primitive iterable
sources are boxed in that Realm before reading `@@iterator`. Runtime error
allocation reads trusted function-context/defining-Realm metadata through the
existing exhaustive execution-Realm owner; lexical environments and source,
method, iterator or next values do not select a generated TypeError's Realm.
Accessor, Proxy, iterator-method, next, done and value throws preserve their
original values.

The existing destructuring and ArrayAccumulation fixtures pin completion
identity, close/no-close behavior, elision ordering, diagnostics and primitive
boxing in the entry Realm. ListFormat fixtures pin abrupt identity and close
precedence. These entry-Realm syntax fixtures alone do not prove cross-Realm
syntax allocation. Unmatched dynamic source remains the explicitly reported
AOT capability boundary described by AGENTS.md.

## Historical verification

The following results retain the older source/consumer inventory of their
original checkpoints. They do not establish the newer weak-constructor rows,
the complete current pinned suite or the ongoing broad workspace sweep.

The all-target compile and formatting check pass. The consumer-capability,
protocol-error, direct `for-of`, `Math.sumPrecise`, resumable plain-async,
iterator-local, destructuring-step, destructuring-local, and IteratorClose
structure targets pass `42/42`. The new ArrayAccumulation fixture plus six
retained exact Wasm-AOT CLI witnesses pass `7/7`. The new fixture also passes
`node --check`, and the module boundary guard is green.

Five pinned Array-spread leaves and four Array-destructuring leaves pass all
`18/18` sloppy/strict Wasm-AOT executions with every failure and non-success
bucket at zero. The callable-Proxy/body-Realm follow-up reran the five directly
affected structure targets at `23/23`, five exact CLI controls at `5/5`, and eight
unchanged direct iterator/Proxy leaves at `16/16`, with every failure and
non-success bucket zero. The all-target compile, formatting, module-boundary,
task-plan, shortcut-inventory, and diff checks remain green.


Current focused structure command:

```sh
cargo test -p lila-aot-wasm --test structure_async -- sync_iterator_consumer_capability_structure:: --test-threads=1
```

The stable contract path remains
`docs/rust-rewrite/contracts/sync-iterator-consumer-capability.md`.
