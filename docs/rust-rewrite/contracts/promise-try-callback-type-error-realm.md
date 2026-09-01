# `Promise.try` callback TypeError Realm

Status: implemented with focused source and finite runtime verification on
2026-08-26.

## Ownership

`Promise.try` first creates a capability from constructor `C`, then calls its
callback with the remaining arguments. A non-callable callback produces a
TypeError in the executing `Promise.try` function's Realm. The resulting Throw
completion is passed to the already-created capability's reject function; it
is not returned synchronously from the builtin.

The callback error Realm is therefore owned by the borrowed static method, not
by constructor `C`, the caller's current Realm or the capability Promise.

## Typed boundary

The private `builtins/promise/promise_try_callback_type_error.rs` owner contains
the non-`Copy`, must-use `PromiseTryCallbackTypeErrorPrototypeLocal`, its only
factory and its consuming throw emitter. The two methods are visible only to
the parent Promise family. The parent may pass their inferred proof between
them, but cannot name, import, re-export, construct or project its raw prototype
local. The factory explicitly selects the entry TypeError prototype for a zero
standard-builtin environment. A nonzero environment loads the self-backed
TypeError-prototype snapshot published on that `Promise.try` function and traps
if the slot is absent. It does not consult `CURRENT_REALM`, the Promise
constructor or a dynamic constructor catalog.

The one-shot consumer creates the TypeError with the proved prototype and
releases the local. `Promise.try` acquires the proof only in the invalid
callback branch. The callable branch retains the shared function-or-Proxy call
emitter unchanged.

Capability construction remains before callback and argument-vector work.
Argument copying remains before callback classification. Both callback
branches join at the existing completion check, which selects reject for Throw
and resolve otherwise before returning the capability Promise.

## Focused evidence

The recursive source test pins the private child, sole opaque proof,
construction, projection and method ownership, explicit entry selection,
strict self-backed snapshot trap, one-shot release, callback branch shape and
capability/argument/settlement ordering. The finite CLI witness borrows
created-Realm `Promise.try`, supplies a primitive callback, and checks the
asynchronously observed rejection against that Realm's TypeError prototype.
Its success sentinel is gated by the rejection checkpoint.

The source-equivalent owner move selected the exact two-line proof block at
SHA-256
`ab90ccc6decb25132becf4d66e08b2ee989ea3795974428d29cc59ccd7b60737`
and the exact 46-line method block at SHA-256
`b236edae20d47dc546d9c641ddb6841086ee117205610fb9de4fc19c8c3163f1`.
Their combined 48 selected lines retain SHA-256
`8f17cb7d9f467e43b229d96e6354fbce421746b6d01518b7491b5bab513b6eb2`.
The resulting 53-line child has SHA-256
`64b2ce31da7fe3ac71a60e93302388ed0fd2ef81c390d0ce019e395ed5c6aff3`,
and the parent decreases from 9,606 to 9,558 lines. Method and caller bodies are
unchanged; only child ownership and parent-facing `pub(super)` spelling were
added. The recursive structure target passes `5/5`; module boundaries,
task-plan policy, focused formatting and diff hygiene are green. The exact
created-Realm Promise internal-callback CLI witness passes `1/1`, and the
shared `cargo xc` checkpoint is green with only the workspace's existing
warnings. Semantic goldens were not rerun for this source-equivalent owner
move.

```sh
cargo test -p lila-aot-wasm --test promise_try_callback_error_realm_structure --quiet
cargo test -p lila-aot-wasm --test created_realm_promise_publication_structure --quiet
cargo test -p lila-cli --test cli run_wasm_backend_preserves_created_realm_promise_internal_callbacks --quiet
```

The following workspace semantic golden passes `2/2` in 702.89 seconds with
667 dumps. It adds only the independent iterator-policy witness, removes none,
and preserves 665 of 666 retained non-accounting summaries. This expanded
Promise callback witness alone gains one internal/named function and two main
locals.

## Deferrals

This boundary does not change callable callback behavior, PromiseResolve,
general AggregateError construction, combinator result ownership, async
Promise allocation, async-generator requests or broader error construction.
Shared README, task and Realm indexes remain with their integration owner.
