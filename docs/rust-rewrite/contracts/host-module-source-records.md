# Host-provided module source records

`EmbeddedModuleInput::ModuleSource(EmbeddedModuleSourceValueInput)` declares an
opaque native host module with a canonical identity, metadata URL, and source-tag
string. `LoadedModuleKind::ModuleSource` offers the same capability to custom
host loaders. Filesystem bytes do not acquire this capability. Host source data
is never parsed as ECMAScript, and has no JavaScript body or exported bindings.

A validated embedded resolution retains its actual target kind. An untyped
request cannot silently choose between Source Text and native source records
at the same locator. Fingerprints bind exact tags and actual target kinds. Wire
projection carries the explicit `module_source` kind and one text tag payload;
it cannot combine a source tag with a binary payload.

Source-phase resolution already returns the typed `ModuleSource` binding and
actual module unit identity. Re-export and star ambiguity comparisons preserve
that pair. Execution now retains a separate `ModuleCellIr::ModuleSource` and
actual source descriptors, independently of either namespace cell. Source
requests load the record without an evaluation dependency. Native host records
have no JavaScript activation; their empty namespaces remain ordinary namespace
objects. Source Text, JSON, Bytes and Text records still reject source imports
because they have no module-source representation.

The AOT companion owns branded GC objects, source cells and their actual
ModuleRecord and defining Realm. Tags are full Unicode host data. Repeated
imports and re-export paths read one canonical cell; user prototypes and proxies
cannot mint the private brand. No parser, interpreter, VM or host oracle is
embedded in emitted Wasm.

The Test262 host catalog alone maps its implementation-defined `<module source>`
virtual specifier to a genuine native host record. This does not inspect test
names or source patterns. Other specifiers and unsupported attributes retain
normal host resolution behavior. The hidden Boa oracle currently rejects this
new opaque native host-record capability explicitly; it is not a fallback for
the Wasm compiler path.

Focused controls include all three original pinned source/re-export test bodies,
identity and ambiguity, actual host catalog/wire admission, dynamic source jobs,
GC, private branding and defining-Realm ownership. They are authored but unrun.
Source-only composition/formatting/guards do not establish native conformance.
