# Object Environment logical assignment retains one Reference

## Scope and exact evidence

This contract covers identifier `&&=`, `||=` and `??=` when ResolveBinding can
reach an Object Environment Record. The exact Test262 cohort is the three
initially unresolvable strict-global cases:

- `language/expressions/logical-assignment/lgcl-and-assignment-operator-unresolved-lhs.js`;
- `language/expressions/logical-assignment/lgcl-or-assignment-operator-unresolved-lhs.js`;
- `language/expressions/logical-assignment/lgcl-nullish-assignment-operator-unresolved-lhs.js`.

All three files measured `0/1` as Runtime/NotImplemented with `unbound
identifier \`unresolved\`` against pinned Test262 `aa55200` using the current
available debug binary. That binary predates source HEAD `528e3700` by one
focused batch; current source independently contains the same shared rejection
for all three operators.

No vendored Test262 logical-assignment file contains a `with` statement. A
focused fixture is therefore the evidence for with selection: with outer
`x = 10` and selected `{ x: 0 }`, `with (scope) { x ||= 5 }` must update
`scope.x` to `5` and retain outer `x`; the current available binary instead
leaves `scope.x` unchanged. With behavior is not added to the exact Test262
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
2. For a global Object Environment Record candidate, perform HasBinding as one
   plain HasProperty on the compiler-owned global object. The global record is
   not a with environment and never observes `Symbol.unscopables`. An initial
   miss produces an unresolvable Reference whose GetValue throws ReferenceError
   before the RHS.
3. For each with Object Environment Record, perform HasProperty and then the
   `Symbol.unscopables` visibility check. The first visible binding fixes the
   Reference base. A miss continues to the next selected record or to the
   already-located declarative/global fallback.
4. GetValue on the retained Object Record independently performs HasProperty
   and then Get. A false recheck throws ReferenceError for a strict Reference
   and yields `undefined` for a sloppy Reference.
5. Use that GetValue result as the lhs of the selected `LogicalShortCircuit`:
   `&&=` takes the RHS branch only for truthy lhs, `||=` only for falsy lhs and
   `??=` only for nullish lhs.
6. A short-circuit returns the old value without evaluating the RHS and without
   PutValue.
7. Only the taken branch evaluates the RHS, then calls PutValue on the same
   retained Reference. Object Record SetMutableBinding independently performs
   HasProperty after the RHS. A false recheck throws ReferenceError for strict
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

`ObjectEnvironmentBindingObject::logical_assignment` is the private shared
lifecycle. It consumes one binding-object identity, borrows the referenced
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
retains the same materialized/global object identity for GetValue and PutValue.

`WithEnvironmentReferencePlan::logical_assignment` consumes the non-empty,
non-Clone/non-Copy selection chain and wraps the shared lifecycle in each
HasProperty/unscopables selection condition, with the already-lowered
pre-located fallback as the final branch.

`GlobalObjectEnvironmentReferencePlan::logical_assignment` consumes its
compiler-owned global object, name and strictness. It wraps the shared lifecycle
in the initial plain HasProperty condition and emits RuntimeThrow ReferenceError
on a miss. There is no caller-built global object expression or unscopables
state in this type.

The identifier logical arm locates its declarative/global fallback before
lowering the RHS. Direct unresolvable, unproven names use the global plan;
selected with chains use the with plan; existing declarative and proven-global
paths retain their representation but place their write only in the
short-circuit RHS.

Observable Object Environment selection can mutate any fallback before either
selecting the object or reaching it. With-conditional fallback metadata is
therefore Dynamic/all runtime tags, and conditional global facts are not marked
proven present. A global initial-miss throw can be caught while the property
remains absent.

No new backend expression is introduced. Re-resolving after the RHS,
evaluating RHS outside the short-circuit branch, putting on a short-circuit,
selecting a different Object Record for the write, global unscopables lookup,
specializing from pre-observation metadata, and a reusable plan are outside the
producer API.

## Verification

After batch integration:

```sh
cargo fmt --all --check
cargo test -p lila-ir object_environment_logical_assignment --quiet
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

The durable fixture covers all three operators in taken and short-circuit
forms, initially missing global ReferenceError before RHS, strict getter
deletion before PutValue, sloppy recreation, with selection over a declarative
fallback, unscopables fallback, same-object identity and no PutValue on the
short-circuit path. The adjacent unresolved-RHS logical tests and retained plain,
eager and numeric Object Environment fixtures are focused regression controls.
Broad language and pinned-matrix publication remain the central verification
checkpoint after this focused ladder.
