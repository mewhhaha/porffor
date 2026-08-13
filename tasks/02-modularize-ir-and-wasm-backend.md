# T02 — Modularize the IR and Wasm backend

**Status:** In progress — major builtin ownership bottlenecks split; broader lowering/emitter seams remain

**Parallel group:** Bootstrap/foundation  
**Depends on:** None  
**Blocks:** Safe parallel work in T04-T24

## Current repository state

Both crates now expose dedicated IR, lowering, analysis, diagnostics,
operations, ABI, heap, object, function, environment, control-flow and builtin
modules, and `./scripts/check-module-boundaries.sh` enforces the highest-value
seams and line budgets. The split remains partial: `lowering.rs`,
`builtins/standard.rs`, several family files and object/operation emitters are
still large implementation stores. Treat the landed boundaries as independent
ownership surfaces, but continue coordinating broad edits to those remaining
hotspots.

### Landed 2026-08-12–13: builtin metadata and family body boundaries

Thirteen previously coupled builtin stores now have separate owners:

- `lila-ir/src/lowering/builtin_shapes.rs` owns 98 pure shape/signature
  constructors. At extraction, `lowering.rs` fell from 39,177 to 31,979 lines;
  subsequent work leaves it at 31,998 lines, below the enforced cap. The moved
  methods have only parent-module visibility except the existing crate test
  hook.
- `lila-ir/src/builtins/catalog.rs` is the single 779-row
  `StandardBuiltinId` registry. One row generates the enum, names, flags,
  function-ID mappings and independent function/global order arrays. Typed
  dense ordinals plus const duplicate/hole/ID checks preserve the deliberately
  different declaration, 779-function and 52-global orders.
- `lila-aot-wasm/src/builtins/object.rs` owns all 34 Object builtin bodies and
  their private helpers. Three grouped choices are closed enums rather than
  generic builtin IDs or booleans.
- `lila-aot-wasm/src/builtins/proxy.rs` owns the three Proxy lifecycle bodies;
  `reflect.rs` remains the separate owner of the 13 Reflect bodies and their
  proxy-trap machinery.
- `lila-aot-wasm/src/builtins/math.rs` owns all 37 Math bodies behind a private
  closed `MathBuiltin` domain. `standard.rs` gives every Math ID its own typed
  one-line delegate, and both Math behavior matches are exhaustive. The
  min/max direction is a two-case private enum rather than a generic builtin
  ID. After the Object, Proxy and Math moves, `standard.rs` has fallen from
  49,179 to 36,807 lines.
- `lila-aot-wasm/src/builtins/symbol.rs` owns all seven Symbol bodies and the
  three shared Symbol receiver/description helpers behind a private closed
  `SymbolBuiltin` domain. `String(symbol)` reaches the one helper it shares
  through a parent-private method; the remaining helpers cannot escape the
  family. The catalog dispatch keeps seven typed delegates, and `standard.rs`
  fell from 36,789 to 36,313 lines without changing an emitted instruction.
- `lila-aot-wasm/src/builtins/bigint.rs` owns all six BigInt intrinsic bodies
  behind a private closed `BigIntBuiltin` domain. The constructor, signed and
  unsigned fixed-width operations, and three prototype methods moved verbatim;
  general BigInt conversion, allocation and stringification helpers remain
  with their existing operation and heap owners. The catalog dispatch keeps
  six typed delegates, and `standard.rs` fell from 36,313 to 35,647 lines.
- `lila-aot-wasm/src/builtins/boolean.rs` owns all three Boolean intrinsic
  bodies behind a private closed `BooleanBuiltin` domain. The constructor keeps
  the same argument/result-local ordering previously shared with Number and
  String, while the two prototype methods keep their boxed-receiver checks and
  realm-local TypeError route together. After the intervening T20 residue
  consolidation, the extraction reduced `standard.rs` from 35,532 to 35,439
  lines.
