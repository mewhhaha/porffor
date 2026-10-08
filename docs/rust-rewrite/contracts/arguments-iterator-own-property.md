# Arguments iterator own property

`CreateMappedArguments` and `CreateUnmappedArguments` create an ordinary own
`@@iterator` data property whose initial value is the defining realm's original
`%Array.prototype.values%`. It is writable and configurable, and it is not
enumerable. The mapped parameter cells and strict `callee` accessor retain their
existing separate storage and semantics.

The AOT backend stores `@@iterator` in the existing Arguments named-property
table. Reads, assignment, `defineProperty`, deletion, own descriptor projection,
and own-key enumeration therefore consult the same property state. Deletion
persists; subsequent lookup follows the actual prototype chain. Recreating the
property with assignment gives the ordinary writable, enumerable, configurable
data attributes. A string key `"Symbol.iterator"` remains distinct from the
well-known symbol.

Each realm's intrinsic record retains the actual function object shared
by its initial `Array.prototype.values` and `Array.prototype[Symbol.iterator]`
properties. The entry bootstrap captures that object immediately after installing
the alias pair, before source execution. Created-realm bootstrap stores its own
materialized function object in the same typed slot. Arguments creation loads
that slot through the active callable's defining realm; it does not allocate a
replacement function or read a mutable prototype property. Missing bootstrap
state is an internal invariant failure, not a compatibility path.

`Array.prototype.values` is a foundational compiler root because every ordinary
callable can create an Arguments object, including source functions that contain
no explicit Array or Symbol reference. The additional intrinsic pointer is part
of the existing realm-record heap layout with `pointer: true`. Its allocation
uses the enlarged record extent and the existing realm-record storage/copy path.
The T05 collector-phase metadata remains passive; this change does not claim
executable collection or alter its capability boundary.

The original pinned mapped and unmapped `Symbol.iterator.js` fixtures are kept
unchanged. The dedicated Engine target covers initial descriptors and symbol
enumeration, replacement and iteration, deletion and recreation, descriptor
changes and inherited accessors with an explicit receiver, poisoning both public
prototype properties before and after creation, created-realm function
identity, and iteration through an inherited real Symbol after deletion while a
throwing string-key alias is ignored. The shared consumer helper performs one
ordinary Symbol Get for both own and inherited properties. Each regression is requested in ordinary and strict Script contexts;
the ordinary Script also contains a genuinely mapped factory.

This source packet has not been compiled or executed. Root must verify the two
pinned noStrict witnesses, the new Engine target, the existing Arguments
iteration and property regressions, the heap-layout checks, and the joined batch.
The combined T09 status note is owned by the separate native-function lane.
