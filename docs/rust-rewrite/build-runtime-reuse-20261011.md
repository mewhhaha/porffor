# Cargo runtime reuse — 2026-10-11

Cargo's native-runtime producer now shares the existing bounded Cranelift
function-cache owner. It still regenerates and admits the current raw runtime,
calls Wasmtime `precompile_module` on those exact bytes, and publishes a package
bound to the full current compiler fingerprint. No runtime native-file input or
native-module retention owner is added. Cache initialization failure preserves
ordinary precompilation. Cargo watches the existing cache environment settings.

The package, native image and manifest are immutable static byte slices. Their
unique storage avoids expanding large constants at individual compiler uses.
The existing digest, source, target, configuration, mode and native compatibility
checks remain the admission boundary.

The cloud machine retains its inherited 32 GiB memory cap and quota-bound four
CPUs, with serial Cargo/libtest and serial build-time native compilation. Its
function-cache disk budget is now 512 MiB: the measured core working set is
about 313 MiB, so the earlier 256 MiB setting evicted entries during every scan.
Stricter explicit settings continue to be honored. Local cache defaults remain
unchanged. Set `LILA_FUNCTION_CACHE_LIMIT_BYTES=536870912` when reproducing this
cloud checkpoint, then use the normal limited-verification launcher.

| Actual producer | Native precompile | Hits | Misses |
| --- | ---: | ---: | ---: |
| First 256 MiB run | 120.724 s | 992 | 1,106 |
| Repeated 256 MiB run | 119.290 s | 995 | 1,103 |
| Refill at 512 MiB | 53.302 s | 1,726 | 372 |
| Warm at 512 MiB | 37.776 s | 2,098 | 0 |

The complete warm producer takes 42.879 seconds. Every probe produces the same
complete package, native image and manifest bytes. The native image also equals
the prior parser batch's SHA-256
`53807dd87957348705d39d76039a52d944c9502c27367b3fdf8613ffbf89f3da`.
These isolated samples demonstrate this core's reuse, not a general benchmark.
The low-budget result is retained as diagnostic evidence.

A controlled Rust 1.99 metadata experiment with the same 1 MiB payload emits
4,198,119 bytes for `const` versus 1,051,149 for `static`. The actual all-feature
engine library now emits 288,239,418 bytes of metadata; the historical all-feature
check emitted about 1.1 GiB. Those real builds differ in source and check/native
profile, so the controlled experiment supplies the isolated comparison.

All three focused embedded-runtime unit checks pass, covering framing damage,
target/raw/native parity, compatibility rejection and actual execution across
cold/warm caches, changed code and changed optimization settings. The existing
product control also passes: fresh programs execute through embedded R without
native R compilation. The first all-feature build includes dependency bootstrap;
its wall time is not a runtime-cache benchmark.

The [exact receipt](build-runtime-reuse-20261011.json) retains commands, hashes,
compiler identity, probe results and metadata evidence. Complete workspace,
ignored runtime, differential and pinned Test262 acceptance remain pending.
Task states and publisher-generated conformance totals are unchanged.