- `lila-aot-wasm/src/builtins/function.rs` owns the complete five-member
  Function intrinsic family behind a private closed `FunctionBuiltin` domain:
  the constructor and `Function.prototype.{call,apply,bind,toString}`. The
  catalog dispatch keeps five typed delegates, while the moved bodies retain
  their exact instruction and temporary-local order. The extraction reduced
  `standard.rs` from 34,461 to 34,088 lines.
- `lila-aot-wasm/src/builtins/uri.rs` owns all six global URI and Annex-B codec
  wrappers behind a private closed `UriBuiltin` domain. The UTF-8/UTF-16 codec
  primitives remain with their existing `string.rs` owner; only the complete
  global wrapper family moved. Six typed delegates preserve the flat catalog
  dispatch, and `standard.rs` fell from 35,439 to 35,394 lines.
- `lila-aot-wasm/src/builtins/global_numeric.rs` owns both coercing global
  numeric predicate bodies behind a private closed `GlobalNumericBuiltin`
  domain. `Number.isFinite` and `Number.isNaN` remain with the distinct
  non-coercing Number family, while `parseInt` and `parseFloat` remain host
  builtin emitters. The catalog dispatch keeps one typed delegate for each of
  `isFinite` and `isNaN`, and `standard.rs` fell from 35,394 to 35,372 lines.
- `lila-aot-wasm/src/builtins/errors.rs` owns the complete eleven-member Error
  intrinsic family as well as its pre-existing allocation, realm-prototype,
  cause, iterable and throw helpers. A private closed `ErrorBuiltin` domain
  distinguishes the static predicate, the nine constructors carried by the
  existing closed `NativeErrorKind`, and `Error.prototype.toString`; unrelated
  `StandardBuiltinId` values cannot reach this family emitter. Eleven typed
  delegates preserve the catalog dispatch without duplicating the error-kind
  registry, and `standard.rs` fell from 35,372 to 34,948 lines.
- `lila-aot-wasm/src/builtins/json.rs` owns all four JSON namespace bodies
  alongside the parse, reviver, stringify and raw-JSON machinery they already
  consume. A private closed `JsonBuiltin` domain covers `parse`, `stringify`,
  `rawJSON` and `isRawJSON`; hidden static-JSON lowering and runtime helpers
  remain implementation details rather than pretend namespace members. Four
  typed delegates preserve the flat catalog dispatch, and `standard.rs` fell
  from 34,948 to 34,461 lines.

The earlier central feature-enabled CLI compile, which covers `lila-aot-wasm`
and `lila-intl`, and the focused builtin catalog tests pass for the moves that
reached that checkpoint. The source moves were also compared against their
pre-extraction bodies, and the boundary audit prevents these stores from being
folded back into their parents. The later
Proxy move is source-equivalent by a static body comparison and is included in
the green compile checkpoint and product-artifact boundary proof. The Math move
is statically source-equivalent, boundary-checked, and covered by that compile
checkpoint. The later Symbol move is statically source-equivalent and
boundary-checked, passes the centralized feature-enabled CLI compile, and is
covered by the exact String/Symbol hook fixture through the product Wasm
backend. The BigInt move is statically source-equivalent and boundary-checked;
its centralized feature-enabled compile and the exact constructor/fixed-width,
wrapper-coercion and cross-realm prototype behavior checkpoints are green.
The Boolean move is statically instruction-sequence equivalent and
boundary-checked; its compile, focused fixture, and real Boolean shard gates
remain queued behind the active resource-bounded matrix run.
The Function move is an exact 389-line body match after normalizing only the
five closed enum arm headers. Its compile, focused constructor/call/apply/bind/
toString fixtures and real `built-ins/Function` shard remain queued behind the
same matrix run.
The URI move is statically source-equivalent after normalizing only the closed
enum path and rustfmt's block-expression layout, and is boundary-checked; its
compile, focused global-codec fixtures and real URI/Annex-B shard gates remain
queued behind the same matrix run.
The global numeric move is statically source-equivalent after normalizing only
the closed enum path, and is boundary-checked; its compile, focused coercion
and cross-realm fixture gates and real `isFinite`/`isNaN` shards remain queued
behind that matrix run.
The Error move preserves the existing emitter and local-allocation sequences;
its only semantic-free rewrites replace raw builtin-ID tests with the closed
`ErrorBuiltin` and `NativeErrorKind` domains. Its compile, focused constructor,
cross-realm, static predicate and prototype-method fixtures, and real Error,
NativeErrors, AggregateError and SuppressedError shards remain queued behind
the same matrix run.
The JSON move is a verbatim body extraction after normalizing only the closed
enum path and rustfmt layout. Its compile, focused parse/reviver, stringify,
raw-JSON and cross-realm gates, and real `built-ins/JSON` shard remain queued
behind the same matrix run.

