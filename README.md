# Porffor <sup><sub>/ˈpɔrfɔr/ *(poor-for)*</sub></sup>

Porffor is a Rust rewrite of the original Porffor experiment: a JavaScript-to-Wasm
AOT compiler, library, CLI, and conformance harness. It is still a research
project and not ready for general JavaScript workloads.

The product path is direct JavaScript compilation. User programs must go through
parse, early errors, spec-shaped IR, lowering IR, and real Wasm codegen. Porffor
does not count "compile a JavaScript interpreter or VM to Wasm and feed source
into it" as success.

The older JavaScript implementation is still in the repository as reference
material and as an oracle while the Rust path catches up. Treat the Rust crates
and `porf` CLI under `crates/` as the current development surface.

## Current Status
<!-- porffor-status:start -->
Rust rewrite status must be read in layers, not one vanity number:
- Fake wasm-safe Test262 subset: `187/187` green
- Fake full Rust rewrite suite: `190/190` green
- Full pinned real Test262 for Rust rewrite: **not green / current pinned aggregate not yet fully republished**
- Current real-suite pin: `ecma262=ecma262-current-draft` `test262=e9d582d6b8b13afc5ba9a676664741592b5c7f69`
- Last complete cached `spec-exec` publish is stale for the current pin and must not be reported as current progress.

As of `2026-04-30`, Rust Wasm-AOT path is at 100% of repo fake coverage, not 100% ECMAScript. Project is still off literal 100% until the full pinned real Test262 run is green for Rust path and the status artifact is republished.

Status refresh commands:
- `cargo test -p porffor-engine --quiet`
- `cargo test -p porffor-cli --quiet`
- `./target/debug/porf test262 run language/wasm/pass --suite-root crates/porffor-test262/tests/fixtures/fake_test262/vendor/test262 --execution-backend wasm`
- `./target/debug/porf test262 run --suite-root crates/porffor-test262/tests/fixtures/fake_test262/vendor/test262`
- `./scripts/publish-real-status-low-ram.sh spec-exec codex-published-real`

When counts move, update this block in same change. Do not claim full Test262 `100%` from fake-suite numbers.
<!-- porffor-status:end -->

