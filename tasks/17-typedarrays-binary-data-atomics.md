# T17 — ArrayBuffer, DataView, TypedArray, SharedArrayBuffer and Atomics

**Status:** In progress — broad binary-data support exists; GC/agents and full-tree closure remain

**Parallel group:** Feature lane; split internally by API family  
**Depends on:** T03, T04, T05, T06, T10; iterator paths use T15; `waitAsync` uses T14  
**Blocks:** Binary-data and concurrency portions of T26

## Current repository state

ArrayBuffer, SharedArrayBuffer, DataView, TypedArray and Atomics have dedicated
backend implementations, including resizable/growable backing-store and
`waitAsync` work with focused fixtures. Binary-data-specific harness rewrites
remain, real GC is unavailable, and the shortcut-free real-agent/full-tree
acceptance criteria have not been demonstrated on a current complete matrix.
The token-aware shortcut inventory assigns 80 observations to T17, split
between five semantic shortcuts and 75 diagnostic guards. The earlier
token-aware checkpoint counted 190. That count was higher than the line-oriented
checkpoints below because the newer scanner covers exact rewrite calls, source
contract guards and normalized selector tables, not because materializers were
restored.

The exact `%TypedArray%.prototype.at` helper matcher and source guard are now
gone. A 15-source/30-execution invariant pins unchanged bodies in both Script
modes and both prelude profiles: all 13 typed-array-helper consumers use the
complete vendored `testTypedArray.js`, three also use the complete configured
`propertyHelper.js`, and the two resizable-helper cases retain only T13's
separately owned static-subclass substitution. This removes one semantic
shortcut and one diagnostic guard while keeping the four exact resizable-buffer
support admissions.
The rebuilt post-delete leaf passes all `30/30` sloppy/strict Wasm-AOT
executions, and exact controls for the surviving compact property, split
dispatcher and shared T13 helper routes pass `6/6`, with every non-success
bucket at zero.

The exact `%TypedArray%.prototype.filter` and `map` source matchers and their
compact prelude consumers are now gone. A shared invariant scans all 84
physical sources and 168 sloppy/strict executions in each directory. It pins
the 18 retired matcher contracts, both prelude stores and exact materialized
bytes: filter has 81 complete `testTypedArray.js` consumers and three sources
without that include; map has 79 and five. Six metadata sources also use the
complete configured `propertyHelper.js`. Sixteen matcher paths move from the
intrinsic fragment to the complete helper, while the two controls were already
complete. This removes two semantic shortcuts and two diagnostic guards
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
provenance is exactly `28/82`. The deletion removes three T17 semantic
shortcuts and one T17 diagnostic guard; the other semantic and diagnostic
deletions belong to T15. The rebuilt release CLI passes six representative
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
zero. This is not a complete `420/420` product replay or broad T17 closure.

The nine DataView `buffer`, `byteLength` and `byteOffset` accessor-metadata
cases no longer use a handwritten source replacement. Their pinned
`length`/`name`/`prop-desc` sources and full upstream `propertyHelper.js` pass in
raw sloppy and strict runs (`18/18`). Ordinary materialization uses the complete
embedded LocalMerged helper, while a unit pins its exact source/provenance and
the unchanged test suffix. That first chunk deleted four audited semantic
shortcuts.

The nine accessor wrong-receiver sources also no longer use handwritten
assertions. All eighteen raw sloppy and strict runs pass against the full
upstream assertion helper. Their primitive and wrong-brand receiver checks
materialize from the unchanged pinned sources with only the complete
LocalMerged assertion prelude. A real-source 3x3 invariant pins empty includes,
AOT applicability, exact source identity and assert-only provenance. Removing
that rewrite and its now-dead accessor mapper deletes five audited semantic
shortcuts.

The five `ArrayBuffer.isView` typed-array-helper cases no longer use handwritten
per-constructor assertions or omit `testTypedArray.js`. Their pinned direct-view,
buffer, constructor-object, subclass and callable-alias sources pass unchanged
with the full vendored helper in sloppy and strict modes (`10/10`). A
real-source unit pins exact source identity, byte-for-byte full-helper
provenance and the removed skip boundary. The four sameValue-only sources retain
the shared trimmed SameValue assertion route, while the subclass source uses the
complete merged assertion prelude. Removing the original four rewrites and
helper skips deleted eight audited semantic shortcuts; deleting the
callable-alias expansion and its final path-specific skip deletes one more.

The typed-array buffer `defined-length.js` constructor case also executes its
unchanged pinned source with the full upstream `testTypedArray.js` in sloppy and
strict modes (`2/2`). Its ordinary materialization keeps the legitimate trimmed
SameValue assertion route but now uses the complete vendored typed-array helper.
A real-source unit pins the exact source, full-helper bytes and provenance, and
the removed helper-skip boundary. Deleting its handwritten constructor fan-out
and path-specific skip removes one audited semantic shortcut.

The four DataView `getBigInt64`/`getBigUint64` ToIndex error and coercion cases
now execute their unmodified pinned sources with the complete merged assertion
prelude. All four pass in sloppy and strict modes (`8/8`), and a real-source
unit pins their empty include lists, exact source suffixes, and LocalMerged
assertion source/provenance. This deletes five audited semantic shortcuts and,
after the accessor wrong-receiver cleanup, leaves 88 T17-owned entries in the
then-current 256-entry line-oriented inventory.

The eight numeric DataView `set-values-return-undefined` cases also run their
unmodified pinned sources with the full upstream assertion and
`byteConversionValues.js` harnesses in sloppy and strict modes (`16/16`). Their
ordinary materialization uses the complete LocalMerged assertion prelude and
complete VendoredHarness conversion table. A real-source invariant pins those
origins, exact helper source, unchanged test suffixes, and exclusion of the
trimmed SameValue route. Removing the standalone handwritten conversion
rewrite deletes four audited semantic shortcuts. After the callable-alias
and typed-array defined-length cleanups, 82 T17-owned entries remained in the
249-entry inventory.

The four `%TypedArray%[@@species]` `result`, `prop-desc`, `name` and `length`
sources now run unchanged with the full upstream assertion, property and
typed-array helpers in sloppy and strict modes (`8/8`). Ordinary materialization
uses the complete VendoredHarness `testTypedArray.js` for every case and the
complete embedded LocalMerged `propertyHelper.js` for the two descriptor
metadata cases. The two sameValue-only cases keep the legitimate trimmed
SameValue assertion route, and the real-source invariant pins that nonclaim
alongside exact source and helper provenance. Removing the species-specific
compact-helper matcher and its two consumers deletes one audited semantic
shortcut, leaving 81 T17-owned entries in the then-current 248-entry
line-oriented inventory. The
following 666-dump workspace semantic golden passes `2/2` in 704.11 seconds,
adds only the independent array key-selection witness, removes none and
preserves every retained non-accounting summary.

The fifteen ArrayBuffer accessor and `slice` metadata sources now run unchanged
with the complete upstream assertion and property helpers in sloppy and strict
modes (`30/30`). Their ordinary materialization uses the complete embedded
LocalMerged assertion and property sections; a second invariant route pins the
complete upstream `propertyHelper.js` with VendoredHarness provenance so a
compact substitute cannot remain hidden behind the configured local helper.
The real-source matrix also pins all source bodies and declared includes.
Deleting the ArrayBuffer-specific matcher, compact projection and its special
byteLength property prelude removes two audited semantic shortcuts. Removing
the DataView method metadata rewrite removes two more, leaving 77 T17-owned
entries in the then-current 240-entry line-oriented inventory. The following
workspace semantic golden passes `2/2` in 702.89 seconds with 667 dumps, adds
only the iterator-policy witness and removes none. Of 666 retained dumps, 665
preserve every non-accounting summary; only the independently expanded Promise
callback witness changes structurally.

The forty-two DataView getter and setter `length` and `name` metadata sources
also run unchanged with the complete upstream assertion and property helpers in
sloppy and strict modes (`84/84`). Their ordinary materialization uses the
complete embedded LocalMerged assertion and property sections, and a separate
route pins the complete upstream `propertyHelper.js` with VendoredHarness
provenance. The real-source 21x2 matrix pins every source body and the current
pin's absent `setBigUint64` metadata pair. Deleting only the metadata rewrite
and dispatcher removes two audited semantic shortcuts, leaving 77 T17-owned
entries in the then-current 240-entry line-oriented inventory; the shared
method mapper remains owned by the range and resizable rewrites.
The following shared workspace semantic golden passes `2/2` in 696.00 seconds
with 668 dumps, adds only the independently expanded shape-accessor witness,
and removes none. After accounting normalization, 664 of 667 retained dumps
are equal; the only structural changes are the intended Array reduce, Promise
internal-callback Realm, and TypedArray constructor no-species witnesses.

The TypedArray sort value matrix, `TypedArray.of` zero case and eleven borrowed
Array callback resize cases now materialize their 13 pinned Test262 source
bodies unchanged. A 13x2 invariant pins both Script modes, original bytes,
declared includes, the supported-feature boundary, known-static identity and
the absence of a self-contained replacement. It also checks exact bytes and
origins through the LocalMerged and vendored-only prelude stores. The
LocalMerged route uses no `sta.js`, preserves the legitimate trimmed
CompareArray and SameValue assertion routes for the sort and `of` cases, and
uses the complete merged assertion for all eleven callbacks. Both routes use
the complete vendored `testTypedArray.js` and optional `compareArray.js`; the
vendored-only route also uses complete `sta.js` and `assert.js`. Removing the
constructor fan-outs, callback replacements, dispatch paths and typed-array
helper omission deletes exactly 29 T17 semantic observations. An isolated
post-delete Wasm-AOT run of those exact 13 sources passes all `26/26`
sloppy/strict executions with every non-success bucket at zero. At the later
Array `includes` retirement checkpoint, the inventory had 353 entries, 205
semantic shortcuts and 161 T17-owned entries.

The handwritten metadata and constructability arms for the eight top-level
DataView constructor surface sources are removed. A direct raw preflight before
deletion passed all `16/16` sloppy/strict executions
through complete vendored `sta.js`, `assert.js` and the declared
`propertyHelper.js` or `isConstructor.js`. Every execution reported
`backend_used: WasmAot`, with every non-success bucket at zero. This is scoped
unchanged-source evidence, not a post-delete production-dispatch result. The
8x2 replacement invariant pins exact modes, source bytes, includes,
supported-feature and no-rewrite boundaries, then compares exact source and
origins through LocalMerged and vendored-only stores. LocalMerged uses the
trimmed SameValue assertion for `constructor.js`, `proto.js` and
`extensibility.js`; the five helper-bearing sources use the complete embedded
assertion and declared helper. The vendored-only route uses complete `sta.js`,
`assert.js` and the declared helper. The rebuilt production dispatcher then
passes the same exact `16/16` cohort with every non-success bucket at zero. The
surviving constructor dispatcher keeps all validation, range, custom-prototype,
instance-extensibility and SAB authorities. Removing these eight arms changes
its one match fingerprint but no observation count, leaving T17 at 161 at that
checkpoint.

