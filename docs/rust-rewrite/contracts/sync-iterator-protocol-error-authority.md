# Cached synchronous iterator protocol ownership

The atomic GC draft's OwnedSyncIterator carries a concrete IteratorRecord and
closed SyncIteratorConsumer diagnostic policy. The record retains whole
iterator and next values, so next is fetched once and callability is checked at
the actual invocation. SyncIteratorProtocolError has four closed variants and
the diagnostic consumer exhaustively handles each admitted consumer/error pair.

The cached step owner checks the next call, result Object and done observation,
then reads value only when requested and not done. Elisions and strict Zip
completion probes omit value Gets. The two explicit completion-output entries
allow native helpers to finish lifecycle and closing before publishing a Throw;
the existing propagating entries preserve their original completion behavior.
IteratorClose remains the sole whole-completion protocol owner, with existing
Throw precedence and completed-record admission.

The old manual-layout source guard depended on retired fields, producers and
literal occurrence counts. It retires with that layout. The native helper's
finite paired semantic controls exercise cached next, next/done/value Throw,
elisions, strict probes, callback closing, reentrancy and creation/called Realms.
See the [native helper contract](gc-iterator-helper-execution.md).

This is source authoring, with exact preimages retained in the atomic draft.
Compilation, fixture parsing, Wasm validation and runtime controls are pending.
All-task source completion precedes later verification under a confirmed
aggregate 4096 MiB cgroup budget and one worker. Historical proofs retain only
their original manual-layout source scope.
