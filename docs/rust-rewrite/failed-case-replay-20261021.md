# Replay of every previous failure — 2026-10-21

All sixteen failed execution IDs from the completed Test262 baseline were
replayed through the fixed Wasm-AOT compiler. The same IDs and executable were
then used with explicit reviewed fixture corrections.

| Failed-case replay | Passed | Bug | Crash | Not implemented |
| --- | ---: | ---: | ---: | ---: |
| Original upstream fixtures | 12 | 4 | 0 | 0 |
| Explicit reviewed fixture corrections | 16 | 0 | 0 | 0 |

The twelve global-property failures are repaired by the compiler. The remaining
four originals are specification/test expectation conflicts: the implicit
`arguments` case, both modes of the Johnston/Honolulu timezone case, and the
labelled-function staging case. They remain red in the original-fixture replay.
The amended result is **not unmodified upstream Test262 conformance**.

## Remaining four recheck

A subsequent, separate rerun selected exactly the four remaining original
execution IDs, with no fixture amendments. **All four still fail.** Ten adjacent
original-fixture controls pass. No compiler change was made in this recheck, and
no additional original failure is claimed fixed. The three upstream test files
were also fetched from Test262's current `main`; all three match the pinned
originals byte for byte. Updating those files from `main` would not resolve this.

| Original test | Modes | Result | Evidence behind the disagreement |
| --- | --- | --- | --- |
| `annexB/language/function-code/block-decl-func-skip-arguments.js` | sloppy | Fail | Expects the implicit Arguments object after the block. Pinned FunctionDeclarationInstantiation tests formal `paramNames`, while implicit `arguments` is added only to `paramBindings`, so the function copy replaces it. The unchanged staging `block-scoped-functions-annex-b-arguments.js` expects that replacement and passes. |
| `intl402/Temporal/ZonedDateTime/links.js` | sloppy, strict | Both fail | Expects Johnston and Honolulu to compare equal. The provider retains Johnston's UM identity rather than Honolulu's US identity. ECMA-402 requires country preservation, but its cross-country algorithm assumes a backzone Link where this data has a Zone. |
| `staging/sm/lexical-environment/block-scoped-functions-annex-b-label.js` | sloppy | Fail | Expects recursive labelled block declarations and outer copies. Pinned lexical collection stops at `LabelledItem : Statement`; ordinary var collection returns empty for a labelled function; Annex B selects directly contained declarations. V8 accepts the original staging behavior. |

The label distinction matters: a **single** label directly containing a function
does contribute a block/case lexical declaration, initialized on scope entry.
A label chain does not. `TopLevelVarScopedDeclarations` follows label chains
only at the script/function/eval top level; it does not lift a function out of an
enclosing brace block. A top-level BlockStatement delegates to ordinary
`VarScopedDeclarations`, whose `LabelledItem : FunctionDeclaration` result is
empty. Thus top-level label hoisting does not resolve the failing block cases.

The ten passing controls cover the contradictory staging `arguments` expectation,
formal/default-parameter exclusions, canonical timezone aliases, historical link
acceptance and strict labelled-function rejection. They support the diagnosis;
they are not a full regression campaign or proof of complete conformance.

[Machine-readable recheck](remaining-four-recheck-20261021.json) and its
[evidence archive](../../test262/snapshots/remaining-four-recheck-20261021.tar.gz)
retain both native snapshots, the exact four- and ten-ID lists, commands,
unmodified exit statuses, transcripts, source hashes and fetched upstream files.
The same source-equivalent optimized compiler and 60,000 ms deadline were reused.
The original four-case command returns exit status 1; the controls return 0.

To reproduce, extract `original-four.txt` and `controls.txt` from that archive,
then use the `run-list` command below with each list and a fresh snapshot directory,
keeping `--suite-root test262/vendor/test262`. No amended suite is needed.
The original full publication remains unchanged. The README chart now shows a
labeled reconciliation with this exact replay and the audited product-scope
partition. The outstanding work is to resolve these test/specification/compatibility disagreements and then
run the complete suite on the resulting compiler; amended tests alone do not
complete that work.

The labelled-function correction follows the pinned LexicallyScopedDeclarations,
BlockDeclarationInstantiation and Annex B direct-declaration rules. It retains
all thirteen original call groups, the same execution mode and each strict-mode
syntax check. The ten unaffected call bodies are unchanged. A block label chain
must lack a binding both before and after its declaration position. A direct
labelled function must be initialized before block/switch evaluation, remain
callable within that scope, and be inaccessible after leaving it. Positive
in-scope checks prevent the new error assertions from masking failed initialization.
Node/V8 accepts the original staging test and rejects the amended expectation;
that disagreement is preserved, not presented as V8 validation of this correction.
The source specification and unchanged originals are included in the archive.

The earlier `arguments` and timezone amendments remain unchanged. The timezone
abstract algorithm still has the documented backzone-Link versus backzone-Zone
gap; this run does not resolve that standardization question. No upstream issue
or message was sent.

## Execution and provenance

Both native replays used the actual Wasm-AOT backend, one reusable case worker,
fresh per-case state, one compiler job and the original 60,000 ms deadline.
Every requested ID completed exactly once; no failure, skip, case filter within
a fixture or timeout extension was introduced.

The executable is the previously verified ordinary optimized global-prelude
candidate, honestly identified as an unversioned source archive. Its immutable
SHA-256 is `ef346e256752843179eceab239b6b995b3e43f1acb97c96b44cb05530a756666`.
Before reuse, 2,581 Rust/manifest/lock/toolchain input files were compared byte
for byte with main at `a5b7bd994`. Cargo patch paths resolve to the same unchanged
vendor sources. There are no product-source changes since the integrated compiler
repair. Reuse avoids another release build; the executable is not relabelled as
a new Git-produced binary. Native snapshots retain its actual compiler identity.

[Validation receipt](failed-case-replay-20261021.json) and
[lossless evidence archive](../../test262/snapshots/failed-case-replay-20261021.tar.gz)
retain the exact sixteen-ID inventory, source checks, original and amended native
snapshots, exit statuses, transcripts, differential output, fixture bytes and
SHA-256 inventory. The [errata manifest](../../test262/errata/manifest.json)
binds each original and replacement hash to the source pins.

## Reproduce the targeted check

Build or select the matching ordinary compiler. Extract `executions.txt` from
the evidence archive and select a fresh snapshot directory for each run:

```sh
scriptw --jobs 1 test262 run-list executions.txt \
  --suite-root test262/vendor/test262 --snapshot-dir target/failures-original \
  --snapshot-name failures-original --execution-backend wasm-aot \
  --threads 1 --timeout-ms 60000 --reuse-case-workers

python3 scripts/prepare_test262_errata.py --output target/reviewed-errata
scriptw --jobs 1 test262 run-list executions.txt \
  --suite-root target/reviewed-errata/suite --snapshot-dir target/failures-amended \
  --snapshot-name failures-amended --execution-backend wasm-aot \
  --threads 1 --timeout-ms 60000 --reuse-case-workers
```

On the managed cloud machine, run verification through
`python3 scripts/limited_verification.py --cloud -- ...` with the finite inherited
memory budget. The original-fixture run is expected to return nonzero and records
the four literal failures; retain that exit status and its output.

This is a replay of the previous failures. The other 102,978 baseline successes
were not rerun on the fixed compiler. The canonical publication remains the
complete pre-fix measurement; the README chart labels its separate reconciled
view explicitly. A fresh full campaign and
the other native/release acceptance gates are still required for closure.
