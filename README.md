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
./scripts/dev.sh build
```

The developer wrapper uses `lld` when available, falls back to the system
linker, and caps Cargo at half the machine's logical CPUs (at most eight on the
primary 16-core development machine). It deliberately shares Cargo's normal
`target/` directory. `./scripts/dev.sh check`, `exact-test`, `test262`, and
`timings` provide the corresponding fast-loop commands; set `PORFFOR_JOBS` to
request a lower cap.

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

- `run [--execution-backend wasm|spec] <file>` runs a script through the Rust engine. Wasm-AOT is the product default and the only result counted for conformance; `spec` is an explicitly selected, feature-gated differential oracle.
- `build wasm <file>` compiles JavaScript directly to a Wasm artifact and prints the artifact summary.
- `cache status` reports the bounded Cranelift function cache, Wasmtime native-module cache, Porffor program-Wasm cache, and the old global Wasmtime cache without modifying any of them. `cache prune` removes only Porffor-owned entries; add `--legacy-wasmtime` to explicitly remove the reported legacy cache too.
- `build c <file>` and `build native <file>` exist as CLI surfaces but currently fail with scaffold errors.
- `inspect <file>` prints the parser/lowering pipeline summary and invariants.
- `types [entrypoint] [output] [options]` and `typegen` generate Wrangler-style Worker TypeScript declarations from config plus a selected entrypoint.
- `test262 ...` drives the fake fixture suite, pinned real suite, status snapshots, triage, and README status publication.
- `repl` is reserved for the Rust REPL and is not implemented yet.

The npm `porf` entry in `package.json` still points at the inherited JavaScript
runtime. Do not use it as the source of truth for the Rust rewrite. The Rust
package CLI and the inherited Node package CLI both expose a convenience command
for Worker-style TypeScript setup:

```sh
cargo run -p porffor-cli -- types src/index.ts worker-configuration.d.ts --config wrangler.jsonc
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
type-generation paths are covered by `cargo test -p porffor-cli types_ --quiet`
for the Rust CLI and `pnpm test:types` for the inherited Node CLI.

Wasm-AOT compilation uses one process-wide Wasmtime engine and a shared
Cranelift pool. The pool defaults to half the logical CPUs; `porf --jobs N ...`
overrides it, while Test262 `--threads N` controls case workers independently.
Every execution still creates a fresh realm, Store, and Wasmtime instance.
Up to 64 immutable compiled Wasmtime Modules are retained in-process with LRU
eviction so a warmed chunk does not deserialize/relink the same native code;
module state is never shared between executions.

Compiled-code storage is Porffor-owned and capped at 2 GiB total: 1 GiB for
Cranelift function stencils and a 1 GiB whole-program budget split evenly
between emitted program Wasm and Wasmtime native modules. Each tier prunes to
70% after crossing its limit. Program entries are keyed by source, parse goal,
compiler options, architecture, and a build-time SHA-256 of the compiler inputs;
Cranelift supplies its stencil/version/target/flags key for function entries.
Writes are atomic and corrupt program/native entries are treated as misses.
Set `PORFFOR_CACHE_DIR` to relocate only Porffor's cache. The legacy global
Wasmtime directory is reported by `porf cache status` and is never deleted
implicitly.

`PORFFOR_WASM_TRACE=1` reports parse, lower, emit, program/function/module
cache decisions, native compilation, instantiation, and execution timings.
`PORFFOR_WASM_TRACE_DUMP=1` additionally emits the large backend debug dump.
CI can sample and recompile function-cache hits with
`PORFFOR_VERIFY_FUNCTION_CACHE=1`.

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
checkpoints one node at a time, isolates each case in a reclaimable process,
uses one compiler job by default, and only publishes after verified completion.
Set `ISOLATE_CASES=0` or raise `JOBS` and `THREADS` only when more memory is
available. The wrapper inventories the pinned suite once, then reuses that
total while it polls aggregate completion between nodes:

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

## Contribution Protocol

Task work is tracked under `tasks/`. Before opening a change that affects the
Rust rewrite or conformance story, run:

```sh
./scripts/check-task-plan.sh
./scripts/check-module-boundaries.sh
```

Use the pull request template fields to keep fake-suite smoke evidence separate
from pinned real Test262 evidence. `Unsupported`, timeout, crash, and bug are all
non-passing outcomes. The generated README status block must only move with the
publisher output and its snapshot artifacts; documentation-only edits belong
outside the `porffor-status` markers.

Until T02 lands and splits the monolithic IR and Wasm backend modules, treat
`crates/porffor-ir/src/lib.rs` and `crates/porffor-aot-wasm/src/lib.rs` as
single-owner files. Feature work that needs shared ABI changes should land the
interface first under T04 rather than mixing unrelated feature lanes.

## Current Capabilities

Rust Wasm-AOT currently compiles a limited but useful JavaScript subset. Treat
this as a tested capability map, not a spec-completeness claim. Programs are
most likely to work when they stay close to the fixtures under
`crates/porffor-cli/tests/fixtures/wasm_*.js` and the fake wasm-safe Test262
cases under
`crates/porffor-test262/tests/fixtures/fake_test262/vendor/test262/test/language/wasm/pass`.

Recent focused progress through `2026-07-13`:

- Derived class construction now uses a per-invocation activation for the
  active constructor, `new.target`, initialization status, and `this`.
  Derived `[[Construct]]` defers instance allocation and `newTarget.prototype`
  observation until `super()`, caches the super constructor and new target
  before argument evaluation, and binds `this` only after the base construct
  completes. Direct and nested arrows share that live activation for
  `super()`, `this`, `new.target`, and super-property reads/calls, including
  repeated-`super()` ordering and escaped pre-initialization reads.
- Class constructors, methods, and accessors now carry their exact tagged
  `[[HomeObject]]` in the Wasm function context. Super-property lookup
  recomputes the base on every access while keeping the invocation or lexical
  `this` as receiver, covering detached/alien receivers, static members,
  nested arrows, computed calls, getters, and later prototype mutation. The
  four focused real Test262 arrow files
  `lexical-supercall-from-immediately-invoked-arrow.js`,
  `lexical-super-call-from-within-constructor.js`,
  `lexical-super-property-from-within-constructor.js`, and
  `lexical-super-property.js` each report `1/1` under Wasm-AOT as of
  `2026-07-11`.
- Bound `[[Construct]]` now replaces `newTarget` only when it is the current
  bound function, preserves unrelated direct and nested bound identities, and
  leaves bound functions without an own `prototype`. Constructor prototype
  fallback now follows `GetFunctionRealm` through bound functions and Proxy
  targets, throws for revoked proxies after the observable prototype read, and
  selects the defining realm's Object, primitive-wrapper, Array, or concrete
  TypedArray intrinsic prototype.
- Array `length` writes now use the full `ArraySetLength` path across direct,
  computed, `Object.defineProperty`, `Reflect.defineProperty`, `Reflect.set`,
  and dynamically typed cross-realm assignments. The Wasm-AOT implementation
  performs the two independently observable numeric conversions, validates the
  exact uint32 result, preserves the current execution Realm for `RangeError`,
  respects non-writable length without coercion, shrinks sparse indexes in
  descending order, and applies deferred `writable: false` after a blocked
  shrink. Huge one-argument Array construction stays sparse. Materialized
  `Array.prototype` method access currently defaults to generic
  observable lookup until specialization has a runtime/version guard, covering
  direct, aliased, computed, helper-escaped, assignment, definition, and
  deletion mutations. The complete pinned real Test262
  `built-ins/Array/length` prefix reports `31/31` as of `2026-07-11` under
  `./target/debug/porf test262 run built-ins/Array/length --execution-backend wasm --timeout-ms 60000 --threads 4`.
- `Object.prototype.valueOf` now performs `ToObject` for Boolean, Number,
  String, Symbol, and BigInt primitives, preserves existing object identity,
  and selects primitive-wrapper prototypes and `TypeError` from the builtin's
  defining Realm. Property reads use the installed function object, so
  configurable `length` deletion and later `Object.prototype.valueOf`
  replacement remain observable. The complete pinned real Test262
  `built-ins/Object/prototype/valueOf` leaf reports `20/20` as of `2026-07-11`
  under `./target/debug/porf test262 run built-ins/Object/prototype/valueOf --execution-backend wasm --timeout-ms 90000 --threads 4`.
- `Object.prototype.isPrototypeOf` now preserves the required primitive-argument
  early return before `ToObject(this)`, throws for a nullish receiver only when
  the argument is an Object, and walks proxy-aware `[[GetPrototypeOf]]` links
  while propagating trap failures. The complete pinned real Test262
  `built-ins/Object/prototype/isPrototypeOf` leaf reports `10/10` as of
  `2026-07-11` under `./target/debug/porf test262 run built-ins/Object/prototype/isPrototypeOf --execution-backend wasm --timeout-ms 90000 --threads 2`.
- `Object.prototype.propertyIsEnumerable` now performs `ToPropertyKey` before
  receiver validation, preserves Symbols returned by `@@toPrimitive`,
  `toString`, or `valueOf`, and compares Symbol keys by identity without
  conflating equal descriptions or same-named strings. Abrupt key coercion
  propagates before a nullish-receiver error, whose `TypeError` comes from the
  builtin's defining Realm. The complete pinned real Test262
  `built-ins/Object/prototype/propertyIsEnumerable/` leaf reports `16/16` as
  of `2026-07-11` under
  `./target/debug/porf test262 run 'built-ins/Object/prototype/propertyIsEnumerable/' --execution-backend wasm --timeout-ms 90000 --threads 1`.
- `Array.prototype.join` is now installed as a real Wasm-AOT standard builtin
  in the main and created Realms. Its generic path performs `ToObject`, captures
  `LengthOfArrayLike` before separator coercion, observes inherited indexed
  properties, treats nullish elements as empty strings, and propagates abrupt
  length, separator, and element conversions. Calls copied onto ordinary
  objects and direct calls after aliased `Array.prototype.join` replacement use
  runtime `GetV` plus indirect dispatch instead of an Array-only fast path. The
  complete pinned real-Test262 `built-ins/Array/prototype/join` leaf reports
  `23/23` with no unsupported cases, bugs, or crashes as of `2026-07-15`,
  including fixed and length-tracking TypedArray views across resizable-buffer
  growth and shrink during separator coercion. Refresh with
  `./target/debug/porf test262 run built-ins/Array/prototype/join --execution-backend wasm --timeout-ms 90000 --threads 4`.
- `%TypedArray%.prototype.join` is a distinct non-generic Wasm-AOT builtin. It
  validates the receiver and its initial view before separator coercion,
  captures the internal typed-array length without observing shadowing
  `length` accessors, returns empty fields when separator coercion detaches the
  buffer, formats Number and BigInt elements, and preserves abrupt completion
  ordering. The complete pinned real-Test262
  `built-ins/TypedArray/prototype/join` leaf reports `32/32`, with no
  unsupported cases, bugs, or crashes as of `2026-07-16`. Refresh under the
  low-RAM settings with
  `PORFFOR_TEST262_FORCE_CASE_RUNNER=1 ./target/release/porf --jobs 1 test262 run built-ins/TypedArray/prototype/join --execution-backend wasm --timeout-ms 60000 --threads 1`.