The exact `built-ins/Array/prototype/map/resizable-buffer.js` source passed a
pre-delete raw `4/4` matrix across both Script modes and both prelude stores
with exact source bytes and only T13's static-subclass helper substitution. The
unmodified helper still stops at the explicit Function-constructor
AOT-unsupported boundary, so this is neither full-helper support nor
post-delete production-dispatch evidence. The expanded six-source invariant
pins the map source, declared comparison and resizable helpers, and exact
LocalMerged and vendored-only bytes and origins in both modes. Deleting only
the map branch from the known-static `for-of` rewrite removes one T17 semantic
observation. The remaining TypedArray accessor authority and shared
resizable-directory substitutions stay intact. That checkpoint's inventory had
352 entries, including 204
semantic shortcuts; T16 owns 69. T17 owns 160, split between 79 semantic
shortcuts and 81 diagnostic guards. After deletion, the rebuilt production
dispatcher passed the exact map source in both Script modes (`2/2`) with every
failure and non-success bucket at zero.

The seven neighboring Array iteration `resizable-buffer.js` sources for
`find`, `findIndex`, `findLast`, `findLastIndex`, `every`, `some` and `filter`
then passed a raw `28/28` matrix across both Script modes and both prelude
stores. The separate `find` preflight supplied `4/4`; sibling proof lanes
supplied `24/24`. Every run used Wasm-AOT with the exact pinned body and only
T13's static-subclass replacement in `resizableArrayBufferUtils.js`; the
unmodified helper retains the explicit Function-constructor AOT-unsupported
boundary. The expanded real-source invariant pins the exact two-store
materialization, including `filter`'s comparison helper. Deleting the complete
Array iteration rewrite authority removes eight T16 semantic observations but
does not change T17's shared helper substitution or TypedArray authorities.
After deletion, the rebuilt production dispatcher passed the exact seven-source
cohort in both Script modes (`14/14`) with every failure and non-success bucket
at zero. That historical 344-entry, 196-semantic checkpoint assigned 61
observations to T16 and kept T17 at 160, split between 79 semantic shortcuts
and 81 diagnostic guards. The six pinned Array `reduce` and `reduceRight`
resizable-buffer sources next passed an exact raw `24/24` matrix across both
Script modes and both prelude stores. Every execution used Wasm-AOT with the
exact pinned source and declared `compareArray.js`; only T13's static-subclass
replacement in `resizableArrayBufferUtils.js` changed. A representative run
with the unmodified helper stopped at the explicit Function-constructor
dynamic-code-generation boundary, so this is scoped pre-delete evidence rather
than full-helper support. The expanded nineteen-source invariant pins exact
source, modes, stores, preludes, origins, suffixes, no-rewrite boundaries and
T13 contract membership. Deleting the complete reduce rewrite, its sole
dispatcher call, both one-caller source builders and the obsolete synthetic
rewrite test removes six T16 semantic observations without changing T17's
shared helper substitution or TypedArray authorities. Broad Array reduce
admission and neighboring resizable authorities remain. After deletion, the
rebuilt production dispatcher passed the exact six-source cohort in both Script
modes (`12/12`) with every failure and non-success bucket at zero. The current
338-entry, 190-semantic checkpoint assigned 55 observations to T16 and kept
T17 at 160. The four pinned Array `indexOf` and three pinned Array `lastIndexOf`
resizable-buffer sources then passed an exact raw `28/28` matrix across both
Script modes and both prelude stores. Every run used Wasm-AOT with the exact
source and declared resizable helper; only T13's static-subclass replacement
changed. A representative unmodified-helper run stopped at the explicit
Function-constructor dynamic-code-generation boundary. Review found that the
handwritten `lastIndexOf` rewrite bypassed the feature gate because Array
`lastIndexOf/` lacked broad resizable admission. One closed Array search prefix
set now admits `includes/`, `indexOf/` and `lastIndexOf/`, with a witness for
all three. The expanded twenty-six-source invariant pins both stores, both
modes, exact source and prelude bytes, includes, origins, suffixes, no-rewrite
boundaries and T13 contract membership. Deleting both search rewrite functions,
their two dispatcher calls, seven path predicates, two synthetic tests and two
dead shared source builders removes nine T16 semantic observations;
consolidating the previous two Array-search diagnostic predicates removes one
more T16 observation. T13's helper substitution and the broad TypedArray search
admission remain, along with neighboring Array mid-iteration and
`toLocaleString` authorities. After deletion, the rebuilt production dispatcher
passed the exact seven-source cohort in both Script modes (`14/14`) with every
failure and non-success bucket at zero. That Array-search checkpoint had 328
entries and 181 semantic shortcuts, with 45 observations assigned to T16. The
fourteen Array `every`/`some`/`filter`/`find`/`findIndex`/`findLast`/
`findLastIndex` grow/shrink-mid-iteration sources next passed a raw `56/56`
matrix across both Script modes and both prelude stores, split into `24/24`
quantifier and `32/32` find-family executions. Every run used Wasm-AOT,
preserved the exact source and ordered `compareArray.js` plus
`resizableArrayBufferUtils.js` includes, and changed only the existing T13
dynamic-subclass block. An unmodified-helper witness stopped at the explicit
Function-constructor dynamic-code-generation boundary. The exact invariant
now owns all fourteen paths, modes, stores, bytes, origins, suffixes,
no-rewrite checks and T13 contract membership. Removing the complete shared
Array rewrite, sole dispatcher call, one-caller constructor list and synthetic
replacement test deletes its entrypoint and all fifteen direct predicates.
All seven broad Array admissions, the T13 static-helper contract and
neighboring Array values, iterator and `toLocaleString` authorities remain.
After deletion, the rebuilt production dispatcher passed the exact
fourteen-source cohort in both Script modes (`28/28`) with every failure and
non-success bucket at zero. That historical inventory had 312 entries and 165
semantic shortcuts, with 29 assigned to T16. The three pinned Array `values`
base/grow/shrink resizable-buffer sources then passed all `12/12` raw Wasm-AOT
executions across both Script modes and both prelude stores. The sources and
their ordered comparison plus resizable-helper includes stayed exact; only
T13's static-subclass replacement ran. Its helper fingerprint is
`0x6466_6602_9ee8_9d5d`, and the case fingerprints are
`0x5e5c_6ead_7b7c_0dda`, `0x3d18_7152_c6ff_a624` and
`0x60c2_a9ec_1dff_dd03`. Changed helper, path, include or source bytes keep
`new Function` and reach the explicit Function-constructor dynamic-code-
generation boundary. The exact invariant covers all three modes, stores,
bytes, origins, suffixes, no-rewrite checks and T13 memberships. Deleting both
complete Array-values rewrites, their two sole dispatch calls and both obsolete
synthetic tests removes two entrypoints and three direct predicates. Broad
Array-values admission, Array keys/entries iterator paths, T13's contract and
neighboring `toLocaleString` authority remain. The current 307-entry inventory
has 160 semantic shortcuts and assigns 24 observations to T16. T17 remains at
160, split between 79 semantic shortcuts and 81 diagnostic guards. After
deletion, the rebuilt production dispatcher passed the exact three-source
cohort in both Script modes (`6/6`) with every failure and non-success bucket
at zero.

The three pinned Array `toLocaleString` resizable-buffer sources then passed an
exact raw `12/12` matrix across both Script modes and both prelude stores. Every
execution used Wasm-AOT, kept the pinned source, declared only
`resizableArrayBufferUtils.js`, and applied only T13's replacement of the
dynamic subclass block with three static classes. The helper fingerprint
`0x6466_6602_9ee8_9d5d` and case fingerprints `0x9da9_18f5_d04d_d764`,
`0xc380_4490_04ea_5b59` and `0x07d1_d14e_3a0b_bb89` admit that one change.
Changed helper, path, include or source bytes retain `new Function`; a
representative unmodified-helper run stopped at the explicit
Function-constructor dynamic-code-generation boundary. The expanded invariant
pins all three sources, modes, stores, bytes, origins, suffixes, no-rewrite
checks and T13 memberships. Deleting the complete Array `toLocaleString`
rewrite, its sole dispatch and obsolete synthetic test removes one entrypoint
and three direct predicates. Broad Array `toLocaleString` admission and its
witness, T13's helper contract, TypedArray `toLocaleString` behavior and the
neighboring DataView rewrite authorities remain. The pre-retirement baseline
contained 307 entries and 160 semantic shortcuts. The regenerated source ledger
contains 303 entries: 35 legitimate harness adaptations, 112 diagnostic
instrumentation sites and 156 semantic shortcuts. T16 owns 24; T17 remains at
160 and T18 owns 12. After deletion, the rebuilt production dispatcher passed
the exact three-source cohort in both Script modes (`6/6`) with every failure
and non-success bucket at zero.

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
LocalMerged assert-only materialization and vendored `sta.js` plus `assert.js`
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
and no-rewrite boundary. LocalMerged materialization uses `sta-preamble.js`
then `assert.js` for the 20 conversion-order sources and only `assert.js` for
the 21 out-of-range sources; vendored-only materialization always uses complete
`sta.js` then `assert.js`. Deleting the sole dispatcher call, complete range
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
materialization uses `sta-preamble.js` then `assert.js`; vendored-only
materialization uses `sta.js` then `assert.js`. Deleting the sole dispatcher
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
original bytes. It also pins the four LocalMerged groups: 18 trimmed SameValue,
14 full assertion, nine `sta-preamble.js` plus assertion and two assertion plus
property-helper sources. Vendored-only materialization uses exact `sta.js` and
`assert.js` bytes, plus `propertyHelper.js` for the two extensibility sources.
Deleting the sole dispatcher call, complete constructor rewrite, its sole
filename selector and the obsolete synthetic test removes exactly seven T17
semantic observations. The regenerated ledger contains 248 entries: 35
legitimate, 112 diagnostic and 101 semantic. T16 owns 24; T17 owns 105, split
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
coverage is a materialization/provenance assertion, not an execution claim: a
complete host-before-harness `$262` model remains T03 work. Deleting both
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

The three strict `%TypedArray%.prototype` iterator methods now select the closed
`ArrayIteratorReceiverPolicy::TypedArray` instead of sharing a raw validation
Boolean with generic Array methods. The TypedArray policy is the sole strict
brand/view-validation route and directly materializes the typed iterator
record, while `GenericArrayLike` retains runtime TypedArray specialization for
borrowed Array iterator methods. The bounded 3/3 producer and two-projection
guard passes `3/3`, and the finite all-six-method CLI witness passes `1/1`.
This closure does not change iterator kind encoding, record layout,
resizable/detached-buffer semantics, Realm behavior or Test262 shortcuts. The
following 667-dump workspace semantic golden passes `2/2` in 702.89 seconds,
adds only the iterator-policy witness, removes none and changes no retained
non-accounting summary except the independently expanded Promise callback
witness.

