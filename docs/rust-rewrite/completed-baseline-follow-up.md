# Completed baseline follow-up — 2026-09-14

This work starts from freshly fetched `origin/main` at
`2abe452111099c84d8c0dbb8dab15db1a4de7aa1`, the merge of PR #51.
The repository has no `master` branch; `main` is its default branch.

The historical `c5115bf03` Wasm-AOT sweep completed all 102,043 execution
identities and 744 matrix nodes: 87,641 Success, 9,212 Bug, 957 Crash, and
4,233 NotImplemented. Its Test262 tree is
`aa55200d1310384c5cf69ea95b2a2ecba457007b`.
Completion means the entire baseline was measured; it does not mean the
compiler conforms to all of Test262.

All 14,402 historical failures are retained for replay against the merged
compiler. The inventory also retains all 87,641 historical successes for
regression verification. The completed observation preserves the previous
80,675 execution identities and outcomes and adds 2,024 failing executions.
No source paths, unsupported outcomes, or ambiguous staging ownership are
discarded from this inventory.

The first 50-case merged-main replay completed with 44 Bug and 6 Success.
It reproduces 20 BigInt relational failures, 22 function-name failures and
two NUL parsing failures. Two ordinary function-name executions and four
`with` numeric-update executions already pass and remain controls. The
remaining 14,352 historical failures are being replayed separately, starting
with representatives of distinct diagnostics. Their pending results must not
be reported as current compiler failures or passes.

## Expression and declaration batch

- BigInt/string relational comparisons use exact integer parsing in both
  operand orders. Invalid integer strings are unordered for all four
  operators. Other primitive operands convert to Number without converting
  the BigInt. Symbol errors follow evaluation of both operands.
- Methods and accessors derive their names from the actual property key,
  including Symbol descriptions and getter/setter prefixes. Computed anonymous
  function values receive inferred names. Anonymous class names exist before
  static initialization, with their normalized key retained across an awaited
  heritage expression. Static `name` members can still replace the initial
  name. See [the naming contract](contracts/function-name-evaluation.md).
- `var` declarations inside `with` participate in variable hoisting, including
  declarations in unexecuted branches. Ordinary initialized identifier
  declarations select their reference before evaluating the initializer and
  preserve empty declaration completion. Classic `for (var …)` heads reuse
  that path. Specialized suspension and destructuring initialization remain
  separate lowering paths.
- The parser accepts literal NUL characters where the JavaScript grammar
  permits them, including strings, templates, regular expressions and comments.
  NUL between tokens remains a syntax error, and embedded NUL does not hide
  later source text.
- Private prefix/postfix increment and decrement retain the original base
  across the getter, numeric coercion and setter. The existing private
  reference and numeric-update operations preserve Number/BigInt results,
  brand checks and abrupt completions.

## Namespace, enumeration, calendar and harness batch

- Namespace construction uses trusted linker metadata and explicit IR. Runtime
  internal methods read live export bindings and return data descriptors,
  including TDZ errors and deferred evaluation triggers. See
  [the namespace contract](contracts/module-namespace-internal-methods.md).
  The deferred module activation lifecycle still requires linker work; this
  batch does not claim the whole module family.
- `for-in` enumerates each prototype level lazily, observes current descriptors,
  and retains visited string names, including non-enumerable shadowing names.
  All assignment-head forms share that algorithm and preserve completion values.
  See [the enumeration contract](contracts/for-in-enumeration.md).
- Buddhist calendar arithmetic projects calendar years to ISO storage and back
  through a typed calendar definition. Gregorian-family non-ISO week getters
  return undefined. The [calendar note](temporal-buddhist-calendar.md)
  lists the remaining calendar and ZonedDateTime operations.
- The embedded assertion, Test262Error and property-helper sections retain the
  pinned Test262 semantics, including error object identity, observable property
  checks and restoration. Weakened assertion omission and property-helper
  compaction are removed.
- Reverse RegExp matching admits the start/end anchors already implemented by
  the matcher. The following checkpoint extends assertion grammar and nesting;
  reverse backreferences remain a separate gap.
- Private update dependency scanning records lexical `this` and computed target
  captures. With statements normalize empty body completions to undefined.
  Parser diagnostic spans use typed lexer positions, including all ECMAScript
  line terminators, instead of parsing formatted error messages.

## Lookaround, binding and field-replacement checkpoint

- RegExp lookahead compiles its full disjunction through the shared matcher,
  including captures, alternatives, nested assertions and scoped flags. Private
  assertion backtracking restores the cursor and parent direction, retains
  successful positive captures and restores negative captures. Legacy quantified
  assertions preserve zero-progress semantics. Reverse scalar/property atoms use
  existing matching operations, and ASCII classes reject non-ASCII bitmap aliases.
  See [the assertion contract](contracts/regexp-lookbehind-polarity.md).
- With entry performs ToObject before executing the body. A typed HasBinding
  operation outlines the property/unscopables query into a shared runtime
  function, preserving lookup order, realm and abrupt completions. Exact emitted
  size and native admission of the five oversized cases still require measurement.
  See [the binding contract](contracts/with-has-binding.md).
- Namespace identity includes eager/deferred phase. Transparent Proxy definition,
  failed namespace Set handling and key-array allocation retain the namespace
  internal-method and builtin-realm contracts. Full module instantiation and
  deferred activation remain open.
- ZonedDateTime.with uses ordered partial-field reads, calendar field merging,
  strict offset-string conversion and shared exact epoch conversion.
  ZonedDateTime.toPlainDate uses the same local components as toPlainDateTime.
  Named-zone/DST and other-calendar support remain open. See
  [the field-replacement note](temporal-zoned-field-replacement.md).

## Verification in progress

The fifth source checkpoint is
`0175307c05523f77e3ad7927197f3532d093d6a6`. Its workspace all-targets release
check and immutable compiler/test build pass. Its compiler SHA-256 is
`908f4c82189add2ff3cc874aff4c9f4d6ea75a68a757a450cb4775820dcd00b9`.
Focused verification passes all 13 function-coercion tests, six for-of
completion tests, eleven TypedArray.fill tests, and the word-boundary and
whitespace suites. Two backreference regressions exposed a capture pre-scan bug
and an incorrect unanchored-match expectation; both are corrected in the next
source batch. Candidate replay remains in progress. This checkpoint adds word boundaries, shared whitespace cursor advancement,
case-insensitive forward/reverse backreferences, bounded Number-only bitwise
emission and dedicated TypedArray.fill semantics. See the
[backreference contract](../../crates/lila-aot-wasm/docs/regexp-backreference-folding.md) and
[fill contract](contracts/typed-array-fill-buffer-witness.md).

The fourth frozen checkpoint is
`3e5d3cd4bebdccb8109a4715a5a499b632b2177e`, with compiler SHA-256
`166bd3c42b2196f8ba6f3da37ec8dcc748f47249ed87ac774d9d19d3272662b0`.
Its workspace release check and immutable CLI/test build completed. The paired
five-case With replay now passes all five executions, repairing five crashes on
merged main with zero timeouts. The separate admitted-source cohort passes
11/11: two function-coercion repairs and nine retained main passes, also without
timeouts. These are scoped results, not a new full-suite status.

Its eight case-folding IR tests, seven case-folding native tests and six reverse
backreference native tests pass. All twelve With binding, nine Zoned field
replacement and seventeen namespace native tests pass, including the previously
failing realm, diagnostic-pool and deferred-completion paths.
The function-coercion suite passes seven and
fails three: array-element abrupt identity, Number conversion of an object that
produces BigInt, and two paths in the foreign-realm fixture. Focused diagnostics
confirm those causes; checkpoint five repairs the production paths and passes all 13 native controls. Four of five for-of completion tests pass; the fifth used
an incorrect conditional continue expectation. IfStatement applies UpdateEmpty with undefined, whereas
a bare continue retains the preceding expression value. The fifth-checkpoint
fixture tests both cases. Every frozen failed verdict is retained.

The third lookaround replay completes with 144/162 Success and zero timeouts.
Its complete 162 audited main comparisons contain 42 repairs, 102 retained
passes and 18 remaining failures. The fourth checkpoint passes 150/162:
48 main failures repaired, 102 passes retained and 12 failures remaining,
with no regressions or timeouts.
The third full IR suite completes with 1,124 passed and four failed. Two stale
loop expectations are corrected in checkpoint four; two module tests require
updated deferred-namespace identity and explicit graph reachability. Checkpoint
five corrects both fixtures and retains the source linker's explicit unsupported
collision boundary for distinct namespace cells; valid module scopes are not
classified as syntax or link errors.
The third checkpoint's full backend suite passes all 439 tests.

