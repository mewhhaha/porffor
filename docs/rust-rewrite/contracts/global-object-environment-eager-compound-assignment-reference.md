# Global Environment eager compound assignment retains one Reference

## Scope and exact cohort

This contract covers eager compound assignments whose LeftHandSideExpression
is an IdentifierReference and whose ResolveBinding walk reaches the Global
Environment Record. Its live delegate may be a lexical binding or an object
property. The original pinned cohort exercises the Object Record delegate.

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
this batch. Local declarative bindings retain their existing storage paths;
all source-global eager assignments use the retained Global Record lifecycle.

## Normative lifecycle

For an in-scope `x op= rhs`:

1. ResolveBinding reaches the Global Environment Record after the intervening
   environments. Its HasBinding checks its declarative record before the
   Object Record's HasProperty on the actual global object. This precedes
   GetValue and RHS evaluation and never reads `Symbol.unscopables`.
2. If no binding exists, GetValue throws ReferenceError before the RHS in
   either strictness mode.
3. Otherwise retain that exact **Global Environment Record** and strictness.
   The selected declarative/object delegate is not the Reference base.
4. GetBindingValue checks the same Global Record's current lexical table.
   A lexical installed during resolution is now visible; otherwise the Object
   Record independently performs HasProperty and Get, with its strict/sloppy
   absence behavior.
5. Evaluate the original RHS once, then apply the selected arithmetic/bitwise
   operation in ECMAScript coercion order.
6. PutValue retains the same Global Record without resolving again. Its
   SetMutableBinding checks for a lexical installed by Get, RHS or coercion,
   applying the lexical's const/TDZ rules when present.
7. If the object delegate remains selected, its own HasProperty recheck and
   strict/sloppy SetMutableBinding rules apply.
8. Return the applied result only after successful PutValue.

When no lexical intervenes, the initial HasBinding, GetBindingValue and
SetMutableBinding HasProperty calls are distinct observable operations. The
exact witnesses delete `x` in Get, so a raw global read plus checked write does
not implement this lifecycle.

## Rust invariant and IR composition

`EnvironmentIdentifierIr::global` fixes the resolution start to the actual
Global Environment. The closed `EagerCompound { operation, rhs }` form retains
the original source RHS under the shared AOT Reference owner. Its existing
emitter performs resolution, Get, RHS/coercion, Put and root release with one
Reference; Get and Put recheck the live Global Record delegate. The exhaustive
`EagerCompoundAssignmentOp::environment_operation` conversion admits the twelve
eager operators and excludes logical assignment.

All source-global eager assignments use this form, including a Script `var`
with declaration-publication storage and any global fallback after explicit
With selection. Possible resolution/Get effects invalidate static facts before
lowering the RHS; operator hooks invalidate facts after it. Candidate callable
identities remain Open for source admission and native linkage. No object-only
presence/value facts are published for a potentially lexical result.

An explicit With selection retains its actual Object Environment Record.
`WithEnvironmentResolution` alone owns its HasProperty/unscopables visibility
condition. Its `EagerCompoundAssignmentBindings` carrier still fixes the
old-value, result and write-completion roles. Local/captured fallbacks keep
their storage; a global fallback independently starts at the Global Environment
and then retains that Record through Get/RHS/Put.

No new backend expression or parallel arithmetic algorithm is introduced.
The object-only `GlobalObjectEnvironmentReferencePlan` is removed: choosing
an object property once cannot preserve a Global Record's later lexical
delegate changes.

The joined source repair extends
`aot_ordinary_global_assignment_reference::global_prototype_hasbinding_precedes_rhs_and_plain_assignment_does_not_get`
with late global lexical controls and lexical installation during initial Has
or the eager RHS. Parsed IR controls cover all twelve operators, root/nested
owners, original RHS operands, callable candidate retention and With fallbacks.
These new controls are pending execution; earlier focused results do not verify
the joined repair.

## Verification

The focused ladder after batch integration is:

```sh
cargo fmt --all --check
cargo test -p lila-ir script_global_compound_assignments --quiet
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