TypedArray iterator construction now accepts the same closed
`ArrayIteratorKind::{Key, Value, KeyAndValue}` domain and writes a stable word
only into the private iterator record. TypedArray `next` decodes through every
row, exhaustively emits key, value or pair semantics, and traps rather than
defaulting if the private word violates the compiler invariant. This
source-equivalent type closure does not change buffer witnessing or complete
TypedArray or T17.

The entry-Realm hidden `%TypedArray%` constructor now receives its prototype at
birth through `FunctionPrototypeMaterialization::BootstrapSupplied`. Its own
`prototype` property is appended once with the required non-writable,
non-enumerable and non-configurable attributes, so descriptor validation no
longer rejects a later attempt to replace an automatically created prototype
and reflective reads reach the same object on which bootstrap publishes the
TypedArray methods. The focused structure guard passes `3/3`, and the retained
iterator fixture passes `1/1` after observing the repaired prototype graph
before exercising its methods. Five pinned prototype and descriptor leaves pass
both variants (`10/10`) with every failure bucket at zero.

The hidden constructor now has its own closed
`StandardBuiltinId::TypedArrayConstructor` identity rather than borrowing
`Function` metadata. The catalog makes that function constructable, gives it
the native name `TypedArray` and length zero, and records that its body cannot
complete normally. Lowering `Object.getPrototypeOf` on any concrete typed-array
constructor now returns that exact target; mixed call and construct candidates
therefore exclude its always-throwing branch from their normal result. Entry-
and created-Realm bootstrap each materialize a distinct function with the
Realm's `%Function.prototype%`, the immutable intrinsic `prototype` link and
the reciprocal constructor link. Direct call and direct construction throw the
function Realm's `TypeError`, while the value remains a valid `newTarget` for
`Reflect.construct`. The focused IR regressions pass `4/4`, the structural
guard passes `5/5`, and the two-Realm Wasm-AOT fixture passes `1/1` on
2026-08-31. The six exact pinned `name`, `length`, `invoked`, `prototype`,
`prototype/constructor` and `Uint8Array/proto` leaves pass both variants
(`12/12`) with every non-success bucket at zero. This closes the identity,
name, length and call/construct debt recorded by the focused
[prototype-publication contract](../docs/rust-rewrite/contracts/typed-array-intrinsic-prototype-publication.md).
The following shared 683-dump semantic golden passes `2/2` in 655.10 seconds,
adds and removes none, and leaves 682 retained summaries equal after accounting
normalization; its sole structural change is the independent Array corruption
witness.

The `waitAsync` timeout checkpoint now takes one closed private
`AtomicsWaitAsyncTimeoutCheckpointMode::{Drain, Poll}` domain. Its two named
wrappers select the mode, and an exhaustive match makes Poll the sole early-exit
path while Drain continues waiting through finite deadlines. The bounded
structure target passes `3/3`, the existing timeout/microtask CLI fixture passes
`1/1`, and the 633-fixture Wasm golden capture is byte-identical. This removes
an internal boolean protocol; it does not establish multi-agent or full-tree
Atomics closure.

`Atomics.wait` and `Atomics.waitAsync` result spelling now crosses the private,
non-derived `AtomicsWaitOutcome::{Ok, NotEqual, TimedOut}` domain. Thirteen
semantic producers retain the exact `4/3/6` variant split, while one borrowed
exhaustive projection owns the three ECMAScript strings and both result helpers
reject arbitrary string input. Wasm atomic-wait codes and `agent_call` waiter
identifiers, counts and statuses remain numeric ABI state and are decoded only
when materializing a JavaScript result or fulfilling a Promise. The focused
[outcome contract](../docs/rust-rewrite/contracts/atomics-wait-outcome.md)
records the exact producer census, string-pool order, numeric-boundary guard and
retained runtime witnesses. At the 2026-08-27 checkpoint, its bounded structure
target and the CLI `atomics_wait` filter each pass `4/4`. The four exact
current-pin waitAsync result-object leaves pass both sloppy and strict variants,
for `8/8` Wasm-AOT executions under `--jobs 1 --threads 1`, with every reported
failure and non-success bucket at zero. This is a source-equivalent invariant
checkpoint; no full Atomics or T17 closure is claimed.

The nine load/store/RMW `Atomics` methods now carry their shared compiler
policy through a non-derived, non-copyable `AtomicsIntegerOperation` authority.
Five borrowed exhaustive decisions own value arity, both diagnostic tables,
the core operation and result publication. In particular, `store` has an
explicit empty post-operation arm while the other eight rows own the unchanged
old-value Number/BigInt publication body; a future row can no longer inherit
that behavior from `operation != Store`. The six RMW rows still narrow to the
existing `AtomicsRmwOperation`. The lexical recursive 48-identifier census,
exact nine producer routes, all policy tables and contiguous address/core
operation/result/reverse-release order are recorded in
[`atomics-integer-operation.md`](../docs/rust-rewrite/contracts/atomics-integer-operation.md).
This is source-equivalent compile-time hardening, not new Atomics behavior,
multi-agent evidence or T17 closure. The bounded structure target passes
`3/3`, the four exact operation CLI witnesses pass `4/4`, and the six selected
Number/BigInt add/store plus compare-exchange/xor leaves pass all `12/12`
sloppy/strict Wasm-AOT executions with every reported failure bucket at zero.
Independent dry re-review is clean after the recursive ownership and
contiguous emission guards were hardened. The following shared workspace
compile, formatter, module-boundary, task-plan and diff gates all pass.

The shared Atomics integer element-kind consumers now require a private,
move-only `ValidatedAtomicsIntegerElementKindLocal` instead of an arbitrary
`u32` Wasm local. `Atomics.wait`, `Atomics.waitAsync` and the nine-operation
load/store/RMW compiler each reserve one pending local and hand it to the sole
validation boundary. That boundary exhaustively selects the existing
any-integer or Int32/BigInt64 waitable domain, emits the current-function-Realm
TypeError branch, and only then mints the validated authority. The owners borrow
it across normalize, load, store, compare-exchange and RMW projections and
consume it only when releasing the local. The focused
[`atomics-integer-element-kind-local.md`](../docs/rust-rewrite/contracts/atomics-integer-element-kind-local.md)
contract and structure guard pin the complete three-producer/seven-consumer
census. Standalone source-only execution passes `4/4` for the new guard, `3/3`
for the updated integer-operation fingerprint and `5/5` for the updated shared
witness guard. This is source-equivalent invariant hardening; workspace compile
and runtime evidence remain deferred to the shared batch checkpoint, and no
new Atomics semantics, Test262 progress or T17 closure is claimed.

The cross-instance async-waiter transport now shares the closed
`lila-runtime::AgentHostOperation` wire domain with the rest of the Wasm agent
ABI. Registration, polling, notification and cancellation are typed at every
AOT producer and exhaustively dispatched by the engine; their stable wire
values remain 10 through 13. This prevents producer/consumer opcode drift but
does not by itself prove waiter semantics or multi-agent stress safety.

Resizable-buffer observation now has a typed AOT seam for callback and
search/access consumers. A private TypedArray view record keeps the stored fixed
byte extent immutable, while a fresh buffer witness derives out-of-bounds
state, element length and an element-aligned index bound from one cached
backing-store length. Its closed use domain distinguishes validated TypedArray
method entry, generic Array length snapshots, live integer-indexed property
observations and the three-kind view-accessor projection. The callback families
shared with T16 use that seam,
including both `reduce` property checks; so do `at`, the generic Array index
searches and the non-generic TypedArray search methods. TypedArray search length
is validated and snapshotted once at method entry, while generic Array search
keeps its `LengthOfArrayLike` and live integer-indexed behavior. Focused
contracts cover fixed-view out-of-bounds/regrow behavior and the Uint16
odd-byte floor.

The non-generic `includes`, `indexOf` and `lastIndexOf` compiler now projects
its closed `TypedArraySearchKind` through twelve direct exhaustive matches.
The kind no longer implements equality, and the former six `==` plus three
`!=` decisions cannot silently give a later search kind IndexOf-like defaults.
The three dispatcher-to-wrapper and wrapper-to-kind producers remain exact;
SameValueZero/strict equality, Boolean/index results, forward/reverse
`fromIndex`, invalid-index handling and traversal retain their existing
pairings. The exact census and expected byte-identical output are recorded in
the focused
[TypedArray search-kind contract](../docs/rust-rewrite/contracts/typed-array-search-kind.md).
The bounded structure target passes `3/3`, and the exact existing
`wasm_typedarray_search.js` CLI fixture passes `1/1`. `cargo xc` and the shared
671-dump semantic golden also pass; all 669 retained dumps are equal after
accounting normalization, confirming the expected source-equivalent output.
No Test262 baseline or published-count change is claimed.

TypedArray `reduce` and `reduceRight` also share the private
`ArrayReduceDirection` domain with their generic Array counterparts. Their two
dispatch producers select opposite variants explicitly; method and diagnostic
text plus all traversal decisions are exhaustive projections rather than a
raw reverse Boolean. The bounded four-producer/nine-decision guard and the
existing CLI reduce fixture include distinct forward and reverse TypedArray
witnesses.

Batch AQ makes the raw `ArrayReduceDirection` and shared reducer private to
`builtins/array.rs`. The fixed TypedArray `reduce` and `reduceRight` entries
select opposite directions internally, and standard dispatch cannot import or
pass the raw direction. At the 2026-08-28 Batch AQ checkpoint, `cargo xc` is
green, the strengthened direction and neighboring receiver-kind structure
targets each pass `4/4`, and the exact Array/TypedArray forward/reverse reduce
CLI control passes `1/1`. This source-equivalent tightening claims no new
TypedArray behavior and no Batch AQ Test262 or semantic-golden result.

Batch AR makes the raw shared-`at` policy and compiler private to `array.rs`.
Standard dispatch can call only the fixed Array and TypedArray entries and
cannot import, construct or pass the private `ArrayAtReceiverPolicy`. The
frozen 34-line compiler has SHA-256
`4888ef68f6f42b58d9e14480d5381cf64018176ed21504a10fc6883dac564aaa`;
normalizing its private name and visibility reproduces that hash exactly. At
the 2026-08-28 Batch AR checkpoint, `cargo xc` is green, the strengthened
structure target passes `4/4`, and the exact runtime-kinds CLI control passes
`1/1`. This source-equivalent tightening claims no new Array behavior and no new
TypedArray behavior, and no Batch AR Test262 or semantic-golden result.

