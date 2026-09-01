# Generator delegate protocol-error authority

Status: implemented as a source-equivalent Wasm-AOT invariant boundary.

`GeneratorDelegateProtocolError` is the private eight-row authority for every
TypeError raised directly by synchronous and asynchronous `yield*` delegation.
The rows distinguish a non-iterable target, a non-callable iterator method, an
iterator method's non-object result, a later non-object iterator result, a
missing `throw` method, and non-callable `return`, `throw` and `next` methods.
The domain derives no cloning, copying, equality, debugging or default
capability.

All eighteen producer sites name one row. The shared callability and
object-result checks accept the domain rather than a string, while the five
direct failure paths invoke the same typed emitter. That emitter owns the sole
exhaustive row-to-message projection and the only raw runtime-error emission in
the delegation module. Adding a row without a diagnostic is therefore an
exhaustiveness error, and passing an arbitrary or misspelled diagnostic from a
producer is a type error.

`generator_delegate_protocol_error_authority_structure.rs` pins the private
eight-row declaration, exact eighteen-producer census, typed shared-check
signatures, one occurrence of every diagnostic and the wildcard-free
projection. The standalone dependency-free structure executable passes `4/4`;
`rustfmt --check` passes for both changed Rust files, and `git diff --check`
passes for the four-file invariant diff.

This boundary changes Rust authority only. It preserves the existing messages,
throw propagation and emitted instruction order; it adds no generator behavior,
conformance support or broad T15 claim. No Cargo compile, CLI fixture, Test262
cohort or semantic golden was run for this source-equivalent follow-up.
