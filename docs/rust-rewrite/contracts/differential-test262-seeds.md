# Test262 snapshot seeds

`Test262SeedPlan` selects a bounded, sorted prefix from two explicitly named,
verified complete Wasm-AOT snapshots. Selection is either candidate failures or
the original comparison owner's newly-green execution IDs. The bridge retains
the eligible and selected counts, original failure and backlog ownership,
compiler identities, pins and manifest identity. It does not publish suite
status or turn an unselected execution into a pass.

Every checked seed retains the original physical source and frontmatter,
execution mode, includes, negative phase/type and original host requirement.
The two canonical full embedded backend harnesses go through the original
`materialize_test`: strict directives precede helpers, raw source stays byte
identical, Module source and Script prelude stay separate, and async/agent host
preludes remain owned by the original runner. Missing helper materialization is
retained as unavailable evidence, not source stripping or an empty replacement.
Custom local harness files and host-only profiles are refused by this bridge.

The seed also binds a path/length-framed SHA-256 inventory of all actual suite
files, including Module dependencies and host fixtures. Replay requires the
original suite and snapshot foundations to remain available and exact; this is
a durable pinned replay input, not a standalone archive of Test262. Seed loading
rederives snapshot membership, metadata, both materializations and the complete
file inventory. Unknown wire fields, changed mode/source/harness, missing
dependencies, changed ownership or changed snapshot provenance fail admission.
Admission bounds the inventory to 200,000 regular files, 512 MiB of file data
and 64 directory levels; symlinks and special files are refused explicitly.

The selected `DifferentialWorkerRunner` uses its sole bounded process transport
with a distinct Test262 request and journal. Header identity binds the selected
executable, current source identity, exact execution ID, backend, nonce and whole
seed fingerprint. Each worker remints the checked seed before executing the
original Test262 evaluator. Negative expectations, async `$DONE`, module loading,
agents and metadata-declared Wasm-AOT unsupported cases use those existing owners.
Dependency mutation during execution invalidates the result. Exact bounded
journal and stderr bytes survive in the replay report; timeout, malformed frames,
crash and cleanup failure remain worker failures.

The observation contract compares Test262 harness outcomes, not arbitrary
post-execution values or whole-program equivalence. Both negative paths retain
Lila compiler preflight; parse/early negatives may pass from the same original
front-end diagnostic before the spec-exec oracle runs. These seeds therefore do
not establish independent parser agreement. The separate `negative-source-v1`
protocol owns independent front-end observations. Matching owned failures stays
red, including unsupported data or host operations; only two actual harness
passes make this replay green.

```
lila differential seed-test262 --base before --candidate after \
  --selection newly-green --max-seeds 16 --output-dir seeds
lila differential replay-test262 --seed seeds/0000.test262-seed.json \
  --output-report replay.json --oracle spec-exec --worker-bin ./lila
```

Seed output starts incomplete, retains each exact per-case input, then publishes
complete only after all selected seeds are durable. Replay reserves an incomplete
report before launching workers and replaces it atomically with the complete
observations. Fresh output locations are required. Controls cover original mode,
harness, dependency and failure ownership, red comparison and journal admission.
This source slice is authored; compilation, controls and selected-worker replay
have not run during the dry batch.
