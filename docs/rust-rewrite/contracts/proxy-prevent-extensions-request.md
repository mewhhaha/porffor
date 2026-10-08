# Proxy `[[PreventExtensions]]` request and completion contract

Status: request/completion ownership is retained. The 2026-10-04 Realm
follow-up is authored source; current compilation/runtime acceptance is pending.
The original bounded contract and evidence at commit `f77ec3c2a` remain scoped
to that historical source.

## Trusted operation Realm — 2026-10-04 dry source

The real outlined PreventExtensions helper receives the outer operation's
trusted Proxy execution Realm context in parameter6, installs it in the existing
current-environment local and forwards it through recursive delegation, handler
Get, trap dispatch and target IsExtensible. The existing closed source
projections admit the three traversal helpers; scalar signature and request
roles remain unchanged. Revocation/noncallable/extensible-target native errors
use that trusted Realm and retain active-handler abrupt routing. Arbitrary
trap/getter thrown values are preserved.

A false trap result skips target IsExtensible. Reflect.preventExtensions returns
false; Object.preventExtensions throws a TypeError selected from the actual
called builtin's defining Realm. Its primitive-return behavior is unchanged.
Successful Reflect returns true and successful Object returns its original
object. No mutable public constructor determines those native errors.

The two finite strict/sloppy Engine fixtures for prototype/extensibility helper
Realms cover both Object/Reflect consumer pairs in both borrowed Realm
directions. They assert recursive errors, original foreign marker identity,
prior assignment/finally effects, false-result policy and successful target
mutation/return identity. Mutable foreign globals are clobbered after saving
native identities. Controls are authored and unexecuted; current source has not
been compiled or run. Full T11 and semantic GC remain open.

## Historical evidence boundary

The vendored Test262 tree at the original checkpoint had content identity
`aa55200d1310384c5cf69ea95b2a2ecba457007b`. Its
`built-ins/Proxy/preventExtensions` leaf contains 12 physical files and 23
execution identities. This batch owns exactly one physical file and one Module
execution:

```text
built-ins/Proxy/preventExtensions/trap-is-undefined-target-is-proxy.js
```

That historical harness did not execute the source honestly. It recognized the
exact path in `rewrite_proxy_prevent_extensions_case` and replaced its
self-imported module namespace with an ordinary non-extensible object. The
recorded one-case success snapshot and the older path-counted leaf result are
therefore materialized evidence, not proof of the product operation. This
batch's original requirement was to remove the rewrite once the product path
accepted the original source. These older materialized counts are not current
acceptance evidence.

## Normative lifecycle

For `O.[[PreventExtensions]]()`:

1. The caller constructs one `ObjectPreventExtensionsRequest` containing a
   `PreventExtensionsTraversalTargetLocals` and a
   `PreventExtensionsResultLocal`. The request owns those roles and is consumed
   exactly once.
2. An ordinary target performs the representation-specific ordinary
   `[[PreventExtensions]]` operation and publishes its Boolean result through
   the request's result role.
3. A Proxy target is classified once and its target and handler are loaded
   through the existing typed, live `ProxySlotLocals` authority. A revoked
   handler throws before either slot becomes usable.
4. `GetMethod(handler, "preventExtensions")` uses the complete retained handler
   tag and the proxy-aware object `[[Get]]` operation. Abrupt completion from
   lookup is routed before nullish/callable classification.
5. If the trap is `undefined` or `null`, the target's own
   `[[PreventExtensions]]` operation is invoked. This fallback is unbounded:
   no fixed nesting depth may turn a Proxy target into an ordinary target.
6. Otherwise a non-callable trap throws a current-realm `TypeError`. A callable
   trap is called with the exact handler as `this` and the exact target as its
   sole argument.
7. The call first produces `PendingProxyPreventExtensionsTrapResultLocals`.
   Abrupt completion must be routed before the only transition to
   `NormalProxyPreventExtensionsTrapResultLocals`. Only the normal role may be
   consumed by `ToBoolean` or any target-invariant check.
8. A false trap result publishes `false`. A true trap result performs the
   target's full proxy-aware `[[IsExtensible]]` operation recursively. An
   extensible target throws a current-realm `TypeError`; a non-extensible target
   publishes `true`.
9. `Reflect.preventExtensions` returns the published Boolean. For an object
   argument, `Object.preventExtensions` consumes the same result and throws a
   `TypeError` when it is false; its existing primitive-return behavior remains
   separate.

The result is published only after every abrupt-capable observation required
for that branch. A trap throw must never be replaced by a later invariant
`TypeError`.

## Closed Rust seam

The product seam uses these exact names:

- `RuntimeHelperId::ObjectPreventExtensions` is the unconditional outlined
  helper catalog entry;
