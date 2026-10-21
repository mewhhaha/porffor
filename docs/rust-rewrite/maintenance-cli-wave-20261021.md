# CLI and report maintenance — 2026-10-21

H036–H044 are complete in `59dae70cf`. This nine-hill wave starts at `b996eb721` and brings
the maintenance backlog to 45 completed hills. The [bounded evidence](maintenance-cli-wave-20261021.json)
records source hashes, commands, executable identities, original failures and
their corrected reruns. It does not change published conformance totals.

## Changes

- H036/H038 give public Test262 commands a closed enum, exhaustive dispatch and
  command-specific option admission. Execution consumes an admitted command;
  inventory/report commands cannot silently ignore run-only options. Hidden
  workers and release closure retain their separate admission paths.
- H037 shares option/value consumption with worker and global parsers. H040
  extracts global options in source order in one pass, retaining validation,
  defaults and flag-like value boundaries.
- H039 makes sharding explicit: `test262 shard --shard INDEX/TOTAL [filter]`.
  Both numbers must be positive and the index must fit the total. A positional
  `1/2` is now an ordinary filter. Maintained callers and examples use the new
  spelling; historical procedures remain historical.
- H041 uses one byte-preserving JSONC pass for comments and trailing commas.
  Strings, escapes, Unicode, line positions and token boundaries survive;
  unterminated comments and malformed comma sequences remain errors.
- H042/H044 share complete/partial outcome totals and human count rendering.
  Partial reports retain unobserved work. H043 defines taxonomy variants, wire
  labels and ordering once while preserving lexical JSON map-key order.

## Verification and corrections

Locked all-target/all-feature Cargo check passed. Module-boundary guards, all
15 Intl source identities and three verdict structure controls passed.

The first frozen native checkpoint ran complete CLI integration coverage
(856 IDs, four disjoint partitions), CLI units, Test262 library units and six
adjacent scopes. It found twelve failures:

- One new argument test expected admission to reject an input that the parser
  already correctly rejected. Its input now isolates command admission.
- One CLI rejection changed its diagnostic wording. The shared admission path
  now prints the public command spelling and preserves the established message.
- Eight differential fingerprint expectations and two host-harness inventory
  expectations still described the pre-Scriptw identity. Git and an independent
  hash calculation traced these to rename commit `1e0c82cbb`, before this wave.
  Only those expectations changed: the host asset is 2,219 bytes, with FNV-1a
  `58cac928d5803302`, and differential domains retain the current Scriptw name.
  Corpus inputs, semantic observations and earlier receipts were not rewritten.

The corrected frozen checkpoint passed **478 Test262 library tests, 89 CLI
unit tests and all 14 affected CLI Test262 integration tests**, with zero failed
or ignored tests. Both native receipts confirm their source stayed unchanged.
The earlier adjacent scopes passed: closure (3), fake publication (2), product
artifact execution (2), case deadlines (5), execution lists (4), verdict structure
(3). Fake publication measured the full 191-case fixture suite.

The other 842 CLI integration tests retain their earlier source-bound results;
we did not repeat them after changes limited to test expectations and diagnostic
wording. The full earlier run took 642 seconds; the corrected 14-command subset
took 390 seconds because it executes fixture-suite commands. This is verified
affected-scope closure, not a new complete workspace or full Test262 campaign.
The four tracked upstream disagreements remain ordinary failures.

Raw receipts and transcripts remain under
`target/maintenance-remaining-20261021/cli`; the independent rename audit remains
at `target/maintenance-remaining-20261021/native-check-failure-audit.md`.
