# Generic native Array mutations

The atomic GC draft's pop, push, shift and unshift share the actual whole-value
Object operations. MutationReceiver prepares one ToObject and one observable
LengthOfArrayLike snapshot before dependent property operations. Consuming its
owner releases the cached receiver and length. Closed removal and growth kinds
cannot select each other's private execution policy.

Pop performs Get, DeletePropertyOrThrow and final strict length Set, including
empty receivers. Shift first reads index zero, moves present or inherited values
forward using HasProperty/Get/strict Set, deletes hole destinations and the last
index, then sets length. Unshift moves backwards before inserting original
argument List entries in order. Push appends those same whole arguments.
Both growth methods check the mathematical safe length before property writes;
zero arguments still perform the final strict length Set.

All property operations retain whole values, proxy hooks, prototype lookup,
original Throw identity and defining Realm error authority. A failure stops the
remaining algorithm and preserves the mutations already made. Decimal index
keys use the sole native exact-integer formatter; the generic algorithms neither
allocate according to logical length nor bypass ordinary or Array descriptors.
The sparse Array property storage owner remains a coordinated source obligation.

Algorithms follow [ECMA-262 Array operations](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.pop).
Six finite paired Engine controls cover cached coercion, whole values, empty
length writes, proxy operation order, sparse inherited properties, backward
movement, partial failures, generic receivers and borrowed error Realms.
They are authored and unrun. Compilation, fixture parsing, Wasm validation and
execution remain pending until the full task source pass; later verification
requires a confirmed aggregate 4096 MiB cgroup cap, zero swap and one worker.
