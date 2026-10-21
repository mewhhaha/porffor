# Continue Scriptw

Start with the [current inventory](docs/rust-rewrite/current-inventory.md) for
measured results, source identities and outstanding acceptance gates. Use the
[batch workflow](docs/rust-rewrite/batch-workflow.md) for commands and resource
policy. The [maintenance backlog](docs/maintenance-hills.md) tracks cleanup work.

## Current maintenance work

All 100 hills are authorized. H001–H035 and H091 are complete (36 total).
The [first-wave record](docs/rust-rewrite/maintenance-first-wave-20261021.md) and
[tooling-wave record](docs/rust-rewrite/maintenance-tooling-wave-20261021.md) retain
checks and failed attempts. Continue implementing the remaining 64 hills:
H036–H090 and H092–H100. Investigation entries need measured or characterized
evidence before deciding whether code changes improve the project.

Use coherent CLI/conformance, compiler/runtime and Intl batches, with Luna
agents for read-only exploration/review and root-owned implementation. Push each
verified batch to main. Do not treat maintenance completion as full-suite or
release acceptance; the broader queue below remains open.

## Decisions to retain

- Keep the four original Test262 disagreements as ordinary failures with their
  README explanations and [upstream tracking](test262/errata/proposals/upstream-status-20261021.md).
  Hold local fixture/specification proposals; no upstream posting is authorized.
- Wasm is the only product target. C/native output and a REPL are excluded.
- Canonical Test262 publication stays at its complete measured baseline until
  a new full original-suite campaign authorizes replacement.
- Preserve original failed runs, compiler identities and immutable evidence.
  A passing diagnostic retry does not replace a failed receipt.
- Keep this file and tasks until every acceptance criterion passes.

## Broader continuation after this batch

1. Reconcile the stopped native campaign's receipts before re-admitting remaining
   scopes in a fresh namespace. Saved coordinator state is not a running process.
2. Diagnose the original-budget differential worker timeout with a general fix;
   preserve its original request and failed executable.
3. Finish current-source native/default/release, retention, real shard/resume,
   differential, host reproducibility and calibrated performance gates.
4. Complete the required original Test262 families and native publication. Keep
   unresolved upstream failures visible; do not claim literal green acceptance.

Exact outstanding gates and the missing historical replay input are recorded in
[the inventory](docs/rust-rewrite/current-inventory.md) and
[acceptance criteria](docs/rust-rewrite/conformance-acceptance.md).
