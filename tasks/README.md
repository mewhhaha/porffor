# Lila Rust AOT + Test262 execution plan

This directory is the 30-task epic-level implementation backlog and
current-status record for the Rust rewrite. It is designed so multiple contributors can work
concurrently without turning the remaining large IR and Wasm backend modules
into permanent merge-conflict bottlenecks. Individual task status fields and
their dated current-state sections are authoritative only until the next
current-pin aggregate or material repository change; generated Test262
artifacts remain the authority for conformance counts.

The north star is the repository contract in `AGENTS.md`: Lila compiles
JavaScript directly to Wasm, does not ship an interpreter/VM inside the
artifact, and drives the pinned real Test262 suite to zero unowned failures.
Fake-suite results are smoke tests only. An `Unsupported` result is visible
debt, never a passing result, and must not be hidden in a skip list or status
denominator.

Backend policy: `wasm-aot` is the product. It is the execution path every task
targets, the only backend whose results may be published as Lila conformance,
and the only backend the T26 release gate accepts. `spec-exec` (the Boa-based
engine in `crates/lila-spec-exec`) is an internal differential-testing and
debug oracle only, used by T25 and quarantined by T27 — never the CLI default,
never a silent fallback, never part of an emitted artifact, and never a source
of published conformance numbers. Wherever a task mentions running spec-exec,
that run is oracle triage; the Wasm-AOT run is the requirement.

## Current status snapshot — 2026-08-31

| State | Tasks | Repository evidence |
|---|---|---|
| Complete | T00, T27-T29 | Repository contracts are enforced, the interpreter is quarantined from the product, the legacy JavaScript product is retired, and the Lila identity cutover is verified |
| In progress | T01-T12, T14-T25 | Substantial implementation exists; T23's deterministic Intl architecture is live, but each task retains unmet acceptance criteria described in its current-state section |
| Policy, typed accounting and no-source eval implemented; textual static subsets open | T13 | Generic runtime dynamic source stays explicit Wasm-AOT unsupported; no-argument and proven non-String `%eval%` execute without crossing that boundary, while String-capable eval, all Function-family constructors and realm `evalScript` retain closed compiler diagnostics and sound textual subsets remain open |
| Blocked final gate | T26 | The current pinned real Wasm-AOT aggregate is not green or fully republished |

The current working tree passes the task-plan, module-boundary, host-ABI,
interpreter-dependency and Test262 shortcut audits. The shortcut audit now pins
an exact 186-entry token-aware generated inventory: 32 legitimate harness
adaptations, 105 diagnostic instrumentation sites and 49 semantic shortcuts.
The removal-task summary assigns 35 entries to T03 and leaves T17 at 80. The
T03 removal bucket contains 32 legitimate adaptations, two diagnostic guards
and one semantic shortcut. Every entry has a closed classification, reason and
concrete owner/removal task; none uses the old aggregate `T26-unclassified`
owner. The scanner covers multiline expressions, same-line multiplicity, exact
rewrite calls, source contract guards and normalized `match`/`matches!`
selector tables. Audit green
therefore means “no selector drift,” not “no shortcuts.” Do not close a
semantic task from focused green leaves while that task's full-tree and
materialization-removal criteria remain unmet.

The final twelve T18 semantic observations are gone, leaving T18 with zero
shortcut ownership. Its five physical String cases retain their exact vendored
sources across ten sloppy/strict executions. The spec-exec oracle passes
`10/10`; Wasm-AOT passes `0/10` and classifies all ten as typed `Unsupported`:
four direct-`eval` sources require a caller-environment lowering seam, while the
ordinary-`Function` source requires a target-Realm environment seam. These are
visible T13 dynamic-source gaps, not skipped or passing product cases. Six
adjacent non-dynamic product controls pass all `12/12` sloppy/strict Wasm-AOT
executions.

