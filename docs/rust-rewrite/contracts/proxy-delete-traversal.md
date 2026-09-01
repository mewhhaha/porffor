# Proxy `[[Delete]]` target traversal

## Boundary

`emit_object_delete` owns the complete absent-trap target traversal. It copies
the caller's object payload and tag into dedicated current-target locals, then
emits one Wasm loop. Every iteration reads the current Proxy's live target and
handler slots, performs full handler `[[Get]]`, propagates an abrupt lookup,
and classifies the resulting `deleteProperty` method.

A null or undefined method replaces both current-target locals with the exact
typed Proxy target and continues the loop. A callable method is invoked with
the retained tagged handler as `this`, the target and the already-converted
property key; its Boolean result passes through the existing direct descriptor
invariant before the traversal completes. Another present value throws. A
non-Proxy current target reaches the ordinary representation-aware delete
operation once.

The emitted state has three named values: inspect the current target, complete
the trapped delete, or follow the current Proxy target. The former recursive
Rust emitter and its integer depth parameter are deleted, so increasing the
runtime Proxy chain no longer duplicates instructions, locals, or source-level
control frames. Dedicated current-target locals also keep this operation from
mutating the caller's object operands.

## Evidence

The focused CLI fixture crosses six nullish forwarding handlers before one
innermost callable trap. Getter observations require the exact outside-in
order, the trap must run once, and the property must be absent from the
ultimate ordinary target. The existing fixture retains callable-Proxy traps,
handler representation tags, abrupt lookup and call identity, revocation,
Boolean conversion, descriptor invariants, ordinary fallback, direct delete,
and `Reflect.deleteProperty` coverage.

The structural boundary in
`crates/lila-aot-wasm/tests/proxy_delete_traversal_structure.rs` rejects a
source-generated depth emitter or recursive call, pins the single loop and its
three transitions, and retains the exact three nested-target Test262 files.
They have no single-mode flag: three physical files and six executions.

At 2026-09-01, `cargo check -p lila-aot-wasm` is green. The traversal and
adjacent revocation-route structure targets each pass `4/4`, and the exact CLI
fixture passes `1/1` through emitted Wasm. The missing, null and undefined
nested-target Test262 files each pass both default modes, for `6/6` total with
every failure bucket at zero. The fixture also passes JavaScript syntax and
reference-semantics evaluation under Node. Independent dry review found the
loop labels, state transitions, local ownership and operation order clean.
Formatting, diff, task-plan, module-boundary and exact 186-entry shortcut gates
are green.

## Nonclaims

This slice does not make the direct post-trap descriptor fact recursively
Proxy-aware. If a callable outer trap returns true and its target is itself a
Proxy, complete recursive `[[GetOwnProperty]]` compatibility remains separate
T11 work. This is not a full Proxy, Reflect, or Test262 claim.
