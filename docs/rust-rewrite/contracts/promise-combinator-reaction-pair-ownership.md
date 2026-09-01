# Promise combinator reaction-pair ownership

Status: implemented for standard iterable Promise combinators.

## Boundary

The private `builtins/promise/promise_combinator_reaction_pair.rs` owner
contains `PromiseCombinatorReactionPairLocals`, the one-shot authority for the
two callbacks passed to each per-element `then` invocation in `Promise.all`,
`Promise.allSettled` and `Promise.any`. Each role is a `TaggedLocals`, so its
payload and tag cannot be transposed. One exhaustive `PromiseCombinatorMode`
match selects both roles together:

- `Values` uses its resolve-element callback and the capability reject;
- `SettledRecords` uses its resolve-element and reject-element callbacks;
- `FirstFulfillment` uses the capability resolve and its reject-element callback.

The carrier derives no cloning or copying capability. Its raw construction,
projection and consuming `then` call are child-private. The Promise parent can
invoke only the named semantic method and cannot name, import, re-export,
construct or project the pair. The owner destructures the complete pair by
value and preserves the observable fulfillment-before-rejection argument
order. The `then` call cannot reconstruct either role from the mode, and adding
a combinator mode requires selecting both callbacks in one compiler-enforced
arm.

## Durable evidence

`crates/lila-aot-wasm/tests/promise_combinator_reaction_pair_ownership_structure.rs`
recursively pins the private module, zero imports or re-exports, private
non-derived declaration, five child-only production mentions, sole semantic
entry and parent caller, exact three-row selection, paired tagged locals,
consuming projection, argument order and absence of the four former loose
callback-local names at the invocation. The neighboring
`promise_combinator_mode_structure` guard pins four parent-owned body
projections plus this one child-owned paired route.

The source-equivalent owner move selects the exact five-line carrier at
SHA-256
`ebdea093363ba0f80b64d7787525bf050b47b85c2f2146b1b6e4306da98b8585`
and the exact 43-line selection, projection and invocation block at SHA-256
`d6f594bb8c009eafbfe9414be7b3342967920529e8a71c1bd500c246168440e3`.
Both hashes are unchanged after relocation. The resulting 74-line child has
SHA-256
`0566a6451be281d6da341e602695a683986e2a45d5e31493813bbe8ae795ae0e`
and reduces the concurrent Promise parent from 7,591 to 7,561 lines. The two
focused structure targets pass `6/6`, the exact all-mode runtime witness passes
`1/1`, and the shared `cargo xc`, formatting, diff, module-boundary and
task-plan checks are green. Semantic goldens were not rerun for this
source-equivalent owner move.

The retained combinator fixture exercises fulfillment and rejection for all
three standard modes, as well as the separate keyed modes:

```sh
cargo test -p lila-aot-wasm --test promise_combinator_reaction_pair_ownership_structure -- --test-threads=1
cargo test -p lila-aot-wasm --test promise_combinator_mode_structure -- --test-threads=1
cargo test -p lila-cli --test cli functions::run_wasm_backend_distinguishes_all_promise_combinator_modes -- --exact --test-threads=1
cargo check -p lila-aot-wasm --lib
```

## Nonclaims

This is source-equivalent Rust ownership hardening. It changes no callback,
lookup order, Promise capability, reaction record, job scheduling, emitted
Wasm or JavaScript behavior. Keyed combinators retain their separate closed
mode and callback-routing path. This does not close T14's broader Promise,
async-function or job-queue conformance work.
