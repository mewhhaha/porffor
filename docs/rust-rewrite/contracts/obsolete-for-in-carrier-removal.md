# Retired per-protocol ForIn carriers

T02 removes the uncalled `OrdinaryGeneratorForInIr` and
`AsyncFunctionForInIr` constructors, lowering wrappers and statement variants.
Generator, plain Async and AsyncGenerator already enter the source-bound common
`AsyncGeneratorForInIr` through the analyzed complete-owner token and its closed
execution tag. The removed variants therefore had no production producer.

The native emitter now consumes that common carrier directly. Its original
head TDZ, four invocation cells, lazy enumeration cursor, selected-key
initialization, per-iteration environment, completion value and cleanup order
remain on the same physical path. Dead IR/backend traversal and label arms,
scalar source facades and the obsolete bare-Block initialization constructor
are removed. Storage accepts only the initializer proof minted by actual source
lowering; the original AST identity, head classification, binding names,
ignored-Identifier Reference validation and environment checks remain consumed.

The original Generator and Async private control modules remain mounted under
the common carrier, retaining their test names and JavaScript source literals.
The eager ForIn enumeration and WebCompat invalid-Reference paths remain live.
ForOf carriers are outside this retirement because their remaining producer
paths have not yet been excluded by an enforced source invariant.

This is an authored source change. Compilation, tests and formatting remain
deferred to the combined verification checkpoint; no conformance or performance
result is claimed.
