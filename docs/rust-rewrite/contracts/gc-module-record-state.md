# GC ModuleRecord state authority

The four actual evaluation-state, completion, body-state and activation-kind
readers accept only a rooted nonnull ModuleRecord and an I32 scalar destination.
Each reads its declared GC field and checks membership in the field's existing
closed enum domain. Allocation and transitions publish through those typed
fields. Existing DFS, cycle roots, cached Promise settlement, async parents and
entry completion consume these readers without byte offsets or raw record IDs.

Fourteen superseded manual heap methods retire: eight have no product callers;
four module readers are replaced by the actual private record_state child; the
Promise-state and request-phase readers already have typed native owners. The
remaining raw allocator/load-store APIs still belong to the unfinished Temporal
port and must retire after those callers move. Shared scalar memory instructions
and the real bounded private-byte transport retain their separate roles.

This source successor preserves exact current afterimages and every original
receipt. Existing meaningful module/Promise execution controls remain; no new
source mirror is added. One isolated format read the owned Rust files. Current
compilation, Wasm validation, runtime and pinned verification remain unrun. All
task source must finish before the confirmed aggregate 4 GiB serial checkpoint.
