# Ordered object property definition traversal

`ExprIr::ObjectPropertyDefinition` represents one property of a retained object
literal. Its checked owner supplies the concrete Object target and the existing
`ObjectPropertyIr`; the native emitter uses the same physical property body as
an ordinary literal. Every operand walker visits the target first and then the
single property in the literal's established order.

The string pool shares one exhaustive property collector between literals and
definitions. It pools static keys, visits computed keys before values, and keeps
method-function registration in its existing owner. Finite RegExp candidate
collection shares its original conservative value-only property collector;
computed keys do not become new candidate sources.

Global-object exposure, global-property collection and function reachability use
their existing exhaustive property helpers. Reachability retains method/getter/
setter function identities and spread's builtin dependencies. Derived-constructor
validation shares its original property walker, including computed-key order and
the separation between the outer constructor and method bodies. Throw inference
merges the target, then key and value without reassociating the old literal's
accumulator.

Caller-flow proof requires both the target and the established literal-property
proof. Calls in either operand invalidate the proof; computed properties and
spread keep their existing conservative rejection. A retained-definition control
passes through the real finalized-invocation proof with pure, target-call and
value-call cases. The node produces an Object value, so Reference reconstruction
rejects it and carried PutValue strictness classification returns no Reference
failure.

All old controls remain. This batch is source-only; compilation and semantic
execution are deferred to the combined checkpoint.
