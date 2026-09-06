# T16 — Array exotic semantics and complete Array API

**Status:** In progress — many Array leaves are green; materialization-free full-tree closure remains

**Parallel group:** Feature lane  
**Depends on:** T04, T05, T10; iterator consumers use T15  
**Blocks:** Array-related T26 closure

## Current repository state

Array exotic storage, descriptors, species and most prototype families have
substantial implementations and many focused complete-leaf results recorded in
the README. `crates/lila-aot-wasm/src/builtins/array.rs` remains a very large
shared implementation file, and the Test262 harness still contains numerous
Array-specific path rewrites and source reductions. This task cannot close
until the full current-pin Array tree is green through general semantics.

The two `Array.prototype.flatMap` custom-species materializers have been
removed. Their unchanged pinned sources execute with the full assertion and
`propertyHelper.js` harness for all four sloppy/strict variants, covering the
custom result descriptors and poisoned-constructor abrupt completion. The
token-aware generated inventory assigns 24 T16 observations to the broader
Array work. The historical checkpoints below record each preceding reduction.

`Array.fromAsync` result publication now converts ordinary property-definition
and length-Set failures through one closed object-mutation Realm authority. The
three legal body sources project exhaustively to the executing builtin's trusted
environment or the main fallback; all eight descriptor rejection producers and
both `CreateDataPropertyOrThrow` rejection sites plus the ordinary
non-writable/non-extensible Set owners consume that projection.
Constructor `C` and result object `A` remain independent from the method Realm,
while setter and Proxy-trap throws preserve their original identity. The
non-extensible descriptor error retains its no-own-message shape. A bounded
structure target and finite cross-Realm fixture cover incompatible indices,
both length-publication routes and abrupt identity. Both structure targets pass
`4/4`, the focused CLI fixture passes `1/1`, and the six relevant pinned files
pass all twelve sloppy/strict Wasm-AOT executions. The isolated boundary is recorded in
[`array-from-async-result-definition-error-realm.md`](../docs/rust-rewrite/contracts/array-from-async-result-definition-error-realm.md).
The coordinated semantic golden passes `2/2` in 800.46 seconds with 679 dumps,
adds only this fixture, removes none and leaves 677 of 678 retained dumps equal
after accounting normalization. The independent expanded Promise witness is
the sole retained structural change. Complete Array.fromAsync Test262
verification remains deferred.

The 48 `prop-desc.js`, `length.js` and `name.js` sources for the 16 implemented
Array prototype methods now execute unchanged with the complete upstream
property helper in sloppy and strict modes (`96/96`). Ordinary materialization
uses the complete LocalMerged assertion/property sections, while a separate
route pins the complete vendored helper and its provenance. A real-source 16x3
matrix pins every source body and supported feature boundary. Deleting the
metadata dispatcher and three path predicates removes four T16 semantic
observations.

The pinned `Array.prototype.at` `coerced-index-resize.js` and
`typed-array-resizable-buffer.js` bodies now use ordinary materialization in
both Script modes. A direct raw preflight passed all `4/4` Wasm-AOT executions
after streaming complete vendored `sta.js`, `assert.js`,
`resizableArrayBufferUtils.js` with only T13's existing replacement of the
dynamic subclass block with three static classes, and the exact test body. The
full unmodified helper still reports the explicit Function-constructor
AOT-unsupported diagnostic in both modes. The preflight therefore proves
unchanged-source readiness before deletion; it does not prove full-helper
support or post-delete production dispatch, and T13 keeps ownership of that
substitution. The 2x2 real-source invariant pins exact modes, source bytes,
includes, supported-feature and no-rewrite boundaries, plus the source suffix
and exact materialization bytes and origins. LocalMerged uses the
complete embedded assertion and the transformed vendored resizable helper;
vendored-only uses complete `sta.js`, `assert.js` and the same helper. Deleting
the complete rewrite helper, its sole dispatcher call and both path predicates
removes three T16 semantic observations. Broad resizable admission and every
neighboring `TypedArray.prototype.at` case remain unchanged. The rebuilt
production dispatcher passed the same exact `4/4` cohort with every
non-success bucket at zero while retaining T13's helper substitution. That
checkpoint left 73 T16 observations. The pinned
`Array.prototype.includes/resizable-buffer-special-float-values.js` body then
passed a separate raw `4/4` preflight across both Script modes and both prelude
stores. Every execution used Wasm-AOT with the exact source and applicable
prelude bytes. Only T13's static-subclass helper substitution was applied; the
unmodified helper still reports the explicit Function-constructor
AOT-unsupported diagnostic. This is pre-delete evidence, not full-helper
support or a post-delete production run. The expanded three-source invariant
pins both stores, both modes, the original source suffix, exact materialized
bytes, the sole helper replacement and contract membership. Removing the
terminal special-float materializer left the other two Array `includes`
rewrites and shared dispatcher unchanged. After deletion, the rebuilt
production dispatcher passed the exact source in both Script modes (`2/2`)
with every failure and non-success bucket at zero. That historical checkpoint
contained 356 entries, 208 semantic shortcuts and 72 T16-owned observations.
The two remaining Array `includes` sources each pass a separate direct raw
`4/4` preflight across both Script modes and both prelude stores. Every run
reports `backend_used: WasmAot`; the only helper change is T13's static
subclass substitution. The unmodified helper still reports the explicit
Function-constructor AOT-unsupported diagnostic. The expanded five-source
invariant pins both retired Array `at` bodies and all three retired Array
`includes` bodies with exact modes, sources, preludes, origins, original
suffixes, contract membership and the sole helper replacement. Deleting the
complete remaining Array `includes` rewrite, its dispatcher call and both path
predicates removes three more T16 semantic observations. Broad Array
`includes` resizable admission, T13's helper contract and neighboring Array
search rewrites remain unchanged. That historical checkpoint contained 353
entries, 205 semantic shortcuts and 69 T16-owned observations; T17 owned 161.
After deletion, the rebuilt production dispatcher passed the exact final
two-source cohort in both Script modes (`4/4`) with every failure and
non-success bucket at zero.

The exact `built-ins/Array/prototype/map/resizable-buffer.js` body then passed a
pre-delete raw `4/4` matrix across both Script modes and both prelude stores
with exact source bytes and only T13's static-subclass helper substitution. The
unmodified helper still reports the explicit Function-constructor
AOT-unsupported diagnostic, so this is neither full-helper support nor
post-delete production-dispatch evidence. The expanded six-source invariant
pins the map source, declared comparison and resizable helpers, and exact
LocalMerged and vendored-only bytes and origins in both modes. Deleting only
the map branch from the known-static `for-of` rewrite removes one T17 semantic
observation. The remaining TypedArray accessor authority and shared
resizable-directory substitutions stay intact. That checkpoint's inventory
contained 352 entries and 204
semantic shortcuts and 69 T16-owned observations; T17 owns 160, split between
79 semantic shortcuts and 81 diagnostic guards. After deletion, the rebuilt
production dispatcher passed the exact map source in both Script modes (`2/2`)
with every failure and non-success bucket at zero.

The seven pinned Array iteration `resizable-buffer.js` bodies for `find`,
`findIndex`, `findLast`, `findLastIndex`, `every`, `some` and `filter` then
passed an exact raw `28/28` matrix across both Script modes and both prelude
stores. The `find` source supplied a separate `4/4` preflight; sibling proof
lanes supplied the remaining `24/24`. Every execution reported
`backend_used: WasmAot`, kept the exact source, and changed only the resizable
helper's T13-owned dynamic subclass block into three static classes. `filter`
declares `compareArray.js` and `resizableArrayBufferUtils.js`; the other six
declare only the resizable helper. The unmodified helper still reports the
explicit Function-constructor AOT-unsupported diagnostic, so the raw matrix does
not establish full-helper support.
The expanded thirteen-source invariant pins both modes and stores, original
source and exact materialization bytes, includes, origins, original suffixes,
supported-feature and no-rewrite boundaries, the sole helper replacement and
T13 contract membership. Deleting the complete handwritten iteration rewrite,
its sole dispatcher call and all seven path predicates removes eight T16
semantic observations. Broad `find*`, `every`, `some` and `filter` resizable
admission remains unchanged, as do the neighboring mid-iteration,
`toLocaleString` and search rewrite authorities. After deletion, the rebuilt
production dispatcher passed the exact seven-source cohort in both Script modes
(`14/14`) with every failure and non-success bucket at zero. That historical
checkpoint contained 344 entries, including 196 semantic
shortcuts; T16 owned 61. The six pinned Array `reduce` and `reduceRight`
resizable-buffer bodies then passed an exact raw `24/24` matrix across both
Script modes and both prelude stores. Every execution reported
`backend_used: WasmAot`, preserved the exact source and declared
`compareArray.js`, and applied only T13's static-subclass replacement in
`resizableArrayBufferUtils.js`. A representative unmodified-helper execution
stopped at the explicit Function-constructor dynamic-code-generation boundary,
so this is scoped pre-delete evidence rather than full-helper support. The
expanded nineteen-source invariant pins exact modes, original source and
prelude bytes, includes, origins, suffixes, no-rewrite boundaries and T13
contract membership. Deleting the complete reduce rewrite, its sole dispatcher
call, both one-caller source builders and the obsolete synthetic rewrite test
removes six T16 semantic observations. Broad reduce/reduceRight resizable
admission, `array_iteration_resizable_constructor_names`, T13's helper
contract, and neighboring resizable authorities remain. After deletion, the
rebuilt production dispatcher passed the exact six-source cohort in both Script
modes (`12/12`) with every failure and non-success bucket at zero. That
historical checkpoint contained 338 entries, including 190 semantic shortcuts;
T16 owned 55. The exact four-source Array `indexOf` and three-source Array
`lastIndexOf` resizable cohort then passed a raw `28/28` matrix across both
Script modes and both prelude stores. Every execution reported
`backend_used: WasmAot`, kept the pinned body and declared
`resizableArrayBufferUtils.js`, and changed only T13's dynamic subclass block
to three static classes. The unmodified helper stopped at the explicit
Function-constructor dynamic-code-generation boundary. Review found that the
handwritten `lastIndexOf` rewrite had also hidden a missing broad Array
`lastIndexOf/` resizable admission. One explicit closed prefix set now admits
`includes/`, `indexOf/` and `lastIndexOf/`, and the feature-gate witness proves
all three members. The expanded twenty-six-source invariant pins exact modes,
source and prelude bytes, includes, origins, suffixes, no-rewrite boundaries
and T13 contract membership. Deleting both complete search rewrites, both
dispatcher calls, all seven direct path predicates, both obsolete synthetic
tests and both dead shared prelude/constructor builders removes nine T16
semantic observations. Consolidating the two prior search admissions removes
one diagnostic observation. The neighboring mid-iteration and
`toLocaleString` authorities and broad TypedArray search admission remain. After
deletion, the rebuilt production dispatcher passed the exact seven-source cohort
in both Script modes (`14/14`) with every failure and non-success bucket at zero.
That Array-search retirement checkpoint contained 328 entries: 35 legitimate
harness adaptations, 112 diagnostic instrumentation sites and 181 semantic
shortcuts; T16 owned 45. The fourteen Array
`every`/`some`/`filter`/`find`/`findIndex`/`findLast`/`findLastIndex`
grow/shrink-mid-iteration sources then passed all `56/56` raw Wasm-AOT
executions across both Script modes and both prelude stores, split into `24/24`
quantifier and `32/32` find-family cases. Each retained its exact source and
ordered `compareArray.js` plus `resizableArrayBufferUtils.js` includes; only
T13's dynamic subclass definitions became the three static classes. The
unmodified helper stopped at the explicit Function-constructor
dynamic-code-generation boundary. The expanded real-source invariant pins all
fourteen paths, both modes and stores, source and prelude bytes, include order,
origins, suffixes, no-rewrite boundaries and T13 membership. Deleting the
complete shared rewrite, its sole dispatcher call, the one-caller constructor
list and the obsolete synthetic rewrite test removes its entrypoint and all
fifteen direct predicates. Broad resizable admissions for all seven methods,
the T13 helper contract and neighboring Array values, iterator and
`toLocaleString` authorities remain. After deletion, the rebuilt production
dispatcher passed the exact fourteen-source cohort in both Script modes
(`28/28`) with every failure and non-success bucket at zero. That historical
inventory contained 312 entries and 165 semantic shortcuts; T16 owned 29. The
exact Array `values` `resizable-buffer.js`, grow-mid-iteration and
shrink-mid-iteration sources then passed all `12/12` raw Wasm-AOT executions
across both Script modes and both prelude stores. Each kept its exact body and
ordered `compareArray.js` plus `resizableArrayBufferUtils.js` includes; only
T13's dynamic subclass block became three static classes. The helper
fingerprint `0x6466_6602_9ee8_9d5d` and case fingerprints
`0x5e5c_6ead_7b7c_0dda`, `0x3d18_7152_c6ff_a624` and
`0x60c2_a9ec_1dff_dd03` authorize that change. Modified helper, path, includes
or source bytes retain `new Function` and reach the explicit
Function-constructor dynamic-code-generation boundary. The expanded exact
invariant pins all three sources, modes, stores, bytes, origins, suffixes,
no-rewrite checks and T13 contract membership. Removing both complete
Array-values rewrites, both sole dispatch calls and both obsolete synthetic
tests deletes two entrypoints and three direct predicates. Broad Array-values
resizable admission, Array keys/entries iterator paths, T13's helper contract
and the neighboring `toLocaleString` rewrite stay intact. The current inventory
contains 307 entries: 35 legitimate harness adaptations, 112 diagnostic
instrumentation sites and 160 semantic shortcuts. T16 owns 24; T17 remains at
160, split between 79 semantic shortcuts and 81 diagnostic guards. After
deletion, the rebuilt production dispatcher passed the exact three-source
cohort in both Script modes (`6/6`) with every failure and non-success bucket
at zero.

