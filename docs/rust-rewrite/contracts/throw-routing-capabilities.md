# Throw-routing capability boundary

The current product path uses rooted whole `CompletionLocals`. The consumed
Proxy dispatcher is `functions/call_dispatch.rs::emit_proxy_call_dispatch`;
its caller selects actual propagation through the closed `PropagateCallThrow`
domain. Indirect operations retain the completion for their enclosing handler,
so user `catch`/`finally` and arbitrary thrown values remain observable.

The older scalar Proxy and primitive-number routing wrappers are retired. Their
unconsumed ProxyCallThrowRouting declaration and four structure controls that
mirrored its removed wrappers were deleted in the 2026-10-06 source batch. No
public policy, compatibility wrapper or alternate dispatcher is introduced.
Existing semantic Proxy/call/throw and numeric-conversion cohorts remain the
verification surface; no new mirrored source test is added.

This source cleanup has not been compiled or executed. Whole-value completion
and Realm behavior still require the affected focused and broad checkpoints
under the shared one-CPU, 4096 MiB verification cap. Published Test262 counts
are unchanged.
