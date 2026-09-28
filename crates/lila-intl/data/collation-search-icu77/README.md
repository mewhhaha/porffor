# Pinned search collation Jamo export

These 22 TOML files repair the search tailoring data exported by ICU's
[`icu4x/2025-05-01/77.x`](https://github.com/unicode-org/icu/tree/594c9b890abfa794a259162f1a29513445365f89)
tag. Despite its tag name, that source declares ICU version 78.0.1. The
original release ZIP is retained as the base source; these files replace only
its `collation/implicithan/*_search*_data.toml` entries for ICU4X 2.0.0 blob
generation.

The tag's `genrb -X` exporter drops every U+1100–U+11FF mapping from all
collation tries, but emits a separate Jamo table only for the root standard
collation. [`source.patch`](source.patch) preserves the root standard table and
retains Jamo mappings in the other tries. The 22 search entries are the complete
set in the pinned ZIP. Export comparison requires all non-trie fields, all
metadata, and the root standard data to remain byte-identical to that ZIP.
The blob generator also checks that the ICU4X lookup differs only within the
Jamo block.

Run `python3 scripts/generate-intl-collation-search-export.py --check` for a
local identity check. `--verify-generated` rebuilds the patched export from
the pinned source archive and compares every file. `--refresh` rebuilds and
updates the checked-in files and manifest. Then run
`python3 scripts/generate-intl-collation-search.py --refresh` to regenerate the
ICU4X blob. Both generators verify pinned source SHA-256 values.

The data and patch are under the ICU license in [`LICENSE`](LICENSE).
