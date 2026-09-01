# Test262 verdict command authority

Status: implemented and structure-verified for direct `lila test262 run` and
`lila test262 shard` completion, 2026-08-27. Focused CLI results are recorded
below.

## Boundary

`Test262VerdictCommand::{Run, Shard}` is the private compile-time authority for
the command name included in a direct Test262 verdict error. It has exactly two
producers: the `run` and `shard` command arms. Its sole observation is one
exhaustive spelling projection: `Run` becomes `"run"` and `Shard` becomes
`"shard"`.

The domain derives no cloning, copying, debugging, equality, ordering, hashing
or default-construction capability. A future command can therefore neither
inherit an existing spelling through an equality/default branch nor enter the
verdict boundary through a raw string. Adding a variant requires an explicit
spelling arm.

The command authority is separate from `ConformanceRunVerdict`. The latter
continues to distinguish no evidence, a non-empty passing run and a non-empty
failed run. The CLI converts the typed command to its spelling before matching
that verdict, retains the exact no-evidence and failure messages, and requests
the summary verdict only after the selected run or shard has completed and its
summary has been printed.

## Durable evidence

`crates/lila-cli/tests/test262_verdict_command_structure.rs` recursively pins
the exact five source mentions, private two-row domain, absent capabilities,
exhaustive spelling table, typed verdict consumer and messages, exact two
producers, and summary-before-verdict order.

The focused behavioral witnesses are:

- `frontend::test262_run_exits_unsuccessfully_when_a_case_fails`;
- `frontend::test262_shard_exits_unsuccessfully_and_keeps_failure_snapshot`;
- `frontend::test262_run_exits_unsuccessfully_when_selection_is_empty`; and
- `frontend::test262_run_and_shard_reject_unsupported_and_keep_failure_snapshots`.

The dedicated structure target passes `3/3`. The failed-run, empty-selection,
and unsupported run/shard witnesses pass `3/3`. Independent review found the
strengthened private-declaration and capability guard clean. The focused failed-shard
witness reaches and verifies the expected command-specific failed verdict, but
its separate snapshot-directory assertion currently observes the generated
`test/` directory alongside the JSON and text snapshots (`3` entries rather
than `2`). That snapshot-layout failure is outside this derive-only lane and is
not represented as green evidence. `cargo fmt --all -- --check` is green.

## Nonclaims

This source-equivalent capability closure does not change Test262 discovery,
execution, failure classification, snapshot persistence, the
`ConformanceRunVerdict` domain, process exit policy, backend selection or
published conformance status. It adds no CLI fixture or semantic golden and
makes no broad-suite conformance claim.