Each fifth-checkpoint family includes native tests and a complete pinned source
cohort where available. The reviewed fill path includes immutable backing-buffer
rejection before coercion. Candidate replay remains pending. The following
staged work covers Float16Array, legacy pooled-class UTF-16 membership and
Intl.Locale construction/getters. The integrated provider changes preserve valid long tags and five-to-eight-letter
language subtags. The shared TypedArray constructor now checks Number/BigInt
content domains even for empty sources. Same-kind constructor/set copies preserve
raw NaN bits, set checks capacity before content type, and constructor-owned
buffers use the executing Realm. The set/copy algorithm has a dedicated family
module, preserving the standard dispatcher size boundary. These changes await
compilation and runtime validation. Created-Realm Intl publication and keyword
value aliases remain open and are being implemented separately.

The fourth full IR suite passes 1,125 tests and fails three stale shape
expectations: two module expectations corrected in checkpoint five and one
for-of property-write completion expectation corrected in the next batch. Its
full Test262 harness suite passes 361 tests and records one agent timeout; an
isolated recheck is still required. Frozen failures remain in the evidence.

The fourth checkpoint replaces Function-specific string/number conversion
bypasses with the existing ToPrimitive operation, preserving observable hooks,
conversion order, abrupt values and the executing realm. Reverse RegExp
backreferences use the shared matcher, and character-set compilation separates
legacy uppercase from Unicode simple folding. See the
[case-folding note](regexp-case-folding.md) for data provenance and remaining gaps.
Dynamic property writes reuse the existing shared Set operation instead of
copying array dispatch into every possible With branch. Non-callable Proxy get
traps create errors in the current execution realm. Zoned field replacement
includes every diagnostic required by its shared PlainDate field reader.
For-of assignment heads preserve loop completion without discarding assignment
errors or iterator closing. Deferred evaluation retains the original abrupt
completion for later namespace operations, while preserving the module body's
declaration scope; see [the lifecycle boundary](deferred-module-completion.md).
Full module allocation, indirect import cells and asynchronous dependency
scheduling remain unfinished.

The third checkpoint's focused native tests pass all nine lookaround, seven
Buddhist calendar, nine for-in, eight private-update and twelve With-completion
tests. Namespace tests pass twelve of thirteen; a frozen diagnostic confirms
the remaining test needs the Test262 host policy to create its foreign realm.
The new With binding suite passes nine of ten, exposing the Proxy get error
realm defect, and Zoned field replacement passes six of nine, with all three
failures caused by missing pooled diagnostics. These causes are repaired in
source and pass the fourth-checkpoint native suites above. Its full frontend
suite passes 164 tests; its Test262 unit suite reports 361 passed and one stale dynamic-source
fixture failure. The fixture now uses runtime-generated source, matching the
existing unsupported dynamic-source tests.

The first batch is frozen at `dc5180db9cd2f9fe179f219d0ce2d3fd8d2f6136`.
Its paired replay passes all 50 initial executions: 44 Bug-to-Success repairs
and six retained Success controls, with zero timeouts. Compiler binary SHA-256:
`37b24ea9a2bf2ecd0a9ff25708a308752173ee75a729e57e4d1d05cee495e5fe`.
The workspace all-targets release check passes. All six BigInt and six naming
native tests pass. The separate 74-case replay completes with 69 Success and
five Bug, repairing 68 historical crashes and one missing implementation on
current main. Those five newly unblocked With tests expose oversized emitted
functions; they remain failures until the compiler-size cause is repaired.
Across both paired cohorts that is 119/124 Success, 113 repaired current-main
failures, six retained passing controls and zero timeouts. The full IR suite
passes all 1,126 tests. The backend suite reports 431 passed and four failed;
three failures share the naming-string seed placement error and the fourth is
the assignment planner test measuring an unrelated object initializer. Both
corrections pass all four focused tests on the second checkpoint.

Verification found a private arrow-capture bug, a NUL diagnostic-span bug, a
tagged-template fixture escape error and stale structural test markers. The
second batch includes those corrections. After the span correction, all 164
frontend unit tests and four NUL integration tests pass. The second checkpoint,
`f1fe521f48c6fac4bc5e8004dd9c9b6b269eee29`, passes the workspace all-targets
release check and all native for-in (9), private update (8), With completion (12),
NUL (3), naming (6), BigInt comparison (6), and pinned harness (4) tests.
Its namespace, calendar and RegExp tests expose further defects or fixture
errors addressed in the following checkpoint. Its Test262 unit suite reports
351 passed and 10 failed; the failure evidence is retained while test setup and
capability expectations are corrected and rechecked. A 769-execution main
reference cohort covers the second batch's affected families, including
historical passing controls; native replay and the remaining broad tests are
still running.

Evidence is under `target/failure-review/completed-baseline-20260914`.
`batch1-candidate-initial-main-audit.json` records the verified 50-case result;
`batch1-test-verdicts.json` retains every first-batch test failure and completed
stage. The original baseline, frozen compiler inputs and failed launch logs
are retained. The module-boundary guard is reconciled with the current private
owners, including absence checks for removed Test262-specific for-in recognizers;
it passes on the fifth-checkpoint source. Two stale CI Realm-routing assertions
are staged for the next batch. No generated full-suite status numbers have been
changed.

## Seventh source batch and sixth-checkpoint findings

Checkpoint six is `7a761070589ba794aa1fa919a84ed2c1cda5dc5f` with compiler
SHA-256 `6517ab5d2ee6d4f0a8c57413105c938e998abd3f81d425e311f6d2d25387b84b`
and 2,981 frozen inputs. Its workspace all-targets release check passes.
All 32 focused test targets completed. Passing native targets include byte
copies (6), fill (11), backreference folding (8), and numeric behavior (7).
The Intl provider passes 17 unit tests. The fifth-checkpoint Test262 library
suite passes all 362 tests, including the agent test that previously timed out.
The fifth-checkpoint pinned fill cohort separately passes all 102 executions
with zero timeouts. The completed paired audit, refreshed on 2026-09-18,
records 30 Bug-to-Success repairs, eight successful timeout rechecks and 64
retained main passes.

Sixth-checkpoint failures remain recorded in `batch6-test-verdicts.json`.
Float16Array passes 10 tests and fails one foreign constructor error test;
follow-up probes reproduce the same entry-Realm TypeError on all twelve
constructors' call and iterator validation paths. The shared paths now select
the executing function Realm. Intl Locale constructor and option tests expose
a missing host-import dependency, and a foreign getter test requires the full
created-Realm Intl namespace. Legacy pooled-class tests expose a parser that
validates raw astral range endpoints as code points in non-Unicode mode;
the IR test also assumes pooled storage for an optimized singleton class.
Structural census failures cover the new shared byte-copy owner and updated
source boundaries. These are failures and follow-up work, not passing claims.

All 24 numeric stress executions still fail at checkpoint six because their
Wasm functions are too large. The next emission change proves only immutable
Number IR conditions and omits unreachable branches after complete lowering
and planning. Its controls preserve early errors, hoists, statement completion,
labels, constructors, calls, mutable properties, and BigInt errors. The 24-case
replay must pass before this family can be called repaired.

The seventh source batch also restores pinned `isConstructor.js` and native
function matching without semantic harness replacements; merges both branches'
flow facts; snapshots compound-assignment left-value types before lowering the
right side; fixes Set operation copy timing; supplies all 65 keyword-value
aliases from pinned CLDR 47; and publishes represented Intl intrinsics in
created Realms. Cached format functions use the first getter's Realm and retain
the formatter in the canonical closure capture slot. Source tests, native
targets and generated alias checks accompany these changes. The shortcut
inventory now contains 108 exact observations after removing the two callable
harness replacements. No generated full-suite counts have been changed.

Earlier candidate replay pipelines were retired after complete audited cohorts
so subsequent comparisons can use the current compiler. Their completed and
partial artifacts remain intact; pending executions have no assigned result.
Current-main reference replay continues independently. Module instantiation
and the wider failure list remain open, and the PR stays draft until the
requested fixes and verification are complete.

## Verification resumed on 2026-09-18

The seventh checkpoint is `4f67925eafdc81e7ae8e6c3c0a3c764ef636091a`.
Its workspace all-targets release check and immutable build passed, yielding
98 test executables and compiler SHA-256
`66737bc66dfecd3dd77d8831081fef30869c1b050e1e7dd1795c087249193628`.
The source manifest contains 3,023 inputs with SHA-256
`b556dc967b1775f7e2f1a345a5105525bf252c97b1e909d7ca835306191fa1d1`.