The strict TypedArray reducer and `forEach` entries also share the private
`ArrayCallbackReceiverKind` domain with their generic Array counterparts. The
kind no longer implements equality: six reducer receiver decisions, five
`forEach` receiver decisions and the reducer's two existing direction-paired
receiver decisions are direct exhaustive matches. The former two
`typed_array_only` Boolean carriers can no longer route a future entry family
through generic Array defaults. The focused
[Array callback receiver-kind contract](../docs/rust-rewrite/contracts/array-callback-receiver-kind.md)
pins all six dispatcher producers while preserving each strict
`ValidatedMethodEntry` witness and each generic live
`IntegerIndexedProperty` witness. The bounded structure target passes `4/4`,
and the three existing focused CLI witnesses pass `3/3`. The shared 674-dump
semantic golden passes `2/2` in 717.58 seconds; this source-only closure adds no
fixture and all 671 retained dumps are equal after accounting normalization.
It claims no TypedArray semantic, Test262 baseline or published-count change.

The shared integer-index validity predicate now consumes the closed
`IntegerIndexedProperty` projection of that same witness. It classifies the
numeric index before loading one immutable view and making one non-throwing
backing-store observation; detached, fixed/tracking out-of-bounds and
index-at-or-above-current-length states all project to an absent property.
Current `Get`, `HasProperty`, `GetOwnProperty`, `DefineOwnProperty`, `Set`,
`Delete` and method callers inherit that observation without reconstructing
private slots, reading backing length separately or dividing byte length
locally. The focused
[integer-index buffer-witness contract](../docs/rust-rewrite/contracts/typed-array-integer-index-buffer-witness.md),
structural guard and expanded `Reflect.has` CLI fixture are written and
focused-verified: `cargo xc` is green, the structure target passes `2/2`, and
the exact CLI fixture passes `1/1`. The direct pinned Test262 leaf discovers
two variants but both stop at the harness's declared `resizable-arraybuffer`
feature gate, so no Test262 pass or unsupported-retirement claim is made.

The witness is still not the universal integer-indexed exotic protocol. Key
classification and each internal method's descriptor, prototype and result
policy remain separate owners, other binary-data consumers still use older
emitters, and no Test262 resizable-buffer rewrite has been retired. The
TypedArray iterator boundaries are migrated separately below; ordinary Array
iterators do not require a TypedArray backing-store witness.
Constructor/subclass and BigInt variants represented by those rewrites remain
separate closure work. The shared `at` emitter encodes its generic-array-like
versus validated-TypedArray receiver policy as a closed enum; the old raw
boolean can no longer route a new caller to the wrong incompatible-receiver
behavior.

The payload-bearing `TypedArrayWitnessUse` is now a move-only witness-use
authority. Its validation decision borrows the policy without binding any
destination local, while its final consuming projection owns the sole result
publication from the cached backing-store observation. Copying the authority
can therefore no longer duplicate publication or permit post-projection reuse.
The recursive Rust-lexical
`typed_array_witness_use_ownership_structure` guard pins the attribute-free
four-variant declaration, exact global route census, sole witness boundary and
borrow-before-consume order. This is source-equivalent hardening and does not
claim new TypedArray behavior, Test262 progress or T17 closure; focused results
are `4/4` for the ownership guard, `5/5` for the neighboring Atomics witness
guard and `1/1` for the exact TypedArray iterator CLI witness. The older
iterator structure target is currently `1/2` because its unrelated producer
subtest has a stale `StandardBuiltinId::ArrayPrototypeKeys` source marker; its
Realm-validation subtest passes with the borrowed match.

ArrayBuffer slicing now has a closed late-source-observation seam. The three
builtin operations project exhaustively to detachable-bounded, shared-bounded,
or detachable-exact-final copy policy. The sole copy writer rechecks ordinary
detachment and reloads current source length and data after observable work.
Ordinary `slice` bounds the copy by the bytes still available from the initially
normalized start, so a species-provided target suffix remains untouched.
`sliceToImmutable` instead rejects a current length below the resolved final
bound before allocating its target, then copies the exact requested length.
Shared sources keep their distinct non-detachable bounded branch. The focused
[slice source re-observation contract](../docs/rust-rewrite/contracts/array-buffer-slice-source-reobservation.md)
and CLI fixture cover detachment during coercion/species, ordinary bounded
resizable shrinkage, and `sliceToImmutable` detach-versus-short-source error
precedence; this is not yet a claim of complete ArrayBuffer or shared memory
correctness.

That payload-bearing policy is now a non-`Clone`, non-`Copy` owned authority
with 31 lexical mentions. The grouped builtin owner makes five borrowed
pre-handoff decisions before its sole owned copy-writer handoff; the writer
makes two borrowed writer decisions before its final consuming
source-selection decision. The structure guard pins all three producer rows,
all eight exhaustive projections, the one handoff, and complete owner/writer
fingerprints. This makes duplicate publication or post-handoff reuse a Rust
move error without changing emitted instructions or claiming broader slice
conformance. Focused evidence is `5/5` for the new structure guard, `3/3` for
the neighboring bound guard, and `1/1` for the exact Wasm-AOT source
re-observation CLI witness.

Batch AI makes the neighboring `ArrayBufferSliceCopyLocals` bundle a single
move-only carrier. Its five production mentions comprise the private
declaration and constructor, standard-builtin import, sole constructor call and
owned copy-writer parameter. That writer is the only consumer and makes
thirteen field projections with the exact `4/3/1/5` source-object, start, final
and requested-length split. The recursive guard pins the attribute-free
four-field declaration, absence of incidental capabilities or alias/borrowed
handoff routes, sole producer and handoff, exact constructor, and unchanged
grouped-owner and writer fingerprints. The derive-only product change preserves
all emitted instructions and ordering. At the shared Batch AI checkpoint,
`cargo xc` exits zero, `array_buffer_slice_copy_policy_structure` passes `6/6`,
and the exact
`binary_data::run_wasm_backend_reobserves_arraybuffer_slice_source_after_observable_work`
CLI witness passes `1/1`. The exact pinned
`built-ins/ArrayBuffer/prototype/slice/species.js`,
`built-ins/ArrayBuffer/prototype/slice/species-returns-larger-arraybuffer.js`
and `built-ins/SharedArrayBuffer/prototype/slice/species.js` leaves pass all
`6/6` sloppy/strict Wasm-AOT executions with every failure bucket at zero. No
semantic golden was needed or run for Batch AI. Final formatter, diff,
module-boundary, task-plan and 240-entry shortcut-inventory gates are green.

The attribute-excluding carrier declaration remains
`c27d446dd7c67d0222a3d8e3bff7517b8ee65aa9adbedd64a77ad7217d839355`,
its constructor remains
`8e40b3be759a28a4f2240a79c86ed83a281d6ccf249aefb0442f9dbb18454e4f`,
and the raw writer body remains
`b95a56d5e6b021795271d5f61cf0fa05acea24c7c5b0802aabccaa3372eb6f7b`.
The normalized grouped-owner and writer fingerprints remain
`(14341, 0xd07f66f964485b66)` and `(7153, 0x32291bb08809c608)`.

Batch AJ makes `ArrayBufferSliceKind` a single capability-free slice-kind
authority. Its five production type mentions are the private declaration and
implementation plus the three grouped ordinary, shared and immutable slice
producers. One owned `slice_kind` selection has six borrowed projections
covering copy policy, species use, default result prototype, brand and flags,
and immutable-species rejection. Clone, copy, debug, default,
comparison, hashing and ordering capabilities are absent, so the operation
choice cannot be forked or retained through an incidental copy.

The strengthened seven-test recursive guard pins the three producers, exact
type and authority censuses, all six exhaustive mappings, and the absence of
alias, clone, dereference and mutable-borrow routes. The attribute-excluding
domain remains
`1860871f6edaec4bf2afd40c0a737ae469e58faeef6b5895c5a51e6e49aad664`.
The borrowed implementation is
`f8d8a88fcfd4720095628c1f95c978f8f48e2881586621537b2fc0cbf45dc3b9`;
its whitespace-normalized form is
`484c872e14c67c4834faae4a5e1778f651eacb3a5f9bc1a4fe233b647ca0ef1e`,
and removing only the six borrow markers preserves the pre-AJ semantic hash
`c5d2ec645ff3c40ea4a20971528ebcb8a099ba3f09efbd48bf9c0fd150452392`
and guard fingerprint `(1179, 0x21c812f9ad84ac3e)`. The grouped owner remains
`(14341, 0xd07f66f964485b66)`. No emitted instruction or observable ordering
changes. Shared `cargo xc` passes, the structure target passes `7/7`, both
exact ArrayBuffer CLI witnesses pass `2/2`, and the ordinary, immutable-result
and shared species leaves pass all `6/6` sloppy/strict Wasm-AOT executions with
every failure bucket at zero. No semantic golden was needed or run.

Batch AH makes the neighboring
`ArrayBufferSliceBound::{Start, End}` role capability-free. The grouped slice
owner remains the complete two-producer set. Each producer moves one selection
into the bound normalizer, which borrows that same authority through the
exhaustive argument-index and missing-or-undefined-default projections. Clone,
copy, debug, default, comparison, ordering and hashing capabilities are absent,
so the argument position and its paired default cannot be selected from
independent copies.

The strengthened guard pins eight exact product mentions, two producers, two
borrowed projections and the one-definition/two-call helper census. The
borrowed implementation is
`97ce3ae5aa8c7de1615d675b4836107a3e77cd7e74915eb68f2348bf3d9cf69b`,
the borrowed helper is
`f5c68fdc3acc539e902205d7991db025c8bfc5015863f6a87bbe91e9d6534766`,
and the grouped standard producer body remains
`9473147f5242fa296038457091c82408b1bfb7bbea1b5a90bd7a08ecafde7599`.
At the shared Batch AH checkpoint, `cargo xc` exits zero,
`array_buffer_slice_bound_structure` passes `3/3`, and the exact
`binary_data::run_wasm_backend_succeeds_for_supported_arraybuffer_slice_species_capture_fixture`
CLI witness passes `1/1`. The pinned
`built-ins/ArrayBuffer/prototype/slice/start-default-if-undefined.js` and
`built-ins/ArrayBuffer/prototype/slice/end-default-if-absent.js` leaves pass all
`4/4` sloppy/strict Wasm-AOT executions. Batch AH did not run a semantic golden.
Final formatter, diff, module-boundary, task-plan and 240-entry
shortcut-inventory gates are green. The bounded evidence remains in
`docs/rust-rewrite/contracts/array-buffer-slice-source-reobservation.md`.

