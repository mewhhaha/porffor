# T03 — Test262 harness integrity and host contract

**Status:** In progress — complete Wasm-AOT host ownership and the exact typed shortcut ledger are enforced; semantic cleanup remains

**Parallel group:** Bootstrap/foundation  
**Depends on:** T01 for the authoritative inventory  
**Blocks:** Trustworthy results for every feature lane

## Current repository state

The repository has a checked-in host-ABI contract, shortcut inventory, exact
per-entry ledger and CI audits. `./scripts/check-test262-host-abi.sh` passes in
the current working tree. `./scripts/audit-test262-shortcuts.sh --check` now
pins every production observation on its audited surface by a stable key and
SHA-256 selector-evidence fingerprint. Rewrite-dispatch entries are keyed by
the called rewrite function, so deleting one no longer renumbers every later
entry; observations inside a declaration retain a local occurrence ordinal. It
rejects new, missing, duplicated or drifted entries, invalid classifications
and non-concrete task IDs, then byte-compares the generated inventory.

The current scanner-visible ledger contains 181 observations: 32 legitimate
harness adaptations, 106 diagnostic instrumentation sites and 43 semantic
shortcuts. The removal-task summary assigns 35 entries to T03 and leaves T17 at
80. The T03 removal bucket contains 32 legitimate adaptations, two diagnostic
guards and one semantic shortcut. Every entry has a concrete owner, removal
task and closed reason code; none use `T26-unclassified`. This is an honest
cleanup map, not completion. The semantic materialization layer is still
large, so harness results cannot yet satisfy
this task's integrity acceptance criteria.

The final twelve T18 semantic observations are now gone, leaving T18 with zero
shortcut ownership. The five affected physical String sources preserve their
exact vendored bodies and produce ten sloppy/strict executions. The spec-exec
oracle passes `10/10`; Wasm-AOT passes `0/10` and reports ten typed
`Unsupported` outcomes—eight executions from four direct-`eval` sources that
require a caller-environment lowering seam, and two from one ordinary
`Function` source that requires a target-Realm environment seam. The exact
source, metadata, prelude and diagnostic invariant replaces the materializers;
six adjacent non-dynamic product controls pass all `12/12` sloppy/strict
Wasm-AOT executions.

Reduced assertion selection has now been removed in full. The exact combined
census is 17,540 physical sources and 33,715 executions, all of which now
materialize the full LocalMerged `assert.js`. `typed_array_literal_helper_plan`
still owns 319 physical sources and 622 executions, but its assertion domain is
only `Full` or `Omit`: 296/576 use the full helper and 23/46 explicitly omit
unused assertion code. The SameValue and CompareArray assertion modes, their
prelude constants and their source-shape predicates no longer exist. This
retirement includes `ArrayBuffer.isView`, the
typed-array defined-length and `%TypedArray%[@@species]` sources, TypedArray
sort/`of`, DataView constructor cohorts, ProxyCreate, `Error.isError`, staging
`flatMap` and every other non-literal-plan consumer.

The compact typed-array property helper now accepts only the `TypeError`
raised by a strict failed write during its writability probe; every other
setter failure propagates unchanged. Exact Wasm-AOT runs for the `copyWithin`,
`findLast` and `findLastIndex` `length.js`/`name.js` cases pass all `12/12`
sloppy/strict executions through the rebuilt CLI. Focused runtime regressions
pin the strict non-writable probe and Proxy-setter error identity. This is a
harness-integrity correction, not a compiler/runtime semantic change, and it
does not change the shortcut census.

The exact `%TypedArray%.prototype.at` helper matcher and source guard are now
gone. A 15-source/30-execution invariant pins unchanged bodies in both Script
modes and both prelude profiles: all 13 typed-array-helper consumers use the
complete vendored `testTypedArray.js`, three also use the complete configured
`propertyHelper.js`, and the two resizable-helper cases retain only T13's
separately owned static-subclass substitution. This deletion removes one T17
semantic shortcut and one T17 diagnostic guard without changing the four exact
resizable-buffer support admissions. The rebuilt post-delete leaf passes all
`30/30` sloppy/strict Wasm-AOT executions, and exact controls for the surviving
compact property, split dispatcher and shared T13 helper routes pass `6/6`,
with every non-success bucket at zero.

The exact `%TypedArray%.prototype.filter` and `map` source matchers and their
compact prelude consumers are now gone. A shared invariant scans all 84
physical sources and 168 sloppy/strict executions in each directory. It pins
the 18 retired matcher contracts, both prelude stores and exact materialized
bytes: filter has 81 complete `testTypedArray.js` consumers and three sources
without that include; map has 79 and five. Six metadata sources also use the
complete configured `propertyHelper.js`. Sixteen matcher paths move from the
intrinsic fragment to the complete helper, while the two controls were already
complete. This removes two T17 semantic shortcuts and two diagnostic guards
without changing the resizable-buffer admissions. The rebuilt release CLI
passes all 36 exact current-pin executions as of `2026-08-30` under suite pin
`aa55200d1310384c5cf69ea95b2a2ecba457007b`; the then-live
`slice/invoked-as-func.js` compact route also passed `2/2` before the separate
slice retirement, with every non-success bucket at zero.

The combined exact `%TypedArray%.prototype.every`/`some` source matcher and
fingerprint guard are now gone. A replacement invariant pins 12 physical
sources and 24 sloppy/strict executions in four closed three-source cohorts,
with exact bytes and provenance under both prelude profiles. Only the three
`every` cases that declare `testTypedArray.js` change, moving from the split
dispatcher to the complete 14,921-byte helper. Three other `every` cases remain
without that helper; three `some` consumers remain on the typed-array literal
plan's 12,362-byte split route, and three others remain without it. The six
`resizableArrayBufferUtils.js` consumers retain T13's static-subclass
substitution. This removes one T17 semantic shortcut and one diagnostic guard,
leaves the broad `every`/`some` resizable-buffer admissions unchanged, and
at that checkpoint narrowed the retired iterator/find contract to 41 paths. The rebuilt
release CLI passes all 24 exact executions as of `2026-08-31` under suite pin
`aa55200d1310384c5cf69ea95b2a2ecba457007b`; the surviving
then-surviving `find/callbackfn-resize.js` split route passed `2/2`, with every non-success
bucket at zero.

The `%TypedArray%.prototype.slice` family-prefix selector, exact eight-path
source matcher and fingerprint guard, and slice-specific compact property
selector are now gone. The replacement invariant scans all 91 physical sources
and 182 sloppy/strict executions in both prelude stores and permits only 87
complete 14,921-byte `testTypedArray.js` consumers or four sources without that
helper. Fourteen former compact and eight former intrinsic cases now use the
full helper; 65 cases were already full. The three metadata sources retain
complete `propertyHelper.js`, `not-a-constructor.js` retains complete
`isConstructor.js`, and the four `resizableArrayBufferUtils.js` consumers
retain T13's exact static-subclass substitution. This removes two T17 semantic
shortcuts and one diagnostic guard, leaves the broad slice resizable-buffer
admissions unchanged, and limits shared family-prefix compaction to `includes`,
`indexOf` and `lastIndexOf`. The rebuilt release CLI passes all 44 changed
executions as of `2026-08-31` under suite pin
`aa55200d1310384c5cf69ea95b2a2ecba457007b`; exact surviving-route controls
pass `6/6`, with every non-success bucket at zero. This is a focused
post-retirement replay, not a new complete `182/182` execution claim.

