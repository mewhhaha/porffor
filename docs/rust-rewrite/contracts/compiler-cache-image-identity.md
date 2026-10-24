# Loaded executable identity and startup experiments

`CompilerIdentity::current()` continues to capture SHA-256 of the actual loaded
executable, opened through `/proc/self/exe` on Linux. It validates unchanged
length/modification metadata around the read and rejects missing, unreadable,
changing or empty images. Reports retain this full SHA-256 evidence regardless
of the cache experiment. A source fingerprint alone never authorizes a
compiler-bound cache.

`SCRIPTW_COMPILER_CACHE_IMAGE_HASH=sha256|blake3` selects a separate, closed
compiler-cache image-hash cohort, parsed once per process. The default is the
existing SHA-256 image digest and exact `scriptw-program-cache-compiler-v4`
fingerprint. Explicit BLAKE3 hashes the same opened executable serially under
the same metadata checks and uses the separate
`scriptw-program-cache-compiler-v5-blake3` domain. Both cache fingerprints bind
the validated build-source digest and actual executable digest. The outer cache
fingerprint remains SHA-256; it never labels the BLAKE3 image digest SHA-256.
Unknown selections and image read failures disable compiler-bound caches; the
diagnostic accessor returns the concrete error without selecting another hash.

The experiment pins the maintained official [`blake3` crate, version1.8.7](https://github.com/BLAKE3-team/BLAKE3/blob/1.8.7/Cargo.toml).
Only its standard single-threaded API is used; the optional Rayon and mmap
features are disabled. The official [implementation repository](https://github.com/BLAKE3-team/BLAKE3)
describes its runtime SIMD detection and feature choices. No speed improvement
is accepted until paired startup evidence is retained.

Private validated build metadata is shared with portable-bundle source-only
admission. It supplies source identity without hashing the executable; it does
not supply executable evidence. External bundle checksums, source/profile/Intl
and full raw-Wasm/layout validation are unchanged.

`compiler_cache_identity()` exposes the actual admitted algorithm, image digest
and domain. New boundary examples report those separately from the mandatory
`executable_sha256`. The ordinary six benchmark harness files remain unchanged;
nondefault cache-image cohorts must not be merged with their default reports.

Runtime profiles additionally report R and P module loading separately. A P
cache hit is not evidence of an R hit. Optional embedded-image spans cover
manifest/digest admission, portable lease acquisition including decode/digest,
and native deserialization. They are nested within R loading and the overall
module phase. Matched diagnostic images retain their existing separate access
label; their production cost remains in R loading. Cache-owner observations
report actual entry/image-byte limits and retained native image accounting,
independent of GC capacity or process RSS.

The `scripts/startup_retention_cohort.py` experiment uses the same frozen
executable in A/B/B/A order:1entry/64MiB versus2entries/128MiB. It changes only
each measured payload's positive cache-limit environment inside the existing
4GiB/one-CPU cgroup, retaining all code in the existing accounted cache and
creating fresh Stores/Realms. It never pools mutable JS state or changes global
verification budgets. The old30fc executable cannot report R cache outcomes or
owner occupancy: receipts mark those unavailable and preserve its P-only label.

The completed observations are recorded in the
[measured checkpoint](../performance-six-workstreams-20261024.md). Reproduction
requires the matching frozen executable, rather than the current rebuilt runner:

```sh
python3 scripts/limited_verification.py -- python3 scripts/startup_retention_cohort.py \
  --runner /path/to/frozen-30fc/runtime_baseline \
  --fixtures target/performance-work-20261024/after/baseline/workload \
  --source-commit 30fc1931471646253e339100a6d392b2337a373b \
  --time /path/to/gnu-time \
  --output-dir NEW_DIRECTORY
```

The source-commit and loaded-executable checks reject a subsequently rebuilt
binary at the same path. The completed checkpoint includes the Rust SDK,
cache identity/read-failure and loader regressions, driver cohort checks, and
serial measurements under the unchanged resource budget.
