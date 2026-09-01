# Differential oracle compile boundary

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