The final `includes`, `indexOf` and `lastIndexOf` prefix compaction is now gone.
The combined invariant covers 130 physical sources and 260 sloppy/strict
executions in both prelude stores: 117 use the complete helper and 13 omit it.
Fifteen former compact and twelve former intrinsic cases now use the full
helper. Removing the shared helper, source parser and selector deletes five
T17 semantic shortcuts. The rebuilt release CLI passes the 54 changed
executions plus `4/4` surviving-authority controls under suite pin
`aa55200d1310384c5cf69ea95b2a2ecba457007b`; this is not a complete `260/260`
replay or broad T17 closure.

The shadowed 41-path TypedArray iterator/find matcher layer is now gone. All 17
iterator and 24 find contracts were already exact members of the closed
319-case literal plan, so the deleted fallback did not change materialized
bytes. A replacement invariant pins 82 sloppy/strict executions and 164
materializations across both prelude stores: 18 physical sources use the split
full-vendored plan, 23 have no `testTypedArray.js`, 21 retain compare-array
provenance and T13's static resizable-helper rewrite, and local/vendored STA
provenance is exactly `28/82`. Deleting both matcher tables, their fingerprint
guards, the source-only intrinsic fallback and the obsolete split-eligibility
wrapper removes four semantic and two diagnostic observations. The rebuilt
release CLI passes six representative sources in both Script modes (`12/12`)
under suite pin `aa55200d1310384c5cf69ea95b2a2ecba457007b`, with every
non-success bucket at zero. This is not a complete `82/82` replay or broad T17
closure.

The split dispatcher no longer scans source text for ten tail-only bindings or
conditionally retains the unused 2,854-byte end of `testTypedArray.js`. The
closed literal-plan invariant proves all 218 FullVendored physical sources,
representing 420 executions, have zero references to those bindings and always
materialize the canonical 12,362-byte split with FNV-1a
`0x92c7_bac7_27f5_772d`; the split appears exactly once and the tail marker is
absent. Drifted cases and helpers still fall back to the full vendored prelude
at the exact contract boundary. Removing the dead source predicate and
full-tail branch deletes two T17 semantic observations without changing any
admitted materialization bytes. Four representative `some`, `find`, `entries`
and `copyWithin` sources pass all `7/7` applicable executions under suite pin
`aa55200d1310384c5cf69ea95b2a2ecba457007b`, with every non-success bucket at
zero. This is not a complete `420/420` product replay.

The twenty retired Iterator-helper metadata branches changed eight surviving
T15 selector-table fingerprints without changing the observation count. Each
table still contains other materializers, so that checkpoint remained at 389
observations, 241 semantic shortcuts and 36 T15-owned observations. The
replacement invariant pins the twenty physical sources in both Script modes
with exact LocalMerged and vendored helper provenance. It now passes `1/1`, and
an isolated raw Wasm-AOT run of those exact twenty sources passes all `40/40`
sloppy/strict executions with every non-success bucket at zero. That metadata
retirement checkpoint left T15 at 34 observations.

The seven-case `Iterator.prototype.forEach` dispatcher and path-selector body
are now deleted. The replacement invariant covers the one built-in and
six staging sources in both Script modes, rejects any self-contained rewrite,
and pins exact original bytes plus LocalMerged/vendored assertion, `sta.js`,
`compareArray.js` and active-realm-host provenance. This removes two T15
semantic observations. The earlier dated built-in and staging leaf results were
rewrite-backed; a raw 14-execution Wasm-AOT replay remains pending.

The standalone Rust scanner tokenizes production code rather than reading one
line at a time. It records multiline expressions, every selector on a shared
line, exact whole-case rewrite calls, source reducers and normalized selector
patterns from `match` and `matches!`. Unsupported selector match-arm grammar
fails closed instead of producing a partial inventory. Report-only `test_path`
grouping and exact `#[cfg(test)] mod tests` bodies remain outside the production
contract. Audit green means no drift on this explicit selector surface, not no
remaining shortcuts.

The DataView accessor-metadata lane deleted its complete nine-case physical
rewrite. All nine pinned `buffer`, `byteLength` and `byteOffset`
`length`/`name`/`prop-desc` sources pass unchanged with the full upstream
`propertyHelper.js` in raw sloppy and strict runs (`18/18`). Ordinary Wasm-AOT
materialization uses the complete embedded LocalMerged `propertyHelper.js`
section; a real-source unit pins its exact source, provenance, and the unchanged
test suffix. That deletion removed exactly four semantic observations.

The neighboring nine accessor wrong-receiver sources now also execute
unchanged, with all eighteen raw sloppy and strict runs passing against the
full upstream assertion helper. Their primitive, Object, Array, ArrayBuffer,
SharedArrayBuffer and TypedArray receivers use only the complete LocalMerged
assertion prelude in ordinary materialization; a real-source 3x3 invariant pins
empty include lists, the supported-feature boundary, exact source identity and
assert-only provenance. Removing the dispatcher, rewrite owner, now-dead
accessor mapper and three suffix predicates deletes five more T17 semantic
observations.

The five `ArrayBuffer.isView` typed-array-helper sources now execute unchanged
with the full vendored `testTypedArray.js` in sloppy and strict modes (`10/10`).
Their real-source invariant pins source identity, exact complete-helper
materialization and provenance. All five ordinary materializations now use the
full LocalMerged `assert.js`, including the four sameValue-only sources.
Removing the original four rewrites and
helper skips deleted eight semantic observations; deleting the callable-alias
expansion and its final path-specific skip deletes one more.

The pinned typed-array buffer `defined-length.js` constructor source now also
executes unchanged with the full upstream `testTypedArray.js` in sloppy and
strict modes (`2/2`). Ordinary materialization uses the complete vendored
typed-array helper after the full LocalMerged `assert.js`. A real-source
invariant pins the exact source, full-helper bytes and provenance, and removed
helper-skip boundary. Deleting its static constructor fan-out and
path-specific skip removes one more semantic observation.

The four DataView BigInt-get ToIndex sources now also execute unchanged with
the complete merged assertion prelude in sloppy and strict modes (`8/8`). Their
real-source invariant pins the empty include list, full LocalMerged assertion
source and provenance, and exact original-source suffix. Removing their sole
rewrite owner and dispatcher deletes five more semantic observations and leaves
93 assigned to T17; it does not claim the remaining binary-data
materializations are complete.

The eight numeric DataView `set-values-return-undefined` sources now execute
unchanged with the full upstream assertion and `byteConversionValues.js`
harnesses in sloppy and strict modes (`16/16`). Ordinary materialization uses
the complete LocalMerged assertion prelude and complete VendoredHarness
conversion table; a real-source invariant pins both origins, the exact vendored
table, and the unchanged source suffix while requiring the full assertion
helper. Removing their standalone rewrite and dispatcher deletes four more T17
semantic observations.

The four `%TypedArray%[@@species]` metadata sources now execute unchanged with
the full upstream assertion, property and typed-array helpers in sloppy and
strict modes (`8/8`). Ordinary materialization uses the complete vendored
`testTypedArray.js` for all four cases and the complete embedded LocalMerged
`propertyHelper.js` for `name.js` and `length.js`. All four use the full
LocalMerged `assert.js`, including the two sameValue-only cases. A real-source
invariant pins those exact sources, complete helpers and origins. Removing the
species-specific compact-helper authorization deletes one T17 semantic
observation. The following workspace semantic golden passes `2/2` in 704.11
seconds with 666 dumps, adds only the independent array key-selection witness,
removes none and preserves all 665 retained non-accounting summaries.