Reduced assertion selection has been deleted in full. All 17,540 physical
sources and 33,715 executions that formerly selected a reduced body now use the
full LocalMerged `assert.js`. The typed-array literal contract has 319 physical
sources and 622 executions: 296/576 use the full helper and 23/46 explicitly
omit unused assertion code. The SameValue and CompareArray assertion modes,
their prelude constants and their source-shape predicates are gone. The
compact typed-array descriptor probe now accepts only the `TypeError` raised
by a strict write to non-writable `length` or `name`; every other setter
failure propagates unchanged. Exact Wasm-AOT runs for the `copyWithin`,
`findLast` and `findLastIndex` `length.js`/`name.js` cases pass all `12/12`
sloppy/strict executions, and a Proxy-setter regression pins non-`TypeError`
propagation. The exact `%TypedArray%.prototype.at` helper matcher and source
guard are also gone. A 15-source/30-execution invariant pins unchanged bodies
in both Script modes and both prelude profiles: all 13 typed-array-helper
consumers use the complete vendored `testTypedArray.js`, three also use the
complete configured `propertyHelper.js`, and the two resizable-helper cases
retain only T13's separately owned static-subclass substitution. The rebuilt
post-delete leaf passes all `30/30` sloppy/strict Wasm-AOT executions, and three
exact adjacent controls pass `6/6` with every non-success bucket at zero. The
retirement covers the formerly generic
`ArrayBuffer.isView`, typed-array defined-length, `%TypedArray%[@@species]`,
TypedArray sort/`of`, DataView constructor, ProxyCreate, `Error.isError` and
staging `flatMap` cohorts.

The exact `%TypedArray%.prototype.filter` and `map` source matchers and their
compact prelude consumers are now gone. A shared invariant scans all 84
physical sources and 168 sloppy/strict executions in each directory, pins the
18 retired matcher contracts in both prelude stores and permits only complete
`testTypedArray.js` or no typed-array helper. Filter has 81 complete consumers
and three sources without the include; map has 79 and five. Six metadata
sources also use the complete configured `propertyHelper.js`. Sixteen matcher
paths move from the intrinsic fragment to the complete helper, while the two
controls were already complete. This removes two T17 semantic shortcuts and
two diagnostic guards without changing the resizable-buffer admissions. The
rebuilt release CLI passes all 36 exact current-pin executions as of
`2026-08-30` under suite pin `aa55200d1310384c5cf69ea95b2a2ecba457007b`;
the then-live `slice/invoked-as-func.js` compact route also passed `2/2`
before the separate slice retirement, with every non-success bucket at zero.

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

The final TypedArray family-prefix compaction for `includes`, `indexOf` and
`lastIndexOf` is now gone. A combined exact invariant scans all 130 physical
sources and 260 sloppy/strict executions in both prelude stores and permits
only 117 complete `testTypedArray.js` consumers or 13 sources without that
helper. Fifteen former compact and twelve former intrinsic cases now use the
full helper. The 13 no-helper cases remain distinct from the 11 T13
static-resizable-helper consumers. Deleting the shared 5,254-byte helper, its
source parser and the three-prefix selector removes five T17 semantic
shortcuts while preserving broad resizable-buffer admission and the closed
literal/iterator/find authorities. The rebuilt release CLI passes all 54
changed executions as of `2026-08-31` under suite pin
`aa55200d1310384c5cf69ea95b2a2ecba457007b`; the literal-plan and iterator/find
controls pass `4/4`, with every non-success bucket at zero. This is a focused
post-retirement replay, not a new complete `260/260` execution claim.

The shadowed 41-path TypedArray iterator/find matcher layer is now gone. All 17
iterator and 24 find contracts were already exact members of the closed
319-case literal plan, so the deleted fallback did not change materialized
bytes. A replacement invariant pins 82 sloppy/strict executions and 164
materializations across both prelude stores: 18 physical sources use the split
full-vendored plan, 23 have no `testTypedArray.js`, 21 retain compare-array
provenance and T13's static resizable-helper rewrite, and local/vendored STA
provenance is exactly `28/82`. The deletion removes four semantic and two
diagnostic observations. The rebuilt release CLI passes six representative
sources in both Script modes (`12/12`) under suite pin
`aa55200d1310384c5cf69ea95b2a2ecba457007b`, with every non-success bucket at
zero. This is not a complete `82/82` replay or broad T17 closure.

The split dispatcher no longer scans source text for ten tail-only bindings or
conditionally retains the unused 2,854-byte end of `testTypedArray.js`. The
closed literal-plan invariant proves all 218 FullVendored physical sources,
representing 420 executions, have zero references to those bindings and always
materialize the canonical 12,362-byte split with FNV-1a
`0x92c7_bac7_27f5_772d`; the split appears exactly once and the tail marker is
absent. Drifted cases and helpers still fall back to the full vendored prelude.
Removing the dead source predicate and full-tail branch deletes two T17
semantic observations without changing admitted materialization bytes. Four
representative `some`, `find`, `entries` and `copyWithin` sources pass all
`7/7` applicable executions under suite pin
`aa55200d1310384c5cf69ea95b2a2ecba457007b`, with every non-success bucket at
zero. This is not a complete `420/420` product replay.

