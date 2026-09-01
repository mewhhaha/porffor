# JSON.stringify replacer invocation authority

Status: normative for the AOT JSON.stringify replacer call boundary.

## Closed invocation roles

`JsonStringifyReplacerInvocationLocals` is the move-only
JSON.stringify replacer invocation authority. Its private child module owns
four distinct tagged roles for the replacer function, exact callback receiver,
property key, and mutable value/result carrier. The authority's only constructor
requires all four roles, and the sole emitter consumes the complete authority.
A producer cannot transpose replacer, receiver, property key, and value roles
through the former eight positional `u32` parameters without a Rust type error.

All six product producers retain their existing mappings: the synthetic root
uses the wrapper object as `this`; Array and Proxy-Array elements use the exact
array; ordinary, specialized, and Proxy-aware object paths use the holder that
provided the property. The argument vector remains `[key, value]`, the call
result overwrites the value carrier, and an abrupt completion still propagates
that exact thrown value.

## Durable guard and nonclaims

`json_stringify_replacer_invocation_authority_structure` uses a recursive
Rust-lexical census that excludes comments and normal, raw, byte, C-string and
character literals. It pins the attribute-free private roles, typed constructor,
six complete producers, receiver mapping, sole ownership-consuming projection,
call argument/result ordering, active CLI registration, and non-vacuous public
fixture.

This is source-equivalent type hardening. It does not change replacer
callability checks, traversal order, property-list behavior, `toJSON`, spacing,
cycle detection, Realm selection, parsing/reviver behavior, or conformance
counts. At `2026-08-27`, the authority structure target passes `5/5`, the
neighboring reviver-frame structure target passes `5/5`, and the exact public
Wasm-AOT fixture passes `1/1`. Broad JSON and Test262 verification remain
deferred.