The fifteen ArrayBuffer `byteLength`, `detached`, `maxByteLength`, `resizable`
and `slice` metadata sources now execute unchanged with the complete upstream
assertion and property helpers in sloppy and strict modes (`30/30`). Ordinary
materialization uses the complete embedded LocalMerged assertion and property
sections, while a separate VendoredHarness route retains the complete upstream
`propertyHelper.js`. One real-source matrix pins all fifteen source bodies,
declared includes, exact helper bytes and both provenance routes. Removing the
ArrayBuffer metadata matcher and compact-property projection deletes two T17
semantic observations. The following workspace semantic golden passes `2/2`
in 702.89 seconds with 667 dumps, adds only the independent iterator-policy
witness, removes none and preserves 665 of 666 retained non-accounting
summaries; the sole retained structural change is the expanded Promise callback
witness.

The forty-two DataView method `length` and `name` metadata sources now execute
unchanged with the complete upstream assertion and property helpers in sloppy
and strict modes (`84/84`). Ordinary materialization uses the complete embedded
LocalMerged assertion and property sections, while a separate VendoredHarness
route retains the complete upstream `propertyHelper.js`. A real-source 21x2
matrix pins every source body, the exact helper bytes and provenance, and the
current pin's absent `setBigUint64` metadata pair. Removing the metadata rewrite
owner and dispatcher deletes two more T17 semantic observations; the shared
DataView method mapper remains for the range and resizable rewrites.

The TypedArray sort value matrix, `TypedArray.of` zero case and eleven borrowed
Array callback resize cases now use their 13 pinned Test262 source bodies. One
13x2 invariant pins the exact sloppy/strict execution modes, original bytes,
declared includes, supported-feature boundary and absence of either a
self-contained or known-static replacement. It also pins exact source and
provenance for two materialization routes. The LocalMerged route uses no
`sta.js` and uses the complete merged assertion for all thirteen sources,
including the two single-assertion cases. Both routes use the complete vendored
`testTypedArray.js` and optional `compareArray.js`; the vendored-only route also
uses complete `assert.js` then `sta.js`. Deleting the handwritten constructor
fan-outs, their dispatches and the typed-array-helper omission removes exactly 29 T17 semantic
observations. An isolated post-delete Wasm-AOT run of those exact 13 sources
passes all `26/26` sloppy/strict executions with every non-success bucket at
zero. At that checkpoint, the ledger had 360 entries, including 212 semantic
shortcuts. T17 owned 161 entries, split between 80 semantic shortcuts and 81
diagnostic guards.

The eight top-level DataView constructor surface sources, `constructor.js`,
`dataview.js`, `length.js`, `name.js`, `proto.js`, `prototype.js`,
`extensibility.js` and `is-a-constructor.js`, now have no handwritten match
arms. Before deletion, a direct raw Wasm-AOT preflight ran their exact pinned
bodies through complete vendored `sta.js`, `assert.js` and each declared
`propertyHelper.js` or `isConstructor.js`. All `16/16` sloppy/strict
executions passed with `backend_used: WasmAot` and every non-success bucket at
zero. That result proves source readiness, not post-delete production dispatch.
The replacement 8x2 invariant pins exact modes, source bytes, includes,
supported-feature and no-rewrite boundaries, plus exact LocalMerged and
vendored-only materialization bytes and origins. The three assertion-only
sources and the other five all use the complete embedded assertion; the latter
also retain their declared helper. The vendored-only
route uses complete `assert.js`, `sta.js` and the declared helper. The rebuilt
production dispatcher then passes the same exact `16/16` cohort with every
non-success bucket at zero. Removing the eight arms only changes the fingerprint
of the surviving DataView constructor match observation, so the ledger remained
at 360 entries and T17 remained at 161.

The forty-eight Array prototype method `prop-desc.js`, `length.js` and
`name.js` sources now execute unchanged with the complete upstream property
helper in sloppy and strict modes (`96/96`). The 16x3 real-source invariant pins
every source body, supported feature boundary, unchanged source suffix, exact
LocalMerged assertion/property helper bytes and provenance, and a separate
complete VendoredHarness property-helper route. Removing the metadata rewrite
owner and its three path predicates deletes four T16 semantic observations.
The following shared workspace semantic golden passes `2/2` in 696.00 seconds
with 668 dumps. It adds only the shape-accessor reference-selection witness,
removes none, and leaves 664 of 667 retained dumps equal after accounting
normalization. The only retained structural changes are the intended Array
reduce, Promise internal-callback Realm, and TypedArray constructor no-species
witnesses.

The pinned `Array.prototype.at` `coerced-index-resize.js` and
`typed-array-resizable-buffer.js` bodies now pass through ordinary
materialization in both Script modes. Before deletion, a direct stdin Wasm-AOT
preflight combined complete vendored `sta.js` and `assert.js`, the disk-pinned
`resizableArrayBufferUtils.js` after replacing only its dynamic subclass block
with the three static classes already owned by T13, and the exact test source.
All `4/4` direct executions passed. The unmodified helper reached the explicit
Function-constructor AOT-unsupported diagnostic in both modes, so this result
does not establish full-helper support or post-delete production dispatch and
does not retire T13's substitution.
The 2x2 invariant pins exact modes, original bytes, includes, supported-feature
and no-rewrite boundaries. It also pins the original source suffix and exact
materialization bytes and origins: LocalMerged uses the complete embedded
assertion plus the transformed vendored helper, while vendored-only uses
complete `assert.js`, `sta.js` and that same helper. Deleting the complete
Array `at` rewrite helper, its only dispatch and its two predicates removes
three T16 semantic observations. Broad resizable admission and neighboring
`TypedArray.prototype.at` cases remain intact. The rebuilt production
dispatcher passed the same exact `4/4` cohort with every non-success bucket at
zero while retaining T13's helper substitution. That checkpoint left 73 T16
entries. The pinned
`Array.prototype.includes/resizable-buffer-special-float-values.js` source then
passed a separate raw `4/4` preflight across both Script modes and both prelude
stores with exact source and prelude bytes. Every execution reported
`backend_used: WasmAot`; only T13's static-subclass helper substitution was
applied. The unmodified helper still reaches the explicit Function-constructor
AOT-unsupported boundary, so this is scoped pre-delete evidence rather than a
full-helper or post-delete production-dispatch result. Removing its terminal
materializer left both neighboring Array `includes` rewrites and their shared
dispatch intact. After deletion, the rebuilt production dispatcher passed the
exact source in both Script modes (`2/2`) with every failure and non-success
bucket at zero. That historical checkpoint had 356 entries, including 208
semantic shortcuts; T16 owned 72. Each of the two remaining Array `includes`
sources then passed an exact raw `4/4` matrix across both Script modes and both
prelude stores. Every execution reported `backend_used: WasmAot` after only
T13's static-subclass substitution. The unmodified helper still reaches the
explicit Function-constructor AOT-unsupported boundary. The expanded
five-source invariant pins the two retired Array `at` sources and all three
retired Array `includes` sources with exact source, mode, prelude and
provenance checks, including the sole helper substitution. Deleting the
complete remaining Array
`includes` rewrite authority removes three more semantic observations. That
historical checkpoint had 353 entries, including 205 semantic shortcuts; T16
owned 69 and T17 owned 161. After deletion, the rebuilt production dispatcher
passed the exact final two-source cohort in both Script modes (`4/4`) with
every failure and non-success bucket at zero.

