# Observed paired-run variation — 2026-10-23

P005 is complete as a diagnostic for the preserved source52/source53 comparison.
It establishes an empirical range for these runs, without rerunning JavaScript
or changing earlier acceptance decisions. The 16 case-wise ABBA comparisons
contain 64 separate runner processes, each with seven measured fresh-Store
invocations after ten warmups. Both products used the same pinned CPU and checked
workloads; their compiler, executable, Wasm and runtime identities are retained.

[Recomputed observations](performance-repeatability-20261023.json) bind the
exact receipt, analyzer and every raw child JSON. The [source checkpoint](performance-string-hash-20261023.json)
binds its [portable raw archive](performance-string-hash-20261023.tar.gz).
The analyzer verifies byte digests, ordered ABBA rounds, source/runner/Wasm and
runtime identities, warmups, module-cache hits, raw timing samples and cross-round
stdout consistency. The original producer supplied source-byte and Node-oracle
checks; this analysis does not independently rerun the oracle.

The two same-source process medians give the **observed round spread**:
`(larger / smaller - 1) * 100`. The last two columns compare adjacent A/B and
B/A rounds, respectively; negative means the after product was faster.
Every sample and slower round remains in the evidence.

| Workload | Before spread | After spread | A/B effect | B/A effect |
| --- | ---: | ---: | ---: | ---: |
| `map-get-short.js` | 24.21% | 7.78% | -5.34% | -29.30% |
| `map-get-long.js` | 15.23% | 0.20% | -56.14% | -49.57% |
| `map-get-numeric.js` | 5.24% | 2.53% | +1.88% | -5.58% |
| `set-has-short.js` | 2.16% | 11.84% | -10.65% | +2.09% |
| `set-has-long.js` | 2.80% | 5.32% | -55.65% | -51.98% |
| `set-has-numeric.js` | 5.02% | 0.55% | +4.79% | -0.76% |
| `property-get-short.js` | 31.58% | 1.84% | -14.24% | +14.92% |
| `property-get-long.js` | 11.64% | 18.31% | -83.36% | -82.37% |
| `property-get-numeric.js` | 2.01% | 9.64% | -9.55% | -2.79% |
| `retained-unhashed.js` | 11.26% | 8.81% | +7.04% | +4.69% |
| `retained-first-hash.js` | 2.88% | 10.78% | +6.96% | -0.67% |
| `define-long-missing.js` | 13.30% | 8.88% | -44.45% | -46.61% |
| `define-long-present.js` | 6.99% | 7.81% | -66.86% | -66.61% |
| `integer.js` | 9.33% | 4.83% | -1.96% | -14.46% |
| `objects.js` | 4.01% | 0.44% | -9.72% | -12.81% |
| `retained-objects.js` | 0.69% | 6.93% | +5.60% | -0.55% |

Short property lookup changes sign between pairs (about −14% and +15%), while
long property lookup is consistently about 82–83% faster. Long Map and Set
lookup pairs also retain large improvements. Unhashed-string retention is
slower in both pairs (about +7% and +5%); first-hash and ordinary retained-row
effects change sign. Those costs and uncertainties remain visible.

These are **two round medians per product**, not a confidence interval,
significance test, universal noise percentage or bound on future variation.
Timing drift, GC scheduling and other run-to-run effects are not causally
separated. Before accepting a small future gain within the observed variation,
collect further balanced or same-source controls for that workload. Do not
discard inconvenient observations or pool unrelated workloads into one threshold.

Reproduce using the archived child files; this runs only Python analysis:

```sh
mkdir -p target/repeatability-evidence
tar -xzf docs/rust-rewrite/performance-string-hash-20261023.tar.gz \
  -C target/repeatability-evidence --wildcards \
  'evidence/candidate53-hash-confirm/*.json'
python3 -B scripts/paired_repeatability.py \
  target/repeatability-evidence/evidence/candidate53-hash-confirm/receipt.json \
  --output target/repeatability-reproduced.json
cmp docs/rust-rewrite/performance-repeatability-20261023.json \
  target/repeatability-reproduced.json
```

Four focused tooling tests pass, including raw-byte tampering, altered source
identity, incomplete/reordered rounds, invalid samples, changed stdout,
cache misses and changed runtime identity. The source53 native and original
Test262 acceptance remains its separate checkpoint; this diagnostic changes no
compiler code, full-suite graph or V8 baseline score.
