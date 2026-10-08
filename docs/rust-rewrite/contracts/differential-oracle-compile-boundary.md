# Differential oracle compile boundary

## Current process boundary — 2026-10-07 joined source

Native replay input, mandatory selected-worker configuration and the typed
feature-off result are available in default builds. A feature-off replay
returns `OracleNotLinked` before spawn. The real process supervisor, source
admission/backend worker and named `lila-differential-worker` binary require
`spec-exec-oracle`; the CLI's hidden dispatch occurs before Realm construction.
There is no interpreter product route or parent raw backend execution path.

Goal spellings and the native input fingerprint helpers are also available in
default builds because persisted replay identity precedes worker selection.
They hash the original goal, source, locator, deadline, protocol, graph and v7
limits without constructing an Engine or executing a backend. Execution,
projection, protocol comparison and mismatch signatures retain their explicit
test/oracle gates. The source guard checks these two physical responsibilities
separately, including the feature-off refusal before any worker call.

Cargo controls explicitly select the named worker instead of executing their
own harness. CLI embedder controls select Cargo's `lila` through `--worker-bin`.
V7 native graph wires and checked constructors remain available in default
builds. Rooted comparison, signatures and terminal-limit validation retain the
test/oracle compile gate; actual Script/Module graph execution remains only in
the feature-gated sole backend worker. This adds no default product oracle
route. The affected source guards are authored and unrun for the joined
checkpoint. See the [worker lifecycle](differential-worker-lifecycle.md).

## Historical compile-boundary checkpoint

Status: implemented as a T25 developer-oracle capability boundary.

Differential replay keeps its public schemas and typed feature-off
`OracleNotLinked` result in every build. The execution, projection, comparison
and mismatch-signature machinery is compiled only for unit tests or when the
off-by-default `spec-exec-oracle` feature is linked. This makes it impossible
for a default product build to acquire the interpreter-oracle implementation
through an unused private path while retaining full default unit coverage.

`WireIdentityList::values_mut` is a snapshot-corruption fixture API with ten
callers, all inside the crate's unit-test module. Its `#[cfg(test)]` boundary
keeps production wire values immutable through that method. The uncalled
template-source scanner is deleted; the live quoted-source scanner remains.
The feature-only module-loader fixture is gated separately because only the
feature-enabled oracle test consumes it.

The deleted template-source scanner has SHA-256
`8ed6a8721c8d157ea263418918138258a2e68a26670059923570f814b293b69e`.
The original mutable wire-list accessor has SHA-256
`bcecce80a7145d8c00525efc0bbfe0ec3b3a7110a6b7f8aa1590706231d21a89`.

This boundary changes no corpus schema, report schema, replay result or
feature-enabled oracle behavior. It adds no Test262 materialization,
capability claim or published count.

At the Batch BX checkpoint, default and `spec-exec-oracle` package checks are
green without `lila-test262` warnings; the new boundary target passes `3/3`,
the retained output-policy and backend-ownership targets pass `10/10`, the two
focused comparison units pass `2/2`, and the feature-enabled committed
two-backend replay passes `1/1`.