The exact `built-ins/Array/prototype/map/resizable-buffer.js` source then passed
a pre-delete raw `4/4` matrix across both Script modes and both prelude stores
with exact source bytes and only T13's static-subclass helper substitution. The
unmodified helper still stops at the explicit Function-constructor
AOT-unsupported boundary, so this is neither full-helper support nor
post-delete production-dispatch evidence. The expanded six-source invariant
pins the map source, declared comparison and resizable helpers, and exact
LocalMerged and vendored-only bytes and origins in both modes. Deleting only
the map branch from the known-static `for-of` rewrite removes one T17 semantic
observation. The remaining TypedArray accessor authority and shared
resizable-directory substitutions stay intact. That checkpoint's ledger had
352 entries: 35 legitimate
harness adaptations, 113 diagnostic instrumentation sites and 204 semantic
shortcuts. T16 owns 69; T17 owns 160, split between 79 semantic shortcuts and
81 diagnostic guards. After deletion, the rebuilt production dispatcher passed
the exact map source in both Script modes (`2/2`) with every failure and
non-success bucket at zero.

The exact pinned Array iteration `resizable-buffer.js` sources for `find`,
`findIndex`, `findLast`, `findLastIndex`, `every`, `some` and `filter` then
passed a raw `28/28` matrix across both Script modes and both prelude stores.
The separate `find` proof supplied `4/4`; sibling proof lanes supplied the
remaining `24/24`. Every execution reported `backend_used: WasmAot`, retained
the pinned test body, and applied only T13's replacement of the dynamic
subclass block with three static classes. `filter` declares `compareArray.js`
and `resizableArrayBufferUtils.js`; the other six declare only the resizable
helper. The unmodified helper still reaches the explicit Function-constructor
AOT-unsupported boundary, so the raw matrix does not establish full-helper
support. The expanded
thirteen-source invariant pins exact modes, original bytes, includes,
supported-feature and no-rewrite boundaries, exact LocalMerged and
vendored-only bytes and origins, source suffixes, the sole helper replacement
and T13 contract membership. Deleting the complete handwritten rewrite, its
sole dispatch and seven path predicates removes eight T16 semantic
observations. Broad per-method resizable admission and the neighboring
mid-iteration, `toLocaleString` and search rewrite authorities remain. After
deletion, the rebuilt production dispatcher passed the exact seven-source
cohort in both Script modes (`14/14`) with every failure and non-success bucket
at zero. That historical checkpoint had 344 entries, including 196 semantic
shortcuts; T16 owned 61. The six pinned Array `reduce` and `reduceRight`
resizable-buffer sources then passed a raw `24/24` matrix across both Script
modes and both prelude stores. Every execution reported
`backend_used: WasmAot`, retained the exact source and declared
`compareArray.js`, and applied
only T13's static-subclass replacement in `resizableArrayBufferUtils.js`. A
representative execution with the unmodified helper stopped at the explicit
Function-constructor dynamic-code-generation boundary. The evidence is scoped
to the transformed helper and does not establish full-helper support. The
expanded nineteen-source invariant pins exact modes, source and prelude bytes,
origins, suffixes, no-rewrite boundaries and T13 contract membership. Deleting
the complete reduce rewrite, its sole dispatcher call, both one-caller source
builders and the obsolete synthetic rewrite test removes six T16 semantic
observations while preserving broad reduce admission and neighboring
resizable authorities. After deletion, the rebuilt production dispatcher passed
the exact six-source cohort in both Script modes (`12/12`) with every failure
and non-success bucket at zero. That historical checkpoint had 338 entries,
including 190 semantic shortcuts; T16 owned 55. The four pinned Array `indexOf`
and three pinned Array `lastIndexOf` resizable-buffer sources next passed an
exact raw `28/28` matrix across both Script modes and both prelude stores. Every
execution reported `backend_used: WasmAot`, preserved the exact source and
declared resizable helper, and applied only T13's static-subclass replacement.
The unmodified helper stopped at the explicit Function-constructor
dynamic-code-generation boundary. Review also found that the handwritten
`lastIndexOf` rewrite bypassed the feature gate because no broad Array
`lastIndexOf/` resizable admission existed. The boundary now uses one closed
prefix set for `includes/`, `indexOf/` and `lastIndexOf/`, with an admission
witness for all three. The expanded twenty-six-source invariant pins exact
modes, source and prelude bytes, includes, origins, suffixes, no-rewrite
boundaries and T13 contract membership. Deleting both search rewrites, both
dispatcher calls, seven path predicates, both synthetic rewrite tests and both
dead shared source builders removes nine semantic observations. Consolidating
the two old Array-search diagnostic predicates removes one diagnostic
observation. Neighboring mid-iteration and `toLocaleString` authorities and the
TypedArray search admission remain. After deletion, the rebuilt production
dispatcher passed the exact seven-source cohort in both Script modes (`14/14`)
with every failure and non-success bucket at zero. That Array-search retirement
ledger had 328 entries: 35 legitimate harness adaptations, 112 diagnostic instrumentation
sites and 181 semantic shortcuts; this is the historical Array-search
retirement checkpoint, where T16 owned 45. The fourteen Array
`every`/`some`/`filter`/`find`/`findIndex`/`findLast`/`findLastIndex`
grow/shrink-mid-iteration sources next passed a raw `56/56` matrix across both
Script modes and both prelude stores, split into `24/24` quantifier and `32/32`
find-family executions. All used Wasm-AOT, retained byte-exact source and the
ordered comparison plus resizable-helper includes, and applied only T13's
static-subclass substitution. The unmodified helper stopped at the explicit
Function-constructor dynamic-code-generation boundary. The existing
pinned-source invariant now covers all fourteen files with exact modes, stores,
prelude origins and bytes, suffixes, no-rewrite checks and T13 contract
membership. Removing the complete shared rewrite, sole dispatcher call,
one-caller constructor list and obsolete synthetic test deletes its entrypoint
and fifteen direct selectors. The seven broad Array admissions, T13 helper
contract and neighboring Array values, iterator and `toLocaleString`
authorities remain. After deletion, the rebuilt production dispatcher passed
the exact fourteen-source cohort in both Script modes (`28/28`) with every
failure and non-success bucket at zero. That historical ledger had 312 entries
and 165 semantic shortcuts; T16 owned 29. The three pinned Array `values`
base/grow/shrink resizable-buffer sources then passed a raw `12/12` matrix
across both Script modes and both prelude stores. Every execution used
Wasm-AOT, preserved the exact source and ordered comparison plus resizable
helper includes, and applied only T13's static-subclass replacement. The helper
fingerprint `0x6466_6602_9ee8_9d5d` and case fingerprints
`0x5e5c_6ead_7b7c_0dda`, `0x3d18_7152_c6ff_a624` and
`0x60c2_a9ec_1dff_dd03` authorize it; any changed helper, path, include list or
source keeps `new Function` and reaches the explicit Function-constructor
dynamic-code-generation boundary. The exact pinned-source invariant covers all
three modes, stores, bytes, origins, suffixes, no-rewrite checks and T13
memberships. Deleting both complete rewrite functions, their two sole dispatch
calls and both synthetic replacement tests removes two entrypoints and three
direct predicates. Broad Array-values admission, Array keys/entries iterator
paths, the T13 contract and neighboring `toLocaleString` authority remain. The
checkpoint ledger had 307 entries: 35 legitimate harness adaptations, 112
diagnostic instrumentation sites and 160 semantic shortcuts. T16 owns 24;
T17 remains at 160, split between 79 semantic shortcuts and 81 diagnostic
guards. After deletion, the rebuilt production dispatcher passed the exact
three-source cohort in both Script modes (`6/6`) with every failure and
non-success bucket at zero.