The three exact Array `toLocaleString` resizable-buffer sources then passed all
`12/12` raw Wasm-AOT executions across both Script modes and both prelude
stores. Each preserved its pinned body, declared only
`resizableArrayBufferUtils.js`, and changed only T13's dynamic subclass block
to three static classes. The helper fingerprint `0x6466_6602_9ee8_9d5d` and
case fingerprints `0x9da9_18f5_d04d_d764`, `0xc380_4490_04ea_5b59` and
`0x07d1_d14e_3a0b_bb89` admit that replacement. Modified helper, path, include
or source bytes retain `new Function`; a representative unmodified-helper run
stopped at the explicit Function-constructor dynamic-code-generation boundary.
The expanded invariant pins all three sources, modes, stores, bytes, origins,
suffixes, no-rewrite checks and T13 memberships. Removing the complete Array
`toLocaleString` rewrite, its sole dispatch and obsolete synthetic test deletes
one entrypoint and three direct predicates. Broad Array `toLocaleString`
resizable admission and its witness, T13's helper contract, TypedArray
`toLocaleString` behavior and neighboring DataView rewrites remain. The
pre-retirement inventory contained 307 entries and 160 semantic shortcuts. The
regenerated source ledger contains 303 entries: 35 legitimate harness
adaptations, 112 diagnostic instrumentation sites and 156 semantic shortcuts.
T16 owns 24; T17 remains at 160 and T18 owns 12. After deletion, the rebuilt
production dispatcher passed the exact three-source cohort in both Script
modes (`6/6`) with every failure and non-success bucket at zero.

The non-callable-`join` `Array.prototype.toString` path now preserves the
ordinary `Object.prototype.toString` builtin-tag algorithm. Its fallback uses
typed recursive `IsArray`/`IsCallable` decisions and the Error, Date and RegExp
internal brands before `@@toStringTag`, so direct and nested Proxy Arrays keep
the Array tag and revoked Proxies throw in the required observation order. The
former exact-path materializer is gone; its unchanged pinned source and full
assert harness pass both sloppy and strict Wasm-AOT executions.

The generic callback tranche (`map`, `flatMap`, `every`, `some`, `filter`,
`find*`, `forEach`, `reduce` and `reduceRight`) and the search/access tranche now
observe borrowed TypedArrays through a closed view/witness API. The view carries
the immutable fixed extent, a length witness snapshots one backing-store length
for `LengthOfArrayLike`, and each live integer-indexed `HasProperty` or `Get`
gate takes a fresh witness. `at` selects generic length observation or validated
TypedArray entry through its closed receiver policy. Generic `includes`
continues to perform the observable `LengthOfArrayLike` and per-index `Get`
operations rather than borrowing the non-generic TypedArray entry rule. This
prevents an out-of-bounds observation from erasing the extent needed after a
later regrow, and length-tracking views floor odd backing-byte lengths to whole
elements.

The generic `Array.prototype.every` and `some` compilers now have only their
single valid Array-like entry state. Their unreachable `typed_array_only`
Boolean, duplicated strict TypedArray validation and dead brand locals are
gone; each body retains exactly one live `IntegerIndexedProperty` witness for
borrowed TypedArrays. The dispatcher separately routes the strict
`%TypedArray%.prototype` methods through the closed
`TypedArrayQuantifierKind::{Every, Some}` compiler. The exact producer boundary,
preserved live-index policy and focused evidence are recorded in the
[Array quantifier entry contract](../docs/rust-rewrite/contracts/array-quantifier-entry-boundary.md).
The bounded structure target passes `4/4`, and all six existing focused CLI
fixtures pass: two Array `every`, three Array `some` and the strict TypedArray
family control. `cargo xc` is green. The following 669-dump semantic golden
passes `2/2` in 771.49 seconds, adds only the independent Temporal arithmetic
witness and removes none. After accounting normalization, 667 of 668 retained
dumps are equal; the sole retained structural change is the independent
Promise callback Realm witness. No Test262 baseline or published-count change
is claimed.

The 2026-08-29 quantifier residue cleanup removes the copied Array-result path
from the generic `every` and `some` compilers. Neither compiler now reads
`constructor` or `Symbol.species`, allocates an unused result Array, or retains
the target, flattening and declaration-only TypedArray temporaries copied from
Array-producing methods. Each top-level temporary census falls from 50 to 28,
while the live borrowed-TypedArray view and integer-indexed witness remain.
The structure target pins those absences, the five required TypedArray locals,
one fresh witness and reverse release order. A combined CLI fixture records that
both methods ignore throwing constructor and species getters, but that runtime
case already passed with the former constant-false Wasm branch. It documents
semantics rather than proving instruction removal. The two bounded structure
targets pass `10/10`; this fixture and four neighboring Array/TypedArray core
and resizable-buffer controls pass `5/5`. No Test262 or published-count change
is claimed.

The shared Array/TypedArray reducer now selects traversal through the private,
capability-free `ArrayReduceDirection` domain. The four `reduce`/`reduceRight`
producers choose `LeftToRight` or `RightToLeft` explicitly, and the compiler
borrows that single owned authority through the method name, both
direction-sensitive diagnostics and all six cursor decisions. Clone, copy,
debug, default, comparison, ordering and hashing capabilities are absent. The
fully borrowed reducer is
`3acf772d37f91e4c1d9ca47302e70a49dfa0bace06f8eddddddc1d9ec61331d8`;
erasing only direction borrow markers reproduces
`ca2e89b9653e32b049f844629a4c0a3c3df7252b229cb15e38c16bfe10ddb475`.
A bounded structure guard pins the four-producer and nine-decision census
without a Boolean projection, while the existing reduce fixture distinguishes
both directions for ordinary Arrays and TypedArrays.

Batch AQ makes the raw `ArrayReduceDirection` and shared reducer private to
`builtins/array.rs`. Four fixed Array/TypedArray `reduce` and `reduceRight`
entries select receiver kind and direction internally; standard dispatch can
no longer import, construct or pass the raw direction. At the 2026-08-28 Batch
AQ checkpoint, `cargo xc` is green, the strengthened direction and neighboring
receiver-kind structure targets each pass `4/4`, and the exact
Array/TypedArray forward/reverse reduce CLI control passes `1/1`. This
source-equivalent tightening claims no new Array or TypedArray behavior and no
Batch AQ Test262 or semantic-golden result.
The following shared workspace semantic golden passes `2/2` in 696.00 seconds
with 668 dumps, adds only the independently expanded shape-accessor witness,
and removes none. After accounting normalization, 664 of 667 retained dumps
are equal; the only structural changes are the intended Array reduce, Promise
internal-callback Realm, and TypedArray constructor no-species witnesses.
At the 2026-08-28 Batch Z checkpoint, `cargo xc` is green, the strengthened
structure target passes `3/3`, and the three exact CLI controls pass `3/3`.
The four pinned forward/reverse Array and TypedArray leaves pass all `8/8`
Wasm-AOT executions with every failure bucket at zero. Semantic goldens remain
deferred. The bounded evidence is recorded in
`docs/rust-rewrite/contracts/array-callback-receiver-kind.md`.

The shared `sort`/`toSorted` emitter now receives the crate-private non-Copy
`ArraySortOutput::{Receiver, Copy}` domain. Four borrowed exhaustive matches
own result allocation, hole collection, sorted-entry publication and trailing
source deletion, so a future output cannot inherit an equality fallback.
The exact two standard producers and four semantic projections are recorded in
the focused
[Array sort output contract](../docs/rust-rewrite/contracts/array-sort-output.md).
The strengthened structure target passes `3/3`, the three exact sort CLI
fixtures pass `3/3`, and six focused sort/toSorted Test262 leaves pass all
`12/12` sloppy/strict Wasm-AOT executions with every failure bucket at zero.
The package formatter check is green. This is a source-equivalent invariant
closure with no new fixture or broad Array/Test262 status claim.

The output authority and 604-line raw sort algorithm are now private behind
two fixed `sort` and `toSorted` entries. The strengthened
[`Array sort output contract`](../docs/rust-rewrite/contracts/array-sort-output.md)
records exact original domain and raw-algorithm SHA-256 witnesses
`1745b093aab4e0643c08de0b1d402f3770ef5a9618635ae7b31ec318a8c74c4c`
and
`aa8c4c988b2c5e64568cfc9f4a294c98a32144af941450cf59ac882948afbf25`.
The output target passes `4/4`, its dispatch-owner neighbor passes `5/5`, and
the three exact Array sort CLI controls pass `3/3`. The repository gates are
green. This source-equivalent hardening has no new Array behavior and does not
close T16.

