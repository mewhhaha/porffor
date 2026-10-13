# Complete native baseline and repair batch — 2026-10-12

The source-frozen `cd8aa4a32b785c823983bf3a2d3ccf71fb26019b` baseline
completes all 106 Cargo workspace scopes: 13 libraries, 78 integration targets,
two binaries and 13 documentation targets. All features and ignored controls
are included. There are 8,939 passes, 11 failures and zero ignores; 97 scopes
pass and nine contain failures. Exact qualified verdicts, inventory/transcript
hashes and source identity are preserved in
[the complete baseline](native-workspace-baseline-20261012.json).

Against the original 354 controls, seven failures are eliminated, one remains
and no previously passing control fails. Against the preceding 3,574-control
gate, the remaining stale Annex ownership assertion is eliminated, the mutation
campaign remains red and no previous passing ID regresses. Wider coverage finds
additional failures; those are recorded separately from regressions.

The baseline failures are the original-budget mutation campaign, five outdated
source/contract assertions, the Bytes module immutable-buffer getter, WeakMap
primitive receiver diagnostics, an Error-stack fixture expecting an own property,
a generated provider documentation example and the twenty-case performance
workload. The last aborts on a required, unbound fixture host helper before its
timing assertion. The warm and cold single-case performance controls pass.

The coherent repair batch installs the missing branded
`ArrayBuffer.prototype.immutable` accessor, preserves called-Realm/proxy controls,
restores distinct primitive WeakMap/WeakSet receiver diagnostics and aligns the
Error-stack fixture with the pinned prototype-accessor proposal. Source guards
retain their actual exhaustive domains and validate current local contracts.
The ICU repair authenticates the captured raw exporter bytes and projects only
the schematic documentation into an executable provider example.

Static literal property names use their already-interned String keys; computed
keys retain ordered evaluation and full ToPropertyKey. JSON punctuation writes
fixed UTF-16 units into the private output buffer without temporary JS strings.
Minimal Intl runtime identity hashes are reused after first complete admission
of the process-wide immutable selected bundle. Custom profiles still hash their
full physical frames, including profiles reusing a textual name.

The runtime producer selects Backtracking register allocation for immutable R.
Program execution retains Fast/SinglePass and the existing Wasmtime feature,
GC, epoch, stack, target and admission settings. A diagnostic using the actual
baseline raw R, with the allocator as the sole changed input, measures native
images of 143,328,328 and 79,702,936 bytes: 44.391% smaller. Uncached compilation
is 60.178 versus 80.063 seconds; loading is 0.0618 versus 0.0365 seconds. These
measurements establish size/load behavior, not JavaScript execution speed or
campaign acceptance. Fresh actual producer and native measurements are required.

The 2026-10-13 runner dispatches independent whole scopes before idle barriers,
including the partitioned CLI. Original inventory order, object/index ownership,
labels and continuation membership remain unchanged; only dispatch order changes.
This prevents an early CLI drain from leaving later independent targets unopened
while long Intl/RegExp scopes run. Four focused ownership/order controls pass;
there is no controlled before/after native scheduling timing yet. The complete
[default checkpoint](native-default-checkpoint-20261013.json) records 8,881 passes
across 105 frozen-source scopes with zero failures or ignores.

`scripts/run_workspace_tests.py` preserves one Cargo feature graph, seals actual
artifact selection and checks every complete native inventory. The optional
CLI partitioner preflights disjoint exact inventories before executing any body,
then requires every ID exactly once with zero ignored controls. Documentation
uses `--no-fail-fast` so one failure cannot prevent collecting later scopes.
Publication, performance, documentation and original-budget campaigns remain
idle-exclusive. Completed test executables may be retired after their hashes
and native transcripts are retained; pass/fail verdicts are never reused.

This record distinguishes completed baseline evidence from repairs awaiting
verification. Full successor/default suites, current pinned Test262 acceptance,
publication, sustained campaigns and remaining task acceptance are still open.
The historical September 30 execution-ID aggregate remains unavailable.

