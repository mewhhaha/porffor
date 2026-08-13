# T06 — Realms, intrinsics and cross-realm semantics

**Status:** In progress — typed intrinsic and created-realm function foundations exist; full allocation and isolation remain

**Parallel group:** Core foundations  
**Depends on:** T03, T04, T05  
**Blocks:** T11-T14, T17, T21-T24

## Current repository state

Realm IDs, realm records, intrinsic metadata and realm-owned prototype
references are present in the runtime/backend. The current 23 intrinsic rows now
live in one declarative registry that generates `IntrinsicKind`, ordered
descriptors, callable `name`/`length` properties and 46 ordered property
templates. A closed `IntrinsicLink` relation distinguishes internal
`[[Prototype]]` inheritance from constructor/prototype own-property links.
Const validation pins the exact row/property counts, role compatibility and
reciprocal relationships, so incomplete registry additions fail compilation.
The design contract is recorded in
`docs/rust-rewrite/realm-intrinsics.md`.

Wasm-AOT created-realm function materialization now takes a private typed
`RealmRecordLocal` minted only by realm-record allocation. The 83 bootstrap
sites that previously allocated a function under the current realm and then
repaired its defining-realm header now go through one in-realm choke point;
the canonical `parseInt`/`parseFloat` installer delegates to the same path.
Environment/self-backing remains a separate choice because it has distinct
execution semantics. `GetFunctionRealm` now returns opaque result locals that
cannot expose their realm until a consuming route has handled both nonresolved
states. Constructor/default-prototype routes preserve the specified revoked
Proxy `TypeError`; Promise-job creation explicitly selects the specification's
current-realm fallback for a revoked callback. Every route traps a missing
defining realm or unknown callable representation as an internal invariant
failure instead of silently selecting a prototype.

Created-realm `%Array.prototype%` bootstrap now has a closed typed seam. A
reserved local must be consumed by Array-layout initialization before it can be
published, receive Array named properties, form the realm-local `%Array%` /
`%Array.prototype%` links, or be released. The general intrinsic writer accepts
a closed non-Array slot domain, while the Array slot has dedicated typed
created-realm and hard-coded entry-realm publication operations. The initialized
Array exotic points at the created realm's `%Object.prototype%`; its constructor
is born through a realm-aware `BootstrapSupplied` choke point without an
automatic plain prototype, and its links use the Array-aware descriptor path
and the exact ECMAScript attributes. Resolved-realm Array default-prototype
fallback requires the resolved realm's populated Array slot and preserves the
Array tag, with no entry-global substitution or payload identity heuristic.
The ordinary-object defaults selected by construction now use a separate closed
slot domain and a non-copyable loaded-prototype witness. Object, String, Number,
Boolean and Date construction require their resolved realm's populated
intrinsic slot and consume the witness together with its Object tag; missing
realm bootstrap state traps instead of selecting an entry-realm global. Date
reuses the same required fallback policy after its arity-specific value
calculation rather than the shared direct-constructor dispatcher.

This remains metadata foundation rather than full realm bootstrap. Intrinsic
objects are not yet independently allocated from these templates across the
complete ECMAScript set, the registry is not yet shared with `lila-ir`, and the
eleven focused `lila-runtime` contracts are green. `lila-engine` re-exports the
typed link relation with the rest of the public realm vocabulary.
Dynamic-source-dependent cross-realm cases remain explicit
unsupported cases, and no current complete Wasm-AOT aggregate proves the full
realm acceptance matrix. Complete intrinsic allocation, host-capability
scoping, teardown, borrowed builtins and realm-correct errors therefore remain
active work. The Array seam does not make `%Function.prototype%` callable or
repair the other intrinsic families or unrelated partial-bootstrap prototype
loaders.

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

The runtime registry is now the single source for its current rows and property
templates. Expanding it to the complete intrinsic set and making `lila-ir`
consume the same registry remain part of this work item.

- Generate intrinsic installation from one declarative registry shared with `lila-ir` builtin metadata.
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

Extend `lila-runtime::HostHooks` or replace it with typed capability traits. Host hooks must be scoped by realm/agent and may not expose spec-exec engine objects to product Wasm semantics. `createRealm` must produce a truly separate global and intrinsic graph.

## Acceptance criteria

- Two realms have distinct global objects and intrinsic identities.
- Cross-realm constructor/prototype fallback and thrown-error prototype tests pass without exact-test materialization.
- Builtin descriptors are generated from one registry and verified by unit tests.
- A function always retains the correct defining realm after binding, storage, proxy wrapping or cross-realm transfer.
- Realm destruction releases host resources only after JavaScript reachability allows it.
- No fallback returns the current realm when realm creation is unavailable; failures are explicit.

## Required tests

```sh
cargo test -p lila-runtime --quiet
cargo test -p lila-ir intrinsic_ --quiet
cargo test -p lila-aot-wasm realm_ --quiet
cargo test -p lila-spec-exec realm_ --quiet
cargo test -p lila-engine --quiet
```

Run real Test262 cases containing `createRealm`, `proto-from-ctor-realm`, `newtarget-proto-fallback`, cross-realm error constructors, species and borrowed builtins.