Static `sort()` lowering now constructs the local, capability-free
`SortMethodDispatch::{TypedArrayCanonical, ArrayCanonical, GenericGetCall}`
authority. A sole strict TypedArray shape target has precedence over a sole
Array builtin target; absent, accessor and ambiguous targets fall through to
ordinary property Get and Call. One exhaustive match owns both canonical calls
and the generic fallthrough, with no kind-only shortcut or independent target
Boolean. This prevents an own Array `sort` override from being bypassed and
prevents TypedArrays from entering the generic Array length and string-order
algorithm. The Array sort body remains pinned at
`20aa3a5afff0f855e5c574ba03d4fc38be8649be093faa091e99ee3c593a8ba2`; the
strict TypedArray entry plus private stable-sort body remains pinned at
`0936699959dc0e3e55f343e7b37101ebf6d13d9ab9bb32cb0df6896e6c2c23b4`.
A new Array override fixture pins custom receiver, ordinary/spread argument
order, return value and unchanged elements. The existing TypedArray sort
fixture pins internal length and numeric default order. Focused compilation,
runtime and Test262 verification are green: the recursive structure target
passes `5/5`, both exact runtime controls pass `2/2`, and the three pinned
Array plus three pinned TypedArray leaves pass all `12/12` Wasm-AOT executions
with every failure bucket at zero. The shared `cargo xc`, formatting, diff,
module-boundary and task-plan checks are green. The bounded contract is
`docs/rust-rewrite/contracts/array-sort-dispatch-owner.md`.

The shared reducer and `forEach` entry compilers now project the private
two-case `ArrayCallbackReceiverKind` directly. The kind no longer implements
equality, and the former two equality collapses and two `typed_array_only`
Boolean carriers are gone. Six reducer decisions, five `forEach` decisions and
the reducer's two existing receiver/direction decisions form thirteen direct
exhaustive projections. The exact six dispatcher producers and the preserved
validated-entry and live integer-indexed witnesses are recorded in the focused
[Array callback receiver-kind contract](../docs/rust-rewrite/contracts/array-callback-receiver-kind.md).
The bounded structure target passes `4/4`, and the three existing focused CLI
witnesses pass `3/3`. The following shared 674-dump semantic golden passes
`2/2` in 717.58 seconds; this source-only closure adds no fixture and all 671
retained dumps are equal after accounting normalization. It makes no Test262
baseline or published-count change.

Batch AA makes the capability-free `ArrayCallbackReceiverKind` the sole
receiver-policy authority. The six dispatcher entries still move exactly one
of the two variants into one of two owning compiler parameters. Both reducer
helper projections and all eleven compiler matches now borrow those owners, so
receiver diagnostics, generic-versus-strict entry, key construction and
property-presence policy cannot be selected from independently copied
authorities. The borrowed direction projection, reducer and `forEach` bodies
are respectively
`20daa9d9e1b1e235a96c6253c5f7c6ad23c13ce269b92bab79a4cd497c00c3ff`,
`ab4ecea3dddb22dcfb0e812be2d05ddc657369fad9dcf7d31bdfd480329ceb90`
and `ea047de76bef8b4c5fbc8eb440c42329e7693feecf848cef753011cf2a541c26`.
Erasing only the new receiver borrows reproduces the surveyed bodies at
`ed439784343d2db70ab528aef33047b628d077db2de431935d9a372180446de4`,
`3acf772d37f91e4c1d9ca47302e70a49dfa0bace06f8eddddddc1d9ec61331d8`
and `52d8982bbef8b3a99ce51a870919b604394773948aa1944d3f21e939a7aa15fb`.
At the Batch AA checkpoint, `cargo xc` is green, the strengthened structure
target passes `4/4`, and the focused Array reduce, BigInt TypedArray reduce and
resizable-TypedArray `forEach` CLI witnesses pass `3/3`. The four pinned
Array/TypedArray reduce and reduceRight leaves pass all `8/8` Wasm-AOT variants
with every failure bucket at zero. The pinned `forEach` leaves and semantic
goldens were not rerun.

That receiver authority and the 442-line raw `forEach` compiler are now
private behind fixed Array and TypedArray `forEach` entries, matching the four
already-fixed reducer routes. The updated
[`callback receiver-kind contract`](../docs/rust-rewrite/contracts/array-callback-receiver-kind.md)
records exact original SHA-256 witnesses
`c073b0a9449fae68b12f82e43fc0bf7dc52a0a0bc98b1a6eb2bf6d5b0bce3ea1`
and
`ea047de76bef8b4c5fbc8eb440c42329e7693feecf848cef753011cf2a541c26`.
The callback and direction targets pass `4/4` each, and the exact
resizable-TypedArray generic `forEach` CLI control passes `1/1`. The repository
gates are green. This source-equivalent hardening has no new Array behavior and
does not close T16.

The generic `Array.prototype.flatMap` TypedArray specialization now loads one
immutable view and uses it for both incompatible observation points: one
non-throwing `ArrayLikeLengthSnapshot` captures the source bound, and a fresh
`IntegerIndexedProperty` observation inside the loop determines whether the
current source index is present. The existing live indexed read and mapper call
remain after that presence result. A bounded guard also rejects any legacy raw
current-length call across the complete Array builtin source. The focused
[flatMap TypedArray buffer-witness contract](../docs/rust-rewrite/contracts/array-flat-map-typed-array-buffer-witness.md),
structure target and resizable/detached CLI fixture are written as of
2026-08-24. At that date's central checkpoint, `cargo check` and `cargo xc`
were green, the structure target passed `3/3`, the exact CLI fixture passed
`1/1`, and the one unrewritten vendored leaf passed both ordinary executions
`2/2` with every failure bucket at zero. No baseline, README, published-count
or broader conformance change is claimed.

The Array and TypedArray `find`, `findIndex`, `findLast` and `findLastIndex`
emitters now share one closed, capability-free `FindViaPredicateKind`. Each
dispatcher entry moves one kind into its compiler, which borrows the same
authority through all seven exhaustive direction, result and surface-text
projections. Clone, copy, debug, default, comparison, ordering and hashing
capabilities are absent. The capability-free `FindDirection` remains private and binds
each compiler's index initialization and advancement through two exhaustive
borrowed consumers; the same owned direction reaches all four call sites and
cannot be copied or compared into a second traversal decision. The private,
capability-free `FindProjection` likewise binds each compiler's miss-result
initialization to its successful-match value/index projection through two
exhaustive borrowed consumers and four call sites. Erasing only the projection
borrow markers reproduces the frozen Batch X raw compiler hashes
`ece6c116f388ab7ca262b90d55ff58529e85a2d5ef5c2abfa0610c790ad797c9`
and `21a37e37281c0528d4148d935f56196c14f2e58716e0784a9eea12960dbc136f`;
erasing both projection and direction borrow markers reproduces
`5aaece4591126bfc317affcc137762a7f00bba4288ce5f8cd8e93dc6331fa32e`
and `9f54a114dbee477e0c430d03e54159cd3a452247ac3f58a17969fdbf54622103`;
the eight standard mappings remain
`13b2e609dd878f19762612dad1851febd9390c21b4bca021c3f41c71908ff1a8`.
The old generic booleans and unreachable TypedArray-only branch are gone. A
private, non-Copy predicate witness is constructible only through the general
`IsCallable` operation and has one ownership-consuming, Proxy-aware `Call`
boundary. This admits callable Proxy predicates while retaining receiver/length
observation before callability validation for both entry families. The
strengthened structure target passes `5/5`, and the exact Array, reverse Array
and TypedArray CLI controls pass `3/3` at the 2026-08-28 Batch X checkpoint.
The pinned resizable-buffer, strict-callback-`this` and abrupt-length leaves pass
all `5/5` Wasm-AOT executions with every failure bucket at zero; `cargo xc` is
green. Semantic snapshot and broader Array/Test262 verification remain
deferred. Formatting, diff, module-boundary and task-plan checks are green.
At the 2026-08-28 Batch Y checkpoint, projection hardening passes the same
structure target `5/5` and exact Array, reverse Array and TypedArray CLI
controls `3/3`. The four projection-focused Wasm-AOT leaves pass all `8/8`
executions with every failure bucket at zero, and the shared `cargo xc`
checkpoint is green. The exact boundary and its nonclaims are recorded in
`docs/rust-rewrite/contracts/array-find-via-predicate.md`.

The four-way kind and both raw family compilers are now private to that child;
standard dispatch can call only eight fixed Array/TypedArray find-family
entries. The updated contract records exact original SHA-256 witnesses
`3989f2ebe1ce925d23b20d4e06eb35f00e1e840f7509b8226b9b425a639c4e5c`,
`40be1db2dd3ccb1f35a9e022061f4fb23a8adc8fac8e446f06fdb93879b3e92d`
and
`b71e9cfcea61c77cdbef9aeb68917c65e1e54ab1bbe735e49a4175d82f00673e`.
The structure target passes `5/5`, and the exact forward Array, reverse Array
and TypedArray controls pass `3/3`. The repository gates are green. This
source-equivalent hardening has no new Array behavior and does not close T16.

The distinct Array and TypedArray `toLocaleString` entry points now share one
element-invocation boundary. A private, non-`Copy` validation token pairs the
general-`IsCallable`-validated method with the exact original element receiver,
and its sole ownership-consuming call path is Proxy-aware and passes no
arguments. A non-callable element method now throws in the active built-in's
current-function realm, including when a created realm's Array or TypedArray
method is borrowed. The exact boundary, static evidence and baseline
nonclaims are recorded in
`docs/rust-rewrite/contracts/array-to-locale-string-invocation.md`.

The shared `at` emitter now owns a capability-free
`ArrayAtReceiverPolicy::{GenericArrayLike, TypedArray}` rather than projecting
the authority through a reusable validation Boolean. Its two standard entry
producers move the policy to the shared emitter; direct TypedArray lowering now
selects the strict standard entry. The emitter borrows the policy through four
direct exhaustive decisions for Array/Arguments handling, TypedArray witness
selection, ordinary Object/Function handling and primitive/nullish handling.
Adding a new receiver policy therefore fails to compile until every independent
receiver decision is stated. The exact producer, ordering, behavior and
verification boundaries are recorded in
`docs/rust-rewrite/contracts/array-at-receiver-policy.md`.
On 2026-08-28, the owned structure target passed `3/3`, the direct-entry owner
target passed `4/4`, and the exact Array/TypedArray runtime-kinds CLI witness
passed `1/1`. Independent review confirmed the 14-use authority census, the two
standard policy constructors and all four complete receiver-policy bodies.
Targeted Rust formatting and the scoped diff check passed. The neighboring
TypedArray search-kind target, broad workspace compile and Array Test262
verification were not rerun in the direct-entry lane.

Batch AR makes the raw policy and shared compiler private to `array.rs`.
Standard dispatch can call only the fixed Array and TypedArray `at` entries and
cannot import, construct or pass the private `ArrayAtReceiverPolicy`. The
frozen 34-line compiler has SHA-256
`4888ef68f6f42b58d9e14480d5381cf64018176ed21504a10fc6883dac564aaa`;
normalizing its private name and visibility reproduces that hash exactly. At
the 2026-08-28 Batch AR checkpoint, `cargo xc` is green, the strengthened
structure target passes `4/4`, and the exact runtime-kinds CLI control passes
`1/1`. This source-equivalent tightening claims no new Array behavior and no new
TypedArray behavior, and no Batch AR Test262 or semantic-golden result.