### Landed 2026-07-31: the `intrinsics/` boundary

`crates/lila-aot-wasm/src/intrinsics/` now holds per-family realm bootstrap
and property-descriptor installation, extracted from
`builtins/bootstrap.rs::init_builtin_constructor_object`. That function was a
single ~4,760-line body and the worst merge point in the backend: two lanes
adding builtins to unrelated families still collided inside it.

`bootstrap.rs` went from 8,080 to 4,117 lines; 23 dispatch arms became one-line
delegations into 15 family modules. The boundary is enforced by
`check-module-boundaries.sh`.

Every arm moved **verbatim** — installers destructure an `IntrinsicInstall`
context back into the original identifier names (including `builtin`, which
multi-variant arms branch on), so no body text was rewritten. The move was
verified byte-identical across all 527 CLI fixtures with
`crates/lila-aot-wasm/tests/emit_golden.rs`, which matters because property
installation order is observable through `Object.keys` and the ordinary suites
assert on program output rather than emitted bytes.

The earlier intrinsic split left three immediate follow-ups. All now have
bounded owners:

- **Resolved 2026-08-12:** the append-only no-op dispatch is gone. The standard
  builtin catalog requires a closed installer class on every row, and
  `bootstrap.rs` consumes it through an exhaustive installer match.
- **Resolved 2026-08-12:** the parallel `StandardBuiltinId` tables are one
  catalog with compile-time ordering and uniqueness invariants.
- **Resolved for Object, Proxy, Math, Symbol, BigInt, Boolean, Function, global
  numeric, URI, Error and JSON 2026-08-13:** their bodies are family modules; Reflect
  already has the same boundary. Other large inline families should follow the
  same exhaustive-delegate shape.

### Landed 2026-08-12: catalog-owned bootstrap routing

`init_builtin_constructor_object` performs common function/prototype setup for
every initialized `StandardBuiltinId`, but only 34 IDs then run one of 33
family intrinsic installers. Before this seam that distinction was encoded
backwards: the 33 productive arms were followed by no-op arms naming every
other ID. Adding a builtin compiled only after someone appended its name to an
unrelated no-op tail, while the catalog that owned the builtin could not say
whether an installer was required.

The landed seam is a mandatory catalog field whose value is a closed
`StandardBuiltinInstaller` domain. `None` must skip family dispatch; every
other case must be consumed by an exhaustive backend match that invokes the
corresponding installer. This is behavioral routing rather than passive
metadata: omitting the field makes a new catalog row fail to parse, and adding
an installer variant makes the backend fail to compile until it handles the
case. The existing catalog/function iteration and the location of dispatch
after common setup remain unchanged, preserving construction and observable
property-installation order.

The catalog now records 34 productive roots across 33 installer classes and
745 explicit `None` choices. `ArrayBuffer` and `SharedArrayBuffer` deliberately
share one class because their installer branches on the carried builtin ID.
The backend match contains only the productive classes; the former raw-ID
no-op groups were deleted, reducing `builtins/bootstrap.rs` from 4,903 to 4,156
lines. A catalog contract pins the productive root sequence, and the module
boundary audit requires both the mandatory field and the typed backend
dispatch. The focused catalog contract and central feature-enabled CLI compile
are green; broader behavioral suites remain part of this task's acceptance
gate.

