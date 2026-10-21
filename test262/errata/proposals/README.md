# Proposed upstream resolutions

These are **local drafts, not submitted or adopted**. They do not change Scriptw,
the vendored tests, source pins, or published conformance counts. The exact input,
patch and proposed-output hashes are in [manifest.json](manifest.json).

| Draft | Intended destination | Purpose |
| --- | --- | --- |
| [Arguments test](test262-arguments.patch) | Test262 | Update the stale expectation to the already accepted 2017 normative change; add an explicit-formal control. |
| [Johnston test](test262-johnston.patch) | Test262 | Move Johnston to the existing acceptance/name-preservation group; retain the other aliases' equality/offset assertions. |
| [Historical Zone branch](ecma402-backzone-zone.patch) | ECMA-402 | Define the missing backzone-Zone branch without crossing country boundaries. |
| [Labelled declarations](ecma262-labelled-functions.patch) | ECMA-262 | Propose recursive labelled lexical collection and Annex B copies through labels, matching the measured engines and original staging test. |

Ready-to-review problem statements:

- [Implicit arguments](arguments-report.md)
- [Timezone identifiers](timezone-report.md)
- [Labelled functions](labels-report.md)

All four patches were dry-run and applied without fuzz in isolated copies, then
checked against their exact proposed-output hashes. The Test262 patches produce
the same bytes as the two corresponding explicit fixture amendments. The
specification patches are proposals, not executable correctness proofs.

The label proposal changes the pinned specification's behavior. It is an
**alternative** to the existing local label-test amendment, which follows the
literal pinned wording. Do not adopt both. If the normative proposal is accepted,
update the specification authority, implement its general declaration rules,
and restore the original staging test as the regression. Until then, that
original failure stays open.

The timezone proposal makes the provider's existing general historical-Zone
policy explicit. It does not establish that the current specification already
contains that branch. Its test correction still needs upstream review.

[Standalone probes](../reproducers/README.md) and
[execution evidence](../../../docs/rust-rewrite/final-four-resolution-20261021.md)
record engine agreement and disagreements. Temporal is absent from the installed
comparison engines; no timezone differential success is claimed.

Before adoption, require the authoritative resolution, passing targeted and
adjacent tests, and a fresh complete native suite publication. No original
failure is closed by merely applying these drafts to a local copy.
