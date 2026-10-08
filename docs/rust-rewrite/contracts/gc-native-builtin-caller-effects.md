# Native builtin caller effects — 2026-10-05 dry source

The actual builtin catalog is the authority consumed by IR caller-flow
invalidation and the planned callable protocol. An entry whose native body can
invoke a getter, conversion hook, Proxy trap, source constructor or disposer
must declare synchronous user code. This includes internal continuations that
resume an observable walk. Direct kernels that cannot invoke those operations
retain their narrower effect classification.

BigInt and Symbol retain their actual constructor capability. Their native
bodies reject a non-undefined NewTarget before value conversion. Reflect
construction therefore acquires its argument list before that body rejection;
class heritage, Proxy construction traps and use as an ordinary target's
NewTarget remain admitted. The builtin constructor sections and §10.3 describe
these separate requirements: [BigInt](https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-bigint-constructor),
[Symbol](https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-symbol-constructor),
[builtin function objects](https://tc39.es/ecma262/multipage/ordinary-and-exotic-objects-behaviours.html#sec-built-in-function-objects).

The three Engine cohorts in `aot_gc_native_caller_effects.rs` observe captured
binding and property changes across native hooks, getter changes during
statically known super construction, and constructor admission versus native-body
rejection for BigInt and Symbol. They use actual Wasm AOT in strict and non-strict source modes.
All controls, compilation and runtime verification remain unrun. Later
verification requires the confirmed aggregate 4096 MiB kernel cap and serial
workers; this source contract provides no execution or conformance result.