The first successor focus completes all 55 selected controls: 54 pass, one
fails and none are ignored. Exact qualified-ID comparison against the frozen
baseline matches 53 existing IDs: seven failures are eliminated, one remains,
and no previously passing ID regresses. Both additional controls pass. This is
focused evidence, not complete successor acceptance. Exact qualified verdicts
and the source-bound comparison are preserved in
[the focused manifest](native-repair-focused-20261012.json) (before receipt SHA
`409049a302bc2a0cb2028c6c84fe14352e19fc41cfd70754ba891c0f600a38d4`,
after receipt SHA
`aebd07561e18180dd99329855cbf13a2e66acd81f889685a5543a2f6e97a9351`).

Bytes now completes the original module predicates on Wasm-AOT. The remaining
optional-oracle failure is the first indexed mutation of immutable data. The
follow-up uses the immutable ArrayBuffer proposal's Boolean Set rejection before
receiver/index/value work, nonwritable/nonconfigurable element descriptors and
the existing SameValue descriptor-compatibility kernel. Explicit typed Read/Write
admission covers mutating methods, species/static construction destinations,
Uint8 codecs and atomic writes; read paths remain admitted. Paired existing
strict/sloppy controls now run both enabled backends, and an added control checks
zero argument/callback observations, empty destinations and preserved reads.
The proposal source inspected is commit
`cdfbfa731ba0e880115f5fd1b7aa48f5203e3ca1`; pinned Test262 immutable controls
also specify the required order. This optional oracle remains excluded from the
default product binary. The first paired follow-up runs 11 controls: ten pass, one fails, zero are
ignored. Bytes now passes both backends. The added write-order control passes;
the existing descriptor control passes Wasm-AOT and fails its newly enabled
SpecExec integrity check. Exact comparison retains this gate-level regression
alongside the eliminated Bytes failure in
[the oracle follow-up receipt](native-repair-oracle-20261012.json).
The oracle's mutable-only freeze shortcut must defer immutable descriptors to
the ordinary integrity kernel. The next repair also rechecks `sliceToImmutable`
detachment/final bounds before allocation and limits ordinary slice copying to
bytes surviving species effects; zero-byte copying does not form an invalid
source range. An added paired control retains strict/sloppy conversion/species
order, shrink/detachment error types, exact copied bytes and empty ranges.
The final paired follow-up passes all 12 controls with zero ignores. Exact
comparison preserves the nine original IDs, eliminates the Bytes failure and
introduces no failure on a previous passing ID; all three added controls pass.
Against the first paired follow-up, the descriptor/freeze regression is
eliminated and the added slice control passes. Source-bound verdicts and actual
artifact/transcript hashes are retained in
[the final oracle comparison](native-repair-oracle-final-20261012.json).

The 473-control Python tooling run exposes one PID-file publication race in the
cancellation fixture, with a duplicate cleanup error. Atomic rename publishes
complete parent/child PIDs without changing watcher deadlines or descendant-kill
assertions. The fresh full rerun passes all 473 tooling controls in 147.777
seconds. Current all-feature/all-target type admission and all ten source guards
also pass after the final freeze/slice follow-up. Workspace formatting uses each
Cargo target's declared edition; formatting inconsistencies were corrected.

Actual selected runtime production reports 18.247 seconds and 2,153 function
stencil hits with zero misses. The selected native image is approximately 77 MiB;
this measures build/image behavior without claiming original-budget campaign or
twenty-case timing acceptance.

A diagnostic output-label collision replaced the first focus's two raw unit
transcripts (AOT and engine libraries); their original receipts, exact verdicts,
inventories and launcher logs remain. The seven repaired-failure transcripts,
other selected scopes and complete 8,950-ID baseline are intact. The first
focused manifest explicitly records this evidence limit. Later diagnostics use
unique labels, refuse existing log paths and seal immutable transcript copies.
Fresh full native unit verification remains mandatory; the lost raw logs are
not reconstructed or treated as available proof.