Fourteen AggregateError and SuppressedError core-property materializations are
now gone. Their 28 exact sloppy/strict executions preserve pinned sources and
full applicable helper provenance; six raw cohorts pass `36/36` including
eight adjacent prototype controls. T24 therefore owns five remaining
observations, all explicit dynamic-source substitutions rather than ordinary
Error semantics.

Twenty Iterator-helper metadata branches are now gone across `every`, `some`,
`find`, `reduce`, `map`, `filter`, `flatMap` and `take`. The pinned-source
matrix covers both Script modes and exact LocalMerged/vendored helper bytes and
provenance. The focused invariant passes `1/1`, and an isolated raw Wasm-AOT
run of those exact twenty sources passes all `40/40` sloppy/strict executions
with every non-success bucket at zero. The eight enclosing selector tables
retain other rewrites, so that checkpoint remained at 360 total entries, 212
semantic shortcuts and 36 T15-owned observations.

The complete seven-case `Iterator.prototype.forEach` Test262 materializer is
also gone. Its one built-in and six staging sources now retain exact pinned
bytes across both Script modes, with exact LocalMerged/vendored assertion,
`sta.js`, `compareArray.js` and active-realm-host provenance. Removing its
dispatcher and path-selector body drops the current inventory to 186 total
observations, 49 semantic shortcuts and 32 T15-owned observations. The earlier
dated `27/27` built-in and `12/12` staging leaf results were rewrite-backed; the
raw 14-execution replacement replay remains pending.

The earlier T17 cleanup checkpoint left 190 entries after deleting the exact
nine-case DataView accessor-metadata, nine-case accessor wrong-receiver,
four-case `ArrayBuffer.isView` typed-array-argument, its callable-alias case, and
four-case DataView BigInt-get ToIndex rewrites, plus the eight-case numeric
DataView setter conversion
rewrite, the typed-array buffer defined-length expansion and the four-case
`%TypedArray%[@@species]` compact-helper authorization and the fifteen-source
ArrayBuffer metadata compact-helper boundary and the forty-two-source DataView
method metadata rewrite. Their real-source
invariants pin unchanged sources and the complete ordinary helper boundary;
the accessor wrong-receiver matrix passes all
eighteen raw sloppy and strict executions, and the numeric setter matrix passes
all sixteen. The TypedArray sort value matrix, `TypedArray.of` zero case and
eleven borrowed Array callback resize cases now preserve all 13 pinned sources
in both Script modes and both prelude stores. An isolated post-delete Wasm-AOT
run of those exact sources passes all `26/26` sloppy/strict executions with
every non-success bucket at zero. Removing their constructor fan-outs, helper
omission and dispatch paths deletes 29 more semantic observations. That cleanup
checkpoint left T17 with 161 entries, 80 semantic and 81 diagnostic. A separate
direct raw preflight of the eight top-level DataView constructor surface sources passed
all `16/16` sloppy/strict executions through complete vendored assertion and
declared helpers. Each reported `backend_used: WasmAot`, and every non-success
bucket stayed at zero. This was unchanged-source evidence before arm removal,
not a post-delete production-dispatch run. The replacement 8x2 invariant pins
exact LocalMerged and vendored-only materialization. The rebuilt production
dispatcher then passes the same exact `16/16` cohort with every non-success
bucket at zero. Removing those eight arms changes only the surviving selector
fingerprint, so that checkpoint's counts remained unchanged. The remaining T17
materializations stay open.

