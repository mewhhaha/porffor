# Array-destructuring iterator steps have a closed value-read policy

Status: structure-verified on 2026-08-27.

## Closed domain

The shared array-destructuring iterator-step emitter accepts one private
`DestructuringIteratorStepKind`:

- `Elision` performs `IteratorStep` but must not perform `IteratorValue`;
- `Value` performs `IteratorValue` after the iterator result reports
  `done: false`.

The domain derives no cloning, copying, debugging, equality or default
capability. Its sole consumer matches it exhaustively at the value-read point;
there is no equality predicate, wildcard or implicit Boolean fallback for a
future step kind to inherit.

## Producer and ordering boundary

The three producers are confined to the array-element compiler. The elision
arm selects `Elision`, while the target and rest arms each select `Value`.
Target preparation and rest-array allocation remain outside this policy.

The shared consumer still calls `next`, validates the result object, reads and
coerces `done`, and initializes the output to `undefined` before interpreting
the step kind. Only the `Value` arm reads the `value` property and routes an
abrupt getter completion before clearing the local done guard. The match emits
the same instructions in the same order as the former `matches!` branch, so
this source-equivalent closure is expected to leave emitted Wasm
byte-identical.

## Verification

The recursive bounded structure target pins the private attribute-free
two-variant declaration, complete seven-mention ownership census, exact
one-elision/two-value producer mapping and the full ordered value-read arm:

```console
cargo test -p lila-aot-wasm --test destructuring_iterator_step_kind_structure
cargo test -p lila-cli --test cli array::run_wasm_backend_uses_iterators_for_array_destructuring -- --exact --test-threads=1
cargo test -p lila-cli --test cli array::run_wasm_backend_preserves_array_destructuring_iterator_abrupt_completions -- --exact --test-threads=1
```

The ordinary iterator fixture covers target and rest value reads. The abrupt
fixture gives an elided iterator result an observable `value` getter and
requires zero getter calls, directly distinguishing the two variants.
The structure target passes `4/4`, and those two exact CLI witnesses pass
`2/2`. Independent review confirmed the complete seven-mention/capability
closure, exact three producers, full ordered value-read arm and preserved
instruction order. Coordinated `cargo xc`, full formatter, diff,
module-boundary and task-plan checks are green.

## Non-claims

This closure adds no destructuring behavior. It does not change iterator
acquisition, `IteratorClose`, default initializers, nested patterns, assignment
results, object destructuring, async iteration or resource management. No
semantic golden, broad Test262 filter or published status refresh is claimed.