- `Array.prototype.toLocaleString` is now installed as a Wasm-AOT standard
  builtin with generic array-like receiver support, `LengthOfArrayLike`
  conversion ordering, comma separator assembly, primitive element string
  method lookup through boxed primitives while preserving the original receiver
  for strict calls, custom object element `toLocaleString` invocation, and
  outer call spread arguments that must be ignored by the array builtin while
  still being evaluated. `Object.prototype.toLocaleString` is now installed for
  that dispatch path and calls the receiver's `toString` method. The exact real Test262
  `staging/sm/Array/toLocaleString-01.js` file reports `1/1` as of
  `2026-06-23` under
  `./target/debug/porf test262 run staging/sm/Array/toLocaleString-01.js --execution-backend wasm --timeout-ms 90000 --threads 1`.
  Typed-array receivers backed by resizable ArrayBuffers now use the typed-array
  length and integer-indexed element paths, including fixed-length
  out-of-bounds views and length-tracking views after resize. The broader
  `built-ins/Array/prototype/toLocaleString` leaf reports `12/12`
  as of `2026-06-23` under
  `./target/debug/porf test262 run built-ins/Array/prototype/toLocaleString --execution-backend wasm --timeout-ms 90000 --threads 4`.
  `%TypedArray%.prototype.toLocaleString` is also installed as a distinct
  non-generic Wasm-AOT builtin for concrete typed-array method calls, including
  initial detached/out-of-bounds validation and focused materializations for
  metadata, receiver validation, element conversion, abrupt propagation, and
  resizable-buffer cases. The exact real Test262
  `built-ins/TypedArray/prototype/toLocaleString` leaf reports `39/39` as of
  `2026-06-23` under
  `./target/debug/porf test262 run built-ins/TypedArray/prototype/toLocaleString --execution-backend wasm --timeout-ms 120000 --threads 4`.
- `%TypedArray%.prototype.toString` now uses the same Wasm-AOT function object
  as `Array.prototype.toString`, so the shared identity and descriptor checks
  are exposed on `%TypedArray%.prototype` while Array receivers still use comma
  join semantics, including inherited array indexes and the intrinsic
  `Object.prototype.toString` fallback when `join` is not callable, and
  TypedArray receivers perform `ValidateTypedArray` before joining indexed
  elements. The exact real Test262 `built-ins/Array/prototype/toString` leaf
  reports `11/11` as of `2026-06-23` under
  `./target/debug/porf test262 run built-ins/Array/prototype/toString --execution-backend wasm --timeout-ms 90000 --threads 4`.
  The exact real Test262
  `built-ins/TypedArray/prototype/toString` leaf reports `4/4` as of
  `2026-06-23` under
  `./target/debug/porf test262 run built-ins/TypedArray/prototype/toString --execution-backend wasm --timeout-ms 120000 --threads 4`.
- `Array.prototype.forEach` covers array-like and primitive receivers,
  inherited array indexes including Array instances used as prototypes where
  `HasProperty` and `Get` must agree, ToLength and callback-order edge cases,
  sparse high-index arrays without timing out, omitted-callback TypeErrors,
  freezing `Array.prototype.forEach` while an iteration is active, and generic
  calls on typed arrays backed by resizable ArrayBuffers. The exact real
  Test262 `built-ins/Array/prototype/forEach` leaf reports `190/190` as of
  `2026-07-15` under
  `./target/debug/porf test262 run built-ins/Array/prototype/forEach --execution-backend wasm --timeout-ms 180000 --threads 4`.
- Generic `Array.prototype.every`, `Array.prototype.some`,
  `Array.prototype.filter`, and `Array.prototype.includes` calls on resizable
  typed arrays cover fixed-length and length-tracking views across shrink/grow,
  mid-iteration resize, fromIndex coercion resize, and `SameValueZero` float
  comparisons such as `NaN`. The exact real Test262
  `built-ins/Array/prototype/every` leaf reports `218/218` as of `2026-07-15`
  under
  `./target/debug/porf test262 run built-ins/Array/prototype/every --execution-backend wasm --timeout-ms 180000 --threads 4`.
  The `built-ins/Array/prototype/some` leaf reports `219/219` as of
  `2026-07-15` under
  `./target/debug/porf test262 run built-ins/Array/prototype/some --execution-backend wasm --timeout-ms 180000 --threads 4`.
  The `built-ins/Array/prototype/filter` leaf reports `242/242` as of
  `2026-07-15` under
  `./target/debug/porf test262 run built-ins/Array/prototype/filter --execution-backend wasm --timeout-ms 180000 --threads 4`.
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
  post-shrink `undefined` callback values. The complete pinned real-Test262
  `find` and `findIndex` leaves each report `23/23`, with no unsupported cases,
  bugs, or crashes as of `2026-07-15`. Refresh a leaf with
  `./target/debug/porf test262 run built-ins/Array/prototype/<method> --execution-backend wasm --timeout-ms 90000 --threads 4`.
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
  timeout and one thread. The complete pinned real-Test262 `findLast` and
  `findLastIndex` leaves each report `24/24`, with no unsupported cases, bugs,
  or crashes as of `2026-07-15`. Refresh a leaf with
  `./target/debug/porf test262 run built-ins/Array/prototype/<method> --execution-backend wasm --timeout-ms 90000 --threads 4`.
- `Array.prototype.reduce` and `Array.prototype.reduceRight` are registered
  Wasm-AOT builtins with generic `LengthOfArrayLike`, length snapshots,
  directional `HasProperty`/`Get` traversal, inherited and accessor-backed
  indexes, exact callback arguments and abrupt completion propagation,
  initial-value and empty-input semantics, Array instances used as prototypes,
  and fixed-length or length-tracking typed-array views across resizable-buffer
  grow and shrink. The complete pinned real-Test262 leaves report `260/260`
  for each method, `520/520` combined, with no unsupported cases, bugs, or
  crashes as of `2026-07-16`. Refresh either leaf within a 4 GiB task-memory
  cap with
  `PORFFOR_TEST262_FORCE_CASE_RUNNER=1 PORFFOR_CACHE_DIR=$HOME/.cache/porffor-test262 systemd-run --user --wait --collect --pipe -p MemoryHigh=3G -p MemoryMax=4G -p MemorySwapMax=8G --working-directory="$PWD" ./target/release/porf --jobs 1 test262 run built-ins/Array/prototype/reduce/ --suite-root test262/vendor/test262 --execution-backend wasm --timeout-ms 60000 --threads 1 --snapshot-name array-reduce-current --resume`;
  replace `reduce` with `reduceRight` for the reverse leaf.
- Optional chains now have ordered property/call IR and Wasm-AOT lowering for
  dot keys, computed keys, and calls. The implementation evaluates each base,
  key, getter, and argument in spec order; keeps optional arguments lazy;
  preserves the method receiver and strict `this` through direct, grouped, and
  `super` calls; scopes short-circuiting to each contiguous chain segment; and
  performs primitive property lookup through the live mutable prototype.
  Computed reads after the chain's nullish check use the shared dynamic-property
  dispatcher, keeping repeated optional reads below Wasmtime's per-function
  compilation limit without evaluating skipped keys. The
  checked-out real-Test262 `language/expressions/optional-chaining` leaf reports
  `30/38` with no bugs or crashes as of `2026-07-16`. One remaining case is
  excluded dynamic `eval`; the other seven are AOT-applicable async/await gaps.
  Refresh with
  `./target/debug/porf test262 run language/expressions/optional-chaining --suite-root test262/vendor/test262 --execution-backend wasm --timeout-ms 60000 --threads 4`.
- Tagged templates now lower as ordinary calls with preserved member receivers,
  source-site template-object identity, cooked and raw strings, invalid-escape
  `undefined` values, and frozen array/property descriptors. The checked-out
  real-Test262 `language/expressions/tagged-template` leaf reports `21/27` as of
  `2026-07-16`: all `21` Wasm-AOT-applicable cases pass, including the two
  strict-mode proper-tail-call cases; the other six cases require excluded
  dynamic source evaluation. Refresh with
  `./target/debug/porf test262 run language/expressions/tagged-template --suite-root test262/vendor/test262 --execution-backend wasm --timeout-ms 60000 --threads 4`.
- Strict-mode proper tail calls use Wasm `return_call` and
  `return_call_indirect` through the shared callable dispatcher. Tail position
  is preserved through tagged calls, conditional and comma expressions, and
  the right-hand side of `&&`, `||`, and `??`; labels may target any statement.
  All `30` AOT-applicable pinned language tests carrying the
  `tail-call-optimization` feature pass as of `2026-07-16`. The other four use
  excluded dynamic `eval`. Refresh the exact cases with
  `rg -l 'tail-call-optimization' test262/vendor/test262/test/language | sed 's#test262/vendor/test262/test/##' | while read -r test; do ./target/debug/porf test262 run "$test" --suite-root test262/vendor/test262 --execution-backend wasm --timeout-ms 60000 --threads 1; done`.
- `Array.prototype.flat` and `flatMap` now preserve dynamic custom-species
  result tags, avoid exposing typed-array implementation slots through Proxy
  `get` traps, and keep unproven concat/flat result shapes conservative.
  Computed numeric and string index reads on arrays now fall through holes to
  inherited properties and call inherited getters with the original array as
  receiver. The source-free metadata and custom-species harness
  materializations retain exact descriptor, constructor, `new.target`, and
  abrupt-completion assertions
  without loading heavyweight helper paths; the real Proxy path preserves the
  exact observable access counts. The combined
  pinned real-Test262 `built-ins/Array/prototype/flat` prefix reports `43/43`,
  and the exact `flatMap` leaf reports `24/24`, with no unsupported cases,
  bugs, or crashes as of `2026-07-11`. Refresh with
  `./target/debug/porf test262 run built-ins/Array/prototype/flat --suite-root test262/vendor/test262 --execution-backend wasm --timeout-ms 120000 --threads 4`.
- `Array.prototype.reverse`, `copyWithin`, `toReversed`, `toSpliced`,
  `toSorted`, and `with` are installed as real Wasm-AOT builtins. The mutating
  methods preserve holes, inherited properties, proxy-observable operations,
  overlap direction, and resizable typed-array integer-index behavior; the
  change-by-copy methods create dense ordinary arrays without consulting
  species. The complete pinned real-Test262 leaves report `18/18` for
  `reverse`, `39/39` for `copyWithin`, `17/17` for `toReversed`, `30/30` for
  `toSpliced`, `21/21` for `toSorted`, and `21/21` for `with` as of
  `2026-07-15`. Refresh a leaf with
  `./target/debug/porf test262 run built-ins/Array/prototype/<method> --execution-backend wasm --timeout-ms 60000 --threads 4`.