The three `%TypedArray%.prototype` view accessors now share the same live
buffer-witness seam as the migrated Array/TypedArray consumers. A closed
`TypedArrayAccessorKind` makes `byteLength`, `byteOffset`, and `length` explicit
projections; each builtin delegates with one variant, and the accessor compiler
cannot directly read backing length, data, or the length-tracking slot. The
single witness therefore owns detached/out-of-bounds zeroing, fixed-view
regrowth, and whole-element flooring for odd-byte length-tracking buffers.
The focused
[accessor buffer-witness contract](../docs/rust-rewrite/contracts/typed-array-accessor-buffer-witness.md)
and existing accessor fixture pin those rules. This closes the accessor
duplication, not the older shared indexed `Get`, constructor, or
remaining binary-data consumers, and it does not retire a Test262 rewrite.

TypedArray iterator creation and stepping now use that same live buffer witness
instead of reconstructing private view slots through the older raw validator.
Both boundaries select the closed `ValidatedMethodEntry` projection: creation
consumes validation, while `next` consumes the length derived from the one
cached backing-store observation. Detached and out-of-bounds errors route
through the current function Realm, including created-Realm TypedArray methods
and their Realm-owned `%ArrayIteratorPrototype%.next`. The focused
[iterator buffer-witness contract](../docs/rust-rewrite/contracts/typed-array-iterator-buffer-witness.md)
and existing iterator fixture pin Realm identity, detach/shrink timing, current
resizable length, whole-element flooring and permanently-done behavior. Its
foreign buffers borrow the entry Realm's `resize`, so the proof does not claim
complete created-Realm ArrayBuffer prototype bootstrap. The focused structure
and CLI fixture pass on the current working tree. The remaining raw TypedArray
validators and full integer-indexed/iterator closure remain open; this does not
claim a new Test262 baseline pass.

`%TypedArray%.prototype.join` now uses the validated-method-entry projection of
that same buffer witness. Its compiler performs the receiver-brand check first,
loads one immutable view record, and consumes the witness's element length
directly instead of reconstructing private slots, calling the legacy raw
validator and dividing byte length itself. Detached and out-of-bounds failures
therefore use the executing builtin's Realm, including when a created Realm's
`join` is borrowed onto an entry-Realm receiver. Separator coercion remains
after the initially captured length, and later integer-indexed reads remain
live. The focused
[join buffer-witness contract](../docs/rust-rewrite/contracts/typed-array-join-buffer-witness.md)
and CLI fixture pin Realm identity, fixed and tracking resize behavior, BigInt,
and whole-element flooring. Created-Realm `join` is installed through the
self-backed TypedArray method table; the foreign buffer borrows the entry
Realm's `resize`, so complete created-Realm ArrayBuffer surface parity remains
open. The focused structure and CLI fixture pass on the current working tree.
Remaining raw validators, the shared indexed `Get`, Test262 rewrites and full
binary-data closure remain separate work.

The `%TypedArray%.prototype.reverse` and `toReversed` compilers now use the
same validated-method-entry buffer witness. Each method brand-checks its
receiver, loads one immutable `TypedArrayViewLocals` record and consumes the
element length produced by one `ValidatedMethodEntry` projection instead of
calling the legacy raw validator and dividing byte length locally.
`toReversed` retains its separate element-kind load and intrinsic same-kind
allocation; both reversal loops and their indexed read/write order are
unchanged. The focused
[reverse-family buffer-witness contract](../docs/rust-rewrite/contracts/typed-array-reverse-family-buffer-witness.md)
and bounded source-structure regression record that ownership. Under the shared
eight-core cap, `cargo xc` is green; the structural witness and the exact
`reverse` and `toReversed` CLI fixtures each pass `1/1`. The pinned
`reverse/resizable-buffer.js` and `toReversed/reverses.js` Test262 leaves each
pass `2/2` Wasm-AOT executions with every non-success bucket at zero.

The `%TypedArray%.prototype.sort` and `toSorted` compilers now carry the same
validated-method-entry ownership. Comparator admissibility remains before the
receiver check, and each compiler completes that brand guard before loading one
immutable `TypedArrayViewLocals` record and consuming one
`ValidatedMethodEntry` witness. Both retain one separate element-kind load.
`sort` still targets and returns its receiver; `toSorted` still performs
same-kind allocation, copies the complete captured range before sorting the
distinct result and returns that result. The shared stable-sort emitter is
unchanged. The focused
[sort-family buffer-witness contract](../docs/rust-rewrite/contracts/typed-array-sort-family-buffer-witness.md)
and bounded source-structure regression record those invariants. The
implementation and guard are independently reviewed. Under the shared
eight-core cap, `cargo xc` is green, the structural guard passes `1/1`, and the
exact `sort` and `toSorted` CLI fixtures each pass `1/1`. The pinned
`sort/return-abrupt-from-this-out-of-bounds.js` and
`toSorted/length-property-ignored.js` leaves each pass `2/2` Wasm-AOT
executions with all non-success buckets at zero under `--jobs 1 --threads 1`.
The fixtures now separately preserve their own `length = 50` shadow and check
the six integer-indexed elements, removing a contradictory assertion found by
the focused run. No aggregate or published conformance-count change is claimed.

The four `%TypedArray%.prototype` find-family methods now have the same written
method-entry ownership. Their shared `FindViaPredicateKind` compiler completes
the receiver-brand check, loads one immutable `TypedArrayViewLocals` record and
consumes one `ValidatedMethodEntry` witness before predicate validation. That
witness produces the single snapshot length used by all four direction and
value/index projections; later indexed reads, Proxy-aware predicate calls,
abrupt routing and result policies remain in the existing shared algorithm. The
focused
[find-family buffer-witness contract](../docs/rust-rewrite/contracts/typed-array-find-family-buffer-witness.md)
and hardened bounded source-structure regression record those invariants and
reject the raw validator, private-slot reconstruction, parallel backing-store
observation and local byte-length division. The guard also fixes all eight
Array/TypedArray builtin-to-kind mappings, the single brand-error owner, exact
callback receiver/argument wiring, and the live-read, abrupt-propagation,
truthiness and projection sequence. The implementation and guard are written
and independently reviewed. Under the shared eight-core cap, `cargo xc` is
green, the structural guard passes `4/4`, the exact
`wasm_typedarray_find.js` CLI fixture passes `1/1`, and the current-pin
`find/return-abrupt-from-this-out-of-bounds.js` and
`findLastIndex/detached-buffer.js` leaves each pass `2/2` Wasm-AOT executions
with `--jobs 1 --threads 1`. No new-pass, baseline or published-count change is
claimed.

The `%TypedArray%.prototype.every` and `some` quantifier family now uses one
validated-method-entry witness after its receiver-brand check and before callback
validation. The shared compiler consumes the witness-produced snapshot length
without a raw validator, private-slot reconstruction or local byte-length
division, while retaining live indexed reads, callback ordering and the closed
`Every`/`Some` short-circuit polarities. The focused
[quantifier-family buffer-witness contract](../docs/rust-rewrite/contracts/typed-array-quantifier-family-buffer-witness.md)
and `3/3` structural guard are implemented, independently reviewed and
focused-verified as of 2026-08-23. Under the shared eight-core cap,
`cargo fmt --all -- --check` and `cargo xc` are green; the exact
`wasm_typedarray_every_some.js` CLI fixture passes `1/1`, and the exact
current-pin `every/return-abrupt-from-this-out-of-bounds.js` and
`some/detached-buffer.js` Test262 leaves each pass `2/2`, for `4/4` Wasm-AOT
executions with all failure buckets at zero under `--jobs 1 --threads 1`.

The generic Array quantifier emitters no longer retain a second unreachable
strict TypedArray entry mode. `ArrayPrototypeEvery` and `ArrayPrototypeSome`
have argument-free generic compiler entries and preserve one fresh
`IntegerIndexedProperty` witness each for borrowed TypedArrays, while the two
strict TypedArray producers remain exclusively mapped to
`TypedArrayQuantifierKind::{Every, Some}`. The separate
[Array quantifier entry contract](../docs/rust-rewrite/contracts/array-quantifier-entry-boundary.md)
pins this four-producer boundary. The bounded structure target passes `4/4`,
and the six existing focused Array/TypedArray CLI fixtures pass `6/6`. `cargo
xc` is green. The following 669-dump semantic golden passes `2/2` in 771.49
seconds, adds only the independent Temporal arithmetic witness and removes
none. After accounting normalization, 667 of 668 retained dumps are equal; the
sole retained structural change is the independent Promise callback Realm
witness. No Test262 baseline or published-count change is claimed.

The direct `%TypedArray%.prototype.toLocaleString` entry now uses the same
validated-method-entry witness after its receiver-brand check. One cached
backing-store observation supplies the captured loop length, while the shared
loop retains live per-index reads; the generic
`Array.prototype.toLocaleString` branch keeps its distinct non-throwing
`LengthOfArrayLike` policy. The focused
[toLocaleString buffer-witness contract](../docs/rust-rewrite/contracts/typed-array-to-locale-string-buffer-witness.md),
companion invocation guard and bounded witness guard are implemented,
independently reviewed and focused-verified at the 2026-08-23 direct-entry
checkpoint. Under the shared eight-core cap at that checkpoint,
`cargo fmt --all -- --check`, `cargo xc` and `git diff --check` were green; the
companion invocation structure suite passed `4/4`, the then-three-test witness
structure target passed `3/3`, and the then-current core and invocation CLI
fixtures each passed `1/1`. The pinned out-of-bounds, detached-buffer,
mid-invocation growth and mid-invocation shrink Test262 leaves each passed
`2/2`, for `8/8` Wasm-AOT executions with all failure buckets at zero under
`--jobs 1 --threads 1`. The later generic companion migration expands the
shared witness target to four tests and changes the core fixture. Its separate
2026-08-24 checkpoint below supersedes the historical `3/3` and core `1/1`
results for those changed artifacts.

The generic `Array.prototype.toLocaleString` TypedArray length specialization
now expresses that distinct policy through one non-throwing
`ArrayLikeLengthSnapshot` witness. The ArrayLike arm loads one immutable view
and consumes the witness-produced element length instead of calling the raw
current-byte-length emitter and dividing locally. Detached and initially
out-of-bounds views therefore retain their zero-length result, while an
available length-tracking view floors its current byte extent to whole
elements. The captured loop bound and the downstream live integer-indexed reads
remain separate. The focused
[generic Array toLocaleString TypedArray buffer-witness contract](../docs/rust-rewrite/contracts/array-to-locale-string-typed-array-buffer-witness.md),
bounded shared-owner guard and strengthened core fixture are focused-verified:
the shared witness and companion invocation structure targets pass `4/4` each,
and the strengthened core fixture passes `1/1`. At that feature checkpoint, the
exact three Array Test262 leaves passed all six ordinary sloppy/strict variants
through a Uint8-only materializer. The later source-retirement checkpoint above
supersedes that harness claim with the raw all-constructor and BigInt matrix,
deletes the materializer and records its source-ledger change. It does not claim
a rebuilt post-delete production run. This migration also does not correct own
or inherited TypedArray `length` shadowing in the generic `LengthOfArrayLike` path
or migrate the downstream indexed-read owner. It did not itself change either
`flatMap` raw observation or any `objects.rs` raw consumer; the later flatMap
checkpoint below supersedes that former two-site nonclaim, and the later
property/index checkpoint supersedes the former three-site raw census.

