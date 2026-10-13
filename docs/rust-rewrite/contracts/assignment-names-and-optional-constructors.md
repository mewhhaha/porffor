# Assignment names and optional constructor grammar

Authority: ECMA-262 `5345883164f463e87f8b40aca4956157ecba8783`,
AssignmentExpression evaluation for `&&=`, `||=` and `??=`, IsIdentifierRef,
IsAnonymousFunctionDefinition, NamedEvaluation and the NewExpression grammar.

The historical actual `aee6bc109` compiler fails both original modes of
`staging/sm/expressions/short-circuit-compound-assignment-anon-fns.js`.
Bare `a` targets infer `a`; grouped `(a)` targets must leave an anonymous
function's name empty. The parser checks its converted AssignTarget, which
erases grouping, and consequently infers the name for both shapes.
Checking the original LeftHandSideExpression instead retains IsIdentifierRef.
The existing conditional RHS execution and ordinary function/class name
publication then handle evaluation; lowering needs no new name-inference path.
Controls retain the bare/grouped pair, all three operators, strict/sloppy code,
explicit names, property targets, skipped RHS effects and async/generator/with
reference routes.

The same actual compiler fails
`sloppy-script:staging/sm/expressions/optional-chain.js`: invalid constructor
syntax reaches execution and throws TypeError instead of SyntaxError.
The recursive MemberExpression parser must reject `?.` immediately after an
unparenthesized constructor target, before parsing its argument list.
`new object?.Constructor()` and `new Constructor?.()` are invalid;
`new (object?.Constructor)()` and `new Constructor()?.property` are valid.
Paired Script/Module frontend controls preserve these grammar boundaries,
including optional expressions in arguments and grouped constructor targets.

No original fixture is modified. Both repairs are prepared source; native and
original-execution acceptance must follow the coherent batch build. All prior
passing IDs and unsupported dynamic-source executions remain in the inventory.
