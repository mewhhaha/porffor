# Native ordered collections use one GC object model

Map/Set native dispatch consumes actual MapObject/SetObject, entry and iterator
schemas. A separate GC hash index supplies lookup; the ordered GC entry table
supplies insertion history. Bucket links encode entry offsets plus one, never
reference addresses. Completed insertion appends; deletion unlinks before it
drops both retained key/value roots. Table growth copies tombstones and rebuilds
bucket links. Clear drops all entries and links while retaining history length.
A cursor reads the current table and history after each callback or next call;
exhaustion clears its collection edge and is permanent.

String/BigInt hashes derive from primitive contents. Number hashing unifies
NaNs and signed zero consistently with SameValueZero. Object/Symbol keys receive
stable private IDs from one typed module counter, retained in GC fields without
observable property access. Counter exhaustion traps before reuse. The hash
index remains bounded by geometrically grown history capacity. The native
receiver gate accepts the exact concrete reference and rejects a Proxy.

Constructors observe newTarget/prototype before their adder, then acquire one
completed cached iterator record. Entry validation, ordered key/value Gets,
conversion and adder/callback failures close that owner with the original whole
Throw. IteratorStepValue failures propagate through their direct spec path.
GetOrInsertComputed validates its callback even for an existing key, passes
canonical positive zero, and probes the current collection after the callback.

FromEntries uses own data-property definition. GroupBy converts each key once,
retains original items in ordered fresh Arrays and creates a null-prototype
Object or intrinsic Map. Set-like admission observes size/ToNumber/integer
validation/has/keys once in order. Set algebra uses its required size branch,
private difference copy, live receiver membership and duplicate-key suppression.
Union/symmetric difference call keys and cache next before copying the receiver.
Only early other-iterator predicate exits close with a Normal completion.
Results use the executing builtin's Realm and actual intrinsic prototypes.

Nine finite paired strict/sloppy Engine controls are authored for keys, GC,
live cursors, callbacks, constructor Close, computed insertion, grouping,
Set-like branch mutation, predicate Close and borrowed Realms. They are unrun.
Only source review and isolated formatting can be reported for this draft.
The atomic T05 cutover, compilation, emitted Wasm and task/conformance acceptance
remain open. Complete all task source before verified 4096 MiB aggregate capped,
serial verification. No README count or full conformance claim changes here.

Normative algorithms: [TC39 keyed collections](https://tc39.es/ecma262/multipage/keyed-collections.html),
[FromEntries](https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-object.fromentries),
and [GroupBy](https://tc39.es/ecma262/multipage/abstract-operations.html#sec-groupby).
