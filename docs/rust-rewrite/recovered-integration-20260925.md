# Recovered integration verification — 2026-09-25

The interrupted Temporal IANA and TypedArray Number-key/detachment lanes are
integrated on `feat/test262-upstream-20260924`, in the `t262-upstream` worktree.
The original `fix/after-pr52-20260923` checkout remains untouched.

The [TypedArray recovery report](typedarray-number-key-recovery-20260925.md)
and [Temporal recovery report](temporal-iana-recovery-20260925.md) distinguish
saved lane results from fresh recovery checks. Their Test262 results use
upstream `7ab7faf`, vendored tree
`91b2052adad1f066ae031e2ff3a1e9bd6d732886`. They are not a full-suite publication.

## Review and integration repairs

- Numeric property-key review covered `-0`, invalid indices, Proxy String keys,
  prototype receivers, immutable buffers, and value-coercion ordering. A new
  engine regression covers Array prototype getters, Proxy prototypes and index
  boundaries through the ordinary compiled execution path.
- Temporal host requests now reject extreme signed-second values without
  signed-absolute-value overflow. Decoded-wire tests exercise both integer
  extremes, every query kind, and named/fixed zones. Encoding derives the
  identifier from the stored closed zone value rather than accepting a second,
  potentially inconsistent identifier.
- Generated Intl identities were refreshed. The named-zone identity recipe now
  includes the Temporal wire protocol and kernel sources; a mutation test proves
  that either source change invalidates the identity. Pinned catalogue and
  transition data are unchanged.
- Thirteen engine unit-test setup calls attempted to reconfigure the shared
  compilation pool after other tests could initialize it. These calls were
  removed from five test modules after checking that none asserted pool policy.
  Semantic assertions and the production configuration API are unchanged.
- The full engine run exposed three test defects. Async iterator disposal must
  call `return` with zero arguments; its expectation now matches the pinned
  Test262 test and the specification. The Promise.try identity test now handles
  and asserts its intentional rejection instead of failing the engine's
  unhandled-rejection policy. The setter test replaces an obsolete unsupported
  syntax expectation with checks for default evaluation, explicit arguments,
  assignment results and the receiver. All three corrected tests pass; no
  production behavior was changed to satisfy them.

## Fresh integrated verification

The product source at `7df339486` passed:

| Check | Result |
|---|---:|
| `lila-front`, `lila-ir`, `lila-intl`, `lila-aot-wasm`, `lila-test262`: all test targets | 4,510 passed; 0 failed; 0 ignored; 528 targets |
| Temporal, Intl, array-index, decimal-scratch, for-in and Number-key engine integration targets | 175 passed; 0 failed; 0 ignored; 22 targets |
| Named-zone and time-zone-name generator tests | 18 passed |
| Keyword-alias and Locale information generator tests | 15 passed |
| Named-zone, time-zone-name, datetime identity, keyword-alias and Locale information generation checks | passed |
| Formatting, whitespace, module boundaries, host ABI, repository identity and legacy-retirement checks | passed |

The engine test-setup repair is `a07d4f133`. Its complete engine unit run finished
with **771 passed, three failed, zero ignored**. The three failures above were
corrected in `33eb4a9a4`; a rebuilt focused run passed **3/3**. This is a complete
initial run plus focused verification of its corrections, not a claim that the
entire engine unit suite was rerun after those test-only edits. None of the
shared-pool setup failures recurred.

The complete CLI target inventory at `33eb4a9a4` passed across **22 areas:
806 passed, zero failed, one existing declared ignore (807 tests)**. The ignore
is `heap::run_wasm_backend_succeeds_for_heap_page_boundary_stress_fixture`,
owned by T05 in `crates/lila-cli/tests/known-failures.tsv`. It was not executed.
The frontend fixture-subset CLI test also completed its child Wasm-AOT run:
**187/187 fixture executions passed**. This is fixture evidence, not full
Test262 conformance.

Rust commands use `--release --locked -j3` and `--test-threads=3`, under
`systemd-run --scope --user --slice=lila-build.slice -p MemoryMax=10G
-p MemorySwapMax=0`. Long commands use `scripts/run-watched.sh`. Logs live in
the integration worktree's `target/watched/recovery-*.log`; machine-readable
summaries and the release CLI area runner live under
`target/recovery-20260925/`. CLI area selection is derived from the compiled
libtest inventory, including the existing declared ignore, and overlapping
substring filters are excluded so every test belongs to exactly one area.
The temporary runner initially read the progress wrapper's log rather than the
libtest verdict log. Its parser was corrected, and a separate audit reconciled
every area's actual verdict, exit status and counts against the compiled
inventory. All 22 test processes exited zero; the corrected runner resumed
from the audited receipts and exited zero without rerunning them.

The combined crate command is:

```sh
cargo test --release --locked -j3 \
  -p lila-front -p lila-ir -p lila-intl -p lila-aot-wasm -p lila-test262 \
  --tests --no-fail-fast -- --test-threads=3
```

The full engine unit command is:

```sh
cargo test --release --locked -j3 -p lila-engine --lib -- --test-threads=3
```

Each CLI area uses:

```sh
cargo test --release --locked -j3 -p lila-cli --test cli -- \
  --test-threads=3 '<area>::'
```

The `array::` invocation additionally uses `--skip typed_array::` to avoid
libtest's substring overlap; `typed_array::` runs separately. No test is omitted
by this partition. Final formatting, whitespace, module-boundary, host-ABI,
repository-identity and legacy-retirement checks also passed after the test
corrections.

## Continuing runs and limits of the evidence

The previous session's copied `c7bbcea53` compiler completed the Array
`6117/6117`, TypedArrayConstructors `1446/1446`, ArrayBuffer `442/442`,
DataView `1122/1122`, and Atomics `778/778`
reruns during recovery. These supplement that lane's saved TypedArray
`2890/2890` result; they do not test the integrated Temporal compiler.
Its remaining family queue and the before/after built-ins Temporal sweeps
were preserved rather than restarted. Partial checkpoints are not passes.

An additional NumberFormat profile regeneration check was terminated with exit
143 before completion; its last reported progress was 826 of 1,082 source profiles.
It had verified the pinned source archive and reported no mismatch before
termination; the full regeneration check is **incomplete**. NumberFormat inputs
and generated products are unchanged by this recovery.

The full pinned Test262 suite has not been refreshed or published. Known
calendar, time-zone authority and host-capability gaps remain explicit in the
lane reports. Published full-suite status and counts are unchanged.