Batch AB provisionally replaced the lossy truthiness cell for Array-owned
`Symbol.isConcatSpreadable` with an exact tagged dedicated slot. Batch AC's
focused descriptor review then showed why that provisional representation
could not be retained: runtime Symbol writes could split between ordinary named
storage and the slot, and the slot could represent a getter but not its setter.
Its receiver write also overwrote getter-only and non-writable descriptors as
though every occupied slot were writable data. The Batch AB slot is historical
evidence only and does not describe the current product state; Batch AC
supersedes it.

Batch AC closes that representation split by deleting the dedicated slot in
full. The ordinary Array named-property owner is now the only storage for this
Symbol key. Static and computed assignment use ordinary Set; static reads and
concat use ordinary Get; and Object.defineProperty sends both data and accessor
descriptors to the existing Array named descriptor compilers. Getter, setter,
writable, enumerable and configurable state therefore cannot drift between two
owners. Arguments objects retain their distinct exotic implementation.

The removed capability enum, read/write emitters, four heap fields, four layout
rows and two initializer groups have zero product occurrences. Their offsets
remain padding: `HEAP_ARRAY_RECORD_SIZE`, later Array offsets and dense element
storage are unchanged. The recursive owner target passes `5/5`; the focused
`array_concat_spreadable` CLI filter passes `5/5`; and the exact
descriptor-assignment witness passes `1/1`. Together they cover getter-only
sloppy/strict rejection, setter receiver/value behavior, non-writable
sloppy/strict rejection, direct accessors, aggregate concat results, receiver
identity and observable order/errors. The neighboring Array-at receiver-policy
target passes `3/3`. The pinned `is-concat-spreadable-val-truthy.js`,
`is-concat-spreadable-get-order.js` and `is-concat-spreadable-get-err.js`
controls pass all `6/6`
sloppy/strict Wasm-AOT executions with every failure bucket at zero, and the
shared `cargo xc` checkpoint is green. The frozen raw owner hashes are
`b970db24ecd2f945b25e564610b598b5c4163a4661bb61a507f38a81cb760bde`
(named data descriptor),
`88549739cc949da6a6e5834ef75e52593b5cdd85ba87899f416ddca4bb3771de`
(named accessor descriptor),
`e3bcf4992367960b8a205469f5ec94e1d56ade0a4039ff7f64ebf2995a7fd3e4`
(concat compiler),
`febe236df75d13bf589053d980867bb63f44e595c9e1a4a1613bb111b164098f`
(Array own named read) and
`d58fee7ab153c8d398a112fb38ac086c3661c3f198b3503add6ff960d70f454c`
(OrdinarySet receiver fallback). The semantic golden remains deferred. The
storage and verification boundary is recorded in
`docs/rust-rewrite/contracts/array-concat-spreadable-tagged-slot.md`.

Batch AD makes the existing five-state inherited Array-index Set outcome a
closed compiler authority. The capability-free `ArrayInheritedIndexSetState`
no longer derives clone, copy, debug or equality, and no longer relies on a
Rust numeric representation or implicit discriminants. Its sole `code(&self)`
projection exhaustively maps `Unhandled`, `Setter`, `OrdinaryRejected`,
`Handled` and `ProxyRejected` to their unchanged Wasm-local codes 0 through 4.
A future sixth state therefore fails to compile until its runtime mapping is
reviewed, while the outcome cannot be copied or compared into a second policy.

The exact census is 19 type-name occurrences and 16 code producers, distributed
2/3/6/2/3 across the five variants. Ordinary Array assignment and canonical
dense Push remain the only two consumers of the prototype-chain state emitter.
The assignment body
`e91057b92ce1a9491c657ab22936fdebe348d3ead110abf9cacef79c347b23d3`,
state emitter
`c4806b5db178523368c877c3a663d0fe77ca4963022ac46f1a2602a083efb089`
and canonical Push branch
`c954cac5f939488f2cb6d07e5b9d70fba3224d33ec57c708570e6759856cf6c8`
remain byte-identical. `standard.rs` is unchanged. At the shared Batch AD
checkpoint, `cargo xc` is green, `object_write_proxy_realm_structure` passes
`5/5`, and the exact
`object::proxy_set_errors_use_the_borrowed_builtin_realm` and
`array::run_wasm_backend_expands_all_array_push_arguments_before_appending` CLI
controls each pass `1/1`. The exact
`staging/sm/Array/set-with-indexed-property-on-prototype-chain.js` Test262 leaf
passes both sloppy and strict Wasm-AOT executions (`2/2`). No semantic golden
was run for Batch AD. The bounded contract is
`docs/rust-rewrite/contracts/array-inherited-index-set-state.md`.

The available current-pin Array prototype baseline predates the T10
`Object.prototype.toLocaleString` repair and still records two primitive
`toLocaleString` failures caused by its former boxed getter and call receiver.
The Object path now statically preserves the original primitive through GetV
and Proxy-aware Call. The focused structure and CLI fixture pass on the current
working tree; pinned Test262 execution remains deferred, so neither seam
carries a current-SHA baseline-delta or full-subtree-green claim.

`Array.prototype.pop` now has one compiler algorithm owner. Statically named
method calls delegate to `StandardBuiltinId::ArrayPrototypePop` instead of
reading and shrinking the raw dense-Array heap record in `functions.rs`. The
canonical standard body therefore owns `ToObject`, `LengthOfArrayLike`, the
last-property `Get`, deletion, current-function-realm deletion errors, and the
strict `length` write in their observable order. The former direct path could
resurface an old dense slot after a later length regrowth and could not observe
accessors, descriptors or deletion failures. The ownership boundary and its
focused static evidence are recorded in
`docs/rust-rewrite/contracts/array-pop-algorithm-owner.md`.

Static `pop()` lowering now constructs the local, capability-free
`PopMethodDispatch::{ArrayCanonical, GenericGetCall}` authority. Only a data
property whose sole target is `ArrayPrototypePop` selects the canonical call;
absent, accessor and ambiguous targets fall through to ordinary property Get
and Call. One exhaustive match owns both states with no kind/heap-shape shortcut
or independent target Boolean, so an own Array `pop` method cannot be bypassed.
The canonical Pop arm remains unchanged and pinned at
`240862a71152eef7a1373e0bfd98b928ccad87dd1daf057851c099e437c91038`.
A new override fixture pins receiver identity, ordinary/spread argument order,
custom return value and unchanged elements; the existing algorithm-owner
fixture remains the canonical behavior control. At the shared checkpoint, the
recursive structure target passed `5/5`, the exact own-method dispatch and
canonical-algorithm CLI controls each passed `1/1`, and the pinned
`set-length-array-length-is-non-writable.js`,
`set-length-zero-array-length-is-non-writable.js` and
`throws-with-string-receiver.js` leaves each passed `2/2`, for `6/6` Wasm-AOT
executions. The shared `cargo xc` checkpoint is green. This closure changes no
published count, removes no Test262 materializer, and does not claim the Array
or Array prototype tree green.

The three generic Array iterator methods and three strict TypedArray iterator
methods select the private, capability-free `ArrayIteratorReceiverPolicy`
instead of sharing a raw validation Boolean. Batch AE removes its clone, copy,
debug and equality capabilities. One private emitter owns the policy and
borrows it through both exhaustive `GenericArrayLike`/`TypedArray` projections:
first for receiver validation and then for iterator materialization. The same
authority therefore cannot be copied, compared or collapsed to a Boolean while
generic borrowed behavior and runtime TypedArray specialization remain
unchanged.

The exact census is 12 type-name occurrences, two borrowed projections and six
producers split 3/3. The compiler body, normalized only by removing the two
borrow tokens, remains
`4a216f733e6662fc93633fa1c26a2e71317fc3cc1c7bafeac60af38b315e601f`;
the byte-identical producer block remains
`125b6fb9bf2123a2612ab5e530b339e8af1456cacfd362e69b4ad66956f2b77a`.
The finite product witness still covers all six methods, ordinary array-like
borrowing, generic TypedArray specialization, strict TypedArray rejection and
terminal results. At the shared Batch AE checkpoint, `cargo xc` is green, the
bounded structure target passes `3/3`, and the exact
`array::run_wasm_backend_preserves_array_iterator_receiver_policies` CLI witness
passes `1/1`. The exact
`built-ins/Array/prototype/keys/returns-iterator-from-object.js`,
`built-ins/Array/prototype/entries/resizable-buffer.js` and
`built-ins/TypedArray/prototype/keys/this-is-not-typedarray-instance.js` leaves
each pass `2/2`, for `6/6` Wasm-AOT executions with every failure bucket at
zero.

The checkpoint repaired only the structure guard's producer census slice so it
includes the first `ArrayPrototypeKeys` row; product code and both frozen hashes
are unchanged. The earlier 667-dump workspace semantic golden passed `2/2` in
702.89 seconds and added only the iterator-policy witness, but predates this
capability hardening. No semantic golden was run for Batch AE. The bounded
contract is
`docs/rust-rewrite/contracts/array-iterator-receiver-policy.md`.

Batch AF makes the existing
`TypedArrayAccessorKind::{ByteLength, ByteOffset, Length}` selector
capability-free. The three standard accessor entries and the generic
TypedArray `length` read are its complete four-producer set; each selection is
moved through the accessor compiler or directly into `TypedArrayWitnessUse`
and consumed by the sole exhaustive three-arm projection. Clone, copy, debug,
default, comparison, ordering and hashing capabilities are absent, so a future
duplicated or equality-based accessor policy is a compile error rather than an
incidental operation on the selector.

The exact census is 12 product mentions, four producers, one compiler handoff
and one exhaustive projection. The byte-identical declaration body is
`4ce6e008183a7157593950bb1f3f37b10fc02e23a4838e4e811137001158bd54`, the
projection is
`2432528af60e6e41782b24ff671453987486c0692aee51e971f144217f8b25a1`, the
accessor compiler is
`63e797108010410db586f0a136c1238c0502b4e3f816241604ea4b6a3f02e648`, and
the unchanged three-standard-producer block is
`ee60ce214986d62e9811eb190636a02468f9099c60d334b79d3d3341310a1fe7`.
At the shared Batch AF checkpoint, `cargo xc` is green, exact guard
`tests::typed_array_accessors_use_the_closed_buffer_witness` passes `1/1`, and
the exact `typed_array::run_wasm_backend_succeeds_for_typedarray_accessors_fixture`
CLI witness passes `1/1`. The pinned
`built-ins/TypedArray/prototype/byteLength/return-bytelength.js`,
`built-ins/TypedArray/prototype/byteOffset/return-byteoffset.js` and
`built-ins/TypedArray/prototype/length/return-length.js` leaves pass all `6/6`
Wasm-AOT executions with every failure bucket at zero. Batch AF did not rerun
the semantic golden. The bounded contract is
`docs/rust-rewrite/contracts/typed-array-accessor-buffer-witness.md`.

Batch AG makes the shared `TypedArrayViewLocals` private-slot carrier
non-`Clone` and non-`Copy`. Its 46 producers each construct one owned
five-local description, while all live buffer observations borrow that owner;
multi-observation algorithms therefore cannot fork the same private-slot roles
into independently reusable carrier copies. No producer or witness instruction
body changes.

