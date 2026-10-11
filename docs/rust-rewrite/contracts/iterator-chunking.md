# Iterator chunking contract

The source authority is the stage-three
[Iterator Chunking proposal](https://github.com/tc39/proposal-iterator-chunking/blob/0fbb09fbf5b8afa47bf2397d6a0f37e6302fc623/spec.emu),
commit `0fbb09fbf5b8afa47bf2397d6a0f37e6302fc623`. Both methods have length one;
the windows undersized mode is an optional second argument.

Receiver admission precedes strict integral Number and range validation. No
argument coercion occurs. Invalid arguments close the provisional iterator
before reading next, preserving the original TypeError/RangeError. Windows
accepts only undefined, "only-full" and "allow-partial". Valid admission then
gets next once without an eager callable check.

The native buffered helper owns its cached IteratorRecord, captured Realm,
closed mode, requested size, and independent started/executing/done flags.
An internal GC List retains whole StoredValues through ArgumentListNodes.
Storage grows only with consumed values; the largest valid requested size
does not allocate an eager buffer. Snapshot arrays are fresh and belong to
the helper's captured execution Realm.

Chunks clears its List after full yields and yields one final nonempty partial.
Windows steps the source before dropping a full List's first element and
appending the new value. At exhaustion a full List remains full, so partial
mode yields only when the complete input was shorter than the requested size.
It never emits suffixes after full windows. Values and input errors are not
replayed. Abrupt next/done/value operations complete the helper without closing
the underlying iterator; return closes once, preserving original completions.

Generic helper dispatch recognizes either public creator and materializes
shared buffered next/return entries in the captured Realm. An unstarted return
retains the invoking method's Realm. IR shapes and method targets record the
real publication and synchronous user-code effects. The optional oracle uses
the same admission, List order and completion lifecycle independently.
