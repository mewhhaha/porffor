# T06 — Realms, intrinsics and cross-realm semantics

**Status:** Blocked on T04/T05  
**Parallel group:** Core foundations  
**Depends on:** T03, T04, T05  
**Blocks:** T11-T14, T17, T21-T24

## Objective

Turn the minimal Rust `Realm` shell and backend-specific prototype slots into a first-class ECMAScript realm model with independently allocated intrinsics, global environment, host hooks and realm-correct error creation.

## Required model

Each realm must own or reference:

- a unique realm ID and agent association;
- the global object, global `this` value and global environment record;
- an intrinsic table containing every constructor, prototype, iterator prototype, well-known function and `%ThrowTypeError%`;
- template maps for builtin properties and exact descriptors;
- job queue/host-defined data interfaces;
- locale/time-zone hooks used by Date/Intl/Temporal;
- module registry and host loader hooks;
- dynamic-source policy from T13.

Do not encode realm identity as a collection of one-off function header fields. Use a general reference from functions and builtin objects to their defining realm.

## Intrinsic bootstrap

- Generate intrinsic installation from one declarative registry shared with `porffor-ir` builtin metadata.
- Define constructor/prototype links, method `name`/`length`, writable/enumerable/configurable attributes and well-known-symbol properties in data, not repeated emitter code.
- Allow feature modules to register their intrinsic families without editing one giant bootstrap match.
- Validate that all references resolve, property keys are unique and every builtin function has a defining realm.

## Cross-realm behavior

Implement and test:

- `OrdinaryCreateFromConstructor` fallback to the new target's realm;
- error objects created in the realm required by the invoked function/operation;
- cross-realm prototype and `instanceof` behavior;
- calling borrowed builtin methods across realms;
- realm-local `%Array.prototype%`, `%TypeError.prototype%`, iterator prototypes and species constructors;
- object identity and wrapper behavior across `$262.createRealm()`;
- teardown that cannot invalidate still-reachable objects.

## Host integration

Extend `porffor-runtime::HostHooks` or replace it with typed capability traits. Host hooks must be scoped by realm/agent and may not expose spec-exec engine objects to product Wasm semantics. `createRealm` must produce a truly separate global and intrinsic graph.

## Acceptance criteria

- Two realms have distinct global objects and intrinsic identities.
- Cross-realm constructor/prototype fallback and thrown-error prototype tests pass without exact-test materialization.
- Builtin descriptors are generated from one registry and verified by unit tests.
- A function always retains the correct defining realm after binding, storage, proxy wrapping or cross-realm transfer.
- Realm destruction releases host resources only after JavaScript reachability allows it.
- No fallback returns the current realm when realm creation is unavailable; failures are explicit.

## Required tests

```sh
cargo test -p porffor-runtime --quiet
cargo test -p porffor-ir intrinsic_ --quiet
cargo test -p porffor-aot-wasm realm_ --quiet
cargo test -p porffor-spec-exec realm_ --quiet
cargo test -p porffor-engine --quiet
```

Run real Test262 cases containing `createRealm`, `proto-from-ctor-realm`, `newtarget-proto-fallback`, cross-realm error constructors, species and borrowed builtins.