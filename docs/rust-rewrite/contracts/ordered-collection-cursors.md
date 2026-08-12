# Ordered collection cursor contract

## Scope

This contract owns the mutation-visible cursor shared by strong `Map` and
`Set` iterators. It does not describe weak reachability, collection key lookup,
or the iterator result's key/value shape.

The pinned Test262 evidence makes two distinct requirements observable:

- `MapIteratorPrototype/next/iteration-mutable.js` and
  `SetIteratorPrototype/next/iteration-mutable.js` require an insertion made
  before exhaustion to remain visible, while an insertion made after an
  iterator has returned `done: true` remains invisible to that iterator;
- `Map/prototype/clear/map-data-list-is-preserved.js`,
  `Map/prototype/delete/does-not-break-iterators.js`, and the Map/Set
  `forEach` reinsertion cases require deletion to preserve positions and
  reinsertion to create a new position at the end.

## Representation law

An ordered collection record owns an append-only history of entry positions.
Its `entries_len` is the history length, not the number of live entries. The
separate `live_count` is the observable `size`.

The history obeys these rules:

1. construction starts with an empty history;
2. updating an existing live Map key does not create a position;
3. inserting an absent Map key or Set value appends one position;
4. `delete` and `clear` replace live positions with tombstones and never
   shrink, compact, or reuse the history;
5. growing storage may move the backing allocation, but it does not change
   position numbers.

A cursor is a pair `(state, next_index)` where `state` is the closed domain
`Scanning | Exhausted`. While scanning, one `next()` call repeatedly:

1. reads the collection's current history length;
2. transitions permanently to `Exhausted` when `next_index` reaches that
   length and severs the cursor's collection pointer;
3. otherwise reloads the current backing pointer, selects `next_index`, and
   persists `next_index + 1` before inspecting the entry;
4. skips a tombstone or returns the live entry.

Reading the current history length makes pre-exhaustion appends visible.
Persisting the increment before inspecting a tombstone makes deletion unable
to rewind the cursor. Reloading the backing pointer makes growth safe. The
irreversible `Scanning -> Exhausted` transition makes post-exhaustion appends
invisible and releases the exhausted iterator's strong reachability edge.

## Rust invariant

`CollectionIteratorCursorState` is the only source of persisted state words,
and the emitter exhaustively handles every state. An invalid heap word traps
instead of inheriting scanning or exhausted behavior.

`StrongCollectionCursor` is the closed layout domain for `Map` and `Set`. The
shared cursor emitter consumes it to select the collection pointer, history,
entry and state offsets. Map and Set therefore cannot silently drift into
different mutation semantics through duplicated control flow.

This is stronger than a test-only assertion: adding a cursor state or a strong
collection cursor kind requires updating exhaustive Rust matches before the
backend builds.

## Non-claims

The type and shared emitter close the product representation seam. They do not
claim the full pinned Map/Set trees are green, do not establish cross-realm
coverage, and do not alter the T05/T14 blocker on weak reachability and
finalization.