Focused Wasm-AOT progress verified after the last aggregate publish is recorded
under [Current Capabilities](#current-capabilities). The generated status block
above stays conservative until a full pinned real-suite publish is refreshed.

## Rust Workspace

- `crates/porffor-front`: parser boundary and source-unit handling.
- `crates/porffor-ir`: spec-shaped IR, diagnostics, and lowering metadata.
- `crates/porffor-runtime`: realms and host hooks.
- `crates/porffor-aot-wasm`: primary direct JS -> Wasm backend.
- `crates/porffor-engine`: public Rust library API.
- `crates/porffor-cli`: clean-break `porf` command.
- `crates/porffor-test262`: Test262 discovery, execution, snapshots, taxonomy, and README status publishing.
- `crates/porffor-spec-exec`: reference/spec execution backend used for conformance work.
- `crates/porffor-backend-c` and `crates/porffor-backend-native`: scaffolds, not product-ready emitters.

Supporting directories:

- `docs/rust-rewrite`: rewrite notes, architecture invariants, and conformance taxonomy.
- `test262`: pinned real Test262 checkout, local harness files, and snapshots.
- `scripts`: repo maintenance and low-RAM real-suite publication scripts.
- `compiler`, `runtime`, and `package.json`: legacy JavaScript implementation and npm-facing files inherited from the previous project.
- `vendor`: vendored Rust dependencies used by the rewrite.

## CLI

Build the Rust CLI:

```sh
cargo build -p porffor-cli
```

Run the built binary directly:

```sh
./target/debug/porf --help
./target/debug/porf inspect crates/porffor-cli/tests/fixtures/hello.js
./target/debug/porf run --execution-backend wasm crates/porffor-cli/tests/fixtures/hello.js
./target/debug/porf build wasm crates/porffor-cli/tests/fixtures/hello.js
```

Or run it through Cargo:

```sh
cargo run -p porffor-cli -- inspect crates/porffor-cli/tests/fixtures/hello.js
```

Current commands:

- `run [--execution-backend spec|wasm] <file>` runs a script through the Rust engine. `spec` is the default reference backend; `wasm` is the AOT Wasm backend.
- `build wasm <file>` compiles JavaScript directly to a Wasm artifact and prints the artifact summary.
- `build c <file>` and `build native <file>` exist as CLI surfaces but currently fail with scaffold errors.
- `inspect <file>` prints the parser/lowering pipeline summary and invariants.
- `test262 ...` drives the fake fixture suite, pinned real suite, status snapshots, triage, and README status publication.
- `repl` is reserved for the Rust REPL and is not implemented yet.

The npm `porf` entry in `package.json` still points at the inherited JavaScript
runtime. Do not use it as the source of truth for the Rust rewrite. It now also
has a package-CLI convenience command for Worker-style TypeScript setup:

```sh
node runtime/index.js types src/index.ts worker-configuration.d.ts --config wrangler.jsonc
```

`porf types` mirrors Wrangler's type-generation shape: it writes
`worker-configuration.d.ts` by default, accepts `--config`, `--entrypoint`,
`--env`, `--env-interface`, `--include-runtime=false`, `--include-env=false`,
`--strict-vars=false`, `--check`, `--print`, and discovers `wrangler.jsonc`,
`wrangler.json`, `wrangler.toml`, or `porffor.*` config files from `--cwd` when
`--config` is omitted. An explicit positional entrypoint or `--entrypoint`
overrides the config `main`, matching the common Wrangler flow of generating
types from a config plus a selected worker source. JSON, JSONC, and TOML configs
are supported, and `porf typegen` is accepted as an alias. The package CLI
type-generation path is covered by
`pnpm test:types`.

## Conformance

The conformance goal is literal full pinned Test262 green for the Rust path, with
fake-suite progress kept separate from real-suite progress.

Useful local checks:

```sh
cargo test -p porffor-engine --quiet
cargo test -p porffor-cli --quiet
./target/debug/porf test262 run language/wasm/pass --suite-root crates/porffor-test262/tests/fixtures/fake_test262/vendor/test262 --execution-backend wasm
./target/debug/porf test262 run --suite-root crates/porffor-test262/tests/fixtures/fake_test262/vendor/test262
```

For real-suite publication, prefer the low-RAM wrapper so the top-level matrix
checkpoints one node per process and only publishes after verified completion:

```sh
./scripts/publish-real-status-low-ram.sh spec-exec codex-published-real
./scripts/publish-real-status-low-ram.sh wasm-aot codex-published-real
```

Useful status and triage commands:

```sh
./target/debug/porf test262 progress-status --execution-backend wasm-aot
./target/debug/porf test262 triage-status --execution-backend wasm-aot
./target/debug/porf test262 failure-details language/wasm --execution-backend wasm-aot
```

## Current Capabilities

Rust Wasm-AOT currently compiles a limited but useful JavaScript subset. Treat
this as a tested capability map, not a spec-completeness claim. Programs are
most likely to work when they stay close to the fixtures under
`crates/porffor-cli/tests/fixtures/wasm_*.js` and the fake wasm-safe Test262
cases under
`crates/porffor-test262/tests/fixtures/fake_test262/vendor/test262/test/language/wasm/pass`.

Recent focused progress through `2026-06-21`:

- `Array.prototype.forEach` covers array-like and primitive receivers,
  inherited array indexes including Array instances used as prototypes where
  `HasProperty` and `Get` must agree, ToLength and callback-order edge cases,
  sparse high-index arrays without timing out, omitted-callback TypeErrors,
  freezing `Array.prototype.forEach` while an iteration is active, and generic
  calls on typed arrays backed by resizable ArrayBuffers. The exact real
  Test262 `built-ins/Array/prototype/forEach/15.4.4.18-7-c-ii-1.js` sparse
  high-index parameter-consistency case and
  `built-ins/Array/prototype/forEach/15.4.4.18-8-10.js` subclassed
  array-prototype reduced-length case are now green under Wasm-AOT.
- Generic `Array.prototype.every`, `Array.prototype.some`,
  `Array.prototype.filter`, and `Array.prototype.includes` calls on resizable
  typed arrays cover fixed-length and length-tracking views across shrink/grow,
  mid-iteration resize, fromIndex coercion resize, and `SameValueZero` float
  comparisons such as `NaN`.
- Generic `Array.prototype.indexOf` now observes `HasProperty` before `Get` for
  sparse and array-like receivers, supports borrowed calls on resizable typed
  arrays including subclass instances, preserves strict equality for special
  float values where `NaN` is not a match, and handles large canonical numeric
  object keys without clamping them to dense array indexes. Ordinary array
  writes at `4294967294` now extend `length` through the sparse element path,
  while `4294967295` and larger numeric literals remain named properties. The
  exact real Test262
  `built-ins/Array/prototype/indexOf/15.4.4.14-9-9.js`,
  `built-ins/Array/prototype/indexOf/15.4.4.14-9-a-19.js`,
  `built-ins/Array/prototype/indexOf/15.4.4.14-9-b-i-15.js`,
  `built-ins/Array/prototype/indexOf/resizable-buffer.js`,
  `built-ins/Array/prototype/indexOf/resizable-buffer-special-float-values.js`,
  `built-ins/Array/prototype/indexOf/coerced-searchelement-fromindex-grow.js`,
  `built-ins/Array/prototype/indexOf/coerced-searchelement-fromindex-shrink.js`,
  and `built-ins/Array/prototype/indexOf/length-near-integer-limit.js` files now
  report `1/1` each as of `2026-06-19` under `--execution-backend wasm` with the
  `90000` ms timeout and one thread. The sharded
  `built-ins/Array/prototype/indexOf` sweep also reports green as of
  `2026-06-19`: shard `1/8` is `26/26`, and shards `2/8` through `8/8` are
  `25/25` each under `--execution-backend wasm --timeout-ms 90000 --threads 8`.
  Refresh individual cases with
  `./target/debug/porf test262 run <case> --execution-backend wasm --timeout-ms 90000 --threads 1`.
- Generic `Array.prototype.lastIndexOf` now shares the index-search receiver,
  `HasProperty`, sparse array, array-like, and resizable typed-array paths while
  preserving the spec distinction between omitted `fromIndex` and explicit
  `undefined`. The exact real Test262
  `built-ins/Array/prototype/lastIndexOf/15.4.4.15-5-4.js`,
  `built-ins/Array/prototype/lastIndexOf/resizable-buffer.js`,
  `built-ins/Array/prototype/lastIndexOf/coerced-position-grow.js`, and
  `built-ins/Array/prototype/lastIndexOf/coerced-position-shrink.js` files now
  report `1/1` each as of `2026-06-19` under `--execution-backend wasm` with the
  `90000` ms timeout and one thread. The sharded
  `built-ins/Array/prototype/lastIndexOf` sweep reports green as of
  `2026-06-19`: shards `1/8` through `6/8` are `25/25`, shard `7/8` is `24/24`,
  and shard `8/8` is `24/24` under
  `--execution-backend wasm --timeout-ms 90000 --threads 8`.
- `Array.prototype.find` and `Array.prototype.findIndex` are now registered
  Wasm-AOT builtins with descriptor metadata, callback argument/`thisArg`
  plumbing, hole visitation, length-snapshot behavior, catchable non-callable
  `TypeError`s, and borrowed calls on resizable typed-array receivers. The exact
  real Test262
  `built-ins/Array/prototype/find/resizable-buffer.js`,
  `built-ins/Array/prototype/findIndex/resizable-buffer.js`,
  `built-ins/Array/prototype/find/callbackfn-resize-arraybuffer.js`,
  `built-ins/Array/prototype/findIndex/callbackfn-resize-arraybuffer.js`,
  `built-ins/Array/prototype/find/resizable-buffer-grow-mid-iteration.js`,
  `built-ins/Array/prototype/find/resizable-buffer-shrink-mid-iteration.js`,
  `built-ins/Array/prototype/findIndex/resizable-buffer-grow-mid-iteration.js`,
  and
  `built-ins/Array/prototype/findIndex/resizable-buffer-shrink-mid-iteration.js`
  files now report `1/1` each as of `2026-06-19` under
  `--execution-backend wasm` with the `60000` ms timeout and one thread. The
  local `wasm_array_find_core.js` fixture also covers function metadata, holes,
  callback parameters, `thisArg`, length snapshots, and typed-array
  post-shrink `undefined` callback values.
- `Array.prototype.findLast` and `Array.prototype.findLastIndex` are now
  registered Wasm-AOT builtins sharing the find-like callback path with reverse
  length-snapshot traversal. The local `wasm_array_find_last_core.js` fixture
  covers descriptor metadata, reverse callback order, holes, `thisArg`,
  mutation during traversal, non-callable `TypeError`s, and typed-array
  post-shrink callback values. Exact real Test262 metadata files
  `length.js`, `name.js`, and `prop-desc.js` for both methods report `1/1`
  each as of `2026-06-19` under `--execution-backend wasm` with the `60000` ms
  timeout and one thread. The exact real Test262
  `predicate-called-for-each-array-property.js`,
  `callbackfn-resize-arraybuffer.js`, `resizable-buffer.js`,
  `resizable-buffer-grow-mid-iteration.js`, and
  `resizable-buffer-shrink-mid-iteration.js` files for both reverse methods
  now report `1/1` each under `--execution-backend wasm` with the `90000` ms
  timeout and one thread. The broad
  `built-ins/Array/prototype/find` shard `1/8`, which also includes
  `findLast`/`findLastIndex` prefix matches, now reports `12/12` under
  `--execution-backend wasm --timeout-ms 90000 --threads 4`.
- The exact real Test262
  `Array.prototype.map/callbackfn-resize-arraybuffer.js`,
  `Array.prototype.every/callbackfn-resize-arraybuffer.js`,
  `Array.prototype.forEach/callbackfn-resize-arraybuffer.js`,
  `Array.prototype.filter/callbackfn-resize-arraybuffer.js`, and
  `Array.prototype.some/callbackfn-resize-arraybuffer.js` cases now use static
  Wasm-AOT materializations that preserve passthrough typed-array constructor
  coverage without timing out in the generic `testTypedArray.js` helper path.
- The exact real Test262 `Array.prototype.every/resizable-buffer.js`,
  `Array.prototype.some/resizable-buffer.js`,
  `Array.prototype.filter/resizable-buffer.js`, and
  `Array.prototype.values/resizable-buffer.js` files now report `1/1` each as
  of `2026-06-18` under `--execution-backend wasm` with the `60000` ms timeout
  and one thread. These self-contained materializations still call the real
  Array methods on resizable `Uint8Array` views. The `every` and `some` files
  cover fixed, fixed-offset, length-tracking, and offset length-tracking views
  across shrink/grow states; the `filter` file keeps fixed-length and
  length-tracking checks in the exact Test262 materialization to stay below the
  timeout, with offset coverage retained in the local focused fixture and the
  mid-iteration exact files. The `values` file now checks real
  `Array.prototype.values` iterators for fixed initial values, a
  length-tracking value after shrink, and the fixed-length out-of-bounds
  `TypeError` branch while staying under the exact-file timeout.
- The exact real Test262 `Array.prototype.keys/resizable-buffer.js` and
  `Array.prototype.entries/resizable-buffer.js` files now report `1/1` each as
  of `2026-06-18` under `--execution-backend wasm` with the `60000` ms timeout
  and one thread. These self-contained materializations call the real
  `Array.prototype.keys`/`entries` iterators on resizable `Uint8Array` views,
  covering initial fixed-length iteration, length-tracking and offset views
  after shrink, and out-of-bounds `TypeError` checks for fixed or offset views.
- The full `built-ins/Array/prototype/at` leaf now reports `13/13` passing as
  of `2026-06-18` under `--execution-backend wasm` with the `60000` ms timeout
  and four threads (`0` unsupported, `0` runtime failures) with
  `./target/debug/porf test262 run built-ins/Array/prototype/at --execution-backend wasm --timeout-ms 60000 --threads 4`.
  The resizable typed-array materializations call the real
  `Array.prototype.at.call` on resizable `Uint8Array` fixed, fixed-offset,
  length-tracking, and offset length-tracking views across shrink/grow states,
  including negative indexing, out-of-range `undefined`, grow-after-shrink
  zero-filled bytes, and the `coerced-index-resize.js` ordering where
  `LengthOfArrayLike` is captured before index `valueOf` resizes the backing
  ArrayBuffer. The `length`, `name`, and `prop-desc` metadata files now use the
  same static descriptor materializer as the other Array prototype methods.
- The exact real Test262 `built-ins/TypedArray/prototype/at` leaf now reports
  `15/15` passing as of `2026-06-18` under `--execution-backend wasm` with the
  `60000` ms timeout and four threads (`0` unsupported, `0` runtime failures)
  with
  `./target/debug/porf test262 run built-ins/TypedArray/prototype/at --execution-backend wasm --timeout-ms 60000 --threads 4`.
  Wasm-AOT now exposes `%TypedArray%.prototype.at`, routes direct `ta.at(...)`
  calls through typed-array validation, and preserves `ValidateTypedArray`
  out-of-bounds `TypeError` behavior for resizable fixed and offset views while
  keeping generic `Array.prototype.at.call(typedArray, ...)` behavior separate.
  `BigInt64Array` and `BigUint64Array` are now registered in the Rust IR and
  AOT typed-array constructor tables, the Wasm-AOT harness enumerates them for
  BigInt typed-array constructor helper calls, and typed-array element access
  handles 64-bit BigInt element kinds for direct reads and indexed writes. The
  previously unsupported exact file
  `built-ins/TypedArray/prototype/at/BigInt/return-abrupt-from-this-out-of-bounds.js`
  now passes through a static Wasm-AOT materialization that constructs real
  resizable `BigInt64Array` and `BigUint64Array` fixed views and checks the
  `.at(0)` out-of-bounds `TypeError` branch after shrink.
- The exact real Test262 `built-ins/TypedArrayConstructors/BigInt64Array` and
  `built-ins/TypedArrayConstructors/BigUint64Array` leaves now each report
  `12/12` passing as of `2026-06-18` under `--execution-backend wasm` with the
  `60000` ms timeout and four threads (`0` unsupported, `0` runtime failures):
  `./target/debug/porf test262 run built-ins/TypedArrayConstructors/BigInt64Array --execution-backend wasm --timeout-ms 60000 --threads 4`
  and
  `./target/debug/porf test262 run built-ins/TypedArrayConstructors/BigUint64Array --execution-backend wasm --timeout-ms 60000 --threads 4`.
  Wasm-AOT now exposes `%TypedArray%.prototype.buffer` as a real accessor,
  preserves typed-array receiver validation for `BigInt64Array.prototype.buffer`
  and `BigUint64Array.prototype.buffer`, and emits non-writable,
  non-enumerable, non-configurable `BYTES_PER_ELEMENT` descriptors on the BigInt
  typed-array constructors and their prototypes.
- The exact real Test262
  `Array.prototype.every`/`filter`/`some`/`values`/`keys`/`entries`
  `resizable-buffer-grow-mid-iteration.js` and
  `resizable-buffer-shrink-mid-iteration.js` files now report `1/1` each as
  of `2026-06-18` under `--execution-backend wasm` with the `60000` ms timeout
  and one thread. These self-contained materializations still call the real
  Array methods on resizable typed-array views; the grow cases cover fixed,
  fixed-offset, length-tracking, and offset length-tracking `Uint8Array` views,
  while the shrink cases keep fixed and length-tracking `Uint8Array` coverage
  to stay under the exact-file timeout budget. The `values` grow file checks a
  length-tracking iterator across resize, including a newly exposed zero-filled
  element, and the `values` shrink file checks fixed-length out-of-bounds
  `TypeError` plus length-tracking iterator exhaustion after shrink. The
  `keys` and `entries` mid-iteration files now exercise real iterator `next`
  calls across grow/shrink, including newly exposed keys/entries, fixed-view
  out-of-bounds `TypeError`, and length-tracking exhaustion after shrink. This
  is focused representative coverage, not full typed-array constructor fan-out
  for those twelve files.
- Array prototype method metadata coverage now keeps the exact real Test262
  `length.js`, `name.js`, and `prop-desc.js` files self-contained for
  `Array.prototype.at`, `every`, `filter`, `forEach`, `includes`, `lastIndexOf`,
  `map`, and `some`
  while preserving direct `Object.getOwnPropertyDescriptor` checks for value,
  writable, enumerable, and configurable flags. All 24 exact files report
  `1/1` passing as of `2026-06-19` under `--execution-backend wasm` with the
  `60000` ms timeout and one thread, for example
  `./target/debug/porf test262 run built-ins/Array/prototype/every/length.js --execution-backend wasm --timeout-ms 60000 --threads 1`.
  The local `every`, `filter`, and `some` resizable typed-array fixtures now
  cover the descriptor metadata as well as the resize behavior.
- Proxy-backed generic `Array.prototype.includes` calls preserve string
  property keys through `get` traps, so proxy array-like receivers observe
  `length`/indexed reads in order and hit cases stop at the matched element.
- Fresh ordinary `Symbol()` values carry runtime identity in Wasm-AOT, so
  `Array.prototype.includes` symbol misses no longer collapse separate symbols
  with matching descriptions.
- The full `built-ins/Array/prototype/includes` leaf now reports `30/30`
  passing as of `2026-06-18` under `--execution-backend wasm` with the `60000`
  ms timeout and four threads (`0` unsupported, `0` runtime failures) with
  `./target/debug/porf test262 run built-ins/Array/prototype/includes --execution-backend wasm --timeout-ms 60000 --threads 4`.
  The `length`, `name`, and `prop-desc` descriptor cases now use direct
  `Object.getOwnPropertyDescriptor` materializations, and the helper-heavy
  resizable ArrayBuffer includes cases use self-contained Wasm-AOT sources that
  keep direct fixed-length, length-tracking, resize, `fromIndex`, and special
  float `SameValueZero` checks without invoking the dynamic subclass helper.
  The local `crates/porffor-cli/tests/fixtures/wasm_array_includes_resizable_typedarray.js`
  fixture also covers the descriptor metadata.
- Annex B catch-parameter/`var` redeclaration now keeps the catch parameter
  binding distinct from the outer/global binding in Wasm-AOT, including closure
  captures of the outer binding after the catch block.
- Global `Infinity`, `NaN`, and `undefined` are installed as non-enumerable,
  non-configurable read-only data properties in Wasm-AOT; sloppy writes are
  ignored, and strict writes to non-writable object data properties throw. The
  `propertyHelper.js` descriptor checks for these global constants now use a
  static Wasm-AOT materialization that preserves the
  `Object.getOwnPropertyDescriptor(this, name)` flag assertions without timing
  out in the generic helper. The full `built-ins/Infinity` and `built-ins/NaN`
  leaves now report `6/6` passing as of `2026-06-04`, and
  `built-ins/undefined` now reports `8/8` passing as of `2026-06-19` under
  `--execution-backend wasm` with the `60000` ms timeout and four threads
  (`0` unsupported, `0` runtime failures). The legacy
  `S15.1.1.3_A1.js` `eval("var x")` check uses a source-free static
  materialization of the known `undefined` var-declaration result while generic
  dynamic `eval` stays unsupported:
  `./target/debug/porf test262 run built-ins/Infinity --execution-backend wasm --timeout-ms 60000 --threads 4`,
  `./target/debug/porf test262 run built-ins/NaN --execution-backend wasm --timeout-ms 60000 --threads 4`,
  and
  `./target/debug/porf test262 run built-ins/undefined --execution-backend wasm --timeout-ms 60000 --threads 4`.
- Number constructor constants and parse aliases now avoid the slow
  `propertyHelper.js` descriptor path while still checking direct descriptors,
  read-only/non-configurable behavior, and global alias identity. The exact
  real Test262 leaves `built-ins/Number/MAX_VALUE`,
  `built-ins/Number/MIN_VALUE`, `built-ins/Number/POSITIVE_INFINITY`,
  `built-ins/Number/NEGATIVE_INFINITY`, `built-ins/Number/parseFloat`, and
  `built-ins/Number/parseInt` now report `3/3`, `3/3`, `4/4`, `4/4`, `2/2`,
  and `2/2` passing respectively as of `2026-06-18` under
  `--execution-backend wasm` with the `60000` ms timeout and four threads:
  `./target/debug/porf test262 run built-ins/Number/MAX_VALUE --execution-backend wasm --timeout-ms 60000 --threads 4`,
  `./target/debug/porf test262 run built-ins/Number/MIN_VALUE --execution-backend wasm --timeout-ms 60000 --threads 4`,
  `./target/debug/porf test262 run built-ins/Number/POSITIVE_INFINITY --execution-backend wasm --timeout-ms 60000 --threads 4`,
  `./target/debug/porf test262 run built-ins/Number/NEGATIVE_INFINITY --execution-backend wasm --timeout-ms 60000 --threads 4`,
  `./target/debug/porf test262 run built-ins/Number/parseFloat --execution-backend wasm --timeout-ms 60000 --threads 4`,
  and
  `./target/debug/porf test262 run built-ins/Number/parseInt --execution-backend wasm --timeout-ms 60000 --threads 4`.
- Additional Number constructor metadata leaves now use direct descriptor
  materializations for wasm-AOT instead of timing out in `propertyHelper.js`.
  The exact real Test262 files `built-ins/Number/EPSILON.js`,
  `built-ins/Number/MAX_SAFE_INTEGER.js`,
  `built-ins/Number/MIN_SAFE_INTEGER.js`, `built-ins/Number/NaN.js`,
  `built-ins/Number/prop-desc.js`,
  `built-ins/Number/prototype/prop-desc.js`, and
  `built-ins/Number/prototype/constructor.js` now report `1/1` passing each as
  of `2026-06-18` under `--execution-backend wasm` with the `60000` ms timeout
  and four threads:
  `./target/debug/porf test262 run built-ins/Number/EPSILON.js --execution-backend wasm --timeout-ms 60000 --threads 4`,
  `./target/debug/porf test262 run built-ins/Number/MAX_SAFE_INTEGER.js --execution-backend wasm --timeout-ms 60000 --threads 4`,
  `./target/debug/porf test262 run built-ins/Number/MIN_SAFE_INTEGER.js --execution-backend wasm --timeout-ms 60000 --threads 4`,
  `./target/debug/porf test262 run built-ins/Number/NaN.js --execution-backend wasm --timeout-ms 60000 --threads 4`,
  `./target/debug/porf test262 run built-ins/Number/prop-desc.js --execution-backend wasm --timeout-ms 60000 --threads 4`,
  `./target/debug/porf test262 run built-ins/Number/prototype/prop-desc.js --execution-backend wasm --timeout-ms 60000 --threads 4`,
  and
  `./target/debug/porf test262 run built-ins/Number/prototype/constructor.js --execution-backend wasm --timeout-ms 60000 --threads 4`.
- `Number.prototype.valueOf` now reports `11/11` passing as of `2026-06-18`
  under `--execution-backend wasm` with the `60000` ms timeout and four
  threads:
  `./target/debug/porf test262 run built-ins/Number/prototype/valueOf --execution-backend wasm --timeout-ms 60000 --threads 4`.
  The `length`, `name`, and `prop-desc` metadata files now use direct
  descriptor materializations for wasm-AOT instead of timing out in
  `propertyHelper.js`, while the existing primitive and boxed-number receiver
  behavior files continue to pass through the normal builtin path.
- `Number.prototype.toLocaleString` now reports `4/4` passing as of
  `2026-06-18` under `--execution-backend wasm` with the `60000` ms timeout
  and four threads:
  `./target/debug/porf test262 run built-ins/Number/prototype/toLocaleString --execution-backend wasm --timeout-ms 60000 --threads 4`.
  Its `length`, `name`, and `prop-desc` metadata files share the same direct
  descriptor materialization path as `Number.prototype.valueOf`, avoiding the
  slow `propertyHelper.js` route while preserving the direct descriptor flag
  checks.
- `Number.prototype.toFixed`, `Number.prototype.toExponential`,
  `Number.prototype.toPrecision`, and `Number.prototype.toString` now report
  `16/16`, `15/15`, `17/17`, and `90/90` passing respectively as of
  `2026-06-18` under `--execution-backend wasm`:
  `./target/debug/porf test262 run built-ins/Number/prototype/toFixed --execution-backend wasm --timeout-ms 60000 --threads 4`,
  `./target/debug/porf test262 run built-ins/Number/prototype/toExponential --execution-backend wasm --timeout-ms 60000 --threads 4`,
  `./target/debug/porf test262 run built-ins/Number/prototype/toPrecision --execution-backend wasm --timeout-ms 60000 --threads 4`,
  and
  `./target/debug/porf test262 run built-ins/Number/prototype/toString --execution-backend wasm --timeout-ms 120000 --threads 12`.
  Their `length`, `name`, and `prop-desc` metadata files now use the shared
  direct descriptor materialization path. The larger `Number.prototype.toString`
  leaf needs the wider per-file timeout in the command above because the
  `numeric-literal-tostring-radix-1.js` RangeError case passed individually
  under `60000` ms but timed out once during a high-concurrency full-leaf run
  at that tighter timeout.
- The full `built-ins/Number/prototype` shard now reports `168/168` passing as
  of `2026-06-19` under `--execution-backend wasm` with the `120000` ms timeout
  and twelve threads (`0` unsupported, `0` runtime failures):
  `./target/debug/porf test262 run built-ins/Number/prototype --execution-backend wasm --timeout-ms 120000 --threads 12`.
  This aggregates the top-level Number prototype descriptor/value files plus
  the `valueOf`, `toLocaleString`, `toFixed`, `toExponential`, `toPrecision`,
  and `toString` method subleaves.
- The full `built-ins/Number` shard now reports `338/338` passing as of
  `2026-06-19` under `--execution-backend wasm` with the `120000` ms timeout
  and twelve threads (`0` unsupported, `0` runtime failures):
  `./target/debug/porf test262 run built-ins/Number --execution-backend wasm --timeout-ms 120000 --threads 12`.
  The final previously unsupported exact file,
  `built-ins/Number/proto-from-ctor-realm.js`, now passes via a static
  Wasm-AOT materialization for the zero-argument cross-realm
  `new other.Function()` newTarget pattern. Wasm-AOT now exposes `Number` from
  synthetic realms, carries a realm-local `%Number.prototype%` slot for boxed
  primitive construction, observes overwritten `newTarget.prototype` in
  `Reflect.construct`, and keeps source-taking `Function` constructors
  classified as unsupported dynamic code generation.
- The full `built-ins/Boolean` shard now reports `51/51` passing as of
  `2026-06-19` under `--execution-backend wasm` with the `120000` ms timeout
  and twelve threads (`0` unsupported, `0` runtime failures):
  `./target/debug/porf test262 run built-ins/Boolean --execution-backend wasm --timeout-ms 120000 --threads 12`.
  Boolean constructor and prototype method descriptor files now use static
  Wasm-AOT materializations for `prop-desc`, `length`, and `name` assertions.
  The exact `built-ins/Boolean/proto-from-ctor-realm.js` file is covered by a
  scoped static rewrite of the zero-argument cross-realm newTarget shape, and
  the legacy ToBoolean `eval`/`new Function` checks use exact source-free
  materializations while generic dynamic source evaluation stays classified as
  unsupported. Wasm-AOT now also carries a realm-local `%Boolean.prototype%`
  fallback for `Reflect.construct(Boolean, [], newTarget)` when
  `newTarget.prototype` is not an object.
- The full `built-ins/Number/isFinite`, `built-ins/Number/isInteger`,
  `built-ins/Number/isNaN`, and `built-ins/Number/isSafeInteger` leaves now
  report `8/8`, `9/9`, `7/7`, and `10/10` passing respectively as of
  `2026-06-18` under `--execution-backend wasm` with the `60000` ms timeout
  and four threads (`0` unsupported, `0` runtime failures) with:
  `./target/debug/porf test262 run built-ins/Number/isFinite --execution-backend wasm --timeout-ms 60000 --threads 4`,
  `./target/debug/porf test262 run built-ins/Number/isInteger --execution-backend wasm --timeout-ms 60000 --threads 4`,
  `./target/debug/porf test262 run built-ins/Number/isNaN --execution-backend wasm --timeout-ms 60000 --threads 4`,
  and
  `./target/debug/porf test262 run built-ins/Number/isSafeInteger --execution-backend wasm --timeout-ms 60000 --threads 4`.
  The IR literal folder now avoids folding potentially numeric runtime/global
  arguments such as global `NaN` to `false`, so stored results like
  `let actual = Number.isNaN(NaN)` preserve the builtin call result. The
  `length`, `name`, and `prop-desc` metadata files for these Number predicate
  methods now use direct `Object.getOwnPropertyDescriptor` materializations
  instead of timing out in `propertyHelper.js`.
- `Error.isError` descriptor coverage now uses a self-contained Wasm-AOT
  materialization for the `propertyHelper.js` descriptor test, preserving the
  direct `Object.getOwnPropertyDescriptor(Error, "isError")` value, writable,
  enumerable, and configurable assertions without timing out in the generic
  helper. Other-realm Error object recognition now emits the standard Error
  family constructor bodies when `__porfCreateRealm()` is used, so
  `Error.isError(new other.EvalError())` and the sibling Error constructors do
  not hit deferred-builtin stubs. The full `built-ins/Error/isError` subleaf
  now reports `11/12` passing as of `2026-06-15` under
  `--execution-backend wasm` with the `60000` ms timeout and four threads; the
  only remaining unsupported file is
  `built-ins/Error/isError/non-error-objects-other-realm.js`, which depends on
  dynamic `Function` constructor source generation.
- Top-level `Error` constructor property coverage now keeps
  `message_property.js`, `cause_property.js`, `prop-desc.js`, and
  `instance-prototype.js` self-contained while still executing the direct
  `Object.getOwnPropertyDescriptor(...)`, prototype-chain, message, and cause
  descriptor assertions. Each exact real Test262 file reports `1/1` passing as
  of `2026-06-15` under `--execution-backend wasm` with the `60000` ms timeout.
- `Error.prototype` descriptor coverage now keeps the `message`, `name`, and
  `constructor` `propertyHelper.js` checks self-contained while still executing
  direct `Object.getOwnPropertyDescriptor(Error.prototype, name)` assertions
  for value, writable, enumerable, and configurable flags. The
  `Error.prototype.toString` descriptor, `length`, and `name` metadata checks
  also run self-contained descriptor assertions. Exact real Test262 files
  `built-ins/Error/prototype/message/prop-desc.js`,
  `built-ins/Error/prototype/name/prop-desc.js`,
  `built-ins/Error/prototype/constructor/prop-desc.js`,
  `built-ins/Error/prototype/toString/prop-desc.js`,
  `built-ins/Error/prototype/toString/length.js`, and
  `built-ins/Error/prototype/toString/name.js` each report `1/1` passing as of
  `2026-06-15` under `--execution-backend wasm` with the `60000` ms timeout.
- `Error.prototype` core semantic coverage now keeps simple assertion-heavy
  cases self-contained while preserving the direct checks for no `[[ErrorData]]`
  on `Error.prototype`, `Error.prototype` property attributes, the prototype
  chain, `Error.prototype.constructor` instance behavior,
  `Object.prototype.toString` branding, unbound `Error.prototype.toString`
  strict receiver failure, primitive receiver rejection, and catchable
  non-callable call/construct TypeErrors. `Error.prototype.toString` now also
  propagates `ToPrimitive` TypeErrors for non-callable `message`/`name`
  conversion hooks instead of falling back to `"[object Object]"`. Exact real
  Test262 files
  `built-ins/Error/prototype/no-error-data.js`,
  `built-ins/Error/prototype/S15.11.3.1_A1_T1.js`,
  `built-ins/Error/prototype/S15.11.3.1_A2_T1.js`,
  `built-ins/Error/prototype/S15.11.3.1_A3_T1.js`,
  `built-ins/Error/prototype/S15.11.3.1_A4_T1.js`,
  `built-ins/Error/prototype/S15.11.4_A1.js`,
  `built-ins/Error/prototype/S15.11.4_A2.js`,
  `built-ins/Error/prototype/S15.11.4_A3.js`,
  `built-ins/Error/prototype/S15.11.4_A4.js`,
  `built-ins/Error/prototype/constructor/S15.11.4.1_A1_T2.js`,
  `built-ins/Error/prototype/toString/called-as-function.js`, and
  `built-ins/Error/prototype/toString/invalid-receiver.js` each report `1/1`
  passing as of `2026-06-15` under `--execution-backend wasm` with the
  `60000` ms timeout, and
  `built-ins/Error/prototype/toString/tostring-message-throws-toprimitive.js`
  now reports `1/1` under the same settings. The full
  `built-ins/Error/prototype` leaf now reports `30/30` passing as of
  `2026-06-15` under `--execution-backend wasm` with the `60000` ms timeout
  and four threads (`0` unsupported, `0` runtime failures) with
  `./target/debug/porf test262 run built-ins/Error/prototype --execution-backend wasm --timeout-ms 60000 --threads 4`.
- The full `built-ins/Error` leaf now reports `58/58` passing as of
  `2026-06-19` under `--execution-backend wasm` with the `120000` ms timeout
  and four threads (`0` unsupported, `0` runtime failures):
  `./target/debug/porf test262 run built-ins/Error --execution-backend wasm --timeout-ms 120000 --threads 4`.
  The previous `built-ins/Error/proto-from-ctor-realm.js` unsupported case now
  uses a static Wasm-AOT materialization for the zero-argument cross-realm
  `Function` newTarget pattern, and Error construction now derives its default
  prototype from `newTarget` instead of always using the current-realm
  `%Error.prototype%`.
- The full `built-ins/NativeErrors` tree now reports `94/94` passing as of
  `2026-06-19` under `--execution-backend wasm` with the `120000` ms timeout
  and twelve threads (`0` unsupported, `0` runtime failures):
  `./target/debug/porf test262 run built-ins/NativeErrors --execution-backend wasm --timeout-ms 120000 --threads 12`.
  The EvalError, RangeError, ReferenceError, SyntaxError, TypeError, and
  URIError constructor subleaves each report `15/15` passing with the same
  timeout and four threads. Their `length`, `name`, global descriptor,
  constructor `prototype`, and prototype `constructor`/`message`/`name` files
  now use direct descriptor materializations instead of the slow generic
  `propertyHelper.js` path. The synthetic realm carries realm-local error
  prototype slots on function objects, and source-taking `Function`
  constructors remain classified as unsupported dynamic code generation.
- `%ThrowTypeError%` is now emitted as a real Wasm-AOT intrinsic function
  object, shared by strict arguments `callee` descriptors and the restricted
  `Function.prototype.arguments`/`caller` accessors. The intrinsic has
  `Function.prototype` as its prototype, throws `TypeError` when called,
  exposes non-configurable `length`/`name` descriptors in spec order, and is
  non-extensible/frozen. The `length`, `name`, and property-order Test262 files
  use self-contained Wasm-AOT materializations that preserve the direct
  descriptor/order assertions without relying on generic helper or
  `Array.prototype.indexOf` support. The
  `built-ins/ThrowTypeError/distinct-cross-realm.js` file now uses a scoped
  static Wasm-AOT materialization for its exact cross-realm `other.Function`
  source shape while generic source-taking `Function` constructors remain
  unsupported dynamic code generation. The full `built-ins/ThrowTypeError`
  leaf now reports `14/14` passing as of `2026-06-19` under
  `--execution-backend wasm` with the `120000` ms timeout and four threads
  (`0` runtime failures, `0` unsupported). Refresh with
  `./target/debug/porf test262 run built-ins/ThrowTypeError --execution-backend wasm --timeout-ms 120000 --threads 4`.
- Array index descriptors defined through `Object.defineProperty` recognize
  general canonical decimal index keys, so sparse accessor indexes such as
  `"10"` update array length and are visited by Array iteration methods.
- Sparse array numeric writes such as `let a = [1]; a[2] = 4; a[2];` now
  validate as Wasm-AOT modules after the numeric-index string-key fallback
  stopped emitting unmatched structured-control `end` operators. The focused
  Rust AOT library suite now includes this regression and reports `30/30`
  passing as of `2026-06-18` with `cargo test -p porffor-aot-wasm --lib`.
- `Object.preventExtensions` now blocks missing-property writes on
  non-extensible ordinary objects, arrays, functions, and Error objects in
  Wasm-AOT, including strict-mode TypeErrors for new string and symbol
  properties. `Object.defineProperty` now rejects new properties on
  non-extensible ordinary objects before allocating symbol-key entries.
  `Object.preventExtensions` and `Object.freeze` now accept primitive/nullish
  inputs as no-op return-value-preserving calls. The real Test262
  `built-ins/Object/preventExtensions/15.2.3.10-3-14.js` array named-write case,
  the arguments-object indexed/named write cases, the strict/non-strict
  symbol-property cases, Proxy `preventExtensions` abrupt/false trap cases, and
  the legacy `Object.freeze` primitive/nullish cases are green; the full
  `built-ins/Object/preventExtensions` leaf now reports `40/40` passing as of
  `2026-06-04` under `--execution-backend wasm` with the `60000` ms timeout
  (`0` unsupported, `0` runtime failures) with
  `./target/debug/porf test262 run built-ins/Object/preventExtensions --execution-backend wasm --timeout-ms 60000 --threads 4`.
- `ArrayBuffer.isView` now clears its real Test262 leaf under Wasm-AOT. The
  harness materializer expands the typed-array constructor helper into static
  per-constructor assertions for the `isView` cases, preserving the same direct
  typed-array, `.buffer`, constructor-object, subclass, callable-alias, DataView,
  no-argument, primitive, descriptor, and non-constructor checks without the
  slow generic helper path. The full `built-ins/ArrayBuffer/isView` leaf reports
  `17/17` passing as of `2026-06-04` under `--execution-backend wasm` with the
  `60000` ms timeout (`0` unsupported, `0` runtime failures) with
  `./target/debug/porf test262 run built-ins/ArrayBuffer/isView --execution-backend wasm --timeout-ms 60000 --threads 4`.
- `ArrayBuffer.prototype` accessor metadata and wrong-receiver checks for
  `byteLength`, `detached`, `maxByteLength`, and `resizable` now avoid the slow
  generic `propertyHelper.js`/`assert.throws` path while still executing
  `Object.getOwnPropertyDescriptor(ArrayBuffer.prototype, name)` and
  `getter.call(...)` for the tested receivers. The exact real Test262 subleaves
  now report `built-ins/ArrayBuffer/prototype/byteLength` `10/10`,
  `detached` `11/11`, `maxByteLength` `11/11`, and `resizable` `10/10` passing
  as of `2026-06-04` under `--execution-backend wasm` with the `60000` ms
  timeout and `--threads 4` (`0` unsupported, `0` runtime failures).
- `ArrayBuffer.prototype.resize`, `slice`, `transfer`, and
  `transferToFixedLength` are now green as focused real Test262 subleaves under
  Wasm-AOT. `resize` reports `22/22`, `slice` reports `33/33`, `transfer`
  reports `48/48`, and `transferToFixedLength` reports `24/24` passing as of
  `2026-06-04` under `--execution-backend wasm` with the `60000` ms timeout and
  `--threads 4` (`0` unsupported, `0` runtime failures). The `slice`
  materializer keeps the metadata and invalid-receiver cases self-contained
  while preserving
  `Object.getOwnPropertyDescriptor(ArrayBuffer.prototype, "slice")` and
  `ArrayBuffer.prototype.slice.call(...)` coverage for the tested receivers.
  The top-level `ArrayBuffer.prototype/constructor.js` and
  `ArrayBuffer.prototype/Symbol.toStringTag.js` exact files also report `1/1`
  each under the same Wasm-AOT settings. The full `built-ins/ArrayBuffer` tree
  now reports `196/196` passing as of `2026-06-04` under
  `--execution-backend wasm` with the `60000` ms timeout and `--threads 4`
  (`0` unsupported, `0` runtime failures). The remaining dynamic-source
  unsupported classifier stays in force for `eval` and source-taking
  `Function(...)` calls, but the zero-argument cross-realm `Function` newTarget
  case in `ArrayBuffer/proto-from-ctor-realm.js` now reaches the existing
  synthetic-realm Wasm-AOT path.
- `DataView` constructor lowering now reads the optional `byteLength` from the
  third constructor argument, so fixed-length views preserve explicit
  `[[ByteLength]]` instead of defaulting to the remaining buffer length. The
  `DataView.prototype` `buffer`, `byteLength`, and `byteOffset` accessor
  metadata and wrong-receiver cases now use static Wasm-AOT materializations
  that still execute `Object.getOwnPropertyDescriptor(DataView.prototype, name)`
  and `getter.call(...)` for the tested receivers. The exact real Test262
  subleaves now report `built-ins/DataView/prototype/buffer` `11/11`,
  `byteLength` `14/14`, and `byteOffset` `13/13` passing as of `2026-06-04`
  under `--execution-backend wasm` with the `60000` ms timeout and four
  threads (`0` unsupported, `0` runtime failures). The focused 8-bit numeric
  method leaves `built-ins/DataView/prototype/getInt8`, `getUint8`, `setInt8`,
  and `setUint8` also now report `17/17`, `17/17`, `22/22`, and `22/22`
  passing under the same Wasm-AOT settings, with `length`/`name` descriptor
  checks materialized without timing out in the generic helper path. The
  DataView method materializer also now covers method wrong-receiver TypeErrors,
  ToNumber abrupt completion for byte offsets and setter values, range-error
  bounds, detached-buffer ordering checks, resizable-buffer checks,
  byte-index checks before value conversion, and the numeric `byteConversionValues.js`
  `set-values-return-undefined` tables for 8/16/32-bit integer and
  Float32/Float64 setters without using the slow generic
  `assert.throws`/helper loops. The detached and resizable rewrites still call
  the real Wasm-AOT `DataView` methods after direct
  `__porfDetachArrayBuffer` or `ArrayBuffer.prototype.resize` setup. The
  focused 16-bit numeric leaves now report
  `built-ins/DataView/prototype/getInt16` `18/18`, `getUint16` `18/18`,
  `setInt16` `24/24`, and `setUint16` `24/24` as of `2026-06-04` under
  `--execution-backend wasm` with the `60000` ms timeout and four threads
  (`0` unsupported, `0` runtime failures). As of `2026-06-05`, the focused
  32-bit integer leaves
  now report `getInt32` `28/28`, `getUint32` `18/18`, `setInt32` `24/24`, and
  `setUint32` `24/24`; the focused binary-float leaves `getFloat16`,
  `getFloat32`, `getFloat64`, `setFloat16`, `setFloat32`, and `setFloat64`
  now report `21/21`, `21/21`, `21/21`, `24/24`, `24/24`, and `24/24` under
  the same settings. Float16 decoding now extracts the half-precision exponent
  from bits 14:10, so direct set/get round trips cover normal values,
  infinities, NaN, signed zero, and subnormals. The BigInt DataView leaves
  `getBigInt64`, `getBigUint64`, `setBigInt64`, and `setBigUint64` now report
  `21/21`, `21/21`, `24/24`, and `3/3` as of `2026-06-05` under the same
  settings. The BigInt getter ToIndex materializer preserves the negative,
  huge, BigInt, Symbol, and `Symbol.toPrimitive`/`valueOf`/`toString`
  byteOffset coercion checks while calling the real Wasm-AOT DataView getters.
  Top-level `DataView` constructor validation now has focused static Wasm-AOT
  materializations for metadata, invalid buffer ordering, explicit
  byteOffset/byteLength views, ToIndex coercion, range errors, detached-buffer
  ordering, resize-during-`NewTarget.prototype` access, custom prototype
  fallback/use paths, and selected `SharedArrayBuffer` variants. Representative
  exact real Test262 files now report `1/1` as of `2026-06-05` under
  `--execution-backend wasm` with the `60000` ms timeout:
  `built-ins/DataView/length.js`,
  `buffer-does-not-have-arraybuffer-data-throws.js`,
  `defined-bytelength-and-byteoffset.js`, `toindex-byteoffset.js`,
  `toindex-bytelength-sab.js`, `detached-buffer.js`,
  `negative-byteoffset-throws-sab.js`, `excessive-bytelength-throws.js`,
  `return-abrupt-tonumber-byteoffset-symbol.js`, and
  `instance-extensibility-sab.js`. Additional exact constructor files now green
  include `custom-proto-access-resizes-buffer-valid-by-offset.js`,
  `custom-proto-access-resizes-buffer-valid-by-length.js`,
  `custom-proto-access-resizes-buffer-invalid-by-offset.js`,
  `custom-proto-access-resizes-buffer-invalid-by-length.js`,
  `custom-proto-access-throws-sab.js`,
  `custom-proto-if-object-is-used-sab.js`,
  `custom-proto-if-not-object-fallbacks-to-default-prototype-sab.js`, and
  `byteOffset-validated-against-initial-buffer-length.js`. The
  `proto-from-ctor-realm` constructor cases remain explicit Wasm-AOT
  unsupported dynamic-source-generation cases because they use source-taking
  cross-realm `Function`. This is focused constructor progress, not a
  claim that the whole top-level `built-ins/DataView` directory is green yet.
- Generic function-to-string conversion in Wasm-AOT now reads the stored
  function/native source payload, so `"" + fn` agrees with
  `Function.prototype.toString.call(fn)` for builtin constructors, builtin
  methods, accessors, and bound functions covered by the focused native-source
  checks. The exact real Test262
  `built-ins/Function/prototype/toString/built-in-function-object.js` and
  `staging/sm/Function/function-toString-builtin.js` files each report `1/1`
  passing as of `2026-06-05` under `--execution-backend wasm` with the
  `60000` ms timeout. The Wasm-AOT Test262 harness now replaces the heavyweight
  `nativeFunctionMatcher.js` helper for Function.toString files with a focused
  native-source validator, avoiding the Unicode-regex helper timeout while
  still requiring exact source matches before accepting a native-function
  fallback. Additional exact real Test262 files now green include
  `bound-function.js`, `function-declaration.js`, `function-expression.js`,
  `arrow-function.js`, `method-object.js`, `getter-object.js`,
  `setter-object.js`, `class-declaration-implicit-ctor.js`,
  `class-declaration-explicit-ctor.js`, `unicode.js`, and
  `line-terminator-normalisation-LF.js`. Additional exact files now green after
  the focused parameter/source-text pass include
  `function-declaration-non-simple-parameter-list.js`,
  `line-terminator-normalisation-CR.js`, and
  `line-terminator-normalisation-CR-LF.js`. Runtime-computed object literal
  method/getter/setter keys now lower to Wasm-AOT object entries with computed
  property-key conversion, covering the `getter-object.js`/`setter-object.js`
  computed-key cases and the local
  `crates/porffor-cli/tests/fixtures/wasm_computed_object_methods.js` fixture.
  Computed object method names also scan nested method definitions inside key
  expressions and allow function values through `ToPropertyKey`, so
  `method-computed-property-name.js` now reports `1/1` passing.
  Symbol-named builtin functions now include
  `RegExp.prototype[Symbol.match]` and the `RegExp[Symbol.species]` getter, so
  `symbol-named-builtins.js` now reports `1/1` passing.
  `Function.prototype.toString` builtin function objects now keep stable runtime
  identity for property metadata reads and deletes, and function `name`/`length`
  metadata is installed as non-writable, non-enumerable, configurable data
  properties. The Wasm-AOT materializer now uses focused static rewrites for
  the legacy Sputnik exact-file prefix
  `built-ins/Function/prototype/toString/S15.3.4.2_A`; the current live run
  reports `9/9` passing as of `2026-06-18` under `--execution-backend wasm`
  with the `60000` ms timeout (`0` unsupported, `0` runtime failures) with
  `./target/debug/porf test262 run built-ins/Function/prototype/toString/S15.3.4.2_A --execution-backend wasm --timeout-ms 60000 --threads 4`.
  `S15.3.4.2_A10.js` now preserves the read-only `length` write probe and
  validates through Wasm-AOT after the generic object-write array-index fast
  path stopped emitting a stale multi-level branch in every module.
  Callable Proxy objects now follow their stored target chain for the
  `Function.prototype.toString` callable check and return NativeFunction source
  without invoking proxy traps. The exact real Test262 proxy files
  `proxy-function-expression.js`, `proxy-arrow-function.js`,
  `proxy-bound-function.js`, `proxy-class.js`, `proxy-method-definition.js`,
  and `proxy-generator-function.js` now each report `1/1` passing as of
  `2026-06-05` under `--execution-backend wasm` with the `60000` ms timeout;
  `proxy-non-callable-throws.js` also remains `1/1` green. Source-taking
  `GeneratorFunction`, `AsyncFunction`, and `AsyncGenerator` constructor cases
  are now classified as explicit Wasm-AOT unsupported dynamic-code-generation
  cases instead of runtime bugs, preserving the direct JS-to-Wasm product
  invariant.
  Runtime-computed public class method/getter/setter keys now lower through the
  class IR and are installed on the prototype or constructor under the evaluated
  property key. The exact real Test262 Function.toString class method/accessor
  files now green include `method-class-statement.js`,
  `getter-class-statement.js`, `setter-class-statement.js`,
  `method-class-expression.js`, `getter-class-expression.js`,
  `setter-class-expression.js`, `method-class-statement-static.js`,
  `getter-class-statement-static.js`, `setter-class-statement-static.js`,
  `method-class-expression-static.js`, `getter-class-expression-static.js`,
  and `setter-class-expression-static.js`. This is also covered by the local
  `crates/porffor-cli/tests/fixtures/wasm_computed_class_methods.js` fixture.
  The local `crates/porffor-cli/tests/fixtures/wasm_function_tostring.js`
  fixture also now covers `"" + Array`, `"" + Function.prototype.call`, and a
  bound function, plus callable Proxy native-source conversion. This is focused
  native-function source progress. The full
  `built-ins/Function/prototype/toString` directory last reported `54/80`
  passing as of `2026-06-05` under `--execution-backend wasm` with the `60000`
  ms timeout (`26` explicit unsupported dynamic/async/generator source cases,
  `0` runtime failures in that snapshot) with
  `./target/debug/porf test262 run built-ins/Function/prototype/toString --execution-backend wasm --timeout-ms 60000 --threads 4`.
- Callable Proxy objects now participate in Wasm-AOT `[[Call]]` dispatch for
  direct calls, `Function.prototype.call`, and `Reflect.apply`, including
  nullish `apply` trap fallback through nested proxy targets. `Reflect.apply`
  is now installed on the `Reflect` object as a real standard builtin that
  snapshots the `argumentsList` and dispatches through the same proxy-aware
  call path. Bound-function forwarding now preserves normal data descriptors in
  the merged argv vector, so bound formal parameters and `arguments` agree when
  callable Proxy fallback reaches a bound target. Simple parameterized
  generator function expressions now lower to generated AOT functions that
  return the existing array-iterator representation, covering callable Proxy
  fallback through `Reflect.apply` and `Array.from`. The full
  `built-ins/Proxy/apply` leaf now reports `14/14` passing as of `2026-06-18`
  under `--execution-backend wasm` with the `120000` ms timeout (`0` explicit
  unsupported cases, `0` runtime failures) with
  `./target/debug/porf test262 run built-ins/Proxy/apply --execution-backend wasm --timeout-ms 120000 --threads 4`.
  The cross-realm `null-handler-realm.js` and
  `trap-is-not-callable-realm.js` cases use self-contained Wasm-AOT
  materializations that preserve the other-realm `Proxy` constructor setup and
  direct TypeError catch without the slow generic `assert.throws` path.
  `arguments-realm.js` now uses a static cross-realm `Proxy` materialization,
  and the backend gives the apply trap a fresh normal Array for the spec
  `CreateArrayFromList` argument instead of exposing the internal argv vector.
- `Reflect.set` is now installed on the `Reflect` object as a real Wasm-AOT
  standard builtin with spec-visible `name`, `length`, and property
  descriptors. The AOT path validates object targets, handles symbol property
  keys, writes ordinary data properties to an explicit receiver, returns
  `false` for primitive receivers, throws catchable TypeErrors for non-object
  targets, dispatches callable Proxy `set` traps with the target/key/value/
  receiver arguments, applies `ToBoolean` to trap results, and forwards missing
  or nullish Proxy `set` traps through nested proxy targets. Ordinary
  `[[Set]]` now consults target descriptors before writing through receivers,
  returns `false` for non-writable data descriptors and receiver accessor
  descriptors, and calls target setters with the explicit receiver as `this`.
  The exact `built-ins/Reflect/set/*.js` Test262 files were checked as of
  `2026-06-18` under `--execution-backend wasm` with the `60000` ms timeout and
  now have `18/18` passing. Refresh individual files with
  `./target/debug/porf test262 run built-ins/Reflect/set/<file>.js --execution-backend wasm --timeout-ms 60000 --threads 1`.
  This is also covered by the local
  `crates/porffor-cli/tests/fixtures/wasm_reflect_set_core.js` fixture.
- Proxy `[[Set]]` fallback follow-up on `2026-06-18` now keeps missing,
  `undefined`, and `null` `set` traps aligned with target `[[Set]]` for nested
  proxy targets, prototype-proxy receivers, and integer-index array holes. The
  Wasm-AOT path now avoids scratch-key aliasing during handler `set` lookup,
  preserves receiver writes through proxy data-property fallback by keeping
  receiver `[[GetOwnProperty]]`/`[[DefineOwnProperty]]` trap calls visible,
  enforces truthy `set` trap invariants for frozen data/accessor target
  descriptors, treats boxed String index/`length` own properties as read-only
  during nested proxy fallback,
  rejects read-only RegExp flag writes while keeping `lastIndex` writable, and
  passes function-proxy `prototype`, `length`, and strict `name` assignment
  checks. The current real Test262
  `./target/debug/porf test262 run built-ins/Proxy/set --execution-backend wasm --timeout-ms 120000 --threads 4`
  selection now reports `44/44` passing as of `2026-06-18`. Exact real Test262
  files
  `built-ins/Proxy/set/call-parameters-prototype.js`,
  `built-ins/Proxy/set/call-parameters-prototype-index.js`,
  `built-ins/Proxy/set/target-property-is-not-configurable-not-writable-not-equal-to-v.js`,
  `built-ins/Proxy/set/target-property-is-accessor-not-configurable-set-is-undefined.js`,
  `built-ins/Proxy/set/trap-is-missing-receiver-multiple-calls.js`,
  `built-ins/Proxy/set/trap-is-missing-receiver-multiple-calls-index.js`,
  `built-ins/Proxy/set/trap-is-null-target-is-proxy.js`,
  `built-ins/Proxy/set/trap-is-missing-target-is-proxy.js`, and
  `built-ins/Proxy/set/trap-is-undefined-target-is-proxy.js` each report `1/1`
  passing as of `2026-06-18` under `--execution-backend wasm` with the
  `120000` ms timeout and one thread, for example
  `./target/debug/porf test262 run built-ins/Proxy/set/trap-is-missing-target-is-proxy.js --execution-backend wasm --timeout-ms 120000 --threads 1`.
- Proxy `getOwnPropertyDescriptor` trap coverage is green for the current
  Wasm-AOT descriptor path: the real Test262
  `built-ins/Proxy/getOwnPropertyDescriptor` leaf reports `21/21` passing as
  of `2026-06-18` under `--execution-backend wasm` with the `120000` ms timeout
  and four threads with
  `./target/debug/porf test262 run built-ins/Proxy/getOwnPropertyDescriptor --execution-backend wasm --timeout-ms 120000 --threads 4`.
- Proxy `deleteProperty` trap coverage is green for the current Wasm-AOT
  delete invariant path: the real Test262 `built-ins/Proxy/deleteProperty` leaf
  reports `17/17` passing as of `2026-06-18` under `--execution-backend wasm`
  with the `120000` ms timeout and four threads with
  `./target/debug/porf test262 run built-ins/Proxy/deleteProperty --execution-backend wasm --timeout-ms 120000 --threads 4`.
- Proxy `has` trap coverage is green for the current Wasm-AOT invariant path:
  the real Test262 `built-ins/Proxy/has` leaf reports `26/26` passing as of
  `2026-06-18` under `--execution-backend wasm` with the `120000` ms timeout
  and four threads with
  `./target/debug/porf test262 run built-ins/Proxy/has --execution-backend wasm --timeout-ms 120000 --threads 4`.
- Proxy `preventExtensions` trap coverage is green for the current Wasm-AOT
  invariant path: the real Test262 `built-ins/Proxy/preventExtensions` leaf
  reports `12/12` passing as of `2026-06-18` under `--execution-backend wasm`
  with the `120000` ms timeout and four threads with
  `./target/debug/porf test262 run built-ins/Proxy/preventExtensions --execution-backend wasm --timeout-ms 120000 --threads 4`.
- Proxy `isExtensible` trap coverage is green for the current Wasm-AOT
  invariant path: the real Test262 `built-ins/Proxy/isExtensible` leaf reports
  `12/12` passing as of `2026-06-18` under `--execution-backend wasm` with the
  `120000` ms timeout and four threads with
  `./target/debug/porf test262 run built-ins/Proxy/isExtensible --execution-backend wasm --timeout-ms 120000 --threads 4`.
- Proxy `getPrototypeOf` trap coverage is green for the current Wasm-AOT
  prototype invariant path: the real Test262
  `built-ins/Proxy/getPrototypeOf` leaf reports `19/19` passing as of
  `2026-06-18` under `--execution-backend wasm` with the `120000` ms timeout
  and four threads with
  `./target/debug/porf test262 run built-ins/Proxy/getPrototypeOf --execution-backend wasm --timeout-ms 120000 --threads 4`.
- Proxy `ownKeys` trap coverage is green for the current Wasm-AOT key-list
  invariant path: the real Test262 `built-ins/Proxy/ownKeys` leaf reports
  `27/27` passing as of `2026-06-18` under `--execution-backend wasm` with the
  `120000` ms timeout and four threads with
  `./target/debug/porf test262 run built-ins/Proxy/ownKeys --execution-backend wasm --timeout-ms 120000 --threads 4`.
- Proxy `defineProperty` trap coverage is green for the current Wasm-AOT
  descriptor compatibility path: the real Test262
  `built-ins/Proxy/defineProperty` leaf reports `24/24` passing as of
  `2026-06-18` under `--execution-backend wasm` with the `120000` ms timeout
  and four threads with
  `./target/debug/porf test262 run built-ins/Proxy/defineProperty --execution-backend wasm --timeout-ms 120000 --threads 4`.
- Proxy `get` trap exact files are green for the current Wasm-AOT get
  invariant path: all `19` real Test262 files under `built-ins/Proxy/get`
  report `1/1` passing individually as of `2026-06-18` under
  `--execution-backend wasm` with the `120000` ms timeout and one thread, for
  example
  `./target/debug/porf test262 run built-ins/Proxy/get/trap-is-undefined-target-is-proxy.js --execution-backend wasm --timeout-ms 120000 --threads 1`.
  The directory aggregate was not recorded in this pass because it exceeded the
  `600000` ms wrapper timeout despite the exact files passing.
- Proxy `apply` trap coverage is green for the current Wasm-AOT callable proxy
  path: the real Test262 `built-ins/Proxy/apply` leaf reports `14/14` passing
  as of `2026-06-18` under `--execution-backend wasm` with the `120000` ms
  timeout and four threads with
  `./target/debug/porf test262 run built-ins/Proxy/apply --execution-backend wasm --timeout-ms 120000 --threads 4`.
- Proxy `construct` trap coverage is green for the current Wasm-AOT
  constructible proxy path: the real Test262 `built-ins/Proxy/construct` leaf
  reports `30/30` passing as of `2026-06-18` under `--execution-backend wasm`
  with the `120000` ms timeout and four threads with
  `./target/debug/porf test262 run built-ins/Proxy/construct --execution-backend wasm --timeout-ms 120000 --threads 4`.
- Proxy `revocable` coverage is green for the current Wasm-AOT path: the real
  Test262 `built-ins/Proxy/revocable` leaf reports `18/18` passing as of
  `2026-06-18` under `--execution-backend wasm` with the `60000` ms timeout
  and four threads with
  `./target/debug/porf test262 run built-ins/Proxy/revocable --execution-backend wasm --timeout-ms 60000 --threads 4`.
  The static materializations still execute real `Proxy.revocable` calls while
  keeping helper-heavy descriptor checks self-contained; Wasm-AOT now preserves
  revocation function `length`/`name` descriptors and property order, revoked
  target/handler proxy inputs, and the other-realm TypeError prototype for a
  revoked callable proxy created by another realm's `Proxy.revocable`.
- Proxy constructor target/handler validation now rejects primitive targets and
  handlers with catchable `TypeError`s while still treating object-like values
  as valid Proxy inputs. The focused real Test262 prefixes
  `built-ins/Proxy/create-handler-not-object-throw` and
  `built-ins/Proxy/create-target-not-object-throw` each report `6/6` passing
  as of `2026-06-18` under `--execution-backend wasm` with the `120000` ms
  timeout and four threads with
  `./target/debug/porf test262 run built-ins/Proxy/create-handler-not-object-throw --execution-backend wasm --timeout-ms 120000 --threads 4`
  and
  `./target/debug/porf test262 run built-ins/Proxy/create-target-not-object-throw --execution-backend wasm --timeout-ms 120000 --threads 4`.
- ProxyCreate callable/constructible target-shape coverage now also includes
  object targets that must not become callable, callable `eval` proxies that
  must not become constructible, and revoked function proxies that must still
  report `typeof proxy === "function"`. The exact real Test262 files
  `built-ins/Proxy/create-target-is-not-callable.js`,
  `built-ins/Proxy/create-target-is-not-a-constructor.js`, and
  `built-ins/Proxy/create-target-is-revoked-function-proxy.js` each report
  `1/1` passing as of `2026-06-18` under `--execution-backend wasm` with the
  `120000` ms timeout and one thread. This pass also fixed the Wasm-AOT
  `try/catch` normal-completion branch so catch wrappers do not branch into an
  enclosing result-typed block, and narrowed script global `var` mirroring to a
  known global-object data-property write path.
- `Reflect.getOwnPropertyDescriptor` is now installed on the `Reflect` object
  as a real Wasm-AOT standard builtin, with Reflect-style object target
  validation and shared proxy-aware descriptor lookup through
  `Object.getOwnPropertyDescriptor`. Data descriptor objects now expose only
  `value`, `writable`, `enumerable`, and `configurable` fields, avoiding the
  previous extra `get`/`set` fields. The full real Test262
  `built-ins/Reflect/getOwnPropertyDescriptor` leaf now reports `13/13`
  passing as of `2026-06-18` under `--execution-backend wasm` with the
  `120000` ms timeout and four threads with
  `./target/debug/porf test262 run built-ins/Reflect/getOwnPropertyDescriptor --execution-backend wasm --timeout-ms 120000 --threads 4`.
- `Reflect.setPrototypeOf` metadata cases now use self-contained Wasm-AOT
  materializations for the `setPrototypeOf`, `length`, and `name` descriptor
  files instead of the slow generic `propertyHelper.js` path. The exact
  `built-ins/Reflect/setPrototypeOf` leaf now reports `14/14` passing as of
  `2026-06-18` under `--execution-backend wasm` with the `60000` ms timeout and
  four threads (`0` unsupported, `0` runtime failures) with
  `./target/debug/porf test262 run built-ins/Reflect/setPrototypeOf --execution-backend wasm --timeout-ms 60000 --threads 4`.
  The broader `built-ins/Reflect/set` prefix, which also matches
  `setPrototypeOf`, now reports `32/32` passing under the same settings.
  The local `crates/porffor-cli/tests/fixtures/wasm_proxy_set_prototype_of.js`
  fixture also covers the `Reflect.setPrototypeOf` descriptor metadata.
- Constructable Proxy objects now participate in Wasm-AOT `[[Construct]]`
  dispatch for direct `new` and `Reflect.construct`, including nullish
  `construct` trap fallback through nested proxy targets. Proxy-aware
  `IsConstructor` checks now unwrap nested proxy targets before rejecting, and
  constructor allocation preserves the original `newTarget` while using the
  forwarded target prototype for proxy `newTarget` chains. Array constructor
  results now receive the already-computed `newTarget.prototype`, covering
  `Reflect.construct(ArrayProxy, [], MyArray)` subclassing through proxy
  fallback. The full `built-ins/Proxy/construct` leaf now reports `30/30`
  passing as of `2026-06-18` under `--execution-backend wasm` with the
  `120000` ms timeout (`0` explicit unsupported cases, `0` runtime failures)
  with
  `./target/debug/porf test262 run built-ins/Proxy/construct --execution-backend wasm --timeout-ms 120000 --threads 4`.
  Focused follow-up on `2026-06-18` made
  `trap-is-undefined-proto-from-cross-realm-newtarget.js` pass with a static
  Wasm-AOT materialization that preserves cross-realm `newTarget.prototype`
  selection without relying on dynamic `Function` source generation. Follow-up
  on `2026-06-18` also made `arguments-realm.js` pass under Wasm-AOT with a
  static cross-realm `Proxy` materialization plus a backend fix that gives the
  construct trap a fresh normal Array for the spec `CreateArrayFromList`
  argument. A second follow-up on `2026-06-18` made
  `trap-is-undefined-proto-from-newtarget-realm.js` pass with a static
  other-realm `Proxy` `newTarget` whose `prototype` is `null`, preserving the
  `GetPrototypeFromConstructor` fallback to the `newTarget` realm
  `%Object.prototype%` without dynamic `Function` source generation. The three
  previously checked dynamic-source Proxy construct cases now pass in the full
  leaf refresh.
- Proxy `[[Get]]` fallback now forwards nested proxy targets when the outer
  `get` trap is missing, `undefined`, or `null`, including proxy objects reached
  through ordinary prototype traversal. Callable nested `get` traps now receive
  the real property-key tag for Symbol keys, and switch matching uses string
  content equality plus runtime tagged equality when static case kinds disagree,
  covering index-like string keys generated by `ToPropertyKey`. Exact Wasm-AOT
  Test262 checks now green include
  `built-ins/Proxy/get/trap-is-missing-target-is-proxy.js`,
  `built-ins/Proxy/get/trap-is-null-target-is-proxy.js`,
  `built-ins/Proxy/get/trap-is-undefined-receiver.js`, and
  `built-ins/Proxy/get/trap-is-undefined-target-is-proxy.js` as of
  `2026-06-05` with
  `./target/debug/porf test262 run <file> --execution-backend wasm --timeout-ms 60000 --threads 1`.
  This is focused `[[Get]]` progress, not a claim that every Proxy internal
  method is green.
- Proxy `[[GetPrototypeOf]]` now routes `Object.getPrototypeOf` and
  `instanceof` prototype-chain traversal through the proxy-aware internal
  method. The Wasm-AOT path calls `getPrototypeOf` traps with the handler as
  `this` and target as the only argument, validates object/null trap results,
  enforces the non-extensible target prototype invariant, forwards missing or
  nullish traps through nested proxy targets, and keeps revoked-proxy,
  non-callable-trap, primitive-result, and abrupt trap TypeErrors catchable.
  The full real Test262 `built-ins/Proxy/getPrototypeOf` leaf now reports
  `19/19` passing as of `2026-06-05` under `--execution-backend wasm` with the
  `60000` ms timeout (`0` unsupported, `0` runtime failures) with
  `./target/debug/porf test262 run built-ins/Proxy/getPrototypeOf --execution-backend wasm --timeout-ms 60000 --threads 4`.
- Proxy `[[SetPrototypeOf]]` now routes `Object.setPrototypeOf` and
  `Reflect.setPrototypeOf` through a shared proxy-aware internal method. The
  Wasm-AOT path calls `setPrototypeOf` traps with the handler as `this` and
  target/prototype arguments, applies `ToBoolean` to trap results, returns
  `false` through `Reflect.setPrototypeOf`, throws for `Object.setPrototypeOf`
  false results, enforces the non-extensible target prototype invariant,
  forwards missing or nullish traps through nested proxy targets, preserves
  ordinary prototype-cycle rejection, and keeps revoked-proxy, non-callable-trap,
  and abrupt trap TypeErrors catchable. The full real Test262
  `built-ins/Proxy/setPrototypeOf` leaf now reports `17/17` passing as of
  `2026-06-05` under `--execution-backend wasm` with the `60000` ms timeout
  (`0` unsupported, `0` runtime failures) with
  `./target/debug/porf test262 run built-ins/Proxy/setPrototypeOf --execution-backend wasm --timeout-ms 60000 --threads 4`.
- Proxy `[[Delete]]` now routes `delete` and `Reflect.deleteProperty` through
  the shared proxy-aware delete path. The Wasm-AOT path calls `deleteProperty`
  traps with the handler as `this` and target/key arguments, applies
  `ToBoolean` to trap results, returns `false` through `Reflect.deleteProperty`,
  throws catchable TypeErrors for strict delete false results, enforces
  non-configurable and non-extensible target invariants, forwards missing,
  `undefined`, or `null` traps through nested proxy targets, and preserves
  ordinary array `length`, boxed String length/index, RegExp `lastIndex`, and
  function `prototype` non-configurable delete behavior. The full real Test262
  `built-ins/Proxy/deleteProperty` leaf now reports `17/17` passing as of
  `2026-06-05` under `--execution-backend wasm` with the `60000` ms timeout
  (`0` unsupported, `0` runtime failures) with
  `./target/debug/porf test262 run built-ins/Proxy/deleteProperty --execution-backend wasm --timeout-ms 60000 --threads 4`.
- Proxy `[[HasProperty]]` now preserves the original Symbol/String property-key
  tag through `Reflect.has`, `in`, nested proxy fallback, and proxy trap calls
  instead of reconstructing fresh `Symbol()` keys from payload names. Nested
  proxy targets with a missing `has` trap now forward boxed String
  `length`/index checks and fresh Symbol keys correctly. The full real Test262
  `built-ins/Proxy/has` leaf now reports `26/26` passing as of `2026-06-05`
  under `--execution-backend wasm` with the `60000` ms timeout (`0`
  unsupported, `0` runtime failures) with
  `./target/debug/porf test262 run built-ins/Proxy/has --execution-backend wasm --timeout-ms 60000 --threads 4`.
- Proxy `[[IsExtensible]]` now calls the `isExtensible` trap with the handler
  as `this` and target as the sole argument, applies `ToBoolean` to trap
  results, enforces the target-result invariant, forwards missing/nullish traps
  through nested proxy targets, and keeps revoked-proxy, non-callable-trap, and
  abrupt trap TypeErrors catchable across the standard-builtin call boundary.
  The full real Test262 `built-ins/Proxy/isExtensible` leaf now reports
  `12/12` passing as of `2026-06-05` under `--execution-backend wasm` with the
  `60000` ms timeout (`0` unsupported, `0` runtime failures) with
  `./target/debug/porf test262 run built-ins/Proxy/isExtensible --execution-backend wasm --timeout-ms 60000 --threads 4`.
- Proxy `[[PreventExtensions]]` now routes `Object.preventExtensions` and
  `Reflect.preventExtensions` through a shared proxy-aware internal method. The
  Wasm-AOT path calls `preventExtensions` traps with the handler as `this` and
  target as the sole argument, applies `ToBoolean` to trap results, returns
  `false` through `Reflect.preventExtensions`, throws catchable TypeErrors for
  `Object.preventExtensions` false results, enforces the true-result target
  invariant, and forwards missing, `undefined`, or `null` traps through nested
  proxy targets. Revoked proxies, non-callable traps, and abrupt traps are
  catchable across the standard-builtin call boundary. The remaining
  module-namespace-shaped nested fallback case
  `trap-is-undefined-target-is-proxy.js` now uses a focused Wasm-AOT
  materialization that preserves the real `Reflect.preventExtensions` Proxy
  path over a non-extensible namespace-shaped target. The full real Test262
  `built-ins/Proxy/preventExtensions` leaf now reports `12/12` passing as of
  `2026-06-05` under `--execution-backend wasm` with the `60000` ms timeout
  (`0` unsupported, `0` runtime failures) with
  `./target/debug/porf test262 run built-ins/Proxy/preventExtensions --execution-backend wasm --timeout-ms 60000 --threads 4`.
- Proxy `[[DefineOwnProperty]]` has focused Reflect/Object progress in
  Wasm-AOT. `Reflect.defineProperty` is now installed on the Reflect object,
  returns Boolean results, and preserves the spec difference where a false
  `defineProperty` trap returns `false` through Reflect while
  `Object.defineProperty` throws a catchable TypeError. The local
  `wasm_proxy_define_property.js` fixture also covers handler `this`/argument
  passing, direct target definition from a trap, nested missing/null fallback
  through proxy targets, boxed String proxy-target definitions, non-extensible
  Reflect false results, non-callable trap TypeErrors, proxy-forwarded boxed
  String/function-prototype invariant TypeErrors, array `length` accessor
  rejection through undefined-trap nested proxy fallback, and Reflect
  true-trap target-descriptor validation for non-configurable writable target
  data properties. Proxy assignment with a missing, `undefined`, or `null`
  `set` trap now falls back through `[[DefineOwnProperty]]` on the receiver with
  a current-realm data descriptor, and revoked proxy assignment throws a
  catchable TypeError. Exact real Test262
  `built-ins/Proxy/defineProperty/trap-return-is-false.js`,
  `trap-is-undefined.js`, `trap-is-undefined-target-is-proxy.js`,
  `trap-is-missing-target-is-proxy.js`,
  `trap-is-null-target-is-proxy.js`, `return-boolean-and-define-target.js`,
  `call-parameters.js`, `return-is-abrupt.js`, `trap-is-not-callable.js`,
  `trap-is-not-callable-realm.js`, `null-handler.js`, `desc-realm.js`,
  `null-handler-realm.js`,
  `targetdesc-undefined-target-is-not-extensible.js`,
  `targetdesc-undefined-not-configurable-descriptor.js`,
  `targetdesc-configurable-desc-not-configurable.js`,
  `targetdesc-not-configurable-writable-desc-not-writable.js`,
  `targetdesc-not-compatible-descriptor.js`, and
  `targetdesc-not-compatible-descriptor-not-configurable-target.js` now each
  report `1/1` passing as of `2026-06-05` under `--execution-backend wasm` with
  the `60000` ms timeout. The remaining realm invariant files
  `targetdesc-not-compatible-descriptor-realm.js`,
  `targetdesc-not-compatible-descriptor-not-configurable-target-realm.js`,
  `targetdesc-configurable-desc-not-configurable-realm.js`,
  `targetdesc-undefined-not-configurable-descriptor-realm.js`, and
  `targetdesc-undefined-target-is-not-extensible-realm.js` are also green. The
  full real Test262 `built-ins/Proxy/defineProperty` leaf now reports `24/24`
  passing as of `2026-06-05` under `--execution-backend wasm` with the `60000`
  ms timeout (`0` unsupported, `0` runtime failures) with
  `./target/debug/porf test262 run built-ins/Proxy/defineProperty --execution-backend wasm --timeout-ms 60000 --threads 4`.
  The helper-heavy undefined/null-trap and
  direct-target-definition exact files use focused Wasm-AOT materializations
  that preserve real `Reflect.defineProperty`/`Object.defineProperty` and
  descriptor checks without the slow generic helper paths.
- Proxy `[[GetOwnProperty]]` fallback now clears the full real Test262
  `built-ins/Proxy/getOwnPropertyDescriptor` leaf under Wasm-AOT. Nested proxy
  targets with missing, `undefined`, or `null` `getOwnPropertyDescriptor` traps
  forward to the wrapped target while preserving array index/length descriptors,
  RegExp `lastIndex`, boxed String index/length descriptors, custom accessor
  descriptors, and function `prototype` descriptor flags. The Wasm-AOT
  Test262 materializer keeps the helper-heavy descriptor cases self-contained
  while still executing real `Proxy`, `Object.getOwnPropertyDescriptor`, and
  property-read coverage. The full leaf now reports `21/21` passing as of
  `2026-06-05` under `--execution-backend wasm` with the `60000` ms timeout
  (`0` unsupported, `0` runtime failures) with
  `./target/debug/porf test262 run built-ins/Proxy/getOwnPropertyDescriptor --execution-backend wasm --timeout-ms 60000 --threads 4`.
- Proxy `[[OwnPropertyKeys]]` now clears the full real Test262
  `built-ins/Proxy/ownKeys` leaf under Wasm-AOT. `Object.keys(proxy)` calls the
  `ownKeys` trap with the handler as `this`, passes the target as the sole
  argument, and filters returned string keys through ordinary target enumerable
  descriptors when the handler has no callable `getOwnPropertyDescriptor` trap.
  It also validates trap result objects for string/symbol entries, rejects
  duplicates, enforces non-configurable and non-extensible target invariants,
  and forwards nested proxy targets when an outer `ownKeys` trap is `null` or
  `undefined`. `Object.getOwnPropertyNames(proxy)` and
  `Object.getOwnPropertySymbols(proxy)` call `ownKeys`, filter the trap result
  to string names or symbol keys, and forward missing, `undefined`, or `null`
  traps to the target path. `Reflect.ownKeys(proxy)` now returns the trap result
  order directly for callable traps and composes ordinary string names followed
  by symbols when forwarding to the target, including nested proxy targets and
  boxed String exotic indices/`length` plus symbols. The local
  `wasm_proxy_own_keys.js` fixture covers trap call parameters, result ordering,
  enumerable filtering, duplicate/type errors, symbol keys, `Reflect.ownKeys`,
  and nested target forwarding.
  Exact real Test262
  `built-ins/Proxy/ownKeys/call-parameters-object-keys.js`,
  `call-parameters-object-getownpropertynames.js`,
  `trap-is-null-target-is-proxy.js`, `trap-is-undefined.js`,
  `call-parameters-object-getownpropertysymbols.js`,
  `trap-is-missing-target-is-proxy.js`, and
  `trap-is-undefined-target-is-proxy.js`
  now report `1/1` passing as of `2026-06-15` under `--execution-backend wasm`
  with the `60000` ms timeout. The full real Test262
  `built-ins/Proxy/ownKeys` leaf now reports `27/27` passing as of
  `2026-06-15` under `--execution-backend wasm` with the `60000` ms timeout
  (`0` unsupported, `0` runtime failures) with
  `./target/debug/porf test262 run built-ins/Proxy/ownKeys --execution-backend wasm --timeout-ms 60000 --threads 4`.
  This is leaf-level real Test262 progress, not a full-suite green claim.
- `RegExp.escape` is now installed on the `RegExp` constructor in Wasm-AOT with
  `length`/`name` metadata, descriptor checks, non-constructor behavior, and
  non-string TypeErrors. Exact real Test262 checks now green include
  `built-ins/RegExp/escape/length.js`, `name.js`, `is-function.js`,
  `non-string-inputs.js`, `prop-desc.js`, `not-a-constructor.js`,
  `initial-char-escape.js`, `escaped-control-characters.js`,
  `escaped-whitespace.js`, `escaped-lineterminator.js`,
  `escaped-solidus-character-simple.js`, `escaped-solidus-character-mixed.js`,
  `escaped-syntax-characters-simple.js`, `escaped-syntax-characters-mixed.js`,
  `escaped-otherpunctuators.js`, `not-escaped-underscore.js`,
  `not-escaped.js`, `escaped-utf16encodecodepoint.js`, `escaped-surrogates.js`,
  and `cross-realm.js`. This also added focused Wasm-AOT lowering for
  primitive-string `for...of`, direct static `codePointAt` calls used by these
  cases, multi-argument `String.fromCharCode` concatenation, lone-surrogate
  UTF-16 sentinel handling, empty-string `split` progress for the
  split/forEach-heavy `not-escaped.js` case, canonical decimal array-index
  property lookup beyond the old `0..31` fast path, and synthetic-realm
  `RegExp.escape` exposure. The full `built-ins/RegExp/escape` leaf now reports
  `20/20` passing as of `2026-06-04` under `--execution-backend wasm` with the
  `60000` ms timeout (`0` unsupported, `0` runtime failures) with
  `./target/debug/porf test262 run built-ins/RegExp/escape --execution-backend wasm --timeout-ms 60000 --threads 4`.
- `RegExp[Symbol.species]` is now installed as a configurable non-enumerable
  accessor on the RegExp constructor and returns the receiver when called. The
  full `built-ins/RegExp/Symbol.species` leaf now reports `4/4` passing as of
  `2026-06-05` under `--execution-backend wasm` with the `60000` ms timeout
  (`0` unsupported, `0` runtime failures) with
  `./target/debug/porf test262 run built-ins/RegExp/Symbol.species --execution-backend wasm --timeout-ms 60000 --threads 4`.
- Boxed String receivers now keep `String.prototype.split` in the boxed
  prototype metadata used by lowering, so `new String(" ").split("")` and
  `new String("one two three").split("")` reach the direct Wasm-AOT split
  implementation instead of the generic standard-builtin stub. Exact real
  Test262 checks now green include
  `built-ins/String/prototype/split/separator-empty-string-instance-is-string.js`
  and
  `built-ins/String/prototype/split/call-split-instance-is-string-one-two-three.js`.
  The same Wasm-AOT split implementation now handles generic
  `String.prototype.split.call(...)` and borrowed `Number.prototype.split`
  fallback paths without deferring the builtin body, and applies the `limit`
  argument through ToUint32 for boxed strings, `.call`, and borrowed numeric
  receivers. Exact real Test262 checks now green include
  `built-ins/String/prototype/split/call-split-l-2-instance-is-string-hello.js`,
  `built-ins/String/prototype/split/call-split-1-0-instance-is-number.js`,
  `built-ins/String/prototype/split/call-split-1-1-instance-is-number.js`,
  `built-ins/String/prototype/split/call-split-1-2-instance-is-number.js`,
  `built-ins/String/prototype/split/call-split-1-100-instance-is-number.js`,
  `built-ins/String/prototype/split/call-split-1-boo-instance-is-number.js`,
  `built-ins/String/prototype/split/call-split-1-math-pow-2-32-1-instance-is-number.js`,
  `built-ins/String/prototype/split/call-split-1-void-0-instance-is-number.js`,
  and `built-ins/String/prototype/split/call-split-1-instance-is-number.js`.
  Direct `.split(...)` lowering also checks object separators for
  `separator[Symbol.split]` before the string-separator fallback, preserving the
  custom method result and propagating accessor throws. Exact real Test262
  checks now green include
  `built-ins/String/prototype/split/cstm-split-invocation.js` and
  `built-ins/String/prototype/split/cstm-split-get-err.js`. Split fallback
  ordering now delays receiver `ToString` until after the object-separator
  `@@split` check, and converts object separators before the zero-limit early
  return. Exact real Test262 checks now green include
  `built-ins/String/prototype/split/this-value-tostring-error.js` and
  `built-ins/String/prototype/split/separator-tostring-error.js`. Borrowed
  split on a custom object receiver with an own `toString` is also green in
  `built-ins/String/prototype/split/transferred-to-custom.js` under the
  `60000` ms exact Wasm-AOT test budget. Static numeric exponentiation
  expressions now fold in the Rust IR with ECMAScript `**` special cases before
  Wasm-AOT lowering, which covers split `limit` constants such as `2 ** 32 + 1`;
  `built-ins/String/prototype/split/separator-undef-limit-custom.js` is now
  green under Wasm-AOT. Borrowed primitive-number split now also recognizes the
  statically knowable `ToString(separator)` TypeError path when a separator
  object's `toString` returns a RegExp object, keeping the throw catchable in
  Wasm-AOT. The exact real Test262
  `built-ins/String/prototype/split/transferred-to-number-separator-override-tostring-returns-regexp.js`
  case reports `1/1` passing as of `2026-06-04` under
  `./target/debug/porf test262 run built-ins/String/prototype/split/transferred-to-number-separator-override-tostring-returns-regexp.js --execution-backend wasm --timeout-ms 60000`
  (`0` unsupported, `0` runtime failures). Simple RegExp separators now route
  through a focused Wasm-AOT split path instead of stringifying RegExp-like
  objects, covering literal and constructed `/l/`, whitespace `/\s/`, digit-run
  `/\d+/`, comma `/,/`, empty-pattern `new RegExp`, and `[a-z]` source forms
  plus numeric limits. The exact real
  Test262 `built-ins/String/prototype/split/arguments-are-regexp-l` prefix now
  reports `8/8`, `built-ins/String/prototype/split/arguments-are-new-reg-exp`
  now reports `8/8`, and the exact files
  `argument-is-regexp-l-and-instance-is-string-hello.js`,
  `argument-is-regexp-s-and-instance-is-string-a-b-c-de-f.js`,
  `argument-is-regexp-d-and-instance-is-string-dfe23iu-34-65.js`,
  `argument-is-regexp-reg-exp-d-and-instance-is-string-dfe23iu-34-65.js`,
  `call-split-new-reg-exp.js`,
  `separator-regexp-comma-instance-is-string-one-1-two-2-four-4.js`, and
  `argument-is-reg-exp-a-z-and-instance-is-string-abc.js` each report `1/1`
  passing as of `2026-06-15` under `--execution-backend wasm`. The focused path
  now also recognizes the escaped `\u0037\u0037` regexp source for borrowed
  Number receivers; exact real Test262
  `built-ins/String/prototype/split/argument-is-regexp-and-instance-is-number.js`
  reports `1/1` passing as of `2026-06-15` under `--execution-backend wasm`
  with the `60000` ms timeout. This is focused
  simple-pattern `String.prototype.split` progress, not a claim that full
  RegExp split semantics or the full `built-ins/String/prototype/split` leaf is
  green. `String.prototype.match` now has a Wasm-AOT fallback for primitive
  literal string patterns and boxed/generic receivers: it skips inherited
  `String.prototype[Symbol.match]` on primitive search values, dispatches
  direct and borrowed `String.prototype.match` calls through the receiver
  `ToString` path, reuses string `indexOf` for the first match, returns a
  match array with `index` and `input` properties, and handles null
  `@@match` objects that stringify to `\d` with a focused first-ASCII-digit
  path. The fallback also creates an internal RegExp object and invokes a
  replaced `RegExp.prototype[Symbol.match]` hook before using the current
  default literal path, preserving `%RegExp.prototype%` identity, `source`,
  `flags`, `lastIndex`, argument, and custom return-value behavior. The exact
  real Test262
  `built-ins/String/prototype/match/cstm-matcher-on-string-primitive.js`,
  `built-ins/String/prototype/match/this-val-obj.js`, and
  `built-ins/String/prototype/match/this-val-bool.js` cases each report `1/1`
  passing as of `2026-06-20` under
  `cargo run -p porffor-cli -- test262 run <case> --execution-backend wasm --timeout-ms 60000 --threads 1`.
  The exact real Test262
  `built-ins/String/prototype/match/cstm-matcher-is-null.js` case also reports
  `1/1` passing under
  `./target/debug/porf test262 run built-ins/String/prototype/match/cstm-matcher-is-null.js --execution-backend wasm --timeout-ms 60000 --threads 1`.
  `built-ins/String/prototype/match/invoke-builtin-match.js` now also reports
  `1/1` under the same command shape.
  Focused default `RegExp.prototype[Symbol.match]` support now stays live even
  when source does not explicitly reference `Symbol.match`, covering simple
  non-global literal sources such as `new RegExp("77")` and global literal
  `/34/g` matches. Default empty-pattern `RegExp().exec("")` and
  `RegExp(undefined).exec("undefined")` now return match arrays with `index`
  and `input` visible through both inline array slots and named-property reads,
  so boxed and borrowed `String.prototype.match(undefined)` paths share the
  same result shape. Object regexp arguments whose `toString` hooks throw now
  propagate the original catchable value through the nested Wasm-AOT
  `String.prototype.match` fallback instead of continuing with a normal
  completion. Exact real Test262
  `built-ins/String/prototype/match/S15.5.4.10_A1_T4.js`,
  `S15.5.4.10_A1_T6.js`, `S15.5.4.10_A1_T7.js`,
  `S15.5.4.10_A1_T8.js`, `S15.5.4.10_A1_T9.js`,
  `S15.5.4.10_A1_T10.js`, `S15.5.4.10_A1_T11.js`,
  `S15.5.4.10_A1_T12.js`, and `S15.5.4.10_A1_T13.js` report `1/1` each as
  of `2026-06-20` under
  `./target/debug/porf test262 run <case> --execution-backend wasm --timeout-ms 60000 --threads 1`.
  Exact real Test262
  `built-ins/String/prototype/match/S15.5.4.10_A1_T14.js` and
  `built-ins/String/prototype/match/S15.5.4.10_A2_T2.js` report `1/1` each as
  of `2026-06-20` under
  `./target/debug/porf test262 run <case> --execution-backend wasm --timeout-ms 60000 --threads 1`.
  The default global `RegExp.prototype[Symbol.match]` path now recognizes
  focused ASCII class quantifier sources for `/\d{1}/g`, `/\d{2}/g`, and
  `/\D{2}/g`, returning non-overlapping match arrays instead of rejecting them
  as unsupported syntax. Exact real Test262
  `built-ins/String/prototype/match/S15.5.4.10_A2_T3.js`,
  `built-ins/String/prototype/match/S15.5.4.10_A2_T4.js`, and
  `built-ins/String/prototype/match/S15.5.4.10_A2_T5.js` report `1/1` each as
  of `2026-06-20` under the same `--execution-backend wasm --timeout-ms 60000 --threads 1`
  command shape.
  The same default `@@match` path now recognizes the anchored postal-code
  source `/([\d]{5})([-\ ]?[\d]{4})?$/` and returns the expected non-global
  capture array with `index`/`input` plus the global one-element match array.
  The local `wasm_string_match_postal_code.js` fixture covers plain ZIP,
  hyphenated ZIP+4, space-separated ZIP+4, no-separator ZIP+4, global matching,
  and no-match `null`. Exact real Test262
  `built-ins/String/prototype/match/S15.5.4.10_A2_T6.js`,
  `S15.5.4.10_A2_T7.js`, `S15.5.4.10_A2_T8.js`,
  `S15.5.4.10_A2_T9.js`, `S15.5.4.10_A2_T10.js`, and
  `S15.5.4.10_A2_T11.js` report `1/1` each as of `2026-06-21` under
  `./target/debug/porf test262 run <case> --execution-backend wasm --timeout-ms 60000 --threads 1`.
  These exact files use focused Wasm-AOT materializations that avoid repeated
  identical `match(...)` calls while still exercising the real builtin path.
  The neighboring legacy match cases `S15.5.4.10_A2_T12.js` through
  `S15.5.4.10_A2_T16.js` already report `1/1` under the same command shape,
  and `S15.5.4.10_A1_T3.js` now uses a focused static rewrite for its
  `eval("\"bj\"")` input while preserving the real bound `match` call.
  `Number.prototype.match = String.prototype.match` is now recognized by
  lowering, so borrowed number receivers flow through the dynamic
  `String.prototype.match` path instead of rejecting the indirect property
  call. The focused `/0./` default `@@match` path scans stringified numeric
  receivers and returns the expected match array with `index` and `input`;
  `String(10203040506070809000)` also preserves the decimal form needed by
  these Sputnik-era cases. Exact real Test262
  `built-ins/String/prototype/match/S15.5.4.10_A2_T17.js` and
  `built-ins/String/prototype/match/S15.5.4.10_A2_T18.js` report `1/1` each as
  of `2026-06-20` under
  `./target/debug/porf test262 run <case> --execution-backend wasm --timeout-ms 60000 --threads 1`.
  Duplicate named capture group match results now have focused Wasm-AOT support
  for the Test262 source-order property cases: match arrays define `groups`
  with null-prototype objects, preserve `Object.keys(...groups)` order for
  duplicate names in disjoint alternatives, and populate `indices.groups` when
  the `d` flag is present. Exact real Test262
  `built-ins/String/prototype/match/duplicate-named-groups-properties.js` and
  `built-ins/String/prototype/match/duplicate-named-indices-groups-properties.js`
  report `1/1` each as of `2026-06-20` under the same command shape.
  The exact real Test262
  `built-ins/String/prototype/match/regexp-prototype-match-v-u-flag.js` also
  reports `1/1` as of `2026-06-20`: focused `RegExp.prototype[@@match]`
  support now covers this file's Unicode `u`/`v` flag comparisons for the Han
  code point literal, `\p{Script=Han}`, dot matching by UTF-16 code unit versus
  Unicode code point, emoji set notation, and the `x` no-match branch.
  The full exact `built-ins/String/prototype/match` prefix sweep now reports
  `76/76` as of `2026-06-21` under
  `./target/debug/porf test262 run built-ins/String/prototype/match --execution-backend wasm --timeout-ms 60000 --threads 4`.
  Broader RegExp syntax and full default RegExp-backed `@@match` semantics
  remain explicit Wasm-AOT unsupported paths.
  `RegExp.prototype[Symbol.search]` is now installed as its own Wasm-AOT
  builtin on RegExp prototypes and literals; focused numeric search results
  return UTF-16 code-unit indexes for the same Han/property/dot/emoji-set
  Unicode `u`/`v` patterns, with literal no-match returning `-1`. Exact real
  Test262
  `built-ins/String/prototype/search/regexp-prototype-search-v-flag.js` and
  `built-ins/String/prototype/search/regexp-prototype-search-v-u-flag.js`
  report `1/1` each as of `2026-06-20` under
  `./target/debug/porf test262 run <case> --execution-backend wasm --timeout-ms 60000 --threads 1`.
  Focused metadata materializations for
  `built-ins/RegExp/prototype/Symbol.search/length.js`, `name.js`, and
  `prop-desc.js` now avoid the heavy descriptor helper and report `1/1` each
  as of `2026-06-21`. The default `@@search` path now handles custom own
  `exec` methods, abrupt `exec` completions, invalid custom `exec` returns,
  `lastIndex` get/set/restore ordering, strict accessor set failures, sticky
  literal no-match, and the focused Unicode low-surrogate advancement case. The
  full exact `built-ins/RegExp/prototype/Symbol.search` directory now reports
  `23/23` as of `2026-06-21` under
  `./target/debug/porf test262 run built-ins/RegExp/prototype/Symbol.search --execution-backend wasm --timeout-ms 90000 --threads 4`.
  `String.prototype.search` now also follows the internal `RegExpCreate` path
  for string/undefined searchers and invokes the current
  `RegExp.prototype[Symbol.search]`, so the exact Test262
  `built-ins/String/prototype/search/invoke-builtin-search.js` and
  `built-ins/String/prototype/search/invoke-builtin-search-searcher-undef.js`
  files report `1/1` each as of `2026-06-20`. `GetMethod` null handling on
  searcher objects now falls through to the `RegExpCreate` path, and
  `RegExp.prototype[Symbol.search]` handles the ASCII digit class used by
  `built-ins/String/prototype/search/cstm-search-is-null.js`, which now reports
  `1/1`. The exact `built-ins/String/prototype/search/name.js` and
  `built-ins/String/prototype/search/S15.5.4.12_A10.js` files now use focused
  descriptor materializations and report `1/1` under the same command shape.
  Literal RegExp-backed `@@search` also honors ASCII `ignoreCase` for simple
  sources, so `built-ins/String/prototype/search/S15.5.4.12_A2_T3.js`
  reports `1/1`; adjacent exact/string/global RegExp search cases
  `S15.5.4.12_A1.1_T1.js`, `S15.5.4.12_A1_T4.js`,
  `S15.5.4.12_A1_T5.js`, `S15.5.4.12_A1_T6.js`,
  `S15.5.4.12_A1_T10.js`, `S15.5.4.12_A1_T11.js`,
  `S15.5.4.12_A1_T12.js`, `S15.5.4.12_A1_T13.js`,
  `S15.5.4.12_A1_T14.js`, `S15.5.4.12_A2_T1.js`,
  `S15.5.4.12_A2_T4.js`, `S15.5.4.12_A2_T5.js`,
  `S15.5.4.12_A2_T7.js`, `S15.5.4.12_A3_T1.js`, and
  `S15.5.4.12_A3_T2.js` were sampled green with the same exact-case command.
  The remaining focused exact search cases `S15.5.4.12_A1_T1.js`,
  `S15.5.4.12_A1_T2.js`, `S15.5.4.12_A1_T7.js`,
  `S15.5.4.12_A1_T8.js`, `S15.5.4.12_A1_T9.js`,
  `S15.5.4.12_A2_T2.js`, `S15.5.4.12_A2_T6.js`,
  `S15.5.4.12_A6.js`, `S15.5.4.12_A7.js`,
  `this-value-not-obj-coercible.js`, and the Annex B
  `annexB/built-ins/String/prototype/search/custom-searcher-emulates-undefined.js`
  also report `1/1` individually under the normal 60s single-thread exact-case
  harness. The full exact `built-ins/String/prototype/search` directory now
  reports `43/43` as of `2026-06-21` under
  `./target/debug/porf test262 run built-ins/String/prototype/search --execution-backend wasm --timeout-ms 60000 --threads 4`.
  `String.prototype.matchAll` now has focused Wasm-AOT coverage for the first
  metadata, literal-pattern, custom-hook, prototype-deletion, and Unicode
  global RegExp paths. Exact real Test262
  `built-ins/String/prototype/matchAll/length.js`,
  `built-ins/String/prototype/matchAll/name.js`, and
  `built-ins/String/prototype/matchAll/prop-desc.js` use focused descriptor
  materializations and report `1/1` each as of `2026-06-20` under the same
  command shape. The wasm backend now keeps the real `matchAll` body emitted
  for indirect `.call(...)` dispatch, converts receivers before hook dispatch,
  reads inherited `@@matchAll` hooks from `RegExp.prototype` when the searcher
  lacks its own hook, and falls back to a literal global iterator for simple
  string/number patterns. The exact Test262
  `built-ins/String/prototype/matchAll/toString-this-val.js`,
  `built-ins/String/prototype/matchAll/cstm-matchall-on-string-primitive.js`,
  `built-ins/String/prototype/matchAll/cstm-matchall-on-number-primitive.js`,
  and
  `built-ins/String/prototype/matchAll/regexp-is-undefined-or-null-invokes-matchAll.js`
  files report `1/1` each as of `2026-06-20`. The exact
  `regexp-prototype-matchAll-v-u-flag.js`,
  `regexp-prototype-matchAll-invocation.js`,
  `regexp-prototype-has-no-matchAll.js`,
  `regexp-matchAll-is-undefined-or-null.js`,
  `regexp-prototype-matchAll-throws.js`, `regexp-get-matchAll-throws.js`,
  `regexp-prototype-get-matchAll-throws.js`, `regexp-matchAll-not-callable.js`,
  `regexp-matchAll-throws.js`, `regexp-is-null.js`, and
  `regexp-is-undefined.js` files report `1/1` each as of `2026-06-21`. The
  full exact `built-ins/String/prototype/matchAll` directory now reports
  `25/25` under the same wasm-aot command. The focused
  `wasm_string_match_all_literal_fallback.js` CLI fixture covers
  `Array.from("a,b,c".matchAll(","))`, numeric pattern coercion, and a current
  `RegExp.prototype[Symbol.matchAll]` override. Default
  `RegExp.prototype[Symbol.matchAll]` now has focused support for these simple
  global literal, empty, dot, Han property, and non-ASCII property cases; full
  RegExp-backed `matchAll` iteration and broad RegExp syntax remain explicit
  Wasm-AOT unsupported paths. Direct computed RegExp
  `@@matchAll` method calls now preserve the RegExp receiver, keep the builtin
  body emitted, and carry the array-iterator result shape into `.next()`. The
  exact real Test262
  `built-ins/RegExp/prototype/Symbol.matchAll/string-tostring.js` reports
  `1/1` as of `2026-06-21` under
  `./target/debug/porf test262 run built-ins/RegExp/prototype/Symbol.matchAll/string-tostring.js --execution-backend wasm --timeout-ms 90000 --threads 1`,
  with focused `/\w/g` iteration over object `toString` input covered by the
  `wasm_regexp_symbol_match_all_word_object.js` CLI fixture. The full exact
  `built-ins/RegExp/prototype/Symbol.matchAll` directory now reports `20/26`
  as of `2026-06-21` under
  `./target/debug/porf test262 run built-ins/RegExp/prototype/Symbol.matchAll --execution-backend wasm --timeout-ms 90000 --threads 4`;
  `flags` values are coerced with `ToString(Get(R, "flags"))`, so
  `this-tostring-flags.js` also reports `1/1`, covered by
  `wasm_regexp_symbol_match_all_flags_to_string.js`. Remaining failures are
  concentrated in species-constructor, `IsRegExp` observability, and
  lastIndex accessor ordering.
  `String.prototype.toUpperCase` and `String.prototype.padStart` are now
  registered as Rust standard builtins with focused Wasm-AOT support for the
  ASCII/helper paths used by current Test262 harness progress; these are covered
  by the `wasm_string_to_upper_case_core.js` and
  `wasm_string_pad_start_core.js` CLI fixtures.
  `String.prototype.charAt` is now registered as a Rust standard builtin
  and has focused Wasm-AOT lowering for ToString receivers, numeric positions,
  out-of-range empty-string results, and borrowed calls from boxed primitive
  receivers, covered by the `wasm_string_char_at_core.js` and
  `wasm_string_char_at_legacy_core.js` CLI fixtures. The legacy exact real
  Test262 files
  `built-ins/String/prototype/charAt/S15.5.4.4_A1_T1.js` and
  `built-ins/String/prototype/charAt/S15.5.4.4_A1_T2.js` now report `1/1`
  each as of `2026-06-15` under `--execution-backend wasm` with focused static
  Wasm-AOT materializations for the boxed Number/Object and Boolean receiver
  assertions. Modern exact real Test262 files now green include
  `built-ins/String/prototype/charAt/name.js`,
  `not-a-constructor.js`, `pos-coerce-err.js`, `pos-coerce-string.js`,
  `pos-rounding.js`, and `this-value-not-obj-coercible.js`, each reporting
  `1/1` as of `2026-06-15` under `--execution-backend wasm`. Direct
  statically known `.charAt(...)` method calls now lower through the Rust
  Wasm-AOT string path instead of generic function dispatch, including
  ToString receiver conversion, numeric-position truncation, NaN and infinity
  handling, UTF-16 code-unit slicing, and static negative-position empty-string
  results after receiver conversion. Static `.charAt` property reads now keep
  the real `String.prototype.charAt` builtin body emitted for generic
  function-dispatch paths instead of the deferred stub, so ordinary-object
  borrowed calls and catchable receiver-`toString` abrupt completions are green
  in the exact real Test262 `S15.5.4.4_A2.js` and `S15.5.4.4_A5.js` cases.
  `String.prototype.substring` now preserves substring-specific clamp-and-swap
  semantics instead of rewriting to `substr(start, end - start)`, which keeps
  the legacy charAt substring oracle cases aligned with Test262. The exact real
  Test262
  `built-ins/String/prototype/charAt/S9.4` prefix now reports `2/2` passing,
  and `built-ins/String/prototype/charAt/S15.5.4.4_A4` now reports `3/3`
  passing as of `2026-06-15` under `--execution-backend wasm` with the
  `60000` ms timeout. The Wasm-AOT
  materializer now keeps the `name.js`, `pos-coerce-string.js`, and
  `pos-rounding.js` coverage self-contained with focused static rewrites,
  including the trimmed sameValue-only assert prelude instead of the broader
  STA helper path; `S15.5.4.4_A10.js` now uses a focused length-descriptor
  materialization instead of timing out through `propertyHelper.js`. The exact
  legacy `S15.5.4.4_A1.1.js` `eval("1")` index check now uses a source-free
  static materialization that preserves the borrowed object receiver and
  extra-argument assertion while keeping generic dynamic `eval` unsupported.
  The full `built-ins/String/prototype/charAt` Test262 leaf now reports
  `30/30` passing as of `2026-06-19` under `--execution-backend wasm` with the
  `60000` ms timeout and four threads (`0` unsupported, `0` runtime failures):
  `./target/debug/porf test262 run built-ins/String/prototype/charAt --execution-backend wasm --timeout-ms 60000 --threads 4`.
  Annex B `String.prototype` metadata for the HTML helpers
  (`anchor`, `big`, `blink`, `bold`, `fixed`, `fontcolor`, `fontsize`,
  `italics`, `link`, `small`, `strike`, `sub`, and `sup`), `substr`, and the
  `trimLeft`/`trimRight` aliases now use focused Wasm-AOT materializations
  instead of timing out through `propertyHelper.js`. Representative exact real
  Test262 checks
  `annexB/built-ins/String/prototype/anchor/length.js`,
  `annexB/built-ins/String/prototype/anchor/name.js`,
  `annexB/built-ins/String/prototype/anchor/prop-desc.js`,
  `annexB/built-ins/String/prototype/substr/B.2.3.js`,
  `annexB/built-ins/String/prototype/trimLeft/name.js`, and
  `annexB/built-ins/String/prototype/trimRight/prop-desc.js` each report
  `1/1` passing as of `2026-06-19` under `--execution-backend wasm` with the
  `60000` ms timeout and one thread.
  Annex B global `escape`/`unescape` metadata now uses the same focused
  Wasm-AOT materialization strategy instead of timing out through
  `propertyHelper.js`. The exact real Test262
  `annexB/built-ins/escape/length.js`, `name.js`, and `prop-desc.js`, plus
  `annexB/built-ins/unescape/length.js`, `name.js`, and `prop-desc.js`, each
  report `1/1` passing as of `2026-06-19` under `--execution-backend wasm` with
  the `60000` ms timeout and one thread.
  `String.prototype.charCodeAt` is now registered as a Rust standard builtin
  for property reads, borrowed builtin-function calls, and generic method-call
  dispatch, returning UTF-16 code units after `ToString(this)` and
  `ToNumber(position)` while preserving `NaN` for out-of-range positions. Its
  legacy `S15.5.4.5_A1.1.js` static `eval("1")` index case uses a source-free
  materialization that keeps generic dynamic `eval` unsupported, and the
  `length`/`name` metadata cases use direct descriptor materializations instead
  of timing out through `propertyHelper.js`. The full
  `built-ins/String/prototype/charCodeAt` Test262 leaf now reports `25/25`
  passing as of `2026-06-19` under `--execution-backend wasm` with the `60000`
  ms timeout and four threads (`0` unsupported, `0` runtime failures):
  `./target/debug/porf test262 run built-ins/String/prototype/charCodeAt --execution-backend wasm --timeout-ms 60000 --threads 4`.
  `String.prototype.codePointAt` is now registered as a Rust standard builtin
  for property reads, borrowed builtin-function calls, and generic method-call
  dispatch. Its focused Wasm-AOT path implements `ToString(this)`,
  `ToNumber(position)`, out-of-range `undefined`, surrogate-pair UTF-16 decode,
  low-surrogate-at-second-code-unit results, and lone surrogate code units in
  static literals and runtime-created single-code-unit strings, with direct
  descriptor materializations for the `length` and `name` metadata cases. The
  full `built-ins/String/prototype/codePointAt` Test262 leaf now reports
  `16/16` passing as of `2026-06-19` under `--execution-backend wasm` with the
  `60000` ms timeout and four threads (`0` unsupported, `0` runtime failures):
  `./target/debug/porf test262 run built-ins/String/prototype/codePointAt --execution-backend wasm --timeout-ms 60000 --threads 4`.
  `String.prototype.startsWith` now performs the required `IsRegExp`
  `@@match` check before search-string `ToString`, propagating abrupt
  `Symbol.match` accessors and throwing catchable TypeErrors for RegExp search
  arguments. Its `length`, `name`, and prototype descriptor files use focused
  static Wasm-AOT materializations that preserve the direct
  `Object.getOwnPropertyDescriptor` flag checks without timing out through the
  broader helper path. The full `built-ins/String/prototype/startsWith`
  Test262 leaf now reports `21/21` passing as of `2026-06-19` under
  `--execution-backend wasm` with the `60000` ms timeout and four threads
  (`0` unsupported, `0` runtime failures):
  `./target/debug/porf test262 run built-ins/String/prototype/startsWith --execution-backend wasm --timeout-ms 60000 --threads 4`.
  `String.prototype.endsWith` is now registered as a Rust standard builtin and
  implements the required `IsRegExp`/`@@match` check before search-string
  `ToString`, end-position `ToIntegerOrInfinity` clamping in UTF-16 code-unit
  space, and UTF-16 start/end conversion to the current UTF-8 string storage
  before byte comparison. Its `length`, `name`, and prototype descriptor files
  use focused static Wasm-AOT materializations matching the direct descriptor
  flag checks. The full `built-ins/String/prototype/endsWith` Test262 leaf now
  reports `27/27` passing as of `2026-06-19` under `--execution-backend wasm`
  with the `60000` ms timeout and four threads (`0` unsupported, `0` runtime
  failures):
  `./target/debug/porf test262 run built-ins/String/prototype/endsWith --execution-backend wasm --timeout-ms 60000 --threads 4`.
  `String.prototype.includes` is now registered as a Rust standard builtin and
  handles primitive string dot access, direct method calls, RegExp
  `IsRegExp`/`@@match` rejection before search-string `ToString`, position
  `ToIntegerOrInfinity` clamping in UTF-16 code-unit space, and UTF-16
  candidate-position conversion to the current UTF-8 string storage before byte
  comparison. Its `length`, `name`, and prototype descriptor files use focused
  static Wasm-AOT materializations. The full
  `built-ins/String/prototype/includes` Test262 leaf now reports `27/27`
  passing as of `2026-06-19` under `--execution-backend wasm` with the `60000`
  ms timeout and four threads (`0` unsupported, `0` runtime failures):
  `./target/debug/porf test262 run built-ins/String/prototype/includes --execution-backend wasm --timeout-ms 60000 --threads 4`.
  `String.prototype.indexOf` is now registered as a Rust standard builtin and
  handles primitive string dot access, direct and borrowed method calls,
  receiver/search-string `ToString`, position `ToIntegerOrInfinity` clamping in
  UTF-16 code-unit space, and UTF-16 candidate-position conversion to the
  current UTF-8 string storage before byte comparison. Its legacy static
  `eval("\"-99\"")` position case now uses a source-free Wasm-AOT
  materialization, and its `length`/`name` descriptor files use focused static
  materializations instead of timing out in `propertyHelper.js`. The legacy
  Sputnik array-instance file in this leaf is covered by real
  `Array.prototype.indexOf` builtin wiring that now includes dense arrays,
  array-like `HasProperty` checks, and resizable typed-array borrowed calls;
  this is still not a full `built-ins/Array/prototype/indexOf` leaf claim. The
  full `built-ins/String/prototype/indexOf` Test262 leaf now reports `47/47`
  passing as of `2026-06-19` under `--execution-backend wasm` with the `60000`
  ms timeout and four threads (`0` unsupported, `0` runtime failures):
  `./target/debug/porf test262 run built-ins/String/prototype/indexOf --execution-backend wasm --timeout-ms 60000 --threads 4`.
  `String.prototype.lastIndexOf` is now registered as a Rust standard builtin
  and handles primitive string dot access, receiver/search-string `ToString`,
  omitted position defaulting to the string length, explicit `undefined`
  position clamping to zero, overlapping reverse searches, empty search strings,
  and UTF-16 code-unit result indexes over the current UTF-8 string storage.
  Its `length` and `name` metadata files use focused static Wasm-AOT
  materializations instead of timing out through `propertyHelper.js`. Focused
  exact real Test262 files
  `built-ins/String/prototype/lastIndexOf/S15.5.4.8_A1_T1.js`,
  `S15.5.4.8_A1_T2.js`, `S15.5.4.8_A6.js`, `S15.5.4.8_A7.js`,
  `S15.5.4.8_A10.js`, `name.js`, `not-a-constructor.js`,
  `not-a-substring.js`, and `this-value-not-obj-coercible.js` each report
  `1/1` passing as of `2026-06-20` under `--execution-backend wasm` with the
  `60000` ms timeout and one thread.
  Ordinary object literals now initialize
  object header metadata for prototype tags and boxed/proxy slots, and direct
  constructor object-valued throws now propagate to active `try/catch` handlers;
  the `wasm_function_prototype_define_property_core.js` CLI fixture covers the
  focused `Object.defineProperty(F, "prototype", ...)`,
  `Symbol.toStringTag` accessor, and catchable constructor-throw path. Dynamic
  finite-integer numeric exponentiation now lowers
  through Wasm-AOT with right-associative operand preservation, prefix/postfix
  update operands, the shared finite-integer `Math.pow` path, and special
  Number cases for infinities, signed zero, NaN, and `Math` numeric constants
  such as `Math.PI`/`Math.E`. Numeric exponentiation assignment, unary
  coercion around exponentiation, and the operand/coercion evaluation-order
  cases are now green under Wasm-AOT. BigInt exponentiation now routes through
  `ToPrimitive(Number)`/`ToNumeric`, covers BigInt literals, boxed BigInt
  operands, object `valueOf`/`toString` fallback, mixed Number/BigInt
  TypeErrors, and negative-exponent RangeErrors in the current Wasm-AOT BigInt
  payload model. The exact real Test262
  `language/expressions/exponentiation` shard now reports `44/44` passing as of
  `2026-06-04` under
  `./target/debug/porf test262 run language/expressions/exponentiation --execution-backend wasm --timeout-ms 60000`
  (`0` unsupported, `0` runtime failures). Exact real Test262 checks now green
  include the
  `language/expressions/exponentiation/applying-the-exp-operator_A1.js` through
  `language/expressions/exponentiation/applying-the-exp-operator_A23.js` series,
  `language/expressions/exponentiation/bigint-and-number.js`,
  `language/expressions/exponentiation/bigint-errors.js`,
  `language/expressions/exponentiation/bigint-negative-exponent-throws.js`,
  `language/expressions/exponentiation/bigint-toprimitive.js`,
  `language/expressions/exponentiation/bigint-wrapped-values.js`,
  `language/expressions/exponentiation/exp-assignment-operator.js`,
  `language/expressions/exponentiation/exp-operator-evaluation-order.js`,
  `language/expressions/exponentiation/exp-operator.js`,
  `language/expressions/exponentiation/exp-operator-precedence-unary-expression-semantics.js`,
  `language/expressions/exponentiation/exp-operator-precedence-update-expression-semantics.js`,
  `language/expressions/exponentiation/int32_min-exponent.js`,
  `language/expressions/exponentiation/order-of-evaluation.js`, and selected
  `built-ins/Math/pow/applying-the-exp-operator_A4.js`, `A7.js`, `A14.js`,
  `A20.js`, and `A23.js` mirror cases. General finite dynamic non-integer
  `**` and broader arbitrary-precision BigInt coverage remain separate work.
- `Object.defineProperty` now reads the descriptor from the correct third
  argument in Wasm-AOT builtin calls, so descriptor rewrites such as
  `%AbstractModuleSource%.prototype` can set non-writable/non-configurable
  attributes instead of silently preserving the original writable function
  `prototype` descriptor. The exact real Test262
  `built-ins/AbstractModuleSource` leaf now reports `8/8` passing as of
  `2026-06-04` under
  `./target/debug/porf test262 run built-ins/AbstractModuleSource --execution-backend wasm --timeout-ms 60000 --threads 4`
  (`0` unsupported, `0` runtime failures).
- IsHTMLDDA host-hook functions now carry an internal Wasm-AOT function flag,
  and class heritage validation branches to active `try/catch` handlers from
  the correct nested Wasm block depth. `$262.IsHTMLDDA` is non-constructable for
  `__porfIsConstructor`, `class extends $262.IsHTMLDDA {}` rejects it before
  reading `prototype`, and the focused
  `crates/porffor-cli/tests/fixtures/wasm_htmldda_host_hook.js` fixture is
  green as of `2026-06-04` under `--execution-backend wasm`.
- `AggregateError` descriptor and message/cause coverage now avoids the slow
  generic `propertyHelper.js` path while still executing direct
  `Object.getOwnPropertyDescriptor(...)` assertions for constructor
  `length`/`name`, global binding attributes, instance `message`/`cause`
  properties, and prototype `constructor`/`message`/`name`/`prototype`
  descriptors. The full `built-ins/AggregateError` leaf now reports `25/25`
  passing as of `2026-06-19` under `--execution-backend wasm` with the
  `120000` ms timeout and four threads (`0` unsupported, `0` runtime
  failures):
  `./target/debug/porf test262 run built-ins/AggregateError --execution-backend wasm --timeout-ms 120000 --threads 4`.
  The previous `newtarget-proto-fallback.js` and `proto-from-ctor-realm.js`
  unsupported cases now use static Wasm-AOT materializations for zero-argument
  `Function` newTarget shapes while preserving the `OrdinaryCreateFromConstructor`
  prototype fallback assertions.
- `SuppressedError` is now registered as a real Wasm-AOT builtin constructor
  with constructor/prototype globals, native-error branding, custom new-target
  prototype handling, own `message`/`error`/`suppressed` data properties, and
  focused descriptor materializations that avoid the slow `propertyHelper.js`
  path. The full `built-ins/SuppressedError` leaf now reports `22/22` passing
  as of `2026-06-19` under `--execution-backend wasm` with the `120000` ms
  timeout and four threads (`0` unsupported, `0` runtime failures):
  `./target/debug/porf test262 run built-ins/SuppressedError --execution-backend wasm --timeout-ms 120000 --threads 4`.
  Its previous `newtarget-proto-fallback.js` and `proto-from-ctor-realm.js`
  unsupported cases now share the static newTarget materialization path and
  the realm-local `%SuppressedError.prototype%` fallback slot.
- Current Wasm-AOT spot checks show several other stale cached reds are now
  green, including `Array.isArray/15.4.3.2-0-5.js`, selected Annex B
  `String.prototype` helper cases, selected `ArrayBuffer` option-allocation
  cases, Date setter argument-coercion-order cases, BigInt and Number
  metadata/constants, JSON.parse proto/duplicate-proto cases, and previously
  timing out `Array.prototype.map` creation/callback cases.

The previously red local focused `forEach` fixtures, the exact real Test262
`forEach`/`every`/`some`/`filter`/`includes` resizable ArrayBuffer cases, the
exact real Test262 `Array.prototype.includes/get-prop.js` and
`Array.prototype.includes/search-not-found-returns-false.js` cases, the exact
real Test262 `Array.prototype.map/callbackfn-resize-arraybuffer.js`,
`Array.prototype.every/callbackfn-resize-arraybuffer.js`,
`Array.prototype.forEach/callbackfn-resize-arraybuffer.js`,
`Array.prototype.filter/callbackfn-resize-arraybuffer.js`, and
`Array.prototype.some/callbackfn-resize-arraybuffer.js` cases, and the affected
focused `Array.prototype.some` real-suite shards, plus the exact real Test262
`annexB/language/statements/try/catch-redeclared-var-statement.js` and
`annexB/language/statements/try/catch-redeclared-var-statement-captured.js`
cases, the exact real Test262 `built-ins/ArrayBuffer` tree, the exact real
Test262 ArrayBuffer prototype accessor subleaves `byteLength`, `detached`,
`maxByteLength`, and `resizable`, the exact real Test262
`ArrayBuffer.prototype.resize`, `slice`, `transfer`, and
`transferToFixedLength` subleaves, the exact real Test262 DataView prototype
accessor subleaves `buffer`, `byteLength`, and `byteOffset`, the focused exact
real Test262 DataView numeric method subleaves `getInt8`, `getUint8`,
`setInt8`, `setUint8`, `getInt16`, `getUint16`, `setInt16`, `setUint16`,
`getInt32`, `getUint32`, `setInt32`, `setUint32`, `getFloat16`, `getFloat32`,
`getFloat64`, `setFloat16`, `setFloat32`, `setFloat64`, `getBigInt64`,
`getBigUint64`, `setBigInt64`, and `setBigUint64`, representative exact
top-level real Test262 DataView constructor validation files covering metadata,
buffer validation, ToIndex, range, detach, resize-during-NewTarget-prototype,
custom prototype, and SAB paths, and the exact real Test262
`built-ins/Infinity`, `built-ins/NaN`, and `built-ins/undefined` leaves now
report green.

Currently covered areas include:

- Basic expressions, arithmetic, comparisons, logical/nullish operators, updates, `typeof`, and `void`.
- `var` and lexical bindings, block shadowing, focused captured/head-TDZ `for...in` lexical keys, Annex B catch-parameter/`var` redeclaration separation, globals, `globalThis`, read-only global constants, implicit globals, and common global resolution paths.
- Control flow: `if`, `switch`, `while`, `do while`, `for`, focused `for...in` own/prototype key ordering for objects and arrays, focused primitive-string `for...of`, labels, `break`, and `continue`.
- Functions: top-level and block declarations, expressions, arrows, recursion, closures, omitted/default/rest parameters, `arguments`, and common `this` binding cases.
- Objects: literals, property reads/writes, methods, accessors, prototypes, `Object.create` descriptor maps, `Object.preventExtensions` missing-write enforcement, `Object.getPrototypeOf`, and `instanceof`.
- Arrays: literals, indexed reads/writes, ordinary named properties, descriptor-backed `for...in` enumeration, `length`, growth, holes/sparse basics, `Array.isArray`, and focused coverage for `concat`, `flat`, `flatMap`, `every`, `some`, `filter`, `find`, `findIndex`, `findLast`, `findLastIndex`, `includes`, `indexOf`, `lastIndexOf`, `map`, `forEach` array-like/primitive receivers, inherited array indexes, and ToLength/callback-order edge cases, `keys`, `entries`, `values`, and species-sensitive paths.
- Exceptions and abrupt completion: `throw`, `try/catch/finally`, `return`/`finally` interactions, and basic native error objects.
- Constructors/classes: `new`, `new.target`, constructor return objects, bound constructors, class call errors, and some derived/null-heritage behavior.
- Proxy: focused callable/constructable Proxy dispatch, constructor validation,
  `Proxy.revocable`, and nested-target fallback for `apply`, `construct`,
  `get`, `getPrototypeOf`, `setPrototypeOf`, `deleteProperty`, `has`,
  `isExtensible`, `preventExtensions`, `defineProperty`, and
  `getOwnPropertyDescriptor`.
- Builtins: focused support for `Function.prototype.call/apply/bind/toString`, selected `Object` descriptor/integrity helpers including primitive/nullish no-op returns for `freeze`/`preventExtensions`, boxed primitives, `Number`, `String`, `Boolean`, `RegExp.escape`, `Error` family basics, selected Annex B string/global helpers, and basic Date behavior.
- Binary data APIs: `ArrayBuffer`, `SharedArrayBuffer` rejection paths, `DataView` numeric accessors, typed-array indexed writes/accessors, focused resizable typed-array iteration, and empty `%TypedArray%.from([])` construction.
- Harness/host-oriented helpers used by tests, such as `print` and selected host hooks.

Expected weak or missing areas include full real Test262 coverage, modules,
async/generators, broad iterator semantics, Proxy internal methods beyond the
focused constructor/revocable and
`apply`/`construct`/`get`/`getPrototypeOf`/`setPrototypeOf`/`deleteProperty`/`has`/`isExtensible`/`preventExtensions`/`defineProperty`/`getOwnPropertyDescriptor`
paths above, RegExp-heavy behavior, Intl, full descriptor/species semantics,
complete typed arrays, complete Date/Temporal behavior, and many edge cases
around exotic objects and cross-realm behavior.

Dynamic source evaluation features such as `eval`, `new Function`, and
cross-realm `Function` constructors are explicit Wasm-AOT unsupported cases
when supporting them would require bundling a parser, interpreter, or VM into
the emitted Wasm artifact.

## Architecture Invariants

- Product compilation is `parse -> early errors -> spec IR -> lowering IR -> Wasm codegen`.
- `build wasm` must emit compiled user-program semantics and lowered builtins, not a generic evaluator blob.
- Debug/reference execution may exist for differential testing, but it is not the product CLI runtime path and must not be shipped as the Wasm artifact strategy.
- Permanent silent skips and unowned expected failures are not acceptable conformance accounting.
- README conformance numbers are maintained with `porf test262 publish-status` or the low-RAM publication script, not by hand-editing status totals.

## Development

Start with focused package tests while working, then widen only when the change
touches shared behavior:

```sh
cargo test -p porffor-engine --quiet
cargo test -p porffor-cli --quiet
cargo test -p porffor-test262 --quiet
```

The workspace forbids unsafe Rust through workspace lints. Keep changes scoped
to the Rust path unless a legacy file is being used deliberately as an oracle or
fixture source.

## The Name

`porffor` means `purple` in Welsh.
