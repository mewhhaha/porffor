# Task and acceptance index

[Current project inventory](../docs/rust-rewrite/current-inventory.md) owns live
capability, verification and remaining-gate status. [CONTINUE](../CONTINUE.md)
owns the next-action queue. [Maintenance hills](../docs/maintenance-hills.md)
tracks bounded cleanup separately from conformance acceptance.

The permanent T00–T29 areas and their criteria are maintained in the
[ownership registry](../docs/rust-rewrite/conformance-ownership.md) and
[acceptance document](../docs/rust-rewrite/conformance-acceptance.md).
The individual task files remain retained; sparse checkouts may omit them locally.

The [task checkpoint history through 2026-10-21](history-through-20261021.md)
preserves the former ledger and its dated verification claims. It is historical,
not an instruction to restart old coordinators or use old compiler receipts.

Wasm is the sole product target. C/native output and a REPL are excluded goals.
The four documented Test262 disagreements remain ordinary failures. Keep this
directory and CONTINUE until the authoritative acceptance criteria pass.
