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
are retained. No generated full-suite status numbers have been changed.
