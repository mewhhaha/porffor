# Object builtin policy domains

The GC draft keeps closed policies with exhaustive matches in their actual
private algorithm owners. Enumerable own properties selects Keys, Values or
Entries. Integrity testing selects Sealed or Frozen. Legacy prototype lookup
and definition select Getter or Setter. The Object parent has closed String vs
Symbol key filtering, Sealed vs Frozen integrity setting, and Object-function
vs legacy-accessor prototype setting. No string policy or arbitrary integer
selector reaches these algorithms.

The private native result-array owner allocates using the executing function's
Realm, publishes complete indexed descriptors before advancing its count, and
is consumed to set the compact result length. The same owner serves
Keys/Values/Entries and own names/symbols. Prototype/species getters and public
Array setters do not participate in this private publication.

Object-function prototype setting rejects an invalid prototype even when its
target is primitive. The legacy accessor checks RequireObjectCoercible first,
then ignores invalid prototypes and primitive receivers. Both use the shared
SetPrototypeOf owner for real object targets and throw on Normal false.

Historical policy/source-equivalence checks apply to the preceding source.
The obsolete spelling guard is retired. Current controls are authored but
unrun; final verification waits for all-task implementation and the confirmed
4096 MiB process-tree cap. No conformance totals have changed.
