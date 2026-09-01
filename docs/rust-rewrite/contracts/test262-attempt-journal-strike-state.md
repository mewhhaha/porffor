# Test262 attempt-journal strike state

Status: implemented as a T03 harness-integrity invariant on 2026-08-28.

## Admitted domain

`AttemptJournalFile.strikes` stores `CaseStrikes`, whose private payload is a
`NonZeroU32`. Absence from the map is the only zero-strike state. Runtime
charging, quarantine admission, retirement, and test inspection therefore
consume the typed value directly instead of repeatedly translating a raw
integer and implicitly treating zero as absence.

The wire-only `StrikeEntries` retains raw `u32` counts so duplicate JSON keys
remain visible during deserialization. `decode_current_journal` is the sole
admission boundary: it rejects zero, parses each execution-id key, verifies
selection membership, and inserts only `CaseStrikes` into runtime state.

## Evidence and limits

`CaseStrikes` has transparent serialization, so the durable representation is
still a numeric JSON count. The owner witness drives admission and survivor
charging through the product journal path, then verifies that one typed strike
is persisted as `1`:

```console
cargo test -p lila-test262 --test attempt_journal_strike_state_structure
cargo test -p lila-test262 attempt_journal::tests::attempt_journal_persists_typed_strikes_as_numeric_counts -- --exact
```

This invariant does not change the journal JSON schema, strike limit,
quarantine behavior, selected cases, materialized Test262 source, or
conformance counts. It does not close T03's remaining harness-materialization
debt.
