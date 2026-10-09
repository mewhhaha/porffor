# Global Environment numeric update retains one Reference

## Scope and exact cohort

This contract covers the four numeric update forms whose operand is an
IdentifierReference and whose ResolveBinding walk reaches a Global Environment
Record. That Record can select a lexical binding or its Object Environment
Record. The exact original object-record cohort is:

- `language/expressions/prefix-increment/operator-prefix-increment-x-calls-putvalue-lhs-newvalue--1.js`;
- `language/expressions/prefix-decrement/operator-prefix-decrement-x-calls-putvalue-lhs-newvalue--1.js`;
- `language/expressions/postfix-increment/operator-x-postfix-increment-calls-putvalue-lhs-newvalue--1.js`;
- `language/expressions/postfix-decrement/operator-x-postfix-decrement-calls-putvalue-lhs-newvalue--1.js`.

Each file enters a strict nested function, obtains `x` through a configurable
accessor installed on the global object, deletes that property during GetValue,
and requires the later SetMutableBinding to throw ReferenceError. The adjacent
bare-suffix files use an Object Environment Record introduced by `with`; they
remain regression controls under the with-environment contract.

The plain assignment witness is not part of this cohort: assignment has no
GetValue or ToNumeric phase, and its exact global deletion case already passes.
Logical assignments have a separate short-circuit lifecycle. Eager arithmetic
and bitwise compound assignments share the retained Environment Reference owner.
Property References, declarative bindings, resumable functions, modules, and
dynamic source generation are not claims of this batch.

At pre-batch HEAD `f6b6af6a1779840eaf5d7c88cff2b9ff33db9381`, the exact
prefix-increment global witness reported `0/1` as Runtime/NotImplemented with
`unsupported in lila wasm-aot first slice: unbound identifier \`x\``. The
adjacent plain-assignment witness reported `1/1`. These are focused current-SHA
measurements against pinned Test262 `aa55200`; they are not a full subtree or
pinned-matrix publication.

## Normative lifecycle

For any in-scope `++x`, `--x`, `x++`, or `x--`:

1. ResolveBinding reaches the Global Environment Record after checking the
   intervening environments. Its HasBinding first checks its declarative
   record, then calls the Object Record's HasBinding if no lexical exists.
   The latter observes HasProperty on the compiler-owned global object and
   never reads `Symbol.unscopables`.
2. If no binding exists, the Reference is unresolvable and GetValue throws
   ReferenceError before ToNumeric in sloppy or strict code.
3. Otherwise retain that exact **Global Environment Record** as `[[Base]]`,
   together with the source strictness. Do not cache the selected delegate.
4. GetValue calls its GetBindingValue. It checks the current declarative record
   again, including lexicals installed by a HasProperty hook. Only an object
   delegate performs the independent HasProperty/Get sequence; a false object
   recheck throws in strict mode or returns `undefined` in sloppy mode.
5. Apply ToNumeric exactly once, then apply the increment/decrement delta in
   the resulting Number or BigInt domain.
6. PutValue uses the same retained Global Record without resolving again. Its
   SetMutableBinding checks the current lexical delegate after Get/coercion;
   a newly installed lexical receives the value, and const/TDZ rules apply.
7. If still delegated to the Object Record, SetMutableBinding performs its own
   HasProperty recheck. Absence throws in strict mode; the sloppy object path
   remains observable and follows the Object Record write algorithm.
8. Only a successful PutValue permits prefix to return the new numeric value
   or postfix to return the old numeric value.

When all three phases delegate to the object record, the initial HasBinding,
GetBindingValue and SetMutableBinding HasProperty operations are distinct
observations. A raw global read plus checked write omits required resolution.

## Rust invariant and IR composition

`EnvironmentIdentifierIr::global` fixes the resolution start to the actual
Global Environment, separate from the public `globalThis` property. Its closed
`Update { operation, return_mode }` operation reuses the existing AOT retained
Reference lifecycle: resolve, Get, ToNumeric/delta, Put, then return the old/new
value and release the roots. Global Get and Put refresh the live declarative
delegate through their shared owner, including abrupt completions.

Every source-global update uses this path, including declared `var`, initially
known own properties, and a global fallback after explicit `with` selection.
Known property presence cannot substitute for a runtime Environment Reference.
Source facts are invalidated for possible Has/Get/coercion effects; known
callable candidates remain Open rather than becoming invocation authority.
The lowerer exhaustively maps all four `UpdateOp` forms to `NumericUpdateOp`
and `UpdateReturnMode`.

Explicit `with` selection still owns its selected Object Environment Record and
its `Symbol.unscopables` observation. Its `NumericUpdateBindings` fixed-role
carrier preserves Get, numeric delta, same-object Put and prefix/postfix result.
Only the unselected global fallback starts the Global Environment resolver;
local and captured declarative fallbacks retain their original storage.

No new backend expression or parallel numeric algorithm is introduced. The old
object-only `GlobalObjectEnvironmentReferencePlan` is removed because it cannot
represent a live Global Record's lexical delegate.

The joined source repair adds controls to
`aot_ordinary_global_assignment_reference::global_prototype_hasbinding_precedes_rhs_and_plain_assignment_does_not_get`
for a pre-existing late lexical, lexical installation during numeric coercion,
and declared-global initial HasProperty returning false with zero Get calls.
Parsed IR controls cover all four forms, same-spelled locals/captures and With
fallback ownership. These new source and native controls are pending execution;
the earlier focused evidence below does not verify this repair.

## Verification

The focused ladder after batch integration is:

```sh
cargo fmt --all --check
cargo test -p lila-ir lowers_script_global_update_from_class_constructor --quiet
cargo test -p lila-aot-wasm --test structure_language --quiet -- \
  global_object_environment_numeric_update_structure::
cargo test -p lila-cli --test cli \
  language::run_wasm_backend_succeeds_for_global_object_environment_numeric_update_fixture \
  -- --exact --test-threads=1
for case_file in \
  language/expressions/prefix-increment/operator-prefix-increment-x-calls-putvalue-lhs-newvalue--1.js \
  language/expressions/prefix-decrement/operator-prefix-decrement-x-calls-putvalue-lhs-newvalue--1.js \
  language/expressions/postfix-increment/operator-x-postfix-increment-calls-putvalue-lhs-newvalue--1.js \
  language/expressions/postfix-decrement/operator-x-postfix-decrement-calls-putvalue-lhs-newvalue--1.js
do
  ./target/debug/lila --jobs 1 test262 run "$case_file" \
    --suite-root test262/vendor/test262 --execution-backend wasm-aot \
    --timeout-ms 180000 --threads 1
done
```

The durable fixture also covers initially absent ReferenceError before
ToNumeric, sloppy getter deletion/recreation, successful prefix/postfix old/new
results, Number and BigInt, and same-global-object identity. The four adjacent
bare-suffix with files and the eleven global eager-compound files are focused
regression controls. Broad language and pinned-matrix publication remain the
central verification checkpoint.
