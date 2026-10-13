# Annex B arguments binding time

Authority: ECMA-262 `5345883164f463e87f8b40aca4956157ecba8783`,
FunctionDeclarationInstantiation's normative-optional web-compat insertion and
alternative function declaration evaluation, and Declarative Environment Record
SetMutableBinding. Lila implements Annex B web-legacy block functions.

The algorithm excludes `arguments` from new var binding creation at entry;
it does not exclude an eligible block function's evaluation-time assignment.
A formal parameter with the same name still blocks this Annex B copy.
With simple parameters the copy updates the implicit arguments binding.
With parameter expressions the body VariableEnvironment is separate: before
the block declaration a read resolves to parameter arguments; when evaluated,
sloppy SetMutableBinding creates the body binding if it is absent. Parameter
initializer closures keep the original parameter environment, while body
closures observe the newly created binding through their live lexical chain.
An arrow's own variable environment follows the same late-creation rule.

The historical actual `aee6bc109` compiler records Bugs in
`sloppy-script:staging/sm/lexical-environment/block-scoped-functions-annex-b-arguments.js`
and `sloppy-script:staging/sm/regress/regress-602621.js`. Analysis previously
blocked the implicit binding as though it were a formal parameter.
The repair separates copy eligibility from static variable creation, preserves
real independent declarations, and adds an exhaustive function-variable target.
That target uses existing named-record SetMutableBinding at declaration
evaluation. Units containing an arguments function candidate retain named
environment visibility, so reads/writes and escaping closures resolve the live
binding. No source rewrite or fixture exception is used.

The existing IR control keeps both original inputs and its ID, correcting its
implicit-arguments expectation to the normative copy. New controls retain
formal and lexical collisions, unselected blocks, pre/post identity, parameter
initializer reads/writes and closures, and an arrow's outer arguments identity.
No native acceptance claim is made before the coherent batch compiles and the
unchanged original executions and complete affected inventories are checked.
