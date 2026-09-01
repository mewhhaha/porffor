# Array.fromAsync Promise Realm context

Status: implemented on 2026-08-26; focused verification is recorded below.

## Ownership

`Array.fromAsync` is an async function. Its returned Promise and internal await
throwaway capability use the canonical `%Promise%` of the executing method's
Realm. Constructor `C`, supplied as the method receiver, independently chooses
the result Array and cannot select either Promise capability.

Entry publication may execute with a zero environment and explicitly selects
the entry `%Promise%` global. Created-Realm publication is self-backed; it must
load the method's defining Realm, that Realm's intrinsic table and the canonical
Promise-constructor slot. The Promise job active at the call site is not an
ownership source.

## Typed boundary

`ArrayFromAsyncExecutionRealmContext` is private, non-`Copy` and must-use. Its
factory reserves the durable Promise-constructor, Realm, default Function
prototype and TypeError-prototype locals before its transient intrinsic-table
local. Missing nonentry Realm or catalog state traps rather than substituting
entry intrinsics.

The typed capability consumer borrows the context, supplies the Function tag
internally and is the only raw `emit_new_promise_capability` caller in
`array_from_async.rs`. One context is acquired before the outer capability,
borrowed by exactly three capability allocations and consumed by one explicit
release after both runtime branches have been emitted.

## Covered capabilities

The boundary covers:

1. the Promise returned by `Array.fromAsync`;
2. the array-like path's await throwaway capability; and
3. the iterable path's await throwaway capability.

Both branch helpers accept only the typed context. Raw Promise-constructor
payload and tag parameters no longer cross either helper boundary.

## Focused evidence

The bounded structure target pins context privacy and lifecycle, explicit entry
selection, strict nonentry catalog lookup, the exact one-factory/three-consumer/
one-release census, typed branch signatures and existing created-Realm method
publication.

The same context now also owns internal fulfilled/rejected callback
materialization. The callback closure, continuation-state transport and
observable Realm controls are specified by the focused
`array-from-async-internal-callback-realm-context.md` contract.

The finite CLI fixture proves the Promise and result-Array authorities in both
directions: a created method with the entry Array constructor returns a created
Promise containing an entry Array, while the entry method with the created
Array constructor returns an entry Promise containing a created Array. Its
invalid-mapper branch also observes the created method's TypeError prototype.
The structure target passes `5/5` and the finite CLI target passes `1/1` on
2026-08-26. The existing CLI tests whose names contain `array_from_async` also
pass `4/4` with the strengthened fixture included.

The following shared semantic golden passes `2/2` in 697.36 seconds and
contains 671 dumps. Relative to the preceding 669-dump checkpoint it adds only
this fixture and the independent Temporal plain-difference fixture, removes
none and leaves all 669 retained dumps equal after accounting normalization.

```sh
cargo test -p lila-aot-wasm --test array_from_async_promise_realm_context_structure --quiet
cargo test -p lila-cli --test cli run_wasm_backend_uses_the_array_from_async_method_realm_for_its_promise --quiet
env LILA_GOLDEN_OUT=$PWD/target/golden/post-array-from-async-realm-temporal-difference-typedarray-search-v1 cargo test -p lila-aot-wasm --test emit_golden
```

This boundary does not change Promise allocation in other async builtins, async
iterator object ownership or the complete T06/T14/Test262 acceptance matrices.
