# Cold compiler verification repair — 2026-10-11

The complete focused attempts on `05f1a740a` executed 506 native tests:
494 passed, twelve failed and none were ignored. Two failures were stale
TypedArray feature inventories. One differential worker exceeded its original
5,000 ms budget. Eight retained campaign reports prove the same AOT timeout,
with normal oracle completion. The ninth campaign failure records only
`WorkerFailure`; its original assertion lacks enough detail to identify the
subtype. Original inputs, reports and outcomes are preserved. The complete
four-test corpus, ownership/stack guards and CLI policy/deadline controls pass.

The signed-zero source completes normally in a separate cold diagnostic:
lowering takes 361 ms, emission 3.845 s, native module loading 1.925 s and actual
JavaScript execution 13 ms. Emission includes 1.524 s of runtime-data admission
and 878 ms of checked raw-runtime package loading. This diagnostic uses its own
empty cache directory and is not a five-second acceptance pass.

The repair keeps the original cases, worker counts, deadlines and runtime
admission. Native Rust dev/test builds optimize Intl admission and the Wasm
emitter at level 1; frontend/IR edits retain level 0. Matching dev/test settings
reuse Cargo units. Wasmtime's Fast and SizeOptimized settings are unchanged.
The optional Boa oracle builds only its Rust library. Its unused `cdylib`
caused unhashed `libboa_engine.rlib` outputs to collide across feature graphs,
forcing repeated three-minute rebuilds. Oracle and product semantics are unchanged.

Raw runtime artifacts now retain at most two completed profiles and 256 MiB of
raw bytes. Weak initialization slots let concurrent callers share an emission
without permanently retaining failed or oversized results. Active artifacts
remain valid after eviction. The native-mode and parsed Intl-kernel caches also
retain two hot profiles instead of an unbounded history. These limits bound
cache ownership, not process RSS or memory held by active callers.

Both TypedArray fixture tables now carry literal upstream feature vectors for
all 47 records, rather than guessing features from filenames. Source, include
and contract fingerprints and the full pinned corpus remain unchanged. The
generator-finalizer assertion prints the full actual report on failure.

The next workspace driver selects artifacts by Cargo package identity and
target, builds the workspace library-test cohort once, uses unique watcher
paths and preserves compressed build logs byte-for-byte. Native targets still
run on one CPU with one libtest thread and include ignored tests.

The final all-feature/all-target workspace type check passes. All 458 tooling
tests, 47 publication-driver controls, eight regenerated Intl reproduction
checks and repository guards pass. The first type attempt caught a four-field
destructure left behind by the five-field fixture migration; the corrected
retry passes and both transcripts are preserved.

Fresh complete native acceptance is next. No speedup or complete acceptance is
claimed yet. Default, release, campaigns and full pinned closure remain required.
The task ledger and handoff remain until their actual acceptance and reference
migration finish.
