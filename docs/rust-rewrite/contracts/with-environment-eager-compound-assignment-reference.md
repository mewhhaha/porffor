# With-environment eager compound assignments retain one resolved Reference

## Scope and exact cohort

This contract covers eager compound assignments whose LeftHandSideExpression
is an IdentifierReference and whose ResolveBinding walk selects an Object
Environment Record introduced by `with`.

The exact Test262 cohort is 44 `noStrict` files. Thirty-three cover function,
global and nested Object Environment Record fallbacks for each evidenced
operator:

- `language/expressions/compound-assignment/S11.13.2_A5.1_T1.js` through
  `S11.13.2_A5.1_T3.js` (`*=`);
- `S11.13.2_A5.2_T1.js` through `S11.13.2_A5.2_T3.js` (`/=`);
- `S11.13.2_A5.3_T1.js` through `S11.13.2_A5.3_T3.js` (`%=`);
- `S11.13.2_A5.4_T1.js` through `S11.13.2_A5.4_T3.js` (`+=`);
- `S11.13.2_A5.5_T1.js` through `S11.13.2_A5.5_T3.js` (`-=`);
- `S11.13.2_A5.6_T1.js` through `S11.13.2_A5.6_T3.js` (`<<=`);
- `S11.13.2_A5.7_T1.js` through `S11.13.2_A5.7_T3.js` (`>>=`);
- `S11.13.2_A5.8_T1.js` through `S11.13.2_A5.8_T3.js` (`>>>=`);
- `S11.13.2_A5.9_T1.js` through `S11.13.2_A5.9_T3.js` (`&=`);
- `S11.13.2_A5.10_T1.js` through `S11.13.2_A5.10_T3.js` (`^=`);
- `S11.13.2_A5.11_T1.js` through `S11.13.2_A5.11_T3.js` (`|=`).

Eleven modern witnesses enter a strict nested function from a non-strict
`with`, delete the selected property during GetValue and require
SetMutableBinding to throw ReferenceError. They are
`compound-assignment-operator-calls-putvalue-lref--v-.js` and the ten files
with suffixes `--2.js`, `--4.js`, `--6.js`, `--8.js`, `--10.js`, `--12.js`,
`--14.js`, `--16.js`, `--18.js` and `--20.js` in the same directory.

The producer's closed eager domain also includes `**=` because it has the same
Reference lifecycle and already has an exhaustive arithmetic operation in the
IR. It is local invariant coverage, not a forty-fifth Test262 claim. Logical
assignments (`&&=`, `||=`, `??=`), property References, strict `with` syntax,
resumable captured Object Environment Records and general `with` closure are
not claims of this batch. Strict References created by a nested function are
in scope even though strict `with` syntax is not.

## Normative lifecycle

For `x op= rhs` in the eager operator domain inside an active `with` chain:

1. Evaluate the IdentifierReference by running ResolveBinding before the RHS.
   ResolveBinding walks Object Environment Records from inner to outer before
   the already-located declarative/global fallback.
2. Each Object Environment Record performs HasBinding, including its
   observable `@@unscopables` lookup, exactly once when visited. The first
   visible binding fixes the Reference base and its `[[Strict]]`.
3. GetValue calls GetBindingValue on that same Object Environment Record. It
   performs a fresh HasProperty recheck and then Get. Either operation can
   complete abruptly, in which case the RHS is not evaluated.
4. Evaluate `rhs` exactly once, after GetValue. Apply the selected closed
   arithmetic or bitwise operation to the obtained left value and the RHS in
   ECMAScript coercion order.
5. PutValue calls SetMutableBinding on the Object Environment Record selected
   in step 2; resolution never restarts. SetMutableBinding performs an
   independent post-Get/post-RHS HasProperty recheck on that same binding
   object.
6. If that recheck is false and the retained Reference is strict, PutValue
   throws ReferenceError and does not call Set. In sloppy code the recheck is
   still observable, after which Set runs even when it answered false.
7. Only after PutValue succeeds does the compound assignment return the
   applied value.

If no Object Environment Record selects `x`, the pre-located fallback performs
the ordinary declarative/global compound assignment. An uninitialized
declarative fallback throws before RHS evaluation when reached. An
unresolvable fallback performs a run-time global HasProperty check and throws
ReferenceError when still absent; it is not a compile-time reason to reject a
program whose `with` binding resolves at run time.

