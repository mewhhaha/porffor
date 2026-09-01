# Static strict-equality kind domain

The singleton fast path in `compile_strict_equality_i32` exhaustively assigns
every `ValueKind` to one of four existing algorithms. Number uses binary64
equality, String uses code-unit equality, Function and BigInt use tagged
payload equality, and the primitive identity and ordinary heap-reference kinds
use raw-payload equality. Dynamic explicitly uses tagged equality rather than
inheriting an accidental raw-payload default.

There is no wildcard or defensive unreachable arm. Adding a value kind cannot
compile until strict equality selects its representation-aware algorithm. The
general runtime-tag path remains unchanged for expressions whose tags or kind
sets are dynamic at runtime.

This source-equivalent classification preserves the unequal-singleton early
exit, operand order, String scratch-local discipline, tagged temporary-local
lifecycle and completion publication.

```sh
cargo test -p lila-aot-wasm --test strict_equality_static_kind_structure
cargo test -p lila-cli --test cli language_numerics::run_wasm_backend_succeeds_for_spec_strict_equality_fixture -- --exact --test-threads=1
```

The bounded pinned controls are `S11.9.4_A5.js`, `S11.9.4_A7.js` and
`bigint-and-bigint.js` under `language/expressions/strict-equals`. The total
domain target passes `3/3`, the exact CLI witness passes `1/1`, and the three
pinned controls pass all `6/6` Wasm-AOT executions with every failure bucket at
zero. The shared `cargo xc`, formatting, diff, module-boundary and task-plan
checks are green.
