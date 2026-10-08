# Ordered collection cursor contract

Actual GC MapObject/SetObject retain append-only entry positions and a separate
live size. Their concrete iterator schemas retain the collection edge, next
index, closed iteration kind and terminal done state. Bucket lookup is separate
from this history and cannot compact or reorder positions.

An update keeps its existing Map position. An absent insertion appends.
Deletion/clear remove retained entries while preserving history length.
Reinsertion appends a new position. Each iterator step and callback walk reloads
the actual collection table and current history length; table growth copies all
positions. Therefore appends made before exhaustion remain visible, including
after clear, and appends made after terminal exhaustion remain invisible.

A yielded live entry advances the saved index before producing its result.
A tombstone advances the working index before another position is inspected.
Exhaustion permanently clears the collection edge and sets done. Entry pairs
and iterator-result objects use the executing builtin's intrinsic Realm.

The consumed GC schema is the layout authority. Raw brands, pointers, offsets,
AUX words and CollectionIteratorCursorState transport codecs have no native
collection consumer. MapIterationKind/SetIterationKind select closed output
shapes; they cannot select a foreign collection layout.

The native collection controls author meaningful mutation/GC/Realm cases,
including pinned cases such as MapIteratorPrototype/next/iteration-mutable.js,
SetIteratorPrototype/next/iteration-mutable.js and clear preserving list history.
Their source is unrun; no native runtime, pinned green status, weak reachability
or full T05/T21 acceptance follows from this draft.