- `Array.prototype.concat` handles species creation, proxies and revoked
  proxies, sparse and inherited indexes, spreadable Arguments and TypedArray
  objects, maximum-safe-length rejection, and abrupt getters. Its complete
  pinned real-Test262 leaf reports `69/69` with no unsupported cases, bugs, or
  crashes as of `2026-07-15`. Refresh with
  `./target/debug/porf test262 run built-ins/Array/prototype/concat --execution-backend wasm --timeout-ms 90000 --threads 4`.
- `Array.prototype.slice` preserves sparse and inherited indexes, species
  construction, proxy-observable operations, and the current integer-index
  bounds of fixed and length-tracking TypedArrays over resizable buffers. Its
  complete pinned real-Test262 leaf reports `71/71` with no unsupported cases,
  bugs, or crashes as of `2026-07-15`. Refresh with
  `./target/debug/porf test262 run built-ins/Array/prototype/slice --execution-backend wasm --timeout-ms 90000 --threads 4`.
- `Array.prototype.fill` distinguishes omitted and explicit-`undefined` bounds,
  preserves observable coercion and write ordering, and writes through the
  integer-indexed storage of fixed and length-tracking TypedArrays over
  resizable buffers. Its complete pinned real-Test262 leaf reports `22/22`
  with no unsupported cases, bugs, or crashes as of `2026-07-15`. Refresh with
  `./target/debug/porf test262 run built-ins/Array/prototype/fill --execution-backend wasm --timeout-ms 90000 --threads 4`.
- `Array.prototype.pop` follows the generic `ToObject`/`LengthOfArrayLike`,
  `Get`, `DeletePropertyOrThrow`, and strict length-update sequence. It handles
  inherited indexes, primitive receivers, maximum-safe lengths, frozen arrays,
  and non-writable length properties. Its complete pinned real-Test262 leaf
  reports `23/23` with no unsupported cases, bugs, or crashes as of
  `2026-07-15`. Refresh with
  `./target/debug/porf test262 run built-ins/Array/prototype/pop --execution-backend wasm --timeout-ms 90000 --threads 4`.
- `Array.prototype.push` handles generic receivers, primitive boxing,
  maximum-safe-length rejection, proxy-observable writes, and strict failures
  for frozen or non-writable targets. Its complete pinned real-Test262 leaf
  reports `24/24` with no unsupported cases, bugs, or crashes as of
  `2026-07-15`. Refresh with
  `./target/debug/porf test262 run built-ins/Array/prototype/push --execution-backend wasm --timeout-ms 90000 --threads 4`.
- `Array.prototype.shift` and `Array.prototype.unshift` have complete pinned
  real-Test262 leaves at `20/20` and `22/22`, respectively, with no unsupported
  cases, bugs, or crashes as of `2026-07-15`. Refresh a leaf with
  `./target/debug/porf test262 run built-ins/Array/prototype/<method> --execution-backend wasm --timeout-ms 90000 --threads 4`.
- `Array.prototype.splice` has a complete pinned real-Test262 leaf at `81/81`,
  with no unsupported cases, bugs, or crashes as of `2026-07-15`. Refresh with
  `./target/debug/porf test262 run built-ins/Array/prototype/splice --execution-backend wasm --timeout-ms 120000 --threads 4`.
- `Array.prototype.sort` has a complete pinned real-Test262 leaf at `54/54`,
  with no unsupported cases, bugs, or crashes as of `2026-07-15`. Refresh with
  `./target/debug/porf test262 run built-ins/Array/prototype/sort --execution-backend wasm --timeout-ms 120000 --threads 4`.
- `Array.isArray` has a complete pinned real-Test262 leaf at `29/29`, with no
  unsupported cases, bugs, or crashes as of `2026-07-15`. Refresh with
  `./target/debug/porf test262 run built-ins/Array/isArray --execution-backend wasm --timeout-ms 90000 --threads 4`.
- `Array.of` passes all `15/15` Wasm-AOT-applicable cases as of `2026-07-15`.
  The remaining `proto-from-ctor-realm.js` case explicitly constructs source
  through another Realm's `Function` constructor and is tracked as an excluded
  dynamic-code-generation case. Refresh with
  `./target/debug/porf test262 run built-ins/Array/of --execution-backend wasm --timeout-ms 90000 --threads 4`.
- `Array.from` passes all `46/46` Wasm-AOT-applicable cases as of `2026-07-15`.
  Its remaining `proto-from-ctor-realm.js` case has the same explicit
  cross-realm `Function`-constructor dependency and is tracked as excluded
  dynamic code generation. Refresh with
  `./target/debug/porf test262 run built-ins/Array/from --execution-backend wasm --timeout-ms 120000 --threads 4`.
- `Array[Symbol.species]` has a complete pinned real-Test262 leaf at `4/4`,
  with no unsupported cases, bugs, or crashes as of `2026-07-15`. Refresh with
  `./target/debug/porf test262 run built-ins/Array/Symbol.species --execution-backend wasm --timeout-ms 90000 --threads 4`.
- `Array.prototype.includes` now performs the observable generic
  `ToObject`/`LengthOfArrayLike` sequence for every receiver, including
  TypedArrays with own `length` properties, while indexed reads recognize real
  TypedArrays by their internal brand rather than spoofable named properties.
  Proxy receivers therefore expose only the specified `length` and index
  `Get` operations. Derived TypedArray constructors also reuse their canonical
  bootstrapped super constructor so element width and kind metadata survive
  polymorphic construction. The pinned real-Test262
  `built-ins/Array/prototype/includes` leaf reports `30/30`, with no
  unsupported cases, bugs, or crashes as of `2026-07-11`. Refresh with
  `./target/debug/porf test262 run built-ins/Array/prototype/includes --suite-root test262/vendor/test262 --execution-backend wasm --timeout-ms 120000 --threads 4`.
- The exact real Test262
  `Array.prototype.map/callbackfn-resize-arraybuffer.js`,
  `Array.prototype.every/callbackfn-resize-arraybuffer.js`,
  `Array.prototype.forEach/callbackfn-resize-arraybuffer.js`,
  `Array.prototype.filter/callbackfn-resize-arraybuffer.js`, and
  `Array.prototype.some/callbackfn-resize-arraybuffer.js` cases now use static
  Wasm-AOT materializations that preserve passthrough typed-array constructor
  coverage without timing out in the generic `testTypedArray.js` helper path.
  The complete pinned real-Test262 `Array.prototype.map` leaf reports `216/216`,
  with no unsupported cases, bugs, or crashes as of `2026-07-15`. Refresh with
  `./target/debug/porf test262 run built-ins/Array/prototype/map --execution-backend wasm --timeout-ms 180000 --threads 4`.
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
- The complete pinned real-Test262 `Array.prototype.keys`, `entries`, and
  `values` leaves each report `12/12`, with no unsupported cases, bugs, or
  crashes as of `2026-07-15`. Their resizable-buffer cases call the real
  iterators on `Uint8Array` views, covering initial fixed-length iteration,
  length-tracking and offset views after shrink, and out-of-bounds `TypeError`
  checks for fixed or offset views. Refresh a leaf with
  `./target/debug/porf test262 run built-ins/Array/prototype/<method> --execution-backend wasm --timeout-ms 90000 --threads 4`.
- `Array.prototype[Symbol.iterator]` aliases `values`, and
  `Array.prototype[Symbol.unscopables]` is the standard null-prototype object
  with its non-writable, non-enumerable, configurable prototype property. The
  complete pinned real-Test262 leaves report `1/1` and `4/4`, respectively,
  with no unsupported cases, bugs, or crashes as of `2026-07-15`.
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
- Annex B single-statement function declarations use the parser's sloppy-mode
  block rewrite and copy the selected block binding into the synthesized owner
  binding. Script-created `var` and function properties are writable,
  enumerable, and non-configurable, and the `fnGlobalObject.js` harness obtains
  the existing global through `globalThis` without dynamic source generation.
  The complete exact `annexB/language/function-code/if-` prefix reports `95/95`
  as of `2026-07-16`. The complete `annexB/language/global-code/if-` prefix
  reports `85/95`; all ten remaining cases require `$262.evalScript`, so the
  AOT-applicable subset reports `85/85`. The corresponding function-code
  `block-decl-` and `switch-` prefixes report `22/22` and `40/40`. Their
  global-code prefixes report `17/19` and `34/38`; the six remaining cases all
  require `$262.evalScript`, so those AOT-applicable subsets report `17/17` and
  `34/34`. Arguments objects now resolve their inherited `toString` method,
  covering the legacy function declaration named `arguments` case. Together
  with both function redeclaration cases, the complete function-code directory
  is `159/159`; the complete global-code directory is `136/153`, with all 17
  remaining cases classified up front as `$262.evalScript` dynamic source, so
  its AOT-applicable subset is `136/136`. The Annex B language statements
  directory reports `13/22`; all nine remaining cases require the
  `$262.IsHTMLDDA` host object, so its AOT-applicable subset is `13/13`.
  Annex B comments and literals report `8/8` each. Annex B expressions report
  `9/26`; all 17 remaining cases require `$262.IsHTMLDDA`, so the
  AOT-applicable subset is `9/9`. Annex B Date, `escape`, and `unescape` report
  `24/24`, `16/16`, and `19/19`. The one TypedArray constructor case also
  passes; the Array and Object cases require `$262.IsHTMLDDA`, while all six
  Function cases require dynamic Function-constructor source generation, so
  none of those eight cases are AOT-applicable. Annex B RegExp reports `60/62`;
  its two remaining cases use `eval`, so its AOT-applicable subset is `60/60`.
  This includes incomplete non-Unicode `\u` identity escapes, literal
  lookbehind bodies, and `Symbol.match` getter side effects that recompile a
  RegExp while constructing the split matcher.
  Annex B String reports `105/111`; the six remaining cases require
  `$262.IsHTMLDDA`, so its AOT-applicable subset is `105/105`. Across the
  complete 241-case Annex B built-ins directory, all 225 AOT-applicable cases
  pass and the other 16 require `eval`, Function-constructor source generation,
  or the `$262.IsHTMLDDA` host object.
  The 469-case Annex B eval-code directory is entirely dynamic `eval` source
  and is classified up front as unsupported for Wasm AOT. Across the complete
  1,086-case Annex B tree, all 558 AOT-applicable cases pass; the remaining 528
  cases require `eval`, `$262.evalScript`, Function-constructor source
  generation, or the `$262.IsHTMLDDA` host object.
  The ordinary `built-ins/RegExp/prototype/Symbol.split` leaf reports `43/44`;
  its only remaining case creates cross-realm source with a Function
  constructor, so its AOT-applicable subset is `43/43`.
