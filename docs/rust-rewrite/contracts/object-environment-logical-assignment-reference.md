# Object Environment logical assignment retains one Reference

Current source uses one retained Global Environment Reference for every global
logical assignment, including previously proven properties and Script `var`
names. Getter/Has effects precede RHS lowering, and PutValue refreshes the same
Global Record's delegate. The new IR/native controls are authored; their joined
verification is pending. The original focused evidence below is historical.

## Scope and exact evidence

This contract covers identifier `&&=`, `||=` and `??=` when ResolveBinding can
reach an Object Environment Record. The exact Test262 cohort is the three
initially unresolvable strict-global cases:

- `language/expressions/logical-assignment/lgcl-and-assignment-operator-unresolved-lhs.js`;
- `language/expressions/logical-assignment/lgcl-or-assignment-operator-unresolved-lhs.js`;
- `language/expressions/logical-assignment/lgcl-nullish-assignment-operator-unresolved-lhs.js`.

All three files originally measured `0/1` as Runtime/NotImplemented with `unbound
identifier \`unresolved\`` against pinned Test262 `aa55200` using the then-available
debug binary. That binary predated source HEAD `528e3700` by one focused batch;
that predecessor source contained the same shared rejection for all three
operators.

No vendored Test262 logical-assignment file contains a `with` statement. A
focused fixture is therefore the evidence for with selection: with outer
`x = 10` and selected `{ x: 0 }`, `with (scope) { x ||= 5 }` must update
`scope.x` to `5` and retain outer `x`; that predecessor binary instead
left `scope.x` unchanged. With behavior is not added to the exact Test262
count.

Property and private References are separate lowering domains. Plain
assignment, eager arithmetic/bitwise compound assignment, numeric update,
resumable functions, modules and dynamic source generation are not claims of
this batch.

## Normative lifecycle

For an in-scope identifier logical assignment:

1. Evaluate the LeftHandSideExpression and perform ResolveBinding before
   lowering or evaluating the RHS. Retain that exact Reference and its
   strictness.
2. For a global candidate, start ResolveBinding at the source Realm's Global
   Environment. Its declarative record takes precedence over the object record;
   the latter performs plain HasProperty on the compiler-owned global object.
   It never observes `Symbol.unscopables`. An initial miss produces an
   unresolvable Reference whose GetValue throws ReferenceError before the RHS.
3. For each with Object Environment Record, perform HasProperty and then the
   `Symbol.unscopables` visibility check. The first visible binding fixes the
   Reference base. A miss continues to the next selected record or to the
   already-located declarative/global fallback.
4. GetValue refreshes the selected Global Record's declarative/object delegate.
   An Object Record independently performs HasProperty and then Get. A false
   recheck throws ReferenceError for a strict Reference and yields `undefined`
   for a sloppy Reference.
5. Use that GetValue result to choose the logical branch:
   `&&=` takes the RHS branch only for truthy lhs, `||=` only for falsy lhs and
   `??=` only for nullish lhs.
6. A short-circuit returns the old value without evaluating the RHS and without
   PutValue.
7. Only the taken branch evaluates the RHS, then calls PutValue on the same
   retained Reference. The Global Record's live delegate is refreshed; Object
   Record SetMutableBinding independently performs HasProperty after the RHS. A false recheck throws ReferenceError for strict
   code; sloppy code still observes the recheck and performs Set.
8. Only after PutValue succeeds does the taken branch return the RHS value.

The initial Object Record selection, GetBindingValue recheck and
SetMutableBinding recheck are distinct observable operations. An assignment
wrapped around a completed `LogicalShortCircuit` is invalid because it performs
PutValue even on the short-circuit branch.

## Rust invariant and IR composition

The existing closed `LogicalBinaryOp::{And, Or, Coalesce}` is the operation
domain. Lowering maps the three Boa `AssignOp` variants exhaustively without a
catch-all in the relevant mapper.