- `PreventExtensionsTraversalTargetLocals` is the tagged value currently being
  traversed;
- `PreventExtensionsResultLocal` is the Boolean result destination;
- `ObjectPreventExtensionsRequest` is private, capability-free and
  `#[must_use]`;
- `PendingProxyPreventExtensionsTrapResultLocals` and
  `NormalProxyPreventExtensionsTrapResultLocals` close the trap-completion
  transition;
- `object_prevent_extensions_helper_function_index` derives the helper index
  from `RuntimeHelperId`;
- `compile_object_prevent_extensions_helper` emits the shared helper body;
- `emit_call_object_prevent_extensions_helper` is the recursive outlined call
  boundary.

The request constructor is the only public-in-crate way to pair the traversal
and result roles. The main emitter consumes the request. Raw positional
`payload`, `tag`, and `result` integers are not its public API, so swapping or
omitting a role is a type error. The pending trap-result role is not accepted by
the normal consumer. The request, traversal target, result destination, pending
trap result and normal trap result implement no clone, copy, debug, default,
comparison, ordering or hashing capability. Their only usable surface is the
one-way construction and consumption lifecycle.

At the historical Batch Y checkpoint, the normal-completion transition,
normal-result consumer and recursive traversal bodies were byte-identical at
`08ec7efc44446238a2faa8a34163b212cad3de76427bc5d35dfb9c5429979616`,
`158d5fa2f9ce31871ac1e711310b1167a126671eaa5d095a2470b04261de8c38`
and `ffbac884ee4acaee1567677169776c5ad4417b9b182df24c9b3d4d356e4b5c5a`.
Batch Y strengthens the existing three-test source guard to reject derived and
manual incidental capabilities. At the 2026-08-28 Batch Y checkpoint, that
guard passes `3/3`, the exact existing Proxy CLI fixture passes `1/1`, and
`cargo xc` is green. This capability-only change did not rerun Test262 or alter
the older complete-leaf evidence below.

The outlined helper uses the standard four-result JavaScript helper ABI:
parameters 0 and 1 carry the traversal payload and tag; the first result slot
carries Boolean `0` or `1`; the remaining results carry the normal/abrupt
completion tuple. Recursive missing/nullish fallback calls that helper instead
of inlining to a bounded Rust emission depth. The helper's own builder disables
only its entry outlining so it can emit the body once while recursive runtime
edges still target the cataloged helper function.

## Ownership and deletion

Product ownership is limited to:

- `crates/lila-aot-wasm/src/runtime_helpers.rs`;
- `crates/lila-aot-wasm/src/emit.rs`;
- `crates/lila-aot-wasm/src/objects.rs`;
- `crates/lila-aot-wasm/src/builtins/object.rs`;
- `crates/lila-aot-wasm/src/builtins/reflect.rs` only if the typed call boundary
  requires a mechanical consumer update.

The product batch deletes the fixed-depth emitter and the unreachable duplicate
inside the Object builtin. The evidence lane separately deletes
`rewrite_proxy_prevent_extensions_case`, its materialization unit, both shortcut
inventory observations, and updates the focused status/structure witnesses.

## Nonclaims

This batch does not implement a general Proxy internal-method framework, real
module-namespace exotic property descriptors, or another Proxy method. It does
not remove the retained `defineProperty`, `getOwnPropertyDescriptor`, RegExp,
Error, or other Test262 rewrites. It makes no full Proxy-subtree or full Test262
claim. Dynamic source cases remain outside this batch.

## Verification ladder

After all producer and consumer lanes are assembled:

```sh
cargo fmt --all -- --check
git diff --check
./scripts/check-module-boundaries.sh
cargo check --workspace --all-targets

cargo test -p lila-aot-wasm proxy_prevent_extensions -- --nocapture
cargo test -p lila-cli run_wasm_backend_succeeds_for_object_prevent_extensions_proxy_fixture -- --nocapture

./target/debug/lila test262 run \
  built-ins/Proxy/preventExtensions/trap-is-undefined-target-is-proxy.js \
  --execution-backend wasm --timeout-ms 120000 --threads 1

./target/debug/lila test262 run built-ins/Proxy/preventExtensions \
  --execution-backend wasm --timeout-ms 120000 --threads 4

./target/debug/lila test262 run built-ins/Proxy/isExtensible \
  --execution-backend wasm --timeout-ms 120000 --threads 4
```

The exact original Module execution must report `1/1`, and the complete current
leaf must report `23/23`, with zero unsupported, crash, timeout or runtime-failure
outcomes and without the path rewrite. Broader workspace and current-pin matrix
verification remains the centralized final gate.
