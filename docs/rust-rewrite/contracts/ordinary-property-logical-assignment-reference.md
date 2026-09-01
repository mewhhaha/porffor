# Ordinary property logical-assignment Reference

Status: normative implementation contract for `&&=`, `||=` and `??=` through
an ordinary property Reference.

## Exact conformance boundary

The selected strict false-`Set` cohort is:

- `lgcl-and-assignment-operator-no-set-put.js`;
- `lgcl-or-assignment-operator-no-set-put.js`;
- `lgcl-nullish-assignment-operator-no-set-put.js`;
- `lgcl-and-assignment-operator-non-writeable-put.js`;
- `lgcl-or-assignment-operator-non-writeable-put.js`;
- `lgcl-nullish-assignment-operator-non-writeable-put.js`;
- `lgcl-or-assignment-operator-non-extensible.js`; and
- `lgcl-nullish-assignment-operator-non-extensible.js`.

Every path is under
`language/expressions/logical-assignment/`, has `flags: [onlyStrict]`, and
asserts that a taken logical-assignment branch throws `TypeError` when
PutValue's `Set` returns false. At clean pre-batch commit `04e38f2ba`, the
three `no-set-put.js` files were measured separately and reported `0/3`, all as
`Runtime/Bug`; the runner supplied an error value where `assert.throws`
required a real error object. The remaining five files select the same
strict-false transition but were not separately measured before the patch.

The independent evaluation-order controls are the three
`lgcl-{and,or,nullish}-assignment-operator-lhs-before-rhs.js` files. Their six
sloppy/strict executions were already `6/6` at the same baseline. The
short-circuit controls are:

- `lgcl-and-assignment-operator-no-set.js`;
- `lgcl-or-assignment-operator-non-writeable.js`; and
- `lgcl-and-assignment-operator-non-extensible.js`.

They directly prove that an untaken branch does not reach PutValue. The durable
runtime fixture below separately observes that the RHS is skipped. None of
these exact paths has a runner rewrite or known-failure mask.

## Normative Reference lifecycle

For `base[key] op= rhs`, the compiler must preserve one ordinary property
Reference through this order:

1. evaluate `base` and propagate an abrupt completion;
2. evaluate the raw computed-key expression and propagate an abrupt
   completion;
3. begin GetValue on the retained Reference: reject a nullish base before
   observable `ToPropertyKey` coercion;
4. derive `ToObject(base)` as the GetValue target `O`, apply `ToPropertyKey`
   exactly once, then perform `O.[[Get]](key, GetThisValue(reference))`;
5. select the branch from the obtained value: `&&=` takes a truthy value,
   `||=` takes a falsy value, and `??=` takes `null` or `undefined`;
6. on a short circuit, publish the obtained value without evaluating `rhs` or
   attempting PutValue;
7. on a taken branch, evaluate `rhs` and propagate an abrupt completion;
8. perform PutValue's `[[Set]]` on the object obtained from the original base,
   using the original receiver and the same canonical key as GetValue;
9. if `Set` returns false, throw `TypeError` only when the captured Reference
   is strict; and
10. publish `rhs` only after PutValue completes normally.

The target and receiver are distinct semantic roles. ECMAScript describes a
`ToObject(base)` in both GetValue and PutValue, while an inherited getter or
setter observes the original primitive as `Receiver`. This backend safely
retains the first boxed target for the later Set rather than allocating an
equivalent second wrapper; that is an implementation optimization, not a
spec-mandated single boxing. Losing the target/receiver distinction or
re-evaluating the key is wrong; a semantically equivalent reboxing backend is
not forbidden by this contract.

## Closed producer shape

`OrdinaryPropertyReferencePlan` is the sole producer. It is private,
non-`Clone`, non-`Copy`, `#[must_use]`, and owns one evaluated
base-and-receiver expression, one raw referenced name and the Reference's
`Strictness`. Its consuming transition is:

```text
logical_assignment(op, rhs, possible_getters, possible_setters) -> TypedExpr
```

The closed `LogicalBinaryOp` and retained RHS become one
`ExprIr::OrdinaryPropertyLogicalAssignment`. The carrier has no public
constructor and exposes only:

- `base_and_receiver()`;
- `referenced_name()`;
- `rhs()`;
- `op()`;
- `strictness()`;
- `possible_getters()`; and
- `possible_setters()`.

Only `PropertyAccess::Simple` enters this carrier. Super and private property
References retain their dedicated lowering paths. A decomposed
`LogicalShortCircuit(PropertyRead, PropertyWrite)` is not an alternative
source-level representation because it cannot make single-key and same-target
ownership structural.

