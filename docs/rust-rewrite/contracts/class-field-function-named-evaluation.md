# Class field function NamedEvaluation

Status: source-prepared on 2026-10-14; native acceptance is pending.

Authority is ECMA-262 commit `5345883164f463e87f8b40aca4956157ecba8783`:
[HasName](https://tc39.es/ecma262/#sec-static-semantics-hasname),
[IsAnonymousFunctionDefinition](https://tc39.es/ecma262/#sec-isanonymousfunctiondefinition),
[NamedEvaluation](https://tc39.es/ecma262/#sec-runtime-semantics-namedevaluation),
[EvaluateBody](https://tc39.es/ecma262/#sec-runtime-semantics-evaluatebody) for
class field initializer functions, and
[SetFunctionName](https://tc39.es/ecma262/#sec-setfunctionname).

The original historical `staging/sm/Function/function-name-computed-01.js` and
`function-name-computed-02.js` execute in both script modes. Their object arrow
properties already receive the correct names at source HEAD `115279841`;
the identically worded later class-field assertions expose missing field
NamedEvaluation. The actual source-bound probe records `object:arrowFunc,
asyncArrowFunc` followed by `field:,`. A failure message alone does not identify
which repeated assertion in a source file failed.

Class field initializer lowering previously special-cased anonymous classes.
All function and arrow initializers followed ordinary expression evaluation,
so computed fields discarded their original key when allocating the callable.
Parser-inferred literal/private labels cannot express a runtime computed key.

The field owner now uses source semantics to identify anonymous function
definitions. Arrow and async arrow syntax cannot declare a binding name.
Ordinary, generator, async and async generator expressions use their explicit
`has_binding_identifier` flag; an inferred parser display name does not block
NamedEvaluation. Parentheses forward this classification. Comma expressions
and references do not, and explicitly named functions retain their source name.
Class expressions retain their existing creation-time protocol unchanged.

`ClassFieldNamedFunctionValue` carries a source callable identity and the
original `ClassFieldNameIr`: a static/private String or one retained computed
key slot. The backend reads the initializer's rooted class context, allocates
the source callable once, applies SetFunctionName with that String/Symbol, then
returns it. Both class and function names consume one shared key reader.
The computed key was evaluated and converted during class definition; it is
never converted from the original object again per instance or per name.

SetFunctionName retains the existing configurable, non-enumerable, non-writable
name descriptor and symbol-description behavior. Field definition, initializer
ordering, lexical this, private environments and abrupt key completion keep
their existing owners. No synthetic object allocation or source-text rewriting
is introduced. Unsupported source functions retain their diagnostic and never
become named callable values through this path.

One IR control checks original static/private/computed keys, all function
protocols, grouped syntax and explicit/sequence/reference exclusions. Two new
native controls execute both script modes and check string/numeric/symbol names,
descriptors, lexical this, fresh callable identity, cached key effects, static
ordering and abrupt completion. Existing function-name and class-field controls
must retain every prior pass. The four original historical execution IDs remain
in the denominator and require unmodified replay for acceptance.

This patch does not close a language task or full-suite acceptance.