The generic `Array.prototype.flatMap` TypedArray specialization now consumes
the same non-throwing witness protocol at both of its formerly raw observation
points. One immutable `TypedArrayViewLocals` feeds an
`ArrayLikeLengthSnapshot` before target allocation and a fresh
`IntegerIndexedProperty` projection inside the captured-length loop. Growth
during a mapper call cannot extend the walk; shrinkage, odd-byte availability,
out-of-bounds state or detachment can make the next index absent. The existing
live indexed `Get` and mapper ordering remain unchanged. The focused
[flatMap TypedArray buffer-witness contract](../docs/rust-rewrite/contracts/array-flat-map-typed-array-buffer-witness.md),
bounded structure target and resizable/detached CLI fixture are written as of
2026-08-24. The guard also pins zero legacy raw current-length calls across all
of `builtins/array.rs`.

At the 2026-08-24 central checkpoint, `cargo check` and `cargo xc` were green,
the structure target passed `3/3`, the exact CLI fixture passed `1/1`, and the
exact direct pinned cohort is the unrewritten vendored
`built-ins/Array/prototype/flatMap/array-like-objects-typedarrays.js` leaf,
materialized with the normal harness preludes into two ordinary sloppy/strict
executions. Both passed `2/2` with every failure bucket at zero. The leaf covers
only fixed `Int32Array` borrowing, so the resizable, odd-byte and detached
evidence remains confined to the focused CLI fixture. No all-constructor,
BigInt, baseline, README or published-count change is claimed. The later
property/index checkpoint below closes the shared indexed-read and remaining
raw `objects.rs` observations.

The three central TypedArray property/index owners in `objects.rs` now consume
the same closed buffer-witness protocol. The specialized `length` read selects
`Accessor::Length`; the shared indexed read and write select
`IntegerIndexedProperty`. All three load one immutable
`TypedArrayViewLocals`, and the retired raw current-byte-length emitter no
longer exists. Integer-indexed reads and writes now floor length-tracking views
to whole elements, so a trailing partial Uint16 element is absent. Writes keep
element-kind selection and observable value coercion before their fresh
witness, then load the usable backing pointer only after the witness accepts
the index. A `valueOf` resize or detachment is therefore reflected by the
write, without reusing a pre-coercion pointer.

The bounded property/index structure target passes `5/5`. The new combined
odd-byte, growth, shrink and detachment CLI fixture passes `1/1`; the existing
accessor, canonical numeric indexed-read and integer-indexed write fixtures
each pass their exact `1/1` targets. The fixture parses under `node --check`.
No broad suite, post-change golden capture, Test262 cohort, baseline, README
count or published-status change is claimed. This migration does not correct
the older specialized `length` shortcut's own/inherited property-shadowing
debt.

Constructed TypedArray species targets now pass through the same immutable view
and `ValidatedMethodEntry` witness before their capacity is compared with the
requested element count. The shared validator consumes the witness-produced
element length directly instead of invoking the legacy raw validator and
dividing current byte length locally. Detached and out-of-bounds targets now
throw from the executing builtin's Realm; the focused constructed-target
fixture pins detached and out-of-bounds results for a created-Realm method
borrowed onto entry-Realm sources.
The bounded constructed-target structure target passes `2/2`, and its exact
CLI fixture passes `1/1`. No broad suite, golden capture, Test262 cohort,
baseline or published-status change is claimed; five legacy raw-validator
calls remain in `builtins/standard.rs`.

The `%TypedArray%.prototype.map` and `filter` compilers now use that same
validated-method-entry witness after their receiver-brand guards and before
callback validation. Each loads one immutable `TypedArrayViewLocals` record
and consumes its witness-produced snapshot length without a raw validator,
private-slot reconstruction or local byte-length division. The migration keeps
the algorithms' distinct allocation order: `map` performs species construction
before its callback loop, while `filter` completes callback collection before
species construction and selected-value writes. Live per-index reads and the
existing callback `(value, index, receiver)` wiring remain unchanged.

The focused
[map/filter buffer-witness contract](../docs/rust-rewrite/contracts/typed-array-map-filter-buffer-witness.md)
and bounded source guard are implemented, independently reviewed and
focused-verified as of 2026-08-23. Under the shared eight-core, 22 GB cap,
`cargo fmt --all -- --check`, `cargo xc` and `git diff --check` are green; the
structural guard passes `3/3`, and the exact `map` and `filter` CLI fixtures
each pass `1/1`, including detached and out-of-bounds entry controls that prove
the callback is not called. The eight pinned detached, out-of-bounds, growth
and shrink Test262 leaves each pass `2/2`, for `16/16` Wasm-AOT executions with
every failure bucket at zero under `--jobs 1 --threads 1`.

`%TypedArray%.prototype.copyWithin` now uses one immutable
`TypedArrayViewLocals` record and exactly two validated-method-entry witnesses:
the entry witness captures the range before coercion, while a second witness
inside the positive-count branch reobserves the buffer after target, start and
end coercion. The typed seam preserves fixed-view extent, whole-element
flooring, current-length truncation and the zero-count rule that skips the
second observation and all copy setup. Its structural guard pins coercion and
clamping order, both length snapshots, branch containment, current-length
caps, overlap direction and the byte-copy loop.

The implementation and guard were independently reviewed and focused-verified
on 2026-08-23. Under the shared eight-core, 22 GB cap, the structure suite
passes `3/3`, the exact CLI fixture passes `1/1`, and the six exact Test262
leaves pass `12/12` Wasm-AOT variants with every failure bucket at zero under
`--jobs 1 --threads 1`. The source of truth is
`docs/rust-rewrite/contracts/typed-array-copy-within-buffer-witness.md`.

`%TypedArray%.prototype.slice` now loads one immutable source view and consumes
an entry witness plus a conditional post-species witness. The late observation
caps copying after shrinkage while leaving the originally constructed target
length intact, preserves whole-element flooring and stays after target
validation and content-type checks. Its implementation, durable guard and
expanded CLI fixture are focused-verified on 2026-08-24: the structure target
passes `6/6`, the exact CLI fixture passes `1/1`, and the seven pinned leaves
pass all `14/14` Wasm-AOT variants with every failure bucket at zero. The source
of truth is
`docs/rust-rewrite/contracts/typed-array-slice-buffer-witness.md`.

The four Wasm-AOT Atomics access owners now load one immutable
`TypedArrayViewLocals` value and consume one `ValidatedMethodEntry` witness
before index coercion. `notify`, `waitAsync`, `wait` and the shared integer-
operation compiler use the witness-produced element length directly for their
post-`ToIndex` bound; they no longer reconstruct current byte length or admit a
trailing partial element. The validated projection also intentionally corrects
the old fixed-view behavior: an initially detached or out-of-bounds view throws
TypeError before a side-effecting index is coerced, while a valid zero-length
tracking view still coerces the index and then throws the operation-specific
RangeError against the captured length. The pre-coercion backing-pointer
snapshot remains separate for address formation, preserving the current
Atomics pointer timing. The focused
[Atomics buffer-witness contract](../docs/rust-rewrite/contracts/atomics-typed-array-buffer-witness.md),
bounded four-owner structural guard and CLI fixture are focused-verified. The
guard passes `3/3`, the CLI fixture passes `1/1`, and the four exact pinned
Test262 files pass `8/8` Wasm-AOT variants with every non-success bucket at
zero. This does not implement post-coercion `RevalidateAtomicAccess` or claim
complete Atomics semantics.

The shared TypedArray HasProperty predicate used by `Array.prototype.concat`
and the TypedArray receiver branch of `Array.prototype.slice` now consumes the
closed `IntegerIndexedProperty` witness. It keeps its non-throwing result
policy: detached, fixed/tracking out-of-bounds and index-at-or-above-current-
length states are absent, while fixed-view regrowth restores the stored index
extent. The witness also floors odd available byte lengths before comparing an
index, so concat cannot create an own `undefined` target property for a
trailing partial element that should remain a hole. Non-TypedArray receivers
still select the ordinary-object fallback through the separate classification
output. The focused
[concat TypedArray buffer-witness contract](../docs/rust-rewrite/contracts/array-concat-typed-array-buffer-witness.md),
bounded predicate/caller guard and CLI fixture are focused-verified: the
structure target passes `3/3`, the CLI fixture passes `1/1`, and the concat plus
Array-slice Test262 controls pass `4/4` Wasm-AOT variants with every non-success
bucket at zero. This closes one shared raw HasProperty owner, not concat, Array
slice or integer-indexed exotic semantics as a whole.

The direct TypedArray branch of `Object.getOwnPropertyNames` now loads one
immutable `TypedArrayViewLocals` value and consumes one non-throwing
`ArrayLikeLengthSnapshot` witness. The witness-produced element length owns the
ascending integer-key prefix, including detached/out-of-bounds zeroing,
fixed-view regrowth and whole-element flooring for odd-byte length-tracking
buffers. Ordinary String keys remain after that prefix and Symbol keys remain
excluded. Proxy `ownKeys` dispatch and every non-TypedArray fallback keep their
existing order and behavior. The focused
[`Object.getOwnPropertyNames` TypedArray buffer-witness contract](../docs/rust-rewrite/contracts/object-get-own-property-names-typed-array-buffer-witness.md),
bounded owner guard and CLI fixture are focused-verified: `cargo xc` is green,
the structure target passes `3/3`, and the exact CLI fixture passes `1/1`. The
pinned suite has no direct
`Object.getOwnPropertyNames` TypedArray leaf, so the contract inventories the
two smallest adjacent resizable-buffer `[[OwnPropertyKeys]]` controls. They
pass all `4/4` Wasm-AOT variants with every non-success bucket at zero, while
remaining adjacent rather than direct evidence for this compiler.

