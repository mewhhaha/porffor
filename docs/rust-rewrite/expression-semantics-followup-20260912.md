# Expression semantics baseline follow-up — 2026-09-12

This batch starts from `ac017904aa72c07caf44b70db7ba3a3eb58a911a`, the merge
of PR #50, on a new branch from freshly fetched `origin/main`.

The running historical compiler baseline was frozen at 70,258 of 102,043
executions and 509 of 744 nodes: 58,984 Success, 8,262 Bug, 746 Crash and
2,266 NotImplemented. Compared with the preceding complete frozen observation
at 66,569 executions, 437 additional failures were available for investigation.
All 66,569 preceding execution identities/outcomes and 487 leaf hashes were
preserved. A later counters-only check had reached 69,403 executions; 283
additional failures were counted after that check, but counters cannot identify
which exact executions make up that delta. The 437-entry inventory has no
overlap with PR #50's 102-entry replay.

The repair scope is the 210 newly inventoried executions outside dynamic
import, plus adjacent controls. Every selected execution is reproduced using
a frozen compiler built from merged main before comparing candidate results.
Historical failures already fixed by the merged work remain passing controls.
The complete 437-entry main replay records 175 Success, 118 Bug and
144 NotImplemented, with no Crash or timeout. Within the selected 210 entries,
175 already pass and 35 still fail (17 Bug and 18 NotImplemented).

## Changes

- Number `%` and `%=` share an exact binary-significand remainder emitter.
  It handles extreme finite operands, subnormals, infinities, NaN and signed
  zero, while retained operand locals protect the left value across nested
  right-hand expressions.
- Property deletion evaluates the base and raw key once, performs the required
  object and property-key conversions, then uses the shared deletion operation.
  Nullish bases throw; boxed primitive properties retain their descriptors.
  Computed `length` keys compare string contents for arrays and boxed strings.
  Identifier deletion inside `with` uses the existing environment selection,
  including `Symbol.unscopables`, without reading the property's value.
- Loose BigInt/string equality parses the string exactly and treats invalid
  BigInt text as unequal. Object coercion still occurs in its specified order,
  and the BigInt constructor retains its own SyntaxError behavior.
- Indirect and cross-realm eval can use finite source text held in captured
  bindings. The compiler prepares those sources ahead of time; runtime callable
  identity, source equality, realm ownership and fresh private environments
  still determine execution. Unmatched source remains an explicit AOT
  capability failure.
- The Test262 runner admits the expression and declaration fixtures for
  `SharedArrayBuffer` subclassing. Direct merged-main Wasm execution already
  supports their construction/prototype behavior and growable storage.

## Verification

Verification is in progress. The final replay inventory and audited results
will be recorded here before publication. Generated full-suite status counts
are unchanged by this scoped replay.

## Remaining baseline work

The other 227 newly inventoried executions exercise dynamic import, owned by
T12 in `test262/backlog/ownership-map.tsv`. They need
separate module lifecycle work: retained graphs for computed specifiers, lazy
evaluation, and promise rejection for target parsing, linking and evaluation
errors. Source-phase imports of JavaScript modules also require the specified
rejection. These failures remain visible in the main replay; they are excluded
from this PR's repair cohort, not added to a skip list.

The original full baseline continues against its earlier compiler. Its counters
measure that compiler, while this PR's paired replay measures these changes.
Neither is a completed, current full Test262 conformance result.
