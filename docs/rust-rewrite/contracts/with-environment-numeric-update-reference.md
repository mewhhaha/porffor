# With-environment numeric updates retain one resolved Reference

## Scope and exact cohort

This contract covers all four numeric-update forms whose
LeftHandSideExpression is an IdentifierReference and whose ResolveBinding walk
selects an Object Environment Record introduced by `with`.

The exact cohort is 16 `noStrict` files. Twelve cover function, global and
nested Object Environment Record fallbacks for every operator form:

- `language/expressions/postfix-increment/S11.3.1_A5_T1.js`;
- `language/expressions/postfix-increment/S11.3.1_A5_T2.js`;
- `language/expressions/postfix-increment/S11.3.1_A5_T3.js`;
- `language/expressions/postfix-decrement/S11.3.2_A5_T1.js`;
- `language/expressions/postfix-decrement/S11.3.2_A5_T2.js`;
- `language/expressions/postfix-decrement/S11.3.2_A5_T3.js`;
- `language/expressions/prefix-increment/S11.4.4_A5_T1.js`;
- `language/expressions/prefix-increment/S11.4.4_A5_T2.js`;
- `language/expressions/prefix-increment/S11.4.4_A5_T3.js`;
- `language/expressions/prefix-decrement/S11.4.5_A5_T1.js`;
- `language/expressions/prefix-decrement/S11.4.5_A5_T2.js`;
- `language/expressions/prefix-decrement/S11.4.5_A5_T3.js`.

Four modern witnesses enter a strict nested function from a non-strict
`with`, delete the selected property during GetValue and require
SetMutableBinding to throw ReferenceError:

- `language/expressions/postfix-increment/operator-x-postfix-increment-calls-putvalue-lhs-newvalue-.js`;
- `language/expressions/postfix-decrement/operator-x-postfix-decrement-calls-putvalue-lhs-newvalue-.js`;
- `language/expressions/prefix-increment/operator-prefix-increment-x-calls-putvalue-lhs-newvalue-.js`;
- `language/expressions/prefix-decrement/operator-prefix-decrement-x-calls-putvalue-lhs-newvalue-.js`.

`language/statements/with/unscopables-inc-dec.js` is the adjacent
ResolveBinding-order witness. Property-reference updates, compound assignment,
strict `with` syntax, resumable captured Object Environment Records and general
`with` closure are not claims of this batch. Strict *references created by a
nested function* are in scope even though strict `with` syntax is not.

## Normative lifecycle

For `x++`, `++x`, `x--` or `--x` inside an active `with` chain:

1. ResolveBinding walks Object Environment Records from inner to outer before
   the already-located declarative/global fallback.
2. Each record performs HasBinding, including its observable
   `@@unscopables` lookup, exactly once when visited. The first visible binding
   fixes the Reference base and its `[[Strict]]`.
3. GetValue calls GetBindingValue on that same Object Environment Record. It
   performs a fresh HasProperty recheck and then Get. Either operation can
   complete abruptly.
4. ToNumeric is applied exactly once to the obtained value. The closed
   increment/decrement operation produces the new value in the same Number or
   BigInt domain.
5. PutValue calls SetMutableBinding on the Object Environment Record selected
   in step 2; resolution never restarts. SetMutableBinding performs an
   independent post-Get HasProperty recheck on that same binding object.
6. If that recheck is false and the retained Reference is strict, PutValue
   throws ReferenceError and does not call Set. In sloppy code the recheck is
   still observable, after which Set runs even when it answered false.
7. Only after PutValue succeeds does a prefix form return the new numeric
   value or a postfix form return the old numeric value.

If no Object Environment Record selects `x`, the pre-located fallback performs
the ordinary declarative/global update. An unresolvable fallback throws
ReferenceError only if that branch is reached; it is not a compile-time reason
to reject a program whose `with` binding resolves at run time.

## Rust invariant and IR composition

`SelectedWithEnvironmentObjects` remains the only producer of the structurally
non-empty `WithEnvironmentReferencePlan`. The plan is neither `Clone` nor
`Copy`. Its numeric-update consumer takes `self`, a closed `NumericUpdateOp`, a
closed `UpdateReturnMode`, one opaque
`WithEnvironmentNumericUpdateBindings` role carrier and the already-lowered
fallback. The carrier's sole allocator creates old-value, result and write
bindings in fixed roles, so three same-typed names cannot be transposed at the
call boundary.

For each selected-object branch the consumer composes existing closed IR:

1. materialize `oldValue = binding_object.get_value(...)`;
2. materialize `result = UpdateIdentifier(oldValue, op, return_mode,
   ValueKind::Dynamic)`; this performs ToNumeric exactly once, mutates the
   compiler-private binding to the new Number or BigInt, and retains the
   correct prefix/postfix result;
3. materialize `write = binding_object.put_value(newValue, ...)`, where
   `newValue` is the now-updated private binding;
4. only in the write materialization's body, read `result`.

The three dotted temporary names are allocated by the lowerer and cannot
collide with source bindings. The GetValue and PutValue helpers each retain a
clone of the same already-materialized binding-object identity; neither can
substitute an outer fallback. Because `UpdateIdentifier` already carries the
closed operation/return-mode/value-kind product, no new backend expression or
parallel numeric implementation is introduced.

The fallback update always carries `ValueKind::Dynamic`: the initial
HasProperty/`@@unscopables` observation may mutate a previously inferred
Number fallback to BigInt or another coercible value before returning false.
Static post-expression metadata becomes fully Dynamic. The observation may
mutate the fallback to any value and then either select the `with` object, or
miss and reach a ToNumeric which itself throws before write-back; neither path
justifies a Number-or-BigInt-only postcondition.

A global fallback also performs a run-time HasProperty check immediately
before its Dynamic `GlobalPropertyUpdate`. Initial Object Environment
HasBinding may delete a statically proven global before returning false; an
absent fallback must throw ReferenceError from GetValue in both sloppy and
strict code, never coerce `undefined` and recreate the property. Conversely,
if the same observation creates a formerly unresolvable global before
returning false, the run-time check admits the ordinary Dynamic update. A
configurable tracked global also loses its static `proven_present` fact because
the missing path can throw, be caught, and leave the property absent.

Consequently an empty resolution chain, a second consumption of the same
Reference plan, a fallback target substituted after GetValue, a return before
successful PutValue and a direct `PropertyUpdate` shortcut are all outside the
producer API.

## Verification

The integrated batch is green at the focused boundary:

- `cargo xc`;
- the IR invariant (`1/1`);
- the source-bounded contract suite (`4/4`);
- the Wasm lifecycle fixture (`1/1`);
- the 16 exact files listed above under current-source Wasm-AOT (`16/16`).

The focused commands are:

```sh
cargo fmt --all --check
cargo test -p lila-ir with_environment_numeric_update --quiet
cargo test -p lila-aot-wasm --test with_environment_numeric_update_structure --quiet
cargo test -p lila-cli --test cli \
  language::run_wasm_backend_succeeds_for_with_environment_numeric_update_fixture \
  -- --exact --test-threads=1
./target/debug/lila --jobs 1 test262 run <exact-file> \
  --suite-root test262/vendor/test262 --execution-backend wasm-aot \
  --timeout-ms 180000 --threads 1
```

The Test262 result is a focused cohort, not a full language-subtree or pinned
matrix publication.