`%TypedArray%.prototype.subarray` now loads one immutable
`TypedArrayViewLocals` and consumes the non-throwing
`ArrayLikeLengthSnapshot` projection. Detached and initially out-of-bounds
sources therefore contribute a zero source-length snapshot without skipping
begin/end coercion or species construction. An explicitly selected constructor
still owns any later detached-buffer error and its Realm, and a custom species may
return a compatible in-bounds result. The compiler retains the stored source
byte offset, floors available bytes to whole elements, selects the source
element kind for the intrinsic default constructor, and keeps the normative
two-argument result construction only when the source is length-tracking and
`end` is omitted.
After species construction and the result brand check, the arm loads a distinct
immutable result view and consumes exactly one `ValidatedMethodEntry`
projection before content-type acceptance. A species-returned detached or
currently out-of-bounds TypedArray therefore throws a TypeError from the
executing builtin's Realm rather than being published. The focused
[subarray buffer-witness contract](../docs/rust-rewrite/contracts/typed-array-subarray-buffer-witness.md),
bounded owner guard and CLI fixture retain the source-witness checkpoint's
focused evidence. At the 2026-08-25 coordinated checkpoint, `cargo xc` is
green, the updated structure target passes `3/3`, the extended exact CLI
fixture passes `1/1`, and the six direct Test262 leaves pass all `12/12`
Wasm-AOT variants at vendored suite content tree
`aa55200d1310384c5cf69ea95b2a2ecba457007b`, with every failure and
non-success bucket at zero. The first fixture run exposed a missing
created-Realm `subarray` materialization; adding the method to that Realm's
TypedArray inventory made the borrowed builtin own the invalid-result
TypeError as required. The pinned suite has no direct leaf for a
species-returned detached or out-of-bounds result.

The adjacent custom-species-constructor invocation failures have been isolated
to the separate
[subarray species argument-vector arity contract](../docs/rust-rewrite/contracts/typed-array-subarray-species-argument-arity.md).
The pre-fix arm first built `(buffer, beginByteOffset, newLength)`, which set
both the call count and heap-visible argv length to three. For a
length-tracking source with omitted `end`, it then reduced only the call count
to two. A custom constructor therefore received two formal arguments while its
escaped `arguments` object read the stale vector length and exposed a phantom
third entry. The bounded correction now selects an exclusive two- or
three-entry vector before the one shared construct, keeping both arity carriers
coherent without changing arguments-object construction.

At vendored suite content tree
`aa55200d1310384c5cf69ea95b2a2ecba457007b`, the exact Number and BigInt
`speciesctor-get-species-custom-ctor-invocation.js` files expand to four sloppy
and strict Wasm-AOT variants. The pre-fix result is `0/4` `Runtime/Bug`: both
sloppy variants throw `Constructor called with arguments`, and both strict
variants reach Boa's `Cannot assign to property` TypeError. Post-fix, the
expanded structure target passes `4/4`, the existing exact subarray CLI fixture
passes `1/1`, and the raw Number and BigInt leaves pass `2/2` each with every
failure and non-success bucket at zero. This `0/4` to `4/4` transition is
focused arity evidence and does not alter the earlier `12/12` source-witness
pass claim.

`%TypedArray%.prototype.with` now captures its entry length through one
immutable `TypedArrayViewLocals` record and one `ValidatedMethodEntry` witness.
Both arguments remain acquired before the receiver brand check; the witness
still precedes index and replacement-value coercion, and the existing fresh
integer-indexed observation remains after those coercions. Its invalid-index
RangeError now comes from the executing method's Realm. The focused fixture
pins that route with a created-Realm method borrowed onto an entry-Realm view,
the entry-builtin error Realm for detached and out-of-bounds receivers, no
coercion for an invalid entry witness, and whole-element flooring for an
odd-byte length-tracking Uint16 view. The bounded structure target passes
`2/2`, and the exact CLI fixture passes `1/1`. No broad suite, golden capture,
Test262 cohort, baseline or published-status change is claimed.

`%TypedArray%.prototype.set` now uses one immutable receiver view for two
`ValidatedMethodEntry` observations: entry validation occurs before offset
coercion, then a fresh witness replaces the receiver length after that
observable coercion. The TypedArray-source arm loads its own immutable view and
consumes a third validated witness before content-type and capacity checks.
The overlap path still snapshots every source value into temporary storage
before its first target write. The focused fixture covers entry detachment,
post-offset growth, shrinkage, detachment and fixed-view out-of-bounds state,
detached and out-of-bounds TypedArray sources, and odd-byte Uint16 source
flooring. All four direct capacity failures now construct RangeError through
the executing method's Realm; the focused fixture exercises both capacity
predicates for TypedArray and array-like sources with a created-Realm method
borrowed onto entry-Realm targets. The bounded structure target passes `3/3`,
the new exact CLI fixture passes `1/1`, and the retained overlap/array-like
fixture passes `1/1`. No broad suite, golden capture, Test262 cohort, baseline
or published-status change is claimed.

The shared `ToIndex` normalization emitter now routes both of its own
RangeErrors through the existing closed `NumericErrorRealmSource` projection.
Standard builtins use their self-backed current-function Realm, numeric helper
bodies retain their typed Realm argument, and main, user, host and ordinary
helper bodies retain the main-Realm fallback without interpreting a lexical
environment as Realm storage. A bounded structure target pins both the
non-finite/outside-safe-integer and negative-result throw sites to that one
projection and passes `3/3`; the projection unit passes `1/1`, and the existing
TypedArray set CLI target passes `1/1` with a created-Realm method borrowed onto
an entry-Realm target and a negative offset. No broad suite, golden capture,
Test262 cohort, baseline or published-status change is claimed.

The four Atomics access owners now route their positive post-`ToIndex` index
bounds through the executing builtin's Realm while preserving their captured
entry-witness lengths and operation-specific messages. The bounded
`notify`/`waitAsync`/`wait`/shared-integer-operation source guard pins exactly
one current-function-Realm RangeError in each owner and forbids the generic
entry-global RangeError route. All 16 algorithm-created TypeErrors across
`pause`, `notify`, `waitAsync`, `wait`, its suspension check and the shared
integer-operation compiler now follow the same executing-builtin Realm rule.
The six-region guard pins the closed `1/3/1/4/4/3` TypeError census and forbids
the generic route. The expanded structure target passes `5/5`; the retained
Atomics buffer-witness fixture and the representative entry-Realm TypeError
fixture each pass `1/1`. Created realms now publish an ordinary `Atomics`
namespace with the complete currently implemented 14-method surface and exact
global, method and `@@toStringTag` descriptors. One closed
`ATOMICS_PUBLICATION_ORDER: [StandardBuiltinId; 14]` drives both main- and
created-Realm publication directly; catalog native names cannot drift between
the two paths. The emitter's private, non-derived `AtomicsBuiltin` selection
domain is separately reachable only through fourteen fixed family entries, so
neither publication owner nor the shared catalog dispatcher can construct or
forward raw Atomics policy. Reconstructing the former declaration and selector
produces the exact original 39-line selection with SHA-256
`3382f4b6d98ca6acfb04ad9c9f452bd1f93bf65f9d3334e0cef0f17583366231`.
The strengthened
[`Atomics dispatch contract`](../docs/rust-rewrite/contracts/atomics-builtin-dispatch-boundary.md)
target passes `5/5`; seven neighboring Atomics structure targets pass `27/27`,
and the exact entry- and created-Realm controls each pass `1/1`. This is
source-equivalent hardening with no new Atomics behavior or T17 closure.
Created-Realm functions are
fresh, inherit that Realm's Function prototype, self-back their environment and
capture its TypeError and RangeError prototypes. The bounded publication target
passes `3/3`; its focused borrowed-`add` fixture passes `1/1`, covering all
method identities/names/lengths/descriptors and defining-Realm TypeError and
RangeError behavior without invoking `wait` or `waitAsync`. Both entry- and
created-Realm Atomics functions are now self-backed so borrowed `waitAsync`
recovers its defining Realm without dynamic current-Realm state. One private,
non-copyable proof resolves the required Object prototype for both result
wrappers. The async emitter separately acquires the opaque intrinsic Promise
allocation context from the same function Realm, trapping on missing Realm
intrinsics.
Both result shapes define enumerable `async` then `value` CreateDataProperties
with writable/configurable attributes. The bounded source target passes `4/4`,
and a separate non-blocking fixture passes `1/1`; together they cover not-
equal, timeout-zero and immediate-notify async branches, wrapper/Promise
prototypes, descriptor flags, key order and resolved `"ok"` value. The retained
entry-Realm waitAsync core fixture also passes its exact `1/1` regression. The
latest shared semantic golden passes `2/2` in 722.99 seconds and contains 678
fixture dumps. This source-only proof closure adds no fixture; the checkpoint
adds the four independent Array.fromAsync callback-Realm, Object-policy,
Promise-mode and Set-domain witnesses, removes none and leaves all 674 retained
dumps equal after accounting normalization.

DataView's ten getter/setter owner groups and all eight direct constructor
offset/length RangeError branches now use the executing builtin's Realm. The
constructor's created-Realm function header retains its own environment handle
and TypeError/RangeError prototypes, so both initial normalization/capacity failures and
post-prototype backing-store revalidation failures preserve constructor Realm
identity without changing conversion, prototype-observation or resize order.
The three direct constructor TypeErrors and the three-route current-length
validator now use that same Realm authority. An exact 11-call-site source
census covers the grouped private-slot accessor and ten getter/setter owners,
representing 24 published callables. The bounded DataView source guard passes
`6/6`; its six-branch borrowed-
constructor CLI fixture passes `1/1`, and the retained constructor-ordering
fixture passes `1/1`. Created realms now publish the complete currently
implemented DataView prototype through one closed main-Realm-ordered plan: three
accessors, 22 numeric methods and `@@toStringTag`. Names remain owned by the
standard builtin catalog, while descriptor kind is a closed accessor/method
choice. Every callable is materialized in the created Realm, self-backed and
given that Realm's TypeError and RangeError prototypes before publication. The
bounded publication target passes `3/3`; its focused borrowed getter/setter
fixture passes `1/1`, including method identity/descriptors and both positive-
bound RangeError prototype checks. The borrowed TypeError fixture passes `1/1`
and covers invalid receivers, constructor invocation/buffer rejection, initial
and post-prototype detachment, out-of-bounds views and the associated coercion
ordering. The shared workspace check and repository policy gates pass. The
656-dump semantic golden adds the four focused Atomics/DataView Realm fixtures
with no removal; the 652 retained dumps preserve their structural fields apart
from the expected main-local/largest-function changes in two independently
expanded fixtures. No broad binary-data Test262 cohort, baseline or
published-status change is claimed.

Batch AK gives the created-Realm DataView prototype plan one move-only
publication lifecycle. Its twenty-six publication rows retain the exact
twenty-five-callable/one-tag order and three-accessor/twenty-two-method split.
The sole installer consumes each publication once, then carries a callable
row's one property role through two borrowed property-kind decisions for name
derivation and descriptor emission. Both private domains are capability-free,
so a publication or its accessor/method role cannot be forked through an
incidental copy.

The strengthened four-test publication guard pins both exact domains, their
three-mention censuses, all producer rows, one consuming publication match, two
borrowed property matches, the unchanged plan and borrow-normalized installer
fingerprints, and the existing main-Realm order/Realm-capture/CLI controls. The
focused
[created-Realm DataView publication lifecycle contract](../docs/rust-rewrite/contracts/created-realm-data-view-publication-lifecycle.md)
records the exact hashes and nonclaims. No emitted instruction or observable
ordering changes. Batch AK shared `cargo xc` is green, the structure target
passes `4/4`, and the exact created-Realm CLI fixture passes `1/1`. The exact
DataView accessor, method-name and `@@toStringTag` Test262 leaves pass all
`6/6` Wasm-AOT variants with every failure bucket at zero. No semantic golden
was required or run. Final formatter, diff, module-boundary, task-plan and
240-entry shortcut-inventory gates are green.