The two borrowed `Array.prototype.at` resizable-buffer sources now preserve
their exact pinned bodies in sloppy and strict modes. A scoped direct raw
preflight passed all `4/4` executions after combining complete vendored
`sta.js`, `assert.js`, `resizableArrayBufferUtils.js` with only T13's
replacement of the dynamic subclass block with three static classes, and the
exact source. The full unmodified helper still hits the explicit
Function-constructor AOT-unsupported boundary. This is pre-delete source
evidence, not full-helper support or a post-delete production-dispatch run. The
replacement 2x2 invariant pins exact LocalMerged and vendored-only
materialization, including the original suffix and the sole helper replacement.
After deletion, the rebuilt production dispatcher passed that exact `4/4`
cohort with every non-success bucket at zero while retaining T13's helper
substitution.
Deleting the complete rewrite helper, its dispatch and its two path predicates
removes three T16 semantic observations, leaving 73 T16 entries at that
checkpoint. The exact
`Array.prototype.includes/resizable-buffer-special-float-values.js` source then
passed a separate raw `4/4` preflight across both Script modes and both prelude
stores. Every execution reported `backend_used: WasmAot`; the sole helper
change was T13's static-subclass substitution. The unmodified helper still
reaches the explicit Function-constructor AOT-unsupported boundary, so this is
not full-helper or post-delete production-dispatch evidence. Removing only its
terminal materializer preserved the two neighboring Array `includes` rewrites
and shared dispatcher. After deletion, the rebuilt production dispatcher
passed the exact source in both Script modes (`2/2`) with every failure and
non-success bucket at zero. That historical checkpoint had 356 entries,
including 208 semantic shortcuts; T16 owned 72. The two remaining Array
`includes` sources each pass a separate raw `4/4` preflight across both Script
modes and both prelude stores. Every execution reports `backend_used: WasmAot`
after only T13's static-subclass helper substitution. The unmodified helper
still reaches the explicit Function-constructor AOT-unsupported boundary. The
expanded five-source invariant pins exact source, mode, prelude and provenance
bytes and the sole helper substitution for the two retired Array `at` sources
and all three retired Array `includes` sources. Removing the final two-source
rewrite authority deletes three more semantic observations. That historical
checkpoint had 353 entries, including 205 semantic shortcuts; T16 owned 69 and
T17 owned 161. After deletion, the rebuilt production dispatcher passed
the exact final two-source cohort in both Script modes (`4/4`) with every
failure and non-success bucket at zero.

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
resizable-directory substitutions stay intact. That checkpoint's inventory had
352 entries: 35 legitimate
harness adaptations, 113 diagnostic instrumentation sites and 204 semantic
shortcuts. T16 owns 69; T17 owns 160, split between 79 semantic shortcuts and
81 diagnostic guards. After deletion, the rebuilt production dispatcher passed
the exact map source in both Script modes (`2/2`) with every failure and
non-success bucket at zero. The seven pinned Array iteration
`resizable-buffer.js` sources for `find`, `findIndex`, `findLast`,
`findLastIndex`, `every`, `some` and `filter` then passed an exact raw `28/28`
matrix across both Script modes and both prelude stores. A separate `find`
preflight supplied `4/4`; the sibling proof lanes supplied `24/24`. Every run
used Wasm-AOT and preserved the exact source. Only T13's replacement of the
dynamic subclass helper block with three static classes was applied. `filter`
declares the comparison and resizable helpers; the other six declare only the
resizable helper. The unmodified helper still reaches the explicit
Function-constructor AOT-unsupported boundary. The expanded thirteen-source
invariant pins exact modes, sources, includes, prelude bytes and origins,
original suffixes, no-rewrite boundaries and T13 contract membership. Deleting
the complete handwritten iteration rewrite, its sole dispatch and seven path
predicates removes eight T16 semantic observations without changing broad
per-method admission or the neighboring mid-iteration, `toLocaleString` and
search rewrite authorities. After deletion, the rebuilt production dispatcher
passed the exact seven-source cohort in both Script modes (`14/14`) with every
failure and non-success bucket at zero. That historical checkpoint had 344
entries, including 196 semantic shortcuts; T16 owned 61. The six pinned Array
`reduce` and `reduceRight` resizable-buffer sources then passed an exact raw
`24/24` matrix across both Script modes and both prelude stores. Every run used
Wasm-AOT, preserved the pinned source, retained the declared `compareArray.js`,
and applied only T13's static-subclass replacement in
`resizableArrayBufferUtils.js`. A representative unmodified-helper run stopped
at the explicit Function-constructor dynamic-code-generation boundary, so this
is scoped pre-delete evidence rather than full-helper support. The expanded
nineteen-source invariant pins exact modes, source and prelude bytes, origins,
suffixes, no-rewrite boundaries and T13 contract membership. Deleting the
complete reduce rewrite, its sole dispatcher call, both one-caller source
builders and the obsolete synthetic rewrite test removes six T16 semantic
observations without changing broad reduce admission or neighboring resizable
authorities. After deletion, the rebuilt production dispatcher passed the exact
six-source cohort in both Script modes (`12/12`) with every failure and
non-success bucket at zero. That historical checkpoint had 338 entries,
including 190 semantic shortcuts; T16 owned 55. The four pinned Array `indexOf`
and three pinned Array `lastIndexOf` resizable-buffer sources then passed an
exact raw `28/28` matrix across both Script modes and both prelude stores. Every
run used Wasm-AOT with the exact source and declared resizable helper; only
T13's static-subclass replacement changed. The unmodified helper stopped at the
explicit Function-constructor dynamic-code-generation boundary. Dry review
found that the handwritten `lastIndexOf` rewrite had hidden a missing broad
Array `lastIndexOf/` resizable admission. A single closed prefix set now admits
`includes/`, `indexOf/` and `lastIndexOf/`, and its admission witness covers all
three. The expanded twenty-six-source invariant pins exact modes, source and
prelude bytes, includes, origins, suffixes, no-rewrite boundaries and T13
contract membership. Deleting both complete search rewrites, their two
dispatcher calls, seven path predicates, two obsolete synthetic tests and the
two dead shared prelude/constructor builders removes nine T16 semantic
observations; consolidating the two previous search admissions removes one
diagnostic observation. Neighboring mid-iteration and `toLocaleString`
authorities and broad TypedArray search admission remain. After deletion, the
rebuilt production dispatcher passed the exact seven-source cohort in both
Script modes (`14/14`) with every failure and non-success bucket at zero. The
Array-search retirement checkpoint had 328 entries: 35 legitimate harness
adaptations, 112 diagnostic instrumentation sites and 181 semantic shortcuts;
T16 owned 45. The fourteen Array
`every`/`some`/`filter`/`find`/`findIndex`/`findLast`/`findLastIndex`
grow/shrink-mid-iteration sources then passed all `56/56` raw executions across
both Script modes and both prelude stores, split into `24/24` quantifier and
`32/32` find-family cases. Every run reported `backend_used: WasmAot`, kept the
exact source and ordered `compareArray.js` plus
`resizableArrayBufferUtils.js` includes, and used only T13's static-subclass
replacement. The unmodified helper stopped at the explicit
Function-constructor dynamic-code-generation boundary. The pinned-source
invariant now owns all fourteen paths with exact modes, stores, bytes, origins,
suffixes, no-rewrite boundaries and T13 membership. Deleting the complete
shared rewrite, sole dispatcher call, one-caller constructor list and obsolete
synthetic test removes its entrypoint and fifteen direct predicates while the
seven broad Array admissions, T13 helper contract and neighboring Array
values, iterator and `toLocaleString` authorities remain. After deletion, the
rebuilt production dispatcher passed the exact fourteen-source cohort in both
Script modes (`28/28`) with every failure and non-success bucket at zero. That
historical checkpoint had 312 entries and 165 semantic shortcuts; T16 owned
29. The three pinned Array `values` base/grow/shrink resizable-buffer sources
then passed all `12/12` raw Wasm-AOT executions across both Script modes and
both prelude stores with byte-exact sources and ordered `compareArray.js` plus
`resizableArrayBufferUtils.js` includes. Only T13's static-subclass replacement
was applied. Its helper fingerprint is `0x6466_6602_9ee8_9d5d`; the three case
fingerprints are `0x5e5c_6ead_7b7c_0dda`, `0x3d18_7152_c6ff_a624` and
`0x60c2_a9ec_1dff_dd03`. Changed helper, path, include or source bytes keep
`new Function` and reach the explicit Function-constructor dynamic-code-
generation boundary. The exact invariant now owns all three modes, stores,
bytes, origins, suffixes, no-rewrite checks and T13 memberships. Removing both
complete rewrite functions, their two sole dispatcher calls and both obsolete
synthetic tests deletes two entrypoints and three direct predicates. Broad
Array-values admission, Array keys/entries iterator paths, T13's helper
contract and `toLocaleString` remain. That checkpoint's inventory had 307
entries: 35 legitimate harness adaptations, 112 diagnostic instrumentation
sites and 160 semantic shortcuts. T16 owns 24; T17 remains at 160, split between 79
semantic shortcuts and 81 diagnostic guards. After deletion, the rebuilt
production dispatcher passed the exact three-source cohort in both Script modes
(`6/6`) with every failure and non-success bucket at zero.

