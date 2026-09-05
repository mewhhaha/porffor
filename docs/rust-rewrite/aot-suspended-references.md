# Suspended ordinary property References

This follow-up to native exception-control emission implements ordinary property
assignment after plain and delegated async-generator yield. It uses the existing
Reference IR and object-write machinery, not an interpreter or alternative object
model. Synchronous generators use the same helper and also receive the deferred
computed-key correction.

The execution owner selects one activation layout. The async activation appends
four initialized words for two tagged values: the original base/receiver and the
raw computed key. Existing fields retain their offsets. Payload words are included
in the canonical pointer layout and tags are non-pointer words. Iterator
delegation keeps its separate record.

The base and key expression are evaluated once before the RHS. ToPropertyKey is
deferred until normal RHS completion. The write saves the received value in
dedicated locals before key coercion can run user code. A nullish base fails
ToObject before key coercion; a throwing conversion or setter reaches the active
catch/finally. The carried Reference strictness controls failed Set behavior.

Delegated terminal throws and returns first cross a local cleanup block, which
retires the iterator record and saved Reference before dispatching to the outer
catch/finally. This covers rejected iterator results, throwing getters/calls, and
missing return methods as well as ordinary delegated completion. A later yield
in the handler must not inherit the old delegation state. Genuine await/yield
suspensions still return without running this cleanup. A delegate that handles
`.throw()` and completes normally does perform the pending assignment.

## Verification

`cargo test --locked -p lila-engine --test aot_suspended_references -- --test-threads=1`
executes 19 compiled-JavaScript regressions through Wasmtime and asserts exact
observable traces. The cases cover evaluation order, abrupt resumption, strict
and sloppy writes, Symbol keys, nullish bases, queued/interleaved activations,
synchronous generators, and delegated cleanup with suspending handlers.
The existing AOT regression workflow includes this target. Heap layout and full
backend tests remain enabled; no ignores, skips or generated conformance counts
are changed. Results on the final revision are recorded in the PR.

Captured lexical environments across suspension, nested for-await and other
dispatcher gaps remain separate work. Runtime dynamic source retains AGENTS.md's
explicit unsupported policy.

## Specification

- [Property Reference evaluation](https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-evaluate-property-access-with-expression-key)
- [PutValue](https://tc39.es/ecma262/multipage/ecmascript-data-types-and-values.html#sec-putvalue)
