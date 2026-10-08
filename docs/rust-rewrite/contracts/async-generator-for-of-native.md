# Complete mixed iterator phases

`AsyncGeneratorForOfIr` is the sole native admission authority for complete
AsyncGenerator ForOf and ForAwait. Its checked source initializer preserves the
original head TDZ and per-key Environment Records. Head, initializer and body
regions carry the actual mixed Await/Yield tape; implicit Next and Close Await
points retain their original SavedLexicalChain protocol.

The native owner uses the existing InvocationFrame iterator table. One checked
owner entry selects an original `ForAwaitIteratorState` containing the actual
IteratorRecord and async-iterator flag. Sync iteration caches the same record
with a false flag and performs the existing synchronous step and close. Awaited
iteration shares the original ForAwait acquisition, cached Next, IteratorResult
extraction and close-call bodies. A synchronous fallback uses the original
AsyncFromSync methods. No second iterator representation or GC field is added.

Acquisition and Next/result errors remain outside the per-key close scope.
Initialization and body resume inside that scope after reconstructing the
original iteration record and before injecting Return, Throw or rejected Await.
Normal and matching Continue retain StatementList V, leave the per-key record
and advance once. Body resumption neither reacquires nor advances nor repeats
initialization. Head and initializer operands suppress source completion V;
the complete body owns its retained V cell.

Abrupt initialization/body outcomes leave the original per-key environment
before IteratorClose. Sync close is never awaited. Awaited close parks the whole
pending Completion using the existing frame stack and keeps the iterator cache
until the actual reaction. Incoming Throw retains priority over close errors;
Return and Break can be replaced by a close failure or non-object result. The
cache row and head/incoming/V cells retire before outward dispatch.

The ordinary product controls in `aot_async_generator_for_of_lifecycle.rs` cover
strict and sloppy compilation, nested pattern close before outer close, cached
Next receiver identity, synchronous and async-fallback protocols, queued Return,
close rejection, protocol errors without close, initializer rejection with close,
whole completion identity, GC and awaiting/yielding finalizers. They are authored
source controls; compilation and execution remain deferred to the batch checkpoint.