The three pinned Array `toLocaleString` resizable-buffer sources then passed an
exact raw `12/12` matrix across both Script modes and both prelude stores. Every
execution used Wasm-AOT, preserved the pinned source, declared only
`resizableArrayBufferUtils.js`, and applied only T13's replacement of the
dynamic subclass block with three static classes. The helper fingerprint
`0x6466_6602_9ee8_9d5d` and case fingerprints `0x9da9_18f5_d04d_d764`,
`0xc380_4490_04ea_5b59` and `0x07d1_d14e_3a0b_bb89` admit that one change.
Changed helper, path, include or source bytes retain `new Function`; a
representative unmodified-helper run stopped at the explicit
Function-constructor dynamic-code-generation boundary. The expanded invariant
pins the three exact sources, modes, stores, bytes, origins, suffixes,
no-rewrite checks and T13 memberships. Deleting the complete Array
`toLocaleString` rewrite, its sole dispatch and obsolete synthetic test removes
one entrypoint and three direct predicates. Broad Array `toLocaleString`
resizable admission and its witness, T13's contract, TypedArray
`toLocaleString` behavior and neighboring DataView rewrites remain. The
pre-retirement baseline contained 307 entries and 160 semantic shortcuts. The
regenerated source ledger has 303 entries: 35 legitimate harness adaptations,
112 diagnostic instrumentation sites and 156 semantic shortcuts. T16 owns 24;
T17 remains at 160 and T18 owns 12. After deletion, the rebuilt production
dispatcher passed the exact three-source cohort in both Script modes (`6/6`)
with every failure and non-success bucket at zero.

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
prefix compaction is now gone too. A combined invariant covers 130 physical
sources and 260 sloppy/strict executions in both prelude stores: 117 use the
complete helper and 13 omit it. Fifteen former compact and twelve former
intrinsic cases now use the full helper. Removing the shared helper, source
parser and selector deletes five T17 semantic shortcuts. The rebuilt release
CLI passes the 54 changed executions plus `4/4` surviving-authority controls
under suite pin `aa55200d1310384c5cf69ea95b2a2ecba457007b`; this is not a
complete `260/260` replay or broad T17 closure.

