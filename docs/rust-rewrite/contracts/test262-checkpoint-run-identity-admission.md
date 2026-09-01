# Test262 checkpoint run-identity admission

Status: implemented as a T03 harness-integrity invariant on 2026-08-27.

## Admitted domain

`CheckpointRunIdentity` is the opaque, producer-owned pairing of a terminal
run kind and matrix path. Its private parser admits exactly these states:

- `full` with an empty matrix path;
- canonical one-based `shard-I/N` with an empty matrix path and
  `1 <= I <= N`;
- `matrix-filter-leaf` or `matrix-chunk-leaf` with a non-empty matrix path.

The persisted representation remains the existing
`{ terminal_run_kind, matrix_path }` object. Deserialization first reads that
untrusted object into `CheckpointRunIdentityWire`, then calls the fallible
parser. Invalid field combinations therefore cannot cross the snapshot input
boundary as a `CheckpointRunIdentity`.

## Product ownership

Full, shard, and matrix execution construct the admitted identity through
named factories. `execute_cases`, periodic checkpoint writes,
`ResumeCheckpointIdentity`, and direct snapshot matching accept the opaque
identity instead of independent strings and paths. Resume comparison uses the
complete identity, so neither half can be substituted or forgotten at a
forwarding seam. The former repeated `validate()` calls are absent: validation
happens once during admission, not conditionally at later consumers.

## Evidence and limits

The focused structure guard pins the private fields, manual deserialization,
fallible parser, named producers, typed forwarding seams, and absence of a
derived or repeated validation escape hatch:

```console
cargo test -p lila-test262 --test checkpoint_run_identity_admission_structure
cargo test -p lila-test262 tests::direct_terminal_and_checkpoint_identity_are_exactly_bound -- --exact
```

This invariant changes neither snapshot JSON shape, selected cases, result
classification, materialized Test262 source, nor conformance counts. It does
not close T03's remaining semantic materialization debt.
