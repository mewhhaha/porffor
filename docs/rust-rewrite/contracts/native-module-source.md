# Native ModuleSource ownership

The host may supply a typed module-source record with a stable resolved module
identity, a host source tag, and an import-meta URL. It supplies no JavaScript
body. The compiler does not recognize fixture names or source text to create
this kind of record. Ordinary source-text, JSON, bytes, and text modules do not
become module sources through source-phase syntax.

Each Realm's actual module registry allocates the native ModuleRecord without
an ECMAScript activation, marks its empty native body completed, and retains
that defining Realm. Source-only requests never instantiate or evaluate an
ECMAScript body. Evaluation or defer requests for this native record use its
already-completed empty module and separate empty namespace objects.

The record owns one strong BindingCell for its source value, separately from
its eager and deferred namespace cells. The first source-cell access creates
one real ModuleSourceObject GC record. It retains the actual ModuleRecord,
defining Realm, host-provided source tag, and an ordinary object header whose
prototype is that Realm's stable host %ModuleSource%.prototype. The host
prototype's initial prototype is the same Realm's intrinsic
%AbstractModuleSource%.prototype exposed by its Test262 host facade. Both
prototype identities belong to the Realm intrinsic table, and source-cell
creation reads that table without allocating another host prototype. Repeated
source imports and reexports read the same canonical cell and object; a
different Realm has a different module registry and object identity.

The intrinsic toStringTag getter tests the concrete ModuleSourceObject GC
brand. It returns the retained tag for a genuine source and undefined for
primitives, ordinary objects, forged prototypes, proxies, and the intrinsic
prototype itself. Prototype mutation cannot forge or remove the brand. The
getter performs no user property lookup or module evaluation.

The GC declarations keep all owner edges typed, strong, immutable references.
The runtime and program share the exact recursive core layout. This component
remains unpublished and unverified until the final composed batch passes its
metadata, native ownership, source-import, reexport, Realm, getter, and GC
regressions. It makes no Test262 acceptance claim on source inspection alone.

Runtime layout 9 reserves a distinct closed intrinsic slot for the stable host
prototype. The preceding layout 8 allocates only the old object-intrinsic table
and may use its next slot for a private Intl symbol. Actual Intl frames, GC
struct equality and compiler-owned pool boundaries cannot certify that table
contract. Both authenticated cache and build-package envelopes therefore reject
layout 8 before a checked runtime proof can admit it. The three raw native
emitters keep their checked constructors; no existing R8 result is relabelled.
