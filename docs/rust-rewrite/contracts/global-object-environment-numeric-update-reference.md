# Global Object Environment numeric update retains one Reference

## Scope and exact cohort

This contract covers the four numeric update forms whose operand is an
IdentifierReference, whose lexical ResolveBinding walk finds no declarative
binding, and whose Global Environment Record selects a property through its
Object Environment Record:

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
and bitwise compound assignments use the existing sealed eager operation plan.
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

1. ResolveBinding reaches the Global Environment Record after finding no
   declarative binding. Its Object Record performs HasBinding as one
   HasProperty operation on the compiler-owned global object. This global
   Object Environment Record has `[[IsWithEnvironment]] = false`; it never
   reads `Symbol.unscopables`.
2. If the initial HasProperty is false, the Reference is unresolvable and
   GetValue throws ReferenceError before ToNumeric in sloppy or strict code.
3. If present, retain that exact Object Environment Record as `[[Base]]` and
   retain the strictness of the source which created the Reference.
4. GetValue calls GetBindingValue on that same Object Record. GetBindingValue
   independently performs HasProperty and then Get. A false recheck throws
   ReferenceError for a strict Reference and yields `undefined` for a sloppy
   Reference.
5. Apply ToNumeric exactly once to the obtained value, then apply the selected
   increment or decrement delta without changing Number/BigInt numeric domain.
6. PutValue calls SetMutableBinding on the Object Record selected in step 1;
   resolution never restarts. SetMutableBinding independently performs
   HasProperty after GetValue, ToNumeric, and the delta.
7. If the write recheck is false and the retained Reference is strict, throw
   ReferenceError without calling Set. In sloppy code the recheck remains
   observable, then Set runs even if it answered false.
8. Only after PutValue succeeds does prefix return the new numeric value or
   postfix return the old numeric value.

The initial ResolveBinding, GetBindingValue, and SetMutableBinding HasProperty
operations are three distinct specification observations. A raw global read
plus a checked write omits the first two and is not this contract.

## Rust invariant and IR composition

`NumericUpdateBindings` is one opaque fixed-role carrier for both with and
global Object Environment Records. Its sole allocator creates old-value,
result, and write-completion names in a fixed order. The fields remain private;
callers cannot transpose three same-typed `String`s.

`ObjectEnvironmentBindingObject::numeric_update` owns the shared selected-base
lifecycle. It accepts the typed `NumericUpdateOp`, typed `UpdateReturnMode`, and
the borrowed role carrier, then composes only existing IR:

1. materialize `oldValue = binding_object.get_value(...)`;
2. materialize `result = UpdateIdentifier(oldValue, op, returnMode, Dynamic)`;
3. write the mutated old-value binding through
   `binding_object.put_value(...)`;
4. materialize that write completion;
5. only in the write materialization's body, read `result`.

The update operation mutates the private old-value binding to the new numeric
value while returning old or new according to `UpdateReturnMode`; the separate
result binding retains that expression result across PutValue.

`WithEnvironmentResolution` alone wraps the shared lifecycle in its
HasProperty/`Symbol.unscopables` visibility condition and fallback chain.
`GlobalObjectEnvironmentReferencePlan` owns exactly one compiler-known global
binding object, referenced name, and `Strictness`. It is neither `Clone` nor
`Copy`; its consuming `numeric_update` wraps the shared lifecycle in a plain
initial HasProperty condition whose missing branch is RuntimeThrow
ReferenceError. Thus global selection cannot accidentally consult
unscopables, and with selection cannot accidentally omit it.

Lowering maps all four `UpdateOp` variants exhaustively into the product
`NumericUpdateOp` and `UpdateReturnMode`. Only a pre-located
`LocatedIdentifierReference::Unresolvable` whose global property is not proven
present enters the new plan. Existing declarative and proven-global paths keep
their current specializations. The dynamic global path uses fully Dynamic
runtime tags and does not create or strengthen a static `proven_present` fact;
an initially missing throw can be caught while the property remains absent.

No new backend expression or parallel numeric implementation is introduced.
A caller-built global object expression, an empty or reusable role carrier, a
second plan consumption, an unscopables read on the global path, a restarted
resolution, specialized pre-observation value metadata, a result before
successful PutValue, and a direct `GlobalPropertyUpdate` shortcut are outside
the producer API.

## Verification

The focused ladder after batch integration is:

```sh
cargo fmt --all --check
cargo test -p lila-ir global_object_environment_numeric_update --quiet
cargo test -p lila-aot-wasm \
  --test global_object_environment_numeric_update_structure --quiet
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
