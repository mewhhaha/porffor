# Lila

Lila—Swedish for “purple”—is a Rust JavaScript-to-Wasm AOT compiler, library,
CLI, and conformance harness, formerly developed as Porffor. It is still a
research project and not ready for general JavaScript workloads.

The public project and all current Rust packages, commands, environment
variables, cache paths, diagnostics and host ABI names use the Lila identity.
The GitHub repository URL and current DNS name retain their external locators
until those resources are moved.

The product path is direct JavaScript compilation. User programs must go through
parse, early errors, spec-shaped IR, lowering IR, and real Wasm codegen. Lila
does not count "compile a JavaScript interpreter or VM to Wasm and feed source
into it" as success.

The older JavaScript implementation was retired from the working tree at Git
commit `2107dfe9ad58c730e3d19b0cc1c73ed4390602f8`. History remains available for
archaeology; it is not a development surface or an oracle. The Rust workspace
and its `lila` CLI are the only current product implementation.

Async-generator ordinary property assignments across `yield` and `yield*` now
use the shared suspended Reference path. The base and raw key survive suspension;
normal resumption performs key conversion and the strictness-aware write, while
abrupt resumption bypasses it. This is a focused backend capability, not a claim
of complete generator or Test262 conformance. See
[the suspended Reference follow-up](docs/rust-rewrite/aot-suspended-references.md).

Async-generator `for await` loops can also retain captured `let`/`const` head
bindings across body yields. Closures and the resumed body share the same cell;
each iteration gets a fresh cell, and iterator closing runs after the parent
environment is restored. Additional materialized body scopes and nested
for-await remain separate work. See
[the captured iteration follow-up](docs/rust-rewrite/aot-captured-for-await.md).

Date parsing now uses a shared bounded cursor rather than epoch-only display
string recognition. The direct Wasm path handles reduced ISO date-time forms,
valid `24:00` rollover, and the canonical UTC display formats emitted by Lila.
The focused Wasmtime regression target and remaining UTC/time-zone limitations
are described in [the Date parsing follow-up](docs/rust-rewrite/aot-date-parsing.md).
This does not change the published conformance counts below.

Array `flatMap` now uses shared observable length, callability, species and
property operations. Its source length is captured before mapper validation and
species side effects; sparse properties and Proxy traps remain live during
mapping. TypedArray receivers use ordinary `length` access rather than a private
extent shortcut. The focused Wasm-AOT regressions, pinned subtree command and
remaining work are documented in
[the flatMap conformance follow-up](docs/rust-rewrite/aot-flat-map.md).
This does not change the published full-suite status or denominator.

Array `map`, `filter`, `every` and `some` now share one observable callback
iteration compiler. It preserves the initial length, live sparse/Proxy reads,
callable Proxy callbacks and borrowed TypedArray length overrides. Map and filter
share species construction and data-property definition; quantifiers short-circuit
without species effects. See [the callback iteration follow-up](docs/rust-rewrite/aot-array-callback-iteration.md)
for regression commands, evidence boundaries and remaining work. The generated
full-suite status below is unchanged.

## Current Status
<!-- lila-status:start -->
Rust rewrite status must be read in layers, not one vanity number:
- Fake wasm-safe Test262 subset: `187/187` green
- Fake full Rust rewrite suite: `190/190` green
- Full pinned real Test262 for Rust rewrite: **not green / current pinned aggregate not yet fully republished**
- Current real-suite pin: `ecma262=ecma262-current-draft` `test262=e9d582d6b8b13afc5ba9a676664741592b5c7f69`
- Last complete cached `spec-exec` publish is stale for the current pin and must not be reported as current progress.

As of `2026-04-30`, Rust Wasm-AOT path is at 100% of repo fake coverage, not 100% ECMAScript. Project is still off literal 100% until the full pinned real Test262 run is green for Rust path and the status artifact is republished.

Status refresh commands:
- `cargo test -p lila-engine --quiet`
- `cargo test -p lila-cli --quiet`
- `./target/debug/lila test262 run language/wasm/pass --suite-root crates/lila-test262/tests/fixtures/fake_test262/vendor/test262 --execution-backend wasm`
- `./target/debug/lila test262 run --suite-root crates/lila-test262/tests/fixtures/fake_test262/vendor/test262`
- `./scripts/publish-real-status-low-ram.sh wasm-aot codex-published-real`

When counts move, update this block in same change. Do not claim full Test262 `100%` from fake-suite numbers.
<!-- lila-status:end -->

The generated block above is preserved as the last published path-only
checkpoint, not current execution-aware proof. Its fake full-suite `190/190`
means 190 physical paths; the current denominator is statically derived as 191
executions from those files because the one unflagged parse-negative now runs
in both sloppy and strict Script mode. Both fake-suite execution-aware reruns
and the pinned real Test262 refresh remain pending until the centralized
Cargo/Test262 verification lease.

Focused Wasm-AOT progress verified after the last aggregate publish is recorded
under [Current Capabilities](#current-capabilities). The generated status block
above stays conservative until a full pinned real-suite publish is refreshed.

## Implementation Progress

As of `2026-08-13`, the Rust rewrite has 30 epic-level tasks:

- `4` complete: the repository operating contract (`T00`), interpreter
  quarantine and Wasm-AOT product default (`T27`), retirement of the legacy
  JavaScript product (`T28`), and the verified Lila identity cutover (`T29`);
- `24` in progress with substantial implementation but unmet closure criteria,
  including the deterministic Intl architecture plus its first consumed,
  provider-backed locale canonicalization operation (`T23`);
- `1` with policy, typed accounting, the no-source `%eval%` branch and bounded
  intrinsic `Function.prototype.call` forwarding implemented while textual
  compilation remains open: dynamic source evaluation (`T13`);
- `1` blocked final gate: zero-failure current-pin Wasm-AOT conformance (`T26`).

These are closure counts, not an estimate that “2/30 of JavaScript” is
implemented. The task epics differ greatly in size, and many in-progress lanes
already contain broad working implementations.

What is already in place:

- direct JS-to-Wasm compilation is the product default;
- the Boa interpreter is feature-gated as a developer-only oracle and excluded
  from default product dependency graphs;
- both repository fake suites were green at the last path-only checkpoint;
  execution-aware refreshes remain pending;
- shared IR, lowering, operation, ABI, heap, object, control-flow and builtin
  modules exist;
- substantial focused support exists across functions/classes, objects,
  arrays, promises/async execution, generators/iterators, binary data,
  collections, strings, RegExp, numbers/BigInt/JSON, Date/Temporal and host
  builtins.
- `AsyncGeneratorExpression` parameter/body `Contains super` failures now have
  the closed `E_ASYNC_GENERATOR_EXPRESSION_CONTAINS_SUPER` identity across
  Script, Module and retained module-graph parsing. The exact four-file/eight-
  execution Test262 replay remains pending, so this is not a new pass claim.
- class auto-accessors compile on the direct Wasm path for public/private and
  instance/static placement with hidden backing fields; the focused raw grammar
  cohort and public staging semantics are green, while the private staging
  file's eval-based duplicate-name checks remain dynamic-source debt;
- `%DisposableStack%` has a real constructor, distinct synchronous brand and
  the complete typed `use`/`adopt`/`defer`/`move`/`dispose`/`disposed`
  lifecycle, including exact `Symbol.dispose` identity, LIFO disposal and
  `SuppressedError` folding; the focused current-SHA Wasm-AOT checkpoint is
  green, while the complete 76-file lifecycle sweep remains pending.

The largest remaining closure work is:

- publish a complete current-pin Wasm-AOT Test262 aggregate and generated
  failure backlog;
- remove the large Test262 path/source materialization layer—the shortcut audit
  now passes against an exact 186-entry token-aware baseline: 32 legitimate
  harness adaptations, 105 diagnostic instrumentation sites and 49 semantic
  shortcuts. The removal-task summary assigns 35 entries to T03 and leaves
  T17 at 80; the T03 removal bucket contains 32 legitimate adaptations, two
  diagnostic guards and one semantic shortcut. This census retains the
  audit-coverage rebaseline: the scanner covers multiline
  expressions, same-line multiplicity, exact rewrite calls, source contract
  guards and normalized `match`/`matches!` selector tables. Reduced assertion
  selection is now gone. The final twelve T18 semantic observations are gone,
  leaving T18 with zero shortcut ownership. Its five physical String cases
  retain their exact vendored sources across ten sloppy/strict executions. The
  spec-exec oracle passes all `10/10`; the Wasm-AOT product
  path passes `0/10` and reports every execution as typed `Unsupported`. Four
  sources use direct `eval` and require a caller-environment lowering seam; the
  fifth uses the ordinary `Function` constructor and requires a target-Realm
  environment seam. Six adjacent non-dynamic product controls pass all `12/12`
  sloppy/strict Wasm-AOT executions. The older
  `charAt`, `charCodeAt`, `indexOf`, `match` and `slice` leaf-green claims below
  are dated rewrite-backed checkpoints, not current raw-source Wasm-AOT
  results. All 17,540 physical sources and 33,715 executions that
  formerly selected a reduced body now receive the full LocalMerged
  `assert.js`. The typed-array literal contract contains 319 physical sources
  and 622 executions: 296/576 use the full assertion helper and 23/46 explicitly
  omit unused assertion code. The SameValue and CompareArray assertion modes,
  their prelude constants and their source-shape predicates no longer exist.
  The compact typed-array descriptor probe now accepts only the `TypeError`
  raised by a strict write to non-writable `length` or `name`; every other
  setter failure propagates unchanged. Exact Wasm-AOT runs for the
  `copyWithin`, `findLast` and `findLastIndex` `length.js`/`name.js` cases pass
  all `12/12` sloppy/strict executions, and a Proxy-setter regression pins
  non-`TypeError` propagation. The exact `%TypedArray%.prototype.at` helper
  matcher and source guard are also gone. A 15-source/30-execution invariant
  pins unchanged bodies in both Script modes and both prelude profiles: all 13
  typed-array-helper consumers use the complete vendored `testTypedArray.js`,
  three also use the complete configured `propertyHelper.js`, and the two
  resizable-helper cases retain only T13's separately owned static-subclass
  substitution. The rebuilt post-delete leaf passes all `30/30` sloppy/strict
  Wasm-AOT executions, and three exact adjacent controls pass `6/6` with every
  non-success bucket at zero. This includes the formerly generic
  `ArrayBuffer.isView`, typed-array defined-length and `%TypedArray%`
  `@@species`, TypedArray sort/`of`, DataView constructor, ProxyCreate,
  `Error.isError` and staging `flatMap` cohorts.

  The exact `%TypedArray%.prototype.filter` and `map` source matchers and their
  compact prelude consumers are now gone. A shared invariant scans all 84
  physical sources and 168 sloppy/strict executions in each directory. It
  pins the 18 retired matcher contracts, both prelude stores and exact
  materialized bytes: filter has 81 complete `testTypedArray.js` consumers and
  three sources without that include; map has 79 and five. Six metadata
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
  dispatcher to the complete 14,921-byte helper. Three other `every` cases
  remain without that helper; three `some` consumers remain on the typed-array
  literal plan's 12,362-byte split route, and three others remain without it.
  The six `resizableArrayBufferUtils.js` consumers retain T13's static-subclass
  substitution. This removes one T17 semantic shortcut and one diagnostic
  guard, leaves the broad `every`/`some` resizable-buffer admissions unchanged,
  and, at that checkpoint, narrowed the retired iterator/find contract to 41
  paths. The
  rebuilt release CLI passes all 24 exact executions as of `2026-08-31` under
  suite pin `aa55200d1310384c5cf69ea95b2a2ecba457007b`; the surviving
  then-surviving `find/callbackfn-resize.js` split route passed `2/2`, with every non-success
  bucket at zero.

  The `%TypedArray%.prototype.slice` family-prefix selector, exact eight-path
  source matcher and fingerprint guard, and slice-specific compact property
  selector are now gone. The replacement invariant scans all 91 physical
  sources and 182 sloppy/strict executions in both prelude stores and permits
  only 87 complete 14,921-byte `testTypedArray.js` consumers or four sources
  without that helper. Fourteen former compact and eight former intrinsic
  cases now use the full helper; 65 cases were already full. The three metadata
  sources retain complete `propertyHelper.js`, `not-a-constructor.js` retains
  complete `isConstructor.js`, and the four `resizableArrayBufferUtils.js`
  consumers retain T13's exact static-subclass substitution. This removes two
  T17 semantic shortcuts and one diagnostic guard, leaves the broad slice
  resizable-buffer admissions unchanged, and limits shared family-prefix
  compaction to `includes`, `indexOf` and `lastIndexOf`. The rebuilt release CLI
  passes all 44 changed executions as of `2026-08-31` under suite pin
  `aa55200d1310384c5cf69ea95b2a2ecba457007b`; exact surviving-route controls
  pass `6/6`, with every non-success bucket at zero. This is a focused
  post-retirement replay, not a new complete `182/182` execution claim.

  The DataView accessor-metadata
  and accessor wrong-receiver, `ArrayBuffer.isView` typed-array-helper, and
  BigInt-get ToIndex, numeric-setter, typed-array defined-length and
  `%TypedArray%` `@@species`, ArrayBuffer metadata and DataView method metadata cleanups removed
  thirty-three T17 entries after
  their pinned sources passed with full upstream helpers in sloppy and strict
  modes (`18/18`, `18/18`, `10/10`, `8/8`, `16/16`, `2/2`, `8/8`, `30/30`,
  and `84/84`); that checkpoint left 190 T17-owned entries. The TypedArray sort
  value matrix, `TypedArray.of` zero case and eleven borrowed Array callback
  resize cases now preserve their 13 pinned source bodies across both Script
  modes and both prelude stores. An isolated post-delete Wasm-AOT run of those
  exact sources passes all `26/26` sloppy/strict executions with every
  non-success bucket at zero. Deleting their constructor fan-outs, helper
  omission and dispatch paths removed 29 more semantic observations and left
  161 T17 entries, split between 80 semantic shortcuts and 81 diagnostic
  guards at that checkpoint. A direct raw preflight of the eight top-level
  DataView constructor surface sources passed all `16/16` sloppy/strict executions through the
  complete vendored `sta.js`, `assert.js` and declared property or constructor
  helpers. Every execution reported `backend_used: WasmAot`, with all
  non-success buckets at zero. That run established unchanged-source readiness
  before deletion; it is not post-delete production-dispatch evidence. Their
  8x2 materialization invariant pins both prelude stores and all exact source,
  include and provenance bytes. The rebuilt production dispatcher then passes
  the same exact `16/16` cohort with every non-success bucket at zero. Removing
  those eight arms only narrows the surviving DataView constructor selector
  fingerprint, so the inventory counts remain unchanged. The 48 Array prototype
  method metadata sources also pass unchanged
  with the complete property helper in sloppy and strict modes (`96/96`),
  removing four T16 entries and leaving 76 at that checkpoint. The two borrowed
  `Array.prototype.at` resizable-buffer sources also preserve their exact pinned
  bodies in sloppy and strict modes. A direct raw preflight passed all `4/4`
  executions after concatenating complete vendored `sta.js`, `assert.js`,
  `resizableArrayBufferUtils.js` with only T13's owned replacement of the
  dynamic subclass block with three static classes, and the exact source. The
  unmodified helper still reaches the explicit
  Function-constructor AOT-unsupported boundary, so this is scoped pre-delete
  evidence rather than a full-helper or post-delete production-dispatch result.
  A 2x2 invariant pins both materialization stores, the original source suffix
  and the exact helper replacement. After deletion, the rebuilt production
  dispatcher passed the same exact `4/4` cohort with every non-success bucket
  at zero while retaining T13's helper substitution. Deleting the complete Array
  `at` rewrite authority removes three semantic observations, leaving 73 T16
  entries at that checkpoint. The adjacent
  `Array.prototype.includes/resizable-buffer-special-float-values.js` source
  also preserves its pinned body in both Script modes. A separate raw `4/4`
  preflight covered both materialization stores with exact source and prelude
  bytes; every execution reported `backend_used: WasmAot`. The only helper
  change was T13's static-subclass substitution. The unmodified helper still
  stops at the explicit Function-constructor AOT-unsupported boundary, so this
  is not full-helper or post-delete production-dispatch evidence. Removing the
  terminal special-float arm left the other two Array `includes` rewrites and
  their shared dispatcher intact. It deleted one T16 semantic observation,
  leaving 72 T16 entries in the historical 356-entry, 208-semantic checkpoint.
  After deletion, the rebuilt production dispatcher passed the exact source in
  both Script modes (`2/2`) with every failure and non-success bucket at zero.
  The two remaining Array `includes` sources each pass a separate direct raw
  `4/4` preflight across both Script modes and both prelude stores. Every run
  reports `backend_used: WasmAot`; only T13's static-subclass substitution is
  applied. The unmodified helper still stops at the explicit
  Function-constructor AOT-unsupported boundary. The five-source invariant now
  pins the two retired Array `at` bodies and all three retired Array `includes`
  bodies with exact source, mode, prelude and provenance checks, including the
  sole helper substitution. Deleting the complete remaining Array `includes`
  rewrite authority removes three more semantic observations. That historical
  checkpoint had 353 entries, including 205 semantic shortcuts; T16 owned 69
  and T17 owned 161. After deletion, the rebuilt production dispatcher passed
  the exact final two-source cohort in both Script modes (`4/4`) with every
  failure and non-success bucket at zero. The exact
  `built-ins/Array/prototype/map/resizable-buffer.js` source then passed a
  pre-delete raw `4/4` matrix across both Script modes and both prelude stores
  with exact source bytes and only T13's static-subclass helper substitution.
  The unmodified helper still stops at the explicit Function-constructor
  AOT-unsupported boundary, so this is neither full-helper support nor
  post-delete production-dispatch evidence. The expanded six-source invariant
  pins the map source, declared comparison and resizable helpers, and exact
  LocalMerged and vendored-only bytes and origins in both modes. Deleting only
  the map branch from the known-static `for-of` rewrite removes one T17
  semantic observation. The remaining TypedArray accessor authority and shared
  resizable-directory substitutions stay intact. That checkpoint's inventory
  had 352 entries: 35
  legitimate harness adaptations, 113 diagnostic instrumentation sites and
  204 semantic shortcuts. T16 owns 69; T17 owns 160, split between 79 semantic
  shortcuts and 81 diagnostic guards. After deletion, the rebuilt production
  dispatcher passed the exact map source in both Script modes (`2/2`) with
  every failure and non-success bucket at zero. The seven pinned Array
  iteration `resizable-buffer.js` sources for `find`, `findIndex`, `findLast`,
  `findLastIndex`, `every`, `some` and `filter` then passed an exact raw
  `28/28` matrix across both Script modes and both prelude stores. The separate
  `find` proof supplied `4/4`; the sibling proof lanes supplied the remaining
  `24/24`. Every execution reported `backend_used: WasmAot`, preserved the
  pinned test source, and changed only the resizable helper's T13-owned dynamic
  subclass block into three static classes. `filter` declares `compareArray.js`
  and `resizableArrayBufferUtils.js`; the other six declare only the resizable
  helper. The unmodified helper still reaches the explicit Function-constructor
  AOT-unsupported boundary. The expanded thirteen-source invariant pins both
  modes, both stores, exact source and prelude bytes and origins, source suffixes,
  no-rewrite boundaries, and T13 contract membership. Deleting the complete
  handwritten iteration rewrite, its sole dispatcher call and all seven path
  predicates removes eight T16 semantic observations while preserving broad
  per-method resizable admission and the neighboring mid-iteration,
  `toLocaleString` and search rewrite authorities. After deletion, the rebuilt
  production dispatcher passed the exact seven-source cohort in both Script
  modes (`14/14`) with every failure and non-success bucket at zero. That
  historical checkpoint had 344 entries, including 196 semantic shortcuts;
  T16 owned 61. The six pinned Array `reduce` and `reduceRight` resizable-buffer
  sources then passed an exact raw `24/24` matrix across both Script modes and
  both prelude stores. Every execution reported `backend_used: WasmAot`, kept
  the exact source, and applied only T13's static-subclass replacement in
  `resizableArrayBufferUtils.js`; all six sources also retain their declared
  `compareArray.js`. A representative run with the unmodified helper stopped
  at the explicit Function-constructor dynamic-code-generation boundary, so
  this is scoped pre-delete evidence rather than full-helper support. The
  expanded nineteen-source invariant pins exact modes, source and prelude
  bytes, origins, suffixes, no-rewrite boundaries and T13 contract membership.
  Deleting the complete reduce rewrite, its sole dispatcher call, both
  one-caller source builders and the obsolete synthetic rewrite test removes
  six T16 semantic observations while preserving broad reduce admission and
  the neighboring resizable authorities. After deletion, the rebuilt production
  dispatcher passed the exact six-source cohort in both Script modes (`12/12`)
  with every failure and non-success bucket at zero. That historical checkpoint
  had 338 entries, including 190 semantic shortcuts; T16 owned 55. The four
  pinned Array `indexOf` and three pinned Array `lastIndexOf` resizable-buffer
  sources then passed an exact raw `28/28` matrix across both Script modes and
  both prelude stores. Every execution reported `backend_used: WasmAot`, kept
  the exact source and declared `resizableArrayBufferUtils.js`, and applied only
  T13's static-subclass replacement. A representative run with the unmodified
  helper stopped at the explicit Function-constructor dynamic-code-generation
  boundary. Dry review found that the handwritten `lastIndexOf` rewrite had
  also bypassed the feature gate because Array `lastIndexOf/` lacked the broad
  resizable admission already present for `includes/` and `indexOf/`. One closed
  prefix set now admits all three Array search methods, and the admission test
  exhausts that set. The expanded twenty-six-source invariant pins exact modes,
  source and prelude bytes, includes, origins, suffixes, no-rewrite boundaries
  and T13 contract membership. Deleting both complete search rewrites, their two
  dispatcher calls, seven direct path predicates, two obsolete synthetic tests
  and the two now-dead shared prelude/constructor builders removes nine T16
  semantic observations. Consolidating the two prior Array-search diagnostic
  predicates removes one diagnostic observation while preserving neighboring
  mid-iteration and `toLocaleString` authorities and the broad TypedArray search
  admission. After deletion, the rebuilt production dispatcher passed the exact
  seven-source cohort in both Script modes (`14/14`) with every failure and
  non-success bucket at zero. That Array-search retirement checkpoint had 328
  entries: 35 legitimate harness adaptations, 112 diagnostic instrumentation
  sites and 181 semantic shortcuts; T16 owned 45. The fourteen pinned Array
  `every`/`some`/`filter`/`find`/`findIndex`/`findLast`/`findLastIndex`
  grow/shrink-mid-iteration sources then passed a raw `56/56` matrix across
  both Script modes and both prelude stores, split into `24/24` quantifier and
  `32/32` find-family executions. Every run reported `backend_used: WasmAot`,
  preserved the exact source and ordered `compareArray.js` plus
  `resizableArrayBufferUtils.js` includes, and changed only T13's dynamic
  subclass block to its three static classes. A representative run with the
  unmodified helper stopped at the explicit Function-constructor
  dynamic-code-generation boundary. The expanded pinned-source invariant owns
  all fourteen files with exact modes, stores, source and prelude bytes,
  origins, suffixes, no-rewrite boundaries and T13 contract membership.
  Deleting the complete shared rewrite, its sole dispatcher call, the
  one-caller constructor list and the obsolete synthetic rewrite test removes
  its entrypoint and all fifteen direct predicates. Broad resizable admissions
  for all seven Array methods remain, as do the T13 helper contract and the
  neighboring Array values, iterator and `toLocaleString` authorities. After
  deletion, the rebuilt production dispatcher passed the exact fourteen-source
  cohort in both Script modes (`28/28`) with every failure and non-success
  bucket at zero. That historical checkpoint had 312 entries and 165 semantic
  shortcuts; T16 owned 29. The three pinned Array `values` base/grow/shrink
  resizable-buffer sources then passed all `12/12` raw executions across both
  Script modes and both prelude stores. Every run reported
  `backend_used: WasmAot`, kept the exact source and ordered `compareArray.js`
  plus `resizableArrayBufferUtils.js` includes, and changed only T13's dynamic
  subclass definitions to the three static classes. The exact helper
  fingerprint `0x6466_6602_9ee8_9d5d` and case fingerprints
  `0x5e5c_6ead_7b7c_0dda`, `0x3d18_7152_c6ff_a624` and
  `0x60c2_a9ec_1dff_dd03` authorize that replacement; changed helper, path,
  includes or source bytes retain `new Function` and reach the explicit
  Function-constructor dynamic-code-generation boundary. The pinned-source
  invariant now covers all three files with exact modes, stores, source and
  prelude bytes, origins, suffixes, no-rewrite checks and T13 contract
  membership. Deleting both complete Array-values rewrite functions, their two
  sole dispatcher calls and both obsolete synthetic rewrite tests removes two
  entrypoints and three direct predicates. Broad Array-values resizable
  admission, the Array keys/entries iterator paths, T13's helper contract and
  the neighboring `toLocaleString` rewrite remain. That checkpoint's inventory
  had 307 entries: 35 legitimate harness adaptations, 112 diagnostic
  instrumentation sites and 160 semantic shortcuts. T16 owns 24; T17 remains
  at 160, split between 79 semantic shortcuts and 81 diagnostic guards. After
  deletion, the rebuilt production dispatcher passed the exact three-source
  cohort in both Script modes (`6/6`) with every failure and non-success bucket
  at zero. The three pinned Array `toLocaleString` resizable-buffer sources then
  passed an exact raw `12/12` matrix across both Script modes and both prelude
  stores. Every execution reported `backend_used: WasmAot`, preserved the
  pinned source, declared only `resizableArrayBufferUtils.js`, and applied only
  T13's replacement of the dynamic subclass block with three static classes.
  The helper fingerprint `0x6466_6602_9ee8_9d5d` and case fingerprints
  `0x9da9_18f5_d04d_d764`, `0xc380_4490_04ea_5b59` and
  `0x07d1_d14e_3a0b_bb89` admit that one change. Changed helper, path, include or
  source bytes retain `new Function`; a representative unmodified-helper run
  stopped at the explicit Function-constructor dynamic-code-generation
  boundary. The expanded invariant pins all three sources, modes, stores,
  bytes, origins, suffixes, no-rewrite checks and T13 contract memberships.
  Deleting the complete Array `toLocaleString` rewrite, its sole dispatcher call
  and its obsolete synthetic test removes one entrypoint and three direct
  predicates. Broad Array `toLocaleString` resizable admission, its feature-gate
  witness, T13's helper contract, TypedArray `toLocaleString` behavior and the
  neighboring DataView rewrite authorities remain. The pre-retirement baseline
  contained 307 entries, including 160 semantic shortcuts. The regenerated
  source ledger has 303 entries: 35 legitimate harness adaptations, 112
  diagnostic instrumentation sites and 156 semantic shortcuts. T16 owns 24;
  T17 remains at 160 and T18 owns 12. After deletion, the rebuilt production
  dispatcher passed the exact three-source cohort in both Script modes (`6/6`)
  with every failure and non-success bucket at zero. The seven pinned
  `%TypedArray%.prototype` accessor resizable-buffer sources then passed an
  exact raw `28/28` matrix across both Script modes and both prelude stores:
  `byteLength/resizable-buffer-assorted.js`,
  `byteLength/resized-out-of-bounds-1.js`,
  `byteLength/resized-out-of-bounds-2.js`,
  `byteOffset/resized-out-of-bounds.js`,
  `length/resizable-buffer-assorted.js`,
  `length/resized-out-of-bounds-1.js` and
  `length/resized-out-of-bounds-2.js`. Every execution used Wasm-AOT,
  preserved the exact source, declared ordered `compareArray.js` and
  `resizableArrayBufferUtils.js` includes, and retained the exact
  `resizable-arraybuffer` feature with empty flags and no negative metadata. Only
  T13's static-subclass replacement changed the helper. The unmodified helper
  stopped at the explicit Function-constructor dynamic-code-generation
  boundary. The renamed shared Array and TypedArray invariant pins the seven
  new sources with exact modes, stores, source and prelude bytes, origins,
  suffixes and T13 contract membership. Deleting the complete known-static
  `for-of` wrapper and TypedArray accessor rewrite, the wrapper's sole
  materialization call and the obsolete identity assertions removes all 13
  T17 semantic observations; ordinary materialization now appends the original
  source directly. The three broad TypedArray accessor admissions and T13's
  helper contract remain. The historical pre-delete ledger contained 303
  entries: 35 legitimate harness adaptations, 112 diagnostic instrumentation
  sites and 156 semantic shortcuts. The regenerated ledger contains 290
  entries: 35 legitimate, 112 diagnostic and 143 semantic. T16 owns 24; T17
  owns 147, split between 66 semantic shortcuts and 81 diagnostic guards; T18
  owns 12. After deletion, the rebuilt production dispatcher passed the exact
  seven-source cohort in both Script modes (`14/14`) with every failure and
  non-success bucket at zero. This does not claim broad T17 closure. The 43
  pinned DataView method wrong-receiver sources now keep their original bytes.
  The exact set contains `this-is-not-object.js` and
  `this-has-no-dataview-internal.js` for the 21 mapped methods present at the
  current pin, plus the sole
  `getInt32/this-has-no-dataview-internal-sab.js`. Mapped `setBigUint64` has
  none of those files, and no other mapped method has the SAB suffix. A
  pre-delete direct raw probe covered `getInt8` primitive receivers, the
  `setFloat16` wrong-slot case, the `getBigInt64` and `setBigInt64` metadata
  shapes, and the `getInt32` SAB case across both Script modes and both prelude
  stores. All `20/20` executions reported `backend_used: WasmAot`. This bounded
  proof did not run every physical source. The replacement invariant scans all
  22 mapped methods against all three suffixes and pins the exact 43-source
  census, contract fingerprints, metadata, mode order, admission, original
  bytes, LocalMerged assert-only materialization and vendored `assert.js` then
  `sta.js` materialization. Deleting the sole dispatcher call, complete
  rewrite and obsolete synthetic test removes exactly six T17 semantic
  observations. The verified pre-retirement ledger contained 290 entries,
  including 143 semantic shortcuts. The regenerated ledger contains 284
  entries: 35 legitimate, 112 diagnostic and 137 semantic. T16 owns 24; T17
  owns 141, split between 60 semantic shortcuts and 81 diagnostic guards; T18
  owns 12. The shared method mapper, range and resizable rewrites,
  method-metadata and accessor invariants, and broad DataView SAB admission
  remain. After deletion, the rebuilt production dispatcher passed all 43
  exact sources in both Script modes (`86/86`) with every failure and
  non-success bucket at zero. This does not claim broad T17 closure. The 41
  pinned DataView method range sources now keep their original bytes. The exact
  cohort has `index-is-out-of-range.js` for all 11 getters and 10 setters, plus
  `range-check-after-value-conversion.js` and
  `index-check-before-value-conversion.js` for those same 10 setters. The
  current pin has none of the three files for `setBigUint64` and no getter
  conversion-order files. A pre-delete raw run passed every physical source
  with LocalMerged sloppy materialization (`41/41`). The `setUint16`
  range-after, `setBigInt64` index-before, `getBigUint64` out-of-range and
  `setFloat16` out-of-range representatives also passed both Script modes and
  both prelude stores (`16/16`). The first manually assembled conversion-order
  stream omitted LocalMerged `sta-preamble.js` and failed because
  `Test262Error` was unbound. Restoring the normal prelude made that source
  pass; no corrected compiler or runtime cell failed. The replacement
  invariant pins the closed 41-source census, absent files, fingerprints,
  metadata, modes, admission, original bytes and no-rewrite boundary.
  LocalMerged materialization uses `assert.js` then `sta-preamble.js` for the
  20 conversion-order sources and only `assert.js` for the 21 out-of-range
  sources; vendored-only materialization always uses complete `assert.js` then
  `sta.js`. Deleting the sole dispatcher call, complete range rewrite,
  `dataview_method_range_info`, `dataview_method_call` and obsolete synthetic
  test removes exactly six T17
  semantic observations. The verified post-wrong-receiver baseline, after its
  `86/86` production run, contained 284 entries and 137 semantic shortcuts.
  The regenerated ledger contains 278 entries: 35 legitimate, 112 diagnostic
  and 131 semantic. T16 owns 24; T17 owns 135, split between 54 semantic
  shortcuts and 81 diagnostic guards; T18 owns 12. At that checkpoint the
  shared method mapper, complete resizable rewrite and helpers, admissions and
  neighboring invariants remained. After the following resizable deletion, a
  rebuilt production run passed this exact range cohort (`82/82`) with every
  failure and non-success bucket at zero. The 22
  pinned DataView method `resizable-buffer.js` sources now also keep their
  original bytes. The exact cohort has one source for each of `getInt8`,
  `getUint8`, `getInt16`, `getUint16`, `getInt32`, `getUint32`, `getFloat16`,
  `getFloat32`, `getFloat64`, `getBigInt64`, `getBigUint64`, `setInt8`,
  `setUint8`, `setInt16`, `setUint16`, `setInt32`, `setUint32`, `setFloat16`,
  `setFloat32`, `setFloat64`, `setBigInt64` and `setBigUint64`. A pre-delete
  raw run passed all sources through Wasm-AOT with LocalMerged and vendored-only
  preludes in both Script modes (`88/88`). The replacement invariant pins all
  22 source fingerprints and bytes, exact metadata, both modes, admission,
  no-rewrite status and exact prelude order, provenance and bytes. LocalMerged
  materialization uses `assert.js` then `sta-preamble.js`; vendored-only
  materialization uses `assert.js` then `sta.js`. Deleting the sole dispatcher
  call, complete resizable rewrite, its value-literal helpers, the now-dead
  shared method mapper, all three mapper-only test assertions and the obsolete
  synthetic test removes exactly five T17 semantic observations. The verified
  post-range checkpoint contained 278 entries, including 131 semantic
  shortcuts, and assigned 135 observations to T17. The regenerated ledger
  contains 273 entries: 35 legitimate, 112 diagnostic and 126 semantic. T16
  owns 24; T17 owns 130, split between 49 semantic shortcuts and 81 diagnostic
  guards; T18 owns 12. Broad DataView resizable, SAB and immutable admissions,
  constructor and accessor authorities, and neighboring source invariants
  remain. The same rebuilt production run passed the exact resizable cohort
  (`44/44`), for `126/126` combined DataView method executions. This does not
  claim broad T17 closure. That verified method run and its 273-entry ledger,
  including 126 semantic shortcuts, form the historical constructor
  pre-retirement baseline. The 43 pinned DataView constructor validation
  sources now keep their original bytes. The exact cohort has ordinary and SAB
  sources for 19 filenames, plus the ordinary `buffer-not-object-throws.js`
  source and four ordinary resize-during-custom-prototype sources. Those five
  SAB counterparts are absent at the current pin. A bounded pre-delete raw
  probe ran eight representative sources through LocalMerged and vendored-only
  preludes in both Script modes, then ran one LocalMerged sloppy execution for
  each of the other 16 filename arms. All `48/48` executions reported
  `backend_used: WasmAot`; no compiler, runtime or harness cell
  failed. The replacement invariant pins the 43-present/5-absent census,
  sorted source-contract fingerprints, exact metadata, both mode executions,
  admission, no self-contained rewrite and original bytes. Its LocalMerged
  groups are now 32 full-assertion sources, nine full assertion plus
  `sta-preamble.js` and two full assertion plus property-helper sources.
  Vendored-only materialization uses exact `assert.js` then `sta.js`
  bytes, plus `propertyHelper.js` for the two extensibility sources. Deleting
  the sole dispatcher call, complete constructor rewrite, its sole filename
  selector and the obsolete synthetic test removes exactly seven T17 semantic
  observations. That T17 retirement checkpoint contained 248 entries: 35
  legitimate, 112 diagnostic and 101 semantic. T16 owns 24; T17 owns 105, split
  between 24 semantic shortcuts and 81 diagnostic guards; T18 owns 12. Broad
  DataView SAB and resizable admissions, the existing eight-source
  constructor-surface invariant, method and accessor replacement invariants,
  metadata authorities and unselected constructor neighbors remain. After
  deletion, the rebuilt production dispatcher passed all 43 exact sources in
  both Script modes
  (`86/86`) with every failure and non-success bucket at zero. This does not
  claim broad T17 closure. The pinned `toReversed/this-value-invalid.js` and
  `toSorted/this-value-invalid.js` sources now also execute without handwritten
  replacements. A pre-delete raw probe passed both sources in sloppy and strict
  LocalMerged modes (`4/4`), and six representative change-by-copy programs
  passed with the complete upstream `testTypedArray.js`. The replacement
  invariants pin both receiver contracts and the exact 21-source
  `toReversed`/`toSorted` helper cohort across 42 Script executions, both
  prelude stores, unchanged source suffixes and the intact 14,921-byte upstream
  helper; neither compact nor split dispatcher materialization is admitted.
  Vendored-only coverage at that checkpoint was a materialization/provenance
  assertion, not an execution claim. The typed host boundary described below
  now supplies the missing materialization contract. Deleting both receiver
  rewrite authorities and the two family-specific
  dispatcher-split gates removes twelve T17 semantic observations from the
  266-entry constructor checkpoint. The rebuilt production CLI passes the
  complete `toReversed` and `toSorted` directories (`18/18` and `24/24`,
  `42/42` combined) with every failure and non-success bucket at zero. Shared
  split-helper machinery remains for independently owned TypedArray families;
  this does not claim broad T17 closure. The `with/` directory no longer
  selects that shared split-helper path either. A bounded pre-delete raw probe
  passed four representative unchanged executions (`4/4`). The replacement
  invariant pins all 22 physical sources and 44 sloppy/strict executions,
  exactly 21 full `testTypedArray.js` consumers, the one no-helper neighbor,
  source contracts, metadata, both prelude stores and unchanged source
  suffixes. Deleting the sole `with/` selector removes one T17 semantic
  observation. The rebuilt production CLI passes the complete directory
  (`44/44`) with every failure and non-success bucket at zero. Split-helper
  ownership remains for other independently tracked TypedArray families; this
  does not claim broad T17 closure. The family-prefix selectors for
  `toLocaleString`, `slice`, `filter` and `map` are now all retired. Exact
  invariants cover `39/78`, `91/182`, `84/168` and `84/168`
  physical/execution identities respectively and permit only complete
  `testTypedArray.js` or an explicitly absent helper. Earlier complete-leaf
  replays passed `78/78`, `182/182`, `168/168` and `168/168` before the slice
  retirement. The first three prefix deletions plus a source-text guard removed
  four T17 semantic observations; the later slice wave removes two more
  semantic observations and one diagnostic guard. Its rebuilt CLI passes all
  44 changed executions plus `6/6` adjacent authority controls, rather than
  claiming a new complete `182/182` sweep. The final `includes`, `indexOf` and
  `lastIndexOf` prefix compaction is now gone too. One exact invariant scans all
  130 physical sources and 260 sloppy/strict executions in both prelude stores,
  permitting exactly 117 full helpers and 13 sources without
  `testTypedArray.js`. Fifteen former compact and twelve former intrinsic cases
  now use the complete helper; the 13 no-helper sources remain distinct from
  the 11 T13 static-resizable-helper consumers. Deleting the shared 5,254-byte
  helper, its source parser and the final three-prefix selector removes five
  T17 semantic observations. The rebuilt release CLI passes all 54 changed
  executions plus `4/4` literal-plan and then-surviving iterator/find controls under suite pin
  `aa55200d1310384c5cf69ea95b2a2ecba457007b`, with every non-success bucket at
  zero. No family-prefix compaction remains; closed literal plans own the
  remaining intrinsic and split dispatch. This does
  not claim a complete `260/260` replay or broad T17 closure.

  The shadowed 41-path TypedArray iterator/find matcher layer is now gone.
  Every one of its 17 iterator and 24 find contracts was already owned by the
  closed 319-case literal plan, so the deleted fallback could not affect
  materialized bytes. A replacement invariant pins all 82 sloppy/strict
  executions and 164 materializations across both prelude stores: 18 physical
  sources use the split full-vendored plan, 23 have no `testTypedArray.js`, 21
  retain compare-array provenance and T13's static resizable-helper rewrite,
  and local/vendored STA provenance is exactly `28/82`. Deleting both matcher
  tables, their fingerprint guards, the source-only intrinsic fallback and the
  obsolete split-eligibility wrapper removes four semantic and two diagnostic
  observations. The rebuilt release CLI passes six representative sources in
  both Script modes (`12/12`) under suite pin
  `aa55200d1310384c5cf69ea95b2a2ecba457007b`, with every non-success bucket at
  zero. This is a representative product replay, not a complete `82/82` run or
  broad T17 closure.

  The split dispatcher no longer scans source text for ten tail-only bindings
  or conditionally retains the unused 2,854-byte end of `testTypedArray.js`.
  The closed literal-plan invariant proves all 218 FullVendored physical
  sources, representing 420 executions, have zero references to those
  bindings and always materialize the canonical 12,362-byte split with FNV-1a
  `0x92c7_bac7_27f5_772d`; the split appears exactly once and the tail marker is
  absent. Drifted cases and helpers still fall back to the full vendored
  prelude at the exact contract boundary. Removing the dead source predicate
  and full-tail branch deletes two T17 semantic observations without changing
  any admitted materialization bytes. Four representative `some`, `find`,
  `entries` and `copyWithin` sources pass all `7/7` applicable executions under
  suite pin `aa55200d1310384c5cf69ea95b2a2ecba457007b`, with every non-success
  bucket at zero. This is not a complete `420/420` product replay.

  Test262 prelude
  loading now records private `None`,
  `EmbeddedSpecExecSta`, or opaque complete Wasm-AOT host ownership.
  `EmbeddedWasmAotHostOnly` combines that Wasm-AOT host with complete vendored
  named helpers. Only the child-module validator can construct the embedded-host
  witness. Required named `assert.js` and `sta.js` entries are checked before
  ownership is stored, and replacing either entry revokes that ownership.
  Non-raw materialization resolves every declared include before
  host planning, fails with the execution id and missing include name, and fixes
  host-requiring source order as strict directive, host, `assert.js`, then
  `sta.js`. The source-and-resolved-helper census contains 797 physical sources
  and 1,547 executions; ten exact self-contained rewrite sources account for 20
  executions, leaving 787 physical sources and 1,527 executions that emit the
  host prelude. Agent workers receive the same host/assertion/`sta.js` prelude
  through private materialized state rather than runner-side source inspection;
  the pinned Atomics notification case passes through the product runner with
  that exact host order.
  The four ProxyCreate target-shape sources and the Proxy apply non-callable-trap
  Realm source also pass unchanged (`8/8` and `2/2`). After
  deleting their source rewrites and the Proxy apply null-handler Realm branch,
  retiring the complete `Proxy.revocable` rewrite removes four more semantic
  observations and leaves 6 T11-owned entries. Its 17 ordinary physical cases
  preserve their pinned sources; `tco-fn-realm.js` preserves raw
  `other.evalScript` and remains owned by T13's typed `RealmEvalScript` AOT
  unsupported boundary. The eight staging flatMap sources and five Array
  keys/entries resizable-buffer sources also pass unchanged in sloppy and
  strict modes (`16/16` and `10/10`); deleting their three rewrite owners and
  the entries case's second source transform leaves 36 T15-owned entries. The
  twenty `every`/`some`/`find`/`reduce`/`map`/`filter`/`flatMap`/`take`
  metadata branches are also gone. A pinned materialization matrix covers all
  forty sloppy/strict variants with exact original bytes and full applicable
  LocalMerged and vendored helper provenance. Its focused invariant passes
  `1/1`, and an isolated raw Wasm-AOT run of those exact twenty sources passes
  all `40/40` sloppy/strict executions with every non-success bucket at zero.
  Their eight enclosing selector tables still own other rewrites, so the
  inventory at that checkpoint remained 360 total, 212 semantic and 36
  T15-owned entries;
- the latest shared semantic golden passes `2/2` in 681.86 seconds with 684
  dumps, adding only the PlainDateTime field-read witness and removing none.
  All 683 retained dumps preserve every non-accounting summary; 51 differ only
  in compiler accounting, each with 294 fewer emitted code bytes;
- implement executable GC and real weak reachability, plus complete
  arbitrary-precision BigInt operations;
- finish parser grammar and structured early-error closure while preserving the
  landed parse-once boundary;
- finish modules/linking, broad RegExp grammar, complete Intl, general suspended
  async/generator control flow and remaining cross-realm/exotic-object edges;
- build the planned differential generation, reduction, replay and sustained
  fuzzing pipeline.

See [the implementation task plan](tasks/README.md) for the status and current
repository evidence for every epic. Conformance percentages remain governed by
the generated status block above, not by this task summary.

## Rust Workspace

- `crates/lila-front`: parser boundary and source-unit handling.
- `crates/lila-ir`: spec-shaped IR, diagnostics, and lowering metadata.
- `crates/lila-intl`: Intl data/profile/protocol domains and the first pinned,
  host-embedded locale canonicalization provider.
- `crates/lila-runtime`: realms plus typed host clock, randomness, and output
  capabilities.
- `crates/lila-aot-wasm`: primary direct JS -> Wasm backend.
- `crates/lila-engine`: public Rust library API.
- `crates/lila-cli`: clean-break `lila` command.
- `crates/lila-test262`: Test262 discovery, execution, snapshots, taxonomy, and README status publishing.
- `crates/lila-spec-exec`: reference/spec execution backend used for conformance work.
- `crates/lila-backend-c` and `crates/lila-backend-native`: scaffolds, not product-ready emitters.

Supporting directories:

- `docs/rust-rewrite`: rewrite notes, architecture invariants, and conformance taxonomy.
- `test262`: pinned real Test262 checkout, snapshots, and generated backlog.
- `scripts`: repo maintenance and low-RAM real-suite publication scripts.
- `vendor`: vendored Rust dependencies used by the rewrite.

## CLI

Build the Rust CLI:

```sh
./scripts/dev.sh build
```

The developer wrapper uses `lld` when available, falls back to the system
linker, and caps Cargo at half the machine's logical CPUs (at most eight on the
primary 16-core development machine). It deliberately shares Cargo's normal
`target/` directory. `./scripts/dev.sh check`, `exact-test`, `test262`, and
`timings` provide the corresponding fast-loop commands; set `LILA_JOBS` to
request a lower cap.

Run the built binary directly:

```sh
./target/debug/lila --help
./target/debug/lila inspect crates/lila-cli/tests/fixtures/hello.js
./target/debug/lila run --execution-backend wasm crates/lila-cli/tests/fixtures/hello.js
./target/debug/lila build wasm crates/lila-cli/tests/fixtures/hello.js
```

Or run it through Cargo:

```sh
cargo run -p lila-cli -- inspect crates/lila-cli/tests/fixtures/hello.js
```

Current commands:

- `run [--execution-backend wasm|spec] <file>` runs a script through the Rust engine. Wasm-AOT is the product default and the only result counted for conformance; `spec` is an explicitly selected, feature-gated differential oracle.
- `build wasm <file>` compiles JavaScript directly to a Wasm artifact and prints the artifact summary.
- `cache status` reports the bounded Cranelift function cache, Wasmtime native-module cache, Lila program-Wasm cache, and the old global Wasmtime cache without modifying any of them. `cache prune` removes only Lila-owned entries; add `--legacy-wasmtime` to explicitly remove the reported legacy cache too.
- `build c <file>` and `build native <file>` exist as CLI surfaces but currently fail with scaffold errors.
- `inspect <file>` prints the parser/lowering pipeline summary and invariants.
- `types [entrypoint] [output] [options]` and `typegen` generate Wrangler-style Worker TypeScript declarations from config plus a selected entrypoint.
- `test262 ...` drives the fake fixture suite, pinned real suite, status snapshots, triage, and README status publication.
- `repl` is reserved for the Rust REPL and is not implemented yet.

The Rust CLI also exposes a convenience command for Worker-style TypeScript
setup:

```sh
cargo run -p lila-cli -- types src/index.ts worker-configuration.d.ts --config wrangler.jsonc
```

`lila types` mirrors Wrangler's type-generation shape: it writes
`worker-configuration.d.ts` by default, accepts `--config`, `--entrypoint`,
`--env`, `--env-interface`, `--include-runtime=false`, `--include-env=false`,
`--strict-vars=false`, `--check`, `--print`, and discovers `wrangler.jsonc`,
`wrangler.json`, `wrangler.toml`, or `lila.*` config files from `--cwd` when
`--config` is omitted. An explicit positional entrypoint or `--entrypoint`
overrides the config `main`, matching the common Wrangler flow of generating
types from a config plus a selected worker source. JSON, JSONC, and TOML configs
are supported, and `lila typegen` is accepted as an alias. The type-generation
paths are covered by `cargo test -p lila-cli types_ --quiet`.

Wasm-AOT compilation uses one process-wide Wasmtime engine and a shared
Cranelift pool. The pool defaults to half the logical CPUs; `lila --jobs N ...`
overrides it, while Test262 `--threads N` controls case workers independently.
Every execution still creates a fresh realm, Store, and Wasmtime instance.
Up to 64 immutable compiled Wasmtime Modules are retained in-process with LRU
eviction so a warmed chunk does not deserialize/relink the same native code;
module state is never shared between executions.

Compiled-code storage is Lila-owned and capped at 2 GiB total: 1 GiB for
Cranelift function stencils and a 1 GiB whole-program budget split evenly
between emitted program Wasm and Wasmtime native modules. Each tier prunes to
70% after crossing its limit. Program entries are keyed by a versioned,
presence/length-framed tuple of source, parse goal, compiler options and
architecture, plus a build-time SHA-256 of the compiler inputs;
Cranelift supplies its stencil/version/target/flags key for function entries.
Writes are atomic and corrupt program/native entries are treated as misses.
Test262 agent roots and the complete `agent prelude + worker source` use the
same bounded program-Wasm cache; only immutable Wasm bytes are reused, while
every Store, instance, realm, shared-memory backing, report queue, and worker
remains fresh. Host globals are selected by a typed compilation policy:
ordinary product/CLI compilation exposes ECMAScript globals and the deliberate
`print`/`gc` extensions, while the Test262 runner explicitly enables its
`__lila*` capabilities and agent workers inherit that same policy. The policy
also participates in the program-Wasm cache key. Conformance fixtures invoked
through `lila run` can opt in explicitly with `--host-surface test262`; ordinary
CLI invocations remain on the product surface. A focused
shared-buffer/report regression measured `22.05 s`
cold and `0.32 s` warm after both the root and worker became cache hits.
Concurrent pruning may remove an entry during a cache scan; that vanished entry
is skipped without turning an otherwise valid cache write into a failure.
Set `LILA_CACHE_DIR` to relocate only Lila's cache. The legacy global
Wasmtime directory is reported by `lila cache status` and is never deleted
implicitly.

`LILA_WASM_TRACE=1` reports parse, lower, emit, program/function/module
cache decisions, native compilation, instantiation, and execution timings.
`LILA_WASM_TRACE_DUMP=1` additionally emits the large backend debug dump.
CI can sample and recompile function-cache hits with
`LILA_VERIFY_FUNCTION_CACHE=1`.

## Conformance

The conformance goal is literal full pinned Test262 green for the Rust path, with
fake-suite progress kept separate from real-suite progress.

Useful local checks:

```sh
cargo test -p lila-engine --quiet
./scripts/run-watched.sh --label cli --stall 900 -- cargo test -p lila-cli --test cli -- --test-threads=2
./target/debug/lila test262 run language/wasm/pass --suite-root crates/lila-test262/tests/fixtures/fake_test262/vendor/test262 --execution-backend wasm
./target/debug/lila test262 run --suite-root crates/lila-test262/tests/fixtures/fake_test262/vendor/test262
```

The CLI suite is 617 default-executing tests (618 compile; 8 more sit behind the
`spec-exec-oracle` feature): about 26 minutes at `--test-threads=8` on 16 CPUs,
an estimated 1 h 45 min at `--test-threads=2`. Raise the thread count on a
machine with spare cores, but keep `--stall 900` — a single cold Wasm-AOT
compile can exceed the 300 s default of log silence, and the guard then kills a
healthy run with exit code 124.

**Do not use `--test-threads=1`.** libtest then runs every test on the thread
named `main`, the per-test name the suite routes on is unavailable, and all 617
tests fall back to spawning a cold `lila` child process instead of the warm
in-process call the 26-minute figure is built on. It is correct and terminating,
just far slower. For a single test use `-- --exact <name>`.

It no longer needs `--skip atomics_wait_core`: that case is green and the
known-failure row was removed in batch 6. The guarded child-process path remains
available for future explicitly declared hangs, but the current CLI ledger has
no hang entry. A hang in a test with
*no* ledger row is bounded too, on the in-process path, so the suite terminates
either way.

Do not compare the result against a list by hand. That ledger is enforced by
the suite itself — a new failure, a declared failure that starts passing, a
declared failure that fails for a *different* reason, a renamed or deleted
declared test, an orphan ledger row, or an `#[ignore]` with no owner all turn
rung 1c red. Green means exactly the declared outcomes, for the declared
reasons.

Developer differential builds expose a versioned bounded replay protocol.
`lila differential replay <case.json> --oracle spec-exec` keeps schema v1's
self-checking, no-output disposition comparison unchanged and additively admits
schema v2 `primitive_completion_no_output` cases. V2 compares normal-versus-throw
plus `undefined`, `null`, Boolean, canonical Number bits, UTF-16 String units or
decimal BigInt; output, Symbol and Object observations make the bounded contract
red. Schema v3 `primitive_completion_print_transcript` compares the same
primitive completion plus the exact ordered root `PrintLine` transcript. It has
a distinct green verdict; unavailable output, Symbol, Object and backend
failures remain red, while mismatches receive a length-delimited stable
signature. Every match still reports semantic equivalence as `not_established`.
All three schemas are dependency-sealed: Module goals and actual or
conservatively possible outer Script dynamic imports are rejected because the
wire carries no graph. Replay fixes AOT root and agent-worker graph discovery,
plus spec-exec root, created-realm and agent contexts, to a mandatory reject-all
loader. Imports executed through spec-exec dynamic source reject without
ambient IO; AOT dynamic-source compilation remains unsupported. Existing valid
Script bytes and fingerprints are unchanged. See the
[source-closure contract](docs/rust-rewrite/contracts/differential-source-closure.md)
and
[schema-v3 observation contract](docs/rust-rewrite/contracts/differential-primitive-print-transcript.md).

The same builds also expose a deterministic bounded campaign:
`lila differential generate-arithmetic <output.json> --seed N --checks N
--depth 1|2|3|4 --max-replays N --oracle spec-exec` generates and, on a
backend mismatch, type-safely reduces the `integer-arithmetic-v1` Add/Sub
corpus slice. Like `differential replay`, it requires a
`--features spec-exec-oracle` build and explicit oracle selection.

For local Wasm-AOT Test262 iteration, use the default in-process case runner:
execution remains exact-source, realm-isolated, and epoch-timeout bounded while
reusing the process-wide engine and caches. Set
`LILA_TEST262_FORCE_CASE_RUNNER=1` only for crash reproduction or deliberate
per-case process isolation. Ensure `LILA_CACHE_DIR` is writable; on a
representative 18.48 MiB TypedArray iterator module, a cold exact run took about
50 seconds while an identical warm run completed in 2 seconds from the program
and module caches.

For real-suite publication, prefer the low-RAM wrapper so the top-level matrix
checkpoints one node at a time, isolates each case in a reclaimable process,
uses one compiler job by default, and only publishes after verified completion.
Set `ISOLATE_CASES=0` or raise `JOBS` and `THREADS` only when more memory is
available. The wrapper asks Lila's Rust matrix planner for progress between
nodes and only accepts the product backend:

```sh
./scripts/publish-real-status-low-ram.sh wasm-aot codex-published-real
```

Oracle matrices remain available through `lila test262 report-all
--execution-backend spec-exec`; they cannot enter the status publisher.

Useful status and triage commands:

```sh
./target/debug/lila test262 progress-status --execution-backend wasm-aot
./target/debug/lila test262 triage-status --execution-backend wasm-aot
./target/debug/lila test262 failure-details language/wasm --execution-backend wasm-aot
```

## Contribution Protocol

Task work is tracked under `tasks/`. Before opening a change that affects the
Rust rewrite or conformance story, run:

```sh
./scripts/check-task-plan.sh
./scripts/check-module-boundaries.sh
```

Use the pull request template fields to keep fake-suite smoke evidence separate
from pinned real Test262 evidence. `Unsupported`, timeout, crash, and bug are all
non-passing outcomes. The generated README status block must only move with the
publisher output and its snapshot artifacts; documentation-only edits belong
outside the `lila-status` markers.

Until T02 lands and splits the monolithic IR and Wasm backend modules, treat
`crates/lila-ir/src/lib.rs` and `crates/lila-aot-wasm/src/lib.rs` as
single-owner files. Feature work that needs shared ABI changes should land the
interface first under T04 rather than mixing unrelated feature lanes.

## Current Capabilities

Rust Wasm-AOT currently compiles a limited but useful JavaScript subset. Treat
this as a tested capability map, not a spec-completeness claim. Programs are
most likely to work when they stay close to the fixtures under
`crates/lila-cli/tests/fixtures/wasm_*.js` and the fake wasm-safe Test262
cases under
`crates/lila-test262/tests/fixtures/fake_test262/vendor/test262/test/language/wasm/pass`.

Focused Test262 counts below that predate the execution-identity cutover are
historical physical/path evidence, not current execution numerators or
denominators. Unflagged files now contribute separate sloppy and strict
executions, so only entries that explicitly report execution variants from a
post-cutover rerun are current focused evidence.

Recent focused progress through `2026-09-01`:

- Call argument lowering now returns a must-consume ordering authority. Any
  intervening write, call or spread clears heap-shape evidence captured by an
  earlier argument and explicitly named callee/receiver snapshots before
  result analysis; direct, private, optional, super and constructor calls all
  consume that authority. Source-function `this` observations now occur after
  arguments, and optional-chain property/getter analysis is interleaved with
  source evaluation instead of replayed after every argument. The standard
  builtin catalog also closes all 29 Promise entries into 24 synchronous-user-
  code and five synchronously pure cases, with narrow ordinary-call,
  missing/primitive-executor and primitive-resolution bypasses, and accounts
  for `Function.prototype.apply` array-like getters. Focused IR tests and two
  compiled Wasm-AOT fixtures cover descriptor and prototype arguments,
  ordinary/default/private/optional receivers, constructor prototype changes,
  getter-before-argument order and Proxy-backed apply lists; all pass. The
  broad suite was not rerun for this batch.

- Two `lila-ir` raw-line caps are restored through real ownership seams. The
  closed callable source-text representation and its exhaustive materializer
  now live in a 38-line child behind the existing public re-export, while the
  nonduplicable invocation-effect proof lifecycle and closed analyzed-effects
  state plus an opaque source/host caller-flow aggregate live in a 192-line
  private lowering child with no compatibility path. `AlreadyApplied` and
  `MustAttach` make the post-analysis state exhaustive instead of encoding it
  as an optional proof. Source-call preservation is admitted only by a
  nonduplicable token from an exhaustive finalized-invocation proof that
  includes parameter defaults. Base constructors fold instance initializer
  effects; synthetic derived constructors account for their implicit dynamic
  `super`. The host catalog admits only realm creation as preserving, while
  non-callback mutations stay invalidating. The standard-builtin catalog marks
  the complete Object/Reflect proxy-capable surface as synchronous user code,
  including spread and mixed-candidate paths; exact proven-safe returns retain
  their existing precision. Collection construction is argument-sensitive,
  and Iterator/Set protocol methods retain their hook effects. Optional-chain
  data reads preserve facts only when their shape proves that no getter or
  proxy hook can run. The guarded parents are 1,748 of
  1,760 and 2,243 of 2,250 lines respectively. Tree-wide module policy checks
  pin the sole owners, proof constructors, complete 34/83/29 IR census,
  indexed-mutator catalog, `Drop` boundary, narrow re-export, colocated
  behavior tests and bounded child sizes. See
  [`lila-ir-module-budget-owner-splits.md`](docs/rust-rewrite/contracts/lila-ir-module-budget-owner-splits.md).

- The closed plain-async synchronous `for-of` plan now admits `let` and
  `const` array and object binding patterns with one direct body `await`.
  `AsyncFunctionForOfIteratorHeadIr` derives one of exactly three storage
  lifetimes: activation, fresh iteration Environment Record, or an unspellable
  entry local. Lexical patterns use the entry local plus an exact complete set
  of iteration slots and TDZ placeholders. Capture analysis materializes every
  BoundName before assigning capture hops, and lowering predeclares all names
  before any default or computed key, so direct reads and retained closures
  observe the correct fresh cells across suspension. The source-free
  `wasm_plain_async_sync_for_of_lexical_pattern_heads.js` oracle covers nested
  patterns, defaults, computed object keys, array and object rest, mutable
  `let`, forward and captured-head TDZ, post-await `const` assignment, semantic
  empty patterns, and nested plus outer IteratorClose precedence. `cargo fmt
  --all -- --check` and the relevant
  all-target compile pass; the IR `for_of` filter and exact rejection witness
  pass `27/27` and `1/1`, six focused and affected structure targets pass
  `28/28`, and the new plus four retained CLI oracles pass `5/5`. The fixture
  passes `node --check` and its Node semantic baseline. The pinned Test262
  checkout has no exact lexical-pattern/direct-await leaf, so no Test262 count
  is claimed. See
  [`plain-async-synchronous-for-of-lexical-pattern-heads.md`](docs/rust-rewrite/contracts/plain-async-synchronous-for-of-lexical-pattern-heads.md).

- The activation-backed plain-async synchronous `for-of` plan now admits
  assignment patterns and `var` binding patterns. Array and object
  destructuring, nesting, non-suspending defaults, rest, and typed identifier,
  public-member, and private-member assignment targets execute once inside
  IteratorClose and before the direct body `await`; resume does not replay the
  pattern. `var` BoundNames remain in the async activation across suspension.
  Assignment-pattern capture analysis now exhaustively visits both pattern
  shapes and both nesting directions. The source-free
  `wasm_plain_async_sync_for_of_nonlexical_pattern_heads.js` oracle covers
  array and object `var` patterns, computed assignment order, getters,
  defaults, rest targets, once-only effects, and nested plus outer close Throw
  precedence. `cargo fmt --all -- --check` and the relevant all-target compile
  pass; the IR `for_of` filter passes `24/24`, its explicit rejection matrix
  passes `1/1`, six focused and affected structure targets pass `25/25`, and
  the new plus three retained CLI oracles pass `4/4`. The fixture passes
  `node --check` and its Node semantic baseline. The later lexical-pattern
  checkpoint above supersedes this checkpoint's historical `let`/`const`
  rejection with a full fresh per-iteration Environment Record and TDZ model.
  No matching pinned Test262 cohort is claimed. See
  [`plain-async-synchronous-for-of-nonlexical-pattern-heads.md`](docs/rust-rewrite/contracts/plain-async-synchronous-for-of-nonlexical-pattern-heads.md).

- The activation-backed plain-async synchronous `for-of` plan now admits
  static, computed, and private member-reference heads in addition to its
  single-name forms. Each yielded value enters the existing `$forof.access`
  slot, then the member Reference is re-evaluated and written once inside the
  IteratorClose frame before the body `await`; resumption does not repeat the
  write. Capture analysis now owns member bases and computed keys used only by
  the loop head. The source-free
  `wasm_plain_async_sync_for_of_member_heads.js` oracle covers target/key
  re-evaluation, writes to changing targets, public setter and private-brand
  failures, IteratorClose counts, and Throw precedence. Resource heads,
  `super`, suspending member operands, nonlinear body suspension, and `for
  await` remain explicit nonclaims; the later nonlexical- and lexical-pattern
  checkpoints above supersede the historical all-pattern nonclaim. `cargo fmt
  --all -- --check` and the
  relevant all-target compile pass; the IR `for_of` filter passes `21/21`, its
  explicit rejection matrix passes `1/1`, six focused and affected structure
  targets pass `25/25`, and the new plus retained capture CLI oracles pass
  `2/2`. The fixture passes `node --check`. No matching pinned Test262 cohort,
  semantic golden, or published-status refresh is claimed. See
  [`plain-async-synchronous-for-of-member-heads.md`](docs/rust-rewrite/contracts/plain-async-synchronous-for-of-member-heads.md).

- All 15 synchronous `for-of` acquisition and stepping checks now route
  through the closed `SyncIteratorProtocolError` diagnostic and body-Realm
  projections with
  `SyncIteratorConsumer::ForOf`. The boundary covers the five
  checks owned by each of ordinary direct `for-of`, direct `for-of` with an
  async-disposable head, and the resumable plain-async synchronous iterator
  path. Primitive lookup in the two inline direct owners also boxes through
  the current function Realm. Main and user bodies create their
  algorithm-generated errors in the main Realm; only a trusted self-backed
  standard builtin may use its current environment as Realm metadata. The
  entry-Realm CLI fixture covers all five
  error branches, their four diagnostics, and a valid control, but it cannot
  distinguish current-function from main-Realm behavior because Wasm AOT does
  not dynamically compile a user function in a created Realm. The focused and
  affected structure targets pass `37/37`, the exact error fixture and four
  success-path CLI controls pass `5/5`, and four pinned direct `for-of` leaves
  pass all `8/8` Wasm-AOT executions with every failure and non-success bucket
  at zero. See
  [`direct-synchronous-for-of-protocol-error-realm.md`](docs/rust-rewrite/contracts/direct-synchronous-for-of-protocol-error-realm.md).

- Ordinary direct synchronous `for-of` now applies general `IsCallable` and
  Proxy-aware `Call` to both the source's `@@iterator` method and the cached
  iterator `next` method. Callable Proxies receive the original iterable or
  iterator as `this` with no arguments; apply-trap and revoked-Proxy
  completions propagate without being replaced by a protocol diagnostic, and
  abrupt stepping does not call `return`. A bounded Rust guard rejects
  Function-tag gates and Function-only calls in the owner. The source-free
  entry-Realm fixture covers callable, non-callable, throwing, and revoked
  Proxy methods in both positions. It makes no cross-Realm Proxy-internal
  TypeError claim. Thirteen retained captures also pin primitive and
  non-callable Proxy diagnostics to the entry `%TypeError.prototype%`, so a
  lexical-environment slot cannot masquerade as function Realm metadata. The
  affected compile/format check, 23 structure tests, five CLI controls, and 16
  unchanged Test262 executions pass; repository guards remain green and the
  shortcut inventory stays at 240. See
  [`direct-synchronous-for-of-protocol-error-realm.md`](docs/rust-rewrite/contracts/direct-synchronous-for-of-protocol-error-realm.md#callable-proxy-method-follow-up).

- The synchronous iterator path now uses the non-`Copy`
  `SyncIteratorConsumer::{ArrayDestructuring, ArrayAccumulation, ForOf,
  MathSumPrecise}` domain. Its product with four protocol errors gives 16
  exhaustive diagnostic rows, including distinct destructuring and `array
  spread` messages. The confirmed source census is 17 typed projector calls
  and 35 error identifiers. Consumer selection controls wording only.
  Primitive acquisition boxes through the current function Realm, while
  algorithm-created protocol TypeErrors use an exhaustive builder Realm-source
  match: trusted standard builtins use their self-backed current Realm and
  main, user, host, and runtime-helper bodies use the main Realm.
  Destructuring's custom step retains typed
  callability/result checks and `next`, result, `done`, then conditional
  `value` order. ArrayAccumulation step failures propagate without
  IteratorClose. The entry-Realm fixtures cannot distinguish current-function
  from main-Realm error identity, and this checkpoint does not claim the
  current function Realm's `%Array.prototype%` for a fresh Array literal or
  Array-rest result. The all-target compile and formatting check pass; nine
  structure targets pass `42/42`; seven exact Wasm-AOT CLI witnesses pass
  `7/7`; and nine pinned Array-spread/destructuring leaves pass all `18/18`
  sloppy/strict executions with every failure bucket at zero. See
  [`sync-iterator-consumer-capability.md`](docs/rust-rewrite/contracts/sync-iterator-consumer-capability.md).

- Shared synchronous `IteratorClose` now constructs its two algorithm-created
  TypeErrors in the current function Realm. Its 67 external entry routes split
  into 16 direct, 48 preserving-current-Throw, and 3 preserving-saved-Throw
  routes. The preserving routes still restore the incoming Throw, and entry
  code with no current environment still uses the main Realm fallback. The
  source-structure target passes `4/4`, the exact created-Realm CLI test passes
  `1/1`, and the affected `iterator_close` CLI sweep passes `6/6`. The two
  pinned direct `for-of` leaves pass `4/4` Wasm-AOT executions with every
  failure and non-success bucket at zero. This close-only checkpoint left
  ordinary direct `for-of` acquisition and stepping errors as a separate
  nonclaim; the later full boundary above supersedes that historical nonclaim.
  See [`iterator-close-error-realm.md`](docs/rust-rewrite/contracts/iterator-close-error-realm.md).

- Direct synchronous String `for-of` now uses the generic iterator protocol.
  `StatementIr::ForOfString`, `compile_for_of_string`,
  `STRING_CODE_POINT_WALK`, and its two String-specific premises are deleted.
  The loop value is `Dynamic`, and primitive lookup boxes with the current
  function Realm while preserving the primitive as the strict accessor and
  iterator-method receiver. The focused witness replaces both
  `String.prototype[Symbol.iterator]` and `%StringIteratorPrototype%.next`; it
  also requires a Number result from the custom iterator and one
  break-driven `IteratorClose` call. At the direct-path checkpoint, the String
  structure target passed `3/3`, the affected companion structures passed
  `19/19`, the IR `for_of` target passed `17/17`, and the CLI witness passed
  `1/1`. The BMP, astral, and truncated astral leaves passed `6/6` Wasm-AOT
  executions with every failure bucket at zero. The direct-path checkpoint does
  not claim complete iterator-error Realm ownership; the later full boundary
  above supersedes that historical nonclaim. String loops with a directly
  awaiting body now use the resumable
  synchronous iterator plan described in the next entry. The boundary is
  recorded in
  [`synchronous-string-for-of-iterator-protocol.md`](docs/rust-rewrite/contracts/synchronous-string-for-of-iterator-protocol.md).

- Direct synchronous Array `for-of` no longer has an index-walk IR or backend
  emitter. `StatementIr::ForOfArray`, `compile_for_of_array`, and the
  synchronous `ARRAY_INDEX_WALK` witness are deleted. Exact Arrays now use
  `StatementIr::ForOfIterator` with `SYNC_ITERATOR_PROTOCOL`, and their yielded
  value is `Dynamic` because `@@iterator` is replaceable. The focused runtime
  witness covers length growth, an inherited indexed getter, a prototype
  iterator that yields a String, and break-driven `IteratorClose`.
  At the direct-path checkpoint, the focused structure targets passed `3/3`
  and `4/4`, the IR `for_of` tests passed `16/16`, the planner regression passed
  `1/1`, and the new CLI witness plus its ordinary Array control each passed
  `1/1`. The four pinned Array length-mutation leaves passed `8/8` Wasm-AOT
  executions with every failure bucket at zero. The plain-async body-`await`
  form now has its own
  `StatementIr::AsyncFunctionForOfIterator` and a closed
  `AsyncFunctionForOfIteratorPlanIr`. It acquires one synchronous Iterator
  Record before the first iteration and persists that record across each body
  `await`; the yielded value remains `Dynamic`. This deletes
  `AsyncForOfArrayWalkForm`, `lower_async_for_of_array_with_body_await`, and
  `ARRAY_INDEX_WALK_RESUMABLE`. Focused fixtures cover once-only
  `@@iterator`/`next` acquisition, natural exhaustion, String support,
  close-completion precedence, protocol errors that must not close, and fresh
  captured bindings. Simple single-name declarations and bare identifier
  assignment heads are admitted; the protocol fixture observes the latter
  being updated before and after each await. `cargo check -p lila-aot-wasm`
  passes. The five focused structure targets pass `19/19`, the `lila-ir`
  `for_of` target passes `18/18`, and the four exact CLI oracles pass `4/4`.
  All four fixtures pass `node --check`, and the two pinned
  `Array.fromAsync` leaves pass `4/4` Wasm-AOT executions with every failure
  and non-success bucket at zero. The complete 95-file `Array.fromAsync` leaf,
  semantic golden, and published-status refresh were not run. Direct
  `break`/`continue`, suspending head operands, iterable suspension, and owners
  other than plain async functions remain outside this resumable form. The
  later member-reference, nonlexical-pattern, and lexical-pattern checkpoints
  above supersede the historical property- and pattern-head nonclaims. The
  iterator boundary is recorded in
  [`synchronous-array-for-of-iterator-protocol.md`](docs/rust-rewrite/contracts/synchronous-array-for-of-iterator-protocol.md).

- A bare identifier in a `for await` head now writes its resolved outer
  Reference through a synthetic iterator-result slot instead of being declared
  as a loop-owned `var`. The private closed head domain has no clone, debug or
  equality capability; capture analysis records write-only heads, and the
  lowering reuses the ordinary checked identifier write for mutable,
  immutable, `with` and unresolvable cases. Created realms also publish fresh
  WeakRef constructors and prototypes through a private non-copyable must-use
  token that couples the Realm slot, exact descriptors, self-backed callables
  and defining-Realm TypeErrors before global exposure. Constructor-first
  linking preserves the exact `constructor`, `deref`, `Symbol.toStringTag`
  prototype order. Their bounded structure targets pass `9/9`, focused IR
  tests pass `3/3`, and both CLI witnesses pass `2/2`. The exact for-await leaf
  passes `2/2`; six selected non-GC WeakRef leaves pass `12/12`, with every
  failure bucket at zero. The earlier shared golden checkpoint passed `2/2` in
  685.75 seconds with 682 dumps, added only these two witnesses, removed none
  and left all 680 retained dumps byte-identical. Weak reachability and
  dynamic-Function cross-Realm coverage remain open.

- Created realms now publish fresh FinalizationRegistry constructors,
  prototypes, `register`, and `unregister` functions through a private
  non-copyable token. Realm-slot storage, exact descriptors, self-backed
  callables and defining-Realm TypeErrors are complete before global exposure.
  Constructor-first linking preserves the exact `constructor`, `register`,
  `unregister`, `Symbol.toStringTag` prototype order; reverse materialization
  preserves the forward WeakRef/FinalizationRegistry global property order
  while satisfying stack-shaped temporary-local ownership. The bounded
  structure target passes `7/7`, the source-free CLI witness passes
  `1/1`, and six pinned identity, descriptor, receiver and cross-Realm fallback
  files pass all `12/12` sloppy/strict Wasm-AOT executions. Weak reachability
  and cleanup jobs remain open; created-Realm WeakMap/WeakSet publication is
  closed by the next boundary. The FinalizationRegistry boundary is recorded in
  [`finalization-registry-created-realm-publication.md`](docs/rust-rewrite/contracts/finalization-registry-created-realm-publication.md).

- Created realms now publish fresh WeakMap and WeakSet constructors,
  prototypes and all nine methods through one private non-copyable token. The
  materializer writes both closed Realm slots, links constructors before
  methods, gives every callable the created Function and TypeError identities,
  and reuses the sole typed collection `@@toStringTag` authority before global
  exposure. Reverse materialization preserves temporary-local ownership while
  the observable present-global subsequence follows `Map`, `WeakMap`,
  `WeakSet`, `WeakRef`, `FinalizationRegistry`, `Set`. The bounded structure
  target passes `6/6`, the source-free CLI witness passes `1/1`, and sixteen
  pinned identity, descriptor, method and cross-Realm files pass all `32/32`
  sloppy/strict Wasm-AOT executions with every failure bucket at zero.
  `cargo xc` is green and the broad backend target retains the same seven
  unrelated baseline failures at `367/374`. Weak reachability, cleanup jobs,
  full created-global ordering and complete weak-collection trees remain open.
  The boundary is recorded in
  [`weak-collection-created-realm-publication.md`](docs/rust-rewrite/contracts/weak-collection-created-realm-publication.md).

- Descriptor step-four compatibility now projects `Absent`, statically
  `Present`, and run-time presence through one private exhaustive
  `Never`/`Always`/`AtRuntime` emission domain. Ordinary and stored Array
  validators share it for all six fields and both descriptor-kind transitions;
  fresh runtime errors append `name` and `message` only at their proven-new
  allocation boundary, avoiding recursive validation while preserving exact
  flags. The structure target passes `6/6`, the focused CLI witness passes
  `1/1`, and seven selected `Object.defineProperty` leaves pass `14/14` with
  every failure bucket at zero. The 683-dump shared golden passes `2/2` in
  676.81 seconds, adds only this witness, removes none, and preserves all 682
  retained non-accounting summaries.

- Dynamic `Number.prototype.toFixed`, `toExponential` and `toPrecision` now
  share an exact finite-binary64 decimal core selected by private exhaustive
  formatting domains. Supplied precision expands `M * 2^e` into a proven
  768-byte decimal scratch bound before rounding, while omitted exponential
  precision remains a distinct shortest-scientific mode and omitted
  `toPrecision` remains ordinary Number-to-string. The old empty-string
  sentinels, precision answer table and magic integer branch are gone. Three
  structure executables pass `12/12`; the new long-digit/carry/subnormal
  dynamic matrix and both older Number regressions pass `3/3`; and the exact
  fixed, exponential and precision Test262 leaves pass all `6/6`
  sloppy/strict Wasm-AOT executions. The shared golden passes `2/2` in 672.44
  seconds with 680 dumps, adds only the new Number witness, removes none, and
  leaves all 679 retained dumps structurally equal after accounting
  normalization. Function and local counts remain unchanged; the retained
  code-size increase is the expected shared standard-builtin formatter body.
  This is focused decimal-formatting closure, not the complete Number or
  ECMA-402 locale surface.

- Object descriptor, `CreateDataPropertyOrThrow` and Set rejection paths now
  select synthesized TypeError prototypes through one exhaustive
  three-source/two-authority Realm domain; borrowed `Promise.prototype.catch`
  and `finally` share a private non-`Copy` validated delegated-`then` token;
  Array.fromAsync iterator-result reads use only the closed `Done`/`Value`
  property domain; and the eleven Number builtins route through exact
  equality-free policy domains. The Error.prototype.toString emitter moved
  intact into its own private module to keep the parent beneath the enforced
  source-size boundary. `cargo xc`, formatting, diff, task-plan,
  module-boundary and exact 240-entry shortcut gates pass; nine bounded
  structure executables pass `43/43`; seven focused CLI regressions pass
  `7/7`; and the Promise plus Array.fromAsync pinned controls pass all `20/20`
  sloppy/strict Wasm-AOT executions. The shared golden passes `2/2` in 800.46
  seconds with 679 dumps, adds only the Array.fromAsync result-definition
  error-Realm witness, removes none, and leaves 677 of 678 retained dumps
  equal after accounting normalization. The expanded Promise Realm witness is
  the sole retained structural change. At that checkpoint the Number-family
  CLI fixture remained independently red; the newer entry above records its
  T20 decimal-formatting repair. This is a bounded invariant/Realm checkpoint,
  not broad Array, Promise, Number or Test262 closure.

- Duplicate static import-attribute keys now have the typed
  `ModuleDuplicateImportAttributeKey` identity across import and export-from
  declarations. A prefix-anchored parser pattern prevents user-controlled
  export names from forging that classification, while a const ownership check
  and the exhaustive IR map keep the closed domain at 56 variants and the
  message-pattern table at 55 rows. Under the eight-core cap, `cargo xc` is
  green; the front and module-early cohorts pass `93/93` and `39/39`; and the
  exact Test262 duplicate-attribute filter passes `3/3` Wasm-AOT executions
  with every non-success bucket at zero. This is bounded typed-diagnostic
  evidence, not a measured pass gain, T07 closure or aggregate conformance.
- Strict-mode `delete` early errors now have distinct typed identities for
  identifier and private-reference operands. A vendored Boa repair places both
  families beneath one strictness guard and exhaustively recognizes
  private-ending optional chains, while sloppy undeclared private operands stay
  owned by `InvalidPrivateIdentifier`. The closed domain has 55 variants and
  the parse classifier has 54 rows; const parse-ownership witnesses and the
  exhaustive IR map make drift fail compilation. The capped serial front,
  early-module, retained-dependency graph and focused IR gates pass `89/89`,
  `38/38`, `1/1` and `3/3`; `cargo xc` and the release CLI build are green; and
  the exact 194-file cohort passes `386/386` Wasm-AOT executions with every
  failure and non-success bucket at zero and an exact completed-ID set match.
  This is typed diagnostic closure and bounded no-regression, not a measured
  pass gain, runtime delete/private-element support, T07 closure or aggregate
  conformance.
- Callable bodies containing a Use Strict Directive with non-simple parameters
  now have one typed `Early`/`SyntaxError` condition across declarations,
  expressions, methods, setters and arrows. Three narrow parser repairs make
  the producer boundary grammar-honest: private getters require `()`, class
  setters accept exactly one non-rest parameter, and the binding-identifier
  arrow path no longer carries an impossible non-simple-list branch. The
  closed domain has 53 variants, the parse classifier has 52 rows, and a source
  inventory test pins all 16 remaining parser producers. The capped
  serial front, retained-module and focused IR gates pass `85/85`, `38/38` and
  `3/3`; `cargo xc` is green; and the exact 110-file cohort passes `220/220`
  sloppy/strict Wasm-AOT executions with every failure and non-success bucket
  at zero. This is typed diagnostic closure and bounded no-regression evidence,
  not a measured pass gain, dynamic-source support, runtime parameter
  semantics, T07 closure or aggregate Test262 progress.
- Static `JSON.parse` reviver specialization lives in the private 295-line
  `lowering/static_json_parse.rs` owner. Its two-phase `prepare`/`finish`
  protocol snapshots and parses proven static input after callee acquisition
  but before argument effects, then consumes that proof only after the lowered
  reviver has callable-kind and known-target evidence. The specialized IR owns
  the callee, input and reviver operands, and the emitter evaluates them in
  source order before materializing the prepared value. Spread arguments, TDZ
  bindings and mutable loop or captured bindings remain dynamic. Static String
  facts live in a 33-line private owner keyed by binding storage identity, so
  an inner shadow cannot overwrite the same-spelled outer fact and scope exit
  removes only the inner entry. Dynamic reviver-target discovery and
  observation remain in the parent. This is an ordering and binding-lifecycle
  repair in addition to the original ownership split, not a Test262 count
  claim.
- Recursive throw-value inference now lives in
  `lowering/throw_inference.rs`: six methods form one private 895-line owner,
  with only block inference visible to its sole consumer in
  `lowering/try_statement.rs`. The byte-exact extraction reduces `lowering.rs`
  from 21,877 to 20,986 lines. Capped pre/post goldens cover 633 fixtures in
  635 byte-identical artifacts; both compile gates, three focused IR cohorts,
  two exact structure witnesses and three exact CLI witnesses pass. Sixteen
  mutation controls and independent semantic and policy reviews cover the
  ownership, call graph, exhaustive-match and recursive-wrapper invariants.
  This is an ownership result, not broad Test262, full-workspace behavior or
  throw-conformance progress.
- For-in lowering now lives in `lowering/for_in.rs`: the sole statement-facing
  lowerer and twelve owner-only helpers form one private 571-line module, while
  shared for-in/of environment, TDZ, scope and analysis helpers remain with
  their existing owners. The byte-exact extraction reduces `lowering.rs` from
  22,444 to 21,877 lines. Capped pre/post goldens cover 633 fixtures in 635
  byte-identical artifacts, both compile gates pass, and focused IR witnesses
  pass `8/8`, `2/2`, `1/1` and `1/1`. CLI evidence is unchanged from the clean
  parent at `4/7` for the `for_in_` filter and `2/3` supplemental exact
  witnesses; the same four object-order/object-keys failures predate the move.
  This is an ownership and no-regression result, not broad Test262,
  full-workspace or for-in conformance progress.
- For-of lowering now lives in `lowering/for_of.rs`: one owner carries every
  specialization decision plus the private `ForOfLoweringIr` proof. The
  Array-walk classifier present at the extraction checkpoint has since been
  deleted in favor of a resumable synchronous Iterator Record. The 1,026-line
  source-family move leaves a 1,036-line
  child and reduces `lowering.rs` to 22,444 lines; the lowering-only carrier no
  longer leaks through the public IR surface. Seven focused IR witnesses,
  thirteen structure checks and four exact CLI witnesses pass. Capped pre/post
  goldens cover 633 fixtures in 635 artifacts and are byte-identical. No broad
  Test262, full-workspace or for-of conformance improvement is claimed.
- The callable-parameter `Contains YieldExpression` / `Contains
  AwaitExpression` matrix now has closed typed conditions across declarations,
  expressions, methods and ordinary/async arrows. Existing fixed arrow
  wordings map to two codes keyed by the containment condition rather than the
  syntax form. Three narrow vendored-parser repairs preserve the enclosing
  Yield grammar in parenthesized async-arrow parameters and add the missing
  Await containment checks for async function expressions and async methods.
  The only exact pinned cohort is two files expanding to four sloppy/strict
  executions through the ordinary-arrow producers. No pinned source reaches
  any of the three repaired producers, whose evidence is the direct front-end
  and retained-module witnesses. This closes that bounded matrix, not T07 or
  aggregate parser closure.
  The capped serial front, retained-module and focused IR gates pass `81/81`,
  `37/37` and `3/3`, respectively, and `cargo xc` is green. The exact pinned
  ordinary-arrow cohort passes `4/4` sloppy/strict Wasm-AOT executions with
  every failure and non-success bucket at zero.
- Labelled-statement lowering now lives in `lowering/labelled_statement.rs`:
  one owner carries nested-label collection, target-kind classification,
  active-label stack management and final `Labelled` IR assembly while shared
  break/continue label types remain in the parent. The exact 68-line family
  move reduces `lowering.rs` from 23,502 to 23,434 raw lines; the child is 72
  lines. Five focused IR filters and three focused CLI filters pass. Pre/post
  golden captures pass `2/2`, contain 635 artifacts each and are byte-identical.
  No labelled-statement behavior or conformance change is claimed.
- Direct `using` or `await using` declarations in switch CaseClause and
  DefaultClause StatementLists now share one typed early-error code across
  Script, Module and retained dependency parsing. Nested blocks, loops,
  functions and direct `let`/`const` declarations remain valid clause
  boundaries. The capped front gate passes `61/61`, the IR early-error filter
  passes `3/3`, and the exact four-file cohort passes `8/8` sloppy/strict
  Wasm-AOT executions with every failure and non-success bucket at zero. This
  is bounded classification, not disposal execution, direct eval, all switch
  grammar, or broad T07 closure.
- `for-in` heads using `using` or `await using` now share one typed early-error
  code across Script, Module and retained dependency parsing. `for-of`,
  ordinary `let`/`const` `for-in`, and initialized `using` in classic `for`
  remain valid grammar siblings. The capped front gate passes `59/59`, the IR
  early-error filter passes `3/3`, and the exact two-file cohort passes `4/4`
  sloppy/strict Wasm-AOT executions with every failure and non-success bucket
  at zero. This is bounded classification, not disposal execution, direct eval,
  all iterable-loop grammar, or broad T07 closure.
- While-family lowering now lives in `lowering/while_loop.rs`: one owner
  carries ordinary/resumable `while` construction and the deliberate
  `do while` suspension refusal while shared loop-resumption helpers remain in
  the parent. The exact 99-line family move reduces `lowering.rs` from 23,601
  to 23,502 raw lines; the child is 106 lines. Five focused IR filters and
  three focused CLI filters pass. Pre/post golden captures pass `2/2`, contain
  635 artifacts each and are byte-identical. No while/do-while behavior or
  conformance change is claimed.
- If-statement lowering now lives in `lowering/if_statement.rs`: one owner
  carries static condition selection, branch-local var/global facts,
  post-branch joins, abrupt-completion result typing and generator yield-state
  splitting/merging. The exact 137-line family move reduces `lowering.rs` from
  23,738 to 23,601 raw lines; the child is 141 lines. Six focused IR filters
  and four focused CLI filters pass. Pre/post golden captures pass `2/2`,
  contain 635 artifacts each and are byte-identical. No if-statement behavior
  or conformance change is claimed.
- ScriptBody top-level `using` now has one typed early-error code for Boa's
  fixed-position post-parse producer. Nested Script boundaries remain valid,
  retained Modules allow both top-level `using` forms, and the earlier untyped
  parser rejection for top-level Script `await using` remains honest rather
  than being forced through a source-interpolating classifier. The capped front
  gate passes `57/57`, the IR early-error filter passes `3/3`, and the exact
  two-file cohort passes `4/4` sloppy/strict Wasm-AOT executions with every
  failure and non-success bucket at zero. This is bounded classification, not
  parser-reachability repair, disposal execution, or broad T07 closure.
- Classic `for` lowering now lives in `lowering/for_loop.rs`: one owner carries
  head validation, lexical TDZ/environment setup, flow merging, resumable-state
  construction and the final `For`/`GeneratorLoop` choice. The exact 209-line
  method move reduces `lowering.rs` from 23,947 to 23,738 raw lines; the child
  is 213 lines. Eight focused IR filters and three focused CLI filters pass.
  Pre/post golden captures pass `2/2`, contain 635 artifacts each and are
  byte-identical. No classic-for behavior or conformance change is claimed.
- ScriptBody `Contains NewTarget` now has one typed early-error code for direct
  and top-level-arrow-carried `new.target`. Ordinary functions, their nested
  arrows, constructors, methods and class static blocks remain valid; retained
  dependencies keep the separate `ModuleTopLevelNewTarget` code. The capped
  front gate passes `55/55`, the IR early-error filter passes `3/3`, and the
  exact two-file cohort passes `4/4` sloppy/strict Wasm-AOT executions with
  every failure and non-success bucket at zero. This is bounded Script
  classification, not direct-eval or broad T07 closure.
- ObjectLiteral `CoverInitializedName` now has one typed early-error code across
  Script, function-body, Module-item and class-static-block parser contexts.
  Assignment/binding reinterpretations, arrow parameters, shorthand and data
  properties remain parse-valid, and retained dependencies preserve the same
  `Early`/`SyntaxError` diagnostic. The capped front gate passes `53/53`, the IR
  early-error filter passes `3/3`, and the exact pinned witness passes `2/2`
  sloppy/strict Wasm-AOT executions with every failure and non-success bucket
  at zero. This is bounded classification, not broad ObjectLiteral or T07
  closure.
- Statement dispatch now lives in `lowering/statement.rs`: one exhaustive
  owner routes ordinary and resumable expression statements plus every
  control-flow/declaration form to its focused lowerer. The exact 255-line
  method move reduces `lowering.rs` from 24,202 to 23,947 raw lines; the child
  is 259 lines. Seven focused IR filters and four focused CLI filters pass.
  Pre/post golden captures pass `2/2`, contain 635 artifacts each and are
  byte-identical. No statement behavior or conformance change is claimed.
- Public static ordinary, generator, async, async-generator, getter and setter
  methods with the literal name `prototype` now share one typed early-error
  code across Script, Module and retained dependency parsing. Instance literal,
  public computed and private static names remain parse-valid, preserving the
  separate computed-key run-time installation rule. The capped front gate
  passes `51/51`, the IR early-error filter passes `3/3`, and the exact
  twelve-file pinned cohort passes `24/24` sloppy/strict Wasm-AOT executions
  with every failure and non-success bucket at zero. This is bounded diagnostic
  classification, not method execution or broad T07 closure.
- Class-static-block `ContainsAwait` now has one typed pre-evaluation code
  across Script, Module and retained dependency parsing. Its classifier uses
  the adjacent rendered fragment `invalid await usage at line`, keeping Boa's
  longer generator-parameter error distinct; positive tests preserve nested
  async ordinary and arrow function bodies. The capped front gate passes
  `49/49`, the IR early-error filter passes `3/3`, and the exact pinned
  Test262 witness passes `2/2` Wasm-AOT executions with every failure and
  non-success bucket at zero. This is bounded diagnostic classification, not
  broad T07 closure.
- New-expression lowering now lives in `lowering/new_expression.rs`: one owner
  carries constructor target resolution, argument evaluation, result typing,
  dynamic-source rejection and static RegExp compilation. The exact source
  move reduces `lowering.rs` from 24,446 to 24,202 raw lines; the child is 248
  lines. Five focused IR filters pass; the two Map/Set iterable-construction
  shape assertions fail identically at parent `394e8fda7`. Five focused CLI
  filters pass. Pre/post golden captures pass `2/2`, contain 635 artifacts each
  and are byte-identical. No constructor behavior or conformance change is
  claimed.
- Property-access lowering now lives in `lowering/property_access.rs`: one
  dispatcher owns ordinary, private and super reads plus primitive/exotic
  routing and unknown-effect invalidation. Its target-kind match names
  `ValueKind::Number` instead of hiding future variants behind a catch-all.
  This reduces `lowering.rs` from 24,663 to 24,446 raw lines; the child is 223
  lines. Focused IR property cohorts pass `2/2`, `6/6`, `1/1`, `1/1` and
  `34/34`; focused CLI cohorts pass `3/3`, `1/1` and `6/6`. Pre/post golden
  captures pass `2/2`, contain 635 artifacts each and are byte-identical. No
  property-access behavior or conformance change is claimed.
- Try/catch/finally lowering now lives in `lowering/try_statement.rs`: catch
  Environment Record construction, consumption of inferred thrown values,
  resumable-state planning and final try IR assembly move together. Private
  named catch/finally records replace the former eight- and five-field tuples,
  so generator and async entry/exit states cannot be transposed by positional
  access. Reusable throw-value inference now has its own private owner. This
  reduces `lowering.rs` from 24,910 to 24,663 raw lines. Focused IR coverage
  passes `12/12` for `try_` and `14/14` for `catch`; three CLI filters pass
  `2/2` each. Pre/post golden captures pass `2/2`, contain 635 artifacts each
  and are byte-identical. No try-statement behavior or conformance change is
  claimed.
- Delete-expression lowering now lives in
  `lowering/delete_expression.rs`: one exhaustive target dispatcher owns
  ordinary/private/super property References, identifiers and non-Reference
  values while reusable helpers remain in the parent. The exact 213-line
  method move reduces `lowering.rs` from 25,123 to 24,910 raw lines and changes
  only its private-module visibility. The capped workspace check is green;
  serial delete coverage passes `7/7` CLI, `2/2` AOT-Wasm and `4/4` engine
  tests. No delete behavior or conformance change is claimed.
- Assignment-expression lowering now lives in `lowering/assignment.rs`: one
  exhaustive dispatcher owns identifier, property, private, destructuring,
  logical and eager compound assignment while specialized Reference carriers
  remain in their typed modules. The exact 707-line method move reduces
  `lowering.rs` from 25,830 to 25,123 raw lines and changes only its
  private-module visibility. The capped workspace check and IR assignment
  cohort (`34/34`) are green; CLI assignment remains `6/7` before and after the
  move with the identical with-environment failure.
- Ordinary function-definition lowering now lives in
  `lowering/function_definition.rs`: nested lowerer state, parameters, body,
  captures, signatures, resumable metadata and final `FunctionIr` assembly
  move together, while shared helpers remain in the parent. The exact 717-line
  method move reduces `lowering.rs` from 26,547 to 25,830 raw lines and changes
  only its private-module visibility. Capped serial IR function coverage passes
  `61/61`; CLI function coverage reports `45/49`, with all four failures
  reproduced at the exact parent commit.
- Builtin call-result analysis now lives in
  `lowering/builtin_call_info.rs`: one exhaustive `StandardBuiltinId` table
  owns return kinds and shapes plus its narrowly related observation updates,
  while four lowering paths remain consumers. The exact 2,146-line method move
  reduces `lowering.rs` from 28,693 to 26,547 raw lines and changes only its
  private-module visibility. The capped workspace check and CLI `call_` cohort
  (`6/6`) are green; current IR `call_` is green at `34/34` after its
  materialized-receiver contract accepted canonical `GetV` alongside typed
  `PropertyRead`. A Wasm-AOT witness completes with `boolean(true)`.
- Atomics backend ownership now lives in `builtins/atomics.rs`: all fourteen
  intrinsic bodies, integer/RMW domains, wait/notify state and atomic-memory
  helpers sit behind one closed `AtomicsBuiltin` dispatch. A six-case RMW type
  removes four runtime `unreachable!` fallbacks, while three checked hooks serve
  the TypedArray, event-loop and Promise consumers. This reduces
  `builtins/standard.rs` from 33,275 to 30,567 raw lines without changing the
  emitted family bodies. Capped serial focused coverage passes `2/2` AOT-Wasm
  and `5/5` engine tests; the CLI cohort passes `12/13`, with its remaining
  `Atomics.isLockFree` core-fixture failure reproduced unchanged at the parent
  commit.
- Duplicate class private names now carry one typed early-error code across
  fields, methods, accessors and static/instance conflicts. Script, Module and
  retained-module paths agree on `Early`/`SyntaxError`; valid getter/setter
  pairs and nested-class private-name domains remain accepted. The capped
  serial front and focused IR gates pass `42/42` and `3/3`, and the exact
  32-file pinned Test262 cohort passes `64/64` sloppy/strict Wasm-AOT
  executions with every failure bucket at zero. This is bounded parser
  evidence, not class-grammar or aggregate closure.
- Public class-field literal-name restrictions now carry two typed early-error
  codes: non-static fields/auto-accessors reject literal `constructor`, while
  static forms reject literal `constructor` or `prototype`. All eight parser
  branches share the Script, Module and retained-module boundary; computed
  names and ordinary constructor methods remain valid. A narrow vendored
  parser repair stops identifier `constructor` fields from being misrouted
  into parameter parsing. Computed static `prototype` remains valid syntax but
  now throws at class definition after the required field-initializer ordering;
  methods, accessors and auto-accessors share that public-static guard, and the
  class constructor's own `prototype` descriptor is all-false. The exact
  18-file class-field cohort passes `36/36` executions, the adjacent nine-file
  runtime/descriptor cohort passes `18/18`, and the durable Wasm class-element
  fixture passes `1/1`.
- Strict-mode `with` statements now carry one typed early-error code from the
  sole parser producer through retained Module diagnostics. Strict Script and
  function bodies, class methods and Modules reject, while sloppy Script and
  function contexts remain valid. The capped serial front and focused IR gates
  pass `44/44` and `3/3`; the exact seven-file pinned cohort passes `7/7`
  Wasm-AOT executions with every failure bucket at zero. This is bounded
  parser classification only; valid sloppy `with` runtime semantics are
  unchanged.
- Class-field `ContainsArguments` parser rejections now carry one typed early-
  error code across public/private, instance/static and auto-accessor
  initializers. The front-end preserves lexical traversal through arrows and
  stops at ordinary function/method boundaries; retained dependency modules
  project the same `Early`/`SyntaxError` diagnostic. The focused front and IR
  gates pass `40/40` and `3/3`, and the exact 60-file pinned Test262 cohort
  passes `120/120` Wasm-AOT executions with every failure bucket at zero.
  Literal direct-`eval` source remains explicit T13 dynamic-source debt.
- Ordinary-property `&&=`, `||=` and `??=` now consume one fused Reference
  carrier. One base/receiver and raw key flow through nullish validation, a
  sole `ToPropertyKey`/GetValue transition, branch-local RHS and same-reference
  Set, strict false-Set routing and result publication only after normal
  PutValue. As a safe implementation optimization, the backend retains one
  boxed target `O` separately from the original receiver, preserving primitive
  accessor receivers through both Get and a taken Set; eager compound
  assignment and numeric update share that backend invariant. Possible writes
  also invalidate dependent global-property facts and Array prototype fast
  paths. At clean pre-batch commit `04e38f2ba`, the three exact
  strict `no-set-put.js` witnesses were `0/3` Runtime/Bug, while the three
  independent ordering files were already `6/6`. Workspace/all-target check,
  focused IR `2/2`, new structure `6/6`, affected retained structures `21/21`
  and the Wasm lifecycle fixture `1/1` in `76.52s` are green. The selected raw
  post-batch cohorts pass strict false-Set `8/8`, ordering `6/6` and
  short-circuit `3/3`, with every failure and NotImplemented/Crash/Bug bucket
  at zero. This is focused seventeen-execution evidence, not complete logical
  assignment or pinned-matrix closure. Subsequent implicit-hook effect and
  compact target-provenance hardening passed an eight-core-capped
  workspace/all-target check, the filtered ordinary-property IR suite
  (`49/49`), all four ordinary-property structure suites (`27/27`), the Wasm
  logical-assignment lifecycle fixture (`1/1`) and the complete current
  logical-assignment leaf (`132/132`, zero failure or non-success outcomes).
- Unicode 17 `Emoji_Keycap_Sequence` now has an exact finite RegExp
  property-of-strings representation: twelve `[#*0-9] FE0F 20E3` strings.
  Direct `\p{Emoji_Keycap_Sequence}` atoms and UnicodeSets union,
  intersection and subtraction reuse the canonical finite class-string set;
  the direct `iv` form is identical because every member is simple-case-fold
  invariant. Other string properties remain typed unsupported and
  negated-string-class early errors remain intact. At clean pre-batch commit `04e38f2ba`, the direct
  property file and one generated string-union representative were each `0/2`
  Runtime/NotImplemented. Workspace/all-target checking, focused IR `1/1`,
  retained structure `7/7`, the expanded Wasm fixture `1/1` in `24.04s`, and
  the exact 37-file/74-execution inventory `74/74` are green, including the
  three negative syntax files. Every failure and NotImplemented/Crash/Bug
  bucket is zero. This does not claim the remaining Unicode string properties
  or broader RegExp completion.

- A bounded RegExp batch now implements finite UnicodeSets `\q{…}` class-string
  algebra. At clean pre-batch commit `f580b424d`, one union, one intersection
  and one subtraction representative each reported `0/2` sloppy/strict
  Wasm-AOT executions: `string-literal-union-string-literal.js`,
  `string-literal-intersection-string-literal.js`, and
  `string-literal-difference-string-literal.js`. All six measured executions
  were `Runtime/NotImplemented` with `RegExp.prototype.exec unsupported
  pattern`. The compiler now retains a canonical range-and-string set through
  union, intersection and subtraction, emits longest strings before the
  singleton class and empty member, and uses the same exhaustive forward/reverse
  lowering. Central verification passed
  workspace/all-target checking, `cargo xc`, the focused IR invariant `1/1`,
  the bounded structure witness `7/7`, the source-free Wasm lifecycle fixture
  `1/1`, and the exact unmasked 27-file/54-execution Test262 cohort `54/54`
  with zero parser, early-error, lowering, runtime, Wasm-backend, harness,
  unsupported, crash or bug outcomes. The runtime fixture exposed and closed a
  reverse-lookbehind gap by sharing the canonical Unicode range-membership
  emitter in both matcher directions. Other Unicode properties of strings and
  direct class-string `/iv` folding remain explicit typed capability
  boundaries. This records no broader
  UnicodeSets or RegExp completion claim.
- A bounded matcher batch now implements RepeatMatcher's nullable
  unbounded-quantifier progress rule. At clean pre-batch commit `44247b836b`,
  exact unflagged Test262 file `built-ins/RegExp/nullable-quantifier.js`
  reported `0/2` sloppy/strict Wasm-AOT executions. Both were
  `Runtime/NotImplemented` with `RegExp.prototype.exec unsupported pattern`,
  and the path has no exact rewrite, materializer or known-failure entry. The
  durable CLI oracle covers the exact `(a?b??)*` result, rejection of only
  a zero-progress optional iteration while suffix backtracking remains live,
  greedy/lazy and required-minimum behavior, bounded and captured controls,
  nested nullable loops, reverse lookbehind compilation and global empty-match
  advancement. Central verification passed workspace/all-target `cargo check`
  and `cargo xc`; the focused IR test passed `1/1` in `8.37s`, the bounded
  structure executable passed `5/5` in `22.36s`, the new lifecycle fixture
  passed `1/1` in `22.83s`, and the retained quantifier fixture passed `1/1` in
  `27.19s`. The exact Test262 file now passes `2/2` with zero unsupported,
  crash or bug verdicts. This is focused evidence only: no broader RegExp or
  full-suite claim is made. Other Unicode properties of strings, runtime
  pattern compilation and the complete RegExp/String trees remain separate T19
  work.
- Non-resumable object-literal methods, getters and setters now carry their
  exact function identity and object-method protocol in a dedicated IR value;
  a generic function expression cannot enter any of the six method/accessor
  property rows. The AOT lifecycle pairs that carrier with the already
  allocated literal, installs the literal as `[[HomeObject]]` before defining
  the property, and keeps the invocation `this` distinct from the super base
  for reads and writes. The durable CLI oracle covers named and computed
  methods/accessors, a super read in a parameter initializer before the body,
  source-order computed keys around a static key, detached calls with an alien
  receiver, later literal-prototype replacement, and nonconstructability. At
  clean pre-batch commit `304e4bbad3`, the five exact Test262 files
  `language/expressions/object/method.js`,
  `language/expressions/object/method-definition/name-super-prop-body.js`,
  `language/expressions/object/method-definition/name-super-prop-param.js`,
  `language/expressions/object/getter-super-prop.js`, and
  `language/expressions/object/setter-super-prop.js` reported `0/10` sloppy and
  strict Script executions; every execution was `Runtime/NotImplemented` with
  ``unsupported in lila wasm-aot first slice: object literal method``. The
  shared workspace/all-target check and `cargo xc` are green; the focused
  IR invariant is `1/1`, the bounded structure executable is `5/5`, and the
  Wasm CLI fixture is `1/1` in 19.75s. The exact five-file cohort is now
  `10/10`, with zero unsupported, crash or bug outcomes.
  The adjacent lexical-arrow lifecycle is now verified against a closed
  owner-role analysis: an arrow can inherit invocation `this` and
  `[[HomeObject]]` only through an enclosing object/class method capability,
  while an intervening ordinary function remains a lexical boundary. At clean
  pre-batch commit `039253d27`, exact Test262 files
  `language/expressions/super/prop-dot-obj-val-from-arrow.js` and
  `language/expressions/super/prop-expr-obj-val-from-arrow.js` reported `0/4`
  sloppy and strict Script executions, all with the same object-literal-method
  Runtime/NotImplemented diagnostic. The workspace/all-target check is green;
  the focused IR invariant is `1/1`, the bounded structure executable is
  `4/4`, and the Wasm CLI fixture is `1/1` in 19.37s. Both exact files now pass
  `4/4`, with zero unsupported, crash or bug outcomes. The durable fixture
  covers named and computed reads, parameter-created and multiply nested
  arrows, detached alien receivers, and later prototype replacement. As
  controls, `language/expressions/object/concise-generator.js` remains `2/2`,
  the two
  `generator-super-prop-{body,param}.js` files were `4/4`, and the two
  `async-super-call-{body,param}.js` files were `4/4`. Those controls do not
  prove complete suspension-safe object-method transport. Async-generator
  object methods and the complete object-expression subtree remain separate
  gates. The lexical-arrow boundary and nonclaims are recorded in
  `docs/rust-rewrite/contracts/object-method-arrow-super.md`.
- Numeric update and eager arithmetic/bitwise compound assignment through a
  non-resumable `super` property now use a fused Reference lifecycle. The IR
  owns receiver, raw key, strictness and one closed mutation operation; the AOT
  lifecycle
  retains the evaluated super base through the sole `ToPropertyKey`, GetValue,
  arithmetic and PutValue transitions. The fixture makes prototype mutation
  during key coercion observable with the exact traces
  `key,getA,rhs,setA:3:true` and `key,getA,setA:2:true`, including a detached
  alien receiver, and also covers every prefix/postfix increment/decrement
  form for Number and BigInt, strict failed Set, and uninitialized-`this`
  ordering. At the near-HEAD pre-batch `b0d1d1300` boundary, the four exact
  `language/expressions/super/prop-expr-{getsuperbase-before-topropertykey,uninitialized-this}-putvalue-{increment,compound-assign}.js`
  files reported `2/8`: both increment files were `0/4`
  Runtime/NotImplemented, the uninitialized-`this` compound file was `0/2`
  Runtime/Bug, and the GetSuperBase compound guard remained `2/2`. The
  available binary preceded that commit by four minutes, so this is explicitly
  near-HEAD evidence. Post-batch verification is green: workspace check and
  `cargo xc`; the focused IR invariant `1/1`; the bounded structure executable
  `5/5`; the compiled Wasm fixture `1/1` in `10.82s`; the exact cohort `8/8`;
  and the two adjacent `uninitialized-this` and
  `getsuperbase-before-topropertykey` filters `8/8` each, all with zero
  unsupported, crash or bug outcomes. Logical super assignment, private
  mutation, suspension and the broader super-expression matrix remain
  unclaimed. The boundary and exclusions are recorded in
  `docs/rust-rewrite/contracts/super-property-reference-mutation.md`.
- Computed ordinary-property eager arithmetic and bitwise compound assignment
  now uses one fused Reference lifecycle. The private producer
  plan owns the evaluated base/receiver, raw key and `[[Strict]]`; its consuming
  operation mints the old-value read and one of the twelve closed eager
  operations. The durable CLI oracle covers all twelve operators (including
  the local `**=` boundary), base and raw-key abrupt completion, nullish-base
  rejection before `ToPropertyKey`, one canonical key across `[[Get]]`, RHS
  and `[[Set]]`, Proxy/accessor receiver identity, RHS mutation of the raw key,
  strict false-Set rejection and result publication only after PutValue. At
  clean pre-batch commit `ae1bd994b`, a fresh raw run of the complete legacy
  `language/expressions/compound-assignment/S11.13.2_A7.1..11_T1..4.js`
  matrix measured `22/88`: all 22 T3 control executions passed, while all 66
  T1, T2 and T4 executions were `Runtime/Bug`. No rewrite, matrix mask or
  known-failure entry owns those results. Post-batch verification is green:
  workspace/all-target check and `cargo xc`; the focused IR invariant `1/1`;
  the bounded structure executable `7/7`; retained Super, `with`, and global
  compound-assignment structures `5/5`, `5/5`, and `4/4`; the compiled Wasm
  lifecycle fixture `1/1` in `75.42s`; and the exact raw matrix `88/88`, with
  zero unsupported, not-implemented, crash, or bug outcomes. This focused
  batch does not change plain,
  logical or numeric property assignment, `super`, private, identifier,
  global/Object Environment, `with`, or suspending property References.
- Computed ordinary-property prefix/postfix `++` and `--` now use the adjacent
  fused numeric-update Reference lifecycle. The same
  non-copyable producer plan consumes one evaluated base/receiver, raw key and
  captured `[[Strict]]` into closed increment/decrement and prefix/postfix
  domains. The durable CLI oracle covers all eight Number/BigInt combinations,
  old-versus-new result selection, base/raw-key/`ToPropertyKey`/`ToNumeric`
  abrupt paths, one canonical key and receiver across get/set, mutation of the
  raw key during coercion, strict false-Set rejection, sloppy false-Set
  behavior, and publication only after PutValue. At pre-batch head
  `0f004c0c6`, the four raw A6 T1 files (eight sloppy/strict executions) were
  freshly `0/8`, all `Runtime/Bug`: a throwing key coercion incorrectly won
  over the required nullish-base `TypeError`. No runner rewrite, matrix mask or
  known-failure entry owns them. Post-batch verification is green:
  workspace/all-target check; the focused IR invariant `1/1`; the new and
  retained eager-compound structure executables `7/7` each; the compiled Wasm
  lifecycle fixture `1/1` in `60.43s`; and the exact raw cohort `8/8`, with
  zero unsupported, not-implemented, crash, or bug outcomes.
  Eager/logical/plain assignment, `super`, private,
  identifier/global/Object Environment, `with`, optional-chain and suspended
  References remain outside this focused batch.
- Plain assignment through an ordinary property Reference now uses a focused
  staging seam. A private consuming producer plan owns
  one evaluated base/receiver, one raw computed key, the RHS and captured
  `[[Strict]]`; the AOT boundary performs PutValue in the order base,
  raw key, RHS, nullish `ToObject` validation, exactly one `ToPropertyKey`,
  `[[Set]]`, strict-false routing and only then RHS-result publication. The
  durable CLI oracle makes this order observable through Proxy/accessor
  receiver traces, RHS-before-coercion key mutation, nullish and abrupt paths,
  exactly-once evaluation, strict and sloppy false Set results, and primitive
  receivers.

  At clean pre-batch head `eb32c63a`, the exact raw Test262 files
  `language/expressions/assignment/target-member-computed-reference-null.js`
  and `target-member-identifier-reference-null.js` were each freshly `0/2`
  `Runtime/NotImplemented`, while
  `target-member-identifier-reference-undefined.js` was `1/2`: strict passed
  and sloppy was `Runtime/Bug`. The selected three-file, six-execution
  baseline is therefore `1/6`. The adjacent
  `target-member-computed-reference-undefined.js` and
  `target-member-computed-reference.js` controls were each `2/2`. No runner
  rewrite, matrix mask or known-failure entry owns these results. Post-batch
  verification is green: the workspace/all-target check in 15.18 seconds and
  cached `cargo xc` in 0.17 seconds; the focused IR invariant `1/1` in 6.85
  seconds after an 8.25-second build; the new structure executable `7/7` in
  0.01 seconds after a 20.76-second build; retained eager-compound and numeric
  structures `7/7` each in 0.22 and 0.02 seconds; and the exact Wasm CLI fixture
  `1/1` in 66.90 seconds. The three selected raw files now pass all `6/6`
  executions with zero unsupported, not-implemented, crash or bug outcomes, while both adjacent
  controls remain `4/4`. Focused runtime verification removed only an
  unsupported `(1).p` property-read assertion from the fixture; its sloppy and
  strict primitive-assignment oracles remain. These focused results do not
  claim the broader assignment leaf. Destructuring,
  `super`, private, identifier/global/Object Environment, `with`, optional-chain
  and resumable assignments remain outside this focused boundary.
- Direct identifier calls selected through `with` now have a verified,
  Reference-preserving lowering seam. A private non-copyable
  `WithEnvironmentIdentifierCallReferencePlan` consumes the analyzed non-empty
  Object Environment chain and can produce a selected indirect call only with
  the same binding object as both GetBindingValue source and explicit `this`;
  its complete ordinary fallback retains undefined-this semantics. The
  lowerer intercepts this form before name-specific builtin folds, locates the
  fallback before observable HasBinding, and clears mutable fallback value and
  function-target facts before lowering arguments. The durable CLI oracle
  covers the exact selected receiver, getter deletion with the retained base,
  `HasProperty`/unscopables/Get/call order, arguments after callee evaluation,
  nested-unscopables selection, strict and sloppy fallback `this`, selected
  builtin shadowing, an empty-with builtin fallback, and a declining Proxy
  `has` trap that replaces the fallback function. Package checks and `cargo xc`
  pass; the focused IR filter is `2/2`, the bounded structure executable is
  `4/4`, and the exact CLI fixture is `1/1` in 23.19 seconds. The broader CLI
  `environment` slice is `13/13` in 315.07 seconds. One test-only hardcoded-key
  expectation was corrected during the focused IR rerun. The exact no-strict
  Test262 file `language/expressions/call/with-base-obj.js` now passes `1/1`
  with zero unsupported, crash or bug outcomes. These focused results do not
  claim the complete call/with subtree or pinned matrix is green.
- Object Environment identifier `&&=`, `||=` and `??=` now have a verified
  Reference lifecycle for both global and `with` resolution. The existing
  closed logical-op enum feeds one private binding-object operation whose
  PutValue is structurally inside only the taken short-circuit branch; distinct
  consuming with/global plans own selection, strictness and the same binding
  object across GetValue and SetMutableBinding. A non-copyable pre-RHS Reference
  carrier snapshots any proven-global value metadata before lowering the RHS,
  preventing an untaken RHS write from changing the old value's emitted tag.
  The durable CLI oracle covers all three modes, initial global misses before
  RHS, dynamic global short circuits and taken writes, strict getter deletion,
  sloppy recreation, selected/declarative/nested-unscopables `with` paths, and
  an observable `huhgdrhs` selection/Get/RHS/Put trace. Package checks for
  `lila-ir`, `lila-aot-wasm` and `lila-cli`, plus `cargo xc`, pass; the two
  focused IR lifecycle tests are `2/2`, four final source-bounded structure
  executables are `4/4`, and the exact Wasm fixture is `1/1` in 87.88 seconds.
  The broader focused environment test selection is `12/12` in 270.93 seconds.
  The three selected strict unresolved-lhs Test262 files now pass `3/3`, and
  the six adjacent unresolved-RHS physical files pass all `12/12` sloppy and
  strict executions. One stale derive marker was corrected during the final
  structural rerun; product code was unchanged. No vendored logical-assignment
  file contains `with`, so that behavior remains fixture evidence rather than
  part of the exact Test262 counts. These focused results do not claim the
  complete language subtree or pinned matrix is green.
- Global Object Environment identifier `++` and `--` now use a verified
  Reference lifecycle beside the retained `with` lifecycle. One private
  fixed-role numeric carrier feeds the shared Object Environment
  GetBindingValue/ToNumeric/SetMutableBinding operation, while a distinct
  non-copyable global plan performs the initial plain `HasProperty` without an
  unscopables query. The durable CLI oracle covers all four prefix/postfix
  forms, Number and BigInt results, an initially missing binding throwing from
  GetValue before ToNumeric, strict getter deletion without recreation, sloppy
  getter deletion with recreation, and an observable
  HasBinding/GetBindingValue/SetMutableBinding trace. Its bounded source witness
  owns the four exact global Test262 files, the four already-green bare-suffix
  `with` controls, and the eleven already-green global eager-compound files as
  regression gates. At pre-batch commit `f6b6af6a`, the exact global
  prefix-increment witness reported `0/1` as `Runtime/NotImplemented` with
  ``unsupported in lila wasm-aot first slice: unbound identifier `x```; the
  adjacent plain-assignment witness reported `1/1`. The other three selected
  numeric files are source-proven to have reached the same refusal but were not
  separately measured pre-batch. The affected `lila-ir`, `lila-aot-wasm` and
  `lila-cli` package checks and `cargo xc` are green; the focused IR test is
  `1/1`, four source-bounded structure executables total `17/17`, and the Wasm
  lifecycle fixture is `1/1` in 45.02 seconds. The exact selected global
  numeric cohort now passes `4/4`, its bare-suffix `with` controls remain `4/4`,
  and the modern eager-compound prefix remains `22/22` with zero unsupported,
  crash or bug outcomes. These are focused current-batch results, not a full
  language-subtree or pinned-matrix publication.
- Global Object Environment eager compound assignment now has a verified
  Reference lifecycle beside the retained `with` lifecycle. A distinct
  non-copyable plan performs the global Object Record's initial plain
  `HasProperty`, then consumes the same sealed old-value/result/write carrier
  through independent GetBindingValue and SetMutableBinding rechecks; the
  global path cannot carry `Symbol.unscopables`. The durable CLI oracle covers
  all eleven directly evidenced operators, an initially absent binding
  throwing before RHS evaluation, strict accessor deletion without recreation,
  sloppy accessor deletion with recreation, inherited selection and result
  publication only after PutValue succeeds. At pre-batch commit `450f67050`,
  the exact modern filename prefix reported `11/22`: all eleven already-green
  `with` siblings passed, while the eleven selected global siblings were
  `Runtime/NotImplemented` with the diagnostic ``unsupported in lila wasm-aot
  first slice: unbound identifier `x```.
  The affected-package compile is green; the IR lifecycle test is `1/1`, the
  new source-bounded suite is `4/4`, the retained compound/numeric suites are
  `5/5` and `4/4`, and the Wasm lifecycle fixture is `1/1`. All eleven selected
  Test262 executions now pass `11/11`; the adjacent modern prefix is `22/22`,
  retaining every `with` sibling with zero unsupported, crash or bug outcomes.
  `**=` is covered by the closed Rust operation but has no twelfth direct
  Test262 witness, and neither the full language subtree nor the pinned matrix
  is claimed.
- Eager identifier compound assignments inside `with` now use one sealed,
  consuming Object Environment Reference lifecycle. The lowerer
  exhaustively separates the six arithmetic and six bitwise operators from
  short-circuiting logical assignment, while an opaque fixed-role carrier
  orders GetBindingValue, RHS/application, same-base SetMutableBinding and the
  returned value without adding a parallel backend operation. The durable CLI
  oracle covers all twelve operators, selected-object identity across getter
  deletion and RHS effects, strict post-Get deletion, function/global/outer
  fallbacks, and run-time fallback mutation, deletion and creation. A bounded
  source witness pins the exact current-source Test262 inventory of 44
  `noStrict` files (44 executions): 33 historical function/global/nested-object
  cases and 11 strict nested-function SetMutableBinding rechecks. `**=` has the
  same closed local invariant coverage but no forty-fifth direct vendored
  witness. The IR domain test is `1/1`, the source-bounded suite is `5/5`, the
  retained numeric-reference suite remains `4/4`, the Wasm lifecycle fixture is
  `1/1`, and the exact current-source Test262 cohort is `44/44`. The adjacent
  global Object Environment follow-up remains separate from this focused
  `with` claim.
- Identifier `++` and `--` inside `with` now consume the same non-empty,
  non-copyable Object Environment Reference plan as direct reads and writes.
  Each selected branch fixes one binding object across GetBindingValue's second
  `HasProperty`, ToNumeric and the delta, then SetMutableBinding's post-Get
  `HasProperty`; strict nested-function References throw before Set when a
  getter deleted the property, while sloppy References recreate it without
  falling through to an outer binding. A mutating `@@unscopables` getter also
  forces a pre-located Number fallback to become BigInt before the object record
  declines it, pinning a Dynamic fallback update and all-runtime-tags metadata.
  Proxy `has` traps also prove both sides of the run-time global fallback
  boundary: deleting a previously proven global must throw rather than recreate
  it, while creating a previously unresolved global must admit the update. A
  durable CLI oracle and bounded source witness cover all four prefix/postfix
  forms, their returned values and the exact current-pin inventory of 16
  `noStrict` files (16 executions). At pre-batch commit
  `156aeb38b28378e04bb852f8d00679f47b401d34`,
  `prefix-increment/operator-prefix-increment-x-calls-putvalue-lhs-newvalue-.js`
  and
  `postfix-decrement/operator-x-postfix-decrement-calls-putvalue-lhs-newvalue-.js`
  each reported `0/1` as `Runtime/NotImplemented` with the exact diagnostic
  ``unsupported in lila wasm-aot first slice: unbound identifier `x```.
  The integrated IR invariant is `1/1`, the source-bounded contract suite is
  `4/4`, the Wasm lifecycle fixture is `1/1`, and the exact current-source
  Test262 cohort is now `16/16`; these are focused results, not a full-suite
  status publication.
- Primitive String computed-property reads now preserve every non-index key for
  the ordinary `ToPropertyKey` and `%String.prototype%` path, while canonical
  indices use UTF-16 own-property lookup and out-of-bounds indices fall through
  to the prototype. The formerly unsupported non-index witnesses are `2/2`
  execution variants each, and the adjacent pinned `15.5.5.5.2` family is
  `28/28` under Wasm-AOT. This is focused evidence, not String-tree closure.
- Class elements now report specific early `SyntaxError` codes for non-static
  async methods, getters and setters named `constructor`, and for every private
  `#constructor` form. The adjacent expression and statement early-error
  subtrees are each `444/444` under Wasm-AOT; this is bounded parser evidence,
  not full language or aggregate closure.
- Non-resumable synchronous `using` declarations that are direct children of
  ordinary blocks or function bodies now lower to the dedicated, statically
  non-empty `StatementIr::SyncDisposableScope` capability instead of generic
  `TryFinally`. The Wasm consumer acquires each `@@dispose` method before
  initializing its lexical binding, captures every outgoing completion, walks
  registered resources in reverse, continues after disposer throws, folds
  nested `SuppressedError` values and restores the final completion exactly
  once. A bounded source witness and CLI consumer cover TDZ method
  acquisition, nullish skipping, LIFO, initializer/return/body abrupt paths and
  suppression descriptors. The integrated current-SHA checkpoint is green:
  `cargo xc`, 3/3 focused IR tests, 4/4 structure tests and the end-to-end CLI
  consumer pass. The exact 18-file non-dynamic lifecycle cohort is 36/36 under
  Wasm-AOT. This is focused evidence, not a claim about the complete 78-file
  `language/statements/using` directory or the full pinned aggregate.
- Plain synchronous generators now carry statement-list `using` scopes through
  the required `SyncDisposableScopeExecutionIr::{Immediate, PlainGenerator}`
  owner domain. Analysis exhaustively names ordinary, generator, async-function
  and async-generator owners; only the generator route can mint the private
  `PlainGeneratorSyncDisposableCapabilityIr` through the suspension-owned
  binding allocator. The Wasm backend publishes that activation-backed
  capability when execution first reaches the declaration, retains it across
  every `yield`, and consumes a non-`Copy` storage witness into a detached
  capability only when the scope completes. The detached path marks the record
  disposed, clears its live entries, materializes the registered resources and
  reuses the existing LIFO completion fold before publishing normal, external
  `return()` or external `throw()` results. The durable generator fixture also
  covers acquisition failure, nested capabilities, disposer errors,
  `SuppressedError` ordering and exactly-once disposal. At pre-batch source
  commit `904da7b355811ad399ff284bf0ddeac47d2cc9c2`, the exact unflagged
  `language/statements/using/initializer-disposed-at-end-of-generatorbody.js`
  witness reported `0/2` Wasm-AOT executions, both
  `Runtime/NotImplemented` with the diagnostic `using declaration in a
  generator or async function`. The integrated current-SHA checkpoint is green:
  the workspace/all-target check and `cargo xc` pass after correcting one stale
  exhaustive lowering match, the focused IR invariant is `1/1`, the bounded
  structure suite is `6/6`, and the generator CLI fixture is `1/1` in 55.90
  seconds. The fixture retains a nested non-yielding scope; only the unsupported
  nested-yield shape was removed before the passing run. The exact Test262
  witness is now
  `2/2` with zero unsupported, crash or bug results, and the retained ordinary
  synchronous-using fixture remains `1/1` in 42.05 seconds. This focused batch
  does not claim async functions or generators, `await using`, resource-bearing
  classic-`for`/`for-of` heads beyond their separate batches, modules, dynamic
  source, the complete 78-file `language/statements/using` directory or the full
  pinned aggregate.
- The adjacent plain-async-function batch implements the required
  `SyncDisposableScopeExecutionIr::AsyncFunction` owner and its private
  `AsyncFunctionSyncDisposableCapabilityIr`, minted only through the
  suspension-owned binding allocator. The AOT backend exhaustively converts
  the distinct generator/async IR proofs into one non-`Copy`
  `ActivationSyncDisposeOwner`, which selects the owning execution kind,
  resume-state offset, resumable body compiler and terminal completion
  dispatcher. For a plain async function that means
  `HEAP_ASYNC_RESUME_STATE_OFFSET`, retention through `AsyncAwait`, and the
  `DispatchAsyncFunction` path only after the capability has been detached, its
  entries disposed in reverse and the folded completion restored. A durable
  CLI oracle covers no acquisition before call, retention at the first await,
  normal and explicit-return completion, source throw, rejected-await
  resumption, acquisition failure, nested non-await scopes, LIFO suppression
  and exactly-once disposal. At pre-batch source commit
  `1f27bc71f678d5b27e08d2719c660b9777021af4`, both executions of the exact
  async-flagged source file
  `language/statements/using/initializer-disposed-at-end-of-asyncfunctionbody.js`
  reported `Runtime/NotImplemented` with the diagnostic `using declaration in
  an async function or async generator`. The shared workspace/all-target check
  and `cargo xc` are green; the focused IR invariant is `1/1`; the async and
  retained generator structure executables are `7/7` and `6/6`; the async CLI
  lifecycle oracle is `1/1` in 15.21 seconds; and the retained generator oracle
  remains `1/1` in 55.21 seconds. The exact async Test262 witness is now `2/2`
  with zero unsupported, crash or bug results. Async generators, `await using`,
  `await` inside a `using` initializer, resource-bearing loop heads, modules,
  dynamic source, the complete 78-file `language/statements/using` directory
  and the full pinned aggregate remain explicit nonclaims.
- The adjacent async-generator synchronous-`using` batch is verified around the
  fourth required execution owner,
  `SyncDisposableScopeExecutionIr::AsyncGenerator`, and its private
  `AsyncGeneratorSyncDisposableCapabilityIr`. The closed
  `ActivationSyncDisposeOwner` maps that proof to
  `FunctionExecutionKind::AsyncGenerator`,
  `HEAP_ASYNC_GENERATOR_RESUME_STATE_OFFSET`, the existing async body compiler
  and `DispatchAsyncGenerator`; generator-only and async-function-only offsets
  cannot be selected without changing an exhaustive match. The shared
  activation-backed capability is initialized only when a request first
  reaches the declaration, retained through both `yield` and `await`, then
  detached and disposed before the current request completes or later requests
  drain. A durable CLI oracle covers pre-start/yield/await retention, normal
  completion, external `return()` and `throw()`, rejected-await resumption,
  acquisition failure, a nested non-suspending scope, LIFO suppression,
  exactly-once disposal, queued requests and a request synchronously enqueued
  by a disposer. The reentrant oracle records both promise reactions after
  disposal and the queued reaction before the current-request reaction, as
  observed from the host/spec request-drain order. At pre-batch source commit
  `a5606a73cbbb2a8ffd81c0c2e2dee945bb2b9a4b`, both executions of the exact
  async-flagged file
  `language/statements/using/initializer-disposed-at-end-of-asyncgeneratorbody.js`
  reported `Runtime/NotImplemented` with the exact diagnostic `unsupported in
  lila wasm-aot first slice: using declaration in an async generator`. The
  shared workspace/all-target check and `cargo xc` are green; the focused IR
  invariant is `1/1`; the async-generator, retained async-function and retained
  generator structure executables are `7/7`, `7/7` and `6/6`; and their CLI
  lifecycle oracles are `1/1` in 16.81, 13.09 and 53.84 seconds respectively.
  The exact async-generator Test262 witness is now `2/2` with zero unsupported,
  crash or bug results. Central verification also fixed the dispatcher
  preflight and suspension scanner to recurse through the typed async-generator
  scope.
  `await using`, async disposers, suspension inside a resource initializer,
  resource loop heads, modules, dynamic source, nonlinear async-generator
  forms, the complete `using` tree and the full pinned aggregate remain
  explicit nonclaims.
- The adjacent plain-async-function `await using` batch is implemented as a
  distinct `StatementIr::AsyncDisposableScope`, not a flag on synchronous
  `using`. Its private non-empty resource list and activation-owned capability
  carry a four-state finalizer plan whose strictly ordered entry, dispose,
  resume and exit states cannot overlap source `await` states. Acquisition
  observes `@@asyncDispose` first, uses the spec wrapper only for the
  `@@dispose` fallback, registers before binding initialization and retains the
  capability across every disposal Await. The backend's closed Empty, async
  method and sync-fallback entry kinds keep the fallback's ignored normal
  return separate from a direct async method result. A durable CLI oracle covers
  both lookup routes, receiver identity, TDZ/acquisition ordering, an empty
  resource's required Await versus an unreachable declaration, strictly
  sequential reverse awaits, normal/return/throw/rejection completion, nested
  LIFO, `SuppressedError` order and exactly-once disposal. At pre-batch source
  commit `7a89e27ec79fe6210fff04a58b6bb3eace535e09`, the exact
  `initializer-Symbol.{asyncDispose,dispose}-called-at-end-of-asyncfunctionbody.js`
  files reported `0/4`: all four sloppy/strict Script executions were
  `Runtime/NotImplemented` with the exact diagnostic `unsupported in lila
  wasm-aot first slice: await using declaration`. Central verification is now
  green for `cargo check --workspace --all-targets`, `cargo xc`, the focused
  `lila-ir` `await_using` tests (`2/2`, including capture ownership), the
  bounded IR/AOT structure executable (`6/6`), the complete CLI lifecycle
  fixture (`1/1` in 13.10 seconds), and the retained synchronous-using CLI
  family filter (`6/6` in 58.29 seconds). The two exact Test262 paths are now
  `4/4` with zero unsupported, crash or bug results. The other 47 positive
  plain-async statement-list files are an explicit regression inventory, not a
  broad `49/49` claim. Async generators, resource loop heads, modules, dynamic
  source, suspension inside an initializer, nonlinear async control flow, the
  complete `await using` directory and the full pinned aggregate remain outside
  this batch.
- The adjacent async-generator `await using` path is now verified against a
  distinct `AsyncDisposableScopeExecutionIr::AsyncGenerator` capability. The
  durable CLI oracle keeps that activation-owned capability live across both
  `yield` and body `await`, then covers normal completion, external return and
  throw, awaited rejection, direct `@@asyncDispose`, the ignored-return
  `@@dispose` fallback, later acquisition failure, nested LIFO,
  `SuppressedError`, exactly-once disposal, queued requests and synchronous
  reentrancy from an async disposer. Unlike synchronous disposal, its awaited
  disposer records the current-request reaction before the queued reaction,
  with both reactions after disposal. At source commit `5ad393f3d0`, the exact
  `initializer-Symbol.{asyncDispose,dispose}-called-at-end-of-asyncgeneratorbody.js`
  files reported `0/4`: all four sloppy/strict Script executions were
  `Runtime/NotImplemented` with `unsupported in lila wasm-aot first slice: await
  using declaration in an async generator`, and neither path had a rewrite,
  mask or known-failure entry. Central verification is green for `cargo check
  --workspace --all-targets`, `cargo xc`, the focused `lila-ir`
  `async_generator_await_using` tests (`2/2` in 12.34s, including the exact
  state-collision invariant), the new bounded structure executable (`5/5`),
  and the retained plain-async structure executable (`6/6`). The async-generator
  lifecycle fixture passes `1/1` in 23.63s; the retained plain-async await-using
  and synchronous async-generator using fixtures pass `1/1` in 11.96s and
  `1/1` in 16.56s. Both exact files now pass `2/2`, for `4/4` total with zero
  unsupported, crash or bug outcomes. The state-collision repair reserves the
  three implicit finalizer states before the following suspension and makes
  AOT assert that each resumable statement entry continues the preceding exit.
  Classic-`for` and `for-of` resource heads, modules, dynamic source, binding
  patterns, suspension inside a resource initializer, nonlinear async-generator
  forms, the complete `await using` directory and the full pinned aggregate
  remain explicit nonclaims.
- The adjacent plain-async classic-`for` `await using` batch uses the closed
  `ForInitIr::AsyncDisposable(AsyncDisposableForInitIr)` capability. The direct
  `StatementIr::For` remains the label target, while its nonempty resource list
  and activation-owned finalizer span initializer acquisition, test, every body
  and update, and terminal completion. The source-free CLI oracle contains no
  explicit Await or Yield expression beyond the `await using` declaration and
  covers async-first lookup, sync fallback, body-before-disposal, normal exit,
  local break/continue, labelled control targeting the resource loop, return,
  throw, abrupt test and update,
  later acquisition failure, LIFO `SuppressedError`, loop-environment capture
  and exactly-once disposal. At clean pre-batch commit `bca90f2ff9`, the exact
  `initializer-Symbol.{asyncDispose,dispose}-{called-at-end-of-forstatement,called-if-subsequent-initializer-throws-in-forstatement-head}.js`
  cohort reports `0/8`: every sloppy/strict Script execution is
  `Runtime/NotImplemented` with `unsupported in lila wasm-aot first slice: await
  using declaration`, and none has a rewrite or known-failure mask. Central
  verification is green for `cargo check --workspace --all-targets`, `cargo
  xc`, the focused IR test (`1/1` in 12.11s), and the structure executable
  (`5/5`). The new CLI fixture passes `1/1` in 22.81s; retained plain-async and
  async-generator await-using fixtures pass `1/1` in 12.00s and 22.60s, and the
  retained synchronous classic-for fixture passes `1/1` in 30.22s. The four
  exact files now each pass `2/2`, for `8/8` total with zero unsupported, crash
  or bug outcomes. Runtime verification caught an over-broad generic Labelled
  state scan; the repair forwards resumable state only through a transparent
  label chain ending directly in an async-disposable For. Async generators,
  ordinary and generator owners, modules, dynamic source, binding patterns,
  `for-of` and `for-await-of`, source suspension in any loop region, the
  complete `await using` directory, outer labelled-block or enclosing-loop
  control, repeated or nonlinear re-entry of the same resource-loop node, and
  the full pinned aggregate remain explicit nonclaims.
- The plain-async resource-loop batch now supports synchronous `for-of` with
  an `await using` head. Its source-free CLI oracle keeps the
  generic iterator protocol observable and covers async-first lookup, the
  ignored-return synchronous fallback, fresh captured bindings, head TDZ and
  immutability, disposal before the next iterator step, local continue without
  close, disposal before break/return/throw/IteratorClose, later-iteration
  acquisition failure, the outer head binding surviving a nested implicit
  finalizer, nested LIFO `SuppressedError` folding and exactly-once disposal.
  At clean pre-batch commit `009219b28`, the two per-iteration
  `Symbol.{asyncDispose,dispose}` protocol files, the for-head TDZ file, the
  immutable-assignment file and the `for (await using of of ...)` grammar file
  report `0/10`. Every sloppy/strict Script execution is
  `Runtime/NotImplemented` with the exact diagnostic `unsupported in lila
  wasm-aot first slice: await using declaration in for-of`; none has an exact
  Wasm-AOT rewrite or known-failure entry. Central verification is green for
  `cargo check --workspace --all-targets`, `cargo xc`, the focused IR test
  (`1/1` in 12.17s), and the bounded structure executable (`5/5`). The new CLI
  lifecycle fixture passes `1/1` both in a cached central rerun (`0.23s`) and
  an uncached focused run (`14.25s`); the retained async await-using fixtures
  pass `4/4` in 37.83s, and the retained synchronous using-for-of fixture
  passes `1/1` in 48.82s. Each of the five exact raw Test262 files now passes
  `2/2`, for `10/10` total with zero unsupported, crash or bug outcomes. This
  is focused evidence only: the Module-only fresh-binding witness,
  `for-await-of`, async generators, binding patterns, dynamic source, the
  complete `await using` directory and the full pinned aggregate remain
  explicit nonclaims.
- The adjacent classic-`for` extension gives a synchronous
  `using` head the closed, statically non-empty
  `ForInitIr::SyncDisposable(SyncDisposableResourcesIr)` capability while
  retaining the direct `StatementIr::For` node needed by labelled break and
  continue. Every head binding enters TDZ before acquisition; when a captured
  binding materializes a for-head environment, it encloses acquisition, test,
  body, update and eventual disposal. Continue retains the capability, while
  normal or abrupt loop exit consumes it through the existing LIFO completion
  fold.
  A focused CLI oracle covers labelled continue/break, nullish acquisition,
  outer/inner binding isolation, a later binding's observable TDZ during the
  first resource GetMethod, false-test LIFO, later initializer failure and
  suppression, and immutable-binding update failure. The exact adjacent
  vendored inventory is five files: three adjacent grammar/binding witnesses
  plus two focused disposal-lifecycle witnesses. The integrated current-SHA
  checkpoint is green: `cargo xc`, 4/4 focused IR tests, 5/5 structure tests
  and the end-to-end CLI consumer pass. Those five files report 10/10
  sloppy/strict Wasm-AOT executions. This remains focused evidence, not a claim
  about the complete 78-file `language/statements/using` directory or the full
  pinned aggregate.
- Synchronous `using` in `for-of` heads keeps resource heads on the generic
  iterator protocol. All direct synchronous Array, String, and resource heads
  use `ForOfIteratorHeadIr`, which exhaustively separates
  ordinary assignment from a private, one-binding `SyncDisposable` capability
  that cannot carry an async plan. The intended per-iteration lifecycle creates
  a fresh immutable binding, disposes before the next iterator step, keeps a
  local continue inside the loop without closing, and disposes break, return,
  throw, disposer failure or acquisition failure before IteratorClose. The
  durable CLI oracle uses only custom iterator objects and covers those orderings,
  head TDZ and captured-binding freshness. The exact current-pin failure cohort
  is three unflagged files and six sloppy/strict executions:
  `head-using-bound-names-fordecl-tdz.js`,
  `head-using-fresh-binding-per-iteration.js`, and
  `using-invalid-assignment-statement-body-for-of.js`. At pre-batch commit
  `681ca415ba1e74c220fa8a5982cba1e7adedc151`, focused inspection rejected all
  six at the Wasm-AOT `for-of initializer` boundary. The integrated current-SHA
  checkpoint is green: `cargo xc`, 3/3 focused IR tests, 5/5 bounded structure
  tests and the end-to-end CLI lifecycle oracle pass; the three files now report
  6/6 Wasm-AOT executions with every failure bucket at zero. This is focused
  evidence, not a claim about the complete `language/statements/using`
  directory or the full pinned aggregate. Resource heads are
  BindingIdentifier-only; pattern-looking source such as `using[resource]` is
  ordinary element-access assignment grammar, not a resource binding pattern.
  `await using`, `for-await-of`,
  resumable owners, modules, `for-in` and dynamic source remain outside this
  batch.

- Promise construction now runs executors synchronously through the real
  Wasm-AOT call path, creates branded pending promise records, supplies distinct
  resolving functions, preserves first-settlement-wins behavior, and converts
  executor throws into rejection. Created Realms publish a fresh Promise
  constructor and prototype, all three implemented prototype methods, all ten
  implemented static methods, `@@species` and `@@toStringTag` from the same
  closed catalogs as the main Realm, with exact descriptors and Realm-local
  function identities. Promise allocation now consumes an opaque context
  coupling the selected prototype with the executing Realm; constructor
  fallback requires the typed Realm `%Promise.prototype%` slot, and executor
  resolving functions inherit that Realm's Function and error prototypes. A
  focused non-blocking fixture passes `1/1`, proving created
  constructor/`Promise.resolve` result prototypes and the constructor TypeError
  Realm without draining jobs.
  `Atomics.waitAsync` now consumes that opaque intrinsic context through a
  private non-copyable result context tied to the executing Atomics function
  Realm. Its synchronous wrappers and async Promise use the created Realm's
  required Object/Promise prototypes; enumerable writable configurable
  `async` then `value` properties preserve CreateDataProperty order. A distinct
  immediate-notify fixture passes `1/1`, covering not-equal, timeout-zero and
  async resolution without blocking; its bounded result contract passes `4/4`.
  The consolidated semantic golden passes `2/2` in 733.38 seconds and contains
  660 fixture dumps. Relative to the preceding 658-dump checkpoint it adds only
  `wasm_promise_created_realm.js` and
  `wasm_atomics_wait_async_created_realm.js`, removes none, and preserves every
  retained dump after normalizing emitted-function byte accounting; roots,
  builtin/helper counts, locals, imports, exports, globals, memories, data
  segments and name counts are unchanged.
  All fourteen escaping Promise algorithm closures now pass through one typed
  materializer that installs their defining Realm, that Realm's
  `%Function.prototype%`, TypeError/RangeError snapshots, GC-visible algorithm
  capture and self environment together. Capability-executor repeat calls and
  Promise self-resolution construct TypeError from the same owned Realm. The
  bounded source target passes `6/6`, the retained publication target passes
  `5/5`, and a finite created-Realm callback fixture passes `1/1` while checking
  resolving, capability, `finally`, keyed and standard combinator functions.
  Callback-created `Promise.allSettled` and `Promise.allSettledKeyed` records
  now derive `%Object.prototype%` from those self-backed functions' defining
  Realm through a private non-copyable allocation context. `Promise.any`
  likewise propagates the executing combinator's AggregateError prototype
  snapshot into its reject-element function and consumes an opaque allocation
  context in both the nonempty and empty rejection branches. A separate
  non-blocking four-branch fixture passes `1/1`, covering both settlement
  directions for standard and keyed results, exact record descriptors/key
  order, both `Promise.any` paths and borrowed created-Realm prototypes; its
  bounded allocation contract passes `6/6`. Standard `Promise.all`,
  `Promise.allSettled` and both `Promise.any` terminal paths now allocate their
  outer arrays from the executing method's defining-Realm `%Array%` catalog,
  independently of constructor `C`, through the existing opaque one-shot
  allocation proof. Entry methods use an explicit zero-environment catalog
  path, while self-backed created-Realm methods trap if their defining Realm or
  intrinsic catalog is absent. The expanded allocation structure target passes
  `7/7`, and the finite cross-Realm CLI fixture passes `1/1`. General
  AggregateError construction remains active work. The following 665-dump
  semantic golden passes `2/2` in 707.16 seconds, adds only the RegExp
  result-mode fixture and removes none. After normalizing emitted-code and
  local-accounting fields, 663 of 664 retained dumps are identical; only the
  deliberately expanded Promise allocation witness changes structurally,
  gaining two internal/named functions and four main-function locals.
  Async invocation now derives an opaque execution-Realm context from the
  callee, retains it in a traced ordinary activation slot, and reuses the
  async-generator activation's existing function edge. The returned
  async-function Promise, all three direct rejected-Promise control-flow
  wrappers and the five captured reaction kinds use that durable authority;
  default reactions retain their handler-or-null job policy. A finite
  created-Realm job fixture covers ordinary async and async-generator resumes.
  PromiseResolve constructor catalogs and other async builtins remain explicit
  follow-on work.
  The consolidated semantic golden passes `2/2` in 677.52 seconds and contains
  663 fixture dumps. Relative to the preceding 660-dump checkpoint it adds only
  the async-execution, callback-created-allocation and internal-callback Realm
  witnesses, removes none, and preserves every retained structural summary
  after normalizing the four expected code-size/local-accounting fields.
  Async-generator `next`/`return`/`throw` now load the canonical `%Promise%`
  constructor from the executing method's defining-Realm catalog through one
  opaque non-copyable proof. Entry publication self-backs those three method
  identities, capability allocation remains before receiver validation, and
  neither the entry Promise global nor the active job Realm is a fallback. The
  bounded contract passes `4/4` and the strengthened finite Realm fixture
  passes `1/1`. A subsequent 664-dump semantic golden passes `2/2` in 707.34
  seconds, adds only the Temporal date-field-mode fixture and removes none. Of
  663 retained dumps, 662 preserve every non-accounting summary; the
  intentionally strengthened async Realm witness gains five internal/named
  functions for its valid and invalid request paths.
  `Promise.prototype.then` queues FIFO reaction
  jobs for pending, fulfilled, and rejected sources, while static
  `Promise.resolve`/`Promise.reject` use generic constructor capabilities rather
  than directly mutating promise records. The complete current-pin
  `built-ins/Promise/reject` leaf reports `15/15` with every failure bucket at
  zero (manifest `17161705280949401201`). The complete
  `built-ins/Promise/resolve` leaf reports `30/30` after moving receiver
  object-validation ahead of the same-constructor identity shortcut (manifest
  `6454436055780916821`). `Promise.withResolvers` now exposes the generic
  constructor capability as an ordinary ordered
  `{ promise, resolve, reject }` record; its complete pinned leaf reports `6/6`
  with every failure bucket at zero (manifest `8421357701156147894`). Its outer
  record now takes `%Object.prototype%` from the executing method's defining
  Realm independently of constructor `C`, through a private one-shot allocation
  proof with strict nonentry catalog traps. The bounded Realm contract passes
  `5/5`, the retained publication contract passes `5/5`, and the finite
  two-direction created-Realm fixture passes `1/1`. The following 666-dump
  semantic golden passes `2/2` in 704.11 seconds, adds only the array
  named-key-selection fixture, removes none and preserves all 665 retained
  non-accounting summaries.
  `Promise.try` now invokes the callback with `undefined` and trailing
  arguments, resolves normal results, rejects abrupt completion, and preserves
  generic constructor capabilities; its complete pinned leaf reports `12/12`
  with every failure bucket at zero (manifest `15089719507409975374`). A
  non-callable callback now rejects with TypeError from the executing method's
  defining Realm through a private one-shot prototype proof, rather than the
  entry Realm; capability creation, argument-vector formation and the existing
  reject path retain their order. The bounded contract passes `5/5`, the
  retained publication contract passes `5/5`, and the FIFO created-Realm
  callback fixture passes `1/1`. The following 667-dump semantic golden passes
  `2/2` in 702.89 seconds, adds only the iterator receiver-policy fixture and
  removes none. After accounting normalization, 665 of 666 retained dumps are
  identical; only the expanded Promise callback witness changes structurally,
  gaining one internal/named function and two main-function locals.
  Borrowed `Promise.prototype.then` and `Promise.prototype.finally` now derive
  SpeciesConstructor's default `%Promise%` and both validation TypeErrors from
  one private, must-use context tied to the executing method's defining-Realm
  catalog. The entry route is explicit, and missing self-backed Realm catalog
  state traps without receiver, constructor or active-job fallback. The bounded
  contract and finite borrowed-method witness cover default construction plus
  primitive-constructor and invalid-species TypeError identity.
  Their direct incompatible-receiver branches also consume a separate one-shot
  TypeError-prototype proof from the executing method's self-backed snapshot.
  The finite witness covers borrowed `then` and `finally` receiver errors;
  shared ToObject and Call error ownership remains open.
  The six Promise combinator static methods now pair algorithmic TypeError and
  RangeError prototypes in one private non-copyable context owned by the
  executing method's Realm. The three lowering families acquire it only after
  the observable `C.resolve` lookup, borrow it at exactly fifteen live failure
  sites and consume it once. The returned Promise remains independently owned
  by constructor `C`. The bounded structure target passes `5/5`, and the finite
  created-Realm witness passes `1/1`.
  The direct `built-ins/Promise` matrix node reports all `57/57`
  AOT-applicable roots green at the current pin; its 58th root,
  `proto-from-ctor-realm.js`, invokes the cross-realm Function constructor and
  remains an explicit dynamic-source exclusion (manifest
  `2879933483929296098`). Refresh this non-recursive residual node with
  `./target/release/lila --jobs 1 test262 run --matrix-node built-ins/Promise --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 1 --timeout-ms 60000 --snapshot-name promise-direct-root-baseline-58-20260721`.
  The general `--matrix-node` selector avoids recursively rerunning all Promise
  subdirectories when measuring one residual matrix leaf.
  Mechanically deduplicating authoritative current-pin Wasm-AOT snapshots, the
  complete `built-ins/Promise` subtree has all `651/651` AOT-applicable roots
  exact-green. The 652nd root is the cross-realm Function-constructor dynamic
  source exclusion above; no AOT-applicable root is skipped or inferred green.
  `Promise.allSettled` shares the generic iterable and constructor-capability
  path with `Promise.all`, but uses paired resolve/reject element functions
  with one already-called guard and ordered `{ status, value }` or
  `{ status, reason }` records. Its complete pinned directory reports
  `104/104` under Wasm-AOT on `2026-07-21`, with every failure bucket and
  timeout count at zero (manifest `4524389048728247828`). Refresh it with
  `./target/release/lila --jobs 1 test262 run built-ins/Promise/allSettled --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 4 --timeout-ms 60000 --snapshot-name allsettled-wasm-aot-release-20260721`.
  `Promise.any` uses the same generic constructor capability, one-time
  `resolve` lookup, and iterator-close behavior. It resolves on the first
  fulfillment; paired per-element rejection state preserves input order and
  rejects the empty or all-rejected case with an `AggregateError`. Its complete
  pinned directory reports `94/94` under Wasm-AOT on `2026-07-21`, with every
  failure bucket and timeout count at zero (manifest `7726540635021801166`).
  Refresh it with
  `./target/release/lila --jobs 1 test262 run built-ins/Promise/any --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 4 --timeout-ms 60000 --snapshot-name promise-any-wasm-aot-release-20260721`.
  The Stage 3 `Promise.allKeyed` and `Promise.allSettledKeyed` builtins collect
  proxy-aware own enumerable string and symbol keys into ordered null-prototype
  result objects, while retaining generic constructor capabilities, one-time
  `resolve` lookup, abrupt rejection, and per-element already-called guards.
  Their complete current-pin leaves each report `6/6` with every failure bucket
  and timeout count at zero (manifests `14832762644447495093` and
  `7332652697133906527`). Refresh them by replacing `<method>` with
  `allKeyed` or `allSettledKeyed` in
  `./target/release/lila --jobs 1 test262 run built-ins/Promise/<method> --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 1 --timeout-ms 60000 --snapshot-name promise-<method>-wasm-aot-release-fixed-20260721`.
  `Promise.all` now consumes generic
  iterables through the receiver constructor's capability and `resolve`
  function, preserves input order and a shared reject function, guards each
  resolve element against repeated calls, closes iterators on the required
  abrupt paths, and settles through the capability's observable functions.
  The complete pinned `built-ins/Promise/all` directory reports `98/98` under
  Wasm-AOT on `2026-07-21` at
  Test262 revision `aa55200d1310384c5cf69ea95b2a2ecba457007b`: `length.js`,
  `name.js`, `prop-desc.js`, `iter-arg-is-string-resolve.js`,
  `invoke-resolve-get-once-multiple-calls.js`, `invoke-resolve-error-close.js`,
  `invoke-then-error-close.js`, `iter-step-err-no-close.js`,
  `resolve-thenable.js`, and `capability-resolve-throws-no-close.js` under
  `built-ins/Promise/all/`. Twelve additional roots cover derived-constructor
  context, shared rejection identity, resolve-element repeated calls and input
  ordering, early resolution before loop exit, repeated thenables,
  resolve-element function shape and non-constructability, and capability
  resolve throws before rejection. Twenty constructor/capability and observable
  resolve/then roots add invalid receiver and constructor cases, executor
  misuse, per-iteration resolve calls, resolve-get/return/throw behavior, and
  abrupt `then` access/invocation. Thirty-nine further unique roots cover
  iterator result validation and close/no-close precedence, primitive iterable
  rejection, deferred/immediate rejection, ignored late settlement,
  resolve-element function descriptors, poisoned/noncallable/non-thenable
  resolution, Array-setter isolation, legacy numbered cases, and species-get
  failures. All 98 roots are AOT-applicable, the directory has no dynamic-source
  exclusions, and every root is present in the full checkpoint (manifest
  `2207607493869671962`). Refresh it with
  `./target/release/lila --jobs 1 test262 run built-ins/Promise/all --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 4 --timeout-ms 60000 --snapshot-name promise-all-complete-98`.
  `Promise.race` now uses the same generic constructor capability, one-time
  `resolve` lookup, iterable and IteratorClose machinery, and directly chains
  every resolved value to the shared resolve/reject functions. Empty iterables
  remain pending and first settlement wins. The complete pinned
  `built-ins/Promise/race` directory reports `94/94` under Wasm-AOT on the same
  date and Test262 pin, covering installation,
  receiver/constructor validation, capability errors, observable resolve/then
  calls, String iteration, immediate rejection, shared settlement functions,
  species errors, iterator result validation, abrupt step/value completion,
  iterator close/no-close precedence, malformed primitive iterables,
  per-iteration resolve observability, thenables, and settlement ordering. The
  directory contains no dynamic-source exclusions and every failure bucket and
  timeout count is zero (manifest `2843367700383518511`). Refresh it with
  `./target/release/lila --jobs 1 test262 run built-ins/Promise/race --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 4 --timeout-ms 60000 --snapshot-name promise-race-complete-94-20260721`.
  `Promise.prototype.finally` now performs generic receiver `then` invocation,
  species construction, callable and noncallable forwarding, cleanup
  assimilation, and original value/reason preservation or replacement. Its
  complete pinned directory reports `29/29` under Wasm-AOT with every failure
  bucket at zero (manifest `6304521883779310500`). Refresh it with
  `./target/release/lila --jobs 1 test262 run built-ins/Promise/prototype/finally --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 4 --timeout-ms 60000 --snapshot-name promise-finally-general`.
  Resolution now
  assimilates callable function and Proxy thenables asynchronously, rejects abrupt `then` access and
  self-resolution, and preserves first-settlement-wins behavior across thenable
  jobs; fresh pinned thenable and self-resolution clusters report `2/2` and
  `1/1`. `Promise.prototype.catch` performs the generic observable `then`
  invocation required by the spec; its complete current-pin leaf reports
  `14/14` with every failure bucket at zero (manifest
  `3320451833449117852`). The six direct `Promise.prototype` roots also report
  `6/6`, including the standard getter-only `Symbol.toStringTag` descriptor.
  The complete current-pin `Promise.prototype.then` leaf reports `75/75` with
  receiver/brand, species/capability, reaction shape, thenable assimilation,
  queue ordering, and realm cases all green (manifest
  `9291152077727223222`).
  Species construction performs the observable constructor and
  `Symbol.species` lookups and chains through generic promise capabilities;
  the complete current-pin `built-ins/Promise/Symbol.species` leaf reports
  `5/5` with every failure bucket at zero (manifest
  `10564167133939605890`). Older constructor and capability counts are not
  current-pin Wasm evidence and are being remeasured. Async function declarations now return intrinsic promises
  immediately and linear function bodies resume through the Promise job queue
  after plain `await` statements, identifier assignments, and `return await`;
  rejection resumes as a throw and lexical bindings survive suspension.
  Array-specialized `for...of` loops with one body `await` now preserve a
  captured lexical head in a fresh environment for every iteration. The
  current-SHA consumer oracle retains six closures and calls them after the
  loop, while the two exact `Array.fromAsync/asyncitems-*-not-callable.js`
  witnesses report `4/4`; this is focused evidence, not a refreshed complete
  `Array.fromAsync` publication.
  Named and anonymous async function expressions use the same real activation
  path and are non-constructable; the pinned `expression-returns-promise.js`,
  `name.js`, and `syntax-expression-is-PrimaryExpression.js` roots report
  `3/3`. Async `try`/`catch`/`finally` preserves pending completions across
  rejected and returned awaits, including finalizer overrides; the nine pinned
  `try-{reject,return,throw}-finally-{reject,return,throw}.js` roots report
  `9/9`. Ordinary async object methods preserve their receiver across `await`,
  remain non-constructable, and apply ordered parameter TDZ semantics; their
  initial pinned runtime checkpoint reports `8/8`. Instance and static async
  class methods share that execution path; a declaration/expression checkpoint
  reports `9/9`. Async arrows preserve lexical `this`/`arguments`/`new.target`,
  parameter TDZ, and nested expression-body awaits; an initial AOT cohort
  reports `10/10`, while adjacent actual `eval` and `new Function` roots remain
  explicit dynamic-code exclusions. A first real `for await...of` /
  AsyncFromSyncIterator checkpoint reports `9/9`, covering array values,
  per-iteration bindings, nested IteratorClose, abrupt continuation, and
  PromiseResolve timing. Awaited arrays now perform observable
  `Symbol.asyncIterator` / `Symbol.iterator` acquisition instead of taking the
  direct array-index fast path. Two focused engine regressions cover an
  `Array.prototype[Symbol.iterator]` override with exact receiver, cached
  `next`, awaited values, and close behavior, plus cached
  `%ArrayIteratorPrototype%.next` lookup. The adjacent exact pinned cohort
  reports `10/10` on `2026-07-20`, covering Promise timing and nested
  synchronous-iterator close and abrupt-completion paths. Upstream has no
  direct outer-array prototype-mutation or `next`-cache root, so those remain
  focused engine evidence. A separate custom synchronous-iterator and
  destructuring checkpoint reports `20/20`, covering cached `next`, awaited
  values, `return`/close precedence, rejected and non-object iterator results,
  and destructuring close/no-close behavior. Native async-iterator acquisition
  now takes precedence over the synchronous fallback, awaits `next` and
  `return` results, and applies AsyncIteratorClose GetMethod precedence; its
  first exact close checkpoint reports `5/5`, and two constructor-lookup/job
  ordering roots report `2/2`. Awaited String primitives now perform observable
  `Symbol.asyncIterator` then `Symbol.iterator` lookup before using the same
  AsyncFromSync state machine. Their focused checkpoint reports `3/3`, covering
  Unicode code points and lone surrogates, awaited String-iterator close values,
  and async-iterator preference with strict primitive receiver identity. The
  adjacent pinned String-iterator, AsyncFromSync timing, and rejection-close
  roots report `4/4` on `2026-07-19`.
  The native async-iterator path preserves yielded values, including Promise
  objects, for result creation and mapper calls while still awaiting mapper
  results. The complete pinned `Array.fromAsync` leaf reports `95/95` on
  `2026-07-27` under
  `./target/release/lila test262 run built-ins/Array/fromAsync --suite-root test262/vendor/test262 --execution-backend wasm-aot --timeout-ms 60000 --threads 1`;
  its returned and await throwaway capabilities, fulfilled/rejected callback
  pair, callback Function prototype, defining Realm and TypeError prototype now
  come from one private non-copyable executing-method context. Continuation
  state uses the GC-visible builtin-closure slot rather than masquerading as a
  function environment. The two bounded Realm targets pass `10/10`, and the
  dedicated array-like/iterable fulfillment/rejection witness passes `1/1`.
  broader async iteration remains active conformance work, and this is not a
  claim of complete Promise or async-function support. The pinned
  six-file declaration
  and body baseline reports `6/6` on `2026-07-19`; refresh it by running the
  exact `declaration-returns-promise.js`, `evaluation-body.js`,
  `evaluation-body-that-returns.js`,
  `evaluation-body-that-returns-after-await.js`,
  `evaluation-body-that-throws.js`, and
  `evaluation-body-that-throws-after-await.js` paths under
  `language/statements/async-function/` with
  `./target/debug/lila test262 run <path> --suite-root test262/vendor/test262 --execution-backend wasm --timeout-ms 60000 --threads 1`.
- `%AsyncIteratorPrototype%[Symbol.asyncDispose]` is a distinct
  non-constructible Rust/AOT builtin. It creates a defining-realm intrinsic
  Promise before reading `return`, converts getter and call throws into
  rejections, passes one explicit `undefined` argument, awaits the returned
  value, and resolves to `undefined` or preserves the rejection reason. The
  pinned `built-ins/AsyncIteratorPrototype/Symbol.asyncDispose` directory
  reports `9/9` on `2026-07-28` at Test262 pin
  `aa55200d1310384c5cf69ea95b2a2ecba457007b`; refresh it with
  `./target/debug/lila --jobs 1 test262 run built-ins/AsyncIteratorPrototype/Symbol.asyncDispose --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 1 --timeout-ms 120000`.
- Async-generator declarations and expressions now create suspended-start
  generator objects whose terminal `next`, `throw`, and `return` requests settle
  intrinsic promises with the required iterator-result or rejection outcome,
  and the bounded linear no-Yield body path resumes from Await fulfillment or
  rejection without replaying its prefix. Invalid receivers for all three
  `%AsyncGeneratorPrototype%` request methods create an intrinsic Promise
  capability, reject it with the method-defining realm's `TypeError`, and
  return normally without entering the valid-generator queue. The pinned
  `built-ins/AsyncGeneratorPrototype` directory reports `48/48` on
  `2026-07-28` at Test262 pin
  `aa55200d1310384c5cf69ea95b2a2ecba457007b`; refresh it with
  `./target/debug/lila --jobs 1 test262 run built-ins/AsyncGeneratorPrototype --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 1 --timeout-ms 120000`.
  The scoped exact checkpoint reports
  `13/13` on `2026-07-20` at Test262 pin
  `aa55200d1310384c5cf69ea95b2a2ecba457007b`. Only two of those roots exercise
  body-Await or implicit async-return Await semantics:
  `language/statements/async-generator/return-undefined-implicit-and-explicit.js`
  and
  `built-ins/AsyncGeneratorPrototype/return/return-state-completed-broken-promise.js`.
  The other eleven cover immediate intrinsic-Promise results, iterator-result
  shape, suspended-start boundaries, AwaitReturn assimilation, and completed
  queue settlement:
  `built-ins/AsyncGeneratorPrototype/next/return-promise.js`,
  `built-ins/AsyncGeneratorPrototype/return/return-promise.js`,
  `built-ins/AsyncGeneratorPrototype/next/iterator-result-prototype.js`,
  `built-ins/AsyncGeneratorPrototype/return/iterator-result-prototype.js`,
  `built-ins/AsyncGeneratorPrototype/return/return-suspendedStart.js`,
  `built-ins/AsyncGeneratorPrototype/throw/throw-suspendedStart.js`,
  `built-ins/AsyncGeneratorPrototype/throw/throw-suspendedStart-promise.js`,
  `built-ins/AsyncGeneratorPrototype/return/return-suspendedStart-promise.js`,
  `built-ins/AsyncGeneratorPrototype/return/return-suspendedStart-broken-promise.js`,
  `built-ins/AsyncGeneratorPrototype/return/return-state-completed.js`, and
  `built-ins/AsyncGeneratorPrototype/throw/throw-state-completed.js`.
  Refresh each listed path with
  `./target/release/lila --jobs 1 test262 run <exact-path> --suite-root test262/vendor/test262 --execution-backend wasm-aot --timeout-ms 60000 --threads 1 --snapshot-name asyncgen-awaitonly-<case>-20260720`.
  A separate non-overlapping intrinsic checkpoint reports `20/20` at the same
  pin: the `Symbol.toStringTag` and `constructor` roots directly under
  `built-ins/AsyncGeneratorPrototype`; the `length.js`, `name.js`, and
  `prop-desc.js` roots under each of its `next`, `return`, and `throw`
  directories; and
  `built-ins/AsyncGeneratorFunction/{extensibility,length,name}.js` plus its
  `prototype/{Symbol.toStringTag,constructor,extensibility,not-callable,prop-desc,prototype}.js`
  roots. Refresh each of those exact paths with the same command and an
  `asyncgen-meta-<case>-20260720` snapshot name. A third non-overlapping exact
  checkpoint reports `7/7` on `2026-07-20` for
  `language/expressions/async-generator/expression-yield-as-statement.js`,
  `expression-yield-as-operand.js`, and `expression-yield-newline.js`, plus
  `built-ins/AsyncGeneratorPrototype/return/return-suspendedYield.js`,
  `return-suspendedYield-promise.js`,
  `built-ins/AsyncGeneratorPrototype/throw/throw-suspendedYield.js`, and
  `throw-suspendedYield-promise.js`. These exercise suspended-Yield request
  settlement and the `next`, `return`, and `throw` resume boundaries. Refresh
  each exact path with the same command and an
  `asyncgen-yield-exact-<case>-20260720` snapshot name. A fourth checkpoint
  reports `4/4` on `2026-07-21` for
  `built-ins/AsyncGeneratorPrototype/next/request-queue-order.js`,
  `next/request-queue-order-state-executing.js`,
  `built-ins/AsyncGeneratorPrototype/return/request-queue-order-state-executing.js`,
  and
  `built-ins/AsyncGeneratorPrototype/throw/request-queue-order-state-executing.js`.
  These cover Yield-driven FIFO settlement and queued `next`, `return`, and
  `throw` requests while the generator is executing. Refresh each exact path
  with the same command and an `asyncgen-promise-all-final-<case>-20260721`
  snapshot name. The four checkpoints contain 44 unique exact roots.
  A separate async-generator expression grammar/metadata checkpoint adds 41
  non-overlapping exact roots: function/prototype metadata, all twelve reserved
  `await` grammar roots, twenty-one early-error roots, and direct forbidden
  `arguments`/`caller` properties. Binding an async-generator expression to the
  name `eval` is no longer misclassified as dynamic evaluation, while actual
  `eval(...)` remains excluded; parameter initialization also precedes the
  per-call iterator prototype lookup. Ordinary nested `if` statements now run
  in async-generator bodies when their branches contain no suspension, while
  branch-contained Await/Yield remains explicit. Exact `yield await value`
  staging and non-empty async `yield*` delegation now preserve awaited iterator
  results and resume through `next`, `throw`, and `return`, using
  `Symbol.asyncIterator` before the synchronous fallback. Rejected yielded
  promises now route through direct requests, transparent `for await` over
  async or synchronous iterators, and async/sync delegation while preserving
  close and original-completion priority. Array and object spread operands also
  retain iterable normalization and source order across Yield suspension.
  Yielded thenables use intrinsic PromiseResolve and a shared already-resolved
  guard, preserving the first resolve/reject call even when `then` calls both
  and then throws. Core `yield*` delegation now covers named and unnamed
  async-generator expressions over both async and synchronous delegates for
  `next`, `return`, and `throw`, including thenable result ordering. The full
  async-iterator acquisition/error matrix also validates GetMethod, call, and
  result failures. Synchronous fallback acquisition also preserves null
  `Symbol.asyncIterator`, getter/call abrupt completions, and iterator-result
  validation. Delegated `next` is captured once, checked for callability, and
  called with the forwarded value; its awaited result is object-validated
  before observable `done` and `value` reads. The 24 named and unnamed
  non-thenable `next` call/get/non-object/non-callable roots are exact-green;
  another 18 roots cover abrupt `then` access/invocation and all seven
  non-callable `then` values for both named and unnamed expressions. A missing
  delegate `throw` now awaits and validates `return` cleanup before producing
  the required TypeError. The
  `language/expressions/async-generator` directory contains 623 roots: 618 are
  AOT-applicable and five invoke actual `eval`. The most recent full-directory
  run reports `186/618`. Independently, the bounded exact checkpoints described
  above account for 227 unique roots.
  A separate non-overlapping checkpoint on `2026-07-27` adds all 32 roots under
  `language/expressions/async-generator/dstr/ary-ptrn-elem`. Combined bounded
  evidence therefore accounts for 259 unique exact-green roots, leaving at
  most 359 AOT-applicable roots without exact-green evidence. Refresh the
  prefix with
  `./target/release/lila --jobs 1 test262 run language/expressions/async-generator/dstr/ary-ptrn-elem --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 1 --timeout-ms 60000`.
  This bounded checkpoint does not recompute the broader aggregate.
  Queued `.return(value)` requests at an ordinary or delegated suspended Yield
  now unwrap through their own Promise continuation without double-awaiting the
  final completion; the four statement-form async/sync delegation,
  missing-value, and then-getter tick-order roots are exact-green. The 18
  `yield-star-{async-return,sync-return}.js` method-form mirrors are also
  exact-green on `2026-07-22` at the same Test262 pin: object methods plus
  public/private instance/static methods in class declarations and expressions.
  Refresh each exact path with
  `./target/release/lila --jobs 1 test262 run <exact-path> --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 1 --timeout-ms 60000 --snapshot-name asyncgen-method-mirror-<case>-20260722`.
  A bounded 24-root scheduling and delegation checkpoint is exact-green on
  `2026-07-22` at Test262 pin
  `aa55200d1310384c5cf69ea95b2a2ecba457007b`. It covers request-queue ordering,
  rejected direct Yield, transparent `for await` over async and synchronous
  iterators, async/sync `yield*`, return and then-getter tick ordering, and
  abrupt or non-callable delegated `then`. Synchronous iterator fallback now
  awaits each iterator-result value without unwrapping values produced by a
  real async iterator. Every failure bucket is zero in the exact snapshots
  named `asyncgen-scheduling-final24-20260722-*.json`. Refresh each listed exact
  root with
  `./target/release/lila --jobs 1 test262 run <exact-path> --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 1 --timeout-ms 60000 --snapshot-name asyncgen-scheduling-final24-20260722`.
  The adjacent 30 statement-form `yield*` acquisition roots are exact-green on
  `2026-07-22` at the same pin. They cover abrupt operand evaluation, the
  inaccessible AsyncFromSync wrapper boundary, async-iterator preference,
  nullish async-method fallback, abrupt getter and method calls, non-callable
  iterator methods, and primitive iterator results. Every failure bucket is
  zero in `asyncgen-statement-acquisition-final30-20260722-*.json`. Refresh each
  exact root with the same command and
  `--snapshot-name asyncgen-statement-acquisition-final30-20260722`.
  A further non-overlapping 24-root statement/class-declaration checkpoint is
  exact-green on `2026-07-22` at the same pin. It covers delegated `next`
  getter and call failures, awaited non-object results, abrupt `done` and
  `value` access, and all seven non-callable `next` values for async-generator
  declarations and class instance methods. Every failure bucket is zero in
  `asyncgen-statement-next-validation-final24-20260722-*.json`. Refresh each
  exact root with the same command and
  `--snapshot-name asyncgen-statement-next-validation-final24-20260722`.
  The matching 24 class static and private-instance method roots are also
  exact-green on `2026-07-22` at the same pin. They cover delegated `next`
  getter and call failures, awaited non-object results, abrupt `done` and
  `value` access, and all seven non-callable `next` values. Every failure
  bucket is zero in
  `asyncgen-class-static-private-next-final24-20260722-*.json`. Refresh each
  exact root with the same command and
  `--snapshot-name asyncgen-class-static-private-next-final24-20260722`.
  An adjacent 30-root class static/private-instance continuation checkpoint is
  exact-green on `2026-07-22` at the same pin. It covers rejected Yield before
  async/sync delegation, delegated async/sync `next` and `throw` ordering, and
  `next` result assimilation with abrupt `then` access/invocation plus all
  seven non-callable `then` values. Every failure bucket is zero in
  `asyncgen-class-static-private-continuation-final30-20260722-*.json`. Refresh
  each exact root with the same command and
  `--snapshot-name asyncgen-class-static-private-continuation-final30-20260722`.
  Async-generator try/catch/finally now shares the body's preplanned suspension
  states and preserves pending abrupt completions across finalizer Yield. The
  three `yield-star-{normal,return,throw}-notdone-iter-value-throws.js` statement
  roots are exact-green on `2026-07-22` at Test262
  `aa55200d1310384c5cf69ea95b2a2ecba457007b`, with every failure bucket at zero
  (manifests `8156554394922646296`, `17360652823301751254`, and
  `14599242529674358692`). Refresh each exact path with
  `./target/release/lila --jobs 1 test262 run language/statements/async-generator/yield-star-<normal|return|throw>-notdone-iter-value-throws.js --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 1 --timeout-ms 60000 --snapshot-name asyncgen-try-notdone-<case>-20260722`.
  Arbitrary mixed Await/Yield sequences now share that collision-free state
  plan. A bounded 20-root suspended try/catch/finally checkpoint is exact-green
  on `2026-07-22` at the same pin, covering `.return()`/`.throw()` overrides,
  rejected Yield caught in the body, and statement/named/unnamed expression
  mirrors of async delegation through `next`, `return`, and `throw`. Every
  failure bucket is zero; the manifests, in checkpoint order, are
  `2254365628558528862`, `15627516562269089907`, `6464620846581743844`,
  `3027586255731303108`, `11751139455940417830`, `5373195989092074489`,
  `12019605220262227606`, `18424959393445981383`, `17496395210074582520`,
  `8436591244830497459`, `3242868264558116722`, `18424960346985568620`,
  `12410543517481961054`, `2765747284222495604`, `16701367308381327356`,
  `6075337559788816786`, `15049589881886505538`, `8173557398602242706`,
  `11207893843153698446`, and `679067303864347086`. Refresh an exact root with
  `./target/release/lila --jobs 1 test262 run <exact-path> --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 1 --timeout-ms 60000 --snapshot-name asyncgen-try-cohort-<case>-20260722`.
  Direct `yield` statements in runtime-selected `if`/`else` branches now use a
  dedicated merge state, so resumption does not re-evaluate the condition or
  collide with later suspension points. Classic async-generator `for` loops
  with one direct body `await` reuse that suspension edge across iterations,
  retain their lexical counter in the activation, and resume through
  update/test without rerunning initialization. The adjacent direct-`yield`
  path now gives both a lexical loop initializer and direct body lexical
  declarations activation-owned storage, so later iterations retain their
  counter and a body local remains readable after `yield`. The exact existing
  CLI oracle covers zero, one and three iterations, an abrupt update,
  per-iteration TDZ and a post-yield lexical read; its pre-batch result was
  `0/1`, and the three direct `Array.fromAsync` witnesses were `0/6` at Test262
  content tree `aa55200d1310384c5cf69ea95b2a2ecba457007b`. On 2026-08-25,
  the post-implementation CLI rerun passed `1/1`, those exact Test262 files
  passed `6/6`, and the adjacent direct-`await` control passed `2/2`, with every
  non-success bucket at zero. These focused results do not alter the published
  aggregate status counts. Loop-head suspension,
  break/continue, captured per-iteration environments, multiple or nested body
  suspensions, `while`, `do`, `for-of`, `for-await-of`, general
  async-generator continuation and GC layout remain explicit unsupported or
  unclaimed paths. Conditional `await` and broader suspended-body control flow
  also remain unsupported.
  Request scheduling remains active conformance work.
  This checkpoint is not a claim of full async-generator support.
- Wasmtime shared-memory exports now participate in result rendering,
  exception-name reads, and host printing. The real Wasm atomic RMW path clears
  the seven pinned `add`/`and`/`compareExchange`/`exchange`/`or`/`sub`/`xor`
  leaves at `107/107`; the RMW SharedArrayBuffer gate is backed by exact-source
  execution rather than metadata-only admission. The pinned `Atomics.load` and
  `Atomics.store` leaves report `14/14` and `16/16`; store applies
  `ToIntegerOrInfinity` before element conversion while returning the original
  numeric value. `Atomics.isLockFree` reports `7/7`, and 35 of 36 bounded
  no-waiter `Atomics.notify` cases pass. Real in-place growable
  SharedArrayBuffer support, whose pinned `grow` leaf reports `15/15`, clears
  the final bounded notify case for `36/36`. `Atomics.wait` now emits real
  Wasm `memory.atomic.wait32`/`wait64` operations, observes the host's `CanBlock`
  capability, and distinguishes `ok`, `not-equal`, and `timed-out`;
  `Atomics.notify` emits `memory.atomic.notify` after specification-ordered
  count coercion. `Atomics.waitAsync` waiters are registered in their host
  agent group, so `notify` can claim FIFO waiters owned by another Wasm module;
  each origin module polls the claim, settles its private Promise, and removes
  the waiter.
  The 16 direct algorithm-created Atomics TypeErrors across `pause`, `notify`,
  `waitAsync`, `wait`, its suspension check and the shared integer-operation
  compiler now route through the executing builtin's Realm. A closed six-region
  source guard pins the `1/3/1/4/4/3` census and a representative entry-Realm
  fixture passes `1/1`. Created realms now publish the complete implemented
  14-method Atomics namespace through the same closed publication order as the
  main realm, with catalog-derived names, exact descriptors, fresh function
  identities, self environments and defining-Realm TypeError/RangeError
  capture. Its structural target passes `3/3`; a non-blocking borrowed-`add`
  fixture passes `1/1` across all method identities/descriptors and both error
  Realms without invoking `wait` or `waitAsync`. Entry and created Atomics
  functions are now self-backed; borrowed `waitAsync` derives required Object
  and Promise intrinsics from its defining Realm through one private exhaustive
  result context and traps on missing Realm state. Its `async`/`value` result
  descriptors and key order are CreateDataProperty-exact. A separate
  immediate-notify fixture passes `1/1`, covering not-equal, timeout-zero and
  asynchronous created-Realm wrapper/Promise ownership plus resolved `"ok"`
  behavior; the bounded result contract passes `4/4`, and the retained
  entry-Realm waitAsync core regression passes its exact `1/1`.
  Unnotified finite waits settle as `timed-out` against a monotonic host
  deadline; notification claims and removes a waiter only while its deadline
  remains live. The Wasm-AOT Test262 host compiles `$262.agent.start` source as
  a separate Wasm module, gives every worker the same host-owned shared-memory
  arena, reconstructs broadcast SharedArrayBuffers, and implements reports,
  sleep, monotonic time, leaving, shutdown, and worker-error joins. Identical
  worker sources reuse their emitted artifact within the agent group. The
  Test262 materializer installs an include-scoped asynchronous host-sleep timer
  before `atomicsHelper.js`, avoiding that helper's allocation-heavy Promise
  polling fallback without changing unrelated cases. The canonical no-waiter,
  one- and two-waiter, renotify-noop, FIFO-order, and
  BigInt per-location `Atomics.notify` agent cases pass. Async-agent report
  collection through lexical array literals with direct `await` elements now
  lowers into ordered suspension states. Lexical and `var` declaration
  initializers also preserve ordered suspension through unconditional unary
  and arithmetic expression trees; branch-sensitive expressions, composite
  awaited array elements, and array spread across those suspension points
  remain explicitly unsupported.
  Integer TypedArray writes use ECMAScript
  modulo/clamping conversions, and integer-indexed
  `Object.defineProperty`/`Reflect.defineProperty` writes preserve descriptor,
  bounds, conversion-order, and abrupt-completion semantics.
- ArrayBuffer and SharedArrayBuffer backing state now lives in unforgeable
  brand-selected header slots rather than ordinary `$ArrayBuffer...`
  properties. TypedArray view metadata and DataView state use the same private
  headers; instances expose inherited standard accessors instead of `$...`
  mirrors or raw backing pointers. The real keyed detach operation rejects
  forged and shared buffers, supports repeat detachment, preserves resizable
  flags, and is shared by the host hook and transfer path. Fifteen original
  upstream ArrayBuffer and DataView detachment cases pass after removing their
  classifier catch-alls and fake detached-source rewrites. TypedArray-owned
  backing buffers now initialize the same private state; three original
  detached `toString`/`toLocaleString` paths improved from `0/3` to `3/3` after
  replacing source-output rewrites with a compact harness that still exercises
  every real constructor and assertion.
  TypedArray buffer-argument construction now recognizes only privately branded
  ArrayBuffer and SharedArrayBuffer instances, performs offset and length
  coercions in specification order, rechecks detach/resize state afterward,
  and distinguishes fixed from length-tracking views. Its focused mixed
  numeric/BigInt regression passes. Constructor emission now closes the source
  discriminator before one shared non-buffer allocation/copy path, so ordinary
  array-like sources cannot inherit a zero backing pointer. Final Wasm local
  declarations also use each function's observed temporary high-water mark.
  A bounded six-file original numeric checkpoint, including detach-during-offset
  and length conversion, aligned-offset failure, fixed length/offset, and two
  resizable-buffer bounds cases, reports `6/6` as of `2026-07-19`; the largest
  observed total system use was `26,502,610,944` bytes. A corresponding
  pinned defined-length source now also runs unchanged in sloppy and strict
  modes (`2/2`) with the complete vendored `testTypedArray.js`; its handwritten
  per-constructor expansion and helper omission are removed. The four
  `%TypedArray%[@@species]` result and metadata sources likewise run unchanged
  in both modes (`8/8`) with the complete ordinary typed-array and property
  helpers; their species-specific compact-helper authorization is removed. A
  fifteen-source ArrayBuffer accessor and `slice` metadata family also runs
  unchanged with complete assertion and property helpers in both modes
  (`30/30`); its two ArrayBuffer-specific compact-helper authorities are
  removed. A
  corresponding six-file BigInt buffer-argument checkpoint reports `6/6`; it also exposed and fixed
  TypedArray-source copying through ordinary property reads, which now uses the
  required integer-indexed access path. Its largest observed total system use
  was `30,777,458,688` bytes. The corresponding numeric source-copy checkpoint
  reports `3/3`, including all 36 integer source/destination pairs over a
  SharedArrayBuffer; its largest observed total system use was `32,985,120,768`
  bytes. Together with the adjacent defined/negative/excessive length and
  byte-offset, `ToIndex`, abrupt-coercion, prototype, detachment, and identity
  checkpoints, the complete AOT-applicable buffer-argument cohort reports
  `102/102` as of `2026-07-19`. Four additional cross-realm prototype files each
  use `new other.Function()` and remain explicit dynamic-source exclusions. The
  separate TypedArray-source constructor cohort reports `23/23` AOT-applicable
  cases, covering same- and cross-kind copies, numeric/BigInt mismatch errors,
  prototype selection, empty-source extensibility, and current resizable-buffer
  bounds. Two adjacent cross-realm prototype files use dynamic Function
  constructors and remain explicit exclusions. The adjacent object-argument
  constructor cohort has 57 AOT-applicable roots plus two equivalent dynamic
  cross-realm exclusions. The complete AOT-applicable cohort reports `57/57`
  exact-green as of `2026-07-20`. The adjacent combined numeric and BigInt
  length-argument cohort has 22 AOT-applicable roots plus two cross-realm
  `new other.Function()` exclusions. The complete AOT-applicable cohort reports
  `22/22` exact-green as of `2026-07-20`, covering zero initialization, length
  coercion errors, ToIndex, newTarget/prototype selection, result identity, and
  extensibility in both constructor families. Newly owned TypedArray backing
  stores are explicitly zeroed, and failed or
  memory32-unrepresentable backing allocations surface as catchable
  RangeErrors instead of Wasm traps. The adjacent combined numeric and BigInt
  no-argument cohort has 12 AOT-applicable roots plus two cross-realm
  `new other.Function()` exclusions. The complete AOT-applicable cohort reports
  `12/12` exact-green as of `2026-07-20`, covering abrupt prototype access,
  newTarget/prototype selection, result identity, and extensibility in both
  constructor families.
- Heap-backed BigInts now represent unsigned 64-bit DataView results and
  arbitrary-size literal magnitudes as canonical little-endian limbs. Runtime
  tags survive bindings, and strict equality, `Object.is`, `includes`,
  `typeof`, and truthiness compare the represented value rather than heap
  handles even when one operand is inline and the other is heap-backed. Unary
  minus uses `ToNumeric` and preserves captured heap-backed BigInts instead of
  forcing them through Number conversion. BigInt typed-array element
  conversion reduces arbitrary-size values modulo 2^64; other unsupported
  multi-limb arithmetic and conversions still reject explicitly. The fresh pinned
  `DataView.prototype.getBigUint64` leaf reports `21/21`.
- Array iterators observe configurable mapped and unmapped `arguments.length`;
  the pinned `ArrayIteratorPrototype` leaf reports `27/27`. Array search and
  concatenation propagate indexed getter exceptions, search methods use the
  observable Arguments length, Array/TypedArray `toString` rejects nullish
  receivers, and the complete `Array.prototype.toLocaleString` leaf reports
  `12/12` after preserving primitive receivers and resize-time length snapshots.
- Real-suite execution checkpoints every ten completed cases even on a first
  run, forced child snapshots live in private temporary directories, and the
  low-RAM publisher leaves node validation and retry decisions to the Rust
  harness. `test262 run` and `test262 shard` exit unsuccessfully whenever a
  selection is empty or any requested case does not pass, so shell batches can
  stop reliably instead of continuing after a conformance failure. On this machine, two case workers improved throughput without
  exceeding the task's 50% RAM ceiling; four workers added contention and were
  not retained. Dormant `$262.createRealm` support no longer roots every
  standard global: one representative non-realm case compiled 17.5% faster
  with 6.2% lower peak RSS, while a noisier control still used 9.9% less RSS
  and actual realm cases retained the full bootstrap. Low-RAM `--jobs 1` runs
  also disable Wasmtime's otherwise idle parallel-module preparation. Emission
  keeps its conservative temporary-local capacity while building a function,
  then declares only the observed high-water locals in the final Wasm body; a
  mixed ordinary/buffer/JSON/decimal/RegExp module validates with no function
  retaining the old 2,048-local floor. Direct-call exact-context propagation
  now memoizes completed function/context pairs within each fixed-point pass.
  The 36-pair SharedArrayBuffer TypedArray constructor case fell from about
  `1,196` seconds to about `57` seconds on the same release path while remaining
  exact-green, with lowering reduced to about `48` seconds and Wasmtime module
  compilation to about `8.5` seconds. Wasm-AOT
  async cases must now report exactly one `$DONE`
  completion after the Promise job queue drains; explicit failure, missing
  completion, and repeated completion can no longer false-green.
- Base-10 Number stringification now uses a complete Ryū shortest-roundtrip
  authority for emitted dynamic binary64 values and the pinned `ryu-js`
  authority for static lowering. Both paths apply ECMAScript's fixed/scientific
  spelling thresholds, including fixed `1e19` and `1e20`, scientific `1e21`,
  subnormals, adjacent powers, fractional rounding and `-0` normalization. The
  exact current-pin `toString` and `toFixed` leaves pass all `180/180` and
  `32/32` sloppy/strict Wasm-AOT executions, respectively; `toFixed` retains
  its distinct exact-integer rounding semantics. The shared semantic golden
  contains 656 fixture dumps: four new Atomics/DataView Realm fixtures and no
  removals. All 652 retained dumps preserve their imports, exports, runtime
  roots, helper counts, memory, data segments and name counts; code-size deltas
  partition exactly into the Ryū body, the Atomics Realm routes, the DataView
  Realm/publication routes and their combinations. Only the two deliberately
  expanded Number and Proxy fixtures also change main-function locals and
  largest-function attribution.
- Identifier `typeof` now performs a run-time global-property read after calls,
  conditional deletion, or observable `with` resolution make presence unknown;
  only names still proven absent use the unresolved fast path. The exact
  `BigInt.prototype.toString` leaf that exposed the defect now passes all
  `26/26` sloppy/strict Wasm-AOT executions, with an independent BigInt control
  at `2/2`.
- `JSON.parse` now applies the strict JSON number grammar to values nested in
  arrays and objects, including delimiter validation, without treating numeric
  text inside strings as tokens. Five formerly failing pinned SpiderMonkey
  numeric/trailing-comma files report `5/5` with every failure taxonomy at zero.
  Dynamic composite input is materialized by an iterative growable-frame AOT
  parser rather than falling through to numeric conversion; the pinned
  2,097,153-element mega-array case passes in 4.6 seconds. Decimal source text
  now uses one exact emitted conversion path for JSON, `parseFloat`, and
  `StringToNumber`; ten focused pinned number cases report `10/10` after
  removing an inexact compile-time JSON-number fold. Callable dynamic revivers
  now use iterative post-order traversal with per-node primitive source text,
  key/length snapshots, callable Proxy support, deletion and replacement; six
  representative pinned reviver cases pass. Lowering now gives known ordinary
  revivers the dynamic object-or-array holder shape, so forward holder mutation
  is observed during later traversal; a seven-case pinned reviver tranche is
  green. Indirect calls now preserve exact object and primitive throw identity
  through catch and completion save/restore paths. Array `[[Get]]` now stops
  before prototype fallback when an own accessor throws, clearing the final
  pinned reviver accessor-identity case. `JSON.rawJSON` and `JSON.isRawJSON`
  report `10/10` and `6/6` with strict primitive-text validation, an
  unforgeable frozen wrapper brand, and raw-source stringify embedding.
  `JSON.stringify` now snapshots replacer arrays into one spec PropertyList and
  treats callable Proxies as replacers, `toJSON` methods, and omittable callable
  values with the same ordering and abrupt-completion behavior as functions.
  Space coercion observes boxed `@@toPrimitive`/ordinary hooks and callable
  Proxies exactly once, uses defining-realm errors, and truncates indentation
  to ten UTF-16 code units rather than encoded bytes; its exact leaf remains
  `8/8` with focused non-ASCII and abrupt-order coverage. String quoting now
  decodes UTF-8/WTF-8 scalars generically, escapes every unmatched surrogate,
  canonicalizes valid pairs, and clears the exact string-value leaf at `3/3`.
  Primitive and boxed heap-backed BigInts now preserve their tag through
  `toJSON`, replacer, `valueOf`, and decimal/radix formatting paths, including
  positive and negative multi-limb values. Synthetic-realm `Object(BigInt)`
  wrappers inherit that realm's `BigInt.prototype`, while serialization errors
  come from the `JSON.stringify` defining realm; the exact pinned
  `value-bigint` cohort reports `6/6`.
- Focused Wasm-AOT coverage for `Date.prototype.toDateString`, `toTimeString`,
  `toString`, `toUTCString`/`toGMTString`, `toISOString`, and generic `toJSON`
  includes invalid dates, extended years, time-clip boundaries, correct
  RangeError/TypeError behavior, `ToPrimitive(number)` ordering, the
  non-finite-to-null path, and observable `toISOString` lookup and invocation.
  The pinned `toDateString`, `toTimeString`, and `toString` leaves report
  `7/7`, `6/6`, and `8/8`, respectively, on `2026-07-27`. Refresh one with
  `./target/release/lila --jobs 1 test262 run built-ins/Date/prototype/<method> --suite-root test262/vendor/test262 --execution-backend wasm-aot --timeout-ms 60000 --threads 1`.
  The exact pinned
  `Date.prototype[Symbol.toPrimitive]` leaf reports `18/18`, including property
  attributes, hint validation, ordinary conversion order, and abrupt hooks.
  A fresh isolated-cache run of the complete pinned `built-ins/Date` shard
  reports `591/594` on `2026-07-28`: all `591/591` AOT-applicable tests pass,
  and the only three unsupported tests use excluded cross-realm dynamic
  `Function` construction. This includes `Date.parse` at `8/8`, the three
  locale-string metadata leaves at `12/12`, callable and constructible Date
  coercion, private Date branding, and `toTemporalInstant` at `8/8`. Refresh
  with
  `LILA_CACHE_DIR=/tmp/lila-date-cache ./target/debug/lila --jobs 4 test262 run built-ins/Date --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 8 --timeout-ms 60000`.
  The exact `setUTCMonth/arg-coercion-order.js` witness now executes its
  unchanged pinned source with the full merged `assert.js` and vendored
  `compareArray.js` preludes. Both sloppy/strict Wasm-AOT executions pass
  `2/2` with every failure bucket at zero as of `2026-08-30`; a provenance
  test pins the helper origins and complete concatenated source bytes.
  `Temporal.Instant` now has a real namespace binding, constructor, prototype,
  private epoch-nanoseconds slot, branded `epochNanoseconds` and floor-rounded
  `epochMilliseconds` accessors, and a real static `from` path. `from` copies
  branded instants, performs object `ToPrimitive(string)`, parses exact ISO
  instant strings without routing through millisecond-only `Date.parse`, and
  preserves nanosecond offsets, annotations, leap seconds, and range
  boundaries. Its instances also have branded canonical UTC `toString`
  formatting. A distinct branded `Temporal.ZonedDateTime` record now retains
  exact epoch nanoseconds, a stored time-zone string, and the canonical
  `iso8601` calendar identifier; UTC spellings are canonicalized to `UTC`, and
  numeric offset syntax is range-checked. Its static `from` path now copies
  branded ZonedDateTime records and parses ISO strings with bracketed `UTC` or
  minute-precision numeric offset zones, including date-only forms, leap
  seconds, expanded years, calendar annotations, and the standard
  `disambiguation`, `offset`, and `overflow` option validation order. Fixed-zone
  `reject`, `use`, `prefer`, and `ignore` offset semantics retain exact
  nanoseconds. The same path accepts ordinary-object, function, and Array
  property bags with ISO `year`, `month` or `monthCode`, `day`, optional time
  and offset fields, and a required fixed-zone `timeZone`; field reads and
  options reads follow the Temporal order, while `overflow` performs ISO
  constrain/reject regulation before exact epoch-nanosecond construction.
  On `2026-07-29`, the complete pinned `withTimeZone`, `from`, and `equals`
  leaves report `14/16`, `78/91`, and `49/55`, respectively (artifacts
  `verify-temporal-zdt-withtimezone-current-20260729-3809840749374109757.json`,
  `verify-temporal-zdt-from-current-20260729-13737876247778637226.json`, and
  `verify-temporal-zdt-equals-current-20260729-5397219743879800583.json`).
  Branded `offset` and `offsetNanoseconds` accessors derive the canonical
  `±HH:MM` string and exact numeric nanoseconds directly from those fixed
  zones. Branded ISO civil accessors (`year`, `month`, `monthCode`, `day`,
  `hour`, `minute`, `second`, `millisecond`, `microsecond`, and `nanosecond`)
  project the exact epoch plus fixed offset, including negative sub-millisecond
  epochs; their ten pinned directories report `40/40`. ZonedDateTime
  `equals` compares the epoch, time-zone, and calendar slots after intrinsic
  argument conversion. The 21 residual failures group around missing real
  PlainDateTime and Duration support, ISO calendar-string and calendar-object
  conversion, ZonedDateTime string limits, and month-code/offset validation
  ordering. Named IANA zones remain explicit errors until the compiler has real
  time-zone transition resolution; they are not guessed through the host
  `Date` or `TZ` environment.
  `Temporal.Instant.from` copies that private epoch slot without consulting
  shadowable ZonedDateTime properties, and `Temporal.Instant.prototype.equals`
  compares the exact private BigInt epoch after the same intrinsic conversion.
  The exact pinned
  `built-ins/Temporal/Instant/basic.js` case and both three-case accessor leaves
  report `1/1`, `3/3`, and `3/3` on `2026-07-28`; `toTemporalInstant` returns
  that same intrinsic object model. This is not a broader Temporal conformance
  claim.
  The five plain `ToTemporal*` converters now accept the closed
  `TemporalConversionOverflowOptions` domain. Five public `from` producers
  carry real options payloads; fifteen internal conversions carry `Omit` and no
  dummy undefined locals. Sixteen exhaustive matches preserve the observable
  read points. The bounded structure target passes `3/3`, and the finite
  witness executes all twenty producers and passes `1/1`.
  PlainDateTime property-bag conversion and `with` now share a private
  two-case field-read mode, so only conversion can emit the calendar
  canonicalization step. `with` performs the required observable
  `Get("calendar")` and `Get("timeZone")` operations before the alphabetical
  field sweep and does not read `calendar` again. The bounded structure target
  passes `4/4`, the Proxy CLI witness passes `1/1`, and the pinned `from` order,
  `with` order, and forbidden-calendar leaves pass all `6/6` variants. The
  `with` order leaf moved from `0/2` runtime bugs before this repair to `2/2`.
  `Date.now` reads integer
  Unix-epoch milliseconds from the host wall clock; Atomics timeout scheduling
  continues to use the separate monotonic nanosecond clock. `Math.random`
  reads the current realm's validated `[0, 1)` host-randomness capability; the
  production provider uses operating-system entropy, while embedders and exact
  tests can inject a deterministic provider without changing the Wasm path.
- Complete pinned builtin-shard evidence reports `built-ins/Boolean` at
  `99/101` as of `2026-08-25` and `built-ins/DataView` at `559/561` as of
  `2026-07-28`. The two Boolean failures are the sloppy and strict variants of
  `S9.2_A1_T1.js`, which now reach the explicit unsupported dynamic `eval`
  boundary instead of handwritten source. All `559/559` AOT-applicable
  DataView tests pass, while its two unsupported tests use excluded dynamic
  `Function` construction. Combined exact evidence
  covers `built-ins/BigInt` at `77/77`: a fresh full-shard baseline passed
  `75/77`, then exact reruns passed the two corrected relational-comparison and
  wrapper `ToPrimitive` cases. Refresh a complete shard with
  `./target/debug/lila --jobs 4 test262 run built-ins/<Boolean|DataView|BigInt> --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 8 --timeout-ms 60000`.
- The first heap-backed `Map` slice implements nullish construction plus
  `clear`, `delete`, `get`, `has`, `set`, and the `size` accessor with ordered
  tombstoned entries, SameValueZero keys, and `-0` normalization. Twelve pinned
  non-iterator core cases report `12/12`. Iterable construction preserves
  setter/iterator observation order and `IteratorClose`; fourteen exact
  constructor cases report `14/14`. `Map.prototype.forEach` uses live ordered
  traversal and its exact pinned directory reports `19/19`. The complete Map
  iterator surface reports `43/43` across keys, values, entries, the default
  iterator, and `MapIteratorPrototype`, including live mutation and permanent
  exhaustion. `Map.groupBy` and `Object.groupBy` each report `14/14`; the
  latter preserves symbol keys, safely defines `__proto__`, and returns the
  required null-prototype object. Their shared compiler now carries the result
  through the closed two-case `GroupByResult` domain: its two wrappers are the
  only producers, and all eleven diagnostics, allocation, key-treatment and
  storage decisions are direct exhaustive matches. The bounded structure
  target passes `3/3`, and the finite Map-vs-Object witness passes `1/1`.
  `Object.fromEntries` reports `25/25` on
  `2026-07-29`, including symbol and duplicate keys, direct entry-property
  access, define semantics, and the required iterator-close boundary. Refresh
  it with
  `./target/debug/lila --jobs 1 test262 run built-ins/Object/fromEntries --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 1 --timeout-ms 120000`.
  `Map.prototype.getOrInsert` reports `14/14`,
  while the older `getOrInsertComputed` checkpoint reports `17/19`; that
  checkpoint predates WeakMap support and was not refreshed in this batch.
  Non-dynamic cross-realm
  constructor-prototype selection passes focused direct, bound, proxy, and
  revoked-proxy tests; the pinned realm file itself uses an excluded dynamic
  `Function` constructor and remains classified accordingly.
- `WeakMap` has a distinct internal brand, record layout, prototype, and weak
  entry representation rather than aliasing `Map`. Its constructor and
  `delete`, `get`, `has`, `set`, `getOrInsert`, and `getOrInsertComputed`
  methods implement weak-key eligibility, observable iterator/setter ordering,
  IteratorClose, and abrupt completion. Weak entry key/value relationships are
  registered as ephemeron metadata; the collector remains metadata-only, so
  this is not yet a claim of observable garbage-collection behavior. The
  complete pinned `built-ins/WeakMap` leaf reports `139/141` on `2026-07-27`;
  that historical run attributed both misses to dynamic `Function`
  construction. The current `proto-from-ctor-realm.js` root now passes `2/2`
  Wasm-AOT executions as part of the 2026-08-31 created-Realm cohort, but the
  complete leaf was not rerun and no new total is claimed. Refresh with
  `./target/release/lila --jobs 1 test262 run built-ins/WeakMap --suite-root test262/vendor/test262 --execution-backend wasm-aot --timeout-ms 60000 --threads 1`.
- `WeakSet` has a real global intrinsic, realm-aware prototype and `newTarget`
  allocation, a private brand, and a distinct weak entry layout. Its constructor
  consumes iterables through the observable `add` method with `IteratorClose`
  on abrupt completion; `add`, `delete`, and `has` support objects and
  nonregistered symbols with the ECMAScript weak-value rules. Weak entries are
  represented for the collector contract but cannot clear until collection is
  executable.
- `WeakRef` has a real global intrinsic, prototype, private brand, and tagged
  weak-target record. Construction accepts objects and non-registered symbols,
  observes `newTarget.prototype` with defining-realm fallback, and `deref`
  validates its receiver before returning the current target. The target record
  is registered as a weak edge and excluded from ordinary strong tracing.
  Observable clearing is not claimed yet: the Wasm-AOT collector and `gc()`
  hook remain non-executable, so synchronous `deref` coverage cannot
  deterministically exercise collection.
- `FinalizationRegistry` has a real global intrinsic, prototype, private brand,
  callable cleanup callback slot, and heap-backed cell list. `register` accepts
  object and non-registered-symbol targets and tokens, preserves holdings as a
  strong cell edge, rejects `SameValue(target, holdings)`, and appends distinct
  cells. `unregister` removes every cell with the matching token and releases
  its holdings. Created realms publish independent constructor, prototype and
  method identities with defining-Realm errors and NewTarget fallback. Target
  and token edges are registered as weak; cleanup delivery is not claimed
  because collection and finalizer queueing remain non-executable.
- The corresponding heap-backed `Set` core implements nullish construction,
  `add`, `clear`, `delete`, `has`, and `size` with ordered tombstones,
  SameValueZero values, distinct Map/Set brands, and defining-realm prototype
  fallback. Fifteen representative pinned non-iterator cases report `15/15`.
  Iterable construction captures `add` and `next`, preserves observable
  ordering, supports arrays, strings, functions, and callable Proxies, and
  performs `IteratorClose` only for abrupt adder calls; ten exact constructor
  and close cases report `10/10`. `Set.prototype.forEach` implements the live
  insertion-list semantics required for deletion, re-addition, clearing, and
  additions during callbacks; its exact pinned directory reports `33/33`.
  The corresponding Set iterator surface reports `49/49` with the required
  values/keys/default-iterator aliases, pair entries, brands, realms, and live
  traversal behavior. `difference`, `intersection`, `symmetricDifference`, and
  `union` now report `113/113`. `isDisjointFrom`, `isSubsetOf`, and
  `isSupersetOf` report `73/73`. Their fourteen former generator prerequisites
  are green after lazy generator activations and linear yield continuations
  replaced the eager static-generator path.

- Derived class construction now uses a per-invocation activation for the
  active constructor, `new.target`, initialization status, and `this`.
  Derived `[[Construct]]` defers instance allocation and `newTarget.prototype`
  observation until `super()`, caches the super constructor and new target
  before argument evaluation, and binds `this` only after the base construct
  completes. Direct and nested arrows share that live activation for
  `super()`, `this`, `new.target`, and super-property reads/calls, including
  repeated-`super()` ordering and escaped pre-initialization reads.
- Class constructors, methods, and accessors now carry their exact tagged
  `[[HomeObject]]` in the Wasm function context. Super-property lookup
  recomputes the base on every access while keeping the invocation or lexical
  `this` as receiver, covering detached/alien receivers, static members,
  nested arrows, computed calls, getters, and later prototype mutation.
  In supported class and lexical-class contexts, `delete super.x` and
  `delete super[key]` use a fused Reference plan which checks current `this`,
  evaluates the raw computed value without
  `ToPropertyKey`, and then throws `ReferenceError` without invoking property
  deletion. Non-resumable object-literal methods and accessors now use a
  separate typed HomeObject lifecycle, verified by the focused Wasm fixture and
  exact `10/10` cohort. Direct generator and async object-method controls are
  green, but complete suspension-safe and async-generator transport remain
  explicit debt. Object-method lexical-arrow transport has a separate
  dry-written closed owner-role boundary; its runtime verification remains
  deferred.
  The four focused real Test262 arrow files
  `lexical-supercall-from-immediately-invoked-arrow.js`,
  `lexical-super-call-from-within-constructor.js`,
  `lexical-super-property-from-within-constructor.js`, and
  `lexical-super-property.js` each report `1/1` under Wasm-AOT as of
  `2026-07-11`.
- Bound `[[Construct]]` now replaces `newTarget` only when it is the current
  bound function, preserves unrelated direct and nested bound identities, and
  leaves bound functions without an own `prototype`. Constructor prototype
  fallback now follows `GetFunctionRealm` through bound functions and Proxy
  targets, throws for revoked proxies after the observable prototype read, and
  selects the defining realm's Object, primitive-wrapper, Array, or concrete
  TypedArray intrinsic prototype.
- Array `length` writes now use the full `ArraySetLength` path across direct,
  computed, `Object.defineProperty`, `Reflect.defineProperty`, `Reflect.set`,
  and dynamically typed cross-realm assignments. The Wasm-AOT implementation
  performs the two independently observable numeric conversions, validates the
  exact uint32 result, preserves the current execution Realm for `RangeError`,
  respects non-writable length without coercion, shrinks sparse indexes in
  descending order, and applies deferred `writable: false` after a blocked
  shrink. Huge one-argument Array construction stays sparse. Materialized
  `Array.prototype` method access currently defaults to generic
  observable lookup until specialization has a runtime/version guard, covering
  direct, aliased, computed, helper-escaped, assignment, definition, and
  deletion mutations. The complete pinned real Test262
  `built-ins/Array/length` prefix reports `31/31` as of `2026-07-11` under
  `./target/debug/lila test262 run built-ins/Array/length --execution-backend wasm --timeout-ms 60000 --threads 4`.
- `Object.prototype.valueOf` now performs `ToObject` for Boolean, Number,
  String, Symbol, and BigInt primitives, preserves existing object identity,
  and selects primitive-wrapper prototypes and `TypeError` from the builtin's
  defining Realm. Property reads use the installed function object, so
  configurable `length` deletion and later `Object.prototype.valueOf`
  replacement remain observable. The complete pinned real Test262
  `built-ins/Object/prototype/valueOf` leaf reports `20/20` as of `2026-07-11`
  under `./target/debug/lila test262 run built-ins/Object/prototype/valueOf --execution-backend wasm --timeout-ms 90000 --threads 4`.
- `Object.prototype.isPrototypeOf` now preserves the required primitive-argument
  early return before `ToObject(this)`, throws for a nullish receiver only when
  the argument is an Object, and walks proxy-aware `[[GetPrototypeOf]]` links
  while propagating trap failures. The complete pinned real Test262
  `built-ins/Object/prototype/isPrototypeOf` leaf reports `10/10` as of
  `2026-07-11` under `./target/debug/lila test262 run built-ins/Object/prototype/isPrototypeOf --execution-backend wasm --timeout-ms 90000 --threads 2`.
- `Object.prototype.propertyIsEnumerable` now performs `ToPropertyKey` before
  receiver validation, preserves Symbols returned by `@@toPrimitive`,
  `toString`, or `valueOf`, and compares Symbol keys by identity without
  conflating equal descriptions or same-named strings. Abrupt key coercion
  propagates before a nullish-receiver error, whose `TypeError` comes from the
  builtin's defining Realm. The complete pinned real Test262
  `built-ins/Object/prototype/propertyIsEnumerable/` leaf reports `16/16` as
  of `2026-07-11` under
  `./target/debug/lila test262 run 'built-ins/Object/prototype/propertyIsEnumerable/' --execution-backend wasm --timeout-ms 90000 --threads 1`.
- `Array.prototype.join` is now installed as a real Wasm-AOT standard builtin
  in the main and created Realms. Its generic path performs `ToObject`, captures
  `LengthOfArrayLike` before separator coercion, observes inherited indexed
  properties, treats nullish elements as empty strings, and propagates abrupt
  length, separator, and element conversions. Calls copied onto ordinary
  objects and direct calls after aliased `Array.prototype.join` replacement use
  runtime `GetV` plus indirect dispatch instead of an Array-only fast path. The
  complete pinned real-Test262 `built-ins/Array/prototype/join` leaf reports
  `23/23` with no unsupported cases, bugs, or crashes as of `2026-07-15`,
  including fixed and length-tracking TypedArray views across resizable-buffer
  growth and shrink during separator coercion. Refresh with
  `./target/debug/lila test262 run built-ins/Array/prototype/join --execution-backend wasm --timeout-ms 90000 --threads 4`.
- `%TypedArray%.prototype.join` is a distinct non-generic Wasm-AOT builtin. It
  validates the receiver and its initial view before separator coercion,
  captures the internal typed-array length without observing shadowing
  `length` accessors, returns empty fields when separator coercion detaches the
  buffer, formats Number and BigInt elements, and preserves abrupt completion
  ordering. The complete pinned real-Test262
  `built-ins/TypedArray/prototype/join` leaf reports `32/32`, with no
  unsupported cases, bugs, crashes, or timeouts as of `2026-07-21` (manifest
  `11374343618813182054`). Refresh under the low-RAM settings with
  `LILA_TEST262_FORCE_CASE_RUNNER=1 ./target/release/lila --jobs 1 test262 run built-ins/TypedArray/prototype/join --suite-root test262/vendor/test262 --execution-backend wasm-aot --timeout-ms 60000 --threads 1 --snapshot-name typedarray-prototype-join-current-pin-32`.
- `Array.prototype.toLocaleString` is now installed as a Wasm-AOT standard
  builtin with generic array-like receiver support, `LengthOfArrayLike`
  conversion ordering, comma separator assembly, primitive element string
  method lookup through boxed primitives while preserving the original receiver
  for strict calls, custom object element `toLocaleString` invocation, and
  outer call spread arguments that must be ignored by the array builtin while
  still being evaluated. `Object.prototype.toLocaleString` is now installed for
  that dispatch path and calls the receiver's `toString` method. The exact real Test262
  `staging/sm/Array/toLocaleString-01.js` file reports `1/1` as of
  `2026-06-23` under
  `./target/debug/lila test262 run staging/sm/Array/toLocaleString-01.js --execution-backend wasm --timeout-ms 90000 --threads 1`.
  Typed-array receivers backed by resizable ArrayBuffers now use the typed-array
  length and integer-indexed element paths, including fixed-length
  out-of-bounds views and length-tracking views after resize. The broader
  `built-ins/Array/prototype/toLocaleString` leaf reports `12/12`
  as of `2026-06-23` under
  `./target/debug/lila test262 run built-ins/Array/prototype/toLocaleString --execution-backend wasm --timeout-ms 90000 --threads 4`.
  `%TypedArray%.prototype.toLocaleString` is also installed as a distinct
  non-generic Wasm-AOT builtin for concrete typed-array method calls, including
  internal-brand and current-view validation, internal length snapshotting,
  primitive-receiver element calls, callable Proxies, abrupt propagation, and
  resizable-buffer cases. Forged internal-looking properties and own `length`
  accessors do not affect typed-array semantics. The historical current-pin
  `built-ins/TypedArray/prototype/toLocaleString` leaf reports `39/39` as of
  `2026-07-21`, with every failure bucket and timeout count at zero (manifest
  `2525782695925974509`). The current working tree removes the family-specific
  helper split; all 39 physical sources now pass in both Script modes from
  unchanged bodies and the full upstream helper where declared (`78/78`).
  Refresh it with
  `./target/release/lila --jobs 1 test262 run built-ins/TypedArray/prototype/toLocaleString --suite-root test262/vendor/test262 --execution-backend wasm-aot --timeout-ms 60000 --threads 4 --snapshot-name typedarray-prototype-to-locale-string-current-pin-39`.
- `%TypedArray%.prototype.toString` now uses the same Wasm-AOT function object
  as `Array.prototype.toString`, so the shared identity and descriptor checks
  are exposed on `%TypedArray%.prototype` while Array receivers still use comma
  join semantics, including inherited array indexes and the intrinsic
  `Object.prototype.toString` fallback when `join` is not callable. That
  fallback recursively classifies direct and nested Proxy-wrapped Arrays,
  while the preceding Proxy-aware `Get(O, "join")` and fallback `IsArray` both
  reject revoked Proxies with the borrowed builtin function's Realm. The
  outlined Proxy `[[Get]]` helper receives only a trusted standard-builtin
  Realm environment or the main-Realm fallback; user/host lexical environments
  are never interpreted as Realm metadata. The fallback preserves the complete
  callable and internal-brand tag decision before `@@toStringTag`. The unchanged full-harness
  `non-callable-join-string-tag.js` source passes both ordinary Wasm-AOT
  variants as of `2026-08-26`. The unchanged `%TypedArray%.prototype.toString`
  identity/descriptor and non-constructor sources now also use the complete
  vendored `propertyHelper.js`, `testTypedArray.js` and `isConstructor.js`
  harnesses instead of path/source-selected reduced preludes. Both exact files
  report `2/2` ordinary Wasm-AOT variants as of `2026-08-26`, with every
  failure and unsupported bucket at zero. Meanwhile,
  TypedArray receivers perform `ValidateTypedArray` before joining indexed
  elements. The exact real Test262 `built-ins/Array/prototype/toString` leaf
  reports `11/11` as of `2026-06-23` under
  `./target/debug/lila test262 run built-ins/Array/prototype/toString --execution-backend wasm --timeout-ms 90000 --threads 4`.
  The exact real Test262
  `built-ins/TypedArray/prototype/toString` leaf reports `4/4` as of
  `2026-06-23` under
  `./target/debug/lila test262 run built-ins/TypedArray/prototype/toString --execution-backend wasm --timeout-ms 120000 --threads 4`.
- Proxy `[[Set]]` now carries the executing standard builtin's Realm through a
  closed source projection. ObjectWrite, both receiver-side helpers and both
  OrdinarySet helpers accept only a caller-projected standard-builtin Realm
  record or zero, so nonzero user and host lexical environments cannot be read
  as Realm metadata. Nested `Reflect.set` function materialization consumes
  that same typed projection. Direct and prototype-forwarded revoked handlers,
  non-callable traps, strict falsy trap results and incompatible frozen target
  descriptors therefore construct TypeErrors in a borrowed created-Realm
  Array or Reflect builtin's defining Realm. Assignment keeps its strict-mode
  guard, while both Array push paths unconditionally throw for their internal
  `Set(..., Throw=true)`. The exhaustive structure target
  passes `4/4`, its typed projection unit test passes `1/1`, and the ten-branch
  borrowed-builtin CLI fixture passes `1/1` as of `2026-08-26`. The shared
  semantic golden passes `2/2` across 658 fixture dumps, adding only
  `wasm_atomics_created_realm.js` and `wasm_proxy_set_error_realm.js` to the
  prior 656-dump checkpoint. No retained fixture changes any structural field
  except emitted-function byte sizes: all 656 retain their roots, builtin and
  helper counts, locals, imports, exports, globals, memories, data segments and
  name counts.
- `Array.prototype.forEach` covers array-like and primitive receivers,
  inherited array indexes including Array instances used as prototypes where
  `HasProperty` and `Get` must agree, ToLength and callback-order edge cases,
  sparse high-index arrays without timing out, omitted-callback TypeErrors,
  freezing `Array.prototype.forEach` while an iteration is active, and generic
  calls on typed arrays backed by resizable ArrayBuffers. The exact real
  Test262 `built-ins/Array/prototype/forEach` leaf reports `190/190` as of
  `2026-07-15` under
  `./target/debug/lila test262 run built-ins/Array/prototype/forEach --execution-backend wasm --timeout-ms 180000 --threads 4`.
- Generic `Array.prototype.every`, `Array.prototype.some`,
  `Array.prototype.filter`, and `Array.prototype.includes` calls on resizable
  typed arrays cover fixed-length and length-tracking views across shrink/grow,
  mid-iteration resize, fromIndex coercion resize, and `SameValueZero` float
  comparisons such as `NaN`. The exact real Test262
  `built-ins/Array/prototype/every` leaf reports `218/218` as of `2026-07-15`
  under
  `./target/debug/lila test262 run built-ins/Array/prototype/every --execution-backend wasm --timeout-ms 180000 --threads 4`.
  The `built-ins/Array/prototype/some` leaf reports `219/219` as of
  `2026-07-15` under
  `./target/debug/lila test262 run built-ins/Array/prototype/some --execution-backend wasm --timeout-ms 180000 --threads 4`.
  The `built-ins/Array/prototype/filter` leaf reports `242/242` as of
  `2026-07-15` under
  `./target/debug/lila test262 run built-ins/Array/prototype/filter --execution-backend wasm --timeout-ms 180000 --threads 4`.
- Generic `Array.prototype.indexOf` now observes `HasProperty` before `Get` for
  sparse and array-like receivers, supports borrowed calls on resizable typed
  arrays including subclass instances, preserves strict equality for special
  float values where `NaN` is not a match, and handles large canonical numeric
  object keys without clamping them to dense array indexes. Ordinary array
  writes at `4294967294` now extend `length` through the sparse element path,
  while `4294967295` and larger numeric literals remain named properties. The
  exact real Test262
  `built-ins/Array/prototype/indexOf/15.4.4.14-9-9.js`,
  `built-ins/Array/prototype/indexOf/15.4.4.14-9-a-19.js`,
  `built-ins/Array/prototype/indexOf/15.4.4.14-9-b-i-15.js`,
  `built-ins/Array/prototype/indexOf/resizable-buffer.js`,
  `built-ins/Array/prototype/indexOf/resizable-buffer-special-float-values.js`,
  `built-ins/Array/prototype/indexOf/coerced-searchelement-fromindex-grow.js`,
  `built-ins/Array/prototype/indexOf/coerced-searchelement-fromindex-shrink.js`,
  and `built-ins/Array/prototype/indexOf/length-near-integer-limit.js` files now
  report `1/1` each as of `2026-06-19` under `--execution-backend wasm` with the
  `90000` ms timeout and one thread. The sharded
  `built-ins/Array/prototype/indexOf` sweep also reports green as of
  `2026-06-19`: shard `1/8` is `26/26`, and shards `2/8` through `8/8` are
  `25/25` each under `--execution-backend wasm --timeout-ms 90000 --threads 8`.
  Refresh individual cases with
  `./target/debug/lila test262 run <case> --execution-backend wasm --timeout-ms 90000 --threads 1`.
- Generic `Array.prototype.lastIndexOf` now shares the index-search receiver,
  `HasProperty`, sparse array, array-like, and resizable typed-array paths while
  preserving the spec distinction between omitted `fromIndex` and explicit
  `undefined`. The exact real Test262
  `built-ins/Array/prototype/lastIndexOf/15.4.4.15-5-4.js`,
  `built-ins/Array/prototype/lastIndexOf/resizable-buffer.js`,
  `built-ins/Array/prototype/lastIndexOf/coerced-position-grow.js`, and
  `built-ins/Array/prototype/lastIndexOf/coerced-position-shrink.js` files now
  report `1/1` each as of `2026-06-19` under `--execution-backend wasm` with the
  `90000` ms timeout and one thread. The sharded
  `built-ins/Array/prototype/lastIndexOf` sweep reports green as of
  `2026-06-19`: shards `1/8` through `6/8` are `25/25`, shard `7/8` is `24/24`,
  and shard `8/8` is `24/24` under
  `--execution-backend wasm --timeout-ms 90000 --threads 8`.
- `Array.prototype.find` and `Array.prototype.findIndex` are now registered
  Wasm-AOT builtins with descriptor metadata, callback argument/`thisArg`
  plumbing, hole visitation, length-snapshot behavior, catchable non-callable
  `TypeError`s, and borrowed calls on resizable typed-array receivers. The exact
  real Test262
  `built-ins/Array/prototype/find/resizable-buffer.js`,
  `built-ins/Array/prototype/findIndex/resizable-buffer.js`,
  `built-ins/Array/prototype/find/callbackfn-resize-arraybuffer.js`,
  `built-ins/Array/prototype/findIndex/callbackfn-resize-arraybuffer.js`,
  `built-ins/Array/prototype/find/resizable-buffer-grow-mid-iteration.js`,
  `built-ins/Array/prototype/find/resizable-buffer-shrink-mid-iteration.js`,
  `built-ins/Array/prototype/findIndex/resizable-buffer-grow-mid-iteration.js`,
  and
  `built-ins/Array/prototype/findIndex/resizable-buffer-shrink-mid-iteration.js`
  files now report `1/1` each as of `2026-06-19` under
  `--execution-backend wasm` with the `60000` ms timeout and one thread. The
  local `wasm_array_find_core.js` fixture also covers function metadata, holes,
  callback parameters, `thisArg`, length snapshots, and typed-array
  post-shrink `undefined` callback values. The complete pinned real-Test262
  `find` and `findIndex` leaves each report `23/23`, with no unsupported cases,
  bugs, or crashes as of `2026-07-15`. Refresh a leaf with
  `./target/debug/lila test262 run built-ins/Array/prototype/<method> --execution-backend wasm --timeout-ms 90000 --threads 4`.
- `Array.prototype.findLast` and `Array.prototype.findLastIndex` are now
  registered Wasm-AOT builtins sharing the find-like callback path with reverse
  length-snapshot traversal. The local `wasm_array_find_last_core.js` fixture
  covers descriptor metadata, reverse callback order, holes, `thisArg`,
  mutation during traversal, non-callable `TypeError`s, and typed-array
  post-shrink callback values. Exact real Test262 metadata files
  `length.js`, `name.js`, and `prop-desc.js` for both methods report `1/1`
  each as of `2026-06-19` under `--execution-backend wasm` with the `60000` ms
  timeout and one thread. The exact real Test262
  `predicate-called-for-each-array-property.js`,
  `callbackfn-resize-arraybuffer.js`, `resizable-buffer.js`,
  `resizable-buffer-grow-mid-iteration.js`, and
  `resizable-buffer-shrink-mid-iteration.js` files for both reverse methods
  now report `1/1` each under `--execution-backend wasm` with the `90000` ms
  timeout and one thread. The complete pinned real-Test262 `findLast` and
  `findLastIndex` leaves each report `24/24`, with no unsupported cases, bugs,
  or crashes as of `2026-07-15`. Refresh a leaf with
  `./target/debug/lila test262 run built-ins/Array/prototype/<method> --execution-backend wasm --timeout-ms 90000 --threads 4`.
- `Array.prototype.reduce` and `Array.prototype.reduceRight` are registered
  Wasm-AOT builtins with generic `LengthOfArrayLike`, length snapshots,
  directional `HasProperty`/`Get` traversal, inherited and accessor-backed
  indexes, exact callback arguments and abrupt completion propagation,
  initial-value and empty-input semantics, Array instances used as prototypes,
  and fixed-length or length-tracking typed-array views across resizable-buffer
  grow and shrink. The complete pinned real-Test262 leaves report `260/260`
  for each method, `520/520` combined, with no unsupported cases, bugs, or
  crashes as of `2026-07-16`. Refresh either leaf within a 4 GiB task-memory
  cap with
  `LILA_TEST262_FORCE_CASE_RUNNER=1 LILA_CACHE_DIR=$HOME/.cache/lila-test262 systemd-run --user --wait --collect --pipe -p MemoryHigh=3G -p MemoryMax=4G -p MemorySwapMax=8G --working-directory="$PWD" ./target/release/lila --jobs 1 test262 run built-ins/Array/prototype/reduce/ --suite-root test262/vendor/test262 --execution-backend wasm --timeout-ms 60000 --threads 1 --snapshot-name array-reduce-current --resume`;
  replace `reduce` with `reduceRight` for the reverse leaf.
- Optional chains now have ordered property/call IR and Wasm-AOT lowering for
  dot keys, computed keys, and calls. The implementation evaluates each base,
  key, getter, and argument in spec order; keeps optional arguments lazy;
  preserves the method receiver and strict `this` through direct, grouped, and
  `super` calls; scopes short-circuiting to each contiguous chain segment; and
  performs primitive property lookup through the live mutable prototype.
  Computed reads after the chain's nullish check use the shared dynamic-property
  dispatcher, keeping repeated optional reads below Wasmtime's per-function
  compilation limit without evaluating skipped keys. The
  checked-out real-Test262 `language/expressions/optional-chaining` leaf reports
  `37/38` with no bugs or crashes as of `2026-07-27`. The sole remaining case
  uses excluded dynamic `eval`, so all `37/37` AOT-applicable roots pass.
  Refresh with
  `./target/debug/lila test262 run language/expressions/optional-chaining --suite-root test262/vendor/test262 --execution-backend wasm --timeout-ms 60000 --threads 4`.
- Tagged templates now lower as ordinary calls with preserved member receivers,
  source-site template-object identity, cooked and raw strings, invalid-escape
  `undefined` values, and frozen array/property descriptors. The checked-out
  real-Test262 `language/expressions/tagged-template` leaf reports `21/27` as of
  `2026-07-16`: all `21` Wasm-AOT-applicable cases pass, including the two
  strict-mode proper-tail-call cases; the other six cases require excluded
  dynamic source evaluation. Refresh with
  `./target/debug/lila test262 run language/expressions/tagged-template --suite-root test262/vendor/test262 --execution-backend wasm --timeout-ms 60000 --threads 4`.
- Strict-mode proper tail calls use Wasm `return_call` and
  `return_call_indirect` through the shared callable dispatcher. Tail position
  is preserved through tagged calls, conditional and comma expressions, and
  the right-hand side of `&&`, `||`, and `??`; labels may target any statement.
  All `30` AOT-applicable pinned language tests carrying the
  `tail-call-optimization` feature pass as of `2026-07-16`. The other four use
  excluded dynamic `eval`. Refresh the exact cases with
  `rg -l 'tail-call-optimization' test262/vendor/test262/test/language | sed 's#test262/vendor/test262/test/##' | while read -r test; do ./target/debug/lila test262 run "$test" --suite-root test262/vendor/test262 --execution-backend wasm --timeout-ms 60000 --threads 1; done`.
- `Array.prototype.flat` and `flatMap` now preserve dynamic custom-species
  result tags, avoid exposing typed-array implementation slots through Proxy
  `get` traps, and keep unproven concat/flat result shapes conservative.
  Computed numeric and string index reads on arrays now fall through holes to
  inherited properties and call inherited getters with the original array as
  receiver. The source-free metadata and custom-species harness
  materializations retain exact descriptor, constructor, `new.target`, and
  abrupt-completion assertions
  without loading heavyweight helper paths; the real Proxy path preserves the
  exact observable access counts. The combined
  pinned real-Test262 `built-ins/Array/prototype/flat` prefix reports `43/43`,
  and the exact `flatMap` leaf reports `24/24`, with no unsupported cases,
  bugs, or crashes as of `2026-07-11`. Refresh with
  `./target/debug/lila test262 run built-ins/Array/prototype/flat --suite-root test262/vendor/test262 --execution-backend wasm --timeout-ms 120000 --threads 4`.
- `Array.prototype.reverse`, `copyWithin`, `toReversed`, `toSpliced`,
  `toSorted`, and `with` are installed as real Wasm-AOT builtins. The mutating
  methods preserve holes, inherited properties, proxy-observable operations,
  overlap direction, and resizable typed-array integer-index behavior; the
  change-by-copy methods create dense ordinary arrays without consulting
  species. The complete pinned real-Test262 leaves report `18/18` for
  `reverse`, `39/39` for `copyWithin`, `17/17` for `toReversed`, `30/30` for
  `toSpliced`, `21/21` for `toSorted`, and `21/21` for `with` as of
  `2026-07-15`. Refresh a leaf with
  `./target/debug/lila test262 run built-ins/Array/prototype/<method> --execution-backend wasm --timeout-ms 60000 --threads 4`.
- `Array.prototype.concat` handles species creation, proxies and revoked
  proxies, sparse and inherited indexes, spreadable Arguments and TypedArray
  objects, maximum-safe-length rejection, abrupt getters, and inherited
  numeric properties on Function objects without reading Function records as
  Object boxed-primitive or TypedArray state. Combined exact evidence accounts
  for all `69/69` pinned real-Test262 roots with no unsupported cases, bugs, or
  crashes as of `2026-07-27`: a fresh `68/69` full-leaf baseline plus the exact
  corrected survivor. Refresh the complete leaf with
  `./target/debug/lila test262 run built-ins/Array/prototype/concat --execution-backend wasm --timeout-ms 90000 --threads 4`.
- `Array.prototype.slice` preserves sparse and inherited indexes, species
  construction, proxy-observable operations, and the current integer-index
  bounds of fixed and length-tracking TypedArrays over resizable buffers. Its
  complete pinned real-Test262 leaf reports `71/71` with no unsupported cases,
  bugs, or crashes as of `2026-07-15`. Refresh with
  `./target/debug/lila test262 run built-ins/Array/prototype/slice --execution-backend wasm --timeout-ms 90000 --threads 4`.
- `Array.prototype.fill` distinguishes omitted and explicit-`undefined` bounds,
  preserves observable coercion and write ordering, and writes through the
  integer-indexed storage of fixed and length-tracking TypedArrays over
  resizable buffers. Its complete pinned real-Test262 leaf reports `22/22`
  with no unsupported cases, bugs, or crashes as of `2026-07-15`. Refresh with
  `./target/debug/lila test262 run built-ins/Array/prototype/fill --execution-backend wasm --timeout-ms 90000 --threads 4`.
- `Array.prototype.pop` follows the generic `ToObject`/`LengthOfArrayLike`,
  `Get`, `DeletePropertyOrThrow`, and strict length-update sequence. It handles
  inherited indexes, primitive receivers, maximum-safe lengths, frozen arrays,
  and non-writable length properties. Its complete pinned real-Test262 leaf
  reports `23/23` with no unsupported cases, bugs, or crashes as of
  `2026-07-15`. Refresh with
  `./target/debug/lila test262 run built-ins/Array/prototype/pop --execution-backend wasm --timeout-ms 90000 --threads 4`.
- `Array.prototype.push` handles generic receivers, primitive boxing,
  maximum-safe-length rejection, proxy-observable writes, and strict failures
  for frozen or non-writable targets. Its complete pinned real-Test262 leaf
  reports `24/24` with no unsupported cases, bugs, or crashes as of
  `2026-07-15`. Refresh with
  `./target/debug/lila test262 run built-ins/Array/prototype/push --execution-backend wasm --timeout-ms 90000 --threads 4`.
- `Array.prototype.shift` and `Array.prototype.unshift` have complete pinned
  real-Test262 leaves at `20/20` and `22/22`, respectively, with no unsupported
  cases, bugs, or crashes as of `2026-07-15`. Refresh a leaf with
  `./target/debug/lila test262 run built-ins/Array/prototype/<method> --execution-backend wasm --timeout-ms 90000 --threads 4`.
- `Array.prototype.splice` has a complete pinned real-Test262 leaf at `81/81`,
  with no unsupported cases, bugs, or crashes as of `2026-07-15`. Refresh with
  `./target/debug/lila test262 run built-ins/Array/prototype/splice --execution-backend wasm --timeout-ms 120000 --threads 4`.
- `Array.prototype.sort` has a complete pinned real-Test262 leaf at `54/54`,
  with no unsupported cases, bugs, or crashes as of `2026-07-15`. Refresh with
  `./target/debug/lila test262 run built-ins/Array/prototype/sort --execution-backend wasm --timeout-ms 120000 --threads 4`.
- `Array.isArray` has a complete pinned real-Test262 leaf at `29/29`, with no
  unsupported cases, bugs, or crashes as of `2026-07-15`. Refresh with
  `./target/debug/lila test262 run built-ins/Array/isArray --execution-backend wasm --timeout-ms 90000 --threads 4`.
- `Array.of` passes all `15/15` Wasm-AOT-applicable cases as of `2026-07-15`.
  The remaining `proto-from-ctor-realm.js` case explicitly constructs source
  through another Realm's `Function` constructor and is tracked as an excluded
  dynamic-code-generation case. Refresh with
  `./target/debug/lila test262 run built-ins/Array/of --execution-backend wasm --timeout-ms 90000 --threads 4`.
- `Array.from` passes all `46/46` Wasm-AOT-applicable cases as of `2026-07-15`.
  Its remaining `proto-from-ctor-realm.js` case has the same explicit
  cross-realm `Function`-constructor dependency and is tracked as excluded
  dynamic code generation. Refresh with
  `./target/debug/lila test262 run built-ins/Array/from --execution-backend wasm --timeout-ms 120000 --threads 4`.
- `Array[Symbol.species]` has a complete pinned real-Test262 leaf at `4/4`,
  with no unsupported cases, bugs, or crashes as of `2026-07-15`. Refresh with
  `./target/debug/lila test262 run built-ins/Array/Symbol.species --execution-backend wasm --timeout-ms 90000 --threads 4`.
- `Array.prototype.includes` now performs the observable generic
  `ToObject`/`LengthOfArrayLike` sequence for every receiver, including
  TypedArrays with own `length` properties, while indexed reads recognize real
  TypedArrays by their internal brand rather than spoofable named properties.
  Proxy receivers therefore expose only the specified `length` and index
  `Get` operations. Derived TypedArray constructors also reuse their canonical
  bootstrapped super constructor so element width and kind metadata survive
  polymorphic construction. The pinned real-Test262
  `built-ins/Array/prototype/includes` leaf reports `30/30`, with no
  unsupported cases, bugs, or crashes as of `2026-07-11`. Refresh with
  `./target/debug/lila test262 run built-ins/Array/prototype/includes --suite-root test262/vendor/test262 --execution-backend wasm --timeout-ms 120000 --threads 4`.
- The exact real Test262
  `Array.prototype.map/callbackfn-resize-arraybuffer.js`,
  `Array.prototype.every/callbackfn-resize-arraybuffer.js`,
  `Array.prototype.forEach/callbackfn-resize-arraybuffer.js`,
  `Array.prototype.filter/callbackfn-resize-arraybuffer.js`, and
  `Array.prototype.some/callbackfn-resize-arraybuffer.js` cases now use static
  Wasm-AOT materializations that preserve passthrough typed-array constructor
  coverage without timing out in the generic `testTypedArray.js` helper path.
  The complete pinned real-Test262 `Array.prototype.map` leaf reports `216/216`,
  with no unsupported cases, bugs, or crashes as of `2026-07-15`. Refresh with
  `./target/debug/lila test262 run built-ins/Array/prototype/map --execution-backend wasm --timeout-ms 180000 --threads 4`.
- The exact real Test262 `Array.prototype.every/resizable-buffer.js`,
  `Array.prototype.some/resizable-buffer.js`,
  `Array.prototype.filter/resizable-buffer.js`, and
  `Array.prototype.values/resizable-buffer.js` files now report `1/1` each as
  of `2026-06-18` under `--execution-backend wasm` with the `60000` ms timeout
  and one thread. These self-contained materializations still call the real
  Array methods on resizable `Uint8Array` views. The `every` and `some` files
  cover fixed, fixed-offset, length-tracking, and offset length-tracking views
  across shrink/grow states; the `filter` file keeps fixed-length and
  length-tracking checks in the exact Test262 materialization to stay below the
  timeout, with offset coverage retained in the local focused fixture and the
  mid-iteration exact files. The `values` file now checks real
  `Array.prototype.values` iterators for fixed initial values, a
  length-tracking value after shrink, and the fixed-length out-of-bounds
  `TypeError` branch while staying under the exact-file timeout.
- The complete pinned real-Test262 `Array.prototype.keys`, `entries`, and
  `values` leaves each report `12/12`, with no unsupported cases, bugs, or
  crashes as of `2026-07-15`. Their resizable-buffer cases call the real
  iterators on `Uint8Array` views, covering initial fixed-length iteration,
  length-tracking and offset views after shrink, and out-of-bounds `TypeError`
  checks for fixed or offset views. As of `2026-08-26`, the keys
  resizable-buffer case runs its unchanged vendored body through the general
  `Array.from` iterator path and passes both Wasm-AOT execution modes; its old
  source-spliced collector and keys-specific self-contained materializer are
  gone. The three Array methods and their three strict TypedArray counterparts
  now select a closed two-variant receiver policy; validation and iterator
  materialization are two exhaustive projections, preserving generic borrowing
  and runtime TypedArray specialization without a raw Boolean. The bounded
  producer/consumer census passes `3/3`, and the finite all-six-method fixture
  passes `1/1`. The following 667-dump semantic golden passes `2/2` in 702.89
  seconds, adds only that fixture, removes none and preserves every retained
  non-accounting summary except the independently expanded Promise callback
  witness. Refresh a leaf with
  `./target/debug/lila test262 run built-ins/Array/prototype/<method> --execution-backend wasm --timeout-ms 90000 --threads 4`.
  The shared `reduce`, `reduceRight` and `forEach` compiler family now projects
  the closed two-case `ArrayCallbackReceiverKind` directly for generic Array
  and strict TypedArray entries. Thirteen exhaustive matches replace two
  equality collapses and two Boolean carriers while preserving the existing
  validated-entry and live integer-indexed witnesses. The bounded structure
  target passes `4/4`, and the three existing focused runtime witnesses pass
  `3/3`.
- `Array.prototype[Symbol.iterator]` aliases `values`, and
  `Array.prototype[Symbol.unscopables]` is the standard null-prototype object
  with its non-writable, non-enumerable, configurable prototype property. The
  complete pinned real-Test262 leaves report `1/1` and `4/4`, respectively,
  with no unsupported cases, bugs, or crashes as of `2026-07-15`.
- The full `built-ins/Array/prototype/at` leaf now reports `13/13` passing as
  of `2026-06-18` under `--execution-backend wasm` with the `60000` ms timeout
  and four threads (`0` unsupported, `0` runtime failures) with
  `./target/debug/lila test262 run built-ins/Array/prototype/at --execution-backend wasm --timeout-ms 60000 --threads 4`.
  The resizable typed-array materializations call the real
  `Array.prototype.at.call` on resizable `Uint8Array` fixed, fixed-offset,
  length-tracking, and offset length-tracking views across shrink/grow states,
  including negative indexing, out-of-range `undefined`, grow-after-shrink
  zero-filled bytes, and the `coerced-index-resize.js` ordering where
  `LengthOfArrayLike` is captured before index `valueOf` resizes the backing
  ArrayBuffer. The `length`, `name`, and `prop-desc` metadata files now use the
  same static descriptor materializer as the other Array prototype methods.
- The exact current-pin real Test262 `built-ins/TypedArray/prototype/at` leaf
  reports `30/30` under Wasm-AOT as of `2026-08-30`, with every failure bucket
  and timeout count at zero (manifest `4266848050910758690`). Refresh it with
  `./target/debug/lila --jobs 2 test262 run built-ins/TypedArray/prototype/at --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 2 --timeout-ms 60000 --snapshot-name t03-typedarray-at-full-helper`.
  Wasm-AOT exposes a distinct `%TypedArray%.prototype.at` intrinsic, so both
  direct `ta.at(...)` and dynamically selected typed intrinsic calls perform
  full brand, detached-buffer, and out-of-bounds validation. Generic
  `Array.prototype.at.call(typedArray, ...)` remains non-validating as required.
  `BigInt64Array` and `BigUint64Array` are now registered in the Rust IR and
  AOT typed-array constructor tables, the Wasm-AOT harness enumerates them for
  BigInt typed-array constructor helper calls, and typed-array element access
  handles 64-bit BigInt element kinds for direct reads and indexed writes. The
  previously unsupported exact file
  `built-ins/TypedArray/prototype/at/BigInt/return-abrupt-from-this-out-of-bounds.js`
  now reaches its unchanged pinned source with the complete vendored
  `testTypedArray.js`; that source constructs real resizable `BigInt64Array`
  and `BigUint64Array` fixed views and checks the `.at(0)` out-of-bounds
  `TypeError` branch after shrink. The two `resizableArrayBufferUtils.js`
  consumers retain T13's separately owned static-subclass substitution.
- The five `%TypedArray%.prototype` accessor leaves `buffer`, `byteLength`,
  `byteOffset`, `length`, and `Symbol.toStringTag` report `12/12`, `18/18`,
  `16/16`, `18/18`, and `18/18` under Wasm-AOT as of `2026-07-22`, with every
  failure bucket and timeout count at zero (manifests `11070985033035019212`,
  `10250546625084257154`, `17214440125282356686`, `6654572848190539168`, and
  `8948270343658110990`). The numeric accessors distinguish detached and
  resized-out-of-bounds views from valid fixed and length-tracking views, while
  `@@toStringTag` reads the internal element kind without validating the
  backing buffer and returns `undefined` for receivers without TypedArray
  internal slots. Refresh an accessor by replacing `<accessor>` in
  `./target/release/lila --jobs 1 test262 run built-ins/TypedArray/prototype/<accessor> --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 1 --timeout-ms 60000 --snapshot-name typedarray-prototype-<accessor>-current-pin`.
- The `%TypedArray%.prototype` iterator leaves `values`, `keys`, `entries`, and
  `Symbol.iterator` report `21/21`, `19/19`, `19/19`, and `1/1` under Wasm-AOT
  as of `2026-07-22`, with every failure bucket and timeout count at zero
  (manifests `8556036983037034978`, `14837431350955281706`,
  `7264627580614754722`, and `17043656959946077867`). The three iterator
  methods are distinct non-generic `%TypedArray%` intrinsics, `@@iterator`
  aliases `values` by function identity, and iterator steps observe current
  values and current resizable-buffer lengths while rejecting detached or
  out-of-bounds views. Exhausted iterators remain done after later buffer
  growth or shrinkage. Refresh a leaf by replacing `<method>` in
  `./target/release/lila --jobs 1 test262 run built-ins/TypedArray/prototype/<method> --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 1 --timeout-ms 60000 --snapshot-name typedarray-prototype-<method>-current-pin`.
- The `%TypedArray%.prototype` leaves `find`, `findIndex`, `findLast`, and
  `findLastIndex` each report `38/38` under Wasm-AOT as of `2026-07-22`, with
  every failure bucket and timeout count at zero (manifests
  `3286424623423387775`, `8463525698580930132`, `17063345085000905537`, and
  `12976369256671968300`). They are distinct non-generic `%TypedArray%`
  intrinsics that validate the receiver before the predicate, snapshot the
  iteration length, and read current element values after detach, growth, or
  shrinkage. The resizable-buffer cases use static Wasm-AOT materializations
  rather than dynamic source evaluation. Refresh a leaf by replacing
  `<method>` in
  `./target/release/lila --jobs 1 test262 run built-ins/TypedArray/prototype/<method> --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 1 --timeout-ms 60000 --snapshot-name typedarray-prototype-<method>-current-pin`.
- The `%TypedArray%.prototype.every` leaf reports `44/44` under Wasm-AOT as of
  `2026-07-22`, with every failure bucket and timeout count at zero (manifest
  `10128406413910089111`). It is a distinct non-generic `%TypedArray%`
  intrinsic that validates the receiver before the predicate, snapshots the
  iteration length, and reads current element values after detach, growth, or
  shrinkage. Its six resizable-buffer roots use static Wasm-AOT
  materializations rather than dynamic source evaluation. Refresh it with
  `./target/release/lila --jobs 1 test262 run built-ins/TypedArray/prototype/every --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 1 --timeout-ms 120000 --snapshot-name typedarray-prototype-every-current-pin-final`.
- The nine exact pinned real-Test262 numeric concrete-constructor leaves
  (`Int8Array`, `Int16Array`, `Int32Array`, `Uint8Array`,
  `Uint8ClampedArray`, `Uint16Array`, `Uint32Array`, `Float32Array`, and
  `Float64Array`) each report `11/11`, or `99/99` selected roots together,
  under Wasm-AOT as of `2026-07-20` at Test262 revision
  `aa55200d1310384c5cf69ea95b2a2ecba457007b`, with no unsupported cases,
  bugs, or crashes. They cover constructor identity and constructability,
  name/length/prototype descriptors, `%TypedArray%` constructor and prototype
  inheritance, immutable `BYTES_PER_ELEMENT` values, and rejection of the
  unbranded prototype object by the `buffer` accessor. Refresh any listed leaf
  with
  `./target/release/lila --jobs 1 test262 run built-ins/TypedArrayConstructors/<Constructor> --execution-backend wasm-aot --threads 1 --timeout-ms 60000 --snapshot-name typedarray-<lowercase-constructor>-current-tree-20260720`.
  All ninety-nine selected vendored test bodies run without static case
  rewrites; their declared helper preludes come from the Wasm-AOT local merged
  harness rather than byte-for-byte vendored harness files. The upstream
  `Float32Array/prototype/not-typedarray-object.js` root is mislabeled and
  actually asserts `Float64Array.prototype.buffer`; the reported `11/11` is
  the exact selected Float32 directory count, not eleven Float32-specific
  assertions.
- The hidden `%TypedArray%` constructor now has one dedicated compiler builtin
  identity in every Realm. It is not a global binding; its native name is
  `TypedArray`, its length is zero, and its own `prototype` descriptor is
  non-writable, non-enumerable and non-configurable. The value is constructable
  so it can serve as `newTarget`, while calling it or constructing it directly
  throws its defining Realm's `TypeError`. Concrete typed-array constructors
  inherit from the matching Realm-local identity, and lowering retains that
  exact target through `Object.getPrototypeOf`. As of `2026-08-31`, the exact
  pinned `built-ins/TypedArray/{name,length,invoked,prototype}`,
  `built-ins/TypedArray/prototype/constructor` and
  `built-ins/TypedArrayConstructors/Uint8Array/proto` leaves pass both variants
  (`12/12`) under Wasm-AOT with every non-success bucket at zero. Refresh one
  leaf with `./target/debug/lila --jobs 1 test262 run <leaf> --suite-root
  test262/vendor/test262 --execution-backend wasm-aot --threads 1
  --timeout-ms 60000 --snapshot-name typedarray-identity-<leaf>`.
- The exact pinned real-Test262 `BigInt64Array` and `BigUint64Array` concrete
  leaves each report `12/12`, or `24/24` together, under Wasm-AOT as of
  `2026-07-20` at the same Test262 revision, with every failure bucket at zero.
  Refresh them with
  `./target/release/lila --jobs 1 test262 run built-ins/TypedArrayConstructors/BigInt64Array --execution-backend wasm-aot --threads 1 --timeout-ms 60000 --snapshot-name typedarray-bigint64array-vendored-current-tree-20260720`
  and
  `./target/release/lila --jobs 1 test262 run built-ins/TypedArrayConstructors/BigUint64Array --execution-backend wasm-aot --threads 1 --timeout-ms 60000 --snapshot-name typedarray-biguint64array-vendored-current-tree-20260720`.
  All twenty-four selected vendored test bodies now run through the normal
  materializer without static case rewrites. Their declared helper preludes
  come from the Wasm-AOT local merged harness rather than byte-for-byte
  vendored harness files. In particular, the merged `propertyHelper.js`
  verifies the requested descriptor fields but does not yet perform every
  operational writable, enumerable, and configurable check made by the
  upstream helper. Together, the exact selected concrete-constructor family
  reports `123/123` vendored bodies with local merged helpers at this pin.
  Wasm-AOT now exposes `%TypedArray%.prototype.buffer` as a real accessor,
  preserves typed-array receiver validation for the BigInt prototypes, and
  emits non-writable, non-enumerable, non-configurable
  `BYTES_PER_ELEMENT` descriptors on the constructors and their prototypes.
- The exact pinned real-Test262
  `built-ins/TypedArrayConstructors/ctors/no-species.js` file reports `1/1`
  under Wasm-AOT as of `2026-07-20` at the same Test262 revision, with every
  failure bucket at zero (manifest `2653835606409241992`). It runs the literal
  vendored body through the normal materializer, verifies that cloning a typed
  array does not reconstruct its ArrayBuffer subclass or consult its poisoned
  `Symbol.species` getter, and observes an ordinary `ArrayBuffer.prototype` on
  the clone through the standard `Object.prototype.__proto__` accessor. Refresh
  it with
  `./target/release/lila --jobs 1 test262 run built-ins/TypedArrayConstructors/ctors/no-species.js --execution-backend wasm-aot --threads 1 --timeout-ms 60000 --snapshot-name typedarray-ctors-no-species-vendored-current-tree-20260720`.
  The body is byte-for-byte vendored; its `Test262Error` and `assert.sameValue`
  support comes from the local merged Wasm-AOT harness rather than byte-for-byte
  upstream harness preludes.
- TypedArray integer-indexed `[[OwnPropertyKeys]]` now synthesizes the live
  in-bounds index range, then preserves ordinary string insertion order and
  appends symbols. The exact pinned
  `built-ins/TypedArrayConstructors/internals/OwnPropertyKeys` directory reports
  `10/10` under Wasm-AOT as of `2026-07-20` at Test262 revision
  `aa55200d1310384c5cf69ea95b2a2ecba457007b`, with no dynamic-source
  exclusions. This covers numeric and BigInt views plus fixed and
  length-tracking resizable views before and after grow, shrink, and
  out-of-bounds transitions. Refresh it with
  `./target/release/lila --jobs 1 test262 run built-ins/TypedArrayConstructors/internals/OwnPropertyKeys --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 1 --timeout-ms 60000 --snapshot-name typedarray-own-property-keys-current-tree-20260720`.
- `%TypedArray%.prototype.subarray` preserves coercion ordering, snapshots the
  current view length, performs observable constructor and `Symbol.species`
  selection, shares the source backing buffer, and validates the returned typed
  array and Number/BigInt content type. A species-returned detached or
  out-of-bounds view is rejected in the executing builtin's Realm, including
  when a created-Realm method is borrowed. Omitting `end` from a length-tracking
  source selects a two-argument species construction and a length-tracking
  result. A focused arity checkpoint corrected one product-path mismatch: the
  pre-fix branch reduced the callee count to two after allocating a three-entry
  argv object, so an escaped species `arguments` object observed a phantom
  third entry. At Test262 content tree
  `aa55200d1310384c5cf69ea95b2a2ecba457007b`, the exact Number and BigInt
  custom-species invocation files move from `0/4` pre-fix sloppy/strict
  variants to `4/4` post-fix: the Number and BigInt leaves pass `2/2` each with
  every non-success bucket at zero. The bounded structure target passes `4/4`
  and the extended exact subarray CLI fixture passes `1/1`. The correction
  selects a coherent two- or three-entry vector before the shared construct; it
  does not claim general arguments-object changes or close the separate
  nullish-species default-constructor Realm debt.
- `TypedArray.from` snapshots generic iterable values before target construction
  and conversion, supports Proxy iterator methods, `next`, mappers, and
  constructors, and applies the array-like `ToLength` and mapper ordering rules.
  `TypedArray.of` constructs through the generic receiver before ordered element
  conversion. Both validate the constructed target's internal typed-array brand
  and current view rather than trusting forged public properties. Their complete
  current-pin leaves report `21/21` and `8/8` under Wasm-AOT on `2026-07-21`,
  with every failure bucket and timeout count at zero (manifests
  `14485063322838869338` and `18238188842051720004`). Refresh either by
  replacing `<method>` with `from` or `of` in
  `./target/release/lila --jobs 1 test262 run built-ins/TypedArray/<method> --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 1 --timeout-ms 60000 --snapshot-name typedarray-<method>-current-pin`.
- TypedArray integer-indexed `[[GetOwnProperty]]` now distinguishes canonical
  numeric index strings from ordinary string and symbol properties, suppresses
  invalid, detached, and out-of-bounds indices without walking the prototype
  chain, and returns the required writable, enumerable, configurable data
  descriptor for live elements. Synthetic realms install their own
  `%TypedArray%.prototype.buffer` accessor, so cross-realm views use the same
  private backing-buffer state. The complete pinned
  `built-ins/TypedArrayConstructors/internals/GetOwnProperty` directory reports
  `24/24` under Wasm-AOT as of `2026-07-20` at Test262 revision
  `aa55200d1310384c5cf69ea95b2a2ecba457007b`, with no dynamic-source
  exclusions. Refresh it with
  `./target/release/lila --jobs 1 test262 run built-ins/TypedArrayConstructors/internals/GetOwnProperty --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 1 --timeout-ms 60000 --snapshot-name typedarray-get-own-property-current-tree-20260720`.
- TypedArray integer-indexed `[[Get]]` now routes canonical numeric strings
  through integer-index validation before any ordinary prototype lookup, so
  fractional, negative-zero, negative, infinity, detached, and out-of-bounds
  keys return `undefined` without exposing inherited accessors. Noncanonical
  strings and symbols retain ordinary property access. The complete pinned
  `built-ins/TypedArrayConstructors/internals/Get` directory reports `28/28`
  under Wasm-AOT as of `2026-07-21` at Test262 revision
  `aa55200d1310384c5cf69ea95b2a2ecba457007b`, with every failure bucket at zero
  (manifest `7640730389657240498`). Refresh it with
  `./target/release/lila --jobs 1 test262 run built-ins/TypedArrayConstructors/internals/Get --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 4 --timeout-ms 60000 --snapshot-name typedarray-get-post-prototype-set-complete-28`.
- TypedArray integer-indexed `[[HasProperty]]` now handles live, detached,
  out-of-bounds, resizable, ordinary, symbol, and inherited Proxy paths without
  leaking canonical numeric keys into the prototype chain. Dynamic `with`
  lookup preserves outer object and function bindings, and `%TypedArray%`
  obtained through `Object.getPrototypeOf(Int8Array)` retains its Function tag.
  The complete pinned
  `built-ins/TypedArrayConstructors/internals/HasProperty` directory reports
  `32/32` under Wasm-AOT as of `2026-07-20` at Test262 revision
  `aa55200d1310384c5cf69ea95b2a2ecba457007b`, with no dynamic-source
  exclusions and every failure bucket at zero (manifest
  `4378182180659179029`). Refresh it with
  `./target/release/lila --jobs 1 test262 run built-ins/TypedArrayConstructors/internals/HasProperty --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 4 --timeout-ms 60000 --snapshot-name typedarray-has-property-final-current-tree-20260720`.
- TypedArray integer-indexed `[[DefineOwnProperty]]` now distinguishes canonical
  numeric indices from ordinary string and symbol properties, rejects invalid
  integer-index descriptors, and performs the required numeric or BigInt
  element conversion before storing. `Object.defineProperty` throws on a false
  result while `Reflect.defineProperty` reports it, and synthetic-realm
  TypedArray constructors retain their element kind and bytes-per-element
  metadata. The complete pinned
  `built-ins/TypedArrayConstructors/internals/DefineOwnProperty` directory
  reports `54/54` under Wasm-AOT as of `2026-07-21` at Test262 revision
  `aa55200d1310384c5cf69ea95b2a2ecba457007b`, with no dynamic-source
  exclusions and every failure bucket at zero (manifest
  `7031645897862764810`). This includes numeric and BigInt views, NaN
  conversion consistency, detached and resizable buffers, cross-realm errors,
  accessor rejection, and ordinary non-index properties. Refresh it with
  `./target/release/lila --jobs 1 test262 run built-ins/TypedArrayConstructors/internals/DefineOwnProperty --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 4 --timeout-ms 60000 --snapshot-name typedarray-define-own-property-final-current-tree-20260721`.
- TypedArray integer-indexed `[[Set]]` now classifies canonical numeric keys at
  the shared object-write boundary, preserves conversion ordering, implements
  altered-receiver `Reflect.set` and inherited TypedArray prototype behavior,
  and initializes the numeric payload before the small-index string fast path.
  The complete pinned
  `built-ins/TypedArrayConstructors/internals/Set` directory reports `53/53`
  under Wasm-AOT as of
  `2026-07-21` at Test262 revision
  `aa55200d1310384c5cf69ea95b2a2ecba457007b`, with every failure bucket and
  timeout count at zero (manifest `10971918057948772961`). It covers canonical invalid, valid,
  fractional, negative-zero, out-of-bounds, noncanonical, and symbol keys;
  strict writes; ordinary and TypedArray receivers; prototype-chain writes;
  and `Reflect.set`. The added number cohort covers numeric conversion and NaN
  consistency, stored values, conversion throws, detached buffers and realms,
  and a resizable view returning in bounds, plus BigInt key, conversion,
  detached-buffer, and value-exception semantics. All 53 roots are
  AOT-applicable and the directory contains no dynamic-source cases. Refresh it
  with
  `./target/release/lila --jobs 1 test262 run built-ins/TypedArrayConstructors/internals/Set --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 4 --timeout-ms 60000 --snapshot-name typedarray-set-complete-53-final`.
- Direct TypedArray `length`, indexed reads and indexed writes now derive their
  live bounds through the shared backing-buffer witness. Length-tracking views
  expose only complete elements, including odd-byte Uint16 backing extents.
  Indexed writes still coerce the incoming value before observing resize or
  detachment, then acquire the usable backing pointer only after the fresh
  witness accepts the index; a resizing `valueOf` cannot leave a stale pointer
  or byte-level partial-element bound behind.
- `%TypedArray%.prototype.set` now copies array-like and TypedArray sources in
  observable order, snapshots typed sources for overlap safety, performs
  numeric or BigInt conversion, validates offsets and content types, handles
  shared and resizable buffers, and revalidates the target after offset
  coercion can detach or resize it. The complete pinned
  `built-ins/TypedArray/prototype/set` directory reports `109/109` under
  Wasm-AOT as of `2026-07-21` at the same Test262 revision, with every failure
  bucket at zero (manifest `8330323270441760429`). Refresh it with
  `./target/release/lila --jobs 1 test262 run built-ins/TypedArray/prototype/set --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 4 --timeout-ms 60000 --snapshot-name typedarray-prototype-set-post-validation-109`.
- `%TypedArray%.prototype.toReversed` now validates the current view, captures
  the current fixed or tracking length, allocates the same intrinsic TypedArray
  kind without constructor or species lookup, reverse-copies numeric or BigInt
  values, and leaves the source unchanged. The historical pinned leaf reports
  `9/9` under Wasm-AOT as of `2026-07-21` at the same Test262 revision, with
  every failure bucket at zero (manifest `12517032484477954620`). The current
  working tree removes the former invalid-receiver replacement and the
  family-specific helper split; all nine physical sources now pass in both
  Script modes from unchanged bodies and the full upstream helper (`18/18`).
  Refresh the leaf with
  `./target/release/lila --jobs 1 test262 run built-ins/TypedArray/prototype/toReversed --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 4 --timeout-ms 60000 --snapshot-name typedarray-prototype-to-reversed-complete-9`.
- `%TypedArray%.prototype.reverse` now validates and captures the current view
  length, swaps typed elements in place for every numeric and BigInt kind, and
  returns the original receiver. The complete pinned leaf reports `21/21`
  under Wasm-AOT as of `2026-07-21` at the same Test262 revision, including
  shared, detached, fixed, resizable, and length-tracking views, with every
  failure bucket at zero (manifest `3206610524287450104`). Refresh it with
  `./target/release/lila --jobs 1 test262 run built-ins/TypedArray/prototype/reverse --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 4 --timeout-ms 60000 --snapshot-name typedarray-prototype-reverse-complete-21`.
- `%TypedArray%.prototype.with` now applies `ToIntegerOrInfinity` relative-index
  normalization and replacement conversion in spec order, revalidates the view
  after user coercion can detach or resize it, allocates the same intrinsic kind
  without species lookup, and copies without mutating the source. The historical
  pinned leaf reports `22/22` under Wasm-AOT as of `2026-07-21` at the same
  Test262 revision, with every failure bucket at zero (manifest
  `4222886790829078659`). The current working tree removes the family-specific
  helper split; all 22 physical sources now pass in both Script modes from
  unchanged bodies and the full upstream helper (`44/44`). Refresh it with
  `./target/release/lila --jobs 1 test262 run built-ins/TypedArray/prototype/with --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 4 --timeout-ms 60000 --snapshot-name typedarray-prototype-with-complete-22`.
- `%TypedArray%.prototype.toSorted` now makes a same-kind copy before sorting,
  ignores constructor/species and public length properties, applies stable
  numeric or BigInt default ordering with NaN and signed-zero rules, and invokes
  callable comparators with `ToNumber` and abrupt completion propagation. The
  historical pinned leaf reports `12/12` under Wasm-AOT as of `2026-07-21` at
  the same Test262 revision, with every failure bucket at zero (manifest
  `13233608438829661408`). The current working tree also removes the former
  invalid-receiver replacement and family-specific helper split; all twelve
  physical sources pass in both Script modes from unchanged bodies and the full
  upstream helper (`24/24`). Refresh it with
  `./target/release/lila --jobs 1 test262 run built-ins/TypedArray/prototype/toSorted --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 4 --timeout-ms 60000 --snapshot-name typedarray-prototype-to-sorted-complete-12`.
- `%TypedArray%.prototype.sort` reuses the stable comparison core, writes the
  ordered values back in place, returns the receiver, and stops after comparator
  coercion if user code detaches the target. The complete pinned leaf reports
  `35/35` under Wasm-AOT as of `2026-07-21` at the same Test262 revision,
  including numeric/BigInt defaults, custom comparators, shared and resizable
  views, with every failure bucket at zero (manifest
  `11175084542474034069`). Refresh it with
  `./target/release/lila --jobs 1 test262 run built-ins/TypedArray/prototype/sort --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 4 --timeout-ms 60000 --snapshot-name typedarray-prototype-sort-complete-35`.
  Together, the seven pinned `TypedArrayConstructors/internals` directories now
  report `240/240` exact-green AOT-applicable roots with no dynamic-source
  exclusions.
- TypedArray integer-indexed `[[Delete]]` now rejects deletion of valid indices,
  lets strict delete convert that rejection to `TypeError`, and returns true for
  invalid canonical indices without creating or deleting ordinary properties.
  Noncanonical strings and symbols retain ordinary deletion semantics, while
  Proxy `deleteProperty` remains trap-first. The complete pinned
  `built-ins/TypedArrayConstructors/internals/Delete` directory reports `39/39`
  under Wasm-AOT as of `2026-07-20` at the same Test262 revision, with no
  dynamic-source exclusions and every failure bucket at zero (manifest
  `16709326299855855162`). This includes numeric and BigInt ArrayBuffer,
  SharedArrayBuffer, detached, cross-realm, strict, and non-strict cases.
  Refresh it with
  `./target/release/lila --jobs 1 test262 run built-ins/TypedArrayConstructors/internals/Delete --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 4 --timeout-ms 60000 --snapshot-name typedarray-delete-current-tree-20260720`.
- The exact real Test262
  `Array.prototype.every`/`filter`/`some`/`values`/`keys`/`entries`
  `resizable-buffer-grow-mid-iteration.js` and
  `resizable-buffer-shrink-mid-iteration.js` files now report `1/1` each as
  of `2026-06-18` under `--execution-backend wasm` with the `60000` ms timeout
  and one thread. The callback and `values` cases retain their self-contained
  materializations. The `values` grow file checks a
  length-tracking iterator across resize, including a newly exposed zero-filled
  element, and the `values` shrink file checks fixed-length out-of-bounds
  `TypeError` plus length-tracking iterator exhaustion after shrink. The four
  `keys` and `entries` mid-iteration files now execute their unchanged pinned
  bodies with the complete constructor and fixed/tracking/offset view matrices;
  all eight sloppy and strict executions pass. The unchanged entries
  `resizable-buffer.js` body adds the full shrink-to-zero and regrow matrix and
  passes `2/2`. Ordinary materialization retains only the T13 static-subclass
  adaptation inside the vendored resizable-buffer helper.
- Array prototype method metadata now runs the exact real Test262 `length.js`,
  `name.js`, and `prop-desc.js` sources unchanged for `at`, `every`, `filter`,
  `find`, `findIndex`, `findLast`, `findLastIndex`, `flat`, `flatMap`, `forEach`,
  `includes`, `indexOf`, `lastIndexOf`, `map`, `some`, and `toString`. All 48
  sources pass with the complete upstream `propertyHelper.js` in sloppy and
  strict modes (`96/96`). A real-source invariant pins every source body, the
  supported-feature boundary, exact LocalMerged assertion/property helper
  bytes and provenance, and a separate complete VendoredHarness property-helper
  route. The former metadata dispatcher and three path predicates are gone.
- Proxy-backed generic `Array.prototype.includes` calls preserve string
  property keys through `get` traps, so proxy array-like receivers observe
  `length`/indexed reads in order and hit cases stop at the matched element.
- Fresh ordinary `Symbol()` values carry runtime identity in Wasm-AOT, so
  `Array.prototype.includes` symbol misses no longer collapse separate symbols
  with matching descriptions.
- The full `built-ins/Array/prototype/includes` leaf now reports `30/30`
  passing as of `2026-06-18` under `--execution-backend wasm` with the `60000`
  ms timeout and four threads (`0` unsupported, `0` runtime failures) with
  `./target/debug/lila test262 run built-ins/Array/prototype/includes --execution-backend wasm --timeout-ms 60000 --threads 4`.
  The `length`, `name`, and `prop-desc` descriptor cases now use direct
  `Object.getOwnPropertyDescriptor` materializations, and the helper-heavy
  resizable ArrayBuffer includes cases use self-contained Wasm-AOT sources that
  keep direct fixed-length, length-tracking, resize, `fromIndex`, and special
  float `SameValueZero` checks without invoking the dynamic subclass helper.
  The local `crates/lila-cli/tests/fixtures/wasm_array_includes_resizable_typedarray.js`
  fixture also covers the descriptor metadata.
- Annex B catch-parameter/`var` redeclaration now keeps the catch parameter
  binding distinct from the outer/global binding in Wasm-AOT, including closure
  captures of the outer binding after the catch block.
- Annex B single-statement function declarations use the parser's sloppy-mode
  block rewrite and copy the selected block binding into the synthesized owner
  binding. Script-created `var` and function properties are writable,
  enumerable, and non-configurable, and the `fnGlobalObject.js` harness obtains
  the existing global through `globalThis` without dynamic source generation.
  The complete exact `annexB/language/function-code/if-` prefix reports `95/95`
  as of `2026-07-16`. The complete `annexB/language/global-code/if-` prefix
  reports `85/95`; all ten remaining cases require `$262.evalScript`, so the
  AOT-applicable subset reports `85/85`. The corresponding function-code
  `block-decl-` and `switch-` prefixes report `22/22` and `40/40`. Their
  global-code prefixes report `17/19` and `34/38`; the six remaining cases all
  require `$262.evalScript`, so those AOT-applicable subsets report `17/17` and
  `34/34`. Arguments objects now resolve their inherited `toString` method,
  covering the legacy function declaration named `arguments` case. Together
  with both function redeclaration cases, the complete function-code directory
  is `159/159`; the complete global-code directory is `136/153`, with all 17
  remaining cases classified up front as `$262.evalScript` dynamic source, so
  its AOT-applicable subset is `136/136`. The Annex B language statements
  directory reports `13/22`; all nine remaining cases require the
  `$262.IsHTMLDDA` host object, so its AOT-applicable subset is `13/13`.
  Annex B comments and literals report `8/8` each. Annex B expressions report
  `9/26`; all 17 remaining cases require `$262.IsHTMLDDA`, so the
  AOT-applicable subset is `9/9`. Annex B Date, `escape`, and `unescape` report
  `24/24`, `16/16`, and `19/19`. The one TypedArray constructor case also
  passes; the Array and Object cases require `$262.IsHTMLDDA`, while all six
  Function cases require dynamic Function-constructor source generation, so
  none of those eight cases are AOT-applicable. Annex B RegExp reports `60/62`;
  its two remaining cases use `eval`, so its AOT-applicable subset is `60/60`.
  This includes incomplete non-Unicode `\u` identity escapes, literal
  lookbehind bodies, and `Symbol.match` getter side effects that recompile a
  RegExp while constructing the split matcher.
  Annex B String reports `105/111`; the six remaining cases require
  `$262.IsHTMLDDA`, so its AOT-applicable subset is `105/105`. Across the
  complete 241-case Annex B built-ins directory, all 225 AOT-applicable cases
  pass and the other 16 require `eval`, Function-constructor source generation,
  or the `$262.IsHTMLDDA` host object.
  The 469-case Annex B eval-code directory is entirely dynamic `eval` source
  and is classified up front as unsupported for Wasm AOT. Across the complete
  1,086-case Annex B tree, all 558 AOT-applicable cases pass; the remaining 528
  cases require `eval`, `$262.evalScript`, Function-constructor source
  generation, or the `$262.IsHTMLDDA` host object.
  The ordinary `built-ins/RegExp/prototype/Symbol.split` leaf reports `43/44`;
  its only remaining case creates cross-realm source with a Function
  constructor, so its AOT-applicable subset is `43/43`.
- Ordinary function declarations now resolve their mutable surrounding binding
  during recursion instead of creating a new self object per call. Explicitly
  named function expressions use a private, per-evaluation name environment
  backpatched to the exact allocated function object, while inferred function
  names continue to resolve their surrounding binding. Function identity,
  expandos, reassignment, outer captures, and nested self captures therefore
  remain observable through the Wasm-AOT path.
- Global `Infinity`, `NaN`, and `undefined` are installed as non-enumerable,
  non-configurable read-only data properties in Wasm-AOT; sloppy writes are
  ignored, and strict writes to non-writable object data properties throw. The
  `propertyHelper.js` descriptor checks for these global constants now use a
  static Wasm-AOT materialization that preserves the
  `Object.getOwnPropertyDescriptor(this, name)` flag assertions without timing
  out in the generic helper. The full `built-ins/Infinity` and `built-ins/NaN`
  leaves now report `6/6` passing as of `2026-06-04`, and
  `built-ins/undefined` now reports `8/8` passing as of `2026-06-19` under
  `--execution-backend wasm` with the `60000` ms timeout and four threads
  (`0` unsupported, `0` runtime failures). The legacy
  `S15.1.1.3_A1.js` `eval("var x")` check uses a source-free static
  materialization of the known `undefined` var-declaration result while generic
  dynamic `eval` stays unsupported:
  `./target/debug/lila test262 run built-ins/Infinity --execution-backend wasm --timeout-ms 60000 --threads 4`,
  `./target/debug/lila test262 run built-ins/NaN --execution-backend wasm --timeout-ms 60000 --threads 4`,
  and
  `./target/debug/lila test262 run built-ins/undefined --execution-backend wasm --timeout-ms 60000 --threads 4`.
- Exact pinned Wasm-AOT URI-codec runs report `encodeURI` at `31/31`,
  `encodeURIComponent` at `31/31`, `decodeURI` at `54/55`, and
  `decodeURIComponent` at `55/56` as of `2026-07-29`, for `171/173` combined.
  The sole `Crash:Runtime` cases are
  `built-ins/decodeURI/S15.1.3.1_A2.5_T1.js` and
  `built-ins/decodeURIComponent/S15.1.3.2_A2.5_T1.js`. Both exhaust wasm32
  memory in exhaustive million-iteration RFC-3629 checks under the
  non-reclaiming bump heap; they are not codec semantic mismatches. The safe
  resolution is heap reclamation or GC, with no codec-local workaround.
  The snapshots are `uri-encodeuri-current-20260729`,
  `uri-encodeuricomponent-current-20260729`,
  `uri-decodeuri-current-20260729`, and
  `uri-decodeuricomponent-current-20260729`. Refresh one with
  `./target/debug/lila --jobs 1 test262 run built-ins/<codec> --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 1 --timeout-ms 120000 --snapshot-name <snapshot>`.
- Number constructor constants and parse aliases now avoid the slow
  `propertyHelper.js` descriptor path while still checking direct descriptors,
  read-only/non-configurable behavior, and global alias identity. The exact
  real Test262 leaves `built-ins/Number/MAX_VALUE`,
  `built-ins/Number/MIN_VALUE`, `built-ins/Number/POSITIVE_INFINITY`,
  `built-ins/Number/NEGATIVE_INFINITY`, `built-ins/Number/parseFloat`, and
  `built-ins/Number/parseInt` now report `3/3`, `3/3`, `4/4`, `4/4`, `2/2`,
  and `2/2` passing respectively as of `2026-06-18` under
  `--execution-backend wasm` with the `60000` ms timeout and four threads:
  `./target/debug/lila test262 run built-ins/Number/MAX_VALUE --execution-backend wasm --timeout-ms 60000 --threads 4`,
  `./target/debug/lila test262 run built-ins/Number/MIN_VALUE --execution-backend wasm --timeout-ms 60000 --threads 4`,
  `./target/debug/lila test262 run built-ins/Number/POSITIVE_INFINITY --execution-backend wasm --timeout-ms 60000 --threads 4`,
  `./target/debug/lila test262 run built-ins/Number/NEGATIVE_INFINITY --execution-backend wasm --timeout-ms 60000 --threads 4`,
  `./target/debug/lila test262 run built-ins/Number/parseFloat --execution-backend wasm --timeout-ms 60000 --threads 4`,
  and
  `./target/debug/lila test262 run built-ins/Number/parseInt --execution-backend wasm --timeout-ms 60000 --threads 4`.
- Additional Number constructor metadata leaves now use direct descriptor
  materializations for wasm-AOT instead of timing out in `propertyHelper.js`.
  The exact real Test262 files `built-ins/Number/EPSILON.js`,
  `built-ins/Number/MAX_SAFE_INTEGER.js`,
  `built-ins/Number/MIN_SAFE_INTEGER.js`, `built-ins/Number/NaN.js`,
  `built-ins/Number/prop-desc.js`,
  `built-ins/Number/prototype/prop-desc.js`, and
  `built-ins/Number/prototype/constructor.js` now report `1/1` passing each as
  of `2026-06-18` under `--execution-backend wasm` with the `60000` ms timeout
  and four threads:
  `./target/debug/lila test262 run built-ins/Number/EPSILON.js --execution-backend wasm --timeout-ms 60000 --threads 4`,
  `./target/debug/lila test262 run built-ins/Number/MAX_SAFE_INTEGER.js --execution-backend wasm --timeout-ms 60000 --threads 4`,
  `./target/debug/lila test262 run built-ins/Number/MIN_SAFE_INTEGER.js --execution-backend wasm --timeout-ms 60000 --threads 4`,
  `./target/debug/lila test262 run built-ins/Number/NaN.js --execution-backend wasm --timeout-ms 60000 --threads 4`,
  `./target/debug/lila test262 run built-ins/Number/prop-desc.js --execution-backend wasm --timeout-ms 60000 --threads 4`,
  `./target/debug/lila test262 run built-ins/Number/prototype/prop-desc.js --execution-backend wasm --timeout-ms 60000 --threads 4`,
  and
  `./target/debug/lila test262 run built-ins/Number/prototype/constructor.js --execution-backend wasm --timeout-ms 60000 --threads 4`.
- `Number.prototype.valueOf` now reports `11/11` passing as of `2026-06-18`
  under `--execution-backend wasm` with the `60000` ms timeout and four
  threads:
  `./target/debug/lila test262 run built-ins/Number/prototype/valueOf --execution-backend wasm --timeout-ms 60000 --threads 4`.
  The `length`, `name`, and `prop-desc` metadata files now use direct
  descriptor materializations for wasm-AOT instead of timing out in
  `propertyHelper.js`, while the existing primitive and boxed-number receiver
  behavior files continue to pass through the normal builtin path.
- `Number.prototype.toLocaleString` now reports `4/4` passing as of
  `2026-06-18` under `--execution-backend wasm` with the `60000` ms timeout
  and four threads:
  `./target/debug/lila test262 run built-ins/Number/prototype/toLocaleString --execution-backend wasm --timeout-ms 60000 --threads 4`.
  Its `length`, `name`, and `prop-desc` metadata files share the same direct
  descriptor materialization path as `Number.prototype.valueOf`, avoiding the
  slow `propertyHelper.js` route while preserving the direct descriptor flag
  checks.
- `Number.prototype.toFixed`, `Number.prototype.toExponential`,
  `Number.prototype.toPrecision`, and `Number.prototype.toString` now report
  `16/16`, `15/15`, `17/17`, and `90/90` passing respectively as of
  `2026-06-18` under `--execution-backend wasm`:
  `./target/debug/lila test262 run built-ins/Number/prototype/toFixed --execution-backend wasm --timeout-ms 60000 --threads 4`,
  `./target/debug/lila test262 run built-ins/Number/prototype/toExponential --execution-backend wasm --timeout-ms 60000 --threads 4`,
  `./target/debug/lila test262 run built-ins/Number/prototype/toPrecision --execution-backend wasm --timeout-ms 60000 --threads 4`,
  and
  `./target/debug/lila test262 run built-ins/Number/prototype/toString --execution-backend wasm --timeout-ms 120000 --threads 12`.
  Their `length`, `name`, and `prop-desc` metadata files now use the shared
  direct descriptor materialization path. The larger `Number.prototype.toString`
  leaf needs the wider per-file timeout in the command above because the
  `numeric-literal-tostring-radix-1.js` RangeError case passed individually
  under `60000` ms but timed out once during a high-concurrency full-leaf run
  at that tighter timeout.
- The full `built-ins/Number/prototype` shard now reports `168/168` passing as
  of `2026-06-19` under `--execution-backend wasm` with the `120000` ms timeout
  and twelve threads (`0` unsupported, `0` runtime failures):
  `./target/debug/lila test262 run built-ins/Number/prototype --execution-backend wasm --timeout-ms 120000 --threads 12`.
  This aggregates the top-level Number prototype descriptor/value files plus
  the `valueOf`, `toLocaleString`, `toFixed`, `toExponential`, `toPrecision`,
  and `toString` method subleaves.
- The complete current-pin `built-ins/Number` tree reports `337/338` under
  Wasm-AOT as of `2026-07-22`; all `337/337` AOT-applicable roots pass, with
  zero parser, early-error, lowering, runtime, Wasm-backend, host-harness,
  crash, or bug outcomes (manifest `12919822299029746592`). The explicit
  unsupported `built-ins/Number/proto-from-ctor-realm.js` root executes
  zero-argument cross-realm `new other.Function()` and is no longer replaced
  by a Proxy newTarget. Refresh Test262 revision
  `aa55200d1310384c5cf69ea95b2a2ecba457007b` with
  `./target/release/lila --jobs 1 test262 run built-ins/Number --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 1 --timeout-ms 120000 --snapshot-name number-current-pin-final-20260722`.
  Wasm-AOT still exposes `Number` from synthetic realms, carries a realm-local
  `%Number.prototype%` slot for boxed primitive construction, and observes
  source-free custom `newTarget.prototype` behavior in `Reflect.construct`.
- The full `built-ins/Boolean` shard reports `99/101` passing as of
  `2026-08-25` under `--execution-backend wasm-aot` with the `120000` ms
  timeout and four threads (`0` unsupported, `2` runtime failures):
  `cargo run -p lila-cli -- test262 run built-ins/Boolean --suite-root test262/vendor/test262 --execution-backend wasm-aot --timeout-ms 120000 --threads 4`.
  Boolean constructor and prototype method descriptor files execute their
  unchanged Test262 sources and full property-helper harness.
  The exact `built-ins/Boolean/proto-from-ctor-realm.js` file is covered by a
  scoped static rewrite of the zero-argument cross-realm newTarget shape, and
  `S9.2_A6_T1.js` executes unchanged and passes both variants;
  `S9.2_A1_T1.js` executes unchanged and its two variants expose the explicit
  dynamic `eval` debt as `NotImplemented/Runtime`. Wasm-AOT also carries a
  realm-local `%Boolean.prototype%`
  fallback for `Reflect.construct(Boolean, [], newTarget)` when
  `newTarget.prototype` is not an object.
- The full `built-ins/Number/isFinite`, `built-ins/Number/isInteger`,
  `built-ins/Number/isNaN`, and `built-ins/Number/isSafeInteger` leaves now
  report `8/8`, `9/9`, `7/7`, and `10/10` passing respectively as of
  `2026-06-18` under `--execution-backend wasm` with the `60000` ms timeout
  and four threads (`0` unsupported, `0` runtime failures) with:
  `./target/debug/lila test262 run built-ins/Number/isFinite --execution-backend wasm --timeout-ms 60000 --threads 4`,
  `./target/debug/lila test262 run built-ins/Number/isInteger --execution-backend wasm --timeout-ms 60000 --threads 4`,
  `./target/debug/lila test262 run built-ins/Number/isNaN --execution-backend wasm --timeout-ms 60000 --threads 4`,
  and
  `./target/debug/lila test262 run built-ins/Number/isSafeInteger --execution-backend wasm --timeout-ms 60000 --threads 4`.
  The IR literal folder now avoids folding potentially numeric runtime/global
  arguments such as global `NaN` to `false`, so stored results like
  `let actual = Number.isNaN(NaN)` preserve the builtin call result. The
  `length`, `name`, and `prop-desc` metadata files for these Number predicate
  methods now use direct `Object.getOwnPropertyDescriptor` materializations
  instead of timing out in `propertyHelper.js`.
- `Error.isError` descriptor, native-error recognition, and non-error-object
  tests now run their unchanged pinned sources with the applicable full
  descriptor helper and full LocalMerged `assert.js` for all eight sloppy
  and strict Wasm-AOT executions across four physical files, including all
  three pinned `SuppressedError` assertions. Other-realm Error object
  recognition now emits standard Error family constructor bodies when
  `__lilaCreateRealm()` is used, so
  `Error.isError(new other.EvalError())` and the sibling Error constructors do
  not hit deferred-builtin stubs. The newly unmaterialized
  `errors-other-realm.js` leaf passes both executions (`2/2`) as of `2026-08-30`,
  with every failure and non-success bucket at zero. The full
  `built-ins/Error/isError` subleaf now reports `11/12` passing as of
  `2026-06-15` under
  `--execution-backend wasm` with the `60000` ms timeout and four threads; the
  only remaining unsupported file is
  `built-ins/Error/isError/non-error-objects-other-realm.js`, which depends on
  dynamic `Function` constructor source generation.
- Top-level `Error` constructor property coverage now runs
  `message_property.js`, `cause_property.js`, `prop-desc.js`, and
  `instance-prototype.js` unchanged with the full `propertyHelper.js`; all
  eight sloppy and strict Wasm-AOT executions pass as of `2026-08-25`.
- `Error.prototype` descriptor coverage now keeps the `message`, `name`, and
  `constructor` `propertyHelper.js` checks self-contained while still executing
  direct `Object.getOwnPropertyDescriptor(Error.prototype, name)` assertions
  for value, writable, enumerable, and configurable flags. The
  `Error.prototype.toString` descriptor, `length`, and `name` metadata checks
  also run self-contained descriptor assertions. Exact real Test262 files
  `built-ins/Error/prototype/message/prop-desc.js`,
  `built-ins/Error/prototype/name/prop-desc.js`,
  `built-ins/Error/prototype/constructor/prop-desc.js`,
  `built-ins/Error/prototype/toString/prop-desc.js`,
  `built-ins/Error/prototype/toString/length.js`, and
  `built-ins/Error/prototype/toString/name.js` each report `1/1` passing as of
  `2026-06-15` under `--execution-backend wasm` with the `60000` ms timeout.
- `built-ins/Error/prototype/no-error-data.js`,
  `built-ins/Error/prototype/S15.11.3.1_A1_T1.js`,
  `built-ins/Error/prototype/S15.11.3.1_A2_T1.js`,
  `built-ins/Error/prototype/S15.11.3.1_A3_T1.js`,
  `built-ins/Error/prototype/S15.11.3.1_A4_T1.js`,
  `built-ins/Error/prototype/S15.11.4_A1.js`,
  `built-ins/Error/prototype/S15.11.4_A2.js`,
  `built-ins/Error/prototype/S15.11.4_A3.js`,
  `built-ins/Error/prototype/S15.11.4_A4.js`,
  `built-ins/Error/prototype/constructor/S15.11.4.1_A1_T2.js`,
  `built-ins/Error/prototype/toString/called-as-function.js`, and
  `built-ins/Error/prototype/toString/invalid-receiver.js` now run unchanged
  with their full harnesses for all twenty-four sloppy and strict Wasm-AOT
  executions. `Error.prototype.toString` now also propagates `ToPrimitive`
  TypeErrors for non-callable `message`/`name`
  conversion hooks instead of falling back to `"[object Object]"`. Exact real
  Test262 file
  `built-ins/Error/prototype/toString/tostring-message-throws-toprimitive.js`
  reports `1/1` passing as of `2026-06-15` under `--execution-backend wasm`
  with the `60000` ms timeout. The full
  `built-ins/Error/prototype` leaf now reports `30/30` passing as of
  `2026-06-15` under `--execution-backend wasm` with the `60000` ms timeout
  and four threads (`0` unsupported, `0` runtime failures) with
  `./target/debug/lila test262 run built-ins/Error/prototype --execution-backend wasm --timeout-ms 60000 --threads 4`.
- The full current-pin `built-ins/Error` leaf reports `56/58` passing as of
  `2026-07-22`; all `56/56` AOT-applicable roots pass, with zero parser,
  early-error, lowering, runtime, Wasm-backend, host-harness, crash, or bug
  outcomes. The two explicit unsupported roots are
  `built-ins/Error/isError/non-error-objects-other-realm.js`, which calls
  `new other.Function("")`, and `built-ins/Error/proto-from-ctor-realm.js`,
  which calls `new other.Function()`. Refresh Test262 revision
  `aa55200d1310384c5cf69ea95b2a2ecba457007b` with
  `./target/release/lila --jobs 1 test262 run built-ins/Error --suite-root test262/vendor/test262 --execution-backend wasm-aot --timeout-ms 120000 --threads 1 --snapshot-name error-current-pin-final-20260722`.
  Error construction still derives its default prototype from `newTarget` for
  source-free constructor shapes; Function-constructor source generation is
  tracked as outside the Wasm-AOT product path rather than replaced by a
  static Proxy surrogate.
- The complete current-pin `built-ins/NativeErrors` tree reports `88/94`
  passing under Wasm-AOT as of `2026-07-22`; all `88/88` AOT-applicable roots
  pass, with zero parser, early-error, lowering, runtime, Wasm-backend,
  host-harness, crash, or bug outcomes (manifest `13146929685012755363`). The
  EvalError, RangeError, ReferenceError, SyntaxError, TypeError, and URIError
  subleaves each have `14/14` applicable roots green plus one explicit
  `proto-from-ctor-realm.js` exclusion because it executes
  `new other.Function()`. Refresh Test262 revision
  `aa55200d1310384c5cf69ea95b2a2ecba457007b` with
  `./target/release/lila --jobs 1 test262 run built-ins/NativeErrors --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 1 --timeout-ms 120000 --snapshot-name nativeerrors-current-pin-final-20260722`.
  All seven metadata roots for each of the six families—constructor `length`,
  constructor `name`, global descriptor, constructor `prototype`, and prototype
  `constructor`, `message`, and `name`—now run unchanged with their full
  harnesses for all 84 sloppy and strict Wasm-AOT executions. The obsolete
  NativeError metadata rewriter is gone. Cross-realm Function construction is
  tracked as unsupported dynamic code generation,
  not replaced with a Proxy surrogate.
- `%ThrowTypeError%` is now emitted as a real Wasm-AOT intrinsic function
  object, shared by strict arguments `callee` descriptors and the restricted
  `Function.prototype.arguments`/`caller` accessors. The intrinsic has
  `Function.prototype` as its prototype, throws `TypeError` when called,
  exposes non-configurable `length`/`name` descriptors in spec order, and is
  non-extensible/frozen. The `length`, `name`, and property-order Test262 files
  use self-contained Wasm-AOT materializations that preserve the direct
  descriptor/order assertions without relying on generic helper or
  `Array.prototype.indexOf` support. The
  full current-pin `built-ins/ThrowTypeError` leaf reports `13/14` under
  Wasm-AOT as of `2026-07-22`; all `13/13` AOT-applicable roots pass, with zero
  parser, early-error, lowering, runtime, Wasm-backend, host-harness, crash, or
  bug outcomes (manifest `8076903577417753920`). The explicit unsupported
  `distinct-cross-realm.js` root constructs two source-bearing functions with
  `new other.Function(...)`; it is no longer replaced by a static local
  function. Refresh Test262 revision
  `aa55200d1310384c5cf69ea95b2a2ecba457007b` with
  `./target/release/lila --jobs 1 test262 run built-ins/ThrowTypeError --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 1 --timeout-ms 120000 --snapshot-name throwtypeerror-current-pin-final-20260722`.
- Array index descriptors defined through `Object.defineProperty` recognize
  general canonical decimal index keys, so sparse accessor indexes such as
  `"10"` update array length and are visited by Array iteration methods.
- Sparse array numeric writes such as `let a = [1]; a[2] = 4; a[2];` now
  validate as Wasm-AOT modules after the numeric-index string-key fallback
  stopped emitting unmatched structured-control `end` operators. The focused
  Rust AOT library suite now includes this regression and reports `30/30`
  passing as of `2026-06-18` with `cargo test -p lila-aot-wasm --lib`.
- `Object.preventExtensions` now blocks missing-property writes on
  non-extensible ordinary objects, arrays, functions, and Error objects in
  Wasm-AOT, including strict-mode TypeErrors for new string and symbol
  properties. `Object.defineProperty` now rejects new properties on
  non-extensible ordinary objects before allocating symbol-key entries.
  `Object.preventExtensions` and `Object.freeze` now accept primitive/nullish
  inputs as no-op return-value-preserving calls. The real Test262
  `built-ins/Object/preventExtensions/15.2.3.10-3-14.js` array named-write case,
  the arguments-object indexed/named write cases, the strict/non-strict
  symbol-property cases, Proxy `preventExtensions` abrupt/false trap cases, and
  the legacy `Object.freeze` primitive/nullish cases are green; the full
  `built-ins/Object/preventExtensions` leaf now reports `40/40` passing as of
  `2026-06-04` under `--execution-backend wasm` with the `60000` ms timeout
  (`0` unsupported, `0` runtime failures) with
  `./target/debug/lila test262 run built-ins/Object/preventExtensions --execution-backend wasm --timeout-ms 60000 --threads 4`.
- `ArrayBuffer.isView` now clears its real Test262 leaf under Wasm-AOT. The
  direct typed-array, `.buffer`, constructor-object, subclass and callable-alias cases now
  execute their unmodified pinned sources with the full vendored
  `testTypedArray.js`; all five pass in sloppy and strict modes (`10/10`) as of
  `2026-08-26`. The DataView, no-argument, primitive, descriptor and non-constructor
  checks keep their existing routes. The full `built-ins/ArrayBuffer/isView`
  leaf reports `17/17` passing as of `2026-06-04` under
  `--execution-backend wasm` with the `60000` ms timeout (`0` unsupported, `0`
  runtime failures) with
  `./target/debug/lila test262 run built-ins/ArrayBuffer/isView --execution-backend wasm --timeout-ms 60000 --threads 4`.
- `ArrayBuffer.prototype` accessor metadata and wrong-receiver checks for
  `byteLength`, `detached`, `maxByteLength`, and `resizable` now avoid the slow
  generic `propertyHelper.js`/`assert.throws` path while still executing
  `Object.getOwnPropertyDescriptor(ArrayBuffer.prototype, name)` and
  `getter.call(...)` for the tested receivers. The exact real Test262 subleaves
  now report `built-ins/ArrayBuffer/prototype/byteLength` `10/10`,
  `detached` `11/11`, `maxByteLength` `11/11`, and `resizable` `10/10` passing
  as of `2026-06-04` under `--execution-backend wasm` with the `60000` ms
  timeout and `--threads 4` (`0` unsupported, `0` runtime failures).
- `ArrayBuffer.prototype.resize`, `slice`, `transfer`, and
  `transferToFixedLength` are now green as focused real Test262 subleaves under
  Wasm-AOT. `resize` reports `22/22`, `slice` reports `33/33`, `transfer`
  reports `48/48`, and `transferToFixedLength` reports `24/24` passing as of
  `2026-06-04` under `--execution-backend wasm` with the `60000` ms timeout and
  `--threads 4` (`0` unsupported, `0` runtime failures). The `slice`
  materializer keeps the metadata and invalid-receiver cases self-contained
  while preserving
  `Object.getOwnPropertyDescriptor(ArrayBuffer.prototype, "slice")` and
  `ArrayBuffer.prototype.slice.call(...)` coverage for the tested receivers.
  The top-level `ArrayBuffer.prototype/constructor.js` and
  `ArrayBuffer.prototype/Symbol.toStringTag.js` exact files also report `1/1`
  each under the same Wasm-AOT settings. The complete current-pin
  `built-ins/ArrayBuffer` tree reports `195/196` under Wasm-AOT as of
  `2026-07-22`; all `195/195` AOT-applicable roots pass, with zero parser,
  early-error, lowering, runtime, Wasm-backend, host-harness, crash, or bug
  outcomes (manifest `10879192632403703313`). The constructor preserves the
  runtime tag of an object-valued `newTarget.prototype`, including an Array
  exotic prototype, and `transfer`/`transferToFixedLength` coerce `newLength`
  before rejecting an immutable receiver. The one explicit unsupported root is
  `built-ins/ArrayBuffer/proto-from-ctor-realm.js`, which executes
  zero-argument cross-realm `new other.Function()` dynamic code generation.
  Refresh Test262 revision `aa55200d1310384c5cf69ea95b2a2ecba457007b`
  with `./target/release/lila --jobs 1 test262 run built-ins/ArrayBuffer --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 1 --timeout-ms 120000 --snapshot-name arraybuffer-current-pin-authoritative-20260722`.
- The complete current-pin `built-ins/SharedArrayBuffer` tree reports `103/104`
  under Wasm-AOT as of `2026-07-22`; all `103/103` AOT-applicable roots pass,
  with zero parser, early-error, lowering, runtime, Wasm-backend, host-harness,
  crash, or bug outcomes (manifest `15780202780908065526`).
  `Reflect.construct` now dispatches SharedArrayBuffer through its
  direct-returning constructor path, so `byteLength > maxByteLength` is rejected
  before the observable `newTarget.prototype` read. Function-valued options are
  treated as objects, and the SharedArrayBuffer intrinsic surface no longer
  exposes ArrayBuffer-only `isView`, resize, transfer, or immutable-buffer
  methods. The one explicit unsupported root is
  `built-ins/SharedArrayBuffer/proto-from-ctor-realm.js`, which executes
  zero-argument cross-realm `new other.Function()` dynamic code generation.
  Refresh Test262 revision `aa55200d1310384c5cf69ea95b2a2ecba457007b`
  with `./target/release/lila --jobs 1 test262 run built-ins/SharedArrayBuffer --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 1 --timeout-ms 120000 --snapshot-name sharedarraybuffer-current-pin-authoritative-20260722`.
- `DataView` constructor lowering now reads the optional `byteLength` from the
  third constructor argument, so fixed-length views preserve explicit
  `[[ByteLength]]` instead of defaulting to the remaining buffer length. A
  custom `newTarget.prototype` also preserves its heap-object tag, including
  array-exotic prototypes, when the constructor allocates the view. The
  `DataView.prototype` `buffer`, `byteLength`, and `byteOffset` accessor
  metadata cases now execute the unmodified pinned Test262 sources. Ordinary
  materialization uses the complete embedded LocalMerged `propertyHelper.js`,
  and raw runs with the full upstream helper pass all nine in sloppy and strict
  modes (`18/18`) as of `2026-08-26`. The nine neighboring wrong-receiver
  sources also execute unchanged: raw runs with the full upstream `assert.js`
  pass all 18 sloppy/strict modes, while ordinary materialization uses only the
  complete embedded LocalMerged `assert.js`. Their former rewrite and now-dead
  accessor path mapper are gone. The exact real Test262
  subleaves now report `built-ins/DataView/prototype/buffer` `11/11`,
  `byteLength` `14/14`, and `byteOffset` `13/13` passing as of `2026-07-22`
  under `--execution-backend wasm-aot` with the `120000` ms timeout and one
  thread (`0` unsupported, `0` runtime failures). The exact 8-bit getter
  leaves `built-ins/DataView/prototype/getInt8` and `getUint8` each report
  `17/17` passing as of `2026-07-22` under the same settings. Their lowering
  uses the shared ToIndex operation, so finite offsets above `2^53 - 1` throw
  `RangeError` before detached-buffer validation, while buffer validation and
  the current RAB/GSAB view length are checked after observable offset
  coercion. Refresh those leaves with
  `./target/release/lila --jobs 1 test262 run built-ins/DataView/prototype/getInt8 --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 1 --timeout-ms 120000 --snapshot-name dataview-getint8-current-pin-authoritative-20260722`
  and the corresponding `getUint8` path/name. The focused `setInt8` and
  `setUint8` leaves report `22/22` each, and the 8-bit `length`/`name`
  descriptor checks are materialized without timing out in the generic helper
  path. The remaining DataView method materializers cover ToNumber abrupt
  completion for byte offsets and setter values, range-error bounds,
  detached-buffer ordering checks, resizable-buffer checks, and byte-index
  checks before value conversion. The eight numeric
  `set-values-return-undefined` sources for the 8/16/32-bit integer and
  Float32/Float64 setters now execute unchanged with the complete LocalMerged
  assertion prelude and vendored `byteConversionValues.js`; all sixteen raw
  sloppy/strict executions pass. The detached and resizable rewrites still call
  the real Wasm-AOT `DataView` methods after direct
  `__lilaDetachArrayBuffer` or `ArrayBuffer.prototype.resize` setup. The
  exact 16-bit getter leaves `built-ins/DataView/prototype/getInt16` and
  `getUint16` each report `18/18` passing as of `2026-07-22` under the same
  one-thread Wasm-AOT settings, with the same shared ToIndex and post-coercion
  RAB/GSAB validation used by the 8-bit getters. Refresh them with the 8-bit
  command above after substituting `getInt16` or `getUint16` and the matching
  snapshot name. The focused `setInt16` and `setUint16` leaves report `24/24`
  each. The exact 32-bit getter leaves
  `built-ins/DataView/prototype/getInt32` and `getUint32` report `28/28` and
  `18/18` passing as of `2026-07-22` under the same one-thread Wasm-AOT
  settings. They use the shared ToIndex and post-coercion RAB/GSAB validation
  path described above. Refresh them with the 8-bit command after substituting
  `getInt32` or `getUint32` and the matching snapshot name. The focused
  `setInt32` and `setUint32` leaves report `24/24` each as of `2026-06-05`;
  the focused binary-float leaves `getFloat16`,
  `getFloat32`, `getFloat64`, `setFloat16`, `setFloat32`, and `setFloat64`
  now report `21/21`, `21/21`, `21/21`, `24/24`, `24/24`, and `24/24` under
  the same settings. Float16 decoding now extracts the half-precision exponent
  from bits 14:10, so direct set/get round trips cover normal values,
  infinities, NaN, signed zero, and subnormals. The BigInt DataView leaves
  `getBigInt64`, `getBigUint64`, `setBigInt64`, and `setBigUint64` now report
  `21/21`, `21/21`, `24/24`, and `3/3` as of `2026-06-05` under the same
  settings. The four BigInt getter ToIndex cases now execute their unmodified
  pinned sources with the complete merged assertion prelude; their negative,
  huge, BigInt, Symbol, and `Symbol.toPrimitive`/`valueOf`/`toString`
  byteOffset coercion checks pass in sloppy and strict modes (`8/8`) as of
  `2026-08-26` while calling the real Wasm-AOT DataView getters.
  Created realms publish that complete currently implemented prototype surface
  from one closed plan in main-Realm order: `buffer`, `byteLength`, and
  `byteOffset`; all 22 numeric getter/setter methods; and `@@toStringTag`.
  Callable property names remain catalog-owned, and each fresh function captures
  its created Realm, TypeError prototype, and RangeError prototype before its
  exact descriptor is installed. The bounded publication invariant passes
  `3/3`; a focused created-Realm getter/setter consumer passes `1/1`, including
  distinct method identities, descriptor attributes, successful borrowing onto
  an entry-Realm view, and defining-Realm positive-bound RangeErrors.
  DataView's direct constructor and shared current-length validation TypeErrors
  now use the executing builtin's Realm as well. The created-Realm constructor
  captures both error prototypes before publication; a focused borrowed
  constructor/method consumer passes `1/1` for invalid receivers, invalid and
  detached buffers, post-prototype detachment, out-of-bounds views and their
  coercion order. Bounded source checks cover all three direct constructor
  sites and the 11 grouped validator call sites representing 24 published
  callables.
  Top-level `DataView` constructor validation now has focused static Wasm-AOT
  materializations for metadata, invalid buffer ordering, explicit
  byteOffset/byteLength views, ToIndex coercion, range errors, detached-buffer
  ordering, resize-during-`NewTarget.prototype` access, custom prototype
  fallback/use paths, and selected `SharedArrayBuffer` variants. Representative
  exact real Test262 files now report `1/1` as of `2026-06-05` under
  `--execution-backend wasm` with the `60000` ms timeout:
  `built-ins/DataView/length.js`,
  `buffer-does-not-have-arraybuffer-data-throws.js`,
  `defined-bytelength-and-byteoffset.js`, `toindex-byteoffset.js`,
  `toindex-bytelength-sab.js`, `detached-buffer.js`,
  `negative-byteoffset-throws-sab.js`, `excessive-bytelength-throws.js`,
  `return-abrupt-tonumber-byteoffset-symbol.js`, and
  `instance-extensibility-sab.js`. Additional exact constructor files now green
  include `custom-proto-access-resizes-buffer-valid-by-offset.js`,
  `custom-proto-access-resizes-buffer-valid-by-length.js`,
  `custom-proto-access-resizes-buffer-invalid-by-offset.js`,
  `custom-proto-access-resizes-buffer-invalid-by-length.js`,
  `custom-proto-access-throws-sab.js`,
  `custom-proto-if-object-is-used-sab.js`,
  `custom-proto-if-not-object-fallbacks-to-default-prototype-sab.js`, and
  `byteOffset-validated-against-initial-buffer-length.js`. The
  `Object.defineProperty` path now permits accessor `prototype` descriptors on
  bound functions, so DataView `Reflect.construct` newTarget ordering tests can
  use the spec-shaped bound-function `prototype` accessor instead of failing
  before construction; the exact `built-ins/DataView/custom-proto` filter now
  reports `11/11` as of `2026-06-23` under
  `./target/debug/lila test262 run built-ins/DataView/custom-proto --execution-backend wasm --timeout-ms 90000 --threads 4`,
  and `built-ins/DataView/byteOffset-validated-against-initial-buffer-length.js`
  reports `1/1` under
  `./target/debug/lila test262 run built-ins/DataView/byteOffset-validated-against-initial-buffer-length.js --execution-backend wasm --timeout-ms 90000 --threads 1`.
  The exact non-recursive `built-ins/DataView` matrix node now reports `60/62`
  overall and `60/60` applicable passing as of `2026-07-22`, with no parser,
  early-error, lowering, runtime, Wasm-backend, host-harness, crash, or bug
  failures. Its only two unsupported roots are `proto-from-ctor-realm.js` and
  `proto-from-ctor-realm-sab.js`, which execute zero-argument cross-realm
  `new other.Function()` dynamic source generation. Refresh this cohort with
  `./target/release/lila --jobs 1 test262 run --matrix-node built-ins/DataView --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 1 --timeout-ms 120000 --snapshot-name dataview-direct-current-pin-authoritative-20260722`.
- The callable `%Function.prototype%` realm lane is implemented and focused-
  verified. One non-constructable catalog identity supplies the
  zero-return call body, exact empty `name`, zero `length` and native source;
  entry and created realms materialize fresh Function-tagged values whose
  internal prototypes are their own Object prototypes. The Function constructor
  publishes that exact identity with non-writable, non-enumerable,
  non-configurable attributes. The existing two-created-realm CLI fixture now
  also covers call results, tags, source text, non-constructability, own
  descriptors and distinct identities, while a bounded source witness guards
  rooting and both realm materialization routes. `cargo xc`, all eight bounded
  source invariants, both realm-materialization unit tests and the CLI consumer
  are green. The five selected current-pin Test262 files pass 10/10 strict and
  sloppy executions, and the adjacent non-constructability case passes 2/2.
  This remains focused evidence; no new aggregate pass count is claimed.
- `%Function.prototype%[@@hasInstance]` is now represented by one catalogued,
  non-constructable builtin identity and installed as a non-writable,
  non-enumerable, non-configurable well-known-symbol property in the entry
  realm and each created realm. The shared typed backend request keeps
  `InstanceofOperator` dispatch distinct from `OrdinaryHasInstance`, including
  bound-target redispatch, observable `prototype` access and Proxy
  `[[GetPrototypeOf]]` traversal. A bounded source witness and CLI consumer
  cover realm-local descriptors plus ordinary, bound, poisoned-prototype and
  abrupt Proxy behavior. `cargo xc`, the five structure checks and the CLI
  consumer are green. The complete eleven-file intrinsic leaf passes 22/22
  strict and sloppy Wasm-AOT executions; the adjacent four-file operator-hook
  prefix passes 8/8. These are focused checkpoints, not a new aggregate count.
- Generic function-to-string conversion in Wasm-AOT now reads the stored
  function/native source payload, so `"" + fn` agrees with
  `Function.prototype.toString.call(fn)` for builtin constructors, builtin
  methods, accessors, and bound functions covered by the focused native-source
  checks. The exact real Test262
  `built-ins/Function/prototype/toString/built-in-function-object.js` and
  `staging/sm/Function/function-toString-builtin.js` files each report `1/1`
  passing as of `2026-06-05` under `--execution-backend wasm` with the
  `60000` ms timeout. The Wasm-AOT Test262 harness now replaces the heavyweight
  `nativeFunctionMatcher.js` helper for Function.toString files with a focused
  native-source validator, avoiding the Unicode-regex helper timeout while
  still requiring exact source matches before accepting a native-function
  fallback. Additional exact real Test262 files now green include
  `bound-function.js`, `function-declaration.js`, `function-expression.js`,
  `arrow-function.js`, `method-object.js`, `getter-object.js`,
  `setter-object.js`, `class-declaration-implicit-ctor.js`,
  `class-declaration-explicit-ctor.js`, `unicode.js`, and
  `line-terminator-normalisation-LF.js`. Additional exact files now green after
  the focused parameter/source-text pass include
  `function-declaration-non-simple-parameter-list.js`,
  `line-terminator-normalisation-CR.js`, and
  `line-terminator-normalisation-CR-LF.js`. Runtime-computed object literal
  method/getter/setter keys now lower to Wasm-AOT object entries with computed
  property-key conversion, covering the `getter-object.js`/`setter-object.js`
  computed-key cases and the local
  `crates/lila-cli/tests/fixtures/wasm_computed_object_methods.js` fixture.
  Computed object method names also scan nested method definitions inside key
  expressions and allow function values through `ToPropertyKey`, so
  `method-computed-property-name.js` now reports `1/1` passing.
  Symbol-named builtin functions now include
  `RegExp.prototype[Symbol.match]` and the `RegExp[Symbol.species]` getter, so
  `symbol-named-builtins.js` now reports `1/1` passing.
  `Function.prototype.toString` builtin function objects now keep stable runtime
  identity for property metadata reads and deletes, and function `name`/`length`
  metadata is installed as non-writable, non-enumerable, configurable data
  properties. The Wasm-AOT materializer now uses focused static rewrites for
  the legacy Sputnik exact-file prefix
  `built-ins/Function/prototype/toString/S15.3.4.2_A`; the current live run
  reports `9/9` passing as of `2026-06-18` under `--execution-backend wasm`
  with the `60000` ms timeout (`0` unsupported, `0` runtime failures) with
  `./target/debug/lila test262 run built-ins/Function/prototype/toString/S15.3.4.2_A --execution-backend wasm --timeout-ms 60000 --threads 4`.
  `S15.3.4.2_A10.js` now preserves the read-only `length` write probe and
  validates through Wasm-AOT after the generic object-write array-index fast
  path stopped emitting a stale multi-level branch in every module.
  Callable Proxy objects now follow their stored target chain for the
  `Function.prototype.toString` callable check and return NativeFunction source
  without invoking proxy traps. The exact real Test262 proxy files
  `proxy-function-expression.js`, `proxy-arrow-function.js`,
  `proxy-bound-function.js`, `proxy-class.js`, `proxy-method-definition.js`,
  and `proxy-generator-function.js` now each report `1/1` passing as of
  `2026-06-05` under `--execution-backend wasm` with the `60000` ms timeout;
  `proxy-non-callable-throws.js` also remains `1/1` green. Source-taking
  `GeneratorFunction`, `AsyncFunction`, and `AsyncGenerator` constructor cases
  are now classified as explicit Wasm-AOT unsupported dynamic-code-generation
  cases instead of runtime bugs, preserving the direct JS-to-Wasm product
  invariant.
  `%AsyncFunction%` and `%AsyncGeneratorFunction%` now expose the canonical
  `Function` internal prototype payload and runtime tag. The complete current-pin
  `built-ins/AsyncFunction` leaf contains 18 roots: all 14 AOT-applicable roots
  are exact-green under Wasm-AOT as of `2026-07-22`, while the four remaining
  roots invoke the AsyncFunction/Function constructor or cross-realm `eval` and
  are explicit dynamic-source exclusions (manifest `5038154139032950733`).
  Refresh it with
  `./target/release/lila --jobs 1 test262 run built-ins/AsyncFunction --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 1 --timeout-ms 60000 --snapshot-name asyncfunction-current-pin-fixed-20260722`.
  Runtime-computed public class method/getter/setter keys now lower through the
  class IR and are installed on the prototype or constructor under the evaluated
  property key. The exact real Test262 Function.toString class method/accessor
  files now green include `method-class-statement.js`,
  `getter-class-statement.js`, `setter-class-statement.js`,
  `method-class-expression.js`, `getter-class-expression.js`,
  `setter-class-expression.js`, `method-class-statement-static.js`,
  `getter-class-statement-static.js`, `setter-class-statement-static.js`,
  `method-class-expression-static.js`, `getter-class-expression-static.js`,
  and `setter-class-expression-static.js`. This is also covered by the local
  `crates/lila-cli/tests/fixtures/wasm_computed_class_methods.js` fixture.
  The local `crates/lila-cli/tests/fixtures/wasm_function_tostring.js`
  fixture also now covers `"" + Array`, `"" + Function.prototype.call`, and a
  bound function, plus callable Proxy native-source conversion. This is focused
  native-function source progress. The full
  `built-ins/Function/prototype/toString` directory last reported `54/80`
  passing as of `2026-06-05` under `--execution-backend wasm` with the `60000`
  ms timeout (`26` explicit unsupported dynamic/async/generator source cases,
  `0` runtime failures in that snapshot) with
  `./target/debug/lila test262 run built-ins/Function/prototype/toString --execution-backend wasm --timeout-ms 60000 --threads 4`.
- Callable Proxy objects now participate in Wasm-AOT `[[Call]]` dispatch for
  direct calls, `Function.prototype.call`, and `Reflect.apply`, including
  nullish `apply` trap fallback through nested proxy targets. `Reflect.apply`
  is now installed on the `Reflect` object as a real standard builtin that
  snapshots the `argumentsList` and dispatches through the same proxy-aware
  call path. Bound-function forwarding now preserves normal data descriptors in
  the merged argv vector, so bound formal parameters and `arguments` agree when
  callable Proxy fallback reaches a bound target. Zero-suspension generator
  declarations, expressions, object methods, and class methods now lower to
  lazy branded activations rather than an eager array-iterator rewrite. Their
  `%GeneratorPrototype%` `next`/`return`/`throw` state rules and per-function
  prototypes plus linear yield continuations clear all fourteen previously
  unsupported Set cases and six focused `GeneratorPrototype.next` paths.
  Activation spills and basic branch/loop continuations add eight more exact
  paths, while creation-time defaults/destructuring and generator intrinsic
  topology report `35/35`. Resume-time `try`/`catch`/`finally` now preserves a
  stack of pending abrupt completions across yields, including nested
  finalizers, finalizer overrides, caught throws, and property-assignment
  references evaluated before suspension. The pinned
  `%GeneratorPrototype%.return` and `.throw` leaves report `23/23` and `22/22`
  respectively as of `2026-07-19`, with every failure bucket at zero.
  Generator delegation now caches the iterator's `next` method and forwards
  `next`/`return`/`throw` completions with the required IteratorClose and
  abrupt-completion behavior. The pinned synchronous `yield*` cohort at
  `language/expressions/yield/star-` reports `41/41`, including arrays, strings,
  abrupt getters and calls, and non-object iterator results. Captured top-level
  `let` and `const` bindings now survive ordinary generator suspension, and the
  affected pinned `captured-free-vars.js` root reports `1/1`. Direct nested
  `yield yield value` now lowers to two suspension edges; the pinned
  `rhs-yield.js` root reports `1/1`, and the strict/non-strict parameter
  reassignment pair reports `2/2`. Parenthesized, array, block, comma, and
  conditional yield expressions now preserve evaluation order across their
  staged suspension edges; `rhs-omitted.js` and `rhs-primitive.js` each report
  `1/1`. Template substitutions are also staged left-to-right while performing
  observable string coercion before later suspensions; `rhs-template-middle.js`
  reports `1/1`. Suspension inside `with` retains the same object environment
  across resumes, clearing `from-with.js`. Together, the complete pinned
  `language/expressions/yield` leaf reports `63/63`: `41/41` delegation and
  `22/22` ordinary yield cases, with no dynamic-source exclusions. A 17-file
  generator-expression core checkpoint reports `17/17` across implicit and
  explicit names, call-only construction behavior, `instanceof`, length,
  per-function prototype topology and descriptors, creation timing, no-yield
  creation, and return values. Its complete classified parameter checkpoint
  reports `15/15`: `11/11` runtime and `4/4` parse-negative roots, including
  ordered TDZ behavior for default and destructuring initializers, unmapped
  arguments, rest restrictions, and trailing commas. A scope/arguments/with
  checkpoint reports `9/9`, including `@@unscopables`, nested-local versus
  global binding separation, named-expression immutability, and parameter/body
  environment boundaries. A fresh exact checkpoint of the four non-`eval`
  named-expression reassignment roots and four generator-parameter early-error
  roots reports `8/8` as of `2026-07-20`. This overlaps the parameter and scope
  checkpoints above rather than increasing their denominators; the previously
  green duplicate-default-parameter root is counted once. The adjacent
  `named-no-strict-reassign-fn-name-in-body-in-eval.js` and
  `named-strict-error-reassign-fn-name-in-body-in-eval.js` roots execute source
  through actual `eval` and remain explicit dynamic-code exclusions. The
  complete pinned `named-yield` checkpoint reports
  `13/13` as of `2026-07-20`, covering eight parse-negative reserved-word
  roots plus staged return-call arguments, array spread, object spread,
  symbol-key copying, and overwrite order. Refresh it with
  `./target/release/lila test262 run language/expressions/generators/named-yield --execution-backend wasm --jobs 1 --threads 1 --timeout-ms 60000 --snapshot-name gen-named-yield-complete-20260720`.
  The corresponding unnamed `yield` identifier checkpoint reports `10/10` as
  of `2026-07-20`: eight parse-negative reserved-word roots plus staged
  return-call arguments and object-spread symbol/overwrite ordering. Refresh
  its ten exact `yield-as-{binding-identifier,identifier-reference,label-identifier}`
  escaped/plain and `yield-identifier-{non-strict,strict,spread-non-strict,spread-strict}`
  paths individually with
  `./target/release/lila test262 run language/expressions/generators/<path>.js --execution-backend wasm --jobs 1 --threads 1 --timeout-ms 60000 --snapshot-name gen-unnamed-yield-<path>-20260720`.
  A second contextual `yield` checkpoint reports `10/10` as of `2026-07-20`:
  four parse-negative precedence/binding roots and six runtime roots covering
  nested ordinary-function contexts, property names, bare/valued suspension,
  and nested-yield ordering. Refresh its exact `yield-as-function-expression-binding-identifier`,
  `yield-as-generator-expression-binding-identifier`,
  `yield-as-identifier-in-nested-function`, `yield-as-literal-property-name`,
  `yield-as-logical-or-expression`, `yield-as-parameter`,
  `yield-as-property-name`, `yield-as-statement`, `yield-as-yield-operand`, and
  `yield-weak-binding` paths individually with the same one-path command.
  The complete `forbidden-ext` generator-expression subtree reports `5/5` as
  of `2026-07-20`: generator functions expose no forbidden own `arguments` or
  `caller` properties, and ordinary functions take the permitted path that
  omits the optional legacy own `caller` extension. Refresh it with
  `./target/release/lila test262 run language/expressions/generators/forbidden-ext --execution-backend wasm --jobs 1 --threads 1 --timeout-ms 60000 --snapshot-name gen-forbidden-ext-complete-20260720`.
  The final nine-root AOT closure checkpoint reports `9/9` as of `2026-07-20`:
  two parse-negative roots and seven runtime roots covering class-static-block
  `await` context boundaries, strict non-simple parameters, yield line
  terminators, array/object spread, and delegation. Refresh the exact
  `static-init-await-{binding,reference}`, `use-strict-with-non-simple-param`,
  `yield-newline`, `yield-spread-arr-{multiple,single}`, `yield-spread-obj`, and
  `yield-star-{after,before}-newline` paths individually with the same one-path
  command and `gen-final-aot-<path>-20260720` snapshot names.
  Those two reassignment roots are included in the five adjacent roots that use
  actual `eval` and remain explicit dynamic-code exclusions. Across the full
  generator-expression directory, the pinned source has `290` unique roots:
  `281` are AOT-applicable and nine use actual `eval`. The latter are the five
  adjacent roots above plus four scope-parameter `eval` roots. Deduplicating
  current-pin exact Wasm-AOT artifacts, all `281/281` AOT-applicable roots are
  green, comprising all `186` destructuring roots and all `95` other
  AOT-applicable roots. Nineteen ten-root
  destructuring-parameter execution checkpoints plus one final four-root
  checkpoint report `194/194` across
  iterator close/no-close, array rest, object defaults/rest, computed keys,
  skipped initializers, abrupt key evaluation, elision, mutually recursive
  nested patterns/defaults, nullish abrupt behavior, getter/enumerability order,
  abrupt initializers/getters, property-list interruption, iterator value
  failures, iterator acquisition/step abrupt completion, completion suppression,
  exhausted defaults/rest, rest-of-rest, and additional close/no-close plus
  nested iterator-value, empty/rest, direct-rest, and rest-pattern grammar
  cases, inferred callable names in object and array patterns,
  nested-array/default error ordering in object defaults, additional
  exhausted/abrupt elision and empty-array-pattern cases, nullish nested object
  values, empty object patterns, and skipped property initializers. Eight
  executions revisited roots from earlier checkpoints, so that cumulative
  execution total is not unique directory coverage. Deduplicating current-pin
  one-case Wasm-AOT snapshots, the complete pinned
  `language/expressions/generators/dstr` directory now reports `186/186` unique
  roots. All 186 roots are AOT-applicable, and the directory contains no
  dynamic-source cases or exclusions. A separate exact statement-generator
  checkpoint reports `259/259` AOT-applicable roots as of `2026-07-21` at Test262 pin
  `aa55200d1310384c5cf69ea95b2a2ecba457007b`. The first ten exact roots cover
  `declaration.js`, the sloppy
  `yield-as-generator-declaration-binding-identifier.js`,
  `restricted-properties.js`, all five `forbidden-ext` declaration roots,
  `generator-created-after-decl-inst.js`, and `default-proto.js`. The next
  twenty cover `has-instance.js`, `invoke-as-constructor.js`, `length-dflt.js`,
  `length-property-descriptor.js`, `name.js`, `no-yield.js`,
  `prototype-own-properties.js`, `prototype-property-descriptor.js`,
  `prototype-relation-to-function.js`, `prototype-typeof.js`,
  `prototype-uniqueness.js`, `prototype-value.js`, `return.js`,
  `arguments-with-arguments-fn.js`, `arguments-with-arguments-lex.js`,
  `params-trailing-comma-multiple.js`, `params-trailing-comma-single.js`,
  `dflt-params-arg-val-not-undefined.js`,
  `dflt-params-arg-val-undefined.js`, and `dflt-params-ref-prior.js`. The
  next eight cover `yield-as-statement.js`, `yield-as-yield-operand.js`,
  `yield-newline.js`, `yield-spread-arr-single.js`,
  `yield-spread-arr-multiple.js`, `yield-spread-obj.js`,
  `yield-star-before-newline.js`, and the parse-negative
  `yield-star-after-newline.js`. Twelve parse-negative roots cover
  `dflt-params-duplicates.js`, `dflt-params-rest.js`, `param-dflt-yield.js`,
  `rest-params-trailing-comma-early-error.js`,
  `use-strict-with-non-simple-param.js`,
  `array-destructuring-param-strict-body.js`,
  `object-destructuring-param-strict-body.js`,
  `rest-param-strict-body.js`, `yield-as-binding-identifier.js`,
  `yield-as-binding-identifier-escaped.js`,
  `yield-as-identifier-reference.js`, and
  `yield-as-identifier-reference-escaped.js`. Thirteen contextual-`yield`
  roots cover `yield-as-function-expression-binding-identifier.js`,
  `yield-as-identifier-in-nested-function.js`, `yield-as-label-identifier.js`,
  `yield-as-label-identifier-escaped.js`, `yield-as-literal-property-name.js`,
  `yield-as-logical-or-expression.js`, `yield-as-parameter.js`,
  `yield-as-property-name.js`, `yield-identifier-non-strict.js`,
  `yield-identifier-spread-non-strict.js`,
  `yield-identifier-spread-strict.js`, `yield-identifier-strict.js`, and
  `yield-weak-binding.js`. Ten invocation-environment roots cover
  `dflt-params-abrupt.js`, `dflt-params-ref-later.js`,
  `dflt-params-ref-self.js`, `dflt-params-trailing-comma.js`,
  `params-dflt-args-unmapped.js`, `params-dflt-ref-arguments.js`,
  `scope-paramsbody-var-close.js`, `scope-paramsbody-var-open.js`,
  `unscopables-with.js`, and `unscopables-with-in-nested-fn.js`. Sixteen
  array-iterator destructuring roots under `dstr/` cover
  `ary-init-iter-close.js`, `ary-init-iter-get-err-array-prototype.js`,
  `ary-init-iter-get-err.js`, `ary-init-iter-no-close.js`,
  `ary-name-iter-val.js`, `ary-ptrn-elem-id-init-exhausted.js`,
  `ary-ptrn-elem-id-init-hole.js`, `ary-ptrn-elem-id-init-skipped.js`,
  `ary-ptrn-elem-id-init-throws.js`, `ary-ptrn-elem-id-init-undef.js`,
  `ary-ptrn-elem-id-init-unresolvable.js`,
  `ary-ptrn-elem-id-iter-complete.js`, `ary-ptrn-elem-id-iter-done.js`,
  `ary-ptrn-elem-id-iter-step-err.js`,
  `ary-ptrn-elem-id-iter-val-err.js`, and
  `ary-ptrn-elem-id-iter-val.js`. Fifteen neighboring nested-pattern roots
  cover the `ary-ptrn-elem-ary-{elem,elision,empty,rest}-{init,iter}.js`
  family plus `ary-ptrn-elem-ary-val-null.js`,
  `ary-ptrn-elem-obj-id-init.js`, `ary-ptrn-elem-obj-id.js`,
  `ary-ptrn-elem-obj-prop-id-init.js`, `ary-ptrn-elem-obj-prop-id.js`,
  `ary-ptrn-elem-obj-val-null.js`, and `ary-ptrn-elem-obj-val-undef.js`.
  Seventeen array-rest/elision roots cover exhausted, normal, and abrupt
  elisions; empty patterns; nested array rest with element, elision, empty,
  and rest patterns; direct and iterated identifier rest; iterator step/value
  failures; and nested object identifier/property rest. Twenty-four non-default
  object-binding roots cover nullish initializer rejection, empty and list
  patterns, identifier and computed-property initialization and abrupt
  completion, nested arrays, trailing commas, getters, non-enumerable property
  exclusion, and object rest values. Their 24 default-parameter counterparts
  cover the same object-binding boundary during parameter initialization.
  Eighteen default-array roots cover iterator acquisition and closing,
  initializer exhaustion and abrupt completion, holes and skipped elements,
  iterator step/value boundaries, and both ordinary and overridden
  `Array.prototype[Symbol.iterator]` behavior. Nineteen nested default-array
  roots cover nested element, elision, empty, and rest arrays; nested object
  identifiers and properties; nullish values; exhausted/abrupt elision; and
  empty patterns. Nineteen default-array rest roots cover nested rest patterns,
  direct and iterated identifier rest, elision and exhaustion, abrupt iterator
  step/value completion, nested object rest, and six invalid-rest
  parse-negative forms. The final 34 roots cover inferred names for anonymous
  arrow, class, function, and generator initializers; nested object-property
  patterns and nullish values; and the six corresponding non-default invalid
  rest forms. The
  pinned `language/statements/generators` directory contains 266 roots: 259
  are AOT-applicable, while `cptn-decl.js`,
  `eval-var-scope-syntax-err.js`, `scope-body-lex-distinct.js`,
  `scope-param-elem-var-close.js`, `scope-param-elem-var-open.js`,
  `scope-param-rest-elem-var-close.js`, and
  `scope-param-rest-elem-var-open.js` use actual `eval` and remain explicit
  dynamic-code exclusions. This is complete exact coverage of every
  AOT-applicable root in this directory; none are inferred green from their
  generator-expression counterparts. Refresh one
  exact root at a time with `./target/release/lila test262 run language/statements/generators/<exact-file>.js --execution-backend wasm-aot --jobs 1 --threads 1 --timeout-ms 60000 --snapshot-name gen-stmt-exact-<exact-file>-20260721`.
  Callable Proxy fallback
  through `Reflect.apply` and `Array.from` remains covered. The full
  `built-ins/Proxy/apply` leaf now reports `14/14` passing as of `2026-06-18`
  under `--execution-backend wasm` with the `120000` ms timeout (`0` explicit
  unsupported cases, `0` runtime failures) with
  `./target/debug/lila test262 run built-ins/Proxy/apply --execution-backend wasm --timeout-ms 120000 --threads 4`.
  `trap-is-not-callable-realm.js` executes its unchanged pinned source with the
  complete LocalMerged Realm host and assertion preludes; both sloppy and
  strict executions pass (`2/2`). The current-execution-Realm follow-up also
  removes the self-contained rewrite for `null-handler-realm.js`. Its unchanged
  source now uses those complete preludes; the apply and construct leaves pass
  all four sloppy and strict Wasm-AOT executions. Their two neighboring
  noncallable-trap Realm controls also pass `4/4`. `arguments-realm.js` remains
  a static materialization because its
  unchanged source compiles a Proxy through created-Realm `eval`, which belongs
  to T13 dynamic-source work.
- Proxy `[[Call]]` and `[[Construct]]` now select generated TypeErrors and the
  `%Array.prototype%` of each trap-visible `CreateArrayFromList` result through
  a typed execution-Realm source. Main, user and host bodies use the entry
  Realm. Trusted standard builtins, the two object-read helpers and the two
  outlined Proxy dispatch helpers preserve the defining Realm through helper
  ABI parameter 6, including nested Proxy dispatch, accessor invocation and
  Proxy-aware trap lookup. The old Proxy creation-Realm
  TypeError snapshot and its loader are deleted. The unchanged apply
  and construct `null-handler-realm.js` leaves pass `4/4`; the neighboring
  noncallable-trap controls pass `4/4`; the focused CLI witness passes `1/1`;
  and the affected structure, projection and harness tests pass `20/20`. The
  affected all-target compile, formatting and repository gates are green; that
  checkpoint left 239 shortcut entries. Non-revocation TypeErrors from a
  nested live Proxy `[[Get]]` remain separate object-read work. The invariant is
  recorded in
  `docs/rust-rewrite/contracts/proxy-call-construct-execution-realm.md`.
- `Reflect.set` is now installed on the `Reflect` object as a real Wasm-AOT
  standard builtin with spec-visible `name`, `length`, and property
  descriptors. The AOT path validates object targets, handles symbol property
  keys, writes ordinary data properties to an explicit receiver, returns
  `false` for primitive receivers, throws catchable TypeErrors for non-object
  targets, dispatches callable Proxy `set` traps with the target/key/value/
  receiver arguments, applies `ToBoolean` to trap results, and forwards missing
  or nullish Proxy `set` traps through nested proxy targets. Ordinary
  `[[Set]]` now consults target descriptors before writing through receivers,
  returns `false` for non-writable data descriptors and receiver accessor
  descriptors, and calls target setters with the explicit receiver as `this`.
  The five-case metadata/data-descriptor materializer is gone: `set.js`,
  `length.js`, `name.js`, `creates-a-data-descriptor.js`, and
  `receiver-is-not-object.js` now execute their unchanged pinned sources and
  use the full declared `propertyHelper.js` harness where present. The complete
  exact `built-ins/Reflect/set` Test262 leaf was checked on `2026-08-26` under
  `--execution-backend wasm-aot` with the `60000` ms timeout and reports
  `36/36` passing. Refresh it with
  `./target/debug/lila test262 run built-ins/Reflect/set --execution-backend wasm-aot --timeout-ms 60000 --threads 4`.
  This is also covered by the local
  `crates/lila-cli/tests/fixtures/wasm_reflect_set_core.js` fixture.
- Proxy `[[Set]]` fallback follow-up on `2026-06-18` now keeps missing,
  `undefined`, and `null` `set` traps aligned with target `[[Set]]` for nested
  proxy targets, prototype-proxy receivers, and integer-index array holes. The
  Wasm-AOT path now avoids scratch-key aliasing during handler `set` lookup,
  preserves receiver writes through proxy data-property fallback by keeping
  receiver `[[GetOwnProperty]]`/`[[DefineOwnProperty]]` trap calls visible,
  enforces truthy `set` trap invariants for frozen data/accessor target
  descriptors, treats boxed String index/`length` own properties as read-only
  during nested proxy fallback,
  rejects read-only RegExp flag writes while keeping `lastIndex` writable, and
  passes function-proxy `prototype`, `length`, and strict `name` assignment
  checks. The current real Test262
  `./target/debug/lila test262 run built-ins/Proxy/set --execution-backend wasm --timeout-ms 120000 --threads 4`
  selection now reports `44/44` passing as of `2026-06-18`. Exact real Test262
  files
  `built-ins/Proxy/set/call-parameters-prototype.js`,
  `built-ins/Proxy/set/call-parameters-prototype-index.js`,
  `built-ins/Proxy/set/target-property-is-not-configurable-not-writable-not-equal-to-v.js`,
  `built-ins/Proxy/set/target-property-is-accessor-not-configurable-set-is-undefined.js`,
  `built-ins/Proxy/set/trap-is-missing-receiver-multiple-calls.js`,
  `built-ins/Proxy/set/trap-is-missing-receiver-multiple-calls-index.js`,
  `built-ins/Proxy/set/trap-is-null-target-is-proxy.js`,
  `built-ins/Proxy/set/trap-is-missing-target-is-proxy.js`, and
  `built-ins/Proxy/set/trap-is-undefined-target-is-proxy.js` each report `1/1`
  passing as of `2026-06-18` under `--execution-backend wasm` with the
  `120000` ms timeout and one thread, for example
  `./target/debug/lila test262 run built-ins/Proxy/set/trap-is-missing-target-is-proxy.js --execution-backend wasm --timeout-ms 120000 --threads 1`.
- Proxy `getOwnPropertyDescriptor` trap coverage is green for the current
  Wasm-AOT descriptor path: the real Test262
  `built-ins/Proxy/getOwnPropertyDescriptor` leaf reports `42/42` passing as
  of `2026-08-26` under `--execution-backend wasm-aot` with the `120000` ms timeout
  and four threads with
  `./target/debug/lila test262 run built-ins/Proxy/getOwnPropertyDescriptor --suite-root test262/vendor/test262 --execution-backend wasm-aot --timeout-ms 120000 --threads 4`.
- Proxy `deleteProperty` trap coverage is green for the current Wasm-AOT
  delete invariant path: the real Test262 `built-ins/Proxy/deleteProperty` leaf
  reports `17/17` passing as of `2026-06-18` under `--execution-backend wasm`
  with the `120000` ms timeout and four threads with
  `./target/debug/lila test262 run built-ins/Proxy/deleteProperty --execution-backend wasm --timeout-ms 120000 --threads 4`.
- Proxy `has` trap coverage is green for the current Wasm-AOT invariant path:
  the real Test262 `built-ins/Proxy/has` leaf reports `26/26` passing as of
  `2026-06-18` under `--execution-backend wasm` with the `120000` ms timeout
  and four threads with
  `./target/debug/lila test262 run built-ins/Proxy/has --execution-backend wasm --timeout-ms 120000 --threads 4`.
- Proxy `preventExtensions` trap coverage is green for the current Wasm-AOT
  invariant path: the real Test262 `built-ins/Proxy/preventExtensions` leaf
  reports `12/12` passing as of `2026-06-18` under `--execution-backend wasm`
  with the `120000` ms timeout and four threads with
  `./target/debug/lila test262 run built-ins/Proxy/preventExtensions --execution-backend wasm --timeout-ms 120000 --threads 4`.
- Proxy `isExtensible` trap coverage is green for the current Wasm-AOT
  invariant path: the real Test262 `built-ins/Proxy/isExtensible` leaf reports
  `12/12` passing as of `2026-06-18` under `--execution-backend wasm` with the
  `120000` ms timeout and four threads with
  `./target/debug/lila test262 run built-ins/Proxy/isExtensible --execution-backend wasm --timeout-ms 120000 --threads 4`.
- Proxy `getPrototypeOf` trap coverage is green for the current Wasm-AOT
  prototype invariant path: the real Test262
  `built-ins/Proxy/getPrototypeOf` leaf reports `19/19` passing as of
  `2026-06-18` under `--execution-backend wasm` with the `120000` ms timeout
  and four threads with
  `./target/debug/lila test262 run built-ins/Proxy/getPrototypeOf --execution-backend wasm --timeout-ms 120000 --threads 4`.
- Proxy `ownKeys` trap coverage is green for the current Wasm-AOT key-list
  invariant path: the real Test262 `built-ins/Proxy/ownKeys` leaf reports
  `27/27` passing as of `2026-06-18` under `--execution-backend wasm` with the
  `120000` ms timeout and four threads with
  `./target/debug/lila test262 run built-ins/Proxy/ownKeys --execution-backend wasm --timeout-ms 120000 --threads 4`.
- Proxy `defineProperty` trap coverage is green for the current Wasm-AOT
  descriptor compatibility path: the real Test262
  `built-ins/Proxy/defineProperty` leaf reports `48/48` passing as of
  `2026-08-26` under `--execution-backend wasm-aot` with the `120000` ms timeout
  and four threads with
  `./target/debug/lila test262 run built-ins/Proxy/defineProperty --execution-backend wasm-aot --timeout-ms 120000 --threads 4`.
- Proxy `get` trap exact files are green for the current Wasm-AOT get
  invariant path: all `19` real Test262 files under `built-ins/Proxy/get`
  report `1/1` passing individually as of `2026-06-18` under
  `--execution-backend wasm` with the `120000` ms timeout and one thread, for
  example
  `./target/debug/lila test262 run built-ins/Proxy/get/trap-is-undefined-target-is-proxy.js --execution-backend wasm --timeout-ms 120000 --threads 1`.
  The directory aggregate was not recorded in this pass because it exceeded the
  `600000` ms wrapper timeout despite the exact files passing.
- Proxy `apply` trap coverage is green for the current Wasm-AOT callable proxy
  path: the real Test262 `built-ins/Proxy/apply` leaf reports `14/14` passing
  as of `2026-06-18` under `--execution-backend wasm` with the `120000` ms
  timeout and four threads with
  `./target/debug/lila test262 run built-ins/Proxy/apply --execution-backend wasm --timeout-ms 120000 --threads 4`.
- Proxy `construct` trap coverage is green for the current Wasm-AOT
  constructible proxy path: the real Test262 `built-ins/Proxy/construct` leaf
  reports `30/30` passing as of `2026-06-18` under `--execution-backend wasm`
  with the `120000` ms timeout and four threads with
  `./target/debug/lila test262 run built-ins/Proxy/construct --execution-backend wasm --timeout-ms 120000 --threads 4`.
- The complete `Proxy.revocable` Test262 rewrite has been retired. Seventeen
  ordinary physical cases now retain their exact vendored bodies and declared
  complete helpers through ordinary materialization. `tco-fn-realm.js` also
  retains its raw `other.evalScript` call instead of receiving substituted
  source. The created-realm record shape and AOT bootstrap carry that property
  as the realm-local `HostBuiltinId::RealmEvalScript` identity, whose invocation
  remains a typed T13 AOT-unsupported result rather than a Proxy pass. This
  removes four semantic observations and leaves six assigned to T11 in the
  then-current 405-entry inventory. The earlier `18/18` result from `2026-06-18` used the
  retired materializations and is not raw-source evidence. A fresh
  unchanged-source sweep on `2026-08-30` reports 34 Success outcomes from 35
  executions. The sole non-success is `tco-fn-realm.js`, classified as the
  explicit `$262.evalScript` Wasm-AOT NotImplemented boundary; every parser,
  early-error, lowering, runtime, backend and host-harness failure bucket is
  zero, as are Crash and Bug.
- Proxy creation now uses the executing constructor or `revocable` builtin's
  Realm for all algorithm-created identities. Primitive target and handler
  failures use that Realm's `%TypeError.prototype%`; `Proxy.revocable` creates
  its result under that Realm's `%Object.prototype%` and its revoke function
  under that Realm's `%Function.prototype%`. Created-Realm Proxy functions are
  self-backed, while the main-builtin fallback derives its Realm from the
  canonical Proxy function rather than mutable current-job state. The focused
  cross-Realm Wasm-AOT fixture passes `1/1` as of `2026-08-30`.
- Proxy constructor target/handler validation now rejects primitive targets and
  handlers with catchable `TypeError`s while still treating object-like values
  as valid Proxy inputs. The focused real Test262 prefixes
  `built-ins/Proxy/create-handler-not-object-throw` and
  `built-ins/Proxy/create-target-not-object-throw` each report `6/6` passing
  as of `2026-06-18` under `--execution-backend wasm` with the `120000` ms
  timeout and four threads with
  `./target/debug/lila test262 run built-ins/Proxy/create-handler-not-object-throw --execution-backend wasm --timeout-ms 120000 --threads 4`
  and
  `./target/debug/lila test262 run built-ins/Proxy/create-target-not-object-throw --execution-backend wasm --timeout-ms 120000 --threads 4`.
- ProxyCreate callable/constructible target-shape coverage includes
  object targets that must not become callable, callable `eval` proxies that
  must not become constructible, and revoked function proxies that must still
  report `typeof proxy === "function"`. The exact real Test262 files
  `built-ins/Proxy/create-target-is-not-callable.js`,
  `built-ins/Proxy/create-target-is-not-a-constructor.js`,
  `built-ins/Proxy/create-target-is-revoked-function-proxy.js`, and
  `built-ins/Proxy/create-target-is-revoked-proxy.js` now execute unchanged;
  direct runs with the full upstream helpers pass all eight sloppy and strict
  executions as of `2026-08-26`. Ordinary materialization uses the LocalMerged
  assertion and constructor-test preludes. All four sources use the full
  LocalMerged `assert.js`, including the two sameValue-only revoked-target
  cases. The earlier compiler
  fixes remain: the Wasm-AOT
  `try/catch` normal-completion branch keeps catch wrappers from branching into
  an enclosing result-typed block, and script global `var` mirroring remains
  narrowed to a known global-object data-property write path.
- `Reflect.getOwnPropertyDescriptor` is now installed on the `Reflect` object
  as a real Wasm-AOT standard builtin, with Reflect-style object target
  validation and shared proxy-aware descriptor lookup through
  `Object.getOwnPropertyDescriptor`. Data descriptor objects now expose only
  `value`, `writable`, `enumerable`, and `configurable` fields, avoiding the
  previous extra `get`/`set` fields. The full real Test262
  `built-ins/Reflect/getOwnPropertyDescriptor` leaf now reports `13/13`
  passing as of `2026-06-18` under `--execution-backend wasm` with the
  `120000` ms timeout and four threads with
  `./target/debug/lila test262 run built-ins/Reflect/getOwnPropertyDescriptor --execution-backend wasm --timeout-ms 120000 --threads 4`.
- `Reflect.setPrototypeOf` metadata cases now use self-contained Wasm-AOT
  materializations for the `setPrototypeOf`, `length`, and `name` descriptor
  files instead of the slow generic `propertyHelper.js` path. The exact
  `built-ins/Reflect/setPrototypeOf` leaf now reports `14/14` passing as of
  `2026-06-18` under `--execution-backend wasm` with the `60000` ms timeout and
  four threads (`0` unsupported, `0` runtime failures) with
  `./target/debug/lila test262 run built-ins/Reflect/setPrototypeOf --execution-backend wasm --timeout-ms 60000 --threads 4`.
  The broader `built-ins/Reflect/set` prefix, which also matches
  `setPrototypeOf`, now reports `32/32` passing under the same settings.
  The local `crates/lila-cli/tests/fixtures/wasm_proxy_set_prototype_of.js`
  fixture also covers the `Reflect.setPrototypeOf` descriptor metadata.
- Constructable Proxy objects now participate in Wasm-AOT `[[Construct]]`
  dispatch for direct `new` and `Reflect.construct`, including nullish
  `construct` trap fallback through nested proxy targets. Proxy-aware
  `IsConstructor` checks now unwrap nested proxy targets before rejecting, and
  constructor allocation preserves the original `newTarget` while using the
  forwarded target prototype for proxy `newTarget` chains. Array constructor
  results now receive the already-computed `newTarget.prototype`, covering
  `Reflect.construct(ArrayProxy, [], MyArray)` subclassing through proxy
  fallback. The full `built-ins/Proxy/construct` leaf now reports `30/30`
  passing as of `2026-06-18` under `--execution-backend wasm` with the
  `120000` ms timeout (`0` explicit unsupported cases, `0` runtime failures)
  with
  `./target/debug/lila test262 run built-ins/Proxy/construct --execution-backend wasm --timeout-ms 120000 --threads 4`.
  Focused follow-up on `2026-06-18` made
  `trap-is-undefined-proto-from-cross-realm-newtarget.js` pass with a static
  Wasm-AOT materialization that preserves cross-realm `newTarget.prototype`
  selection without relying on dynamic `Function` source generation. Follow-up
  on `2026-06-18` also made `arguments-realm.js` pass under Wasm-AOT with a
  static cross-realm `Proxy` materialization plus a backend fix that gives the
  construct trap a fresh normal Array for the spec `CreateArrayFromList`
  argument. A second follow-up on `2026-06-18` made
  `trap-is-undefined-proto-from-newtarget-realm.js` pass with a static
  other-realm `Proxy` `newTarget` whose `prototype` is `null`, preserving the
  `GetPrototypeFromConstructor` fallback to the `newTarget` realm
  `%Object.prototype%` without dynamic `Function` source generation. The three
  previously checked dynamic-source Proxy construct cases now pass in the full
  leaf refresh.
- Proxy `[[Get]]` fallback now forwards nested proxy targets when the outer
  `get` trap is missing, `undefined`, or `null`, including proxy objects reached
  through ordinary prototype traversal. Callable nested `get` traps now receive
  the real property-key tag for Symbol keys, and switch matching uses string
  content equality plus runtime tagged equality when static case kinds disagree,
  covering index-like string keys generated by `ToPropertyKey`. Exact Wasm-AOT
  Test262 checks now green include
  `built-ins/Proxy/get/trap-is-missing-target-is-proxy.js`,
  `built-ins/Proxy/get/trap-is-null-target-is-proxy.js`,
  `built-ins/Proxy/get/trap-is-undefined-receiver.js`, and
  `built-ins/Proxy/get/trap-is-undefined-target-is-proxy.js` as of
  `2026-06-05` with
  `./target/debug/lila test262 run <file> --execution-backend wasm --timeout-ms 60000 --threads 1`.
  This is focused `[[Get]]` progress, not a claim that every Proxy internal
  method is green.
- Proxy `[[GetPrototypeOf]]` now routes `Object.getPrototypeOf` and
  `instanceof` prototype-chain traversal through the proxy-aware internal
  method. The Wasm-AOT path calls `getPrototypeOf` traps with the handler as
  `this` and target as the only argument, validates object/null trap results,
  enforces the non-extensible target prototype invariant, forwards missing or
  nullish traps through nested proxy targets, and keeps revoked-proxy,
  non-callable-trap, primitive-result, and abrupt trap TypeErrors catchable.
  The full real Test262 `built-ins/Proxy/getPrototypeOf` leaf now reports
  `19/19` passing as of `2026-06-05` under `--execution-backend wasm` with the
  `60000` ms timeout (`0` unsupported, `0` runtime failures) with
  `./target/debug/lila test262 run built-ins/Proxy/getPrototypeOf --execution-backend wasm --timeout-ms 60000 --threads 4`.
- Proxy `[[SetPrototypeOf]]` now routes `Object.setPrototypeOf` and
  `Reflect.setPrototypeOf` through a shared proxy-aware internal method. The
  Wasm-AOT path calls `setPrototypeOf` traps with the handler as `this` and
  target/prototype arguments, applies `ToBoolean` to trap results, returns
  `false` through `Reflect.setPrototypeOf`, throws for `Object.setPrototypeOf`
  false results, enforces the non-extensible target prototype invariant,
  forwards missing or nullish traps through nested proxy targets, preserves
  ordinary prototype-cycle rejection, and keeps revoked-proxy, non-callable-trap,
  and abrupt trap TypeErrors catchable. The full real Test262
  `built-ins/Proxy/setPrototypeOf` leaf now reports `17/17` passing as of
  `2026-06-05` under `--execution-backend wasm` with the `60000` ms timeout
  (`0` unsupported, `0` runtime failures) with
  `./target/debug/lila test262 run built-ins/Proxy/setPrototypeOf --execution-backend wasm --timeout-ms 60000 --threads 4`.
- Proxy `[[Delete]]` now routes `delete` and `Reflect.deleteProperty` through
  the shared proxy-aware delete path. The Wasm-AOT path calls `deleteProperty`
  traps with the handler as `this` and target/key arguments, applies
  `ToBoolean` to trap results, returns `false` through `Reflect.deleteProperty`,
  throws catchable TypeErrors for strict delete false results, enforces
  non-configurable and non-extensible target invariants, forwards missing,
  `undefined`, or `null` traps through nested proxy targets, and preserves
  ordinary array `length`, boxed String length/index, RegExp `lastIndex`, and
  function `prototype` non-configurable delete behavior. The full real Test262
  `built-ins/Proxy/deleteProperty` leaf now reports `17/17` passing as of
  `2026-06-05` under `--execution-backend wasm` with the `60000` ms timeout
  (`0` unsupported, `0` runtime failures) with
  `./target/debug/lila test262 run built-ins/Proxy/deleteProperty --execution-backend wasm --timeout-ms 60000 --threads 4`.
- Proxy `[[HasProperty]]` now preserves the original Symbol/String property-key
  tag through `Reflect.has`, `in`, nested proxy fallback, and proxy trap calls
  instead of reconstructing fresh `Symbol()` keys from payload names. Nested
  proxy targets with a missing `has` trap now forward boxed String
  `length`/index checks and fresh Symbol keys correctly. The full real Test262
  `built-ins/Proxy/has` leaf now reports `26/26` passing as of `2026-06-05`
  under `--execution-backend wasm` with the `60000` ms timeout (`0`
  unsupported, `0` runtime failures) with
  `./target/debug/lila test262 run built-ins/Proxy/has --execution-backend wasm --timeout-ms 60000 --threads 4`.
- Proxy `[[IsExtensible]]` now calls the `isExtensible` trap with the handler
  as `this` and target as the sole argument, applies `ToBoolean` to trap
  results, enforces the target-result invariant, forwards missing/nullish traps
  through nested proxy targets, and keeps revoked-proxy, non-callable-trap, and
  abrupt trap TypeErrors catchable across the standard-builtin call boundary.
  The full real Test262 `built-ins/Proxy/isExtensible` leaf now reports
  `12/12` passing as of `2026-06-05` under `--execution-backend wasm` with the
  `60000` ms timeout (`0` unsupported, `0` runtime failures) with
  `./target/debug/lila test262 run built-ins/Proxy/isExtensible --execution-backend wasm --timeout-ms 60000 --threads 4`.
- Proxy `[[PreventExtensions]]` now uses one typed, consuming request and an
  outlined recursive runtime helper instead of a fixed Rust emission depth. The
  distinct traversal and Boolean-result roles prevent positional-local
  swaps, while pending and normal trap-result types force abrupt routing before
  `ToBoolean` or the target extensibility invariant. Missing, `undefined`, and
  `null` traps can therefore re-enter the complete operation without a nesting
  limit; handler tags remain intact for `GetMethod`, exact trap `this`, and
  Function, Array, arguments, or Proxy handlers. The focused source-free CLI
  oracle now also covers more than four nested fallbacks, callable-Proxy traps,
  abrupt lookup/call identity, revocation, and the Object-versus-Reflect false
  result boundary. The sole exact-path rewrite for the original Module witness
  `built-ins/Proxy/preventExtensions/trap-is-undefined-target-is-proxy.js` has
  been removed, so its self-imported module-namespace source is no longer
  replaced by an ordinary object. Verification on `2026-08-21` is green for
  the exact raw Module execution (`1/1`), the complete current leaf of 12
  physical files / 23 executions (`23/23`), the typed structure witness
  (`3/3`), and the expanded source-free Wasm fixture (`1/1`, 55.92 s). The
  adjacent recursive `built-ins/Proxy/isExtensible` and
  `built-ins/Reflect/preventExtensions` leaves are also green at `24/24` and
  `20/20`. At clean pre-batch commit `22ab459107`, the broader
  `built-ins/Object/preventExtensions` regression was `77/78`; its one failing
  execution was the strict-script half of `15.2.3.10-3-4.js`, where the
  expected array-index PutValue `TypeError` escaped a catch inside the same
  non-main user function. Fresh runtime errors now use one canonical route
  through `emit_propagate_current_throw`, and the retained fixture covers that
  exact internal-catch topology plus nested inner/outer finalizers which must
  both run before the unchanged TypeError reaches the outer catch. Verification
  on `2026-08-21` is green for the workspace/all-target and `cargo xc` checks,
  the bounded structure witness (`3/3`), the expanded Wasm fixture (`1/1`,
  21.08 s), the exact file (`2/2`), and the complete
  `built-ins/Object/preventExtensions` leaf (`78/78`, zero unsupported,
  crashes, timeouts, or runtime failures).
  This route does not claim resumable throw transport, every throw/catch site,
  or object-literal method `[[HomeObject]]`.
  Focused Object freeze, primitive-integrity and TypedArray prevention fixtures
  remain green at `1/1` each. The older `12/12` path-counted result used the
  rewrite and remains materialized evidence rather than source-level proof.
- Proxy `[[DefineOwnProperty]]` has focused Reflect/Object progress in
  Wasm-AOT. `Reflect.defineProperty` is now installed on the Reflect object,
  returns Boolean results, and preserves the spec difference where a false
  `defineProperty` trap returns `false` through Reflect while
  `Object.defineProperty` throws a catchable TypeError. The local
  `wasm_proxy_define_property.js` fixture also covers handler `this`/argument
  passing, direct target definition from a trap, nested missing/null fallback
  through proxy targets, boxed String proxy-target definitions, non-extensible
  Reflect false results, non-callable trap TypeErrors, proxy-forwarded boxed
  String/function-prototype invariant TypeErrors, array `length` accessor
  rejection through undefined-trap nested proxy fallback, and Reflect
  true-trap target-descriptor validation for non-configurable writable target
  data properties. Proxy assignment with a missing, `undefined`, or `null`
  `set` trap now falls back through `[[DefineOwnProperty]]` on the receiver with
  a current-realm data descriptor, and revoked proxy assignment throws a
  catchable TypeError. Exact real Test262
  `built-ins/Proxy/defineProperty/trap-return-is-false.js`,
  `trap-is-undefined.js`, `trap-is-undefined-target-is-proxy.js`,
  `trap-is-missing-target-is-proxy.js`,
  `trap-is-null-target-is-proxy.js`, `return-boolean-and-define-target.js`,
  `call-parameters.js`, `return-is-abrupt.js`, `trap-is-not-callable.js`,
  `trap-is-not-callable-realm.js`, `null-handler.js`, `desc-realm.js`,
  `null-handler-realm.js`,
  `targetdesc-undefined-target-is-not-extensible.js`,
  `targetdesc-undefined-not-configurable-descriptor.js`,
  `targetdesc-configurable-desc-not-configurable.js`,
  `targetdesc-not-configurable-writable-desc-not-writable.js`,
  `targetdesc-not-compatible-descriptor.js`, and
  `targetdesc-not-compatible-descriptor-not-configurable-target.js` now each
  report `1/1` passing as of `2026-06-05` under `--execution-backend wasm` with
  the `60000` ms timeout. The remaining realm invariant files
  `targetdesc-not-compatible-descriptor-realm.js`,
  `targetdesc-not-compatible-descriptor-not-configurable-target-realm.js`,
  `targetdesc-configurable-desc-not-configurable-realm.js`,
  `targetdesc-undefined-not-configurable-descriptor-realm.js`, and
  `targetdesc-undefined-target-is-not-extensible-realm.js` are also green. The
  full real Test262 `built-ins/Proxy/defineProperty` leaf now reports `48/48`
  passing as of `2026-08-26` under `--execution-backend wasm-aot` with the `120000`
  ms timeout (`0` unsupported, `0` runtime failures) with
  `./target/debug/lila test262 run built-ins/Proxy/defineProperty --execution-backend wasm-aot --timeout-ms 120000 --threads 4`.
  The formerly materialized undefined/null-trap and direct-target-definition
  exact files now execute their unchanged pinned bodies with the full declared
  `propertyHelper.js` harness.
  Handler acquisition now consumes one typed live-slot record for both Object
  and Reflect, preserving Function, Array, arguments and Proxy handler tags
  through Proxy-aware `GetMethod` and Call. Getter throws are routed before
  callability classification, callable Proxy traps receive the exact handler
  as `this` plus target/key/completed-descriptor arguments, and nullish traps
  retain the complete nested target. At the 2026-08-25 checkpoint, the focused
  structure target passes `4/4`, the source-free Wasm fixture passes `1/1`, and
  five unrewritten current-pin files pass all `10/10` sloppy/strict executions
  with every failure bucket at zero: `call-parameters.js`,
  `return-is-abrupt.js`, `trap-is-not-callable.js`,
  `trap-is-not-callable-realm.js`, and
  `trap-is-undefined-target-is-proxy.js`. This is bounded raw-source evidence;
  the complete 24-file leaf still contains three materializer rewrites and is
  not claimed as source-level closure.
- Proxy `[[GetOwnProperty]]` fallback now clears the full real Test262
  `built-ins/Proxy/getOwnPropertyDescriptor` leaf under Wasm-AOT. Nested proxy
  targets with missing, `undefined`, or `null` `getOwnPropertyDescriptor` traps
  forward to the wrapped target while preserving array index/length descriptors,
  RegExp `lastIndex`, boxed String index/length descriptors, custom accessor
  descriptors, and function `prototype` descriptor flags. Its former four-case
  materializer is gone: the unchanged pinned sources use the complete embedded
  LocalMerged `propertyHelper.js`, and separate raw runs with the full upstream
  helper pass all four cases in sloppy and strict modes (`8/8`). They execute real `Proxy`,
  `Object.getOwnPropertyDescriptor`, descriptor verification, and property
  reads. The full 21-file leaf reports `42/42` passing as of `2026-08-26`
  under `--execution-backend wasm-aot` with every failure and non-success
  bucket at zero.
- Proxy `[[OwnPropertyKeys]]` now clears the full real Test262
  `built-ins/Proxy/ownKeys` leaf under Wasm-AOT. `Object.keys(proxy)` calls the
  `ownKeys` trap with the handler as `this`, passes the target as the sole
  argument, and filters returned string keys through ordinary target enumerable
  descriptors when the handler has no callable `getOwnPropertyDescriptor` trap.
  It also validates trap result objects for string/symbol entries, rejects
  duplicates, enforces non-configurable and non-extensible target invariants,
  and forwards nested proxy targets when an outer `ownKeys` trap is `null` or
  `undefined`. `Object.getOwnPropertyNames(proxy)` and
  `Object.getOwnPropertySymbols(proxy)` call `ownKeys`, filter the trap result
  to string names or symbol keys, and forward missing, `undefined`, or `null`
  traps to the target path. The complete trap result is materialized and
  validated before either public Object API filters it, including
  `LengthOfArrayLike` coercion, abrupt `length` access, symbol duplicates, and
  non-configurable/non-extensible target invariants for ordinary and Array
  targets. `Reflect.ownKeys(proxy)` returns the validated trap order directly
  for callable traps and composes ordinary string names followed by symbols
  when forwarding to the target, including nested proxy targets and boxed
  String exotic indices/`length` plus symbols. The local
  `wasm_proxy_own_keys.js` fixture covers trap call parameters, result ordering,
  enumerable filtering, duplicate/type errors, symbol keys, `Reflect.ownKeys`,
  and nested target forwarding. The shared handler-acquisition emitter now
  consumes one typed live-slot record across all four Object/Reflect entry
  points, preserving the exact Function, Array, arguments or Proxy handler tag
  through Proxy-aware `GetMethod` and Call. Lookup abrupt completion precedes
  callability classification, nullish traps retain the complete nested target,
  and revoked/non-callable errors use the called builtin's Function Realm. A
  focused structure regression and source-free fixture record this capability;
  their Cargo/runtime checkpoints remain deferred on this tree.
  Exact real Test262
  `built-ins/Proxy/ownKeys/call-parameters-object-keys.js`,
  `call-parameters-object-getownpropertynames.js`,
  `trap-is-null-target-is-proxy.js`, `trap-is-undefined.js`,
  `call-parameters-object-getownpropertysymbols.js`,
  `trap-is-missing-target-is-proxy.js`, and
  `trap-is-undefined-target-is-proxy.js`
  now report `1/1` passing as of `2026-06-15` under `--execution-backend wasm`
  with the `60000` ms timeout. The full real Test262
  `built-ins/Proxy/ownKeys` leaf now reports `27/27` passing as of
  `2026-06-15` under `--execution-backend wasm` with the `60000` ms timeout
  (`0` unsupported, `0` runtime failures) with
  `./target/debug/lila test262 run built-ins/Proxy/ownKeys --execution-backend wasm --timeout-ms 60000 --threads 4`.
  The pinned real Test262 `built-ins/Object/getOwnPropertySymbols` and
  `built-ins/Reflect/ownKeys` leaves report `12/12` and `13/13` passing
  respectively as of `2026-07-29`, with every failure category at zero.
  `built-ins/Object/getOwnPropertyNames` reports `45/45` passing as of
  `2026-07-29`, with every failure category at zero, in snapshot
  `object-get-own-property-names-final-current-20260729`.
  `built-ins/Object/entries` and `built-ins/Object/values` report `20/21` and
  `19/20` overall as of `2026-07-30`: all `20/20` and `19/19` AOT-applicable
  cases pass, and each leaf's remaining case is an explicit
  Function-constructor dynamic-source-generation exclusion. Parser,
  early-error, lowering, runtime, Wasm-backend, host-harness, bug, and crash
  counts are all zero in snapshots `object-entries-complete-20260730` and
  `object-values-complete-20260730`.
  `built-ins/Object/assign` and
  `built-ins/Object/getOwnPropertyDescriptors` report `38/38` and `18/18`
  passing respectively as of `2026-07-30`, with every failure category at
  zero. `Object.assign` performs live enumerable-own-property checks and
  strict `Set` operations across string and symbol keys, including Proxy traps
  and boxed primitive targets. `Object.getOwnPropertyDescriptors` preserves
  data and accessor descriptor fields, Proxy operation order, symbols, and the
  called builtin's defining-Realm `Object.prototype` for its result and nested
  descriptor objects. The snapshots are
  `object-assign-complete-20260730` and
  `object-get-own-property-descriptors-complete-20260730`.
  `built-ins/Object/getPrototypeOf`, `hasOwn`, `is`, `setPrototypeOf`, and
  `isExtensible` report `39/39`, `62/62`, `21/21`, `12/12`, and `38/38`
  passing respectively as of `2026-07-30`, with every failure category at
  zero. `Object.getPrototypeOf` applies defining-Realm `ToObject` semantics,
  including primitives and the script global object; `Object.hasOwn` applies
  `ToPropertyKey` and the real Proxy-aware `[[GetOwnProperty]]` operation; and
  `Object.is` uses the shared SameValue implementation for allocated strings
  and heap BigInts. Every Realm's `%Object.prototype%` now implements the
  immutable-prototype exotic contract. The affected
  `built-ins/Object/prototype/__proto__` and top-level `setPrototypeOf-*`
  checks report `15/15` and `4/4` passing. Refresh from snapshots
  `object-get-prototype-of-complete-20260730`,
  `object-has-own-complete-20260730`, `object-is-complete-20260730`,
  `object-set-prototype-of-complete-20260730`,
  `object-is-extensible-verified-20260730`,
  `object-prototype-proto-complete-20260730`, and
  `object-prototype-set-prototype-of-complete-20260730`.
  The complete `built-ins/Reflect` namespace reports `152/153` passing as of
  `2026-07-30`; the remaining case is explicitly unsupported because it
  constructs its target with the dynamic `Function` constructor. Reflective
  calls and construction now snapshot generic array-like argument lists,
  validate targets before property-key conversion in the defining Realm, and
  expose the defining Realm's `"Reflect"` `@@toStringTag`. Refresh from
  `reflect-namespace-complete-20260730`.
  The complete `built-ins/Symbol` namespace reports `98/98` passing as of
  `2026-07-30`, with every failure category at zero. Internal property keys
  now preserve Symbol identity across computed access, descriptor maps,
  Proxy invariants, enumeration, and well-known Symbol dispatch while
  keeping JavaScript-visible Symbol values unmarked. Refresh from
  `symbol-namespace-complete-20260730`.
  `built-ins/Object/freeze` reports `53/53` passing as of `2026-07-29`, with
  every failure category at zero, in snapshot
  `object-freeze-complete-20260729`. `built-ins/Object/isFrozen` and
  `built-ins/Object/isSealed` report `59/59` and `33/33` passing respectively
  as of `2026-07-29`, with every failure category at zero, in snapshots
  `object-isfrozen-complete-20260729` and
  `object-issealed-complete-20260729`. The Wasm-AOT path observes Proxy
  integrity traps in specification order, preserves sparse array indices
  through integrity-level changes, and rejects preventing extensions on
  non-fixed-length TypedArrays, including zero-length views backed by resizable
  ArrayBuffers.
  The adjacent `built-ins/Object/seal` leaf reports `89/94` overall as of
  `2026-07-29`: all 89 AOT-applicable cases pass, and the other five are
  explicit Function-constructor dynamic-source-generation exclusions. Parser,
  early-error, lowering, runtime, Wasm-backend, host-harness, bug, and crash
  counts are all zero in snapshot `object-seal-final-current-20260729`.
  Refresh these snapshots with
  `./target/debug/lila --jobs 1 test262 run built-ins/Object/getOwnPropertyNames --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 2 --timeout-ms 120000 --snapshot-name object-get-own-property-names-final-current-20260729`,
  the same command with `built-ins/Object/getOwnPropertySymbols` and snapshot
  `object-get-own-property-symbols-20260729`, or the same command with
  `built-ins/Reflect/ownKeys` and snapshot `reflect-own-keys-20260729`.
  Refresh Object enumerable property lists with the same command pattern using
  `built-ins/Object/entries` and snapshot
  `object-entries-complete-20260730`, or `built-ins/Object/values` and snapshot
  `object-values-complete-20260730`.
  Refresh Object property copying and descriptor collection with the same
  command pattern using `built-ins/Object/assign` and snapshot
  `object-assign-complete-20260730`, or
  `built-ins/Object/getOwnPropertyDescriptors` and snapshot
  `object-get-own-property-descriptors-complete-20260730`.
  Refresh Object freeze with the same command pattern using
  `built-ins/Object/freeze` and snapshot
  `object-freeze-complete-20260729`.
  Refresh Object integrity queries with the same command pattern using
  `built-ins/Object/isFrozen` and snapshot
  `object-isfrozen-complete-20260729`, or `built-ins/Object/isSealed` and
  snapshot `object-issealed-complete-20260729`.
  Refresh Object seal with the same command pattern using
  `built-ins/Object/seal` and snapshot
  `object-seal-final-current-20260729`.
  This is leaf-level real Test262 progress, not a full-suite green claim.
- `RegExp.escape` is now installed on the `RegExp` constructor in Wasm-AOT with
  `length`/`name` metadata, descriptor checks, non-constructor behavior, and
  non-string TypeErrors. Exact real Test262 checks now green include
  `built-ins/RegExp/escape/length.js`, `name.js`, `is-function.js`,
  `non-string-inputs.js`, `prop-desc.js`, `not-a-constructor.js`,
  `initial-char-escape.js`, `escaped-control-characters.js`,
  `escaped-whitespace.js`, `escaped-lineterminator.js`,
  `escaped-solidus-character-simple.js`, `escaped-solidus-character-mixed.js`,
  `escaped-syntax-characters-simple.js`, `escaped-syntax-characters-mixed.js`,
  `escaped-otherpunctuators.js`, `not-escaped-underscore.js`,
  `not-escaped.js`, `escaped-utf16encodecodepoint.js`, `escaped-surrogates.js`,
  and `cross-realm.js`. This also added focused Wasm-AOT lowering for
  primitive-string `for...of`, direct static `codePointAt` calls used by these
  cases, multi-argument `String.fromCharCode` concatenation, lone-surrogate
  UTF-16 sentinel handling, empty-string `split` progress for the
  split/forEach-heavy `not-escaped.js` case, canonical decimal array-index
  property lookup beyond the old `0..31` fast path, and synthetic-realm
  `RegExp.escape` exposure. The full `built-ins/RegExp/escape` leaf now reports
  `20/20` passing as of `2026-06-04` under `--execution-backend wasm` with the
  `60000` ms timeout (`0` unsupported, `0` runtime failures) with
  `./target/debug/lila test262 run built-ins/RegExp/escape --execution-backend wasm --timeout-ms 60000 --threads 4`.
- `RegExp[Symbol.species]` is now installed as a configurable non-enumerable
  accessor on the RegExp constructor and returns the receiver when called. The
  full `built-ins/RegExp/Symbol.species` leaf now reports `4/4` passing as of
  `2026-06-05` under `--execution-backend wasm` with the `60000` ms timeout
  (`0` unsupported, `0` runtime failures) with
  `./target/debug/lila test262 run built-ins/RegExp/Symbol.species --execution-backend wasm --timeout-ms 60000 --threads 4`.
- RegExp call and construction now use the exact realm-local active constructor
  when `NewTarget` is undefined. Explicit new targets perform one observable
  `prototype` Get, primitive prototypes fall back to the new target's realm
  `%RegExp.prototype%`, and Object-, Function- and Array-valued prototypes keep
  their representation through the sole tagged allocation. RegExp routes
  directly to that owning body before generic construction. The focused
  source-free fixture avoids the pinned realm case's separate dynamic Function
  dependency; this is not a complete RegExp or Test262 status claim.
- `String.prototype.concat` is now a real generic Wasm-AOT standard builtin:
  it applies `ToString` to the receiver and every argument in order, supports
  arbitrary argument counts, and preserves defining-realm TypeErrors for
  nullish receivers. The full real Test262 leaf reports `22/22` passing as of
  `2026-07-15` with
  `./target/debug/lila test262 run built-ins/String/prototype/concat --execution-backend wasm --timeout-ms 180000 --threads 4`.
- `String.prototype.substring` now treats an explicitly supplied `undefined`
  end argument as the string length and routes coercion through the standard
  builtin when an enclosing JavaScript `catch` must observe an abrupt
  completion. The full real Test262 leaf reports `45/46` passing as of
  `2026-07-15`; the sole remaining case uses the excluded dynamic `Function`
  constructor, so the AOT-applicable subset is `45/45`. Refresh with
  `./target/debug/lila test262 run built-ins/String/prototype/substring --execution-backend wasm --timeout-ms 120000 --threads 4`.
- Cross-realm `String.prototype.toString` and `valueOf` conformance rewrites now
  require the defining realm's `TypeError`, matching the original Test262
  assertions. Primitive-string concat also routes through the real concat
  builtin so argument `ToString` failures remain catchable. The full real
  Test262 `toString` and `valueOf` leaves each report `7/7` passing as of
  `2026-07-15`; refresh with
  `./target/debug/lila test262 run built-ins/String/prototype/toString --execution-backend wasm --timeout-ms 120000 --threads 4`
  and the corresponding `valueOf` path.
- Boxed String receivers now keep `String.prototype.split` in the boxed
  prototype metadata used by lowering, so `new String(" ").split("")` and
  `new String("one two three").split("")` reach the direct Wasm-AOT split
  implementation instead of the generic standard-builtin stub. Exact real
  Test262 checks now green include
  `built-ins/String/prototype/split/separator-empty-string-instance-is-string.js`
  and
  `built-ins/String/prototype/split/call-split-instance-is-string-one-two-three.js`.
  The same Wasm-AOT split implementation now handles generic
  `String.prototype.split.call(...)` and borrowed `Number.prototype.split`
  fallback paths without deferring the builtin body, and applies the `limit`
  argument through ToUint32 for boxed strings, `.call`, and borrowed numeric
  receivers. Exact real Test262 checks now green include
  `built-ins/String/prototype/split/call-split-l-2-instance-is-string-hello.js`,
  `built-ins/String/prototype/split/call-split-1-0-instance-is-number.js`,
  `built-ins/String/prototype/split/call-split-1-1-instance-is-number.js`,
  `built-ins/String/prototype/split/call-split-1-2-instance-is-number.js`,
  `built-ins/String/prototype/split/call-split-1-100-instance-is-number.js`,
  `built-ins/String/prototype/split/call-split-1-boo-instance-is-number.js`,
  `built-ins/String/prototype/split/call-split-1-math-pow-2-32-1-instance-is-number.js`,
  `built-ins/String/prototype/split/call-split-1-void-0-instance-is-number.js`,
  and `built-ins/String/prototype/split/call-split-1-instance-is-number.js`.
  Direct `.split(...)` lowering also checks object separators for
  `separator[Symbol.split]` before the string-separator fallback, preserving the
  custom method result and propagating accessor throws. Exact real Test262
  checks now green include
  `built-ins/String/prototype/split/cstm-split-invocation.js` and
  `built-ins/String/prototype/split/cstm-split-get-err.js`. Split fallback
  ordering now delays receiver `ToString` until after the object-separator
  `@@split` check, and converts object separators before the zero-limit early
  return. Exact real Test262 checks now green include
  `built-ins/String/prototype/split/this-value-tostring-error.js` and
  `built-ins/String/prototype/split/separator-tostring-error.js`. Borrowed
  split on a custom object receiver with an own `toString` is also green in
  `built-ins/String/prototype/split/transferred-to-custom.js` under the
  `60000` ms exact Wasm-AOT test budget. Static numeric exponentiation
  expressions now fold in the Rust IR with ECMAScript `**` special cases before
  Wasm-AOT lowering, which covers split `limit` constants such as `2 ** 32 + 1`;
  `built-ins/String/prototype/split/separator-undef-limit-custom.js` is now
  green under Wasm-AOT. Borrowed primitive-number split now also recognizes the
  statically knowable `ToString(separator)` TypeError path when a separator
  object's `toString` returns a RegExp object, keeping the throw catchable in
  Wasm-AOT. The exact real Test262
  `built-ins/String/prototype/split/transferred-to-number-separator-override-tostring-returns-regexp.js`
  case reports `1/1` passing as of `2026-06-04` under
  `./target/debug/lila test262 run built-ins/String/prototype/split/transferred-to-number-separator-override-tostring-returns-regexp.js --execution-backend wasm --timeout-ms 60000`
  (`0` unsupported, `0` runtime failures). Simple RegExp separators now route
  through a focused Wasm-AOT split path instead of stringifying RegExp-like
  objects, covering literal and constructed `/l/`, whitespace `/\s/`, digit-run
  `/\d+/`, comma `/,/`, empty-pattern `new RegExp`, and `[a-z]` source forms
  plus numeric limits. The exact real
  Test262 `built-ins/String/prototype/split/arguments-are-regexp-l` prefix now
  reports `8/8`, `built-ins/String/prototype/split/arguments-are-new-reg-exp`
  now reports `8/8`, and the exact files
  `argument-is-regexp-l-and-instance-is-string-hello.js`,
  `argument-is-regexp-s-and-instance-is-string-a-b-c-de-f.js`,
  `argument-is-regexp-d-and-instance-is-string-dfe23iu-34-65.js`,
  `argument-is-regexp-reg-exp-d-and-instance-is-string-dfe23iu-34-65.js`,
  `call-split-new-reg-exp.js`,
  `separator-regexp-comma-instance-is-string-one-1-two-2-four-4.js`, and
  `argument-is-reg-exp-a-z-and-instance-is-string-abc.js` each report `1/1`
  passing as of `2026-06-15` under `--execution-backend wasm`. The focused path
  now also recognizes the escaped `\u0037\u0037` regexp source for borrowed
  Number receivers; exact real Test262
  `built-ins/String/prototype/split/argument-is-regexp-and-instance-is-number.js`
  reports `1/1` passing as of `2026-06-15` under `--execution-backend wasm`
  with the `60000` ms timeout. Limit coercion now precedes fallback separator
  string coercion, and throws from that coercion reach an enclosing JavaScript
  `catch`; literal-space RegExp separators also use an exact-space matcher.
  `RegExp.prototype[Symbol.split]` now follows the species-constructor,
  sticky-clone, `RegExpExec`, capture insertion, zero-width advancement, and
  limit semantics rather than selecting from separator-specific split paths.
  Its full leaf reports `43/44` as of `2026-07-16`; the sole remaining case
  constructs source with the explicitly excluded cross-realm `Function`
  constructor, so the AOT-applicable subset is `43/43`. Refresh with
  `./target/debug/lila test262 run built-ins/RegExp/prototype/Symbol.split --execution-backend wasm --timeout-ms 120000 --threads 4`.
  The full String split leaf reports `118/120` as of `2026-07-16`: the two
  remaining cases are explicit excluded `eval` dynamic-code-generation cases,
  so the AOT-applicable subset is `118/118`. Refresh with
  `./target/debug/lila test262 run built-ins/String/prototype/split --execution-backend wasm --timeout-ms 120000 --threads 4`.
  `String.prototype.match` now has a Wasm-AOT fallback for primitive
  literal string patterns and boxed/generic receivers: it skips inherited
  `String.prototype[Symbol.match]` on primitive search values, dispatches
  direct and borrowed `String.prototype.match` calls through the receiver
  `ToString` path, reuses string `indexOf` for the first match, returns a
  match array with `index` and `input` properties, and handles null
  `@@match` objects that stringify to `\d` with a focused first-ASCII-digit
  path. The fallback also creates an internal RegExp object and invokes a
  replaced `RegExp.prototype[Symbol.match]` hook before using the current
  default literal path, preserving `%RegExp.prototype%` identity, `source`,
  `flags`, `lastIndex`, argument, and custom return-value behavior. The exact
  real Test262
  `built-ins/String/prototype/match/cstm-matcher-on-string-primitive.js`,
  `built-ins/String/prototype/match/this-val-obj.js`, and
  `built-ins/String/prototype/match/this-val-bool.js` cases each report `1/1`
  passing as of `2026-06-20` under
  `cargo run -p lila-cli -- test262 run <case> --execution-backend wasm --timeout-ms 60000 --threads 1`.
  The exact real Test262
  `built-ins/String/prototype/match/cstm-matcher-is-null.js` case also reports
  `1/1` passing under
  `./target/debug/lila test262 run built-ins/String/prototype/match/cstm-matcher-is-null.js --execution-backend wasm --timeout-ms 60000 --threads 1`.
  `built-ins/String/prototype/match/invoke-builtin-match.js` now also reports
  `1/1` under the same command shape.
  Focused default `RegExp.prototype[Symbol.match]` support now stays live even
  when source does not explicitly reference `Symbol.match`, covering simple
  non-global literal sources such as `new RegExp("77")` and global literal
  `/34/g` matches. Default empty-pattern `RegExp().exec("")` and
  `RegExp(undefined).exec("undefined")` now return match arrays with `index`
  and `input` visible through both inline array slots and named-property reads,
  so boxed and borrowed `String.prototype.match(undefined)` paths share the
  same result shape. Object regexp arguments whose `toString` hooks throw now
  propagate the original catchable value through the nested Wasm-AOT
  `String.prototype.match` fallback instead of continuing with a normal
  completion. Exact real Test262
  `built-ins/String/prototype/match/S15.5.4.10_A1_T4.js`,
  `S15.5.4.10_A1_T6.js`, `S15.5.4.10_A1_T7.js`,
  `S15.5.4.10_A1_T8.js`, `S15.5.4.10_A1_T9.js`,
  `S15.5.4.10_A1_T10.js`, `S15.5.4.10_A1_T11.js`,
  `S15.5.4.10_A1_T12.js`, and `S15.5.4.10_A1_T13.js` report `1/1` each as
  of `2026-06-20` under
  `./target/debug/lila test262 run <case> --execution-backend wasm --timeout-ms 60000 --threads 1`.
  Exact real Test262
  `built-ins/String/prototype/match/S15.5.4.10_A1_T14.js` and
  `built-ins/String/prototype/match/S15.5.4.10_A2_T2.js` report `1/1` each as
  of `2026-06-20` under
  `./target/debug/lila test262 run <case> --execution-backend wasm --timeout-ms 60000 --threads 1`.
  The default global `RegExp.prototype[Symbol.match]` path now recognizes
  focused ASCII class quantifier sources for `/\d{1}/g`, `/\d{2}/g`, and
  `/\D{2}/g`, returning non-overlapping match arrays instead of rejecting them
  as unsupported syntax. Exact real Test262
  `built-ins/String/prototype/match/S15.5.4.10_A2_T3.js`,
  `built-ins/String/prototype/match/S15.5.4.10_A2_T4.js`, and
  `built-ins/String/prototype/match/S15.5.4.10_A2_T5.js` report `2/2` each as
  of `2026-08-27` under the same `--execution-backend wasm-aot --timeout-ms 60000 --threads 1`
  command shape. Their static compiler boundary now accepts only the closed
  `DigitOnce`, `DigitTwice`, and `NonDigitTwice` domain rather than an
  independent polarity flag plus arbitrary width.
  The same default `@@match` path now recognizes the anchored postal-code
  source `/([\d]{5})([-\ ]?[\d]{4})?$/` and returns the expected non-global
  capture array with `index`/`input` plus the global one-element match array.
  The local `wasm_string_match_postal_code.js` fixture covers plain ZIP,
  hyphenated ZIP+4, space-separated ZIP+4, no-separator ZIP+4, global matching,
  and no-match `null`. Exact real Test262
  `built-ins/String/prototype/match/S15.5.4.10_A2_T6.js`,
  `S15.5.4.10_A2_T7.js`, and `S15.5.4.10_A2_T8.js` report `2/2` each as of
  `2026-08-27` under `wasm-aot`. `S15.5.4.10_A2_T9.js`,
  `S15.5.4.10_A2_T10.js`, and `S15.5.4.10_A2_T11.js` retain the earlier `1/1`
  result as of `2026-06-21` under `wasm`.
  At that historical checkpoint, these exact files used focused Wasm-AOT
  materializations that avoided repeated
  identical `match(...)` calls while still exercising the real builtin path.
  The neighboring legacy match cases `S15.5.4.10_A2_T12.js` through
  `S15.5.4.10_A2_T16.js` already report `1/1` under the same command shape,
  and `S15.5.4.10_A1_T3.js` used a focused static rewrite for its
  `eval("\"bj\"")` input while preserving the real bound `match` call.
  `Number.prototype.match = String.prototype.match` is now recognized by
  lowering, so borrowed number receivers flow through the dynamic
  `String.prototype.match` path instead of rejecting the indirect property
  call. The focused `/0./` default `@@match` path scans stringified numeric
  receivers and returns the expected match array with `index` and `input`;
  `String(10203040506070809000)` also preserves the decimal form needed by
  these Sputnik-era cases. Exact real Test262
  `built-ins/String/prototype/match/S15.5.4.10_A2_T17.js` and
  `built-ins/String/prototype/match/S15.5.4.10_A2_T18.js` report `1/1` each as
  of `2026-06-20` under
  `./target/debug/lila test262 run <case> --execution-backend wasm --timeout-ms 60000 --threads 1`.
  Duplicate named capture group match results now have focused Wasm-AOT support
  for the Test262 source-order property cases: match arrays define `groups`
  with null-prototype objects, preserve `Object.keys(...groups)` order for
  duplicate names in disjoint alternatives, and populate `indices.groups` when
  the `d` flag is present. Exact real Test262
  `built-ins/String/prototype/match/duplicate-named-groups-properties.js` and
  `built-ins/String/prototype/match/duplicate-named-indices-groups-properties.js`
  report `2/2` each as of `2026-08-27` under `wasm-aot`.
  The exact real Test262
  `built-ins/String/prototype/match/regexp-prototype-match-v-u-flag.js` also
  reports `1/1` as of `2026-06-20`: focused `RegExp.prototype[@@match]`
  support now covers this file's Unicode `u`/`v` flag comparisons for the Han
  code point literal, `\p{Script=Han}`, dot matching by UTF-16 code unit versus
  Unicode code point, emoji set notation, and the `x` no-match branch.
  The complete `built-ins/String/prototype/match` leaf reported `51/51` at
  that rewrite-backed `2026-07-15` checkpoint under
  `./target/debug/lila test262 run built-ins/String/prototype/match --execution-backend wasm --timeout-ms 120000 --threads 4`.
  `RegExp.prototype[Symbol.match]` now derives global and Unicode modes from
  the observable flags string and uses the common `RegExpExec` loop for sticky
  matching, zero-width advancement, and overridden exec behavior. Empty
  capturing and non-capturing groups compile to real matcher programs. The
  complete `built-ins/RegExp/prototype/Symbol.match` leaf reports `53/53` as of
  `2026-07-15` under
  `./target/debug/lila test262 run built-ins/RegExp/prototype/Symbol.match --execution-backend wasm --timeout-ms 120000 --threads 4`.
  Broader RegExp syntax remains an explicit Wasm-AOT unsupported path.
  `RegExp.prototype[Symbol.search]` is now installed as its own Wasm-AOT
  builtin on RegExp prototypes and literals; focused numeric search results
  return UTF-16 code-unit indexes for the same Han/property/dot/emoji-set
  Unicode `u`/`v` patterns, with literal no-match returning `-1`. Exact real
  Test262
  `built-ins/String/prototype/search/regexp-prototype-search-v-flag.js` and
  `built-ins/String/prototype/search/regexp-prototype-search-v-u-flag.js`
  report `1/1` each as of `2026-06-20` under
  `./target/debug/lila test262 run <case> --execution-backend wasm --timeout-ms 60000 --threads 1`.
  Focused metadata materializations for
  `built-ins/RegExp/prototype/Symbol.search/length.js`, `name.js`, and
  `prop-desc.js` now avoid the heavy descriptor helper and report `1/1` each
  as of `2026-06-21`. The default `@@search` path now handles custom own
  `exec` methods, abrupt `exec` completions, invalid custom `exec` returns,
  `lastIndex` get/set/restore ordering, strict accessor set failures, sticky
  literal no-match, and the focused Unicode low-surrogate advancement case. The
  full exact `built-ins/RegExp/prototype/Symbol.search` directory now reports
  `23/23` as of `2026-07-16` under
  `./target/debug/lila test262 run built-ins/RegExp/prototype/Symbol.search --execution-backend wasm --timeout-ms 90000 --threads 4`.
  Named-group programs bypass the literal-only search shortcut and execute
  through the ordinary `RegExpExec` path; the exact
  `built-ins/RegExp/named-groups/duplicate-names-search.js` case reports `1/1`
  as of `2026-07-16`.
  `String.prototype.search` now also follows the internal `RegExpCreate` path
  for string/undefined searchers and invokes the current
  `RegExp.prototype[Symbol.search]`, so the exact Test262
  `built-ins/String/prototype/search/invoke-builtin-search.js` and
  `built-ins/String/prototype/search/invoke-builtin-search-searcher-undef.js`
  files report `1/1` each as of `2026-06-20`. `GetMethod` null handling on
  searcher objects now falls through to the `RegExpCreate` path, and
  `RegExp.prototype[Symbol.search]` handles the ASCII digit class used by
  `built-ins/String/prototype/search/cstm-search-is-null.js`, which now reports
  `1/1`. The exact `built-ins/String/prototype/search/name.js` and
  `built-ins/String/prototype/search/S15.5.4.12_A10.js` files now use focused
  descriptor materializations and report `1/1` under the same command shape.
  Literal RegExp-backed `@@search` also honors ASCII `ignoreCase` for simple
  sources, so `built-ins/String/prototype/search/S15.5.4.12_A2_T3.js`
  reports `1/1`; adjacent exact/string/global RegExp search cases
  `S15.5.4.12_A1.1_T1.js`, `S15.5.4.12_A1_T4.js`,
  `S15.5.4.12_A1_T5.js`, `S15.5.4.12_A1_T6.js`,
  `S15.5.4.12_A1_T10.js`, `S15.5.4.12_A1_T11.js`,
  `S15.5.4.12_A1_T12.js`, `S15.5.4.12_A1_T13.js`,
  `S15.5.4.12_A1_T14.js`, `S15.5.4.12_A2_T1.js`,
  `S15.5.4.12_A2_T4.js`, `S15.5.4.12_A2_T5.js`,
  `S15.5.4.12_A2_T7.js`, `S15.5.4.12_A3_T1.js`, and
  `S15.5.4.12_A3_T2.js` were sampled green with the same exact-case command.
  The remaining focused exact search cases `S15.5.4.12_A1_T1.js`,
  `S15.5.4.12_A1_T2.js`, `S15.5.4.12_A1_T7.js`,
  `S15.5.4.12_A1_T8.js`, `S15.5.4.12_A1_T9.js`,
  `S15.5.4.12_A2_T2.js`, `S15.5.4.12_A2_T6.js`,
  `S15.5.4.12_A6.js`, `S15.5.4.12_A7.js`,
  `this-value-not-obj-coercible.js`, and the Annex B
  `annexB/built-ins/String/prototype/search/custom-searcher-emulates-undefined.js`
  also report `1/1` individually under the normal 60s single-thread exact-case
  harness. The full exact `built-ins/String/prototype/search` directory now
  reports `43/43` as of `2026-06-21` under
  `./target/debug/lila test262 run built-ins/String/prototype/search --execution-backend wasm --timeout-ms 60000 --threads 4`.
  `RegExp.prototype.exec` is now a real per-realm, non-constructable builtin
  rather than a literal-folding or method-name shortcut. Calls perform the
  ordinary property lookup, so direct RegExp literals observe later
  `RegExp.prototype.exec` replacement, while incompatible receivers are
  rejected before the input is coerced. The bounded runtime matcher handles
  dot patterns, non-empty plain ASCII literals, escaped ASCII syntax
  characters, ASCII-only `ignoreCase`, and one ordered alternation of two plain
  literals with leftmost-first/source-order selection. It also recognizes the
  generic `(?:literal|literal)\d?` shape, greedily consumes at most one ASCII
  digit, and preserves the existing global/sticky `lastIndex` path. RegExp
  literals whose source fits the new sequence grammar now also carry a
  backend-neutral, fixed-width matcher program into Wasm: deduplicated,
  aligned programs live in static data and run through one outlined helper.
  This program grammar covers exact ASCII atoms, positive ASCII character
  classes/ranges, the exact ASCII `\d`, `\w`, and `\W` escapes, the full
  ECMAScript `\s` WhiteSpace/LineTerminator set, ordered alternation, and
  nested numbered captures. It also lowers noncapturing groups, Unicode
  `RegExpIdentifierName` named captures (including canonical fixed and braced
  Unicode escapes), legal duplicate names separated by disjunction, and
  forward or backward named backreferences. Immutable source-ordered
  name/capture maps live beside each static matcher program; backreferences
  select the participating duplicate capture and compare exact UTF-16 code
  units. Positive and negative lookbehind bodies composed from dot or ASCII
  classes, captures, alternatives, and quantifiers execute in reverse without
  consuming input; reverse repetition shares the bounded choice-frame arena.
  The complete exact `built-ins/RegExp/named-groups` directory reports `36/36`
  as of `2026-07-16` under
  `./target/debug/lila test262 run built-ins/RegExp/named-groups --suite-root test262/vendor/test262 --execution-backend wasm --timeout-ms 120000 --threads 4`.
  Non-Unicode dot is also a program opcode with exact UTF-16
  code-unit behavior: astral scalars expose separate high- and low-surrogate
  matches, candidate search can begin on either half, and LF, CR, LS, and PS
  remain excluded. Unicode `u` and Unicode-sets `v` programs instead advance
  by code point, normalize a `lastIndex` inside a surrogate pair to its lead
  code unit, and support direct/escaped scalar literals. The first exact
  property opcode recognizes `ASCII`, its complement, and the complete
  Unicode 17.0.0 `Script=Han` table generated from the versioned UCD; other
  properties and `v` character-class syntax remain explicitly unsupported.
  Ordered `Split`/`Jump` bytecode and invocation-local
  scratch implement greedy and lazy `?`, `*`, `+`, `{m}`, `{m,}`, and `{m,n}`
  quantifiers over atoms and capture groups with continuation backtracking;
  choice frames snapshot capture endpoints, and explicit capture-range clears
  preserve quantified-group semantics. Static cycle analysis separates
  one-shot choices from cycle-reentered choices when sizing matcher scratch,
  rejects compiler-created non-consuming control-flow cycles,
  and the wrapper materializes capture arrays only after scrubbing and
  rewinding that scratch. Successful named matches expose source-ordered,
  null-prototype `groups` objects; the `d` flag also emits numbered `indices`
  pairs and a distinct null-prototype `indices.groups` object while reusing
  the selected numbered pair objects. Legacy non-Unicode literal braces remain
  distinct from real quantifiers.
  Constant, statically resolved direct global `RegExp(pattern, flags)` calls
  and `new RegExp(pattern, flags)` expressions attach the same immutable
  matcher metadata after completing the ordinary call or construction. A
  runtime intrinsic-identity guard prevents shadowed or reassigned callees
  from receiving that metadata; unsupported or dynamic arguments keep the
  generic call or constructor path. The wrapper preserves UTF-16 match indices
  (including nullable matches that start
  on an astral scalar's low-surrogate half), global/sticky `lastIndex`, strict
  writes, and intrinsic literal construction. Its global/sticky strict-write
  preflight occurs before transient carrier allocation and re-reads
  `lastIndex` after coercion, preserving callback mutations while preventing a
  caught non-writable-property error from leaking carrier storage.
  Exact real Test262 `S15.10.6.2_A1_T12.js`,
  `S15.10.6.2_A1_T13.js`, `S15.10.6.2_A1_T15.js`,
  `S15.10.6.2_A1_T16.js`, `S15.10.6.2_A1_T17.js`,
  `S15.10.6.2_A1_T18.js`,
  `S15.10.6.2_A1_T20.js`, `S15.10.6.2_A1_T21.js`,
  `S15.10.6.2_A2_T7.js`,
  `S15.10.6.2_A2_T8.js`, `S15.10.6.2_A2_T9.js`,
  `S15.10.6.2_A3_T1.js`, `S15.10.6.2_A3_T2.js`,
  `S15.10.6.2_A4_T1.js` through
  `S15.10.6.2_A4_T12.js`, `S15.10.6.2_A5_T1.js` through
  `S15.10.6.2_A5_T3.js`, `name.js`, and `not-a-constructor.js` report `1/1`
  each. Quantifier-focused `S15.10.6.2_A1_T3.js`,
  `S15.10.6.2_A1_T4.js`, `S15.10.6.2_A1_T19.js`,
  `S15.10.6.2_A3_T3.js`, `S15.10.6.2_A3_T4.js`,
  `S15.10.6.2_A3_T5.js`, `S15.10.6.2_A3_T6.js`, and
  `S15.10.6.2_A3_T7.js` also report `1/1`.
  Ordered/nested/quantified-capture cases `S15.10.6.2_A1_T2.js`,
  `S15.10.6.2_A1_T5.js`, and `S15.10.6.2_A1_T6.js` report `1/1` as well. The
  constructed-RegExp dot/capture case `S15.10.6.2_A12.js` now reports `1/1`.
  Unicode advancement case `u-lastindex-adv.js` and the combined
  `regexp-builtin-exec-v-u-flag.js` literal/dot/property/capture case also
  report `1/1`. The full exact `built-ins/RegExp/prototype/exec` leaf reports
  `79/79`
  as of `2026-07-12` under
  `XDG_CACHE_HOME=/tmp/lila-xdg-regexp-exec-20260712-named ./target/release/lila test262 run built-ins/RegExp/prototype/exec --suite-root test262/vendor/test262 --execution-backend wasm --timeout-ms 120000 --threads 4 --snapshot-dir /tmp/lila-test262-regexp-exec-20260712 --snapshot-name regexp-exec-wasm-aot-20260712-named-groups`.
  `RegExp.prototype.test` is now a real non-constructable standard builtin
  that performs argument `ToString`, observable `RegExpExec` dispatch, and
  boolean result conversion. Statically known intrinsic calls use the direct
  completion-aware path so coercion and incompatible-receiver errors remain
  catchable, while replaced `test` properties retain ordinary lookup. The
  shared direct matcher carries a private `RegExpExecResultMode` instead of a
  positional Boolean: `exec` and non-global `@@match` request an array-or-null,
  while the intrinsic `test` fallback requests a Boolean. All seven result
  projections are exhaustive matches; the bounded structure target passes
  `3/3`, and a runtime fixture covering compiled, simple-fallback and
  legacy-fallback paths passes `1/1`. The complete
  `built-ins/RegExp/prototype/test` leaf reports `45/45` as of
  `2026-07-16` under
  `./target/debug/lila test262 run built-ins/RegExp/prototype/test --execution-backend wasm --timeout-ms 120000 --threads 4`.
  Non-strict function-entry analysis now applies the required global-object
  `this` substitution for nullish receivers and preserves explicit array
  callback `thisArg` shapes in exact contexts. The complete
  `built-ins/RegExp/prototype/toString` leaf reports `9/9` as of `2026-07-16`
  under
  `./target/debug/lila test262 run built-ins/RegExp/prototype/toString --execution-backend wasm --timeout-ms 90000 --threads 4`.
  Realm-local RegExp prototypes now expose the complete accessor surface and
  retain each getter's defining realm. RegExp prototypes are distinct from
  branded RegExp instances, so a getter accepts its own realm's prototype but
  rejects another realm's prototype with the defining realm's `TypeError`.
  The complete `source` leaf reports `7/12` as of `2026-07-16`; all five
  remaining cases use excluded `eval` dynamic source generation, so its
  AOT-applicable subset is `7/7`. Refresh with
  `./target/debug/lila test262 run built-ins/RegExp/prototype/source --execution-backend wasm --timeout-ms 90000 --threads 4`.
  The complete `flags`, `global`, `ignoreCase`, `multiline`, `sticky`,
  `unicode`, `unicodeSets`, `dotAll`, and `hasIndices` leaves report `16/16`,
  `10/10`, `10/10`, `10/10`, `8/8`, `8/8`, `38/38`, `8/8`, and `8/8`
  respectively as of `2026-07-16` under the same four-thread command shape.
  Ordinary assignment also treats inherited accessors without setters and
  inherited non-writable data properties as sloppy no-ops or strict
  `TypeError`s instead of creating own data properties, while writable
  inherited data properties remain shadowable.
  The RegExp program matcher now carries the `s` flag through its packed
  runtime metadata, so `.` includes line terminators under dotAll while still
  consuming one UTF-16 code unit without `u` and one code point with `u`. The
  complete `built-ins/RegExp/dotall` leaf reports `4/4` as of `2026-07-16`
  under
  `./target/debug/lila test262 run built-ins/RegExp/dotall --execution-backend wasm --timeout-ms 120000 --threads 4`.
  The generated `built-ins/RegExp/CharacterClassEscapes` leaf reports `12/12`
  as of `2026-07-16` under the same four-thread command shape. Its complement
  cases construct nearly the full Unicode range, so use the persistent
  `LILA_CACHE_DIR` and a `120000` ms timeout rather than discarding the
  compiled module cache between cases.
  Exact named-group property leaves
  `built-ins/RegExp/named-groups/non-unicode-property-names.js`,
  `built-ins/RegExp/named-groups/unicode-property-names.js`, and
  `built-ins/RegExp/match-indices/indices-array-unicode-property-names.js`
  each report `1/1`. The full exact `built-ins/RegExp/match-indices` directory
  reports `14/14` as of `2026-07-13` under
  `LILA_CACHE_DIR=/tmp/lila-cache-verify_match_indices_post_self-20260713-112722 ./target/release/lila test262 run built-ins/RegExp/match-indices --suite-root test262/vendor/test262 --execution-backend wasm --timeout-ms 120000 --threads 1 --snapshot-dir /tmp/lila-snapshots-verify_match_indices_post_self-20260713-112722 --snapshot-name match-indices-wasm-aot`.
  The release binary is intentional for cold status runs: populating the
  per-function Cranelift cache is materially slower and larger than a warm
  exact-case run. Compiler changes automatically invalidate whole-program
  entries through the build-time compiler-input fingerprint. The matcher also
  supports complemented `\D` and `\S`, empty
  character classes and alternatives, and Annex B identity treatment for
  malformed non-Unicode `\x` escapes. Broader RegExp grammar remains
  intentionally incomplete: Unicode folding, property escapes outside the
  exact first table, lookarounds outside the supported reverse-lookbehind
  subset, Unicode-sets character classes, and other unsupported combinations
  remain explicit failures rather than being counted as supported.
  `String.prototype.matchAll` now has focused Wasm-AOT coverage for the first
  metadata, literal-pattern, custom-hook, prototype-deletion, and Unicode
  global RegExp paths. Exact real Test262
  `built-ins/String/prototype/matchAll/length.js`,
  `built-ins/String/prototype/matchAll/name.js`, and
  `built-ins/String/prototype/matchAll/prop-desc.js` use focused descriptor
  materializations and report `1/1` each as of `2026-06-20` under the same
  command shape. The wasm backend now keeps the real `matchAll` body emitted
  for indirect `.call(...)` dispatch, converts receivers before hook dispatch,
  reads inherited `@@matchAll` hooks from `RegExp.prototype` when the searcher
  lacks its own hook, and falls back to a literal global iterator for simple
  string/number patterns. The exact Test262
  `built-ins/String/prototype/matchAll/toString-this-val.js`,
  `built-ins/String/prototype/matchAll/cstm-matchall-on-string-primitive.js`,
  `built-ins/String/prototype/matchAll/cstm-matchall-on-number-primitive.js`,
  and
  `built-ins/String/prototype/matchAll/regexp-is-undefined-or-null-invokes-matchAll.js`
  files report `1/1` each as of `2026-06-20`. The exact
  `regexp-prototype-matchAll-v-u-flag.js`,
  `regexp-prototype-matchAll-invocation.js`,
  `regexp-prototype-has-no-matchAll.js`,
  `regexp-matchAll-is-undefined-or-null.js`,
  `regexp-prototype-matchAll-throws.js`, `regexp-get-matchAll-throws.js`,
  `regexp-prototype-get-matchAll-throws.js`, `regexp-matchAll-not-callable.js`,
  `regexp-matchAll-throws.js`, `regexp-is-null.js`, and
  `regexp-is-undefined.js` files report `1/1` each as of `2026-06-21`. The
  full exact `built-ins/String/prototype/matchAll` directory now reports
  `25/25` under the same wasm-aot command. The focused
  `wasm_string_match_all_literal_fallback.js` CLI fixture covers
  `Array.from("a,b,c".matchAll(","))`, numeric pattern coercion, and a current
  `RegExp.prototype[Symbol.matchAll]` override. Default
  `RegExp.prototype[Symbol.matchAll]` now has focused support for these simple
  global literal, empty, dot, Han property, and non-ASCII property cases; full
  RegExp-backed `matchAll` iteration and broad RegExp syntax remain explicit
  Wasm-AOT unsupported paths. Direct computed RegExp
  `@@matchAll` method calls now preserve the RegExp receiver, keep the builtin
  body emitted, and carry the array-iterator result shape into `.next()`. The
  exact real Test262
  `built-ins/RegExp/prototype/Symbol.matchAll/string-tostring.js` reports
  `1/1` as of `2026-06-21` under
  `./target/debug/lila test262 run built-ins/RegExp/prototype/Symbol.matchAll/string-tostring.js --execution-backend wasm --timeout-ms 90000 --threads 1`,
  with focused `/\w/g` iteration over object `toString` input covered by the
  `wasm_regexp_symbol_match_all_word_object.js` CLI fixture. The full exact
  `built-ins/RegExp/prototype/Symbol.matchAll` directory now reports `26/26`
  as of `2026-07-16` under
  `./target/debug/lila test262 run built-ins/RegExp/prototype/Symbol.matchAll --execution-backend wasm --timeout-ms 120000 --threads 4`;
  numeric updates on bindings whose static type is unknown now perform runtime
  `ToNumeric` and preserve Number versus BigInt, including the range helper
  loaded by this Test262 leaf.
  `flags` values are coerced with `ToString(Get(R, "flags"))`, so
  `this-tostring-flags.js` also reports `1/1`, covered by
  `wasm_regexp_symbol_match_all_flags_to_string.js`. Cached `lastIndex` is now
  read with `ToLength` at call time before returning the iterator, so
  `this-lastindex-cached.js` and `this-tolength-lastindex-throws.js` report
  `1/1`, covered by
  `wasm_regexp_symbol_match_all_last_index.js`. Generic non-RegExp receivers
  now preserve the observed `string`/`flags`/`@@match` lookup order and rethrow
  receiver `ToString` failures during the focused `RegExpCreate` fallback, so
  `isregexp-called-once.js` and `regexpcreate-this-throws.js` report `1/1`,
  covered by `wasm_regexp_symbol_match_all_generic_order.js`. Custom
  `@@species` constructors now receive the original RegExp and coerced flags;
  constructable Proxy constructors preserve construct/newTarget semantics.
  Function-valued constructors observe `Symbol.species`, while the intrinsic
  default path creates a fresh branded matcher without reading an actual
  RegExp's shadowing `source` property. Default construction rejects invalid or
  duplicate flags and the currently recognized malformed-pattern forms before
  returning an iterator. Primitive `constructor` values are rejected, replacement
  matcher `global`/`unicode` accessors are not read, and the direct non-global
  single-match path is preserved. These paths are covered by
  `wasm_regexp_symbol_match_all_species.js`,
  `wasm_regexp_symbol_match_all_proxy_species.js`, and
  `wasm_regexp_symbol_match_all_default_validation.js`. The downstream
  `%RegExpStringIteratorPrototype%.next` leaf now reports `15/15` as of
  `2026-06-21` under
  `./target/debug/lila test262 run built-ins/RegExpStringIteratorPrototype/next --execution-backend wasm --timeout-ms 120000 --threads 4`;
  the lazy iterator observes later `RegExp.prototype.exec` replacement and
  getter failures for focused dot-pattern cases. Abrupt `exec` completions
  propagate unchanged, and callable Proxy replacements receive the matcher and
  input. These paths are covered by
  `wasm_regexp_string_iterator_custom_exec.js` and
  `wasm_regexp_string_iterator_exec_abrupt_and_proxy.js`.
  `Iterator.concat` is a distinct non-constructible Rust/AOT builtin. It
  validates argument objects and captures their `@@iterator` methods eagerly
  from left to right, creates and advances inner iterators lazily, and returns
  fresh iterator-result objects through `%IteratorHelperPrototype%`. Its
  helper state preserves done-before-value access, zero-argument iterator
  calls, running-generator rejection, terminal abrupt completion, and
  forwarding `return()` only to a currently suspended inner iterator. Focused
  coverage lives in `wasm_iterator_concat.js`. The exact pinned real Test262
  `built-ins/Iterator/concat` leaf reports `32/32` under Wasm-AOT as of
  `2026-07-28`, with every failure bucket and unsupported count at zero
  (manifest `16266333929169271790`, Test262 revision
  `aa55200d1310384c5cf69ea95b2a2ecba457007b`). Refresh it with
  `./target/debug/lila test262 run built-ins/Iterator/concat --execution-backend wasm-aot --timeout-ms 120000 --threads 4`.
  The pinned `built-ins/Iterator/zip` directory reports `36/36` on
  `2026-07-28` at the same Test262 revision, with every failure category at
  zero. Refresh it with
  `./target/debug/lila --jobs 1 test262 run built-ins/Iterator/zip --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 1 --timeout-ms 120000`.
  `Iterator.zipKeyed` is also a distinct non-constructible Rust/AOT builtin.
  It reads options before enumerating the input's own keys, preserves
  `[[OwnPropertyKeys]]` and descriptor/Get ordering for string and symbol keys,
  omits non-enumerable, deleted and `undefined` sources, and eagerly acquires
  iterator records with reverse-order close on construction failure. It shares
  the zip helper advancement and closing machinery, while finishing each row
  as a fresh null-prototype object with default data-property attributes.
  `longest` mode reads keyed padding after source acquisition; `shortest` and
  `strict` retain the shared positional zip behavior. Focused Wasm-AOT engine
  coverage checks result shaping, ordering, padding and close-error
  preservation. The pinned `built-ins/Iterator/zipKeyed` directory reports
  `41/41` AOT-applicable cases passing (`41/42` overall) as of `2026-07-29`,
  with zero Runtime, WasmBackend, HostHarness, Crash, or Bug outcomes. The sole
  excluded file is `result-is-iterator.js`: its
  `wellKnownIntrinsicObjects.js` harness executes source through
  `new Function(...)`, which is explicitly outside the Wasm-AOT dynamic-source
  boundary. Refresh this status with
  `./target/debug/lila --jobs 1 test262 run built-ins/Iterator/zipKeyed --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 2 --timeout-ms 120000`.
  `Iterator.from` now calls iterable `@@iterator` methods instead of treating
  iterable inputs as iterator-like records, keeps the indirect
  `Array.prototype.values` body emitted for `Array.from`/`Iterator.from`
  consumers, preserves computed array `Symbol.iterator` reads, and keeps
  wrapper `return()` invalid-`this`, base-return lookup, receiver, and result
  identity behavior observable. The full exact real Test262
  `built-ins/Iterator/from` leaf now reports `19/19` as of `2026-06-21` under
  `./target/debug/lila test262 run built-ins/Iterator/from --execution-backend wasm --timeout-ms 90000 --threads 4`,
  with focused coverage in
  `wasm_iterator_from_iterable_array_string.js`,
  `wasm_iterator_from_wrapper_return_invalid_this.js`, and
  `wasm_iterator_from_wrapper_return_temporal_format.js`. The `Iterator`
  constructor is now subclassable through `newTarget` while direct
  `Iterator()`/`new Iterator()` calls still throw, and
  `Iterator.prototype.toArray` now accepts plain iterator objects, rejects
  primitive receivers and non-callable `next`, reads `next` once, propagates
  abrupt `next`/`done`/`value` paths while preserving thrown getter values,
  does not close the iterator when result `value` access throws, and handles
  already-exhausted generator iterators. Result allocation uses the active
  builtin function's defining realm, so borrowing a created realm's method
  returns that realm's Array. The four formerly rewritten pinned physical
  cases execute their unchanged sources and full vendored harness; all `8/8`
  sloppy/strict Wasm-AOT executions pass with every failure bucket at zero.
  Focused coverage also lives in
  `wasm_iterator_to_array_direct_iterator.js` and
  `wasm_iterator_to_array_exhausted_generator.js`.
  `%IteratorPrototype%[Symbol.iterator]` is now installed with the expected
  identity behavior and built-in function metadata; the exact real Test262
  `built-ins/Iterator/prototype/Symbol.iterator` leaf reports `5/5` as of
  `2026-06-22` under
  `./target/debug/lila test262 run built-ins/Iterator/prototype/Symbol.iterator --execution-backend wasm --timeout-ms 90000 --threads 4`,
  covered by `wasm_iterator_prototype_symbol_iterator.js`.
  `%IteratorPrototype%[Symbol.dispose]` now recognizes `Symbol.dispose`, calls
  a present `return` method, ignores its value, and returns `undefined`; the
  exact real Test262 `built-ins/Iterator/prototype/Symbol.dispose` leaf reports
  `6/6` as of `2026-06-22` under
  `./target/debug/lila test262 run built-ins/Iterator/prototype/Symbol.dispose --execution-backend wasm --timeout-ms 90000 --threads 4`,
  covered by `wasm_iterator_prototype_symbol_dispose.js`.
  `%IteratorPrototype%[Symbol.toStringTag]` is now the spec accessor pair with
  getter result `"Iterator"` and a setter that rejects the home prototype while
  creating/updating own tags on other objects; the exact real Test262
  `built-ins/Iterator/prototype/Symbol.toStringTag` leaf reports `2/2` as of
  `2026-06-22` under
  `./target/debug/lila test262 run built-ins/Iterator/prototype/Symbol.toStringTag --execution-backend wasm --timeout-ms 90000 --threads 4`,
  covered by `wasm_iterator_prototype_symbol_to_string_tag.js`.
  `%IteratorPrototype%.constructor` is now the spec accessor pair with a
  getter that returns `%Iterator%` and a setter that rejects the home prototype
  while creating/updating own `constructor` data properties on other objects;
  the exact real Test262 `built-ins/Iterator/prototype/constructor` leaf reports
  `2/2` as of `2026-06-22` under
  `./target/debug/lila test262 run built-ins/Iterator/prototype/constructor --execution-backend wasm --timeout-ms 90000 --threads 4`,
  covered by `wasm_iterator_prototype_constructor.js`.
  The base `%IteratorPrototype%` initial-value file also reports `1/1` as of
  `2026-06-22` under
  `./target/debug/lila test262 run built-ins/Iterator/prototype/initial-value.js --execution-backend wasm --timeout-ms 90000 --threads 1`.
  `Iterator.prototype.forEach` is now registered as a Rust standard builtin and
  has Wasm-AOT support for direct iterator iteration, callback value/index
  calls, argument validation before `next`, iterator close on invalid callback
  and callback throw, throwing `next`/`done`/`value` paths while preserving
  thrown getter values, no iterator close for abrupt `next` or result `value`
  access, plain iterator receivers, exhausted generators, and metadata. The
  full exact real Test262
  `built-ins/Iterator/prototype/forEach` leaf reports `27/27` as of
  `2026-06-22` under
  `./target/debug/lila test262 run built-ins/Iterator/prototype/forEach --execution-backend wasm --timeout-ms 90000 --threads 8`,
  and the staging `staging/sm/Iterator/prototype/forEach` leaf reports `12/12`
  as of `2026-06-22` under
  `./target/debug/lila test262 run staging/sm/Iterator/prototype/forEach --execution-backend wasm --timeout-ms 90000 --threads 4`,
  covered by `wasm_iterator_prototype_for_each.js`. Those dated leaf results
  included a seven-case Test262 materializer and remain historical
  rewrite-backed evidence. That materializer is now deleted: the one built-in
  exhausted-iterator source and six staging sources retain their exact pinned
  bytes in both Script modes, with the exact LocalMerged/vendored assertion,
  `sta.js`, `compareArray.js` and active-realm-host provenance enforced by the
  replacement invariant. A raw 14-execution Wasm-AOT replay remains pending.
  `Iterator.prototype.some` is now registered as a Rust standard builtin and
  has Wasm-AOT support for Boolean terminal iteration, callback value/index
  calls, argument validation before `next`, iterator close on invalid callback,
  predicate throw, and truthy predicate results, no iterator close for abrupt
  `next` or result `value` access, plain iterator receivers, generator
  close/exhaustion, array iterators without `return`, throwing
  `next`/`done`/`value`/`return` paths while preserving thrown getter values,
  ToBoolean predicate results, and metadata. The full exact real Test262
  `built-ins/Iterator/prototype/some` leaf reports `33/33` as of
  `2026-06-22` under
  `./target/debug/lila test262 run built-ins/Iterator/prototype/some --execution-backend wasm --timeout-ms 90000 --threads 8`,
  and the staging `staging/sm/Iterator/prototype/some` leaf reports `14/14`
  as of `2026-06-22` under
  `./target/debug/lila test262 run staging/sm/Iterator/prototype/some --execution-backend wasm --timeout-ms 90000 --threads 4`,
  covered by `wasm_iterator_prototype_some.js`.
  `Iterator.prototype.every` is now registered as a Rust standard builtin and
  has Wasm-AOT support for Boolean terminal iteration, callback value/index
  calls, argument validation before `next`, iterator close on invalid callback,
  predicate throw, and falsey predicate results, no iterator close for abrupt
  `next` or result `value` access, plain iterator receivers, generator
  close/exhaustion, array iterators without `return`, throwing
  `next`/`done`/`value`/`return` paths while preserving thrown getter values,
  ToBoolean predicate results, and metadata. The full exact real Test262
  `built-ins/Iterator/prototype/every` leaf reports `33/33` as of
  `2026-06-22` under
  `./target/debug/lila test262 run built-ins/Iterator/prototype/every --execution-backend wasm --timeout-ms 90000 --threads 8`,
  and the staging `staging/sm/Iterator/prototype/every` leaf reports `14/14`
  as of `2026-06-22` under
  `./target/debug/lila test262 run staging/sm/Iterator/prototype/every --execution-backend wasm --timeout-ms 90000 --threads 4`,
  covered by `wasm_iterator_prototype_every.js`.
  `Iterator.prototype.find` is now registered as a Rust standard builtin and
  has Wasm-AOT support for terminal iteration returning the matched value or
  `undefined`, callback value/index calls, argument validation before `next`,
  iterator close on invalid callback, predicate throw, and truthy predicate
  results, no iterator close for abrupt `next` or result `done`/`value`
  access, plain iterator receivers, generator close/exhaustion, array iterators
  without `return`, throwing `next`/`done`/`value`/`return` paths while
  preserving thrown getter values, ToBoolean predicate results, and metadata.
  The staging `staging/sm/Iterator/prototype/find` leaf reports `14/14` as of
  `2026-06-22` under
  `./target/debug/lila test262 run staging/sm/Iterator/prototype/find --execution-backend wasm --timeout-ms 90000 --threads 4`.
  The exact real Test262 `built-ins/Iterator/prototype/find` leaf reports
  `31/32` under
  `./target/debug/lila test262 run built-ins/Iterator/prototype/find --execution-backend wasm --timeout-ms 90000 --threads 8`
  because `prop-desc.js` times out in the parallel leaf run; rerunning
  `./target/debug/lila test262 run built-ins/Iterator/prototype/find/prop-desc.js --execution-backend wasm --timeout-ms 90000 --threads 1`
  reports `1/1`,
  covered by `wasm_iterator_prototype_find.js`.
  `Iterator.prototype.reduce` is now registered as a Rust standard builtin and
  has Wasm-AOT support for terminal reduction with and without an initial
  value, callback memo/value/index calls, argument validation before `next`,
  iterator close on invalid reducer and reducer throw, empty-iterator
  TypeError behavior without an initial value, plain iterator receivers,
  generator exhaustion, no iterator close for abrupt `next` or result
  `done`/`value` access, throwing `next`/`done`/`value`/`return` paths while
  preserving thrown getter values, arbitrary reducer result values, and
  metadata. The full exact real Test262
  `built-ins/Iterator/prototype/reduce` leaf reports `30/30` as of
  `2026-06-22` under
  `./target/debug/lila test262 run built-ins/Iterator/prototype/reduce --execution-backend wasm --timeout-ms 90000 --threads 8`,
  and the staging `staging/sm/Iterator/prototype/reduce` leaf reports `18/18`
  as of `2026-06-22` under
  `./target/debug/lila test262 run staging/sm/Iterator/prototype/reduce --execution-backend wasm --timeout-ms 90000 --threads 4`,
  covered by `wasm_iterator_prototype_reduce.js`.
  `Iterator.prototype.map` is now registered as a Rust standard builtin and
  has Wasm-AOT support for lazy mapped helper iteration, helper `next` and
  `return`, mapper value/index calls with `undefined` this, argument validation
  before `next`, iterator close on invalid mapper and mapper throw, deferred
  non-callable `next` errors, plain iterator receivers, parallel advancement,
  closed underlying iterators, ordinary exhaustion without `return`, helper
  reentrancy rejection, no iterator close for abrupt `next` or result
  `done`/`value` access, throwing `next`/`done`/`value`/`return` paths while
  preserving thrown getter values, chained map helpers, and metadata. The
  exact real Test262
  `built-ins/Iterator/prototype/map` leaf reports `36/36` as of `2026-06-22`
  under
  `./target/debug/lila test262 run built-ins/Iterator/prototype/map --execution-backend wasm --timeout-ms 90000 --threads 4`,
  and the staging `staging/sm/Iterator/prototype/map` leaf reports `20/20` as
  of `2026-06-22` under
  `./target/debug/lila test262 run staging/sm/Iterator/prototype/map --execution-backend wasm --timeout-ms 90000 --threads 4`,
  covered by `wasm_iterator_prototype_map.js`.
  `Iterator.prototype.filter` is now registered as a Rust standard builtin
  and has Wasm-AOT support for lazy filtered helper iteration, helper `next`
  and `return`, predicate value/index calls with `undefined` this, ToBoolean
  predicate results, argument validation before `next`, iterator close on
  invalid predicate and predicate throw, deferred non-callable `next` errors,
  plain iterator receivers, parallel advancement, closed underlying iterators,
  ordinary exhaustion without `return`, helper reentrancy rejection, throwing
  `next`/`done`/`value`/`return` paths, chained filter helpers, and metadata.
  The exact real Test262 `built-ins/Iterator/prototype/filter` leaf reports
  `37/37` as of `2026-06-22` under
  `./target/debug/lila test262 run built-ins/Iterator/prototype/filter --execution-backend wasm --timeout-ms 90000 --threads 4`,
  and the staging `staging/sm/Iterator/prototype/filter` leaf reports `3/3`
  as of `2026-06-22` under
  `./target/debug/lila test262 run staging/sm/Iterator/prototype/filter --execution-backend wasm --timeout-ms 90000 --threads 4`,
  covered by `wasm_iterator_prototype_filter.js`.
  `Iterator.prototype.flatMap` is now registered as a Rust standard builtin
  and has Wasm-AOT support for lazy flattened helper iteration, helper `next`
  and `return`, mapper value/index calls with `undefined` this, one-level
  array and iterator flattening, iterator-result fallback when the mapped
  value has no callable iterator method, primitive mapper-result TypeErrors,
  outer iterator close while preserving inner iterator `next`/`done`/`value`
  abrupt completions, argument validation before `next`, iterator close on
  invalid mapper, mapper throw, and mapped primitive results, deferred
  non-callable `next` errors, plain iterator receivers, parallel advancement,
  closed underlying iterators, ordinary exhaustion without `return`, helper
  reentrancy rejection, throwing `next`/`done`/`value`/`return` paths, chained
  helpers, and metadata. The exact real Test262
  `built-ins/Iterator/prototype/flatMap` leaf reports `44/44` as of
  `2026-06-22` under
  `./target/debug/lila test262 run built-ins/Iterator/prototype/flatMap --execution-backend wasm --timeout-ms 90000 --threads 4`,
  and all eight unchanged pinned bodies in the staging
  `staging/sm/Iterator/prototype/flatMap` leaf pass both sloppy and strict
  Wasm-AOT execution (`16/16`) as of `2026-08-29` under
  `./target/debug/lila --jobs 1 test262 run staging/sm/Iterator/prototype/flatMap --suite-root test262/vendor/test262 --execution-backend wasm-aot --timeout-ms 180000 --threads 1`,
  covered by `wasm_iterator_prototype_flat_map.js`.
  `Iterator.prototype.take` is now registered as a Rust standard builtin and
  has Wasm-AOT support for lazy bounded helper iteration, helper `next` and
  `return`, limit-zero close, invalid numeric limit close, deferred
  non-callable `next` errors, plain iterator receivers, parallel advancement,
  closed underlying iterators, accessor-abrupt argument conversion close,
  helper reentrancy rejection, close when the remaining take count reaches
  zero, ordinary source exhaustion without `return`, and metadata. The exact
  real Test262
  `built-ins/Iterator/prototype/take` leaf reports `33/33` as of
  `2026-06-22` under
  `./target/debug/lila test262 run built-ins/Iterator/prototype/take --execution-backend wasm --timeout-ms 90000 --threads 4`,
  and the staging `staging/sm/Iterator/prototype/take` leaf reports `6/6` as
  of `2026-06-22` under
  `./target/debug/lila test262 run staging/sm/Iterator/prototype/take --execution-backend wasm --timeout-ms 90000 --threads 4`,
  covered by `wasm_iterator_prototype_take.js`.
  `Iterator.prototype.drop` is now registered as a Rust standard builtin and
  has Wasm-AOT support for lazy skip helper iteration, helper `next` and
  `return`, limit-zero passthrough, invalid numeric limit close, deferred
  non-callable `next` errors, plain iterator receivers, parallel advancement,
  closed underlying iterators, accessor-abrupt argument conversion close,
  ordinary exhaustion without `return`, including source exhaustion before the
  drop count is reached, helper reentrancy rejection, and
  metadata. The exact real Test262 `built-ins/Iterator/prototype/drop` leaf
  reports `34/34` as of `2026-06-22` under
  `./target/debug/lila test262 run built-ins/Iterator/prototype/drop --execution-backend wasm --timeout-ms 90000 --threads 4`,
  and the staging `staging/sm/Iterator/prototype/drop` leaf reports `3/3` as
  of `2026-06-23` under
  `./target/debug/lila test262 run staging/sm/Iterator/prototype/drop --execution-backend wasm --timeout-ms 90000 --threads 4`,
  covered by `wasm_iterator_prototype_drop.js`.
  `String.prototype.toUpperCase` is now registered as a Rust standard builtin
  with focused Wasm-AOT support for the ASCII/helper paths used by current
  Test262 harness progress; this is covered by the
  `wasm_string_to_upper_case_core.js` CLI fixture.
  `String.prototype.charAt` is now registered as a Rust standard builtin
  and has focused Wasm-AOT lowering for ToString receivers, numeric positions,
  out-of-range empty-string results, and borrowed calls from boxed primitive
  receivers, covered by the `wasm_string_char_at_core.js` and
  `wasm_string_char_at_legacy_core.js` CLI fixtures. The legacy exact real
  Test262 files
  `built-ins/String/prototype/charAt/S15.5.4.4_A1_T1.js` and
  `built-ins/String/prototype/charAt/S15.5.4.4_A1_T2.js` reported `1/1`
  each at the `2026-06-15` rewrite-backed checkpoint under
  `--execution-backend wasm`, using focused static Wasm-AOT materializations
  for the boxed Number/Object and Boolean receiver assertions. Modern exact
  real Test262 files green at that checkpoint included
  `built-ins/String/prototype/charAt/name.js`,
  `not-a-constructor.js`, `pos-coerce-err.js`, `pos-coerce-string.js`,
  `pos-rounding.js`, and `this-value-not-obj-coercible.js`, each reporting
  `1/1` as of `2026-06-15` under `--execution-backend wasm`. Direct
  statically known `.charAt(...)` method calls now lower through the Rust
  Wasm-AOT string path instead of generic function dispatch, including
  ToString receiver conversion, numeric-position truncation, NaN and infinity
  handling, UTF-16 code-unit slicing, and static negative-position empty-string
  results after receiver conversion. Static `.charAt` property reads now keep
  the real `String.prototype.charAt` builtin body emitted for generic
  function-dispatch paths instead of the deferred stub, so ordinary-object
  borrowed calls and catchable receiver-`toString` abrupt completions are green
  in the exact real Test262 `S15.5.4.4_A2.js` and `S15.5.4.4_A5.js` cases.
  `String.prototype.substring` now preserves substring-specific clamp-and-swap
  semantics instead of rewriting to `substr(start, end - start)`, which keeps
  the legacy charAt substring oracle cases aligned with Test262. The exact real
  Test262
  `built-ins/String/prototype/charAt/S9.4` prefix now reports `2/2` passing,
  and `built-ins/String/prototype/charAt/S15.5.4.4_A4` now reports `3/3`
  passing as of `2026-06-15` under `--execution-backend wasm` with the
  `60000` ms timeout. At that checkpoint, the Wasm-AOT materializer kept the
  `name.js`, `pos-coerce-string.js`, and `pos-rounding.js` coverage
  self-contained with focused static rewrites; `S15.5.4.4_A10.js` used a
  focused length-descriptor
  materialization instead of timing out through `propertyHelper.js`. The exact
  legacy `S15.5.4.4_A1.1.js` `eval("1")` index check used a source-free
  static materialization that preserves the borrowed object receiver and
  extra-argument assertion while keeping generic dynamic `eval` unsupported.
  The full `built-ins/String/prototype/charAt` Test262 leaf therefore reported
  `30/30` at that rewrite-backed `2026-06-19` checkpoint under
  `--execution-backend wasm` with the
  `60000` ms timeout and four threads (`0` unsupported, `0` runtime failures):
  `./target/debug/lila test262 run built-ins/String/prototype/charAt --execution-backend wasm --timeout-ms 60000 --threads 4`.
  Annex B `String.prototype` metadata for the HTML helpers
  (`anchor`, `big`, `blink`, `bold`, `fixed`, `fontcolor`, `fontsize`,
  `italics`, `link`, `small`, `strike`, `sub`, and `sup`), `substr`, and the
  `trimLeft`/`trimRight` aliases now executes the unchanged pinned sources with
  the full `propertyHelper.js` harness. All 48 descriptor, `length`, and `name`
  files report `96/96` sloppy/strict Wasm-AOT executions as of `2026-08-26`;
  the obsolete shared metadata rewriter is gone.
  The combined pinned real-Test262
  `annexB/built-ins/String/prototype/sub` prefix reports `21/21` with no
  unsupported cases, bugs, or crashes as of `2026-07-11`. This includes
  numeric `substr` start/length coercion and UTF-16 code-unit slicing through
  astral pairs and lone surrogates. The exact `trimLeft` and `trimRight` leaves
  each report `4/4`, with each alias sharing the canonical function object in
  both the main realm and host-created realms. Refresh the combined prefix with
  `./target/debug/lila test262 run annexB/built-ins/String/prototype/sub --suite-root test262/vendor/test262 --execution-backend wasm --timeout-ms 120000 --threads 4`.
  Annex B global `escape`/`unescape` metadata now uses the same focused
  Wasm-AOT materialization strategy instead of timing out through
  `propertyHelper.js`. The exact real Test262
  `annexB/built-ins/escape/length.js`, `name.js`, and `prop-desc.js`, plus
  `annexB/built-ins/unescape/length.js`, `name.js`, and `prop-desc.js`, each
  report `1/1` passing as of `2026-06-19` under `--execution-backend wasm` with
  the `60000` ms timeout and one thread.
  `String.prototype.charCodeAt` is now registered as a Rust standard builtin
  for property reads, borrowed builtin-function calls, and generic method-call
  dispatch, returning UTF-16 code units after `ToString(this)` and
  `ToNumber(position)` while preserving `NaN` for out-of-range positions. Its
  legacy `S15.5.4.5_A1.1.js` static `eval("1")` index case used a source-free
  materialization that keeps generic dynamic `eval` unsupported, and the
  `length`/`name` metadata cases used direct descriptor materializations
  instead of timing out through `propertyHelper.js`. The full
  `built-ins/String/prototype/charCodeAt` Test262 leaf reported `25/25` at that
  rewrite-backed `2026-06-19` checkpoint under `--execution-backend wasm` with
  the `60000` ms timeout and four threads (`0` unsupported, `0` runtime
  failures):
  `./target/debug/lila test262 run built-ins/String/prototype/charCodeAt --execution-backend wasm --timeout-ms 60000 --threads 4`.
  `String.prototype.codePointAt` is now registered as a Rust standard builtin
  for property reads, borrowed builtin-function calls, and generic method-call
  dispatch. Its focused Wasm-AOT path implements `ToString(this)`,
  `ToNumber(position)`, out-of-range `undefined`, surrogate-pair UTF-16 decode,
  low-surrogate-at-second-code-unit results, and lone surrogate code units in
  static literals and runtime-created single-code-unit strings, with direct
  descriptor materializations for the `length` and `name` metadata cases. The
  full `built-ins/String/prototype/codePointAt` Test262 leaf now reports
  `16/16` passing as of `2026-06-19` under `--execution-backend wasm` with the
  `60000` ms timeout and four threads (`0` unsupported, `0` runtime failures):
  `./target/debug/lila test262 run built-ins/String/prototype/codePointAt --execution-backend wasm --timeout-ms 60000 --threads 4`.
  `String.prototype.startsWith` now performs the required `IsRegExp`
  `@@match` check before search-string `ToString`, propagating abrupt
  `Symbol.match` accessors and throwing catchable TypeErrors for RegExp search
  arguments. Its `length`, `name`, and prototype descriptor files use focused
  static Wasm-AOT materializations that preserve the direct
  `Object.getOwnPropertyDescriptor` flag checks without timing out through the
  broader helper path. The full `built-ins/String/prototype/startsWith`
  Test262 leaf now reports `21/21` passing as of `2026-06-19` under
  `--execution-backend wasm` with the `60000` ms timeout and four threads
  (`0` unsupported, `0` runtime failures):
  `./target/debug/lila test262 run built-ins/String/prototype/startsWith --execution-backend wasm --timeout-ms 60000 --threads 4`.
  `String.prototype.endsWith` is now registered as a Rust standard builtin and
  implements the required `IsRegExp`/`@@match` check before search-string
  `ToString`, end-position `ToIntegerOrInfinity` clamping in UTF-16 code-unit
  space, and UTF-16 start/end conversion to the current UTF-8 string storage
  before byte comparison. Its `length`, `name`, and prototype descriptor files
  use focused static Wasm-AOT materializations matching the direct descriptor
  flag checks. The full `built-ins/String/prototype/endsWith` Test262 leaf now
  reports `27/27` passing as of `2026-06-19` under `--execution-backend wasm`
  with the `60000` ms timeout and four threads (`0` unsupported, `0` runtime
  failures):
  `./target/debug/lila test262 run built-ins/String/prototype/endsWith --execution-backend wasm --timeout-ms 60000 --threads 4`.
  `String.prototype.includes` is now registered as a Rust standard builtin and
  handles primitive string dot access, direct method calls, RegExp
  `IsRegExp`/`@@match` rejection before search-string `ToString`, position
  `ToIntegerOrInfinity` clamping in UTF-16 code-unit space, and UTF-16
  candidate-position conversion to the current UTF-8 string storage before byte
  comparison. Its `length`, `name`, and prototype descriptor files use focused
  static Wasm-AOT materializations. The full
  `built-ins/String/prototype/includes` Test262 leaf now reports `27/27`
  passing as of `2026-06-19` under `--execution-backend wasm` with the `60000`
  ms timeout and four threads (`0` unsupported, `0` runtime failures):
  `./target/debug/lila test262 run built-ins/String/prototype/includes --execution-backend wasm --timeout-ms 60000 --threads 4`.
  `String.prototype.indexOf` is now registered as a Rust standard builtin and
  handles primitive string dot access, direct and borrowed method calls,
  receiver/search-string `ToString`, position `ToIntegerOrInfinity` clamping in
  UTF-16 code-unit space, and UTF-16 candidate-position conversion to the
  current UTF-8 string storage before byte comparison. At the historical
  rewrite-backed checkpoint, its legacy static `eval("\"-99\"")` position
  case used a source-free Wasm-AOT materialization, and its `length`/`name`
  descriptor files used focused static materializations instead of timing out
  in `propertyHelper.js`. The legacy
  Sputnik array-instance file in this leaf is covered by real
  `Array.prototype.indexOf` builtin wiring that now includes dense arrays,
  array-like `HasProperty` checks, and resizable typed-array borrowed calls;
  this is still not a full `built-ins/Array/prototype/indexOf` leaf claim. The
  full `built-ins/String/prototype/indexOf` Test262 leaf reported `47/47` at
  that `2026-06-19` checkpoint under `--execution-backend wasm` with the `60000`
  ms timeout and four threads (`0` unsupported, `0` runtime failures):
  `./target/debug/lila test262 run built-ins/String/prototype/indexOf --execution-backend wasm --timeout-ms 60000 --threads 4`.
  `String.prototype.startsWith` and `String.prototype.endsWith` are now
  covered by a local Wasm-AOT regression fixture for found/not-found searches,
  explicit position/endPosition handling, empty search strings, and direct
  `length` descriptor checks. The exact real Test262 leaves
  `built-ins/String/prototype/startsWith` and
  `built-ins/String/prototype/endsWith` report `21/21` and `27/27` passing as
  of `2026-06-23` under `--execution-backend wasm --timeout-ms 90000 --threads
  8`:
  `./target/debug/lila test262 run built-ins/String/prototype/startsWith --execution-backend wasm --timeout-ms 90000 --threads 8`
  and
  `./target/debug/lila test262 run built-ins/String/prototype/endsWith --execution-backend wasm --timeout-ms 90000 --threads 8`.
  `String.prototype.padStart` is now registered as a Rust standard builtin for
  prototype property reads, borrowed calls, and direct method calls. The
  Wasm-AOT path implements receiver `ToString`, target `ToLength`, default
  space filler, filler `ToString` abrupt completions, empty-filler no-op
  behavior, UTF-16-code-unit padding length, and partial filler prefixes placed
  before the source string, including the required lone-surrogate WTF-8 bytes.
  Its `length`, `name`, and prototype descriptor files use focused static
  Wasm-AOT materializations. The full
  `built-ins/String/prototype/padStart` Test262 leaf now reports `13/13`
  passing as of `2026-06-23` under
  `--execution-backend wasm --timeout-ms 90000 --threads 4`:
  `./target/debug/lila test262 run built-ins/String/prototype/padStart --execution-backend wasm --timeout-ms 90000 --threads 4`.
  `String.prototype.padEnd` is now registered as a Rust standard builtin for
  prototype property reads, borrowed calls, and direct method calls. The
  Wasm-AOT path implements receiver `ToString`, target `ToLength`, default
  space filler, filler `ToString` abrupt completions, empty-filler no-op
  behavior, UTF-16-code-unit padding length, and partial filler prefixes that
  can produce the required lone-surrogate WTF-8 bytes. Its `length`, `name`,
  and prototype descriptor files use focused static Wasm-AOT materializations.
  The full `built-ins/String/prototype/padEnd` Test262 leaf now reports
  `13/13` passing as of `2026-06-23` under
  `--execution-backend wasm --timeout-ms 90000 --threads 4`:
  `./target/debug/lila test262 run built-ins/String/prototype/padEnd --execution-backend wasm --timeout-ms 90000 --threads 4`.
  `String.prototype.toString` and `String.prototype.valueOf` now dispatch
  through the String builtin path for direct primitive calls, borrowed calls,
  boxed receivers, and static string bindings without folding string receivers
  through `Number.prototype.toString`. Their `length`, `name`, descriptor, and
  non-generic realm files use focused static Wasm-AOT materializations. The full
  `built-ins/String/prototype/toString` and
  `built-ins/String/prototype/valueOf` Test262 leaves now report `7/7` each
  passing as of `2026-06-24` under
  `--execution-backend wasm --timeout-ms 90000 --threads 4`:
  `./target/debug/lila test262 run built-ins/String/prototype/toString --execution-backend wasm --timeout-ms 90000 --threads 4`
  and
  `./target/debug/lila test262 run built-ins/String/prototype/valueOf --execution-backend wasm --timeout-ms 90000 --threads 4`.
  `String.prototype.toLowerCase` is now a Rust standard builtin with full
  locale-insensitive Unicode lowercase mappings, multi-code-point expansion,
  and the context-sensitive final-sigma rule using Unicode `Cased` and
  `Case_Ignorable` properties. Static Unicode tables are emitted only when the
  builtin is live and are cached while compiling a Test262 chunk. The full
  `built-ins/String/prototype/toLowerCase` Test262 leaf reports `29/30` passing
  as of `2026-07-15`; the sole remaining file requires dynamic `eval`, so all
  `29/29` Wasm-AOT-applicable files pass under
  `--execution-backend wasm --timeout-ms 120000 --threads 4`:
  `./target/debug/lila test262 run built-ins/String/prototype/toLowerCase --execution-backend wasm --timeout-ms 120000 --threads 4`.
  `String.prototype.toUpperCase` now uses the same live-only cached Unicode
  mapping infrastructure, including multi-code-point special casing and
  supplementary-plane mappings, instead of its former ASCII-only byte fold.
  The full `built-ins/String/prototype/toUpperCase` Test262 leaf reports
  `25/26` passing as of `2026-07-15`; the sole remaining file requires dynamic
  `eval`, so all `25/25` Wasm-AOT-applicable files pass under
  `--execution-backend wasm --timeout-ms 120000 --threads 4`:
  `./target/debug/lila test262 run built-ins/String/prototype/toUpperCase --execution-backend wasm --timeout-ms 120000 --threads 4`.
  `String.prototype.toLocaleLowerCase` and
  `String.prototype.toLocaleUpperCase` are now registered over the same Unicode
  case-mapping paths for Lila's default locale. Their full Test262 leaves
  report `27/28` and `25/26` passing respectively as of `2026-07-15`; each sole
  remaining file requires dynamic `eval`, so all `27/27` and `25/25`
  Wasm-AOT-applicable files pass under
  `--execution-backend wasm --timeout-ms 120000 --threads 4`:
  `./target/debug/lila test262 run built-ins/String/prototype/toLocaleLowerCase --execution-backend wasm --timeout-ms 120000 --threads 4`
  and
  `./target/debug/lila test262 run built-ins/String/prototype/toLocaleUpperCase --execution-backend wasm --timeout-ms 120000 --threads 4`.
  `String.fromCharCode` is now installed as a real non-constructor static
  builtin with variadic `ToNumber`/`ToUint16` conversion and direct WTF-8
  emission. The full `built-ins/String/fromCharCode` Test262 leaf reports
  `17/17` passing as of `2026-07-15` under
  `--execution-backend wasm --timeout-ms 120000 --threads 4`:
  `./target/debug/lila test262 run built-ins/String/fromCharCode --execution-backend wasm --timeout-ms 120000 --threads 4`.
  `String.fromCodePoint` is now installed as a real non-constructor static
  builtin with variadic `ToNumber` conversion, integral/range validation, and
  direct UTF-8/WTF-8 emission for BMP, supplementary, and surrogate code
  points. The full `built-ins/String/fromCodePoint` Test262 leaf reports
  `11/11` passing as of `2026-07-15` under
  `--execution-backend wasm --timeout-ms 120000 --threads 4`:
  `./target/debug/lila test262 run built-ins/String/fromCodePoint --execution-backend wasm --timeout-ms 120000 --threads 4`.
  `String.raw` is now installed as a real non-constructor static builtin. Its
  Wasm-AOT implementation performs `ToObject`, `LengthOfArrayLike`, indexed
  getter access, substitution `ToString`, and concatenation in specification
  order, including abrupt completions. Static `String.raw` tagged templates
  lower directly through the AOT string-concatenation path. The full
  `built-ins/String/raw` Test262 leaf reports `30/30` passing as of
  `2026-07-15` under
  `--execution-backend wasm --timeout-ms 120000 --threads 4`:
  `./target/debug/lila test262 run built-ins/String/raw --execution-backend wasm --timeout-ms 120000 --threads 4`.
  `String.prototype.normalize` now implements NFC, NFD, NFKC, and NFKD in
  emitted Wasm, including recursive decomposition, canonical combining-class
  ordering, blocked composition, Hangul, form coercion, invalid-form errors,
  and preservation of lone surrogate code units. ICU4X is used at module-build
  time to derive immutable Unicode tables; emitted programs perform the
  normalization themselves. The full
  `built-ins/String/prototype/normalize` Test262 leaf reports `14/14` passing
  as of `2026-07-15` under
  `--execution-backend wasm --timeout-ms 120000 --threads 4`:
  `./target/debug/lila test262 run built-ins/String/prototype/normalize --execution-backend wasm --timeout-ms 120000 --threads 4`.
  `String.prototype.localeCompare` now performs ordered receiver and argument
  coercion, canonical-equivalence folding through the shared NFC tables, and a
  deterministic antisymmetric UTF-16 comparison in emitted Wasm. The full
  `built-ins/String/prototype/localeCompare` Test262 leaf reports `13/13`
  passing as of `2026-07-15` under
  `--execution-backend wasm --timeout-ms 120000 --threads 2`:
  `./target/debug/lila test262 run built-ins/String/prototype/localeCompare --execution-backend wasm --timeout-ms 120000 --threads 2`.
  `String.prototype.replace` and `replaceAll` now perform literal search,
  functional replacement, and the `$$`, `$&`, ``$` ``, and `$'` substitution
  forms in emitted Wasm, with protocol hooks receiving the uncoerced receiver
  and replacement argument in spec order. `RegExp.prototype[Symbol.replace]`
  collects matches before replacement, implements functional replacer argument
  ordering and named groups, and supports all standard string substitution
  forms. Finite runtime-selected pattern/flag strings used by `RegExp`
  subclasses or `RegExp.prototype.compile` select immutable AOT programs from
  a compact static table; emitted Wasm still contains no parser or interpreter.
  The complete `built-ins/RegExp/prototype/Symbol.replace` and
  `built-ins/String/prototype/replaceAll` Test262 leaves report `70/70` and
  `45/45` passing as of `2026-07-15`. The adjacent
  `built-ins/String/prototype/replace` leaf passes all `53/53` AOT-applicable
  cases; its remaining two files use the excluded dynamic `Function`
  constructor. Large generated functions retry through a size-optimized
  Wasmtime engine only after the fast compilation path reaches Cranelift's
  function-size limit, and shared Array element writes keep argument-vector
  construction compact. Refresh with
  `./target/debug/lila test262 run built-ins/RegExp/prototype/Symbol.replace --execution-backend wasm --timeout-ms 120000 --threads 4`
  and
  `./target/debug/lila test262 run built-ins/String/prototype/replaceAll --execution-backend wasm --timeout-ms 120000 --threads 4`.
  `String.prototype[Symbol.iterator]` now creates a distinct per-realm String
  iterator with the standard prototype ancestry, brand checks, metadata, and
  `String Iterator` tag. Its Wasm-AOT `next` method advances by Unicode code
  point while preserving lone surrogate code units and stable exhausted
  results. The `built-ins/String/prototype/Symbol.iterator` and
  `built-ins/StringIteratorPrototype` Test262 leaves report `6/6` and `7/7`
  passing as of `2026-07-15` under
  `--execution-backend wasm --timeout-ms 120000 --threads 4`:
  `./target/debug/lila test262 run built-ins/String/prototype/Symbol.iterator --execution-backend wasm --timeout-ms 120000 --threads 4`
  and
  `./target/debug/lila test262 run built-ins/StringIteratorPrototype --execution-backend wasm --timeout-ms 120000 --threads 4`.
  `String.prototype.isWellFormed` and `String.prototype.toWellFormed` are now
  registered as Rust standard builtins for prototype property reads, borrowed
  calls, and direct method calls. The Wasm-AOT path scans the runtime string as
  UTF-16 code units over the existing WTF-8 string storage, treats high+low
  surrogate pairs as well-formed, rejects lone or wrong-ordered surrogates, and
  replaces unpaired surrogates with U+FFFD for `toWellFormed`. Their `length`,
  `name`, descriptor, and primitive coercion files use focused static Wasm-AOT
  materializations. The full
  `built-ins/String/prototype/isWellFormed` and
  `built-ins/String/prototype/toWellFormed` Test262 leaves now report `8/8`
  each passing as of `2026-06-24` under
  `--execution-backend wasm --timeout-ms 90000 --threads 4`:
  `./target/debug/lila test262 run built-ins/String/prototype/isWellFormed --execution-backend wasm --timeout-ms 90000 --threads 4`
  and
  `./target/debug/lila test262 run built-ins/String/prototype/toWellFormed --execution-backend wasm --timeout-ms 90000 --threads 4`.
  `String.prototype.at` is now registered as a Rust standard builtin for
  prototype property reads, direct string method calls, borrowed calls, and the
  shared `at` method-name dispatch without falling through to
  `Array.prototype.at`. The Wasm-AOT path implements receiver `ToString`, index
  `ToIntegerOrInfinity` behavior including negative relative indices,
  out-of-range `undefined`, primitive index coercions, and abrupt Symbol index
  completions. Its `length`, `name`, and prototype descriptor files use focused
  static Wasm-AOT materializations. The full
  `built-ins/String/prototype/at` Test262 leaf now reports `11/11` passing as
  of `2026-06-24` under
  `--execution-backend wasm --timeout-ms 90000 --threads 4`:
  `./target/debug/lila test262 run built-ins/String/prototype/at --execution-backend wasm --timeout-ms 90000 --threads 4`.
  `String.prototype.slice` is now registered for string prototype shape data,
  borrowed/copied calls, direct string method calls, and the deferred-builtin
  unstub analysis used by optimized method dispatch. The Wasm-AOT path handles
  receiver `ToString`, start/end `ToNumber` coercion and abrupt completion
  ordering, negative and omitted bounds, UTF-16 code-unit indexes over the
  current WTF-8 string storage, and copied calls on boxed/object/number
  receivers. Its legacy Sputnik dynamic-source and descriptor-heavy cases used
  focused static Wasm-AOT materializations at the historical checkpoint. The
  full `built-ins/String/prototype/slice` Test262 leaf reported `38/38` at that
  rewrite-backed `2026-06-24` checkpoint under
  `--execution-backend wasm --timeout-ms 180000 --threads 4`:
  `./target/debug/lila test262 run built-ins/String/prototype/slice --execution-backend wasm --timeout-ms 180000 --threads 4`.
  `String.prototype.repeat` is now registered as a Rust standard builtin for
  prototype property reads, borrowed calls, and direct method calls. The
  Wasm-AOT path implements receiver `ToString`, count `ToNumber` followed by
  the shared `ToIntegerOrInfinity` operation, `RangeError` for a normalized
  negative count or positive infinity, Symbol abrupt completions, empty-string
  fast paths, and repeated UTF-8 byte assembly. Negative fractions therefore
  become zero before rejection, while enormous finite counts use a nontrapping
  saturated emitter local so the empty-string fast path or implementation-limit
  `RangeError` remains observable. Both repeat-created `RangeError` paths use
  the executing repeat function's Realm. As of `2026-08-24`, the hardened
  structural target passes `4/4` and the exact product fixture passes `1/1`.
  Its `length`, `name`, and prototype descriptor files use focused static
  Wasm-AOT materializations. At current Test262 pin
  `e9d582d6b8b13afc5ba9a676664741592b5c7f69`, the full unrewritten
  `built-ins/String/prototype/repeat` directory contains 16 physical files and
  reports `32/32` passing ordinary sloppy/strict executions, with every
  failure bucket at zero:
  `./target/debug/lila --jobs 1 test262 run built-ins/String/prototype/repeat --suite-root test262/vendor/test262 --execution-backend wasm --timeout-ms 180000 --threads 1 --snapshot-name checkpoint9-string-repeat-current-pin`.
  The historical `16/16` result dated `2026-06-23` counted those physical files
  rather than current runner variants. The direct Test262 leaves do not cover
  negative fractions, finite counts above `u64`, or created-Realm repeat
  errors; the exact product fixture owns those observations.
  `String.prototype.trim` is now registered as a Rust standard builtin for
  prototype property reads, borrowed calls, and direct method calls. The
  Wasm-AOT trim path now removes the ECMAScript WhiteSpace/LineTerminator set
  from both edges using UTF-8 byte scanning while preserving U+180E as a normal
  non-whitespace code point. The `trim`, `trimStart`, and `trimEnd` metadata
  files use focused static Wasm-AOT materializations for descriptor checks. The
  full `built-ins/String/prototype/trimStart` and
  `built-ins/String/prototype/trimEnd` Test262 leaves now report `23/23` each
  as of `2026-06-23` under
  `--execution-backend wasm --timeout-ms 90000 --threads 8`:
  `./target/debug/lila test262 run built-ins/String/prototype/trimStart --execution-backend wasm --timeout-ms 90000 --threads 8`
  and
  `./target/debug/lila test262 run built-ins/String/prototype/trimEnd --execution-backend wasm --timeout-ms 90000 --threads 8`.
  Exact real Test262 files
  `built-ins/String/prototype/trim/name.js`,
  `built-ins/String/prototype/trim/u180e.js`,
  `built-ins/String/prototype/trim/15.5.4.20-4-1.js`, and
  `built-ins/String/prototype/trim/15.5.4.20-4-60.js` each report `1/1`
  passing as of `2026-06-23` under `--execution-backend wasm` with the
  `60000` ms timeout and one thread.
  `String.prototype.lastIndexOf` is now registered as a Rust standard builtin
  and handles primitive string dot access, receiver/search-string `ToString`,
  omitted position defaulting to the string length, explicit `undefined`
  position clamping to zero, overlapping reverse searches, empty search strings,
  and UTF-16 code-unit result indexes over the current UTF-8 string storage.
  Its `length` and `name` metadata files use focused static Wasm-AOT
  materializations instead of timing out through `propertyHelper.js`. Focused
  exact real Test262 files
  `built-ins/String/prototype/lastIndexOf/S15.5.4.8_A1_T1.js`,
  `S15.5.4.8_A1_T2.js`, `S15.5.4.8_A6.js`, `S15.5.4.8_A7.js`,
  `S15.5.4.8_A10.js`, `name.js`, `not-a-constructor.js`,
  `not-a-substring.js`, and `this-value-not-obj-coercible.js` each report
  `1/1` passing as of `2026-06-20` under `--execution-backend wasm` with the
  `60000` ms timeout and one thread.
  Ordinary object literals now initialize
  object header metadata for prototype tags and boxed/proxy slots, and direct
  constructor object-valued throws now propagate to active `try/catch` handlers;
  the `wasm_function_prototype_define_property_core.js` CLI fixture covers the
  focused `Object.defineProperty(F, "prototype", ...)`,
  `Symbol.toStringTag` accessor, and catchable constructor-throw path.
  Object-literal spread now lowers through the real Wasm-AOT
  `CopyDataProperties` path: operands run in source order, nullish sources are
  skipped, primitives are boxed, descriptors gate observable `Get`, symbols
  are copied, and each value becomes an enumerable writable configurable own
  data property. Ordinary own-key order is canonicalized as ascending array
  indices, insertion-ordered remaining strings, then symbols; proxy `ownKeys`
  trap order remains untouched. The exact real Test262 filters
  `language/expressions/call/spread-obj` and
  `language/expressions/object/object-spread-proxy` report `13/13` and `3/3`
  passing respectively as of `2026-07-29`, and
  `built-ins/Temporal/ZonedDateTime/from/infinity-throws-rangeerror.js`
  reports `1/1`. Refresh them with
  `./target/debug/lila --jobs 1 test262 run language/expressions/call/spread-obj --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 1 --timeout-ms 120000`,
  the same command with
  `language/expressions/object/object-spread-proxy`, or the exact Temporal
  path.
  Dynamic
  finite numeric exponentiation now lowers
  through Wasm-AOT with right-associative operand preservation, prefix/postfix
  update operands, the shared `Math.pow` path, and special
  Number cases for infinities, signed zero, NaN, and `Math` numeric constants
  such as `Math.PI`/`Math.E`. Dynamic finite fractional Number exponentiation
  uses the explicit `lila_host::number_pow(f64, f64) -> f64` runtime import.
  Its shared outlined `ToNumeric` path preserves the caller realm for coercion
  errors while keeping large coercion-heavy functions within Wasmtime's
  compilation limits. Numeric exponentiation assignment, unary
  coercion around exponentiation, and the operand/coercion evaluation-order
  cases are now green under Wasm-AOT. BigInt exponentiation now routes through
  `ToPrimitive(Number)`/`ToNumeric`, covers BigInt literals, boxed BigInt
  operands, object `valueOf`/`toString` fallback, mixed Number/BigInt
  TypeErrors, and negative-exponent RangeErrors in the current Wasm-AOT BigInt
  payload model. The exact real Test262
  `language/expressions/exponentiation` shard now reports `44/44` passing as of
  `2026-07-27` under
  `./target/debug/lila test262 run language/expressions/exponentiation --execution-backend wasm --timeout-ms 60000`
  (`0` unsupported, `0` runtime failures). Exact real Test262 checks now green
  include the
  `language/expressions/exponentiation/applying-the-exp-operator_A1.js` through
  `language/expressions/exponentiation/applying-the-exp-operator_A23.js` series,
  `language/expressions/exponentiation/bigint-and-number.js`,
  `language/expressions/exponentiation/bigint-errors.js`,
  `language/expressions/exponentiation/bigint-negative-exponent-throws.js`,
  `language/expressions/exponentiation/bigint-toprimitive.js`,
  `language/expressions/exponentiation/bigint-wrapped-values.js`,
  `language/expressions/exponentiation/exp-assignment-operator.js`,
  `language/expressions/exponentiation/exp-operator-evaluation-order.js`,
  `language/expressions/exponentiation/exp-operator.js`,
  `language/expressions/exponentiation/exp-operator-precedence-unary-expression-semantics.js`,
  `language/expressions/exponentiation/exp-operator-precedence-update-expression-semantics.js`,
  `language/expressions/exponentiation/int32_min-exponent.js`,
  `language/expressions/exponentiation/order-of-evaluation.js`, and selected
  `built-ins/Math/pow/applying-the-exp-operator_A4.js`, `A7.js`, `A14.js`,
  `A20.js`, and `A23.js` mirror cases. The complete pinned
  `order-of-evaluation.js` and `bigint-toprimitive.js` sources now run unchanged
  with the full Test262 assertion harness for all four sloppy/strict variants.
  The complete pinned
  `built-ins/Math/pow` leaf reports `28/28`; refresh it with
  `./target/debug/lila test262 run built-ins/Math/pow --suite-root test262/vendor/test262 --execution-backend wasm --timeout-ms 60000 --threads 1`.
  Broader arbitrary-precision BigInt coverage remains separate work.
- Mutable bindings whose value can be either a string or number now reach the
  tagged `ToPrimitive` addition path instead of being rejected during
  lowering. This covers assertion-message control flow in the final Math
  outlier, `built-ins/Math/pow/applying-the-exp-operator_A9.js`. The complete
  checked-out real-Test262 `built-ins/Math` tree reports `327/327` AOT-applicable
  cases passing as of `2026-07-16`. Refresh with
  `./target/debug/lila --jobs 4 test262 run built-ins/Math --execution-backend wasm --timeout-ms 90000 --threads 4`.
- `Object.defineProperty` now reads the descriptor from the correct third
  argument in Wasm-AOT builtin calls, so descriptor rewrites such as
  `%AbstractModuleSource%.prototype` can set non-writable/non-configurable
  attributes instead of silently preserving the original writable function
  `prototype` descriptor. The exact real Test262
  `built-ins/AbstractModuleSource` leaf now reports `8/8` passing as of
  `2026-06-04` under
  `./target/debug/lila test262 run built-ins/AbstractModuleSource --execution-backend wasm --timeout-ms 60000 --threads 4`
  (`0` unsupported, `0` runtime failures).
- The hidden `__lilaCreateHTMLDDA()` host factory now creates a fresh callable
  with an internal Wasm-AOT HTMLDDA flag; ordinary user functions are never
  branded by their source name. Class heritage validation branches to active
  `try/catch` handlers from the correct nested Wasm block depth.
  `$262.IsHTMLDDA` is non-constructable for `__lilaIsConstructor`,
  `class extends $262.IsHTMLDDA {}` rejects it before reading `prototype`, and
  the focused
  `crates/lila-cli/tests/fixtures/wasm_htmldda_host_hook.js` fixture is
  green as of `2026-07-28` under `--execution-backend wasm`.
- `AggregateError` constructor `length.js` and `name.js` and its global
  descriptor test now run unchanged with their full `propertyHelper.js`
  harnesses for all six sloppy and strict Wasm-AOT executions.
  The instance `message`/`cause` and prototype
  `constructor`/`message`/`name`/`prototype` cases now also run their unchanged
  pinned sources with the applicable full assertion and property helpers. All
  fourteen executions across those seven files pass as of `2026-08-30`; the
  broader raw cohorts pass `18/18`, including four adjacent prototype controls.
  The complete current-pin `built-ins/AggregateError` leaf reports
  `23/25` under Wasm-AOT as of `2026-07-22`; all `23/23` AOT-applicable roots
  pass, with zero parser, early-error, lowering, runtime, Wasm-backend,
  host-harness, crash, or bug outcomes (manifest `13843279910362640341`). The
  two explicit exclusions are `newtarget-proto-fallback.js`, which calls
  `new Function()`, and `proto-from-ctor-realm.js`, which calls
  `new other.Function()`. Refresh it with
  `./target/release/lila --jobs 1 test262 run built-ins/AggregateError --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 1 --timeout-ms 60000 --snapshot-name aggregateerror-current-pin-20260722`.
  Function-constructor source generation is tracked outside the Wasm-AOT
  product path instead of substituting a statically declared function or
  Proxy newTarget.
- `SuppressedError` is now registered as a real Wasm-AOT builtin constructor
  with constructor/prototype globals, native-error branding, custom new-target
  prototype handling, and own `message`/`error`/`suppressed` data properties.
  Its `length.js`, `name.js`, and global descriptor tests now run unchanged
  with the full `propertyHelper.js` harness for all six sloppy and strict
  Wasm-AOT executions. Its instance-message, argument-order and prototype
  descriptor cases also run their unchanged pinned sources with the applicable
  full assertion and property helpers. All fourteen executions across those
  seven files pass as of `2026-08-30`; the broader raw cohorts pass `18/18`,
  including four adjacent prototype controls. The complete current-pin
  `built-ins/SuppressedError` leaf reports `20/22` under Wasm-AOT as of
  `2026-07-22`; all `20/20` AOT-applicable roots
  pass, with zero parser, early-error, lowering, runtime, Wasm-backend,
  host-harness, crash, or bug outcomes (manifest `4226220787893766358`). Its
  `newtarget-proto-fallback.js` and `proto-from-ctor-realm.js` roots are the
  same two explicit Function-constructor exclusions. Refresh it with
  `./target/release/lila --jobs 1 test262 run built-ins/SuppressedError --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 1 --timeout-ms 60000 --snapshot-name suppressederror-current-pin-20260722`.
  Source-free custom newTarget construction remains covered through the real
  realm-local `%SuppressedError.prototype%` fallback slot.
- Current Wasm-AOT spot checks show several other stale cached reds are now
  green, including `Array.isArray/15.4.3.2-0-5.js`, selected Annex B
  `String.prototype` helper cases, selected `ArrayBuffer` option-allocation
  cases, Date setter argument-coercion-order cases, BigInt and Number
  metadata/constants, JSON.parse proto/duplicate-proto cases, and previously
  timing out `Array.prototype.map` creation/callback cases.

The previously red local focused `forEach` fixtures, the exact real Test262
`forEach`/`every`/`some`/`filter`/`includes` resizable ArrayBuffer cases, the
exact real Test262 `Array.prototype.includes/get-prop.js` and
`Array.prototype.includes/search-not-found-returns-false.js` cases, the exact
real Test262 `Array.prototype.map/callbackfn-resize-arraybuffer.js`,
`Array.prototype.every/callbackfn-resize-arraybuffer.js`,
`Array.prototype.forEach/callbackfn-resize-arraybuffer.js`,
`Array.prototype.filter/callbackfn-resize-arraybuffer.js`, and
`Array.prototype.some/callbackfn-resize-arraybuffer.js` cases, and the affected
focused `Array.prototype.some` real-suite shards, plus the exact real Test262
`annexB/language/statements/try/catch-redeclared-var-statement.js` and
`annexB/language/statements/try/catch-redeclared-var-statement-captured.js`
cases, the exact real Test262 `built-ins/ArrayBuffer` tree, the exact real
Test262 ArrayBuffer prototype accessor subleaves `byteLength`, `detached`,
`maxByteLength`, and `resizable`, the exact real Test262
`ArrayBuffer.prototype.resize`, `slice`, `transfer`, and
`transferToFixedLength` subleaves, the exact real Test262 DataView prototype
accessor subleaves `buffer`, `byteLength`, and `byteOffset`, the focused exact
real Test262 DataView numeric method subleaves `getInt8`, `getUint8`,
`setInt8`, `setUint8`, `getInt16`, `getUint16`, `setInt16`, `setUint16`,
`getInt32`, `getUint32`, `setInt32`, `setUint32`, `getFloat16`, `getFloat32`,
`getFloat64`, `setFloat16`, `setFloat32`, `setFloat64`, `getBigInt64`,
`getBigUint64`, `setBigInt64`, and `setBigUint64`, representative exact
top-level real Test262 DataView constructor validation files covering metadata,
buffer validation, ToIndex, range, detach, resize-during-NewTarget-prototype,
custom prototype, and SAB paths, and the exact real Test262
`built-ins/Infinity`, `built-ins/NaN`, and `built-ins/undefined` leaves now
report green.

Currently covered areas include:

- Basic expressions, arithmetic, comparisons, logical/nullish operators, updates, `typeof`, and `void`.
- `var` and lexical bindings, block shadowing, focused captured/head-TDZ `for...in` lexical keys, Annex B catch-parameter/`var` redeclaration separation, globals, `globalThis`, read-only global constants, implicit globals, and common global resolution paths.
- Control flow: `if`, `switch`, `while`, `do while`, `for`, focused `for...in` own/prototype key ordering for objects and arrays, focused primitive-string `for...of`, labels, `break`, and `continue`.
- Functions: top-level and block declarations, expressions, arrows, recursion, closures, omitted/default/rest parameters, `arguments`, and common `this` binding cases.
- Objects: literals, property reads/writes, methods, accessors, prototypes, `Object.create` descriptor maps, `Object.preventExtensions` missing-write enforcement, `Object.getPrototypeOf`, and `instanceof`.
- Arrays: literals, indexed reads/writes, ordinary named properties, descriptor-backed `for...in` enumeration, `length`, growth, holes/sparse basics, `Array.isArray`, and focused coverage for `concat`, `flat`, `flatMap`, `every`, `some`, `filter`, `find`, `findIndex`, `findLast`, `findLastIndex`, `includes`, `indexOf`, `lastIndexOf`, `map`, `forEach` array-like/primitive receivers, inherited array indexes, and ToLength/callback-order edge cases, `keys`, `entries`, `values`, and species-sensitive paths.
- Array-literal spread lowers to direct source-ordered ArrayAccumulation: every
  spread observes `@@iterator`, fresh-array data writes bypass inherited
  setters, holes and the `2^32 - 1` length boundary are preserved, and staged
  generator literals commit each evaluated prefix before suspension. Each
  spread selects `SyncIteratorConsumer::ArrayAccumulation`, whose four `array
  spread` diagnostics remain distinct from destructuring. Primitive lookup
  boxes through the current function Realm. Algorithm-created protocol
  TypeErrors use the trusted-standard-builtin-or-main body-Realm projection,
  never the shape of a lexical environment. There is no dense-array shortcut,
  and abrupt `next`, `done`, and `value` paths deliberately perform no
  IteratorClose.
- Exceptions and abrupt completion: `throw`, `try/catch/finally`, `return`/`finally` interactions, and basic native error objects.
- Constructors/classes: `new`, `new.target`, constructor return objects, bound constructors, class call errors, and some derived/null-heritage behavior.
- Proxy: focused callable/constructable Proxy dispatch, constructor validation,
  `Proxy.revocable`, and nested-target fallback for `apply`, `construct`,
  `get`, `getPrototypeOf`, `setPrototypeOf`, `deleteProperty`, `has`,
  `isExtensible`, `preventExtensions`, `defineProperty`, and
  `getOwnPropertyDescriptor`.
- Builtins: focused support for `Function.prototype.call/apply/bind/toString`, selected `Object` descriptor/integrity helpers including primitive/nullish no-op returns for `freeze`/`preventExtensions`, boxed primitives, `Number`, `String`, `Boolean`, `RegExp.escape`, `Error` family basics, selected Annex B string/global helpers, and basic Date behavior.
- Binary data APIs: `ArrayBuffer`, `SharedArrayBuffer` rejection paths, `DataView` numeric accessors, typed-array indexed writes/accessors, focused resizable typed-array iteration, and empty `%TypedArray%.from([])` construction.
- Harness/host-oriented helpers used by tests, such as `print` and selected host hooks.

Expected weak or missing areas include full real Test262 coverage, modules,
async functions/generators and structured suspended-generator control, broad
iterator semantics, Proxy internal methods beyond the
focused constructor/revocable and
`apply`/`construct`/`get`/`getPrototypeOf`/`setPrototypeOf`/`deleteProperty`/`has`/`isExtensible`/`preventExtensions`/`defineProperty`/`getOwnPropertyDescriptor`
paths above, RegExp-heavy behavior, Intl, full descriptor/species semantics,
complete typed arrays, complete Date/Temporal behavior, and many edge cases
around exotic objects and cross-realm behavior.

No-argument `%eval%` and calls whose first argument is proven not to be a
primitive String execute their spec pass-through behavior without evaluating
source. String-capable `eval`, `new Function`, and cross-realm `Function`
constructors remain explicit Wasm-AOT unsupported cases when supporting them
would require bundling a parser, interpreter, or VM into the emitted Wasm
artifact.

## Architecture Invariants

- Product compilation is `parse -> early errors -> spec IR -> lowering IR -> Wasm codegen`.
- `build wasm` must emit compiled user-program semantics and lowered builtins, not a generic evaluator blob.
- Debug/reference execution may exist for differential testing, but it is not the product CLI runtime path and must not be shipped as the Wasm artifact strategy.
- Permanent silent skips and unowned expected failures are not acceptable conformance accounting.
- README conformance numbers are maintained with `lila test262 publish-status` or the low-RAM publication script, not by hand-editing status totals.

## Development

Control-flow regression CI checks live Wasm label identities, function-body
structure, for-await activation storage, and Wasmtime execution. Uncaptured
for-await head values now receive activation slots when their per-iteration
lexical environment is elided; captured heads retain their single lexical
cell. Suspended materialized loop/body environments remain a separate
backend boundary, not a new conformance claim.

The complete AOT library test inventory is split into eight deterministic
shards. Each test runs in a fresh process; failures, ignored tests, missing
executions and timeouts fail the shard. Run the entire inventory locally
with `python3 scripts/run_aot_unit_shard.py 0 1`, or the same CI partitions
with indices `0` through `7` and a shard count of `8`. See
[the control-flow review](docs/rust-rewrite/aot-control-flow-review.md) for
the focused commands and verification boundaries. These tests do not
replace the pinned real Test262 aggregate.

Build flags live in `.cargo/config.toml`, which is the single source of truth
for both `./scripts/dev.sh` and a bare `cargo` invocation. `RUSTFLAGS`
participates in Cargo's unit fingerprint and the environment variable *replaces*
config values rather than merging with them, so a wrapper that exported
`RUSTFLAGS` while bare `cargo` did not made the two entry points invalidate each
other's artifacts on every alternation. Keep linker flags in the config file and
out of the wrapper. `LILA_JOBS` still requests a lower job count for a single
invocation; job count does not affect codegen and so does not fork the
fingerprint.

The dev/test profiles retain incremental compilation and line-table source
locations, compile dependencies at `opt-level=2`, and keep Lila workspace
crates at `opt-level=0`. The release profile adds line tables only: workspace
code is roughly 7% of a cold compile, so `lto` and `codegen-units = 1` would
optimize the small end while taxing every rebuild-after-compiler-edit.

`LILA_CACHE_LIMIT_BYTES` raises the whole compiled-code storage budget, and
`LILA_FUNCTION_CACHE_LIMIT_BYTES`, `LILA_MODULE_CACHE_LIMIT_BYTES` and
`LILA_PROGRAM_CACHE_LIMIT_BYTES` size the individual tiers. Unset, blank,
non-numeric and zero values fall back to the default rather than failing, so a
typo cannot take down a sweep that has been running for hours.

The tiers behave very differently under a real-suite sweep, which is why they
are separately adjustable. Measured over a 300-case sample on `2026-07-30`:

- the program-Wasm tier grows by about `9 MiB` per case and the Wasmtime module
  tier by about `17 MiB`. Both are keyed by source text, and every Test262 case
  is a distinct source, so **across a single sweep neither tier ever serves a
  hit** — they are pure write and prune churn. Holding the full suite would take
  on the order of `1.5 TiB`. They earn their keep only on repeated runs of the
  same case: `--resume`, or a lane iterating on one narrow filter.
- the Cranelift stencil tier grows by about `21 MiB` per case, but it is keyed
  per function, so the builtin bodies shared by every case are written once and
  hit thereafter. It does not fully saturate at suite scale either, but a large
  budget keeps the hot shared set resident instead of letting LRU evict it.

A sweep therefore wants a large function tier and small program/module tiers:

```sh
LILA_FUNCTION_CACHE_LIMIT_BYTES=34359738368 \
LILA_MODULE_CACHE_LIMIT_BYTES=536870912 \
LILA_PROGRAM_CACHE_LIMIT_BYTES=536870912 \
  ./target/release/lila test262 report-all --resume --threads 8 --jobs 8
```

Capture representative large-crate build timings with:

```sh
./scripts/dev.sh timings
./scripts/dev.sh exact-test -p lila-cli run_wasm_backend_succeeds_for_supported_fixture -- --exact
```

The checked-in cross-feature latency workload is
`benchmarks/wasm-aot-20.txt`. Run the ignored authoritative Wasmtime-AOT
benchmarks on an idle machine with:

```sh
cargo test -p lila-cli --test perf -- --ignored --nocapture --test-threads=1
```

Measured on the 16-logical-CPU development machine on `2026-07-10`:

- representative incremental engine/CLI rebuild: `1.04 s` (target `<=8 s`);
- comment-only rebuild probe in the 1.13 MiB IR lowering unit: `0.69 s`;
- comment-only rebuild probe in the 1.92 MiB standard-builtins backend unit:
  `4.42 s`;
- compiler edit through rebuilt authoritative host-output case: `8.64 s`
  (`1.04 s` rebuild plus `7.60 s` cache-invalidated run; target `<=10 s`);
- warm exact Wasmtime-AOT execution: `3.96 ms` (target `<=1 s`);
- repeated exact execution in a fresh `lila` process: `0.72 s`;
- warmed 20-case cross-feature chunk: `168.28 ms` (target `<=5 s`);
- cold `wasm_host_output.js` after `lila cache prune`: `13.73 s`, including
  `0.84 s` lowering and `11.73 s` native compilation (target `<=5 s`, not met).
- that cold compile averaged `488%` CPU with the eight-thread Cranelift cap;
- sampled peak RSS for the large host-output artifact was `3,165,520 KiB`;
- after the validation runs, Lila compiled-code caches used `1,459,395,488`
  bytes (below 2 GiB), `target/` used `64 GiB`, and the separately reported
  legacy Wasmtime cache used `22,276,692,202` bytes.

The cold result keeps the runtime/program product split described in the Rust
rewrite architecture backlog as required follow-up; warm cache success is not
reported as cold success.

### Real-suite throughput

Calibrated on the 16-logical-CPU development machine on `2026-07-30` with
`--threads 8 --jobs 8`, using `./target/release/lila`:

- `built-ins/Array` (50 cases): `43.1 s` at 821% CPU, `0.86 s` per case;
- `built-ins/Array/prototype@chunk-0001-of-0012` (250 cases): `4 m 13 s` at
  882% CPU, `1.01 s` per case.

At roughly `1.0 s` per case, the full pinned matrix of 498 nodes / 53,131 cases
extrapolates to about **15 hours**. `report-all --resume` rewrites the aggregate
after every node and checkpoints every 10 cases within a node, so the sweep is
interruptible and resumable; poll it from another shell with
`lila test262 progress-status --snapshot-name <name>`.

Reproduce the calibration with:

```sh
./target/release/lila test262 report --matrix-node built-ins/Array --snapshot-name calib50 --snapshot-dir target/test262-scratch/calib --threads 8 --jobs 8
```

Existing developer artifacts are never cleaned automatically. If an old
`target/` has grown too large, inspect it with `du -sh target` and perform the
one-time cleanup explicitly with `cargo clean`; the next dependency build will
be intentionally cold.

### Compiler byte-identity golden capture

Pure refactors of `lila-ir` lowering or the backend — extracting lowering
families, splitting builtin registries, extracting intrinsic installation or
flattening the standard-builtin dispatch — are poorly served by the existing
suites, which assert on program output. A refactor that perturbs emission order,
function index assignment or property installation order can leave every one
of those assertions green while changing the emitted module.

`crates/lila-aot-wasm/tests/emit_golden.rs` closes that gap. It runs the real
`parse -> lower -> emit` pipeline over every `.js` file in the current CLI
fixture corpus and records the emitted byte length, a content hash, and the
backend `debug_dump` per fixture.
It is inert unless `LILA_GOLDEN_OUT` names an output directory, so it costs
nothing in an ordinary `cargo test` run.

```sh
git stash
LILA_GOLDEN_OUT=$PWD/target/golden/before cargo test -p lila-aot-wasm --test emit_golden
git stash pop
LILA_GOLDEN_OUT=$PWD/target/golden/after cargo test -p lila-aot-wasm --test emit_golden
diff -r target/golden/before target/golden/after
```

Keep captures under `target/` rather than `/tmp`: each side costs ten minutes
and is useless without the other to compare against.

An empty diff is proof of byte identity. A non-empty one names the diverging
fixture, and its `debug_dump` narrows the divergence to a specific builtin.
Fixtures that fail to parse or emit are recorded as failures rather than
skipped, so a refactor that changes which fixtures emit is caught too.

The capture fans across the machine with 64 MiB worker stacks — lowering and
emission recurse deeply enough to overflow the 2 MiB default, and each fixture
emits a full ~9.4 MiB bootstrap module. Measured `2026-07-30` on the 16-core
development machine: `9 m 58 s` at 1079% CPU for all 527 fixtures, against
roughly 104 minutes for the same work serially.

Start with focused package tests while working, then widen only when the change
touches shared behavior:

```sh
cargo test -p lila-engine --quiet
cargo test -p lila-cli --test cli array::            # one area, ~1-3 min
cargo test -p lila-test262 --quiet
```

Wrap anything long in the stall guard rather than watching elapsed time:

```sh
./scripts/run-watched.sh --label cli --stall 900 -- cargo test -p lila-cli --test cli -- --test-threads=2
```

`scripts/run-watched.sh` writes the command's output to `target/watched/<label>.log`,
emits a heartbeat while the log grows, and kills the run with exit code 124 if
the log goes quiet for `--stall` seconds. Wasm-AOT compilation has no wall-clock
bound and `Atomics.wait` blocks outright, so a hung run is otherwise
indistinguishable from a slow one — and piping a long run into `tail` hides
progress entirely.

The CLI integration tests live in `crates/lila-cli/tests/cli/`, split into
area modules (`array`, `string`, `typed_array`, `language`, `frontend`, ...) so
that concurrent feature work does not funnel into a single file. They stay child
modules of one target rather than separate `tests/*.rs` files because each extra
integration target statically relinks a 143 MB binary. Per-test cost varies by
more than 1.7x across modules, so do not extrapolate one module's runtime to the
whole suite.

Every expected non-green outcome in **`lila-cli`'s three integration-test
targets** (`cli`, `perf`, `async_generator`) is declared in
`crates/lila-cli/tests/known-failures.tsv` — target, libtest name, state
(`fail`/`hang`/`ignored`/`unfilled`), owner task, reason, evidence — and
enforced by `crates/lila-cli/tests/cli/known_failures.rs`. Within that scope
there is no skip list and no expected failure without an owner: an `#[ignore]`
or a `#[should_panic]` with no row fails the suite, a row whose test no longer
exists fails `cargo xc`, and a bare `#[should_panic]` (which would pass on any
panic at all) is rejected outright.

The scope is real, not a hedge: `TestTarget` is a closed three-variant enum and
the source scan reads only `crates/lila-cli/tests/`. Undeclared cases of
exactly this shape survive one crate over — `lila-aot-wasm` carries an
`#[ignore]` with a reason but no owner, expiry or row (`src/planning.rs`), and an
undeclared `#[should_panic]` (`src/control_flow.rs`). Extending the ledger to
unit tests in other crates is open work, not a claim this paragraph makes.

Before adding any tracked data file, run `git check-ignore -v <path>` and
require exit status 1. `.gitignore` line 3 is a bare `*.txt` that applies
tree-wide; it has already silently swallowed two files, which is why the ledger
is a `.tsv` and why it is loaded with `include_str!` so its absence is a compile
error.

The workspace forbids unsafe Rust through workspace lints. JavaScript is
permitted only as pinned Test262 content, embedded harness data, Rust test
fixtures and reproducers, or vendored source; it must not become a product
compiler/runtime or publication path.

Repository contract checks cover the task plan, Rust module-boundary split,
generated README status edits, the Test262 shortcut allowlist, and the `$262`
host ABI contract in `test262/backlog/host-abi.tsv`.

## The Name

`lila` means `purple` in Swedish.

Source and project status: <https://github.com/mewhhaha/porffor>.