The accessor target domain is semantic reachability, not speculative
optimization metadata. Exact shape and descriptor discoveries remain a small
owned set. Lost shape or arbitrary source effects add one shared immutable
universe of planned source functions through `PropertyHookTargets`; carriers
and flow snapshots clone only its `Arc`, not every function ID. Backend
planning must retain every source body which the eventual `[[Get]]` or
branch-local `[[Set]]` can dispatch. Plain assignment, eager compound
assignment and numeric update carry the same provenance, so all four ordinary
mutation carriers have one rooting rule.

## Flow-fact and implicit-call transaction

Lowering treats `ToPropertyKey`, ordinary accessors and Proxy traps as calls to
unknown source code even though the source expression contains no explicit
call node. Before any fact from such a hook may be reused, one shared
invalidation transaction:

- widens captured mutable bindings and script/global property facts;
- discards tracked heap shapes for bindings, constructor `this` and global
  mirrors;
- clears prototype guards and exact static caches;
- retains exact dynamically installed getter/setter ledgers while setting a
  monotone unknown-effect bit whose hook domain refers to the shared planned
  source universe; and
- propagates that effect summary out of nested function lowerers.

Potentially effectful base and raw-key expressions have both pre- and
post-lowering boundaries. For read/modify/write carriers, an object-valued key
also invalidates before the skipped branch is captured and before `[[Get]]`.
A known source getter invalidates immediately after its receiver information is
merged. The taken RHS is invalidated before its flow snapshot is joined with
the skipped path; possible setter effects are applied to that joined state, so
no stale pre-hook fact can be resurrected. A Proxy never carries an ordinary
empty-object shape.

The retained Reference preserves receiver identity, not a promise that its
heap contents stay unchanged. Getter receiver inference drops the old heap
shape when key coercion can run source code; setter inference drops it after
RHS evaluation or numeric conversion. Exact ordinary accessors then apply the
callee's own `this` mode: strict accessors observe a primitive receiver,
whereas sloppy accessors observe its boxed object. Unknown hook provenance is
broader because the same callable may instead be a Proxy trap whose `this` is
the handler. Exact getters observe zero supplied arguments and exact setters
observe one; omitted formals are joined with `undefined`. Unknown hooks widen
the maximum four-argument Proxy prefix and join every later formal with
`undefined`.

An unobserved sloppy-function `this` has an object-like kind domain but no heap
shape. This is an invariant at the function-entry boundary, so copying `this`
into another binding cannot turn the fallback global object into false receiver
evidence. Reflective and Proxy results likewise omit shapes whenever the
operation cannot prove an ordinary object layout.

A conditional receiver retains an immutable flattened set of leaf receiver
facts alongside its merged runtime value. Every possible write invalidates each
leaf and every shape that reaches it through properties, array elements, boxed
primitive contents, or a prototype chain. Mutation authority is also derived
per leaf, so two known ordinary-object alternatives cannot masquerade as an
unknown global or intrinsic-prototype receiver merely because their distinct
shapes do not merge.

The RHS is lowered as a conditional transaction. Scopes, `var` bindings,
current and constructing `this`, globals, prototype guards, accessor ledgers
and exact caches are joined between the skipped and taken paths. A fact is
preserved only when both paths support it; a logical RHS cannot overwrite an
unrelated fact merely because lowering visited the taken branch.

The direct-global value captured after this join is the input to the possible
outer write. This is deliberately later than the old pre-RHS snapshot: if the
RHS changes the property and makes the outer `Set` fail, the RHS mutation must
remain in the final domain alongside the skipped value and the possible outer
write value.

`Object.defineProperty`, `Object.defineProperties`, `Object.setPrototypeOf`
and the corresponding mutating reflection operations discard exact shape
knowledge. Their returned target also carries no pre-mutation shape. This
makes accessors installed through a descriptor bag or a new prototype visible
to a later Reference carrier.

A possible property write invalidates aliases of its base shape. This includes
top-level aliases and enclosing object properties, array elements, prototypes
and boxed-value shapes which recursively contain that base. The compiler may
discard an entire enclosing shape; retaining a nested pre-write type fact is
not permitted. The shape representation is an owned finite tree, so runtime
object cycles are not represented in this traversal.

