# RegExp constructor active function and realm prototype

## Scope

This contract spans the closed active-standard-builtin and ordinary default-
prototype domains, the RegExp constructor body, created-realm constructor
self-backing, direct construct dispatch, structural guards, and one focused
source-free CLI fixture. Existing entry- and created-realm
`%RegExp.prototype%` publication are verified inputs; this seam changes no heap
layout or realm-bootstrap protocol.

When `RegExp` is called without `new`, the
[`RegExp ( pattern, flags )`](https://tc39.es/ecma262/multipage/text-processing.html#sec-regexp-constructor)
algorithm replaces an undefined `NewTarget` with the active function object.
Construction then reaches
[`RegExpAlloc`](https://tc39.es/ecma262/multipage/text-processing.html#sec-regexpalloc),
which performs `OrdinaryCreateFromConstructor(NewTarget,
"%RegExp.prototype%")`. The shared
[`GetPrototypeFromConstructor`](https://tc39.es/ecma262/multipage/ordinary-and-exotic-objects-behaviours.html#sec-getprototypefromconstructor)
rule performs one observable `Get(NewTarget, "prototype")`; a primitive result
selects the `%RegExp.prototype%` intrinsic of `GetFunctionRealm(NewTarget)`, not
the entry realm's prototype.

## Closed active-function normalization

Created-realm standard builtins share emitted bodies with their entry-realm
counterparts. `ActiveStandardBuiltinFunction` therefore names
`RegExpConstructor` and maps it exhaustively to the entry-realm RegExp global.
Its payload operation chooses the self-backed created-realm constructor when a
function environment exists and the entry global otherwise.

A private normalization operation consumes that typed active identity only
when the RegExp body's `NewTarget` tag is `undefined`. It writes the selected
payload and the Function representation tag into the existing new-target
locals. Explicit new targets remain byte-for-byte untouched. The created-realm
RegExp constructor is self-backed in the same way as the Iterator constructor,
so a borrowed `other.RegExp(...)` call uses `other.RegExp`, not the entry
constructor, as the active function.

This normalization is intentionally distinct from RegExp's still-open
`IsRegExp(pattern)` and same-constructor early-return rule. It establishes only
the active function identity required by the subsequent allocation path.

## Required fallback and tagged allocation

`OrdinaryDefaultPrototype` is the closed domain of ordinary-object intrinsic
defaults loaded after `GetFunctionRealm`. `RegExp` is a member and maps
exhaustively to the existing realm-intrinsics `%RegExp.prototype%` slot.

After active-function normalization, the RegExp body must:

1. perform exactly one observable `Get(NewTarget, "prototype")`;
2. resolve the original new target's function realm only if that result is a
   primitive;
3. route revoked and invalid realm results before exposing a realm local;
4. load the selected required RegExp prototype slot, trapping missing
   bootstrap state; and
5. allocate exactly one branded result with both the selected prototype
   payload and its exact representation tag.

It may not use `CurrentGlobal`, the legacy Error-family payload-only wrapper,
or the payload-only plain-object allocator. An explicit Object-, Function- or
Array-valued `NewTarget.prototype` wins and retains its representation tag.

## Construct-dispatch ownership

RegExp is a direct-returning constructor in the shared `[[Construct]]`
dispatcher. The constructor body owns active-function normalization, the sole
prototype Get, fallback, allocation, branding, and result. Direct dispatch must
therefore leave the generic construct block before that block reads
`NewTarget.prototype` or preallocates a receiver. Without this classification,
an observing Proxy new target sees two prototype Gets and the generic receiver
is allocated pointlessly before RegExp returns its own object.

## Storage and publication

The realm-intrinsics record already owns and publishes the RegExp prototype in
both producers:

- entry bootstrap publishes `REGEXP_PROTOTYPE_GLOBAL_INDEX`; and
- `$262.createRealm()` publishes its created `regexp_prototype_local`.

A resolved realm with no populated slot is an internal bootstrap invariant
failure, not permission to substitute the entry global. No record-size,
offset, or bootstrap-publication change belongs in this seam.

## Observable regression

The durable fixture avoids dynamic Function construction. It uses a created
realm's constructable `Proxy` function, whose defining realm is already known,
as an explicit new target with primitive or observing `prototype` behavior.
It checks:

- ordinary entry call and construction;
- borrowed created-realm call and construction use the created RegExp
  prototype;
- a created-realm new target with a primitive prototype falls back to that
  realm's RegExp prototype;
- Object-, Function- and Array-valued custom prototypes preserve exact
  identity and representation; and
- an observing Proxy around a created-realm constructable performs exactly one
  `prototype` Get.

The pinned `built-ins/RegExp/proto-from-ctor-realm.js` case constructs its new
target with `new other.Function()`. Dynamic Function source generation remains
an explicit Wasm-AOT exclusion, so that case is not load-bearing acceptance for
this seam. The source-free CLI fixture isolates the reachable RegExp behavior;
the pinned case may be rerun informationally when its separate dependency is
available.

## Deferred gates

This batch performs static source and diff checks only while central
verification owns Cargo and Test262 resources. Later verification must include:

```sh
cargo fmt --all -- --check
cargo check -p lila-aot-wasm --lib
cargo test -p lila-aot-wasm regexp_constructor_realm_ --quiet
cargo test -p lila-cli --test cli run_wasm_backend_uses_active_regexp_constructor_and_new_target_realm --quiet
./target/debug/lila test262 run built-ins/RegExp/proto-from-ctor-realm --execution-backend wasm --timeout-ms 180000 --threads 1
```

The complete T06/T19 ladders and current-SHA low-RAM publication path remain
the final closure gates.

## Non-claims

This seam does not implement the `IsRegExp`/same-constructor call shortcut,
cloning or flags override semantics, broad runtime pattern compilation,
source/flags getter and coercion ordering, species or RegExp String Iterator
closure, realm-correct RegExp `SyntaxError`s, dynamic Function construction,
complete realm bootstrap/teardown, or full T06, T19, RegExp, or Test262 green.
