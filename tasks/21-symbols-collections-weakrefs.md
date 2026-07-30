# T21 — Symbols, collections, weak collections and weak references

**Status:** In progress — symbols/collections implemented; weak reachability awaits real GC

**Parallel group:** Feature lane; split internally by Symbol, strong collections and weak reachability  
**Depends on:** T05, T06, T10; iterators use T15; cleanup jobs use T14  
**Blocks:** Collection and weak-reachability portions of T26

## Current repository state

Symbol, Map, Set, WeakMap, WeakSet, WeakRef and FinalizationRegistry have
runtime records and builtin implementations, including ordered collection
storage and registered weak/ephemeron edges. Because the collector is not
executable, weak targets cannot clear and finalization cleanup jobs cannot be
driven by reachability. Strong collection coverage has advanced, but the full
weak-semantics and complete-tree criteria remain blocked on T05/T14.

## Objective

Implement Symbol identity and registries, insertion-ordered Map/Set collections, weak collections, WeakRef and FinalizationRegistry on top of the real object/GC/job models. Do not simulate weak behavior with strong maps or deterministic test-only collection.

## Symbol semantics

Implement:

- unique Symbol creation with optional descriptions;
- the process/agent-wide `Symbol.for` registry and `Symbol.keyFor`;
- every well-known Symbol required by the pinned ECMAScript/ECMA-402 revisions;
- Symbol wrapper objects, branding, `description`, `toString`, `valueOf` and `@@toPrimitive`;
- Symbol property keys throughout objects, descriptors, own-key ordering, proxies and reflection;
- cross-realm identity rules for well-known and registry symbols;
- correct TypeErrors for implicit string/number conversion and explicit `String(symbol)` behavior.

Use stable internal symbol IDs independent of string payloads. User symbols with equal descriptions remain distinct.

## Map and Set

Implement spec-shaped internal slots and insertion order for:

- `Map` and `Set` construction from arbitrary iterables with iterator closing;
- `get`, `set`, `has`, `delete`, `clear`, `size`, `forEach`, entries/keys/values and `@@iterator`;
- SameValueZero key semantics, including NaN and canonicalized signed zero;
- mutation during iteration/forEach, deletion/reinsertion and iterator liveness;
- subclassing, custom new target, method extraction and cross-realm behavior;
- all current Set composition methods and Map/collection additions present in the pin;
- `Map.groupBy`, `Object.groupBy` or related grouping APIs in the correct owning task/module when present.

Choose data structures that preserve insertion order and avoid accidental quadratic behavior. Hashing must work for every value kind while respecting identity and GC movement strategy.

## WeakMap and WeakSet

- Store keys through GC-supported weak/ephemeron edges, not strong references.
- Implement constructor iterable handling and all methods/branding/descriptors.
- Support every key category allowed by the pinned spec, including non-registered Symbols if applicable to the pin.
- Ensure entries disappear when keys become unreachable and values do not keep keys alive through cycles.
- Do not expose enumeration, size or deterministic collection timing.

## WeakRef and FinalizationRegistry

Implement:

- WeakRef construction, target validation, `deref` and keep-during-job semantics;
- FinalizationRegistry registration, unregister tokens, cleanup callbacks and holdings restrictions;
- cleanup jobs queued through T14 after GC discovers unreachable targets;
- correct behavior across realms, exceptions in cleanup callbacks and registry lifetime;
- host `gc()` as a real collection request without promising that every eligible object is immediately finalized.

Test support may repeat collection/job checkpoints, but it must not special-case known Test262 objects or force an otherwise-observable cleanup ordering.

## Integration requirements

- Symbols must participate in object shapes and `Reflect.ownKeys` without string conversion.
- Collection iterators use T15's common iterator protocol.
- GC tracing must retain strong collection entries and process ephemerons to a fixed point.
- Pending cleanup jobs and holdings must be rooted correctly.
- Proxies and wrappers retain object identity as keys.

## Acceptance criteria

- Full pinned Symbol, Map, Set, WeakMap, WeakSet, WeakRef and FinalizationRegistry trees are green.
- Symbol key ordering and coercion behavior pass across Object/Reflect/Proxy APIs.
- Map/Set iteration passes all mutation and reinsertion cases.
- Weak collections do not keep otherwise unreachable keys alive in GC stress tests.
- WeakRef keep-alive-within-job and finalization scheduling tests pass without deterministic shortcuts.
- Cross-realm well-known/registry/user-symbol identities are correct.
- No weak API is implemented by a permanent strong-reference table.

## Required tests

```sh
cargo test -p porffor-aot-wasm symbol_ --quiet
cargo test -p porffor-aot-wasm collection_ --quiet
cargo test -p porffor-aot-wasm weak_ --quiet
cargo test -p porffor-cli wasm_collection --quiet
./target/debug/porf test262 run built-ins/Symbol --execution-backend wasm --timeout-ms 120000 --threads 4
./target/debug/porf test262 run built-ins/Map --execution-backend wasm --timeout-ms 120000 --threads 4
./target/debug/porf test262 run built-ins/Set --execution-backend wasm --timeout-ms 120000 --threads 4
```

Run weak-collection, WeakRef and FinalizationRegistry filters repeatedly with GC stress enabled, then rerun Object/Reflect/Proxy own-key tests.