The three pinned Array `toLocaleString` resizable-buffer sources then passed a
raw `12/12` matrix across both Script modes and both prelude stores. Every run
used Wasm-AOT, preserved the exact source, declared only
`resizableArrayBufferUtils.js`, and changed only T13's dynamic subclass block
to its three static classes. The helper fingerprint
`0x6466_6602_9ee8_9d5d` and case fingerprints `0x9da9_18f5_d04d_d764`,
`0xc380_4490_04ea_5b59` and `0x07d1_d14e_3a0b_bb89` authorize that replacement.
Changed helper, path, include or source bytes retain `new Function`; a
representative unmodified-helper run stopped at the explicit
Function-constructor dynamic-code-generation boundary. The expanded exact
invariant pins all three sources, modes, stores, bytes, origins, suffixes,
no-rewrite checks and T13 contract memberships. Deleting the complete Array
`toLocaleString` rewrite, its sole dispatcher call and its obsolete synthetic
test removes one entrypoint and three direct predicates. Broad Array
`toLocaleString` resizable admission and its feature-gate witness, T13's helper
contract, TypedArray `toLocaleString` behavior and neighboring DataView rewrite
authorities remain. The pre-retirement ledger contained 307 entries and 160
semantic shortcuts. The regenerated source ledger contains 303 entries: 35
legitimate harness adaptations, 112 diagnostic instrumentation sites and 156
semantic shortcuts. T16 owns 24; T17 remains at 160 and T18 owns 12. After
deletion, the rebuilt production dispatcher passed the exact three-source
cohort in both Script modes (`6/6`) with every failure and non-success bucket
at zero.

The seven pinned `%TypedArray%.prototype` accessor resizable-buffer sources
then passed an exact raw `28/28` matrix across both Script modes and both
prelude stores: `byteLength/resizable-buffer-assorted.js`,
`byteLength/resized-out-of-bounds-1.js`,
`byteLength/resized-out-of-bounds-2.js`,
`byteOffset/resized-out-of-bounds.js`,
`length/resizable-buffer-assorted.js`,
`length/resized-out-of-bounds-1.js` and
`length/resized-out-of-bounds-2.js`. Every execution used Wasm-AOT, preserved
the exact source, declared ordered `compareArray.js` and
`resizableArrayBufferUtils.js` includes, and retained the exact
`resizable-arraybuffer` feature with empty flags and no negative metadata. Only
T13's static-subclass replacement changed the helper. The unmodified helper
stopped at the explicit Function-constructor dynamic-code-generation boundary.
The renamed shared Array and TypedArray invariant pins the seven new sources
with exact modes, stores, source and prelude bytes, origins, suffixes and T13
contract membership. Deleting the complete known-static `for-of` wrapper and
TypedArray accessor rewrite, the wrapper's sole materialization call and the
obsolete identity assertions removes all 13 T17 semantic observations;
ordinary materialization now appends the original source directly. The three
broad TypedArray accessor admissions and T13's helper contract remain. The
historical pre-delete ledger contained 303 entries: 35 legitimate harness
adaptations, 112 diagnostic instrumentation sites and 156 semantic shortcuts.
The regenerated ledger contains 290 entries: 35 legitimate, 112 diagnostic and
143 semantic. T16 owns 24; T17 owns 147, split between 66 semantic shortcuts
and 81 diagnostic guards; T18 owns 12. After deletion, the rebuilt production
dispatcher passed the exact seven-source cohort in both Script modes (`14/14`)
with every failure and non-success bucket at zero. This does not claim broad
T17 closure.

The 43 pinned DataView method wrong-receiver sources now keep their original
bytes. The exact set contains `this-is-not-object.js` and
`this-has-no-dataview-internal.js` for the 21 mapped methods present at the
current pin, plus the sole
`getInt32/this-has-no-dataview-internal-sab.js`. Mapped `setBigUint64` has none
of those files, and no other mapped method has the SAB suffix. A pre-delete
direct raw probe covered `getInt8` primitive receivers, the `setFloat16`
wrong-slot case, the `getBigInt64` and `setBigInt64` metadata shapes, and the
`getInt32` SAB case across both Script modes and both prelude stores. All
`20/20` executions reported `backend_used: WasmAot`. This bounded proof did
not run every physical source. The replacement invariant scans all 22 mapped
methods against all three suffixes and pins the exact 43-source census,
contract fingerprints, metadata, mode order, admission, original bytes,
LocalMerged assert-only materialization and vendored `assert.js` then `sta.js`
materialization. Deleting the sole dispatcher call, complete rewrite and
obsolete synthetic test removes exactly six T17 semantic observations. The
verified pre-retirement ledger contained 290 entries, including 143 semantic
shortcuts. The regenerated ledger contains 284 entries: 35 legitimate, 112
diagnostic and 137 semantic. T16 owns 24; T17 owns 141, split between 60
semantic shortcuts and 81 diagnostic guards; T18 owns 12. The shared method
mapper, range and resizable rewrites, method-metadata and accessor invariants,
and broad DataView SAB admission remain. After deletion, the rebuilt production
dispatcher passed all 43 exact sources in both Script modes (`86/86`) with
every failure and non-success bucket at zero. This does not claim broad T17
closure.

The 41 pinned DataView method range sources now keep their original bytes. The
exact cohort has `index-is-out-of-range.js` for all 11 getters and 10 setters,
plus `range-check-after-value-conversion.js` and
`index-check-before-value-conversion.js` for those same 10 setters. The current
pin has none of the three files for `setBigUint64` and no getter
conversion-order files. A pre-delete raw run passed every physical source with
LocalMerged sloppy materialization (`41/41`). The `setUint16` range-after,
`setBigInt64` index-before, `getBigUint64` out-of-range and `setFloat16`
out-of-range representatives also passed both Script modes and both prelude
stores (`16/16`). The first manually assembled conversion-order stream omitted
LocalMerged `sta-preamble.js` and failed because `Test262Error` was unbound.
Restoring the normal prelude made that source pass; no corrected compiler or
runtime cell failed. The replacement invariant pins the closed 41-source
census, absent files, fingerprints, metadata, modes, admission, original bytes
and no-rewrite boundary. LocalMerged materialization uses `assert.js` then
`sta-preamble.js` for the 20 conversion-order sources and only `assert.js` for
the 21 out-of-range sources; vendored-only materialization always uses complete
`assert.js` then `sta.js`. Deleting the sole dispatcher call, complete range
rewrite, `dataview_method_range_info`, `dataview_method_call` and obsolete
synthetic test removes exactly six T17 semantic observations. The verified
post-wrong-receiver baseline,
after its `86/86` production run, contained 284 entries and 137 semantic
shortcuts. The regenerated ledger contains 278 entries: 35 legitimate, 112
diagnostic and 131 semantic. T16 owns 24; T17 owns 135, split between 54
semantic shortcuts and 81 diagnostic guards; T18 owns 12. At that checkpoint
the shared method mapper, complete resizable rewrite and helpers, admissions
and neighboring invariants remained. After the following resizable deletion,
a rebuilt production run passed this exact range cohort (`82/82`) with every
failure and non-success bucket at zero.

