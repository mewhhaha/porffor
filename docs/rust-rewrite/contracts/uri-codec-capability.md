# URI codec capability boundary

`UriBuiltin` owns the six global URI and Annex B entry identities. Its Encode
and Decode cases carry the private `UriCodecKind::{Uri, Component}` choice, so
direction and codec scope cannot be selected independently.

Neither domain implements cloning, copying or equality. The builtin dispatcher
consumes its operation once. URI encoding borrows the codec while the emitted
loop selects its exact unescaped punctuation set; URI decoding consumes it in
one exhaustive match that distinguishes reserved-escape preservation from
Component decoding. Adding another codec is therefore a compile error until
both independent projections define its behavior.

The bounded `uri_builtin_codec_domain_structure` guard pins the four named
codec producers, the single operation dispatch, both exact codec identities,
the borrowed encoder projection, the consuming decoder projection, and the
absence of fallback arms and incidental capabilities.

This is a Rust authority change only. It does not change string coercion, URI
encoding or decoding, malformed-input errors, Realm selection, Annex B escape
behavior, emitted Wasm, Test262 materialization or published conformance counts.

```sh
cargo test -p lila-aot-wasm --test uri_builtin_codec_domain_structure
cargo test -p lila-cli language_numerics::run_wasm_backend_succeeds_for_uri_codecs_fixture -- --exact --test-threads=1
```

Batch AP makes `UriBuiltin`, its named codec constants and the raw compiler
private to `builtins/uri.rs`. The standard dispatcher sees only six fixed
semantic wrappers for escape, unescape, encodeURI, encodeURIComponent,
decodeURI and decodeURIComponent. This is a source-equivalent boundary
tightening with no new URI or Annex B behavior. Batch AP verification is green
on 2026-08-28: the strengthened URI structure target and adjacent Annex B
output-coordination guard pass `4/4` and `3/3`, both exact URI and Annex B CLI
controls pass `1/1`, and `cargo xc` is green. The former 51-line owner has SHA-256
`100f2f6d900179e38b1cb5b55251b2eb05791574b32511addcdc7c0be3d24a05`;
the private policy plus six fixed wrappers form a 95-line owner with SHA-256
`babe4ee150351202de89ab34aff52e0e8864441b53ef3713ef5c637ea9aa4fef`.
