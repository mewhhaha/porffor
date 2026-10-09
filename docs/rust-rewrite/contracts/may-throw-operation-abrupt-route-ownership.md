# Named may-throw operation completion ownership

The generic `AbruptRoute` is gone, together with `finish_may_throw_operation`.
The GC backend keeps named operations responsible for their own complete
`CompletionLocals`, so mixing a value payload with an unrelated throw route is
unrepresentable at those interfaces.

`compile_spec_operation_to_locals` handles GetV with its actual ToObject and
property Get semantics. Its common tail copies the whole pending completion,
publishes a value only for Normal, and propagates Throw to the active handler.
`emit_math_coerce_number` performs builtin ToNumber, copies a Throw to the
builtin's output and branches to its cleanup edge before reading Number bits.
These current owners replace the retired scalar-local wrapper names.

The source guard rejects both deleted generic symbols recursively and pins the
actual conversion, whole-completion transport, normal-only publication and
named cleanup order. Current compilation and native verification is pending.
The following checkpoint predates this GC-owner migration.

Focused verification on 2026-08-28:

```sh
cargo test -p lila-aot-wasm --test structure_language -- may_throw_abrupt_route_ownership_structure::
```

The focused structure target passes all 4 tests. The shared `cargo xc`
checkpoint is green. The exact Number builtin-family and abrupt Iterator-helper
dispatch CLI witnesses each pass `1/1`, covering the fixed current-function and
active-handler continuations. Test262 does not apply to deletion of the generic
Rust route.