The 22 pinned DataView method `resizable-buffer.js` sources now also keep their
original bytes. The exact cohort has one source for each of `getInt8`,
`getUint8`, `getInt16`, `getUint16`, `getInt32`, `getUint32`, `getFloat16`,
`getFloat32`, `getFloat64`, `getBigInt64`, `getBigUint64`, `setInt8`,
`setUint8`, `setInt16`, `setUint16`, `setInt32`, `setUint32`, `setFloat16`,
`setFloat32`, `setFloat64`, `setBigInt64` and `setBigUint64`. A pre-delete raw
run passed all sources through Wasm-AOT with LocalMerged and vendored-only
preludes in both Script modes (`88/88`). The replacement invariant pins all 22
source fingerprints and bytes, exact metadata, both modes, admission,
no-rewrite status and exact prelude order, provenance and bytes. LocalMerged
materialization uses `assert.js` then `sta-preamble.js`; vendored-only
materialization uses `assert.js` then `sta.js`. Deleting the sole dispatcher
call, complete resizable rewrite, its value-literal helpers, the now-dead
shared method mapper, all three mapper-only test assertions and the obsolete
synthetic test removes exactly five T17 semantic observations. The verified
post-range checkpoint contained 278 entries, including 131 semantic shortcuts,
and assigned 135 observations to T17. The regenerated ledger contains 273
entries: 35 legitimate, 112 diagnostic and 126 semantic. T16 owns 24; T17 owns
130, split between 49 semantic shortcuts and 81 diagnostic guards; T18 owns
12. Broad DataView resizable, SAB and immutable admissions, constructor and
accessor authorities, and neighboring source invariants remain. The same
rebuilt production run passed the exact resizable cohort (`44/44`), for
`126/126` combined DataView method executions. This does not claim broad T17
closure.

That verified method run and its 273-entry ledger, including 126 semantic
shortcuts, form the historical constructor pre-retirement baseline. The 43
pinned DataView constructor validation sources now keep their original bytes.
The exact cohort has ordinary and SAB sources for 19 filenames, plus the
ordinary `buffer-not-object-throws.js` source and four ordinary
resize-during-custom-prototype sources. Those five SAB counterparts are absent
at the current pin. A bounded pre-delete raw probe ran eight representative
sources through LocalMerged and vendored-only preludes in both Script modes,
then ran one LocalMerged sloppy execution for each of the other 16 filename
arms. All `48/48` executions reported `backend_used: WasmAot`; no compiler,
runtime or harness cell failed. The replacement invariant pins the
43-present/5-absent census, sorted source-contract fingerprints, exact
metadata, both mode executions, admission, no self-contained rewrite and
original bytes. Its LocalMerged groups are now 32 full-assertion sources, nine
full assertion plus `sta-preamble.js` and two full assertion plus
property-helper sources. Vendored-only materialization uses exact `assert.js`
then `sta.js` bytes, plus `propertyHelper.js` for the two extensibility sources.
Deleting the sole dispatcher call, complete constructor rewrite, its sole
filename selector and the obsolete synthetic test removes exactly seven T17
semantic observations. That T17 retirement checkpoint contained 248 entries:
35 legitimate, 112 diagnostic and 101 semantic. T16 owns 24; T17 owns 105, split
between 24 semantic shortcuts and 81 diagnostic guards; T18 owns 12. Broad
DataView SAB and resizable admissions, the existing eight-source
constructor-surface invariant, method and accessor replacement invariants,
metadata authorities and unselected constructor neighbors remain. After
deletion, the rebuilt production dispatcher passed all 43 exact sources in both
Script modes (`86/86`) with every failure and non-success bucket at zero. This
does not claim broad T17 closure.

The pinned `toReversed/this-value-invalid.js` and
`toSorted/this-value-invalid.js` sources now execute without handwritten
replacements. A pre-delete raw probe passed both sources in sloppy and strict
LocalMerged modes (`4/4`), and six representative change-by-copy programs
passed with the complete upstream `testTypedArray.js`. The replacement
invariants pin both receiver contracts and the exact 21-source
`toReversed`/`toSorted` helper cohort across 42 Script executions, both prelude
stores, unchanged source suffixes and the intact 14,921-byte upstream helper;
neither compact nor split dispatcher materialization is admitted. Vendored-only
coverage at that checkpoint was a materialization/provenance assertion, not an
execution claim. The typed host boundary described below now supplies the
missing materialization contract. Deleting both
receiver rewrite authorities and the two family-specific dispatcher-split
gates removes twelve T17 semantic observations from the 266-entry constructor
checkpoint. The rebuilt production CLI passes the complete `toReversed` and
`toSorted` directories (`18/18` and `24/24`, `42/42` combined) with every
failure and non-success bucket at zero. Shared split-helper machinery remains
for independently owned TypedArray families; this does not claim broad T17
closure.

The `with/` directory no longer selects that shared split-helper path either. A
bounded pre-delete raw probe passed four representative unchanged executions
(`4/4`). The replacement invariant pins all 22 physical sources and 44
sloppy/strict executions, exactly 21 full `testTypedArray.js` consumers, the
one no-helper neighbor, source contracts, metadata, both prelude stores and
unchanged source suffixes. Deleting the sole `with/` selector removes one T17
semantic observation. The rebuilt production CLI passes the complete directory
(`44/44`) with every failure and non-success bucket at zero. Split-helper
ownership remains for other independently tracked TypedArray families; this
does not claim broad T17 closure.

The family-prefix selectors for `toLocaleString`, `slice`, `filter` and `map`
are now all retired. Exact invariants cover `39/78`, `91/182`, `84/168` and
`84/168` physical/execution identities respectively and permit only complete
`testTypedArray.js` or an explicitly absent helper. Earlier complete-leaf
replays passed `78/78`, `182/182`, `168/168` and `168/168` before the slice
retirement. The first three prefix deletions plus a source-text guard removed
four T17 semantic observations; the later slice wave removes two more semantic
observations and one diagnostic guard. Its rebuilt CLI passes all 44 changed
executions plus `6/6` adjacent authority controls, rather than claiming a new
complete `182/182` sweep. The final `includes`, `indexOf` and `lastIndexOf`
prefix compaction is now gone too. A combined invariant scans all 130 physical
sources and 260 sloppy/strict executions in both prelude stores, permitting
exactly 117 complete helpers and 13 sources without `testTypedArray.js`.
Fifteen former compact and twelve former intrinsic cases now use the full
helper; the 13 no-helper sources remain distinct from the 11 T13
static-resizable-helper consumers. Deleting the shared 5,254-byte helper, its
source parser and the final selector removes five T17 semantic observations.
The rebuilt release CLI passes all 54 changed executions plus `4/4` surviving
literal-plan and iterator/find controls under suite pin
`aa55200d1310384c5cf69ea95b2a2ecba457007b`, with every non-success bucket at
zero. No family-prefix compaction remains; closed literal plans own the
remaining intrinsic and split dispatch. This does not
claim a complete `260/260` replay or broad T17 closure.

