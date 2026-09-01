# Created-Realm iterator-next publication

Status: implemented and focused verification complete on 2026-08-29.

## Publication invariant

Created-Realm bootstrap publishes the four ordinary iterator `next` builtins
through one private, closed target domain. Its four variants are unit variants:

- `CreatedRealmIteratorNextTarget::Array` selects `ArrayIteratorNext`; the
  context maps it to `%ArrayIteratorPrototype%`;
- `CreatedRealmIteratorNextTarget::String` selects `StringIteratorNext`; the
  context maps it to `%StringIteratorPrototype%`;
- `CreatedRealmIteratorNextTarget::Map` selects `MapIteratorNext`; the context
  maps it to `%MapIteratorPrototype%`;
- `CreatedRealmIteratorNextTarget::Set` selects `SetIteratorNext`; the context
  maps it to `%SetIteratorPrototype%`.

A single private-field
`CreatedRealmIteratorNextPublicationContext<'_>` couples the created Realm's
`RealmFunctionMaterializationContext`, `%TypeError.prototype%` local and exact
Array, String, Map and Set iterator-prototype locals. It derives the prototype
local through an exhaustive match on the unit target. Neither the context nor
the target implements `Clone` or `Copy`.

`emit_materialize_created_realm_iterator_next` accepts only the target, a
borrowed publication context and the Wasm function. It derives the builtin,
function-Realm input, error prototype and publication prototype from those two
typed inputs. Before it can return a non-`Copy`, `#[must_use]`
`CreatedRealmIteratorNext` token, it materializes the selected function and
records two pieces of defining-Realm metadata in order:

1. the function's own payload at `HEAP_FUNCTION_ENV_HANDLE_OFFSET`;
2. the created Realm's `%TypeError.prototype%` at
   `HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET`.

The environment handle makes iterator-result allocation use the active
function's Realm. The TypeError slot makes receiver failures use that same
Realm. The token carries the selected prototype local and function local.
`emit_publish_created_realm_iterator_next` consumes it, writes the Function tag
to the reserved tag local, defines the literal `next` property, then releases
both owned locals. The publisher accepts no separate prototype local.

## Bootstrap trust boundary

The context localizes the remaining raw-local trust boundary; it does not make
the underlying Wasm local indices self-authenticating. Rust cannot independently
prove that the `RealmFunctionMaterializationContext`, TypeError prototype local
and four iterator-prototype locals supplied by the large bootstrap function all
belong to one created Realm. The one constructor call names those six exact
bootstrap values, after their allocation, and every downstream materializer
borrows that same context. This prevents individual Array, String, Map or Set
calls from pairing a unit target with an arbitrary prototype or Realm input.

## Observable String behavior

The product fixture borrows another Realm's
`%StringIteratorPrototype%.next`. A successful call must return an iterator
result whose prototype is that Realm's `%Object.prototype%`. Calling the same
function with an ordinary object must throw the existing exact message,
`String Iterator next called on incompatible receiver`, and the error must be
an instance of the defining Realm's `%TypeError%`, not the entry Realm's.

The String iterator step, code-point advancement and incompatible-receiver
message do not change. This repair supplies the Realm metadata already read by
those paths.

## Evidence and nonclaims

The strengthened five-test
`created_realm_iterator_next_publication_structure.rs` target pins the private
move-only context and token, both exhaustive target mappings, one exact context
construction, four unit-target lifecycles, ordered metadata stores and the
Function-tag `LocalSet` immediately before publication. The strengthened target
passes `5/5`; the retained collection receiver target passes `2/2`; and the
created-Realm materialization inventory passes `1/1`. The registered
`wasm_string_iterator_receiver_realm.js` fixture, existing Map/Set receiver
fixture and Array iterator-policy control each pass their focused CLI test
`1/1`. `node --check` accepts the String fixture, `cargo fmt --all -- --check`
is clean and `cargo check -p lila-aot-wasm` is green with only the pre-existing
vendored parser warning.

`ArrayIteratorIdentity` is deliberately outside this authority. It publishes
`%ArrayIteratorPrototype%[@@iterator]`, not a `next` function, and retains its
existing separate materialization path. `ArrayIteratorNext` remains in the
closed four-kind map.

The Map/Set iterator-receiver fixture is a control for this
shared publication boundary, not a new T21 result. This contract does not
change Map/Set validation or cursor behavior. It also does not cover RegExp
String iterators, iterator helpers, full created-Realm intrinsic publication,
the complete pinned String iterator tree or T18 completion.