All 24 numeric shift stress executions now pass, repairing all 24 corresponding
main failures with zero timeouts. Of the 12 RegExp cases still failing at
checkpoint four, eight now pass: reverse backreferences, sticky matching and
word boundaries in both modes. The four runtime-built nested-pattern cases
remain failures. These results have complete main overlap; the broader
3,847-execution candidate selection remains in progress.

The source census completed before the eighth batch was integrated. It found
one obsolete assertion expecting two nullability projections after one had
been removed. The follow-up checks the single owning function directly.
Native and broad tests continue against the frozen seventh compiler.

Five additional resumable probes produce identical outcomes on checkpoints
six and seven. Ordinary async `if/else` incorrectly skips an awaited else
branch; four generator/async-generator shapes explicitly report unsupported
suspension or label plans. These are existing open implementation gaps, not
evidence of complete resumable support.

The eighth batch reuses the existing exact binary64 decoder and BigInt limb
packer for NumberToBigInt, eliminating signed-i64 overflow on valid integral
Numbers. Its moved Number-validation errors select the executing builtin Realm,
and created BigInt functions retain the function identity needed at that call
boundary. Frozen sixth-compiler probes establish both defects. Native tests
and the two pinned Number-conversion-rounding modes remain required.

The fresh main-reference replay stopped after 6,916 of its 14,352 remaining
executions and resumed from those records. The completed 530-case fill/numeric
main cohort contains 468 Success, 54 Bug and eight timed-out Crash outcomes.
No unexecuted case has been assigned an outcome, and these cohort results do
not update the generated full-suite status.

The eighth compiler is frozen from the source later committed as `3578335b5`:
compiler SHA-256
`40c2600ddbba1383b9431538e28ebb0def64373d2d534bb14b6dfeccf0f8a17c`,
3,026-entry source manifest SHA-256
`de656840ad1e70d49ba6697ce5e97dfab67d659c559b3a9773db21b586d31897`,
and 103 test executables. Its workspace release all-targets check passes,
as do all 30 source-census targets, all four Intl import emission tests,
all 21 provider tests and all five NumberToBigInt native tests. The latter
cover integral binary64 exponents, inline/heap boundaries, invalid values,
coercion hooks and the defining Realm's RangeError prototype. Its verification
was subsequently retired after 59 completed stages in favor of checkpoint nine.
The partial 910-execution replay is retained without assigning outcomes to
unfinished executions.

The catch repair scans parameter expressions before entering body scope and
keeps the two lexical environments distinct during lowering. Three initial IR
assertions used incorrect assumptions about qualified class-method names,
materialization of uncaptured bindings, and exact-context function clones.
Their corrected fixtures preserve explicit ownership and capture checks;
execution of those corrections belongs to the ninth checkpoint.

Seventh-checkpoint verification was retired on 2026-09-18 in favor of the
eighth compiler. Its 55 completed stages and partial Intl replay remain
recorded in `batch7-retirement.json`; unfinished stages are not passing results.
Both canonical isConstructor modes and all eight Set operation regression
executions pass. Both nativeFunctionMatcher modes time out after 360 seconds.
Diagnostic programs containing the canonical validator and all grammar cases
execute in milliseconds, with tens of seconds spent lowering. Intrinsic
signature maps contained repeated copies of the global shape before observing
a receiver. The ninth checkpoint removes those unused builtin seeds while
retaining source-function receiver and lexical-capture metadata.

An additional equality regression is present on both sixth and seventh
compilers: `(1n >>> 0n) === 0n` skips the throwing operand because static result
kinds differ. The ninth checkpoint evaluates operands before mismatched-tag
comparison for strict equality, SameValue and SameValueZero. Array and
Arguments conversions that ignore custom coercion hooks, async branch/captured
environment restoration, and module activation/prelude separation remain
separate implementation work. No full-suite conformance claim is made.

The ninth compiler, committed as `ec3f9cee1`, has SHA-256
`39d387d70b4904b47163f2c0f8079356b5feb1052803bee9505e34aa9f947e51`.
Its 3,617-entry source manifest, now including contract and task documents, has
SHA-256 `4dec948ace727a72d240546f2a0a283ae8929152c7e9b8ccb27fffee2db897c7`.
Workspace release all-targets checking, CLI/test builds and all 32 source
checks pass. Focused verification passes catch ownership IR (4), equality
effects (5), intrinsic receiver behavior (4), canonical callable harnesses (5)
and constant numeric conditions (6). The complete callable/Set replay passes
12/12 with zero timeouts: ten seventh-checkpoint passes retained and two former
matcher timeouts cleared. Four callable cases overlap the completed current-main
reference and repair four Bugs; the other eight have no main-pairing claim in
this audit. The unchanged full validator diagnostic improves from 33.63 seconds
to 2.72 seconds on the recorded bounded runs; this is diagnostic timing, not a
general performance guarantee.

The completed ninth catch/BigInt replay passes 388/390 with zero timeouts:
29 main NotImplemented outcomes and six main crashes become Success, and all
353 main successes are retained. Both remaining crashes are the two modes of
`language/statements/try/S12.14_A9_T3.js`. Exact paired evidence is in
`batch9-candidate-catch-and-bigint-main-audit.json`.

The ninth native catch suite retains eight passes and two suspension failures;
saved compound assignments retain three passes and the custom Array conversion
failure. Both causes are addressed in the next source batch. The wider catch
replay also reproduces a compiler panic when stale do-while branch targets reach
a later finalizer; an ordinary labelled-switch probe separately reports an
unknown label. Their target-stack lifecycle repairs keep the backend's invalid
branch-label assertion intact.

The tenth source batch integrates live Array/Arguments conversion hooks, typed
plain-async conditional continuations, resumed async/generator lexical records,
and private synchronous module activation records. Module harness Scripts are
parsed and lowered separately in the same Realm, with their source included in
the program cache key. A private lexical owner keeps retained Module drivers'
declarations out of the global Script environment. Retained cycle/TLA/source
drivers still have inter-module free-name capture and global import-alias gaps;
these are documented unresolved cases, without ignored or falsely passing
tests.

The tenth frozen compiler has SHA-256
`635d709cd09159658ea86885042a10ac5cd762a70c0874bdfdf1fe0904e8e63e`;
its 3,684-input source manifest has SHA-256
`9f5c09972bbe12c736f45ddc8ce5f02bafd30a49eda920d10bcaca00c6a78f68`.
Release workspace all-targets checking, CLI/test builds and the separate
spec-exec-oracle feature check pass. Its 38 source census stages completed
before further edits: 37 pass, and the conversion-Realm census retains two
stale assertions about the receiver domain and removed Function conversion
bridge. Those assertions are updated in checkpoint eleven.

Checkpoint ten passes native control flow (14), Array/Arguments coercion (6),
saved compound values (4), async conditionals (15), generator catch environments
(6), all catch-pattern owners (10), async for-of (4), async loop bindings (6),
and generator loop/call-suspension controls. The original branch and captured
cell diagnostic programs also produce their expected output. Module native
tests report 12/14; both failures infer Number from a linker import placeholder
inside nested closures. The next source batch makes static initializer inference
honor the canonical indirect-import metadata and adds live-capture controls.
An IR fixture also counts a legitimate nested driver as an outer owner, and
the heap inventory passes zero instead of the module record size to its bounds
check; both fixture corrections remain pending execution in checkpoint eleven.

The complete tenth module replay reports 134 Success, 13 Bug and 6
NotImplemented out of 153, with zero crashes or timeouts. Its exact current-main
comparison repairs 39 Bugs and two NotImplemented outcomes while retaining all
93 main successes. Two former Bugs now stop at an explicit NotImplemented
boundary and remain unresolved. The 12 callable/Set controls also all pass.
`batch10-candidate-modules-main-audit.json` records all paired source hashes.

Ninth-checkpoint verification has finished: the frontend library passes 164/164,
IR passes 1,131/1,131, AOT reports 450/452 and Test262 reports 358/362. The two
AOT failures are stale prototype-slot/count assertions. The count is corrected
in checkpoint ten; the line-wrapped prototype-arm assertion is corrected in
checkpoint twelve. All four Test262 failures trace to the missing final section newline after
the otherwise byte-identical pinned property helper; its canonical section
boundary is restored in checkpoint eleven. The complete ninth Intl replay is
300/366 and BigInt retention is 154/154, both without timeouts. The Intl failures
include missing likely-subtag methods, runtime-generated RegExp patterns and
DateTimeFormat capabilities; passing direct tag-validation probes do not close
the canonical generated-pattern cases.