`ObjectEnvironmentBindingObject::logical_assignment` is the private lifecycle
for a selected `with` object. It consumes one binding-object identity, borrows the referenced
name, receives `Strictness`, the closed operation and the lowered RHS, and
composes existing IR only:

```text
LogicalShortCircuit {
  lhs: binding_object.get_value(name, strictness),
  rhs: binding_object.put_value(name, strictness, rhs),
}
```

Because `put_value` owns the RHS expression inside the short-circuit branch,
RHS evaluation, the independent write recheck and Set cannot occur on the
short-circuit path. The cloned binding-object value is compiler-private and
retains the same materialized object identity for GetValue and PutValue.

`WithEnvironmentReferencePlan::logical_assignment` consumes the non-empty,
non-Clone/non-Copy selection chain and wraps the shared lifecycle in each
HasProperty/unscopables selection condition, with the already-lowered
pre-located fallback as the final branch.

Global logical assignment uses `EnvironmentIdentifierIr::global`, which fixes
`EnvironmentIdentifierResolutionStart::GlobalEnvironment` in its constructor.
The existing `LogicalCompound` operation owns the RHS. The emitter resolves
once before GetValue, retains that Reference, and performs PutValue only inside
the taken RHS branch. A getter deleting an own property cannot cause a second
ResolveBinding before the RHS, even when the new prototype has a Proxy `has`
trap. Get/Put refresh only the chosen Global Record's delegate; they do not
restart lexical-chain or `with` selection.

The identifier logical arm locates its declarative/global fallback and snapshots
possible global callable targets before Has/Get effect invalidation and RHS
lowering. With chains retain their selected-object lifecycle; actual local and
captured bindings retain their own storage. All global fallbacks use the same
retained runtime Reference, irrespective of declaration/value metadata. Returned
callable candidates are widened to Open, and no pre-Get tag or shape becomes a
proof about a getter result.

Observable Object Environment selection can mutate any fallback before either
selecting the object or reaching it. With-conditional fallback metadata is
therefore Dynamic/all runtime tags, and conditional global facts are not marked
proven present. A global initial-miss throw can be caught while the property
remains absent.

No new backend expression is introduced. Re-resolving after GetValue,
evaluating RHS outside the short-circuit branch, putting on a short-circuit,
selecting a different Object Record for the write, global unscopables lookup,
specializing from pre-observation metadata, and a reusable plan are outside the
producer API.

## Verification

After batch integration:

```sh
cargo fmt --all --check
cargo test -p lila-ir lowering::object_environment_logical::tests --quiet
cargo test -p lila-aot-wasm \
  --test object_environment_logical_assignment_structure --quiet
cargo test -p lila-cli --test cli \
  language::run_wasm_backend_succeeds_for_object_environment_logical_assignment_fixture \
  -- --exact --test-threads=1
for case_file in \
  language/expressions/logical-assignment/lgcl-and-assignment-operator-unresolved-lhs.js \
  language/expressions/logical-assignment/lgcl-or-assignment-operator-unresolved-lhs.js \
  language/expressions/logical-assignment/lgcl-nullish-assignment-operator-unresolved-lhs.js
do
  ./target/debug/lila --jobs 1 test262 run "$case_file" \
    --suite-root test262/vendor/test262 --execution-backend wasm-aot \
    --timeout-ms 180000 --threads 1
done
```

The existing native `aot_ordinary_global_assignment_reference` cohort
`global_prototype_hasbinding_precedes_rhs_and_plain_assignment_does_not_get`
also covers all three logical operators after a getter deletes the own property,
exact inherited `has` order, skipped Put, abrupt Get and abrupt RHS in both modes.

The durable fixture covers all three operators in taken and short-circuit
forms, initially missing global ReferenceError before RHS, strict getter
deletion before PutValue, sloppy recreation, with selection over a declarative
fallback, unscopables fallback, same-object identity and no PutValue on the
short-circuit path. The adjacent unresolved-RHS logical tests and retained plain,
eager and numeric Object Environment fixtures are focused regression controls.
Broad language and pinned-matrix publication remain the central verification
checkpoint after this focused ladder.
