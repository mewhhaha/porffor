# Anonymous class field NamedEvaluation

This source batch supplies the original field name while evaluating an
anonymous class initializer. Numeric and BigInt property names use the same
normalized field-key cache as other computed names. A nested class receives
its name before its computed elements and static initialization, so a static
block observes the inferred name and may replace or delete it afterward.

`ClassNameInferenceIr` replaces an optional binding string with three closed
cases: no inference, a computed object property's retained binding, or a field
initializer's `ClassFieldNameIr`. The field name is either a static String or
the index of an already-normalized key in the initializer's immutable class
context. Each consumer matches the complete domain. The cache retains Symbol
identity, and the existing SetFunctionName emitter derives its description.
Static private names preserve their `#` description. No receiver property or
source-text label supplies the name, and no user key coercion runs again.

The parser may attach an inferred label to `ClassExpression.name()` for a
literal or private field. The actual `name_scope()` distinguishes an explicit
source binding identifier from that label. Lowering unwraps parentheses and
admits only a ClassExpression without that scope; a named class, comma result,
reference or other expression follows ordinary expression lowering. Existing
parser-derived class binding plans are retained by this bounded batch.

The primary specification establishes the initializer's original name in
[ClassFieldDefinitionEvaluation](https://tc39.es/ecma262/multipage/ecmascript-language-functions-and-classes.html#sec-runtime-semantics-classfielddefinitionevaluation).
[EvaluateBody](https://tc39.es/ecma262/multipage/ordinary-and-exotic-objects-behaviours.html#sec-runtime-semantics-evaluatebody)
selects NamedEvaluation for an anonymous definition.
[IsAnonymousFunctionDefinition and NamedEvaluation](https://tc39.es/ecma262/multipage/syntax-directed-operations.html#sec-runtime-semantics-namedevaluation)
preserve grouping and distinguish explicit binding identifiers; class
NamedEvaluation supplies the name during ClassDefinitionEvaluation.
[SetFunctionName](https://tc39.es/ecma262/2024/multipage/ordinary-and-exotic-objects-behaviours.html#sec-setfunctionname)
defines the name descriptor and Symbol/private-name spelling.

The new Engine target embeds the complete unchanged pinned
`staging/sm/fields/numeric-fields.js`, plus the original `sta.js` and `assert.js`
harnesses. All four tests execute both Script modes when run. Controls cover
numeric/BigInt canonicalization, static/instance/private names and descriptors,
multiple instances after poisoning the key coercion method, Symbol
descriptions, abrupt key conversion, parenthesized anonymous definitions,
explicit class names, references and comma expressions, declaration effects,
inheritance, and static name replacement/deletion/method override. The IR
target pins the original normalized slot ownership and literal/private
authority while preserving its existing computed object and suspension tests.

Compilation and all runtime verification remain unrun in this source packet.
It does not change function-valued field initializers, constructor protocols,
class binding analysis, private-element installation, the Wasm feature target,
or published suite counts, and it does not close the class/private subtree.

Required integration checks are `cargo xc`, the complete `function_names` IR
target, the complete `aot_class_field_named_evaluation` Engine target, adjacent
`aot_function_names` and `aot_public_class_fields` targets, the unchanged pinned
numeric-fields test in both modes, then the joined batch's broad verification.
