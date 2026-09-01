# Reflect property-key conversion

Status: implemented and focused structure-verified 2026-09-01 as a bounded
T11 key-conversion correction.

## Scope

The five legacy Reflect key boundaries in `compile_reflect_get_builtin`,
`compile_reflect_set_builtin`, `compile_reflect_has_builtin`,
`compile_reflect_define_property_builtin` and
`compile_reflect_delete_property_builtin` now consume the full in-place
`ToPropertyKey` authority. Each receives the converted payload and tag rather
than deriving the key kind from the source value's tag.

This distinction is observable when an Object, including a boxed Symbol, is
converted to a Symbol. The internal payload retains the Symbol property-key
marker, and every tagged downstream consumer receives the converted Symbol
tag. Trap arguments receive the unmarked Symbol value exactly once.

## Observable order

Every owner validates its target before coercing the property key. An abrupt
`ToPropertyKey` completion is routed immediately and prevents the target's
internal method or Proxy trap from running. `Reflect.get` and `Reflect.set`
also finish this conversion before applying their optional receiver default.

## Focused evidence

`wasm_reflect_property_key_conversion.js` covers a boxed Symbol, Object-to-
Symbol conversion in all five owners, exact single conversion and exact Symbol
identity in the `Reflect.set` trap, a Function target, an Array receiver and
abrupt conversion identity before every target internal method.

`reflect_property_key_conversion_structure` pins exactly five full conversion
consumers, zero payload-only consumers, converted-tag forwarding, active CLI
wiring, the module-boundary census, this contract and the T11 ledger entry.

The write-phase marker `Verification pending` is retained here only as the
historical status superseded by the measured checkpoint below.

## Focused verification

The contract's focused command set is:

```sh
cargo fmt --all -- --check
git diff --check
cargo test -p lila-aot-wasm --test reflect_property_key_conversion_structure -- --test-threads=1
cargo test -p lila-cli --test cli object::run_wasm_backend_preserves_reflect_property_key_conversion -- --exact --test-threads=1
./scripts/check-module-boundaries.sh
```

The structure target passes `4/4`. The exact CLI command in this block and
`cargo check -p lila-aot-wasm` have no individually attributed result here;
T11 owns the collective seven-CLI and shared-gate results. The payload-derived
tag used for static descriptor-field names is outside these five argument
boundaries. No broad compile, Test262 or published conformance result is
claimed.
