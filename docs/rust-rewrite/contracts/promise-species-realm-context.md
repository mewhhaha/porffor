# Promise SpeciesConstructor Realm context

Status: implemented with focused source and finite runtime verification on
2026-08-26.

## Ownership

`Promise.prototype.then` and `Promise.prototype.finally` call
`SpeciesConstructor(promise, %Promise%)`. The default `%Promise%` constructor
and both TypeErrors created by that abstract operation belong to the defining
Realm of the executing prototype method. The Promise receiver, its constructor
property, the caller and a currently installed Promise-job Realm are not
default intrinsic authority.

This distinction is observable when a created-Realm method is borrowed onto an
entry Promise whose `constructor` is undefined, primitive or exposes a
non-constructor `@@species` value.

## Paired context

`PromiseSpeciesRealmContext` is private, non-`Copy` and must-use. It pairs the
default Promise constructor with the TypeError prototype selected from one
defining-Realm intrinsic catalog. A zero standard-builtin environment
explicitly selects the entry globals. A self-backed nonentry method must
provide its defining Realm, intrinsic record, Promise-constructor slot and
TypeError-prototype slot; each missing link traps without an entry or dynamic
current-Realm fallback.

The private `builtins/promise/promise_species_realm_context.rs` child owns the
paired context, its only factory and the consuming SpeciesConstructor helper.
The Promise parent neither imports nor re-exports the context. Its retained
`then` and `finally` callers can pass only the inferred value between
child-owned methods, so adjacent Promise algorithms cannot construct a mixed
catalog or project either raw local.

The two result locals are reserved before Realm and intrinsics temporaries.
The factory releases intrinsics then Realm. `then` and `finally` acquire the
context only after their receiver precondition and pass it by value to the sole
SpeciesConstructor helper. That helper releases its six internal locals,
TypeError prototype and default constructor in strict reverse order before
propagating an emission failure.

The helper initializes its default from the context, preserves the existing
`constructor` Get and validation before the `@@species` Get and validation,
and constructs both native TypeErrors through the paired prototype. `then`
still creates its new Promise capability only after SpeciesConstructor.
`finally` still materializes its cleanup closures only after SpeciesConstructor.

This batch changes no heap slot, initializer, trace layout or publication
catalog. Entry and created Promise prototype methods already have the function
identity and defining-Realm authority required by the context factory.

## Focused evidence

The bounded source target pins context opacity, same-catalog selection,
explicit zero-environment handling, four strict nonentry traps, exact caller
census, sole construction, exact two/three field projection census,
reverse-order cleanup and observable algorithm order. The finite CLI
witness checks the borrowed `then` default-derived Promise prototype and both
borrowed-method TypeError prototypes, then drains through the retained finite
Promise callback checkpoint.

The source-equivalent extraction selected the exact six-line context at
SHA-256
`4c668cadeb82c06ec0e1d66e0c8baf3c1417d407e077e8400b7671af0158dfd2`,
71-line factory at SHA-256
`676d21f916e740dd4e90a92c87c27970a0371e6d23ed8ef20b8c3d37445f3aa5`
and 110-line consumer at SHA-256
`e9faaabf5ea9fac01908d9c653caba9f64d7850fded52961ac4c1ddc5c2abe46`.
Their combined 187 selected lines retain SHA-256
`3ce5bdf128aa01e4f1586af03ed56f48a98d4e64f4d28430b59d5826471c4188`.
The 190-line child has SHA-256
`91c2fe569f01c52e088cd49c1299a51afc7a3e3caf16b486d2419fd84e077d3e`
and reduces `promise.rs` from 9,224 to 9,038 lines. The recursive ownership
target passes `6/6`; the adjacent receiver-order target passes `8/8`.

```sh
cargo test -p lila-aot-wasm --test promise_species_realm_context_structure --quiet
cargo test -p lila-aot-wasm --test created_realm_promise_publication_structure --quiet
cargo test -p lila-cli --test cli run_wasm_backend_preserves_created_realm_promise_internal_callbacks --quiet
```

## Shared verification

The existing finite created-Realm Promise callback CLI witness passes `1/1`.
The shared `cargo xc`, workspace formatting, diff, module-boundary and task-plan
checks are green. The semantic golden remains deferred because the factory,
consumer and both caller bodies are unchanged apart from child visibility; no
new behavior or conformance claim is made.

The direct incompatible-receiver errors in `then` and `finally` are owned by
the adjacent
[`promise-prototype-receiver-error-realm.md`](promise-prototype-receiver-error-realm.md)
boundary. The adjacent
[`promise-resolve-realm-context.md`](promise-resolve-realm-context.md) boundary
owns PromiseResolve materialization and the NewPromiseCapability errors reached
through those operations. This context does not change combinator iterator
errors, general AggregateError construction or Promise allocation in other
async builtins.
