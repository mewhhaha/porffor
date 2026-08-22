# Object-literal method `[[HomeObject]]`

## Status and evidence boundary

This contract owns non-resumable ordinary methods, getters, and setters created
by object literals, including a default parameter that evaluates a super
property. At clean commit `304e4bbad3`, the current-pin Test262 cohort is five
physical files and ten sloppy/strict Script executions:

```text
language/expressions/object/method.js
language/expressions/object/method-definition/name-super-prop-body.js
language/expressions/object/method-definition/name-super-prop-param.js
language/expressions/object/getter-super-prop.js
language/expressions/object/setter-super-prop.js
```

The existing Wasm binary reports `0/10`. Every execution reaches
Runtime/NotImplemented with the exact diagnostic
`unsupported in lila wasm-aot first slice: object literal method`.

This is one semantic failure, not five unrelated syntax gaps. Object-literal
methods without `super` already lower through `ObjectPropertyIr`; the missing
state is the exact object supplied to MakeMethod as the function's
`[[HomeObject]]`. The lowerer consequently rejects super-property evaluation
because the object-method function has neither a class context nor a lexical
home-object capture. The backend then treats a method function like a data
property value and never attaches the allocated literal object to it.

## Normative lifecycle

For each object-literal method definition in this batch:

1. Allocate the object literal before evaluating any property definition.
2. Visit property definitions in source order. For a computed property name,
   evaluate the key expression and perform ToPropertyKey before creating the
   method closure. A static property name has no key evaluation.
3. Create a function with the exact object-method protocol: ordinary method,
   getter, or setter. These protocols are non-constructable and have ordinary
   execution in this batch.
4. Perform MakeMethod by storing the same allocated literal object as the
   function's `[[HomeObject]]` before defining the property. The function and
   object identities are not recomputed.
5. Define an ordinary method as an enumerable data property, or merge a getter
   or setter into the enumerable accessor descriptor, preserving the existing
   property-definition order and attributes.
6. On invocation, a super-property Reference computes its base from
   `GetFunctionRealm`-independent runtime state:
   `GetPrototypeFromConstructor` is not involved; GetSuperBase reads
   `[[GetPrototypeOf]]` from the retained `[[HomeObject]]`. A later
   `Object.setPrototypeOf(literal, proto)` is therefore observed.
7. A super-property read uses the actual method `this` as Receiver. This
   preserves inherited accessor receiver identity.
8. A super-property write evaluates and retains the actual method `this` while
   forming the Reference, then evaluates the RHS once, and invokes
   `superBase.[[Set]](key, value, this)`. A failed set throws according to the
   Reference's carried strictness; it must never write with `superBase` as both
   target and Receiver.

The object method's home-object authority exists before parameter
initialization, so `method(x = super.toString) {}` follows the same path as a
super property in the body.

## Closed Rust seam

`FunctionProtocolIr` owns the object-method role through these exact variants:

- `ObjectMethod(FunctionExecutionKind)`;
- `ObjectGetter`;
- `ObjectSetter`.

`ObjectMethod(Ordinary)`, `ObjectGetter`, and `ObjectSetter` are the admitted
protocols for this batch. The execution-kind payload keeps generator, async,
and async-generator identities explicit in the closed domain, even though
their resumable HomeObject transport remains a nonclaim. Protocol queries for
flavor, execution kind, constructability, class role, and object-literal
HomeObject must match these variants exhaustively.

`ObjectMethodFunctionIr` is the public IR carrier. It has private fields and no
public constructor. The private exhaustive `ObjectMethodProtocolIr` domain and
`object_method_protocol` mapping are the sole AST-kind decision; the carrier's
crate-private constructor accepts that role instead of an arbitrary
`FunctionProtocolIr`, and analysis derives its protocol from the same role. An
ordinary function, arrow, constructor, class member, or builtin protocol
therefore cannot be passed into the carrier. It exposes the exact function
identity and protocol through read-only accessors.

Every `ObjectPropertyIr::{Method, ComputedMethod, Getter, ComputedGetter,
Setter, ComputedSetter}` carries `ObjectMethodFunctionIr`, never a generic
`TypedExpr`. Data properties continue to carry values. This type difference
makes grouping method materialization into the data-property expression arm a
compile error and forces every exhaustive IR consumer to acknowledge the
HomeObject-bearing role.

`ExprIr::SuperPropertyRead` and `ExprIr::SuperPropertyWrite` carry an explicit
`receiver: Box<TypedExpr>`. The sole lowering producer creates that receiver
with `lower_current_this`; for a write it does so before lowering the RHS.
`ReferenceBase::Super` retains the same receiver when a read becomes a
compound-assignment Reference. Backend consumers therefore receive distinct
target and Receiver inputs; adding the field makes the existing exhaustive
emission arms fail to compile until they consume it.

The AOT consumer must materialize an `ObjectMethodFunctionIr` through a typed,
consuming request that pairs it with the allocated object-literal local, then
store that object in the function context before property definition. Existing
class-member HomeObject storage and super-base loading may be generalized, but
the object-method protocol must not masquerade as `ClassMethod` or acquire
class-only semantics.

## Scope and nonclaims

The exact conformance claim is limited to named non-resumable ordinary methods,
getters, and setters in scripts and ordinary functions. Static and computed
keys share the closed IR carrier and property-order invariant; computed-key
behavior is structural coverage rather than part of the five-file count.

Generator, async, and async-generator object methods remain explicit protocol
members. The direct generator and async SuperProperty witnesses measured after
this batch are green, but complete suspension-safe HomeObject transport remains
outside this contract. Nested arrows using an enclosing object method's
`super` are owned by `object-method-arrow-super.md`. Direct `super()` early
errors, class methods and fields, private elements, optional chains, dynamic
source generation, and cross-realm function construction remain separate.
Nothing here changes ordinary object-literal data properties.

## Ownership

The producer lane owns the contract and the `lila-ir` protocol, analysis, IR,
lowering, traversal, and focused invariant tests. The backend lane owns AOT
planning metadata, object-method materialization, function-context HomeObject
storage/loading, the distinct super-write Receiver, and temp-local budgets. The
evidence lane owns CLI fixtures, structure witnesses, Test262 inventory, task
status, and README publication.

## Verification ladder

Cheap implementation checks:

```sh
cargo fmt --all -- --check
git diff --check
./scripts/check-module-boundaries.sh
```

Central focused verification after all lanes are assembled:

```sh
cargo check --workspace --all-targets
cargo test -p lila-ir object_literal_home_object
cargo test -p lila-aot-wasm --test object_literal_home_object_structure
cargo test -p lila-cli --test cli object_literal_home_object_fixture

./target/debug/lila test262 run language/expressions/object/method.js \
  --execution-backend wasm --timeout-ms 120000 --threads 1
./target/debug/lila test262 run \
  language/expressions/object/method-definition/name-super-prop \
  --execution-backend wasm --timeout-ms 120000 --threads 1
./target/debug/lila test262 run language/expressions/object/getter-super-prop.js \
  --execution-backend wasm --timeout-ms 120000 --threads 1
./target/debug/lila test262 run language/expressions/object/setter-super-prop.js \
  --execution-backend wasm --timeout-ms 120000 --threads 1
```

The exact cohort must report `10/10`, with zero unsupported, crash, timeout, or
runtime-failure outcomes. Resumable object methods and the broader object
expression subtree remain separate gates.