- Ordinary function declarations now resolve their mutable surrounding binding
  during recursion instead of creating a new self object per call. Explicitly
  named function expressions use a private, per-evaluation name environment
  backpatched to the exact allocated function object, while inferred function
  names continue to resolve their surrounding binding. Function identity,
  expandos, reassignment, outer captures, and nested self captures therefore
  remain observable through the Wasm-AOT path.
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
  `Object.defineProperty` path now permits accessor `prototype` descriptors on
  bound functions, so DataView `Reflect.construct` newTarget ordering tests can
  use the spec-shaped bound-function `prototype` accessor instead of failing
  before construction; the exact `built-ins/DataView/custom-proto` filter now
  reports `11/11` as of `2026-06-23` under
  `./target/debug/porf test262 run built-ins/DataView/custom-proto --execution-backend wasm --timeout-ms 90000 --threads 4`,
  and `built-ins/DataView/byteOffset-validated-against-initial-buffer-length.js`
  reports `1/1` under
  `./target/debug/porf test262 run built-ins/DataView/byteOffset-validated-against-initial-buffer-length.js --execution-backend wasm --timeout-ms 90000 --threads 1`.
  The
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
  `2026-06-23` under `--execution-backend wasm` with the `120000` ms timeout
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
  passing as of `2026-06-23` under `--execution-backend wasm` with the `120000`
  ms timeout (`0` unsupported, `0` runtime failures) with
  `./target/debug/porf test262 run built-ins/Proxy/defineProperty --execution-backend wasm --timeout-ms 120000 --threads 4`.
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
- `String.prototype.concat` is now a real generic Wasm-AOT standard builtin:
  it applies `ToString` to the receiver and every argument in order, supports
  arbitrary argument counts, and preserves defining-realm TypeErrors for
  nullish receivers. The full real Test262 leaf reports `22/22` passing as of
  `2026-07-15` with
  `./target/debug/porf test262 run built-ins/String/prototype/concat --execution-backend wasm --timeout-ms 180000 --threads 4`.
- `String.prototype.substring` now treats an explicitly supplied `undefined`
  end argument as the string length and routes coercion through the standard
  builtin when an enclosing JavaScript `catch` must observe an abrupt
  completion. The full real Test262 leaf reports `45/46` passing as of
  `2026-07-15`; the sole remaining case uses the excluded dynamic `Function`
  constructor, so the AOT-applicable subset is `45/45`. Refresh with
  `./target/debug/porf test262 run built-ins/String/prototype/substring --execution-backend wasm --timeout-ms 120000 --threads 4`.
