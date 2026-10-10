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

## `in` operand order — 2026-10-10

The complete CLI checkpoint exposes a separate lowering defect: canonical
HasProperty operands are `(object, key)`, so placing the source `in` expressions
directly in those slots evaluates the RHS first. Lowering now materializes the
LHS key value once before emitting HasProperty with its unchanged operand
layout. Both expressions complete before object validation, which still precedes
ToPropertyKey; abrupt key evaluation prevents RHS evaluation.

Strict/sloppy native controls cover GetValue order, RHS mutation of the source
key binding, late property-key coercion, Proxy `has` order, abrupt operands,
object-validation precedence and suspended generator operands. Reflect.has
retains its ordinary target-first argument order. The IR control fixes the
materialization boundary and canonical object/key slots. These new controls
remain pending until the recorded focused checkpoint completes; the historical
results below do not certify this repair.

```sh
cargo test -p lila-aot-wasm --test structure_language -- spec_operation_object_target_kind_structure::
cargo test -p lila-cli --test cli language_numerics::run_wasm_backend_succeeds_for_spec_has_property_order_fixture -- --exact --test-threads=1
```

The recursive structure target passes `4/4`, the exact HasProperty ordering
CLI witness passes `1/1`, and the shared `cargo xc`, formatting, diff,
module-boundary and task-plan checks are green. No broader conformance suite
was run for this source-equivalent invariant closure.
