# Continue Scriptw

Start with the [current inventory](docs/rust-rewrite/current-inventory.md) for
measured results, source identities and outstanding acceptance gates. Use the
[batch workflow](docs/rust-rewrite/batch-workflow.md) for commands and resource
policy. The [maintenance backlog](docs/maintenance-hills.md) tracks cleanup work.

## Completed maintenance batch

H001–H004, H011, H017, H024 and H091 are complete. The
[verification record](docs/rust-rewrite/maintenance-first-wave-20261021.md) binds
implementation commits and checks, including the original failed attempts and
successful corrections. The remaining 92 maintenance hills are candidates.
Continue with the broader acceptance queue below; do not treat this cleanup as
full-suite or release acceptance.

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