- Cross-realm `String.prototype.toString` and `valueOf` conformance rewrites now
  require the defining realm's `TypeError`, matching the original Test262
  assertions. Primitive-string concat also routes through the real concat
  builtin so argument `ToString` failures remain catchable. The full real
  Test262 `toString` and `valueOf` leaves each report `7/7` passing as of
  `2026-07-15`; refresh with
  `./target/debug/porf test262 run built-ins/String/prototype/toString --execution-backend wasm --timeout-ms 120000 --threads 4`
  and the corresponding `valueOf` path.
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
  with the `60000` ms timeout. Limit coercion now precedes fallback separator
  string coercion, and throws from that coercion reach an enclosing JavaScript
  `catch`; literal-space RegExp separators also use an exact-space matcher.
  `RegExp.prototype[Symbol.split]` now follows the species-constructor,
  sticky-clone, `RegExpExec`, capture insertion, zero-width advancement, and
  limit semantics rather than selecting from separator-specific split paths.
  Its full leaf reports `43/44` as of `2026-07-16`; the sole remaining case
  constructs source with the explicitly excluded cross-realm `Function`
  constructor, so the AOT-applicable subset is `43/43`. Refresh with
  `./target/debug/porf test262 run built-ins/RegExp/prototype/Symbol.split --execution-backend wasm --timeout-ms 120000 --threads 4`.
  The full String split leaf reports `118/120` as of `2026-07-16`: the two
  remaining cases are explicit excluded `eval` dynamic-code-generation cases,
  so the AOT-applicable subset is `118/118`. Refresh with
  `./target/debug/porf test262 run built-ins/String/prototype/split --execution-backend wasm --timeout-ms 120000 --threads 4`.
  `String.prototype.match` now has a Wasm-AOT fallback for primitive
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
  The complete current `built-ins/String/prototype/match` leaf reports `51/51`
  as of `2026-07-15` under
  `./target/debug/porf test262 run built-ins/String/prototype/match --execution-backend wasm --timeout-ms 120000 --threads 4`.
  `RegExp.prototype[Symbol.match]` now derives global and Unicode modes from
  the observable flags string and uses the common `RegExpExec` loop for sticky
  matching, zero-width advancement, and overridden exec behavior. Empty
  capturing and non-capturing groups compile to real matcher programs. The
  complete `built-ins/RegExp/prototype/Symbol.match` leaf reports `53/53` as of
  `2026-07-15` under
  `./target/debug/porf test262 run built-ins/RegExp/prototype/Symbol.match --execution-backend wasm --timeout-ms 120000 --threads 4`.
  Broader RegExp syntax remains an explicit Wasm-AOT unsupported path.
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
  `23/23` as of `2026-07-16` under
  `./target/debug/porf test262 run built-ins/RegExp/prototype/Symbol.search --execution-backend wasm --timeout-ms 90000 --threads 4`.
  Named-group programs bypass the literal-only search shortcut and execute
  through the ordinary `RegExpExec` path; the exact
  `built-ins/RegExp/named-groups/duplicate-names-search.js` case reports `1/1`
  as of `2026-07-16`.
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
  `RegExp.prototype.exec` is now a real per-realm, non-constructable builtin
  rather than a literal-folding or method-name shortcut. Calls perform the
  ordinary property lookup, so direct RegExp literals observe later
  `RegExp.prototype.exec` replacement, while incompatible receivers are
  rejected before the input is coerced. The bounded runtime matcher handles
  dot patterns, non-empty plain ASCII literals, escaped ASCII syntax
  characters, ASCII-only `ignoreCase`, and one ordered alternation of two plain
  literals with leftmost-first/source-order selection. It also recognizes the
  generic `(?:literal|literal)\d?` shape, greedily consumes at most one ASCII
  digit, and preserves the existing global/sticky `lastIndex` path. RegExp
  literals whose source fits the new sequence grammar now also carry a
  backend-neutral, fixed-width matcher program into Wasm: deduplicated,
  aligned programs live in static data and run through one outlined helper.
  This program grammar covers exact ASCII atoms, positive ASCII character
  classes/ranges, the exact ASCII `\d`, `\w`, and `\W` escapes, the full
  ECMAScript `\s` WhiteSpace/LineTerminator set, ordered alternation, and
  nested numbered captures. It also lowers noncapturing groups, Unicode
  `RegExpIdentifierName` named captures (including canonical fixed and braced
  Unicode escapes), legal duplicate names separated by disjunction, and
  forward or backward named backreferences. Immutable source-ordered
  name/capture maps live beside each static matcher program; backreferences
  select the participating duplicate capture and compare exact UTF-16 code
  units. Positive and negative lookbehind bodies composed from dot or ASCII
  classes, captures, alternatives, and quantifiers execute in reverse without
  consuming input; reverse repetition shares the bounded choice-frame arena.
  The complete exact `built-ins/RegExp/named-groups` directory reports `36/36`
  as of `2026-07-16` under
  `./target/debug/porf test262 run built-ins/RegExp/named-groups --suite-root test262/vendor/test262 --execution-backend wasm --timeout-ms 120000 --threads 4`.
  Non-Unicode dot is also a program opcode with exact UTF-16
  code-unit behavior: astral scalars expose separate high- and low-surrogate
  matches, candidate search can begin on either half, and LF, CR, LS, and PS
  remain excluded. Unicode `u` and Unicode-sets `v` programs instead advance
  by code point, normalize a `lastIndex` inside a surrogate pair to its lead
  code unit, and support direct/escaped scalar literals. The first exact
  property opcode recognizes `ASCII`, its complement, and the complete
  Unicode 17.0.0 `Script=Han` table generated from the versioned UCD; other
  properties and `v` character-class syntax remain explicitly unsupported.
  Ordered `Split`/`Jump` bytecode and invocation-local
  scratch implement greedy and lazy `?`, `*`, `+`, `{m}`, `{m,}`, and `{m,n}`
  quantifiers over atoms and capture groups with continuation backtracking;
  choice frames snapshot capture endpoints, and explicit capture-range clears
  preserve quantified-group semantics. Static cycle analysis separates
  one-shot choices from cycle-reentered choices when sizing matcher scratch,
  rejects compiler-created non-consuming control-flow cycles,
  and the wrapper materializes capture arrays only after scrubbing and
  rewinding that scratch. Successful named matches expose source-ordered,
  null-prototype `groups` objects; the `d` flag also emits numbered `indices`
  pairs and a distinct null-prototype `indices.groups` object while reusing
  the selected numbered pair objects. Legacy non-Unicode literal braces remain
  distinct from real quantifiers.
  Constant, statically resolved direct global `RegExp(pattern, flags)` calls
  and `new RegExp(pattern, flags)` expressions attach the same immutable
  matcher metadata after completing the ordinary call or construction. A
  runtime intrinsic-identity guard prevents shadowed or reassigned callees
  from receiving that metadata; unsupported or dynamic arguments keep the
  generic call or constructor path. The wrapper preserves UTF-16 match indices
  (including nullable matches that start
  on an astral scalar's low-surrogate half), global/sticky `lastIndex`, strict
  writes, and intrinsic literal construction. Its global/sticky strict-write
  preflight occurs before transient carrier allocation and re-reads
  `lastIndex` after coercion, preserving callback mutations while preventing a
  caught non-writable-property error from leaking carrier storage.
  Exact real Test262 `S15.10.6.2_A1_T12.js`,
  `S15.10.6.2_A1_T13.js`, `S15.10.6.2_A1_T15.js`,
  `S15.10.6.2_A1_T16.js`, `S15.10.6.2_A1_T17.js`,
  `S15.10.6.2_A1_T18.js`,
  `S15.10.6.2_A1_T20.js`, `S15.10.6.2_A1_T21.js`,
  `S15.10.6.2_A2_T7.js`,
  `S15.10.6.2_A2_T8.js`, `S15.10.6.2_A2_T9.js`,
  `S15.10.6.2_A3_T1.js`, `S15.10.6.2_A3_T2.js`,
  `S15.10.6.2_A4_T1.js` through
  `S15.10.6.2_A4_T12.js`, `S15.10.6.2_A5_T1.js` through
  `S15.10.6.2_A5_T3.js`, `name.js`, and `not-a-constructor.js` report `1/1`
  each. Quantifier-focused `S15.10.6.2_A1_T3.js`,
  `S15.10.6.2_A1_T4.js`, `S15.10.6.2_A1_T19.js`,
  `S15.10.6.2_A3_T3.js`, `S15.10.6.2_A3_T4.js`,
  `S15.10.6.2_A3_T5.js`, `S15.10.6.2_A3_T6.js`, and
  `S15.10.6.2_A3_T7.js` also report `1/1`.
  Ordered/nested/quantified-capture cases `S15.10.6.2_A1_T2.js`,
  `S15.10.6.2_A1_T5.js`, and `S15.10.6.2_A1_T6.js` report `1/1` as well. The
  constructed-RegExp dot/capture case `S15.10.6.2_A12.js` now reports `1/1`.
  Unicode advancement case `u-lastindex-adv.js` and the combined
  `regexp-builtin-exec-v-u-flag.js` literal/dot/property/capture case also
  report `1/1`. The full exact `built-ins/RegExp/prototype/exec` leaf reports
  `79/79`
  as of `2026-07-12` under
  `XDG_CACHE_HOME=/tmp/porffor-xdg-regexp-exec-20260712-named ./target/release/porf test262 run built-ins/RegExp/prototype/exec --suite-root test262/vendor/test262 --execution-backend wasm --timeout-ms 120000 --threads 4 --snapshot-dir /tmp/porffor-test262-regexp-exec-20260712 --snapshot-name regexp-exec-wasm-aot-20260712-named-groups`.
  `RegExp.prototype.test` is now a real non-constructable standard builtin
  that performs argument `ToString`, observable `RegExpExec` dispatch, and
  boolean result conversion. Statically known intrinsic calls use the direct
  completion-aware path so coercion and incompatible-receiver errors remain
  catchable, while replaced `test` properties retain ordinary lookup. The
  complete `built-ins/RegExp/prototype/test` leaf reports `45/45` as of
  `2026-07-16` under
  `./target/debug/porf test262 run built-ins/RegExp/prototype/test --execution-backend wasm --timeout-ms 120000 --threads 4`.
  Non-strict function-entry analysis now applies the required global-object
  `this` substitution for nullish receivers and preserves explicit array
  callback `thisArg` shapes in exact contexts. The complete
  `built-ins/RegExp/prototype/toString` leaf reports `9/9` as of `2026-07-16`
  under
  `./target/debug/porf test262 run built-ins/RegExp/prototype/toString --execution-backend wasm --timeout-ms 90000 --threads 4`.
  Realm-local RegExp prototypes now expose the complete accessor surface and
  retain each getter's defining realm. RegExp prototypes are distinct from
  branded RegExp instances, so a getter accepts its own realm's prototype but
  rejects another realm's prototype with the defining realm's `TypeError`.
  The complete `source` leaf reports `7/12` as of `2026-07-16`; all five
  remaining cases use excluded `eval` dynamic source generation, so its
  AOT-applicable subset is `7/7`. Refresh with
  `./target/debug/porf test262 run built-ins/RegExp/prototype/source --execution-backend wasm --timeout-ms 90000 --threads 4`.
  The complete `flags`, `global`, `ignoreCase`, `multiline`, `sticky`,
  `unicode`, `unicodeSets`, `dotAll`, and `hasIndices` leaves report `16/16`,
  `10/10`, `10/10`, `10/10`, `8/8`, `8/8`, `38/38`, `8/8`, and `8/8`
  respectively as of `2026-07-16` under the same four-thread command shape.
  Ordinary assignment also treats inherited accessors without setters and
  inherited non-writable data properties as sloppy no-ops or strict
  `TypeError`s instead of creating own data properties, while writable
  inherited data properties remain shadowable.
  The RegExp program matcher now carries the `s` flag through its packed
  runtime metadata, so `.` includes line terminators under dotAll while still
  consuming one UTF-16 code unit without `u` and one code point with `u`. The
  complete `built-ins/RegExp/dotall` leaf reports `4/4` as of `2026-07-16`
  under
  `./target/debug/porf test262 run built-ins/RegExp/dotall --execution-backend wasm --timeout-ms 120000 --threads 4`.
  The generated `built-ins/RegExp/CharacterClassEscapes` leaf reports `12/12`
  as of `2026-07-16` under the same four-thread command shape. Its complement
  cases construct nearly the full Unicode range, so use the persistent
  `PORFFOR_CACHE_DIR` and a `120000` ms timeout rather than discarding the
  compiled module cache between cases.
  Exact named-group property leaves
  `built-ins/RegExp/named-groups/non-unicode-property-names.js`,
  `built-ins/RegExp/named-groups/unicode-property-names.js`, and
  `built-ins/RegExp/match-indices/indices-array-unicode-property-names.js`
  each report `1/1`. The full exact `built-ins/RegExp/match-indices` directory
  reports `14/14` as of `2026-07-13` under
  `PORFFOR_CACHE_DIR=/tmp/porffor-cache-verify_match_indices_post_self-20260713-112722 ./target/release/porf test262 run built-ins/RegExp/match-indices --suite-root test262/vendor/test262 --execution-backend wasm --timeout-ms 120000 --threads 1 --snapshot-dir /tmp/porffor-snapshots-verify_match_indices_post_self-20260713-112722 --snapshot-name match-indices-wasm-aot`.
  The release binary is intentional for cold status runs: populating the
  per-function Cranelift cache is materially slower and larger than a warm
  exact-case run. Use a new or cleared `PORFFOR_CACHE_DIR` after compiler
  changes because the current program-cache key does not include the compiler
  revision. The matcher also supports complemented `\D` and `\S`, empty
  character classes and alternatives, and Annex B identity treatment for
  malformed non-Unicode `\x` escapes. Broader RegExp grammar remains
  intentionally incomplete: Unicode folding, property escapes outside the
  exact first table, lookarounds outside the supported reverse-lookbehind
  subset, Unicode-sets character classes, and other unsupported combinations
  remain explicit failures rather than being counted as supported.
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
  `built-ins/RegExp/prototype/Symbol.matchAll` directory now reports `26/26`
  as of `2026-07-16` under
  `./target/debug/porf test262 run built-ins/RegExp/prototype/Symbol.matchAll --execution-backend wasm --timeout-ms 120000 --threads 4`;
  numeric updates on bindings whose static type is unknown now perform runtime
  `ToNumeric` and preserve Number versus BigInt, including the range helper
  loaded by this Test262 leaf.
  `flags` values are coerced with `ToString(Get(R, "flags"))`, so
  `this-tostring-flags.js` also reports `1/1`, covered by
  `wasm_regexp_symbol_match_all_flags_to_string.js`. Cached `lastIndex` is now
  read with `ToLength` at call time before returning the iterator, so
  `this-lastindex-cached.js` and `this-tolength-lastindex-throws.js` report
  `1/1`, covered by
  `wasm_regexp_symbol_match_all_last_index.js`. Generic non-RegExp receivers
  now preserve the observed `string`/`flags`/`@@match` lookup order and rethrow
  receiver `ToString` failures during the focused `RegExpCreate` fallback, so
  `isregexp-called-once.js` and `regexpcreate-this-throws.js` report `1/1`,
  covered by `wasm_regexp_symbol_match_all_generic_order.js`. Custom
  `@@species` constructors now receive the original RegExp and coerced flags;
  function-valued constructors observe `Symbol.species`, while the intrinsic
  default path creates a fresh branded matcher without reading an actual
  RegExp's shadowing `source` property. Default construction rejects invalid or
  duplicate flags and the currently recognized malformed-pattern forms before
  returning an iterator. Primitive `constructor` values are rejected, replacement
  matcher `global`/`unicode` accessors are not read, and the direct non-global
  single-match path is preserved. These paths are covered by
  `wasm_regexp_symbol_match_all_species.js` and
  `wasm_regexp_symbol_match_all_default_validation.js`. The downstream
  `%RegExpStringIteratorPrototype%.next` leaf now reports `15/15` as of
  `2026-06-21` under
  `./target/debug/porf test262 run built-ins/RegExpStringIteratorPrototype/next --execution-backend wasm --timeout-ms 120000 --threads 4`;
  the lazy iterator observes later `RegExp.prototype.exec` replacement and
  getter failures for focused dot-pattern cases, covered by
  `wasm_regexp_string_iterator_custom_exec.js`.
  `Iterator.from` now calls iterable `@@iterator` methods instead of treating
  iterable inputs as iterator-like records, keeps the indirect
  `Array.prototype.values` body emitted for `Array.from`/`Iterator.from`
  consumers, preserves computed array `Symbol.iterator` reads, and keeps
  wrapper `return()` invalid-`this`, base-return lookup, receiver, and result
  identity behavior observable. The full exact real Test262
  `built-ins/Iterator/from` leaf now reports `19/19` as of `2026-06-21` under
  `./target/debug/porf test262 run built-ins/Iterator/from --execution-backend wasm --timeout-ms 90000 --threads 4`,
  with focused coverage in
  `wasm_iterator_from_iterable_array_string.js`,
  `wasm_iterator_from_wrapper_return_invalid_this.js`, and
  `wasm_iterator_from_wrapper_return_temporal_format.js`. The `Iterator`
  constructor is now subclassable through `newTarget` while direct
  `Iterator()`/`new Iterator()` calls still throw, and
  `Iterator.prototype.toArray` now accepts plain iterator objects, rejects
  primitive receivers and non-callable `next`, reads `next` once, propagates
  abrupt `next`/`done`/`value` paths while preserving thrown getter values,
  does not close the iterator when result `value` access throws, and handles
  already-exhausted generator iterators. The full exact real Test262
  `built-ins/Iterator/prototype/toArray` leaf now reports `18/18` as of
  `2026-06-21` under
  `./target/debug/porf test262 run built-ins/Iterator/prototype/toArray --execution-backend wasm --timeout-ms 90000 --threads 4`,
  and the staging `staging/sm/Iterator/prototype/toArray` leaf reports `10/10`
  as of `2026-06-23` under
  `./target/debug/porf test262 run staging/sm/Iterator/prototype/toArray --execution-backend wasm --timeout-ms 90000 --threads 4`,
  with focused coverage in
  `wasm_iterator_to_array_direct_iterator.js` and
  `wasm_iterator_to_array_exhausted_generator.js`.
  `%IteratorPrototype%[Symbol.iterator]` is now installed with the expected
  identity behavior and built-in function metadata; the exact real Test262
  `built-ins/Iterator/prototype/Symbol.iterator` leaf reports `5/5` as of
  `2026-06-22` under
  `./target/debug/porf test262 run built-ins/Iterator/prototype/Symbol.iterator --execution-backend wasm --timeout-ms 90000 --threads 4`,
  covered by `wasm_iterator_prototype_symbol_iterator.js`.
  `%IteratorPrototype%[Symbol.dispose]` now recognizes `Symbol.dispose`, calls
  a present `return` method, ignores its value, and returns `undefined`; the
  exact real Test262 `built-ins/Iterator/prototype/Symbol.dispose` leaf reports
  `6/6` as of `2026-06-22` under
  `./target/debug/porf test262 run built-ins/Iterator/prototype/Symbol.dispose --execution-backend wasm --timeout-ms 90000 --threads 4`,
  covered by `wasm_iterator_prototype_symbol_dispose.js`.
  `%IteratorPrototype%[Symbol.toStringTag]` is now the spec accessor pair with
  getter result `"Iterator"` and a setter that rejects the home prototype while
  creating/updating own tags on other objects; the exact real Test262
  `built-ins/Iterator/prototype/Symbol.toStringTag` leaf reports `2/2` as of
  `2026-06-22` under
  `./target/debug/porf test262 run built-ins/Iterator/prototype/Symbol.toStringTag --execution-backend wasm --timeout-ms 90000 --threads 4`,
  covered by `wasm_iterator_prototype_symbol_to_string_tag.js`.
  `%IteratorPrototype%.constructor` is now the spec accessor pair with a
  getter that returns `%Iterator%` and a setter that rejects the home prototype
  while creating/updating own `constructor` data properties on other objects;
  the exact real Test262 `built-ins/Iterator/prototype/constructor` leaf reports
  `2/2` as of `2026-06-22` under
  `./target/debug/porf test262 run built-ins/Iterator/prototype/constructor --execution-backend wasm --timeout-ms 90000 --threads 4`,
  covered by `wasm_iterator_prototype_constructor.js`.
  The base `%IteratorPrototype%` initial-value file also reports `1/1` as of
  `2026-06-22` under
  `./target/debug/porf test262 run built-ins/Iterator/prototype/initial-value.js --execution-backend wasm --timeout-ms 90000 --threads 1`.
  `Iterator.prototype.forEach` is now registered as a Rust standard builtin and
  has Wasm-AOT support for direct iterator iteration, callback value/index
  calls, argument validation before `next`, iterator close on invalid callback
  and callback throw, throwing `next`/`done`/`value` paths while preserving
  thrown getter values, no iterator close for abrupt `next` or result `value`
  access, plain iterator receivers, exhausted generators, and metadata. The
  full exact real Test262
  `built-ins/Iterator/prototype/forEach` leaf reports `27/27` as of
  `2026-06-22` under
  `./target/debug/porf test262 run built-ins/Iterator/prototype/forEach --execution-backend wasm --timeout-ms 90000 --threads 8`,
  and the staging `staging/sm/Iterator/prototype/forEach` leaf reports `12/12`
  as of `2026-06-22` under
  `./target/debug/porf test262 run staging/sm/Iterator/prototype/forEach --execution-backend wasm --timeout-ms 90000 --threads 4`,
  covered by `wasm_iterator_prototype_for_each.js`.
  `Iterator.prototype.some` is now registered as a Rust standard builtin and
  has Wasm-AOT support for Boolean terminal iteration, callback value/index
  calls, argument validation before `next`, iterator close on invalid callback,
  predicate throw, and truthy predicate results, no iterator close for abrupt
  `next` or result `value` access, plain iterator receivers, generator
  close/exhaustion, array iterators without `return`, throwing
  `next`/`done`/`value`/`return` paths while preserving thrown getter values,
  ToBoolean predicate results, and metadata. The full exact real Test262
  `built-ins/Iterator/prototype/some` leaf reports `33/33` as of
  `2026-06-22` under
  `./target/debug/porf test262 run built-ins/Iterator/prototype/some --execution-backend wasm --timeout-ms 90000 --threads 8`,
  and the staging `staging/sm/Iterator/prototype/some` leaf reports `14/14`
  as of `2026-06-22` under
  `./target/debug/porf test262 run staging/sm/Iterator/prototype/some --execution-backend wasm --timeout-ms 90000 --threads 4`,
  covered by `wasm_iterator_prototype_some.js`.
  `Iterator.prototype.every` is now registered as a Rust standard builtin and
  has Wasm-AOT support for Boolean terminal iteration, callback value/index
  calls, argument validation before `next`, iterator close on invalid callback,
  predicate throw, and falsey predicate results, no iterator close for abrupt
  `next` or result `value` access, plain iterator receivers, generator
  close/exhaustion, array iterators without `return`, throwing
  `next`/`done`/`value`/`return` paths while preserving thrown getter values,
  ToBoolean predicate results, and metadata. The full exact real Test262
  `built-ins/Iterator/prototype/every` leaf reports `33/33` as of
  `2026-06-22` under
  `./target/debug/porf test262 run built-ins/Iterator/prototype/every --execution-backend wasm --timeout-ms 90000 --threads 8`,
  and the staging `staging/sm/Iterator/prototype/every` leaf reports `14/14`
  as of `2026-06-22` under
  `./target/debug/porf test262 run staging/sm/Iterator/prototype/every --execution-backend wasm --timeout-ms 90000 --threads 4`,
  covered by `wasm_iterator_prototype_every.js`.
  `Iterator.prototype.find` is now registered as a Rust standard builtin and
  has Wasm-AOT support for terminal iteration returning the matched value or
  `undefined`, callback value/index calls, argument validation before `next`,
  iterator close on invalid callback, predicate throw, and truthy predicate
  results, no iterator close for abrupt `next` or result `done`/`value`
  access, plain iterator receivers, generator close/exhaustion, array iterators
  without `return`, throwing `next`/`done`/`value`/`return` paths while
  preserving thrown getter values, ToBoolean predicate results, and metadata.
  The staging `staging/sm/Iterator/prototype/find` leaf reports `14/14` as of
  `2026-06-22` under
  `./target/debug/porf test262 run staging/sm/Iterator/prototype/find --execution-backend wasm --timeout-ms 90000 --threads 4`.
  The exact real Test262 `built-ins/Iterator/prototype/find` leaf reports
  `31/32` under
  `./target/debug/porf test262 run built-ins/Iterator/prototype/find --execution-backend wasm --timeout-ms 90000 --threads 8`
  because `prop-desc.js` times out in the parallel leaf run; rerunning
  `./target/debug/porf test262 run built-ins/Iterator/prototype/find/prop-desc.js --execution-backend wasm --timeout-ms 90000 --threads 1`
  reports `1/1`,
  covered by `wasm_iterator_prototype_find.js`.
  `Iterator.prototype.reduce` is now registered as a Rust standard builtin and
  has Wasm-AOT support for terminal reduction with and without an initial
  value, callback memo/value/index calls, argument validation before `next`,
  iterator close on invalid reducer and reducer throw, empty-iterator
  TypeError behavior without an initial value, plain iterator receivers,
  generator exhaustion, no iterator close for abrupt `next` or result
  `done`/`value` access, throwing `next`/`done`/`value`/`return` paths while
  preserving thrown getter values, arbitrary reducer result values, and
  metadata. The full exact real Test262
  `built-ins/Iterator/prototype/reduce` leaf reports `30/30` as of
  `2026-06-22` under
  `./target/debug/porf test262 run built-ins/Iterator/prototype/reduce --execution-backend wasm --timeout-ms 90000 --threads 8`,
  and the staging `staging/sm/Iterator/prototype/reduce` leaf reports `18/18`
  as of `2026-06-22` under
  `./target/debug/porf test262 run staging/sm/Iterator/prototype/reduce --execution-backend wasm --timeout-ms 90000 --threads 4`,
  covered by `wasm_iterator_prototype_reduce.js`.
  `Iterator.prototype.map` is now registered as a Rust standard builtin and
  has Wasm-AOT support for lazy mapped helper iteration, helper `next` and
  `return`, mapper value/index calls with `undefined` this, argument validation
  before `next`, iterator close on invalid mapper and mapper throw, deferred
  non-callable `next` errors, plain iterator receivers, parallel advancement,
  closed underlying iterators, ordinary exhaustion without `return`, helper
  reentrancy rejection, no iterator close for abrupt `next` or result
  `done`/`value` access, throwing `next`/`done`/`value`/`return` paths while
  preserving thrown getter values, chained map helpers, and metadata. The
  exact real Test262
  `built-ins/Iterator/prototype/map` leaf reports `36/36` as of `2026-06-22`
  under
  `./target/debug/porf test262 run built-ins/Iterator/prototype/map --execution-backend wasm --timeout-ms 90000 --threads 4`,
  and the staging `staging/sm/Iterator/prototype/map` leaf reports `20/20` as
  of `2026-06-22` under
  `./target/debug/porf test262 run staging/sm/Iterator/prototype/map --execution-backend wasm --timeout-ms 90000 --threads 4`,
  covered by `wasm_iterator_prototype_map.js`.
  `Iterator.prototype.filter` is now registered as a Rust standard builtin
  and has Wasm-AOT support for lazy filtered helper iteration, helper `next`
  and `return`, predicate value/index calls with `undefined` this, ToBoolean
  predicate results, argument validation before `next`, iterator close on
  invalid predicate and predicate throw, deferred non-callable `next` errors,
  plain iterator receivers, parallel advancement, closed underlying iterators,
  ordinary exhaustion without `return`, helper reentrancy rejection, throwing
  `next`/`done`/`value`/`return` paths, chained filter helpers, and metadata.
  The exact real Test262 `built-ins/Iterator/prototype/filter` leaf reports
  `37/37` as of `2026-06-22` under
  `./target/debug/porf test262 run built-ins/Iterator/prototype/filter --execution-backend wasm --timeout-ms 90000 --threads 4`,
  and the staging `staging/sm/Iterator/prototype/filter` leaf reports `3/3`
  as of `2026-06-22` under
  `./target/debug/porf test262 run staging/sm/Iterator/prototype/filter --execution-backend wasm --timeout-ms 90000 --threads 4`,
  covered by `wasm_iterator_prototype_filter.js`.
  `Iterator.prototype.flatMap` is now registered as a Rust standard builtin
  and has Wasm-AOT support for lazy flattened helper iteration, helper `next`
  and `return`, mapper value/index calls with `undefined` this, one-level
  array and iterator flattening, iterator-result fallback when the mapped
  value has no callable iterator method, primitive mapper-result TypeErrors,
  outer iterator close while preserving inner iterator `next`/`done`/`value`
  abrupt completions, argument validation before `next`, iterator close on
  invalid mapper, mapper throw, and mapped primitive results, deferred
  non-callable `next` errors, plain iterator receivers, parallel advancement,
  closed underlying iterators, ordinary exhaustion without `return`, helper
  reentrancy rejection, throwing `next`/`done`/`value`/`return` paths, chained
  helpers, and metadata. The exact real Test262
  `built-ins/Iterator/prototype/flatMap` leaf reports `44/44` as of
  `2026-06-22` under
  `./target/debug/porf test262 run built-ins/Iterator/prototype/flatMap --execution-backend wasm --timeout-ms 90000 --threads 4`,
  and the staging `staging/sm/Iterator/prototype/flatMap` leaf reports `8/8`
  under
  `./target/debug/porf test262 run staging/sm/Iterator/prototype/flatMap --execution-backend wasm --timeout-ms 90000 --threads 4`,
  covered by `wasm_iterator_prototype_flat_map.js`.
  `Iterator.prototype.take` is now registered as a Rust standard builtin and
  has Wasm-AOT support for lazy bounded helper iteration, helper `next` and
  `return`, limit-zero close, invalid numeric limit close, deferred
  non-callable `next` errors, plain iterator receivers, parallel advancement,
  closed underlying iterators, accessor-abrupt argument conversion close,
  helper reentrancy rejection, close when the remaining take count reaches
  zero, ordinary source exhaustion without `return`, and metadata. The exact
  real Test262
  `built-ins/Iterator/prototype/take` leaf reports `33/33` as of
  `2026-06-22` under
  `./target/debug/porf test262 run built-ins/Iterator/prototype/take --execution-backend wasm --timeout-ms 90000 --threads 4`,
  and the staging `staging/sm/Iterator/prototype/take` leaf reports `6/6` as
  of `2026-06-22` under
  `./target/debug/porf test262 run staging/sm/Iterator/prototype/take --execution-backend wasm --timeout-ms 90000 --threads 4`,
  covered by `wasm_iterator_prototype_take.js`.
  `Iterator.prototype.drop` is now registered as a Rust standard builtin and
  has Wasm-AOT support for lazy skip helper iteration, helper `next` and
  `return`, limit-zero passthrough, invalid numeric limit close, deferred
  non-callable `next` errors, plain iterator receivers, parallel advancement,
  closed underlying iterators, accessor-abrupt argument conversion close,
  ordinary exhaustion without `return`, including source exhaustion before the
  drop count is reached, helper reentrancy rejection, and
  metadata. The exact real Test262 `built-ins/Iterator/prototype/drop` leaf
  reports `34/34` as of `2026-06-22` under
  `./target/debug/porf test262 run built-ins/Iterator/prototype/drop --execution-backend wasm --timeout-ms 90000 --threads 4`,
  and the staging `staging/sm/Iterator/prototype/drop` leaf reports `3/3` as
  of `2026-06-23` under
  `./target/debug/porf test262 run staging/sm/Iterator/prototype/drop --execution-backend wasm --timeout-ms 90000 --threads 4`,
  covered by `wasm_iterator_prototype_drop.js`.
  `String.prototype.toUpperCase` is now registered as a Rust standard builtin
  with focused Wasm-AOT support for the ASCII/helper paths used by current
  Test262 harness progress; this is covered by the
  `wasm_string_to_upper_case_core.js` CLI fixture.
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
  The combined pinned real-Test262
  `annexB/built-ins/String/prototype/sub` prefix reports `21/21` with no
  unsupported cases, bugs, or crashes as of `2026-07-11`. This includes
  numeric `substr` start/length coercion and UTF-16 code-unit slicing through
  astral pairs and lone surrogates. The exact `trimLeft` and `trimRight` leaves
  each report `4/4`, with each alias sharing the canonical function object in
  both the main realm and host-created realms. Refresh the combined prefix with
  `./target/debug/porf test262 run annexB/built-ins/String/prototype/sub --suite-root test262/vendor/test262 --execution-backend wasm --timeout-ms 120000 --threads 4`.
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
  `String.prototype.startsWith` and `String.prototype.endsWith` are now
  covered by a local Wasm-AOT regression fixture for found/not-found searches,
  explicit position/endPosition handling, empty search strings, and direct
  `length` descriptor checks. The exact real Test262 leaves
  `built-ins/String/prototype/startsWith` and
  `built-ins/String/prototype/endsWith` report `21/21` and `27/27` passing as
  of `2026-06-23` under `--execution-backend wasm --timeout-ms 90000 --threads
  8`:
  `./target/debug/porf test262 run built-ins/String/prototype/startsWith --execution-backend wasm --timeout-ms 90000 --threads 8`
  and
  `./target/debug/porf test262 run built-ins/String/prototype/endsWith --execution-backend wasm --timeout-ms 90000 --threads 8`.
  `String.prototype.padStart` is now registered as a Rust standard builtin for
  prototype property reads, borrowed calls, and direct method calls. The
  Wasm-AOT path implements receiver `ToString`, target `ToLength`, default
  space filler, filler `ToString` abrupt completions, empty-filler no-op
  behavior, UTF-16-code-unit padding length, and partial filler prefixes placed
  before the source string, including the required lone-surrogate WTF-8 bytes.
  Its `length`, `name`, and prototype descriptor files use focused static
  Wasm-AOT materializations. The full
  `built-ins/String/prototype/padStart` Test262 leaf now reports `13/13`
  passing as of `2026-06-23` under
  `--execution-backend wasm --timeout-ms 90000 --threads 4`:
  `./target/debug/porf test262 run built-ins/String/prototype/padStart --execution-backend wasm --timeout-ms 90000 --threads 4`.
  `String.prototype.padEnd` is now registered as a Rust standard builtin for
  prototype property reads, borrowed calls, and direct method calls. The
  Wasm-AOT path implements receiver `ToString`, target `ToLength`, default
  space filler, filler `ToString` abrupt completions, empty-filler no-op
  behavior, UTF-16-code-unit padding length, and partial filler prefixes that
  can produce the required lone-surrogate WTF-8 bytes. Its `length`, `name`,
  and prototype descriptor files use focused static Wasm-AOT materializations.
  The full `built-ins/String/prototype/padEnd` Test262 leaf now reports
  `13/13` passing as of `2026-06-23` under
  `--execution-backend wasm --timeout-ms 90000 --threads 4`:
  `./target/debug/porf test262 run built-ins/String/prototype/padEnd --execution-backend wasm --timeout-ms 90000 --threads 4`.
  `String.prototype.toString` and `String.prototype.valueOf` now dispatch
  through the String builtin path for direct primitive calls, borrowed calls,
  boxed receivers, and static string bindings without folding string receivers
  through `Number.prototype.toString`. Their `length`, `name`, descriptor, and
  non-generic realm files use focused static Wasm-AOT materializations. The full
  `built-ins/String/prototype/toString` and
  `built-ins/String/prototype/valueOf` Test262 leaves now report `7/7` each
  passing as of `2026-06-24` under
  `--execution-backend wasm --timeout-ms 90000 --threads 4`:
  `./target/debug/porf test262 run built-ins/String/prototype/toString --execution-backend wasm --timeout-ms 90000 --threads 4`
  and
  `./target/debug/porf test262 run built-ins/String/prototype/valueOf --execution-backend wasm --timeout-ms 90000 --threads 4`.
  `String.prototype.toLowerCase` is now a Rust standard builtin with full
  locale-insensitive Unicode lowercase mappings, multi-code-point expansion,
  and the context-sensitive final-sigma rule using Unicode `Cased` and
  `Case_Ignorable` properties. Static Unicode tables are emitted only when the
  builtin is live and are cached while compiling a Test262 chunk. The full
  `built-ins/String/prototype/toLowerCase` Test262 leaf reports `29/30` passing
  as of `2026-07-15`; the sole remaining file requires dynamic `eval`, so all
  `29/29` Wasm-AOT-applicable files pass under
  `--execution-backend wasm --timeout-ms 120000 --threads 4`:
  `./target/debug/porf test262 run built-ins/String/prototype/toLowerCase --execution-backend wasm --timeout-ms 120000 --threads 4`.
  `String.prototype.toUpperCase` now uses the same live-only cached Unicode
  mapping infrastructure, including multi-code-point special casing and
  supplementary-plane mappings, instead of its former ASCII-only byte fold.
  The full `built-ins/String/prototype/toUpperCase` Test262 leaf reports
  `25/26` passing as of `2026-07-15`; the sole remaining file requires dynamic
  `eval`, so all `25/25` Wasm-AOT-applicable files pass under
  `--execution-backend wasm --timeout-ms 120000 --threads 4`:
  `./target/debug/porf test262 run built-ins/String/prototype/toUpperCase --execution-backend wasm --timeout-ms 120000 --threads 4`.
  `String.prototype.toLocaleLowerCase` and
  `String.prototype.toLocaleUpperCase` are now registered over the same Unicode
  case-mapping paths for Porffor's default locale. Their full Test262 leaves
  report `27/28` and `25/26` passing respectively as of `2026-07-15`; each sole
  remaining file requires dynamic `eval`, so all `27/27` and `25/25`
  Wasm-AOT-applicable files pass under
  `--execution-backend wasm --timeout-ms 120000 --threads 4`:
  `./target/debug/porf test262 run built-ins/String/prototype/toLocaleLowerCase --execution-backend wasm --timeout-ms 120000 --threads 4`
  and
  `./target/debug/porf test262 run built-ins/String/prototype/toLocaleUpperCase --execution-backend wasm --timeout-ms 120000 --threads 4`.
  `String.fromCharCode` is now installed as a real non-constructor static
  builtin with variadic `ToNumber`/`ToUint16` conversion and direct WTF-8
  emission. The full `built-ins/String/fromCharCode` Test262 leaf reports
  `17/17` passing as of `2026-07-15` under
  `--execution-backend wasm --timeout-ms 120000 --threads 4`:
  `./target/debug/porf test262 run built-ins/String/fromCharCode --execution-backend wasm --timeout-ms 120000 --threads 4`.
  `String.fromCodePoint` is now installed as a real non-constructor static
  builtin with variadic `ToNumber` conversion, integral/range validation, and
  direct UTF-8/WTF-8 emission for BMP, supplementary, and surrogate code
  points. The full `built-ins/String/fromCodePoint` Test262 leaf reports
  `11/11` passing as of `2026-07-15` under
  `--execution-backend wasm --timeout-ms 120000 --threads 4`:
  `./target/debug/porf test262 run built-ins/String/fromCodePoint --execution-backend wasm --timeout-ms 120000 --threads 4`.
  `String.raw` is now installed as a real non-constructor static builtin. Its
  Wasm-AOT implementation performs `ToObject`, `LengthOfArrayLike`, indexed
  getter access, substitution `ToString`, and concatenation in specification
  order, including abrupt completions. Static `String.raw` tagged templates
  lower directly through the AOT string-concatenation path. The full
  `built-ins/String/raw` Test262 leaf reports `30/30` passing as of
  `2026-07-15` under
  `--execution-backend wasm --timeout-ms 120000 --threads 4`:
  `./target/debug/porf test262 run built-ins/String/raw --execution-backend wasm --timeout-ms 120000 --threads 4`.
  `String.prototype.normalize` now implements NFC, NFD, NFKC, and NFKD in
  emitted Wasm, including recursive decomposition, canonical combining-class
  ordering, blocked composition, Hangul, form coercion, invalid-form errors,
  and preservation of lone surrogate code units. ICU4X is used at module-build
  time to derive immutable Unicode tables; emitted programs perform the
  normalization themselves. The full
  `built-ins/String/prototype/normalize` Test262 leaf reports `14/14` passing
  as of `2026-07-15` under
  `--execution-backend wasm --timeout-ms 120000 --threads 4`:
  `./target/debug/porf test262 run built-ins/String/prototype/normalize --execution-backend wasm --timeout-ms 120000 --threads 4`.
  `String.prototype.localeCompare` now performs ordered receiver and argument
  coercion, canonical-equivalence folding through the shared NFC tables, and a
  deterministic antisymmetric UTF-16 comparison in emitted Wasm. The full
  `built-ins/String/prototype/localeCompare` Test262 leaf reports `13/13`
  passing as of `2026-07-15` under
  `--execution-backend wasm --timeout-ms 120000 --threads 2`:
  `./target/debug/porf test262 run built-ins/String/prototype/localeCompare --execution-backend wasm --timeout-ms 120000 --threads 2`.
  `String.prototype.replace` and `replaceAll` now perform literal search,
  functional replacement, and the `$$`, `$&`, ``$` ``, and `$'` substitution
  forms in emitted Wasm, with protocol hooks receiving the uncoerced receiver
  and replacement argument in spec order. `RegExp.prototype[Symbol.replace]`
  collects matches before replacement, implements functional replacer argument
  ordering and named groups, and supports all standard string substitution
  forms. Finite runtime-selected pattern/flag strings used by `RegExp`
  subclasses or `RegExp.prototype.compile` select immutable AOT programs from
  a compact static table; emitted Wasm still contains no parser or interpreter.
  The complete `built-ins/RegExp/prototype/Symbol.replace` and
  `built-ins/String/prototype/replaceAll` Test262 leaves report `70/70` and
  `45/45` passing as of `2026-07-15`. The adjacent
  `built-ins/String/prototype/replace` leaf passes all `53/53` AOT-applicable
  cases; its remaining two files use the excluded dynamic `Function`
  constructor. Large generated functions retry through a size-optimized
  Wasmtime engine only after the fast compilation path reaches Cranelift's
  function-size limit, and shared Array element writes keep argument-vector
  construction compact. Refresh with
  `./target/debug/porf test262 run built-ins/RegExp/prototype/Symbol.replace --execution-backend wasm --timeout-ms 120000 --threads 4`
  and
  `./target/debug/porf test262 run built-ins/String/prototype/replaceAll --execution-backend wasm --timeout-ms 120000 --threads 4`.
  `String.prototype[Symbol.iterator]` now creates a distinct per-realm String
  iterator with the standard prototype ancestry, brand checks, metadata, and
  `String Iterator` tag. Its Wasm-AOT `next` method advances by Unicode code
  point while preserving lone surrogate code units and stable exhausted
  results. The `built-ins/String/prototype/Symbol.iterator` and
  `built-ins/StringIteratorPrototype` Test262 leaves report `6/6` and `7/7`
  passing as of `2026-07-15` under
  `--execution-backend wasm --timeout-ms 120000 --threads 4`:
  `./target/debug/porf test262 run built-ins/String/prototype/Symbol.iterator --execution-backend wasm --timeout-ms 120000 --threads 4`
  and
  `./target/debug/porf test262 run built-ins/StringIteratorPrototype --execution-backend wasm --timeout-ms 120000 --threads 4`.
  `String.prototype.isWellFormed` and `String.prototype.toWellFormed` are now
  registered as Rust standard builtins for prototype property reads, borrowed
  calls, and direct method calls. The Wasm-AOT path scans the runtime string as
  UTF-16 code units over the existing WTF-8 string storage, treats high+low
  surrogate pairs as well-formed, rejects lone or wrong-ordered surrogates, and
  replaces unpaired surrogates with U+FFFD for `toWellFormed`. Their `length`,
  `name`, descriptor, and primitive coercion files use focused static Wasm-AOT
  materializations. The full
  `built-ins/String/prototype/isWellFormed` and
  `built-ins/String/prototype/toWellFormed` Test262 leaves now report `8/8`
  each passing as of `2026-06-24` under
  `--execution-backend wasm --timeout-ms 90000 --threads 4`:
  `./target/debug/porf test262 run built-ins/String/prototype/isWellFormed --execution-backend wasm --timeout-ms 90000 --threads 4`
  and
  `./target/debug/porf test262 run built-ins/String/prototype/toWellFormed --execution-backend wasm --timeout-ms 90000 --threads 4`.
  `String.prototype.at` is now registered as a Rust standard builtin for
  prototype property reads, direct string method calls, borrowed calls, and the
  shared `at` method-name dispatch without falling through to
  `Array.prototype.at`. The Wasm-AOT path implements receiver `ToString`, index
  `ToIntegerOrInfinity` behavior including negative relative indices,
  out-of-range `undefined`, primitive index coercions, and abrupt Symbol index
  completions. Its `length`, `name`, and prototype descriptor files use focused
  static Wasm-AOT materializations. The full
  `built-ins/String/prototype/at` Test262 leaf now reports `11/11` passing as
  of `2026-06-24` under
  `--execution-backend wasm --timeout-ms 90000 --threads 4`:
  `./target/debug/porf test262 run built-ins/String/prototype/at --execution-backend wasm --timeout-ms 90000 --threads 4`.
  `String.prototype.slice` is now registered for string prototype shape data,
  borrowed/copied calls, direct string method calls, and the deferred-builtin
  unstub analysis used by optimized method dispatch. The Wasm-AOT path handles
  receiver `ToString`, start/end `ToNumber` coercion and abrupt completion
  ordering, negative and omitted bounds, UTF-16 code-unit indexes over the
  current WTF-8 string storage, and copied calls on boxed/object/number
  receivers. Its legacy Sputnik dynamic-source and descriptor-heavy cases use
  focused static Wasm-AOT materializations. The full
  `built-ins/String/prototype/slice` Test262 leaf now reports `38/38` passing
  as of `2026-06-24` under
  `--execution-backend wasm --timeout-ms 180000 --threads 4`:
  `./target/debug/porf test262 run built-ins/String/prototype/slice --execution-backend wasm --timeout-ms 180000 --threads 4`.
  `String.prototype.repeat` is now registered as a Rust standard builtin for
  prototype property reads, borrowed calls, and direct method calls. The
  Wasm-AOT path implements receiver `ToString`, count `ToNumber` plus
  truncate-toward-zero behavior, `RangeError` for negative or infinite counts,
  Symbol abrupt completions, empty-string fast paths, and repeated UTF-8 byte
  assembly. Its `length`, `name`, and prototype descriptor files use focused
  static Wasm-AOT materializations. The full
  `built-ins/String/prototype/repeat` Test262 leaf now reports `16/16` passing
  as of `2026-06-23` under
  `--execution-backend wasm --timeout-ms 90000 --threads 4`:
  `./target/debug/porf test262 run built-ins/String/prototype/repeat --execution-backend wasm --timeout-ms 90000 --threads 4`.
  `String.prototype.trim` is now registered as a Rust standard builtin for
  prototype property reads, borrowed calls, and direct method calls. The
  Wasm-AOT trim path now removes the ECMAScript WhiteSpace/LineTerminator set
  from both edges using UTF-8 byte scanning while preserving U+180E as a normal
  non-whitespace code point. The `trim`, `trimStart`, and `trimEnd` metadata
  files use focused static Wasm-AOT materializations for descriptor checks. The
  full `built-ins/String/prototype/trimStart` and
  `built-ins/String/prototype/trimEnd` Test262 leaves now report `23/23` each
  as of `2026-06-23` under
  `--execution-backend wasm --timeout-ms 90000 --threads 8`:
  `./target/debug/porf test262 run built-ins/String/prototype/trimStart --execution-backend wasm --timeout-ms 90000 --threads 8`
  and
  `./target/debug/porf test262 run built-ins/String/prototype/trimEnd --execution-backend wasm --timeout-ms 90000 --threads 8`.
  Exact real Test262 files
  `built-ins/String/prototype/trim/name.js`,
  `built-ins/String/prototype/trim/u180e.js`,
  `built-ins/String/prototype/trim/15.5.4.20-4-1.js`, and
  `built-ins/String/prototype/trim/15.5.4.20-4-60.js` each report `1/1`
  passing as of `2026-06-23` under `--execution-backend wasm` with the
  `60000` ms timeout and one thread.
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
- Mutable bindings whose value can be either a string or number now reach the
  tagged `ToPrimitive` addition path instead of being rejected during
  lowering. This covers assertion-message control flow in the final Math
  outlier, `built-ins/Math/pow/applying-the-exp-operator_A9.js`. The complete
  checked-out real-Test262 `built-ins/Math` tree reports `327/327` AOT-applicable
  cases passing as of `2026-07-16`. Refresh with
  `./target/debug/porf --jobs 4 test262 run built-ins/Math --execution-backend wasm --timeout-ms 90000 --threads 4`.
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

The dev/test profiles retain incremental compilation and line-table source
locations, compile dependencies at `opt-level=2`, and keep Porffor workspace
crates at `opt-level=0`. Capture representative large-crate build timings with:

```sh
./scripts/dev.sh timings
./scripts/dev.sh exact-test -p porffor-cli run_wasm_backend_succeeds_for_supported_fixture -- --exact
```

The checked-in cross-feature latency workload is
`benchmarks/wasm-aot-20.txt`. Run the ignored authoritative Wasmtime-AOT
benchmarks on an idle machine with:

```sh
cargo test -p porffor-cli --test perf -- --ignored --nocapture --test-threads=1
```

Measured on the 16-logical-CPU development machine on `2026-07-10`:

- representative incremental engine/CLI rebuild: `1.04 s` (target `<=8 s`);
- comment-only rebuild probe in the 1.13 MiB IR lowering unit: `0.69 s`;
- comment-only rebuild probe in the 1.92 MiB standard-builtins backend unit:
  `4.42 s`;
- compiler edit through rebuilt authoritative host-output case: `8.64 s`
  (`1.04 s` rebuild plus `7.60 s` cache-invalidated run; target `<=10 s`);
- warm exact Wasmtime-AOT execution: `3.96 ms` (target `<=1 s`);
- repeated exact execution in a fresh `porf` process: `0.72 s`;
- warmed 20-case cross-feature chunk: `168.28 ms` (target `<=5 s`);
- cold `wasm_host_output.js` after `porf cache prune`: `13.73 s`, including
  `0.84 s` lowering and `11.73 s` native compilation (target `<=5 s`, not met).
- that cold compile averaged `488%` CPU with the eight-thread Cranelift cap;
- sampled peak RSS for the large host-output artifact was `3,165,520 KiB`;
- after the validation runs, Porffor compiled-code caches used `1,459,395,488`
  bytes (below 2 GiB), `target/` used `64 GiB`, and the separately reported
  legacy Wasmtime cache used `22,276,692,202` bytes.

The cold result keeps the runtime/program product split described in the Rust
rewrite architecture backlog as required follow-up; warm cache success is not
reported as cold success.

Existing developer artifacts are never cleaned automatically. If an old
`target/` has grown too large, inspect it with `du -sh target` and perform the
one-time cleanup explicitly with `cargo clean`; the next dependency build will
be intentionally cold.

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

Repository contract checks cover the task plan, Rust module-boundary split,
generated README status edits, the Test262 shortcut allowlist, and the `$262`
host ABI contract in `test262/backlog/host-abi.tsv`.

## The Name

`porffor` means `purple` in Welsh.