The recursive ownership guard pins 56 exact product mentions, 46 constructors,
two borrowed type boundaries and the attribute-free five-field declaration.
The unchanged declaration is
`64a7e96e10f1d53150a94e915656bd69b2a050449e7fa73b2954093ddd1b5390`, its
constructor implementation is
`7ff4343576674f15b704921718176ace71d92df0927299bfed696ee008a10f80`, and
the shared witness emitter remains
`61daf0915471d6f3f2ac4e62dd3792bb940a318c7a9199676fe327ea852ec226`.
At the shared Batch AG checkpoint, `cargo xc` is green, the expanded ownership
structure target passes `5/5`, and the exact
`typed_array::run_wasm_backend_copies_typedarray_bytes_with_spec_ordering` CLI
witness passes `1/1`. The pinned
`built-ins/TypedArray/prototype/copyWithin/resizable-buffer.js`,
`built-ins/TypedArray/prototype/copyWithin/coerced-values-start-detached.js` and
`built-ins/TypedArray/prototype/copyWithin/coerced-values-end-detached.js`
leaves pass all `6/6` Wasm-AOT executions with every failure bucket at zero.
Batch AG did not rerun the semantic golden. The bounded evidence is recorded in
`docs/rust-rewrite/contracts/typed-array-witness-use-ownership.md`.

The three iterator result kinds now come from the closed
`ArrayIteratorKind::{Key, Value, KeyAndValue}` row authority. Array producers
cannot pass raw words, and ordinary `next` exhaustively emits the three valid
semantics after rejecting an invalid named-property word with the existing
incompatible-receiver `TypeError`. This source-equivalent invariant migration
does not complete Array exotic or broader T16 behavior. Its focused fixture now
also rejects NaN, negative and fractional corrupt words. The following shared
683-dump semantic golden passes `2/2`; the expanded fixture is its sole retained
non-accounting change.

Array named-property count and write emitters now carry the private,
capability-free `ArrayNamedStringKeySelection` authority with `All` and
`EnumerableOnly` variants. The type no longer derives clone, copy, debug or
equality, and each emitter
borrows its owned selection through both exhaustive guard-opening and
guard-closing projections. A future equality or Boolean collapse therefore
fails to compile instead of risking a result allocation whose count disagrees
with the keys written. The four `Object.getOwnPropertyNames` and `Object.keys`
producers remain byte-identical. The consumer span is pinned at
`3fc0be9af14967687a701b61331996f8c929fc87ce99b67c6b131702d6b0c325`; the two
producer spans remain pinned at
`e7fa80b3cdfa13ce4f453febff5ee47a9efc5ccaad3499f6d2235aa677b9151e` and
`37d5be7e6069a3349ac95ebfee64d034d67d51f20a49d30a20a2698b77441327`.
The strengthened structure target passes `3/3`, the existing exact CLI witness
passes `1/1`, and the three adjacent pinned Object leaves pass all six
sloppy/strict Wasm-AOT executions with every failure bucket at zero. The shared
`cargo xc` gate is green. The bounded contract is
`docs/rust-rewrite/contracts/array-named-string-key-selection.md`.

Batch AP makes the raw `ArrayNamedStringKeySelection` and both exhaustive
storage consumers private to `builtins/array.rs`. Four fixed sibling-visible
count/write operations select `All` or `EnumerableOnly` internally; the Object
builtins no longer import, construct or pass the raw policy. The strengthened
structure target passes `4/4`, the exact CLI witness passes `1/1`, and
`cargo xc` is green. This source-equivalent boundary changes no emitted instruction and
claims no new Array or Object behavior.

The Array and TypedArray `toLocaleString` entries now carry their shared
receiver policy through a private, non-derived `ToLocaleStringReceiverKind`.
The compiler borrows that value for exhaustive method-name, element-error and
strict-TypedArray-entry projections; the former `matches!` observation and all
clone, copy, debug and equality capabilities are gone. The two existing branch
bodies and their instruction order are unchanged, so emitted Wasm is expected
to remain byte-identical. The recursive structure target passes `4/4`, and the
neighboring invocation and TypedArray witness targets pass `4/4` each.
The two exact existing CLI witnesses pass `2/2`, and independent review
confirmed the producer/projection tables, capability closure and preserved
instruction order. The coordinated `cargo xc`, formatter, diff and repository
policy checks are green. Broader conformance suites remain deferred. The
bounded contract is
`docs/rust-rewrite/contracts/array-to-locale-string-receiver-kind.md`.

The four `Array.fromAsync` iterator-result continuations now pass a fresh,
non-derived `ArrayFromAsyncIteratorResultProperty::{Done, Value}` selection to
the shared reader. That reader consumes the selection in its sole exhaustive
key projection, so one property choice cannot be duplicated into a second
observable Get. The existing structure guard pins the exact 11 type mentions,
four producers per variant, four Done-before-Value continuation bodies and the
typed reader boundary. This is source-equivalent capability hardening: property
access order, abrupt routing, iterator closing and Promise settlement remain
unchanged. The focused structure target passes `4/4`, and two exact existing
CLI witnesses pass `2/2`. Independent review confirmed the one-shot claim and
the guard now fingerprints the complete reader and all four continuation
bodies. The coordinated workspace checkpoint passes
`cargo fmt --all -- --check`, `cargo xc`, `git diff --check`, the module
boundary check and the task-plan check; the compile retains the repository's
existing warnings. The
bounded contract is
`docs/rust-rewrite/contracts/array-from-async-iterator-result-property-domain.md`.

Batch AL replaces the three free Array.fromAsync source-mode integers with the
private, capability-free `ArrayFromAsyncSourceMode` domain. Its exhaustive
borrowed projection is the only Rust authority for the unchanged ArrayLike,
AsyncIterator and SyncIterator runtime heap wire values. The three semantic
producers and eight comparisons now name that domain; the runtime state offset,
iterator-local handoff, instructions and order are unchanged. The structure
guard also recovers the frozen 51,969-byte normalized algorithm fingerprint
`0x18bf07d71957d97f` after erasing only the typed vocabulary. At the Batch AL
checkpoint, `cargo xc` is green, the structure target passes `4/4`, the two
exact CLI controls pass `2/2`, and the three pinned Test262 leaves pass all
`6/6` Wasm-AOT variants with every failure bucket at zero. No semantic golden
was required or run. The bounded contract is
`docs/rust-rewrite/contracts/array-from-async-source-mode-domain.md`.
Final formatter, diff, module-boundary, task-plan and 240-entry
shortcut-inventory gates are green.

Batch AM replaces the six raw Array.fromAsync continuation-stage integers with
the private, capability-free `ArrayFromAsyncStage` domain. Its exhaustive
borrowed projection is now the only Rust authority for the unchanged
InputValue, MappedValue, AsyncIteratorResult, SyncIteratorDoneValue,
AsyncCloseResult and SyncCloseValue heap wire values. The thirteen stage
producers and nine comparisons name that domain while the stage offset, locals,
instructions and order remain unchanged. Erasing only the typed stage
vocabulary recovers the frozen 41,030-byte normalized algorithm fingerprint
`0xd722936e349517a9`; the neighboring AL guard also preserves its existing
51,969-byte source-mode fingerprint. At the Batch AM checkpoint, `cargo xc` is
green, the new stage and neighboring source-mode structure targets each pass
`4/4`, the three exact CLI controls pass `3/3`, and the three pinned Test262
leaves pass all `6/6` Wasm-AOT variants with every failure bucket at zero. No
semantic golden was required or run. The bounded contract is
`docs/rust-rewrite/contracts/array-from-async-stage-domain.md`.
Final formatter, diff, module-boundary, task-plan and 240-entry
shortcut-inventory gates are green.

The strict TypedArray `every` and `some` entry points now exclusively own the
private, non-derived `TypedArrayQuantifierKind::{Every, Some}` authority. The
standard dispatcher calls the two named entries without access to the kind,
and the shared compiler borrows it through seven exhaustive projections for
diagnostics, callback polarity and both Boolean result paths. A future kind
cannot inherit an equality or Boolean fallback, and another module cannot add
an unreviewed producer. The source-equivalent boundary and its focused static
and behavioral evidence are recorded in
`docs/rust-rewrite/contracts/typed-array-quantifier-kind-authority.md`. The
bounded structure target passes `4/4`, and the exact existing TypedArray
`every`/`some` CLI fixture passes `1/1`; both builds retain the working tree's
existing warnings. Broader conformance verification remains deferred.

The strict TypedArray `includes`, `indexOf` and `lastIndexOf` entry points now
move a private, capability-free `TypedArraySearchKind` into their shared
compiler. The compiler borrows that single authority through all twelve
exhaustive semantic projections; the kind cannot be cloned, copied, compared,
defaulted, ordered, hashed or formatted through a derived debug capability.
The three wrapper producers and three standard mappings remain byte-identical,
with hashes
`e958795ce75a03e5ae44c0aae873180e1c6f709545e884e744efbf1ad1531bb5`
and `2f3e6ff0aeb5df6e64559916af10d70d37eba54b88fd42e40aace08c44800823`.
The strengthened structure target passes `3/3`, the existing exact search CLI
fixture passes `1/1`, and the three pinned Test262 leaves pass all six
sloppy/strict Wasm-AOT executions with every failure bucket at zero. The shared
`cargo xc` checkpoint is green. The bounded ownership and semantic-projection
evidence is recorded in
`docs/rust-rewrite/contracts/typed-array-search-kind.md`.

`Array.prototype.copyWithin` now selects its runtime traversal step through
the private, non-derived
`ArrayCopyWithinDirection::{Forward, Backward}` authority. One exhaustive
projection pairs the forward cursor start with `+1` and the overlapping
backward cursor rewind with `-1`; the compiler body can no longer publish a raw
direction word or change the rewind independently from the step. The exact
default and overlap-only producers, capability boundary and focused structure
evidence are recorded in
`docs/rust-rewrite/contracts/array-copy-within-direction.md`. The focused
structure target passes `3/3`, direct formatting checks for the changed Rust
files are green and the scoped diff check is clean. This is a source-equivalent
invariant closure and does not claim broader Array or Test262 progress.

Direct Array `toString()` lowering now delegates to the same installed
`StandardBuiltinId::TypedArrayPrototypeToString` owner as first-class
`Array.prototype.toString` calls. The former one-caller direct join wrapper and
array-only raw-length join body are deleted, so every product path observes
`Get("join")`, `IsCallable`, Proxy-aware Call and the intrinsic
`Object.prototype.toString` fallback through one canonical algorithm. Real
`Array.prototype.join` ownership is unchanged. The recursive owner/order guard
passes `4/4`, and the direct conversion CLI regression passes `1/1`. The two
remaining focused CLI regressions and exact pinned non-callable join leaf stay
at the shared checkpoint because another source lane was mid-extraction when
their run began. Direct formatting for the frozen source plus the scoped diff
check are green. The bounded contract is
`docs/rust-rewrite/contracts/array-to-string-algorithm-owner.md`.