The four ProxyCreate target-shape sources now execute unchanged in sloppy and
strict modes (`8/8`) with the full upstream assertion and constructor-test
helpers. Ordinary materialization uses the LocalMerged assertion and
constructor-test preludes. All four now use the full LocalMerged `assert.js`,
including the two sameValue-only revoked-target cases. A real-source invariant
pins that exact boundary. Removing the source rewrite, dispatcher and four path
predicates deletes five T11 semantic observations.

The Proxy apply non-callable-trap Realm source now also executes unchanged in
sloppy and strict modes (`2/2`). Its real-source invariant pins the active
Wasm-AOT host followed by complete LocalMerged `assert.js` and `sta.js`, the
unchanged vendored source suffix, and the absence of the handwritten case
rewrite. Removing that exact-path branch and
the later null-handler branch leaves `arguments-realm.js` as the apply rewrite's
sole case; the construct rewrites remain explicit debt.

The final Proxy apply and construct rewrite entry points and their four direct
path predicates are now gone. Their unchanged pinned sources materialize with
the complete host and local assertion/sta preludes, then reach the compiler's
explicit T13 dynamic-source boundary. The pinned sources retain indirect eval
in both `arguments-realm.js` leaves and ordinary Function construction in both
new-target-Realm leaves.
This deletes the last six T11 shortcut observations without claiming four Proxy
passes; T11 now owns zero entries in the 181-entry inventory. One closed
four-path diagnostic observation, owned for removal by T13, prevents the
currently permissive backend from silently counting those programs as passes.

The complete 18-path `Proxy.revocable` rewrite has now been removed. A
real-source matrix rejects any self-contained replacement and pins each
vendored source, declared complete helper and helper provenance. Seventeen
ordinary physical cases therefore use their original bodies. The remaining
`tco-fn-realm.js` source also stays intact, including its raw
`other.evalScript` call and complete Realm prelude, but its invocation belongs
to T13's typed `RealmEvalScript` AOT unsupported boundary. Removing the four
scanner-visible rewrite selectors leaves six observations assigned to T11.

Test262 prelude loading now records private `PreludeHostOwnership`, whose
variants are `None`, `EmbeddedSpecExecSta`, and
`WasmAot(CompleteWasmAotHostPrelude)`. The complete Wasm-AOT variant can be
constructed only after the embedded host passes its marker contract.
`Test262HostRequirement` likewise admits only `None` or `Complete`, with closed
realm-activation and agent-worker requirements, so a partial Wasm-AOT host
cannot enter materialization.
`EmbeddedWasmAot` combines the host with the embedded named harness, while
`EmbeddedWasmAotHostOnly` combines it with complete vendored named helpers.

The complete-host proof is opaque across a real Rust child-module boundary.
`CompleteWasmAotHostPrelude` carries a private `EmbeddedHostValidation` value,
and only that child module's `load_embedded` constructor can create one after
checking the embedded markers and final newline. `load_preludes` loads the
selected named harness first, requires both `assert.js` and `sta.js`, and only
then stores the pending host ownership. Any later `PreludeStore::insert` for
either required name clears that ownership, so helper mutation cannot retain a
stale complete-host proof.

Every non-raw case resolves and deduplicates declared includes before either a
self-contained rewrite or host planning. A missing include fails with the case
execution id and include name, and resolved helper bodies participate in the
same `$262` requirement scan as the original source. The current-pin census is
797 physical sources and 1,547 executions. Ten exact self-contained rewrite
sources account for 20 executions; they resolve their declarations but replace
the source before host emission. The remaining 787 physical sources and 1,527
executions emit the Wasm-AOT host prelude. Host-requiring ordinary
materialization has one leading order: strict directive when required, host,
`assert.js`, then `sta.js`; declared helpers and the original source follow.
The same host/assertion/`sta.js` worker prelude is stored privately on
`MaterializedTest`, so the runner consumes the materialized agent plan instead
of inspecting source text again. The product-runner witness
`wasm_agents_run_pinned_test262_case_with_exact_host_order` materializes the
pinned `Atomics.notify` no-waiter case, pins the host timer before
`atomicsHelper.js`, calls `run_one_case`, and requires a passing result.

The Wasm `agent_call` transport no longer duplicates raw operation integers
between the AOT emitter and engine. `lila-runtime::AgentHostOperation` is the
single closed 13-operation wire domain: the emitter writes its explicitly
pinned `i64` values, the engine rejects an unknown word once at the import
boundary, and the semantic dispatch is an exhaustive Rust match. This closes
one host-ABI drift path; it does not establish that every `$262` operation or
agent case satisfies the acceptance criteria below.

Direct `test262 run` and `test262 shard` completion now cross one typed verdict
boundary. `NoEvidence`, a non-empty all-pass `Passed`, and a non-empty
`Failed` verdict are distinct states backed by `NonZeroUsize`; inconsistent
total, pass and failure counts do not produce a verdict. The CLI exhaustively
maps only `Passed` to process success, while retaining a failed run's snapshot
before returning a non-zero exit. This closes command-level false-green and
zero-selection paths; it does not prove that the harness semantics which
produced a verdict are correct.

The command name carried into that verdict boundary now has its own private
no-capability `Test262VerdictCommand::{Run, Shard}` authority. Exactly the
direct `run` and `shard` completion paths produce it, and one exhaustive match
retains their exact user-visible spellings before the existing typed verdict
match. A recursive structure regression pins all five source mentions, the two
producers, spelling table, messages and summary-before-verdict order. This is a
source-equivalent capability closure; it changes neither verdict construction
nor snapshot, exit or conformance behavior. The strengthened exact-adjacency
and capability guard passes `3/3` and independent dry review is clean. The
failed-run, empty-selection and unsupported run/shard behavioral witnesses are
green; the failed-shard witness verifies its verdict before failing only the
pre-existing snapshot-layout expectation (`3` entries rather than `2`).

Aggregate snapshot progress also crosses one validated boundary. Expected and
completed matrix-node counts are opaque, completion is computed from those
counts instead of stored as an independently writable boolean, and completed
nodes must be unique members of the expected matrix. A zero-node matrix is an
explicit no-evidence state rather than complete by `0 == 0`. The CLI still
reports the same progress fields, but consumers cannot construct a summary
whose completion claim contradicts its node counts. This closes a
false-complete reporting shape; it does not establish that an incomplete
matrix is complete or authorize publication from progress-only evidence.

The shared aggregate schema validator now consumes the private, capability-free
`SnapshotUse::{CurrentState, ReadOnlyEvidence}` authority through one
exhaustive projection. Current, publishable and node-detail consumers retain
the current producer-bound schema requirement; metadata-only progress alone
may inspect a supported legacy envelope before its existing shared envelope
checks. The recursive guard pins all thirteen production mentions, seven
producer sites, both forwarding boundaries and the validation order. This is
source-equivalent harness hardening recorded in
[`test262-snapshot-use.md`](../docs/rust-rewrite/contracts/test262-snapshot-use.md),
not a legacy upgrade or new conformance evidence. The structure target passes
`4/4`, and the exact legacy/current aggregate witness passes `1/1`.
Independent review found and closed two structural escape hatches; final
re-review, the shared workspace compile and all repository gates are green.

