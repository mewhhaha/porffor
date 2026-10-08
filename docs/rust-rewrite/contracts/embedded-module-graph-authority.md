# Embedded module graph source authority

The dependency-sealed replay policy owns one immutable `EmbeddedModuleGraph`
shared by the AOT host and spec-exec. Its sole constructor validates the complete
entry, Source Text Module rows and exact host resolutions before returning an
`Arc`. The entry's goal, identity, source and metadata URL cannot be mutated
independently after admission. Backend parsed records, module instances and
Realm caches remain owned by their existing compiler/runtime paths.

Identities are normalized relative virtual paths containing lowercase ASCII
letters, digits, `-`, `_`, `.`, and `/`. Empty, absolute, backslash, empty-component,
`.`-component and `..`-component paths are rejected. Metadata URLs are separately
declared `lila://` URLs whose remainder satisfies the same virtual-path grammar.
They are observations, never resolution bases. Source strings may be empty and
are retained exactly; parsing and ECMAScript early errors belong to the actual
backend parser rather than this host source constructor.

A Module-goal entry is implicitly included in the canonical Module source rows.
An explicit dependency with that identity is rejected, including an otherwise
identical duplicate. `dependency_modules()` excludes that implicit entry for a
reversible wire projection. A Script entry remains outside the Module rows and
may import a distinct Module record at the same identity; the projection keeps
that Module dependency.

Each resolution row carries the closed `Script(identity)`, `Module(identity)` or
`Unlocated` referrer role. A Script referrer must identify the Script entry; a
Module referrer must identify a declared Module row. An unlocated context uses
only explicitly declared `Unlocated` rows. Every target must identify a Module,
including the implicit entry where applicable. Resolution does not infer a
referrer from the entry, metadata URL, working directory or dependency filename.

A request owns its exact JS specifier and attribute key/value strings. Attribute
keys sort by UTF-16 code-unit order, and duplicate keys reject construction.
Import phases remain separate ECMAScript semantics; the host row key is the
phase-free referrer, specifier and complete attribute set. Exact duplicate row
keys reject even when their targets agree. A query with a different referrer,
specifier, attribute presence or attribute value has no match. An absent row is
a denied request and must never regain a filesystem or network loader.

The graph computes one SHA-256 fingerprint after canonical source-row and
resolution-row sorting. The fixed `lila-embedded-module-graph-v1` domain frames
entry goal, referrer roles, collection counts and every UTF-8 field with fixed
one-byte discriminators and little-endian `u64` byte lengths. Entry and dependency
source bytes, every independently declared metadata URL, every request attribute,
and every target are included, whether or not execution visits them. Input row
or attribute order does not change the fingerprint; changing an unused declared
source or edge does. The AOT artifact identity and v4 replay identity must consume
this fingerprint from the same graph used by their loaders.

`ModuleLoadingPolicy::Embedded` adds this shared owner beside `Filesystem` and
`RejectAll`. The default remains `Filesystem`; the legacy policy meanings and
legacy replay schemas stay separate. Policy cloning retains the same `Arc`, so
root, created-Realm and agent adapters cannot silently restore an ambient loader.
The additive v4 wire decoder and both real backend adapters are required parts
of the coherent batch; this owner alone does not provide runnable graph replay.

Finite source controls cover exact referrer roles, no URL-derived lookup,
UTF-16 attribute canonicalization, input row reordering, immutable policy sharing,
full fingerprint changes and ambiguous/unowned row rejection. Compilation,
focused execution and whole-batch verification remain required before completion.
The already-pinned `sha2` dependency is added to lila-runtime; Cargo lockfile
resolution remains pending until the authorized compilation checkpoint.
