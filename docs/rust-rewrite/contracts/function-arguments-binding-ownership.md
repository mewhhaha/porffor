# Function arguments binding: one validated protocol, one binding

## Decision

Each function builder owns one private `FunctionArgumentsProtocol`. Its
binding lifecycle is closed:

```text
Pending(Absent)  -> BoundAbsent
Pending(Present) -> BoundPresent
Pending(UnobservableLeaf) -> BoundAbsent
```

Parameter binding consumes the pending state exactly once. A present state
moves its mapped or unmapped construction protocol into one
`ArgumentsBindingProtocol`; consuming that authority is the only route to
arguments-object initialization.

User functions first consume a `UserFunctionActivationPlan`. It validates the
original mapped-arguments slots before considering leaf elision, then supplies
the exact source, arguments protocol and owned-binding selection together.
An admitted synchronous leaf replaces present materialization with
`UnobservableLeaf` and drops mapping-only storage in that same plan. Its
nonempty source body still follows normal compilation; the separate empty
strict-body authority is unchanged.

Local-count planning and root-variable reuse need only know whether the
function has its own arguments object. Their reusable `present()` projection
therefore returns `Option<()>`. It cannot expose, borrow or clone the semantic
construction protocol.

## Why ownership is required

FunctionDeclarationInstantiation creates at most one arguments binding. The
former emitter borrowed and cloned the present protocol before initialization,
leaving the original semantic authority available for another binding. A
later duplicate call to parameter binding could therefore construct and
install a second arguments object from the same validated map.

`take_for_binding` moves the protocol out of `Pending` and records which
terminal state was reached. A second call is a compiler-invariant error.
`initialize_arguments_binding` accepts the owned present protocol, so a caller
cannot invoke it twice with the same authority.

## Retained reusable projections

`MappedArgumentEntry`, `ArgumentIndex` and `ParameterEnvironmentSlot` remain
copyable. Arguments-object emission legitimately projects each validated
mapping into its argument index and environment slot at separate points. They
carry no authority to initialize a binding and are not part of the one-shot
lifecycle.

## Enforced invariants

1. A function arguments protocol starts pending and reaches exactly one
   terminal binding state.
2. Mapped or unmapped construction semantics move through one non-cloneable
   binding authority.
3. Presence-only planning cannot recover the construction protocol.
4. Arguments-object initialization consumes an owned present protocol.
5. A repeated binding attempt fails explicitly instead of silently creating a
   second object.

## Verification boundary

The Rust-lexical structure guard pins the private closed state, the one-shot
transition, the presence-only projection and the consuming initialization
route. Arguments-protocol module tests cover absent, mapped and unmapped
classification, validated mapped slots, last-duplicate selection, malformed
storage rejection and repeated-binding rejection.

The guard also pins the activation plan's validation-before-elision ordering
and its consuming builder seam. Production-body checks establish the binding
route without treating additional unit-test calls as new materialization
authority.

This is a source-equivalent compiler ownership closure. It does not add an
arguments-object behavior, complete parameter/body environment separation or
claim a newly passing ECMAScript/Test262 case.
