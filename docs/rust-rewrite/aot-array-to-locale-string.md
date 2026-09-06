# Observable Array `toLocaleString` lowering

## Integration of PRs #19, #20 and #21

The integrated emitter uses the shared `emit_array_like_length_snapshot` owner
from #19 rather than a second inline length algorithm. It retains #19's ordinary
Arguments descriptor Get repair and #21's override-preserving iterator lookup,
while preserving #20's shared live indexed Get and universal non-nullish element
Invoke. All 23 length, 24 observable-locale, and 12 Arguments engine regressions
remain unchanged. Ownership guards now verify the single shared indexed Get
rather than obsolete private routing flags.

The historical evidence below is not a substitute for validation of this
combined revision. Complete runtime inventories, retained fixtures, pinned real
subtrees, and broad CI must pass before merge. No Test262 source, expected
outcome, skip, pin, generated status block, or conformance denominator changes.


This change targets the Rust Wasm-AOT implementation, not the retired JavaScript
implementation and not a fallback evaluator. It is a focused conformance change;
it does not establish full ECMAScript or Test262 compatibility.

## Algorithm ownership

`compile_to_locale_string_builtin` retains two distinct entry policies:

- **ArrayLike:** reject a nullish receiver, perform `ToObject`, then one public
  `Get(receiver, "length")` and `ToLength`. Do not substitute array storage length,
  arguments storage length, or a TypedArray private witness for this operation.
- **TypedArray:** retain brand validation and the validated method-entry witness.
  The direct TypedArray method must not read a shadowing public `length` getter.

The captured iteration bound is stable. Every iteration performs a live indexed
`Get` through the existing shared object dispatch. Null and undefined contribute
an empty substring, but still participate in comma placement. There is no
`HasProperty` filter and no raw arguments-buffer read in this loop.

For every non-nullish element, preserve its original receiver, box only for the
property lookup, get `toLocaleString`, propagate any abrupt completion, validate
callability, invoke the existing validated call protocol, and stringify its
result. Arrays and arguments objects must not bypass this protocol based on
their storage tag. The receiver passed to a strict primitive method/getter is
still the original primitive, not the temporary boxed lookup object.

The entry enum and validated invocation type remain the existing compiler
owners. No additional compatibility representation or alternate runtime path is
introduced. This change does not expand the existing locale-argument protocol
or claim ECMA-402 support.

## Regression and conformance gates

The engine target `aot_array_to_locale_string_observable` contains 24 tests.
It explicitly requests `ExecutionBackend::WasmAot`, with one compilation worker.
Run every compiled test through the repository's existing inventory runner so
that each test has a fresh process, a timeout, and a retained result:

```sh
cargo fmt --all -- --check
cargo check --locked --workspace
cargo test --locked -p lila-aot-wasm --test array_to_locale_string_observable_structure
LILA_MODULE_MEMORY_CACHE_ENTRIES=2 python3 scripts/run_engine_regression_inventory.py \
  aot_array_to_locale_string_observable --output-dir /tmp/locale-engine --timeout 120
cargo build --locked -p lila-cli
./target/debug/lila test262 run built-ins/Array/prototype/toLocaleString/ \
  --execution-backend wasm --threads 2 --jobs 2 --timeout-ms 60000 \
  --snapshot-dir /tmp/locale-test262 --snapshot-name array-toLocaleString
./target/debug/lila test262 run built-ins/TypedArray/prototype/toLocaleString/ \
  --execution-backend wasm --threads 2 --jobs 2 --timeout-ms 60000 \
  --snapshot-dir /tmp/locale-test262 --snapshot-name typed-array-toLocaleString
```

The dedicated workflow runs the complete engine inventory and both complete
pinned real Test262 subtrees; it retains inventories and execution evidence.
No test is ignored, filtered out of the engine inventory, or converted into an
expected failure. Broad pinned-suite status remains a separate measurement.
Do not edit published conformance counts based on these focused tests.

## Specification references

- ECMA-262, Array.prototype.toLocaleString:
  <https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.tolocalestring>
- ECMA-262, %TypedArray%.prototype.toLocaleString:
  <https://tc39.es/ecma262/multipage/indexed-collections.html#sec-%typedarray%.prototype.tolocalestring>

## Verification boundary for this proposed change

The patch-authoring environment checked all 24 JavaScript assertions against
Node v22.16.0 and reproduced the own-length and nested-array-method defects in
a recovered pre-change Wasm-AOT CLI. Neither check executes the modified Rust
compiler. Rust compilation, rustfmt, the engine regressions on the patched
backend, and the two real Test262 subtrees must pass before merge. Broader
conformance counts are intentionally unchanged.

## Review follow-up: property equality, callable setters, and error Realms

The shared indexed-Get helper now receives the trusted method Realm through its
existing ABI slot 6. Its closed helper-domain classification includes that
argument in the object-read and Proxy-call projections; arbitrary user lexical
environments still select the main-Realm fallback. Two additional explicit
Wasm-AOT regressions cover revoked indexed receivers and callable-Proxy getters
with foreign Array locale methods. The observable locale inventory is therefore
26 tests; no earlier test is removed.

The broad retained Array CLI inventory exposed two existing object-model bugs.
The public own-descriptor builtin compared virtual string keys by payload
identity, so a computed `length` missed Array and Arguments descriptors. All
virtual String key predicates in that owner now reuse property-key equality,
which compares string content and preserves distinct Symbol identity. The
OrdinarySet indexed-accessor branch also used a Function-only call after accepting
a callable Proxy. It now shares the Proxy-aware call protocol and propagates the
original abrupt completion before publishing success, retaining the explicit
Receiver and one assigned argument.

The four-test `aot_array_property_regressions` inventory covers computed keys,
descriptor attributes and deletion, non-invoking accessor reflection, Symbol
non-aliasing, dense/sparse Array and Arguments Proxy setters, apply traps,
Reflect.set receivers, thrown identity, revocation, and absent-setter behavior.
The original `wasm_array_hasown_length.js` and
`wasm_array_index_accessor_setter.js` CLI fixtures remain unchanged in the full
Array CLI selection. Structural guards pin the shared equality and call owners.
Final-head execution, not these source assertions, determines the CI outcome.
