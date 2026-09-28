# Pinned full plural-rules data

This standalone Rust tool exports ICU4X 2.0.0 `PluralsCardinalV1`,
`PluralsOrdinalV1`, and `PluralsRangesV1` from the pinned CLDR 47.0.0 JSON ZIP.
Unlike ICU4X's compiled plural data, the source provider includes all 219
cardinal, 104 ordinal, and 92 range identifiers (the last count includes the
empty `und` range table). The tool checks those inventories and verifies that
every source payload survives a round trip through the exported blob.

Run the Python wrapper from the repository root:

```sh
python3 scripts/generate-intl-plural-rules.py --check
python3 scripts/generate-intl-plural-rules.py --verify-generated
python3 scripts/generate-intl-plural-rules.py --refresh
```

`--check` verifies the checked-in manifest and file digests without compiling.
The other modes verify the CLDR ZIP's pinned SHA-256, run this separate locked
Cargo project, and compare or update the generated blob. Use the normal bounded
build scope for those modes; do not run them alongside workspace Cargo checks.
The runtime provider applies locale fallback for requests outside the source
inventory. Each source identifier has a direct blob entry, so export-time
deduplication cannot erase a locale's own rules.

The generated blob, source inventory, manifest, and source license live in
`crates/lila-intl/src/provider/plural_rules/generated`.
