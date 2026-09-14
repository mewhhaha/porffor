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

## Verification in progress

The three focused NUL grammar tests pass for Script and Module parse goals.
The compiler and native regression checkpoint for the complete first batch
is pending. No generated full-suite status numbers have been changed.

The wider main replay has also reproduced module namespace, dynamic import,
Temporal calendar, Intl service, `for-in` and suspension failures. Module
namespaces need exotic internal methods rather than observable export getter
properties. Temporal currently supports ISO and Gregorian calendar arithmetic;
adding further calendar identifiers alone would give incorrect results.
These are continuing implementation work, not resolved by the first batch.
