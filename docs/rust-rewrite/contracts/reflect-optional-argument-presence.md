# Reflect optional-argument presence

Status: implemented and focused structure-verified 2026-09-01 as a bounded
T11 argument-presence correction.

## Scope

The builtin ABI argument count is the sole authority for distinguishing an
omitted argument from a present argument whose value is `undefined`. The
shared emitter computes `argc > index`; the existing builtin argument loader
delegates to it, and `Reflect.construct`, `Reflect.get` and `Reflect.set` are
the three optional-argument consumers.

`Reflect.construct` defaults `newTarget` to `target` only when argument index 2
is absent. A present `undefined` continues to `IsConstructor` and throws.
`Reflect.get` defaults receiver index 2, and `Reflect.set` defaults receiver
index 3, only when the corresponding index is absent. An explicitly supplied
`undefined` is preserved as the receiver.

## Observable order

`Reflect.get` and `Reflect.set` first validate the target, then complete
`ToPropertyKey`, and only then apply their absence-based receiver default. The
default cannot replace an explicitly supplied value. `Reflect.construct`
loads the optional argument, validates `target`, applies the absence default,
and then validates `newTarget`.

## Focused evidence

`wasm_reflect_optional_argument_presence.js` observes omitted and explicit
`undefined` receivers through Proxy get and set traps, distinguishes ordinary
Set mutation behavior, and proves that explicit `undefined` rejects before a
construct trap while omission passes the exact Proxy as `newTarget`.

`reflect_optional_argument_presence_structure` pins the one ABI authority,
three consumers, property-key/default order, active CLI fixture, module guard,
this contract and the T11 ledger entry.

The write-phase marker `Verification pending` is retained here only as the
historical status superseded by the measured checkpoint below.

## Focused verification

The contract's focused command set is:

```sh
cargo fmt --all -- --check
git diff --check
cargo test -p lila-aot-wasm --test reflect_optional_argument_presence_structure -- --test-threads=1
cargo test -p lila-cli --test cli object::run_wasm_backend_distinguishes_omitted_reflect_optional_arguments -- --exact --test-threads=1
./scripts/check-module-boundaries.sh
```

The structure target passes `5/5`. The exact CLI command in this block and
`cargo check -p lila-aot-wasm` have no individually attributed result here;
T11 owns the collective seven-CLI and shared-gate results. No broad compile,
Test262 or published conformance result is claimed.
