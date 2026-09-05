# For-await environment allocation and scope attachment

PR #14 is rebased on the captured-head implementation already merged in PR #12.
The runtime lifecycle remains the one documented in
[aot-captured-for-await.md](aot-captured-for-await.md): consuming saved/active
carriers retain one per-iteration Environment Record and one cleanup owner.

## Allocation and the compiler binding view

`emit_allocate_lexical_environment_record` emits allocation and initialization of
one record without changing compiler scopes, binding hops, or environment depth.
The ordinary `emit_enter_lexical_environment` composes allocation with
`begin_existing_lexical_environment_scope`, preserving its existing behavior.

The resumable for-await owner uses the two operations separately. A successful
value-resume arm allocates the fresh record; the body-resume arm restores the
saved record pointer. Once those runtime paths converge, the compiler attaches
one binding view and publishes the child to the owning activation. No resumed
path allocates another cell or initializes the loop head again.

The parent's iterator bookkeeping, activation layout projection, and consuming
cleanup carriers are retained from main. Cleanup records the attached child
depth, leaves exactly once, republishes the parent, and then dispatches the
existing completion. This also preserves asynchronous IteratorClose behavior.

## Regression coverage

The retained `aot_async_for_of` regression creates a closure before a body yield,
mutates the head after resumption, and checks both the yielded closure result
and the saved closures after two iterations. The required trace is
`1:false`, `11:false`, `2:false`, `12:false`, `captures:11:12`, `undefined:true`.
It requires `ExecutionBackend::WasmAot`, not interpreter fallback.

The activation-layout structure tests require allocation only in the fresh arm,
one binding-view attachment after the branch join, publication before cleanup,
and allocation that cannot silently alter the compiler's scope view. The broader
captured-head regressions from main remain in place, including abrupt completion,
interleaved activations, const heads, and asynchronous iterator closing.

```sh
cargo fmt --all -- --check
cargo test --locked -p lila-aot-wasm --test for_await_activation_layout_structure
cargo test --locked -p lila-engine --test aot_async_for_of --test aot_captured_for_await -- --test-threads=1
```

The existing README capability description remains current. Additional
materialized body environments, nested for-await, and the other unsupported
dispatcher shapes remain separate work. This refactor does not claim full
ECMAScript or pinned Test262 conformance; CI on the rebased head is the
verification record.