Checkpoint eleven additionally integrates `Intl.Locale.prototype.maximize` and
`minimize` through the pinned locale provider. Independent source review found
and corrected missing catalog IDs and unknown `Zzzz`/`ZZ` preprocessing before
integration. All seven native likely-subtag tests pass. Its complete pinned
cohort passes 24/24 with zero timeouts: six executions overlap the ninth Intl
replay and repair six Bugs, while eighteen have no paired comparison. Static versions
of the generated RegExp reproducers pass on frozen checkpoint nine, confirming
that runtime pattern compilation remains a separate required implementation.

Checkpoint ten has completed all 137 verification stages: 126 pass and eleven
retain failed tests. The failures cover module lowering and scope, implicit
async suspension, and obsolete source-shape assertions. Their original verdicts
are preserved in `batch10-verification-complete.json`. Its complete catch/BigInt
cohort now passes 390/390, repairing all 37 current-main failures (29
NotImplemented and eight crashes) while retaining 353 main successes. Intl
remains 300/366; BigInt retention passes 154/154. All three completed replays
have zero timeouts.

Checkpoint eleven's compiler SHA-256 is
`aa07baab681ae8c92f754b2603fecdc1a7f0b52f1d06ec71058df84efc792971`;
its 3,689-input source manifest SHA-256 is
`3dfb7df846a968ca4c688017e8e8050792c4c0e3bddee919b6d9d7da7a613af7`.
Release all-targets checking, CLI/test builds and the separate oracle feature
check pass. Its complete module replay reports 135 Success, 14 Bug and four
NotImplemented across 153 executions, without crashes or timeouts: 42 repaired
current-main failures and 93 retained main successes. All 143 verification
stages finished: 134 pass and nine retain failed tests. Frontend passes 164/164,
IR reports 1,133/1,134, AOT reports 451/452 and Test262 reports 363/364. The
remaining failures cover synchronous module control flow, implicit async
suspension and stale source-shape assertions. Their complete verdicts remain in
`batch11-verification-complete.json` as required regressions for the next build.

The twelfth source batch extends canonical module owners to ordinary and cyclic
synchronous Module graphs. A private instantiation suspension no longer turns
source `try`/`catch`/`finally` and resource scopes into generator control flow.
Every plain-async Await producer saves the active lexical chain before resuming,
including disposal and iterator awaits. Public class fields use DefineProperty
semantics for Proxy, Array, TypedArray and namespace receivers. DateTimeFormat
negotiates and renders the 77 pinned CLDR positional numbering systems through
one generated table. RegExp objects retain one immutable, validated program
descriptor rather than independently mutable code and metadata slots. These
changes are undergoing verification; the runtime RegExp compiler, broader Intl
capabilities and asynchronous module drivers remain open.

The twelfth frozen compiler has SHA-256
`6379fd328782ebc4a8df53516ed3156f25b19e0e34f3e93a4f913b667c293eb4`;
its 3,710-input source manifest has SHA-256
`a0b6f50fc63062b01656440825dc436379ec74db1419c7394d008e55a5d57c47`.
Release checking, immutable compiler/test builds and the separate oracle feature
check pass. All 52 source census stages finished before further changes: 45 pass
and seven expose stale ownership/shape assertions. Those assertions remain
recorded while their exact consumers are reviewed. The new module resource IR
fixture reaches an older blanket admission guard; no module resource pass is
claimed from its immediate-lifetime plan alone.

Native async property assignment passes 10/10, module scope and cycles 11/11,
public class fields 5/5, and property-definition Realm behavior 6/6. The Reflect
descriptor control passes 1/1. The separate frozen-binary diagnostic run passes
18/19, including the formerly skipped repeated namespace getter, both implicit
await paths, foreign descriptor prototypes and all eight exotic error-Realm
checks. Its remaining resource-loop probe is rejected by the same module guard.
The descriptor corruption test passes, while the combined lifetime test reaches
an existing matchAll capability gap when cloning a Unicode program with changed
flags.

Checkpoint twelve completed all 167 verification stages on 2026-09-18: 155 pass
and twelve retain failed tests. The complete verdicts and immutable executable
hashes are in `batch12-verification-complete.json`. Frontend passes 164/164,
IR reports 1,143/1,144, AOT passes 443/443 and Test262 passes 364/364. The failed
stages cover the module resource admission guard, RegExp descriptor/protocol
behavior, seven source-census fixtures and the module-root-this IR fixture.

The completed module replay passes 137/153 with twelve Bug and four
NotImplemented outcomes: 44 repaired current-main failures and 93 retained
successes. The Intl replay passes 334/394 with sixty Bug outcomes; its 366-case
overlap with checkpoint ten repairs eight failures and retains all 300 previous
successes. The separate lookaround replay passes 158/162, repairing 56 main
failures and retaining 102 main successes. A complete 24-execution Date locale
reference passes twenty and records four Bugs. All four completed replays have
zero crashes and zero timeouts. The separate RegExp folding replay passes
225/242 with six Bugs and eleven timeout Crashes. Compared with main, it repairs
sixteen Bugs, clears one previous timeout and retains 208 successes; the other
eleven timeouts remain unresolved. The completed catch/BigInt replay passes
390/390, repairing 37 current-main failures and retaining 353 successes, with
zero crashes or timeouts.
These are bounded cohort results, separate from a refreshed full Test262 status.

The thirteenth source batch admits canonical synchronous module resource scopes,
orders RegExp `lastIndex` coercion before program selection, and repairs stale
source fixtures without dropping their ownership and ordering checks. It adds
an emitted compiler for computed legacy RegExp patterns and routes Date locale
methods through shared DateTimeFormat initialization. Runtime Unicode RegExp
grammar, broader Intl locale/calendar support and asynchronous module drivers
remain open; the following counts cover bounded checks, not a new full sweep.

The thirteenth frozen compiler has SHA-256
`9758cb830f3bd369cf9dca0e61f44f6bd4271dc5cab2b27a68d873fc58f86770`;
its 3,734-input source manifest has SHA-256
`549d156feaaf7f2dc5ed9c732b3846a8ce27d74ab9bcd9b9bdc23cb413f791b7`.
Release checking, immutable compiler/test builds and the oracle-feature check
pass. The source census completes 54 stages with 52 passes and two stale
source-boundary failures. Date locale passes 12/12 native tests and 24/24 pinned
executions, with four repairs and twenty retained checkpoint-twelve successes.
RegExp recompilation passes 7/7 and descriptor lifetime passes 3/3. The runtime
compiler target passes 5/6: its unchanged large grammar fixture exceeds the
Wasmtime function-size limit. Module instantiation passes 21/23 native tests;
the two failures retain backend guards on resource loop heads. All 172 selected
verification stages finished: 168 pass and four retain failed tests. A later
library source guard read the next batch's live source; that limitation is
recorded in `batch13-live-source-reader-limitation.json`, so the full result is
not claimed as a single-source verification. The completed module replay remains
137/153. Intl passes 374/394, repairing forty checkpoint-twelve failures and
retaining 334 successes, with zero crashes/timeouts. Lookaround passes 157/162:
four old Bugs remain and one former success exceeds the Wasmtime function-size
limit. Folding passes 224/242 with six Bugs and twelve timeout Crashes; all 208
main successes remain successful. The completed catch/BigInt replay passes
390/390: 37 main failures repaired and all 353 main successes retained, with
zero crashes/timeouts.

The fourteenth batch fixes those resource guards using the closed source
execution kind, and publishes host globals needed only by independently
compiled Script, Function and module-prelude sources. Prepared declarations
remain owned by runtime instantiation. Bare entry vars reuse host properties,
entry functions retain their override, and later replacement/deletion remains
observable. New regressions cover direct and queued callbacks, separate Realm
Scripts, Function bodies and module completion jobs.

The same batch adds canonical synchronous import continuations: dynamic-only
targets wait for their import job, load/link failures reject the owning promise,
and repeated evaluations retain the original completion. Computed legacy RegExp
patterns gain nested scoped `i`/`m`/`s` flags, including changed-global-flag clones.
Array present-index bookkeeping moves into one emitted helper; the original
large RegExp grammar fixture is retained as the end-to-end size regression.
The array change preserves the bookkeeping algorithm and has no runtime-speed
claim. Separate phase probes still investigate the remaining RegExp
timeout failures.


