# Mixed literal, class and optional evaluation

The checked FunctionBody source allocator owns both explicit Await and Yield
points. A closed operand protocol feeds the existing ArrayAccumulation,
ObjectPropertyDefinition and ClassDefinitionEvaluation lowering owners. Array
prefixes drain before the next suspension, object computed names complete
ToPropertyKey before property values, and class heritage/computed-name prefixes
retain the original class-name and constructor environments.

Every suspended optional key or argument uses one checked mixed If carrier.
Its skipped branch has no suspension or operand evaluation; its selected branch
contains the complete original source operand. The same physical optional-link
loop retains GetValue, property-call receivers, argument spreads and shorted
suffix state. Grouped outer calls retain the terminal property Reference while
their arguments still evaluate when the inner chain shorts.

Delete consumes the terminal property Reference directly, including computed
Await keys and shorted optional chains. It never acquires a terminal GetValue.
The original Delete and Call operations retain strictness, coercion order,
receiver identity and whole abrupt completions. Foreign iterator/resource
source domains retain their original admission boundaries.

The original Super receiver gate runs before a suspended computed key. Super
reads, private `in` tests and numeric updates retain completed raw operands and
consume the original Reference operations. Number/BigInt coercion, private
brands, receiver identity and update return modes stay with those owners. A
sloppy Annex B call-update completes the same CallEvaluation before its original
ReferenceError; a thrown call keeps its whole abrupt completion. Private optional
links consume the original private Get/brand operation and retain the same
grouped-call receiver. Suspended Super delete retains the raw key and consumes
the original uncoerced ReferenceError operation. Import stages both raw operands
before handing them to the existing capability/module-job operation; its later
ToString and options validation retain their original rejection behavior.

Eight authored IR controls check mixed tapes, retained literal storage, class
prefix/name ownership, skipped/selected optional ranges and terminal Delete or
grouped-call roles. Engine cohorts cover prefix/spread/elision order, caller
mutation and GC, inferred/explicit class names, lazy skipped tails, receiver
identity, no-Get deletion, raw key coercion and queued Return through yielding
finally, plus private/Super reads and numeric/Annex B updates. All controls remain
unrun until the complete dry-coding batch is ready.
