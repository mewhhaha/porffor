# Promise combinator outer Array Realm

Status: implemented with focused source and finite runtime verification on
2026-08-26.

## Ownership

`Promise.all`, `Promise.allSettled` and `Promise.any` allocate one internal
Array before iterating their input. The Array belongs to the defining Realm of
the executing combinator method. Constructor `C` selects the returned Promise,
but it does not select this Array's prototype. A later reaction job only fills
the already-owned Array and cannot replace its Realm authority.

The common standard-combinator emitter owns the only allocation. `Promise.all`
and `Promise.allSettled` resolve their capability with that Array. `Promise.any`
uses it as the `errors` value on its AggregateError for both empty input and the
last rejection.

Keyed combinators are excluded because their outer collection is deliberately
a null-prototype object. `Promise.race` has no result collection.

## Typed allocation

`CurrentFunctionRealmArrayPrototypeLocal` is opaque, non-`Copy` and must-use.
Its complete proof lifecycle has one private
`functions/current_function_realm_array_prototype.rs` owner. The parent neither
imports nor re-exports the witness; callers infer it through the existing load
and consuming-install methods. Only the child can construct the tuple witness
or project its raw local, so an unrelated scratch local cannot be presented as
the active function Realm's Array prototype.

For an entry standard builtin with a zero environment, its factory explicitly
selects the entry `%Array.prototype%`. A self-backed builtin must provide a
nonzero defining Realm, intrinsic record and Array-prototype slot; missing
catalog state traps and cannot silently substitute the entry global. The
factory never reads the dynamic current-Realm global.

The one-shot Array allocator completes its fallible raw allocation before
acquiring the prototype proof, then immediately consumes that proof while
installing the final prototype and representation tag. The proof factory
reserves prototype, Realm and intrinsics locals in that order, releases
intrinsics then Realm, and leaves the prototype local for the consuming
installer. No emission-error path can strand the proof.

This batch changes no heap size, offset, initializer or trace layout. The Realm
Array-prototype slot and the combinator shared-context pointer already exist.

## Focused evidence

The bounded source checks pin strict catalog loading, explicit zero-environment
entry selection, reverse-order lifecycle and the sole common combinator call.
They also pin the private module, sole witness/method owner, one tuple
construction, two raw-local projections, the one load/install pair after
successful raw allocation, and the Promise/Iterator pair of higher-level
allocator consumers. The exact 83 moved source lines retain SHA-256
`5e46c40f4844d54c08c0d74f25cc6046fef7063ad255be7e73add4dd5e87b490`;
the 88-line private child has SHA-256
`58863a33bd4367870947a6b562b619e5fd1559e4464347fa96212ba472c09739`.
The finite fixture borrows all three methods from a created Realm while passing
the entry Promise constructor. It observes entry-Realm returned Promises and
created-Realm Arrays for `all`, `allSettled`, nonempty `any` and empty `any`.

```sh
cargo test -p lila-aot-wasm --lib iterator_to_array_allocation_uses_the_active_function_realm --quiet
cargo test -p lila-aot-wasm --test promise_callback_created_allocation_realm_structure --quiet
cargo test -p lila-cli --test cli run_wasm_backend_uses_callback_realms_for_promise_created_allocations --quiet
```

The following workspace semantic golden passes `2/2` in 707.16 seconds with
665 dumps. It adds only the independent RegExp result-mode fixture, removes
none, and preserves every retained non-accounting summary except this expanded
Promise witness. That witness gains two internal/named functions and four main
locals for the added cross-Realm branches.

The private-owner extraction itself runs only the two scoped structure targets,
module-boundary and task-plan audits, scoped formatting and `git diff --check`.
The CLI witness, semantic golden, workspace compilation and broad suites remain
deferred to the coordinated shared checkpoint; the earlier semantic results
above are not re-claimed as fresh extraction evidence.

## Deferrals

This boundary does not change keyed combinators, `Promise.race`, general
AggregateError construction, PromiseResolve constructor or surrogate
ownership, async builtin allocation, or broader Promise job Realm switching.