Checkpoint fourteen revision one has compiler SHA-256
`593922739990098fb718914456b8662e0b1e66b51a7803cadafe3f182beb821c`
and 3,745-input manifest SHA-256
`4daf47d5fe29a8f56589fcbc9fb73462c500f525c0102f45842427f8f90d82bf`.
The first attempt stopped at a Rust formatting-macro error before producing a
compiler; its failure evidence remains separate. Revision one passes release
checking, immutable builds and the oracle-feature check. Its 61 source/library
stages finish before source unfreezes: 60 pass, with one stale helper-count
assertion. Frontend passes 164/164, IR 1,148/1,148, AOT 443/444, and Test262
364/364. Native module instantiation passes 24/24 and import-job ordering passes
9/9. The completed pinned module audit passes 147/153, repairing 54 main
failures and retaining all 93 main successes, with zero crashes/timeouts. The
remaining two Bugs and four unsupported cases concern asynchronous deferred
module evaluation.

The unchanged runtime-RegExp grammar fixture now fits Wasmtime's function-size
limit: its main body falls from 4,556,950 to 3,183,163 bytes with 283 locals in
both artifacts. It then exposes dirty compiler workspace reused by capture
arrays. Three standalone computed-pattern probes fail at that same boundary.
The next batch clears released storage on successful publication and every
typed failure while preserving immutable descriptors. One new native prepared
Realm-Script test also lacked its `$262` host bridge; the correctly bridged
source passes on the immutable fourteen compiler, and the fixture is corrected.
These failed checks remain recorded rather than counted as passes.

The full fourteen-revision-one verification finishes at 178/181 passing test
groups. The three failed groups are the stale AOT helper count, runtime RegExp
workspace reuse, and the missing host bridge in the prepared Realm fixture.
The fifteenth batch repairs those paths, implements both legacy accessor
definers, separates Test262 Script rejection reporting from Script completion,
and makes allocated dense array capacity authoritative for indexed storage and
iteration. Sparse descriptors transfer intact when capacity grows. Its public
Promise policy rejects the retained async Module driver when ignoring rejection
reports would discard module evaluation failure.

Checkpoint fifteen has compiler SHA-256
`fa38c03f424111162e4ebd208f7dff09e95f7b6d79b9e0c898c500785b6756c3`
and 3,753-input manifest SHA-256
`560cec320a4dd5e4785417473b390cd3fe20d71d8a7ffadb1cc46cb675dc2536`.
Release all-targets checking, immutable CLI/test builds, and the oracle-feature
check pass. All 64 source/library groups pass before source unfreezes, including
Frontend 164/164, IR 1,149/1,149, AOT 444/444, and Test262 366/366. The completed
paired Promise audit passes 28/28: 26 main Bugs repaired and two async main
successes retained. Legacy accessor definitions pass 21/21, repairing eighteen
main Bugs and retaining three successes. Folding passes 242/242, repairing 22
main Bugs, passing all twelve main timeout rechecks and retaining 208 successes.
Lookaround passes 161/162: all sixty main Bugs are repaired, 101 main successes
remain, and the strict named-lookbehind fixture still regresses at Wasmtime's
function-size limit. Modules retain 147/153 with the same six async-defer
failures. These complete cohorts have zero crashes/timeouts. Native verification
finishes at 188/190 passing groups: the two failed groups expose sparse
descriptor transfer and a Promise test's formatted-note assertion. Revision one
below addresses both. Intl retains 374/394 and Date locale retains 24/24, with
zero crashes/timeouts. The completed catch/BigInt replay passes 390/390,
repairing 37 main failures and retaining all 353 main successes.

Checkpoint fifteen revision one has compiler SHA-256
`35b1e763240b4f17c2afb306620d0e91a8edb0a6121bbe5ceff781ced4702fe4`
and source-manifest SHA-256
`92c06b7fc9c91e5767b5427eaffc5781ddd5eb44587837d5a09d04fca86563fb`.
Its release checks and builds pass. All 77 verification groups complete: 74 pass
and three fail. The 64 source/library groups pass against verified frozen
sources; frontend is 164/164, IR 1,149/1,149, AOT 445/445 and Test262 366/366.
Promise policy passes 8/8, including the public async-Module guard; JSON passes
27/27 and Array 52/52. The original sparse descriptor-transfer failure passes.
Array storage now reports 10/11, indexed deletion 7/8 and Arguments concat 3/4:
their remaining failures expose stale carriers after deletion and a missing
descriptor when reusing a sparse tombstone. The following source batch repairs
those causes and retains the failed receipts for comparison.

Checkpoint sixteen revision three has compiler SHA-256
`fc93fae26be677472c3cb1aedfdbb51aee2fe3aa58991b826d87598bc95f02fe`
and 3,841-input source-manifest SHA-256
`633722fb07590caaf1930100b949fc5d16ecc1870f4464d6a57cd80ee90a0df2`.
Release all-target checking, immutable builds and the oracle-feature check pass.
The three earlier compilation attempts retain separate failure evidence and
produced no frozen compiler. All 101 verification groups complete: 97 pass.
The 69 source/library groups finish against verified frozen inputs, with three
failures from stale contract/catalog assertions; the remaining failing provider
group exposes Iceland's incorrect primary-zone mapping. The next batch repairs
the mapping from IANA's explicit geographic alias authority and updates those
assertions without changing their intended invariants.

Native named-zone tests pass 9/9, String match 8/8 and emitted-size controls 2/2.
Array storage passes 12/12, deletion 8/8 and Arguments concat 4/4. Both independent
deletion/reinsertion reproducers pass. Frontend passes 164/164, AOT 445/445 and
Test262 366/366; IR is 1,148/1,149 because of the catalog-order assertion. The
completed lookaround audit passes 162/162, repairing all 60 main Bugs and retaining
102 main successes. Folding passes 242/242: 22 main Bugs repaired, 12 main timeout
rechecks pass and 208 main successes remain. Neither cohort has crashes/timeouts.
The unchanged strict named-lookbehind fixture also passes in an isolated native
run; its main Wasm function falls from 4,144,083 to 1,263,006 bytes with 104 locals
in both versions. The completed Intl audit passes 390/394, repairing sixteen
checkpoint-fifteen failures and retaining all 374 successes. Four Bugs remain
in Chinese related-year and Arabic Temporal-formatting cases. Date locale
passes 24/24. Neither replay has timeouts.

Checkpoint seventeen revision one has compiler SHA-256
`a2a061767782b86f1ff206b9851d59c071d08c6be5e51ca80a709c086e75ad33`
and 3,895-input source-manifest SHA-256
`4bd4df936d5d0b7c6f081d2604307cb96a7e93bdf171590c858ea79cda9e4efe`.
The selected Instant audit completes 229 executions: 227 Success and two Bug,
with no timeouts. It repairs 225 main failures and retains both selected main
successes. Until/since still reject a recognized date largestUnit before reading
later options; the next source batch restores their required observation order.
Wide Duration native tests pass 8/8, Instant methods 12/12 and Intl provider
tests 56/56. Frontend passes 164/164 and IR 1,149/1,149. These paired cohort and
native results are separate from the full historical sweep and the still-running
immutable-main replay.

All 148 checkpoint-seventeen verification groups complete: 142 pass. The 101
source/library groups ran before releasing the verified source freeze. Four
guards still name obsolete code boundaries, and the AOT dependency test finds
that Reflect's ordinary definition body was not retained for standalone JSON.
The JSON reviver target passes 6/8: inherited descriptor reads and locked Array
length expose shared property-definition defects. The next source batch fixes
those causes, keeps the failures as evidence and adds direct ordinary API
controls. All 142 frozen test executables and all 148 logs are hashed in the
completion receipt. The 27 retained JSON, 52 Array and other selected native
groups pass; this does not erase the new focused failures.

Checkpoint eighteen revision two has compiler SHA-256
`52a8b39aada65b67fb6b39092003db4120406efc2817f34a5b38f653e71078a4`
and 3,913-input source-manifest SHA-256
`ae3d623eb58bc1a5ea5096a2e22ce3361a39f4aa9404e98b7d0e0efb9bffc5c5`.
Its release all-target check, immutable build and oracle-feature check pass.
On 2026-09-19, all 159 verification groups complete successfully, containing
3,033 tests. All 105 source/library groups run before releasing the verified
source freeze. Frontend passes 164/164, IR 1,149/1,149, Wasm backend 447/447,
Test262 harness 369/369 and Intl provider 56/56. The completion receipt hashes
all 152 frozen executables and all 159 logs.

Native module entry completion passes 12/12, JSON reviver definitions 9/9,
Reflect/Object descriptors 9/9, Instant methods 19/19, wide Duration 8/8,
Promise policy 8/8 and Array index storage 13/13. The nine unchanged saved
reproductions all pass through Wasm AOT: canonical descriptors and JSON sibling
callbacks, locked Array length, TypedArray coercion throws, nested Proxy
definitions and Instant conversion boundaries. No new paired conformance result
is inferred from these native tests.

