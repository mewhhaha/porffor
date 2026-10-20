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
![Test262: 102,978 passing, 16 failing, 0 not implemented; out of scope not classified](docs/rust-rewrite/assets/test262-status.svg)

Published snapshot: **2026-10-20**, pin `6a4268a93`,
102,994 mode-specific executions. Red combines 16 bugs/crashes;
gray means unimplemented, not unrun. Measured compiler: `c5053c9d4` on the Wasm-AOT backend. This is a complete pinned-suite baseline, not full ECMAScript conformance.
[Published evidence](test262/snapshots/published-status-wasm-aot.json) · [Run details](docs/rust-rewrite/test262-current.md).

Blue reserves **out of scope** for dynamic `eval` / Function-constructor cases
requiring a parser or VM inside emitted Wasm. Scope classification is unaudited,
so that count is **unknown**, not zero; the published totals
above remain unchanged. [Scope details](docs/rust-rewrite/runtime-requirements.md#out-of-scope-tests).
<!-- scriptw-test-graph:end -->

## Caveats

- **Conformance is incomplete.** The chart identifies its measured compiler
  and complete pinned suite. [Current verification and remaining work](docs/rust-rewrite/current-inventory.md).
  [Verified repairs and the remaining labelled-function case](docs/rust-rewrite/annex-label-triage-20261020.md).
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
Rust rewrite status must be read in layers, not one vanity number:
- Fake wasm-safe Test262 subset: `187/187` green
- Fake full Rust rewrite suite: `191/191` green
- Pinned real Test262 baseline (`wasm-aot`, refreshed `2026-10-20`): `102978/102994` not green (`100.0%`)
- Real Test262 goal: Success=102978/102994 (100.0%); burn down NotImplemented=0, Crash=0, Bug=16 to zero
- Pinned revisions: `ecma262=5345883164f463e87f8b40aca4956157ecba8783` `test262=6a4268a9354a545d41c3b62efebc478ed8c521f4`
- Compiler build fingerprint: `53007aa326f737bd5721312044ac1e8e5981e34481941eef5707bf6c48c7380e`; executing image SHA-256: `771a15da06cbac5c7b45b24ecd8b9232af152eac9f1e24cd1e615976642ff1c6`
- Current real outcomes: `Success=102978`, `NotImplemented=0`, `Crash=0`, `Bug=16`
- Biggest current real failing kinds: `Runtime=16`
- Biggest current real failing origins: `unknown=16`
- Worst current real matrix targets: `built-ins/Object/getOwnPropertyDescriptor: 116/120 passed`, `built-ins/Object/getOwnPropertyNames: 88/90 passed`, `built-ins/parseFloat: 106/108 passed`
- Published status artifacts: `/workspace/porffor/target/scriptw-test262-refresh-20261020-run1/snapshots/scriptw-current-20261020-aggregate-13327628081868727304.json` and `/workspace/porffor/target/scriptw-test262-refresh-20261020-run1/snapshots/scriptw-current-20261020-aggregate-13327628081868727304.txt`

As of `2026-10-20`, Rust Wasm-AOT path is at 100% of repo fake coverage, not 100% ECMAScript. Project is still off literal 100% until full pinned real Test262 run is green for Rust path.

Status refresh commands:
- `cargo test -p scriptw-engine --quiet`
- `cargo test -p scriptw-cli --quiet`
- `./target/debug/scriptw test262 run language/wasm/pass --suite-root crates/scriptw-test262/tests/fixtures/fake_test262/vendor/test262 --execution-backend wasm`
- `./target/debug/scriptw test262 run --suite-root crates/scriptw-test262/tests/fixtures/fake_test262/vendor/test262`
- `./scripts/publish-real-status-low-ram.sh wasm-aot codex-published-real`

When counts move, update this block in same change. Do not claim full Test262 `100%` from fake-suite numbers.
<!-- scriptw-status:end -->

</details>

## Documentation

[Project inventory](docs/rust-rewrite/current-inventory.md) ·
[Rust architecture and development](docs/rust-rewrite/README.md) ·
[Contributing](CONTRIBUTING.md) ·
[Development history](DEVELOPMENT_HISTORY.md).
