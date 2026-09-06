# `Array.prototype.flatMap` algorithm ownership

The Wasm-AOT implementation has one canonical Array algorithm,
`compile_array_prototype_flat_map_builtin`. The static Array entry continues to
use the shared argument-vector call boundary; the Iterator-helper branch is
unchanged. The removed direct Array wrapper must not return.

## Observable algorithm

All call-site argument expressions, including unused arguments and spreads,
finish before builtin execution. Inside the builtin, ToObject and one observable
LengthOfArrayLike precede IsCallable and ArraySpeciesCreate. Missing argument zero
is undefined, not a reason to skip observable length access. The length snapshot
survives species getters and construction without being refreshed.

Source indices below the snapshot use live HasProperty, then Get, then Call with
(value, index, boxed source). Callable Proxies and thisArg use the shared call
owner. A mapped value passes through the shared IsArray operation. Only Arrays
flatten, at depth one; their original Proxy receiver is retained for length,
presence and element access. Holes are skipped without mapper calls. Abrupt
completion stops later observable work.

A private append owner checks the maximum safe integer bound before shared
CreateDataPropertyOrThrow, propagates an abrupt definition, and increments only
on success. Custom species targets do not receive a synthetic length write.

The algorithm no longer reconstructs TypedArray private slots or implements its
own numeric length conversion, Proxy classification or species construction.
Those operations retain their existing semantic owners. The shared
ArraySpeciesCreate emitter now serves flatMap, slice and splice; its structural
census is updated without changing the operation catalog's evidence categories.

## Durable guards and execution evidence

The owner structure target pins one dispatcher/algorithm, complete argument
forwarding, operation order and append ordering. The TypedArray structure target
forbids private representation bypasses and retains the exact existing
resizable-buffer fixture matrix. The new `lila-engine` `aot_flat_map` target
executes observable programs through WasmAot, not an interpreter oracle.

See [the conformance follow-up](../aot-flat-map.md) for commands, evidence layers
and remaining work. Historical August 28 results described the direct-call
boundary only; they are not execution evidence for this replacement algorithm.
This change does not repair neighboring Array methods or the static branch's
broader property-lookup/classification policy.
