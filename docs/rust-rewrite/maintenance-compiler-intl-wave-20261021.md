# Compiler, runtime and Intl maintenance — 2026-10-21

This batch addresses H061–H090 and H092–H100 from the authorized maintenance
inventory, starting at `fc8a80a79`. All 39 hills are complete, bringing the
authorized maintenance inventory to **100 of 100 complete**. The
[source-bound evidence](maintenance-compiler-intl-wave-20261021.json) records
verification and earlier failures. Published Test262 totals and broader
default/release acceptance remain separate.

Implementation commit: `f5b081dc90ef20fb1934486527998664baac4ce9`. The evidence-binding follow-up changes
only this record and its JSON metadata.

## Compiler and frontend

H061/H069/H085/H086 move existing frontend, IR, AOT and engine tests into child
modules by topic. Test bodies and fixture paths are retained. Structure checks
follow their actual new owner. The runtime compiler fingerprint excludes only
explicit test files, avoiding runtime-image invalidation for future test edits.

H062 shares Dynamic Function grammar flags between UTF-8 and UTF-16 preparation.
H063/H068 document encoded source byte lengths separately from UTF-16 units and
Boa code-point coordinates, with astral, isolated-surrogate and line-ending
controls. H064 uses a closed sentinel decoder state and exhaustively checks all
65,536 units. H065 shares checked JSON cursor/digit operations while preserving
its grammar and error offsets.

H066 separates analysis preparation from lowering orchestration. H067 removes
both deep allocation-state copies: the template registry stays caller-owned;
analysis receives its shared parse-cache handle, copyable origin and counters.
Counter updates retain their original timing before lowering. Both registry and
allocation state cease to implement Clone. This removes the two vector copies;
no elapsed compilation speedup is claimed without a matched benchmark.

H070 shares only the identical generator state guard. H072 types the three
lowering invariants while preserving their report labels. H081 shortens default
emission wrapper chains. H082 shares the existing exact runtime/linkage proof;
H083 types the three source-compilation imports; H084 checks function and global
index spaces before mutating emitted imports. H087 shares execution setup with
an explicit parse goal. H088 shares cache parsing with caller-specific defaults.

## Oracle and snapshots

H073 separates oracle module-context and host-session responsibilities. H074
owns agent TLS cleanup with a thread-confined scope. H075 restores the creator
realm on ordinary return, errors and unwind. H076 uses serde_json for all argv
control-character escaping, with round-trip and actual JavaScript evaluation.

H079 centralizes numeric-index, string and symbol property ordering with the
existing array-length and budget checks. Hostile wire orders and boundary array
indices remain rejected. Snapshot validation keeps canonical encounter order.

## Investigation decisions

- H071: FunctionId intentionally names the same callable across definition,
  target, owner and analysis domains. Existing FunctionTargetKnowledge and
  related enums distinguish actual alternatives. No demonstrated cross-domain
  substitution justifies wrapping every String or adding a parallel identity.
- H077: embedded module lookup already uses sorted storage and binary search.
  New characterization covers 128, 1,024 and 8,192 modules, successful and failed
  resolution, and content-kind rejection. Admission and lookup are timed
  separately; identifiers are prepared outside the lookup timer. No extra index.
- H078: SnapshotBudget already owns checked charges; capture charges before
  node/symbol/realm insertion and wire decoding rejects over-budget elements
  before allocation. Existing adjacent-boundary controls remain authoritative.
  A second wrapper would duplicate charges without strengthening the boundary.
- H080: the backend's same_object relation is the identity authority. New wide
  graph controls count comparisons and test hard work-budget rejection. Keep
  the bounded scan until a backend supplies a proven stable hash identity;
  object addresses are not a substitute for that proof. Timing covers capture
  and validation, not isolated identity lookup.
- H089: scriptw-runtime-image already owns immutable runtime bytes behind
  out-of-line accessors; engine owns checked admission. The historical metadata
  reduction is documented in runtime-image-metadata-boundary-20261011.md.
  Fresh artifact and compilation measurements are recorded below. AOT and ICU
  compilation dominate this check; no further crate split is justified by these
  observations. No matched compile-time speedup is claimed.
