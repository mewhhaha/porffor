# Conformance harness maintenance — 2026-10-21

H045–H060 are complete, bringing the authorized maintenance inventory to 61 of
100 hills. The [source-bound record](maintenance-conformance-wave-20261021.json)
retains every failed attempt, corrected checkpoint, inventory and artifact hash.
This wave starts at `243c7feb5`; implementation is `5b92e16aaca7963ca582ada93e6fe1223545dc03`.
It changes no published Test262 totals.

## Changes

- H045 classifies typed engine failures before reading diagnostic prose. H046
  admits publication recovery only for typed missing/incomplete aggregates;
  invalid or ambiguous evidence remains a hard error. H054 similarly types stale
  checkpoint envelope reasons without treating ordinary corruption as stale.
- H047 validates the README marker census and order, then replaces it atomically.
  H048 writes immutable JSON/TXT members and selects their checked generation by
  one atomic manifest. Graph/readme tooling verifies both members and never
  falls back from a corrupt manifest. Legacy pairs remain readable only when
  no manifest exists. Publication failure controls preserve the prior generation.
- H049 gives decoded snapshots one admission owner. H050 removes duplicate
  public runner facades. H051 gives each reused worker one session/retirement
  owner, including timeout, malformed response, case limit and cleanup paths.
  H052 preserves a known primary case failure through secondary cleanup errors;
  a cleanup failure after success cannot be reported as a pass. H053 makes
  backend, execution-stack and agent dispatch exhaustive.
- H055 validates the frontmatter fields actually consumed by discovery, while
  retaining unrelated descriptive metadata. H056 uses a conservative text gate
  followed by actual request discovery, retains that scan, and lowers finite
  prepared sources only when the parsed source requires it. Scripts without
  module requests keep their in-memory execution path.
- H057–H059 distinguish module IO errors from missing imports, bind location and
  content kind once, and name request-row fields. H060 shares sorted traversal
  without swallowing filesystem metadata errors.

## Verification and corrections

Locked all-target/all-feature CLI and Test262 Cargo check passed. All 669 tooling
tests passed in 176.490 seconds with zero skips, along with publication artifact
guards, 53 Test262 structure checks, module-boundary guards and all 15 Intl source
identity recipes. The full pinned frontmatter census admitted 53,910 files and
102,994 execution IDs with zero rejections. The first census exposed 22 valid
four-space headers; admission now accepts consistent positive-space indentation.

The first complete selected native checkpoint found six execution regressions:
the broader module text gate required a file path before discovering whether a
Script actually requested modules. Request discovery now happens first and its
result is reused. The other failures concerned contradictory checkpoint fixture
expectations and new tests that expected an unnecessary catalog or JavaScript
error text the oracle deliberately keeps opaque. Their final assertions retain
loading-denial, integrity, typed classification and opaque-reporting controls.
All original failure logs remain in the record.

The corrected production checkpoint passed all **856 CLI integration tests,
91 CLI unit tests and 29 adjacent harness/publication tests**. The final library
checkpoint passed **493 Test262 library tests**, with zero failures or ignored
tests. Only test expectations changed between those checkpoints; both receipts
prove their source stayed unchanged during execution. Together they cover 1,469
selected native tests. This is affected-scope verification, not a complete
workspace, release or original Test262 campaign. The four tracked upstream
disagreements remain ordinary failures.

Raw evidence is retained under
`target/maintenance-remaining-20261021/conformance`.
