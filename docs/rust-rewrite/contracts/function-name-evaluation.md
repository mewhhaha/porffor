# Function name evaluation

Owner: T09 (function and class evaluation), T15 (Wasm materialization).

The completed baseline exposed object accessors named `value` instead of
`get value`, and computed methods named `<method>` instead of the name derived
from their property key. The same missing operation affected anonymous values
under computed keys and public class accessors and methods.

`ComputedPropertyNameInferenceIr` distinguishes ordinary value evaluation,
anonymous function evaluation, and anonymous class evaluation. The producer
uses the AST's anonymous-definition predicate, including parenthesized
definitions. Explicit names, existing function references, and comma
expressions retain their names. Static literal property names already carried
by the parser continue through function metadata; statically foldable computed
keys still perform named evaluation. Unnamed arrows have the empty name.

The Wasm `emit_set_function_name` implementation consumes an already-evaluated
PropertyKey. String keys supply the name directly. Symbol keys supply the
description in brackets, with an undefined description producing the empty
name and an empty description producing `[]`. The typed prefix distinguishes
ordinary names from getter and setter names. The resulting own data property
is non-writable, non-enumerable, and configurable.

Object-method materialization requires the function, its HomeObject, and its
evaluated property key in one consuming request. It attaches HomeObject and
sets the function name before publishing the property. Public class methods
and accessors use the same name operation. Private names continue through their
existing source-derived metadata.

Anonymous class evaluation consumes the computed property's normalized key
through a compiler-private binding. It sets the constructor name before
evaluating static members; a static `name` field or method may therefore
replace it. When supported async class evaluation suspends, the key binding
belongs to the activation and is initialized before the class heritage. The
continuation retains that binding, so resumption does not repeat key coercion.
This does not introduce a class lexical name binding.

General object-literal suspension sequencing remains distinct from class
continuation naming. In particular, the existing staged-generator object
lowerer rejects computed properties. These checks do not establish support for
all object-literal combinations containing `yield` or `await`.

Focused verification commands:

```sh
cargo test --locked -p lila-ir --test function_names
cargo test --locked -p lila-aot-wasm --test object_literal_home_object_structure
cargo test --locked -p lila-aot-wasm --lib nested_object_properties_retain_values_across_function_name_materialization
cargo test --locked -p lila-engine --test aot_function_names -- --test-threads=1
```

The native target checks descriptor mutation, explicit-name controls, Symbol
descriptions, accessor prefixes, class static initialization, abrupt key
coercion, and an async class continuation. Test262 replay evidence belongs to
the completed-baseline follow-up report rather than this implementation
contract.
