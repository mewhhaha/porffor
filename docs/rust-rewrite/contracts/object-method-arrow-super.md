# Object-method lexical-arrow `super`

## Status and evidence boundary

This contract owns lexical `super` property reads in an arrow nested inside an
object-literal method. At clean commit `039253d27`, the exact current-pin
Test262 cohort is two physical files and four sloppy/strict Script executions:

```text
language/expressions/super/prop-dot-obj-val-from-arrow.js
language/expressions/super/prop-expr-obj-val-from-arrow.js
```

The current Wasm-AOT binary reports `0/4`. Every execution reaches
Runtime/NotImplemented with the exact diagnostic
`unsupported in lila wasm-aot first slice: object literal method`.

This is not a general object-method or resumable-function failure. At the same
commit, `object/concise-generator.js` is `2/2`, the two
`generator-super-prop-*` files are `4/4`, and the two
`async-super-call-{body,param}.js` files are `4/4`. The missing state is the
lexical arrow's captured pair of invocation `this` and the enclosing method's
`[[HomeObject]]`.

## Normative lifecycle

For an arrow containing a super-property Reference:

1. Walk outward through arrow owners only. An ordinary function is a lexical
   `super` boundary even when that function is nested in a method.
2. If the first non-arrow owner is an object or class method/getter/setter,
   resolve the arrow's lexical `this` and `[[HomeObject]]` to that owner's
   activation. The two bindings are one semantic capability and must be
   captured together.
3. If the first non-arrow owner is a derived constructor, retain the existing
   derived-constructor activation capability instead. It owns active function,
   new target, initialized-this state and this value; it is not interchangeable
   with a method HomeObject capability.
4. If no eligible owner exists, do not invent a HomeObject binding. Existing
   syntax/early-error handling remains authoritative.
5. A HomeObject-owning activation exposes the compiler-private
   `$homeObject` binding before parameter and body closures are lowered. The
   binding receives the exact function-context HomeObject installed by
   MakeMethod; it is never reconstructed from the property name or call
   receiver.
6. The arrow captures both `$this` and `$homeObject` through the normal lexical
   environment chain. Multiple nested arrows preserve the same owner and do
   not manufacture an intermediate HomeObject.
7. At execution, the arrow's super-property Reference reads GetSuperBase from
   captured `$homeObject` and uses captured `$this` as Receiver. Named and
   computed keys share this lifecycle; computed-key evaluation remains in the
   existing Super Reference order.

## Closed Rust seam

`OwnerPlan` carries a required, closed `LexicalSuperOwnerRole`:

- `None`;
- `HomeObject`;
- `DerivedConstructorActivation`.

There is no boolean pair and no catch-all mapping. Every `OwnerPlan` literal
must choose one role, so a new owner kind cannot silently inherit class-only
behavior.

A sole exhaustive projection from `FunctionProtocolIr` assigns
`HomeObject` to object/class methods, getters and setters, and `None` to
ordinary standalone functions and arrows. Class execution owners which are not
ordinary function plans, including field/static-block execution owners, choose
their role explicitly. A derived constructor overrides the ordinary class
owner role with `DerivedConstructorActivation`.

`lexical_super_owner_role(owner_id)` is the only ancestry query. It walks
through owners whose flavor is `Arrow`, stops at the first non-arrow owner, and
returns that owner's required role. The former semantic test against
`class_execution_ids` is deleted: an allocation registry is not authority for
which lexical capability an owner provides.

When generic function planning sees `HomeObject`, it inserts
`LEXICAL_HOME_OBJECT_NAME` into both the activation's root bindings and the
parameter-environment binding authority before activation registration.
`record_lexical_super_property_refs` then matches the resolved role
exhaustively:

- `HomeObject` records `LEXICAL_THIS_NAME` and
  `LEXICAL_HOME_OBJECT_NAME`, in that order;
- `DerivedConstructorActivation` records the existing four derived activation
  bindings;
- `None` records nothing.

The downstream lowering and AOT path already consumes these bindings. A
captured HomeObject makes the arrow's class/super lowering context available;
the owner activation copies the function context's HomeObject into its owned
environment slot; and the arrow loads that captured slot when evaluating
GetSuperBase. No second backend HomeObject representation is introduced.

## Scope and nonclaims

The exact conformance claim is the two-file, four-execution cohort above.
The durable fixture may additionally cover multiple nested arrows, detached
method calls, later prototype replacement and named/computed reads as contract
oracles without increasing the Test262 numerator.

This batch does not claim `super()`, dynamic source generation, private
references, class-field expansion, or arbitrary suspended-body control flow.
It also does not claim Super Reference numeric/compound update closure. That
path must retain one evaluated GetSuperBase across GetValue, RHS evaluation and
PutValue; pinning only `this` or the key would leave the current read/write base
reload bug intact.

## Ownership

The producer lane owns this contract and `lila-ir` owner-role analysis plus
focused IR invariants. The evidence lane owns the CLI fixture, bounded
IR/AOT structure witness, exact Test262 inventory, task status and README. The
review lane audits the existing AOT environment-slot producer/consumer and the
ordinary-function lexical boundary without changing backend product code unless
the compile forces an exhaustive consumer update.

## Verification ladder

Cheap implementation checks:

```sh
cargo fmt --all -- --check
git diff --check
./scripts/check-module-boundaries.sh
```

Central focused verification after the lanes are assembled:

```sh
cargo check --workspace --all-targets
cargo xc
cargo test -p lila-ir object_method_arrow_super
cargo test -p lila-aot-wasm --test object_method_arrow_super_structure
cargo test -p lila-cli --test cli object_method_arrow_super_fixture

./target/debug/lila test262 run \
  language/expressions/super/prop-dot-obj-val-from-arrow.js \
  --suite-root test262/vendor/test262 --execution-backend wasm-aot \
  --timeout-ms 120000 --threads 1
./target/debug/lila test262 run \
  language/expressions/super/prop-expr-obj-val-from-arrow.js \
  --suite-root test262/vendor/test262 --execution-backend wasm-aot \
  --timeout-ms 120000 --threads 1
```

The exact cohort must report `4/4`, with zero unsupported, crash, timeout or
runtime-failure outcomes. The generator and async controls above must remain
green. This focused result is not a complete current-pin Test262 aggregate.
