# Spec-operation object-target kind

Six object-only specification operations share one closed static target
classification: `Get`, `HasProperty`, `HasOwnProperty`,
`DeletePropertyOrThrow`, `Set` and `CreateDataPropertyOrThrow`.

`SpecOperationObjectTargetKind` has exactly three states:

- `StaticallyObjectLike` skips a redundant tag check for Object, Array,
  Arguments and Function;
- `RuntimeDynamic` emits and later closes the existing runtime object-like tag
  branch; and
- `StaticallyPrimitive` enters the operation's existing TypeError path.

One exhaustive `ValueKind` projection owns that decision. Each operation keeps
its distinct error text, local release and completion routing, but no longer
maintains a wildcard primitive complement that could silently misclassify a new
heap kind. The authority derives no cloning, copying, formatting or equality
capability; every operation borrows it for the primary branch and consumes it
when closing a Dynamic branch.

This source-equivalent migration changes no evaluation, conversion, object
operation, error or completion order.

```sh
cargo test -p lila-aot-wasm --test spec_operation_object_target_kind_structure
cargo test -p lila-cli --test cli language_numerics::run_wasm_backend_succeeds_for_spec_has_property_order_fixture -- --exact --test-threads=1
```

The recursive structure target passes `4/4`, the exact HasProperty ordering
CLI witness passes `1/1`, and the shared `cargo xc`, formatting, diff,
module-boundary and task-plan checks are green. No broader conformance suite
was run for this source-equivalent invariant closure.
