# Function arguments binding: one validated protocol, one binding

## Decision

Each function builder owns one private `FunctionArgumentsProtocol`. Its
binding lifecycle is closed:

```text
Pending(Absent)  -> BoundAbsent
Pending(Present) -> BoundPresent
```

Parameter binding consumes the pending state exactly once. A present state
moves one binding action into `ArgumentsBindingProtocol`: materialize its
mapped or unmapped construction plan, defer a validated mapped object in one
native-GC invocation frame, or keep the implicit binding while eliding a
proven-unobserved object. Each action owns its binding initialization.

Local-count planning and root-variable reuse need only know whether the
function has its own arguments binding. Their reusable `present()` projection
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
`initialize_arguments_binding` accepts the owned materialization plan, so a
caller cannot invoke it twice with the same authority. The elided arm creates
binding storage and initializes it to `undefined`; it allocates no object.
The deferred arm also initializes the unobserved binding to `undefined`, then
transfers the validated mapping to a native invocation frame. That frame caches
exactly one canonical Arguments object if legacy reflection observes it; it
never reinitializes the function's binding.

## Retained reusable projections

`MappedArgumentEntry`, `ArgumentIndex` and `ParameterEnvironmentSlot` remain
copyable. Arguments-object emission legitimately projects each validated
mapping into its argument index and environment slot at separate points. They
carry no authority to initialize a binding and are not part of the one-shot
lifecycle.

## Enforced invariants

1. A function arguments protocol starts pending and reaches exactly one
   terminal binding state.
2. Eager materialization, deferral or proven elision moves through one
   non-cloneable binding authority. The mapped/unmapped semantics are retained.
3. Presence-only planning cannot recover the binding action or construction
   protocol.
4. Eager binding initialization consumes an owned materialization plan;
   deferred materialization consumes the frame's pending state once.
5. A repeated binding attempt fails explicitly instead of silently creating a
   second object.

## Verification boundary

The Rust-lexical structure guard pins the private closed state, the one-shot
transition, the presence-only projection and the consuming initialization
route. Arguments-protocol module tests cover absent, mapped and unmapped
classification, proven elision, validated mapped slots, last-duplicate
selection, malformed storage rejection and repeated-binding rejection.

The ownership protocol preserves the existing arguments-object behavior when
the object is observable. It does not complete parameter/body environment
separation or claim full ECMAScript/Test262 conformance.
