# RegExp constructor active function and realm prototype

## Scope

This contract spans the closed active-standard-builtin and ordinary default-
prototype domains, the RegExp constructor body, shared constructor allocation,
direct construct dispatch, structural guards, and focused native fixtures.
Existing entry- and created-realm
`%RegExp.prototype%` publication are verified inputs; this seam changes no heap
layout or realm-bootstrap protocol.

When `RegExp` is called without `new`, the
[`RegExp ( pattern, flags )`](https://tc39.es/ecma262/multipage/text-processing.html#sec-regexp-constructor)
algorithm first returns the pattern unchanged when `IsRegExp(pattern)` is true,
flags are undefined, and the pattern's constructor is the active function.
Otherwise it replaces an undefined `NewTarget` with that active function.
Construction then reaches
[`RegExpAlloc`](https://tc39.es/ecma262/multipage/text-processing.html#sec-regexpalloc),
which performs `OrdinaryCreateFromConstructor(NewTarget,
"%RegExp.prototype%")`. The shared
[`GetPrototypeFromConstructor`](https://tc39.es/ecma262/multipage/ordinary-and-exotic-objects-behaviours.html#sec-getprototypefromconstructor)
rule performs one observable `Get(NewTarget, "prototype")`; a primitive result
selects the `%RegExp.prototype%` intrinsic of `GetFunctionRealm(NewTarget)`, not
the entry realm's prototype.

## Closed active-function normalization

Entry and created-realm standard builtins share emitted bodies.
`ActiveStandardBuiltinFunction` selects the four identity-sensitive
constructors in the shared function allocator: Iterator, RegExp, AggregateError
and SuppressedError. Each allocated function stores itself in its environment
handle. Calls through aliases, bound functions and proxies retain that captured
callee identity even after a global binding changes. The typed active-function
operation consumes the stored handle; its entry-global fallback serves only
compiler-internal invocations without a captured handle. An exhaustive map
ties `RegExpConstructor` to the entry-realm RegExp global.

A shared normalization operation consumes that typed active identity only
when the RegExp body's `NewTarget` tag is `undefined`. It writes the selected
payload and the Function representation tag into the existing new-target
locals. Explicit new targets remain byte-for-byte untouched. Both entry and
created constructors are self-backed by the allocator; host publication does
not repeat the store. A borrowed `other.RegExp(...)` call therefore retains
that actual constructor as its active function.

The same active identity also governs the preceding `IsRegExp(pattern)` and
same-constructor early-return rule. Pattern source/flags access is skipped when
that rule returns the original object.

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
target with `new other.Function()`. Finite prepared Function sources, including
the empty body, now have an AOT path. Irreducibly dynamic sources retain the
separate [dynamic-source capability boundary](dynamic-source-capability.md).
The source-free CLI fixture still isolates prototype selection from source
discovery.

`aot_regexp_constructor_and_iterator` adds native controls for retained active
constructor identity after global replacement, constructor source/flags and
prototype access order, matcher snapshots across input recompilation, flag
errors in the defining realm, modifier grammar, and realm-local string iterator
prototypes. These controls belong to the
[2026-09-10 baseline repair batch](../latest-baseline-repairs.md).

## Deferred gates

This batch performs static source and diff checks only while central
verification owns Cargo and Test262 resources. Later verification must include:

```sh
cargo fmt --all -- --check
cargo check -p lila-aot-wasm --lib
cargo test -p lila-aot-wasm regexp_constructor_realm_ --quiet
cargo test -p lila-cli --test cli run_wasm_backend_uses_active_regexp_constructor_and_new_target_realm --quiet
./target/debug/lila test262 run built-ins/RegExp/proto-from-ctor-realm --execution-backend wasm-aot --timeout-ms 180000 --threads 1
```

The complete T06/T19 ladders and current-SHA low-RAM publication path remain
the final closure gates.

## Non-claims

The active-function and prototype contract does not establish broad runtime
pattern compilation, complete Unicode property coverage, all pattern-error
realm behavior, irreducibly dynamic Function construction, complete realm
bootstrap/teardown, or full T06, T19, RegExp, or Test262 green. The repair report
separately records implemented constructor and iterator semantics and remaining
matcher gaps.
