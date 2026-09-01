# Named may-throw operation completion ownership

The first two shared may-throw operation wrappers own their fixed completion
policies directly. `compile_property_get_v_to_locals` selects the exact `GetV`
descriptor and then propagates a throw to the active handler using the result
locals. `emit_builtin_arg_to_number_payload` converts argument zero, removes
the conversion result from the Wasm stack and then returns the current function
when the completion is a throw.

The generic `AbruptRoute` is gone, together with
`finish_may_throw_operation`. Both variants had exactly one producer, so the
shared enum only made the two wrong combinations expressible inside the
module: GetV could select current-function return, and builtin ToNumber could
select active-handler propagation. Inlining each fixed continuation into its
named wrapper makes both mismatches unrepresentable and removes an abstraction
with no shared policy selection.

The bounded Rust-lexical regression recursively rejects either deleted generic
symbol. It separately pins descriptor/conversion selection, stack cleanup and
the exact named continuation order in both wrappers. This is a
source-equivalent ownership invariant; it changes neither the completion ABI
nor emitted Wasm.

Focused verification on 2026-08-28:

```sh
cargo test -p lila-aot-wasm --test may_throw_abrupt_route_ownership_structure
```

The focused structure target passes all 4 tests. The shared `cargo xc`
checkpoint is green. The exact Number builtin-family and abrupt Iterator-helper
dispatch CLI witnesses each pass `1/1`, covering the fixed current-function and
active-handler continuations. Test262 does not apply to deletion of the generic
Rust route.
