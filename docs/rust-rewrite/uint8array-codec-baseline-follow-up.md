# Uint8Array codecs after the September 12 baseline observation

The six Uint8Array Base64 and hexadecimal methods now have Rust lowering and
native Wasm codegen: `fromBase64`, `fromHex`, `setFromBase64`, `setFromHex`,
`toBase64`, and `toHex`. No parser, interpreter, or host codec is embedded in
emitted programs. Candidate verification is pending; the measurements below
are from the frozen baseline and a separate current-main replay.

## Scope and provenance

On 2026-09-12, the continuing `c5115bf03` baseline had completed 60,183 of
102,043 executions across 457 of 744 nodes: 49,477 Success, 8,120 Bug,
746 Crash, and 1,840 NotImplemented. This old-compiler observation is not a
measurement of current main. The prior observation contained 41,545 executions
and 312 nodes; all prior outcome identities and completed leaf hashes remained
unchanged. The additional 18,638 executions contain 11,943 Success and 6,695
non-passing outcomes (5,968 Bug, 721 NotImplemented, and 6 Crash).

Freshly fetched main `6dff6eb0da401710301d705564d8f43c5d93b75f` merges PR #47.
The frozen main oracle was reused from that PR: all 2,881 declared build inputs,
including Cargo configuration and the toolchain file, match main byte for byte.
Its executable SHA-256 is
`e2e46be19feb4682dc52657aa3dab8b904cb189f59b7d4d1a8c8766775bf9cfe`.
The real Test262 pin is `aa55200d1310384c5cf69ea95b2a2ecba457007b`.

All 108 newly observed Uint8Array codec failures reproduce as Bug on main, with
no timeouts. Their exact identities are retained in
[`uint8array-codecs-20260912.observed.executions`](../../test262/replays/uint8array-codecs-20260912.observed.executions).
The complete pinned `built-ins/Uint8Array` subtree contains 68 files and 136
executions; its
[full replay list](../../test262/replays/uint8array-codecs-20260912.executions)
also includes 28 previously passing executions. Those controls must not be
counted as new repairs.

## Semantics

The implementation follows the [ECMAScript Uint8Array specification](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-uint8array).
Only genuine Uint8Array receivers are accepted. String arguments and string
options are checked without coercion. Base64 option access preserves observable
Get order and abrupt completion identity, and validates the backing buffer after
those accesses. Static methods allocate intrinsic Uint8Array and ArrayBuffer
objects in the defining method's realm; setter result objects and errors retain
that realm as well. Methods preserve ordinary descriptor, name, length, and
non-constructability contracts.
Created realms now publish the existing ArrayBuffer and TypedArray metadata
getters, so codec results expose their actual length, offset, and buffer state.

Immutable write rejection occurs at the receiver boundary, before source and
option handling, following the project's existing
[immutable ArrayBuffer extension](https://tc39.es/proposal-immutable-arraybuffer/#sec-validateuint8array).
Detached and out-of-bounds checks remain after the required option Gets.

The Base64 decoder supports both alphabets and all three last-chunk modes,
strict padding and unused-bit checks, exact read/written counts, bounded writes,
and completed-prefix writes on malformed suffixes. Hex decoding checks the
whole UTF-16 input length before bounded decoding and validates each consumed
pair before writing it. Encoding uses the current typed-array view, including
its byte offset, without observable indexed property reads. Raw backing-store
access uses the selected buffer memory for shared and ordinary buffers.

Native regressions exercise detached and resized buffers, shared storage,
immutable buffers, retained foreign-realm methods, option getters, partial
writes, all byte values, and input strings absent from static source literals.
Output representation bounds are checked. The existing Wasm heap allocator's
host memory-exhaustion behavior is unchanged by this feature.

## Verification

The coordinated checkpoint will run the three new native codec targets,
neighboring buffer/typed-array/Temporal regressions, the full 136-execution
pinned codec replay, IR and backend library tests, related backend structural
tests, workspace checks, and the complete fake suite. The fake suite and this
focused real-suite cohort remain separate from published full-suite status.

Refresh the focused real-suite evidence with:

```sh
cargo build --release --locked -j2 -p lila-cli
python3 scripts/replay-test262-executions.py \
  test262/replays/uint8array-codecs-20260912.executions \
  --binary target/release/lila \
  --output-dir target/test262-scratch/uint8array-codecs-20260912 \
  --workers 6
python3 scripts/audit-test262-replay.py \
  target/test262-scratch/uint8array-codecs-20260912 \
  --compiler target/test262-scratch/uint8array-codecs-20260912/run.json \
  --output target/test262-scratch/uint8array-codecs-20260912.audit.json
```

The auditor reconciles every execution mode, native transcript, snapshot, source
hash, and frozen compiler hash. Its optional `--origin` report compares exact
identities and distinguishes timeout rechecks from other repairs. The published
checkpoint additionally verifies every build input against the source commit.

Raw evidence for this checkpoint is retained locally under
`target/failure-review/after-pr47-20260912/`. It includes the frozen observation,
all 6,695 assigned failure identities and reasons, main replay transcripts,
snapshots, source manifests, and hashes. No skips or timeout limits are relaxed.

## Remaining baseline work

A separate 40-execution diagnostic probe of other newly observed groups on main
finds 7 already passing, 28 Bug, 2 Crash, and 3 NotImplemented, with no timeouts.
It is a diagnostic sample, not a repair count or an estimate of main's full-suite
pass rate. The complete added-failure inventory retains ownership and original
reasons; this feature batch addresses the coherent Uint8Array codec group.

| Group | Probes | Outcomes |
| --- | ---: | --- |
| Already passing on merged main | 7 | 7 Success |
| Intl constructor and method entry points | 14 | 14 Bug |
| Intl conversion, formatting and casing | 4 | 4 Bug |
| Temporal methods, calendars and time zones | 7 | 7 Bug |
| TypedArray.fill detachment check | 1 | 1 Bug |
| Error and harness realm behavior | 2 | 1 Bug, 1 Crash |
| URI stress allocation trap | 1 | 1 Crash |
| Unprepared indirect eval | 1 | 1 NotImplemented |
| Generator destructuring suspension | 2 | 2 NotImplemented |
| Destructuring assignment rejection | 1 | 1 Bug |


The sample includes missing Intl constructors and methods; incomplete calendar and
named-time-zone support; `TypedArray.fill` after coercion detaches its buffer;
cross-realm error behavior; URI stress allocation traps; indirect eval without a
compiled specialization; generator destructuring suspension; and assignment to
an uninitialized binding. The probe records the first observed failure. In
particular, Collator and RegExp failures prevent two Intl probes from reaching
the behavior their filenames describe. The two traps still need reductions.

The [40 exact diagnostic identities](../../test262/replays/baseline-after-pr47-20260912.triage.executions)
and their [findings and evidence hashes](../../test262/replays/baseline-after-pr47-20260912.triage.json)
are retained for the next baseline batch.

The created-realm inspection also found absent ArrayBuffer prototype methods
(such as `slice`, `resize`, and transfer methods) and constructor `@@species`.
They remain separate work; this batch repairs metadata accessor publication.
