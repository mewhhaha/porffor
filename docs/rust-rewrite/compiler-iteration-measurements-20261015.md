# Measured compiler iteration costs — 2026-10-15

The genuine default release CLI at `d2f4bc51` took **627.657 seconds** to build.
The [measurement record](normal-cli-build-costs-d2-20261015.json) embeds the exact
Cargo timing report and identifies the ordinary CLI checkpoint.

| Measured work | Seconds |
| --- | ---: |
| Rust code generation in the rebuilt units | 469.75 |
| Rust frontend work | 87.94 |
| Runtime-image producer, including native compilation | 66.76 |

The host IR/AOT build dependencies and the ordinary target IR/AOT libraries both
rebuilt. Their four units took 97.91, 81.73, 134.43 and 127.60 seconds. These are
compiler execution times, so reducing dependency waits alone would leave most
of this cost.

A crate split is useful if a stable IR model lets the Wasm backend and runtime
producer avoid depending on source lowering. Moving files behind re-exports
would retain the dependency and rebuilds. This split is an opportunity; it has
not been implemented or benchmarked.

The runtime already uses a shared, precompiled native image. Its producer had
a recorded module-cache miss for unchanged emitted Wasm. Keeping the exact
compiler/target/configuration cache entry is another concrete opportunity;
image-content equality alone is insufficient for reuse.

An actual warm worker ran twenty fixed, already-passing cases in 2.806 seconds.
Its first runtime-module load took 121.368 ms; the remaining nineteen took
roughly 0.003 ms each. All twenty program-module loads missed the memory cache.
The reused worker therefore already retained runtime code before the new
runtime-only routing. No speed improvement is claimed for that change, and
this small diagnostic does not establish full-suite performance.

CPU, worker, thread, timeout and cache limits remain unchanged. Native checks
reuse exact Cargo-selected test images when source and build inputs permit;
Cargo metadata and verdicts retain their ordinary meaning.
