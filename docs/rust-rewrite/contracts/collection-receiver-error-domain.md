# Collection receiver error domain

Status: receiver-error domain and Map/Set iterator-next Realm metadata repair
implemented and focused verification green.

## Closed failure authority

`CollectionReceiverError` is the private two-state failure authority shared by
ordinary collection data receivers and strong collection iterator receivers:

- `NonObject` means the receiver's ECMAScript Type is not Object;
- `MissingInternalSlots` means it is an Object without the required collection
  or iterator slots.

The domain derives no cloning, copying, debugging, equality or default
capability and has no manual implementations. The data and iterator message
tables match both states exhaustively for every collection family, while
`CollectionReceiverRequirement` forwards the same typed failure into the
appropriate table. There is no wildcard, equality or Boolean fallback.

## Producer boundary

The shared representation-safe receiver validator is the complete producer
set. An ordinary branded Object whose brand does not match and an Object layout
without a brand both produce `MissingInternalSlots`. A non-Object produces
`NonObject`; the compile-time-only non-runtime representation remains an
internal trap. The successful branded path loads the record before the
brand-mismatch error branch, preserving the existing instruction order.

The dedicated structure guard recursively pins all nineteen production
mentions, the twelve exact message rows, typed forwarding, the three ordered
validator bodies and the adjacent attribute-free declaration. The existing
fixture sources contain both failure categories, all six collection families
and successful receiver controls. Only the data-receiver fixture is green
runtime evidence from the recorded receiver-domain checkpoint.

## Created-Realm iterator-next publication

The receiver helper creates a failure from the active builtin function's Realm,
but a created-Realm function can supply that Realm only when bootstrap records
the function metadata before publication. The shared T18 publication boundary
now performs both required stores for Array, String, Map and Set iterator
`next` functions:

1. the function's own payload is stored at `HEAP_FUNCTION_ENV_HANDLE_OFFSET`;
2. the defining Realm's `%TypeError.prototype%` is stored at
   `HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET`;
3. only then is the function published as the prototype's `next` property.

The four unit `CreatedRealmIteratorNextTarget` variants map exactly to the four
builtins. One private context, constructed once in bootstrap, maps those same
targets to the Array, String, Map and Set iterator prototypes and carries the
Realm-function and TypeError-prototype inputs. A non-`Copy`, `#[must_use]`
token exists only after both stores, owns the selected prototype and function
locals, and must be consumed to publish the literal `next` property. The
dedicated five-test target pins that lifecycle and the Function-tag local store;
`collection_receiver_structure.rs` retains its two receiver-domain tests.

The context confines the raw bootstrap trust boundary to one constructor call.
It cannot prove that the `RealmFunctionMaterializationContext` and five raw
prototype-local indices came from the same Realm. This publication repair is
independent of the `CollectionReceiverError` source-equivalent closure. It
changes created-Realm function metadata, not the two error categories or their
messages. The full shared boundary is specified in
[`created-realm-iterator-next-publication.md`](created-realm-iterator-next-publication.md).

## Verification and nonclaims

The receiver-error-domain structure target retains its recorded `4/4` result,
and the data-receiver fixture retains its recorded `1/1` result. Those results
support the unchanged category mapping. Independent review also confirmed the
capability and mention closure, all twelve message rows, typed forwarding,
exact validator throw and return bodies, and brand-check/load/failure order.

At the 2026-08-29 checkpoint, `collection_receiver_structure` passes `2/2`, the
strengthened iterator-next publication target passes `5/5` and the shared
created-Realm materialization inventory passes `1/1`. The existing
`run_wasm_backend_succeeds_for_collection_iterator_receiver_realm_fixture`
passes `1/1`: it first checks defining-Realm provenance, then both error
categories and successful receiver controls.

The earlier receiver-domain checkpoint passed `cargo fmt --all -- --check`,
`cargo xc`, `git diff --check`, the module boundary check and the task-plan
check. It predates the metadata repair and does not verify it. This focused
repair does not close weak reachability, collection algorithms, full Test262
trees or T21.
