# scriptw

Scriptw is an experimental JavaScript-to-WebAssembly compiler written in Rust.
Compile JavaScript and its builtins to Wasm, run it through Wasmtime, or embed
it in a Rust application. WebAssembly is the sole output target.

## Benchmarks vs V8

Measured on 2026-10-20 at compiler source `c5053c9d4`. Medians of seven samples;
all workload outputs checked against Node. V8 is faster in these measured cases.

| Measurement | scriptw / Wasmtime | Node / V8 |
| --- | ---: | ---: |
| Cold start | 1,221.570 ms | 32.789 ms |
| Cached start | 920.607 ms | 35.565 ms |
| Execute: 10,000 integer steps | 17.020 ms | 2.400 ms |
| Execute: 500 objects | 44.490 ms | 0.044 ms |

AMD EPYC 9V74; 32 GiB memory limit; four-CPU cloud quota. Both runtimes pinned
to one CPU. Wasmtime 47.0.0; Node 24.19.0 / V8 13.6.233.17-node.51.

Cold and cached starts use fresh processes with empty or populated application
caches; the shared runtime is precompiled and the OS cache is not flushed.
Execution reuses compiled code with a fresh Wasmtime Store or V8 context.
The Rust compiler uses `release`; Wasmtime program modules use its normal Fast
policy. These small workloads are a local baseline.
[Methodology, ranges, raw samples and reproduction](docs/rust-rewrite/v8-baseline.md).

## Tests

<!-- scriptw-test-graph:start -->
![Test262: 96,678 passing, 4,240 failing, 1,125 not implemented; out of scope not classified](docs/rust-rewrite/assets/test262-status.svg)

Published snapshot: **2026-10-02**, pin `aa55200d1`,
102,043 path-based entries. Red combines 4,240 bugs/crashes;
gray means unimplemented, not unrun. This historical snapshot does not establish
current compiler conformance. [Published evidence](test262/snapshots/published-status-wasm-aot.json).

Blue marks **out of scope**: dynamic `eval` / Function-constructor cases that
would require a parser or VM inside emitted Wasm. The snapshot has no case-level
scope breakdown, so that count is **unknown**, not zero; the published totals
above remain unchanged. [Scope details](docs/rust-rewrite/runtime-requirements.md#out-of-scope-tests).
<!-- scriptw-test-graph:end -->

## Caveats

- **Conformance is incomplete.** The chart is a dated publication, not a pass
  rate for the current compiler. [Current verification and remaining work](docs/rust-rewrite/current-inventory.md).
- **Dynamic code generation is limited.** Prepared `eval` / `new Function`
  sources can run; the native host can compile additional supported sources.
  Standalone Wasm does not contain a parser or provide general runtime `eval`.
  Cases requiring a parser or VM inside emitted Wasm are outside product scope.
- **Host APIs are limited.** Scriptw is not a Node.js or browser replacement.
  Three synchronous React SSR cases work; the public browser/streaming entry
  fails. [React probe and limitations](docs/rust-rewrite/react-ssr.md).

## Wasm requirements and support

The backend requires:

- Wasm GC structs and arrays;
- reference types and typed function references;
- exception handling and tail calls;
- multi-memory and threads/shared memory.

The host must also provide the matching Scriptw runtime, `scriptw_host` imports
and Intl data. The native integration selects Wasmtime's copying collector.

| Runtime / host | Scriptw support |
| --- | --- |
| Vendored Wasmtime 47.0.0 through the Scriptw CLI or Rust API | Tested execution path |
| Browsers, Node's Wasm API and other engines | Integration and compatibility unverified; feature support alone is insufficient |
| A bare WASI host | Does not supply Scriptw's JavaScript runtime or host bindings |

[Runtime requirements and dynamic-source details](docs/rust-rewrite/runtime-requirements.md).

## Try it

Requires Rust 1.94 or newer. From a checkout:

```sh
cargo build --locked --release -p scriptw-cli --bin scriptw
printf 'print("Hello Wasm");\n' > hello.js
./target/release/scriptw run hello.js
SCRIPTW_WASM_DUMP=hello.wasm ./target/release/scriptw build wasm hello.js
```

<details>
<summary>Detailed published conformance record</summary>

## Current Status
<!-- scriptw-status:start -->
<!-- lila-status:start -->
Rust rewrite status must be read in layers, not one vanity number:
- Fake wasm-safe Test262 subset: `187/187` green
- Fake full Rust rewrite suite: `191/191` green
- Pinned real Test262 baseline (`wasm-aot`, refreshed `2026-10-02`): `96678/102043` not green (`94.7%`)
- Real Test262 goal: Success=96678/102043 (94.7%); burn down NotImplemented=1125, Crash=39, Bug=4201 to zero
- Pinned revisions: `ecma262=ecma262-current-draft` `test262=aa55200d1310384c5cf69ea95b2a2ecba457007b`
- Current real outcomes: `Success=96678`, `NotImplemented=1125`, `Crash=39`, `Bug=4201`
- Biggest current real failing kinds: `Runtime=4441`, `Unsupported=753`, `HostHarness=162`
- Biggest current real failing origins: `unknown=5191`, `icu-intl=162`, `boa-parser=9`
- Worst current real matrix targets: `intl402/DurationFormat: 0/222 passed`, `intl402/Temporal/PlainDateTime: 29/250 passed`, `intl402/Temporal/ZonedDateTime: 54/250 passed`
- Published status artifacts: `/home/mewhhaha/src/porffor/target/publication-freeze-20260930-1790756341160032048/test262/snapshots/current-pin-wasm-aot-20260930-2885103bea4c-2x1-aggregate-8622785491567939163.json` and `/home/mewhhaha/src/porffor/target/publication-freeze-20260930-1790756341160032048/test262/snapshots/current-pin-wasm-aot-20260930-2885103bea4c-2x1-aggregate-8622785491567939163.txt`

As of `2026-10-02`, Rust Wasm-AOT path is at 100% of repo fake coverage, not 100% ECMAScript. Project is still off literal 100% until full pinned real Test262 run is green for Rust path.

Status refresh commands:
- `cargo test -p lila-engine --quiet`
- `cargo test -p lila-cli --quiet`
- `./target/debug/lila test262 run language/wasm/pass --suite-root crates/lila-test262/tests/fixtures/fake_test262/vendor/test262 --execution-backend wasm`
- `./target/debug/lila test262 run --suite-root crates/lila-test262/tests/fixtures/fake_test262/vendor/test262`
- `./scripts/publish-real-status-low-ram.sh wasm-aot codex-published-real`

When counts move, update this block in same change. Do not claim full Test262 `100%` from fake-suite numbers.
<!-- lila-status:end -->
<!-- scriptw-status:end -->

</details>

## Documentation

[Project inventory](docs/rust-rewrite/current-inventory.md) ·
[Rust architecture and development](docs/rust-rewrite/README.md) ·
[Contributing](CONTRIBUTING.md) ·
[Development history](DEVELOPMENT_HISTORY.md).
