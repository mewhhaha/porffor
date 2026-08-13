# Canonical module-request identity

This contract separates the two domains previously conflated in one Rust map
key: phase-free ModuleRequest identity and phaseful request occurrences.

## Defects closed

`ModuleRequestsEqual` compares `[[Specifier]]` and `[[Attributes]]`; phase does
not participate. The previous graph keyed host resolutions by
`ModuleRequestIr`, whose derived equality also compared phase. Evaluation,
defer and source occurrences of the same request could therefore require three
host rows even though they name one module-map entry.

The attribute list had a second representation defect. ECMAScript treats its
key/value records independently of source order, while Rust equality for a raw
`Vec` is order-sensitive. Parser-produced requests were sorted, but
`ModuleRequestIr` had public fields and `ModuleGraphSources::resolutions` is a
public embedder boundary. A host could supply the same attributes in another
order and silently miss the retained parse's graph key.

Finally, public resolution rows were collected with `BTreeMap::insert`. Two
rows resolving one `(referrer, request)` identity to different units selected
the last row rather than rejecting a contradictory host result.

A phase-free `requested_modules` list would overcorrect the first defect. For
`import source artifact from "m"; import "n"; import "m"`, projecting to keys
first produces `m, n`; finding evaluation occurrences through that projection
then evaluates `m` before `n`, contrary to the phaseful source order. Host
identity may erase phase, but the specification list and its consumers may not.

## Closed domains

`ModuleRequestAttributesIr` is the only attribute-list type accepted by a
request key or a statically known dynamic-import request. Its constructor:

1. sorts keys by UTF-16 code-unit order, matching ECMAScript String ordering;
2. rejects duplicate keys; and
3. exposes only an immutable slice.

`ModuleRequestKeyIr { specifier, canonical attributes }` is the phase-free
identity used by `ModuleRequestsEqual`, host resolution and graph/module maps.
Its fields are private and its constructors are the only way to create it.

`ModuleRequestIr { key, phase }` is a full occurrence used by import/export
entries, dynamic-component registration and evaluation classification. Its
phase remains available for dispatch, but `ModuleRequestIr::key()` is the only
identity that crosses the resolver boundary.

`SourceTextModuleRecordIr` records both domains:

- `requested_modules` is the source-ordered phaseful `[[RequestedModules]]`
  list, deduplicated by `(key, phase)`; and
- `module_resolution_requests` is its first-seen phase-free projection for
  host discovery only.

This lets evaluation, defer and source occurrences share one resolution while
remaining distinguishable and source ordered wherever phase has semantics.

## Public boundary and migration

These public surfaces now carry `ModuleRequestKeyIr`:

- `ModuleGraphSources::resolutions`;
- `ModuleGraphIr::resolutions`; and
- `HostModuleLoader::resolve`.

Both `lila-ir` and the public `lila-engine` loader face re-export the key type.

An embedder that previously constructed a `ModuleRequestIr` resolution row or
implemented the loader trait with `&ModuleRequestIr` gets a type error and must
migrate to `ModuleRequestKeyIr`. Direct construction and later mutation of the
key are also compile errors; rustdoc `compile_fail` examples pin those
boundaries.

Resolution rows with the same `(referrer, key, target)` coalesce. Rows with the
same `(referrer, key)` and different targets remove the mapping and produce
`ModuleLinkErrorIr::InconsistentResolution`; row order cannot select a winner.

## Invariants

1. Phase variants of one specifier/attribute pair have one host identity.
2. Reordering a valid request's attributes cannot change that identity.
3. UTF-16 ordering is fixed once, not re-derived by graph, loader or dispatcher
   code.
4. A duplicate attribute key cannot inhabit the canonical list.
5. A request key cannot be mutated after it becomes a graph-map key.
6. A contradictory public resolution table is rejected, never last-write-wins.
7. Evaluation order walks the phaseful list; a phase-free projection cannot
   reorder dependencies.
8. Runtime dynamic-import lookup still retains phase in its full occurrence.

## Durable regressions

Record tests compare keys constructed from opposite input orders, including
U+10000 and U+E000 to distinguish UTF-16 order from Rust scalar-value order,
and reject duplicate keys. They also require evaluation, defer and source
occurrences to remain three entries in `requested_modules` while coalescing to
one `module_resolution_requests` key.

Graph and filesystem-loader regressions require one phase-free row to serve all
three phase variants. A public graph-row regression supplies attributes in the
opposite order from source. Another supplies two different targets for one key
and requires an inconsistent-resolution error with no retained winner.
Interleaved source/evaluation and defer/evaluation regressions require the
evaluation dependencies to remain in their phaseful source order.

## Nonclaims

Boa 0.21.1 parses attributed re-exports but discards their attributes:
`ExportDeclaration::ReExport` retains only a `ModuleSpecifier`, and the parser
calls `parse_ignored_import_attributes`. Lila therefore still records
attributed re-exports as attribute-free requests. Preserving them requires a
Boa AST/parser change and is explicitly not claimed by this seam.

This contract also does not add a module type to the default filesystem
loader, make dynamic target evaluation lazy, implement exact namespace exotic
internal methods, coordinate top-level-await jobs, or close T12/Test262. The
default loader continues to reject every attributed request until it implements
the requested module type.