Static `reverse()` lowering now constructs one local, capability-free
`ReverseMethodDispatch::{TypedArrayCanonical, ArrayCanonical, GenericGetCall}`
decision. A sole strict TypedArray shape target has precedence over a sole
Array builtin shape target, while absent or ambiguous targets fall through to
ordinary property Get and Call. Array kind and Array heap shape alone are not
canonical authority, so an own Array `reverse` override cannot be bypassed.
One exhaustive match owns the two canonical emissions and generic fallthrough,
so the former unconditional Array routing cannot be restored by changing a
separate Boolean or call site. The deleted dense Array owner remains absent;
the generic Array and strict TypedArray compilers remain distinct and unchanged.
The recursive guard pins the three producers/consumers, TypedArray-first order,
complete argument forwarding, both canonical dispatcher owners, and the Array
endpoint observation order. A new Array fixture pins own-method receiver,
argument/spread order and unchanged elements; the existing TypedArray reverse
fixture's throwing own `length` getter controls private-length dispatch.
The Array and TypedArray bodies remain pinned at
`6bd42e25ba1e1235dd4f0a08d8df88c5891ed2d05b15e2667a55f6b7cbed7688` and
`f930f4c6b07e2729928bc23fea4268d8d09d6d197eda363b8aa8da1321c41a0e`.
Targeted formatting, fixture syntax, diff, module-boundary and task-plan checks
are green. The recursive structure target passes `5/5`, and both exact runtime
controls pass `2/2`. The three pinned Array and three pinned TypedArray controls
pass all `12/12` Wasm-AOT executions with every failure bucket at zero. The
shared `cargo xc` checkpoint is green. The bounded contract is
`docs/rust-rewrite/contracts/array-reverse-algorithm-owner.md`.

Static direct `Array.prototype.includes` lowering now delegates to
`StandardBuiltinId::ArrayPrototypeIncludes`. The deleted one-caller entry
compiled only the first two source arguments, allowing third-argument side
effects and spread iteration to disappear. The shared direct-call boundary now
evaluates and expands the complete argument list left to right before the
unchanged canonical compiler reads its first two values and enters the
unchanged generic Includes algorithm. A recursive owner/order guard pins the
sole entry, argument order and preserved `ToObject`/length/indexed-Get/
`SameValueZero` sequence. A focused CLI fixture observes ignored third and
spread arguments without letting them affect the search result. The owner and
neighboring `at` structure targets pass `4/4` and `3/3`; the new and existing
exact CLI witnesses pass `2/2`. Direct formatting for the five Rust files and
the scoped diff check are green. No Test262 leaf or broad suite was rerun. The
bounded contract is
`docs/rust-rewrite/contracts/array-includes-algorithm-owner.md`.

Static direct `Array.prototype.indexOf` lowering now delegates to
`StandardBuiltinId::ArrayPrototypeIndexOf`. Its deleted one-caller entry also
compiled only the first two source arguments, so later argument effects and
spread iteration could disappear before the search. The shared direct-call
boundary now evaluates and expands the complete argument list left to right;
the unchanged canonical compiler then projects only the first two values and
enters the unchanged generic IndexOf algorithm. A recursive owner/order guard
pins the sole entry, argument order and preserved `ToObject`/length/
`HasProperty`/indexed-Get/strict-equality sequence. A focused CLI fixture
records the ignored third argument and spread iterator before an indexed getter
records the start of the search. The neighboring `at` guard changes only its
stale end marker. The owner and neighboring `at` structure targets pass `4/4`
and `3/3`; the new argument-order and existing TypedArray search CLI controls
pass `2/2`. The canonical compiler and inner algorithm retain their exact
pre-edit hashes, direct formatting for the five touched Rust files is green,
and the scoped diff check is clean. No broad workspace compile or Test262 run
was performed. The bounded contract is
`docs/rust-rewrite/contracts/array-index-of-algorithm-owner.md`.

Static direct `Array.prototype.lastIndexOf` lowering now delegates to
`StandardBuiltinId::ArrayPrototypeLastIndexOf`. Its deleted one-caller entry
also compiled only the first two source arguments, so later argument effects
and spread iteration could disappear before the reverse search. The shared
direct-call boundary now evaluates and expands the complete argument list left
to right; the unchanged canonical compiler then projects only the first two
values, preserves its omitted-`fromIndex` sentinel and enters the unchanged
generic LastIndexOf algorithm. A recursive owner/order guard pins the sole
entry, omission policy, argument order and preserved `ToObject`/length/reverse
retreat/`HasProperty`/indexed-Get/strict-equality sequence. A focused CLI
fixture records the ignored third argument and spread iterator before an
indexed getter records the start of the reverse search; the existing
`fromIndex` fixture remains the omission-versus-explicit-`undefined` control.
The neighboring `at` guard changes only its stale end marker. The owner and
neighboring `at` structure targets pass `4/4` and `3/3`; the new argument-order
and existing `fromIndex` CLI controls pass `2/2`. The canonical compiler and
inner algorithm retain their exact pre-edit hashes, direct formatting for the
five touched Rust files is green, and the scoped diff check is clean. The
shared `cargo xc` checkpoint is green, and three pinned Test262 controls pass
both variants (`6/6`). The bounded contract is
`docs/rust-rewrite/contracts/array-last-index-of-algorithm-owner.md`.

Static direct `Array.prototype.flat` lowering now delegates to
`StandardBuiltinId::ArrayPrototypeFlat`. Its deleted one-caller entry compiled
arguments as standalone expressions before constructing argv, so a spread
argument was rejected outside the call instead of invoking the iterator
protocol. The shared direct-call boundary now propagates receiver and argument
abrupt completions and expands the complete argument list left to right before
the unchanged canonical compiler projects only argument zero as depth. A
recursive owner/order guard pins the sole entry, zero-argc/default-depth policy,
argument order and preserved depth-conversion/source-length/`HasProperty`/
indexed-Get sequence. A focused CLI fixture records an ignored second argument
and custom spread iterator before an indexed getter records the start of
flattening; the existing core and Proxy access-count fixtures remain algorithm
controls. No neighboring structure marker names the deleted wrapper.

Static `flat()` lowering now constructs the local, capability-free
`FlatMethodDispatch::{ArrayCanonical, GenericGetCall}` authority. Only a data
property whose sole target is `ArrayPrototypeFlat` selects the canonical call;
absent, accessor, ambiguous and unknown targets fall through to ordinary
property Get and Call. One exhaustive match owns both states with no kind or
Array heap-shape shortcut, so an own Array `flat` method cannot be bypassed. The
canonical compiler remains pinned at
`c83ffc356528d69e9de4a63e29cb30a4d55d751c662c30810aab2eba9c390c56`, and
`flatMap` remains unchanged.

The recursive owner target passes `5/5`. A new own-method fixture pins
receiver identity, ordinary/spread argument order, custom return value and the
unchanged nested source; the existing argument-order, core and Proxy fixtures
remain canonical controls. The exact own-method, argument-evaluation, core and
Proxy CLI controls each pass `1/1`. The pinned
`non-numeric-depth-should-not-throw.js`, `proxy-access-count.js`,
`positive-infinity.js` and call-expression `spread-mult-iter.js` leaves each
pass `2/2`, for `8/8` Wasm-AOT executions with every failure bucket at zero.
The shared `cargo xc` checkpoint is green. The bounded contract is
`docs/rust-rewrite/contracts/array-flat-algorithm-owner.md`.

The Array arm of static `Array.prototype.flatMap` lowering now delegates to
`StandardBuiltinId::ArrayPrototypeFlatMap`. Its deleted one-caller entry
compiled arguments as standalone expressions before constructing argv, so a
spread argument was rejected outside the call instead of invoking the iterator
protocol. The shared direct-call boundary now propagates receiver and argument
abrupt completions and expands the complete argument list left to right before
the unchanged canonical compiler projects mapper and optional `thisArg`. A
recursive owner/order guard pins the unchanged Array/Iterator classification,
both destinations, sole Array entry, argument order and preserved receiver-
conversion/length/`HasProperty`/indexed-Get/mapper-Call sequence. A focused CLI
fixture records an ignored third argument and custom spread iterator before an
indexed getter and mapper record FlatMap execution; the existing core and Proxy
access-count fixtures remain algorithm controls. Existing structure guards
bound the canonical compiler rather than the deleted wrapper, so no marker
changes. On 2026-08-28, the recursive owner target passed `4/4`, and the exact
new argument-evaluation, core and Proxy access-count CLI tests each passed
`1/1` against the Wasm backend. The canonical compiler source hash remained
`009ab7510a4d965f1db3ff83df63ed3b1739ae9c137d878c9148ee68801c5761`, and the
deleted wrapper has zero Rust source occurrences. The shared `cargo xc`
checkpoint is green, and three pinned Test262 controls pass all five generated
variants (`5/5`). The bounded contract is
`docs/rust-rewrite/contracts/array-flat-map-algorithm-owner.md`.

The static direct `at` branch now delegates its complete argument list to
`StandardBuiltinId::TypedArrayPrototypeAt`, preserving its existing strict
TypedArray receiver policy while deleting the one-caller
`emit_array_at_method_call`. That former entry compiled only argument zero, so
later argument expressions were skipped and a later spread never invoked its
iterator protocol. The shared direct-call boundary now evaluates and expands
all arguments before the unchanged canonical compiler projects the relative
index. A recursive owner/order guard pins the strict standard entry, complete
arguments, absence of the deleted owner, policy selection and preserved
TypedArray-witness/index-coercion/indexed-read order. A focused CLI fixture
records an ignored second expression and custom spread before index coercion;
the existing runtime-kinds fixture remains the receiver-policy control. The
CopyWithin structure guard changes only its stale end marker, and the existing
receiver-policy evidence now pins the two standard policy constructors plus
direct selection of the strict entry. On 2026-08-28, the recursive owner target
passed `4/4`, the receiver-policy and CopyWithin targets each passed `3/3`, and
the exact new argument-evaluation and existing runtime-kinds CLI tests each
passed `1/1` against the Wasm backend. The canonical compiler source hash
remained `7e4346ef5dac8e59cf58a832157a442c5dc3315e55de56a4b5f601c53aafd33b`,
and the deleted wrapper has zero Rust source occurrences. The shared `cargo xc`
checkpoint is green. The pinned `index-argument-tointeger.js`,
`coerced-index-resize.js` and `spread-mult-iter.js` controls pass all `6/6`
sloppy/strict Wasm-AOT executions with every failure bucket at zero. No broader
Array or Test262 refresh was performed. The bounded contract is
`docs/rust-rewrite/contracts/array-at-algorithm-owner.md`.

The Array arm of static `Array.prototype.map` lowering now delegates its
complete argument list to `StandardBuiltinId::ArrayPrototypeMap`. Its deleted
one-caller entry compiled arguments as standalone expressions before
constructing the call, so a spread argument was rejected outside the call
instead of invoking the iterator protocol. The shared direct-call boundary now
evaluates and expands all arguments before the unchanged canonical compiler
projects mapper and optional `thisArg`. A recursive owner/order guard pins the
unchanged Array/Iterator receiver classification, both Iterator destinations,
sole Array entry, complete arguments and preserved receiver-conversion/
`HasProperty`/indexed-Get/mapper-Call order. A focused CLI fixture records an
ignored third argument and custom spread before an indexed getter and mapper;
the existing Map core fixture remains the callback and sparse-array control.
Existing guards use canonical compiler boundaries, so no marker changes.
On 2026-08-28, the recursive owner target passed `4/4`, and the exact new
argument-evaluation and existing Map core CLI tests each passed `1/1` against
the Wasm backend. The canonical compiler source hash remained
`6aab327d7a4ae85907a93eebe0acd0b7c88529f0114b1197db526633d9b72b32`, and the
deleted wrapper has zero Rust source occurrences. The shared `cargo xc`
checkpoint is green. The pinned `create-proxy.js`,
`callbackfn-resize-arraybuffer.js` and `spread-mult-iter.js` controls pass all
`6/6` sloppy/strict Wasm-AOT executions with every failure bucket at zero. No
broader Array or Test262 refresh was performed. The bounded contract is
`docs/rust-rewrite/contracts/array-map-algorithm-owner.md`.

