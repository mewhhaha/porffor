# Shadowed property-read arm removal

Status: implemented as a source-equivalent T02 reachability closure.

`compile_property_read_from_locals` returned immediately to the complete
dynamic property-read dispatcher for `ValueKind::Dynamic`. Its later dynamic
match arm was therefore unreachable, as was a second String arm shadowed by
the earlier exhaustive String-key match. The dynamic return now occupies the
single `ValueKind::Dynamic` match arm, so the outer match expresses one owner
for every value kind and a future duplicate becomes a compiler diagnostic.

The same compile census found two unused broad imports. `RealmRecordLocal`
remains privately owned by `functions.rs` and is imported directly by its
cross-module consumer; the unused crate-root re-export is gone.
`StaticRegExpCompilation` remains the sole `lila_ir` import in `operations.rs`.

The deleted 242-line dynamic arm has SHA-256
`68165b09f3c33dde58a972643a8dd69cf970bca44fff30af6baa600ad1063f76`.
The deleted 20-line String arm has SHA-256
`ed859523f2e4b103fb5b069adf5931321c934efd3ef99f6e6e98b359e63e6c87`.
The removed crate-root `RealmRecordLocal` re-export has SHA-256
`763d09a61590ffcf1b4afeac60d93302e8094d3bab928f822518150cd87a02f1`.
The narrowed `operations.rs` import's original line has SHA-256
`8abe81e3220990ad0a59d373e364761cc6f47981f226475794efe66ddc9a324c`.

This reachability closure has no new JavaScript behavior and changes no
emitted Wasm: both removed match arms were statically unreachable, and the
removed imports carried no runtime semantics. It adds no Test262
materialization, capability claim or published count.

At the Batch BV checkpoint, `cargo xc` is green with no `lila-aot-wasm`
warnings, the focused absence target passes `3/3`, and the three retained
outlined dynamic-property-read module validations pass `3/3`.
