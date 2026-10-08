# Suspended ordinary-generator object literals

The compiler creates one ordinary native object before evaluating its first
property. Its private activation binding retains that same object across every
yield. Construction carries no complete heap-shape proof across caller code.

Each property follows source order: evaluate its computed key, complete
`ToPropertyKey`, evaluate its value, then define the property before starting the
next property. The normalized key and raw value use existing activation-owned
bindings. Spread copies source properties immediately, before a later yield.

Both ordinary and suspended construction borrow the original AST property and
consume the same private lowering owner. Method function identities therefore
remain connected to analysis. One native property emitter owns prototype
setters, data properties, spreads, function names, HomeObject, and accessor
merging for both paths. `ObjectPropertyDefinitionIr` admits only an exact Object
target and binds that target to one actual `ObjectPropertyIr`.

For an anonymous class under a computed property, NamedEvaluation consumes the
already normalized key before the class's computed element names or static
initialization. The prepared class value is substituted into the shared property
owner; its evaluation and naming do not replay at property definition.
The private prepared-class carrier binds this evaluated value to its original
computed property. A generic async operand pin cannot mint that naming proof.

Literal `__proto__` changes the prototype only for an object or null value.
Computed and shorthand `__proto__` are ordinary data properties. Methods retain
the actual constructed object as HomeObject, including after a resumed key.
Getter and setter halves share ordinary accessor-definition semantics.

The existing whole Completion path handles key conversion, value evaluation and
property-definition throws, plus injected generator Return and Throw. Yielding
finally blocks preserve the pending completion, and abandoned construction does
not start later property evaluation. Native temporary roots use the same cleanup
as ordinary construction; suspended operands live in the existing invocation
environment.

Two IR controls inspect actual source-order property owners and captured class
name transport. One real Wasm AOT cohort covers strict and sloppy execution,
single key coercion, Symbol names, property order, prototype forms, accessor
merging, HomeObject receivers, GC between resumes, class static naming, and whole
abrupt completions through yielding finally blocks. These controls are authored;
compilation and execution remain mandatory acceptance work.