Ordinary `delete` performs the same enclosing-alias invalidation because a
successful delete can expose an inherited accessor, while a failed delete can
retain the own property. Destructuring property targets currently take the
broader unknown-effect transaction: their target carrier does not yet encode a
closed accessor set, and PutValue can dispatch `__proto__`, a source setter or
a Proxy trap.

Finally, `ToPropertyKey`, `[[Get]]`, conversion and `[[Set]]` hooks can throw
any ECMAScript value. Catch-binding inference therefore includes an unknown
runtime value for every ordinary mutation carrier. A strict failed-Set
`TypeError` is an additional path, never a reason to narrow away a string,
number or other value thrown by user code.

Possible setters also observe the value actually passed to `[[Set]]`: plain
and logical RHS values, eager applied results, and numeric Number-or-BigInt new
values. Function-parameter facts carry an explicit observation state, so an
unknown accessor/Proxy convention cannot later be narrowed again by a direct
call. Lost-shape hooks conservatively observe unknown `this` and the maximum
Proxy trap argument prefix; exact accessors retain the narrower ordinary
getter/setter convention.

## Closed backend staging

The backend moves through three typed states:

1. the evaluated original base/receiver and raw key;
2. the boxed target, original receiver, canonical key and obtained old value;
3. the taken branch's RHS plus the result of `Set`.

Only the second state may choose the logical branch. Only the taken branch may
compile the RHS and request the shared ordinary-`Set` helper. Publication of
the RHS follows normal completion of that helper and strict-false routing.
Exhaustive matches over `LogicalBinaryOp` determine which side of the Wasm
`if` is taken; there is no catch-all operation arm.

The shared ordinary-reference GetValue transition also serves eager compound
assignment and numeric update. Its boxed-target optimization is therefore a
deliberate shared backend invariant: those carriers retain `O` separately from
the primitive receiver through their later PutValue as well.

Temporary-local planning names each simultaneously live phase. In particular,
the strict failed-Set branch budgets runtime-error construction together with
its nested descriptor flags while all write-persistent Reference locals remain
live. The depth regression crosses the backend's minimum local-count floor so
that a one-local undercount cannot be masked by shallow programs.

## Durable runtime contract

`wasm_ordinary_property_logical_assignment_reference.js` makes the lifecycle
observable without dynamic source generation. It covers:

- one base evaluation, one raw-key evaluation and one `ToPropertyKey` across
  Get and taken Set;
- taken and short-circuited behavior for all three operators;
- nullish-base, raw-key, RHS and setter abrupt completion ordering;
- result nonpublication after an abrupt RHS, abrupt Set or strict false Set;
- sloppy false Set, strict missing-setter, non-writable and non-extensible
  targets;
- a primitive Number receiver across all six taken/short-circuit modes;
- conditional flow joins, global aliases, script-global mirrors and dynamic
  global keys;
- key-coercion, getter, setter and Proxy effects invalidating global and
  prototype facts;
- accessors installed by nested calls, descriptor bags, prototype mutation
  and the logical RHS itself;
- nested object and array aliases losing stale shapes after a write;
- arbitrary getter/setter throw values reaching strict catch bindings for all
  four ordinary mutation carriers;
- a possible `globalThis` write invalidating a narrower global-property fact;
- a possible `Array.prototype` write disabling later intrinsic method
  selection.

The source-bounded structure test owns the exact eight false-Set witnesses,
three order controls and three short-circuit controls, and rejects exact
runner/known-failure masks.

## Explicit nonclaims

This contract does not close logical assignment through Super, private names,
identifiers, Global/Object Environment Records, `with`, optional chains or
suspending References. It does not claim the complete logical-assignment
directory or the pinned Test262 matrix. Dynamic source generation and the
spec-exec debug oracle are unchanged.

## Verification

```sh
cargo fmt --all -- --check
CARGO_BUILD_JOBS=1 cargo check --workspace --all-targets
CARGO_BUILD_JOBS=1 cargo test -p lila-ir ordinary_property_logical -- \
  --test-threads=1
CARGO_BUILD_JOBS=1 cargo test -p lila-aot-wasm \
  --test ordinary_property_logical_assignment_structure -- --test-threads=1
CARGO_BUILD_JOBS=1 cargo test -p lila-cli --test cli -- \
  --exact language_numerics::run_wasm_backend_preserves_ordinary_property_logical_assignment_reference \
  --test-threads=1
```

Run all fourteen exact Test262 paths separately under `--execution-backend
wasm-aot`; basename-only or shared-fragment filters are not valid selection
evidence. Publication must report the false-Set, order and short-circuit
cohorts separately.
