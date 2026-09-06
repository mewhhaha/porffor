# Shared Array callback iteration

Owner: T16. Baseline: `65d1b70382d03e6bb1ffc17a5394c05125d8bbc5`.

`Array.prototype.map`, `filter`, `every` and `some` now compile through
`builtins/array/callback_iteration.rs`. The four existing standard-builtin
entrypoints remain, but no longer own separate copies of the algorithm. A
closed `ArrayCallbackIterationKind` selects result policy with exhaustive
matches; it does not select a generic-versus-TypedArray receiver mode.

## Observable contract

One shared path performs ToObject and captures LengthOfArrayLike before
IsCallable, optional thisArg projection, and species effects. The captured bound
survives callbacks and source mutations. An own or inherited `length` property
on a borrowed TypedArray is observable; private backing-store extent is not a
substitute. Shared ToLength clamps large inputs without a Wasm conversion trap.
The now-unreferenced private hexadecimal-length parser is deleted rather than
retained as an alternative Number conversion policy.

Each visited index performs live HasProperty, then Get, then Proxy-aware Call
with `(value, index, boxedReceiver)`. A hole does not call the callback. The
shared indexed dispatcher, rather than a copied private-slot or clean-prototype
cache, owns integer-indexed exotic reads after resize or detachment. Abrupt
completion returns before later steps.

Map uses shared ArraySpeciesCreate with the captured length and defines mapped
values at their original indices. Filter uses length zero, tests callback
truthiness, and defines the value captured *before* Call at successive output
indices. Both use CreateDataPropertyOrThrow, not Set: inherited setters are not
invoked, descriptor failures propagate, and custom targets receive no invented
`length` assignment. ObjectDefineProperty is explicitly rooted for minimal
Array-result programs.

Every and Some emit no species operation or result allocation. They use ordinary
ToBoolean and return immediately at their respective false/true condition; empty
inputs return true/false. Callable Proxy validation remains distinct from Call,
including revoked callable Proxies on empty inputs.

The generic Array entry family remains separate from strict
`%TypedArray%.prototype` algorithms. Direct-call receiver classification, complete
argument evaluation, and Iterator helper routing are unchanged.

The algorithms follow ECMA-262's
[Array.prototype.map](https://tc39.es/ecma262/#sec-array.prototype.map),
[filter](https://tc39.es/ecma262/#sec-array.prototype.filter),
[every](https://tc39.es/ecma262/#sec-array.prototype.every), and
[some](https://tc39.es/ecma262/#sec-array.prototype.some), together with
[ArraySpeciesCreate](https://tc39.es/ecma262/#sec-arrayspeciescreate).

## Verification

The explicit Wasm-AOT target `aot_array_callback_iteration` contains 21 regression
programs. Reference evaluation in Node is a check of test expectations, not
product execution evidence. Structural guards pin all four producers, input
operation ordering, exhaustive result policy, live indexed-operation ownership,
complete temporary release, and the descriptor dependency. The inventory runner
executes each exact test in a fresh process, rejects empty, ignored or missing
selections and checks that the complete nonempty inventory ran. Five Python
regressions cover its reporting contract. Existing direct-call argument fixtures
and strict TypedArray witness guards remain in place.

The exact unchanged baseline passed 11 of the original 20 new programs; the
callback implementation passed all 20. The review then reproduced the internal
descriptor bug on that implementation before applying its fix. The corrected
compiler passed all 21 programs, including the new regression. These are focused
engine results, not a full-suite percentage. The revision-specific evidence is
in the [baseline inventory run](https://github.com/mewhhaha/porffor/actions/runs/34023616463)
and [descriptor review run](https://github.com/mewhhaha/porffor/actions/runs/34024035648).

Reproduce the focused checks:

```sh
cargo fmt --all -- --check
python3 scripts/run_engine_regression_inventory.py aot_array_callback_iteration \
  --output-dir /tmp/array-callback-engine
cargo test --locked -p lila-aot-wasm --test array_callback_iteration_structure \
  --test array_map_algorithm_owner_structure --test array_filter_algorithm_owner_structure \
  --test array_every_algorithm_owner_structure --test array_some_algorithm_owner_structure \
  --test array_species_create_operation_evidence_structure \
  --test typed_array_quantifier_family_witness_structure
cargo test --locked -p lila-cli --test cli -- array:: --test-threads=2
```

For each of `map`, `filter`, `every` and `some`, run its entire pinned real subtree
with the same execution modes and unchanged sources on baseline and head:

```sh
./target/debug/lila test262 run built-ins/Array/prototype/map/ \
  --execution-backend wasm --threads 2 --jobs 2 --timeout-ms 60000 \
  --snapshot-dir /tmp/array-callback-test262 --snapshot-name map
```

A complete subtree exceeded the original 60-minute CI job limit while making
steady progress. CI therefore builds the product once, checks its source SHA in
each consumer, and executes all four one-based disjoint shards for each of the
four subtrees. For example, `test262 shard 1/4 built-ins/Array/prototype/map/`
uses the same backend and timeout arguments above; shards `2/4`, `3/4`, and `4/4`
are required too. No test or execution mode is excluded. Every shard must
complete with a nonempty passing verdict. Retained Array CLI fixtures run in a
separate job so their budget is not shared with the engine regression inventory.

Exact executed counts, failures and revision-specific CI evidence belong in the
PR verification record. No full-suite percentage or generated README status is
changed by authoring this batch. A green focused subtree is not full Test262
conformance, and historical materialized-source results are not raw-source proof.

## Review findings and next work

Review found a pre-existing bug in the shared Array-result definition helper:
its internal descriptor carrier inherited Object.prototype, exposing inherited
`get`/`set` getters during ObjectDefineProperty conversion. Reusing it would
also regress map/filter. The carrier now has a null prototype; public
FromPropertyDescriptor objects and Proxy trap descriptors are unchanged.
A regression covers both inherited field names across map/filter/flatMap,
with both default and custom Array species results, and a structural guard
pins the internal carrier's isolation.

The original implementations duplicated callback and species checks, dispatched
Call to function handles rather than general callable Proxies, and bypassed
observable borrowed-TypedArray length with private state. The old structural
guards pinned those copies. The new guards retain entry-dispatch protection but
pin shared semantic owners rather than require the obsolete implementation.

Remaining species copies are Array `flat`/`concat` and the separately owned strict
TypedArray `slice`/`map`/`filter` paths. Their contracts and abrupt order must be
reviewed independently; this batch does not silently redirect them. Broader
direct-dispatch named-property lookup remains separate, as do `forEach`, find
methods, reduce methods, generic dynamic source, and Array-specific Test262
materializers. No Test262 pin, test body, prelude, exclusion, or expected-failure
list changes here.

The highest-priority measurement task remains a complete current-pin Wasm-AOT
failure inventory, grouped by shared semantic owner and separated from explicit
dynamic-code-generation unsupported cases. Only a verified complete publish can
replace the stale generated aggregate and establish distance to literal 100%.