Test262 case execution now carries the private, capability-free
`WasmAotExecutionStack::{DedicatedWorker, PersistentTest262Worker}` authority.
The test-only direct runner and product persistent-worker runner are its exact
two producers; the shared runner borrows it through two exhaustive projections
while preserving the persistent-agent, generic-agent, persistent module/script
and ordinary module/script route order and arguments. The recursive guard pins
all eight production mentions, the product `panic::catch_unwind` forwarding
call and the complete ordered routing region. Focused evidence is
`execute_cases_runs_wasm_aot_cases_on_persistent_workers` and
`wasm_aot_enforces_async_done_output_after_jobs_drain`; the latter reaches the
test-only dedicated entry point, while exact agent-route ownership remains
structural rather than a full agent-conformance claim. The structure target
passes `4/4`, and both exact behavioral witnesses pass `1/1`. The
source-equivalent boundary is recorded in
[`test262-wasm-aot-execution-stack.md`](../docs/rust-rewrite/contracts/test262-wasm-aot-execution-stack.md)
and does not close T03's remaining harness-materialization debt. Independent
dry re-review is clean after the recursive route censuses were made
Rust-lexical. The following shared workspace compile, formatter,
module-boundary, task-plan and diff gates all pass.

The full assertion/property prelude uses one SameValue algorithm. It treats
`NaN` as equal to `NaN`, distinguishes `+0` from `-0`, and is consumed by
`assert.sameValue`, `assert.notSameValue`, array comparison, and
`verifyProperty` value checks. The prior `!==` property comparison could
misclassify a correct descriptor as a runtime bug, while the retired
equality-only preludes could hide a signed-zero defect. A source-level contract
test pins the full body and its consumers. The typed-array literal plan's only
other assertion mode is explicit `Omit` for contracts that prove no assertion
use. This is a harness-integrity correction, not product semantics.

Direct-run resume checkpoints now bind the exact selected execution set,
manifest hash, pins, execution backend, matrix-strategy version, and intended
canonical terminal kind and matrix path before any recorded completion can
enter a new run. Full, shard, and matrix-node selections therefore cannot
share completion or journal state, and a SpecExec checkpoint cannot skip
Wasm-AOT execution and be rewritten under a Wasm-AOT label. Completed ids and
nested failure, timeout, and slow records are validated against that selection.
This closes the checkpoint/shard identity boundary; the remaining semantic
materialization debt stays open work.

The terminal run kind and matrix path inside that checkpoint boundary now
cross one typed admission point. Snapshot deserialization reads raw fields into
a private wire shape and must parse a canonical full, shard, or non-empty
matrix identity before returning `CheckpointRunIdentity`. Full, shard, and
matrix producers use named factories; case execution, periodic writes, resume
identity, and direct matching carry the opaque value rather than independent
strings and paths. Invalid persisted pairs therefore fail at deserialization,
and later consumers cannot forget a repeated validation call because none is
required. The source-equivalent boundary is recorded in
[`test262-checkpoint-run-identity-admission.md`](../docs/rust-rewrite/contracts/test262-checkpoint-run-identity-admission.md).

Runtime attempt-journal strikes are non-zero by construction. The persisted
wire map still reads numeric counts and rejects duplicate keys, but its sole
decoder now converts each count into `CaseStrikes` before it enters
`AttemptJournalFile`. Charging and quarantine admission consume that typed
map directly, so a later interior path cannot reinterpret zero as absence or
forget the non-zero check. Transparent serialization preserves the numeric
JSON schema; a product-path owner witness pins the projection. This is
source-equivalent accounting hardening recorded in
[`test262-attempt-journal-strike-state.md`](../docs/rust-rewrite/contracts/test262-attempt-journal-strike-state.md),
not evidence that T03's remaining semantic materialization debt is complete.

## Objective

Make the Test262 runner an honest observer of compiler behavior rather than a second semantic implementation. Replace source-pattern simulations, test-path materializations and permissive host fallbacks with explicit host APIs and general compiler/runtime semantics.

Current areas to audit include the embedded local-harness assets owned by
`lila-test262`, source materialization, `RunOptions.test_path`, and Wasm
backend branches that recognize exact Test262 paths or source shapes.

## Work items

### 1. Inventory semantic shortcuts

Generate a checked-in report of every branch that depends on:

- an exact test path or directory;
- assertion text, source regexes or known helper source;
- a hard-coded expected value for a real Test262 case;
- replacing an upstream helper with reduced behavior;
- converting a timeout into success.

Classify each item as legitimate harness adaptation, temporary diagnostic instrumentation, or semantic cheat. Assign removal to the relevant task ID.

The checked-in ledger completes this classification baseline for the current
mechanical scan. Stable keys intentionally exclude line numbers; line numbers
exist only in the generated report for review. Any source edit that retains a
key but changes its matched expression changes the fingerprint and fails the
guard. Semantic entries remain open work in their removal tasks.

### 2. Define the `$262` host ABI

Specify typed host operations for at least:

- `global`, `getGlobal`, `createRealm`, realm `evalScript` and `destroy`;
- `detachArrayBuffer`;
- `gc`;
- `IsHTMLDDA` and `AbstractModuleSource` where required by the pin;
- agent start/broadcast/report/sleep/leaving/monotonic time;
- async completion and `$DONE` reporting.

The Wasm-AOT product runner owns this ABI. The spec-exec oracle runner may implement it differently for differential runs, but the JavaScript-visible behavior and failure reporting must match, and nothing about the ABI design may assume the interpreter is available on the product path.

### 3. Remove fake concurrency behavior

The local harness currently contains source-pattern handling and `new Function`-based agent simulation. Replace this with real host-managed agents, shared backing stores and waiter queues. Never parse agent source with regexes to infer its expected behavior.

### 4. Separate harness adaptation from product semantics

Prelude merging may adapt Test262's shell contract, but it must not implement missing Array, Atomics, Promise, Proxy or other ECMAScript semantics. Product builtins must be installed by the runtime/compiler path.

### 5. Add integrity checks

Add tests or a lint that reject new exact-path semantic branches outside a narrowly documented allowlist for discovery/snapshot routing. The allowlist must name the reason and removal task.

## Failure behavior

Missing host capability must produce a stable `HostHarness` or explicit `Unsupported` failure. It must not return an object that aliases the current realm, silently ignore buffer detachment, or synthesize an agent report that lets a test pass.

## Acceptance criteria

- The harness contains no source regexes that emulate agent programs or expected assertions.
- Every `$262` method has a documented backend contract and direct tests.
- Realms are distinct or the operation fails explicitly; no same-global fallback.
- Buffer detachment and GC hooks either perform the requested operation or fail visibly.
- The runner correctly handles async pass, async rejection, timeout and duplicate `$DONE`.
- The generated shortcut inventory has an owner and removal task for every remaining item.
- Running an intentionally unsupported host case cannot be counted as success.

## Required tests

```sh
cargo test -p lila-spec-exec --quiet
cargo test -p lila-test262 --quiet
cargo test -p lila-engine --quiet
cargo test -p lila-cli test262_ --quiet
./target/debug/lila test262 run harness --execution-backend wasm
# Oracle runner parity check (diagnostic only):
./target/debug/lila test262 run harness --execution-backend spec
```

Add focused fake fixtures for each host method, but validate representative real `harness`, `language/module-code`, `built-ins/Atomics` and cross-realm cases before completion.
