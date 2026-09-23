# Pinned NumberFormat profiles

This directory contains the complete NumberFormat profile derived from CLDR
47, commit `2ef784e3a4168bc2a43cd1b5b9839b6636f5899c`, and Unicode 16.0.0
properties. The pinned CLDR tools select ICU 77.1; the pinned ICU Unicode
version is 16.0. ICU source is evidence for specific selection rules, not a
runtime dependency or an output oracle.

The profile contains 1,082 canonical formatting locales, all 77 numeric
numbering systems, 307 currency labels, standard currency fraction defaults,
all 45 sanctioned simple units, and general per-unit composition in three
widths. Equivalent immutable profiles and tables are interned. The inventory
contains every real main locale other than root. One upstream default-content
entry, ife_TG, has neither a main file nor an ife parent in the complete pinned
Git tree; coverage.json records that source inconsistency explicitly. Missing
required content in a real main locale fails generation.

sources.tar.gz retains exact source bytes and licences. source-manifest.json
records archive and member SHA-256 hashes, URLs and pinned Git blob identities.
profile-provenance.json.gz records each selected leaf's locale, path, draft
status and content hash, with interned selection sets per requested locale.
The locale selected for formatting remains distinct from each leaf's owner.

Regenerate or compare offline from the repository root:

```sh
python3 scripts/generate-intl-numberformat-profile.py --verify-sources
python3 scripts/generate-intl-numberformat-profile.py --check
python3 scripts/generate-intl-numberformat-profile.py
```

The last command rewrites generated output only after the complete candidate
inventory succeeds. The binary contains no JavaScript or executable code. A
closed decoder validates references, ordering, cardinality, scalar values,
pattern roles, Unicode intervals, plural predicates and the complete locale
inventory before exposing NumberProfiles.

Numeric skeleton precision is deliberately consumed by the exact numeric
kernel's already validated options. CLDR skeletons supply grouping structure;
they cannot override the ECMAScript digit/rounding settings. Scientific and
engineering notation use the exact exponent and locale exponential symbol.
CLDR's legacy scientific-format skeletons are not a second precision policy.

Inheritance follows the pinned main-parent/default-content and alias graph,
including lateral count/gender/case/alt fallback, inheritance markers and the
selected draft/alt policy. Proposed alternatives are excluded; narrow and
alphaNextToNumber alternatives remain reachable. Approved, contributed,
provisional and unconfirmed leaves are retained with their draft provenance.
Component plural and
compound grammar inheritance uses its own declared component policy.

Currency-name placement is a locale resource selected from the locale's
default numbering system, independently of an explicit nu option. Missing
resources inherit through the locale chain, using each ancestor's default
system. Compact sets can fall back to the same locale's latn set, then from
long to short; this changes neither the negotiated digits nor locale identity.
Unknown well-formed currency codes use the code itself and the same generated
DEFAULT fraction precision. Cash rounding is not used by NumberFormat.

See docs/rust-rewrite/intl-numberformat-provider.md for exact operand,
composition, range and output-ownership policies. This data alone does not
publish an Intl intrinsic or claim product-level NumberFormat conformance.