The initial eighteenth compile failed on a missing IR import. Revision one
then exposed a missing interned error string in fifty verification groups and
one stale completion-shape guard. Both failures remain recorded; revision two
registers the string and updates the guard while preserving its completion
routing assertion. Revision one's 108/159 result is superseded only for the
verified revised source, not rewritten in the evidence.

The next source change preserves the Arguments indexed own-property bit when
all descriptor attributes and the ParameterMap flags are zero. Its batch keeps
all source/library guards and the affected Arguments, Array, property-definition
and JSON native regressions. The planned paired replay carries the same 641
execution identities: property definitions (231), Instant (229), modules (153)
and Promise policy (28). This replaces the unstarted eighteenth-checkpoint
replay; no observed outcome is discarded. Generated full-suite status remains
unchanged, and the immutable-main replay is still incomplete.

Checkpoint nineteen has compiler SHA-256
`21273808b5be7f963225149ab64dc655a1a3d3676571fd30f5d6b0f083703f71`
and 3,914-input source-manifest SHA-256
`af122712623fba3af6f3ea490e94cc951283702e35859fc49bec60b102cc6513`.
The release all-target check, immutable builds and oracle-feature check pass.
All 121 selected verification groups complete successfully, containing 2712
tests. The 106 source/library groups run against verified frozen source. The
five new Arguments descriptor tests, six Array/Arguments primitive-conversion
controls, fourteen Arguments iteration tests and four concat tests pass, along
with the selected Array, JSON and property-definition retention groups. Both
saved Arguments reproducers pass through the frozen Wasm-AOT CLI. The source
change from checkpoint eighteen is confined to the shared Arguments indexed
entry store, its tests and documentation; unrelated native groups retain their
separate completed eighteenth-checkpoint result.

The paired replay completed on this frozen nineteenth compiler on 2026-09-19.
Property definitions pass 231/231, repairing 47 main failures and retaining 184
main successes. Instant passes 229/229, repairing 227 failures and retaining two
successes. Promise policy passes 28/28, repairing 26 failures and retaining two
successes. Modules remain 147/153, repairing 54 failures and retaining all 93
main successes; the remaining four NotImplemented and two Bug outcomes are the
six asynchronous deferred-module cases tracked below. All four complete audits
have zero crashes and timeouts. Their 641 execution identities are disjoint;
the completion receipt records each audit digest and the unchanged frozen
compiler/source identities. This is still a bounded comparison against the
selected main references, not a refreshed full-suite conformance claim.
Refresh the same selection with the retained
`replay-batch19.sh`, or use `scripts/replay-test262-executions.py` with each
recorded execution list, immutable compiler and a fresh output directory, then
`scripts/audit-test262-replay.py` with its recorded main-reference audit. Native
refresh commands include `cargo check --release --locked --workspace --all-targets`
and `cargo test --release --locked -p lila-engine --test aot_arguments_index_descriptors
--test aot_array_arguments_primitive --test aot_arguments_iteration
--test aot_arguments_concat -- --test-threads=2`.

## Instant arithmetic and differences

The remaining-main snapshot at `2026-09-18T20:23:20Z` contains 227 observed
missing-method Bugs across Instant add (26), subtract (24), round (40), until
(68), and since (69), plus two selected passing epoch-limit controls. These
are selected executions from an incomplete immutable-main replay, not a full
suite count or a candidate result. The exact 229-execution replay and copied
source/outcome/transcript hashes live under
`target/failure-review/completed-baseline-20260914/temporal-instant-methods-followup`.

The new methods use the existing canonical Duration conversion and arithmetic
pipeline, a shared exact Instant/ZonedDateTime BigInt splitter, and the private
validated epoch allocation proof. Instant rounding uses a floor day and a
within-day nanosecond remainder; half-even includes the day contribution to
global quotient parity. Difference options retain observable getter order,
time-unit admission, signed rounding and since's rounding-mode inversion.

Instant and Duration member installation is shared between entry and created
Realms. Only these two Temporal families are added to created-Realm publication
in this change; the wider pre-existing omission of other Temporal families
and Temporal.Now there remains separate work. Result prototypes come from
immutable Realm slots, and foreign constructor fallback follows NewTarget.

The batch must be integrated together with the canonical wide Duration field
foundation. Pinned add/subtract minimum-maximum cases require valid Number
fields beyond i64, and largestUnit nano/micro differences must also retain
those values. The required native regression includes real Duration instances
and later Duration operations; there is no Instant-only conversion bypass.
No candidate pass count is claimed before root runs the new native target,
existing arithmetic controls and the exact canonical replay. See
[the Instant method contract](contracts/temporal-instant-methods.md).


## Structured async for-of body continuations

The frozen batch20r4 compiler rejects the original module lifecycle loop that
awaits each of two concurrent imports inside try/catch. The same failure is
independent of modules: seven ordinary async-function witnesses reject with
`async for-of body did not lower to a direct await sequence`. The two selected
neighboring direct-await and standalone-try controls pass; all ten recorded
executions complete without timeouts. Exact sources and receipts are retained
under `target/failure-review/completed-baseline-20260914/async-for-of-nested-continuation-review`
and `async-loop-before-results`. This is a focused implementation baseline,
not a full-suite result.

The follow-up replaces the linear body split with a private-constructor body
whose nested continuation ranges are checked against the plain async
statement dispatcher. The existing iterator emitter retains its acquisition,
step and IteratorClose ownership, then dispatches the complete structured body.
Captured iteration environments are restored from the saved parent chain;
body and catch owners restore their own descendants. Shared await scheduling,
Promise rejection transport and Realm ownership remain canonical.

Candidate verification is required before claiming the demonstrated failures
are fixed: the `async_for_of_body` library tests, the
`async_for_of_continuations` IR target, the existing
`plain_async_sync_for_of_iterator_record_structure` target and the new
`aot_async_for_of_continuations` native target. The latter retains the exact
original two-file module graph, ordinary async controls, return/throw/finally
close precedence, distinct captured scopes and foreign-Realm rejection
identity. Existing async loop, async-if, iterator protocol and module lifecycle
regressions remain required controls. No candidate execution or published
conformance count is recorded by this staged change.

## Provider and module checkpoint, 2026-09-19

The frozen `batch20r6` build completes all 153 selected native/source groups:
152 groups pass, with 3,001 passing tests and one failure. The failing
`async_module_resources_have_a_canonical_execution_owner` test exposes the
remaining rejection of a non-suspending resource loop in an async module.
All 105 source/library groups pass. Workspace all-target release checking,
the CLI/native build and the optional oracle-feature check also pass. Exact
group membership, executable hashes and transcripts are retained in
`target/failure-review/completed-baseline-20260914/batch20r6-test-verdicts.json`.
This is a completed run with a known failure, not a green checkpoint.

All 15 separate DateTimeFormat CLI probes pass through Wasm-AOT with zero
timeouts. They cover locale and calendar resolution, numeric parts, negative
subsecond and far-domain inputs, option/coercion order, foreign-Realm behavior,
range patterns and Plain Temporal time-zone independence. The separate
637-execution pinned replay completes with 621 Success, 16 Bug, zero Crash,
zero NotImplemented and zero timeouts. Modules pass 153/153, including 60
current-main repairs and 93 retained successes. Date locale passes 24/24 and
Plain Temporal time-zone controls pass 42/42. The DateTimeFormat group passes
384/394, repairing four earlier failures but regressing ten earlier successes;
additional calendar/option controls pass 18/24. The 16 formatting failures
remain owned by the provider follow-up. Exact disjoint membership and audited
compiler/input identities are in `batch20r6-product-replay-complete.json` under
the evidence directory. No published full-suite count is changed.

The next integrated batch adds the complete NumberFormat intrinsic family,
its Number/BigInt locale consumers, and structured async for-of continuations.
The pure NumberFormat provider passes 89 tests in both debug and optimized
release with overflow checks; its earlier failing range-spacing and sharing
receipts remain preserved. Product checks are pending. Its pinned replay
contains 552 direct family executions and 58 disjoint adjacent controls;
335 have current-main observations (285 Bug, 50 Success). Current-main outcomes
for the other 275 are not inferred from the historical sweep.

