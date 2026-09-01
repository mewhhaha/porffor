# Promise reaction initialization policy

## Closed policy

`PromiseReactionInitialization` is the private two-variant choice that couples
a reaction's callback kind with its Realm authority:

- `Default` stores the null Realm sentinel, selects the default callback and
  resolves through the executing Promise operation's Realm;
- `AsyncExecution` stores the activation-owned Realm, selects its typed async
  continuation callback and resolves through that same captured Realm.

The policy is deliberately non-`Clone` and non-`Copy`. Intrinsic Await owns one
choice and borrows it for the PromiseResolve authority, fulfill reaction and
reject reaction. The direct reaction initializer also borrows the choice. A
by-value projection that accidentally moves the choice now fails to compile
before the remaining projections, and both consumers must project every variant
exhaustively.

## Producers and consumers

Exactly four named construction sites own the domain: default and async direct
reaction wrappers, plus default and async intrinsic-Await wrappers. The shared
reaction initializer exhaustively selects the stored Realm and callback kind.
The intrinsic-Await emitter separately selects PromiseResolve Realm authority,
then passes the same borrowed policy to the fulfill initializer before the
reject initializer.

`AsyncAwaitContinuation` is the separate private five-way authority selecting
the callback word and the async-function or async-generator activation Realm.
It also derives no cloning or copying capability. The shared Await emitter owns
one continuation, borrows it to select the activation Realm, then moves it once
into `PromiseReactionInitialization`. Its callback-word projection borrows all
five variants, so it cannot consume the continuation before reaction
initialization finishes. The separate async-generator AwaitReturn path
constructs one explicit continuation for each of its fulfill and reject
reactions in that order.

`PromiseReactionType` and the encoded reaction callback word remain distinct
domains. This closure changes no heap word, emitted Wasm local, Wasm instruction
or evaluation order. Emitted Wasm is expected to remain byte-identical; no
semantic golden was run for this source-equivalent ownership migration.

## Durable evidence

The Rust-lexical bounded structure target recursively fixes the eleven reaction
initialization mentions and seventeen continuation mentions, including the
exact `2/2/3/2/2` variant-route census. It pins both attribute-free declarations,
manual-capability closure, four initialization producers, all six continuation
producers, both exhaustive policy projections, the five-row borrowed callback
projection, owned-to-borrowed-to-moved Await flow, fulfill-before-reject order
and reverse local release. The existing async execution Realm fixture crosses
ordinary default Promise reactions plus async-function and async-generator
Await reactions in created-Realm calls.

```sh
cargo test -p lila-aot-wasm --test promise_reaction_initialization_structure --quiet
cargo test -p lila-aot-wasm --test promise_resolve_realm_context_structure --quiet
cargo test -p lila-aot-wasm --test async_execution_realm_structure --quiet
cargo test -p lila-cli --test cli functions::run_wasm_backend_uses_async_function_realms_for_promises_and_reactions -- --exact --test-threads=1
```

The dedicated and two neighboring structure targets pass `13/13`, and the
exact created-Realm CLI witness passes `1/1`. Independent review confirmed the
private capability/mention closure, four producer mappings, both exhaustive
projections, shared-policy borrowing, fulfill-before-reject order and source
equivalence. The reaction-initialization and continuation ownership changes now
share a green coordinated checkpoint: `cargo fmt --all -- --check`, `cargo xc`,
`git diff --check`, the module-boundary check and the task-plan check.

This is an internal ownership invariant, not a new Promise behavior or a claim
of broader Promise/async Test262 closure.