Batch AN replaces the four raw ArrayBuffer flag constants with the closed,
capability-free `ArrayBufferFlag::{Resizable, Shared, Immutable, Detached}`
wire vocabulary. One borrowed exhaustive projection owns the stable `1`, `2`,
`4` and `8` bits; the aggregate private flags field remains `u64` because those
properties are legally composable. The migration retargets exactly 25 product
projections and the heap test's four-bit valid mask without changing any
instruction, bitwise operation or observation order.

The recursive four-test guard pins one declaration and implementation, 29
named projections, zero raw flag constants, the exact `2/6/17/4`
objects/binary-data/standard/heap distribution, and the complete eight-owner
product census. The 25 legacy projection rows retain their exact normalized
fingerprint `(1773, 0xa28c775059daa571)`, raw SHA-256
`5d75104504642d0ff4e5e41dbfc02e253bae885b7b40b3e17fd92a708ed7d144`
and normalized SHA-256
`8b058a539e4e37d8ea53cb6a8054931e0810602a17cd49c83b8a1597aa3f4437`.
The focused
[ArrayBuffer flag wire-domain contract](../docs/rust-rewrite/contracts/array-buffer-flag-wire-domain.md)
records the exact boundary, witnesses and nonclaims. At the Batch AN
checkpoint, `cargo xc` is green, the structure target passes `4/4`, the exact
CLIs pass `3/3`, and all
`8/8` Wasm-AOT variants across the four pinned leaves with every failure bucket
at zero. No semantic golden was required or run. Final formatter, diff,
module-boundary, task-plan and 240-entry shortcut-inventory gates are green.

Batch AO replaces the six raw TypedArray fixed-versus-length-tracking constants
with the exclusive, capability-free
`TypedArrayLengthMode::{Fixed, Tracking}` wire authority. Its borrowed,
exhaustive word projection preserves `Fixed = 0` and `Tracking = 1`; runtime
locals and the private heap slot remain `u64`. Three writers and three readers
now select named modes with the exact `Fixed 5 / Tracking 1` split, while the
grouped eleven-constructor arm remains the sole publisher to
`HEAP_TYPED_ARRAY_LENGTH_TRACKING_OFFSET`.

The recursive four-test guard pins eight total type mentions, six named
projections, the exact `objects 1 / binary_data 3 / standard 2` distribution,
all reader/writer owners, and the unchanged seven-offset census. The legacy
six-row sequence retains raw fingerprint `(358, 0xc988361080d6b5cc)` and
normalized fingerprint `(288, 0x7e691158ebdeda94)`, with SHA-256 hashes
`9b36ddbd8cb543cd8e4780c84c458695e5fd846fe9593cd5da206274d13ebfba`
and
`04193b36264ff26e0780c45446bc517e85aef7b306fcf1d2fe0ede71994d6d4f`.
The focused
[TypedArray length-mode wire-domain contract](../docs/rust-rewrite/contracts/typed-array-length-mode-wire-domain.md)
records the exact boundary, controls and nonclaims. Shared `cargo xc`, the new
structure target and two exact CLI controls pass `4/4` and `2/2`. The two
prototype-length leaves pass `4/4` Wasm-AOT variants with every failure bucket
at zero. The length-tracking leaf's two variants stop at the existing declared
`resizable-arraybuffer` unsupported feature gate, matching the earlier
integer-index witness finding; no unsupported-retirement claim is made. No
semantic golden was required or run for Batch AO.

TypedArray construction from a TypedArray source now loads a distinct immutable
source view and consumes one `ValidatedMethodEntry` witness. Its one cached
backing-store observation supplies the source length snapshot before target
backing-store allocation and indexed copying, including whole-element flooring
for odd-byte length-tracking views. Detached and out-of-bounds sources now throw
the executing constructor's Realm-owned TypeError. Target element-kind
selection, Number/BigInt conversion checks, direct ArrayBuffer allocation,
source indexing, target prototype selection and the no-ArrayBuffer-species path
remain in their existing order. This removes the last production caller of the
legacy throwing current-byte-length validator; both raw TypedArray current-
length helpers are now deleted. The dominated no-`length` fallback no longer
rechecks the source brand or reconstructs element length from raw TypedArray
private offsets. The constructor guard pins the remaining private-offset uses
to target initialization and the remaining unsigned division to backing-memory
page sizing. The synthetic static-generator throw-slot probe is confined to the
non-TypedArray source branch, so constructing from a TypedArray cannot observe
an own or inherited `$LilaGeneratorThrow` property before its brand/witness
path. The bounded structure target passes `3/3`, the new exact CLI fixture
passes `1/1`, and the retained no-species constructor
fixture passes `1/1`. No broad suite, golden capture, Test262 cohort, baseline
or published-status change is claimed. The fixture covers the entry Realm; a
current unrelated created-Realm global-resolution failure prevents a direct
cross-Realm constructor control, so the executing-function-Realm route is
owned structurally rather than claimed as runtime evidence here.

These migrations still do not cover all constructor validation or other
binary-data observers. They do not change key
classification, caller-specific integer-indexed descriptor/result policy,
result allocation, SharedArrayBuffer synchronization, Test262 rewrites or
published counts. The toLocaleString, map/filter and copyWithin fixtures do not
prove created-Realm buffer-error prototype identity at direct method entry;
only the shared witness's current-function-Realm route is structurally owned
for that case.
`subarray` additionally retains one adjacent semantic debt: its nullish-species
default constructor comes from entry globals rather than the executing Realm.
The post-species validation and argument-vector arity lanes do not change
constructor selection, argument coercion, general arguments-object semantics,
result allocation, Test262 rewrites or published counts. The arity lane also
does not generalize argument-vector construction across unrelated call sites or
claim resizable-buffer growth, shrinkage, detachment or out-of-bounds behavior
beyond the already verified witness boundary.

## Objective

Implement the complete binary-data stack, integer-indexed exotic semantics and real agent/Atomics behavior. Replace rejection-only SharedArrayBuffer behavior and harness simulations with general backing-store and host concurrency support.

## Backing stores and ArrayBuffer

- Model detachable, resizable, growable/shared and fixed backing stores separately from view objects.
- Implement `ArrayBuffer` construction, `byteLength`, `maxByteLength`, `resizable`, `resize`, `slice`, `transfer`, `transferToFixedLength`, detachment and species behavior.
- Preserve backing-store identity across views and define safe host access during memory growth/detachment.
- Implement `SharedArrayBuffer` and growable shared buffers where present; they must not be detachable.

## DataView

Complete constructor validation and every get/set method, including:

- ToIndex/offset ordering;
- detached/out-of-bounds checks before and after observable coercion;
- endian handling;
- integer, Float16, Float32/64 and BigInt64/BigUint64 conversion;
- resizable/growable buffer behavior;
- realm/species/custom-new-target descriptors.

## Typed arrays

Implement all concrete typed-array constructors and `%TypedArray%` semantics:

- construction from length, buffer/offset/length, typed arrays and iterables/array-likes;
- integer-indexed exotic internal methods and canonical numeric index strings;
- fixed vs length-tracking views over resizable/growable buffers;
- BigInt/Number element-kind separation, clamping, Float16 and NaN/signed-zero rules;
- all static/prototype methods, iterators, species and subclassing;
- detachment/out-of-bounds validation at exact spec points;
- generic Array method borrowing where allowed and non-generic TypedArray methods where required.

## Atomics and agents

- Implement all Atomics operations with correct element-kind validation and sequentially consistent behavior required by ECMAScript.
- Provide host-managed shared backing stores and actual agent threads/workers for Test262.
- Implement wait queues, `wait`, `notify`, `waitAsync`, timeouts, `isLockFree`, blocking restrictions and monotonic timing.
- Integrate job completion for `waitAsync` with T14.
- Eliminate regex/source-pattern agent simulations from the embedded
  `lila-test262` local harness under T03.

### Resolved CLI hang and remaining concurrency debt

`binary_data::run_wasm_backend_succeeds_for_atomics_wait_core_fixture` used to
hang the CLI suite. The bounded known-failure machinery detected when it began
passing in batch 6, and its hang row, `should_panic` annotation and compile-time
ledger assertion were removed together. It is now an ordinary passing test and
the current CLI ledger contains no declared hang. The suite must run without an
`atomics_wait_core` skip.

That focused result proves only that the fixture's non-equal waits return; it
does not prove the real-agent acceptance criteria below. Host-managed agents,
wait queues, notifications, timeouts and `waitAsync` job integration remain
open until the real Test262 agent trees pass without source-pattern simulation.
The generic per-invocation timeout and watched-run safeguards remain useful for
detecting the next hang and are not evidence of an expected failure.

## Wasm/runtime strategy

The backend uses a hybrid design. Shared scalar memory operations use Wasm
shared memory and atomic instructions. Host-managed agent orchestration and
the cross-instance `waitAsync` waiter registry use the typed `agent_call`
import, because waiters and reports must cross independently instantiated Wasm
modules. The host operation is decoded into a closed Rust enum before semantic
dispatch; an unknown wire value is a visible host error. Both paths must still
preserve JavaScript object identity, detachment rules and agent
synchronization. Single-threaded scripted simulation is not concurrency
coverage.

## Acceptance criteria

- Complete pinned trees for ArrayBuffer, SharedArrayBuffer, DataView, TypedArray constructors/prototypes and Atomics are green.
- Integer-indexed exotic descriptor/key/proxy cases pass.
- Resizable/growable buffer tests pass before, during and after coercion/callback mutation.
- BigInt and Number typed arrays reject mixed values correctly.
- Real multi-agent wait/notify/report tests pass without source pattern matching.
- Detached/out-of-bounds checks occur at spec-required times.
- No data races or host panics under repeated agent stress tests.

## Required tests

```sh
cargo test -p lila-aot-wasm typed_array_ --quiet
cargo test -p lila-spec-exec agent_ --quiet
cargo test -p lila-test262 agent_ --quiet
cargo test -p lila-cli wasm_typed_array --quiet
./target/debug/lila test262 run built-ins/ArrayBuffer --execution-backend wasm --timeout-ms 180000 --threads 4
./target/debug/lila test262 run built-ins/TypedArray --execution-backend wasm --timeout-ms 180000 --threads 4
./target/debug/lila test262 run built-ins/Atomics --execution-backend wasm --timeout-ms 180000 --threads 2
```

Run DataView and every concrete typed-array subtree separately during implementation, then execute shared-buffer/agent tests under repeated stress.
