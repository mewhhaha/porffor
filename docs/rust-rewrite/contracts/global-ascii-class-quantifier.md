# Global ASCII class quantifier

Status: implemented for the three static global-match patterns `\d{1}`,
`\d{2}` and `\D{2}`.

## Boundary

The private `builtins/string/global_ascii_class_quantifier.rs` child alone owns
the capability-free
`GlobalAsciiClassQuantifier::{DigitOnce, DigitTwice, NonDigitTwice}` domain and
the raw parameterized emitter. One direct exhaustive match selects a one- or
two-scalar width. A second direct exhaustive match selects whether the emitted
ASCII digit predicate is retained or inverted. The boundary admits neither an
arbitrary integer width nor an independent Boolean polarity.

The String parent can request only the `digit once`, `digit twice` and
`non-digit twice` semantic matchers. It cannot name or construct the raw
quantifier, import the child policy, or call the parameterized emitter. The
child wrappers bind each semantic operation to exactly one variant.

The typed selection does not change scalar decoding, probe advancement,
non-overlapping advancement after a match, one-scalar advancement after a
mismatch, or the final Array-versus-null result decision.

## Durable evidence

`global_ascii_class_quantifier_structure.rs` recursively pins the private
module, exact three variants and lack of convenience capabilities, zero parent
raw-policy names, both exhaustive projections, the retained scan transitions,
the exact semantic-wrapper mapping, 11 child domain mentions and the private
definition-plus-three-call census.

The moved five-line domain, 203-line emitter and combined 208-line owner retain
SHA-256
`2c70e7cfdceb62904b990196833997be1cfb643595987e38f8871942bfc49860`,
`6a7fce3d1705ae08dbd92d96b2046445a07c2740bb314f882d7cf4f6a4320211`
and
`9f97f9a45640274960049b633cd448a8090cabc8925ece61692e71c7b5470f69`.
The complete 47-line semantic-wrapper selection has SHA-256
`7ab3b4b38eee91b8d482df3fa4399c7f9cf25b0bff832d5c61562c415edc8745`.
The resulting 20,671-line parent and 261-line child have SHA-256
`80b9e6796957b0af8b819121339d5690543fdf6cd75d77795f3a796808d0efcb`
and
`7500204a87f75dccd18aa2dc3cf10ce13642f7eb9e6c7c9cdbe943baad7d8240`.
The former eight-line raw caller selections had SHA-256
`4aeeb7173cdbc86f3671ee3802ce780a33c3056997c0fd82d17c5ba6666a8f35`,
`6d366b22680068f69ee8232c9fb5c9a34973321d8e6cca10ae37aeec21b95a10`
and
`433f64c9e4ad06512b2653f92a2730d68d3b3f5ce3961258b068f69b4e2aaae2`;
the narrowed seven-line calls have SHA-256
`1f24cbb10aa4b93cdcec16e2bf5b00310ca4c80f48cba5d63155887085daa02c`,
`2b045167fb5769079c21262e4aecbde88570e7674b8bd7ab6913c0918957bc13`
and
`1d17d363fadae9654f56ea1b6d4d901c4c33b711a8bd6276b3bba7cf5fe816d1`.

## Verification

```sh
cargo test -p lila-aot-wasm --test global_ascii_class_quantifier_structure
cargo test -p lila-cli --test cli string::run_wasm_backend_succeeds_for_string_symbol_hooks_fixture -- --exact --test-threads=1
./target/debug/lila --jobs 1 test262 run built-ins/String/prototype/match/S15.5.4.10_A2_T3.js --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 1 --timeout-ms 60000
./target/debug/lila --jobs 1 test262 run built-ins/String/prototype/match/S15.5.4.10_A2_T4.js --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 1 --timeout-ms 60000
./target/debug/lila --jobs 1 test262 run built-ins/String/prototype/match/S15.5.4.10_A2_T5.js --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 1 --timeout-ms 60000
cargo xc
cargo fmt --all -- --check
git diff --check
```

The raw owner move is byte-equivalent; only the three parent call spellings are
narrowed to semantic wrappers. At the Batch AD shared checkpoint, `cargo xc` is
green, `global_ascii_class_quantifier_structure` passes `3/3`, the neighboring
`postal_code_match_result_shape_structure` passes `3/3`, and the exact
`string::run_wasm_backend_succeeds_for_string_symbol_hooks_fixture` CLI witness
passes `1/1`. The exact `S15.5.4.10_A2_T3.js`, `S15.5.4.10_A2_T4.js` and
`S15.5.4.10_A2_T5.js` leaves each pass `2/2`, for `6/6` total with every failure
bucket at zero. The semantic golden was not run.

## Deferrals

This contract does not generalize runtime RegExp compilation, add Unicode
character-class semantics, retire other static matcher shortcuts, or complete
String, RegExp, T18 or T19.