The same batch corrects the demonstrated `new String(Symbol())` defect. String
now owns both its call and construct results, performs conversion in the called
function's Realm, and reads `NewTarget.prototype` after conversion. The generic
constructor wrapper no longer preallocates or boxes its result. Frozen r6
probes demonstrate the previous missing Symbol throw, reversed observable
order, and incorrect foreign conversion-error Realm. Seven new native controls
retain the unchanged pinned Symbol-conversion fixture and cover boxed UTF-16
properties, bound calls, foreign fallback prototypes, subclassing, and exact
abrupt values. Candidate execution remains pending.


## Borrowed classic-for initialization in prepared direct eval

The exact sloppy execution
`language/statements/for/head-init-var-check-empty-inc-empty-completion.js`
traps in `heap_alloc` through `array_alloc` on both immutable main and frozen
`batch20r6`. This is an observed allocation trap, not a timeout. The retained
r6 diagnostics isolate the cause: five bounded witnesses read the old caller
binding after a classic-for `var` initializer; a fresh binding reads undefined
and then NaN on every update. The three adjacent standalone-declaration,
strict/indirect-eval and destructuring-head controls pass. All eight executions
complete without timeouts. Exact compiler, source and transcript hashes remain
under `target/failure-review/completed-baseline-20260914/prepared-eval-loop-progress-followup/evidence`.

`lower_var_init` previously used direct declaration storage for every simple
identifier head outside `with`. Sloppy direct eval borrows its caller variable
environment, while the loop's reads and writes already resolve through that
environment. The repair admits those borrowed heads to the existing declaration
statement path. It preserves source-order initialization, runtime Reference
selection before each initializer, empty declaration completion and the same
strict/direct/indirect ownership decisions as a standalone `var` statement.
No runtime source parser, alternate evaluator or special test-name path is added.

The required candidate gates are `borrowed_eval_loop_heads` (including three new
IR assertions), `with_var_hoisting`, `declaration_completion`, the six new cases
in `aot_declaration_completion`, the existing direct-eval owner/callee controls,
the eight retained bounded diagnostics, and the unchanged exact Test262
execution. Native fixtures cover the original finite loop, caller closures and
initializer mutations, object-environment Reference stability and setters,
initializer/publication throws, empty and body completions, and owned/custom
callee controls. Candidate execution remains pending; no full-suite count is
changed by this source repair.

## Saved draft checkpoint, 2026-09-20

This checkpoint preserves the integrated implementation in draft PR #52 while
the remaining failures are repaired. It includes the DateTimeFormat and
NumberFormat providers, async module lifecycle, structured async for-of bodies,
non-suspending resource loops, String construction, Realm-owned throwers, and
borrowed direct-eval loop initialization. It is not ready to merge and does not
claim that all historical baseline failures are fixed.

The tested compiler is `batch21`, built on 2026-09-19 from parent
`04e328657972f6c7500828bed248d71d1ffdd9c3` with uncommitted implementation changes.
Its binary SHA-256 is
`902c42ea090aacb49cdae844bd0a2747a19e1f3280355a5d9a7778ded6ac093f`;
its 4,116-input source manifest SHA-256 is
`5b5433cb23cdbe8eb5bfd7ebf5cc29fc71726e2dd4b7e48de5cab297ae154d43`.
Those inputs were verified unchanged before this documentation-only checkpoint
update. The pinned Test262 tree remains
`aa55200d1310384c5cf69ea95b2a2ecba457007b`.

Release workspace all-target checking, CLI/native compilation, the separate
`spec-exec-oracle` feature check, and Rust formatting pass. The staged whitespace
check reports only whitespace retained verbatim in three pinned upstream CLDR
source documents; those bytes remain unchanged for provenance. The planned 180-group
verification stopped after its 110 source/library groups: 107 groups pass,
with 2,735 passing tests and six failing tests. Seven separately executed
focused runtime groups completed with 94 passing tests and four failures:

| Runtime target | Passed | Failed |
| --- | ---: | ---: |
| `aot_intl_numberformat` | 15 | 1 |
| `aot_intl_datetime_provider` | 16 | 0 |
| `aot_string_constructor` | 7 | 0 |
| `aot_throw_type_error_realm` | 4 | 2 |
| `aot_async_for_of_continuations` | 11 | 0 |
| `aot_async_resource_loops` | 8 | 1 |
| `aot_declaration_completion` | 33 | 0 |

These disjoint completed checks total 2,829 passes and ten failures, with zero
ignored tests. The remaining 63 planned groups and the prepared 1,300-execution
pinned replay have not run on this compiler. Earlier checkpoint results remain
separate evidence.

Outstanding findings at this saved checkpoint:

- Six source/library failures cover an over-wide classic-for source guard,
  the NumberFormat installer-order expectation, two stale String/prototype
  ownership assertions, and two host-import assertions. Constant-only programs
  currently pull in Intl and clock support through builtin dependencies; the
  dependency expansion requires investigation and repair.
- NumberFormat's foreign constructor fails the error-Realm assertion for a null
  locale list in the primitive-option/abrupt-completion fixture. The subsequent
  investigation below corrects the initial attribution to error allocation.
- Two thrower tests fail. Isolated diagnostics show `Reflect.get` and
  `Reflect.set` on an unmapped Arguments `callee` return instead of invoking
  the accessor. A prepared foreign Function with a destructuring parameter
  also reports a parse error. Other isolated factory shapes and throw paths
  pass; these results do not prove the complete thrower family correct.
- The resource-loop eager-class/nested-function fixture fails. An additional
  bounded async captured-block probe reports a TypeError while a captured
  resource-head control passes; activation-owned disposal storage needs its
  environment address checked. These probes are separate from native counts.

The immutable compilers, source manifests, per-group membership and transcripts
remain under `target/failure-review/completed-baseline-20260914`, including
`batch21-test-verdicts.json`, `batch21-new-runtime-verdicts.json`,
`batch21-diagnostic-results` and `batch21-throw-isolation-results`.
They are local evidence, not files committed to the PR. Focused runtime results
can be refreshed with `cargo test --release --locked -j2 -p lila-engine --test
<target> -- --test-threads=2`, using the target names above and
`LILA_MODULE_MEMORY_CACHE_ENTRIES=1`. Published full-suite status is unchanged.

## Historical-failure replay and checkpoint repairs, 2026-09-22

The frozen-main comparison now covers all 14,402 historical failing execution
identities. Its two disjoint selections were audited against every saved native
snapshot and transcript, with no duplicate or omitted execution identities.
Main passes 4,699; the remaining outcomes are 8,347 Bug, 480 Crash and 876
NotImplemented. The 14,352-execution selection contains 353 recorded timeouts.
These are results for the historical failure selection; the original 87,641
successes were not rerun and published full-suite counts remain unchanged.

The reference compiler binary SHA-256 is
`04c2d07e071a21893e21795a978367f92d3f2fc753a3b878e4653f0623b292e1`.
All 2,916 source-manifest entries match main commit
`2abe452111099c84d8c0dbb8dab15db1a4de7aa1`; its source manifest SHA-256 is
`6d74ac0a9c9619d9ea1c911669f0bd1804be584dea6b12cf03807393209e7102`.
The Test262 pin is unchanged. The local audit is retained in
`target/failure-review/completed-baseline-20260914/batch22-main-replay-audit`.

The largest remaining path families on this reference are Temporal (4,552),
RegExp (666), dynamic import (533), NumberFormat (494), DurationFormat (222)
and Locale (192). These are triage groups, not shared-root-cause counts or
measurements of the current PR compiler.

Checkpoint diagnostics identified separate causes for the latest failures:

- Reflect's Arguments `length`/`callee` path must honor the source descriptor,
  retain the explicit receiver and update the dedicated receiver slots with
  their attributes and presence marker.
- Isolated Function parameter parsing incorrectly rejects a completed final
  object or array binding pattern while checking for an optional initializer.
- Activation-owned disposal storage must resolve through the current lexical
  environment depth. Synthetic class element calls need a real active function
  while preserving the class definition's captured environments.
- The NumberFormat null-locale error has the correct foreign prototype.
  Constructor throw inference omits possible body completions, allowing a later
  string throw to narrow the catch binding and miscompile its property reads.
- Runtime bootstrap pulls primitive locale methods and their Intl/clock
  dependencies into literal-only scripts. A conservative lowered-IR proof can
  omit an unobservable bootstrap while preserving the ordinary expression
  emitter and the experimental Wasm GC requirement. Expressions that still
  lower through generic coercion operations retain the runtime, including
  numeric arithmetic on literals; their value regressions remain covered.

