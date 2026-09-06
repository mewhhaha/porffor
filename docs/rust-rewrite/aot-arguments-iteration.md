# Arguments iteration: bounded repair and the next measurement gate

Owners: **T09** (Arguments exotic objects), **T15** (iterator consumers),
**T01** (the separate full-suite baseline deliverable).

## Regression and invariant

On baseline `abdcf56f1bd88d5debbb1d8c291f2e7213f77371`, the direct Wasm-AOT
product throws `TypeError: for-of target is not iterable` for this program:

```js
function collect(a, b, c) {
  var result = "";
  for (var value of arguments) result += value + ":";
  return result;
}
collect(2, 1, 3); // must complete with "2:1:3:"
```

The retained `wasm_backend_arguments_iterators_observe_length_truncation` test
passes on that baseline. It exercises explicit `arguments[Symbol.iterator]()`
and exhaustion, not the failing loop acquisition path. It is an adjacent
control, not evidence that the regression is absent.

The ordinary and async-disposable synchronous `for-of` emitters used only a
generic object lookup for `@@iterator`. Arguments uses a dedicated lookup in
the current exotic representation. Both loop emitters now dispatch on the
**runtime Arguments tag** and reuse `emit_arguments_iterator_method_to_locals`,
including when Arguments escapes through an alias. Other receivers retain the
existing symbol-key lookup and original receiver. Callability checks, cached
`next`, per-iteration environments, disposal, and IteratorClose are unchanged.

This is not a rewrite of Arguments construction or iteration. In particular,
own `@@iterator` overrides/deletion/descriptors and created-realm iterator
identity remain separate known gaps. No Test262 source, pin, prelude,
materializer, exclusion, or expected-failure ledger is changed.

## Reproducible focused verification

The complete `aot_arguments_iteration` integration target explicitly runs
`ExecutionBackend::WasmAot`. Its async test asserts the completed output
transcript, so a rejected promise or a missing continuation cannot pass merely
because the initial script returned successfully. The first regression does
not reference `Array` or `Symbol`, avoiding incidental bootstrap dependencies.

```sh
cargo fmt --all -- --check
python3 scripts/tests/test_engine_regression_inventory.py
python3 scripts/run_engine_regression_inventory.py aot_arguments_iteration \
  --output-dir /tmp/arguments-engine
cargo test --locked -p lila-aot-wasm \
  --test for_of_string_iterator_protocol_structure \
  --test direct_sync_for_of_protocol_error_realm_structure \
  --test plain_async_for_of_await_using_structure \
  --test synchronous_using_for_of_structure \
  --test sync_iterator_locals_release_ownership_structure \
  --test sync_iterator_consumer_capability_structure \
  --test math_sum_precise_runtime_structure
cargo test --locked -p lila-engine --lib \
  wasm_backend_arguments_iterators_observe_length_truncation -- --nocapture
cargo build --locked -p lila-cli
./target/debug/lila test262 list language/statements/for-of/arguments-
./target/debug/lila test262 run language/statements/for-of/arguments- \
  --execution-backend wasm --threads 1 --jobs 1 --timeout-ms 60000 \
  --snapshot-dir /tmp/arguments-test262 --snapshot-name arguments-head
```

Run the same pinned real filter on baseline and head, preserving execution
modes, source text, timeout, and denominator. The current pin selects six
`noStrict` executions covering mapped/unmapped traversal, mutation, and
parameter aliasing. These are a focused regression family, not a full-suite
baseline. Exact executed results and source/binary hashes belong in the PR
verification record and CI artifacts; commands above are not claims of passes.

## Next major deliverable: T01's reproducible full-suite baseline

The focused Arguments repair is the next bounded code change. After it, the
priority deliverable is a **complete current-pin Wasm-AOT aggregate plus its
reproducible non-passing backlog**, not another synthetic-fixture percentage or
completed-epic count. T01 remains in progress until that publication exists.

Use the existing T01 tooling rather than invent a second status format:

```sh
cargo build --release --locked -p lila-cli
git rev-parse HEAD
sha256sum target/release/lila
rustc --version --verbose
LILA_BIN=./target/release/lila THREADS=1 JOBS=1 \
  ./scripts/publish-real-status-low-ram.sh wasm-aot t01-full-baseline
```

Keep the source commit, executable SHA-256, toolchain, exact command, suite pin,
harness identity, backend, and timeout/resource settings with the run logs.
Source/executable provenance is an accompanying record, not an invented field
in the existing snapshot schema. Do not resume evidence across compiler
revisions or silently mix configurations.

Acceptance requires every matrix node and execution mode to be accounted for
exactly once, integrity-checked and reconciled to the complete denominator.
`Unsupported`, timeouts, crashes, and bugs all remain non-passing. Preserve the
raw snapshots and generate the deterministic failure backlog grouped by
semantic owner, with unsupported dynamic-code cases identified but not removed
from the denominator. Follow T01's existing `generate-backlog` and snapshot
comparison contracts.

Only that complete verified publication may update the canonical
`published-status-wasm-aot.json` / `.txt` pair and the generated README status
block together. This PR leaves those artifacts, their counts, and T26's gate
unchanged. A credible completion estimate must be based on the resulting
failure inventory and measured repair rates, with uncertainty stated; passing
this small regression family cannot supply that estimate.