The Array arm of static `Array.prototype.every` lowering now delegates its
complete argument list to `StandardBuiltinId::ArrayPrototypeEvery`. Its deleted
one-caller entry compiled arguments as standalone expressions before
constructing the call, so a spread argument was rejected outside the call
instead of invoking the iterator protocol. The shared direct-call boundary now
evaluates and expands all arguments before the unchanged canonical compiler
projects predicate and optional `thisArg`. A recursive owner/order guard pins
the unchanged Array/Iterator receiver classification, both Iterator
destinations, sole Array entry, complete arguments and preserved receiver-
conversion/`HasProperty`/indexed-Get/predicate-Call/truthiness order. A focused
CLI fixture records an ignored third argument and custom spread before an
indexed getter and predicate; the existing Every core fixture remains the
generic-receiver and short-circuit control. Existing quantifier-family guards
use canonical compiler boundaries, so no marker changes. Focused and broader
verification is recorded here. On 2026-08-28, the recursive owner target passed
`4/4`, and the exact new argument-evaluation and existing
Every core CLI tests each passed `1/1` against the Wasm backend. The canonical
compiler source hash remained
`806b26541d7a713834383c191ffb5377f3dc43366d87454dcbf9989f6f0b4cff`, and the
deleted wrapper has zero Rust source occurrences. The shared `cargo xc`
checkpoint is green. The pinned `callbackfn-resize-arraybuffer.js`,
`resizable-buffer-grow-mid-iteration.js` and `spread-mult-iter.js` controls pass
all `6/6` sloppy/strict Wasm-AOT executions with every failure bucket at zero.
No broader Array or Test262 refresh was performed. The bounded contract is
`docs/rust-rewrite/contracts/array-every-algorithm-owner.md`.

The Array arm of static `Array.prototype.some` lowering now delegates its
complete argument list to `StandardBuiltinId::ArrayPrototypeSome`. Its deleted
one-caller entry compiled arguments as standalone expressions before
constructing the call, so a spread argument was rejected outside the call
instead of invoking the iterator protocol. The shared direct-call boundary now
evaluates and expands all arguments before the unchanged canonical compiler
projects predicate and optional `thisArg`. A recursive owner/order guard pins
the unchanged Array/Iterator receiver classification, both Iterator
destinations, sole Array entry, complete arguments and preserved receiver-
conversion/`HasProperty`/indexed-Get/predicate-Call/truthiness order. A focused
CLI fixture records an ignored third argument and custom spread before an
indexed getter and predicate; the existing Some core fixture remains the
generic-receiver and short-circuit control. Existing quantifier-family guards
use canonical compiler boundaries, so no marker changes. Focused and broader
verification remains at the shared checkpoint. On 2026-08-28, the recursive
owner target passed `4/4`, and the exact new argument-evaluation and existing
Some core CLI tests each passed `1/1` against the Wasm backend. The canonical
compiler source hash remained
`5301cd10772a6e9b71783b283533b5cb77889d84b87f069b61d4fb113cda0b7d`, and the
deleted wrapper has zero Rust source occurrences. The bounded contract is
`docs/rust-rewrite/contracts/array-some-algorithm-owner.md`. The shared
`cargo xc`, workspace formatting, diff, module-boundary and task-plan checks
are green. The pinned `callbackfn-resize-arraybuffer.js`,
`resizable-buffer-shrink-mid-iteration.js` and `spread-mult-iter.js` controls
pass all `6/6` sloppy/strict Wasm-AOT executions with every failure bucket at
zero.

The Array arm of static `Array.prototype.filter` lowering now delegates its
complete argument list to `StandardBuiltinId::ArrayPrototypeFilter`. Its
deleted one-caller entry selected the same builtin and already used complete
argv construction, so this is a source-equivalent single-owner closure rather
than a JavaScript behavior change. A recursive owner/order guard pins the
unchanged Array/Iterator receiver classification, both Iterator destinations,
sole Array entry, complete arguments and preserved receiver-conversion/
`HasProperty`/indexed-Get/predicate-Call/truthiness/target-write order. The
existing Filter core fixture remains the finite generic-receiver, sparse-array,
callback and result control. Existing guards use canonical compiler boundaries,
so no marker changes; the unrelated dead ForEach emitter is outside this lane.
On 2026-08-28, the recursive owner target passed `4/4`, and the exact existing
Filter core CLI control passed `1/1` against the Wasm backend. The canonical
compiler source hash remained
`1f76b4049e22ebd399898021a726a09231429085da68490dc69b5e4339349edd`, and the
deleted wrapper has zero Rust source occurrences. The shared `cargo xc` and
workspace hygiene gates pass. The pinned Proxy species, non-extensible target
and generic spread controls pass all `6/6` sloppy/strict Wasm-AOT executions
with every failure bucket at zero. The bounded contract is
`docs/rust-rewrite/contracts/array-filter-algorithm-owner.md`.

The unused `emit_array_for_each_method_call` duplicate algorithm has been
deleted. It had no caller and had drifted from the live standard ForEach owner,
so keeping it allowed a stale receiver, callability and callback-call path to be
reactivated without changing the canonical compiler. A recursive ownership
guard now proves the deleted symbol has no Rust source occurrence, pins the one
canonical `compile_array_like_for_each_builtin` owner and its Array-like/strict
TypedArray standard producers, and preserves the distinct Iterator ForEach
branch. The existing callback-receiver guard now ends at the next live Array
emitter instead of the deleted function. The resizable TypedArray ForEach CLI
fixture remains the exact finite behavior control because this dead-owner
closure changes no JavaScript behavior. On 2026-08-28, the recursive owner and
existing callback-receiver targets each passed `4/4`, and the exact resizable
TypedArray ForEach CLI control passed `1/1` against the Wasm backend. The
canonical compiler source is pinned at
`52d8982bbef8b3a99ce51a870919b604394773948aa1944d3f21e939a7aa15fb`, and the
deleted emitter has zero Rust source occurrences. The shared `cargo xc` and
workspace hygiene gates pass. The pinned Array resize, Array shrink and
TypedArray resize controls pass all `6/6` sloppy/strict Wasm-AOT executions
with every failure bucket at zero. The bounded contract is
`docs/rust-rewrite/contracts/array-for-each-algorithm-owner.md`.

The unreachable specialized Splice subgraph has been removed:
`emit_array_splice_insert_method_call` had no external caller and its only
internal edge selected `emit_array_splice_delete_one_method_call`. Those two
functions admitted only partial static Array cases and could drift from the
live standard algorithm without affecting its compiler.

Static `splice()` lowering now constructs the local, capability-free
`SpliceMethodDispatch::{ArrayCanonical, GenericGetCall}` authority. Only a data
property whose sole target is `ArrayPrototypeSplice` delegates the receiver and
complete argument list through the shared direct-call boundary; absent,
accessor, ambiguous and unknown targets fall through to ordinary property Get
and Call. One exhaustive match owns both states with no kind or Array heap-shape
shortcut, so an own Array `splice` method cannot be bypassed. The unchanged
`compile_array_prototype_splice_builtin` remains the sole standard algorithm
owner, pinned at
`7236e422756416048ad4668122d118af0f83b7a49ed27c913eb4941d29972394`, and the
separate `spliceFromArray` extension remains live.

A five-test recursive owner/order guard pins both deleted symbols at zero source
occurrences, the closed dispatch, the direct and standard producers, canonical
observable operation order, and `spliceFromArray`. A new own-method fixture pins
receiver identity, ordinary/spread argument order, custom return value and
unchanged elements; the existing Find core fixture remains the canonical
mutation control. At the shared checkpoint, the recursive structure target
passed `5/5`, the exact own-Splice and existing canonical Find-core CLI controls
each passed `1/1`, and the pinned `called_with_one_argument.js`,
`property-traps-order-with-species.js` and `create-proxy.js` leaves each passed
`2/2`, for `6/6` Wasm-AOT executions. The shared `cargo xc` checkpoint is green.
The bounded contract is
`docs/rust-rewrite/contracts/array-splice-algorithm-owner.md`.

The Array arm of static `Array.prototype.find` lowering now calls the shared
direct builtin boundary without the redundant one-caller
`emit_array_find_method_call`. The deleted wrapper only forwarded the same
builtin, label, receiver, complete arguments and destination, so this is a
source-equivalent ownership closure rather than a JavaScript behavior change.
A recursive owner/order guard pins the unchanged Array, strict TypedArray and
both Iterator destinations, sole Array entry, complete argument forwarding,
shared receiver/argument/call order and the canonical receiver-conversion/
predicate-validation/indexed-Get/predicate-Call/truthiness/projection order.
The existing Find core fixture remains the finite sparse, borrowed TypedArray
and callable-Proxy control. The existing FindViaPredicate guard drops only its
stale retained-wrapper assertion. On 2026-08-28, the recursive owner target
passed `4/4`, the existing FindViaPredicate target passed `5/5`, and the exact
Find core CLI control passed `1/1` against the Wasm backend. The canonical
FindViaPredicate module source remained
`f3785b16f21f209331fdbb16888c6752afdcfda87eb570dbbd3211349b848184`, and the
deleted wrapper has zero Rust source occurrences. The pinned
`callbackfn-resize-arraybuffer.js`, `predicate-call-this-strict.js` and
`return-abrupt-from-this-length.js` controls pass all `5/5` Wasm-AOT
executions with zero failures. The shared `cargo xc`, formatting, diff,
module-boundary and task-plan checks are green. The bounded contract is
`docs/rust-rewrite/contracts/array-find-algorithm-owner.md`.

Static `Array.prototype.findIndex` lowering now calls the shared direct builtin
boundary without the redundant one-caller
`emit_array_find_index_method_call`. The deleted wrapper only forwarded the
same builtin, label, receiver, complete arguments and destination. The strict
TypedArray shape branch remains first and unchanged; every remaining generic
receiver continues to select `StandardBuiltinId::ArrayPrototypeFindIndex`. A
recursive owner/order guard pins both branches, sole generic entry, complete
argument forwarding, shared receiver/argument/call order and the canonical
receiver-conversion/predicate-validation/indexed-Get/predicate-Call/truthiness/
index-projection order. The existing Find core fixture remains the finite
generic, `thisArg`, callable-Proxy and Proxy-error control. The existing
FindViaPredicate guard drops only its stale retained-wrapper assertion. On
2026-08-28, the recursive owner target passed `4/4`, the canonical
FindViaPredicate module source remained
`f3785b16f21f209331fdbb16888c6752afdcfda87eb570dbbd3211349b848184`, and the
deleted wrapper has zero Rust source occurrences. The existing FindViaPredicate
target passes `5/5`, and the exact Find core CLI control passes `1/1`. The
pinned `callbackfn-resize-arraybuffer.js`, `predicate-call-this-strict.js` and
`return-abrupt-from-this-length.js` controls pass all `5/5` Wasm-AOT executions
with every failure bucket at zero. The shared `cargo xc`, formatting, diff,
module-boundary and task-plan checks are green. The bounded contract is
`docs/rust-rewrite/contracts/array-find-index-algorithm-owner.md`.