## Rust invariant and IR composition

`SelectedWithEnvironmentObjects` remains the only producer of the structurally
non-empty `WithEnvironmentReferencePlan`. The plan is neither `Clone` nor
`Copy`. Its eager compound-assignment consumer takes `self`, one sealed
`WithEnvironmentCompoundAssignment` and the already-lowered fallback. A sealed
assignment can only be produced by consuming
`WithEnvironmentCompoundAssignmentBindings`, whose sole allocator creates
old-value, result and write bindings in fixed roles. Callers cannot transpose
three same-typed names or construct the sealed payload directly.

The lowerer maps every eager `AssignOp` exhaustively into
`EagerCompoundAssignmentOp::{Arithmetic, Bitwise}`. It obtains the old-value
operand only through the role carrier, maps the operation exhaustively to the
existing canonical coercive-add, coercive-number or bitwise-numeric IR, and
seals that result into the carrier. For each selected-object branch the plan
then composes existing IR:

1. materialize `oldValue = binding_object.get_value(...)`;
2. materialize `result = apply(oldValue, rhs)`;
3. materialize `write = binding_object.put_value(result, ...)`;
4. only in the write materialization's body, read `result`.

The three dotted temporary names are allocated by the lowerer and cannot
collide with source bindings. Every selected branch and the fallback contains
its own clone of the already-lowered RHS IR, but the ResolveBinding
conditionals execute exactly one branch. The GetValue and PutValue helpers
retain clones of the same already-materialized binding-object identity; neither
can substitute an outer fallback. No new backend expression or parallel
runtime arithmetic implementation is introduced.

The conditional fallback's emitted operand information is always fully
Dynamic. Initial HasBinding/`@@unscopables` and selected GetValue are observable
and may mutate a previously inferred fallback before selecting or rejecting a
`with` binding. Static post-expression metadata for a mutable declarative or
global fallback therefore becomes all-runtime-tags Dynamic, independent of the
normal result domain.

A global fallback performs a run-time HasProperty check immediately before its
Dynamic read/apply/write. Initial Object Environment observation may delete a
statically proven global before returning false; absent GetValue must throw
ReferenceError in sloppy and strict code rather than treating the value as
`undefined`. Conversely, the same observation may create a formerly
unresolvable global before returning false, and the run-time check then admits
the ordinary update. A configurable tracked global loses its static
`proven_present` fact because the missing path can throw, be caught, and leave
the property absent.

Consequently an empty resolution chain, a second consumption of the same plan,
a caller-built old/result/write name tuple, a fallback target re-resolved after
GetValue, a return before successful PutValue and a direct property-update
shortcut are outside the producer API.

## Verification

The integrated batch is green at the focused boundary:

- the IR operation-domain test (`1/1`);
- the source-bounded contract suite (`5/5`);
- the pre-existing numeric-reference suite (`4/4`);
- the Wasm lifecycle fixture (`1/1`);
- the exact 44-file current-source Wasm-AOT cohort (`44/44`).

The focused ladder is:

```sh
cargo fmt --all --check
cargo test -p lila-ir with_environment_compound_assignment --quiet
cargo test -p lila-aot-wasm \
  --test with_environment_compound_assignment_structure --quiet
cargo test -p lila-cli --test cli \
  language::run_wasm_backend_succeeds_for_with_environment_compound_assignment_fixture \
  -- --exact --test-threads=1
./target/debug/lila --jobs 1 test262 run \
  language/expressions/compound-assignment/S11.13.2_A5. \
  --suite-root test262/vendor/test262 --execution-backend wasm-aot \
  --timeout-ms 180000 --threads 1
for suffix in '' 2 4 6 8 10 12 14 16 18 20; do
  case_file="language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v-"
  test -z "$suffix" || case_file="${case_file}-${suffix}"
  ./target/debug/lila --jobs 1 test262 run "${case_file}.js" \
    --suite-root test262/vendor/test262 --execution-backend wasm-aot \
    --timeout-ms 180000 --threads 1
done
```

The first Test262 filter is the exact 33-file historical cohort. The explicit
loop names only the 11 modern `with` targets. A broader filename-prefix run
also measured the 11 adjacent global Object Environment cases as unsupported;
they are follow-up evidence, not part of the 44-file passing claim. This result
is not a full language-subtree or pinned-matrix publication.