Test262 prelude loading now records private `None`, `EmbeddedSpecExecSta`, or
opaque complete Wasm-AOT host ownership. `EmbeddedWasmAotHostOnly` combines the
Wasm-AOT host with complete vendored named helpers. The embedded-host witness
can be constructed only inside its child module. Named `assert.js` and `sta.js`
must exist before ownership is stored, and replacing either entry revokes it.
Non-raw materialization resolves declared includes before host planning, fails
with the execution id and missing include name, and fixes host-requiring source
order as strict directive, host, `assert.js`, then `sta.js`. The source-and-resolved-helper census contains
797 physical sources and 1,547 executions; ten exact self-contained rewrite
sources account for 20 executions, leaving 787 physical sources and 1,527
executions that emit the host prelude. Agent workers receive that same
host/assertion/`sta.js` prelude through private materialized state rather than
runner-side source inspection. The pinned Atomics notification case also passes
through the product runner with that exact host order.

The four ProxyCreate target-shape sources also pass unchanged in sloppy and
strict modes (`8/8`). Removing their complete source-rewrite authority deletes
five semantic observations. All four now use the full LocalMerged `assert.js`,
including the two sameValue-only cases.

The Proxy apply non-callable-trap Realm source passes unchanged in sloppy and
strict modes (`2/2`) with complete LocalMerged Realm and assertion preludes.
Removing its exact-path rewrite and the later null-handler branch first left 10
observations assigned to T11. Retiring the complete `Proxy.revocable` rewrite
removes four more. Its 17 ordinary physical cases preserve their pinned
sources and declared helpers. `tco-fn-realm.js` preserves raw
`other.evalScript`, which resolves to the typed `RealmEvalScript` AOT
unsupported boundary owned by T13 rather than a manufactured Proxy result. The
Proxy checkpoint's 307-entry inventory assigned six observations to T11;
the remaining apply and construct rewrites stay open.

## Non-negotiable rules

