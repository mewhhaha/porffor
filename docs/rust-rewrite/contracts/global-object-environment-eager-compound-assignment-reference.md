# Global Object Environment eager compound assignment retains one Reference

## Scope and exact cohort

This contract covers eager compound assignments whose LeftHandSideExpression
is an IdentifierReference, whose lexical ResolveBinding walk finds no
declarative binding, and whose Global Environment Record selects a property
through its Object Environment Record.

The exact Test262 cohort is the eleven `noStrict` files below. Each enters a
strict nested function, obtains `x` from an accessor installed directly on the
global object, deletes that property during GetValue, and requires the later
strict SetMutableBinding to throw ReferenceError:

- `compound-assignment-operator-calls-putvalue-lref--v--1.js` (`^=`);
- `compound-assignment-operator-calls-putvalue-lref--v--3.js` (`|=`);
- `compound-assignment-operator-calls-putvalue-lref--v--5.js` (`*=`);
- `compound-assignment-operator-calls-putvalue-lref--v--7.js` (`/=`);
- `compound-assignment-operator-calls-putvalue-lref--v--9.js` (`%=`);
- `compound-assignment-operator-calls-putvalue-lref--v--11.js` (`+=`);
- `compound-assignment-operator-calls-putvalue-lref--v--13.js` (`-=`);
- `compound-assignment-operator-calls-putvalue-lref--v--15.js` (`<<=`);
- `compound-assignment-operator-calls-putvalue-lref--v--17.js` (`>>=`);
- `compound-assignment-operator-calls-putvalue-lref--v--19.js` (`>>>=`);
- `compound-assignment-operator-calls-putvalue-lref--v--21.js` (`&=`).

All paths are relative to
`test262/vendor/test262/test/language/expressions/compound-assignment/`.
The adjacent bare suffix and even-numbered files use an Object Environment
Record introduced by `with`; they belong to the separate with-environment
contract. A Global Environment Record's Object Record has
`[[IsWithEnvironment]] = false`, so this lane never reads
`Symbol.unscopables`.

The producer's closed eager domain also includes `**=` because it shares the
same Reference lifecycle. It is local invariant coverage, not a twelfth
Test262 claim. Logical assignments, property References, declarative bindings,
resumable functions, modules, and dynamic source generation are not claims of
this batch. Existing proven declarative/global specializations remain outside
this new dynamic path.

## Normative lifecycle

For an in-scope `x op= rhs`:

1. ResolveBinding reaches the Global Environment Record after finding no
   declarative binding. The Object Record performs HasBinding as one
   HasProperty operation on the compiler-owned global object. This happens
   before GetValue and before evaluating `rhs`.
2. If that HasProperty is false, the Reference is unresolvable and GetValue
   throws ReferenceError before evaluating `rhs`, in sloppy or strict code.
3. If it is true, the resulting Reference retains that exact Object
   Environment Record as `[[Base]]` and retains the strictness of the source
   which created it.
4. GetValue calls GetBindingValue on the same Object Record. GetBindingValue
   independently performs HasProperty and then Get. A false recheck throws
   ReferenceError for a strict Reference and yields `undefined` for a sloppy
   Reference.
5. Evaluate `rhs` exactly once after GetValue, then apply the selected closed
   arithmetic or bitwise operation in ECMAScript coercion order.
6. PutValue calls SetMutableBinding on the same Object Record selected in step
   1. SetMutableBinding independently performs HasProperty after the getter,
   RHS, and coercions. Resolution does not restart.
7. If the write recheck is false and the retained Reference is strict, throw
   ReferenceError without calling Set. In sloppy code the recheck remains
   observable, then Set runs even when it answered false.
8. Only after PutValue succeeds does the expression return the applied value.

The three HasProperty operations are distinct observable specification
operations: initial ResolveBinding, GetBindingValue, and SetMutableBinding.
The exact witnesses delete `x` in the intervening Get, so replacing the
lifecycle with a raw global read plus checked write is not the contract.

## Rust invariant and IR composition

`ObjectEnvironmentBindingObject` is the only object identity admitted by the
shared Object Environment operations. Its constructors distinguish a
materialized `with` object from the compiler-owned global object; callers
cannot provide an arbitrary `TypedExpr`. HasProperty, GetBindingValue, and
SetMutableBinding therefore read clones of one validated identity.

`WithEnvironmentResolution` alone owns `Symbol.unscopables` selection.
`GlobalObjectEnvironmentReferencePlan` owns exactly one global binding object,
referenced name, and typed `Strictness`. The plan is neither `Clone` nor
`Copy`, carries `#[must_use]`, and has one consuming eager compound-assignment
operation. Consequently a caller cannot accidentally run with-environment
selection for a global record, omit unscopables from a with record, or apply
GetValue and PutValue to different bases.

One opaque `EagerCompoundAssignmentBindings` allocator fixes old-value,
result, and write-completion binding roles. Consuming it produces the only
`EagerCompoundAssignment` accepted by either Reference plan. The lowerer maps
the eager arithmetic and bitwise domains into `EagerCompoundAssignmentOp`
exhaustively, obtains the old operand only from the role carrier, applies the
canonical existing coercive-add/coercive-number/bitwise IR, and seals the
result.

After its initial plain HasProperty condition, the global plan composes only
existing IR:

1. materialize `oldValue = binding_object.get_value(...)`;
2. materialize `result = apply(oldValue, rhs)`;
3. materialize `write = binding_object.put_value(result, ...)`;
4. only in the write materialization's body, read `result`.

No new backend expression or parallel arithmetic implementation is admitted.
An initially missing property selects a branch-local RuntimeThrow before the
old-value materialization and therefore before the cloned RHS. Unknown global
property metadata remains fully Dynamic and is never marked proven present.

Thus a caller-built global object expression, a missing initial HasProperty,
an unscopables lookup on the global path, positional same-typed binding roles,
a second plan consumption, a restarted resolution, a result before successful
PutValue, and a raw GlobalPropertyCompoundAssign shortcut are outside the
producer API.

## Verification

The focused ladder after batch integration is:

```sh
cargo fmt --all --check
cargo test -p lila-ir global_object_environment_compound_assignment --quiet
cargo test -p lila-aot-wasm \
  --test global_object_environment_compound_assignment_structure --quiet
cargo test -p lila-cli --test cli \
  language::run_wasm_backend_succeeds_for_global_object_environment_compound_assignment_fixture \
  -- --exact --test-threads=1
for suffix in 1 3 5 7 9 11 13 15 17 19 21; do
  case_file="language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--${suffix}.js"
  ./target/debug/lila --jobs 1 test262 run "$case_file" \
    --suite-root test262/vendor/test262 --execution-backend wasm-aot \
    --timeout-ms 180000 --threads 1
done
```

The durable fixture also covers an initially missing property throwing before
RHS evaluation and a sloppy accessor deletion being recreated on the same
global object. The broader `...lref--v-` prefix is a useful adjacent regression
check because it includes the eleven already-green with-environment cases, but
it is not a separate twenty-two-file claim for this batch. The full language
subtree and pinned matrix remain later verification checkpoints.