- H090: NativeSourceCompilerContext already privately owns ValidatedRuntimeArtifact
  and the original armed execution Arc. Its one production constructor checks
  the same remaining budget before and after admission. Existing controls
  verify Arc identity, expiration and profile mismatch; no redundant wrapper.

## Intl and generated files

H092 parses each completed Collator locale once. H093 shares only the locale
lookup truncation primitive, retaining each caller's parsing and extension rules.
H094 records and checks the complete ordered marker load association (request,
attributes, effective locale, fallback). Valid fallback for 1,082 candidate
locales remains admitted; 144 exact metadata IDs are a separate source domain.
The new matrix checks 49 required configurations for every exact ID (7,056),
in addition to catalogue admission. Conditional SpecialPrimaries stays explicit.

H095 centralizes nine raw Collator markers across admission, projection and
export; locale inventory remains a separate envelope member. H096 names binary
framing extents and metadata flag bits. H097 checks nonempty export inventories
and requires singleton markers to contain exactly the unattributed und ID.

H098 makes the Collator producer accept an explicit repository root, including
reexport and documentation refresh. H099 stages related generated bytes and
persists a checked journal before replacement. Interrupted writes roll forward
only after every recorded member is verified. A mixed generation can exist
until recovery; --check is read-only and rejects a pending journal. Individual
replacements are atomic, not the whole multi-file set. Journal/member paths
reject symlinks and escapes, and outputs cannot overwrite transaction state.
H100 separates immutable archive ancestry from current manifest/lock admission;
Locale Collations and Segmenter bind current feature choices and transitive
registry identities. Every source-identity recipe includes its new consumed
owners; generated semantic data must remain byte-identical.

## Verification

- All 109 all-feature native scopes are covered: **9,712 passing tests**,
  zero remaining failures and zero ignored tests. The frozen workspace checkpoint
  passed 107 scopes; complete frontend (181) and engine structure (33) replays
  close two stale source-layout assertions. Only those two test files changed
  between checkpoints. Exact dependency-closure comparisons admit both narrow
  replays; the auxiliary feature anchor owns zero tests.
- The final complete checkpoint passes **685 tooling controls** in 173.973 s
  with zero skips under TMPDIR=/tmp. Three new mutation controls exercise the
  corrected standalone guards. The earlier 682-test checkpoint and ten-test
  storage replay remain recorded with their original six filesystem-layout skips.
- Locked workspace all-target/all-feature Cargo check passed in **272 s**.
  All **15 Intl identity recipes** and module-boundary checks passed.
  Twenty-two pinned semantic/archive/generated files are byte-identical to the
  batch baseline; cached Collator/List/Segmenter exports retain one hash each.
- The affected-source inventory retains all 1,246 earlier tests and adds 18;
  the test-move manifest accounts for 827 moved functions. A literal audit preserves all 606
  multiline literals, including 105 restored after the first move attempt.

### Failed attempts and source boundaries

The first combined native build exhausted disk before executing tests. Closed
build artifacts were archived and verified before unlinking; their inventories
and hashes are retained. A second run exposed test-literal and historical
document-owner issues and was stopped before replay on corrected source.
Fourteen test-only files restore exact original literal bytes and historical
document ownership; none changes product semantics.

The final frozen workspace run then exposed two additional stale test owners:
the frontend classifier census had moved into explicit child modules, and backend
dispatch had moved into run_source. The repairs preserve the original classifier
counts and exhaustive dispatch assertions at their actual owners. Complete
affected-target replays pass. The evidence retains original failed receipts,
source maps, compiler logs and test-only deltas; it does not relabel failures.