1. Product execution remains `parse -> early errors -> spec IR -> lowering IR -> Wasm codegen`.
2. Do not add source-path, test-name, or assertion-text branches that manufacture Test262 results. Existing focused materializations must be catalogued and retired as general semantics replace them.
3. Every change starts with a reproducible failing real Test262 filter or exact case and ends with the same command green. Add a small CLI/engine regression fixture when it isolates the behavior better than the upstream case.
4. Preserve evaluation order, abrupt completion, realm ownership, property attributes, observable coercions, and proxy traps. Passing the happy path is not enough.
5. Do not hand-edit published conformance totals. Use `lila test262 publish-status` or `scripts/publish-real-status-low-ram.sh` after a complete verified matrix.
6. Keep `unsafe_code = "forbid"`. New dependencies require a reason, license review, deterministic behavior, and a clear Wasm/runtime story.
7. Feature PRs should not combine unrelated refactors. When a prerequisite interface is missing, land the interface first under its foundation task.
8. The interpreter stays quarantined. No CLI or library product path may execute user programs through `spec-exec` by default or as a silent fallback, and emitted Wasm never embeds an interpreter/VM or feeds user source to one. T27 enforces this in code; every other task must not reintroduce it.
9. Backend design targets the experimental Wasmtime lower bound from `AGENTS.md` (Wasm GC, typed function references, reference types, `exnref` exception handling). Do not build second object models, closure representations, or exception mechanisms for runtimes that lack these features; reject such runtimes at the boundary.
10. The legacy JavaScript product exists only in Git history at the recovery commit recorded by T28. Do not restore its compiler, runtime, package, publication, benchmark, or playground surfaces. JavaScript remains only as Rust-owned test/conformance data or vendored source.

## T01 comparison identity — 2026-09-06

Snapshot comparison now rejects a missing requested name instead of silently
substituting another complete run and reporting an empty diff. Explicit
self-comparison, complete-evidence validation and status/backlog discovery remain
intact. The retained `snapshot_comparison_identity` integration target covers the
boundary with product-front-end compile-negative fixtures, not real-suite
conformance evidence. See [T01](01-baseline-and-generated-backlog.md) and the
[comparison contract](../docs/rust-rewrite/test262-snapshot-comparison.md).

Next: finish the guarded current-pin Wasm-AOT matrix with fixed compiler inputs,
publish only its verified canonical artifacts, then generate and curate the
failure backlog. Compare explicitly named compatible runs; mandatory compiler
provenance is still a separate schema migration. No aggregate status or T26
closure is claimed by this repair.

## How to execute one task

1. Read this file, the selected task file, `AGENTS.md`, and the touched crate manifests.
2. Record the exact baseline commands and counts in the PR description.
3. Implement the smallest general semantic layer that fixes the whole failure family; do not special-case the representative test.
4. Run `cargo fmt --all --check`, targeted crate tests, focused CLI fixtures, and the real Test262 filter listed by the task.
5. Search for regressions in adjacent filters that share the same abstract operation or builtin.
6. In the PR description, report: files changed, semantic invariant added, exact tests/counts, remaining failures, and follow-up task IDs.

## Parallel work graph

### Bootstrap and coordination

T00 is complete. T01-T04 have landed useful infrastructure and module
boundaries, but their remaining acceptance criteria are still active.

| ID | Task | Parallel notes |
|---|---|---|
| [T00](00-operating-contract.md) | Operating contract and contribution protocol | Documentation/CI only; independent |
| [T01](01-baseline-and-generated-backlog.md) | Reproducible real-suite baseline and generated failure inventory | Independent; feeds every lane |
| [T02](02-modularize-ir-and-wasm-backend.md) | Split monolithic IR/backend modules | Coordinate before broad feature work |
| [T03](03-conformance-harness-integrity.md) | Honest Test262 harness and host contract | Independent of most language semantics |
| [T04](04-spec-operations-and-completion-abi.md) | Shared abstract operations and completion ABI | Foundation for most feature lanes |

### Core semantic foundations

These foundations are all in progress. Use the landed T02/T04 interfaces, and
coordinate changes to remaining large shared modules.

| ID | Task | Primary ownership |
|---|---|---|
| [T05](05-values-heap-gc.md) | Value representation, heap, GC, weak reachability | runtime + Wasm heap modules |
| [T06](06-realms-intrinsics-cross-realm.md) | Realms, intrinsics, host hooks, cross-realm identity | runtime + intrinsic bootstrap |
| [T07](07-parser-grammar-early-errors.md) | Parser boundary, grammar coverage, early errors | front + IR parser boundary |
| [T08](08-environments-control-flow.md) | Environments, TDZ, references, control flow and abrupt completion | IR lowering + control-flow emitter |
| [T09](09-functions-classes-private-elements.md) | Call/construct, functions, classes, private elements | function/class lowering and emitter |
| [T10](10-object-model-descriptors-exotics.md) | Ordinary objects, descriptors, integrity, exotic object protocol | object/descriptor modules |

### Feature lanes

These lanes have partial implementations at different depths. Their dependency
lists still identify semantic ownership; a dependency marked in progress does
not forbid focused work when its required interface already exists.

