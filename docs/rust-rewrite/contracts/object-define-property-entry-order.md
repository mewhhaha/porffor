# Object descriptor definition entry order

Object.defineProperty performs the native object-target check, ToPropertyKey,
ToPropertyDescriptor, and DefinePropertyOrThrow in that order. Target rejection
precedes key or descriptor hooks; each conversion runs once. Shared validation
retains partial field presence and whole getter/setter/value identities.

Object.defineProperties checks its target before boxing the properties bag.
It takes one own-key snapshot, obtains each current own descriptor, and for
enumerable keys performs Get followed by ToPropertyDescriptor. A private GC
list retains the successfully converted partial descriptor and original key.
Only after the entire collection succeeds does ordered target definition begin.
A late conversion Throw therefore leaves the target unchanged. An application
Throw retains earlier completed definitions, as the specification requires.
Object.create uses this same owner after allocating its validated object/null
prototype, including null-prototype creation.

The 2026-10-05 GC controls cover late conversion failure, retained converted
values, omitted fields on existing data/accessor properties, and null-prototype
creation. They are authored and unrun. Historical checkpoint results do not
verify this source. Run verification only after the full source pass under the
confirmed 4096 MiB process-tree cap.
