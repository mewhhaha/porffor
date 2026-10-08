# Proxy `[[Delete]]` target traversal

## Completed target follow-up — 2026-10-04 dry source

A true outer trap now acquires the actual recursive target descriptor through
one private completed owner. Original descriptor throws precede outer
constraints; absence returns true, nonconfigurability fails before IsExtensible,
and a present configurable property on a nonextensible target still fails.
Nested errors and native Delete failures use the current operation Realm.
Inline source Delete feeds the active catch/finally handler rather than returning
past it. The prior CurrentCompletion revocation route is retired after its sole
producer moves to the existing Proxy execution Realm/active-handler route.

The target-descriptor and paired Realm controls are authored but unexecuted.
Compilation and full T11/Test262 acceptance remain pending. See the
[completed target contract](proxy-target-descriptor-completion.md). The
traversal checkpoint below retains its original narrower source/evidence.

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

The original traversal slice did not make the direct post-trap descriptor
fact recursively Proxy-aware. That source gap is addressed by the newer shared
completed target follow-up above; its compile/runtime verification remains
pending. Full Proxy, Reflect and Test262 acceptance remain open.
