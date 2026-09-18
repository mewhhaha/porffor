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
