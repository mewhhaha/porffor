# Whole mixed resource scopes

The checked AsyncGenerator resource scope owns one DisposeCapability for its
entire lexical resource list. It allocates the original private capability
before any initializer and consumes dedicated checked registration statements
in actual source order. Each registration first completes its staged initializer,
then uses the original GetDisposeMethod algorithm, appends the cached value and
method, and initializes the original const binding. An abrupt initializer or
GetMethod disposes only earlier successful registrations. Yield and normal Await
retain that same capability; final completion retires it before outward dispatch.

Scopes containing only `using` retain the original synchronous capability and
disposal loop. Scopes containing `await using` use the existing asynchronous
capability, including genuine synchronous entries. Its closed entry domain adds
SyncMethod alongside Empty, AsyncMethod and SyncFallbackMethod. True sync method
results are ignored without reading `then`; the await-using fallback still uses
the original promise wrapper and Await.

Empty asynchronous entries share the spec's needsAwait and hasAwaited flags
across the one disposal pass. Before a true synchronous entry, needsAwait with
no preceding method Await causes Await(undefined). That prelude clears needsAwait
but does not set hasAwaited. Its cursor and table row remain uncommitted through
suspension, so the original sync entry runs once after the reaction. Real async
method results set hasAwaited; the final empty-entry Await retains the original
one-shot behavior. Multiple declarators are never split into independent stacks.

The original whole pending-completion owner, reverse disposal loop and
SuppressedError construction remain shared. Complete scope disposal reconstructs
label and loop destinations before resuming, and dispatches Break, Continue,
Return and Throw only after capability retirement. Nested scopes retain their
own original capabilities and restore the enclosing compile-time registration
authority even when emission returns an error.

A resource `for-of` or `for-await-of` head owns one capability per original
iteration record, spanning its checked initialization and complete body. The
shared resource lifetime emitter allocates it before GetDisposeMethod and keeps
the original per-key environment attached throughout disposal Await. Continue
disposes before advancing; Break, Return and Throw dispose before the outer
iterator close. Only then is the iteration environment left. Both ordinary
lexical scopes and per-key heads consume the same registration and disposal
algorithms, with authority minted from their actual opaque carriers.

A classic `for` resource head keeps one capability for the whole loop, including
test and update suspensions and local Continue. Its original const record is
not cloned per iteration. Switch clauses require an explicit nested Block for
resource declarations; direct clause declarations are early errors. An entered
Block owns its capability until control leaves that Block, including fallthrough
to another clause. Unmatched clauses never register resources. Source Break,
outward Continue, Return and Throw consume the original lexical disposal finally.
Captured bindings remain in their original records after disposal.

`aot_async_generator_resource_lifecycle.rs` compiles strict/sloppy product scripts
covering empty-entry prelude/final Await ordering, a true sync method whose return
has a poisoned then getter, cached method identity, mixed initializers, partial
registration, retained TDZ, queued Return, SuppressedError identity, GC and
awaiting/yielding finalizers. The per-key control additionally checks captured
head cells inside an asynchronous disposer, cached methods, disposal before
Next and disposal before outer close. The classic cohort checks one capability
across repeated test/update. The corrected switch cohort checks disposal before
fallthrough, retained original cells while an externally pending disposer holds
a queued request, lazy registration, later GetMethod failure, and injected
Return/Throw identities. The 2026-10-07 fixture repair follows the discovery of
the same invalid direct clause declarations in another resource control. All
nine corrected resource parsing/lowering checks pass with no diagnostics. The
corrected switch lifecycle control passes both native modes in
`tasks-calendars-resource-repairs1` (180.18 seconds). This result does not claim
acceptance of the full lifecycle matrix.

The flag and method distinction follows
[ECMA-262 DisposeResources](https://tc39.es/ecma262/multipage/abstract-operations.html#sec-disposeresources)
and GetDisposeMethod in the same specification.
