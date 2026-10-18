Measurements from clean recovery source `d6c5df9ed790e96448184d12a5e6a948cadd2681`
on the managed 32 GiB, four-CPU cloud machine, 2026-10-18. The
[lossless evidence](verification-costs-20261018.json) includes the actual Cargo
producer events, build-script phase outputs, runtime manifests and release timing
report. This is a cost inventory; it does not establish a full-suite speedup.

| Observed work | Time |
| --- | ---: |
| Debug runtime image compilation, module cache miss | 169.441 s |
| Release runtime image load, module cache hit | 0.749 s |
| Fresh raw runtime emission/admission, debug / release | 3.126 / 1.927 s |
| Complete release CLI build | 1,142.962 s |
| Sum of the 20 release compiler units | 1,141.170 s |

The two runtime producers have identical 40,497,446-byte raw Wasm, 40,497,614-byte
packages and 81,341,296-byte native images. The release producer reuses the native
module produced under the debug build. These are different build contexts, so
their times must not be presented as a controlled application benchmark. They do
show that the shared runtime is already reused across these contexts.

The release build spends almost all its time inside Rust compiler units:

| Crate | Observed unit times |
| --- | ---: |
| `lila-aot-wasm` | 196.27 s + 134.93 s |
| `lila-ir` | 184.14 s + 178.40 s |
| `lila-intl` | 91.03 s + 89.21 s |
| `lila-test262` | 87.70 s |
| `lila-engine` | 55.95 s |

The timing rows alone do not identify the profiles of duplicate units. Assigning
one of those times to a host or product profile requires the corresponding Cargo
artifact evidence. The small `cow-utils` dependency now has aligned test debug
info; [unit-graph reuse is verified](cow-utils-profile-reuse-20261018.json), while
its build-time savings remain unmeasured. A useful crate split would remove a measured
rebuild dependency; moving the same dependencies behind another crate name would
not establish that result.

Keep the existing shared-image reuse and serial Cargo/libtest defaults. Measure
any proposed compiler boundary against these same phase outputs and complete
native/CLI gates. Preserve exact IDs, original budgets and failed outcomes when
comparing end-to-end verification time.

The exact shared runtime also has a measured Wasm size breakdown:

| Section group | Encoded bytes |
| --- | ---: |
| Function code | 16,025,440 |
| Linear-memory data | 1,988,926 |
| Custom sections, mainly selected Intl images | 22,462,458 |
| Other sections and header | 20,622 |

There are 1,032 defined functions. The largest bodies include
`Temporal.Duration.prototype.round` (873,759 bytes), the RegExp matcher
(836,075), intrinsic initialization (633,873), and
`Temporal.Duration.prototype.total` (548,782). These are concrete candidates for
per-function compiler profiling and possible outlining of repeated emitted code.
Their sizes alone do not establish their share of the 169-second native compile.
Rust crate boundaries address Rust rebuild dependencies; emitted Wasm functions
address this separate native compilation phase.

Projected Intl tests build distinct runtime profiles. Current code and data
validation bind each selected profile to its exact runtime bytes. Sharing code
across those profiles would first require comparing code and address layouts,
then a typed data-owner boundary if they differ. Reusing the full embedded image
would bypass the selected-data coverage. No such refactor or speedup is claimed.
