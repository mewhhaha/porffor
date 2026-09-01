# Math builtin policy domains

Status: implemented for the nested capability-free Math dispatch domains.

## Closed routing

The private, capability-free `MathBuiltin` separates the 29 one-argument Math
operations from the eight algorithms with distinct argument or host behavior.
Its `Unary(MathUnaryBuiltin)` variant carries the complete one-argument choice;
the remaining variants are `Atan2`, `Hypot`, `Imul`, `Max`, `Min`, `Pow`,
`Random` and `SumPrecise`.

The nested, capability-free `MathUnaryBuiltin` names exactly the 29 operations
whose shared entry first coerces argument zero and then selects the result
algorithm. The standard dispatcher constructs one nested unary policy for each
of those namespace members. The Math emitter consumes `MathBuiltin` once and,
only for `Unary`, consumes the carried `MathUnaryBuiltin` through a second
exhaustive match.

Neither domain can be cloned, copied, formatted, defaulted, compared, ordered
or hashed. The previous inner arms for impossible non-unary operations are
gone: a non-unary operation cannot inhabit `MathUnaryBuiltin`, so adding or
misrouting an operation is now a compile error instead of an unreachable
runtime policy branch.

## Durable regression

`math_builtin_policy_domains_structure.rs` pins both exact domains, all 37
standard producers, the nested 29/8 split, the consuming exhaustive matches and
the absence of incidental capabilities, equality routing, wildcard arms and an
impossible unary fallback.

```sh
cargo test -p lila-aot-wasm --test math_builtin_policy_domains_structure --quiet
```

At the 2026-08-28 Batch AK checkpoint, `cargo xc` is green, the structure
target passes `4/4`, and the existing extremum, `hypot` and `sumPrecise` CLI
controls pass `3/3`. The exact `Math.abs` and `Math.round` Test262 leaves pass
all `4/4` Wasm-AOT variants with every failure bucket at zero. No semantic
golden was required or run.

Batch AK changes no coercion, emitted instruction or operation ordering and
claims no new Math behavior. It does not close platform-sensitive numeric
accuracy, randomness, the complete Math namespace or the full pinned Test262
tree.

## Batch AW dispatcher boundary

Both capability-free domains are now private to `math.rs`, together with the
raw exhaustive emitter. Standard dispatch reaches them only through 37 fixed Math entries,
one for each namespace operation. The frozen 825-line domain/emitter selection
has SHA-256
`25cedc56bf9f821608dad8f2c4b3d6b079a09279bbc5ca6e0703679d16e98049`;
restoring only the former enum and emitter visibility reproduces that source
exactly. `cargo xc` passes. The policy, extremum, `hypot`, `sumPrecise` limb and
`sumPrecise` runtime structure targets pass `4/4`, `3/3`, `3/3`, `3/3` and
`6/6`; the three established Math Wasm-AOT CLI controls pass `3/3`. No Test262
leaf or Wasm golden was required for this source-equivalent dispatcher boundary,
which claims no new Math behavior, conformance result or published-count change.