The current original-budget timing/documentation checkpoint completes all 19
IDs in three named scopes: 17 pass, two fail and none are ignored. The generated
provider doctest now passes. The mutation campaign times out on its first
seed-nine case at the unchanged 5,000 ms deadline; the twenty-case workload
completes correctly but measures 5.304175098 seconds against its unchanged
five-second ceiling. Both single-case performance controls pass. Exact
comparison eliminates the documentation failure, retains the two timing
failures and introduces no failure on a prior passing ID. The complete
[timing checkpoint](native-repair-timing-gates-20261012.json) records all 19
verdicts, actual artifacts, source identity and native transcript hashes.

Separate phase diagnostics use the exact Cargo-selected worker and canonical
seed-nine input, plus the original seed-ten generated input. They retain the
inner five-second budget while allowing the diagnostic parent to collect late
phase results; therefore they do not count as campaign acceptance. Cold total
time is 6.291 and 7.200 seconds: first Intl admission costs about 0.53 seconds,
raw package admission about 0.94 seconds, P emission about 1.75 seconds, and
execution about 1.85 and 2.57 seconds. A standalone trace of the unchanged
twenty fixtures/order measures 4.812 seconds after warmup; this diagnostic does
not replace the actual failing performance gate. Optimizations must create a
repeatable margin under the original limits, followed by fresh native checks.

## Complete successor and next measured batch

The frozen c235e0f4f successor completes all 106 scopes with 8,952 passes, two
failures and zero ignores. Exact comparison preserves all 8,950 baseline IDs,
eliminates nine original failures and finds no failure on a previously passing
native ID; all four added IDs pass. The mutation campaign's finer case scope
regresses seed nine from a selected-dimension match to a five-second timeout;
seed ten remains unexecuted. The twenty-case control measures 5.096707977
seconds against five seconds. Both remain red. The [complete manifest](native-workspace-successor-20261012.json)
binds actual full transcripts, source and both comparison scopes. Fresh AOT and
Engine library transcripts resolve the earlier focused-log re-verification
requirement without reconstructing those lost logs.

Linux/macOS Intl capture and their actual byte/identity/pin/native-count
comparison pass for this revision in [CI](https://github.com/mewhhaha/porffor/actions/runs/38202805367).
Public terminal job HTML and all three artifact metadata identities were verified
locally. Artifact ZIPs were not retrieved; there is no local receipt-byte audit.

Measured cold diagnostics identify IR lookup/signature copies, P compilation
and Array-heavy reflection as costly. Instrumented successor seed ten takes
7.729 seconds, with 539 ms lowering and 2.907 seconds execution; UTF-16
extraction accounts for 1,240 ms and identity searches for 736 ms. These counters
change input and add overhead; they are diagnosis, never five-second acceptance.
The root-written next batch indexes immutable AST recipes rather than inferred
facts, borrows exact signatures and clones only each selected parameter's facts.
Current context/fixpoint/effect phases remain unchanged; native IR unit builds
compare against the previous independent algorithms. Shared ObjectRead returns
proven concrete Array own data and length without a public-descriptor roundtrip.
Key conversion, own accessors, inherited holes, Proxy traps and arbitrary throws
retain their existing paths. A strict/sloppy control checks receiver/key/mutation
and abrupt semantics. The scheduler skips barriers for already completed doc
rows while actual documentation stays idle-exclusive. All-feature/all-target Rust type verification passes; native verification
of this batch completes 2,564 controls: 2,562 pass, two fail, zero ignored.
All 2,563 prior selected IDs remain; no previously passing ID regresses and the
added Array control passes. All 2,175 IR controls pass against the prior lookup
algorithms. Seed nine's case-level regression is eliminated; seed ten retains
the original 5,000 ms timeout. The twenty-case measurement is 5.330369186 seconds.
The [focused source-bound manifest](native-measured-iteration-20261012.json)
preserves these results. Fresh cold diagnosis of unchanged seed ten measures
5.787 seconds, including 540 ms lowering, 892 ms raw runtime admission and
1.328 seconds execution. This is diagnosis, not original-budget acceptance.
The tracked September backlog provides 5,365 historical IDs for exact replay;
all are retained, including two whose file is absent from the current pin.
The original raw aggregate and current compiler-bound replay remain unavailable.