The final standalone boundary check exposed four stale source guards for shared
linkage, the extracted engine-test census and typed import slots. Its earlier
log had incorrectly been treated as passing; the evidence now records it as a
failure. Corrected guards retain the original 707-selector hash, all exact test
exclusions and gates, shared pointer/layout checks, and three distinct import
roles. Mutation controls deliberately remove these requirements. The complete
boundary check and full 685-test tooling rerun now pass. These six verification
files change no compiler or native-test source, so the native evidence remains
applicable. The publication record distinguishes those edits from documentation.

The 272 s Cargo-check receipt precedes the test-only corrections above. Complete
native builds and the two affected-target replays verify the corrected test
sources; product sources are unchanged across those later corrections.

The previously timed-out differential generated-campaign control passes in this
checkpoint at its original 5,000 ms deadline. The old failed receipt remains
historical; no cause or general timeout repair is inferred from this pass.

### Measured costs and decisions

These are local observations on the four-CPU cloud machine. The 272 s Cargo check
used warm dependencies, so it is not a cold-build benchmark. The largest recorded
units were AOT optimized build (97.07 s), Intl compilation (40.24 s), AOT check
(25.04 s), AOT test check (24.69 s), and runtime-image build execution (20.14 s).

| Artifact | Bytes |
| --- | ---: |
| AOT metadata | 42,225,082 |
| Engine metadata | 1,962,402 |
| Runtime-image facade metadata | 3,864 |
| Runtime-image static archive | 121,431,432 |
| Raw runtime image | 40,138,962 |
| Native runtime image | 81,291,072 |

The existing separate image producer keeps immutable image bytes out of consumer
metadata. Retain that boundary. No additional crate split or before/after
compilation speedup is claimed from these measurements.

The following exact controls ran alone under one CPU using the same hashed
executables as the complete native checkpoint. Module admission and lookup are
separate timers; snapshot timing includes capture and validation. They are single
samples, not a calibrated benchmark:

```text
embedded module lookup: modules=128 resolutions=128 admission_us=567 lookups=2056 lookup_us=316
embedded module lookup: modules=1024 resolutions=1024 admission_us=3845 lookups=16392 lookup_us=3832
embedded module lookup: modules=8192 resolutions=8192 admission_us=31035 lookups=131080 lookup_us=43591
snapshot identities: nodes=16 comparisons=120 capture_and_validation_us=67
snapshot identities: nodes=128 comparisons=8128 capture_and_validation_us=538
snapshot identities: nodes=512 comparisons=130816 capture_and_validation_us=5481
snapshot hard ceiling: comparisons=1996001 rejected=Work
```

Embedded-module lookup already uses binary search. Snapshot identity comparisons
follow N(N−1)/2 for these wide graphs and are bounded by the existing work limit;
the 4,096-node case rejects at the hard ceiling. Keep the backend's same_object
authority until a stable hash identity is available.

The first measurement selection refused an executable retained from a different
build configuration before collecting any timings. Rebuilding the complete
12-test module target reproduced the original executable and inventory hashes,
and all 12 tests passed. The measurements above use those matching artifacts;
the extra replay adds no unique tests to the native total.

The six longest native groups by elapsed execution time are below. They use
different recorded CPU allocations and concurrency, so these costs are not
additive wall time or a matched before/after benchmark:

| Native group | Seconds |
| --- | ---: |
| scriptw-cli:cli | 597.033 |
| scriptw-engine:aot_realm_modules | 414.580 |
| scriptw-engine:aot_builtins | 406.429 |
| scriptw-engine:aot_intl | 393.884 |
| scriptw-engine:aot_regexp | 374.231 |
| scriptw-aot-wasm:scriptw_aot_wasm | 322.272 |

Keep the shared runtime image and bounded parallel partitions. For iteration,
use complete affected-target replays only with an unchanged dependency closure
and an explicit source boundary, as demonstrated above.

Raw artifacts are retained under target/maintenance-remaining-20261021/next-compiler.
The four tracked upstream Test262 disagreements remain ordinary failures, and the
README retains its complete pre-repair Test262 baseline. Default/release, fresh
full Test262, retention, host and calibrated performance gates remain open in
the [current inventory](current-inventory.md).
