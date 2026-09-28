# Pinned search-collation data

This offline Rust tool exports the search collations omitted from ICU4X's
default compiled data. It uses ICU4X 2.0.0 with CLDR 47.0.0 and ICU export
`icu4x/2025-05-01/77.x`, matching the runtime Collator data generation.
Only search attributes of `CollationMetadataV1` and `CollationTailoringV1`
are exported; ordinary collation and normalization data remain in ICU4X's
compiled provider. All available search locales and their inherited root
rules are retained.

The original ICU export omits tailored Hangul jamo trie entries. The checked-in
export repair restores those entries using the pinned exporter source and a
small patch, documented in
`crates/lila-intl/data/collation-search-icu77/README.md`. Before writing the blob,
this Rust tool compares both data providers over every Unicode code point:
only U+1100–U+11FF mappings may differ, and all CE and context arrays must match.

From the repository root:

```sh
python3 scripts/generate-intl-collation-search.py --check
python3 scripts/generate-intl-collation-search.py --verify-generated
python3 scripts/generate-intl-collation-search.py --refresh
```

To verify the exporter from its pinned C++ source as well, first run
`python3 scripts/generate-intl-collation-search-export.py --verify-generated`.
This requires Clang and Make and uses the same bounded build scope.

`--check` verifies the manifest without compilation. The other commands
download source ZIPs into `target/intl-collation-sources`, verify their pinned
SHA-256 hashes, and run this separate, locked Cargo project. Use the normal
bounded build scope for these commands; they must not run alongside workspace
Cargo verification. `--verify-generated` compares all generated files byte for
byte, while `--refresh` updates the files and manifest. The Rust tool accepts
only local ZIP paths (CLDR, repaired ICU export, original ICU export); the Python
wrapper owns source verification and overlays the checked-in repair data.

The generated tables and Unicode license are in
`crates/lila-intl/src/provider/collation_search/generated`. The manifest binds
the source URLs and hashes, generator code and lockfile, generated tables, and
license. Source ZIPs are cached locally rather than duplicated in the repository.
