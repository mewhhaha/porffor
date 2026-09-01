# RegExp range-search ownership

Status: normative for the Wasm-AOT Unicode-property range-pool search.

## Private owner

The matcher stores each canonical Unicode range as an inclusive start followed
by an inclusive end. `RegExpRangeBound::{Start, End}` is the closed authority
for selecting those fields, and its exhaustive projection binds them to byte
offsets zero and four.

The private `builtins/regexp/range_search.rs` child owns that domain, its
projection, the sole raw range-bound reader and the complete binary search.
The parent matcher can request only the semantic
`emit_regexp_unicode_property_mismatch` operation. It cannot construct a bound,
project an offset or call the raw reader. The two retained parent calls select
forward and reverse Unicode-property matching without carrying a raw policy.

The exact 14-line domain/projection and 101-line search/reader selections
retain visibility-normalized SHA-256
`7ac765b2195a8ad7e2935bbfb3da1b9e8e641a63906bb007eb67ea49e0da17b6`
and
`eb9ceaad299ab3277aa5bbf1228776d74098876f79f25bed106179e699489098`;
their combined 115-line selection retains SHA-256
`14e35ba4c6a910319e4e301ded5213315ba30fe537a3d92fcd2c3207b29b7801`.
The resulting 3,661-line parent and 120-line child have SHA-256
`60a443e0f39f719c28871815f5be6c7a7fd638e8389e6070167db05abc09b30b`
and
`c36626fb9c53468a49449012538a9ad32e80c37c9387ee7252d807864f9c9e8f`.
Each unchanged 11-line parent call retains SHA-256
`125fa46e9fab12f49c12f6280b95f618baa2d1cb88fad021fb7fc26029e63ab2`.

The recursive guard and module policy pin zero parent raw-domain and reader
mentions, all five domain mentions, both qualified variants and all three raw
reader sites in the child, plus exactly two parent semantic calls.

## Observable witness

The existing Unicode-property fixture exercises a first-range start, a gap,
the final-range end and the first excluded code point. The neighboring matcher
result and Unicode-sets structure targets protect the retained parent matcher
boundaries:

```console
cargo test -p lila-aot-wasm --test regexp_range_bound_domain_structure
cargo test -p lila-aot-wasm --test regexp_matcher_result_domain_structure
cargo test -p lila-aot-wasm --test regexp_unicode_sets_class_strings_structure
cargo test -p lila-cli --test cli regexp::run_wasm_backend_succeeds_for_regexp_exec_unicode_property_program_fixture -- --exact
```

This is a source-equivalent owner move except for the required `pub(super)`
visibility on the semantic operation. At the Batch AA checkpoint, `cargo xc`
is green, the range target passes `3/3`, the neighboring matcher-result and
Unicode-sets targets pass `4/4` and `7/7`, and the exact Unicode-property CLI
witness passes `1/1`. The emitted-Wasm golden was not rerun.

## Nonclaims

This boundary changes no matcher program, range encoding, search order,
forward/reverse behavior or Unicode data. It adds no RegExp grammar, dynamic
pattern compilation or broader conformance claim.