Static `Array.prototype.findLast` lowering now calls the shared direct builtin
boundary without the redundant one-caller
`emit_array_find_last_method_call`. The deleted wrapper only forwarded the same
builtin, label, receiver, complete arguments and destination. The strict
TypedArray shape branch remains first and unchanged; every remaining generic
receiver continues to select `StandardBuiltinId::ArrayPrototypeFindLast`. A
recursive owner/order guard pins both branches, sole generic entry, complete
argument forwarding, shared receiver/argument/call order and the canonical
receiver-conversion/predicate-validation/reverse-initialization/indexed-Get/
predicate-Call/truthiness/value-projection/reverse-advance order. The existing
FindLast core fixture remains the finite reverse, borrowed TypedArray,
callable-Proxy and Proxy-error control. The existing FindViaPredicate guard
drops only its stale retained-wrapper assertion. On 2026-08-28, the recursive
owner target passed `4/4`, the canonical FindViaPredicate module source remained
`f3785b16f21f209331fdbb16888c6752afdcfda87eb570dbbd3211349b848184`, and the
deleted wrapper has zero Rust source occurrences. The existing FindViaPredicate
target passes `5/5`, and the exact FindLast core CLI control passes `1/1`. The
pinned `callbackfn-resize-arraybuffer.js`, `predicate-call-this-strict.js` and
`return-abrupt-from-this-length.js` controls pass all `5/5` Wasm-AOT executions
with every failure bucket at zero. The shared `cargo xc`, formatting, diff,
module-boundary and task-plan checks are green. The bounded contract is
`docs/rust-rewrite/contracts/array-find-last-algorithm-owner.md`.

Static `Array.prototype.findLastIndex` lowering now calls the shared direct
builtin boundary without the final redundant Find-family forwarding owner,
`emit_array_find_last_index_method_call`. The deleted wrapper only forwarded
the same builtin, label, receiver, complete arguments and destination. The
strict TypedArray shape branch remains first and unchanged; every remaining
generic receiver continues to select
`StandardBuiltinId::ArrayPrototypeFindLastIndex`. A recursive owner/order guard
pins both branches, sole generic entry, complete argument forwarding, shared
receiver/argument/call order and the canonical receiver-conversion/predicate-
validation/reverse-initialization/indexed-Get/predicate-Call/truthiness/index-
projection/reverse-advance order. The existing FindLast core fixture remains
the finite reverse-index, `thisArg`, callable-Proxy and Proxy-error control. The
existing FindViaPredicate guard drops its final stale method-specific wrapper
row. On 2026-08-28, the recursive owner target passed `4/4`, and the existing
FindViaPredicate target passed `5/5`. The canonical FindViaPredicate module
source remained
`f3785b16f21f209331fdbb16888c6752afdcfda87eb570dbbd3211349b848184`, and the
deleted wrapper has zero Rust source occurrences. Targeted formatting and the
scoped diff check are green. The exact existing FindLast core CLI control passes
`1/1`. The pinned `callbackfn-resize-arraybuffer.js`,
`predicate-call-this-strict.js` and `return-abrupt-from-this-length.js`
FindLastIndex controls pass all `5/5` Wasm-AOT executions with every failure
bucket at zero. The shared `cargo xc`, formatting, diff, module-boundary and
task-plan checks are green. The bounded contract is
`docs/rust-rewrite/contracts/array-find-last-index-algorithm-owner.md`.

Static `Array.prototype.concat` lowering now calls the shared direct builtin
boundary without the redundant one-caller `emit_array_concat_method_call`.
The deleted wrapper selected the same Array builtin and independently owned
receiver evaluation, an internal function-object materialization, complete
argument-vector construction and generic handle dispatch. The earlier String
receiver and statically resolved `String.prototype.concat` branch remains
first and unchanged; every generic fallback continues to select
`StandardBuiltinId::ArrayPrototypeConcat`. A recursive owner/order guard pins
both branches, complete receiver and argument forwarding, the sole canonical
Concat compiler, shared receiver/argument/call order and the preserved receiver
conversion/species construction/spreadability/length/indexed-presence/indexed-
Get/target-write order. The existing Concat core fixture remains the finite
zero-, Array-, ordinary-object-, multiple-argument and sparse-result control.
The existing Filter ownership guard now uses the shared direct boundary as its
end marker. On 2026-08-28, the recursive owner target passed `5/5` and the
neighboring Filter owner target passed `4/4`. The canonical Concat compiler remained
`fe301d8165ba41828b9e742f7b19a1e49fabcac9dc35c625bb0a84d4ff29a8e9`, and the
deleted wrapper has zero Rust source occurrences. Targeted formatting and the
scoped diff check are green. The exact existing Concat core CLI control passes
`1/1`. The pinned `call-with-boolean.js`,
`is-concat-spreadable-get-order.js` and
`Array.prototype.concat_small-typed-array.js` controls pass all `6/6`
Wasm-AOT executions with every failure bucket at zero. The shared `cargo xc`,
formatting, diff, module-boundary and task-plan checks are green. The bounded contract is
`docs/rust-rewrite/contracts/array-concat-algorithm-owner.md`.

Static `Array.prototype.push` lowering now delegates its complete argument
list to `StandardBuiltinId::ArrayPrototypePush` and the redundant specialized
`emit_array_push_method_call` is gone. Before that ownership closure, the
standard body emitted two fixed eight-case argument expansions, so dynamically
obtained or borrowed Push calls silently ignored arguments after index seven;
the specialized static owner also rejected spread arguments outside the call
boundary. The canonical dense Array and generic receiver paths now each use a
runtime argc loop and dynamic argv read. The dense path preserves inherited-
index setters, maximum Array length and non-writable length failure order. The
generic path preserves receiver conversion, observable length Get/ToLength,
the safe-integer pre-write guard, sequential Sets and final length Set. The
existing compile-time Array classification remains unchanged, while the shared
boundary now guarantees complete left-to-right argument and spread evaluation
before either path begins. A recursive owner/order guard pins the sole standard
owner, both dynamic loops, absence of the fixed cap, shared call order and both
receiver-specific write/length sequences. The new focused CLI fixture supplies
eight direct values, three custom-iterable spread values and a final twelfth
argument, proving expansion completes before the target mutates and every value
is appended. On 2026-08-28, the recursive Push owner target passed `4/4`. The
affected Proxy-set Realm and Concat owner targets each passed `4/4`, the String
code-unit boundary target passed `6/6`, and the FindLastIndex owner boundary
target passed `4/4`. The canonical Push arm is pinned at
`c954cac5f939488f2cb6d07e5b9d70fba3224d33ec57c708570e6759856cf6c8`, the
deleted wrapper has zero Rust source occurrences, and the fixed eight-case form
has zero occurrences in the canonical arm. The exact new CLI control passes
`1/1`. The pinned `set-length-array-length-is-non-writable.js`,
`length-near-integer-limit.js` and `throws-if-integer-limit-exceeded.js`
controls pass all `6/6` Wasm-AOT executions with every failure bucket at zero.
The shared `cargo xc`, formatting, diff, module-boundary and task-plan checks
are green. The bounded contract is
`docs/rust-rewrite/contracts/array-push-algorithm-owner.md`.

## Objective

Complete Array exotic object behavior and every pinned Array constructor/prototype method using general internal operations. Retire focused static Test262 materializations as each family becomes fully semantic.

## Array exotic object

Implement and validate:

- `ArrayCreate`, initial prototype selection and maximum length handling;
- `ArraySetLength`, including descriptor validation, truncation order, deletion failures and rollback;
- `[[DefineOwnProperty]]` for canonical array indexes and `length`;
- dense-to-sparse transitions without changing key ordering or hole semantics;
- inherited indexes, accessors, non-writable length and non-extensible arrays;
- canonical index boundaries around `2^32 - 1` and large named numeric keys;
- `ownKeys` ordering and interaction with symbols/proxies.

Dense storage is an optimization only. Every observable operation must agree with the exotic protocol.

## Constructors and species

Complete `Array`, `Array.of`, `Array.from`, `Array.fromAsync` if present, `Array.isArray`, `@@species`, subclass construction and cross-realm behavior. Constructors must use iterator closing, mapping call order and custom `this` semantics.

## Prototype families

Implement the full pinned API, grouped so separate PRs can land within this task:

- mutators: `push`, `pop`, `shift`, `unshift`, `splice`, `copyWithin`, `fill`, `reverse`, `sort`;
- creators: `concat`, `slice`, `toSpliced`, `toReversed`, `toSorted`, `with`, `flat`, `flatMap`;
- search/access: `at`, `includes`, `indexOf`, `lastIndexOf`, `find*`;
- iteration/callback: `forEach`, `map`, `filter`, `every`, `some`, `reduce`, `reduceRight`;
- string/locale: `join`, `toString`, `toLocaleString`;
- iterators: `keys`, `values`, `entries`, `@@iterator`.

## Correctness matrix

Every method must be exercised against:

- ordinary arrays, sparse arrays and arrays with inherited indexes;
- generic array-like receivers and primitive receivers where allowed;
- proxies/accessors with observable operation order;
- subclasses, species constructors and cross-realm constructors;
- typed-array borrowed receivers where generic;
- mutation during iteration and length snapshots;
- abrupt callbacks/coercions and iterator closing.

Avoid method-specific duplicates of `LengthOfArrayLike`, `HasProperty`, `Get`, callback invocation or species logic.

## Acceptance criteria

- `built-ins/Array` and `built-ins/Array/prototype` are fully green for the pin.
- No path-specific materializer remains for covered Array tests.
- Array index/length descriptor edge cases pass.
- Sort is stable and obeys comparator/coercion/holes/undefined semantics.
- Sparse arrays do not cause loops proportional to impossible maximum lengths when the spec permits key-based optimization; observable access order remains correct.
- Species, proxy, inherited-index and cross-realm tests pass across creator methods.
- Adjacent TypedArray generic-borrow tests do not regress.

## Required tests

```sh
cargo test -p lila-aot-wasm array_ --quiet
cargo test -p lila-cli wasm_array --quiet
./target/debug/lila test262 run built-ins/Array --execution-backend wasm --timeout-ms 180000 --threads 8
```

During development use method-level filters and deterministic shards. Before closing, run the entire Array tree and all local `wasm_array_*` fixtures.


## 2026-09-06: Generic flatMap observable-operation closure

The canonical Wasm-AOT flatMap compiler now delegates ToObject/LengthOfArrayLike,
IsCallable, ArraySpeciesCreate, IsArray, HasProperty, Get and target property
creation to shared operation owners. Length is observable and captured before
mapper validation or species side effects. Missing callbacks no longer bypass
length getters, huge numeric lengths use bounded ToLength, and TypedArray private
extents no longer bypass own or inherited length properties.

The source and mapped-array loops retain live property observations, nested Proxy
traps, sparse behavior, one-level flattening and abrupt-completion order. A single
append owner guards the maximum safe integer bound before data-property creation.
The three affected structural targets track this ownership change; existing CLI
fixture bodies remain unchanged. Seventeen new engine regression programs select
WasmAot explicitly. CI includes a nonempty compiled-inventory check and the entire
pinned flatMap subtree, and repairs the previously stale pinned-agent test filter.

[The flatMap follow-up](../docs/rust-rewrite/aot-flat-map.md) gives the verification
commands and follow-on priorities. This is not closure of T16 or T26, not a change
to the Test262 denominator, and not a claim that the full current-pin suite is
green. Runtime results must be attached to the exact tested PR revision.