The first `batch22` checkpoint completed 11 focused groups with 79 passes and
eight failures, with no ignored tests. It stopped before broad verification.
The failures exposed a second end-of-input lookahead in object binding patterns,
an invalid fixture declaring the same private name in static and instance
elements, computed Number/BigInt method-call exclusions, and an over-wide new
import-elision assertion for coercive IR. The value tests and the existing
literal-only import assertions remain intact.

Computed Number and BigInt calls now use ordinary property-key conversion and
receiver-preserving indirect calls. Their regressions cover Symbol identity,
primitive getter/call receivers, base/key/getter/argument order and abrupt
completion identity.

The corrected `batch22r1` completed 129 of 198 planned groups with 2,857
passes and seven failures, with no ignored tests. Its failures were five source
guards that still assumed the earlier async disposal layout and two library
assertions that assumed every script allocated the runtime heap. Those guards
now check the current ownership boundaries, and the heap assertions cover both
runtime-free and allocating programs without weakening their layout checks.

## Saved checkpoint, 2026-09-22

The frozen `batch22r2` compiler completes all 198 planned source/library and
runtime groups with 3,446 passing tests, two failures and zero ignored tests.
Every group ran on this checkpoint; no earlier group result was reused.
Its 13 initial changed-path groups pass all 92 tests. The source/library
verification boundary includes those initial groups and passes all 129 groups,
with 2,864 passing tests. These are subsets of the 3,446 total, not additional
tests. Release workspace all-target checking, CLI/native builds,
the optional `spec-exec-oracle` feature check, formatting and the generated
shortcut-status check also pass.

The compiler was built from parent `715fc58c5c48a7d2388858a11fdcdbac7c5e4ed3`
with uncommitted repairs. Its binary SHA-256 is
`ecedbbe6a6538c43552c0ceb2796b78c722b5ea1dfa4809b850e30536a2f728c`;
its 4,129-input source manifest SHA-256 is
`42aafe466381f1d9d57cc45441ce0bf5badf51b2a0bbf5fe75e22821bf474d80`.
Every frozen source input and test executable was verified after the final
group, before this documentation update. The Test262 pin remains
`aa55200d1310384c5cf69ea95b2a2ecba457007b`.

The two remaining native failures are:

- `intl_host_imports::a_program_without_provider_callers_omits_the_intl_import`
  still finds an Intl import for the original `1 + 1` fixture. The new
  literal-only proof does not yet admit its coercive arithmetic IR. The
  original assertion is retained.
- `language_numerics::run_wasm_backend_keeps_bigint_prototype_result_policies_distinct`
  fails the original captured-main-lexical Symbol update assertion. Its exact
  CLI fixture remains intact; error ownership and binding identity are being
  isolated before the repair.

A separate arithmetic review reproduced a `typeof` bug: static result types
can suppress operand effects and thrown values. Three of four small CLI probes
fail on this frozen compiler; these probes are separate from the native counts
above. They cover unary conversion, comma-expression effects and abrupt
completion, with an arithmetic control. Repairs and candidate execution remain
pending at this saved checkpoint.

Current native verification includes all 16 NumberFormat tests, 16
DateTimeFormat provider tests, six thrower-Realm tests, 13 async resource-loop
tests and eight Arguments descriptor tests. All pass. No result from the
prepared 1,300-execution pinned replay is claimed on this compiler; it has not
run. Earlier pinned replay counts remain separate evidence. This checkpoint
preserves the work in PR #52 and does not close all historical failures.

Immutable compiler/source manifests, exact test membership, executable and
transcript hashes, and the completed-run audit remain under
`target/failure-review/completed-baseline-20260914`, including
`batch22r2-checkpoint-result.json`, `batch22r2-test-verdicts.json` and
`batch22r2-typeof-before`. To refresh the changed paths, run
`cargo check --release --workspace --all-targets --locked -j2`, then
`LILA_MODULE_MEMORY_CACHE_ENTRIES=1 cargo test --release --locked -j2 -p
lila-engine --test aot_intl_numberformat --test aot_intl_datetime_provider
--test aot_throw_type_error_realm --test aot_async_resource_loops --test
aot_arguments_index_descriptors -- --test-threads=2`. The two failing targets
are refreshed with `cargo test --release --locked -j2 -p lila-aot-wasm --test
intl_host_imports` and `cargo test --release --locked -j2 -p lila-cli --test cli
language_numerics::run_wasm_backend_keeps_bigint_prototype_result_policies_distinct
-- --exact`.

## Arithmetic completion and Instant locale follow-up, 2026-09-22

The `batch22r3` frozen compiler passes both original `batch22r2` failures.
Number-only coercive arithmetic now uses the canonical emitter's static operand
proof to avoid unrelated runtime imports. Unproven operands retain their
observable conversions. Static `typeof` results still evaluate the operand and
propagate its effects or abrupt completion. Primitive ToNumeric propagates a
Symbol conversion error before publishing a numeric result or attempting the
binding write, preserving TypeError for captured and uncaptured const updates.

Workspace all-targets release checking, builds and the optional oracle feature
check pass. The focused boundary completes 26 of 217 planned groups with 196
passing tests, one failure and no ignored tests. The other 191 groups did not
run. Its sole failure is a new artifact assertion demanding an internal helper
for exponentiation. The corrected assertion checks the actual
`lila_host.number_pow` dependency and retains the value regression. The original
scalar import-absence and captured-Symbol CLI assertions remain intact and pass.
All 19 separate Symbol-update and `typeof` CLI probes pass without timeouts.

The compiler binary SHA-256 is
`34ea89391f4a5b7e14da0300a2a5ade9602d39de850f734a5f7cf1b30781e3d3`;
its 4,129-input source manifest SHA-256 is
`e1a0cd8f77cead115b638d39d2bb7e36b78ec6f900c2ccb5e4b5b08e6396646a`.
The audited partial result is `batch22r3-checkpoint-result.json` in the local
evidence directory. No pinned replay ran on this compiler.

CI also exposed two stale expectations: the Arguments length source guard
still looked for the implementation before its ownership extraction, and a
typed-array fixture expected Float16Array to be absent. The guard now verifies
the extracted descriptor/accessor path. The fixture positively checks
Float16Array.from NaN conversion while retaining every other assertion.

The next batch adds the missing own `Temporal.Instant.prototype.toLocaleString`
method. It validates the receiver before observing locales or options and
uses the intrinsic DateTimeFormat owner with exact epoch values, intrinsic
identity and called-method Realm behavior. Six native/IR/artifact regressions
cover those boundaries; see the [method contract](contracts/temporal-instant-methods.md).
Three standalone before probes fail on the saved compiler and remain paired
for candidate execution. The complete pinned method cohort contains 42
executions, including 30 verified merged-main Bugs and 12 historical-success
controls. It is disjoint from the earlier 1,386 selections, giving 1,428 total;
these are prepared selections, not passing results. The Islamic-tbla formatting
expectation remains selected even though its calendar may exceed the existing
provider profile. Candidate verification is pending.

The first `batch23` run passed workspace all-target checking and built the CLI
and native test executables. Its focused boundary completed 33 of 221 groups:
269 tests passed, one test failed and none were ignored. The 188 remaining
groups and all pinned candidate executions did not run. The single failure was
a new exponentiation artifact assertion whose all-literal input was folded
before code generation; it therefore could not require a `number_pow` import.
The assertion now uses a runtime binding to test the actual import boundary.
The original scalar-import, captured-Symbol and new Instant focused targets
all passed in the frozen run. The corrected assertion passes its exact release
test locally; the full 221-group checkpoint remains unverified.

Refresh the changed paths with `cargo check --release --workspace --all-targets
--locked -j2`, then `LILA_MODULE_MEMORY_CACHE_ENTRIES=1 cargo test --release
--locked -j2 -p lila-engine --test aot_runtime_import_reachability --test
aot_bigint_numeric_updates --test aot_temporal_instant_methods --test
aot_intl_datetime_provider --test aot_date_locale -- --test-threads=2`.
Run `cargo test --release --locked -j2 -p lila-aot-wasm --test
runtime_import_reachability --test intl_host_imports --test
typed_array_to_locale_string_witness_structure` and `cargo test --release
--locked -j2 -p lila-ir --test temporal_instant_methods` for the artifact,
source and IR boundaries. The exact typed-array CLI regression is
`cargo test --release --locked -j2 -p lila-cli --test cli
typed_array::run_wasm_backend_succeeds_for_typedarray_from_nan_conversion_fixture
-- --exact`. Published full-suite counts remain unchanged.

## Merged-main verification

PR #52 was merged before this checkpoint completed. The follow-up
verification of the merged tree, its fixes and remaining gaps are recorded in
[merged-main verification](merged-main-verification-20260923.md).