| ID | Task | Depends on |
|---|---|---|
| [T11](11-proxy-reflect-metaobject.md) | Proxy and Reflect meta-object protocol | T04, T05, T06, T09, T10 |
| [T12](12-modules-linking-loading.md) | Modules, linking, namespace objects, TLA host flow | T06, T07, T08, T09, T10 |
| [T13](13-dynamic-source-evaluation.md) | `eval`, `Function`, realm evaluation policy/implementation | T06, T08, T09, T12 |
| [T14](14-promises-jobs-async.md) | Promise jobs, async functions, async iteration | T04, T05, T06, T09 |
| [T15](15-generators-iterators-resource-management.md) | Generators, iterators/helpers, disposal | T04, T05, T08, T09 |
| [T16](16-arrays-and-array-builtins.md) | Array exotic semantics and complete Array API | T04, T05, T10 |
| [T17](17-typedarrays-binary-data-atomics.md) | ArrayBuffer, DataView, typed arrays, SAB, Atomics | T04, T05, T06, T10; async wait paths use T14 |
| [T18](18-strings-unicode.md) | ECMAScript strings and Unicode-correct String API | T04, T05, T10 |
| [T19](19-regexp.md) | ECMAScript RegExp syntax and semantics | T04, T05, T10, T18 |
| [T20](20-number-bigint-math-json.md) | Numeric semantics, BigInt, Math, JSON | T04, T05, T10 |
| [T21](21-symbols-collections-weakrefs.md) | Symbols, Map/Set, weak collections, WeakRef/finalization | T05, T06, T10 |
| [T22](22-date-temporal.md) | Date and Temporal | T04, T05, T06, T10, T18, T20 |
| [T23](23-intl402.md) | ECMA-402 Intl implementation | T06, T10, T18, T20, T22 |
| [T24](24-globals-errors-annexb-host.md) | Globals, native errors, Annex B, remaining host-visible builtins | T04, T06, T07, T09, T10 |

### Validation and closure

| ID | Task | Depends on |
|---|---|---|
| [T25](25-differential-fuzzing-performance.md) | Differential testing, fuzzing, timeout and code-size work | T01-T04; runs continuously |
| [T27](27-interpreter-quarantine-and-product-default.md) | Interpreter quarantine and Wasm-AOT product default | T02, T03; complete |
| [T26](26-zero-failure-conformance-closure.md) | Full pinned suite closure and release gate | All applicable tasks, including T27 |

### Repository ownership and identity

| ID | Task | Depends on |
|---|---|---|
| [T28](28-retire-legacy-js.md) | Retire the legacy JavaScript product and enforce the Rust-only boundary | T00; complete |
| [T29](29-lila-identifier-migration.md) | Coordinated Rust identifier migration to Lila; product cutover, version-6 Lila producer, and read-only version-4/version-5 decoder verified | T28; complete |

## Merge-conflict policy

T02 has landed initial boundaries, but several IR/lowering, object/operation and
builtin implementation files remain large shared hotspots. Coordinate broad
edits to those files. Feature work should continue moving code toward dedicated
IR, Wasm emitter, builtin and focused-fixture ownership. Shared ABI changes
belong in T04 and should land before dependent feature changes.

When two tasks require the same abstract operation, the first agent implements it in the shared operation layer with unit tests; the second consumes it. Do not copy slightly different `ToObject`, `ToLength`, `Get`, `Call`, iterator, descriptor, or completion logic into feature-specific code.

## Definition of done for a feature lane

A lane is complete only when:

- its real Test262 subtree is fully green for the Wasm-AOT backend and pinned revision;
- parser, early-error, runtime, backend, host-harness, timeout, and crash failures are all zero in that subtree;
- no test-specific semantic materialization remains for the covered behavior;
- descriptor metadata, subclassing/species, proxies, cross-realm behavior, abrupt completions, and coercion order have representative coverage;
- adjacent fake-suite and CLI regression tests remain green;
- README status is refreshed only if a complete real-suite publication was performed.

## Final acceptance target

`T26` owns the final evidence: a complete resumable Wasm-AOT matrix for the current pin, verified snapshot artifacts, zero crashes and bugs, no silent skips, no stale status claims, a green T27 interpreter-quarantine audit (no interpreter in product builds or emitted artifacts), and an explicit accounting of any dynamic-source cases permitted by `AGENTS.md`. Literal `passed == total` remains the project target; architecture exceptions must stay separately visible until the project deliberately resolves them.
