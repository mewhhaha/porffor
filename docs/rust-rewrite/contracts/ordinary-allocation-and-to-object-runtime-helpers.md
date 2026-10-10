# Ordinary allocation and explicit Realm ToObject helpers

Ordinary headers and primitive boxing now have one physical Wasm body each.
Bootstrap namespaces and prototypes, function headers, arrays and concrete
Temporal records consume the same ordinary allocation facade. Ordinary and
borrowed-builtin ToObject routes consume the same explicitly Realm-bound boxing
facade. These operations previously emitted their complete algorithms in every
caller, even after property access and property publication had been outlined.

| Helper | Operands | Result |
| --- | --- | --- |
| OrdinaryObjectAllocate | complete prototype Value, immutable-prototype word | nonnull OrdinaryObject |
| ValueToObject | nonnull selected RealmRecord, complete input Value | complete Completion |

`objects/allocation.rs` owns the original allocation facades, their private
closed `ObjectPrototypeMutability` policy, the registered helper compiler, and
the private physical allocation kernel. The ordinary facade selects Mutable;
the Object.prototype bootstrap facade selects Immutable. An absent prototype
becomes the original Null Value before the call. The kernel retains the
prototype in a StoredValue, allocates independent OrdinaryPropertyStorage and
PrivateElementTable records, and constructs the original extensible header
with its selected mutation policy and zero identity hash. The allocator does
not choose a Realm or invoke JavaScript.

The header retains an immutable edge to its property storage. That record owns
an ordered PropertyTable, logical insertion extent and auxiliary hash index.
Named-property append doubles capacity when full, starting at four slots, and
uses typed Wasm GC `array.copy` to retain the old prefix. Spare slots are null
and lie outside the logical extent. Own-key enumeration and rooted snapshots
read this extent. Deletion clears an entry without rewinding it, so re-adding a
String or Symbol appends in the proper insertion order. Descriptor replacement
keeps its original entry. The shared append owner also checks for an existing
key, so even direct intrinsic installation cannot publish duplicate entries.

The registered OrdinaryPropertyFind helper returns a nullable PropertyEntry.
Its open-addressed index has twice the ordered table's power-of-two capacity;
zero buckets terminate a probe and nonzero buckets store insertion positions
plus one. Deleted entries remain tombstones until reuse or rehash, so a deleted
collision head cannot hide later keys. Each entry retains its full hash and
immutable insertion position. Growth rebuilds the index from live entries;
lookups compare full hashes and then complete String/Symbol semantic equality.
The index never determines observable key order. Capacity arithmetic is checked
before allocation or doubling. Physical limits remain runtime resource failures.

Property and collection key hashing share the exact UTF-16 FNV loop, stable
Symbol identity field and final tag mix in `operations/property_key_hash.rs`.
No lossy UTF-8 conversion, Symbol description or GC reference address serves as
identity. Number/BigInt/object collection hashing retains its original rules.
Find, append and delete keep keys, entries and replacement arrays rooted; no
JavaScript callback occurs between an index lookup and its physical mutation.

Ordinary/exotic own-key collection relies on unique, disjoint physical key
owners: named entries; Array or Arguments indexed entries; virtual String or
TypedArray indices; and the Array's separate `length`. It appends each candidate
once without comparing it against all previous keys. Numeric indices are
gathered into a rooted ArrayIndexKeyConstruction, sorted by the shared unsigned
heap sort, and reified as canonical decimal Strings. Other Strings and Symbols
retain insertion order in two following passes. Empty lists and sparse high
indices use candidate-count storage, never logical array length. The
[indexed-storage contract](gc-array-indexed-storage.md) supplies the indexed
uniqueness invariant. Module namespace order and Proxy trap duplicate/invariant
validation use their existing separate paths.

The facade consumes `ReferenceHelperResult` through the shared
`RuntimeSchema::helper_reference_on_stack` converter. This returns the existing
typed GC stack reference and preserves all allocation caller signatures. It
cannot manufacture a reference from a raw index or scalar. The helper compiler
enters its private kernel directly, emits the nonnull result while its operands
are rooted, and releases its parameter roots afterward.

`operations/to_object.rs` owns the three original ToObject entry routes. They
continue to choose the active runtime Realm, the current callable's defining
Realm, or the resolved callee Realm before calling ValueToObject. Revoked-Proxy
callee Realm lookup keeps its original UseCurrentRealm route. The private
boxing kernel receives that exact Realm and never substitutes helper entry's
Realm or a FunctionContext. Existing objects pass through unchanged. Number,
String, Boolean, Symbol and BigInt each allocate their original PrimitiveBox
using the corresponding prototype from that Realm. String boxing publishes
the exact UTF16 length with nonwritable, nonenumerable, nonconfigurable flags.
Null and Undefined retain the TypeError prototype from that same selected
Realm.

The helper returns the whole Completion. Its caller keeps every existing
pending-result check, copy and Throw propagation; outlining does not create a
new caller exit or change the input receiver. The private physical kernel is
only entered by the registered compiler, while nested header allocation and
property publication use their own typed helpers. No helper calls its own
public facade.

The existing object-construction Wasm runtime control now also covers
independent property tables, mutable null headers, immutable Object.prototype,
all five borrowed-Realm primitive boxes, astral plus unpaired UTF16 length, and
foreign nullish ToObject errors. Source controls pin explicit Realm transport,
closed allocation policy, private kernel ownership and whole-result/root
publication order. The shared artifact control requires both helper bodies
once with nonself direct consumers; existing size ceilings remain unchanged.

This extension passes the combined workspace type check and focused ownership
controls. Actual Wasm validation passes; the retained fixture's separate 1 MiB
body-size gate still rejects three Temporal Duration consumers. Native runtime
controls and RSS measurements are deferred to the combined checkpoint.
No amount of native compiler memory reduction is claimed from source alone.
