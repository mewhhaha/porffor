# Function prototype receiver ownership

Status: normative for the AOT Function prototype builtin receiver boundary.

## Paired receiver authority

`FunctionPrototypeReceiverLocals` is the
paired Function prototype receiver authority for `@@hasInstance`, `call`,
`apply`, `bind`, and `toString`. Its
payload and tag fields are private inside a child module, the type is neither
`Clone` nor `Copy`, and its sole constructor reads both values together from a
`FunctionBuilder`'s invocation `this` slots. The constructor accepts no raw
local, receiver tuple, or `new.target` input.

Each of the five builtin branches retains that carrier for its complete
receiver-sensitive algorithm. A call site cannot mix payload and tag sources
while constructing the authority, cannot substitute `new.target`, and cannot
silently accept a state in which only one invocation-receiver local exists.
Missing receiver errors retain the exact builtin name as evidence. The carrier
does not validate callability or branding because those checks belong to the
individual ECMAScript operations and occur in their existing order.

The raw call and object emitters still accept local indices for their broader
callers. This boundary instead removes the five duplicated and independently
fallible receiver extractions from the Function builtin owner, where a
payload/tag transposition would otherwise compile as ordinary `u32` values.

## Durable guard and nonclaims

`function_prototype_receiver_ownership_structure` uses a recursive
Rust-lexical census that excludes comments and normal, raw, byte and C-string
literals. It pins the private attribute-free carrier, its sole paired
constructor, the exact five producers, absence of raw `this`/`new.target`
access in those branches, and every payload/tag projection count. A lexical
probe prevents comments, nested comments, raw identifiers and literals from
making the census vacuous.

This is source-equivalent type hardening. It does not change receiver
adaptation, callability checks, Proxy behavior, bound-function construction,
Function source text, Realm bootstrap, or conformance counts.

At `2026-08-27`, the receiver structure target passes `4/4`, and the neighboring
private-element ownership target passes `5/5`. The exact CLI test passes `1/1`,
executing `wasm_function_builtins.js` through Wasm-AOT with `boolean(true)` and
covering the unchanged `Function.prototype.call` and
`Function.prototype.apply` behavior.
The older Proxy call-routing guard also passes `2/3`, with its enum-end marker
now spanning a neighboring enum. Broader T09 and Test262 verification remain
deferred.

## Batch AV dispatcher boundary

The eight-case outer family now uses a private `FunctionBuiltin` with no derived
capabilities, and the raw emitter is private to `function.rs`. Standard dispatch
can reach it only through eight fixed Function entries: seven public intrinsic
operations and the separately named hidden bound-function invoker. The frozen
409-line domain/emitter selection has SHA-256
`f922e7edf4c8c1626a9b40920c2a9f418c8b3badcce3c347ffb09b55109d2093`.
Restoring only the former derive and visibility reproduces that source exactly.
`cargo xc` passes. The receiver-ownership, callable-prototype and
`Symbol.hasInstance` structure targets pass `4/4`, `8/8` and `5/5`; the exact
Function-builtin Wasm-AOT CLI fixture passes `1/1`. No Test262 or Wasm golden
was required for this source-equivalent dispatcher boundary. It claims no new Function behavior,
conformance result or published-count change.