## Objective

Split the current monolithic compiler implementation into stable ownership boundaries without changing JavaScript behavior or emitted semantics. At the time this plan was written, `lila-ir/src/lib.rs` and `lila-aot-wasm/src/lib.rs` are tens of thousands of lines and are the primary merge-conflict bottleneck.

## Required module boundaries

The exact filenames may change, but the resulting architecture must expose equivalent boundaries.

### `lila-ir`

- `ir/`: public `ProgramIr`, statements, expressions, functions, classes, properties, shapes, value information and IDs.
- `lowering/`: AST-to-spec-IR lowering, split by declarations, expressions, statements, functions/classes and modules.
- `early_errors/`: checks that are not delegated blindly to parser diagnostics.
- `builtins/`: builtin IDs, metadata, intrinsic ownership and feature registration.
- `analysis/`: scope/capture analysis, static shape/value analysis and unsupported-feature reporting.
- `operations/`: typed representations of shared ECMAScript abstract operations consumed by backends.
- `diagnostics/`: structured diagnostic codes and source locations.

### `lila-aot-wasm`

- `module/`: sections, imports/exports, tables, globals, data and validation.
- `abi/`: tagged values, call/construct convention, completion convention and host imports.
- `heap/`: allocation/layout/string/object/environment storage and memory growth.
- `emit/`: statement/expression/control-flow dispatch.
- `operations/`: code generation for shared abstract operations.
- `objects/`, `functions/`, `environments/`: internal-method emitters.
- `builtins/<family>.rs`: separate builtin families such as array, string, regexp, typed-array, date and intl.
- `intrinsics/`: realm bootstrap and property descriptor installation.

Keep a small `lib.rs` that re-exports the public API and invokes the top-level pipeline.

## Implementation sequence

1. Add characterization tests before moving code: compile representative fixtures, validate Wasm, execute with the project's lower-bound runtime (experimental Wasmtime feature set per `AGENTS.md`), and record observable outputs/completion kinds. Do not gate characterization on engines that lack the lower-bound features (Wasm GC, typed function references, `exnref`).
2. Extract pure data types and constants first, then helpers, then large emit branches.
3. Replace magic global/table/layout numbers with named registries that can assert uniqueness and stable ordering.
4. Use private modules and narrow `pub(crate)` interfaces. Do not make every helper public to avoid import work.
5. Preserve error text only where tests rely on classification; otherwise prefer stable diagnostic codes over brittle strings.
6. Move tests beside the module they cover while retaining end-to-end crate tests.
7. Keep each extraction commit buildable so regressions are bisectable.

## Non-goals

- No new builtin coverage.
- No redesign of the value representation or completion ABI; those belong to T04/T05.
- No bulk formatting of legacy JavaScript code.
- No generated Test262 count changes beyond accidental stale-status cleanup.

## Acceptance criteria

- Both giant `lib.rs` files become orchestration/re-export surfaces rather than implementation stores.
- Feature families have clear files that separate agents can own.
- There are no cyclic module dependencies or duplicate constant registries.
- Public APIs used by `lila-engine` remain coherent and documented.
- Representative emitted artifacts behave identically before and after extraction. If byte identity is not practical, compare imports, exports, validation, output, completion kind and thrown error class.
- Workspace compile time and binary size do not regress materially solely because of module movement.

## Required tests

```sh
cargo fmt --all --check
cargo check --workspace
cargo test -p lila-ir --quiet
cargo test -p lila-aot-wasm --quiet
cargo test -p lila-engine --quiet
cargo test -p lila-cli --quiet
./target/debug/lila test262 run language/wasm/pass \
  --suite-root crates/lila-test262/tests/fixtures/fake_test262/vendor/test262 \
  --execution-backend wasm
```

Also run several previously green real Test262 filters from different families to detect moved-helper regressions.
