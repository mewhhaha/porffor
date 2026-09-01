# Promise combinator mode domains

Status: implemented and focused-verified for the five standard and keyed
Promise combinator compiler entries, 2026-08-26.

## Closed domains

Standard iterable combinators use `PromiseCombinatorMode`, whose exact
inhabitants are `Values`, `SettledRecords` and `FirstFulfillment`. Keyed
combinators use the narrower `PromiseKeyedCombinatorMode`, whose exact
inhabitants are `Values` and `SettledRecords`.

Neither domain implements `PartialEq` or `Eq`. The keyed lowering cannot accept
`FirstFulfillment`, and neither lowering may collapse mode policy to equality,
inequality or a Boolean default.

The three standard wrappers and two keyed wrappers are the only producers. The
keyed lowering directly and exhaustively projects its restricted mode for its
diagnostic name, resolve-element callback and optional reject-element callback.
The standard family has six exhaustive review points: its diagnostic name plus
five body decisions for terminal capability, resolve-element callback,
reject-element callback, one paired per-element reaction route, and final Array
resolution versus AggregateError rejection. Four body projections remain in
the parent; the paired route and its non-copyable
`PromiseCombinatorReactionPairLocals` live in the private
`promise_combinator_reaction_pair.rs` owner, so fulfillment and rejection
callbacks cannot drift through independent mode matches or parent-side raw
pair construction.

This prevents an illegal keyed-any state from combining all-settled records
with direct keyed rejection, and forces every future mode to name all of its
settlement policy before the compiler builds.

## Private keyed-mode boundary

`builtins/promise/promise_keyed_combinator_mode.rs` is the sole owner of
`PromiseKeyedCombinatorMode`, both keyed semantic producers and the raw keyed
lowerer. The Promise parent and standard-builtin dispatcher cannot name,
import, re-export or construct the keyed mode; the dispatcher can only call the
unchanged `pub(crate)` `allKeyed` and `allSettledKeyed` wrappers. The raw
`emit_promise_keyed` consumer remains child-private.

The pre-extraction five-line keyed domain and 632-line producer/consumer
lifecycle retain SHA-256
`489be0d316aa862e31ef48f6b526e40233f171f2066dd47abb6c8b382d6459ba`
and
`f38a4742ce2a901162f25f68bb1cb6cf3e353862c4212805d4b5b92d569cf363`.
Their combined 637 selected lines retain SHA-256
`aaf6deb2d7557380c453c06da4a2e5b2a22f42aa00f9aed307e9ed75fed37f5f`.
The formatted 641-line child has SHA-256
`3b36cbf4fb06350f3dfd551c31a180b6ac523101f30aceac4d614082097deda7`
and reduces the concurrent Promise parent from 8,245 to 7,608 lines.

## Focused evidence

`promise_combinator_mode_structure.rs` pins both declarations, the exact
three-plus-two producer census, the restricted keyed signature, three keyed
and six standard exhaustive policy projections, and the absence of equality
escape hatches. The paired projection is independently pinned by
`promise_combinator_reaction_pair_ownership_structure.rs` and
[`promise-combinator-reaction-pair-ownership.md`](promise-combinator-reaction-pair-ownership.md).
After the private keyed-mode extraction, the recursive guard additionally pins
the private module, zero imports/re-exports, ten child-only keyed-mode mentions,
both unchanged producers and the one raw consumer. Its include-only target
passes `3/3`. The keyed all-settled engine witness and all-five-mode CLI
witness each pass `1/1`, and the shared `cargo xc`, formatting, diff,
module-boundary and task-plan checks are green. No post-extraction semantic
golden or broad conformance suite was run for this source-equivalent move.
The adjacent reaction-pair guard separately pins its private owner and the
four-parent-plus-one-child standard body projection census.

`wasm_promise_combinator_modes.js` observes ordered values and direct rejection
for `Promise.all`, fulfilled and rejected records for `Promise.allSettled`, both
terminal paths of `Promise.any`, raw null-prototype keyed values and direct
rejection for `Promise.allKeyed`, and null-prototype keyed settlement records
for `Promise.allSettledKeyed`.

```sh
cargo test -p lila-aot-wasm --test promise_combinator_mode_structure --quiet
cargo test -p lila-cli --test cli functions::run_wasm_backend_distinguishes_all_promise_combinator_modes -- --exact --test-threads=1
```

The shared semantic golden passes `2/2` in 722.99 seconds with 678 dumps. It
adds this witness plus the independent Array.fromAsync callback-Realm, Object
policy and Set-domain witnesses, removes none and leaves all 674 retained dumps
equal after accounting normalization. Broad Test262 verification remains
deferred. This boundary changes no iterator protocol, observable lookup order,
callback metadata, Promise capability layout, Realm authority or published
conformance count. It does not complete T14.
