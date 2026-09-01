# Shape accessor reference selection

Status: normative for Lila's static heap-shape function reachability planner.

## Boundary

The reachability planner must retain accessor functions that an emitted property
operation can invoke through its statically inferred heap shape. Three and only
three selections are valid:

| Selection | Producers | Referenced accessor slots |
| --- | --- | --- |
| `Getter` | optional property chains and property reads | getter only |
| `Setter` | ordinary property assignments and property writes | setter only |
| `GetterOrSetter` | logical assignments, numeric updates and eager compound assignments | getter or setter |

The private `ShapeAccessorReferenceSelection` replaces two independent
Booleans, which previously admitted the meaningless neither-getter-nor-setter
state. It passes unchanged through recursive prototype-shape traversal. Static
property-key lookup and dynamic-key traversal each project it with a direct
exhaustive match. There is no Boolean projection, default, wildcard or
unreachable arm.

This selection exists only while Rust computes the standard-builtin reachability
plan. It adds no IR, heap or Wasm field and does not change property-key
evaluation, accessor invocation, prototype traversal, temporary-local ownership
or emitted instruction order.

The corresponding lowerer preserves accessor provenance from every shaped
branch of a conditional receiver even when the branches cannot be represented
by one merged `HeapShape`. It separately proves whether all branches are shaped;
only a genuinely unshaped branch enters the all-accessors fallback. Known
effect-free Map/Set size getters keep their Number result and cannot poison the
current function's parameter signature through a fabricated recursive hook.
The same immutable leaf set owns mutation-authority calculation and post-write
alias invalidation. Nested conditionals therefore invalidate every possible
receiver shape without treating a merged missing shape as authority over all
intrinsic prototypes. Alias reachability includes ordinary and array prototype
chains as well as properties, elements, and boxed primitive contents.
Ordinary object receivers conservatively retain inherited `__proto__` getter
and setter provenance. The closed
`BuiltinGetterReceiverProvenance::{ProvenNonProxy, MayBeProxy}` proof prevents
that getter from fabricating transitive user-code effects for a carried heap
shape: Proxy construction has no heap shape, and only the `MayBeProxy` route can
dispatch the `getPrototypeOf` trap. Unknown and reflective call boundaries erase
receiver shapes, while an unobserved sloppy-function `this` begins without one,
so aliases cannot manufacture a false non-Proxy proof.

The receiver-provenance proof has no clone, copy, debug or equality capability.
Both construction routes lend it to the shared builtin-getter classifier. The
`Object.prototype.__proto__` row projects both provenance variants through an
exhaustive borrowed match: `ProvenNonProxy` cannot call user code, while
`MayBeProxy` can. Adding another provenance state therefore requires an explicit
classification instead of inheriting the non-Proxy result from a `matches!`
predicate. The classifier's surrounding builtin rows and catch-all retain their
existing behavior.

## Durable witnesses

`shape_accessor_reference_selection_structure.rs` pins the exact three-variant
domain, both direct exhaustive projections, recursive typed forwarding and the
seven-producer mapping: two `Getter`, two `Setter` and three `GetterOrSetter`.

`builtin_getter_receiver_provenance_structure.rs` pins the private
no-capability two-variant proof, both shape-derived producer decisions, both
borrowed call routes, and the complete builtin classifier with its exhaustive
`__proto__` polarity. Independent review added the exact six-use local-name
census and source-wide bans on clone, equality, `matches!`, discriminant and
cast observers. The coordinated workspace checkpoint passes
`cargo fmt --all -- --check`, `cargo xc`, `git diff --check`, the
module-boundary check and the task-plan check.

`wasm_shape_accessor_reference_selection.js` joins `Map` and `Set` heap shapes.
It observes their `size` getters, assigns through the inherited `__proto__`
setter, and performs plain nested-conditional and logical writes. The fixture
also writes through a shared prototype. Both alternatives of every joined
shape must retain the required standard accessors, and every possible receiver
or inheriting descendant must lose stale shape facts after a write.

The `lila-ir` joined-conditional invariant pins both size getters directly in
the logical-assignment IR, before Wasm reachability planning. Four mutation
regressions pin nested conditional, logical, unrelated-intrinsic and
prototype-descendant invalidation independently.

## Focused verification

```sh
cargo test -p lila-aot-wasm --test shape_accessor_reference_selection_structure
cargo test -p lila-ir --test builtin_getter_receiver_provenance_structure
cargo test -p lila-aot-wasm planning::tests::dynamic_property_keys_root_every_possible_shape_accessor -- --exact
cargo test -p lila-aot-wasm planning::tests::joined_ -- --test-threads=1
cargo test -p lila-aot-wasm planning::tests::proven_non_proxy_proto_getter_does_not_root_unrelated_builtin_accessors -- --exact
cargo test -p lila-ir nested_conditional_property_write_invalidates_every_receiver_shape
cargo test -p lila-ir conditional_logical_write_invalidates_both_receiver_shapes
cargo test -p lila-ir conditional_ordinary_object_write_preserves_number_prototype_fact
cargo test -p lila-ir conditional_prototype_write_invalidates_both_inheriting_shapes
cargo test -p lila-cli --test cli object::run_wasm_backend_preserves_shape_accessor_reference_selection -- --exact --test-threads=1
rustfmt --edition 2021 --check crates/lila-ir/src/lowering.rs crates/lila-ir/src/lowering/ordinary_property_compound.rs crates/lila-ir/tests/builtin_getter_receiver_provenance_structure.rs
git diff --check
```

## Deferrals

This boundary does not change user accessor allocation, Proxy trap execution,
accessor call ABI, Realm selection or Test262 shortcuts. Broader Object
execution and conformance publication remain separate verification surfaces.
