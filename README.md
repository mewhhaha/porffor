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
runtime. Do not use it as the source of truth for the Rust rewrite.

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

Currently covered areas include:

- Basic expressions, arithmetic, comparisons, logical/nullish operators, updates, `typeof`, and `void`.
- `var` and lexical bindings, globals, `globalThis`, implicit globals, and common global resolution paths.
- Control flow: `if`, `switch`, `while`, `do while`, `for`, labels, `break`, and `continue`.
- Functions: declarations, expressions, arrows, recursion, closures, default/rest parameters, `arguments`, and common `this` binding cases.
- Objects: literals, property reads/writes, methods, accessors, prototypes, `Object.create`, `Object.getPrototypeOf`, and `instanceof`.
- Arrays: literals, indexed reads/writes, `length`, growth, holes/sparse basics, `Array.isArray`, and focused coverage for `concat`, `flat`, `flatMap`, `every`, `some`, `filter`, `map`, `forEach`, `keys`, `entries`, `values`, and species-sensitive paths.
- Exceptions and abrupt completion: `throw`, `try/catch/finally`, `return`/`finally` interactions, and basic native error objects.
- Constructors/classes: `new`, `new.target`, constructor return objects, bound constructors, class call errors, and some derived/null-heritage behavior.
- Builtins: focused support for `Function.prototype.call/apply/bind/toString`, boxed primitives, `Number`, `String`, `Boolean`, `Error` family basics, selected Annex B string/global helpers, and basic Date behavior.
- Binary data APIs: `ArrayBuffer`, `SharedArrayBuffer` rejection paths, `DataView` numeric accessors, typed-array indexed writes/accessors, and empty `%TypedArray%.from([])` construction.
- Harness/host-oriented helpers used by tests, such as `print` and selected host hooks.

Expected weak or missing areas include full real Test262 coverage, modules,
async/generators, broad iterator semantics, Proxy, RegExp-heavy behavior, Intl,
full descriptor/species semantics, complete typed arrays, complete Date/Temporal
behavior, and many edge cases around exotic objects and cross-realm behavior.

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
