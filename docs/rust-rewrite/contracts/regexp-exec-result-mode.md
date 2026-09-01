# RegExp execution result mode

Status: normative for Lila's internal RegExp execution emitters.

## Boundary

`FunctionBuilder::emit_regexp_prototype_exec_from_locals` routes an intrinsic
RegExp operation through the emitted bytecode-program matcher, the simple
matcher and a final legacy-pattern fallback. Each route must produce one of
exactly two JavaScript result shapes:

| Mode | Producers | Observable result |
| --- | --- | --- |
| `MatchArrayOrNull` | non-global `RegExp.prototype[Symbol.match]` and `RegExp.prototype.exec` | a match Array on success and `null` on failure |
| `Boolean` | the intrinsic fallback in `RegExp.prototype.test` | `true` on success and `false` on failure |

The private `RegExpExecResultMode` derives no cloning, copying, debugging,
equality or default-construction capability. The wrapper owns the authority,
lends it to the bytecode-program and simple matchers, and consumes it only in
its final projection. Seven direct exhaustive matches select result
materialization and the bytecode matcher's capture-carrier lifetime. There is
no Boolean projection, default or wildcard arm. A newly added result mode
therefore cannot silently inherit an allocation, cleanup or result-shape
policy.

This is Rust-time emitter state only. It adds no emitted Wasm word or ABI and
does not change temporary-local reservation, matcher instruction order,
`lastIndex` coercion or `lastIndex` update order.

The callable custom-`exec` branch of `RegExp.prototype.test` remains outside
this boundary. That branch observes the user-supplied call result and converts
object/null to Boolean as required by the public RegExp protocol; it does not
invoke the internal matcher seam.

## Durable witnesses

`regexp_exec_result_mode_structure.rs` pins the exact private, attribute-free
two-variant domain, its absent capabilities, one owning and two borrowed typed
parameters, ordered forwarding, seven direct exhaustive projections, the
recursive 21-mention ownership census and exact three-producer mapping.

`wasm_regexp_exec_result_modes.js` selects every internal matcher route:

- a static capture pattern selects the bytecode-program matcher and exercises
  non-global `@@match`, `exec` success/failure and `test` success/failure;
- a pattern assembled with `String.fromCharCode` has no exact source literal
  and selects the simple matcher; and
- an unseen `(?:)` source assembled from separate array elements reaches the
  final legacy fallback.

The fixture distinguishes Array/null from Boolean results and preserves
captures, indices, inputs and global `lastIndex` updates/resets.

## Focused verification

```sh
cargo test -p lila-aot-wasm --test regexp_exec_result_mode_structure
cargo test -p lila-cli --test cli regexp::run_wasm_backend_preserves_regexp_exec_result_modes -- --exact --test-threads=1
./scripts/check-module-boundaries.sh
cargo fmt --all -- --check
git diff --check
```

Independent dry review is clean after the structure guard was strengthened to
bind the three signatures, all seven lexically normalized match bodies and the
borrow-borrow-consume order. The shared `cargo fmt --all -- --check`, `cargo
xc`, diff, module-boundary and task-plan checkpoint is green with the
workspace's existing warnings.

The following workspace semantic golden passes `2/2` in 707.16 seconds with
665 dumps. It adds only this fixture, removes none, and preserves 663 of 664
retained non-accounting summaries; the sole retained structural change is the
independently expanded Promise Realm witness.

## Deferrals

This source-equivalent type closure does not change the callable custom-`exec`
protocol, global `@@match`, arbitrary runtime pattern compilation, RegExp
grammar or matcher opcodes, Realm allocation, match Array descriptors,
Test262 shortcut retirement, broad RegExp/Test262 execution, or conformance and
status publication.
