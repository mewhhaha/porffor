# With-environment identifier-call Reference

## Status and evidence boundary

This contract owns the non-resumable direct-identifier call form whose
ResolveBinding walk can select a `with` Object Environment Record. At clean
commit `88de596ce22a69b8b7c47dacaed051172adf46b6`, the exact current-pin witness

`language/expressions/call/with-base-obj.js`

reports `0/1` under Wasm AOT as `Bug/Runtime`: its `via CallExpression`
SameValue assertion observes that `method()` did not receive the selected
binding object as `this`. The file has `flags: [noStrict]`, so it is one
physical file and one execution, not a two-mode denominator.

The failure is current-source-proven independently of that measurement.
Identifier GetValue already walks the analyzed Object Environment chain, but
the generic identifier-call lowering subsequently emits `CallIndirect` with
`this_arg: None`. ECMA-262 CallExpression evaluation instead applies
`WithBaseObject()` when the Reference base is an Environment Record selected
from a `with` environment.

This is focused current evidence, not a claim that the complete call or `with`
subtree, or the pinned matrix, is green.

## Normative lifecycle

For a direct call `name(arguments)` in the supported source domain:

1. Evaluate the callee IdentifierReference and create its Reference before
   evaluating any argument.
2. Resolve the identifier through the analyzed, inner-to-outer environment
   order. Each candidate `with` Object Environment Record performs
   `HasBinding(name)`: `HasProperty(bindingObject, name)`, then the observable
   `@@unscopables` lookup and block test.
3. If a candidate is selected, perform that same record's GetBindingValue:
   independently re-run `HasProperty(bindingObject, name)`, then `Get` the
   callee. An abrupt selection or GetValue completion precedes every argument.
4. Retain the exact selected `bindingObject` as the call Reference's
   `WithBaseObject()` result. The callee and receiver are one product; neither
   may be recomputed or paired with a different Object Environment candidate.
5. Evaluate arguments from left to right only after normal callee GetValue.
   Invoke the callee with the retained binding object as `this`.
6. If every `with` candidate declines the name, continue with the already
   located declarative/global/unresolvable fallback Reference. Its ordinary
   identifier call retains the existing undefined-this path and runtime Call
   behavior. HasBinding and `@@unscopables` are observable and may replace a
   previously known fallback callable before declining the name, so this
   conditional fallback is always a fresh Dynamic read with no static function
   targets or direct-builtin fold.

The runtime selection is expressed as mutually exclusive IR branches. Cloned
argument IR in those branches is evaluated once at runtime because only the
selected branch executes; lowering the source argument list itself remains a
single compiler action.

## Closed Rust seam

`WithEnvironmentIdentifierCallReferencePlan` is a private, non-`Clone`,
non-`Copy`, `#[must_use]` capability. It owns the existing non-empty
`WithEnvironmentReferencePlan`; its only public(crate) transition is
`call(args, fallback) -> TypedExpr`.

`SelectedWithEnvironmentObjects::into_identifier_call_plan` is the sole
constructor. It retains the already-materialized binding-object identities and
allocates the unscopables temporaries in the same inner-to-outer order as other
with References.

The plan's consuming call builds each selected branch from one
`ObjectEnvironmentBindingObject`: that value produces both GetBindingValue's
callee and `WithBaseObject`'s receiver. Callers cannot pass either role as a
positional `TypedExpr`, so swapping the receiver, dropping it, or attaching an
undefined receiver to a selected callee is unavailable at the API boundary.
The final fallback is a complete ordinary call expression, which structurally
keeps it outside the selected-object receiver path. A prelocated declarative
fallback reads its carried storage; a global or unresolvable fallback uses the
runtime global IdentifierReference read. Neither re-runs compiler lookup after
the observable Object Environment selection.

The lowerer must locate the fallback Reference and create this plan before
lowering arguments. It must not first lower a value-only identifier and try to
recover the Reference base afterward.

## Scope and nonclaims

The product scope is direct, non-optional IdentifierReference calls in scripts
and ordinary non-resumable functions. The selected callee may be any runtime
callable value, including a callable Proxy; runtime Call remains authoritative.

This batch does not claim optional calls, property/private/super calls, direct
or indirect `eval`, dynamic source generation, constructors, tagged templates,
or resumable async/generator bodies with captured `with` environments. Those
forms retain their existing dedicated Reference and capability boundaries.

## Verification ladder

Cheap implementation checks:

```sh
cargo fmt --all -- --check
git diff --check
./scripts/check-module-boundaries.sh
```

Central focused verification after batch integration:

```sh
cargo test -p lila-ir with_environment_identifier_call
cargo test -p lila-aot-wasm --test with_environment_identifier_call_structure
cargo test -p lila-cli --test cli with_environment_identifier_call_fixture
./target/debug/lila test262 run language/expressions/call/with-base-obj.js \
  --suite-root test262/vendor/test262 --execution-backend wasm-aot \
  --timeout-ms 180000 --threads 1
```

Only after those focused gates should a broader `language/expressions/call`
or pinned-matrix refresh be interpreted.
