# `Promise.withResolvers` result Realm

Status: implemented with focused source and finite runtime verification on
2026-08-26.

## Ownership

`Promise.withResolvers` first creates a Promise capability from constructor
`C`, then creates the ordinary `{ promise, resolve, reject }` result in the
executing builtin's Realm. Constructor `C` owns the Promise and its resolving
functions. It does not select the result object's `%Object.prototype%`.

This distinction is observable when a created-Realm method is borrowed with
the entry Promise constructor, and in the reverse direction when the entry
method is called with a created-Realm Promise constructor.

## Typed allocation

The private
`builtins/promise/promise_with_resolvers_result_allocation.rs` owner contains
the non-`Copy`, must-use `PromiseWithResolversResultAllocationContext`, its sole
factory and its consuming installer. The methods are visible only to the parent
Promise family. The parent may pass their inferred context at the retained
`Promise.withResolvers` call site, but cannot name, import, re-export, construct
or project its raw prototype local. An entry builtin with a zero environment
explicitly selects the entry `%Object.prototype%`. A self-backed created-Realm
builtin must provide its defining Realm, intrinsic record and Object-prototype
slot; missing catalog state traps instead of substituting an entry or
dynamically current Realm.

The context factory reserves prototype, Realm and intrinsics locals in that
order, releases intrinsics then Realm, and leaves only the prototype proof for
the one-shot installer. The fallible plain-object allocation completes before
the context is acquired. The unexposed null-prototype shell then receives the
proved prototype payload and Object tag, and the installer releases the proof.
No emission-error path can strand the context.

Capability creation remains before the raw result allocation, context
acquisition and prototype installation. Property creation retains the required
`promise`, `resolve`, `reject` order and writable, enumerable and configurable
attributes.

This batch adds no heap field, initializer or trace obligation. Created Promise
static methods already carry self environments and defining Realms.

## Focused evidence

The recursive source target pins the private child, sole four-use context, one
construction, two projections, both child-owned methods, explicit entry branch,
strict created-Realm catalog traps, reverse-order lifecycle and exact one-caller
census. It also preserves capability-before-result order in the parent. The
finite fixture checks both borrowed-method directions and separately observes
the result Object, capability Promise and resolving-function prototypes without
invoking either resolver or queuing a reaction.

The source-equivalent owner move selected the exact four-line context block at
SHA-256
`a65d951c68f000af43899262f7e8fb82dbc543c1849024841fb5aa9af98a20e2`
and the exact 74-line method block at SHA-256
`7711cc79c15e769c56e3f642e9e0cb538ecddbcfeb1db1c07945f7d6ff08306b`.
Their combined 78 selected lines retain SHA-256
`58a16feba356e97d5fad4bac3311c632335a03d7105df42c9492347c3fb448bf`.
The resulting 83-line child has SHA-256
`3c3ee3ee1187f412e1b0a8232ca6a77e03fc94d7e3d0f50d4dd3e0d68163ba2a`,
and the parent decreases from 9,391 to 9,312 lines. Method and caller bodies are
unchanged; only child ownership and parent-facing `pub(super)` spelling were
added. The focused recursive owner target passes `5/5`, and the neighboring
created-allocation-Realm target passes `7/7`. The shared `cargo xc` checkpoint,
module-boundary, task-plan, formatter and diff-hygiene checks pass. The exact
created-Realm Promise publication CLI witness passes `1/1`. A semantic golden
does not apply to this source-equivalent owner move.

```sh
cargo test -p lila-aot-wasm --test promise_with_resolvers_result_realm_structure --quiet
cargo test -p lila-aot-wasm --test created_realm_promise_publication_structure --quiet
cargo test -p lila-cli --test cli run_wasm_backend_publishes_created_realm_promise_foundation --quiet
```

The following workspace semantic golden passes `2/2` in 704.11 seconds with
666 dumps. It adds only the independent array key-selection witness, removes
none and preserves all 665 retained non-accounting summaries.

## Deferrals

This boundary does not change `PromiseResolve`, combinator AggregateError
construction, async builtin Promise allocation, async-generator iterator-result
objects or broader Promise job Realm switching. Shared README, task and Realm
status indexes are updated by their owning integration lane.
