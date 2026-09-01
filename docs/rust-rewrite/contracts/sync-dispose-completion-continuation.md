# Synchronous disposal completion continuation

The private, must-use, capability-free
`SyncDisposeCompletionContinuation::{Dispatch, DispatchAsyncFunction,
DispatchAsyncGenerator, DeferToIteratorClose}` domain owns the one legal action
after a synchronous disposal walk restores its saved completion. It cannot be
cloned, copied, formatted, defaulted, compared, ordered or hashed.

The activation-backed owner exhaustively selects the plain-generator,
async-function or async-generator dispatch variant. The two non-activation
scope/loop exits select ordinary dispatch directly, while synchronous
`using`-for-of selects deferred IteratorClose. Each producer moves its choice
into `consume_sync_disposable_resources`; the sole ownership-consuming
continuation match runs only after completion restoration and states all four
actions without a wildcard. A caller cannot replay the same dispatch authority
or independently inspect it before disposal completes.

This capability closure preserves the existing reverse disposer walk,
`SuppressedError` fold, completion restoration and dispatch/IteratorClose
ordering. It changes no emitted Wasm or runtime behavior. The recursive guard
pins the exact domain, producer census, owner mapping, consuming signature and
post-restoration exhaustive match. At the shared Batch AC checkpoint,
`cargo xc` is green, the structure target passes `3/3`, its neighboring
synchronous-using-for-of target passes `5/5`, and the exact synchronous-scope,
plain-generator, plain-async-function, async-generator and using-for-of CLI
lifecycle witnesses pass `5/5`. No Test262 cohort or semantic golden was run
for this source-equivalent capability closure.

This boundary does not change resource acquisition, async disposal, iterator
closing, environment lifetime, generator state, or the ECMAScript disposal
algorithms. It makes no conformance or broad T15 completion claim.
